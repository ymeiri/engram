//! Integration tests for digest source MCP tooling.

use engram_index::{SearchService, WorkService};
use engram_mcp::tools::{self, DigestRequest, RetrievalScopeRequest, ToolState};
use engram_store::{connect_and_init, StoreConfig};
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

fn parse_json(response: &str) -> Value {
    serde_json::from_str(response).expect("response should be valid JSON")
}

fn global_scope() -> Option<RetrievalScopeRequest> {
    Some(RetrievalScopeRequest {
        relevance_mode: Some("global".to_string()),
        ..RetrievalScopeRequest::default()
    })
}

fn digest_request(action: &str, scope: Option<RetrievalScopeRequest>) -> DigestRequest {
    DigestRequest {
        action: action.to_string(),
        root_path: None,
        output_path: None,
        review_path: None,
        limit: None,
        include_operational: None,
        max_source_bytes: None,
        max_candidates_per_source: None,
        max_candidate_chars: None,
        write: None,
        scope,
    }
}

async fn related_state() -> ToolState {
    let db = connect_and_init(&StoreConfig::memory())
        .await
        .expect("Failed to connect");
    let work = WorkService::new(db.clone());
    work.init().await.expect("Failed to initialize work schema");
    work.create_project("alpha", None).await.unwrap();
    work.create_task("alpha", "alpha-one", None, Some("ALPHA-1"))
        .await
        .unwrap();
    let state = ToolState::new();
    state.init_search(SearchService::new(db)).await;
    state
}

#[tokio::test]
async fn mcp_digest_actions_abstain_before_filesystem_or_service_access() {
    let source = tempdir().expect("source tempdir should be created");
    let output_parent = tempdir().expect("output tempdir should be created");
    let output = output_parent.path().join("must-not-be-created");
    fs::write(source.path().join("private-digest.md"), "filesystem canary").unwrap();

    for action in [
        "inventory",
        "review_export",
        "review_apply",
        "extraction_plan",
        "source_index",
    ] {
        let mut request = digest_request(action, None);
        request.root_path = Some(source.path().display().to_string());
        request.review_path = Some(source.path().display().to_string());
        request.output_path = Some(output.display().to_string());
        request.write = Some(true);
        let response = tools::digest_new(&ToolState::new(), request)
            .await
            .unwrap_or_else(|error| {
                panic!("{action} should abstain before filesystem access: {error}")
            });
        let json = parse_json(&response);
        assert_eq!(json["executed"], false, "unexpected response for {action}");
        assert_eq!(json["relevance_mode"], "local");
        assert_eq!(json["authorization_scope_enforced"], true);
        assert_eq!(json["omitted_layers"], serde_json::json!(["digest"]));
        assert!(!response.contains("filesystem canary"));
    }
    assert!(!output.exists());

    let error = tools::digest_new(&ToolState::new(), digest_request("unknown", None))
        .await
        .expect_err("unknown actions should be rejected before scope handling");
    assert!(error.contains("Unknown action"));
}

#[tokio::test]
async fn mcp_digest_related_scope_abstains_when_file_ownership_is_unprovable() {
    let state = related_state().await;
    let source = tempdir().expect("source tempdir should be created");
    fs::write(source.path().join("private-digest.md"), "filesystem canary").unwrap();

    for scope in [
        RetrievalScopeRequest {
            relevance_mode: Some("related".to_string()),
            project: Some("alpha".to_string()),
            ..RetrievalScopeRequest::default()
        },
        RetrievalScopeRequest {
            relevance_mode: Some("related".to_string()),
            project: Some("alpha".to_string()),
            task: Some("ALPHA-1".to_string()),
            ..RetrievalScopeRequest::default()
        },
    ] {
        let mut request = digest_request("inventory", Some(scope));
        request.root_path = Some(source.path().display().to_string());
        let response = tools::digest_new(&state, request).await.unwrap();
        let json = parse_json(&response);
        assert_eq!(json["executed"], false);
        assert_eq!(json["relevance_mode"], "related");
        assert_eq!(json["resolved_project"], "alpha");
        assert_eq!(json["omitted_layers"], serde_json::json!(["digest"]));
        assert!(!response.contains("filesystem canary"));
    }
}

#[tokio::test]
async fn test_mcp_digest_inventory_classifies_candidates_and_exclusions() {
    let dir = tempdir().expect("tempdir should be created");
    fs::create_dir_all(dir.path().join("slack-digest/morning")).unwrap();
    fs::create_dir_all(dir.path().join("mail-digest")).unwrap();
    fs::write(
        dir.path().join("slack-digest/morning/2026-04-26.md"),
        "slack",
    )
    .unwrap();
    fs::write(
        dir.path().join("mail-digest/digest-2026-04-26.html"),
        "mail",
    )
    .unwrap();
    fs::write(dir.path().join("mail-digest/_queue.json"), "{}").unwrap();

    let response = tools::digest_new(
        &ToolState::new(),
        DigestRequest {
            action: "inventory".to_string(),
            root_path: Some(dir.path().display().to_string()),
            output_path: None,
            review_path: None,
            limit: None,
            include_operational: None,
            max_source_bytes: None,
            max_candidates_per_source: None,
            max_candidate_chars: None,
            write: None,
            scope: global_scope(),
        },
    )
    .await
    .expect("digest inventory should work");
    let json = parse_json(&response);

    assert_eq!(json["inventory"]["files_scanned"], 3);
    assert_eq!(json["inventory"]["total_candidates"], 2);
    assert_eq!(json["inventory"]["excluded_count"], 1);
    assert_eq!(json["inventory"]["by_source_kind"]["slack"], 1);
    assert_eq!(json["inventory"]["by_source_kind"]["email"], 1);
    assert!(json["inventory"]["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .all(|candidate| candidate["reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reason| reason
                .as_str()
                .unwrap()
                .contains("Inventory does not read file contents"))));
}

#[tokio::test]
async fn test_mcp_digest_inventory_requires_root_path() {
    let err = tools::digest_new(
        &ToolState::new(),
        DigestRequest {
            action: "inventory".to_string(),
            root_path: None,
            output_path: None,
            review_path: None,
            limit: None,
            include_operational: None,
            max_source_bytes: None,
            max_candidates_per_source: None,
            max_candidate_chars: None,
            write: None,
            scope: global_scope(),
        },
    )
    .await
    .unwrap_err();

    assert!(err.contains("root_path required for inventory"));
}

#[tokio::test]
async fn test_mcp_digest_review_export_writes_batch() {
    let dir = tempdir().expect("tempdir should be created");
    let output = tempdir().expect("output tempdir should be created");
    fs::create_dir_all(dir.path().join("slack-digest/morning")).unwrap();
    fs::write(
        dir.path().join("slack-digest/morning/2026-04-26.md"),
        "private slack",
    )
    .unwrap();

    let response = tools::digest_new(
        &ToolState::new(),
        DigestRequest {
            action: "review_export".to_string(),
            root_path: Some(dir.path().display().to_string()),
            output_path: Some(output.path().display().to_string()),
            review_path: None,
            limit: None,
            include_operational: None,
            max_source_bytes: None,
            max_candidates_per_source: None,
            max_candidate_chars: None,
            write: None,
            scope: global_scope(),
        },
    )
    .await
    .expect("digest review export should work");
    let json = parse_json(&response);

    assert_eq!(json["export"]["inventory"]["total_candidates"], 1);
    assert!(json["export"]["files_written"]
        .as_array()
        .unwrap()
        .iter()
        .any(|path| path.as_str() == Some("index.md")));
    assert!(json["export"]["files_written"]
        .as_array()
        .unwrap()
        .iter()
        .any(|path| path
            .as_str()
            .is_some_and(|path| path.starts_with("candidates/"))));
    assert!(output.path().join("index.md").exists());
}

#[tokio::test]
async fn test_mcp_digest_review_apply_parses_reviewed_batch() {
    let dir = tempdir().expect("tempdir should be created");
    let output = tempdir().expect("output tempdir should be created");
    fs::create_dir_all(dir.path().join("slack-digest/morning")).unwrap();
    fs::write(
        dir.path().join("slack-digest/morning/2026-04-26.md"),
        "private slack",
    )
    .unwrap();

    let export_response = tools::digest_new(
        &ToolState::new(),
        DigestRequest {
            action: "review_export".to_string(),
            root_path: Some(dir.path().display().to_string()),
            output_path: Some(output.path().display().to_string()),
            review_path: None,
            limit: None,
            include_operational: None,
            max_source_bytes: None,
            max_candidates_per_source: None,
            max_candidate_chars: None,
            write: None,
            scope: global_scope(),
        },
    )
    .await
    .expect("digest review export should work");
    let export_json = parse_json(&export_response);
    let candidate_path = export_json["export"]["files_written"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|path| path.as_str())
        .find(|path| path.starts_with("candidates/"))
        .expect("candidate file should be written");
    let path = output.path().join(candidate_path);
    let contents = fs::read_to_string(&path).unwrap();
    fs::write(
        &path,
        contents.replace(
            "decision: pending # accept | reject | quarantine | source_only",
            "decision: accept # accept | reject | quarantine | source_only",
        ),
    )
    .unwrap();

    let apply_response = tools::digest_new(
        &ToolState::new(),
        DigestRequest {
            action: "review_apply".to_string(),
            root_path: None,
            output_path: None,
            review_path: Some(output.path().display().to_string()),
            limit: None,
            include_operational: None,
            max_source_bytes: None,
            max_candidates_per_source: None,
            max_candidate_chars: None,
            write: None,
            scope: global_scope(),
        },
    )
    .await
    .expect("digest review apply should work");
    let json = parse_json(&apply_response);

    assert_eq!(json["apply"]["accepted_count"], 1);
    assert_eq!(
        json["apply"]["planned_sources"].as_array().unwrap().len(),
        1
    );
    let serialized = serde_json::to_string(&json).unwrap();
    assert!(!serialized.contains("private slack"));
}

#[tokio::test]
async fn test_mcp_digest_extraction_plan_reads_only_accepted_sources() {
    let dir = tempdir().expect("tempdir should be created");
    let review = tempdir().expect("review tempdir should be created");
    let output = tempdir().expect("output tempdir should be created");
    fs::create_dir_all(dir.path().join("slack-digest/morning")).unwrap();
    fs::create_dir_all(dir.path().join("notes-digest")).unwrap();
    fs::write(
        dir.path().join("slack-digest/morning/2026-04-26.md"),
        "accepted source body with enough detail for candidate memory extraction",
    )
    .unwrap();
    fs::write(
        dir.path().join("notes-digest/digest-2026-04-26.md"),
        "source only body should not be copied into extraction output",
    )
    .unwrap();

    let export_response = tools::digest_new(
        &ToolState::new(),
        DigestRequest {
            action: "review_export".to_string(),
            root_path: Some(dir.path().display().to_string()),
            output_path: Some(review.path().display().to_string()),
            review_path: None,
            limit: None,
            include_operational: None,
            max_source_bytes: None,
            max_candidates_per_source: None,
            max_candidate_chars: None,
            write: None,
            scope: global_scope(),
        },
    )
    .await
    .expect("digest review export should work");
    let export_json = parse_json(&export_response);
    let candidate_paths = export_json["export"]["files_written"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|path| path.as_str())
        .filter(|path| path.starts_with("candidates/"))
        .collect::<Vec<_>>();
    for candidate_path in candidate_paths {
        if candidate_path.contains("slack") {
            set_review_decision(review.path(), candidate_path, "accept");
        } else {
            set_review_decision(review.path(), candidate_path, "source_only");
        }
    }

    let plan_response = tools::digest_new(
        &ToolState::new(),
        DigestRequest {
            action: "extraction_plan".to_string(),
            root_path: None,
            output_path: Some(output.path().display().to_string()),
            review_path: Some(review.path().display().to_string()),
            limit: None,
            include_operational: None,
            max_source_bytes: None,
            max_candidates_per_source: Some(2),
            max_candidate_chars: Some(500),
            write: None,
            scope: global_scope(),
        },
    )
    .await
    .expect("digest extraction plan should work");
    let json = parse_json(&plan_response);

    assert_eq!(json["plan"]["accepted_sources"], 1);
    assert_eq!(json["plan"]["source_only_sources"], 1);
    assert_eq!(json["plan"]["sources_read"], 1);
    assert_eq!(json["plan"]["candidates"].as_array().unwrap().len(), 1);
    let output_text = fs::read_to_string(
        output.path().join(
            json["plan"]["candidates"][0]["review_path"]
                .as_str()
                .unwrap(),
        ),
    )
    .unwrap();
    assert!(output_text.contains("accepted source body"));
    assert!(!output_text.contains("source only body"));
}

#[tokio::test]
async fn test_mcp_digest_source_index_reads_only_source_only_sources_dry_run() {
    let dir = tempdir().expect("tempdir should be created");
    let review = tempdir().expect("review tempdir should be created");
    fs::create_dir_all(dir.path().join("slack-digest/morning")).unwrap();
    fs::create_dir_all(dir.path().join("notes-digest")).unwrap();
    fs::write(
        dir.path().join("slack-digest/morning/2026-04-26.md"),
        "accepted source body should not be indexed by source_only indexing",
    )
    .unwrap();
    fs::write(
        dir.path().join("notes-digest/digest-2026-04-26.md"),
        "source only body should become document evidence after review",
    )
    .unwrap();

    let export_response = tools::digest_new(
        &ToolState::new(),
        DigestRequest {
            action: "review_export".to_string(),
            root_path: Some(dir.path().display().to_string()),
            output_path: Some(review.path().display().to_string()),
            review_path: None,
            limit: None,
            include_operational: None,
            max_source_bytes: None,
            max_candidates_per_source: None,
            max_candidate_chars: None,
            write: None,
            scope: global_scope(),
        },
    )
    .await
    .expect("digest review export should work");
    let export_json = parse_json(&export_response);
    let candidate_paths = export_json["export"]["files_written"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|path| path.as_str())
        .filter(|path| path.starts_with("candidates/"))
        .collect::<Vec<_>>();
    for candidate_path in candidate_paths {
        if candidate_path.contains("slack") {
            set_review_decision(review.path(), candidate_path, "accept");
        } else {
            set_review_decision(review.path(), candidate_path, "source_only");
        }
    }

    let source_index_response = tools::digest_new(
        &ToolState::new(),
        DigestRequest {
            action: "source_index".to_string(),
            root_path: None,
            output_path: None,
            review_path: Some(review.path().display().to_string()),
            limit: None,
            include_operational: None,
            max_source_bytes: None,
            max_candidates_per_source: None,
            max_candidate_chars: None,
            write: Some(false),
            scope: global_scope(),
        },
    )
    .await
    .expect("digest source index should work");
    let json = parse_json(&source_index_response);

    assert_eq!(json["dry_run"], true);
    assert_eq!(json["indexed_documents"], 0);
    assert_eq!(json["plan"]["accepted_sources"], 1);
    assert_eq!(json["plan"]["source_only_sources"], 1);
    assert_eq!(json["plan"]["sources_read"], 1);
    assert_eq!(json["plan"]["documents"].as_array().unwrap().len(), 1);
    let serialized = serde_json::to_string(&json).unwrap();
    assert!(!serialized.contains("source only body should become"));
    assert!(!serialized.contains("accepted source body"));
}

fn set_review_decision(root: &std::path::Path, candidate_path: &str, decision: &str) {
    let path = root.join(candidate_path);
    let contents = fs::read_to_string(&path).unwrap();
    fs::write(
        path,
        contents.replace(
            "decision: pending # accept | reject | quarantine | source_only",
            &format!("decision: {decision} # accept | reject | quarantine | source_only"),
        ),
    )
    .unwrap();
}
