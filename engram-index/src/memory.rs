//! Memory OS service.
//!
//! Provides the first service layer over source-grounded memory items and
//! knowledge commits. MCP and CLI surfaces should delegate to this layer.

use crate::digest::{
    apply_digest_extraction_review_batch, build_digest_extraction_commit,
    DigestExtractionReviewApply, DigestExtractionReviewApplyOptions,
};
use crate::error::{IndexError, IndexResult};
use crate::memory_ranker::{
    is_open_ended_plan_work_prompt, memory_scope_matches, rank_memory_item, rank_memory_items,
    MemoryRankContext,
};
use crate::migration::{
    MigrationInventory, MigrationInventoryOptions, MigrationReviewApply,
    MigrationReviewApplyOptions, MigrationReviewExport, MigrationReviewStatus, MigrationService,
};
use crate::repository::{
    normalize_remote_reference, refresh_checkout_git_state, resolve_matching_components,
    RepositoryService,
};
use crate::vault::{
    init_memory_vault, inspect_memory_vault, read_memory_vault_page, write_memory_vault,
    MemoryVaultExport, MemoryVaultInit, MemoryVaultPage, MemoryVaultStatus,
    RepositoryVaultSnapshot,
};
use engram_core::entity::Observation;
use engram_core::id::Id;
use engram_core::memory::{
    ClaimOrigin, CorrectionProposal, CorrectionProposalStatus, EvidenceKind, EvidenceRef, Harness,
    KnowledgeCommit, MemoryChange, MemoryChangeType, MemoryCursor, MemoryItem, MemoryKind,
    MemoryReviewState, MemoryScope, MemoryStatus, MemoryTrustMetadata, ProcedureCard,
    ProcedurePrerequisite, ProcedurePrerequisiteSource, WriterProvenance,
};
use engram_core::repository::{ProjectRepositoryLink, RecentGitCommit, RepositoryContext};
use engram_core::session::{Event, EventType};
use engram_core::telemetry::{BrainHarnessIntent, BrainHarnessOperation, BrainHarnessTrace};
use engram_store::{Db, MemoryRepo, RepositoryRepo, SessionRepo, TelemetryRepo};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use time::OffsetDateTime;
use tracing::info;

const BRAIN_LOOP_TOP_ITEM_LIMIT: usize = 5;
const BRAIN_LOOP_SUMMARY_CHAR_LIMIT: usize = 240;
const ORIENT_HOT_CONTEXT_ITEM_LIMIT: usize = 3;
const ORIENT_RECENT_GIT_COMMIT_LIMIT: usize = 5;
const ORIENT_RECENT_GIT_COMMIT_PATH_LIMIT: usize = 8;
const CURRENT_PLAN_TAG: &str = "current-plan";
const MIN_PROCEDURE_TEXT_MATCH_SCORE: f32 = 0.75;
const MAX_PROCEDURE_CANDIDATES_TO_EVALUATE: usize = 20;
const MAX_PROCEDURE_PREREQUISITES: usize = 16;
const MAX_CONDITION_SOURCE_BYTES: u64 = 64 * 1024;
const MAX_OPERATION_EVIDENCE_INDEX_BYTES: usize = 64 * 1024;
const MAX_OPERATION_EVIDENCE_CANDIDATES: usize = 128;
const MAX_PROCEDURE_QUERY_CHARS: usize = 512;

/// Machine-readable receipt produced by a successful procedure verification command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureVerificationReceipt {
    /// Exact verification command that ran.
    pub command: String,
    /// Observed process exit code.
    pub exit_code: i32,
    /// Captured output used only during verification; Engram stores its file hash, not this body.
    pub output: String,
    /// Exact environment conditions observed during the successful run.
    #[serde(default)]
    pub conditions: BTreeMap<String, String>,
}

/// Conditions and scope for retrieving an applicable procedure.
#[derive(Debug, Clone, Default)]
pub struct ProcedureMatchInput {
    /// Bounded task intent or failure text used only for retrieval.
    pub query: String,
    /// Explicit project scope.
    pub project: Option<String>,
    /// Current working directory.
    pub cwd: Option<String>,
    /// Exact caller-observed environment conditions.
    pub conditions: BTreeMap<String, String>,
    /// Maximum applicable procedures to return.
    pub limit: Option<usize>,
}

/// Applicability decision for one procedure candidate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureApplicability {
    /// Memory item ID.
    pub memory_id: Id,
    /// Procedure title.
    pub title: String,
    /// Whether every scope, freshness, condition, and proof check passed.
    pub applicable: bool,
    /// Deterministic reasons for applying or abstaining.
    pub reasons: Vec<String>,
    /// Condition keys the caller must resolve from authoritative local sources before retrying.
    pub unresolved_condition_keys: Vec<String>,
    /// Trusted source observations made by Engram for source-backed prerequisites.
    pub condition_observations: Vec<ProcedureConditionObservation>,
}

/// Outcome of one deterministic checkout-local prerequisite observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcedureConditionObservationStatus {
    /// The observed scalar matched the verified prerequisite.
    Matched,
    /// The observed scalar differed from the verified prerequisite.
    Mismatched,
    /// The source could not be observed safely.
    Unavailable,
}

/// Auditable evidence for a source-backed prerequisite without exposing either scalar value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureConditionObservation {
    /// Stable condition key.
    pub condition_key: String,
    /// Declarative source read from the current checkout.
    pub source: ProcedurePrerequisiteSource,
    /// Match outcome.
    pub status: ProcedureConditionObservationStatus,
    /// SHA-256 of the complete current source file when it was read successfully.
    pub source_sha256: Option<String>,
    /// Bounded explanation that never includes the observed or expected scalar.
    pub detail: String,
}

/// One bounded checkout-local source candidate for resolving a procedure no-result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationEvidenceCandidate {
    /// Git-tracked path relative to the current checkout root.
    pub path: String,
    /// Canonical absolute path for one exact host read in the current checkout.
    pub resolved_path: String,
    /// SHA-256 of the complete current file without returning its contents.
    pub source_sha256: String,
    /// Bounded explanation of why this path was selected.
    pub reason: String,
    /// Whether the host must read this source before returning a final abstention.
    pub required_before_final_abstention: bool,
    /// Whether repository-local evidence collection remains allowed while project scope is unresolved.
    pub allowed_when_project_requires_confirmation: bool,
    /// Whether this evidence candidate authorizes executing a remembered procedure.
    pub authorizes_procedure_execution: bool,
}

/// Procedure retrieval report with explicit abstention diagnostics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcedureMatchReport {
    /// Validated bounded retrieval query.
    pub query: String,
    /// Compact checkout identity and separate project-authorization boundary.
    pub identity: OrientationIdentity,
    /// Project/repository identity resolution performed at the procedure boundary.
    pub resolution: OrientationResolution,
    /// Applicable, verified procedures only.
    pub procedures: Vec<MemoryItem>,
    /// Decisions for every query-relevant candidate considered.
    pub diagnostics: Vec<ProcedureApplicability>,
    /// Whether no procedure was safe to apply.
    pub abstained: bool,
    /// Short outcome statement.
    pub message: String,
    /// Deduplicated condition keys that must be observed before a safe retry.
    pub required_condition_keys: Vec<String>,
    /// Bounded next actions that do not reveal the verified condition values.
    pub next_actions: Vec<String>,
    /// Unique tracked runbook candidate for one direct read, when deterministically resolvable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_operation_evidence: Option<OperationEvidenceCandidate>,
    /// Root of the currently resolved checkout, when repository identity is available.
    pub current_checkout_root: Option<String>,
    /// Safety rule separating stored checkout provenance from the current execution location.
    pub execution_guidance: Option<String>,
}

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
    /// Optional exact task used for authorization and relevance scoring.
    pub task: Option<String>,
    /// Optional cwd used for repository relevance scoring.
    pub cwd: Option<String>,
    /// Optional prompt/query used for keyword scoring.
    pub query: Option<String>,
    /// Caller intent for telemetry correlation.
    pub intent: Option<BrainHarnessIntent>,
    /// Optional host/application session label for telemetry correlation.
    pub external_session_id: Option<String>,
    /// Whether returned MemoryItems must match the supplied project/task/cwd boundary.
    pub enforce_scope: bool,
    /// Whether knowledge commits should be withheld from the response while still advancing the
    /// opaque cursor across them.
    pub omit_commits: bool,
}

/// Internal deletion result for a permanently forgotten memory item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryForgetReport {
    /// Exact memory item requested for deletion.
    pub id: Id,
    /// Whether the canonical memory item existed and was deleted.
    pub deleted: bool,
    /// Other memory items scrubbed of links or evidence naming the forgotten item.
    pub memory_items_updated: usize,
    /// Knowledge commits stripped of the forgotten item's linked change and message.
    pub commits_redacted: usize,
    /// Retrieval traces deleted because they returned the forgotten item.
    pub traces_deleted: usize,
    /// Feedback records deleted because they referenced the forgotten item or a deleted trace.
    pub feedback_deleted: usize,
    /// Typed correction-proposal records deleted because they referenced the forgotten item.
    pub correction_proposals_deleted: usize,
    /// Inactive pending replacement items deleted with a forgotten obsolete item.
    pub proposal_replacements_deleted: usize,
    /// Surviving obsolete items atomically unlocked when a pending replacement was forgotten.
    pub proposal_obsoletes_unlocked: usize,
    /// Whether this call resumed cleanup from a durable post-deletion receipt.
    pub cleanup_resumed: bool,
    /// Whether projection counts cover the complete forget operation rather than this retry only.
    pub projection_counts_complete: bool,
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
    /// Project was derived from a caller-supplied, validated task.
    Task,
    /// Resolved from a component-scoped repository link.
    ComponentLink,
    /// Resolved from an unambiguous repository link.
    RepositoryLink,
    /// No project was selected.
    Unresolved,
}

/// Whether a project name is authorized for scoped retrieval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrientationProjectStatus {
    /// The caller, task, or an exact topology link authorized this project.
    Authorized,
    /// Repository identity is known, but the project needs an explicit user decision.
    RequiresConfirmation,
    /// Neither an authorized project nor a material project candidate is available.
    Unavailable,
}

/// Compact, credential-free repository identity for an orientation boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrientationRepositoryIdentity {
    /// Engram topology record correlated with this identity.
    pub repository_id: Id,
    /// Canonical repository name.
    pub name: String,
    /// Credential-free normalized remote, when known.
    pub normalized_remote: Option<String>,
    /// Engram checkout record correlated with the current root, when known.
    pub checkout_id: Option<Id>,
    /// Absolute root of the current checkout, when known.
    pub checkout_root: Option<String>,
    /// Last Git HEAD recorded for the resolved checkout, when available.
    pub head_sha: Option<String>,
}

/// Project authorization kept distinct from repository and component identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrientationProjectIdentity {
    /// Authorization state for project-scoped retrieval.
    pub status: OrientationProjectStatus,
    /// Authorized project name, if any.
    pub name: Option<String>,
    /// Exact source of the authorization decision.
    pub source: OrientationResolutionSource,
    /// Candidate names that require confirmation or explain an explicit override.
    pub candidates: Vec<String>,
    /// Engram topology links that support the selected project or candidates.
    pub project_link_ids: Vec<Id>,
    /// Bounded explanation of the authorization decision.
    pub reason: String,
    /// Material ambiguity, when confirmation is required.
    pub ambiguity: Option<String>,
}

/// Deterministic identity boundary shared by orientation and procedure matching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrientationIdentity {
    /// Current repository and checkout identity, independent of project authorization.
    pub repository: Option<OrientationRepositoryIdentity>,
    /// Project-scoped retrieval authorization.
    pub project: OrientationProjectIdentity,
    /// Git-tracked component identities containing the current path.
    pub components: Vec<OrientationComponentEvidence>,
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
    /// Credential-free normalized repository remote matched from cwd, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository_remote: Option<String>,
    /// Component names matched from cwd.
    pub component_names: Vec<String>,
    /// Source evidence for component identities matched from cwd.
    #[serde(default)]
    pub component_evidence: Vec<OrientationComponentEvidence>,
    /// Project candidates considered.
    pub project_candidates: Vec<String>,
    /// Ambiguity details, if any.
    pub ambiguity: Option<String>,
}

/// Evidence explaining a component identity returned by orientation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrientationComponentEvidence {
    /// Component name.
    pub name: String,
    /// Repository-relative component path.
    pub component_path: String,
    /// Checkout-relative authoritative source path, when live-derived.
    pub source_path: Option<String>,
    /// SHA-256 of the authoritative source observed for this orientation.
    pub source_sha256: Option<String>,
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
            repository_remote: repository_context
                .and_then(|context| context.repository.remote_url.as_deref())
                .and_then(normalize_remote_reference),
            component_names: component_names(repository_context),
            component_evidence: component_evidence(repository_context),
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
    /// Exact task selected for task-scoped retrieval, when supplied.
    pub task: Option<String>,
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
    /// Compact checkout identity and separate project-authorization boundary.
    pub identity: OrientationIdentity,
    /// Structured resolution explaining which project/repository context was selected.
    pub resolution: OrientationResolution,
    /// Repository topology resolved from cwd, when known.
    pub repository_context: Option<RepositoryContext>,
    /// Memory cursor for later changes_since checks.
    pub memory_cursor: MemoryCursor,
    /// Stable IDs for high-priority hot-context memory items.
    pub hot_context_ids: Vec<Id>,
    /// Compact hot-context memory items surfaced before the large context pack.
    pub hot_context_items: Vec<BrainLoopItem>,
    /// Memory IDs selected by orient as candidates for telemetry `used_memory_ids`.
    pub used_memory_candidate_ids: Vec<Id>,
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
    /// Active handoffs relevant to this scope.
    pub handoffs: Vec<MemoryItem>,
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
    db: Db,
    repo: MemoryRepo,
    telemetry_repo: TelemetryRepo,
    repository_repo: RepositoryRepo,
    session_repo: SessionRepo,
    migration_service: MigrationService,
}

/// Agent-supplied content for a correction proposal.
///
/// Kind, scope, lifecycle status, origin, tags, and confidence are derived from the obsolete item
/// by the service. A structured procedure card is accepted only when the obsolete item is itself a
/// procedure. Caller-supplied verification state is rejected; the replacement must pass the
/// dedicated verify-while-inactive transition before it can be applied.
#[derive(Debug, Clone)]
pub struct CorrectionProposalInput {
    /// Exact active item the proposal would replace.
    pub obsolete_id: Id,
    /// Proposed replacement title.
    pub title: String,
    /// Proposed replacement content.
    pub content: String,
    /// Agent provenance. The service fixes the actor to `agent`.
    pub writer: WriterProvenance,
    /// Source evidence for the proposed replacement.
    pub evidence: Vec<EvidenceRef>,
    /// Structured replacement procedure when correcting procedure memory.
    pub procedure: Option<ProcedureCard>,
}

/// Full-profile inspection view for one durable correction proposal and its exact pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectionProposalInspection {
    /// Server-minted proposal record, including canonical and applied digests.
    pub proposal: CorrectionProposal,
    /// Proposed replacement item.
    pub replacement: MemoryItem,
    /// Item the proposal would or did supersede.
    pub obsolete: MemoryItem,
}

impl MemoryService {
    /// Create a new memory service.
    pub fn new(db: Db) -> Self {
        Self {
            db: db.clone(),
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

    /// Verify that the backing datastore can commit a reversible write transaction.
    pub async fn probe_storage_writable(&self) -> IndexResult<()> {
        engram_store::probe_writable(&self.db).await?;
        Ok(())
    }

    /// Persist a memory item after domain and capture-policy validation.
    pub async fn capture_memory(&self, item: MemoryItem) -> IndexResult<MemoryItem> {
        Self::reject_proposal_replacement_mutation(&item, "capture")?;
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

        if let Some(locked) = self
            .repo
            .list_memory_items(Some(MemoryStatus::Active), None)
            .await?
            .into_iter()
            .find(|candidate| {
                candidate.pending_correction_proposal_id.is_some()
                    && is_current_plan_item(candidate)
                    && current_plan_scope_key(&candidate.scope)
                        == current_plan_scope_key(&input.scope)
            })
        {
            return Err(IndexError::InvalidState(format!(
                "cannot capture a new current plan while same-scope memory {} is locked by pending correction proposal {}; apply the proposal or forget its pending replacement first",
                locked.id,
                locked
                    .pending_correction_proposal_id
                    .expect("locked current-plan item has a proposal ID")
            )));
        }

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

    /// Promote a review candidate into active memory with an unverified reviewer assertion.
    ///
    /// The assertion is retained for auditability but does not confer human-review authority.
    pub async fn promote_memory(
        &self,
        id: &Id,
        reviewer: impl Into<String>,
        rationale: impl Into<String>,
    ) -> IndexResult<MemoryItem> {
        let item = self.get_required_memory(id).await?;
        self.reject_pending_correction_pair_mutation(&item, "promote")
            .await?;
        if item.status != MemoryStatus::NeedsReview {
            return Err(IndexError::InvalidState(format!(
                "memory item {id} is not needs_review (status: {})",
                item.status
            )));
        }

        let expected = item.clone();
        let item = item
            .with_status(MemoryStatus::Active)
            .with_evidence(review_evidence(reviewer, rationale)?);
        self.repo
            .save_memory_item_if_unchanged(&expected, &item)
            .await?;
        Ok(item)
    }

    /// Reject a review candidate while keeping its unverified reviewer assertion auditable.
    pub async fn reject_memory(
        &self,
        id: &Id,
        reviewer: impl Into<String>,
        rationale: impl Into<String>,
    ) -> IndexResult<MemoryItem> {
        let item = self.get_required_memory(id).await?;
        self.reject_pending_correction_pair_mutation(&item, "reject")
            .await?;
        if item.status != MemoryStatus::NeedsReview {
            return Err(IndexError::InvalidState(format!(
                "memory item {id} is not needs_review (status: {})",
                item.status
            )));
        }

        let expected = item.clone();
        let item = item
            .with_status(MemoryStatus::Rejected)
            .with_evidence(review_evidence(reviewer, rationale)?);
        self.repo
            .save_memory_item_if_unchanged(&expected, &item)
            .await?;
        Ok(item)
    }

    /// Promote a replacement memory item and mark the replaced item as superseded.
    /// Reviewer labels on this caller-controlled path are unverified assertions.
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
        self.reject_pending_correction_pair_mutation(&new_item, "supersede")
            .await?;
        self.reject_pending_correction_pair_mutation(&old_item, "supersede")
            .await?;
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

        let expected_new_item = new_item.clone();
        let expected_old_item = old_item.clone();
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

        self.repo
            .save_memory_correction(&expected_new_item, &expected_old_item, &new_item, &old_item)
            .await?;
        Ok((new_item, old_item))
    }

    /// Create a digest-bound, inactive replacement proposal for one exact active item.
    ///
    /// The proposal is agent-authored, remains `needs_review`, and does not affect retrieval until
    /// an operator applies the exact bound digest through `apply_correction`.
    pub async fn propose_correction(
        &self,
        input: CorrectionProposalInput,
        project: Option<&str>,
        task: Option<&str>,
        cwd: Option<&str>,
    ) -> IndexResult<(CorrectionProposal, MemoryItem, MemoryItem)> {
        let project = normalized_boundary_value(project);
        let task = normalized_boundary_value(task);
        let cwd = normalized_boundary_value(cwd);
        let obsolete = self.get_required_memory(&input.obsolete_id).await?;
        Self::authorize_correction_item(&obsolete, "obsolete", project, task, cwd)?;
        if obsolete.status != MemoryStatus::Active {
            return Err(IndexError::InvalidState(format!(
                "memory item {} cannot receive a correction proposal from status {}",
                obsolete.id, obsolete.status
            )));
        }
        match (&obsolete.kind, &input.procedure) {
            (MemoryKind::Procedure, None) => {
                return Err(IndexError::InvalidState(
                    "procedure correction proposals require structured replacement procedure details"
                        .to_string(),
                ));
            }
            (MemoryKind::Procedure, Some(procedure))
                if procedure.verification.evidence_path.is_some()
                    || procedure.verification.evidence_sha256.is_some()
                    || procedure.verification.verified_at.is_some()
                    || procedure.expires_at.is_some() =>
            {
                return Err(IndexError::InvalidState(
                    "procedure correction proposals cannot supply verification proof or expiry; use verify_correction_procedure on the inactive replacement"
                        .to_string(),
                ));
            }
            (MemoryKind::Procedure, Some(_)) | (_, None) => {}
            (_, Some(_)) => {
                return Err(IndexError::InvalidState(
                    "structured replacement procedure details require an obsolete procedure"
                        .to_string(),
                ));
            }
        }
        if let Some(existing) = self
            .repo
            .list_correction_proposals_for_memory(&obsolete.id)
            .await?
            .into_iter()
            .find(|proposal| {
                proposal.status == CorrectionProposalStatus::Pending
                    && proposal.obsolete_id == obsolete.id
            })
        {
            return Err(IndexError::InvalidState(format!(
                "memory item {} already has pending correction proposal {}",
                obsolete.id, existing.id
            )));
        }
        if input.evidence.is_empty() {
            return Err(IndexError::InvalidState(
                "correction proposal requires source evidence".to_string(),
            ));
        }
        if has_manual_review_evidence_refs(&input.evidence) {
            return Err(IndexError::InvalidState(
                "correction proposal cannot carry manual_review evidence".to_string(),
            ));
        }

        let mut writer = input.writer;
        writer.actor = "agent".to_string();
        let mut replacement = MemoryItem::new(
            obsolete.kind.clone(),
            input.title,
            input.content,
            obsolete.scope.clone(),
            ClaimOrigin::AgentInferred,
            writer.clone(),
        )
        .with_status(MemoryStatus::NeedsReview)
        .with_confidence(obsolete.confidence.value());
        replacement.tags.clone_from(&obsolete.tags);
        replacement.review_after = obsolete.review_after;
        replacement.procedure = input.procedure;
        for evidence in input.evidence {
            replacement = replacement.with_evidence(evidence);
        }
        validate_memory_item(&replacement)?;

        let mut proposal = CorrectionProposal::new(
            obsolete.id,
            replacement.id,
            obsolete.kind.clone(),
            obsolete.scope.clone(),
            "pending",
            writer,
        );
        replacement.correction_proposal_id = Some(proposal.id);
        let mut locked_obsolete = obsolete.clone();
        locked_obsolete.pending_correction_proposal_id = Some(proposal.id);
        proposal.canonical_digest =
            correction_proposal_digest(&proposal, &locked_obsolete, &replacement)?;
        self.repo
            .save_correction_proposal(&proposal, &replacement, &obsolete, &locked_obsolete)
            .await?;
        Ok((proposal, replacement, locked_obsolete))
    }

    /// Inspect one durable correction proposal through an exact authorization boundary.
    pub async fn inspect_correction_proposal(
        &self,
        proposal_id: &Id,
        project: Option<&str>,
        task: Option<&str>,
        cwd: Option<&str>,
    ) -> IndexResult<CorrectionProposalInspection> {
        let project = normalized_boundary_value(project);
        let task = normalized_boundary_value(task);
        let cwd = normalized_boundary_value(cwd);
        let proposal = self
            .repo
            .get_correction_proposal(proposal_id)
            .await?
            .ok_or_else(|| {
                IndexError::NotFound(format!("correction proposal {proposal_id} not found"))
            })?;
        let replacement = self.get_required_memory(&proposal.replacement_id).await?;
        let obsolete = self.get_required_memory(&proposal.obsolete_id).await?;
        validate_proposal_pair(&proposal, &obsolete, &replacement)?;
        Self::authorize_correction_item(&replacement, "replacement", project, task, cwd)?;
        Self::authorize_correction_item(&obsolete, "obsolete", project, task, cwd)?;
        Ok(CorrectionProposalInspection {
            proposal,
            replacement,
            obsolete,
        })
    }

    /// List durable correction proposals visible inside one exact authorization boundary.
    pub async fn list_correction_proposals(
        &self,
        limit: Option<usize>,
        project: Option<&str>,
        task: Option<&str>,
        cwd: Option<&str>,
    ) -> IndexResult<Vec<CorrectionProposalInspection>> {
        let project = normalized_boundary_value(project);
        let task = normalized_boundary_value(task);
        let cwd = normalized_boundary_value(cwd);
        let proposals = self.repo.list_correction_proposals(None, None).await?;
        let mut inspections = Vec::new();
        for proposal in proposals {
            let replacement = self.get_required_memory(&proposal.replacement_id).await?;
            let obsolete = self.get_required_memory(&proposal.obsolete_id).await?;
            validate_proposal_pair(&proposal, &obsolete, &replacement)?;
            if !Self::correction_item_matches_boundary(
                &replacement,
                "replacement",
                project,
                task,
                cwd,
            )? || !Self::correction_item_matches_boundary(
                &obsolete, "obsolete", project, task, cwd,
            )? {
                continue;
            }
            inspections.push(CorrectionProposalInspection {
                proposal,
                replacement,
                obsolete,
            });
            if inspections.len() == limit.unwrap_or(usize::MAX) {
                break;
            }
        }
        Ok(inspections)
    }

    /// Verify a pending procedure replacement while keeping it inactive.
    ///
    /// The caller must present the exact proposal-time P0 digest. Receipt proof is attached to the
    /// `needs_review` replacement and all three proposal records are compare-and-swapped in one
    /// transaction. The proposal remains pending and receives the resulting P1 digest.
    pub async fn verify_correction_procedure(
        &self,
        proposal_id: &Id,
        expected_digest: &str,
        receipt_path: &Path,
        expires_at: OffsetDateTime,
        project: Option<&str>,
        task: Option<&str>,
        cwd: Option<&str>,
    ) -> IndexResult<(CorrectionProposal, MemoryItem, MemoryItem)> {
        let expected_digest = expected_digest.trim();
        if expected_digest.is_empty() {
            return Err(IndexError::Parse(
                "expected correction proposal P0 digest must not be empty".to_string(),
            ));
        }

        let proposal = self
            .repo
            .get_correction_proposal(proposal_id)
            .await?
            .ok_or_else(|| {
                IndexError::NotFound(format!("correction proposal {proposal_id} not found"))
            })?;
        if proposal.canonical_digest != expected_digest {
            return Err(IndexError::InvalidState(format!(
                "correction proposal {proposal_id} P0 digest mismatch"
            )));
        }

        let project = normalized_boundary_value(project);
        let task = normalized_boundary_value(task);
        let cwd = normalized_boundary_value(cwd);
        let mut replacement = self.get_required_memory(&proposal.replacement_id).await?;
        let obsolete = self.get_required_memory(&proposal.obsolete_id).await?;
        Self::authorize_correction_item(&replacement, "replacement", project, task, cwd)?;
        Self::authorize_correction_item(&obsolete, "obsolete", project, task, cwd)?;
        validate_proposal_pair(&proposal, &obsolete, &replacement)?;

        if proposal.status != CorrectionProposalStatus::Pending {
            return Err(IndexError::InvalidState(format!(
                "correction proposal {proposal_id} cannot be verified from status {}",
                proposal.status
            )));
        }
        if proposal.memory_kind != MemoryKind::Procedure {
            return Err(IndexError::InvalidState(format!(
                "correction proposal {proposal_id} is not a procedure correction"
            )));
        }
        if obsolete.status != MemoryStatus::Active
            || obsolete.pending_correction_proposal_id != Some(proposal.id)
        {
            return Err(IndexError::InvalidState(format!(
                "obsolete procedure {} no longer has the active pending-proposal lock",
                obsolete.id
            )));
        }
        if replacement.status != MemoryStatus::NeedsReview
            || replacement.origin != ClaimOrigin::AgentInferred
            || replacement.correction_proposal_id != Some(proposal.id)
        {
            return Err(IndexError::InvalidState(format!(
                "replacement procedure {} no longer has its inactive proposal state",
                replacement.id
            )));
        }
        if replacement.evidence.is_empty() || has_manual_review_evidence(&replacement) {
            return Err(IndexError::InvalidState(format!(
                "replacement procedure {} must retain non-review source evidence",
                replacement.id
            )));
        }
        let actual_p0 = correction_proposal_digest(&proposal, &obsolete, &replacement)?;
        if actual_p0 != proposal.canonical_digest {
            return Err(IndexError::InvalidState(format!(
                "correction proposal {proposal_id} canonical pair changed before procedure verification"
            )));
        }

        let procedure = replacement.procedure.as_ref().ok_or_else(|| {
            IndexError::InvalidState(format!(
                "replacement procedure {} has no structured procedure card",
                replacement.id
            ))
        })?;
        if procedure.verification.evidence_path.is_some()
            || procedure.verification.evidence_sha256.is_some()
            || procedure.verification.verified_at.is_some()
            || procedure.expires_at.is_some()
        {
            return Err(IndexError::InvalidState(format!(
                "replacement procedure {} already carries verification state; verification is a one-time P0 to P1 transition",
                replacement.id
            )));
        }

        let receipt_bytes = fs::read(receipt_path)?;
        let receipt: ProcedureVerificationReceipt = serde_json::from_slice(&receipt_bytes)
            .map_err(|error| {
                IndexError::Parse(format!(
                    "invalid procedure verification receipt {}: {error}",
                    receipt_path.display()
                ))
            })?;
        validate_procedure_receipt(procedure, &receipt)?;
        let evidence_path = self
            .portable_procedure_evidence_path(&replacement.scope, receipt_path)
            .await?;
        let mut verification_evidence = EvidenceRef::new(EvidenceKind::File, evidence_path.clone())
            .with_summary(
                "Engram-verified inactive correction procedure receipt; hash and semantics are rechecked before apply and at use time.",
            );
        let now = verification_evidence
            .observed_at
            .max(replacement.updated_at);
        verification_evidence.observed_at = now;
        if expires_at <= now {
            return Err(IndexError::Parse(
                "procedure correction expiry must be in the future".to_string(),
            ));
        }

        let expected_proposal = proposal;
        let expected_replacement = replacement.clone();
        let expected_obsolete = obsolete.clone();
        {
            let procedure = replacement.procedure.as_mut().ok_or_else(|| {
                IndexError::InvalidState(format!(
                    "replacement procedure {} lost its structured procedure card",
                    replacement.id
                ))
            })?;
            procedure.verification.evidence_path = Some(evidence_path.clone());
            procedure.verification.evidence_sha256 = Some(sha256_hex(&receipt_bytes));
            procedure.verification.verified_at = Some(now);
            procedure.expires_at = Some(expires_at);
        }
        replacement.updated_at = now;
        replacement.evidence.push(verification_evidence);
        validate_memory_item(&replacement)?;

        let p1 = correction_proposal_digest(&expected_proposal, &expected_obsolete, &replacement)?;
        if p1 == expected_proposal.canonical_digest {
            return Err(IndexError::InvalidState(
                "procedure correction verification did not rotate the canonical digest".to_string(),
            ));
        }
        let verified_proposal = expected_proposal.clone().with_canonical_digest(p1);
        self.repo
            .verify_correction_procedure(
                &expected_proposal,
                &expected_replacement,
                &expected_obsolete,
                &verified_proposal,
                &replacement,
                &expected_obsolete,
            )
            .await?;
        Ok((verified_proposal, replacement, expected_obsolete))
    }

    /// Apply one exact pending correction proposal selected through the full operator surface.
    ///
    /// Operator selection is an administrative trust boundary, not proof of a human identity or
    /// reviewer authority. The replacement therefore becomes active-but-unreviewed.
    pub async fn apply_correction(
        &self,
        proposal_id: &Id,
        expected_digest: &str,
        project: Option<&str>,
        task: Option<&str>,
        cwd: Option<&str>,
    ) -> IndexResult<(CorrectionProposal, MemoryItem, MemoryItem)> {
        let expected_digest = expected_digest.trim();
        if expected_digest.is_empty() {
            return Err(IndexError::Parse(
                "expected correction proposal digest must not be empty".to_string(),
            ));
        }

        let proposal = self
            .repo
            .get_correction_proposal(proposal_id)
            .await?
            .ok_or_else(|| {
                IndexError::NotFound(format!("correction proposal {proposal_id} not found"))
            })?;
        if proposal.canonical_digest != expected_digest {
            return Err(IndexError::InvalidState(format!(
                "correction proposal {proposal_id} digest mismatch"
            )));
        }

        let project = normalized_boundary_value(project);
        let task = normalized_boundary_value(task);
        let cwd = normalized_boundary_value(cwd);
        let mut replacement = self.get_required_memory(&proposal.replacement_id).await?;
        let mut obsolete = self.get_required_memory(&proposal.obsolete_id).await?;
        Self::authorize_correction_item(&replacement, "replacement", project, task, cwd)?;
        Self::authorize_correction_item(&obsolete, "obsolete", project, task, cwd)?;
        validate_proposal_pair(&proposal, &obsolete, &replacement)?;

        if proposal.status == CorrectionProposalStatus::Applied {
            if proposal.memory_kind == MemoryKind::Procedure {
                return Err(IndexError::InvalidState(format!(
                    "procedure correction proposal {proposal_id} was already applied; a retry is not causal activation evidence"
                )));
            }
            let actual_applied_digest =
                correction_proposal_digest(&proposal, &obsolete, &replacement)?;
            if proposal.applied_digest.as_deref() == Some(actual_applied_digest.as_str())
                && replacement.status == MemoryStatus::Active
                && obsolete.status == MemoryStatus::Superseded
                && replacement.supersedes.contains(&obsolete.id)
                && replacement.correction_proposal_id.is_none()
                && obsolete.pending_correction_proposal_id.is_none()
                && !has_manual_review_evidence(&replacement)
            {
                return Ok((proposal, replacement, obsolete));
            }
            return Err(IndexError::InvalidState(format!(
                "applied correction proposal {proposal_id} no longer has its exact applied pair"
            )));
        }
        if proposal.status != CorrectionProposalStatus::Pending {
            return Err(IndexError::InvalidState(format!(
                "correction proposal {proposal_id} cannot be applied from status {}",
                proposal.status
            )));
        }
        if obsolete.status != MemoryStatus::Active {
            return Err(IndexError::InvalidState(format!(
                "obsolete memory item {} must still be active (status: {})",
                obsolete.id, obsolete.status
            )));
        }
        if replacement.status != MemoryStatus::NeedsReview {
            return Err(IndexError::InvalidState(format!(
                "proposed replacement memory item {} must still need review (status: {})",
                replacement.id, replacement.status
            )));
        }
        if replacement.origin != ClaimOrigin::AgentInferred {
            return Err(IndexError::InvalidState(format!(
                "proposed replacement memory item {} has an invalid origin",
                replacement.id
            )));
        }
        if replacement.correction_proposal_id != Some(proposal.id) {
            return Err(IndexError::InvalidState(format!(
                "proposed replacement memory item {} lost its server-minted proposal link",
                replacement.id
            )));
        }
        if obsolete.pending_correction_proposal_id != Some(proposal.id) {
            return Err(IndexError::InvalidState(format!(
                "obsolete memory item {} lost its server-minted proposal lock",
                obsolete.id
            )));
        }
        if replacement.evidence.is_empty() || has_manual_review_evidence(&replacement) {
            return Err(IndexError::InvalidState(format!(
                "proposed replacement memory item {} must retain non-review source evidence",
                replacement.id
            )));
        }
        let actual_digest = correction_proposal_digest(&proposal, &obsolete, &replacement)?;
        if actual_digest != proposal.canonical_digest {
            return Err(IndexError::InvalidState(format!(
                "correction proposal {proposal_id} canonical pair changed after proposal creation"
            )));
        }
        if proposal.memory_kind == MemoryKind::Procedure {
            let procedure = replacement.procedure.as_ref().ok_or_else(|| {
                IndexError::InvalidState(format!(
                    "replacement procedure {} has no structured procedure card",
                    replacement.id
                ))
            })?;
            let now = OffsetDateTime::now_utc();
            let expires_at = procedure.expires_at.ok_or_else(|| {
                IndexError::InvalidState(format!(
                    "replacement procedure {} has no verification expiry",
                    replacement.id
                ))
            })?;
            if expires_at <= now || !procedure.is_verified_at(now) {
                return Err(IndexError::InvalidState(format!(
                    "replacement procedure {} does not have complete unexpired verification proof",
                    replacement.id
                )));
            }
            let evidence_path =
                procedure
                    .verification
                    .evidence_path
                    .as_deref()
                    .ok_or_else(|| {
                        IndexError::InvalidState(format!(
                            "replacement procedure {} has no verification receipt path",
                            replacement.id
                        ))
                    })?;
            let expected_hash = procedure
                .verification
                .evidence_sha256
                .as_deref()
                .ok_or_else(|| {
                    IndexError::InvalidState(format!(
                        "replacement procedure {} has no verification receipt hash",
                        replacement.id
                    ))
                })?;
            if !replacement.evidence.iter().any(|evidence| {
                evidence.kind == EvidenceKind::File && evidence.target == evidence_path
            }) {
                return Err(IndexError::InvalidState(format!(
                    "replacement procedure {} is missing its verification receipt evidence reference",
                    replacement.id
                )));
            }
            let repository_context = self.resolve_repository_context(cwd).await?;
            let checkout_root = repository_context
                .as_ref()
                .and_then(|context| context.checkout.as_ref())
                .map(|checkout| PathBuf::from(&checkout.local_path));
            if !Path::new(evidence_path).is_absolute() && checkout_root.is_none() {
                return Err(IndexError::InvalidState(format!(
                    "replacement procedure {} has a checkout-relative receipt but no current checkout could be resolved",
                    replacement.id
                )));
            }
            let receipt_path =
                resolve_procedure_evidence_path(evidence_path, checkout_root.as_deref());
            let receipt_bytes = fs::read(&receipt_path)?;
            if sha256_hex(&receipt_bytes) != expected_hash {
                return Err(IndexError::InvalidState(format!(
                    "replacement procedure {} verification receipt hash changed at {}",
                    replacement.id,
                    receipt_path.display()
                )));
            }
            let receipt: ProcedureVerificationReceipt = serde_json::from_slice(&receipt_bytes)
                .map_err(|error| {
                    IndexError::Parse(format!(
                        "invalid procedure verification receipt {}: {error}",
                        receipt_path.display()
                    ))
                })?;
            validate_procedure_receipt(procedure, &receipt)?;
        }

        let expected_proposal = proposal;
        let expected_replacement = replacement.clone();
        let expected_obsolete = obsolete.clone();
        let applied_evidence = EvidenceRef::new(
            EvidenceKind::ToolCall,
            format!("memory.apply_correction:{}", expected_proposal.id),
        )
        .with_summary(format!(
            "Operator-selected correction proposal {} applied replacement {} over {}",
            expected_proposal.id, replacement.id, obsolete.id
        ));
        replacement = replacement
            .with_status(MemoryStatus::Active)
            .with_superseded_item(obsolete.id)
            .with_evidence(applied_evidence.clone());
        replacement.correction_proposal_id = None;
        obsolete = obsolete
            .with_status(MemoryStatus::Superseded)
            .with_evidence(applied_evidence);
        obsolete.pending_correction_proposal_id = None;
        let mut proposal = expected_proposal.clone().with_applied();
        let applied_digest = correction_proposal_digest(&proposal, &obsolete, &replacement)?;
        proposal = proposal.with_applied_digest(applied_digest);
        self.repo
            .apply_correction_proposal(
                &expected_proposal,
                &expected_replacement,
                &expected_obsolete,
                &proposal,
                &replacement,
                &obsolete,
            )
            .await?;
        Ok((proposal, replacement, obsolete))
    }

    /// Link an already-active user correction to the exact active item it replaces.
    ///
    /// This path deliberately adds no manual-review evidence and confers no reviewer authority.
    pub async fn correct_memory(
        &self,
        obsolete_id: &Id,
        replacement_id: &Id,
        reason: impl Into<String>,
        project: Option<&str>,
        task: Option<&str>,
        cwd: Option<&str>,
    ) -> IndexResult<(MemoryItem, MemoryItem)> {
        if obsolete_id == replacement_id {
            return Err(IndexError::InvalidState(
                "memory item cannot correct itself".to_string(),
            ));
        }

        let reason = reason.into();
        if reason.trim().is_empty() {
            return Err(IndexError::Parse(
                "correction reason must not be empty".to_string(),
            ));
        }

        let project = project.map(str::trim).filter(|value| !value.is_empty());
        let task = task.map(str::trim).filter(|value| !value.is_empty());
        let cwd = cwd.map(str::trim).filter(|value| !value.is_empty());
        let mut replacement = self.get_required_memory(replacement_id).await?;
        let mut obsolete = self.get_required_memory(obsolete_id).await?;
        self.reject_pending_correction_pair_mutation(&replacement, "correct")
            .await?;
        self.reject_pending_correction_pair_mutation(&obsolete, "correct")
            .await?;
        Self::authorize_correction_item(&replacement, "replacement", project, task, cwd)?;
        Self::authorize_correction_item(&obsolete, "obsolete", project, task, cwd)?;
        if obsolete.status == MemoryStatus::Superseded
            && replacement.status == MemoryStatus::Active
            && replacement.supersedes.contains(obsolete_id)
        {
            return Ok((replacement, obsolete));
        }
        if obsolete.status != MemoryStatus::Active {
            return Err(IndexError::InvalidState(format!(
                "memory item {obsolete_id} cannot be corrected from status {}",
                obsolete.status
            )));
        }
        if replacement.status != MemoryStatus::Active {
            return Err(IndexError::InvalidState(format!(
                "replacement memory item {replacement_id} must already be active (status: {})",
                replacement.status
            )));
        }
        if replacement.origin != ClaimOrigin::UserCorrected {
            return Err(IndexError::InvalidState(format!(
                "replacement memory item {replacement_id} must have origin user_corrected"
            )));
        }
        if replacement.kind != obsolete.kind || replacement.scope != obsolete.scope {
            return Err(IndexError::InvalidState(
                "correction requires replacement and obsolete memory to have identical kind and scope"
                    .to_string(),
            ));
        }
        if replacement.evidence.is_empty() {
            return Err(IndexError::InvalidState(format!(
                "replacement memory item {replacement_id} must have source evidence"
            )));
        }
        if has_manual_review_evidence(&replacement) {
            return Err(IndexError::InvalidState(
                "agent correction cannot carry manual_review evidence".to_string(),
            ));
        }

        let expected_replacement = replacement.clone();
        let expected_obsolete = obsolete.clone();
        let correction = EvidenceRef::new(EvidenceKind::ToolCall, "memory.correct").with_summary(
            format!("Exact-ID correction linked {replacement_id} over {obsolete_id}: {reason}"),
        );
        replacement = replacement
            .with_superseded_item(*obsolete_id)
            .with_evidence(correction.clone());
        obsolete = obsolete
            .with_status(MemoryStatus::Superseded)
            .with_evidence(correction);
        self.repo
            .save_memory_correction(
                &expected_replacement,
                &expected_obsolete,
                &replacement,
                &obsolete,
            )
            .await?;
        Ok((replacement, obsolete))
    }

    fn authorize_correction_item(
        item: &MemoryItem,
        label: &str,
        project: Option<&str>,
        task: Option<&str>,
        cwd: Option<&str>,
    ) -> IndexResult<()> {
        let authorized = Self::correction_item_matches_boundary(item, label, project, task, cwd)?;
        if !authorized {
            return Err(IndexError::InvalidState(format!(
                "{label} memory item '{}' is outside the resolved correction authorization boundary",
                item.id
            )));
        }
        Ok(())
    }

    fn correction_item_matches_boundary(
        item: &MemoryItem,
        label: &str,
        project: Option<&str>,
        task: Option<&str>,
        cwd: Option<&str>,
    ) -> IndexResult<bool> {
        let authorized = match &item.scope {
            MemoryScope::Global | MemoryScope::User => true,
            MemoryScope::Project {
                project_id,
                project_name,
            } => project.is_some_and(|selector| {
                project_name.eq_ignore_ascii_case(selector)
                    || project_id.is_some_and(|id| selector == id.to_string())
            }),
            MemoryScope::Task {
                project_id,
                project_name,
                task_id,
                task_name,
            } => {
                let task_matches = task.is_some_and(|selector| {
                    task_name.eq_ignore_ascii_case(selector)
                        || task_id.is_some_and(|id| selector == id.to_string())
                });
                let project_matches = match (project_id, project_name.as_deref()) {
                    (None, None) => true,
                    (stored_id, stored_name) => project.is_some_and(|selector| {
                        stored_name.is_some_and(|name| name.eq_ignore_ascii_case(selector))
                            || stored_id.is_some_and(|id| selector == id.to_string())
                    }),
                };
                task_matches && project_matches
            }
            MemoryScope::Repository { local_path, .. } => {
                match (
                    cwd,
                    local_path
                        .as_deref()
                        .map(str::trim)
                        .filter(|value| !value.is_empty()),
                ) {
                    (Some(cwd), Some(local_path))
                        if Path::new(cwd).is_absolute() && Path::new(local_path).is_absolute() =>
                    {
                        path_starts_with(
                            &canonical_or_original(Path::new(cwd)),
                            &canonical_or_original(Path::new(local_path)),
                        )
                    }
                    _ => false,
                }
            }
            MemoryScope::Entity { .. }
            | MemoryScope::Session { .. }
            | MemoryScope::Custom { .. } => {
                return Err(IndexError::InvalidState(format!(
                    "{label} memory item '{}' uses a scope without a trusted correction selector",
                    item.id
                )));
            }
        };
        Ok(authorized)
    }

    fn reject_proposal_replacement_mutation(item: &MemoryItem, action: &str) -> IndexResult<()> {
        if let Some(proposal_id) = item
            .correction_proposal_id
            .or(item.pending_correction_proposal_id)
        {
            return Err(IndexError::InvalidState(format!(
                "memory item {} is immutable while bound to correction proposal {}; use apply_correction on the proposal instead of {action}",
                item.id, proposal_id
            )));
        }
        Ok(())
    }

    async fn reject_pending_correction_pair_mutation(
        &self,
        item: &MemoryItem,
        action: &str,
    ) -> IndexResult<()> {
        Self::reject_proposal_replacement_mutation(item, action)?;
        if let Some(proposal) = self
            .repo
            .list_correction_proposals_for_memory(&item.id)
            .await?
            .into_iter()
            .find(|proposal| proposal.status == CorrectionProposalStatus::Pending)
        {
            return Err(IndexError::InvalidState(format!(
                "memory item {} is immutable while referenced by pending correction proposal {}; apply or forget the proposal pair before {action}",
                item.id, proposal.id
            )));
        }
        Ok(())
    }

    /// Archive a memory item with metadata.
    pub async fn archive_memory(
        &self,
        id: &Id,
        reason: impl Into<String>,
        archived_by: Option<String>,
    ) -> IndexResult<MemoryItem> {
        let item = self.get_required_memory(id).await?;
        self.reject_pending_correction_pair_mutation(&item, "archive")
            .await?;
        let expected = item.clone();
        let item = item.with_archive(reason, archived_by);
        self.repo
            .save_memory_item_if_unchanged(&expected, &item)
            .await?;
        Ok(item)
    }

    /// Permanently remove a memory item and its linked internal projections.
    ///
    /// Generated vault exports and external copies are outside the canonical store and must be
    /// recompiled or deleted separately by their owner.
    pub async fn forget_memory(&self, id: &Id) -> IndexResult<MemoryForgetReport> {
        let pending_receipt = self.repo.get_pending_memory_forget_receipt(id).await?;
        let cleanup_resumed = pending_receipt.is_some();
        let mut transitioning_pair_ids = HashSet::new();
        if let Some(receipt) = &pending_receipt {
            transitioning_pair_ids.extend(receipt.pending_replacement_ids.iter().copied());
            transitioning_pair_ids.extend(receipt.unlocked_obsolete_ids.iter().copied());
        } else {
            for proposal in self.repo.list_correction_proposals_for_memory(id).await? {
                if proposal.status == CorrectionProposalStatus::Pending {
                    transitioning_pair_ids.insert(proposal.obsolete_id);
                    transitioning_pair_ids.insert(proposal.replacement_id);
                }
            }
        }
        let id_text = id.to_string();
        if let Some(blocking) = self
            .repo
            .list_memory_items(None, None)
            .await?
            .into_iter()
            .find(|item| {
                item.id != *id
                    && !transitioning_pair_ids.contains(&item.id)
                    && (item.correction_proposal_id.is_some()
                        || item.pending_correction_proposal_id.is_some())
                    && memory_item_references_text(item, &id_text)
            })
        {
            let proposal_id = blocking
                .correction_proposal_id
                .or(blocking.pending_correction_proposal_id)
                .expect("proposal-bound item has a proposal ID");
            return Err(IndexError::InvalidState(format!(
                "cannot forget memory {id} because proposal-bound memory {} references it; apply correction proposal {proposal_id} or forget its pending replacement first",
                blocking.id
            )));
        }
        let correction_purge = match pending_receipt {
            Some(receipt) => receipt,
            None => {
                self.repo
                    .delete_memory_with_correction_projections(id)
                    .await?
            }
        };

        let mut memory_items_updated = 0;
        let mut commits_redacted = 0;
        let mut traces_deleted = 0;
        let mut feedback_deleted = 0;
        let target = correction_purge.deleted.then_some(id);
        for deleted_id in correction_purge
            .pending_replacement_ids
            .iter()
            .chain(target)
        {
            let telemetry = self
                .telemetry_repo
                .purge_memory_references(deleted_id)
                .await?;
            let references = self.repo.purge_memory_item_references(deleted_id).await?;
            memory_items_updated += references.memory_items_updated;
            commits_redacted += references.commits_redacted;
            traces_deleted += telemetry.traces_deleted;
            feedback_deleted += telemetry.feedback_deleted;
        }
        if correction_purge.deleted {
            self.repo.complete_memory_forget_receipt(id).await?;
        }

        Ok(MemoryForgetReport {
            id: *id,
            deleted: correction_purge.deleted,
            memory_items_updated: memory_items_updated
                + correction_purge.unlocked_obsolete_ids.len(),
            commits_redacted,
            traces_deleted,
            feedback_deleted,
            correction_proposals_deleted: correction_purge.proposal_ids.len(),
            proposal_replacements_deleted: correction_purge.pending_replacement_ids.len(),
            proposal_obsoletes_unlocked: correction_purge.unlocked_obsolete_ids.len(),
            cleanup_resumed,
            projection_counts_complete: !cleanup_resumed,
        })
    }

    /// Verify a procedure candidate against an immutable, machine-readable success receipt.
    ///
    /// The receipt body is checked but not copied into memory. Engram stores its path and SHA-256
    /// and revalidates both at retrieval time.
    pub async fn verify_procedure(
        &self,
        id: &Id,
        receipt_path: &Path,
        expires_at: Option<OffsetDateTime>,
    ) -> IndexResult<MemoryItem> {
        let mut item = self.get_required_memory(id).await?;
        self.reject_pending_correction_pair_mutation(&item, "verify_procedure")
            .await?;
        if item.kind != MemoryKind::Procedure {
            return Err(IndexError::InvalidState(format!(
                "memory item {id} is not a procedure"
            )));
        }
        if item.status != MemoryStatus::NeedsReview {
            return Err(IndexError::InvalidState(format!(
                "procedure {id} must be needs_review before verification (status: {})",
                item.status
            )));
        }

        let receipt_bytes = fs::read(receipt_path)?;
        let receipt: ProcedureVerificationReceipt = serde_json::from_slice(&receipt_bytes)
            .map_err(|error| {
                IndexError::Parse(format!(
                    "invalid procedure verification receipt {}: {error}",
                    receipt_path.display()
                ))
            })?;
        let now = OffsetDateTime::now_utc();
        if expires_at.is_some_and(|expires_at| expires_at <= now) {
            return Err(IndexError::Parse(
                "procedure expiry must be in the future".to_string(),
            ));
        }
        let evidence_path = self
            .portable_procedure_evidence_path(&item.scope, receipt_path)
            .await?;

        let expected = item.clone();
        {
            let procedure = item.procedure.as_mut().ok_or_else(|| {
                IndexError::InvalidState(format!(
                    "procedure memory {id} has no structured procedure card"
                ))
            })?;
            validate_procedure_receipt(procedure, &receipt)?;
            procedure.verification.evidence_path = Some(evidence_path.clone());
            procedure.verification.evidence_sha256 = Some(sha256_hex(&receipt_bytes));
            procedure.verification.verified_at = Some(now);
            procedure.expires_at = expires_at;
        }
        item.status = MemoryStatus::Active;
        item.updated_at = now;
        item.evidence.push(
            EvidenceRef::new(EvidenceKind::File, evidence_path).with_summary(
                "Engram-verified procedure success receipt; hash rechecked at use time.",
            ),
        );
        validate_memory_item(&item)?;
        self.repo
            .save_memory_item_if_unchanged(&expected, &item)
            .await?;
        Ok(item)
    }

    /// Return only procedures whose scope, prerequisites, freshness, and proof still match.
    pub async fn match_procedures(
        &self,
        input: ProcedureMatchInput,
    ) -> IndexResult<ProcedureMatchReport> {
        if input.query.trim().is_empty() {
            return Err(IndexError::Parse(
                "procedure query must not be empty".to_string(),
            ));
        }
        let query_chars = input.query.chars().count();
        if query_chars > MAX_PROCEDURE_QUERY_CHARS {
            return Err(IndexError::Parse(format!(
                "procedure query must be a task-focused retrieval phrase of at most {MAX_PROCEDURE_QUERY_CHARS} characters; received {query_chars}"
            )));
        }
        let repository_context = self
            .resolve_repository_context(input.cwd.as_deref())
            .await?;
        let checkout_root = repository_context
            .as_ref()
            .and_then(|context| context.checkout.as_ref())
            .map(|checkout| PathBuf::from(&checkout.local_path));
        let current_checkout_root = checkout_root
            .as_ref()
            .map(|root| root.display().to_string());
        let execution_guidance = current_checkout_root.as_ref().map(|root| {
            format!(
                "Execute repository-scoped procedure commands from current_checkout_root `{root}`. Stored procedure scope.local_path and evidence paths are provenance only; never use them as execution targets."
            )
        });
        let orientation = resolve_orientation_project(
            input.project.as_deref(),
            input.cwd.as_deref(),
            repository_context.as_ref(),
        );
        let identity = orientation_identity(&orientation, repository_context.as_ref());
        let unlinked_repository_boundary = input.project.is_none()
            && repository_context.is_some()
            && orientation.project_candidates.is_empty();
        if (orientation.requires_confirmation || orientation.ambiguity.is_some())
            && !unlinked_repository_boundary
        {
            let detail = orientation
                .ambiguity
                .as_deref()
                .unwrap_or(orientation.reason.as_str())
                .to_string();
            return Ok(ProcedureMatchReport {
                query: input.query,
                identity,
                resolution: orientation,
                procedures: Vec::new(),
                diagnostics: Vec::new(),
                abstained: true,
                message: format!(
                    "Procedure match abstained because repository/project scope is ambiguous: {detail}"
                ),
                required_condition_keys: Vec::new(),
                next_actions: vec![
                    "Confirm the canonical project or register the repository/project link before retrying."
                        .to_string(),
                ],
                suggested_operation_evidence: None,
                current_checkout_root,
                execution_guidance,
            });
        }
        let selected_project = orientation.selected_project.as_deref();
        let ranked = rank_memory_items(
            self.list_active_memory(None)
                .await?
                .into_iter()
                .filter(|item| item.kind == MemoryKind::Procedure)
                .filter(|item| {
                    procedure_scope_matches(
                        &item.scope,
                        selected_project,
                        input.cwd.as_deref(),
                        repository_context.as_ref(),
                    )
                })
                .collect(),
            MemoryRankContext::search(None, None, Some(&input.query)),
        )
        .into_iter()
        .filter(|ranked| ranked.components.text >= MIN_PROCEDURE_TEXT_MATCH_SCORE)
        .take(MAX_PROCEDURE_CANDIDATES_TO_EVALUATE)
        .collect::<Vec<_>>();

        let now = OffsetDateTime::now_utc();
        let mut procedures = Vec::new();
        let mut diagnostics = Vec::new();
        let mut required_condition_keys = BTreeSet::new();
        for ranked in ranked {
            let applicability = procedure_applicability(
                &ranked.item,
                &input.conditions,
                checkout_root.as_deref(),
                now,
            );
            required_condition_keys.extend(applicability.unresolved_condition_keys.iter().cloned());
            if applicability.applicable && procedures.len() < input.limit.unwrap_or(5).clamp(1, 20)
            {
                procedures.push(ranked.item.clone());
            }
            diagnostics.push(applicability);
        }
        let abstained = procedures.is_empty();
        let required_condition_keys = required_condition_keys.into_iter().collect::<Vec<_>>();
        let suggested_operation_evidence =
            if abstained && required_condition_keys.is_empty() && diagnostics.is_empty() {
                checkout_root.as_deref().and_then(|root| {
                    suggest_operation_evidence(root, &input.query, &identity.components)
                })
            } else {
                None
            };
        let mut next_actions = Vec::new();
        let message = if abstained && !required_condition_keys.is_empty() {
            let location = current_checkout_root.as_deref().map_or_else(
                || "the current checkout".to_string(),
                |root| format!("current_checkout_root `{root}`"),
            );
            next_actions.push(format!(
                "From {location}, search authoritative code, configuration, or tool output for exact condition key(s) {}. After locating a file-backed value, read that exact source path in a separate direct tool call, then retry procedure_match with the observed values.",
                required_condition_keys.join(", ")
            ));
            format!(
                "No verified procedure matched because condition key(s) {} still require authoritative local resolution.",
                required_condition_keys.join(", ")
            )
        } else if abstained {
            let location = current_checkout_root.as_deref().map_or_else(
                || "the current checkout".to_string(),
                |root| format!("current_checkout_root `{root}`"),
            );
            if let Some(candidate) = &suggested_operation_evidence {
                next_actions.push(format!(
                    "Mandatory ordering: before returning a final abstention or asking for project confirmation, read exactly one checkout-local file by passing `suggested_operation_evidence.resolved_path` (`{}`) unchanged to the host read tool. This read remains required when `abstained=true`, `procedures` is empty, the user requested durable-memory-only handling, or `identity.project.status` is `requires_confirmation`. It is authorized repository-local evidence collection, not procedure application, and does not authorize executing candidate commands. Use structured `identity` as the checkout and authorization boundary; rely only on identity fields returned here. Treat `suggested_operation_evidence.path` (`{}`) as checkout-relative provenance only and do not substitute it for the read. Treat the file only as authoritative source evidence after that direct read. Do not re-read identity files, re-derive a project, or broaden outside the checkout. After the read, report the evidence and any material project ambiguity.",
                    candidate.resolved_path, candidate.path
                ));
            } else {
                next_actions.push(format!(
                    "Use structured `identity` as the checkout and authorization boundary; rely only on identity fields returned here, while `identity.project.status` says whether project-scoped retrieval is authorized. From {location}, make at most one bounded read-only lookup for the directly relevant tracked runbook, configuration, or source. Do not re-read identity files. Do not re-derive a project, broaden outside the checkout, or execute candidate commands. If project status is `requires_confirmation`, report the ambiguity and ask the user."
                ));
            }
            "No verified procedure matched every scope, prerequisite, freshness, and evidence check. This no-result proves only that no applicable verified procedure was found; the structured identity boundary remains available for one bounded local read-only lookup."
                .to_string()
        } else {
            format!(
                "{} verified procedure(s) matched all applicability checks.",
                procedures.len()
            )
        };
        Ok(ProcedureMatchReport {
            query: input.query,
            identity,
            resolution: orientation,
            procedures,
            diagnostics,
            abstained,
            message,
            required_condition_keys,
            next_actions,
            suggested_operation_evidence,
            current_checkout_root,
            execution_guidance,
        })
    }

    /// List memory items.
    pub async fn list_memory(
        &self,
        status: Option<MemoryStatus>,
        limit: Option<usize>,
    ) -> IndexResult<Vec<MemoryItem>> {
        Ok(self.repo.list_memory_items(status, limit).await?)
    }

    /// Whether a MemoryItem is applicable to one project/task/cwd authorization boundary.
    #[must_use]
    pub fn item_matches_boundary(
        item: &MemoryItem,
        project: Option<&str>,
        task: Option<&str>,
        cwd: Option<&str>,
    ) -> bool {
        crate::memory_ranker::memory_scope_matches(
            item,
            MemoryRankContext::scoped_search(project, None, task, None, cwd, None),
        )
    }

    /// List MemoryItems after applying an authorization boundary and before applying the limit.
    pub async fn list_memory_for_boundary(
        &self,
        status: Option<MemoryStatus>,
        limit: Option<usize>,
        project: Option<&str>,
        task: Option<&str>,
        cwd: Option<&str>,
    ) -> IndexResult<Vec<MemoryItem>> {
        let mut items = self.list_memory(status, None).await?;
        if status == Some(MemoryStatus::NeedsReview) {
            items.retain(|item| item.correction_proposal_id.is_none());
        }
        items.retain(|item| Self::item_matches_boundary(item, project, task, cwd));
        if let Some(limit) = limit {
            items.truncate(limit);
        }
        Ok(items)
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
        let mut items = self
            .repo
            .list_memory_items_needing_review(OffsetDateTime::now_utc(), None)
            .await?;
        items.retain(|item| item.correction_proposal_id.is_none());
        if let Some(limit) = limit {
            items.truncate(limit);
        }
        Ok(items)
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
                    && candidate.pending_correction_proposal_id.is_none()
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
            let expected = previous_item.clone();
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
            self.repo
                .save_memory_item_if_unchanged(&expected, &superseded)
                .await?;
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

    async fn list_commits_relevant_to_scope(
        &self,
        limit: Option<usize>,
        project: Option<&str>,
        cwd: Option<&str>,
    ) -> IndexResult<Vec<KnowledgeCommit>> {
        let mut commits = self.list_commits(None).await?;
        if project.is_none() && cwd.is_none() {
            if let Some(limit) = limit {
                commits.truncate(limit);
            }
            return Ok(commits);
        }

        let context = MemoryRankContext::orientation(project, cwd, None);
        let mut item_scope_cache = HashMap::new();
        let mut scoped_commits = Vec::new();
        for commit in commits {
            if self
                .knowledge_commit_matches_scope(&commit, context, &mut item_scope_cache)
                .await?
            {
                scoped_commits.push(commit);
                if limit.is_some_and(|limit| scoped_commits.len() == limit) {
                    break;
                }
            }
        }
        Ok(scoped_commits)
    }

    async fn knowledge_commit_matches_scope(
        &self,
        commit: &KnowledgeCommit,
        context: MemoryRankContext<'_>,
        item_scope_cache: &mut HashMap<Id, bool>,
    ) -> IndexResult<bool> {
        for item_id in commit.changes.iter().filter_map(|change| change.item_id) {
            let matches = if let Some(matches) = item_scope_cache.get(&item_id) {
                *matches
            } else {
                let matches = self
                    .repo
                    .get_memory_item(&item_id)
                    .await?
                    .as_ref()
                    .is_some_and(|item| memory_scope_matches(item, context));
                item_scope_cache.insert(item_id, matches);
                matches
            };
            if matches {
                return Ok(true);
            }
        }
        Ok(false)
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
        if options.enforce_scope {
            items.retain(|item| {
                Self::item_matches_boundary(
                    item,
                    options.project.as_deref(),
                    options.task.as_deref(),
                    options.cwd.as_deref(),
                )
            });
        }
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
        let observed_commits = self
            .repo
            .list_knowledge_commits_after(cursor.timestamp, limit)
            .await?;
        let commits = if options.omit_commits {
            Vec::new()
        } else {
            observed_commits.clone()
        };

        let mut next_timestamp = cursor.timestamp;
        for item in &items {
            next_timestamp = next_timestamp.max(item.updated_at);
        }
        for commit in &observed_commits {
            next_timestamp = next_timestamp.max(commit.created_at);
        }

        let next_commit_id = observed_commits
            .last()
            .map(|commit| commit.id)
            .or(cursor.commit_id);

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
        self.orient_with_task(input, None, None).await
    }

    /// Build an orientation packet narrowed to one caller-validated task.
    pub async fn orient_for_task(
        &self,
        input: OrientInput,
        task: String,
        task_project: String,
    ) -> IndexResult<OrientationPacket> {
        self.orient_with_task(input, Some(task), Some(task_project))
            .await
    }

    async fn orient_with_task(
        &self,
        input: OrientInput,
        task: Option<String>,
        task_project: Option<String>,
    ) -> IndexResult<OrientationPacket> {
        let started = Instant::now();
        let limit = input.limit.unwrap_or(20);
        let mut repository_context = self
            .resolve_repository_context(input.cwd.as_deref())
            .await?;
        if input.include_recent_commits {
            attach_recent_git_commits(&mut repository_context)?;
        }
        let resolution = match (input.project.as_deref(), task_project.as_deref()) {
            (None, Some(task_project)) => resolve_orientation_task_project(
                task.as_deref().unwrap_or("unknown task"),
                task_project,
                repository_context.as_ref(),
            ),
            _ => resolve_orientation_project(
                input.project.as_deref(),
                input.cwd.as_deref(),
                repository_context.as_ref(),
            ),
        };
        if resolution.source == OrientationResolutionSource::Task
            && resolution.requires_confirmation
        {
            return Err(IndexError::InvalidState(
                resolution
                    .ambiguity
                    .clone()
                    .unwrap_or_else(|| resolution.reason.clone()),
            ));
        }
        let identity = orientation_identity(&resolution, repository_context.as_ref());
        let cursor = self.current_cursor().await?;
        let active = self.list_active_memory(None).await?;
        let review = self
            .list_memory_needing_review(None)
            .await?
            .into_iter()
            .filter(|item| item.correction_proposal_id.is_none())
            .take(limit)
            .collect();
        let effective_project = resolution.selected_project.as_deref();
        let recent_commits = if input.include_recent_commits {
            self.list_commits_relevant_to_scope(
                Some(limit),
                effective_project,
                input.cwd.as_deref(),
            )
            .await?
        } else {
            Vec::new()
        };

        let repository_id = repository_context
            .as_ref()
            .map(|context| &context.repository.id);
        let repository_remote = repository_context
            .as_ref()
            .and_then(|context| context.repository.remote_url.as_deref());
        let rank_context = MemoryRankContext::orientation(
            effective_project,
            input.cwd.as_deref(),
            input.prompt.as_deref(),
        )
        .with_task(task.as_deref())
        .with_repository(repository_id, repository_remote)
        .requiring_text_match(orientation_requires_text_match(
            input.intent.as_ref(),
            input.prompt.as_deref(),
        ));
        let relevant_active = filter_relevant(active, rank_context);
        let has_task_boundary =
            has_orientation_task_boundary(effective_project, task.as_deref(), input.cwd.as_deref());
        let relevant_active = prioritize_current_plan_for_orientation(
            relevant_active,
            input.intent.as_ref(),
            input.prompt.as_deref(),
            has_task_boundary,
        );
        let mut relevant_review = filter_relevant(review, rank_context);
        relevant_review.truncate(limit);

        let active_decisions = take_kind(&relevant_active, MemoryKind::Decision, limit);
        let active_rules = take_kind(&relevant_active, MemoryKind::Rule, limit);
        let preferences = take_kind(&relevant_active, MemoryKind::Preference, limit);
        let limitations = take_kind(&relevant_active, MemoryKind::Limitation, limit);
        let handoffs = if matches!(input.intent, Some(BrainHarnessIntent::ResumeSession)) {
            take_kind(&relevant_active, MemoryKind::Handoff, limit)
        } else {
            Vec::new()
        };

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
        if repository_context.is_some() && input.cwd.is_some() {
            recommended_actions.push(
                "Before executing or exploring any actionable repository task, call \
                 repository-local procedure_match once with a bounded task-focused query copied \
                 from the user's operation request and the current cwd. Preserve concrete \
                 operation terms and identifiers verbatim; omit unrelated instructions and \
                 secret values. This applies even without remembered or learned wording and when \
                 the request says to use durable procedure memory. memory(action=list) is not a \
                 substitute."
                    .to_string(),
            );
        }
        if resolution.requires_confirmation {
            recommended_actions.push(
                "Ask the user to confirm the intended project before using project-scoped memory."
                    .to_string(),
            );
            if repository_context.is_some() && input.cwd.is_some() {
                recommended_actions.push(
                    "Project ambiguity blocks only project/task-scoped memory; it does not block \
                     the mandatory repository-local procedure_match for an actionable task."
                        .to_string(),
                );
            }
        }

        let scope = scope_label(effective_project, task.as_deref(), input.cwd.as_deref());
        let context_pack_parts = ContextPackParts {
            scope: &scope,
            cursor: &cursor,
            resolution: &resolution,
            repository_context: repository_context.as_ref(),
            project: effective_project,
            task: task.as_deref(),
            cwd: input.cwd.as_deref(),
            query: input.prompt.as_deref(),
            intent: input.intent.as_ref(),
            decisions: &active_decisions,
            rules: &active_rules,
            preferences: &preferences,
            limitations: &limitations,
            handoffs: &handoffs,
            review_needed: &relevant_review,
            commits: &recent_commits,
            ambiguities: &ambiguities,
            recommended_actions: &recommended_actions,
        };
        let hot_context_items = build_hot_context_items(&context_pack_parts);
        let hot_context_ids = hot_context_items.iter().map(|item| item.id).collect();
        let brain_loop = build_brain_loop(BrainLoopParts {
            scope: &scope,
            resolution: &resolution,
            project: effective_project,
            task: task.as_deref(),
            cwd: input.cwd.as_deref(),
            query: input.prompt.as_deref(),
            intent: input.intent.as_ref(),
            has_task_boundary,
            decisions: &active_decisions,
            rules: &active_rules,
            preferences: &preferences,
            limitations: &limitations,
            handoffs: &handoffs,
            review_needed: &relevant_review,
            ambiguities: &ambiguities,
        });
        let used_memory_candidate_ids = used_memory_candidate_ids(&brain_loop, &hot_context_items);
        let context_pack = build_context_pack(&context_pack_parts, &used_memory_candidate_ids);

        let returned_memory_ids = returned_orientation_memory_ids(&[
            &active_decisions,
            &active_rules,
            &preferences,
            &limitations,
            &handoffs,
            &relevant_review,
        ]);
        let memory_metadata = orientation_memory_metadata(&[
            &active_decisions,
            &active_rules,
            &preferences,
            &limitations,
            &handoffs,
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
            task,
            agent: input.agent,
            intent: input.intent,
            trace_id: Some(trace.id),
            prompt: input.prompt,
            scope,
            identity,
            resolution,
            repository_context,
            memory_cursor: cursor,
            hot_context_ids,
            hot_context_items,
            used_memory_candidate_ids,
            context_pack,
            brain_loop,
            active_decisions,
            active_rules,
            preferences,
            limitations,
            handoffs,
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
            let service = RepositoryService::new(self.db.clone());
            return match service.detect_repository(&cwd_path).await {
                Ok(_) => service.resolve_cwd(&cwd_path).await,
                Err(IndexError::NotFound(_) | IndexError::InvalidState(_)) => Ok(None),
                Err(error) => Err(error),
            };
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
        let matching_components =
            resolve_matching_components(&repository.id, &cwd_path, &checkout_path, components)?;
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

    async fn portable_procedure_evidence_path(
        &self,
        scope: &MemoryScope,
        receipt_path: &Path,
    ) -> IndexResult<String> {
        let canonical_receipt = receipt_path.canonicalize()?;
        let absolute_path = canonical_receipt.display().to_string();
        if !matches!(scope, MemoryScope::Repository { .. }) {
            return Ok(absolute_path);
        }

        let Some(parent) = canonical_receipt.parent() else {
            return Ok(absolute_path);
        };
        let parent = parent.to_string_lossy();
        let Some(context) = self.resolve_repository_context(Some(&parent)).await? else {
            return Ok(absolute_path);
        };
        if !repository_scope_stable_identity_matches(scope, &context) {
            return Ok(absolute_path);
        }
        let Some(checkout) = context.checkout.as_ref() else {
            return Ok(absolute_path);
        };
        let checkout_root = canonical_or_original(Path::new(&checkout.local_path));
        let Ok(relative_path) = canonical_receipt.strip_prefix(checkout_root) else {
            return Ok(absolute_path);
        };
        if relative_path.as_os_str().is_empty() {
            return Ok(absolute_path);
        }

        Ok(relative_path.display().to_string())
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

fn filter_relevant(items: Vec<MemoryItem>, context: MemoryRankContext<'_>) -> Vec<MemoryItem> {
    rank_memory_items(items, context)
        .into_iter()
        .map(|ranked| ranked.item)
        .collect()
}

fn orientation_requires_text_match(
    intent: Option<&BrainHarnessIntent>,
    query: Option<&str>,
) -> bool {
    query.is_some_and(|query| !query.trim().is_empty())
        && intent.is_some()
        && !matches!(
            intent,
            Some(BrainHarnessIntent::ResumeSession | BrainHarnessIntent::PrepareHandoff)
        )
}

fn prioritize_current_plan_for_orientation(
    items: Vec<MemoryItem>,
    intent: Option<&BrainHarnessIntent>,
    query: Option<&str>,
    has_task_boundary: bool,
) -> Vec<MemoryItem> {
    let suppress_older = matches!(
        intent,
        Some(BrainHarnessIntent::ResumeSession | BrainHarnessIntent::PrepareHandoff)
    );
    let promote_latest = suppress_older
        || should_prioritize_current_plan_for_plan_work(intent, query, has_task_boundary);
    if !promote_latest {
        return items;
    }

    prioritize_latest_current_plan(
        items,
        suppress_older,
        matches!(intent, Some(BrainHarnessIntent::PrepareHandoff)),
    )
}

fn should_prioritize_current_plan_for_plan_work(
    intent: Option<&BrainHarnessIntent>,
    query: Option<&str>,
    has_task_boundary: bool,
) -> bool {
    if !matches!(intent, Some(BrainHarnessIntent::PlanWork)) || !has_task_boundary {
        return false;
    }

    match query.map(str::trim) {
        None | Some("") => true,
        Some(query) => is_open_ended_plan_work_prompt(query),
    }
}

fn has_orientation_task_boundary(
    project: Option<&str>,
    task: Option<&str>,
    cwd: Option<&str>,
) -> bool {
    project.is_some_and(|project| !project.trim().is_empty())
        || task.is_some_and(|task| !task.trim().is_empty())
        || cwd.is_some_and(|cwd| !cwd.trim().is_empty())
}

fn prioritize_latest_current_plan(
    items: Vec<MemoryItem>,
    suppress_older: bool,
    collapse_scopes: bool,
) -> Vec<MemoryItem> {
    let mut latest_by_scope: HashMap<String, MemoryItem> = HashMap::new();
    for item in items.iter().filter(|item| is_current_plan_item(item)) {
        let scope_key = if collapse_scopes {
            "handoff".to_string()
        } else {
            current_plan_scope_key(&item.scope)
        };
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
    if suppress_older {
        prioritized.extend(items.into_iter().filter(|item| !is_current_plan_item(item)));
    } else {
        let latest_ids = prioritized
            .iter()
            .map(|item| item.id)
            .collect::<HashSet<_>>();
        prioritized.extend(
            items
                .into_iter()
                .filter(|item| !latest_ids.contains(&item.id)),
        );
    }
    prioritized
}

fn is_current_plan_item(item: &MemoryItem) -> bool {
    item.status == MemoryStatus::Active
        && matches!(item.kind, MemoryKind::Decision | MemoryKind::Rule)
        && item
            .tags
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
            repository_remote: repository_context
                .and_then(|context| context.repository.remote_url.as_deref())
                .and_then(normalize_remote_reference),
            component_names: component_names(repository_context),
            component_evidence: component_evidence(repository_context),
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
            | OrientationResolutionSource::Task
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
            | OrientationResolutionSource::Task
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
            repository_remote: context
                .repository
                .remote_url
                .as_deref()
                .and_then(normalize_remote_reference),
            component_names: component_names(Some(context)),
            component_evidence: component_evidence(Some(context)),
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

fn resolve_orientation_task_project(
    task: &str,
    task_project: &str,
    repository_context: Option<&RepositoryContext>,
) -> OrientationResolution {
    let candidates = project_candidates(repository_context);
    let mismatch = repository_context.is_some()
        && !candidates.is_empty()
        && !candidates
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(task_project));
    let ambiguity = mismatch.then(|| {
        format!(
            "Task '{task}' belongs to project '{task_project}', but cwd matched repository '{}' candidate(s): {}.",
            repository_context
                .expect("repository context exists when task project mismatches")
                .repository
                .name,
            candidates.join(", ")
        )
    });
    OrientationResolution {
        explicit_project: None,
        selected_project: Some(task_project.to_string()),
        source: OrientationResolutionSource::Task,
        confidence: 1.0,
        requires_confirmation: mismatch,
        reason: format!("Project '{task_project}' was derived from caller-supplied task '{task}'."),
        repository_name: repository_context.map(|context| context.repository.name.clone()),
        repository_remote: repository_context
            .and_then(|context| context.repository.remote_url.as_deref())
            .and_then(normalize_remote_reference),
        component_names: component_names(repository_context),
        component_evidence: component_evidence(repository_context),
        project_candidates: candidates,
        ambiguity,
    }
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

fn component_evidence(
    repository_context: Option<&RepositoryContext>,
) -> Vec<OrientationComponentEvidence> {
    repository_context
        .map(|context| {
            context
                .matching_components
                .iter()
                .map(|component| OrientationComponentEvidence {
                    name: component.name.clone(),
                    component_path: component.path.clone(),
                    source_path: component.source_path.clone(),
                    source_sha256: component.source_sha256.clone(),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn orientation_identity(
    resolution: &OrientationResolution,
    repository_context: Option<&RepositoryContext>,
) -> OrientationIdentity {
    let repository = repository_context.map(|context| OrientationRepositoryIdentity {
        repository_id: context.repository.id,
        name: context.repository.name.clone(),
        normalized_remote: context
            .repository
            .remote_url
            .as_deref()
            .and_then(normalize_remote_reference),
        checkout_id: context.checkout.as_ref().map(|checkout| checkout.id),
        checkout_root: context
            .checkout
            .as_ref()
            .map(|checkout| checkout.local_path.clone()),
        head_sha: context
            .checkout
            .as_ref()
            .and_then(|checkout| checkout.head_sha.clone()),
    });
    let status = if resolution.selected_project.is_some() && !resolution.requires_confirmation {
        OrientationProjectStatus::Authorized
    } else if repository_context.is_some() || !resolution.project_candidates.is_empty() {
        OrientationProjectStatus::RequiresConfirmation
    } else {
        OrientationProjectStatus::Unavailable
    };
    let mut project_link_ids = match (resolution.source, repository_context) {
        (
            OrientationResolutionSource::ComponentLink
            | OrientationResolutionSource::RepositoryLink,
            Some(context),
        ) => {
            let selected = resolution.selected_project.as_deref();
            candidate_links_for_context(context)
                .into_iter()
                .filter(|link| {
                    selected
                        .map(|project| link.project_name.eq_ignore_ascii_case(project))
                        .unwrap_or(true)
                })
                .map(|link| link.id)
                .collect()
        }
        (OrientationResolutionSource::Unresolved, Some(context)) => {
            candidate_links_for_context(context)
                .into_iter()
                .map(|link| link.id)
                .collect()
        }
        _ => Vec::new(),
    };
    project_link_ids.sort_by_key(std::string::ToString::to_string);
    project_link_ids.dedup();

    OrientationIdentity {
        repository,
        project: OrientationProjectIdentity {
            status,
            name: resolution.selected_project.clone(),
            source: resolution.source,
            candidates: resolution.project_candidates.clone(),
            project_link_ids,
            reason: resolution.reason.clone(),
            ambiguity: resolution.ambiguity.clone(),
        },
        components: resolution.component_evidence.clone(),
    }
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
    )
    .with_task(options.task.as_deref());
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

fn scope_label(project: Option<&str>, task: Option<&str>, cwd: Option<&str>) -> String {
    if let Some(task) = task {
        return match project {
            Some(project) => format!("task:{project}/{task}"),
            None => format!("task:{task}"),
        };
    }
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
    project: Option<&'a str>,
    task: Option<&'a str>,
    cwd: Option<&'a str>,
    query: Option<&'a str>,
    intent: Option<&'a BrainHarnessIntent>,
    decisions: &'a [MemoryItem],
    rules: &'a [MemoryItem],
    preferences: &'a [MemoryItem],
    limitations: &'a [MemoryItem],
    handoffs: &'a [MemoryItem],
    review_needed: &'a [MemoryItem],
    commits: &'a [KnowledgeCommit],
    ambiguities: &'a [String],
    recommended_actions: &'a [String],
}

struct BrainLoopParts<'a> {
    scope: &'a str,
    resolution: &'a OrientationResolution,
    project: Option<&'a str>,
    task: Option<&'a str>,
    cwd: Option<&'a str>,
    query: Option<&'a str>,
    intent: Option<&'a BrainHarnessIntent>,
    has_task_boundary: bool,
    decisions: &'a [MemoryItem],
    rules: &'a [MemoryItem],
    preferences: &'a [MemoryItem],
    limitations: &'a [MemoryItem],
    handoffs: &'a [MemoryItem],
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
            items: parts.handoffs,
            reason: "Active handoff matched the resume scope.",
            original_index: 4,
            score: 0.0,
        },
        BrainLoopGroup {
            items: parts.review_needed,
            reason: "Review-needed memory matched the orientation scope.",
            original_index: 5,
            score: 0.0,
        },
    ];
    let continuity_current_plan =
        should_pin_current_plan_in_brain_loop(parts.intent, parts.query, parts.has_task_boundary)
            && parts.decisions.first().is_some_and(is_current_plan_item);
    let follow_user_preference =
        matches!(parts.intent, Some(BrainHarnessIntent::FollowUserPreference))
            && !parts.preferences.is_empty();
    let resume_handoff = matches!(parts.intent, Some(BrainHarnessIntent::ResumeSession))
        && !parts.handoffs.is_empty();
    if parts.query.is_some_and(|query| !query.trim().is_empty()) {
        let context = MemoryRankContext::orientation(parts.project, parts.cwd, parts.query)
            .with_task(parts.task);
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
    if continuity_current_plan {
        groups[3].score = f32::INFINITY;
    }
    if follow_user_preference {
        groups[1].score = f32::INFINITY;
    }
    if resume_handoff {
        groups[4].score = f32::INFINITY;
    }
    if parts.query.is_some_and(|query| !query.trim().is_empty())
        || continuity_current_plan
        || follow_user_preference
        || resume_handoff
    {
        groups.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.original_index.cmp(&right.original_index))
        });
    }
    let mut offsets = [0usize; 6];

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

fn should_pin_current_plan_in_brain_loop(
    intent: Option<&BrainHarnessIntent>,
    query: Option<&str>,
    has_task_boundary: bool,
) -> bool {
    if matches!(
        intent,
        Some(BrainHarnessIntent::ResumeSession | BrainHarnessIntent::PrepareHandoff)
    ) {
        return true;
    }

    if !matches!(intent, Some(BrainHarnessIntent::PlanWork)) || !has_task_boundary {
        return false;
    }

    matches!(query.map(str::trim), None | Some(""))
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

fn used_memory_candidate_ids(
    brain_loop: &BrainLoop,
    hot_context_items: &[BrainLoopItem],
) -> Vec<Id> {
    let mut ids = Vec::new();
    for item in hot_context_items.iter().chain(&brain_loop.top_items) {
        if !ids.contains(&item.id) {
            ids.push(item.id);
        }
    }
    ids
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
            OrientationResolutionSource::Task => "explicit task",
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

fn build_context_pack(parts: &ContextPackParts<'_>, used_memory_candidate_ids: &[Id]) -> String {
    let mut lines = vec![
        format!("# Context Pack: {}", parts.scope),
        String::new(),
        format!("- Memory cursor timestamp: {}", parts.cursor.timestamp),
    ];
    if let Some(commit_id) = parts.cursor.commit_id {
        lines.push(format!("- Latest knowledge commit: {commit_id}"));
    }
    if !used_memory_candidate_ids.is_empty() {
        let ids = used_memory_candidate_ids
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!(
            "- used_memory_candidate_ids: {ids}; submit the subset that shaped your answer in telemetry used_memory_ids."
        ));
    }
    append_resolution_section(&mut lines, parts.resolution);
    if let Some(task) = parts.task {
        lines.push(format!("- Selected task: {task}"));
    }
    append_repository_section(&mut lines, parts.repository_context);
    append_hot_context_section(&mut lines, parts);
    append_memory_section(&mut lines, "Active Decisions", parts.decisions);
    append_memory_section(&mut lines, "Active Rules", parts.rules);
    append_memory_section(&mut lines, "Preferences", parts.preferences);
    append_memory_section(&mut lines, "Limitations", parts.limitations);
    append_memory_section(&mut lines, "Handoffs", parts.handoffs);
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

fn build_hot_context_items(parts: &ContextPackParts<'_>) -> Vec<BrainLoopItem> {
    intent_matched_reviewed_preferences(parts)
        .into_iter()
        .map(|item| brain_loop_item(item, "Intent-matched reviewed preference for this request."))
        .collect()
}

fn append_hot_context_section(lines: &mut Vec<String>, parts: &ContextPackParts<'_>) {
    let hot_preferences = intent_matched_reviewed_preferences(parts);
    if hot_preferences.is_empty() {
        return;
    }

    lines.push(String::new());
    lines.push("## Hot Context".to_string());
    lines.push(
        "- Intent-matched reviewed preferences. Read these before lower-priority memory."
            .to_string(),
    );
    let ids = hot_preferences
        .iter()
        .map(|item| item.id.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    lines.push(format!(
        "- Use these memory IDs in telemetry used_memory_ids when they shape behavior: {ids}"
    ));
    lines.push("### Reviewed Preferences".to_string());
    for item in hot_preferences {
        let metadata = item.trust_metadata();
        lines.push(format!(
            "- Memory {}: {}: {}",
            item.id, item.title, item.content
        ));
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

fn intent_matched_reviewed_preferences<'a>(parts: &ContextPackParts<'a>) -> Vec<&'a MemoryItem> {
    if !matches!(parts.intent, Some(BrainHarnessIntent::FollowUserPreference)) {
        return Vec::new();
    }

    let has_query = parts.query.is_some_and(|query| !query.trim().is_empty());
    let context =
        MemoryRankContext::orientation(parts.project, parts.cwd, parts.query).with_task(parts.task);
    parts
        .preferences
        .iter()
        .filter(|item| item.trust_metadata().review_state == MemoryReviewState::Reviewed)
        .filter(|item| {
            !has_query
                || rank_memory_item((*item).clone(), context)
                    .is_some_and(|ranked| ranked.components.text > 0.0)
        })
        .take(ORIENT_HOT_CONTEXT_ITEM_LIMIT)
        .collect()
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
    path.canonicalize().unwrap_or_else(|_| {
        let mut normalized = PathBuf::new();
        for component in path.components() {
            match component {
                Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
                Component::RootDir => normalized.push(Path::new("/")),
                Component::CurDir => {}
                Component::ParentDir => {
                    normalized.pop();
                }
                Component::Normal(value) => normalized.push(value),
            }
        }
        normalized
    })
}

fn validate_procedure_receipt(
    procedure: &ProcedureCard,
    receipt: &ProcedureVerificationReceipt,
) -> IndexResult<()> {
    if receipt.command.trim() != procedure.verification.command.trim() {
        return Err(IndexError::InvalidState(
            "verification receipt command does not match the procedure card".to_string(),
        ));
    }
    if receipt.exit_code != procedure.verification.expected_exit_code {
        return Err(IndexError::InvalidState(format!(
            "verification receipt exit code {} does not match expected {}",
            receipt.exit_code, procedure.verification.expected_exit_code
        )));
    }
    if !receipt
        .output
        .contains(&procedure.verification.expected_output_contains)
    {
        return Err(IndexError::InvalidState(
            "verification receipt output does not contain the expected success marker".to_string(),
        ));
    }
    for prerequisite in &procedure.prerequisites {
        let observed = receipt.conditions.get(&prerequisite.key).ok_or_else(|| {
            IndexError::InvalidState(format!(
                "verification receipt is missing prerequisite condition {}",
                prerequisite.key
            ))
        })?;
        if normalize_condition(observed) != normalize_condition(&prerequisite.expected) {
            return Err(IndexError::InvalidState(format!(
                "verification receipt condition {} does not match the procedure prerequisite",
                prerequisite.key
            )));
        }
    }
    Ok(())
}

fn observe_procedure_prerequisite(
    prerequisite: &ProcedurePrerequisite,
    source: &ProcedurePrerequisiteSource,
    checkout_root: Option<&Path>,
) -> ProcedureConditionObservation {
    let unavailable =
        |detail: String, source_sha256: Option<String>| ProcedureConditionObservation {
            condition_key: prerequisite.key.clone(),
            source: source.clone(),
            status: ProcedureConditionObservationStatus::Unavailable,
            source_sha256,
            detail,
        };
    let Some(checkout_root) = checkout_root else {
        return unavailable("no current checkout root was resolved".to_string(), None);
    };
    let (relative_path, key_path) = match source {
        ProcedurePrerequisiteSource::Toml {
            relative_path,
            key_path,
        } => (relative_path.as_str(), key_path.as_slice()),
    };
    let bytes = match read_tracked_condition_source(checkout_root, relative_path) {
        Ok(bytes) => bytes,
        Err(detail) => return unavailable(detail, None),
    };
    let source_sha256 = Some(sha256_hex(&bytes));
    let contents = match std::str::from_utf8(&bytes) {
        Ok(contents) => contents,
        Err(_) => {
            return unavailable(
                "trusted source is not UTF-8 text".to_string(),
                source_sha256,
            )
        }
    };
    let document = match toml::from_str::<toml::Value>(contents) {
        Ok(document) => document,
        Err(_) => {
            return unavailable(
                "trusted source is not valid TOML".to_string(),
                source_sha256,
            )
        }
    };
    let mut selected = &document;
    for key in key_path {
        let Some(next) = selected.get(key) else {
            return unavailable(
                "configured TOML key path is absent".to_string(),
                source_sha256,
            );
        };
        selected = next;
    }
    let observed = match selected {
        toml::Value::String(value) => value.clone(),
        toml::Value::Integer(value) => value.to_string(),
        toml::Value::Float(value) => value.to_string(),
        toml::Value::Boolean(value) => value.to_string(),
        toml::Value::Datetime(value) => value.to_string(),
        toml::Value::Array(_) | toml::Value::Table(_) => {
            return unavailable(
                "configured TOML key path does not select a scalar".to_string(),
                source_sha256,
            )
        }
    };
    let status = if normalize_condition(&observed) == normalize_condition(&prerequisite.expected) {
        ProcedureConditionObservationStatus::Matched
    } else {
        ProcedureConditionObservationStatus::Mismatched
    };
    ProcedureConditionObservation {
        condition_key: prerequisite.key.clone(),
        source: source.clone(),
        status,
        source_sha256,
        detail: match status {
            ProcedureConditionObservationStatus::Matched => {
                "trusted current-checkout source matched the verified prerequisite".to_string()
            }
            ProcedureConditionObservationStatus::Mismatched => {
                "trusted current-checkout source did not match the verified prerequisite"
                    .to_string()
            }
            ProcedureConditionObservationStatus::Unavailable => unreachable!(),
        },
    }
}

fn read_tracked_condition_source(
    checkout_root: &Path,
    relative_path: &str,
) -> Result<Vec<u8>, String> {
    let relative = Path::new(relative_path);
    if !is_safe_checkout_relative_path(relative) {
        return Err("configured source path is not a safe checkout-relative path".to_string());
    }
    let checkout_root = checkout_root
        .canonicalize()
        .map_err(|_| "current checkout root is unavailable".to_string())?;
    let mut source = checkout_root.clone();
    for component in relative.components() {
        let Component::Normal(part) = component else {
            return Err("configured source path is not a safe checkout-relative path".to_string());
        };
        source.push(part);
        let metadata = fs::symlink_metadata(&source)
            .map_err(|_| "configured source file is unavailable".to_string())?;
        if metadata.file_type().is_symlink() {
            return Err("configured source path contains a symlink".to_string());
        }
    }
    let source = source
        .canonicalize()
        .map_err(|_| "configured source file is unavailable".to_string())?;
    if !source.starts_with(&checkout_root) {
        return Err("configured source path escapes the current checkout".to_string());
    }
    let metadata =
        fs::metadata(&source).map_err(|_| "configured source file is unavailable".to_string())?;
    if !metadata.is_file() {
        return Err("configured source path is not a regular file".to_string());
    }
    if metadata.len() > MAX_CONDITION_SOURCE_BYTES {
        return Err(format!(
            "configured source file exceeds the {}-byte limit",
            MAX_CONDITION_SOURCE_BYTES
        ));
    }
    let tracked = Command::new("git")
        .arg("-C")
        .arg(&checkout_root)
        .args(["--literal-pathspecs", "ls-files", "--error-unmatch", "--"])
        .arg(relative)
        .output()
        .map_err(|_| "Git tracking state could not be inspected".to_string())?;
    if !tracked.status.success() {
        return Err("configured source file is not Git-tracked".to_string());
    }
    let bytes = fs::read(&source)
        .map_err(|_| "configured source file could not be read safely".to_string())?;
    if bytes.len() as u64 > MAX_CONDITION_SOURCE_BYTES {
        return Err(format!(
            "configured source file exceeds the {}-byte limit",
            MAX_CONDITION_SOURCE_BYTES
        ));
    }
    Ok(bytes)
}

fn suggest_operation_evidence(
    checkout_root: &Path,
    query: &str,
    components: &[OrientationComponentEvidence],
) -> Option<OperationEvidenceCandidate> {
    let checkout_root = checkout_root.canonicalize().ok()?;
    let query_terms = operation_query_terms(query);
    if query_terms.is_empty() {
        return None;
    }

    let output = Command::new("git")
        .arg("-C")
        .arg(&checkout_root)
        .args([
            "ls-files",
            "-z",
            "--",
            ":(icase,glob)**/runbook*/**/*.md",
            ":(icase,glob)**/*runbook*.md",
        ])
        .output()
        .ok()?;
    if !output.status.success() || output.stdout.len() > MAX_OPERATION_EVIDENCE_INDEX_BYTES {
        return None;
    }

    let component_sources = components
        .iter()
        .filter_map(|component| component.source_path.as_deref())
        .collect::<HashSet<_>>();
    let mut candidates = Vec::new();
    for raw_path in output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
    {
        if candidates.len() >= MAX_OPERATION_EVIDENCE_CANDIDATES {
            return None;
        }
        let path = std::str::from_utf8(raw_path).ok()?;
        if component_sources.contains(path) || !is_safe_checkout_relative_path(Path::new(path)) {
            continue;
        }
        let path_terms = ascii_terms(path);
        let filename_terms = Path::new(path)
            .file_stem()
            .and_then(|value| value.to_str())
            .map(ascii_terms)
            .unwrap_or_default();
        let matched_terms = query_terms
            .iter()
            .filter(|term| path_terms.contains(term.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        if matched_terms.is_empty() {
            continue;
        }
        let filename_matches = matched_terms
            .iter()
            .filter(|term| filename_terms.contains(term.as_str()))
            .count();
        let Ok(bytes) = read_tracked_condition_source(&checkout_root, path) else {
            continue;
        };
        let Ok(resolved_path) = checkout_root.join(path).canonicalize() else {
            continue;
        };
        if !resolved_path.starts_with(&checkout_root) {
            continue;
        }
        candidates.push((
            matched_terms.len(),
            filename_matches,
            OperationEvidenceCandidate {
                path: path.to_string(),
                resolved_path: resolved_path.display().to_string(),
                source_sha256: sha256_hex(&bytes),
                reason: format!(
                    "highest-ranked Git-tracked runbook path with a unique query-term score: {}",
                    matched_terms.join(", ")
                ),
                required_before_final_abstention: true,
                allowed_when_project_requires_confirmation: true,
                authorizes_procedure_execution: false,
            },
        ));
    }
    candidates.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| right.1.cmp(&left.1))
            .then_with(|| left.2.path.cmp(&right.2.path))
    });
    let best = candidates.first()?;
    if candidates
        .get(1)
        .is_some_and(|next| (next.0, next.1) == (best.0, best.1))
    {
        return None;
    }
    Some(best.2.clone())
}

fn operation_query_terms(query: &str) -> Vec<String> {
    let mut terms = ascii_terms(query)
        .into_iter()
        .filter(|term| term.len() >= 4)
        .filter(|term| !is_generic_operation_term(term))
        .collect::<Vec<_>>();
    terms.sort();
    terms.dedup();
    terms
}

fn ascii_terms(value: &str) -> HashSet<String> {
    value
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|term| !term.is_empty())
        .map(str::to_ascii_lowercase)
        .collect()
}

fn is_generic_operation_term(term: &str) -> bool {
    matches!(
        term,
        "applicable"
            | "available"
            | "belongs"
            | "checkout"
            | "context"
            | "current"
            | "durable"
            | "evaluation"
            | "fresh"
            | "handle"
            | "identity"
            | "inspect"
            | "local"
            | "memory"
            | "only"
            | "procedure"
            | "repository"
            | "session"
            | "using"
    )
}

fn is_safe_checkout_relative_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn procedure_applicability(
    item: &MemoryItem,
    conditions: &BTreeMap<String, String>,
    checkout_root: Option<&Path>,
    now: OffsetDateTime,
) -> ProcedureApplicability {
    let mut reasons = Vec::new();
    let mut unresolved_condition_keys = Vec::new();
    let mut condition_observations = Vec::new();
    let Some(procedure) = &item.procedure else {
        return ProcedureApplicability {
            memory_id: item.id,
            title: item.title.clone(),
            applicable: false,
            reasons: vec!["Structured procedure details are missing.".to_string()],
            unresolved_condition_keys,
            condition_observations,
        };
    };

    if item.status != MemoryStatus::Active {
        reasons.push(format!(
            "Procedure status is {} instead of active.",
            item.status
        ));
    }
    if !procedure.is_verified_at(now) {
        if procedure
            .expires_at
            .is_some_and(|expires_at| expires_at <= now)
        {
            reasons.push("Procedure verification has expired.".to_string());
        } else {
            reasons.push("Procedure does not have complete verification metadata.".to_string());
        }
    }
    for prerequisite in &procedure.prerequisites {
        if let Some(source) = &prerequisite.source {
            let observation = observe_procedure_prerequisite(prerequisite, source, checkout_root);
            match observation.status {
                ProcedureConditionObservationStatus::Matched => {}
                ProcedureConditionObservationStatus::Mismatched => reasons.push(format!(
                    "Condition '{}' from its trusted current-checkout source does not match the verified value.",
                    prerequisite.key
                )),
                ProcedureConditionObservationStatus::Unavailable => reasons.push(format!(
                    "Condition '{}' could not be resolved from its trusted current-checkout source: {}",
                    prerequisite.key, observation.detail
                )),
            }
            condition_observations.push(observation);
            continue;
        }
        match conditions.get(&prerequisite.key) {
            None => {
                reasons.push(format!(
                    "Required condition '{}' was not supplied; abstaining instead of guessing.",
                    prerequisite.key
                ));
                unresolved_condition_keys.push(prerequisite.key.clone());
            }
            Some(observed)
                if normalize_condition(observed) != normalize_condition(&prerequisite.expected) =>
            {
                reasons.push(format!(
                    "Condition '{}' does not match the verified value.",
                    prerequisite.key
                ));
                unresolved_condition_keys.push(prerequisite.key.clone());
            }
            Some(_) => {}
        }
    }

    match (
        procedure.verification.evidence_path.as_deref(),
        procedure.verification.evidence_sha256.as_deref(),
    ) {
        (Some(path), Some(expected_hash)) => {
            let path = resolve_procedure_evidence_path(path, checkout_root);
            match fs::read(&path) {
                Ok(bytes) if sha256_hex(&bytes) == expected_hash => {}
                Ok(_) => reasons.push(format!(
                    "Verification receipt hash changed at {}.",
                    path.display()
                )),
                Err(_) => reasons.push(format!(
                    "Verification receipt is unavailable at {}.",
                    path.display()
                )),
            }
        }
        _ => reasons.push("Verification receipt path or hash is missing.".to_string()),
    }

    let applicable = reasons.is_empty();
    if applicable {
        reasons.push(
            "Scope, prerequisites, freshness, and verification receipt hash all match.".to_string(),
        );
    }
    ProcedureApplicability {
        memory_id: item.id,
        title: item.title.clone(),
        applicable,
        reasons,
        unresolved_condition_keys,
        condition_observations,
    }
}

fn procedure_scope_matches(
    scope: &MemoryScope,
    project: Option<&str>,
    cwd: Option<&str>,
    repository_context: Option<&RepositoryContext>,
) -> bool {
    match scope {
        MemoryScope::Global | MemoryScope::User => true,
        MemoryScope::Project { project_name, .. } => {
            project.is_some_and(|project| project_name.eq_ignore_ascii_case(project))
        }
        MemoryScope::Repository { local_path, .. } => {
            let stable_identity_match = repository_context
                .is_some_and(|context| repository_scope_stable_identity_matches(scope, context));
            let checkout_path_match =
                cwd.zip(local_path.as_deref())
                    .is_some_and(|(cwd, local_path)| {
                        canonical_or_original(Path::new(cwd))
                            .starts_with(canonical_or_original(Path::new(local_path)))
                    });
            stable_identity_match || checkout_path_match
        }
        MemoryScope::Task { .. }
        | MemoryScope::Entity { .. }
        | MemoryScope::Session { .. }
        | MemoryScope::Custom { .. } => false,
    }
}

fn repository_scope_stable_identity_matches(
    scope: &MemoryScope,
    context: &RepositoryContext,
) -> bool {
    let MemoryScope::Repository {
        repository_id,
        remote_url,
        ..
    } = scope
    else {
        return false;
    };

    repository_id.is_some_and(|id| id == context.repository.id)
        || match (
            remote_url.as_deref().and_then(normalize_remote_reference),
            context
                .repository
                .remote_url
                .as_deref()
                .and_then(normalize_remote_reference),
        ) {
            (Some(expected), Some(actual)) => expected == actual,
            _ => false,
        }
}

fn resolve_procedure_evidence_path(path: &str, checkout_root: Option<&Path>) -> PathBuf {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        path
    } else if let Some(root) = checkout_root {
        root.join(path)
    } else {
        path
    }
}

fn normalize_condition(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn normalized_boundary_value(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

fn validate_proposal_pair(
    proposal: &CorrectionProposal,
    obsolete: &MemoryItem,
    replacement: &MemoryItem,
) -> IndexResult<()> {
    if proposal.obsolete_id != obsolete.id || proposal.replacement_id != replacement.id {
        return Err(IndexError::InvalidState(format!(
            "correction proposal {} no longer names its exact memory pair",
            proposal.id
        )));
    }
    if proposal.memory_kind != obsolete.kind
        || proposal.memory_kind != replacement.kind
        || proposal.scope != obsolete.scope
        || proposal.scope != replacement.scope
    {
        return Err(IndexError::InvalidState(format!(
            "correction proposal {} no longer has its exact kind and scope",
            proposal.id
        )));
    }
    Ok(())
}

#[derive(Serialize)]
struct CorrectionDigestItem<'a> {
    id: &'a Id,
    kind: &'a MemoryKind,
    title: &'a str,
    content: &'a str,
    scope: &'a MemoryScope,
    origin: &'a ClaimOrigin,
    writer: &'a WriterProvenance,
    evidence: &'a [EvidenceRef],
    confidence: f32,
    status: MemoryStatus,
    supersedes: &'a [Id],
    tags: &'a [String],
    review_after: Option<OffsetDateTime>,
    archive: &'a Option<engram_core::memory::ArchiveMetadata>,
    procedure: &'a Option<ProcedureCard>,
    correction_proposal_id: Option<Id>,
    pending_correction_proposal_id: Option<Id>,
}

impl<'a> From<&'a MemoryItem> for CorrectionDigestItem<'a> {
    fn from(item: &'a MemoryItem) -> Self {
        Self {
            id: &item.id,
            kind: &item.kind,
            title: &item.title,
            content: &item.content,
            scope: &item.scope,
            origin: &item.origin,
            writer: &item.writer,
            evidence: &item.evidence,
            confidence: item.confidence.value(),
            status: item.status,
            supersedes: &item.supersedes,
            tags: &item.tags,
            review_after: item.review_after,
            archive: &item.archive,
            procedure: &item.procedure,
            correction_proposal_id: item.correction_proposal_id,
            pending_correction_proposal_id: item.pending_correction_proposal_id,
        }
    }
}

#[derive(Serialize)]
struct CorrectionDigestPayload<'a> {
    schema_version: u32,
    proposal_id: &'a Id,
    obsolete_id: &'a Id,
    replacement_id: &'a Id,
    memory_kind: &'a MemoryKind,
    scope: &'a MemoryScope,
    obsolete: CorrectionDigestItem<'a>,
    replacement: CorrectionDigestItem<'a>,
}

fn correction_proposal_digest(
    proposal: &CorrectionProposal,
    obsolete: &MemoryItem,
    replacement: &MemoryItem,
) -> IndexResult<String> {
    if proposal.digest_schema_version != 1 {
        return Err(IndexError::InvalidState(format!(
            "correction proposal {} uses unsupported digest schema version {}",
            proposal.id, proposal.digest_schema_version
        )));
    }
    let payload = CorrectionDigestPayload {
        schema_version: proposal.digest_schema_version,
        proposal_id: &proposal.id,
        obsolete_id: &proposal.obsolete_id,
        replacement_id: &proposal.replacement_id,
        memory_kind: &proposal.memory_kind,
        scope: &proposal.scope,
        obsolete: obsolete.into(),
        replacement: replacement.into(),
    };
    let bytes = serde_json::to_vec(&payload).map_err(|error| {
        IndexError::Parse(format!("cannot digest correction proposal: {error}"))
    })?;
    Ok(sha256_hex(&bytes))
}

fn memory_item_references_text(item: &MemoryItem, id_text: &str) -> bool {
    item.supersedes.iter().any(|id| id.to_string() == id_text)
        || item.evidence.iter().any(|evidence| {
            evidence.target.contains(id_text)
                || evidence
                    .summary
                    .as_deref()
                    .is_some_and(|summary| summary.contains(id_text))
                || evidence
                    .excerpt
                    .as_deref()
                    .is_some_and(|excerpt| excerpt.contains(id_text))
        })
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
    match (&item.kind, &item.procedure) {
        (MemoryKind::Procedure, Some(procedure)) => {
            if procedure.task.trim().is_empty() {
                return Err(IndexError::Parse(
                    "procedure task must not be empty".to_string(),
                ));
            }
            if procedure.commands.is_empty()
                || procedure
                    .commands
                    .iter()
                    .any(|command| command.trim().is_empty())
            {
                return Err(IndexError::Parse(
                    "procedure requires at least one non-empty command".to_string(),
                ));
            }
            if procedure.verification.command.trim().is_empty()
                || procedure
                    .verification
                    .expected_output_contains
                    .trim()
                    .is_empty()
            {
                return Err(IndexError::Parse(
                    "procedure verification command and expected output marker are required"
                        .to_string(),
                ));
            }
            if procedure.prerequisites.len() > MAX_PROCEDURE_PREREQUISITES {
                return Err(IndexError::Parse(format!(
                    "procedure supports at most {MAX_PROCEDURE_PREREQUISITES} prerequisites"
                )));
            }
            let mut prerequisite_keys = BTreeSet::new();
            for prerequisite in &procedure.prerequisites {
                if prerequisite.key.trim().is_empty() || prerequisite.expected.trim().is_empty() {
                    return Err(IndexError::Parse(
                        "procedure prerequisites require non-empty keys and expected values"
                            .to_string(),
                    ));
                }
                if !prerequisite_keys.insert(prerequisite.key.to_ascii_lowercase()) {
                    return Err(IndexError::Parse(
                        "procedure prerequisite keys must be unique".to_string(),
                    ));
                }
                if let Some(ProcedurePrerequisiteSource::Toml {
                    relative_path,
                    key_path,
                }) = &prerequisite.source
                {
                    if !is_safe_checkout_relative_path(Path::new(relative_path)) {
                        return Err(IndexError::Parse(
                            "procedure prerequisite source requires a safe checkout-relative path"
                                .to_string(),
                        ));
                    }
                    if key_path.is_empty() || key_path.iter().any(|key| key.trim().is_empty()) {
                        return Err(IndexError::Parse(
                            "procedure prerequisite TOML source requires a non-empty key path"
                                .to_string(),
                        ));
                    }
                }
            }
        }
        (MemoryKind::Procedure, None) => {
            return Err(IndexError::Parse(
                "procedure memory requires structured procedure details".to_string(),
            ));
        }
        (_, Some(_)) => {
            return Err(IndexError::Parse(
                "structured procedure details require memory kind procedure".to_string(),
            ));
        }
        (_, None) => {}
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

    if item.kind == MemoryKind::Procedure
        && !item
            .procedure
            .as_ref()
            .is_some_and(|procedure| procedure.is_verified_at(OffsetDateTime::now_utc()))
    {
        return item.with_status(MemoryStatus::NeedsReview);
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
        MemoryKind::Decision | MemoryKind::Rule | MemoryKind::Limitation | MemoryKind::Procedure
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
        MemoryReviewState, MemoryScope, MemoryStatus, ModelIdentity, ProcedureCard,
        ProcedurePrerequisite, ProcedureVerification,
    };
    use engram_core::repository::{
        GitRepository, LocalCheckout, MonorepoComponent, ProjectRepositoryLink,
        ProjectRepositoryRole,
    };
    use engram_core::search::SearchLayer;
    use engram_core::telemetry::AgentFeedback;
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

    async fn correction_pair(
        service: &MemoryService,
        scope: MemoryScope,
        label: &str,
    ) -> (MemoryItem, MemoryItem) {
        let obsolete = service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Decision,
                    format!("Obsolete {label}"),
                    format!("Use the obsolete {label} guidance."),
                    scope.clone(),
                    ClaimOrigin::UserStated,
                    writer(),
                )
                .with_evidence(EvidenceRef::new(
                    EvidenceKind::File,
                    format!("docs/{label}-obsolete.md"),
                )),
            )
            .await
            .unwrap();
        let replacement = service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Decision,
                    format!("Replacement {label}"),
                    format!("Use the replacement {label} guidance."),
                    scope,
                    ClaimOrigin::UserCorrected,
                    writer(),
                )
                .with_evidence(EvidenceRef::new(
                    EvidenceKind::File,
                    format!("docs/{label}-replacement.md"),
                )),
            )
            .await
            .unwrap();
        (obsolete, replacement)
    }

    fn correction_proposal_input(obsolete_id: Id, label: &str) -> CorrectionProposalInput {
        CorrectionProposalInput {
            obsolete_id,
            title: format!("Proposed {label}"),
            content: format!("Use the proposed {label} guidance."),
            writer: writer(),
            evidence: vec![EvidenceRef::new(
                EvidenceKind::File,
                format!("docs/{label}-proposal.md"),
            )],
            procedure: None,
        }
    }

    fn procedure_card(task: &str, command: &str, marker: &str) -> ProcedureCard {
        ProcedureCard::new(
            task,
            vec![command.to_string()],
            ProcedureVerification::new(command, 0, marker),
        )
    }

    async fn active_procedure(
        service: &MemoryService,
        receipt_path: &Path,
        title: &str,
        task: &str,
        command: &str,
        marker: &str,
    ) -> MemoryItem {
        let receipt = ProcedureVerificationReceipt {
            command: command.to_string(),
            exit_code: 0,
            output: format!("verification: {marker}"),
            conditions: BTreeMap::new(),
        };
        let receipt_bytes = serde_json::to_vec_pretty(&receipt).unwrap();
        fs::write(receipt_path, &receipt_bytes).unwrap();
        let mut card = procedure_card(task, command, marker);
        card.verification.evidence_path = Some(receipt_path.display().to_string());
        card.verification.evidence_sha256 = Some(sha256_hex(&receipt_bytes));
        card.verification.verified_at = Some(OffsetDateTime::now_utc());
        card.expires_at = Some(OffsetDateTime::now_utc() + time::Duration::days(30));
        service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Procedure,
                    title,
                    format!("Verified procedure for {task}."),
                    MemoryScope::project("engram"),
                    ClaimOrigin::UserStated,
                    writer(),
                )
                .with_procedure(card)
                .with_evidence(EvidenceRef::new(
                    EvidenceKind::File,
                    receipt_path.display().to_string(),
                )),
            )
            .await
            .unwrap()
    }

    fn procedure_correction_input(obsolete_id: Id) -> CorrectionProposalInput {
        CorrectionProposalInput {
            obsolete_id,
            title: "Corrected safe check procedure".to_string(),
            content: "Run the corrected safe check procedure.".to_string(),
            writer: writer(),
            evidence: vec![EvidenceRef::new(
                EvidenceKind::File,
                "docs/corrected-safe-check.md",
            )],
            procedure: Some(procedure_card(
                "run corrected safe check",
                "cargo test -p engram-index corrected_safe_check",
                "CORRECTED_SAFE_CHECK_OK",
            )),
        }
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
    async fn promote_observation_creates_active_item_with_review_assertion() {
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
        let trust = item.trust_metadata();
        assert!(trust.review_asserted);
        assert!(!trust.reviewed);
        assert_eq!(trust.review_state, MemoryReviewState::ActiveUnreviewed);

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
    async fn capture_current_plan_rejects_locked_same_scope_before_writing() {
        let service = setup_service().await;
        let old = service
            .capture_current_plan(current_plan_input(
                "engram",
                "Locked current plan",
                "Keep this plan active while its correction is pending.",
            ))
            .await
            .unwrap()
            .item;
        let (proposal, _, _) = service
            .propose_correction(
                correction_proposal_input(old.id, "locked-current-plan"),
                Some("engram"),
                None,
                None,
            )
            .await
            .unwrap();

        let error = service
            .capture_current_plan(current_plan_input(
                "engram",
                "Must not be persisted",
                "A failed capture must not leave a second active plan.",
            ))
            .await
            .unwrap_err();
        assert!(error.to_string().contains(&proposal.id.to_string()));
        let all = service.list_memory(None, None).await.unwrap();
        assert!(!all.iter().any(|item| item.title == "Must not be persisted"));
        assert_eq!(
            service.get_memory(&old.id).await.unwrap().unwrap().status,
            MemoryStatus::Active
        );
    }

    #[tokio::test]
    async fn capture_current_plan_does_not_supersede_non_guidance_current_plan_tags() {
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
        let tagged_fact = service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::ProjectFact,
                    "Non-guidance fact with current-plan tag",
                    "This fact records validation evidence and should not be replaced by \
                     current-plan capture lifecycle.",
                    MemoryScope::project("engram"),
                    ClaimOrigin::ToolResult,
                    writer(),
                )
                .with_status(MemoryStatus::Active)
                .with_tag(CURRENT_PLAN_TAG),
            )
            .await
            .unwrap();

        let mut input = current_plan_input(
            "engram",
            "New current plan",
            "Resume from the new current plan.",
        );
        input.create_commit = true;
        let capture = service.capture_current_plan(input).await.unwrap();
        let new = capture.item;

        assert!(new.supersedes.contains(&old.id));
        assert!(!new.supersedes.contains(&tagged_fact.id));
        let commit = capture
            .commit
            .expect("superseding current-plan capture should create a commit");
        assert!(commit.changes.iter().any(|change| {
            change.change_type == MemoryChangeType::Superseded && change.item_id == Some(old.id)
        }));
        assert!(!commit
            .changes
            .iter()
            .any(|change| change.item_id == Some(tagged_fact.id)));
        assert_eq!(
            service.get_memory(&old.id).await.unwrap().unwrap().status,
            MemoryStatus::Superseded
        );
        assert_eq!(
            service
                .get_memory(&tagged_fact.id)
                .await
                .unwrap()
                .unwrap()
                .status,
            MemoryStatus::Active
        );
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
    async fn promote_memory_activates_candidate_with_review_assertion() {
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
        let trust = promoted.trust_metadata();
        assert!(trust.review_asserted);
        assert!(!trust.reviewed);
        assert_eq!(trust.review_state, MemoryReviewState::ActiveUnreviewed);
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
    async fn correct_memory_requires_active_evidenced_same_scope_user_correction() {
        let service = setup_service().await;
        let obsolete = service
            .capture_memory(memory_item("Correction target"))
            .await
            .unwrap();
        let replacement = service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Decision,
                    "User correction",
                    "Use the corrected workflow.",
                    MemoryScope::project("engram"),
                    ClaimOrigin::UserCorrected,
                    writer(),
                )
                .with_evidence(EvidenceRef::new(EvidenceKind::File, "docs/correction.md")),
            )
            .await
            .unwrap();

        let error = service
            .correct_memory(
                &obsolete.id,
                &replacement.id,
                "Wrong project.",
                Some("atlas"),
                None,
                None,
            )
            .await
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("outside the resolved correction authorization boundary"));
        assert_eq!(
            service
                .get_memory(&obsolete.id)
                .await
                .unwrap()
                .unwrap()
                .status,
            MemoryStatus::Active
        );
        assert!(service
            .get_memory(&replacement.id)
            .await
            .unwrap()
            .unwrap()
            .supersedes
            .is_empty());

        let (corrected, superseded) = service
            .correct_memory(
                &obsolete.id,
                &replacement.id,
                "The current user corrected the workflow.",
                Some("engram"),
                None,
                None,
            )
            .await
            .unwrap();
        assert!(corrected.supersedes.contains(&obsolete.id));
        assert_eq!(superseded.status, MemoryStatus::Superseded);
        assert!(!corrected
            .evidence
            .iter()
            .any(|evidence| evidence.kind == EvidenceKind::ManualReview));
        assert_eq!(
            corrected.trust_metadata().review_state,
            MemoryReviewState::ActiveUnreviewed
        );

        let (repeated, _) = service
            .correct_memory(
                &obsolete.id,
                &replacement.id,
                "Exact retry.",
                Some("engram"),
                None,
                None,
            )
            .await
            .unwrap();
        assert_eq!(repeated.evidence.len(), corrected.evidence.len());

        let error = service
            .correct_memory(
                &obsolete.id,
                &replacement.id,
                "Wrong-scope idempotent retry.",
                Some("atlas"),
                None,
                None,
            )
            .await
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("outside the resolved correction authorization boundary"));
        assert_eq!(
            service
                .get_memory(&replacement.id)
                .await
                .unwrap()
                .unwrap()
                .evidence
                .len(),
            corrected.evidence.len()
        );

        let other_obsolete = service
            .capture_memory(memory_item("Other correction target"))
            .await
            .unwrap();
        let wrong_origin = service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Decision,
                    "Agent replacement",
                    "An agent-observed item cannot claim to be a user correction.",
                    MemoryScope::project("engram"),
                    ClaimOrigin::AgentObserved,
                    writer(),
                )
                .with_evidence(EvidenceRef::new(EvidenceKind::File, "docs/agent.md")),
            )
            .await
            .unwrap();
        let error = service
            .correct_memory(
                &other_obsolete.id,
                &wrong_origin.id,
                "Wrong origin.",
                Some("engram"),
                None,
                None,
            )
            .await
            .unwrap_err();
        assert!(error.to_string().contains("origin user_corrected"));

        let wrong_scope = service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Decision,
                    "Wrong-scope correction",
                    "This correction belongs to another project.",
                    MemoryScope::project("atlas"),
                    ClaimOrigin::UserCorrected,
                    writer(),
                )
                .with_evidence(EvidenceRef::new(EvidenceKind::File, "docs/atlas.md")),
            )
            .await
            .unwrap();
        let error = service
            .correct_memory(
                &other_obsolete.id,
                &wrong_scope.id,
                "Wrong scope.",
                Some("engram"),
                None,
                None,
            )
            .await
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("outside the resolved correction authorization boundary"));

        let wrong_kind = service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Rule,
                    "Wrong-kind correction",
                    "A different memory kind cannot replace the decision.",
                    MemoryScope::project("engram"),
                    ClaimOrigin::UserCorrected,
                    writer(),
                )
                .with_evidence(EvidenceRef::new(EvidenceKind::File, "docs/rule.md")),
            )
            .await
            .unwrap();
        let error = service
            .correct_memory(
                &other_obsolete.id,
                &wrong_kind.id,
                "Wrong kind.",
                Some("engram"),
                None,
                None,
            )
            .await
            .unwrap_err();
        assert!(error.to_string().contains("identical kind and scope"));
    }

    #[tokio::test]
    async fn correct_memory_enforces_exact_supported_authorization_boundaries() {
        let service = setup_service().await;

        let (project_obsolete, project_replacement) =
            correction_pair(&service, MemoryScope::project("engram"), "project").await;
        let error = service
            .correct_memory(
                &project_obsolete.id,
                &project_replacement.id,
                "Blank project boundary.",
                Some("   "),
                None,
                None,
            )
            .await
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("outside the resolved correction authorization boundary"));
        assert_eq!(
            service
                .get_memory(&project_obsolete.id)
                .await
                .unwrap()
                .unwrap()
                .status,
            MemoryStatus::Active
        );

        let task_scope = MemoryScope::Task {
            project_id: None,
            project_name: Some("engram".to_string()),
            task_id: None,
            task_name: "ENG-123".to_string(),
        };
        let (task_obsolete, task_replacement) = correction_pair(&service, task_scope, "task").await;
        for (project, task) in [(Some("engram"), None), (Some("engram"), Some("ENG-456"))] {
            let error = service
                .correct_memory(
                    &task_obsolete.id,
                    &task_replacement.id,
                    "Wrong task boundary.",
                    project,
                    task,
                    None,
                )
                .await
                .unwrap_err();
            assert!(error
                .to_string()
                .contains("outside the resolved correction authorization boundary"));
        }
        service
            .correct_memory(
                &task_obsolete.id,
                &task_replacement.id,
                "Exact task boundary.",
                Some("engram"),
                Some("ENG-123"),
                None,
            )
            .await
            .unwrap();

        let repository_root = tempdir().unwrap();
        let repository_child = repository_root.path().join("crates/worker");
        fs::create_dir_all(&repository_child).unwrap();
        let unrelated = tempdir().unwrap();
        let repository_scope = MemoryScope::repository(
            Some("https://github.com/acme/engram".to_string()),
            Some(repository_root.path().display().to_string()),
        );
        let (repository_obsolete, repository_replacement) =
            correction_pair(&service, repository_scope, "repository").await;
        let error = service
            .correct_memory(
                &repository_obsolete.id,
                &repository_replacement.id,
                "Unrelated checkout.",
                None,
                None,
                Some(&unrelated.path().display().to_string()),
            )
            .await
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("outside the resolved correction authorization boundary"));
        service
            .correct_memory(
                &repository_obsolete.id,
                &repository_replacement.id,
                "Matching descendant checkout.",
                None,
                None,
                Some(&repository_child.display().to_string()),
            )
            .await
            .unwrap();

        for (scope, label) in [(MemoryScope::Global, "global"), (MemoryScope::User, "user")] {
            let (obsolete, replacement) = correction_pair(&service, scope, label).await;
            service
                .correct_memory(
                    &obsolete.id,
                    &replacement.id,
                    "Account-level correction.",
                    None,
                    None,
                    None,
                )
                .await
                .unwrap();
        }

        for (scope, label) in [
            (MemoryScope::entity("engram"), "entity"),
            (
                MemoryScope::Session {
                    session_id: Id::new(),
                },
                "session",
            ),
            (
                MemoryScope::Custom {
                    name: "private-scope".to_string(),
                },
                "custom",
            ),
        ] {
            let (obsolete, replacement) = correction_pair(&service, scope, label).await;
            let error = service
                .correct_memory(
                    &obsolete.id,
                    &replacement.id,
                    "Unsupported selector.",
                    Some("engram"),
                    Some("ENG-123"),
                    Some(repository_root.path().to_str().unwrap()),
                )
                .await
                .unwrap_err();
            assert!(error
                .to_string()
                .contains("scope without a trusted correction selector"));
        }
    }

    #[tokio::test]
    async fn correction_proposal_is_inactive_digest_bound_and_idempotently_applied() {
        let service = setup_service().await;
        let obsolete = service
            .capture_memory(memory_item("Proposal target"))
            .await
            .unwrap();

        let error = service
            .propose_correction(
                correction_proposal_input(obsolete.id, "decision"),
                Some("atlas"),
                None,
                None,
            )
            .await
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("outside the resolved correction authorization boundary"));

        let (proposal, replacement, unchanged_obsolete) = service
            .propose_correction(
                correction_proposal_input(obsolete.id, "decision"),
                Some("engram"),
                None,
                None,
            )
            .await
            .unwrap();
        assert_eq!(proposal.status, CorrectionProposalStatus::Pending);
        assert_eq!(proposal.obsolete_id, obsolete.id);
        assert_eq!(proposal.replacement_id, replacement.id);
        assert_eq!(proposal.canonical_digest.len(), 64);
        assert_eq!(replacement.kind, obsolete.kind);
        assert_eq!(replacement.scope, obsolete.scope);
        assert_eq!(replacement.origin, ClaimOrigin::AgentInferred);
        assert_eq!(replacement.status, MemoryStatus::NeedsReview);
        assert_eq!(unchanged_obsolete.status, MemoryStatus::Active);
        assert_eq!(
            replacement.trust_metadata().review_state,
            MemoryReviewState::NeedsReview
        );
        let active = service.list_active_memory(None).await.unwrap();
        assert!(active.iter().any(|item| item.id == obsolete.id));
        assert!(!active.iter().any(|item| item.id == replacement.id));
        let error = service
            .archive_memory(
                &replacement.id,
                "Must not archive one half of a pending pair.",
                Some("operator".to_string()),
            )
            .await
            .unwrap_err();
        assert!(error.to_string().contains("immutable while bound"));

        let error = service
            .apply_correction(&proposal.id, &"0".repeat(64), Some("engram"), None, None)
            .await
            .unwrap_err();
        assert!(error.to_string().contains("digest mismatch"));

        let tampered_replacement = replacement.clone().with_status(MemoryStatus::Active);
        let tampered_json = serde_json::to_value(&tampered_replacement).unwrap();
        let tampered_digest = sha256_hex(&serde_json::to_vec(&tampered_replacement).unwrap());
        service
            .db
            .query(
                r#"UPDATE type::thing("memory_item", $id)
                    SET item = $item, status_key = "active", snapshot_digest = $digest"#,
            )
            .bind(("id", replacement.id.to_string()))
            .bind(("item", tampered_json))
            .bind(("digest", tampered_digest))
            .await
            .unwrap()
            .check()
            .unwrap();
        let error = service
            .apply_correction(
                &proposal.id,
                &proposal.canonical_digest,
                Some("engram"),
                None,
                None,
            )
            .await
            .unwrap_err();
        assert!(error.to_string().contains("must still need review"));
        let replacement_json = serde_json::to_value(&replacement).unwrap();
        let replacement_digest = sha256_hex(&serde_json::to_vec(&replacement).unwrap());
        service
            .db
            .query(
                r#"UPDATE type::thing("memory_item", $id)
                    SET item = $item, status_key = "needs_review", snapshot_digest = $digest"#,
            )
            .bind(("id", replacement.id.to_string()))
            .bind(("item", replacement_json))
            .bind(("digest", replacement_digest))
            .await
            .unwrap()
            .check()
            .unwrap();

        let mut changed_replacement = replacement.clone();
        changed_replacement.evidence.push(EvidenceRef::new(
            EvidenceKind::File,
            "docs/unreviewed-mutation.md",
        ));
        let changed_json = serde_json::to_value(&changed_replacement).unwrap();
        let changed_digest = sha256_hex(&serde_json::to_vec(&changed_replacement).unwrap());
        service
            .db
            .query(
                r#"UPDATE type::thing("memory_item", $id)
                    SET item = $item, snapshot_digest = $digest"#,
            )
            .bind(("id", replacement.id.to_string()))
            .bind(("item", changed_json))
            .bind(("digest", changed_digest))
            .await
            .unwrap()
            .check()
            .unwrap();
        let error = service
            .apply_correction(
                &proposal.id,
                &proposal.canonical_digest,
                Some("engram"),
                None,
                None,
            )
            .await
            .unwrap_err();
        assert!(error.to_string().contains("canonical pair changed"));
        let replacement_json = serde_json::to_value(&replacement).unwrap();
        let replacement_digest = sha256_hex(&serde_json::to_vec(&replacement).unwrap());
        service
            .db
            .query(
                r#"UPDATE type::thing("memory_item", $id)
                    SET item = $item, snapshot_digest = $digest"#,
            )
            .bind(("id", replacement.id.to_string()))
            .bind(("item", replacement_json))
            .bind(("digest", replacement_digest))
            .await
            .unwrap()
            .check()
            .unwrap();

        let error = service
            .apply_correction(
                &proposal.id,
                &proposal.canonical_digest,
                Some("atlas"),
                None,
                None,
            )
            .await
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("outside the resolved correction authorization boundary"));

        let (applied, active_replacement, superseded) = service
            .apply_correction(
                &proposal.id,
                &proposal.canonical_digest,
                Some("engram"),
                None,
                None,
            )
            .await
            .unwrap();
        assert_eq!(applied.status, CorrectionProposalStatus::Applied);
        assert_eq!(active_replacement.status, MemoryStatus::Active);
        assert_eq!(superseded.status, MemoryStatus::Superseded);
        assert!(active_replacement.supersedes.contains(&obsolete.id));
        assert_eq!(
            active_replacement.trust_metadata().review_state,
            MemoryReviewState::ActiveUnreviewed
        );
        assert!(active_replacement
            .evidence
            .iter()
            .all(|evidence| evidence.kind != EvidenceKind::ManualReview));

        let evidence_count = active_replacement.evidence.len();
        let (_, retried, _) = service
            .apply_correction(
                &proposal.id,
                &proposal.canonical_digest,
                Some("engram"),
                None,
                None,
            )
            .await
            .unwrap();
        assert_eq!(retried.evidence.len(), evidence_count);
        let error = service
            .apply_correction(
                &proposal.id,
                &proposal.canonical_digest,
                Some("atlas"),
                None,
                None,
            )
            .await
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("outside the resolved correction authorization boundary"));
    }

    #[tokio::test]
    async fn correction_proposal_enforces_exact_supported_scope_selectors() {
        let service = setup_service().await;
        let task_obsolete = service
            .capture_memory(memory_item("Task proposal target").with_status(MemoryStatus::Active))
            .await
            .unwrap();
        let mut task_obsolete = task_obsolete;
        task_obsolete.scope = MemoryScope::Task {
            project_id: None,
            project_name: Some("engram".to_string()),
            task_id: None,
            task_name: "ENG-123".to_string(),
        };
        service.repo.save_memory_item(&task_obsolete).await.unwrap();
        assert!(service
            .propose_correction(
                correction_proposal_input(task_obsolete.id, "task"),
                Some("engram"),
                None,
                None,
            )
            .await
            .is_err());
        service
            .propose_correction(
                correction_proposal_input(task_obsolete.id, "task"),
                Some("engram"),
                Some("ENG-123"),
                None,
            )
            .await
            .unwrap();

        let root = tempdir().unwrap();
        let child = root.path().join("crates/worker");
        fs::create_dir_all(&child).unwrap();
        let unrelated = tempdir().unwrap();
        let repository_obsolete = service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::RepositoryFact,
                    "Repository proposal target",
                    "Repository-scoped obsolete guidance.",
                    MemoryScope::repository(None, Some(root.path().display().to_string())),
                    ClaimOrigin::UserStated,
                    writer(),
                )
                .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "unit-test")),
            )
            .await
            .unwrap();
        assert!(service
            .propose_correction(
                correction_proposal_input(repository_obsolete.id, "repository"),
                None,
                None,
                unrelated.path().to_str(),
            )
            .await
            .is_err());
        service
            .propose_correction(
                correction_proposal_input(repository_obsolete.id, "repository"),
                None,
                None,
                child.to_str(),
            )
            .await
            .unwrap();

        let global_obsolete = service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Rule,
                    "Global proposal target",
                    "Global obsolete guidance.",
                    MemoryScope::Global,
                    ClaimOrigin::UserStated,
                    writer(),
                )
                .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "unit-test")),
            )
            .await
            .unwrap();
        service
            .propose_correction(
                correction_proposal_input(global_obsolete.id, "global"),
                None,
                None,
                None,
            )
            .await
            .unwrap();

        let entity_obsolete = service
            .capture_memory(MemoryItem::new(
                MemoryKind::Decision,
                "Entity proposal target",
                "Entity obsolete guidance.",
                MemoryScope::entity("engram"),
                ClaimOrigin::UserStated,
                writer(),
            ))
            .await
            .unwrap();
        let error = service
            .propose_correction(
                correction_proposal_input(entity_obsolete.id, "entity"),
                Some("engram"),
                None,
                None,
            )
            .await
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("scope without a trusted correction selector"));
    }

    #[tokio::test]
    async fn procedure_correction_is_verified_inactive_digest_rotated_and_freshly_retrieved() {
        let service = setup_service().await;
        let dir = tempdir().unwrap();
        let obsolete = active_procedure(
            &service,
            &dir.path().join("obsolete-receipt.json"),
            "Obsolete safe check procedure",
            "run obsolete safe check",
            "cargo test -p engram-index obsolete_safe_check",
            "OBSOLETE_SAFE_CHECK_OK",
        )
        .await;
        let receipt_path = dir.path().join("replacement-receipt.json");
        let receipt = ProcedureVerificationReceipt {
            command: "cargo test -p engram-index corrected_safe_check".to_string(),
            exit_code: 0,
            output: "test result: CORRECTED_SAFE_CHECK_OK".to_string(),
            conditions: BTreeMap::new(),
        };
        let receipt_bytes = serde_json::to_vec_pretty(&receipt).unwrap();
        fs::write(&receipt_path, &receipt_bytes).unwrap();

        let error = service
            .propose_correction(
                correction_proposal_input(obsolete.id, "procedure-without-card"),
                Some("engram"),
                None,
                None,
            )
            .await
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("require structured replacement procedure"));
        let mut forged = procedure_correction_input(obsolete.id);
        forged.procedure.as_mut().unwrap().expires_at =
            Some(OffsetDateTime::now_utc() + time::Duration::days(1));
        let error = service
            .propose_correction(forged, Some("engram"), None, None)
            .await
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("cannot supply verification proof"));

        let (proposal, replacement, locked_obsolete) = service
            .propose_correction(
                procedure_correction_input(obsolete.id),
                Some("engram"),
                None,
                None,
            )
            .await
            .unwrap();
        let p0 = proposal.canonical_digest.clone();
        assert_eq!(proposal.status, CorrectionProposalStatus::Pending);
        assert_eq!(replacement.status, MemoryStatus::NeedsReview);
        assert_eq!(replacement.origin, ClaimOrigin::AgentInferred);
        assert_eq!(locked_obsolete.status, MemoryStatus::Active);
        assert!(!replacement
            .procedure
            .as_ref()
            .unwrap()
            .is_verified_at(OffsetDateTime::now_utc()));
        let active = service.list_active_memory(None).await.unwrap();
        assert!(active.iter().any(|item| item.id == obsolete.id));
        assert!(!active.iter().any(|item| item.id == replacement.id));

        let error = service
            .verify_procedure(
                &replacement.id,
                &receipt_path,
                Some(OffsetDateTime::now_utc() + time::Duration::days(30)),
            )
            .await
            .unwrap_err();
        assert!(error.to_string().contains("immutable while bound"));

        let error = service
            .apply_correction(&proposal.id, &p0, Some("engram"), None, None)
            .await
            .unwrap_err();
        assert!(error.to_string().contains("verification expiry"));
        let error = service
            .verify_correction_procedure(
                &proposal.id,
                &"0".repeat(64),
                &receipt_path,
                OffsetDateTime::now_utc() + time::Duration::days(30),
                Some("engram"),
                None,
                None,
            )
            .await
            .unwrap_err();
        assert!(error.to_string().contains("P0 digest mismatch"));
        let error = service
            .verify_correction_procedure(
                &proposal.id,
                &p0,
                &receipt_path,
                OffsetDateTime::now_utc() + time::Duration::days(30),
                Some("atlas"),
                None,
                None,
            )
            .await
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("outside the resolved correction authorization boundary"));
        let wrong_receipt = ProcedureVerificationReceipt {
            command: "cargo test -p engram-index wrong_check".to_string(),
            exit_code: 0,
            output: "CORRECTED_SAFE_CHECK_OK".to_string(),
            conditions: BTreeMap::new(),
        };
        fs::write(
            &receipt_path,
            serde_json::to_vec_pretty(&wrong_receipt).unwrap(),
        )
        .unwrap();
        let error = service
            .verify_correction_procedure(
                &proposal.id,
                &p0,
                &receipt_path,
                OffsetDateTime::now_utc() + time::Duration::days(30),
                Some("engram"),
                None,
                None,
            )
            .await
            .unwrap_err();
        assert!(error.to_string().contains("command does not match"));
        let still_p0 = service
            .inspect_correction_proposal(&proposal.id, Some("engram"), None, None)
            .await
            .unwrap();
        assert_eq!(still_p0.proposal.canonical_digest, p0);
        assert!(still_p0
            .replacement
            .procedure
            .as_ref()
            .unwrap()
            .verification
            .evidence_sha256
            .is_none());
        fs::write(&receipt_path, &receipt_bytes).unwrap();

        let proposed_evidence = replacement.evidence.clone();
        let (verified_proposal, verified_replacement, still_active_obsolete) = service
            .verify_correction_procedure(
                &proposal.id,
                &p0,
                &receipt_path,
                OffsetDateTime::now_utc() + time::Duration::days(30),
                Some("engram"),
                None,
                None,
            )
            .await
            .unwrap();
        let p1 = verified_proposal.canonical_digest.clone();
        assert_ne!(p0, p1);
        assert_eq!(verified_proposal.status, CorrectionProposalStatus::Pending);
        assert_eq!(verified_replacement.status, MemoryStatus::NeedsReview);
        assert_eq!(still_active_obsolete.status, MemoryStatus::Active);
        assert!(verified_replacement.updated_at >= replacement.updated_at);
        assert!(verified_replacement
            .evidence
            .starts_with(&proposed_evidence));
        let verification_evidence = verified_replacement
            .evidence
            .last()
            .expect("verification adds exact receipt evidence");
        assert_eq!(verification_evidence.kind, EvidenceKind::File);
        assert_eq!(
            verification_evidence.observed_at,
            verified_replacement.updated_at
        );
        assert_eq!(
            verified_replacement
                .procedure
                .as_ref()
                .unwrap()
                .verification
                .verified_at,
            Some(verification_evidence.observed_at)
        );
        assert!(verified_replacement
            .evidence
            .iter()
            .all(|evidence| evidence.observed_at <= verified_replacement.updated_at));
        assert!(verified_replacement
            .procedure
            .as_ref()
            .unwrap()
            .is_verified_at(OffsetDateTime::now_utc()));
        let active = service.list_active_memory(None).await.unwrap();
        assert!(active.iter().any(|item| item.id == obsolete.id));
        assert!(!active.iter().any(|item| item.id == replacement.id));

        let error = service
            .verify_correction_procedure(
                &proposal.id,
                &p0,
                &receipt_path,
                OffsetDateTime::now_utc() + time::Duration::days(30),
                Some("engram"),
                None,
                None,
            )
            .await
            .unwrap_err();
        assert!(error.to_string().contains("P0 digest mismatch"));
        let error = service
            .apply_correction(&proposal.id, &p0, Some("engram"), None, None)
            .await
            .unwrap_err();
        assert!(error.to_string().contains("digest mismatch"));

        fs::write(&receipt_path, b"{\"tampered\":true}").unwrap();
        let error = service
            .apply_correction(&proposal.id, &p1, Some("engram"), None, None)
            .await
            .unwrap_err();
        assert!(error.to_string().contains("receipt hash changed"));
        fs::write(&receipt_path, &receipt_bytes).unwrap();

        let (applied, active_replacement, superseded) = service
            .apply_correction(&proposal.id, &p1, Some("engram"), None, None)
            .await
            .unwrap();
        assert_eq!(applied.status, CorrectionProposalStatus::Applied);
        assert_eq!(active_replacement.status, MemoryStatus::Active);
        assert_eq!(superseded.status, MemoryStatus::Superseded);
        assert!(active_replacement.supersedes.contains(&obsolete.id));
        assert_eq!(
            active_replacement.trust_metadata().review_state,
            MemoryReviewState::ActiveUnreviewed
        );
        assert!(active_replacement
            .evidence
            .iter()
            .all(|evidence| evidence.kind != EvidenceKind::ManualReview));

        let error = service
            .apply_correction(&proposal.id, &p1, Some("engram"), None, None)
            .await
            .unwrap_err();
        assert!(error.to_string().contains("retry is not causal"));

        let matched = service
            .match_procedures(ProcedureMatchInput {
                query: "run corrected safe check".to_string(),
                project: Some("engram".to_string()),
                cwd: None,
                conditions: BTreeMap::new(),
                limit: Some(5),
            })
            .await
            .unwrap();
        assert!(!matched.abstained);
        assert_eq!(matched.procedures.len(), 1);
        assert_eq!(matched.procedures[0].id, active_replacement.id);

        service
            .archive_memory(
                &active_replacement.id,
                "Correction no longer applies.",
                Some("operator".to_string()),
            )
            .await
            .unwrap();
        let after_archive = service
            .match_procedures(ProcedureMatchInput {
                query: "run corrected safe check".to_string(),
                project: Some("engram".to_string()),
                cwd: None,
                conditions: BTreeMap::new(),
                limit: Some(5),
            })
            .await
            .unwrap();
        assert!(after_archive.abstained);

        assert!(
            service
                .forget_memory(&active_replacement.id)
                .await
                .unwrap()
                .deleted
        );
        assert!(service.forget_memory(&obsolete.id).await.unwrap().deleted);
        let restarted = MemoryService::new(service.db.clone());
        restarted.init_schema().await.unwrap();
        assert_eq!(restarted.list_memory(None, None).await.unwrap().len(), 0);
        assert!(restarted
            .repo
            .get_correction_proposal(&proposal.id)
            .await
            .unwrap()
            .is_none());
        let after_forget = restarted
            .match_procedures(ProcedureMatchInput {
                query: "run corrected safe check".to_string(),
                project: Some("engram".to_string()),
                cwd: None,
                conditions: BTreeMap::new(),
                limit: Some(5),
            })
            .await
            .unwrap();
        assert!(after_forget.abstained);
        assert!(after_forget.procedures.is_empty());
    }

    #[tokio::test]
    async fn expired_procedure_correction_proof_cannot_be_applied() {
        let service = setup_service().await;
        let dir = tempdir().unwrap();
        let obsolete = active_procedure(
            &service,
            &dir.path().join("obsolete-expiry-receipt.json"),
            "Obsolete expiring procedure",
            "run obsolete safe check",
            "cargo test -p engram-index obsolete_safe_check",
            "OBSOLETE_SAFE_CHECK_OK",
        )
        .await;
        let receipt_path = dir.path().join("replacement-expiry-receipt.json");
        let receipt = ProcedureVerificationReceipt {
            command: "cargo test -p engram-index corrected_safe_check".to_string(),
            exit_code: 0,
            output: "CORRECTED_SAFE_CHECK_OK".to_string(),
            conditions: BTreeMap::new(),
        };
        fs::write(&receipt_path, serde_json::to_vec_pretty(&receipt).unwrap()).unwrap();
        let (proposal, replacement, _) = service
            .propose_correction(
                procedure_correction_input(obsolete.id),
                Some("engram"),
                None,
                None,
            )
            .await
            .unwrap();
        let (verified, _, _) = service
            .verify_correction_procedure(
                &proposal.id,
                &proposal.canonical_digest,
                &receipt_path,
                OffsetDateTime::now_utc() + time::Duration::seconds(1),
                Some("engram"),
                None,
                None,
            )
            .await
            .unwrap();

        tokio::time::sleep(std::time::Duration::from_millis(1_100)).await;
        let error = service
            .apply_correction(
                &proposal.id,
                &verified.canonical_digest,
                Some("engram"),
                None,
                None,
            )
            .await
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("complete unexpired verification proof"));
        assert_eq!(
            service
                .get_memory(&replacement.id)
                .await
                .unwrap()
                .unwrap()
                .status,
            MemoryStatus::NeedsReview
        );
        assert_eq!(
            service
                .get_memory(&obsolete.id)
                .await
                .unwrap()
                .unwrap()
                .status,
            MemoryStatus::Active
        );
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
    async fn forget_memory_removes_item_instead_of_archiving_it() {
        let service = setup_service().await;
        let item = service
            .capture_memory(memory_item("Forget permanently"))
            .await
            .unwrap();

        assert!(service.forget_memory(&item.id).await.unwrap().deleted);
        assert!(service.get_memory(&item.id).await.unwrap().is_none());
        assert!(service.list_memory(None, None).await.unwrap().is_empty());
        assert!(!service.forget_memory(&item.id).await.unwrap().deleted);
    }

    #[tokio::test]
    async fn forget_memory_purges_internal_projections_and_references() {
        let service = setup_service().await;
        let canary = "FORGET_CANARY_7f4a2e";
        let forgotten = service.capture_memory(memory_item(canary)).await.unwrap();
        let replacement = service
            .capture_memory(memory_item("Replacement decision").with_superseded_item(forgotten.id))
            .await
            .unwrap();
        let commit = service
            .save_commit(
                KnowledgeCommit::new(writer(), format!("Commit {canary}")).with_change(
                    MemoryChange::new(
                        MemoryChangeType::Added,
                        canary,
                        format!("Persisted {canary}"),
                    )
                    .with_item(forgotten.id),
                ),
            )
            .await
            .unwrap();
        let trace = BrainHarnessTrace::new(BrainHarnessOperation::Orient)
            .with_returned_memory_ids(vec![forgotten.id])
            .with_returned_result_ids(vec![forgotten.id.to_string()]);
        service.telemetry_repo.save_trace(&trace).await.unwrap();
        let mut feedback = AgentFeedback::new(trace.id);
        feedback.used_memory_ids = vec![forgotten.id];
        feedback.note = Some(format!("Used {canary}"));
        service
            .telemetry_repo
            .save_feedback(&feedback)
            .await
            .unwrap();

        let report = service.forget_memory(&forgotten.id).await.unwrap();
        assert!(report.deleted);
        assert_eq!(report.memory_items_updated, 1);
        assert_eq!(report.commits_redacted, 1);
        assert_eq!(report.traces_deleted, 1);
        assert_eq!(report.feedback_deleted, 1);

        assert!(service.get_memory(&forgotten.id).await.unwrap().is_none());
        let replacement = service.get_memory(&replacement.id).await.unwrap().unwrap();
        assert!(!replacement.supersedes.contains(&forgotten.id));
        let commit = service.get_commit(&commit.id).await.unwrap().unwrap();
        assert!(!serde_json::to_string(&commit).unwrap().contains(canary));
        assert!(commit
            .changes
            .iter()
            .all(|change| change.item_id != Some(forgotten.id)));
        assert!(service
            .telemetry_repo
            .get_trace(&trace.id)
            .await
            .unwrap()
            .is_none());
        assert!(service
            .telemetry_repo
            .get_feedback(&feedback.id)
            .await
            .unwrap()
            .is_none());
        let graph = crate::graph::GraphService::new(service.db.clone())
            .subgraph(None, 1)
            .await
            .unwrap();
        let graph_json = serde_json::to_string(&graph).unwrap();
        assert!(!graph_json.contains(canary));
        assert!(!graph_json.contains(&forgotten.id.to_string()));
    }

    #[tokio::test]
    async fn forget_memory_refuses_to_rewrite_a_pending_correction_pair() {
        let service = setup_service().await;
        let referenced = service
            .capture_memory(memory_item("Referenced by pending proposal"))
            .await
            .unwrap();
        let obsolete = service
            .capture_memory(memory_item("Pending proposal target"))
            .await
            .unwrap();
        let mut input = correction_proposal_input(obsolete.id, "locked-reference");
        input.evidence = vec![EvidenceRef::new(
            EvidenceKind::File,
            format!("memory:{}", referenced.id),
        )];
        let (proposal, replacement, _) = service
            .propose_correction(input, Some("engram"), None, None)
            .await
            .unwrap();

        let error = service.forget_memory(&referenced.id).await.unwrap_err();
        assert!(error.to_string().contains(&proposal.id.to_string()));
        assert!(service.get_memory(&referenced.id).await.unwrap().is_some());
        assert!(service.get_memory(&replacement.id).await.unwrap().is_some());

        let cleared = service.forget_memory(&replacement.id).await.unwrap();
        assert!(cleared.deleted);
        assert_eq!(cleared.correction_proposals_deleted, 1);
        assert_eq!(cleared.proposal_obsoletes_unlocked, 1);
        let unlocked = service.get_memory(&obsolete.id).await.unwrap().unwrap();
        assert_eq!(unlocked.status, MemoryStatus::Active);
        assert!(unlocked.pending_correction_proposal_id.is_none());

        assert!(service.forget_memory(&referenced.id).await.unwrap().deleted);
    }

    #[tokio::test]
    async fn forgetting_pending_obsolete_removes_proposal_and_replacement() {
        let service = setup_service().await;
        let obsolete = service
            .capture_memory(memory_item("Forget pending obsolete"))
            .await
            .unwrap();
        let (proposal, replacement, _) = service
            .propose_correction(
                correction_proposal_input(obsolete.id, "forget-obsolete"),
                Some("engram"),
                None,
                None,
            )
            .await
            .unwrap();

        let report = service.forget_memory(&obsolete.id).await.unwrap();
        assert!(report.deleted);
        assert_eq!(report.correction_proposals_deleted, 1);
        assert_eq!(report.proposal_replacements_deleted, 1);
        assert!(service.get_memory(&obsolete.id).await.unwrap().is_none());
        assert!(service.get_memory(&replacement.id).await.unwrap().is_none());
        assert!(service
            .repo
            .get_correction_proposal(&proposal.id)
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn forget_receipt_retries_projection_cleanup_after_service_restart() {
        let service = setup_service().await;
        let forgotten = service
            .capture_memory(memory_item("Forget with retry receipt"))
            .await
            .unwrap();
        let referencing = service
            .capture_memory(
                memory_item("Projection cleanup retry").with_superseded_item(forgotten.id),
            )
            .await
            .unwrap();
        service
            .db
            .query(
                r#"UPDATE type::thing("memory_item", $id)
                    SET item.content = "PASSWORD=synthetic-retry-canary""#,
            )
            .bind(("id", referencing.id.to_string()))
            .await
            .unwrap()
            .check()
            .unwrap();

        let error = service.forget_memory(&forgotten.id).await.unwrap_err();
        assert!(error.to_string().contains("persistence policy"));
        assert!(service.get_memory(&forgotten.id).await.unwrap().is_none());
        assert!(service
            .repo
            .get_pending_memory_forget_receipt(&forgotten.id)
            .await
            .unwrap()
            .is_some());

        let repaired_json = serde_json::to_value(&referencing).unwrap();
        let repaired_digest = sha256_hex(&serde_json::to_vec(&referencing).unwrap());
        service
            .db
            .query(
                r#"UPDATE type::thing("memory_item", $id)
                    SET item = $item, snapshot_digest = $digest"#,
            )
            .bind(("id", referencing.id.to_string()))
            .bind(("item", repaired_json))
            .bind(("digest", repaired_digest))
            .await
            .unwrap()
            .check()
            .unwrap();
        let restarted = MemoryService::new(service.db.clone());
        restarted.init_schema().await.unwrap();
        let report = restarted.forget_memory(&forgotten.id).await.unwrap();
        assert!(report.deleted);
        assert!(report.cleanup_resumed);
        assert!(!report.projection_counts_complete);
        assert!(restarted
            .repo
            .get_pending_memory_forget_receipt(&forgotten.id)
            .await
            .unwrap()
            .is_none());
        let mut receipt_rows = restarted
            .db
            .query(
                r#"SELECT meta::id(id) AS id
                    FROM type::thing("memory_forget_receipt", $id)"#,
            )
            .bind(("id", forgotten.id.to_string()))
            .await
            .unwrap()
            .check()
            .unwrap();
        let receipt_rows: Vec<serde_json::Value> = receipt_rows.take(0).unwrap();
        assert!(receipt_rows.is_empty());
        let referencing = restarted
            .get_memory(&referencing.id)
            .await
            .unwrap()
            .unwrap();
        assert!(!referencing.supersedes.contains(&forgotten.id));
        assert!(
            !restarted
                .forget_memory(&forgotten.id)
                .await
                .unwrap()
                .deleted
        );
    }

    #[tokio::test]
    async fn verified_procedure_requires_matching_conditions_and_unchanged_receipt() {
        let service = setup_service().await;
        let dir = tempdir().unwrap();
        let receipt_path = dir.path().join("integration-success.json");
        let receipt = ProcedureVerificationReceipt {
            command: "cargo test -p queue-worker --test integration".to_string(),
            exit_code: 0,
            output: "integration result: PASS".to_string(),
            conditions: BTreeMap::from([("cargo.version".to_string(), "1.80.0".to_string())]),
        };
        fs::write(&receipt_path, serde_json::to_vec_pretty(&receipt).unwrap()).unwrap();

        let card = ProcedureCard::new(
            "run queue worker integration tests",
            vec!["cargo test -p queue-worker --test integration".to_string()],
            ProcedureVerification::new(
                "cargo test -p queue-worker --test integration",
                0,
                "result: PASS",
            ),
        )
        .with_prerequisite(ProcedurePrerequisite::new("cargo.version", "1.80.0"))
        .with_failure_signature("unknown option --integration");
        let candidate = service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Procedure,
                    "Queue worker integration test",
                    "Use the verified queue worker integration-test procedure.",
                    MemoryScope::project("atlas"),
                    ClaimOrigin::AgentObserved,
                    writer(),
                )
                .with_procedure(card)
                .with_status(MemoryStatus::Active),
            )
            .await
            .unwrap();
        assert_eq!(candidate.status, MemoryStatus::NeedsReview);

        let verified = service
            .verify_procedure(
                &candidate.id,
                &receipt_path,
                Some(OffsetDateTime::now_utc() + time::Duration::days(30)),
            )
            .await
            .unwrap();
        assert_eq!(verified.status, MemoryStatus::Active);
        assert!(verified
            .procedure
            .as_ref()
            .unwrap()
            .is_verified_at(OffsetDateTime::now_utc()));

        let deploy_receipt_path = dir.path().join("deploy-success.json");
        let deploy_receipt = ProcedureVerificationReceipt {
            command: "./bin/deploy-worker atlas queue-worker".to_string(),
            exit_code: 0,
            output: "ATLAS_WORKER_READY".to_string(),
            conditions: BTreeMap::new(),
        };
        fs::write(
            &deploy_receipt_path,
            serde_json::to_vec_pretty(&deploy_receipt).unwrap(),
        )
        .unwrap();
        let deploy = service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Procedure,
                    "Deploy queue worker",
                    "Use the verified queue worker deploy procedure.",
                    MemoryScope::project("atlas"),
                    ClaimOrigin::AgentObserved,
                    writer(),
                )
                .with_procedure(ProcedureCard::new(
                    "deploy queue worker",
                    vec!["./bin/deploy-worker atlas queue-worker".to_string()],
                    ProcedureVerification::new(
                        "./bin/deploy-worker atlas queue-worker",
                        0,
                        "ATLAS_WORKER_READY",
                    ),
                )),
            )
            .await
            .unwrap();
        service
            .verify_procedure(
                &deploy.id,
                &deploy_receipt_path,
                Some(OffsetDateTime::now_utc() + time::Duration::days(30)),
            )
            .await
            .unwrap();

        let matched = service
            .match_procedures(ProcedureMatchInput {
                query: "run queue worker integration tests".to_string(),
                project: Some("atlas".to_string()),
                cwd: None,
                conditions: BTreeMap::from([("cargo.version".to_string(), "1.80.0".to_string())]),
                limit: Some(5),
            })
            .await
            .unwrap();
        assert!(!matched.abstained);
        assert_eq!(matched.procedures.len(), 1);
        assert_eq!(matched.procedures[0].id, candidate.id);
        assert!(matched.diagnostics[0].applicable);

        let unscoped = service
            .match_procedures(ProcedureMatchInput {
                query: "run queue worker integration tests".to_string(),
                project: None,
                cwd: None,
                conditions: BTreeMap::from([("cargo.version".to_string(), "1.80.0".to_string())]),
                limit: Some(5),
            })
            .await
            .unwrap();
        assert!(unscoped.abstained);
        assert!(unscoped.procedures.is_empty());

        let wrong_project = service
            .match_procedures(ProcedureMatchInput {
                query: "run queue worker integration tests".to_string(),
                project: Some("other-project".to_string()),
                cwd: None,
                conditions: BTreeMap::from([("cargo.version".to_string(), "1.80.0".to_string())]),
                limit: Some(5),
            })
            .await
            .unwrap();
        assert!(wrong_project.abstained);
        assert!(wrong_project.procedures.is_empty());

        let missing_condition = service
            .match_procedures(ProcedureMatchInput {
                query: "run queue worker integration tests".to_string(),
                project: Some("atlas".to_string()),
                cwd: None,
                conditions: BTreeMap::new(),
                limit: Some(5),
            })
            .await
            .unwrap();
        assert!(missing_condition.abstained);
        assert_eq!(
            missing_condition.required_condition_keys,
            vec!["cargo.version"]
        );
        assert_eq!(
            missing_condition.diagnostics[0].unresolved_condition_keys,
            vec!["cargo.version"]
        );
        assert_eq!(missing_condition.next_actions.len(), 1);
        assert!(missing_condition.next_actions[0].contains("the current checkout"));
        assert!(missing_condition.next_actions[0].contains("separate direct tool call"));
        assert!(missing_condition.current_checkout_root.is_none());
        assert!(!missing_condition.next_actions[0].contains("1.80.0"));

        let mismatch = service
            .match_procedures(ProcedureMatchInput {
                query: "run queue worker integration tests".to_string(),
                project: Some("atlas".to_string()),
                cwd: None,
                conditions: BTreeMap::from([("cargo.version".to_string(), "1.79.0".to_string())]),
                limit: Some(5),
            })
            .await
            .unwrap();
        assert!(mismatch.abstained);
        assert!(mismatch.diagnostics[0]
            .reasons
            .iter()
            .any(|reason| reason.contains("does not match")));

        fs::write(&receipt_path, b"tampered").unwrap();
        let tampered = service
            .match_procedures(ProcedureMatchInput {
                query: "run queue worker integration tests".to_string(),
                project: Some("atlas".to_string()),
                cwd: None,
                conditions: BTreeMap::from([("cargo.version".to_string(), "1.80.0".to_string())]),
                limit: Some(5),
            })
            .await
            .unwrap();
        assert!(tampered.abstained);
        assert!(tampered.diagnostics[0]
            .reasons
            .iter()
            .any(|reason| reason.contains("hash changed")));
    }

    #[tokio::test]
    async fn source_backed_prerequisite_uses_the_current_moved_checkout() {
        let service = setup_service().await;
        let dir = tempdir().unwrap();
        let original = dir.path().join("original/atlas");
        let moved = dir.path().join("moved/atlas");
        let receipt = ProcedureVerificationReceipt {
            command: "./bin/context-probe --channel cobalt".to_string(),
            exit_code: 0,
            output: "ATLAS_CONTEXT_PROBE_OK".to_string(),
            conditions: BTreeMap::from([("tool.version".to_string(), "3".to_string())]),
        };
        let receipt_bytes = serde_json::to_vec_pretty(&receipt).unwrap();
        for (checkout, version) in [(&original, "3"), (&moved, "2")] {
            fs::create_dir_all(checkout).unwrap();
            assert!(Command::new("git")
                .args(["init", "--quiet"])
                .current_dir(checkout)
                .status()
                .unwrap()
                .success());
            assert!(Command::new("git")
                .args([
                    "remote",
                    "add",
                    "origin",
                    "git@github.com:engram-tests/atlas.git",
                ])
                .current_dir(checkout)
                .status()
                .unwrap()
                .success());
            fs::write(
                checkout.join("toolchain.toml"),
                format!("version = \"{version}\"\n"),
            )
            .unwrap();
            fs::write(checkout.join("procedure-success.json"), &receipt_bytes).unwrap();
            assert!(Command::new("git")
                .args(["add", "--", "toolchain.toml"])
                .current_dir(checkout)
                .status()
                .unwrap()
                .success());
        }

        let repositories = RepositoryService::new(service.db.clone());
        repositories.init_schema().await.unwrap();
        let original_detection = repositories.detect_repository(&original).await.unwrap();
        let candidate = service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Procedure,
                    "procedure-atlas-context-probe-v1",
                    "Run the verified Atlas context probe.",
                    MemoryScope::Repository {
                        repository_id: Some(original_detection.context.repository.id),
                        remote_url: Some("https://github.com/engram-tests/atlas".to_string()),
                        local_path: Some(original.display().to_string()),
                    },
                    ClaimOrigin::AgentObserved,
                    writer(),
                )
                .with_procedure(
                    ProcedureCard::new(
                        "run the Atlas context probe",
                        vec!["./bin/context-probe --channel cobalt".to_string()],
                        ProcedureVerification::new(
                            "./bin/context-probe --channel cobalt",
                            0,
                            "ATLAS_CONTEXT_PROBE_OK",
                        ),
                    )
                    .with_prerequisite(
                        ProcedurePrerequisite::new("tool.version", "3").with_source(
                            ProcedurePrerequisiteSource::Toml {
                                relative_path: "toolchain.toml".to_string(),
                                key_path: vec!["version".to_string()],
                            },
                        ),
                    ),
                ),
            )
            .await
            .unwrap();
        service
            .verify_procedure(
                &candidate.id,
                &original.join("procedure-success.json"),
                Some(OffsetDateTime::now_utc() + time::Duration::days(30)),
            )
            .await
            .unwrap();

        let no_candidate = service
            .match_procedures(ProcedureMatchInput {
                query: "activate ORBIT_ONLY_CANARY for the queue worker".to_string(),
                project: None,
                cwd: Some(moved.display().to_string()),
                conditions: BTreeMap::new(),
                limit: Some(5),
            })
            .await
            .unwrap();
        assert!(no_candidate.abstained);
        assert!(no_candidate.procedures.is_empty());
        assert!(no_candidate.diagnostics.is_empty());
        assert!(no_candidate.required_condition_keys.is_empty());
        assert_eq!(no_candidate.next_actions.len(), 1);
        assert!(no_candidate.next_actions[0].contains("current_checkout_root"));
        assert!(no_candidate.next_actions[0].contains("structured `identity`"));
        assert!(no_candidate.next_actions[0].len() < 700);
        assert!(no_candidate
            .message
            .contains("structured identity boundary"));
        let repository_identity = no_candidate.identity.repository.as_ref().unwrap();
        assert_eq!(repository_identity.name, "atlas");
        assert_eq!(
            repository_identity.normalized_remote.as_deref(),
            Some("github.com/engram-tests/atlas")
        );
        assert_eq!(
            repository_identity.checkout_root.as_deref(),
            Some(fs::canonicalize(&moved).unwrap().to_str().unwrap())
        );
        assert_eq!(
            no_candidate.identity.project.status,
            OrientationProjectStatus::RequiresConfirmation
        );
        assert!(no_candidate.identity.project.name.is_none());

        let mismatch = service
            .match_procedures(ProcedureMatchInput {
                query: "run the Atlas context probe".to_string(),
                project: None,
                cwd: Some(moved.display().to_string()),
                conditions: BTreeMap::from([("tool.version".to_string(), "3".to_string())]),
                limit: Some(5),
            })
            .await
            .unwrap();
        assert!(mismatch.abstained);
        assert!(mismatch.required_condition_keys.is_empty());
        assert!(mismatch.suggested_operation_evidence.is_none());
        assert_eq!(mismatch.next_actions.len(), 1);
        assert!(mismatch.next_actions[0].contains("current_checkout_root"));
        assert!(mismatch.next_actions[0].contains("`identity.project.status`"));
        assert!(mismatch.next_actions[0].contains("at most one bounded read-only lookup"));
        assert!(mismatch.next_actions[0].contains("Do not re-derive a project"));
        assert!(mismatch.next_actions[0].contains("broaden outside the checkout"));
        assert!(mismatch.next_actions[0].contains("execute candidate commands"));
        assert!(mismatch.next_actions[0].contains("`requires_confirmation`"));
        assert!(mismatch.next_actions[0].len() < 700);
        assert!(mismatch.message.contains("structured identity boundary"));
        assert_eq!(
            mismatch.diagnostics[0].condition_observations[0].status,
            ProcedureConditionObservationStatus::Mismatched
        );
        assert!(mismatch.diagnostics[0].condition_observations[0]
            .source_sha256
            .is_some());
        assert!(!serde_json::to_string(&mismatch)
            .unwrap()
            .contains("\"expected\""));

        fs::write(moved.join("toolchain.toml"), "channel = \"cobalt\"\n").unwrap();
        let unavailable = service
            .match_procedures(ProcedureMatchInput {
                query: "run the Atlas context probe".to_string(),
                project: None,
                cwd: Some(moved.display().to_string()),
                conditions: BTreeMap::from([("tool.version".to_string(), "3".to_string())]),
                limit: Some(5),
            })
            .await
            .unwrap();
        assert!(unavailable.abstained);
        assert_eq!(
            unavailable.diagnostics[0].condition_observations[0].status,
            ProcedureConditionObservationStatus::Unavailable
        );
        assert!(unavailable.diagnostics[0].condition_observations[0]
            .detail
            .contains("key path is absent"));
        assert!(unavailable.diagnostics[0].condition_observations[0]
            .source_sha256
            .is_some());

        fs::write(moved.join("toolchain.toml"), "version = \"3\"\n").unwrap();
        let matched = service
            .match_procedures(ProcedureMatchInput {
                query: "run the Atlas context probe".to_string(),
                project: None,
                cwd: Some(moved.display().to_string()),
                conditions: BTreeMap::from([("tool.version".to_string(), "2".to_string())]),
                limit: Some(5),
            })
            .await
            .unwrap();
        assert!(!matched.abstained, "{}", matched.message);
        assert_eq!(matched.procedures[0].id, candidate.id);
        assert_eq!(
            matched.diagnostics[0].condition_observations[0].status,
            ProcedureConditionObservationStatus::Matched
        );
        assert_eq!(
            matched.current_checkout_root.as_deref(),
            Some(moved.canonicalize().unwrap().to_string_lossy().as_ref())
        );
    }

    #[tokio::test]
    async fn procedure_no_result_routes_to_one_unique_tracked_runbook_without_returning_body() {
        if !git_available() {
            return;
        }
        let service = setup_service().await;
        let dir = tempdir().unwrap();
        let checkout = dir.path().join("orbit");
        let component_dir = checkout.join("services/worker");
        let runbooks_dir = checkout.join("runbooks");
        fs::create_dir_all(&component_dir).unwrap();
        fs::create_dir_all(&runbooks_dir).unwrap();
        run_git(&checkout, &["init"]);
        run_git(
            &checkout,
            &[
                "remote",
                "add",
                "origin",
                "https://github.com/engram-tests/orbit.git",
            ],
        );
        fs::write(
            component_dir.join("component.json"),
            r#"{"name":"queue-worker","kind":"service"}"#,
        )
        .unwrap();
        let runbook_body = "# Deploy the Orbit worker\n\nORBIT_ONLY_CANARY\n";
        fs::write(runbooks_dir.join("deploy-worker.md"), runbook_body).unwrap();
        fs::write(runbooks_dir.join("deploy-api.md"), "# Deploy the API\n").unwrap();
        fs::write(component_dir.join("README.md"), "# Orbit worker\n").unwrap();
        commit_all(&checkout, "add operation evidence");
        fs::write(
            runbooks_dir.join("worker-local-only.md"),
            "# Untracked local notes\n",
        )
        .unwrap();

        let repositories = RepositoryService::new(service.db.clone());
        repositories.init_schema().await.unwrap();
        repositories
            .detect_repository(&component_dir)
            .await
            .unwrap();
        let report = service
            .match_procedures(ProcedureMatchInput {
                query: "Handle the worker procedure using durable memory only if it belongs to the current repository."
                    .to_string(),
                project: None,
                cwd: Some(component_dir.display().to_string()),
                conditions: BTreeMap::new(),
                limit: Some(5),
            })
            .await
            .unwrap();

        assert!(report.abstained);
        assert!(report.procedures.is_empty());
        assert_eq!(report.identity.components[0].name, "queue-worker");
        assert_eq!(
            report.identity.project.status,
            OrientationProjectStatus::RequiresConfirmation
        );
        let candidate = report.suggested_operation_evidence.as_ref().unwrap();
        assert_eq!(candidate.path, "runbooks/deploy-worker.md");
        assert_eq!(
            candidate.resolved_path,
            checkout
                .join("runbooks/deploy-worker.md")
                .canonicalize()
                .unwrap()
                .display()
                .to_string()
        );
        assert_eq!(candidate.source_sha256, sha256_hex(runbook_body.as_bytes()));
        assert!(candidate.reason.contains("worker"));
        assert!(candidate.required_before_final_abstention);
        assert!(candidate.allowed_when_project_requires_confirmation);
        assert!(!candidate.authorizes_procedure_execution);
        assert!(report.next_actions[0].contains("suggested_operation_evidence.resolved_path"));
        assert!(report.next_actions[0].contains("Mandatory ordering"));
        assert!(report.next_actions[0].contains("durable-memory-only"));
        assert!(report.next_actions[0].contains("not procedure application"));
        assert!(report.next_actions[0].contains("provenance only"));
        assert!(report.next_actions[0].contains("runbooks/deploy-worker.md"));
        let packet = serde_json::to_vec(&report).unwrap();
        assert!(
            packet.len() <= 8_192,
            "procedure-match packet exceeded the pilot per-call budget: {} bytes",
            packet.len()
        );
        assert!(!std::str::from_utf8(&packet)
            .unwrap()
            .contains("ORBIT_ONLY_CANARY"));
    }

    #[test]
    fn operation_evidence_abstains_when_tracked_runbook_ranking_is_tied() {
        if !git_available() {
            return;
        }
        let checkout = tempdir().unwrap();
        let runbooks_dir = checkout.path().join("runbooks");
        fs::create_dir_all(&runbooks_dir).unwrap();
        run_git(checkout.path(), &["init"]);
        fs::write(runbooks_dir.join("deploy-worker.md"), "# Deploy worker\n").unwrap();
        fs::write(
            runbooks_dir.join("rollback-worker.md"),
            "# Rollback worker\n",
        )
        .unwrap();
        commit_all(checkout.path(), "add ambiguous runbooks");

        assert!(
            suggest_operation_evidence(checkout.path(), "Handle the worker procedure.", &[],)
                .is_none()
        );
    }

    #[test]
    fn tracked_condition_source_rejects_unsafe_files() {
        let root = tempdir().unwrap();
        assert!(Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(root.path())
            .status()
            .unwrap()
            .success());
        fs::write(root.path().join("tracked.toml"), "version = \"3\"\n").unwrap();
        fs::write(root.path().join("untracked.toml"), "version = \"3\"\n").unwrap();
        fs::write(
            root.path().join("oversized.toml"),
            vec![b'x'; MAX_CONDITION_SOURCE_BYTES as usize + 1],
        )
        .unwrap();
        assert!(Command::new("git")
            .args(["add", "--", "tracked.toml", "oversized.toml"])
            .current_dir(root.path())
            .status()
            .unwrap()
            .success());

        assert!(read_tracked_condition_source(root.path(), "tracked.toml").is_ok());
        assert!(
            read_tracked_condition_source(root.path(), "../tracked.toml")
                .unwrap_err()
                .contains("safe checkout-relative")
        );
        assert!(read_tracked_condition_source(root.path(), "untracked.toml")
            .unwrap_err()
            .contains("not Git-tracked"));
        assert!(read_tracked_condition_source(root.path(), "oversized.toml")
            .unwrap_err()
            .contains("exceeds"));

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink("tracked.toml", root.path().join("linked.toml")).unwrap();
            assert!(read_tracked_condition_source(root.path(), "linked.toml")
                .unwrap_err()
                .contains("symlink"));
        }
    }

    #[tokio::test]
    async fn repository_procedure_matches_moved_checkout_by_stable_remote_identity() {
        let service = setup_service().await;
        let dir = tempdir().unwrap();
        let original = dir.path().join("original/engram-procedure-test");
        let moved = dir.path().join("moved/engram-procedure-test");
        for checkout in [&original, &moved] {
            fs::create_dir_all(checkout).unwrap();
            assert!(Command::new("git")
                .args(["init", "--quiet"])
                .current_dir(checkout)
                .status()
                .unwrap()
                .success());
            assert!(Command::new("git")
                .args([
                    "remote",
                    "add",
                    "origin",
                    "git@github.com:engram-tests/portable-procedure.git",
                ])
                .current_dir(checkout)
                .status()
                .unwrap()
                .success());
        }

        let repositories = RepositoryService::new(service.db.clone());
        repositories.init_schema().await.unwrap();
        let original_detection = repositories.detect_repository(&original).await.unwrap();
        let repository_repo = RepositoryRepo::new(service.db.clone());
        repository_repo.init_schema().await.unwrap();
        let receipt_path = original.join("procedure-success.json");
        let receipt = ProcedureVerificationReceipt {
            command: "cargo test -p portable-tests".to_string(),
            exit_code: 0,
            output: "portable result: PASS".to_string(),
            conditions: BTreeMap::new(),
        };
        let receipt_bytes = serde_json::to_vec_pretty(&receipt).unwrap();
        fs::write(&receipt_path, &receipt_bytes).unwrap();
        fs::write(moved.join("procedure-success.json"), &receipt_bytes).unwrap();

        let candidate = service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Procedure,
                    "Portable repository test procedure",
                    "Run the portable repository test command.",
                    MemoryScope::Repository {
                        repository_id: Some(original_detection.context.repository.id),
                        remote_url: Some(
                            "https://github.com/engram-tests/portable-procedure".to_string(),
                        ),
                        local_path: Some(original.display().to_string()),
                    },
                    ClaimOrigin::AgentObserved,
                    writer(),
                )
                .with_procedure(ProcedureCard::new(
                    "run portable repository tests",
                    vec!["cargo test -p portable-tests".to_string()],
                    ProcedureVerification::new("cargo test -p portable-tests", 0, "result: PASS"),
                )),
            )
            .await
            .unwrap();
        let verified = service
            .verify_procedure(
                &candidate.id,
                &receipt_path,
                Some(OffsetDateTime::now_utc() + time::Duration::days(30)),
            )
            .await
            .unwrap();
        assert_eq!(
            verified
                .procedure
                .as_ref()
                .unwrap()
                .verification
                .evidence_path
                .as_deref(),
            Some("procedure-success.json")
        );
        assert_eq!(
            verified.evidence.last().unwrap().target,
            "procedure-success.json"
        );

        fs::write(&receipt_path, b"tampered in the original checkout").unwrap();

        let matched = service
            .match_procedures(ProcedureMatchInput {
                query: "run portable repository tests".to_string(),
                project: None,
                cwd: Some(moved.display().to_string()),
                conditions: BTreeMap::new(),
                limit: Some(5),
            })
            .await
            .unwrap();

        assert!(!matched.abstained, "{}", matched.message);
        assert_eq!(matched.procedures[0].id, candidate.id);
        assert_eq!(
            matched.current_checkout_root.as_deref(),
            Some(moved.canonicalize().unwrap().to_string_lossy().as_ref())
        );
        assert!(matched
            .execution_guidance
            .as_deref()
            .is_some_and(|guidance| guidance.contains("provenance only")));
        assert_ne!(
            matched.current_checkout_root.as_deref(),
            Some(original.canonicalize().unwrap().to_string_lossy().as_ref())
        );
        let moved_context = repositories.resolve_cwd(&moved).await.unwrap().unwrap();
        assert_eq!(
            moved_context.repository.id,
            original_detection.context.repository.id
        );
        assert!(moved_context.linked_projects.is_empty());

        repository_repo
            .save_project_link(&ProjectRepositoryLink::new(
                "portable-project",
                original_detection.context.repository.id,
                ProjectRepositoryRole::Primary,
            ))
            .await
            .unwrap();

        let conflicting_project = service
            .match_procedures(ProcedureMatchInput {
                query: "run portable repository tests".to_string(),
                project: Some("other-project".to_string()),
                cwd: Some(moved.display().to_string()),
                conditions: BTreeMap::new(),
                limit: Some(5),
            })
            .await
            .unwrap();
        assert!(conflicting_project.abstained);
        assert!(conflicting_project.procedures.is_empty());
        assert!(conflicting_project.message.contains("ambiguous"));
    }

    #[tokio::test]
    async fn capture_rejects_high_confidence_secret_material() {
        let service = setup_service().await;
        let item = MemoryItem::new(
            MemoryKind::ProjectFact,
            "Leaked credential",
            "Use token ghp_1234567890abcdefghijklmnop for the API.",
            MemoryScope::project("engram"),
            ClaimOrigin::AgentObserved,
            writer(),
        );

        let error = service.capture_memory(item).await.unwrap_err();
        assert!(error.to_string().contains("likely GitHub token"));
        assert!(service.list_memory(None, None).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn procedure_match_rejects_an_unbounded_query_before_retrieval() {
        let service = setup_service().await;
        let error = service
            .match_procedures(ProcedureMatchInput {
                query: "x".repeat(MAX_PROCEDURE_QUERY_CHARS + 1),
                project: None,
                cwd: None,
                conditions: BTreeMap::new(),
                limit: None,
            })
            .await
            .unwrap_err();

        assert!(error.to_string().contains("task-focused retrieval phrase"));
        assert!(error
            .to_string()
            .contains(&MAX_PROCEDURE_QUERY_CHARS.to_string()));
    }

    #[tokio::test]
    async fn migration_review_assertion_stays_unverified_in_orient_and_search() {
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
        assert_eq!(metadata.review_state, MemoryReviewState::ActiveUnreviewed);
        assert!(metadata.review_asserted);
        assert!(!metadata.reviewed);
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
                && metadata.review_state == MemoryReviewState::ActiveUnreviewed
                && metadata.review_asserted
                && !metadata.reviewed));
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
            Some(MemoryReviewState::ActiveUnreviewed)
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
        assert_eq!(
            packet.used_memory_candidate_ids,
            packet
                .brain_loop
                .top_items
                .iter()
                .map(|item| item.id)
                .collect::<Vec<_>>()
        );
        assert!(packet.context_pack.contains("used_memory_candidate_ids"));
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
    async fn orient_prepare_handoff_prioritizes_latest_current_plan_and_keeps_gates() {
        let service = setup_service().await;
        let mut stale_repository_plan = MemoryItem::new(
            MemoryKind::Decision,
            "Current plan after Codex document lifecycle follow-through",
            "Older repository-scoped current-plan guidance that should not lead a compact \
             Brain Harness handoff.",
            MemoryScope::repository(None, Some("/Users/yuval.meiri/projects/engram".to_string())),
            ClaimOrigin::ToolResult,
            writer(),
        )
        .with_status(MemoryStatus::Active)
        .with_evidence(EvidenceRef::new(
            EvidenceKind::ToolCall,
            "stale-repository-current-plan",
        ))
        .with_tag(CURRENT_PLAN_TAG);
        stale_repository_plan.updated_at = OffsetDateTime::now_utc() - time::Duration::days(2);
        let stale_repository_plan = service.capture_memory(stale_repository_plan).await.unwrap();

        let mut latest_plan = MemoryItem::new(
            MemoryKind::Decision,
            "Current plan: fix prepare_handoff orientation",
            "Latest current plan: add a narrow prepare_handoff orientation fixture before any \
             migration, lifecycle, hook, schema, public MCP, broad ranking, or payload change.",
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
        latest_plan.updated_at = OffsetDateTime::now_utc();
        let latest_plan = service.capture_memory(latest_plan).await.unwrap();

        service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Decision,
                    "Mission-class PlanWork current-plan gap resolved narrowly",
                    "Earlier mission-class plan_work prompts now preserve current-plan continuity, \
                     but this implementation-history item is not the current handoff plan.",
                    MemoryScope::project("engram"),
                    ClaimOrigin::ToolResult,
                    writer(),
                )
                .with_evidence(EvidenceRef::new(EvidenceKind::ToolCall, "secondary-decision")),
            )
            .await
            .unwrap();
        service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Rule,
                    "Brain Harness work follows research method",
                    "Brain Harness work uses explicit research questions, competing hypotheses, \
                     evidence levels, falsifiers, decision gates, and claim-ledger updates.",
                    MemoryScope::project("engram"),
                    ClaimOrigin::UserStated,
                    writer(),
                )
                .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "unit-test")),
            )
            .await
            .unwrap();
        service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Preference,
                    "Software design philosophy: deep modules and evidence over confidence",
                    "Prefer Ousterhout-style deep modules, low cognitive load, no unrequested \
                     features, and evidence over confidence.",
                    MemoryScope::project("engram"),
                    ClaimOrigin::UserStated,
                    writer(),
                )
                .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "unit-test")),
            )
            .await
            .unwrap();
        service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Limitation,
                    "Non-gated calibration does not prove broad ranking quality",
                    "The non-gated continuation calibration fixes one prompt class but should not \
                     be treated as broad ranking proof.",
                    MemoryScope::project("engram"),
                    ClaimOrigin::ToolResult,
                    writer(),
                )
                .with_evidence(EvidenceRef::new(
                    EvidenceKind::ToolCall,
                    "calibration-noise",
                )),
            )
            .await
            .unwrap();

        let m6_gate = service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Limitation,
                    "M6 migration approval gate remains explicit",
                    "Brain Harness handoff approval gates must say that M6 migration read-only \
                     inventory or review export needs explicit user-approved scope, and write \
                     apply, deletion, cleanup, or legacy simplification need reviewed candidates, \
                     dry-run evidence, rollback planning, and explicit approval.",
                    MemoryScope::project("engram"),
                    ClaimOrigin::UserStated,
                    writer(),
                )
                .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "unit-test")),
            )
            .await
            .unwrap();
        let harness_gate = service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Rule,
                    "Harness adapter and hook write approval gate",
                    "Brain Harness handoffs must preserve the harness-write gate: do not install \
                     or modify Claude Code, Codex, Gemini CLI, or Cursor adapters, settings, or \
                     hooks without explicit user approval.",
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
                cwd: Some("/Users/yuval.meiri/projects/engram".to_string()),
                prompt: Some(
                    "Prepare a compact Brain Harness handoff: current plan, approval gates, \
                     evidence-quality state, and next non-gated work."
                        .to_string(),
                ),
                intent: Some(BrainHarnessIntent::PrepareHandoff),
                limit: Some(10),
                ..OrientInput::default()
            })
            .await
            .unwrap();

        assert_eq!(
            packet.active_decisions.first().map(|item| item.id),
            Some(latest_plan.id)
        );
        assert!(
            !packet
                .active_decisions
                .iter()
                .any(|item| item.id == stale_repository_plan.id),
            "handoff should not present stale repository-scoped current-plan guidance as current"
        );
        assert_eq!(
            packet.brain_loop.top_items.first().map(|item| item.id),
            Some(latest_plan.id)
        );
        let top_titles = packet
            .brain_loop
            .top_items
            .iter()
            .map(|item| item.title.as_str())
            .collect::<Vec<_>>();
        assert!(
            packet
                .brain_loop
                .top_items
                .iter()
                .any(|item| item.id == m6_gate.id),
            "expected M6 gate in {top_titles:?}"
        );
        assert!(
            packet
                .brain_loop
                .top_items
                .iter()
                .any(|item| item.id == harness_gate.id),
            "expected harness gate in {top_titles:?}"
        );
        assert_eq!(
            service
                .get_memory(&stale_repository_plan.id)
                .await
                .unwrap()
                .unwrap()
                .status,
            MemoryStatus::Active,
            "handoff orientation should not mutate stale memory lifecycle"
        );
    }

    #[tokio::test]
    async fn orient_mission_prompt_diagnostic_distinguishes_intent_from_ranking() {
        let service = setup_service().await;
        let current_plan = service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Decision,
                    "Current plan after lean orient real verification smoke",
                    "Lean orient passed a real read-only verification smoke. Next step: add a \
                     deterministic diagnostic fixture before changing ranking, migration, hooks, \
                     or the orient payload.",
                    MemoryScope::project("engram"),
                    ClaimOrigin::ToolResult,
                    writer(),
                )
                .with_status(MemoryStatus::Active)
                .with_confidence(0.95)
                .with_evidence(EvidenceRef::new(
                    EvidenceKind::ToolCall,
                    "latest-current-plan",
                ))
                .with_tag(CURRENT_PLAN_TAG),
            )
            .await
            .unwrap();

        service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Rule,
                    "Brain Harness work follows research method",
                    "Engram Brain Harness work toward a production-quality Brain OS must use \
                     explicit research questions, competing hypotheses, evidence gates, and \
                     claim-ledger updates.",
                    MemoryScope::project("engram"),
                    ClaimOrigin::UserStated,
                    writer(),
                )
                .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "unit-test")),
            )
            .await
            .unwrap();
        service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Preference,
                    "Commit every meaningful Engram step",
                    "When developing Engram, commit each meaningful validated step and keep \
                     unrelated user-owned files out of the commit.",
                    MemoryScope::project("engram"),
                    ClaimOrigin::UserStated,
                    writer(),
                )
                .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "unit-test")),
            )
            .await
            .unwrap();
        service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Limitation,
                    "Mission-class Brain OS prompt can miss latest current-plan memory",
                    "A broad prompt to complete Engram into a production-quality Brain OS can \
                     surface older Brain Harness guidance before the latest current-plan item.",
                    MemoryScope::project("engram"),
                    ClaimOrigin::ToolResult,
                    writer(),
                )
                .with_evidence(EvidenceRef::new(EvidenceKind::ToolCall, "mission-gap")),
            )
            .await
            .unwrap();
        service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Decision,
                    "Current plan after Codex document lifecycle follow-through",
                    "The next product-facing Brain Harness slice completed document lifecycle \
                     follow-through for Codex while working toward Engram as a Brain OS.",
                    MemoryScope::project("engram"),
                    ClaimOrigin::ToolResult,
                    writer(),
                )
                .with_evidence(EvidenceRef::new(
                    EvidenceKind::GitCommit,
                    "document-lifecycle-follow-through",
                )),
            )
            .await
            .unwrap();
        service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Decision,
                    "Review mission prompt policy before changing ranking",
                    "Inferred mission-class Brain OS planning policy should stay review-needed \
                     until a diagnostic fixture distinguishes caller intent from ranker behavior.",
                    MemoryScope::project("engram"),
                    ClaimOrigin::AgentInferred,
                    writer(),
                )
                .with_evidence(EvidenceRef::new(EvidenceKind::ToolCall, "mission-gap")),
            )
            .await
            .unwrap();

        let explicit_next_step = service
            .orient(OrientInput {
                project: Some("engram".to_string()),
                prompt: Some(
                    "What is the current plan / next step for Engram? Continue from where we \
                     left off."
                        .to_string(),
                ),
                intent: Some(BrainHarnessIntent::PlanWork),
                limit: Some(10),
                ..OrientInput::default()
            })
            .await
            .unwrap();
        assert_eq!(
            explicit_next_step
                .active_decisions
                .first()
                .map(|item| item.id),
            Some(current_plan.id)
        );
        assert_eq!(
            explicit_next_step
                .brain_loop
                .top_items
                .first()
                .map(|item| item.id),
            Some(current_plan.id)
        );

        let mission_prompt =
            "Complete Engram into a production-quality Brain OS / brain harness for AI agents.";
        let plan_work_mission = service
            .orient(OrientInput {
                project: Some("engram".to_string()),
                prompt: Some(mission_prompt.to_string()),
                intent: Some(BrainHarnessIntent::PlanWork),
                limit: Some(10),
                ..OrientInput::default()
            })
            .await
            .unwrap();
        assert_eq!(plan_work_mission.intent, Some(BrainHarnessIntent::PlanWork));
        assert_eq!(
            plan_work_mission
                .active_decisions
                .first()
                .map(|item| item.id),
            Some(current_plan.id),
            "mission-class PlanWork should promote the latest current plan within decisions"
        );
        assert!(
            plan_work_mission
                .used_memory_candidate_ids
                .contains(&current_plan.id),
            "mission-class PlanWork should include the latest current plan in used candidates"
        );
        assert_ne!(
            plan_work_mission
                .brain_loop
                .top_items
                .first()
                .map(|item| item.id),
            Some(current_plan.id),
            "PlanWork should not use the ResumeSession brain-loop pin"
        );

        let resume_mission = service
            .orient(OrientInput {
                project: Some("engram".to_string()),
                prompt: Some(mission_prompt.to_string()),
                intent: Some(BrainHarnessIntent::ResumeSession),
                limit: Some(10),
                ..OrientInput::default()
            })
            .await
            .unwrap();
        assert_eq!(
            resume_mission.active_decisions.first().map(|item| item.id),
            Some(current_plan.id)
        );
        assert_eq!(
            resume_mission
                .brain_loop
                .top_items
                .first()
                .map(|item| item.id),
            Some(current_plan.id)
        );
    }

    #[test]
    fn open_ended_plan_work_prompt_detection_stays_narrow() {
        assert!(is_open_ended_plan_work_prompt(
            "Complete Engram into a production-quality Brain OS / brain harness for AI agents."
        ));
        assert!(is_open_ended_plan_work_prompt(
            "Continue from where we left off and move forward."
        ));
        assert!(!is_open_ended_plan_work_prompt("plan schema migration"));
        assert!(!is_open_ended_plan_work_prompt(
            "implement request throttling"
        ));
    }

    #[tokio::test]
    async fn orient_recent_knowledge_commits_respect_explicit_project_scope() {
        let service = setup_service().await;

        let mut engram_plan = current_plan_input(
            "engram",
            "Engram scope-noise plan",
            "Investigate Engram orientation scope noise before larger ranking claims.",
        );
        engram_plan.create_commit = true;
        let engram_capture = service.capture_current_plan(engram_plan).await.unwrap();
        let engram_commit = engram_capture
            .commit
            .expect("Engram current plan should create a commit");

        let mut voice_layer_plan = current_plan_input(
            "voice-layer",
            "Voice Layer calibration plan",
            "Run a fresh Claude calibration for the voice-layer project.",
        );
        voice_layer_plan.create_commit = true;
        let voice_layer_capture = service
            .capture_current_plan(voice_layer_plan)
            .await
            .unwrap();
        let voice_layer_commit = voice_layer_capture
            .commit
            .expect("voice-layer current plan should create a commit");

        let packet = service
            .orient(OrientInput {
                project: Some("engram".to_string()),
                prompt: Some("How should we proceed with Engram after BAF006?".to_string()),
                include_recent_commits: true,
                limit: Some(10),
                ..OrientInput::default()
            })
            .await
            .unwrap();

        let commit_ids = packet
            .recent_knowledge_commits
            .iter()
            .map(|commit| commit.id)
            .collect::<Vec<_>>();
        assert!(commit_ids.contains(&engram_commit.id));
        assert!(!commit_ids.contains(&voice_layer_commit.id));
        assert!(!packet
            .context_pack
            .contains("Capture current plan: Voice Layer calibration plan"));
        assert!(!packet
            .active_decisions
            .iter()
            .any(|item| item.id == voice_layer_capture.item.id));
    }

    #[tokio::test]
    async fn orient_does_not_hot_boost_unverified_preference_review_assertion() {
        let service = setup_service().await;
        let preference = service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Preference,
                    "Commit every meaningful Engram step",
                    "When developing Engram, create a focused git commit after each meaningful \
                     implementation, validation, or documentation step. Keep unrelated \
                     user-owned files, such as AGENTS.md, out of those commits unless the user \
                     explicitly asks to include them.",
                    MemoryScope::project("engram"),
                    ClaimOrigin::UserStated,
                    writer(),
                )
                .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "unit-test")),
            )
            .await
            .unwrap();
        service
            .capture_memory(MemoryItem::new(
                MemoryKind::Decision,
                "Cursor workflow preference uses generated harness files",
                "Cursor harness workflow decisions can mention user workflow, generated files, \
                 settings, commits, and what to keep out of local configuration.",
                MemoryScope::project("engram"),
                ClaimOrigin::AgentObserved,
                writer(),
            ))
            .await
            .unwrap();

        let packet = service
            .orient(OrientInput {
                project: Some("engram".to_string()),
                prompt: Some(
                    "We are changing Engram code and documentation. What user workflow \
                     preference should I follow after each meaningful step, and what should I \
                     keep out of the commit?"
                        .to_string(),
                ),
                intent: Some(BrainHarnessIntent::FollowUserPreference),
                limit: Some(10),
                ..OrientInput::default()
            })
            .await
            .unwrap();

        assert_eq!(
            packet.brain_loop.top_items.first().map(|item| item.id),
            Some(preference.id)
        );
        assert!(packet.hot_context_ids.is_empty());
        assert!(packet.hot_context_items.is_empty());
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
        assert!(packet.identity.repository.is_none());
        assert_eq!(
            packet.identity.project.status,
            OrientationProjectStatus::Unavailable
        );
        assert!(packet.identity.project.name.is_none());
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
                prompt: Some(
                    "Handle the context probe using durable procedure memory only when it is \
                     applicable to the current checkout."
                        .to_string(),
                ),
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
        assert_eq!(
            packet.identity.project.status,
            OrientationProjectStatus::Authorized
        );
        assert_eq!(
            packet.identity.project.name.as_deref(),
            Some("Debug with AI")
        );
        assert_eq!(packet.identity.project.project_link_ids.len(), 1);
        assert_eq!(
            packet
                .identity
                .repository
                .as_ref()
                .and_then(|repository| repository.checkout_root.as_deref()),
            Some(dir.path().to_str().unwrap())
        );
        assert_eq!(packet.active_decisions[0].title, "Use repo candidate");
    }

    #[tokio::test]
    async fn orient_matches_repository_memory_by_resolved_repository_id() {
        let (service, repo) = setup_service_with_repository_repo().await;
        let dir = tempdir().unwrap();
        let repository =
            GitRepository::new("atlas").with_remote_url("git@github.com:acme/atlas.git");
        repo.save_repository(&repository).await.unwrap();
        repo.save_checkout(
            &LocalCheckout::new(dir.path().display().to_string()).with_repository(repository.id),
        )
        .await
        .unwrap();
        repo.save_project_link(&ProjectRepositoryLink::new(
            "atlas",
            repository.id,
            ProjectRepositoryRole::Primary,
        ))
        .await
        .unwrap();

        let decision = MemoryItem::new(
            MemoryKind::Decision,
            "Use reqwest for Atlas HTTP",
            "New HTTP client code uses reqwest with shared timeout middleware.",
            MemoryScope::Repository {
                repository_id: Some(repository.id),
                remote_url: Some("https://github.com/acme/atlas".to_string()),
                local_path: None,
            },
            ClaimOrigin::UserStated,
            writer(),
        )
        .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "unit-test"));
        let decision = service.capture_memory(decision).await.unwrap();

        let packet = service
            .orient(OrientInput {
                cwd: Some(dir.path().display().to_string()),
                prompt: Some("Which HTTP client should new code use?".to_string()),
                intent: Some(BrainHarnessIntent::AnswerQuestion),
                include_recent_commits: false,
                ..OrientInput::default()
            })
            .await
            .unwrap();

        assert_eq!(
            packet.active_decisions.first().map(|item| item.id),
            Some(decision.id)
        );
    }

    #[tokio::test]
    async fn orient_prompt_intent_excludes_zero_text_scope_matches() {
        let service = setup_service().await;
        service
            .capture_memory(memory_item("Queue backend decision"))
            .await
            .unwrap();

        let packet = service
            .orient(OrientInput {
                project: Some("engram".to_string()),
                prompt: Some(
                    "What is the approved approach for quantum-resistant image thumbnails?"
                        .to_string(),
                ),
                intent: Some(BrainHarnessIntent::AnswerQuestion),
                include_recent_commits: false,
                ..OrientInput::default()
            })
            .await
            .unwrap();

        assert!(packet.active_decisions.is_empty());
        assert!(packet.brain_loop.top_items.is_empty());
        assert!(packet.used_memory_candidate_ids.is_empty());
    }

    #[tokio::test]
    async fn orient_resume_session_surfaces_active_handoff_first() {
        let service = setup_service().await;
        service
            .capture_memory(memory_item("Unrelated architecture decision"))
            .await
            .unwrap();
        let handoff = service
            .capture_memory(MemoryItem::new(
                MemoryKind::Handoff,
                "Queue worker handoff",
                "Next action: update the JetStream consumer and run the integration test.",
                MemoryScope::project("engram"),
                ClaimOrigin::AgentObserved,
                writer(),
            ))
            .await
            .unwrap();

        let packet = service
            .orient(OrientInput {
                project: Some("engram".to_string()),
                prompt: Some("Continue from where we left off.".to_string()),
                intent: Some(BrainHarnessIntent::ResumeSession),
                include_recent_commits: false,
                ..OrientInput::default()
            })
            .await
            .unwrap();

        assert_eq!(
            packet.handoffs.first().map(|item| item.id),
            Some(handoff.id)
        );
        assert_eq!(
            packet.brain_loop.top_items.first().map(|item| item.id),
            Some(handoff.id)
        );
        assert!(packet.used_memory_candidate_ids.contains(&handoff.id));
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
    async fn orient_ambiguous_projects_blocks_project_memory_but_allows_local_procedure_route() {
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
        assert!(packet.recommended_actions.iter().any(|item| {
            item.contains("Before executing or exploring any actionable repository task")
                && item.contains("bounded task-focused query")
                && item.contains("Preserve concrete operation terms and identifiers verbatim")
                && item.contains("durable procedure memory")
                && item.contains("memory(action=list) is not a substitute")
        }));
        assert!(packet.recommended_actions.iter().any(|item| {
            item.contains("Project ambiguity blocks only project/task-scoped memory")
                && item.contains("mandatory repository-local procedure_match")
        }));
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
    async fn orient_returns_tracked_component_identity_with_source_evidence() {
        if !git_available() {
            return;
        }
        let service = setup_service().await;
        let dir = tempdir().unwrap();
        let component_dir = dir.path().join("services/worker");
        std::fs::create_dir_all(&component_dir).unwrap();
        run_git(dir.path(), &["init"]);
        run_git(
            dir.path(),
            &[
                "remote",
                "add",
                "origin",
                "https://github.com/acme/atlas.git",
            ],
        );
        std::fs::write(
            component_dir.join("component.json"),
            r#"{"name":"queue-worker","kind":"service"}"#,
        )
        .unwrap();
        commit_all(dir.path(), "add component identity");

        let packet = service
            .orient(OrientInput {
                cwd: Some(component_dir.display().to_string()),
                project: None,
                include_recent_commits: false,
                limit: Some(10),
                ..OrientInput::default()
            })
            .await
            .unwrap();

        assert!(packet.resolution.selected_project.is_none());
        assert!(packet.resolution.requires_confirmation);
        assert_eq!(
            packet.identity.project.status,
            OrientationProjectStatus::RequiresConfirmation
        );
        assert!(packet.identity.project.name.is_none());
        let repository = packet.identity.repository.as_ref().unwrap();
        assert_eq!(repository.name, "atlas");
        assert_eq!(
            repository.normalized_remote.as_deref(),
            Some("github.com/acme/atlas")
        );
        assert_eq!(
            repository.checkout_root.as_deref(),
            Some(fs::canonicalize(dir.path()).unwrap().to_str().unwrap())
        );
        assert_eq!(
            packet.resolution.repository_remote.as_deref(),
            Some("github.com/acme/atlas")
        );
        assert_eq!(packet.resolution.component_names, vec!["queue-worker"]);
        assert_eq!(packet.resolution.component_evidence.len(), 1);
        let evidence = &packet.resolution.component_evidence[0];
        assert_eq!(evidence.name, "queue-worker");
        assert_eq!(evidence.component_path, "services/worker");
        assert_eq!(
            evidence.source_path.as_deref(),
            Some("services/worker/component.json")
        );
        assert_eq!(evidence.source_sha256.as_deref().map(str::len), Some(64));
        assert_eq!(packet.identity.components.len(), 1);
        assert_eq!(
            packet.identity.components[0].source_path,
            evidence.source_path
        );
        assert_eq!(
            packet.identity.components[0].source_sha256,
            evidence.source_sha256
        );
        assert!(packet
            .ambiguities
            .iter()
            .any(|item| item.contains("no linked project candidates")));
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
