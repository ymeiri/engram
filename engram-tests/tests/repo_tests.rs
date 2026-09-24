//! Integration tests for repository topology MCP tooling.

use engram_index::{MemoryService, RepositoryService, SearchService, WorkService};
use engram_mcp::tools::{
    self, OrientRequest, OrientResponseShape, RepoRequest, RetrievalScopeRequest, ToolState,
};
use engram_store::{connect_and_init, StoreConfig};
use serde_json::Value;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};
use tempfile::tempdir;

async fn setup_tool_state() -> ToolState {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config).await.expect("Failed to connect");

    let repository_service = RepositoryService::new(db.clone());
    repository_service
        .init_schema()
        .await
        .expect("Failed to initialize repository schema");

    let memory_service = MemoryService::new(db.clone());
    memory_service
        .init_schema()
        .await
        .expect("Failed to initialize memory schema");

    let state = ToolState::new();
    state.init_repository(repository_service).await;
    state.init_memory(memory_service).await;
    state.init_search(SearchService::new(db.clone())).await;
    let work_service = WorkService::new(db);
    work_service
        .init()
        .await
        .expect("Failed to initialize work schema");
    state.init_work(work_service).await;
    state
}

fn repo_request(action: &str) -> RepoRequest {
    RepoRequest {
        action: action.to_string(),
        scope: None,
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

fn global_scope() -> Option<RetrievalScopeRequest> {
    Some(RetrievalScopeRequest {
        relevance_mode: Some("global".to_string()),
        ..RetrievalScopeRequest::default()
    })
}

fn related_scope(project: &str) -> Option<RetrievalScopeRequest> {
    Some(RetrievalScopeRequest {
        relevance_mode: Some("related".to_string()),
        project: Some(project.to_string()),
        ..RetrievalScopeRequest::default()
    })
}

fn related_task_scope(project: &str, task: &str) -> Option<RetrievalScopeRequest> {
    Some(RetrievalScopeRequest {
        relevance_mode: Some("related".to_string()),
        project: Some(project.to_string()),
        task: Some(task.to_string()),
        ..RetrievalScopeRequest::default()
    })
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
            task: None,
            agent: Some("codex".to_string()),
            external_session_id: None,
            intent: None,
            scenario_id: None,
            arm: None,
            include_recent_commits: Some(false),
            limit: Some(10),
            response_shape: None,
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
    assert_eq!(
        orient_json["resolution"]["repository_remote"],
        "github.com/datadog/dd-source"
    );
}

#[tokio::test]
async fn test_mcp_orient_auto_detects_unseen_checkout_by_normalized_remote() {
    if !git_available() {
        return;
    }

    let state = setup_tool_state().await;
    let mut register = repo_request("register");
    register.repository_name = Some("atlas".to_string());
    register.remote_url = Some("git@github.com:acme/atlas.git".to_string());
    let registered = tools::repo_new(&state, register)
        .await
        .expect("register should work");
    let repository_id = parse_json(&registered)["repository"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let mut component = repo_request("component_add");
    component.repository_id = Some(repository_id.clone());
    component.component_name = Some("queue-worker".to_string());
    component.component_path = Some("services/worker".to_string());
    tools::repo_new(&state, component)
        .await
        .expect("component_add should work");

    let mut link = repo_request("link_project");
    link.repository_id = Some(repository_id);
    link.project_name = Some("atlas".to_string());
    link.role = Some("primary".to_string());
    link.component_path = Some("services/worker".to_string());
    tools::repo_new(&state, link)
        .await
        .expect("link_project should work");

    let checkout = tempdir().expect("tempdir");
    let component_dir = checkout.path().join("services/worker");
    std::fs::create_dir_all(&component_dir).expect("component dir");
    run_git(checkout.path(), &["init"]);
    run_git(
        checkout.path(),
        &["remote", "add", "origin", "https://github.com/acme/atlas"],
    );

    let orient_response = tools::orient(
        &state,
        OrientRequest {
            cwd: Some(component_dir.display().to_string()),
            prompt: Some("orient before editing the queue worker".to_string()),
            project: None,
            task: None,
            agent: Some("codex".to_string()),
            external_session_id: None,
            intent: None,
            scenario_id: Some("repo_identity_moved_worktree".to_string()),
            arm: Some("codex_lean_engram".to_string()),
            include_recent_commits: Some(false),
            limit: Some(5),
            response_shape: None,
        },
    )
    .await
    .expect("orient should auto-detect the unseen checkout");
    let orient = parse_json(&orient_response);

    assert_eq!(orient["repository_context"]["repository"]["name"], "atlas");
    assert_eq!(
        orient["repository_context"]["matching_components"][0]["name"],
        "queue-worker"
    );
    assert_eq!(orient["resolution"]["selected_project"], "atlas");
    assert_eq!(orient["resolution"]["source"], "component_link");
    assert_eq!(orient["resolution"]["requires_confirmation"], false);
}

#[tokio::test]
async fn lean_orient_separates_unlinked_checkout_identity_from_project_authorization() {
    if !git_available() {
        return;
    }

    let state = setup_tool_state().await;
    let checkout = tempdir().expect("tempdir");
    let component = checkout.path().join("services/worker");
    std::fs::create_dir_all(&component).expect("component dir");
    run_git(checkout.path(), &["init", "-b", "main"]);
    run_git(checkout.path(), &["config", "user.name", "Engram Test"]);
    run_git(
        checkout.path(),
        &["config", "user.email", "engram-test@example.invalid"],
    );
    run_git(checkout.path(), &["config", "commit.gpgsign", "false"]);
    run_git(
        checkout.path(),
        &["remote", "add", "origin", "git@github.com:acme/orbit.git"],
    );
    std::fs::write(
        component.join("component.json"),
        "{\"name\":\"queue-worker\",\"kind\":\"service\"}\n",
    )
    .expect("component manifest");
    run_git(checkout.path(), &["add", "."]);
    run_git(checkout.path(), &["commit", "-m", "fixture"]);

    let response = tools::orient(
        &state,
        OrientRequest {
            cwd: Some(component.display().to_string()),
            prompt: Some("debug the queue worker with durable procedure memory".to_string()),
            project: None,
            task: None,
            agent: Some("claude_code".to_string()),
            external_session_id: None,
            intent: None,
            scenario_id: Some("unlinked_structured_identity".to_string()),
            arm: Some("provider_free_lean_engram".to_string()),
            include_recent_commits: Some(false),
            limit: Some(5),
            response_shape: Some(OrientResponseShape::Lean),
        },
    )
    .await
    .expect("lean orientation should resolve checkout identity without inventing a project");
    assert!(
        response.len() <= 8_192,
        "lean identity packet exceeded 8192 bytes"
    );
    let response = parse_json(&response);

    assert_eq!(response["identity"]["repository"]["name"], "orbit");
    assert_eq!(
        response["identity"]["repository"]["normalized_remote"],
        "github.com/acme/orbit"
    );
    assert_eq!(
        response["identity"]["repository"]["checkout_root"],
        std::fs::canonicalize(checkout.path())
            .expect("checkout should canonicalize")
            .display()
            .to_string()
    );
    assert_eq!(
        response["identity"]["project"]["status"],
        "requires_confirmation"
    );
    assert_eq!(response["identity"]["project"]["name"], Value::Null);
    assert!(response["identity"]["project"]["project_link_ids"]
        .as_array()
        .is_some_and(Vec::is_empty));
    assert_eq!(
        response["identity"]["components"][0]["source_path"],
        "services/worker/component.json"
    );
    assert_eq!(
        response["identity"]["components"][0]["source_sha256"]
            .as_str()
            .map(str::len),
        Some(64)
    );
    assert_eq!(response["resolution"]["selected_project"], Value::Null);
    assert_eq!(response["resolution"]["requires_confirmation"], true);
}

#[tokio::test]
async fn task_orientation_identifies_git_worktree_and_rejects_other_project_checkout() {
    if !git_available() {
        return;
    }

    let state = setup_tool_state().await;
    {
        let work = state.work_service.read().await;
        let work = work.as_ref().expect("work service should be initialized");
        work.create_project("atlas", None)
            .await
            .expect("atlas project should be created");
        work.create_project("orbit", None)
            .await
            .expect("orbit project should be created");
        work.create_task("atlas", "queue-migration", None, Some("ATLAS-101"))
            .await
            .expect("task should be created");
    }

    let root = tempdir().expect("tempdir");
    let main = root.path().join("atlas-main");
    let worktree = root.path().join("atlas-task-worktree");
    let component = worktree.join("services/worker");
    std::fs::create_dir_all(main.join("services/worker")).expect("component dir");
    run_git(&main, &["init", "-b", "main"]);
    run_git(&main, &["config", "user.name", "Engram Test"]);
    run_git(
        &main,
        &["config", "user.email", "engram-test@example.invalid"],
    );
    run_git(&main, &["config", "commit.gpgsign", "false"]);
    run_git(
        &main,
        &["remote", "add", "origin", "git@github.com:acme/atlas.git"],
    );
    std::fs::write(
        main.join("services/worker/component.json"),
        "{\"name\":\"queue-worker\",\"kind\":\"service\"}\n",
    )
    .expect("component manifest");
    run_git(&main, &["add", "."]);
    run_git(&main, &["commit", "-m", "fixture"]);
    let worktree_path = worktree.display().to_string();
    run_git(
        &main,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "task-atlas-101",
            &worktree_path,
        ],
    );

    let mut register = repo_request("register");
    register.repository_name = Some("atlas".to_string());
    register.remote_url = Some("git@github.com:acme/atlas.git".to_string());
    let registered = tools::repo_new(&state, register)
        .await
        .expect("atlas registration should work");
    let repository_id = parse_json(&registered)["repository"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let mut link = repo_request("link_project");
    link.repository_id = Some(repository_id);
    link.project_name = Some("atlas".to_string());
    link.role = Some("primary".to_string());
    link.component_path = Some("services/worker".to_string());
    tools::repo_new(&state, link)
        .await
        .expect("atlas project link should work");

    for repetition in 1..=5 {
        let started = Instant::now();
        let response = tools::orient(
            &state,
            OrientRequest {
                cwd: Some(component.display().to_string()),
                prompt: Some("continue ATLAS-101".to_string()),
                project: None,
                task: Some("ATLAS-101".to_string()),
                agent: Some("codex".to_string()),
                external_session_id: None,
                intent: None,
                scenario_id: Some("task_worktree_identity".to_string()),
                arm: Some("provider_free_lean_engram".to_string()),
                include_recent_commits: Some(false),
                limit: Some(5),
                response_shape: Some(OrientResponseShape::Lean),
            },
        )
        .await
        .expect("task orientation should resolve the worktree");
        assert!(
            started.elapsed() <= Duration::from_millis(500),
            "task/worktree orientation repetition {repetition} exceeded 500 ms"
        );
        assert!(
            response.len() <= 8_192,
            "task/worktree orientation repetition {repetition} exceeded 8192 bytes"
        );
        let response = parse_json(&response);
        assert_eq!(response["task"], "queue-migration");
        assert_eq!(response["task_context"]["jira_key"], "ATLAS-101");
        assert_eq!(response["scope"], "task:atlas/queue-migration");
        assert_eq!(response["resolution"]["explicit_project"], Value::Null);
        assert_eq!(response["resolution"]["selected_project"], "atlas");
        assert_eq!(response["resolution"]["source"], "task");
        assert_eq!(response["resolution"]["repository_name"], "atlas");
        assert_eq!(response["resolution"]["component_names"][0], "queue-worker");
        assert_eq!(response["identity"]["project"]["status"], "authorized");
        assert_eq!(response["identity"]["project"]["name"], "atlas");
        assert_eq!(response["identity"]["repository"]["name"], "atlas");
        assert_eq!(
            response["identity"]["repository"]["normalized_remote"],
            "github.com/acme/atlas"
        );
        assert_eq!(
            response["identity"]["repository"]["checkout_root"],
            std::fs::canonicalize(&worktree)
                .expect("worktree should canonicalize")
                .display()
                .to_string()
        );
        assert_eq!(
            response["identity"]["components"][0]["source_path"],
            "services/worker/component.json"
        );
        assert_eq!(
            response["identity"]["components"][0]["source_sha256"]
                .as_str()
                .map(str::len),
            Some(64)
        );
    }

    let mut context = repo_request("context");
    context.cwd = Some(component.display().to_string());
    let context = tools::repo_new(&state, context)
        .await
        .expect("progressive repository context should work");
    let context = parse_json(&context);
    assert_eq!(context["matched"], true);
    assert_eq!(
        context["context"]["checkout"]["local_path"],
        std::fs::canonicalize(&worktree)
            .expect("worktree should canonicalize")
            .display()
            .to_string()
    );
    assert_eq!(
        context["context"]["checkout"]["current_branch"],
        "task-atlas-101"
    );
    assert_eq!(context["context"]["checkout"]["is_dirty"], false);
    assert!(context["context"]["checkout"]["head_sha"]
        .as_str()
        .is_some_and(|sha| sha.len() == 40));

    let orbit = root.path().join("orbit-main");
    std::fs::create_dir_all(&orbit).expect("orbit dir");
    run_git(&orbit, &["init", "-b", "main"]);
    run_git(
        &orbit,
        &["remote", "add", "origin", "git@github.com:acme/orbit.git"],
    );
    let mut register = repo_request("register");
    register.repository_name = Some("orbit".to_string());
    register.remote_url = Some("git@github.com:acme/orbit.git".to_string());
    let registered = tools::repo_new(&state, register)
        .await
        .expect("orbit registration should work");
    let orbit_id = parse_json(&registered)["repository"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let mut link = repo_request("link_project");
    link.repository_id = Some(orbit_id);
    link.project_name = Some("orbit".to_string());
    link.role = Some("primary".to_string());
    tools::repo_new(&state, link)
        .await
        .expect("orbit project link should work");

    let error = tools::orient(
        &state,
        OrientRequest {
            cwd: Some(orbit.display().to_string()),
            prompt: Some("continue ATLAS-101".to_string()),
            project: None,
            task: Some("ATLAS-101".to_string()),
            agent: Some("codex".to_string()),
            external_session_id: None,
            intent: None,
            scenario_id: Some("task_checkout_conflict".to_string()),
            arm: Some("provider_free_lean_engram".to_string()),
            include_recent_commits: Some(false),
            limit: Some(5),
            response_shape: Some(OrientResponseShape::Lean),
        },
    )
    .await
    .expect_err("a task from another project must not override checkout identity");
    assert!(error.contains("Task 'queue-migration' belongs to project 'atlas'"));
    assert!(error.contains("repository 'orbit' candidate(s): orbit"));
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

    let mut list = repo_request("list");
    list.scope = global_scope();
    let list_response = tools::repo_new(&state, list)
        .await
        .expect("list should work");
    let list_json = parse_json(&list_response);

    assert_eq!(list_json["count"], 1);
    assert_eq!(list_json["repositories"][0]["name"], "engram");
    assert_eq!(list_json["repositories"][0]["default_branch"], "main");
}

#[tokio::test]
async fn mcp_repo_admin_reads_abstain_locally_before_service_access() {
    let state = ToolState::new();

    for (action, surface) in [
        ("list", "repositories"),
        ("migration_inventory", "repository_migration"),
        ("migration_review_export", "repository_migration"),
        ("migration_review_status", "repository_migration_review"),
        ("migration_review_apply", "repository_migration_review"),
    ] {
        let response = tools::repo_new(&state, repo_request(action))
            .await
            .unwrap_or_else(|error| {
                panic!("{action} should abstain before service access: {error}")
            });
        let json = parse_json(&response);
        assert_eq!(json["executed"], false, "unexpected response for {action}");
        assert_eq!(json["relevance_mode"], "local");
        assert_eq!(json["authorization_scope_enforced"], true);
        assert_eq!(json["omitted_layers"], serde_json::json!([surface]));
    }
}

#[tokio::test]
async fn mcp_repo_related_scope_filters_links_inventory_and_review_admin() {
    let state = setup_tool_state().await;
    {
        let guard = state.work_service.read().await;
        let work = guard.as_ref().expect("work service");
        work.create_project(
            "alpha",
            Some("Source repository https://github.com/acme/alpha"),
        )
        .await
        .unwrap();
        work.create_project(
            "beta",
            Some("Source repository https://github.com/acme/beta"),
        )
        .await
        .unwrap();
        work.create_task("alpha", "alpha-one", None, Some("ALPHA-1"))
            .await
            .unwrap();
    }

    let mut alpha_register = repo_request("register");
    alpha_register.repository_name = Some("alpha-repo".to_string());
    alpha_register.remote_url = Some("https://github.com/acme/alpha".to_string());
    let alpha = parse_json(
        &tools::repo_new(&state, alpha_register)
            .await
            .expect("register alpha repository"),
    );
    let alpha_id = alpha["repository"]["id"].as_str().unwrap().to_string();

    let mut beta_register = repo_request("register");
    beta_register.repository_name = Some("beta-repo".to_string());
    beta_register.remote_url = Some("https://github.com/acme/beta".to_string());
    let beta = parse_json(
        &tools::repo_new(&state, beta_register)
            .await
            .expect("register beta repository"),
    );
    let beta_id = beta["repository"]["id"].as_str().unwrap().to_string();

    for (project, repository_id) in [("alpha", alpha_id), ("beta", beta_id)] {
        let mut link = repo_request("link_project");
        link.project_name = Some(project.to_string());
        link.repository_id = Some(repository_id);
        tools::repo_new(&state, link)
            .await
            .unwrap_or_else(|error| panic!("link {project} repository: {error}"));
    }

    let mut related_list = repo_request("list");
    related_list.scope = related_scope("alpha");
    related_list.limit = Some(1);
    let related = parse_json(
        &tools::repo_new(&state, related_list)
            .await
            .expect("related repository list"),
    );
    assert_eq!(related["count"], 1);
    assert_eq!(related["repositories"][0]["name"], "alpha-repo");
    assert_eq!(related["resolved_project"], "alpha");
    assert_eq!(related["authorization_scope_enforced"], true);

    let mut exact_task_list = repo_request("list");
    exact_task_list.scope = related_task_scope("alpha", "ALPHA-1");
    let exact_task = parse_json(
        &tools::repo_new(&state, exact_task_list)
            .await
            .expect("exact-task repository list should abstain"),
    );
    assert_eq!(exact_task["executed"], false);
    assert_eq!(
        exact_task["omitted_layers"],
        serde_json::json!(["repositories"])
    );
    assert_eq!(exact_task["resolved_task"], "alpha-one");

    let mut inventory = repo_request("migration_inventory");
    inventory.scope = related_scope("alpha");
    let inventory = parse_json(
        &tools::repo_new(&state, inventory)
            .await
            .expect("related migration inventory"),
    );
    assert_eq!(inventory["inventory"]["project_filter"], "alpha");
    assert!(inventory["inventory"]["by_project"].get("beta").is_none());
    assert_eq!(inventory["resolved_project"], "alpha");

    let export_dir = tempdir().expect("migration export dir");
    let mut export = repo_request("migration_review_export");
    export.scope = related_scope("alpha");
    export.migration_review_path = Some(export_dir.path().display().to_string());
    let export = parse_json(
        &tools::repo_new(&state, export)
            .await
            .expect("related migration export"),
    );
    assert_eq!(export["export"]["inventory"]["project_filter"], "alpha");
    assert_eq!(export["resolved_project"], "alpha");

    let mut status = repo_request("migration_review_status");
    status.scope = related_scope("alpha");
    status.migration_review_path = Some(export_dir.path().display().to_string());
    let status = parse_json(
        &tools::repo_new(&state, status)
            .await
            .expect("related review status should abstain"),
    );
    assert_eq!(status["executed"], false);
    assert_eq!(
        status["omitted_layers"],
        serde_json::json!(["repository_migration_review"])
    );

    let mut mismatch = repo_request("list");
    mismatch.project_name = Some("beta".to_string());
    mismatch.scope = related_scope("alpha");
    let error = tools::repo_new(&state, mismatch)
        .await
        .expect_err("mismatched project filter must be rejected");
    assert!(error.contains("does not match resolved authorization project 'alpha'"));

    let mut global = repo_request("list");
    global.scope = global_scope();
    let global = parse_json(
        &tools::repo_new(&state, global)
            .await
            .expect("explicit global repository list"),
    );
    assert_eq!(global["count"], 2);
    assert_eq!(global["authorization_scope_enforced"], false);
}

#[tokio::test]
async fn mcp_repo_context_is_local_and_related_project_links_are_filtered() {
    if !git_available() {
        return;
    }

    let state = setup_tool_state().await;
    {
        let guard = state.work_service.read().await;
        let work = guard.as_ref().expect("work service");
        for project in ["alpha", "beta", "gamma"] {
            work.create_project(project, None).await.unwrap();
        }
        work.create_task("alpha", "alpha-one", None, Some("ALPHA-1"))
            .await
            .unwrap();
    }

    let checkout = tempdir().expect("checkout");
    run_git(checkout.path(), &["init"]);
    run_git(
        checkout.path(),
        &["remote", "add", "origin", "https://github.com/acme/shared"],
    );
    let mut detect = repo_request("detect");
    detect.cwd = Some(checkout.path().display().to_string());
    let detected = parse_json(
        &tools::repo_new(&state, detect)
            .await
            .expect("detect repository"),
    );
    let repository_id = detected["detection"]["context"]["repository"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    for project in ["alpha", "beta"] {
        let mut link = repo_request("link_project");
        link.project_name = Some(project.to_string());
        link.repository_id = Some(repository_id.clone());
        tools::repo_new(&state, link).await.unwrap();
    }

    let mut local = repo_request("context");
    local.cwd = Some(checkout.path().display().to_string());
    let local = parse_json(
        &tools::repo_new(&state, local)
            .await
            .expect("local repository context"),
    );
    assert_eq!(local["relevance_mode"], "local");
    assert_eq!(local["authorization_scope_enforced"], true);
    assert_eq!(
        local["scope_enforced_layers"],
        serde_json::json!(["repository_context"])
    );
    assert_eq!(
        local["context"]["linked_projects"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    let mut related = repo_request("context");
    related.cwd = Some(checkout.path().display().to_string());
    related.scope = related_scope("alpha");
    let related = parse_json(
        &tools::repo_new(&state, related)
            .await
            .expect("related repository context"),
    );
    assert_eq!(related["resolved_project"], "alpha");
    assert_eq!(
        related["context"]["linked_projects"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        related["context"]["linked_projects"][0]["project_name"],
        "alpha"
    );

    let mut wrong_project = repo_request("context");
    wrong_project.cwd = Some(checkout.path().display().to_string());
    wrong_project.scope = related_scope("gamma");
    let error = tools::repo_new(&state, wrong_project)
        .await
        .expect_err("unlinked project context must be rejected");
    assert!(error.contains("outside resolved authorization project 'gamma'"));

    let mut exact_task = repo_request("context");
    exact_task.cwd = Some(checkout.path().display().to_string());
    exact_task.scope = related_task_scope("alpha", "ALPHA-1");
    let exact_task = parse_json(
        &tools::repo_new(&state, exact_task)
            .await
            .expect("exact-task repository context should abstain"),
    );
    assert_eq!(exact_task["matched"], false);
    assert_eq!(
        exact_task["omitted_layers"],
        serde_json::json!(["repository_context"])
    );
}

#[tokio::test]
async fn test_mcp_repo_migration_inventory_empty_store() {
    let state = setup_tool_state().await;

    let mut inventory = repo_request("migration_inventory");
    inventory.limit = Some(10);
    inventory.scope = global_scope();

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
    apply.scope = global_scope();

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
    status.scope = global_scope();

    let response = tools::repo_new(&state, status)
        .await
        .expect("migration_review_status should work");
    let json = parse_json(&response);

    assert_eq!(json["status"]["files_scanned"], 0);
    assert_eq!(json["status"]["planned_record_count"], 0);
    assert_eq!(json["status"]["ready_to_apply"], true);
}
