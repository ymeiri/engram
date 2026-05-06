//! Integration tests for agent obligation MCP tooling.

use engram_index::{MemoryService, ObligationService};
use engram_mcp::tools::{self, MemoryEvidenceRequest, ObligationRequest, OrientRequest, ToolState};
use engram_store::{connect_and_init, StoreConfig};
use serde_json::Value;

async fn setup_tool_state() -> ToolState {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config).await.expect("Failed to connect");
    let memory_service = MemoryService::new(db.clone());
    memory_service
        .init_schema()
        .await
        .expect("Failed to initialize memory schema");
    let obligation_service = ObligationService::new(db);
    obligation_service
        .init_schema()
        .await
        .expect("Failed to initialize obligation schema");

    let state = ToolState::new();
    state.init_memory(memory_service).await;
    state.init_obligation(obligation_service).await;
    state
}

fn request(action: &str) -> ObligationRequest {
    ObligationRequest {
        action: action.to_string(),
        cwd: None,
        prompt: None,
        project: None,
        limit: None,
        write: None,
        id: None,
        kind: None,
        title: None,
        description: None,
        status: None,
        trigger_kind: None,
        trigger_target: None,
        trigger_summary: None,
        required_resolutions: Vec::new(),
        resolution: None,
        summary: None,
        reason: None,
        evidence: Vec::new(),
        writer_harness: None,
        model_provider: None,
        model: None,
        surface: None,
        actor: None,
        writer_session_id: None,
    }
}

fn with_writer(mut req: ObligationRequest) -> ObligationRequest {
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
async fn test_mcp_obligations_detect_write_and_doctor() {
    let state = setup_tool_state().await;

    let mut detect = with_writer(request("detect"));
    detect.project = Some("engram".to_string());
    detect.write = Some(true);
    detect.prompt = Some(
        "Implement the design after a failed tool call due to wrong parameters and read the source"
            .to_string(),
    );

    let response = tools::obligations_new(&state, detect)
        .await
        .expect("detect should work");
    let json = parse_json(&response);

    assert_eq!(json["dry_run"], false);
    let written = json["written"].as_array().unwrap();
    assert!(written.iter().any(|item| item["kind"] == "source_reading"));
    assert!(written
        .iter()
        .any(|item| item["kind"] == "design_context_reading"));
    assert!(written
        .iter()
        .any(|item| item["kind"] == "tool_failure_recovery"));

    let doctor_response = tools::obligations_new(&state, request("doctor"))
        .await
        .expect("doctor should work");
    let doctor = parse_json(&doctor_response);

    assert_eq!(doctor["open"].as_array().unwrap().len(), 3);
    assert!(doctor["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .all(|warning| warning
            .as_str()
            .unwrap()
            .contains("must be resolved or skipped")));
}

#[tokio::test]
async fn test_mcp_obligations_add_resolve_and_skip() {
    let state = setup_tool_state().await;

    let mut add = with_writer(request("add"));
    add.project = Some("engram".to_string());
    add.kind = Some("document_disposition".to_string());
    add.title = Some("Resolve test document".to_string());
    add.description = Some("A durable document needs a memory decision.".to_string());
    add.trigger_kind = Some("test".to_string());
    add.trigger_target = Some("docs/example.md".to_string());
    add.trigger_summary = Some("test document changed".to_string());
    add.required_resolutions = vec![
        "indexed_document".to_string(),
        "skipped_with_reason".to_string(),
    ];
    add.evidence = vec![MemoryEvidenceRequest {
        kind: "file".to_string(),
        target: "docs/example.md".to_string(),
        summary: Some("test evidence".to_string()),
        excerpt: None,
    }];

    let add_response = tools::obligations_new(&state, add)
        .await
        .expect("add should work");
    let add_json = parse_json(&add_response);
    let id = add_json["id"].as_str().unwrap().to_string();
    assert_eq!(add_json["status"], "open");

    let mut resolve = request("resolve");
    resolve.id = Some(id.clone());
    resolve.resolution = Some("indexed_document".to_string());
    resolve.summary = Some("Indexed the document.".to_string());
    resolve.actor = Some("agent".to_string());
    let resolve_response = tools::obligations_new(&state, resolve)
        .await
        .expect("resolve should work");
    let resolved = parse_json(&resolve_response);
    assert_eq!(resolved["id"], id);
    assert_eq!(resolved["status"], "resolved");
    assert_eq!(resolved["resolution"]["kind"], "indexed_document");

    let mut add_second = with_writer(request("add"));
    add_second.project = Some("engram".to_string());
    add_second.kind = Some("tool_failure_recovery".to_string());
    add_second.title = Some("Recover failed test tool".to_string());
    add_second.description = Some("A test tool failure needs disposition.".to_string());
    add_second.trigger_kind = Some("test".to_string());
    add_second.trigger_summary = Some("tool failed".to_string());

    let second_response = tools::obligations_new(&state, add_second)
        .await
        .expect("second add should work");
    let second_id = parse_json(&second_response)["id"]
        .as_str()
        .unwrap()
        .to_string();

    let mut skip = request("skip");
    skip.id = Some(second_id);
    skip.reason = Some("Failure was intentionally exercised by the test.".to_string());
    let skip_response = tools::obligations_new(&state, skip)
        .await
        .expect("skip should work");
    let skipped = parse_json(&skip_response);
    assert_eq!(skipped["status"], "skipped");
    assert_eq!(skipped["resolution"]["kind"], "skipped_with_reason");
}

#[tokio::test]
async fn test_mcp_orient_surfaces_open_obligations() {
    let state = setup_tool_state().await;

    let mut add = with_writer(request("add"));
    add.project = Some("engram".to_string());
    add.kind = Some("document_disposition".to_string());
    add.title = Some("Review generated design note".to_string());
    add.description =
        Some("A durable design note needs ingestion or an explicit skip.".to_string());
    add.trigger_kind = Some("test".to_string());
    add.trigger_target = Some("docs/design.md".to_string());
    add.trigger_summary = Some("design note changed".to_string());
    add.required_resolutions = vec![
        "indexed_document".to_string(),
        "memory_recorded".to_string(),
        "skipped_with_reason".to_string(),
    ];
    tools::obligations_new(&state, add)
        .await
        .expect("add should work");

    let response = tools::orient(
        &state,
        OrientRequest {
            cwd: Some("/Users/yuval.meiri/projects/engram".to_string()),
            prompt: Some("continue the Engram brain harness work".to_string()),
            project: Some("engram".to_string()),
            agent: Some("codex".to_string()),
            intent: Some("plan_work".to_string()),
            include_recent_commits: Some(false),
            limit: Some(5),
        },
    )
    .await
    .expect("orient should work");
    let orient = parse_json(&response);

    assert_eq!(orient["obligation_summary"]["available"], true);
    assert_eq!(orient["obligation_summary"]["returned_count"], 1);
    assert_eq!(
        orient["open_obligations"][0]["title"],
        "Review generated design note"
    );
    assert!(orient["recommended_actions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|action| action
            .as_str()
            .unwrap()
            .contains("open obligation(s) should be resolved")));
    assert!(orient["context_pack"]
        .as_str()
        .unwrap()
        .contains("## Open Obligations"));
}
