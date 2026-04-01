//! Session service for Layer 2: Session History.
//!
//! Provides business logic for managing coding sessions:
//! tracking sessions, logging events, and searching history.

use crate::error::{IndexError, IndexResult};
use engram_core::id::Id;
use engram_core::session::{Event, EventType, Session, SessionStats, SessionStatus};
use engram_store::{Db, SessionRepo};
use tracing::info;

/// Service for session history management.
#[derive(Clone)]
pub struct SessionService {
    repo: SessionRepo,
}

impl SessionService {
    /// Create a new session service.
    pub fn new(db: Db) -> Self {
        Self {
            repo: SessionRepo::new(db),
        }
    }

    /// Initialize the session schema.
    pub async fn init(&self) -> IndexResult<()> {
        self.repo.init_schema().await?;
        Ok(())
    }

    // =========================================================================
    // Session Management
    // =========================================================================

    /// Start a new session.
    pub async fn start_session(
        &self,
        agent: Option<&str>,
        project: Option<&str>,
        goal: Option<&str>,
    ) -> IndexResult<Session> {
        info!(
            "Starting new session (agent: {:?}, project: {:?})",
            agent, project
        );

        let mut session = Session::new();
        if let Some(a) = agent {
            session = session.with_agent(a);
        }
        if let Some(p) = project {
            session = session.with_project(p);
        }
        if let Some(g) = goal {
            session = session.with_goal(g);
        }

        self.repo.save_session(&session).await?;

        info!("Started session: {}", session.id);
        Ok(session)
    }

    /// Get a session by ID.
    pub async fn get_session(&self, id: &Id) -> IndexResult<Option<Session>> {
        Ok(self.repo.get_session(id).await?)
    }

    /// End a session.
    pub async fn end_session(&self, id: &Id, summary: Option<&str>) -> IndexResult<()> {
        info!("Ending session: {}", id);

        // Verify session exists
        let session = self
            .repo
            .get_session(id)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Session not found: {}", id)))?;

        if session.status != SessionStatus::Active {
            return Err(IndexError::InvalidState(format!(
                "Session {} is not active (status: {})",
                id, session.status
            )));
        }

        self.repo
            .end_session(id, SessionStatus::Completed, summary.map(String::from))
            .await?;
        info!("Ended session: {}", id);
        Ok(())
    }

    /// Abandon a session.
    pub async fn abandon_session(&self, id: &Id) -> IndexResult<()> {
        info!("Abandoning session: {}", id);
        self.repo
            .end_session(id, SessionStatus::Abandoned, None)
            .await?;
        Ok(())
    }

    /// List sessions with optional filters.
    pub async fn list_sessions(
        &self,
        status: Option<&SessionStatus>,
        agent: Option<&str>,
        project: Option<&str>,
        limit: Option<usize>,
    ) -> IndexResult<Vec<Session>> {
        Ok(self
            .repo
            .list_sessions(status, agent, project, limit)
            .await?)
    }

    /// Get active sessions.
    pub async fn get_active_sessions(&self, project: Option<&str>) -> IndexResult<Vec<Session>> {
        Ok(self.repo.get_active_sessions(project).await?)
    }

    /// Get the most recent active session (optionally for a project).
    pub async fn get_current_session(&self, project: Option<&str>) -> IndexResult<Option<Session>> {
        let active = self.get_active_sessions(project).await?;
        Ok(active.into_iter().next())
    }

    /// Get a session with all its events.
    pub async fn get_session_with_events(&self, id: &Id) -> IndexResult<(Session, Vec<Event>)> {
        let session = self
            .repo
            .get_session(id)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Session not found: {}", id)))?;
        let events = self.repo.get_events(id).await?;
        Ok((session, events))
    }

    /// Delete a session and its events.
    pub async fn delete_session(&self, id: &Id) -> IndexResult<()> {
        info!("Deleting session: {}", id);
        self.repo.delete_session(id).await?;
        Ok(())
    }

    // =========================================================================
    // Event Logging
    // =========================================================================

    /// Log an event to a session.
    pub async fn log_event(
        &self,
        session_id: &Id,
        event_type: EventType,
        content: &str,
        context: Option<&str>,
        source: Option<&str>,
    ) -> IndexResult<Event> {
        info!("Logging {:?} event to session {}", event_type, session_id);

        // Verify session exists and is active
        let session =
            self.repo.get_session(session_id).await?.ok_or_else(|| {
                IndexError::NotFound(format!("Session not found: {}", session_id))
            })?;

        if session.status != SessionStatus::Active {
            return Err(IndexError::InvalidState(format!(
                "Cannot log events to session {} (status: {})",
                session_id, session.status
            )));
        }

        let mut event = Event::new(session_id.clone(), event_type, "agent", content);
        if let Some(ctx) = context {
            event = event.with_context(ctx);
        }
        if let Some(src) = source {
            event = event.with_source(src);
        }

        self.repo.add_event(&event).await?;
        Ok(event)
    }

    /// Log a decision event.
    pub async fn log_decision(
        &self,
        session_id: &Id,
        decision: &str,
        rationale: Option<&str>,
    ) -> IndexResult<Event> {
        self.log_event(session_id, EventType::Decision, decision, rationale, None)
            .await
    }

    /// Log an observation event.
    pub async fn log_observation(
        &self,
        session_id: &Id,
        observation: &str,
        source: Option<&str>,
    ) -> IndexResult<Event> {
        self.log_event(
            session_id,
            EventType::Observation,
            observation,
            None,
            source,
        )
        .await
    }

    /// Log an error event.
    pub async fn log_error(
        &self,
        session_id: &Id,
        error: &str,
        context: Option<&str>,
    ) -> IndexResult<Event> {
        self.log_event(session_id, EventType::Error, error, context, None)
            .await
    }

    /// Get events for a session.
    pub async fn get_events(&self, session_id: &Id) -> IndexResult<Vec<Event>> {
        Ok(self.repo.get_events(session_id).await?)
    }

    // =========================================================================
    // Search
    // =========================================================================

    /// Search events by content (across all sessions).
    pub async fn search_events(
        &self,
        query: &str,
        limit: Option<usize>,
    ) -> IndexResult<Vec<Event>> {
        Ok(self.repo.search_events(query, limit).await?)
    }

    /// Get events by type (across all sessions).
    pub async fn get_events_by_type(
        &self,
        event_type: &EventType,
        limit: Option<usize>,
    ) -> IndexResult<Vec<Event>> {
        Ok(self.repo.get_events_by_type(event_type, limit).await?)
    }

    /// Search for decisions across all sessions.
    pub async fn search_decisions(
        &self,
        query: &str,
        limit: Option<usize>,
    ) -> IndexResult<Vec<Event>> {
        // First get all decision events
        let decisions = self
            .repo
            .get_events_by_type(&EventType::Decision, None)
            .await?;

        // Filter by query
        let query_lower = query.to_lowercase();
        let filtered: Vec<Event> = decisions
            .into_iter()
            .filter(|e| {
                e.content.to_lowercase().contains(&query_lower)
                    || e.context
                        .as_ref()
                        .map(|c| c.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
            })
            .take(limit.unwrap_or(50))
            .collect();

        Ok(filtered)
    }

    // =========================================================================
    // Statistics
    // =========================================================================

    /// Get session statistics.
    pub async fn stats(&self) -> IndexResult<SessionStats> {
        Ok(self.repo.stats().await?)
    }
}
