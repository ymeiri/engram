//! Integration tests for Memory OS harness MCP tooling.

use engram_index::{HandoffService, MemoryService, ObligationService};
use engram_mcp::tools::{self, HarnessRequest, ToolState};
use engram_store::{connect_and_init, StoreConfig};
use serde_json::Value;

async fn setup_tool_state() -> ToolState {
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
    state.init_memory(memory).await;
    state.init_obligation(obligations).await;
    state.init_handoff(handoff).await;
    state
}

fn request(action: &str) -> HarnessRequest {
    HarnessRequest {
        action: action.to_string(),
        harness: Some("claude_code".to_string()),
        root: None,
        adapter: None,
        write: None,
        adopt_user_owned: None,
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
