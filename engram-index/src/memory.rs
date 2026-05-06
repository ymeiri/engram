//! Memory OS service.
//!
//! Provides the first service layer over source-grounded memory items and
//! knowledge commits. MCP and CLI surfaces should delegate to this layer.

use crate::digest::{
    apply_digest_extraction_review_batch, build_digest_extraction_commit,
    DigestExtractionReviewApply, DigestExtractionReviewApplyOptions,
};
use crate::error::{IndexError, IndexResult};
use crate::memory_ranker::{rank_memory_item, rank_memory_items, MemoryRankContext};
use crate::migration::{
    MigrationInventory, MigrationInventoryOptions, MigrationReviewApply,
    MigrationReviewApplyOptions, MigrationReviewExport, MigrationReviewStatus, MigrationService,
};
use crate::repository::refresh_checkout_git_state;
use crate::vault::{
    init_memory_vault, inspect_memory_vault, read_memory_vault_page, write_memory_vault,
    MemoryVaultExport, MemoryVaultInit, MemoryVaultPage, MemoryVaultStatus,
    RepositoryVaultSnapshot,
};
use engram_core::id::Id;
use engram_core::memory::{
    ClaimOrigin, EvidenceKind, EvidenceRef, Harness, KnowledgeCommit, MemoryChange, MemoryCursor,
    MemoryItem, MemoryKind, MemoryScope, MemoryStatus, MemoryTrustMetadata, WriterProvenance,
};
use engram_core::repository::{MonorepoComponent, ProjectRepositoryLink, RepositoryContext};
use engram_core::session::{Event, EventType};
use engram_core::telemetry::{BrainHarnessIntent, BrainHarnessOperation, BrainHarnessTrace};
use engram_store::{Db, MemoryRepo, RepositoryRepo, SessionRepo, TelemetryRepo};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use std::time::Instant;
use time::OffsetDateTime;
use tracing::info;

/// Relevance score for a memory item returned by changes_since.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryChangeRelevance {
    /// Memory item ID.
    pub item_id: Id,
    /// Deterministic score.
    pub score: f32,
    /// Reasons contributing to the score.
    pub reasons: Vec<String>,
}

/// Options for changes_since filtering and relevance scoring.
#[derive(Debug, Clone, Default)]
pub struct MemoryChangesSinceOptions {
    /// Optional writer harness filter.
    pub writer_harness: Option<String>,
    /// Optional model filter.
    pub model: Option<String>,
    /// Optional surface filter.
    pub surface: Option<String>,
    /// Optional writer session filter.
    pub writer_session_id: Option<Id>,
    /// Optional project used for relevance scoring.
    pub project: Option<String>,
    /// Optional cwd used for repository relevance scoring.
    pub cwd: Option<String>,
    /// Optional prompt/query used for keyword scoring.
    pub query: Option<String>,
    /// Caller intent for telemetry correlation.
    pub intent: Option<BrainHarnessIntent>,
}

/// Memory changes visible after a cursor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryChanges {
    /// Cursor used for the query.
    pub since: MemoryCursor,
    /// Cursor to use for the next poll.
    pub next_cursor: MemoryCursor,
    /// Trace ID for later telemetry feedback.
    pub trace_id: Option<Id>,
    /// Memory items updated after the cursor.
    pub items: Vec<MemoryItem>,
    /// Knowledge commits created after the cursor.
    pub commits: Vec<KnowledgeCommit>,
    /// Relevance scores for returned memory items.
    pub item_relevance: Vec<MemoryChangeRelevance>,
}

/// Dry-run session distillation candidate generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDistillation {
    /// Session distilled.
    pub session_id: Id,
    /// Candidates generated for review.
    pub candidates: Vec<MemoryItem>,
    /// Warning explaining that candidates are not durable writes.
    pub warning: String,
}

/// Aggregate memory count by writer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryWriterStat {
    /// Harness.
    pub harness: String,
    /// Model provider.
    pub model_provider: String,
    /// Model.
    pub model: String,
    /// Surface.
    pub surface: Option<String>,
    /// Item count.
    pub count: usize,
}

impl MemoryChanges {
    /// Whether any changes were returned.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty() && self.commits.is_empty()
    }

    /// Total number of changed records.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len() + self.commits.len()
    }
}

/// Input for building an orientation context packet.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrientInput {
    /// Current working directory, when known.
    pub cwd: Option<String>,
    /// User prompt that triggered orientation.
    pub prompt: Option<String>,
    /// Explicit project name, when known.
    pub project: Option<String>,
    /// Agent/harness name.
    pub agent: Option<String>,
    /// Caller intent for telemetry correlation.
    pub intent: Option<BrainHarnessIntent>,
    /// Include recent knowledge commits.
    pub include_recent_commits: bool,
    /// Maximum memory items per grouped bucket.
    pub limit: Option<usize>,
}

/// Project resolution source for orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrientationResolutionSource {
    /// User supplied the project explicitly.
    ExplicitProject,
    /// Resolved from a component-scoped repository link.
    ComponentLink,
    /// Resolved from an unambiguous repository link.
    RepositoryLink,
    /// No project was selected.
    Unresolved,
}

/// Structured project/repository resolution for an orientation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrientationResolution {
    /// Project supplied by the caller.
    pub explicit_project: Option<String>,
    /// Project selected for retrieval, if any.
    pub selected_project: Option<String>,
    /// Source of the selected project.
    pub source: OrientationResolutionSource,
    /// Confidence score from 0.0 to 1.0.
    pub confidence: f32,
    /// Whether the caller should confirm before relying on the selected/candidate project.
    pub requires_confirmation: bool,
    /// Human-readable reason for the resolution.
    pub reason: String,
    /// Repository matched from cwd, if any.
    pub repository_name: Option<String>,
    /// Component names matched from cwd.
    pub component_names: Vec<String>,
    /// Project candidates considered.
    pub project_candidates: Vec<String>,
    /// Ambiguity details, if any.
    pub ambiguity: Option<String>,
}

impl OrientationResolution {
    fn unresolved(
        explicit_project: Option<&str>,
        repository_context: Option<&RepositoryContext>,
        reason: impl Into<String>,
        ambiguity: Option<String>,
    ) -> Self {
        Self {
            explicit_project: explicit_project.map(str::to_string),
            selected_project: None,
            source: OrientationResolutionSource::Unresolved,
            confidence: 0.0,
            requires_confirmation: ambiguity.is_some(),
            reason: reason.into(),
            repository_name: repository_context.map(|context| context.repository.name.clone()),
            component_names: component_names(repository_context),
            project_candidates: project_candidates(repository_context),
            ambiguity,
        }
    }
}

/// Orientation context packet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrientationPacket {
    /// Explicit project name, when supplied.
    pub project: Option<String>,
    /// Current working directory, when supplied.
    pub cwd: Option<String>,
    /// Agent/harness name.
    pub agent: Option<String>,
    /// Caller intent, when supplied.
    pub intent: Option<BrainHarnessIntent>,
    /// Trace ID for later telemetry feedback.
    pub trace_id: Option<Id>,
    /// Prompt that triggered orientation.
    pub prompt: Option<String>,
    /// Human-readable scope label.
    pub scope: String,
    /// Structured resolution explaining which project/repository context was selected.
    pub resolution: OrientationResolution,
    /// Repository topology resolved from cwd, when known.
    pub repository_context: Option<RepositoryContext>,
    /// Memory cursor for later changes_since checks.
    pub memory_cursor: MemoryCursor,
    /// Markdown context pack.
    pub context_pack: String,
    /// Active decisions relevant to this scope.
    pub active_decisions: Vec<MemoryItem>,
    /// Active rules relevant to this scope.
    pub active_rules: Vec<MemoryItem>,
    /// Active preferences relevant to this scope.
    pub preferences: Vec<MemoryItem>,
    /// Active limitations relevant to this scope.
    pub limitations: Vec<MemoryItem>,
    /// Review-needed memory relevant to this scope.
    pub review_needed: Vec<MemoryItem>,
    /// Trust metadata for memory returned in the orientation packet.
    pub memory_metadata: Vec<MemoryTrustMetadata>,
    /// Recent knowledge commits, if requested.
    pub recent_knowledge_commits: Vec<KnowledgeCommit>,
    /// Recommended next actions for the caller.
    pub recommended_actions: Vec<String>,
    /// Ambiguities the agent should not silently ignore.
    pub ambiguities: Vec<String>,
}

/// Service for Memory OS persistence and query behavior.
#[derive(Clone)]
pub struct MemoryService {
    repo: MemoryRepo,
    telemetry_repo: TelemetryRepo,
    repository_repo: RepositoryRepo,
    session_repo: SessionRepo,
    migration_service: MigrationService,
}

impl MemoryService {
    /// Create a new memory service.
    pub fn new(db: Db) -> Self {
        Self {
            repo: MemoryRepo::new(db.clone()),
            telemetry_repo: TelemetryRepo::new(db.clone()),
            repository_repo: RepositoryRepo::new(db.clone()),
            session_repo: SessionRepo::new(db.clone()),
            migration_service: MigrationService::new(db),
        }
    }

    /// Initialize Memory OS schema.
    pub async fn init_schema(&self) -> IndexResult<()> {
        self.repo.init_schema().await?;
        self.telemetry_repo.init_schema().await?;
        self.repository_repo.init_schema().await?;
        self.session_repo.init_schema().await?;
        Ok(())
    }

    /// Persist a memory item after basic domain validation.
    pub async fn capture_memory(&self, item: MemoryItem) -> IndexResult<MemoryItem> {
        validate_memory_item(&item)?;
        self.repo.save_memory_item(&item).await?;
        Ok(item)
    }

    /// Get a memory item by ID.
    pub async fn get_memory(&self, id: &Id) -> IndexResult<Option<MemoryItem>> {
        Ok(self.repo.get_memory_item(id).await?)
    }

    /// Archive a memory item with metadata.
    pub async fn archive_memory(
        &self,
        id: &Id,
        reason: impl Into<String>,
        archived_by: Option<String>,
    ) -> IndexResult<MemoryItem> {
        let item = self
            .repo
            .get_memory_item(id)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("memory item not found: {id}")))?;
        let item = item.with_archive(reason, archived_by);
        self.repo.save_memory_item(&item).await?;
        Ok(item)
    }

    /// List memory items.
    pub async fn list_memory(
        &self,
        status: Option<MemoryStatus>,
        limit: Option<usize>,
    ) -> IndexResult<Vec<MemoryItem>> {
        Ok(self.repo.list_memory_items(status, limit).await?)
    }

    /// List items eligible for normal retrieval.
    pub async fn list_active_memory(&self, limit: Option<usize>) -> IndexResult<Vec<MemoryItem>> {
        self.list_memory(Some(MemoryStatus::Active), limit).await
    }

    /// List items requiring human or agent review.
    pub async fn list_memory_needing_review(
        &self,
        limit: Option<usize>,
    ) -> IndexResult<Vec<MemoryItem>> {
        Ok(self
            .repo
            .list_memory_items_needing_review(OffsetDateTime::now_utc(), limit)
            .await?)
    }

    /// Persist an already-built knowledge commit.
    pub async fn save_commit(&self, commit: KnowledgeCommit) -> IndexResult<KnowledgeCommit> {
        validate_knowledge_commit(&commit)?;
        self.repo.save_knowledge_commit(&commit).await?;
        Ok(commit)
    }

    /// Build and save a knowledge commit.
    pub async fn commit_changes(
        &self,
        writer: WriterProvenance,
        message: impl Into<String>,
        changes: Vec<MemoryChange>,
        session_id: Option<Id>,
        parent_id: Option<Id>,
    ) -> IndexResult<KnowledgeCommit> {
        let mut commit = KnowledgeCommit::new(writer, message);
        if let Some(session_id) = session_id {
            commit = commit.with_session(session_id);
        }
        if let Some(parent_id) = parent_id {
            commit = commit.with_parent(parent_id);
        }
        for change in changes {
            commit = commit.with_change(change);
        }

        self.save_commit(commit).await
    }

    /// Get a knowledge commit by ID.
    pub async fn get_commit(&self, id: &Id) -> IndexResult<Option<KnowledgeCommit>> {
        Ok(self.repo.get_knowledge_commit(id).await?)
    }

    /// List knowledge commits.
    pub async fn list_commits(&self, limit: Option<usize>) -> IndexResult<Vec<KnowledgeCommit>> {
        Ok(self.repo.list_knowledge_commits(limit).await?)
    }

    /// Aggregate memory records by writer provenance.
    pub async fn writer_stats(&self) -> IndexResult<Vec<MemoryWriterStat>> {
        let mut stats: std::collections::BTreeMap<(String, String, String, Option<String>), usize> =
            std::collections::BTreeMap::new();
        for item in self.repo.list_memory_items(None, None).await? {
            *stats
                .entry((
                    item.writer.harness.to_string(),
                    item.writer.model.provider,
                    item.writer.model.model,
                    item.writer.surface,
                ))
                .or_default() += 1;
        }
        Ok(stats
            .into_iter()
            .map(
                |((harness, model_provider, model, surface), count)| MemoryWriterStat {
                    harness,
                    model_provider,
                    model,
                    surface,
                    count,
                },
            )
            .collect())
    }

    /// Export Memory OS records into an Obsidian-compatible Markdown vault.
    ///
    /// Existing files without the Engram generated marker are left untouched.
    pub async fn export_vault(&self, root: impl AsRef<Path>) -> IndexResult<MemoryVaultExport> {
        let items = self.list_memory(None, None).await?;
        let commits = self.list_commits(None).await?;
        let repositories = self.repository_vault_snapshots().await?;
        write_memory_vault(root.as_ref(), &items, &commits, &repositories)
    }

    /// Create the Memory OS vault directory skeleton.
    pub async fn init_vault(&self, root: impl AsRef<Path>) -> IndexResult<MemoryVaultInit> {
        init_memory_vault(root.as_ref())
    }

    /// Inspect the current vault state without writing files.
    pub async fn vault_status(&self, root: impl AsRef<Path>) -> IndexResult<MemoryVaultStatus> {
        let items = self.list_memory(None, None).await?;
        let commits = self.list_commits(None).await?;
        let repositories = self.repository_vault_snapshots().await?;
        inspect_memory_vault(root.as_ref(), &items, &commits, &repositories)
    }

    /// Read a generated or user-authored page from the vault.
    pub async fn vault_page(
        &self,
        root: impl AsRef<Path>,
        page: &str,
    ) -> IndexResult<Option<MemoryVaultPage>> {
        read_memory_vault_page(root.as_ref(), page)
    }

    /// Build a non-destructive inventory of existing Engram data for future migration.
    pub async fn migration_inventory(
        &self,
        options: MigrationInventoryOptions,
    ) -> IndexResult<MigrationInventory> {
        self.migration_service.inventory(options).await
    }

    /// Export a non-destructive Markdown review batch for migration candidates.
    pub async fn export_migration_review(
        &self,
        root: impl AsRef<Path>,
        options: MigrationInventoryOptions,
    ) -> IndexResult<MigrationReviewExport> {
        self.migration_service
            .export_review_batch(root.as_ref(), options)
            .await
    }

    /// Parse a generated migration review batch and report readiness without writing records.
    pub async fn migration_review_status(
        &self,
        root: impl AsRef<Path>,
    ) -> IndexResult<MigrationReviewStatus> {
        self.migration_service
            .review_batch_status(root.as_ref())
            .await
    }

    /// Apply a reviewed migration batch. Dry-run mode reports planned writes only.
    pub async fn apply_migration_review(
        &self,
        root: impl AsRef<Path>,
        options: MigrationReviewApplyOptions,
    ) -> IndexResult<MigrationReviewApply> {
        self.migration_service
            .apply_review_batch(root.as_ref(), options)
            .await
    }

    /// Apply a reviewed digest extraction batch. Dry-run mode reports planned writes only.
    pub async fn apply_digest_extraction_review(
        &self,
        root: impl AsRef<Path>,
        options: DigestExtractionReviewApplyOptions,
    ) -> IndexResult<DigestExtractionReviewApply> {
        let existing_candidate_tags = self.existing_digest_extraction_candidate_tags().await?;
        let mut report = apply_digest_extraction_review_batch(
            root.as_ref(),
            options.clone(),
            existing_candidate_tags,
        )?;

        if !options.dry_run {
            for item in report.planned_items.clone() {
                let item = self.capture_memory(item).await?;
                report.written_items.push(item);
            }
            if options.create_commit && !report.written_items.is_empty() {
                let commit = build_digest_extraction_commit(&options.writer, &report.written_items);
                self.save_commit(commit.clone()).await?;
                report.commit = Some(commit);
            }
        }

        Ok(report)
    }

    /// Generate review candidates from a session event stream. This does not
    /// persist memory; callers must export/review/apply accepted candidates and
    /// create a knowledge commit separately.
    pub async fn distill_session(
        &self,
        session_id: Id,
        writer: WriterProvenance,
    ) -> IndexResult<SessionDistillation> {
        let events = self.session_repo.get_events(&session_id).await?;
        let candidates = events
            .into_iter()
            .filter_map(|event| distill_event_candidate(event, writer.clone()))
            .collect();
        Ok(SessionDistillation {
            session_id,
            candidates,
            warning: "Dry-run candidates only; durable writes require accepted review decisions and a knowledge commit.".to_string(),
        })
    }

    /// Create a cursor for the current point in time.
    pub async fn current_cursor(&self) -> IndexResult<MemoryCursor> {
        let latest_commit_id = self
            .repo
            .latest_knowledge_commit()
            .await?
            .map(|commit| commit.id);
        Ok(MemoryCursor::now(latest_commit_id))
    }

    /// Return memory and commit changes after a cursor.
    pub async fn changes_since(
        &self,
        cursor: MemoryCursor,
        limit: Option<usize>,
    ) -> IndexResult<MemoryChanges> {
        self.changes_since_with_options(cursor, limit, MemoryChangesSinceOptions::default())
            .await
    }

    /// Return filtered memory and commit changes after a cursor.
    pub async fn changes_since_with_options(
        &self,
        cursor: MemoryCursor,
        limit: Option<usize>,
        options: MemoryChangesSinceOptions,
    ) -> IndexResult<MemoryChanges> {
        let started = Instant::now();
        let mut items = self
            .repo
            .list_memory_items_updated_after(cursor.timestamp, None)
            .await?
            .into_iter()
            .filter(|item| matches_changes_since_filters(item, &options))
            .collect::<Vec<_>>();
        let scores = score_changes_since_items(&items, &options);
        items.sort_by(|left, right| {
            change_relevance_score(&scores, right.id)
                .partial_cmp(&change_relevance_score(&scores, left.id))
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| right.updated_at.cmp(&left.updated_at))
        });
        if let Some(limit) = limit {
            items.truncate(limit);
        }
        let item_relevance = score_changes_since_items(&items, &options);
        let commits = self
            .repo
            .list_knowledge_commits_after(cursor.timestamp, limit)
            .await?;

        let mut next_timestamp = cursor.timestamp;
        for item in &items {
            next_timestamp = next_timestamp.max(item.updated_at);
        }
        for commit in &commits {
            next_timestamp = next_timestamp.max(commit.created_at);
        }

        let next_commit_id = commits.last().map(|commit| commit.id).or(cursor.commit_id);

        info!(
            "Memory changes_since returned {} items and {} commits",
            items.len(),
            commits.len()
        );

        let returned_memory_ids = items.iter().map(|item| item.id).collect::<Vec<_>>();
        let trace = BrainHarnessTrace::new(BrainHarnessOperation::ChangesSince)
            .with_intent(options.intent.clone())
            .with_query(options.query.clone())
            .with_project(options.project.clone())
            .with_returned_memory_ids(returned_memory_ids.clone())
            .with_returned_result_ids(
                returned_memory_ids
                    .iter()
                    .map(std::string::ToString::to_string)
                    .chain(commits.iter().map(|commit| commit.id.to_string()))
                    .collect(),
            )
            .with_latency_ms(started.elapsed().as_millis() as u64);
        self.telemetry_repo.save_trace(&trace).await?;

        Ok(MemoryChanges {
            since: cursor,
            next_cursor: MemoryCursor {
                commit_id: next_commit_id,
                timestamp: next_timestamp,
            },
            trace_id: Some(trace.id),
            items,
            commits,
            item_relevance,
        })
    }

    async fn existing_digest_extraction_candidate_tags(&self) -> IndexResult<HashSet<String>> {
        let items = self.repo.list_memory_items(None, None).await?;
        Ok(items
            .into_iter()
            .flat_map(|item| item.tags.into_iter())
            .filter(|tag| tag.starts_with("digest-extraction-candidate:"))
            .collect())
    }

    /// Build the first-version orientation context packet.
    pub async fn orient(&self, input: OrientInput) -> IndexResult<OrientationPacket> {
        let started = Instant::now();
        let limit = input.limit.unwrap_or(20);
        let cursor = self.current_cursor().await?;
        let active = self.list_active_memory(None).await?;
        let review = self.list_memory_needing_review(Some(limit)).await?;
        let repository_context = self
            .resolve_repository_context(input.cwd.as_deref())
            .await?;
        let recent_commits = if input.include_recent_commits {
            self.list_commits(Some(limit)).await?
        } else {
            Vec::new()
        };
        let resolution = resolve_orientation_project(
            input.project.as_deref(),
            input.cwd.as_deref(),
            repository_context.as_ref(),
        );
        let effective_project = resolution.selected_project.as_deref();

        let relevant_active = filter_relevant(
            active,
            effective_project,
            input.cwd.as_deref(),
            input.prompt.as_deref(),
        );
        let mut relevant_review = filter_relevant(
            review,
            effective_project,
            input.cwd.as_deref(),
            input.prompt.as_deref(),
        );
        relevant_review.truncate(limit);

        let active_decisions = take_kind(&relevant_active, MemoryKind::Decision, limit);
        let active_rules = take_kind(&relevant_active, MemoryKind::Rule, limit);
        let preferences = take_kind(&relevant_active, MemoryKind::Preference, limit);
        let limitations = take_kind(&relevant_active, MemoryKind::Limitation, limit);

        let mut ambiguities = Vec::new();
        if let Some(ambiguity) = &resolution.ambiguity {
            ambiguities.push(ambiguity.clone());
        } else if input.cwd.is_some() && repository_context.is_none() {
            ambiguities.push(
                "cwd did not match a registered repository checkout; run repo detect/register if this workspace should be part of Memory OS orientation.".to_string(),
            );
        }
        if relevant_active.is_empty() {
            ambiguities.push("No active memory matched this orientation scope.".to_string());
        }

        let mut recommended_actions = vec![
            "Use the returned memory_cursor with memory changes_since during long sessions."
                .to_string(),
        ];
        if !relevant_review.is_empty() {
            recommended_actions.push(
                "Review needs_review memory before treating it as active context.".to_string(),
            );
        }
        if resolution.requires_confirmation {
            recommended_actions.push(
                "Ask the user to confirm the intended project before using project-scoped memory."
                    .to_string(),
            );
        }

        let scope = scope_label(effective_project, input.cwd.as_deref());
        let context_pack = build_context_pack(ContextPackParts {
            scope: &scope,
            cursor: &cursor,
            resolution: &resolution,
            repository_context: repository_context.as_ref(),
            decisions: &active_decisions,
            rules: &active_rules,
            preferences: &preferences,
            limitations: &limitations,
            review_needed: &relevant_review,
            commits: &recent_commits,
            ambiguities: &ambiguities,
            recommended_actions: &recommended_actions,
        });

        let returned_memory_ids = returned_orientation_memory_ids(&[
            &active_decisions,
            &active_rules,
            &preferences,
            &limitations,
            &relevant_review,
        ]);
        let memory_metadata = orientation_memory_metadata(&[
            &active_decisions,
            &active_rules,
            &preferences,
            &limitations,
            &relevant_review,
        ]);
        let trace = BrainHarnessTrace::new(BrainHarnessOperation::Orient)
            .with_agent(input.agent.clone())
            .with_intent(input.intent.clone())
            .with_query(input.prompt.clone())
            .with_project(effective_project.map(str::to_string))
            .with_returned_memory_ids(returned_memory_ids.clone())
            .with_returned_result_ids(
                returned_memory_ids
                    .iter()
                    .map(std::string::ToString::to_string)
                    .collect(),
            )
            .with_latency_ms(started.elapsed().as_millis() as u64);
        self.telemetry_repo.save_trace(&trace).await?;

        Ok(OrientationPacket {
            project: input.project,
            cwd: input.cwd,
            agent: input.agent,
            intent: input.intent,
            trace_id: Some(trace.id),
            prompt: input.prompt,
            scope,
            resolution,
            repository_context,
            memory_cursor: cursor,
            context_pack,
            active_decisions,
            active_rules,
            preferences,
            limitations,
            review_needed: relevant_review,
            memory_metadata,
            recent_knowledge_commits: recent_commits,
            recommended_actions,
            ambiguities,
        })
    }

    async fn resolve_repository_context(
        &self,
        cwd: Option<&str>,
    ) -> IndexResult<Option<RepositoryContext>> {
        let Some(cwd) = cwd else {
            return Ok(None);
        };
        let cwd_path = Path::new(cwd)
            .canonicalize()
            .unwrap_or_else(|_| Path::new(cwd).to_path_buf());
        let checkouts = self.repository_repo.list_checkouts().await?;
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
            self.repository_repo.save_checkout(&checkout).await?;
        }
        let Some(repository_id) = checkout.repository_id else {
            return Ok(None);
        };
        let Some(repository) = self.repository_repo.get_repository(&repository_id).await? else {
            return Ok(None);
        };

        let components = self.repository_repo.list_components(&repository.id).await?;
        let checkout_path = canonical_or_original(Path::new(&checkout.local_path));
        let matching_components = matching_components(&cwd_path, &checkout_path, components);
        let linked_projects = self
            .repository_repo
            .list_project_links(&repository.id)
            .await?;

        Ok(Some(RepositoryContext {
            repository,
            checkout: Some(checkout),
            matching_components,
            linked_projects,
        }))
    }

    async fn repository_vault_snapshots(&self) -> IndexResult<Vec<RepositoryVaultSnapshot>> {
        let repositories = self.repository_repo.list_repositories(None).await?;
        let checkouts = self.repository_repo.list_checkouts().await?;
        let mut snapshots = Vec::with_capacity(repositories.len());

        for repository in repositories {
            let repository_checkouts = checkouts
                .iter()
                .filter(|checkout| checkout.repository_id == Some(repository.id))
                .cloned()
                .collect();
            let components = self.repository_repo.list_components(&repository.id).await?;
            let project_links = self
                .repository_repo
                .list_project_links(&repository.id)
                .await?;

            snapshots.push(RepositoryVaultSnapshot {
                repository,
                checkouts: repository_checkouts,
                components,
                project_links,
            });
        }

        Ok(snapshots)
    }
}

fn filter_relevant(
    items: Vec<MemoryItem>,
    project: Option<&str>,
    cwd: Option<&str>,
    query: Option<&str>,
) -> Vec<MemoryItem> {
    rank_memory_items(items, MemoryRankContext::orientation(project, cwd, query))
        .into_iter()
        .map(|ranked| ranked.item)
        .collect()
}

fn resolve_orientation_project(
    explicit_project: Option<&str>,
    cwd: Option<&str>,
    repository_context: Option<&RepositoryContext>,
) -> OrientationResolution {
    if let Some(project) = explicit_project.filter(|project| !project.trim().is_empty()) {
        let candidates = project_candidates(repository_context);
        let ambiguity = match repository_context {
            Some(context)
                if !candidates.is_empty()
                    && !candidates
                        .iter()
                        .any(|candidate| candidate.eq_ignore_ascii_case(project)) =>
            {
                Some(format!(
                    "Explicit project '{}' overrides repository '{}' candidate(s): {}.",
                    project,
                    context.repository.name,
                    candidates.join(", ")
                ))
            }
            _ => None,
        };
        return OrientationResolution {
            explicit_project: Some(project.to_string()),
            selected_project: Some(project.to_string()),
            source: OrientationResolutionSource::ExplicitProject,
            confidence: 1.0,
            requires_confirmation: false,
            reason: "Project was supplied explicitly by the caller.".to_string(),
            repository_name: repository_context.map(|context| context.repository.name.clone()),
            component_names: component_names(repository_context),
            project_candidates: candidates,
            ambiguity,
        };
    }

    let Some(context) = repository_context else {
        let ambiguity = if cwd.is_some() {
            "No explicit project was supplied and cwd did not match a registered repository checkout; run repo detect/register if this workspace should be part of Memory OS orientation.".to_string()
        } else {
            "No project or cwd was supplied; orientation is limited to global/user memory."
                .to_string()
        };
        return OrientationResolution::unresolved(
            None,
            None,
            "No explicit project and no repository context were available.",
            Some(ambiguity),
        );
    };

    let candidate_links = candidate_links_for_context(context);
    let candidates = unique_project_names(candidate_links.iter().copied());
    if candidates.is_empty() {
        return OrientationResolution::unresolved(
            None,
            Some(context),
            format!(
                "Repository '{}' matched cwd, but no project links are registered.",
                context.repository.name
            ),
            Some(format!(
                "Repository '{}' has no linked project candidates.",
                context.repository.name
            )),
        );
    }

    if candidates.len() == 1 {
        let selected_project = candidates[0].clone();
        let source = if candidate_links
            .iter()
            .any(|link| link.component_path.is_some())
        {
            OrientationResolutionSource::ComponentLink
        } else {
            OrientationResolutionSource::RepositoryLink
        };
        let confidence = match source {
            OrientationResolutionSource::ComponentLink => 0.9,
            OrientationResolutionSource::RepositoryLink => 0.75,
            OrientationResolutionSource::ExplicitProject
            | OrientationResolutionSource::Unresolved => 0.0,
        };
        let reason = match source {
            OrientationResolutionSource::ComponentLink => format!(
                "cwd matched repository '{}' and component-scoped project link '{}'.",
                context.repository.name, selected_project
            ),
            OrientationResolutionSource::RepositoryLink => format!(
                "cwd matched repository '{}' with one linked project '{}'.",
                context.repository.name, selected_project
            ),
            OrientationResolutionSource::ExplicitProject
            | OrientationResolutionSource::Unresolved => String::new(),
        };
        return OrientationResolution {
            explicit_project: None,
            selected_project: Some(selected_project),
            source,
            confidence,
            requires_confirmation: false,
            reason,
            repository_name: Some(context.repository.name.clone()),
            component_names: component_names(Some(context)),
            project_candidates: candidates,
            ambiguity: None,
        };
    }

    OrientationResolution::unresolved(
        None,
        Some(context),
        format!(
            "Repository '{}' matched cwd, but multiple project candidates exist.",
            context.repository.name
        ),
        Some(format!(
            "cwd matches repository '{}' linked to multiple project candidates: {}.",
            context.repository.name,
            candidates.join(", ")
        )),
    )
}

fn candidate_links_for_context(context: &RepositoryContext) -> Vec<&ProjectRepositoryLink> {
    let matching_paths: Vec<_> = context
        .matching_components
        .iter()
        .map(|component| component.path.as_str())
        .collect();

    let component_links: Vec<_> = context
        .linked_projects
        .iter()
        .filter(|link| {
            link.component_path
                .as_deref()
                .is_some_and(|path| matching_paths.contains(&path))
        })
        .collect();
    if !component_links.is_empty() {
        return component_links;
    }

    let repo_links: Vec<_> = context
        .linked_projects
        .iter()
        .filter(|link| link.component_path.is_none())
        .collect();
    if !repo_links.is_empty() {
        return repo_links;
    }

    context.linked_projects.iter().collect()
}

fn unique_project_names<'a>(links: impl Iterator<Item = &'a ProjectRepositoryLink>) -> Vec<String> {
    let mut names: Vec<_> = links.map(|link| link.project_name.clone()).collect();
    names.sort();
    names.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
    names
}

fn project_candidates(repository_context: Option<&RepositoryContext>) -> Vec<String> {
    repository_context
        .map(|context| unique_project_names(context.linked_projects.iter()))
        .unwrap_or_default()
}

fn component_names(repository_context: Option<&RepositoryContext>) -> Vec<String> {
    repository_context
        .map(|context| {
            context
                .matching_components
                .iter()
                .map(|component| component.name.clone())
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
fn is_relevant(item: &MemoryItem, project: Option<&str>, cwd: Option<&str>) -> bool {
    crate::memory_ranker::memory_scope_matches(
        item,
        MemoryRankContext::orientation(project, cwd, None),
    )
}

fn matches_changes_since_filters(item: &MemoryItem, options: &MemoryChangesSinceOptions) -> bool {
    if let Some(harness) = &options.writer_harness {
        if item.writer.harness.to_string() != Harness::parse(harness).to_string() {
            return false;
        }
    }
    if let Some(model) = &options.model {
        if item.writer.model.model != *model {
            return false;
        }
    }
    if let Some(surface) = &options.surface {
        if item.writer.surface.as_deref() != Some(surface.as_str()) {
            return false;
        }
    }
    if let Some(session_id) = options.writer_session_id {
        if item.writer.session_id != Some(session_id) {
            return false;
        }
    }
    true
}

fn score_changes_since_items(
    items: &[MemoryItem],
    options: &MemoryChangesSinceOptions,
) -> Vec<MemoryChangeRelevance> {
    let context = MemoryRankContext::changes_since(
        options.project.as_deref(),
        options.cwd.as_deref(),
        options.query.as_deref(),
    );
    items
        .iter()
        .map(|item| {
            let mut reasons = Vec::new();
            let ranked = rank_memory_item(item.clone(), context);
            let score = ranked.as_ref().map(|ranked| ranked.score).unwrap_or(0.0);

            if ranked
                .as_ref()
                .is_some_and(|ranked| ranked.components.scope > 0.0)
            {
                reasons.push("scope_match".to_string());
            }
            if ranked
                .as_ref()
                .is_some_and(|ranked| ranked.components.text > 0.0)
            {
                reasons.push("keyword_match".to_string());
            }
            reasons.push("recency".to_string());
            if item.status == MemoryStatus::Active {
                reasons.push("active".to_string());
            }
            MemoryChangeRelevance {
                item_id: item.id,
                score,
                reasons,
            }
        })
        .collect()
}

fn change_relevance_score(scores: &[MemoryChangeRelevance], item_id: Id) -> f32 {
    scores
        .iter()
        .find(|score| score.item_id == item_id)
        .map(|score| score.score)
        .unwrap_or(0.0)
}

fn distill_event_candidate(event: Event, writer: WriterProvenance) -> Option<MemoryItem> {
    let (kind, title) = match event.event_type {
        EventType::Decision => (MemoryKind::Decision, "Session decision"),
        EventType::Preference => (MemoryKind::Preference, "Session preference"),
        EventType::Rule => (MemoryKind::Rule, "Session rule"),
        EventType::Limitation => (MemoryKind::Limitation, "Session limitation"),
        EventType::HandoffUpdate => (MemoryKind::Handoff, "Session handoff update"),
        EventType::Observation | EventType::Milestone => {
            (MemoryKind::SessionInsight, "Session insight")
        }
        _ => return None,
    };
    Some(
        MemoryItem::new(
            kind,
            title,
            event.content,
            MemoryScope::Session {
                session_id: event.session_id,
            },
            ClaimOrigin::GeneratedSummary,
            writer,
        )
        .with_evidence(
            EvidenceRef::new(EvidenceKind::SessionEvent, event.id.to_string())
                .with_summary("Generated from session distillation candidate"),
        )
        .with_status(MemoryStatus::NeedsReview)
        .with_tag("distillation-candidate"),
    )
}

fn take_kind(items: &[MemoryItem], kind: MemoryKind, limit: usize) -> Vec<MemoryItem> {
    items
        .iter()
        .filter(|item| item.kind == kind)
        .take(limit)
        .cloned()
        .collect()
}

fn scope_label(project: Option<&str>, cwd: Option<&str>) -> String {
    if let Some(project) = project {
        return project.to_string();
    }
    if let Some(cwd) = cwd {
        return format!("cwd:{cwd}");
    }
    "global".to_string()
}

struct ContextPackParts<'a> {
    scope: &'a str,
    cursor: &'a MemoryCursor,
    resolution: &'a OrientationResolution,
    repository_context: Option<&'a RepositoryContext>,
    decisions: &'a [MemoryItem],
    rules: &'a [MemoryItem],
    preferences: &'a [MemoryItem],
    limitations: &'a [MemoryItem],
    review_needed: &'a [MemoryItem],
    commits: &'a [KnowledgeCommit],
    ambiguities: &'a [String],
    recommended_actions: &'a [String],
}

fn build_context_pack(parts: ContextPackParts<'_>) -> String {
    let mut lines = vec![
        format!("# Context Pack: {}", parts.scope),
        String::new(),
        format!("- Memory cursor timestamp: {}", parts.cursor.timestamp),
    ];
    if let Some(commit_id) = parts.cursor.commit_id {
        lines.push(format!("- Latest knowledge commit: {commit_id}"));
    }
    append_resolution_section(&mut lines, parts.resolution);
    append_repository_section(&mut lines, parts.repository_context);
    append_memory_section(&mut lines, "Active Decisions", parts.decisions);
    append_memory_section(&mut lines, "Active Rules", parts.rules);
    append_memory_section(&mut lines, "Preferences", parts.preferences);
    append_memory_section(&mut lines, "Limitations", parts.limitations);
    append_memory_section(&mut lines, "Needs Review", parts.review_needed);

    lines.push(String::new());
    lines.push("## Recent Knowledge Commits".to_string());
    if parts.commits.is_empty() {
        lines.push("- None".to_string());
    } else {
        for commit in parts.commits {
            lines.push(format!("- {}: {}", commit.id, commit.message));
        }
    }

    append_string_section(&mut lines, "Recommended Actions", parts.recommended_actions);
    append_string_section(&mut lines, "Ambiguities", parts.ambiguities);
    lines.join("\n")
}

fn append_resolution_section(lines: &mut Vec<String>, resolution: &OrientationResolution) {
    lines.push(String::new());
    lines.push("## Orientation Resolution".to_string());
    lines.push(format!(
        "- Selected project: {}",
        resolution.selected_project.as_deref().unwrap_or("none")
    ));
    lines.push(format!("- Source: {:?}", resolution.source));
    lines.push(format!("- Confidence: {:.2}", resolution.confidence));
    lines.push(format!(
        "- Requires confirmation: {}",
        resolution.requires_confirmation
    ));
    lines.push(format!("- Reason: {}", resolution.reason));
    if !resolution.project_candidates.is_empty() {
        lines.push(format!(
            "- Project candidates: {}",
            resolution.project_candidates.join(", ")
        ));
    }
    if let Some(ambiguity) = &resolution.ambiguity {
        lines.push(format!("- Ambiguity: {ambiguity}"));
    }
}

fn append_repository_section(lines: &mut Vec<String>, context: Option<&RepositoryContext>) {
    lines.push(String::new());
    lines.push("## Repository Context".to_string());
    let Some(context) = context else {
        lines.push("- None".to_string());
        return;
    };

    lines.push(format!("- Repository: {}", context.repository.name));
    if let Some(remote_url) = &context.repository.remote_url {
        lines.push(format!("- Remote: {remote_url}"));
    }
    if let Some(checkout) = &context.checkout {
        lines.push(format!("- Local path: {}", checkout.local_path));
        if let Some(branch) = &checkout.current_branch {
            lines.push(format!("- Branch: {branch}"));
        }
        if let Some(is_dirty) = checkout.is_dirty {
            lines.push(format!("- Dirty: {is_dirty}"));
        }
    }
    if context.matching_components.is_empty() {
        lines.push("- Matching components: none".to_string());
    } else {
        let components = context
            .matching_components
            .iter()
            .map(|component| format!("{} ({})", component.name, component.path))
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!("- Matching components: {components}"));
    }
    if context.linked_projects.is_empty() {
        lines.push("- Linked projects: none".to_string());
    } else {
        let projects = context
            .linked_projects
            .iter()
            .map(|link| format!("{} ({})", link.project_name, link.role))
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!("- Linked projects: {projects}"));
    }
}

fn append_memory_section(lines: &mut Vec<String>, title: &str, items: &[MemoryItem]) {
    lines.push(String::new());
    lines.push(format!("## {title}"));
    if items.is_empty() {
        lines.push("- None".to_string());
    } else {
        for item in items {
            let metadata = item.trust_metadata();
            lines.push(format!("- {}: {}", item.title, item.content));
            lines.push(format!(
                "  - Trust: status={}, review_state={}, freshness={}, origin={}, confidence={:.2}, evidence_count={}, writer={}/{}",
                metadata.status,
                metadata.review_state,
                metadata.freshness,
                metadata.claim_origin,
                metadata.confidence,
                metadata.evidence_count,
                metadata.writer.harness,
                metadata.writer.model
            ));
        }
    }
}

fn returned_orientation_memory_ids(groups: &[&[MemoryItem]]) -> Vec<Id> {
    groups
        .iter()
        .flat_map(|items| items.iter().map(|item| item.id))
        .collect()
}

fn orientation_memory_metadata(groups: &[&[MemoryItem]]) -> Vec<MemoryTrustMetadata> {
    groups
        .iter()
        .flat_map(|items| items.iter().map(MemoryItem::trust_metadata))
        .collect()
}

fn append_string_section(lines: &mut Vec<String>, title: &str, items: &[String]) {
    lines.push(String::new());
    lines.push(format!("## {title}"));
    if items.is_empty() {
        lines.push("- None".to_string());
    } else {
        for item in items {
            lines.push(format!("- {item}"));
        }
    }
}

fn path_starts_with(path: &Path, prefix: &Path) -> bool {
    path == prefix || path.starts_with(prefix)
}

fn canonical_or_original(path: &Path) -> std::path::PathBuf {
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

fn validate_memory_item(item: &MemoryItem) -> IndexResult<()> {
    if item.title.trim().is_empty() {
        return Err(IndexError::Parse(
            "memory item title must not be empty".to_string(),
        ));
    }
    if item.content.trim().is_empty() {
        return Err(IndexError::Parse(
            "memory item content must not be empty".to_string(),
        ));
    }
    Ok(())
}

fn validate_knowledge_commit(commit: &KnowledgeCommit) -> IndexResult<()> {
    if commit.message.trim().is_empty() {
        return Err(IndexError::Parse(
            "knowledge commit message must not be empty".to_string(),
        ));
    }
    if commit.changes.is_empty() {
        return Err(IndexError::InvalidState(
            "knowledge commit must contain at least one change".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::digest::{DigestExtractionOptions, DigestInventoryOptions, DigestService};
    use engram_core::memory::{
        ClaimOrigin, EvidenceKind, EvidenceRef, Harness, MemoryChangeType, MemoryKind, MemoryScope,
        ModelIdentity,
    };
    use engram_core::repository::{
        GitRepository, LocalCheckout, MonorepoComponent, ProjectRepositoryLink,
        ProjectRepositoryRole,
    };
    use engram_store::RepositoryRepo;
    use std::process::Command;
    use tempfile::tempdir;

    async fn setup_service() -> MemoryService {
        let config = engram_store::StoreConfig::memory();
        let db = engram_store::connect_and_init(&config).await.unwrap();
        let service = MemoryService::new(db);
        service.init_schema().await.unwrap();
        service
    }

    async fn setup_service_with_repository_repo() -> (MemoryService, RepositoryRepo) {
        let config = engram_store::StoreConfig::memory();
        let db = engram_store::connect_and_init(&config).await.unwrap();
        let service = MemoryService::new(db.clone());
        service.init_schema().await.unwrap();
        let repo = RepositoryRepo::new(db);
        repo.init_schema().await.unwrap();
        (service, repo)
    }

    fn writer() -> WriterProvenance {
        WriterProvenance::agent(Harness::Codex, ModelIdentity::new("openai", "gpt-5.5"))
    }

    fn memory_item(title: &str) -> MemoryItem {
        MemoryItem::new(
            MemoryKind::Decision,
            title,
            "Use MemoryService as the service boundary for MCP and CLI surfaces.",
            MemoryScope::project("engram"),
            ClaimOrigin::UserStated,
            writer(),
        )
        .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "unit-test"))
    }

    fn project_memory_item(project: &str, title: &str) -> MemoryItem {
        MemoryItem::new(
            MemoryKind::Decision,
            title,
            format!("Project scoped memory for {project}."),
            MemoryScope::project(project),
            ClaimOrigin::UserStated,
            writer(),
        )
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

    fn git_stdout(cwd: &Path, args: &[&str]) -> String {
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
        String::from_utf8_lossy(&output.stdout).trim().to_string()
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
    async fn capture_and_get_memory() {
        let service = setup_service().await;
        let item = memory_item("Service boundary");

        let captured = service.capture_memory(item.clone()).await.unwrap();
        let retrieved = service.get_memory(&captured.id).await.unwrap().unwrap();

        assert_eq!(retrieved.id, item.id);
        assert_eq!(retrieved.title, "Service boundary");
    }

    #[tokio::test]
    async fn capture_rejects_empty_content() {
        let service = setup_service().await;
        let mut item = memory_item("Invalid");
        item.content.clear();

        let err = service.capture_memory(item).await.unwrap_err();
        assert!(matches!(err, IndexError::Parse(_)));
    }

    #[tokio::test]
    async fn list_active_and_review_memory() {
        let service = setup_service().await;
        let active = memory_item("Active");
        let review = memory_item("Needs review").with_status(MemoryStatus::NeedsReview);

        service.capture_memory(active.clone()).await.unwrap();
        service.capture_memory(review.clone()).await.unwrap();

        let active_items = service.list_active_memory(None).await.unwrap();
        assert_eq!(active_items.len(), 1);
        assert_eq!(active_items[0].id, active.id);

        let review_items = service.list_memory_needing_review(None).await.unwrap();
        assert_eq!(review_items.len(), 1);
        assert_eq!(review_items[0].id, review.id);
    }

    #[tokio::test]
    async fn commit_changes_and_query_changes_since_cursor() {
        let service = setup_service().await;
        let cursor = service.current_cursor().await.unwrap();
        let item = service
            .capture_memory(memory_item("Committed item"))
            .await
            .unwrap();

        let change = MemoryChange::new(
            MemoryChangeType::Added,
            "Committed item",
            "Captured a memory item.",
        )
        .with_item(item.id);
        let commit = service
            .commit_changes(
                writer(),
                "Capture committed item",
                vec![change],
                item.writer.session_id,
                None,
            )
            .await
            .unwrap();

        let changes = service.changes_since(cursor, None).await.unwrap();

        assert_eq!(commit.change_count(), 1);
        assert_eq!(changes.items.len(), 1);
        assert_eq!(changes.commits.len(), 1);
        assert_eq!(changes.next_cursor.commit_id, Some(commit.id));
        assert!(!changes.is_empty());
        assert_eq!(changes.len(), 2);
    }

    #[tokio::test]
    async fn commit_requires_at_least_one_change() {
        let service = setup_service().await;
        let err = service
            .commit_changes(writer(), "Empty commit", Vec::new(), None, None)
            .await
            .unwrap_err();

        assert!(matches!(err, IndexError::InvalidState(_)));
    }

    #[tokio::test]
    async fn apply_digest_extraction_review_writes_once_and_commits() {
        let service = setup_service().await;
        let dir = tempdir().unwrap();
        let review = tempdir().unwrap();
        let output = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("slack-digest/morning")).unwrap();
        std::fs::write(
            dir.path().join("slack-digest/morning/2026-04-26.md"),
            "accepted digest source with enough specific detail for a persisted memory item",
        )
        .unwrap();

        let export = DigestService::new()
            .export_review_batch(review.path(), DigestInventoryOptions::new(dir.path()))
            .unwrap();
        edit_digest_review_decision(
            review.path(),
            &export.files_written,
            "slack-digest",
            "accept",
            &[
                ("memory_kind", "project_fact"),
                ("scope_type", "project"),
                ("scope_name", "\"Engram\""),
                ("title", "\"Persisted digest fact\""),
            ],
        );
        let plan = DigestService::new()
            .plan_extraction(
                review.path(),
                output.path(),
                DigestExtractionOptions::default(),
            )
            .unwrap();
        edit_digest_extraction_decision(output.path(), &plan.candidates[0].review_path, "accept");

        let apply = service
            .apply_digest_extraction_review(
                output.path(),
                DigestExtractionReviewApplyOptions {
                    dry_run: false,
                    writer: writer(),
                    create_commit: true,
                },
            )
            .await
            .unwrap();

        assert_eq!(apply.accepted_count, 1);
        assert_eq!(apply.written_count(), 1);
        assert!(apply.commit.is_some());
        assert_eq!(service.list_active_memory(None).await.unwrap().len(), 1);
        assert_eq!(service.list_commits(None).await.unwrap().len(), 1);

        let second = service
            .apply_digest_extraction_review(
                output.path(),
                DigestExtractionReviewApplyOptions {
                    dry_run: false,
                    writer: writer(),
                    create_commit: true,
                },
            )
            .await
            .unwrap();
        assert_eq!(second.duplicate_count, 1);
        assert_eq!(second.written_count(), 0);
        assert_eq!(service.list_active_memory(None).await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn orient_groups_relevant_memory_and_returns_cursor() {
        let service = setup_service().await;
        service
            .capture_memory(memory_item("Project decision"))
            .await
            .unwrap();
        service
            .capture_memory(MemoryItem::new(
                MemoryKind::Preference,
                "Global preference",
                "Keep updates concise.",
                MemoryScope::User,
                ClaimOrigin::UserStated,
                writer(),
            ))
            .await
            .unwrap();
        service
            .capture_memory(memory_item("Different project").with_status(MemoryStatus::NeedsReview))
            .await
            .unwrap();

        let packet = service
            .orient(OrientInput {
                project: Some("engram".to_string()),
                cwd: Some("/Users/yuval.meiri/projects/engram".to_string()),
                agent: Some("codex".to_string()),
                intent: None,
                prompt: Some("continue".to_string()),
                include_recent_commits: false,
                limit: Some(10),
            })
            .await
            .unwrap();

        assert_eq!(packet.project.as_deref(), Some("engram"));
        assert_eq!(packet.active_decisions.len(), 1);
        assert_eq!(packet.preferences.len(), 1);
        assert!(packet.context_pack.contains("Project decision"));
        assert!(packet.context_pack.contains("Memory cursor timestamp"));
    }

    #[tokio::test]
    async fn orient_limit_applies_per_bucket_after_relevance_grouping() {
        let service = setup_service().await;
        service
            .capture_memory(memory_item("Older decision"))
            .await
            .unwrap();
        for index in 0..3 {
            service
                .capture_memory(MemoryItem::new(
                    MemoryKind::ProjectFact,
                    format!("Recent project fact {index}"),
                    "Recent project facts should not starve decision retrieval.",
                    MemoryScope::project("engram"),
                    ClaimOrigin::UserStated,
                    writer(),
                ))
                .await
                .unwrap();
        }

        let packet = service
            .orient(OrientInput {
                project: Some("engram".to_string()),
                limit: Some(1),
                ..OrientInput::default()
            })
            .await
            .unwrap();

        assert_eq!(packet.active_decisions.len(), 1);
        assert_eq!(packet.active_decisions[0].title, "Older decision");
    }

    #[test]
    fn repository_scope_relevance_uses_path_boundaries() {
        let item = MemoryItem::new(
            MemoryKind::RepositoryFact,
            "Repository path",
            "Repository memory should only match paths inside the checkout.",
            MemoryScope::repository(None, Some("/tmp/project".to_string())),
            ClaimOrigin::UserStated,
            writer(),
        );

        assert!(is_relevant(&item, None, Some("/tmp/project/services/api")));
        assert!(!is_relevant(
            &item,
            None,
            Some("/tmp/project-other/services/api")
        ));
    }

    #[tokio::test]
    async fn orient_reports_ambiguity_without_project_or_cwd() {
        let service = setup_service().await;
        let packet = service.orient(OrientInput::default()).await.unwrap();

        assert_eq!(packet.scope, "global");
        assert!(packet.resolution.selected_project.is_none());
        assert!(packet.resolution.requires_confirmation);
        assert!(packet
            .ambiguities
            .iter()
            .any(|item| item.contains("No project or cwd")));
    }

    #[tokio::test]
    async fn orient_uses_single_repo_project_candidate() {
        let (service, repo) = setup_service_with_repository_repo().await;
        let dir = tempdir().unwrap();
        let repository = GitRepository::new("debug-with-ai");
        repo.save_repository(&repository).await.unwrap();
        repo.save_checkout(
            &LocalCheckout::new(dir.path().display().to_string()).with_repository(repository.id),
        )
        .await
        .unwrap();
        repo.save_project_link(&ProjectRepositoryLink::new(
            "Debug with AI",
            repository.id,
            ProjectRepositoryRole::Primary,
        ))
        .await
        .unwrap();

        service
            .capture_memory(project_memory_item("Debug with AI", "Use repo candidate"))
            .await
            .unwrap();

        let packet = service
            .orient(OrientInput {
                cwd: Some(dir.path().display().to_string()),
                project: None,
                include_recent_commits: false,
                limit: Some(10),
                ..OrientInput::default()
            })
            .await
            .unwrap();

        assert_eq!(
            packet.resolution.selected_project.as_deref(),
            Some("Debug with AI")
        );
        assert_eq!(
            packet.resolution.source,
            OrientationResolutionSource::RepositoryLink
        );
        assert!(!packet.resolution.requires_confirmation);
        assert_eq!(packet.active_decisions[0].title, "Use repo candidate");
    }

    #[tokio::test]
    async fn orient_refreshes_repository_checkout_git_state() {
        if !git_available() {
            return;
        }
        let (service, repo) = setup_service_with_repository_repo().await;
        let dir = tempdir().unwrap();
        run_git(dir.path(), &["init"]);
        std::fs::write(dir.path().join("README.md"), "current\n").unwrap();
        commit_all(dir.path(), "current");
        let current_head = git_stdout(dir.path(), &["rev-parse", "HEAD"]);

        let repository = GitRepository::new("fresh-orient");
        repo.save_repository(&repository).await.unwrap();
        let mut checkout =
            LocalCheckout::new(dir.path().display().to_string()).with_repository(repository.id);
        checkout.update_detected_state(
            Some("stale-branch".to_string()),
            Some("stale-head".to_string()),
            Some(true),
        );
        let stale_seen_at = checkout.last_seen_at;
        repo.save_checkout(&checkout).await.unwrap();
        repo.save_project_link(&ProjectRepositoryLink::new(
            "Fresh Orient",
            repository.id,
            ProjectRepositoryRole::Primary,
        ))
        .await
        .unwrap();

        let packet = service
            .orient(OrientInput {
                cwd: Some(dir.path().display().to_string()),
                project: None,
                include_recent_commits: false,
                ..OrientInput::default()
            })
            .await
            .unwrap();

        let checkout = packet
            .repository_context
            .as_ref()
            .and_then(|context| context.checkout.as_ref())
            .expect("orientation should include refreshed checkout");
        assert_eq!(checkout.head_sha.as_deref(), Some(current_head.as_str()));
        assert_eq!(checkout.is_dirty, Some(false));
        assert!(checkout.last_seen_at >= stale_seen_at);
    }

    #[tokio::test]
    async fn orient_does_not_use_project_memory_when_repo_candidates_are_ambiguous() {
        let (service, repo) = setup_service_with_repository_repo().await;
        let dir = tempdir().unwrap();
        let repository = GitRepository::new("shared-repo");
        repo.save_repository(&repository).await.unwrap();
        repo.save_checkout(
            &LocalCheckout::new(dir.path().display().to_string()).with_repository(repository.id),
        )
        .await
        .unwrap();
        for project in ["Project A", "Project B"] {
            repo.save_project_link(&ProjectRepositoryLink::new(
                project,
                repository.id,
                ProjectRepositoryRole::Primary,
            ))
            .await
            .unwrap();
            service
                .capture_memory(project_memory_item(project, &format!("{project} decision")))
                .await
                .unwrap();
        }

        let packet = service
            .orient(OrientInput {
                cwd: Some(dir.path().display().to_string()),
                project: None,
                include_recent_commits: false,
                limit: Some(10),
                ..OrientInput::default()
            })
            .await
            .unwrap();

        assert!(packet.resolution.selected_project.is_none());
        assert!(packet.resolution.requires_confirmation);
        assert_eq!(packet.resolution.project_candidates.len(), 2);
        assert!(packet.active_decisions.is_empty());
        assert!(packet
            .ambiguities
            .iter()
            .any(|item| item.contains("multiple project candidates")));
    }

    #[tokio::test]
    async fn orient_component_link_narrows_project_candidate() {
        let (service, repo) = setup_service_with_repository_repo().await;
        let dir = tempdir().unwrap();
        let api_dir = dir.path().join("services/api");
        let web_dir = dir.path().join("apps/web");
        std::fs::create_dir_all(&api_dir).unwrap();
        std::fs::create_dir_all(&web_dir).unwrap();

        let repository = GitRepository::new("monorepo");
        repo.save_repository(&repository).await.unwrap();
        repo.save_checkout(
            &LocalCheckout::new(dir.path().display().to_string()).with_repository(repository.id),
        )
        .await
        .unwrap();

        let api_component = MonorepoComponent::new(repository.id, "api", "services/api");
        let web_component = MonorepoComponent::new(repository.id, "web", "apps/web");
        repo.save_component(&api_component).await.unwrap();
        repo.save_component(&web_component).await.unwrap();
        repo.save_project_link(
            &ProjectRepositoryLink::new(
                "API Project",
                repository.id,
                ProjectRepositoryRole::Primary,
            )
            .with_component(Some(api_component.id), api_component.path.clone()),
        )
        .await
        .unwrap();
        repo.save_project_link(
            &ProjectRepositoryLink::new(
                "Web Project",
                repository.id,
                ProjectRepositoryRole::Primary,
            )
            .with_component(Some(web_component.id), web_component.path.clone()),
        )
        .await
        .unwrap();
        service
            .capture_memory(project_memory_item("API Project", "API decision"))
            .await
            .unwrap();
        service
            .capture_memory(project_memory_item("Web Project", "Web decision"))
            .await
            .unwrap();

        let packet = service
            .orient(OrientInput {
                cwd: Some(api_dir.display().to_string()),
                project: None,
                include_recent_commits: false,
                limit: Some(10),
                ..OrientInput::default()
            })
            .await
            .unwrap();

        assert_eq!(
            packet.resolution.selected_project.as_deref(),
            Some("API Project")
        );
        assert_eq!(
            packet.resolution.source,
            OrientationResolutionSource::ComponentLink
        );
        assert_eq!(packet.resolution.component_names, vec!["api"]);
        assert_eq!(packet.active_decisions.len(), 1);
        assert_eq!(packet.active_decisions[0].title, "API decision");
    }

    #[tokio::test]
    async fn orient_explicit_project_overrides_repo_candidate() {
        let (service, repo) = setup_service_with_repository_repo().await;
        let dir = tempdir().unwrap();
        let repository = GitRepository::new("repo-a");
        repo.save_repository(&repository).await.unwrap();
        repo.save_checkout(
            &LocalCheckout::new(dir.path().display().to_string()).with_repository(repository.id),
        )
        .await
        .unwrap();
        repo.save_project_link(&ProjectRepositoryLink::new(
            "Project A",
            repository.id,
            ProjectRepositoryRole::Primary,
        ))
        .await
        .unwrap();

        service
            .capture_memory(project_memory_item(
                "Project B",
                "Explicit project decision",
            ))
            .await
            .unwrap();

        let packet = service
            .orient(OrientInput {
                cwd: Some(dir.path().display().to_string()),
                project: Some("Project B".to_string()),
                include_recent_commits: false,
                limit: Some(10),
                ..OrientInput::default()
            })
            .await
            .unwrap();

        assert_eq!(
            packet.resolution.selected_project.as_deref(),
            Some("Project B")
        );
        assert_eq!(
            packet.resolution.source,
            OrientationResolutionSource::ExplicitProject
        );
        assert!(!packet.resolution.requires_confirmation);
        assert_eq!(packet.resolution.project_candidates, vec!["Project A"]);
        assert_eq!(
            packet.active_decisions[0].title,
            "Explicit project decision"
        );
    }

    fn edit_digest_review_decision(
        root: &Path,
        files_written: &[String],
        source_fragment: &str,
        decision: &str,
        fields: &[(&str, &str)],
    ) {
        let candidate_path = files_written
            .iter()
            .filter(|path| path.starts_with("candidates/"))
            .find(|path| {
                std::fs::read_to_string(root.join(path))
                    .is_ok_and(|contents| contents.contains(source_fragment))
            })
            .expect("candidate review page for source should exist");
        let path = root.join(candidate_path);
        let mut contents = std::fs::read_to_string(&path).unwrap();
        contents = contents.replace(
            "decision: pending # accept | reject | quarantine | source_only",
            &format!("decision: {decision} # accept | reject | quarantine | source_only"),
        );
        for (key, value) in fields {
            contents = replace_review_field(&contents, key, value);
        }
        std::fs::write(path, contents).unwrap();
    }

    fn edit_digest_extraction_decision(root: &Path, candidate_path: &str, decision: &str) {
        let path = root.join(candidate_path);
        let contents = std::fs::read_to_string(&path).unwrap().replace(
            "decision: pending # accept | reject | quarantine",
            &format!("decision: {decision} # accept | reject | quarantine"),
        );
        std::fs::write(path, contents).unwrap();
    }

    fn replace_review_field(contents: &str, key: &str, value: &str) -> String {
        contents
            .lines()
            .map(|line| {
                if line.starts_with(&format!("{key}:")) {
                    format!("{key}: {value}")
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
