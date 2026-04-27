//! Repository topology service.
//!
//! This service records source-control facts that can be resolved from a
//! current working directory: canonical repositories, local checkouts,
//! monorepo components, and project links.

use crate::error::{IndexError, IndexResult};
use engram_core::entity::Entity;
use engram_core::id::Id;
use engram_core::memory::{KnowledgeCommit, MemoryChange, MemoryChangeType, WriterProvenance};
use engram_core::repository::{
    GitRepository, LocalCheckout, MonorepoComponent, ProjectRepositoryLink, ProjectRepositoryRole,
    RepositoryContext,
};
use engram_core::session::{Event, Session};
use engram_core::work::{Pr, Project, ProjectObservation, Task, TaskObservation};
use engram_store::{Db, EntityRepo, MemoryRepo, RepositoryRepo, SessionRepo, WorkRepo};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use time::OffsetDateTime;
use tracing::info;

const REPOSITORY_REVIEW_GENERATED_BY: &str = "engram-memory-os";
const REPOSITORY_REVIEW_GENERATED_MARKER: &str =
    "<!-- engram:generated:file repository-migration-review-v1 -->";
const REPOSITORY_MACHINE_RECORD_HEADING: &str = "## Machine Record";
const REPOSITORY_MACHINE_RECORD_FENCE: &str = "```json";

/// Result of detecting a Git checkout from a working directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryDetection {
    /// Resolved repository context.
    pub context: RepositoryContext,
    /// Git root detected by `git rev-parse --show-toplevel`.
    pub detected_root: String,
    /// Warnings from optional detection steps.
    pub warnings: Vec<String>,
}

/// Options for repository topology migration inventory.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepositoryMigrationOptions {
    /// Optional project name filter.
    pub project_filter: Option<String>,
    /// Maximum candidates to return after inventorying all sources.
    pub limit: Option<usize>,
    /// Include Layer 1 entity descriptions and observations.
    pub include_entity_observations: bool,
    /// Include Layer 2 session goals, summaries, decisions, and events.
    pub include_session_history: bool,
    /// Include Layer 7 projects, tasks, PRs, and observations.
    pub include_work_records: bool,
}

impl RepositoryMigrationOptions {
    /// Options that scan all currently supported source layers.
    #[must_use]
    pub fn all() -> Self {
        Self {
            project_filter: None,
            limit: None,
            include_entity_observations: true,
            include_session_history: true,
            include_work_records: true,
        }
    }
}

/// Existing Engram source layer for repository topology migration evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepositoryMigrationSourceKind {
    /// Layer 1 entity description.
    EntityDescription,
    /// Layer 1 entity observation.
    EntityObservation,
    /// Layer 2 session field or decision.
    SessionRecord,
    /// Layer 2 session event.
    SessionEvent,
    /// Layer 7 project description.
    ProjectDescription,
    /// Layer 7 project observation.
    ProjectObservation,
    /// Layer 7 task description.
    TaskDescription,
    /// Layer 7 task observation.
    TaskObservation,
    /// Layer 7 PR URL/repository metadata.
    PullRequest,
}

impl std::fmt::Display for RepositoryMigrationSourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EntityDescription => write!(f, "entity_description"),
            Self::EntityObservation => write!(f, "entity_observation"),
            Self::SessionRecord => write!(f, "session_record"),
            Self::SessionEvent => write!(f, "session_event"),
            Self::ProjectDescription => write!(f, "project_description"),
            Self::ProjectObservation => write!(f, "project_observation"),
            Self::TaskDescription => write!(f, "task_description"),
            Self::TaskObservation => write!(f, "task_observation"),
            Self::PullRequest => write!(f, "pull_request"),
        }
    }
}

/// Kind of repository topology reference discovered during migration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepositoryReferenceKind {
    /// A canonical Git remote reference.
    Remote,
    /// A local checkout/path reference.
    LocalPath,
}

impl std::fmt::Display for RepositoryReferenceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Remote => write!(f, "remote"),
            Self::LocalPath => write!(f, "local_path"),
        }
    }
}

/// Recommendation for a repository topology migration candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepositoryMigrationDisposition {
    /// Candidate appears worth human review.
    Review,
    /// Candidate should be held aside until ambiguity is resolved.
    Quarantine,
    /// Candidate is probably too low-value or non-canonical.
    Skip,
}

impl std::fmt::Display for RepositoryMigrationDisposition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Review => write!(f, "review"),
            Self::Quarantine => write!(f, "quarantine"),
            Self::Skip => write!(f, "skip"),
        }
    }
}

/// Evidence that produced a repository topology migration candidate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMigrationEvidence {
    /// Source layer.
    pub source_kind: RepositoryMigrationSourceKind,
    /// Source record identifier.
    pub source_id: String,
    /// Human-readable source label.
    pub source_label: String,
    /// Project scope of the source, when known.
    pub project_name: Option<String>,
    /// Text excerpt containing the reference.
    pub excerpt: String,
    /// Source creation time.
    #[serde(with = "time::serde::rfc3339")]
    pub source_created_at: OffsetDateTime,
    /// Source update time.
    #[serde(with = "time::serde::rfc3339")]
    pub source_updated_at: OffsetDateTime,
}

/// Candidate repository topology record inferred from existing Engram data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMigrationCandidate {
    /// Discovered reference kind.
    pub reference_kind: RepositoryReferenceKind,
    /// Proposed repository name when known or inferred.
    pub repository_name: Option<String>,
    /// Original remote URL/reference when present.
    pub remote_url: Option<String>,
    /// Normalized remote key such as `github.com/org/repo`.
    pub normalized_remote: Option<String>,
    /// Local path reference when present.
    pub local_path: Option<String>,
    /// Project linked by the source context, when known.
    pub project_name: Option<String>,
    /// Component path inferred from known checkouts, when possible.
    pub component_path: Option<String>,
    /// Suggested project-repository role.
    pub role: ProjectRepositoryRole,
    /// Heuristic confidence from 0.0 to 1.0.
    pub confidence: f32,
    /// Recommended handling.
    pub disposition: RepositoryMigrationDisposition,
    /// Reasons for the recommendation.
    pub reasons: Vec<String>,
    /// Source evidence records grouped into this candidate.
    pub evidence: Vec<RepositoryMigrationEvidence>,
}

/// Non-destructive repository topology migration inventory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMigrationInventory {
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
    /// Candidate counts by reference kind.
    pub by_reference_kind: BTreeMap<String, usize>,
    /// Candidate counts by disposition.
    pub by_disposition: BTreeMap<String, usize>,
    /// Candidate counts by project name.
    pub by_project: BTreeMap<String, usize>,
    /// Candidate counts by confidence bucket.
    pub by_confidence: BTreeMap<String, usize>,
    /// Warnings about the dry run.
    pub warnings: Vec<String>,
    /// Candidate records.
    pub candidates: Vec<RepositoryMigrationCandidate>,
}

/// Result of writing a repository topology migration review batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMigrationReviewExport {
    /// Review batch root path.
    pub root: String,
    /// Files created or updated, relative to root.
    pub files_written: Vec<String>,
    /// Existing files skipped because they were not generated by Engram.
    pub files_skipped: Vec<String>,
    /// Inventory used for this review batch.
    pub inventory: RepositoryMigrationInventory,
}

impl RepositoryMigrationReviewExport {
    /// Number of files written by this export.
    #[must_use]
    pub fn file_count(&self) -> usize {
        self.files_written.len()
    }
}

/// Options for applying a reviewed repository topology migration batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMigrationReviewApplyOptions {
    /// When true, parse and report the batch without writing topology records.
    pub dry_run: bool,
    /// Writer/importer provenance for the optional knowledge commit.
    pub writer: Option<WriterProvenance>,
    /// Create a knowledge commit for written topology records.
    pub create_commit: bool,
}

/// Planned or written repository topology record from a migration review batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMigrationAppliedRecord {
    /// Human-readable candidate title.
    pub title: String,
    /// Source review file.
    pub review_file: String,
    /// Repository record that would be written, or was written.
    pub repository: GitRepository,
    /// Whether the repository already existed before apply.
    pub repository_existing: bool,
    /// Checkout record that would be written, or was written.
    pub checkout: Option<LocalCheckout>,
    /// Whether the checkout already existed before apply.
    pub checkout_existing: bool,
    /// Component record that would be written, or was written.
    pub component: Option<MonorepoComponent>,
    /// Whether the component already existed before apply.
    pub component_existing: bool,
    /// Project link that would be written, or was written.
    pub project_link: Option<ProjectRepositoryLink>,
    /// Whether the project link already existed before apply.
    pub project_link_existing: bool,
}

impl RepositoryMigrationAppliedRecord {
    /// Whether every included topology record already existed.
    #[must_use]
    pub fn all_existing(&self) -> bool {
        self.repository_existing
            && self
                .checkout
                .as_ref()
                .map_or(true, |_| self.checkout_existing)
            && self
                .component
                .as_ref()
                .map_or(true, |_| self.component_existing)
            && self
                .project_link
                .as_ref()
                .map_or(true, |_| self.project_link_existing)
    }
}

/// Result of applying, or dry-running, a repository topology review batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMigrationReviewApply {
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
    /// Generated candidate files present under candidates/ but not linked from index.md.
    pub files_not_in_index: Vec<String>,
    /// Candidate files linked from index.md but missing on disk.
    pub indexed_files_missing: Vec<String>,
    /// Accepted candidates using generated topology.
    pub accepted_count: usize,
    /// Accepted candidates using edited topology fields.
    pub accepted_with_edits_count: usize,
    /// Accepted candidate review files.
    pub accepted_files: Vec<String>,
    /// Candidates explicitly quarantined by review.
    pub quarantined_count: usize,
    /// Quarantined candidate review files.
    pub quarantined_files: Vec<String>,
    /// Candidates explicitly rejected by review.
    pub rejected_count: usize,
    /// Rejected candidate review files.
    pub rejected_files: Vec<String>,
    /// Planned records that already existed before apply.
    pub existing_record_count: usize,
    /// Topology records that would be written, or were written in non-dry-run mode.
    pub planned_records: Vec<RepositoryMigrationAppliedRecord>,
    /// Topology records written in non-dry-run mode.
    pub written_records: Vec<RepositoryMigrationAppliedRecord>,
    /// Knowledge commit created in non-dry-run mode.
    pub commit: Option<KnowledgeCommit>,
    /// Non-fatal warnings surfaced during parsing/apply.
    pub warnings: Vec<String>,
}

impl RepositoryMigrationReviewApply {
    /// Number of planned accepted topology records.
    #[must_use]
    pub fn planned_count(&self) -> usize {
        self.planned_records.len()
    }

    /// Number of written topology records.
    #[must_use]
    pub fn written_count(&self) -> usize {
        self.written_records.len()
    }
}

/// Parsed status for a generated repository topology migration review batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMigrationReviewStatus {
    /// Review batch root path.
    pub root: String,
    /// Candidate review files scanned from the generated index.
    pub files_scanned: usize,
    /// Generated candidate files skipped with a reason.
    pub files_skipped: Vec<String>,
    /// Candidate files that had no selected decision checkbox.
    pub files_with_no_decision: Vec<String>,
    /// Candidate files that selected multiple conflicting decisions.
    pub files_with_conflicts: Vec<String>,
    /// Generated candidate files present under candidates/ but not linked from index.md.
    pub files_not_in_index: Vec<String>,
    /// Candidate files linked from index.md but missing on disk.
    pub indexed_files_missing: Vec<String>,
    /// Accepted candidates using generated topology.
    pub accepted_count: usize,
    /// Accepted candidates using edited topology fields.
    pub accepted_with_edits_count: usize,
    /// Accepted candidate review files.
    pub accepted_files: Vec<String>,
    /// Candidates explicitly quarantined by review.
    pub quarantined_count: usize,
    /// Quarantined candidate review files.
    pub quarantined_files: Vec<String>,
    /// Candidates explicitly rejected by review.
    pub rejected_count: usize,
    /// Rejected candidate review files.
    pub rejected_files: Vec<String>,
    /// Planned records that already existed before apply.
    pub existing_record_count: usize,
    /// Accepted topology records that would be written by apply.
    pub planned_record_count: usize,
    /// Whether the batch has no pending, conflicting, orphaned, or missing files.
    pub ready_to_apply: bool,
    /// Non-fatal warnings surfaced during parsing.
    pub warnings: Vec<String>,
}

impl From<RepositoryMigrationReviewApply> for RepositoryMigrationReviewStatus {
    fn from(apply: RepositoryMigrationReviewApply) -> Self {
        let planned_record_count = apply.planned_count();
        let ready_to_apply = apply.files_with_no_decision.is_empty()
            && apply.files_with_conflicts.is_empty()
            && apply.files_not_in_index.is_empty()
            && apply.indexed_files_missing.is_empty();
        Self {
            root: apply.root,
            files_scanned: apply.files_scanned,
            files_skipped: apply.files_skipped,
            files_with_no_decision: apply.files_with_no_decision,
            files_with_conflicts: apply.files_with_conflicts,
            files_not_in_index: apply.files_not_in_index,
            indexed_files_missing: apply.indexed_files_missing,
            accepted_count: apply.accepted_count,
            accepted_with_edits_count: apply.accepted_with_edits_count,
            accepted_files: apply.accepted_files,
            quarantined_count: apply.quarantined_count,
            quarantined_files: apply.quarantined_files,
            rejected_count: apply.rejected_count,
            rejected_files: apply.rejected_files,
            existing_record_count: apply.existing_record_count,
            planned_record_count,
            ready_to_apply,
            warnings: apply.warnings,
        }
    }
}

/// Repository topology service.
#[derive(Clone)]
pub struct RepositoryService {
    repo: RepositoryRepo,
    memory_repo: MemoryRepo,
    entity_repo: EntityRepo,
    session_repo: SessionRepo,
    work_repo: WorkRepo,
}

impl RepositoryService {
    /// Create a new repository topology service.
    pub fn new(db: Db) -> Self {
        Self {
            repo: RepositoryRepo::new(db.clone()),
            memory_repo: MemoryRepo::new(db.clone()),
            entity_repo: EntityRepo::new(db.clone()),
            session_repo: SessionRepo::new(db.clone()),
            work_repo: WorkRepo::new(db),
        }
    }

    /// Initialize repository topology schema.
    pub async fn init_schema(&self) -> IndexResult<()> {
        self.repo.init_schema().await?;
        Ok(())
    }

    /// Build a non-destructive inventory of repository topology references in legacy data.
    pub async fn migration_inventory(
        &self,
        options: RepositoryMigrationOptions,
    ) -> IndexResult<RepositoryMigrationInventory> {
        let options = normalize_repository_migration_options(options);
        let now = OffsetDateTime::now_utc();
        let mut warnings = vec![
            "Dry run only: no repository topology records were written.".to_string(),
            "Candidates should be reviewed before registering repositories, checkouts, components, or project links.".to_string(),
        ];
        let mut sources_scanned = 0;
        let mut candidates = Vec::new();
        let known_checkouts = self.repo.list_checkouts().await?;

        let project_filter = options.project_filter.as_deref();
        let project_for_filter = if let Some(project_name) = project_filter {
            let project = self.resolve_project_filter(project_name).await?;
            if project.is_none() {
                warnings.push(format!(
                    "Project filter '{}' did not match a Layer 7 project; work scans returned no project-scoped records.",
                    project_name
                ));
            }
            project
        } else {
            None
        };

        if options.include_work_records
            && (project_filter.is_none() || project_for_filter.is_some())
        {
            sources_scanned += self
                .inventory_repository_work_records(
                    project_for_filter.as_ref(),
                    &known_checkouts,
                    &mut candidates,
                )
                .await?;
        }
        if options.include_entity_observations {
            sources_scanned += self
                .inventory_repository_entity_records(
                    project_for_filter.as_ref(),
                    project_filter,
                    &known_checkouts,
                    &mut candidates,
                )
                .await?;
        }
        if options.include_session_history {
            sources_scanned += self
                .inventory_repository_session_records(
                    project_filter,
                    &known_checkouts,
                    &mut candidates,
                )
                .await?;
        }

        let mut candidates = aggregate_repository_candidates(candidates);
        candidates.sort_by(|left, right| {
            repository_disposition_rank(left.disposition)
                .cmp(&repository_disposition_rank(right.disposition))
                .then_with(|| right.confidence.total_cmp(&left.confidence))
                .then_with(|| {
                    left.repository_name
                        .as_deref()
                        .unwrap_or("")
                        .cmp(right.repository_name.as_deref().unwrap_or(""))
                })
                .then_with(|| {
                    left.normalized_remote
                        .as_deref()
                        .unwrap_or("")
                        .cmp(right.normalized_remote.as_deref().unwrap_or(""))
                })
                .then_with(|| {
                    left.local_path
                        .as_deref()
                        .unwrap_or("")
                        .cmp(right.local_path.as_deref().unwrap_or(""))
                })
        });

        let total_candidates = candidates.len();
        let truncated = options.limit.is_some_and(|limit| candidates.len() > limit);
        if let Some(limit) = options.limit {
            candidates.truncate(limit);
        }
        let returned_candidates = candidates.len();

        Ok(RepositoryMigrationInventory {
            generated_at: now,
            project_filter: options.project_filter,
            sources_scanned,
            total_candidates,
            returned_candidates,
            truncated,
            by_reference_kind: count_repository_references(&candidates),
            by_disposition: count_repository_dispositions(&candidates),
            by_project: count_repository_projects(&candidates),
            by_confidence: count_repository_confidence(&candidates),
            warnings,
            candidates,
        })
    }

    /// Export a non-destructive repository topology migration review batch.
    pub async fn export_migration_review(
        &self,
        root: impl AsRef<Path>,
        options: RepositoryMigrationOptions,
    ) -> IndexResult<RepositoryMigrationReviewExport> {
        let inventory = self.migration_inventory(options).await?;
        write_repository_migration_review(root.as_ref(), inventory)
    }

    /// Parse a generated repository migration review batch and report readiness without writes.
    pub async fn migration_review_status(
        &self,
        root: impl AsRef<Path>,
    ) -> IndexResult<RepositoryMigrationReviewStatus> {
        let apply = self
            .apply_migration_review(
                root,
                RepositoryMigrationReviewApplyOptions {
                    dry_run: true,
                    writer: None,
                    create_commit: false,
                },
            )
            .await?;
        Ok(apply.into())
    }

    /// Apply a reviewed repository topology migration batch.
    pub async fn apply_migration_review(
        &self,
        root: impl AsRef<Path>,
        options: RepositoryMigrationReviewApplyOptions,
    ) -> IndexResult<RepositoryMigrationReviewApply> {
        let root = root.as_ref();
        let mut report = RepositoryMigrationReviewApply {
            root: root.display().to_string(),
            dry_run: options.dry_run,
            files_scanned: 0,
            files_skipped: Vec::new(),
            files_with_no_decision: Vec::new(),
            files_with_conflicts: Vec::new(),
            files_not_in_index: Vec::new(),
            indexed_files_missing: Vec::new(),
            accepted_count: 0,
            accepted_with_edits_count: 0,
            accepted_files: Vec::new(),
            quarantined_count: 0,
            quarantined_files: Vec::new(),
            rejected_count: 0,
            rejected_files: Vec::new(),
            existing_record_count: 0,
            planned_records: Vec::new(),
            written_records: Vec::new(),
            commit: None,
            warnings: Vec::new(),
        };

        for path in collect_repository_candidate_review_files(root, &mut report)? {
            report.files_scanned += 1;
            let relative_path = relative_repository_review_path(root, &path);
            let contents = fs::read_to_string(&path)?;
            let Some(parsed) =
                parse_repository_review_candidate_page(&contents, &relative_path, &mut report)?
            else {
                continue;
            };

            self.apply_parsed_repository_review(parsed, &options, &mut report, &relative_path)
                .await?;
        }

        if !options.dry_run && options.create_commit && !report.written_records.is_empty() {
            let writer = options.writer.as_ref().ok_or_else(|| {
                IndexError::Parse(
                    "writer provenance is required to create repository migration commit"
                        .to_string(),
                )
            })?;
            let commit = build_repository_migration_commit(writer, &report.written_records);
            self.memory_repo.save_knowledge_commit(&commit).await?;
            report.commit = Some(commit);
        }

        report.files_skipped.sort();
        report.files_with_no_decision.sort();
        report.files_with_conflicts.sort();
        report.files_not_in_index.sort();
        report.indexed_files_missing.sort();
        report.accepted_files.sort();
        report.quarantined_files.sort();
        report.rejected_files.sort();
        report.warnings.sort();
        Ok(report)
    }

    /// Register or update a canonical repository.
    pub async fn register_repository(
        &self,
        name: &str,
        remote_url: Option<&str>,
        default_branch: Option<&str>,
        description: Option<&str>,
    ) -> IndexResult<GitRepository> {
        validate_non_empty(name, "repository name")?;

        let existing = if let Some(remote_url) = remote_url {
            self.repo.get_repository_by_remote_url(remote_url).await?
        } else {
            self.repo.get_repository_by_name(name).await?
        }
        .or(self.repo.get_repository_by_name(name).await?);

        let mut repository = existing.unwrap_or_else(|| GitRepository::new(name));
        repository.name = name.to_string();
        if let Some(remote_url) = remote_url.filter(|value| !value.trim().is_empty()) {
            repository = repository.with_remote_url(remote_url.to_string());
        }
        if let Some(default_branch) = default_branch.filter(|value| !value.trim().is_empty()) {
            repository.default_branch = Some(default_branch.to_string());
        }
        if let Some(description) = description.filter(|value| !value.trim().is_empty()) {
            repository.description = Some(description.to_string());
        }
        repository.touch();

        self.repo.save_repository(&repository).await?;
        Ok(repository)
    }

    /// Get a repository by ID.
    pub async fn get_repository(&self, id: &Id) -> IndexResult<Option<GitRepository>> {
        Ok(self.repo.get_repository(id).await?)
    }

    /// Resolve a repository by ID or name.
    pub async fn resolve_repository(
        &self,
        repository_id: Option<&Id>,
        repository_name: Option<&str>,
    ) -> IndexResult<GitRepository> {
        if let Some(id) = repository_id {
            return self
                .repo
                .get_repository(id)
                .await?
                .ok_or_else(|| IndexError::NotFound(format!("repository not found: {id}")));
        }

        let name = repository_name
            .filter(|name| !name.trim().is_empty())
            .ok_or_else(|| IndexError::Parse("repository_id or repository_name required".into()))?;
        self.repo
            .get_repository_by_name(name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("repository not found: {name}")))
    }

    /// List repositories.
    pub async fn list_repositories(&self, limit: Option<usize>) -> IndexResult<Vec<GitRepository>> {
        Ok(self.repo.list_repositories(limit).await?)
    }

    /// Detect a Git repository from a working directory and register the checkout.
    pub async fn detect_repository(&self, cwd: &Path) -> IndexResult<RepositoryDetection> {
        let root = run_git_required(cwd, &["rev-parse", "--show-toplevel"])?;
        let root_path = PathBuf::from(root.trim());
        let root_path = root_path.canonicalize().unwrap_or(root_path);
        let root = root_path.display().to_string();

        let remote_url = run_git_optional(&root_path, &["remote", "get-url", "origin"])?;
        let default_branch = detect_default_branch(&root_path)?;
        let current_branch = run_git_optional(&root_path, &["rev-parse", "--abbrev-ref", "HEAD"])?;
        let head_sha = run_git_optional(&root_path, &["rev-parse", "HEAD"])?;
        let is_dirty = detect_dirty(&root_path)?;

        let name = remote_url
            .as_deref()
            .and_then(repository_name_from_remote)
            .or_else(|| {
                root_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(str::to_string)
            })
            .ok_or_else(|| IndexError::Parse("could not derive repository name".into()))?;

        let repository = self
            .register_repository(
                &name,
                remote_url.as_deref(),
                default_branch.as_deref(),
                None,
            )
            .await?;

        let mut checkout = self
            .repo
            .get_checkout_by_path(&root)
            .await?
            .unwrap_or_else(|| LocalCheckout::new(&root).with_repository(repository.id));
        checkout.repository_id = Some(repository.id);
        checkout.update_detected_state(current_branch, head_sha, is_dirty);
        self.repo.save_checkout(&checkout).await?;

        let context = self.resolve_cwd(Path::new(&root)).await?.ok_or_else(|| {
            IndexError::InvalidState("detected checkout was not resolvable".into())
        })?;

        info!(
            "Detected repository '{}' at {}",
            context.repository.name, root
        );

        Ok(RepositoryDetection {
            context,
            detected_root: root,
            warnings: Vec::new(),
        })
    }

    /// Register or update a monorepo component.
    pub async fn register_component(
        &self,
        repository_id: Option<&Id>,
        repository_name: Option<&str>,
        name: &str,
        path: &str,
        kind: Option<&str>,
        description: Option<&str>,
    ) -> IndexResult<MonorepoComponent> {
        validate_non_empty(name, "component name")?;
        validate_non_empty(path, "component path")?;

        let repository = self
            .resolve_repository(repository_id, repository_name)
            .await?;
        let normalized_path = normalize_component_path(path);

        let mut component = self
            .repo
            .get_component_by_path(&repository.id, &normalized_path)
            .await?
            .unwrap_or_else(|| MonorepoComponent::new(repository.id, name, &normalized_path));
        component.name = name.to_string();
        component.path = normalized_path;
        component.kind = kind
            .filter(|value| !value.trim().is_empty())
            .map(str::to_string);
        component.description = description
            .filter(|value| !value.trim().is_empty())
            .map(str::to_string);
        component.updated_at = OffsetDateTime::now_utc();

        self.repo.save_component(&component).await?;
        Ok(component)
    }

    /// Link a project name to a repository.
    pub async fn link_project(
        &self,
        project_name: &str,
        repository_id: Option<&Id>,
        repository_name: Option<&str>,
        role: ProjectRepositoryRole,
        component_path: Option<&str>,
    ) -> IndexResult<ProjectRepositoryLink> {
        validate_non_empty(project_name, "project name")?;
        let repository = self
            .resolve_repository(repository_id, repository_name)
            .await?;
        let project = self.work_repo.get_project_by_name(project_name).await?;
        let component_scope = component_path
            .filter(|value| !value.trim().is_empty())
            .map(normalize_component_path);
        let component_id = if let Some(path) = &component_scope {
            self.repo
                .get_component_by_path(&repository.id, path)
                .await?
                .map(|component| component.id)
        } else {
            None
        };

        let mut link = self
            .repo
            .list_project_links(&repository.id)
            .await?
            .into_iter()
            .find(|link| {
                link.project_name.eq_ignore_ascii_case(project_name)
                    && link.component_path == component_scope
            })
            .unwrap_or_else(|| ProjectRepositoryLink::new(project_name, repository.id, role));
        link.role = role;
        link.project_id = project.map(|project| project.id);
        link.component_id = component_id;
        link.component_path = component_scope;
        link.updated_at = OffsetDateTime::now_utc();

        self.repo.save_project_link(&link).await?;
        Ok(link)
    }

    /// Resolve repository context for a cwd from registered checkouts.
    pub async fn resolve_cwd(&self, cwd: &Path) -> IndexResult<Option<RepositoryContext>> {
        let cwd_path = cwd.canonicalize().unwrap_or_else(|_| cwd.to_path_buf());
        let checkouts = self.repo.list_checkouts().await?;
        let checkout = checkouts
            .into_iter()
            .filter(|checkout| {
                let checkout_path = canonical_or_original(Path::new(&checkout.local_path));
                path_starts_with(&cwd_path, &checkout_path)
            })
            .max_by_key(|checkout| {
                canonical_or_original(Path::new(&checkout.local_path))
                    .components()
                    .count()
            });

        let Some(mut checkout) = checkout else {
            return Ok(None);
        };
        if refresh_checkout_git_state(&mut checkout)? {
            self.repo.save_checkout(&checkout).await?;
        }
        let Some(repository_id) = checkout.repository_id else {
            return Ok(None);
        };

        let Some(repository) = self.repo.get_repository(&repository_id).await? else {
            return Ok(None);
        };

        let components = self.repo.list_components(&repository.id).await?;
        let checkout_path = canonical_or_original(Path::new(&checkout.local_path));
        let matching_components = matching_components(&cwd_path, &checkout_path, components);
        let linked_projects = self.repo.list_project_links(&repository.id).await?;

        Ok(Some(RepositoryContext {
            repository,
            checkout: Some(checkout),
            matching_components,
            linked_projects,
        }))
    }

    async fn apply_parsed_repository_review(
        &self,
        parsed: ParsedRepositoryReviewCandidate,
        options: &RepositoryMigrationReviewApplyOptions,
        report: &mut RepositoryMigrationReviewApply,
        relative_path: &str,
    ) -> IndexResult<()> {
        match parsed.decision {
            RepositoryReviewDecision::Accept | RepositoryReviewDecision::AcceptWithEdits => {
                if parsed.decision == RepositoryReviewDecision::Accept {
                    report.accepted_count += 1;
                } else {
                    report.accepted_with_edits_count += 1;
                }
                report.accepted_files.push(relative_path.to_string());

                let candidate = apply_repository_edits(parsed);
                let record = self
                    .plan_repository_apply(&candidate, relative_path, &report.planned_records)
                    .await?;
                if record.all_existing() {
                    report.existing_record_count += 1;
                }
                if !options.dry_run {
                    self.write_repository_apply_record(&record).await?;
                    report.written_records.push(record.clone());
                }
                report.planned_records.push(record);
            }
            RepositoryReviewDecision::Quarantine => {
                report.quarantined_count += 1;
                report.quarantined_files.push(relative_path.to_string());
            }
            RepositoryReviewDecision::Reject => {
                report.rejected_count += 1;
                report.rejected_files.push(relative_path.to_string());
            }
        }
        Ok(())
    }

    async fn plan_repository_apply(
        &self,
        candidate: &RepositoryMigrationCandidate,
        relative_path: &str,
        planned_records: &[RepositoryMigrationAppliedRecord],
    ) -> IndexResult<RepositoryMigrationAppliedRecord> {
        let repository = self
            .resolve_repository_for_apply(candidate, planned_records)
            .await?;
        let repository_existing = repository.1;
        let repository = repository.0;

        let (checkout, checkout_existing) = if let Some(path) = candidate.local_path.as_deref() {
            let path = canonical_path_string(path);
            let planned_checkout = planned_records.iter().find_map(|record| {
                record
                    .checkout
                    .as_ref()
                    .filter(|checkout| checkout.local_path == path)
                    .cloned()
            });
            let checkout = if planned_checkout.is_some() {
                planned_checkout
            } else {
                self.repo
                    .get_checkout_by_path(&path)
                    .await?
                    .map(|mut checkout| {
                        checkout.repository_id = Some(repository.id);
                        checkout
                    })
            };
            match checkout {
                Some(checkout) => (Some(checkout), true),
                None => (
                    Some(LocalCheckout::new(path).with_repository(repository.id)),
                    false,
                ),
            }
        } else {
            (None, false)
        };

        let (component, component_existing) = if let Some(component_path) = candidate
            .component_path
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            let component_path = normalize_component_path(component_path);
            let planned_component = planned_records.iter().find_map(|record| {
                record.component.as_ref().filter(|component| {
                    component.repository_id == repository.id && component.path == component_path
                })
            });
            match planned_component.cloned().or(self
                .repo
                .get_component_by_path(&repository.id, &component_path)
                .await?)
            {
                Some(component) => (Some(component), true),
                None => {
                    let name = component_name_from_path(&component_path);
                    (
                        Some(MonorepoComponent::new(repository.id, name, component_path)),
                        false,
                    )
                }
            }
        } else {
            (None, false)
        };

        let (project_link, project_link_existing) = if let Some(project_name) = candidate
            .project_name
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            let component_path = component.as_ref().map(|component| component.path.clone());
            let planned_link = planned_records.iter().find_map(|record| {
                record.project_link.as_ref().filter(|link| {
                    link.repository_id == repository.id
                        && link.project_name.eq_ignore_ascii_case(project_name)
                        && link.component_path == component_path
                })
            });
            let existing = planned_link.cloned().or(self
                .repo
                .list_project_links(&repository.id)
                .await?
                .into_iter()
                .find(|link| {
                    link.project_name.eq_ignore_ascii_case(project_name)
                        && link.component_path == component_path
                }));
            let project = self.work_repo.get_project_by_name(project_name).await?;
            match existing {
                Some(mut link) => {
                    link.role = candidate.role;
                    link.project_id = project.map(|project| project.id);
                    link.component_id = component.as_ref().map(|component| component.id);
                    link.component_path = component_path;
                    link.updated_at = OffsetDateTime::now_utc();
                    (Some(link), true)
                }
                None => {
                    let mut link =
                        ProjectRepositoryLink::new(project_name, repository.id, candidate.role);
                    if let Some(project) = project {
                        link = link.with_project_id(project.id);
                    }
                    if let Some(component) = &component {
                        link = link.with_component(Some(component.id), component.path.clone());
                    }
                    (Some(link), false)
                }
            }
        } else {
            (None, false)
        };

        Ok(RepositoryMigrationAppliedRecord {
            title: repository_candidate_title(candidate),
            review_file: relative_path.to_string(),
            repository,
            repository_existing,
            checkout,
            checkout_existing,
            component,
            component_existing,
            project_link,
            project_link_existing,
        })
    }

    async fn resolve_repository_for_apply(
        &self,
        candidate: &RepositoryMigrationCandidate,
        planned_records: &[RepositoryMigrationAppliedRecord],
    ) -> IndexResult<(GitRepository, bool)> {
        let remote_url = canonical_remote_url(candidate);
        let remote_key = remote_url
            .as_deref()
            .and_then(normalize_remote_reference)
            .or_else(|| candidate.normalized_remote.clone());
        let repository_name = candidate
            .repository_name
            .clone()
            .or_else(|| {
                remote_key
                    .as_deref()
                    .and_then(repository_name_from_normalized_remote)
            })
            .or_else(|| {
                candidate
                    .local_path
                    .as_deref()
                    .and_then(repository_name_from_path)
            })
            .unwrap_or_else(|| "migrated-repository".to_string());

        if let Some(remote_key) = &remote_key {
            if let Some(repository) = planned_records
                .iter()
                .map(|record| &record.repository)
                .find(|repository| {
                    repository
                        .remote_url
                        .as_deref()
                        .and_then(normalize_remote_reference)
                        .as_ref()
                        == Some(remote_key)
                })
            {
                return Ok((repository.clone(), true));
            }
            if let Some(mut repository) =
                self.repo
                    .list_repositories(None)
                    .await?
                    .into_iter()
                    .find(|repository| {
                        repository
                            .remote_url
                            .as_deref()
                            .and_then(normalize_remote_reference)
                            .as_ref()
                            == Some(remote_key)
                    })
            {
                if repository.remote_url.is_none() {
                    repository.remote_url = remote_url;
                }
                repository.name = repository_name;
                repository.touch();
                return Ok((repository, true));
            }
        }

        if let Some(repository) = planned_records
            .iter()
            .map(|record| &record.repository)
            .find(|repository| repository.name.eq_ignore_ascii_case(&repository_name))
        {
            return Ok((repository.clone(), true));
        }

        if let Some(repository) = self.repo.get_repository_by_name(&repository_name).await? {
            return Ok((repository, true));
        }

        let mut repository = GitRepository::new(repository_name);
        if let Some(remote_url) = remote_url {
            repository = repository.with_remote_url(remote_url);
        }
        Ok((repository, false))
    }

    async fn write_repository_apply_record(
        &self,
        record: &RepositoryMigrationAppliedRecord,
    ) -> IndexResult<()> {
        self.repo.save_repository(&record.repository).await?;
        if let Some(checkout) = &record.checkout {
            self.repo.save_checkout(checkout).await?;
        }
        if let Some(component) = &record.component {
            self.repo.save_component(component).await?;
        }
        if let Some(link) = &record.project_link {
            self.repo.save_project_link(link).await?;
        }
        Ok(())
    }

    async fn inventory_repository_work_records(
        &self,
        project_for_filter: Option<&Project>,
        known_checkouts: &[LocalCheckout],
        candidates: &mut Vec<RepositoryMigrationCandidate>,
    ) -> IndexResult<usize> {
        let projects = if let Some(project) = project_for_filter {
            vec![project.clone()]
        } else {
            self.work_repo.list_projects(None).await?
        };

        let mut scanned = 0;
        for project in projects {
            if let Some(description) = project.description.as_deref() {
                scanned += 1;
                append_repository_candidates(
                    description,
                    repository_source(
                        RepositoryMigrationSourceKind::ProjectDescription,
                        project.id.to_string(),
                        format!("project:{} description", project.name),
                        Some(project.name.clone()),
                        project.created_at,
                        project.updated_at,
                    ),
                    known_checkouts,
                    candidates,
                );
            }

            for observation in self.work_repo.get_project_observations(&project.id).await? {
                scanned += 1;
                append_repository_candidates(
                    &observation.content,
                    source_from_project_observation(&project, &observation),
                    known_checkouts,
                    candidates,
                );
            }

            for task in self.work_repo.list_tasks(&project.id, None).await? {
                if let Some(description) = task.description.as_deref() {
                    scanned += 1;
                    append_repository_candidates(
                        description,
                        repository_source(
                            RepositoryMigrationSourceKind::TaskDescription,
                            task.id.to_string(),
                            format!(
                                "task:{} description",
                                task.jira_key.as_deref().unwrap_or(&task.name)
                            ),
                            Some(project.name.clone()),
                            task.created_at,
                            task.updated_at,
                        ),
                        known_checkouts,
                        candidates,
                    );
                }

                for observation in self.work_repo.get_task_observations(&task.id).await? {
                    scanned += 1;
                    append_repository_candidates(
                        &observation.content,
                        source_from_task_observation(&project, &task, &observation),
                        known_checkouts,
                        candidates,
                    );
                }
            }

            for pr in self.work_repo.list_prs(Some(&project.id), None).await? {
                scanned += 1;
                append_repository_candidates(
                    &format!(
                        "{} {} {}",
                        pr.url,
                        pr.repo,
                        pr.title.as_deref().unwrap_or("")
                    ),
                    source_from_pr(&project, &pr),
                    known_checkouts,
                    candidates,
                );
            }
        }

        Ok(scanned)
    }

    async fn resolve_project_filter(&self, project_name: &str) -> IndexResult<Option<Project>> {
        if let Some(project) = self.work_repo.get_project_by_name(project_name).await? {
            return Ok(Some(project));
        }

        Ok(self
            .work_repo
            .list_projects(None)
            .await?
            .into_iter()
            .find(|project| project.name.eq_ignore_ascii_case(project_name)))
    }

    async fn inventory_repository_entity_records(
        &self,
        project_for_filter: Option<&Project>,
        project_filter: Option<&str>,
        known_checkouts: &[LocalCheckout],
        candidates: &mut Vec<RepositoryMigrationCandidate>,
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
            if let Some(description) = entity.description.as_deref() {
                scanned += 1;
                append_repository_candidates(
                    description,
                    source_from_entity_description(&entity),
                    known_checkouts,
                    candidates,
                );
            }

            for observation in self.entity_repo.get_observations(&entity.id).await? {
                scanned += 1;
                append_repository_candidates(
                    &observation.content,
                    repository_source(
                        RepositoryMigrationSourceKind::EntityObservation,
                        observation.id.to_string(),
                        format!("entity:{} observation", entity.name),
                        None,
                        observation.created_at,
                        observation.updated_at,
                    ),
                    known_checkouts,
                    candidates,
                );
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

    async fn inventory_repository_session_records(
        &self,
        project_filter: Option<&str>,
        known_checkouts: &[LocalCheckout],
        candidates: &mut Vec<RepositoryMigrationCandidate>,
    ) -> IndexResult<usize> {
        let sessions = self
            .session_repo
            .list_sessions(None, None, project_filter, None)
            .await?;

        let mut scanned = 0;
        for session in sessions {
            if let Some(goal) = session.goal.as_deref() {
                scanned += 1;
                append_repository_candidates(
                    goal,
                    source_from_session_record(&session, "goal", session.started_at),
                    known_checkouts,
                    candidates,
                );
            }
            if let Some(summary) = session.summary.as_deref() {
                scanned += 1;
                append_repository_candidates(
                    summary,
                    source_from_session_record(
                        &session,
                        "summary",
                        session.ended_at.unwrap_or(session.started_at),
                    ),
                    known_checkouts,
                    candidates,
                );
            }
            for (index, decision) in session.key_decisions.iter().enumerate() {
                scanned += 1;
                append_repository_candidates(
                    decision,
                    source_from_session_record(
                        &session,
                        &format!("decision:{index}"),
                        session.ended_at.unwrap_or(session.started_at),
                    ),
                    known_checkouts,
                    candidates,
                );
            }

            for event in self.session_repo.get_events(&session.id).await? {
                scanned += 1;
                append_repository_candidates(
                    &format!(
                        "{} {}",
                        event.content,
                        event.context.as_deref().unwrap_or("")
                    ),
                    source_from_session_event(&session, &event),
                    known_checkouts,
                    candidates,
                );
            }
        }

        Ok(scanned)
    }
}

fn build_repository_migration_commit(
    writer: &WriterProvenance,
    written_records: &[RepositoryMigrationAppliedRecord],
) -> KnowledgeCommit {
    let mut commit = KnowledgeCommit::new(
        writer.clone(),
        format!(
            "Apply reviewed repository topology migration batch ({} records)",
            written_records.len()
        ),
    );
    for record in written_records {
        commit = commit.with_change(repository_migration_change(record));
    }
    commit
}

fn repository_migration_change(record: &RepositoryMigrationAppliedRecord) -> MemoryChange {
    MemoryChange::new(
        MemoryChangeType::Linked,
        format!("Repository topology: {}", record.repository.name),
        repository_migration_change_summary(record),
    )
}

fn repository_migration_change_summary(record: &RepositoryMigrationAppliedRecord) -> String {
    let mut parts = vec![
        format!("review_file={}", record.review_file),
        format!("repository_id={}", record.repository.id),
        format!("repository_existing={}", record.repository_existing),
    ];
    if let Some(remote_url) = &record.repository.remote_url {
        parts.push(format!("remote_url={remote_url}"));
    }
    if let Some(checkout) = &record.checkout {
        parts.push(format!("checkout_id={}", checkout.id));
        parts.push(format!("checkout_path={}", checkout.local_path));
        parts.push(format!("checkout_existing={}", record.checkout_existing));
    }
    if let Some(component) = &record.component {
        parts.push(format!("component_id={}", component.id));
        parts.push(format!("component_path={}", component.path));
        parts.push(format!("component_existing={}", record.component_existing));
    }
    if let Some(link) = &record.project_link {
        parts.push(format!("project_link_id={}", link.id));
        parts.push(format!("project={}", link.project_name));
        parts.push(format!(
            "project_link_existing={}",
            record.project_link_existing
        ));
    }
    parts.join("; ")
}

fn normalize_repository_migration_options(
    mut options: RepositoryMigrationOptions,
) -> RepositoryMigrationOptions {
    if !options.include_entity_observations
        && !options.include_session_history
        && !options.include_work_records
    {
        options.include_entity_observations = true;
        options.include_session_history = true;
        options.include_work_records = true;
    }
    options
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RepositoryReviewDecision {
    Accept,
    AcceptWithEdits,
    Quarantine,
    Reject,
}

struct ParsedRepositoryReviewCandidate {
    decision: RepositoryReviewDecision,
    candidate: RepositoryMigrationCandidate,
    edited_repository_name: Option<String>,
    edited_remote_url: Option<String>,
    edited_normalized_remote: Option<String>,
    edited_local_path: Option<String>,
    edited_project_name: Option<String>,
    edited_component_path: Option<String>,
    edited_role: Option<ProjectRepositoryRole>,
}

fn apply_repository_edits(parsed: ParsedRepositoryReviewCandidate) -> RepositoryMigrationCandidate {
    let mut candidate = parsed.candidate;
    if parsed.decision == RepositoryReviewDecision::AcceptWithEdits {
        if let Some(value) = parsed.edited_repository_name {
            candidate.repository_name = Some(value);
        }
        if let Some(value) = parsed.edited_remote_url {
            candidate.remote_url = Some(value);
        }
        if let Some(value) = parsed.edited_normalized_remote {
            candidate.normalized_remote = Some(value);
        }
        if let Some(value) = parsed.edited_local_path {
            candidate.local_path = Some(canonical_path_string(&value));
        }
        if let Some(value) = parsed.edited_project_name {
            candidate.project_name = Some(value);
        }
        if let Some(value) = parsed.edited_component_path {
            candidate.component_path = Some(normalize_component_path(&value));
        }
        if let Some(value) = parsed.edited_role {
            candidate.role = value;
        }
    }
    candidate
}

fn collect_repository_candidate_review_files(
    root: &Path,
    report: &mut RepositoryMigrationReviewApply,
) -> IndexResult<Vec<PathBuf>> {
    let index_path = root.join("index.md");
    if !index_path.exists() {
        skip_repository_candidate_files_without_generated_index(
            root,
            report,
            "missing generated index.md",
        )?;
        return Ok(Vec::new());
    }

    let index_contents = fs::read_to_string(&index_path)?;
    if !index_contents.contains(REPOSITORY_REVIEW_GENERATED_MARKER) {
        skip_repository_candidate_files_without_generated_index(
            root,
            report,
            "index.md is not an Engram-generated review file",
        )?;
        return Ok(Vec::new());
    }

    let mut indexed_paths = indexed_repository_candidate_review_paths(&index_contents);
    indexed_paths.sort();
    indexed_paths.dedup();
    let indexed_markdown_paths = indexed_paths
        .iter()
        .map(|path| path_to_markdown(path))
        .collect::<HashSet<_>>();

    let candidates_dir = root.join("candidates");
    if candidates_dir.exists() {
        for entry in fs::read_dir(&candidates_dir)? {
            let path = entry?.path();
            if !path.extension().is_some_and(|extension| extension == "md") {
                continue;
            }
            let relative_path = relative_repository_review_path(root, &path);
            if !indexed_markdown_paths.contains(&relative_path) {
                report.files_not_in_index.push(relative_path.clone());
                report.files_skipped.push(relative_path.clone());
                report.warnings.push(format!(
                    "{relative_path}: skipped candidate file not listed in generated index.md"
                ));
            }
        }
    }

    let mut files = Vec::new();
    for relative_path in indexed_paths {
        let path = root.join(&relative_path);
        if path.exists() {
            files.push(path);
        } else {
            let relative_path = path_to_markdown(&relative_path);
            report.indexed_files_missing.push(relative_path.clone());
            report.files_skipped.push(relative_path.clone());
            report.warnings.push(format!(
                "{relative_path}: indexed candidate file is missing"
            ));
        }
    }
    files.sort();
    Ok(files)
}

fn skip_repository_candidate_files_without_generated_index(
    root: &Path,
    report: &mut RepositoryMigrationReviewApply,
    reason: &str,
) -> IndexResult<()> {
    let candidates_dir = root.join("candidates");
    if !candidates_dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(candidates_dir)? {
        let path = entry?.path();
        if !path.extension().is_some_and(|extension| extension == "md") {
            continue;
        }
        let relative_path = relative_repository_review_path(root, &path);
        report.files_not_in_index.push(relative_path.clone());
        report.files_skipped.push(relative_path.clone());
        report.warnings.push(format!(
            "{relative_path}: skipped candidate file because {reason}"
        ));
    }
    Ok(())
}

fn indexed_repository_candidate_review_paths(index_contents: &str) -> Vec<PathBuf> {
    index_contents
        .lines()
        .filter_map(markdown_link_target)
        .filter(|path| is_safe_repository_candidate_review_link(path))
        .map(PathBuf::from)
        .collect()
}

fn markdown_link_target(line: &str) -> Option<&str> {
    let link_start = line.find("](")?;
    let after_start = &line[link_start + 2..];
    let link_end = after_start.find(')')?;
    Some(&after_start[..link_end])
}

fn is_safe_repository_candidate_review_link(path: &str) -> bool {
    path.starts_with("candidates/")
        && path.ends_with(".md")
        && !path.starts_with('/')
        && !path.contains("..")
        && !path.contains('\\')
}

fn parse_repository_review_candidate_page(
    contents: &str,
    relative_path: &str,
    report: &mut RepositoryMigrationReviewApply,
) -> IndexResult<Option<ParsedRepositoryReviewCandidate>> {
    if !contents.contains(REPOSITORY_REVIEW_GENERATED_MARKER) {
        report.files_skipped.push(relative_path.to_string());
        report.warnings.push(format!(
            "{relative_path}: skipped non-generated review file"
        ));
        return Ok(None);
    }

    let decisions = selected_repository_review_decisions(contents);
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

    let candidate = parse_repository_machine_candidate(contents).map_err(|error| {
        IndexError::Parse(format!(
            "{relative_path}: invalid repository migration candidate record: {error}"
        ))
    })?;

    Ok(Some(ParsedRepositoryReviewCandidate {
        decision: decisions[0],
        candidate,
        edited_repository_name: bullet_value(contents, "Repository name"),
        edited_remote_url: bullet_value(contents, "Remote URL"),
        edited_normalized_remote: bullet_value(contents, "Normalized remote"),
        edited_local_path: bullet_value(contents, "Local path"),
        edited_project_name: bullet_value(contents, "Project"),
        edited_component_path: bullet_value(contents, "Possible component path"),
        edited_role: bullet_value(contents, "Suggested role")
            .map(|value| ProjectRepositoryRole::parse(&value)),
    }))
}

fn selected_repository_review_decisions(contents: &str) -> Vec<RepositoryReviewDecision> {
    contents
        .lines()
        .filter_map(|line| {
            let normalized = line.trim().to_lowercase();
            if !normalized.starts_with("- [x]") {
                return None;
            }

            if normalized.contains("accept repository record") {
                Some(RepositoryReviewDecision::Accept)
            } else if normalized.contains("accept with edits") {
                Some(RepositoryReviewDecision::AcceptWithEdits)
            } else if normalized.contains("quarantine") {
                Some(RepositoryReviewDecision::Quarantine)
            } else if normalized.contains("reject / skip") {
                Some(RepositoryReviewDecision::Reject)
            } else {
                None
            }
        })
        .collect()
}

fn parse_repository_machine_candidate(
    contents: &str,
) -> Result<RepositoryMigrationCandidate, serde_json::Error> {
    let json = repository_machine_record_json(contents).unwrap_or_default();
    serde_json::from_str(json)
}

fn repository_machine_record_json(contents: &str) -> Option<&str> {
    let heading_start = contents.find(REPOSITORY_MACHINE_RECORD_HEADING)?;
    let after_heading = &contents[heading_start + REPOSITORY_MACHINE_RECORD_HEADING.len()..];
    let fence_start = after_heading.find(REPOSITORY_MACHINE_RECORD_FENCE)?;
    let after_fence = &after_heading[fence_start + REPOSITORY_MACHINE_RECORD_FENCE.len()..];
    let json_start = after_fence.strip_prefix('\n').unwrap_or(after_fence);
    let fence_end = json_start.find("```")?;
    Some(json_start[..fence_end].trim())
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

fn relative_repository_review_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map(path_to_markdown)
        .unwrap_or_else(|_| path.display().to_string())
}

fn canonical_remote_url(candidate: &RepositoryMigrationCandidate) -> Option<String> {
    let normalized = candidate.normalized_remote.clone().or_else(|| {
        candidate
            .remote_url
            .as_deref()
            .and_then(normalize_remote_reference)
    });
    let original = candidate
        .remote_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if let Some(original) = original {
        let sanitized = original
            .trim_end_matches(".git")
            .trim_end_matches('/')
            .to_string();
        if (sanitized.starts_with("git@") || sanitized.starts_with("ssh://git@"))
            && !sanitized.contains("/pull/")
        {
            return Some(format!("{sanitized}.git"));
        }
    }

    normalized.map(|remote| format!("https://{remote}.git"))
}

fn component_name_from_path(path: &str) -> String {
    path.rsplit('/')
        .find(|segment| !segment.trim().is_empty())
        .unwrap_or("component")
        .to_string()
}

fn repository_source(
    source_kind: RepositoryMigrationSourceKind,
    source_id: String,
    source_label: String,
    project_name: Option<String>,
    source_created_at: OffsetDateTime,
    source_updated_at: OffsetDateTime,
) -> RepositoryMigrationEvidence {
    RepositoryMigrationEvidence {
        source_kind,
        source_id,
        source_label,
        project_name,
        excerpt: String::new(),
        source_created_at,
        source_updated_at,
    }
}

fn source_from_project_observation(
    project: &Project,
    observation: &ProjectObservation,
) -> RepositoryMigrationEvidence {
    repository_source(
        RepositoryMigrationSourceKind::ProjectObservation,
        observation.id.to_string(),
        format!("project:{} observation", project.name),
        Some(project.name.clone()),
        observation.created_at,
        observation.updated_at,
    )
}

fn source_from_task_observation(
    project: &Project,
    task: &Task,
    observation: &TaskObservation,
) -> RepositoryMigrationEvidence {
    repository_source(
        RepositoryMigrationSourceKind::TaskObservation,
        observation.id.to_string(),
        format!(
            "task:{} observation",
            task.jira_key.as_deref().unwrap_or(&task.name)
        ),
        Some(project.name.clone()),
        observation.created_at,
        observation.updated_at,
    )
}

fn source_from_pr(project: &Project, pr: &Pr) -> RepositoryMigrationEvidence {
    repository_source(
        RepositoryMigrationSourceKind::PullRequest,
        pr.id.to_string(),
        format!("pr:{}#{}", pr.repo, pr.pr_number),
        Some(project.name.clone()),
        pr.created_at,
        pr.updated_at,
    )
}

fn source_from_entity_description(entity: &Entity) -> RepositoryMigrationEvidence {
    repository_source(
        RepositoryMigrationSourceKind::EntityDescription,
        entity.id.to_string(),
        format!("entity:{} description", entity.name),
        None,
        entity.created_at,
        entity.updated_at,
    )
}

fn source_from_session_record(
    session: &Session,
    field: &str,
    updated_at: OffsetDateTime,
) -> RepositoryMigrationEvidence {
    repository_source(
        RepositoryMigrationSourceKind::SessionRecord,
        format!("{}:{field}", session.id),
        format!("session:{} {field}", session.id),
        session.project.clone(),
        session.started_at,
        updated_at,
    )
}

fn source_from_session_event(session: &Session, event: &Event) -> RepositoryMigrationEvidence {
    repository_source(
        RepositoryMigrationSourceKind::SessionEvent,
        event.id.to_string(),
        format!("session:{} event:{}", session.id, event.event_type),
        session.project.clone(),
        event.timestamp,
        event.timestamp,
    )
}

fn append_repository_candidates(
    text: &str,
    source: RepositoryMigrationEvidence,
    known_checkouts: &[LocalCheckout],
    candidates: &mut Vec<RepositoryMigrationCandidate>,
) {
    let text = text.trim();
    if text.is_empty() {
        return;
    }

    for remote in extract_remote_references(text) {
        let mut evidence = source.clone();
        evidence.excerpt = excerpt_around(text, &remote.original);
        candidates.push(candidate_from_remote(remote, evidence));
    }

    for local_path in extract_local_paths(text) {
        if is_sensitive_local_path_reference(&local_path) {
            continue;
        }
        let mut evidence = source.clone();
        evidence.excerpt = excerpt_around(text, &local_path);
        candidates.push(candidate_from_local_path(
            local_path,
            evidence,
            known_checkouts,
        ));
    }
}

#[derive(Debug, Clone)]
struct RemoteReference {
    original: String,
    normalized: String,
    repository_name: Option<String>,
}

fn candidate_from_remote(
    remote: RemoteReference,
    evidence: RepositoryMigrationEvidence,
) -> RepositoryMigrationCandidate {
    let mut reasons = vec!["Found a Git remote URL/reference in legacy Engram data.".to_string()];
    let project_name = evidence.project_name.clone();
    let mut confidence = 0.78_f32;
    if project_name.is_some() {
        confidence += 0.07;
        reasons
            .push("Source has project scope, so a project-repository link may be needed.".into());
    } else {
        reasons.push("Source has no project scope; review before linking to a project.".into());
    }
    if remote.normalized.starts_with("github.com/") || remote.normalized.starts_with("gitlab.com/")
    {
        confidence += 0.05;
    }

    RepositoryMigrationCandidate {
        reference_kind: RepositoryReferenceKind::Remote,
        repository_name: remote.repository_name,
        remote_url: Some(remote.original),
        normalized_remote: Some(remote.normalized),
        local_path: None,
        project_name,
        component_path: None,
        role: ProjectRepositoryRole::Related,
        confidence: confidence.clamp(0.0, 1.0),
        disposition: RepositoryMigrationDisposition::Review,
        reasons,
        evidence: vec![evidence],
    }
}

fn candidate_from_local_path(
    local_path: String,
    evidence: RepositoryMigrationEvidence,
    known_checkouts: &[LocalCheckout],
) -> RepositoryMigrationCandidate {
    let project_name = evidence.project_name.clone();
    let original_path = canonical_path_string(&local_path);
    let git_root = nearest_git_root(Path::new(&original_path));
    let path = git_root
        .as_ref()
        .map(|root| root.display().to_string())
        .unwrap_or_else(|| original_path.clone());
    let repository_name = repository_name_from_path(&path);
    let component_path = git_root
        .as_ref()
        .and_then(|root| infer_component_path_from_root(&original_path, root))
        .or_else(|| infer_component_path(&path, known_checkouts));
    let mut reasons = vec!["Found a local filesystem path in legacy Engram data.".to_string()];
    let mut confidence = 0.58_f32;
    let mut disposition = if git_root.is_some() || component_path.is_some() {
        RepositoryMigrationDisposition::Review
    } else {
        RepositoryMigrationDisposition::Quarantine
    };

    if path_exists_or_has_known_home_prefix(&path) {
        confidence += 0.08;
    } else {
        confidence -= 0.10;
        reasons.push("Path does not currently exist or may be from another machine.".to_string());
    }

    if git_root.is_some() {
        confidence += 0.15;
        reasons.push("Path resolves to a local Git checkout root.".to_string());
    } else if component_path.is_none() {
        confidence -= 0.12;
        reasons.push(
            "Path is not currently inside a detected or known Git checkout; quarantine before treating as repository topology."
                .to_string(),
        );
    }

    if project_name.is_some() {
        confidence += 0.07;
        reasons.push("Source has project scope, so a project-checkout link may be needed.".into());
    } else {
        disposition = RepositoryMigrationDisposition::Quarantine;
        reasons.push("Source has no project scope; review before treating as canonical.".into());
    }

    if let Some(component_path) = &component_path {
        confidence += 0.08;
        reasons.push(format!(
            "Path is under a known checkout; inferred possible component path `{component_path}`."
        ));
    }

    RepositoryMigrationCandidate {
        reference_kind: RepositoryReferenceKind::LocalPath,
        repository_name,
        remote_url: None,
        normalized_remote: None,
        local_path: Some(path),
        project_name,
        component_path,
        role: ProjectRepositoryRole::Related,
        confidence: confidence.clamp(0.0, 1.0),
        disposition,
        reasons,
        evidence: vec![evidence],
    }
}

fn aggregate_repository_candidates(
    candidates: Vec<RepositoryMigrationCandidate>,
) -> Vec<RepositoryMigrationCandidate> {
    let mut by_key: HashMap<String, RepositoryMigrationCandidate> = HashMap::new();
    for candidate in candidates {
        let key = repository_candidate_key(&candidate);
        if let Some(existing) = by_key.get_mut(&key) {
            existing.confidence = (existing.confidence + 0.03)
                .max(candidate.confidence)
                .min(1.0);
            existing.evidence.extend(candidate.evidence);
            for reason in candidate.reasons {
                if !existing.reasons.contains(&reason) {
                    existing.reasons.push(reason);
                }
            }
            if existing.project_name.is_none() {
                existing.project_name = candidate.project_name;
            }
            if existing.component_path.is_none() {
                existing.component_path = candidate.component_path;
            }
            existing.disposition =
                most_actionable_repository_disposition(existing.disposition, candidate.disposition);
        } else {
            by_key.insert(key, candidate);
        }
    }

    let mut candidates = by_key.into_values().collect::<Vec<_>>();
    for candidate in &mut candidates {
        candidate
            .evidence
            .sort_by(|left, right| left.source_label.cmp(&right.source_label));
        candidate.evidence.dedup_by(|left, right| {
            left.source_kind == right.source_kind && left.source_id == right.source_id
        });
        if candidate.evidence.len() > 1 {
            candidate.confidence = (candidate.confidence + 0.05).min(1.0);
            candidate
                .reasons
                .push("Reference appeared in multiple legacy records.".to_string());
        }
    }
    candidates
}

fn repository_candidate_key(candidate: &RepositoryMigrationCandidate) -> String {
    let project = candidate.project_name.as_deref().unwrap_or("");
    match candidate.reference_kind {
        RepositoryReferenceKind::Remote => format!(
            "remote:{}:{project}",
            candidate.normalized_remote.as_deref().unwrap_or("")
        ),
        RepositoryReferenceKind::LocalPath => {
            format!(
                "path:{}:{project}",
                candidate.local_path.as_deref().unwrap_or("")
            )
        }
    }
}

fn most_actionable_repository_disposition(
    left: RepositoryMigrationDisposition,
    right: RepositoryMigrationDisposition,
) -> RepositoryMigrationDisposition {
    if repository_disposition_rank(left) <= repository_disposition_rank(right) {
        left
    } else {
        right
    }
}

fn repository_disposition_rank(disposition: RepositoryMigrationDisposition) -> u8 {
    match disposition {
        RepositoryMigrationDisposition::Review => 0,
        RepositoryMigrationDisposition::Quarantine => 1,
        RepositoryMigrationDisposition::Skip => 2,
    }
}

fn count_repository_references(
    candidates: &[RepositoryMigrationCandidate],
) -> BTreeMap<String, usize> {
    count_repository_by(
        candidates
            .iter()
            .map(|candidate| candidate.reference_kind.to_string()),
    )
}

fn count_repository_dispositions(
    candidates: &[RepositoryMigrationCandidate],
) -> BTreeMap<String, usize> {
    count_repository_by(
        candidates
            .iter()
            .map(|candidate| candidate.disposition.to_string()),
    )
}

fn count_repository_projects(
    candidates: &[RepositoryMigrationCandidate],
) -> BTreeMap<String, usize> {
    count_repository_by(candidates.iter().map(|candidate| {
        candidate
            .project_name
            .as_deref()
            .unwrap_or("(none)")
            .to_string()
    }))
}

fn count_repository_confidence(
    candidates: &[RepositoryMigrationCandidate],
) -> BTreeMap<String, usize> {
    count_repository_by(
        candidates
            .iter()
            .map(|candidate| repository_confidence_bucket(candidate.confidence).to_string()),
    )
}

fn repository_confidence_bucket(confidence: f32) -> &'static str {
    if confidence >= 0.85 {
        "very_high"
    } else if confidence >= 0.70 {
        "high"
    } else if confidence >= 0.50 {
        "medium"
    } else {
        "low"
    }
}

fn count_repository_by(values: impl Iterator<Item = String>) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for value in values {
        *counts.entry(value).or_insert(0) += 1;
    }
    counts
}

fn extract_remote_references(text: &str) -> Vec<RemoteReference> {
    let mut remotes = Vec::new();
    let mut seen = HashSet::new();
    for raw_token in text.split_whitespace() {
        let token = sanitize_reference_token(raw_token);
        if token.is_empty() {
            continue;
        }
        if let Some(normalized) = normalize_remote_reference(&token) {
            if seen.insert(normalized.clone()) {
                remotes.push(RemoteReference {
                    repository_name: repository_name_from_normalized_remote(&normalized),
                    original: token,
                    normalized,
                });
            }
        }
    }
    remotes
}

fn normalize_remote_reference(value: &str) -> Option<String> {
    let raw = value.trim().trim_end_matches('/');
    let raw_without_query = raw.split(['?', '#']).next()?.trim_end_matches('/');
    let explicit_git_suffix = raw_without_query.trim_end_matches('/').ends_with(".git");
    let uses_git_ssh =
        raw_without_query.starts_with("git@") || raw_without_query.starts_with("ssh://git@");
    let trimmed = raw_without_query
        .trim_end_matches(".git")
        .trim_end_matches('/');

    let without_scheme = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
        .or_else(|| trimmed.strip_prefix("ssh://git@"))
        .or_else(|| trimmed.strip_prefix("git@"))
        .or_else(|| {
            (trimmed.starts_with("github.com/")
                || trimmed.starts_with("gitlab.com/")
                || trimmed.starts_with("bitbucket.org/"))
            .then_some(trimmed)
        });

    let remote = without_scheme?;
    let (host, path) = if let Some((host, path)) = remote.split_once(':') {
        (host, path)
    } else if let Some((host, path)) = remote.split_once('/') {
        (host, path)
    } else {
        return None;
    };

    if !host.contains('.') {
        return None;
    }
    if !is_git_remote_host(host) && !explicit_git_suffix && !uses_git_ssh {
        return None;
    }

    let mut segments = path
        .trim_start_matches('/')
        .split('/')
        .filter(|segment| !segment.trim().is_empty());
    let owner = segments.next()?.trim_end_matches(".git");
    let repo = segments.next()?.trim_end_matches(".git");
    if owner.is_empty() || repo.is_empty() {
        return None;
    }

    Some(
        format!("{}/{}/{}", host.to_lowercase(), owner, repo)
            .trim_end_matches(".git")
            .to_string(),
    )
}

fn is_git_remote_host(host: &str) -> bool {
    matches!(
        host.to_lowercase().as_str(),
        "github.com" | "gitlab.com" | "bitbucket.org"
    ) || host.to_lowercase().contains(".git.")
}

fn repository_name_from_normalized_remote(remote: &str) -> Option<String> {
    remote
        .rsplit('/')
        .next()
        .filter(|name| !name.trim().is_empty())
        .map(str::to_string)
}

fn extract_local_paths(text: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let mut seen = HashSet::new();
    for raw_token in text.split_whitespace() {
        let token = trim_line_suffix(&sanitize_reference_token(raw_token));
        if is_probable_local_path(&token) && seen.insert(token.clone()) {
            paths.push(token);
        }
    }
    paths
}

fn sanitize_reference_token(value: &str) -> String {
    let token = value
        .split(['\n', '\r'])
        .next()
        .unwrap_or(value)
        .split("\\n")
        .next()
        .unwrap_or(value);
    token
        .trim_matches(|ch: char| {
            matches!(
                ch,
                '`' | '"' | '\'' | '<' | '>' | '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';'
            )
        })
        .trim_end_matches(['.', ',', ';', ':', ')', ']', '}'])
        .to_string()
}

fn trim_line_suffix(value: &str) -> String {
    let Some((path, suffix)) = value.rsplit_once(':') else {
        return value.to_string();
    };
    if suffix.chars().all(|ch| ch.is_ascii_digit()) && path.starts_with('/') {
        path.to_string()
    } else {
        value.to_string()
    }
}

fn is_probable_local_path(value: &str) -> bool {
    (value.starts_with("/Users/")
        || value.starts_with("/home/")
        || value.starts_with("/workspace/")
        || value.starts_with("/workspaces/")
        || value.starts_with("~/"))
        && !value.contains("://")
        && value.matches('/').count() >= 2
}

fn canonical_path_string(path: &str) -> String {
    let expanded = path.strip_prefix("~/").map(|suffix| {
        std::env::var("HOME")
            .map(|home| format!("{home}/{suffix}"))
            .unwrap_or_else(|_| path.to_string())
    });
    let expanded = expanded.as_deref().unwrap_or(path);
    Path::new(expanded)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(expanded))
        .display()
        .to_string()
}

fn path_exists_or_has_known_home_prefix(path: &str) -> bool {
    Path::new(path).exists()
        || path.starts_with("/Users/")
        || path.starts_with("/home/")
        || path.starts_with("/workspace/")
        || path.starts_with("/workspaces/")
}

fn is_sensitive_local_path_reference(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.contains("/.mcp-credentials/")
        || lower.contains("/.aws/")
        || lower.contains("/.ssh/")
        || lower.contains("/.config/gcloud/")
        || lower.contains("/credentials")
        || lower.contains("/credential")
        || lower.contains("/secrets")
        || lower.contains("/secret")
        || lower.ends_with("credentials.json")
        || lower.ends_with("credential.json")
        || lower.ends_with("secrets.json")
        || lower.ends_with("secret.json")
        || lower.ends_with(".pem")
        || lower.ends_with(".key")
}

fn repository_name_from_path(path: &str) -> Option<String> {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .map(str::to_string)
}

fn infer_component_path(path: &str, known_checkouts: &[LocalCheckout]) -> Option<String> {
    let path = canonical_or_original(Path::new(path));
    let checkout = known_checkouts
        .iter()
        .filter(|checkout| {
            let checkout_path = canonical_or_original(Path::new(&checkout.local_path));
            path_starts_with(&path, &checkout_path)
        })
        .max_by_key(|checkout| {
            canonical_or_original(Path::new(&checkout.local_path))
                .components()
                .count()
        })?;
    let checkout_path = canonical_or_original(Path::new(&checkout.local_path));
    let relative = path.strip_prefix(checkout_path).ok()?;
    if relative.as_os_str().is_empty() {
        return None;
    }
    let components = relative
        .components()
        .take(2)
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    (!components.is_empty()).then(|| normalize_component_path(&components.join("/")))
}

fn nearest_git_root(path: &Path) -> Option<PathBuf> {
    let mut current = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent()?.to_path_buf()
    };

    loop {
        if current.join(".git").exists() {
            return Some(canonical_or_original(&current));
        }
        if !current.pop() {
            return None;
        }
    }
}

fn infer_component_path_from_root(path: &str, git_root: &Path) -> Option<String> {
    let path = canonical_or_original(Path::new(path));
    let relative = path.strip_prefix(git_root).ok()?;
    if relative.as_os_str().is_empty() {
        return None;
    }
    let components = relative
        .components()
        .take(2)
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    (!components.is_empty()).then(|| normalize_component_path(&components.join("/")))
}

fn excerpt_around(text: &str, needle: &str) -> String {
    let Some(index) = text.find(needle) else {
        return text.chars().take(240).collect();
    };
    let start = text[..index]
        .char_indices()
        .rev()
        .nth(80)
        .map(|(idx, _)| idx)
        .unwrap_or(0);
    let end = text[index..]
        .char_indices()
        .nth(needle.chars().count() + 80)
        .map(|(idx, _)| index + idx)
        .unwrap_or(text.len());
    text[start..end].trim().to_string()
}

fn write_repository_migration_review(
    root: &Path,
    inventory: RepositoryMigrationInventory,
) -> IndexResult<RepositoryMigrationReviewExport> {
    fs::create_dir_all(root)?;

    let mut export = RepositoryMigrationReviewExport {
        root: root.display().to_string(),
        files_written: Vec::new(),
        files_skipped: Vec::new(),
        inventory,
    };

    write_repository_review_file(
        root,
        Path::new("index.md").to_path_buf(),
        &repository_review_index_page(&export.inventory),
        &mut export,
    )?;

    let candidates = export.inventory.candidates.clone();
    for (index, candidate) in candidates.iter().enumerate() {
        write_repository_review_file(
            root,
            repository_candidate_review_path(index, candidate),
            &repository_candidate_review_page(index, candidate),
            &mut export,
        )?;
    }

    export.files_written.sort();
    export.files_skipped.sort();
    Ok(export)
}

fn repository_review_index_page(inventory: &RepositoryMigrationInventory) -> String {
    let mut output = repository_review_frontmatter(
        "repository_migration_review_index",
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

    output.push_str("# Repository Migration Review Batch\n\n");
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

    append_repository_count_section(
        &mut output,
        "Reference Counts",
        &inventory.by_reference_kind,
    );
    append_repository_count_section(&mut output, "Disposition Counts", &inventory.by_disposition);
    append_repository_count_section(&mut output, "Project Counts", &inventory.by_project);
    append_repository_count_section(&mut output, "Confidence Counts", &inventory.by_confidence);
    append_repository_candidate_queue_section(
        &mut output,
        "Review Queue",
        inventory,
        RepositoryMigrationDisposition::Review,
    );
    append_repository_candidate_queue_section(
        &mut output,
        "Quarantine Queue",
        inventory,
        RepositoryMigrationDisposition::Quarantine,
    );
    append_repository_candidate_queue_section(
        &mut output,
        "Skip Queue",
        inventory,
        RepositoryMigrationDisposition::Skip,
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
        output.push_str("No repository topology migration candidates in this batch.\n");
    } else {
        for (index, candidate) in inventory.candidates.iter().enumerate() {
            output.push_str(&format!(
                "- [{}]({}) - {} - {} - {:.2}\n",
                escape_link_text(&repository_candidate_title(candidate)),
                path_to_markdown(&repository_candidate_review_path(index, candidate)),
                candidate.disposition,
                candidate.reference_kind,
                candidate.confidence
            ));
        }
    }

    output
}

fn repository_candidate_review_page(
    index: usize,
    candidate: &RepositoryMigrationCandidate,
) -> String {
    let mut output = repository_review_frontmatter(
        "repository_migration_candidate_review",
        vec![
            ("candidate_number".to_string(), (index + 1).to_string()),
            (
                "reference_kind".to_string(),
                yaml_string(&candidate.reference_kind.to_string()),
            ),
            (
                "disposition".to_string(),
                yaml_string(&candidate.disposition.to_string()),
            ),
            (
                "confidence".to_string(),
                format!("{:.3}", candidate.confidence),
            ),
        ],
    );

    output.push_str(&format!("# {}\n\n", repository_candidate_title(candidate)));
    output.push_str("## Review Decision\n\n");
    output.push_str("- [ ] Accept repository record\n");
    output.push_str("- [ ] Accept with edits\n");
    output.push_str("- [ ] Quarantine\n");
    output.push_str("- [ ] Reject / skip\n\n");
    output.push_str("Reviewer notes:\n\n");

    output.push_str("## Proposed Topology\n\n");
    output.push_str(&format!("- Reference kind: {}\n", candidate.reference_kind));
    if let Some(name) = &candidate.repository_name {
        output.push_str(&format!("- Repository name: {}\n", name));
    }
    if let Some(remote) = &candidate.remote_url {
        output.push_str(&format!("- Remote URL: {}\n", remote));
    }
    if let Some(remote) = &candidate.normalized_remote {
        output.push_str(&format!("- Normalized remote: {}\n", remote));
    }
    if let Some(path) = &candidate.local_path {
        output.push_str(&format!("- Local path: {}\n", path));
    }
    if let Some(project) = &candidate.project_name {
        output.push_str(&format!("- Project: {}\n", project));
        output.push_str(&format!("- Suggested role: {}\n", candidate.role));
    }
    if let Some(component_path) = &candidate.component_path {
        output.push_str(&format!("- Possible component path: {}\n", component_path));
    }
    output.push_str(&format!("- Confidence: {:.3}\n", candidate.confidence));
    output.push_str(&format!("- Disposition: {}\n\n", candidate.disposition));

    output.push_str("## Evidence\n\n");
    for evidence in &candidate.evidence {
        output.push_str(&format!(
            "- {} `{}` ({})\n",
            evidence.source_kind, evidence.source_id, evidence.source_label
        ));
        if let Some(project_name) = &evidence.project_name {
            output.push_str(&format!("  Project: {}\n", project_name));
        }
        if !evidence.excerpt.is_empty() {
            output.push_str(&format!("  Excerpt: {}\n", evidence.excerpt));
        }
    }
    output.push('\n');

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
            .expect("repository migration candidate JSON serialization should succeed"),
    );
    output.push_str("\n```\n");

    output
}

fn repository_candidate_title(candidate: &RepositoryMigrationCandidate) -> String {
    candidate
        .repository_name
        .clone()
        .or_else(|| candidate.normalized_remote.clone())
        .or_else(|| candidate.local_path.clone())
        .unwrap_or_else(|| "Unknown repository reference".to_string())
}

fn append_repository_count_section(
    output: &mut String,
    title: &str,
    counts: &BTreeMap<String, usize>,
) {
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

fn append_repository_candidate_queue_section(
    output: &mut String,
    title: &str,
    inventory: &RepositoryMigrationInventory,
    disposition: RepositoryMigrationDisposition,
) {
    output.push_str(&format!("## {}\n\n", title));
    let mut found = false;
    for (index, candidate) in inventory.candidates.iter().enumerate() {
        if candidate.disposition != disposition {
            continue;
        }
        found = true;
        output.push_str(&format!(
            "- [{}]({}) - {} - {:.2}\n",
            escape_link_text(&repository_candidate_title(candidate)),
            path_to_markdown(&repository_candidate_review_path(index, candidate)),
            candidate.reference_kind,
            candidate.confidence
        ));
    }
    if !found {
        output.push_str("No entries.\n");
    }
    output.push('\n');
}

fn repository_review_frontmatter(page_type: &str, fields: Vec<(String, String)>) -> String {
    let mut output = String::new();
    output.push_str("---\n");
    output.push_str(&format!(
        "generated_by: {}\n",
        yaml_string(REPOSITORY_REVIEW_GENERATED_BY)
    ));
    output.push_str(&format!("page_type: {}\n", yaml_string(page_type)));
    for (key, value) in fields {
        output.push_str(&format!("{key}: {value}\n"));
    }
    output.push_str("---\n\n");
    output.push_str(REPOSITORY_REVIEW_GENERATED_MARKER);
    output.push_str("\n\n");
    output
}

fn write_repository_review_file(
    root: &Path,
    relative_path: PathBuf,
    contents: &str,
    export: &mut RepositoryMigrationReviewExport,
) -> IndexResult<()> {
    let path = root.join(&relative_path);
    if path.exists() {
        let existing = fs::read(&path)?;
        if !String::from_utf8_lossy(&existing).contains(REPOSITORY_REVIEW_GENERATED_MARKER) {
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

fn repository_candidate_review_path(
    index: usize,
    candidate: &RepositoryMigrationCandidate,
) -> PathBuf {
    Path::new("candidates").join(format!(
        "{:04}-{}-{}.md",
        index + 1,
        candidate.disposition,
        slugify(&repository_candidate_title(candidate))
    ))
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
        "unknown".to_string()
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

fn validate_non_empty(value: &str, label: &str) -> IndexResult<()> {
    if value.trim().is_empty() {
        Err(IndexError::Parse(format!("{label} must not be empty")))
    } else {
        Ok(())
    }
}

fn run_git_required(cwd: &Path, args: &[&str]) -> IndexResult<String> {
    let output = Command::new("git").arg("-C").arg(cwd).args(args).output()?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if stdout.is_empty() {
            Err(IndexError::InvalidState(format!(
                "git {} returned empty output",
                args.join(" ")
            )))
        } else {
            Ok(stdout)
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(IndexError::NotFound(format!(
            "git {} failed in {}: {}",
            args.join(" "),
            cwd.display(),
            stderr
        )))
    }
}

fn run_git_optional(cwd: &Path, args: &[&str]) -> IndexResult<Option<String>> {
    let output = Command::new("git").arg("-C").arg(cwd).args(args).output()?;
    if !output.status.success() {
        return Ok(None);
    }
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stdout.is_empty() {
        Ok(None)
    } else {
        Ok(Some(stdout))
    }
}

pub(crate) fn refresh_checkout_git_state(checkout: &mut LocalCheckout) -> IndexResult<bool> {
    let checkout_path = canonical_or_original(Path::new(&checkout.local_path));
    let Some(git_root) = run_git_optional(&checkout_path, &["rev-parse", "--show-toplevel"])?
    else {
        return Ok(false);
    };
    let git_root_path = canonical_or_original(Path::new(git_root.trim()));
    if git_root_path != checkout_path {
        return Ok(false);
    }

    let current_branch = run_git_optional(&checkout_path, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    let head_sha = run_git_optional(&checkout_path, &["rev-parse", "HEAD"])?;
    let is_dirty = detect_dirty(&checkout_path)?;

    if current_branch.is_none() && head_sha.is_none() && is_dirty.is_none() {
        return Ok(false);
    }

    checkout.update_detected_state(current_branch, head_sha, is_dirty);
    Ok(true)
}

fn detect_default_branch(cwd: &Path) -> IndexResult<Option<String>> {
    let Some(branch) = run_git_optional(
        cwd,
        &["symbolic-ref", "--short", "refs/remotes/origin/HEAD"],
    )?
    else {
        return Ok(None);
    };
    Ok(Some(
        branch
            .strip_prefix("origin/")
            .unwrap_or(branch.as_str())
            .to_string(),
    ))
}

fn detect_dirty(cwd: &Path) -> IndexResult<Option<bool>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(["status", "--porcelain"])
        .output()?;
    if !output.status.success() {
        return Ok(None);
    }
    Ok(Some(
        !String::from_utf8_lossy(&output.stdout).trim().is_empty(),
    ))
}

fn repository_name_from_remote(remote_url: &str) -> Option<String> {
    let trimmed = remote_url.trim().trim_end_matches(".git");
    let name = trimmed
        .rsplit(['/', ':'])
        .next()
        .filter(|value| !value.trim().is_empty())?;
    Some(name.to_string())
}

fn normalize_component_path(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() || trimmed == "." {
        return ".".to_string();
    }
    trimmed
        .trim_start_matches("./")
        .trim_end_matches('/')
        .to_string()
}

fn path_starts_with(path: &Path, prefix: &Path) -> bool {
    path == prefix || path.starts_with(prefix)
}

fn canonical_or_original(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn matching_components(
    cwd: &Path,
    checkout_root: &Path,
    components: Vec<MonorepoComponent>,
) -> Vec<MonorepoComponent> {
    let relative = cwd.strip_prefix(checkout_root).unwrap_or(cwd);
    let mut matches: Vec<_> = components
        .into_iter()
        .filter(|component| {
            component.path == "." || relative.starts_with(Path::new(&component.path))
        })
        .collect();
    matches.sort_by_key(|component| std::cmp::Reverse(component.path.len()));
    matches
}

#[cfg(test)]
mod tests {
    use super::*;
    use engram_core::entity::EntityType;
    use tempfile::tempdir;

    async fn setup_service() -> RepositoryService {
        let config = engram_store::StoreConfig::memory();
        let db = engram_store::connect_and_init(&config).await.unwrap();
        let service = RepositoryService::new(db);
        service.init_schema().await.unwrap();
        service
    }

    async fn setup_workspace() -> (RepositoryService, WorkRepo, EntityRepo, SessionRepo) {
        let config = engram_store::StoreConfig::memory();
        let db = engram_store::connect_and_init(&config).await.unwrap();
        let service = RepositoryService::new(db.clone());
        service.init_schema().await.unwrap();
        (
            service,
            WorkRepo::new(db.clone()),
            EntityRepo::new(db.clone()),
            SessionRepo::new(db),
        )
    }

    fn migration_writer() -> WriterProvenance {
        WriterProvenance::agent(
            engram_core::memory::Harness::Codex,
            engram_core::memory::ModelIdentity::new("openai", "gpt-5.5"),
        )
        .with_surface("test")
    }

    fn run_git(cwd: &Path, args: &[&str]) {
        let output = Command::new("git")
            .arg("-C")
            .arg(cwd)
            .args(args)
            .output()
            .expect("git should run");
        assert!(
            output.status.success(),
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn commit_all(cwd: &Path, message: &str) {
        run_git(cwd, &["add", "."]);
        run_git(
            cwd,
            &[
                "-c",
                "user.name=Engram Test",
                "-c",
                "user.email=engram-test@example.com",
                "commit",
                "-m",
                message,
            ],
        );
    }

    fn git_available() -> bool {
        Command::new("git")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    #[tokio::test]
    async fn detect_registers_git_checkout() {
        if !git_available() {
            return;
        }
        let service = setup_service().await;
        let dir = tempdir().unwrap();
        run_git(dir.path(), &["init"]);
        run_git(
            dir.path(),
            &[
                "remote",
                "add",
                "origin",
                "git@github.com:ymeiri/engram.git",
            ],
        );

        let detection = service.detect_repository(dir.path()).await.unwrap();

        assert_eq!(detection.context.repository.name, "engram");
        assert_eq!(
            detection.context.repository.remote_url.as_deref(),
            Some("git@github.com:ymeiri/engram.git")
        );
        assert_eq!(
            detection.context.checkout.as_ref().unwrap().repository_id,
            Some(detection.context.repository.id)
        );
    }

    #[tokio::test]
    async fn component_and_project_links_are_resolved_from_cwd() {
        if !git_available() {
            return;
        }
        let service = setup_service().await;
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("services/api")).unwrap();
        run_git(dir.path(), &["init"]);
        run_git(
            dir.path(),
            &[
                "remote",
                "add",
                "origin",
                "https://github.com/acme/mono.git",
            ],
        );

        let detection = service.detect_repository(dir.path()).await.unwrap();
        service
            .register_component(
                Some(&detection.context.repository.id),
                None,
                "api",
                "services/api",
                Some("service"),
                None,
            )
            .await
            .unwrap();
        service
            .link_project(
                "Debug with AI",
                Some(&detection.context.repository.id),
                None,
                ProjectRepositoryRole::Primary,
                None,
            )
            .await
            .unwrap();

        let context = service
            .resolve_cwd(&dir.path().join("services/api"))
            .await
            .unwrap()
            .unwrap();

        assert_eq!(context.matching_components.len(), 1);
        assert_eq!(context.matching_components[0].name, "api");
        assert_eq!(context.linked_projects.len(), 1);
        assert_eq!(context.linked_projects[0].project_name, "Debug with AI");
    }

    #[tokio::test]
    async fn resolve_cwd_refreshes_checkout_git_state() {
        if !git_available() {
            return;
        }
        let service = setup_service().await;
        let dir = tempdir().unwrap();
        run_git(dir.path(), &["init"]);
        run_git(
            dir.path(),
            &[
                "remote",
                "add",
                "origin",
                "https://github.com/acme/fresh.git",
            ],
        );
        std::fs::write(dir.path().join("README.md"), "first\n").unwrap();
        commit_all(dir.path(), "first");

        let detection = service.detect_repository(dir.path()).await.unwrap();
        let first_head = detection
            .context
            .checkout
            .as_ref()
            .and_then(|checkout| checkout.head_sha.clone())
            .expect("detection should record initial HEAD");

        std::fs::write(dir.path().join("README.md"), "second\n").unwrap();
        commit_all(dir.path(), "second");
        let second_head = run_git_required(dir.path(), &["rev-parse", "HEAD"]).unwrap();
        assert_ne!(first_head, second_head);

        let context = service.resolve_cwd(dir.path()).await.unwrap().unwrap();
        let checkout = context.checkout.expect("context should include checkout");
        assert_eq!(checkout.head_sha.as_deref(), Some(second_head.as_str()));
        assert_eq!(checkout.is_dirty, Some(false));
        assert!(checkout.last_seen_at >= detection.context.checkout.unwrap().last_seen_at);
    }

    #[tokio::test]
    async fn repository_migration_inventory_finds_remotes_paths_and_projects() {
        let (service, work_repo, entity_repo, session_repo) = setup_workspace().await;
        let project = Project::new("Debug with AI").with_description(
            "Uses git@github.com:acme/co-gen-backend.git and /Users/yuval/projects/webui.",
        );
        work_repo.create_project(&project).await.unwrap();
        work_repo
            .add_project_observation(
                &ProjectObservation::new(
                    project.id,
                    "dd-source lives at github.com/DataDog/dd-source and includes services/debug-ai.",
                )
                .with_key("repos.dd-source"),
            )
            .await
            .unwrap();

        let entity = Entity::new("webui", EntityType::Repo)
            .with_description("Frontend remote is https://github.com/acme/webui.git.");
        entity_repo.save_entity(&entity).await.unwrap();

        let session = Session::new()
            .with_project("Debug with AI")
            .with_agent("codex")
            .with_goal("Work in /Users/yuval/projects/dd-source/services/debug-ai.");
        session_repo.save_session(&session).await.unwrap();

        let inventory = service
            .migration_inventory(RepositoryMigrationOptions {
                project_filter: Some("Debug with AI".to_string()),
                ..RepositoryMigrationOptions::all()
            })
            .await
            .unwrap();

        assert!(inventory.sources_scanned >= 3);
        assert!(inventory.total_candidates >= 3);
        assert!(inventory
            .candidates
            .iter()
            .any(|candidate| candidate.normalized_remote.as_deref()
                == Some("github.com/acme/co-gen-backend")));
        assert!(inventory.candidates.iter().any(
            |candidate| candidate.local_path.as_deref() == Some("/Users/yuval/projects/webui")
        ));
        assert!(inventory
            .candidates
            .iter()
            .any(|candidate| candidate.project_name.as_deref() == Some("Debug with AI")));
    }

    #[tokio::test]
    async fn repository_migration_project_filter_is_case_insensitive_and_never_broadens_on_miss() {
        let (service, work_repo, _entity_repo, _session_repo) = setup_workspace().await;
        let engram = Project::new("engram")
            .with_description("Canonical repo is https://github.com/ymeiri/engram.git.");
        let other = Project::new("other-project")
            .with_description("Canonical repo is https://github.com/acme/other.git.");
        work_repo.create_project(&engram).await.unwrap();
        work_repo.create_project(&other).await.unwrap();

        let inventory = service
            .migration_inventory(RepositoryMigrationOptions {
                project_filter: Some("Engram".to_string()),
                include_entity_observations: false,
                include_session_history: false,
                include_work_records: true,
                limit: None,
            })
            .await
            .unwrap();

        assert_eq!(inventory.sources_scanned, 1);
        assert_eq!(inventory.total_candidates, 1);
        assert_eq!(
            inventory.candidates[0].normalized_remote.as_deref(),
            Some("github.com/ymeiri/engram")
        );
        assert!(inventory
            .warnings
            .iter()
            .all(|warning| !warning.contains("did not match")));

        let missing = service
            .migration_inventory(RepositoryMigrationOptions {
                project_filter: Some("missing-project".to_string()),
                include_entity_observations: false,
                include_session_history: false,
                include_work_records: true,
                limit: None,
            })
            .await
            .unwrap();

        assert_eq!(missing.sources_scanned, 0);
        assert_eq!(missing.total_candidates, 0);
        assert!(missing
            .warnings
            .iter()
            .any(|warning| warning.contains("did not match")));
    }

    #[tokio::test]
    async fn repository_migration_skips_sensitive_local_path_references() {
        let (service, work_repo, _entity_repo, _session_repo) = setup_workspace().await;
        let project = Project::new("engram");
        work_repo.create_project(&project).await.unwrap();
        work_repo
            .add_project_observation(
                &ProjectObservation::new(
                    project.id,
                    "Local auth file is /Users/yuval.meiri/.mcp-credentials/google-credentials.json.",
                )
                .with_key("config.google-auth"),
            )
            .await
            .unwrap();

        let inventory = service
            .migration_inventory(RepositoryMigrationOptions {
                project_filter: Some("engram".to_string()),
                include_entity_observations: false,
                include_session_history: false,
                include_work_records: true,
                limit: None,
            })
            .await
            .unwrap();

        assert_eq!(inventory.sources_scanned, 1);
        assert_eq!(inventory.total_candidates, 0);
        assert!(inventory.candidates.is_empty());
    }

    #[tokio::test]
    async fn repository_migration_review_export_writes_generated_pages() {
        let (service, work_repo, _entity_repo, _session_repo) = setup_workspace().await;
        let project = Project::new("engram")
            .with_description("Canonical repository is https://github.com/ymeiri/engram.git.");
        work_repo.create_project(&project).await.unwrap();

        let dir = tempdir().unwrap();
        let export = service
            .export_migration_review(dir.path(), RepositoryMigrationOptions::all())
            .await
            .unwrap();

        assert_eq!(export.inventory.total_candidates, 1);
        assert_eq!(export.file_count(), 2);
        assert!(dir.path().join("index.md").exists());
        assert!(export
            .files_written
            .iter()
            .any(|path| path.starts_with("candidates/")));

        let index = std::fs::read_to_string(dir.path().join("index.md")).unwrap();
        assert!(index.contains(REPOSITORY_REVIEW_GENERATED_MARKER));
        assert!(index.contains("Repository Migration Review Batch"));
        assert!(index.contains("Confidence Counts"));
        assert!(index.contains("Review Queue"));
        assert!(index.contains("Quarantine Queue"));
        assert!(index.contains("Skip Queue"));

        let candidate_path = export
            .files_written
            .iter()
            .find(|path| path.starts_with("candidates/"))
            .unwrap();
        let candidate = std::fs::read_to_string(dir.path().join(candidate_path)).unwrap();
        assert!(candidate.contains("- [ ] Accept repository record"));
        assert!(candidate.contains("github.com/ymeiri/engram"));
        assert!(candidate.contains("## Machine Record"));
    }

    #[tokio::test]
    async fn repository_migration_review_export_skips_user_owned_files() {
        let (service, _work_repo, _entity_repo, _session_repo) = setup_workspace().await;
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("index.md"), "# User repository notes\n").unwrap();

        let export = service
            .export_migration_review(dir.path(), RepositoryMigrationOptions::all())
            .await
            .unwrap();

        assert_eq!(export.inventory.total_candidates, 0);
        assert_eq!(export.files_written.len(), 0);
        assert_eq!(export.files_skipped, vec!["index.md"]);
        assert_eq!(
            std::fs::read_to_string(dir.path().join("index.md")).unwrap(),
            "# User repository notes\n"
        );
    }

    #[tokio::test]
    async fn repository_migration_review_apply_dry_run_does_not_write() {
        let (service, work_repo, _entity_repo, _session_repo) = setup_workspace().await;
        let project = Project::new("engram")
            .with_description("Canonical repository is https://github.com/ymeiri/engram.git.");
        work_repo.create_project(&project).await.unwrap();

        let dir = tempdir().unwrap();
        let export = service
            .export_migration_review(dir.path(), RepositoryMigrationOptions::all())
            .await
            .unwrap();
        check_first_repository_candidate(dir.path(), &export, "Accept repository record");

        let apply = service
            .apply_migration_review(
                dir.path(),
                RepositoryMigrationReviewApplyOptions {
                    dry_run: true,
                    writer: Some(migration_writer()),
                    create_commit: true,
                },
            )
            .await
            .unwrap();

        assert_eq!(apply.planned_count(), 1);
        assert_eq!(apply.written_count(), 0);
        assert_eq!(apply.accepted_count, 1);
        assert!(apply.commit.is_none());
        assert!(service.list_repositories(None).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn repository_migration_review_apply_ignores_accepted_orphan_not_listed_in_index() {
        let (service, work_repo, _entity_repo, _session_repo) = setup_workspace().await;
        let project = Project::new("engram")
            .with_description("Canonical repository is https://github.com/ymeiri/engram.git.");
        work_repo.create_project(&project).await.unwrap();

        let dir = tempdir().unwrap();
        let export = service
            .export_migration_review(dir.path(), RepositoryMigrationOptions::all())
            .await
            .unwrap();
        let indexed_candidate = first_repository_candidate_path(&export);
        let orphan_path = dir.path().join("candidates/9999-review-stale.md");
        std::fs::copy(dir.path().join(&indexed_candidate), &orphan_path).unwrap();
        check_repository_candidate_at_path(&orphan_path, "Accept repository record");

        let apply = service
            .apply_migration_review(
                dir.path(),
                RepositoryMigrationReviewApplyOptions {
                    dry_run: false,
                    writer: Some(migration_writer()),
                    create_commit: true,
                },
            )
            .await
            .unwrap();

        assert_eq!(apply.planned_count(), 0);
        assert_eq!(apply.written_count(), 0);
        assert_eq!(apply.accepted_count, 0);
        assert_eq!(apply.files_with_no_decision, vec![indexed_candidate]);
        assert!(apply
            .files_skipped
            .contains(&"candidates/9999-review-stale.md".to_string()));
        assert_eq!(
            apply.files_not_in_index,
            vec!["candidates/9999-review-stale.md".to_string()]
        );
        assert!(apply
            .warnings
            .iter()
            .any(|warning| warning.contains("not listed in generated index.md")));
        assert!(service.list_repositories(None).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn repository_migration_review_status_reports_pending_conflicts_orphans_and_missing() {
        let (service, work_repo, _entity_repo, _session_repo) = setup_workspace().await;
        for (name, remote) in [
            ("repo-one", "https://github.com/acme/repo-one.git"),
            ("repo-two", "https://github.com/acme/repo-two.git"),
            ("repo-three", "https://github.com/acme/repo-three.git"),
        ] {
            work_repo
                .create_project(
                    &Project::new(name)
                        .with_description(format!("Canonical repository is {remote}.")),
                )
                .await
                .unwrap();
        }

        let dir = tempdir().unwrap();
        let export = service
            .export_migration_review(dir.path(), RepositoryMigrationOptions::all())
            .await
            .unwrap();
        let paths = repository_candidate_paths(&export);
        assert_eq!(paths.len(), 3);
        check_repository_candidate(dir.path(), &paths[1], "Accept repository record");
        check_repository_candidate(dir.path(), &paths[1], "Quarantine");
        std::fs::remove_file(dir.path().join(&paths[2])).unwrap();
        let orphan_path = dir.path().join("candidates/9999-review-orphan.md");
        std::fs::copy(dir.path().join(&paths[0]), &orphan_path).unwrap();
        check_repository_candidate_at_path(&orphan_path, "Accept repository record");

        let status = service.migration_review_status(dir.path()).await.unwrap();

        assert!(!status.ready_to_apply);
        assert_eq!(status.files_scanned, 2);
        assert_eq!(status.files_with_no_decision, vec![paths[0].clone()]);
        assert_eq!(status.files_with_conflicts, vec![paths[1].clone()]);
        assert_eq!(status.indexed_files_missing, vec![paths[2].clone()]);
        assert_eq!(
            status.files_not_in_index,
            vec!["candidates/9999-review-orphan.md".to_string()]
        );
        assert_eq!(status.planned_record_count, 0);
        assert_eq!(status.accepted_count, 0);
    }

    #[tokio::test]
    async fn repository_migration_review_apply_writes_only_accepted_mixed_decisions() {
        let (service, work_repo, _entity_repo, _session_repo) = setup_workspace().await;
        for (name, remote) in [
            ("repo-one", "https://github.com/acme/repo-one.git"),
            ("repo-two", "https://github.com/acme/repo-two.git"),
            ("repo-three", "https://github.com/acme/repo-three.git"),
        ] {
            work_repo
                .create_project(
                    &Project::new(name)
                        .with_description(format!("Canonical repository is {remote}.")),
                )
                .await
                .unwrap();
        }

        let dir = tempdir().unwrap();
        let export = service
            .export_migration_review(dir.path(), RepositoryMigrationOptions::all())
            .await
            .unwrap();
        let paths = repository_candidate_paths(&export);
        assert_eq!(paths.len(), 3);
        check_repository_candidate(dir.path(), &paths[0], "Accept repository record");
        check_repository_candidate(dir.path(), &paths[1], "Quarantine");
        check_repository_candidate(dir.path(), &paths[2], "Reject / skip");

        let apply = service
            .apply_migration_review(
                dir.path(),
                RepositoryMigrationReviewApplyOptions {
                    dry_run: false,
                    writer: Some(migration_writer()),
                    create_commit: true,
                },
            )
            .await
            .unwrap();

        assert_eq!(apply.planned_count(), 1);
        assert_eq!(apply.written_count(), 1);
        assert_eq!(apply.accepted_count, 1);
        assert_eq!(apply.quarantined_count, 1);
        assert_eq!(apply.rejected_count, 1);
        assert_eq!(apply.accepted_files, vec![paths[0].clone()]);
        assert_eq!(apply.quarantined_files, vec![paths[1].clone()]);
        assert_eq!(apply.rejected_files, vec![paths[2].clone()]);
        assert_eq!(service.list_repositories(None).await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn repository_migration_review_status_is_ready_when_all_indexed_files_are_decided() {
        let (service, work_repo, _entity_repo, _session_repo) = setup_workspace().await;
        for (name, remote) in [
            ("repo-one", "https://github.com/acme/repo-one.git"),
            ("repo-two", "https://github.com/acme/repo-two.git"),
            ("repo-three", "https://github.com/acme/repo-three.git"),
        ] {
            work_repo
                .create_project(
                    &Project::new(name)
                        .with_description(format!("Canonical repository is {remote}.")),
                )
                .await
                .unwrap();
        }

        let dir = tempdir().unwrap();
        let export = service
            .export_migration_review(dir.path(), RepositoryMigrationOptions::all())
            .await
            .unwrap();
        let paths = repository_candidate_paths(&export);
        check_repository_candidate(dir.path(), &paths[0], "Accept repository record");
        check_repository_candidate(dir.path(), &paths[1], "Quarantine");
        check_repository_candidate(dir.path(), &paths[2], "Reject / skip");

        let status = service.migration_review_status(dir.path()).await.unwrap();

        assert!(status.ready_to_apply);
        assert_eq!(status.planned_record_count, 1);
        assert_eq!(status.accepted_files, vec![paths[0].clone()]);
        assert_eq!(status.quarantined_files, vec![paths[1].clone()]);
        assert_eq!(status.rejected_files, vec![paths[2].clone()]);
        assert!(status.files_skipped.is_empty());
    }

    #[tokio::test]
    async fn repository_migration_review_apply_writes_topology_idempotently() {
        let (service, work_repo, _entity_repo, _session_repo) = setup_workspace().await;
        let project = Project::new("Debug with AI").with_description(
            "Use https://github.com/acme/mono.git as the canonical project repository.",
        );
        work_repo.create_project(&project).await.unwrap();

        let dir = tempdir().unwrap();
        let export = service
            .export_migration_review(dir.path(), RepositoryMigrationOptions::all())
            .await
            .unwrap();
        check_all_repository_candidates(dir.path(), &export, "Accept repository record");

        let apply = service
            .apply_migration_review(
                dir.path(),
                RepositoryMigrationReviewApplyOptions {
                    dry_run: false,
                    writer: Some(migration_writer()),
                    create_commit: true,
                },
            )
            .await
            .unwrap();

        assert_eq!(apply.planned_count(), export.inventory.returned_candidates);
        assert_eq!(apply.written_count(), export.inventory.returned_candidates);
        let commit = apply.commit.as_ref().expect("write should create commit");
        assert_eq!(commit.change_count(), apply.written_count());
        assert!(service
            .memory_repo
            .get_knowledge_commit(&commit.id)
            .await
            .unwrap()
            .is_some());
        assert!(service
            .list_repositories(None)
            .await
            .unwrap()
            .iter()
            .any(|repository| repository.remote_url.as_deref()
                == Some("https://github.com/acme/mono.git")));

        let repository = service.list_repositories(None).await.unwrap()[0].clone();
        let links = service
            .repo
            .list_project_links(&repository.id)
            .await
            .unwrap();
        assert_eq!(links[0].project_name, "Debug with AI");

        let second_apply = service
            .apply_migration_review(
                dir.path(),
                RepositoryMigrationReviewApplyOptions {
                    dry_run: false,
                    writer: Some(migration_writer()),
                    create_commit: true,
                },
            )
            .await
            .unwrap();
        assert_eq!(
            second_apply.planned_count(),
            export.inventory.returned_candidates
        );
        assert!(second_apply.existing_record_count >= 1);
        assert_eq!(service.list_repositories(None).await.unwrap().len(), 1);
    }

    #[test]
    fn repository_remote_extraction_rejects_non_git_urls() {
        let remotes = extract_remote_references(
            "Use https://github.com/ymeiri/engram and \
             git@github.com:DataDog/dd-source.git. Also parse \
             https://github.com/ymeiri/engram.git\\ncd safely. Ignore \
             https://docs.google.com/document/d/abc, \
             https://datadoghq.atlassian.net/wiki/spaces/IDEAI, \
             https://app.datadoghq.com/dashboard/bzc-4ty-2hn.",
        );

        let normalized = remotes
            .iter()
            .map(|remote| remote.normalized.as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            normalized,
            vec!["github.com/ymeiri/engram", "github.com/DataDog/dd-source"]
        );
    }

    #[test]
    fn local_path_candidates_use_git_root_and_quarantine_non_repos() {
        let repo_dir = tempdir().unwrap();
        run_git(repo_dir.path(), &["init"]);
        let src_dir = repo_dir.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        let source_file = src_dir.join("main.rs");
        std::fs::write(&source_file, "fn main() {}\n").unwrap();
        let evidence = repository_source(
            RepositoryMigrationSourceKind::ProjectObservation,
            "source-1".to_string(),
            "project:engram observation".to_string(),
            Some("engram".to_string()),
            OffsetDateTime::now_utc(),
            OffsetDateTime::now_utc(),
        );

        let git_candidate =
            candidate_from_local_path(source_file.display().to_string(), evidence.clone(), &[]);
        let expected_root = canonical_or_original(repo_dir.path()).display().to_string();
        assert_eq!(
            git_candidate.local_path.as_deref(),
            Some(expected_root.as_str())
        );
        assert_eq!(git_candidate.component_path.as_deref(), Some("src/main.rs"));
        assert_eq!(
            git_candidate.disposition,
            RepositoryMigrationDisposition::Review
        );

        let notes_dir = tempdir().unwrap();
        let note = notes_dir.path().join("handoff.md");
        std::fs::write(&note, "# Handoff\n").unwrap();
        let non_repo_candidate =
            candidate_from_local_path(note.display().to_string(), evidence, &[]);
        assert_eq!(
            non_repo_candidate.disposition,
            RepositoryMigrationDisposition::Quarantine
        );
    }

    fn check_first_repository_candidate(
        root: &Path,
        export: &RepositoryMigrationReviewExport,
        decision: &str,
    ) {
        let candidate_path = first_repository_candidate_path(export);
        check_repository_candidate(root, &candidate_path, decision);
    }

    fn check_all_repository_candidates(
        root: &Path,
        export: &RepositoryMigrationReviewExport,
        decision: &str,
    ) {
        for candidate_path in repository_candidate_paths(export) {
            check_repository_candidate(root, &candidate_path, decision);
        }
    }

    fn first_repository_candidate_path(export: &RepositoryMigrationReviewExport) -> String {
        repository_candidate_paths(export)
            .into_iter()
            .next()
            .unwrap()
    }

    fn repository_candidate_paths(export: &RepositoryMigrationReviewExport) -> Vec<String> {
        let mut paths = export
            .files_written
            .iter()
            .filter(|path| path.starts_with("candidates/"))
            .cloned()
            .collect::<Vec<_>>();
        paths.sort();
        paths
    }

    fn check_repository_candidate(root: &Path, candidate_path: &str, decision: &str) {
        check_repository_candidate_at_path(&root.join(candidate_path), decision);
    }

    fn check_repository_candidate_at_path(path: &Path, decision: &str) {
        let contents = std::fs::read_to_string(path).unwrap();
        let checked = contents.replace(&format!("- [ ] {decision}"), &format!("- [x] {decision}"));
        std::fs::write(path, checked).unwrap();
    }
}
