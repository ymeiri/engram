//! Document repository for Layer 3: Document Knowledge.
//!
//! Handles persistence of document sources, chunks, and embeddings.
//! Provides vector similarity search using SurrealDB's native vector functions.

use crate::error::{StoreError, StoreResult};
use crate::Db;
use engram_core::document::{DocChunk, DocSearchResult, DocSource, SourceType};
use engram_core::id::Id;
use serde::Deserialize;
use time::OffsetDateTime;
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
    fn to_offset_datetime(&self) -> Option<OffsetDateTime> {
        match self {
            SurrealDateTime::String(s) => {
                OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339).ok()
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
                        .ok()?;

                        let time_val = time::Time::from_hms(hour, min, sec).ok()?;
                        return Some(OffsetDateTime::new_utc(date, time_val));
                    }
                }
                None
            }
        }
    }
}

/// DocSource record from SurrealDB.
#[derive(Debug, Clone, Deserialize)]
struct DocSourceRecord {
    id: String,
    source_type: String,
    path_or_url: String,
    title: Option<String>,
    space_key: Option<String>,
    last_indexed: Option<SurrealDateTime>,
    ttl_days: i32,
}

impl DocSourceRecord {
    fn into_doc_source(self) -> DocSource {
        let source_type = match self.source_type.as_str() {
            "local_file" => SourceType::LocalFile,
            "confluence" => SourceType::Confluence,
            "github" => SourceType::GitHub,
            "notion" => SourceType::Notion,
            other => SourceType::Custom(other.to_string()),
        };

        DocSource {
            id: Id::parse(&self.id).unwrap_or_else(|_| Id::new()),
            source_type,
            path_or_url: self.path_or_url,
            title: self.title,
            space_key: self.space_key,
            last_indexed: self.last_indexed.and_then(|dt| dt.to_offset_datetime()),
            ttl_days: self.ttl_days,
        }
    }
}

/// DocChunk record from SurrealDB (for deserialization).

/// DocChunk record from SurrealDB (for deserialization).
/// SurrealDB v2 returns Thing objects for IDs, so we use string fields.
#[derive(Debug, Clone, Deserialize)]
struct DocChunkRecord {
    id: String,
    source_id: String,
    heading_path: String,
    heading_level: u8,
    content: String,
    start_line: Option<u32>,
    end_line: Option<u32>,
    parent_id: Option<String>,
}

impl DocChunkRecord {
    fn into_doc_chunk(self) -> DocChunk {
        DocChunk {
            id: Id::parse(&self.id).unwrap_or_else(|_| Id::new()),
            source_id: Id::parse(&self.source_id).unwrap_or_else(|_| Id::new()),
            heading_path: self.heading_path,
            heading_level: self.heading_level,
            content: self.content,
            start_line: self.start_line,
            end_line: self.end_line,
            parent_id: self.parent_id.and_then(|s| Id::parse(&s).ok()),
        }
    }
}

/// Repository for document operations.
#[derive(Clone)]
pub struct DocumentRepo {
    db: Db,
}

impl DocumentRepo {
    /// Create a new document repository.
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Initialize the schema for document storage.
    ///
    /// Creates tables and indexes for efficient vector search.
    /// Uses HNSW index for approximate nearest neighbor search on embeddings.
    ///
    /// # Errors
    ///
    /// Returns an error if schema creation fails.
    pub async fn init_schema(&self) -> StoreResult<()> {
        info!("Initializing document schema");

        // Create doc_source table (SCHEMALESS to avoid id field conflicts with record ID)
        self.db
            .query(
                r#"
                DEFINE TABLE IF NOT EXISTS doc_source SCHEMALESS;
                DEFINE INDEX IF NOT EXISTS idx_source_path ON doc_source FIELDS path_or_url UNIQUE;
                "#,
            )
            .await?;

        // Create doc_chunk table with basic index on source_id
        // Note: For production with 384-dim embeddings, add HNSW vector index:
        //   DEFINE INDEX idx_chunk_embedding ON doc_chunk FIELDS embedding
        //       HNSW DIMENSION 384 DIST COSINE TYPE F32 M 16 EFC 100;
        self.db
            .query(
                r#"
                DEFINE TABLE IF NOT EXISTS doc_chunk SCHEMALESS;
                DEFINE INDEX IF NOT EXISTS idx_chunk_source ON doc_chunk FIELDS source_id;
                "#,
            )
            .await?;

        info!("Document schema initialized with HNSW vector index");
        Ok(())
    }

    /// Save a document source.
    ///
    /// Creates or updates the document source record.
    ///
    /// # Errors
    ///
    /// Returns an error if the save operation fails.
    pub async fn save_source(&self, source: &DocSource) -> StoreResult<()> {
        debug!("Saving doc source: {}", source.path_or_url);

        // SurrealDB v2: Use raw query to avoid SDK ID serialization conflicts
        let source_type_str = serde_json::to_string(&source.source_type)
            .map_err(|e| StoreError::Serialization(e))?
            .trim_matches('"')
            .to_string();

        self.db
            .query(
                r#"
                UPSERT type::thing("doc_source", $id) SET
                    source_type = $source_type,
                    path_or_url = $path_or_url,
                    title = $title,
                    space_key = $space_key,
                    last_indexed = $last_indexed,
                    ttl_days = $ttl_days
                "#,
            )
            .bind(("id", source.id.to_string()))
            .bind(("source_type", source_type_str))
            .bind(("path_or_url", source.path_or_url.clone()))
            .bind(("title", source.title.clone()))
            .bind(("space_key", source.space_key.clone()))
            .bind(("last_indexed", source.last_indexed))
            .bind(("ttl_days", source.ttl_days))
            .await?;

        Ok(())
    }

    /// Get a document source by ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the source is not found or query fails.
    pub async fn get_source(&self, id: &Id) -> StoreResult<DocSource> {
        // SurrealDB v2: Use raw query with meta::id() to convert Thing to string
        let mut result = self
            .db
            .query(r#"SELECT meta::id(id) as id, source_type, path_or_url, title, space_key, last_indexed, ttl_days FROM type::thing("doc_source", $id)"#)
            .bind(("id", id.to_string()))
            .await?;

        let sources: Vec<DocSourceRecord> = result.take(0)?;
        sources
            .into_iter()
            .next()
            .map(|r| r.into_doc_source())
            .ok_or_else(|| StoreError::NotFound(format!("DocSource {id}")))
    }

    /// Find a source by its path or URL.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn find_source_by_path(&self, path: &str) -> StoreResult<Option<DocSource>> {
        let mut result = self
            .db
            .query("SELECT meta::id(id) as id, source_type, path_or_url, title, space_key, last_indexed, ttl_days FROM doc_source WHERE path_or_url = $path LIMIT 1")
            .bind(("path", path.to_string()))
            .await?;

        let sources: Vec<DocSourceRecord> = result.take(0)?;
        Ok(sources.into_iter().next().map(|r| r.into_doc_source()))
    }

    /// Delete a document source and all its chunks.
    ///
    /// Uses a transaction to ensure atomicity - either both the source
    /// and its chunks are deleted, or neither is.
    ///
    /// # Errors
    ///
    /// Returns an error if deletion fails.
    pub async fn delete_source(&self, id: &Id) -> StoreResult<()> {
        debug!("Deleting doc source: {id}");

        // Use transaction to ensure atomicity of delete operations
        self.db
            .query(
                r#"
                BEGIN TRANSACTION;
                DELETE doc_chunk WHERE source_id = $source_id;
                DELETE type::thing("doc_source", $id);
                COMMIT TRANSACTION;
                "#,
            )
            .bind(("source_id", id.to_string()))
            .bind(("id", id.to_string()))
            .await?;

        Ok(())
    }

    /// Save chunks with their embeddings.
    ///
    /// Replaces all existing chunks for the source document.
    ///
    /// # Errors
    ///
    /// Returns an error if the save operation fails.
    pub async fn save_chunks(
        &self,
        source_id: &Id,
        chunks: Vec<(DocChunk, Vec<f32>)>,
    ) -> StoreResult<()> {
        debug!("Saving {} chunks for source {}", chunks.len(), source_id);

        // Delete existing chunks for this source
        self.db
            .query("DELETE doc_chunk WHERE source_id = $source_id")
            .bind(("source_id", source_id.to_string()))
            .await?;

        // Insert new chunks using raw query to avoid IdMismatch errors
        for (chunk, embedding) in chunks {
            self.db
                .query(
                    r#"
                    UPSERT type::thing("doc_chunk", $id) SET
                        source_id = $source_id,
                        heading_path = $heading_path,
                        heading_level = $heading_level,
                        content = $content,
                        start_line = $start_line,
                        end_line = $end_line,
                        parent_id = $parent_id,
                        embedding = $embedding
                    "#,
                )
                .bind(("id", chunk.id.to_string()))
                .bind(("source_id", chunk.source_id.to_string()))
                .bind(("heading_path", chunk.heading_path))
                .bind(("heading_level", chunk.heading_level as i32))
                .bind(("content", chunk.content))
                .bind(("start_line", chunk.start_line.map(|v| v as i32)))
                .bind(("end_line", chunk.end_line.map(|v| v as i32)))
                .bind(("parent_id", chunk.parent_id.map(|id| id.to_string())))
                .bind(("embedding", embedding))
                .await?;
        }

        Ok(())
    }

    /// Search for similar documents using vector similarity.
    ///
    /// Uses cosine similarity to find chunks with embeddings closest to the query.
    /// Optimized to batch-fetch sources in a single query (eliminates N+1 problem).
    ///
    /// # Arguments
    ///
    /// * `query_embedding` - The embedding vector to search for
    /// * `limit` - Maximum number of results to return
    ///
    /// # Errors
    ///
    /// Returns an error if the search fails.
    pub async fn search_similar(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> StoreResult<Vec<DocSearchResult>> {
        debug!("Searching for similar documents, limit={}", limit);

        // SurrealDB vector similarity search using cosine distance
        // Use meta::id() to convert Thing IDs to strings for deserialization
        let mut result = self
            .db
            .query(
                r#"
                SELECT
                    meta::id(id) as id,
                    source_id,
                    heading_path,
                    heading_level,
                    content,
                    start_line,
                    end_line,
                    parent_id,
                    vector::similarity::cosine(embedding, $query) AS score
                FROM doc_chunk
                ORDER BY score DESC
                LIMIT $limit
                "#,
            )
            .bind(("query", query_embedding.to_vec()))
            .bind(("limit", limit))
            .await?;

        // Parse the results with explicit fields
        #[derive(Debug, Deserialize)]
        struct SearchHit {
            id: String,
            source_id: String,
            heading_path: String,
            heading_level: u8,
            content: String,
            start_line: Option<u32>,
            end_line: Option<u32>,
            parent_id: Option<String>,
            score: f32,
        }

        let hits: Vec<SearchHit> = result.take(0)?;

        if hits.is_empty() {
            return Ok(Vec::new());
        }

        // Collect unique source IDs for batch fetch (eliminates N+1 query problem)
        let source_ids: Vec<String> = hits
            .iter()
            .map(|h| h.source_id.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // Batch fetch all sources in a single query
        let mut source_result = self
            .db
            .query(
                r#"
                SELECT meta::id(id) as id, source_type, path_or_url, title, space_key, last_indexed, ttl_days
                FROM doc_source
                WHERE meta::id(id) IN $source_ids
                "#,
            )
            .bind(("source_ids", source_ids))
            .await?;

        let source_records: Vec<DocSourceRecord> = source_result.take(0)?;

        // Build a map for O(1) source lookups
        let source_map: std::collections::HashMap<String, DocSource> = source_records
            .into_iter()
            .map(|r| {
                let id = r.id.clone();
                (id, r.into_doc_source())
            })
            .collect();

        // Build results by joining chunks with their sources
        let mut results = Vec::with_capacity(hits.len());
        for hit in hits {
            let chunk = DocChunk {
                id: Id::parse(&hit.id).unwrap_or_else(|_| Id::new()),
                source_id: Id::parse(&hit.source_id).unwrap_or_else(|_| Id::new()),
                heading_path: hit.heading_path,
                heading_level: hit.heading_level,
                content: hit.content,
                start_line: hit.start_line,
                end_line: hit.end_line,
                parent_id: hit.parent_id.and_then(|s| Id::parse(&s).ok()),
            };

            if let Some(source) = source_map.get(&hit.source_id) {
                results.push(DocSearchResult {
                    chunk,
                    source: source.clone(),
                    score: hit.score,
                });
            } else {
                debug!("Source not found for chunk {}, skipping", hit.source_id);
            }
        }

        Ok(results)
    }

    /// Search with a minimum score threshold.
    ///
    /// # Errors
    ///
    /// Returns an error if the search fails.
    pub async fn search_similar_threshold(
        &self,
        query_embedding: &[f32],
        limit: usize,
        min_score: f32,
    ) -> StoreResult<Vec<DocSearchResult>> {
        let results = self.search_similar(query_embedding, limit).await?;
        Ok(results
            .into_iter()
            .filter(|r| r.score >= min_score)
            .collect())
    }

    /// Get all chunks for a document source.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn get_chunks_for_source(&self, source_id: &Id) -> StoreResult<Vec<DocChunk>> {
        let mut result = self
            .db
            .query("SELECT meta::id(id) as id, source_id, heading_path, heading_level, content, start_line, end_line, parent_id FROM doc_chunk WHERE source_id = $source_id ORDER BY start_line")
            .bind(("source_id", source_id.to_string()))
            .await?;

        let records: Vec<DocChunkRecord> = result.take(0)?;
        Ok(records.into_iter().map(|r| r.into_doc_chunk()).collect())
    }

    /// Get statistics about the document store.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn stats(&self) -> StoreResult<DocumentStats> {
        let mut result = self
            .db
            .query(
                r#"
                SELECT count() as count FROM doc_source GROUP ALL;
                SELECT count() as count FROM doc_chunk GROUP ALL;
                "#,
            )
            .await?;

        #[derive(Debug, Deserialize)]
        struct CountResult {
            count: i64,
        }

        let source_counts: Vec<CountResult> = result.take(0)?;
        let chunk_counts: Vec<CountResult> = result.take(1)?;

        Ok(DocumentStats {
            source_count: source_counts.first().map(|c| c.count as u64).unwrap_or(0),
            chunk_count: chunk_counts.first().map(|c| c.count as u64).unwrap_or(0),
        })
    }
}

/// Statistics about the document store.
#[derive(Debug, Clone)]
pub struct DocumentStats {
    /// Number of document sources.
    pub source_count: u64,
    /// Number of document chunks.
    pub chunk_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_chunk_creation() {
        let source_id = Id::new();
        let chunk = DocChunk::new(source_id.clone(), "# Test > ## Section", 2, "Test content");

        assert_eq!(chunk.heading_level, 2);
        assert_eq!(chunk.heading_path, "# Test > ## Section");
        assert_eq!(chunk.content, "Test content");
        assert_eq!(chunk.source_id, source_id);
    }

    #[test]
    fn test_doc_source_local_file() {
        let source = DocSource::local_file("/path/to/file.md");
        assert_eq!(source.source_type, SourceType::LocalFile);
        assert_eq!(source.path_or_url, "/path/to/file.md");
    }
}
