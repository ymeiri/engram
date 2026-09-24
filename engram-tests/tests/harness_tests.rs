//! Integration tests for Memory OS harness MCP tooling.

use engram_core::id::Id;
use engram_core::repository::{
    GitRepository, LocalCheckout, ProjectRepositoryLink, ProjectRepositoryRole,
};
use engram_core::session::EventType;
use engram_index::{
    HandoffService, MemoryService, ObligationService, SearchService, SessionService, WorkService,
};
use engram_mcp::tools::{self, HandoffRequest, HarnessRequest, RetrievalScopeRequest, ToolState};
use engram_store::{connect_and_init, RepositoryRepo, StoreConfig};
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

async fn setup_tool_state() -> ToolState {
    setup_tool_state_with_memory().await.0
}

async fn setup_tool_state_with_memory() -> (ToolState, MemoryService) {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config).await.expect("Failed to connect");

    let memory = MemoryService::new(db.clone());
    memory
        .init_schema()
        .await
        .expect("Failed to initialize memory schema");
    let obligations = ObligationService::new(db.clone());
    obligations
        .init_schema()
        .await
        .expect("Failed to initialize obligation schema");
    let handoff = HandoffService::new(db.clone());
    handoff
        .init_schema()
        .await
        .expect("Failed to initialize handoff schema");
    let sessions = SessionService::new(db.clone());
    sessions
        .init()
        .await
        .expect("Failed to initialize session schema");
    let work = WorkService::new(db.clone());
    work.init().await.expect("Failed to initialize work schema");

    let state = ToolState::new();
    state.init_memory(memory.clone()).await;
    state.init_obligation(obligations).await;
    state.init_handoff(handoff).await;
    state.init_session(sessions).await;
    state.init_search(SearchService::new(db)).await;
    state.init_work(work).await;
    (state, memory)
}

fn request(action: &str) -> HarnessRequest {
    HarnessRequest {
        action: action.to_string(),
        harness: Some("claude_code".to_string()),
        enforcement: None,
        root: None,
        adapter: None,
        write: None,
        adopt_user_owned: None,
        settings_target: None,
        observed_mcp_tools: Vec::new(),
        attest_host_configuration: false,
        hook_event_name: None,
        session_id: None,
        cwd: None,
        transcript_path: None,
        prompt: None,
        tool_name: None,
        tool_error: None,
        tool_input_command: None,
        tool_input_action: None,
        file_path: None,
        last_assistant_message: None,
        compact_summary: None,
        trigger: None,
        reason: None,
        stop_hook_active: None,
        write_policy: None,
        project: None,
        scope: Some(RetrievalScopeRequest {
            relevance_mode: Some("global".to_string()),
            ..RetrievalScopeRequest::default()
        }),
        model_provider: None,
        model: None,
        surface: None,
        actor: None,
    }
}

fn parse_json(response: &str) -> Value {
    serde_json::from_str(response).expect("response should be valid JSON")
}

async fn setup_related_harness_state(root: &std::path::Path) -> ToolState {
    let db = connect_and_init(&StoreConfig::memory())
        .await
        .expect("Failed to connect");
    let work = WorkService::new(db.clone());
    work.init().await.expect("Failed to initialize work schema");
    let alpha = work.create_project("alpha", None).await.unwrap();
    work.create_project("beta", None).await.unwrap();
    work.create_task("alpha", "alpha-one", None, Some("ALPHA-1"))
        .await
        .unwrap();
    let sessions = SessionService::new(db.clone());
    sessions
        .init()
        .await
        .expect("Failed to initialize session schema");
    let repositories = RepositoryRepo::new(db.clone());
    repositories
        .init_schema()
        .await
        .expect("Failed to initialize repository schema");
    let repository = GitRepository::new("alpha-repository");
    repositories.save_repository(&repository).await.unwrap();
    repositories
        .save_checkout(
            &LocalCheckout::new(root.display().to_string()).with_repository(repository.id),
        )
        .await
        .unwrap();
    repositories
        .save_project_link(
            &ProjectRepositoryLink::new("alpha", repository.id, ProjectRepositoryRole::Primary)
                .with_project_id(alpha.id),
        )
        .await
        .unwrap();
    let handoff = HandoffService::new(db.clone());
    handoff
        .init_schema()
        .await
        .expect("Failed to initialize handoff schema");

    let state = ToolState::new();
    state.init_search(SearchService::new(db)).await;
    state.init_handoff(handoff).await;
    state
}

#[tokio::test]
async fn mcp_harness_status_and_doctor_abstain_locally_before_host_or_file_access() {
    let root = tempdir().expect("tempdir should be created");
    fs::write(root.path().join("private-canary"), "filesystem canary").unwrap();

    for action in ["status", "doctor"] {
        let mut request = request(action);
        request.harness = Some("codex".to_string());
        request.root = Some(root.path().display().to_string());
        request.scope = None;
        let response = tools::harness_new(&ToolState::new(), request)
            .await
            .unwrap_or_else(|error| {
                panic!("{action} should abstain before harness inspection: {error}")
            });
        let json = parse_json(&response);
        assert_eq!(json["executed"], false, "unexpected response for {action}");
        assert_eq!(json["relevance_mode"], "local");
        assert_eq!(json["authorization_scope_enforced"], true);
        assert_eq!(json["omitted_layers"], serde_json::json!(["harness"]));
        assert!(!response.contains("filesystem canary"));
        assert!(json.get("host").is_none());
        assert!(json.get("adapters").is_none());
    }

    let error = tools::harness_new(&ToolState::new(), request("unknown"))
        .await
        .expect_err("unknown actions should be rejected before scope handling");
    assert!(error.contains("Unknown action"));
}

#[tokio::test]
async fn mcp_harness_related_scope_requires_a_checkout_owned_by_the_project() {
    let root = tempdir().expect("tempdir should be created");
    let unregistered_root = tempdir().expect("unregistered tempdir should be created");
    let adapter = root
        .path()
        .join(".codex/skills/engram-memory-session/SKILL.md");
    fs::create_dir_all(adapter.parent().unwrap()).unwrap();
    fs::write(&adapter, "user-owned harness canary").unwrap();
    let state = setup_related_harness_state(root.path()).await;

    let mut status = request("status");
    status.harness = Some("codex".to_string());
    status.root = Some(root.path().display().to_string());
    status.scope = related_scope("alpha");
    let status = parse_json(&tools::harness_new(&state, status).await.unwrap());
    assert_eq!(status["relevance_mode"], "related");
    assert_eq!(status["resolved_project"], "alpha");
    assert_eq!(status["authorization_scope_enforced"], true);
    assert_eq!(status["root"], root.path().display().to_string());
    assert!(status["adapters"].as_array().unwrap().iter().any(|check| {
        check["path"] == adapter.display().to_string() && check["status"] == "user_owned"
    }));

    let mut doctor = request("doctor");
    doctor.harness = Some("codex".to_string());
    doctor.root = Some(root.path().display().to_string());
    doctor.scope = related_task_scope("alpha", "ALPHA-1");
    let doctor = parse_json(&tools::harness_new(&state, doctor).await.unwrap());
    assert_eq!(doctor["resolved_project"], "alpha");
    assert_eq!(doctor["resolved_task"], "alpha-one");
    assert!(doctor["adapters"].is_array());

    for (scope, requested_root) in [
        (
            related_scope("beta"),
            Some(root.path().display().to_string()),
        ),
        (related_scope("alpha"), None),
        (
            related_scope("alpha"),
            Some(unregistered_root.path().display().to_string()),
        ),
    ] {
        let mut request = request("status");
        request.harness = Some("codex".to_string());
        request.root = requested_root;
        request.scope = scope;
        let response = tools::harness_new(&state, request).await.unwrap();
        let json = parse_json(&response);
        assert_eq!(json["executed"], false);
        assert_eq!(json["relevance_mode"], "related");
        assert_eq!(json["omitted_layers"], serde_json::json!(["harness"]));
        assert!(json.get("host").is_none());
        assert!(json.get("adapters").is_none());
    }
    assert_eq!(
        fs::read_to_string(adapter).unwrap(),
        "user-owned harness canary"
    );
}

#[tokio::test]
async fn mcp_harness_hook_resolves_canonical_project_from_registered_cwd() {
    let root = tempdir().expect("tempdir should be created");
    let state = setup_related_harness_state(root.path()).await;

    let mut hook = request("hook_event");
    hook.hook_event_name = Some("SessionEnd".to_string());
    hook.cwd = Some(root.path().display().to_string());
    hook.write_policy = Some("durable".to_string());
    tools::harness_new(&state, hook)
        .await
        .expect("registered cwd should resolve for SessionEnd");

    let handoff_guard = state.handoff_service.read().await;
    let handoff = handoff_guard.as_ref().expect("handoff service");
    let item = handoff
        .get(Some("alpha"), None)
        .await
        .unwrap()
        .item
        .expect("SessionEnd should write an alpha-scoped handoff");
    assert!(matches!(
        item.scope,
        engram_core::memory::MemoryScope::Project { project_name, .. }
            if project_name == "alpha"
    ));

    let mut mismatched = request("hook_event");
    mismatched.hook_event_name = Some("SessionEnd".to_string());
    mismatched.cwd = Some(root.path().display().to_string());
    mismatched.project = Some("beta".to_string());
    mismatched.write_policy = Some("durable".to_string());
    let error = tools::harness_new(&state, mismatched)
        .await
        .expect_err("explicit project must not override canonical cwd topology");
    assert!(error.contains("does not match canonical project 'alpha'"));
}

fn handoff_request(action: &str) -> HandoffRequest {
    HandoffRequest {
        action: action.to_string(),
        project: None,
        scope: Some(RetrievalScopeRequest {
            relevance_mode: Some("global".to_string()),
            ..RetrievalScopeRequest::default()
        }),
        session_id: None,
        content: None,
        next_actions: Vec::new(),
        dry_run: None,
        writer_harness: Some("codex".to_string()),
        model_provider: Some("openai".to_string()),
        model: Some("gpt-5.5".to_string()),
        surface: Some("desktop".to_string()),
    }
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

#[tokio::test]
async fn test_mcp_harness_render_policy_requires_compact_agent_tools() {
    let state = ToolState::new();
    let mut render = request("render_policy");
    render.harness = Some("codex".to_string());

    let response = tools::harness_new(&state, render)
        .await
        .expect("render_policy should work");
    let json = parse_json(&response);
    assert_eq!(json["enforcement_profile"], "soft");
    assert_eq!(json["soft_contract"], true);
    let tools = json["required_mcp_tools"]
        .as_array()
        .expect("required_mcp_tools should be an array");

    assert_eq!(
        tools,
        &[
            Value::String("orient".to_string()),
            Value::String("memory".to_string()),
            Value::String("repo".to_string()),
            Value::String("search".to_string()),
            Value::String("harness".to_string()),
            Value::String("obligations".to_string()),
        ]
    );
}

#[tokio::test]
async fn test_mcp_harness_render_claude_session_end_hook_defaults_to_nudge() {
    let state = ToolState::new();
    let mut render = request("render_adapter");
    render.harness = Some("claude_code".to_string());
    render.adapter = Some("claude-session-end-hook".to_string());

    let response = tools::harness_new(&state, render)
        .await
        .expect("render_adapter should work");
    let json = parse_json(&response);

    assert_eq!(json["count"], 1);
    let contents = json["adapters"][0]["contents"]
        .as_str()
        .expect("rendered adapter should include contents");
    assert!(contents.contains(r#".write_policy // "nudge""#));
    assert!(!contents.contains(r#".write_policy // "durable""#));
}

#[tokio::test]
async fn test_mcp_harness_render_claude_settings_defaults_to_low_overhead_hooks() {
    let state = ToolState::new();
    let mut render = request("render_adapter");
    render.harness = Some("claude_code".to_string());
    render.adapter = Some("claude-settings-snippet".to_string());

    let response = tools::harness_new(&state, render)
        .await
        .expect("render_adapter should work");
    let json = parse_json(&response);
    let contents = json["adapters"][0]["contents"]
        .as_str()
        .expect("rendered adapter should include contents");
    let settings: Value = serde_json::from_str(contents).expect("snippet should be valid JSON");

    assert!(settings.pointer("/hooks/SessionStart").is_some());
    assert!(settings.pointer("/hooks/PreCompact").is_some());
    assert!(settings.pointer("/hooks/PostCompact").is_some());
    assert!(settings.pointer("/hooks/SessionEnd").is_some());
    assert!(settings.pointer("/hooks/UserPromptSubmit").is_none());
    assert!(settings.pointer("/hooks/PreToolUse").is_none());
    assert!(settings.pointer("/hooks/PostToolUse").is_none());
    assert!(settings.pointer("/hooks/PostToolUseFailure").is_none());
    assert!(settings.pointer("/hooks/Stop").is_none());
}

#[tokio::test]
async fn test_mcp_harness_render_claude_settings_includes_graduated_hooks_when_requested() {
    let state = ToolState::new();
    let mut render = request("render_adapter");
    render.harness = Some("claude_code".to_string());
    render.adapter = Some("claude-settings-snippet".to_string());
    render.enforcement = Some("graduated".to_string());

    let response = tools::harness_new(&state, render)
        .await
        .expect("render_adapter should work");
    let json = parse_json(&response);
    let contents = json["adapters"][0]["contents"]
        .as_str()
        .expect("rendered adapter should include contents");
    let settings: Value = serde_json::from_str(contents).expect("snippet should be valid JSON");

    assert!(settings.pointer("/hooks/PreToolUse").is_some());
    assert!(settings.pointer("/hooks/PostToolUseFailure").is_some());
    assert!(settings.pointer("/hooks/Stop").is_some());
    assert_eq!(
        settings["hooks"]["PreToolUse"][0]["hooks"][0]["input"]["enforcement"],
        "graduated"
    );
}

#[tokio::test]
async fn test_mcp_harness_render_adapter_uses_bounded_agent_workflow() {
    let state = ToolState::new();
    let mut render = request("render_adapter");
    render.harness = Some("codex".to_string());
    render.adapter = Some("codex-memory-session-skill".to_string());

    let response = tools::harness_new(&state, render)
        .await
        .expect("render_adapter should work");
    let json = parse_json(&response);

    assert_eq!(json["count"], 1);
    let contents = json["adapters"][0]["contents"]
        .as_str()
        .expect("rendered adapter should include contents");
    assert!(contents.contains("`orient`"));
    assert!(contents.contains("search(query=..."));
    assert!(contents.contains("memory(action=procedure_match"));
    assert!(contents.contains("memory(action=add)"));
    assert!(!contents.contains("trace_id"));
    assert!(!contents.contains("telemetry(action="));
    assert!(contents.contains("soft profile"));
    assert!(contents.contains("response_shape=\"lean\""));
}

#[tokio::test]
async fn test_mcp_harness_doctor_returns_structured_lifecycle_report() {
    let state = ToolState::new();
    let root = tempdir().expect("tempdir should be created");

    let mut install = request("install");
    install.harness = Some("codex".to_string());
    install.root = Some(root.path().display().to_string());
    install.write = Some(true);
    tools::harness_new(&state, install)
        .await
        .expect("install should work");

    let mut doctor = request("doctor");
    doctor.harness = Some("codex".to_string());
    doctor.root = Some(root.path().display().to_string());
    let response = tools::harness_new(&state, doctor)
        .await
        .expect("doctor should work");
    let json = parse_json(&response);

    assert_eq!(json["ready"], true);
    assert_eq!(json["lifecycle"]["enforcement_profile"], "soft");
    assert_eq!(json["lifecycle"]["soft_contract"], true);
    assert_eq!(json["lifecycle"]["enforced"], false);
    assert_eq!(json["mcp_tools"]["checked"], false);
    assert_eq!(json["mcp_server"]["checked"], false);
    assert_eq!(json["host"]["checked"], true);
    assert_eq!(json["host"]["effective_configuration_verified"], false);
    assert!(json["mcp_tools"]["missing_tools"]
        .as_array()
        .expect("missing_tools should be an array")
        .is_empty());
    let triggers = json["lifecycle"]["advisory_triggers"]
        .as_array()
        .expect("advisory_triggers should be an array");
    assert!(triggers
        .iter()
        .any(|trigger| trigger.as_str() == Some("task_start_orient")));
    assert!(triggers
        .iter()
        .any(|trigger| trigger.as_str() == Some("before_final_obligations")));
    assert!(json["lifecycle"]["message"]
        .as_str()
        .expect("lifecycle message should be a string")
        .contains("low-overhead"));
    for adapter in json["adapters"]
        .as_array()
        .expect("adapters should be an array")
    {
        assert_eq!(adapter["expected_sha256"], adapter["actual_sha256"]);
    }
}

#[tokio::test]
async fn test_mcp_harness_status_reports_missing_observed_mcp_tools() {
    let state = ToolState::new();
    let root = tempdir().expect("tempdir should be created");

    let mut install = request("install");
    install.harness = Some("codex".to_string());
    install.root = Some(root.path().display().to_string());
    install.write = Some(true);
    tools::harness_new(&state, install)
        .await
        .expect("install should work");

    let mut status = request("status");
    status.harness = Some("codex".to_string());
    status.root = Some(root.path().display().to_string());
    status.observed_mcp_tools = vec![
        "orient".to_string(),
        "memory".to_string(),
        "harness".to_string(),
        "lint".to_string(),
        "graph".to_string(),
        "handoff".to_string(),
        "obligations".to_string(),
        "vault".to_string(),
    ];
    let response = tools::harness_new(&state, status)
        .await
        .expect("status should work");
    let json = parse_json(&response);

    assert_eq!(json["ready"], false);
    assert_eq!(json["mcp_tools"]["checked"], true);
    assert_eq!(
        json["mcp_tools"]["missing_tools"],
        serde_json::json!(["repo", "search"])
    );
    assert_eq!(
        json["missing_mcp_tools"],
        serde_json::json!(["repo", "search"])
    );
    let warnings = json["warnings"]
        .as_array()
        .expect("warnings should be an array");
    for missing in ["repo", "search"] {
        assert!(warnings.iter().any(|warning| {
            warning
                .as_str()
                .is_some_and(|warning| warning.contains(&format!("'{missing}'")))
        }));
    }
}

#[tokio::test]
async fn test_mcp_harness_claude_ready_warns_effective_hooks_need_live_hooks_proof() {
    let state = ToolState::new();
    let root = tempdir().expect("tempdir should be created");

    let mut install = request("install");
    install.harness = Some("claude_code".to_string());
    install.root = Some(root.path().display().to_string());
    install.write = Some(true);
    tools::harness_new(&state, install)
        .await
        .expect("install should work");

    let mut status = request("status");
    status.harness = Some("claude_code".to_string());
    status.root = Some(root.path().display().to_string());
    let response = tools::harness_new(&state, status)
        .await
        .expect("status should work");
    let json = parse_json(&response);

    assert_eq!(json["ready"], true);
    let warnings = json["warnings"]
        .as_array()
        .expect("warnings should be an array");
    assert!(warnings.iter().any(|warning| {
        let warning = warning.as_str().expect("warning should be a string");
        warning.contains("does not prove live effective hook visibility")
            && warning.contains("Claude Code /hooks")
    }));
}

#[tokio::test]
async fn test_mcp_harness_hook_event_returns_claude_hook_json() {
    let state = setup_tool_state().await;

    let mut hook = request("hook_event");
    hook.hook_event_name = Some("PostToolUseFailure".to_string());
    hook.cwd = Some("/tmp/engram".to_string());
    hook.tool_name = Some("mcp__engram__memory".to_string());
    hook.tool_error = Some("invalid type: string, expected struct".to_string());
    hook.write_policy = Some("durable".to_string());
    hook.model_provider = Some("anthropic".to_string());
    hook.model = Some("claude-code".to_string());
    hook.surface = Some("claude-code".to_string());
    hook.actor = Some("agent".to_string());

    let response = tools::harness_new(&state, hook)
        .await
        .expect("hook_event should work");
    let json = parse_json(&response);

    assert_eq!(json["continue"], true);
    assert!(json.get("hookSpecificOutput").is_none());
    assert!(json["systemMessage"]
        .as_str()
        .unwrap()
        .contains("memory_written=1"));
}

#[tokio::test]
async fn test_mcp_harness_pre_tool_use_denies_until_orient_runs() {
    let state = setup_tool_state().await;

    let mut prompt = request("hook_event");
    prompt.enforcement = Some("graduated".to_string());
    prompt.hook_event_name = Some("UserPromptSubmit".to_string());
    prompt.cwd = Some("/tmp/engram".to_string());
    prompt.prompt = Some("Inspect the repository and report GA release status".to_string());
    prompt.write_policy = Some("durable".to_string());
    tools::harness_new(&state, prompt)
        .await
        .expect("prompt hook should work");

    let mut denied = request("hook_event");
    denied.enforcement = Some("graduated".to_string());
    denied.hook_event_name = Some("PreToolUse".to_string());
    denied.cwd = Some("/tmp/engram".to_string());
    denied.tool_name = Some("Bash".to_string());
    let denied_response = tools::harness_new(&state, denied)
        .await
        .expect("pretool hook should work");
    let denied_json = parse_json(&denied_response);
    assert_eq!(
        denied_json["hookSpecificOutput"]["permissionDecision"],
        "deny"
    );
    assert!(
        denied_json["hookSpecificOutput"]["permissionDecisionReason"]
            .as_str()
            .unwrap()
            .contains("response_shape=\"lean\"")
    );

    let mut orient = request("hook_event");
    orient.enforcement = Some("graduated".to_string());
    orient.hook_event_name = Some("PostToolUse".to_string());
    orient.cwd = Some("/tmp/engram".to_string());
    orient.tool_name = Some("mcp__engram__orient".to_string());
    tools::harness_new(&state, orient)
        .await
        .expect("orient posttool hook should work");

    let mut allowed = request("hook_event");
    allowed.enforcement = Some("graduated".to_string());
    allowed.hook_event_name = Some("PreToolUse".to_string());
    allowed.cwd = Some("/tmp/engram".to_string());
    allowed.tool_name = Some("Bash".to_string());
    let allowed_response = tools::harness_new(&state, allowed)
        .await
        .expect("pretool hook should work after orient");
    let allowed_json = parse_json(&allowed_response);
    assert_eq!(allowed_json["continue"], true);
    assert!(allowed_json
        .get("hookSpecificOutput")
        .and_then(|output| output.get("permissionDecision"))
        .is_none());
}

#[tokio::test]
async fn mcp_handoff_reads_abstain_locally_before_service_access() {
    let state = ToolState::new();

    for action in ["get", "compile"] {
        let mut request = handoff_request(action);
        request.scope = None;
        let response = tools::handoff_new(&state, request)
            .await
            .unwrap_or_else(|error| {
                panic!("{action} should abstain before service access: {error}")
            });
        let json = parse_json(&response);
        assert_eq!(json["executed"], false, "unexpected response for {action}");
        assert_eq!(json["relevance_mode"], "local");
        assert_eq!(json["authorization_scope_enforced"], true);
        assert_eq!(json["omitted_layers"], serde_json::json!(["handoff"]));
    }
}

#[tokio::test]
async fn mcp_handoff_related_scope_enforces_project_and_session_ownership() {
    let (state, _) = setup_tool_state_with_memory().await;
    {
        let guard = state.work_service.read().await;
        let work = guard.as_ref().expect("work service");
        work.create_project("alpha", None).await.unwrap();
        work.create_project("beta", None).await.unwrap();
        work.create_task("alpha", "alpha-one", None, Some("ALPHA-1"))
            .await
            .unwrap();
    }

    let (alpha_session, beta_session, unowned_session) = {
        let guard = state.session_service.read().await;
        let sessions = guard.as_ref().expect("session service");
        let alpha = sessions
            .start_session(Some("codex"), Some("alpha"), Some("alpha work"))
            .await
            .unwrap();
        let beta = sessions
            .start_session(Some("codex"), Some("beta"), Some("beta work"))
            .await
            .unwrap();
        let unowned = sessions
            .start_session(Some("codex"), None, Some("unowned work"))
            .await
            .unwrap();
        sessions
            .log_event(
                &alpha.id,
                EventType::Decision,
                "alpha session decision",
                None,
                Some("test"),
            )
            .await
            .unwrap();
        sessions
            .log_event(
                &beta.id,
                EventType::Decision,
                "beta session decision",
                None,
                Some("test"),
            )
            .await
            .unwrap();
        (alpha, beta, unowned)
    };

    for (project, content) in [
        ("alpha", "# Alpha Project Handoff"),
        ("beta", "# Beta Project Handoff"),
    ] {
        let mut update = handoff_request("update");
        update.project = Some(project.to_string());
        update.content = Some(content.to_string());
        update.dry_run = Some(false);
        tools::handoff_new(&state, update).await.unwrap();
    }
    for (session_id, content) in [
        (alpha_session.id, "# Alpha Session Handoff"),
        (beta_session.id, "# Beta Session Handoff"),
        (unowned_session.id, "# Unowned Session Handoff"),
    ] {
        let mut update = handoff_request("update");
        update.session_id = Some(session_id.to_string());
        update.content = Some(content.to_string());
        update.dry_run = Some(false);
        tools::handoff_new(&state, update).await.unwrap();
    }

    let mut project_get = handoff_request("get");
    project_get.scope = related_scope("alpha");
    let project_get = parse_json(&tools::handoff_new(&state, project_get).await.unwrap());
    assert!(project_get["item"]["content"]
        .as_str()
        .unwrap()
        .contains("Alpha Project Handoff"));
    assert_eq!(project_get["resolved_project"], "alpha");

    let mut alpha_get = handoff_request("get");
    alpha_get.scope = related_scope("alpha");
    alpha_get.session_id = Some(alpha_session.id.to_string());
    let alpha_get = parse_json(&tools::handoff_new(&state, alpha_get).await.unwrap());
    assert!(alpha_get["item"]["content"]
        .as_str()
        .unwrap()
        .contains("Alpha Session Handoff"));

    for hidden_session in [beta_session.id, unowned_session.id] {
        let mut get = handoff_request("get");
        get.scope = related_scope("alpha");
        get.session_id = Some(hidden_session.to_string());
        let get = parse_json(&tools::handoff_new(&state, get).await.unwrap());
        assert!(get["item"].is_null());
    }

    let mut alpha_compile = handoff_request("compile");
    alpha_compile.scope = related_scope("alpha");
    alpha_compile.session_id = Some(alpha_session.id.to_string());
    let alpha_compile = parse_json(&tools::handoff_new(&state, alpha_compile).await.unwrap());
    assert!(alpha_compile["content"]
        .as_str()
        .unwrap()
        .contains("alpha session decision"));
    assert_eq!(alpha_compile["resolved_project"], "alpha");

    let mut hidden_compile = handoff_request("compile");
    hidden_compile.scope = related_scope("alpha");
    hidden_compile.session_id = Some(beta_session.id.to_string());
    let hidden_compile = parse_json(&tools::handoff_new(&state, hidden_compile).await.unwrap());
    assert_eq!(hidden_compile["executed"], false);
    assert!(hidden_compile.get("content").is_none());

    let mut exact_task = handoff_request("get");
    exact_task.scope = related_task_scope("alpha", "ALPHA-1");
    let exact_task = parse_json(&tools::handoff_new(&state, exact_task).await.unwrap());
    assert_eq!(exact_task["executed"], false);
    assert_eq!(exact_task["resolved_task"], "alpha-one");
    assert_eq!(exact_task["omitted_layers"], serde_json::json!(["handoff"]));

    let mut conflicting_project = handoff_request("get");
    conflicting_project.scope = related_scope("alpha");
    conflicting_project.project = Some("beta".to_string());
    let error = tools::handoff_new(&state, conflicting_project)
        .await
        .expect_err("conflicting project target should fail");
    assert!(error.contains("does not match resolved authorization project 'alpha'"));

    let mut global = handoff_request("get");
    global.session_id = Some(beta_session.id.to_string());
    let global = parse_json(&tools::handoff_new(&state, global).await.unwrap());
    assert!(global["item"]["content"]
        .as_str()
        .unwrap()
        .contains("Beta Session Handoff"));
    assert_eq!(global["relevance_mode"], "global");
}

#[tokio::test]
async fn test_mcp_handoff_update_supersedes_previous_handoff() {
    let (state, memory) = setup_tool_state_with_memory().await;

    let mut first = handoff_request("update");
    first.project = Some("engram".to_string());
    first.content = Some("# First Handoff".to_string());
    first.next_actions = vec!["Continue from first handoff".to_string()];
    first.dry_run = Some(false);
    let first_response = tools::handoff_new(&state, first)
        .await
        .expect("first handoff update should work");
    let first_json = parse_json(&first_response);
    let first_id = first_json["item"]["id"]
        .as_str()
        .expect("first update should return item id")
        .to_string();
    let first_id = Id::parse(&first_id).expect("first item id should parse");

    let mut second = handoff_request("update");
    second.project = Some("engram".to_string());
    second.content = Some("# Second Handoff".to_string());
    second.next_actions = vec!["Continue from second handoff".to_string()];
    second.dry_run = Some(false);
    let second_response = tools::handoff_new(&state, second)
        .await
        .expect("second handoff update should work");
    let second_json = parse_json(&second_response);

    let stored_first = memory
        .get_memory(&first_id)
        .await
        .expect("memory get should work")
        .expect("first handoff should remain stored");

    assert_eq!(second_json["previous_id"], first_json["item"]["id"]);
    assert_eq!(second_json["item"]["status"], "active");
    assert_eq!(
        second_json["item"]["supersedes"][0],
        first_json["item"]["id"]
    );
    assert_eq!(stored_first.status.to_string(), "superseded");
}
