//! Frozen evaluation protocol and deterministic scorer for engineering-context agents.

pub mod comparison;
pub mod fixture;
pub mod lean_probe;
pub mod native_audit;
pub mod native_correction;
mod native_document;
mod native_execution;
pub mod native_instructions_control;
pub mod native_pilot;
pub mod native_report;
pub mod native_runner;
pub mod native_stale;
pub mod native_successor;
mod native_successor_artifact;
mod native_successor_core;
mod native_successor_policy;
mod native_successor_relay;
mod native_successor_semantic;
pub mod pilot;
pub mod procedure_probe;
pub mod retrieval_probe;
pub mod retrieval_probe_v2;
pub mod retrieval_probe_v3;
pub mod retrieval_probe_v4;
pub mod retrieval_probe_v5;
pub mod seed;

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use thiserror::Error;
use unicode_normalization::UnicodeNormalization;

/// Evaluation loading or validation error.
#[derive(Debug, Error)]
pub enum EvalError {
    /// Filesystem error.
    #[error("filesystem error: {0}")]
    Io(#[from] std::io::Error),
    /// JSON parsing error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    /// Invalid manifest, scenario, or run record.
    #[error("invalid evaluation data: {0}")]
    Invalid(String),
}

/// Evaluation result type.
pub type EvalResult<T> = Result<T, EvalError>;

/// One comparison arm in a controlled evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvaluationArm {
    /// Stable arm identifier recorded in run results.
    pub id: String,
    /// Human-readable configuration description.
    pub description: String,
    /// Required host harness for this arm.
    pub host: String,
    /// Whether this arm requires matched Engram runtime attestation.
    pub uses_engram: bool,
}

/// Frozen suite manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvaluationManifest {
    /// Protocol schema version.
    pub schema_version: u32,
    /// Stable suite identifier.
    pub suite_id: String,
    /// JSONL scenario file, relative to the manifest.
    pub scenario_file: String,
    /// Comparison arms that must all run.
    pub arms: Vec<EvaluationArm>,
    /// Required repetitions for every scenario/arm pair.
    pub repetitions: u32,
    /// Preregistered matched comparisons; required by schema v2 and absent from frozen v1.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub comparisons: Vec<comparison::EvaluationComparison>,
}

/// Product journey exercised by a scenario.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Journey {
    /// Resolve repository, project, component, or worktree identity.
    RepositoryIdentity,
    /// Recall a current engineering decision.
    Decision,
    /// Recall and apply a verified procedure or gotcha.
    Procedure,
    /// Prevent unrelated project context from entering the packet.
    WrongScope,
    /// Prevent stale or superseded guidance from affecting work.
    StaleMemory,
    /// Correctly report that no durable context applies.
    NoResult,
    /// Resume correctly after compaction or a new session.
    CompactionResume,
    /// Reject sensitive content and remove projections on deletion.
    Safety,
}

/// Exact expectations used for deterministic scoring.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScenarioExpectations {
    /// Expected normalized repository remote, when identity is resolvable.
    pub repository_remote: Option<String>,
    /// Expected project name.
    pub project: Option<String>,
    /// Expected monorepo component.
    pub component: Option<String>,
    /// Expected first externally visible action label.
    pub first_action: Option<String>,
    /// Stable context claim keys that must appear.
    #[serde(default)]
    pub required_context_keys: Vec<String>,
    /// Stable context claim keys that must not be returned or applied.
    #[serde(default)]
    pub forbidden_context_keys: Vec<String>,
    /// Evidence targets that must be cited or opened.
    #[serde(default)]
    pub required_evidence_targets: Vec<String>,
    /// Synthetic canary markers that must never appear in durable/context output.
    #[serde(default)]
    pub forbidden_markers: Vec<String>,
    /// Whether the correct behavior is to abstain or ask for clarification.
    #[serde(default)]
    pub abstain: bool,
    /// Whether repository, project, and component must all remain unresolved.
    #[serde(default)]
    pub unresolved_identity: bool,
}

/// Per-scenario resource budgets. These are declared before results are collected.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScenarioBudgets {
    /// Maximum wall-clock latency in milliseconds.
    pub latency_ms: Option<u64>,
    /// Maximum injected context packet size in bytes.
    pub context_packet_bytes: Option<u64>,
    /// Maximum total input plus output tokens.
    pub total_tokens: Option<u64>,
}

/// One frozen scenario.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Scenario {
    /// Stable scenario identifier.
    pub id: String,
    /// Journey category.
    pub journey: Journey,
    /// Prompt supplied to the host.
    pub prompt: String,
    /// Logical fixture checkout identifier or path.
    pub cwd: Option<String>,
    /// Explicit project supplied to the host, if any.
    pub project: Option<String>,
    /// Deterministic expectations.
    pub expected: ScenarioExpectations,
    /// Predeclared resource budgets.
    #[serde(default)]
    pub budgets: ScenarioBudgets,
    /// Human-readable setup details that are not scored.
    pub notes: Option<String>,
}

/// One observed run emitted by a host-specific runner or human evaluator.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunRecord {
    /// Scenario identifier.
    pub scenario_id: String,
    /// Comparison arm identifier.
    pub arm: String,
    /// One-based repetition number.
    pub repetition: u32,
    /// Host harness, such as codex or claude_code.
    pub host: String,
    /// Exact host version or build label.
    pub host_version: String,
    /// Exact model identifier used for the run.
    pub model: String,
    /// Immutable fixture Git revision or content hash.
    pub fixture_revision: String,
    /// Engram runtime attestation for arms that use Engram.
    pub engram_attestation: Option<RunAttestation>,
    /// Repository remote resolved by the host/Engram flow.
    pub repository_remote: Option<String>,
    /// Project resolved by the host/Engram flow.
    pub project: Option<String>,
    /// Component resolved by the host/Engram flow.
    pub component: Option<String>,
    /// First externally visible action label.
    pub first_action: Option<String>,
    /// Stable context claim keys returned in the context packet.
    #[serde(default)]
    pub returned_context_keys: Vec<String>,
    /// Stable context claim keys actually applied while completing the task.
    #[serde(default)]
    pub applied_context_keys: Vec<String>,
    /// Evidence targets cited or opened during the run.
    #[serde(default)]
    pub evidence_targets: Vec<String>,
    /// Synthetic markers observed in durable or injected output.
    #[serde(default)]
    pub observed_markers: Vec<String>,
    /// Whether the task's acceptance test passed.
    pub task_success: bool,
    /// Whether the host abstained or asked for clarification.
    pub abstained: bool,
    /// Number of repetitions of a known failed approach.
    #[serde(default)]
    pub repeated_failures: u32,
    /// Number of user corrections or interventions.
    #[serde(default)]
    pub user_corrections: u32,
    /// Stale context claim keys that influenced an action.
    #[serde(default)]
    pub stale_context_keys_applied: Vec<String>,
    /// Evidence claims checked against their sources.
    #[serde(default)]
    pub evidence_claims_checked: u32,
    /// Checked evidence claims found correct.
    #[serde(default)]
    pub evidence_claims_correct: u32,
    /// End-to-end wall-clock latency.
    pub latency_ms: u64,
    /// Host-reported input tokens.
    #[serde(default)]
    pub input_tokens: u64,
    /// Host-reported output tokens.
    #[serde(default)]
    pub output_tokens: u64,
    /// Injected context packet size.
    #[serde(default)]
    pub context_packet_bytes: u64,
    /// Optional evaluator note.
    pub notes: Option<String>,
}

/// Runtime provenance captured before an Engram-backed run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunAttestation {
    /// CLI semantic version.
    pub cli_version: String,
    /// Live daemon semantic version.
    pub daemon_version: String,
    /// Live schema version.
    pub schema_version: u32,
    /// Adapter expected SHA-256.
    pub adapter_expected_sha256: String,
    /// Adapter actual SHA-256.
    pub adapter_actual_sha256: String,
    /// Attestation status: matched, degraded, or drifted.
    pub status: String,
}

/// Deterministic outcome for one run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScoredRun {
    /// Scenario identifier.
    pub scenario_id: String,
    /// Comparison arm identifier.
    pub arm: String,
    /// One-based repetition number.
    pub repetition: u32,
    /// Whether repository/project/component identity matched exactly.
    pub identity_correct: bool,
    /// Whether the first action label matched.
    pub first_action_correct: bool,
    /// Required context claim keys not returned.
    pub missing_required_context_keys: Vec<String>,
    /// Forbidden context claim keys returned or applied.
    pub scope_leak_context_keys: Vec<String>,
    /// Required evidence targets not cited or opened.
    pub missing_evidence_targets: Vec<String>,
    /// Forbidden synthetic markers observed.
    pub leaked_markers: Vec<String>,
    /// Whether abstention behavior matched the scenario.
    pub abstention_correct: bool,
    /// Evidence accuracy for checked claims.
    pub evidence_accuracy: Option<f64>,
    /// Predeclared budget violations.
    pub budget_violations: Vec<String>,
    /// Hard correctness gate failures.
    pub gate_failures: Vec<String>,
}

/// Aggregate metrics for one arm.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArmScore {
    /// Arm identifier.
    pub arm: String,
    /// Expected run count.
    pub expected_runs: usize,
    /// Submitted run count.
    pub completed_runs: usize,
    /// Runs with no hard gate failures.
    pub clean_runs: usize,
    /// Task success rate.
    pub task_success_rate: f64,
    /// Exact identity success rate.
    pub identity_accuracy: f64,
    /// Correct first-action rate.
    pub first_action_accuracy: f64,
    /// Correct abstention rate.
    pub abstention_accuracy: f64,
    /// Runs containing returned or applied wrong-scope context.
    pub scope_leak_runs: usize,
    /// Runs influenced by stale items.
    pub stale_influence_runs: usize,
    /// Runs exposing synthetic secret markers.
    pub secret_leak_runs: usize,
    /// Total repeated failures.
    pub repeated_failures: u64,
    /// Total user corrections.
    pub user_corrections: u64,
    /// Accuracy across all checked evidence claims.
    pub evidence_accuracy: Option<f64>,
    /// Median latency.
    pub latency_p50_ms: u64,
    /// P95 latency using nearest-rank percentile.
    pub latency_p95_ms: u64,
    /// Median context packet size.
    pub context_packet_p50_bytes: u64,
    /// P95 context packet size.
    pub context_packet_p95_bytes: u64,
    /// Median total token count.
    pub total_tokens_p50: u64,
    /// P95 total token count.
    pub total_tokens_p95: u64,
    /// Total predeclared budget violations.
    pub budget_violation_count: usize,
}

/// Full deterministic scoring report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScoreReport {
    /// Protocol schema version.
    pub schema_version: u32,
    /// Suite identifier.
    pub suite_id: String,
    /// Missing scenario/arm/repetition combinations.
    pub missing_runs: Vec<String>,
    /// Per-run deterministic outcomes.
    pub runs: Vec<ScoredRun>,
    /// Aggregate metrics by arm.
    pub arms: Vec<ArmScore>,
    /// Preregistered matched treatment-minus-baseline comparisons.
    pub comparisons: Vec<comparison::ComparisonScore>,
    /// Claim groups that observed incremental value on every declared host.
    pub portable_incremental_value_groups: Vec<String>,
}

/// Load and validate a suite manifest and its relative scenario file.
pub fn load_suite(manifest_path: &Path) -> EvalResult<(EvaluationManifest, Vec<Scenario>)> {
    let manifest: EvaluationManifest = serde_json::from_reader(File::open(manifest_path)?)?;
    let scenario_path = manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(&manifest.scenario_file);
    let scenarios = read_json_lines(&scenario_path)?;
    validate_suite(&manifest, &scenarios)?;
    Ok((manifest, scenarios))
}

/// Load JSONL run records.
pub fn load_runs(path: &Path) -> EvalResult<Vec<RunRecord>> {
    read_json_lines(path)
}

/// Validate suite invariants that make comparisons reproducible.
pub fn validate_suite(manifest: &EvaluationManifest, scenarios: &[Scenario]) -> EvalResult<()> {
    if !matches!(manifest.schema_version, 1 | 2) {
        return Err(EvalError::Invalid(format!(
            "unsupported schema_version {}; expected 1 or 2",
            manifest.schema_version
        )));
    }
    if manifest.suite_id.trim().is_empty() {
        return Err(EvalError::Invalid("suite_id must not be empty".to_string()));
    }
    if manifest.repetitions == 0 {
        return Err(EvalError::Invalid(
            "repetitions must be greater than zero".to_string(),
        ));
    }
    if manifest.arms.len() < 2 {
        return Err(EvalError::Invalid(
            "at least two comparison arms are required".to_string(),
        ));
    }
    unique_nonempty(manifest.arms.iter().map(|arm| arm.id.as_str()), "arm ID")?;
    for arm in &manifest.arms {
        if arm.host.trim().is_empty() {
            return Err(EvalError::Invalid(format!(
                "arm {} has an empty host",
                arm.id
            )));
        }
    }
    let arms = manifest
        .arms
        .iter()
        .map(|arm| (arm.id.as_str(), arm))
        .collect::<BTreeMap<_, _>>();
    comparison::validate_comparisons(manifest, &arms)?;
    unique_nonempty(
        scenarios.iter().map(|scenario| scenario.id.as_str()),
        "scenario ID",
    )?;
    if scenarios.is_empty() {
        return Err(EvalError::Invalid(
            "at least one scenario is required".to_string(),
        ));
    }
    for scenario in scenarios {
        if scenario.prompt.trim().is_empty() {
            return Err(EvalError::Invalid(format!(
                "scenario {} has an empty prompt",
                scenario.id
            )));
        }
        let required: BTreeSet<_> = scenario.expected.required_context_keys.iter().collect();
        let forbidden: BTreeSet<_> = scenario.expected.forbidden_context_keys.iter().collect();
        if let Some(item_id) = required.intersection(&forbidden).next() {
            return Err(EvalError::Invalid(format!(
                "scenario {} both requires and forbids item {item_id}",
                scenario.id
            )));
        }
    }
    Ok(())
}

/// Score all submitted runs against the frozen suite.
pub fn score_runs(
    manifest: &EvaluationManifest,
    scenarios: &[Scenario],
    runs: &[RunRecord],
) -> EvalResult<ScoreReport> {
    validate_suite(manifest, scenarios)?;
    let scenario_by_id: BTreeMap<_, _> = scenarios
        .iter()
        .map(|scenario| (scenario.id.as_str(), scenario))
        .collect();
    let arms: BTreeMap<_, _> = manifest
        .arms
        .iter()
        .map(|arm| (arm.id.as_str(), arm))
        .collect();
    let mut seen = BTreeSet::new();
    let mut scored_runs = Vec::with_capacity(runs.len());

    for run in runs {
        let scenario = scenario_by_id
            .get(run.scenario_id.as_str())
            .ok_or_else(|| {
                EvalError::Invalid(format!("unknown scenario in run: {}", run.scenario_id))
            })?;
        let arm = arms
            .get(run.arm.as_str())
            .ok_or_else(|| EvalError::Invalid(format!("unknown arm in run: {}", run.arm)))?;
        if normalize_label(&run.host) != normalize_label(&arm.host) {
            return Err(EvalError::Invalid(format!(
                "run {}/{} uses host {}, expected {}",
                run.scenario_id, run.arm, run.host, arm.host
            )));
        }
        if arm.uses_engram {
            let attestation = run.engram_attestation.as_ref().ok_or_else(|| {
                EvalError::Invalid(format!(
                    "Engram arm {}/{} is missing runtime attestation",
                    run.scenario_id, run.arm
                ))
            })?;
            if attestation.status != "matched"
                || attestation.adapter_expected_sha256 != attestation.adapter_actual_sha256
            {
                return Err(EvalError::Invalid(format!(
                    "Engram arm {}/{} has unmatched runtime attestation",
                    run.scenario_id, run.arm
                )));
            }
        } else if run.engram_attestation.is_some() {
            return Err(EvalError::Invalid(format!(
                "non-Engram arm {}/{} unexpectedly includes Engram attestation",
                run.scenario_id, run.arm
            )));
        }
        if run.repetition == 0 || run.repetition > manifest.repetitions {
            return Err(EvalError::Invalid(format!(
                "run {}/{} has repetition {} outside 1..={}",
                run.scenario_id, run.arm, run.repetition, manifest.repetitions
            )));
        }
        let key = (run.scenario_id.clone(), run.arm.clone(), run.repetition);
        if !seen.insert(key.clone()) {
            return Err(EvalError::Invalid(format!(
                "duplicate run: {}/{}/{}",
                key.0, key.1, key.2
            )));
        }
        if run.evidence_claims_correct > run.evidence_claims_checked {
            return Err(EvalError::Invalid(format!(
                "run {}/{} reports more correct evidence claims than checked claims",
                run.scenario_id, run.arm
            )));
        }
        for (label, value) in [
            ("host", run.host.as_str()),
            ("host_version", run.host_version.as_str()),
            ("model", run.model.as_str()),
            ("fixture_revision", run.fixture_revision.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(EvalError::Invalid(format!(
                    "run {}/{} has an empty {label}",
                    run.scenario_id, run.arm
                )));
            }
        }
        scored_runs.push(score_run(scenario, run));
    }

    let missing_runs = missing_runs(manifest, scenarios, &seen);
    let arm_scores = manifest
        .arms
        .iter()
        .map(|arm| aggregate_arm(manifest, scenarios.len(), &arm.id, runs, &scored_runs))
        .collect();
    let (comparison_scores, portable_incremental_value_groups) =
        comparison::score_comparisons(manifest, scenarios, runs, &scored_runs)?;
    scored_runs.sort_by(|left, right| {
        left.arm
            .cmp(&right.arm)
            .then_with(|| left.scenario_id.cmp(&right.scenario_id))
            .then_with(|| left.repetition.cmp(&right.repetition))
    });

    Ok(ScoreReport {
        schema_version: manifest.schema_version,
        suite_id: manifest.suite_id.clone(),
        missing_runs,
        runs: scored_runs,
        arms: arm_scores,
        comparisons: comparison_scores,
        portable_incremental_value_groups,
    })
}

fn score_run(scenario: &Scenario, run: &RunRecord) -> ScoredRun {
    let expected = &scenario.expected;
    let identity_correct = if expected.unresolved_identity {
        run.repository_remote.is_none() && run.project.is_none() && run.component.is_none()
    } else {
        optional_identity_matches(
            expected.repository_remote.as_deref(),
            run.repository_remote.as_deref(),
            true,
        ) && optional_identity_matches(expected.project.as_deref(), run.project.as_deref(), false)
            && optional_identity_matches(
                expected.component.as_deref(),
                run.component.as_deref(),
                false,
            )
    };
    let first_action_correct = optional_identity_matches(
        expected.first_action.as_deref(),
        run.first_action.as_deref(),
        false,
    );
    let returned: BTreeSet<_> = run.returned_context_keys.iter().cloned().collect();
    let applied: BTreeSet<_> = run.applied_context_keys.iter().cloned().collect();
    let observed: BTreeSet<_> = returned.union(&applied).cloned().collect();
    let evidence: BTreeSet<_> = run.evidence_targets.iter().cloned().collect();
    let observed_markers: BTreeSet<_> = run.observed_markers.iter().cloned().collect();
    let missing_required_context_keys = expected
        .required_context_keys
        .iter()
        .filter(|id| !returned.contains(*id))
        .cloned()
        .collect::<Vec<_>>();
    let scope_leak_context_keys = expected
        .forbidden_context_keys
        .iter()
        .filter(|id| observed.contains(*id))
        .cloned()
        .collect::<Vec<_>>();
    let missing_evidence_targets = expected
        .required_evidence_targets
        .iter()
        .filter(|target| !evidence.contains(*target))
        .cloned()
        .collect::<Vec<_>>();
    let leaked_markers = expected
        .forbidden_markers
        .iter()
        .filter(|marker| observed_markers.contains(*marker))
        .cloned()
        .collect::<Vec<_>>();
    let abstention_correct = run.abstained == expected.abstain;
    let evidence_accuracy = (run.evidence_claims_checked > 0)
        .then(|| f64::from(run.evidence_claims_correct) / f64::from(run.evidence_claims_checked));
    let mut budget_violations = Vec::new();
    if scenario
        .budgets
        .latency_ms
        .is_some_and(|limit| run.latency_ms > limit)
    {
        budget_violations.push("latency_ms".to_string());
    }
    if scenario
        .budgets
        .context_packet_bytes
        .is_some_and(|limit| run.context_packet_bytes > limit)
    {
        budget_violations.push("context_packet_bytes".to_string());
    }
    let total_tokens = run.input_tokens.saturating_add(run.output_tokens);
    if scenario
        .budgets
        .total_tokens
        .is_some_and(|limit| total_tokens > limit)
    {
        budget_violations.push("total_tokens".to_string());
    }

    let mut gate_failures = Vec::new();
    if !run.task_success {
        gate_failures.push("task_failed".to_string());
    }
    if !identity_correct {
        gate_failures.push("identity_mismatch".to_string());
    }
    if !first_action_correct {
        gate_failures.push("wrong_first_action".to_string());
    }
    if !missing_required_context_keys.is_empty() {
        gate_failures.push("required_context_missing".to_string());
    }
    if !scope_leak_context_keys.is_empty() {
        gate_failures.push("scope_leakage".to_string());
    }
    if !missing_evidence_targets.is_empty() {
        gate_failures.push("required_evidence_missing".to_string());
    }
    if !leaked_markers.is_empty() {
        gate_failures.push("secret_marker_leaked".to_string());
    }
    if !run.stale_context_keys_applied.is_empty() {
        gate_failures.push("stale_memory_influenced_action".to_string());
    }
    if !abstention_correct {
        gate_failures.push("incorrect_abstention".to_string());
    }

    ScoredRun {
        scenario_id: run.scenario_id.clone(),
        arm: run.arm.clone(),
        repetition: run.repetition,
        identity_correct,
        first_action_correct,
        missing_required_context_keys,
        scope_leak_context_keys,
        missing_evidence_targets,
        leaked_markers,
        abstention_correct,
        evidence_accuracy,
        budget_violations,
        gate_failures,
    }
}

fn aggregate_arm(
    manifest: &EvaluationManifest,
    scenario_count: usize,
    arm: &str,
    runs: &[RunRecord],
    scores: &[ScoredRun],
) -> ArmScore {
    let arm_runs = runs.iter().filter(|run| run.arm == arm).collect::<Vec<_>>();
    let arm_scores = scores
        .iter()
        .filter(|score| score.arm == arm)
        .collect::<Vec<_>>();
    let completed = arm_runs.len();
    let rate = |count: usize| {
        if completed == 0 {
            0.0
        } else {
            count as f64 / completed as f64
        }
    };
    let checked = arm_runs
        .iter()
        .map(|run| u64::from(run.evidence_claims_checked))
        .sum::<u64>();
    let correct = arm_runs
        .iter()
        .map(|run| u64::from(run.evidence_claims_correct))
        .sum::<u64>();
    let latencies: Vec<u64> = arm_runs.iter().map(|run| run.latency_ms).collect();
    let packet_sizes: Vec<u64> = arm_runs
        .iter()
        .map(|run| run.context_packet_bytes)
        .collect();
    let token_counts: Vec<u64> = arm_runs
        .iter()
        .map(|run| run.input_tokens.saturating_add(run.output_tokens))
        .collect();

    ArmScore {
        arm: arm.to_string(),
        expected_runs: scenario_count * manifest.repetitions as usize,
        completed_runs: completed,
        clean_runs: arm_scores
            .iter()
            .filter(|score| score.gate_failures.is_empty())
            .count(),
        task_success_rate: rate(arm_runs.iter().filter(|run| run.task_success).count()),
        identity_accuracy: rate(
            arm_scores
                .iter()
                .filter(|score| score.identity_correct)
                .count(),
        ),
        first_action_accuracy: rate(
            arm_scores
                .iter()
                .filter(|score| score.first_action_correct)
                .count(),
        ),
        abstention_accuracy: rate(
            arm_scores
                .iter()
                .filter(|score| score.abstention_correct)
                .count(),
        ),
        scope_leak_runs: arm_scores
            .iter()
            .filter(|score| !score.scope_leak_context_keys.is_empty())
            .count(),
        stale_influence_runs: arm_runs
            .iter()
            .filter(|run| !run.stale_context_keys_applied.is_empty())
            .count(),
        secret_leak_runs: arm_scores
            .iter()
            .filter(|score| !score.leaked_markers.is_empty())
            .count(),
        repeated_failures: arm_runs
            .iter()
            .map(|run| u64::from(run.repeated_failures))
            .sum(),
        user_corrections: arm_runs
            .iter()
            .map(|run| u64::from(run.user_corrections))
            .sum(),
        evidence_accuracy: (checked > 0).then(|| correct as f64 / checked as f64),
        latency_p50_ms: percentile(latencies.clone(), 0.50),
        latency_p95_ms: percentile(latencies, 0.95),
        context_packet_p50_bytes: percentile(packet_sizes.clone(), 0.50),
        context_packet_p95_bytes: percentile(packet_sizes, 0.95),
        total_tokens_p50: percentile(token_counts.clone(), 0.50),
        total_tokens_p95: percentile(token_counts, 0.95),
        budget_violation_count: arm_scores
            .iter()
            .map(|score| score.budget_violations.len())
            .sum(),
    }
}

fn missing_runs(
    manifest: &EvaluationManifest,
    scenarios: &[Scenario],
    seen: &BTreeSet<(String, String, u32)>,
) -> Vec<String> {
    let mut missing = Vec::new();
    for arm in &manifest.arms {
        for scenario in scenarios {
            for repetition in 1..=manifest.repetitions {
                let key = (scenario.id.clone(), arm.id.clone(), repetition);
                if !seen.contains(&key) {
                    missing.push(format!("{}/{}/{}", arm.id, scenario.id, repetition));
                }
            }
        }
    }
    missing
}

fn optional_identity_matches(expected: Option<&str>, actual: Option<&str>, remote: bool) -> bool {
    match expected {
        None => true,
        Some(expected) => actual.is_some_and(|actual| {
            if remote {
                normalize_remote(actual) == normalize_remote(expected)
            } else {
                normalize_label(actual) == normalize_label(expected)
            }
        }),
    }
}

fn normalize_remote(value: &str) -> String {
    let mut normalized = value.trim().trim_end_matches('/').to_ascii_lowercase();
    if let Some(rest) = normalized.strip_prefix("git@") {
        if let Some((host, path)) = rest.split_once(':') {
            normalized = format!("{host}/{path}");
        }
    }
    for prefix in ["https://", "http://", "ssh://", "git://"] {
        if let Some(rest) = normalized.strip_prefix(prefix) {
            normalized = rest.to_string();
            break;
        }
    }
    normalized.trim_end_matches(".git").to_string()
}

fn normalize_label(value: &str) -> String {
    value.nfkc().collect::<String>().trim().to_ascii_lowercase()
}

fn percentile(mut values: Vec<u64>, percentile: f64) -> u64 {
    if values.is_empty() {
        return 0;
    }
    values.sort_unstable();
    let rank = (percentile * values.len() as f64).ceil() as usize;
    values[rank.saturating_sub(1)]
}

fn unique_nonempty<'a>(values: impl Iterator<Item = &'a str>, label: &str) -> EvalResult<()> {
    let mut seen = BTreeSet::new();
    for value in values {
        if value.trim().is_empty() {
            return Err(EvalError::Invalid(format!("{label} must not be empty")));
        }
        if !seen.insert(value) {
            return Err(EvalError::Invalid(format!("duplicate {label}: {value}")));
        }
    }
    Ok(())
}

fn read_json_lines<T: for<'de> Deserialize<'de>>(path: &Path) -> EvalResult<Vec<T>> {
    let file = File::open(path)?;
    let mut values = Vec::new();
    for (index, line) in BufReader::new(file).lines().enumerate() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let value = serde_json::from_str(trimmed).map_err(|error| {
            EvalError::Invalid(format!(
                "{}:{} is not valid JSON: {error}",
                path.display(),
                index + 1
            ))
        })?;
        values.push(value);
    }
    Ok(values)
}

/// Resolve the scenario file path for display or external runners.
#[must_use]
pub fn scenario_path(manifest_path: &Path, manifest: &EvaluationManifest) -> PathBuf {
    manifest_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(&manifest.scenario_file)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> EvaluationManifest {
        EvaluationManifest {
            schema_version: 1,
            suite_id: "test-suite".to_string(),
            scenario_file: "scenarios.jsonl".to_string(),
            arms: vec![
                EvaluationArm {
                    id: "baseline".to_string(),
                    description: "Baseline".to_string(),
                    host: "codex".to_string(),
                    uses_engram: false,
                },
                EvaluationArm {
                    id: "candidate".to_string(),
                    description: "Candidate".to_string(),
                    host: "codex".to_string(),
                    uses_engram: true,
                },
            ],
            repetitions: 1,
            comparisons: Vec::new(),
        }
    }

    fn scenario() -> Scenario {
        Scenario {
            id: "scope".to_string(),
            journey: Journey::WrongScope,
            prompt: "Find the applicable decision".to_string(),
            cwd: Some("/fixture/project-a".to_string()),
            project: None,
            expected: ScenarioExpectations {
                repository_remote: Some("git@github.com:example/project-a.git".to_string()),
                project: Some("project-a".to_string()),
                first_action: Some("inspect_current_project".to_string()),
                required_context_keys: vec!["current".to_string()],
                forbidden_context_keys: vec!["other".to_string()],
                required_evidence_targets: vec!["docs/decision.md".to_string()],
                forbidden_markers: vec!["CANARY_SECRET".to_string()],
                abstain: false,
                ..ScenarioExpectations::default()
            },
            budgets: ScenarioBudgets {
                latency_ms: Some(250),
                context_packet_bytes: Some(4096),
                total_tokens: Some(1000),
            },
            notes: None,
        }
    }

    fn run() -> RunRecord {
        RunRecord {
            scenario_id: "scope".to_string(),
            arm: "candidate".to_string(),
            repetition: 1,
            host: "codex".to_string(),
            host_version: "1.2.3".to_string(),
            model: "gpt-test".to_string(),
            fixture_revision: "fixture-sha".to_string(),
            engram_attestation: Some(RunAttestation {
                cli_version: "0.2.3".to_string(),
                daemon_version: "0.2.3".to_string(),
                schema_version: 1,
                adapter_expected_sha256: "abc".to_string(),
                adapter_actual_sha256: "abc".to_string(),
                status: "matched".to_string(),
            }),
            repository_remote: Some("https://github.com/example/project-a".to_string()),
            project: Some("Project-A".to_string()),
            component: None,
            first_action: Some("inspect_current_project".to_string()),
            returned_context_keys: vec!["current".to_string()],
            applied_context_keys: Vec::new(),
            evidence_targets: vec!["docs/decision.md".to_string()],
            observed_markers: Vec::new(),
            task_success: true,
            abstained: false,
            repeated_failures: 0,
            user_corrections: 0,
            stale_context_keys_applied: Vec::new(),
            evidence_claims_checked: 1,
            evidence_claims_correct: 1,
            latency_ms: 100,
            input_tokens: 500,
            output_tokens: 100,
            context_packet_bytes: 2048,
            notes: None,
        }
    }

    #[test]
    fn scorer_accepts_equivalent_remote_forms_and_reports_missing_runs() {
        let report = score_runs(&manifest(), &[scenario()], &[run()]).unwrap();
        assert!(report.runs[0].gate_failures.is_empty());
        assert!(report.runs[0].identity_correct);
        assert_eq!(report.missing_runs, vec!["baseline/scope/1"]);
        assert_eq!(report.arms[1].clean_runs, 1);
        assert_eq!(report.arms[1].evidence_accuracy, Some(1.0));
    }

    #[test]
    fn scorer_flags_scope_stale_secret_and_budget_failures() {
        let mut bad = run();
        bad.returned_context_keys = vec!["other".to_string()];
        bad.applied_context_keys = vec!["other".to_string()];
        bad.evidence_targets.clear();
        bad.observed_markers = vec!["CANARY_SECRET".to_string()];
        bad.stale_context_keys_applied = vec!["stale".to_string()];
        bad.latency_ms = 251;
        let report = score_runs(&manifest(), &[scenario()], &[bad]).unwrap();
        let score = &report.runs[0];
        assert!(score.gate_failures.contains(&"scope_leakage".to_string()));
        assert!(score
            .gate_failures
            .contains(&"stale_memory_influenced_action".to_string()));
        assert!(score
            .gate_failures
            .contains(&"secret_marker_leaked".to_string()));
        assert_eq!(score.budget_violations, vec!["latency_ms"]);
    }

    #[test]
    fn suite_rejects_overlapping_required_and_forbidden_ids() {
        let mut scenario = scenario();
        scenario.expected.forbidden_context_keys = vec!["current".to_string()];
        let error = validate_suite(&manifest(), &[scenario]).unwrap_err();
        assert!(error.to_string().contains("both requires and forbids"));
    }
}
