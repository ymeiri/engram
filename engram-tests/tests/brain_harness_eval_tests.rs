//! Brain harness eval spike.
//!
//! These tests keep the proposed brain-harness direction executable while the
//! RFC is still forming. They intentionally cover a small slice: trace capture,
//! orientation behavior that works today, and one ignored target-state gap.

use engram_core::memory::{
    ClaimOrigin, Harness, MemoryItem, MemoryKind, MemoryScope, ModelIdentity, WriterProvenance,
};
use engram_core::telemetry::{AgentFeedback, BrainHarnessIntent, BrainHarnessOperation};
use engram_index::ToolIntelService;
use engram_index::{
    EntityService, MemoryService, OrientInput, SearchService, SessionService, TelemetryService,
};
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
    latency_ms: Option<u64>,
    degraded: bool,
    returned_item_ids: Vec<String>,
    used_item_ids: Vec<String>,
    missing_expected_item_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EvalOutcome {
    success: bool,
    preference_adhered: bool,
    repeated_context_questions: u32,
    conflict_resolution_correct: Option<bool>,
    bad_memory_used: bool,
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

async fn setup_memory_and_telemetry_services() -> (MemoryService, TelemetryService) {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config)
        .await
        .expect("failed to connect to in-memory store");

    let telemetry_service = TelemetryService::new(db.clone());
    telemetry_service
        .init_schema()
        .await
        .expect("failed to initialize telemetry schema");

    let memory_service = MemoryService::new(db);
    memory_service
        .init_schema()
        .await
        .expect("failed to initialize memory schema");

    (memory_service, telemetry_service)
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
            latency_ms: Some(91),
            degraded: false,
            returned_item_ids: vec!["mem-pref-concise-status".to_string()],
            used_item_ids: vec!["mem-pref-concise-status".to_string()],
            missing_expected_item_ids: Vec::new(),
        }],
        retrieved_item_ids: vec!["mem-pref-concise-status".to_string()],
        used_item_ids: vec!["mem-pref-concise-status".to_string()],
        outcome: EvalOutcome {
            success: true,
            preference_adhered: true,
            repeated_context_questions: 0,
            conflict_resolution_correct: None,
            bad_memory_used: false,
            quality_score: 0.92,
        },
    };

    let encoded = serde_json::to_value(&trace).expect("trace should serialize");
    assert_eq!(encoded["arm"], "memory_items");
    assert_eq!(encoded["memory_calls"][0]["tool"], "orient");
    assert_eq!(encoded["memory_calls"][0]["degraded"], false);

    let restored: BrainHarnessEvalTrace =
        serde_json::from_value(encoded).expect("trace should deserialize");
    assert_eq!(restored.arm, EvalArm::MemoryItems);
    assert_eq!(restored.useful_memory_count(), 1);
    assert!((restored.retrieval_precision() - 1.0).abs() < f32::EPSILON);
    assert!(restored.outcome.success);
    assert!(restored.outcome.preference_adhered);
    assert_eq!(restored.outcome.repeated_context_questions, 0);
    assert!(!restored.outcome.bad_memory_used);
    assert!(restored.outcome.quality_score >= 0.8);
}

#[tokio::test]
async fn memoryitem_eval_trace_records_orient_feedback_and_intent_stats() {
    let (memory_service, telemetry_service) = setup_memory_and_telemetry_services().await;
    let commit_preference = memory_service
        .capture_memory(MemoryItem::new(
            MemoryKind::Preference,
            "Commit every completed step",
            "The user wants each successful implementation checkpoint committed before continuing.",
            MemoryScope::project("engram"),
            ClaimOrigin::UserStated,
            writer(),
        ))
        .await
        .expect("preference should be captured");
    let unrelated_guidance = memory_service
        .capture_memory(MemoryItem::new(
            MemoryKind::Decision,
            "Keep snippet-only install mode",
            "Claude harness installation should support a no-settings-write snippet-only mode.",
            MemoryScope::project("engram"),
            ClaimOrigin::AgentInferred,
            writer(),
        ))
        .await
        .expect("distractor guidance should be captured");

    let prompt =
        "Continue Engram implementation and honor the user's commit-every-step preference.";
    let packet = memory_service
        .orient(OrientInput {
            project: Some("engram".to_string()),
            cwd: Some("/Users/yuval.meiri/projects/engram".to_string()),
            prompt: Some(prompt.to_string()),
            agent: Some("codex".to_string()),
            intent: Some(BrainHarnessIntent::FollowUserPreference),
            include_recent_commits: false,
            limit: Some(10),
        })
        .await
        .expect("orient should return a packet");

    assert!(packet
        .preferences
        .iter()
        .any(|item| item.id == commit_preference.id));
    let trace_id = packet.trace_id.expect("orient should return trace_id");
    let stored_trace = telemetry_service
        .get_trace(&trace_id)
        .await
        .expect("trace lookup should run")
        .expect("trace should exist");
    assert_eq!(stored_trace.operation, BrainHarnessOperation::Orient);
    assert_eq!(
        stored_trace.intent,
        Some(BrainHarnessIntent::FollowUserPreference)
    );
    assert!(stored_trace
        .returned_memory_ids
        .contains(&commit_preference.id));
    assert!(stored_trace
        .returned_memory_ids
        .contains(&unrelated_guidance.id));

    let returned_item_ids = stored_trace
        .returned_memory_ids
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let eval_trace = BrainHarnessEvalTrace {
        trace_id: trace_id.to_string(),
        scenario: "preference_applied_later".to_string(),
        arm: EvalArm::MemoryItems,
        prompt: prompt.to_string(),
        memory_calls: vec![MemoryCallTrace {
            tool: stored_trace.operation.to_string(),
            query: stored_trace.query.clone().unwrap_or_default(),
            latency_ms: stored_trace.latency_ms,
            degraded: !stored_trace.warnings.is_empty(),
            returned_item_ids: returned_item_ids.clone(),
            used_item_ids: vec![commit_preference.id.to_string()],
            missing_expected_item_ids: Vec::new(),
        }],
        retrieved_item_ids: returned_item_ids,
        used_item_ids: vec![commit_preference.id.to_string()],
        outcome: EvalOutcome {
            success: true,
            preference_adhered: true,
            repeated_context_questions: 0,
            conflict_resolution_correct: None,
            bad_memory_used: false,
            quality_score: 0.9,
        },
    };
    assert_eq!(eval_trace.useful_memory_count(), 1);
    assert!(eval_trace.retrieval_precision() > 0.0);
    assert!(eval_trace.outcome.preference_adhered);

    let mut feedback = AgentFeedback::new(trace_id);
    feedback.agent = Some("codex".to_string());
    feedback.used_memory_ids = vec![commit_preference.id];
    feedback.rejected_memory_ids = vec![unrelated_guidance.id];
    feedback.usefulness_score = Some(5);
    feedback.correctness_score = Some(5);
    feedback.noise_score = Some(2);
    feedback.note =
        Some("The preference was surfaced and applied without asking the user again.".to_string());

    telemetry_service
        .submit_feedback(feedback)
        .await
        .expect("agent feedback should be recorded");
    let stats = telemetry_service
        .stats_by_intent()
        .await
        .expect("intent stats should aggregate");
    let preference_stats = stats
        .iter()
        .find(|item| item.intent == "follow_user_preference")
        .expect("follow_user_preference stats should exist");
    assert_eq!(preference_stats.trace_count, 1);
    assert_eq!(preference_stats.feedback_count, 1);
    assert_eq!(preference_stats.used_memory_count, 1);
    assert_eq!(preference_stats.rejected_memory_count, 1);
    assert_eq!(preference_stats.avg_usefulness_score, Some(5.0));
    assert_eq!(preference_stats.avg_correctness_score, Some(5.0));
    assert_eq!(preference_stats.avg_noise_score, Some(2.0));
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
