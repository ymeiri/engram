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
use engram_core::entity::Observation;
use engram_core::id::Id;
use engram_core::memory::{
    ClaimOrigin, EvidenceKind, EvidenceRef, Harness, KnowledgeCommit, MemoryChange,
    MemoryChangeType, MemoryCursor, MemoryItem, MemoryKind, MemoryScope, MemoryStatus,
    MemoryTrustMetadata, WriterProvenance,
};
use engram_core::repository::{
    MonorepoComponent, ProjectRepositoryLink, RecentGitCommit, RepositoryContext,
};
use engram_core::session::{Event, EventType};
use engram_core::telemetry::{BrainHarnessIntent, BrainHarnessOperation, BrainHarnessTrace};
use engram_store::{Db, MemoryRepo, RepositoryRepo, SessionRepo, TelemetryRepo};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;
use std::time::Instant;
use time::OffsetDateTime;
use tracing::info;

const BRAIN_LOOP_TOP_ITEM_LIMIT: usize = 5;
const BRAIN_LOOP_SUMMARY_CHAR_LIMIT: usize = 240;
const ORIENT_RECENT_GIT_COMMIT_LIMIT: usize = 5;
const ORIENT_RECENT_GIT_COMMIT_PATH_LIMIT: usize = 8;
const CURRENT_PLAN_TAG: &str = "current-plan";

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
    /// Optional host/application session label for telemetry correlation.
    pub external_session_id: Option<String>,
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

/// Input for promoting an entity observation into a durable Memory OS item.
#[derive(Debug, Clone)]
pub struct ObservationPromotionInput {
    /// Memory kind to create.
    pub kind: MemoryKind,
    /// Curated title for the promoted memory.
    pub title: String,
    /// Optional replacement content. Defaults to the source observation content.
    pub content: Option<String>,
    /// Scope for the promoted memory.
    pub scope: MemoryScope,
    /// Origin of the promoted claim.
    pub origin: ClaimOrigin,
    /// Writer provenance for the promotion.
    pub writer: WriterProvenance,
    /// Target lifecycle status. Only active and needs_review are valid for v1.
    pub status: MemoryStatus,
    /// Optional confidence override.
    pub confidence: Option<f32>,
    /// Extra tags to attach.
    pub tags: Vec<String>,
    /// Reviewer identity. Required when status is active.
    pub reviewer: Option<String>,
    /// Review rationale. Required when status is active.
    pub rationale: Option<String>,
}

/// Input for low-friction capture of current plan/method/next-action guidance.
#[derive(Debug, Clone)]
pub struct CurrentPlanCaptureInput {
    /// Memory kind to create. Must be decision or rule.
    pub kind: MemoryKind,
    /// Compact title for the current plan memory.
    pub title: String,
    /// Compact current plan, method, or next-action content.
    pub content: String,
    /// Scope for the captured plan.
    pub scope: MemoryScope,
    /// Origin of the captured claim.
    pub origin: ClaimOrigin,
    /// Writer provenance for the capture.
    pub writer: WriterProvenance,
    /// Evidence backing the current plan.
    pub evidence: Vec<EvidenceRef>,
    /// Optional confidence override.
    pub confidence: Option<f32>,
    /// Extra tags to attach.
    pub tags: Vec<String>,
    /// Whether to write a knowledge commit for the captured plan.
    pub create_commit: bool,
    /// Optional commit message. Defaults to the title.
    pub commit_message: Option<String>,
    /// Optional session that produced the knowledge commit.
    pub session_id: Option<Id>,
    /// Optional parent knowledge commit.
    pub parent_id: Option<Id>,
}

/// Result of capturing current plan guidance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentPlanCapture {
    /// Captured active MemoryItem.
    pub item: MemoryItem,
    /// Knowledge commit written for the capture, when requested.
    pub commit: Option<KnowledgeCommit>,
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
    /// Optional host/application session label for telemetry correlation.
    pub external_session_id: Option<String>,
    /// Caller intent for telemetry correlation.
    pub intent: Option<BrainHarnessIntent>,
    /// Free-form controlled eval scenario identifier for telemetry correlation.
    pub scenario_id: Option<String>,
    /// Free-form eval or comparison arm for telemetry correlation.
    pub arm: Option<String>,
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

/// Frictionless task-boundary context compiled by orient.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainLoop {
    /// Short scoped narrative for the current task.
    pub compiled_context: String,
    /// Highest-priority memory signals used to compile the context.
    pub top_items: Vec<BrainLoopItem>,
    /// Whether orient had to return a partial brain-loop projection.
    pub degraded: bool,
}

/// Auditable memory signal included in a brain-loop projection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainLoopItem {
    /// Memory item ID.
    pub id: Id,
    /// Memory kind.
    pub kind: MemoryKind,
    /// Memory title.
    pub title: String,
    /// Compact one-line memory summary.
    pub summary: String,
    /// Trust metadata for the memory item.
    pub trust: MemoryTrustMetadata,
    /// Why this memory was selected for the brain loop.
    pub why_relevant: String,
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
    /// Brain Loop v1 projection generated from the scoped orient result.
    pub brain_loop: BrainLoop,
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

    /// Persist a memory item after domain and capture-policy validation.
    pub async fn capture_memory(&self, item: MemoryItem) -> IndexResult<MemoryItem> {
        validate_memory_item(&item)?;
        let item = apply_capture_policy(item);
        self.repo.save_memory_item(&item).await?;
        Ok(item)
    }

    /// Capture compact active current-plan guidance and optionally commit it.
    pub async fn capture_current_plan(
        &self,
        input: CurrentPlanCaptureInput,
    ) -> IndexResult<CurrentPlanCapture> {
        validate_current_plan_capture(&input)?;

        let mut item = MemoryItem::new(
            input.kind,
            input.title,
            input.content,
            input.scope,
            input.origin,
            input.writer.clone(),
        )
        .with_status(MemoryStatus::Active)
        .with_tag(CURRENT_PLAN_TAG);

        for evidence in input.evidence {
            item = item.with_evidence(evidence);
        }
        if let Some(confidence) = input.confidence {
            item = item.with_confidence(confidence);
        }
        for tag in input.tags {
            if !item.tags.contains(&tag) {
                item = item.with_tag(tag);
            }
        }

        let item = self.capture_memory(item).await?;
        if item.status != MemoryStatus::Active {
            return Err(IndexError::InvalidState(format!(
                "current plan capture produced {} memory; add manual_review evidence or use memory(action=add)",
                item.status
            )));
        }
        let (item, superseded_ids) = self.supersede_previous_current_plan_items(item).await?;

        let commit = if input.create_commit {
            let message = input
                .commit_message
                .unwrap_or_else(|| format!("Capture current plan: {}", item.title));
            let mut changes = vec![MemoryChange::new(
                MemoryChangeType::Added,
                item.title.clone(),
                "Captured compact current-plan guidance for future resume orientation.",
            )
            .with_item(item.id)];
            for superseded_id in superseded_ids {
                changes.push(
                    MemoryChange::new(
                        MemoryChangeType::Superseded,
                        format!("Superseded current-plan memory {superseded_id}"),
                        format!(
                            "Superseded by newer current-plan memory {} for the same scope.",
                            item.id
                        ),
                    )
                    .with_item(superseded_id),
                );
            }
            Some(
                self.commit_changes(
                    input.writer,
                    message,
                    changes,
                    input.session_id,
                    input.parent_id,
                )
                .await?,
            )
        } else {
            None
        };

        Ok(CurrentPlanCapture { item, commit })
    }

    /// Promote an entity observation into Memory OS while preserving source evidence.
    pub async fn promote_observation_to_memory(
        &self,
        observation: &Observation,
        input: ObservationPromotionInput,
    ) -> IndexResult<MemoryItem> {
        if !matches!(
            input.status,
            MemoryStatus::Active | MemoryStatus::NeedsReview
        ) {
            return Err(IndexError::InvalidState(format!(
                "observation promotion cannot create {} memory",
                input.status
            )));
        }
        if let Some(existing) = self
            .memory_promoted_from_observation(&observation.id)
            .await?
        {
            return Err(IndexError::InvalidState(format!(
                "observation {} is already promoted to memory item {}",
                observation.id, existing.id
            )));
        }

        let review = match (input.reviewer, input.rationale) {
            (Some(reviewer), Some(rationale)) => Some(review_evidence(reviewer, rationale)?),
            (None, None) => None,
            _ => {
                return Err(IndexError::Parse(
                    "reviewer and rationale must be provided together".to_string(),
                ))
            }
        };
        if input.status == MemoryStatus::Active && review.is_none() {
            return Err(IndexError::Parse(
                "reviewer and rationale required for active observation promotion".to_string(),
            ));
        }

        let source_key = observation.key.as_deref().unwrap_or("unkeyed observation");
        let content = input.content.unwrap_or_else(|| observation.content.clone());
        let mut item = MemoryItem::new(
            input.kind,
            input.title,
            content,
            input.scope,
            input.origin,
            input.writer,
        )
        .with_evidence(
            EvidenceRef::new(EvidenceKind::Observation, observation.id.to_string())
                .with_summary(format!("Promoted entity observation `{source_key}`.")),
        )
        .with_status(input.status)
        .with_tag(format!("source-observation:{}", observation.id));

        if let Some(confidence) = input.confidence {
            item = item.with_confidence(confidence);
        }
        for tag in input.tags {
            item = item.with_tag(tag);
        }
        if let Some(review) = review {
            item = item.with_evidence(review);
        }

        self.capture_memory(item).await
    }

    /// Get a memory item by ID.
    pub async fn get_memory(&self, id: &Id) -> IndexResult<Option<MemoryItem>> {
        Ok(self.repo.get_memory_item(id).await?)
    }

    /// Promote a review candidate into active memory with reviewer evidence.
    pub async fn promote_memory(
        &self,
        id: &Id,
        reviewer: impl Into<String>,
        rationale: impl Into<String>,
    ) -> IndexResult<MemoryItem> {
        let item = self.get_required_memory(id).await?;
        if item.status != MemoryStatus::NeedsReview {
            return Err(IndexError::InvalidState(format!(
                "memory item {id} is not needs_review (status: {})",
                item.status
            )));
        }

        let item = item
            .with_status(MemoryStatus::Active)
            .with_evidence(review_evidence(reviewer, rationale)?);
        self.repo.save_memory_item(&item).await?;
        Ok(item)
    }

    /// Reject a review candidate while keeping it auditable.
    pub async fn reject_memory(
        &self,
        id: &Id,
        reviewer: impl Into<String>,
        rationale: impl Into<String>,
    ) -> IndexResult<MemoryItem> {
        let item = self.get_required_memory(id).await?;
        if item.status != MemoryStatus::NeedsReview {
            return Err(IndexError::InvalidState(format!(
                "memory item {id} is not needs_review (status: {})",
                item.status
            )));
        }

        let item = item
            .with_status(MemoryStatus::Rejected)
            .with_evidence(review_evidence(reviewer, rationale)?);
        self.repo.save_memory_item(&item).await?;
        Ok(item)
    }

    /// Promote a replacement memory item and mark the replaced item as superseded.
    pub async fn supersede_memory(
        &self,
        new_id: &Id,
        old_id: &Id,
        reviewer: impl Into<String>,
        rationale: impl Into<String>,
    ) -> IndexResult<(MemoryItem, MemoryItem)> {
        if new_id == old_id {
            return Err(IndexError::InvalidState(
                "memory item cannot supersede itself".to_string(),
            ));
        }

        let reviewer = reviewer.into();
        let rationale = rationale.into();
        let mut new_item = self.get_required_memory(new_id).await?;
        let mut old_item = self.get_required_memory(old_id).await?;
        if matches!(
            old_item.status,
            MemoryStatus::Archived | MemoryStatus::Rejected | MemoryStatus::Superseded
        ) {
            return Err(IndexError::InvalidState(format!(
                "memory item {old_id} cannot be superseded from status {}",
                old_item.status
            )));
        }
        if matches!(
            new_item.status,
            MemoryStatus::Archived | MemoryStatus::Rejected
        ) {
            return Err(IndexError::InvalidState(format!(
                "memory item {new_id} cannot supersede from status {}",
                new_item.status
            )));
        }

        if !new_item.supersedes.contains(old_id) {
            new_item = new_item.with_superseded_item(*old_id);
        }
        new_item = new_item
            .with_status(MemoryStatus::Active)
            .with_evidence(review_evidence(
                reviewer.clone(),
                format!("Supersedes {old_id}: {rationale}"),
            )?);
        old_item = old_item
            .with_status(MemoryStatus::Superseded)
            .with_evidence(review_evidence(
                reviewer,
                format!("Superseded by {new_id}: {rationale}"),
            )?);

        self.repo.save_memory_item(&new_item).await?;
        self.repo.save_memory_item(&old_item).await?;
        Ok((new_item, old_item))
    }

    /// Archive a memory item with metadata.
    pub async fn archive_memory(
        &self,
        id: &Id,
        reason: impl Into<String>,
        archived_by: Option<String>,
    ) -> IndexResult<MemoryItem> {
        let item = self.get_required_memory(id).await?;
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

    async fn memory_promoted_from_observation(
        &self,
        observation_id: &Id,
    ) -> IndexResult<Option<MemoryItem>> {
        let target = observation_id.to_string();
        let items = self.repo.list_memory_items(None, None).await?;
        Ok(items.into_iter().find(|item| {
            !matches!(item.status, MemoryStatus::Archived | MemoryStatus::Rejected)
                && item
                    .evidence
                    .iter()
                    .any(|e| e.kind == EvidenceKind::Observation && e.target == target)
        }))
    }

    async fn supersede_previous_current_plan_items(
        &self,
        mut item: MemoryItem,
    ) -> IndexResult<(MemoryItem, Vec<Id>)> {
        let previous = self
            .repo
            .list_memory_items(Some(MemoryStatus::Active), None)
            .await?
            .into_iter()
            .filter(|candidate| {
                candidate.id != item.id
                    && is_current_plan_item(candidate)
                    && current_plan_scope_key(&candidate.scope)
                        == current_plan_scope_key(&item.scope)
            })
            .collect::<Vec<_>>();

        if previous.is_empty() {
            return Ok((item, Vec::new()));
        }

        let superseded_ids = previous
            .iter()
            .map(|previous| previous.id)
            .collect::<Vec<_>>();
        for superseded_id in &superseded_ids {
            if !item.supersedes.contains(superseded_id) {
                item = item.with_superseded_item(*superseded_id);
            }
        }
        item = item.with_evidence(
            EvidenceRef::new(
                EvidenceKind::ToolCall,
                "memory(action=capture_current_plan)",
            )
            .with_summary(format!(
                "Supersedes {} older active current-plan item(s) for the same scope.",
                superseded_ids.len()
            )),
        );
        self.repo.save_memory_item(&item).await?;

        for previous_item in previous {
            let superseded = previous_item
                .with_status(MemoryStatus::Superseded)
                .with_evidence(
                    EvidenceRef::new(
                        EvidenceKind::ToolCall,
                        format!("memory(action=capture_current_plan):{}", item.id),
                    )
                    .with_summary(format!(
                        "Superseded by newer current-plan memory {} for the same scope.",
                        item.id
                    )),
                );
            self.repo.save_memory_item(&superseded).await?;
        }

        Ok((item, superseded_ids))
    }

    async fn get_required_memory(&self, id: &Id) -> IndexResult<MemoryItem> {
        self.repo
            .get_memory_item(id)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("memory item not found: {id}")))
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
            .with_external_session_id(options.external_session_id.clone())
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
        let mut repository_context = self
            .resolve_repository_context(input.cwd.as_deref())
            .await?;
        if input.include_recent_commits {
            attach_recent_git_commits(&mut repository_context)?;
        }
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
        let relevant_active =
            prioritize_current_plan_for_resume(relevant_active, input.intent.as_ref());
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
        let brain_loop = build_brain_loop(BrainLoopParts {
            scope: &scope,
            resolution: &resolution,
            project: effective_project,
            cwd: input.cwd.as_deref(),
            query: input.prompt.as_deref(),
            intent: input.intent.as_ref(),
            decisions: &active_decisions,
            rules: &active_rules,
            preferences: &preferences,
            limitations: &limitations,
            review_needed: &relevant_review,
            ambiguities: &ambiguities,
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
            .with_external_session_id(input.external_session_id.clone())
            .with_agent(input.agent.clone())
            .with_intent(input.intent.clone())
            .with_scenario_id(input.scenario_id.clone())
            .with_arm(input.arm.clone())
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
            brain_loop,
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
            recent_commits: Vec::new(),
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

fn prioritize_current_plan_for_resume(
    items: Vec<MemoryItem>,
    intent: Option<&BrainHarnessIntent>,
) -> Vec<MemoryItem> {
    if !matches!(intent, Some(BrainHarnessIntent::ResumeSession)) {
        return items;
    }

    let mut latest_by_scope: HashMap<String, MemoryItem> = HashMap::new();
    for item in items.iter().filter(|item| is_current_plan_item(item)) {
        let scope_key = current_plan_scope_key(&item.scope);
        let should_replace = latest_by_scope
            .get(&scope_key)
            .map(|existing| {
                item.updated_at > existing.updated_at
                    || (item.updated_at == existing.updated_at
                        && item.id.to_string() > existing.id.to_string())
            })
            .unwrap_or(true);
        if should_replace {
            latest_by_scope.insert(scope_key, item.clone());
        }
    }

    if latest_by_scope.is_empty() {
        return items;
    }

    let mut latest_current_plans = latest_by_scope.into_values().collect::<Vec<_>>();
    latest_current_plans.sort_by(|left, right| {
        right
            .updated_at
            .cmp(&left.updated_at)
            .then_with(|| right.id.to_string().cmp(&left.id.to_string()))
    });

    let mut prioritized = latest_current_plans;
    prioritized.extend(items.into_iter().filter(|item| !is_current_plan_item(item)));
    prioritized
}

fn is_current_plan_item(item: &MemoryItem) -> bool {
    item.tags
        .iter()
        .any(|tag| tag.eq_ignore_ascii_case(CURRENT_PLAN_TAG))
}

fn current_plan_scope_key(scope: &MemoryScope) -> String {
    match scope {
        MemoryScope::Global => "global".to_string(),
        MemoryScope::User => "user".to_string(),
        MemoryScope::Project { project_name, .. } => {
            format!("project:{}", project_name.to_lowercase())
        }
        MemoryScope::Task {
            project_name,
            task_name,
            ..
        } => format!(
            "task:{}/{}",
            project_name.as_deref().unwrap_or("").to_lowercase(),
            task_name.to_lowercase()
        ),
        MemoryScope::Entity { entity_name, .. } => {
            format!("entity:{}", entity_name.to_lowercase())
        }
        MemoryScope::Repository {
            remote_url,
            local_path,
            ..
        } => format!(
            "repository:{}",
            local_path
                .as_deref()
                .or(remote_url.as_deref())
                .unwrap_or("")
                .to_lowercase()
        ),
        MemoryScope::Session { session_id } => format!("session:{session_id}"),
        MemoryScope::Custom { name } => format!("custom:{}", name.to_lowercase()),
    }
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

struct BrainLoopParts<'a> {
    scope: &'a str,
    resolution: &'a OrientationResolution,
    project: Option<&'a str>,
    cwd: Option<&'a str>,
    query: Option<&'a str>,
    intent: Option<&'a BrainHarnessIntent>,
    decisions: &'a [MemoryItem],
    rules: &'a [MemoryItem],
    preferences: &'a [MemoryItem],
    limitations: &'a [MemoryItem],
    review_needed: &'a [MemoryItem],
    ambiguities: &'a [String],
}

struct BrainLoopGroup<'a> {
    items: &'a [MemoryItem],
    reason: &'static str,
    original_index: usize,
    score: f32,
}

fn build_brain_loop(parts: BrainLoopParts<'_>) -> BrainLoop {
    let top_items = brain_loop_top_items(&parts);
    let compiled_context =
        brain_loop_compiled_context(parts.scope, parts.resolution, &top_items, parts.ambiguities);

    BrainLoop {
        compiled_context,
        top_items,
        degraded: false,
    }
}

fn brain_loop_top_items(parts: &BrainLoopParts<'_>) -> Vec<BrainLoopItem> {
    let mut items = Vec::new();
    let mut groups = [
        BrainLoopGroup {
            items: parts.rules,
            reason: "Active rule matched the orientation scope.",
            original_index: 0,
            score: 0.0,
        },
        BrainLoopGroup {
            items: parts.preferences,
            reason: "Preference matched the orientation scope.",
            original_index: 1,
            score: 0.0,
        },
        BrainLoopGroup {
            items: parts.limitations,
            reason: "Known limitation matched the orientation scope.",
            original_index: 2,
            score: 0.0,
        },
        BrainLoopGroup {
            items: parts.decisions,
            reason: "Active decision matched the orientation scope.",
            original_index: 3,
            score: 0.0,
        },
        BrainLoopGroup {
            items: parts.review_needed,
            reason: "Review-needed memory matched the orientation scope.",
            original_index: 4,
            score: 0.0,
        },
    ];
    let resume_current_plan = matches!(parts.intent, Some(BrainHarnessIntent::ResumeSession))
        && parts.decisions.first().is_some_and(is_current_plan_item);
    if parts.query.is_some_and(|query| !query.trim().is_empty()) {
        let context = MemoryRankContext::orientation(parts.project, parts.cwd, parts.query);
        for group in &mut groups {
            group.score = group
                .items
                .first()
                .and_then(|item| rank_memory_item(item.clone(), context))
                .filter(|ranked| ranked.components.text > 0.0)
                .map(|ranked| ranked.score)
                .unwrap_or(0.0);
        }
    }
    if resume_current_plan {
        groups[3].score = f32::INFINITY;
    }
    if parts.query.is_some_and(|query| !query.trim().is_empty()) || resume_current_plan {
        groups.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.original_index.cmp(&right.original_index))
        });
    }
    let mut offsets = [0usize; 5];

    loop {
        let mut added = false;
        for (index, group) in groups.iter().enumerate() {
            if items.len() == BRAIN_LOOP_TOP_ITEM_LIMIT {
                return items;
            }
            if let Some(item) = group.items.get(offsets[index]) {
                items.push(brain_loop_item(item, group.reason));
                offsets[index] += 1;
                added = true;
            }
        }
        if !added {
            return items;
        }
    }
}

fn brain_loop_item(item: &MemoryItem, reason: &str) -> BrainLoopItem {
    BrainLoopItem {
        id: item.id,
        kind: item.kind.clone(),
        title: item.title.clone(),
        summary: compact_brain_loop_summary(&item.content),
        trust: item.trust_metadata(),
        why_relevant: reason.to_string(),
    }
}

fn brain_loop_compiled_context(
    scope: &str,
    resolution: &OrientationResolution,
    top_items: &[BrainLoopItem],
    ambiguities: &[String],
) -> String {
    let mut parts = vec![format!("Brain Loop v1 orientation for {scope}.")];
    if let Some(project) = &resolution.selected_project {
        let source = match resolution.source {
            OrientationResolutionSource::ExplicitProject => "explicit project",
            OrientationResolutionSource::ComponentLink => "component link",
            OrientationResolutionSource::RepositoryLink => "repository link",
            OrientationResolutionSource::Unresolved => "unresolved scope",
        };
        parts.push(format!(
            "Using project-scoped memory for {project} ({source})."
        ));
    } else {
        parts.push("No project scope was selected.".to_string());
    }

    if top_items.is_empty() {
        parts.push("No scoped memory signals were selected.".to_string());
    } else {
        let signals = top_items
            .iter()
            .map(|item| format!("{}: {}", item.kind, item.title))
            .collect::<Vec<_>>()
            .join("; ");
        parts.push(format!("Top signals: {signals}."));
    }

    if !ambiguities.is_empty() {
        parts.push(format!("Ambiguities: {}.", ambiguities.join("; ")));
    }

    parts.join(" ")
}

fn compact_brain_loop_summary(content: &str) -> String {
    let normalized = content.split_whitespace().collect::<Vec<_>>().join(" ");
    let char_count = normalized.chars().count();
    if char_count <= BRAIN_LOOP_SUMMARY_CHAR_LIMIT {
        return normalized;
    }

    let summary = normalized
        .chars()
        .take(BRAIN_LOOP_SUMMARY_CHAR_LIMIT)
        .collect::<String>();
    format!("{summary}...")
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
    if !context.recent_commits.is_empty() {
        lines.push("- Recent Git commits:".to_string());
        for commit in &context.recent_commits {
            let paths = if commit.changed_paths.is_empty() {
                String::new()
            } else {
                format!(" [{}]", commit.changed_paths.join(", "))
            };
            lines.push(format!(
                "  - {}: {}{}",
                short_commit_sha(&commit.sha),
                commit.summary,
                paths
            ));
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

fn attach_recent_git_commits(context: &mut Option<RepositoryContext>) -> IndexResult<()> {
    let Some(context) = context else {
        return Ok(());
    };
    let Some(checkout) = &context.checkout else {
        return Ok(());
    };
    context.recent_commits = recent_git_commits(Path::new(&checkout.local_path))?;
    Ok(())
}

fn recent_git_commits(checkout_path: &Path) -> IndexResult<Vec<RecentGitCommit>> {
    let limit = ORIENT_RECENT_GIT_COMMIT_LIMIT.to_string();
    let output = Command::new("git")
        .arg("-C")
        .arg(checkout_path)
        .args(["log", "-n", limit.as_str(), "--pretty=format:%H%x1f%s"])
        .output()?;
    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut commits = Vec::new();
    for line in stdout.lines() {
        let Some((sha, summary)) = line.split_once('\u{1f}') else {
            continue;
        };
        let sha = sha.trim();
        commits.push(RecentGitCommit::new(
            sha,
            summary.trim(),
            recent_git_commit_paths(checkout_path, sha)?,
        ));
    }
    Ok(commits)
}

fn recent_git_commit_paths(checkout_path: &Path, sha: &str) -> IndexResult<Vec<String>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(checkout_path)
        .args([
            "show",
            "--pretty=format:",
            "--name-only",
            "--diff-filter=ACMR",
            sha,
        ])
        .output()?;
    if !output.status.success() {
        return Ok(Vec::new());
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(ORIENT_RECENT_GIT_COMMIT_PATH_LIMIT)
        .map(str::to_string)
        .collect())
}

fn short_commit_sha(sha: &str) -> &str {
    sha.get(..12).unwrap_or(sha)
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

fn validate_current_plan_capture(input: &CurrentPlanCaptureInput) -> IndexResult<()> {
    match input.kind {
        MemoryKind::Decision | MemoryKind::Rule => {}
        _ => {
            return Err(IndexError::Parse(
                "current plan capture only supports decision or rule memory".to_string(),
            ));
        }
    }
    if input.evidence.is_empty() {
        return Err(IndexError::Parse(
            "current plan capture requires at least one evidence record".to_string(),
        ));
    }
    if origin_requires_review(&input.origin) && !has_manual_review_evidence_refs(&input.evidence) {
        return Err(IndexError::Parse(
            "current plan capture from this origin requires manual_review evidence".to_string(),
        ));
    }
    Ok(())
}

fn apply_capture_policy(item: MemoryItem) -> MemoryItem {
    if item.status != MemoryStatus::Active {
        return item;
    }

    if origin_requires_review(&item.origin) && !has_manual_review_evidence(&item) {
        return item.with_status(MemoryStatus::NeedsReview);
    }

    if item.kind == MemoryKind::Preference {
        if preference_can_be_active(&item) {
            return item;
        }
        return item.with_status(MemoryStatus::NeedsReview);
    }

    if durable_guidance_requires_evidence(&item.kind) && item.evidence.is_empty() {
        return item.with_status(MemoryStatus::NeedsReview);
    }

    item
}

fn review_evidence(
    reviewer: impl Into<String>,
    rationale: impl Into<String>,
) -> IndexResult<EvidenceRef> {
    let reviewer = reviewer.into();
    let rationale = rationale.into();
    if reviewer.trim().is_empty() {
        return Err(IndexError::Parse("reviewer must not be empty".to_string()));
    }
    if rationale.trim().is_empty() {
        return Err(IndexError::Parse(
            "review rationale must not be empty".to_string(),
        ));
    }

    Ok(EvidenceRef::new(EvidenceKind::ManualReview, reviewer).with_summary(rationale))
}

fn durable_guidance_requires_evidence(kind: &MemoryKind) -> bool {
    matches!(
        kind,
        MemoryKind::Decision | MemoryKind::Rule | MemoryKind::Limitation
    )
}

fn preference_can_be_active(item: &MemoryItem) -> bool {
    matches!(
        item.origin,
        ClaimOrigin::UserStated | ClaimOrigin::UserCorrected
    ) || has_manual_review_evidence(item)
}

fn origin_requires_review(origin: &ClaimOrigin) -> bool {
    matches!(
        origin,
        ClaimOrigin::AgentInferred
            | ClaimOrigin::Imported
            | ClaimOrigin::Migrated
            | ClaimOrigin::GeneratedSummary
            | ClaimOrigin::Custom(_)
    )
}

fn has_manual_review_evidence(item: &MemoryItem) -> bool {
    item.evidence
        .iter()
        .any(|evidence| matches!(evidence.kind, EvidenceKind::ManualReview))
}

fn has_manual_review_evidence_refs(evidence: &[EvidenceRef]) -> bool {
    evidence
        .iter()
        .any(|evidence| matches!(evidence.kind, EvidenceKind::ManualReview))
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
    use crate::search::{SearchOptions, SearchService};
    use engram_core::entity::Observation;
    use engram_core::memory::{
        ClaimOrigin, EvidenceKind, EvidenceRef, Harness, MemoryChangeType, MemoryKind,
        MemoryReviewState, MemoryScope, MemoryStatus, ModelIdentity,
    };
    use engram_core::repository::{
        GitRepository, LocalCheckout, MonorepoComponent, ProjectRepositoryLink,
        ProjectRepositoryRole,
    };
    use engram_core::search::SearchLayer;
    use engram_core::work::{Project, ProjectObservation};
    use engram_store::{RepositoryRepo, WorkRepo};
    use std::fs;
    use std::path::Path;
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

    async fn setup_migration_viability_services() -> (MemoryService, SearchService, WorkRepo) {
        let config = engram_store::StoreConfig::memory();
        let db = engram_store::connect_and_init(&config).await.unwrap();
        engram_store::init_schema(&db).await.unwrap();

        (
            MemoryService::new(db.clone()),
            SearchService::new(db.clone()),
            WorkRepo::new(db),
        )
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
        .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "unit-test"))
    }

    fn current_plan_input(project: &str, title: &str, content: &str) -> CurrentPlanCaptureInput {
        CurrentPlanCaptureInput {
            kind: MemoryKind::Decision,
            title: title.to_string(),
            content: content.to_string(),
            scope: MemoryScope::project(project),
            origin: ClaimOrigin::ToolResult,
            writer: writer(),
            evidence: vec![EvidenceRef::new(EvidenceKind::ToolCall, "unit-test")],
            confidence: Some(0.9),
            tags: Vec::new(),
            create_commit: false,
            commit_message: None,
            session_id: None,
            parent_id: None,
        }
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

    fn accept_first_migration_candidate(root: &Path, export: &MigrationReviewExport) -> String {
        let mut candidate_paths = export
            .files_written
            .iter()
            .filter(|path| path.starts_with("candidates/"))
            .cloned()
            .collect::<Vec<_>>();
        candidate_paths.sort();
        let candidate_path = candidate_paths
            .into_iter()
            .next()
            .expect("review export should include a candidate");
        let path = root.join(&candidate_path);
        let contents = fs::read_to_string(&path).expect("candidate page should be readable");
        fs::write(
            &path,
            contents.replace("- [ ] Accept for migration", "- [x] Accept for migration"),
        )
        .expect("candidate page should be writable");
        candidate_path
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
    async fn promote_observation_to_memory_creates_reviewed_active_item() {
        let service = setup_service().await;
        let observation = Observation::new(
            Id::new(),
            "Brain Loop v1 should keep raw observations out of the hot path.",
        )
        .with_key("decisions.brain-loop-observation-promotion")
        .with_source("unit-test");

        let item = service
            .promote_observation_to_memory(
                &observation,
                ObservationPromotionInput {
                    kind: MemoryKind::Decision,
                    title: "Promote important observations explicitly".to_string(),
                    content: None,
                    scope: MemoryScope::project("engram"),
                    origin: ClaimOrigin::AgentObserved,
                    writer: writer(),
                    status: MemoryStatus::Active,
                    confidence: Some(0.9),
                    tags: vec!["brain-loop".to_string()],
                    reviewer: Some("yuval".to_string()),
                    rationale: Some("Reviewed as durable architecture guidance.".to_string()),
                },
            )
            .await
            .unwrap();

        assert_eq!(item.status, MemoryStatus::Active);
        assert_eq!(item.confidence.value(), 0.9);
        assert_eq!(item.content, observation.content);
        assert!(item
            .tags
            .contains(&format!("source-observation:{}", observation.id)));
        assert!(item.evidence.iter().any(|evidence| {
            evidence.kind == EvidenceKind::Observation
                && evidence.target == observation.id.to_string()
        }));
        assert!(item
            .evidence
            .iter()
            .any(|evidence| evidence.kind == EvidenceKind::ManualReview));

        let err = service
            .promote_observation_to_memory(
                &observation,
                ObservationPromotionInput {
                    kind: MemoryKind::Decision,
                    title: "Duplicate".to_string(),
                    content: None,
                    scope: MemoryScope::project("engram"),
                    origin: ClaimOrigin::AgentObserved,
                    writer: writer(),
                    status: MemoryStatus::Active,
                    confidence: None,
                    tags: Vec::new(),
                    reviewer: Some("yuval".to_string()),
                    rationale: Some("Already promoted.".to_string()),
                },
            )
            .await
            .unwrap_err();
        assert!(err.to_string().contains("already promoted"));
    }

    #[tokio::test]
    async fn promote_observation_to_active_memory_requires_review() {
        let service = setup_service().await;
        let observation =
            Observation::new(Id::new(), "Potential guidance.").with_key("decisions.needs-review");

        let err = service
            .promote_observation_to_memory(
                &observation,
                ObservationPromotionInput {
                    kind: MemoryKind::Decision,
                    title: "Missing review".to_string(),
                    content: None,
                    scope: MemoryScope::project("engram"),
                    origin: ClaimOrigin::AgentObserved,
                    writer: writer(),
                    status: MemoryStatus::Active,
                    confidence: None,
                    tags: Vec::new(),
                    reviewer: None,
                    rationale: None,
                },
            )
            .await
            .unwrap_err();
        assert!(err.to_string().contains("reviewer and rationale required"));
    }

    #[tokio::test]
    async fn capture_current_plan_creates_active_item_and_commit() {
        let service = setup_service().await;

        let capture = service
            .capture_current_plan(CurrentPlanCaptureInput {
                kind: MemoryKind::Decision,
                title: "Current Brain Harness plan".to_string(),
                content: "Implement compact current-plan capture before ranking changes."
                    .to_string(),
                scope: MemoryScope::project("engram"),
                origin: ClaimOrigin::ToolResult,
                writer: writer(),
                evidence: vec![EvidenceRef::new(
                    EvidenceKind::ToolCall,
                    "engram.orient trace current-plan-test",
                )
                .with_summary("Orient missed current plan until active MemoryItems were added.")],
                confidence: Some(0.94),
                tags: vec!["current-plan".to_string(), "brain-harness".to_string()],
                create_commit: true,
                commit_message: Some("Capture current Brain Harness plan".to_string()),
                session_id: None,
                parent_id: None,
            })
            .await
            .unwrap();

        assert_eq!(capture.item.status, MemoryStatus::Active);
        assert_eq!(capture.item.kind, MemoryKind::Decision);
        assert_eq!(capture.item.confidence.value(), 0.94);
        assert!(capture.item.tags.contains(&"current-plan".to_string()));
        assert!(capture.item.tags.contains(&"brain-harness".to_string()));
        assert_eq!(
            capture
                .item
                .tags
                .iter()
                .filter(|tag| tag.as_str() == "current-plan")
                .count(),
            1
        );

        let commit = capture
            .commit
            .expect("current plan capture should commit by default");
        assert_eq!(commit.message, "Capture current Brain Harness plan");
        assert_eq!(commit.change_count(), 1);
        assert_eq!(commit.changes[0].change_type, MemoryChangeType::Added);
        assert_eq!(commit.changes[0].item_id, Some(capture.item.id));
    }

    #[tokio::test]
    async fn capture_current_plan_supersedes_previous_same_project_current_plan() {
        let service = setup_service().await;

        let old = service
            .capture_current_plan(current_plan_input(
                "engram",
                "Old current plan",
                "Resume from the old current plan.",
            ))
            .await
            .unwrap()
            .item;
        let other_project = service
            .capture_current_plan(current_plan_input(
                "other",
                "Other project current plan",
                "Keep the other project plan active.",
            ))
            .await
            .unwrap()
            .item;
        let mut input = current_plan_input(
            "engram",
            "New current plan",
            "Resume from the new current plan.",
        );
        input.create_commit = true;
        let capture = service.capture_current_plan(input).await.unwrap();
        let new = capture.item;

        assert!(new.supersedes.contains(&old.id));
        assert!(!new.supersedes.contains(&other_project.id));
        let commit = capture
            .commit
            .expect("superseding current-plan capture should create a commit");
        assert_eq!(commit.change_count(), 2);
        assert!(commit.changes.iter().any(|change| {
            change.change_type == MemoryChangeType::Superseded && change.item_id == Some(old.id)
        }));
        assert_eq!(
            service.get_memory(&old.id).await.unwrap().unwrap().status,
            MemoryStatus::Superseded
        );
        assert_eq!(
            service
                .get_memory(&other_project.id)
                .await
                .unwrap()
                .unwrap()
                .status,
            MemoryStatus::Active
        );

        let active_engram_current_plans = service
            .list_active_memory(None)
            .await
            .unwrap()
            .into_iter()
            .filter(|item| {
                is_current_plan_item(item)
                    && matches!(
                        &item.scope,
                        MemoryScope::Project { project_name, .. }
                            if project_name == "engram"
                    )
            })
            .collect::<Vec<_>>();
        assert_eq!(active_engram_current_plans.len(), 1);
        assert_eq!(active_engram_current_plans[0].id, new.id);
    }

    #[tokio::test]
    async fn capture_current_plan_requires_evidence_and_guidance_kind() {
        let service = setup_service().await;

        let base = CurrentPlanCaptureInput {
            kind: MemoryKind::Decision,
            title: "Current plan".to_string(),
            content: "Keep this plan available for resume orientation.".to_string(),
            scope: MemoryScope::project("engram"),
            origin: ClaimOrigin::AgentObserved,
            writer: writer(),
            evidence: Vec::new(),
            confidence: None,
            tags: Vec::new(),
            create_commit: false,
            commit_message: None,
            session_id: None,
            parent_id: None,
        };

        let err = service
            .capture_current_plan(base.clone())
            .await
            .unwrap_err();
        assert!(err.to_string().contains("requires at least one evidence"));

        let mut unsupported = base;
        unsupported.kind = MemoryKind::ProjectFact;
        unsupported.evidence = vec![EvidenceRef::new(EvidenceKind::File, "docs/current.md")];
        let err = service.capture_current_plan(unsupported).await.unwrap_err();
        assert!(err.to_string().contains("decision or rule"));
    }

    #[tokio::test]
    async fn capture_policy_allows_user_preferences_without_extra_evidence() {
        let service = setup_service().await;
        for origin in [ClaimOrigin::UserStated, ClaimOrigin::UserCorrected] {
            let item = MemoryItem::new(
                MemoryKind::Preference,
                format!("Preference from {origin}"),
                "User prefers concise status updates.",
                MemoryScope::User,
                origin,
                writer(),
            );

            let captured = service.capture_memory(item).await.unwrap();

            assert_eq!(captured.status, MemoryStatus::Active);
            assert!(captured.evidence.is_empty());
        }
    }

    #[tokio::test]
    async fn capture_policy_downgrades_active_durable_guidance_without_evidence() {
        let service = setup_service().await;
        for kind in [
            MemoryKind::Decision,
            MemoryKind::Rule,
            MemoryKind::Limitation,
        ] {
            let item = MemoryItem::new(
                kind.clone(),
                format!("Unevidenced {kind}"),
                "Durable guidance must not become active without evidence.",
                MemoryScope::project("engram"),
                ClaimOrigin::UserStated,
                writer(),
            );

            let captured = service.capture_memory(item).await.unwrap();

            assert_eq!(captured.status, MemoryStatus::NeedsReview);
        }

        assert!(service.list_active_memory(None).await.unwrap().is_empty());
        assert_eq!(
            service
                .list_memory_needing_review(None)
                .await
                .unwrap()
                .len(),
            3
        );
    }

    #[tokio::test]
    async fn capture_policy_allows_evidenced_durable_guidance() {
        let service = setup_service().await;
        for kind in [
            MemoryKind::Decision,
            MemoryKind::Rule,
            MemoryKind::Limitation,
        ] {
            let item = MemoryItem::new(
                kind.clone(),
                format!("Evidenced {kind}"),
                "Durable guidance can become active when backed by evidence.",
                MemoryScope::project("engram"),
                ClaimOrigin::UserStated,
                writer(),
            )
            .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "unit-test"));

            let captured = service.capture_memory(item).await.unwrap();

            assert_eq!(captured.status, MemoryStatus::Active);
        }

        assert_eq!(service.list_active_memory(None).await.unwrap().len(), 3);
        assert!(service
            .list_memory_needing_review(None)
            .await
            .unwrap()
            .is_empty());
    }

    #[tokio::test]
    async fn capture_policy_keeps_review_origins_out_of_active_memory() {
        let service = setup_service().await;
        for origin in [
            ClaimOrigin::AgentInferred,
            ClaimOrigin::Imported,
            ClaimOrigin::Migrated,
            ClaimOrigin::GeneratedSummary,
        ] {
            let item = MemoryItem::new(
                MemoryKind::ProjectFact,
                format!("Review origin {origin}"),
                "Review-origin memory should stay gated unless manually reviewed.",
                MemoryScope::project("engram"),
                origin,
                writer(),
            )
            .with_status(MemoryStatus::Active);

            let captured = service.capture_memory(item).await.unwrap();

            assert_eq!(captured.status, MemoryStatus::NeedsReview);
        }

        assert!(service.list_active_memory(None).await.unwrap().is_empty());
        assert_eq!(
            service
                .list_memory_needing_review(None)
                .await
                .unwrap()
                .len(),
            4
        );
    }

    #[tokio::test]
    async fn capture_policy_keeps_low_friction_agent_observations_active_without_evidence() {
        let service = setup_service().await;
        for kind in [
            MemoryKind::ProjectFact,
            MemoryKind::RepositoryFact,
            MemoryKind::TaskFact,
            MemoryKind::UserFact,
            MemoryKind::SessionInsight,
            MemoryKind::Handoff,
        ] {
            let item = MemoryItem::new(
                kind.clone(),
                format!("Low friction {kind}"),
                "Low-friction memory can be captured without evidence.",
                MemoryScope::project("engram"),
                ClaimOrigin::AgentObserved,
                writer(),
            );

            let captured = service.capture_memory(item).await.unwrap();

            assert_eq!(captured.status, MemoryStatus::Active);
            assert!(captured.evidence.is_empty());
        }

        assert_eq!(service.list_active_memory(None).await.unwrap().len(), 6);
        assert!(service
            .list_memory_needing_review(None)
            .await
            .unwrap()
            .is_empty());
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
    async fn promote_memory_activates_review_candidate_with_review_evidence() {
        let service = setup_service().await;
        let candidate = service
            .capture_memory(MemoryItem::new(
                MemoryKind::Decision,
                "Candidate decision",
                "Candidate durable guidance should require explicit promotion.",
                MemoryScope::project("engram"),
                ClaimOrigin::AgentInferred,
                writer(),
            ))
            .await
            .unwrap();
        assert_eq!(candidate.status, MemoryStatus::NeedsReview);

        let promoted = service
            .promote_memory(
                &candidate.id,
                "yuval",
                "Reviewed and accepted for future agents.",
            )
            .await
            .unwrap();

        assert_eq!(promoted.status, MemoryStatus::Active);
        assert!(promoted.evidence.iter().any(|evidence| evidence.kind
            == EvidenceKind::ManualReview
            && evidence.target == "yuval"
            && evidence
                .summary
                .as_deref()
                .is_some_and(|summary| summary.contains("accepted"))));
        assert_eq!(service.list_active_memory(None).await.unwrap().len(), 1);
        assert!(service
            .list_memory_needing_review(None)
            .await
            .unwrap()
            .is_empty());
    }

    #[tokio::test]
    async fn promote_memory_requires_review_candidate_and_rationale() {
        let service = setup_service().await;
        let active = service
            .capture_memory(memory_item("Already active"))
            .await
            .unwrap();

        let err = service
            .promote_memory(&active.id, "yuval", "Already active.")
            .await
            .unwrap_err();
        assert!(matches!(err, IndexError::InvalidState(_)));

        let review = service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Rule,
                    "Missing rationale",
                    "Review operations must carry reviewer rationale.",
                    MemoryScope::project("engram"),
                    ClaimOrigin::AgentInferred,
                    writer(),
                )
                .with_status(MemoryStatus::NeedsReview),
            )
            .await
            .unwrap();
        let err = service
            .promote_memory(&review.id, "yuval", "  ")
            .await
            .unwrap_err();
        assert!(matches!(err, IndexError::Parse(_)));
    }

    #[tokio::test]
    async fn reject_memory_keeps_review_candidate_auditable() {
        let service = setup_service().await;
        let candidate = service
            .capture_memory(MemoryItem::new(
                MemoryKind::Rule,
                "Bad candidate",
                "This candidate should not guide future work.",
                MemoryScope::project("engram"),
                ClaimOrigin::AgentInferred,
                writer(),
            ))
            .await
            .unwrap();

        let rejected = service
            .reject_memory(
                &candidate.id,
                "agent-reviewer",
                "Conflicts with current evidence.",
            )
            .await
            .unwrap();

        assert_eq!(rejected.status, MemoryStatus::Rejected);
        assert!(rejected.evidence.iter().any(|evidence| evidence.kind
            == EvidenceKind::ManualReview
            && evidence.target == "agent-reviewer"
            && evidence
                .summary
                .as_deref()
                .is_some_and(|summary| summary.contains("Conflicts"))));
        assert!(service.list_active_memory(None).await.unwrap().is_empty());
        assert!(service
            .list_memory_needing_review(None)
            .await
            .unwrap()
            .is_empty());
        assert_eq!(
            service
                .get_memory(&candidate.id)
                .await
                .unwrap()
                .unwrap()
                .status,
            MemoryStatus::Rejected
        );
    }

    #[tokio::test]
    async fn supersede_memory_promotes_replacement_and_hides_replaced_item() {
        let service = setup_service().await;
        let old = service
            .capture_memory(memory_item("Old decision"))
            .await
            .unwrap();
        let replacement = service
            .capture_memory(MemoryItem::new(
                MemoryKind::Decision,
                "Replacement decision",
                "Use the replacement decision after review.",
                MemoryScope::project("engram"),
                ClaimOrigin::AgentInferred,
                writer(),
            ))
            .await
            .unwrap();
        assert_eq!(replacement.status, MemoryStatus::NeedsReview);

        let (new_item, old_item) = service
            .supersede_memory(
                &replacement.id,
                &old.id,
                "yuval",
                "Replacement reflects current available evidence.",
            )
            .await
            .unwrap();

        assert_eq!(new_item.status, MemoryStatus::Active);
        assert!(new_item.supersedes.contains(&old.id));
        assert!(new_item.evidence.iter().any(|evidence| evidence.kind
            == EvidenceKind::ManualReview
            && evidence.target == "yuval"
            && evidence
                .summary
                .as_deref()
                .is_some_and(|summary| summary.contains(&old.id.to_string()))));
        assert_eq!(old_item.status, MemoryStatus::Superseded);
        assert!(old_item.evidence.iter().any(|evidence| evidence.kind
            == EvidenceKind::ManualReview
            && evidence.target == "yuval"
            && evidence
                .summary
                .as_deref()
                .is_some_and(|summary| summary.contains(&replacement.id.to_string()))));

        let active_items = service.list_active_memory(None).await.unwrap();
        assert_eq!(active_items.len(), 1);
        assert_eq!(active_items[0].id, replacement.id);
        assert_eq!(
            service.get_memory(&old.id).await.unwrap().unwrap().status,
            MemoryStatus::Superseded
        );
    }

    #[tokio::test]
    async fn supersede_memory_rejects_self_or_terminal_old_item() {
        let service = setup_service().await;
        let item = service.capture_memory(memory_item("Self")).await.unwrap();

        let err = service
            .supersede_memory(&item.id, &item.id, "yuval", "Impossible replacement.")
            .await
            .unwrap_err();
        assert!(matches!(err, IndexError::InvalidState(_)));

        let terminal = service
            .archive_memory(
                &item.id,
                "Retired before replacement.",
                Some("yuval".to_string()),
            )
            .await
            .unwrap();
        let replacement = service
            .capture_memory(MemoryItem::new(
                MemoryKind::Decision,
                "Replacement for terminal",
                "Terminal records should not be superseded again.",
                MemoryScope::project("engram"),
                ClaimOrigin::AgentInferred,
                writer(),
            ))
            .await
            .unwrap();
        let err = service
            .supersede_memory(
                &replacement.id,
                &terminal.id,
                "yuval",
                "Terminal old item should fail.",
            )
            .await
            .unwrap_err();
        assert!(matches!(err, IndexError::InvalidState(_)));
    }

    #[tokio::test]
    async fn archive_memory_retires_item_from_active_retrieval() {
        let service = setup_service().await;
        let item = service
            .capture_memory(memory_item("Retire me"))
            .await
            .unwrap();

        let archived = service
            .archive_memory(&item.id, "No longer applies.", Some("yuval".to_string()))
            .await
            .unwrap();

        assert_eq!(archived.status, MemoryStatus::Archived);
        assert_eq!(
            archived
                .archive
                .as_ref()
                .map(|archive| archive.reason.as_str()),
            Some("No longer applies.")
        );
        assert_eq!(
            archived
                .archive
                .as_ref()
                .and_then(|archive| archive.archived_by.as_deref()),
            Some("yuval")
        );
        assert!(service.list_active_memory(None).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn migration_viability_surfaces_reviewed_memory_in_orient_and_search() {
        let (memory_service, search_service, work_repo) =
            setup_migration_viability_services().await;
        let project = Project::new("engram");
        work_repo.create_project(&project).await.unwrap();
        work_repo
            .add_project_observation(
                &ProjectObservation::new(
                    project.id,
                    "Agents should request Memory OS orientation before substantial Engram implementation work.",
                )
                .with_key("decisions.memory-orientation"),
            )
            .await
            .unwrap();

        let inventory = memory_service
            .migration_inventory(MigrationInventoryOptions::all())
            .await
            .unwrap();
        assert_eq!(inventory.sources_scanned, 1);
        assert_eq!(inventory.returned_candidates, 1);

        let review_dir = tempdir().unwrap();
        let export = memory_service
            .export_migration_review(review_dir.path(), MigrationInventoryOptions::all())
            .await
            .unwrap();
        let accepted_path = accept_first_migration_candidate(review_dir.path(), &export);
        let status = memory_service
            .migration_review_status(review_dir.path())
            .await
            .unwrap();
        assert!(status.ready_to_apply);
        assert_eq!(status.planned_count, 1);
        assert_eq!(status.accepted_files, vec![accepted_path.clone()]);

        let apply = memory_service
            .apply_migration_review(
                review_dir.path(),
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
        let migrated = apply.written_items[0].clone();
        assert_eq!(migrated.status, MemoryStatus::Active);
        assert_eq!(migrated.kind, MemoryKind::Decision);
        assert_eq!(migrated.origin, ClaimOrigin::Migrated);
        assert!(migrated.tags.iter().any(|tag| tag == "migration"));
        assert!(migrated.tags.iter().any(|tag| tag == "migration-reviewed"));
        assert!(migrated
            .tags
            .iter()
            .any(|tag| tag.starts_with("migration-source:project_observation:")));
        assert!(migrated
            .evidence
            .iter()
            .any(|evidence| evidence.kind == EvidenceKind::Observation));
        assert!(migrated
            .evidence
            .iter()
            .any(|evidence| evidence.kind == EvidenceKind::ManualReview
                && evidence.target == accepted_path));
        let metadata = migrated.trust_metadata();
        assert_eq!(metadata.review_state, MemoryReviewState::Reviewed);
        assert_eq!(metadata.evidence_count, 2);

        let packet = memory_service
            .orient(OrientInput {
                project: Some("engram".to_string()),
                prompt: Some("continue substantial implementation work".to_string()),
                include_recent_commits: false,
                limit: Some(10),
                ..OrientInput::default()
            })
            .await
            .unwrap();
        assert!(packet
            .active_decisions
            .iter()
            .any(|item| item.id == migrated.id));
        assert!(packet
            .memory_metadata
            .iter()
            .any(|metadata| metadata.memory_id == migrated.id
                && metadata.review_state == MemoryReviewState::Reviewed));
        assert!(packet.context_pack.contains("Memory OS orientation"));

        let search_results = search_service
            .search_with_options(
                "Memory OS orientation substantial implementation",
                10,
                Some(0.0),
                Some(&[SearchLayer::Memory]),
                SearchOptions {
                    project: Some("engram".to_string()),
                    cwd: None,
                },
            )
            .await
            .unwrap();
        let migrated_result = search_results
            .iter()
            .find(|result| result.id == migrated.id.to_string())
            .expect("migrated memory should be searchable");
        assert_eq!(
            migrated_result
                .memory_metadata
                .as_ref()
                .map(|metadata| metadata.review_state),
            Some(MemoryReviewState::Reviewed)
        );

        let second_apply = memory_service
            .apply_migration_review(
                review_dir.path(),
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
            memory_service.list_memory(None, None).await.unwrap().len(),
            1
        );
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
                external_session_id: None,
                intent: None,
                scenario_id: None,
                arm: None,
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
        assert!(!packet.brain_loop.degraded);
        assert!(packet.brain_loop.top_items.len() <= BRAIN_LOOP_TOP_ITEM_LIMIT);
        assert!(packet
            .brain_loop
            .compiled_context
            .contains("Project decision"));
        assert!(packet
            .brain_loop
            .compiled_context
            .contains("Global preference"));
        assert!(packet.brain_loop.top_items.iter().any(|item| {
            item.kind == MemoryKind::Preference
                && item.title == "Global preference"
                && item.trust.memory_id == item.id
        }));
    }

    #[tokio::test]
    async fn orient_resume_session_prioritizes_latest_current_plan_and_suppresses_older_ones() {
        let service = setup_service().await;
        let mut old = MemoryItem::new(
            MemoryKind::Decision,
            "User confirmed post-capture resume probe looked great",
            "Older resume-continuity current plan that should not lead a new resume.",
            MemoryScope::project("engram"),
            ClaimOrigin::ToolResult,
            writer(),
        )
        .with_status(MemoryStatus::Active)
        .with_evidence(EvidenceRef::new(EvidenceKind::ToolCall, "old-current-plan"))
        .with_tag(CURRENT_PLAN_TAG);
        old.updated_at = OffsetDateTime::now_utc() - time::Duration::hours(1);
        let old = service.capture_memory(old).await.unwrap();

        let mut latest = MemoryItem::new(
            MemoryKind::Decision,
            "Three-intent regression points to current-plan supersession",
            "Latest current plan: implement current-plan freshness before ranking or M6 changes.",
            MemoryScope::project("engram"),
            ClaimOrigin::ToolResult,
            writer(),
        )
        .with_status(MemoryStatus::Active)
        .with_evidence(EvidenceRef::new(
            EvidenceKind::ToolCall,
            "latest-current-plan",
        ))
        .with_tag(CURRENT_PLAN_TAG);
        latest.updated_at = OffsetDateTime::now_utc();
        let latest = service.capture_memory(latest).await.unwrap();
        service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Rule,
                    "Brain Harness work follows research method",
                    "Brain Harness development should be run through explicit research \
                     questions, evidence levels, falsifiers, decision gates, and claim-ledger \
                     updates.",
                    MemoryScope::project("engram"),
                    ClaimOrigin::UserStated,
                    writer(),
                )
                .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "unit-test")),
            )
            .await
            .unwrap();

        let packet = service
            .orient(OrientInput {
                project: Some("engram".to_string()),
                prompt: Some(
                    "I just restarted Codex and want to resume Engram Brain Harness work. \
                     What is the current plan, current gate, and next action?"
                        .to_string(),
                ),
                intent: Some(BrainHarnessIntent::ResumeSession),
                limit: Some(10),
                ..OrientInput::default()
            })
            .await
            .unwrap();

        assert_eq!(
            packet.active_decisions.first().map(|item| item.id),
            Some(latest.id)
        );
        assert!(!packet.active_decisions.iter().any(|item| item.id == old.id));
        assert_eq!(
            packet.brain_loop.top_items.first().map(|item| item.id),
            Some(latest.id)
        );
        assert_eq!(
            service.get_memory(&old.id).await.unwrap().unwrap().status,
            MemoryStatus::Active,
            "resume guard should not mutate existing records"
        );
    }

    #[tokio::test]
    async fn orient_brain_loop_keeps_top_items_bounded() {
        let service = setup_service().await;
        for index in 0..8 {
            service
                .capture_memory(MemoryItem::new(
                    MemoryKind::Preference,
                    format!("Preference {index}"),
                    "Bounded brain loop context should not grow with every matching memory item.",
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
                limit: Some(10),
                ..OrientInput::default()
            })
            .await
            .unwrap();

        assert_eq!(packet.preferences.len(), 8);
        assert_eq!(packet.brain_loop.top_items.len(), BRAIN_LOOP_TOP_ITEM_LIMIT);
        assert!(packet
            .brain_loop
            .top_items
            .iter()
            .all(|item| item.kind == MemoryKind::Preference));
        assert!(packet.brain_loop.compiled_context.contains("Brain Loop v1"));
    }

    #[tokio::test]
    async fn orient_brain_loop_balances_memory_buckets() {
        let service = setup_service().await;
        for index in 0..8 {
            service
                .capture_memory(MemoryItem::new(
                    MemoryKind::Preference,
                    format!("Preference {index}"),
                    "Repeated preferences should not starve other relevant memory buckets.",
                    MemoryScope::project("engram"),
                    ClaimOrigin::UserStated,
                    writer(),
                ))
                .await
                .unwrap();
        }
        service
            .capture_memory(memory_item("Architecture decision"))
            .await
            .unwrap();

        let packet = service
            .orient(OrientInput {
                project: Some("engram".to_string()),
                limit: Some(10),
                ..OrientInput::default()
            })
            .await
            .unwrap();

        assert_eq!(packet.brain_loop.top_items.len(), BRAIN_LOOP_TOP_ITEM_LIMIT);
        assert!(
            packet
                .brain_loop
                .top_items
                .iter()
                .any(|item| item.kind == MemoryKind::Decision
                    && item.title == "Architecture decision")
        );
    }

    #[tokio::test]
    async fn orient_brain_loop_prioritizes_prompt_specific_reviewed_decision() {
        let service = setup_service().await;
        service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Limitation,
                    "Daemon command routing limitation",
                    "Known operational limitation for daemon startup and direct command routing.",
                    MemoryScope::project("engram"),
                    ClaimOrigin::ToolResult,
                    writer(),
                )
                .with_evidence(EvidenceRef::new(EvidenceKind::ToolCall, "daemon smoke")),
            )
            .await
            .unwrap();
        service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Decision,
                    "Promote durable observations into MemoryItems",
                    "Use promote_observation to graduate keyed entity observations into reviewed \
                     MemoryItems when they should affect orient or Brain Loop output. Do not put \
                     raw entity observations directly in the hot orientation path.",
                    MemoryScope::project("engram"),
                    ClaimOrigin::ToolResult,
                    writer(),
                )
                .with_evidence(EvidenceRef::new(
                    EvidenceKind::Observation,
                    "architecture.observation-promotion-memoryitems",
                ))
                .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "unit-test")),
            )
            .await
            .unwrap();

        let packet = service
            .orient(OrientInput {
                project: Some("engram".to_string()),
                prompt: Some(
                    "When should Engram promote keyed entity observations into durable \
                     MemoryItems instead of adding raw observations to orient Brain Loop?"
                        .to_string(),
                ),
                limit: Some(10),
                ..OrientInput::default()
            })
            .await
            .unwrap();

        assert_eq!(
            packet
                .active_decisions
                .first()
                .map(|item| item.title.as_str()),
            Some("Promote durable observations into MemoryItems")
        );
        assert_eq!(
            packet
                .brain_loop
                .top_items
                .first()
                .map(|item| item.title.as_str()),
            Some("Promote durable observations into MemoryItems")
        );
        assert!(packet
            .brain_loop
            .top_items
            .iter()
            .any(|item| item.title == "Daemon command routing limitation"));
    }

    #[tokio::test]
    async fn orient_prioritizes_reviewed_gate_over_broad_current_plan_for_specific_prompt() {
        let service = setup_service().await;
        service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Decision,
                    "Migration Must Be Review-Gated",
                    "Memory OS migration for Engram must be review-gated. Existing Engram data \
                     can be valuable, but migrated records must pass inventory, source \
                     classification, staleness scoring, quarantine for uncertainty, human batch \
                     review, provenance preservation, and a knowledge commit before becoming \
                     active memory. Do not bulk-write migrated memory from raw Engram data or \
                     vault export alone.",
                    MemoryScope::project("engram"),
                    ClaimOrigin::Migrated,
                    writer(),
                )
                .with_confidence(0.70)
                .with_status(MemoryStatus::Active)
                .with_evidence(EvidenceRef::new(
                    EvidenceKind::SessionEvent,
                    "migration-gate-session",
                ))
                .with_evidence(
                    EvidenceRef::new(EvidenceKind::ManualReview, "unit-test")
                        .with_summary("User accepted migration gate as reviewed safety guidance."),
                ),
            )
            .await
            .unwrap();
        service
            .capture_current_plan(current_plan_input(
                "engram",
                "Current-plan supersession validated; next measure topic noise",
                "Current-plan supersession passed. Next high-confidence step is not M6, graph, \
                 or obligations; it is a narrow same-project topic-noise calibration so \
                 prompt-specific reviewed safety gates, especially the migration gate, outrank \
                 broad current-plan context.",
            ))
            .await
            .unwrap();
        service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Decision,
                    "Agent-native harness should use generalized obligations, not document-only checks",
                    "Engram should model agent-native behavior as generated obligations from \
                     session cues: durable document writes require ingest/register/record/skip \
                     disposition; failed tool calls require schema/help inspection; design/code \
                     tasks require source-reading obligations over AGENTS, README, relevant docs, \
                     and existing code before asserting behavior.",
                    MemoryScope::project("engram"),
                    ClaimOrigin::UserStated,
                    writer(),
                )
                .with_evidence(EvidenceRef::new(
                    EvidenceKind::ManualReview,
                    "broad-obligations-memory",
                )),
            )
            .await
            .unwrap();

        let packet = service
            .orient(OrientInput {
                project: Some("engram".to_string()),
                prompt: Some(
                    "Before changing ranking, graph, obligations, or applying M6 write/apply \
                     migration behavior, should Engram proceed with M6 write/apply now? What \
                     safety gate applies?"
                        .to_string(),
                ),
                intent: Some(BrainHarnessIntent::VerifyDecision),
                limit: Some(10),
                ..OrientInput::default()
            })
            .await
            .unwrap();

        assert_eq!(
            packet
                .active_decisions
                .first()
                .map(|item| item.title.as_str()),
            Some("Migration Must Be Review-Gated")
        );
        assert_eq!(
            packet
                .brain_loop
                .top_items
                .first()
                .map(|item| item.title.as_str()),
            Some("Migration Must Be Review-Gated")
        );
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
    async fn orient_can_include_recent_current_branch_git_commits() {
        if !git_available() {
            return;
        }
        let (service, repo) = setup_service_with_repository_repo().await;
        let dir = tempdir().unwrap();
        run_git(dir.path(), &["init"]);
        fs::write(dir.path().join("README.md"), "initial\n").unwrap();
        commit_all(dir.path(), "Initial repository context");
        fs::create_dir_all(dir.path().join("docs")).unwrap();
        fs::write(
            dir.path().join("docs/BRAIN_HARNESS_DOGFOOD_PROTOCOL.md"),
            "# Brain Harness Dogfood Protocol\n",
        )
        .unwrap();
        commit_all(dir.path(), "Add brain harness dogfood protocol");

        let repository = GitRepository::new("engram");
        repo.save_repository(&repository).await.unwrap();
        repo.save_checkout(
            &LocalCheckout::new(dir.path().display().to_string()).with_repository(repository.id),
        )
        .await
        .unwrap();
        repo.save_project_link(&ProjectRepositoryLink::new(
            "engram",
            repository.id,
            ProjectRepositoryRole::Primary,
        ))
        .await
        .unwrap();

        let packet = service
            .orient(OrientInput {
                cwd: Some(dir.path().display().to_string()),
                project: Some("engram".to_string()),
                prompt: Some("resume after adding the dogfood protocol".to_string()),
                include_recent_commits: true,
                limit: Some(10),
                ..OrientInput::default()
            })
            .await
            .unwrap();

        let recent_commits = &packet
            .repository_context
            .as_ref()
            .expect("repository context should be resolved")
            .recent_commits;
        assert_eq!(
            recent_commits.first().map(|commit| commit.summary.as_str()),
            Some("Add brain harness dogfood protocol")
        );
        assert!(recent_commits[0]
            .changed_paths
            .iter()
            .any(|path| path == "docs/BRAIN_HARNESS_DOGFOOD_PROTOCOL.md"));
        assert!(packet.context_pack.contains("## Repository Context"));
        assert!(packet.context_pack.contains("Recent Git commits"));
        assert!(packet
            .context_pack
            .contains("Add brain harness dogfood protocol"));
        assert!(packet
            .context_pack
            .contains("docs/BRAIN_HARNESS_DOGFOOD_PROTOCOL.md"));

        let without_commits = service
            .orient(OrientInput {
                cwd: Some(dir.path().display().to_string()),
                project: Some("engram".to_string()),
                include_recent_commits: false,
                limit: Some(10),
                ..OrientInput::default()
            })
            .await
            .unwrap();
        assert!(without_commits
            .repository_context
            .as_ref()
            .expect("repository context should be resolved")
            .recent_commits
            .is_empty());
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
