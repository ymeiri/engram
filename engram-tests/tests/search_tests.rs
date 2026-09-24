//! Integration tests for unified search across all layers.
//!
//! Tests the SearchService that searches entities, aliases, observations,
//! session events, documents, and tool usages with a single query.

use engram_core::entity::{EntityType, RelationType};
use engram_core::memory::{
    ClaimOrigin, EvidenceKind, EvidenceRef, Harness, MemoryFreshness, MemoryItem, MemoryKind,
    MemoryReviewState, MemoryScope, MemoryStatus, ModelIdentity, WriterProvenance,
};
use engram_core::repository::ProjectRepositoryRole;
use engram_core::search::{SearchLayer, SearchResultSource};
use engram_core::session::EventType;
use engram_core::tool::ToolOutcome;
use engram_index::{
    EntityService, MemoryService, RepositoryService, SearchOptions, SearchService, SessionService,
    ToolIntelService, WorkService,
};
use engram_mcp::tools::{
    self as mcp_tools, DocsRequestNew, EntityObserveRequestNew, EntityRequestNew,
    EntityStatsRequest, KnowledgeStatsRequest, RetrievalScopeRequest, SessionRequestNew,
    SessionStatsRequest, ToolIntelStatsRequest, ToolRequestNew, ToolState, WorkStatsRequest,
};
use engram_store::{connect_and_init, StoreConfig};
use serde_json::{json, Value};
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;
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

async fn setup_related_search_service() -> (
    SearchService,
    EntityService,
    SessionService,
    ToolIntelService,
    WorkService,
    MemoryService,
) {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config).await.expect("Failed to connect");
    let entity_service = EntityService::new(db.clone());
    entity_service.init().await.unwrap();
    let session_service = SessionService::new(db.clone());
    session_service.init().await.unwrap();
    let tool_service = ToolIntelService::new(db.clone());
    tool_service.init().await.unwrap();
    let work_service = WorkService::new(db.clone());
    work_service.init().await.unwrap();
    let memory_service = MemoryService::new(db.clone());
    memory_service.init_schema().await.unwrap();

    (
        SearchService::new(db),
        entity_service,
        session_service,
        tool_service,
        work_service,
        memory_service,
    )
}

fn writer() -> WriterProvenance {
    WriterProvenance::agent(Harness::Codex, ModelIdentity::new("openai", "gpt-5.5"))
        .with_surface("search-test")
}

fn git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn run_git(cwd: &Path, args: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(args)
        .output()
        .expect("git should run");
    assert!(
        output.status.success(),
        "git {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
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
    assert_eq!(metadata.review_state, MemoryReviewState::ActiveUnreviewed);
    assert!(metadata.review_asserted);
    assert!(!metadata.reviewed);
    assert_eq!(metadata.freshness, MemoryFreshness::Unscheduled);
    assert_eq!(metadata.claim_origin, ClaimOrigin::UserStated);
    assert_eq!(metadata.evidence_count, 1);
    assert_eq!(metadata.writer.harness, Harness::Codex);
    assert!(result
        .context
        .as_deref()
        .unwrap()
        .contains("review_state: active_unreviewed"));
}

#[tokio::test]
async fn test_memory_search_metadata_reports_review_assertion_and_evidence() {
    let (search_service, memory_service) = setup_search_and_memory_service().await;
    let item = MemoryItem::new(
        MemoryKind::Decision,
        "Review assertion memory metadata",
        "Unified search should expose evidence-backed trust metadata without treating caller-supplied review claims as authority.",
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
            "review assertion evidence-backed trust metadata",
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
    assert_eq!(metadata.review_state, MemoryReviewState::ActiveUnreviewed);
    assert!(!metadata.reviewed);
    assert!(metadata.review_asserted);
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
        Some(MemoryReviewState::ActiveUnreviewed)
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
        Some(MemoryReviewState::ActiveUnreviewed)
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
        Some(MemoryReviewState::ActiveUnreviewed)
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
async fn test_memory_search_t143_current_handoff_does_not_outrank_current_plan() {
    let (search_service, memory_service) = setup_search_and_memory_service().await;
    let now = OffsetDateTime::now_utc();

    let mut latest_handoff = MemoryItem::new(
        MemoryKind::Handoff,
        "Rolling handoff",
        "T142 is complete. Commit 293b322 records a source-only validation baseline after \
         T140/T141. The next product-moving step is still exact T141 approval: install the \
         current engram-cli binary, restart the daemon, and run read-only live validation of the \
         T140 continuation/current-plan approval-gate-context query class only.",
        MemoryScope::project("engram"),
        ClaimOrigin::AgentObserved,
        writer(),
    )
    .with_status(MemoryStatus::Active)
    .with_confidence(0.99)
    .with_evidence(EvidenceRef::new(
        EvidenceKind::ToolCall,
        "t143-latest-handoff",
    ));
    latest_handoff.updated_at = now;
    let latest_handoff = memory_service.capture_memory(latest_handoff).await.unwrap();

    let mut current_plan = MemoryItem::new(
        MemoryKind::Decision,
        "Current plan after T142 source validation baseline",
        "T142 is complete as a source-only validation baseline. Continue toward the Engram Brain \
         Harness goal from this current plan. The next product-moving slice remains T141: install \
         the current engram-cli binary, restart the daemon, and run read-only live validation of \
         the T140 continuation/current-plan approval-gate-context query class only.",
        MemoryScope::project("engram"),
        ClaimOrigin::AgentObserved,
        writer(),
    )
    .with_status(MemoryStatus::Active)
    .with_confidence(0.8)
    .with_evidence(EvidenceRef::new(EvidenceKind::GitCommit, "293b322"))
    .with_tag("current-plan");
    current_plan.updated_at = now - time::Duration::minutes(1);
    let current_plan = memory_service.capture_memory(current_plan).await.unwrap();

    let results = search_service
        .search_with_options(
            "current plan next step continue move forward Engram Brain Harness after T142 T141 \
             T140 approval gates",
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
            .any(|result| result.id == latest_handoff.id.to_string()),
        "fresh handoff context should remain retrievable"
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

// =============================================================================
// Authorization Scope Tests
// =============================================================================

#[tokio::test]
async fn local_memory_search_uses_stable_repository_identity_without_cross_project_broadening() {
    if !git_available() {
        return;
    }

    let db = connect_and_init(&StoreConfig::memory()).await.unwrap();
    let work = WorkService::new(db.clone());
    work.init().await.unwrap();
    work.create_project("repo-scope-atlas", None).await.unwrap();
    work.create_project("repo-scope-orbit", None).await.unwrap();
    let repositories = RepositoryService::new(db.clone());
    repositories.init_schema().await.unwrap();
    let memory = MemoryService::new(db.clone());
    memory.init_schema().await.unwrap();

    let root = tempdir().unwrap();
    let atlas_primary = root.path().join("atlas-primary");
    let atlas_moved = root.path().join("arbitrary-moved-name");
    let orbit = root.path().join("orbit");
    for (checkout, remote) in [
        (&atlas_primary, "git@github.com:acme/atlas.git"),
        (&atlas_moved, "https://github.com/acme/atlas.git"),
        (&orbit, "git@github.com:acme/orbit.git"),
    ] {
        std::fs::create_dir_all(checkout).unwrap();
        run_git(checkout, &["init"]);
        run_git(checkout, &["remote", "add", "origin", remote]);
    }

    let atlas = repositories
        .detect_repository(&atlas_primary)
        .await
        .unwrap()
        .context
        .repository;
    repositories
        .link_project(
            "repo-scope-atlas",
            Some(&atlas.id),
            None,
            ProjectRepositoryRole::Primary,
            None,
        )
        .await
        .unwrap();
    let moved = repositories.detect_repository(&atlas_moved).await.unwrap();
    assert_eq!(moved.context.repository.id, atlas.id);

    let orbit_repository = repositories
        .detect_repository(&orbit)
        .await
        .unwrap()
        .context
        .repository;
    repositories
        .link_project(
            "repo-scope-orbit",
            Some(&orbit_repository.id),
            None,
            ProjectRepositoryRole::Primary,
            None,
        )
        .await
        .unwrap();

    let atlas_memory = memory
        .capture_memory(
            MemoryItem::new(
                MemoryKind::Decision,
                "repository identity canary atlas",
                "Use the Atlas repository-scoped invocation.",
                MemoryScope::Repository {
                    repository_id: Some(atlas.id),
                    remote_url: atlas.remote_url.clone(),
                    local_path: None,
                },
                ClaimOrigin::AgentObserved,
                writer(),
            )
            .with_evidence(EvidenceRef::new(EvidenceKind::File, "atlas-adr.md")),
        )
        .await
        .unwrap();
    let orbit_memory = memory
        .capture_memory(
            MemoryItem::new(
                MemoryKind::Decision,
                "repository identity canary orbit",
                "Use the Orbit repository-scoped invocation.",
                MemoryScope::Repository {
                    repository_id: Some(orbit_repository.id),
                    remote_url: orbit_repository.remote_url.clone(),
                    local_path: None,
                },
                ClaimOrigin::AgentObserved,
                writer(),
            )
            .with_evidence(EvidenceRef::new(EvidenceKind::File, "orbit-adr.md")),
        )
        .await
        .unwrap();

    let search = SearchService::new(db);
    let moved_results = search
        .search_local_memory(
            "repository identity canary",
            10,
            Some(0.0),
            &SearchOptions {
                project: Some("repo-scope-atlas".to_string()),
                cwd: Some(atlas_moved.display().to_string()),
            },
            None,
        )
        .await
        .unwrap();
    assert!(moved_results
        .iter()
        .any(|result| result.id == atlas_memory.id.to_string()));
    assert!(moved_results
        .iter()
        .all(|result| result.id != orbit_memory.id.to_string()));

    let compatibility_results = search
        .search_with_options(
            "repository identity canary",
            10,
            Some(0.0),
            Some(&[SearchLayer::Memory]),
            SearchOptions {
                project: Some("repo-scope-atlas".to_string()),
                cwd: Some(atlas_moved.display().to_string()),
            },
        )
        .await
        .unwrap();
    assert!(compatibility_results
        .iter()
        .any(|result| result.id == atlas_memory.id.to_string()));
    assert!(compatibility_results
        .iter()
        .all(|result| result.id != orbit_memory.id.to_string()));

    let related_results = search
        .search_related(
            "repository identity canary",
            10,
            Some(0.0),
            Some(&[SearchLayer::Memory]),
            &SearchOptions {
                project: Some("repo-scope-atlas".to_string()),
                cwd: Some(atlas_moved.display().to_string()),
            },
            None,
        )
        .await
        .unwrap();
    assert!(related_results
        .results
        .iter()
        .any(|result| result.id == atlas_memory.id.to_string()));
    assert!(related_results
        .results
        .iter()
        .all(|result| result.id != orbit_memory.id.to_string()));

    let cwd_only_results = search
        .search_local_memory(
            "repository identity canary",
            10,
            Some(0.0),
            &SearchOptions {
                project: None,
                cwd: Some(atlas_moved.display().to_string()),
            },
            None,
        )
        .await
        .unwrap();
    assert!(cwd_only_results
        .iter()
        .any(|result| result.id == atlas_memory.id.to_string()));
    assert!(cwd_only_results
        .iter()
        .all(|result| result.id != orbit_memory.id.to_string()));

    let conflicting_results = search
        .search_local_memory(
            "repository identity canary",
            10,
            Some(0.0),
            &SearchOptions {
                project: Some("repo-scope-atlas".to_string()),
                cwd: Some(orbit.display().to_string()),
            },
            None,
        )
        .await
        .unwrap();
    assert!(conflicting_results.iter().all(|result| {
        result.id != atlas_memory.id.to_string() && result.id != orbit_memory.id.to_string()
    }));
}

#[tokio::test]
async fn related_retrieval_never_returns_another_projects_legacy_records() {
    let (search, entities, sessions, tools, work, _) = setup_related_search_service().await;
    work.create_project("scope-alpha", None).await.unwrap();
    work.create_project("scope-beta", None).await.unwrap();
    work.create_task("scope-alpha", "alpha-task", None, Some("ALPHA-1"))
        .await
        .unwrap();
    work.create_task("scope-beta", "beta-task", None, Some("BETA-1"))
        .await
        .unwrap();
    work.add_pr(
        "scope-alpha",
        Some("alpha-task"),
        "https://github.com/example/alpha/pull/1",
        None,
    )
    .await
    .unwrap();
    work.add_pr(
        "scope-beta",
        Some("beta-task"),
        "https://github.com/example/beta/pull/1",
        None,
    )
    .await
    .unwrap();
    work.add_project_observation("scope-alpha", "alpha project observation", None)
        .await
        .unwrap();
    work.add_project_observation("scope-beta", "beta project observation", None)
        .await
        .unwrap();
    work.add_task_observation("ALPHA-1", "alpha task observation", None)
        .await
        .unwrap();
    work.add_task_observation("BETA-1", "beta task observation", None)
        .await
        .unwrap();

    let alpha_entity = entities
        .create_entity(
            "alpha-service",
            EntityType::Service,
            Some("scope-canary alpha entity"),
        )
        .await
        .unwrap();
    let beta_entity = entities
        .create_entity(
            "beta-service",
            EntityType::Service,
            Some("scope-canary beta entity"),
        )
        .await
        .unwrap();
    entities
        .add_alias("alpha-service", "scope-canary-alpha-alias")
        .await
        .unwrap();
    entities
        .add_alias("beta-service", "scope-canary-beta-alias")
        .await
        .unwrap();
    work.connect_project_to_entity("scope-alpha", "alpha-service", None)
        .await
        .unwrap();
    work.connect_project_to_entity("scope-beta", "beta-service", None)
        .await
        .unwrap();
    entities
        .relate("alpha-service", RelationType::RelatedTo, "beta-service")
        .await
        .unwrap();
    let alpha_observation = entities
        .add_observation(
            "alpha-service",
            "scope-canary alpha observation",
            Some("scope.canary"),
            None,
        )
        .await
        .unwrap()
        .0;
    let beta_observation = entities
        .add_observation(
            "beta-service",
            "scope-canary beta observation",
            Some("scope.canary"),
            None,
        )
        .await
        .unwrap()
        .0;

    let alpha_session = sessions
        .start_session(Some("codex"), Some("scope-alpha"), None)
        .await
        .unwrap();
    let beta_session = sessions
        .start_session(Some("codex"), Some("scope-beta"), None)
        .await
        .unwrap();
    let alpha_event = sessions
        .log_event(
            &alpha_session.id,
            EventType::Observation,
            "scope-canary alpha session event",
            None,
            None,
        )
        .await
        .unwrap();
    let beta_event = sessions
        .log_event(
            &beta_session.id,
            EventType::Observation,
            "scope-canary beta session event",
            None,
            None,
        )
        .await
        .unwrap();
    entities
        .create_entity("scope-alpha-tool", EntityType::Tool, None)
        .await
        .unwrap();
    entities
        .create_entity("scope-beta-tool", EntityType::Tool, None)
        .await
        .unwrap();
    let alpha_usage = tools
        .log_usage(
            "scope-alpha-tool",
            "scope-canary alpha tool usage",
            ToolOutcome::Success,
            Some(&alpha_session.id),
        )
        .await
        .unwrap();
    let beta_usage = tools
        .log_usage(
            "scope-beta-tool",
            "scope-canary beta tool usage",
            ToolOutcome::Success,
            Some(&beta_session.id),
        )
        .await
        .unwrap();

    let related = search
        .search_related(
            "scope-canary",
            20,
            Some(0.0),
            None,
            &SearchOptions {
                project: Some("scope-alpha".to_string()),
                cwd: None,
            },
            None,
        )
        .await
        .unwrap();

    assert_eq!(related.project, "scope-alpha");
    assert!(related.omitted_layers.contains(&SearchLayer::Document));
    for expected_id in [
        alpha_entity.id,
        alpha_observation.id,
        alpha_event.id,
        alpha_usage.id,
    ] {
        assert!(
            related
                .results
                .iter()
                .any(|result| result.id == expected_id.to_string()),
            "missing alpha result {expected_id}"
        );
    }
    for forbidden_id in [
        beta_entity.id,
        beta_observation.id,
        beta_event.id,
        beta_usage.id,
    ] {
        assert!(
            related
                .results
                .iter()
                .all(|result| result.id != forbidden_id.to_string()),
            "related search leaked beta result {forbidden_id}"
        );
    }

    let defensive_scoped_api = search
        .search_with_options(
            "scope-canary",
            20,
            Some(0.0),
            None,
            SearchOptions {
                project: Some("scope-alpha".to_string()),
                cwd: None,
            },
        )
        .await
        .unwrap();
    assert!(defensive_scoped_api
        .iter()
        .all(|result| result.id != beta_observation.id.to_string()));
    assert!(defensive_scoped_api
        .iter()
        .all(|result| result.id != beta_event.id.to_string()));
    assert!(defensive_scoped_api
        .iter()
        .all(|result| result.id != beta_usage.id.to_string()));

    let global = search
        .search("scope-canary", 20, Some(0.0), None)
        .await
        .unwrap();
    assert!(global
        .iter()
        .any(|result| result.id == beta_observation.id.to_string()));
    assert!(global
        .iter()
        .any(|result| result.id == beta_event.id.to_string()));
    assert!(global
        .iter()
        .any(|result| result.id == beta_usage.id.to_string()));

    let state = ToolState::new();
    state.init_entity(entities).await;
    state.init_session(sessions).await;
    state.init_tool_intel(tools).await;
    state.init_work(work).await;
    state.init_search(search).await;
    let related_scope = RetrievalScopeRequest {
        relevance_mode: Some("related".to_string()),
        project: Some("scope-alpha".to_string()),
        ..RetrievalScopeRequest::default()
    };
    let global_scope = RetrievalScopeRequest {
        relevance_mode: Some("global".to_string()),
        ..RetrievalScopeRequest::default()
    };
    let parse = |response: String| serde_json::from_str::<Value>(&response).unwrap();

    let local_entity = parse(
        mcp_tools::entity_new(
            &state,
            serde_json::from_value::<EntityRequestNew>(json!({
                "action": "search",
                "query": "service"
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(local_entity["count"], 0);
    assert_eq!(local_entity["relevance_mode"], "local");
    assert_eq!(local_entity["omitted_layers"], json!(["entity"]));

    let related_entity = parse(
        mcp_tools::entity_new(
            &state,
            serde_json::from_value::<EntityRequestNew>(json!({
                "action": "search",
                "query": "service",
                "search_scope": related_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(related_entity["count"], 1);
    assert!(related_entity
        .to_string()
        .contains(&alpha_entity.id.to_string()));
    assert!(!related_entity
        .to_string()
        .contains(&beta_entity.id.to_string()));
    assert_eq!(related_entity["scope_enforced_layers"], json!(["entity"]));

    let global_entity = parse(
        mcp_tools::entity_new(
            &state,
            serde_json::from_value::<EntityRequestNew>(json!({
                "action": "search",
                "query": "service",
                "search_scope": global_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert!(global_entity
        .to_string()
        .contains(&alpha_entity.id.to_string()));
    assert!(global_entity
        .to_string()
        .contains(&beta_entity.id.to_string()));
    assert_eq!(global_entity["authorization_scope_enforced"], false);

    let local_observation = parse(
        mcp_tools::entity_observe_new(
            &state,
            serde_json::from_value::<EntityObserveRequestNew>(json!({
                "action": "search",
                "entity": "alpha-service",
                "query": "scope-canary"
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(local_observation["count"], 0);
    assert_eq!(local_observation["omitted_layers"], json!(["observation"]));

    let related_other_observation = parse(
        mcp_tools::entity_observe_new(
            &state,
            serde_json::from_value::<EntityObserveRequestNew>(json!({
                "action": "search",
                "entity": "beta-service",
                "query": "scope-canary",
                "search_scope": related_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(related_other_observation["count"], 0);
    assert!(!related_other_observation
        .to_string()
        .contains(&beta_observation.id.to_string()));

    let related_observation = parse(
        mcp_tools::entity_observe_new(
            &state,
            serde_json::from_value::<EntityObserveRequestNew>(json!({
                "action": "search",
                "entity": "alpha-service",
                "query": "scope-canary",
                "search_scope": related_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert!(related_observation
        .to_string()
        .contains(&alpha_observation.id.to_string()));
    assert_eq!(
        related_observation["scope_enforced_layers"],
        json!(["observation"])
    );

    let global_observation = parse(
        mcp_tools::entity_observe_new(
            &state,
            serde_json::from_value::<EntityObserveRequestNew>(json!({
                "action": "search",
                "entity": "beta-service",
                "query": "scope-canary",
                "search_scope": global_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert!(global_observation
        .to_string()
        .contains(&beta_observation.id.to_string()));

    let local_session = parse(
        mcp_tools::session_new(
            &state,
            serde_json::from_value::<SessionRequestNew>(json!({
                "action": "search",
                "query": "scope-canary"
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(local_session["count"], 0);
    assert_eq!(local_session["omitted_layers"], json!(["session_event"]));

    let related_session = parse(
        mcp_tools::session_new(
            &state,
            serde_json::from_value::<SessionRequestNew>(json!({
                "action": "search",
                "query": "scope-canary",
                "search_scope": related_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert!(related_session
        .to_string()
        .contains(&alpha_event.id.to_string()));
    assert!(!related_session
        .to_string()
        .contains(&beta_event.id.to_string()));

    let global_session = parse(
        mcp_tools::session_new(
            &state,
            serde_json::from_value::<SessionRequestNew>(json!({
                "action": "search",
                "query": "scope-canary",
                "search_scope": global_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert!(global_session
        .to_string()
        .contains(&alpha_event.id.to_string()));
    assert!(global_session
        .to_string()
        .contains(&beta_event.id.to_string()));

    let local_tool = parse(
        mcp_tools::tool_new(
            &state,
            serde_json::from_value::<ToolRequestNew>(json!({
                "action": "search",
                "query": "scope-canary"
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(local_tool["count"], 0);
    assert_eq!(local_tool["omitted_layers"], json!(["tool_usage"]));

    let related_tool = parse(
        mcp_tools::tool_new(
            &state,
            serde_json::from_value::<ToolRequestNew>(json!({
                "action": "search",
                "query": "scope-canary",
                "search_scope": related_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert!(related_tool
        .to_string()
        .contains(&alpha_usage.id.to_string()));
    assert!(!related_tool
        .to_string()
        .contains(&beta_usage.id.to_string()));

    let global_tool = parse(
        mcp_tools::tool_new(
            &state,
            serde_json::from_value::<ToolRequestNew>(json!({
                "action": "search",
                "query": "scope-canary",
                "search_scope": global_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert!(global_tool
        .to_string()
        .contains(&alpha_usage.id.to_string()));
    assert!(global_tool.to_string().contains(&beta_usage.id.to_string()));

    let local_entity_get = parse(
        mcp_tools::entity_new(
            &state,
            serde_json::from_value::<EntityRequestNew>(json!({
                "action": "get",
                "name": "alpha-service"
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(local_entity_get["found"], false);
    assert_eq!(local_entity_get["omitted_layers"], json!(["entity"]));

    let related_other_entity_get = parse(
        mcp_tools::entity_new(
            &state,
            serde_json::from_value::<EntityRequestNew>(json!({
                "action": "get",
                "name": "beta-service",
                "scope": related_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(related_other_entity_get["found"], false);
    assert!(!related_other_entity_get
        .to_string()
        .contains(&beta_entity.id.to_string()));

    let related_entity_get = parse(
        mcp_tools::entity_new(
            &state,
            serde_json::from_value::<EntityRequestNew>(json!({
                "action": "get",
                "name": "alpha-service",
                "scope": related_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(related_entity_get["found"], true);
    assert!(related_entity_get
        .to_string()
        .contains(&alpha_observation.id.to_string()));
    assert!(!related_entity_get.to_string().contains("beta-service"));

    let global_entity_get = parse(
        mcp_tools::entity_new(
            &state,
            serde_json::from_value::<EntityRequestNew>(json!({
                "action": "get",
                "name": "alpha-service",
                "scope": global_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert!(global_entity_get.to_string().contains("beta-service"));
    assert_eq!(global_entity_get["authorization_scope_enforced"], false);

    let related_entity_list = parse(
        mcp_tools::entity_new(
            &state,
            serde_json::from_value::<EntityRequestNew>(json!({
                "action": "list",
                "scope": related_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert!(related_entity_list
        .to_string()
        .contains(&alpha_entity.id.to_string()));
    assert!(!related_entity_list
        .to_string()
        .contains(&beta_entity.id.to_string()));

    let related_other_observation_get = parse(
        mcp_tools::entity_observe_new(
            &state,
            serde_json::from_value::<EntityObserveRequestNew>(json!({
                "action": "get",
                "entity": "beta-service",
                "key": "scope.canary",
                "scope": related_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(related_other_observation_get["found"], false);
    assert!(!related_other_observation_get
        .to_string()
        .contains(&beta_observation.id.to_string()));

    let related_observation_get = parse(
        mcp_tools::entity_observe_new(
            &state,
            serde_json::from_value::<EntityObserveRequestNew>(json!({
                "action": "get",
                "entity": "alpha-service",
                "key": "scope.canary",
                "scope": related_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert!(related_observation_get
        .to_string()
        .contains(&alpha_observation.id.to_string()));

    let related_other_session_get = parse(
        mcp_tools::session_new(
            &state,
            serde_json::from_value::<SessionRequestNew>(json!({
                "action": "get",
                "session_id": beta_session.id.to_string(),
                "scope": related_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(related_other_session_get["found"], false);
    assert!(!related_other_session_get
        .to_string()
        .contains(&beta_event.id.to_string()));

    let related_session_get = parse(
        mcp_tools::session_new(
            &state,
            serde_json::from_value::<SessionRequestNew>(json!({
                "action": "get",
                "session_id": alpha_session.id.to_string(),
                "scope": related_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert!(related_session_get
        .to_string()
        .contains(&alpha_event.id.to_string()));

    let related_session_list = parse(
        mcp_tools::session_new(
            &state,
            serde_json::from_value::<SessionRequestNew>(json!({
                "action": "list",
                "scope": related_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert!(related_session_list
        .to_string()
        .contains(&alpha_session.id.to_string()));
    assert!(!related_session_list
        .to_string()
        .contains(&beta_session.id.to_string()));

    let local_recommendations = parse(
        mcp_tools::tool_new(
            &state,
            serde_json::from_value::<ToolRequestNew>(json!({
                "action": "recommend",
                "context": "scope-canary"
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(local_recommendations["count"], 0);

    let related_recommendations = parse(
        mcp_tools::tool_new(
            &state,
            serde_json::from_value::<ToolRequestNew>(json!({
                "action": "recommend",
                "context": "scope-canary",
                "scope": related_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert!(related_recommendations
        .to_string()
        .contains("scope-alpha-tool"));
    assert!(!related_recommendations
        .to_string()
        .contains("scope-beta-tool"));

    let global_recommendations = parse(
        mcp_tools::tool_new(
            &state,
            serde_json::from_value::<ToolRequestNew>(json!({
                "action": "recommend",
                "context": "scope-canary",
                "scope": global_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert!(global_recommendations
        .to_string()
        .contains("scope-alpha-tool"));
    assert!(global_recommendations
        .to_string()
        .contains("scope-beta-tool"));

    let related_tool_list = parse(
        mcp_tools::tool_new(
            &state,
            serde_json::from_value::<ToolRequestNew>(json!({
                "action": "list",
                "scope": related_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert!(related_tool_list
        .to_string()
        .contains(&alpha_usage.id.to_string()));
    assert!(!related_tool_list
        .to_string()
        .contains(&beta_usage.id.to_string()));

    let related_alpha_stats = parse(
        mcp_tools::tool_new(
            &state,
            serde_json::from_value::<ToolRequestNew>(json!({
                "action": "stats",
                "tool_name": "scope-alpha-tool",
                "scope": related_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(related_alpha_stats["total_usages"], 1);
    assert_eq!(related_alpha_stats["preferences_count"], 0);

    let related_beta_stats = parse(
        mcp_tools::tool_new(
            &state,
            serde_json::from_value::<ToolRequestNew>(json!({
                "action": "stats",
                "tool_name": "scope-beta-tool",
                "scope": related_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(related_beta_stats["total_usages"], 0);

    let local_entity_stats = parse(
        mcp_tools::entity_stats(
            &state,
            serde_json::from_value::<EntityStatsRequest>(json!({})).unwrap(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(local_entity_stats["entity_count"], 0);
    assert_eq!(
        local_entity_stats["omitted_layers"],
        json!(["entity_stats"])
    );

    let related_entity_stats = parse(
        mcp_tools::entity_stats(
            &state,
            serde_json::from_value::<EntityStatsRequest>(json!({
                "scope": related_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(related_entity_stats["entity_count"], 1);
    assert_eq!(related_entity_stats["alias_count"], 2);
    assert_eq!(related_entity_stats["observation_count"], 1);
    assert_eq!(related_entity_stats["relationship_count"], 0);
    assert_eq!(
        related_entity_stats["scope_enforced_layers"],
        json!(["entity_stats"])
    );

    let global_entity_stats = parse(
        mcp_tools::entity_stats(
            &state,
            serde_json::from_value::<EntityStatsRequest>(json!({
                "scope": global_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(global_entity_stats["entity_count"], 4);
    assert_eq!(global_entity_stats["relationship_count"], 1);
    assert_eq!(global_entity_stats["alias_count"], 6);
    assert_eq!(global_entity_stats["authorization_scope_enforced"], false);

    let related_session_stats = parse(
        mcp_tools::session_stats(
            &state,
            serde_json::from_value::<SessionStatsRequest>(json!({
                "scope": related_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(related_session_stats["total_sessions"], 1);
    assert_eq!(related_session_stats["total_events"], 1);
    assert_eq!(related_session_stats["events_by_type"]["observation"], 1);

    let global_session_stats = parse(
        mcp_tools::session_stats(
            &state,
            serde_json::from_value::<SessionStatsRequest>(json!({
                "scope": global_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(global_session_stats["total_sessions"], 2);
    assert_eq!(global_session_stats["total_events"], 2);

    let related_tool_intel_stats = parse(
        mcp_tools::tool_intel_stats(
            &state,
            serde_json::from_value::<ToolIntelStatsRequest>(json!({
                "scope": related_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(related_tool_intel_stats["usage_count"], 1);
    assert_eq!(related_tool_intel_stats["preference_count"], 0);

    let global_tool_intel_stats = parse(
        mcp_tools::tool_intel_stats(
            &state,
            serde_json::from_value::<ToolIntelStatsRequest>(json!({
                "scope": global_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(global_tool_intel_stats["usage_count"], 2);

    let related_work_stats = parse(
        mcp_tools::work_stats(
            &state,
            serde_json::from_value::<WorkStatsRequest>(json!({
                "scope": related_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(related_work_stats["project_count"], 1);
    assert_eq!(related_work_stats["task_count"], 1);
    assert_eq!(related_work_stats["pr_count"], 1);
    assert_eq!(related_work_stats["project_observation_count"], 1);
    assert_eq!(related_work_stats["task_observation_count"], 1);

    let global_work_stats = parse(
        mcp_tools::work_stats(
            &state,
            serde_json::from_value::<WorkStatsRequest>(json!({
                "scope": global_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(global_work_stats["project_count"], 2);
    assert_eq!(global_work_stats["task_count"], 2);
    assert_eq!(global_work_stats["pr_count"], 2);

    let local_knowledge_stats = parse(
        mcp_tools::knowledge_stats(
            &state,
            serde_json::from_value::<KnowledgeStatsRequest>(json!({})).unwrap(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(local_knowledge_stats["doc_count"], 0);
    assert_eq!(
        local_knowledge_stats["omitted_layers"],
        json!(["knowledge_stats"])
    );

    let related_knowledge_stats = parse(
        mcp_tools::knowledge_stats(
            &state,
            serde_json::from_value::<KnowledgeStatsRequest>(json!({
                "scope": related_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(related_knowledge_stats["doc_count"], 0);
    assert_eq!(
        related_knowledge_stats["omitted_layers"],
        json!(["knowledge_stats"])
    );

    let local_document_stats = parse(
        mcp_tools::docs_new(
            &state,
            serde_json::from_value::<DocsRequestNew>(json!({
                "action": "stats"
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(local_document_stats["source_count"], 0);
    assert_eq!(
        local_document_stats["omitted_layers"],
        json!(["document_stats"])
    );

    let related_document_stats = parse(
        mcp_tools::docs_new(
            &state,
            serde_json::from_value::<DocsRequestNew>(json!({
                "action": "stats",
                "scope": related_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(related_document_stats["source_count"], 0);
    assert_eq!(
        related_document_stats["omitted_layers"],
        json!(["document_stats"])
    );

    let global_document_stats = parse(
        mcp_tools::docs_new(
            &state,
            serde_json::from_value::<DocsRequestNew>(json!({
                "action": "stats",
                "scope": global_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(global_document_stats["source_count"], 0);
    assert_eq!(global_document_stats["authorization_scope_enforced"], false);
    assert_eq!(global_document_stats["omitted_layers"], json!([]));

    let local_docs = parse(
        mcp_tools::docs_new(
            &state,
            serde_json::from_value::<DocsRequestNew>(json!({
                "action": "search",
                "query": "scope-canary"
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(local_docs["count"], 0);
    assert_eq!(local_docs["omitted_layers"], json!(["document"]));

    let related_docs = parse(
        mcp_tools::docs_new(
            &state,
            serde_json::from_value::<DocsRequestNew>(json!({
                "action": "search",
                "query": "scope-canary",
                "search_scope": related_scope
            }))
            .unwrap(),
        )
        .await
        .unwrap(),
    );
    assert_eq!(related_docs["count"], 0);
    assert_eq!(related_docs["resolved_project"], "scope-alpha");
    assert_eq!(related_docs["omitted_layers"], json!(["document"]));
}

#[tokio::test]
async fn related_search_exact_task_excludes_sibling_task_and_unowned_layers() {
    let (search, entities, sessions, tools, work, memory) = setup_related_search_service().await;
    let project = work
        .create_project("task-scope-project", None)
        .await
        .unwrap();
    let current_task = work
        .create_task("task-scope-project", "current-task", None, Some("SCOPE-1"))
        .await
        .unwrap();
    let sibling_task = work
        .create_task("task-scope-project", "sibling-task", None, Some("SCOPE-2"))
        .await
        .unwrap();
    work.add_project_observation("task-scope-project", "task-scope project observation", None)
        .await
        .unwrap();
    work.add_task_observation("SCOPE-1", "current task observation", None)
        .await
        .unwrap();
    work.add_task_observation("SCOPE-2", "sibling task observation", None)
        .await
        .unwrap();
    work.add_pr(
        "task-scope-project",
        Some("current-task"),
        "https://github.com/example/task-scope/pull/1",
        None,
    )
    .await
    .unwrap();
    work.add_pr(
        "task-scope-project",
        Some("sibling-task"),
        "https://github.com/example/task-scope/pull/2",
        None,
    )
    .await
    .unwrap();
    let current_entity = entities
        .create_entity(
            "current-task-service",
            EntityType::Service,
            Some("task-scope-canary current entity"),
        )
        .await
        .unwrap();
    let sibling_entity = entities
        .create_entity(
            "sibling-task-service",
            EntityType::Service,
            Some("task-scope-canary sibling entity"),
        )
        .await
        .unwrap();
    work.connect_task_to_entity(&current_task.id.to_string(), "current-task-service", None)
        .await
        .unwrap();
    work.connect_task_to_entity(&sibling_task.id.to_string(), "sibling-task-service", None)
        .await
        .unwrap();
    let current_observation = entities
        .add_observation(
            "current-task-service",
            "task-scope-canary current observation",
            None,
            None,
        )
        .await
        .unwrap()
        .0;
    let sibling_observation = entities
        .add_observation(
            "sibling-task-service",
            "task-scope-canary sibling observation",
            None,
            None,
        )
        .await
        .unwrap()
        .0;

    let project_session = sessions
        .start_session(Some("codex"), Some("task-scope-project"), None)
        .await
        .unwrap();
    let project_event = sessions
        .log_event(
            &project_session.id,
            EventType::Observation,
            "task-scope-canary session without task ownership",
            None,
            None,
        )
        .await
        .unwrap();
    entities
        .create_entity("task-scope-tool", EntityType::Tool, None)
        .await
        .unwrap();
    let project_usage = tools
        .log_usage(
            "task-scope-tool",
            "task-scope-canary usage without task ownership",
            ToolOutcome::Success,
            Some(&project_session.id),
        )
        .await
        .unwrap();

    let current_memory = memory
        .capture_memory(
            MemoryItem::new(
                MemoryKind::Decision,
                "task-scope-canary current memory",
                "current task memory",
                MemoryScope::Task {
                    project_id: Some(project.id),
                    project_name: None,
                    task_id: Some(current_task.id),
                    task_name: current_task.name.clone(),
                },
                ClaimOrigin::UserStated,
                writer(),
            )
            .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "scope-test")),
        )
        .await
        .unwrap();
    let sibling_memory = memory
        .capture_memory(
            MemoryItem::new(
                MemoryKind::Decision,
                "task-scope-canary sibling memory",
                "sibling task memory",
                MemoryScope::Task {
                    project_id: Some(project.id),
                    project_name: Some(project.name.clone()),
                    task_id: Some(sibling_task.id),
                    task_name: sibling_task.name.clone(),
                },
                ClaimOrigin::UserStated,
                writer(),
            )
            .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "scope-test")),
        )
        .await
        .unwrap();

    let related = search
        .search_related(
            "task-scope-canary",
            20,
            Some(0.0),
            None,
            &SearchOptions {
                project: Some(project.name.clone()),
                cwd: None,
            },
            Some(&current_task.id.to_string()),
        )
        .await
        .unwrap();

    assert_eq!(related.task.as_deref(), Some("current-task"));
    for expected_id in [current_entity.id, current_observation.id, current_memory.id] {
        assert!(related
            .results
            .iter()
            .any(|result| result.id == expected_id.to_string()));
    }
    for forbidden_id in [
        sibling_entity.id,
        sibling_observation.id,
        sibling_memory.id,
        project_event.id,
        project_usage.id,
    ] {
        assert!(
            related
                .results
                .iter()
                .all(|result| result.id != forbidden_id.to_string()),
            "exact-task search leaked result {forbidden_id}"
        );
    }
    assert!(related.omitted_layers.contains(&SearchLayer::Document));
    assert!(related.omitted_layers.contains(&SearchLayer::SessionEvent));
    assert!(related.omitted_layers.contains(&SearchLayer::ToolUsage));
    assert!(!related.results.iter().any(|result| matches!(
        result.source,
        SearchResultSource::SessionEvent | SearchResultSource::ToolUsage
    )));

    let state = ToolState::new();
    state.init_entity(entities).await;
    state.init_session(sessions).await;
    state.init_tool_intel(tools).await;
    state.init_work(work).await;
    state.init_search(search).await;
    let exact_task_scope = RetrievalScopeRequest {
        relevance_mode: Some("related".to_string()),
        project: Some("task-scope-project".to_string()),
        task: Some(current_task.id.to_string()),
        cwd: None,
    };

    let session_response = mcp_tools::session_new(
        &state,
        serde_json::from_value::<SessionRequestNew>(json!({
            "action": "search",
            "query": "task-scope-canary",
            "search_scope": exact_task_scope
        }))
        .unwrap(),
    )
    .await
    .unwrap();
    let session_json: Value = serde_json::from_str(&session_response).unwrap();
    assert_eq!(session_json["count"], 0);
    assert_eq!(session_json["resolved_task"], "current-task");
    assert_eq!(session_json["omitted_layers"], json!(["session_event"]));
    assert!(!session_response.contains(&project_event.id.to_string()));

    let tool_response = mcp_tools::tool_new(
        &state,
        serde_json::from_value::<ToolRequestNew>(json!({
            "action": "search",
            "query": "task-scope-canary",
            "search_scope": exact_task_scope
        }))
        .unwrap(),
    )
    .await
    .unwrap();
    let tool_json: Value = serde_json::from_str(&tool_response).unwrap();
    assert_eq!(tool_json["count"], 0);
    assert_eq!(tool_json["resolved_task"], "current-task");
    assert_eq!(tool_json["omitted_layers"], json!(["tool_usage"]));
    assert!(!tool_response.contains(&project_usage.id.to_string()));

    for request in [
        json!({
            "action": "get",
            "session_id": project_session.id.to_string(),
            "scope": exact_task_scope
        }),
        json!({
            "action": "list",
            "scope": exact_task_scope
        }),
    ] {
        let response = mcp_tools::session_new(
            &state,
            serde_json::from_value::<SessionRequestNew>(request).unwrap(),
        )
        .await
        .unwrap();
        let response_json: Value = serde_json::from_str(&response).unwrap();
        assert_eq!(response_json["resolved_task"], "current-task");
        assert_eq!(response_json["omitted_layers"], json!(["session_event"]));
        assert!(!response.contains(&project_event.id.to_string()));
    }

    for request in [
        json!({
            "action": "list",
            "scope": exact_task_scope
        }),
        json!({
            "action": "recommend",
            "context": "task-scope-canary",
            "scope": exact_task_scope
        }),
        json!({
            "action": "stats",
            "tool_name": "task-scope-tool",
            "scope": exact_task_scope
        }),
    ] {
        let response = mcp_tools::tool_new(
            &state,
            serde_json::from_value::<ToolRequestNew>(request).unwrap(),
        )
        .await
        .unwrap();
        let response_json: Value = serde_json::from_str(&response).unwrap();
        assert_eq!(response_json["resolved_task"], "current-task");
        assert_eq!(response_json["omitted_layers"], json!(["tool_usage"]));
        assert!(!response.contains(&project_usage.id.to_string()));
        assert_eq!(response_json["count"].as_u64().unwrap_or(0), 0);
        assert_eq!(response_json["total_usages"].as_u64().unwrap_or(0), 0);
    }

    let exact_entity_stats = mcp_tools::entity_stats(
        &state,
        serde_json::from_value::<EntityStatsRequest>(json!({
            "scope": exact_task_scope
        }))
        .unwrap(),
    )
    .await
    .unwrap();
    let exact_entity_stats_json: Value = serde_json::from_str(&exact_entity_stats).unwrap();
    assert_eq!(exact_entity_stats_json["entity_count"], 1);
    assert_eq!(exact_entity_stats_json["observation_count"], 1);
    assert_eq!(exact_entity_stats_json["resolved_task"], "current-task");

    let exact_session_stats = mcp_tools::session_stats(
        &state,
        serde_json::from_value::<SessionStatsRequest>(json!({
            "scope": exact_task_scope
        }))
        .unwrap(),
    )
    .await
    .unwrap();
    let exact_session_stats_json: Value = serde_json::from_str(&exact_session_stats).unwrap();
    assert_eq!(exact_session_stats_json["total_sessions"], 0);
    assert_eq!(
        exact_session_stats_json["omitted_layers"],
        json!(["session_stats"])
    );

    let exact_tool_stats = mcp_tools::tool_intel_stats(
        &state,
        serde_json::from_value::<ToolIntelStatsRequest>(json!({
            "scope": exact_task_scope
        }))
        .unwrap(),
    )
    .await
    .unwrap();
    let exact_tool_stats_json: Value = serde_json::from_str(&exact_tool_stats).unwrap();
    assert_eq!(exact_tool_stats_json["usage_count"], 0);
    assert_eq!(
        exact_tool_stats_json["omitted_layers"],
        json!(["tool_intel_stats"])
    );

    let exact_work_stats = mcp_tools::work_stats(
        &state,
        serde_json::from_value::<WorkStatsRequest>(json!({
            "scope": exact_task_scope
        }))
        .unwrap(),
    )
    .await
    .unwrap();
    let exact_work_stats_json: Value = serde_json::from_str(&exact_work_stats).unwrap();
    assert_eq!(exact_work_stats_json["project_count"], 1);
    assert_eq!(exact_work_stats_json["task_count"], 1);
    assert_eq!(exact_work_stats_json["pr_count"], 1);
    assert_eq!(exact_work_stats_json["project_observation_count"], 0);
    assert_eq!(exact_work_stats_json["task_observation_count"], 1);
}

#[tokio::test]
async fn scoped_search_fails_closed_without_a_boundary_and_local_defaults_to_global_user_memory() {
    let (search, _, _, _, _, memory) = setup_related_search_service().await;
    let project_memory = memory
        .capture_memory(
            MemoryItem::new(
                MemoryKind::Decision,
                "unscoped-canary project memory",
                "must not be returned by unscoped local search",
                MemoryScope::project("another-project"),
                ClaimOrigin::UserStated,
                writer(),
            )
            .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "scope-test")),
        )
        .await
        .unwrap();
    let global_memory = memory
        .capture_memory(
            MemoryItem::new(
                MemoryKind::Preference,
                "unscoped-canary global memory",
                "eligible without a project boundary",
                MemoryScope::Global,
                ClaimOrigin::UserStated,
                writer(),
            )
            .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "scope-test")),
        )
        .await
        .unwrap();

    let local = search
        .search_local_memory(
            "unscoped-canary",
            20,
            Some(0.0),
            &SearchOptions::default(),
            None,
        )
        .await
        .unwrap();
    assert!(local
        .iter()
        .any(|result| result.id == global_memory.id.to_string()));
    assert!(local
        .iter()
        .all(|result| result.id != project_memory.id.to_string()));

    let error = search
        .search_related(
            "unscoped-canary",
            20,
            Some(0.0),
            None,
            &SearchOptions::default(),
            None,
        )
        .await
        .unwrap_err();
    assert!(error.to_string().contains("related search requires"));
}
