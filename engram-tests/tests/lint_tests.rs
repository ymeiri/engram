//! Integration tests for Memory OS lint MCP tooling.

use engram_core::memory::{
    ClaimOrigin, EvidenceKind, EvidenceRef, Harness, MemoryItem, MemoryKind, MemoryScope,
    ModelIdentity, WriterProvenance,
};
use engram_core::telemetry::AgentFeedback;
use engram_core::Id;
use engram_index::{LintService, MemoryService, SearchService, WorkService};
use engram_mcp::tools::{self, LintRequest, RetrievalScopeRequest, ToolState};
use engram_store::{connect_and_init, StoreConfig, TelemetryRepo};
use serde_json::Value;
use tempfile::tempdir;

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
    let telemetry_repo = TelemetryRepo::new(db.clone());
    let work = WorkService::new(db.clone());
    work.init().await.expect("Failed to initialize work schema");

    let state = ToolState::new();
    state.init_lint(lint_service).await;
    state.init_search(SearchService::new(db)).await;
    state.init_work(work).await;
    (state, memory_service, telemetry_repo)
}

fn lint_request(action: &str) -> LintRequest {
    LintRequest {
        action: action.to_string(),
        project: None,
        scope: Some(RetrievalScopeRequest {
            relevance_mode: Some("global".to_string()),
            ..RetrievalScopeRequest::default()
        }),
        vault_path: None,
        limit: None,
        write: None,
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

fn writer() -> WriterProvenance {
    WriterProvenance::agent(Harness::Codex, ModelIdentity::new("openai", "gpt-5.5"))
        .with_surface("test")
}

fn parse_json(response: &str) -> Value {
    serde_json::from_str(response).expect("response should be valid JSON")
}

#[tokio::test]
async fn mcp_lint_reads_abstain_locally_before_service_access() {
    let state = ToolState::new();

    for (action, write) in [("run", None), ("list", None), ("apply_safe", Some(true))] {
        let mut request = lint_request(action);
        request.scope = None;
        request.write = write;
        let response = tools::lint_new(&state, request)
            .await
            .unwrap_or_else(|error| {
                panic!("{action} should abstain before service access: {error}")
            });
        let json = parse_json(&response);
        assert_eq!(json["executed"], false, "unexpected response for {action}");
        assert_eq!(json["relevance_mode"], "local");
        assert_eq!(json["authorization_scope_enforced"], true);
        assert_eq!(json["omitted_layers"], serde_json::json!(["lint"]));
    }
}

#[tokio::test]
async fn mcp_lint_related_scope_filters_findings_and_mutates_only_project_owned_items() {
    let (state, memory_service, _) = setup_tool_state().await;
    {
        let guard = state.work_service.read().await;
        let work = guard.as_ref().expect("work service");
        work.create_project("alpha", None).await.unwrap();
        work.create_project("beta", None).await.unwrap();
        work.create_task("alpha", "alpha-one", None, Some("ALPHA-1"))
            .await
            .unwrap();
    }

    let mut missing_ids = Vec::new();
    for (scope, title) in [
        (MemoryScope::Global, "Global missing evidence"),
        (MemoryScope::project("alpha"), "Alpha missing evidence"),
        (MemoryScope::project("beta"), "Beta missing evidence"),
        (
            MemoryScope::entity("unowned-entity"),
            "Entity missing evidence",
        ),
    ] {
        let item = MemoryItem::new(
            MemoryKind::Decision,
            title,
            format!("{title} content"),
            scope,
            ClaimOrigin::AgentObserved,
            writer(),
        );
        missing_ids.push((title, item.id));
        memory_service.capture_memory(item).await.unwrap();
    }

    let global_old = MemoryItem::new(
        MemoryKind::Decision,
        "Global superseded item",
        "Globally visible item that related lint must not mutate.",
        MemoryScope::Global,
        ClaimOrigin::AgentObserved,
        writer(),
    )
    .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "lint_tests"));
    let alpha_old = MemoryItem::new(
        MemoryKind::Decision,
        "Alpha superseded item",
        "Alpha item that related lint may safely archive.",
        MemoryScope::project("alpha"),
        ClaimOrigin::AgentObserved,
        writer(),
    )
    .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "lint_tests"));
    let alpha_replacement = MemoryItem::new(
        MemoryKind::Decision,
        "Alpha replacement",
        "Replacement that supersedes both test items.",
        MemoryScope::project("alpha"),
        ClaimOrigin::AgentObserved,
        writer(),
    )
    .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "lint_tests"))
    .with_superseded_item(global_old.id)
    .with_superseded_item(alpha_old.id);
    let global_old_id = global_old.id;
    let alpha_old_id = alpha_old.id;
    memory_service.capture_memory(global_old).await.unwrap();
    memory_service.capture_memory(alpha_old).await.unwrap();
    memory_service
        .capture_memory(alpha_replacement)
        .await
        .unwrap();

    let mut run = lint_request("run");
    run.scope = related_scope("alpha");
    let run = parse_json(&tools::lint_new(&state, run).await.unwrap());
    let findings = run["findings"].as_array().unwrap();
    let finding_ids = findings
        .iter()
        .filter_map(|finding| finding["item_id"].as_str())
        .collect::<Vec<_>>();
    assert!(finding_ids.contains(&missing_ids[0].1.to_string().as_str()));
    assert!(finding_ids.contains(&missing_ids[1].1.to_string().as_str()));
    assert!(!finding_ids.contains(&missing_ids[2].1.to_string().as_str()));
    assert!(!finding_ids.contains(&missing_ids[3].1.to_string().as_str()));
    assert_eq!(run["resolved_project"], "alpha");

    let mut apply = lint_request("apply_safe");
    apply.scope = related_scope("alpha");
    apply.write = Some(true);
    let apply = parse_json(&tools::lint_new(&state, apply).await.unwrap());
    assert_eq!(apply["applied_safe_actions"], 1);
    assert_eq!(
        memory_service
            .get_memory(&alpha_old_id)
            .await
            .unwrap()
            .unwrap()
            .status
            .to_string(),
        "archived"
    );
    assert_eq!(
        memory_service
            .get_memory(&global_old_id)
            .await
            .unwrap()
            .unwrap()
            .status
            .to_string(),
        "active"
    );

    let mut global_apply = lint_request("apply_safe");
    global_apply.project = Some("alpha".to_string());
    global_apply.write = Some(true);
    let global_apply = parse_json(&tools::lint_new(&state, global_apply).await.unwrap());
    assert_eq!(global_apply["applied_safe_actions"], 1);
    assert_eq!(
        memory_service
            .get_memory(&global_old_id)
            .await
            .unwrap()
            .unwrap()
            .status
            .to_string(),
        "archived"
    );

    let mut exact_task = lint_request("run");
    exact_task.scope = related_task_scope("alpha", "ALPHA-1");
    let exact_task = parse_json(&tools::lint_new(&state, exact_task).await.unwrap());
    assert_eq!(exact_task["executed"], false);
    assert_eq!(exact_task["resolved_task"], "alpha-one");
    assert_eq!(exact_task["omitted_layers"], serde_json::json!(["lint"]));

    let mut conflicting_project = lint_request("run");
    conflicting_project.scope = related_scope("alpha");
    conflicting_project.project = Some("beta".to_string());
    let error = tools::lint_new(&state, conflicting_project)
        .await
        .expect_err("conflicting project target should fail");
    assert!(error.contains("does not match resolved authorization project 'alpha'"));

    let vault = tempdir().unwrap();
    let mut vault_lint = lint_request("run");
    vault_lint.scope = related_scope("alpha");
    vault_lint.vault_path = Some(vault.path().display().to_string());
    let error = tools::lint_new(&state, vault_lint)
        .await
        .expect_err("related vault lint should require global authorization");
    assert!(error.contains("vault_path requires scope.relevance_mode=global"));

    let global = parse_json(&tools::lint_new(&state, lint_request("run")).await.unwrap());
    let global_ids = global["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|finding| finding["item_id"].as_str())
        .collect::<Vec<_>>();
    assert!(global_ids.contains(&missing_ids[2].1.to_string().as_str()));
    assert!(global_ids.contains(&missing_ids[3].1.to_string().as_str()));
    assert_eq!(global["relevance_mode"], "global");
}

#[tokio::test]
async fn test_mcp_lint_project_filter_excludes_unrelated_project_memory() {
    let (state, memory_service, _) = setup_tool_state().await;

    let engram = MemoryItem::new(
        MemoryKind::Decision,
        "Engram scoped item",
        "Project-scoped lint should include this missing-evidence item.",
        MemoryScope::project("engram"),
        ClaimOrigin::AgentObserved,
        writer(),
    );
    let engram_id = engram.id;
    memory_service
        .capture_memory(engram)
        .await
        .expect("engram memory should be captured");

    let other = MemoryItem::new(
        MemoryKind::Decision,
        "Other project item",
        "Project-scoped lint should exclude this missing-evidence item.",
        MemoryScope::project("other-project"),
        ClaimOrigin::AgentObserved,
        writer(),
    );
    let other_id = other.id;
    memory_service
        .capture_memory(other)
        .await
        .expect("other memory should be captured");

    let mut request = lint_request("run");
    request.project = Some("engram".to_string());
    let response = tools::lint_new(&state, request)
        .await
        .expect("lint should run");
    let json = parse_json(&response);
    let findings = json["findings"]
        .as_array()
        .expect("findings should be an array");

    assert!(findings
        .iter()
        .any(|finding| finding["item_id"] == engram_id.to_string()));
    assert!(!findings
        .iter()
        .any(|finding| finding["item_id"] == other_id.to_string()));
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
async fn test_mcp_lint_prioritizes_feedback_signal_under_limit() {
    let (state, memory_service, telemetry_repo) = setup_tool_state().await;

    let current_plan = MemoryItem::new(
        MemoryKind::Decision,
        "Current plan after older slice",
        "Old current-plan guidance that feedback says is stale.",
        MemoryScope::project("engram"),
        ClaimOrigin::AgentObserved,
        writer(),
    )
    .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "lint_tests"))
    .with_tag("current-plan");
    let current_plan_id = current_plan.id;
    memory_service
        .capture_memory(current_plan)
        .await
        .expect("current-plan memory should be captured");

    let mut feedback = AgentFeedback::new(Id::new());
    feedback.stale_memory_ids = vec![current_plan_id];
    telemetry_repo
        .save_feedback(&feedback)
        .await
        .expect("feedback should be saved");

    let old = MemoryItem::new(
        MemoryKind::Decision,
        "Superseded older decision",
        "Old content.",
        MemoryScope::project("engram"),
        ClaimOrigin::AgentObserved,
        writer(),
    )
    .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "lint_tests"));
    let replacement = MemoryItem::new(
        MemoryKind::Decision,
        "Replacement decision",
        "New content.",
        MemoryScope::project("engram"),
        ClaimOrigin::AgentObserved,
        writer(),
    )
    .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "lint_tests"))
    .with_superseded_item(old.id);
    memory_service
        .capture_memory(old)
        .await
        .expect("old memory should be captured");
    memory_service
        .capture_memory(replacement)
        .await
        .expect("replacement memory should be captured");

    for index in 0..3 {
        let item = MemoryItem::new(
            MemoryKind::ProjectFact,
            format!("Duplicate entity fact {index}"),
            "Duplicate entity-scoped content.",
            MemoryScope::entity("ide-mcp-eval"),
            ClaimOrigin::AgentObserved,
            writer(),
        )
        .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "lint_tests"));
        memory_service
            .capture_memory(item)
            .await
            .expect("duplicate entity item should be captured");
    }

    let mut request = lint_request("run");
    request.limit = Some(1);
    let response = tools::lint_new(&state, request)
        .await
        .expect("lint should run");
    let json = parse_json(&response);
    let findings = json["findings"]
        .as_array()
        .expect("findings should be an array");

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0]["rule"], "feedback_stale_current_plan");
    assert_eq!(findings[0]["item_id"], current_plan_id.to_string());
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

#[tokio::test]
async fn test_mcp_lint_keeps_stale_migration_authorization_generic() {
    let (state, memory_service, telemetry_repo) = setup_tool_state().await;

    let item = MemoryItem::new(
        MemoryKind::ProjectFact,
        "Approved repo topology migration write applied first batch",
        "Old migration approval record from an earlier scoped repository topology write. \
         It is not current M6 authorization.",
        MemoryScope::project("engram"),
        ClaimOrigin::ToolResult,
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
        .find(|finding| {
            finding["rule"] == "feedback_stale_active_memory"
                && finding["item_id"] == item_id.to_string()
        })
        .expect("stale migration authorization should use generic stale feedback lint");

    assert_eq!(finding["severity"], "info");
    assert_eq!(finding["safe_action"], "none");
    assert!(finding["message"]
        .as_str()
        .expect("finding should include a message")
        .contains("Approved repo topology migration write applied first batch"));
    assert!(!findings.iter().any(|finding| {
        finding["rule"] == "feedback_stale_current_plan"
            && finding["item_id"] == item_id.to_string()
    }));
}
