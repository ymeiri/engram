//! Integration tests for Layer 3: Document Chunks.
//!
//! Tests document source management, chunk storage, and vector similarity search.

use engram_core::document::{DocChunk, DocSource, SourceType};
use engram_core::id::Id;
use engram_store::repos::DocumentRepo;
use engram_store::{connect_and_init, StoreConfig};

// =============================================================================
// Test Fixtures
// =============================================================================

async fn setup_repo() -> DocumentRepo {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config).await.expect("Failed to connect");
    let repo = DocumentRepo::new(db);
    repo.init_schema().await.expect("Failed to init schema");
    repo
}

// =============================================================================
// DocSource CRUD Tests
// =============================================================================

#[tokio::test]
async fn test_save_and_get_source() {
    let repo = setup_repo().await;

    let source = DocSource::local_file("/path/to/README.md").with_title("Project README");

    repo.save_source(&source)
        .await
        .expect("Failed to save source");

    let retrieved = repo
        .get_source(&source.id)
        .await
        .expect("Failed to get source");

    assert_eq!(retrieved.id, source.id);
    assert_eq!(retrieved.path_or_url, "/path/to/README.md");
    assert_eq!(retrieved.title, Some("Project README".to_string()));
    assert_eq!(retrieved.source_type, SourceType::LocalFile);
}

#[tokio::test]
async fn test_confluence_source() {
    let repo = setup_repo().await;

    let source = DocSource::confluence("https://wiki.example.com/page/123", "TEAM")
        .with_title("Team Documentation");

    repo.save_source(&source)
        .await
        .expect("Failed to save source");

    let retrieved = repo
        .get_source(&source.id)
        .await
        .expect("Failed to get source");

    assert_eq!(retrieved.source_type, SourceType::Confluence);
    assert_eq!(retrieved.space_key, Some("TEAM".to_string()));
}

#[tokio::test]
async fn test_find_source_by_path() {
    let repo = setup_repo().await;

    let source1 = DocSource::local_file("/docs/api.md");
    let source2 = DocSource::local_file("/docs/setup.md");

    repo.save_source(&source1).await.unwrap();
    repo.save_source(&source2).await.unwrap();

    let found = repo
        .find_source_by_path("/docs/api.md")
        .await
        .expect("Query failed");

    assert!(found.is_some());
    assert_eq!(found.unwrap().id, source1.id);

    let not_found = repo
        .find_source_by_path("/docs/nonexistent.md")
        .await
        .expect("Query failed");

    assert!(not_found.is_none());
}

#[tokio::test]
async fn test_delete_source() {
    let repo = setup_repo().await;

    let source = DocSource::local_file("/path/to/delete.md");
    repo.save_source(&source).await.unwrap();

    // Verify it exists
    assert!(repo.get_source(&source.id).await.is_ok());

    // Delete it
    repo.delete_source(&source.id)
        .await
        .expect("Failed to delete");

    // Verify it's gone
    let result = repo.get_source(&source.id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_source_not_found() {
    let repo = setup_repo().await;

    let fake_id = Id::new();
    let result = repo.get_source(&fake_id).await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("not found") || err.contains("NotFound"),
        "Expected not found error, got: {}",
        err
    );
}

// =============================================================================
// DocChunk Storage Tests
// =============================================================================

#[tokio::test]
async fn test_save_and_get_chunks() {
    let repo = setup_repo().await;

    // Create source first
    let source = DocSource::local_file("/docs/guide.md");
    repo.save_source(&source).await.unwrap();

    // Create chunks with embeddings
    let chunk1 = DocChunk::new(
        source.id.clone(),
        "# Guide",
        1,
        "This is the main guide content.",
    )
    .with_lines(1, 10);

    let chunk2 = DocChunk::new(
        source.id.clone(),
        "# Guide > ## Installation",
        2,
        "Run cargo install to get started.",
    )
    .with_lines(11, 20);

    // Simple embeddings for testing
    let embedding1 = vec![0.1, 0.2, 0.3, 0.4, 0.5];
    let embedding2 = vec![0.2, 0.3, 0.4, 0.5, 0.6];

    repo.save_chunks(
        &source.id,
        vec![(chunk1.clone(), embedding1), (chunk2.clone(), embedding2)],
    )
    .await
    .expect("Failed to save chunks");

    // Retrieve chunks
    let chunks = repo
        .get_chunks_for_source(&source.id)
        .await
        .expect("Failed to get chunks");

    assert_eq!(chunks.len(), 2);
    // Chunks should be ordered by start_line
    assert_eq!(chunks[0].heading_path, "# Guide");
    assert_eq!(chunks[1].heading_path, "# Guide > ## Installation");
}

#[tokio::test]
async fn test_replace_chunks() {
    let repo = setup_repo().await;

    let source = DocSource::local_file("/docs/replace.md");
    repo.save_source(&source).await.unwrap();

    // Save initial chunks
    let chunk1 = DocChunk::new(source.id.clone(), "# Old", 1, "Old content");
    repo.save_chunks(&source.id, vec![(chunk1, vec![0.1, 0.2, 0.3])])
        .await
        .unwrap();

    // Verify initial state
    let initial = repo.get_chunks_for_source(&source.id).await.unwrap();
    assert_eq!(initial.len(), 1);
    assert_eq!(initial[0].heading_path, "# Old");

    // Replace with new chunks
    let chunk2 = DocChunk::new(source.id.clone(), "# New", 1, "New content");
    let chunk3 = DocChunk::new(
        source.id.clone(),
        "# New > ## Section",
        2,
        "Section content",
    );
    repo.save_chunks(
        &source.id,
        vec![(chunk2, vec![0.2, 0.3, 0.4]), (chunk3, vec![0.3, 0.4, 0.5])],
    )
    .await
    .unwrap();

    // Verify replacement
    let replaced = repo.get_chunks_for_source(&source.id).await.unwrap();
    assert_eq!(replaced.len(), 2);
    assert_eq!(replaced[0].heading_path, "# New");
}

#[tokio::test]
async fn test_delete_source_deletes_chunks() {
    let repo = setup_repo().await;

    let source = DocSource::local_file("/docs/cascade.md");
    repo.save_source(&source).await.unwrap();

    let chunk = DocChunk::new(source.id.clone(), "# Test", 1, "Content");
    repo.save_chunks(&source.id, vec![(chunk, vec![0.1, 0.2, 0.3])])
        .await
        .unwrap();

    // Verify chunks exist
    let before = repo.get_chunks_for_source(&source.id).await.unwrap();
    assert_eq!(before.len(), 1);

    // Delete source (should cascade to chunks)
    repo.delete_source(&source.id).await.unwrap();

    // Verify chunks are gone
    let after = repo.get_chunks_for_source(&source.id).await.unwrap();
    assert_eq!(after.len(), 0);
}

// =============================================================================
// Vector Similarity Search Tests
// =============================================================================

#[tokio::test]
async fn test_vector_similarity_search() {
    let repo = setup_repo().await;

    // Create source
    let source = DocSource::local_file("/docs/vectors.md").with_title("Vector Test");
    repo.save_source(&source).await.unwrap();

    // Create chunks with distinct embeddings
    let chunk_rust = DocChunk::new(source.id.clone(), "# Rust", 1, "Rust programming language");
    let chunk_python = DocChunk::new(
        source.id.clone(),
        "# Python",
        1,
        "Python programming language",
    );
    let chunk_go = DocChunk::new(source.id.clone(), "# Go", 1, "Go programming language");

    // Embeddings: rust is similar to query, python is medium, go is different
    let rust_embedding = vec![0.9, 0.8, 0.7, 0.6, 0.5];
    let python_embedding = vec![0.5, 0.5, 0.5, 0.5, 0.5];
    let go_embedding = vec![0.1, 0.2, 0.3, 0.4, 0.5];

    repo.save_chunks(
        &source.id,
        vec![
            (chunk_rust, rust_embedding),
            (chunk_python, python_embedding),
            (chunk_go, go_embedding),
        ],
    )
    .await
    .unwrap();

    // Search with embedding similar to rust
    let query_embedding = vec![0.95, 0.85, 0.75, 0.65, 0.55];
    let results = repo
        .search_similar(&query_embedding, 3)
        .await
        .expect("Search failed");

    assert_eq!(results.len(), 3);
    // Results should be ordered by similarity (highest first)
    // The rust chunk should be most similar to our query
    assert!(results[0].score > results[1].score);
    assert!(results[1].score > results[2].score);
}

#[tokio::test]
async fn test_search_with_threshold() {
    let repo = setup_repo().await;

    let source = DocSource::local_file("/docs/threshold.md");
    repo.save_source(&source).await.unwrap();

    let chunk1 = DocChunk::new(source.id.clone(), "# Similar", 1, "Very similar content");
    let chunk2 = DocChunk::new(
        source.id.clone(),
        "# Different",
        1,
        "Very different content",
    );

    // chunk1 embedding is similar to query, chunk2 is orthogonal
    repo.save_chunks(
        &source.id,
        vec![
            (chunk1, vec![1.0, 0.0, 0.0, 0.0, 0.0]),
            (chunk2, vec![0.0, 1.0, 0.0, 0.0, 0.0]),
        ],
    )
    .await
    .unwrap();

    let query = vec![1.0, 0.0, 0.0, 0.0, 0.0];

    // Search with high threshold - should only return similar chunk
    let results = repo
        .search_similar_threshold(&query, 10, 0.9)
        .await
        .expect("Search failed");

    // Only the similar chunk should pass the threshold
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].chunk.heading_path, "# Similar");
}

#[tokio::test]
async fn test_search_with_limit() {
    let repo = setup_repo().await;

    let source = DocSource::local_file("/docs/many.md");
    repo.save_source(&source).await.unwrap();

    // Create many chunks
    let mut chunks = Vec::new();
    for i in 0..10 {
        let chunk = DocChunk::new(
            source.id.clone(),
            &format!("# Section {}", i),
            1,
            &format!("Content {}", i),
        );
        let embedding = vec![0.1 * i as f32, 0.2, 0.3, 0.4, 0.5];
        chunks.push((chunk, embedding));
    }

    repo.save_chunks(&source.id, chunks).await.unwrap();

    let query = vec![0.5, 0.2, 0.3, 0.4, 0.5];

    // Limit to 3 results
    let results = repo.search_similar(&query, 3).await.expect("Search failed");

    assert_eq!(results.len(), 3);
}

#[tokio::test]
async fn test_search_empty_database() {
    let repo = setup_repo().await;

    let query = vec![0.1, 0.2, 0.3, 0.4, 0.5];
    let results = repo
        .search_similar(&query, 10)
        .await
        .expect("Search failed");

    assert!(results.is_empty());
}

// =============================================================================
// Statistics Tests
// =============================================================================

#[tokio::test]
async fn test_document_stats() {
    let repo = setup_repo().await;

    // Initially empty
    let stats = repo.stats().await.expect("Failed to get stats");
    assert_eq!(stats.source_count, 0);
    assert_eq!(stats.chunk_count, 0);

    // Add some data
    let source1 = DocSource::local_file("/docs/s1.md");
    let source2 = DocSource::local_file("/docs/s2.md");
    repo.save_source(&source1).await.unwrap();
    repo.save_source(&source2).await.unwrap();

    let chunk1 = DocChunk::new(source1.id.clone(), "# A", 1, "Content A");
    let chunk2 = DocChunk::new(source1.id.clone(), "# B", 1, "Content B");
    let chunk3 = DocChunk::new(source2.id.clone(), "# C", 1, "Content C");

    repo.save_chunks(
        &source1.id,
        vec![(chunk1, vec![0.1, 0.2, 0.3]), (chunk2, vec![0.2, 0.3, 0.4])],
    )
    .await
    .unwrap();
    repo.save_chunks(&source2.id, vec![(chunk3, vec![0.3, 0.4, 0.5])])
        .await
        .unwrap();

    let stats = repo.stats().await.expect("Failed to get stats");
    assert_eq!(stats.source_count, 2);
    assert_eq!(stats.chunk_count, 3);
}

// =============================================================================
// Edge Cases
// =============================================================================

#[tokio::test]
async fn test_special_characters_in_path() {
    let repo = setup_repo().await;

    let paths = [
        "/path/with spaces/file.md",
        "/path/with-dashes/file.md",
        "/path/with_underscores/file.md",
        "/path/with.dots/file.md",
    ];

    for path in paths {
        let source = DocSource::local_file(path);
        repo.save_source(&source)
            .await
            .expect(&format!("Failed with path: {}", path));

        let found = repo.find_source_by_path(path).await.unwrap();
        assert!(found.is_some(), "Should find source with path: {}", path);
    }
}

#[tokio::test]
async fn test_long_content_chunk() {
    let repo = setup_repo().await;

    let source = DocSource::local_file("/docs/long.md");
    repo.save_source(&source).await.unwrap();

    let long_content = "A".repeat(10000);
    let chunk = DocChunk::new(source.id.clone(), "# Long", 1, &long_content);

    repo.save_chunks(&source.id, vec![(chunk, vec![0.1, 0.2, 0.3])])
        .await
        .expect("Should handle long content");

    let chunks = repo.get_chunks_for_source(&source.id).await.unwrap();
    assert_eq!(chunks[0].content.len(), 10000);
}

#[tokio::test]
async fn test_hierarchical_chunks() {
    let repo = setup_repo().await;

    let source = DocSource::local_file("/docs/hierarchy.md");
    repo.save_source(&source).await.unwrap();

    let parent = DocChunk::new(source.id.clone(), "# Main", 1, "Main content");
    let parent_id = parent.id.clone();

    let child = DocChunk::new(source.id.clone(), "# Main > ## Sub", 2, "Sub content")
        .with_parent(parent_id.clone());

    repo.save_chunks(
        &source.id,
        vec![(parent, vec![0.1, 0.2, 0.3]), (child, vec![0.2, 0.3, 0.4])],
    )
    .await
    .unwrap();

    let chunks = repo.get_chunks_for_source(&source.id).await.unwrap();
    assert_eq!(chunks.len(), 2);

    // Find the child and verify parent link
    let child_chunk = chunks.iter().find(|c| c.heading_level == 2).unwrap();
    assert_eq!(child_chunk.parent_id, Some(parent_id));
}
