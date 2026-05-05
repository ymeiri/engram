//! Brain harness eval spike.
//!
//! These tests keep the proposed brain-harness direction executable while the
//! RFC is still forming. They intentionally cover a small slice: trace capture,
//! orientation behavior that works today, and one ignored target-state gap.

use engram_core::memory::{
    ClaimOrigin, Harness, MemoryItem, MemoryKind, MemoryScope, ModelIdentity, WriterProvenance,
};
use engram_index::ToolIntelService;
use engram_index::{EntityService, MemoryService, OrientInput, SearchService, SessionService};
use engram_store::{connect_and_init, StoreConfig};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum EvalArm {
    NoMemory,
    LegacyObservations,
    MemoryItems,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MemoryCallTrace {
    tool: String,
    query: String,
    returned_item_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EvalOutcome {
    success: bool,
    repeated_context_questions: u32,
    quality_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BrainHarnessEvalTrace {
    trace_id: String,
    scenario: String,
    arm: EvalArm,
    prompt: String,
    memory_calls: Vec<MemoryCallTrace>,
    retrieved_item_ids: Vec<String>,
    used_item_ids: Vec<String>,
    outcome: EvalOutcome,
}

impl BrainHarnessEvalTrace {
    fn useful_memory_count(&self) -> usize {
        self.used_item_ids
            .iter()
            .filter(|used| self.retrieved_item_ids.contains(used))
            .count()
    }

    fn retrieval_precision(&self) -> f32 {
        if self.retrieved_item_ids.is_empty() {
            return 0.0;
        }

        self.useful_memory_count() as f32 / self.retrieved_item_ids.len() as f32
    }
}

async fn setup_memory_service() -> MemoryService {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config)
        .await
        .expect("failed to connect to in-memory store");
    let service = MemoryService::new(db);
    service
        .init_schema()
        .await
        .expect("failed to initialize memory schema");
    service
}

async fn setup_memory_and_search_services() -> (MemoryService, SearchService) {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config)
        .await
        .expect("failed to connect to in-memory store");

    let entity_service = EntityService::new(db.clone());
    entity_service
        .init()
        .await
        .expect("failed to initialize entity service");
    let session_service = SessionService::new(db.clone());
    session_service
        .init()
        .await
        .expect("failed to initialize session service");
    let tool_service = ToolIntelService::new(db.clone());
    tool_service
        .init()
        .await
        .expect("failed to initialize tool service");

    let memory_service = MemoryService::new(db.clone());
    memory_service
        .init_schema()
        .await
        .expect("failed to initialize memory schema");

    (memory_service, SearchService::new(db))
}

fn writer() -> WriterProvenance {
    WriterProvenance::agent(Harness::Codex, ModelIdentity::new("openai", "gpt-5.5"))
        .with_surface("brain-harness-eval")
}

#[test]
fn brain_harness_trace_schema_links_retrieval_to_outcome() {
    let trace = BrainHarnessEvalTrace {
        trace_id: "trace-prefers-concise-status-memory-items".to_string(),
        scenario: "multi-session preference continuity".to_string(),
        arm: EvalArm::MemoryItems,
        prompt: "Continue the implementation and report status.".to_string(),
        memory_calls: vec![MemoryCallTrace {
            tool: "orient".to_string(),
            query: "project=engram prompt=status update".to_string(),
            returned_item_ids: vec!["mem-pref-concise-status".to_string()],
        }],
        retrieved_item_ids: vec!["mem-pref-concise-status".to_string()],
        used_item_ids: vec!["mem-pref-concise-status".to_string()],
        outcome: EvalOutcome {
            success: true,
            repeated_context_questions: 0,
            quality_score: 0.92,
        },
    };

    let encoded = serde_json::to_value(&trace).expect("trace should serialize");
    assert_eq!(encoded["arm"], "memory_items");
    assert_eq!(encoded["memory_calls"][0]["tool"], "orient");

    let restored: BrainHarnessEvalTrace =
        serde_json::from_value(encoded).expect("trace should deserialize");
    assert_eq!(restored.arm, EvalArm::MemoryItems);
    assert_eq!(restored.useful_memory_count(), 1);
    assert!((restored.retrieval_precision() - 1.0).abs() < f32::EPSILON);
    assert!(restored.outcome.success);
    assert_eq!(restored.outcome.repeated_context_questions, 0);
    assert!(restored.outcome.quality_score >= 0.8);
}

#[tokio::test]
async fn orient_surfaces_user_preference_as_raw_and_compiled_context() {
    let service = setup_memory_service().await;
    let preference = MemoryItem::new(
        MemoryKind::Preference,
        "Use concise status updates",
        "User prefers concise implementation updates with validation notes.",
        MemoryScope::project("engram"),
        ClaimOrigin::UserStated,
        writer(),
    );
    let captured = service
        .capture_memory(preference)
        .await
        .expect("preference should be captured");

    let packet = service
        .orient(OrientInput {
            project: Some("engram".to_string()),
            cwd: Some("/Users/yuval.meiri/projects/engram".to_string()),
            prompt: Some("Continue the implementation and report status.".to_string()),
            agent: Some("codex".to_string()),
            intent: None,
            include_recent_commits: false,
            limit: Some(10),
        })
        .await
        .expect("orient should return a packet");

    assert!(packet.preferences.iter().any(|item| item.id == captured.id));
    assert!(packet.context_pack.contains("## Preferences"));
    assert!(packet.context_pack.contains("Use concise status updates"));
    assert!(packet
        .context_pack
        .contains("User prefers concise implementation updates"));
    assert!(packet.review_needed.is_empty());
}

#[tokio::test]
async fn unified_search_should_find_memoryitem_guidance() {
    let (memory_service, search_service) = setup_memory_and_search_services().await;
    let guidance = MemoryItem::new(
        MemoryKind::Decision,
        "MemoryItem is the canonical cognitive unit",
        "Brain-harness retrieval should make MemoryItems first-class search results.",
        MemoryScope::project("engram"),
        ClaimOrigin::UserStated,
        writer(),
    );
    let captured = memory_service
        .capture_memory(guidance)
        .await
        .expect("guidance should be captured");

    let results = search_service
        .search("canonical cognitive unit", 10, Some(0.0), None)
        .await
        .expect("search should run");

    assert!(
        results
            .iter()
            .any(|result| result.id == captured.id.to_string()),
        "unified search should return matching MemoryItems"
    );
    assert!(results
        .iter()
        .any(|result| result.source.to_string() == "memory"));
}
