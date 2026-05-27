//! Integration tests for Memory OS lint MCP tooling.

use engram_core::memory::{
    ClaimOrigin, EvidenceKind, EvidenceRef, Harness, MemoryItem, MemoryKind, MemoryScope,
    ModelIdentity, WriterProvenance,
};
use engram_index::{LintService, MemoryService};
use engram_mcp::tools::{self, LintRequest, ToolState};
use engram_store::{connect_and_init, StoreConfig};
use serde_json::Value;

async fn setup_tool_state() -> (ToolState, MemoryService) {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config).await.expect("Failed to connect");

    let memory_service = MemoryService::new(db.clone());
    memory_service
        .init_schema()
        .await
        .expect("Failed to initialize memory schema");

    let lint_service = LintService::new(db);
    lint_service
        .init_schema()
        .await
        .expect("Failed to initialize lint schema");

    let state = ToolState::new();
    state.init_lint(lint_service).await;
    (state, memory_service)
}

fn lint_request(action: &str) -> LintRequest {
    LintRequest {
        action: action.to_string(),
        vault_path: None,
        limit: None,
        write: None,
    }
}

fn writer() -> WriterProvenance {
    WriterProvenance::agent(Harness::Codex, ModelIdentity::new("openai", "gpt-5.5"))
        .with_surface("test")
}

fn parse_json(response: &str) -> Value {
    serde_json::from_str(response).expect("response should be valid JSON")
}

#[tokio::test]
async fn test_mcp_lint_bounds_duplicate_entity_candidate_messages() {
    let (state, memory_service) = setup_tool_state().await;
    let mut item_ids = Vec::new();

    for index in 0..10 {
        let item = MemoryItem::new(
            MemoryKind::ProjectFact,
            format!("Duplicate entity fact {index}"),
            "Duplicate entity-scoped content.",
            MemoryScope::entity("ide-mcp-eval"),
            ClaimOrigin::AgentObserved,
            writer(),
        )
        .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "lint_tests"));
        item_ids.push(item.id);
        memory_service
            .capture_memory(item)
            .await
            .expect("memory item should be captured");
    }

    let response = tools::lint_new(&state, lint_request("run"))
        .await
        .expect("lint should run");
    let json = parse_json(&response);
    let finding = json["findings"]
        .as_array()
        .expect("findings should be an array")
        .iter()
        .find(|finding| finding["rule"] == "duplicate_entity_candidate")
        .expect("duplicate entity finding should be present");
    let message = finding["message"]
        .as_str()
        .expect("finding should include a message");

    let displayed_id_count = item_ids
        .iter()
        .filter(|item_id| message.contains(&item_id.to_string()))
        .count();

    assert!(message.contains("10 active items"));
    assert!(message.contains("... (2 more)"));
    assert_eq!(displayed_id_count, 8);
}
