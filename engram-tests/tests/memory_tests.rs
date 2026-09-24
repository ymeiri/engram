//! Integration tests for Memory OS MCP tooling.

use engram_core::telemetry::AgentFeedback;
use engram_core::Id;
use engram_index::{
    EntityService, GraphService, MemoryService, ProcedureVerificationReceipt, SearchService,
    WorkService,
};
use engram_mcp::tools::{
    self, EntityObserveRequestNew, EntityRequestNew, MemoryChangeRequest, MemoryEvidenceRequest,
    MemoryProcedurePrerequisiteSourceRequest, MemoryProcedureRequest, MemoryRequestNew,
    OrientRequest, OrientResponseShape, RetrievalScopeRequest, SearchRequest, ToolState,
    VaultRequest,
};
use engram_store::{connect_and_init, StoreConfig, TelemetryRepo};
use serde_json::Value;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

async fn setup_tool_state() -> ToolState {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config).await.expect("Failed to connect");
    let memory_service = MemoryService::new(db.clone());
    memory_service
        .init_schema()
        .await
        .expect("Failed to initialize memory schema");
    let entity_service = EntityService::new(db.clone());
    entity_service
        .init()
        .await
        .expect("Failed to initialize entity schema");

    let state = ToolState::new();
    state.init_memory(memory_service).await;
    state.init_entity(entity_service).await;
    state.init_search(SearchService::new(db.clone())).await;
    let work_service = WorkService::new(db);
    work_service
        .init()
        .await
        .expect("Failed to initialize work schema");
    state.init_work(work_service).await;
    state
}

fn global_scope() -> Option<RetrievalScopeRequest> {
    Some(RetrievalScopeRequest {
        relevance_mode: Some("global".to_string()),
        ..RetrievalScopeRequest::default()
    })
}

fn local_project_scope(project: &str) -> Option<RetrievalScopeRequest> {
    Some(RetrievalScopeRequest {
        relevance_mode: Some("local".to_string()),
        project: Some(project.to_string()),
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

fn request(action: &str) -> MemoryRequestNew {
    MemoryRequestNew {
        action: action.to_string(),
        scope: global_scope(),
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
        external_session_id: None,
        evidence: Vec::new(),
        procedure: None,
        conditions: std::collections::BTreeMap::new(),
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
        replacement_id: None,
        correction_reason: None,
        confirm_correction: None,
        proposal_id: None,
        expected_digest: None,
        receipt: None,
        expires_at: None,
        confirm_forget: None,
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

fn manual_review_evidence(summary: &str) -> Vec<MemoryEvidenceRequest> {
    vec![MemoryEvidenceRequest {
        kind: "manual_review".to_string(),
        target: "memory_tests".to_string(),
        summary: Some(summary.to_string()),
        excerpt: None,
    }]
}

fn vault_request(action: &str, path: &str) -> VaultRequest {
    VaultRequest {
        action: action.to_string(),
        vault_path: Some(path.to_string()),
        page: None,
        scope: global_scope(),
    }
}

#[tokio::test]
async fn mcp_vault_reads_abstain_before_filesystem_or_service_access() {
    let dir = tempdir().expect("tempdir should be created");
    fs::write(dir.path().join("private.md"), "filesystem canary").unwrap();

    for action in ["compile", "status", "page"] {
        let mut request = vault_request(action, &dir.path().display().to_string());
        request.scope = None;
        request.page = Some("private.md".to_string());
        let response = tools::vault_new(&ToolState::new(), request)
            .await
            .unwrap_or_else(|error| {
                panic!("{action} should abstain before filesystem access: {error}")
            });
        let json = parse_json(&response);
        assert_eq!(json["executed"], false, "unexpected response for {action}");
        assert_eq!(json["relevance_mode"], "local");
        assert_eq!(json["authorization_scope_enforced"], true);
        assert_eq!(json["omitted_layers"], serde_json::json!(["vault"]));
        assert!(!response.contains("filesystem canary"));
    }

    let error = tools::vault_new(
        &ToolState::new(),
        VaultRequest {
            action: "unknown".to_string(),
            vault_path: None,
            page: None,
            scope: None,
        },
    )
    .await
    .expect_err("unknown actions should be rejected before scope handling");
    assert!(error.contains("Unknown action"));
}

#[tokio::test]
async fn mcp_vault_related_scope_abstains_when_file_ownership_is_unprovable() {
    let state = setup_tool_state().await;
    {
        let guard = state.work_service.read().await;
        let work = guard.as_ref().expect("work service");
        work.create_project("alpha", None).await.unwrap();
        work.create_task("alpha", "alpha-one", None, Some("ALPHA-1"))
            .await
            .unwrap();
    }
    let dir = tempdir().expect("tempdir should be created");
    fs::write(dir.path().join("private.md"), "filesystem canary").unwrap();

    for scope in [
        related_scope("alpha"),
        related_task_scope("alpha", "ALPHA-1"),
    ] {
        let mut request = vault_request("page", &dir.path().display().to_string());
        request.scope = scope;
        request.page = Some("private.md".to_string());
        let response = tools::vault_new(&state, request).await.unwrap();
        let json = parse_json(&response);
        assert_eq!(json["executed"], false);
        assert_eq!(json["relevance_mode"], "related");
        assert_eq!(json["resolved_project"], "alpha");
        assert_eq!(json["omitted_layers"], serde_json::json!(["vault"]));
        assert!(!response.contains("filesystem canary"));
    }

    let mut compile = vault_request("compile", &dir.path().display().to_string());
    compile.scope = related_scope("alpha");
    let response = tools::vault_new(&state, compile).await.unwrap();
    assert_eq!(parse_json(&response)["executed"], false);
    assert!(!dir.path().join("memory/index.md").exists());
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
async fn mcp_memory_admin_reads_abstain_locally_before_service_access() {
    let state = ToolState::new();

    for (action, surface) in [
        ("log", "memory_admin"),
        ("diff", "memory_admin"),
        ("writer_stats", "memory_admin"),
        ("export_vault", "memory_admin"),
        ("migration_inventory", "memory_migration"),
        ("migration_review_export", "memory_migration"),
        ("migration_review_status", "memory_admin"),
        ("migration_review_apply", "memory_admin"),
        ("digest_extraction_apply", "memory_admin"),
    ] {
        let mut req = request(action);
        req.scope = None;
        let response = tools::memory_new(&state, req)
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
async fn mcp_memory_scope_filters_items_reviews_changes_and_migration() {
    let state = setup_tool_state().await;
    {
        let guard = state.work_service.read().await;
        let work = guard.as_ref().expect("work service");
        work.create_project("alpha", None).await.unwrap();
        work.create_project("beta", None).await.unwrap();
        work.create_task("alpha", "alpha-one", None, Some("ALPHA-1"))
            .await
            .unwrap();
        work.create_task("alpha", "alpha-two", None, Some("ALPHA-2"))
            .await
            .unwrap();
    }

    let cursor = parse_json(
        &tools::memory_new(&state, request("cursor"))
            .await
            .expect("cursor"),
    );
    let cursor_timestamp = cursor["cursor"]["timestamp"].as_str().unwrap().to_string();

    let fixtures = [
        ("global", None, None, "global-active", "active"),
        ("project", Some("alpha"), None, "alpha-project", "active"),
        ("project", Some("beta"), None, "beta-project", "active"),
        (
            "task",
            Some("alpha"),
            Some("alpha-one"),
            "alpha-one",
            "active",
        ),
        (
            "task",
            Some("alpha"),
            Some("alpha-two"),
            "alpha-two",
            "active",
        ),
        (
            "project",
            Some("alpha"),
            None,
            "alpha-review",
            "needs_review",
        ),
        ("project", Some("beta"), None, "beta-review", "needs_review"),
    ];
    let mut ids = std::collections::BTreeMap::new();
    for (scope_type, project, task, title, status) in fixtures {
        let mut add = with_writer(request("add"));
        add.kind = Some("decision".to_string());
        add.title = Some(title.to_string());
        add.content = Some(format!("Scoped fixture for {title}."));
        add.origin = Some("user_stated".to_string());
        add.scope_type = Some(scope_type.to_string());
        add.project_name = project.map(str::to_string);
        add.task_name = task.map(str::to_string);
        add.status = Some(status.to_string());
        add.evidence = manual_review_evidence("Reviewed authorization fixture.");
        let response = tools::memory_new(&state, add)
            .await
            .unwrap_or_else(|error| panic!("add {title}: {error}"));
        ids.insert(
            title,
            parse_json(&response)["item"]["id"]
                .as_str()
                .unwrap()
                .to_string(),
        );
    }

    let mut commit = with_writer(request("commit"));
    commit.message = Some("Cross-project authorization canary commit".to_string());
    commit.changes = vec![MemoryChangeRequest {
        change_type: "added".to_string(),
        item_id: Some(ids["beta-project"].clone()),
        title: "Beta project".to_string(),
        summary: "Cross-project summary must not appear in scoped changes_since.".to_string(),
        before_hash: None,
        after_hash: None,
    }];
    tools::memory_new(&state, commit)
        .await
        .expect("commit canary");

    let mut local = request("list");
    local.scope = None;
    local.status_filter = Some("active".to_string());
    let local = parse_json(
        &tools::memory_new(&state, local)
            .await
            .expect("local memory list"),
    );
    let local_titles = local["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["title"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(local_titles, vec!["global-active"]);
    assert_eq!(local["authorization_scope_enforced"], true);

    let mut related = request("list");
    related.scope = related_scope("alpha");
    related.status_filter = Some("active".to_string());
    related.limit = Some(4);
    let related = parse_json(
        &tools::memory_new(&state, related)
            .await
            .expect("related memory list"),
    );
    let related_titles = related["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["title"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(related_titles.contains(&"global-active"));
    assert!(related_titles.contains(&"alpha-project"));
    assert!(related_titles.contains(&"alpha-one"));
    assert!(related_titles.contains(&"alpha-two"));
    assert!(!related_titles.contains(&"beta-project"));
    assert_eq!(related["resolved_project"], "alpha");

    let mut exact_task = request("list");
    exact_task.scope = related_task_scope("alpha", "ALPHA-1");
    exact_task.status_filter = Some("active".to_string());
    let exact_task = parse_json(
        &tools::memory_new(&state, exact_task)
            .await
            .expect("exact-task memory list"),
    );
    let exact_titles = exact_task["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["title"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(exact_titles.contains(&"global-active"));
    assert!(exact_titles.contains(&"alpha-project"));
    assert!(exact_titles.contains(&"alpha-one"));
    assert!(!exact_titles.contains(&"alpha-two"));
    assert!(!exact_titles.contains(&"beta-project"));
    assert_eq!(exact_task["resolved_task"], "alpha-one");

    let mut wrong_get = request("get");
    wrong_get.scope = related_scope("alpha");
    wrong_get.id = Some(ids["beta-project"].clone());
    let error = tools::memory_new(&state, wrong_get)
        .await
        .expect_err("wrong-project get must fail");
    assert!(error.contains("outside the resolved authorization boundary"));

    let mut mismatch = request("list");
    mismatch.scope = related_scope("alpha");
    mismatch.scope_type = Some("project".to_string());
    mismatch.project_name = Some("beta".to_string());
    let error = tools::memory_new(&state, mismatch)
        .await
        .expect_err("mismatched project filter must fail");
    assert!(error.contains("does not match resolved authorization project 'alpha'"));

    let mut review = request("review");
    review.scope = related_scope("alpha");
    let review = parse_json(
        &tools::memory_new(&state, review)
            .await
            .expect("related review queue"),
    );
    assert_eq!(review["count"], 1);
    assert_eq!(review["items"][0]["title"], "alpha-review");

    let mut procedure = request("procedure_match");
    procedure.scope = global_scope();
    procedure.query = Some("authorization canary".to_string());
    let procedure = parse_json(
        &tools::memory_new(&state, procedure)
            .await
            .expect("local procedure_match"),
    );
    assert_eq!(procedure["abstained"], true);
    assert_eq!(procedure["relevance_mode"], "local");
    assert_eq!(procedure["authorization_scope_enforced"], true);

    let mut changes = request("changes_since");
    changes.scope = related_task_scope("alpha", "ALPHA-1");
    changes.timestamp = Some(cursor_timestamp.clone());
    let changes = parse_json(
        &tools::memory_new(&state, changes)
            .await
            .expect("scoped changes_since"),
    );
    let changed_titles = changes["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["title"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(changed_titles.contains(&"global-active"));
    assert!(changed_titles.contains(&"alpha-project"));
    assert!(changed_titles.contains(&"alpha-one"));
    assert!(!changed_titles.contains(&"alpha-two"));
    assert!(!changed_titles.contains(&"beta-project"));
    assert_eq!(changes["commit_count"], 0);
    assert_eq!(changes["commits_omitted"], true);

    let mut global_changes = request("changes_since");
    global_changes.timestamp = Some(cursor_timestamp);
    let global_changes = parse_json(
        &tools::memory_new(&state, global_changes)
            .await
            .expect("global changes_since"),
    );
    assert_eq!(global_changes["item_count"], 7);
    assert_eq!(global_changes["commit_count"], 1);
    assert_eq!(global_changes["commits_omitted"], false);
    assert_eq!(global_changes["authorization_scope_enforced"], false);

    let mut migration = request("migration_inventory");
    migration.scope = related_scope("alpha");
    let migration = parse_json(
        &tools::memory_new(&state, migration)
            .await
            .expect("related migration inventory"),
    );
    assert_eq!(migration["inventory"]["project_filter"], "alpha");
    assert_eq!(migration["resolved_project"], "alpha");
}

#[tokio::test]
async fn test_mcp_memory_list_filters_by_tags_before_limit() {
    let state = setup_tool_state().await;

    let mut matching = with_writer(request("add"));
    matching.kind = Some("decision".to_string());
    matching.title = Some("Tagged current plan".to_string());
    matching.content =
        Some("This tagged item should be returned by tag-filtered list.".to_string());
    matching.origin = Some("user_stated".to_string());
    matching.scope_type = Some("project".to_string());
    matching.project_name = Some("engram".to_string());
    matching.tags = vec!["current-plan".to_string(), "brain-harness".to_string()];
    matching.evidence = manual_review_evidence("Reviewed tagged list fixture.");
    tools::memory_new(&state, matching)
        .await
        .expect("matching add should work");

    let mut non_matching = with_writer(request("add"));
    non_matching.kind = Some("project_fact".to_string());
    non_matching.title = Some("Newer untagged fact".to_string());
    non_matching.content = Some("This newer item should not satisfy the tag filter.".to_string());
    non_matching.origin = Some("tool_result".to_string());
    non_matching.scope_type = Some("project".to_string());
    non_matching.project_name = Some("engram".to_string());
    non_matching.tags = vec!["current-plan".to_string()];
    non_matching.evidence = manual_review_evidence("Reviewed nonmatching tagged list fixture.");
    tools::memory_new(&state, non_matching)
        .await
        .expect("non-matching add should work");

    let mut list = request("list");
    list.status_filter = Some("active".to_string());
    list.tags = vec!["current-plan".to_string(), "brain-harness".to_string()];
    list.limit = Some(1);
    let list_response = tools::memory_new(&state, list)
        .await
        .expect("list should work");
    let list_json = parse_json(&list_response);

    assert_eq!(list_json["count"], 1);
    assert_eq!(list_json["items"][0]["title"], "Tagged current plan");
}

#[tokio::test]
async fn test_mcp_memory_list_filters_by_scope_before_limit() {
    let state = setup_tool_state().await;

    let mut matching = with_writer(request("add"));
    matching.kind = Some("decision".to_string());
    matching.title = Some("Engram current plan".to_string());
    matching.content =
        Some("This project-scoped Engram current plan should be returned.".to_string());
    matching.origin = Some("tool_result".to_string());
    matching.scope_type = Some("project".to_string());
    matching.project_name = Some("engram".to_string());
    matching.tags = vec!["current-plan".to_string()];
    matching.evidence = manual_review_evidence("Reviewed scoped list matching fixture.");
    tools::memory_new(&state, matching)
        .await
        .expect("matching add should work");

    tokio::time::sleep(std::time::Duration::from_millis(1)).await;

    let mut wrong_scope = with_writer(request("add"));
    wrong_scope.kind = Some("decision".to_string());
    wrong_scope.title = Some("Voice layer current plan".to_string());
    wrong_scope.content =
        Some("This newer wrong-project current plan should not be returned.".to_string());
    wrong_scope.origin = Some("tool_result".to_string());
    wrong_scope.scope_type = Some("project".to_string());
    wrong_scope.project_name = Some("voice-layer".to_string());
    wrong_scope.tags = vec!["current-plan".to_string()];
    wrong_scope.evidence = manual_review_evidence("Reviewed scoped list wrong-project fixture.");
    tools::memory_new(&state, wrong_scope)
        .await
        .expect("wrong-scope add should work");

    let mut list = request("list");
    list.status_filter = Some("active".to_string());
    list.scope_type = Some("project".to_string());
    list.project_name = Some("engram".to_string());
    list.limit = Some(1);
    let list_response = tools::memory_new(&state, list)
        .await
        .expect("list should work");
    let list_json = parse_json(&list_response);

    assert_eq!(list_json["count"], 1);
    assert_eq!(list_json["items"][0]["title"], "Engram current plan");
    assert_eq!(list_json["items"][0]["scope"]["project_name"], "engram");
}

#[tokio::test]
async fn test_mcp_memory_list_applies_limit_after_scope_filter() {
    let state = setup_tool_state().await;

    let mut older_matching = with_writer(request("add"));
    older_matching.kind = Some("decision".to_string());
    older_matching.title = Some("Older Engram scoped item".to_string());
    older_matching.content = Some("This older matching item should be eligible.".to_string());
    older_matching.origin = Some("tool_result".to_string());
    older_matching.scope_type = Some("project".to_string());
    older_matching.project_name = Some("engram".to_string());
    older_matching.evidence = manual_review_evidence("Reviewed older scoped limit fixture.");
    tools::memory_new(&state, older_matching)
        .await
        .expect("older matching add should work");

    tokio::time::sleep(std::time::Duration::from_millis(1)).await;

    let mut newer_matching = with_writer(request("add"));
    newer_matching.kind = Some("decision".to_string());
    newer_matching.title = Some("Newer Engram scoped item".to_string());
    newer_matching.content = Some("This newer matching item should be eligible.".to_string());
    newer_matching.origin = Some("tool_result".to_string());
    newer_matching.scope_type = Some("project".to_string());
    newer_matching.project_name = Some("engram".to_string());
    newer_matching.evidence = manual_review_evidence("Reviewed newer scoped limit fixture.");
    tools::memory_new(&state, newer_matching)
        .await
        .expect("newer matching add should work");

    tokio::time::sleep(std::time::Duration::from_millis(1)).await;

    let mut wrong_scope = with_writer(request("add"));
    wrong_scope.kind = Some("decision".to_string());
    wrong_scope.title = Some("DD source scoped item".to_string());
    wrong_scope.content = Some("This newer wrong-project item should not be returned.".to_string());
    wrong_scope.origin = Some("tool_result".to_string());
    wrong_scope.scope_type = Some("project".to_string());
    wrong_scope.project_name = Some("dd-source".to_string());
    wrong_scope.evidence = manual_review_evidence("Reviewed wrong-scope limit fixture.");
    tools::memory_new(&state, wrong_scope)
        .await
        .expect("wrong-scope add should work");

    let mut list = request("list");
    list.status_filter = Some("active".to_string());
    list.scope_type = Some("project".to_string());
    list.project_name = Some("engram".to_string());
    list.limit = Some(1);
    let list_response = tools::memory_new(&state, list)
        .await
        .expect("list should work");
    let list_json = parse_json(&list_response);

    assert_eq!(list_json["count"], 1);
    assert_eq!(list_json["items"][0]["scope"]["project_name"], "engram");
}

#[tokio::test]
async fn test_mcp_memory_list_project_name_implies_project_scope_before_limit() {
    let state = setup_tool_state().await;

    let mut matching = with_writer(request("add"));
    matching.kind = Some("decision".to_string());
    matching.title = Some("Engram project-only current plan".to_string());
    matching.content = Some("This project-scoped item should be returned.".to_string());
    matching.origin = Some("tool_result".to_string());
    matching.scope_type = Some("project".to_string());
    matching.project_name = Some("engram".to_string());
    matching.evidence = manual_review_evidence("Reviewed project-name-only list fixture.");
    tools::memory_new(&state, matching)
        .await
        .expect("matching add should work");

    tokio::time::sleep(std::time::Duration::from_millis(1)).await;

    let mut wrong_scope = with_writer(request("add"));
    wrong_scope.kind = Some("decision".to_string());
    wrong_scope.title = Some("DD source project-only current plan".to_string());
    wrong_scope.content = Some("This newer wrong-project item should not be returned.".to_string());
    wrong_scope.origin = Some("tool_result".to_string());
    wrong_scope.scope_type = Some("project".to_string());
    wrong_scope.project_name = Some("dd-source".to_string());
    wrong_scope.evidence =
        manual_review_evidence("Reviewed project-name-only wrong-project fixture.");
    tools::memory_new(&state, wrong_scope)
        .await
        .expect("wrong-scope add should work");

    let mut list = request("list");
    list.status_filter = Some("active".to_string());
    list.project_name = Some("engram".to_string());
    list.limit = Some(1);
    let list_response = tools::memory_new(&state, list)
        .await
        .expect("list should work");
    let list_json = parse_json(&list_response);

    assert_eq!(list_json["count"], 1);
    assert_eq!(
        list_json["items"][0]["title"],
        "Engram project-only current plan"
    );
    assert_eq!(list_json["items"][0]["scope"]["project_name"], "engram");
}

#[tokio::test]
async fn test_mcp_memory_list_project_name_scope_inference_filters_current_plan_tags() {
    let state = setup_tool_state().await;

    let mut matching = with_writer(request("add"));
    matching.kind = Some("decision".to_string());
    matching.title = Some("Engram tagged current plan".to_string());
    matching.content =
        Some("This tagged Engram project-scoped item should be returned.".to_string());
    matching.origin = Some("tool_result".to_string());
    matching.scope_type = Some("project".to_string());
    matching.project_name = Some("engram".to_string());
    matching.tags = vec!["current-plan".to_string()];
    matching.evidence = manual_review_evidence("Reviewed project-name tag matching fixture.");
    tools::memory_new(&state, matching)
        .await
        .expect("matching add should work");

    tokio::time::sleep(std::time::Duration::from_millis(1)).await;

    let mut untagged = with_writer(request("add"));
    untagged.kind = Some("project_fact".to_string());
    untagged.title = Some("Engram untagged project fact".to_string());
    untagged.content =
        Some("This same-project item should not satisfy the tag filter.".to_string());
    untagged.origin = Some("tool_result".to_string());
    untagged.scope_type = Some("project".to_string());
    untagged.project_name = Some("engram".to_string());
    untagged.evidence = manual_review_evidence("Reviewed project-name tag untagged fixture.");
    tools::memory_new(&state, untagged)
        .await
        .expect("untagged add should work");

    tokio::time::sleep(std::time::Duration::from_millis(1)).await;

    let mut wrong_scope = with_writer(request("add"));
    wrong_scope.kind = Some("decision".to_string());
    wrong_scope.title = Some("Voice layer tagged current plan".to_string());
    wrong_scope.content =
        Some("This newer wrong-project tagged item should not be returned.".to_string());
    wrong_scope.origin = Some("tool_result".to_string());
    wrong_scope.scope_type = Some("project".to_string());
    wrong_scope.project_name = Some("voice-layer".to_string());
    wrong_scope.tags = vec!["current-plan".to_string()];
    wrong_scope.evidence =
        manual_review_evidence("Reviewed project-name tag wrong-project fixture.");
    tools::memory_new(&state, wrong_scope)
        .await
        .expect("wrong-scope add should work");

    let mut list = request("list");
    list.status_filter = Some("active".to_string());
    list.project_name = Some("engram".to_string());
    list.tags = vec!["current-plan".to_string()];
    let list_response = tools::memory_new(&state, list)
        .await
        .expect("list should work");
    let list_json = parse_json(&list_response);

    assert_eq!(list_json["count"], 1);
    assert_eq!(list_json["items"][0]["title"], "Engram tagged current plan");
    assert_eq!(list_json["items"][0]["scope"]["project_name"], "engram");
}

#[tokio::test]
async fn test_mcp_memory_list_project_name_current_plan_tags_preserves_limit() {
    let state = setup_tool_state().await;

    for index in 0..6 {
        let mut matching = with_writer(request("add"));
        matching.kind = Some("decision".to_string());
        matching.title = Some(format!("Engram tagged current plan {index}"));
        matching.content =
            Some("This Engram current-plan item should remain eligible.".to_string());
        matching.origin = Some("tool_result".to_string());
        matching.scope_type = Some("project".to_string());
        matching.project_name = Some("engram".to_string());
        matching.tags = vec!["current-plan".to_string()];
        matching.evidence =
            manual_review_evidence("Reviewed project-name tag limit matching fixture.");
        tools::memory_new(&state, matching)
            .await
            .expect("matching add should work");
    }

    tokio::time::sleep(std::time::Duration::from_millis(1)).await;

    let mut wrong_scope = with_writer(request("add"));
    wrong_scope.kind = Some("decision".to_string());
    wrong_scope.title = Some("Voice layer tagged current plan".to_string());
    wrong_scope.content =
        Some("This newer wrong-project tagged item should not be returned.".to_string());
    wrong_scope.origin = Some("tool_result".to_string());
    wrong_scope.scope_type = Some("project".to_string());
    wrong_scope.project_name = Some("voice-layer".to_string());
    wrong_scope.tags = vec!["current-plan".to_string()];
    wrong_scope.evidence =
        manual_review_evidence("Reviewed project-name tag limit wrong-project fixture.");
    tools::memory_new(&state, wrong_scope)
        .await
        .expect("wrong-scope add should work");

    tokio::time::sleep(std::time::Duration::from_millis(1)).await;

    let mut untagged = with_writer(request("add"));
    untagged.kind = Some("project_fact".to_string());
    untagged.title = Some("Engram untagged project fact".to_string());
    untagged.content =
        Some("This same-project item should not satisfy the tag filter.".to_string());
    untagged.origin = Some("tool_result".to_string());
    untagged.scope_type = Some("project".to_string());
    untagged.project_name = Some("engram".to_string());
    untagged.evidence = manual_review_evidence("Reviewed project-name tag limit untagged fixture.");
    tools::memory_new(&state, untagged)
        .await
        .expect("untagged add should work");

    let mut list = request("list");
    list.status_filter = Some("active".to_string());
    list.project_name = Some("engram".to_string());
    list.tags = vec!["current-plan".to_string()];
    list.limit = Some(5);
    let list_response = tools::memory_new(&state, list)
        .await
        .expect("list should work");
    let list_json = parse_json(&list_response);

    assert_eq!(list_json["count"], 5);
    for item in list_json["items"]
        .as_array()
        .expect("items should be an array")
    {
        assert_eq!(item["scope"]["project_name"], "engram");
        assert!(item["tags"]
            .as_array()
            .expect("tags should be an array")
            .iter()
            .any(|tag| tag == "current-plan"));
    }
}

#[tokio::test]
async fn test_mcp_memory_list_project_name_scope_inference_preserves_limit() {
    let state = setup_tool_state().await;

    let mut older_matching = with_writer(request("add"));
    older_matching.kind = Some("decision".to_string());
    older_matching.title = Some("Older Engram project-only item".to_string());
    older_matching.content =
        Some("This older matching item should be eligible after inferred scope.".to_string());
    older_matching.origin = Some("tool_result".to_string());
    older_matching.scope_type = Some("project".to_string());
    older_matching.project_name = Some("engram".to_string());
    older_matching.evidence = manual_review_evidence("Reviewed older project-only limit fixture.");
    tools::memory_new(&state, older_matching)
        .await
        .expect("older matching add should work");

    tokio::time::sleep(std::time::Duration::from_millis(1)).await;

    let mut newer_matching = with_writer(request("add"));
    newer_matching.kind = Some("decision".to_string());
    newer_matching.title = Some("Newer Engram project-only item".to_string());
    newer_matching.content =
        Some("This newer matching item should be eligible after inferred scope.".to_string());
    newer_matching.origin = Some("tool_result".to_string());
    newer_matching.scope_type = Some("project".to_string());
    newer_matching.project_name = Some("engram".to_string());
    newer_matching.evidence = manual_review_evidence("Reviewed newer project-only limit fixture.");
    tools::memory_new(&state, newer_matching)
        .await
        .expect("newer matching add should work");

    tokio::time::sleep(std::time::Duration::from_millis(1)).await;

    let mut wrong_scope = with_writer(request("add"));
    wrong_scope.kind = Some("decision".to_string());
    wrong_scope.title = Some("DD source project-only item".to_string());
    wrong_scope.content = Some("This newer wrong-project item should not be returned.".to_string());
    wrong_scope.origin = Some("tool_result".to_string());
    wrong_scope.scope_type = Some("project".to_string());
    wrong_scope.project_name = Some("dd-source".to_string());
    wrong_scope.evidence = manual_review_evidence("Reviewed wrong-scope project-only fixture.");
    tools::memory_new(&state, wrong_scope)
        .await
        .expect("wrong-scope add should work");

    let mut list = request("list");
    list.status_filter = Some("active".to_string());
    list.project_name = Some("engram".to_string());
    list.limit = Some(1);
    let list_response = tools::memory_new(&state, list)
        .await
        .expect("list should work");
    let list_json = parse_json(&list_response);

    assert_eq!(list_json["count"], 1);
    assert_eq!(list_json["items"][0]["scope"]["project_name"], "engram");
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
            scope: None,
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
            scope: None,
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

    let orientation = tools::orient(
        &state,
        OrientRequest {
            cwd: None,
            prompt: Some("Which workflow decision should guide future agents?".to_string()),
            project: Some("engram".to_string()),
            task: None,
            agent: Some("codex".to_string()),
            external_session_id: None,
            intent: Some("verify_decision".to_string()),
            scenario_id: Some("correction_projection_postcondition".to_string()),
            arm: Some("engram".to_string()),
            include_recent_commits: Some(false),
            limit: Some(10),
            response_shape: None,
        },
    )
    .await
    .expect("post-correction orientation should work");
    assert!(orientation.contains("Replacement decision"));
    assert!(orientation.contains("The replacement workflow should guide future agents."));
    assert!(!orientation.contains("The old workflow should be replaced."));

    let search = tools::search(
        &state,
        SearchRequest {
            query: "workflow decision future agents".to_string(),
            limit: 10,
            min_score: Some(0.0),
            layers: Some(vec!["memory".to_string()]),
            intent: Some("verify_decision".to_string()),
            scenario_id: Some("correction_projection_postcondition".to_string()),
            arm: Some("engram".to_string()),
            agent: Some("codex".to_string()),
            session_id: None,
            external_session_id: None,
            project: Some("engram".to_string()),
            task: None,
            cwd: None,
            relevance_mode: Some("local".to_string()),
        },
    )
    .await
    .expect("post-correction unified search should work");
    assert!(search.contains("Replacement decision"));
    assert!(search.contains("The replacement workflow should guide future agents."));
    assert!(!search.contains("The old workflow should be replaced."));
}

#[tokio::test]
async fn test_mcp_memory_proposal_preserves_retrieval_until_operator_apply() {
    let state = setup_tool_state().await;

    let mut old = with_writer(request("add"));
    old.kind = Some("decision".to_string());
    old.title = Some("Obsolete proposal target".to_string());
    old.content = Some("Use the obsolete proposal workflow.".to_string());
    old.origin = Some("user_stated".to_string());
    old.scope_type = Some("project".to_string());
    old.project_name = Some("engram".to_string());
    old.evidence = vec![MemoryEvidenceRequest {
        kind: "file".to_string(),
        target: "docs/obsolete-proposal.md".to_string(),
        summary: None,
        excerpt: None,
    }];
    let old_id = parse_json(&tools::memory_new(&state, old).await.unwrap())["item"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let mut propose = with_writer(request("propose_correction"));
    propose.scope = local_project_scope("engram");
    propose.id = Some(old_id.clone());
    propose.title = Some("Proposed current workflow".to_string());
    propose.content = Some("Use the proposed current workflow.".to_string());
    propose.evidence = vec![MemoryEvidenceRequest {
        kind: "session_event".to_string(),
        target: "codex://threads/proposal-fixture".to_string(),
        summary: Some("Agent-observed correction proposal.".to_string()),
        excerpt: None,
    }];
    let proposed = parse_json(
        &tools::memory_new(&state, propose)
            .await
            .expect("bounded proposal should work"),
    );
    let proposal_id = proposed["proposal_id"].as_str().unwrap().to_string();
    let replacement_id = proposed["replacement_id"].as_str().unwrap().to_string();
    let digest = proposed["canonical_digest"].as_str().unwrap().to_string();
    assert_eq!(digest.len(), 64);
    assert_eq!(proposed["proposal"]["status"], "pending");
    assert_eq!(proposed["replacement"]["status"], "needs_review");
    assert_eq!(proposed["replacement"]["origin"], "agent_inferred");
    assert_eq!(proposed["replacement"]["kind"], "decision");
    assert_eq!(proposed["replacement"]["scope"]["project_name"], "engram");
    assert_eq!(proposed["digest_phase"], "p0");
    assert_eq!(proposed["activated"], false);
    assert_eq!(proposed["human_identity_authenticated"], false);
    assert_eq!(proposed["intent_verified"], false);
    assert_eq!(proposed["human_review_verified"], false);
    assert_eq!(proposed["reviewer_authority_conferred"], false);
    assert_eq!(proposed["scope_selector_match_enforced"], true);
    assert_eq!(proposed["scope_selector_authenticated"], false);

    let mut list = request("list");
    list.scope = local_project_scope("engram");
    list.status_filter = Some("active".to_string());
    let active_before_apply = tools::memory_new(&state, list.clone()).await.unwrap();
    assert!(active_before_apply.contains("Use the obsolete proposal workflow."));
    assert!(!active_before_apply.contains("Use the proposed current workflow."));

    let mut unfiltered_list = request("list");
    unfiltered_list.scope = local_project_scope("engram");
    let unfiltered_before_apply = tools::memory_new(&state, unfiltered_list).await.unwrap();
    assert!(unfiltered_before_apply.contains("Use the obsolete proposal workflow."));
    assert!(!unfiltered_before_apply.contains("Use the proposed current workflow."));

    let orient_before_apply = tools::orient(
        &state,
        OrientRequest {
            cwd: None,
            prompt: Some("Which proposal workflow is active?".to_string()),
            project: Some("engram".to_string()),
            task: None,
            agent: Some("codex".to_string()),
            external_session_id: None,
            intent: Some("verify_proposal_boundary".to_string()),
            scenario_id: Some("correction_proposal".to_string()),
            arm: Some("engram".to_string()),
            include_recent_commits: Some(false),
            limit: Some(10),
            response_shape: None,
        },
    )
    .await
    .unwrap();
    assert!(orient_before_apply.contains("Use the obsolete proposal workflow."));
    assert!(!orient_before_apply.contains("Use the proposed current workflow."));

    let search_before_apply = tools::search(
        &state,
        SearchRequest {
            query: "proposal workflow".to_string(),
            limit: 10,
            min_score: Some(0.0),
            layers: Some(vec!["memory".to_string()]),
            intent: Some("verify_proposal_boundary".to_string()),
            scenario_id: Some("correction_proposal".to_string()),
            arm: Some("engram".to_string()),
            agent: Some("codex".to_string()),
            session_id: None,
            external_session_id: None,
            project: Some("engram".to_string()),
            task: None,
            cwd: None,
            relevance_mode: Some("local".to_string()),
        },
    )
    .await
    .unwrap();
    assert!(search_before_apply.contains("Use the obsolete proposal workflow."));
    assert!(!search_before_apply.contains("Use the proposed current workflow."));

    let mut apply = request("apply_correction");
    apply.scope = local_project_scope("engram");
    apply.proposal_id = Some(proposal_id.clone());
    apply.expected_digest = Some(digest.clone());
    let applied = parse_json(
        &tools::memory_new(&state, apply.clone())
            .await
            .expect("operator-selected proposal should apply"),
    );
    assert_eq!(applied["proposal"]["id"], proposal_id);
    assert_eq!(applied["proposal"]["status"], "applied");
    assert_eq!(applied["item"]["id"], replacement_id);
    assert_eq!(applied["item"]["status"], "active");
    assert_eq!(applied["corrected_item"]["id"], old_id);
    assert_eq!(applied["corrected_item"]["status"], "superseded");
    assert_eq!(applied["authority_boundary"], "operator_selected");
    assert_eq!(applied["human_identity_authenticated"], false);
    assert_eq!(applied["reviewer_authority_conferred"], false);
    assert_eq!(applied["scope_selector_match_enforced"], true);
    assert_eq!(applied["scope_selector_authenticated"], false);
    assert!(applied["item"]["evidence"]
        .as_array()
        .unwrap()
        .iter()
        .all(|evidence| evidence["kind"] != "manual_review"));

    let evidence_count = applied["item"]["evidence"].as_array().unwrap().len();
    let retried = parse_json(&tools::memory_new(&state, apply).await.unwrap());
    assert_eq!(
        retried["item"]["evidence"].as_array().unwrap().len(),
        evidence_count
    );

    let active_after_apply = tools::memory_new(&state, list).await.unwrap();
    assert!(active_after_apply.contains("Use the proposed current workflow."));
    assert!(!active_after_apply.contains("Use the obsolete proposal workflow."));

    let orient_after_apply = tools::orient(
        &state,
        OrientRequest {
            cwd: None,
            prompt: Some("Which proposal workflow is active?".to_string()),
            project: Some("engram".to_string()),
            task: None,
            agent: Some("codex".to_string()),
            external_session_id: None,
            intent: Some("verify_proposal_apply".to_string()),
            scenario_id: Some("correction_proposal".to_string()),
            arm: Some("engram".to_string()),
            include_recent_commits: Some(false),
            limit: Some(10),
            response_shape: None,
        },
    )
    .await
    .unwrap();
    assert!(orient_after_apply.contains("Use the proposed current workflow."));
    assert!(!orient_after_apply.contains("Use the obsolete proposal workflow."));

    let search_after_apply = tools::search(
        &state,
        SearchRequest {
            query: "proposal workflow".to_string(),
            limit: 10,
            min_score: Some(0.0),
            layers: Some(vec!["memory".to_string()]),
            intent: Some("verify_proposal_apply".to_string()),
            scenario_id: Some("correction_proposal".to_string()),
            arm: Some("engram".to_string()),
            agent: Some("codex".to_string()),
            session_id: None,
            external_session_id: None,
            project: Some("engram".to_string()),
            task: None,
            cwd: None,
            relevance_mode: Some("local".to_string()),
        },
    )
    .await
    .unwrap();
    assert!(search_after_apply.contains("Use the proposed current workflow."));
    assert!(!search_after_apply.contains("Use the obsolete proposal workflow."));
}

#[tokio::test]
async fn test_mcp_procedure_correction_requires_inactive_verification_before_apply() {
    let state = setup_tool_state().await;
    let dir = tempdir().unwrap();
    let obsolete_receipt = dir.path().join("obsolete-procedure-receipt.json");
    let obsolete_receipt_body = ProcedureVerificationReceipt {
        command: "cargo test obsolete_procedure".to_string(),
        exit_code: 0,
        output: "OBSOLETE_PROCEDURE_OK".to_string(),
        conditions: std::collections::BTreeMap::new(),
    };
    fs::write(
        &obsolete_receipt,
        serde_json::to_vec_pretty(&obsolete_receipt_body).unwrap(),
    )
    .unwrap();

    let mut old = with_writer(request("add"));
    old.kind = Some("procedure".to_string());
    old.title = Some("Obsolete procedure correction target".to_string());
    old.content = Some("Run the obsolete procedure.".to_string());
    old.origin = Some("agent_observed".to_string());
    old.scope_type = Some("project".to_string());
    old.project_name = Some("engram".to_string());
    old.evidence = vec![MemoryEvidenceRequest {
        kind: "file".to_string(),
        target: "docs/obsolete-procedure.md".to_string(),
        summary: None,
        excerpt: None,
    }];
    old.procedure = Some(MemoryProcedureRequest {
        task: "run obsolete procedure".to_string(),
        commands: vec!["cargo test obsolete_procedure".to_string()],
        prerequisites: std::collections::BTreeMap::new(),
        prerequisite_sources: std::collections::BTreeMap::new(),
        failure_signatures: Vec::new(),
        verification_command: "cargo test obsolete_procedure".to_string(),
        verification_exit_code: 0,
        verification_output_contains: "OBSOLETE_PROCEDURE_OK".to_string(),
    });
    let old_id = parse_json(&tools::memory_new(&state, old).await.unwrap())["item"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let old_id_parsed = Id::parse(&old_id).unwrap();
    {
        let guard = state.memory_service.read().await;
        guard
            .as_ref()
            .unwrap()
            .verify_procedure(
                &old_id_parsed,
                &obsolete_receipt,
                Some(time::OffsetDateTime::now_utc() + time::Duration::days(30)),
            )
            .await
            .unwrap();
    }

    let replacement_receipt = dir.path().join("replacement-procedure-receipt.json");
    let replacement_receipt_body = ProcedureVerificationReceipt {
        command: "cargo test corrected_procedure".to_string(),
        exit_code: 0,
        output: "CORRECTED_PROCEDURE_OK".to_string(),
        conditions: std::collections::BTreeMap::new(),
    };
    fs::write(
        &replacement_receipt,
        serde_json::to_vec_pretty(&replacement_receipt_body).unwrap(),
    )
    .unwrap();

    let mut propose = with_writer(request("propose_correction"));
    propose.scope = local_project_scope("engram");
    propose.id = Some(old_id.clone());
    propose.title = Some("Corrected procedure".to_string());
    propose.content = Some("Run the corrected procedure.".to_string());
    propose.evidence = vec![MemoryEvidenceRequest {
        kind: "session_event".to_string(),
        target: "codex://threads/procedure-correction".to_string(),
        summary: None,
        excerpt: None,
    }];
    propose.procedure = Some(MemoryProcedureRequest {
        task: "run corrected procedure".to_string(),
        commands: vec!["cargo test corrected_procedure".to_string()],
        prerequisites: std::collections::BTreeMap::new(),
        prerequisite_sources: std::collections::BTreeMap::new(),
        failure_signatures: vec!["obsolete invocation".to_string()],
        verification_command: "cargo test corrected_procedure".to_string(),
        verification_exit_code: 0,
        verification_output_contains: "CORRECTED_PROCEDURE_OK".to_string(),
    });
    let proposed = parse_json(&tools::memory_new(&state, propose).await.unwrap());
    let proposal_id = proposed["proposal_id"].as_str().unwrap().to_string();
    let replacement_id = proposed["replacement_id"].as_str().unwrap().to_string();
    let p0 = proposed["canonical_digest"].as_str().unwrap().to_string();
    assert_eq!(proposed["replacement"]["status"], "needs_review");
    assert_eq!(proposed["obsolete_id"], old_id);

    let mut premature_apply = request("apply_correction");
    premature_apply.scope = local_project_scope("engram");
    premature_apply.proposal_id = Some(proposal_id.clone());
    premature_apply.expected_digest = Some(p0.clone());
    let error = tools::memory_new(&state, premature_apply)
        .await
        .expect_err("unverified replacement must not activate");
    assert!(error.contains("verification expiry"));

    let mut verify = request("verify_correction_procedure");
    verify.scope = local_project_scope("engram");
    verify.proposal_id = Some(proposal_id.clone());
    verify.expected_digest = Some(p0.clone());
    verify.receipt = Some(replacement_receipt.display().to_string());
    verify.expires_at = Some(
        (time::OffsetDateTime::now_utc() + time::Duration::days(30))
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap(),
    );
    let verified = parse_json(&tools::memory_new(&state, verify).await.unwrap());
    let p1 = verified["p1_digest"].as_str().unwrap().to_string();
    assert_ne!(p0, p1);
    assert_eq!(verified["p0_digest"], p0);
    assert_eq!(verified["proposal"]["status"], "pending");
    assert_eq!(verified["replacement"]["status"], "needs_review");
    assert_eq!(verified["obsolete"]["status"], "active");
    assert_eq!(verified["activated"], false);
    assert_eq!(verified["human_identity_authenticated"], false);
    assert_eq!(verified["intent_verified"], false);
    assert_eq!(verified["human_review_verified"], false);
    assert_eq!(verified["reviewer_authority_conferred"], false);

    let mut list = request("list");
    list.scope = local_project_scope("engram");
    list.status_filter = Some("active".to_string());
    let active_before_apply = tools::memory_new(&state, list).await.unwrap();
    assert!(active_before_apply.contains("Run the obsolete procedure."));
    assert!(!active_before_apply.contains("Run the corrected procedure."));

    let mut apply = request("apply_correction");
    apply.scope = local_project_scope("engram");
    apply.proposal_id = Some(proposal_id);
    apply.expected_digest = Some(p1);
    let applied = parse_json(&tools::memory_new(&state, apply).await.unwrap());
    assert_eq!(applied["proposal"]["status"], "applied");
    assert_eq!(applied["item"]["id"], replacement_id);
    assert_eq!(applied["item"]["status"], "active");
    assert_eq!(applied["corrected_item"]["id"], old_id);
    assert_eq!(applied["corrected_item"]["status"], "superseded");
}

#[tokio::test]
async fn test_mcp_memory_correct_links_unreviewed_user_correction() {
    let state = setup_tool_state().await;

    let mut old = with_writer(request("add"));
    old.kind = Some("decision".to_string());
    old.title = Some("Obsolete correction target".to_string());
    old.content = Some("Use the obsolete correction workflow.".to_string());
    old.origin = Some("user_stated".to_string());
    old.scope_type = Some("project".to_string());
    old.project_name = Some("engram".to_string());
    old.evidence = vec![MemoryEvidenceRequest {
        kind: "file".to_string(),
        target: "docs/obsolete-workflow.md".to_string(),
        summary: Some("Original source for the obsolete workflow.".to_string()),
        excerpt: None,
    }];
    let old_response = tools::memory_new(&state, old)
        .await
        .expect("old add should work");
    let old_id = parse_json(&old_response)["item"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let mut replacement = with_writer(request("add"));
    replacement.kind = Some("decision".to_string());
    replacement.title = Some("Current correction replacement".to_string());
    replacement.content = Some("Use the current corrected workflow.".to_string());
    replacement.origin = Some("user_corrected".to_string());
    replacement.scope_type = Some("project".to_string());
    replacement.project_name = Some("engram".to_string());
    replacement.evidence = vec![MemoryEvidenceRequest {
        kind: "session_event".to_string(),
        target: "codex://threads/correction-fixture".to_string(),
        summary: Some("The current user supplied corrected guidance.".to_string()),
        excerpt: None,
    }];
    let replacement_response = tools::memory_new(&state, replacement)
        .await
        .expect("replacement add should work");
    let replacement_id = parse_json(&replacement_response)["item"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let mut unconfirmed = request("correct");
    unconfirmed.scope = local_project_scope("engram");
    unconfirmed.id = Some(old_id.clone());
    unconfirmed.replacement_id = Some(replacement_id.clone());
    unconfirmed.correction_reason = Some("The user replaced the obsolete workflow.".to_string());
    let error = tools::memory_new(&state, unconfirmed)
        .await
        .expect_err("unconfirmed correction must fail");
    assert!(error.contains("confirm_correction=true"));

    let mut global = request("correct");
    global.id = Some(old_id.clone());
    global.replacement_id = Some(replacement_id.clone());
    global.correction_reason = Some("Global correction must remain forbidden.".to_string());
    global.confirm_correction = Some(true);
    let error = tools::memory_new(&state, global)
        .await
        .expect_err("global correction must fail");
    assert!(error.contains("forbids relevance_mode=global"));

    let mut wrong_scope = request("correct");
    wrong_scope.scope = local_project_scope("orbit");
    wrong_scope.id = Some(old_id.clone());
    wrong_scope.replacement_id = Some(replacement_id.clone());
    wrong_scope.correction_reason = Some("Attempt correction from another project.".to_string());
    wrong_scope.confirm_correction = Some(true);
    let error = tools::memory_new(&state, wrong_scope)
        .await
        .expect_err("cross-project correction must fail");
    assert!(error.contains("outside the resolved correction authorization boundary"));

    let mut correct = request("correct");
    correct.scope = local_project_scope("engram");
    correct.id = Some(old_id.clone());
    correct.replacement_id = Some(replacement_id.clone());
    correct.correction_reason = Some("The user replaced the obsolete workflow.".to_string());
    correct.confirm_correction = Some(true);
    let corrected = tools::memory_new(&state, correct.clone())
        .await
        .expect("confirmed correction should work");
    let corrected_json = parse_json(&corrected);
    assert_eq!(corrected_json["item"]["id"], replacement_id);
    assert_eq!(corrected_json["item"]["status"], "active");
    assert_eq!(corrected_json["item"]["supersedes"][0], old_id);
    assert_eq!(corrected_json["corrected_item"]["id"], old_id);
    assert_eq!(corrected_json["corrected_item"]["status"], "superseded");
    assert_eq!(corrected_json["reviewer_authority_conferred"], false);
    assert_eq!(corrected_json["scope_selector_match_enforced"], true);
    assert_eq!(corrected_json["scope_selector_authenticated"], false);
    assert!(corrected_json["item"]["evidence"]
        .as_array()
        .unwrap()
        .iter()
        .all(|evidence| evidence["kind"] != "manual_review"));

    let evidence_count = corrected_json["item"]["evidence"].as_array().unwrap().len();
    let repeated = tools::memory_new(&state, correct.clone())
        .await
        .expect("exact correction retry should be idempotent");
    assert_eq!(
        parse_json(&repeated)["item"]["evidence"]
            .as_array()
            .unwrap()
            .len(),
        evidence_count
    );

    correct.scope = local_project_scope("orbit");
    let error = tools::memory_new(&state, correct)
        .await
        .expect_err("wrong-scope idempotent correction retry must fail");
    assert!(error.contains("outside the resolved correction authorization boundary"));

    let mut get_replacement = request("get");
    get_replacement.id = Some(replacement_id.clone());
    let replacement_after_rejected_retry = parse_json(
        &tools::memory_new(&state, get_replacement)
            .await
            .expect("replacement should remain readable"),
    );
    assert_eq!(
        replacement_after_rejected_retry["item"]["evidence"]
            .as_array()
            .unwrap()
            .len(),
        evidence_count
    );

    let mut list = request("list");
    list.status_filter = Some("active".to_string());
    let list = tools::memory_new(&state, list)
        .await
        .expect("active list should work");
    assert!(list.contains("Use the current corrected workflow."));
    assert!(!list.contains("Use the obsolete correction workflow."));

    let orientation = tools::orient(
        &state,
        OrientRequest {
            cwd: None,
            prompt: Some("Which correction workflow is current?".to_string()),
            project: Some("engram".to_string()),
            task: None,
            agent: Some("codex".to_string()),
            external_session_id: None,
            intent: Some("verify_decision".to_string()),
            scenario_id: Some("agent_correction_postcondition".to_string()),
            arm: Some("engram".to_string()),
            include_recent_commits: Some(false),
            limit: Some(10),
            response_shape: None,
        },
    )
    .await
    .expect("post-correction orientation should work");
    assert!(orientation.contains("Use the current corrected workflow."));
    assert!(!orientation.contains("Use the obsolete correction workflow."));

    let search = tools::search(
        &state,
        SearchRequest {
            query: "correction workflow current".to_string(),
            limit: 10,
            min_score: Some(0.0),
            layers: Some(vec!["memory".to_string()]),
            intent: Some("verify_decision".to_string()),
            scenario_id: Some("agent_correction_postcondition".to_string()),
            arm: Some("engram".to_string()),
            agent: Some("codex".to_string()),
            session_id: None,
            external_session_id: None,
            project: Some("engram".to_string()),
            task: None,
            cwd: None,
            relevance_mode: Some("local".to_string()),
        },
    )
    .await
    .expect("post-correction search should work");
    assert!(search.contains("Use the current corrected workflow."));
    assert!(!search.contains("Use the obsolete correction workflow."));
}

#[tokio::test]
async fn test_mcp_memory_correction_proposal_persistence_subprocess_worker() {
    let Ok(mode) = std::env::var("ENGRAM_CORRECTION_PERSISTENCE_WORKER_MODE") else {
        return;
    };
    let root = std::path::PathBuf::from(
        std::env::var("ENGRAM_CORRECTION_PERSISTENCE_ROOT")
            .expect("worker root should be provided"),
    );
    let config = StoreConfig::rocksdb(root.join("data"));
    let db = connect_and_init(&config).await.unwrap();
    let state = ToolState::new();
    state.init_memory(MemoryService::new(db)).await;

    if mode == "verify" {
        assert!(!root.join("correction-ids.json").exists());
        let mut list_proposals = request("list_correction_proposals");
        list_proposals.scope = local_project_scope("engram");
        let discovered = parse_json(
            &tools::memory_new(&state, list_proposals)
                .await
                .expect("full profile should rediscover a pending proposal after restart"),
        );
        assert_eq!(discovered["count"], 1);
        let inspection = &discovered["proposals"][0];
        let proposal_id = inspection["proposal"]["id"].as_str().unwrap();
        let old_id = inspection["proposal"]["obsolete_id"].as_str().unwrap();
        let replacement_id = inspection["proposal"]["replacement_id"].as_str().unwrap();
        let digest = inspection["proposal"]["canonical_digest"].as_str().unwrap();
        assert_eq!(inspection["proposal"]["status"], "pending");
        assert_eq!(inspection["replacement"]["status"], "needs_review");
        assert_eq!(inspection["obsolete"]["status"], "active");
        assert_eq!(digest.len(), 64);

        let mut inspect = request("get_correction_proposal");
        inspect.scope = local_project_scope("engram");
        inspect.proposal_id = Some(proposal_id.to_string());
        let exact = parse_json(
            &tools::memory_new(&state, inspect)
                .await
                .expect("full profile should inspect the rediscovered exact proposal"),
        );
        assert_eq!(exact["proposal"]["id"], proposal_id);
        assert_eq!(exact["proposal"]["canonical_digest"], digest);
        assert_eq!(exact["replacement"]["id"], replacement_id);
        assert_eq!(exact["obsolete"]["id"], old_id);

        let mut get_old = request("get");
        get_old.id = Some(old_id.to_string());
        let old = tools::memory_new(&state, get_old)
            .await
            .expect("obsolete item should remain active before recovered apply");
        assert_eq!(parse_json(&old)["item"]["status"], "active");

        let mut get_replacement = request("get");
        get_replacement.id = Some(replacement_id.to_string());
        let replacement = tools::memory_new(&state, get_replacement)
            .await
            .expect("replacement should survive process restart");
        let replacement = parse_json(&replacement);
        assert_eq!(replacement["item"]["status"], "needs_review");

        let mut apply = request("apply_correction");
        apply.scope = local_project_scope("engram");
        apply.proposal_id = Some(proposal_id.to_string());
        apply.expected_digest = Some(digest.to_string());
        let applied = parse_json(
            &tools::memory_new(&state, apply)
                .await
                .expect("rediscovered exact proposal should apply after restart"),
        );
        assert_eq!(applied["proposal"]["status"], "applied");
        assert_eq!(applied["item"]["id"], replacement_id);
        assert_eq!(applied["item"]["status"], "active");
        assert_eq!(applied["corrected_item"]["id"], old_id);
        assert_eq!(applied["corrected_item"]["status"], "superseded");
        assert!(applied["item"]["evidence"]
            .as_array()
            .unwrap()
            .iter()
            .all(|evidence| evidence["kind"] != "manual_review"));

        let mut list = request("list");
        list.status_filter = Some("active".to_string());
        let list = tools::memory_new(&state, list)
            .await
            .expect("active list should work after process restart");
        assert!(list.contains("Persist the current correction canary."));
        assert!(!list.contains("Persist the obsolete correction canary."));

        return;
    }
    assert_eq!(mode, "seed");

    let mut old = with_writer(request("add"));
    old.kind = Some("decision".to_string());
    old.title = Some("Persistent obsolete decision".to_string());
    old.content = Some("Persist the obsolete correction canary.".to_string());
    old.origin = Some("user_stated".to_string());
    old.scope_type = Some("project".to_string());
    old.project_name = Some("engram".to_string());
    old.evidence = vec![MemoryEvidenceRequest {
        kind: "file".to_string(),
        target: "docs/persistent-old.md".to_string(),
        summary: None,
        excerpt: None,
    }];
    let old_id = parse_json(&tools::memory_new(&state, old).await.unwrap())["item"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let mut proposal = with_writer(request("propose_correction"));
    proposal.scope = local_project_scope("engram");
    proposal.id = Some(old_id.clone());
    proposal.title = Some("Persistent corrected decision".to_string());
    proposal.content = Some("Persist the current correction canary.".to_string());
    proposal.evidence = vec![MemoryEvidenceRequest {
        kind: "session_event".to_string(),
        target: "codex://threads/persistent-correction".to_string(),
        summary: None,
        excerpt: None,
    }];
    let proposed = parse_json(&tools::memory_new(&state, proposal).await.unwrap());
    assert_eq!(proposed["proposal"]["status"], "pending");
    assert_eq!(proposed["replacement"]["status"], "needs_review");
    assert_eq!(proposed["obsolete_id"], old_id);
    assert!(root.join("data/CURRENT").is_file());
    assert!(!root.join("correction-ids.json").exists());
}

#[test]
fn test_mcp_memory_correction_proposal_survives_persistent_process_restart() {
    let data = tempdir().expect("persistent correction directory should be created");
    let executable = std::env::current_exe().expect("test executable should resolve");
    for mode in ["seed", "verify"] {
        let status = Command::new(&executable)
            .arg("--exact")
            .arg("test_mcp_memory_correction_proposal_persistence_subprocess_worker")
            .arg("--nocapture")
            .env("ENGRAM_CORRECTION_PERSISTENCE_WORKER_MODE", mode)
            .env("ENGRAM_CORRECTION_PERSISTENCE_ROOT", data.path())
            .status()
            .expect("persistence worker should start");
        assert!(status.success(), "{mode} worker should pass");
    }
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
async fn test_mcp_memory_commit_rejects_secret_material() {
    let state = setup_tool_state().await;
    let canary = "Authorization: Bearer synthetic-mcp-commit-secret";

    let mut commit = with_writer(request("commit"));
    commit.message = Some("Attempt a secret-bearing knowledge commit".to_string());
    commit.changes = vec![MemoryChangeRequest {
        change_type: "added".to_string(),
        item_id: None,
        title: "Secret persistence canary".to_string(),
        summary: canary.to_string(),
        before_hash: None,
        after_hash: None,
    }];

    let error = tools::memory_new(&state, commit)
        .await
        .expect_err("secret-bearing knowledge commits should be rejected");

    assert!(error.contains("knowledge commit"));
    assert!(error.contains("secret material was not persisted"));

    let log = tools::memory_new(&state, request("log"))
        .await
        .expect("memory log should remain readable");
    assert!(!log.contains(canary));
}

#[tokio::test]
async fn test_mcp_flagship_memory_writes_reject_secret_canaries_without_persistence() {
    let dir = tempdir().expect("persistent test directory should be created");
    let config = StoreConfig::rocksdb(dir.path().join("data"));
    let db = connect_and_init(&config)
        .await
        .expect("persistent store should initialize");
    let memory = MemoryService::new(db);
    let state = ToolState::new();
    state.init_memory(memory).await;
    let bearer_canary = "Authorization: Bearer synthetic-mcp-memory-secret";
    let assignment_canary = "API_TOKEN=synthetic-mcp-procedure-secret";

    let mut add = with_writer(request("add"));
    add.kind = Some("decision".to_string());
    add.title = Some("Secret-bearing decision".to_string());
    add.content = Some(bearer_canary.to_string());
    add.origin = Some("tool_result".to_string());
    add.scope_type = Some("project".to_string());
    add.project_name = Some("engram".to_string());
    add.evidence = manual_review_evidence("Safe review evidence.");
    let error = tools::memory_new(&state, add)
        .await
        .expect_err("secret-bearing memory add should be rejected");
    assert!(error.contains("secret material was not persisted"));

    let mut procedure = with_writer(request("add"));
    procedure.kind = Some("procedure".to_string());
    procedure.title = Some("Secret-bearing procedure".to_string());
    procedure.content = Some("This candidate must not reach durable storage.".to_string());
    procedure.origin = Some("agent_observed".to_string());
    procedure.scope_type = Some("project".to_string());
    procedure.project_name = Some("engram".to_string());
    procedure.procedure = Some(MemoryProcedureRequest {
        task: "run the integration suite".to_string(),
        commands: vec![format!("{assignment_canary} cargo test")],
        prerequisites: std::collections::BTreeMap::new(),
        prerequisite_sources: std::collections::BTreeMap::new(),
        failure_signatures: Vec::new(),
        verification_command: "cargo test".to_string(),
        verification_exit_code: 0,
        verification_output_contains: "test result: ok".to_string(),
    });
    let error = tools::memory_new(&state, procedure)
        .await
        .expect_err("secret-bearing procedure add should be rejected");
    assert!(error.contains("secret material was not persisted"));

    let mut current_plan = with_writer(request("capture_current_plan"));
    current_plan.kind = Some("decision".to_string());
    current_plan.title = Some("Secret-bearing current plan".to_string());
    current_plan.content = Some(bearer_canary.to_string());
    current_plan.origin = Some("tool_result".to_string());
    current_plan.project_name = Some("engram".to_string());
    current_plan.message = Some("Capture a secret-bearing plan".to_string());
    current_plan.evidence = manual_review_evidence("Safe current-plan evidence.");
    current_plan.create_commit = Some(true);
    let error = tools::memory_new(&state, current_plan)
        .await
        .expect_err("secret-bearing current plan should be rejected");
    assert!(error.contains("secret material was not persisted"));

    assert!(
        dir.path().join("data/CURRENT").is_file(),
        "test must exercise a real RocksDB store"
    );

    let mut list = request("list");
    list.project_name = Some("engram".to_string());
    let persisted_items = tools::memory_new(&state, list)
        .await
        .expect("memory list should remain readable");
    let persisted_commits = tools::memory_new(&state, request("log"))
        .await
        .expect("memory log should remain readable");

    assert_eq!(parse_json(&persisted_items)["count"], 0);
    assert_eq!(parse_json(&persisted_commits)["count"], 0);
    assert!(!persisted_items.contains(bearer_canary));
    assert!(!persisted_items.contains(assignment_canary));
    assert!(!persisted_commits.contains(bearer_canary));
    assert!(!persisted_commits.contains(assignment_canary));
}

#[tokio::test]
async fn test_mcp_memory_changes_since_commit_id_error_names_cursor_timestamp() {
    let state = setup_tool_state().await;

    let mut changes = request("changes_since");
    changes.commit_id = Some("019e8304-b42e-7170-811f-b1a211ce18b8".to_string());

    let error = tools::memory_new(&state, changes)
        .await
        .expect_err("changes_since should require cursor timestamp");

    assert!(error.contains("timestamp required for changes_since"));
    assert!(error.contains("commit_id was provided"));
    assert!(error.contains("memory_cursor.timestamp"));
    assert!(error.contains("memory_cursor.commit_id"));
}

#[tokio::test]
async fn test_mcp_memory_capture_current_plan_commits_and_orients() {
    let state = setup_tool_state().await;

    let mut capture = with_writer(request("capture_current_plan"));
    capture.kind = Some("decision".to_string());
    capture.title = Some("Current Brain Harness plan".to_string());
    capture.content = Some(
        "Use compact active MemoryItems for current method, plan, and next action.".to_string(),
    );
    capture.project_name = Some("engram".to_string());
    capture.origin = Some("tool_result".to_string());
    capture.message = Some("Capture current Brain Harness plan".to_string());
    capture.tags = vec!["brain-harness".to_string()];
    capture.evidence = vec![MemoryEvidenceRequest {
        kind: "tool_call".to_string(),
        target: "engram.orient trace current-plan-mcp-test".to_string(),
        summary: Some(
            "Resume continuity improved after current-plan MemoryItems were added.".to_string(),
        ),
        excerpt: None,
    }];

    let capture_response = tools::memory_new(&state, capture)
        .await
        .expect("capture_current_plan should work");
    let capture_json = parse_json(&capture_response);
    let item_id = capture_json["item"]["id"].as_str().unwrap().to_string();
    assert_eq!(capture_json["item"]["status"], "active");
    assert_eq!(capture_json["item"]["kind"], "decision");
    assert_eq!(capture_json["item"]["tags"][0], "current-plan");
    assert_eq!(
        capture_json["commit"]["message"],
        "Capture current Brain Harness plan"
    );
    assert_eq!(capture_json["commit"]["changes"][0]["item_id"], item_id);

    let orient_response = tools::orient(
        &state,
        OrientRequest {
            cwd: Some("/Users/yuval.meiri/projects/engram".to_string()),
            prompt: Some("resume current Brain Harness plan".to_string()),
            project: Some("engram".to_string()),
            task: None,
            agent: Some("codex".to_string()),
            external_session_id: None,
            intent: Some("resume_session".to_string()),
            scenario_id: Some("current_plan_capture_test".to_string()),
            arm: Some("capture_current_plan".to_string()),
            include_recent_commits: Some(false),
            limit: Some(5),
            response_shape: None,
        },
    )
    .await
    .expect("orient should work");
    let orient_json = parse_json(&orient_response);

    assert!(orient_json["active_decisions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["id"].as_str() == Some(item_id.as_str())));
    assert_eq!(
        orient_json["brain_loop"]["top_items"][0]["trust"]["memory_id"].as_str(),
        Some(item_id.as_str())
    );
}

#[tokio::test]
async fn test_mcp_memory_capture_current_plan_accepts_string_evidence_from_tool_schema() {
    let state = setup_tool_state().await;

    let capture: MemoryRequestNew = serde_json::from_value(serde_json::json!({
        "action": "capture_current_plan",
        "kind": "decision",
        "title": "String evidence fallback",
        "content": "Accept string evidence from MCP tool schemas while preserving evidence validation.",
        "project_name": "engram",
        "origin": "agent_observed",
        "writer_harness": "codex",
        "model_provider": "openai",
        "model": "gpt-5.5",
        "surface": "desktop",
        "evidence": ["engram.orient trace string-evidence-test"]
    }))
    .expect("tool-schema-shaped request should deserialize");

    let capture_response = tools::memory_new(&state, capture)
        .await
        .expect("string evidence fallback should work");
    let capture_json = parse_json(&capture_response);

    assert_eq!(capture_json["item"]["status"], "active");
    assert_eq!(
        capture_json["item"]["evidence"][0]["target"],
        "engram.orient trace string-evidence-test"
    );
}

#[tokio::test]
async fn test_mcp_memory_string_evidence_does_not_bypass_manual_review_policy() {
    let state = setup_tool_state().await;

    let capture: MemoryRequestNew = serde_json::from_value(serde_json::json!({
        "action": "capture_current_plan",
        "kind": "decision",
        "title": "Reviewed inferred plan",
        "content": "Agent-inferred current plans still require explicit manual_review evidence.",
        "project_name": "engram",
        "origin": "agent_inferred",
        "writer_harness": "codex",
        "model_provider": "openai",
        "model": "gpt-5.5",
        "surface": "desktop",
        "evidence": ["agent inferred this plan from surrounding work"]
    }))
    .expect("tool-schema-shaped request should deserialize");

    let err = tools::memory_new(&state, capture)
        .await
        .expect_err("string evidence must not satisfy manual review");

    assert!(err.contains("manual_review evidence"));
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
async fn test_mcp_memory_forget_requires_confirmation_and_purges_linked_projections() {
    let data = tempdir().expect("persistent test directory should be created");
    let config = StoreConfig::rocksdb(data.path().join("data"));
    let db = connect_and_init(&config)
        .await
        .expect("persistent store should initialize");
    let memory_service = MemoryService::new(db.clone());
    let entity_service = EntityService::new(db.clone());
    let work_service = WorkService::new(db.clone());
    let state = ToolState::new();
    state.init_memory(memory_service).await;
    state.init_entity(entity_service).await;
    state.init_search(SearchService::new(db.clone())).await;
    state.init_work(work_service).await;
    let vault = tempdir().unwrap();
    let canary = "FORGET_CANARY_7f4a2e";
    let mut add = with_writer(request("add"));
    add.kind = Some("decision".to_string());
    add.title = Some(canary.to_string());
    add.content = Some(format!(
        "The {canary} memory should be permanently removed."
    ));
    add.origin = Some("user_stated".to_string());
    add.scope_type = Some("project".to_string());
    add.project_name = Some("engram".to_string());
    let added = tools::memory_new(&state, add)
        .await
        .expect("add should work");
    let id = parse_json(&added)["item"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let orientation = tools::orient(
        &state,
        OrientRequest {
            cwd: None,
            prompt: Some(format!("Retrieve {canary}")),
            project: Some("engram".to_string()),
            task: None,
            agent: Some("codex".to_string()),
            external_session_id: None,
            intent: None,
            scenario_id: Some("forget_projection_purge".to_string()),
            arm: Some("engram".to_string()),
            include_recent_commits: Some(false),
            limit: Some(10),
            response_shape: Some(OrientResponseShape::Lean),
        },
    )
    .await
    .expect("orient should create a trace referencing the item");
    let trace_id = Id::parse(
        parse_json(&orientation)["trace_id"]
            .as_str()
            .expect("orientation should return its trace ID"),
    )
    .expect("trace ID should parse");
    let forgotten_id = Id::parse(&id).expect("memory ID should parse");
    let telemetry = TelemetryRepo::new(db.clone());
    let mut feedback = AgentFeedback::new(trace_id);
    feedback.used_memory_ids = vec![forgotten_id];
    feedback.note = Some(format!("Used {canary}"));
    let feedback_id = feedback.id;
    telemetry
        .save_feedback(&feedback)
        .await
        .expect("feedback referencing the item should persist");
    let mut commit = with_writer(request("commit"));
    commit.message = Some(format!("Commit {canary}"));
    commit.changes = vec![MemoryChangeRequest {
        change_type: "added".to_string(),
        item_id: Some(id.clone()),
        title: canary.to_string(),
        summary: format!("Persisted {canary}"),
        before_hash: None,
        after_hash: None,
    }];
    tools::memory_new(&state, commit)
        .await
        .expect("knowledge commit should be created");
    let mut export = request("export_vault");
    export.vault_path = Some(vault.path().display().to_string());
    let export = tools::memory_new(&state, export)
        .await
        .expect("vault export should work");
    let projected_relative_path = parse_json(&export)["export"]["files_written"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .find(|path| path.starts_with(&format!("memory/items/{id}-")))
        .expect("memory item projection should be written")
        .to_string();
    let projected_path = vault.path().join(&projected_relative_path);
    assert!(projected_path.exists());

    let mut unconfirmed = request("forget");
    unconfirmed.id = Some(id.clone());
    unconfirmed.archive_reason = Some("Canary cleanup".to_string());
    let error = tools::memory_new(&state, unconfirmed)
        .await
        .expect_err("unconfirmed forget must fail");
    assert!(error.contains("confirm_forget=true"));

    let mut forget = request("forget");
    forget.id = Some(id.clone());
    forget.archive_reason = Some("Canary cleanup".to_string());
    forget.confirm_forget = Some(true);
    forget.vault_path = Some(vault.path().display().to_string());
    let response = tools::memory_new(&state, forget)
        .await
        .expect("confirmed forget should work");
    let response = parse_json(&response);
    assert_eq!(response["deleted"], true);
    assert_eq!(response["internal_purge"]["commits_redacted"], 1);
    assert_eq!(response["internal_purge"]["traces_deleted"], 1);
    assert_eq!(response["internal_purge"]["feedback_deleted"], 1);
    assert!(
        data.path().join("data/CURRENT").is_file(),
        "test must exercise a real RocksDB store"
    );
    assert!(response["vault_refresh"]["files_removed"]
        .as_array()
        .unwrap()
        .iter()
        .any(|path| path.as_str() == Some(&projected_relative_path)));
    assert!(!projected_path.exists());
    for relative_path in response["vault_refresh"]["files_written"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
    {
        let contents = fs::read_to_string(vault.path().join(relative_path)).unwrap();
        assert!(
            !contents.contains(canary),
            "{relative_path} retained canary"
        );
    }

    let mut get = request("get");
    get.id = Some(id.clone());
    assert!(tools::memory_new(&state, get).await.is_err());

    let mut list = request("list");
    list.project_name = Some("engram".to_string());
    let list = tools::memory_new(&state, list)
        .await
        .expect("post-forget list should work");
    let log = tools::memory_new(&state, request("log"))
        .await
        .expect("post-forget commit log should work");
    assert!(!list.contains(canary));
    assert!(!list.contains(&id));
    assert!(!log.contains(canary));
    assert!(!log.contains(&id));

    let orientation = tools::orient(
        &state,
        OrientRequest {
            cwd: None,
            prompt: Some(format!("Retrieve {canary}")),
            project: Some("engram".to_string()),
            task: None,
            agent: Some("codex".to_string()),
            external_session_id: None,
            intent: None,
            scenario_id: Some("forget_projection_postcondition".to_string()),
            arm: Some("engram".to_string()),
            include_recent_commits: Some(true),
            limit: Some(10),
            response_shape: Some(OrientResponseShape::Lean),
        },
    )
    .await
    .expect("post-forget orientation should work");
    assert!(!orientation.contains(canary));
    assert!(!orientation.contains(&id));

    let search = tools::search(
        &state,
        SearchRequest {
            query: canary.to_string(),
            limit: 10,
            min_score: Some(0.0),
            layers: Some(vec!["memory".to_string()]),
            intent: Some("verify_decision".to_string()),
            scenario_id: Some("forget_projection_postcondition".to_string()),
            arm: Some("engram".to_string()),
            agent: Some("codex".to_string()),
            session_id: None,
            external_session_id: None,
            project: Some("engram".to_string()),
            task: None,
            cwd: None,
            relevance_mode: Some("local".to_string()),
        },
    )
    .await
    .expect("post-forget unified search should work");
    assert!(!search.contains(canary));
    assert!(!search.contains(&id));
    assert!(telemetry
        .get_feedback(&feedback_id)
        .await
        .expect("post-forget feedback lookup should work")
        .is_none());
    let graph = GraphService::new(db)
        .subgraph(None, 1)
        .await
        .expect("post-forget graph derivation should work");
    let graph = serde_json::to_string(&graph).expect("graph should serialize");
    assert!(!graph.contains(canary));
    assert!(!graph.contains(&id));
}

#[tokio::test]
async fn test_mcp_orient_resolves_exact_task_and_excludes_sibling_task_memory() {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config).await.expect("Failed to connect");
    let memory_service = MemoryService::new(db.clone());
    memory_service
        .init_schema()
        .await
        .expect("Failed to initialize memory schema");
    let work_service = WorkService::new(db);
    work_service
        .init()
        .await
        .expect("Failed to initialize work schema");
    work_service
        .create_project("atlas", None)
        .await
        .expect("project should be created");
    work_service
        .create_project("orbit", None)
        .await
        .expect("second project should be created");
    work_service
        .create_task("atlas", "task-alpha", None, Some("ATLAS-101"))
        .await
        .expect("alpha task should be created");
    work_service
        .create_task("atlas", "task-beta", None, Some("ATLAS-102"))
        .await
        .expect("beta task should be created");

    let state = ToolState::new();
    state.init_memory(memory_service).await;
    state.init_work(work_service).await;

    for (task, title, content) in [
        (
            "task-alpha",
            "Alpha queue rollout",
            "Use the alpha-only queue rollout sequence.",
        ),
        (
            "task-beta",
            "Beta queue rollout",
            "Use the beta-only queue rollout sequence.",
        ),
    ] {
        let mut add = with_writer(request("add"));
        add.kind = Some("decision".to_string());
        add.title = Some(title.to_string());
        add.content = Some(content.to_string());
        add.origin = Some("agent_observed".to_string());
        add.scope_type = Some("task".to_string());
        add.project_name = Some("atlas".to_string());
        add.task_name = Some(task.to_string());
        add.evidence = manual_review_evidence("Reviewed task-scope fixture.");
        tools::memory_new(&state, add)
            .await
            .expect("task memory should be created");
    }

    let project_only = tools::orient(
        &state,
        OrientRequest {
            cwd: None,
            prompt: Some("Which queue rollout applies?".to_string()),
            project: Some("atlas".to_string()),
            task: None,
            agent: Some("codex".to_string()),
            external_session_id: None,
            intent: None,
            scenario_id: Some("task_scope_without_task".to_string()),
            arm: Some("lean_engram".to_string()),
            include_recent_commits: Some(false),
            limit: Some(10),
            response_shape: Some(OrientResponseShape::Full),
        },
    )
    .await
    .expect("project orientation should work");
    let project_only = parse_json(&project_only);
    assert!(project_only["active_decisions"]
        .as_array()
        .unwrap()
        .is_empty());

    let alpha = tools::orient(
        &state,
        OrientRequest {
            cwd: None,
            prompt: Some("Which queue rollout applies?".to_string()),
            project: None,
            task: Some("ATLAS-101".to_string()),
            agent: Some("codex".to_string()),
            external_session_id: None,
            intent: None,
            scenario_id: Some("task_scope_exact".to_string()),
            arm: Some("lean_engram".to_string()),
            include_recent_commits: Some(false),
            limit: Some(10),
            response_shape: Some(OrientResponseShape::Full),
        },
    )
    .await
    .expect("task orientation should work");
    let alpha = parse_json(&alpha);
    assert_eq!(alpha["task"], "task-alpha");
    assert_eq!(alpha["task_context"]["jira_key"], "ATLAS-101");
    assert_eq!(alpha["task_context"]["project"], "atlas");
    assert_eq!(alpha["scope"], "task:atlas/task-alpha");
    assert_eq!(alpha["resolution"]["explicit_project"], Value::Null);
    assert_eq!(alpha["resolution"]["selected_project"], "atlas");
    assert_eq!(alpha["resolution"]["source"], "task");
    assert!(alpha["resolution"]["reason"]
        .as_str()
        .unwrap()
        .contains("caller-supplied task 'task-alpha'"));
    assert_eq!(alpha["active_decisions"].as_array().unwrap().len(), 1);
    assert_eq!(alpha["active_decisions"][0]["title"], "Alpha queue rollout");
    assert!(!alpha.to_string().contains("Beta queue rollout"));

    let alpha_lean = tools::orient(
        &state,
        OrientRequest {
            cwd: None,
            prompt: Some("Which queue rollout applies?".to_string()),
            project: Some("atlas".to_string()),
            task: Some("task-alpha".to_string()),
            agent: Some("claude_code".to_string()),
            external_session_id: None,
            intent: None,
            scenario_id: Some("task_scope_exact_lean".to_string()),
            arm: Some("lean_engram".to_string()),
            include_recent_commits: Some(false),
            limit: Some(10),
            response_shape: None,
        },
    )
    .await
    .expect("lean task orientation should work");
    let alpha_lean = parse_json(&alpha_lean);
    assert_eq!(alpha_lean["response_shape"], "lean");
    assert_eq!(alpha_lean["task"], "task-alpha");
    assert_eq!(alpha_lean["task_context"]["jira_key"], "ATLAS-101");
    assert_eq!(alpha_lean["scope"], "task:atlas/task-alpha");
    assert_eq!(
        alpha_lean["brain_loop"]["top_items"][0]["title"],
        "Alpha queue rollout"
    );
    assert!(!alpha_lean.to_string().contains("Beta queue rollout"));

    let missing_project = tools::orient(
        &state,
        OrientRequest {
            cwd: None,
            prompt: Some("Continue the task".to_string()),
            project: None,
            task: Some("task-alpha".to_string()),
            agent: Some("codex".to_string()),
            external_session_id: None,
            intent: None,
            scenario_id: Some("task_name_without_project".to_string()),
            arm: Some("lean_engram".to_string()),
            include_recent_commits: Some(false),
            limit: Some(10),
            response_shape: Some(OrientResponseShape::Lean),
        },
    )
    .await
    .expect_err("a task name must not resolve without its project");
    assert!(missing_project.contains("supply its project when using a task name"));

    let error = tools::orient(
        &state,
        OrientRequest {
            cwd: None,
            prompt: Some("Continue the task".to_string()),
            project: Some("orbit".to_string()),
            task: Some("ATLAS-101".to_string()),
            agent: Some("codex".to_string()),
            external_session_id: None,
            intent: None,
            scenario_id: None,
            arm: None,
            include_recent_commits: Some(false),
            limit: Some(10),
            response_shape: Some(OrientResponseShape::Lean),
        },
    )
    .await
    .expect_err("task/project mismatch must fail closed");
    assert!(error.contains("belongs to project 'atlas'"));
}

#[tokio::test]
async fn test_mcp_procedure_add_accepts_declarative_prerequisite_source() {
    let state = setup_tool_state().await;
    let mut add = with_writer(request("add"));
    add.kind = Some("procedure".to_string());
    add.title = Some("Source-backed context probe".to_string());
    add.content = Some("Run only when the tracked toolchain matches.".to_string());
    add.origin = Some("agent_observed".to_string());
    add.scope_type = Some("repository".to_string());
    add.remote_url = Some("https://github.com/acme/atlas".to_string());
    add.procedure = Some(MemoryProcedureRequest {
        task: "run the Atlas context probe".to_string(),
        commands: vec!["./bin/context-probe --channel cobalt".to_string()],
        prerequisites: std::collections::BTreeMap::from([(
            "tool.version".to_string(),
            "3".to_string(),
        )]),
        prerequisite_sources: std::collections::BTreeMap::from([(
            "tool.version".to_string(),
            MemoryProcedurePrerequisiteSourceRequest {
                format: "toml".to_string(),
                relative_path: "toolchain.toml".to_string(),
                key_path: vec!["version".to_string()],
            },
        )]),
        failure_signatures: vec!["context probe failed".to_string()],
        verification_command: "./bin/context-probe --channel cobalt".to_string(),
        verification_exit_code: 0,
        verification_output_contains: "ATLAS_CONTEXT_PROBE_OK".to_string(),
    });

    let added = parse_json(&tools::memory_new(&state, add).await.unwrap());

    assert_eq!(
        added["item"]["procedure"]["prerequisites"][0]["source"]["format"],
        "toml"
    );
    assert_eq!(
        added["item"]["procedure"]["prerequisites"][0]["source"]["relative_path"],
        "toolchain.toml"
    );
    assert_eq!(
        added["item"]["procedure"]["prerequisites"][0]["source"]["key_path"],
        serde_json::json!(["version"])
    );
}

#[tokio::test]
async fn test_mcp_procedure_add_rejects_invalid_or_unknown_prerequisite_sources() {
    let state = setup_tool_state().await;
    let mut invalid_path = with_writer(request("add"));
    invalid_path.kind = Some("procedure".to_string());
    invalid_path.title = Some("Unsafe source path".to_string());
    invalid_path.content = Some("Must not escape the resolved checkout.".to_string());
    invalid_path.origin = Some("agent_observed".to_string());
    invalid_path.scope_type = Some("repository".to_string());
    invalid_path.remote_url = Some("https://github.com/acme/atlas".to_string());
    invalid_path.procedure = Some(MemoryProcedureRequest {
        task: "run the Atlas context probe".to_string(),
        commands: vec!["./bin/context-probe".to_string()],
        prerequisites: std::collections::BTreeMap::from([(
            "tool.version".to_string(),
            "3".to_string(),
        )]),
        prerequisite_sources: std::collections::BTreeMap::from([(
            "tool.version".to_string(),
            MemoryProcedurePrerequisiteSourceRequest {
                format: "toml".to_string(),
                relative_path: "../outside.toml".to_string(),
                key_path: vec!["version".to_string()],
            },
        )]),
        failure_signatures: Vec::new(),
        verification_command: "./bin/context-probe".to_string(),
        verification_exit_code: 0,
        verification_output_contains: "ATLAS_CONTEXT_PROBE_OK".to_string(),
    });

    let invalid_path_error = tools::memory_new(&state, invalid_path)
        .await
        .expect_err("checkout traversal must be rejected at capture time");
    assert!(invalid_path_error.contains("safe checkout-relative path"));

    let mut unknown_key = with_writer(request("add"));
    unknown_key.kind = Some("procedure".to_string());
    unknown_key.title = Some("Unknown source key".to_string());
    unknown_key.content = Some("Every source must name a prerequisite.".to_string());
    unknown_key.origin = Some("agent_observed".to_string());
    unknown_key.scope_type = Some("repository".to_string());
    unknown_key.remote_url = Some("https://github.com/acme/atlas".to_string());
    unknown_key.procedure = Some(MemoryProcedureRequest {
        task: "run the Atlas context probe".to_string(),
        commands: vec!["./bin/context-probe".to_string()],
        prerequisites: std::collections::BTreeMap::from([(
            "tool.version".to_string(),
            "3".to_string(),
        )]),
        prerequisite_sources: std::collections::BTreeMap::from([(
            "tool.channel".to_string(),
            MemoryProcedurePrerequisiteSourceRequest {
                format: "toml".to_string(),
                relative_path: "toolchain.toml".to_string(),
                key_path: vec!["channel".to_string()],
            },
        )]),
        failure_signatures: Vec::new(),
        verification_command: "./bin/context-probe".to_string(),
        verification_exit_code: 0,
        verification_output_contains: "ATLAS_CONTEXT_PROBE_OK".to_string(),
    });

    let unknown_key_error = tools::memory_new(&state, unknown_key)
        .await
        .expect_err("a source without a matching prerequisite must be rejected");
    assert!(unknown_key_error.contains("unknown condition key(s): tool.channel"));
}

#[tokio::test]
async fn test_mcp_procedure_match_returns_only_receipt_verified_applicable_procedure() {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config).await.expect("Failed to connect");
    let service = MemoryService::new(db);
    service.init_schema().await.unwrap();
    let state = ToolState::new();
    state.init_memory(service.clone()).await;
    let dir = tempdir().unwrap();
    let receipt_path = dir.path().join("success.json");

    let mut add = with_writer(request("add"));
    add.kind = Some("procedure".to_string());
    add.title = Some("Queue worker integration test".to_string());
    add.content = Some("Run the verified queue worker integration test procedure.".to_string());
    add.origin = Some("agent_observed".to_string());
    add.scope_type = Some("project".to_string());
    add.project_name = Some("atlas".to_string());
    add.procedure = Some(MemoryProcedureRequest {
        task: "run queue worker integration tests".to_string(),
        commands: vec!["cargo test -p queue-worker --test integration".to_string()],
        prerequisites: std::collections::BTreeMap::from([(
            "cargo.version".to_string(),
            "1.80.0".to_string(),
        )]),
        prerequisite_sources: std::collections::BTreeMap::new(),
        failure_signatures: vec!["unknown option --integration".to_string()],
        verification_command: "cargo test -p queue-worker --test integration".to_string(),
        verification_exit_code: 0,
        verification_output_contains: "result: PASS".to_string(),
    });
    let added = tools::memory_new(&state, add)
        .await
        .expect("procedure candidate add should work");
    let added = parse_json(&added);
    let id = Id::parse(added["item"]["id"].as_str().unwrap()).unwrap();
    assert_eq!(added["item"]["status"], "needs_review");

    let receipt = ProcedureVerificationReceipt {
        command: "cargo test -p queue-worker --test integration".to_string(),
        exit_code: 0,
        output: "integration result: PASS".to_string(),
        conditions: std::collections::BTreeMap::from([(
            "cargo.version".to_string(),
            "1.80.0".to_string(),
        )]),
    };
    fs::write(&receipt_path, serde_json::to_vec(&receipt).unwrap()).unwrap();
    service
        .verify_procedure(
            &id,
            &receipt_path,
            Some(time::OffsetDateTime::now_utc() + time::Duration::days(30)),
        )
        .await
        .unwrap();

    let mut matched = request("procedure_match");
    matched.query = Some("run queue worker integration tests".to_string());
    matched.project_name = Some("atlas".to_string());
    matched
        .conditions
        .insert("cargo.version".to_string(), "1.80.0".to_string());
    let matched = tools::memory_new(&state, matched)
        .await
        .expect("procedure match should work");
    let matched = parse_json(&matched);
    assert_eq!(matched["abstained"], false);
    assert_eq!(matched["procedures"][0]["id"], id.to_string());

    let mut mismatched = request("procedure_match");
    mismatched.query = Some("run queue worker integration tests".to_string());
    mismatched.project_name = Some("atlas".to_string());
    mismatched
        .conditions
        .insert("cargo.version".to_string(), "1.79.0".to_string());
    let mismatched = tools::memory_new(&state, mismatched)
        .await
        .expect("procedure mismatch should return abstention");
    assert_eq!(parse_json(&mismatched)["abstained"], true);
}

#[tokio::test]
async fn test_mcp_local_procedure_match_uses_unlinked_repository_remote_and_guides_retry() {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config).await.expect("Failed to connect");
    let service = MemoryService::new(db);
    service.init_schema().await.unwrap();
    let state = ToolState::new();
    state.init_memory(service.clone()).await;

    let dir = tempdir().unwrap();
    let original = dir.path().join("original/atlas");
    let moved = dir.path().join("moved/arbitrary-name");
    for checkout in [&original, &moved] {
        fs::create_dir_all(checkout).unwrap();
        assert!(Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(checkout)
            .status()
            .unwrap()
            .success());
        assert!(Command::new("git")
            .args(["remote", "add", "origin", "git@github.com:acme/atlas.git",])
            .current_dir(checkout)
            .status()
            .unwrap()
            .success());
    }

    let receipt = ProcedureVerificationReceipt {
        command: "./bin/context-probe --channel cobalt".to_string(),
        exit_code: 0,
        output: "ATLAS_CONTEXT_PROBE_OK".to_string(),
        conditions: std::collections::BTreeMap::from([(
            "tool.version".to_string(),
            "3".to_string(),
        )]),
    };
    let receipt_bytes = serde_json::to_vec(&receipt).unwrap();
    let receipt_path = original.join("procedure-success.json");
    fs::write(&receipt_path, &receipt_bytes).unwrap();
    fs::write(moved.join("procedure-success.json"), &receipt_bytes).unwrap();

    let mut add = with_writer(request("add"));
    add.kind = Some("procedure".to_string());
    add.title = Some("procedure-atlas-context-probe-v1".to_string());
    add.content = Some("Run the verified Atlas context probe.".to_string());
    add.origin = Some("agent_observed".to_string());
    add.scope_type = Some("repository".to_string());
    add.remote_url = Some("https://github.com/acme/atlas".to_string());
    add.local_path = Some(original.display().to_string());
    add.procedure = Some(MemoryProcedureRequest {
        task: "run the Atlas context probe".to_string(),
        commands: vec!["./bin/context-probe --channel cobalt".to_string()],
        prerequisites: std::collections::BTreeMap::from([(
            "tool.version".to_string(),
            "3".to_string(),
        )]),
        prerequisite_sources: std::collections::BTreeMap::new(),
        failure_signatures: vec!["./bin/context-probe --channel amber".to_string()],
        verification_command: "./bin/context-probe --channel cobalt".to_string(),
        verification_exit_code: 0,
        verification_output_contains: "ATLAS_CONTEXT_PROBE_OK".to_string(),
    });
    let added = parse_json(&tools::memory_new(&state, add).await.unwrap());
    let id = Id::parse(added["item"]["id"].as_str().unwrap()).unwrap();
    service
        .verify_procedure(
            &id,
            &receipt_path,
            Some(time::OffsetDateTime::now_utc() + time::Duration::days(30)),
        )
        .await
        .unwrap();

    let mut missing = request("procedure_match");
    missing.scope = Some(RetrievalScopeRequest {
        relevance_mode: Some("local".to_string()),
        cwd: Some(moved.display().to_string()),
        ..RetrievalScopeRequest::default()
    });
    missing.query = Some("context probe".to_string());
    let missing = parse_json(&tools::memory_new(&state, missing).await.unwrap());
    assert_eq!(missing["abstained"], true);
    assert_eq!(
        missing["required_condition_keys"],
        serde_json::json!(["tool.version"])
    );
    assert!(missing["next_actions"][0]
        .as_str()
        .unwrap()
        .contains("current_checkout_root"));
    assert!(missing["next_actions"][0]
        .as_str()
        .unwrap()
        .contains("separate direct tool call"));
    assert_eq!(
        missing["resolution"]["repository_remote"],
        "github.com/acme/atlas"
    );
    assert_eq!(missing["identity"]["repository"]["name"], "atlas");
    assert_eq!(
        missing["identity"]["repository"]["normalized_remote"],
        "github.com/acme/atlas"
    );
    assert_eq!(
        missing["identity"]["repository"]["checkout_root"],
        moved.canonicalize().unwrap().display().to_string()
    );
    assert_eq!(
        missing["identity"]["project"]["status"],
        "requires_confirmation"
    );
    assert_eq!(missing["identity"]["project"]["name"], Value::Null);

    let mut matched = request("procedure_match");
    matched.scope = Some(RetrievalScopeRequest {
        relevance_mode: Some("local".to_string()),
        cwd: Some(moved.display().to_string()),
        ..RetrievalScopeRequest::default()
    });
    matched.query = Some("context probe".to_string());
    matched
        .conditions
        .insert("tool.version".to_string(), "3".to_string());
    let matched = parse_json(&tools::memory_new(&state, matched).await.unwrap());
    assert_eq!(matched["abstained"], false);
    assert_eq!(matched["procedures"][0]["id"], id.to_string());
    assert_eq!(
        matched["current_checkout_root"],
        moved.canonicalize().unwrap().display().to_string()
    );
    assert!(matched["execution_guidance"]
        .as_str()
        .unwrap()
        .contains("provenance only"));
    assert_ne!(
        matched["current_checkout_root"],
        matched["procedures"][0]["scope"]["local_path"]
    );
    assert_eq!(matched["resolved_project"], Value::Null);
    assert_eq!(matched["authorization_scope_enforced"], true);
    assert_eq!(matched["resolution"]["source"], "unresolved");
    assert_eq!(matched["resolution"]["repository_name"], "atlas");
    assert_eq!(
        matched["resolution"]["repository_remote"],
        "github.com/acme/atlas"
    );
    assert_eq!(matched["resolution"]["requires_confirmation"], true);
    assert_eq!(
        matched["identity"]["project"]["status"],
        "requires_confirmation"
    );
    assert_eq!(matched["identity"]["project"]["name"], Value::Null);
    assert!(matched["resolution"]["ambiguity"]
        .as_str()
        .unwrap()
        .contains("no linked project candidates"));
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
    add.evidence = manual_review_evidence("Orient test expects active durable guidance.");
    tools::memory_new(&state, add)
        .await
        .expect("add should work");

    let response = tools::orient(
        &state,
        OrientRequest {
            cwd: Some("/Users/yuval.meiri/projects/engram".to_string()),
            prompt: Some("continue implementation".to_string()),
            project: Some("engram".to_string()),
            task: None,
            agent: Some("codex".to_string()),
            external_session_id: None,
            intent: None,
            scenario_id: None,
            arm: None,
            include_recent_commits: Some(true),
            limit: Some(10),
            response_shape: None,
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
    assert_eq!(
        json["memory_metadata"][0]["review_state"],
        "active_unreviewed"
    );
    assert_eq!(json["memory_metadata"][0]["review_asserted"], true);
    assert_eq!(json["memory_metadata"][0]["reviewed"], false);
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
        .contains("Trust: status=active, review_state=active_unreviewed"));
    assert_eq!(json["brain_loop"]["degraded"], false);
    assert!(json["brain_loop"]["compiled_context"]
        .as_str()
        .unwrap()
        .contains("Orient through memory"));
    assert_eq!(
        json["brain_loop"]["top_items"][0]["trust"]["memory_id"],
        json["active_decisions"][0]["id"]
    );
    assert_eq!(
        json["used_memory_candidate_ids"][0],
        json["brain_loop"]["top_items"][0]["id"]
    );
    assert!(json["context_pack"]
        .as_str()
        .unwrap()
        .contains("used_memory_candidate_ids"));
    assert!(json["memory_cursor"]["timestamp"].is_string());
}

#[tokio::test]
async fn test_mcp_orient_lean_response_shape_omits_duplicate_payloads() {
    let state = setup_tool_state().await;

    let mut add = with_writer(request("add"));
    add.kind = Some("decision".to_string());
    add.title = Some("Lean orient preserves Brain Loop signal".to_string());
    add.content = Some(
        "Verification tasks should use compact Brain Loop guidance without duplicated raw memory."
            .to_string(),
    );
    add.origin = Some("user_stated".to_string());
    add.scope_type = Some("project".to_string());
    add.project_name = Some("engram".to_string());
    add.evidence = manual_review_evidence("Lean orient test expects reviewed guidance.");
    tools::memory_new(&state, add)
        .await
        .expect("add should work");

    let full_response = tools::orient(
        &state,
        OrientRequest {
            cwd: Some("/Users/yuval.meiri/projects/engram".to_string()),
            prompt: Some("verify compact Brain Loop guidance".to_string()),
            project: Some("engram".to_string()),
            task: None,
            agent: Some("claude_code".to_string()),
            external_session_id: None,
            intent: Some("verify_decision".to_string()),
            scenario_id: None,
            arm: None,
            include_recent_commits: Some(false),
            limit: Some(5),
            response_shape: Some(OrientResponseShape::Full),
        },
    )
    .await
    .expect("full orient should work");
    let full_json = parse_json(&full_response);
    assert!(full_json["context_pack"].is_string());
    assert!(full_json["active_decisions"].is_array());
    assert!(full_json["memory_metadata"].is_array());
    assert!(full_json["brain_loop"]["top_items"][0]["trust"].is_object());

    let lean_response = tools::orient(
        &state,
        OrientRequest {
            cwd: Some("/Users/yuval.meiri/projects/engram".to_string()),
            prompt: Some("verify compact Brain Loop guidance".to_string()),
            project: Some("engram".to_string()),
            task: None,
            agent: Some("claude_code".to_string()),
            external_session_id: None,
            intent: Some("verify_decision".to_string()),
            scenario_id: None,
            arm: None,
            include_recent_commits: Some(false),
            limit: Some(5),
            response_shape: Some(OrientResponseShape::Lean),
        },
    )
    .await
    .expect("lean orient should work");
    let lean_json = parse_json(&lean_response);

    assert_eq!(lean_json["response_shape"], "lean");
    assert_eq!(lean_json["project"], "engram");
    assert!(lean_json["trace_id"].is_string());
    assert!(lean_json["memory_cursor"]["timestamp"].is_string());
    assert_eq!(
        lean_json["brain_loop"]["top_items"][0]["title"],
        "Lean orient preserves Brain Loop signal"
    );
    assert_eq!(
        lean_json["used_memory_candidate_ids"][0],
        lean_json["brain_loop"]["top_items"][0]["id"]
    );
    assert_eq!(lean_json["obligation_summary"]["available"], false);
    assert!(lean_json.get("context_pack").is_none());
    assert!(lean_json.get("active_decisions").is_none());
    assert!(lean_json.get("memory_metadata").is_none());
    assert!(lean_json.get("recent_knowledge_commits").is_none());
    assert!(lean_json["brain_loop"]["top_items"][0]
        .get("trust")
        .is_none());
    assert!(lean_response.len() < full_response.len());
}

#[tokio::test]
async fn test_mcp_orient_claude_omitted_shape_defaults_to_lean_but_explicit_full_wins() {
    let state = setup_tool_state().await;

    let omitted_response = tools::orient(
        &state,
        OrientRequest {
            cwd: Some("/Users/yuval.meiri/projects/engram".to_string()),
            prompt: Some("verify Claude default response shape".to_string()),
            project: Some("engram".to_string()),
            task: None,
            agent: Some("claude_code".to_string()),
            external_session_id: None,
            intent: Some("verify_decision".to_string()),
            scenario_id: None,
            arm: None,
            include_recent_commits: Some(false),
            limit: Some(5),
            response_shape: None,
        },
    )
    .await
    .expect("omitted Claude orient should work");
    let omitted_json = parse_json(&omitted_response);
    assert_eq!(omitted_json["response_shape"], "lean");
    assert!(omitted_json.get("context_pack").is_none());

    let explicit_full_response = tools::orient(
        &state,
        OrientRequest {
            cwd: Some("/Users/yuval.meiri/projects/engram".to_string()),
            prompt: Some("verify Claude explicit full response shape".to_string()),
            project: Some("engram".to_string()),
            task: None,
            agent: Some("claude_code".to_string()),
            external_session_id: None,
            intent: Some("verify_decision".to_string()),
            scenario_id: None,
            arm: None,
            include_recent_commits: Some(false),
            limit: Some(5),
            response_shape: Some(OrientResponseShape::Full),
        },
    )
    .await
    .expect("explicit full Claude orient should work");
    let explicit_full_json = parse_json(&explicit_full_response);
    assert!(explicit_full_json["context_pack"].is_string());
    assert!(explicit_full_json.get("response_shape").is_none());
}

#[tokio::test]
async fn test_mcp_orient_prepare_handoff_lean_surfaces_current_plan_and_gates() {
    let state = setup_tool_state().await;

    let mut stale = with_writer(request("add"));
    stale.kind = Some("decision".to_string());
    stale.title = Some("Current plan after Codex document lifecycle follow-through".to_string());
    stale.content = Some(
        "Older repository-scoped current-plan guidance that should not lead a compact handoff."
            .to_string(),
    );
    stale.origin = Some("tool_result".to_string());
    stale.scope_type = Some("repository".to_string());
    stale.local_path = Some("/Users/yuval.meiri/projects/engram".to_string());
    stale.tags = vec!["current-plan".to_string()];
    stale.evidence = manual_review_evidence("Stale repository current-plan fixture.");
    let stale_response = tools::memory_new(&state, stale)
        .await
        .expect("stale current-plan add should work");
    let stale_id = parse_json(&stale_response)["item"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    tokio::time::sleep(std::time::Duration::from_millis(1)).await;

    let mut latest = with_writer(request("add"));
    latest.kind = Some("decision".to_string());
    latest.title = Some("Current plan: fix prepare_handoff orientation".to_string());
    latest.content = Some(
        "Latest current plan: validate compact prepare_handoff orientation before any migration, \
         lifecycle, hook, schema, public MCP, broad ranking, or payload change."
            .to_string(),
    );
    latest.origin = Some("tool_result".to_string());
    latest.scope_type = Some("project".to_string());
    latest.project_name = Some("engram".to_string());
    latest.tags = vec!["current-plan".to_string()];
    latest.evidence = manual_review_evidence("Latest current-plan fixture.");
    let latest_response = tools::memory_new(&state, latest)
        .await
        .expect("latest current-plan add should work");
    let latest_id = parse_json(&latest_response)["item"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let mut secondary = with_writer(request("add"));
    secondary.kind = Some("decision".to_string());
    secondary.title = Some("Mission-class PlanWork current-plan gap resolved narrowly".to_string());
    secondary.content = Some(
        "Earlier mission-class plan_work prompts now preserve current-plan continuity, but this \
         implementation-history item is not the current handoff plan."
            .to_string(),
    );
    secondary.origin = Some("tool_result".to_string());
    secondary.scope_type = Some("project".to_string());
    secondary.project_name = Some("engram".to_string());
    secondary.evidence = manual_review_evidence("Secondary decision fixture.");
    tools::memory_new(&state, secondary)
        .await
        .expect("secondary decision add should work");

    let mut research_rule = with_writer(request("add"));
    research_rule.kind = Some("rule".to_string());
    research_rule.title = Some("Brain Harness work follows research method".to_string());
    research_rule.content = Some(
        "Brain Harness work uses explicit research questions, competing hypotheses, evidence \
         levels, falsifiers, decision gates, and claim-ledger updates."
            .to_string(),
    );
    research_rule.origin = Some("user_stated".to_string());
    research_rule.scope_type = Some("project".to_string());
    research_rule.project_name = Some("engram".to_string());
    research_rule.evidence = manual_review_evidence("Research method rule fixture.");
    tools::memory_new(&state, research_rule)
        .await
        .expect("research rule add should work");

    let mut preference = with_writer(request("add"));
    preference.kind = Some("preference".to_string());
    preference.title =
        Some("Software design philosophy: deep modules and evidence over confidence".to_string());
    preference.content = Some(
        "Prefer Ousterhout-style deep modules, low cognitive load, no unrequested features, and \
         evidence over confidence."
            .to_string(),
    );
    preference.origin = Some("user_stated".to_string());
    preference.scope_type = Some("project".to_string());
    preference.project_name = Some("engram".to_string());
    preference.evidence = manual_review_evidence("Software design preference fixture.");
    tools::memory_new(&state, preference)
        .await
        .expect("preference add should work");

    let mut non_gate_noise = with_writer(request("add"));
    non_gate_noise.kind = Some("limitation".to_string());
    non_gate_noise.title =
        Some("Non-gated calibration does not prove broad ranking quality".to_string());
    non_gate_noise.content = Some(
        "The non-gated continuation calibration fixes one prompt class but should not be treated \
         as broad ranking proof."
            .to_string(),
    );
    non_gate_noise.origin = Some("tool_result".to_string());
    non_gate_noise.scope_type = Some("project".to_string());
    non_gate_noise.project_name = Some("engram".to_string());
    non_gate_noise.evidence = manual_review_evidence("Non-gated calibration noise fixture.");
    tools::memory_new(&state, non_gate_noise)
        .await
        .expect("non-gated calibration noise add should work");

    let mut m6_gate = with_writer(request("add"));
    m6_gate.kind = Some("limitation".to_string());
    m6_gate.title = Some("M6 migration approval gate remains explicit".to_string());
    m6_gate.content = Some(
        "Brain Harness handoff approval gates must say that M6 migration read-only inventory or \
         review export needs explicit user-approved scope, and write apply, deletion, cleanup, or \
         legacy simplification need reviewed candidates, dry-run evidence, rollback planning, and \
         explicit approval."
            .to_string(),
    );
    m6_gate.origin = Some("user_stated".to_string());
    m6_gate.scope_type = Some("project".to_string());
    m6_gate.project_name = Some("engram".to_string());
    m6_gate.evidence = manual_review_evidence("M6 handoff gate fixture.");
    let m6_response = tools::memory_new(&state, m6_gate)
        .await
        .expect("M6 gate add should work");
    let m6_id = parse_json(&m6_response)["item"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let mut harness_gate = with_writer(request("add"));
    harness_gate.kind = Some("rule".to_string());
    harness_gate.title = Some("Harness adapter and hook write approval gate".to_string());
    harness_gate.content = Some(
        "Brain Harness handoffs must preserve the harness-write gate: do not install or modify \
         Claude Code, Codex, Gemini CLI, or Cursor adapters, settings, or hooks without explicit \
         user approval."
            .to_string(),
    );
    harness_gate.origin = Some("user_stated".to_string());
    harness_gate.scope_type = Some("project".to_string());
    harness_gate.project_name = Some("engram".to_string());
    harness_gate.evidence = manual_review_evidence("Harness-write handoff gate fixture.");
    let harness_response = tools::memory_new(&state, harness_gate)
        .await
        .expect("harness gate add should work");
    let harness_id = parse_json(&harness_response)["item"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let response = tools::orient(
        &state,
        OrientRequest {
            cwd: Some("/Users/yuval.meiri/projects/engram".to_string()),
            prompt: Some(
                "Prepare a compact Brain Harness handoff: current plan, approval gates, \
                 evidence-quality state, and next non-gated work."
                    .to_string(),
            ),
            project: Some("engram".to_string()),
            task: None,
            agent: Some("codex".to_string()),
            external_session_id: None,
            intent: Some("prepare_handoff".to_string()),
            scenario_id: Some("t35_prepare_handoff_gate_summary_20260527".to_string()),
            arm: Some("fixture".to_string()),
            include_recent_commits: Some(false),
            limit: Some(10),
            response_shape: Some(OrientResponseShape::Lean),
        },
    )
    .await
    .expect("prepare_handoff orient should work");
    let json = parse_json(&response);
    let top_ids = json["brain_loop"]["top_items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["id"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();

    assert_eq!(json["response_shape"], "lean");
    assert_eq!(top_ids.first(), Some(&latest_id));
    assert!(top_ids.contains(&m6_id));
    assert!(top_ids.contains(&harness_id));
    assert!(!top_ids.contains(&stale_id));
    assert!(json["used_memory_candidate_ids"]
        .as_array()
        .unwrap()
        .iter()
        .all(|id| id.as_str() != Some(stale_id.as_str())));
    assert!(json.get("context_pack").is_none());
    assert!(json.get("active_decisions").is_none());
    assert!(json["brain_loop"]["top_items"][0].get("trust").is_none());
}

#[tokio::test]
async fn test_mcp_orient_no_prompt_plan_work_surfaces_current_plan_at_project_boundary() {
    let state = setup_tool_state().await;

    let mut current_plan = with_writer(request("add"));
    current_plan.kind = Some("decision".to_string());
    current_plan.title = Some("Current plan: implement T146 no-prompt orientation".to_string());
    current_plan.content = Some(
        "Latest current plan: implement the narrow no-prompt PlanWork current-plan fix before \
         any migration, schema, harness, public MCP, payload, lifecycle, or runtime change."
            .to_string(),
    );
    current_plan.origin = Some("tool_result".to_string());
    current_plan.scope_type = Some("project".to_string());
    current_plan.project_name = Some("engram".to_string());
    current_plan.tags = vec!["current-plan".to_string()];
    current_plan.evidence = manual_review_evidence("T146 current-plan fixture.");
    let current_plan_response = tools::memory_new(&state, current_plan)
        .await
        .expect("current-plan add should work");
    let current_plan_id = parse_json(&current_plan_response)["item"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    tokio::time::sleep(std::time::Duration::from_millis(1)).await;

    let mut secondary = with_writer(request("add"));
    secondary.kind = Some("decision".to_string());
    secondary.title = Some("Secondary implementation note".to_string());
    secondary.content =
        Some("A later non-current decision should not displace the current plan.".to_string());
    secondary.origin = Some("tool_result".to_string());
    secondary.scope_type = Some("project".to_string());
    secondary.project_name = Some("engram".to_string());
    secondary.evidence = manual_review_evidence("Secondary no-prompt fixture.");
    tools::memory_new(&state, secondary)
        .await
        .expect("secondary decision add should work");

    let mut rule = with_writer(request("add"));
    rule.kind = Some("rule".to_string());
    rule.title = Some("Brain Harness changes stay narrow".to_string());
    rule.content = Some(
        "No-prompt orientation fixes should not expand orient payload shape, public MCP \
         parameters, migration behavior, harness behavior, or storage schema."
            .to_string(),
    );
    rule.origin = Some("user_stated".to_string());
    rule.scope_type = Some("project".to_string());
    rule.project_name = Some("engram".to_string());
    rule.evidence = manual_review_evidence("No-prompt rule fixture.");
    tools::memory_new(&state, rule)
        .await
        .expect("rule add should work");

    let mut preference = with_writer(request("add"));
    preference.kind = Some("preference".to_string());
    preference.title = Some("Prefer evidence before ranking changes".to_string());
    preference.content =
        Some("Ranking behavior should change only with focused fixture evidence.".to_string());
    preference.origin = Some("user_stated".to_string());
    preference.scope_type = Some("project".to_string());
    preference.project_name = Some("engram".to_string());
    preference.evidence = manual_review_evidence("No-prompt preference fixture.");
    tools::memory_new(&state, preference)
        .await
        .expect("preference add should work");

    let full_response = tools::orient(
        &state,
        OrientRequest {
            cwd: Some("/Users/yuval.meiri/projects/engram".to_string()),
            prompt: None,
            project: Some("engram".to_string()),
            task: None,
            agent: Some("codex".to_string()),
            external_session_id: None,
            intent: Some("plan_work".to_string()),
            scenario_id: Some("t146_no_prompt_plan_work_project_boundary".to_string()),
            arm: Some("fixture".to_string()),
            include_recent_commits: Some(false),
            limit: Some(5),
            response_shape: None,
        },
    )
    .await
    .expect("no-prompt full orient should work");
    let full_json = parse_json(&full_response);

    assert_eq!(full_json["active_decisions"][0]["id"], current_plan_id);
    assert_eq!(
        full_json["brain_loop"]["top_items"][0]["id"],
        current_plan_id
    );
    assert!(full_json["used_memory_candidate_ids"]
        .as_array()
        .unwrap()
        .iter()
        .any(|id| id.as_str() == Some(current_plan_id.as_str())));

    let lean_response = tools::orient(
        &state,
        OrientRequest {
            cwd: Some("/Users/yuval.meiri/projects/engram".to_string()),
            prompt: None,
            project: Some("engram".to_string()),
            task: None,
            agent: Some("codex".to_string()),
            external_session_id: None,
            intent: Some("plan_work".to_string()),
            scenario_id: Some("t146_no_prompt_plan_work_project_boundary".to_string()),
            arm: Some("fixture".to_string()),
            include_recent_commits: Some(false),
            limit: Some(5),
            response_shape: Some(OrientResponseShape::Lean),
        },
    )
    .await
    .expect("no-prompt lean orient should work");
    let lean_json = parse_json(&lean_response);

    assert_eq!(lean_json["response_shape"], "lean");
    assert_eq!(
        lean_json["brain_loop"]["top_items"][0]["id"],
        current_plan_id
    );
    assert!(lean_json.get("context_pack").is_none());
    assert!(lean_json.get("active_decisions").is_none());
    assert!(lean_json["brain_loop"]["top_items"][0]
        .get("trust")
        .is_none());
}

#[tokio::test]
async fn test_mcp_orient_no_prompt_plan_work_without_boundary_or_plan_does_not_synthesize_plan() {
    let state = setup_tool_state().await;

    let mut current_plan = with_writer(request("add"));
    current_plan.kind = Some("decision".to_string());
    current_plan.title = Some("Current plan: project-scoped only".to_string());
    current_plan.content = Some(
        "Project-scoped current plan should not appear in an unscoped orientation.".to_string(),
    );
    current_plan.origin = Some("tool_result".to_string());
    current_plan.scope_type = Some("project".to_string());
    current_plan.project_name = Some("engram".to_string());
    current_plan.tags = vec!["current-plan".to_string()];
    current_plan.evidence = manual_review_evidence("Unscoped current-plan guard fixture.");
    let current_plan_response = tools::memory_new(&state, current_plan)
        .await
        .expect("current-plan add should work");
    let current_plan_id = parse_json(&current_plan_response)["item"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let mut global_rule = with_writer(request("add"));
    global_rule.kind = Some("rule".to_string());
    global_rule.title = Some("Global orientation rule".to_string());
    global_rule.content =
        Some("Global memory can appear when no project boundary is available.".to_string());
    global_rule.origin = Some("user_stated".to_string());
    global_rule.scope_type = Some("global".to_string());
    global_rule.evidence = manual_review_evidence("Unscoped global rule fixture.");
    let global_rule_response = tools::memory_new(&state, global_rule)
        .await
        .expect("global rule add should work");
    let global_rule_id = parse_json(&global_rule_response)["item"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let unscoped_response = tools::orient(
        &state,
        OrientRequest {
            cwd: None,
            prompt: None,
            project: None,
            task: None,
            agent: Some("codex".to_string()),
            external_session_id: None,
            intent: Some("plan_work".to_string()),
            scenario_id: Some("t146_no_prompt_plan_work_unscoped_guard".to_string()),
            arm: Some("fixture".to_string()),
            include_recent_commits: Some(false),
            limit: Some(5),
            response_shape: None,
        },
    )
    .await
    .expect("unscoped orient should work");
    let unscoped_json = parse_json(&unscoped_response);

    assert_eq!(
        unscoped_json["brain_loop"]["top_items"][0]["id"],
        global_rule_id
    );
    assert!(unscoped_json["used_memory_candidate_ids"]
        .as_array()
        .unwrap()
        .iter()
        .all(|id| id.as_str() != Some(current_plan_id.as_str())));

    let no_plan_state = setup_tool_state().await;
    let mut scoped_rule = with_writer(request("add"));
    scoped_rule.kind = Some("rule".to_string());
    scoped_rule.title = Some("Scoped orientation rule".to_string());
    scoped_rule.content = Some(
        "No-prompt project orientation without current-plan memory should stay rule-led."
            .to_string(),
    );
    scoped_rule.origin = Some("user_stated".to_string());
    scoped_rule.scope_type = Some("project".to_string());
    scoped_rule.project_name = Some("engram".to_string());
    scoped_rule.evidence = manual_review_evidence("No-current-plan guard fixture.");
    let scoped_rule_response = tools::memory_new(&no_plan_state, scoped_rule)
        .await
        .expect("scoped rule add should work");
    let scoped_rule_id = parse_json(&scoped_rule_response)["item"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let no_plan_response = tools::orient(
        &no_plan_state,
        OrientRequest {
            cwd: Some("/Users/yuval.meiri/projects/engram".to_string()),
            prompt: None,
            project: Some("engram".to_string()),
            task: None,
            agent: Some("codex".to_string()),
            external_session_id: None,
            intent: Some("plan_work".to_string()),
            scenario_id: Some("t146_no_prompt_plan_work_no_current_plan".to_string()),
            arm: Some("fixture".to_string()),
            include_recent_commits: Some(false),
            limit: Some(5),
            response_shape: None,
        },
    )
    .await
    .expect("no-current-plan orient should work");
    let no_plan_json = parse_json(&no_plan_response);

    assert_eq!(
        no_plan_json["brain_loop"]["top_items"][0]["id"],
        scoped_rule_id
    );
    assert!(no_plan_json["active_decisions"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn test_mcp_orient_routes_inferred_memory_to_review_needed() {
    let state = setup_tool_state().await;

    let mut add = with_writer(request("add"));
    add.kind = Some("decision".to_string());
    add.title = Some("Inferred branch policy".to_string());
    add.content =
        Some("Agents inferred that feature branches should be rebased daily.".to_string());
    add.origin = Some("agent_inferred".to_string());
    add.scope_type = Some("project".to_string());
    add.project_name = Some("engram".to_string());
    tools::memory_new(&state, add)
        .await
        .expect("add should work");

    let response = tools::orient(
        &state,
        OrientRequest {
            cwd: Some("/Users/yuval.meiri/projects/engram".to_string()),
            prompt: Some("continue branch policy work".to_string()),
            project: Some("engram".to_string()),
            task: None,
            agent: Some("codex".to_string()),
            external_session_id: None,
            intent: Some("plan_work".to_string()),
            scenario_id: None,
            arm: None,
            include_recent_commits: Some(false),
            limit: Some(5),
            response_shape: None,
        },
    )
    .await
    .expect("orient should work");

    let json = parse_json(&response);
    assert!(json["active_decisions"].as_array().unwrap().is_empty());
    assert_eq!(json["review_needed"][0]["title"], "Inferred branch policy");
    assert_eq!(json["review_needed"][0]["status"], "needs_review");
    assert!(json["recommended_actions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|action| action
            .as_str()
            .unwrap()
            .contains("Review needs_review memory")));
}

#[tokio::test]
async fn test_mcp_orient_ranks_reviewed_decisions_by_prompt() {
    let state = setup_tool_state().await;

    let mut throttling = with_writer(request("add"));
    throttling.kind = Some("decision".to_string());
    throttling.title = Some("Prefer token bucket throttling".to_string());
    throttling.content = Some(
        "Request throttling work should use a token bucket limiter before adding new queues."
            .to_string(),
    );
    throttling.origin = Some("user_stated".to_string());
    throttling.scope_type = Some("project".to_string());
    throttling.project_name = Some("engram".to_string());
    throttling.tags = vec!["throttling".to_string(), "requests".to_string()];
    throttling.evidence = manual_review_evidence("Reviewed throttling guidance.");
    tools::memory_new(&state, throttling)
        .await
        .expect("throttling add should work");

    let mut migration = with_writer(request("add"));
    migration.kind = Some("decision".to_string());
    migration.title = Some("Prefer write-ahead schema migration".to_string());
    migration.content = Some(
        "Schema migration work should write an append-only migration log before changing tables."
            .to_string(),
    );
    migration.origin = Some("user_stated".to_string());
    migration.scope_type = Some("project".to_string());
    migration.project_name = Some("engram".to_string());
    migration.tags = vec!["schema".to_string(), "migration".to_string()];
    migration.evidence = manual_review_evidence("Reviewed schema migration guidance.");
    tools::memory_new(&state, migration)
        .await
        .expect("migration add should work");

    let mut current_plan = with_writer(request("add"));
    current_plan.kind = Some("decision".to_string());
    current_plan.title = Some("Current plan: finish no-prompt orientation".to_string());
    current_plan.content = Some(
        "Latest current plan covers no-prompt PlanWork continuity and should not override a \
         specific implementation prompt."
            .to_string(),
    );
    current_plan.origin = Some("tool_result".to_string());
    current_plan.scope_type = Some("project".to_string());
    current_plan.project_name = Some("engram".to_string());
    current_plan.tags = vec!["current-plan".to_string()];
    current_plan.evidence = manual_review_evidence("Specific prompt guard fixture.");
    let current_plan_response = tools::memory_new(&state, current_plan)
        .await
        .expect("current-plan add should work");
    let current_plan_id = parse_json(&current_plan_response)["item"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let throttling_response = tools::orient(
        &state,
        OrientRequest {
            cwd: Some("/Users/yuval.meiri/projects/engram".to_string()),
            prompt: Some("implement request throttling".to_string()),
            project: Some("engram".to_string()),
            task: None,
            agent: Some("codex".to_string()),
            external_session_id: None,
            intent: Some("plan_work".to_string()),
            scenario_id: None,
            arm: None,
            include_recent_commits: Some(false),
            limit: Some(5),
            response_shape: None,
        },
    )
    .await
    .expect("orient should work");
    let throttling_json = parse_json(&throttling_response);
    assert_eq!(
        throttling_json["active_decisions"][0]["title"],
        "Prefer token bucket throttling"
    );
    assert_ne!(
        throttling_json["active_decisions"][0]["id"],
        current_plan_id
    );
    assert_eq!(
        throttling_json["brain_loop"]["top_items"][0]["title"],
        "Prefer token bucket throttling"
    );
    assert_ne!(
        throttling_json["brain_loop"]["top_items"][0]["id"],
        current_plan_id
    );

    let migration_response = tools::orient(
        &state,
        OrientRequest {
            cwd: Some("/Users/yuval.meiri/projects/engram".to_string()),
            prompt: Some("plan schema migration".to_string()),
            project: Some("engram".to_string()),
            task: None,
            agent: Some("codex".to_string()),
            external_session_id: None,
            intent: Some("plan_work".to_string()),
            scenario_id: None,
            arm: None,
            include_recent_commits: Some(false),
            limit: Some(5),
            response_shape: None,
        },
    )
    .await
    .expect("orient should work");
    let migration_json = parse_json(&migration_response);
    assert_eq!(
        migration_json["active_decisions"][0]["title"],
        "Prefer write-ahead schema migration"
    );
    assert_eq!(
        migration_json["brain_loop"]["top_items"][0]["title"],
        "Prefer write-ahead schema migration"
    );
}

#[tokio::test]
async fn test_mcp_orient_keeps_unverified_review_assertion_out_of_hot_context() {
    let state = setup_tool_state().await;

    let mut decision = with_writer(request("add"));
    decision.kind = Some("decision".to_string());
    decision.title = Some("Calibration run is active".to_string());
    decision.content =
        Some("Calibration update planning should stay scoped to the current run log.".to_string());
    decision.origin = Some("user_stated".to_string());
    decision.scope_type = Some("project".to_string());
    decision.project_name = Some("engram".to_string());
    decision.evidence = manual_review_evidence("Reviewed calibration decision.");
    tools::memory_new(&state, decision)
        .await
        .expect("decision add should work");

    let mut preference = with_writer(request("add"));
    preference.kind = Some("preference".to_string());
    preference.title = Some("Commit every meaningful Engram step".to_string());
    preference.content = Some(
        "When developing Engram, create a focused git commit after each meaningful implementation, validation, or documentation step. Keep unrelated user-owned files, such as AGENTS.md, out of those commits unless the user explicitly asks to include them.".to_string(),
    );
    preference.origin = Some("user_stated".to_string());
    preference.scope_type = Some("project".to_string());
    preference.project_name = Some("engram".to_string());
    preference.evidence = manual_review_evidence("Reviewed commit-hygiene preference.");
    let preference_response = tools::memory_new(&state, preference)
        .await
        .expect("preference add should work");
    let preference_id = parse_json(&preference_response)["item"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let response = tools::orient(
        &state,
        OrientRequest {
            cwd: Some("/Users/yuval.meiri/projects/engram".to_string()),
            prompt: Some(
                "Prepare a small Engram doc-only calibration update plan. Include how you will handle unrelated files and when you will commit. Do not implement yet."
                    .to_string(),
            ),
            project: Some("engram".to_string()),
            task: None,
            agent: Some("claude_code".to_string()),
            external_session_id: None,
            intent: Some("follow_user_preference".to_string()),
            scenario_id: Some("claude_rescue_commit_hygiene_001".to_string()),
            arm: Some("test_hot_context".to_string()),
            include_recent_commits: Some(false),
            limit: Some(5),
            response_shape: Some(OrientResponseShape::Full),
        },
    )
    .await
    .expect("orient should work");
    let hot_ids_index = response
        .find("\"hot_context_ids\"")
        .expect("hot_context_ids should be top-level output");
    let context_pack_index = response
        .find("\"context_pack\"")
        .expect("context_pack should be top-level output");
    assert!(hot_ids_index < context_pack_index);
    let json = parse_json(&response);
    let context_pack = json["context_pack"].as_str().unwrap();
    assert!(!context_pack.contains("## Hot Context"));
    assert!(context_pack.contains("Commit every meaningful Engram step"));
    assert!(json["hot_context_ids"].as_array().unwrap().is_empty());
    assert!(json["hot_context_items"].as_array().unwrap().is_empty());
    let metadata = json["memory_metadata"]
        .as_array()
        .unwrap()
        .iter()
        .find(|metadata| metadata["memory_id"].as_str() == Some(preference_id.as_str()))
        .expect("preference trust metadata should be present");
    assert_eq!(metadata["review_asserted"], true);
    assert_eq!(metadata["reviewed"], false);
    assert_eq!(metadata["review_state"], "active_unreviewed");
    assert!(json["used_memory_candidate_ids"]
        .as_array()
        .unwrap()
        .iter()
        .any(|id| id.as_str() == Some(preference_id.as_str())));
}
