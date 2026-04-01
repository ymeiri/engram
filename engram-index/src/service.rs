//! Document service - coordinates indexing and search.
//!
//! The service layer provides a high-level API for document operations,
//! combining the ingestion pipeline with the storage repository.

use crate::error::IndexResult;
use crate::pipeline::{IndexedDocument, Pipeline, PipelineConfig};
use engram_core::document::DocSearchResult;
use engram_embed::Embedder;
use engram_store::{Db, DocumentRepo};
use std::path::Path;
use tracing::{debug, info};

/// The document service for indexing and searching documents.
pub struct DocumentService {
    pipeline: Pipeline,
    repo: DocumentRepo,
    embedder: Embedder,
}

impl DocumentService {
    /// Create a new document service.
    pub fn new(db: Db, embedder: Embedder, config: PipelineConfig) -> Self {
        let pipeline = Pipeline::new(embedder.clone(), config);
        let repo = DocumentRepo::new(db);
        Self {
            pipeline,
            repo,
            embedder,
        }
    }

    /// Create a document service with default configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the embedder cannot be initialized.
    pub fn with_defaults(db: Db) -> IndexResult<Self> {
        let embedder = Embedder::default_model()?;
        Ok(Self::new(db, embedder, PipelineConfig::default()))
    }

    /// Initialize the document schema in the database.
    ///
    /// # Errors
    ///
    /// Returns an error if schema creation fails.
    pub async fn init_schema(&self) -> IndexResult<()> {
        self.repo.init_schema().await?;
        Ok(())
    }

    /// Index a file and store it in the database.
    ///
    /// # Errors
    ///
    /// Returns an error if indexing or storage fails.
    pub async fn index_file(&self, path: impl AsRef<Path>) -> IndexResult<IndexedDocument> {
        let path = path.as_ref();
        info!("Indexing file: {}", path.display());

        // Check if already indexed
        let path_str = path.display().to_string();
        if let Some(existing) = self.repo.find_source_by_path(&path_str).await? {
            if !existing.needs_reindex() {
                debug!("File already indexed and fresh: {}", path.display());
                // Return existing document info
                let chunks = self.repo.get_chunks_for_source(&existing.id).await?;
                let parsed = crate::parser::parse_file(path)?;
                return Ok(IndexedDocument {
                    source: existing,
                    parsed,
                    chunks: chunks
                        .into_iter()
                        .map(|chunk| crate::pipeline::IndexedChunk {
                            chunk,
                            embedding: Vec::new(), // Not loading embeddings for existing
                        })
                        .collect(),
                });
            }
            debug!("Re-indexing stale file: {}", path.display());
        }

        // Index the file
        let mut result = self.pipeline.index_file(path)?;

        // Mark as indexed and save
        result.source.mark_indexed();
        self.repo.save_source(&result.source).await?;

        // Save chunks with embeddings
        let chunks_with_embeddings: Vec<_> = result
            .chunks
            .iter()
            .map(|ic| (ic.chunk.clone(), ic.embedding.clone()))
            .collect();
        self.repo
            .save_chunks(&result.source.id, chunks_with_embeddings)
            .await?;

        info!(
            "Indexed {} with {} chunks",
            path.display(),
            result.chunks.len()
        );

        Ok(result)
    }

    /// Index a directory and store all documents.
    ///
    /// # Errors
    ///
    /// Returns an error if indexing fails.
    pub async fn index_directory(
        &self,
        path: impl AsRef<Path>,
    ) -> IndexResult<Vec<IndexedDocument>> {
        let path = path.as_ref();
        info!("Indexing directory: {}", path.display());

        let results = self.pipeline.index_directory(path)?;

        // Save all to database
        for doc in &results {
            let mut source = doc.source.clone();
            source.mark_indexed();
            self.repo.save_source(&source).await?;

            let chunks_with_embeddings: Vec<_> = doc
                .chunks
                .iter()
                .map(|ic| (ic.chunk.clone(), ic.embedding.clone()))
                .collect();
            self.repo
                .save_chunks(&source.id, chunks_with_embeddings)
                .await?;
        }

        Ok(results)
    }

    /// Search for documents by semantic similarity.
    ///
    /// # Arguments
    ///
    /// * `query` - The search query text
    /// * `limit` - Maximum number of results to return
    ///
    /// # Errors
    ///
    /// Returns an error if search fails.
    pub async fn search(&self, query: &str, limit: usize) -> IndexResult<Vec<DocSearchResult>> {
        debug!("Searching for: {}", query);

        // Generate embedding for the query
        let query_embedding = self.embedder.embed(query)?;

        // Search in database
        let results = self.repo.search_similar(&query_embedding, limit).await?;

        info!("Found {} results for query", results.len());
        Ok(results)
    }

    /// Search with minimum score threshold.
    ///
    /// # Errors
    ///
    /// Returns an error if search fails.
    pub async fn search_threshold(
        &self,
        query: &str,
        limit: usize,
        min_score: f32,
    ) -> IndexResult<Vec<DocSearchResult>> {
        let results = self.search(query, limit).await?;
        Ok(results
            .into_iter()
            .filter(|r| r.score >= min_score)
            .collect())
    }

    /// Get statistics about indexed documents.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn stats(&self) -> IndexResult<DocumentStats> {
        let db_stats = self.repo.stats().await?;
        Ok(DocumentStats {
            source_count: db_stats.source_count,
            chunk_count: db_stats.chunk_count,
            embedding_dimension: self.pipeline.embedding_dimension(),
        })
    }
}

/// Statistics about the document index.
#[derive(Debug, Clone)]
pub struct DocumentStats {
    /// Number of indexed document sources.
    pub source_count: u64,
    /// Number of document chunks.
    pub chunk_count: u64,
    /// Embedding vector dimension.
    pub embedding_dimension: usize,
}
