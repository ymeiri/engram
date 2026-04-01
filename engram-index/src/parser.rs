//! Markdown document parsing.
//!
//! Parses markdown documents and extracts structure (headings, sections).

use crate::error::{IndexError, IndexResult};
use std::path::Path;

/// A parsed markdown document.
#[derive(Debug, Clone)]
pub struct ParsedDocument {
    /// Original file path.
    pub path: String,
    /// Document title (first H1 or filename).
    pub title: String,
    /// All sections in the document.
    pub sections: Vec<Section>,
    /// Raw content.
    pub raw_content: String,
}

/// A section of a markdown document.
#[derive(Debug, Clone)]
pub struct Section {
    /// Heading text (without the # prefix).
    pub heading: String,
    /// Heading level (1-6).
    pub level: u8,
    /// Full heading path (e.g., "# Main > ## Setup > ### Install").
    pub heading_path: String,
    /// Section content (excluding subsections).
    pub content: String,
    /// Start line number (1-indexed).
    pub start_line: u32,
    /// End line number (1-indexed).
    pub end_line: u32,
}

/// Parse a markdown file.
///
/// # Errors
///
/// Returns an error if the file cannot be read.
pub fn parse_file(path: impl AsRef<Path>) -> IndexResult<ParsedDocument> {
    let path = path.as_ref();
    let content = std::fs::read_to_string(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            IndexError::FileNotFound(path.display().to_string())
        } else {
            IndexError::Io(e)
        }
    })?;

    parse_content(path.display().to_string(), content)
}

/// Parse markdown content.
pub fn parse_content(path: String, content: String) -> IndexResult<ParsedDocument> {
    let mut sections = Vec::new();
    let mut heading_stack: Vec<(u8, String)> = Vec::new();

    let mut current_section: Option<SectionBuilder> = None;
    let mut line_num: u32 = 0;

    for (idx, line) in content.lines().enumerate() {
        line_num = (idx + 1) as u32;

        if let Some((level, heading)) = parse_heading(line) {
            // Finish previous section
            if let Some(builder) = current_section.take() {
                sections.push(builder.finish(line_num - 1));
            }

            // Update heading stack
            while heading_stack
                .last()
                .map(|(l, _)| *l >= level)
                .unwrap_or(false)
            {
                heading_stack.pop();
            }
            heading_stack.push((level, heading.clone()));

            // Build heading path
            let heading_path = heading_stack
                .iter()
                .map(|(l, h)| format!("{} {}", "#".repeat(*l as usize), h))
                .collect::<Vec<_>>()
                .join(" > ");

            // Start new section
            current_section = Some(SectionBuilder {
                heading,
                level,
                heading_path,
                start_line: line_num,
                content_lines: Vec::new(),
            });
        } else if let Some(ref mut builder) = current_section {
            // Store owned strings to avoid lifetime issues
            builder.content_lines.push(line.to_string());
        }
    }

    // Finish last section
    if let Some(builder) = current_section.take() {
        sections.push(builder.finish(line_num));
    }

    // Extract title
    let title = sections
        .iter()
        .find(|s| s.level == 1)
        .map(|s| s.heading.clone())
        .unwrap_or_else(|| {
            Path::new(&path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled")
                .to_string()
        });

    Ok(ParsedDocument {
        path,
        title,
        sections,
        raw_content: content,
    })
}

/// Parse a markdown heading line.
///
/// Markdown requires a space after the `#` symbols for a valid heading.
fn parse_heading(line: &str) -> Option<(u8, String)> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('#') {
        return None;
    }

    let level = trimmed.chars().take_while(|c| *c == '#').count();
    if level > 6 || level == 0 {
        return None;
    }

    // Check for required space after # symbols
    let rest = &trimmed[level..];
    if !rest.starts_with(' ') && !rest.starts_with('\t') {
        return None;
    }

    let heading = rest.trim().to_string();
    if heading.is_empty() {
        return None;
    }

    Some((level as u8, heading))
}

/// Builder for sections.
struct SectionBuilder {
    heading: String,
    level: u8,
    heading_path: String,
    start_line: u32,
    content_lines: Vec<String>,
}

impl SectionBuilder {
    fn finish(self, end_line: u32) -> Section {
        let content = self.content_lines.join("\n").trim().to_string();

        Section {
            heading: self.heading,
            level: self.level,
            heading_path: self.heading_path,
            content,
            start_line: self.start_line,
            end_line,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_heading() {
        assert_eq!(parse_heading("# Hello"), Some((1, "Hello".to_string())));
        assert_eq!(parse_heading("## World"), Some((2, "World".to_string())));
        assert_eq!(parse_heading("### Test"), Some((3, "Test".to_string())));
        assert_eq!(parse_heading("Not a heading"), None);
        assert_eq!(parse_heading("#NoSpace"), None);
        assert_eq!(parse_heading(""), None);
    }

    #[test]
    fn test_parse_content() {
        let content = r#"# Main Title

Some intro text.

## Section One

Content for section one.

### Subsection

More details here.

## Section Two

Another section.
"#;

        let doc = parse_content("test.md".to_string(), content.to_string()).unwrap();

        assert_eq!(doc.title, "Main Title");
        assert_eq!(doc.sections.len(), 4);

        assert_eq!(doc.sections[0].heading, "Main Title");
        assert_eq!(doc.sections[0].level, 1);

        assert_eq!(doc.sections[1].heading, "Section One");
        assert_eq!(doc.sections[1].level, 2);
        assert_eq!(
            doc.sections[1].heading_path,
            "# Main Title > ## Section One"
        );

        assert_eq!(doc.sections[2].heading, "Subsection");
        assert_eq!(doc.sections[2].level, 3);
        assert_eq!(
            doc.sections[2].heading_path,
            "# Main Title > ## Section One > ### Subsection"
        );
    }
}
