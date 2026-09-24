//! Bounded structured-candidate ablation over the known retrieval-v3 failure corpus.

use crate::fixture::materialize_engineering_context_fixture;
use crate::native_runner::write_private_json;
use crate::pilot::require_empty_target;
use crate::retrieval_probe::{
    add_corpus_expansions, aggregate_arm, resolve_fixture_uri, resolved_scope_matches,
    score_query_run, validate_judgments, validate_protocol, RetrievalArmMetrics, RetrievalHit,
    RetrievalProtocol, RetrievalQuery, RetrievalQueryRun,
};
use crate::retrieval_probe_v2::{
    attest_model_snapshot, dense_hits, memory_document_text, validate_v2_protocol,
    BaseProtocolReference, LocalModelAttestation, RetrievalV2Protocol, SemanticTiming,
};
use crate::retrieval_probe_v3::{validate_selective_protocol, SelectiveProtocol};
use crate::seed::open_seeded_engineering_context_fixture;
use crate::{EvalError, EvalResult};
use engram_core::memory::{MemoryItem, MemoryKind, MemoryStatus};
use engram_embed::{EmbedConfig, Embedder};
use engram_index::{MemoryService, SearchOptions, SearchService};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::time::Instant;
use time::OffsetDateTime;

const STRUCTURED_PROBE_SCHEMA_VERSION: u32 = 4;
const CURRENT_ARM: &str = "current_lexical";
const DENSE_ARM: &str = "dense_candidate_top5";
const STRUCTURED_KIND_ARM: &str = "structured_kind_candidates";
const EXACT_TAG_ARM: &str = "exact_tag_candidates";
const LIFECYCLE_ARM: &str = "lifecycle_successor_candidates";
const BUNDLE_ARM: &str = "candidate_bundle_max10";

/// Frozen high-signal structured candidate rules.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StructuredCandidateRules {
    /// Query terms that permit verified procedure candidates.
    pub procedure_cues: Vec<String>,
    /// Query terms that permit current handoff candidates.
    pub handoff_cues: Vec<String>,
    /// Query terms that permit current rule candidates.
    pub rule_cues: Vec<String>,
    /// Maximum candidates from the combined kind-aware lane.
    pub structured_kind_limit: usize,
    /// Maximum exact-tag candidates.
    pub exact_tag_limit: usize,
    /// Maximum explicit lifecycle successors.
    pub lifecycle_limit: usize,
    /// Maximum candidates in the deduplicated presentation bundle.
    pub bundle_max_candidates: usize,
    /// Deterministic presentation priority; scores are never fused.
    pub source_priority: Vec<String>,
    /// Exact verified-procedure policy name.
    pub procedure_policy: String,
    /// Exact tag policy name.
    pub tag_policy: String,
    /// Exact lifecycle policy name.
    pub lifecycle_policy: String,
}

/// Frozen gates for the bounded candidate-set diagnostic.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StructuredQualityGates {
    /// Minimum quality-query recall in the bounded candidate set.
    pub bundle_min_recall: f64,
    /// Maximum observed candidates in any bundle.
    pub bundle_max_candidates: usize,
    /// Minimum v3 baseline misses recovered by structured-kind metadata.
    pub structured_min_baseline_miss_recoveries: usize,
    /// Maximum out-of-scope results for every lane and the bundle.
    pub max_scope_leakage: usize,
    /// Maximum inactive results for every lane and the bundle.
    pub max_stale_results: usize,
    /// Maximum explicitly forbidden results for every lane and the bundle.
    pub max_forbidden_results: usize,
    /// Maximum p95 query-embedding latency.
    pub semantic_query_p95_ms: u64,
}

/// Frozen v4 diagnostic protocol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StructuredCandidateProtocol {
    /// Protocol schema version.
    pub schema_version: u32,
    /// Stable probe identifier.
    pub probe_id: String,
    /// Exact v3 protocol supplying the known failure corpus.
    pub source_protocol: BaseProtocolReference,
    /// Checked-in v3 results document used to establish observed misses.
    pub source_results: BaseProtocolReference,
    /// SHA-256 of the raw v3 report named in the checked-in results.
    pub source_report_sha256: String,
    /// Explicit non-generalization mode.
    pub evaluation_mode: String,
    /// Ordered report arms.
    pub arms: Vec<String>,
    /// Frozen structured candidate rules and budgets.
    pub structured: StructuredCandidateRules,
    /// Frozen quality and safety gates.
    pub quality_gates: StructuredQualityGates,
}

/// One selected bundle candidate with every lane that surfaced it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CandidateBundleEntry {
    /// Presentation rank only; it is not an applicability score.
    pub rank: usize,
    /// Stable semantic key.
    pub key: String,
    /// Candidate lanes that surfaced this item.
    pub sources: Vec<String>,
}

/// Query-level source provenance for the bounded candidate bundle.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CandidateBundleEvidence {
    /// Frozen source-query identifier.
    pub query_id: String,
    /// Deduplicated candidates in deterministic presentation order.
    pub entries: Vec<CandidateBundleEntry>,
}

/// Per-lane contribution to quality-query recall.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LaneContribution {
    /// Candidate lane.
    pub lane: String,
    /// Quality queries for which this lane contained a relevant result.
    pub quality_relevant_count: usize,
}

/// Candidate-set recall and burden evidence.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CandidateBundleMetrics {
    /// Number of non-abstention quality queries.
    pub quality_query_count: usize,
    /// Quality queries recalled by the final bounded set.
    pub quality_recalled_count: usize,
    /// Set recall over quality queries.
    pub quality_recall: f64,
    /// Mean candidates across all quality and abstention queries.
    pub mean_candidates: f64,
    /// Largest observed bundle.
    pub max_candidates: usize,
    /// Abstention queries that still surfaced explicit search candidates.
    pub abstention_nonempty_count: usize,
    /// Quality-query recall supplied by each separate lane.
    pub lane_contributions: Vec<LaneContribution>,
    /// Structured-kind recoveries absent from the lexical/dense v3 baseline union.
    pub structured_baseline_miss_recovery_count: usize,
}

/// Reproducible provider-free v4 structured-candidate report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StructuredCandidateReport {
    /// Probe identifier.
    pub probe_id: String,
    /// SHA-256 of exact v4 protocol bytes.
    pub protocol_sha256: String,
    /// SHA-256 of exact v3 source protocol bytes.
    pub source_protocol_sha256: String,
    /// SHA-256 of exact checked-in v3 results bytes.
    pub source_results_sha256: String,
    /// SHA-256 of the raw v3 report declared by the protocol and results document.
    pub source_report_sha256: String,
    /// Explicit diagnostic mode.
    pub evaluation_mode: String,
    /// Frozen fixture content revision.
    pub fixture_revision: String,
    /// Attested model identity inherited from retrieval v2.
    pub model: LocalModelAttestation,
    /// Observed local model file hashes.
    pub observed_model_files: BTreeMap<String, String>,
    /// Number of all records in the isolated store.
    pub corpus_record_count: usize,
    /// Number of active candidate records.
    pub active_record_count: usize,
    /// Number of reused v3 evaluation queries.
    pub query_count: usize,
    /// Semantic latency evidence.
    pub timing: SemanticTiming,
    /// Aggregate metrics for each separate lane and the bounded bundle.
    pub arms: Vec<RetrievalArmMetrics>,
    /// Candidate-set recall and burden.
    pub bundle: CandidateBundleMetrics,
    /// Query-level lane scores and safety evidence.
    pub runs: Vec<RetrievalQueryRun>,
    /// Query-level candidate provenance without fused scores.
    pub bundle_evidence: Vec<CandidateBundleEvidence>,
    /// Frozen gate violations.
    pub gate_violations: Vec<String>,
    /// True only when every frozen gate passes.
    pub passed: bool,
    /// Claims this diagnostic cannot support.
    pub limitations: Vec<String>,
}

/// Run the frozen provider-free structured-candidate diagnostic.
pub async fn run_structured_candidate_probe(
    protocol_path: &Path,
    model_snapshot: &Path,
    output: &Path,
) -> EvalResult<StructuredCandidateReport> {
    run_structured_candidate_probe_internal(protocol_path, model_snapshot, output, None, true).await
}

/// Replay the byte-frozen v4 candidate algorithm against a separately frozen query set.
pub(crate) async fn replay_structured_candidate_queries(
    protocol_path: &Path,
    model_snapshot: &Path,
    output: &Path,
    queries: &[RetrievalQuery],
) -> EvalResult<StructuredCandidateReport> {
    run_structured_candidate_probe_internal(
        protocol_path,
        model_snapshot,
        output,
        Some(queries),
        false,
    )
    .await
}

async fn run_structured_candidate_probe_internal(
    protocol_path: &Path,
    model_snapshot: &Path,
    output: &Path,
    query_override: Option<&[RetrievalQuery]>,
    write_report: bool,
) -> EvalResult<StructuredCandidateReport> {
    let protocol_bytes = fs::read(protocol_path)?;
    let protocol: StructuredCandidateProtocol = serde_json::from_slice(&protocol_bytes)?;
    validate_structured_protocol(&protocol)?;
    let protocol_sha256 = sha256_bytes(&protocol_bytes);

    let owner = protocol_path.parent().unwrap_or_else(|| Path::new("."));
    let source_path = owner.join(&protocol.source_protocol.path);
    let source_bytes = fs::read(&source_path)?;
    let source_sha256 = sha256_bytes(&source_bytes);
    require_hash(
        "source v3 protocol",
        &source_sha256,
        &protocol.source_protocol.sha256,
    )?;
    let source: SelectiveProtocol = serde_json::from_slice(&source_bytes)?;
    validate_selective_protocol(&source)?;
    let evaluation_queries = query_override.unwrap_or(&source.held_out_queries);

    let source_results_path = owner.join(&protocol.source_results.path);
    let source_results_bytes = fs::read(&source_results_path)?;
    let source_results_sha256 = sha256_bytes(&source_results_bytes);
    require_hash(
        "source v3 results",
        &source_results_sha256,
        &protocol.source_results.sha256,
    )?;
    let source_results_text = String::from_utf8_lossy(&source_results_bytes);
    if !source_results_text.contains(&protocol.source_report_sha256)
        || !source_results_text.contains(&protocol.source_protocol.sha256)
    {
        return Err(EvalError::Invalid(
            "v3 results do not attest the declared source protocol and raw report".to_string(),
        ));
    }

    let calibration_path = source_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(&source.calibration_protocol.path);
    let calibration_bytes = fs::read(&calibration_path)?;
    let calibration_sha256 = sha256_bytes(&calibration_bytes);
    require_hash(
        "v2 calibration protocol",
        &calibration_sha256,
        &source.calibration_protocol.sha256,
    )?;
    let calibration: RetrievalV2Protocol = serde_json::from_slice(&calibration_bytes)?;
    validate_v2_protocol(&calibration)?;

    let base_path = calibration_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(&calibration.base_protocol.path);
    let base_bytes = fs::read(&base_path)?;
    let base_sha256 = sha256_bytes(&base_bytes);
    require_hash(
        "v1 base protocol",
        &base_sha256,
        &calibration.base_protocol.sha256,
    )?;
    let base: RetrievalProtocol = serde_json::from_slice(&base_bytes)?;
    validate_protocol(&base)?;

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
        queries: evaluation_queries.to_vec(),
        ..base.clone()
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
    let mut bundle_evidence = Vec::new();
    for query in evaluation_queries {
        let cwd = resolve_fixture_uri(&layout, &query.cwd)?;
        let current = search
            .search_local_memory(
                &query.query,
                source.top_k,
                Some(source.current_min_score),
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

        let mut query_vector = None;
        for _ in 0..source.repetitions {
            let started = Instant::now();
            let vector = embedder.embed(&query.query).map_err(invalid)?;
            query_latencies.push(elapsed_ms(started));
            query_vector.get_or_insert(vector);
        }
        let query_vector = query_vector.ok_or_else(|| {
            EvalError::Invalid("v4 repetitions unexpectedly produced no embedding".to_string())
        })?;
        let scoped_active = active_items
            .iter()
            .filter(|item| resolved_scope_matches(item, query))
            .collect::<Vec<_>>();
        let dense = dense_hits(
            &query_vector,
            &scoped_active,
            &key_by_id,
            &vectors_by_key,
            source.top_k,
        );
        let structured = structured_kind_hits(
            query,
            &scoped_active,
            &key_by_id,
            &protocol.structured,
            OffsetDateTime::now_utc(),
        );
        let exact_tags = exact_tag_hits(
            query,
            &scoped_active,
            &key_by_id,
            protocol.structured.exact_tag_limit,
        );
        let lifecycle = lifecycle_successor_hits(
            query,
            &all_items,
            &key_by_id,
            protocol.structured.lifecycle_limit,
        );

        let lane_hits = BTreeMap::from([
            (CURRENT_ARM.to_string(), current_hits.clone()),
            (DENSE_ARM.to_string(), dense.clone()),
            (STRUCTURED_KIND_ARM.to_string(), structured.clone()),
            (EXACT_TAG_ARM.to_string(), exact_tags.clone()),
            (LIFECYCLE_ARM.to_string(), lifecycle.clone()),
        ]);
        let evidence = compose_bundle(
            query,
            &lane_hits,
            &protocol.structured.source_priority,
            protocol.structured.bundle_max_candidates,
        );
        let bundle_hits = evidence
            .entries
            .iter()
            .map(|entry| RetrievalHit {
                rank: entry.rank,
                key: entry.key.clone(),
                score: 0.0,
            })
            .collect::<Vec<_>>();

        for (arm, hits) in [
            (CURRENT_ARM, current_hits),
            (DENSE_ARM, dense),
            (STRUCTURED_KIND_ARM, structured),
            (EXACT_TAG_ARM, exact_tags),
            (LIFECYCLE_ARM, lifecycle),
            (BUNDLE_ARM, bundle_hits),
        ] {
            runs.push(score_query_run(arm, query, hits, &item_by_key));
        }
        bundle_evidence.push(evidence);
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
        .map(|arm| aggregate_arm(arm, evaluation_queries, &runs))
        .collect::<Vec<_>>();
    let bundle = bundle_metrics(
        evaluation_queries,
        &runs,
        &bundle_evidence,
        &protocol.structured.source_priority,
    );
    let gate_violations = gate_violations(&protocol.quality_gates, &arms, &bundle, &timing);
    let report = StructuredCandidateReport {
        probe_id: protocol.probe_id,
        protocol_sha256,
        source_protocol_sha256: source_sha256,
        source_results_sha256,
        source_report_sha256: protocol.source_report_sha256,
        evaluation_mode: protocol.evaluation_mode,
        fixture_revision: seed.fixture_revision,
        model: calibration.model,
        observed_model_files,
        corpus_record_count: all_items.len(),
        active_record_count: active_items.len(),
        query_count: evaluation_queries.len(),
        timing,
        arms,
        bundle,
        runs,
        bundle_evidence,
        passed: gate_violations.is_empty(),
        gate_violations,
        limitations: vec![
            "The v3 query texts and misses were known before this ablation; this is diagnostic evidence, not held-out generalization.".to_string(),
            "Candidates are explicit-search possibilities, not applicability judgments, automatic context, or permission to execute a procedure.".to_string(),
            "Bundle rank is deterministic presentation priority only; lexical, dense, and structured scores are not fused or made comparable.".to_string(),
            "The intent cues are hand-built against a small synthetic corpus and require an independent adversarial evaluation before product use.".to_string(),
            "Exact normalized tags are tested, but entity aliases and graph traversal are deferred because the fixture lacks an authoritative entity-to-memory ownership edge.".to_string(),
            "Abstention candidate burden is reported but not treated as unsafe application because this probe never auto-applies candidates.".to_string(),
            "Host-native task use remains a separately approval-gated evaluation and no provider was called.".to_string(),
        ],
    };
    if write_report {
        write_private_json(&output.join("retrieval-report.json"), &report)?;
    }
    Ok(report)
}

pub(crate) fn validate_structured_protocol(
    protocol: &StructuredCandidateProtocol,
) -> EvalResult<()> {
    let expected_arms = [
        CURRENT_ARM,
        DENSE_ARM,
        STRUCTURED_KIND_ARM,
        EXACT_TAG_ARM,
        LIFECYCLE_ARM,
        BUNDLE_ARM,
    ];
    let expected_priority = [
        LIFECYCLE_ARM,
        STRUCTURED_KIND_ARM,
        EXACT_TAG_ARM,
        CURRENT_ARM,
        DENSE_ARM,
    ];
    if protocol.schema_version != STRUCTURED_PROBE_SCHEMA_VERSION
        || protocol.probe_id.trim().is_empty()
        || protocol.evaluation_mode != "known_v3_failure_corpus_diagnostic_ablation"
        || protocol.arms.iter().map(String::as_str).collect::<Vec<_>>() != expected_arms
        || protocol
            .structured
            .source_priority
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            != expected_priority
    {
        return Err(EvalError::Invalid(
            "invalid structured candidate protocol header, arms, or mode".to_string(),
        ));
    }
    if protocol.structured.procedure_cues.is_empty()
        || protocol.structured.handoff_cues.is_empty()
        || protocol.structured.rule_cues.is_empty()
        || protocol.structured.structured_kind_limit == 0
        || protocol.structured.exact_tag_limit == 0
        || protocol.structured.lifecycle_limit == 0
        || protocol.structured.bundle_max_candidates == 0
        || protocol.structured.bundle_max_candidates != protocol.quality_gates.bundle_max_candidates
    {
        return Err(EvalError::Invalid(
            "structured candidate cues and budgets must be non-empty and consistent".to_string(),
        ));
    }
    if protocol.structured.procedure_policy != "active_scoped_receipt_verified_unexpired"
        || protocol.structured.tag_policy != "exact_normalized_tag_phrase"
        || protocol.structured.lifecycle_policy
            != "all_query_terms_match_inactive_then_follow_explicit_successor"
    {
        return Err(EvalError::Invalid(
            "unsupported structured candidate policy".to_string(),
        ));
    }
    if !(0.0..=1.0).contains(&protocol.quality_gates.bundle_min_recall)
        || protocol.source_protocol.sha256.len() != 64
        || protocol.source_results.sha256.len() != 64
        || protocol.source_report_sha256.len() != 64
    {
        return Err(EvalError::Invalid(
            "invalid structured candidate hashes or gates".to_string(),
        ));
    }
    Ok(())
}

fn structured_kind_hits(
    query: &RetrievalQuery,
    items: &[&MemoryItem],
    key_by_id: &BTreeMap<String, String>,
    rules: &StructuredCandidateRules,
    now: OffsetDateTime,
) -> Vec<RetrievalHit> {
    let query_terms = all_terms(&query.query);
    let wants_procedure = contains_cue(&query_terms, &rules.procedure_cues);
    let wants_handoff = contains_cue(&query_terms, &rules.handoff_cues);
    let wants_rule = contains_cue(&query_terms, &rules.rule_cues);
    let mut scored = items
        .iter()
        .filter(|item| match &item.kind {
            MemoryKind::Procedure => {
                wants_procedure
                    && item
                        .procedure
                        .as_ref()
                        .is_some_and(|procedure| procedure.is_verified_at(now))
            }
            MemoryKind::Handoff => wants_handoff,
            MemoryKind::Rule => wants_rule,
            _ => false,
        })
        .filter_map(|item| {
            let key = key_by_id.get(&item.id.to_string())?.clone();
            let overlap = query_terms
                .intersection(&all_terms(&memory_document_text(item)))
                .count();
            Some((key, overlap as f64))
        })
        .collect::<Vec<_>>();
    rank_pairs(&mut scored, rules.structured_kind_limit)
}

fn exact_tag_hits(
    query: &RetrievalQuery,
    items: &[&MemoryItem],
    key_by_id: &BTreeMap<String, String>,
    limit: usize,
) -> Vec<RetrievalHit> {
    let normalized_query = format!(" {} ", normalize_phrase(&query.query));
    let mut scored = items
        .iter()
        .filter_map(|item| {
            let score = item
                .tags
                .iter()
                .map(|tag| normalize_phrase(tag))
                .filter(|tag| !tag.is_empty())
                .filter(|tag| normalized_query.contains(&format!(" {tag} ")))
                .map(|tag| tag.split_whitespace().count())
                .max()?;
            key_by_id
                .get(&item.id.to_string())
                .cloned()
                .map(|key| (key, score as f64))
        })
        .collect::<Vec<_>>();
    rank_pairs(&mut scored, limit)
}

fn lifecycle_successor_hits(
    query: &RetrievalQuery,
    items: &[MemoryItem],
    key_by_id: &BTreeMap<String, String>,
    limit: usize,
) -> Vec<RetrievalHit> {
    let query_terms = lifecycle_terms(&query.query);
    if query_terms.is_empty() {
        return Vec::new();
    }
    let matched = items
        .iter()
        .filter(|item| item.status != MemoryStatus::Active && resolved_scope_matches(item, query))
        .filter(|item| query_terms.is_subset(&lifecycle_terms(&memory_document_text(item))))
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
        .map(|key| (key, 1.0))
        .collect::<Vec<_>>();
    rank_pairs(&mut successors, limit)
}

fn compose_bundle(
    query: &RetrievalQuery,
    lanes: &BTreeMap<String, Vec<RetrievalHit>>,
    priority: &[String],
    limit: usize,
) -> CandidateBundleEvidence {
    let mut sources_by_key = BTreeMap::<String, BTreeSet<String>>::new();
    for (lane, hits) in lanes {
        for hit in hits {
            sources_by_key
                .entry(hit.key.clone())
                .or_default()
                .insert(lane.clone());
        }
    }
    let mut selected = Vec::new();
    let mut seen = BTreeSet::new();
    for lane in priority {
        for hit in lanes.get(lane).into_iter().flatten() {
            if seen.insert(hit.key.clone()) {
                selected.push(hit.key.clone());
                if selected.len() == limit {
                    break;
                }
            }
        }
        if selected.len() == limit {
            break;
        }
    }
    let entries = selected
        .into_iter()
        .enumerate()
        .map(|(index, key)| CandidateBundleEntry {
            rank: index + 1,
            sources: priority
                .iter()
                .filter(|lane| {
                    sources_by_key
                        .get(&key)
                        .is_some_and(|sources| sources.contains(*lane))
                })
                .cloned()
                .collect(),
            key,
        })
        .collect();
    CandidateBundleEvidence {
        query_id: query.id.clone(),
        entries,
    }
}

fn bundle_metrics(
    queries: &[RetrievalQuery],
    runs: &[RetrievalQueryRun],
    evidence: &[CandidateBundleEvidence],
    lanes: &[String],
) -> CandidateBundleMetrics {
    let quality = queries
        .iter()
        .filter(|query| !query.abstain)
        .collect::<Vec<_>>();
    let quality_recalled_count = quality
        .iter()
        .filter(|query| run_for(runs, BUNDLE_ARM, &query.id).is_some_and(|run| run.recalled_at_k))
        .count();
    let lane_contributions = lanes
        .iter()
        .map(|lane| LaneContribution {
            lane: lane.clone(),
            quality_relevant_count: quality
                .iter()
                .filter(|query| run_for(runs, lane, &query.id).is_some_and(|run| run.recalled_at_k))
                .count(),
        })
        .collect::<Vec<_>>();
    let structured_baseline_miss_recovery_count = quality
        .iter()
        .filter(|query| {
            let structured =
                run_for(runs, STRUCTURED_KIND_ARM, &query.id).is_some_and(|run| run.recalled_at_k);
            let baseline = run_for(runs, CURRENT_ARM, &query.id)
                .is_some_and(|run| run.recalled_at_k)
                || run_for(runs, DENSE_ARM, &query.id).is_some_and(|run| run.recalled_at_k);
            structured && !baseline
        })
        .count();
    let candidate_counts = evidence
        .iter()
        .map(|query| query.entries.len())
        .collect::<Vec<_>>();
    CandidateBundleMetrics {
        quality_query_count: quality.len(),
        quality_recalled_count,
        quality_recall: fraction(quality_recalled_count, quality.len()),
        mean_candidates: if candidate_counts.is_empty() {
            0.0
        } else {
            candidate_counts.iter().sum::<usize>() as f64 / candidate_counts.len() as f64
        },
        max_candidates: candidate_counts.into_iter().max().unwrap_or_default(),
        abstention_nonempty_count: queries
            .iter()
            .filter(|query| query.abstain)
            .filter(|query| {
                evidence
                    .iter()
                    .find(|item| item.query_id == query.id)
                    .is_some_and(|item| !item.entries.is_empty())
            })
            .count(),
        lane_contributions,
        structured_baseline_miss_recovery_count,
    }
}

fn gate_violations(
    gates: &StructuredQualityGates,
    arms: &[RetrievalArmMetrics],
    bundle: &CandidateBundleMetrics,
    timing: &SemanticTiming,
) -> Vec<String> {
    let mut violations = Vec::new();
    if bundle.quality_recall < gates.bundle_min_recall {
        violations.push(format!(
            "bundle_recall:{:.4}<{}",
            bundle.quality_recall, gates.bundle_min_recall
        ));
    }
    if bundle.max_candidates > gates.bundle_max_candidates {
        violations.push(format!(
            "bundle_max_candidates:{}>{}",
            bundle.max_candidates, gates.bundle_max_candidates
        ));
    }
    if bundle.structured_baseline_miss_recovery_count
        < gates.structured_min_baseline_miss_recoveries
    {
        violations.push(format!(
            "structured_baseline_miss_recoveries:{}<{}",
            bundle.structured_baseline_miss_recovery_count,
            gates.structured_min_baseline_miss_recoveries
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

fn run_for<'a>(
    runs: &'a [RetrievalQueryRun],
    arm: &str,
    query_id: &str,
) -> Option<&'a RetrievalQueryRun> {
    runs.iter()
        .find(|run| run.arm == arm && run.query_id == query_id)
}

fn rank_pairs(scored: &mut [(String, f64)], limit: usize) -> Vec<RetrievalHit> {
    scored.sort_by(|left, right| {
        right
            .1
            .total_cmp(&left.1)
            .then_with(|| left.0.cmp(&right.0))
    });
    scored
        .iter()
        .take(limit)
        .enumerate()
        .map(|(index, (key, score))| RetrievalHit {
            rank: index + 1,
            key: key.clone(),
            score: *score,
        })
        .collect()
}

fn contains_cue(query_terms: &BTreeSet<String>, cues: &[String]) -> bool {
    cues.iter()
        .any(|cue| query_terms.contains(&cue.to_ascii_lowercase()))
}

fn all_terms(value: &str) -> BTreeSet<String> {
    value
        .to_ascii_lowercase()
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|term| term.len() >= 2)
        .map(ToString::to_string)
        .collect()
}

fn lifecycle_terms(value: &str) -> BTreeSet<String> {
    all_terms(value)
        .into_iter()
        .filter(|term| {
            !matches!(
                term.as_str(),
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
        .collect()
}

fn normalize_phrase(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|term| !term.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn require_hash(label: &str, observed: &str, expected: &str) -> EvalResult<()> {
    if observed != expected {
        return Err(EvalError::Invalid(format!(
            "{label} hash mismatch: observed {observed}, expected {expected}"
        )));
    }
    Ok(())
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
    fn exact_tags_match_normalized_phrases_not_partial_words() {
        assert!(format!(" {} ", normalize_phrase("debug with bits now"))
            .contains(&format!(" {} ", normalize_phrase("debug-with-bits"))));
        assert!(!format!(" {} ", normalize_phrase("debug bitstream now"))
            .contains(&format!(" {} ", normalize_phrase("bits"))));
    }

    #[test]
    fn cue_matching_is_case_insensitive_and_term_exact() {
        let query = all_terms("What VERIFIED command should I use?");
        assert!(contains_cue(&query, &["verified".to_string()]));
        assert!(!contains_cue(&query, &["verify".to_string()]));
    }

    #[test]
    fn bundle_deduplicates_and_preserves_frozen_source_priority() {
        let query = RetrievalQuery {
            id: "query".to_string(),
            category: "quality".to_string(),
            query: "query".to_string(),
            project: "atlas".to_string(),
            cwd: "fixture://atlas/main".to_string(),
            repository_remote: "git@github.com:acme/atlas.git".to_string(),
            relevant_keys: vec!["structured".to_string()],
            forbidden_keys: Vec::new(),
            abstain: false,
        };
        let lanes = BTreeMap::from([
            (
                CURRENT_ARM.to_string(),
                vec![hit("shared", 1), hit("lexical", 2)],
            ),
            (
                STRUCTURED_KIND_ARM.to_string(),
                vec![hit("structured", 1), hit("shared", 2)],
            ),
        ]);
        let priority = vec![STRUCTURED_KIND_ARM.to_string(), CURRENT_ARM.to_string()];
        let evidence = compose_bundle(&query, &lanes, &priority, 2);
        assert_eq!(
            evidence
                .entries
                .iter()
                .map(|entry| entry.key.as_str())
                .collect::<Vec<_>>(),
            vec!["structured", "shared"]
        );
        assert_eq!(
            evidence.entries[1].sources,
            vec![STRUCTURED_KIND_ARM.to_string(), CURRENT_ARM.to_string()]
        );
    }

    fn hit(key: &str, rank: usize) -> RetrievalHit {
        RetrievalHit {
            rank,
            key: key.to_string(),
            score: 1.0,
        }
    }
}
