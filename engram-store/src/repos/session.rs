//! Session repository for Layer 2: Session History.
//!
//! Handles persistence of Session and Event data.

use crate::error::{StoreError, StoreResult};
use crate::Db;
use engram_core::id::Id;
use engram_core::session::{Event, EventType, Session, SessionStats, SessionStatus};
use serde::Deserialize;
use std::collections::HashMap;
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
                // SurrealDB native format: [year, month, day, hour, min, sec, nano, offset_h, offset_m]
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

/// Session record from SurrealDB.
#[derive(Debug, Deserialize)]
struct SessionRecord {
    id: String,
    project: Option<String>,
    agent: Option<String>,
    goal: Option<String>,
    status: String,
    summary: Option<String>,
    #[serde(default)]
    key_decisions: Vec<String>,
    started_at: SurrealDateTime,
    ended_at: Option<SurrealDateTime>,
}

/// Event record from SurrealDB.
#[derive(Debug, Deserialize)]
struct EventRecord {
    id: String,
    session_id: String,
    event_type: String,
    actor: String,
    content: String,
    context: Option<String>,
    source: Option<String>,
    #[serde(default)]
    entities_mentioned: Vec<String>,
    timestamp: SurrealDateTime,
}

/// Table names for session storage.
const TABLE_SESSION: &str = "session";
const TABLE_EVENT: &str = "session_event";

/// Repository for session operations.
#[derive(Clone)]
pub struct SessionRepo {
    db: Db,
}

impl SessionRepo {
    /// Create a new session repository.
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Initialize the session schema.
    pub async fn init_schema(&self) -> StoreResult<()> {
        info!("Initializing session schema (Layer 2)");

        // Session table
        self.db
            .query(format!(
                r#"
                DEFINE TABLE IF NOT EXISTS {TABLE_SESSION} SCHEMALESS;
                DEFINE INDEX IF NOT EXISTS idx_session_status ON {TABLE_SESSION} FIELDS status;
                DEFINE INDEX IF NOT EXISTS idx_session_agent ON {TABLE_SESSION} FIELDS agent;
                DEFINE INDEX IF NOT EXISTS idx_session_started ON {TABLE_SESSION} FIELDS started_at;
                DEFINE INDEX IF NOT EXISTS idx_session_project ON {TABLE_SESSION} FIELDS project;
                "#
            ))
            .await?;

        // Event table
        self.db
            .query(format!(
                r#"
                DEFINE TABLE IF NOT EXISTS {TABLE_EVENT} SCHEMALESS;
                DEFINE INDEX IF NOT EXISTS idx_event_session ON {TABLE_EVENT} FIELDS session_id;
                DEFINE INDEX IF NOT EXISTS idx_event_type ON {TABLE_EVENT} FIELDS event_type;
                DEFINE INDEX IF NOT EXISTS idx_event_timestamp ON {TABLE_EVENT} FIELDS timestamp;
                "#
            ))
            .await?;

        info!("Session schema initialized");
        Ok(())
    }

    // =========================================================================
    // Session Operations
    // =========================================================================

    /// Save a session.
    pub async fn save_session(&self, session: &Session) -> StoreResult<()> {
        debug!("Saving session: {}", session.id);

        let ended_at_str = session.ended_at.map(|dt| {
            dt.format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default()
        });

        self.db
            .query(
                r#"
                UPSERT type::thing("session", $id) SET
                    project = $project,
                    agent = $agent,
                    goal = $goal,
                    status = $status,
                    summary = $summary,
                    key_decisions = $key_decisions,
                    started_at = $started_at,
                    ended_at = $ended_at
            "#,
            )
            .bind(("id", session.id.to_string()))
            .bind(("project", session.project.clone()))
            .bind(("agent", session.agent.clone()))
            .bind(("goal", session.goal.clone()))
            .bind(("status", session.status.to_string()))
            .bind(("summary", session.summary.clone()))
            .bind(("key_decisions", session.key_decisions.clone()))
            .bind((
                "started_at",
                session
                    .started_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            ))
            .bind(("ended_at", ended_at_str))
            .await?;

        Ok(())
    }

    /// Get a session by ID.
    pub async fn get_session(&self, id: &Id) -> StoreResult<Option<Session>> {
        debug!("Getting session: {}", id);

        let mut result = self.db
            .query(r#"SELECT meta::id(id) as id, project, agent, goal, status, summary, key_decisions, started_at, ended_at FROM type::thing("session", $id)"#)
            .bind(("id", id.to_string()))
            .await?;

        let records: Vec<SessionRecord> = result.take(0)?;

        if let Some(record) = records.into_iter().next() {
            Ok(Some(self.record_to_session(record)?))
        } else {
            Ok(None)
        }
    }

    /// List sessions with optional filters.
    pub async fn list_sessions(
        &self,
        status: Option<&SessionStatus>,
        agent: Option<&str>,
        project: Option<&str>,
        limit: Option<usize>,
    ) -> StoreResult<Vec<Session>> {
        debug!(
            "Listing sessions (status: {:?}, agent: {:?}, project: {:?})",
            status, agent, project
        );

        let mut conditions = Vec::new();
        if let Some(s) = status {
            conditions.push(format!("status = '{}'", s));
        }
        if let Some(a) = agent {
            conditions.push(format!("agent = '{}'", a));
        }
        if let Some(p) = project {
            conditions.push(format!("project = '{}'", p));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let limit_clause = limit.map(|l| format!("LIMIT {}", l)).unwrap_or_default();

        let query = format!(
            "SELECT meta::id(id) as id, project, agent, goal, status, summary, key_decisions, started_at, ended_at FROM {} {} ORDER BY started_at DESC {}",
            TABLE_SESSION, where_clause, limit_clause
        );

        let mut result = self.db.query(query).await?;
        let records: Vec<SessionRecord> = result.take(0)?;

        let mut sessions = Vec::new();
        for record in records {
            sessions.push(self.record_to_session(record)?);
        }

        Ok(sessions)
    }

    /// Get active sessions (optionally filtered by project).
    pub async fn get_active_sessions(&self, project: Option<&str>) -> StoreResult<Vec<Session>> {
        self.list_sessions(Some(&SessionStatus::Active), None, project, None)
            .await
    }

    /// End a session.
    pub async fn end_session(
        &self,
        id: &Id,
        status: SessionStatus,
        summary: Option<String>,
    ) -> StoreResult<()> {
        debug!("Ending session: {} with status {:?}", id, status);

        let ended_at = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap();

        self.db
            .query(
                r#"
                UPDATE type::thing("session", $id) SET
                    status = $status,
                    summary = $summary,
                    ended_at = $ended_at
            "#,
            )
            .bind(("id", id.to_string()))
            .bind(("status", status.to_string()))
            .bind(("summary", summary))
            .bind(("ended_at", ended_at))
            .await?;

        Ok(())
    }

    /// Delete a session and its events.
    pub async fn delete_session(&self, id: &Id) -> StoreResult<()> {
        debug!("Deleting session: {}", id);

        // Delete associated events
        self.db
            .query("DELETE FROM session_event WHERE session_id = $id")
            .bind(("id", id.to_string()))
            .await?;

        // Delete the session
        self.db
            .query(r#"DELETE type::thing("session", $id)"#)
            .bind(("id", id.to_string()))
            .await?;

        Ok(())
    }

    // =========================================================================
    // Event Operations
    // =========================================================================

    /// Add an event to a session.
    pub async fn add_event(&self, event: &Event) -> StoreResult<()> {
        debug!(
            "Adding event to session {}: {:?}",
            event.session_id, event.event_type
        );

        let entities_mentioned: Vec<String> = event
            .entities_mentioned
            .iter()
            .map(|id| id.to_string())
            .collect();

        self.db
            .query(
                r#"
                CREATE type::thing("session_event", $id) SET
                    session_id = $session_id,
                    event_type = $event_type,
                    actor = $actor,
                    content = $content,
                    context = $context,
                    source = $source,
                    entities_mentioned = $entities,
                    timestamp = $timestamp
            "#,
            )
            .bind(("id", event.id.to_string()))
            .bind(("session_id", event.session_id.to_string()))
            .bind(("event_type", event.event_type.to_string()))
            .bind(("actor", event.actor.clone()))
            .bind(("content", event.content.clone()))
            .bind(("context", event.context.clone()))
            .bind(("source", event.source.clone()))
            .bind(("entities", entities_mentioned))
            .bind((
                "timestamp",
                event
                    .timestamp
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            ))
            .await?;

        Ok(())
    }

    /// Get events for a session.
    pub async fn get_events(&self, session_id: &Id) -> StoreResult<Vec<Event>> {
        debug!("Getting events for session: {}", session_id);

        let mut result = self.db
            .query("SELECT meta::id(id) as id, session_id, event_type, actor, content, context, source, entities_mentioned, timestamp FROM session_event WHERE session_id = $session_id ORDER BY timestamp ASC")
            .bind(("session_id", session_id.to_string()))
            .await?;

        let records: Vec<EventRecord> = result.take(0)?;

        let mut events = Vec::new();
        for record in records {
            events.push(self.record_to_event(record)?);
        }

        Ok(events)
    }

    /// Search events by content (across all sessions).
    pub async fn search_events(
        &self,
        query: &str,
        limit: Option<usize>,
    ) -> StoreResult<Vec<Event>> {
        debug!("Searching events: {}", query);

        let limit_clause = limit
            .map(|l| format!("LIMIT {}", l))
            .unwrap_or_else(|| "LIMIT 50".to_string());

        let mut result = self.db
            .query(format!(
                "SELECT meta::id(id) as id, session_id, event_type, actor, content, context, source, entities_mentioned, timestamp FROM session_event WHERE string::lowercase(content) CONTAINS $query ORDER BY timestamp DESC {}",
                limit_clause
            ))
            .bind(("query", query.to_lowercase()))
            .await?;

        let records: Vec<EventRecord> = result.take(0)?;

        let mut events = Vec::new();
        for record in records {
            events.push(self.record_to_event(record)?);
        }

        Ok(events)
    }

    /// Get events by type (across all sessions).
    pub async fn get_events_by_type(
        &self,
        event_type: &EventType,
        limit: Option<usize>,
    ) -> StoreResult<Vec<Event>> {
        debug!("Getting events by type: {:?}", event_type);

        let limit_clause = limit
            .map(|l| format!("LIMIT {}", l))
            .unwrap_or_else(|| "LIMIT 50".to_string());

        let mut result = self.db
            .query(format!(
                "SELECT meta::id(id) as id, session_id, event_type, actor, content, context, source, entities_mentioned, timestamp FROM session_event WHERE event_type = $event_type ORDER BY timestamp DESC {}",
                limit_clause
            ))
            .bind(("event_type", event_type.to_string()))
            .await?;

        let records: Vec<EventRecord> = result.take(0)?;

        let mut events = Vec::new();
        for record in records {
            events.push(self.record_to_event(record)?);
        }

        Ok(events)
    }

    // =========================================================================
    // Statistics
    // =========================================================================

    /// Get session statistics.
    pub async fn stats(&self) -> StoreResult<SessionStats> {
        debug!("Getting session statistics");

        // Count sessions by status
        let mut result = self
            .db
            .query(
                r#"
                SELECT
                    count() as total,
                    count(status = 'active') as active,
                    count(status = 'completed') as completed,
                    count(status = 'abandoned') as abandoned
                FROM session GROUP ALL
            "#,
            )
            .await?;

        #[derive(Debug, Deserialize)]
        struct SessionCounts {
            total: usize,
            active: usize,
            completed: usize,
            abandoned: usize,
        }

        let counts: Vec<SessionCounts> = result.take(0)?;
        let counts = counts.into_iter().next().unwrap_or(SessionCounts {
            total: 0,
            active: 0,
            completed: 0,
            abandoned: 0,
        });

        // Count events by type
        let mut result = self
            .db
            .query("SELECT event_type, count() as count FROM session_event GROUP BY event_type")
            .await?;

        #[derive(Debug, Deserialize)]
        struct EventCount {
            event_type: String,
            count: usize,
        }

        let event_counts: Vec<EventCount> = result.take(0)?;
        let events_by_type: HashMap<String, usize> = event_counts
            .into_iter()
            .map(|ec| (ec.event_type, ec.count))
            .collect();

        let total_events: usize = events_by_type.values().sum();

        Ok(SessionStats {
            total_sessions: counts.total,
            active_sessions: counts.active,
            completed_sessions: counts.completed,
            abandoned_sessions: counts.abandoned,
            total_events,
            events_by_type,
        })
    }

    // =========================================================================
    // Helper Methods
    // =========================================================================

    fn parse_record_id(&self, id_str: &str) -> StoreResult<Id> {
        // SurrealDB returns IDs as "table:uuid"
        let id_part = id_str.split(':').last().unwrap_or(id_str);
        Id::parse(id_part).map_err(|e| StoreError::Deserialization(format!("Invalid ID: {}", e)))
    }

    fn record_to_session(&self, record: SessionRecord) -> StoreResult<Session> {
        let id = self.parse_record_id(&record.id)?;
        let started_at = record.started_at.to_offset_datetime()?;
        let ended_at = record
            .ended_at
            .map(|dt| dt.to_offset_datetime())
            .transpose()?;

        Ok(Session {
            id,
            project: record.project,
            agent: record.agent,
            goal: record.goal,
            status: SessionStatus::parse(&record.status),
            summary: record.summary,
            key_decisions: record.key_decisions,
            started_at,
            ended_at,
        })
    }

    fn record_to_event(&self, record: EventRecord) -> StoreResult<Event> {
        let id = self.parse_record_id(&record.id)?;
        let session_id = self.parse_record_id(&record.session_id)?;
        let timestamp = record.timestamp.to_offset_datetime()?;

        let entities_mentioned: Vec<Id> = record
            .entities_mentioned
            .iter()
            .filter_map(|s| Id::parse(s).ok())
            .collect();

        Ok(Event {
            id,
            session_id,
            event_type: EventType::parse(&record.event_type),
            actor: record.actor,
            content: record.content,
            context: record.context,
            source: record.source,
            entities_mentioned,
            timestamp,
        })
    }
}
