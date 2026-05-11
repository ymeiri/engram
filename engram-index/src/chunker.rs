//! Document chunking by heading hierarchy.
//!
//! Chunks documents into sections suitable for embedding and retrieval.

use crate::parser::{ParsedDocument, Section};
use engram_core::document::{DocChunk, DocSource};
use engram_core::id::Id;
use serde::{Deserialize, Serialize};

/// How a parsed document will be chunked for indexing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChunkingStrategy {
    /// The document has no indexable content.
    Empty,
    /// The document is short enough to preserve as one chunk.
    WholeDocument,
    /// The document has markdown headings and will be chunked by section.
    HeadingSections,
    /// The document has no markdown headings and will be split as one synthetic section.
    SyntheticSections,
}

/// Configuration for the chunker.
#[derive(Debug, Clone)]
pub struct ChunkerConfig {
    /// Maximum character length for indexing a document as one whole-document chunk.
    /// Set to 0 to always use section-based chunking.
    pub short_document_char_limit: usize,

    /// Minimum heading level to create chunks (1-6).
    /// Level 2 means H2 and below create chunks.
    pub min_chunk_level: u8,

    /// Maximum chunk size in characters.
    pub max_chunk_size: usize,

    /// Minimum preferred chunk size in characters.
    /// Small sections are still indexed so short durable facts are not lost.
    pub min_chunk_size: usize,

    /// Include parent heading context in chunks.
    pub include_heading_path: bool,
}

impl Default for ChunkerConfig {
    fn default() -> Self {
        Self {
            short_document_char_limit: 4000,
            min_chunk_level: 2,
            max_chunk_size: 2000,
            min_chunk_size: 100,
            include_heading_path: true,
        }
    }
}

/// Chunk a parsed document into `DocChunk`s.
pub fn chunk_document(
    doc: &ParsedDocument,
    source: &DocSource,
    config: &ChunkerConfig,
) -> Vec<DocChunk> {
    let sections = match chunking_strategy(doc, config) {
        ChunkingStrategy::Empty => return Vec::new(),
        ChunkingStrategy::WholeDocument => {
            return vec![create_whole_document_chunk(doc, source, config)]
        }
        ChunkingStrategy::HeadingSections => doc.sections.clone(),
        ChunkingStrategy::SyntheticSections => vec![synthetic_document_section(doc)],
    };

    let mut chunks = Vec::new();

    for section in &sections {
        // Skip sections above the minimum chunk level
        if section.level < config.min_chunk_level {
            // But include H1 content if it exists
            if section.level == 1 && !section.content.is_empty() {
                append_section_chunks(&mut chunks, source.id, section, config);
            }
            continue;
        }

        append_section_chunks(&mut chunks, source.id, section, config);
    }

    // Build parent relationships
    build_parent_relationships(&mut chunks);

    chunks
}

/// Classify how a document will be chunked before embeddings are generated.
#[must_use]
pub fn chunking_strategy(doc: &ParsedDocument, config: &ChunkerConfig) -> ChunkingStrategy {
    let trimmed_content = doc.raw_content.trim();
    if trimmed_content.is_empty() {
        return ChunkingStrategy::Empty;
    }

    if config.short_document_char_limit > 0
        && trimmed_content.len() <= config.short_document_char_limit
    {
        return ChunkingStrategy::WholeDocument;
    }

    if doc.sections.is_empty() {
        ChunkingStrategy::SyntheticSections
    } else {
        ChunkingStrategy::HeadingSections
    }
}

fn append_section_chunks(
    chunks: &mut Vec<DocChunk>,
    source_id: Id,
    section: &Section,
    config: &ChunkerConfig,
) {
    if section.content.len() <= config.max_chunk_size {
        chunks.push(create_chunk(source_id, section, None, config));
    } else {
        // Split large sections into multiple chunks
        let sub_chunks = split_large_section(section, config.max_chunk_size);
        for (idx, content) in sub_chunks.into_iter().enumerate() {
            let mut chunk = create_chunk_with_content(source_id, section, None, content, config);
            if idx > 0 {
                chunk.id = Id::new(); // New ID for split chunks
                chunk.heading_path = format!("{} (part {})", section.heading_path, idx + 1);
            }
            chunks.push(chunk);
        }
    }
}

fn create_whole_document_chunk(
    doc: &ParsedDocument,
    source: &DocSource,
    config: &ChunkerConfig,
) -> DocChunk {
    let section = synthetic_document_section(doc);
    create_chunk_with_content(
        source.id,
        &section,
        None,
        doc.raw_content.trim().to_string(),
        config,
    )
}

fn synthetic_document_section(doc: &ParsedDocument) -> Section {
    let line_count = doc.raw_content.lines().count().max(1) as u32;
    Section {
        heading: doc.title.clone(),
        level: 1,
        heading_path: format!("# {}", doc.title),
        content: doc.raw_content.trim().to_string(),
        start_line: 1,
        end_line: line_count,
    }
}

/// Create a chunk from a section.
fn create_chunk(
    source_id: Id,
    section: &Section,
    parent_id: Option<Id>,
    config: &ChunkerConfig,
) -> DocChunk {
    create_chunk_with_content(
        source_id,
        section,
        parent_id,
        section.content.clone(),
        config,
    )
}

fn create_chunk_with_content(
    source_id: Id,
    section: &Section,
    parent_id: Option<Id>,
    content: String,
    config: &ChunkerConfig,
) -> DocChunk {
    let mut chunk = DocChunk::new(
        source_id,
        section.heading_path.clone(),
        section.level,
        contextual_chunk_content(&section.heading_path, &content, config),
    )
    .with_lines(section.start_line, section.end_line);

    if let Some(parent) = parent_id {
        chunk = chunk.with_parent(parent);
    }

    chunk
}

fn contextual_chunk_content(heading_path: &str, content: &str, config: &ChunkerConfig) -> String {
    if !config.include_heading_path {
        return content.to_string();
    }

    let content = content.trim();
    if content.is_empty() {
        heading_path.to_string()
    } else {
        format!("{heading_path}\n\n{content}")
    }
}

/// Split a large section into multiple chunks.
fn split_large_section(section: &Section, max_size: usize) -> Vec<String> {
    let content = &section.content;
    let mut chunks = Vec::new();
    let mut current = String::new();

    // Try to split on paragraph boundaries
    for paragraph in content.split("\n\n") {
        if current.len() + paragraph.len() + 2 > max_size && !current.is_empty() {
            chunks.push(current.trim().to_string());
            current = String::new();
        }
        if !current.is_empty() {
            current.push_str("\n\n");
        }
        current.push_str(paragraph);
    }

    if !current.is_empty() {
        chunks.push(current.trim().to_string());
    }

    // If we still have chunks that are too large, split by sentences
    let mut final_chunks = Vec::new();
    for chunk in chunks {
        if chunk.len() <= max_size {
            final_chunks.push(chunk);
        } else {
            // Fallback: split by sentences or hard limit
            final_chunks.extend(split_by_sentences(&chunk, max_size));
        }
    }

    final_chunks
}

/// Split text by sentences.
fn split_by_sentences(text: &str, max_size: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();

    // Simple sentence splitting (. ! ?)
    for part in text.split_inclusive(['.', '!', '?']) {
        if current.len() + part.len() > max_size && !current.is_empty() {
            chunks.push(current.trim().to_string());
            current = String::new();
        }
        current.push_str(part);
    }

    if !current.is_empty() {
        chunks.push(current.trim().to_string());
    }

    chunks
}

/// Build parent-child relationships between chunks.
fn build_parent_relationships(chunks: &mut [DocChunk]) {
    // Create a map of heading levels to chunk IDs
    let mut level_stack: Vec<(u8, Id)> = Vec::new();

    for chunk in chunks.iter_mut() {
        // Find parent (nearest chunk with lower level)
        while level_stack
            .last()
            .map(|(l, _)| *l >= chunk.heading_level)
            .unwrap_or(false)
        {
            level_stack.pop();
        }

        if let Some((_, parent_id)) = level_stack.last() {
            chunk.parent_id = Some(*parent_id);
        }

        level_stack.push((chunk.heading_level, chunk.id));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_content;

    #[test]
    fn test_chunk_document() {
        let content = r#"# Main Title

Intro paragraph.

## Section One

Content for section one with enough text to be meaningful.
This is additional content to meet the minimum chunk size.

### Subsection A

Details about subsection A.
More content here to reach minimum.

## Section Two

Another section with its own content.
This section also has multiple lines.
"#;

        let doc = parse_content("test.md".to_string(), content.to_string()).unwrap();
        let source = DocSource::local_file("test.md");
        let config = ChunkerConfig {
            short_document_char_limit: 0,
            min_chunk_size: 10, // Lower for testing
            ..Default::default()
        };

        let chunks = chunk_document(&doc, &source, &config);

        assert!(!chunks.is_empty());

        // Verify heading paths
        let headings: Vec<_> = chunks.iter().map(|c| c.heading_path.as_str()).collect();
        assert!(headings.iter().any(|h| h.contains("Section One")));
        assert!(headings.iter().any(|h| h.contains("Subsection A")));
    }

    #[test]
    fn test_short_document_indexes_as_whole_document_chunk() {
        let content = r#"# Tiny Note

This is a short note with one durable idea.

## Detail

The detail stays with the rest of the note.
"#;

        let doc = parse_content("tiny.md".to_string(), content.to_string()).unwrap();
        let source = DocSource::local_file("tiny.md");
        let chunks = chunk_document(&doc, &source, &ChunkerConfig::default());

        assert_eq!(
            chunking_strategy(&doc, &ChunkerConfig::default()),
            ChunkingStrategy::WholeDocument
        );
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].heading_path, "# Tiny Note");
        assert!(chunks[0].content.contains("## Detail"));
        assert!(chunks[0].content.starts_with("# Tiny Note"));
    }

    #[test]
    fn test_unstructured_long_document_is_not_dropped() {
        let content = "unstructured memory line. ".repeat(300);
        let doc = parse_content("plain.md".to_string(), content).unwrap();
        let source = DocSource::local_file("plain.md");
        let config = ChunkerConfig {
            short_document_char_limit: 0,
            min_chunk_size: 10,
            max_chunk_size: 200,
            ..Default::default()
        };

        let chunks = chunk_document(&doc, &source, &config);

        assert_eq!(
            chunking_strategy(&doc, &config),
            ChunkingStrategy::SyntheticSections
        );
        assert!(chunks.len() > 1);
        assert!(chunks
            .iter()
            .all(|chunk| chunk.heading_path.starts_with("# plain")));
    }

    #[test]
    fn test_small_non_empty_sections_are_not_dropped() {
        let content = r#"# Big Document

## Preference

No AI names in commit messages.

## Long Section

Longer material that forces section-based chunking and keeps the document above the whole-document
threshold for this test.
"#;

        let doc = parse_content("preferences.md".to_string(), content.to_string()).unwrap();
        let source = DocSource::local_file("preferences.md");
        let config = ChunkerConfig {
            short_document_char_limit: 0,
            min_chunk_size: 100,
            ..Default::default()
        };

        let chunks = chunk_document(&doc, &source, &config);

        assert!(chunks
            .iter()
            .any(|chunk| chunk.content.contains("No AI names in commit messages.")));
    }

    #[test]
    fn test_split_large_section() {
        let section = Section {
            heading: "Test".to_string(),
            level: 2,
            heading_path: "## Test".to_string(),
            content: "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.".to_string(),
            start_line: 1,
            end_line: 5,
        };

        let chunks = split_large_section(&section, 30);
        assert!(chunks.len() > 1);
    }
}
