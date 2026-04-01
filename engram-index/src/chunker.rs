//! Document chunking by heading hierarchy.
//!
//! Chunks documents into sections suitable for embedding and retrieval.

use crate::parser::{ParsedDocument, Section};
use engram_core::document::{DocChunk, DocSource};
use engram_core::id::Id;

/// Configuration for the chunker.
#[derive(Debug, Clone)]
pub struct ChunkerConfig {
    /// Minimum heading level to create chunks (1-6).
    /// Level 2 means H2 and below create chunks.
    pub min_chunk_level: u8,

    /// Maximum chunk size in characters.
    pub max_chunk_size: usize,

    /// Minimum chunk size in characters (smaller chunks are merged up).
    pub min_chunk_size: usize,

    /// Include parent heading context in chunks.
    pub include_heading_path: bool,
}

impl Default for ChunkerConfig {
    fn default() -> Self {
        Self {
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
    let mut chunks = Vec::new();

    for section in &doc.sections {
        // Skip sections above the minimum chunk level
        if section.level < config.min_chunk_level {
            // But include H1 content if it exists
            if section.level == 1 && !section.content.is_empty() {
                chunks.push(create_chunk(source.id, section, None));
            }
            continue;
        }

        // Handle content size
        if section.content.len() <= config.max_chunk_size {
            if section.content.len() >= config.min_chunk_size || section.content.is_empty() {
                chunks.push(create_chunk(source.id, section, None));
            }
            // Small chunks are skipped (will be merged in a future enhancement)
        } else {
            // Split large sections into multiple chunks
            let sub_chunks = split_large_section(section, config.max_chunk_size);
            for (idx, content) in sub_chunks.into_iter().enumerate() {
                let mut chunk = create_chunk(source.id, section, None);
                chunk.content = content;
                if idx > 0 {
                    chunk.id = Id::new(); // New ID for split chunks
                    chunk.heading_path = format!("{} (part {})", section.heading_path, idx + 1);
                }
                chunks.push(chunk);
            }
        }
    }

    // Build parent relationships
    build_parent_relationships(&mut chunks);

    chunks
}

/// Create a chunk from a section.
fn create_chunk(source_id: Id, section: &Section, parent_id: Option<Id>) -> DocChunk {
    let mut chunk = DocChunk::new(
        source_id,
        section.heading_path.clone(),
        section.level,
        section.content.clone(),
    )
    .with_lines(section.start_line, section.end_line);

    if let Some(parent) = parent_id {
        chunk = chunk.with_parent(parent);
    }

    chunk
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
