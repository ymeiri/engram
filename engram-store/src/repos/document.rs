//! Document repository for Layer 3: Document Knowledge.
//!
//! Handles persistence of document sources, chunks, and embeddings.
//! Provides vector similarity search using SurrealDB's native vector functions.

use crate::error::{StoreError, StoreResult};
use crate::Db;
use engram_core::document::{DocChunk, DocSearchResult, DocSource, SourceType};
use engram_core::id::Id;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use time::format_description::well_known::Rfc3339;
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
            .await?
            .check()?;

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
            .await?
            .check()?;

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
            .map_err(StoreError::Serialization)?
            .trim_matches('"')
            .to_string();
        let last_indexed = source
            .last_indexed
            .map(|dt| dt.format(&Rfc3339))
            .transpose()
            .map_err(|e| StoreError::Deserialization(format!("invalid last_indexed: {e}")))?;

        self.db
            .query(
                r#"
                UPSERT type::thing("doc_source", $id) SET
                    source_type = $source_type,
                    path_or_url = $path_or_url,
                    title = $title,
                    space_key = $space_key,
                    last_indexed = IF $last_indexed IS NONE THEN NONE ELSE type::datetime($last_indexed) END,
                    ttl_days = $ttl_days
                "#,
            )
            .bind(("id", source.id.to_string()))
            .bind(("source_type", source_type_str))
            .bind(("path_or_url", source.path_or_url.clone()))
            .bind(("title", source.title.clone()))
            .bind(("space_key", source.space_key.clone()))
            .bind(("last_indexed", last_indexed))
            .bind(("ttl_days", source.ttl_days))
            .await?
            .check()?;

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
            .await?
            .check()?;

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
            .await?
            .check()?;

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
                .await?
                .check()?;
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

        #[derive(Debug, Deserialize)]
        struct SourceIdRecord {
            id: String,
        }

        let mut source_id_result = self
            .db
            .query("SELECT meta::id(id) as id FROM doc_source")
            .await?;
        let source_ids: Vec<String> = source_id_result
            .take::<Vec<SourceIdRecord>>(0)?
            .into_iter()
            .map(|record| record.id)
            .collect();
        if source_ids.is_empty() {
            return Ok(Vec::new());
        }

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
                WHERE source_id IN $source_ids
                ORDER BY score DESC
                LIMIT $limit
                "#,
            )
            .bind(("query", query_embedding.to_vec()))
            .bind(("source_ids", source_ids))
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

    /// Delete chunks for missing source IDs only when those IDs are still orphaned.
    ///
    /// This never deletes chunks whose `source_id` currently exists in `doc_source`.
    ///
    /// # Errors
    ///
    /// Returns an error if counting or deletion fails.
    pub async fn delete_orphan_chunks_for_sources(
        &self,
        missing_source_ids: &[String],
    ) -> StoreResult<DocumentOrphanDeleteResult> {
        let mut unique_missing_ids = missing_source_ids
            .iter()
            .filter(|source_id| !source_id.trim().is_empty())
            .cloned()
            .collect::<Vec<_>>();
        unique_missing_ids.sort();
        unique_missing_ids.dedup();

        if unique_missing_ids.is_empty() {
            return Ok(DocumentOrphanDeleteResult {
                requested_source_ids: 0,
                deleted_chunk_count: 0,
                deleted_sources: Vec::new(),
                protected_source_ids: Vec::new(),
            });
        }

        #[derive(Debug, Deserialize)]
        struct SourceIdRecord {
            id: String,
        }

        let mut source_result = self
            .db
            .query("SELECT meta::id(id) as id FROM doc_source")
            .await?;
        let current_source_ids: Vec<String> = source_result
            .take::<Vec<SourceIdRecord>>(0)?
            .into_iter()
            .map(|record| record.id)
            .collect();
        let current_source_id_set = current_source_ids.iter().collect::<HashSet<_>>();
        let protected_source_ids = unique_missing_ids
            .iter()
            .filter(|source_id| current_source_id_set.contains(source_id))
            .cloned()
            .collect::<Vec<_>>();

        let mut count_result = self
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
                    parent_id
                FROM doc_chunk
                WHERE source_id IN $target_source_ids
                  AND source_id NOT IN $current_source_ids
                ORDER BY source_id, start_line, heading_path
                "#,
            )
            .bind(("target_source_ids", unique_missing_ids.clone()))
            .bind(("current_source_ids", current_source_ids.clone()))
            .await?;
        let records: Vec<DocChunkRecord> = count_result.take(0)?;

        let mut counts_by_source: BTreeMap<String, u64> = BTreeMap::new();
        for record in records {
            *counts_by_source.entry(record.source_id).or_default() += 1;
        }

        self.db
            .query(
                r#"
                DELETE doc_chunk
                WHERE source_id IN $target_source_ids
                  AND source_id NOT IN $current_source_ids;
                "#,
            )
            .bind(("target_source_ids", unique_missing_ids.clone()))
            .bind(("current_source_ids", current_source_ids))
            .await?
            .check()?;

        let deleted_sources = counts_by_source
            .into_iter()
            .map(
                |(missing_source_id, deleted_chunks)| DocumentDeletedOrphanSource {
                    missing_source_id,
                    deleted_chunks,
                },
            )
            .collect::<Vec<_>>();
        let deleted_chunk_count = deleted_sources
            .iter()
            .map(|source| source.deleted_chunks)
            .sum();

        Ok(DocumentOrphanDeleteResult {
            requested_source_ids: unique_missing_ids.len(),
            deleted_chunk_count,
            deleted_sources,
            protected_source_ids,
        })
    }

    /// Get statistics about the document store.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn stats(&self) -> StoreResult<DocumentStats> {
        #[derive(Debug, Deserialize)]
        struct SourceIdRecord {
            id: String,
        }

        let mut source_id_result = self
            .db
            .query("SELECT meta::id(id) as id FROM doc_source")
            .await?;
        let source_ids: Vec<String> = source_id_result
            .take::<Vec<SourceIdRecord>>(0)?
            .into_iter()
            .map(|record| record.id)
            .collect();

        let mut result = self
            .db
            .query(
                r#"
                SELECT count() as count FROM doc_source GROUP ALL;
                SELECT count() as count FROM doc_chunk GROUP ALL;
                SELECT count() as count FROM doc_chunk WHERE source_id IN $source_ids GROUP ALL;
                SELECT count() as count FROM doc_chunk WHERE source_id NOT IN $source_ids GROUP ALL;
                "#,
            )
            .bind(("source_ids", source_ids))
            .await?;

        #[derive(Debug, Deserialize)]
        struct CountResult {
            count: i64,
        }

        let source_counts: Vec<CountResult> = result.take(0)?;
        let chunk_counts: Vec<CountResult> = result.take(1)?;
        let searchable_chunk_counts: Vec<CountResult> = result.take(2)?;
        let orphan_chunk_counts: Vec<CountResult> = result.take(3)?;

        Ok(DocumentStats {
            source_count: source_counts.first().map(|c| c.count as u64).unwrap_or(0),
            chunk_count: chunk_counts.first().map(|c| c.count as u64).unwrap_or(0),
            searchable_chunk_count: searchable_chunk_counts
                .first()
                .map(|c| c.count as u64)
                .unwrap_or(0),
            orphan_chunk_count: orphan_chunk_counts
                .first()
                .map(|c| c.count as u64)
                .unwrap_or(0),
        })
    }

    /// Build a read-only recovery report for chunks whose source record is missing.
    ///
    /// This does not mutate the store. It groups orphan chunks by their missing source ID,
    /// returns bounded samples, and extracts source references from chunk content.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn orphan_report(
        &self,
        group_limit: usize,
        sample_limit_per_group: usize,
        preview_chars: usize,
    ) -> StoreResult<DocumentOrphanReport> {
        #[derive(Debug, Deserialize)]
        struct SourceRecord {
            id: String,
            path_or_url: String,
        }

        let mut source_result = self
            .db
            .query("SELECT meta::id(id) as id, path_or_url FROM doc_source")
            .await?;
        let sources: Vec<SourceRecord> = source_result.take(0)?;
        let source_ids: Vec<String> = sources.iter().map(|source| source.id.clone()).collect();
        let source_by_path: HashMap<String, String> = sources
            .into_iter()
            .map(|source| (source.path_or_url, source.id))
            .collect();

        let mut chunk_result = self
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
                    parent_id
                FROM doc_chunk
                WHERE source_id NOT IN $source_ids
                ORDER BY source_id, start_line, heading_path
                "#,
            )
            .bind(("source_ids", source_ids))
            .await?;
        let orphan_records: Vec<DocChunkRecord> = chunk_result.take(0)?;

        let orphan_chunk_count = orphan_records.len() as u64;
        let mut groups_by_source: BTreeMap<String, Vec<DocChunk>> = BTreeMap::new();
        for record in orphan_records {
            let chunk = record.into_doc_chunk();
            groups_by_source
                .entry(chunk.source_id.to_string())
                .or_default()
                .push(chunk);
        }

        let orphan_source_count = groups_by_source.len();
        let mut groups: Vec<DocumentOrphanGroup> = groups_by_source
            .into_iter()
            .map(|(missing_source_id, chunks)| {
                build_orphan_group(
                    missing_source_id,
                    chunks,
                    sample_limit_per_group,
                    preview_chars,
                    &source_by_path,
                )
            })
            .collect();

        groups.sort_by(|left, right| {
            right
                .chunk_count
                .cmp(&left.chunk_count)
                .then_with(|| left.missing_source_id.cmp(&right.missing_source_id))
        });

        if group_limit > 0 {
            groups.truncate(group_limit);
        } else {
            groups.clear();
        }

        let groups_with_known_source_match = groups
            .iter()
            .filter(|group| {
                group
                    .detected_references
                    .iter()
                    .any(|reference| reference.existing_source_id.is_some())
            })
            .count();
        let recovery_summary = DocumentRecoverySummary::from_groups(&groups);

        Ok(DocumentOrphanReport {
            orphan_chunk_count,
            orphan_source_count,
            groups_returned: groups.len(),
            sample_limit_per_group,
            recovery_summary,
            groups_with_known_source_match,
            groups_with_candidate_matches: 0,
            candidate_files_scanned: 0,
            candidate_files_skipped: 0,
            candidate_scan_warnings: Vec::new(),
            groups,
        })
    }
}

fn build_orphan_group(
    missing_source_id: String,
    chunks: Vec<DocChunk>,
    sample_limit_per_group: usize,
    preview_chars: usize,
    source_by_path: &HashMap<String, String>,
) -> DocumentOrphanGroup {
    let mut seen_references = HashSet::new();
    let mut detected_references = Vec::new();

    for chunk in &chunks {
        for mut reference in extract_source_references(&chunk.content) {
            reference.existing_source_id = source_by_path.get(&reference.value).cloned();
            if seen_references.insert((reference.reference_type.clone(), reference.value.clone())) {
                detected_references.push(reference);
            }
        }
    }

    let recovery_hint = if detected_references
        .iter()
        .any(|reference| reference.existing_source_id.is_some())
    {
        "matches_existing_source"
    } else if detected_references.iter().any(|reference| {
        matches!(
            reference.reference_type.as_str(),
            "absolute_path" | "source_path"
        )
    }) {
        "source_path_detected"
    } else {
        "unknown_source"
    }
    .to_string();
    let recovery_class = classify_recovery_group(&recovery_hint, &detected_references, &[]);

    let normalized_content = chunks
        .iter()
        .map(|chunk| normalize_for_fingerprint(&chunk.content))
        .collect::<Vec<_>>()
        .join("\n");
    let content_fingerprint = stable_fingerprint(&normalized_content);
    let content_anchors = content_anchors_for_chunks(&chunks, 24);
    let content_anchor_count = content_anchors.len();

    let samples = chunks
        .iter()
        .take(sample_limit_per_group)
        .map(|chunk| DocumentOrphanChunkSample {
            chunk_id: chunk.id.to_string(),
            heading_path: chunk.heading_path.clone(),
            start_line: chunk.start_line,
            end_line: chunk.end_line,
            content_preview: content_preview(&chunk.content, preview_chars),
        })
        .collect();

    DocumentOrphanGroup {
        missing_source_id,
        chunk_count: chunks.len() as u64,
        recovery_class,
        recovery_hint,
        content_fingerprint,
        content_anchor_count,
        content_anchors,
        detected_references,
        candidate_matches: Vec::new(),
        samples,
    }
}

fn classify_recovery_group(
    recovery_hint: &str,
    detected_references: &[DocumentDetectedReference],
    candidate_matches: &[DocumentRecoveryCandidateMatch],
) -> DocumentRecoveryClass {
    if !candidate_matches.is_empty()
        || detected_references
            .iter()
            .any(|reference| reference.existing_source_id.is_some())
        || recovery_hint == "matches_existing_source"
    {
        DocumentRecoveryClass::Recoverable
    } else if recovery_hint == "source_path_detected"
        || detected_references.iter().any(|reference| {
            matches!(
                reference.reference_type.as_str(),
                "absolute_path" | "source_path" | "document_path" | "original_path"
            )
        })
    {
        DocumentRecoveryClass::Unknown
    } else {
        DocumentRecoveryClass::SafeToQuarantine
    }
}

/// Normalize text for deterministic content fingerprinting and substring matching.
#[must_use]
pub fn normalize_for_fingerprint(content: &str) -> String {
    let mut normalized = String::with_capacity(content.len());
    let mut last_was_space = true;

    for ch in content.chars().flat_map(char::to_lowercase) {
        if ch.is_alphanumeric() {
            normalized.push(ch);
            last_was_space = false;
        } else if !last_was_space {
            normalized.push(' ');
            last_was_space = true;
        }
    }

    normalized.trim().to_string()
}

/// Stable FNV-1a fingerprint for already-normalized content.
#[must_use]
pub fn stable_fingerprint(content: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in content.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn content_anchors_for_chunks(chunks: &[DocChunk], max_anchors: usize) -> Vec<String> {
    let mut anchors = Vec::new();
    let mut seen = HashSet::new();
    for chunk in chunks {
        let normalized = normalize_for_fingerprint(&chunk.content);
        for anchor in content_anchors(&normalized) {
            if seen.insert(anchor.clone()) {
                anchors.push(anchor);
                if anchors.len() >= max_anchors {
                    return anchors;
                }
            }
        }
    }
    anchors
}

fn content_anchors(normalized_content: &str) -> Vec<String> {
    const WINDOW: usize = 14;
    const MIN_ANCHOR_CHARS: usize = 64;

    let tokens: Vec<&str> = normalized_content.split_whitespace().collect();
    if tokens.len() < WINDOW {
        return Vec::new();
    }

    let mut positions = vec![0];
    if tokens.len() > WINDOW * 2 {
        positions.push(tokens.len() / 3);
        positions.push(tokens.len() * 2 / 3);
    }

    let mut anchors = Vec::new();
    for position in positions {
        let start = position.min(tokens.len() - WINDOW);
        let anchor = tokens[start..start + WINDOW].join(" ");
        if anchor.len() >= MIN_ANCHOR_CHARS {
            anchors.push(anchor);
        }
    }
    anchors
}

fn extract_source_references(content: &str) -> Vec<DocumentDetectedReference> {
    const PREFIXES: [(&str, &str); 7] = [
        ("Absolute path:", "absolute_path"),
        ("Source path:", "source_path"),
        ("Document path:", "document_path"),
        ("Original path:", "original_path"),
        ("Source review:", "source_review"),
        ("Review path:", "review_path"),
        ("Migration review:", "migration_review"),
    ];

    let mut references = Vec::new();
    for line in content.lines() {
        let line = line.trim().trim_start_matches('-').trim();
        for (prefix, reference_type) in PREFIXES {
            if let Some(value) = line.strip_prefix(prefix).and_then(clean_reference_value) {
                references.push(DocumentDetectedReference {
                    reference_type: reference_type.to_string(),
                    value,
                    existing_source_id: None,
                });
            }
        }
    }
    references
}

fn clean_reference_value(value: &str) -> Option<String> {
    let value = value.trim();
    let value = value.strip_prefix('`').unwrap_or(value);
    let value = value.strip_suffix('`').unwrap_or(value);
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn content_preview(content: &str, preview_chars: usize) -> String {
    let collapsed = content.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut preview = String::new();
    for ch in collapsed.chars().take(preview_chars) {
        preview.push(ch);
    }
    if collapsed.chars().count() > preview_chars {
        preview.push_str("...");
    }
    preview
}

/// Statistics about the document store.
#[derive(Debug, Clone)]
pub struct DocumentStats {
    /// Number of document sources.
    pub source_count: u64,
    /// Number of document chunks.
    pub chunk_count: u64,
    /// Number of chunks attached to a persisted source.
    pub searchable_chunk_count: u64,
    /// Number of chunks whose source record is missing.
    pub orphan_chunk_count: u64,
}

/// Read-only recovery report for document chunks whose source is missing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentOrphanReport {
    /// Number of chunks whose source record is missing.
    pub orphan_chunk_count: u64,
    /// Number of missing source IDs referenced by orphan chunks.
    pub orphan_source_count: usize,
    /// Number of groups returned in this report.
    pub groups_returned: usize,
    /// Number of sample chunks included per group.
    pub sample_limit_per_group: usize,
    /// Recovery classification counts for returned groups.
    pub recovery_summary: DocumentRecoverySummary,
    /// Number of returned groups with a detected reference matching a known source path.
    pub groups_with_known_source_match: usize,
    /// Number of returned groups with at least one fingerprint candidate match.
    pub groups_with_candidate_matches: usize,
    /// Candidate files or digest sources scanned for fingerprint matching.
    pub candidate_files_scanned: usize,
    /// Candidate files skipped due to limits, unsupported types, or read failures.
    pub candidate_files_skipped: usize,
    /// Non-fatal warnings from optional candidate scanning.
    pub candidate_scan_warnings: Vec<String>,
    /// Orphan groups, sorted by chunk count descending.
    pub groups: Vec<DocumentOrphanGroup>,
}

impl DocumentOrphanReport {
    /// Recompute derived recovery fields after report enrichment.
    pub fn refresh_recovery_summary(&mut self) {
        for group in &mut self.groups {
            group.recovery_class = classify_recovery_group(
                &group.recovery_hint,
                &group.detected_references,
                &group.candidate_matches,
            );
        }
        self.recovery_summary = DocumentRecoverySummary::from_groups(&self.groups);
    }
}

/// Summary counts by recovery classification for returned groups.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentRecoverySummary {
    /// Groups that have a direct source match or candidate match.
    pub recoverable: usize,
    /// Groups with source/path clues but no current match.
    pub unknown: usize,
    /// Groups without source/path clues or candidate matches, suitable for quarantine review.
    pub safe_to_quarantine: usize,
}

impl DocumentRecoverySummary {
    fn from_groups(groups: &[DocumentOrphanGroup]) -> Self {
        let mut summary = Self::default();
        for group in groups {
            match group.recovery_class {
                DocumentRecoveryClass::Recoverable => summary.recoverable += 1,
                DocumentRecoveryClass::Unknown => summary.unknown += 1,
                DocumentRecoveryClass::SafeToQuarantine => summary.safe_to_quarantine += 1,
            }
        }
        summary
    }
}

/// Result of deleting orphan chunks for missing source IDs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentOrphanDeleteResult {
    /// Number of unique source IDs requested for deletion.
    pub requested_source_ids: usize,
    /// Number of orphan chunks deleted.
    pub deleted_chunk_count: u64,
    /// Per-source deletion counts.
    pub deleted_sources: Vec<DocumentDeletedOrphanSource>,
    /// Requested source IDs that currently exist in doc_source and were protected from deletion.
    pub protected_source_ids: Vec<String>,
}

/// Per-source orphan deletion count.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentDeletedOrphanSource {
    /// Missing source ID whose orphan chunks were deleted.
    pub missing_source_id: String,
    /// Number of chunks deleted for this source ID.
    pub deleted_chunks: u64,
}

/// Recovery classification for an orphan source group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentRecoveryClass {
    /// Group has a direct known source match or fingerprint candidate match.
    Recoverable,
    /// Group has clues but no current match.
    Unknown,
    /// Group has no source clues or matches and can be reviewed for quarantine.
    SafeToQuarantine,
}

/// Orphan chunks that reference the same missing source ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentOrphanGroup {
    /// Missing source ID referenced by the chunks.
    pub missing_source_id: String,
    /// Number of chunks in this orphan group.
    pub chunk_count: u64,
    /// Conservative recovery class for migration review.
    pub recovery_class: DocumentRecoveryClass,
    /// Recovery classification derived from detected references.
    pub recovery_hint: String,
    /// Stable fingerprint over normalized orphan group content.
    pub content_fingerprint: String,
    /// Number of content anchors available for fingerprint matching.
    pub content_anchor_count: usize,
    /// Bounded normalized anchors used internally for candidate matching.
    #[serde(skip)]
    pub content_anchors: Vec<String>,
    /// Source references extracted from chunk content.
    pub detected_references: Vec<DocumentDetectedReference>,
    /// Candidate files or reviewed digest sources that appear to contain this orphan content.
    pub candidate_matches: Vec<DocumentRecoveryCandidateMatch>,
    /// Bounded chunk samples for human review.
    pub samples: Vec<DocumentOrphanChunkSample>,
}

/// Candidate file or digest source that may recover an orphan chunk group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRecoveryCandidateMatch {
    /// Match source: file or digest_reviewed_source.
    pub match_type: String,
    /// Candidate path.
    pub path: String,
    /// Confidence score from 0.0 to 1.0.
    pub score: f32,
    /// Number of orphan anchors found in the candidate.
    pub matched_anchors: usize,
    /// Number of anchors tested for this group.
    pub total_anchors: usize,
    /// Whether the normalized full-content fingerprint matched exactly.
    pub exact_fingerprint_match: bool,
    /// Short matched anchor previews for human review.
    pub evidence: Vec<String>,
}

/// Source-like reference detected inside an orphan chunk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentDetectedReference {
    /// Reference type, such as absolute_path, source_path, or source_review.
    pub reference_type: String,
    /// Reference value.
    pub value: String,
    /// Existing doc_source ID when the value exactly matches a known source path.
    pub existing_source_id: Option<String>,
}

/// Bounded sample of an orphan chunk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentOrphanChunkSample {
    /// Chunk ID.
    pub chunk_id: String,
    /// Heading path.
    pub heading_path: String,
    /// Start line in the original source, if known.
    pub start_line: Option<u32>,
    /// End line in the original source, if known.
    pub end_line: Option<u32>,
    /// Whitespace-collapsed content preview.
    pub content_preview: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_chunk_creation() {
        let source_id = Id::new();
        let chunk = DocChunk::new(source_id, "# Test > ## Section", 2, "Test content");

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
