//! Document ingestion pipeline.
//!
//! Orchestrates the full ingestion process:
//! 1. Parse documents
//! 2. Chunk into sections
//! 3. Generate embeddings
//! 4. Store in database

use crate::chunker::{chunk_document, chunking_strategy, ChunkerConfig, ChunkingStrategy};
use crate::error::IndexResult;
use crate::parser::{parse_content, parse_file, ParsedDocument};
use engram_core::document::{DocChunk, DocSource};
use engram_embed::Embedder;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Result of indexing a single document.
#[derive(Debug)]
pub struct IndexedDocument {
    /// The document source.
    pub source: DocSource,
    /// Parsed document info.
    pub parsed: ParsedDocument,
    /// Generated chunks with embeddings.
    pub chunks: Vec<IndexedChunk>,
}

/// A chunk with its embedding.
#[derive(Debug)]
pub struct IndexedChunk {
    /// The document chunk.
    pub chunk: DocChunk,
    /// The embedding vector.
    pub embedding: Vec<f32>,
}

/// Dry-run plan for document ingestion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentIngestionPlan {
    /// Planned documents.
    pub documents: Vec<PlannedDocument>,
    /// Files or sources that could not be planned.
    pub warnings: Vec<String>,
}

impl DocumentIngestionPlan {
    /// Total number of planned chunks.
    #[must_use]
    pub fn total_chunks(&self) -> usize {
        self.documents
            .iter()
            .map(|document| document.chunk_count)
            .sum()
    }
}

/// Dry-run plan for one document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedDocument {
    /// Source path or URL.
    pub path: String,
    /// Parsed document title.
    pub title: String,
    /// Source character count after trimming whitespace.
    pub content_chars: usize,
    /// Number of markdown sections parsed from headings.
    pub section_count: usize,
    /// Number of chunks that would be embedded.
    pub chunk_count: usize,
    /// Chunking strategy selected by the policy.
    pub chunking_strategy: ChunkingStrategy,
    /// Current short-document threshold.
    pub short_document_char_limit: usize,
    /// Current maximum chunk size.
    pub max_chunk_size: usize,
}

/// Configuration for the ingestion pipeline.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Chunker configuration.
    pub chunker: ChunkerConfig,
    /// File extensions to index.
    pub extensions: Vec<String>,
    /// Whether to recurse into subdirectories.
    pub recursive: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            chunker: ChunkerConfig::default(),
            extensions: vec!["md".to_string(), "markdown".to_string()],
            recursive: true,
        }
    }
}

/// The document ingestion pipeline.
pub struct Pipeline {
    embedder: Embedder,
    config: PipelineConfig,
}

impl Pipeline {
    /// Create a new pipeline with the given embedder and configuration.
    pub fn new(embedder: Embedder, config: PipelineConfig) -> Self {
        Self { embedder, config }
    }

    /// Create a pipeline with default configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the embedder cannot be initialized.
    pub fn with_defaults() -> IndexResult<Self> {
        let embedder = Embedder::default_model()?;
        Ok(Self::new(embedder, PipelineConfig::default()))
    }

    /// Index a single file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or processed.
    pub fn index_file(&self, path: impl AsRef<Path>) -> IndexResult<IndexedDocument> {
        let path = path.as_ref();
        info!("Indexing file: {}", path.display());

        // Parse the document
        let parsed = parse_file(path)?;

        // Create document source
        let source =
            DocSource::local_file(path.display().to_string()).with_title(parsed.title.clone());

        // Chunk the document
        let chunks = chunk_document(&parsed, &source, &self.config.chunker);
        debug!("Created {} chunks from {}", chunks.len(), path.display());

        // Generate embeddings
        let indexed_chunks = self.embed_chunks(chunks)?;

        Ok(IndexedDocument {
            source,
            parsed,
            chunks: indexed_chunks,
        })
    }

    /// Build a dry-run ingestion plan for a single file without generating embeddings.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn plan_file(&self, path: impl AsRef<Path>) -> IndexResult<PlannedDocument> {
        Self::plan_file_with_config(path, &self.config)
    }

    /// Build a dry-run ingestion plan for a single file without constructing an embedder.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn plan_file_with_config(
        path: impl AsRef<Path>,
        config: &PipelineConfig,
    ) -> IndexResult<PlannedDocument> {
        let path = path.as_ref();
        let parsed = parse_file(path)?;
        let source =
            DocSource::local_file(path.display().to_string()).with_title(parsed.title.clone());

        Ok(plan_parsed_document(&parsed, &source, &config.chunker))
    }

    /// Build a dry-run ingestion plan for caller-supplied markdown content.
    ///
    /// # Errors
    ///
    /// Returns an error if the content cannot be parsed.
    pub fn plan_content(
        &self,
        path_or_url: impl Into<String>,
        content: impl Into<String>,
        title: Option<String>,
    ) -> IndexResult<PlannedDocument> {
        let path_or_url = path_or_url.into();
        let mut parsed = parse_content(path_or_url.clone(), content.into())?;
        if let Some(title) = title.filter(|title| !title.trim().is_empty()) {
            parsed.title = title;
        }
        let source = DocSource::local_file(path_or_url).with_title(parsed.title.clone());

        Ok(plan_parsed_document(&parsed, &source, &self.config.chunker))
    }

    /// Index markdown content supplied by the caller.
    ///
    /// # Errors
    ///
    /// Returns an error if the content cannot be parsed or embedded.
    pub fn index_content(
        &self,
        path_or_url: impl Into<String>,
        content: impl Into<String>,
        title: Option<String>,
    ) -> IndexResult<IndexedDocument> {
        let path_or_url = path_or_url.into();
        info!("Indexing supplied document content: {}", path_or_url);

        let mut parsed = parse_content(path_or_url.clone(), content.into())?;
        if let Some(title) = title.filter(|title| !title.trim().is_empty()) {
            parsed.title = title;
        }

        let source = DocSource::local_file(path_or_url.clone()).with_title(parsed.title.clone());
        let chunks = chunk_document(&parsed, &source, &self.config.chunker);
        debug!("Created {} chunks from {}", chunks.len(), path_or_url);
        let indexed_chunks = self.embed_chunks(chunks)?;

        Ok(IndexedDocument {
            source,
            parsed,
            chunks: indexed_chunks,
        })
    }

    /// Index a directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be read.
    pub fn index_directory(&self, path: impl AsRef<Path>) -> IndexResult<Vec<IndexedDocument>> {
        let path = path.as_ref();
        info!("Indexing directory: {}", path.display());

        let files = self.find_files(path)?;
        let total = files.len();
        info!("Found {} files to index", total);

        let mut results = Vec::new();
        for file in &files {
            match self.index_file(file) {
                Ok(doc) => results.push(doc),
                Err(e) => {
                    warn!("Failed to index {}: {}", file.display(), e);
                }
            }
        }

        info!("Successfully indexed {} of {} files", results.len(), total);
        Ok(results)
    }

    /// Build a dry-run ingestion plan for all indexable files in a directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be read.
    pub fn plan_directory(&self, path: impl AsRef<Path>) -> IndexResult<DocumentIngestionPlan> {
        Self::plan_directory_with_config(path, &self.config)
    }

    /// Build a dry-run ingestion plan for all indexable files in a directory without constructing
    /// an embedder.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be read.
    pub fn plan_directory_with_config(
        path: impl AsRef<Path>,
        config: &PipelineConfig,
    ) -> IndexResult<DocumentIngestionPlan> {
        let path = path.as_ref();
        let files = find_files_with_config(path, config)?;
        let mut documents = Vec::new();
        let mut warnings = Vec::new();

        for file in &files {
            match Self::plan_file_with_config(file, config) {
                Ok(document) => documents.push(document),
                Err(error) => warnings.push(format!("{}: {}", file.display(), error)),
            }
        }

        Ok(DocumentIngestionPlan {
            documents,
            warnings,
        })
    }

    /// Build a dry-run ingestion plan for a file or directory without constructing an embedder.
    ///
    /// # Errors
    ///
    /// Returns an error if the path cannot be read or parsed.
    pub fn plan_path_with_config(
        path: impl AsRef<Path>,
        config: &PipelineConfig,
    ) -> IndexResult<DocumentIngestionPlan> {
        let path = path.as_ref();
        if path.is_dir() {
            Self::plan_directory_with_config(path, config)
        } else {
            Ok(DocumentIngestionPlan {
                documents: vec![Self::plan_file_with_config(path, config)?],
                warnings: Vec::new(),
            })
        }
    }

    /// Find all indexable files in a directory.
    fn find_files(&self, path: &Path) -> IndexResult<Vec<PathBuf>> {
        find_files_with_config(path, &self.config)
    }

    /// Generate embeddings for chunks.
    fn embed_chunks(&self, chunks: Vec<DocChunk>) -> IndexResult<Vec<IndexedChunk>> {
        if chunks.is_empty() {
            return Ok(Vec::new());
        }

        // Prepare texts for embedding
        let texts: Vec<&str> = chunks.iter().map(|c| c.content.as_str()).collect();

        // Generate embeddings in batch
        let embeddings = self.embedder.embed_batch(&texts)?;

        // Combine chunks with embeddings
        let indexed = chunks
            .into_iter()
            .zip(embeddings)
            .map(|(chunk, embedding)| IndexedChunk { chunk, embedding })
            .collect();

        Ok(indexed)
    }

    /// Get the embedding dimension.
    #[must_use]
    pub fn embedding_dimension(&self) -> usize {
        self.embedder.dimension()
    }
}

fn plan_parsed_document(
    parsed: &ParsedDocument,
    source: &DocSource,
    chunker: &ChunkerConfig,
) -> PlannedDocument {
    let chunks = chunk_document(parsed, source, chunker);
    PlannedDocument {
        path: parsed.path.clone(),
        title: parsed.title.clone(),
        content_chars: parsed.raw_content.trim().chars().count(),
        section_count: parsed.sections.len(),
        chunk_count: chunks.len(),
        chunking_strategy: chunking_strategy(parsed, chunker),
        short_document_char_limit: chunker.short_document_char_limit,
        max_chunk_size: chunker.max_chunk_size,
    }
}

fn find_files_with_config(path: &Path, config: &PipelineConfig) -> IndexResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    find_files_recursive_with_config(path, config, &mut files)?;
    Ok(files)
}

fn find_files_recursive_with_config(
    path: &Path,
    config: &PipelineConfig,
    files: &mut Vec<PathBuf>,
) -> IndexResult<()> {
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if config.recursive {
                find_files_recursive_with_config(&path, config, files)?;
            }
        } else if should_index_with_config(&path, config) {
            files.push(path);
        }
    }
    Ok(())
}

fn should_index_with_config(path: &Path, config: &PipelineConfig) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| config.extensions.iter().any(|e| e == ext))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn test_find_files() {
        let dir = TempDir::new().unwrap();
        create_test_file(dir.path(), "test1.md", "# Test 1\n\nContent");
        create_test_file(dir.path(), "test2.md", "# Test 2\n\nContent");
        create_test_file(dir.path(), "ignore.txt", "Not markdown");

        // Create pipeline without embedder (just for file finding)
        let config = PipelineConfig::default();

        let files: Vec<PathBuf> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.extension()
                    .and_then(|e| e.to_str())
                    .map(|ext| config.extensions.iter().any(|e| e == ext))
                    .unwrap_or(false)
            })
            .collect();

        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_plan_path_with_config_does_not_require_embedder() {
        let dir = TempDir::new().unwrap();
        create_test_file(
            dir.path(),
            "short.md",
            "# Short\n\nA compact memory note that should stay whole.",
        );
        create_test_file(
            dir.path(),
            "long.md",
            &format!(
                "# Long\n\n## Context\n\n{}",
                "This is repeated long context. ".repeat(200)
            ),
        );
        create_test_file(dir.path(), "ignore.txt", "not indexed");

        let config = PipelineConfig::default();
        let plan = Pipeline::plan_path_with_config(dir.path(), &config).unwrap();

        assert_eq!(plan.documents.len(), 2);
        assert_eq!(plan.warnings.len(), 0);
        assert!(plan.total_chunks() >= 2);
        assert!(plan
            .documents
            .iter()
            .any(|document| document.chunking_strategy == ChunkingStrategy::WholeDocument));
        assert!(plan
            .documents
            .iter()
            .any(|document| document.chunking_strategy == ChunkingStrategy::HeadingSections));
    }

    #[test]
    #[ignore = "requires model download"]
    fn test_index_file() {
        let dir = TempDir::new().unwrap();
        let path = create_test_file(
            dir.path(),
            "test.md",
            r#"# Test Document

Introduction paragraph.

## Section One

This is the content of section one.
It has multiple lines.

## Section Two

Another section here.
"#,
        );

        let pipeline = Pipeline::with_defaults().unwrap();
        let result = pipeline.index_file(&path).unwrap();

        assert_eq!(result.source.title, Some("Test Document".to_string()));
        assert!(!result.chunks.is_empty());
        assert_eq!(result.chunks[0].embedding.len(), 384);
    }
}
