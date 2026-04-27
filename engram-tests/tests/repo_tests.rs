//! Integration tests for repository topology MCP tooling.

use engram_index::{MemoryService, RepositoryService};
use engram_mcp::tools::{self, OrientRequest, RepoRequest, ToolState};
use engram_store::{connect_and_init, StoreConfig};
use serde_json::Value;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

async fn setup_tool_state() -> ToolState {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config).await.expect("Failed to connect");

    let repository_service = RepositoryService::new(db.clone());
    repository_service
        .init_schema()
        .await
        .expect("Failed to initialize repository schema");

    let memory_service = MemoryService::new(db);
    memory_service
        .init_schema()
        .await
        .expect("Failed to initialize memory schema");

    let state = ToolState::new();
    state.init_repository(repository_service).await;
    state.init_memory(memory_service).await;
    state
}

fn repo_request(action: &str) -> RepoRequest {
    RepoRequest {
        action: action.to_string(),
        cwd: None,
        repository_id: None,
        repository_name: None,
        remote_url: None,
        default_branch: None,
        description: None,
        component_name: None,
        component_path: None,
        component_kind: None,
        project_name: None,
        role: None,
        limit: None,
        migration_review_path: None,
        dry_run: None,
        create_commit: None,
        writer_harness: None,
        writer_harness_version: None,
        model_provider: None,
        model: None,
        model_version: None,
        surface: None,
        actor: None,
        writer_session_id: None,
        include_entity_observations: None,
        include_session_history: None,
        include_work_records: None,
    }
}

fn parse_json(response: &str) -> Value {
    serde_json::from_str(response).expect("response should be valid JSON")
}

fn git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn run_git(cwd: &Path, args: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(args)
        .output()
        .expect("git should run");
    assert!(
        output.status.success(),
        "git {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[tokio::test]
async fn test_mcp_repo_detect_component_link_context_and_orient() {
    if !git_available() {
        return;
    }

    let state = setup_tool_state().await;
    let dir = tempdir().expect("tempdir");
    let component_dir = dir.path().join("services/cogen-backend");
    std::fs::create_dir_all(&component_dir).expect("component dir");
    run_git(dir.path(), &["init"]);
    run_git(
        dir.path(),
        &[
            "remote",
            "add",
            "origin",
            "git@github.com:datadog/dd-source.git",
        ],
    );

    let mut detect = repo_request("detect");
    detect.cwd = Some(dir.path().display().to_string());
    let detect_response = tools::repo_new(&state, detect)
        .await
        .expect("detect should work");
    let detect_json = parse_json(&detect_response);
    let repo_id = detect_json["detection"]["context"]["repository"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(
        detect_json["detection"]["context"]["repository"]["name"],
        "dd-source"
    );

    let mut component = repo_request("component_add");
    component.repository_id = Some(repo_id.clone());
    component.component_name = Some("cogen-backend".to_string());
    component.component_path = Some("services/cogen-backend".to_string());
    component.component_kind = Some("service".to_string());
    tools::repo_new(&state, component)
        .await
        .expect("component_add should work");

    let mut link = repo_request("link_project");
    link.repository_id = Some(repo_id);
    link.project_name = Some("Debug with AI".to_string());
    link.role = Some("primary".to_string());
    link.component_path = Some("services/cogen-backend".to_string());
    tools::repo_new(&state, link)
        .await
        .expect("link_project should work");

    let mut context = repo_request("context");
    context.cwd = Some(component_dir.display().to_string());
    let context_response = tools::repo_new(&state, context)
        .await
        .expect("context should work");
    let context_json = parse_json(&context_response);
    assert_eq!(context_json["matched"], true);
    assert_eq!(
        context_json["context"]["matching_components"][0]["name"],
        "cogen-backend"
    );
    assert_eq!(
        context_json["context"]["linked_projects"][0]["project_name"],
        "Debug with AI"
    );

    let orient_response = tools::orient(
        &state,
        OrientRequest {
            cwd: Some(component_dir.display().to_string()),
            prompt: Some("continue implementation".to_string()),
            project: None,
            agent: Some("codex".to_string()),
            include_recent_commits: Some(false),
            limit: Some(10),
        },
    )
    .await
    .expect("orient should work");
    let orient_json = parse_json(&orient_response);
    assert_eq!(
        orient_json["repository_context"]["repository"]["name"],
        "dd-source"
    );
    assert_eq!(
        orient_json["repository_context"]["linked_projects"][0]["project_name"],
        "Debug with AI"
    );
    assert_eq!(
        orient_json["resolution"]["selected_project"],
        "Debug with AI"
    );
    assert_eq!(orient_json["resolution"]["source"], "component_link");
    assert_eq!(orient_json["resolution"]["requires_confirmation"], false);
}

#[tokio::test]
async fn test_mcp_repo_register_and_list() {
    let state = setup_tool_state().await;

    let mut register = repo_request("register");
    register.repository_name = Some("engram".to_string());
    register.remote_url = Some("git@github.com:ymeiri/engram.git".to_string());
    register.default_branch = Some("main".to_string());
    tools::repo_new(&state, register)
        .await
        .expect("register should work");

    let list_response = tools::repo_new(&state, repo_request("list"))
        .await
        .expect("list should work");
    let list_json = parse_json(&list_response);

    assert_eq!(list_json["count"], 1);
    assert_eq!(list_json["repositories"][0]["name"], "engram");
    assert_eq!(list_json["repositories"][0]["default_branch"], "main");
}

#[tokio::test]
async fn test_mcp_repo_migration_inventory_empty_store() {
    let state = setup_tool_state().await;

    let mut inventory = repo_request("migration_inventory");
    inventory.limit = Some(10);

    let response = tools::repo_new(&state, inventory)
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
async fn test_mcp_repo_migration_review_apply_empty_batch_dry_run() {
    let state = setup_tool_state().await;
    let dir = tempdir().expect("tempdir");

    let mut apply = repo_request("migration_review_apply");
    apply.migration_review_path = Some(dir.path().display().to_string());
    apply.dry_run = Some(true);

    let response = tools::repo_new(&state, apply)
        .await
        .expect("migration_review_apply should work");
    let json = parse_json(&response);

    assert_eq!(json["apply"]["dry_run"], true);
    assert_eq!(json["apply"]["files_scanned"], 0);
    assert_eq!(
        json["apply"]["planned_records"].as_array().unwrap().len(),
        0
    );
    assert_eq!(
        json["apply"]["written_records"].as_array().unwrap().len(),
        0
    );
}

#[tokio::test]
async fn test_mcp_repo_migration_review_status_empty_batch() {
    let state = setup_tool_state().await;
    let dir = tempdir().expect("tempdir");

    let mut status = repo_request("migration_review_status");
    status.migration_review_path = Some(dir.path().display().to_string());

    let response = tools::repo_new(&state, status)
        .await
        .expect("migration_review_status should work");
    let json = parse_json(&response);

    assert_eq!(json["status"]["files_scanned"], 0);
    assert_eq!(json["status"]["planned_record_count"], 0);
    assert_eq!(json["status"]["ready_to_apply"], true);
}
