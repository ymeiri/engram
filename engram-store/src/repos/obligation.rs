//! Agent obligation repository.

use crate::error::{StoreError, StoreResult};
use crate::Db;
use engram_core::id::Id;
use engram_core::obligation::{AgentObligation, AgentObligationStatus};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

#[derive(Debug, Clone, Deserialize)]
struct AgentObligationRecord {
    record_id: String,
    obligation: serde_json::Value,
}

impl AgentObligationRecord {
    fn into_obligation(self) -> StoreResult<AgentObligation> {
        let mut obligation: AgentObligation = from_json(self.obligation)?;
        obligation.id = Id::parse(&self.record_id)
            .map_err(|e| StoreError::Deserialization(format!("Invalid obligation ID: {e}")))?;
        Ok(obligation)
    }
}

/// Repository for agent-native obligations.
#[derive(Clone)]
pub struct ObligationRepo {
    db: Db,
}

impl ObligationRepo {
    /// Create a new obligation repository.
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Initialize obligation tables and indexes.
    pub async fn init_schema(&self) -> StoreResult<()> {
        info!("Initializing obligation schema");

        self.db
            .query(
                r#"
                DEFINE TABLE IF NOT EXISTS agent_obligation SCHEMALESS;
                DEFINE INDEX IF NOT EXISTS idx_agent_obligation_kind ON agent_obligation FIELDS kind_key;
                DEFINE INDEX IF NOT EXISTS idx_agent_obligation_status ON agent_obligation FIELDS status_key;
                DEFINE INDEX IF NOT EXISTS idx_agent_obligation_scope ON agent_obligation FIELDS scope_key;
                DEFINE INDEX IF NOT EXISTS idx_agent_obligation_harness ON agent_obligation FIELDS harness_key;
                DEFINE INDEX IF NOT EXISTS idx_agent_obligation_session ON agent_obligation FIELDS session_id;
                DEFINE INDEX IF NOT EXISTS idx_agent_obligation_created ON agent_obligation FIELDS created_at;
                DEFINE INDEX IF NOT EXISTS idx_agent_obligation_updated ON agent_obligation FIELDS updated_at;
                "#,
            )
            .await?;

        info!("Obligation schema initialized");
        Ok(())
    }

    /// Save an obligation.
    pub async fn save_obligation(&self, obligation: &AgentObligation) -> StoreResult<()> {
        debug!("Saving obligation: {}", obligation.id);

        self.db
            .query(
                r#"
                UPSERT type::thing("agent_obligation", $id) SET
                    obligation = $obligation,
                    kind_key = $kind_key,
                    status_key = $status_key,
                    scope_key = $scope_key,
                    harness_key = $harness_key,
                    session_id = $session_id,
                    created_at = $created_at,
                    updated_at = $updated_at
                "#,
            )
            .bind(("id", obligation.id.to_string()))
            .bind(("obligation", to_json(obligation)?))
            .bind(("kind_key", obligation.kind.to_string()))
            .bind(("status_key", obligation.status.to_string()))
            .bind(("scope_key", scope_key(obligation)))
            .bind(("harness_key", obligation.writer.harness.to_string()))
            .bind((
                "session_id",
                obligation.writer.session_id.map(|id| id.to_string()),
            ))
            .bind(("created_at", format_rfc3339(obligation.created_at)?))
            .bind(("updated_at", format_rfc3339(obligation.updated_at)?))
            .await?;

        Ok(())
    }

    /// Get an obligation by ID.
    pub async fn get_obligation(&self, id: &Id) -> StoreResult<Option<AgentObligation>> {
        debug!("Getting obligation: {id}");

        let mut result = self
            .db
            .query(
                r#"
                SELECT meta::id(id) AS record_id, obligation
                FROM type::thing("agent_obligation", $id)
                "#,
            )
            .bind(("id", id.to_string()))
            .await?;

        let records: Vec<AgentObligationRecord> = result.take(0)?;
        records
            .into_iter()
            .next()
            .map(AgentObligationRecord::into_obligation)
            .transpose()
    }

    /// List obligations, newest updates first.
    pub async fn list_obligations(
        &self,
        status: Option<AgentObligationStatus>,
        limit: Option<usize>,
    ) -> StoreResult<Vec<AgentObligation>> {
        let mut query =
            "SELECT meta::id(id) AS record_id, obligation, updated_at FROM agent_obligation"
                .to_string();
        if status.is_some() {
            query.push_str(" WHERE status_key = $status");
        }
        query.push_str(" ORDER BY updated_at DESC");
        if let Some(limit) = limit {
            query.push_str(&format!(" LIMIT {limit}"));
        }

        let mut result = if let Some(status) = status {
            self.db
                .query(query)
                .bind(("status", status.to_string()))
                .await?
        } else {
            self.db.query(query).await?
        };

        decode_obligations(result.take(0)?)
    }
}

fn decode_obligations(records: Vec<AgentObligationRecord>) -> StoreResult<Vec<AgentObligation>> {
    records
        .into_iter()
        .map(AgentObligationRecord::into_obligation)
        .collect()
}

fn format_rfc3339(value: time::OffsetDateTime) -> StoreResult<String> {
    value
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| StoreError::Deserialization(format!("Invalid timestamp: {e}")))
}

fn to_json<T: Serialize>(value: &T) -> StoreResult<serde_json::Value> {
    serde_json::to_value(value).map_err(StoreError::Serialization)
}

fn from_json<T: DeserializeOwned>(value: serde_json::Value) -> StoreResult<T> {
    serde_json::from_value(value).map_err(StoreError::Serialization)
}

fn scope_key(obligation: &AgentObligation) -> String {
    match &obligation.scope {
        engram_core::memory::MemoryScope::Global => "global".to_string(),
        engram_core::memory::MemoryScope::User => "user".to_string(),
        engram_core::memory::MemoryScope::Project { project_name, .. } => {
            format!("project:{project_name}")
        }
        engram_core::memory::MemoryScope::Task { task_name, .. } => {
            format!("task:{task_name}")
        }
        engram_core::memory::MemoryScope::Entity { entity_name, .. } => {
            format!("entity:{entity_name}")
        }
        engram_core::memory::MemoryScope::Repository {
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
        engram_core::memory::MemoryScope::Session { session_id } => {
            format!("session:{session_id}")
        }
        engram_core::memory::MemoryScope::Custom { name } => format!("custom:{name}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engram_core::memory::{
        EvidenceKind, EvidenceRef, Harness, MemoryScope, ModelIdentity, WriterProvenance,
    };
    use engram_core::obligation::{
        AgentObligationKind, AgentObligationResolution, AgentObligationResolutionKind,
        AgentObligationTrigger,
    };

    async fn setup_repo() -> ObligationRepo {
        let config = crate::StoreConfig::memory();
        let db = crate::connect_and_init(&config).await.unwrap();
        let repo = ObligationRepo::new(db);
        repo.init_schema().await.unwrap();
        repo
    }

    fn writer() -> WriterProvenance {
        WriterProvenance::agent(Harness::Codex, ModelIdentity::new("openai", "gpt-5.5"))
    }

    fn obligation(title: &str) -> AgentObligation {
        AgentObligation::new(
            AgentObligationKind::SourceReading,
            title,
            "Read source before implementing.",
            MemoryScope::project("engram"),
            AgentObligationTrigger::new("prompt", "User asked for implementation"),
            writer(),
        )
        .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "test"))
    }

    #[tokio::test]
    async fn save_get_and_list_obligations() {
        let repo = setup_repo().await;
        let obligation = obligation("Read the harness code");

        repo.save_obligation(&obligation).await.unwrap();
        let retrieved = repo.get_obligation(&obligation.id).await.unwrap().unwrap();
        assert_eq!(retrieved.id, obligation.id);
        assert_eq!(retrieved.kind, AgentObligationKind::SourceReading);

        let open = repo
            .list_obligations(Some(AgentObligationStatus::Open), Some(10))
            .await
            .unwrap();
        assert_eq!(open.len(), 1);
    }

    #[tokio::test]
    async fn resolved_obligations_are_filterable() {
        let repo = setup_repo().await;
        let mut obligation = obligation("Recover failed tool call");
        obligation.resolve(AgentObligationResolution::new(
            AgentObligationResolutionKind::RetriedTool,
            "Retried successfully.",
            "agent",
        ));

        repo.save_obligation(&obligation).await.unwrap();

        let open = repo
            .list_obligations(Some(AgentObligationStatus::Open), None)
            .await
            .unwrap();
        let resolved = repo
            .list_obligations(Some(AgentObligationStatus::Resolved), None)
            .await
            .unwrap();

        assert!(open.is_empty());
        assert_eq!(resolved.len(), 1);
    }
}
