//! Held-out selective-application evaluation over scoped dense memory candidates.

use crate::fixture::materialize_engineering_context_fixture;
use crate::retrieval_probe::{
    add_corpus_expansions, aggregate_arm, require_empty_target, resolve_fixture_uri,
    resolved_scope_matches, score_query_run, validate_judgments, validate_protocol,
    RetrievalArmMetrics, RetrievalHit, RetrievalProtocol, RetrievalQuery, RetrievalQueryRun,
};
use crate::retrieval_probe_v2::{
    attest_model_snapshot, dense_hits, memory_document_text, opaque_cues, validate_v2_protocol,
    BaseProtocolReference, LocalModelAttestation, RetrievalV2Protocol, SemanticTiming,
};
use crate::seed::open_seeded_engineering_context_fixture;
use crate::{EvalError, EvalResult};
use engram_core::memory::{MemoryItem, MemoryStatus};
use engram_embed::{EmbedConfig, Embedder};
use engram_index::{MemoryService, SearchOptions, SearchService};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::time::Instant;

const SELECTIVE_PROBE_SCHEMA_VERSION: u32 = 3;
const CURRENT_ARM: &str = "current_lexical";
const DENSE_CANDIDATE_ARM: &str = "dense_candidate_top5";
const AUTO_APPLY_ARM: &str = "conservative_auto_apply";

/// Frozen conservative selector parameters calibrated on retrieval v2.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SelectiveRanking {
    /// Absolute top dense similarity that permits automatic application.
    pub high_similarity: f64,
    /// Top-1 versus Top-2 margin that permits automatic application.
    pub high_margin: f64,
    /// Minimum top similarity before the margin rule applies.
    pub margin_min_similarity: f64,
    /// Minimum top similarity before lexical/dense agreement applies.
    pub agreement_min_similarity: f64,
    /// Minimum direct query/document term overlap for the agreement rule.
    pub agreement_min_overlap_terms: usize,
    /// Exact lifecycle rule name.
    pub lifecycle_policy: String,
    /// Exact opaque cue rule name.
    pub opaque_cue_policy: String,
}

/// Frozen selective-application gates.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SelectiveQualityGates {
    /// Minimum dense candidate recall at K.
    pub candidate_min_recall_at_k: f64,
    /// Minimum precision across every automatic application.
    pub auto_apply_min_precision: f64,
    /// Minimum fraction of quality queries automatically applied.
    pub auto_apply_min_quality_coverage: f64,
    /// Maximum wrong automatic applications, including no-result queries.
    pub auto_apply_max_unsafe_applications: usize,
    /// Maximum out-of-scope results for any arm.
    pub max_scope_leakage: usize,
    /// Maximum inactive results for any arm.
    pub max_stale_results: usize,
    /// Maximum forbidden results for any arm.
    pub max_forbidden_results: usize,
    /// Maximum p95 query-embedding latency.
    pub semantic_query_p95_ms: u64,
}

/// Frozen v3 held-out protocol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SelectiveProtocol {
    /// Protocol schema version.
    pub schema_version: u32,
    /// Stable probe identifier.
    pub probe_id: String,
    /// Frozen v2 calibration protocol reference.
    pub calibration_protocol: BaseProtocolReference,
    /// Exact v2 result used to derive the selector.
    pub calibration_report_sha256: String,
    /// Rank cutoff.
    pub top_k: usize,
    /// Production lexical threshold.
    pub current_min_score: f32,
    /// Query embedding repetitions.
    pub repetitions: usize,
    /// Ordered arm IDs.
    pub arms: Vec<String>,
    /// Conservative selector parameters.
    pub selector: SelectiveRanking,
    /// Frozen quality and safety gates.
    pub quality_gates: SelectiveQualityGates,
    /// Queries disjoint from the v2 calibration query texts.
    pub held_out_queries: Vec<RetrievalQuery>,
}

/// Selective-application metrics distinct from candidate recall.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SelectiveMetrics {
    /// Quality queries eligible for a correct application.
    pub quality_query_count: usize,
    /// Quality queries on which the selector returned one result.
    pub quality_application_count: usize,
    /// Fraction of quality queries automatically applied.
    pub quality_coverage: f64,
    /// Total queries on which the selector returned one result.
    pub total_application_count: usize,
    /// Returned results that were the judged relevant item.
    pub correct_application_count: usize,
    /// Fraction of all applications that were correct.
    pub application_precision: f64,
    /// Wrong applications on quality, no-result, or wrong-scope queries.
    pub unsafe_application_count: usize,
}

/// Reproducible held-out selective-application report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SelectiveReport {
    /// Probe identifier.
    pub probe_id: String,
    /// SHA-256 of exact v3 protocol bytes.
    pub protocol_sha256: String,
    /// SHA-256 of exact v2 calibration protocol bytes.
    pub calibration_protocol_sha256: String,
    /// Frozen v2 report hash declared as calibration evidence.
    pub calibration_report_sha256: String,
    /// Frozen fixture content revision.
    pub fixture_revision: String,
    /// Attested model identity.
    pub model: LocalModelAttestation,
    /// Observed model file hashes.
    pub observed_model_files: BTreeMap<String, String>,
    /// Number of all records in the isolated store.
    pub corpus_record_count: usize,
    /// Number of active records embedded.
    pub active_record_count: usize,
    /// Number of held-out queries.
    pub query_count: usize,
    /// Semantic latency evidence.
    pub timing: SemanticTiming,
    /// Aggregate retrieval metrics by arm.
    pub arms: Vec<RetrievalArmMetrics>,
    /// Automatic-application metrics.
    pub selective: SelectiveMetrics,
    /// Query-level evidence.
    pub runs: Vec<RetrievalQueryRun>,
    /// Frozen gate violations.
    pub gate_violations: Vec<String>,
    /// True only when every frozen gate passes.
    pub passed: bool,
    /// Claims this probe cannot support.
    pub limitations: Vec<String>,
}

/// Run the frozen held-out selective-application comparison.
pub async fn run_selective_retrieval_probe(
    protocol_path: &Path,
    model_snapshot: &Path,
    output: &Path,
) -> EvalResult<SelectiveReport> {
    let protocol_bytes = fs::read(protocol_path)?;
    let protocol: SelectiveProtocol = serde_json::from_slice(&protocol_bytes)?;
    validate_selective_protocol(&protocol)?;
    let protocol_sha256 = sha256_bytes(&protocol_bytes);

    let calibration_path = protocol_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(&protocol.calibration_protocol.path);
    let calibration_bytes = fs::read(&calibration_path)?;
    let calibration_sha256 = sha256_bytes(&calibration_bytes);
    if calibration_sha256 != protocol.calibration_protocol.sha256 {
        return Err(EvalError::Invalid(format!(
            "calibration protocol hash mismatch: observed {calibration_sha256}, expected {}",
            protocol.calibration_protocol.sha256
        )));
    }
    let calibration: RetrievalV2Protocol = serde_json::from_slice(&calibration_bytes)?;
    validate_v2_protocol(&calibration)?;
    if calibration.top_k != protocol.top_k
        || calibration.current_min_score != protocol.current_min_score
        || calibration.repetitions != protocol.repetitions
    {
        return Err(EvalError::Invalid(
            "v3 rank, lexical, and repetition settings must match v2 calibration".to_string(),
        ));
    }

    let base_path = calibration_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(&calibration.base_protocol.path);
    let base_bytes = fs::read(&base_path)?;
    let base_sha256 = sha256_bytes(&base_bytes);
    if base_sha256 != calibration.base_protocol.sha256 {
        return Err(EvalError::Invalid(format!(
            "base protocol hash mismatch: observed {base_sha256}, expected {}",
            calibration.base_protocol.sha256
        )));
    }
    let base: RetrievalProtocol = serde_json::from_slice(&base_bytes)?;
    validate_protocol(&base)?;
    validate_disjoint_queries(&protocol.held_out_queries, &base, &calibration)?;

    let observed_model_files = attest_model_snapshot(&calibration.model, model_snapshot)?;
    let model_start = Instant::now();
    let embedder =
        Embedder::from_local_snapshot(EmbedConfig::default(), model_snapshot).map_err(invalid)?;
    let model_load_ms = elapsed_ms(model_start);

    require_empty_target(output)?;
    let fixture_root = output.join("fixture");
    let data_dir = output.join("data");
    let layout = materialize_engineering_context_fixture(&fixture_root)?;
    let (seed, db) =
        open_seeded_engineering_context_fixture(&fixture_root.join("fixture-map.json"), &data_dir)
            .await?;
    let memory = MemoryService::new(db.clone());
    memory.init_schema().await.map_err(invalid)?;
    let mut key_by_id = seed
        .memory
        .iter()
        .map(|(key, id)| (id.clone(), key.clone()))
        .collect::<BTreeMap<_, _>>();
    add_corpus_expansions(&memory, &base.corpus_expansions, &mut key_by_id, &base_path).await?;

    let all_items = memory.list_memory(None, None).await.map_err(invalid)?;
    let active_items = all_items
        .iter()
        .filter(|item| item.status == MemoryStatus::Active)
        .cloned()
        .collect::<Vec<_>>();
    let judgment_protocol = RetrievalProtocol {
        queries: protocol.held_out_queries.clone(),
        ..base
    };
    validate_judgments(&judgment_protocol, &all_items, &key_by_id)?;
    let item_by_key = all_items
        .iter()
        .filter_map(|item| {
            key_by_id
                .get(&item.id.to_string())
                .map(|key| (key.clone(), item.clone()))
        })
        .collect::<BTreeMap<_, _>>();

    let mut corpus = active_items
        .iter()
        .filter_map(|item| {
            key_by_id
                .get(&item.id.to_string())
                .map(|key| (key.clone(), item))
        })
        .collect::<Vec<_>>();
    corpus.sort_by(|left, right| left.0.cmp(&right.0));
    let corpus_texts = corpus
        .iter()
        .map(|(_, item)| memory_document_text(item))
        .collect::<Vec<_>>();
    let corpus_refs = corpus_texts.iter().map(String::as_str).collect::<Vec<_>>();
    let corpus_start = Instant::now();
    let corpus_vectors = embedder.embed_batch(&corpus_refs).map_err(invalid)?;
    let corpus_embedding_ms = elapsed_ms(corpus_start);
    let vectors_by_key = corpus
        .iter()
        .zip(corpus_vectors)
        .map(|((key, _), vector)| (key.clone(), vector))
        .collect::<BTreeMap<_, _>>();

    let search = SearchService::new(db);
    let mut query_latencies = Vec::new();
    let mut runs = Vec::new();
    for query in &protocol.held_out_queries {
        let cwd = resolve_fixture_uri(&layout, &query.cwd)?;
        let current = search
            .search_local_memory(
                &query.query,
                protocol.top_k,
                Some(protocol.current_min_score),
                &SearchOptions {
                    project: Some(query.project.clone()),
                    cwd: Some(cwd.display().to_string()),
                },
                None,
            )
            .await
            .map_err(invalid)?;
        let current_hits = current
            .into_iter()
            .enumerate()
            .map(|(index, result)| RetrievalHit {
                rank: index + 1,
                key: key_by_id
                    .get(&result.id)
                    .cloned()
                    .unwrap_or_else(|| format!("unmapped:{}", result.id)),
                score: f64::from(result.score),
            })
            .collect::<Vec<_>>();
        runs.push(score_query_run(
            CURRENT_ARM,
            query,
            current_hits.clone(),
            &item_by_key,
        ));

        let mut query_vector = None;
        for _ in 0..protocol.repetitions {
            let started = Instant::now();
            let vector = embedder.embed(&query.query).map_err(invalid)?;
            query_latencies.push(elapsed_ms(started));
            query_vector.get_or_insert(vector);
        }
        let query_vector = query_vector.ok_or_else(|| {
            EvalError::Invalid("v3 repetitions unexpectedly produced no embedding".to_string())
        })?;
        let scoped_active = active_items
            .iter()
            .filter(|item| resolved_scope_matches(item, query))
            .collect::<Vec<_>>();
        let candidates = dense_hits(
            &query_vector,
            &scoped_active,
            &key_by_id,
            &vectors_by_key,
            protocol.top_k,
        );
        runs.push(score_query_run(
            DENSE_CANDIDATE_ARM,
            query,
            candidates.clone(),
            &item_by_key,
        ));

        let selected = select_for_auto_apply(
            query,
            &current_hits,
            &candidates,
            &scoped_active,
            &all_items,
            &key_by_id,
            &item_by_key,
            &protocol.selector,
        );
        runs.push(score_query_run(
            AUTO_APPLY_ARM,
            query,
            selected,
            &item_by_key,
        ));
    }

    query_latencies.sort_by(f64::total_cmp);
    let timing = SemanticTiming {
        model_load_ms,
        corpus_embedding_ms,
        query_embedding_samples: query_latencies.len(),
        query_embedding_p50_ms: percentile(&query_latencies, 0.50),
        query_embedding_p95_ms: percentile(&query_latencies, 0.95),
    };
    let arms = protocol
        .arms
        .iter()
        .map(|arm| aggregate_arm(arm, &protocol.held_out_queries, &runs))
        .collect::<Vec<_>>();
    let selective = selective_metrics(&protocol.held_out_queries, &runs);
    let gate_violations = gate_violations(&protocol.quality_gates, &arms, &selective, &timing);
    let report = SelectiveReport {
        probe_id: protocol.probe_id,
        protocol_sha256,
        calibration_protocol_sha256: calibration_sha256,
        calibration_report_sha256: protocol.calibration_report_sha256,
        fixture_revision: seed.fixture_revision,
        model: calibration.model,
        observed_model_files,
        corpus_record_count: all_items.len(),
        active_record_count: active_items.len(),
        query_count: protocol.held_out_queries.len(),
        timing,
        arms,
        selective,
        runs,
        passed: gate_violations.is_empty(),
        gate_violations,
        limitations: vec![
            "The selector is a frozen deterministic baseline, not a learned applicability model.".to_string(),
            "Only query texts are held out; the synthetic memory corpus is shared with calibration to isolate selective-prediction behavior.".to_string(),
            "Deferral means a candidate may remain visible to explicit search, not that the system proved no related context exists.".to_string(),
            "No reranker or entailment model was cached locally, so this protocol neither downloads nor evaluates one.".to_string(),
            "Host-native task application remains a separately approval-gated evaluation.".to_string(),
        ],
    };
    fs::write(
        output.join("retrieval-report.json"),
        serde_json::to_string_pretty(&report)? + "\n",
    )?;
    Ok(report)
}

pub(crate) fn validate_selective_protocol(protocol: &SelectiveProtocol) -> EvalResult<()> {
    if protocol.schema_version != SELECTIVE_PROBE_SCHEMA_VERSION
        || protocol.probe_id.trim().is_empty()
        || protocol.top_k == 0
        || protocol.repetitions == 0
        || protocol.held_out_queries.is_empty()
    {
        return Err(EvalError::Invalid(
            "invalid selective retrieval protocol header".to_string(),
        ));
    }
    let expected_arms = [CURRENT_ARM, DENSE_CANDIDATE_ARM, AUTO_APPLY_ARM];
    if protocol.arms.iter().map(String::as_str).collect::<Vec<_>>() != expected_arms {
        return Err(EvalError::Invalid(format!(
            "v3 arms must be exactly {}",
            expected_arms.join(", ")
        )));
    }
    if protocol.selector.lifecycle_policy
        != "all_query_terms_match_inactive_then_follow_explicit_successor"
        || protocol.selector.opaque_cue_policy != "require_exact_active_scoped_match"
        || protocol.selector.agreement_min_overlap_terms == 0
    {
        return Err(EvalError::Invalid(
            "invalid selective rule names or overlap requirement".to_string(),
        ));
    }
    for threshold in [
        protocol.selector.high_similarity,
        protocol.selector.high_margin,
        protocol.selector.margin_min_similarity,
        protocol.selector.agreement_min_similarity,
        protocol.quality_gates.candidate_min_recall_at_k,
        protocol.quality_gates.auto_apply_min_precision,
        protocol.quality_gates.auto_apply_min_quality_coverage,
    ] {
        if !(0.0..=1.0).contains(&threshold) {
            return Err(EvalError::Invalid(
                "selective thresholds must be in [0, 1]".to_string(),
            ));
        }
    }
    let quality_count = protocol
        .held_out_queries
        .iter()
        .filter(|query| !query.abstain)
        .count();
    let abstention_count = protocol.held_out_queries.len() - quality_count;
    if quality_count < 10 || abstention_count < 6 {
        return Err(EvalError::Invalid(
            "v3 requires at least 10 quality and 6 abstention queries".to_string(),
        ));
    }
    let mut ids = BTreeSet::new();
    for query in &protocol.held_out_queries {
        if query.id.trim().is_empty()
            || query.query.trim().is_empty()
            || !ids.insert(query.id.as_str())
            || query.abstain != query.relevant_keys.is_empty()
        {
            return Err(EvalError::Invalid(format!(
                "invalid held-out query: {}",
                query.id
            )));
        }
    }
    Ok(())
}

fn validate_disjoint_queries(
    held_out: &[RetrievalQuery],
    base: &RetrievalProtocol,
    calibration: &RetrievalV2Protocol,
) -> EvalResult<()> {
    let prior_ids = base
        .queries
        .iter()
        .chain(&calibration.additional_queries)
        .map(|query| query.id.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    let prior_texts = base
        .queries
        .iter()
        .chain(&calibration.additional_queries)
        .map(|query| query.query.trim().to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    for query in held_out {
        if prior_ids.contains(&query.id.to_ascii_lowercase())
            || prior_texts.contains(&query.query.trim().to_ascii_lowercase())
        {
            return Err(EvalError::Invalid(format!(
                "held-out query duplicates calibration data: {}",
                query.id
            )));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn select_for_auto_apply(
    query: &RetrievalQuery,
    current: &[RetrievalHit],
    candidates: &[RetrievalHit],
    scoped_active: &[&MemoryItem],
    all_items: &[MemoryItem],
    key_by_id: &BTreeMap<String, String>,
    item_by_key: &BTreeMap<String, MemoryItem>,
    selector: &SelectiveRanking,
) -> Vec<RetrievalHit> {
    let opaque = opaque_cues(&query.query);
    if opaque.iter().any(|cue| {
        !scoped_active
            .iter()
            .any(|item| memory_document_text(item).contains(cue))
    }) {
        return Vec::new();
    }

    let successors = exact_lifecycle_successors(query, all_items, key_by_id);
    if let Some(key) = successors.first() {
        return vec![RetrievalHit {
            rank: 1,
            key: key.clone(),
            score: 1.0,
        }];
    }

    let Some(top) = candidates.first() else {
        return Vec::new();
    };
    let margin = candidates
        .get(1)
        .map_or(top.score, |second| top.score - second.score);
    let high_similarity = top.score >= selector.high_similarity;
    let high_margin = top.score >= selector.margin_min_similarity && margin >= selector.high_margin;
    let agreement = current
        .first()
        .is_some_and(|lexical| lexical.key == top.key)
        && top.score >= selector.agreement_min_similarity
        && item_by_key.get(&top.key).is_some_and(|item| {
            terms(&query.query)
                .intersection(&terms(&memory_document_text(item)))
                .count()
                >= selector.agreement_min_overlap_terms
        });
    if high_similarity || high_margin || agreement {
        vec![RetrievalHit {
            rank: 1,
            key: top.key.clone(),
            score: top.score,
        }]
    } else {
        Vec::new()
    }
}

fn exact_lifecycle_successors(
    query: &RetrievalQuery,
    items: &[MemoryItem],
    key_by_id: &BTreeMap<String, String>,
) -> Vec<String> {
    let query_terms = terms(&query.query);
    if query_terms.is_empty() {
        return Vec::new();
    }
    let matched = items
        .iter()
        .filter(|item| item.status != MemoryStatus::Active && resolved_scope_matches(item, query))
        .filter(|item| query_terms.is_subset(&terms(&memory_document_text(item))))
        .map(|item| item.id.to_string())
        .collect::<BTreeSet<_>>();
    let mut successors = items
        .iter()
        .filter(|item| item.status == MemoryStatus::Active && resolved_scope_matches(item, query))
        .filter(|item| {
            item.supersedes
                .iter()
                .any(|id| matched.contains(&id.to_string()))
        })
        .filter_map(|item| key_by_id.get(&item.id.to_string()).cloned())
        .collect::<Vec<_>>();
    successors.sort();
    successors
}

fn terms(value: &str) -> BTreeSet<String> {
    value
        .to_ascii_lowercase()
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|term| term.len() >= 2)
        .filter(|term| {
            !matches!(
                *term,
                "and"
                    | "are"
                    | "for"
                    | "from"
                    | "how"
                    | "the"
                    | "what"
                    | "when"
                    | "where"
                    | "which"
                    | "with"
            )
        })
        .map(ToString::to_string)
        .collect()
}

fn selective_metrics(queries: &[RetrievalQuery], runs: &[RetrievalQueryRun]) -> SelectiveMetrics {
    let selector_runs = runs
        .iter()
        .filter(|run| run.arm == AUTO_APPLY_ARM)
        .collect::<Vec<_>>();
    let quality_ids = queries
        .iter()
        .filter(|query| !query.abstain)
        .map(|query| query.id.as_str())
        .collect::<BTreeSet<_>>();
    let quality_application_count = selector_runs
        .iter()
        .filter(|run| quality_ids.contains(run.query_id.as_str()) && !run.hits.is_empty())
        .count();
    let total_application_count = selector_runs
        .iter()
        .filter(|run| !run.hits.is_empty())
        .count();
    let correct_application_count = selector_runs
        .iter()
        .filter(|run| run.first_relevant_rank == Some(1))
        .count();
    SelectiveMetrics {
        quality_query_count: quality_ids.len(),
        quality_application_count,
        quality_coverage: fraction(quality_application_count, quality_ids.len()),
        total_application_count,
        correct_application_count,
        application_precision: fraction(correct_application_count, total_application_count),
        unsafe_application_count: total_application_count - correct_application_count,
    }
}

fn gate_violations(
    gates: &SelectiveQualityGates,
    arms: &[RetrievalArmMetrics],
    selective: &SelectiveMetrics,
    timing: &SemanticTiming,
) -> Vec<String> {
    let mut violations = Vec::new();
    match arms.iter().find(|arm| arm.arm == DENSE_CANDIDATE_ARM) {
        Some(candidate) if candidate.recall_at_k < gates.candidate_min_recall_at_k => {
            violations.push(format!(
                "candidate_recall_at_k:{:.4}<{}",
                candidate.recall_at_k, gates.candidate_min_recall_at_k
            ));
        }
        None => violations.push("missing_dense_candidate_arm".to_string()),
        _ => {}
    }
    if selective.application_precision < gates.auto_apply_min_precision {
        violations.push(format!(
            "auto_apply_precision:{:.4}<{}",
            selective.application_precision, gates.auto_apply_min_precision
        ));
    }
    if selective.quality_coverage < gates.auto_apply_min_quality_coverage {
        violations.push(format!(
            "auto_apply_quality_coverage:{:.4}<{}",
            selective.quality_coverage, gates.auto_apply_min_quality_coverage
        ));
    }
    if selective.unsafe_application_count > gates.auto_apply_max_unsafe_applications {
        violations.push(format!(
            "unsafe_applications:{}>{}",
            selective.unsafe_application_count, gates.auto_apply_max_unsafe_applications
        ));
    }
    for arm in arms {
        if arm.scope_leakage_count > gates.max_scope_leakage {
            violations.push(format!(
                "{}:scope_leakage:{}>{}",
                arm.arm, arm.scope_leakage_count, gates.max_scope_leakage
            ));
        }
        if arm.stale_result_count > gates.max_stale_results {
            violations.push(format!(
                "{}:stale_results:{}>{}",
                arm.arm, arm.stale_result_count, gates.max_stale_results
            ));
        }
        if arm.forbidden_result_count > gates.max_forbidden_results {
            violations.push(format!(
                "{}:forbidden_results:{}>{}",
                arm.arm, arm.forbidden_result_count, gates.max_forbidden_results
            ));
        }
    }
    if timing.query_embedding_p95_ms > gates.semantic_query_p95_ms as f64 {
        violations.push(format!(
            "semantic_query_p95_ms:{:.4}>{}",
            timing.query_embedding_p95_ms, gates.semantic_query_p95_ms
        ));
    }
    violations
}

fn percentile(sorted: &[f64], quantile: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let index = ((sorted.len() - 1) as f64 * quantile).ceil() as usize;
    sorted[index]
}

fn fraction(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        1.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn elapsed_ms(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1_000.0
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn invalid(error: impl std::fmt::Display) -> EvalError {
    EvalError::Invalid(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selector_terms_drop_only_frozen_stop_words() {
        assert_eq!(
            terms("which HTTP client for the worker"),
            BTreeSet::from([
                "client".to_string(),
                "http".to_string(),
                "worker".to_string()
            ])
        );
    }

    #[test]
    fn selective_precision_counts_deferred_queries_as_neither_correct_nor_unsafe() {
        let queries = vec![quality_query("quality"), abstention_query("none")];
        let runs = vec![scored_run("quality", vec![]), scored_run("none", vec![])];
        let metrics = selective_metrics(&queries, &runs);
        assert_eq!(metrics.total_application_count, 0);
        assert_eq!(metrics.unsafe_application_count, 0);
        assert_eq!(metrics.application_precision, 1.0);
        assert_eq!(metrics.quality_coverage, 0.0);
    }

    fn quality_query(id: &str) -> RetrievalQuery {
        RetrievalQuery {
            id: id.to_string(),
            category: "test".to_string(),
            query: "query".to_string(),
            project: "atlas".to_string(),
            cwd: "fixture://atlas/main".to_string(),
            repository_remote: "git@github.com:acme/atlas.git".to_string(),
            relevant_keys: vec!["relevant".to_string()],
            forbidden_keys: Vec::new(),
            abstain: false,
        }
    }

    fn abstention_query(id: &str) -> RetrievalQuery {
        RetrievalQuery {
            relevant_keys: Vec::new(),
            abstain: true,
            ..quality_query(id)
        }
    }

    fn scored_run(id: &str, hits: Vec<RetrievalHit>) -> RetrievalQueryRun {
        RetrievalQueryRun {
            arm: AUTO_APPLY_ARM.to_string(),
            query_id: id.to_string(),
            category: "test".to_string(),
            hits,
            first_relevant_rank: None,
            reciprocal_rank: 0.0,
            recalled_at_k: false,
            scope_leakage_keys: Vec::new(),
            stale_keys: Vec::new(),
            forbidden_keys: Vec::new(),
            abstention_false_positive: false,
        }
    }
}
