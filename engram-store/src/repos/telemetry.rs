//! Brain harness telemetry repository.

use crate::error::{StoreError, StoreResult};
use crate::Db;
use engram_core::id::Id;
use engram_core::telemetry::{AgentFeedback, BrainHarnessTrace};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

const TABLE_TRACE: &str = "brain_harness_trace";
const TABLE_FEEDBACK: &str = "agent_feedback";

#[derive(Debug, Clone, Deserialize)]
struct TraceRecord {
    record_id: String,
    trace: serde_json::Value,
}

impl TraceRecord {
    fn into_trace(self) -> StoreResult<BrainHarnessTrace> {
        let mut trace: BrainHarnessTrace = from_json(self.trace)?;
        trace.id = Id::parse(&self.record_id)
            .map_err(|e| StoreError::Deserialization(format!("Invalid trace ID: {e}")))?;
        Ok(trace)
    }
}

#[derive(Debug, Clone, Deserialize)]
struct FeedbackRecord {
    record_id: String,
    feedback: serde_json::Value,
}

impl FeedbackRecord {
    fn into_feedback(self) -> StoreResult<AgentFeedback> {
        let mut feedback: AgentFeedback = from_json(self.feedback)?;
        feedback.id = Id::parse(&self.record_id)
            .map_err(|e| StoreError::Deserialization(format!("Invalid feedback ID: {e}")))?;
        Ok(feedback)
    }
}

/// Repository for brain-harness traces and feedback.
#[derive(Clone)]
pub struct TelemetryRepo {
    db: Db,
}

impl TelemetryRepo {
    /// Create a new telemetry repository.
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Initialize telemetry tables and indexes.
    pub async fn init_schema(&self) -> StoreResult<()> {
        info!("Initializing brain harness telemetry schema");

        self.db
            .query(
                r#"
                DEFINE TABLE IF NOT EXISTS brain_harness_trace SCHEMALESS;
                DEFINE INDEX IF NOT EXISTS idx_trace_operation ON brain_harness_trace FIELDS operation_key;
                DEFINE INDEX IF NOT EXISTS idx_trace_intent ON brain_harness_trace FIELDS intent_key;
                DEFINE INDEX IF NOT EXISTS idx_trace_session ON brain_harness_trace FIELDS session_id;
                DEFINE INDEX IF NOT EXISTS idx_trace_external_session ON brain_harness_trace FIELDS external_session_id;
                DEFINE INDEX IF NOT EXISTS idx_trace_project ON brain_harness_trace FIELDS project;
                DEFINE INDEX IF NOT EXISTS idx_trace_created ON brain_harness_trace FIELDS created_at;

                DEFINE TABLE IF NOT EXISTS agent_feedback SCHEMALESS;
                DEFINE INDEX IF NOT EXISTS idx_feedback_trace ON agent_feedback FIELDS trace_id;
                DEFINE INDEX IF NOT EXISTS idx_feedback_session ON agent_feedback FIELDS session_id;
                DEFINE INDEX IF NOT EXISTS idx_feedback_external_session ON agent_feedback FIELDS external_session_id;
                DEFINE INDEX IF NOT EXISTS idx_feedback_created ON agent_feedback FIELDS created_at;
                "#,
            )
            .await?;

        info!("Brain harness telemetry schema initialized");
        Ok(())
    }

    /// Save a trace.
    pub async fn save_trace(&self, trace: &BrainHarnessTrace) -> StoreResult<()> {
        debug!("Saving brain harness trace: {}", trace.id);

        self.db
            .query(
                r#"
                UPSERT type::thing("brain_harness_trace", $id) SET
                    trace = $trace,
                    operation_key = $operation_key,
                    intent_key = $intent_key,
                    session_id = $session_id,
                    external_session_id = $external_session_id,
                    project = $project,
                    created_at = $created_at
                "#,
            )
            .bind(("id", trace.id.to_string()))
            .bind(("trace", to_json(trace)?))
            .bind(("operation_key", trace.operation.to_string()))
            .bind((
                "intent_key",
                trace.intent.as_ref().map(std::string::ToString::to_string),
            ))
            .bind(("session_id", trace.session_id.map(|id| id.to_string())))
            .bind(("external_session_id", trace.external_session_id.clone()))
            .bind(("project", trace.project.clone()))
            .bind(("created_at", format_rfc3339(trace.created_at)?))
            .await?;

        Ok(())
    }

    /// Get a trace by ID.
    pub async fn get_trace(&self, id: &Id) -> StoreResult<Option<BrainHarnessTrace>> {
        debug!("Getting brain harness trace: {id}");

        let mut result = self
            .db
            .query(
                r#"
                SELECT meta::id(id) AS record_id, trace
                FROM type::thing("brain_harness_trace", $id)
                "#,
            )
            .bind(("id", id.to_string()))
            .await?;

        let records: Vec<TraceRecord> = result.take(0)?;
        records
            .into_iter()
            .next()
            .map(TraceRecord::into_trace)
            .transpose()
    }

    /// List traces, newest first.
    pub async fn list_traces(&self, limit: Option<usize>) -> StoreResult<Vec<BrainHarnessTrace>> {
        debug!("Listing brain harness traces");

        let mut result = self
            .db
            .query(format!(
                r#"
                SELECT meta::id(id) AS record_id, trace, created_at
                FROM {TABLE_TRACE}
                ORDER BY created_at DESC
                LIMIT {}
                "#,
                limit.unwrap_or(100)
            ))
            .await?;

        let records: Vec<TraceRecord> = result.take(0)?;
        records.into_iter().map(TraceRecord::into_trace).collect()
    }

    /// Save agent feedback.
    pub async fn save_feedback(&self, feedback: &AgentFeedback) -> StoreResult<()> {
        debug!("Saving agent feedback: {}", feedback.id);

        self.db
            .query(
                r#"
                UPSERT type::thing("agent_feedback", $id) SET
                    feedback = $feedback,
                    trace_id = $trace_id,
                    session_id = $session_id,
                    external_session_id = $external_session_id,
                    created_at = $created_at
                "#,
            )
            .bind(("id", feedback.id.to_string()))
            .bind(("feedback", to_json(feedback)?))
            .bind(("trace_id", feedback.trace_id.to_string()))
            .bind(("session_id", feedback.session_id.map(|id| id.to_string())))
            .bind(("external_session_id", feedback.external_session_id.clone()))
            .bind(("created_at", format_rfc3339(feedback.created_at)?))
            .await?;

        Ok(())
    }

    /// Get feedback by ID.
    pub async fn get_feedback(&self, id: &Id) -> StoreResult<Option<AgentFeedback>> {
        debug!("Getting agent feedback: {id}");

        let mut result = self
            .db
            .query(
                r#"
                SELECT meta::id(id) AS record_id, feedback
                FROM type::thing("agent_feedback", $id)
                "#,
            )
            .bind(("id", id.to_string()))
            .await?;

        let records: Vec<FeedbackRecord> = result.take(0)?;
        records
            .into_iter()
            .next()
            .map(FeedbackRecord::into_feedback)
            .transpose()
    }

    /// List feedback for a trace.
    pub async fn list_feedback_for_trace(&self, trace_id: &Id) -> StoreResult<Vec<AgentFeedback>> {
        debug!("Listing feedback for trace: {trace_id}");

        let mut result = self
            .db
            .query(
                r#"
                SELECT meta::id(id) AS record_id, feedback, created_at
                FROM agent_feedback
                WHERE trace_id = $trace_id
                ORDER BY created_at ASC
                "#,
            )
            .bind(("trace_id", trace_id.to_string()))
            .await?;

        let records: Vec<FeedbackRecord> = result.take(0)?;
        records
            .into_iter()
            .map(FeedbackRecord::into_feedback)
            .collect()
    }

    /// List feedback for a set of traces, newest first.
    pub async fn list_feedback_for_traces(
        &self,
        trace_ids: &[Id],
    ) -> StoreResult<Vec<AgentFeedback>> {
        debug!("Listing feedback for {} traces", trace_ids.len());

        if trace_ids.is_empty() {
            return Ok(Vec::new());
        }

        let trace_ids = trace_ids
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        let mut result = self
            .db
            .query(
                r#"
                SELECT meta::id(id) AS record_id, feedback, created_at
                FROM agent_feedback
                WHERE trace_id IN $trace_ids
                ORDER BY created_at DESC
                "#,
            )
            .bind(("trace_ids", trace_ids))
            .await?;

        let records: Vec<FeedbackRecord> = result.take(0)?;
        records
            .into_iter()
            .map(FeedbackRecord::into_feedback)
            .collect()
    }

    /// List feedback, newest first.
    pub async fn list_feedback(&self, limit: Option<usize>) -> StoreResult<Vec<AgentFeedback>> {
        debug!("Listing agent feedback");

        let mut result = self
            .db
            .query(format!(
                r#"
                SELECT meta::id(id) AS record_id, feedback, created_at
                FROM {TABLE_FEEDBACK}
                ORDER BY created_at DESC
                LIMIT {}
                "#,
                limit.unwrap_or(100)
            ))
            .await?;

        let records: Vec<FeedbackRecord> = result.take(0)?;
        records
            .into_iter()
            .map(FeedbackRecord::into_feedback)
            .collect()
    }
}

fn to_json<T: Serialize>(value: &T) -> StoreResult<serde_json::Value> {
    Ok(serde_json::to_value(value)?)
}

fn from_json<T: DeserializeOwned>(value: serde_json::Value) -> StoreResult<T> {
    Ok(serde_json::from_value(value)?)
}

fn format_rfc3339(value: time::OffsetDateTime) -> StoreResult<String> {
    value
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| StoreError::Deserialization(format!("Invalid timestamp: {e}")))
}
