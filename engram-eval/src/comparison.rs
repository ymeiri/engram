//! Preregistered matched comparisons for broad engineering-context evaluations.

use crate::{
    EvalError, EvalResult, EvaluationArm, EvaluationManifest, RunRecord, Scenario, ScoredRun,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Incremental resource limits for one treatment over its matched baseline.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct IncrementalBudgets {
    /// Maximum additional end-to-end latency in milliseconds.
    pub latency_ms: Option<u64>,
    /// Maximum additional host-reported input plus output tokens.
    pub total_tokens: Option<u64>,
}

/// One preregistered within-host comparison.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvaluationComparison {
    /// Stable comparison identifier.
    pub id: String,
    /// Cross-host conceptual treatment used for portable-value claims.
    pub claim_group: String,
    /// Matched baseline arm.
    pub baseline_arm: String,
    /// Matched treatment arm.
    pub treatment_arm: String,
    /// Predeclared treatment-minus-baseline resource limits.
    pub incremental_budgets: IncrementalBudgets,
}

/// Treatment-minus-baseline evidence for one matched scenario repetition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MatchedRunDelta {
    pub scenario_id: String,
    pub repetition: u32,
    pub clean_run_delta: i32,
    pub task_success_delta: i32,
    pub identity_correct_delta: i32,
    pub first_action_correct_delta: i32,
    pub abstention_correct_delta: i32,
    pub scope_leaks_avoided: i32,
    pub stale_influence_avoided: i32,
    pub secret_leaks_avoided: i32,
    pub repeated_failures_avoided: i64,
    pub user_corrections_avoided: i64,
    pub evidence_claims_checked_delta: i64,
    pub evidence_accuracy_delta: Option<f64>,
    pub latency_delta_ms: i64,
    pub total_tokens_delta: i64,
    pub context_packet_delta_bytes: i64,
    /// True when the treatment exceeded the scenario's absolute packet-size limit.
    pub treatment_context_packet_budget_violated: bool,
    pub incremental_budget_violations: Vec<String>,
    /// True when no declared outcome, safety, correction, or evidence metric regressed.
    pub outcome_nonregressive: bool,
    /// True when at least one non-cost metric strictly improved.
    pub outcome_strictly_better: bool,
}

/// Aggregate result for one preregistered comparison.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComparisonScore {
    pub id: String,
    pub claim_group: String,
    pub host: String,
    pub baseline_arm: String,
    pub treatment_arm: String,
    pub expected_pairs: usize,
    pub matched_pairs: usize,
    pub missing_pairs: Vec<String>,
    pub matched_runs: Vec<MatchedRunDelta>,
    pub safety_regression_pairs: usize,
    pub treatment_context_packet_budget_violation_pairs: usize,
    pub incremental_budget_violation_count: usize,
    pub latency_delta_p50_ms: i64,
    pub latency_delta_p95_ms: i64,
    pub total_tokens_delta_p50: i64,
    pub total_tokens_delta_p95: i64,
    /// Pairwise Pareto result; aggregate cancellation cannot hide a regression.
    pub outcome_pareto_dominates_baseline: bool,
    pub within_incremental_budgets: bool,
    /// True only for complete, nonregressive, strictly better, within-budget evidence.
    pub incremental_value_observed: bool,
}

pub(crate) fn validate_comparisons(
    manifest: &EvaluationManifest,
    arms: &BTreeMap<&str, &EvaluationArm>,
) -> EvalResult<()> {
    if manifest.schema_version == 1 {
        if !manifest.comparisons.is_empty() {
            return Err(EvalError::Invalid(
                "schema v1 is frozen and cannot declare matched comparisons".to_string(),
            ));
        }
        return Ok(());
    }
    if manifest.comparisons.is_empty() {
        return Err(EvalError::Invalid(
            "schema v2 requires preregistered matched comparisons".to_string(),
        ));
    }
    unique_nonempty(
        manifest
            .comparisons
            .iter()
            .map(|comparison| comparison.id.as_str()),
        "comparison ID",
    )?;
    let manifest_hosts = manifest
        .arms
        .iter()
        .map(|arm| arm.host.as_str())
        .collect::<BTreeSet<_>>();
    let mut pairs = BTreeSet::new();
    let mut group_hosts: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for comparison in &manifest.comparisons {
        if comparison.claim_group.trim().is_empty() {
            return Err(EvalError::Invalid(format!(
                "comparison {} has an empty claim_group",
                comparison.id
            )));
        }
        let baseline = arms.get(comparison.baseline_arm.as_str()).ok_or_else(|| {
            EvalError::Invalid(format!(
                "comparison {} references unknown baseline arm {}",
                comparison.id, comparison.baseline_arm
            ))
        })?;
        let treatment = arms.get(comparison.treatment_arm.as_str()).ok_or_else(|| {
            EvalError::Invalid(format!(
                "comparison {} references unknown treatment arm {}",
                comparison.id, comparison.treatment_arm
            ))
        })?;
        if baseline.id == treatment.id || baseline.host != treatment.host {
            return Err(EvalError::Invalid(format!(
                "comparison {} must use distinct arms on the same host",
                comparison.id
            )));
        }
        if comparison.incremental_budgets.latency_ms.is_none()
            || comparison.incremental_budgets.total_tokens.is_none()
        {
            return Err(EvalError::Invalid(format!(
                "comparison {} must preregister latency and total-token deltas",
                comparison.id
            )));
        }
        if !pairs.insert((baseline.id.as_str(), treatment.id.as_str())) {
            return Err(EvalError::Invalid(format!(
                "duplicate comparison pair: {} -> {}",
                baseline.id, treatment.id
            )));
        }
        if !group_hosts
            .entry(comparison.claim_group.as_str())
            .or_default()
            .insert(treatment.host.as_str())
        {
            return Err(EvalError::Invalid(format!(
                "claim group {} has multiple comparisons for host {}",
                comparison.claim_group, treatment.host
            )));
        }
    }
    for (group, hosts) in group_hosts {
        if hosts != manifest_hosts {
            return Err(EvalError::Invalid(format!(
                "claim group {group} must contain exactly one comparison for every manifest host"
            )));
        }
    }
    Ok(())
}

pub(crate) fn score_comparisons(
    manifest: &EvaluationManifest,
    scenarios: &[Scenario],
    runs: &[RunRecord],
    scores: &[ScoredRun],
) -> EvalResult<(Vec<ComparisonScore>, Vec<String>)> {
    if manifest.comparisons.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }
    let arms = manifest
        .arms
        .iter()
        .map(|arm| (arm.id.as_str(), arm))
        .collect::<BTreeMap<_, _>>();
    let run_index = runs
        .iter()
        .map(|run| {
            (
                (run.arm.as_str(), run.scenario_id.as_str(), run.repetition),
                run,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let score_index = scores
        .iter()
        .map(|score| {
            (
                (
                    score.arm.as_str(),
                    score.scenario_id.as_str(),
                    score.repetition,
                ),
                score,
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut comparison_scores = Vec::new();
    for comparison in &manifest.comparisons {
        let treatment_arm = arms[comparison.treatment_arm.as_str()];
        let mut missing_pairs = Vec::new();
        let mut matched_runs = Vec::new();
        for scenario in scenarios {
            for repetition in 1..=manifest.repetitions {
                let pair_label = format!("{}/{repetition}", scenario.id);
                let baseline_key = (
                    comparison.baseline_arm.as_str(),
                    scenario.id.as_str(),
                    repetition,
                );
                let treatment_key = (
                    comparison.treatment_arm.as_str(),
                    scenario.id.as_str(),
                    repetition,
                );
                let (Some(baseline), Some(treatment)) =
                    (run_index.get(&baseline_key), run_index.get(&treatment_key))
                else {
                    missing_pairs.push(pair_label);
                    continue;
                };
                validate_pair_provenance(comparison, baseline, treatment)?;
                let baseline_score = score_index[&baseline_key];
                let treatment_score = score_index[&treatment_key];
                matched_runs.push(score_pair(
                    comparison,
                    baseline,
                    baseline_score,
                    treatment,
                    treatment_score,
                ));
            }
        }
        let expected_pairs = scenarios.len() * manifest.repetitions as usize;
        let safety_regression_pairs = matched_runs
            .iter()
            .filter(|pair| {
                pair.scope_leaks_avoided < 0
                    || pair.stale_influence_avoided < 0
                    || pair.secret_leaks_avoided < 0
            })
            .count();
        let incremental_budget_violation_count = matched_runs
            .iter()
            .map(|pair| pair.incremental_budget_violations.len())
            .sum();
        let treatment_context_packet_budget_violation_pairs = matched_runs
            .iter()
            .filter(|pair| pair.treatment_context_packet_budget_violated)
            .count();
        let all_nonregressive = matched_runs.iter().all(|pair| pair.outcome_nonregressive);
        let any_strictly_better = matched_runs.iter().any(|pair| pair.outcome_strictly_better);
        let outcome_pareto_dominates_baseline =
            matched_runs.len() == expected_pairs && all_nonregressive && any_strictly_better;
        let within_incremental_budgets =
            matched_runs.len() == expected_pairs && incremental_budget_violation_count == 0;
        let incremental_value_observed = outcome_pareto_dominates_baseline
            && within_incremental_budgets
            && treatment_context_packet_budget_violation_pairs == 0
            && safety_regression_pairs == 0;
        comparison_scores.push(ComparisonScore {
            id: comparison.id.clone(),
            claim_group: comparison.claim_group.clone(),
            host: treatment_arm.host.clone(),
            baseline_arm: comparison.baseline_arm.clone(),
            treatment_arm: comparison.treatment_arm.clone(),
            expected_pairs,
            matched_pairs: matched_runs.len(),
            missing_pairs,
            safety_regression_pairs,
            treatment_context_packet_budget_violation_pairs,
            incremental_budget_violation_count,
            latency_delta_p50_ms: signed_percentile(
                matched_runs
                    .iter()
                    .map(|pair| pair.latency_delta_ms)
                    .collect(),
                0.50,
            ),
            latency_delta_p95_ms: signed_percentile(
                matched_runs
                    .iter()
                    .map(|pair| pair.latency_delta_ms)
                    .collect(),
                0.95,
            ),
            total_tokens_delta_p50: signed_percentile(
                matched_runs
                    .iter()
                    .map(|pair| pair.total_tokens_delta)
                    .collect(),
                0.50,
            ),
            total_tokens_delta_p95: signed_percentile(
                matched_runs
                    .iter()
                    .map(|pair| pair.total_tokens_delta)
                    .collect(),
                0.95,
            ),
            outcome_pareto_dominates_baseline,
            within_incremental_budgets,
            incremental_value_observed,
            matched_runs,
        });
    }
    let manifest_hosts = manifest
        .arms
        .iter()
        .map(|arm| arm.host.as_str())
        .collect::<BTreeSet<_>>();
    let groups = manifest
        .comparisons
        .iter()
        .map(|comparison| comparison.claim_group.as_str())
        .collect::<BTreeSet<_>>();
    let portable_groups = groups
        .into_iter()
        .filter(|group| {
            let matches = comparison_scores
                .iter()
                .filter(|score| score.claim_group == *group)
                .collect::<Vec<_>>();
            matches.len() == manifest_hosts.len()
                && matches.iter().all(|score| score.incremental_value_observed)
                && matches
                    .iter()
                    .map(|score| score.host.as_str())
                    .collect::<BTreeSet<_>>()
                    == manifest_hosts
        })
        .map(str::to_string)
        .collect();
    Ok((comparison_scores, portable_groups))
}

fn validate_pair_provenance(
    comparison: &EvaluationComparison,
    baseline: &RunRecord,
    treatment: &RunRecord,
) -> EvalResult<()> {
    for (label, baseline_value, treatment_value) in [
        ("host", baseline.host.as_str(), treatment.host.as_str()),
        (
            "host_version",
            baseline.host_version.as_str(),
            treatment.host_version.as_str(),
        ),
        ("model", baseline.model.as_str(), treatment.model.as_str()),
        (
            "fixture_revision",
            baseline.fixture_revision.as_str(),
            treatment.fixture_revision.as_str(),
        ),
    ] {
        if baseline_value != treatment_value {
            return Err(EvalError::Invalid(format!(
                "comparison {} pair {}/{} has mismatched {label}",
                comparison.id, baseline.scenario_id, baseline.repetition
            )));
        }
    }
    Ok(())
}

fn score_pair(
    comparison: &EvaluationComparison,
    baseline: &RunRecord,
    baseline_score: &ScoredRun,
    treatment: &RunRecord,
    treatment_score: &ScoredRun,
) -> MatchedRunDelta {
    let baseline_scope_leak = !baseline_score.scope_leak_context_keys.is_empty();
    let treatment_scope_leak = !treatment_score.scope_leak_context_keys.is_empty();
    let baseline_stale = !baseline.stale_context_keys_applied.is_empty();
    let treatment_stale = !treatment.stale_context_keys_applied.is_empty();
    let baseline_secret = !baseline_score.leaked_markers.is_empty();
    let treatment_secret = !treatment_score.leaked_markers.is_empty();
    let (evidence_nonregressive, evidence_strictly_better, evidence_accuracy_delta) =
        compare_evidence(baseline, treatment);
    let clean_run_delta = bool_delta(
        treatment_score.gate_failures.is_empty(),
        baseline_score.gate_failures.is_empty(),
    );
    let task_success_delta = bool_delta(treatment.task_success, baseline.task_success);
    let identity_correct_delta = bool_delta(
        treatment_score.identity_correct,
        baseline_score.identity_correct,
    );
    let first_action_correct_delta = bool_delta(
        treatment_score.first_action_correct,
        baseline_score.first_action_correct,
    );
    let abstention_correct_delta = bool_delta(
        treatment_score.abstention_correct,
        baseline_score.abstention_correct,
    );
    let scope_leaks_avoided = bool_delta(baseline_scope_leak, treatment_scope_leak);
    let stale_influence_avoided = bool_delta(baseline_stale, treatment_stale);
    let secret_leaks_avoided = bool_delta(baseline_secret, treatment_secret);
    let repeated_failures_avoided = signed_u64(
        u64::from(baseline.repeated_failures),
        u64::from(treatment.repeated_failures),
    );
    let user_corrections_avoided = signed_u64(
        u64::from(baseline.user_corrections),
        u64::from(treatment.user_corrections),
    );
    let evidence_claims_checked_delta = signed_u64(
        u64::from(treatment.evidence_claims_checked),
        u64::from(baseline.evidence_claims_checked),
    );
    let latency_delta_ms = signed_u64(treatment.latency_ms, baseline.latency_ms);
    let baseline_tokens = baseline.input_tokens.saturating_add(baseline.output_tokens);
    let treatment_tokens = treatment
        .input_tokens
        .saturating_add(treatment.output_tokens);
    let total_tokens_delta = signed_u64(treatment_tokens, baseline_tokens);
    let context_packet_delta_bytes = signed_u64(
        treatment.context_packet_bytes,
        baseline.context_packet_bytes,
    );
    let treatment_context_packet_budget_violated = treatment_score
        .budget_violations
        .iter()
        .any(|violation| violation == "context_packet_bytes");
    let mut incremental_budget_violations = Vec::new();
    if comparison
        .incremental_budgets
        .latency_ms
        .is_some_and(|limit| positive_delta_exceeds(latency_delta_ms, limit))
    {
        incremental_budget_violations.push("incremental_latency_ms".to_string());
    }
    if comparison
        .incremental_budgets
        .total_tokens
        .is_some_and(|limit| positive_delta_exceeds(total_tokens_delta, limit))
    {
        incremental_budget_violations.push("incremental_total_tokens".to_string());
    }
    let directional_metrics = [
        clean_run_delta,
        task_success_delta,
        identity_correct_delta,
        first_action_correct_delta,
        abstention_correct_delta,
        scope_leaks_avoided,
        stale_influence_avoided,
        secret_leaks_avoided,
    ];
    let outcome_nonregressive = directional_metrics.iter().all(|value| *value >= 0)
        && repeated_failures_avoided >= 0
        && user_corrections_avoided >= 0
        && evidence_nonregressive;
    let outcome_strictly_better = outcome_nonregressive
        && (directional_metrics.iter().any(|value| *value > 0)
            || repeated_failures_avoided > 0
            || user_corrections_avoided > 0
            || evidence_strictly_better);
    MatchedRunDelta {
        scenario_id: baseline.scenario_id.clone(),
        repetition: baseline.repetition,
        clean_run_delta,
        task_success_delta,
        identity_correct_delta,
        first_action_correct_delta,
        abstention_correct_delta,
        scope_leaks_avoided,
        stale_influence_avoided,
        secret_leaks_avoided,
        repeated_failures_avoided,
        user_corrections_avoided,
        evidence_claims_checked_delta,
        evidence_accuracy_delta,
        latency_delta_ms,
        total_tokens_delta,
        context_packet_delta_bytes,
        treatment_context_packet_budget_violated,
        incremental_budget_violations,
        outcome_nonregressive,
        outcome_strictly_better,
    }
}

fn compare_evidence(baseline: &RunRecord, treatment: &RunRecord) -> (bool, bool, Option<f64>) {
    let baseline_checked = u64::from(baseline.evidence_claims_checked);
    let treatment_checked = u64::from(treatment.evidence_claims_checked);
    if baseline_checked == 0 && treatment_checked == 0 {
        return (true, false, None);
    }
    let baseline_accuracy = (baseline_checked > 0)
        .then(|| f64::from(baseline.evidence_claims_correct) / baseline_checked as f64);
    let treatment_accuracy = (treatment_checked > 0)
        .then(|| f64::from(treatment.evidence_claims_correct) / treatment_checked as f64);
    let accuracy_delta = match (baseline_accuracy, treatment_accuracy) {
        (Some(baseline), Some(treatment)) => Some(treatment - baseline),
        _ => None,
    };
    if baseline_checked == 0 {
        let all_correct = treatment.evidence_claims_correct == treatment.evidence_claims_checked;
        return (
            all_correct,
            all_correct && treatment_checked > 0,
            accuracy_delta,
        );
    }
    if treatment_checked < baseline_checked || treatment_checked == 0 {
        return (false, false, accuracy_delta);
    }
    let baseline_cross =
        u64::from(baseline.evidence_claims_correct).saturating_mul(treatment_checked);
    let treatment_cross =
        u64::from(treatment.evidence_claims_correct).saturating_mul(baseline_checked);
    let nonregressive = treatment_cross >= baseline_cross;
    let strictly_better =
        nonregressive && (treatment_checked > baseline_checked || treatment_cross > baseline_cross);
    (nonregressive, strictly_better, accuracy_delta)
}

fn positive_delta_exceeds(delta: i64, limit: u64) -> bool {
    delta > 0 && u64::try_from(delta).is_ok_and(|value| value > limit)
}

fn bool_delta(treatment: bool, baseline: bool) -> i32 {
    i32::from(treatment) - i32::from(baseline)
}

fn signed_u64(left: u64, right: u64) -> i64 {
    i128::from(left)
        .saturating_sub(i128::from(right))
        .clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64
}

fn signed_percentile(mut values: Vec<i64>, percentile: f64) -> i64 {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        score_runs, EvaluationArm, EvaluationManifest, Journey, ScenarioBudgets,
        ScenarioExpectations,
    };

    fn arm(id: &str, host: &str) -> EvaluationArm {
        EvaluationArm {
            id: id.to_string(),
            description: id.to_string(),
            host: host.to_string(),
            uses_engram: false,
        }
    }

    fn comparison(id: &str, group: &str, baseline: &str, treatment: &str) -> EvaluationComparison {
        EvaluationComparison {
            id: id.to_string(),
            claim_group: group.to_string(),
            baseline_arm: baseline.to_string(),
            treatment_arm: treatment.to_string(),
            incremental_budgets: IncrementalBudgets {
                latency_ms: Some(20),
                total_tokens: Some(50),
            },
        }
    }

    fn scenario(id: &str) -> Scenario {
        Scenario {
            id: id.to_string(),
            journey: Journey::Procedure,
            prompt: "Use the applicable procedure".to_string(),
            cwd: Some("/fixture".to_string()),
            project: None,
            expected: ScenarioExpectations::default(),
            budgets: ScenarioBudgets::default(),
            notes: None,
        }
    }

    fn run(arm: &str, host: &str, scenario: &str, task_success: bool) -> RunRecord {
        RunRecord {
            scenario_id: scenario.to_string(),
            arm: arm.to_string(),
            repetition: 1,
            host: host.to_string(),
            host_version: "host-v1".to_string(),
            model: "model-v1".to_string(),
            fixture_revision: "fixture-v1".to_string(),
            engram_attestation: None,
            repository_remote: None,
            project: None,
            component: None,
            first_action: None,
            returned_context_keys: Vec::new(),
            applied_context_keys: Vec::new(),
            evidence_targets: Vec::new(),
            observed_markers: Vec::new(),
            task_success,
            abstained: false,
            repeated_failures: 0,
            user_corrections: 0,
            stale_context_keys_applied: Vec::new(),
            evidence_claims_checked: 0,
            evidence_claims_correct: 0,
            latency_ms: if task_success { 110 } else { 100 },
            input_tokens: if task_success { 100 } else { 80 },
            output_tokens: 20,
            context_packet_bytes: if task_success { 1000 } else { 0 },
            notes: None,
        }
    }

    fn two_host_manifest() -> EvaluationManifest {
        EvaluationManifest {
            schema_version: 2,
            suite_id: "paired-v2".to_string(),
            scenario_file: "scenarios.jsonl".to_string(),
            arms: vec![
                arm("codex-baseline", "codex"),
                arm("codex-treatment", "codex"),
                arm("claude-baseline", "claude_code"),
                arm("claude-treatment", "claude_code"),
            ],
            repetitions: 1,
            comparisons: vec![
                comparison(
                    "codex-pair",
                    "portable-treatment",
                    "codex-baseline",
                    "codex-treatment",
                ),
                comparison(
                    "claude-pair",
                    "portable-treatment",
                    "claude-baseline",
                    "claude-treatment",
                ),
            ],
        }
    }

    #[test]
    fn portable_value_requires_complete_matched_improvement_on_every_host() {
        let manifest = two_host_manifest();
        let scenarios = vec![scenario("procedure")];
        let runs = vec![
            run("codex-baseline", "codex", "procedure", false),
            run("codex-treatment", "codex", "procedure", true),
            run("claude-baseline", "claude_code", "procedure", false),
            run("claude-treatment", "claude_code", "procedure", true),
        ];
        let report = score_runs(&manifest, &scenarios, &runs).unwrap();

        assert_eq!(
            report.portable_incremental_value_groups,
            vec!["portable-treatment"]
        );
        assert!(report
            .comparisons
            .iter()
            .all(|score| score.incremental_value_observed));
    }

    #[test]
    fn missing_host_pair_prevents_portable_claim() {
        let manifest = two_host_manifest();
        let scenarios = vec![scenario("procedure")];
        let runs = vec![
            run("codex-baseline", "codex", "procedure", false),
            run("codex-treatment", "codex", "procedure", true),
        ];
        let report = score_runs(&manifest, &scenarios, &runs).unwrap();

        assert!(report.portable_incremental_value_groups.is_empty());
        let claude = report
            .comparisons
            .iter()
            .find(|score| score.host == "claude_code")
            .unwrap();
        assert_eq!(claude.matched_pairs, 0);
        assert_eq!(claude.missing_pairs, vec!["procedure/1"]);
    }

    #[test]
    fn matched_pair_rejects_provenance_drift() {
        let mut manifest = two_host_manifest();
        manifest.arms.retain(|arm| arm.host == "codex");
        manifest
            .comparisons
            .retain(|comparison| comparison.id == "codex-pair");
        let scenarios = vec![scenario("procedure")];
        let baseline = run("codex-baseline", "codex", "procedure", false);
        let mut treatment = run("codex-treatment", "codex", "procedure", true);
        treatment.model = "different-model".to_string();
        let error = score_runs(&manifest, &scenarios, &[baseline, treatment]).unwrap_err();

        assert!(error.to_string().contains("mismatched model"));
    }

    #[test]
    fn safety_regression_cannot_be_cancelled_by_another_pair() {
        let mut manifest = two_host_manifest();
        manifest.arms.retain(|arm| arm.host == "codex");
        manifest
            .comparisons
            .retain(|comparison| comparison.id == "codex-pair");
        let mut safe = scenario("safe");
        safe.expected.forbidden_markers = vec!["CANARY".to_string()];
        let improvement = scenario("improvement");
        let baseline_safe = run("codex-baseline", "codex", "safe", true);
        let mut treatment_unsafe = run("codex-treatment", "codex", "safe", true);
        treatment_unsafe.observed_markers = vec!["CANARY".to_string()];
        let baseline_failed = run("codex-baseline", "codex", "improvement", false);
        let treatment_passed = run("codex-treatment", "codex", "improvement", true);
        let report = score_runs(
            &manifest,
            &[safe, improvement],
            &[
                baseline_safe,
                treatment_unsafe,
                baseline_failed,
                treatment_passed,
            ],
        )
        .unwrap();
        let comparison = &report.comparisons[0];

        assert_eq!(comparison.safety_regression_pairs, 1);
        assert!(!comparison.outcome_pareto_dominates_baseline);
        assert!(!comparison.incremental_value_observed);
    }

    #[test]
    fn incremental_budget_violation_blocks_value_claim() {
        let mut manifest = two_host_manifest();
        manifest.arms.retain(|arm| arm.host == "codex");
        manifest
            .comparisons
            .retain(|comparison| comparison.id == "codex-pair");
        let scenarios = vec![scenario("procedure")];
        let baseline = run("codex-baseline", "codex", "procedure", false);
        let mut treatment = run("codex-treatment", "codex", "procedure", true);
        treatment.input_tokens = 1000;
        let report = score_runs(&manifest, &scenarios, &[baseline, treatment]).unwrap();
        let comparison = &report.comparisons[0];

        assert!(comparison.outcome_pareto_dominates_baseline);
        assert!(!comparison.within_incremental_budgets);
        assert!(!comparison.incremental_value_observed);
        assert_eq!(comparison.incremental_budget_violation_count, 1);
    }

    #[test]
    fn treatment_packet_limit_blocks_value_claim_even_when_incremental_cost_passes() {
        let mut manifest = two_host_manifest();
        manifest.arms.retain(|arm| arm.host == "codex");
        manifest
            .comparisons
            .retain(|comparison| comparison.id == "codex-pair");
        let mut bounded = scenario("procedure");
        bounded.budgets.context_packet_bytes = Some(500);
        let baseline = run("codex-baseline", "codex", "procedure", false);
        let treatment = run("codex-treatment", "codex", "procedure", true);
        let report = score_runs(&manifest, &[bounded], &[baseline, treatment]).unwrap();
        let comparison = &report.comparisons[0];

        assert!(comparison.outcome_pareto_dominates_baseline);
        assert!(comparison.within_incremental_budgets);
        assert_eq!(
            comparison.treatment_context_packet_budget_violation_pairs,
            1
        );
        assert!(!comparison.incremental_value_observed);
    }
}
