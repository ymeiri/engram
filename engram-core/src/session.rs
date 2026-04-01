//! Session history types (Layer 2).
//!
//! Sessions track what happened during agent interactions:
//! decisions made, events logged, rationale captured.

use crate::id::Id;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Status of a session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    /// Session is currently active.
    #[default]
    Active,
    /// Session completed successfully.
    Completed,
    /// Session was abandoned.
    Abandoned,
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Completed => write!(f, "completed"),
            Self::Abandoned => write!(f, "abandoned"),
        }
    }
}

impl SessionStatus {
    /// Parse from string.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "active" => Self::Active,
            "completed" => Self::Completed,
            "abandoned" => Self::Abandoned,
            _ => Self::Active, // Default to active
        }
    }
}

/// A coding session with an AI agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique identifier.
    pub id: Id,

    /// Project being worked on.
    pub project: Option<String>,

    /// Agent being used (e.g., "claude-code", "cursor").
    pub agent: Option<String>,

    /// Goal of the session.
    pub goal: Option<String>,

    /// Session status.
    pub status: SessionStatus,

    /// Summary of what was accomplished.
    pub summary: Option<String>,

    /// Key decisions made during the session.
    #[serde(default)]
    pub key_decisions: Vec<String>,

    /// Start timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub started_at: OffsetDateTime,

    /// End timestamp.
    #[serde(with = "time::serde::rfc3339::option")]
    pub ended_at: Option<OffsetDateTime>,
}

impl Session {
    /// Create a new active session.
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            project: None,
            agent: None,
            goal: None,
            status: SessionStatus::Active,
            summary: None,
            key_decisions: Vec::new(),
            started_at: OffsetDateTime::now_utc(),
            ended_at: None,
        }
    }

    /// Set the project.
    #[must_use]
    pub fn with_project(mut self, project: impl Into<String>) -> Self {
        self.project = Some(project.into());
        self
    }

    /// Set the agent.
    #[must_use]
    pub fn with_agent(mut self, agent: impl Into<String>) -> Self {
        self.agent = Some(agent.into());
        self
    }

    /// Set the goal.
    #[must_use]
    pub fn with_goal(mut self, goal: impl Into<String>) -> Self {
        self.goal = Some(goal.into());
        self
    }

    /// End the session.
    pub fn end(&mut self, summary: Option<String>) {
        self.status = SessionStatus::Completed;
        self.summary = summary;
        self.ended_at = Some(OffsetDateTime::now_utc());
    }

    /// Abandon the session.
    pub fn abandon(&mut self) {
        self.status = SessionStatus::Abandoned;
        self.ended_at = Some(OffsetDateTime::now_utc());
    }

    /// Add a key decision.
    pub fn add_decision(&mut self, decision: impl Into<String>) {
        self.key_decisions.push(decision.into());
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

/// Type of event in a session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// A decision was made.
    Decision,
    /// A command was executed.
    Command,
    /// A file was changed.
    FileChange,
    /// A tool was used.
    ToolUse,
    /// An error occurred.
    Error,
    /// A milestone was reached.
    Milestone,
    /// An observation or note.
    Observation,
    /// Custom event type.
    Custom(String),
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Decision => write!(f, "decision"),
            Self::Command => write!(f, "command"),
            Self::FileChange => write!(f, "file_change"),
            Self::ToolUse => write!(f, "tool_use"),
            Self::Error => write!(f, "error"),
            Self::Milestone => write!(f, "milestone"),
            Self::Observation => write!(f, "observation"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl EventType {
    /// Parse from string.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "decision" => Self::Decision,
            "command" => Self::Command,
            "file_change" | "filechange" => Self::FileChange,
            "tool_use" | "tooluse" => Self::ToolUse,
            "error" => Self::Error,
            "milestone" => Self::Milestone,
            "observation" | "note" => Self::Observation,
            other => Self::Custom(other.to_string()),
        }
    }
}

/// An event within a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Unique identifier.
    pub id: Id,

    /// Session this event belongs to.
    pub session_id: Id,

    /// Type of event.
    pub event_type: EventType,

    /// Who triggered the event (e.g., "user", "agent", "system").
    pub actor: String,

    /// Event content (the main message/description).
    pub content: String,

    /// Additional context or rationale.
    pub context: Option<String>,

    /// Source of this event (e.g., file path, tool name).
    pub source: Option<String>,

    /// Entity IDs mentioned in this event (for cross-referencing with Layer 1).
    #[serde(default)]
    pub entities_mentioned: Vec<Id>,

    /// Timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
}

impl Event {
    /// Create a new event.
    #[must_use]
    pub fn new(
        session_id: Id,
        event_type: EventType,
        actor: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            id: Id::new(),
            session_id,
            event_type,
            actor: actor.into(),
            content: content.into(),
            context: None,
            source: None,
            entities_mentioned: Vec::new(),
            timestamp: OffsetDateTime::now_utc(),
        }
    }

    /// Set the context.
    #[must_use]
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Set the source.
    #[must_use]
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Add a mentioned entity.
    #[must_use]
    pub fn with_entity(mut self, entity_id: Id) -> Self {
        self.entities_mentioned.push(entity_id);
        self
    }
}

/// A decision event with structured data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    /// Summary of the decision.
    pub summary: String,

    /// Rationale for the decision.
    pub rationale: Option<String>,

    /// Files affected by the decision.
    #[serde(default)]
    pub files: Vec<String>,

    /// Alternatives considered.
    #[serde(default)]
    pub alternatives: Vec<String>,
}

impl Decision {
    /// Create a new decision.
    #[must_use]
    pub fn new(summary: impl Into<String>) -> Self {
        Self {
            summary: summary.into(),
            rationale: None,
            files: Vec::new(),
            alternatives: Vec::new(),
        }
    }

    /// Set the rationale.
    #[must_use]
    pub fn with_rationale(mut self, rationale: impl Into<String>) -> Self {
        self.rationale = Some(rationale.into());
        self
    }

    /// Convert to event content string.
    #[must_use]
    pub fn to_content(&self) -> String {
        if let Some(ref rationale) = self.rationale {
            format!("{} (rationale: {})", self.summary, rationale)
        } else {
            self.summary.clone()
        }
    }
}

/// Statistics about sessions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionStats {
    /// Total number of sessions.
    pub total_sessions: usize,
    /// Number of active sessions.
    pub active_sessions: usize,
    /// Number of completed sessions.
    pub completed_sessions: usize,
    /// Number of abandoned sessions.
    pub abandoned_sessions: usize,
    /// Total number of events.
    pub total_events: usize,
    /// Events by type.
    pub events_by_type: std::collections::HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::new()
            .with_project("web-ui")
            .with_agent("claude-code")
            .with_goal("Implement authentication");

        assert_eq!(session.project, Some("web-ui".to_string()));
        assert_eq!(session.agent, Some("claude-code".to_string()));
        assert_eq!(session.status, SessionStatus::Active);
    }

    #[test]
    fn test_session_end() {
        let mut session = Session::new();
        session.end(Some("Completed auth implementation".to_string()));

        assert_eq!(session.status, SessionStatus::Completed);
        assert!(session.ended_at.is_some());
    }

    #[test]
    fn test_event_creation() {
        let session_id = Id::new();
        let event = Event::new(
            session_id,
            EventType::Decision,
            "agent",
            "Chose OAuth over API keys",
        )
        .with_context("Need better security for user data")
        .with_source("auth-service");

        assert_eq!(event.event_type, EventType::Decision);
        assert_eq!(event.content, "Chose OAuth over API keys");
        assert_eq!(
            event.context,
            Some("Need better security for user data".to_string())
        );
    }

    #[test]
    fn test_decision_creation() {
        let decision = Decision::new("Use OAuth instead of API keys")
            .with_rationale("OAuth provides better security and user experience");

        assert_eq!(decision.summary, "Use OAuth instead of API keys");
        assert!(decision.rationale.is_some());
        assert!(decision.to_content().contains("OAuth"));
    }

    #[test]
    fn test_event_type_parse() {
        assert_eq!(EventType::parse("decision"), EventType::Decision);
        assert_eq!(EventType::parse("DECISION"), EventType::Decision);
        assert_eq!(EventType::parse("observation"), EventType::Observation);
        assert_eq!(EventType::parse("note"), EventType::Observation);
        assert!(matches!(
            EventType::parse("custom_type"),
            EventType::Custom(_)
        ));
    }

    #[test]
    fn test_session_status_parse() {
        assert_eq!(SessionStatus::parse("active"), SessionStatus::Active);
        assert_eq!(SessionStatus::parse("completed"), SessionStatus::Completed);
        assert_eq!(SessionStatus::parse("abandoned"), SessionStatus::Abandoned);
    }
}
