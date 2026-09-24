//! Brain harness telemetry repository.

use crate::error::{StoreError, StoreResult};
use crate::secret::redact_serialized_secret_material;
use crate::Db;
use engram_core::id::Id;
use engram_core::telemetry::{AgentFeedback, BrainHarnessTrace};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
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

/// Telemetry records removed because they referenced a permanently forgotten memory item.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TelemetryMemoryPurge {
    /// Retrieval traces deleted.
    pub traces_deleted: usize,
    /// Feedback records deleted.
    pub feedback_deleted: usize,
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
            .await?.check()?;

        info!("Brain harness telemetry schema initialized");
        Ok(())
    }

    /// Save a trace.
    pub async fn save_trace(&self, trace: &BrainHarnessTrace) -> StoreResult<()> {
        let (mut trace, redacted_fields) =
            redact_serialized_secret_material("brain harness trace", trace)?;
        if redacted_fields > 0 {
            trace.warnings.push(format!(
                "Redacted {redacted_fields} secret-bearing telemetry field(s) before durable persistence."
            ));
        }
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
            .bind(("trace", to_json(&trace)?))
            .bind(("operation_key", trace.operation.to_string()))
            .bind((
                "intent_key",
                trace.intent.as_ref().map(std::string::ToString::to_string),
            ))
            .bind(("session_id", trace.session_id.map(|id| id.to_string())))
            .bind(("external_session_id", trace.external_session_id.clone()))
            .bind(("project", trace.project.clone()))
            .bind(("created_at", format_rfc3339(trace.created_at)?))
            .await?
            .check()?;

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
            .await?
            .check()?;

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
            .await?
            .check()?;

        let records: Vec<TraceRecord> = result.take(0)?;
        records.into_iter().map(TraceRecord::into_trace).collect()
    }

    /// List scoped traces, newest first, applying the limit after scope filters.
    pub async fn list_traces_scoped(
        &self,
        limit: Option<usize>,
        project: Option<&str>,
        scenario_id: Option<&str>,
        arm: Option<&str>,
        intent_key: Option<&str>,
    ) -> StoreResult<Vec<BrainHarnessTrace>> {
        debug!("Listing scoped brain harness traces");

        let mut conditions = Vec::new();
        if project.is_some() {
            conditions.push("project = $project");
        }
        if scenario_id.is_some() {
            conditions.push("trace.scenario_id = $scenario_id");
        }
        if arm.is_some() {
            conditions.push("trace.arm = $arm");
        }
        if intent_key.is_some() {
            conditions.push("intent_key = $intent_key");
        }
        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let mut query = self.db.query(format!(
            r#"
            SELECT meta::id(id) AS record_id, trace, created_at
            FROM {TABLE_TRACE}
            {where_clause}
            ORDER BY created_at DESC
            LIMIT {}
            "#,
            limit.unwrap_or(100)
        ));
        if let Some(project) = project {
            query = query.bind(("project", project.to_string()));
        }
        if let Some(scenario_id) = scenario_id {
            query = query.bind(("scenario_id", scenario_id.to_string()));
        }
        if let Some(arm) = arm {
            query = query.bind(("arm", arm.to_string()));
        }
        if let Some(intent_key) = intent_key {
            query = query.bind(("intent_key", intent_key.to_string()));
        }

        let mut result = query.await?.check()?;
        let records: Vec<TraceRecord> = result.take(0)?;
        records.into_iter().map(TraceRecord::into_trace).collect()
    }

    /// Save agent feedback.
    pub async fn save_feedback(&self, feedback: &AgentFeedback) -> StoreResult<()> {
        let (feedback, _) = redact_serialized_secret_material("agent feedback", feedback)?;
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
            .bind(("feedback", to_json(&feedback)?))
            .bind(("trace_id", feedback.trace_id.to_string()))
            .bind(("session_id", feedback.session_id.map(|id| id.to_string())))
            .bind(("external_session_id", feedback.external_session_id.clone()))
            .bind(("created_at", format_rfc3339(feedback.created_at)?))
            .await?
            .check()?;

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
            .await?
            .check()?;

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
            .await?
            .check()?;

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
            .await?
            .check()?;

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
            .await?
            .check()?;

        let records: Vec<FeedbackRecord> = result.take(0)?;
        records
            .into_iter()
            .map(FeedbackRecord::into_feedback)
            .collect()
    }

    /// Delete traces and feedback that reference a permanently forgotten memory item.
    pub async fn purge_memory_references(
        &self,
        memory_id: &Id,
    ) -> StoreResult<TelemetryMemoryPurge> {
        let memory_id_text = memory_id.to_string();
        let mut trace_result = self
            .db
            .query(format!(
                "SELECT meta::id(id) AS record_id, trace FROM {TABLE_TRACE}"
            ))
            .await?
            .check()?;
        let traces: Vec<TraceRecord> = trace_result.take(0)?;
        let traces = traces
            .into_iter()
            .map(TraceRecord::into_trace)
            .collect::<StoreResult<Vec<_>>>()?;
        let trace_ids = traces
            .into_iter()
            .filter(|trace| {
                trace.returned_memory_ids.contains(memory_id)
                    || trace
                        .returned_result_ids
                        .iter()
                        .any(|result_id| result_id == &memory_id_text)
            })
            .map(|trace| trace.id)
            .collect::<HashSet<_>>();

        let mut feedback_result = self
            .db
            .query(format!(
                "SELECT meta::id(id) AS record_id, feedback FROM {TABLE_FEEDBACK}"
            ))
            .await?
            .check()?;
        let feedback: Vec<FeedbackRecord> = feedback_result.take(0)?;
        let feedback_ids = feedback
            .into_iter()
            .map(FeedbackRecord::into_feedback)
            .collect::<StoreResult<Vec<_>>>()?
            .into_iter()
            .filter(|feedback| {
                trace_ids.contains(&feedback.trace_id)
                    || feedback.used_memory_ids.contains(memory_id)
                    || feedback.rejected_memory_ids.contains(memory_id)
                    || feedback.stale_memory_ids.contains(memory_id)
                    || feedback.wrong_scope_memory_ids.contains(memory_id)
                    || feedback
                        .used_result_ids
                        .iter()
                        .chain(&feedback.rejected_result_ids)
                        .any(|result_id| result_id == &memory_id_text)
            })
            .map(|feedback| feedback.id)
            .collect::<HashSet<_>>();

        for feedback_id in &feedback_ids {
            self.db
                .query(r#"DELETE type::thing("agent_feedback", $id)"#)
                .bind(("id", feedback_id.to_string()))
                .await?
                .check()?;
        }
        for trace_id in &trace_ids {
            self.db
                .query(r#"DELETE type::thing("brain_harness_trace", $id)"#)
                .bind(("id", trace_id.to_string()))
                .await?
                .check()?;
        }

        Ok(TelemetryMemoryPurge {
            traces_deleted: trace_ids.len(),
            feedback_deleted: feedback_ids.len(),
        })
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

#[cfg(test)]
mod tests {
    use super::*;
    use engram_core::telemetry::BrainHarnessOperation;

    async fn setup_repo() -> TelemetryRepo {
        let config = crate::StoreConfig::memory();
        let db = crate::connect_and_init(&config).await.unwrap();
        let repo = TelemetryRepo::new(db);
        repo.init_schema().await.unwrap();
        repo
    }

    #[tokio::test]
    async fn save_trace_redacts_secret_material_without_failing_operation() {
        let repo = setup_repo().await;
        let canary = "Authorization: Bearer synthetic-trace-secret";
        let trace = BrainHarnessTrace::new(BrainHarnessOperation::Orient)
            .with_query(Some(canary.to_string()));

        repo.save_trace(&trace).await.unwrap();

        let stored = repo.get_trace(&trace.id).await.unwrap().unwrap();
        let stored_json = serde_json::to_string(&stored).unwrap();
        assert!(!stored_json.contains(canary));
        assert!(stored_json.contains("redacted"));
    }

    #[tokio::test]
    async fn save_feedback_redacts_secret_material_without_dropping_feedback() {
        let repo = setup_repo().await;
        let trace = BrainHarnessTrace::new(BrainHarnessOperation::Orient);
        repo.save_trace(&trace).await.unwrap();
        let canary = "API_TOKEN=synthetic-feedback-secret";
        let mut feedback = AgentFeedback::new(trace.id);
        feedback.note = Some(canary.to_string());

        repo.save_feedback(&feedback).await.unwrap();

        let stored = repo.get_feedback(&feedback.id).await.unwrap().unwrap();
        let stored_json = serde_json::to_string(&stored).unwrap();
        assert!(!stored_json.contains(canary));
        assert!(stored_json.contains("redacted"));
    }
}
