//! Memory OS lint service.

use crate::error::IndexResult;
use engram_core::lint::{LintFinding, LintReport, LintRule, LintSafeAction, LintSeverity};
use engram_core::memory::{MemoryItem, MemoryKind, MemoryScope, MemoryStatus};
use engram_core::obligation::AgentObligationStatus;
use engram_core::session::SessionStatus;
use engram_store::{Db, MemoryRepo, ObligationRepo, SessionRepo};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use time::{Duration, OffsetDateTime};

const VAULT_MARKER: &str = "<!-- engram:generated:file memory-vault-v1 -->";
const GENERATED_VAULT_PREFIXES: &[&str] = &[
    "99_System",
    "memory",
    "entities",
    "projects",
    "repositories",
];
const MAX_DUPLICATE_ENTITY_IDS_IN_FINDING: usize = 8;

/// Options for lint execution.
#[derive(Debug, Clone, Default)]
pub struct LintOptions {
    /// Optional vault root to scan.
    pub vault_path: Option<String>,
    /// Maximum findings to return.
    pub limit: Option<usize>,
}

/// Service for Memory OS health checks and safe remediation.
#[derive(Clone)]
pub struct LintService {
    memory_repo: MemoryRepo,
    session_repo: SessionRepo,
    obligation_repo: ObligationRepo,
}

impl LintService {
    /// Create a lint service.
    pub fn new(db: Db) -> Self {
        Self {
            memory_repo: MemoryRepo::new(db.clone()),
            session_repo: SessionRepo::new(db.clone()),
            obligation_repo: ObligationRepo::new(db),
        }
    }

    /// Initialize schemas needed for linting.
    pub async fn init_schema(&self) -> IndexResult<()> {
        self.memory_repo.init_schema().await?;
        self.session_repo.init_schema().await?;
        self.obligation_repo.init_schema().await?;
        Ok(())
    }

    /// Run all MVP lint rules.
    pub async fn run(&self, options: LintOptions) -> IndexResult<LintReport> {
        let mut findings = Vec::new();
        let items = self.memory_repo.list_memory_items(None, None).await?;

        lint_missing_evidence(&items, &mut findings);
        lint_stale_preferences(&items, &mut findings);
        lint_duplicate_entities(&items, &mut findings);
        lint_orphan_project_subprojects(&items, &mut findings);
        lint_superseded_active_items(&items, &mut findings);
        lint_handoffs_missing_next_actions(&items, &mut findings);
        self.lint_stale_active_sessions(&mut findings).await?;
        self.lint_open_obligations(&mut findings).await?;

        if let Some(vault_path) = options.vault_path {
            lint_vault_pages(Path::new(&vault_path), &mut findings)?;
        }

        findings.sort_by(|left, right| left.id.cmp(&right.id));
        if let Some(limit) = options.limit {
            findings.truncate(limit);
        }

        Ok(LintReport::new(findings))
    }

    /// Apply safe actions for findings. Currently only archives active items that
    /// are superseded by another memory item.
    pub async fn apply_safe(&self, options: LintOptions) -> IndexResult<LintReport> {
        let mut report = self.run(options).await?;
        let mut applied = 0;
        for finding in &report.findings {
            if finding.safe_action != LintSafeAction::ArchiveMemoryItem {
                continue;
            }
            let Some(item_id) = finding.item_id else {
                continue;
            };
            let Some(item) = self.memory_repo.get_memory_item(&item_id).await? else {
                continue;
            };
            if item.status != MemoryStatus::Active {
                continue;
            }
            let item = item.with_archive(
                format!("Archived by lint safe action for {}", finding.rule),
                Some("engram_lint".to_string()),
            );
            self.memory_repo.save_memory_item(&item).await?;
            applied += 1;
        }
        report.applied_safe_actions = applied;
        Ok(report)
    }

    async fn lint_stale_active_sessions(&self, findings: &mut Vec<LintFinding>) -> IndexResult<()> {
        let sessions = self
            .session_repo
            .list_sessions(Some(&SessionStatus::Active), None, None, None)
            .await?;
        let stale_before = OffsetDateTime::now_utc() - Duration::days(1);
        for session in sessions
            .into_iter()
            .filter(|session| session.started_at < stale_before)
        {
            findings.push(
                LintFinding::new(
                    format!("stale-active-session:{}", session.id),
                    LintRule::StaleActiveSession,
                    LintSeverity::Warning,
                    "Active session appears stale",
                    "Session has been active for more than one day; consider ending or abandoning it.",
                )
                .with_session(session.id),
            );
        }
        Ok(())
    }

    async fn lint_open_obligations(&self, findings: &mut Vec<LintFinding>) -> IndexResult<()> {
        let obligations = self
            .obligation_repo
            .list_obligations(Some(AgentObligationStatus::Open), None)
            .await?;
        for obligation in obligations {
            findings.push(
                LintFinding::new(
                    format!("unresolved-agent-obligation:{}", obligation.id),
                    LintRule::UnresolvedAgentObligation,
                    LintSeverity::Warning,
                    "Agent obligation is still open",
                    format!(
                        "Open obligation '{}' ({}) should be resolved or explicitly skipped before final response.",
                        obligation.title, obligation.kind
                    ),
                )
                .with_obligation(obligation.id),
            );
        }
        Ok(())
    }
}

fn lint_missing_evidence(items: &[MemoryItem], findings: &mut Vec<LintFinding>) {
    for item in items.iter().filter(|item| {
        matches!(
            item.status,
            MemoryStatus::Active | MemoryStatus::NeedsReview
        ) && item.evidence.is_empty()
    }) {
        findings.push(
            LintFinding::new(
                format!("missing-evidence:{}", item.id),
                LintRule::MissingEvidence,
                LintSeverity::Warning,
                "Memory item has no evidence",
                format!(
                    "Memory item '{}' ({}) has no evidence; durable memory should point to \
                     source evidence, review, or observed context.",
                    item.title, item.kind
                ),
            )
            .with_item(item.id),
        );
    }
}

fn lint_stale_preferences(items: &[MemoryItem], findings: &mut Vec<LintFinding>) {
    let now = OffsetDateTime::now_utc();
    let stale_before = now - Duration::days(120);
    for item in items.iter().filter(|item| {
        item.status == MemoryStatus::Active
            && item.kind == MemoryKind::Preference
            && (item
                .review_after
                .is_some_and(|review_after| review_after <= now)
                || (item.last_used_at.is_none() && item.created_at < stale_before))
    }) {
        findings.push(
            LintFinding::new(
                format!("stale-preference:{}", item.id),
                LintRule::StalePreference,
                LintSeverity::Info,
                "Preference should be recalibrated",
                "Preference is old or review_after has passed; ask whether it still applies.",
            )
            .with_item(item.id),
        );
    }
}

fn lint_duplicate_entities(items: &[MemoryItem], findings: &mut Vec<LintFinding>) {
    let mut groups: HashMap<String, Vec<&MemoryItem>> = HashMap::new();
    for item in items
        .iter()
        .filter(|item| item.status == MemoryStatus::Active)
    {
        if let MemoryScope::Entity { entity_name, .. } = &item.scope {
            groups
                .entry(entity_name.to_lowercase())
                .or_default()
                .push(item);
        }
    }

    for (entity_name, group) in groups.into_iter().filter(|(_, group)| group.len() > 1) {
        let total = group.len();
        let displayed_ids = group
            .iter()
            .take(MAX_DUPLICATE_ENTITY_IDS_IN_FINDING)
            .map(|item| item.id.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let omitted = total.saturating_sub(MAX_DUPLICATE_ENTITY_IDS_IN_FINDING);
        let item_ids = if omitted == 0 {
            displayed_ids
        } else {
            format!("{displayed_ids}, ... ({omitted} more)")
        };
        findings.push(LintFinding::new(
            format!("duplicate-entity-candidate:{entity_name}"),
            LintRule::DuplicateEntityCandidate,
            LintSeverity::Info,
            "Duplicate entity memory candidate",
            format!(
                "Multiple active memory items target entity '{}': {} active items: {}.",
                entity_name, total, item_ids
            ),
        ));
    }
}

fn lint_orphan_project_subprojects(items: &[MemoryItem], findings: &mut Vec<LintFinding>) {
    for item in items
        .iter()
        .filter(|item| item.status == MemoryStatus::Active)
    {
        match &item.scope {
            MemoryScope::Task {
                project_name: None, ..
            } => findings.push(
                LintFinding::new(
                    format!("orphan-task:{}", item.id),
                    LintRule::OrphanProjectSubproject,
                    LintSeverity::Warning,
                    "Task memory lacks parent project",
                    "Task-scoped memory should include a parent project name when known.",
                )
                .with_item(item.id),
            ),
            MemoryScope::Project { project_name, .. } if project_name.trim().is_empty() => {
                findings.push(
                    LintFinding::new(
                        format!("orphan-project:{}", item.id),
                        LintRule::OrphanProjectSubproject,
                        LintSeverity::Warning,
                        "Project memory lacks project name",
                        "Project-scoped memory must have a stable project name.",
                    )
                    .with_item(item.id),
                );
            }
            _ => {}
        }
    }
}

fn lint_superseded_active_items(items: &[MemoryItem], findings: &mut Vec<LintFinding>) {
    let superseded: HashSet<_> = items
        .iter()
        .filter(|item| item.status == MemoryStatus::Active)
        .flat_map(|item| item.supersedes.iter().copied())
        .collect();

    for item in items
        .iter()
        .filter(|item| item.status == MemoryStatus::Active && superseded.contains(&item.id))
    {
        findings.push(
            LintFinding::new(
                format!("superseded-active:{}", item.id),
                LintRule::SupersededItemStillActive,
                LintSeverity::Warning,
                "Superseded item is still active",
                "Another active memory item supersedes this item; archive it after review.",
            )
            .with_item(item.id)
            .with_safe_action(LintSafeAction::ArchiveMemoryItem),
        );
    }
}

fn lint_handoffs_missing_next_actions(items: &[MemoryItem], findings: &mut Vec<LintFinding>) {
    for item in items.iter().filter(|item| {
        item.status == MemoryStatus::Active
            && item.kind == MemoryKind::Handoff
            && !has_next_actions(&item.content)
    }) {
        findings.push(
            LintFinding::new(
                format!("handoff-missing-next-actions:{}", item.id),
                LintRule::HandoffMissingNextActions,
                LintSeverity::Warning,
                "Handoff is missing next actions",
                format!(
                    "Handoff '{}' is missing next actions; rolling handoffs should include \
                     concrete next actions for future agents.",
                    item.title
                ),
            )
            .with_item(item.id),
        );
    }
}

fn has_next_actions(content: &str) -> bool {
    let lower = content.to_lowercase();
    lower.contains("next action")
        || lower.contains("next actions")
        || lower.contains("next step")
        || lower.contains("next steps")
}

fn lint_vault_pages(root: &Path, findings: &mut Vec<LintFinding>) -> IndexResult<()> {
    if !root.exists() {
        return Ok(());
    }

    fn visit(root: &Path, path: &Path, findings: &mut Vec<LintFinding>) -> IndexResult<()> {
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                visit(root, &entry?.path(), findings)?;
            }
            return Ok(());
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            return Ok(());
        }
        if !is_generated_vault_path(root, path) {
            return Ok(());
        }

        let contents = fs::read_to_string(path)?;
        if !contents.contains(VAULT_MARKER) || !contents.starts_with("---\n") {
            findings.push(
                LintFinding::new(
                    format!("vault-page-metadata:{}", path.display()),
                    LintRule::VaultPageMissingMarkerFrontmatter,
                    LintSeverity::Warning,
                    "Vault page is missing generated marker or frontmatter",
                    "Generated Memory OS vault pages should include frontmatter and Engram's generated marker.",
                )
                .with_path(path.display().to_string()),
            );
        }
        Ok(())
    }

    visit(root, root, findings)
}

fn is_generated_vault_path(root: &Path, path: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(root) else {
        return false;
    };
    let Some(first) = relative.components().next() else {
        return false;
    };
    let first = first.as_os_str().to_string_lossy();
    GENERATED_VAULT_PREFIXES
        .iter()
        .any(|prefix| first == *prefix)
}

#[cfg(test)]
mod tests {
    use super::*;
    use engram_core::memory::{
        ClaimOrigin, EvidenceKind, EvidenceRef, Harness, MemoryKind, MemoryScope, ModelIdentity,
        WriterProvenance,
    };
    use engram_core::obligation::{AgentObligation, AgentObligationKind, AgentObligationTrigger};
    use engram_store::{connect_and_init, StoreConfig};

    async fn service() -> LintService {
        let db = connect_and_init(&StoreConfig::memory()).await.unwrap();
        let service = LintService::new(db);
        service.init_schema().await.unwrap();
        service
    }

    fn writer() -> WriterProvenance {
        WriterProvenance::agent(Harness::Codex, ModelIdentity::new("openai", "gpt-5.5"))
    }

    #[tokio::test]
    async fn lint_reports_missing_evidence() {
        let service = service().await;
        let item = MemoryItem::new(
            MemoryKind::Decision,
            "No evidence",
            "A decision without evidence.",
            MemoryScope::project("engram"),
            ClaimOrigin::UserStated,
            writer(),
        );
        service.memory_repo.save_memory_item(&item).await.unwrap();

        let report = service.run(LintOptions::default()).await.unwrap();

        let finding = report
            .findings
            .iter()
            .find(|finding| finding.rule == LintRule::MissingEvidence)
            .unwrap();
        assert!(finding.message.contains("No evidence"));
        assert!(finding.message.contains("decision"));
    }

    #[tokio::test]
    async fn lint_reports_handoff_titles_when_next_actions_are_missing() {
        let service = service().await;
        let item = MemoryItem::new(
            MemoryKind::Handoff,
            "Incomplete handoff",
            "Useful context without an explicit action list.",
            MemoryScope::project("engram"),
            ClaimOrigin::AgentObserved,
            writer(),
        )
        .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "test"));
        service.memory_repo.save_memory_item(&item).await.unwrap();

        let report = service.run(LintOptions::default()).await.unwrap();

        let finding = report
            .findings
            .iter()
            .find(|finding| finding.rule == LintRule::HandoffMissingNextActions)
            .unwrap();
        assert!(finding.message.contains("Incomplete handoff"));
    }

    #[tokio::test]
    async fn lint_bounds_duplicate_entity_candidate_messages() {
        let service = service().await;
        let mut item_ids = Vec::new();
        for index in 0..10 {
            let item = MemoryItem::new(
                MemoryKind::ProjectFact,
                format!("Duplicate {index}"),
                "Duplicate entity-scoped content.",
                MemoryScope::entity("ide-mcp-eval"),
                ClaimOrigin::AgentObserved,
                writer(),
            )
            .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "test"));
            item_ids.push(item.id);
            service.memory_repo.save_memory_item(&item).await.unwrap();
        }

        let report = service.run(LintOptions::default()).await.unwrap();

        let finding = report
            .findings
            .iter()
            .find(|finding| finding.rule == LintRule::DuplicateEntityCandidate)
            .unwrap();
        let displayed_id_count = item_ids
            .iter()
            .filter(|item_id| finding.message.contains(&item_id.to_string()))
            .count();
        assert!(finding.message.contains("10 active items"));
        assert!(finding.message.contains("... (2 more)"));
        assert_eq!(displayed_id_count, MAX_DUPLICATE_ENTITY_IDS_IN_FINDING);
    }

    #[tokio::test]
    async fn lint_reports_open_agent_obligations() {
        let service = service().await;
        let obligation = AgentObligation::new(
            AgentObligationKind::SourceReading,
            "Read source before implementation",
            "Source reading is required before making changes.",
            MemoryScope::project("engram"),
            AgentObligationTrigger::new("prompt", "implementation request"),
            writer(),
        );
        service
            .obligation_repo
            .save_obligation(&obligation)
            .await
            .unwrap();

        let report = service.run(LintOptions::default()).await.unwrap();

        assert!(report.findings.iter().any(|finding| {
            finding.rule == LintRule::UnresolvedAgentObligation
                && finding.obligation_id == Some(obligation.id)
        }));
    }

    #[tokio::test]
    async fn lint_apply_safe_archives_superseded_active_item() {
        let service = service().await;
        let old = MemoryItem::new(
            MemoryKind::Decision,
            "Old decision",
            "Old content.",
            MemoryScope::project("engram"),
            ClaimOrigin::UserStated,
            writer(),
        )
        .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "test"));
        let new = MemoryItem::new(
            MemoryKind::Decision,
            "New decision",
            "New content.",
            MemoryScope::project("engram"),
            ClaimOrigin::UserStated,
            writer(),
        )
        .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "test"))
        .with_superseded_item(old.id);
        service.memory_repo.save_memory_item(&old).await.unwrap();
        service.memory_repo.save_memory_item(&new).await.unwrap();

        let report = service.apply_safe(LintOptions::default()).await.unwrap();
        let archived = service
            .memory_repo
            .get_memory_item(&old.id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(report.applied_safe_actions, 1);
        assert_eq!(archived.status, MemoryStatus::Archived);
        assert!(archived.archive.is_some());
    }

    #[tokio::test]
    async fn lint_reports_vault_page_missing_marker() {
        let service = service().await;
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("memory")).unwrap();
        fs::write(
            root.path().join("memory/index.md"),
            "# Broken generated page\n",
        )
        .unwrap();

        let report = service
            .run(LintOptions {
                vault_path: Some(root.path().display().to_string()),
                limit: None,
            })
            .await
            .unwrap();

        assert!(report
            .findings
            .iter()
            .any(|finding| { finding.rule == LintRule::VaultPageMissingMarkerFrontmatter }));
    }

    #[tokio::test]
    async fn lint_ignores_user_notes_outside_generated_vault_paths() {
        let service = service().await;
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("notes.md"), "# User page\n").unwrap();

        let report = service
            .run(LintOptions {
                vault_path: Some(root.path().display().to_string()),
                limit: None,
            })
            .await
            .unwrap();

        assert!(!report
            .findings
            .iter()
            .any(|finding| finding.rule == LintRule::VaultPageMissingMarkerFrontmatter));
    }
}
