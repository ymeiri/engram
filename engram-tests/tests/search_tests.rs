//! Integration tests for unified search across all layers.
//!
//! Tests the SearchService that searches entities, aliases, observations,
//! session events, documents, and tool usages with a single query.

use engram_core::entity::EntityType;
use engram_core::memory::{
    ClaimOrigin, EvidenceKind, EvidenceRef, Harness, MemoryFreshness, MemoryItem, MemoryKind,
    MemoryReviewState, MemoryScope, MemoryStatus, ModelIdentity, WriterProvenance,
};
use engram_core::search::SearchLayer;
use engram_core::session::EventType;
use engram_core::tool::ToolOutcome;
use engram_index::{
    EntityService, MemoryService, SearchOptions, SearchService, SessionService, ToolIntelService,
};
use engram_store::{connect_and_init, StoreConfig};
use time::OffsetDateTime;

// =============================================================================
// Test Fixtures
// =============================================================================

async fn setup_search_service() -> (
    SearchService,
    EntityService,
    SessionService,
    ToolIntelService,
) {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config).await.expect("Failed to connect");

    // Initialize all required repos through services
    let entity_service = EntityService::new(db.clone());
    entity_service
        .init()
        .await
        .expect("Failed to init entity service");

    let session_service = SessionService::new(db.clone());
    session_service
        .init()
        .await
        .expect("Failed to init session service");

    let tool_intel_service = ToolIntelService::new(db.clone());
    tool_intel_service
        .init()
        .await
        .expect("Failed to init tool intel service");

    // Create search service (without embedder for tests - document search will be skipped)
    let search_service = SearchService::new(db);

    (
        search_service,
        entity_service,
        session_service,
        tool_intel_service,
    )
}

async fn setup_search_and_memory_service() -> (SearchService, MemoryService) {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config).await.expect("Failed to connect");

    let memory_service = MemoryService::new(db.clone());
    memory_service
        .init_schema()
        .await
        .expect("Failed to init memory service");

    (SearchService::new(db), memory_service)
}

fn writer() -> WriterProvenance {
    WriterProvenance::agent(Harness::Codex, ModelIdentity::new("openai", "gpt-5.5"))
        .with_surface("search-test")
}

// =============================================================================
// Entity Search Tests
// =============================================================================

#[tokio::test]
async fn test_search_finds_entity_by_name() {
    let (search_service, entity_service, _, _) = setup_search_service().await;

    // Create an entity
    entity_service
        .create_entity(
            "metrics-integration",
            EntityType::Service,
            Some("Monitors and APM"),
        )
        .await
        .expect("Failed to create entity");

    // Search by name
    let results = search_service
        .search("metrics", 10, None, None)
        .await
        .expect("Failed to search");

    assert!(!results.is_empty(), "Should find entity by name");
    assert!(results.iter().any(|r| r.title.contains("metrics")));
}

#[tokio::test]
async fn test_search_finds_entity_by_description() {
    let (search_service, entity_service, _, _) = setup_search_service().await;

    // Create an entity with description containing the search term
    entity_service
        .create_entity(
            "my-service",
            EntityType::Service,
            Some("Handles service catalog YAML schema"),
        )
        .await
        .expect("Failed to create entity");

    // Search by description content
    let results = search_service
        .search("service catalog YAML", 10, None, None)
        .await
        .expect("Failed to search");

    assert!(!results.is_empty(), "Should find entity by description");
    assert!(results.iter().any(|r| r.content.contains("catalog")));
}

// =============================================================================
// Alias Search Tests
// =============================================================================

#[tokio::test]
async fn test_search_finds_alias() {
    let (search_service, entity_service, _, _) = setup_search_service().await;

    // Create an entity and add an alias
    entity_service
        .create_entity("main-monorepo", EntityType::Service, None)
        .await
        .expect("Failed to create entity");

    entity_service
        .add_alias("main-monorepo", "mono-source")
        .await
        .expect("Failed to add alias");

    // Search by alias
    let results = search_service
        .search("mono-source", 10, None, Some(&[SearchLayer::Alias]))
        .await
        .expect("Failed to search");

    assert!(!results.is_empty(), "Should find by alias");
}

// =============================================================================
// Observation Search Tests
// =============================================================================

#[tokio::test]
async fn test_search_finds_observation_content() {
    let (search_service, entity_service, _, _) = setup_search_service().await;

    // Create an entity with an observation
    entity_service
        .create_entity("main-repo", EntityType::Repo, None)
        .await
        .expect("Failed to create entity");

    entity_service
        .add_observation(
            "main-repo",
            "The service.yaml file defines the service catalog schema",
            Some("config.services"),
            None,
        )
        .await
        .expect("Failed to add observation");

    // Search for observation content (using a partial query that should match)
    let results = search_service
        .search("service catalog schema", 10, None, None)
        .await
        .expect("Failed to search");

    assert!(!results.is_empty(), "Should find observation by content");
    let obs_results: Vec<_> = results
        .iter()
        .filter(|r| r.source.to_string() == "observation")
        .collect();
    assert!(!obs_results.is_empty(), "Should have observation results");
}

#[tokio::test]
async fn test_search_observations_globally() {
    let (search_service, entity_service, _, _) = setup_search_service().await;

    // Create multiple entities with observations
    entity_service
        .create_entity("repo-a", EntityType::Repo, None)
        .await
        .unwrap();
    entity_service
        .create_entity("repo-b", EntityType::Repo, None)
        .await
        .unwrap();

    entity_service
        .add_observation(
            "repo-a",
            "Uses PostgreSQL database",
            Some("dependencies.db"),
            None,
        )
        .await
        .unwrap();

    entity_service
        .add_observation(
            "repo-b",
            "Also uses PostgreSQL for persistence",
            Some("dependencies.db"),
            None,
        )
        .await
        .unwrap();

    // Search should find observations from both entities
    let results = search_service
        .search("PostgreSQL", 10, None, Some(&[SearchLayer::Observation]))
        .await
        .expect("Failed to search");

    assert_eq!(
        results.len(),
        2,
        "Should find observations from both entities"
    );
}

// =============================================================================
// Session Event Search Tests
// =============================================================================

#[tokio::test]
async fn test_search_finds_session_events() {
    let (search_service, _, session_service, _) = setup_search_service().await;

    // Create a session and log an event
    let session = session_service
        .start_session(
            Some("claude-code"),
            Some("test-project"),
            Some("Testing unified search"),
        )
        .await
        .expect("Failed to start session");

    session_service
        .log_event(
            &session.id,
            EventType::Decision,
            "Decided to use PostgreSQL instead of MySQL for better JSON support",
            None,
            None,
        )
        .await
        .expect("Failed to log event");

    // Search for event content
    let results = search_service
        .search(
            "PostgreSQL instead of MySQL",
            10,
            None,
            Some(&[SearchLayer::SessionEvent]),
        )
        .await
        .expect("Failed to search");

    assert!(!results.is_empty(), "Should find session event");
}

// =============================================================================
// Tool Usage Search Tests
// =============================================================================

#[tokio::test]
async fn test_search_finds_tool_usages() {
    let (search_service, entity_service, _, tool_intel_service) = setup_search_service().await;

    // Create a tool entity first
    entity_service
        .create_entity("ripgrep", EntityType::Tool, Some("Fast text search tool"))
        .await
        .expect("Failed to create tool entity");

    // Log a tool usage
    tool_intel_service
        .log_usage(
            "ripgrep",
            "Searching for API endpoint definitions in codebase",
            ToolOutcome::Success,
            None,
        )
        .await
        .expect("Failed to log tool usage");

    // Search for tool usage context
    let results = search_service
        .search(
            "API endpoint definitions",
            10,
            None,
            Some(&[SearchLayer::ToolUsage]),
        )
        .await
        .expect("Failed to search");

    assert!(!results.is_empty(), "Should find tool usage by context");
}

// =============================================================================
// MemoryItem Search Tests
// =============================================================================

#[tokio::test]
async fn test_search_finds_active_memory_items() {
    let (search_service, memory_service) = setup_search_and_memory_service().await;
    let item = MemoryItem::new(
        MemoryKind::Decision,
        "MemoryItem is canonical",
        "Unified search should return MemoryItems as first-class results.",
        MemoryScope::project("engram"),
        ClaimOrigin::UserStated,
        writer(),
    )
    .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "search-test"));
    let item = memory_service.capture_memory(item).await.unwrap();

    let results = search_service
        .search(
            "canonical MemoryItem",
            10,
            Some(0.0),
            Some(&[SearchLayer::Memory]),
        )
        .await
        .expect("Failed to search");

    assert!(results
        .iter()
        .any(|result| result.id == item.id.to_string()));
    assert!(results
        .iter()
        .all(|result| result.source.to_string() == "memory"));
    let result = results
        .iter()
        .find(|result| result.id == item.id.to_string())
        .expect("matching memory result should be present");
    let metadata = result
        .memory_metadata
        .as_ref()
        .expect("memory result should carry trust metadata");
    assert_eq!(metadata.memory_id, item.id);
    assert_eq!(metadata.status, MemoryStatus::Active);
    assert_eq!(metadata.review_state, MemoryReviewState::Reviewed);
    assert_eq!(metadata.freshness, MemoryFreshness::Unscheduled);
    assert_eq!(metadata.claim_origin, ClaimOrigin::UserStated);
    assert_eq!(metadata.evidence_count, 1);
    assert_eq!(metadata.writer.harness, Harness::Codex);
    assert!(result
        .context
        .as_deref()
        .unwrap()
        .contains("review_state: reviewed"));
}

#[tokio::test]
async fn test_memory_search_metadata_reports_review_and_evidence() {
    let (search_service, memory_service) = setup_search_and_memory_service().await;
    let item = MemoryItem::new(
        MemoryKind::Decision,
        "Reviewed memory metadata",
        "Unified search should expose reviewed evidence-backed trust metadata.",
        MemoryScope::project("engram"),
        ClaimOrigin::AgentObserved,
        writer(),
    )
    .with_evidence(EvidenceRef::new(
        EvidenceKind::ManualReview,
        "unit-test-review",
    ));
    let item = memory_service.capture_memory(item).await.unwrap();

    let results = search_service
        .search(
            "reviewed evidence-backed trust metadata",
            10,
            Some(0.0),
            Some(&[SearchLayer::Memory]),
        )
        .await
        .expect("Failed to search");

    let metadata = results
        .iter()
        .find(|result| result.id == item.id.to_string())
        .and_then(|result| result.memory_metadata.as_ref())
        .expect("memory result should carry trust metadata");
    assert_eq!(metadata.review_state, MemoryReviewState::Reviewed);
    assert!(metadata.reviewed);
    assert!(metadata.has_evidence);
    assert_eq!(metadata.evidence_count, 1);
    assert_eq!(metadata.evidence_kinds, vec![EvidenceKind::ManualReview]);
}

#[tokio::test]
async fn test_memory_search_filters_non_active_items() {
    let (search_service, memory_service) = setup_search_and_memory_service().await;
    memory_service
        .capture_memory(
            MemoryItem::new(
                MemoryKind::Preference,
                "Needs review preference",
                "Do not retrieve unreviewed telemetry preference through unified search.",
                MemoryScope::project("engram"),
                ClaimOrigin::AgentInferred,
                writer(),
            )
            .with_status(MemoryStatus::NeedsReview),
        )
        .await
        .unwrap();

    let results = search_service
        .search(
            "unreviewed telemetry preference",
            10,
            Some(0.0),
            Some(&[SearchLayer::Memory]),
        )
        .await
        .expect("Failed to search");

    assert!(
        results.is_empty(),
        "needs_review memory should not be searched"
    );
}

#[tokio::test]
async fn test_memory_search_respects_project_scope_when_provided() {
    let (search_service, memory_service) = setup_search_and_memory_service().await;
    memory_service
        .capture_memory(
            MemoryItem::new(
                MemoryKind::Decision,
                "Engram telemetry policy",
                "Telemetry retrieval policy belongs to Engram.",
                MemoryScope::project("engram"),
                ClaimOrigin::UserStated,
                writer(),
            )
            .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "search-test")),
        )
        .await
        .unwrap();
    memory_service
        .capture_memory(
            MemoryItem::new(
                MemoryKind::Decision,
                "Other telemetry policy",
                "Telemetry retrieval policy belongs to another project.",
                MemoryScope::project("other"),
                ClaimOrigin::UserStated,
                writer(),
            )
            .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "search-test")),
        )
        .await
        .unwrap();

    let results = search_service
        .search_with_options(
            "telemetry retrieval policy",
            10,
            Some(0.0),
            Some(&[SearchLayer::Memory]),
            SearchOptions {
                project: Some("engram".to_string()),
                cwd: None,
            },
        )
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Engram telemetry policy");
}

#[tokio::test]
async fn test_memory_search_prioritizes_current_plan_for_next_step_query() {
    let (search_service, memory_service) = setup_search_and_memory_service().await;
    let now = OffsetDateTime::now_utc();

    let mut older_decision = MemoryItem::new(
        MemoryKind::Decision,
        "Resume continuity probe uses active MemoryItems before ranking changes",
        "For the current Brain Harness resume-continuity issue, the next action is to test \
         active MemoryItem capture before changing ranking.",
        MemoryScope::project("engram"),
        ClaimOrigin::ToolResult,
        writer(),
    )
    .with_confidence(0.99)
    .with_evidence(EvidenceRef::new(EvidenceKind::ToolCall, "older-probe"));
    older_decision.updated_at = now - time::Duration::days(20);
    memory_service.capture_memory(older_decision).await.unwrap();

    let mut current_plan = MemoryItem::new(
        MemoryKind::Decision,
        "Current plan after adapter refresh dry-run",
        "Pending explicit approval, refresh only generated harness adapters. Continue from this \
         current plan; the next step is not migration or hook work.",
        MemoryScope::project("engram"),
        ClaimOrigin::AgentObserved,
        writer(),
    )
    .with_status(MemoryStatus::Active)
    .with_evidence(EvidenceRef::new(
        EvidenceKind::ToolCall,
        "latest-current-plan",
    ))
    .with_tag("current-plan");
    current_plan.updated_at = now;
    let current_plan = memory_service.capture_memory(current_plan).await.unwrap();

    let results = search_service
        .search_with_options(
            "What is the current plan / next step? Continue from where we left off.",
            10,
            Some(0.0),
            Some(&[SearchLayer::Memory]),
            SearchOptions {
                project: Some("engram".to_string()),
                cwd: None,
            },
        )
        .await
        .expect("Failed to search");

    assert_eq!(results[0].id, current_plan.id.to_string());
}

#[tokio::test]
async fn test_memory_search_prefers_project_current_plan_over_repository_plan() {
    let (search_service, memory_service) = setup_search_and_memory_service().await;
    let now = OffsetDateTime::now_utc();

    let mut repository_plan = MemoryItem::new(
        MemoryKind::Decision,
        "Current plan after Codex document lifecycle follow-through",
        "The next product-facing Brain Harness slice completed document lifecycle follow-through. \
         Continue with this current plan.",
        MemoryScope::Repository {
            repository_id: None,
            remote_url: None,
            local_path: Some("/Users/yuval.meiri/projects/engram".to_string()),
        },
        ClaimOrigin::ToolResult,
        writer(),
    )
    .with_status(MemoryStatus::Active)
    .with_evidence(EvidenceRef::new(EvidenceKind::GitCommit, "old-plan"))
    .with_tag("current-plan");
    repository_plan.updated_at = now - time::Duration::days(2);
    memory_service
        .capture_memory(repository_plan)
        .await
        .unwrap();

    let mut project_plan = MemoryItem::new(
        MemoryKind::Decision,
        "Current plan after adapter refresh dry-run",
        "Pending explicit approval, refresh only generated harness adapters. Continue from this \
         current plan.",
        MemoryScope::project("engram"),
        ClaimOrigin::AgentObserved,
        writer(),
    )
    .with_status(MemoryStatus::Active)
    .with_evidence(EvidenceRef::new(
        EvidenceKind::ToolCall,
        "latest-current-plan",
    ))
    .with_tag("current-plan");
    project_plan.updated_at = now;
    let project_plan = memory_service.capture_memory(project_plan).await.unwrap();

    let results = search_service
        .search_with_options(
            "current plan next step continue",
            10,
            Some(0.0),
            Some(&[SearchLayer::Memory]),
            SearchOptions {
                project: Some("engram".to_string()),
                cwd: Some("/Users/yuval.meiri/projects/engram".to_string()),
            },
        )
        .await
        .expect("Failed to search");

    assert_eq!(results[0].id, project_plan.id.to_string());
}

#[tokio::test]
async fn test_memory_search_treats_non_gated_next_slice_as_current_plan() {
    let (search_service, memory_service) = setup_search_and_memory_service().await;
    let now = OffsetDateTime::now_utc();

    let mut noisy_limitation = MemoryItem::new(
        MemoryKind::Limitation,
        "Broad Brain OS next-step search still has non-current-plan top hit",
        "A move forward next non-gated Brain Harness implementation slice query can surface \
         limitation context before current-plan guidance.",
        MemoryScope::project("engram"),
        ClaimOrigin::ToolResult,
        writer(),
    )
    .with_confidence(0.99)
    .with_evidence(EvidenceRef::new(EvidenceKind::ToolCall, "non-gated-gap"));
    noisy_limitation.updated_at = now;
    memory_service
        .capture_memory(noisy_limitation)
        .await
        .unwrap();

    let mut gate = MemoryItem::new(
        MemoryKind::Decision,
        "Migration Must Be Review-Gated",
        "Memory OS migration apply must not proceed without reviewed candidates, a dry-run \
         report, rollback planning, and explicit approval.",
        MemoryScope::project("engram"),
        ClaimOrigin::UserStated,
        writer(),
    )
    .with_confidence(0.95)
    .with_evidence(EvidenceRef::new(
        EvidenceKind::ManualReview,
        "migration-gate",
    ));
    gate.updated_at = now - time::Duration::days(30);
    let gate = memory_service.capture_memory(gate).await.unwrap();

    let mut current_plan = MemoryItem::new(
        MemoryKind::Decision,
        "Current plan after telemetry-backed lint slice",
        "Continue working toward the active thread goal by choosing the next non-gated Brain \
         Harness implementation slice from this current plan.",
        MemoryScope::project("engram"),
        ClaimOrigin::ToolResult,
        writer(),
    )
    .with_status(MemoryStatus::Active)
    .with_evidence(EvidenceRef::new(
        EvidenceKind::ToolCall,
        "latest-current-plan",
    ))
    .with_tag("current-plan");
    current_plan.updated_at = now - time::Duration::minutes(1);
    let current_plan = memory_service.capture_memory(current_plan).await.unwrap();

    let continuation_results = search_service
        .search_with_options(
            "move forward next non-gated Brain Harness implementation slice current plan",
            10,
            Some(0.0),
            Some(&[SearchLayer::Memory]),
            SearchOptions {
                project: Some("engram".to_string()),
                cwd: None,
            },
        )
        .await
        .expect("Failed to search");

    assert_eq!(continuation_results[0].id, current_plan.id.to_string());

    let mixed_gate_results = search_service
        .search_with_options(
            "next non-gated step, should we proceed with migration apply?",
            10,
            Some(0.0),
            Some(&[SearchLayer::Memory]),
            SearchOptions {
                project: Some("engram".to_string()),
                cwd: None,
            },
        )
        .await
        .expect("Failed to search");

    assert_eq!(mixed_gate_results[0].id, gate.id.to_string());
}

#[tokio::test]
async fn test_memory_search_keeps_gate_guidance_above_current_plan() {
    let (search_service, memory_service) = setup_search_and_memory_service().await;
    let now = OffsetDateTime::now_utc();

    let mut gate = MemoryItem::new(
        MemoryKind::Decision,
        "Migration Must Be Review-Gated",
        "Memory OS migration must not proceed without reviewed candidates, a dry-run report, \
         rollback planning, and explicit approval.",
        MemoryScope::project("engram"),
        ClaimOrigin::UserStated,
        writer(),
    )
    .with_confidence(0.95)
    .with_evidence(EvidenceRef::new(
        EvidenceKind::ManualReview,
        "migration-gate",
    ));
    gate.updated_at = now - time::Duration::days(30);
    let gate = memory_service.capture_memory(gate).await.unwrap();

    let mut current_plan = MemoryItem::new(
        MemoryKind::Decision,
        "Current plan after adapter refresh dry-run",
        "Continue by refreshing generated adapters after approval. This current plan does not \
         approve migration.",
        MemoryScope::project("engram"),
        ClaimOrigin::AgentObserved,
        writer(),
    )
    .with_status(MemoryStatus::Active)
    .with_evidence(EvidenceRef::new(
        EvidenceKind::ToolCall,
        "latest-current-plan",
    ))
    .with_tag("current-plan");
    current_plan.updated_at = now;
    memory_service.capture_memory(current_plan).await.unwrap();

    let results = search_service
        .search_with_options(
            "Should we proceed with migration apply?",
            10,
            Some(0.0),
            Some(&[SearchLayer::Memory]),
            SearchOptions {
                project: Some("engram".to_string()),
                cwd: None,
            },
        )
        .await
        .expect("Failed to search");

    assert_eq!(results[0].id, gate.id.to_string());
}

// =============================================================================
// Cross-Layer Search Tests
// =============================================================================

#[tokio::test]
async fn test_search_across_multiple_layers() {
    let (search_service, entity_service, session_service, _) = setup_search_service().await;

    // Create entity
    entity_service
        .create_entity(
            "auth-service",
            EntityType::Service,
            Some("Authentication microservice"),
        )
        .await
        .unwrap();

    // Add observation
    entity_service
        .add_observation(
            "auth-service",
            "Uses JWT tokens for authentication",
            Some("architecture.auth"),
            None,
        )
        .await
        .unwrap();

    // Create session with event
    let session = session_service
        .start_session(Some("test"), None, None)
        .await
        .unwrap();
    session_service
        .log_event(
            &session.id,
            EventType::Decision,
            "Chose JWT over session cookies for authentication",
            None,
            None,
        )
        .await
        .unwrap();

    // Search should find results from multiple layers
    let results = search_service
        .search("authentication", 10, None, None)
        .await
        .expect("Failed to search");

    // Check we have results from multiple sources
    let sources: std::collections::HashSet<_> =
        results.iter().map(|r| r.source.to_string()).collect();

    assert!(
        sources.len() > 1,
        "Should find results from multiple layers"
    );
}

// =============================================================================
// Layer Filtering Tests
// =============================================================================

#[tokio::test]
async fn test_search_with_layer_filter() {
    let (search_service, entity_service, session_service, _) = setup_search_service().await;

    // Create entity and session event with same keyword
    entity_service
        .create_entity(
            "postgres-db",
            EntityType::Service,
            Some("PostgreSQL database"),
        )
        .await
        .unwrap();

    let session = session_service
        .start_session(Some("test"), None, None)
        .await
        .unwrap();
    session_service
        .log_event(
            &session.id,
            EventType::Observation,
            "Connected to PostgreSQL",
            None,
            None,
        )
        .await
        .unwrap();

    // Search only in entities
    let entity_results = search_service
        .search("postgres", 10, None, Some(&[SearchLayer::Entity]))
        .await
        .expect("Failed to search");

    assert!(entity_results
        .iter()
        .all(|r| r.source.to_string() == "entity"));

    // Search only in session events
    let event_results = search_service
        .search("postgres", 10, None, Some(&[SearchLayer::SessionEvent]))
        .await
        .expect("Failed to search");

    assert!(event_results
        .iter()
        .all(|r| r.source.to_string() == "session_event"));
}

// =============================================================================
// Score Tests
// =============================================================================

#[tokio::test]
async fn test_search_results_sorted_by_score() {
    let (search_service, entity_service, _, _) = setup_search_service().await;

    // Create entities with varying match quality
    entity_service
        .create_entity("postgres", EntityType::Service, Some("Database"))
        .await
        .unwrap();

    entity_service
        .create_entity(
            "redis",
            EntityType::Service,
            Some("Uses postgres for caching metadata"),
        )
        .await
        .unwrap();

    let results = search_service
        .search("postgres", 10, None, None)
        .await
        .expect("Failed to search");

    // Results should be sorted by score (descending)
    for i in 1..results.len() {
        assert!(
            results[i - 1].score >= results[i].score,
            "Results should be sorted by score descending"
        );
    }

    // Exact name match should have higher score
    if results.len() >= 2 {
        let exact_match = results.iter().find(|r| r.title == "postgres");
        let partial_match = results.iter().find(|r| r.title != "postgres");
        if let (Some(exact), Some(partial)) = (exact_match, partial_match) {
            assert!(
                exact.score > partial.score,
                "Exact name match should score higher"
            );
        }
    }
}

#[tokio::test]
async fn test_search_min_score_filter() {
    let (search_service, entity_service, _, _) = setup_search_service().await;

    entity_service
        .create_entity("test-service", EntityType::Service, Some("A test service"))
        .await
        .unwrap();

    // Search with high min_score should filter out low-scoring results
    let results = search_service
        .search("test", 10, Some(0.9), None)
        .await
        .expect("Failed to search");

    for result in &results {
        assert!(
            result.score >= 0.9,
            "All results should meet min_score threshold"
        );
    }
}

// =============================================================================
// Empty Results Tests
// =============================================================================

#[tokio::test]
async fn test_search_no_results() {
    let (search_service, _, _, _) = setup_search_service().await;

    let results = search_service
        .search("nonexistent-query-xyz-123", 10, None, None)
        .await
        .expect("Failed to search");

    assert!(
        results.is_empty(),
        "Should return empty results for non-matching query"
    );
}

#[tokio::test]
async fn test_search_empty_database() {
    let (search_service, _, _, _) = setup_search_service().await;

    // Search on empty database
    let results = search_service
        .search("anything", 10, None, None)
        .await
        .expect("Failed to search");

    assert!(
        results.is_empty(),
        "Should handle empty database gracefully"
    );
}
