//! Agent-native obligation types.
//!
//! Obligations turn harness guidance into explicit, inspectable work items. A
//! harness can create obligations when session cues imply follow-up work, then
//! resolve or skip them before final response.

use crate::id::Id;
use crate::memory::{EvidenceRef, MemoryScope, WriterProvenance};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Agent obligation kind.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentObligationKind {
    /// A durable document was created or changed and needs a disposition.
    DocumentDisposition,
    /// A tool call failed and the agent must recover, retry, or abandon explicitly.
    ToolFailureRecovery,
    /// The task requires reading source files before asserting behavior or changing code.
    SourceReading,
    /// The task requires reading design docs, philosophy, or project instructions.
    DesignContextReading,
    /// A discovery appears durable enough to consider as Memory OS memory.
    MemoryWriteCandidate,
    /// Work should update or compile a handoff.
    HandoffUpdate,
    /// Code changes require relevant verification.
    TestVerification,
    /// Commit work requires checking project/user commit preferences.
    CommitPreferenceCheck,
    /// Custom obligation kind.
    Custom(String),
}

impl std::fmt::Display for AgentObligationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DocumentDisposition => write!(f, "document_disposition"),
            Self::ToolFailureRecovery => write!(f, "tool_failure_recovery"),
            Self::SourceReading => write!(f, "source_reading"),
            Self::DesignContextReading => write!(f, "design_context_reading"),
            Self::MemoryWriteCandidate => write!(f, "memory_write_candidate"),
            Self::HandoffUpdate => write!(f, "handoff_update"),
            Self::TestVerification => write!(f, "test_verification"),
            Self::CommitPreferenceCheck => write!(f, "commit_preference_check"),
            Self::Custom(value) => write!(f, "{value}"),
        }
    }
}

impl AgentObligationKind {
    /// Parse an obligation kind.
    #[must_use]
    pub fn parse(value: &str) -> Self {
        match value.to_lowercase().replace('-', "_").as_str() {
            "document" | "document_disposition" | "doc_disposition" => Self::DocumentDisposition,
            "tool_failure" | "tool_failure_recovery" | "failed_tool" => Self::ToolFailureRecovery,
            "source" | "source_reading" | "code_reading" => Self::SourceReading,
            "design" | "design_context" | "design_context_reading" => Self::DesignContextReading,
            "memory_write" | "memory_write_candidate" => Self::MemoryWriteCandidate,
            "handoff" | "handoff_update" => Self::HandoffUpdate,
            "test" | "tests" | "test_verification" => Self::TestVerification,
            "commit" | "commit_preference" | "commit_preference_check" => {
                Self::CommitPreferenceCheck
            }
            other => Self::Custom(other.to_string()),
        }
    }
}

/// Status of an agent obligation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AgentObligationStatus {
    /// Obligation is still open.
    #[default]
    Open,
    /// Obligation was resolved.
    Resolved,
    /// Obligation was intentionally skipped with a reason.
    Skipped,
}

impl std::fmt::Display for AgentObligationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "open"),
            Self::Resolved => write!(f, "resolved"),
            Self::Skipped => write!(f, "skipped"),
        }
    }
}

impl AgentObligationStatus {
    /// Parse a status.
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_lowercase().replace('-', "_").as_str() {
            "open" => Some(Self::Open),
            "resolved" | "done" => Some(Self::Resolved),
            "skipped" | "skip" => Some(Self::Skipped),
            _ => None,
        }
    }
}

/// Resolution action for an agent obligation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentObligationResolutionKind {
    /// Full document was indexed.
    IndexedDocument,
    /// Compact Memory OS item was written.
    MemoryRecorded,
    /// Knowledge registry entry was created or updated.
    KnowledgeRegistered,
    /// Rolling handoff links the context.
    HandoffLinked,
    /// Agent inspected schema/help and retried successfully.
    RetriedTool,
    /// Agent abandoned a failed path explicitly.
    Abandoned,
    /// Required source files were read.
    SourceRead,
    /// Required design/project philosophy docs were read.
    DesignContextRead,
    /// Tests or checks were run.
    TestsRun,
    /// Commit preferences were checked.
    PreferenceChecked,
    /// Agent intentionally skipped with a reason.
    SkippedWithReason,
    /// Custom resolution.
    Custom(String),
}

impl std::fmt::Display for AgentObligationResolutionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IndexedDocument => write!(f, "indexed_document"),
            Self::MemoryRecorded => write!(f, "memory_recorded"),
            Self::KnowledgeRegistered => write!(f, "knowledge_registered"),
            Self::HandoffLinked => write!(f, "handoff_linked"),
            Self::RetriedTool => write!(f, "retried_tool"),
            Self::Abandoned => write!(f, "abandoned"),
            Self::SourceRead => write!(f, "source_read"),
            Self::DesignContextRead => write!(f, "design_context_read"),
            Self::TestsRun => write!(f, "tests_run"),
            Self::PreferenceChecked => write!(f, "preference_checked"),
            Self::SkippedWithReason => write!(f, "skipped_with_reason"),
            Self::Custom(value) => write!(f, "{value}"),
        }
    }
}

impl AgentObligationResolutionKind {
    /// Parse a resolution kind.
    #[must_use]
    pub fn parse(value: &str) -> Self {
        match value.to_lowercase().replace('-', "_").as_str() {
            "index" | "indexed" | "indexed_document" => Self::IndexedDocument,
            "memory" | "record" | "memory_recorded" => Self::MemoryRecorded,
            "register" | "knowledge_registered" => Self::KnowledgeRegistered,
            "handoff" | "handoff_linked" => Self::HandoffLinked,
            "retry" | "retried_tool" => Self::RetriedTool,
            "abandon" | "abandoned" => Self::Abandoned,
            "source" | "source_read" => Self::SourceRead,
            "design" | "design_context_read" => Self::DesignContextRead,
            "test" | "tests" | "tests_run" => Self::TestsRun,
            "preference" | "preference_checked" => Self::PreferenceChecked,
            "skip" | "skipped" | "skipped_with_reason" => Self::SkippedWithReason,
            other => Self::Custom(other.to_string()),
        }
    }
}

/// What triggered an obligation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentObligationTrigger {
    /// Trigger kind such as prompt, git_status, tool_failure, or agent_decision.
    pub kind: String,
    /// Optional target, for example a file path or tool name.
    pub target: Option<String>,
    /// Human-readable trigger summary.
    pub summary: String,
}

impl AgentObligationTrigger {
    /// Create a trigger.
    #[must_use]
    pub fn new(kind: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            target: None,
            summary: summary.into(),
        }
    }

    /// Attach a target.
    #[must_use]
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }
}

/// Resolution details for an obligation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentObligationResolution {
    /// Resolution kind.
    pub kind: AgentObligationResolutionKind,
    /// Human-readable summary.
    pub summary: String,
    /// Optional evidence refs proving the resolution.
    #[serde(default)]
    pub evidence: Vec<EvidenceRef>,
    /// Who resolved the obligation.
    pub actor: String,
    /// When it was resolved.
    #[serde(with = "time::serde::rfc3339")]
    pub resolved_at: OffsetDateTime,
}

impl AgentObligationResolution {
    /// Create a resolution.
    #[must_use]
    pub fn new(
        kind: AgentObligationResolutionKind,
        summary: impl Into<String>,
        actor: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            summary: summary.into(),
            evidence: Vec::new(),
            actor: actor.into(),
            resolved_at: OffsetDateTime::now_utc(),
        }
    }

    /// Add evidence.
    #[must_use]
    pub fn with_evidence(mut self, evidence: EvidenceRef) -> Self {
        self.evidence.push(evidence);
        self
    }
}

/// An explicit obligation a harness should resolve before final response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentObligation {
    /// Unique ID.
    pub id: Id,
    /// Obligation kind.
    pub kind: AgentObligationKind,
    /// Short title.
    pub title: String,
    /// Human-readable details.
    pub description: String,
    /// Scope where the obligation applies.
    pub scope: MemoryScope,
    /// Trigger that generated it.
    pub trigger: AgentObligationTrigger,
    /// Allowed or expected resolutions.
    #[serde(default)]
    pub required_resolution: Vec<AgentObligationResolutionKind>,
    /// Current status.
    pub status: AgentObligationStatus,
    /// Evidence that the obligation is valid.
    #[serde(default)]
    pub evidence: Vec<EvidenceRef>,
    /// Writer that created the obligation.
    pub writer: WriterProvenance,
    /// Resolution details if closed.
    pub resolution: Option<AgentObligationResolution>,
    /// Tags for filtering and diagnostics.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Creation time.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// Last update time.
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl AgentObligation {
    /// Create a new open obligation.
    #[must_use]
    pub fn new(
        kind: AgentObligationKind,
        title: impl Into<String>,
        description: impl Into<String>,
        scope: MemoryScope,
        trigger: AgentObligationTrigger,
        writer: WriterProvenance,
    ) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: Id::new(),
            kind,
            title: title.into(),
            description: description.into(),
            scope,
            trigger,
            required_resolution: Vec::new(),
            status: AgentObligationStatus::Open,
            evidence: Vec::new(),
            writer,
            resolution: None,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Add an expected resolution.
    #[must_use]
    pub fn with_required_resolution(mut self, resolution: AgentObligationResolutionKind) -> Self {
        self.required_resolution.push(resolution);
        self
    }

    /// Add evidence.
    #[must_use]
    pub fn with_evidence(mut self, evidence: EvidenceRef) -> Self {
        self.evidence.push(evidence);
        self
    }

    /// Add a tag.
    #[must_use]
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Resolve the obligation.
    pub fn resolve(&mut self, resolution: AgentObligationResolution) {
        self.status = AgentObligationStatus::Resolved;
        self.updated_at = resolution.resolved_at;
        self.resolution = Some(resolution);
    }

    /// Skip the obligation with an explicit reason.
    pub fn skip(&mut self, reason: impl Into<String>, actor: impl Into<String>) {
        self.status = AgentObligationStatus::Skipped;
        let resolution = AgentObligationResolution::new(
            AgentObligationResolutionKind::SkippedWithReason,
            reason,
            actor,
        );
        self.updated_at = resolution.resolved_at;
        self.resolution = Some(resolution);
    }

    /// Stable comparison key used to avoid duplicate open obligations.
    #[must_use]
    pub fn dedupe_key(&self) -> String {
        let target = self.trigger.target.as_deref().unwrap_or("");
        format!(
            "{}|{}|{}|{}",
            self.kind,
            self.title,
            scope_key(&self.scope),
            target
        )
    }
}

fn scope_key(scope: &MemoryScope) -> String {
    match scope {
        MemoryScope::Global => "global".to_string(),
        MemoryScope::User => "user".to_string(),
        MemoryScope::Project { project_name, .. } => format!("project:{project_name}"),
        MemoryScope::Task { task_name, .. } => format!("task:{task_name}"),
        MemoryScope::Entity { entity_name, .. } => format!("entity:{entity_name}"),
        MemoryScope::Repository {
            remote_url,
            local_path,
            ..
        } => format!(
            "repository:{}",
            remote_url
                .as_deref()
                .or(local_path.as_deref())
                .unwrap_or("")
        ),
        MemoryScope::Session { session_id } => format!("session:{session_id}"),
        MemoryScope::Custom { name } => format!("custom:{name}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{EvidenceKind, Harness, ModelIdentity};

    fn writer() -> WriterProvenance {
        WriterProvenance::agent(Harness::Codex, ModelIdentity::new("openai", "gpt-5.5"))
    }

    #[test]
    fn obligation_resolve_closes_item() {
        let mut obligation = AgentObligation::new(
            AgentObligationKind::ToolFailureRecovery,
            "Recover failed tool call",
            "Inspect schema and retry.",
            MemoryScope::project("engram"),
            AgentObligationTrigger::new("tool_failure", "MCP call rejected parameters")
                .with_target("engram.memory"),
            writer(),
        )
        .with_required_resolution(AgentObligationResolutionKind::RetriedTool)
        .with_evidence(EvidenceRef::new(EvidenceKind::ToolCall, "engram.memory"));

        obligation.resolve(AgentObligationResolution::new(
            AgentObligationResolutionKind::RetriedTool,
            "Retried with structured evidence payload.",
            "agent",
        ));

        assert_eq!(obligation.status, AgentObligationStatus::Resolved);
        assert_eq!(
            obligation.resolution.as_ref().unwrap().kind,
            AgentObligationResolutionKind::RetriedTool
        );
    }
}
