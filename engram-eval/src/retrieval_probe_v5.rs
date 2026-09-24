//! Post-algorithm adversarial holdout for the frozen v4 candidate strategy.

use crate::native_runner::write_private_json;
use crate::retrieval_probe::{validate_protocol, RetrievalProtocol, RetrievalQuery};
use crate::retrieval_probe_v2::{
    validate_v2_protocol, BaseProtocolReference, LocalModelAttestation, RetrievalV2Protocol,
    SemanticTiming,
};
use crate::retrieval_probe_v3::{validate_selective_protocol, SelectiveProtocol};
use crate::retrieval_probe_v4::{
    replay_structured_candidate_queries, validate_structured_protocol, CandidateBundleEntry,
    CandidateBundleEvidence, StructuredCandidateProtocol, StructuredCandidateReport,
};
use crate::{EvalError, EvalResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const ADVERSARIAL_PROBE_SCHEMA_VERSION: u32 = 5;
const BUNDLE_ARM: &str = "candidate_bundle_max10";

/// Frozen applicability-boundary gates for explicit candidate discovery.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdversarialQualityGates {
    /// Minimum fraction of quality queries with a relevant bundle candidate.
    pub quality_min_recall: f64,
    /// Minimum macro precision across quality-query candidate bundles.
    pub quality_min_macro_candidate_precision: f64,
    /// Minimum fraction of no-result queries producing an empty bundle.
    pub min_abstention_accuracy: f64,
    /// Maximum mean candidate count across all queries.
    pub max_mean_candidates: f64,
    /// Maximum candidate count for any query.
    pub max_candidates: usize,
    /// Maximum results outside the resolved durable scope.
    pub max_scope_leakage: usize,
    /// Maximum inactive results.
    pub max_stale_results: usize,
    /// Maximum results explicitly judged inapplicable or unsafe.
    pub max_forbidden_results: usize,
    /// Maximum p95 local query-embedding latency.
    pub semantic_query_p95_ms: u64,
}

/// Preregistered post-algorithm adversarial query holdout.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdversarialProtocol {
    /// Protocol schema version.
    pub schema_version: u32,
    /// Stable probe identifier.
    pub probe_id: String,
    /// Byte-frozen v4 candidate protocol.
    pub candidate_protocol: BaseProtocolReference,
    /// Checked-in v4 result decision.
    pub candidate_results: BaseProtocolReference,
    /// Raw v4 report named in the checked-in decision.
    pub candidate_report_sha256: String,
    /// Exact temporal holdout mode.
    pub evaluation_mode: String,
    /// Exact no-tuning policy.
    pub algorithm_policy: String,
    /// Frozen applicability and burden gates.
    pub quality_gates: AdversarialQualityGates,
    /// Query texts and judgments created after v4 was frozen.
    pub queries: Vec<RetrievalQuery>,
}

/// Query-level evidence for applicability and candidate burden.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoundaryQueryEvidence {
    /// Frozen query identifier.
    pub query_id: String,
    /// Adversarial category.
    pub category: String,
    /// Whether the correct outcome is an empty candidate set.
    pub abstain: bool,
    /// Number of surfaced candidates.
    pub candidate_count: usize,
    /// Number of judged-relevant candidates.
    pub relevant_candidate_count: usize,
    /// Relevant candidates divided by all surfaced candidates.
    pub candidate_precision: f64,
    /// Whether at least one relevant candidate appeared.
    pub recalled: bool,
    /// First relevant presentation rank, if any.
    pub first_relevant_rank: Option<usize>,
    /// Explicitly inapplicable candidates that appeared.
    pub forbidden_keys: Vec<String>,
    /// Out-of-scope candidates that appeared.
    pub scope_leakage_keys: Vec<String>,
    /// Inactive candidates that appeared.
    pub stale_keys: Vec<String>,
    /// Candidates with separate, unfused source provenance.
    pub candidates: Vec<CandidateBundleEntry>,
}

/// Aggregate applicability-boundary metrics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoundaryMetrics {
    /// Number of quality queries.
    pub quality_query_count: usize,
    /// Quality queries with a relevant candidate.
    pub quality_recalled_count: usize,
    /// Set recall over quality queries.
    pub quality_recall: f64,
    /// Mean per-query candidate precision over quality queries.
    pub quality_macro_candidate_precision: f64,
    /// Number of no-result queries.
    pub abstention_query_count: usize,
    /// No-result queries with an empty bundle.
    pub abstention_correct_count: usize,
    /// Empty-bundle accuracy over no-result queries.
    pub abstention_accuracy: f64,
    /// Mean candidates across every query.
    pub mean_candidates: f64,
    /// Largest candidate bundle.
    pub max_candidates: usize,
    /// Total out-of-scope candidates.
    pub scope_leakage_count: usize,
    /// Total inactive candidates.
    pub stale_result_count: usize,
    /// Total explicitly forbidden candidates.
    pub forbidden_result_count: usize,
}

/// Reproducible adversarial applicability-boundary report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdversarialReport {
    /// Probe identifier.
    pub probe_id: String,
    /// SHA-256 of exact v5 protocol bytes.
    pub protocol_sha256: String,
    /// SHA-256 of exact v4 candidate protocol bytes.
    pub candidate_protocol_sha256: String,
    /// SHA-256 of exact checked-in v4 results bytes.
    pub candidate_results_sha256: String,
    /// SHA-256 of the raw v4 report declared before this holdout.
    pub candidate_report_sha256: String,
    /// Exact temporal holdout mode.
    pub evaluation_mode: String,
    /// Exact no-tuning policy.
    pub algorithm_policy: String,
    /// Frozen fixture content revision.
    pub fixture_revision: String,
    /// Attested local model identity inherited from v4.
    pub model: LocalModelAttestation,
    /// Number of all records in the isolated store.
    pub corpus_record_count: usize,
    /// Number of active candidate records.
    pub active_record_count: usize,
    /// Local semantic timing evidence.
    pub timing: SemanticTiming,
    /// Frozen gates.
    pub quality_gates: AdversarialQualityGates,
    /// Applicability and burden metrics.
    pub boundary: BoundaryMetrics,
    /// Query-level evidence with lane provenance.
    pub queries: Vec<BoundaryQueryEvidence>,
    /// v4's own diagnostic gate failures on the new query set, recorded but not substituted for v5.
    pub candidate_algorithm_gate_violations: Vec<String>,
    /// Frozen v5 gate violations.
    pub gate_violations: Vec<String>,
    /// True only when every v5 gate passes.
    pub passed: bool,
    /// Claims this probe cannot support.
    pub limitations: Vec<String>,
}

/// Replay the frozen v4 candidate strategy against the frozen adversarial query holdout.
pub async fn run_adversarial_candidate_probe(
    protocol_path: &Path,
    model_snapshot: &Path,
    output: &Path,
) -> EvalResult<AdversarialReport> {
    let protocol_bytes = fs::read(protocol_path)?;
    let protocol: AdversarialProtocol = serde_json::from_slice(&protocol_bytes)?;
    validate_adversarial_protocol(&protocol)?;
    let protocol_sha256 = sha256_bytes(&protocol_bytes);

    let owner = protocol_path.parent().unwrap_or_else(|| Path::new("."));
    let candidate_path = owner.join(&protocol.candidate_protocol.path);
    let candidate_bytes = fs::read(&candidate_path)?;
    let candidate_sha256 = sha256_bytes(&candidate_bytes);
    require_hash(
        "candidate v4 protocol",
        &candidate_sha256,
        &protocol.candidate_protocol.sha256,
    )?;
    let candidate: StructuredCandidateProtocol = serde_json::from_slice(&candidate_bytes)?;
    validate_structured_protocol(&candidate)?;

    let candidate_results_path = owner.join(&protocol.candidate_results.path);
    let candidate_results_bytes = fs::read(&candidate_results_path)?;
    let candidate_results_sha256 = sha256_bytes(&candidate_results_bytes);
    require_hash(
        "candidate v4 results",
        &candidate_results_sha256,
        &protocol.candidate_results.sha256,
    )?;
    let candidate_results_text = String::from_utf8_lossy(&candidate_results_bytes);
    if !candidate_results_text.contains(&protocol.candidate_report_sha256)
        || !candidate_results_text.contains(&protocol.candidate_protocol.sha256)
    {
        return Err(EvalError::Invalid(
            "v4 results do not attest the declared candidate protocol and raw report".to_string(),
        ));
    }
    validate_temporal_holdout(&protocol.queries, &candidate_path, &candidate)?;

    let candidate_report = replay_structured_candidate_queries(
        &candidate_path,
        model_snapshot,
        output,
        &protocol.queries,
    )
    .await?;
    let (boundary, query_evidence) = boundary_metrics(&protocol.queries, &candidate_report)?;
    let gate_violations =
        gate_violations(&protocol.quality_gates, &boundary, &candidate_report.timing);
    let report = AdversarialReport {
        probe_id: protocol.probe_id,
        protocol_sha256,
        candidate_protocol_sha256: candidate_sha256,
        candidate_results_sha256,
        candidate_report_sha256: protocol.candidate_report_sha256,
        evaluation_mode: protocol.evaluation_mode,
        algorithm_policy: protocol.algorithm_policy,
        fixture_revision: candidate_report.fixture_revision,
        model: candidate_report.model,
        corpus_record_count: candidate_report.corpus_record_count,
        active_record_count: candidate_report.active_record_count,
        timing: candidate_report.timing,
        quality_gates: protocol.quality_gates,
        boundary,
        queries: query_evidence,
        candidate_algorithm_gate_violations: candidate_report.gate_violations,
        passed: gate_violations.is_empty(),
        gate_violations,
        limitations: vec![
            "The query texts were frozen after v4, but the synthetic corpus is shared so the probe isolates query and judgment generalization rather than corpus generalization.".to_string(),
            "Forbidden judgments cover preregistered high-risk confounders; they are not exhaustive relevance labels for every corpus item.".to_string(),
            "Candidate precision measures explicit search burden, not automatic-action precision; this probe never applies or executes a candidate.".to_string(),
            "The same attested local embedding model is retained to prevent a model change from obscuring the v4 algorithm boundary.".to_string(),
            "Provider-backed host behavior and native-memory incremental value remain separately approval-gated.".to_string(),
        ],
    };
    write_private_json(&output.join("adversarial-report.json"), &report)?;
    Ok(report)
}

fn validate_adversarial_protocol(protocol: &AdversarialProtocol) -> EvalResult<()> {
    if protocol.schema_version != ADVERSARIAL_PROBE_SCHEMA_VERSION
        || protocol.probe_id.trim().is_empty()
        || protocol.evaluation_mode != "post_algorithm_adversarial_query_holdout"
        || protocol.algorithm_policy != "frozen_v4_no_parameter_tuning"
        || protocol.queries.is_empty()
    {
        return Err(EvalError::Invalid(
            "invalid adversarial protocol header or policy".to_string(),
        ));
    }
    for value in [
        protocol.quality_gates.quality_min_recall,
        protocol.quality_gates.quality_min_macro_candidate_precision,
        protocol.quality_gates.min_abstention_accuracy,
    ] {
        if !(0.0..=1.0).contains(&value) {
            return Err(EvalError::Invalid(
                "adversarial accuracy and precision gates must be in [0, 1]".to_string(),
            ));
        }
    }
    if protocol.quality_gates.max_mean_candidates <= 0.0
        || protocol.quality_gates.max_candidates == 0
        || protocol.candidate_protocol.sha256.len() != 64
        || protocol.candidate_results.sha256.len() != 64
        || protocol.candidate_report_sha256.len() != 64
    {
        return Err(EvalError::Invalid(
            "invalid adversarial budgets or source hashes".to_string(),
        ));
    }
    let quality_count = protocol
        .queries
        .iter()
        .filter(|query| !query.abstain)
        .count();
    let abstention_count = protocol.queries.len() - quality_count;
    if quality_count < 8 || abstention_count < 8 {
        return Err(EvalError::Invalid(
            "v5 requires at least eight quality and eight abstention queries".to_string(),
        ));
    }
    let mut ids = BTreeSet::new();
    for query in &protocol.queries {
        if query.id.trim().is_empty()
            || query.query.trim().is_empty()
            || !ids.insert(query.id.to_ascii_lowercase())
            || query.abstain != query.relevant_keys.is_empty()
        {
            return Err(EvalError::Invalid(format!(
                "invalid adversarial query: {}",
                query.id
            )));
        }
    }
    Ok(())
}

fn validate_temporal_holdout(
    queries: &[RetrievalQuery],
    candidate_path: &Path,
    candidate: &StructuredCandidateProtocol,
) -> EvalResult<()> {
    let v3_path = candidate_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(&candidate.source_protocol.path);
    let v3_bytes = fs::read(&v3_path)?;
    require_hash(
        "v4 source v3 protocol",
        &sha256_bytes(&v3_bytes),
        &candidate.source_protocol.sha256,
    )?;
    let v3: SelectiveProtocol = serde_json::from_slice(&v3_bytes)?;
    validate_selective_protocol(&v3)?;

    let v2_path = v3_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(&v3.calibration_protocol.path);
    let v2_bytes = fs::read(&v2_path)?;
    require_hash(
        "v3 source v2 protocol",
        &sha256_bytes(&v2_bytes),
        &v3.calibration_protocol.sha256,
    )?;
    let v2: RetrievalV2Protocol = serde_json::from_slice(&v2_bytes)?;
    validate_v2_protocol(&v2)?;

    let v1_path = v2_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(&v2.base_protocol.path);
    let v1_bytes = fs::read(&v1_path)?;
    require_hash(
        "v2 source v1 protocol",
        &sha256_bytes(&v1_bytes),
        &v2.base_protocol.sha256,
    )?;
    let v1: RetrievalProtocol = serde_json::from_slice(&v1_bytes)?;
    validate_protocol(&v1)?;

    let prior_ids = v1
        .queries
        .iter()
        .chain(&v2.additional_queries)
        .chain(&v3.held_out_queries)
        .map(|query| query.id.trim().to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    let prior_texts = v1
        .queries
        .iter()
        .chain(&v2.additional_queries)
        .chain(&v3.held_out_queries)
        .map(|query| query.query.trim().to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    for query in queries {
        if prior_ids.contains(&query.id.trim().to_ascii_lowercase())
            || prior_texts.contains(&query.query.trim().to_ascii_lowercase())
        {
            return Err(EvalError::Invalid(format!(
                "v5 query duplicates pre-v4 evaluation data: {}",
                query.id
            )));
        }
    }
    Ok(())
}

fn boundary_metrics(
    queries: &[RetrievalQuery],
    candidate_report: &StructuredCandidateReport,
) -> EvalResult<(BoundaryMetrics, Vec<BoundaryQueryEvidence>)> {
    let mut evidence = Vec::new();
    for query in queries {
        let bundle = candidate_report
            .bundle_evidence
            .iter()
            .find(|item| item.query_id == query.id)
            .ok_or_else(|| {
                EvalError::Invalid(format!("missing bundle evidence for query {}", query.id))
            })?;
        let run = candidate_report
            .runs
            .iter()
            .find(|run| run.arm == BUNDLE_ARM && run.query_id == query.id)
            .ok_or_else(|| {
                EvalError::Invalid(format!("missing bundle score for query {}", query.id))
            })?;
        evidence.push(score_boundary_query(query, bundle, run));
    }

    let quality = evidence
        .iter()
        .filter(|query| !query.abstain)
        .collect::<Vec<_>>();
    let abstention = evidence
        .iter()
        .filter(|query| query.abstain)
        .collect::<Vec<_>>();
    let quality_recalled_count = quality.iter().filter(|query| query.recalled).count();
    let abstention_correct_count = abstention
        .iter()
        .filter(|query| query.candidate_count == 0)
        .count();
    let candidate_counts = evidence
        .iter()
        .map(|query| query.candidate_count)
        .collect::<Vec<_>>();
    let metrics = BoundaryMetrics {
        quality_query_count: quality.len(),
        quality_recalled_count,
        quality_recall: fraction(quality_recalled_count, quality.len()),
        quality_macro_candidate_precision: mean(
            &quality
                .iter()
                .map(|query| query.candidate_precision)
                .collect::<Vec<_>>(),
        ),
        abstention_query_count: abstention.len(),
        abstention_correct_count,
        abstention_accuracy: fraction(abstention_correct_count, abstention.len()),
        mean_candidates: mean(
            &candidate_counts
                .iter()
                .map(|count| *count as f64)
                .collect::<Vec<_>>(),
        ),
        max_candidates: candidate_counts.into_iter().max().unwrap_or_default(),
        scope_leakage_count: evidence
            .iter()
            .map(|query| query.scope_leakage_keys.len())
            .sum(),
        stale_result_count: evidence.iter().map(|query| query.stale_keys.len()).sum(),
        forbidden_result_count: evidence
            .iter()
            .map(|query| query.forbidden_keys.len())
            .sum(),
    };
    Ok((metrics, evidence))
}

fn score_boundary_query(
    query: &RetrievalQuery,
    bundle: &CandidateBundleEvidence,
    run: &crate::retrieval_probe::RetrievalQueryRun,
) -> BoundaryQueryEvidence {
    let relevant = query.relevant_keys.iter().collect::<BTreeSet<_>>();
    let relevant_candidate_count = bundle
        .entries
        .iter()
        .filter(|candidate| relevant.contains(&candidate.key))
        .count();
    BoundaryQueryEvidence {
        query_id: query.id.clone(),
        category: query.category.clone(),
        abstain: query.abstain,
        candidate_count: bundle.entries.len(),
        relevant_candidate_count,
        candidate_precision: if bundle.entries.is_empty() {
            0.0
        } else {
            relevant_candidate_count as f64 / bundle.entries.len() as f64
        },
        recalled: relevant_candidate_count > 0,
        first_relevant_rank: run.first_relevant_rank,
        forbidden_keys: run.forbidden_keys.clone(),
        scope_leakage_keys: run.scope_leakage_keys.clone(),
        stale_keys: run.stale_keys.clone(),
        candidates: bundle.entries.clone(),
    }
}

fn gate_violations(
    gates: &AdversarialQualityGates,
    metrics: &BoundaryMetrics,
    timing: &SemanticTiming,
) -> Vec<String> {
    let mut violations = Vec::new();
    if metrics.quality_recall < gates.quality_min_recall {
        violations.push(format!(
            "quality_recall:{:.4}<{}",
            metrics.quality_recall, gates.quality_min_recall
        ));
    }
    if metrics.quality_macro_candidate_precision < gates.quality_min_macro_candidate_precision {
        violations.push(format!(
            "quality_macro_candidate_precision:{:.4}<{}",
            metrics.quality_macro_candidate_precision, gates.quality_min_macro_candidate_precision
        ));
    }
    if metrics.abstention_accuracy < gates.min_abstention_accuracy {
        violations.push(format!(
            "abstention_accuracy:{:.4}<{}",
            metrics.abstention_accuracy, gates.min_abstention_accuracy
        ));
    }
    if metrics.mean_candidates > gates.max_mean_candidates {
        violations.push(format!(
            "mean_candidates:{:.4}>{}",
            metrics.mean_candidates, gates.max_mean_candidates
        ));
    }
    if metrics.max_candidates > gates.max_candidates {
        violations.push(format!(
            "max_candidates:{}>{}",
            metrics.max_candidates, gates.max_candidates
        ));
    }
    if metrics.scope_leakage_count > gates.max_scope_leakage {
        violations.push(format!(
            "scope_leakage:{}>{}",
            metrics.scope_leakage_count, gates.max_scope_leakage
        ));
    }
    if metrics.stale_result_count > gates.max_stale_results {
        violations.push(format!(
            "stale_results:{}>{}",
            metrics.stale_result_count, gates.max_stale_results
        ));
    }
    if metrics.forbidden_result_count > gates.max_forbidden_results {
        violations.push(format!(
            "forbidden_results:{}>{}",
            metrics.forbidden_result_count, gates.max_forbidden_results
        ));
    }
    if timing.query_embedding_p95_ms > gates.semantic_query_p95_ms as f64 {
        violations.push(format!(
            "semantic_query_p95_ms:{:.4}>{}",
            timing.query_embedding_p95_ms, gates.semantic_query_p95_ms
        ));
    }
    violations
}

fn require_hash(label: &str, observed: &str, expected: &str) -> EvalResult<()> {
    if observed != expected {
        return Err(EvalError::Invalid(format!(
            "{label} hash mismatch: observed {observed}, expected {expected}"
        )));
    }
    Ok(())
}

fn fraction(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        1.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::retrieval_probe::{RetrievalHit, RetrievalQueryRun};

    #[test]
    fn boundary_precision_counts_every_surfaced_candidate() {
        let query = quality_query();
        let bundle = CandidateBundleEvidence {
            query_id: query.id.clone(),
            entries: vec![entry("wrong", 1), entry("relevant", 2), entry("noise", 3)],
        };
        let run = RetrievalQueryRun {
            arm: BUNDLE_ARM.to_string(),
            query_id: query.id.clone(),
            category: query.category.clone(),
            hits: vec![hit("wrong", 1), hit("relevant", 2), hit("noise", 3)],
            first_relevant_rank: Some(2),
            reciprocal_rank: 0.5,
            recalled_at_k: true,
            scope_leakage_keys: Vec::new(),
            stale_keys: Vec::new(),
            forbidden_keys: vec!["wrong".to_string()],
            abstention_false_positive: false,
        };
        let evidence = score_boundary_query(&query, &bundle, &run);
        assert_eq!(evidence.relevant_candidate_count, 1);
        assert_eq!(evidence.candidate_precision, 1.0 / 3.0);
        assert_eq!(evidence.forbidden_keys, vec!["wrong"]);
    }

    #[test]
    fn burden_and_forbidden_results_fail_even_with_full_recall() {
        let gates = AdversarialQualityGates {
            quality_min_recall: 0.9,
            quality_min_macro_candidate_precision: 0.5,
            min_abstention_accuracy: 1.0,
            max_mean_candidates: 4.0,
            max_candidates: 6,
            max_scope_leakage: 0,
            max_stale_results: 0,
            max_forbidden_results: 0,
            semantic_query_p95_ms: 100,
        };
        let metrics = BoundaryMetrics {
            quality_query_count: 8,
            quality_recalled_count: 8,
            quality_recall: 1.0,
            quality_macro_candidate_precision: 0.2,
            abstention_query_count: 8,
            abstention_correct_count: 0,
            abstention_accuracy: 0.0,
            mean_candidates: 5.0,
            max_candidates: 8,
            scope_leakage_count: 0,
            stale_result_count: 0,
            forbidden_result_count: 3,
        };
        let timing = SemanticTiming {
            model_load_ms: 1.0,
            corpus_embedding_ms: 1.0,
            query_embedding_samples: 1,
            query_embedding_p50_ms: 1.0,
            query_embedding_p95_ms: 1.0,
        };
        let violations = gate_violations(&gates, &metrics, &timing);
        assert_eq!(violations.len(), 5);
        assert!(violations
            .iter()
            .any(|violation| violation.starts_with("forbidden_results")));
    }

    fn quality_query() -> RetrievalQuery {
        RetrievalQuery {
            id: "quality".to_string(),
            category: "procedure".to_string(),
            query: "query".to_string(),
            project: "atlas".to_string(),
            cwd: "fixture://atlas/main".to_string(),
            repository_remote: "git@github.com:acme/atlas.git".to_string(),
            relevant_keys: vec!["relevant".to_string()],
            forbidden_keys: vec!["wrong".to_string()],
            abstain: false,
        }
    }

    fn entry(key: &str, rank: usize) -> CandidateBundleEntry {
        CandidateBundleEntry {
            rank,
            key: key.to_string(),
            sources: vec!["lane".to_string()],
        }
    }

    fn hit(key: &str, rank: usize) -> RetrievalHit {
        RetrievalHit {
            rank,
            key: key.to_string(),
            score: 0.0,
        }
    }
}
