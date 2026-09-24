//! Provider-free comparative reporting for completed native-memory pilot outcomes.

use crate::native_audit::{
    audit_native_memory_pilot, AuditStatus, NativeAcceptanceAudit, NativeLanePhase,
    NativePilotAudit,
};
use crate::native_document::load_historical_native_plan;
use crate::native_pilot::{
    MemoryLayer, NativePilotExpectedOutcome, NativePilotResourceBudgets, PreparedNativePilot,
};
use crate::{EvalError, EvalResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Unweighted outcome metrics declared by the frozen acceptance contract.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NativeOutcomeMetrics {
    pub passed: bool,
    pub expected_outcome: NativePilotExpectedOutcome,
    pub outcome_correct: bool,
    pub identity_correct: bool,
    pub first_action_correct: bool,
    pub context_applied: bool,
    pub context_handling_correct: bool,
    pub evidence_cited: bool,
    pub procedure_revalidated: bool,
    pub first_procedure_attempt_correct: bool,
    pub successful_command_executed: bool,
    pub expected_exit_observed: bool,
    pub success_marker_observed: bool,
    pub repeated_failures: u32,
    pub abstained: bool,
}

impl From<&NativeAcceptanceAudit> for NativeOutcomeMetrics {
    fn from(value: &NativeAcceptanceAudit) -> Self {
        Self {
            passed: value.passed,
            expected_outcome: value.expected_outcome,
            outcome_correct: value.outcome_correct,
            identity_correct: value.identity_correct,
            first_action_correct: value.first_action_correct,
            context_applied: value.context_applied,
            context_handling_correct: value.context_handling_correct,
            evidence_cited: value.evidence_cited,
            procedure_revalidated: value.procedure_revalidated,
            first_procedure_attempt_correct: value.first_procedure_attempt_correct,
            successful_command_executed: value.successful_command_executed,
            expected_exit_observed: value.expected_exit_observed,
            success_marker_observed: value.success_marker_observed,
            repeated_failures: value.repeated_failures,
            abstained: value.abstained,
        }
    }
}

/// Supplementary host telemetry extracted provider-free from the raw evaluation trace.
///
/// Token fields preserve each host's reported accounting. For Claude, `input_tokens` is the sum
/// of direct, cache-creation, and cache-read input tokens. Codex already reports its total input in
/// `input_tokens`; its cached input remains a subset. Engram result bytes are the exact UTF-8 size
/// of the JSON-serialized MCP `content` arrays returned to the host. Host-reported and
/// runner-observed durations remain separate.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NativeTraceTelemetry {
    pub host_reported_duration_ms: Option<u64>,
    pub runner_provider_duration_ms: Option<u64>,
    pub input_tokens: Option<u64>,
    pub cached_input_tokens: Option<u64>,
    pub cache_creation_input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub reasoning_output_tokens: Option<u64>,
    pub engram_tool_calls: u32,
    pub engram_result_bytes: u64,
    pub engram_max_result_bytes: u64,
    pub engram_result_bytes_by_tool: BTreeMap<String, u64>,
}

/// Host-visible resource evidence for one matched treatment-minus-native lane pair.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NativeMatchedResourceDelta {
    pub case_id: String,
    pub repetition: u32,
    pub native_total_tokens: Option<u64>,
    pub treatment_total_tokens: Option<u64>,
    pub incremental_total_tokens: Option<i64>,
    pub native_runner_duration_ms: Option<u64>,
    pub treatment_runner_duration_ms: Option<u64>,
    pub incremental_runner_duration_ms: Option<i64>,
    pub treatment_engram_result_bytes: Option<u64>,
    pub treatment_engram_max_result_bytes: Option<u64>,
    pub telemetry_failures: Vec<String>,
    pub budget_violations: Vec<String>,
}

/// One lane joined with its frozen host/layer identity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NativeLaneOutcome {
    pub order: u32,
    pub case_id: String,
    pub repetition: u32,
    pub arm: String,
    pub host: String,
    pub memory_layer: MemoryLayer,
    pub phase: NativeLanePhase,
    pub metrics: Option<NativeOutcomeMetrics>,
    pub telemetry: Option<NativeTraceTelemetry>,
    pub integrity_failures: Vec<String>,
    pub outcome_failures: Vec<String>,
}

/// Raw totals for one host/layer; no post-hoc weighting is applied.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NativeLayerAggregate {
    pub host: String,
    pub memory_layer: MemoryLayer,
    pub planned_lanes: u32,
    pub completed_lanes: u32,
    pub passed: u32,
    pub outcome_correct: u32,
    pub identity_correct: u32,
    pub first_action_correct: u32,
    pub context_applied: u32,
    pub context_handling_correct: u32,
    pub evidence_cited: u32,
    pub procedure_revalidated: u32,
    pub first_procedure_attempt_correct: u32,
    pub successful_command_executed: u32,
    pub expected_exit_observed: u32,
    pub success_marker_observed: u32,
    pub repeated_failures: u32,
    pub abstained: u32,
    pub correct_abstentions: u32,
}

impl NativeLayerAggregate {
    fn new(host: &str, memory_layer: MemoryLayer) -> Self {
        Self {
            host: host.to_string(),
            memory_layer,
            planned_lanes: 0,
            completed_lanes: 0,
            passed: 0,
            outcome_correct: 0,
            identity_correct: 0,
            first_action_correct: 0,
            context_applied: 0,
            context_handling_correct: 0,
            evidence_cited: 0,
            procedure_revalidated: 0,
            first_procedure_attempt_correct: 0,
            successful_command_executed: 0,
            expected_exit_observed: 0,
            success_marker_observed: 0,
            repeated_failures: 0,
            abstained: 0,
            correct_abstentions: 0,
        }
    }

    fn observe(&mut self, metrics: &NativeOutcomeMetrics) {
        self.completed_lanes += 1;
        self.passed += u32::from(metrics.passed);
        self.outcome_correct += u32::from(metrics.outcome_correct);
        self.identity_correct += u32::from(metrics.identity_correct);
        self.first_action_correct += u32::from(metrics.first_action_correct);
        self.context_applied += u32::from(metrics.context_applied);
        self.context_handling_correct += u32::from(metrics.context_handling_correct);
        self.evidence_cited += u32::from(metrics.evidence_cited);
        self.procedure_revalidated += u32::from(metrics.procedure_revalidated);
        self.first_procedure_attempt_correct += u32::from(metrics.first_procedure_attempt_correct);
        self.successful_command_executed += u32::from(metrics.successful_command_executed);
        self.expected_exit_observed += u32::from(metrics.expected_exit_observed);
        self.success_marker_observed += u32::from(metrics.success_marker_observed);
        self.repeated_failures += metrics.repeated_failures;
        self.abstained += u32::from(metrics.abstained);
        self.correct_abstentions += u32::from(
            metrics.expected_outcome == NativePilotExpectedOutcome::Abstain && metrics.abstained,
        );
    }
}

/// Matched treatment-minus-native differences for one host.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NativeLayerDelta {
    pub host: String,
    pub treatment: MemoryLayer,
    pub matched_pairs: u32,
    pub passed_delta: i32,
    pub outcome_correct_delta: i32,
    pub identity_correct_delta: i32,
    pub first_action_correct_delta: i32,
    pub context_applied_delta: i32,
    pub context_handling_correct_delta: i32,
    pub evidence_cited_delta: i32,
    pub procedure_revalidated_delta: i32,
    pub first_procedure_attempt_correct_delta: i32,
    pub successful_command_executed_delta: i32,
    pub expected_exit_observed_delta: i32,
    pub success_marker_observed_delta: i32,
    pub repeated_failures_avoided: i32,
    pub abstentions_avoided: i32,
    pub correct_abstentions_delta: i32,
    /// Exact host-visible resource evidence for each matched outcome pair.
    pub resource_pairs: Vec<NativeMatchedResourceDelta>,
    /// Number of matched pairs missing any measurement required by the frozen budgets.
    pub resource_telemetry_missing_pairs: u32,
    /// Number of preregistered resource limits exceeded across all matched pairs.
    pub resource_budget_violation_count: u32,
    /// `None` for historical plans without budgets; otherwise true only for complete clean evidence.
    pub within_resource_budgets: Option<bool>,
    /// Outcome-only Pareto result before applying any declared resource budget.
    pub pareto_dominates_native: bool,
}

/// Conservative descriptive signal from the preregistered matched comparisons.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NativeIncrementalValueSignal {
    NotReady,
    Invalid,
    NotObserved,
    Partial,
    EngramAcrossHosts,
    CombinedAcrossHosts,
    EngramAndCombinedAcrossHosts,
}

/// Complete provider-free comparison report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NativePilotComparisonReport {
    pub pilot_id: String,
    pub run_plan: String,
    pub complete: bool,
    pub invalid: bool,
    pub all_acceptance_passed: bool,
    pub resource_budgets: Option<NativePilotResourceBudgets>,
    pub all_resource_budgets_passed: Option<bool>,
    pub portable_incremental_value_observed: bool,
    pub incremental_value_signal: NativeIncrementalValueSignal,
    pub lane_outcomes: Vec<NativeLaneOutcome>,
    pub layer_aggregates: Vec<NativeLayerAggregate>,
    pub matched_deltas: Vec<NativeLayerDelta>,
    pub claim_limitations: Vec<String>,
}

/// Audit and compare a prepared native-memory pilot without invoking providers.
pub fn report_native_memory_pilot(run_plan: &Path) -> EvalResult<NativePilotComparisonReport> {
    let plan = load_historical_native_plan(run_plan)?;
    let audit = audit_native_memory_pilot(run_plan)?;
    let run_plan = run_plan.canonicalize()?;
    let runner_durations = runner_provider_durations(&run_plan, &plan.pilot_id)?;
    build_report(
        &plan,
        &audit,
        &run_plan.display().to_string(),
        &runner_durations,
    )
}

fn build_report(
    plan: &PreparedNativePilot,
    audit: &NativePilotAudit,
    run_plan: &str,
    runner_durations: &BTreeMap<u32, u64>,
) -> EvalResult<NativePilotComparisonReport> {
    let audits = audit
        .lanes
        .iter()
        .map(|lane| (lane.order, lane))
        .collect::<BTreeMap<_, _>>();
    if audits.len() != audit.lanes.len() || audit.lanes.len() != plan.lanes.len() {
        return Err(EvalError::Invalid(
            "pilot audit and frozen plan lane sets do not match".to_string(),
        ));
    }

    let mut lane_outcomes = Vec::with_capacity(plan.lanes.len());
    let mut aggregates = BTreeMap::new();
    let mut metric_index = BTreeMap::new();
    let mut telemetry_index = BTreeMap::new();
    for planned in &plan.lanes {
        let observed = audits.get(&planned.order).ok_or_else(|| {
            EvalError::Invalid(format!("pilot audit omitted lane {}", planned.order))
        })?;
        if observed.arm != planned.arm {
            return Err(EvalError::Invalid(format!(
                "pilot audit lane {} arm does not match frozen plan",
                planned.order
            )));
        }
        let key = (planned.host.clone(), planned.memory_layer);
        let aggregate = aggregates
            .entry(key)
            .or_insert_with(|| NativeLayerAggregate::new(&planned.host, planned.memory_layer));
        aggregate.planned_lanes += 1;
        let metrics = observed.acceptance.as_ref().map(NativeOutcomeMetrics::from);
        let telemetry = if observed.evaluation_trace.status == AuditStatus::Passed {
            Some(extract_trace_telemetry(
                &planned.host,
                Path::new(&planned.evaluation_trace_path),
                runner_durations.get(&planned.order).copied(),
            )?)
        } else {
            None
        };
        let match_key = (
            planned.host.clone(),
            planned.case_id.clone(),
            planned.repetition,
            planned.memory_layer,
        );
        if telemetry_index
            .insert(match_key.clone(), telemetry.clone())
            .is_some()
        {
            return Err(EvalError::Invalid(format!(
                "duplicate matched telemetry for lane {}",
                planned.order
            )));
        }
        if let Some(metrics) = &metrics {
            aggregate.observe(metrics);
            if metric_index.insert(match_key, metrics.clone()).is_some() {
                return Err(EvalError::Invalid(format!(
                    "duplicate matched outcome for lane {}",
                    planned.order
                )));
            }
        }
        lane_outcomes.push(NativeLaneOutcome {
            order: planned.order,
            case_id: planned.case_id.clone(),
            repetition: planned.repetition,
            arm: planned.arm.clone(),
            host: planned.host.clone(),
            memory_layer: planned.memory_layer,
            phase: observed.phase,
            metrics,
            telemetry,
            integrity_failures: observed.failures.clone(),
            outcome_failures: observed
                .acceptance
                .as_ref()
                .map_or_else(Vec::new, |acceptance| acceptance.failures.clone()),
        });
    }

    let hosts = plan
        .lanes
        .iter()
        .map(|lane| lane.host.clone())
        .collect::<BTreeSet<_>>();
    let mut matched_deltas = Vec::new();
    for host in &hosts {
        for treatment in [MemoryLayer::Engram, MemoryLayer::Both] {
            matched_deltas.push(matched_delta(
                host,
                treatment,
                &metric_index,
                &telemetry_index,
                plan.resource_budgets.as_ref(),
            )?);
        }
    }

    let engram_across_hosts = dominates_across_hosts(&hosts, &matched_deltas, MemoryLayer::Engram);
    let combined_across_hosts = dominates_across_hosts(&hosts, &matched_deltas, MemoryLayer::Both);
    let any_dominance = matched_deltas.iter().any(delta_supports_incremental_value);
    let all_resource_budgets_passed = plan.resource_budgets.as_ref().map(|_| {
        !matched_deltas.is_empty()
            && matched_deltas
                .iter()
                .all(|delta| delta.within_resource_budgets == Some(true))
    });
    let incremental_value_signal = if audit.invalid {
        NativeIncrementalValueSignal::Invalid
    } else if !audit.complete {
        NativeIncrementalValueSignal::NotReady
    } else {
        match (engram_across_hosts, combined_across_hosts, any_dominance) {
            (true, true, _) => NativeIncrementalValueSignal::EngramAndCombinedAcrossHosts,
            (true, false, _) => NativeIncrementalValueSignal::EngramAcrossHosts,
            (false, true, _) => NativeIncrementalValueSignal::CombinedAcrossHosts,
            (false, false, true) => NativeIncrementalValueSignal::Partial,
            (false, false, false) => NativeIncrementalValueSignal::NotObserved,
        }
    };
    let portable_incremental_value_observed =
        audit.complete && !audit.invalid && (engram_across_hosts || combined_across_hosts);

    let distinct_cases = plan
        .lanes
        .iter()
        .map(|lane| lane.case_id.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let max_repetition = plan
        .lanes
        .iter()
        .map(|lane| lane.repetition)
        .max()
        .unwrap_or(0);
    let mut claim_limitations = vec![
        "Pareto dominance is descriptive and uses only preregistered unweighted outcome metrics."
            .to_string(),
    ];
    if plan.resource_budgets.is_some() {
        claim_limitations.push(
            "Portable incremental value additionally requires every matched treatment/native pair to provide complete host token and runner-duration telemetry and satisfy the preregistered packet, token, and duration budgets."
                .to_string(),
        );
        if matched_deltas
            .iter()
            .any(|delta| delta.matched_pairs > 0 && delta.within_resource_budgets == Some(false))
        {
            claim_limitations.push(
                "At least one matched comparison exceeded a preregistered resource budget or lacked required telemetry; it cannot support a portable incremental-value claim."
                    .to_string(),
            );
        }
    } else {
        claim_limitations.push(
            "Trace telemetry is supplementary because this historical plan did not preregister resource budgets. Codex JSONL does not report a host duration; runner-observed duration is available only when the phase report contains per-lane timestamps."
                .to_string(),
        );
    }
    if hosts.len() != 2 || !hosts.contains("codex") || !hosts.contains("claude_code") {
        claim_limitations.push(
            "This is a single-host diagnostic; it cannot establish portable incremental value."
                .to_string(),
        );
    }
    if distinct_cases < 2 || max_repetition < 2 {
        claim_limitations.push(format!(
            "This pipeline pilot has {distinct_cases} distinct case(s) and at most {max_repetition} repetition(s); it is not a statistically powered product claim."
        ));
    }
    if !audit.complete {
        claim_limitations.push(
            "The pilot is incomplete; incremental-value classification is provisional.".to_string(),
        );
    }
    if audit.invalid {
        claim_limitations.push(
            "At least one lane has an integrity failure; outcome comparisons are not valid."
                .to_string(),
        );
    }

    Ok(NativePilotComparisonReport {
        pilot_id: plan.pilot_id.clone(),
        run_plan: run_plan.to_string(),
        complete: audit.complete,
        invalid: audit.invalid,
        all_acceptance_passed: audit.all_acceptance_passed,
        resource_budgets: plan.resource_budgets.clone(),
        all_resource_budgets_passed,
        portable_incremental_value_observed,
        incremental_value_signal,
        lane_outcomes,
        layer_aggregates: aggregates.into_values().collect(),
        matched_deltas,
        claim_limitations,
    })
}

fn runner_provider_durations(
    run_plan: &Path,
    expected_pilot_id: &str,
) -> EvalResult<BTreeMap<u32, u64>> {
    let report_path = run_plan
        .parent()
        .ok_or_else(|| EvalError::Invalid("run plan has no parent".to_string()))?
        .join("runner-evaluation.json");
    if !report_path.exists() {
        return Ok(BTreeMap::new());
    }
    let report: Value = serde_json::from_reader(fs::File::open(&report_path)?)?;
    if report.get("pilot_id").and_then(Value::as_str) != Some(expected_pilot_id)
        || report.get("phase").and_then(Value::as_str) != Some("evaluation")
    {
        return Err(EvalError::Invalid(
            "evaluation runner report does not match the frozen pilot".to_string(),
        ));
    }

    let mut durations = BTreeMap::new();
    for lane in report
        .get("lanes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let order = lane.get("order").and_then(Value::as_u64).ok_or_else(|| {
            EvalError::Invalid("evaluation runner lane omitted its order".to_string())
        })?;
        let order = u32::try_from(order).map_err(|_| {
            EvalError::Invalid("evaluation runner lane order is too large".to_string())
        })?;
        let started = lane.get("provider_started_unix_ms").and_then(Value::as_u64);
        let completed = lane
            .get("provider_completed_unix_ms")
            .and_then(Value::as_u64);
        let duration = match (started, completed) {
            (None, None) => continue,
            (Some(started), Some(completed)) => {
                completed.checked_sub(started).ok_or_else(|| {
                    EvalError::Invalid(format!(
                        "evaluation runner lane {order} completed before it started"
                    ))
                })?
            }
            _ => {
                return Err(EvalError::Invalid(format!(
                    "evaluation runner lane {order} has an incomplete provider interval"
                )))
            }
        };
        if durations.insert(order, duration).is_some() {
            return Err(EvalError::Invalid(format!(
                "evaluation runner report duplicates lane {order}"
            )));
        }
    }
    Ok(durations)
}

fn extract_trace_telemetry(
    host: &str,
    trace: &Path,
    runner_provider_duration_ms: Option<u64>,
) -> EvalResult<NativeTraceTelemetry> {
    let reader = BufReader::new(fs::File::open(trace)?);
    let mut values = Vec::new();
    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        values.push(serde_json::from_str::<Value>(&line).map_err(|error| {
            EvalError::Invalid(format!(
                "evaluation trace line {} is invalid JSON: {error}",
                index + 1
            ))
        })?);
    }
    match host {
        "claude_code" => extract_claude_trace_telemetry(&values, runner_provider_duration_ms),
        "codex" => extract_codex_trace_telemetry(&values, runner_provider_duration_ms),
        other => Err(EvalError::Invalid(format!(
            "unsupported native-memory telemetry host: {other}"
        ))),
    }
}

fn extract_claude_trace_telemetry(
    values: &[Value],
    runner_provider_duration_ms: Option<u64>,
) -> EvalResult<NativeTraceTelemetry> {
    let mut tool_names = BTreeMap::new();
    let mut terminal_usage = None;
    let mut host_reported_duration_ms = None;
    for value in values {
        if value.get("type").and_then(Value::as_str) == Some("assistant") {
            for content in value
                .pointer("/message/content")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                if content.get("type").and_then(Value::as_str) != Some("tool_use") {
                    continue;
                }
                let Some(id) = content.get("id").and_then(Value::as_str) else {
                    continue;
                };
                let Some(name) = content.get("name").and_then(Value::as_str) else {
                    continue;
                };
                tool_names.insert(id.to_string(), name.to_string());
            }
        }
        if value.get("type").and_then(Value::as_str) == Some("result") {
            if terminal_usage.is_some() {
                return Err(EvalError::Invalid(
                    "Claude evaluation trace contains multiple terminal result records".to_string(),
                ));
            }
            terminal_usage = value.get("usage").cloned();
            host_reported_duration_ms = value.get("duration_ms").and_then(Value::as_u64);
        }
    }

    let mut engram_tool_calls = 0_u32;
    let mut engram_result_bytes = 0_u64;
    let mut engram_max_result_bytes = 0_u64;
    let mut engram_result_bytes_by_tool = BTreeMap::new();
    for value in values {
        if value.get("type").and_then(Value::as_str) != Some("user") {
            continue;
        }
        for content in value
            .pointer("/message/content")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if content.get("type").and_then(Value::as_str) != Some("tool_result") {
                continue;
            }
            let Some(id) = content.get("tool_use_id").and_then(Value::as_str) else {
                continue;
            };
            let Some(tool) = tool_names
                .get(id)
                .and_then(|name| name.strip_prefix("mcp__engram__"))
            else {
                continue;
            };
            observe_engram_result(
                &mut engram_tool_calls,
                &mut engram_result_bytes,
                &mut engram_max_result_bytes,
                &mut engram_result_bytes_by_tool,
                tool,
                content.get("content").unwrap_or(&Value::Null),
            )?;
        }
    }

    let direct_input = terminal_usage
        .as_ref()
        .and_then(|usage| usage.get("input_tokens"))
        .and_then(Value::as_u64);
    let cache_creation = terminal_usage
        .as_ref()
        .and_then(|usage| usage.get("cache_creation_input_tokens"))
        .and_then(Value::as_u64);
    let cache_read = terminal_usage
        .as_ref()
        .and_then(|usage| usage.get("cache_read_input_tokens"))
        .and_then(Value::as_u64);
    let input_tokens = sum_present_tokens([direct_input, cache_creation, cache_read])?;

    Ok(NativeTraceTelemetry {
        host_reported_duration_ms,
        runner_provider_duration_ms,
        input_tokens,
        cached_input_tokens: cache_read,
        cache_creation_input_tokens: cache_creation,
        output_tokens: terminal_usage
            .as_ref()
            .and_then(|usage| usage.get("output_tokens"))
            .and_then(Value::as_u64),
        reasoning_output_tokens: None,
        engram_tool_calls,
        engram_result_bytes,
        engram_max_result_bytes,
        engram_result_bytes_by_tool,
    })
}

fn extract_codex_trace_telemetry(
    values: &[Value],
    runner_provider_duration_ms: Option<u64>,
) -> EvalResult<NativeTraceTelemetry> {
    let mut terminal_usage = None;
    let mut engram_tool_calls = 0_u32;
    let mut engram_result_bytes = 0_u64;
    let mut engram_max_result_bytes = 0_u64;
    let mut engram_result_bytes_by_tool = BTreeMap::new();
    for value in values {
        match value.get("type").and_then(Value::as_str) {
            Some("turn.completed") => {
                if terminal_usage.is_some() {
                    return Err(EvalError::Invalid(
                        "Codex evaluation trace contains multiple terminal turn records"
                            .to_string(),
                    ));
                }
                terminal_usage = value.get("usage").cloned();
            }
            Some("item.completed")
                if value.pointer("/item/type").and_then(Value::as_str) == Some("mcp_tool_call")
                    && value.pointer("/item/server").and_then(Value::as_str) == Some("engram") =>
            {
                let tool = value
                    .pointer("/item/tool")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        EvalError::Invalid(
                            "Codex Engram tool result omitted its tool name".to_string(),
                        )
                    })?;
                observe_engram_result(
                    &mut engram_tool_calls,
                    &mut engram_result_bytes,
                    &mut engram_max_result_bytes,
                    &mut engram_result_bytes_by_tool,
                    tool,
                    value
                        .pointer("/item/result/content")
                        .unwrap_or(&Value::Null),
                )?;
            }
            _ => {}
        }
    }

    Ok(NativeTraceTelemetry {
        host_reported_duration_ms: None,
        runner_provider_duration_ms,
        input_tokens: terminal_usage
            .as_ref()
            .and_then(|usage| usage.get("input_tokens"))
            .and_then(Value::as_u64),
        cached_input_tokens: terminal_usage
            .as_ref()
            .and_then(|usage| usage.get("cached_input_tokens"))
            .and_then(Value::as_u64),
        cache_creation_input_tokens: terminal_usage
            .as_ref()
            .and_then(|usage| usage.get("cache_write_input_tokens"))
            .and_then(Value::as_u64),
        output_tokens: terminal_usage
            .as_ref()
            .and_then(|usage| usage.get("output_tokens"))
            .and_then(Value::as_u64),
        reasoning_output_tokens: terminal_usage
            .as_ref()
            .and_then(|usage| usage.get("reasoning_output_tokens"))
            .and_then(Value::as_u64),
        engram_tool_calls,
        engram_result_bytes,
        engram_max_result_bytes,
        engram_result_bytes_by_tool,
    })
}

fn sum_present_tokens(values: [Option<u64>; 3]) -> EvalResult<Option<u64>> {
    if values.iter().all(Option::is_none) {
        return Ok(None);
    }
    values
        .into_iter()
        .flatten()
        .try_fold(Some(0_u64), |total, value| {
            Ok(Some(
                total
                    .unwrap_or_default()
                    .checked_add(value)
                    .ok_or_else(|| {
                        EvalError::Invalid("provider token count overflow".to_string())
                    })?,
            ))
        })
}

fn observe_engram_result(
    call_count: &mut u32,
    total_bytes: &mut u64,
    max_bytes: &mut u64,
    bytes_by_tool: &mut BTreeMap<String, u64>,
    tool: &str,
    value: &Value,
) -> EvalResult<()> {
    let bytes = u64::try_from(serde_json::to_vec(value)?.len())
        .map_err(|_| EvalError::Invalid("serialized Engram result is too large".to_string()))?;
    *call_count = call_count
        .checked_add(1)
        .ok_or_else(|| EvalError::Invalid("Engram tool-call count overflow".to_string()))?;
    *total_bytes = total_bytes
        .checked_add(bytes)
        .ok_or_else(|| EvalError::Invalid("Engram result-byte count overflow".to_string()))?;
    *max_bytes = (*max_bytes).max(bytes);
    let tool_total = bytes_by_tool.entry(tool.to_string()).or_default();
    *tool_total = tool_total
        .checked_add(bytes)
        .ok_or_else(|| EvalError::Invalid("Engram per-tool byte count overflow".to_string()))?;
    Ok(())
}

fn matched_delta(
    host: &str,
    treatment: MemoryLayer,
    metrics: &BTreeMap<(String, String, u32, MemoryLayer), NativeOutcomeMetrics>,
    telemetry: &BTreeMap<(String, String, u32, MemoryLayer), Option<NativeTraceTelemetry>>,
    resource_budgets: Option<&NativePilotResourceBudgets>,
) -> EvalResult<NativeLayerDelta> {
    let mut delta = NativeLayerDelta {
        host: host.to_string(),
        treatment,
        matched_pairs: 0,
        passed_delta: 0,
        outcome_correct_delta: 0,
        identity_correct_delta: 0,
        first_action_correct_delta: 0,
        context_applied_delta: 0,
        context_handling_correct_delta: 0,
        evidence_cited_delta: 0,
        procedure_revalidated_delta: 0,
        first_procedure_attempt_correct_delta: 0,
        successful_command_executed_delta: 0,
        expected_exit_observed_delta: 0,
        success_marker_observed_delta: 0,
        repeated_failures_avoided: 0,
        abstentions_avoided: 0,
        correct_abstentions_delta: 0,
        resource_pairs: Vec::new(),
        resource_telemetry_missing_pairs: 0,
        resource_budget_violation_count: 0,
        within_resource_budgets: resource_budgets.map(|_| false),
        pareto_dominates_native: false,
    };
    let mut every_pair_nonregressive = true;
    let mut any_pair_strictly_better = false;
    for ((native_host, case_id, repetition, layer), native) in metrics {
        if native_host != host || *layer != MemoryLayer::Native {
            continue;
        }
        let Some(treated) =
            metrics.get(&(host.to_string(), case_id.clone(), *repetition, treatment))
        else {
            continue;
        };
        let outcome_contract_matches = treated.expected_outcome == native.expected_outcome;
        let common_pair = [
            bool_delta(treated.passed, native.passed),
            bool_delta(treated.outcome_correct, native.outcome_correct),
            bool_delta(treated.identity_correct, native.identity_correct),
            bool_delta(treated.first_action_correct, native.first_action_correct),
            bool_delta(
                treated.context_handling_correct,
                native.context_handling_correct,
            ),
            bool_delta(treated.evidence_cited, native.evidence_cited),
            bool_delta(treated.procedure_revalidated, native.procedure_revalidated),
            bool_delta(
                treated.first_procedure_attempt_correct,
                native.first_procedure_attempt_correct,
            ),
            signed_u32(native.repeated_failures) - signed_u32(treated.repeated_failures),
        ];
        let outcome_pair =
            if native.expected_outcome == NativePilotExpectedOutcome::ExecuteProcedure {
                vec![
                    bool_delta(
                        treated.successful_command_executed,
                        native.successful_command_executed,
                    ),
                    bool_delta(
                        treated.expected_exit_observed,
                        native.expected_exit_observed,
                    ),
                    bool_delta(
                        treated.success_marker_observed,
                        native.success_marker_observed,
                    ),
                    bool_delta(native.abstained, treated.abstained),
                ]
            } else {
                vec![bool_delta(treated.abstained, native.abstained)]
            };
        every_pair_nonregressive &= outcome_contract_matches
            && common_pair.iter().all(|value| *value >= 0)
            && outcome_pair.iter().all(|value| *value >= 0);
        any_pair_strictly_better |= common_pair.iter().any(|value| *value > 0)
            || outcome_pair.iter().any(|value| *value > 0);
        delta.matched_pairs += 1;
        delta.passed_delta += common_pair[0];
        delta.outcome_correct_delta += common_pair[1];
        delta.identity_correct_delta += common_pair[2];
        delta.first_action_correct_delta += common_pair[3];
        delta.context_applied_delta += bool_delta(treated.context_applied, native.context_applied);
        delta.context_handling_correct_delta += common_pair[4];
        delta.evidence_cited_delta += common_pair[5];
        delta.procedure_revalidated_delta += common_pair[6];
        delta.first_procedure_attempt_correct_delta += common_pair[7];
        delta.successful_command_executed_delta += bool_delta(
            treated.successful_command_executed,
            native.successful_command_executed,
        );
        delta.expected_exit_observed_delta += bool_delta(
            treated.expected_exit_observed,
            native.expected_exit_observed,
        );
        delta.success_marker_observed_delta += bool_delta(
            treated.success_marker_observed,
            native.success_marker_observed,
        );
        delta.repeated_failures_avoided += common_pair[8];
        if native.expected_outcome == NativePilotExpectedOutcome::ExecuteProcedure {
            delta.abstentions_avoided += bool_delta(native.abstained, treated.abstained);
        } else {
            delta.correct_abstentions_delta += bool_delta(treated.abstained, native.abstained);
        }
        if let Some(budgets) = resource_budgets {
            let resource_pair =
                matched_resource_delta(host, case_id, *repetition, treatment, telemetry, budgets)?;
            delta.resource_telemetry_missing_pairs +=
                u32::from(!resource_pair.telemetry_failures.is_empty());
            delta.resource_budget_violation_count = delta
                .resource_budget_violation_count
                .checked_add(
                    u32::try_from(resource_pair.budget_violations.len()).map_err(|_| {
                        EvalError::Invalid("resource-budget violation count overflow".to_string())
                    })?,
                )
                .ok_or_else(|| {
                    EvalError::Invalid("resource-budget violation count overflow".to_string())
                })?;
            delta.resource_pairs.push(resource_pair);
        }
    }
    delta.pareto_dominates_native =
        delta.matched_pairs > 0 && every_pair_nonregressive && any_pair_strictly_better;
    if resource_budgets.is_some() {
        delta.within_resource_budgets = Some(
            delta.resource_pairs.len() == delta.matched_pairs as usize
                && delta.matched_pairs > 0
                && delta.resource_telemetry_missing_pairs == 0
                && delta.resource_budget_violation_count == 0,
        );
    }
    Ok(delta)
}

fn matched_resource_delta(
    host: &str,
    case_id: &str,
    repetition: u32,
    treatment: MemoryLayer,
    telemetry: &BTreeMap<(String, String, u32, MemoryLayer), Option<NativeTraceTelemetry>>,
    budgets: &NativePilotResourceBudgets,
) -> EvalResult<NativeMatchedResourceDelta> {
    let native = telemetry
        .get(&(
            host.to_string(),
            case_id.to_string(),
            repetition,
            MemoryLayer::Native,
        ))
        .and_then(Option::as_ref);
    let treated = telemetry
        .get(&(host.to_string(), case_id.to_string(), repetition, treatment))
        .and_then(Option::as_ref);
    let mut telemetry_failures = Vec::new();
    let native_total_tokens = required_total_tokens(native, "native", &mut telemetry_failures)?;
    let treatment_total_tokens =
        required_total_tokens(treated, "treatment", &mut telemetry_failures)?;
    let native_runner_duration_ms =
        required_runner_duration(native, "native", &mut telemetry_failures);
    let treatment_runner_duration_ms =
        required_runner_duration(treated, "treatment", &mut telemetry_failures);
    let incremental_total_tokens = signed_optional_delta(
        treatment_total_tokens,
        native_total_tokens,
        "total-token delta",
    )?;
    let incremental_runner_duration_ms = signed_optional_delta(
        treatment_runner_duration_ms,
        native_runner_duration_ms,
        "runner-duration delta",
    )?;
    let treatment_engram_result_bytes = treated.map(|value| value.engram_result_bytes);
    let treatment_engram_max_result_bytes = treated.map(|value| value.engram_max_result_bytes);

    let mut budget_violations = Vec::new();
    if treatment_engram_max_result_bytes
        .is_some_and(|value| value > budgets.max_engram_result_bytes_per_call)
    {
        budget_violations.push("engram_result_bytes_per_call".to_string());
    }
    if treatment_engram_result_bytes
        .is_some_and(|value| value > budgets.max_engram_result_bytes_per_lane)
    {
        budget_violations.push("engram_result_bytes_per_lane".to_string());
    }
    if incremental_total_tokens.is_some_and(|value| {
        i128::from(value) > i128::from(budgets.max_incremental_total_tokens_per_lane)
    }) {
        budget_violations.push("incremental_total_tokens".to_string());
    }
    if incremental_runner_duration_ms.is_some_and(|value| {
        i128::from(value) > i128::from(budgets.max_incremental_runner_duration_ms_per_lane)
    }) {
        budget_violations.push("incremental_runner_duration_ms".to_string());
    }

    Ok(NativeMatchedResourceDelta {
        case_id: case_id.to_string(),
        repetition,
        native_total_tokens,
        treatment_total_tokens,
        incremental_total_tokens,
        native_runner_duration_ms,
        treatment_runner_duration_ms,
        incremental_runner_duration_ms,
        treatment_engram_result_bytes,
        treatment_engram_max_result_bytes,
        telemetry_failures,
        budget_violations,
    })
}

fn required_total_tokens(
    telemetry: Option<&NativeTraceTelemetry>,
    side: &str,
    failures: &mut Vec<String>,
) -> EvalResult<Option<u64>> {
    let Some(telemetry) = telemetry else {
        failures.push(format!("{side}_telemetry_unavailable"));
        return Ok(None);
    };
    let (Some(input), Some(output)) = (telemetry.input_tokens, telemetry.output_tokens) else {
        failures.push(format!("{side}_total_tokens_unavailable"));
        return Ok(None);
    };
    Ok(Some(input.checked_add(output).ok_or_else(|| {
        EvalError::Invalid(format!("{side} total-token count overflow"))
    })?))
}

fn required_runner_duration(
    telemetry: Option<&NativeTraceTelemetry>,
    side: &str,
    failures: &mut Vec<String>,
) -> Option<u64> {
    let duration = telemetry.and_then(|value| value.runner_provider_duration_ms);
    if telemetry.is_some() && duration.is_none() {
        failures.push(format!("{side}_runner_duration_unavailable"));
    }
    duration
}

fn signed_optional_delta(
    treatment: Option<u64>,
    native: Option<u64>,
    label: &str,
) -> EvalResult<Option<i64>> {
    let (Some(treatment), Some(native)) = (treatment, native) else {
        return Ok(None);
    };
    i64::try_from(i128::from(treatment) - i128::from(native))
        .map(Some)
        .map_err(|_| EvalError::Invalid(format!("{label} overflow")))
}

fn delta_supports_incremental_value(delta: &NativeLayerDelta) -> bool {
    delta.pareto_dominates_native && delta.within_resource_budgets.unwrap_or(true)
}

fn dominates_across_hosts(
    hosts: &BTreeSet<String>,
    deltas: &[NativeLayerDelta],
    treatment: MemoryLayer,
) -> bool {
    hosts.len() == 2
        && hosts.contains("codex")
        && hosts.contains("claude_code")
        && hosts.iter().all(|host| {
            deltas.iter().any(|delta| {
                delta.host == *host
                    && delta.treatment == treatment
                    && delta_supports_incremental_value(delta)
            })
        })
}

fn bool_delta(treatment: bool, native: bool) -> i32 {
    i32::from(treatment) - i32::from(native)
}

fn signed_u32(value: u32) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn metrics(passed: bool, repeated_failures: u32, abstained: bool) -> NativeOutcomeMetrics {
        NativeOutcomeMetrics {
            passed,
            expected_outcome: NativePilotExpectedOutcome::ExecuteProcedure,
            outcome_correct: passed,
            identity_correct: true,
            first_action_correct: passed,
            context_applied: passed,
            context_handling_correct: passed,
            evidence_cited: passed,
            procedure_revalidated: true,
            first_procedure_attempt_correct: passed,
            successful_command_executed: passed,
            expected_exit_observed: passed,
            success_marker_observed: passed,
            repeated_failures,
            abstained,
        }
    }

    fn abstention_metrics(passed: bool, abstained: bool) -> NativeOutcomeMetrics {
        NativeOutcomeMetrics {
            passed,
            expected_outcome: NativePilotExpectedOutcome::Abstain,
            outcome_correct: passed,
            identity_correct: true,
            first_action_correct: passed,
            context_applied: false,
            context_handling_correct: passed,
            evidence_cited: passed,
            procedure_revalidated: true,
            first_procedure_attempt_correct: passed,
            successful_command_executed: false,
            expected_exit_observed: false,
            success_marker_observed: false,
            repeated_failures: 0,
            abstained,
        }
    }

    fn legacy_delta(
        host: &str,
        treatment: MemoryLayer,
        metrics: &BTreeMap<(String, String, u32, MemoryLayer), NativeOutcomeMetrics>,
    ) -> NativeLayerDelta {
        matched_delta(host, treatment, metrics, &BTreeMap::new(), None).unwrap()
    }

    fn trace_telemetry(
        total_tokens: u64,
        runner_duration_ms: Option<u64>,
        engram_result_bytes: u64,
        engram_max_result_bytes: u64,
    ) -> NativeTraceTelemetry {
        NativeTraceTelemetry {
            host_reported_duration_ms: None,
            runner_provider_duration_ms: runner_duration_ms,
            input_tokens: Some(total_tokens - 10),
            cached_input_tokens: None,
            cache_creation_input_tokens: None,
            output_tokens: Some(10),
            reasoning_output_tokens: None,
            engram_tool_calls: u32::from(engram_result_bytes > 0),
            engram_result_bytes,
            engram_max_result_bytes,
            engram_result_bytes_by_tool: BTreeMap::new(),
        }
    }

    fn resource_budgets() -> NativePilotResourceBudgets {
        NativePilotResourceBudgets {
            max_engram_result_bytes_per_call: 8_192,
            max_engram_result_bytes_per_lane: 16_384,
            max_incremental_total_tokens_per_lane: 500,
            max_incremental_runner_duration_ms_per_lane: 300,
        }
    }

    #[test]
    fn matched_delta_preserves_failed_native_outcome_as_comparison_evidence() {
        let index = BTreeMap::from([
            (
                (
                    "codex".to_string(),
                    "case".to_string(),
                    1,
                    MemoryLayer::Native,
                ),
                metrics(false, 1, true),
            ),
            (
                (
                    "codex".to_string(),
                    "case".to_string(),
                    1,
                    MemoryLayer::Engram,
                ),
                metrics(true, 0, false),
            ),
        ]);
        let delta = legacy_delta("codex", MemoryLayer::Engram, &index);

        assert_eq!(delta.matched_pairs, 1);
        assert_eq!(delta.passed_delta, 1);
        assert_eq!(delta.repeated_failures_avoided, 1);
        assert_eq!(delta.abstentions_avoided, 1);
        assert!(delta.pareto_dominates_native);
    }

    #[test]
    fn matched_delta_rewards_required_abstention_without_requiring_command_success() {
        let index = BTreeMap::from([
            (
                (
                    "codex".to_string(),
                    "mismatch".to_string(),
                    1,
                    MemoryLayer::Native,
                ),
                abstention_metrics(false, false),
            ),
            (
                (
                    "codex".to_string(),
                    "mismatch".to_string(),
                    1,
                    MemoryLayer::Engram,
                ),
                abstention_metrics(true, true),
            ),
        ]);
        let delta = legacy_delta("codex", MemoryLayer::Engram, &index);

        assert_eq!(delta.matched_pairs, 1);
        assert_eq!(delta.outcome_correct_delta, 1);
        assert_eq!(delta.correct_abstentions_delta, 1);
        assert_eq!(delta.successful_command_executed_delta, 0);
        assert_eq!(delta.abstentions_avoided, 0);
        assert!(delta.pareto_dominates_native);
    }

    #[test]
    fn any_regression_prevents_pareto_dominance() {
        let mut treated = metrics(true, 0, false);
        treated.identity_correct = false;
        let index = BTreeMap::from([
            (
                (
                    "claude_code".to_string(),
                    "case".to_string(),
                    1,
                    MemoryLayer::Native,
                ),
                metrics(false, 1, true),
            ),
            (
                (
                    "claude_code".to_string(),
                    "case".to_string(),
                    1,
                    MemoryLayer::Both,
                ),
                treated,
            ),
        ]);
        let delta = legacy_delta("claude_code", MemoryLayer::Both, &index);

        assert_eq!(delta.identity_correct_delta, -1);
        assert!(!delta.pareto_dominates_native);
    }

    #[test]
    fn aggregate_cancellation_does_not_hide_a_pairwise_regression() {
        let mut native_one = metrics(false, 1, true);
        native_one.identity_correct = false;
        let treated_one = metrics(true, 0, false);
        let native_two = metrics(false, 1, true);
        let mut treated_two = metrics(true, 0, false);
        treated_two.identity_correct = false;
        let index = BTreeMap::from([
            (
                (
                    "codex".to_string(),
                    "one".to_string(),
                    1,
                    MemoryLayer::Native,
                ),
                native_one,
            ),
            (
                (
                    "codex".to_string(),
                    "one".to_string(),
                    1,
                    MemoryLayer::Engram,
                ),
                treated_one,
            ),
            (
                (
                    "codex".to_string(),
                    "two".to_string(),
                    1,
                    MemoryLayer::Native,
                ),
                native_two,
            ),
            (
                (
                    "codex".to_string(),
                    "two".to_string(),
                    1,
                    MemoryLayer::Engram,
                ),
                treated_two,
            ),
        ]);
        let delta = legacy_delta("codex", MemoryLayer::Engram, &index);

        assert_eq!(delta.identity_correct_delta, 0);
        assert_eq!(delta.passed_delta, 2);
        assert!(!delta.pareto_dominates_native);
    }

    #[test]
    fn portable_signal_requires_the_same_treatment_to_dominate_every_host() {
        let hosts = BTreeSet::from(["claude_code".to_string(), "codex".to_string()]);
        let mut codex = legacy_delta("codex", MemoryLayer::Engram, &BTreeMap::new());
        codex.matched_pairs = 1;
        codex.pareto_dominates_native = true;
        let mut claude = legacy_delta("claude_code", MemoryLayer::Engram, &BTreeMap::new());
        claude.matched_pairs = 1;
        claude.pareto_dominates_native = true;

        assert!(dominates_across_hosts(
            &hosts,
            &[codex.clone(), claude],
            MemoryLayer::Engram
        ));
        assert!(!dominates_across_hosts(
            &hosts,
            &[codex],
            MemoryLayer::Engram
        ));

        let single_host = BTreeSet::from(["claude_code".to_string()]);
        let mut claude = legacy_delta("claude_code", MemoryLayer::Engram, &BTreeMap::new());
        claude.matched_pairs = 1;
        claude.pareto_dominates_native = true;
        assert!(!dominates_across_hosts(
            &single_host,
            &[claude],
            MemoryLayer::Engram
        ));
    }

    #[test]
    fn complete_resource_evidence_within_budget_preserves_value_signal() {
        let metrics = BTreeMap::from([
            (
                (
                    "codex".to_string(),
                    "case".to_string(),
                    1,
                    MemoryLayer::Native,
                ),
                self::metrics(false, 1, true),
            ),
            (
                (
                    "codex".to_string(),
                    "case".to_string(),
                    1,
                    MemoryLayer::Engram,
                ),
                self::metrics(true, 0, false),
            ),
        ]);
        let telemetry = BTreeMap::from([
            (
                (
                    "codex".to_string(),
                    "case".to_string(),
                    1,
                    MemoryLayer::Native,
                ),
                Some(trace_telemetry(1_000, Some(1_000), 0, 0)),
            ),
            (
                (
                    "codex".to_string(),
                    "case".to_string(),
                    1,
                    MemoryLayer::Engram,
                ),
                Some(trace_telemetry(1_400, Some(1_200), 6_000, 3_000)),
            ),
        ]);

        let delta = matched_delta(
            "codex",
            MemoryLayer::Engram,
            &metrics,
            &telemetry,
            Some(&resource_budgets()),
        )
        .unwrap();

        assert_eq!(delta.within_resource_budgets, Some(true));
        assert_eq!(delta.resource_telemetry_missing_pairs, 0);
        assert_eq!(delta.resource_budget_violation_count, 0);
        assert_eq!(delta.resource_pairs[0].incremental_total_tokens, Some(400));
        assert_eq!(
            delta.resource_pairs[0].incremental_runner_duration_ms,
            Some(200)
        );
        assert!(delta_supports_incremental_value(&delta));
    }

    #[test]
    fn resource_budget_violation_blocks_value_signal() {
        let metrics = BTreeMap::from([
            (
                (
                    "codex".to_string(),
                    "case".to_string(),
                    1,
                    MemoryLayer::Native,
                ),
                self::metrics(false, 1, true),
            ),
            (
                (
                    "codex".to_string(),
                    "case".to_string(),
                    1,
                    MemoryLayer::Engram,
                ),
                self::metrics(true, 0, false),
            ),
        ]);
        let telemetry = BTreeMap::from([
            (
                (
                    "codex".to_string(),
                    "case".to_string(),
                    1,
                    MemoryLayer::Native,
                ),
                Some(trace_telemetry(1_000, Some(1_000), 0, 0)),
            ),
            (
                (
                    "codex".to_string(),
                    "case".to_string(),
                    1,
                    MemoryLayer::Engram,
                ),
                Some(trace_telemetry(1_600, Some(1_200), 9_000, 9_000)),
            ),
        ]);

        let delta = matched_delta(
            "codex",
            MemoryLayer::Engram,
            &metrics,
            &telemetry,
            Some(&resource_budgets()),
        )
        .unwrap();

        assert!(delta.pareto_dominates_native);
        assert_eq!(delta.within_resource_budgets, Some(false));
        assert_eq!(delta.resource_budget_violation_count, 2);
        assert_eq!(
            delta.resource_pairs[0].budget_violations,
            vec![
                "engram_result_bytes_per_call".to_string(),
                "incremental_total_tokens".to_string()
            ]
        );
        assert!(!delta_supports_incremental_value(&delta));
    }

    #[test]
    fn missing_resource_telemetry_fails_closed() {
        let metrics = BTreeMap::from([
            (
                (
                    "codex".to_string(),
                    "case".to_string(),
                    1,
                    MemoryLayer::Native,
                ),
                self::metrics(false, 1, true),
            ),
            (
                (
                    "codex".to_string(),
                    "case".to_string(),
                    1,
                    MemoryLayer::Engram,
                ),
                self::metrics(true, 0, false),
            ),
        ]);
        let telemetry = BTreeMap::from([
            (
                (
                    "codex".to_string(),
                    "case".to_string(),
                    1,
                    MemoryLayer::Native,
                ),
                Some(trace_telemetry(1_000, Some(1_000), 0, 0)),
            ),
            (
                (
                    "codex".to_string(),
                    "case".to_string(),
                    1,
                    MemoryLayer::Engram,
                ),
                Some(trace_telemetry(1_100, None, 1_000, 1_000)),
            ),
        ]);

        let delta = matched_delta(
            "codex",
            MemoryLayer::Engram,
            &metrics,
            &telemetry,
            Some(&resource_budgets()),
        )
        .unwrap();

        assert_eq!(delta.resource_telemetry_missing_pairs, 1);
        assert_eq!(delta.resource_budget_violation_count, 0);
        assert_eq!(delta.within_resource_budgets, Some(false));
        assert_eq!(
            delta.resource_pairs[0].telemetry_failures,
            vec!["treatment_runner_duration_unavailable".to_string()]
        );
        assert!(!delta_supports_incremental_value(&delta));
    }

    #[test]
    fn claude_trace_telemetry_normalizes_tokens_and_counts_engram_results() {
        let temp = tempfile::tempdir().unwrap();
        let trace = temp.path().join("claude.jsonl");
        let result_content = json!([{"type": "text", "text": "packet"}]);
        let values = [
            json!({
                "type": "assistant",
                "message": {"content": [{
                    "type": "tool_use",
                    "id": "tool-1",
                    "name": "mcp__engram__orient"
                }]}
            }),
            json!({
                "type": "user",
                "message": {"content": [{
                    "type": "tool_result",
                    "tool_use_id": "tool-1",
                    "content": result_content.clone()
                }]}
            }),
            json!({
                "type": "result",
                "duration_ms": 1234,
                "usage": {
                    "input_tokens": 10,
                    "cache_creation_input_tokens": 20,
                    "cache_read_input_tokens": 30,
                    "output_tokens": 40
                }
            }),
        ];
        let body = values
            .iter()
            .map(|value| serde_json::to_string(value).unwrap())
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&trace, format!("{body}\n")).unwrap();

        let telemetry = extract_trace_telemetry("claude_code", &trace, Some(1500)).unwrap();

        assert_eq!(telemetry.host_reported_duration_ms, Some(1234));
        assert_eq!(telemetry.runner_provider_duration_ms, Some(1500));
        assert_eq!(telemetry.input_tokens, Some(60));
        assert_eq!(telemetry.cached_input_tokens, Some(30));
        assert_eq!(telemetry.cache_creation_input_tokens, Some(20));
        assert_eq!(telemetry.output_tokens, Some(40));
        assert_eq!(telemetry.reasoning_output_tokens, None);
        assert_eq!(telemetry.engram_tool_calls, 1);
        assert_eq!(
            telemetry.engram_result_bytes,
            serde_json::to_vec(&result_content).unwrap().len() as u64
        );
        assert_eq!(
            telemetry.engram_max_result_bytes,
            telemetry.engram_result_bytes
        );
        assert_eq!(
            telemetry.engram_result_bytes_by_tool,
            BTreeMap::from([("orient".to_string(), telemetry.engram_result_bytes)])
        );
    }

    #[test]
    fn codex_trace_telemetry_preserves_host_usage_and_counts_engram_results() {
        let temp = tempfile::tempdir().unwrap();
        let trace = temp.path().join("codex.jsonl");
        let result_content = json!([{"type": "text", "text": "packet"}]);
        let values = [
            json!({
                "type": "item.completed",
                "item": {
                    "type": "mcp_tool_call",
                    "server": "engram",
                    "tool": "orient",
                    "result": {"content": result_content.clone()}
                }
            }),
            json!({
                "type": "turn.completed",
                "usage": {
                    "input_tokens": 100,
                    "cached_input_tokens": 70,
                    "cache_write_input_tokens": 5,
                    "output_tokens": 20,
                    "reasoning_output_tokens": 4
                }
            }),
        ];
        let body = values
            .iter()
            .map(|value| serde_json::to_string(value).unwrap())
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&trace, format!("{body}\n")).unwrap();

        let telemetry = extract_trace_telemetry("codex", &trace, Some(2000)).unwrap();

        assert_eq!(telemetry.host_reported_duration_ms, None);
        assert_eq!(telemetry.runner_provider_duration_ms, Some(2000));
        assert_eq!(telemetry.input_tokens, Some(100));
        assert_eq!(telemetry.cached_input_tokens, Some(70));
        assert_eq!(telemetry.cache_creation_input_tokens, Some(5));
        assert_eq!(telemetry.output_tokens, Some(20));
        assert_eq!(telemetry.reasoning_output_tokens, Some(4));
        assert_eq!(telemetry.engram_tool_calls, 1);
        assert_eq!(
            telemetry.engram_result_bytes,
            serde_json::to_vec(&result_content).unwrap().len() as u64
        );
        assert_eq!(
            telemetry.engram_max_result_bytes,
            telemetry.engram_result_bytes
        );
        assert_eq!(
            telemetry.engram_result_bytes_by_tool,
            BTreeMap::from([("orient".to_string(), telemetry.engram_result_bytes)])
        );
    }

    #[test]
    fn runner_durations_accept_legacy_reports_and_validate_new_intervals() {
        let temp = tempfile::tempdir().unwrap();
        let plan = temp.path().join("run-plan.json");
        fs::write(&plan, "{}\n").unwrap();
        let report = temp.path().join("runner-evaluation.json");
        fs::write(
            &report,
            serde_json::to_vec(&json!({
                "pilot_id": "pilot",
                "phase": "evaluation",
                "lanes": [
                    {"order": 1},
                    {
                        "order": 2,
                        "provider_started_unix_ms": 1000,
                        "provider_completed_unix_ms": 1250
                    }
                ]
            }))
            .unwrap(),
        )
        .unwrap();

        let durations = runner_provider_durations(&plan, "pilot").unwrap();

        assert_eq!(durations, BTreeMap::from([(2, 250)]));
    }
}
