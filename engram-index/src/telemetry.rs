//! Brain harness telemetry service.

use crate::error::{IndexError, IndexResult};
use engram_core::id::Id;
use engram_core::telemetry::{
    AgentFeedback, BrainHarnessTrace, IntentTelemetryStats, RealSessionConfidenceGate,
    RealSessionEvalAppliedFilters, RealSessionEvalArmRow, RealSessionEvalIntentRow,
    RealSessionEvalReport,
};
use engram_store::{Db, TelemetryRepo};
use std::collections::{BTreeMap, HashMap, HashSet};
use time::OffsetDateTime;

const DEFAULT_REAL_SESSION_EVAL_LIMIT: usize = 10_000;
const MIN_REAL_SESSION_TRACES: usize = 20;
const MIN_REAL_SESSION_FEEDBACK: usize = 10;
const MIN_REAL_SESSION_FEEDBACK_COVERAGE: f32 = 0.5;
const MIN_REAL_SESSION_INTENTS_WITH_FEEDBACK: usize = 3;
const MIN_REAL_SESSION_OUTCOME_FEEDBACK: usize = 1;

/// Service for retrieval traces and agent feedback.
#[derive(Clone)]
pub struct TelemetryService {
    repo: TelemetryRepo,
}

impl TelemetryService {
    /// Create a new telemetry service.
    pub fn new(db: Db) -> Self {
        Self {
            repo: TelemetryRepo::new(db),
        }
    }

    /// Initialize telemetry schema.
    pub async fn init_schema(&self) -> IndexResult<()> {
        self.repo.init_schema().await?;
        Ok(())
    }

    /// Record a brain-harness operation trace.
    pub async fn record_trace(&self, trace: BrainHarnessTrace) -> IndexResult<BrainHarnessTrace> {
        validate_trace(&trace)?;
        self.repo.save_trace(&trace).await?;
        Ok(trace)
    }

    /// Get a trace by ID.
    pub async fn get_trace(&self, id: &Id) -> IndexResult<Option<BrainHarnessTrace>> {
        Ok(self.repo.get_trace(id).await?)
    }

    /// List traces.
    pub async fn list_traces(&self, limit: Option<usize>) -> IndexResult<Vec<BrainHarnessTrace>> {
        Ok(self.repo.list_traces(limit).await?)
    }

    /// List traces scoped by optional project, scenario, and arm filters.
    pub async fn list_traces_scoped(
        &self,
        limit: Option<usize>,
        project: Option<&str>,
        scenario_id: Option<&str>,
        arm: Option<&str>,
    ) -> IndexResult<Vec<BrainHarnessTrace>> {
        let applied_filters = applied_filters(project, scenario_id, arm);
        if !has_any_filter(&applied_filters) {
            return self.list_traces(limit).await;
        }

        Ok(self
            .repo
            .list_traces_scoped(
                limit,
                applied_filters.project.as_deref(),
                applied_filters.scenario_id.as_deref(),
                applied_filters.arm.as_deref(),
            )
            .await?)
    }

    /// Submit agent feedback for a trace.
    pub async fn submit_feedback(&self, mut feedback: AgentFeedback) -> IndexResult<AgentFeedback> {
        let trace = self
            .repo
            .get_trace(&feedback.trace_id)
            .await?
            .ok_or_else(|| {
                IndexError::NotFound(format!("trace not found: {}", feedback.trace_id))
            })?;
        if feedback.external_session_id.is_none() {
            feedback.external_session_id = trace.external_session_id;
        }
        validate_feedback(&feedback)?;

        self.repo.save_feedback(&feedback).await?;
        Ok(feedback)
    }

    /// Get feedback by ID.
    pub async fn get_feedback(&self, id: &Id) -> IndexResult<Option<AgentFeedback>> {
        Ok(self.repo.get_feedback(id).await?)
    }

    /// List feedback for a trace.
    pub async fn list_feedback_for_trace(&self, trace_id: &Id) -> IndexResult<Vec<AgentFeedback>> {
        Ok(self.repo.list_feedback_for_trace(trace_id).await?)
    }

    /// List recent feedback.
    pub async fn list_feedback(&self, limit: Option<usize>) -> IndexResult<Vec<AgentFeedback>> {
        Ok(self.repo.list_feedback(limit).await?)
    }

    /// List recent feedback linked to traces matching optional project, scenario, and arm filters.
    pub async fn list_feedback_scoped(
        &self,
        limit: Option<usize>,
        project: Option<&str>,
        scenario_id: Option<&str>,
        arm: Option<&str>,
    ) -> IndexResult<Vec<AgentFeedback>> {
        if !has_scope_filter(project, scenario_id, arm) {
            return self.list_feedback(limit).await;
        }

        let traces = self.repo.list_traces(limit).await?;
        let traces = filter_traces_by_scope(traces, project, scenario_id, arm);
        let trace_ids = traces.iter().map(|trace| trace.id).collect::<HashSet<_>>();
        let feedback = self.repo.list_feedback(limit).await?;

        Ok(feedback
            .into_iter()
            .filter(|item| trace_ids.contains(&item.trace_id))
            .collect())
    }

    /// Aggregate traces and feedback by intent.
    pub async fn stats_by_intent(&self) -> IndexResult<Vec<IntentTelemetryStats>> {
        let traces = self.repo.list_traces(Some(10_000)).await?;
        let feedback = self.repo.list_feedback(Some(10_000)).await?;

        let trace_intents = traces
            .iter()
            .map(|trace| (trace.id, intent_key(trace)))
            .collect::<HashMap<_, _>>();

        let mut groups = HashMap::<String, IntentAggregate>::new();
        for trace in &traces {
            let key = intent_key(trace);
            let group = groups
                .entry(key.clone())
                .or_insert_with(|| IntentAggregate {
                    stats: IntentTelemetryStats {
                        intent: key,
                        ..Default::default()
                    },
                    latency_sum: 0,
                    latency_count: 0,
                    usefulness_sum: 0,
                    usefulness_count: 0,
                    correctness_sum: 0,
                    correctness_count: 0,
                    noise_sum: 0,
                    noise_count: 0,
                });
            group.stats.trace_count += 1;
            if let Some(latency) = trace.latency_ms {
                group.latency_sum += latency;
                group.latency_count += 1;
            }
        }

        for item in &feedback {
            let key = trace_intents
                .get(&item.trace_id)
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());
            let group = groups
                .entry(key.clone())
                .or_insert_with(|| IntentAggregate {
                    stats: IntentTelemetryStats {
                        intent: key,
                        ..Default::default()
                    },
                    latency_sum: 0,
                    latency_count: 0,
                    usefulness_sum: 0,
                    usefulness_count: 0,
                    correctness_sum: 0,
                    correctness_count: 0,
                    noise_sum: 0,
                    noise_count: 0,
                });

            group.stats.feedback_count += 1;
            group.stats.used_memory_count += item.used_memory_ids.len();
            group.stats.rejected_memory_count += item.rejected_memory_ids.len();
            if item
                .missing_context
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
            {
                group.stats.missing_context_count += 1;
            }
            add_score(
                item.usefulness_score,
                &mut group.usefulness_sum,
                &mut group.usefulness_count,
            );
            add_score(
                item.correctness_score,
                &mut group.correctness_sum,
                &mut group.correctness_count,
            );
            add_score(
                item.noise_score,
                &mut group.noise_sum,
                &mut group.noise_count,
            );
        }

        let mut stats = groups
            .into_values()
            .map(IntentAggregate::into_stats)
            .collect::<Vec<_>>();
        stats.sort_by(|left, right| {
            right
                .trace_count
                .cmp(&left.trace_count)
                .then_with(|| left.intent.cmp(&right.intent))
        });
        Ok(stats)
    }

    /// Build a read-only report over persisted real-session traces and feedback.
    pub async fn real_session_eval_report(
        &self,
        limit: Option<usize>,
    ) -> IndexResult<RealSessionEvalReport> {
        let sample_limit = limit.unwrap_or(DEFAULT_REAL_SESSION_EVAL_LIMIT);
        let traces = self.repo.list_traces(Some(sample_limit)).await?;
        let trace_ids = traces.iter().map(|trace| trace.id).collect::<Vec<_>>();
        let feedback = self.repo.list_feedback_for_traces(&trace_ids).await?;

        Ok(build_real_session_eval_report(
            sample_limit,
            &traces,
            &feedback,
            RealSessionEvalAppliedFilters::default(),
        ))
    }

    /// Build a read-only report over traces scoped by optional project, scenario, and arm filters.
    pub async fn real_session_eval_report_scoped(
        &self,
        limit: Option<usize>,
        project: Option<&str>,
        scenario_id: Option<&str>,
        arm: Option<&str>,
    ) -> IndexResult<RealSessionEvalReport> {
        let applied_filters = applied_filters(project, scenario_id, arm);
        if !has_any_filter(&applied_filters) {
            return self.real_session_eval_report(limit).await;
        }

        let sample_limit = limit.unwrap_or(DEFAULT_REAL_SESSION_EVAL_LIMIT);
        let traces = self
            .repo
            .list_traces_scoped(
                Some(sample_limit),
                applied_filters.project.as_deref(),
                applied_filters.scenario_id.as_deref(),
                applied_filters.arm.as_deref(),
            )
            .await?;
        let trace_ids = traces.iter().map(|trace| trace.id).collect::<Vec<_>>();
        let feedback = self.repo.list_feedback_for_traces(&trace_ids).await?;

        Ok(build_real_session_eval_report(
            sample_limit,
            &traces,
            &feedback,
            applied_filters,
        ))
    }
}

#[derive(Debug)]
struct IntentAggregate {
    stats: IntentTelemetryStats,
    latency_sum: u64,
    latency_count: usize,
    usefulness_sum: u32,
    usefulness_count: usize,
    correctness_sum: u32,
    correctness_count: usize,
    noise_sum: u32,
    noise_count: usize,
}

impl IntentAggregate {
    fn into_stats(mut self) -> IntentTelemetryStats {
        self.stats.avg_latency_ms = average_u64(self.latency_sum, self.latency_count);
        self.stats.avg_usefulness_score = average_u32(self.usefulness_sum, self.usefulness_count);
        self.stats.avg_correctness_score =
            average_u32(self.correctness_sum, self.correctness_count);
        self.stats.avg_noise_score = average_u32(self.noise_sum, self.noise_count);
        self.stats
    }
}

#[derive(Debug)]
struct RealSessionEvalAggregate {
    row: RealSessionEvalIntentRow,
    feedback_trace_ids: HashSet<Id>,
    outcome_trace_ids: HashSet<Id>,
    latency_sum: u64,
    latency_count: usize,
    usefulness_sum: u32,
    usefulness_count: usize,
    correctness_sum: u32,
    correctness_count: usize,
    noise_sum: u32,
    noise_count: usize,
}

impl RealSessionEvalAggregate {
    fn new(intent: String) -> Self {
        Self {
            row: RealSessionEvalIntentRow {
                intent,
                ..Default::default()
            },
            feedback_trace_ids: HashSet::new(),
            outcome_trace_ids: HashSet::new(),
            latency_sum: 0,
            latency_count: 0,
            usefulness_sum: 0,
            usefulness_count: 0,
            correctness_sum: 0,
            correctness_count: 0,
            noise_sum: 0,
            noise_count: 0,
        }
    }

    fn add_trace(&mut self, trace: &BrainHarnessTrace) {
        self.row.trace_count += 1;
        self.row.warning_count += trace.warnings.len();
        self.row.returned_memory_count += trace.returned_memory_ids.len();
        self.row.returned_result_count += trace.returned_result_ids.len();
        if let Some(latency) = trace.latency_ms {
            self.latency_sum += latency;
            self.latency_count += 1;
        }
    }

    fn add_feedback(&mut self, feedback: &AgentFeedback) {
        self.row.feedback_count += 1;
        self.feedback_trace_ids.insert(feedback.trace_id);
        self.row.used_memory_count += feedback.used_memory_ids.len();
        self.row.rejected_memory_count += feedback.rejected_memory_ids.len();
        self.row.stale_memory_count += feedback.stale_memory_ids.len();
        self.row.wrong_scope_memory_count += feedback.wrong_scope_memory_ids.len();
        self.row.used_result_count += feedback.used_result_ids.len();
        self.row.rejected_result_count += feedback.rejected_result_ids.len();

        if has_text(feedback.missing_context.as_deref()) {
            self.row.missing_context_count += 1;
        }
        if has_text(feedback.suggested_memory_changes.as_deref()) {
            self.row.suggested_change_count += 1;
        }
        if feedback.usefulness_score.is_some()
            || feedback.correctness_score.is_some()
            || feedback.noise_score.is_some()
        {
            self.row.scored_feedback_count += 1;
        }
        if has_outcome_feedback(feedback) {
            self.outcome_trace_ids.insert(feedback.trace_id);
        }
        add_outcome_counts_to_intent_row(&mut self.row, feedback);

        add_score(
            feedback.usefulness_score,
            &mut self.usefulness_sum,
            &mut self.usefulness_count,
        );
        add_score(
            feedback.correctness_score,
            &mut self.correctness_sum,
            &mut self.correctness_count,
        );
        add_score(
            feedback.noise_score,
            &mut self.noise_sum,
            &mut self.noise_count,
        );
    }

    fn into_row(mut self) -> RealSessionEvalIntentRow {
        self.row.feedback_trace_count = self.feedback_trace_ids.len();
        self.row.feedback_coverage = coverage(self.row.feedback_trace_count, self.row.trace_count);
        self.row.feedback_records_per_trace =
            coverage(self.row.feedback_count, self.row.trace_count);
        self.row.outcome_trace_count = self.outcome_trace_ids.len();
        self.row.outcome_coverage = coverage(self.row.outcome_trace_count, self.row.trace_count);
        self.row.avg_latency_ms = average_u64(self.latency_sum, self.latency_count);
        self.row.avg_usefulness_score = average_u32(self.usefulness_sum, self.usefulness_count);
        self.row.avg_correctness_score = average_u32(self.correctness_sum, self.correctness_count);
        self.row.avg_noise_score = average_u32(self.noise_sum, self.noise_count);
        self.row
    }
}

#[derive(Debug)]
struct RealSessionEvalArmAggregate {
    row: RealSessionEvalArmRow,
    feedback_trace_ids: HashSet<Id>,
    outcome_trace_ids: HashSet<Id>,
}

impl RealSessionEvalArmAggregate {
    fn new(arm: String) -> Self {
        Self {
            row: RealSessionEvalArmRow {
                arm,
                ..Default::default()
            },
            feedback_trace_ids: HashSet::new(),
            outcome_trace_ids: HashSet::new(),
        }
    }

    fn add_trace(&mut self) {
        self.row.trace_count += 1;
    }

    fn add_feedback(&mut self, feedback: &AgentFeedback) {
        self.row.feedback_count += 1;
        self.feedback_trace_ids.insert(feedback.trace_id);
        self.row.used_memory_count += feedback.used_memory_ids.len();
        self.row.rejected_memory_count += feedback.rejected_memory_ids.len();
        self.row.stale_memory_count += feedback.stale_memory_ids.len();
        self.row.wrong_scope_memory_count += feedback.wrong_scope_memory_ids.len();
        if has_outcome_feedback(feedback) {
            self.outcome_trace_ids.insert(feedback.trace_id);
        }
        add_outcome_counts_to_arm_row(&mut self.row, feedback);
    }

    fn into_row(mut self) -> RealSessionEvalArmRow {
        self.row.feedback_trace_count = self.feedback_trace_ids.len();
        self.row.feedback_coverage = coverage(self.row.feedback_trace_count, self.row.trace_count);
        self.row.feedback_records_per_trace =
            coverage(self.row.feedback_count, self.row.trace_count);
        self.row.outcome_trace_count = self.outcome_trace_ids.len();
        self.row.outcome_coverage = coverage(self.row.outcome_trace_count, self.row.trace_count);
        self.row
    }
}

#[derive(Debug)]
struct TraceAggregation {
    groups: BTreeMap<String, RealSessionEvalAggregate>,
    arm_groups: BTreeMap<String, RealSessionEvalArmAggregate>,
    operation_counts: BTreeMap<String, usize>,
    scenario_counts: BTreeMap<String, usize>,
    unspecified_intent_trace_count: usize,
    external_session_trace_count: usize,
    distinct_external_session_count: usize,
    unspecified_external_session_trace_count: usize,
    unspecified_scenario_trace_count: usize,
    unspecified_arm_trace_count: usize,
    trace_intents: HashMap<Id, String>,
    trace_arms: HashMap<Id, String>,
}

#[derive(Debug)]
struct ReportRows {
    operation_counts: BTreeMap<String, usize>,
    scenario_counts: BTreeMap<String, usize>,
    unspecified_intent_trace_count: usize,
    external_session_trace_count: usize,
    distinct_external_session_count: usize,
    unspecified_external_session_trace_count: usize,
    unspecified_scenario_trace_count: usize,
    unspecified_arm_trace_count: usize,
    intents: Vec<RealSessionEvalIntentRow>,
    arms: Vec<RealSessionEvalArmRow>,
}

#[derive(Debug)]
struct ReportTotals {
    trace_count: usize,
    feedback_count: usize,
    feedback_trace_count: usize,
    memory_judgment_feedback_count: usize,
    memory_judgment_trace_count: usize,
    memory_judgment_eligible_trace_count: usize,
    unjudged_memory_feedback_count: usize,
    external_session_feedback: ExternalSessionFeedbackAggregation,
    outcome_trace_count: usize,
}

fn build_real_session_eval_report(
    sample_limit: usize,
    traces: &[BrainHarnessTrace],
    feedback: &[AgentFeedback],
    applied_filters: RealSessionEvalAppliedFilters,
) -> RealSessionEvalReport {
    let mut aggregation = aggregate_traces(traces);
    aggregate_feedback(
        feedback,
        &aggregation.trace_intents,
        &aggregation.trace_arms,
        &mut aggregation.groups,
        &mut aggregation.arm_groups,
    );

    let mut intents = aggregation
        .groups
        .into_values()
        .map(RealSessionEvalAggregate::into_row)
        .collect::<Vec<_>>();
    intents.sort_by(|left, right| {
        right
            .trace_count
            .cmp(&left.trace_count)
            .then_with(|| left.intent.cmp(&right.intent))
    });

    let mut arms = aggregation
        .arm_groups
        .into_values()
        .map(RealSessionEvalArmAggregate::into_row)
        .collect::<Vec<_>>();
    arms.sort_by(|left, right| {
        right
            .trace_count
            .cmp(&left.trace_count)
            .then_with(|| left.arm.cmp(&right.arm))
    });

    let rows = ReportRows {
        operation_counts: aggregation.operation_counts,
        scenario_counts: aggregation.scenario_counts,
        unspecified_intent_trace_count: aggregation.unspecified_intent_trace_count,
        external_session_trace_count: aggregation.external_session_trace_count,
        distinct_external_session_count: aggregation.distinct_external_session_count,
        unspecified_external_session_trace_count: aggregation
            .unspecified_external_session_trace_count,
        unspecified_scenario_trace_count: aggregation.unspecified_scenario_trace_count,
        unspecified_arm_trace_count: aggregation.unspecified_arm_trace_count,
        intents,
        arms,
    };
    let totals = ReportTotals {
        trace_count: traces.len(),
        feedback_count: feedback.len(),
        feedback_trace_count: count_feedback_traces(traces, feedback),
        memory_judgment_feedback_count: count_memory_judgment_feedback(feedback),
        memory_judgment_trace_count: count_memory_judgment_traces(traces, feedback),
        memory_judgment_eligible_trace_count: count_memory_judgment_eligible_traces(
            traces, feedback,
        ),
        unjudged_memory_feedback_count: count_unjudged_memory_feedback(traces, feedback),
        external_session_feedback: aggregate_feedback_external_sessions(feedback),
        outcome_trace_count: count_outcome_traces(traces, feedback),
    };
    let mut report = report_from_rows(sample_limit, totals, applied_filters, rows);
    report.confidence_gate = confidence_gate(&report);
    report.recommendations = recommendations(&report);
    report
}

fn aggregate_traces(traces: &[BrainHarnessTrace]) -> TraceAggregation {
    let mut groups = BTreeMap::<String, RealSessionEvalAggregate>::new();
    let mut arm_groups = BTreeMap::<String, RealSessionEvalArmAggregate>::new();
    let mut operation_counts = BTreeMap::<String, usize>::new();
    let mut scenario_counts = BTreeMap::<String, usize>::new();
    let mut unspecified_intent_trace_count = 0;
    let mut external_session_trace_count = 0;
    let mut unspecified_external_session_trace_count = 0;
    let mut external_session_ids = HashSet::<String>::new();
    let mut unspecified_scenario_trace_count = 0;
    let mut unspecified_arm_trace_count = 0;
    let mut trace_intents = HashMap::<Id, String>::new();
    let mut trace_arms = HashMap::<Id, String>::new();

    for trace in traces {
        let key = intent_key(trace);
        if trace.intent.is_none() {
            unspecified_intent_trace_count += 1;
        }
        if let Some(external_session_id) = normalized_label(trace.external_session_id.as_deref()) {
            external_session_trace_count += 1;
            external_session_ids.insert(external_session_id);
        } else {
            unspecified_external_session_trace_count += 1;
        }
        if let Some(scenario) = normalized_label(trace.scenario_id.as_deref()) {
            *scenario_counts.entry(scenario).or_default() += 1;
        } else {
            unspecified_scenario_trace_count += 1;
        }
        let arm_key = normalized_label(trace.arm.as_deref()).unwrap_or_else(|| {
            unspecified_arm_trace_count += 1;
            "unspecified".to_string()
        });
        *operation_counts
            .entry(trace.operation.to_string())
            .or_default() += 1;
        trace_intents.insert(trace.id, key.clone());
        trace_arms.insert(trace.id, arm_key.clone());
        groups
            .entry(key.clone())
            .or_insert_with(|| RealSessionEvalAggregate::new(key))
            .add_trace(trace);
        arm_groups
            .entry(arm_key.clone())
            .or_insert_with(|| RealSessionEvalArmAggregate::new(arm_key))
            .add_trace();
    }

    TraceAggregation {
        groups,
        arm_groups,
        operation_counts,
        scenario_counts,
        unspecified_intent_trace_count,
        external_session_trace_count,
        distinct_external_session_count: external_session_ids.len(),
        unspecified_external_session_trace_count,
        unspecified_scenario_trace_count,
        unspecified_arm_trace_count,
        trace_intents,
        trace_arms,
    }
}

fn aggregate_feedback(
    feedback: &[AgentFeedback],
    trace_intents: &HashMap<Id, String>,
    trace_arms: &HashMap<Id, String>,
    groups: &mut BTreeMap<String, RealSessionEvalAggregate>,
    arm_groups: &mut BTreeMap<String, RealSessionEvalArmAggregate>,
) {
    for item in feedback {
        let key = trace_intents
            .get(&item.trace_id)
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        groups
            .entry(key.clone())
            .or_insert_with(|| RealSessionEvalAggregate::new(key))
            .add_feedback(item);
        let arm_key = trace_arms
            .get(&item.trace_id)
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        arm_groups
            .entry(arm_key.clone())
            .or_insert_with(|| RealSessionEvalArmAggregate::new(arm_key))
            .add_feedback(item);
    }
}

fn report_from_rows(
    sample_limit: usize,
    totals: ReportTotals,
    applied_filters: RealSessionEvalAppliedFilters,
    rows: ReportRows,
) -> RealSessionEvalReport {
    RealSessionEvalReport {
        generated_at: OffsetDateTime::now_utc(),
        sample_limit,
        applied_filters,
        trace_count: totals.trace_count,
        feedback_count: totals.feedback_count,
        feedback_trace_count: totals.feedback_trace_count,
        feedback_coverage: coverage(totals.feedback_trace_count, totals.trace_count),
        feedback_records_per_trace: coverage(totals.feedback_count, totals.trace_count),
        memory_judgment_feedback_count: totals.memory_judgment_feedback_count,
        memory_judgment_coverage: coverage(
            totals.memory_judgment_feedback_count,
            totals.feedback_count,
        ),
        memory_judgment_trace_count: totals.memory_judgment_trace_count,
        memory_judgment_trace_coverage: coverage(
            totals.memory_judgment_trace_count,
            totals.memory_judgment_eligible_trace_count,
        ),
        unjudged_memory_feedback_count: totals.unjudged_memory_feedback_count,
        distinct_intent_count: rows.intents.len(),
        distinct_operation_count: rows.operation_counts.len(),
        unspecified_intent_trace_count: rows.unspecified_intent_trace_count,
        external_session_trace_count: rows.external_session_trace_count,
        distinct_external_session_count: rows.distinct_external_session_count,
        unspecified_external_session_trace_count: rows.unspecified_external_session_trace_count,
        external_session_feedback_count: totals.external_session_feedback.with_session_count,
        distinct_external_session_feedback_count: totals
            .external_session_feedback
            .distinct_session_count,
        unspecified_external_session_feedback_count: totals
            .external_session_feedback
            .without_session_count,
        operation_counts: rows.operation_counts,
        distinct_scenario_count: rows.scenario_counts.len(),
        distinct_arm_count: rows
            .arms
            .iter()
            .filter(|row| row.arm != "unspecified" && row.arm != "unknown")
            .count(),
        unspecified_scenario_trace_count: rows.unspecified_scenario_trace_count,
        unspecified_arm_trace_count: rows.unspecified_arm_trace_count,
        scenario_counts: rows.scenario_counts,
        warning_count: sum_rows(&rows.intents, |row| row.warning_count),
        returned_memory_count: sum_rows(&rows.intents, |row| row.returned_memory_count),
        returned_result_count: sum_rows(&rows.intents, |row| row.returned_result_count),
        used_memory_count: sum_rows(&rows.intents, |row| row.used_memory_count),
        rejected_memory_count: sum_rows(&rows.intents, |row| row.rejected_memory_count),
        stale_memory_count: sum_rows(&rows.intents, |row| row.stale_memory_count),
        wrong_scope_memory_count: sum_rows(&rows.intents, |row| row.wrong_scope_memory_count),
        used_result_count: sum_rows(&rows.intents, |row| row.used_result_count),
        rejected_result_count: sum_rows(&rows.intents, |row| row.rejected_result_count),
        missing_context_count: sum_rows(&rows.intents, |row| row.missing_context_count),
        suggested_change_count: sum_rows(&rows.intents, |row| row.suggested_change_count),
        scored_feedback_count: sum_rows(&rows.intents, |row| row.scored_feedback_count),
        outcome_feedback_count: sum_rows(&rows.intents, |row| row.outcome_feedback_count),
        outcome_trace_count: totals.outcome_trace_count,
        outcome_coverage: coverage(totals.outcome_trace_count, totals.trace_count),
        task_success_count: sum_rows(&rows.intents, |row| row.task_success_count),
        task_failure_count: sum_rows(&rows.intents, |row| row.task_failure_count),
        preference_adhered_count: sum_rows(&rows.intents, |row| row.preference_adhered_count),
        preference_violated_count: sum_rows(&rows.intents, |row| row.preference_violated_count),
        repeated_context_question_count: sum_rows(&rows.intents, |row| {
            row.repeated_context_question_count
        }),
        bad_memory_used_count: sum_rows(&rows.intents, |row| row.bad_memory_used_count),
        intents: rows.intents,
        arms: rows.arms,
        confidence_gate: RealSessionConfidenceGate {
            passed: false,
            min_trace_count: MIN_REAL_SESSION_TRACES,
            min_feedback_count: MIN_REAL_SESSION_FEEDBACK,
            min_feedback_coverage: MIN_REAL_SESSION_FEEDBACK_COVERAGE,
            min_intents_with_feedback: MIN_REAL_SESSION_INTENTS_WITH_FEEDBACK,
            min_outcome_feedback_count: MIN_REAL_SESSION_OUTCOME_FEEDBACK,
            requires_user_approval: true,
            reasons: Vec::new(),
        },
        recommendations: Vec::new(),
    }
}

fn confidence_gate(report: &RealSessionEvalReport) -> RealSessionConfidenceGate {
    let mut reasons = Vec::new();
    let intents_with_feedback = report
        .intents
        .iter()
        .filter(|row| row.intent != "unknown" && row.feedback_count > 0)
        .count();
    let memory_judgment_count = report.used_memory_count
        + report.rejected_memory_count
        + report.stale_memory_count
        + report.wrong_scope_memory_count;

    if report.trace_count < MIN_REAL_SESSION_TRACES {
        reasons.push(format!(
            "Need at least {MIN_REAL_SESSION_TRACES} real-session traces; found {}.",
            report.trace_count
        ));
    }
    if report.feedback_count < MIN_REAL_SESSION_FEEDBACK {
        reasons.push(format!(
            "Need at least {MIN_REAL_SESSION_FEEDBACK} agent feedback records; found {}.",
            report.feedback_count
        ));
    }
    if report.feedback_coverage < MIN_REAL_SESSION_FEEDBACK_COVERAGE {
        reasons.push(format!(
            "Need feedback coverage of at least {:.0}%; found {:.0}%.",
            MIN_REAL_SESSION_FEEDBACK_COVERAGE * 100.0,
            report.feedback_coverage * 100.0
        ));
    }
    if intents_with_feedback < MIN_REAL_SESSION_INTENTS_WITH_FEEDBACK {
        reasons.push(format!(
            "Need feedback across at least {MIN_REAL_SESSION_INTENTS_WITH_FEEDBACK} intents; \
             found {intents_with_feedback}."
        ));
    }
    if memory_judgment_count == 0 {
        reasons.push(
            "Need at least one explicit memory relevance signal: used, rejected, stale, \
             or wrong-scope memory."
                .to_string(),
        );
    }
    if report.outcome_feedback_count < MIN_REAL_SESSION_OUTCOME_FEEDBACK {
        reasons.push(format!(
            "Need at least {MIN_REAL_SESSION_OUTCOME_FEEDBACK} feedback record with behavioral \
             outcome evidence; found {}.",
            report.outcome_feedback_count
        ));
    }

    RealSessionConfidenceGate {
        passed: reasons.is_empty(),
        min_trace_count: MIN_REAL_SESSION_TRACES,
        min_feedback_count: MIN_REAL_SESSION_FEEDBACK,
        min_feedback_coverage: MIN_REAL_SESSION_FEEDBACK_COVERAGE,
        min_intents_with_feedback: MIN_REAL_SESSION_INTENTS_WITH_FEEDBACK,
        min_outcome_feedback_count: MIN_REAL_SESSION_OUTCOME_FEEDBACK,
        requires_user_approval: true,
        reasons,
    }
}

fn recommendations(report: &RealSessionEvalReport) -> Vec<String> {
    let mut recommendations = Vec::new();

    if !report.confidence_gate.passed {
        recommendations.push(
            "Keep M6 write-apply blocked until the confidence gate passes and the user \
             explicitly approves the write path."
                .to_string(),
        );
    }
    if report.trace_count == 0 {
        recommendations.push(
            "Collect real orient/search/changes_since traces before using this report for \
             architectural decisions."
                .to_string(),
        );
    }
    if report.feedback_coverage < MIN_REAL_SESSION_FEEDBACK_COVERAGE {
        recommendations.push(
            "Ask agents to submit feedback for more traces, including used/rejected memory \
             and missing_context when retrieval omits expected context."
                .to_string(),
        );
    }
    if report.feedback_records_per_trace > report.feedback_coverage {
        recommendations.push(
            "Multiple feedback records exist for at least one trace; use feedback_coverage for \
             trace coverage and feedback_records_per_trace for feedback density."
                .to_string(),
        );
    }
    if report.unspecified_intent_trace_count > 0 {
        recommendations.push(
            "Set intent on telemetry traces when the workflow phase is known; keep it as \
             secondary metadata, not confidence evidence."
                .to_string(),
        );
    }
    if report.unspecified_external_session_trace_count > 0 {
        recommendations.push(
            "Set external_session_id on telemetry traces when the host thread/session ID is known \
             so traces, feedback, and host transcripts can be joined."
                .to_string(),
        );
    }
    if report.distinct_scenario_count == 0 || report.distinct_arm_count == 0 {
        recommendations.push(
            "Use free-form scenario_id and arm on controlled eval traces so custom memory \
             strategies can be compared without expanding the intent taxonomy."
                .to_string(),
        );
    }
    if report.scored_feedback_count < report.feedback_count {
        recommendations.push(
            "Include usefulness, correctness, and noise scores on feedback records where \
             possible."
                .to_string(),
        );
    }
    if report.outcome_feedback_count < report.feedback_count {
        recommendations.push(
            "Include task_success, preference_adhered, repeated_context_questions, or \
             bad_memory_used when feedback can report task-level behavior."
                .to_string(),
        );
    }
    if report.unjudged_memory_feedback_count > 0 {
        recommendations.push(
            "Ask agents to populate memory attribution fields when returned memory shaped or was \
             considered for the result."
                .to_string(),
        );
    }
    if report.warning_count > 0 {
        recommendations.push(
            "Inspect trace warnings before treating latency or retrieval quality as healthy."
                .to_string(),
        );
    }
    if recommendations.is_empty() {
        recommendations.push(
            "Use this evidence for ranking calibration; migration writes still require explicit \
             user approval."
                .to_string(),
        );
    }

    recommendations
}

#[derive(Debug, Default)]
struct ExternalSessionFeedbackAggregation {
    with_session_count: usize,
    distinct_session_count: usize,
    without_session_count: usize,
}

fn count_feedback_traces(traces: &[BrainHarnessTrace], feedback: &[AgentFeedback]) -> usize {
    let trace_ids = trace_id_set(traces);
    feedback
        .iter()
        .filter(|item| trace_ids.contains(&item.trace_id))
        .map(|item| item.trace_id)
        .collect::<HashSet<_>>()
        .len()
}

fn count_memory_judgment_traces(traces: &[BrainHarnessTrace], feedback: &[AgentFeedback]) -> usize {
    let trace_ids = trace_id_set(traces);
    feedback
        .iter()
        .filter(|item| trace_ids.contains(&item.trace_id) && has_memory_judgment(item))
        .map(|item| item.trace_id)
        .collect::<HashSet<_>>()
        .len()
}

fn count_memory_judgment_eligible_traces(
    traces: &[BrainHarnessTrace],
    feedback: &[AgentFeedback],
) -> usize {
    memory_judgment_eligible_trace_ids(traces, feedback).len()
}

fn count_outcome_traces(traces: &[BrainHarnessTrace], feedback: &[AgentFeedback]) -> usize {
    let trace_ids = trace_id_set(traces);
    feedback
        .iter()
        .filter(|item| trace_ids.contains(&item.trace_id) && has_outcome_feedback(item))
        .map(|item| item.trace_id)
        .collect::<HashSet<_>>()
        .len()
}

fn aggregate_feedback_external_sessions(
    feedback: &[AgentFeedback],
) -> ExternalSessionFeedbackAggregation {
    let mut session_ids = HashSet::<String>::new();
    let mut aggregation = ExternalSessionFeedbackAggregation::default();

    for item in feedback {
        if let Some(external_session_id) = normalized_label(item.external_session_id.as_deref()) {
            aggregation.with_session_count += 1;
            session_ids.insert(external_session_id);
        } else {
            aggregation.without_session_count += 1;
        }
    }

    aggregation.distinct_session_count = session_ids.len();
    aggregation
}

fn trace_id_set(traces: &[BrainHarnessTrace]) -> HashSet<Id> {
    traces.iter().map(|trace| trace.id).collect()
}

fn sum_rows(
    rows: &[RealSessionEvalIntentRow],
    value: impl Fn(&RealSessionEvalIntentRow) -> usize,
) -> usize {
    rows.iter().map(value).sum()
}

fn count_memory_judgment_feedback(feedback: &[AgentFeedback]) -> usize {
    feedback
        .iter()
        .filter(|item| has_memory_judgment(item))
        .count()
}

fn count_unjudged_memory_feedback(
    traces: &[BrainHarnessTrace],
    feedback: &[AgentFeedback],
) -> usize {
    let memory_bearing_trace_ids = memory_judgment_eligible_trace_ids(traces, feedback);

    feedback
        .iter()
        .filter(|item| {
            memory_bearing_trace_ids.contains(&item.trace_id) && !has_memory_judgment(item)
        })
        .count()
}

fn memory_judgment_eligible_trace_ids(
    traces: &[BrainHarnessTrace],
    feedback: &[AgentFeedback],
) -> HashSet<Id> {
    let trace_ids = trace_id_set(traces);
    let mut eligible_ids = traces
        .iter()
        .filter(|trace| !trace.returned_memory_ids.is_empty())
        .map(|trace| trace.id)
        .collect::<HashSet<_>>();

    for item in feedback
        .iter()
        .filter(|item| trace_ids.contains(&item.trace_id) && has_memory_judgment(item))
    {
        eligible_ids.insert(item.trace_id);
    }

    eligible_ids
}

fn has_memory_judgment(feedback: &AgentFeedback) -> bool {
    !feedback.used_memory_ids.is_empty()
        || !feedback.rejected_memory_ids.is_empty()
        || !feedback.stale_memory_ids.is_empty()
        || !feedback.wrong_scope_memory_ids.is_empty()
}

fn add_outcome_counts_to_intent_row(row: &mut RealSessionEvalIntentRow, feedback: &AgentFeedback) {
    if has_outcome_feedback(feedback) {
        row.outcome_feedback_count += 1;
    }
    if let Some(task_success) = feedback.task_success {
        if task_success {
            row.task_success_count += 1;
        } else {
            row.task_failure_count += 1;
        }
    }
    if let Some(preference_adhered) = feedback.preference_adhered {
        if preference_adhered {
            row.preference_adhered_count += 1;
        } else {
            row.preference_violated_count += 1;
        }
    }
    if let Some(repeated_context_questions) = feedback.repeated_context_questions {
        row.repeated_context_question_count += repeated_context_questions as usize;
    }
    if feedback.bad_memory_used == Some(true) {
        row.bad_memory_used_count += 1;
    }
}

fn add_outcome_counts_to_arm_row(row: &mut RealSessionEvalArmRow, feedback: &AgentFeedback) {
    if has_outcome_feedback(feedback) {
        row.outcome_feedback_count += 1;
    }
    if let Some(task_success) = feedback.task_success {
        if task_success {
            row.task_success_count += 1;
        } else {
            row.task_failure_count += 1;
        }
    }
    if let Some(preference_adhered) = feedback.preference_adhered {
        if preference_adhered {
            row.preference_adhered_count += 1;
        } else {
            row.preference_violated_count += 1;
        }
    }
    if let Some(repeated_context_questions) = feedback.repeated_context_questions {
        row.repeated_context_question_count += repeated_context_questions as usize;
    }
    if feedback.bad_memory_used == Some(true) {
        row.bad_memory_used_count += 1;
    }
}

fn validate_trace(trace: &BrainHarnessTrace) -> IndexResult<()> {
    validate_external_session_id(trace.external_session_id.as_deref())?;
    if trace.returned_memory_ids.is_empty()
        && trace.returned_result_ids.is_empty()
        && trace.query.as_deref().unwrap_or_default().trim().is_empty()
    {
        return Err(IndexError::Parse(
            "trace must include query, returned_memory_ids, or returned_result_ids".to_string(),
        ));
    }
    Ok(())
}

fn validate_feedback(feedback: &AgentFeedback) -> IndexResult<()> {
    validate_external_session_id(feedback.external_session_id.as_deref())?;
    validate_score("usefulness_score", feedback.usefulness_score)?;
    validate_score("correctness_score", feedback.correctness_score)?;
    validate_score("noise_score", feedback.noise_score)?;

    let has_signal = !feedback.used_memory_ids.is_empty()
        || !feedback.rejected_memory_ids.is_empty()
        || !feedback.used_result_ids.is_empty()
        || !feedback.rejected_result_ids.is_empty()
        || !feedback.stale_memory_ids.is_empty()
        || !feedback.wrong_scope_memory_ids.is_empty()
        || feedback
            .missing_context
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
        || feedback
            .suggested_memory_changes
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
        || feedback
            .note
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
        || has_outcome_feedback(feedback);

    if !has_signal {
        return Err(IndexError::Parse(
            "feedback must include at least one concrete signal".to_string(),
        ));
    }

    Ok(())
}

fn validate_external_session_id(external_session_id: Option<&str>) -> IndexResult<()> {
    let Some(external_session_id) = external_session_id else {
        return Ok(());
    };
    let trimmed = external_session_id.trim();
    if trimmed.is_empty() {
        return Err(IndexError::Parse(
            "external_session_id must not be empty when provided".to_string(),
        ));
    }
    if trimmed.len() > 256 {
        return Err(IndexError::Parse(
            "external_session_id must be 256 characters or fewer".to_string(),
        ));
    }
    Ok(())
}

fn validate_score(name: &str, score: Option<u8>) -> IndexResult<()> {
    if let Some(score) = score {
        if !(1..=5).contains(&score) {
            return Err(IndexError::Parse(format!("{name} must be between 1 and 5")));
        }
    }
    Ok(())
}

fn intent_key(trace: &BrainHarnessTrace) -> String {
    trace
        .intent
        .as_ref()
        .map(std::string::ToString::to_string)
        .unwrap_or_else(|| "unspecified".to_string())
}

fn add_score(score: Option<u8>, sum: &mut u32, count: &mut usize) {
    if let Some(score) = score {
        *sum += u32::from(score);
        *count += 1;
    }
}

fn average_u64(sum: u64, count: usize) -> Option<f32> {
    if count == 0 {
        None
    } else {
        Some(sum as f32 / count as f32)
    }
}

fn average_u32(sum: u32, count: usize) -> Option<f32> {
    if count == 0 {
        None
    } else {
        Some(sum as f32 / count as f32)
    }
}

fn coverage(numerator: usize, denominator: usize) -> f32 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f32 / denominator as f32
    }
}

fn has_text(value: Option<&str>) -> bool {
    value.is_some_and(|text| !text.trim().is_empty())
}

fn has_scope_filter(project: Option<&str>, scenario_id: Option<&str>, arm: Option<&str>) -> bool {
    normalized_label_ref(project).is_some()
        || normalized_label_ref(scenario_id).is_some()
        || normalized_label_ref(arm).is_some()
}

fn filter_traces_by_scope(
    traces: Vec<BrainHarnessTrace>,
    project: Option<&str>,
    scenario_id: Option<&str>,
    arm: Option<&str>,
) -> Vec<BrainHarnessTrace> {
    traces
        .into_iter()
        .filter(|trace| trace_matches_scope(trace, project, scenario_id, arm))
        .collect()
}

fn trace_matches_scope(
    trace: &BrainHarnessTrace,
    project: Option<&str>,
    scenario_id: Option<&str>,
    arm: Option<&str>,
) -> bool {
    label_matches(trace.project.as_deref(), project)
        && label_matches(trace.scenario_id.as_deref(), scenario_id)
        && label_matches(trace.arm.as_deref(), arm)
}

fn applied_filters(
    project: Option<&str>,
    scenario_id: Option<&str>,
    arm: Option<&str>,
) -> RealSessionEvalAppliedFilters {
    RealSessionEvalAppliedFilters {
        project: normalized_label(project),
        scenario_id: normalized_label(scenario_id),
        arm: normalized_label(arm),
    }
}

fn has_any_filter(filters: &RealSessionEvalAppliedFilters) -> bool {
    filters.project.is_some() || filters.scenario_id.is_some() || filters.arm.is_some()
}

fn label_matches(actual: Option<&str>, expected: Option<&str>) -> bool {
    let Some(expected) = normalized_label_ref(expected) else {
        return true;
    };
    normalized_label_ref(actual) == Some(expected)
}

fn has_outcome_feedback(feedback: &AgentFeedback) -> bool {
    feedback.task_success.is_some()
        || feedback.preference_adhered.is_some()
        || feedback.repeated_context_questions.is_some()
        || feedback.bad_memory_used.is_some()
}

fn normalized_label(value: Option<&str>) -> Option<String> {
    let label = value?.trim();
    if label.is_empty() {
        None
    } else {
        Some(label.to_string())
    }
}

fn normalized_label_ref(value: Option<&str>) -> Option<&str> {
    let label = value?.trim();
    if label.is_empty() {
        None
    } else {
        Some(label)
    }
}
