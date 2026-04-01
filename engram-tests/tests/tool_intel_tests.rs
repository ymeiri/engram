//! Integration tests for Layer 4: Tool Intelligence.
//!
//! Tests tool usage tracking, learning, and recommendations.

use engram_core::entity::EntityType;
use engram_core::id::Id;
use engram_core::tool::ToolOutcome;
use engram_index::{EntityService, ToolIntelService};
use engram_store::{connect_and_init, StoreConfig};

// =============================================================================
// Test Fixtures
// =============================================================================

struct TestServices {
    entity_service: EntityService,
    tool_intel_service: ToolIntelService,
}

async fn setup_services() -> TestServices {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config).await.expect("Failed to connect");

    // ToolIntelService creates its own EntityService internally,
    // but we also need one for creating test fixture tools.
    // Both share the same database connection.
    let entity_service = EntityService::new(db.clone());
    entity_service
        .init()
        .await
        .expect("Failed to init entity service");

    let tool_intel_service = ToolIntelService::new(db);
    tool_intel_service
        .init()
        .await
        .expect("Failed to init tool intel service");

    TestServices {
        entity_service,
        tool_intel_service,
    }
}

/// Helper to create a tool entity.
async fn create_tool(services: &TestServices, name: &str) {
    services
        .entity_service
        .create_entity(name, EntityType::Tool, Some(&format!("{} tool", name)))
        .await
        .expect(&format!("Failed to create tool '{}'", name));
}

// =============================================================================
// Basic Usage Logging Tests
// =============================================================================

#[tokio::test]
async fn test_log_usage() {
    let services = setup_services().await;
    create_tool(&services, "bzl").await;

    let usage = services
        .tool_intel_service
        .log_usage("bzl", "building go service", ToolOutcome::Success, None)
        .await
        .expect("Failed to log usage");

    assert_eq!(usage.context, "building go service");
    assert_eq!(usage.outcome, ToolOutcome::Success);
}

#[tokio::test]
async fn test_log_usage_with_session() {
    let services = setup_services().await;
    create_tool(&services, "cargo").await;

    let session_id = Id::new();

    let usage = services
        .tool_intel_service
        .log_usage(
            "cargo",
            "building rust project",
            ToolOutcome::Success,
            Some(&session_id),
        )
        .await
        .expect("Failed to log usage");

    assert_eq!(usage.session_id, Some(session_id));
}

#[tokio::test]
async fn test_all_outcome_types() {
    let services = setup_services().await;
    create_tool(&services, "test-tool").await;

    let outcomes = [
        ToolOutcome::Success,
        ToolOutcome::Partial,
        ToolOutcome::Failed,
        ToolOutcome::Switched,
    ];

    for outcome in outcomes {
        let usage = services
            .tool_intel_service
            .log_usage("test-tool", "test context", outcome.clone(), None)
            .await
            .expect(&format!("Failed with {:?}", outcome));

        assert_eq!(usage.outcome, outcome);
    }
}

// =============================================================================
// Usage Listing Tests
// =============================================================================

#[tokio::test]
async fn test_list_usages() {
    let services = setup_services().await;
    create_tool(&services, "tool1").await;
    create_tool(&services, "tool2").await;

    services
        .tool_intel_service
        .log_usage("tool1", "ctx1", ToolOutcome::Success, None)
        .await
        .unwrap();
    services
        .tool_intel_service
        .log_usage("tool2", "ctx2", ToolOutcome::Failed, None)
        .await
        .unwrap();
    services
        .tool_intel_service
        .log_usage("tool1", "ctx3", ToolOutcome::Success, None)
        .await
        .unwrap();

    let all = services
        .tool_intel_service
        .list_usages(None, None)
        .await
        .expect("Failed to list");

    assert_eq!(all.len(), 3);
}

#[tokio::test]
async fn test_list_usages_by_outcome() {
    let services = setup_services().await;
    create_tool(&services, "tool").await;

    services
        .tool_intel_service
        .log_usage("tool", "ctx1", ToolOutcome::Success, None)
        .await
        .unwrap();
    services
        .tool_intel_service
        .log_usage("tool", "ctx2", ToolOutcome::Success, None)
        .await
        .unwrap();
    services
        .tool_intel_service
        .log_usage("tool", "ctx3", ToolOutcome::Failed, None)
        .await
        .unwrap();

    let successes = services
        .tool_intel_service
        .list_usages(Some(&ToolOutcome::Success), None)
        .await
        .expect("Failed to list");

    assert_eq!(successes.len(), 2);
    assert!(successes.iter().all(|u| u.outcome == ToolOutcome::Success));

    let failures = services
        .tool_intel_service
        .list_usages(Some(&ToolOutcome::Failed), None)
        .await
        .expect("Failed to list");

    assert_eq!(failures.len(), 1);
}

#[tokio::test]
async fn test_list_usages_with_limit() {
    let services = setup_services().await;

    for i in 0..20 {
        let tool_name = format!("tool-{}", i);
        create_tool(&services, &tool_name).await;
        services
            .tool_intel_service
            .log_usage(&tool_name, "context", ToolOutcome::Success, None)
            .await
            .unwrap();
    }

    let limited = services
        .tool_intel_service
        .list_usages(None, Some(5))
        .await
        .expect("Failed to list");

    assert_eq!(limited.len(), 5);
}

// =============================================================================
// Search Tests
// =============================================================================

#[tokio::test]
async fn test_search_usages() {
    let services = setup_services().await;
    create_tool(&services, "bzl").await;
    create_tool(&services, "cargo").await;

    services
        .tool_intel_service
        .log_usage("bzl", "building go service", ToolOutcome::Success, None)
        .await
        .unwrap();
    services
        .tool_intel_service
        .log_usage("cargo", "building rust project", ToolOutcome::Success, None)
        .await
        .unwrap();
    services
        .tool_intel_service
        .log_usage("bzl", "testing go service", ToolOutcome::Success, None)
        .await
        .unwrap();

    let go_usages = services
        .tool_intel_service
        .search_usages("go", None)
        .await
        .expect("Failed to search");

    assert_eq!(go_usages.len(), 2);
    assert!(go_usages.iter().all(|u| u.context.contains("go")));
}

#[tokio::test]
async fn test_search_usages_with_limit() {
    let services = setup_services().await;
    create_tool(&services, "tool").await;

    for i in 0..20 {
        services
            .tool_intel_service
            .log_usage(
                "tool",
                &format!("context with keyword {}", i),
                ToolOutcome::Success,
                None,
            )
            .await
            .unwrap();
    }

    let limited = services
        .tool_intel_service
        .search_usages("keyword", Some(5))
        .await
        .expect("Failed to search");

    assert_eq!(limited.len(), 5);
}

// =============================================================================
// Tool Statistics Tests
// =============================================================================

#[tokio::test]
async fn test_get_tool_stats() {
    let services = setup_services().await;
    create_tool(&services, "bzl").await;

    // Log usages for bzl
    services
        .tool_intel_service
        .log_usage("bzl", "ctx1", ToolOutcome::Success, None)
        .await
        .unwrap();
    services
        .tool_intel_service
        .log_usage("bzl", "ctx2", ToolOutcome::Success, None)
        .await
        .unwrap();
    services
        .tool_intel_service
        .log_usage("bzl", "ctx3", ToolOutcome::Failed, None)
        .await
        .unwrap();

    let stats = services
        .tool_intel_service
        .get_tool_stats("bzl")
        .await
        .expect("Failed to get stats");

    assert_eq!(stats.total_usages, 3);
    assert_eq!(stats.success_count, 2);
    assert_eq!(stats.failure_count, 1);
    // Success rate should be 2/3 ≈ 0.67
    assert!(stats.success_rate > 0.6 && stats.success_rate < 0.7);
}

#[tokio::test]
async fn test_get_tool_stats_all_success() {
    let services = setup_services().await;
    create_tool(&services, "perfect-tool").await;

    for _ in 0..5 {
        services
            .tool_intel_service
            .log_usage("perfect-tool", "ctx", ToolOutcome::Success, None)
            .await
            .unwrap();
    }

    let stats = services
        .tool_intel_service
        .get_tool_stats("perfect-tool")
        .await
        .expect("Failed to get stats");

    assert_eq!(stats.total_usages, 5);
    assert_eq!(stats.success_count, 5);
    assert_eq!(stats.failure_count, 0);
    assert!((stats.success_rate - 1.0).abs() < 0.001);
}

#[tokio::test]
async fn test_get_tool_stats_all_failures() {
    let services = setup_services().await;
    create_tool(&services, "broken-tool").await;

    for _ in 0..5 {
        services
            .tool_intel_service
            .log_usage("broken-tool", "ctx", ToolOutcome::Failed, None)
            .await
            .unwrap();
    }

    let stats = services
        .tool_intel_service
        .get_tool_stats("broken-tool")
        .await
        .expect("Failed to get stats");

    assert_eq!(stats.total_usages, 5);
    assert_eq!(stats.success_count, 0);
    assert_eq!(stats.failure_count, 5);
    assert!(stats.success_rate < 0.001);
}

#[tokio::test]
async fn test_get_tool_stats_nonexistent() {
    let services = setup_services().await;

    // Requesting stats for a non-existent tool returns NotFound error
    let result = services
        .tool_intel_service
        .get_tool_stats("nonexistent-tool")
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("not found") || err.contains("NotFound"),
        "Expected 'not found' error, got: {}",
        err
    );
}

// =============================================================================
// Recommendations Tests
// =============================================================================

#[tokio::test]
async fn test_recommendations_basic() {
    let services = setup_services().await;
    create_tool(&services, "bzl").await;
    create_tool(&services, "cargo").await;

    // Log successful usages of bzl for go builds
    for _ in 0..5 {
        services
            .tool_intel_service
            .log_usage("bzl", "building go service", ToolOutcome::Success, None)
            .await
            .unwrap();
    }

    // Log some failures for cargo in same context
    for _ in 0..3 {
        services
            .tool_intel_service
            .log_usage("cargo", "building go service", ToolOutcome::Failed, None)
            .await
            .unwrap();
    }

    let recs = services
        .tool_intel_service
        .get_recommendations("building go service")
        .await
        .expect("Failed to get recommendations");

    // bzl should be recommended with high confidence
    assert!(!recs.is_empty());
    // The first recommendation should be bzl with higher confidence
    if !recs.is_empty() {
        assert_eq!(recs[0].tool_name, "bzl");
    }
}

#[tokio::test]
async fn test_recommendations_by_context_similarity() {
    let services = setup_services().await;
    create_tool(&services, "eslint").await;
    create_tool(&services, "prettier").await;

    // Log usages in similar contexts
    services
        .tool_intel_service
        .log_usage(
            "eslint",
            "linting javascript code",
            ToolOutcome::Success,
            None,
        )
        .await
        .unwrap();
    services
        .tool_intel_service
        .log_usage("eslint", "linting js files", ToolOutcome::Success, None)
        .await
        .unwrap();
    services
        .tool_intel_service
        .log_usage(
            "prettier",
            "formatting javascript",
            ToolOutcome::Success,
            None,
        )
        .await
        .unwrap();

    let recs = services
        .tool_intel_service
        .get_recommendations("javascript")
        .await
        .expect("Failed to get recommendations");

    // Should find tools used with javascript
    assert!(!recs.is_empty());
}

#[tokio::test]
async fn test_recommendations_empty_history() {
    let services = setup_services().await;

    let recs = services
        .tool_intel_service
        .get_recommendations("something completely new")
        .await
        .expect("Failed to get recommendations");

    // No history, so no recommendations
    assert!(recs.is_empty());
}

// =============================================================================
// Overall Statistics Tests
// =============================================================================

#[tokio::test]
async fn test_overall_stats() {
    let services = setup_services().await;
    create_tool(&services, "tool1").await;
    create_tool(&services, "tool2").await;

    services
        .tool_intel_service
        .log_usage("tool1", "ctx1", ToolOutcome::Success, None)
        .await
        .unwrap();
    services
        .tool_intel_service
        .log_usage("tool2", "ctx2", ToolOutcome::Failed, None)
        .await
        .unwrap();
    services
        .tool_intel_service
        .log_usage("tool1", "ctx3", ToolOutcome::Success, None)
        .await
        .unwrap();

    let stats = services
        .tool_intel_service
        .stats()
        .await
        .expect("Failed to get stats");

    assert_eq!(stats.usage_count, 3);
}

// =============================================================================
// Edge Cases
// =============================================================================

#[tokio::test]
async fn test_special_characters_in_tool_name() {
    let services = setup_services().await;

    let names = [
        "tool-with-dashes",
        "tool_with_underscores",
        "tool.with.dots",
        "tool/path/style",
        "tool:version:1.0",
    ];

    for name in names {
        create_tool(&services, name).await;

        let usage = services
            .tool_intel_service
            .log_usage(name, "context", ToolOutcome::Success, None)
            .await
            .expect(&format!("Failed with '{}'", name));

        // Verify usage was logged by checking the context
        assert_eq!(usage.context, "context");
    }
}

#[tokio::test]
async fn test_long_context() {
    let services = setup_services().await;
    create_tool(&services, "tool").await;

    let long_context = "A".repeat(10000);

    let usage = services
        .tool_intel_service
        .log_usage("tool", &long_context, ToolOutcome::Success, None)
        .await
        .expect("Failed with long context");

    assert_eq!(usage.context.len(), 10000);
}

#[tokio::test]
async fn test_many_usages_for_tool() {
    let services = setup_services().await;
    create_tool(&services, "popular-tool").await;

    // Log 100 usages
    for i in 0..100 {
        services
            .tool_intel_service
            .log_usage(
                "popular-tool",
                &format!("context {}", i),
                ToolOutcome::Success,
                None,
            )
            .await
            .expect(&format!("Failed at usage {}", i));
    }

    let stats = services
        .tool_intel_service
        .get_tool_stats("popular-tool")
        .await
        .expect("Failed to get stats");

    assert_eq!(stats.total_usages, 100);
}

#[tokio::test]
async fn test_concurrent_tool_usages() {
    let services = setup_services().await;

    // Simulate multiple tools being used in similar contexts
    let tools = ["git", "gh", "hub"];
    let contexts = ["commit changes", "push to remote", "create pr"];

    for tool in tools {
        create_tool(&services, tool).await;
    }

    for tool in tools {
        for context in contexts {
            services
                .tool_intel_service
                .log_usage(tool, context, ToolOutcome::Success, None)
                .await
                .unwrap();
        }
    }

    let stats = services.tool_intel_service.stats().await.expect("Failed");
    assert_eq!(stats.usage_count, 9); // 3 tools × 3 contexts
}

#[tokio::test]
async fn test_partial_outcome_counted_correctly() {
    let services = setup_services().await;
    create_tool(&services, "tool").await;

    services
        .tool_intel_service
        .log_usage("tool", "ctx", ToolOutcome::Success, None)
        .await
        .unwrap();
    services
        .tool_intel_service
        .log_usage("tool", "ctx", ToolOutcome::Partial, None)
        .await
        .unwrap();
    services
        .tool_intel_service
        .log_usage("tool", "ctx", ToolOutcome::Failed, None)
        .await
        .unwrap();
    services
        .tool_intel_service
        .log_usage("tool", "ctx", ToolOutcome::Switched, None)
        .await
        .unwrap();

    let stats = services
        .tool_intel_service
        .get_tool_stats("tool")
        .await
        .expect("Failed");

    assert_eq!(stats.total_usages, 4);
    // Only Success counts as success
    assert_eq!(stats.success_count, 1);
    // Failed counts as failure
    assert_eq!(stats.failure_count, 1);
    // Partial and Switched are neither success nor failure in base stats
}

#[tokio::test]
async fn test_tool_not_registered_error() {
    let services = setup_services().await;
    // Don't create the tool

    let result = services
        .tool_intel_service
        .log_usage("unregistered-tool", "context", ToolOutcome::Success, None)
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("not found"),
        "Expected 'not found' error, got: {}",
        err
    );
}
