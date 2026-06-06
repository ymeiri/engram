//! Memory OS repository.
//!
//! Persists source-grounded memory items and Git-like knowledge commits.

use crate::error::{StoreError, StoreResult};
use crate::Db;
use engram_core::id::Id;
use engram_core::memory::{KnowledgeCommit, MemoryItem, MemoryStatus};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tracing::{debug, info};

/// SurrealDB datetime representation.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum SurrealDateTime {
    /// ISO 8601 string format.
    String(String),
    /// SurrealDB native datetime format.
    Native(serde_json::Value),
}

impl SurrealDateTime {
    fn to_offset_datetime(&self) -> StoreResult<OffsetDateTime> {
        match self {
            SurrealDateTime::String(s) => {
                OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339)
                    .map_err(|e| StoreError::Deserialization(format!("Invalid datetime: {e}")))
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
                        .map_err(|e| {
                            StoreError::Deserialization(format!("Invalid date in datetime: {e}"))
                        })?;

                        let time = time::Time::from_hms(hour, min, sec).map_err(|e| {
                            StoreError::Deserialization(format!("Invalid time in datetime: {e}"))
                        })?;

                        return Ok(OffsetDateTime::new_utc(date, time));
                    }
                }

                Err(StoreError::Deserialization(
                    "Invalid SurrealDB datetime value".to_string(),
                ))
            }
        }
    }
}

/// Record representation for memory items.
#[derive(Debug, Clone, Deserialize)]
struct MemoryItemRecord {
    record_id: String,
    item: serde_json::Value,
}

impl MemoryItemRecord {
    fn into_memory_item(self) -> StoreResult<MemoryItem> {
        let mut item: MemoryItem = from_json(self.item)?;
        item.id = Id::parse(&self.record_id)
            .map_err(|e| StoreError::Deserialization(format!("Invalid memory item ID: {e}")))?;
        Ok(item)
    }
}

/// Record representation for knowledge commits.
#[derive(Debug, Clone, Deserialize)]
struct KnowledgeCommitRecord {
    record_id: String,
    commit: serde_json::Value,
}

impl KnowledgeCommitRecord {
    fn into_knowledge_commit(self) -> StoreResult<KnowledgeCommit> {
        let mut commit: KnowledgeCommit = from_json(self.commit)?;
        commit.id = Id::parse(&self.record_id).map_err(|e| {
            StoreError::Deserialization(format!("Invalid knowledge commit ID: {e}"))
        })?;
        Ok(commit)
    }
}

/// Latest memory update timestamp record.
#[derive(Debug, Clone, Deserialize)]
struct LatestMemoryTimestampRecord {
    updated_at: SurrealDateTime,
}

/// Latest knowledge commit timestamp record.
#[derive(Debug, Clone, Deserialize)]
struct LatestCommitTimestampRecord {
    created_at: SurrealDateTime,
}

/// Repository for Memory OS persistence.
#[derive(Clone)]
pub struct MemoryRepo {
    db: Db,
}

impl MemoryRepo {
    /// Create a new memory repository.
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Initialize Memory OS tables and indexes.
    ///
    /// # Errors
    ///
    /// Returns an error if schema creation fails.
    pub async fn init_schema(&self) -> StoreResult<()> {
        info!("Initializing memory schema");

        self.db
            .query(
                r#"
                DEFINE TABLE IF NOT EXISTS memory_item SCHEMALESS;
                DEFINE INDEX IF NOT EXISTS idx_memory_item_kind ON memory_item FIELDS kind_key;
                DEFINE INDEX IF NOT EXISTS idx_memory_item_status ON memory_item FIELDS status_key;
                DEFINE INDEX IF NOT EXISTS idx_memory_item_scope ON memory_item FIELDS scope_key;
                DEFINE INDEX IF NOT EXISTS idx_memory_item_harness ON memory_item FIELDS harness_key;
                DEFINE INDEX IF NOT EXISTS idx_memory_item_session ON memory_item FIELDS session_id;
                DEFINE INDEX IF NOT EXISTS idx_memory_item_created ON memory_item FIELDS created_at;
                DEFINE INDEX IF NOT EXISTS idx_memory_item_updated ON memory_item FIELDS updated_at;

                DEFINE TABLE IF NOT EXISTS knowledge_commit SCHEMALESS;
                DEFINE INDEX IF NOT EXISTS idx_knowledge_commit_parent ON knowledge_commit FIELDS parent_id;
                DEFINE INDEX IF NOT EXISTS idx_knowledge_commit_session ON knowledge_commit FIELDS session_id;
                DEFINE INDEX IF NOT EXISTS idx_knowledge_commit_created ON knowledge_commit FIELDS created_at;
                "#,
            )
            .await?;

        info!("Memory schema initialized");
        Ok(())
    }

    /// Save a memory item.
    pub async fn save_memory_item(&self, item: &MemoryItem) -> StoreResult<()> {
        debug!("Saving memory item: {}", item.id);

        let item_json = to_json(item)?;
        self.db
            .query(
                r#"
                UPSERT type::thing("memory_item", $id) SET
                    item = $item,
                    kind_key = $kind_key,
                    status_key = $status_key,
                    scope_key = $scope_key,
                    harness_key = $harness_key,
                    model_key = $model_key,
                    session_id = $session_id,
                    created_at = $created_at,
                    updated_at = $updated_at
                "#,
            )
            .bind(("id", item.id.to_string()))
            .bind(("item", item_json))
            .bind(("kind_key", item.kind.to_string()))
            .bind(("status_key", item.status.to_string()))
            .bind(("scope_key", scope_key(item)))
            .bind(("harness_key", item.writer.harness.to_string()))
            .bind(("model_key", item.writer.model.model.clone()))
            .bind((
                "session_id",
                item.writer.session_id.map(|id| id.to_string()),
            ))
            .bind(("created_at", format_rfc3339(item.created_at)?))
            .bind(("updated_at", format_rfc3339(item.updated_at)?))
            .await?;

        Ok(())
    }

    /// Get a memory item by ID.
    pub async fn get_memory_item(&self, id: &Id) -> StoreResult<Option<MemoryItem>> {
        debug!("Getting memory item: {id}");

        let mut result = self
            .db
            .query(
                r#"
                SELECT meta::id(id) AS record_id, item
                FROM type::thing("memory_item", $id)
                "#,
            )
            .bind(("id", id.to_string()))
            .await?;

        let records: Vec<MemoryItemRecord> = result.take(0)?;
        records
            .into_iter()
            .next()
            .map(MemoryItemRecord::into_memory_item)
            .transpose()
    }

    /// List memory items, newest updates first.
    pub async fn list_memory_items(
        &self,
        status: Option<MemoryStatus>,
        limit: Option<usize>,
    ) -> StoreResult<Vec<MemoryItem>> {
        debug!("Listing memory items (status: {status:?})");

        let mut query =
            "SELECT meta::id(id) AS record_id, item, updated_at FROM memory_item".to_string();
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

        decode_memory_items(result.take(0)?)
    }

    /// List memory items updated after a timestamp.
    pub async fn list_memory_items_updated_after(
        &self,
        timestamp: OffsetDateTime,
        limit: Option<usize>,
    ) -> StoreResult<Vec<MemoryItem>> {
        debug!("Listing memory items updated after {timestamp}");

        let mut items: Vec<_> = self
            .list_memory_items(None, None)
            .await?
            .into_iter()
            .filter(|item| item.updated_at > timestamp)
            .collect();
        items.sort_by_key(|item| item.updated_at);
        if let Some(limit) = limit {
            items.truncate(limit);
        }
        Ok(items)
    }

    /// List memory items needing review.
    pub async fn list_memory_items_needing_review(
        &self,
        now: OffsetDateTime,
        limit: Option<usize>,
    ) -> StoreResult<Vec<MemoryItem>> {
        let items = self.list_memory_items(None, None).await?;
        let mut filtered: Vec<_> = items
            .into_iter()
            .filter(|item| item.needs_review_at(now))
            .collect();
        filtered.sort_by_key(|item| item.updated_at);
        if let Some(limit) = limit {
            filtered.truncate(limit);
        }
        Ok(filtered)
    }

    /// Save a knowledge commit.
    pub async fn save_knowledge_commit(&self, commit: &KnowledgeCommit) -> StoreResult<()> {
        debug!("Saving knowledge commit: {}", commit.id);

        let commit_json = to_json(commit)?;
        self.db
            .query(
                r#"
                UPSERT type::thing("knowledge_commit", $id) SET
                    commit = $commit,
                    parent_id = $parent_id,
                    session_id = $session_id,
                    writer_harness = $writer_harness,
                    message = $message,
                    created_at = $created_at
                "#,
            )
            .bind(("id", commit.id.to_string()))
            .bind(("commit", commit_json))
            .bind(("parent_id", commit.parent_id.map(|id| id.to_string())))
            .bind(("session_id", commit.session_id.map(|id| id.to_string())))
            .bind(("writer_harness", commit.writer.harness.to_string()))
            .bind(("message", commit.message.clone()))
            .bind(("created_at", format_rfc3339(commit.created_at)?))
            .await?;

        Ok(())
    }

    /// Get a knowledge commit by ID.
    pub async fn get_knowledge_commit(&self, id: &Id) -> StoreResult<Option<KnowledgeCommit>> {
        debug!("Getting knowledge commit: {id}");

        let mut result = self
            .db
            .query(
                r#"
                SELECT meta::id(id) AS record_id, commit
                FROM type::thing("knowledge_commit", $id)
                "#,
            )
            .bind(("id", id.to_string()))
            .await?;

        let records: Vec<KnowledgeCommitRecord> = result.take(0)?;
        records
            .into_iter()
            .next()
            .map(KnowledgeCommitRecord::into_knowledge_commit)
            .transpose()
    }

    /// List knowledge commits, newest first.
    pub async fn list_knowledge_commits(
        &self,
        limit: Option<usize>,
    ) -> StoreResult<Vec<KnowledgeCommit>> {
        let mut query = r#"
            SELECT meta::id(id) AS record_id, commit, created_at
            FROM knowledge_commit
            ORDER BY created_at DESC
        "#
        .to_string();
        if let Some(limit) = limit {
            query.push_str(&format!(" LIMIT {limit}"));
        }

        let mut result = self.db.query(query).await?;
        decode_knowledge_commits(result.take(0)?)
    }

    /// List knowledge commits created after a timestamp.
    pub async fn list_knowledge_commits_after(
        &self,
        timestamp: OffsetDateTime,
        limit: Option<usize>,
    ) -> StoreResult<Vec<KnowledgeCommit>> {
        let mut commits: Vec<_> = self
            .list_knowledge_commits(None)
            .await?
            .into_iter()
            .filter(|commit| commit.created_at > timestamp)
            .collect();
        commits.sort_by_key(|commit| commit.created_at);
        if let Some(limit) = limit {
            commits.truncate(limit);
        }
        Ok(commits)
    }

    /// Get the newest knowledge commit.
    pub async fn latest_knowledge_commit(&self) -> StoreResult<Option<KnowledgeCommit>> {
        Ok(self
            .list_knowledge_commits(Some(1))
            .await?
            .into_iter()
            .next())
    }

    /// Latest memory item update timestamp.
    pub async fn latest_memory_timestamp(&self) -> StoreResult<Option<OffsetDateTime>> {
        let mut result = self
            .db
            .query("SELECT updated_at FROM memory_item ORDER BY updated_at DESC LIMIT 1")
            .await?;

        let records: Vec<LatestMemoryTimestampRecord> = result.take(0)?;
        records
            .into_iter()
            .next()
            .map(|record| record.updated_at.to_offset_datetime())
            .transpose()
    }

    /// Latest knowledge commit timestamp.
    pub async fn latest_commit_timestamp(&self) -> StoreResult<Option<OffsetDateTime>> {
        let mut result = self
            .db
            .query("SELECT created_at FROM knowledge_commit ORDER BY created_at DESC LIMIT 1")
            .await?;

        let records: Vec<LatestCommitTimestampRecord> = result.take(0)?;
        records
            .into_iter()
            .next()
            .map(|record| record.created_at.to_offset_datetime())
            .transpose()
    }
}

fn decode_memory_items(records: Vec<MemoryItemRecord>) -> StoreResult<Vec<MemoryItem>> {
    records
        .into_iter()
        .map(MemoryItemRecord::into_memory_item)
        .collect()
}

fn decode_knowledge_commits(
    records: Vec<KnowledgeCommitRecord>,
) -> StoreResult<Vec<KnowledgeCommit>> {
    records
        .into_iter()
        .map(KnowledgeCommitRecord::into_knowledge_commit)
        .collect()
}

fn format_rfc3339(value: OffsetDateTime) -> StoreResult<String> {
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

fn scope_key(item: &MemoryItem) -> String {
    match &item.scope {
        engram_core::memory::MemoryScope::Global => "global".to_string(),
        engram_core::memory::MemoryScope::User => "user".to_string(),
        engram_core::memory::MemoryScope::Project { project_name, .. } => {
            format!("project:{project_name}")
        }
        engram_core::memory::MemoryScope::Task { task_name, .. } => format!("task:{task_name}"),
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
        ClaimOrigin, EvidenceKind, EvidenceRef, Harness, MemoryChange, MemoryChangeType,
        MemoryKind, MemoryScope, ModelIdentity, WriterProvenance,
    };

    async fn setup_repo() -> MemoryRepo {
        let config = crate::StoreConfig::memory();
        let db = crate::connect_and_init(&config).await.unwrap();
        let repo = MemoryRepo::new(db);
        repo.init_schema().await.unwrap();
        repo
    }

    fn writer() -> WriterProvenance {
        WriterProvenance::agent(Harness::Codex, ModelIdentity::new("openai", "gpt-5.5"))
    }

    fn item(title: &str) -> MemoryItem {
        MemoryItem::new(
            MemoryKind::Decision,
            title,
            "Persist memory items as structured JSON plus query keys.",
            MemoryScope::project("engram"),
            ClaimOrigin::UserStated,
            writer(),
        )
        .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "test"))
    }

    #[tokio::test]
    async fn save_and_get_memory_item_round_trips_rich_fields() {
        let repo = setup_repo().await;
        let item = item("Memory Store MVP").with_tag("memory-os");

        repo.save_memory_item(&item).await.unwrap();
        let retrieved = repo.get_memory_item(&item.id).await.unwrap().unwrap();

        assert_eq!(retrieved.id, item.id);
        assert_eq!(retrieved.title, "Memory Store MVP");
        assert_eq!(retrieved.kind, MemoryKind::Decision);
        assert_eq!(retrieved.status, MemoryStatus::Active);
        assert_eq!(retrieved.tags, vec!["memory-os"]);
        assert_eq!(retrieved.evidence.len(), 1);
    }

    #[tokio::test]
    async fn list_memory_items_filters_by_status() {
        let repo = setup_repo().await;
        let active = item("Active memory");
        let review = item("Review memory").with_status(MemoryStatus::NeedsReview);

        repo.save_memory_item(&active).await.unwrap();
        repo.save_memory_item(&review).await.unwrap();

        let active_items = repo
            .list_memory_items(Some(MemoryStatus::Active), None)
            .await
            .unwrap();
        assert_eq!(active_items.len(), 1);
        assert_eq!(active_items[0].id, active.id);
    }

    #[tokio::test]
    async fn list_memory_items_updated_after_cursor_timestamp() {
        let repo = setup_repo().await;
        let before = OffsetDateTime::now_utc();
        let item = item("Changed after cursor");

        repo.save_memory_item(&item).await.unwrap();

        let changed = repo
            .list_memory_items_updated_after(before, None)
            .await
            .unwrap();
        assert_eq!(changed.len(), 1);
        assert_eq!(changed[0].id, item.id);
    }

    #[tokio::test]
    async fn save_and_list_knowledge_commits() {
        let repo = setup_repo().await;
        let item = item("Committed memory");
        let commit = KnowledgeCommit::new(writer(), "Capture committed memory").with_change(
            MemoryChange::new(
                MemoryChangeType::Added,
                "Committed memory",
                "Added a memory item.",
            )
            .with_item(item.id),
        );

        repo.save_knowledge_commit(&commit).await.unwrap();

        let retrieved = repo
            .get_knowledge_commit(&commit.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(retrieved.id, commit.id);
        assert_eq!(retrieved.change_count(), 1);

        let latest = repo.latest_knowledge_commit().await.unwrap().unwrap();
        assert_eq!(latest.id, commit.id);
    }
}
