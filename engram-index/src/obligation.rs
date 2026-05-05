//! Agent-native obligation service.

use crate::error::{IndexError, IndexResult};
use engram_core::memory::{
    EvidenceKind, EvidenceRef, Harness, MemoryScope, ModelIdentity, WriterProvenance,
};
use engram_core::obligation::{
    AgentObligation, AgentObligationKind, AgentObligationResolution, AgentObligationResolutionKind,
    AgentObligationStatus, AgentObligationTrigger,
};
use engram_core::Id;
use engram_store::{Db, ObligationRepo};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Options for detecting obligations from session cues.
#[derive(Debug, Clone)]
pub struct ObligationDetectOptions {
    /// Current working directory used for git-status document detection.
    pub cwd: Option<String>,
    /// Current prompt/task text used for reading and tool-failure cues.
    pub prompt: Option<String>,
    /// Project scope name for generated obligations.
    pub project: Option<String>,
    /// Writer provenance for generated obligations.
    pub writer: WriterProvenance,
    /// Whether to write generated obligations. Omitted/false is dry-run.
    pub write: bool,
    /// Maximum generated obligations.
    pub limit: Option<usize>,
}

/// Result of detecting obligations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObligationDetection {
    /// True when no obligations were written.
    pub dry_run: bool,
    /// Candidate obligations found.
    pub candidates: Vec<AgentObligation>,
    /// Obligations written during this run.
    pub written: Vec<AgentObligation>,
    /// Candidate obligations skipped because an equivalent open obligation exists.
    pub skipped_existing: Vec<AgentObligation>,
    /// Non-fatal warnings.
    pub warnings: Vec<String>,
}

/// Harness doctor report for obligations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObligationDoctorReport {
    /// Open obligations.
    pub open: Vec<AgentObligation>,
    /// Warnings for the agent/harness.
    pub warnings: Vec<String>,
}

/// Service for creating, listing, and resolving obligations.
#[derive(Clone)]
pub struct ObligationService {
    repo: ObligationRepo,
}

impl ObligationService {
    /// Create a service.
    #[must_use]
    pub fn new(db: Db) -> Self {
        Self {
            repo: ObligationRepo::new(db),
        }
    }

    /// Initialize schema.
    pub async fn init_schema(&self) -> IndexResult<()> {
        self.repo.init_schema().await?;
        Ok(())
    }

    /// Save an obligation.
    pub async fn add(&self, obligation: AgentObligation) -> IndexResult<AgentObligation> {
        self.repo.save_obligation(&obligation).await?;
        Ok(obligation)
    }

    /// Get one obligation.
    pub async fn get(&self, id: Id) -> IndexResult<Option<AgentObligation>> {
        Ok(self.repo.get_obligation(&id).await?)
    }

    /// List obligations.
    pub async fn list(
        &self,
        status: Option<AgentObligationStatus>,
        limit: Option<usize>,
    ) -> IndexResult<Vec<AgentObligation>> {
        Ok(self.repo.list_obligations(status, limit).await?)
    }

    /// Detect obligations from the current task/worktree.
    pub async fn detect(
        &self,
        options: ObligationDetectOptions,
    ) -> IndexResult<ObligationDetection> {
        let mut warnings = Vec::new();
        let mut candidates = Vec::new();
        let scope = obligation_scope(options.project.as_deref(), options.cwd.as_deref());

        if let Some(cwd) = &options.cwd {
            match detect_document_obligations(Path::new(cwd), scope.clone(), options.writer.clone())
            {
                Ok(mut docs) => candidates.append(&mut docs),
                Err(error) => {
                    warnings.push(format!("document obligation detection failed: {error}"))
                }
            }
        }

        if let Some(prompt) = &options.prompt {
            candidates.extend(detect_prompt_obligations(
                prompt,
                scope,
                options.writer.clone(),
            ));
        }

        dedupe_candidates(&mut candidates);
        if let Some(limit) = options.limit {
            candidates.truncate(limit);
        }

        let mut written = Vec::new();
        let mut skipped_existing = Vec::new();
        if options.write {
            let existing = self
                .repo
                .list_obligations(Some(AgentObligationStatus::Open), None)
                .await?;
            let mut existing_keys: HashSet<_> =
                existing.iter().map(AgentObligation::dedupe_key).collect();

            for candidate in candidates.iter().cloned() {
                if existing_keys.contains(&candidate.dedupe_key()) {
                    skipped_existing.push(candidate);
                    continue;
                }
                self.repo.save_obligation(&candidate).await?;
                existing_keys.insert(candidate.dedupe_key());
                written.push(candidate);
            }
        }

        Ok(ObligationDetection {
            dry_run: !options.write,
            candidates,
            written,
            skipped_existing,
            warnings,
        })
    }

    /// Resolve an obligation.
    pub async fn resolve(
        &self,
        id: Id,
        resolution: AgentObligationResolution,
    ) -> IndexResult<AgentObligation> {
        let mut obligation = self
            .repo
            .get_obligation(&id)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("obligation {id}")))?;
        obligation.resolve(resolution);
        self.repo.save_obligation(&obligation).await?;
        Ok(obligation)
    }

    /// Skip an obligation with a reason.
    pub async fn skip(
        &self,
        id: Id,
        reason: impl Into<String>,
        actor: impl Into<String>,
    ) -> IndexResult<AgentObligation> {
        let mut obligation = self
            .repo
            .get_obligation(&id)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("obligation {id}")))?;
        obligation.skip(reason, actor);
        self.repo.save_obligation(&obligation).await?;
        Ok(obligation)
    }

    /// Return open-obligation diagnostics.
    pub async fn doctor(&self, limit: Option<usize>) -> IndexResult<ObligationDoctorReport> {
        let open = self
            .repo
            .list_obligations(Some(AgentObligationStatus::Open), limit)
            .await?;
        let warnings = open
            .iter()
            .map(|obligation| {
                format!(
                    "Open obligation '{}' ({}) must be resolved or skipped before final response.",
                    obligation.title, obligation.kind
                )
            })
            .collect();
        Ok(ObligationDoctorReport { open, warnings })
    }
}

fn obligation_scope(project: Option<&str>, cwd: Option<&str>) -> MemoryScope {
    if let Some(project) = project {
        MemoryScope::project(project)
    } else if let Some(cwd) = cwd {
        MemoryScope::Custom {
            name: format!("cwd:{cwd}"),
        }
    } else {
        MemoryScope::Global
    }
}

fn detect_document_obligations(
    cwd: &Path,
    scope: MemoryScope,
    writer: WriterProvenance,
) -> IndexResult<Vec<AgentObligation>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .arg("status")
        .arg("--porcelain")
        .arg("--untracked-files=all")
        .output()?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut obligations = Vec::new();
    for line in stdout.lines() {
        if let Some(path) = parse_git_status_path(line) {
            if is_durable_doc_path(&path) {
                obligations.push(document_obligation(
                    cwd,
                    &path,
                    scope.clone(),
                    writer.clone(),
                ));
            }
        }
    }
    Ok(obligations)
}

fn parse_git_status_path(line: &str) -> Option<String> {
    if line.len() < 4 {
        return None;
    }
    let raw = line[3..].trim();
    let path = raw.split(" -> ").last().unwrap_or(raw).trim_matches('"');
    if path.is_empty() {
        None
    } else {
        Some(path.to_string())
    }
}

fn is_durable_doc_path(path: &str) -> bool {
    let lower = path.to_lowercase();
    let extension = Path::new(&lower)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    if !matches!(extension, "md" | "markdown" | "mdx" | "rst" | "txt") {
        return false;
    }

    extension == "md"
        || lower.contains("/docs/")
        || lower.starts_with("docs/")
        || lower.contains("guide")
        || lower.contains("plan")
        || lower.contains("design")
        || lower.contains("runbook")
        || lower.contains("adr")
        || lower.contains("eval")
        || lower.ends_with("readme.md")
}

fn document_obligation(
    cwd: &Path,
    path: &str,
    scope: MemoryScope,
    writer: WriterProvenance,
) -> AgentObligation {
    let absolute = absolutize(cwd, path);
    AgentObligation::new(
        AgentObligationKind::DocumentDisposition,
        format!("Resolve document memory status for {path}"),
        "A durable document changed in the worktree. Index it, register it, record a compact Memory OS item, link it in the handoff, or explicitly skip it with a reason.",
        scope,
        AgentObligationTrigger::new("git_status", "durable document changed")
            .with_target(path.to_string()),
        writer,
    )
    .with_required_resolution(AgentObligationResolutionKind::IndexedDocument)
    .with_required_resolution(AgentObligationResolutionKind::MemoryRecorded)
    .with_required_resolution(AgentObligationResolutionKind::KnowledgeRegistered)
    .with_required_resolution(AgentObligationResolutionKind::HandoffLinked)
    .with_required_resolution(AgentObligationResolutionKind::SkippedWithReason)
    .with_evidence(EvidenceRef::new(EvidenceKind::File, absolute.to_string_lossy()))
    .with_tag("document")
    .with_tag("agent-native")
}

fn detect_prompt_obligations(
    prompt: &str,
    scope: MemoryScope,
    writer: WriterProvenance,
) -> Vec<AgentObligation> {
    let lower = prompt.to_lowercase();
    let mut obligations = Vec::new();

    if contains_any(
        &lower,
        &[
            "implement",
            "change",
            "fix",
            "refactor",
            "explain",
            "behavior",
            "code",
            "software",
            "architecture",
            "design",
        ],
    ) {
        obligations.push(
            AgentObligation::new(
                AgentObligationKind::SourceReading,
                "Read relevant source before asserting behavior or changing code",
                "The task appears to involve code behavior or implementation. Read the actual source and local patterns before making claims or edits.",
                scope.clone(),
                AgentObligationTrigger::new("prompt", "task requires source-grounded code work"),
                writer.clone(),
            )
            .with_required_resolution(AgentObligationResolutionKind::SourceRead)
            .with_required_resolution(AgentObligationResolutionKind::SkippedWithReason)
            .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "prompt"))
            .with_tag("source-reading")
            .with_tag("agent-native"),
        );
    }

    if contains_any(
        &lower,
        &[
            "design",
            "architecture",
            "philosophy",
            "principle",
            "harness",
        ],
    ) {
        obligations.push(
            AgentObligation::new(
                AgentObligationKind::DesignContextReading,
                "Read project design context before architecture decisions",
                "The task appears to involve design or project philosophy. Read AGENTS, README, relevant docs, and existing design material before deciding.",
                scope.clone(),
                AgentObligationTrigger::new("prompt", "task requires design-context reading"),
                writer.clone(),
            )
            .with_required_resolution(AgentObligationResolutionKind::DesignContextRead)
            .with_required_resolution(AgentObligationResolutionKind::SkippedWithReason)
            .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "prompt"))
            .with_tag("design-context")
            .with_tag("agent-native"),
        );
    }

    if contains_any(
        &lower,
        &[
            "failed tool",
            "tool call failed",
            "wrong parameter",
            "wrong parameters",
            "invalid parameter",
            "schema",
        ],
    ) {
        obligations.push(
            AgentObligation::new(
                AgentObligationKind::ToolFailureRecovery,
                "Recover failed tool call instead of repeating wrong parameters",
                "A tool failure cue appeared. Inspect the tool schema/help, retry correctly if the action still matters, abandon explicitly if it does not, and record reusable gotchas when non-obvious.",
                scope.clone(),
                AgentObligationTrigger::new("prompt", "task mentions failed tool-call recovery"),
                writer.clone(),
            )
            .with_required_resolution(AgentObligationResolutionKind::RetriedTool)
            .with_required_resolution(AgentObligationResolutionKind::Abandoned)
            .with_required_resolution(AgentObligationResolutionKind::MemoryRecorded)
            .with_required_resolution(AgentObligationResolutionKind::SkippedWithReason)
            .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "prompt"))
            .with_tag("tool-failure")
            .with_tag("agent-native"),
        );
    }

    if contains_any(
        &lower,
        &[
            "commit",
            "git commit",
            "commit message",
            "committing",
            "create a commit",
        ],
    ) {
        obligations.push(
            AgentObligation::new(
                AgentObligationKind::CommitPreferenceCheck,
                "Check commit preferences before composing a commit",
                "The task appears to involve a commit workflow. Consult relevant user/project commit preferences, rules, and limitations before composing the commit message.",
                scope,
                AgentObligationTrigger::new("prompt", "task mentions commit workflow"),
                writer,
            )
            .with_required_resolution(AgentObligationResolutionKind::PreferenceChecked)
            .with_required_resolution(AgentObligationResolutionKind::SkippedWithReason)
            .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "prompt"))
            .with_tag("commit")
            .with_tag("agent-native"),
        );
    }

    obligations
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn dedupe_candidates(candidates: &mut Vec<AgentObligation>) {
    let mut seen = HashSet::new();
    candidates.retain(|candidate| seen.insert(candidate.dedupe_key()));
}

fn absolutize(cwd: &Path, path: &str) -> PathBuf {
    let path = Path::new(path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}

/// Build writer provenance from string fields.
#[must_use]
pub fn obligation_writer(
    writer_harness: Option<&str>,
    model_provider: Option<&str>,
    model: Option<&str>,
    surface: Option<&str>,
    actor: Option<&str>,
    session_id: Option<Id>,
) -> WriterProvenance {
    let harness = writer_harness
        .map(Harness::parse)
        .unwrap_or_else(|| Harness::Other("generic".to_string()));
    let model = ModelIdentity::new(
        model_provider.unwrap_or("unknown"),
        model.unwrap_or("unknown"),
    );
    let mut writer = WriterProvenance::agent(harness, model);
    writer.surface = surface.map(ToString::to_string);
    writer.actor = actor.unwrap_or("agent").to_string();
    writer.session_id = session_id;
    writer
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn writer() -> WriterProvenance {
        obligation_writer(
            Some("codex"),
            Some("openai"),
            Some("gpt-5.5"),
            Some("test"),
            Some("agent"),
            None,
        )
    }

    #[tokio::test]
    async fn prompt_detection_creates_source_design_and_tool_obligations() {
        let config = engram_store::StoreConfig::memory();
        let db = engram_store::connect_and_init(&config).await.unwrap();
        let service = ObligationService::new(db);
        service.init_schema().await.unwrap();

        let result = service
            .detect(ObligationDetectOptions {
                cwd: None,
                prompt: Some(
                    "Implement the design after a failed tool call due to wrong parameters, then commit it."
                        .to_string(),
                ),
                project: Some("engram".to_string()),
                writer: writer(),
                write: false,
                limit: None,
            })
            .await
            .unwrap();

        let kinds: Vec<_> = result
            .candidates
            .iter()
            .map(|obligation| obligation.kind.clone())
            .collect();
        assert!(kinds.contains(&AgentObligationKind::SourceReading));
        assert!(kinds.contains(&AgentObligationKind::DesignContextReading));
        assert!(kinds.contains(&AgentObligationKind::ToolFailureRecovery));
        assert!(kinds.contains(&AgentObligationKind::CommitPreferenceCheck));
        assert!(result.dry_run);
        assert!(result.written.is_empty());
    }

    #[tokio::test]
    async fn git_status_detects_document_obligation() {
        let dir = tempfile::tempdir().unwrap();
        Command::new("git")
            .arg("init")
            .current_dir(dir.path())
            .output()
            .unwrap();
        fs::create_dir_all(dir.path().join("docs")).unwrap();
        fs::write(dir.path().join("docs/IDE_MCP_EVAL_GUIDE.md"), "# Guide\n").unwrap();

        let config = engram_store::StoreConfig::memory();
        let db = engram_store::connect_and_init(&config).await.unwrap();
        let service = ObligationService::new(db);
        service.init_schema().await.unwrap();

        let result = service
            .detect(ObligationDetectOptions {
                cwd: Some(dir.path().display().to_string()),
                prompt: None,
                project: Some("engram".to_string()),
                writer: writer(),
                write: false,
                limit: None,
            })
            .await
            .unwrap();

        assert_eq!(result.candidates.len(), 1);
        assert_eq!(
            result.candidates[0].kind,
            AgentObligationKind::DocumentDisposition
        );
    }

    #[tokio::test]
    async fn write_detection_skips_duplicate_open_obligation() {
        let dir = tempfile::tempdir().unwrap();
        Command::new("git")
            .arg("init")
            .current_dir(dir.path())
            .output()
            .unwrap();
        fs::write(dir.path().join("PLAN.md"), "# Plan\n").unwrap();

        let config = engram_store::StoreConfig::memory();
        let db = engram_store::connect_and_init(&config).await.unwrap();
        let service = ObligationService::new(db);
        service.init_schema().await.unwrap();

        let options = || ObligationDetectOptions {
            cwd: Some(dir.path().display().to_string()),
            prompt: None,
            project: Some("engram".to_string()),
            writer: writer(),
            write: true,
            limit: None,
        };

        let first = service.detect(options()).await.unwrap();
        let second = service.detect(options()).await.unwrap();

        assert_eq!(first.written.len(), 1);
        assert_eq!(second.written.len(), 0);
        assert_eq!(second.skipped_existing.len(), 1);
    }
}
