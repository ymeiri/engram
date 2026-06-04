//! Integration tests for Memory OS MCP tooling.

use engram_index::{EntityService, MemoryService};
use engram_mcp::tools::{
    self, EntityObserveRequestNew, EntityRequestNew, MemoryChangeRequest, MemoryEvidenceRequest,
    MemoryRequestNew, OrientRequest, OrientResponseShape, ToolState, VaultRequest,
};
use engram_store::{connect_and_init, StoreConfig};
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

async fn setup_tool_state() -> ToolState {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config).await.expect("Failed to connect");
    let memory_service = MemoryService::new(db.clone());
    memory_service
        .init_schema()
        .await
        .expect("Failed to initialize memory schema");
    let entity_service = EntityService::new(db);
    entity_service
        .init()
        .await
        .expect("Failed to initialize entity schema");

    let state = ToolState::new();
    state.init_memory(memory_service).await;
    state.init_entity(entity_service).await;
    state
}

fn request(action: &str) -> MemoryRequestNew {
    MemoryRequestNew {
        action: action.to_string(),
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
    }
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
    assert_eq!(json["memory_metadata"][0]["review_state"], "reviewed");
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
        .contains("Trust: status=active, review_state=reviewed"));
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
            prompt: Some("verify current behavior".to_string()),
            project: Some("engram".to_string()),
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
            prompt: Some("verify current behavior".to_string()),
            project: Some("engram".to_string()),
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
async fn test_mcp_orient_puts_follow_user_preference_in_hot_context() {
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
            agent: Some("claude_code".to_string()),
            external_session_id: None,
            intent: Some("follow_user_preference".to_string()),
            scenario_id: Some("claude_rescue_commit_hygiene_001".to_string()),
            arm: Some("test_hot_context".to_string()),
            include_recent_commits: Some(false),
            limit: Some(5),
            response_shape: None,
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
    let hot_index = context_pack
        .find("## Hot Context")
        .expect("hot context should be present");
    let preference_index = context_pack
        .find("Commit every meaningful Engram step")
        .expect("preference should appear in context pack");
    let decisions_index = context_pack
        .find("## Active Decisions")
        .expect("decisions section should be present");

    assert!(hot_index < preference_index);
    assert!(preference_index < decisions_index);
    assert_eq!(
        json["hot_context_ids"][0].as_str(),
        Some(preference_id.as_str())
    );
    assert_eq!(
        json["hot_context_items"][0]["id"].as_str(),
        Some(preference_id.as_str())
    );
    assert!(json["used_memory_candidate_ids"]
        .as_array()
        .unwrap()
        .iter()
        .any(|id| id.as_str() == Some(preference_id.as_str())));
    assert!(context_pack.contains(&format!("Memory {preference_id}:")));
    assert_eq!(
        json["brain_loop"]["top_items"][0]["title"],
        "Commit every meaningful Engram step"
    );
}
