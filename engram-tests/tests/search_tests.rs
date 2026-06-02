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
async fn test_memory_search_surfaces_active_design_philosophy_preference() {
    let (search_service, memory_service) = setup_search_and_memory_service().await;

    let preference = memory_service
        .capture_memory(
            MemoryItem::new(
                MemoryKind::Preference,
                "Software design philosophy preference",
                "User prefers software design in the spirit of John Ousterhout's A Philosophy \
                 of Software Design: deep modules with simple interfaces, low cognitive load, \
                 no unrequested features, small end-to-end slices, and evidence over confidence \
                 when making design claims.",
                MemoryScope::User,
                ClaimOrigin::UserStated,
                writer(),
            )
            .with_evidence(EvidenceRef::new(
                EvidenceKind::ManualReview,
                "user-stated-design-philosophy",
            ))
            .with_tag("preference")
            .with_tag("software-design")
            .with_tag("ousterhout"),
        )
        .await
        .unwrap();

    let generic_design_note = memory_service
        .capture_memory(
            MemoryItem::new(
                MemoryKind::ProjectFact,
                "Generic software design note",
                "Architecture notes can discuss modules, interfaces, and implementation slices \
                 without encoding the user's durable design preference.",
                MemoryScope::project("engram"),
                ClaimOrigin::AgentObserved,
                writer(),
            )
            .with_confidence(0.99)
            .with_evidence(EvidenceRef::new(
                EvidenceKind::ManualReview,
                "generic-design-note",
            )),
        )
        .await
        .unwrap();

    let results = search_service
        .search_with_options(
            "Ousterhout deep modules no unrequested features small end-to-end slices evidence over confidence",
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

    let preference_index = results
        .iter()
        .position(|result| result.id == preference.id.to_string())
        .expect("active design philosophy preference should be returned");
    let generic_index = results
        .iter()
        .position(|result| result.id == generic_design_note.id.to_string())
        .expect("generic design control should be returned");

    assert!(
        preference_index < generic_index,
        "specific user preference should rank ahead of generic design context"
    );
    assert_eq!(
        results[preference_index]
            .memory_metadata
            .as_ref()
            .map(|metadata| metadata.review_state),
        Some(MemoryReviewState::Reviewed)
    );
}

#[tokio::test]
async fn test_memory_search_surfaces_active_telemetry_feedback_rule() {
    let (search_service, memory_service) = setup_search_and_memory_service().await;

    let feedback_rule = memory_service
        .capture_memory(
            MemoryItem::new(
                MemoryKind::Rule,
                "Telemetry feedback expectations",
                "Agent telemetry feedback expectations: include used_memory_ids for returned \
                 memory that shaped an answer, rejected_memory_ids for memory rejected as stale, \
                 noisy, wrong_scope, or irrelevant, structured missing_context when expected \
                 context is absent or buried, and bad_memory_used when memory caused harmful \
                 behavior. Treat agent feedback as a weak signal until it is correlated with \
                 transcript, tests, user review, or later memory edits.",
                MemoryScope::project("engram"),
                ClaimOrigin::UserStated,
                writer(),
            )
            .with_evidence(EvidenceRef::new(
                EvidenceKind::ManualReview,
                "orient-contract-feedback-expectations",
            ))
            .with_tag("telemetry")
            .with_tag("feedback")
            .with_tag("weak-signal"),
        )
        .await
        .unwrap();

    let generic_telemetry_note = memory_service
        .capture_memory(
            MemoryItem::new(
                MemoryKind::ProjectFact,
                "Telemetry implementation note",
                "Telemetry records traces, feedback rows, and aggregate coverage metrics for \
                 Brain Harness reports.",
                MemoryScope::project("engram"),
                ClaimOrigin::AgentObserved,
                writer(),
            )
            .with_confidence(0.99)
            .with_evidence(EvidenceRef::new(
                EvidenceKind::ManualReview,
                "generic-telemetry-note",
            )),
        )
        .await
        .unwrap();

    let results = search_service
        .search_with_options(
            "telemetry feedback expectations used_memory_ids rejected stale wrong_scope missing_context weak signal",
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

    let rule_index = results
        .iter()
        .position(|result| result.id == feedback_rule.id.to_string())
        .expect("active telemetry feedback rule should be returned");
    let generic_index = results
        .iter()
        .position(|result| result.id == generic_telemetry_note.id.to_string())
        .expect("generic telemetry control should be returned");

    assert!(
        rule_index < generic_index,
        "specific feedback rule should rank ahead of generic telemetry context"
    );
    assert_eq!(
        results[rule_index]
            .memory_metadata
            .as_ref()
            .map(|metadata| metadata.review_state),
        Some(MemoryReviewState::Reviewed)
    );
}

#[tokio::test]
async fn test_memory_search_surfaces_active_orient_contract_rule() {
    let (search_service, memory_service) = setup_search_and_memory_service().await;

    let orient_contract_rule = memory_service
        .capture_memory(
            MemoryItem::new(
                MemoryKind::Rule,
                "Lean orient contract",
                "`orient` lean response shape trace_id memory_cursor candidate ids obligation \
                 summary contract: lean `orient` preserves trace_id, memory_cursor, candidate \
                 IDs, Brain Loop guidance, recommended actions, ambiguities, obligation_summary, \
                 and open_obligations while omitting context_pack, raw memory buckets, \
                 memory_metadata, recent_knowledge_commits, and repeated trust payloads. Lean \
                 shape is a presentation option only and must not change retrieval, ranking, \
                 trace creation, candidate IDs, or obligation surfacing.",
                MemoryScope::project("engram"),
                ClaimOrigin::UserStated,
                writer(),
            )
            .with_confidence(0.96)
            .with_evidence(EvidenceRef::new(
                EvidenceKind::ManualReview,
                "orient-contract-lean-shape",
            ))
            .with_tag("orient")
            .with_tag("orient-contract")
            .with_tag("lean")
            .with_tag("hot-path"),
        )
        .await
        .unwrap();

    let generic_orient_note = memory_service
        .capture_memory(
            MemoryItem::new(
                MemoryKind::ProjectFact,
                "Orient implementation note",
                "Orient implementation details can mention traces, cursors, candidate lists, \
                 and obligation information without encoding the reviewed lean hot-path \
                 contract.",
                MemoryScope::project("engram"),
                ClaimOrigin::AgentObserved,
                writer(),
            )
            .with_confidence(0.99)
            .with_evidence(EvidenceRef::new(
                EvidenceKind::ManualReview,
                "generic-orient-note",
            )),
        )
        .await
        .unwrap();

    let results = search_service
        .search_with_options(
            "orient lean response shape trace_id memory_cursor candidate ids obligation summary",
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

    let rule_index = results
        .iter()
        .position(|result| result.id == orient_contract_rule.id.to_string())
        .expect("active lean orient contract rule should be returned");
    let generic_index = results
        .iter()
        .position(|result| result.id == generic_orient_note.id.to_string())
        .expect("generic orient control should be returned");

    assert!(
        rule_index < generic_index,
        "specific lean orient contract should rank ahead of generic orient context"
    );
    assert_eq!(
        results[rule_index]
            .memory_metadata
            .as_ref()
            .map(|metadata| metadata.review_state),
        Some(MemoryReviewState::Reviewed)
    );
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
async fn test_memory_search_t60_what_should_happen_next_promotes_current_plan() {
    let (search_service, memory_service) = setup_search_and_memory_service().await;
    let now = OffsetDateTime::now_utc();

    let mut research_method = MemoryItem::new(
        MemoryKind::Rule,
        "Brain Harness work follows research method",
        "Continue the Engram Brain Harness work by stating the research question, \
         hypotheses, measurement, and what should happen next before implementation.",
        MemoryScope::project("engram"),
        ClaimOrigin::UserStated,
        writer(),
    )
    .with_confidence(0.99)
    .with_evidence(EvidenceRef::new(
        EvidenceKind::ManualReview,
        "research-method-rule",
    ));
    research_method.updated_at = now;
    memory_service
        .capture_memory(research_method)
        .await
        .unwrap();

    let mut historical_calibration = MemoryItem::new(
        MemoryKind::ProjectFact,
        "Non-gated continuation search calibration landed",
        "Historical calibration notes mention continuing the Brain Harness work and choosing \
         what should happen next, but they are not the active current plan.",
        MemoryScope::project("engram"),
        ClaimOrigin::ToolResult,
        writer(),
    )
    .with_confidence(0.99)
    .with_evidence(EvidenceRef::new(
        EvidenceKind::ToolCall,
        "historical-calibration",
    ));
    historical_calibration.updated_at = now;
    memory_service
        .capture_memory(historical_calibration)
        .await
        .unwrap();

    let mut m6_gate = MemoryItem::new(
        MemoryKind::Limitation,
        "M6 migration approval gate remains explicit",
        "Brain Harness work must not run M6 migration read-only inventory or review export \
         without explicit user-approved scope. M6 write apply requires reviewed candidates, \
         dry-run evidence, rollback planning, and explicit approval.",
        MemoryScope::project("engram"),
        ClaimOrigin::UserStated,
        writer(),
    )
    .with_confidence(0.98)
    .with_evidence(EvidenceRef::new(
        EvidenceKind::ManualReview,
        "m6-approval-gate",
    ));
    m6_gate.updated_at = now - time::Duration::hours(2);
    let m6_gate = memory_service.capture_memory(m6_gate).await.unwrap();

    let mut current_plan = MemoryItem::new(
        MemoryKind::Decision,
        "T59 M6 review export approval packet prepared",
        "The current plan is to keep M6 review export blocked until the user explicitly \
         approves the T59 scope. Continue the Brain Harness work with non-gated validation.",
        MemoryScope::project("engram"),
        ClaimOrigin::AgentObserved,
        writer(),
    )
    .with_status(MemoryStatus::Active)
    .with_confidence(0.8)
    .with_evidence(EvidenceRef::new(EvidenceKind::GitCommit, "t59-packet"))
    .with_tag("current-plan");
    current_plan.updated_at = now - time::Duration::minutes(1);
    let current_plan = memory_service.capture_memory(current_plan).await.unwrap();

    let results = search_service
        .search_with_options(
            "Continue the Engram Brain Harness work. What is the current plan and what \
             should happen next?",
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

    assert_eq!(results[0].id, current_plan.id.to_string());
    assert!(
        results
            .iter()
            .any(|result| result.id == m6_gate.id.to_string()),
        "continuation query should still keep M6 gate evidence retrievable"
    );

    let explicit_gate_results = search_service
        .search_with_options(
            "What is the current plan, and should we run migration_review_export?",
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

    assert_ne!(explicit_gate_results[0].id, current_plan.id.to_string());
    assert!(
        explicit_gate_results
            .iter()
            .take(2)
            .any(|result| result.id == m6_gate.id.to_string()),
        "explicit review-export prompt should keep active M6 gate in top gate context"
    );
}

#[tokio::test]
async fn test_memory_search_t107_broad_next_step_promotes_current_plan() {
    let (search_service, memory_service) = setup_search_and_memory_service().await;
    let now = OffsetDateTime::now_utc();

    for (index, (kind, title, content)) in [
        (
            MemoryKind::ProjectFact,
            "AI Council and Claude next-step synthesis after orient contract",
            "On 2026-05-06, Codex consulted Claude Bridge and AI Council on the next \
             Brain Harness step. The old synthesis says what should happen next only for \
             that historical checkpoint.",
        ),
        (
            MemoryKind::Rule,
            "Brain Harness work follows research method",
            "Engram Brain Harness work should state research questions, hypotheses, \
             measurement, and what should happen next before implementation.",
        ),
        (
            MemoryKind::Handoff,
            "Rolling handoff",
            "The rolling handoff summarizes current state and next actions, but it is \
             continuity context rather than the active current-plan guidance item.",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let mut item = MemoryItem::new(
            kind,
            title,
            content,
            MemoryScope::project("engram"),
            ClaimOrigin::ToolResult,
            writer(),
        )
        .with_confidence(0.99)
        .with_evidence(EvidenceRef::new(
            EvidenceKind::ToolCall,
            format!("t107-distractor-{index}"),
        ));
        item.updated_at = now - time::Duration::minutes(index as i64);
        memory_service.capture_memory(item).await.unwrap();
    }

    let mut m6_gate = MemoryItem::new(
        MemoryKind::Limitation,
        "M6 migration approval gate remains explicit",
        "Brain Harness work must not run M6 migration read-only inventory or review export \
         without explicit user-approved scope. M6 write apply, deletion, cleanup, or legacy \
         simplification additionally require reviewed candidates, dry-run evidence, rollback \
         planning, and explicit approval.",
        MemoryScope::project("engram"),
        ClaimOrigin::UserStated,
        writer(),
    )
    .with_confidence(0.98)
    .with_evidence(EvidenceRef::new(
        EvidenceKind::ManualReview,
        "m6-approval-gate",
    ));
    m6_gate.updated_at = now - time::Duration::hours(2);
    let m6_gate = memory_service.capture_memory(m6_gate).await.unwrap();

    let mut current_plan = MemoryItem::new(
        MemoryKind::Decision,
        "Current plan after T106 harness readiness drift recheck",
        "T106 recorded a docs-only read-only harness readiness drift recheck. The next \
         product-moving gate remains exact T69, but without that approval the next work \
         must stay non-gated and evidence-focused.",
        MemoryScope::project("engram"),
        ClaimOrigin::AgentObserved,
        writer(),
    )
    .with_status(MemoryStatus::Active)
    .with_confidence(0.93)
    .with_evidence(EvidenceRef::new(EvidenceKind::GitCommit, "t106-report"))
    .with_tag("current-plan");
    current_plan.updated_at = now - time::Duration::minutes(1);
    let current_plan = memory_service.capture_memory(current_plan).await.unwrap();

    for query in [
        "what should happen next Engram Brain Harness",
        "what should we do next for Engram?",
    ] {
        let results = search_service
            .search_with_options(
                query,
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

        assert_eq!(results[0].id, current_plan.id.to_string(), "{query}");
    }

    let explicit_gate_results = search_service
        .search_with_options(
            "should we proceed with M6 migration apply?",
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

    assert_eq!(explicit_gate_results[0].id, m6_gate.id.to_string());
}

#[tokio::test]
async fn test_memory_search_t118_exact_approval_command_promotes_matching_current_plan() {
    let (search_service, memory_service) = setup_search_and_memory_service().await;
    let now = OffsetDateTime::now_utc();
    let approval_command = "Approve T70: index exact files T59, T68, and T69.";

    for (index, title) in [
        approval_command,
        "T109 handoff repeats Approve T70: index exact files T59, T68, and T69.",
    ]
    .into_iter()
    .enumerate()
    {
        let mut handoff = MemoryItem::new(
            MemoryKind::Handoff,
            title,
            "Historical handoff text repeats the T70 approval command and adjacent T59/T68/T69 \
             tokens, but it is continuity context rather than active current-plan guidance.",
            MemoryScope::project("engram"),
            ClaimOrigin::ToolResult,
            writer(),
        )
        .with_status(MemoryStatus::Active)
        .with_confidence(0.99)
        .with_evidence(EvidenceRef::new(
            EvidenceKind::ToolCall,
            format!("t118-old-handoff-{index}"),
        ));
        handoff.updated_at = now - time::Duration::minutes(index as i64);
        memory_service.capture_memory(handoff).await.unwrap();
    }

    let mut current_plan = MemoryItem::new(
        MemoryKind::Decision,
        "T117 current-plan keeps exact T70 gate context authoritative",
        "The active current plan says exact approval commands should recover this plan before \
         old handoffs. The command is `Approve T70: index exact files T59, T68, and T69.` \
         T69 inspection and M6 write apply remain separately gated.",
        MemoryScope::project("engram"),
        ClaimOrigin::AgentObserved,
        writer(),
    )
    .with_status(MemoryStatus::Active)
    .with_confidence(0.9)
    .with_evidence(EvidenceRef::new(
        EvidenceKind::GitCommit,
        "t117-parity-audit",
    ))
    .with_tag("current-plan");
    current_plan.updated_at = now - time::Duration::minutes(3);
    let current_plan = memory_service.capture_memory(current_plan).await.unwrap();

    let results = search_service
        .search_with_options(
            approval_command,
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

    assert_eq!(results[0].id, current_plan.id.to_string());

    let non_command_results = search_service
        .search_with_options(
            "Approve T70 without colon",
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

    assert_ne!(non_command_results[0].id, current_plan.id.to_string());
}

#[tokio::test]
async fn test_memory_search_t140_continuation_with_approval_gate_context_promotes_current_plan() {
    let (search_service, memory_service) = setup_search_and_memory_service().await;
    let now = OffsetDateTime::now_utc();

    for (index, content) in [
        "T138 is complete. Continue the Engram Brain Harness after T139 and T135; the handoff \
         repeats approval gate context but is not the active current-plan guidance.",
        "T133A and T135 harness repair approval gate details are preserved here as historical \
         rolling handoff continuity context.",
        "T139 and T135 approval gate notes mention current plan, next step, continue, move \
         forward, and Brain Harness, but this older handoff should not lead the search result.",
    ]
    .into_iter()
    .enumerate()
    {
        let mut handoff = MemoryItem::new(
            MemoryKind::Handoff,
            "Rolling handoff",
            content,
            MemoryScope::project("engram"),
            ClaimOrigin::ToolResult,
            writer(),
        )
        .with_status(MemoryStatus::Active)
        .with_confidence(0.99)
        .with_evidence(EvidenceRef::new(
            EvidenceKind::ToolCall,
            format!("t140-handoff-distractor-{index}"),
        ));
        handoff.updated_at = now - time::Duration::minutes(index as i64);
        memory_service.capture_memory(handoff).await.unwrap();
    }

    let mut m6_gate = MemoryItem::new(
        MemoryKind::Limitation,
        "M6 migration approval gate remains explicit",
        "M6 migration and quarantine inspection remain gated. This approval gate context should \
         stay retrievable, but it is not itself the current plan.",
        MemoryScope::project("engram"),
        ClaimOrigin::UserStated,
        writer(),
    )
    .with_confidence(0.98)
    .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "t140-m6-gate"));
    m6_gate.updated_at = now - time::Duration::hours(2);
    let m6_gate = memory_service.capture_memory(m6_gate).await.unwrap();

    let mut current_plan = MemoryItem::new(
        MemoryKind::Decision,
        "T139 stale current-plan approval packet committed; archive remains gated",
        "T139 is complete. Continue toward the Engram Brain Harness goal from this current plan. \
         The next step must respect T135 and T139 approval gates; without exact approval, work \
         stays non-gated and evidence-focused.",
        MemoryScope::project("engram"),
        ClaimOrigin::AgentObserved,
        writer(),
    )
    .with_status(MemoryStatus::Active)
    .with_confidence(0.8)
    .with_evidence(EvidenceRef::new(EvidenceKind::GitCommit, "t139-packet"))
    .with_tag("current-plan");
    current_plan.updated_at = now - time::Duration::minutes(1);
    let current_plan = memory_service.capture_memory(current_plan).await.unwrap();

    let results = search_service
        .search_with_options(
            "current plan next step continue move forward Engram Brain Harness after T139 T135 \
             T139 approval gate",
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

    assert_eq!(results[0].id, current_plan.id.to_string());
    assert!(
        results
            .iter()
            .any(|result| result.id == m6_gate.id.to_string()),
        "approval gate context should remain retrievable"
    );
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
async fn test_memory_search_t114_current_plan_outranks_stale_and_wrong_scope_noise() {
    let (search_service, memory_service) = setup_search_and_memory_service().await;
    let now = OffsetDateTime::now_utc();

    let mut stale_repository_plan = MemoryItem::new(
        MemoryKind::Decision,
        "Current plan after Codex document lifecycle follow-through",
        "The next product-facing Brain Harness slice is complete for Codex adapter guidance. \
         It mentions recent failures, open risks, stale current-plan feedback, and safe_action \
         none, but it is older repository-scoped review noise.",
        MemoryScope::Repository {
            repository_id: None,
            remote_url: None,
            local_path: Some("/Users/yuval.meiri/projects/engram".to_string()),
        },
        ClaimOrigin::ToolResult,
        writer(),
    )
    .with_status(MemoryStatus::Active)
    .with_confidence(0.99)
    .with_evidence(EvidenceRef::new(EvidenceKind::GitCommit, "old-plan"))
    .with_tag("current-plan");
    stale_repository_plan.updated_at = now - time::Duration::days(7);
    let stale_repository_plan = memory_service
        .capture_memory(stale_repository_plan)
        .await
        .unwrap();

    let claude_writer = WriterProvenance::agent(
        Harness::ClaudeCode,
        ModelIdentity::new("anthropic", "claude-code"),
    )
    .with_surface("claude-code");
    let mut wrong_scope_rule = MemoryItem::new(
        MemoryKind::Rule,
        "Claude Code user-stated instruction",
        "Read-only critique request for Engram. The text mentions recent failures, caveats, \
         open risks, wrong-scope feedback, stale current-plan guidance, safe_action none, and \
         T113, but it is not the active project plan.",
        MemoryScope::project("engram"),
        ClaimOrigin::UserStated,
        claude_writer,
    )
    .with_status(MemoryStatus::Active)
    .with_confidence(0.99)
    .with_evidence(EvidenceRef::new(
        EvidenceKind::ManualReview,
        "claude-rule-noise",
    ));
    wrong_scope_rule.updated_at = now - time::Duration::hours(4);
    let wrong_scope_rule = memory_service
        .capture_memory(wrong_scope_rule)
        .await
        .unwrap();

    let mut current_plan = MemoryItem::new(
        MemoryKind::Decision,
        "Current plan after T113 startup retrieval validation",
        "T113 validated startup retrieval after T112. Continue only non-gated validation work; \
         recent failures and open risks are stale repository current-plan feedback with \
         safe_action none and a Claude Code user-stated instruction that may appear as \
         wrong-scope noise.",
        MemoryScope::project("engram"),
        ClaimOrigin::AgentObserved,
        writer(),
    )
    .with_status(MemoryStatus::Active)
    .with_confidence(0.8)
    .with_evidence(EvidenceRef::new(
        EvidenceKind::GitCommit,
        "t113-startup-validation",
    ))
    .with_tag("current-plan");
    current_plan.updated_at = now;
    let current_plan = memory_service.capture_memory(current_plan).await.unwrap();

    let results = search_service
        .search_with_options(
            "recent failures caveats open risks wrong-scope Claude Code user-stated \
             instruction stale current-plan safe_action none T113",
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

    assert_eq!(results[0].id, current_plan.id.to_string());

    let stale_plan_id = stale_repository_plan.id.to_string();
    let stale_plan_index = results
        .iter()
        .position(|result| result.id == stale_plan_id)
        .expect("expected stale repository current-plan noise in search results");
    assert!(
        stale_plan_index > 0,
        "stale repository current-plan noise must not outrank the latest project current plan"
    );

    let wrong_scope_rule_id = wrong_scope_rule.id.to_string();
    let wrong_scope_rule_index = results
        .iter()
        .position(|result| result.id == wrong_scope_rule_id)
        .expect("expected Claude Code rule noise in search results");
    assert!(
        wrong_scope_rule_index > 0,
        "wrong-scope Claude Code rule noise must not outrank the latest project current plan"
    );
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

    let mut noisy_calibration = MemoryItem::new(
        MemoryKind::ProjectFact,
        "Non-gated continuation search calibration landed",
        "T11 feedback stabilization confirmed the M6 gate must remain visible, but the \
         next-step query should still retrieve the current plan before calibration notes.",
        MemoryScope::project("engram"),
        ClaimOrigin::ToolResult,
        writer(),
    )
    .with_confidence(0.99)
    .with_evidence(EvidenceRef::new(
        EvidenceKind::ToolCall,
        "non-gated-calibration",
    ));
    noisy_calibration.updated_at = now;
    memory_service
        .capture_memory(noisy_calibration)
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

    let gate_context_results = search_service
        .search_with_options(
            "current plan next step non-gated Brain Harness completion T11 feedback \
             stabilization M6 gate",
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

    assert_eq!(gate_context_results[0].id, current_plan.id.to_string());

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
async fn test_memory_search_t40_mixed_query_surfaces_current_plan_and_m6_gate() {
    let (search_service, memory_service) = setup_search_and_memory_service().await;
    let now = OffsetDateTime::now_utc();

    let mut calibration = MemoryItem::new(
        MemoryKind::ProjectFact,
        "Non-gated continuation search calibration landed",
        "T11 feedback stabilization confirmed the M6 gate must remain visible, but the \
         next-step query should still retrieve the current plan before calibration notes.",
        MemoryScope::project("engram"),
        ClaimOrigin::ToolResult,
        writer(),
    )
    .with_confidence(0.99)
    .with_evidence(EvidenceRef::new(
        EvidenceKind::ToolCall,
        "non-gated-calibration",
    ));
    calibration.updated_at = now;
    memory_service.capture_memory(calibration).await.unwrap();

    let mut limitation_noise = MemoryItem::new(
        MemoryKind::Limitation,
        "Non-gated calibration does not prove broad ranking quality",
        "The non-gated continuation calibration fixes one prompt class but should not be \
         treated as proof that broad Brain Harness ranking quality is complete.",
        MemoryScope::project("engram"),
        ClaimOrigin::ToolResult,
        writer(),
    )
    .with_confidence(0.99)
    .with_evidence(EvidenceRef::new(
        EvidenceKind::ToolCall,
        "non-gated-limitation",
    ));
    limitation_noise.updated_at = now;
    memory_service
        .capture_memory(limitation_noise)
        .await
        .unwrap();

    let mut stale_repository_plan = MemoryItem::new(
        MemoryKind::Decision,
        "Current plan after Codex document lifecycle follow-through",
        "The next product-facing Brain Harness slice is complete for Codex adapter guidance. \
         Continue from this older current plan only as stale review noise.",
        MemoryScope::Repository {
            repository_id: None,
            remote_url: None,
            local_path: Some("/Users/yuval.meiri/projects/engram".to_string()),
        },
        ClaimOrigin::ToolResult,
        writer(),
    )
    .with_status(MemoryStatus::Active)
    .with_confidence(0.8)
    .with_evidence(EvidenceRef::new(EvidenceKind::GitCommit, "old-plan"))
    .with_tag("current-plan");
    stale_repository_plan.updated_at = now - time::Duration::days(6);
    let stale_repository_plan = memory_service
        .capture_memory(stale_repository_plan)
        .await
        .unwrap();

    let mut m6_gate = MemoryItem::new(
        MemoryKind::Limitation,
        "M6 migration approval gate remains explicit",
        "Brain Harness work must not run M6 migration read-only inventory or review export \
         without explicit user-approved scope. M6 write apply, deletion, cleanup, or legacy \
         simplification additionally require reviewed candidates, dry-run evidence, rollback \
         planning, and explicit approval.",
        MemoryScope::project("engram"),
        ClaimOrigin::UserStated,
        writer(),
    )
    .with_confidence(0.98)
    .with_evidence(EvidenceRef::new(
        EvidenceKind::ManualReview,
        "m6-approval-gate",
    ));
    m6_gate.updated_at = now - time::Duration::hours(2);
    let m6_gate = memory_service.capture_memory(m6_gate).await.unwrap();

    let mut current_plan = MemoryItem::new(
        MemoryKind::Decision,
        "T40 partial completion audit recorded; next work is gated",
        "T40 partial completion audit is recorded. Approved read-only surfaces remain coherent \
         enough to continue, but the next work must stay non-gated unless the user explicitly \
         approves a gated path.",
        MemoryScope::project("engram"),
        ClaimOrigin::AgentObserved,
        writer(),
    )
    .with_status(MemoryStatus::Active)
    .with_confidence(0.8)
    .with_evidence(EvidenceRef::new(EvidenceKind::GitCommit, "t40-audit"))
    .with_tag("current-plan");
    current_plan.updated_at = now;
    let current_plan = memory_service.capture_memory(current_plan).await.unwrap();

    let mixed_results = search_service
        .search_with_options(
            "current plan next non-gated Brain Harness feedback confidence M6 gate",
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

    assert_eq!(mixed_results[0].id, current_plan.id.to_string());
    let m6_gate_id = m6_gate.id.to_string();
    let m6_gate_index = mixed_results
        .iter()
        .position(|result| result.id == m6_gate_id)
        .expect("expected M6 gate in mixed-query memory results");
    assert!(
        m6_gate_index < 5,
        "expected M6 gate in first five memory results, got index {m6_gate_index}"
    );
    let stale_plan_id = stale_repository_plan.id.to_string();
    let stale_plan_index = mixed_results
        .iter()
        .position(|result| result.id == stale_plan_id)
        .expect("expected stale repository plan in mixed-query memory results");
    assert!(
        stale_plan_index > 0,
        "stale current-plan guidance must not outrank the latest current plan"
    );

    let explicit_gate_results = search_service
        .search_with_options(
            "approved M6 write apply deletion cleanup legacy simplification now",
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

    assert_eq!(explicit_gate_results[0].id, m6_gate.id.to_string());
}

#[tokio::test]
async fn test_memory_search_promotes_m6_gate_context_below_current_plan_for_mixed_query() {
    let (search_service, memory_service) = setup_search_and_memory_service().await;
    let now = OffsetDateTime::now_utc();

    for (index, (kind, title, content)) in [
        (
            MemoryKind::ProjectFact,
            "Non-gated continuation search calibration landed",
            "The current plan next non-gated Brain Harness feedback confidence M6 gate \
             query should retrieve the current plan before calibration notes.",
        ),
        (
            MemoryKind::Limitation,
            "Non-gated calibration does not prove broad ranking quality",
            "The non-gated continuation calibration mentions M6 gate context, feedback, \
             and confidence, but it is only ranking caveat noise.",
        ),
        (
            MemoryKind::ProjectFact,
            "AI Council and Claude next-step synthesis after orient contract",
            "Brain Harness current plan and M6 gate discussions should not expand orient \
             or migration behavior.",
        ),
        (
            MemoryKind::ProjectFact,
            "Brain Harness Architecture synced after orient contract checkpoint",
            "Current plan and feedback confidence evidence are useful, but broad ranking \
             quality remains unproven.",
        ),
        (
            MemoryKind::Rule,
            "Harness adapter and hook write approval gate",
            "Brain Harness work must not install adapters or hooks without approval. This \
             is not an M6 migration gate.",
        ),
        (
            MemoryKind::ProjectFact,
            "Memory OS completion is paused at migration review gate",
            "M6 migration apply must not proceed without reviewed candidates, a dry-run \
             report, rollback planning, and explicit approval.",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let mut item = MemoryItem::new(
            kind,
            title,
            content,
            MemoryScope::project("engram"),
            ClaimOrigin::ToolResult,
            writer(),
        )
        .with_confidence(0.99)
        .with_evidence(EvidenceRef::new(
            EvidenceKind::ToolCall,
            format!("mixed-noise-{index}"),
        ));
        item.updated_at = now - time::Duration::minutes(index as i64);
        memory_service.capture_memory(item).await.unwrap();
    }

    let mut stale_repository_plan = MemoryItem::new(
        MemoryKind::Decision,
        "Current plan after Codex document lifecycle follow-through",
        "The next product-facing Brain Harness slice is complete for Codex adapter guidance. \
         Continue from this older current plan only as stale review noise.",
        MemoryScope::Repository {
            repository_id: None,
            remote_url: None,
            local_path: Some("/Users/yuval.meiri/projects/engram".to_string()),
        },
        ClaimOrigin::ToolResult,
        writer(),
    )
    .with_status(MemoryStatus::Active)
    .with_confidence(0.8)
    .with_evidence(EvidenceRef::new(EvidenceKind::GitCommit, "old-plan"))
    .with_tag("current-plan");
    stale_repository_plan.updated_at = now - time::Duration::days(6);
    let stale_repository_plan = memory_service
        .capture_memory(stale_repository_plan)
        .await
        .unwrap();

    let mut m6_gate = MemoryItem::new(
        MemoryKind::Limitation,
        "M6 migration approval gate remains explicit",
        "Brain Harness work must not run M6 migration read-only inventory or review export \
         without explicit user-approved scope. M6 write apply, deletion, cleanup, or legacy \
         simplification additionally require reviewed candidates, dry-run evidence, rollback \
         planning, and explicit approval.",
        MemoryScope::project("engram"),
        ClaimOrigin::UserStated,
        writer(),
    )
    .with_confidence(0.98)
    .with_evidence(EvidenceRef::new(
        EvidenceKind::ManualReview,
        "m6-approval-gate",
    ));
    m6_gate.updated_at = now - time::Duration::hours(2);
    let m6_gate = memory_service.capture_memory(m6_gate).await.unwrap();

    let mut current_plan = MemoryItem::new(
        MemoryKind::Decision,
        "T42 baseline failed; next work is live mixed-query retrieval repair",
        "The next non-gated Brain Harness work is a prompt-specific mixed-query repair. \
         Continue from this current plan while preserving the M6 gate.",
        MemoryScope::project("engram"),
        ClaimOrigin::AgentObserved,
        writer(),
    )
    .with_status(MemoryStatus::Active)
    .with_confidence(0.8)
    .with_evidence(EvidenceRef::new(EvidenceKind::GitCommit, "t42-result"))
    .with_tag("current-plan");
    current_plan.updated_at = now;
    let current_plan = memory_service.capture_memory(current_plan).await.unwrap();

    let mixed_results = search_service
        .search_with_options(
            "current plan next non-gated Brain Harness feedback confidence M6 gate",
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

    assert_eq!(mixed_results[0].id, current_plan.id.to_string());
    let m6_gate_id = m6_gate.id.to_string();
    let m6_gate_index = mixed_results
        .iter()
        .position(|result| result.id == m6_gate_id)
        .expect("expected M6 gate in mixed-query memory results");
    assert!(
        m6_gate_index > 0 && m6_gate_index < 5,
        "expected M6 gate below current plan and in first five results, got index {m6_gate_index}"
    );
    let stale_plan_id = stale_repository_plan.id.to_string();
    let stale_plan_index = mixed_results
        .iter()
        .position(|result| result.id == stale_plan_id)
        .expect("expected stale repository plan in mixed-query memory results");
    assert!(
        stale_plan_index > 0,
        "stale current-plan guidance must not outrank the latest current plan"
    );

    let pure_continuation_results = search_service
        .search_with_options(
            "current plan next non-gated Brain Harness feedback confidence",
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

    assert_eq!(pure_continuation_results[0].id, current_plan.id.to_string());
    assert!(
        pure_continuation_results
            .iter()
            .position(|result| result.id == m6_gate_id)
            .map(|index| index >= 5)
            .unwrap_or(true),
        "pure continuation query should not newly promote M6 gate into top five"
    );

    let explicit_gate_results = search_service
        .search_with_options(
            "approved M6 write apply deletion cleanup legacy simplification now",
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

    assert_ne!(explicit_gate_results[0].id, current_plan.id.to_string());
    assert!(
        explicit_gate_results
            .iter()
            .take(2)
            .any(|result| result.id == m6_gate.id.to_string()),
        "explicit gate query should keep active M6 gate in top gate context"
    );
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

#[tokio::test]
async fn test_memory_search_promotes_live_like_migration_gate_over_calibration_noise() {
    let (search_service, memory_service) = setup_search_and_memory_service().await;
    let now = OffsetDateTime::now_utc();

    let mut calibration = MemoryItem::new(
        MemoryKind::ProjectFact,
        "Non-gated continuation search calibration landed",
        "T11 feedback stabilization confirmed the M6 gate must remain visible, but the \
         next-step query should still retrieve the current plan before calibration notes.",
        MemoryScope::project("engram"),
        ClaimOrigin::ToolResult,
        writer(),
    )
    .with_confidence(0.99)
    .with_evidence(EvidenceRef::new(
        EvidenceKind::ToolCall,
        "non-gated-calibration",
    ));
    calibration.updated_at = now;
    memory_service.capture_memory(calibration).await.unwrap();

    let mut current_plan = MemoryItem::new(
        MemoryKind::Decision,
        "Current plan after T13 installed-runtime validation",
        "The next non-gated Brain Harness slice should investigate explicit \
         migration-apply gate queries; do not run M6 inventory or write apply.",
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
    current_plan.updated_at = now - time::Duration::minutes(1);
    memory_service.capture_memory(current_plan).await.unwrap();

    let mut broad_contract = MemoryItem::new(
        MemoryKind::Rule,
        "Lean orient contract is a presentation option",
        "`orient` should keep migration, graph, and lint outside the hot path; do not \
         expand the payload from a gate query.",
        MemoryScope::project("engram"),
        ClaimOrigin::UserStated,
        writer(),
    )
    .with_confidence(0.96)
    .with_evidence(EvidenceRef::new(
        EvidenceKind::ManualReview,
        "orient-contract",
    ));
    broad_contract.updated_at = now;
    memory_service.capture_memory(broad_contract).await.unwrap();

    let mut broad_implementation_history = MemoryItem::new(
        MemoryKind::Decision,
        "Memory OS harness completion implementation landed",
        "Implemented Memory OS harness completion slice with dry-run session distillation \
         candidates and implementation-plan checklist updates. Migration remains \
         review-gated with no automatic promotion from orphan/digest/legacy data.",
        MemoryScope::project("engram"),
        ClaimOrigin::AgentObserved,
        writer(),
    )
    .with_confidence(0.98)
    .with_evidence(EvidenceRef::new(
        EvidenceKind::ToolCall,
        "implementation-history",
    ));
    broad_implementation_history.updated_at = now;
    memory_service
        .capture_memory(broad_implementation_history)
        .await
        .unwrap();

    let mut reviewed_batch_status = MemoryItem::new(
        MemoryKind::ProjectFact,
        "First Memory OS migration review batch has conservative decisions and dry-run validation",
        "The first migration review batch was marked with conservative decisions and \
         validated without migration writes. Next step requires explicit user approval \
         immediately before any migration --write apply.",
        MemoryScope::project("engram"),
        ClaimOrigin::ToolResult,
        writer(),
    )
    .with_confidence(0.98)
    .with_evidence(EvidenceRef::new(
        EvidenceKind::ManualReview,
        "reviewed-batch-status",
    ));
    reviewed_batch_status.updated_at = now;
    memory_service
        .capture_memory(reviewed_batch_status)
        .await
        .unwrap();

    let mut migration_gate = MemoryItem::new(
        MemoryKind::ProjectFact,
        "Memory OS completion is paused at migration review gate",
        "M6 migration apply must not proceed without reviewed candidates, a dry-run \
         report, rollback planning, and explicit approval.",
        MemoryScope::project("engram"),
        ClaimOrigin::ToolResult,
        writer(),
    )
    .with_confidence(0.95)
    .with_evidence(EvidenceRef::new(
        EvidenceKind::ManualReview,
        "migration-gate",
    ));
    migration_gate.updated_at = now - time::Duration::days(30);
    let migration_gate = memory_service.capture_memory(migration_gate).await.unwrap();

    let mut old_approval = MemoryItem::new(
        MemoryKind::ProjectFact,
        "Approved repo topology migration write applied first batch",
        "After explicit user approval, the first repository topology migration write was \
         applied for an older reviewed batch.",
        MemoryScope::project("engram"),
        ClaimOrigin::ToolResult,
        writer(),
    )
    .with_confidence(0.98)
    .with_evidence(EvidenceRef::new(
        EvidenceKind::ManualReview,
        "old-migration-approval",
    ));
    old_approval.updated_at = now;
    memory_service.capture_memory(old_approval).await.unwrap();

    for query in [
        "Should we proceed with migration apply?",
        "next non-gated step, should we proceed with migration apply?",
    ] {
        let results = search_service
            .search_with_options(
                query,
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

        assert_eq!(results[0].id, migration_gate.id.to_string());
    }
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
