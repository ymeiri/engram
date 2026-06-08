//! Memory OS lint service.

use crate::error::IndexResult;
use engram_core::lint::{LintFinding, LintReport, LintRule, LintSafeAction, LintSeverity};
use engram_core::memory::{MemoryItem, MemoryKind, MemoryScope, MemoryStatus};
use engram_core::obligation::AgentObligationStatus;
use engram_core::session::SessionStatus;
use engram_core::telemetry::AgentFeedback;
use engram_core::Id;
use engram_store::{Db, MemoryRepo, ObligationRepo, SessionRepo, TelemetryRepo};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use time::{Duration, OffsetDateTime};

const VAULT_MARKER: &str = "<!-- engram:generated:file memory-vault-v1 -->";
const CURRENT_PLAN_TAG: &str = "current-plan";
const GENERATED_VAULT_PREFIXES: &[&str] = &[
    "99_System",
    "memory",
    "entities",
    "projects",
    "repositories",
];
const MAX_DUPLICATE_ENTITY_IDS_IN_FINDING: usize = 8;
const MAX_FEEDBACK_ROWS_FOR_LINT: usize = 500;

/// Options for lint execution.
#[derive(Debug, Clone, Default)]
pub struct LintOptions {
    /// Optional project scope to lint. Keeps global/user memory and filters
    /// project-bound memory, sessions, and obligations to the requested project.
    pub project: Option<String>,
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
    telemetry_repo: TelemetryRepo,
}

impl LintService {
    /// Create a lint service.
    pub fn new(db: Db) -> Self {
        Self {
            memory_repo: MemoryRepo::new(db.clone()),
            session_repo: SessionRepo::new(db.clone()),
            obligation_repo: ObligationRepo::new(db.clone()),
            telemetry_repo: TelemetryRepo::new(db),
        }
    }

    /// Initialize schemas needed for linting.
    pub async fn init_schema(&self) -> IndexResult<()> {
        self.memory_repo.init_schema().await?;
        self.session_repo.init_schema().await?;
        self.obligation_repo.init_schema().await?;
        self.telemetry_repo.init_schema().await?;
        Ok(())
    }

    /// Run all MVP lint rules.
    pub async fn run(&self, options: LintOptions) -> IndexResult<LintReport> {
        let mut findings = Vec::new();
        let all_items = self.memory_repo.list_memory_items(None, None).await?;
        let project = options.project.as_deref();
        let items = filter_memory_items_for_project(&all_items, project);

        lint_missing_evidence(&items, &mut findings);
        lint_stale_preferences(&items, &mut findings);
        lint_duplicate_entities(&items, &mut findings);
        lint_orphan_project_subprojects(&items, &mut findings);
        lint_superseded_active_items(&items, &mut findings);
        lint_handoffs_missing_next_actions(&items, &mut findings);
        self.lint_stale_active_sessions(project, &mut findings)
            .await?;
        self.lint_open_obligations(project, &mut findings).await?;
        self.lint_feedback_flagged_active_memory(&items, &mut findings)
            .await?;

        if let Some(vault_path) = options.vault_path {
            lint_vault_pages(Path::new(&vault_path), &mut findings)?;
        }

        findings.sort_by(|left, right| {
            let priority = |finding: &LintFinding| match finding.rule {
                LintRule::FeedbackStaleCurrentPlan => 10,
                LintRule::FeedbackWrongScopeActiveMemory => 20,
                LintRule::FeedbackStaleActiveMemory => 30,
                LintRule::UnresolvedAgentObligation => 40,
                LintRule::SupersededItemStillActive
                    if finding.safe_action != LintSafeAction::None =>
                {
                    25
                }
                LintRule::MissingEvidence
                | LintRule::HandoffMissingNextActions
                | LintRule::OrphanProjectSubproject
                | LintRule::StaleActiveSession
                | LintRule::VaultPageMissingMarkerFrontmatter => 50,
                LintRule::StalePreference => 60,
                LintRule::DuplicateEntityCandidate => 90,
                LintRule::SupersededItemStillActive => 50,
            };
            priority(left)
                .cmp(&priority(right))
                .then_with(|| left.id.cmp(&right.id))
        });
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

    async fn lint_stale_active_sessions(
        &self,
        project: Option<&str>,
        findings: &mut Vec<LintFinding>,
    ) -> IndexResult<()> {
        let sessions = self
            .session_repo
            .list_sessions(Some(&SessionStatus::Active), None, project, None)
            .await?;
        let stale_before = OffsetDateTime::now_utc() - Duration::days(1);
        let now = OffsetDateTime::now_utc();
        for session in sessions
            .into_iter()
            .filter(|session| session.started_at < stale_before)
        {
            let age_hours = (now - session.started_at).whole_hours();
            let project = session.project.as_deref().unwrap_or("unknown");
            let agent = session.agent.as_deref().unwrap_or("unknown");
            let started_at = session
                .started_at
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_else(|_| session.started_at.to_string());
            findings.push(
                LintFinding::new(
                    format!("stale-active-session:{}", session.id),
                    LintRule::StaleActiveSession,
                    LintSeverity::Warning,
                    "Active session appears stale",
                    format!(
                        "Session has been active for more than one day \
                         (project: {project}, agent: {agent}, started_at: {started_at}, \
                         age_hours: {age_hours}); consider ending or abandoning it."
                    ),
                )
                .with_session(session.id),
            );
        }
        Ok(())
    }

    async fn lint_open_obligations(
        &self,
        project: Option<&str>,
        findings: &mut Vec<LintFinding>,
    ) -> IndexResult<()> {
        let obligations = self
            .obligation_repo
            .list_obligations(Some(AgentObligationStatus::Open), None)
            .await?;
        for obligation in obligations
            .into_iter()
            .filter(|obligation| scope_matches_project(&obligation.scope, project))
        {
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

    async fn lint_feedback_flagged_active_memory(
        &self,
        items: &[MemoryItem],
        findings: &mut Vec<LintFinding>,
    ) -> IndexResult<()> {
        let feedback = self
            .telemetry_repo
            .list_feedback(Some(MAX_FEEDBACK_ROWS_FOR_LINT))
            .await?;
        lint_feedback_stale_active_memory(items, &feedback, findings);
        lint_feedback_wrong_scope_active_memory(items, &feedback, findings);
        Ok(())
    }
}

fn filter_memory_items_for_project(items: &[MemoryItem], project: Option<&str>) -> Vec<MemoryItem> {
    items
        .iter()
        .filter(|item| scope_matches_project(&item.scope, project))
        .cloned()
        .collect()
}

fn scope_matches_project(scope: &MemoryScope, project: Option<&str>) -> bool {
    let Some(project) = project else {
        return true;
    };
    match scope {
        MemoryScope::Global | MemoryScope::User => true,
        MemoryScope::Project { project_name, .. } => scope_name_matches(project_name, project),
        MemoryScope::Task { project_name, .. } => project_name
            .as_deref()
            .is_some_and(|project_name| scope_name_matches(project_name, project)),
        MemoryScope::Entity { .. }
        | MemoryScope::Repository { .. }
        | MemoryScope::Session { .. }
        | MemoryScope::Custom { .. } => false,
    }
}

fn scope_name_matches(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
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

fn lint_feedback_stale_active_memory(
    items: &[MemoryItem],
    feedback: &[AgentFeedback],
    findings: &mut Vec<LintFinding>,
) {
    let active_items = active_memory_items_by_id(items);
    let stale_counts = feedback_signal_counts(feedback, |feedback| &feedback.stale_memory_ids);

    for (item_id, count) in stale_counts {
        let Some(item) = active_items.get(&item_id) else {
            continue;
        };
        if is_current_plan_guidance(item) {
            findings.push(
                LintFinding::new(
                    format!("feedback-stale-current-plan:{item_id}"),
                    LintRule::FeedbackStaleCurrentPlan,
                    LintSeverity::Info,
                    "Current-plan guidance has stale feedback",
                    format!(
                        "Active current-plan memory item '{}' ({}) was marked stale by {count} \
                         recent feedback record(s). Treat this as a review signal for \
                         supersession, archival, or scope correction; no automatic lifecycle \
                         action is safe.",
                        item.title, item.kind
                    ),
                )
                .with_item(item_id),
            );
            continue;
        }
        findings.push(
            LintFinding::new(
                format!("feedback-stale-active-memory:{item_id}"),
                LintRule::FeedbackStaleActiveMemory,
                LintSeverity::Info,
                "Active memory has stale feedback",
                format!(
                    "Active memory item '{}' ({}) was marked stale by {count} recent \
                     feedback record(s). Treat this as a review signal, not proof; no \
                     automatic lifecycle action is safe.",
                    item.title, item.kind
                ),
            )
            .with_item(item_id),
        );
    }
}

fn is_current_plan_guidance(item: &MemoryItem) -> bool {
    matches!(item.kind, MemoryKind::Decision | MemoryKind::Rule)
        && item
            .tags
            .iter()
            .any(|tag| tag.eq_ignore_ascii_case(CURRENT_PLAN_TAG))
}

fn lint_feedback_wrong_scope_active_memory(
    items: &[MemoryItem],
    feedback: &[AgentFeedback],
    findings: &mut Vec<LintFinding>,
) {
    let active_items = active_memory_items_by_id(items);
    let wrong_scope_counts =
        feedback_signal_counts(feedback, |feedback| &feedback.wrong_scope_memory_ids);

    for (item_id, count) in wrong_scope_counts {
        let Some(item) = active_items.get(&item_id) else {
            continue;
        };
        findings.push(
            LintFinding::new(
                format!("feedback-wrong-scope-active-memory:{item_id}"),
                LintRule::FeedbackWrongScopeActiveMemory,
                LintSeverity::Info,
                "Active memory has wrong-scope feedback",
                format!(
                    "Active memory item '{}' ({}) was marked wrong-scope by {count} recent \
                     feedback record(s). Review its scope or retrieval behavior before changing \
                     lifecycle status.",
                    item.title, item.kind
                ),
            )
            .with_item(item_id),
        );
    }
}

fn active_memory_items_by_id(items: &[MemoryItem]) -> HashMap<Id, &MemoryItem> {
    items
        .iter()
        .filter(|item| item.status == MemoryStatus::Active)
        .map(|item| (item.id, item))
        .collect()
}

fn feedback_signal_counts(
    feedback: &[AgentFeedback],
    ids: impl Fn(&AgentFeedback) -> &[Id],
) -> HashMap<Id, usize> {
    let mut counts = HashMap::new();
    for feedback in feedback {
        let unique_ids: HashSet<_> = ids(feedback).iter().copied().collect();
        for item_id in unique_ids {
            *counts.entry(item_id).or_insert(0) += 1;
        }
    }
    counts
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
    use engram_core::telemetry::AgentFeedback;
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
    async fn lint_prioritizes_feedback_signals_before_duplicate_entity_noise() {
        let service = service().await;
        let current_plan = MemoryItem::new(
            MemoryKind::Decision,
            "Current plan after older slice",
            "Old current-plan guidance that feedback later marked stale.",
            MemoryScope::project("engram"),
            ClaimOrigin::AgentObserved,
            writer(),
        )
        .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "test"))
        .with_tag(CURRENT_PLAN_TAG);
        service
            .memory_repo
            .save_memory_item(&current_plan)
            .await
            .unwrap();

        let mut feedback = AgentFeedback::new(Id::new());
        feedback.stale_memory_ids = vec![current_plan.id];
        service
            .telemetry_repo
            .save_feedback(&feedback)
            .await
            .unwrap();

        let obligation = AgentObligation::new(
            AgentObligationKind::DocumentDisposition,
            "Resolve historical document status",
            "An older document obligation should not hide feedback-stale plan signals.",
            MemoryScope::project("other-project"),
            AgentObligationTrigger::new("test", "historical document obligation"),
            writer(),
        );
        service
            .obligation_repo
            .save_obligation(&obligation)
            .await
            .unwrap();

        let old = MemoryItem::new(
            MemoryKind::Decision,
            "Superseded older decision",
            "Old content.",
            MemoryScope::project("engram"),
            ClaimOrigin::AgentObserved,
            writer(),
        )
        .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "test"));
        let replacement = MemoryItem::new(
            MemoryKind::Decision,
            "Replacement decision",
            "New content.",
            MemoryScope::project("engram"),
            ClaimOrigin::AgentObserved,
            writer(),
        )
        .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "test"))
        .with_superseded_item(old.id);
        service.memory_repo.save_memory_item(&old).await.unwrap();
        service
            .memory_repo
            .save_memory_item(&replacement)
            .await
            .unwrap();

        for index in 0..3 {
            let item = MemoryItem::new(
                MemoryKind::ProjectFact,
                format!("Duplicate {index}"),
                "Duplicate entity-scoped content.",
                MemoryScope::entity("ide-mcp-eval"),
                ClaimOrigin::AgentObserved,
                writer(),
            )
            .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "test"));
            service.memory_repo.save_memory_item(&item).await.unwrap();
        }

        let report = service
            .run(LintOptions {
                project: None,
                vault_path: None,
                limit: Some(1),
            })
            .await
            .unwrap();

        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].rule, LintRule::FeedbackStaleCurrentPlan);
    }

    #[tokio::test]
    async fn lint_prioritizes_superseded_active_items_before_generic_feedback_noise() {
        let service = service().await;
        let current_plan = MemoryItem::new(
            MemoryKind::Decision,
            "Current plan after latest slice",
            "Current-plan guidance that stale feedback should keep first.",
            MemoryScope::project("engram"),
            ClaimOrigin::AgentObserved,
            writer(),
        )
        .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "test"))
        .with_tag(CURRENT_PLAN_TAG);
        service
            .memory_repo
            .save_memory_item(&current_plan)
            .await
            .unwrap();

        let old_handoff = MemoryItem::new(
            MemoryKind::Handoff,
            "Superseded handoff",
            "# Handoff\n\n## Next Actions\n- Continue from the replacement handoff.",
            MemoryScope::project("engram"),
            ClaimOrigin::AgentObserved,
            writer(),
        )
        .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "test"));
        let replacement_handoff = MemoryItem::new(
            MemoryKind::Handoff,
            "Replacement handoff",
            "# Handoff\n\n## Next Actions\n- Continue from the latest handoff.",
            MemoryScope::project("engram"),
            ClaimOrigin::AgentObserved,
            writer(),
        )
        .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "test"))
        .with_superseded_item(old_handoff.id);
        service
            .memory_repo
            .save_memory_item(&old_handoff)
            .await
            .unwrap();
        service
            .memory_repo
            .save_memory_item(&replacement_handoff)
            .await
            .unwrap();

        let mut stale_ids = vec![current_plan.id];
        for index in 0..5 {
            let item = MemoryItem::new(
                MemoryKind::ProjectFact,
                format!("Generic stale item {index}"),
                "Generic active memory with stale feedback.",
                MemoryScope::project("engram"),
                ClaimOrigin::AgentObserved,
                writer(),
            )
            .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "test"));
            stale_ids.push(item.id);
            service.memory_repo.save_memory_item(&item).await.unwrap();
        }
        let mut feedback = AgentFeedback::new(Id::new());
        feedback.stale_memory_ids = stale_ids;
        service
            .telemetry_repo
            .save_feedback(&feedback)
            .await
            .unwrap();

        let report = service
            .run(LintOptions {
                project: None,
                vault_path: None,
                limit: Some(2),
            })
            .await
            .unwrap();

        assert_eq!(report.findings.len(), 2);
        assert_eq!(report.findings[0].rule, LintRule::FeedbackStaleCurrentPlan);
        assert_eq!(report.findings[1].rule, LintRule::SupersededItemStillActive);
        assert_eq!(report.findings[1].item_id, Some(old_handoff.id));
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
    async fn lint_project_scope_filters_memory_obligations_and_sessions() {
        let service = service().await;
        let global = MemoryItem::new(
            MemoryKind::Rule,
            "Global rule without evidence",
            "Global guidance should remain visible in every project-scoped lint report.",
            MemoryScope::Global,
            ClaimOrigin::AgentObserved,
            writer(),
        );
        let engram = MemoryItem::new(
            MemoryKind::Decision,
            "Engram decision without evidence",
            "Project-scoped lint should include this item.",
            MemoryScope::project("engram"),
            ClaimOrigin::AgentObserved,
            writer(),
        );
        let engram_task = MemoryItem::new(
            MemoryKind::TaskFact,
            "Engram task without evidence",
            "Task-scoped lint should include tasks whose parent project matches.",
            MemoryScope::Task {
                project_id: None,
                project_name: Some("engram".to_string()),
                task_id: None,
                task_name: "beta".to_string(),
            },
            ClaimOrigin::AgentObserved,
            writer(),
        );
        let other = MemoryItem::new(
            MemoryKind::Decision,
            "Other project decision without evidence",
            "Project-scoped lint should exclude this item.",
            MemoryScope::project("other-project"),
            ClaimOrigin::AgentObserved,
            writer(),
        );
        service.memory_repo.save_memory_item(&global).await.unwrap();
        service.memory_repo.save_memory_item(&engram).await.unwrap();
        service
            .memory_repo
            .save_memory_item(&engram_task)
            .await
            .unwrap();
        service.memory_repo.save_memory_item(&other).await.unwrap();

        let engram_obligation = AgentObligation::new(
            AgentObligationKind::SourceReading,
            "Read Engram source",
            "Project-scoped lint should include this obligation.",
            MemoryScope::project("engram"),
            AgentObligationTrigger::new("prompt", "implementation request"),
            writer(),
        );
        let other_obligation = AgentObligation::new(
            AgentObligationKind::SourceReading,
            "Read other source",
            "Project-scoped lint should exclude this obligation.",
            MemoryScope::project("other-project"),
            AgentObligationTrigger::new("prompt", "implementation request"),
            writer(),
        );
        service
            .obligation_repo
            .save_obligation(&engram_obligation)
            .await
            .unwrap();
        service
            .obligation_repo
            .save_obligation(&other_obligation)
            .await
            .unwrap();

        let mut engram_session = engram_core::session::Session::new()
            .with_project("engram")
            .with_agent("codex");
        engram_session.started_at = OffsetDateTime::now_utc() - Duration::days(2);
        let mut other_session = engram_core::session::Session::new().with_project("other-project");
        other_session.started_at = OffsetDateTime::now_utc() - Duration::days(2);
        service
            .session_repo
            .save_session(&engram_session)
            .await
            .unwrap();
        service
            .session_repo
            .save_session(&other_session)
            .await
            .unwrap();

        let report = service
            .run(LintOptions {
                project: Some("engram".to_string()),
                vault_path: None,
                limit: None,
            })
            .await
            .unwrap();

        assert!(report.findings.iter().any(|finding| {
            finding.rule == LintRule::MissingEvidence && finding.item_id == Some(global.id)
        }));
        assert!(report.findings.iter().any(|finding| {
            finding.rule == LintRule::MissingEvidence && finding.item_id == Some(engram.id)
        }));
        assert!(report.findings.iter().any(|finding| {
            finding.rule == LintRule::MissingEvidence && finding.item_id == Some(engram_task.id)
        }));
        assert!(!report
            .findings
            .iter()
            .any(|finding| finding.item_id == Some(other.id)));
        assert!(report.findings.iter().any(|finding| {
            finding.rule == LintRule::UnresolvedAgentObligation
                && finding.obligation_id == Some(engram_obligation.id)
        }));
        assert!(!report
            .findings
            .iter()
            .any(|finding| finding.obligation_id == Some(other_obligation.id)));
        assert!(report.findings.iter().any(|finding| {
            finding.rule == LintRule::StaleActiveSession
                && finding.session_id == Some(engram_session.id)
                && finding.message.contains("project: engram")
                && finding.message.contains("agent: codex")
                && finding.message.contains("started_at:")
                && finding.message.contains("age_hours:")
        }));
        assert!(!report
            .findings
            .iter()
            .any(|finding| finding.session_id == Some(other_session.id)));
    }

    #[tokio::test]
    async fn lint_reports_feedback_flagged_active_memory_once_per_item() {
        let service = service().await;
        let item = MemoryItem::new(
            MemoryKind::ProjectFact,
            "Possibly stale fact",
            "Content that feedback later questioned.",
            MemoryScope::project("engram"),
            ClaimOrigin::AgentObserved,
            writer(),
        )
        .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "test"));
        service.memory_repo.save_memory_item(&item).await.unwrap();

        let mut first_feedback = AgentFeedback::new(Id::new());
        first_feedback.stale_memory_ids = vec![item.id, item.id];
        first_feedback.wrong_scope_memory_ids = vec![item.id];
        service
            .telemetry_repo
            .save_feedback(&first_feedback)
            .await
            .unwrap();

        let mut second_feedback = AgentFeedback::new(Id::new());
        second_feedback.stale_memory_ids = vec![item.id];
        service
            .telemetry_repo
            .save_feedback(&second_feedback)
            .await
            .unwrap();

        let report = service.run(LintOptions::default()).await.unwrap();

        let stale_findings = report
            .findings
            .iter()
            .filter(|finding| finding.rule == LintRule::FeedbackStaleActiveMemory)
            .collect::<Vec<_>>();
        let wrong_scope_findings = report
            .findings
            .iter()
            .filter(|finding| finding.rule == LintRule::FeedbackWrongScopeActiveMemory)
            .collect::<Vec<_>>();

        assert_eq!(stale_findings.len(), 1);
        assert_eq!(wrong_scope_findings.len(), 1);
        assert_eq!(
            stale_findings[0].id,
            format!("feedback-stale-active-memory:{}", item.id)
        );
        assert_eq!(stale_findings[0].item_id, Some(item.id));
        assert_eq!(stale_findings[0].severity, LintSeverity::Info);
        assert_eq!(stale_findings[0].safe_action, LintSafeAction::None);
        assert!(stale_findings[0]
            .message
            .contains("marked stale by 2 recent feedback record(s)"));
        assert!(wrong_scope_findings[0]
            .message
            .contains("marked wrong-scope by 1 recent feedback record(s)"));
    }

    #[tokio::test]
    async fn lint_reports_stale_current_plan_feedback_with_specific_rule() {
        let service = service().await;
        let item = MemoryItem::new(
            MemoryKind::Decision,
            "Current plan after older slice",
            "Old current-plan guidance that feedback later marked stale.",
            MemoryScope::repository(None, Some("/Users/yuval.meiri/projects/engram".to_string())),
            ClaimOrigin::AgentObserved,
            writer(),
        )
        .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "test"))
        .with_tag("current-plan");
        service.memory_repo.save_memory_item(&item).await.unwrap();

        let mut feedback = AgentFeedback::new(Id::new());
        feedback.stale_memory_ids = vec![item.id, item.id];
        service
            .telemetry_repo
            .save_feedback(&feedback)
            .await
            .unwrap();

        let report = service.run(LintOptions::default()).await.unwrap();

        let current_plan_finding = report
            .findings
            .iter()
            .find(|finding| finding.rule == LintRule::FeedbackStaleCurrentPlan)
            .expect("stale current-plan finding should be present");

        assert_eq!(
            current_plan_finding.id,
            format!("feedback-stale-current-plan:{}", item.id)
        );
        assert_eq!(current_plan_finding.item_id, Some(item.id));
        assert_eq!(current_plan_finding.severity, LintSeverity::Info);
        assert_eq!(current_plan_finding.safe_action, LintSafeAction::None);
        assert!(current_plan_finding
            .message
            .contains("Current plan after older slice"));
        assert!(current_plan_finding
            .message
            .contains("marked stale by 1 recent feedback record(s)"));
        assert!(!report.findings.iter().any(|finding| {
            finding.rule == LintRule::FeedbackStaleActiveMemory && finding.item_id == Some(item.id)
        }));
    }

    #[tokio::test]
    async fn lint_keeps_stale_migration_authorization_on_generic_feedback_rule() {
        let service = service().await;
        let item = MemoryItem::new(
            MemoryKind::ProjectFact,
            "Approved repo topology migration write applied first batch",
            "Old migration approval record from an earlier scoped repository topology write. \
             It is not current M6 authorization.",
            MemoryScope::project("engram"),
            ClaimOrigin::ToolResult,
            writer(),
        )
        .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "test"));
        service.memory_repo.save_memory_item(&item).await.unwrap();

        let mut feedback = AgentFeedback::new(Id::new());
        feedback.stale_memory_ids = vec![item.id];
        service
            .telemetry_repo
            .save_feedback(&feedback)
            .await
            .unwrap();

        let report = service.run(LintOptions::default()).await.unwrap();

        let stale_finding = report
            .findings
            .iter()
            .find(|finding| finding.rule == LintRule::FeedbackStaleActiveMemory)
            .expect("stale migration authorization should use generic stale feedback lint");

        assert_eq!(
            stale_finding.id,
            format!("feedback-stale-active-memory:{}", item.id)
        );
        assert_eq!(stale_finding.item_id, Some(item.id));
        assert_eq!(stale_finding.severity, LintSeverity::Info);
        assert_eq!(stale_finding.safe_action, LintSafeAction::None);
        assert!(stale_finding
            .message
            .contains("Approved repo topology migration write applied first batch"));
        assert!(stale_finding
            .message
            .contains("no automatic lifecycle action is safe"));
        assert!(!report.findings.iter().any(|finding| {
            finding.rule == LintRule::FeedbackStaleCurrentPlan && finding.item_id == Some(item.id)
        }));
    }

    #[tokio::test]
    async fn lint_ignores_feedback_for_non_active_or_missing_memory() {
        let service = service().await;
        let item = MemoryItem::new(
            MemoryKind::ProjectFact,
            "Archived fact",
            "Content that is no longer active.",
            MemoryScope::project("engram"),
            ClaimOrigin::AgentObserved,
            writer(),
        )
        .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "test"))
        .with_archive("test archive", Some("test".to_string()));
        service.memory_repo.save_memory_item(&item).await.unwrap();

        let mut feedback = AgentFeedback::new(Id::new());
        feedback.stale_memory_ids = vec![item.id, Id::new()];
        feedback.wrong_scope_memory_ids = vec![item.id, Id::new()];
        service
            .telemetry_repo
            .save_feedback(&feedback)
            .await
            .unwrap();

        let report = service.run(LintOptions::default()).await.unwrap();

        assert!(!report.findings.iter().any(|finding| {
            matches!(
                finding.rule,
                LintRule::FeedbackStaleActiveMemory
                    | LintRule::FeedbackStaleCurrentPlan
                    | LintRule::FeedbackWrongScopeActiveMemory
            )
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
                project: None,
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
                project: None,
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
