//! Brain harness telemetry service.

use crate::error::{IndexError, IndexResult};
use engram_core::id::Id;
use engram_core::telemetry::{
    AgentFeedback, BrainHarnessTrace, IntentTelemetryStats, RealSessionConfidenceGate,
    RealSessionEvalIntentRow, RealSessionEvalReport,
};
use engram_store::{Db, TelemetryRepo};
use std::collections::{BTreeMap, HashMap};
use time::OffsetDateTime;

const DEFAULT_REAL_SESSION_EVAL_LIMIT: usize = 10_000;
const MIN_REAL_SESSION_TRACES: usize = 20;
const MIN_REAL_SESSION_FEEDBACK: usize = 10;
const MIN_REAL_SESSION_FEEDBACK_COVERAGE: f32 = 0.5;
const MIN_REAL_SESSION_INTENTS_WITH_FEEDBACK: usize = 3;

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

    /// Submit agent feedback for a trace.
    pub async fn submit_feedback(&self, feedback: AgentFeedback) -> IndexResult<AgentFeedback> {
        validate_feedback(&feedback)?;
        if self.repo.get_trace(&feedback.trace_id).await?.is_none() {
            return Err(IndexError::NotFound(format!(
                "trace not found: {}",
                feedback.trace_id
            )));
        }

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
        let feedback = self.repo.list_feedback(Some(sample_limit)).await?;

        Ok(build_real_session_eval_report(
            sample_limit,
            &traces,
            &feedback,
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
        self.row.feedback_coverage = coverage(self.row.feedback_count, self.row.trace_count);
        self.row.avg_latency_ms = average_u64(self.latency_sum, self.latency_count);
        self.row.avg_usefulness_score = average_u32(self.usefulness_sum, self.usefulness_count);
        self.row.avg_correctness_score = average_u32(self.correctness_sum, self.correctness_count);
        self.row.avg_noise_score = average_u32(self.noise_sum, self.noise_count);
        self.row
    }
}

#[derive(Debug)]
struct TraceAggregation {
    groups: BTreeMap<String, RealSessionEvalAggregate>,
    operation_counts: BTreeMap<String, usize>,
    unspecified_intent_trace_count: usize,
    trace_intents: HashMap<Id, String>,
}

fn build_real_session_eval_report(
    sample_limit: usize,
    traces: &[BrainHarnessTrace],
    feedback: &[AgentFeedback],
) -> RealSessionEvalReport {
    let mut aggregation = aggregate_traces(traces);
    aggregate_feedback(
        feedback,
        &aggregation.trace_intents,
        &mut aggregation.groups,
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

    let mut report = report_from_rows(
        sample_limit,
        traces.len(),
        feedback.len(),
        aggregation.operation_counts,
        aggregation.unspecified_intent_trace_count,
        intents,
    );
    report.confidence_gate = confidence_gate(&report);
    report.recommendations = recommendations(&report);
    report
}

fn aggregate_traces(traces: &[BrainHarnessTrace]) -> TraceAggregation {
    let mut groups = BTreeMap::<String, RealSessionEvalAggregate>::new();
    let mut operation_counts = BTreeMap::<String, usize>::new();
    let mut unspecified_intent_trace_count = 0;
    let mut trace_intents = HashMap::<Id, String>::new();

    for trace in traces {
        let key = intent_key(trace);
        if trace.intent.is_none() {
            unspecified_intent_trace_count += 1;
        }
        *operation_counts
            .entry(trace.operation.to_string())
            .or_default() += 1;
        trace_intents.insert(trace.id, key.clone());
        groups
            .entry(key.clone())
            .or_insert_with(|| RealSessionEvalAggregate::new(key))
            .add_trace(trace);
    }

    TraceAggregation {
        groups,
        operation_counts,
        unspecified_intent_trace_count,
        trace_intents,
    }
}

fn aggregate_feedback(
    feedback: &[AgentFeedback],
    trace_intents: &HashMap<Id, String>,
    groups: &mut BTreeMap<String, RealSessionEvalAggregate>,
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
    }
}

fn report_from_rows(
    sample_limit: usize,
    trace_count: usize,
    feedback_count: usize,
    operation_counts: BTreeMap<String, usize>,
    unspecified_intent_trace_count: usize,
    intents: Vec<RealSessionEvalIntentRow>,
) -> RealSessionEvalReport {
    RealSessionEvalReport {
        generated_at: OffsetDateTime::now_utc(),
        sample_limit,
        trace_count,
        feedback_count,
        feedback_coverage: coverage(feedback_count, trace_count),
        distinct_intent_count: intents.len(),
        distinct_operation_count: operation_counts.len(),
        unspecified_intent_trace_count,
        operation_counts,
        warning_count: sum_rows(&intents, |row| row.warning_count),
        returned_memory_count: sum_rows(&intents, |row| row.returned_memory_count),
        returned_result_count: sum_rows(&intents, |row| row.returned_result_count),
        used_memory_count: sum_rows(&intents, |row| row.used_memory_count),
        rejected_memory_count: sum_rows(&intents, |row| row.rejected_memory_count),
        stale_memory_count: sum_rows(&intents, |row| row.stale_memory_count),
        wrong_scope_memory_count: sum_rows(&intents, |row| row.wrong_scope_memory_count),
        used_result_count: sum_rows(&intents, |row| row.used_result_count),
        rejected_result_count: sum_rows(&intents, |row| row.rejected_result_count),
        missing_context_count: sum_rows(&intents, |row| row.missing_context_count),
        suggested_change_count: sum_rows(&intents, |row| row.suggested_change_count),
        scored_feedback_count: sum_rows(&intents, |row| row.scored_feedback_count),
        intents,
        confidence_gate: RealSessionConfidenceGate {
            passed: false,
            min_trace_count: MIN_REAL_SESSION_TRACES,
            min_feedback_count: MIN_REAL_SESSION_FEEDBACK,
            min_feedback_coverage: MIN_REAL_SESSION_FEEDBACK_COVERAGE,
            min_intents_with_feedback: MIN_REAL_SESSION_INTENTS_WITH_FEEDBACK,
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

    RealSessionConfidenceGate {
        passed: reasons.is_empty(),
        min_trace_count: MIN_REAL_SESSION_TRACES,
        min_feedback_count: MIN_REAL_SESSION_FEEDBACK,
        min_feedback_coverage: MIN_REAL_SESSION_FEEDBACK_COVERAGE,
        min_intents_with_feedback: MIN_REAL_SESSION_INTENTS_WITH_FEEDBACK,
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
    if report.unspecified_intent_trace_count > 0 {
        recommendations.push(
            "Set intent on every telemetry trace so retrieval accuracy can be compared by \
             workflow."
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

fn sum_rows(
    rows: &[RealSessionEvalIntentRow],
    value: impl Fn(&RealSessionEvalIntentRow) -> usize,
) -> usize {
    rows.iter().map(value).sum()
}

fn validate_trace(trace: &BrainHarnessTrace) -> IndexResult<()> {
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
            .is_some_and(|value| !value.trim().is_empty());

    if !has_signal {
        return Err(IndexError::Parse(
            "feedback must include at least one concrete signal".to_string(),
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
