//! Document knowledge types (Layer 3).
//!
//! Documents are indexed, chunked, and made searchable.
//! This layer handles READMEs, documentation files, etc.

use crate::id::Id;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Type of document source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    /// Local file on disk.
    LocalFile,
    /// Confluence page.
    Confluence,
    /// GitHub file or README.
    GitHub,
    /// Notion page.
    Notion,
    /// Custom source.
    Custom(String),
}

/// A document source (file, URL, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocSource {
    /// Unique identifier.
    pub id: Id,

    /// Type of source.
    pub source_type: SourceType,

    /// Path or URL to the document.
    pub path_or_url: String,

    /// Document title.
    pub title: Option<String>,

    /// Space key (for Confluence).
    pub space_key: Option<String>,

    /// Last time this document was indexed.
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_indexed: Option<OffsetDateTime>,

    /// TTL in days before re-indexing.
    pub ttl_days: i32,
}

impl DocSource {
    /// Create a new local file source.
    #[must_use]
    pub fn local_file(path: impl Into<String>) -> Self {
        Self {
            id: Id::new(),
            source_type: SourceType::LocalFile,
            path_or_url: path.into(),
            title: None,
            space_key: None,
            last_indexed: None,
            ttl_days: 7,
        }
    }

    /// Create a new Confluence source.
    #[must_use]
    pub fn confluence(url: impl Into<String>, space_key: impl Into<String>) -> Self {
        Self {
            id: Id::new(),
            source_type: SourceType::Confluence,
            path_or_url: url.into(),
            title: None,
            space_key: Some(space_key.into()),
            last_indexed: None,
            ttl_days: 7,
        }
    }

    /// Set the title.
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Check if the document needs re-indexing.
    #[must_use]
    pub fn needs_reindex(&self) -> bool {
        match self.last_indexed {
            None => true,
            Some(indexed) => {
                let now = OffsetDateTime::now_utc();
                let days_since = (now - indexed).whole_days();
                days_since >= self.ttl_days as i64
            }
        }
    }

    /// Mark as indexed now.
    pub fn mark_indexed(&mut self) {
        self.last_indexed = Some(OffsetDateTime::now_utc());
    }
}

/// A chunk of a document (section, paragraph, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocChunk {
    /// Unique identifier.
    pub id: Id,

    /// Source document ID.
    pub source_id: Id,

    /// Heading path (e.g., "# Main > ## Setup > ### Install").
    pub heading_path: String,

    /// Heading level (1-6).
    pub heading_level: u8,

    /// Content of the chunk.
    pub content: String,

    /// Start line in the source document.
    pub start_line: Option<u32>,

    /// End line in the source document.
    pub end_line: Option<u32>,

    /// Parent chunk ID (for hierarchical navigation).
    pub parent_id: Option<Id>,
}

impl DocChunk {
    /// Create a new document chunk.
    #[must_use]
    pub fn new(
        source_id: Id,
        heading_path: impl Into<String>,
        heading_level: u8,
        content: impl Into<String>,
    ) -> Self {
        Self {
            id: Id::new(),
            source_id,
            heading_path: heading_path.into(),
            heading_level,
            content: content.into(),
            start_line: None,
            end_line: None,
            parent_id: None,
        }
    }

    /// Set line numbers.
    #[must_use]
    pub fn with_lines(mut self, start: u32, end: u32) -> Self {
        self.start_line = Some(start);
        self.end_line = Some(end);
        self
    }

    /// Set parent chunk.
    #[must_use]
    pub fn with_parent(mut self, parent_id: Id) -> Self {
        self.parent_id = Some(parent_id);
        self
    }
}

/// Search result for document search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocSearchResult {
    /// The matching chunk.
    pub chunk: DocChunk,

    /// Source document info.
    pub source: DocSource,

    /// Relevance score (0.0 - 1.0).
    pub score: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_source_creation() {
        let source = DocSource::local_file("/path/to/README.md").with_title("README");

        assert_eq!(source.source_type, SourceType::LocalFile);
        assert_eq!(source.path_or_url, "/path/to/README.md");
        assert_eq!(source.title, Some("README".to_string()));
    }

    #[test]
    fn test_needs_reindex() {
        let source = DocSource::local_file("/path/to/file.md");
        assert!(source.needs_reindex()); // Never indexed
    }

    #[test]
    fn test_chunk_creation() {
        let source_id = Id::new();
        let chunk = DocChunk::new(
            source_id,
            "# README > ## Installation",
            2,
            "Run `cargo install engram`...",
        )
        .with_lines(10, 20);

        assert_eq!(chunk.heading_level, 2);
        assert_eq!(chunk.start_line, Some(10));
        assert_eq!(chunk.end_line, Some(20));
    }
}
