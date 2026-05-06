//! Brain harness eval spike.
//!
//! These tests keep the proposed brain-harness direction executable while the
//! RFC is still forming. They intentionally cover a small slice: trace capture,
//! orientation behavior, shared retrieval ranking, and deterministic confidence
//! scenarios.

use engram_core::memory::{
    ClaimOrigin, EvidenceKind, EvidenceRef, Harness, MemoryFreshness, MemoryItem, MemoryKind,
    MemoryReviewState, MemoryScope, ModelIdentity, WriterProvenance,
};
use engram_core::search::SearchLayer;
use engram_core::telemetry::{
    AgentFeedback, BrainHarnessIntent, BrainHarnessOperation, BrainHarnessTrace,
};
use engram_core::{entity::EntityType, session::EventType};
use engram_index::ToolIntelService;
use engram_index::{
    EntityService, MemoryService, OrientInput, SearchOptions, SearchService, SessionService,
    TelemetryService, WorkService,
};
use engram_store::{connect_and_init, StoreConfig};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum EvalArm {
    NoMemory,
    LegacyObservations,
    MemoryItems,
    Hybrid,
}

impl EvalArm {
    const fn as_str(self) -> &'static str {
        match self {
            Self::NoMemory => "no_memory",
            Self::LegacyObservations => "legacy_observations",
            Self::MemoryItems => "memory_items",
            Self::Hybrid => "hybrid",
        }
    }
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

#[derive(Debug, Clone)]
struct EvalArmSummary {
    arm: EvalArm,
    trace_count: usize,
    success_count: usize,
    bad_memory_count: usize,
    missing_expected_count: usize,
    repeated_context_questions: u32,
    avg_quality_score: f32,
    avg_retrieval_precision: f32,
}

#[derive(Debug, Clone)]
struct BrainHarnessEvalReport {
    traces: Vec<BrainHarnessEvalTrace>,
}

impl BrainHarnessEvalReport {
    fn new(traces: Vec<BrainHarnessEvalTrace>) -> Self {
        Self { traces }
    }

    fn scenarios(&self) -> Vec<&str> {
        let mut scenarios = Vec::new();
        for trace in &self.traces {
            let scenario = trace.scenario.as_str();
            if !scenarios.contains(&scenario) {
                scenarios.push(scenario);
            }
        }
        scenarios
    }

    fn trace(&self, scenario: &str, arm: EvalArm) -> Option<&BrainHarnessEvalTrace> {
        self.traces
            .iter()
            .find(|trace| trace.scenario == scenario && trace.arm == arm)
    }

    fn arm_summary(&self, arm: EvalArm) -> EvalArmSummary {
        let traces = self
            .traces
            .iter()
            .filter(|trace| trace.arm == arm)
            .collect::<Vec<_>>();
        let trace_count = traces.len();
        let quality_sum = traces
            .iter()
            .map(|trace| trace.outcome.quality_score)
            .sum::<f32>();
        let precision_sum = traces
            .iter()
            .map(|trace| trace.retrieval_precision())
            .sum::<f32>();

        EvalArmSummary {
            arm,
            trace_count,
            success_count: traces.iter().filter(|trace| trace.outcome.success).count(),
            bad_memory_count: traces
                .iter()
                .filter(|trace| trace.outcome.bad_memory_used)
                .count(),
            missing_expected_count: traces
                .iter()
                .flat_map(|trace| &trace.memory_calls)
                .map(|call| call.missing_expected_item_ids.len())
                .sum(),
            repeated_context_questions: traces
                .iter()
                .map(|trace| trace.outcome.repeated_context_questions)
                .sum(),
            avg_quality_score: if trace_count == 0 {
                0.0
            } else {
                quality_sum / trace_count as f32
            },
            avg_retrieval_precision: if trace_count == 0 {
                0.0
            } else {
                precision_sum / trace_count as f32
            },
        }
    }

    fn memory_items_outperform_legacy(&self) -> bool {
        self.scenarios().iter().all(|scenario| {
            let Some(memory_items) = self.trace(scenario, EvalArm::MemoryItems) else {
                return false;
            };
            let Some(legacy) = self.trace(scenario, EvalArm::LegacyObservations) else {
                return false;
            };

            memory_items.outcome.success
                && memory_items.outcome.quality_score > legacy.outcome.quality_score
                && !memory_items.outcome.bad_memory_used
        })
    }

    fn hybrid_preserves_legacy_and_memory_evidence(&self) -> bool {
        self.scenarios().iter().all(|scenario| {
            let Some(hybrid) = self.trace(scenario, EvalArm::Hybrid) else {
                return false;
            };
            let Some(legacy) = self.trace(scenario, EvalArm::LegacyObservations) else {
                return false;
            };
            let Some(memory_items) = self.trace(scenario, EvalArm::MemoryItems) else {
                return false;
            };

            hybrid
                .retrieved_item_ids
                .iter()
                .any(|id| legacy.retrieved_item_ids.contains(id))
                && hybrid
                    .retrieved_item_ids
                    .iter()
                    .any(|id| memory_items.retrieved_item_ids.contains(id))
        })
    }

    fn no_memory_records_missing_expected_context(&self) -> bool {
        self.traces
            .iter()
            .filter(|trace| trace.arm == EvalArm::NoMemory)
            .all(|trace| {
                trace
                    .memory_calls
                    .iter()
                    .any(|call| !call.missing_expected_item_ids.is_empty())
            })
    }
}

#[derive(Debug, Clone)]
struct ConfidenceScenarioComparison {
    scenario: String,
    no_memory: BrainHarnessEvalTrace,
    memory_items: BrainHarnessEvalTrace,
}

impl ConfidenceScenarioComparison {
    fn memory_items_improved(&self) -> bool {
        self.no_memory.scenario == self.scenario
            && self.memory_items.scenario == self.scenario
            && self.memory_items.outcome.success
            && self.memory_items.outcome.quality_score > self.no_memory.outcome.quality_score
            && self.memory_items.outcome.repeated_context_questions
                <= self.no_memory.outcome.repeated_context_questions
            && !self.memory_items.outcome.bad_memory_used
    }
}

#[derive(Debug, Clone)]
struct FullScenarioComparison {
    scenario: String,
    no_memory: BrainHarnessEvalTrace,
    legacy: BrainHarnessEvalTrace,
    memory_items: BrainHarnessEvalTrace,
    hybrid: BrainHarnessEvalTrace,
}

impl FullScenarioComparison {
    fn memory_items_outperform_legacy(&self) -> bool {
        self.no_memory.scenario == self.scenario
            && self.legacy.scenario == self.scenario
            && self.memory_items.scenario == self.scenario
            && self.memory_items.outcome.success
            && self.memory_items.outcome.quality_score > self.legacy.outcome.quality_score
            && !self.memory_items.outcome.bad_memory_used
    }

    fn hybrid_preserves_legacy_evidence(&self) -> bool {
        self.hybrid.scenario == self.scenario
            && self.hybrid.outcome.success
            && self
                .hybrid
                .retrieved_item_ids
                .iter()
                .any(|id| self.legacy.retrieved_item_ids.contains(id))
            && self
                .hybrid
                .retrieved_item_ids
                .iter()
                .any(|id| self.memory_items.retrieved_item_ids.contains(id))
    }
}

#[derive(Clone)]
struct BrainHarnessEvalServices {
    memory: MemoryService,
    telemetry: TelemetryService,
    entity: EntityService,
    session: SessionService,
    work: WorkService,
    search: SearchService,
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

async fn setup_brain_harness_eval_services() -> BrainHarnessEvalServices {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config)
        .await
        .expect("failed to connect to in-memory store");

    let entity = EntityService::new(db.clone());
    entity
        .init()
        .await
        .expect("failed to initialize entity service");
    let session = SessionService::new(db.clone());
    session
        .init()
        .await
        .expect("failed to initialize session service");
    let work = WorkService::new(db.clone());
    work.init()
        .await
        .expect("failed to initialize work service");
    let memory = MemoryService::new(db.clone());
    memory
        .init_schema()
        .await
        .expect("failed to initialize memory schema");
    let telemetry = TelemetryService::new(db.clone());
    telemetry
        .init_schema()
        .await
        .expect("failed to initialize telemetry schema");

    BrainHarnessEvalServices {
        memory,
        telemetry,
        entity,
        session,
        work,
        search: SearchService::new(db),
    }
}

fn writer() -> WriterProvenance {
    WriterProvenance::agent(Harness::Codex, ModelIdentity::new("openai", "gpt-5.5"))
        .with_surface("brain-harness-eval")
}

fn reviewed_evidence(summary: &str) -> EvidenceRef {
    EvidenceRef::new(EvidenceKind::ManualReview, "brain-harness-eval")
        .with_summary(summary)
        .with_excerpt("accepted by deterministic eval fixture")
}

fn trace_returned_item_ids(trace: &BrainHarnessTrace) -> Vec<String> {
    let mut ids = trace
        .returned_memory_ids
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    for id in &trace.returned_result_ids {
        if !ids.contains(id) {
            ids.push(id.clone());
        }
    }
    ids
}

fn eval_trace_from_operation(
    scenario: &str,
    arm: EvalArm,
    prompt: &str,
    trace: &BrainHarnessTrace,
    used_item_ids: Vec<String>,
    missing_expected_item_ids: Vec<String>,
    outcome: EvalOutcome,
) -> BrainHarnessEvalTrace {
    let returned_item_ids = trace_returned_item_ids(trace);

    BrainHarnessEvalTrace {
        trace_id: trace.id.to_string(),
        scenario: scenario.to_string(),
        arm,
        prompt: prompt.to_string(),
        memory_calls: vec![MemoryCallTrace {
            tool: trace.operation.to_string(),
            query: trace.query.clone().unwrap_or_default(),
            latency_ms: trace.latency_ms,
            degraded: !trace.warnings.is_empty(),
            returned_item_ids: returned_item_ids.clone(),
            used_item_ids: used_item_ids.clone(),
            missing_expected_item_ids,
        }],
        retrieved_item_ids: returned_item_ids,
        used_item_ids,
        outcome,
    }
}

fn apply_outcome_feedback(feedback: &mut AgentFeedback, outcome: &EvalOutcome) {
    feedback.task_success = Some(outcome.success);
    feedback.preference_adhered = Some(outcome.preference_adhered);
    feedback.repeated_context_questions = Some(outcome.repeated_context_questions);
    feedback.bad_memory_used = Some(outcome.bad_memory_used);
}

async fn record_no_memory_baseline(
    telemetry_service: &TelemetryService,
    scenario: &str,
    prompt: &str,
    intent: BrainHarnessIntent,
    expected_item_ids: Vec<String>,
    missing_context: &str,
    outcome: EvalOutcome,
) -> BrainHarnessEvalTrace {
    let trace = telemetry_service
        .record_trace(
            BrainHarnessTrace::new(BrainHarnessOperation::Custom(
                "no_memory_baseline".to_string(),
            ))
            .with_agent(Some("codex".to_string()))
            .with_intent(Some(intent))
            .with_scenario_id(Some(scenario.to_string()))
            .with_arm(Some(EvalArm::NoMemory.as_str().to_string()))
            .with_query(Some(prompt.to_string()))
            .with_project(Some("engram".to_string()))
            .with_latency_ms(0),
        )
        .await
        .expect("no-memory baseline trace should be recorded");

    let mut feedback = AgentFeedback::new(trace.id);
    feedback.agent = Some("codex".to_string());
    feedback.missing_context = Some(missing_context.to_string());
    feedback.usefulness_score = Some(1);
    feedback.correctness_score = Some(2);
    feedback.noise_score = Some(1);
    apply_outcome_feedback(&mut feedback, &outcome);
    telemetry_service
        .submit_feedback(feedback)
        .await
        .expect("no-memory baseline feedback should be recorded");

    eval_trace_from_operation(
        scenario,
        EvalArm::NoMemory,
        prompt,
        &trace,
        Vec::new(),
        expected_item_ids,
        outcome,
    )
}

async fn record_legacy_trace(
    services: &BrainHarnessEvalServices,
    scenario: &str,
    prompt: &str,
    intent: BrainHarnessIntent,
    returned_result_ids: Vec<String>,
    used_result_ids: Vec<String>,
    outcome: EvalOutcome,
) -> BrainHarnessEvalTrace {
    let trace = services
        .telemetry
        .record_trace(
            BrainHarnessTrace::new(BrainHarnessOperation::Custom(
                "legacy_retrieval".to_string(),
            ))
            .with_agent(Some("codex".to_string()))
            .with_intent(Some(intent))
            .with_scenario_id(Some(scenario.to_string()))
            .with_arm(Some(EvalArm::LegacyObservations.as_str().to_string()))
            .with_query(Some(prompt.to_string()))
            .with_project(Some("engram".to_string()))
            .with_returned_result_ids(returned_result_ids)
            .with_latency_ms(0),
        )
        .await
        .expect("legacy trace should be recorded");

    let mut feedback = AgentFeedback::new(trace.id);
    feedback.agent = Some("codex".to_string());
    feedback.used_result_ids = used_result_ids.clone();
    feedback.usefulness_score = Some(if outcome.success { 3 } else { 2 });
    feedback.correctness_score = Some(if outcome.bad_memory_used { 2 } else { 4 });
    feedback.noise_score = Some(if outcome.bad_memory_used { 4 } else { 3 });
    apply_outcome_feedback(&mut feedback, &outcome);
    feedback.note =
        Some("Legacy retrieval returned usable context without trust metadata.".to_string());
    services
        .telemetry
        .submit_feedback(feedback)
        .await
        .expect("legacy feedback should be recorded");

    eval_trace_from_operation(
        scenario,
        EvalArm::LegacyObservations,
        prompt,
        &trace,
        used_result_ids,
        Vec::new(),
        outcome,
    )
}

async fn record_hybrid_trace(
    services: &BrainHarnessEvalServices,
    scenario: &str,
    prompt: &str,
    intent: BrainHarnessIntent,
    returned_memory_ids: Vec<engram_core::id::Id>,
    returned_result_ids: Vec<String>,
    used_item_ids: Vec<String>,
    outcome: EvalOutcome,
) -> BrainHarnessEvalTrace {
    let trace = services
        .telemetry
        .record_trace(
            BrainHarnessTrace::new(BrainHarnessOperation::Custom(
                "hybrid_retrieval".to_string(),
            ))
            .with_agent(Some("codex".to_string()))
            .with_intent(Some(intent))
            .with_scenario_id(Some(scenario.to_string()))
            .with_arm(Some(EvalArm::Hybrid.as_str().to_string()))
            .with_query(Some(prompt.to_string()))
            .with_project(Some("engram".to_string()))
            .with_returned_memory_ids(returned_memory_ids)
            .with_returned_result_ids(returned_result_ids)
            .with_latency_ms(0),
        )
        .await
        .expect("hybrid trace should be recorded");

    let mut feedback = AgentFeedback::new(trace.id);
    feedback.agent = Some("codex".to_string());
    feedback.used_memory_ids = trace
        .returned_memory_ids
        .iter()
        .copied()
        .filter(|id| used_item_ids.contains(&id.to_string()))
        .collect();
    feedback.used_result_ids = trace
        .returned_result_ids
        .iter()
        .filter(|id| used_item_ids.contains(id))
        .cloned()
        .collect();
    feedback.usefulness_score = Some(5);
    feedback.correctness_score = Some(5);
    feedback.noise_score = Some(1);
    apply_outcome_feedback(&mut feedback, &outcome);
    feedback.note =
        Some("Hybrid retrieval used MemoryItems while preserving legacy evidence ids.".to_string());
    services
        .telemetry
        .submit_feedback(feedback)
        .await
        .expect("hybrid feedback should be recorded");

    eval_trace_from_operation(
        scenario,
        EvalArm::Hybrid,
        prompt,
        &trace,
        used_item_ids,
        Vec::new(),
        outcome,
    )
}

async fn memoryitem_trace_from_orient(
    services: &BrainHarnessEvalServices,
    scenario: &str,
    prompt: &str,
    packet_trace_id: engram_core::id::Id,
    used_memory_ids: Vec<engram_core::id::Id>,
    rejected_memory_ids: Vec<engram_core::id::Id>,
    outcome: EvalOutcome,
) -> BrainHarnessEvalTrace {
    let trace = services
        .telemetry
        .get_trace(&packet_trace_id)
        .await
        .expect("trace lookup should run")
        .expect("memory trace should exist");
    let used_item_ids = used_memory_ids
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    let mut feedback = AgentFeedback::new(packet_trace_id);
    feedback.agent = Some("codex".to_string());
    feedback.used_memory_ids = used_memory_ids;
    feedback.rejected_memory_ids = rejected_memory_ids;
    feedback.usefulness_score = Some(5);
    feedback.correctness_score = Some(5);
    feedback.noise_score = Some(1);
    apply_outcome_feedback(&mut feedback, &outcome);
    feedback.note = Some("MemoryItem retrieval supplied scoped trust-bearing context.".to_string());
    services
        .telemetry
        .submit_feedback(feedback)
        .await
        .expect("memory feedback should be recorded");

    eval_trace_from_operation(
        scenario,
        EvalArm::MemoryItems,
        prompt,
        &trace,
        used_item_ids,
        Vec::new(),
        outcome,
    )
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
            scenario_id: Some("commit_preference_trace".to_string()),
            arm: Some(EvalArm::MemoryItems.as_str().to_string()),
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
    assert_eq!(
        stored_trace.scenario_id.as_deref(),
        Some("commit_preference_trace")
    );
    assert_eq!(stored_trace.arm.as_deref(), Some("memory_items"));
    assert!(stored_trace
        .returned_memory_ids
        .contains(&commit_preference.id));
    assert!(stored_trace
        .returned_memory_ids
        .contains(&unrelated_guidance.id));
    let preference_metadata = packet
        .memory_metadata
        .iter()
        .find(|metadata| metadata.memory_id == commit_preference.id)
        .expect("preference trust metadata should be returned");
    assert_eq!(
        preference_metadata.review_state,
        MemoryReviewState::ActiveUnreviewed
    );
    assert_eq!(preference_metadata.freshness, MemoryFreshness::Unscheduled);
    assert_eq!(preference_metadata.claim_origin, ClaimOrigin::UserStated);
    assert_eq!(preference_metadata.writer.harness, Harness::Codex);

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
async fn confidence_scenario_memoryitems_improve_preference_continuity_over_no_memory() {
    let (memory_service, telemetry_service) = setup_memory_and_telemetry_services().await;
    let preference = memory_service
        .capture_memory(
            MemoryItem::new(
                MemoryKind::Preference,
                "Commit every completed step",
                "The user wants each successful implementation checkpoint committed before continuing.",
                MemoryScope::project("engram"),
                ClaimOrigin::UserStated,
                writer(),
            )
            .with_evidence(reviewed_evidence("User explicitly confirmed commit-every-step policy.")),
        )
        .await
        .expect("preference should be captured");
    let distractor = memory_service
        .capture_memory(MemoryItem::new(
            MemoryKind::Decision,
            "Keep Claude snippet-only install mode",
            "Claude harness installation should support a no-settings-write snippet-only mode.",
            MemoryScope::project("engram"),
            ClaimOrigin::AgentObserved,
            writer(),
        ))
        .await
        .expect("distractor should be captured");

    let scenario = "preference_continuity";
    let prompt = "Continue Engram implementation and finish a checkpoint.";
    let no_memory = record_no_memory_baseline(
        &telemetry_service,
        scenario,
        prompt,
        BrainHarnessIntent::FollowUserPreference,
        vec![preference.id.to_string()],
        "Expected the user preference requiring commits after completed checkpoints.",
        EvalOutcome {
            success: false,
            preference_adhered: false,
            repeated_context_questions: 1,
            conflict_resolution_correct: None,
            bad_memory_used: false,
            quality_score: 0.35,
        },
    )
    .await;

    let packet = memory_service
        .orient(OrientInput {
            project: Some("engram".to_string()),
            cwd: Some("/Users/yuval.meiri/projects/engram".to_string()),
            prompt: Some(prompt.to_string()),
            agent: Some("codex".to_string()),
            intent: Some(BrainHarnessIntent::FollowUserPreference),
            scenario_id: Some(scenario.to_string()),
            arm: Some(EvalArm::MemoryItems.as_str().to_string()),
            include_recent_commits: false,
            limit: Some(10),
        })
        .await
        .expect("orient should return a packet");
    assert!(packet
        .preferences
        .iter()
        .any(|item| item.id == preference.id));

    let memory_trace_id = packet.trace_id.expect("orient should return a trace id");
    let memory_trace = telemetry_service
        .get_trace(&memory_trace_id)
        .await
        .expect("trace lookup should run")
        .expect("memory trace should exist");
    let memory_items = eval_trace_from_operation(
        scenario,
        EvalArm::MemoryItems,
        prompt,
        &memory_trace,
        vec![preference.id.to_string()],
        Vec::new(),
        EvalOutcome {
            success: true,
            preference_adhered: true,
            repeated_context_questions: 0,
            conflict_resolution_correct: None,
            bad_memory_used: false,
            quality_score: 0.9,
        },
    );

    let mut feedback = AgentFeedback::new(memory_trace_id);
    feedback.agent = Some("codex".to_string());
    feedback.used_memory_ids = vec![preference.id];
    feedback.rejected_memory_ids = vec![distractor.id];
    feedback.usefulness_score = Some(5);
    feedback.correctness_score = Some(5);
    feedback.noise_score = Some(2);
    feedback.note = Some("Preference was retrieved and applied without asking again.".to_string());
    telemetry_service
        .submit_feedback(feedback)
        .await
        .expect("memory feedback should be recorded");

    let comparison = ConfidenceScenarioComparison {
        scenario: scenario.to_string(),
        no_memory,
        memory_items,
    };
    assert!(comparison.memory_items_improved());
    assert_eq!(
        comparison.no_memory.memory_calls[0].missing_expected_item_ids,
        vec![preference.id.to_string()]
    );
    assert_eq!(comparison.memory_items.useful_memory_count(), 1);

    let stats = telemetry_service
        .stats_by_intent()
        .await
        .expect("intent stats should aggregate");
    let preference_stats = stats
        .iter()
        .find(|item| item.intent == "follow_user_preference")
        .expect("follow_user_preference stats should exist");
    assert_eq!(preference_stats.trace_count, 2);
    assert_eq!(preference_stats.feedback_count, 2);
    assert_eq!(preference_stats.missing_context_count, 1);
    assert_eq!(preference_stats.used_memory_count, 1);
    assert_eq!(preference_stats.rejected_memory_count, 1);
}

#[tokio::test]
async fn confidence_scenario_memoryitems_reject_stale_and_exclude_wrong_scope_memory() {
    let (memory_service, telemetry_service) = setup_memory_and_telemetry_services().await;
    let current_rule = memory_service
        .capture_memory(
            MemoryItem::new(
                MemoryKind::Rule,
                "Run relevant tests after code changes",
                "After modifying Engram code, run the focused test command that covers the change.",
                MemoryScope::project("engram"),
                ClaimOrigin::UserStated,
                writer(),
            )
            .with_evidence(reviewed_evidence(
                "Project instruction requires verification after edits.",
            )),
        )
        .await
        .expect("current rule should be captured");
    let stale_rule = memory_service
        .capture_memory(
            MemoryItem::new(
                MemoryKind::Rule,
                "Skip tests for documentation-era spikes",
                "Old spike workflow said tests could be skipped when changing brain-harness experiments.",
                MemoryScope::project("engram"),
                ClaimOrigin::AgentObserved,
                writer(),
            )
            .with_review_after(time::OffsetDateTime::now_utc() - time::Duration::days(1)),
        )
        .await
        .expect("stale rule should be captured");
    let wrong_scope_rule = memory_service
        .capture_memory(MemoryItem::new(
            MemoryKind::Rule,
            "Skip tests in unrelated prototype",
            "The unrelated prototype can skip tests for exploratory changes.",
            MemoryScope::project("other-project"),
            ClaimOrigin::UserStated,
            writer(),
        ))
        .await
        .expect("wrong-scope rule should be captured");

    let scenario = "stale_and_wrong_scope_rejection";
    let prompt = "Modify Engram code and verify the change.";
    let no_memory = record_no_memory_baseline(
        &telemetry_service,
        scenario,
        prompt,
        BrainHarnessIntent::ImplementChange,
        vec![current_rule.id.to_string()],
        "Expected Engram verification policy before finishing the implementation.",
        EvalOutcome {
            success: false,
            preference_adhered: false,
            repeated_context_questions: 1,
            conflict_resolution_correct: Some(false),
            bad_memory_used: false,
            quality_score: 0.4,
        },
    )
    .await;

    let packet = memory_service
        .orient(OrientInput {
            project: Some("engram".to_string()),
            cwd: Some("/Users/yuval.meiri/projects/engram".to_string()),
            prompt: Some(prompt.to_string()),
            agent: Some("codex".to_string()),
            intent: Some(BrainHarnessIntent::ImplementChange),
            scenario_id: Some(scenario.to_string()),
            arm: Some(EvalArm::MemoryItems.as_str().to_string()),
            include_recent_commits: false,
            limit: Some(10),
        })
        .await
        .expect("orient should return a packet");
    let returned_ids = packet
        .memory_metadata
        .iter()
        .map(|metadata| metadata.memory_id)
        .collect::<Vec<_>>();
    assert!(returned_ids.contains(&current_rule.id));
    assert!(returned_ids.contains(&stale_rule.id));
    assert!(!returned_ids.contains(&wrong_scope_rule.id));
    let stale_metadata = packet
        .memory_metadata
        .iter()
        .find(|metadata| metadata.memory_id == stale_rule.id)
        .expect("stale rule metadata should be surfaced");
    assert_eq!(stale_metadata.freshness, MemoryFreshness::ReviewDue);
    assert!(stale_metadata.review_due);

    let trace_id = packet.trace_id.expect("orient should return a trace id");
    let trace = telemetry_service
        .get_trace(&trace_id)
        .await
        .expect("trace lookup should run")
        .expect("trace should exist");
    let memory_items = eval_trace_from_operation(
        scenario,
        EvalArm::MemoryItems,
        prompt,
        &trace,
        vec![current_rule.id.to_string()],
        Vec::new(),
        EvalOutcome {
            success: true,
            preference_adhered: true,
            repeated_context_questions: 0,
            conflict_resolution_correct: Some(true),
            bad_memory_used: false,
            quality_score: 0.86,
        },
    );

    let mut feedback = AgentFeedback::new(trace_id);
    feedback.agent = Some("codex".to_string());
    feedback.used_memory_ids = vec![current_rule.id];
    feedback.rejected_memory_ids = vec![stale_rule.id];
    feedback.stale_memory_ids = vec![stale_rule.id];
    feedback.usefulness_score = Some(4);
    feedback.correctness_score = Some(5);
    feedback.noise_score = Some(2);
    feedback.note =
        Some("Used current verification rule and rejected review-due guidance.".to_string());
    telemetry_service
        .submit_feedback(feedback)
        .await
        .expect("feedback should be recorded");

    let comparison = ConfidenceScenarioComparison {
        scenario: scenario.to_string(),
        no_memory,
        memory_items,
    };
    assert!(comparison.memory_items_improved());
    assert!(!comparison
        .memory_items
        .retrieved_item_ids
        .contains(&wrong_scope_rule.id.to_string()));
    assert!(comparison
        .memory_items
        .retrieved_item_ids
        .contains(&stale_rule.id.to_string()));
}

#[tokio::test]
async fn confidence_scenario_memoryitems_preserve_decision_continuity() {
    let (memory_service, telemetry_service) = setup_memory_and_telemetry_services().await;
    let next_step = memory_service
        .capture_memory(
            MemoryItem::new(
                MemoryKind::Decision,
                "Next step is confidence scenarios",
                "After shared MemoryItem ranking and trust metadata, build deterministic brain-harness confidence scenarios before legacy migration.",
                MemoryScope::project("engram"),
                ClaimOrigin::AgentObserved,
                writer(),
            )
            .with_evidence(reviewed_evidence("Architecture plan gates migration on eval confidence.")),
        )
        .await
        .expect("next-step decision should be captured");
    let guardrail = memory_service
        .capture_memory(
            MemoryItem::new(
                MemoryKind::Rule,
                "Do not migrate legacy layers before eval evidence",
                "Legacy entity/session/work layers should remain until MemoryItems show better agent outcomes and migration preserves important knowledge.",
                MemoryScope::project("engram"),
                ClaimOrigin::AgentObserved,
                writer(),
            )
            .with_evidence(reviewed_evidence("Brain harness RFC says canonicality must be proven.")),
        )
        .await
        .expect("guardrail should be captured");

    let scenario = "decision_continuity";
    let prompt = "What is the correct next implementation step for Engram after the shared ranker?";
    let no_memory = record_no_memory_baseline(
        &telemetry_service,
        scenario,
        prompt,
        BrainHarnessIntent::PlanWork,
        vec![next_step.id.to_string(), guardrail.id.to_string()],
        "Expected the recent decision to run confidence scenarios before migration.",
        EvalOutcome {
            success: false,
            preference_adhered: false,
            repeated_context_questions: 2,
            conflict_resolution_correct: Some(false),
            bad_memory_used: false,
            quality_score: 0.3,
        },
    )
    .await;

    let packet = memory_service
        .orient(OrientInput {
            project: Some("engram".to_string()),
            cwd: Some("/Users/yuval.meiri/projects/engram".to_string()),
            prompt: Some(prompt.to_string()),
            agent: Some("codex".to_string()),
            intent: Some(BrainHarnessIntent::PlanWork),
            scenario_id: Some(scenario.to_string()),
            arm: Some(EvalArm::MemoryItems.as_str().to_string()),
            include_recent_commits: false,
            limit: Some(10),
        })
        .await
        .expect("orient should return a packet");
    assert!(packet
        .active_decisions
        .iter()
        .any(|item| item.id == next_step.id));
    assert!(packet
        .active_rules
        .iter()
        .any(|item| item.id == guardrail.id));

    let trace_id = packet.trace_id.expect("orient should return a trace id");
    let trace = telemetry_service
        .get_trace(&trace_id)
        .await
        .expect("trace lookup should run")
        .expect("trace should exist");
    let memory_items = eval_trace_from_operation(
        scenario,
        EvalArm::MemoryItems,
        prompt,
        &trace,
        vec![next_step.id.to_string(), guardrail.id.to_string()],
        Vec::new(),
        EvalOutcome {
            success: true,
            preference_adhered: true,
            repeated_context_questions: 0,
            conflict_resolution_correct: Some(true),
            bad_memory_used: false,
            quality_score: 0.92,
        },
    );

    let mut feedback = AgentFeedback::new(trace_id);
    feedback.agent = Some("codex".to_string());
    feedback.used_memory_ids = vec![next_step.id, guardrail.id];
    feedback.usefulness_score = Some(5);
    feedback.correctness_score = Some(5);
    feedback.noise_score = Some(1);
    feedback.note =
        Some("Recent roadmap decision and migration guardrail preserved continuity.".to_string());
    telemetry_service
        .submit_feedback(feedback)
        .await
        .expect("feedback should be recorded");

    let comparison = ConfidenceScenarioComparison {
        scenario: scenario.to_string(),
        no_memory,
        memory_items,
    };
    assert!(comparison.memory_items_improved());
    assert_eq!(comparison.memory_items.useful_memory_count(), 2);
    assert_eq!(
        comparison.memory_items.outcome.conflict_resolution_correct,
        Some(true)
    );
}

#[tokio::test]
async fn confidence_scenarios_compare_memoryitems_with_legacy_and_hybrid() {
    let services = setup_brain_harness_eval_services().await;
    let mut report_traces = Vec::new();
    services
        .entity
        .create_entity("engram", EntityType::Repo, Some("Engram repository"))
        .await
        .expect("engram entity should be created");
    let _engram_project = services
        .work
        .create_project("engram", Some("Engram project"))
        .await
        .expect("engram project should be created");
    let _other_project = services
        .work
        .create_project("other-project", Some("Unrelated prototype"))
        .await
        .expect("other project should be created");

    let (legacy_preference, _) = services
        .entity
        .add_observation(
            "engram",
            "The user wants each successful implementation checkpoint committed before continuing.",
            Some("preferences.commit-workflow"),
            Some("legacy-eval"),
        )
        .await
        .expect("legacy preference should be recorded");
    let preference = services
        .memory
        .capture_memory(
            MemoryItem::new(
                MemoryKind::Preference,
                "Commit every completed step",
                "The user wants each successful implementation checkpoint committed before continuing.",
                MemoryScope::project("engram"),
                ClaimOrigin::UserStated,
                writer(),
            )
            .with_evidence(reviewed_evidence(
                "User explicitly confirmed commit-every-step policy.",
            )),
        )
        .await
        .expect("preference should be captured");

    let scenario = "preference_continuity";
    let prompt = "Continue Engram implementation and finish a checkpoint.";
    let no_memory = record_no_memory_baseline(
        &services.telemetry,
        scenario,
        prompt,
        BrainHarnessIntent::FollowUserPreference,
        vec![preference.id.to_string()],
        "Expected the user preference requiring commits after completed checkpoints.",
        EvalOutcome {
            success: false,
            preference_adhered: false,
            repeated_context_questions: 1,
            conflict_resolution_correct: None,
            bad_memory_used: false,
            quality_score: 0.35,
        },
    )
    .await;
    let legacy_search_results = services
        .search
        .search_with_options(
            "successful implementation checkpoint committed before continuing",
            10,
            Some(0.0),
            Some(&[SearchLayer::Observation]),
            SearchOptions::default(),
        )
        .await
        .expect("legacy observation search should run");
    let legacy_preference_ids = legacy_search_results
        .iter()
        .map(|result| result.id.clone())
        .collect::<Vec<_>>();
    assert!(legacy_preference_ids.contains(&legacy_preference.id.to_string()));
    let legacy = record_legacy_trace(
        &services,
        scenario,
        prompt,
        BrainHarnessIntent::FollowUserPreference,
        legacy_preference_ids.clone(),
        vec![legacy_preference.id.to_string()],
        EvalOutcome {
            success: true,
            preference_adhered: true,
            repeated_context_questions: 0,
            conflict_resolution_correct: None,
            bad_memory_used: false,
            quality_score: 0.72,
        },
    )
    .await;
    let packet = services
        .memory
        .orient(OrientInput {
            project: Some("engram".to_string()),
            cwd: Some("/Users/yuval.meiri/projects/engram".to_string()),
            prompt: Some(prompt.to_string()),
            agent: Some("codex".to_string()),
            intent: Some(BrainHarnessIntent::FollowUserPreference),
            scenario_id: Some(scenario.to_string()),
            arm: Some(EvalArm::MemoryItems.as_str().to_string()),
            include_recent_commits: false,
            limit: Some(10),
        })
        .await
        .expect("orient should return a packet");
    assert!(packet
        .preferences
        .iter()
        .any(|item| item.id == preference.id));
    let memory_items = memoryitem_trace_from_orient(
        &services,
        scenario,
        prompt,
        packet.trace_id.expect("orient should return a trace id"),
        vec![preference.id],
        Vec::new(),
        EvalOutcome {
            success: true,
            preference_adhered: true,
            repeated_context_questions: 0,
            conflict_resolution_correct: None,
            bad_memory_used: false,
            quality_score: 0.9,
        },
    )
    .await;
    let hybrid = record_hybrid_trace(
        &services,
        scenario,
        prompt,
        BrainHarnessIntent::FollowUserPreference,
        vec![preference.id],
        legacy_preference_ids,
        vec![preference.id.to_string(), legacy_preference.id.to_string()],
        EvalOutcome {
            success: true,
            preference_adhered: true,
            repeated_context_questions: 0,
            conflict_resolution_correct: None,
            bad_memory_used: false,
            quality_score: 0.94,
        },
    )
    .await;
    let comparison = FullScenarioComparison {
        scenario: scenario.to_string(),
        no_memory,
        legacy,
        memory_items,
        hybrid,
    };
    assert!(comparison.memory_items_outperform_legacy());
    assert!(comparison.hybrid_preserves_legacy_evidence());
    assert_eq!(comparison.memory_items.useful_memory_count(), 1);
    assert_eq!(
        comparison.no_memory.memory_calls[0].missing_expected_item_ids,
        vec![preference.id.to_string()]
    );
    report_traces.extend([
        comparison.no_memory.clone(),
        comparison.legacy.clone(),
        comparison.memory_items.clone(),
        comparison.hybrid.clone(),
    ]);

    let current_legacy_rule = services
        .work
        .add_project_observation(
            "engram",
            "After modifying Engram code, run the focused test command that covers the change.",
            Some("testing.verification.current"),
        )
        .await
        .expect("current project observation should be recorded");
    let stale_legacy_rule = services
        .work
        .add_project_observation(
            "engram",
            "Old spike workflow said tests could be skipped when changing brain-harness experiments.",
            Some("testing.verification.stale"),
        )
        .await
        .expect("stale project observation should be recorded");
    let wrong_scope_legacy_rule = services
        .work
        .add_project_observation(
            "other-project",
            "The unrelated prototype can skip tests for exploratory changes.",
            Some("testing.verification.wrong-scope"),
        )
        .await
        .expect("wrong-scope project observation should be recorded");
    let current_rule = services
        .memory
        .capture_memory(
            MemoryItem::new(
                MemoryKind::Rule,
                "Run relevant tests after code changes",
                "After modifying Engram code, run the focused test command that covers the change.",
                MemoryScope::project("engram"),
                ClaimOrigin::UserStated,
                writer(),
            )
            .with_evidence(reviewed_evidence(
                "Project instruction requires verification after edits.",
            )),
        )
        .await
        .expect("current rule should be captured");
    let stale_rule = services
        .memory
        .capture_memory(
            MemoryItem::new(
                MemoryKind::Rule,
                "Skip tests for documentation-era spikes",
                "Old spike workflow said tests could be skipped when changing brain-harness experiments.",
                MemoryScope::project("engram"),
                ClaimOrigin::AgentObserved,
                writer(),
            )
            .with_review_after(time::OffsetDateTime::now_utc() - time::Duration::days(1)),
        )
        .await
        .expect("stale rule should be captured");
    let wrong_scope_rule = services
        .memory
        .capture_memory(MemoryItem::new(
            MemoryKind::Rule,
            "Skip tests in unrelated prototype",
            "The unrelated prototype can skip tests for exploratory changes.",
            MemoryScope::project("other-project"),
            ClaimOrigin::UserStated,
            writer(),
        ))
        .await
        .expect("wrong-scope rule should be captured");

    let scenario = "stale_and_wrong_scope_rejection";
    let prompt = "Modify Engram code and verify the change.";
    let no_memory = record_no_memory_baseline(
        &services.telemetry,
        scenario,
        prompt,
        BrainHarnessIntent::ImplementChange,
        vec![current_rule.id.to_string()],
        "Expected Engram verification policy before finishing the implementation.",
        EvalOutcome {
            success: false,
            preference_adhered: false,
            repeated_context_questions: 1,
            conflict_resolution_correct: Some(false),
            bad_memory_used: false,
            quality_score: 0.4,
        },
    )
    .await;
    let work_context = services
        .work
        .get_full_context("engram", None)
        .await
        .expect("work context should be retrieved");
    let legacy_rule_ids = work_context
        .project_observations
        .iter()
        .map(|observation| observation.id.to_string())
        .collect::<Vec<_>>();
    assert!(legacy_rule_ids.contains(&current_legacy_rule.id.to_string()));
    assert!(legacy_rule_ids.contains(&stale_legacy_rule.id.to_string()));
    assert!(!legacy_rule_ids.contains(&wrong_scope_legacy_rule.id.to_string()));
    let legacy = record_legacy_trace(
        &services,
        scenario,
        prompt,
        BrainHarnessIntent::ImplementChange,
        legacy_rule_ids.clone(),
        vec![
            current_legacy_rule.id.to_string(),
            stale_legacy_rule.id.to_string(),
        ],
        EvalOutcome {
            success: true,
            preference_adhered: false,
            repeated_context_questions: 0,
            conflict_resolution_correct: Some(false),
            bad_memory_used: true,
            quality_score: 0.58,
        },
    )
    .await;
    let packet = services
        .memory
        .orient(OrientInput {
            project: Some("engram".to_string()),
            cwd: Some("/Users/yuval.meiri/projects/engram".to_string()),
            prompt: Some(prompt.to_string()),
            agent: Some("codex".to_string()),
            intent: Some(BrainHarnessIntent::ImplementChange),
            scenario_id: Some(scenario.to_string()),
            arm: Some(EvalArm::MemoryItems.as_str().to_string()),
            include_recent_commits: false,
            limit: Some(10),
        })
        .await
        .expect("orient should return a packet");
    let returned_rule_ids = packet
        .memory_metadata
        .iter()
        .map(|metadata| metadata.memory_id)
        .collect::<Vec<_>>();
    assert!(returned_rule_ids.contains(&current_rule.id));
    assert!(returned_rule_ids.contains(&stale_rule.id));
    assert!(!returned_rule_ids.contains(&wrong_scope_rule.id));
    let stale_metadata = packet
        .memory_metadata
        .iter()
        .find(|metadata| metadata.memory_id == stale_rule.id)
        .expect("stale rule metadata should be surfaced");
    assert_eq!(stale_metadata.freshness, MemoryFreshness::ReviewDue);
    assert!(stale_metadata.review_due);
    let memory_items = memoryitem_trace_from_orient(
        &services,
        scenario,
        prompt,
        packet.trace_id.expect("orient should return a trace id"),
        vec![current_rule.id],
        vec![stale_rule.id],
        EvalOutcome {
            success: true,
            preference_adhered: true,
            repeated_context_questions: 0,
            conflict_resolution_correct: Some(true),
            bad_memory_used: false,
            quality_score: 0.86,
        },
    )
    .await;
    let hybrid = record_hybrid_trace(
        &services,
        scenario,
        prompt,
        BrainHarnessIntent::ImplementChange,
        returned_rule_ids,
        legacy_rule_ids,
        vec![
            current_rule.id.to_string(),
            current_legacy_rule.id.to_string(),
        ],
        EvalOutcome {
            success: true,
            preference_adhered: true,
            repeated_context_questions: 0,
            conflict_resolution_correct: Some(true),
            bad_memory_used: false,
            quality_score: 0.88,
        },
    )
    .await;
    let comparison = FullScenarioComparison {
        scenario: scenario.to_string(),
        no_memory,
        legacy,
        memory_items,
        hybrid,
    };
    assert!(comparison.memory_items_outperform_legacy());
    assert!(comparison.hybrid_preserves_legacy_evidence());
    assert!(comparison
        .legacy
        .retrieved_item_ids
        .contains(&stale_legacy_rule.id.to_string()));
    assert!(!comparison
        .memory_items
        .retrieved_item_ids
        .contains(&wrong_scope_rule.id.to_string()));
    report_traces.extend([
        comparison.no_memory.clone(),
        comparison.legacy.clone(),
        comparison.memory_items.clone(),
        comparison.hybrid.clone(),
    ]);

    let session = services
        .session
        .start_session(
            Some("codex"),
            Some("engram"),
            Some("Brain harness planning"),
        )
        .await
        .expect("session should start");
    let legacy_decision = services
        .session
        .log_decision(
            &session.id,
            "After shared MemoryItem ranking and trust metadata, build deterministic brain-harness confidence scenarios before legacy migration.",
            Some("Architecture plan gates migration on eval evidence."),
        )
        .await
        .expect("legacy decision should be recorded");
    let legacy_guardrail = services
        .session
        .log_event(
            &session.id,
            EventType::Rule,
            "Legacy entity/session/work layers should remain until MemoryItems show better agent outcomes and migration preserves important knowledge.",
            Some("Brain harness RFC says canonicality must be proven."),
            Some("legacy-eval"),
        )
        .await
        .expect("legacy guardrail should be recorded");
    let next_step = services
        .memory
        .capture_memory(
            MemoryItem::new(
                MemoryKind::Decision,
                "Next step is confidence scenarios",
                "After shared MemoryItem ranking and trust metadata, build deterministic brain-harness confidence scenarios before legacy migration.",
                MemoryScope::project("engram"),
                ClaimOrigin::AgentObserved,
                writer(),
            )
            .with_evidence(reviewed_evidence(
                "Architecture plan gates migration on eval confidence.",
            )),
        )
        .await
        .expect("next-step decision should be captured");
    let guardrail = services
        .memory
        .capture_memory(
            MemoryItem::new(
                MemoryKind::Rule,
                "Do not migrate legacy layers before eval evidence",
                "Legacy entity/session/work layers should remain until MemoryItems show better agent outcomes and migration preserves important knowledge.",
                MemoryScope::project("engram"),
                ClaimOrigin::AgentObserved,
                writer(),
            )
            .with_evidence(reviewed_evidence(
                "Brain harness RFC says canonicality must be proven.",
            )),
        )
        .await
        .expect("guardrail should be captured");

    let scenario = "decision_continuity";
    let prompt = "What is the correct next implementation step for Engram after the shared ranker?";
    let no_memory = record_no_memory_baseline(
        &services.telemetry,
        scenario,
        prompt,
        BrainHarnessIntent::PlanWork,
        vec![next_step.id.to_string(), guardrail.id.to_string()],
        "Expected the recent decision to run confidence scenarios before migration.",
        EvalOutcome {
            success: false,
            preference_adhered: false,
            repeated_context_questions: 2,
            conflict_resolution_correct: Some(false),
            bad_memory_used: false,
            quality_score: 0.3,
        },
    )
    .await;
    let legacy_decision_results = services
        .search
        .search_with_options(
            "build deterministic brain-harness confidence scenarios before legacy migration",
            10,
            Some(0.0),
            Some(&[SearchLayer::SessionEvent]),
            SearchOptions::default(),
        )
        .await
        .expect("legacy session search should run");
    let mut legacy_decision_ids = legacy_decision_results
        .iter()
        .map(|result| result.id.clone())
        .collect::<Vec<_>>();
    assert!(legacy_decision_ids.contains(&legacy_decision.id.to_string()));
    legacy_decision_ids.push(legacy_guardrail.id.to_string());
    let legacy = record_legacy_trace(
        &services,
        scenario,
        prompt,
        BrainHarnessIntent::PlanWork,
        legacy_decision_ids.clone(),
        vec![
            legacy_decision.id.to_string(),
            legacy_guardrail.id.to_string(),
        ],
        EvalOutcome {
            success: true,
            preference_adhered: true,
            repeated_context_questions: 0,
            conflict_resolution_correct: Some(true),
            bad_memory_used: false,
            quality_score: 0.7,
        },
    )
    .await;
    let packet = services
        .memory
        .orient(OrientInput {
            project: Some("engram".to_string()),
            cwd: Some("/Users/yuval.meiri/projects/engram".to_string()),
            prompt: Some(prompt.to_string()),
            agent: Some("codex".to_string()),
            intent: Some(BrainHarnessIntent::PlanWork),
            scenario_id: Some(scenario.to_string()),
            arm: Some(EvalArm::MemoryItems.as_str().to_string()),
            include_recent_commits: false,
            limit: Some(10),
        })
        .await
        .expect("orient should return a packet");
    assert!(packet
        .active_decisions
        .iter()
        .any(|item| item.id == next_step.id));
    assert!(packet
        .active_rules
        .iter()
        .any(|item| item.id == guardrail.id));
    let returned_decision_ids = packet
        .memory_metadata
        .iter()
        .map(|metadata| metadata.memory_id)
        .collect::<Vec<_>>();
    let memory_items = memoryitem_trace_from_orient(
        &services,
        scenario,
        prompt,
        packet.trace_id.expect("orient should return a trace id"),
        vec![next_step.id, guardrail.id],
        Vec::new(),
        EvalOutcome {
            success: true,
            preference_adhered: true,
            repeated_context_questions: 0,
            conflict_resolution_correct: Some(true),
            bad_memory_used: false,
            quality_score: 0.92,
        },
    )
    .await;
    let hybrid = record_hybrid_trace(
        &services,
        scenario,
        prompt,
        BrainHarnessIntent::PlanWork,
        returned_decision_ids,
        legacy_decision_ids,
        vec![
            next_step.id.to_string(),
            guardrail.id.to_string(),
            legacy_decision.id.to_string(),
            legacy_guardrail.id.to_string(),
        ],
        EvalOutcome {
            success: true,
            preference_adhered: true,
            repeated_context_questions: 0,
            conflict_resolution_correct: Some(true),
            bad_memory_used: false,
            quality_score: 0.95,
        },
    )
    .await;
    let comparison = FullScenarioComparison {
        scenario: scenario.to_string(),
        no_memory,
        legacy,
        memory_items,
        hybrid,
    };
    assert!(comparison.memory_items_outperform_legacy());
    assert!(comparison.hybrid_preserves_legacy_evidence());
    assert_eq!(comparison.memory_items.useful_memory_count(), 2);
    report_traces.extend([
        comparison.no_memory.clone(),
        comparison.legacy.clone(),
        comparison.memory_items.clone(),
        comparison.hybrid.clone(),
    ]);

    let report = BrainHarnessEvalReport::new(report_traces);
    assert_eq!(report.scenarios().len(), 3);
    assert!(report.memory_items_outperform_legacy());
    assert!(report.hybrid_preserves_legacy_and_memory_evidence());
    assert!(report.no_memory_records_missing_expected_context());

    let no_memory_summary = report.arm_summary(EvalArm::NoMemory);
    assert_eq!(no_memory_summary.arm, EvalArm::NoMemory);
    assert_eq!(no_memory_summary.trace_count, 3);
    assert_eq!(no_memory_summary.success_count, 0);
    assert_eq!(no_memory_summary.bad_memory_count, 0);
    assert_eq!(no_memory_summary.missing_expected_count, 4);
    assert_eq!(no_memory_summary.repeated_context_questions, 4);
    assert_eq!(no_memory_summary.avg_retrieval_precision, 0.0);

    let legacy_summary = report.arm_summary(EvalArm::LegacyObservations);
    assert_eq!(legacy_summary.arm, EvalArm::LegacyObservations);
    assert_eq!(legacy_summary.trace_count, 3);
    assert_eq!(legacy_summary.success_count, 3);
    assert_eq!(legacy_summary.bad_memory_count, 1);
    assert!(legacy_summary.avg_quality_score > no_memory_summary.avg_quality_score);

    let memory_summary = report.arm_summary(EvalArm::MemoryItems);
    assert_eq!(memory_summary.arm, EvalArm::MemoryItems);
    assert_eq!(memory_summary.trace_count, 3);
    assert_eq!(memory_summary.success_count, 3);
    assert_eq!(memory_summary.bad_memory_count, 0);
    assert_eq!(memory_summary.repeated_context_questions, 0);
    assert!(memory_summary.avg_quality_score > legacy_summary.avg_quality_score);
    assert!(memory_summary.avg_retrieval_precision > 0.0);

    let hybrid_summary = report.arm_summary(EvalArm::Hybrid);
    assert_eq!(hybrid_summary.arm, EvalArm::Hybrid);
    assert_eq!(hybrid_summary.trace_count, 3);
    assert_eq!(hybrid_summary.success_count, 3);
    assert_eq!(hybrid_summary.bad_memory_count, 0);
    assert!(hybrid_summary.avg_quality_score >= memory_summary.avg_quality_score);
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
            scenario_id: None,
            arm: None,
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
    assert!(!packet.brain_loop.degraded);
    assert!(packet
        .brain_loop
        .compiled_context
        .contains("Use concise status updates"));
    assert!(packet
        .brain_loop
        .top_items
        .iter()
        .any(|item| { item.id == captured.id && item.trust.memory_id == captured.id }));
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
    )
    .with_evidence(reviewed_evidence(
        "Architecture work accepted MemoryItems as first-class retrieval results.",
    ));
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

#[tokio::test]
async fn orient_and_memory_search_share_memoryitem_ranking_order() {
    let (memory_service, search_service) = setup_memory_and_search_services().await;
    for title in ["Shared ranker first", "Shared ranker second"] {
        memory_service
            .capture_memory(
                MemoryItem::new(
                    MemoryKind::Decision,
                    title,
                    "Shared ranker should order MemoryItems consistently for orient and search.",
                    MemoryScope::project("engram"),
                    ClaimOrigin::UserStated,
                    writer(),
                )
                .with_evidence(reviewed_evidence(
                    "Shared ranker ordering is accepted as active guidance.",
                )),
            )
            .await
            .expect("memory should be captured");
    }

    let query = "shared ranker order MemoryItems consistently";
    let packet = memory_service
        .orient(OrientInput {
            project: Some("engram".to_string()),
            cwd: Some("/Users/yuval.meiri/projects/engram".to_string()),
            prompt: Some(query.to_string()),
            agent: Some("codex".to_string()),
            intent: Some(BrainHarnessIntent::VerifyDecision),
            scenario_id: None,
            arm: None,
            include_recent_commits: false,
            limit: Some(10),
        })
        .await
        .expect("orient should return a packet");
    let search_results = search_service
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
        .expect("memory search should run");

    let oriented_ids = packet
        .active_decisions
        .iter()
        .map(|item| item.id.to_string())
        .collect::<Vec<_>>();
    let searched_ids = search_results
        .iter()
        .map(|result| result.id.clone())
        .collect::<Vec<_>>();

    assert_eq!(oriented_ids, searched_ids);
    assert!(packet
        .memory_metadata
        .iter()
        .all(|metadata| metadata.review_state == MemoryReviewState::Reviewed));
    assert!(search_results
        .iter()
        .all(|result| result.memory_metadata.is_some()));
}
