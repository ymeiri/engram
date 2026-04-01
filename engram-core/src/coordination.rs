//! Session coordination types (Layer 5).
//!
//! Enables awareness between parallel AI coding sessions.

use crate::id::Id;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// An active session registration for coordination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveSession {
    /// Session ID.
    pub session_id: Id,

    /// Agent being used.
    pub agent: String,

    /// Project being worked on.
    pub project: String,

    /// Goal of the session.
    pub goal: String,

    /// Components being touched.
    pub components: Vec<String>,

    /// Current file being edited.
    pub current_file: Option<String>,

    /// When the session started.
    #[serde(with = "time::serde::rfc3339")]
    pub started_at: OffsetDateTime,

    /// Last heartbeat timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub last_heartbeat: OffsetDateTime,
}

impl ActiveSession {
    /// Create a new active session registration.
    #[must_use]
    pub fn new(
        session_id: Id,
        agent: impl Into<String>,
        project: impl Into<String>,
        goal: impl Into<String>,
    ) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            session_id,
            agent: agent.into(),
            project: project.into(),
            goal: goal.into(),
            components: Vec::new(),
            current_file: None,
            started_at: now,
            last_heartbeat: now,
        }
    }

    /// Add components being worked on.
    #[must_use]
    pub fn with_components(mut self, components: Vec<String>) -> Self {
        self.components = components;
        self
    }

    /// Update heartbeat to now.
    pub fn heartbeat(&mut self) {
        self.last_heartbeat = OffsetDateTime::now_utc();
    }

    /// Set current file.
    pub fn set_current_file(&mut self, file: Option<String>) {
        self.current_file = file;
        self.heartbeat();
    }

    /// Check if the session has timed out.
    #[must_use]
    pub fn is_stale(&self, timeout_minutes: i64) -> bool {
        let now = OffsetDateTime::now_utc();
        let minutes_since = (now - self.last_heartbeat).whole_minutes();
        minutes_since >= timeout_minutes
    }
}

/// Information about a potential conflict.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictInfo {
    /// The other session.
    pub other_session_id: Id,

    /// Agent of the other session.
    pub other_agent: String,

    /// Goal of the other session.
    pub other_goal: String,

    /// Overlapping components.
    pub overlapping_components: Vec<String>,

    /// Current file of the other session (if any).
    pub other_current_file: Option<String>,
}

impl ConflictInfo {
    /// Create conflict info from an active session and overlapping components.
    #[must_use]
    pub fn from_session(session: &ActiveSession, overlapping: Vec<String>) -> Self {
        Self {
            other_session_id: session.session_id,
            other_agent: session.agent.clone(),
            other_goal: session.goal.clone(),
            overlapping_components: overlapping,
            other_current_file: session.current_file.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_session_creation() {
        let session_id = Id::new();
        let active = ActiveSession::new(
            session_id,
            "claude-code",
            "web-ui",
            "Implement authentication",
        )
        .with_components(vec!["auth-service".to_string(), "user-api".to_string()]);

        assert_eq!(active.agent, "claude-code");
        assert_eq!(active.components.len(), 2);
    }

    #[test]
    fn test_heartbeat() {
        let session_id = Id::new();
        let mut active = ActiveSession::new(session_id, "claude-code", "web-ui", "Test");

        let initial_heartbeat = active.last_heartbeat;
        std::thread::sleep(std::time::Duration::from_millis(10));
        active.heartbeat();

        assert!(active.last_heartbeat > initial_heartbeat);
    }

    #[test]
    fn test_is_stale() {
        let session_id = Id::new();
        let active = ActiveSession::new(session_id, "claude-code", "web-ui", "Test");

        // Just created, should not be stale
        assert!(!active.is_stale(5));
    }
}
