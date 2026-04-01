//! Coordination service for Layer 5: Session Coordination.
//!
//! Provides business logic for managing parallel session awareness,
//! conflict detection, and session lifecycle.

use crate::error::{IndexError, IndexResult};
use engram_core::coordination::{ActiveSession, ConflictInfo};
use engram_core::id::Id;
use engram_store::{CoordinationRepo, CoordinationStats, Db};
use tracing::{debug, info, warn};

/// Default timeout for stale sessions (in minutes).
const DEFAULT_STALE_TIMEOUT_MINUTES: i64 = 30;

/// Service for session coordination management.
#[derive(Clone)]
pub struct CoordinationService {
    repo: CoordinationRepo,
    /// Timeout in minutes after which sessions are considered stale.
    stale_timeout_minutes: i64,
}

impl CoordinationService {
    /// Create a new coordination service.
    pub fn new(db: Db) -> Self {
        Self {
            repo: CoordinationRepo::new(db),
            stale_timeout_minutes: DEFAULT_STALE_TIMEOUT_MINUTES,
        }
    }

    /// Create with custom stale timeout.
    pub fn with_stale_timeout(db: Db, timeout_minutes: i64) -> Self {
        Self {
            repo: CoordinationRepo::new(db),
            stale_timeout_minutes: timeout_minutes,
        }
    }

    /// Initialize the coordination schema.
    pub async fn init(&self) -> IndexResult<()> {
        self.repo.init_schema().await?;
        Ok(())
    }

    // =========================================================================
    // Session Registration
    // =========================================================================

    /// Register a session for coordination.
    ///
    /// This makes the session visible to other concurrent sessions for
    /// conflict detection.
    pub async fn register(
        &self,
        session_id: &Id,
        agent: &str,
        project: &str,
        goal: &str,
    ) -> IndexResult<ActiveSession> {
        info!("Registering session {} for coordination", session_id);

        let session = ActiveSession::new(session_id.clone(), agent, project, goal);
        self.repo.register(&session).await?;

        Ok(session)
    }

    /// Register with components.
    pub async fn register_with_components(
        &self,
        session_id: &Id,
        agent: &str,
        project: &str,
        goal: &str,
        components: Vec<String>,
    ) -> IndexResult<ActiveSession> {
        info!(
            "Registering session {} with components {:?}",
            session_id, components
        );

        let session = ActiveSession::new(session_id.clone(), agent, project, goal)
            .with_components(components);
        self.repo.register(&session).await?;

        Ok(session)
    }

    /// Unregister a session (when ending).
    pub async fn unregister(&self, session_id: &Id) -> IndexResult<()> {
        info!("Unregistering session {}", session_id);
        self.repo.unregister(session_id).await?;
        Ok(())
    }

    // =========================================================================
    // Heartbeat & Updates
    // =========================================================================

    /// Send heartbeat for a session (keeps it active).
    pub async fn heartbeat(&self, session_id: &Id) -> IndexResult<()> {
        debug!("Heartbeat for session {}", session_id);
        self.repo.heartbeat(session_id).await?;
        Ok(())
    }

    /// Update the file currently being edited.
    ///
    /// Returns any conflicts (other sessions editing the same file).
    pub async fn set_current_file(
        &self,
        session_id: &Id,
        file: Option<&str>,
    ) -> IndexResult<Vec<ConflictInfo>> {
        debug!(
            "Setting current file for session {}: {:?}",
            session_id, file
        );

        self.repo.set_current_file(session_id, file).await?;

        // Check for file conflicts
        if let Some(f) = file {
            Ok(self.repo.find_file_conflicts(session_id, f).await?)
        } else {
            Ok(Vec::new())
        }
    }

    /// Update the components being worked on.
    ///
    /// Returns any conflicts (other sessions with overlapping components).
    pub async fn set_components(
        &self,
        session_id: &Id,
        components: &[String],
    ) -> IndexResult<Vec<ConflictInfo>> {
        debug!(
            "Setting components for session {}: {:?}",
            session_id, components
        );

        self.repo.set_components(session_id, components).await?;

        // Check for component conflicts
        Ok(self.repo.find_conflicts(session_id, components).await?)
    }

    // =========================================================================
    // Queries
    // =========================================================================

    /// Get an active session by ID.
    pub async fn get(&self, session_id: &Id) -> IndexResult<Option<ActiveSession>> {
        Ok(self.repo.get(session_id).await?)
    }

    /// List all active sessions.
    pub async fn list_active(&self) -> IndexResult<Vec<ActiveSession>> {
        Ok(self.repo.list_active().await?)
    }

    /// List active sessions for a project.
    pub async fn list_for_project(&self, project: &str) -> IndexResult<Vec<ActiveSession>> {
        Ok(self.repo.list_for_project(project).await?)
    }

    // =========================================================================
    // Conflict Detection
    // =========================================================================

    /// Check for potential conflicts with a session's components.
    pub async fn check_conflicts(&self, session_id: &Id) -> IndexResult<Vec<ConflictInfo>> {
        debug!("Checking conflicts for session {}", session_id);

        // Get the session
        let session = self.repo.get(session_id).await?.ok_or_else(|| {
            IndexError::NotFound(format!("Session not registered: {}", session_id))
        })?;

        // Find conflicts
        Ok(self
            .repo
            .find_conflicts(session_id, &session.components)
            .await?)
    }

    /// Check for file conflicts.
    pub async fn check_file_conflicts(
        &self,
        session_id: &Id,
        file: &str,
    ) -> IndexResult<Vec<ConflictInfo>> {
        debug!(
            "Checking file conflicts for {} editing {}",
            session_id, file
        );
        Ok(self.repo.find_file_conflicts(session_id, file).await?)
    }

    // =========================================================================
    // Maintenance
    // =========================================================================

    /// Clean up stale sessions.
    ///
    /// Sessions that haven't sent a heartbeat in `stale_timeout_minutes` are removed.
    pub async fn cleanup_stale(&self) -> IndexResult<usize> {
        info!(
            "Cleaning up stale sessions (timeout: {} minutes)",
            self.stale_timeout_minutes
        );
        let count = self.repo.cleanup_stale(self.stale_timeout_minutes).await?;

        if count > 0 {
            warn!("Cleaned up {} stale sessions", count);
        }

        Ok(count)
    }

    /// Clean up with custom timeout.
    pub async fn cleanup_stale_with_timeout(&self, timeout_minutes: i64) -> IndexResult<usize> {
        info!(
            "Cleaning up stale sessions (timeout: {} minutes)",
            timeout_minutes
        );
        let count = self.repo.cleanup_stale(timeout_minutes).await?;

        if count > 0 {
            warn!("Cleaned up {} stale sessions", count);
        }

        Ok(count)
    }

    // =========================================================================
    // Statistics
    // =========================================================================

    /// Get coordination statistics.
    pub async fn stats(&self) -> IndexResult<CoordinationStats> {
        Ok(self.repo.stats().await?)
    }
}

/// Convenience struct for conflict check results.
#[derive(Debug, Clone)]
pub struct ConflictCheckResult {
    /// Whether there are any conflicts.
    pub has_conflicts: bool,
    /// Component-based conflicts.
    pub component_conflicts: Vec<ConflictInfo>,
    /// File-based conflicts.
    pub file_conflicts: Vec<ConflictInfo>,
}

impl ConflictCheckResult {
    /// Create an empty result (no conflicts).
    pub fn none() -> Self {
        Self {
            has_conflicts: false,
            component_conflicts: Vec::new(),
            file_conflicts: Vec::new(),
        }
    }

    /// Create from component conflicts.
    pub fn from_components(conflicts: Vec<ConflictInfo>) -> Self {
        Self {
            has_conflicts: !conflicts.is_empty(),
            component_conflicts: conflicts,
            file_conflicts: Vec::new(),
        }
    }

    /// Create from file conflicts.
    pub fn from_files(conflicts: Vec<ConflictInfo>) -> Self {
        Self {
            has_conflicts: !conflicts.is_empty(),
            component_conflicts: Vec::new(),
            file_conflicts: conflicts,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conflict_check_result_none() {
        let result = ConflictCheckResult::none();
        assert!(!result.has_conflicts);
        assert!(result.component_conflicts.is_empty());
        assert!(result.file_conflicts.is_empty());
    }
}
