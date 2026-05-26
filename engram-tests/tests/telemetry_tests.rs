//! Integration tests for brain-harness telemetry and agent feedback.

use engram_core::memory::{
    ClaimOrigin, EvidenceKind, EvidenceRef, Harness, MemoryItem, MemoryKind, MemoryScope,
    ModelIdentity, WriterProvenance,
};
use engram_core::telemetry::{
    AgentFeedback, BrainHarnessIntent, BrainHarnessOperation, BrainHarnessTrace,
};
use engram_index::{
    MemoryChangesSinceOptions, MemoryService, OrientInput, SearchService, TelemetryService,
};
use engram_mcp::tools::{self, OrientRequest, SearchRequest, TelemetryRequest, ToolState};
use engram_store::{connect_and_init, StoreConfig};
use serde_json::Value;

async fn setup_services() -> (TelemetryService, MemoryService) {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config)
        .await
        .expect("failed to connect to in-memory store");

    let telemetry = TelemetryService::new(db.clone());
    telemetry
        .init_schema()
        .await
        .expect("failed to initialize telemetry schema");

    let memory = MemoryService::new(db);
    memory
        .init_schema()
        .await
        .expect("failed to initialize memory schema");

    (telemetry, memory)
}

fn writer() -> WriterProvenance {
    WriterProvenance::agent(Harness::Codex, ModelIdentity::new("openai", "gpt-5.5"))
        .with_surface("telemetry-test")
}

fn telemetry_request(action: &str) -> TelemetryRequest {
    TelemetryRequest {
        action: action.to_string(),
        trace_id: None,
        operation: None,
        intent: None,
        scenario_id: None,
        arm: None,
        query: None,
        project: None,
        agent: None,
        session_id: None,
        external_session_id: None,
        returned_memory_ids: Vec::new(),
        returned_result_ids: Vec::new(),
        latency_ms: None,
        warnings: Vec::new(),
        used_memory_ids: Vec::new(),
        rejected_memory_ids: Vec::new(),
        used_result_ids: Vec::new(),
        rejected_result_ids: Vec::new(),
        stale_memory_ids: Vec::new(),
        wrong_scope_memory_ids: Vec::new(),
        missing_context: None,
        usefulness_score: None,
        correctness_score: None,
        noise_score: None,
        task_success: None,
        preference_adhered: None,
        repeated_context_questions: None,
        bad_memory_used: None,
        suggested_memory_changes: None,
        note: None,
        limit: None,
    }
}

fn parse_json(response: &str) -> Value {
    serde_json::from_str(response).expect("response should be valid JSON")
}

#[tokio::test]
async fn trace_and_feedback_are_persisted_and_aggregated_by_intent() {
    let (telemetry, _) = setup_services().await;
    let memory_id = engram_core::Id::new();
    let trace = BrainHarnessTrace::new(BrainHarnessOperation::Orient)
        .with_agent(Some("codex".to_string()))
        .with_intent(Some(BrainHarnessIntent::ResumeSession))
        .with_query(Some("continue telemetry spike".to_string()))
        .with_project(Some("engram".to_string()))
        .with_returned_memory_ids(vec![memory_id])
        .with_latency_ms(42);

    let trace = telemetry
        .record_trace(trace)
        .await
        .expect("trace should be recorded");
    let stored = telemetry
        .get_trace(&trace.id)
        .await
        .expect("trace lookup should run")
        .expect("trace should exist");
    assert_eq!(stored.intent, Some(BrainHarnessIntent::ResumeSession));
    assert_eq!(stored.returned_memory_ids, vec![memory_id]);

    let mut feedback = AgentFeedback::new(trace.id);
    feedback.agent = Some("codex".to_string());
    feedback.used_memory_ids = vec![memory_id];
    feedback.usefulness_score = Some(5);
    feedback.correctness_score = Some(4);
    feedback.noise_score = Some(1);
    feedback.note = Some("The retrieved preference was useful.".to_string());

    let feedback = telemetry
        .submit_feedback(feedback)
        .await
        .expect("feedback should be accepted");
    let stored_feedback = telemetry
        .list_feedback_for_trace(&trace.id)
        .await
        .expect("feedback lookup should run");
    assert_eq!(stored_feedback.len(), 1);
    assert_eq!(stored_feedback[0].id, feedback.id);

    let stats = telemetry
        .stats_by_intent()
        .await
        .expect("stats should aggregate");
    let resume = stats
        .iter()
        .find(|item| item.intent == "resume_session")
        .expect("resume_session stats should exist");
    assert_eq!(resume.trace_count, 1);
    assert_eq!(resume.feedback_count, 1);
    assert_eq!(resume.used_memory_count, 1);
    assert_eq!(resume.avg_latency_ms, Some(42.0));
    assert_eq!(resume.avg_usefulness_score, Some(5.0));
    assert_eq!(resume.avg_correctness_score, Some(4.0));
    assert_eq!(resume.avg_noise_score, Some(1.0));
}

#[tokio::test]
async fn feedback_rejects_scores_outside_one_to_five() {
    let (telemetry, _) = setup_services().await;
    let trace = telemetry
        .record_trace(
            BrainHarnessTrace::new(BrainHarnessOperation::Search)
                .with_query(Some("memory telemetry".to_string()))
                .with_intent(Some(BrainHarnessIntent::AnswerQuestion)),
        )
        .await
        .expect("trace should be recorded");

    let mut feedback = AgentFeedback::new(trace.id);
    feedback.note = Some("bad score".to_string());
    feedback.usefulness_score = Some(6);

    let err = telemetry.submit_feedback(feedback).await.unwrap_err();
    assert!(err.to_string().contains("usefulness_score"));
}

#[tokio::test]
async fn real_session_eval_report_summarizes_feedback_coverage_and_gate_reasons() {
    let (telemetry, _) = setup_services().await;
    let used_memory_id = engram_core::Id::new();
    let rejected_memory_id = engram_core::Id::new();
    let stale_memory_id = engram_core::Id::new();
    let wrong_scope_memory_id = engram_core::Id::new();

    let orient_trace = telemetry
        .record_trace(
            BrainHarnessTrace::new(BrainHarnessOperation::Orient)
                .with_agent(Some("codex".to_string()))
                .with_intent(Some(BrainHarnessIntent::ImplementChange))
                .with_scenario_id(Some("preference_continuity".to_string()))
                .with_arm(Some("memory_items".to_string()))
                .with_query(Some("continue the telemetry report work".to_string()))
                .with_project(Some("engram".to_string()))
                .with_returned_memory_ids(vec![used_memory_id, rejected_memory_id])
                .with_latency_ms(120)
                .with_warning("memory result set was truncated"),
        )
        .await
        .expect("orient trace should be recorded");
    telemetry
        .record_trace(
            BrainHarnessTrace::new(BrainHarnessOperation::Search)
                .with_agent(Some("codex".to_string()))
                .with_intent(Some(BrainHarnessIntent::AnswerQuestion))
                .with_query(Some("telemetry report".to_string()))
                .with_project(Some("engram".to_string()))
                .with_returned_result_ids(vec!["legacy:entity:telemetry".to_string()]),
        )
        .await
        .expect("search trace should be recorded");

    let mut feedback = AgentFeedback::new(orient_trace.id);
    feedback.agent = Some("codex".to_string());
    feedback.used_memory_ids = vec![used_memory_id];
    feedback.rejected_memory_ids = vec![rejected_memory_id];
    feedback.stale_memory_ids = vec![stale_memory_id];
    feedback.wrong_scope_memory_ids = vec![wrong_scope_memory_id];
    feedback.missing_context = Some("Expected the current M3/M6 gate decision.".to_string());
    feedback.suggested_memory_changes =
        Some("Record the real-session report as the next M3 evidence step.".to_string());
    feedback.usefulness_score = Some(4);
    feedback.correctness_score = Some(5);
    feedback.noise_score = Some(2);
    feedback.task_success = Some(true);
    feedback.preference_adhered = Some(true);
    feedback.repeated_context_questions = Some(0);
    feedback.bad_memory_used = Some(false);
    telemetry
        .submit_feedback(feedback)
        .await
        .expect("feedback should be accepted");

    let report = telemetry
        .real_session_eval_report(Some(100))
        .await
        .expect("real-session report should build");
    assert_eq!(report.sample_limit, 100);
    assert_eq!(report.trace_count, 2);
    assert_eq!(report.feedback_count, 1);
    assert_eq!(report.feedback_coverage, 0.5);
    assert_eq!(report.memory_judgment_feedback_count, 1);
    assert_eq!(report.memory_judgment_coverage, 1.0);
    assert_eq!(report.unjudged_memory_feedback_count, 0);
    assert_eq!(report.operation_counts["orient"], 1);
    assert_eq!(report.operation_counts["search"], 1);
    assert_eq!(report.distinct_scenario_count, 1);
    assert_eq!(report.distinct_arm_count, 1);
    assert_eq!(report.unspecified_scenario_trace_count, 1);
    assert_eq!(report.unspecified_arm_trace_count, 1);
    assert_eq!(report.scenario_counts["preference_continuity"], 1);
    assert_eq!(report.warning_count, 1);
    assert_eq!(report.returned_memory_count, 2);
    assert_eq!(report.returned_result_count, 1);
    assert_eq!(report.used_memory_count, 1);
    assert_eq!(report.rejected_memory_count, 1);
    assert_eq!(report.stale_memory_count, 1);
    assert_eq!(report.wrong_scope_memory_count, 1);
    assert_eq!(report.missing_context_count, 1);
    assert_eq!(report.suggested_change_count, 1);
    assert_eq!(report.scored_feedback_count, 1);
    assert_eq!(report.outcome_feedback_count, 1);
    assert_eq!(report.task_success_count, 1);
    assert_eq!(report.task_failure_count, 0);
    assert_eq!(report.preference_adhered_count, 1);
    assert_eq!(report.preference_violated_count, 0);
    assert_eq!(report.repeated_context_question_count, 0);
    assert_eq!(report.bad_memory_used_count, 0);

    let implement_row = report
        .intents
        .iter()
        .find(|row| row.intent == "implement_change")
        .expect("implement_change row should exist");
    assert_eq!(implement_row.trace_count, 1);
    assert_eq!(implement_row.feedback_count, 1);
    assert_eq!(implement_row.feedback_coverage, 1.0);
    assert_eq!(implement_row.avg_latency_ms, Some(120.0));
    assert_eq!(implement_row.avg_usefulness_score, Some(4.0));
    assert_eq!(implement_row.avg_correctness_score, Some(5.0));
    assert_eq!(implement_row.avg_noise_score, Some(2.0));
    assert_eq!(implement_row.warning_count, 1);
    assert_eq!(implement_row.outcome_feedback_count, 1);
    assert_eq!(implement_row.task_success_count, 1);
    assert_eq!(implement_row.preference_adhered_count, 1);

    let arm_row = report
        .arms
        .iter()
        .find(|row| row.arm == "memory_items")
        .expect("memory_items arm row should exist");
    assert_eq!(arm_row.trace_count, 1);
    assert_eq!(arm_row.feedback_count, 1);
    assert_eq!(arm_row.outcome_feedback_count, 1);
    assert_eq!(arm_row.task_success_count, 1);
    assert_eq!(arm_row.preference_adhered_count, 1);

    assert!(!report.confidence_gate.passed);
    assert_eq!(report.confidence_gate.min_outcome_feedback_count, 1);
    assert!(report
        .confidence_gate
        .reasons
        .iter()
        .any(|reason| reason.contains("20 real-session traces")));
    assert!(report
        .recommendations
        .iter()
        .any(|recommendation| recommendation.contains("Keep M6 write-apply blocked")));
}

#[tokio::test]
async fn real_session_eval_report_separates_trace_coverage_from_feedback_density() {
    let (telemetry, _) = setup_services().await;
    let used_memory_id = engram_core::Id::new();

    let trace_with_feedback = telemetry
        .record_trace(
            BrainHarnessTrace::new(BrainHarnessOperation::Orient)
                .with_intent(Some(BrainHarnessIntent::ImplementChange))
                .with_project(Some("engram".to_string()))
                .with_scenario_id(Some("telemetry_semantics_cleanup_001".to_string()))
                .with_arm(Some("memoryitem_orient".to_string()))
                .with_external_session_id(Some("codex://threads/telemetry-cleanup".to_string()))
                .with_returned_memory_ids(vec![used_memory_id]),
        )
        .await
        .expect("trace with feedback should be recorded");
    telemetry
        .record_trace(
            BrainHarnessTrace::new(BrainHarnessOperation::Orient)
                .with_intent(Some(BrainHarnessIntent::ImplementChange))
                .with_project(Some("engram".to_string()))
                .with_scenario_id(Some("telemetry_semantics_cleanup_001".to_string()))
                .with_arm(Some("memoryitem_orient".to_string()))
                .with_returned_memory_ids(vec![engram_core::Id::new()]),
        )
        .await
        .expect("trace without feedback should be recorded");

    let mut attribution_feedback = AgentFeedback::new(trace_with_feedback.id);
    attribution_feedback.used_memory_ids = vec![used_memory_id];
    telemetry
        .submit_feedback(attribution_feedback)
        .await
        .expect("attribution feedback should be accepted");

    let mut outcome_feedback = AgentFeedback::new(trace_with_feedback.id);
    outcome_feedback.task_success = Some(true);
    outcome_feedback.bad_memory_used = Some(false);
    telemetry
        .submit_feedback(outcome_feedback)
        .await
        .expect("outcome feedback should be accepted");

    let report = telemetry
        .real_session_eval_report_scoped(
            Some(100),
            Some("engram"),
            Some("telemetry_semantics_cleanup_001"),
            Some("memoryitem_orient"),
        )
        .await
        .expect("scoped report should build");

    assert_eq!(report.trace_count, 2);
    assert_eq!(report.feedback_count, 2);
    assert_eq!(report.feedback_trace_count, 1);
    assert_eq!(report.feedback_coverage, 0.5);
    assert_eq!(report.feedback_records_per_trace, 1.0);
    assert_eq!(report.memory_judgment_feedback_count, 1);
    assert_eq!(report.memory_judgment_trace_count, 1);
    assert_eq!(report.memory_judgment_trace_coverage, 0.5);
    assert_eq!(report.outcome_feedback_count, 1);
    assert_eq!(report.outcome_trace_count, 1);
    assert_eq!(report.outcome_coverage, 0.5);
    assert_eq!(report.external_session_feedback_count, 2);
    assert_eq!(report.distinct_external_session_feedback_count, 1);
    assert_eq!(report.unspecified_external_session_feedback_count, 0);

    let intent_row = report
        .intents
        .iter()
        .find(|row| row.intent == "implement_change")
        .expect("implement_change row should exist");
    assert_eq!(intent_row.trace_count, 2);
    assert_eq!(intent_row.feedback_count, 2);
    assert_eq!(intent_row.feedback_trace_count, 1);
    assert_eq!(intent_row.feedback_coverage, 0.5);
    assert_eq!(intent_row.feedback_records_per_trace, 1.0);
    assert_eq!(intent_row.outcome_feedback_count, 1);
    assert_eq!(intent_row.outcome_trace_count, 1);
    assert_eq!(intent_row.outcome_coverage, 0.5);

    let arm_row = report
        .arms
        .iter()
        .find(|row| row.arm == "memoryitem_orient")
        .expect("memoryitem_orient arm row should exist");
    assert_eq!(arm_row.trace_count, 2);
    assert_eq!(arm_row.feedback_count, 2);
    assert_eq!(arm_row.feedback_trace_count, 1);
    assert_eq!(arm_row.feedback_coverage, 0.5);
    assert_eq!(arm_row.feedback_records_per_trace, 1.0);
    assert_eq!(arm_row.outcome_feedback_count, 1);
    assert_eq!(arm_row.outcome_trace_count, 1);
    assert_eq!(arm_row.outcome_coverage, 0.5);

    assert!(report
        .recommendations
        .iter()
        .any(|recommendation| recommendation.contains("feedback_records_per_trace")));
}

#[tokio::test]
async fn real_session_eval_report_tracks_memory_judgment_attribution_gaps() {
    let (telemetry, _) = setup_services().await;
    let scenario_id = "bounded_autonomous_followthrough_006";
    let arm = "memoryitem_orient";
    let judged_memory_id = engram_core::Id::new();
    let unjudged_memory_id = engram_core::Id::new();
    let out_of_scope_memory_id = engram_core::Id::new();

    let judged_trace = telemetry
        .record_trace(
            BrainHarnessTrace::new(BrainHarnessOperation::Orient)
                .with_intent(Some(BrainHarnessIntent::ImplementChange))
                .with_project(Some("engram".to_string()))
                .with_scenario_id(Some(scenario_id.to_string()))
                .with_arm(Some(arm.to_string()))
                .with_returned_memory_ids(vec![judged_memory_id]),
        )
        .await
        .expect("judged trace should be recorded");
    let unjudged_trace = telemetry
        .record_trace(
            BrainHarnessTrace::new(BrainHarnessOperation::Orient)
                .with_intent(Some(BrainHarnessIntent::ImplementChange))
                .with_project(Some("engram".to_string()))
                .with_scenario_id(Some(scenario_id.to_string()))
                .with_arm(Some(arm.to_string()))
                .with_returned_memory_ids(vec![unjudged_memory_id]),
        )
        .await
        .expect("unjudged trace should be recorded");
    let out_of_scope_trace = telemetry
        .record_trace(
            BrainHarnessTrace::new(BrainHarnessOperation::Orient)
                .with_intent(Some(BrainHarnessIntent::ImplementChange))
                .with_project(Some("engram-other".to_string()))
                .with_scenario_id(Some(scenario_id.to_string()))
                .with_arm(Some(arm.to_string()))
                .with_returned_memory_ids(vec![out_of_scope_memory_id]),
        )
        .await
        .expect("out-of-scope trace should be recorded");
    let result_only_trace = telemetry
        .record_trace(
            BrainHarnessTrace::new(BrainHarnessOperation::Search)
                .with_intent(Some(BrainHarnessIntent::ImplementChange))
                .with_project(Some("engram".to_string()))
                .with_scenario_id(Some(scenario_id.to_string()))
                .with_arm(Some(arm.to_string()))
                .with_returned_result_ids(vec!["result-only".to_string()]),
        )
        .await
        .expect("result-only trace should be recorded");

    let mut judged_feedback = AgentFeedback::new(judged_trace.id);
    judged_feedback.rejected_memory_ids = vec![judged_memory_id];
    telemetry
        .submit_feedback(judged_feedback)
        .await
        .expect("judged feedback should be accepted");

    let mut unjudged_feedback = AgentFeedback::new(unjudged_trace.id);
    unjudged_feedback.task_success = Some(true);
    telemetry
        .submit_feedback(unjudged_feedback)
        .await
        .expect("unjudged feedback should be accepted");

    let mut out_of_scope_feedback = AgentFeedback::new(out_of_scope_trace.id);
    out_of_scope_feedback.task_success = Some(false);
    telemetry
        .submit_feedback(out_of_scope_feedback)
        .await
        .expect("out-of-scope feedback should be accepted");

    let mut result_only_feedback = AgentFeedback::new(result_only_trace.id);
    result_only_feedback.used_result_ids = vec!["result-only".to_string()];
    telemetry
        .submit_feedback(result_only_feedback)
        .await
        .expect("result-only feedback should be accepted");

    let report = telemetry
        .real_session_eval_report(Some(100))
        .await
        .expect("real-session report should build");
    assert_eq!(report.feedback_count, 4);
    assert_eq!(report.memory_judgment_feedback_count, 1);
    assert_eq!(report.memory_judgment_coverage, 0.25);
    assert_eq!(report.unjudged_memory_feedback_count, 2);
    assert!(report
        .recommendations
        .iter()
        .any(|recommendation| recommendation.contains("memory attribution fields")));

    let scoped_report = telemetry
        .real_session_eval_report_scoped(Some(100), Some("engram"), Some(scenario_id), Some(arm))
        .await
        .expect("scoped real-session report should build");
    assert_eq!(scoped_report.trace_count, 3);
    assert_eq!(scoped_report.feedback_count, 3);
    assert_eq!(scoped_report.memory_judgment_feedback_count, 1);
    assert!((scoped_report.memory_judgment_coverage - (1.0_f32 / 3.0)).abs() < f32::EPSILON);
    assert_eq!(scoped_report.unjudged_memory_feedback_count, 1);
}

#[tokio::test]
async fn outcome_only_feedback_is_a_concrete_signal() {
    let (telemetry, _) = setup_services().await;
    let trace = telemetry
        .record_trace(
            BrainHarnessTrace::new(BrainHarnessOperation::Orient)
                .with_query(Some(
                    "check whether outcome feedback is accepted".to_string(),
                ))
                .with_intent(Some(BrainHarnessIntent::VerifyDecision)),
        )
        .await
        .expect("trace should be recorded");

    let mut feedback = AgentFeedback::new(trace.id);
    feedback.task_success = Some(false);
    feedback.preference_adhered = Some(false);
    feedback.repeated_context_questions = Some(2);
    feedback.bad_memory_used = Some(true);

    let stored = telemetry
        .submit_feedback(feedback)
        .await
        .expect("outcome-only feedback should be accepted");
    assert_eq!(stored.task_success, Some(false));
    assert_eq!(stored.preference_adhered, Some(false));
    assert_eq!(stored.repeated_context_questions, Some(2));
    assert_eq!(stored.bad_memory_used, Some(true));
}

#[tokio::test]
async fn orient_with_intent_emits_trace_for_agent_feedback() {
    let (telemetry, memory) = setup_services().await;
    let preference = MemoryItem::new(
        MemoryKind::Preference,
        "Prefer intent-aware telemetry",
        "Correlate retrieval feedback with the agent intent.",
        MemoryScope::project("engram"),
        ClaimOrigin::UserStated,
        writer(),
    );
    let preference = memory
        .capture_memory(preference)
        .await
        .expect("memory should be captured");

    let packet = memory
        .orient(OrientInput {
            cwd: Some("/Users/yuval.meiri/projects/engram".to_string()),
            prompt: Some("continue telemetry implementation".to_string()),
            project: Some("engram".to_string()),
            agent: Some("codex".to_string()),
            external_session_id: Some("codex://threads/orient-service-test".to_string()),
            intent: Some(BrainHarnessIntent::ImplementChange),
            scenario_id: Some("telemetry_implementation".to_string()),
            arm: Some("memory_items".to_string()),
            include_recent_commits: false,
            limit: Some(10),
        })
        .await
        .expect("orient should return a packet");

    let trace_id = packet.trace_id.expect("orient should return trace_id");
    let trace = telemetry
        .get_trace(&trace_id)
        .await
        .expect("trace lookup should run")
        .expect("trace should exist");

    assert_eq!(packet.intent, Some(BrainHarnessIntent::ImplementChange));
    assert_eq!(trace.operation, BrainHarnessOperation::Orient);
    assert_eq!(
        trace.external_session_id.as_deref(),
        Some("codex://threads/orient-service-test")
    );
    assert_eq!(trace.intent, Some(BrainHarnessIntent::ImplementChange));
    assert_eq!(
        trace.scenario_id.as_deref(),
        Some("telemetry_implementation")
    );
    assert_eq!(trace.arm.as_deref(), Some("memory_items"));
    assert_eq!(trace.returned_memory_ids, vec![preference.id]);
    assert_eq!(trace.project.as_deref(), Some("engram"));
}

#[tokio::test]
async fn changes_since_with_intent_emits_trace_for_agent_feedback() {
    let (telemetry, memory) = setup_services().await;
    let cursor = memory
        .current_cursor()
        .await
        .expect("cursor should be created");
    let decision = MemoryItem::new(
        MemoryKind::Decision,
        "Track changes_since telemetry",
        "changes_since should produce a trace for agent feedback.",
        MemoryScope::project("engram"),
        ClaimOrigin::UserStated,
        writer(),
    );
    let decision = memory
        .capture_memory(decision)
        .await
        .expect("memory should be captured");

    let changes = memory
        .changes_since_with_options(
            cursor,
            Some(10),
            MemoryChangesSinceOptions {
                project: Some("engram".to_string()),
                query: Some("telemetry trace".to_string()),
                intent: Some(BrainHarnessIntent::ReviewMemory),
                external_session_id: Some("codex://threads/changes-test".to_string()),
                ..Default::default()
            },
        )
        .await
        .expect("changes_since should work");

    let trace_id = changes
        .trace_id
        .expect("changes_since should return trace_id");
    let trace = telemetry
        .get_trace(&trace_id)
        .await
        .expect("trace lookup should run")
        .expect("trace should exist");

    assert_eq!(trace.operation, BrainHarnessOperation::ChangesSince);
    assert_eq!(
        trace.external_session_id.as_deref(),
        Some("codex://threads/changes-test")
    );
    assert_eq!(trace.intent, Some(BrainHarnessIntent::ReviewMemory));
    assert_eq!(trace.returned_memory_ids, vec![decision.id]);
}

#[tokio::test]
async fn mcp_telemetry_tool_records_trace_feedback_and_stats() {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config)
        .await
        .expect("failed to connect to in-memory store");
    let telemetry = TelemetryService::new(db);
    telemetry
        .init_schema()
        .await
        .expect("failed to initialize telemetry schema");
    let state = ToolState::new();
    state.init_telemetry(telemetry).await;

    let mut trace_request = telemetry_request("record_trace");
    trace_request.operation = Some("search".to_string());
    trace_request.intent = Some("answer_question".to_string());
    trace_request.scenario_id = Some("question_answering".to_string());
    trace_request.arm = Some("memory_items".to_string());
    trace_request.query = Some("how does telemetry work?".to_string());
    trace_request.agent = Some("codex".to_string());
    trace_request.external_session_id = Some("codex://threads/test-host-session".to_string());
    trace_request.returned_result_ids = vec!["result-1".to_string()];
    trace_request.latency_ms = Some(18);

    let trace_response = tools::telemetry_new(&state, trace_request)
        .await
        .expect("record_trace should work");
    let trace_json = parse_json(&trace_response);
    let trace_id = trace_json["trace"]["id"].as_str().unwrap().to_string();
    assert_eq!(trace_json["trace"]["scenario_id"], "question_answering");
    assert_eq!(trace_json["trace"]["arm"], "memory_items");
    assert_eq!(
        trace_json["trace"]["external_session_id"],
        "codex://threads/test-host-session"
    );

    let mut feedback_request = telemetry_request("submit_feedback");
    feedback_request.trace_id = Some(trace_id);
    feedback_request.agent = Some("codex".to_string());
    feedback_request.used_result_ids = vec!["result-1".to_string()];
    feedback_request.usefulness_score = Some(4);
    feedback_request.correctness_score = Some(5);
    feedback_request.noise_score = Some(1);
    feedback_request.task_success = Some(true);
    feedback_request.preference_adhered = Some(true);
    feedback_request.repeated_context_questions = Some(0);
    feedback_request.bad_memory_used = Some(false);
    feedback_request.note = Some("The result answered the question.".to_string());

    let feedback_response = tools::telemetry_new(&state, feedback_request)
        .await
        .expect("submit_feedback should work");
    let feedback_json = parse_json(&feedback_response);
    assert_eq!(feedback_json["feedback"]["used_result_ids"][0], "result-1");
    assert_eq!(
        feedback_json["feedback"]["external_session_id"],
        "codex://threads/test-host-session"
    );
    assert_eq!(feedback_json["feedback"]["task_success"], true);
    assert_eq!(feedback_json["feedback"]["preference_adhered"], true);

    let stats_response = tools::telemetry_new(&state, telemetry_request("stats_by_intent"))
        .await
        .expect("stats should work");
    let stats_json = parse_json(&stats_response);
    assert_eq!(stats_json["stats"][0]["intent"], "answer_question");
    assert_eq!(stats_json["stats"][0]["trace_count"], 1);
    assert_eq!(stats_json["stats"][0]["feedback_count"], 1);

    let report_response = tools::telemetry_new(&state, telemetry_request("real_session_eval"))
        .await
        .expect("real-session eval report should work");
    let report_json = parse_json(&report_response);
    assert_eq!(report_json["report"]["trace_count"], 1);
    assert_eq!(report_json["report"]["external_session_trace_count"], 1);
    assert_eq!(report_json["report"]["distinct_external_session_count"], 1);
    assert_eq!(
        report_json["report"]["unspecified_external_session_trace_count"],
        0
    );
    assert_eq!(report_json["report"]["feedback_count"], 1);
    assert_eq!(report_json["report"]["operation_counts"]["search"], 1);
    assert_eq!(
        report_json["report"]["scenario_counts"]["question_answering"],
        1
    );
    assert_eq!(report_json["report"]["arms"][0]["arm"], "memory_items");
    assert_eq!(report_json["report"]["outcome_feedback_count"], 1);
    assert_eq!(report_json["report"]["task_success_count"], 1);
    assert_eq!(report_json["report"]["confidence_gate"]["passed"], false);
}

#[tokio::test]
async fn mcp_telemetry_filters_traces_feedback_and_eval_by_scenario_and_arm() {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config)
        .await
        .expect("failed to connect to in-memory store");
    let telemetry = TelemetryService::new(db);
    telemetry
        .init_schema()
        .await
        .expect("failed to initialize telemetry schema");
    let state = ToolState::new();
    state.init_telemetry(telemetry).await;

    let target_scenario = "bounded_autonomous_followthrough_003";
    let target_arm = "no_memory_same_harness";
    let unrelated_scenario = "bounded_autonomous_followthrough_unrelated";
    let unrelated_arm = "orient_same_harness";
    let mut target_trace_id = String::new();

    for (label, scenario_id, arm, task_success) in [
        ("target", target_scenario, target_arm, true),
        ("unrelated-scenario", unrelated_scenario, target_arm, false),
        ("unrelated-arm", target_scenario, unrelated_arm, false),
    ] {
        let mut trace_request = telemetry_request("record_trace");
        trace_request.operation = Some("search".to_string());
        trace_request.intent = Some("implement_change".to_string());
        trace_request.scenario_id = Some(scenario_id.to_string());
        trace_request.arm = Some(arm.to_string());
        trace_request.query = Some(format!("{label} telemetry scope"));
        trace_request.returned_result_ids = vec![format!("result-{label}")];

        let trace_response = tools::telemetry_new(&state, trace_request)
            .await
            .expect("record_trace should work");
        let trace_json = parse_json(&trace_response);
        let trace_id = trace_json["trace"]["id"].as_str().unwrap().to_string();
        if label == "target" {
            target_trace_id = trace_id.clone();
        }

        let mut feedback_request = telemetry_request("submit_feedback");
        feedback_request.trace_id = Some(trace_id);
        feedback_request.used_result_ids = vec![format!("result-{label}")];
        feedback_request.task_success = Some(task_success);
        feedback_request.note = Some(format!("{label} feedback"));

        tools::telemetry_new(&state, feedback_request)
            .await
            .expect("submit_feedback should work");
    }

    let unfiltered_response = tools::telemetry_new(&state, telemetry_request("list_traces"))
        .await
        .expect("unfiltered list_traces should work");
    let unfiltered_json = parse_json(&unfiltered_response);
    assert_eq!(unfiltered_json["count"], 3);

    let mut list_traces_request = telemetry_request("list_traces");
    list_traces_request.scenario_id = Some(target_scenario.to_string());
    list_traces_request.arm = Some(target_arm.to_string());
    let list_traces_response = tools::telemetry_new(&state, list_traces_request)
        .await
        .expect("filtered list_traces should work");
    let list_traces_json = parse_json(&list_traces_response);
    let traces = list_traces_json["traces"].as_array().unwrap();
    assert_eq!(list_traces_json["count"], 1);
    assert_eq!(traces[0]["id"].as_str(), Some(target_trace_id.as_str()));
    assert_eq!(traces[0]["scenario_id"], target_scenario);
    assert_eq!(traces[0]["arm"], target_arm);

    let mut list_feedback_request = telemetry_request("list_feedback");
    list_feedback_request.scenario_id = Some(target_scenario.to_string());
    list_feedback_request.arm = Some(target_arm.to_string());
    let list_feedback_response = tools::telemetry_new(&state, list_feedback_request)
        .await
        .expect("filtered list_feedback should work");
    let list_feedback_json = parse_json(&list_feedback_response);
    let feedback = list_feedback_json["feedback"].as_array().unwrap();
    assert_eq!(list_feedback_json["count"], 1);
    assert_eq!(
        feedback[0]["trace_id"].as_str(),
        Some(target_trace_id.as_str())
    );
    assert_eq!(feedback[0]["note"], "target feedback");

    let mut eval_request = telemetry_request("real_session_eval");
    eval_request.scenario_id = Some(target_scenario.to_string());
    eval_request.arm = Some(target_arm.to_string());
    let eval_response = tools::telemetry_new(&state, eval_request)
        .await
        .expect("filtered real_session_eval should work");
    let eval_json = parse_json(&eval_response);
    let report = &eval_json["report"];
    assert_eq!(report["trace_count"], 1);
    assert_eq!(report["feedback_count"], 1);
    assert_eq!(report["task_success_count"], 1);
    assert_eq!(report["task_failure_count"], 0);
    assert_eq!(report["scenario_counts"][target_scenario], 1);
    assert!(report["scenario_counts"].get(unrelated_scenario).is_none());
    let arms = report["arms"].as_array().unwrap();
    assert_eq!(arms.len(), 1);
    assert_eq!(arms[0]["arm"], target_arm);
    assert_ne!(arms[0]["arm"], unrelated_arm);
}

#[tokio::test]
async fn mcp_telemetry_list_actions_filter_by_project() {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config)
        .await
        .expect("failed to connect to in-memory store");
    let telemetry = TelemetryService::new(db);
    telemetry
        .init_schema()
        .await
        .expect("failed to initialize telemetry schema");
    let state = ToolState::new();
    state.init_telemetry(telemetry).await;

    let scenario_id = "bounded_autonomous_followthrough_005";
    let arm = "memoryitem_orient";
    let mut target_trace_id = String::new();

    for (label, project) in [("target", "engram"), ("other-project", "engram-other")] {
        let mut trace_request = telemetry_request("record_trace");
        trace_request.operation = Some("orient".to_string());
        trace_request.intent = Some("implement_change".to_string());
        trace_request.project = Some(project.to_string());
        trace_request.scenario_id = Some(scenario_id.to_string());
        trace_request.arm = Some(arm.to_string());
        trace_request.query = Some(format!("{label} project-scoped telemetry"));

        let trace_response = tools::telemetry_new(&state, trace_request)
            .await
            .expect("record_trace should work");
        let trace_json = parse_json(&trace_response);
        let trace_id = trace_json["trace"]["id"].as_str().unwrap().to_string();
        if label == "target" {
            target_trace_id = trace_id.clone();
        }

        let mut feedback_request = telemetry_request("submit_feedback");
        feedback_request.trace_id = Some(trace_id);
        feedback_request.task_success = Some(project == "engram");

        tools::telemetry_new(&state, feedback_request)
            .await
            .expect("submit_feedback should work");
    }

    let mut list_traces_request = telemetry_request("list_traces");
    list_traces_request.project = Some("engram".to_string());
    list_traces_request.scenario_id = Some(scenario_id.to_string());
    list_traces_request.arm = Some(arm.to_string());
    let list_traces_response = tools::telemetry_new(&state, list_traces_request)
        .await
        .expect("project-filtered list_traces should work");
    let list_traces_json = parse_json(&list_traces_response);
    let traces = list_traces_json["traces"].as_array().unwrap();
    assert_eq!(list_traces_json["count"], 1);
    assert_eq!(traces[0]["id"].as_str(), Some(target_trace_id.as_str()));
    assert_eq!(traces[0]["project"], "engram");

    let mut list_feedback_request = telemetry_request("list_feedback");
    list_feedback_request.project = Some("engram".to_string());
    list_feedback_request.scenario_id = Some(scenario_id.to_string());
    list_feedback_request.arm = Some(arm.to_string());
    let list_feedback_response = tools::telemetry_new(&state, list_feedback_request)
        .await
        .expect("project-filtered list_feedback should work");
    let list_feedback_json = parse_json(&list_feedback_response);
    let feedback = list_feedback_json["feedback"].as_array().unwrap();
    assert_eq!(list_feedback_json["count"], 1);
    assert_eq!(
        feedback[0]["trace_id"].as_str(),
        Some(target_trace_id.as_str())
    );
}

#[tokio::test]
async fn mcp_real_session_eval_reports_applied_filters() {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config)
        .await
        .expect("failed to connect to in-memory store");
    let telemetry = TelemetryService::new(db);
    telemetry
        .init_schema()
        .await
        .expect("failed to initialize telemetry schema");
    let state = ToolState::new();
    state.init_telemetry(telemetry).await;

    let target_project = "engram";
    let scenario_id = "bounded_autonomous_followthrough_004";
    let arm = "memoryitem_orient";

    for (label, project, arm, task_success) in [
        ("target", target_project, arm, true),
        ("other-project", "engram-other", arm, false),
        ("other-arm", target_project, "no_memory_same_harness", false),
    ] {
        let mut trace_request = telemetry_request("record_trace");
        trace_request.operation = Some("orient".to_string());
        trace_request.intent = Some("implement_change".to_string());
        trace_request.project = Some(project.to_string());
        trace_request.scenario_id = Some(scenario_id.to_string());
        trace_request.arm = Some(arm.to_string());
        trace_request.query = Some(format!("{label} eval report"));

        let trace_response = tools::telemetry_new(&state, trace_request)
            .await
            .expect("record_trace should work");
        let trace_json = parse_json(&trace_response);
        let trace_id = trace_json["trace"]["id"].as_str().unwrap().to_string();

        let mut feedback_request = telemetry_request("submit_feedback");
        feedback_request.trace_id = Some(trace_id);
        feedback_request.task_success = Some(task_success);

        tools::telemetry_new(&state, feedback_request)
            .await
            .expect("submit_feedback should work");
    }

    let unfiltered_response = tools::telemetry_new(&state, telemetry_request("real_session_eval"))
        .await
        .expect("unfiltered real_session_eval should work");
    let unfiltered_json = parse_json(&unfiltered_response);
    let unfiltered_report = &unfiltered_json["report"];
    assert_eq!(unfiltered_report["trace_count"], 3);
    assert!(unfiltered_report["applied_filters"]["project"].is_null());
    assert!(unfiltered_report["applied_filters"]["scenario_id"].is_null());
    assert!(unfiltered_report["applied_filters"]["arm"].is_null());

    let mut filtered_request = telemetry_request("real_session_eval");
    filtered_request.project = Some(target_project.to_string());
    filtered_request.scenario_id = Some(scenario_id.to_string());
    filtered_request.arm = Some(arm.to_string());

    let filtered_response = tools::telemetry_new(&state, filtered_request)
        .await
        .expect("filtered real_session_eval should work");
    let filtered_json = parse_json(&filtered_response);
    let filtered_report = &filtered_json["report"];
    assert_eq!(filtered_report["trace_count"], 1);
    assert_eq!(filtered_report["feedback_count"], 1);
    assert_eq!(filtered_report["task_success_count"], 1);
    assert_eq!(filtered_report["task_failure_count"], 0);
    assert_eq!(
        filtered_report["applied_filters"]["project"],
        target_project
    );
    assert_eq!(
        filtered_report["applied_filters"]["scenario_id"],
        scenario_id
    );
    assert_eq!(filtered_report["applied_filters"]["arm"], arm);
}

#[tokio::test]
async fn mcp_orient_tags_trace_with_scenario_and_arm() {
    let (telemetry, memory) = setup_services().await;
    memory
        .capture_memory(MemoryItem::new(
            MemoryKind::Decision,
            "Tag orient traces by eval arm",
            "Controlled evals need scenario and arm on the real orient trace.",
            MemoryScope::project("engram"),
            ClaimOrigin::UserStated,
            writer(),
        ))
        .await
        .expect("memory should be captured");

    let state = ToolState::new();
    state.init_memory(memory).await;
    state.init_telemetry(telemetry.clone()).await;

    let response = tools::orient(
        &state,
        OrientRequest {
            cwd: Some("/Users/yuval.meiri/projects/engram".to_string()),
            prompt: Some("continue controlled telemetry eval".to_string()),
            project: Some("engram".to_string()),
            agent: Some("codex".to_string()),
            external_session_id: Some("codex://threads/orient-test".to_string()),
            intent: Some("verify_decision".to_string()),
            scenario_id: Some("controlled_eval_tagging".to_string()),
            arm: Some("memory_items".to_string()),
            include_recent_commits: Some(false),
            limit: Some(10),
            response_shape: None,
        },
    )
    .await
    .expect("orient should work");

    let json = parse_json(&response);
    let trace_id = json["trace_id"].as_str().expect("trace_id should be set");
    let trace = telemetry
        .get_trace(&engram_core::Id::parse(trace_id).unwrap())
        .await
        .expect("trace lookup should run")
        .expect("trace should exist");

    assert_eq!(trace.operation, BrainHarnessOperation::Orient);
    assert_eq!(
        trace.external_session_id.as_deref(),
        Some("codex://threads/orient-test")
    );
    assert_eq!(trace.intent, Some(BrainHarnessIntent::VerifyDecision));
    assert_eq!(
        trace.scenario_id.as_deref(),
        Some("controlled_eval_tagging")
    );
    assert_eq!(trace.arm.as_deref(), Some("memory_items"));
}

#[tokio::test]
async fn mcp_search_returns_trace_id_when_telemetry_is_initialized() {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config)
        .await
        .expect("failed to connect to in-memory store");
    let telemetry = TelemetryService::new(db.clone());
    telemetry
        .init_schema()
        .await
        .expect("failed to initialize telemetry schema");
    let memory = MemoryService::new(db.clone());
    memory
        .init_schema()
        .await
        .expect("failed to initialize memory schema");
    let memory_item = memory
        .capture_memory(
            MemoryItem::new(
                MemoryKind::Decision,
                "Intent aware telemetry",
                "MCP search should expose MemoryItem trust metadata.",
                MemoryScope::project("engram"),
                ClaimOrigin::UserStated,
                writer(),
            )
            .with_evidence(EvidenceRef::new(
                EvidenceKind::ManualReview,
                "telemetry-test",
            )),
        )
        .await
        .expect("memory should be captured");
    let state = ToolState::new();
    state.init_search(SearchService::new(db)).await;
    state.init_telemetry(telemetry.clone()).await;

    let response = tools::search(
        &state,
        SearchRequest {
            query: "intent aware telemetry".to_string(),
            limit: 5,
            min_score: Some(0.0),
            layers: None,
            intent: Some("answer_question".to_string()),
            scenario_id: Some("memory_search_relevance".to_string()),
            arm: Some("memory_items".to_string()),
            agent: Some("codex".to_string()),
            session_id: None,
            external_session_id: Some("codex://threads/search-test".to_string()),
            project: Some("engram".to_string()),
            cwd: None,
        },
    )
    .await
    .expect("search should work");

    let json = parse_json(&response);
    let memory_result = json["results"]
        .as_array()
        .unwrap()
        .iter()
        .find(|result| result["id"] == memory_item.id.to_string())
        .expect("memory result should be returned");
    assert_eq!(memory_result["source"], "memory");
    assert_eq!(
        memory_result["memory_metadata"]["memory_id"],
        memory_item.id.to_string()
    );
    assert_eq!(memory_result["memory_metadata"]["review_state"], "reviewed");
    assert_eq!(
        memory_result["memory_metadata"]["writer"]["harness"],
        "codex"
    );
    let trace_id = json["trace_id"].as_str().expect("trace_id should be set");
    let trace = telemetry
        .get_trace(&engram_core::Id::parse(trace_id).unwrap())
        .await
        .expect("trace lookup should run")
        .expect("trace should exist");

    assert_eq!(trace.operation, BrainHarnessOperation::Search);
    assert_eq!(
        trace.external_session_id.as_deref(),
        Some("codex://threads/search-test")
    );
    assert_eq!(trace.intent, Some(BrainHarnessIntent::AnswerQuestion));
    assert_eq!(
        trace.scenario_id.as_deref(),
        Some("memory_search_relevance")
    );
    assert_eq!(trace.arm.as_deref(), Some("memory_items"));
    assert_eq!(trace.query.as_deref(), Some("intent aware telemetry"));
    assert_eq!(trace.project.as_deref(), Some("engram"));
}

#[tokio::test]
async fn mcp_submit_feedback_warns_when_returned_memory_is_unattributed() {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config)
        .await
        .expect("failed to connect to in-memory store");
    let telemetry = TelemetryService::new(db);
    telemetry
        .init_schema()
        .await
        .expect("failed to initialize telemetry schema");
    let state = ToolState::new();
    state.init_telemetry(telemetry).await;

    let returned_memory_id = engram_core::Id::new();
    let mut trace_request = telemetry_request("record_trace");
    trace_request.operation = Some("orient".to_string());
    trace_request.intent = Some("implement_change".to_string());
    trace_request.query = Some("recover sealed target".to_string());
    trace_request.returned_memory_ids = vec![returned_memory_id.to_string()];

    let trace_response = tools::telemetry_new(&state, trace_request)
        .await
        .expect("record_trace should work");
    let trace_json = parse_json(&trace_response);
    let trace_id = trace_json["trace"]["id"].as_str().unwrap().to_string();

    let mut feedback_request = telemetry_request("submit_feedback");
    feedback_request.trace_id = Some(trace_id.clone());
    feedback_request.task_success = Some(true);
    feedback_request.note = Some("forgot to attribute used memory".to_string());

    let feedback_response = tools::telemetry_new(&state, feedback_request)
        .await
        .expect("submit_feedback should work");
    let feedback_json = parse_json(&feedback_response);
    let warnings = feedback_json["warnings"]
        .as_array()
        .expect("warnings array should be present");
    assert_eq!(warnings.len(), 1);
    let warning = warnings[0].as_str().unwrap();
    assert!(
        warning.contains("used_memory_ids"),
        "warning should reference used_memory_ids: {warning}"
    );
    assert!(
        warning.contains("returned memory"),
        "warning should describe the linked trace situation: {warning}"
    );
    assert_eq!(feedback_json["feedback"]["trace_id"], trace_id);

    let mut attributed_request = telemetry_request("submit_feedback");
    attributed_request.trace_id = Some(trace_id);
    attributed_request.used_memory_ids = vec![returned_memory_id.to_string()];
    attributed_request.task_success = Some(true);

    let attributed_response = tools::telemetry_new(&state, attributed_request)
        .await
        .expect("attributed submit_feedback should work");
    let attributed_json = parse_json(&attributed_response);
    assert_eq!(
        attributed_json["warnings"].as_array().unwrap().len(),
        0,
        "no warning when used_memory_ids is populated"
    );
}

#[tokio::test]
async fn mcp_submit_feedback_skips_warning_when_trace_returned_no_memory() {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config)
        .await
        .expect("failed to connect to in-memory store");
    let telemetry = TelemetryService::new(db);
    telemetry
        .init_schema()
        .await
        .expect("failed to initialize telemetry schema");
    let state = ToolState::new();
    state.init_telemetry(telemetry).await;

    let mut trace_request = telemetry_request("record_trace");
    trace_request.operation = Some("search".to_string());
    trace_request.intent = Some("answer_question".to_string());
    trace_request.query = Some("no memory returned".to_string());
    trace_request.returned_result_ids = vec!["result-1".to_string()];

    let trace_response = tools::telemetry_new(&state, trace_request)
        .await
        .expect("record_trace should work");
    let trace_json = parse_json(&trace_response);
    let trace_id = trace_json["trace"]["id"].as_str().unwrap().to_string();

    let mut feedback_request = telemetry_request("submit_feedback");
    feedback_request.trace_id = Some(trace_id);
    feedback_request.used_result_ids = vec!["result-1".to_string()];
    feedback_request.task_success = Some(true);

    let feedback_response = tools::telemetry_new(&state, feedback_request)
        .await
        .expect("submit_feedback should work");
    let feedback_json = parse_json(&feedback_response);
    assert_eq!(
        feedback_json["warnings"].as_array().unwrap().len(),
        0,
        "no warning when linked trace returned no memory IDs"
    );
}

#[tokio::test]
async fn mcp_submit_feedback_skips_warning_when_only_rejected_memory_is_set() {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config)
        .await
        .expect("failed to connect to in-memory store");
    let telemetry = TelemetryService::new(db);
    telemetry
        .init_schema()
        .await
        .expect("failed to initialize telemetry schema");
    let state = ToolState::new();
    state.init_telemetry(telemetry).await;

    let returned_memory_id = engram_core::Id::new();
    let mut trace_request = telemetry_request("record_trace");
    trace_request.operation = Some("orient".to_string());
    trace_request.intent = Some("implement_change".to_string());
    trace_request.query = Some("memory judged not useful".to_string());
    trace_request.returned_memory_ids = vec![returned_memory_id.to_string()];

    let trace_response = tools::telemetry_new(&state, trace_request)
        .await
        .expect("record_trace should work");
    let trace_json = parse_json(&trace_response);
    let trace_id = trace_json["trace"]["id"].as_str().unwrap().to_string();

    let mut feedback_request = telemetry_request("submit_feedback");
    feedback_request.trace_id = Some(trace_id);
    feedback_request.rejected_memory_ids = vec![returned_memory_id.to_string()];
    feedback_request.task_success = Some(true);

    let feedback_response = tools::telemetry_new(&state, feedback_request)
        .await
        .expect("submit_feedback should work");
    let feedback_json = parse_json(&feedback_response);
    assert_eq!(
        feedback_json["warnings"].as_array().unwrap().len(),
        0,
        "no warning when rejected_memory_ids is populated"
    );
}
