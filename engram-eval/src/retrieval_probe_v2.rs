//! Attested semantic, hybrid, lifecycle, and abstention retrieval comparison.

use crate::fixture::materialize_engineering_context_fixture;
use crate::retrieval_probe::{
    add_corpus_expansions, aggregate_arm, require_empty_target, resolve_fixture_uri,
    resolved_scope_matches, score_query_run, validate_judgments, validate_protocol,
    RetrievalArmMetrics, RetrievalHit, RetrievalProtocol, RetrievalQuery, RetrievalQueryRun,
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

const RETRIEVAL_PROBE_V2_SCHEMA_VERSION: u32 = 2;
const CURRENT_LEXICAL_ARM: &str = "current_lexical";
const DENSE_ARM: &str = "dense_all_minilm_l6_v2";
const HYBRID_ARM: &str = "hybrid_rrf";
const LIFECYCLE_ARM: &str = "hybrid_lifecycle";
const CANDIDATE_ARM: &str = "hybrid_lifecycle_abstain";
const REQUIRED_MODEL_FILES: [&str; 5] = [
    "config.json",
    "model.onnx",
    "special_tokens_map.json",
    "tokenizer.json",
    "tokenizer_config.json",
];

/// Frozen reference to the v1 corpus and query protocol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BaseProtocolReference {
    /// Path relative to this protocol.
    pub path: String,
    /// Exact SHA-256 expected before parsing.
    pub sha256: String,
}

/// Exact local model identity and file hashes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalModelAttestation {
    /// Hugging Face repository identity.
    pub repository: String,
    /// Exact snapshot revision.
    pub revision: String,
    /// Expected output dimension.
    pub dimension: usize,
    /// Required pooling mode.
    pub pooling: String,
    /// Required filename to SHA-256 mapping.
    pub files: BTreeMap<String, String>,
}

/// Frozen ranking parameters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetrievalV2Ranking {
    /// Reciprocal-rank-fusion constant.
    pub rrf_k: usize,
    /// Score added to active successors of strongly matched inactive records.
    pub lifecycle_successor_boost: f64,
    /// Absolute dense-similarity threshold for non-lifecycle candidate results.
    pub dense_min_similarity: f64,
    /// Named opaque cue rule.
    pub opaque_cue_policy: String,
}

/// Frozen v2 gates.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetrievalV2QualityGates {
    /// Minimum current-arm recall over the expanded query set.
    pub current_min_recall_at_k: f64,
    /// Minimum current-arm MRR over the expanded query set.
    pub current_min_mrr: f64,
    /// Candidate arm to evaluate against candidate gates.
    pub candidate_arm: String,
    /// Minimum candidate recall.
    pub candidate_min_recall_at_k: f64,
    /// Minimum candidate MRR.
    pub candidate_min_mrr: f64,
    /// Minimum candidate abstention accuracy.
    pub candidate_min_abstention_accuracy: f64,
    /// Maximum out-of-scope results for any arm.
    pub max_scope_leakage: usize,
    /// Maximum inactive results for any arm.
    pub max_stale_results: usize,
    /// Maximum forbidden results for any arm.
    pub max_forbidden_results: usize,
    /// Maximum p95 query-embedding latency.
    pub semantic_query_p95_ms: u64,
}

/// Preregistered v2 protocol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetrievalV2Protocol {
    /// Protocol schema version.
    pub schema_version: u32,
    /// Stable probe identifier.
    pub probe_id: String,
    /// Frozen base corpus and queries.
    pub base_protocol: BaseProtocolReference,
    /// Rank cutoff.
    pub top_k: usize,
    /// Production search threshold.
    pub current_min_score: f32,
    /// Query embedding repetitions.
    pub repetitions: usize,
    /// Exact local model identity.
    pub model: LocalModelAttestation,
    /// Ordered arm IDs.
    pub arms: Vec<String>,
    /// Ranking and abstention parameters.
    pub ranking: RetrievalV2Ranking,
    /// Frozen quality and safety gates.
    pub quality_gates: RetrievalV2QualityGates,
    /// Additional discriminating queries layered over v1.
    pub additional_queries: Vec<RetrievalQuery>,
}

/// Timing evidence for the semantic arm.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticTiming {
    /// Time to load the attested ONNX snapshot.
    pub model_load_ms: f64,
    /// Time to embed the active corpus.
    pub corpus_embedding_ms: f64,
    /// Number of recorded query embeddings.
    pub query_embedding_samples: usize,
    /// Median query embedding latency.
    pub query_embedding_p50_ms: f64,
    /// 95th percentile query embedding latency.
    pub query_embedding_p95_ms: f64,
}

/// Reproducible retrieval v2 report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetrievalV2Report {
    /// Probe identifier.
    pub probe_id: String,
    /// SHA-256 of exact v2 protocol bytes.
    pub protocol_sha256: String,
    /// SHA-256 of exact v1 base protocol bytes.
    pub base_protocol_sha256: String,
    /// Frozen fixture content revision.
    pub fixture_revision: String,
    /// Model identity whose files passed attestation.
    pub model: LocalModelAttestation,
    /// Observed hashes, equal to the frozen model hashes on any completed run.
    pub observed_model_files: BTreeMap<String, String>,
    /// Number of all records in the isolated store.
    pub corpus_record_count: usize,
    /// Number of active records embedded.
    pub active_record_count: usize,
    /// Number of frozen queries.
    pub query_count: usize,
    /// Rank cutoff.
    pub top_k: usize,
    /// Semantic latency evidence.
    pub timing: SemanticTiming,
    /// Aggregate metrics by arm.
    pub arms: Vec<RetrievalArmMetrics>,
    /// Query-level evidence.
    pub runs: Vec<RetrievalQueryRun>,
    /// Frozen gate violations.
    pub gate_violations: Vec<String>,
    /// True only when every frozen gate passes.
    pub passed: bool,
    /// Claims this probe cannot support.
    pub limitations: Vec<String>,
}

/// Run the frozen provider-free v2 retrieval comparison.
pub async fn run_retrieval_probe_v2(
    protocol_path: &Path,
    model_snapshot: &Path,
    output: &Path,
) -> EvalResult<RetrievalV2Report> {
    let protocol_bytes = fs::read(protocol_path)?;
    let protocol: RetrievalV2Protocol = serde_json::from_slice(&protocol_bytes)?;
    validate_v2_protocol(&protocol)?;
    let protocol_sha256 = sha256_bytes(&protocol_bytes);

    let base_path = protocol_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(&protocol.base_protocol.path);
    let base_bytes = fs::read(&base_path)?;
    let base_sha256 = sha256_bytes(&base_bytes);
    if base_sha256 != protocol.base_protocol.sha256 {
        return Err(EvalError::Invalid(format!(
            "base protocol hash mismatch: observed {base_sha256}, expected {}",
            protocol.base_protocol.sha256
        )));
    }
    let base: RetrievalProtocol = serde_json::from_slice(&base_bytes)?;
    validate_protocol(&base)?;
    if base.top_k != protocol.top_k || base.current_min_score != protocol.current_min_score {
        return Err(EvalError::Invalid(
            "v2 top_k and current_min_score must match the frozen v1 protocol".to_string(),
        ));
    }

    let observed_model_files = attest_model_snapshot(&protocol.model, model_snapshot)?;
    let model_start = Instant::now();
    let embedder =
        Embedder::from_local_snapshot(EmbedConfig::default(), model_snapshot).map_err(invalid)?;
    let model_load_ms = elapsed_ms(model_start);
    if embedder.dimension() != protocol.model.dimension {
        return Err(EvalError::Invalid(format!(
            "model dimension mismatch: observed {}, expected {}",
            embedder.dimension(),
            protocol.model.dimension
        )));
    }

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
    let mut queries = base.queries;
    queries.extend(protocol.additional_queries.clone());
    let combined_protocol = RetrievalProtocol {
        queries: queries.clone(),
        ..base
    };
    validate_judgments(&combined_protocol, &all_items, &key_by_id)?;
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
    let mut query_latencies = Vec::with_capacity(queries.len() * protocol.repetitions);
    let mut runs = Vec::with_capacity(queries.len() * protocol.arms.len());
    for query in &queries {
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
            CURRENT_LEXICAL_ARM,
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
            EvalError::Invalid("v2 repetitions unexpectedly produced no embedding".to_string())
        })?;
        let scoped_active = active_items
            .iter()
            .filter(|item| resolved_scope_matches(item, query))
            .collect::<Vec<_>>();
        let dense_hits = dense_hits(
            &query_vector,
            &scoped_active,
            &key_by_id,
            &vectors_by_key,
            protocol.top_k,
        );
        runs.push(score_query_run(
            DENSE_ARM,
            query,
            dense_hits.clone(),
            &item_by_key,
        ));

        let hybrid_hits = rrf_hits(
            &current_hits,
            &dense_hits,
            protocol.ranking.rrf_k,
            protocol.top_k,
        );
        runs.push(score_query_run(
            HYBRID_ARM,
            query,
            hybrid_hits.clone(),
            &item_by_key,
        ));

        let successor_keys = lifecycle_successors(query, &all_items, &key_by_id);
        let lifecycle_hits = lifecycle_hits(
            &hybrid_hits,
            &successor_keys,
            protocol.ranking.lifecycle_successor_boost,
            protocol.top_k,
        );
        runs.push(score_query_run(
            LIFECYCLE_ARM,
            query,
            lifecycle_hits.clone(),
            &item_by_key,
        ));

        let candidate_hits = if should_abstain(
            query,
            &scoped_active,
            &dense_hits,
            &successor_keys,
            protocol.ranking.dense_min_similarity,
        ) {
            Vec::new()
        } else {
            lifecycle_hits
        };
        runs.push(score_query_run(
            CANDIDATE_ARM,
            query,
            candidate_hits,
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
        .map(|arm| aggregate_arm(arm, &queries, &runs))
        .collect::<Vec<_>>();
    let gate_violations = gate_violations(&protocol.quality_gates, &arms, &timing);
    let report = RetrievalV2Report {
        probe_id: protocol.probe_id,
        protocol_sha256,
        base_protocol_sha256: base_sha256,
        fixture_revision: seed.fixture_revision,
        model: protocol.model,
        observed_model_files,
        corpus_record_count: all_items.len(),
        active_record_count: active_items.len(),
        query_count: queries.len(),
        top_k: protocol.top_k,
        timing,
        arms,
        runs,
        passed: gate_violations.is_empty(),
        gate_violations,
        limitations: vec![
            "The corpus is deterministic and adversarial but synthetic; natural production query distributions require a separate shadow evaluation.".to_string(),
            "The lifecycle arm follows explicit supersedes edges only after a strong lexical match to inactive content; it does not infer arbitrary graph paths.".to_string(),
            "The abstention threshold is frozen for this corpus and model. Calibration on representative held-out user queries remains required before production use.".to_string(),
            "Ranked recall does not establish that an agent applies retrieved context correctly; host-native task success remains separately approval-gated.".to_string(),
        ],
    };
    fs::write(
        output.join("retrieval-report.json"),
        serde_json::to_string_pretty(&report)? + "\n",
    )?;
    Ok(report)
}

pub(crate) fn validate_v2_protocol(protocol: &RetrievalV2Protocol) -> EvalResult<()> {
    if protocol.schema_version != RETRIEVAL_PROBE_V2_SCHEMA_VERSION {
        return Err(EvalError::Invalid(format!(
            "unsupported retrieval v2 schema version: {}",
            protocol.schema_version
        )));
    }
    if protocol.probe_id.trim().is_empty()
        || protocol.top_k == 0
        || protocol.repetitions == 0
        || protocol.additional_queries.is_empty()
    {
        return Err(EvalError::Invalid(
            "v2 requires a probe ID, top_k, repetitions, and additional queries".to_string(),
        ));
    }
    let expected_arms = [
        CURRENT_LEXICAL_ARM,
        DENSE_ARM,
        HYBRID_ARM,
        LIFECYCLE_ARM,
        CANDIDATE_ARM,
    ];
    if protocol.arms.iter().map(String::as_str).collect::<Vec<_>>() != expected_arms {
        return Err(EvalError::Invalid(format!(
            "v2 arms must be exactly {}",
            expected_arms.join(", ")
        )));
    }
    if protocol.model.repository != "Qdrant/all-MiniLM-L6-v2-onnx"
        || protocol.model.dimension != 384
        || protocol.model.pooling != "mean"
        || protocol.model.files.len() != REQUIRED_MODEL_FILES.len()
        || REQUIRED_MODEL_FILES
            .iter()
            .any(|file| !protocol.model.files.contains_key(*file))
    {
        return Err(EvalError::Invalid(
            "v2 requires the exact five-file all-MiniLM-L6-v2 ONNX snapshot".to_string(),
        ));
    }
    if protocol.ranking.rrf_k == 0
        || !(0.0..=1.0).contains(&protocol.ranking.dense_min_similarity)
        || protocol.ranking.lifecycle_successor_boost <= 0.0
        || protocol.ranking.opaque_cue_policy != "require_exact_scoped_match"
        || protocol.quality_gates.candidate_arm != CANDIDATE_ARM
    {
        return Err(EvalError::Invalid(
            "invalid v2 ranking or candidate configuration".to_string(),
        ));
    }
    for threshold in [
        protocol.quality_gates.current_min_recall_at_k,
        protocol.quality_gates.current_min_mrr,
        protocol.quality_gates.candidate_min_recall_at_k,
        protocol.quality_gates.candidate_min_mrr,
        protocol.quality_gates.candidate_min_abstention_accuracy,
    ] {
        if !(0.0..=1.0).contains(&threshold) {
            return Err(EvalError::Invalid(
                "v2 quality thresholds must be in [0, 1]".to_string(),
            ));
        }
    }
    let mut ids = BTreeSet::new();
    for query in &protocol.additional_queries {
        if query.id.trim().is_empty() || !ids.insert(query.id.as_str()) {
            return Err(EvalError::Invalid(format!(
                "invalid or duplicate v2 query ID: {}",
                query.id
            )));
        }
        if query.abstain == query.relevant_keys.is_empty() {
            continue;
        }
        return Err(EvalError::Invalid(format!(
            "v2 query {} has inconsistent abstention judgments",
            query.id
        )));
    }
    Ok(())
}

pub(crate) fn attest_model_snapshot(
    expected: &LocalModelAttestation,
    snapshot: &Path,
) -> EvalResult<BTreeMap<String, String>> {
    if snapshot.file_name().and_then(|name| name.to_str()) != Some(expected.revision.as_str()) {
        return Err(EvalError::Invalid(format!(
            "model snapshot directory must be named with revision {}",
            expected.revision
        )));
    }
    let mut observed = BTreeMap::new();
    for name in REQUIRED_MODEL_FILES {
        let path = snapshot.join(name);
        let bytes = fs::read(&path).map_err(|error| {
            EvalError::Invalid(format!(
                "could not read attested model file {}: {error}",
                path.display()
            ))
        })?;
        let digest = sha256_bytes(&bytes);
        let wanted = expected.files.get(name).ok_or_else(|| {
            EvalError::Invalid(format!("protocol omits required model file {name}"))
        })?;
        if &digest != wanted {
            return Err(EvalError::Invalid(format!(
                "model file hash mismatch for {name}: observed {digest}, expected {wanted}"
            )));
        }
        observed.insert(name.to_string(), digest);
    }
    Ok(observed)
}

pub(crate) fn dense_hits(
    query: &[f32],
    items: &[&MemoryItem],
    key_by_id: &BTreeMap<String, String>,
    vectors_by_key: &BTreeMap<String, Vec<f32>>,
    top_k: usize,
) -> Vec<RetrievalHit> {
    let mut scored = items
        .iter()
        .filter_map(|item| {
            let key = key_by_id.get(&item.id.to_string())?;
            let vector = vectors_by_key.get(key)?;
            Some((key.clone(), cosine_similarity(query, vector)))
        })
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| {
        right
            .1
            .total_cmp(&left.1)
            .then_with(|| left.0.cmp(&right.0))
    });
    scored
        .into_iter()
        .take(top_k)
        .enumerate()
        .map(|(index, (key, score))| RetrievalHit {
            rank: index + 1,
            key,
            score,
        })
        .collect()
}

fn rrf_hits(
    lexical: &[RetrievalHit],
    dense: &[RetrievalHit],
    rrf_k: usize,
    top_k: usize,
) -> Vec<RetrievalHit> {
    let mut scores = BTreeMap::<String, f64>::new();
    for hit in lexical.iter().chain(dense) {
        *scores.entry(hit.key.clone()).or_default() += 1.0 / (rrf_k + hit.rank) as f64;
    }
    rank_scores(scores, top_k)
}

fn lifecycle_hits(
    hybrid: &[RetrievalHit],
    successors: &BTreeSet<String>,
    boost: f64,
    top_k: usize,
) -> Vec<RetrievalHit> {
    let mut scores = hybrid
        .iter()
        .map(|hit| (hit.key.clone(), hit.score))
        .collect::<BTreeMap<_, _>>();
    for successor in successors {
        *scores.entry(successor.clone()).or_default() += boost;
    }
    rank_scores(scores, top_k)
}

fn rank_scores(scores: BTreeMap<String, f64>, top_k: usize) -> Vec<RetrievalHit> {
    let mut scored = scores.into_iter().collect::<Vec<_>>();
    scored.sort_by(|left, right| {
        right
            .1
            .total_cmp(&left.1)
            .then_with(|| left.0.cmp(&right.0))
    });
    scored
        .into_iter()
        .take(top_k)
        .enumerate()
        .map(|(index, (key, score))| RetrievalHit {
            rank: index + 1,
            key,
            score,
        })
        .collect()
}

fn lifecycle_successors(
    query: &RetrievalQuery,
    items: &[MemoryItem],
    key_by_id: &BTreeMap<String, String>,
) -> BTreeSet<String> {
    let query_terms = terms(&query.query);
    if query_terms.is_empty() {
        return BTreeSet::new();
    }
    let matched_inactive = items
        .iter()
        .filter(|item| item.status != MemoryStatus::Active && resolved_scope_matches(item, query))
        .filter(|item| {
            let document_terms = terms(&memory_document_text(item));
            let overlap = query_terms.intersection(&document_terms).count();
            overlap > 0 && overlap as f64 / query_terms.len() as f64 >= 0.5
        })
        .map(|item| item.id.to_string())
        .collect::<BTreeSet<_>>();
    items
        .iter()
        .filter(|item| item.status == MemoryStatus::Active && resolved_scope_matches(item, query))
        .filter(|item| {
            item.supersedes
                .iter()
                .any(|id| matched_inactive.contains(&id.to_string()))
        })
        .filter_map(|item| key_by_id.get(&item.id.to_string()).cloned())
        .collect()
}

fn should_abstain(
    query: &RetrievalQuery,
    scoped_active: &[&MemoryItem],
    dense: &[RetrievalHit],
    lifecycle_successors: &BTreeSet<String>,
    dense_min_similarity: f64,
) -> bool {
    let opaque_cues = opaque_cues(&query.query);
    if !opaque_cues.is_empty()
        && opaque_cues.iter().any(|cue| {
            !scoped_active
                .iter()
                .any(|item| memory_document_text(item).contains(cue))
        })
    {
        return true;
    }
    lifecycle_successors.is_empty()
        && dense
            .first()
            .map_or(true, |hit| hit.score < dense_min_similarity)
}

pub(crate) fn opaque_cues(value: &str) -> Vec<String> {
    value
        .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .filter(|token| {
            token.contains('_') && token.chars().any(|character| character.is_uppercase())
        })
        .map(ToString::to_string)
        .collect()
}

pub(crate) fn memory_document_text(item: &MemoryItem) -> String {
    let mut text = format!(
        "{} {} {} {} {}",
        item.title,
        item.title,
        item.content,
        item.tags.join(" "),
        item.kind
    );
    if let Some(procedure) = &item.procedure {
        text.push(' ');
        text.push_str(&procedure.task);
        text.push(' ');
        text.push_str(&procedure.commands.join(" "));
        text.push(' ');
        text.push_str(&procedure.failure_signatures.join(" "));
        for prerequisite in &procedure.prerequisites {
            text.push(' ');
            text.push_str(&prerequisite.key);
            text.push(' ');
            text.push_str(&prerequisite.expected);
        }
    }
    text
}

fn terms(value: &str) -> BTreeSet<String> {
    value
        .to_ascii_lowercase()
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|term| term.len() >= 2)
        .filter(|term| !matches!(*term, "and" | "for" | "the" | "with"))
        .map(ToString::to_string)
        .collect()
}

fn cosine_similarity(left: &[f32], right: &[f32]) -> f64 {
    if left.len() != right.len() || left.is_empty() {
        return -1.0;
    }
    let mut dot = 0.0_f64;
    let mut left_norm = 0.0_f64;
    let mut right_norm = 0.0_f64;
    for (left, right) in left.iter().zip(right) {
        let left = f64::from(*left);
        let right = f64::from(*right);
        dot += left * right;
        left_norm += left * left;
        right_norm += right * right;
    }
    let denominator = left_norm.sqrt() * right_norm.sqrt();
    if denominator == 0.0 {
        -1.0
    } else {
        dot / denominator
    }
}

fn gate_violations(
    gates: &RetrievalV2QualityGates,
    arms: &[RetrievalArmMetrics],
    timing: &SemanticTiming,
) -> Vec<String> {
    let mut violations = Vec::new();
    match arms.iter().find(|arm| arm.arm == CURRENT_LEXICAL_ARM) {
        Some(current) => {
            if current.recall_at_k < gates.current_min_recall_at_k {
                violations.push(format!(
                    "current_recall_at_k:{:.4}<{}",
                    current.recall_at_k, gates.current_min_recall_at_k
                ));
            }
            if current.mean_reciprocal_rank < gates.current_min_mrr {
                violations.push(format!(
                    "current_mrr:{:.4}<{}",
                    current.mean_reciprocal_rank, gates.current_min_mrr
                ));
            }
        }
        None => violations.push("missing_current_lexical_arm".to_string()),
    }
    match arms.iter().find(|arm| arm.arm == gates.candidate_arm) {
        Some(candidate) => {
            if candidate.recall_at_k < gates.candidate_min_recall_at_k {
                violations.push(format!(
                    "candidate_recall_at_k:{:.4}<{}",
                    candidate.recall_at_k, gates.candidate_min_recall_at_k
                ));
            }
            if candidate.mean_reciprocal_rank < gates.candidate_min_mrr {
                violations.push(format!(
                    "candidate_mrr:{:.4}<{}",
                    candidate.mean_reciprocal_rank, gates.candidate_min_mrr
                ));
            }
            if candidate.abstention_accuracy < gates.candidate_min_abstention_accuracy {
                violations.push(format!(
                    "candidate_abstention_accuracy:{:.4}<{}",
                    candidate.abstention_accuracy, gates.candidate_min_abstention_accuracy
                ));
            }
        }
        None => violations.push("missing_candidate_arm".to_string()),
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
    fn opaque_cue_requires_an_underscore_and_uppercase() {
        assert_eq!(
            opaque_cues("deploy ORBIT_ONLY_CANARY now"),
            ["ORBIT_ONLY_CANARY"]
        );
        assert!(opaque_cues("ordinary words and lower_case").is_empty());
    }

    #[test]
    fn cosine_similarity_handles_identity_and_dimension_mismatch() {
        assert!((cosine_similarity(&[1.0, 0.0], &[1.0, 0.0]) - 1.0).abs() < f64::EPSILON);
        assert_eq!(cosine_similarity(&[1.0], &[1.0, 0.0]), -1.0);
    }

    #[test]
    fn percentile_uses_nearest_rank() {
        assert_eq!(percentile(&[1.0, 2.0, 3.0, 4.0], 0.50), 3.0);
        assert_eq!(percentile(&[1.0, 2.0, 3.0, 4.0], 0.95), 4.0);
    }
}
