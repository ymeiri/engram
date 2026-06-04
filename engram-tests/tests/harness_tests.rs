//! Integration tests for Memory OS harness MCP tooling.

use engram_core::id::Id;
use engram_index::{HandoffService, MemoryService, ObligationService};
use engram_mcp::tools::{self, HandoffRequest, HarnessRequest, ToolState};
use engram_store::{connect_and_init, StoreConfig};
use serde_json::Value;

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
    let handoff = HandoffService::new(db);
    handoff
        .init_schema()
        .await
        .expect("Failed to initialize handoff schema");

    let state = ToolState::new();
    state.init_memory(memory.clone()).await;
    state.init_obligation(obligations).await;
    state.init_handoff(handoff).await;
    (state, memory)
}

fn request(action: &str) -> HarnessRequest {
    HarnessRequest {
        action: action.to_string(),
        harness: Some("claude_code".to_string()),
        root: None,
        adapter: None,
        write: None,
        adopt_user_owned: None,
        settings_target: None,
        observed_mcp_tools: Vec::new(),
        hook_event_name: None,
        session_id: None,
        cwd: None,
        transcript_path: None,
        prompt: None,
        tool_name: None,
        tool_error: None,
        tool_input_command: None,
        file_path: None,
        last_assistant_message: None,
        compact_summary: None,
        trigger: None,
        reason: None,
        stop_hook_active: None,
        write_policy: None,
        project: None,
        model_provider: None,
        model: None,
        surface: None,
        actor: None,
    }
}

fn parse_json(response: &str) -> Value {
    serde_json::from_str(response).expect("response should be valid JSON")
}

fn handoff_request(action: &str) -> HandoffRequest {
    HandoffRequest {
        action: action.to_string(),
        project: None,
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

#[tokio::test]
async fn test_mcp_harness_render_policy_requires_telemetry() {
    let state = ToolState::new();
    let mut render = request("render_policy");
    render.harness = Some("codex".to_string());

    let response = tools::harness_new(&state, render)
        .await
        .expect("render_policy should work");
    let json = parse_json(&response);
    let tools = json["required_mcp_tools"]
        .as_array()
        .expect("required_mcp_tools should be an array");

    assert!(tools.iter().any(|tool| tool.as_str() == Some("telemetry")));
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
async fn test_mcp_harness_render_adapter_mentions_feedback_trace_id() {
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
    assert!(contents.contains("trace_id"));
    assert!(contents.contains("telemetry(action=submit_feedback)"));
    assert!(contents.contains("task_success"));
    assert!(contents.contains("missing_context"));
    assert!(contents.contains("used_memory_ids"));
    assert!(contents.contains("rejected_memory_ids"));
    assert!(contents.contains("stale_memory_ids"));
    assert!(contents.contains("wrong_scope_memory_ids"));
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
