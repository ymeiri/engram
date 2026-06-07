use engram_core::document::DocSearchResult;

/// Merge semantic chunk hits with source-metadata known-item hits.
///
/// Lexical source matches are one representative chunk per source. If semantic search already
/// returned that source, promote the first semantic hit rather than returning a duplicate.
pub(crate) fn merge_document_results(
    mut semantic_results: Vec<DocSearchResult>,
    lexical_results: Vec<DocSearchResult>,
    limit: usize,
) -> Vec<DocSearchResult> {
    if limit == 0 {
        return Vec::new();
    }

    for lexical in lexical_results {
        if let Some(existing) = semantic_results
            .iter_mut()
            .find(|result| result.source.id == lexical.source.id)
        {
            if lexical.score > existing.score {
                existing.score = lexical.score;
            }
        } else {
            semantic_results.push(lexical);
        }
    }

    semantic_results.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.source.path_or_url.cmp(&right.source.path_or_url))
            .then_with(|| left.chunk.id.to_string().cmp(&right.chunk.id.to_string()))
    });
    semantic_results.truncate(limit);
    semantic_results
}

#[cfg(test)]
mod tests {
    use super::*;
    use engram_core::document::{DocChunk, DocSource};

    #[test]
    fn exact_source_match_can_outrank_semantic_hit() {
        let semantic = result("/docs/semantic.md", "Semantic", "# Semantic", 0.91);
        let lexical = result(
            "/docs/BRAIN_HARNESS_T202_HANDOFF_SUPERSESSION_MCP_BOUNDARY_VALIDATION_2026-06-04.md",
            "T202 Handoff Supersession MCP Boundary Validation",
            "# T202",
            1.0,
        );

        let merged = merge_document_results(vec![semantic], vec![lexical], 10);

        assert_eq!(
            merged[0].source.title.as_deref(),
            Some("T202 Handoff Supersession MCP Boundary Validation")
        );
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn lexical_match_promotes_existing_source_without_duplicate() {
        let semantic = result("/docs/t202.md", "T202", "# Semantic Chunk", 0.61);
        let mut lexical = result("/docs/t202.md", "T202", "# Representative Chunk", 1.0);
        lexical.source.id = semantic.source.id;

        let merged = merge_document_results(vec![semantic], vec![lexical], 10);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].score, 1.0);
        assert_eq!(merged[0].chunk.heading_path, "# Semantic Chunk");
    }

    #[test]
    fn merged_results_preserve_limit() {
        let semantic = result("/docs/semantic.md", "Semantic", "# Semantic", 0.9);
        let lexical = result("/docs/t202.md", "T202", "# T202", 1.0);

        let merged = merge_document_results(vec![semantic], vec![lexical], 1);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].source.title.as_deref(), Some("T202"));
    }

    fn result(path: &str, title: &str, heading: &str, score: f32) -> DocSearchResult {
        let source = DocSource::local_file(path).with_title(title);
        let chunk = DocChunk::new(source.id, heading, 1, format!("Content for {title}"));
        DocSearchResult {
            chunk,
            source,
            score,
        }
    }
}
