//! Integration tests for Memory OS lint MCP tooling.

use engram_core::memory::{
    ClaimOrigin, EvidenceKind, EvidenceRef, Harness, MemoryItem, MemoryKind, MemoryScope,
    ModelIdentity, WriterProvenance,
};
use engram_core::telemetry::AgentFeedback;
use engram_core::Id;
use engram_index::{LintService, MemoryService};
use engram_mcp::tools::{self, LintRequest, ToolState};
use engram_store::{connect_and_init, StoreConfig, TelemetryRepo};
use serde_json::Value;

async fn setup_tool_state() -> (ToolState, MemoryService, TelemetryRepo) {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config).await.expect("Failed to connect");

    let memory_service = MemoryService::new(db.clone());
    memory_service
        .init_schema()
        .await
        .expect("Failed to initialize memory schema");

    let lint_service = LintService::new(db.clone());
    lint_service
        .init_schema()
        .await
        .expect("Failed to initialize lint schema");
    let telemetry_repo = TelemetryRepo::new(db);

    let state = ToolState::new();
    state.init_lint(lint_service).await;
    (state, memory_service, telemetry_repo)
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
    let (state, memory_service, _) = setup_tool_state().await;
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

#[tokio::test]
async fn test_mcp_lint_includes_item_titles_for_actionable_warnings() {
    let (state, memory_service, _) = setup_tool_state().await;

    let missing = MemoryItem::new(
        MemoryKind::Decision,
        "Missing source citation",
        "A durable decision without evidence.",
        MemoryScope::project("engram"),
        ClaimOrigin::UserStated,
        writer(),
    );
    memory_service
        .capture_memory(missing)
        .await
        .expect("missing-evidence item should be captured");

    let handoff = MemoryItem::new(
        MemoryKind::Handoff,
        "Incomplete handoff",
        "Useful context without an explicit action list.",
        MemoryScope::project("engram"),
        ClaimOrigin::AgentObserved,
        writer(),
    )
    .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "lint_tests"));
    memory_service
        .capture_memory(handoff)
        .await
        .expect("handoff item should be captured");

    let response = tools::lint_new(&state, lint_request("run"))
        .await
        .expect("lint should run");
    let json = parse_json(&response);
    let findings = json["findings"]
        .as_array()
        .expect("findings should be an array");
    let missing_message = findings
        .iter()
        .find(|finding| finding["rule"] == "missing_evidence")
        .and_then(|finding| finding["message"].as_str())
        .expect("missing evidence finding should include a message");
    let handoff_message = findings
        .iter()
        .find(|finding| finding["rule"] == "handoff_missing_next_actions")
        .and_then(|finding| finding["message"].as_str())
        .expect("handoff finding should include a message");

    assert!(missing_message.contains("Missing source citation"));
    assert!(missing_message.contains("decision"));
    assert!(handoff_message.contains("Incomplete handoff"));
}

#[tokio::test]
async fn test_mcp_lint_reports_feedback_flagged_active_memory() {
    let (state, memory_service, telemetry_repo) = setup_tool_state().await;

    let item = MemoryItem::new(
        MemoryKind::ProjectFact,
        "Telemetry-questioned fact",
        "Content that feedback says may be stale.",
        MemoryScope::project("engram"),
        ClaimOrigin::AgentObserved,
        writer(),
    )
    .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "lint_tests"));
    let item_id = item.id;
    memory_service
        .capture_memory(item)
        .await
        .expect("memory item should be captured");

    let mut feedback = AgentFeedback::new(Id::new());
    feedback.stale_memory_ids = vec![item_id];
    feedback.wrong_scope_memory_ids = vec![item_id];
    telemetry_repo
        .save_feedback(&feedback)
        .await
        .expect("feedback should be saved");

    let response = tools::lint_new(&state, lint_request("run"))
        .await
        .expect("lint should run");
    let json = parse_json(&response);
    let findings = json["findings"]
        .as_array()
        .expect("findings should be an array");

    let stale_finding = findings
        .iter()
        .find(|finding| finding["rule"] == "feedback_stale_active_memory")
        .expect("stale feedback finding should be present");
    let wrong_scope_finding = findings
        .iter()
        .find(|finding| finding["rule"] == "feedback_wrong_scope_active_memory")
        .expect("wrong-scope feedback finding should be present");

    assert_eq!(stale_finding["severity"], "info");
    assert_eq!(stale_finding["safe_action"], "none");
    assert_eq!(stale_finding["item_id"], item_id.to_string());
    assert_eq!(wrong_scope_finding["item_id"], item_id.to_string());
}

#[tokio::test]
async fn test_mcp_lint_reports_stale_current_plan_feedback() {
    let (state, memory_service, telemetry_repo) = setup_tool_state().await;

    let item = MemoryItem::new(
        MemoryKind::Decision,
        "Current plan after older slice",
        "Old current-plan guidance that feedback says is stale.",
        MemoryScope::project("engram"),
        ClaimOrigin::AgentObserved,
        writer(),
    )
    .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "lint_tests"))
    .with_tag("current-plan");
    let item_id = item.id;
    memory_service
        .capture_memory(item)
        .await
        .expect("memory item should be captured");

    let mut feedback = AgentFeedback::new(Id::new());
    feedback.stale_memory_ids = vec![item_id];
    telemetry_repo
        .save_feedback(&feedback)
        .await
        .expect("feedback should be saved");

    let response = tools::lint_new(&state, lint_request("run"))
        .await
        .expect("lint should run");
    let json = parse_json(&response);
    let findings = json["findings"]
        .as_array()
        .expect("findings should be an array");

    let finding = findings
        .iter()
        .find(|finding| finding["rule"] == "feedback_stale_current_plan")
        .expect("stale current-plan feedback finding should be present");

    assert_eq!(finding["severity"], "info");
    assert_eq!(finding["safe_action"], "none");
    assert_eq!(finding["item_id"], item_id.to_string());
    assert!(finding["message"]
        .as_str()
        .expect("finding should include a message")
        .contains("Current plan after older slice"));
    assert!(!findings.iter().any(|finding| {
        finding["rule"] == "feedback_stale_active_memory"
            && finding["item_id"] == item_id.to_string()
    }));
}
