//! Integration tests for Memory OS MCP tooling.

use engram_index::{EntityService, MemoryService};
use engram_mcp::tools::{
    self, EntityObserveRequestNew, EntityRequestNew, MemoryChangeRequest, MemoryEvidenceRequest,
    MemoryRequestNew, OrientRequest, ToolState, VaultRequest,
};
use engram_store::{connect_and_init, StoreConfig};
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

async fn setup_tool_state() -> ToolState {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config).await.expect("Failed to connect");
    let memory_service = MemoryService::new(db.clone());
    memory_service
        .init_schema()
        .await
        .expect("Failed to initialize memory schema");
    let entity_service = EntityService::new(db);
    entity_service
        .init()
        .await
        .expect("Failed to initialize entity schema");

    let state = ToolState::new();
    state.init_memory(memory_service).await;
    state.init_entity(entity_service).await;
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
        source_entity_name: None,
        observation_key: None,
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
        relevance_project: None,
        cwd: None,
        query: None,
        intent: None,
        archive_reason: None,
        archived_by: None,
        reviewer: None,
        rationale: None,
        supersedes_id: None,
        vault_path: None,
        migration_review_path: None,
        exclude_reviewed_path: None,
        digest_extraction_path: None,
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

fn vault_request(action: &str, path: &str) -> VaultRequest {
    VaultRequest {
        action: action.to_string(),
        vault_path: Some(path.to_string()),
        page: None,
    }
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
async fn test_mcp_memory_promote_review_candidate() {
    let state = setup_tool_state().await;

    let mut add = with_writer(request("add"));
    add.kind = Some("decision".to_string());
    add.title = Some("Candidate decision".to_string());
    add.content = Some("Candidate guidance should become active only after review.".to_string());
    add.origin = Some("agent_inferred".to_string());
    add.scope_type = Some("project".to_string());
    add.project_name = Some("engram".to_string());
    let add_response = tools::memory_new(&state, add)
        .await
        .expect("add should work");
    let add_json = parse_json(&add_response);
    let id = add_json["item"]["id"].as_str().unwrap().to_string();
    assert_eq!(add_json["item"]["status"], "needs_review");

    let mut promote = request("promote");
    promote.id = Some(id.clone());
    promote.reviewer = Some("yuval".to_string());
    promote.rationale = Some("Reviewed and accepted.".to_string());
    let promote_response = tools::memory_new(&state, promote)
        .await
        .expect("promote should work");
    let promote_json = parse_json(&promote_response);

    assert_eq!(promote_json["item"]["id"], id);
    assert_eq!(promote_json["item"]["status"], "active");
    assert_eq!(promote_json["item"]["evidence"][0]["kind"], "manual_review");
    assert_eq!(promote_json["item"]["evidence"][0]["target"], "yuval");
    assert_eq!(
        promote_json["item"]["evidence"][0]["summary"],
        "Reviewed and accepted."
    );
}

#[tokio::test]
async fn test_mcp_memory_promote_observation_surfaces_in_orient() {
    let state = setup_tool_state().await;

    tools::entity_new(
        &state,
        EntityRequestNew {
            action: "create".to_string(),
            name: Some("engram".to_string()),
            entity_type: Some("repo".to_string()),
            description: Some("Engram repository".to_string()),
            query: None,
            type_filter: None,
            limit: None,
            target: None,
            relation: None,
            alias: None,
        },
    )
    .await
    .expect("entity create should work");

    tools::entity_observe_new(
        &state,
        EntityObserveRequestNew {
            action: "add".to_string(),
            entity: Some("engram".to_string()),
            content: Some(
                "Observation promotion should create reviewed MemoryItems with source evidence."
                    .to_string(),
            ),
            key: Some("decisions.observation-promotion".to_string()),
            source: Some("memory-tests".to_string()),
            key_pattern: None,
            query: None,
            limit: None,
        },
    )
    .await
    .expect("observation add should work");

    let mut promote = with_writer(request("promote_observation"));
    promote.source_entity_name = Some("engram".to_string());
    promote.observation_key = Some("decisions.observation-promotion".to_string());
    promote.kind = Some("decision".to_string());
    promote.title = Some("Observation promotion feeds Brain Loop".to_string());
    promote.origin = Some("agent_observed".to_string());
    promote.scope_type = Some("project".to_string());
    promote.project_name = Some("engram".to_string());
    promote.reviewer = Some("yuval".to_string());
    promote.rationale = Some("Reviewed as durable project guidance.".to_string());
    promote.tags = vec!["brain-loop".to_string(), "promotion".to_string()];

    let promote_response = tools::memory_new(&state, promote)
        .await
        .expect("promote_observation should work");
    let promote_json = parse_json(&promote_response);
    let source_id = promote_json["source_observation"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    assert_eq!(promote_json["item"]["status"], "active");
    assert_eq!(
        promote_json["item"]["content"],
        "Observation promotion should create reviewed MemoryItems with source evidence."
    );
    let evidence = promote_json["item"]["evidence"].as_array().unwrap();
    assert!(evidence.iter().any(|e| {
        e["kind"] == "observation" && e["target"].as_str() == Some(source_id.as_str())
    }));
    assert!(evidence
        .iter()
        .any(|e| e["kind"] == "manual_review" && e["target"] == "yuval"));

    let err = tools::memory_new(&state, {
        let mut duplicate = with_writer(request("promote_observation"));
        duplicate.source_entity_name = Some("engram".to_string());
        duplicate.observation_key = Some("decisions.observation-promotion".to_string());
        duplicate.kind = Some("decision".to_string());
        duplicate.title = Some("Duplicate promotion".to_string());
        duplicate.origin = Some("agent_observed".to_string());
        duplicate.scope_type = Some("project".to_string());
        duplicate.project_name = Some("engram".to_string());
        duplicate.reviewer = Some("yuval".to_string());
        duplicate.rationale = Some("Already accepted.".to_string());
        duplicate
    })
    .await
    .unwrap_err();
    assert!(err.contains("already promoted"));

    let response = tools::orient(
        &state,
        OrientRequest {
            cwd: Some("/Users/yuval.meiri/projects/engram".to_string()),
            prompt: Some("continue Brain Loop promotion work".to_string()),
            project: Some("engram".to_string()),
            agent: Some("codex".to_string()),
            intent: None,
            include_recent_commits: Some(false),
            limit: Some(10),
        },
    )
    .await
    .expect("orient should work");
    let orient_json = parse_json(&response);
    assert_eq!(
        orient_json["brain_loop"]["top_items"][0]["title"],
        "Observation promotion feeds Brain Loop"
    );
    assert_eq!(
        orient_json["active_decisions"][0]["evidence"][0]["target"]
            .as_str()
            .unwrap(),
        source_id
    );
}

#[tokio::test]
async fn test_mcp_memory_reject_review_candidate() {
    let state = setup_tool_state().await;

    let mut add = with_writer(request("add"));
    add.kind = Some("rule".to_string());
    add.title = Some("Bad candidate".to_string());
    add.content = Some("This inferred rule should be rejected during review.".to_string());
    add.origin = Some("agent_inferred".to_string());
    add.scope_type = Some("project".to_string());
    add.project_name = Some("engram".to_string());
    let add_response = tools::memory_new(&state, add)
        .await
        .expect("add should work");
    let id = parse_json(&add_response)["item"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let mut reject = request("reject");
    reject.id = Some(id.clone());
    reject.reviewer = Some("agent-reviewer".to_string());
    reject.rationale = Some("Contradicted by current project evidence.".to_string());
    let reject_response = tools::memory_new(&state, reject)
        .await
        .expect("reject should work");
    let reject_json = parse_json(&reject_response);

    assert_eq!(reject_json["item"]["id"], id);
    assert_eq!(reject_json["item"]["status"], "rejected");
    assert_eq!(
        reject_json["item"]["evidence"][0]["target"],
        "agent-reviewer"
    );
    assert_eq!(
        reject_json["item"]["evidence"][0]["summary"],
        "Contradicted by current project evidence."
    );

    let review_response = tools::memory_new(&state, request("review"))
        .await
        .expect("review should work");
    let review_json = parse_json(&review_response);
    assert_eq!(review_json["count"], 0);
}

#[tokio::test]
async fn test_mcp_memory_supersede_replaces_active_item() {
    let state = setup_tool_state().await;

    let mut old = with_writer(request("add"));
    old.kind = Some("decision".to_string());
    old.title = Some("Old decision".to_string());
    old.content = Some("The old workflow should be replaced.".to_string());
    old.origin = Some("user_stated".to_string());
    old.scope_type = Some("project".to_string());
    old.project_name = Some("engram".to_string());
    old.evidence = vec![MemoryEvidenceRequest {
        kind: "manual_review".to_string(),
        target: "memory_tests".to_string(),
        summary: Some("Old decision starts as active guidance.".to_string()),
        excerpt: None,
    }];
    let old_response = tools::memory_new(&state, old)
        .await
        .expect("old add should work");
    let old_id = parse_json(&old_response)["item"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let mut new = with_writer(request("add"));
    new.kind = Some("decision".to_string());
    new.title = Some("Replacement decision".to_string());
    new.content = Some("The replacement workflow should guide future agents.".to_string());
    new.origin = Some("agent_inferred".to_string());
    new.scope_type = Some("project".to_string());
    new.project_name = Some("engram".to_string());
    let new_response = tools::memory_new(&state, new)
        .await
        .expect("new add should work");
    let new_json = parse_json(&new_response);
    let new_id = new_json["item"]["id"].as_str().unwrap().to_string();
    assert_eq!(new_json["item"]["status"], "needs_review");

    let mut supersede = request("supersede");
    supersede.id = Some(new_id.clone());
    supersede.supersedes_id = Some(old_id.clone());
    supersede.reviewer = Some("yuval".to_string());
    supersede.rationale = Some("Replacement reflects current evidence.".to_string());
    let supersede_response = tools::memory_new(&state, supersede)
        .await
        .expect("supersede should work");
    let supersede_json = parse_json(&supersede_response);

    assert_eq!(supersede_json["item"]["id"], new_id);
    assert_eq!(supersede_json["item"]["status"], "active");
    assert_eq!(supersede_json["item"]["supersedes"][0], old_id);
    assert_eq!(supersede_json["superseded_item"]["id"], old_id);
    assert_eq!(supersede_json["superseded_item"]["status"], "superseded");

    let mut list = request("list");
    list.status_filter = Some("active".to_string());
    let list_response = tools::memory_new(&state, list)
        .await
        .expect("list should work");
    let list_json = parse_json(&list_response);
    assert_eq!(list_json["count"], 1);
    assert_eq!(list_json["items"][0]["id"], new_id);
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
    add.evidence = vec![MemoryEvidenceRequest {
        kind: "manual_review".to_string(),
        target: "memory_tests".to_string(),
        summary: Some("Vault export test expects active durable guidance.".to_string()),
        excerpt: None,
    }];

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
async fn test_mcp_vault_init_compile_status_page() {
    let state = setup_tool_state().await;

    let mut add = with_writer(request("add"));
    add.kind = Some("decision".to_string());
    add.title = Some("Use dedicated vault tool".to_string());
    add.content = Some("Vault operations should have their own MCP surface.".to_string());
    add.origin = Some("user_stated".to_string());
    add.scope_type = Some("project".to_string());
    add.project_name = Some("engram".to_string());
    add.evidence = vec![MemoryEvidenceRequest {
        kind: "manual_review".to_string(),
        target: "memory_tests".to_string(),
        summary: Some("Vault compile test expects active durable guidance.".to_string()),
        excerpt: None,
    }];
    tools::memory_new(&state, add)
        .await
        .expect("add should work");

    let dir = tempdir().expect("tempdir should be created");
    let root = dir.path().display().to_string();

    let init_response = tools::vault_new(&state, vault_request("init", &root))
        .await
        .expect("vault init should work");
    let init_json = parse_json(&init_response);
    assert_eq!(init_json["init"]["root"], root);
    assert!(dir.path().join("memory/items").is_dir());

    let compile_response = tools::vault_new(&state, vault_request("compile", &root))
        .await
        .expect("vault compile should work");
    let compile_json = parse_json(&compile_response);
    assert_eq!(compile_json["export"]["memory_item_count"], 1);

    let status_response = tools::vault_new(&state, vault_request("status", &root))
        .await
        .expect("vault status should work");
    let status_json = parse_json(&status_response);
    assert_eq!(status_json["status"]["initialized"], true);
    assert_eq!(
        status_json["status"]["generated_file_count"],
        status_json["status"]["expected_generated_file_count"]
    );

    let mut page = vault_request("page", &root);
    page.page = Some("memory/index".to_string());
    let page_response = tools::vault_new(&state, page)
        .await
        .expect("vault page should work");
    let page_json = parse_json(&page_response);
    assert_eq!(page_json["found"], true);
    assert_eq!(page_json["page"]["relative_path"], "memory/index.md");
    assert!(page_json["page"]["contents"]
        .as_str()
        .unwrap()
        .contains("Use dedicated vault tool"));
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
async fn test_mcp_memory_migration_review_status_empty_batch() {
    let state = setup_tool_state().await;
    let dir = tempdir().expect("tempdir should be created");

    let mut status = request("migration_review_status");
    status.migration_review_path = Some(dir.path().display().to_string());

    let response = tools::memory_new(&state, status)
        .await
        .expect("migration_review_status should work");
    let json = parse_json(&response);

    assert_eq!(json["status"]["files_scanned"], 0);
    assert_eq!(json["status"]["planned_count"], 0);
    assert_eq!(json["status"]["ready_to_apply"], true);
}

#[tokio::test]
async fn test_mcp_memory_digest_extraction_apply_empty_batch_dry_run() {
    let state = setup_tool_state().await;
    let dir = tempdir().expect("tempdir should be created");

    let mut apply = with_writer(request("digest_extraction_apply"));
    apply.digest_extraction_path = Some(dir.path().display().to_string());
    apply.dry_run = Some(true);

    let response = tools::memory_new(&state, apply)
        .await
        .expect("digest_extraction_apply should work");
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
    add.evidence = vec![MemoryEvidenceRequest {
        kind: "manual_review".to_string(),
        target: "memory_tests".to_string(),
        summary: Some("Orient test expects active durable guidance.".to_string()),
        excerpt: None,
    }];
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
            intent: None,
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
    assert_eq!(
        json["memory_metadata"][0]["memory_id"],
        json["active_decisions"][0]["id"]
    );
    assert_eq!(json["memory_metadata"][0]["status"], "active");
    assert_eq!(json["memory_metadata"][0]["review_state"], "reviewed");
    assert_eq!(json["memory_metadata"][0]["freshness"], "unscheduled");
    assert_eq!(json["memory_metadata"][0]["claim_origin"], "user_stated");
    assert_eq!(json["memory_metadata"][0]["writer"]["harness"], "codex");
    assert!(json["context_pack"]
        .as_str()
        .unwrap()
        .contains("Memory cursor timestamp"));
    assert!(json["context_pack"]
        .as_str()
        .unwrap()
        .contains("Trust: status=active, review_state=reviewed"));
    assert_eq!(json["brain_loop"]["degraded"], false);
    assert!(json["brain_loop"]["compiled_context"]
        .as_str()
        .unwrap()
        .contains("Orient through memory"));
    assert_eq!(
        json["brain_loop"]["top_items"][0]["trust"]["memory_id"],
        json["active_decisions"][0]["id"]
    );
    assert!(json["memory_cursor"]["timestamp"].is_string());
}
