//! Integration tests for digest source MCP tooling.

use engram_mcp::tools::{self, DigestRequest, ToolState};
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

fn parse_json(response: &str) -> Value {
    serde_json::from_str(response).expect("response should be valid JSON")
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
