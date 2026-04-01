//! End-to-end semantic search integration tests.
//!
//! These tests verify the complete semantic search pipeline:
//! 1. Document parsing and chunking
//! 2. Embedding generation using real models
//! 3. Vector storage in SurrealDB
//! 4. Semantic similarity search with natural language queries
//!
//! NOTE: Tests are ignored by default because they require downloading
//! the embedding model (~90MB). Run with:
//!   cargo test -p engram-tests --test semantic_search_tests -- --ignored

use engram_embed::Embedder;
use engram_index::service::DocumentService;
use engram_store::{connect_and_init, StoreConfig};
use std::io::Write;
use tempfile::TempDir;

// =============================================================================
// Test Fixtures
// =============================================================================

async fn setup_service() -> (DocumentService, TempDir) {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config).await.expect("Failed to connect");
    let service = DocumentService::with_defaults(db).expect("Failed to create service");
    service.init_schema().await.expect("Failed to init schema");
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    (service, temp_dir)
}

fn create_test_file(dir: &std::path::Path, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    let mut file = std::fs::File::create(&path).expect("Failed to create file");
    file.write_all(content.as_bytes()).expect("Failed to write");
    path
}

// =============================================================================
// Full Pipeline Tests
// =============================================================================

/// Test that embedding generation works with the real model.
#[tokio::test]
#[ignore = "requires model download (~90MB)"]
async fn test_embedding_generation() {
    let embedder = Embedder::default_model().expect("Failed to load embedder");

    // Test single embedding
    let embedding = embedder
        .embed("Rust programming language")
        .expect("Failed to embed");
    assert_eq!(embedding.len(), 384, "Expected 384-dimensional embedding");

    // Verify embedding values are normalized (roughly between -1 and 1)
    assert!(embedding.iter().all(|v| *v >= -2.0 && *v <= 2.0));
}

/// Test that semantically similar texts produce similar embeddings.
#[tokio::test]
#[ignore = "requires model download (~90MB)"]
async fn test_embedding_similarity() {
    let embedder = Embedder::default_model().expect("Failed to load embedder");

    // Semantically similar texts
    let rust_embedding = embedder
        .embed("Rust is a systems programming language")
        .unwrap();
    let rust_similar = embedder
        .embed("Rust programming for systems development")
        .unwrap();

    // Semantically different text
    let cooking_embedding = embedder
        .embed("How to bake chocolate chip cookies")
        .unwrap();

    // Calculate cosine similarities
    let similarity_same = cosine_similarity(&rust_embedding, &rust_similar);
    let similarity_diff = cosine_similarity(&rust_embedding, &cooking_embedding);

    // Similar texts should have higher similarity than different topics
    assert!(
        similarity_same > similarity_diff,
        "Similar texts should have higher similarity. Same topic: {}, Different topic: {}",
        similarity_same,
        similarity_diff
    );

    // Sanity check: similar texts should have high similarity (> 0.7)
    assert!(
        similarity_same > 0.7,
        "Similar texts should have cosine similarity > 0.7, got {}",
        similarity_same
    );
}

/// Test the complete indexing and search pipeline.
#[tokio::test]
#[ignore = "requires model download (~90MB)"]
async fn test_full_index_and_search_pipeline() {
    let (service, temp_dir) = setup_service().await;

    // Create test documents with distinct topics
    let rust_doc = create_test_file(
        temp_dir.path(),
        "rust_guide.md",
        r#"# Rust Programming Guide

Rust is a systems programming language focused on safety, speed, and concurrency.

## Memory Safety

Rust's ownership system ensures memory safety without garbage collection.
The borrow checker prevents data races at compile time.

## Concurrency

Rust makes concurrent programming safer through its ownership model.
Threads can share data safely using Arc and Mutex.
"#,
    );

    let cooking_doc = create_test_file(
        temp_dir.path(),
        "cooking_recipes.md",
        r#"# Cooking Recipes

A collection of delicious recipes for home cooking.

## Chocolate Cake

Mix flour, sugar, cocoa powder, and eggs to make a rich chocolate cake.
Bake at 350°F for 30 minutes.

## Pasta Carbonara

Cook spaghetti, then mix with eggs, cheese, and crispy bacon.
The heat from the pasta cooks the eggs into a creamy sauce.
"#,
    );

    // Index both documents
    service
        .index_file(&rust_doc)
        .await
        .expect("Failed to index rust doc");
    service
        .index_file(&cooking_doc)
        .await
        .expect("Failed to index cooking doc");

    // Verify stats
    let stats = service.stats().await.expect("Failed to get stats");
    assert_eq!(stats.source_count, 2, "Should have 2 indexed sources");
    assert!(stats.chunk_count >= 4, "Should have at least 4 chunks");
    assert_eq!(stats.embedding_dimension, 384);

    // Search for Rust-related content
    let rust_results = service
        .search("memory safety and ownership", 5)
        .await
        .expect("Search failed");

    assert!(
        !rust_results.is_empty(),
        "Should find results for Rust query"
    );

    // The top result should be from the Rust document
    let top_result = &rust_results[0];
    assert!(
        top_result.source.path_or_url.contains("rust"),
        "Top result for 'memory safety' should be from Rust doc, got: {}",
        top_result.source.path_or_url
    );

    // Search for cooking-related content
    let cooking_results = service
        .search("baking chocolate desserts", 5)
        .await
        .expect("Search failed");

    assert!(
        !cooking_results.is_empty(),
        "Should find results for cooking query"
    );

    // The top result should be from the cooking document
    let top_cooking = &cooking_results[0];
    assert!(
        top_cooking.source.path_or_url.contains("cooking"),
        "Top result for 'baking' should be from cooking doc, got: {}",
        top_cooking.source.path_or_url
    );
}

/// Test that search results are ranked by semantic relevance.
#[tokio::test]
#[ignore = "requires model download (~90MB)"]
async fn test_search_ranking_by_relevance() {
    let (service, temp_dir) = setup_service().await;

    // Create documents with varying relevance to "database optimization"
    let highly_relevant = create_test_file(
        temp_dir.path(),
        "db_optimization.md",
        r#"# Database Optimization Guide

## Query Performance

Optimize SQL queries by using indexes and avoiding full table scans.
Use EXPLAIN to analyze query execution plans.

## Index Strategy

Create indexes on frequently queried columns.
Composite indexes improve multi-column lookups.
"#,
    );

    let moderately_relevant = create_test_file(
        temp_dir.path(),
        "backend_guide.md",
        r#"# Backend Development

## API Design

Design RESTful APIs with proper HTTP methods and status codes.
Use pagination for large result sets to improve performance.
Implement proper error handling and validation for all endpoints.
Document your API using OpenAPI specifications for better maintainability.

## Database Access

Use connection pooling for efficient database access and resource management.
Consider caching frequently accessed data using Redis or Memcached.
Implement proper transaction handling for data integrity.
"#,
    );

    let not_relevant = create_test_file(
        temp_dir.path(),
        "frontend_guide.md",
        r#"# Frontend Development

## CSS Styling

Use flexbox and grid for responsive layouts that work across devices.
Keep styles modular with CSS-in-JS libraries like styled-components or emotion.
Implement a consistent design system with reusable style tokens and variables.
Consider accessibility and ensure proper color contrast ratios.

## React Components

Build reusable components with proper props and TypeScript interfaces.
Use hooks for state management and side effects in functional components.
Implement proper error boundaries to gracefully handle component failures.
"#,
    );

    // Index all documents
    service.index_file(&highly_relevant).await.unwrap();
    service.index_file(&moderately_relevant).await.unwrap();
    service.index_file(&not_relevant).await.unwrap();

    // Search for database optimization
    let results = service
        .search("database query optimization and indexing", 10)
        .await
        .expect("Search failed");

    assert!(
        results.len() >= 3,
        "Should return multiple results (got {})",
        results.len()
    );

    // Find the positions of each document in results
    let db_opt_pos = results
        .iter()
        .position(|r| r.source.path_or_url.contains("db_optimization"));
    let backend_pos = results
        .iter()
        .position(|r| r.source.path_or_url.contains("backend_guide"));
    let frontend_pos = results
        .iter()
        .position(|r| r.source.path_or_url.contains("frontend_guide"));

    // The database optimization doc should rank highest
    assert!(
        db_opt_pos.is_some(),
        "Database optimization doc should appear in results"
    );

    // Highly relevant should rank higher than moderately relevant
    if let (Some(db_pos), Some(be_pos)) = (db_opt_pos, backend_pos) {
        assert!(
            db_pos < be_pos,
            "DB optimization doc (pos {}) should rank higher than backend doc (pos {})",
            db_pos,
            be_pos
        );
    }

    // Relevant docs should rank higher than irrelevant
    if let (Some(db_pos), Some(fe_pos)) = (db_opt_pos, frontend_pos) {
        assert!(
            db_pos < fe_pos,
            "DB optimization doc (pos {}) should rank higher than frontend doc (pos {})",
            db_pos,
            fe_pos
        );
    }

    // Check that scores decrease as relevance decreases
    if results.len() >= 2 {
        assert!(
            results[0].score >= results[1].score,
            "Scores should be in descending order"
        );
    }
}

/// Test search with score threshold filtering.
#[tokio::test]
#[ignore = "requires model download (~90MB)"]
async fn test_search_with_threshold() {
    let (service, temp_dir) = setup_service().await;

    let doc = create_test_file(
        temp_dir.path(),
        "specific_topic.md",
        r#"# Kubernetes Deployment

Deploy applications to Kubernetes clusters using kubectl.
Write YAML manifests for pods, services, and deployments.
"#,
    );

    service.index_file(&doc).await.unwrap();

    // Search with high threshold - should filter low-relevance results
    let high_threshold_results = service
        .search_threshold("kubernetes pod deployment yaml", 10, 0.5)
        .await
        .expect("Search failed");

    // All results should have score >= 0.5
    for result in &high_threshold_results {
        assert!(
            result.score >= 0.5,
            "Result score {} should be >= 0.5",
            result.score
        );
    }

    // Search with very high threshold for unrelated topic
    let unrelated_results = service
        .search_threshold("chocolate cake recipe baking", 10, 0.8)
        .await
        .expect("Search failed");

    // Should have fewer or no results for unrelated topic with high threshold
    assert!(
        unrelated_results.is_empty() || unrelated_results.len() < high_threshold_results.len(),
        "Unrelated search with high threshold should return fewer results"
    );
}

/// Test reindexing behavior.
#[tokio::test]
#[ignore = "requires model download (~90MB)"]
async fn test_reindexing() {
    let (service, temp_dir) = setup_service().await;

    let doc_path = create_test_file(
        temp_dir.path(),
        "evolving_doc.md",
        r#"# Version 1

Initial content about Python programming.
"#,
    );

    // First index
    service.index_file(&doc_path).await.unwrap();

    let stats1 = service.stats().await.unwrap();
    assert_eq!(stats1.source_count, 1);

    // Search for Python
    let results1 = service.search("Python programming", 5).await.unwrap();
    assert!(!results1.is_empty());

    // Note: Without modifying the file's mtime, reindexing might skip
    // This test primarily verifies the pipeline handles reindex attempts gracefully
    service.index_file(&doc_path).await.unwrap();

    let stats2 = service.stats().await.unwrap();
    assert_eq!(
        stats2.source_count, 1,
        "Should still have 1 source after reindex attempt"
    );
}

/// Test handling of empty and minimal documents.
#[tokio::test]
#[ignore = "requires model download (~90MB)"]
async fn test_edge_cases() {
    let (service, temp_dir) = setup_service().await;

    // Minimal document
    let minimal = create_test_file(temp_dir.path(), "minimal.md", "# Title\n\nOne sentence.");

    // Document with only heading
    let heading_only = create_test_file(temp_dir.path(), "heading_only.md", "# Just a Heading");

    // Document with special characters
    let special_chars = create_test_file(
        temp_dir.path(),
        "special.md",
        "# Special: Chars & Symbols!\n\nContent with émojis 🎉 and unicode: αβγ",
    );

    // All should index without errors
    service
        .index_file(&minimal)
        .await
        .expect("Failed to index minimal doc");
    service
        .index_file(&heading_only)
        .await
        .expect("Failed to index heading-only doc");
    service
        .index_file(&special_chars)
        .await
        .expect("Failed to index special chars doc");

    let stats = service.stats().await.unwrap();
    assert_eq!(stats.source_count, 3, "All documents should be indexed");
}

/// Test directory indexing.
#[tokio::test]
#[ignore = "requires model download (~90MB)"]
async fn test_directory_indexing() {
    let (service, temp_dir) = setup_service().await;

    // Create subdirectory with documents
    let subdir = temp_dir.path().join("docs");
    std::fs::create_dir(&subdir).unwrap();

    create_test_file(&subdir, "doc1.md", "# Document 1\n\nContent about topic A.");
    create_test_file(&subdir, "doc2.md", "# Document 2\n\nContent about topic B.");
    create_test_file(&subdir, "doc3.md", "# Document 3\n\nContent about topic C.");
    create_test_file(&subdir, "not_markdown.txt", "This should be ignored.");

    // Index the directory
    let results = service
        .index_directory(&subdir)
        .await
        .expect("Failed to index directory");

    assert_eq!(results.len(), 3, "Should index 3 markdown files");

    let stats = service.stats().await.unwrap();
    assert_eq!(stats.source_count, 3);
}

// =============================================================================
// Helper Functions
// =============================================================================

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}
