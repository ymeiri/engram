//! Integration tests for Layer 6: Knowledge Documents.
//!
//! Tests knowledge document management, file sync, aliases, and events.

use engram_core::id::Id;
use engram_core::knowledge::{
    DocAlias, DocEvent, DocEventType, DocStatus, DocType, FileSync, KnowledgeDoc, SyncStatus,
};
use engram_index::{KnowledgeConfig, KnowledgeService};
use engram_mcp::tools::{self, KnowledgeRequestNew, ToolState};
use engram_store::repos::KnowledgeRepo;
use engram_store::{connect_and_init, StoreConfig};
use std::path::PathBuf;
use tempfile::TempDir;

// =============================================================================
// Test Fixtures
// =============================================================================

async fn setup_repo() -> KnowledgeRepo {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config).await.expect("Failed to connect");
    let repo = KnowledgeRepo::new(db);
    repo.init_schema().await.expect("Failed to init schema");
    repo
}

// =============================================================================
// KnowledgeDoc CRUD Tests
// =============================================================================

#[tokio::test]
async fn test_save_and_get_doc() {
    let repo = setup_repo().await;

    let doc = KnowledgeDoc::new(
        "API Guide",
        DocType::Howto,
        "# API Guide\n\nHow to use the API.",
    )
    .with_path("docs/api-guide.md")
    .with_owner("platform-team");

    repo.save_doc(&doc).await.expect("Failed to save doc");

    let retrieved = repo.get_doc(&doc.id).await.expect("Failed to get doc");

    assert_eq!(retrieved.id, doc.id);
    assert_eq!(retrieved.name, "API Guide");
    assert_eq!(retrieved.doc_type, DocType::Howto);
    assert_eq!(
        retrieved.canonical_path,
        Some("docs/api-guide.md".to_string())
    );
    assert_eq!(retrieved.owner, Some("platform-team".to_string()));
}

#[tokio::test]
async fn test_all_doc_types() {
    let repo = setup_repo().await;

    let types = [
        DocType::Adr,
        DocType::Runbook,
        DocType::Howto,
        DocType::Research,
        DocType::Design,
        DocType::Readme,
        DocType::Changelog,
        DocType::Custom("custom-type".to_string()),
    ];

    for doc_type in types {
        let doc = KnowledgeDoc::new(&format!("Doc {:?}", doc_type), doc_type.clone(), "Content");
        repo.save_doc(&doc)
            .await
            .expect(&format!("Failed for {:?}", doc_type));

        let retrieved = repo.get_doc(&doc.id).await.unwrap();
        assert_eq!(retrieved.doc_type, doc_type);
    }
}

#[tokio::test]
async fn test_find_doc_by_name() {
    let repo = setup_repo().await;

    let doc1 = KnowledgeDoc::new("Unique Name", DocType::Readme, "Content 1");
    let doc2 = KnowledgeDoc::new("Other Name", DocType::Readme, "Content 2");

    repo.save_doc(&doc1).await.unwrap();
    repo.save_doc(&doc2).await.unwrap();

    let found = repo
        .find_doc_by_name("Unique Name")
        .await
        .expect("Query failed");
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, doc1.id);

    let not_found = repo
        .find_doc_by_name("Nonexistent")
        .await
        .expect("Query failed");
    assert!(not_found.is_none());
}

#[tokio::test]
async fn test_find_doc_by_path() {
    let repo = setup_repo().await;

    let doc =
        KnowledgeDoc::new("Setup Guide", DocType::Howto, "Content").with_path("docs/setup.md");

    repo.save_doc(&doc).await.unwrap();

    let found = repo
        .find_doc_by_path("docs/setup.md")
        .await
        .expect("Query failed");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "Setup Guide");
}

#[tokio::test]
async fn test_find_docs_by_hash() {
    let repo = setup_repo().await;

    // Create two docs with same content (same hash)
    let doc1 = KnowledgeDoc::new("Doc A", DocType::Readme, "Identical content");
    let doc2 = KnowledgeDoc::new("Doc B", DocType::Readme, "Identical content");

    // They should have the same hash since content is identical
    assert_eq!(doc1.content_hash, doc2.content_hash);

    repo.save_doc(&doc1).await.unwrap();
    repo.save_doc(&doc2).await.unwrap();

    let duplicates = repo
        .find_docs_by_hash(&doc1.content_hash)
        .await
        .expect("Query failed");
    assert_eq!(duplicates.len(), 2);
}

#[tokio::test]
async fn test_list_docs() {
    let repo = setup_repo().await;

    for i in 0..5 {
        let doc = KnowledgeDoc::new(
            &format!("Doc {}", i),
            DocType::Readme,
            &format!("Content {}", i),
        );
        repo.save_doc(&doc).await.unwrap();
    }

    let docs = repo.list_docs().await.expect("Failed to list");
    assert_eq!(docs.len(), 5);
}

#[tokio::test]
async fn test_list_docs_by_type() {
    let repo = setup_repo().await;

    let howto1 = KnowledgeDoc::new("Howto 1", DocType::Howto, "Content");
    let howto2 = KnowledgeDoc::new("Howto 2", DocType::Howto, "Content");
    let adr = KnowledgeDoc::new("ADR 1", DocType::Adr, "Content");

    repo.save_doc(&howto1).await.unwrap();
    repo.save_doc(&howto2).await.unwrap();
    repo.save_doc(&adr).await.unwrap();

    let howtos = repo
        .list_docs_by_type(&DocType::Howto)
        .await
        .expect("Failed");
    assert_eq!(howtos.len(), 2);

    let adrs = repo.list_docs_by_type(&DocType::Adr).await.expect("Failed");
    assert_eq!(adrs.len(), 1);
}

#[tokio::test]
async fn test_delete_doc() {
    let repo = setup_repo().await;

    let doc = KnowledgeDoc::new("To Delete", DocType::Readme, "Content");
    repo.save_doc(&doc).await.unwrap();

    // Verify exists
    assert!(repo.get_doc(&doc.id).await.is_ok());

    // Delete
    repo.delete_doc(&doc.id).await.expect("Failed to delete");

    // Verify gone
    let result = repo.get_doc(&doc.id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_doc_not_found() {
    let repo = setup_repo().await;

    let fake_id = Id::new();
    let result = repo.get_doc(&fake_id).await;

    assert!(result.is_err());
}

// =============================================================================
// FileSync Tests
// =============================================================================

#[tokio::test]
async fn test_save_and_get_file_sync() {
    let repo = setup_repo().await;

    let sync = FileSync::new("docs/readme.md", "my-repo", "abc123hash");

    repo.save_file_sync(&sync).await.expect("Failed to save");

    let retrieved = repo.get_file_sync(&sync.id).await.expect("Failed to get");

    assert_eq!(retrieved.id, sync.id);
    assert_eq!(retrieved.path, "docs/readme.md");
    assert_eq!(retrieved.repo, "my-repo");
    assert_eq!(retrieved.last_hash, "abc123hash");
    assert_eq!(retrieved.sync_status, SyncStatus::Synced);
}

#[tokio::test]
async fn test_file_sync_with_doc_link() {
    let repo = setup_repo().await;

    let doc = KnowledgeDoc::new("Linked Doc", DocType::Readme, "Content");
    repo.save_doc(&doc).await.unwrap();

    let sync = FileSync::new("docs/linked.md", "repo", "hash123").with_doc(doc.id.clone());

    repo.save_file_sync(&sync).await.unwrap();

    let retrieved = repo.get_file_sync(&sync.id).await.unwrap();
    assert_eq!(retrieved.doc_id, Some(doc.id));
}

#[tokio::test]
async fn test_find_file_sync_by_path_and_repo() {
    let repo = setup_repo().await;

    let sync1 = FileSync::new("docs/file.md", "repo-a", "hash1");
    let sync2 = FileSync::new("docs/file.md", "repo-b", "hash2");

    repo.save_file_sync(&sync1).await.unwrap();
    repo.save_file_sync(&sync2).await.unwrap();

    let found = repo
        .find_file_sync("docs/file.md", "repo-a")
        .await
        .expect("Query failed");
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, sync1.id);

    let not_found = repo
        .find_file_sync("docs/file.md", "repo-c")
        .await
        .expect("Query failed");
    assert!(not_found.is_none());
}

#[tokio::test]
async fn test_list_file_syncs() {
    let repo = setup_repo().await;

    for i in 0..5 {
        let sync = FileSync::new(&format!("docs/file{}.md", i), "repo", &format!("hash{}", i));
        repo.save_file_sync(&sync).await.unwrap();
    }

    let syncs = repo.list_file_syncs().await.expect("Failed to list");
    assert_eq!(syncs.len(), 5);
}

#[tokio::test]
async fn test_list_file_syncs_by_status() {
    let repo = setup_repo().await;

    let sync1 = FileSync::new("docs/synced.md", "repo", "hash1");
    let mut sync2 = FileSync::new("docs/stale.md", "repo", "hash2");
    sync2.mark_stale();
    let mut sync3 = FileSync::new("docs/deleted.md", "repo", "hash3");
    sync3.mark_deleted();

    repo.save_file_sync(&sync1).await.unwrap();
    repo.save_file_sync(&sync2).await.unwrap();
    repo.save_file_sync(&sync3).await.unwrap();

    let synced = repo
        .list_file_syncs_by_status(SyncStatus::Synced)
        .await
        .unwrap();
    assert_eq!(synced.len(), 1);
    assert_eq!(synced[0].path, "docs/synced.md");

    let stale = repo
        .list_file_syncs_by_status(SyncStatus::Stale)
        .await
        .unwrap();
    assert_eq!(stale.len(), 1);

    let deleted = repo
        .list_file_syncs_by_status(SyncStatus::Deleted)
        .await
        .unwrap();
    assert_eq!(deleted.len(), 1);
}

#[tokio::test]
async fn test_list_file_syncs_for_repo() {
    let repo = setup_repo().await;

    let sync1 = FileSync::new("docs/a.md", "repo-a", "hash1");
    let sync2 = FileSync::new("docs/b.md", "repo-a", "hash2");
    let sync3 = FileSync::new("docs/c.md", "repo-b", "hash3");

    repo.save_file_sync(&sync1).await.unwrap();
    repo.save_file_sync(&sync2).await.unwrap();
    repo.save_file_sync(&sync3).await.unwrap();

    let repo_a_syncs = repo.list_file_syncs_for_repo("repo-a").await.unwrap();
    assert_eq!(repo_a_syncs.len(), 2);

    let repo_b_syncs = repo.list_file_syncs_for_repo("repo-b").await.unwrap();
    assert_eq!(repo_b_syncs.len(), 1);
}

#[tokio::test]
async fn test_find_file_syncs_by_hash() {
    let repo = setup_repo().await;

    // Create syncs with same hash (duplicate files)
    let sync1 = FileSync::new("docs/dup1.md", "repo", "samehash");
    let sync2 = FileSync::new("docs/dup2.md", "repo", "samehash");
    let sync3 = FileSync::new("docs/other.md", "repo", "differenthash");

    repo.save_file_sync(&sync1).await.unwrap();
    repo.save_file_sync(&sync2).await.unwrap();
    repo.save_file_sync(&sync3).await.unwrap();

    let duplicates = repo.find_file_syncs_by_hash("samehash").await.unwrap();
    assert_eq!(duplicates.len(), 2);
}

#[tokio::test]
async fn test_delete_file_sync() {
    let repo = setup_repo().await;

    let sync = FileSync::new("docs/to-delete.md", "repo", "hash");
    repo.save_file_sync(&sync).await.unwrap();

    repo.delete_file_sync(&sync.id)
        .await
        .expect("Failed to delete");

    let result = repo.get_file_sync(&sync.id).await;
    assert!(result.is_err());
}

// =============================================================================
// DocAlias Tests
// =============================================================================

#[tokio::test]
async fn test_save_and_find_by_alias() {
    let repo = setup_repo().await;

    let doc = KnowledgeDoc::new("Deployment Guide", DocType::Runbook, "Content");
    repo.save_doc(&doc).await.unwrap();

    let alias = DocAlias::new("deploy-howto", doc.id.clone());
    repo.save_alias(&alias).await.expect("Failed to save alias");

    let found = repo
        .find_doc_by_alias("deploy-howto")
        .await
        .expect("Query failed");
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, doc.id);
}

#[tokio::test]
async fn test_multiple_aliases() {
    let repo = setup_repo().await;

    let doc = KnowledgeDoc::new("Setup Guide", DocType::Howto, "Content");
    repo.save_doc(&doc).await.unwrap();

    let alias1 = DocAlias::new("setup", doc.id.clone());
    let alias2 = DocAlias::new("installation", doc.id.clone());
    let alias3 = DocAlias::new("getting-started", doc.id.clone());

    repo.save_alias(&alias1).await.unwrap();
    repo.save_alias(&alias2).await.unwrap();
    repo.save_alias(&alias3).await.unwrap();

    // All aliases should resolve to the same doc
    for alias_name in ["setup", "installation", "getting-started"] {
        let found = repo.find_doc_by_alias(alias_name).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, doc.id);
    }

    // List aliases for doc
    let aliases = repo.list_aliases_for_doc(&doc.id).await.unwrap();
    assert_eq!(aliases.len(), 3);
}

#[tokio::test]
async fn test_alias_not_found() {
    let repo = setup_repo().await;

    let found = repo
        .find_doc_by_alias("nonexistent-alias")
        .await
        .expect("Query failed");
    assert!(found.is_none());
}

#[tokio::test]
async fn test_delete_alias() {
    let repo = setup_repo().await;

    let doc = KnowledgeDoc::new("Test Doc", DocType::Readme, "Content");
    repo.save_doc(&doc).await.unwrap();

    let alias = DocAlias::new("test-alias", doc.id.clone());
    repo.save_alias(&alias).await.unwrap();

    // Verify exists
    assert!(repo
        .find_doc_by_alias("test-alias")
        .await
        .unwrap()
        .is_some());

    // Delete
    repo.delete_alias("test-alias").await.unwrap();

    // Verify gone
    assert!(repo
        .find_doc_by_alias("test-alias")
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn test_delete_doc_deletes_aliases() {
    let repo = setup_repo().await;

    let doc = KnowledgeDoc::new("Cascade Doc", DocType::Readme, "Content");
    repo.save_doc(&doc).await.unwrap();

    let alias = DocAlias::new("cascade-alias", doc.id.clone());
    repo.save_alias(&alias).await.unwrap();

    // Delete doc (should cascade to aliases)
    repo.delete_doc(&doc.id).await.unwrap();

    // Verify alias is gone
    let aliases = repo.list_aliases_for_doc(&doc.id).await.unwrap();
    assert!(aliases.is_empty());
}

// =============================================================================
// DocEvent Tests
// =============================================================================

#[tokio::test]
async fn test_save_and_list_events() {
    let repo = setup_repo().await;

    let doc = KnowledgeDoc::new("Event Test", DocType::Readme, "Content");
    repo.save_doc(&doc).await.unwrap();

    let event1 = DocEvent::new(doc.id.clone(), DocEventType::Created, "system");
    let event2 = DocEvent::new(doc.id.clone(), DocEventType::Reviewed, "user@example.com");

    repo.save_event(&event1)
        .await
        .expect("Failed to save event");
    repo.save_event(&event2)
        .await
        .expect("Failed to save event");

    let events = repo
        .list_events_for_doc(&doc.id)
        .await
        .expect("Failed to list");
    assert_eq!(events.len(), 2);
}

#[tokio::test]
async fn test_event_with_details() {
    let repo = setup_repo().await;

    let doc = KnowledgeDoc::new("Detail Test", DocType::Readme, "Content");
    repo.save_doc(&doc).await.unwrap();

    let event = DocEvent::new(doc.id.clone(), DocEventType::Merged, "user").with_details(
        serde_json::json!({
            "merged_from": ["doc-a", "doc-b"],
            "reason": "Consolidation"
        }),
    );

    repo.save_event(&event).await.unwrap();

    let events = repo.list_events_for_doc(&doc.id).await.unwrap();
    assert_eq!(events.len(), 1);
    assert!(events[0].details.get("merged_from").is_some());
}

// =============================================================================
// Statistics Tests
// =============================================================================

#[tokio::test]
async fn test_knowledge_stats() {
    let repo = setup_repo().await;

    // Initially empty
    let stats = repo.stats().await.expect("Failed to get stats");
    assert_eq!(stats.doc_count, 0);
    assert_eq!(stats.file_sync_count, 0);
    assert_eq!(stats.alias_count, 0);

    // Add data
    let doc = KnowledgeDoc::new("Stats Test", DocType::Readme, "Content");
    repo.save_doc(&doc).await.unwrap();

    let sync = FileSync::new("docs/stats.md", "repo", "hash");
    repo.save_file_sync(&sync).await.unwrap();

    let alias = DocAlias::new("stats-alias", doc.id.clone());
    repo.save_alias(&alias).await.unwrap();

    let stats = repo.stats().await.expect("Failed to get stats");
    assert_eq!(stats.doc_count, 1);
    assert_eq!(stats.file_sync_count, 1);
    assert_eq!(stats.alias_count, 1);
}

// =============================================================================
// Edge Cases
// =============================================================================

#[tokio::test]
async fn test_special_characters_in_content() {
    let repo = setup_repo().await;

    let content = r#"
# Special Characters Test

Code block:
```rust
fn main() {
    println!("Hello, world!");
}
```

Unicode: 你好世界 🎉

Quotes: "double" and 'single'

Backslashes: C:\Users\test
"#;

    let doc = KnowledgeDoc::new("Special Chars", DocType::Readme, content);
    repo.save_doc(&doc)
        .await
        .expect("Failed with special chars");

    let retrieved = repo.get_doc(&doc.id).await.unwrap();
    assert!(retrieved.content.contains("fn main()"));
    assert!(retrieved.content.contains("你好世界"));
}

#[tokio::test]
async fn test_long_content() {
    let repo = setup_repo().await;

    let long_content = "A".repeat(100000);
    let doc = KnowledgeDoc::new("Long Content", DocType::Readme, &long_content);

    repo.save_doc(&doc)
        .await
        .expect("Should handle long content");

    let retrieved = repo.get_doc(&doc.id).await.unwrap();
    assert_eq!(retrieved.content.len(), 100000);
}

#[tokio::test]
async fn test_update_doc() {
    let repo = setup_repo().await;

    let mut doc = KnowledgeDoc::new("Mutable Doc", DocType::Readme, "Initial content");
    repo.save_doc(&doc).await.unwrap();

    // Update content
    doc.update_content("Updated content".to_string());

    // Save updated doc
    repo.save_doc(&doc).await.unwrap();

    let retrieved = repo.get_doc(&doc.id).await.unwrap();
    assert_eq!(retrieved.content, "Updated content");
}

#[tokio::test]
async fn test_doc_status_changes() {
    let repo = setup_repo().await;

    let doc = KnowledgeDoc::new("Status Test", DocType::Design, "Content");
    assert_eq!(doc.status, DocStatus::Active);

    repo.save_doc(&doc).await.unwrap();

    // This would typically be done through a service, but testing the storage layer
    // Can save docs with different statuses
    let draft_doc = KnowledgeDoc::new("Draft Doc", DocType::Readme, "WIP");
    // Note: We'd need to modify the doc to set status to Draft for a full test
    repo.save_doc(&draft_doc).await.unwrap();
}

#[tokio::test]
async fn test_many_tags() {
    let repo = setup_repo().await;

    let doc = KnowledgeDoc::new("Tagged Doc", DocType::Readme, "Content").with_tags(vec![
        "rust".to_string(),
        "api".to_string(),
        "v2".to_string(),
        "production".to_string(),
        "guide".to_string(),
    ]);

    repo.save_doc(&doc).await.unwrap();

    let retrieved = repo.get_doc(&doc.id).await.unwrap();
    assert_eq!(retrieved.tags.len(), 5);
    assert!(retrieved.tags.contains(&"rust".to_string()));
}

// =============================================================================
// MCP Tool Handler Tests (Phase 4: Consolidated Knowledge Tool)
// =============================================================================

async fn setup_tool_state() -> (ToolState, TempDir) {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config).await.expect("Failed to connect");

    // Create a temp directory for the knowledge repo
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let knowledge_config = KnowledgeConfig {
        knowledge_repo_path: temp_dir.path().to_path_buf(),
        auto_init_git: false, // Don't init git in tests
        extensions: vec!["md".to_string()],
        recursive: true,
    };

    let knowledge_service = KnowledgeService::new(db, knowledge_config);

    let state = ToolState::new();
    *state.knowledge_service.write().await = Some(knowledge_service);
    (state, temp_dir)
}

#[tokio::test]
async fn test_mcp_knowledge_init() {
    let (state, _temp_dir) = setup_tool_state().await;

    let req = KnowledgeRequestNew {
        action: "init".to_string(),
        path: None,
        repo_name: None,
        name: None,
        doc_type: None,
    };
    let result = tools::knowledge_new(&state, req).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("success"));
    assert!(response.contains("true"));
}

#[tokio::test]
async fn test_mcp_knowledge_list_empty() {
    let (state, _temp_dir) = setup_tool_state().await;

    // Init first
    let req = KnowledgeRequestNew {
        action: "init".to_string(),
        path: None,
        repo_name: None,
        name: None,
        doc_type: None,
    };
    tools::knowledge_new(&state, req).await.unwrap();

    // List - should be empty
    let req = KnowledgeRequestNew {
        action: "list".to_string(),
        path: None,
        repo_name: None,
        name: None,
        doc_type: None,
    };
    let result = tools::knowledge_new(&state, req).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("\"count\": 0"));
}

#[tokio::test]
async fn test_mcp_knowledge_scan() {
    let (state, temp_dir) = setup_tool_state().await;

    // Init first
    let req = KnowledgeRequestNew {
        action: "init".to_string(),
        path: None,
        repo_name: None,
        name: None,
        doc_type: None,
    };
    tools::knowledge_new(&state, req).await.unwrap();

    // Create a test markdown file in the temp directory
    let test_file = temp_dir.path().join("test-doc.md");
    std::fs::write(&test_file, "# Test Document\n\nTest content.").unwrap();

    // Scan the directory
    let req = KnowledgeRequestNew {
        action: "scan".to_string(),
        path: Some(temp_dir.path().to_string_lossy().to_string()),
        repo_name: Some("test-repo".to_string()),
        name: None,
        doc_type: None,
    };
    let result = tools::knowledge_new(&state, req).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("files_found"));
}

#[tokio::test]
async fn test_mcp_knowledge_register() {
    let (state, temp_dir) = setup_tool_state().await;

    // Init first
    let req = KnowledgeRequestNew {
        action: "init".to_string(),
        path: None,
        repo_name: None,
        name: None,
        doc_type: None,
    };
    tools::knowledge_new(&state, req).await.unwrap();

    // Create a test markdown file
    let test_file = temp_dir.path().join("api-guide.md");
    std::fs::write(&test_file, "# API Guide\n\nHow to use the API.").unwrap();

    // Register the document
    let req = KnowledgeRequestNew {
        action: "register".to_string(),
        path: Some(test_file.to_string_lossy().to_string()),
        repo_name: None,
        name: Some("API Guide".to_string()),
        doc_type: Some("howto".to_string()),
    };
    let result = tools::knowledge_new(&state, req).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("API Guide"));
    assert!(response.contains("howto"));
}

#[tokio::test]
async fn test_mcp_knowledge_import() {
    let (state, temp_dir) = setup_tool_state().await;

    // Init first
    let req = KnowledgeRequestNew {
        action: "init".to_string(),
        path: None,
        repo_name: None,
        name: None,
        doc_type: None,
    };
    tools::knowledge_new(&state, req).await.unwrap();

    // Create a test markdown file in a different location
    let source_dir = TempDir::new().unwrap();
    let source_file = source_dir.path().join("source-doc.md");
    std::fs::write(&source_file, "# Source Document\n\nContent to import.").unwrap();

    // Import the document
    let req = KnowledgeRequestNew {
        action: "import".to_string(),
        path: Some(source_file.to_string_lossy().to_string()),
        repo_name: None,
        name: Some("Imported Doc".to_string()),
        doc_type: Some("readme".to_string()),
    };
    let result = tools::knowledge_new(&state, req).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("Imported Doc"));
    assert!(response.contains("readme"));
}

#[tokio::test]
async fn test_mcp_knowledge_duplicates() {
    let (state, _temp_dir) = setup_tool_state().await;

    // Init first
    let req = KnowledgeRequestNew {
        action: "init".to_string(),
        path: None,
        repo_name: None,
        name: None,
        doc_type: None,
    };
    tools::knowledge_new(&state, req).await.unwrap();

    // Check duplicates (should be empty)
    let req = KnowledgeRequestNew {
        action: "duplicates".to_string(),
        path: None,
        repo_name: None,
        name: None,
        doc_type: None,
    };
    let result = tools::knowledge_new(&state, req).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("\"count\": 0"));
    assert!(response.contains("groups"));
}

#[tokio::test]
async fn test_mcp_knowledge_versions() {
    let (state, _temp_dir) = setup_tool_state().await;

    // Init first
    let req = KnowledgeRequestNew {
        action: "init".to_string(),
        path: None,
        repo_name: None,
        name: None,
        doc_type: None,
    };
    tools::knowledge_new(&state, req).await.unwrap();

    // Check version chains (should be empty)
    let req = KnowledgeRequestNew {
        action: "versions".to_string(),
        path: None,
        repo_name: None,
        name: None,
        doc_type: None,
    };
    let result = tools::knowledge_new(&state, req).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("\"count\": 0"));
    assert!(response.contains("chains"));
}

#[tokio::test]
async fn test_mcp_knowledge_invalid_action() {
    let (state, _temp_dir) = setup_tool_state().await;

    let req = KnowledgeRequestNew {
        action: "invalid_action".to_string(),
        path: None,
        repo_name: None,
        name: None,
        doc_type: None,
    };
    let result = tools::knowledge_new(&state, req).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown action"));
}

#[tokio::test]
async fn test_mcp_knowledge_invalid_doc_type() {
    let (state, temp_dir) = setup_tool_state().await;

    // Init first
    let req = KnowledgeRequestNew {
        action: "init".to_string(),
        path: None,
        repo_name: None,
        name: None,
        doc_type: None,
    };
    tools::knowledge_new(&state, req).await.unwrap();

    // Create a test markdown file
    let test_file = temp_dir.path().join("test.md");
    std::fs::write(&test_file, "# Test").unwrap();

    // Try to register with invalid doc type
    let req = KnowledgeRequestNew {
        action: "register".to_string(),
        path: Some(test_file.to_string_lossy().to_string()),
        repo_name: None,
        name: Some("Test Doc".to_string()),
        doc_type: Some("invalid_type".to_string()),
    };
    let result = tools::knowledge_new(&state, req).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown doc_type"));
}

#[tokio::test]
async fn test_mcp_knowledge_register_missing_params() {
    let (state, _temp_dir) = setup_tool_state().await;

    // Init first
    let req = KnowledgeRequestNew {
        action: "init".to_string(),
        path: None,
        repo_name: None,
        name: None,
        doc_type: None,
    };
    tools::knowledge_new(&state, req).await.unwrap();

    // Missing path
    let req = KnowledgeRequestNew {
        action: "register".to_string(),
        path: None,
        repo_name: None,
        name: Some("Test".to_string()),
        doc_type: Some("readme".to_string()),
    };
    let result = tools::knowledge_new(&state, req).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("path required"));

    // Missing name
    let req = KnowledgeRequestNew {
        action: "register".to_string(),
        path: Some("/tmp/test.md".to_string()),
        repo_name: None,
        name: None,
        doc_type: Some("readme".to_string()),
    };
    let result = tools::knowledge_new(&state, req).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("name required"));

    // Missing doc_type
    let req = KnowledgeRequestNew {
        action: "register".to_string(),
        path: Some("/tmp/test.md".to_string()),
        repo_name: None,
        name: Some("Test".to_string()),
        doc_type: None,
    };
    let result = tools::knowledge_new(&state, req).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("doc_type required"));
}
