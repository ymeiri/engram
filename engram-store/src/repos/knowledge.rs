//! Knowledge document repository for Layer 6: Document Intelligence.
//!
//! Handles persistence of KnowledgeDoc, FileSync, DocEvent, and DocAlias.

use crate::error::{StoreError, StoreResult};
use crate::Db;
use engram_core::id::Id;
use engram_core::knowledge::{
    DocAlias, DocEvent, DocStatus, DocType, FileSync, KnowledgeDoc, SyncStatus,
};
use serde::Deserialize;
use time::OffsetDateTime;
use tracing::{debug, info};

/// SurrealDB datetime representation.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum SurrealDateTime {
    /// ISO 8601 string format
    String(String),
    /// SurrealDB native datetime (array of integers)
    Native(serde_json::Value),
}

impl SurrealDateTime {
    fn to_offset_datetime(&self) -> OffsetDateTime {
        match self {
            SurrealDateTime::String(s) => {
                OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339)
                    .unwrap_or_else(|_| OffsetDateTime::now_utc())
            }
            SurrealDateTime::Native(v) => {
                // Try to parse the native SurrealDB format (array of year, month, day, hour, min, sec, nano, offset_h, offset_m)
                if let Some(arr) = v.as_array() {
                    if arr.len() >= 6 {
                        if let (
                            Some(year),
                            Some(month),
                            Some(day),
                            Some(hour),
                            Some(min),
                            Some(sec),
                        ) = (
                            arr[0].as_i64(),
                            arr[1].as_i64().and_then(|m| u8::try_from(m).ok()),
                            arr[2].as_i64().and_then(|d| u8::try_from(d).ok()),
                            arr[3].as_i64().and_then(|h| u8::try_from(h).ok()),
                            arr[4].as_i64().and_then(|m| u8::try_from(m).ok()),
                            arr[5].as_i64().and_then(|s| u8::try_from(s).ok()),
                        ) {
                            if let Ok(date) = time::Date::from_calendar_date(
                                year as i32,
                                time::Month::try_from(month).unwrap_or(time::Month::January),
                                day,
                            ) {
                                if let Ok(time) = time::Time::from_hms(hour, min, sec) {
                                    return OffsetDateTime::new_utc(date, time);
                                }
                            }
                        }
                    }
                }
                OffsetDateTime::now_utc()
            }
        }
    }
}

/// Internal storage representation of FileSync with ID field for queries.
#[derive(Debug, Clone, Deserialize)]
struct FileSyncRecordWithId {
    record_id: String,
    path: String,
    repo: String,
    doc_id: Option<String>,
    last_hash: String,
    last_modified: SurrealDateTime,
    last_synced: SurrealDateTime,
    sync_status: String, // Stored as string to avoid enum deserialization issues
    deleted_at: Option<SurrealDateTime>,
}

impl FileSyncRecordWithId {
    /// Convert to FileSync.
    fn into_file_sync(self, id: Id) -> FileSync {
        let sync_status = match self.sync_status.as_str() {
            "synced" => SyncStatus::Synced,
            "stale" => SyncStatus::Stale,
            "deleted" => SyncStatus::Deleted,
            "conflict" => SyncStatus::Conflict,
            "new" => SyncStatus::New,
            _ => SyncStatus::default(),
        };

        FileSync {
            id,
            path: self.path,
            repo: self.repo,
            doc_id: self.doc_id.and_then(|s| Id::parse(&s).ok()),
            last_hash: self.last_hash,
            last_modified: self.last_modified.to_offset_datetime(),
            last_synced: self.last_synced.to_offset_datetime(),
            sync_status,
            deleted_at: self.deleted_at.map(|dt| dt.to_offset_datetime()),
        }
    }
}

/// Internal storage representation of KnowledgeDoc with ID field for queries.
#[derive(Debug, Clone, Deserialize)]
struct KnowledgeDocRecordWithId {
    record_id: String,
    name: String,
    canonical_path: Option<String>,
    doc_type: String,
    status: String,
    owner: Option<String>,
    last_reviewed: Option<SurrealDateTime>,
    content_hash: String,
    #[serde(default)]
    tags: Vec<String>,
    content: String,
    summary: Option<String>,
    created_at: SurrealDateTime,
    updated_at: SurrealDateTime,
}

impl KnowledgeDocRecordWithId {
    /// Convert to KnowledgeDoc.
    fn into_knowledge_doc(self, id: Id) -> KnowledgeDoc {
        let doc_type = DocType::parse(&self.doc_type);
        let status = match self.status.as_str() {
            "active" => DocStatus::Active,
            "draft" => DocStatus::Draft,
            "archived" => DocStatus::Archived,
            "superseded" => DocStatus::Superseded,
            _ => DocStatus::default(),
        };

        KnowledgeDoc {
            id,
            name: self.name,
            canonical_path: self.canonical_path,
            doc_type,
            status,
            owner: self.owner,
            last_reviewed: self.last_reviewed.map(|dt| dt.to_offset_datetime()),
            content_hash: self.content_hash,
            tags: self.tags,
            content: self.content,
            summary: self.summary,
            created_at: self.created_at.to_offset_datetime(),
            updated_at: self.updated_at.to_offset_datetime(),
        }
    }
}

/// Internal storage representation of DocEvent.
#[derive(Debug, Clone, Deserialize)]
struct DocEventRecord {
    id: String,
    doc_id: String,
    event_type: String,
    details: serde_json::Value,
    actor: String,
    occurred_at: SurrealDateTime,
}

impl DocEventRecord {
    fn into_doc_event(self) -> DocEvent {
        use engram_core::knowledge::DocEventType;
        let event_type = match self.event_type.as_str() {
            "created" | "Created" => DocEventType::Created,
            "merged" | "Merged" => DocEventType::Merged,
            "superseded" | "Superseded" => DocEventType::Superseded,
            "archived" | "Archived" => DocEventType::Archived,
            "restored" | "Restored" => DocEventType::Restored,
            "alias_added" | "AliasAdded" => DocEventType::AliasAdded,
            "owner_changed" | "OwnerChanged" => DocEventType::OwnerChanged,
            "reviewed" | "Reviewed" => DocEventType::Reviewed,
            _ => DocEventType::Created, // Default fallback
        };

        DocEvent {
            id: Id::parse(&self.id).unwrap_or_else(|_| Id::new()),
            doc_id: Id::parse(&self.doc_id).unwrap_or_else(|_| Id::new()),
            event_type,
            details: self.details,
            actor: self.actor,
            occurred_at: self.occurred_at.to_offset_datetime(),
        }
    }
}

/// Internal storage representation of DocAlias.
#[derive(Debug, Clone, Deserialize)]
struct DocAliasRecord {
    alias: String,
    doc_id: String,
}

impl DocAliasRecord {
    fn into_doc_alias(self) -> DocAlias {
        DocAlias {
            alias: self.alias,
            doc_id: Id::parse(&self.doc_id).unwrap_or_else(|_| Id::new()),
        }
    }
}

/// Repository for knowledge document operations.
#[derive(Clone)]
pub struct KnowledgeRepo {
    db: Db,
}

impl KnowledgeRepo {
    /// Create a new knowledge repository.
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Initialize the schema for knowledge storage.
    ///
    /// # Errors
    ///
    /// Returns an error if schema creation fails.
    pub async fn init_schema(&self) -> StoreResult<()> {
        info!("Initializing knowledge schema");

        // Create knowledge_doc table (schemaless to avoid ID conflicts)
        self.db
            .query(
                r#"
                DEFINE TABLE knowledge_doc SCHEMALESS;
                DEFINE INDEX idx_knowledge_doc_name ON knowledge_doc FIELDS name;
                DEFINE INDEX idx_knowledge_doc_hash ON knowledge_doc FIELDS content_hash;
                DEFINE INDEX idx_knowledge_doc_path ON knowledge_doc FIELDS canonical_path;
                "#,
            )
            .await?;

        // Create file_sync table (schemaless to avoid ID conflicts)
        self.db
            .query(
                r#"
                DEFINE TABLE file_sync SCHEMALESS;
                DEFINE INDEX idx_file_sync_path ON file_sync FIELDS path, repo UNIQUE;
                DEFINE INDEX idx_file_sync_hash ON file_sync FIELDS last_hash;
                DEFINE INDEX idx_file_sync_doc ON file_sync FIELDS doc_id;
                "#,
            )
            .await?;

        // Create doc_event table (schemaless to avoid ID conflicts)
        self.db
            .query(
                r#"
                DEFINE TABLE doc_event SCHEMALESS;
                DEFINE INDEX idx_doc_event_doc ON doc_event FIELDS doc_id;
                "#,
            )
            .await?;

        // Create doc_alias table
        self.db
            .query(
                r#"
                DEFINE TABLE doc_alias SCHEMAFULL;
                DEFINE FIELD alias ON doc_alias TYPE string;
                DEFINE FIELD doc_id ON doc_alias TYPE string;
                DEFINE INDEX idx_doc_alias_alias ON doc_alias FIELDS alias UNIQUE;
                "#,
            )
            .await?;

        info!("Knowledge schema initialized");
        Ok(())
    }

    // ==================== KnowledgeDoc Operations ====================

    /// Save a knowledge document.
    pub async fn save_doc(&self, doc: &KnowledgeDoc) -> StoreResult<()> {
        debug!("Saving knowledge doc: {}", doc.name);

        // Use raw query to avoid SurrealDB SDK ID serialization conflicts
        let doc_type_str = doc.doc_type.to_string();
        let status_str = serde_json::to_string(&doc.status)
            .map_err(StoreError::Serialization)?
            .trim_matches('"')
            .to_string();

        self.db
            .query(
                r#"
                UPSERT type::thing("knowledge_doc", $id) SET
                    name = $name,
                    canonical_path = $canonical_path,
                    doc_type = $doc_type,
                    status = $status,
                    owner = $owner,
                    last_reviewed = $last_reviewed,
                    content_hash = $content_hash,
                    tags = $tags,
                    content = $content,
                    summary = $summary,
                    created_at = $created_at,
                    updated_at = $updated_at
                "#,
            )
            .bind(("id", doc.id.to_string()))
            .bind(("name", doc.name.clone()))
            .bind(("canonical_path", doc.canonical_path.clone()))
            .bind(("doc_type", doc_type_str))
            .bind(("status", status_str))
            .bind(("owner", doc.owner.clone()))
            .bind(("last_reviewed", doc.last_reviewed))
            .bind(("content_hash", doc.content_hash.clone()))
            .bind(("tags", doc.tags.clone()))
            .bind(("content", doc.content.clone()))
            .bind(("summary", doc.summary.clone()))
            .bind(("created_at", doc.created_at))
            .bind(("updated_at", doc.updated_at))
            .await?;

        Ok(())
    }

    /// Get a knowledge document by ID.
    pub async fn get_doc(&self, id: &Id) -> StoreResult<KnowledgeDoc> {
        let mut result = self
            .db
            .query(r#"SELECT meta::id(id) as record_id, name, canonical_path, doc_type, status, owner, last_reviewed, content_hash, tags, content, summary, created_at, updated_at FROM type::thing("knowledge_doc", $id)"#)
            .bind(("id", id.to_string()))
            .await?;

        let records: Vec<KnowledgeDocRecordWithId> = result.take(0)?;
        records
            .into_iter()
            .next()
            .and_then(|r| {
                Id::parse(&r.record_id)
                    .ok()
                    .map(|id| r.into_knowledge_doc(id))
            })
            .ok_or_else(|| StoreError::NotFound(format!("KnowledgeDoc {id}")))
    }

    /// Find a document by name.
    pub async fn find_doc_by_name(&self, name: &str) -> StoreResult<Option<KnowledgeDoc>> {
        let mut result = self
            .db
            .query("SELECT meta::id(id) as record_id, name, canonical_path, doc_type, status, owner, last_reviewed, content_hash, tags, content, summary, created_at, updated_at FROM knowledge_doc WHERE name = $name LIMIT 1")
            .bind(("name", name.to_string()))
            .await?;

        let records: Vec<KnowledgeDocRecordWithId> = result.take(0)?;
        Ok(records.into_iter().next().and_then(|r| {
            Id::parse(&r.record_id)
                .ok()
                .map(|id| r.into_knowledge_doc(id))
        }))
    }

    /// Find a document by canonical path.
    pub async fn find_doc_by_path(&self, path: &str) -> StoreResult<Option<KnowledgeDoc>> {
        let mut result = self
            .db
            .query("SELECT meta::id(id) as record_id, name, canonical_path, doc_type, status, owner, last_reviewed, content_hash, tags, content, summary, created_at, updated_at FROM knowledge_doc WHERE canonical_path = $path LIMIT 1")
            .bind(("path", path.to_string()))
            .await?;

        let records: Vec<KnowledgeDocRecordWithId> = result.take(0)?;
        Ok(records.into_iter().next().and_then(|r| {
            Id::parse(&r.record_id)
                .ok()
                .map(|id| r.into_knowledge_doc(id))
        }))
    }

    /// Find documents by content hash (for duplicate detection).
    pub async fn find_docs_by_hash(&self, hash: &str) -> StoreResult<Vec<KnowledgeDoc>> {
        let mut result = self
            .db
            .query("SELECT meta::id(id) as record_id, name, canonical_path, doc_type, status, owner, last_reviewed, content_hash, tags, content, summary, created_at, updated_at FROM knowledge_doc WHERE content_hash = $hash")
            .bind(("hash", hash.to_string()))
            .await?;

        let records: Vec<KnowledgeDocRecordWithId> = result.take(0)?;
        let docs = records
            .into_iter()
            .filter_map(|r| {
                Id::parse(&r.record_id)
                    .ok()
                    .map(|id| r.into_knowledge_doc(id))
            })
            .collect();
        Ok(docs)
    }

    /// List all knowledge documents.
    pub async fn list_docs(&self) -> StoreResult<Vec<KnowledgeDoc>> {
        let mut result = self
            .db
            .query(
                "SELECT *, meta::id(id) AS record_id FROM knowledge_doc ORDER BY updated_at DESC",
            )
            .await?;

        let records: Vec<KnowledgeDocRecordWithId> = result.take(0)?;
        let docs = records
            .into_iter()
            .filter_map(|r| {
                Id::parse(&r.record_id)
                    .ok()
                    .map(|id| r.into_knowledge_doc(id))
            })
            .collect();
        Ok(docs)
    }

    /// List documents by type.
    pub async fn list_docs_by_type(&self, doc_type: &DocType) -> StoreResult<Vec<KnowledgeDoc>> {
        let type_str = doc_type.to_string();
        let mut result = self
            .db
            .query("SELECT meta::id(id) as record_id, name, canonical_path, doc_type, status, owner, last_reviewed, content_hash, tags, content, summary, created_at, updated_at FROM knowledge_doc WHERE doc_type = $doc_type ORDER BY name")
            .bind(("doc_type", type_str))
            .await?;

        let records: Vec<KnowledgeDocRecordWithId> = result.take(0)?;
        let docs = records
            .into_iter()
            .filter_map(|r| {
                Id::parse(&r.record_id)
                    .ok()
                    .map(|id| r.into_knowledge_doc(id))
            })
            .collect();
        Ok(docs)
    }

    /// Delete a knowledge document.
    pub async fn delete_doc(&self, id: &Id) -> StoreResult<()> {
        debug!("Deleting knowledge doc: {id}");

        // Delete associated aliases
        self.db
            .query("DELETE doc_alias WHERE doc_id = $doc_id")
            .bind(("doc_id", id.to_string()))
            .await?;

        // Delete the document using raw query to avoid deserialization issues
        self.db
            .query(r#"DELETE type::thing("knowledge_doc", $id)"#)
            .bind(("id", id.to_string()))
            .await?;

        Ok(())
    }

    // ==================== FileSync Operations ====================

    /// Save a file sync record.
    pub async fn save_file_sync(&self, sync: &FileSync) -> StoreResult<()> {
        debug!("Saving file sync: {}", sync.path);

        // Use raw query to avoid SurrealDB SDK ID serialization conflicts
        // All values must be owned for 'static requirement
        let sync_status_str = serde_json::to_string(&sync.sync_status)
            .map_err(StoreError::Serialization)?
            .trim_matches('"')
            .to_string();

        self.db
            .query(
                r#"
                UPSERT type::thing("file_sync", $id) SET
                    path = $path,
                    repo = $repo,
                    doc_id = $doc_id,
                    last_hash = $last_hash,
                    last_modified = $last_modified,
                    last_synced = $last_synced,
                    sync_status = $sync_status,
                    deleted_at = $deleted_at
                "#,
            )
            .bind(("id", sync.id.to_string()))
            .bind(("path", sync.path.clone()))
            .bind(("repo", sync.repo.clone()))
            .bind(("doc_id", sync.doc_id.map(|d| d.to_string())))
            .bind(("last_hash", sync.last_hash.clone()))
            .bind(("last_modified", sync.last_modified))
            .bind(("last_synced", sync.last_synced))
            .bind(("sync_status", sync_status_str))
            .bind(("deleted_at", sync.deleted_at))
            .await?;

        Ok(())
    }

    /// Get a file sync by ID.
    pub async fn get_file_sync(&self, id: &Id) -> StoreResult<FileSync> {
        let mut result = self
            .db
            .query(r#"SELECT meta::id(id) as record_id, path, repo, doc_id, last_hash, last_modified, last_synced, sync_status, deleted_at FROM type::thing("file_sync", $id)"#)
            .bind(("id", id.to_string()))
            .await?;

        let records: Vec<FileSyncRecordWithId> = result.take(0)?;
        records
            .into_iter()
            .next()
            .and_then(|r| Id::parse(&r.record_id).ok().map(|id| r.into_file_sync(id)))
            .ok_or_else(|| StoreError::NotFound(format!("FileSync {id}")))
    }

    /// Find a file sync by path and repo.
    pub async fn find_file_sync(&self, path: &str, repo: &str) -> StoreResult<Option<FileSync>> {
        let mut result = self
            .db
            .query("SELECT *, meta::id(id) AS record_id FROM file_sync WHERE path = $path AND repo = $repo LIMIT 1")
            .bind(("path", path.to_string()))
            .bind(("repo", repo.to_string()))
            .await?;

        let records: Vec<FileSyncRecordWithId> = result.take(0)?;
        Ok(records
            .into_iter()
            .next()
            .and_then(|r| Id::parse(&r.record_id).ok().map(|id| r.into_file_sync(id))))
    }

    /// Find file syncs by hash (for duplicate detection).
    pub async fn find_file_syncs_by_hash(&self, hash: &str) -> StoreResult<Vec<FileSync>> {
        let mut result = self
            .db
            .query("SELECT *, meta::id(id) AS record_id FROM file_sync WHERE last_hash = $hash")
            .bind(("hash", hash.to_string()))
            .await?;

        let records: Vec<FileSyncRecordWithId> = result.take(0)?;
        let syncs = records
            .into_iter()
            .filter_map(|r| Id::parse(&r.record_id).ok().map(|id| r.into_file_sync(id)))
            .collect();
        Ok(syncs)
    }

    /// List all file syncs.
    pub async fn list_file_syncs(&self) -> StoreResult<Vec<FileSync>> {
        let mut result = self
            .db
            .query("SELECT *, meta::id(id) AS record_id FROM file_sync ORDER BY path")
            .await?;

        let records: Vec<FileSyncRecordWithId> = result.take(0)?;
        let syncs = records
            .into_iter()
            .filter_map(|r| Id::parse(&r.record_id).ok().map(|id| r.into_file_sync(id)))
            .collect();
        Ok(syncs)
    }

    /// List file syncs by status.
    pub async fn list_file_syncs_by_status(
        &self,
        status: SyncStatus,
    ) -> StoreResult<Vec<FileSync>> {
        let status_str = serde_json::to_string(&status)
            .map_err(StoreError::Serialization)?
            .trim_matches('"')
            .to_string();

        let mut result = self
            .db
            .query("SELECT *, meta::id(id) AS record_id FROM file_sync WHERE sync_status = $status ORDER BY path")
            .bind(("status", status_str))
            .await?;

        let records: Vec<FileSyncRecordWithId> = result.take(0)?;
        let syncs = records
            .into_iter()
            .filter_map(|r| Id::parse(&r.record_id).ok().map(|id| r.into_file_sync(id)))
            .collect();
        Ok(syncs)
    }

    /// List file syncs for a repo.
    pub async fn list_file_syncs_for_repo(&self, repo: &str) -> StoreResult<Vec<FileSync>> {
        let mut result = self
            .db
            .query("SELECT *, meta::id(id) AS record_id FROM file_sync WHERE repo = $repo ORDER BY path")
            .bind(("repo", repo.to_string()))
            .await?;

        let records: Vec<FileSyncRecordWithId> = result.take(0)?;
        let syncs = records
            .into_iter()
            .filter_map(|r| Id::parse(&r.record_id).ok().map(|id| r.into_file_sync(id)))
            .collect();
        Ok(syncs)
    }

    /// Delete a file sync.
    pub async fn delete_file_sync(&self, id: &Id) -> StoreResult<()> {
        // Use raw query to avoid deserialization issues
        self.db
            .query(r#"DELETE type::thing("file_sync", $id)"#)
            .bind(("id", id.to_string()))
            .await?;
        Ok(())
    }

    // ==================== DocEvent Operations ====================

    /// Save a document event.
    pub async fn save_event(&self, event: &DocEvent) -> StoreResult<()> {
        debug!(
            "Saving doc event: {:?} for {}",
            event.event_type, event.doc_id
        );

        // Use raw query to avoid SurrealDB SDK ID serialization conflicts
        let event_type_str = serde_json::to_string(&event.event_type)
            .map_err(StoreError::Serialization)?
            .trim_matches('"')
            .to_string();

        self.db
            .query(
                r#"
                UPSERT type::thing("doc_event", $id) SET
                    doc_id = $doc_id,
                    event_type = $event_type,
                    details = $details,
                    actor = $actor,
                    occurred_at = $occurred_at
                "#,
            )
            .bind(("id", event.id.to_string()))
            .bind(("doc_id", event.doc_id.to_string()))
            .bind(("event_type", event_type_str))
            .bind(("details", event.details.clone()))
            .bind(("actor", event.actor.clone()))
            .bind(("occurred_at", event.occurred_at))
            .await?;

        Ok(())
    }

    /// List events for a document.
    pub async fn list_events_for_doc(&self, doc_id: &Id) -> StoreResult<Vec<DocEvent>> {
        let mut result = self
            .db
            .query("SELECT meta::id(id) as id, doc_id, event_type, details, actor, occurred_at FROM doc_event WHERE doc_id = $doc_id ORDER BY occurred_at DESC")
            .bind(("doc_id", doc_id.to_string()))
            .await?;

        let records: Vec<DocEventRecord> = result.take(0)?;
        Ok(records.into_iter().map(|r| r.into_doc_event()).collect())
    }

    // ==================== DocAlias Operations ====================

    /// Save a document alias.
    pub async fn save_alias(&self, alias: &DocAlias) -> StoreResult<()> {
        debug!("Saving doc alias: {} -> {}", alias.alias, alias.doc_id);

        // Use raw query to avoid SurrealDB SDK ID conflicts
        self.db
            .query(
                r#"
                UPSERT type::thing("doc_alias", $alias_key) SET
                    alias = $alias,
                    doc_id = $doc_id
                "#,
            )
            .bind(("alias_key", alias.alias.clone()))
            .bind(("alias", alias.alias.clone()))
            .bind(("doc_id", alias.doc_id.to_string()))
            .await?;

        Ok(())
    }

    /// Find document by alias.
    pub async fn find_doc_by_alias(&self, alias: &str) -> StoreResult<Option<KnowledgeDoc>> {
        let mut result = self
            .db
            .query("SELECT alias, doc_id FROM doc_alias WHERE alias = $alias LIMIT 1")
            .bind(("alias", alias.to_string()))
            .await?;

        let records: Vec<DocAliasRecord> = result.take(0)?;
        if let Some(record) = records.into_iter().next() {
            let doc_alias = record.into_doc_alias();
            return Ok(Some(self.get_doc(&doc_alias.doc_id).await?));
        }
        Ok(None)
    }

    /// List all aliases for a document.
    pub async fn list_aliases_for_doc(&self, doc_id: &Id) -> StoreResult<Vec<DocAlias>> {
        let mut result = self
            .db
            .query("SELECT alias, doc_id FROM doc_alias WHERE doc_id = $doc_id")
            .bind(("doc_id", doc_id.to_string()))
            .await?;

        let records: Vec<DocAliasRecord> = result.take(0)?;
        Ok(records.into_iter().map(|r| r.into_doc_alias()).collect())
    }

    /// Delete an alias.
    pub async fn delete_alias(&self, alias: &str) -> StoreResult<()> {
        // Use raw query to avoid deserialization issues
        self.db
            .query(r#"DELETE type::thing("doc_alias", $alias)"#)
            .bind(("alias", alias.to_string()))
            .await?;
        Ok(())
    }

    // ==================== Statistics ====================

    /// Get knowledge statistics.
    pub async fn stats(&self) -> StoreResult<KnowledgeStats> {
        let mut result = self
            .db
            .query(
                r#"
                SELECT count() as count FROM knowledge_doc GROUP ALL;
                SELECT count() as count FROM file_sync GROUP ALL;
                SELECT count() as count FROM doc_alias GROUP ALL;
                "#,
            )
            .await?;

        #[derive(Debug, Deserialize)]
        struct CountResult {
            count: i64,
        }

        let doc_counts: Vec<CountResult> = result.take(0)?;
        let sync_counts: Vec<CountResult> = result.take(1)?;
        let alias_counts: Vec<CountResult> = result.take(2)?;

        Ok(KnowledgeStats {
            doc_count: doc_counts.first().map(|c| c.count as u64).unwrap_or(0),
            file_sync_count: sync_counts.first().map(|c| c.count as u64).unwrap_or(0),
            alias_count: alias_counts.first().map(|c| c.count as u64).unwrap_or(0),
        })
    }
}

/// Statistics about the knowledge store.
#[derive(Debug, Clone)]
pub struct KnowledgeStats {
    /// Number of knowledge documents.
    pub doc_count: u64,
    /// Number of file sync records.
    pub file_sync_count: u64,
    /// Number of aliases.
    pub alias_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_stats_default() {
        let stats = KnowledgeStats {
            doc_count: 0,
            file_sync_count: 0,
            alias_count: 0,
        };
        assert_eq!(stats.doc_count, 0);
    }
}
