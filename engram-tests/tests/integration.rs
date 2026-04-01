//! Integration tests for engram.
//!
//! These tests verify that all components work together correctly.

use engram_core::{
    entity::{Entity, EntityType},
    knowledge::{DocType, KnowledgeDoc},
};

#[test]
fn test_entity_creation() {
    let entity = Entity::new("test-repo", EntityType::Repo).with_description("A test repository");

    assert_eq!(entity.name, "test-repo");
    assert_eq!(entity.entity_type, EntityType::Repo);
}

#[test]
fn test_knowledge_doc_creation() {
    let doc = KnowledgeDoc::new("Test Doc", DocType::Readme, "# Test\n\nContent here.")
        .with_path("docs/test.md");

    assert_eq!(doc.name, "Test Doc");
    assert_eq!(doc.doc_type, DocType::Readme);
}

// TODO: Add more integration tests as implementation progresses
