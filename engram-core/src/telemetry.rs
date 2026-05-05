//! Brain harness telemetry types.
//!
//! These types describe retrieval traces and agent feedback. They are intended
//! to measure whether memory retrieval helped an agent, not just whether a tool
//! call succeeded.

use crate::id::Id;
use serde::{Deserialize, Serialize};
use std::fmt;
use time::OffsetDateTime;

/// Brain-harness operation being measured.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BrainHarnessOperation {
    /// Memory orientation context retrieval.
    Orient,
    /// Unified search.
    Search,
    /// Memory changes since a cursor.
    ChangesSince,
    /// Agent feedback submission.
    Feedback,
    /// Custom operation.
    Custom(String),
}

impl BrainHarnessOperation {
    /// Parse an operation from a string.
    #[must_use]
    pub fn parse(value: &str) -> Self {
        match value.to_lowercase().replace('-', "_").as_str() {
            "orient" => Self::Orient,
            "search" => Self::Search,
            "changes_since" | "changessince" => Self::ChangesSince,
            "feedback" | "agent_feedback" => Self::Feedback,
            other => Self::Custom(other.to_string()),
        }
    }
}

impl fmt::Display for BrainHarnessOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Orient => write!(f, "orient"),
            Self::Search => write!(f, "search"),
            Self::ChangesSince => write!(f, "changes_since"),
            Self::Feedback => write!(f, "feedback"),
            Self::Custom(value) => write!(f, "{value}"),
        }
    }
}

/// Agent intent for a brain-harness operation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BrainHarnessIntent {
    /// Resume prior project/session context.
    ResumeSession,
    /// Answer a user question.
    AnswerQuestion,
    /// Plan upcoming work.
    PlanWork,
    /// Implement a code or document change.
    ImplementChange,
    /// Debug an error or failure.
    DebugError,
    /// Verify a decision or claim.
    VerifyDecision,
    /// Follow a known user preference.
    FollowUserPreference,
    /// Prepare a handoff.
    PrepareHandoff,
    /// Review, update, or delete memory.
    ReviewMemory,
    /// Custom intent.
    Custom(String),
}

impl BrainHarnessIntent {
    /// Parse an intent from a string.
    #[must_use]
    pub fn parse(value: &str) -> Self {
        match value.to_lowercase().replace('-', "_").as_str() {
            "resume_session" | "resume" => Self::ResumeSession,
            "answer_question" | "answer" => Self::AnswerQuestion,
            "plan_work" | "plan" => Self::PlanWork,
            "implement_change" | "implement" => Self::ImplementChange,
            "debug_error" | "debug" => Self::DebugError,
            "verify_decision" | "verify" => Self::VerifyDecision,
            "follow_user_preference" | "preference" => Self::FollowUserPreference,
            "prepare_handoff" | "handoff" => Self::PrepareHandoff,
            "review_memory" | "memory_review" => Self::ReviewMemory,
            other => Self::Custom(other.to_string()),
        }
    }
}

impl fmt::Display for BrainHarnessIntent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ResumeSession => write!(f, "resume_session"),
            Self::AnswerQuestion => write!(f, "answer_question"),
            Self::PlanWork => write!(f, "plan_work"),
            Self::ImplementChange => write!(f, "implement_change"),
            Self::DebugError => write!(f, "debug_error"),
            Self::VerifyDecision => write!(f, "verify_decision"),
            Self::FollowUserPreference => write!(f, "follow_user_preference"),
            Self::PrepareHandoff => write!(f, "prepare_handoff"),
            Self::ReviewMemory => write!(f, "review_memory"),
            Self::Custom(value) => write!(f, "{value}"),
        }
    }
}

/// A single measured brain-harness operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainHarnessTrace {
    /// Trace ID returned to callers for later feedback.
    pub id: Id,
    /// Optional session ID associated with the operation.
    pub session_id: Option<Id>,
    /// Agent or harness label.
    pub agent: Option<String>,
    /// Operation being measured.
    pub operation: BrainHarnessOperation,
    /// Caller intent, when known.
    pub intent: Option<BrainHarnessIntent>,
    /// Query, prompt, or short operation context.
    pub query: Option<String>,
    /// Project scope, when known.
    pub project: Option<String>,
    /// Memory IDs returned by the operation.
    #[serde(default)]
    pub returned_memory_ids: Vec<Id>,
    /// Generic result IDs returned by the operation.
    #[serde(default)]
    pub returned_result_ids: Vec<String>,
    /// End-to-end operation latency in milliseconds.
    pub latency_ms: Option<u64>,
    /// Non-fatal warnings observed during the operation.
    #[serde(default)]
    pub warnings: Vec<String>,
    /// Creation timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

impl BrainHarnessTrace {
    /// Create a new operation trace.
    #[must_use]
    pub fn new(operation: BrainHarnessOperation) -> Self {
        Self {
            id: Id::new(),
            session_id: None,
            agent: None,
            operation,
            intent: None,
            query: None,
            project: None,
            returned_memory_ids: Vec::new(),
            returned_result_ids: Vec::new(),
            latency_ms: None,
            warnings: Vec::new(),
            created_at: OffsetDateTime::now_utc(),
        }
    }

    /// Set the session ID.
    #[must_use]
    pub const fn with_session(mut self, session_id: Option<Id>) -> Self {
        self.session_id = session_id;
        self
    }

    /// Set the agent label.
    #[must_use]
    pub fn with_agent(mut self, agent: Option<String>) -> Self {
        self.agent = agent;
        self
    }

    /// Set the intent.
    #[must_use]
    pub fn with_intent(mut self, intent: Option<BrainHarnessIntent>) -> Self {
        self.intent = intent;
        self
    }

    /// Set the query or prompt.
    #[must_use]
    pub fn with_query(mut self, query: Option<String>) -> Self {
        self.query = query;
        self
    }

    /// Set the project scope.
    #[must_use]
    pub fn with_project(mut self, project: Option<String>) -> Self {
        self.project = project;
        self
    }

    /// Set returned memory IDs.
    #[must_use]
    pub fn with_returned_memory_ids(mut self, ids: Vec<Id>) -> Self {
        self.returned_memory_ids = ids;
        self
    }

    /// Set returned generic result IDs.
    #[must_use]
    pub fn with_returned_result_ids(mut self, ids: Vec<String>) -> Self {
        self.returned_result_ids = ids;
        self
    }

    /// Set latency in milliseconds.
    #[must_use]
    pub const fn with_latency_ms(mut self, latency_ms: u64) -> Self {
        self.latency_ms = Some(latency_ms);
        self
    }

    /// Add a warning.
    #[must_use]
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }
}

/// Agent-reported feedback for a prior trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFeedback {
    /// Feedback ID.
    pub id: Id,
    /// Trace being evaluated.
    pub trace_id: Id,
    /// Optional session ID.
    pub session_id: Option<Id>,
    /// Agent or harness label.
    pub agent: Option<String>,
    /// Memory IDs the agent reports using.
    #[serde(default)]
    pub used_memory_ids: Vec<Id>,
    /// Memory IDs the agent reports rejecting.
    #[serde(default)]
    pub rejected_memory_ids: Vec<Id>,
    /// Generic result IDs the agent reports using.
    #[serde(default)]
    pub used_result_ids: Vec<String>,
    /// Generic result IDs the agent reports rejecting.
    #[serde(default)]
    pub rejected_result_ids: Vec<String>,
    /// Memory IDs believed stale.
    #[serde(default)]
    pub stale_memory_ids: Vec<Id>,
    /// Memory IDs believed to have the wrong scope.
    #[serde(default)]
    pub wrong_scope_memory_ids: Vec<Id>,
    /// Context the agent expected but did not receive.
    pub missing_context: Option<String>,
    /// Usefulness score, 1-5.
    pub usefulness_score: Option<u8>,
    /// Correctness score, 1-5.
    pub correctness_score: Option<u8>,
    /// Noise score, 1-5.
    pub noise_score: Option<u8>,
    /// Suggested memory changes from the agent.
    pub suggested_memory_changes: Option<String>,
    /// Free-form feedback note.
    pub note: Option<String>,
    /// Creation timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

impl AgentFeedback {
    /// Create feedback for a trace.
    #[must_use]
    pub fn new(trace_id: Id) -> Self {
        Self {
            id: Id::new(),
            trace_id,
            session_id: None,
            agent: None,
            used_memory_ids: Vec::new(),
            rejected_memory_ids: Vec::new(),
            used_result_ids: Vec::new(),
            rejected_result_ids: Vec::new(),
            stale_memory_ids: Vec::new(),
            wrong_scope_memory_ids: Vec::new(),
            missing_context: None,
            usefulness_score: None,
            correctness_score: None,
            noise_score: None,
            suggested_memory_changes: None,
            note: None,
            created_at: OffsetDateTime::now_utc(),
        }
    }
}

/// Aggregate telemetry statistics grouped by intent.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IntentTelemetryStats {
    /// Intent label.
    pub intent: String,
    /// Number of traces.
    pub trace_count: usize,
    /// Number of feedback records linked to traces for this intent.
    pub feedback_count: usize,
    /// Average latency, when any traces recorded latency.
    pub avg_latency_ms: Option<f32>,
    /// Average usefulness score, when feedback included one.
    pub avg_usefulness_score: Option<f32>,
    /// Average correctness score, when feedback included one.
    pub avg_correctness_score: Option<f32>,
    /// Average noise score, when feedback included one.
    pub avg_noise_score: Option<f32>,
    /// Feedback records that reported missing context.
    pub missing_context_count: usize,
    /// Number of memory IDs reported used.
    pub used_memory_count: usize,
    /// Number of memory IDs reported rejected.
    pub rejected_memory_count: usize,
}
