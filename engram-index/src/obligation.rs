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
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

const DOCUMENT_FINGERPRINT_PREFIX: &str = "document_content_fingerprint=git-sha1:";

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
    /// Candidate obligations skipped because an equivalent open obligation exists
    /// or the same document content was already resolved or skipped.
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
        project: Option<&str>,
        cwd: Option<&str>,
        limit: Option<usize>,
    ) -> IndexResult<Vec<AgentObligation>> {
        if project.is_none() && cwd.is_none() {
            return Ok(self.repo.list_obligations(status, limit).await?);
        }

        let mut obligations = self
            .repo
            .list_obligations(status, None)
            .await?
            .into_iter()
            .filter(|obligation| obligation_applies_to_context(obligation, project, cwd))
            .collect::<Vec<_>>();
        if let Some(limit) = limit {
            obligations.truncate(limit);
        }
        Ok(obligations)
    }

    /// List open obligations that apply to the current project/cwd context.
    pub async fn list_open_for_context(
        &self,
        project: Option<&str>,
        cwd: Option<&str>,
    ) -> IndexResult<Vec<AgentObligation>> {
        let mut obligations = self
            .repo
            .list_obligations(Some(AgentObligationStatus::Open), None)
            .await?
            .into_iter()
            .filter(|obligation| obligation_applies_to_context(obligation, project, cwd))
            .filter(|obligation| obligation_is_current_for_context(obligation, cwd))
            .collect::<Vec<_>>();
        obligations.sort_by(|left, right| {
            right
                .updated_at
                .cmp(&left.updated_at)
                .then_with(|| left.title.cmp(&right.title))
        });
        Ok(obligations)
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
            let closed_document_fingerprints =
                self.closed_document_fingerprints_by_dedupe_key().await?;

            for candidate in candidates.iter().cloned() {
                if existing_keys.contains(&candidate.dedupe_key()) {
                    skipped_existing.push(candidate);
                    continue;
                }
                if closed_document_fingerprints_match(&candidate, &closed_document_fingerprints) {
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
        obligation.resolve(with_current_document_fingerprint(&obligation, resolution));
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
        append_current_document_fingerprint_to_resolution(&mut obligation);
        self.repo.save_obligation(&obligation).await?;
        Ok(obligation)
    }

    /// Return open-obligation diagnostics.
    pub async fn doctor(
        &self,
        project: Option<&str>,
        cwd: Option<&str>,
        limit: Option<usize>,
    ) -> IndexResult<ObligationDoctorReport> {
        let mut open = if project.is_some() || cwd.is_some() {
            self.list_open_for_context(project, cwd).await?
        } else {
            self.repo
                .list_obligations(Some(AgentObligationStatus::Open), None)
                .await?
        };
        if let Some(limit) = limit {
            open.truncate(limit);
        }
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

    async fn closed_document_fingerprints_by_dedupe_key(
        &self,
    ) -> IndexResult<HashMap<String, HashSet<String>>> {
        let resolved = self
            .repo
            .list_obligations(Some(AgentObligationStatus::Resolved), None)
            .await?;
        let skipped = self
            .repo
            .list_obligations(Some(AgentObligationStatus::Skipped), None)
            .await?;
        let mut fingerprints_by_key: HashMap<String, HashSet<String>> = HashMap::new();

        for obligation in resolved.into_iter().chain(skipped) {
            if !is_git_status_document_obligation(&obligation) {
                continue;
            }
            if let Some(fingerprint) = resolution_document_fingerprint(&obligation) {
                fingerprints_by_key
                    .entry(obligation.dedupe_key())
                    .or_default()
                    .insert(fingerprint);
            }
        }

        Ok(fingerprints_by_key)
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

fn obligation_applies_to_context(
    obligation: &AgentObligation,
    project: Option<&str>,
    cwd: Option<&str>,
) -> bool {
    match &obligation.scope {
        MemoryScope::Global | MemoryScope::User => true,
        MemoryScope::Project { project_name, .. } => {
            project.is_some_and(|project| project_name.eq_ignore_ascii_case(project))
        }
        MemoryScope::Task { project_name, .. } => project_name
            .as_deref()
            .zip(project)
            .is_some_and(|(item_project, project)| item_project.eq_ignore_ascii_case(project)),
        MemoryScope::Repository { local_path, .. } => {
            let Some(cwd) = cwd else {
                return false;
            };
            local_path
                .as_deref()
                .is_some_and(|local_path| Path::new(cwd).starts_with(Path::new(local_path)))
        }
        MemoryScope::Custom { name } => cwd.is_some_and(|cwd| name == &format!("cwd:{cwd}")),
        MemoryScope::Entity { .. } | MemoryScope::Session { .. } => false,
    }
}

fn obligation_is_current_for_context(obligation: &AgentObligation, cwd: Option<&str>) -> bool {
    if !is_git_status_document_obligation(obligation) {
        return true;
    }

    let (Some(cwd), Some(target)) = (cwd, obligation.trigger.target.as_deref()) else {
        return true;
    };

    match current_git_status_target(cwd, target) {
        GitStatusTarget::Present(status_line) => {
            !is_untracked_root_instruction_file(&status_line, target)
        }
        GitStatusTarget::Missing => false,
        GitStatusTarget::Unavailable => true,
    }
}

fn is_git_status_document_obligation(obligation: &AgentObligation) -> bool {
    obligation.kind == AgentObligationKind::DocumentDisposition
        && obligation.trigger.kind == "git_status"
}

fn closed_document_fingerprints_match(
    candidate: &AgentObligation,
    closed_document_fingerprints: &HashMap<String, HashSet<String>>,
) -> bool {
    if !is_git_status_document_obligation(candidate) {
        return false;
    }

    let Some(candidate_fingerprint) = obligation_document_fingerprint(candidate) else {
        return false;
    };

    closed_document_fingerprints
        .get(&candidate.dedupe_key())
        .is_some_and(|fingerprints| fingerprints.contains(&candidate_fingerprint))
}

enum GitStatusTarget {
    Present(String),
    Missing,
    Unavailable,
}

fn current_git_status_target(cwd: &str, target: &str) -> GitStatusTarget {
    let Ok(output) = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .arg("status")
        .arg("--porcelain")
        .arg("--untracked-files=all")
        .arg("--")
        .arg(target)
        .output()
    else {
        return GitStatusTarget::Unavailable;
    };
    if !output.status.success() {
        return GitStatusTarget::Unavailable;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .find_map(|line| {
            let path = parse_git_status_path(line)?;
            if path == target {
                Some(line.to_string())
            } else {
                None
            }
        })
        .map_or(GitStatusTarget::Missing, GitStatusTarget::Present)
}

fn is_untracked_root_instruction_file(status_line: &str, target: &str) -> bool {
    status_line.starts_with("?? ") && is_root_instruction_file(target)
}

fn is_root_instruction_file(target: &str) -> bool {
    let normalized = target.replace('\\', "/");
    if normalized.contains('/') {
        return false;
    }

    matches!(
        normalized.to_ascii_lowercase().as_str(),
        "agents.md" | "claude.md" | "gemini.md"
    )
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
    let mut file_evidence = EvidenceRef::new(EvidenceKind::File, absolute.to_string_lossy());
    if let Some(fingerprint) = document_fingerprint(&absolute) {
        file_evidence =
            file_evidence.with_summary(format!("{DOCUMENT_FINGERPRINT_PREFIX}{fingerprint}"));
    }
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
    .with_evidence(file_evidence)
    .with_tag("document")
    .with_tag("agent-native")
}

fn with_current_document_fingerprint(
    obligation: &AgentObligation,
    mut resolution: AgentObligationResolution,
) -> AgentObligationResolution {
    if let Some(evidence) = current_document_fingerprint_evidence(obligation) {
        resolution.evidence.push(evidence);
    }
    resolution
}

fn append_current_document_fingerprint_to_resolution(obligation: &mut AgentObligation) {
    let Some(evidence) = current_document_fingerprint_evidence(obligation) else {
        return;
    };
    if let Some(resolution) = &mut obligation.resolution {
        resolution.evidence.push(evidence);
    }
}

fn current_document_fingerprint_evidence(obligation: &AgentObligation) -> Option<EvidenceRef> {
    if !is_git_status_document_obligation(obligation) {
        return None;
    }
    let target = document_file_evidence_target(obligation)?;
    let fingerprint = document_fingerprint(Path::new(&target))?;
    Some(
        EvidenceRef::new(EvidenceKind::File, target)
            .with_summary(format!("{DOCUMENT_FINGERPRINT_PREFIX}{fingerprint}")),
    )
}

fn document_file_evidence_target(obligation: &AgentObligation) -> Option<String> {
    obligation.evidence.iter().find_map(|evidence| {
        if matches!(&evidence.kind, EvidenceKind::File) {
            Some(evidence.target.clone())
        } else {
            None
        }
    })
}

fn obligation_document_fingerprint(obligation: &AgentObligation) -> Option<String> {
    obligation
        .evidence
        .iter()
        .find_map(evidence_document_fingerprint)
}

fn resolution_document_fingerprint(obligation: &AgentObligation) -> Option<String> {
    obligation
        .resolution
        .as_ref()?
        .evidence
        .iter()
        .find_map(evidence_document_fingerprint)
}

fn evidence_document_fingerprint(evidence: &EvidenceRef) -> Option<String> {
    if !matches!(&evidence.kind, EvidenceKind::File) {
        return None;
    }
    evidence
        .summary
        .as_deref()?
        .strip_prefix(DOCUMENT_FINGERPRINT_PREFIX)
        .map(ToString::to_string)
}

fn document_fingerprint(path: &Path) -> Option<String> {
    let output = Command::new("git")
        .arg("hash-object")
        .arg("--")
        .arg(path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let fingerprint = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!fingerprint.is_empty()).then_some(fingerprint)
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

    #[tokio::test]
    async fn write_detection_is_idempotent_for_closed_document_content() {
        let dir = tempfile::tempdir().unwrap();
        Command::new("git")
            .arg("init")
            .current_dir(dir.path())
            .output()
            .unwrap();
        fs::create_dir_all(dir.path().join("docs")).unwrap();
        let doc_path = dir.path().join("docs/report.md");
        fs::write(&doc_path, "# Report\n").unwrap();

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
        assert_eq!(first.written.len(), 1);
        assert!(obligation_document_fingerprint(&first.written[0]).is_some());

        let resolved = service
            .resolve(
                first.written[0].id,
                AgentObligationResolution::new(
                    AgentObligationResolutionKind::IndexedDocument,
                    "Indexed report.",
                    "agent",
                ),
            )
            .await
            .unwrap();
        assert!(resolution_document_fingerprint(&resolved).is_some());

        let same_content = service.detect(options()).await.unwrap();
        assert_eq!(same_content.written.len(), 0);
        assert_eq!(same_content.skipped_existing.len(), 1);

        fs::write(&doc_path, "# Report\n\nSecond edit.\n").unwrap();
        let changed_content = service.detect(options()).await.unwrap();
        assert_eq!(changed_content.written.len(), 1);
        assert_eq!(changed_content.skipped_existing.len(), 0);

        let skipped = service
            .skip(
                changed_content.written[0].id,
                "Document already recorded elsewhere.",
                "agent",
            )
            .await
            .unwrap();
        assert!(resolution_document_fingerprint(&skipped).is_some());

        let same_skipped_content = service.detect(options()).await.unwrap();
        assert_eq!(same_skipped_content.written.len(), 0);
        assert_eq!(same_skipped_content.skipped_existing.len(), 1);
    }
}
