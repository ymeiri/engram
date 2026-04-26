//! Non-destructive migration inventory for Memory OS.
//!
//! This module inspects existing Engram layers and proposes Memory OS
//! candidates without writing MemoryItem records. It is intentionally
//! conservative: generated output is a review queue, not an automatic migration.

use crate::error::{IndexError, IndexResult};
use engram_core::entity::{Entity, EntityType, Observation};
use engram_core::id::Id;
use engram_core::memory::{
    ClaimOrigin, EvidenceKind, EvidenceRef, KnowledgeCommit, MemoryChange, MemoryChangeType,
    MemoryItem, MemoryKind, MemoryScope, MemoryStatus, WriterProvenance,
};
use engram_core::session::{Event, EventType, Session};
use engram_core::work::{Project, ProjectObservation, Task, TaskObservation};
use engram_store::{Db, EntityRepo, MemoryRepo, SessionRepo, WorkRepo};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use time::OffsetDateTime;

const REVIEW_GENERATED_BY: &str = "engram-memory-os";
const REVIEW_GENERATED_MARKER: &str = "<!-- engram:generated:file migration-review-v1 -->";
const MACHINE_RECORD_HEADING: &str = "## Machine Record";
const MACHINE_RECORD_FENCE: &str = "```json";

/// Options for a non-destructive migration inventory.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MigrationInventoryOptions {
    /// Optional project name filter.
    pub project_filter: Option<String>,
    /// Maximum candidates to return after inventorying all sources.
    pub limit: Option<usize>,
    /// Include Layer 1 entity observations.
    pub include_entity_observations: bool,
    /// Include Layer 2 session summaries, decisions, and events.
    pub include_session_history: bool,
    /// Include Layer 7 project/task observations.
    pub include_work_observations: bool,
}

impl MigrationInventoryOptions {
    /// Options that scan all currently supported source layers.
    #[must_use]
    pub fn all() -> Self {
        Self {
            project_filter: None,
            limit: None,
            include_entity_observations: true,
            include_session_history: true,
            include_work_observations: true,
        }
    }
}

/// Existing Engram source layer for a migration candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationSourceKind {
    /// Layer 1 entity observation.
    EntityObservation,
    /// Layer 2 session summary.
    SessionSummary,
    /// Layer 2 session key decision.
    SessionKeyDecision,
    /// Layer 2 session event.
    SessionEvent,
    /// Layer 7 project observation.
    ProjectObservation,
    /// Layer 7 task observation.
    TaskObservation,
}

impl std::fmt::Display for MigrationSourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EntityObservation => write!(f, "entity_observation"),
            Self::SessionSummary => write!(f, "session_summary"),
            Self::SessionKeyDecision => write!(f, "session_key_decision"),
            Self::SessionEvent => write!(f, "session_event"),
            Self::ProjectObservation => write!(f, "project_observation"),
            Self::TaskObservation => write!(f, "task_observation"),
        }
    }
}

/// Recommendation for a migration candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationDisposition {
    /// Candidate appears worth human review for migration.
    Review,
    /// Candidate should be held aside until a human resolves uncertainty.
    Quarantine,
    /// Candidate is probably low-value operational trace data.
    Skip,
}

impl std::fmt::Display for MigrationDisposition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Review => write!(f, "review"),
            Self::Quarantine => write!(f, "quarantine"),
            Self::Skip => write!(f, "skip"),
        }
    }
}

/// Candidate Memory OS record inferred from existing Engram data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationCandidate {
    /// Source layer.
    pub source_kind: MigrationSourceKind,
    /// Source record identifier. Synthetic for session summaries/key decisions.
    pub source_id: String,
    /// Human-readable source label.
    pub source_label: String,
    /// Existing semantic key or event type, when available.
    pub source_key: Option<String>,
    /// Proposed Memory OS title.
    pub title: String,
    /// Proposed Memory OS content.
    pub content: String,
    /// Proposed Memory OS kind.
    pub proposed_kind: MemoryKind,
    /// Proposed Memory OS scope.
    pub proposed_scope: MemoryScope,
    /// Proposed origin. Dry-run candidates always use migrated origin.
    pub proposed_origin: ClaimOrigin,
    /// Heuristic confidence from 0.0 to 1.0.
    pub confidence: f32,
    /// Age of the source record at inventory time.
    pub staleness_days: i64,
    /// Recommended handling.
    pub disposition: MigrationDisposition,
    /// Reasons for the recommendation.
    pub reasons: Vec<String>,
    /// Source creation time.
    #[serde(with = "time::serde::rfc3339")]
    pub source_created_at: OffsetDateTime,
    /// Source update time.
    #[serde(with = "time::serde::rfc3339")]
    pub source_updated_at: OffsetDateTime,
}

/// Non-destructive migration inventory result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationInventory {
    /// When the inventory was generated.
    #[serde(with = "time::serde::rfc3339")]
    pub generated_at: OffsetDateTime,
    /// Optional project filter used for this inventory.
    pub project_filter: Option<String>,
    /// Number of source records scanned before candidate filtering.
    pub sources_scanned: usize,
    /// Number of candidates found before return limiting.
    pub total_candidates: usize,
    /// Number of candidates returned.
    pub returned_candidates: usize,
    /// Whether the returned candidate list was truncated by the limit option.
    pub truncated: bool,
    /// Candidate counts by source kind.
    pub by_source_kind: BTreeMap<String, usize>,
    /// Candidate counts by disposition.
    pub by_disposition: BTreeMap<String, usize>,
    /// Candidate counts by proposed Memory OS kind.
    pub by_memory_kind: BTreeMap<String, usize>,
    /// Warnings about the dry run.
    pub warnings: Vec<String>,
    /// Candidate records.
    pub candidates: Vec<MigrationCandidate>,
}

/// Result of writing a migration review batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationReviewExport {
    /// Review batch root path.
    pub root: String,
    /// Files created or updated, relative to root.
    pub files_written: Vec<String>,
    /// Existing files skipped because they were not generated by Engram.
    pub files_skipped: Vec<String>,
    /// Inventory used for this review batch.
    pub inventory: MigrationInventory,
}

impl MigrationReviewExport {
    /// Number of files written by this export.
    #[must_use]
    pub fn file_count(&self) -> usize {
        self.files_written.len()
    }
}

/// Options for applying a reviewed migration batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationReviewApplyOptions {
    /// When true, parse and report the batch without writing memory records.
    pub dry_run: bool,
    /// Writer/importer provenance to attach to accepted memory records.
    pub writer: WriterProvenance,
    /// Create a knowledge commit for written records.
    pub create_commit: bool,
}

/// Result of applying, or dry-running, a migration review batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationReviewApply {
    /// Review batch root path.
    pub root: String,
    /// Whether this run avoided writes.
    pub dry_run: bool,
    /// Candidate review files scanned.
    pub files_scanned: usize,
    /// Generated candidate files skipped with a reason.
    pub files_skipped: Vec<String>,
    /// Candidate files that had no selected decision checkbox.
    pub files_with_no_decision: Vec<String>,
    /// Candidate files that selected multiple conflicting decisions.
    pub files_with_conflicts: Vec<String>,
    /// Accepted candidates using generated content.
    pub accepted_count: usize,
    /// Accepted candidates using edited title/content/kind.
    pub accepted_with_edits_count: usize,
    /// Candidates explicitly quarantined by review.
    pub quarantined_count: usize,
    /// Candidates explicitly rejected by review.
    pub rejected_count: usize,
    /// Accepted candidates skipped because their source was already migrated.
    pub duplicate_count: usize,
    /// Items that would be written, or were written in non-dry-run mode.
    pub planned_items: Vec<MemoryItem>,
    /// Items written in non-dry-run mode.
    pub written_items: Vec<MemoryItem>,
    /// Knowledge commit created in non-dry-run mode.
    pub commit: Option<KnowledgeCommit>,
    /// Non-fatal warnings surfaced during parsing/apply.
    pub warnings: Vec<String>,
}

impl MigrationReviewApply {
    /// Number of planned accepted memory records.
    #[must_use]
    pub fn planned_count(&self) -> usize {
        self.planned_items.len()
    }

    /// Number of written memory records.
    #[must_use]
    pub fn written_count(&self) -> usize {
        self.written_items.len()
    }
}

/// Service that inventories existing Engram data for future Memory OS migration.
#[derive(Clone)]
pub struct MigrationService {
    entity_repo: EntityRepo,
    memory_repo: MemoryRepo,
    session_repo: SessionRepo,
    work_repo: WorkRepo,
}

impl MigrationService {
    /// Create a new migration service.
    pub fn new(db: Db) -> Self {
        Self {
            entity_repo: EntityRepo::new(db.clone()),
            memory_repo: MemoryRepo::new(db.clone()),
            session_repo: SessionRepo::new(db.clone()),
            work_repo: WorkRepo::new(db),
        }
    }

    /// Build a non-destructive inventory of migration candidates.
    pub async fn inventory(
        &self,
        options: MigrationInventoryOptions,
    ) -> IndexResult<MigrationInventory> {
        let options = normalize_options(options);
        let now = OffsetDateTime::now_utc();
        let mut warnings = vec![
            "Dry run only: no Memory OS records were written.".to_string(),
            "Only explicitly accepted review candidates are eligible for migration writes."
                .to_string(),
        ];
        let mut candidates = Vec::new();
        let mut sources_scanned = 0;

        let project_filter = options.project_filter.as_deref();
        let project_for_filter = if let Some(project_name) = project_filter {
            let project = self.work_repo.get_project_by_name(project_name).await?;
            if project.is_none() {
                warnings.push(format!(
                    "Project filter '{}' did not match a Layer 7 project; work and linked-entity scans returned no project-scoped records.",
                    project_name
                ));
            }
            project
        } else {
            None
        };

        if options.include_work_observations {
            sources_scanned += self
                .inventory_work_observations(project_for_filter.as_ref(), &mut candidates, now)
                .await?;
        }
        if options.include_entity_observations {
            sources_scanned += self
                .inventory_entity_observations(
                    project_for_filter.as_ref(),
                    project_filter,
                    &mut candidates,
                    now,
                )
                .await?;
        }
        if options.include_session_history {
            sources_scanned += self
                .inventory_session_history(project_filter, &mut candidates, now)
                .await?;
        }

        candidates.sort_by(|left, right| {
            disposition_rank(left.disposition)
                .cmp(&disposition_rank(right.disposition))
                .then_with(|| right.source_updated_at.cmp(&left.source_updated_at))
                .then_with(|| {
                    left.source_kind
                        .to_string()
                        .cmp(&right.source_kind.to_string())
                })
                .then_with(|| left.source_id.cmp(&right.source_id))
        });

        let total_candidates = candidates.len();
        let truncated = options.limit.is_some_and(|limit| candidates.len() > limit);
        if let Some(limit) = options.limit {
            candidates.truncate(limit);
        }

        let returned_candidates = candidates.len();
        Ok(MigrationInventory {
            generated_at: now,
            project_filter: options.project_filter,
            sources_scanned,
            total_candidates,
            returned_candidates,
            truncated,
            by_source_kind: count_by_source_kind(&candidates),
            by_disposition: count_by_disposition(&candidates),
            by_memory_kind: count_by_memory_kind(&candidates),
            warnings,
            candidates,
        })
    }

    /// Export a non-destructive migration review batch as generated Markdown.
    pub async fn export_review_batch(
        &self,
        root: impl AsRef<Path>,
        options: MigrationInventoryOptions,
    ) -> IndexResult<MigrationReviewExport> {
        let inventory = self.inventory(options).await?;
        write_review_batch(root.as_ref(), inventory)
    }

    /// Apply a reviewed migration batch. Dry-run mode parses and reports only.
    pub async fn apply_review_batch(
        &self,
        root: impl AsRef<Path>,
        options: MigrationReviewApplyOptions,
    ) -> IndexResult<MigrationReviewApply> {
        let root = root.as_ref();
        let mut report = MigrationReviewApply {
            root: root.display().to_string(),
            dry_run: options.dry_run,
            files_scanned: 0,
            files_skipped: Vec::new(),
            files_with_no_decision: Vec::new(),
            files_with_conflicts: Vec::new(),
            accepted_count: 0,
            accepted_with_edits_count: 0,
            quarantined_count: 0,
            rejected_count: 0,
            duplicate_count: 0,
            planned_items: Vec::new(),
            written_items: Vec::new(),
            commit: None,
            warnings: Vec::new(),
        };

        let existing_sources = self.existing_migration_source_tags().await?;
        let mut seen_sources = existing_sources;

        for path in collect_candidate_review_files(root)? {
            report.files_scanned += 1;
            let relative_path = relative_markdown_path(root, &path);
            let contents = fs::read_to_string(&path)?;
            let Some(parsed) = parse_review_candidate_page(&contents, &relative_path, &mut report)?
            else {
                continue;
            };

            apply_parsed_review(
                parsed,
                &options,
                &mut seen_sources,
                &mut report,
                &relative_path,
            );
        }

        if !options.dry_run {
            for item in &report.planned_items {
                self.memory_repo.save_memory_item(item).await?;
                report.written_items.push(item.clone());
            }
            if options.create_commit && !report.written_items.is_empty() {
                let commit = build_migration_commit(&options.writer, &report.written_items);
                self.memory_repo.save_knowledge_commit(&commit).await?;
                report.commit = Some(commit);
            }
        }

        report.files_skipped.sort();
        report.files_with_no_decision.sort();
        report.files_with_conflicts.sort();
        report.warnings.sort();
        Ok(report)
    }

    async fn existing_migration_source_tags(&self) -> IndexResult<HashSet<String>> {
        let items = self.memory_repo.list_memory_items(None, None).await?;
        Ok(items
            .into_iter()
            .flat_map(|item| item.tags.into_iter())
            .filter(|tag| tag.starts_with("migration-source:"))
            .collect())
    }

    async fn inventory_work_observations(
        &self,
        project_for_filter: Option<&Project>,
        candidates: &mut Vec<MigrationCandidate>,
        now: OffsetDateTime,
    ) -> IndexResult<usize> {
        let projects = if let Some(project) = project_for_filter {
            vec![project.clone()]
        } else {
            self.work_repo.list_projects(None).await?
        };

        let mut scanned = 0;
        for project in projects {
            for observation in self.work_repo.get_project_observations(&project.id).await? {
                scanned += 1;
                candidates.push(candidate_from_project_observation(
                    &project,
                    observation,
                    now,
                ));
            }

            for task in self.work_repo.list_tasks(&project.id, None).await? {
                for observation in self.work_repo.get_task_observations(&task.id).await? {
                    scanned += 1;
                    candidates.push(candidate_from_task_observation(
                        &project,
                        &task,
                        observation,
                        now,
                    ));
                }
            }
        }
        Ok(scanned)
    }

    async fn inventory_entity_observations(
        &self,
        project_for_filter: Option<&Project>,
        project_filter: Option<&str>,
        candidates: &mut Vec<MigrationCandidate>,
        now: OffsetDateTime,
    ) -> IndexResult<usize> {
        let entities = if let Some(project) = project_for_filter {
            let ids = self
                .work_repo
                .get_project_entities(&project.id)
                .await?
                .into_iter()
                .map(|(id, _)| id)
                .collect::<HashSet<_>>();
            self.entities_by_id(ids).await?
        } else if project_filter.is_some() {
            Vec::new()
        } else {
            self.entity_repo.list_entities(None).await?
        };

        let mut scanned = 0;
        for entity in entities {
            for observation in self.entity_repo.get_observations(&entity.id).await? {
                scanned += 1;
                candidates.push(candidate_from_entity_observation(&entity, observation, now));
            }
        }
        Ok(scanned)
    }

    async fn entities_by_id(&self, ids: HashSet<Id>) -> IndexResult<Vec<Entity>> {
        let mut entities = Vec::new();
        for id in ids {
            if let Some(entity) = self.entity_repo.get_entity(&id).await? {
                entities.push(entity);
            }
        }
        entities.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(entities)
    }

    async fn inventory_session_history(
        &self,
        project_filter: Option<&str>,
        candidates: &mut Vec<MigrationCandidate>,
        now: OffsetDateTime,
    ) -> IndexResult<usize> {
        let sessions = self
            .session_repo
            .list_sessions(None, None, project_filter, None)
            .await?;

        let mut scanned = 0;
        for session in sessions {
            if let Some(summary) = session
                .summary
                .as_deref()
                .filter(|value| !value.trim().is_empty())
            {
                scanned += 1;
                candidates.push(candidate_from_session_summary(&session, summary, now));
            }

            for (index, decision) in session.key_decisions.iter().enumerate() {
                scanned += 1;
                candidates.push(candidate_from_session_key_decision(
                    &session, index, decision, now,
                ));
            }

            for event in self.session_repo.get_events(&session.id).await? {
                scanned += 1;
                candidates.push(candidate_from_session_event(&session, event, now));
            }
        }
        Ok(scanned)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReviewDecision {
    Accept,
    AcceptWithEdits,
    Quarantine,
    Reject,
}

struct ParsedReviewCandidate {
    decision: ReviewDecision,
    candidate: MigrationCandidate,
    edited_title: Option<String>,
    edited_content: Option<String>,
    edited_kind: Option<MemoryKind>,
}

fn apply_parsed_review(
    parsed: ParsedReviewCandidate,
    options: &MigrationReviewApplyOptions,
    seen_sources: &mut HashSet<String>,
    report: &mut MigrationReviewApply,
    relative_path: &str,
) {
    match parsed.decision {
        ReviewDecision::Accept | ReviewDecision::AcceptWithEdits => {
            let source_tag = migration_source_tag(&parsed.candidate);
            if !seen_sources.insert(source_tag.clone()) {
                report.duplicate_count += 1;
                report.warnings.push(format!(
                    "{relative_path}: source {} was already migrated; skipped duplicate",
                    parsed.candidate.source_id
                ));
                return;
            }

            if parsed.decision == ReviewDecision::Accept {
                report.accepted_count += 1;
            } else {
                report.accepted_with_edits_count += 1;
            }

            let item = memory_item_from_review(parsed, options, relative_path, source_tag);
            report.planned_items.push(item);
        }
        ReviewDecision::Quarantine => {
            report.quarantined_count += 1;
        }
        ReviewDecision::Reject => {
            report.rejected_count += 1;
        }
    }
}

fn memory_item_from_review(
    parsed: ParsedReviewCandidate,
    options: &MigrationReviewApplyOptions,
    relative_path: &str,
    source_tag: String,
) -> MemoryItem {
    let use_edits = parsed.decision == ReviewDecision::AcceptWithEdits;
    let title = if use_edits {
        parsed
            .edited_title
            .unwrap_or_else(|| parsed.candidate.title.clone())
    } else {
        parsed.candidate.title.clone()
    };
    let content = if use_edits {
        parsed
            .edited_content
            .unwrap_or_else(|| parsed.candidate.content.clone())
    } else {
        parsed.candidate.content.clone()
    };
    let kind = if use_edits {
        parsed
            .edited_kind
            .unwrap_or_else(|| parsed.candidate.proposed_kind.clone())
    } else {
        parsed.candidate.proposed_kind.clone()
    };
    let source_evidence = source_evidence(&parsed.candidate);
    let review_evidence = EvidenceRef::new(EvidenceKind::ManualReview, relative_path)
        .with_summary("Accepted from a generated Engram migration review batch.");

    let mut item = MemoryItem::new(
        kind,
        title,
        content,
        parsed.candidate.proposed_scope.clone(),
        parsed.candidate.proposed_origin.clone(),
        options.writer.clone(),
    )
    .with_confidence(parsed.candidate.confidence)
    .with_status(MemoryStatus::Active)
    .with_evidence(source_evidence)
    .with_evidence(review_evidence)
    .with_tag("migration")
    .with_tag("migration-reviewed")
    .with_tag(source_tag);
    if use_edits {
        item = item.with_tag("migration-reviewed-edited");
    }
    item
}

fn source_evidence(candidate: &MigrationCandidate) -> EvidenceRef {
    let kind = match candidate.source_kind {
        MigrationSourceKind::EntityObservation
        | MigrationSourceKind::ProjectObservation
        | MigrationSourceKind::TaskObservation => EvidenceKind::Observation,
        MigrationSourceKind::SessionSummary
        | MigrationSourceKind::SessionKeyDecision
        | MigrationSourceKind::SessionEvent => EvidenceKind::SessionEvent,
    };

    EvidenceRef::new(kind, &candidate.source_id)
        .with_summary(&candidate.source_label)
        .with_excerpt(candidate.content.chars().take(500).collect::<String>())
}

fn build_migration_commit(
    writer: &WriterProvenance,
    written_items: &[MemoryItem],
) -> KnowledgeCommit {
    let mut commit = KnowledgeCommit::new(
        writer.clone(),
        format!(
            "Apply reviewed migration batch ({} items)",
            written_items.len()
        ),
    );
    for item in written_items {
        commit = commit.with_change(
            MemoryChange::new(
                MemoryChangeType::Added,
                &item.title,
                "Migrated reviewed legacy Engram memory into Memory OS.",
            )
            .with_item(item.id),
        );
    }
    commit
}

fn normalize_options(mut options: MigrationInventoryOptions) -> MigrationInventoryOptions {
    if !options.include_entity_observations
        && !options.include_session_history
        && !options.include_work_observations
    {
        options.include_entity_observations = true;
        options.include_session_history = true;
        options.include_work_observations = true;
    }
    options
}

fn candidate_from_project_observation(
    project: &Project,
    observation: ProjectObservation,
    now: OffsetDateTime,
) -> MigrationCandidate {
    let source_key = observation.key;
    let source_is_keyed = source_key.is_some();
    let force_disposition = if is_transient_project_observation_key(source_key.as_deref()) {
        Some(MigrationDisposition::Quarantine)
    } else {
        None
    };
    let mut base_reasons =
        vec!["Layer 7 project observation maps directly to project memory.".into()];
    if force_disposition == Some(MigrationDisposition::Quarantine) {
        base_reasons.push(
            "Project status/artifact rollups are transient; quarantine until current state is confirmed."
                .into(),
        );
    }
    let kind = classify_from_key_or_content(
        source_key.as_deref(),
        &observation.content,
        MemoryKind::ProjectFact,
    );
    let title = title_from_key_or_content(source_key.as_deref(), &observation.content);
    let scope = MemoryScope::Project {
        project_id: Some(project.id),
        project_name: project.name.clone(),
    };
    build_candidate(CandidateInput {
        source_kind: MigrationSourceKind::ProjectObservation,
        source_id: observation.id.to_string(),
        source_label: format!("project:{} observation", project.name),
        source_key,
        title,
        content: observation.content,
        proposed_kind: kind,
        proposed_scope: scope,
        source_created_at: observation.created_at,
        source_updated_at: observation.updated_at,
        source_is_keyed,
        source_has_project: true,
        force_disposition,
        base_reasons,
        now,
    })
}

fn candidate_from_task_observation(
    project: &Project,
    task: &Task,
    observation: TaskObservation,
    now: OffsetDateTime,
) -> MigrationCandidate {
    let source_key = observation.key;
    let source_is_keyed = source_key.is_some();
    let kind = classify_from_key_or_content(
        source_key.as_deref(),
        &observation.content,
        MemoryKind::TaskFact,
    );
    let title = title_from_key_or_content(source_key.as_deref(), &observation.content);
    let scope = MemoryScope::Task {
        project_id: Some(project.id),
        project_name: Some(project.name.clone()),
        task_id: Some(task.id),
        task_name: task.jira_key.clone().unwrap_or_else(|| task.name.clone()),
    };
    build_candidate(CandidateInput {
        source_kind: MigrationSourceKind::TaskObservation,
        source_id: observation.id.to_string(),
        source_label: format!(
            "task:{} observation",
            task.jira_key.as_deref().unwrap_or(&task.name)
        ),
        source_key,
        title,
        content: observation.content,
        proposed_kind: kind,
        proposed_scope: scope,
        source_created_at: observation.created_at,
        source_updated_at: observation.updated_at,
        source_is_keyed,
        source_has_project: true,
        force_disposition: None,
        base_reasons: vec!["Layer 7 task observation maps directly to task memory.".into()],
        now,
    })
}

fn candidate_from_entity_observation(
    entity: &Entity,
    observation: Observation,
    now: OffsetDateTime,
) -> MigrationCandidate {
    let source_key = observation.key;
    let source_is_keyed = source_key.is_some();
    let default_kind = match entity.entity_type {
        EntityType::Repo => MemoryKind::RepositoryFact,
        _ => MemoryKind::ProjectFact,
    };
    let kind =
        classify_from_key_or_content(source_key.as_deref(), &observation.content, default_kind);
    let title = title_from_key_or_content(source_key.as_deref(), &observation.content);
    let scope = MemoryScope::Entity {
        entity_id: Some(entity.id),
        entity_name: entity.name.clone(),
    };
    build_candidate(CandidateInput {
        source_kind: MigrationSourceKind::EntityObservation,
        source_id: observation.id.to_string(),
        source_label: format!("entity:{} observation", entity.name),
        source_key,
        title,
        content: observation.content,
        proposed_kind: kind,
        proposed_scope: scope,
        source_created_at: observation.created_at,
        source_updated_at: observation.updated_at,
        source_is_keyed,
        source_has_project: false,
        force_disposition: Some(MigrationDisposition::Quarantine),
        base_reasons: vec![
            "Layer 1 entity observation maps to entity-scoped memory.".into(),
            "Entity observations may be linked broadly across projects; quarantine until scope is confirmed."
                .into(),
        ],
        now,
    })
}

fn candidate_from_session_summary(
    session: &Session,
    summary: &str,
    now: OffsetDateTime,
) -> MigrationCandidate {
    build_candidate(CandidateInput {
        source_kind: MigrationSourceKind::SessionSummary,
        source_id: format!("{}:summary", session.id),
        source_label: format!("session:{} summary", session.id),
        source_key: Some("summary".to_string()),
        title: title_from_key_or_content(Some("session.summary"), summary),
        content: summary.to_string(),
        proposed_kind: MemoryKind::Handoff,
        proposed_scope: scope_from_session(session),
        source_created_at: session.started_at,
        source_updated_at: session.ended_at.unwrap_or(session.started_at),
        source_is_keyed: true,
        source_has_project: session.project.is_some(),
        force_disposition: Some(MigrationDisposition::Quarantine),
        base_reasons: vec![
            "Session summary may become handoff memory after review.".into(),
            "Session summaries are generated rollups; quarantine until a stable project memory is distilled."
                .into(),
        ],
        now,
    })
}

fn candidate_from_session_key_decision(
    session: &Session,
    index: usize,
    decision: &str,
    now: OffsetDateTime,
) -> MigrationCandidate {
    build_candidate(CandidateInput {
        source_kind: MigrationSourceKind::SessionKeyDecision,
        source_id: format!("{}:decision:{}", session.id, index),
        source_label: format!("session:{} key decision", session.id),
        source_key: Some("key_decision".to_string()),
        title: title_from_key_or_content(Some("session.key_decision"), decision),
        content: decision.to_string(),
        proposed_kind: MemoryKind::Decision,
        proposed_scope: scope_from_session(session),
        source_created_at: session.started_at,
        source_updated_at: session.ended_at.unwrap_or(session.started_at),
        source_is_keyed: true,
        source_has_project: session.project.is_some(),
        force_disposition: None,
        base_reasons: vec!["Session key decision maps to decision memory.".into()],
        now,
    })
}

fn candidate_from_session_event(
    session: &Session,
    event: Event,
    now: OffsetDateTime,
) -> MigrationCandidate {
    let (kind, force_disposition, mut reasons) = match event.event_type {
        EventType::Decision => (
            MemoryKind::Decision,
            None,
            vec!["Decision event maps to decision memory.".to_string()],
        ),
        EventType::Observation | EventType::Milestone => (
            classify_from_key_or_content(None, &event.content, MemoryKind::SessionInsight),
            Some(MigrationDisposition::Quarantine),
            vec![
                "Session observation/milestone may become session insight.".to_string(),
                "Generic session events require distillation before becoming active memory."
                    .to_string(),
            ],
        ),
        EventType::Error => (
            MemoryKind::Limitation,
            Some(MigrationDisposition::Quarantine),
            vec![
                "Error event may describe a limitation or migration caveat.".to_string(),
                "Error events require validation before becoming active limitations.".to_string(),
            ],
        ),
        EventType::Command | EventType::ToolUse | EventType::FileChange => (
            MemoryKind::SessionInsight,
            Some(MigrationDisposition::Skip),
            vec!["Operational trace events are skipped unless manually promoted.".to_string()],
        ),
        EventType::Custom(_) => (
            MemoryKind::SessionInsight,
            Some(MigrationDisposition::Quarantine),
            vec!["Custom event type needs human classification.".to_string()],
        ),
    };
    if event.context.is_some() {
        reasons.push("Event has additional context that should be reviewed.".to_string());
    }

    build_candidate(CandidateInput {
        source_kind: MigrationSourceKind::SessionEvent,
        source_id: event.id.to_string(),
        source_label: format!("session:{} event:{}", session.id, event.event_type),
        source_key: Some(event.event_type.to_string()),
        title: title_from_key_or_content(Some(&event.event_type.to_string()), &event.content),
        content: event.content,
        proposed_kind: kind,
        proposed_scope: scope_from_session(session),
        source_created_at: event.timestamp,
        source_updated_at: event.timestamp,
        source_is_keyed: true,
        source_has_project: session.project.is_some(),
        force_disposition,
        base_reasons: reasons,
        now,
    })
}

struct CandidateInput {
    source_kind: MigrationSourceKind,
    source_id: String,
    source_label: String,
    source_key: Option<String>,
    title: String,
    content: String,
    proposed_kind: MemoryKind,
    proposed_scope: MemoryScope,
    source_created_at: OffsetDateTime,
    source_updated_at: OffsetDateTime,
    source_is_keyed: bool,
    source_has_project: bool,
    force_disposition: Option<MigrationDisposition>,
    base_reasons: Vec<String>,
    now: OffsetDateTime,
}

fn build_candidate(input: CandidateInput) -> MigrationCandidate {
    let mut reasons = input.base_reasons;
    let content_len = input.content.trim().chars().count();
    let staleness_days = (input.now - input.source_updated_at).whole_days().max(0);
    let mut confidence = 0.55_f32;
    let mut disposition = input
        .force_disposition
        .unwrap_or(MigrationDisposition::Review);

    if input.source_is_keyed {
        confidence += 0.10;
    } else {
        confidence -= 0.10;
        reasons.push("Source has no semantic key.".to_string());
    }

    if input.source_has_project {
        confidence += 0.05;
    } else if matches!(
        input.source_kind,
        MigrationSourceKind::SessionSummary
            | MigrationSourceKind::SessionKeyDecision
            | MigrationSourceKind::SessionEvent
    ) {
        confidence -= 0.10;
        reasons.push("Session source has no project scope.".to_string());
    }

    if content_len < 20 {
        confidence -= 0.25;
        disposition = MigrationDisposition::Quarantine;
        reasons.push("Content is too short to migrate without review.".to_string());
    }

    if staleness_days > 365 {
        confidence -= 0.15;
        reasons.push("Source is more than one year old.".to_string());
    } else if staleness_days > 90 {
        confidence -= 0.05;
        reasons.push("Source is more than 90 days old.".to_string());
    }

    if is_stale_status_key(input.source_key.as_deref(), staleness_days) {
        confidence -= 0.20;
        disposition = MigrationDisposition::Quarantine;
        reasons.push("Status/progress memory is stale and should be recalibrated.".to_string());
    }

    if matches!(input.force_disposition, Some(MigrationDisposition::Skip)) {
        confidence = confidence.min(0.30);
    }

    MigrationCandidate {
        source_kind: input.source_kind,
        source_id: input.source_id,
        source_label: input.source_label,
        source_key: input.source_key,
        title: input.title,
        content: input.content,
        proposed_kind: input.proposed_kind,
        proposed_scope: input.proposed_scope,
        proposed_origin: ClaimOrigin::Migrated,
        confidence: confidence.clamp(0.0, 1.0),
        staleness_days,
        disposition,
        reasons,
        source_created_at: input.source_created_at,
        source_updated_at: input.source_updated_at,
    }
}

fn classify_from_key_or_content(
    key: Option<&str>,
    content: &str,
    default_kind: MemoryKind,
) -> MemoryKind {
    if let Some(kind) = key.and_then(|key| classify_from_key(key, &default_kind)) {
        return kind;
    }

    let haystack =
        key.map(str::to_string).unwrap_or_default().to_lowercase() + " " + &content.to_lowercase();

    if contains_any(
        &haystack,
        &[
            "preference:",
            "preferences:",
            "user prefers",
            "user preference",
            "user.taste",
        ],
    ) {
        MemoryKind::Preference
    } else if contains_any(&haystack, &["rule", "rules", "policy", "must always"]) {
        MemoryKind::Rule
    } else if contains_any(&haystack, &["decision", "decisions", "adr"]) {
        MemoryKind::Decision
    } else if contains_any(
        &haystack,
        &["limitation", "limitations", "gotcha", "gotchas", "risk"],
    ) {
        MemoryKind::Limitation
    } else if contains_any(
        &haystack,
        &["handoff", "status.current", "status.next", "next-step"],
    ) {
        MemoryKind::Handoff
    } else {
        default_kind
    }
}

fn classify_from_key(key: &str, default_kind: &MemoryKind) -> Option<MemoryKind> {
    let key = key.to_lowercase();
    if key.starts_with("preference.") || key.starts_with("preferences.") || key == "user.taste" {
        Some(MemoryKind::Preference)
    } else if key.starts_with("rule.") || key.starts_with("rules.") {
        Some(MemoryKind::Rule)
    } else if key.starts_with("decision.")
        || key.starts_with("decisions.")
        || key.contains(".decision.")
        || key.contains(".decisions.")
    {
        Some(MemoryKind::Decision)
    } else if key.starts_with("gotcha.")
        || key.starts_with("gotchas.")
        || key.starts_with("limitation.")
        || key.starts_with("limitations.")
    {
        Some(MemoryKind::Limitation)
    } else if key.starts_with("handoff.")
        || key.starts_with("status.")
        || key.contains("next-step")
        || key.contains("next_steps")
    {
        Some(MemoryKind::Handoff)
    } else if key.starts_with("design.")
        || key.starts_with("architecture.")
        || key.starts_with("research.")
        || key.starts_with("implementation.")
        || key.starts_with("artifacts.")
    {
        Some(default_kind.clone())
    } else {
        None
    }
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

fn scope_from_session(session: &Session) -> MemoryScope {
    session
        .project
        .as_ref()
        .map(|project_name| MemoryScope::Project {
            project_id: None,
            project_name: project_name.clone(),
        })
        .unwrap_or(MemoryScope::Session {
            session_id: session.id,
        })
}

fn title_from_key_or_content(key: Option<&str>, content: &str) -> String {
    if let Some(key) = key.filter(|value| !value.trim().is_empty()) {
        return title_case_key(key);
    }

    let first_line = content
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("Untitled migrated memory");
    truncate_title(first_line)
}

fn title_case_key(key: &str) -> String {
    let words = key
        .replace(['.', '_', '-'], " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>();
    words.join(" ")
}

fn truncate_title(value: &str) -> String {
    let mut title: String = value.chars().take(80).collect();
    if value.chars().count() > 80 {
        title.push_str("...");
    }
    title
}

fn is_stale_status_key(key: Option<&str>, staleness_days: i64) -> bool {
    staleness_days > 30
        && key
            .map(|key| {
                let key = key.to_lowercase();
                key.starts_with("status.") || key.contains("next-step") || key.contains("progress")
            })
            .unwrap_or(false)
}

fn is_transient_project_observation_key(key: Option<&str>) -> bool {
    key.map(|key| {
        let key = key.to_lowercase();
        key.starts_with("status.")
            || key.starts_with("artifacts.")
            || key.starts_with("handoff.")
            || key == "decisions.recent"
            || key.starts_with("decisions.recent.")
    })
    .unwrap_or(false)
}

fn disposition_rank(disposition: MigrationDisposition) -> u8 {
    match disposition {
        MigrationDisposition::Review => 0,
        MigrationDisposition::Quarantine => 1,
        MigrationDisposition::Skip => 2,
    }
}

fn count_by_source_kind(candidates: &[MigrationCandidate]) -> BTreeMap<String, usize> {
    count_by(
        candidates
            .iter()
            .map(|candidate| candidate.source_kind.to_string()),
    )
}

fn count_by_disposition(candidates: &[MigrationCandidate]) -> BTreeMap<String, usize> {
    count_by(
        candidates
            .iter()
            .map(|candidate| candidate.disposition.to_string()),
    )
}

fn count_by_memory_kind(candidates: &[MigrationCandidate]) -> BTreeMap<String, usize> {
    count_by(
        candidates
            .iter()
            .map(|candidate| candidate.proposed_kind.to_string()),
    )
}

fn count_by(values: impl Iterator<Item = String>) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for value in values {
        *counts.entry(value).or_insert(0) += 1;
    }
    counts
}

fn write_review_batch(
    root: &Path,
    inventory: MigrationInventory,
) -> IndexResult<MigrationReviewExport> {
    fs::create_dir_all(root)?;

    let mut export = MigrationReviewExport {
        root: root.display().to_string(),
        files_written: Vec::new(),
        files_skipped: Vec::new(),
        inventory,
    };

    write_review_file(
        root,
        Path::new("index.md").to_path_buf(),
        &review_index_page(&export.inventory),
        &mut export,
    )?;

    let candidates = export.inventory.candidates.clone();
    for (index, candidate) in candidates.iter().enumerate() {
        write_review_file(
            root,
            candidate_review_path(index, candidate),
            &candidate_review_page(index, candidate),
            &mut export,
        )?;
    }

    export.files_written.sort();
    export.files_skipped.sort();
    Ok(export)
}

fn review_index_page(inventory: &MigrationInventory) -> String {
    let mut output = review_frontmatter(
        "migration_review_index",
        vec![
            (
                "generated_at".to_string(),
                yaml_string(&format_time(inventory.generated_at)),
            ),
            (
                "project_filter".to_string(),
                yaml_string(inventory.project_filter.as_deref().unwrap_or("")),
            ),
            (
                "sources_scanned".to_string(),
                inventory.sources_scanned.to_string(),
            ),
            (
                "total_candidates".to_string(),
                inventory.total_candidates.to_string(),
            ),
            (
                "returned_candidates".to_string(),
                inventory.returned_candidates.to_string(),
            ),
        ],
    );

    output.push_str("# Migration Review Batch\n\n");
    output.push_str("## Summary\n\n");
    if let Some(project_filter) = &inventory.project_filter {
        output.push_str(&format!("- Project filter: {}\n", project_filter));
    }
    output.push_str(&format!(
        "- Sources scanned: {}\n",
        inventory.sources_scanned
    ));
    output.push_str(&format!(
        "- Total candidates: {}\n",
        inventory.total_candidates
    ));
    output.push_str(&format!(
        "- Returned candidates: {}\n",
        inventory.returned_candidates
    ));
    output.push_str(&format!("- Truncated: {}\n\n", inventory.truncated));

    append_count_section(&mut output, "Disposition Counts", &inventory.by_disposition);
    append_count_section(&mut output, "Source Counts", &inventory.by_source_kind);
    append_count_section(
        &mut output,
        "Proposed Memory Kinds",
        &inventory.by_memory_kind,
    );

    if !inventory.warnings.is_empty() {
        output.push_str("## Warnings\n\n");
        for warning in &inventory.warnings {
            output.push_str(&format!("- {}\n", warning));
        }
        output.push('\n');
    }

    output.push_str("## Candidates\n\n");
    if inventory.candidates.is_empty() {
        output.push_str("No migration candidates in this batch.\n");
    } else {
        for (index, candidate) in inventory.candidates.iter().enumerate() {
            output.push_str(&format!(
                "- [{}]({}) - {} - {} - {:.2}\n",
                escape_link_text(&candidate.title),
                path_to_markdown(&candidate_review_path(index, candidate)),
                candidate.disposition,
                candidate.proposed_kind,
                candidate.confidence
            ));
        }
    }

    output
}

fn candidate_review_page(index: usize, candidate: &MigrationCandidate) -> String {
    let mut output = review_frontmatter(
        "migration_candidate_review",
        vec![
            ("candidate_number".to_string(), (index + 1).to_string()),
            (
                "source_kind".to_string(),
                yaml_string(&candidate.source_kind.to_string()),
            ),
            ("source_id".to_string(), yaml_string(&candidate.source_id)),
            (
                "disposition".to_string(),
                yaml_string(&candidate.disposition.to_string()),
            ),
            (
                "proposed_kind".to_string(),
                yaml_string(&candidate.proposed_kind.to_string()),
            ),
            (
                "confidence".to_string(),
                format!("{:.3}", candidate.confidence),
            ),
            (
                "staleness_days".to_string(),
                candidate.staleness_days.to_string(),
            ),
        ],
    );

    output.push_str(&format!("# {}\n\n", candidate.title));
    output.push_str("## Review Decision\n\n");
    output.push_str("- [ ] Accept for migration\n");
    output.push_str("- [ ] Accept with edits\n");
    output.push_str("- [ ] Quarantine\n");
    output.push_str("- [ ] Reject / skip\n\n");
    output.push_str("Reviewer notes:\n\n");

    output.push_str("## Proposed Memory\n\n");
    output.push_str(&format!("- Kind: {}\n", candidate.proposed_kind));
    output.push_str(&format!(
        "- Scope: {}\n",
        scope_label(&candidate.proposed_scope)
    ));
    output.push_str(&format!(
        "- Origin: {}\n",
        origin_label(&candidate.proposed_origin)
    ));
    output.push_str(&format!("- Confidence: {:.3}\n", candidate.confidence));
    output.push_str(&format!("- Disposition: {}\n", candidate.disposition));
    output.push_str(&format!(
        "- Staleness days: {}\n\n",
        candidate.staleness_days
    ));

    output.push_str("## Content\n\n");
    output.push_str(&candidate.content);
    output.push_str("\n\n## Source\n\n");
    output.push_str(&format!("- Source kind: {}\n", candidate.source_kind));
    output.push_str(&format!("- Source ID: {}\n", candidate.source_id));
    output.push_str(&format!("- Source label: {}\n", candidate.source_label));
    if let Some(key) = &candidate.source_key {
        output.push_str(&format!("- Source key: {}\n", key));
    }
    output.push_str(&format!(
        "- Source created: {}\n",
        format_time(candidate.source_created_at)
    ));
    output.push_str(&format!(
        "- Source updated: {}\n\n",
        format_time(candidate.source_updated_at)
    ));

    output.push_str("## Reasons\n\n");
    if candidate.reasons.is_empty() {
        output.push_str("No reasons recorded.\n");
    } else {
        for reason in &candidate.reasons {
            output.push_str(&format!("- {}\n", reason));
        }
    }

    output.push_str("\n## Machine Record\n\n");
    output.push_str("```json\n");
    output.push_str(
        &serde_json::to_string_pretty(candidate)
            .expect("migration candidate JSON serialization should succeed"),
    );
    output.push_str("\n```\n");

    output
}

fn append_count_section(output: &mut String, title: &str, counts: &BTreeMap<String, usize>) {
    output.push_str(&format!("## {}\n\n", title));
    if counts.is_empty() {
        output.push_str("No entries.\n\n");
    } else {
        for (key, count) in counts {
            output.push_str(&format!("- {}: {}\n", key, count));
        }
        output.push('\n');
    }
}

fn review_frontmatter(page_type: &str, fields: Vec<(String, String)>) -> String {
    let mut output = String::new();
    output.push_str("---\n");
    output.push_str(&format!(
        "generated_by: {}\n",
        yaml_string(REVIEW_GENERATED_BY)
    ));
    output.push_str(&format!("page_type: {}\n", yaml_string(page_type)));
    for (key, value) in fields {
        output.push_str(&format!("{key}: {value}\n"));
    }
    output.push_str("---\n\n");
    output.push_str(REVIEW_GENERATED_MARKER);
    output.push_str("\n\n");
    output
}

fn write_review_file(
    root: &Path,
    relative_path: PathBuf,
    contents: &str,
    export: &mut MigrationReviewExport,
) -> IndexResult<()> {
    let path = root.join(&relative_path);
    if path.exists() {
        let existing = fs::read(&path)?;
        if !String::from_utf8_lossy(&existing).contains(REVIEW_GENERATED_MARKER) {
            export.files_skipped.push(path_to_markdown(&relative_path));
            return Ok(());
        }
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, contents)?;
    export.files_written.push(path_to_markdown(&relative_path));
    Ok(())
}

fn candidate_review_path(index: usize, candidate: &MigrationCandidate) -> PathBuf {
    Path::new("candidates").join(format!(
        "{:04}-{}-{}.md",
        index + 1,
        candidate.disposition,
        slugify(&candidate.title)
    ))
}

fn collect_candidate_review_files(root: &Path) -> IndexResult<Vec<PathBuf>> {
    let candidates_dir = root.join("candidates");
    if !candidates_dir.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    for entry in fs::read_dir(candidates_dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|extension| extension == "md") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn parse_review_candidate_page(
    contents: &str,
    relative_path: &str,
    report: &mut MigrationReviewApply,
) -> IndexResult<Option<ParsedReviewCandidate>> {
    if !contents.contains(REVIEW_GENERATED_MARKER) {
        report.files_skipped.push(relative_path.to_string());
        report.warnings.push(format!(
            "{relative_path}: skipped non-generated review file"
        ));
        return Ok(None);
    }

    let decisions = selected_review_decisions(contents);
    if decisions.is_empty() {
        report
            .files_with_no_decision
            .push(relative_path.to_string());
        return Ok(None);
    }
    if decisions.len() > 1 {
        report.files_with_conflicts.push(relative_path.to_string());
        report.warnings.push(format!(
            "{relative_path}: multiple review decisions selected"
        ));
        return Ok(None);
    }

    let candidate = parse_machine_candidate(contents).map_err(|error| {
        IndexError::Parse(format!(
            "{relative_path}: invalid migration candidate record: {error}"
        ))
    })?;

    Ok(Some(ParsedReviewCandidate {
        decision: decisions[0],
        edited_title: first_markdown_heading(contents),
        edited_content: markdown_section(contents, "## Content"),
        edited_kind: bullet_value(contents, "Kind").map(|value| MemoryKind::parse(&value)),
        candidate,
    }))
}

fn selected_review_decisions(contents: &str) -> Vec<ReviewDecision> {
    contents
        .lines()
        .filter_map(|line| {
            let normalized = line.trim().to_lowercase();
            if !(normalized.starts_with("- [x]") || normalized.starts_with("- [X]")) {
                return None;
            }

            if normalized.contains("accept for migration") {
                Some(ReviewDecision::Accept)
            } else if normalized.contains("accept with edits") {
                Some(ReviewDecision::AcceptWithEdits)
            } else if normalized.contains("quarantine") {
                Some(ReviewDecision::Quarantine)
            } else if normalized.contains("reject / skip") {
                Some(ReviewDecision::Reject)
            } else {
                None
            }
        })
        .collect()
}

fn parse_machine_candidate(contents: &str) -> Result<MigrationCandidate, serde_json::Error> {
    let json = machine_record_json(contents).unwrap_or_default();
    serde_json::from_str(json)
}

fn machine_record_json(contents: &str) -> Option<&str> {
    let heading_start = contents.find(MACHINE_RECORD_HEADING)?;
    let after_heading = &contents[heading_start + MACHINE_RECORD_HEADING.len()..];
    let fence_start = after_heading.find(MACHINE_RECORD_FENCE)?;
    let after_fence = &after_heading[fence_start + MACHINE_RECORD_FENCE.len()..];
    let json_start = after_fence.strip_prefix('\n').unwrap_or(after_fence);
    let fence_end = json_start.find("```")?;
    Some(json_start[..fence_end].trim())
}

fn first_markdown_heading(contents: &str) -> Option<String> {
    contents
        .lines()
        .find_map(|line| line.trim().strip_prefix("# "))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn markdown_section(contents: &str, heading: &str) -> Option<String> {
    let start = contents.find(heading)?;
    let after_heading = &contents[start + heading.len()..];
    let body_start = after_heading
        .strip_prefix("\n\n")
        .or_else(|| after_heading.strip_prefix('\n'))
        .unwrap_or(after_heading);
    let end = body_start.find("\n## ").unwrap_or(body_start.len());
    let section = body_start[..end].trim();
    (!section.is_empty()).then(|| section.to_string())
}

fn bullet_value(contents: &str, key: &str) -> Option<String> {
    let prefix = format!("- {key}:");
    contents.lines().find_map(|line| {
        line.trim()
            .strip_prefix(&prefix)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    })
}

fn migration_source_tag(candidate: &MigrationCandidate) -> String {
    format!(
        "migration-source:{}:{}",
        candidate.source_kind, candidate.source_id
    )
}

fn relative_markdown_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map(path_to_markdown)
        .unwrap_or_else(|_| path.display().to_string())
}

fn scope_label(scope: &MemoryScope) -> String {
    match scope {
        MemoryScope::Global => "global".to_string(),
        MemoryScope::User => "user".to_string(),
        MemoryScope::Project { project_name, .. } => format!("project:{project_name}"),
        MemoryScope::Task {
            project_name,
            task_name,
            ..
        } => project_name
            .as_ref()
            .map(|project| format!("task:{project}/{task_name}"))
            .unwrap_or_else(|| format!("task:{task_name}")),
        MemoryScope::Entity { entity_name, .. } => format!("entity:{entity_name}"),
        MemoryScope::Repository {
            repository_id,
            remote_url,
            local_path,
        } => repository_id
            .map(|id| format!("repository:{id}"))
            .or_else(|| remote_url.as_ref().map(|url| format!("repository:{url}")))
            .or_else(|| local_path.as_ref().map(|path| format!("repository:{path}")))
            .unwrap_or_else(|| "repository".to_string()),
        MemoryScope::Session { session_id } => format!("session:{session_id}"),
        MemoryScope::Custom { name } => format!("custom:{name}"),
    }
}

fn origin_label(origin: &ClaimOrigin) -> String {
    match origin {
        ClaimOrigin::UserStated => "user_stated".to_string(),
        ClaimOrigin::UserCorrected => "user_corrected".to_string(),
        ClaimOrigin::AgentObserved => "agent_observed".to_string(),
        ClaimOrigin::AgentInferred => "agent_inferred".to_string(),
        ClaimOrigin::ToolResult => "tool_result".to_string(),
        ClaimOrigin::Imported => "imported".to_string(),
        ClaimOrigin::Migrated => "migrated".to_string(),
        ClaimOrigin::GeneratedSummary => "generated_summary".to_string(),
        ClaimOrigin::Custom(value) => value.clone(),
    }
}

fn path_to_markdown(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut previous_dash = false;
    for ch in value.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            slug.push(ch);
            previous_dash = false;
        } else if !previous_dash {
            slug.push('-');
            previous_dash = true;
        }
        if slug.len() >= 80 {
            break;
        }
    }

    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "untitled".to_string()
    } else {
        slug
    }
}

fn yaml_string(value: &str) -> String {
    serde_json::to_string(value).expect("string serialization cannot fail")
}

fn escape_link_text(value: &str) -> String {
    value.replace('\\', "\\\\").replace(']', "\\]")
}

fn format_time(value: OffsetDateTime) -> String {
    value
        .format(&time::format_description::well_known::Rfc3339)
        .expect("RFC3339 timestamp formatting should succeed")
}

#[cfg(test)]
mod tests {
    use super::*;
    use engram_core::entity::{Entity, EntityType, Observation};
    use engram_core::memory::{Harness, ModelIdentity};
    use engram_core::session::{Event, EventType, Session};
    use engram_core::work::{Project, ProjectObservation, Task, TaskObservation};
    use engram_store::{connect_and_init, MemoryRepo, StoreConfig};
    use tempfile::tempdir;

    async fn setup_service() -> (
        MigrationService,
        EntityRepo,
        SessionRepo,
        WorkRepo,
        MemoryRepo,
    ) {
        let config = StoreConfig::memory();
        let db = connect_and_init(&config).await.unwrap();
        (
            MigrationService::new(db.clone()),
            EntityRepo::new(db.clone()),
            SessionRepo::new(db.clone()),
            WorkRepo::new(db.clone()),
            MemoryRepo::new(db),
        )
    }

    fn writer() -> WriterProvenance {
        WriterProvenance::agent(
            Harness::Other("migration_test".to_string()),
            ModelIdentity::new("engram", "migration-review-apply"),
        )
    }

    #[tokio::test]
    async fn inventory_classifies_work_entity_and_session_sources() {
        let (service, entity_repo, session_repo, work_repo, _memory_repo) = setup_service().await;

        let project = Project::new("engram");
        work_repo.create_project(&project).await.unwrap();
        work_repo
            .add_project_observation(
                &ProjectObservation::new(project.id, "Extend Engram rather than rebuild.")
                    .with_key("decisions.memory-os"),
            )
            .await
            .unwrap();

        let task = Task::new(project.id, "migration dry run").with_jira_key("ENG-1");
        work_repo.create_task(&task).await.unwrap();
        work_repo
            .add_task_observation(
                &TaskObservation::new(task.id, "Inventory must not write MemoryItem records.")
                    .with_key("rules.non-destructive"),
            )
            .await
            .unwrap();

        let entity = Entity::new("engram", EntityType::Repo);
        entity_repo.save_entity(&entity).await.unwrap();
        work_repo
            .connect_project_entity(
                &project.id,
                &entity.id,
                &engram_core::work::ProjectEntityRelation::Involves,
            )
            .await
            .unwrap();
        entity_repo
            .add_observation(
                &Observation::new(entity.id, "Repository topology is tracked separately.")
                    .with_key("architecture.repository-topology"),
            )
            .await
            .unwrap();

        let session = Session::new()
            .with_project("engram")
            .with_agent("codex")
            .with_goal("Implement inventory");
        session_repo.save_session(&session).await.unwrap();
        session_repo
            .add_event(&Event::new(
                session.id,
                EventType::Decision,
                "agent",
                "Use a dry-run review queue before migration.",
            ))
            .await
            .unwrap();

        let inventory = service
            .inventory(MigrationInventoryOptions {
                project_filter: Some("engram".to_string()),
                ..MigrationInventoryOptions::all()
            })
            .await
            .unwrap();

        assert_eq!(inventory.sources_scanned, 4);
        assert_eq!(inventory.total_candidates, 4);
        assert_eq!(inventory.by_disposition.get("review"), Some(&3));
        assert_eq!(inventory.by_disposition.get("quarantine"), Some(&1));
        assert!(inventory
            .candidates
            .iter()
            .any(|candidate| candidate.proposed_kind == MemoryKind::Decision));
        assert!(inventory
            .candidates
            .iter()
            .any(|candidate| candidate.proposed_kind == MemoryKind::Rule));
        assert!(inventory
            .candidates
            .iter()
            .any(
                |candidate| candidate.proposed_kind == MemoryKind::RepositoryFact
                    && candidate.disposition == MigrationDisposition::Quarantine
            ));
    }

    #[tokio::test]
    async fn inventory_skips_low_value_session_trace_events() {
        let (service, _entity_repo, session_repo, _work_repo, _memory_repo) = setup_service().await;
        let session = Session::new().with_agent("codex");
        session_repo.save_session(&session).await.unwrap();
        session_repo
            .add_event(&Event::new(
                session.id,
                EventType::ToolUse,
                "agent",
                "Called shell command",
            ))
            .await
            .unwrap();

        let inventory = service
            .inventory(MigrationInventoryOptions::all())
            .await
            .unwrap();

        assert_eq!(inventory.sources_scanned, 1);
        assert_eq!(inventory.total_candidates, 1);
        assert_eq!(inventory.by_disposition.get("skip"), Some(&1));
        assert_eq!(
            inventory.candidates[0].disposition,
            MigrationDisposition::Skip
        );
    }

    #[tokio::test]
    async fn inventory_quarantines_session_rollups_and_generic_events() {
        let (service, _entity_repo, session_repo, _work_repo, _memory_repo) = setup_service().await;
        let mut session = Session::new()
            .with_project("engram")
            .with_agent("codex")
            .with_goal("Migrate memory safely");
        session.summary = Some("Implemented a large feature and ran validation.".to_string());
        session_repo.save_session(&session).await.unwrap();
        session_repo
            .add_event(&Event::new(
                session.id,
                EventType::Milestone,
                "agent",
                "Finished an implementation milestone.",
            ))
            .await
            .unwrap();
        session_repo
            .add_event(&Event::new(
                session.id,
                EventType::Decision,
                "agent",
                "Keep migrations non-destructive by default.",
            ))
            .await
            .unwrap();

        let inventory = service
            .inventory(MigrationInventoryOptions {
                project_filter: Some("engram".to_string()),
                include_entity_observations: false,
                include_session_history: true,
                include_work_observations: false,
                limit: None,
            })
            .await
            .unwrap();

        assert_eq!(inventory.sources_scanned, 3);
        assert_eq!(inventory.total_candidates, 3);
        assert_eq!(inventory.by_disposition.get("review"), Some(&1));
        assert_eq!(inventory.by_disposition.get("quarantine"), Some(&2));
        assert!(inventory.candidates.iter().any(|candidate| {
            candidate.source_kind == MigrationSourceKind::SessionEvent
                && candidate.proposed_kind == MemoryKind::Decision
                && candidate.disposition == MigrationDisposition::Review
        }));
        assert!(inventory.candidates.iter().any(|candidate| {
            candidate.source_kind == MigrationSourceKind::SessionSummary
                && candidate.disposition == MigrationDisposition::Quarantine
        }));
    }

    #[tokio::test]
    async fn inventory_quarantines_transient_project_observation_rollups() {
        let (service, _entity_repo, _session_repo, work_repo, _memory_repo) = setup_service().await;
        let project = Project::new("engram");
        work_repo.create_project(&project).await.unwrap();
        for (key, content) in [
            (
                "status.current",
                "Current status should be recalibrated before migration.",
            ),
            (
                "artifacts.project-book",
                "Generated artifact references should not become active facts by default.",
            ),
            (
                "decisions.recent",
                "Recent decision rollups need distillation into individual decisions.",
            ),
            (
                "design.memory-os-provenance",
                "Writer provenance is part of the stable Memory OS design.",
            ),
        ] {
            work_repo
                .add_project_observation(
                    &ProjectObservation::new(project.id, content).with_key(key),
                )
                .await
                .unwrap();
        }

        let inventory = service
            .inventory(MigrationInventoryOptions {
                project_filter: Some("engram".to_string()),
                include_entity_observations: false,
                include_session_history: false,
                include_work_observations: true,
                limit: None,
            })
            .await
            .unwrap();

        assert_eq!(inventory.sources_scanned, 4);
        assert_eq!(inventory.total_candidates, 4);
        assert_eq!(inventory.by_disposition.get("review"), Some(&1));
        assert_eq!(inventory.by_disposition.get("quarantine"), Some(&3));
        assert!(inventory.candidates.iter().any(|candidate| {
            candidate.source_key.as_deref() == Some("design.memory-os-provenance")
                && candidate.disposition == MigrationDisposition::Review
        }));
        assert!(inventory.candidates.iter().all(|candidate| {
            !matches!(
                candidate.source_key.as_deref(),
                Some("status.current" | "artifacts.project-book" | "decisions.recent")
            ) || candidate.disposition == MigrationDisposition::Quarantine
        }));
    }

    #[test]
    fn classification_prefers_semantic_key_over_incidental_content() {
        assert_eq!(
            classify_from_key_or_content(
                Some("design.memory-os-readiness-review"),
                "Confirm privacy policy for user preferences and migration risks.",
                MemoryKind::ProjectFact,
            ),
            MemoryKind::ProjectFact
        );
        assert_eq!(
            classify_from_key_or_content(
                Some("rules.non-destructive"),
                "Migration apply must be opt-in.",
                MemoryKind::ProjectFact,
            ),
            MemoryKind::Rule
        );
    }

    #[tokio::test]
    async fn export_review_batch_writes_index_and_candidate_pages() {
        let (service, _entity_repo, _session_repo, work_repo, _memory_repo) = setup_service().await;
        let project = Project::new("engram");
        work_repo.create_project(&project).await.unwrap();
        work_repo
            .add_project_observation(
                &ProjectObservation::new(project.id, "Migration review batches are generated.")
                    .with_key("decisions.review-export"),
            )
            .await
            .unwrap();

        let dir = tempdir().unwrap();
        let export = service
            .export_review_batch(dir.path(), MigrationInventoryOptions::all())
            .await
            .unwrap();

        assert_eq!(export.inventory.total_candidates, 1);
        assert_eq!(export.file_count(), 2);
        assert!(dir.path().join("index.md").exists());
        assert!(export
            .files_written
            .iter()
            .any(|path| path.starts_with("candidates/")));

        let index = fs::read_to_string(dir.path().join("index.md")).unwrap();
        assert!(index.contains(REVIEW_GENERATED_MARKER));
        assert!(index.contains("Migration Review Batch"));

        let candidate_path = export
            .files_written
            .iter()
            .find(|path| path.starts_with("candidates/"))
            .unwrap();
        let candidate = fs::read_to_string(dir.path().join(candidate_path)).unwrap();
        assert!(candidate.contains("- [ ] Accept for migration"));
        assert!(candidate.contains("Migration review batches are generated."));
        assert!(candidate.contains("## Machine Record"));
    }

    #[tokio::test]
    async fn export_review_batch_skips_user_owned_files() {
        let (service, _entity_repo, _session_repo, _work_repo, _memory_repo) =
            setup_service().await;
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("index.md"), "# User notes\n").unwrap();

        let export = service
            .export_review_batch(dir.path(), MigrationInventoryOptions::all())
            .await
            .unwrap();

        assert_eq!(export.inventory.total_candidates, 0);
        assert_eq!(export.files_written.len(), 0);
        assert_eq!(export.files_skipped, vec!["index.md"]);
        assert_eq!(
            fs::read_to_string(dir.path().join("index.md")).unwrap(),
            "# User notes\n"
        );
    }

    #[tokio::test]
    async fn apply_review_batch_dry_run_parses_accepted_candidate_without_writes() {
        let (service, _entity_repo, _session_repo, work_repo, memory_repo) = setup_service().await;
        let project = Project::new("engram");
        work_repo.create_project(&project).await.unwrap();
        work_repo
            .add_project_observation(
                &ProjectObservation::new(project.id, "Dry-run apply should not write memory.")
                    .with_key("rules.migration-dry-run"),
            )
            .await
            .unwrap();

        let dir = tempdir().unwrap();
        let export = service
            .export_review_batch(dir.path(), MigrationInventoryOptions::all())
            .await
            .unwrap();
        check_first_candidate(dir.path(), &export, "Accept for migration");

        let apply = service
            .apply_review_batch(
                dir.path(),
                MigrationReviewApplyOptions {
                    dry_run: true,
                    writer: writer(),
                    create_commit: true,
                },
            )
            .await
            .unwrap();

        assert_eq!(apply.planned_count(), 1);
        assert_eq!(apply.written_count(), 0);
        assert_eq!(apply.accepted_count, 1);
        assert!(apply.commit.is_none());
        assert!(memory_repo
            .list_memory_items(None, None)
            .await
            .unwrap()
            .is_empty());
    }

    #[tokio::test]
    async fn apply_review_batch_writes_once_and_creates_commit() {
        let (service, _entity_repo, _session_repo, work_repo, memory_repo) = setup_service().await;
        let project = Project::new("engram");
        work_repo.create_project(&project).await.unwrap();
        work_repo
            .add_project_observation(
                &ProjectObservation::new(project.id, "Apply accepted reviews into Memory OS.")
                    .with_key("decisions.review-apply"),
            )
            .await
            .unwrap();

        let dir = tempdir().unwrap();
        let export = service
            .export_review_batch(dir.path(), MigrationInventoryOptions::all())
            .await
            .unwrap();
        check_first_candidate(dir.path(), &export, "Accept for migration");

        let apply = service
            .apply_review_batch(
                dir.path(),
                MigrationReviewApplyOptions {
                    dry_run: false,
                    writer: writer(),
                    create_commit: true,
                },
            )
            .await
            .unwrap();

        assert_eq!(apply.planned_count(), 1);
        assert_eq!(apply.written_count(), 1);
        assert!(apply.commit.is_some());
        let items = memory_repo.list_memory_items(None, None).await.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].status, MemoryStatus::Active);
        assert!(items[0]
            .tags
            .iter()
            .any(|tag| tag.starts_with("migration-source:project_observation:")));

        let second_apply = service
            .apply_review_batch(
                dir.path(),
                MigrationReviewApplyOptions {
                    dry_run: false,
                    writer: writer(),
                    create_commit: true,
                },
            )
            .await
            .unwrap();
        assert_eq!(second_apply.planned_count(), 0);
        assert_eq!(second_apply.duplicate_count, 1);
        assert_eq!(
            memory_repo
                .list_memory_items(None, None)
                .await
                .unwrap()
                .len(),
            1
        );
    }

    fn check_first_candidate(root: &Path, export: &MigrationReviewExport, decision: &str) {
        let candidate_path = export
            .files_written
            .iter()
            .find(|path| path.starts_with("candidates/"))
            .unwrap();
        let path = root.join(candidate_path);
        let contents = fs::read_to_string(&path).unwrap();
        let checked = contents.replace(&format!("- [ ] {decision}"), &format!("- [x] {decision}"));
        fs::write(path, checked).unwrap();
    }
}
