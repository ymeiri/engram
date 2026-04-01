//! Coordination repository for Layer 5: Session Coordination.
//!
//! Handles persistence of ActiveSession registrations for parallel session awareness.
//! Provides queries for conflict detection between concurrent sessions.

use crate::error::{StoreError, StoreResult};
use crate::Db;
use engram_core::coordination::{ActiveSession, ConflictInfo};
use engram_core::id::Id;
use serde::Deserialize;
use tracing::{debug, info};

/// SurrealDB datetime representation (handles both string and native formats).
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum SurrealDateTime {
    /// ISO 8601 string format
    String(String),
    /// Native SurrealDB datetime format (array of integers)
    Native(serde_json::Value),
}

impl SurrealDateTime {
    fn to_offset_datetime(&self) -> StoreResult<time::OffsetDateTime> {
        match self {
            SurrealDateTime::String(s) => {
                time::OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339)
                    .map_err(|e| StoreError::Deserialization(format!("Invalid datetime: {}", e)))
            }
            SurrealDateTime::Native(v) => {
                if let Some(arr) = v.as_array() {
                    if arr.len() >= 6 {
                        let year = arr[0].as_i64().unwrap_or(2000) as i32;
                        let month = arr[1].as_i64().unwrap_or(1) as u8;
                        let day = arr[2].as_i64().unwrap_or(1) as u8;
                        let hour = arr[3].as_i64().unwrap_or(0) as u8;
                        let min = arr[4].as_i64().unwrap_or(0) as u8;
                        let sec = arr[5].as_i64().unwrap_or(0) as u8;

                        let date = time::Date::from_calendar_date(
                            year,
                            time::Month::try_from(month).unwrap_or(time::Month::January),
                            day,
                        )
                        .unwrap_or(
                            time::Date::from_calendar_date(2000, time::Month::January, 1).unwrap(),
                        );

                        let time_val =
                            time::Time::from_hms(hour, min, sec).unwrap_or(time::Time::MIDNIGHT);

                        return Ok(time::OffsetDateTime::new_utc(date, time_val));
                    }
                }
                Ok(time::OffsetDateTime::now_utc())
            }
        }
    }
}

/// Active session record from SurrealDB.
#[derive(Debug, Deserialize)]
struct ActiveSessionRecord {
    session_id: String,
    agent: String,
    project: String,
    goal: String,
    #[serde(default)]
    components: Vec<String>,
    current_file: Option<String>,
    started_at: SurrealDateTime,
    last_heartbeat: SurrealDateTime,
}

/// Count result for stats queries.
#[derive(Debug, Deserialize)]
struct CountResult {
    count: u64,
}

/// Table name for coordination storage.
const TABLE_ACTIVE_SESSION: &str = "active_session";

/// Repository for session coordination operations.
#[derive(Clone)]
pub struct CoordinationRepo {
    db: Db,
}

impl CoordinationRepo {
    /// Create a new coordination repository.
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Initialize the coordination schema.
    pub async fn init_schema(&self) -> StoreResult<()> {
        info!("Initializing coordination schema (Layer 5)");

        // Active session table - sessions use their session_id as the record ID
        self.db
            .query(format!(
                r#"
                DEFINE TABLE IF NOT EXISTS {TABLE_ACTIVE_SESSION} SCHEMALESS;
                DEFINE INDEX IF NOT EXISTS idx_active_project ON {TABLE_ACTIVE_SESSION} FIELDS project;
                DEFINE INDEX IF NOT EXISTS idx_active_agent ON {TABLE_ACTIVE_SESSION} FIELDS agent;
                DEFINE INDEX IF NOT EXISTS idx_active_heartbeat ON {TABLE_ACTIVE_SESSION} FIELDS last_heartbeat;
                "#
            ))
            .await?;

        info!("Coordination schema initialized");
        Ok(())
    }

    // =========================================================================
    // Active Session Operations
    // =========================================================================

    /// Register an active session.
    pub async fn register(&self, session: &ActiveSession) -> StoreResult<()> {
        debug!(
            "Registering active session: {} ({})",
            session.session_id, session.agent
        );

        self.db
            .query(
                r#"
                UPSERT type::thing("active_session", $id) SET
                    session_id = $session_id,
                    agent = $agent,
                    project = $project,
                    goal = $goal,
                    components = $components,
                    current_file = $current_file,
                    started_at = $started_at,
                    last_heartbeat = $last_heartbeat
            "#,
            )
            .bind(("id", session.session_id.to_string()))
            .bind(("session_id", session.session_id.to_string()))
            .bind(("agent", session.agent.clone()))
            .bind(("project", session.project.clone()))
            .bind(("goal", session.goal.clone()))
            .bind(("components", session.components.clone()))
            .bind(("current_file", session.current_file.clone()))
            .bind((
                "started_at",
                session
                    .started_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            ))
            .bind((
                "last_heartbeat",
                session
                    .last_heartbeat
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            ))
            .await?;

        Ok(())
    }

    /// Unregister (deactivate) a session.
    pub async fn unregister(&self, session_id: &Id) -> StoreResult<()> {
        debug!("Unregistering session: {}", session_id);

        self.db
            .query("DELETE type::thing('active_session', $id)")
            .bind(("id", session_id.to_string()))
            .await?;

        Ok(())
    }

    /// Update heartbeat for a session.
    pub async fn heartbeat(&self, session_id: &Id) -> StoreResult<()> {
        debug!("Heartbeat for session: {}", session_id);

        let now = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap();

        self.db
            .query(
                r#"
                UPDATE type::thing("active_session", $id) SET
                    last_heartbeat = $last_heartbeat
            "#,
            )
            .bind(("id", session_id.to_string()))
            .bind(("last_heartbeat", now))
            .await?;

        Ok(())
    }

    /// Update current file for a session.
    pub async fn set_current_file(&self, session_id: &Id, file: Option<&str>) -> StoreResult<()> {
        debug!(
            "Setting current file for session {}: {:?}",
            session_id, file
        );

        let now = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap();

        self.db
            .query(
                r#"
                UPDATE type::thing("active_session", $id) SET
                    current_file = $current_file,
                    last_heartbeat = $last_heartbeat
            "#,
            )
            .bind(("id", session_id.to_string()))
            .bind(("current_file", file.map(|s| s.to_string())))
            .bind(("last_heartbeat", now))
            .await?;

        Ok(())
    }

    /// Update components for a session.
    pub async fn set_components(&self, session_id: &Id, components: &[String]) -> StoreResult<()> {
        debug!(
            "Setting components for session {}: {:?}",
            session_id, components
        );

        let now = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap();

        self.db
            .query(
                r#"
                UPDATE type::thing("active_session", $id) SET
                    components = $components,
                    last_heartbeat = $last_heartbeat
            "#,
            )
            .bind(("id", session_id.to_string()))
            .bind(("components", components.to_vec()))
            .bind(("last_heartbeat", now))
            .await?;

        Ok(())
    }

    /// Get an active session by ID.
    pub async fn get(&self, session_id: &Id) -> StoreResult<Option<ActiveSession>> {
        debug!("Getting active session: {}", session_id);

        let mut result = self
            .db
            .query("SELECT * FROM type::thing('active_session', $id)")
            .bind(("id", session_id.to_string()))
            .await?;

        let records: Vec<ActiveSessionRecord> = result.take(0)?;

        if let Some(record) = records.into_iter().next() {
            Ok(Some(self.record_to_session(record)?))
        } else {
            Ok(None)
        }
    }

    /// List all active sessions.
    pub async fn list_active(&self) -> StoreResult<Vec<ActiveSession>> {
        debug!("Listing all active sessions");

        let mut result = self
            .db
            .query(format!(
                "SELECT * FROM {} ORDER BY last_heartbeat DESC",
                TABLE_ACTIVE_SESSION
            ))
            .await?;

        let records: Vec<ActiveSessionRecord> = result.take(0)?;

        let mut sessions = Vec::new();
        for record in records {
            sessions.push(self.record_to_session(record)?);
        }

        Ok(sessions)
    }

    /// List active sessions for a project.
    pub async fn list_for_project(&self, project: &str) -> StoreResult<Vec<ActiveSession>> {
        debug!("Listing active sessions for project: {}", project);

        let mut result = self.db
            .query("SELECT * FROM active_session WHERE project = $project ORDER BY last_heartbeat DESC")
            .bind(("project", project.to_string()))
            .await?;

        let records: Vec<ActiveSessionRecord> = result.take(0)?;

        let mut sessions = Vec::new();
        for record in records {
            sessions.push(self.record_to_session(record)?);
        }

        Ok(sessions)
    }

    /// Find potential conflicts for a session based on overlapping components.
    pub async fn find_conflicts(
        &self,
        session_id: &Id,
        components: &[String],
    ) -> StoreResult<Vec<ConflictInfo>> {
        debug!(
            "Finding conflicts for session {} with components {:?}",
            session_id, components
        );

        if components.is_empty() {
            return Ok(Vec::new());
        }

        // Get all other active sessions
        let sessions = self.list_active().await?;

        let mut conflicts = Vec::new();
        for other_session in sessions {
            // Skip our own session
            if other_session.session_id == *session_id {
                continue;
            }

            // Find overlapping components
            let overlapping: Vec<String> = components
                .iter()
                .filter(|c| other_session.components.contains(c))
                .cloned()
                .collect();

            if !overlapping.is_empty() {
                conflicts.push(ConflictInfo::from_session(&other_session, overlapping));
            }
        }

        Ok(conflicts)
    }

    /// Find sessions editing the same file.
    pub async fn find_file_conflicts(
        &self,
        session_id: &Id,
        file: &str,
    ) -> StoreResult<Vec<ConflictInfo>> {
        debug!(
            "Finding file conflicts for session {} editing {}",
            session_id, file
        );

        let mut result = self.db
            .query("SELECT * FROM active_session WHERE current_file = $file AND session_id != $session_id")
            .bind(("file", file.to_string()))
            .bind(("session_id", session_id.to_string()))
            .await?;

        let records: Vec<ActiveSessionRecord> = result.take(0)?;

        let mut conflicts = Vec::new();
        for record in records {
            let session = self.record_to_session(record)?;
            conflicts.push(ConflictInfo {
                other_session_id: session.session_id,
                other_agent: session.agent,
                other_goal: session.goal,
                overlapping_components: vec![file.to_string()],
                other_current_file: session.current_file,
            });
        }

        Ok(conflicts)
    }

    /// Clean up stale sessions (older than timeout_minutes).
    pub async fn cleanup_stale(&self, timeout_minutes: i64) -> StoreResult<usize> {
        debug!(
            "Cleaning up stale sessions (timeout: {} minutes)",
            timeout_minutes
        );

        let cutoff = time::OffsetDateTime::now_utc() - time::Duration::minutes(timeout_minutes);
        let cutoff_str = cutoff
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap();

        // First count how many we'll delete
        let mut result = self.db
            .query("SELECT count() as count FROM active_session WHERE last_heartbeat < $cutoff GROUP ALL")
            .bind(("cutoff", cutoff_str.clone()))
            .await?;

        let count: Option<CountResult> = result.take(0)?;
        let deleted_count = count.map(|c| c.count as usize).unwrap_or(0);

        // Then delete them
        self.db
            .query("DELETE FROM active_session WHERE last_heartbeat < $cutoff")
            .bind(("cutoff", cutoff_str))
            .await?;

        Ok(deleted_count)
    }

    // =========================================================================
    // Statistics
    // =========================================================================

    /// Get coordination statistics.
    pub async fn stats(&self) -> StoreResult<CoordinationStats> {
        let mut result = self
            .db
            .query(format!(
                "SELECT count() as count FROM {} GROUP ALL",
                TABLE_ACTIVE_SESSION
            ))
            .await?;

        let session_count: Option<CountResult> = result.take(0)?;

        Ok(CoordinationStats {
            active_session_count: session_count.map(|c| c.count).unwrap_or(0),
        })
    }

    // =========================================================================
    // Helper Methods
    // =========================================================================

    fn record_to_session(&self, record: ActiveSessionRecord) -> StoreResult<ActiveSession> {
        let session_id = Id::parse(&record.session_id)
            .map_err(|e| StoreError::Deserialization(format!("Invalid session ID: {}", e)))?;

        Ok(ActiveSession {
            session_id,
            agent: record.agent,
            project: record.project,
            goal: record.goal,
            components: record.components,
            current_file: record.current_file,
            started_at: record.started_at.to_offset_datetime()?,
            last_heartbeat: record.last_heartbeat.to_offset_datetime()?,
        })
    }
}

/// Coordination statistics.
#[derive(Debug, Clone, Default)]
pub struct CoordinationStats {
    /// Number of active sessions.
    pub active_session_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordination_stats_default() {
        let stats = CoordinationStats::default();
        assert_eq!(stats.active_session_count, 0);
    }
}
