//! Memory OS ontology types.
//!
//! These types describe source-grounded memory items before they are stored in a
//! graph, written to a Markdown vault, or served through MCP. The core crate owns
//! this vocabulary so different harnesses can write comparable records.

use crate::id::Id;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// The kind of knowledge captured in a memory item.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryKind {
    /// A user or project preference.
    Preference,
    /// A rule the agent should follow.
    Rule,
    /// A decision that affects future work.
    Decision,
    /// A known limitation or constraint.
    Limitation,
    /// A fact about a project.
    ProjectFact,
    /// A fact about a repository or checkout.
    RepositoryFact,
    /// A fact about a task or feature.
    TaskFact,
    /// A fact about the user.
    UserFact,
    /// A distilled insight from a session.
    SessionInsight,
    /// A handoff or progress marker.
    Handoff,
    /// A condition-aware, evidence-verified engineering procedure or gotcha.
    Procedure,
    /// Custom kind.
    Custom(String),
}

impl std::fmt::Display for MemoryKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Preference => write!(f, "preference"),
            Self::Rule => write!(f, "rule"),
            Self::Decision => write!(f, "decision"),
            Self::Limitation => write!(f, "limitation"),
            Self::ProjectFact => write!(f, "project_fact"),
            Self::RepositoryFact => write!(f, "repository_fact"),
            Self::TaskFact => write!(f, "task_fact"),
            Self::UserFact => write!(f, "user_fact"),
            Self::SessionInsight => write!(f, "session_insight"),
            Self::Handoff => write!(f, "handoff"),
            Self::Procedure => write!(f, "procedure"),
            Self::Custom(value) => write!(f, "{}", value),
        }
    }
}

impl MemoryKind {
    /// Parse from string.
    #[must_use]
    pub fn parse(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "preference" => Self::Preference,
            "rule" => Self::Rule,
            "decision" => Self::Decision,
            "limitation" => Self::Limitation,
            "project_fact" | "projectfact" => Self::ProjectFact,
            "repository_fact" | "repo_fact" | "repositoryfact" | "repofact" => Self::RepositoryFact,
            "task_fact" | "taskfact" => Self::TaskFact,
            "user_fact" | "userfact" => Self::UserFact,
            "session_insight" | "sessioninsight" => Self::SessionInsight,
            "handoff" => Self::Handoff,
            "procedure" | "gotcha" | "runbook" => Self::Procedure,
            other => Self::Custom(other.to_string()),
        }
    }
}

/// A deterministic checkout-local source for a procedure prerequisite.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "format", rename_all = "snake_case")]
pub enum ProcedurePrerequisiteSource {
    /// A scalar selected from a Git-tracked TOML file in the current checkout.
    Toml {
        /// Safe checkout-relative path to the TOML file.
        relative_path: String,
        /// Non-empty sequence of TOML table/key names leading to one scalar value.
        key_path: Vec<String>,
    },
}

/// An exact environmental prerequisite for a procedure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcedurePrerequisite {
    /// Stable condition key, such as `cargo.version` or `os`.
    pub key: String,
    /// Exact expected value after Unicode/case normalization by the caller.
    pub expected: String,
    /// Optional trusted source that Engram resolves from the current checkout at match time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<ProcedurePrerequisiteSource>,
}

impl ProcedurePrerequisite {
    /// Create a prerequisite.
    #[must_use]
    pub fn new(key: impl Into<String>, expected: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            expected: expected.into(),
            source: None,
        }
    }

    /// Attach a deterministic checkout-local source.
    #[must_use]
    pub fn with_source(mut self, source: ProcedurePrerequisiteSource) -> Self {
        self.source = Some(source);
        self
    }
}

/// Proof-at-use metadata for a verified procedure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcedureVerification {
    /// Verification command recorded in the receipt.
    pub command: String,
    /// Required process exit code.
    pub expected_exit_code: i32,
    /// Required marker in the receipt's captured output.
    pub expected_output_contains: String,
    /// Receipt file path, absolute or relative to the repository checkout root.
    pub evidence_path: Option<String>,
    /// SHA-256 of the verified receipt file.
    pub evidence_sha256: Option<String>,
    /// Time the receipt was verified by Engram.
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub verified_at: Option<OffsetDateTime>,
}

impl ProcedureVerification {
    /// Create unverified verification requirements for a procedure candidate.
    #[must_use]
    pub fn new(
        command: impl Into<String>,
        expected_exit_code: i32,
        expected_output_contains: impl Into<String>,
    ) -> Self {
        Self {
            command: command.into(),
            expected_exit_code,
            expected_output_contains: expected_output_contains.into(),
            evidence_path: None,
            evidence_sha256: None,
            verified_at: None,
        }
    }
}

/// Structured procedure card retained separately from raw failed trajectories.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcedureCard {
    /// Task or triggering cue for this procedure.
    pub task: String,
    /// Exact caller-observed conditions required before applying it.
    #[serde(default)]
    pub prerequisites: Vec<ProcedurePrerequisite>,
    /// Exact successful command sequence.
    pub commands: Vec<String>,
    /// Known failure signatures that led to this procedure.
    #[serde(default)]
    pub failure_signatures: Vec<String>,
    /// How successful completion is verified.
    pub verification: ProcedureVerification,
    /// Expiry after which the procedure must abstain until reverified.
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub expires_at: Option<OffsetDateTime>,
}

impl ProcedureCard {
    /// Create a procedure candidate.
    #[must_use]
    pub fn new(
        task: impl Into<String>,
        commands: Vec<String>,
        verification: ProcedureVerification,
    ) -> Self {
        Self {
            task: task.into(),
            prerequisites: Vec::new(),
            commands,
            failure_signatures: Vec::new(),
            verification,
            expires_at: None,
        }
    }

    /// Add an exact prerequisite.
    #[must_use]
    pub fn with_prerequisite(mut self, prerequisite: ProcedurePrerequisite) -> Self {
        self.prerequisites.push(prerequisite);
        self
    }

    /// Add a known failure signature.
    #[must_use]
    pub fn with_failure_signature(mut self, signature: impl Into<String>) -> Self {
        self.failure_signatures.push(signature.into());
        self
    }

    /// Whether this card currently carries complete, unexpired receipt proof.
    #[must_use]
    pub fn is_verified_at(&self, now: OffsetDateTime) -> bool {
        self.verification.verified_at.is_some()
            && self.verification.evidence_path.is_some()
            && self.verification.evidence_sha256.is_some()
            && self.expires_at.map_or(true, |expires_at| expires_at > now)
    }
}

/// Scope in which a memory item applies.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MemoryScope {
    /// Applies globally.
    Global,
    /// Applies to the user independent of a project.
    User,
    /// Applies to a project.
    Project {
        /// Project ID when known.
        project_id: Option<Id>,
        /// Stable project name.
        project_name: String,
    },
    /// Applies to a task within a project.
    Task {
        /// Parent project ID when known.
        project_id: Option<Id>,
        /// Stable project name when known.
        project_name: Option<String>,
        /// Task ID when known.
        task_id: Option<Id>,
        /// Stable task name or tracker key.
        task_name: String,
    },
    /// Applies to an entity in the graph.
    Entity {
        /// Entity ID when known.
        entity_id: Option<Id>,
        /// Entity name.
        entity_name: String,
    },
    /// Applies to a git repository or local checkout.
    Repository {
        /// Repository entity ID when known.
        repository_id: Option<Id>,
        /// Canonical remote URL when known.
        remote_url: Option<String>,
        /// Local checkout path when known.
        local_path: Option<String>,
    },
    /// Applies only to a session.
    Session {
        /// Session ID.
        session_id: Id,
    },
    /// Custom scope.
    Custom {
        /// Scope label.
        name: String,
    },
}

impl MemoryScope {
    /// Create a project scope.
    #[must_use]
    pub fn project(project_name: impl Into<String>) -> Self {
        Self::Project {
            project_id: None,
            project_name: project_name.into(),
        }
    }

    /// Create a task scope.
    #[must_use]
    pub fn task(task_name: impl Into<String>) -> Self {
        Self::Task {
            project_id: None,
            project_name: None,
            task_id: None,
            task_name: task_name.into(),
        }
    }

    /// Create an entity scope.
    #[must_use]
    pub fn entity(entity_name: impl Into<String>) -> Self {
        Self::Entity {
            entity_id: None,
            entity_name: entity_name.into(),
        }
    }

    /// Create a repository scope.
    #[must_use]
    pub fn repository(remote_url: Option<String>, local_path: Option<String>) -> Self {
        Self::Repository {
            repository_id: None,
            remote_url,
            local_path,
        }
    }
}

/// A harness or interface that can write memory.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Harness {
    /// Claude Code.
    ClaudeCode,
    /// Codex.
    Codex,
    /// ChatGPT.
    ChatGpt,
    /// Cursor.
    Cursor,
    /// Other harness.
    Other(String),
}

impl std::fmt::Display for Harness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ClaudeCode => write!(f, "claude_code"),
            Self::Codex => write!(f, "codex"),
            Self::ChatGpt => write!(f, "chatgpt"),
            Self::Cursor => write!(f, "cursor"),
            Self::Other(value) => write!(f, "{}", value),
        }
    }
}

impl Harness {
    /// Parse from string.
    #[must_use]
    pub fn parse(value: &str) -> Self {
        match value.to_lowercase().replace('-', "_").as_str() {
            "claude_code" => Self::ClaudeCode,
            "codex" => Self::Codex,
            "chatgpt" | "chat_gpt" => Self::ChatGpt,
            "cursor" => Self::Cursor,
            other => Self::Other(other.to_string()),
        }
    }
}

/// Model identity associated with a writer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelIdentity {
    /// Provider name, such as "openai" or "anthropic".
    pub provider: String,
    /// Model name.
    pub model: String,
    /// Optional model version or alias.
    pub version: Option<String>,
}

impl ModelIdentity {
    /// Create a model identity.
    #[must_use]
    pub fn new(provider: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            model: model.into(),
            version: None,
        }
    }

    /// Set a version or alias.
    #[must_use]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }
}

/// Provenance for a memory writer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriterProvenance {
    /// Harness/interface that wrote the item.
    pub harness: Harness,
    /// Optional harness version.
    pub harness_version: Option<String>,
    /// Model identity.
    pub model: ModelIdentity,
    /// Human-facing surface, such as "desktop", "cli", or "mcp".
    pub surface: Option<String>,
    /// Actor label, such as "agent", "user", or "importer".
    pub actor: String,
    /// Session ID when the write came from a session.
    pub session_id: Option<Id>,
    /// Creation timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub written_at: OffsetDateTime,
}

impl WriterProvenance {
    /// Create writer provenance for an agent.
    #[must_use]
    pub fn agent(harness: Harness, model: ModelIdentity) -> Self {
        Self {
            harness,
            harness_version: None,
            model,
            surface: None,
            actor: "agent".to_string(),
            session_id: None,
            written_at: OffsetDateTime::now_utc(),
        }
    }

    /// Set the session ID.
    #[must_use]
    pub fn with_session(mut self, session_id: Id) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Set the surface.
    #[must_use]
    pub fn with_surface(mut self, surface: impl Into<String>) -> Self {
        self.surface = Some(surface.into());
        self
    }

    /// Set the harness version.
    #[must_use]
    pub fn with_harness_version(mut self, version: impl Into<String>) -> Self {
        self.harness_version = Some(version.into());
        self
    }
}

/// Where a memory claim came from.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimOrigin {
    /// The user explicitly stated the claim.
    UserStated,
    /// The user corrected a prior claim.
    UserCorrected,
    /// The agent observed the claim from files, tools, or project context.
    AgentObserved,
    /// The agent inferred the claim and it needs review.
    AgentInferred,
    /// The claim came from a tool result.
    ToolResult,
    /// The claim was imported from an external source.
    Imported,
    /// The claim was migrated from an older Engram store.
    Migrated,
    /// The claim came from a generated summary.
    GeneratedSummary,
    /// Custom origin.
    Custom(String),
}

impl ClaimOrigin {
    /// Default lifecycle status for a newly captured claim.
    #[must_use]
    pub fn default_status(&self) -> MemoryStatus {
        match self {
            Self::AgentInferred | Self::Imported | Self::Migrated | Self::GeneratedSummary => {
                MemoryStatus::NeedsReview
            }
            Self::UserStated | Self::UserCorrected | Self::AgentObserved | Self::ToolResult => {
                MemoryStatus::Active
            }
            Self::Custom(_) => MemoryStatus::NeedsReview,
        }
    }
}

impl std::fmt::Display for ClaimOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserStated => write!(f, "user_stated"),
            Self::UserCorrected => write!(f, "user_corrected"),
            Self::AgentObserved => write!(f, "agent_observed"),
            Self::AgentInferred => write!(f, "agent_inferred"),
            Self::ToolResult => write!(f, "tool_result"),
            Self::Imported => write!(f, "imported"),
            Self::Migrated => write!(f, "migrated"),
            Self::GeneratedSummary => write!(f, "generated_summary"),
            Self::Custom(value) => write!(f, "{value}"),
        }
    }
}

/// Lifecycle status of a memory item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MemoryStatus {
    /// Active and eligible for retrieval.
    #[default]
    Active,
    /// Captured but awaiting review.
    NeedsReview,
    /// Replaced by newer memory.
    Superseded,
    /// Archived and hidden from normal retrieval.
    Archived,
    /// Rejected during review.
    Rejected,
}

/// Archive metadata for memory hidden from normal retrieval.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchiveMetadata {
    /// Reason the memory was archived.
    pub reason: String,
    /// Actor or harness that archived it.
    pub archived_by: Option<String>,
    /// Archive timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub archived_at: OffsetDateTime,
}

impl ArchiveMetadata {
    /// Create archive metadata.
    #[must_use]
    pub fn new(reason: impl Into<String>, archived_by: Option<String>) -> Self {
        Self {
            reason: reason.into(),
            archived_by,
            archived_at: OffsetDateTime::now_utc(),
        }
    }
}

impl std::fmt::Display for MemoryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::NeedsReview => write!(f, "needs_review"),
            Self::Superseded => write!(f, "superseded"),
            Self::Archived => write!(f, "archived"),
            Self::Rejected => write!(f, "rejected"),
        }
    }
}

impl MemoryStatus {
    /// Parse from string.
    #[must_use]
    pub fn parse(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "active" => Self::Active,
            "needs_review" | "needsreview" => Self::NeedsReview,
            "superseded" => Self::Superseded,
            "archived" => Self::Archived,
            "rejected" => Self::Rejected,
            _ => Self::NeedsReview,
        }
    }
}

/// Lifecycle status for a server-minted correction proposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorrectionProposalStatus {
    /// The proposed replacement is awaiting an operator decision. It is immutable except for the
    /// dedicated atomic P0-to-P1 verification transition used by procedure corrections.
    Pending,
    /// The exact proposal pair was applied atomically.
    Applied,
}

impl std::fmt::Display for CorrectionProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Applied => write!(f, "applied"),
        }
    }
}

/// Typed, server-minted link between an obsolete item and a proposed replacement.
///
/// The proposal is not reviewer authority. Its digest binds the exact pending pair so an operator
/// can reject a changed proposal instead of silently applying it. Pair identity, kind, and scope
/// never change; a procedure replacement may gain receipt proof only through the atomic P0-to-P1
/// verification transition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CorrectionProposal {
    /// Stable proposal identifier.
    pub id: Id,
    /// Active item the proposal would supersede.
    pub obsolete_id: Id,
    /// `needs_review` replacement created with the proposal. Procedure proof may change only in
    /// the dedicated atomic P0-to-P1 verification transition.
    pub replacement_id: Id,
    /// Server-derived kind shared by the pair.
    pub memory_kind: MemoryKind,
    /// Server-derived scope shared by the pair.
    pub scope: MemoryScope,
    /// SHA-256 over the canonical pending pair.
    ///
    /// For a procedure correction this is the proposal-time P0 digest until Engram verifies the
    /// inactive replacement's receipt, then the verification transition rotates it to P1. The
    /// proposal remains pending throughout that transition.
    pub canonical_digest: String,
    /// Frozen canonicalization version used for both pending and applied pair digests.
    #[serde(default = "default_correction_digest_schema_version")]
    pub digest_schema_version: u32,
    /// SHA-256 over the exact applied pair, used to make completed retries fail closed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub applied_digest: Option<String>,
    /// Proposal lifecycle state.
    pub status: CorrectionProposalStatus,
    /// Agent provenance recorded when the proposal was created.
    pub proposer: WriterProvenance,
    /// Proposal creation timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// Time the exact pair was applied, if it was applied.
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub applied_at: Option<OffsetDateTime>,
}

impl CorrectionProposal {
    /// Create a pending proposal for an exact memory pair.
    #[must_use]
    pub fn new(
        obsolete_id: Id,
        replacement_id: Id,
        memory_kind: MemoryKind,
        scope: MemoryScope,
        canonical_digest: impl Into<String>,
        proposer: WriterProvenance,
    ) -> Self {
        Self {
            id: Id::new(),
            obsolete_id,
            replacement_id,
            memory_kind,
            scope,
            canonical_digest: canonical_digest.into(),
            digest_schema_version: default_correction_digest_schema_version(),
            applied_digest: None,
            status: CorrectionProposalStatus::Pending,
            proposer,
            created_at: OffsetDateTime::now_utc(),
            applied_at: None,
        }
    }

    /// Mark a proposal applied without changing its immutable pair or digest.
    #[must_use]
    pub fn with_applied(mut self) -> Self {
        self.status = CorrectionProposalStatus::Applied;
        self.applied_at = Some(OffsetDateTime::now_utc());
        self
    }

    /// Bind the exact persisted pair produced by applying this proposal.
    #[must_use]
    pub fn with_applied_digest(mut self, digest: impl Into<String>) -> Self {
        self.applied_digest = Some(digest.into());
        self
    }

    /// Rotate the canonical digest after an atomic, still-inactive proposal transition.
    #[must_use]
    pub fn with_canonical_digest(mut self, digest: impl Into<String>) -> Self {
        self.canonical_digest = digest.into();
        self
    }
}

const fn default_correction_digest_schema_version() -> u32 {
    1
}

/// Review state derived from lifecycle status and verified reviewer authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryReviewState {
    /// Active memory with review evidence from a verified authority.
    ///
    /// Engram does not currently expose a verified reviewer-authority channel, so caller-supplied
    /// `manual_review` evidence never produces this state.
    Reviewed,
    /// Active memory without verified reviewer authority.
    ActiveUnreviewed,
    /// Memory captured but awaiting review.
    NeedsReview,
    /// Memory has been replaced by newer memory.
    Superseded,
    /// Memory is archived and hidden from normal retrieval.
    Archived,
    /// Memory was rejected during review.
    Rejected,
}

impl std::fmt::Display for MemoryReviewState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reviewed => write!(f, "reviewed"),
            Self::ActiveUnreviewed => write!(f, "active_unreviewed"),
            Self::NeedsReview => write!(f, "needs_review"),
            Self::Superseded => write!(f, "superseded"),
            Self::Archived => write!(f, "archived"),
            Self::Rejected => write!(f, "rejected"),
        }
    }
}

/// Freshness signal derived from review scheduling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryFreshness {
    /// No review deadline has been scheduled.
    Unscheduled,
    /// A future review is scheduled.
    ReviewScheduled,
    /// The memory is due for review.
    ReviewDue,
}

impl std::fmt::Display for MemoryFreshness {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unscheduled => write!(f, "unscheduled"),
            Self::ReviewScheduled => write!(f, "review_scheduled"),
            Self::ReviewDue => write!(f, "review_due"),
        }
    }
}

impl MemoryFreshness {
    /// Derive freshness from an optional review deadline.
    #[must_use]
    pub fn from_review_after(review_after: Option<OffsetDateTime>, now: OffsetDateTime) -> Self {
        match review_after {
            Some(review_after) if review_after <= now => Self::ReviewDue,
            Some(_) => Self::ReviewScheduled,
            None => Self::Unscheduled,
        }
    }
}

/// Confidence assigned to a memory item.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MemoryConfidence(f32);

impl MemoryConfidence {
    /// Minimum confidence value.
    pub const MIN: f32 = 0.0;
    /// Maximum confidence value.
    pub const MAX: f32 = 1.0;

    /// Create confidence, clamped to the valid range.
    #[must_use]
    pub fn new(value: f32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    /// Get the numeric value.
    #[must_use]
    pub const fn value(self) -> f32 {
        self.0
    }
}

impl Default for MemoryConfidence {
    fn default() -> Self {
        Self(0.8)
    }
}

/// Type of evidence backing a memory item.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    /// A session event.
    SessionEvent,
    /// A tool call or response.
    ToolCall,
    /// A file path or file fragment.
    File,
    /// A git commit.
    GitCommit,
    /// A web URL.
    Url,
    /// A document in the knowledge system.
    Document,
    /// A prior observation.
    Observation,
    /// Unverified assertion that a manual review occurred.
    ///
    /// This legacy evidence kind remains serializable for auditability but does not confer reviewed
    /// trust. A caller-controlled field cannot prove human authority.
    ManualReview,
    /// Custom evidence.
    Custom(String),
}

/// Evidence backing a memory item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceRef {
    /// Evidence kind.
    pub kind: EvidenceKind,
    /// Stable target identifier, path, URL, or commit SHA.
    pub target: String,
    /// Optional human-readable summary.
    pub summary: Option<String>,
    /// Optional excerpt or selector.
    pub excerpt: Option<String>,
    /// When the evidence was observed.
    #[serde(with = "time::serde::rfc3339")]
    pub observed_at: OffsetDateTime,
}

impl EvidenceRef {
    /// Create evidence.
    #[must_use]
    pub fn new(kind: EvidenceKind, target: impl Into<String>) -> Self {
        Self {
            kind,
            target: target.into(),
            summary: None,
            excerpt: None,
            observed_at: OffsetDateTime::now_utc(),
        }
    }

    /// Set a summary.
    #[must_use]
    pub fn with_summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = Some(summary.into());
        self
    }

    /// Set an excerpt or selector.
    #[must_use]
    pub fn with_excerpt(mut self, excerpt: impl Into<String>) -> Self {
        self.excerpt = Some(excerpt.into());
        self
    }
}

/// Compact writer metadata surfaced with trust metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryWriterMetadata {
    /// Harness/interface that wrote the item.
    pub harness: Harness,
    /// Optional harness version.
    pub harness_version: Option<String>,
    /// Model provider.
    pub model_provider: String,
    /// Model name.
    pub model: String,
    /// Optional model version or alias.
    pub model_version: Option<String>,
    /// Human-facing surface, such as "desktop", "cli", or "mcp".
    pub surface: Option<String>,
    /// Actor label, such as "agent", "user", or "importer".
    pub actor: String,
}

impl From<&WriterProvenance> for MemoryWriterMetadata {
    fn from(writer: &WriterProvenance) -> Self {
        Self {
            harness: writer.harness.clone(),
            harness_version: writer.harness_version.clone(),
            model_provider: writer.model.provider.clone(),
            model: writer.model.model.clone(),
            model_version: writer.model.version.clone(),
            surface: writer.surface.clone(),
            actor: writer.actor.clone(),
        }
    }
}

/// Derived trust metadata surfaced to agents alongside MemoryItems.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryTrustMetadata {
    /// Memory item ID this metadata describes.
    pub memory_id: Id,
    /// Memory kind.
    pub kind: MemoryKind,
    /// Scope where this memory applies.
    pub scope: MemoryScope,
    /// Lifecycle status.
    pub status: MemoryStatus,
    /// Review state derived from lifecycle status and verified reviewer authority.
    pub review_state: MemoryReviewState,
    /// Freshness signal derived from review_after.
    pub freshness: MemoryFreshness,
    /// Where the claim came from.
    pub claim_origin: ClaimOrigin,
    /// Confidence score.
    pub confidence: f32,
    /// Number of evidence records attached to the memory.
    pub evidence_count: usize,
    /// Whether the memory has at least one evidence record.
    pub has_evidence: bool,
    /// Evidence kinds attached to the memory.
    pub evidence_kinds: Vec<EvidenceKind>,
    /// Whether the memory has verified reviewer authority.
    pub reviewed: bool,
    /// Whether the memory contains an unverified manual-review assertion.
    pub review_asserted: bool,
    /// Whether review_after is due.
    pub review_due: bool,
    /// Last update timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    /// Optional last retrieval/application timestamp.
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub last_used_at: Option<OffsetDateTime>,
    /// Optional scheduled review timestamp.
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub review_after: Option<OffsetDateTime>,
    /// Writer metadata.
    pub writer: MemoryWriterMetadata,
}

impl MemoryTrustMetadata {
    /// Build metadata for a memory item using a supplied current timestamp.
    #[must_use]
    pub fn from_item(item: &MemoryItem, now: OffsetDateTime) -> Self {
        let review_asserted = item
            .evidence
            .iter()
            .any(|evidence| matches!(&evidence.kind, EvidenceKind::ManualReview));
        // There is intentionally no caller-controlled path to verified reviewer authority. Keep
        // manual-review claims visible for auditability without upgrading their trust state.
        let reviewed = false;
        let review_state = match item.status {
            MemoryStatus::Active if reviewed => MemoryReviewState::Reviewed,
            MemoryStatus::Active => MemoryReviewState::ActiveUnreviewed,
            MemoryStatus::NeedsReview => MemoryReviewState::NeedsReview,
            MemoryStatus::Superseded => MemoryReviewState::Superseded,
            MemoryStatus::Archived => MemoryReviewState::Archived,
            MemoryStatus::Rejected => MemoryReviewState::Rejected,
        };
        let freshness = MemoryFreshness::from_review_after(item.review_after, now);

        Self {
            memory_id: item.id,
            kind: item.kind.clone(),
            scope: item.scope.clone(),
            status: item.status,
            review_state,
            freshness,
            claim_origin: item.origin.clone(),
            confidence: item.confidence.value(),
            evidence_count: item.evidence.len(),
            has_evidence: !item.evidence.is_empty(),
            evidence_kinds: item
                .evidence
                .iter()
                .map(|evidence| evidence.kind.clone())
                .collect(),
            reviewed,
            review_asserted,
            review_due: freshness == MemoryFreshness::ReviewDue,
            updated_at: item.updated_at,
            last_used_at: item.last_used_at,
            review_after: item.review_after,
            writer: MemoryWriterMetadata::from(&item.writer),
        }
    }
}

/// Source-grounded memory item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    /// Unique identifier.
    pub id: Id,
    /// Type of memory.
    pub kind: MemoryKind,
    /// Short title.
    pub title: String,
    /// Markdown-safe content.
    pub content: String,
    /// Scope where this applies.
    pub scope: MemoryScope,
    /// Where the claim came from.
    pub origin: ClaimOrigin,
    /// Writer provenance.
    pub writer: WriterProvenance,
    /// Evidence backing the claim.
    #[serde(default)]
    pub evidence: Vec<EvidenceRef>,
    /// Confidence score.
    pub confidence: MemoryConfidence,
    /// Lifecycle status.
    pub status: MemoryStatus,
    /// Items superseded by this item.
    #[serde(default)]
    pub supersedes: Vec<Id>,
    /// Tags used for vault/frontmatter filtering.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Creation timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// Last update timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    /// Last retrieval or application timestamp.
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub last_used_at: Option<OffsetDateTime>,
    /// Review timestamp for recalibration.
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub review_after: Option<OffsetDateTime>,
    /// Archive metadata when the item has been archived.
    #[serde(default)]
    pub archive: Option<ArchiveMetadata>,
    /// Structured procedure details when kind is `procedure`.
    #[serde(default)]
    pub procedure: Option<ProcedureCard>,
    /// Server-minted proposal link while this item is an inactive correction replacement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correction_proposal_id: Option<Id>,
    /// Server-minted proposal lock while this active item is the obsolete half of a pending pair.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending_correction_proposal_id: Option<Id>,
}

impl MemoryItem {
    /// Create a new memory item.
    #[must_use]
    pub fn new(
        kind: MemoryKind,
        title: impl Into<String>,
        content: impl Into<String>,
        scope: MemoryScope,
        origin: ClaimOrigin,
        writer: WriterProvenance,
    ) -> Self {
        let now = OffsetDateTime::now_utc();
        let status = origin.default_status();

        Self {
            id: Id::new(),
            kind,
            title: title.into(),
            content: content.into(),
            scope,
            origin,
            writer,
            evidence: Vec::new(),
            confidence: MemoryConfidence::default(),
            status,
            supersedes: Vec::new(),
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
            last_used_at: None,
            review_after: None,
            archive: None,
            procedure: None,
            correction_proposal_id: None,
            pending_correction_proposal_id: None,
        }
    }

    /// Add evidence.
    #[must_use]
    pub fn with_evidence(mut self, evidence: EvidenceRef) -> Self {
        self.evidence.push(evidence);
        self.updated_at = OffsetDateTime::now_utc();
        self
    }

    /// Set confidence.
    #[must_use]
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = MemoryConfidence::new(confidence);
        self.updated_at = OffsetDateTime::now_utc();
        self
    }

    /// Set lifecycle status.
    #[must_use]
    pub fn with_status(mut self, status: MemoryStatus) -> Self {
        self.status = status;
        self.updated_at = OffsetDateTime::now_utc();
        self
    }

    /// Archive this item with metadata.
    #[must_use]
    pub fn with_archive(mut self, reason: impl Into<String>, archived_by: Option<String>) -> Self {
        self.status = MemoryStatus::Archived;
        self.archive = Some(ArchiveMetadata::new(reason, archived_by));
        self.updated_at = OffsetDateTime::now_utc();
        self
    }

    /// Add a tag.
    #[must_use]
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self.updated_at = OffsetDateTime::now_utc();
        self
    }

    /// Mark this item as superseding another item.
    #[must_use]
    pub fn with_superseded_item(mut self, item_id: Id) -> Self {
        self.supersedes.push(item_id);
        self.updated_at = OffsetDateTime::now_utc();
        self
    }

    /// Attach structured procedure details.
    #[must_use]
    pub fn with_procedure(mut self, procedure: ProcedureCard) -> Self {
        self.procedure = Some(procedure);
        self.updated_at = OffsetDateTime::now_utc();
        self
    }

    /// Set the next review time.
    #[must_use]
    pub fn with_review_after(mut self, review_after: OffsetDateTime) -> Self {
        self.review_after = Some(review_after);
        self.updated_at = OffsetDateTime::now_utc();
        self
    }

    /// Mark the item as used.
    pub fn mark_used(&mut self) {
        let now = OffsetDateTime::now_utc();
        self.last_used_at = Some(now);
        self.updated_at = now;
    }

    /// Whether the item is eligible for normal retrieval.
    #[must_use]
    pub const fn is_retrievable(&self) -> bool {
        matches!(self.status, MemoryStatus::Active)
    }

    /// Whether the item should be reviewed at the provided time.
    #[must_use]
    pub fn needs_review_at(&self, now: OffsetDateTime) -> bool {
        self.status == MemoryStatus::NeedsReview
            || self
                .review_after
                .is_some_and(|review_after| review_after <= now)
    }

    /// Return derived trust metadata for agent-facing retrieval surfaces.
    #[must_use]
    pub fn trust_metadata(&self) -> MemoryTrustMetadata {
        MemoryTrustMetadata::from_item(self, OffsetDateTime::now_utc())
    }
}

/// Type of change recorded in a knowledge commit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryChangeType {
    /// A new item was added.
    Added,
    /// An existing item was updated.
    Updated,
    /// An item superseded another item.
    Superseded,
    /// An item was archived.
    Archived,
    /// An item was rejected.
    Rejected,
    /// A graph link was created.
    Linked,
    /// A graph link was removed.
    Unlinked,
}

impl std::fmt::Display for MemoryChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Added => write!(f, "added"),
            Self::Updated => write!(f, "updated"),
            Self::Superseded => write!(f, "superseded"),
            Self::Archived => write!(f, "archived"),
            Self::Rejected => write!(f, "rejected"),
            Self::Linked => write!(f, "linked"),
            Self::Unlinked => write!(f, "unlinked"),
        }
    }
}

/// A single memory change inside a knowledge commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryChange {
    /// Change type.
    pub change_type: MemoryChangeType,
    /// Changed memory item ID, when applicable.
    pub item_id: Option<Id>,
    /// Human-readable title.
    pub title: String,
    /// Short explanation of the diff.
    pub summary: String,
    /// Content hash before the change, when known.
    pub before_hash: Option<String>,
    /// Content hash after the change, when known.
    pub after_hash: Option<String>,
}

impl MemoryChange {
    /// Create a change record.
    #[must_use]
    pub fn new(
        change_type: MemoryChangeType,
        title: impl Into<String>,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            change_type,
            item_id: None,
            title: title.into(),
            summary: summary.into(),
            before_hash: None,
            after_hash: None,
        }
    }

    /// Attach the changed item ID.
    #[must_use]
    pub fn with_item(mut self, item_id: Id) -> Self {
        self.item_id = Some(item_id);
        self
    }

    /// Attach content hashes.
    #[must_use]
    pub fn with_hashes(mut self, before_hash: Option<String>, after_hash: Option<String>) -> Self {
        self.before_hash = before_hash;
        self.after_hash = after_hash;
        self
    }
}

/// Git-like commit of accumulated memory changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeCommit {
    /// Unique identifier.
    pub id: Id,
    /// Parent commit when this is part of a chain.
    pub parent_id: Option<Id>,
    /// Session that produced the commit.
    pub session_id: Option<Id>,
    /// Writer that produced the commit.
    pub writer: WriterProvenance,
    /// Commit message.
    pub message: String,
    /// Changes included in the commit.
    #[serde(default)]
    pub changes: Vec<MemoryChange>,
    /// Creation timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

impl KnowledgeCommit {
    /// Create a knowledge commit.
    #[must_use]
    pub fn new(writer: WriterProvenance, message: impl Into<String>) -> Self {
        Self {
            id: Id::new(),
            parent_id: None,
            session_id: None,
            writer,
            message: message.into(),
            changes: Vec::new(),
            created_at: OffsetDateTime::now_utc(),
        }
    }

    /// Set the parent commit.
    #[must_use]
    pub fn with_parent(mut self, parent_id: Id) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// Set the producing session.
    #[must_use]
    pub fn with_session(mut self, session_id: Id) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Add a change.
    #[must_use]
    pub fn with_change(mut self, change: MemoryChange) -> Self {
        self.changes.push(change);
        self
    }

    /// Number of changes in this commit.
    #[must_use]
    pub fn change_count(&self) -> usize {
        self.changes.len()
    }
}

/// Cursor returned with an orientation packet for later `changes_since` checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryCursor {
    /// Latest commit included in a context packet, when known.
    pub commit_id: Option<Id>,
    /// Latest memory timestamp included in a context packet.
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
}

impl MemoryCursor {
    /// Create a cursor at the current time.
    #[must_use]
    pub fn now(commit_id: Option<Id>) -> Self {
        Self {
            commit_id,
            timestamp: OffsetDateTime::now_utc(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Duration;

    fn codex_writer() -> WriterProvenance {
        WriterProvenance::agent(Harness::Codex, ModelIdentity::new("openai", "gpt-5.5"))
            .with_surface("desktop")
    }

    #[test]
    fn user_stated_memory_is_active_by_default() {
        let item = MemoryItem::new(
            MemoryKind::Preference,
            "Commit message preference",
            "Do not mention the AI agent in commit messages.",
            MemoryScope::User,
            ClaimOrigin::UserStated,
            codex_writer(),
        );

        assert_eq!(item.status, MemoryStatus::Active);
        assert!(item.is_retrievable());
    }

    #[test]
    fn inferred_memory_requires_review_by_default() {
        let item = MemoryItem::new(
            MemoryKind::Preference,
            "Possible formatting preference",
            "User may prefer short status updates.",
            MemoryScope::User,
            ClaimOrigin::AgentInferred,
            codex_writer(),
        );

        assert_eq!(item.status, MemoryStatus::NeedsReview);
        assert!(!item.is_retrievable());
        assert!(item.needs_review_at(OffsetDateTime::now_utc()));
    }

    #[test]
    fn writer_provenance_distinguishes_harness_model_and_surface() {
        let session_id = Id::new();
        let writer = WriterProvenance::agent(
            Harness::ClaudeCode,
            ModelIdentity::new("anthropic", "opus").with_version("4.7"),
        )
        .with_harness_version("1.2.3")
        .with_surface("cli")
        .with_session(session_id);

        assert_eq!(writer.harness, Harness::ClaudeCode);
        assert_eq!(writer.model.provider, "anthropic");
        assert_eq!(writer.model.version.as_deref(), Some("4.7"));
        assert_eq!(writer.surface.as_deref(), Some("cli"));
        assert_eq!(writer.session_id, Some(session_id));
    }

    #[test]
    fn memory_items_carry_evidence_scope_and_review_schedule() {
        let review_after = OffsetDateTime::now_utc() + Duration::days(30);
        let item = MemoryItem::new(
            MemoryKind::Decision,
            "Extend Engram first",
            "Build Memory OS as an Engram extension before considering a rewrite.",
            MemoryScope::project("engram"),
            ClaimOrigin::UserCorrected,
            codex_writer(),
        )
        .with_evidence(
            EvidenceRef::new(
                EvidenceKind::Document,
                "docs/MEMORY_OS_IMPLEMENTATION_PLAN.md",
            )
            .with_summary("Implementation plan"),
        )
        .with_confidence(1.5)
        .with_tag("memory-os")
        .with_review_after(review_after);

        assert_eq!(item.evidence.len(), 1);
        assert_eq!(item.confidence.value(), 1.0);
        assert_eq!(item.tags, vec!["memory-os"]);
        assert!(!item.needs_review_at(OffsetDateTime::now_utc()));
        assert!(item.needs_review_at(review_after + Duration::seconds(1)));
    }

    #[test]
    fn knowledge_commit_tracks_memory_diff() {
        let item_id = Id::new();
        let commit = KnowledgeCommit::new(codex_writer(), "Capture Memory OS decision")
            .with_change(
                MemoryChange::new(
                    MemoryChangeType::Added,
                    "Extend Engram first",
                    "Added the core product direction.",
                )
                .with_item(item_id),
            );

        assert_eq!(commit.change_count(), 1);
        assert_eq!(commit.changes[0].item_id, Some(item_id));
        assert_eq!(commit.changes[0].change_type, MemoryChangeType::Added);
    }

    #[test]
    fn serde_uses_tagged_scopes_for_human_readable_vault_frontmatter() {
        let scope = MemoryScope::Repository {
            repository_id: None,
            remote_url: Some("git@github.com:ymeiri/engram.git".to_string()),
            local_path: Some("/Users/yuval.meiri/projects/engram".to_string()),
        };

        let value = serde_json::to_value(scope).unwrap();

        assert_eq!(value["type"], "repository");
        assert_eq!(value["remote_url"], "git@github.com:ymeiri/engram.git");
        assert_eq!(value["local_path"], "/Users/yuval.meiri/projects/engram");
    }

    #[test]
    fn caller_supplied_manual_review_is_an_assertion_not_authority() {
        let item = MemoryItem::new(
            MemoryKind::Decision,
            "Claimed review",
            "This item contains an unverified review assertion.",
            MemoryScope::project("engram"),
            ClaimOrigin::AgentObserved,
            codex_writer(),
        )
        .with_evidence(EvidenceRef::new(
            EvidenceKind::ManualReview,
            "claimed-reviewer",
        ));

        let trust = item.trust_metadata();

        assert!(trust.review_asserted);
        assert!(!trust.reviewed);
        assert_eq!(trust.review_state, MemoryReviewState::ActiveUnreviewed);
    }

    #[test]
    fn procedure_card_requires_complete_unexpired_receipt_proof() {
        let now = OffsetDateTime::now_utc();
        let mut card = ProcedureCard::new(
            "run queue integration tests",
            vec!["cargo test -p queue-tests".to_string()],
            ProcedureVerification::new("cargo test -p queue-tests", 0, "test result: ok"),
        )
        .with_prerequisite(
            ProcedurePrerequisite::new("cargo.version", "1.80.0").with_source(
                ProcedurePrerequisiteSource::Toml {
                    relative_path: "rust-toolchain.toml".to_string(),
                    key_path: vec!["toolchain".to_string(), "channel".to_string()],
                },
            ),
        )
        .with_failure_signature("unknown option --all");

        assert!(!card.is_verified_at(now));
        card.verification.evidence_path = Some(".engram/proofs/queue.json".to_string());
        card.verification.evidence_sha256 = Some("abc123".to_string());
        card.verification.verified_at = Some(now);
        card.expires_at = Some(now + Duration::days(30));
        assert!(card.is_verified_at(now));
        assert!(!card.is_verified_at(now + Duration::days(31)));

        let item = MemoryItem::new(
            MemoryKind::Procedure,
            "Queue integration test procedure",
            "Use the verified queue integration command for this toolchain.",
            MemoryScope::project("engram"),
            ClaimOrigin::AgentObserved,
            codex_writer(),
        )
        .with_procedure(card);
        let round_trip: MemoryItem =
            serde_json::from_value(serde_json::to_value(&item).unwrap()).unwrap();
        assert_eq!(round_trip.procedure, item.procedure);
    }

    #[test]
    fn legacy_prerequisite_without_source_remains_deserializable() {
        let prerequisite: ProcedurePrerequisite = serde_json::from_value(serde_json::json!({
            "key": "cargo.version",
            "expected": "1.80.0"
        }))
        .unwrap();

        assert_eq!(prerequisite.key, "cargo.version");
        assert_eq!(prerequisite.expected, "1.80.0");
        assert!(prerequisite.source.is_none());
    }
}
