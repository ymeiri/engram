//! Integration tests for Memory OS MCP tooling.

use engram_index::MemoryService;
use engram_mcp::tools::{
    self, MemoryChangeRequest, MemoryEvidenceRequest, MemoryRequestNew, OrientRequest, ToolState,
};
use engram_store::{connect_and_init, StoreConfig};
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

async fn setup_tool_state() -> ToolState {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config).await.expect("Failed to connect");
    let service = MemoryService::new(db);
    service
        .init_schema()
        .await
        .expect("Failed to initialize memory schema");

    let state = ToolState::new();
    state.init_memory(service).await;
    state
}

fn request(action: &str) -> MemoryRequestNew {
    MemoryRequestNew {
        action: action.to_string(),
        id: None,
        kind: None,
        title: None,
        content: None,
        origin: None,
        status: None,
        confidence: None,
        tags: Vec::new(),
        scope_type: None,
        project_id: None,
        project_name: None,
        task_id: None,
        task_name: None,
        entity_id: None,
        entity_name: None,
        repository_id: None,
        remote_url: None,
        local_path: None,
        scope_session_id: None,
        scope_name: None,
        writer_harness: None,
        writer_harness_version: None,
        model_provider: None,
        model: None,
        model_version: None,
        surface: None,
        actor: None,
        writer_session_id: None,
        evidence: Vec::new(),
        status_filter: None,
        limit: None,
        message: None,
        parent_id: None,
        session_id: None,
        changes: Vec::new(),
        commit_id: None,
        timestamp: None,
        vault_path: None,
        migration_review_path: None,
        dry_run: None,
        create_commit: None,
        include_entity_observations: None,
        include_session_history: None,
        include_work_observations: None,
    }
}

fn with_writer(mut req: MemoryRequestNew) -> MemoryRequestNew {
    req.writer_harness = Some("codex".to_string());
    req.model_provider = Some("openai".to_string());
    req.model = Some("gpt-5.5".to_string());
    req.surface = Some("desktop".to_string());
    req
}

fn parse_json(response: &str) -> Value {
    serde_json::from_str(response).expect("response should be valid JSON")
}

#[tokio::test]
async fn test_mcp_memory_add_get_list() {
    let state = setup_tool_state().await;

    let mut add = with_writer(request("add"));
    add.kind = Some("decision".to_string());
    add.title = Some("Expose Memory MCP".to_string());
    add.content = Some("The first MCP surface should use MemoryService.".to_string());
    add.origin = Some("user_stated".to_string());
    add.scope_type = Some("project".to_string());
    add.project_name = Some("engram".to_string());
    add.tags = vec!["memory-os".to_string(), "mcp".to_string()];
    add.evidence = vec![MemoryEvidenceRequest {
        kind: "manual_review".to_string(),
        target: "memory_tests".to_string(),
        summary: Some("MCP add/get/list test".to_string()),
        excerpt: None,
    }];

    let add_response = tools::memory_new(&state, add)
        .await
        .expect("add should work");
    let add_json = parse_json(&add_response);
    let id = add_json["item"]["id"].as_str().unwrap().to_string();
    assert_eq!(add_json["item"]["writer"]["harness"], "codex");
    assert_eq!(add_json["item"]["status"], "active");

    let mut get = request("get");
    get.id = Some(id.clone());
    let get_response = tools::memory_new(&state, get)
        .await
        .expect("get should work");
    let get_json = parse_json(&get_response);
    assert_eq!(get_json["item"]["id"], id);
    assert_eq!(get_json["item"]["title"], "Expose Memory MCP");

    let mut list = request("list");
    list.status_filter = Some("active".to_string());
    let list_response = tools::memory_new(&state, list)
        .await
        .expect("list should work");
    let list_json = parse_json(&list_response);
    assert_eq!(list_json["count"], 1);
    assert_eq!(list_json["items"][0]["id"], id);
}

#[tokio::test]
async fn test_mcp_memory_add_requires_writer_provenance() {
    let state = setup_tool_state().await;

    let mut add = request("add");
    add.kind = Some("decision".to_string());
    add.title = Some("Missing writer".to_string());
    add.content = Some("This should fail.".to_string());
    add.origin = Some("user_stated".to_string());
    add.scope_type = Some("project".to_string());
    add.project_name = Some("engram".to_string());

    let err = tools::memory_new(&state, add).await.unwrap_err();
    assert!(err.contains("writer_harness required for add"));
}

#[tokio::test]
async fn test_mcp_memory_review_lists_inferred_items() {
    let state = setup_tool_state().await;

    let mut add = with_writer(request("add"));
    add.kind = Some("preference".to_string());
    add.title = Some("Possible status update preference".to_string());
    add.content = Some("User may prefer concise implementation updates.".to_string());
    add.origin = Some("agent_inferred".to_string());
    add.scope_type = Some("user".to_string());

    tools::memory_new(&state, add)
        .await
        .expect("add should work");

    let review_response = tools::memory_new(&state, request("review"))
        .await
        .expect("review should work");
    let review_json = parse_json(&review_response);

    assert_eq!(review_json["count"], 1);
    assert_eq!(review_json["items"][0]["status"], "needs_review");
    assert_eq!(
        review_json["items"][0]["title"],
        "Possible status update preference"
    );
}

#[tokio::test]
async fn test_mcp_memory_commit_and_changes_since() {
    let state = setup_tool_state().await;

    let cursor_response = tools::memory_new(&state, request("cursor"))
        .await
        .expect("cursor should work");
    let cursor_json = parse_json(&cursor_response);
    let timestamp = cursor_json["cursor"]["timestamp"]
        .as_str()
        .unwrap()
        .to_string();

    let mut add = with_writer(request("add"));
    add.kind = Some("decision".to_string());
    add.title = Some("Track memory diffs".to_string());
    add.content = Some("Use knowledge commits to summarize memory changes.".to_string());
    add.origin = Some("user_stated".to_string());
    add.scope_type = Some("project".to_string());
    add.project_name = Some("engram".to_string());
    let add_response = tools::memory_new(&state, add)
        .await
        .expect("add should work");
    let item_id = parse_json(&add_response)["item"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let mut commit = with_writer(request("commit"));
    commit.message = Some("Capture memory diff".to_string());
    commit.changes = vec![MemoryChangeRequest {
        change_type: "added".to_string(),
        item_id: Some(item_id.clone()),
        title: "Track memory diffs".to_string(),
        summary: "Added memory item for knowledge commits.".to_string(),
        before_hash: None,
        after_hash: None,
    }];
    let commit_response = tools::memory_new(&state, commit)
        .await
        .expect("commit should work");
    let commit_json = parse_json(&commit_response);
    let commit_id = commit_json["commit"]["id"].as_str().unwrap().to_string();
    assert_eq!(commit_json["commit"]["changes"][0]["item_id"], item_id);

    let mut changes = request("changes_since");
    changes.timestamp = Some(timestamp);
    let changes_response = tools::memory_new(&state, changes)
        .await
        .expect("changes_since should work");
    let changes_json = parse_json(&changes_response);

    assert_eq!(changes_json["item_count"], 1);
    assert_eq!(changes_json["commit_count"], 1);
    assert_eq!(changes_json["commits"][0]["id"], commit_id);
    assert_eq!(changes_json["next_cursor"]["commit_id"], commit_id);
}

#[tokio::test]
async fn test_mcp_memory_export_vault() {
    let state = setup_tool_state().await;

    let mut add = with_writer(request("add"));
    add.kind = Some("decision".to_string());
    add.title = Some("Export memory vault".to_string());
    add.content = Some("Memory OS records should be projected to Markdown.".to_string());
    add.origin = Some("user_stated".to_string());
    add.scope_type = Some("project".to_string());
    add.project_name = Some("engram".to_string());

    tools::memory_new(&state, add)
        .await
        .expect("add should work");

    let dir = tempdir().expect("tempdir should be created");
    let mut export = request("export_vault");
    export.vault_path = Some(dir.path().display().to_string());

    let export_response = tools::memory_new(&state, export)
        .await
        .expect("export_vault should work");
    let export_json = parse_json(&export_response);

    assert_eq!(export_json["export"]["memory_item_count"], 1);
    assert!(export_json["export"]["files_written"]
        .as_array()
        .unwrap()
        .iter()
        .any(|path| path.as_str().unwrap().starts_with("memory/items/")));

    let index = fs::read_to_string(dir.path().join("memory/index.md"))
        .expect("memory index should be written");
    assert!(index.contains("generated_by: \"engram-memory-os\""));
    assert!(index.contains("Export memory vault"));
}

#[tokio::test]
async fn test_mcp_memory_migration_inventory_empty_store() {
    let state = setup_tool_state().await;

    let mut inventory = request("migration_inventory");
    inventory.limit = Some(10);

    let response = tools::memory_new(&state, inventory)
        .await
        .expect("migration_inventory should work");
    let json = parse_json(&response);

    assert_eq!(json["inventory"]["sources_scanned"], 0);
    assert_eq!(json["inventory"]["total_candidates"], 0);
    assert!(json["inventory"]["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|warning| warning.as_str().unwrap().contains("Dry run only")));
}

#[tokio::test]
async fn test_mcp_memory_migration_review_export_empty_store() {
    let state = setup_tool_state().await;
    let dir = tempdir().expect("tempdir should be created");

    let mut export = request("migration_review_export");
    export.migration_review_path = Some(dir.path().display().to_string());
    export.limit = Some(10);

    let response = tools::memory_new(&state, export)
        .await
        .expect("migration_review_export should work");
    let json = parse_json(&response);

    assert_eq!(json["export"]["inventory"]["sources_scanned"], 0);
    assert_eq!(json["export"]["files_written"].as_array().unwrap().len(), 1);

    let index =
        fs::read_to_string(dir.path().join("index.md")).expect("review index should be written");
    assert!(index.contains("Migration Review Batch"));
    assert!(index.contains("No migration candidates in this batch."));
}

#[tokio::test]
async fn test_mcp_memory_migration_review_apply_empty_batch_dry_run() {
    let state = setup_tool_state().await;
    let dir = tempdir().expect("tempdir should be created");

    let mut apply = with_writer(request("migration_review_apply"));
    apply.migration_review_path = Some(dir.path().display().to_string());
    apply.dry_run = Some(true);

    let response = tools::memory_new(&state, apply)
        .await
        .expect("migration_review_apply should work");
    let json = parse_json(&response);

    assert_eq!(json["apply"]["dry_run"], true);
    assert_eq!(json["apply"]["files_scanned"], 0);
    assert_eq!(json["apply"]["planned_items"].as_array().unwrap().len(), 0);
    assert_eq!(json["apply"]["written_items"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_mcp_memory_invalid_action() {
    let state = setup_tool_state().await;

    let err = tools::memory_new(&state, request("unknown"))
        .await
        .unwrap_err();

    assert!(err.contains("Unknown action"));
}

#[tokio::test]
async fn test_mcp_orient_returns_context_packet() {
    let state = setup_tool_state().await;

    let mut add = with_writer(request("add"));
    add.kind = Some("decision".to_string());
    add.title = Some("Orient through memory".to_string());
    add.content = Some("Agents should request orientation before substantial work.".to_string());
    add.origin = Some("user_stated".to_string());
    add.scope_type = Some("project".to_string());
    add.project_name = Some("engram".to_string());
    tools::memory_new(&state, add)
        .await
        .expect("add should work");

    let response = tools::orient(
        &state,
        OrientRequest {
            cwd: Some("/Users/yuval.meiri/projects/engram".to_string()),
            prompt: Some("continue implementation".to_string()),
            project: Some("engram".to_string()),
            agent: Some("codex".to_string()),
            include_recent_commits: Some(true),
            limit: Some(10),
        },
    )
    .await
    .expect("orient should work");

    let json = parse_json(&response);
    assert_eq!(json["project"], "engram");
    assert_eq!(
        json["active_decisions"][0]["title"],
        "Orient through memory"
    );
    assert!(json["context_pack"]
        .as_str()
        .unwrap()
        .contains("Memory cursor timestamp"));
    assert!(json["memory_cursor"]["timestamp"].is_string());
}
