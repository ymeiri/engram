//! Document ingestion pipeline.
//!
//! Orchestrates the full ingestion process:
//! 1. Parse documents
//! 2. Chunk into sections
//! 3. Generate embeddings
//! 4. Store in database

use crate::chunker::{chunk_document, ChunkerConfig};
use crate::error::IndexResult;
use crate::parser::{parse_file, ParsedDocument};
use engram_core::document::{DocChunk, DocSource};
use engram_embed::Embedder;
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

    /// Find all indexable files in a directory.
    fn find_files(&self, path: &Path) -> IndexResult<Vec<PathBuf>> {
        let mut files = Vec::new();
        self.find_files_recursive(path, &mut files)?;
        Ok(files)
    }

    fn find_files_recursive(&self, path: &Path, files: &mut Vec<PathBuf>) -> IndexResult<()> {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if self.config.recursive {
                    self.find_files_recursive(&path, files)?;
                }
            } else if self.should_index(&path) {
                files.push(path);
            }
        }
        Ok(())
    }

    /// Check if a file should be indexed.
    fn should_index(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|ext| self.config.extensions.iter().any(|e| e == ext))
            .unwrap_or(false)
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
