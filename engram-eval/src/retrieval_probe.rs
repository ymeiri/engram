//! Provider-free corpus-by-retrieval comparison for engineering memory.

use crate::fixture::{materialize_engineering_context_fixture, FixtureLayout};
use crate::seed::open_seeded_engineering_context_fixture;
use crate::{EvalError, EvalResult};
use engram_core::memory::{
    ClaimOrigin, EvidenceKind, EvidenceRef, Harness, MemoryItem, MemoryKind, MemoryScope,
    MemoryStatus, ModelIdentity, WriterProvenance,
};
use engram_index::{MemoryService, SearchOptions, SearchService};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

const RETRIEVAL_PROBE_SCHEMA_VERSION: u32 = 1;
const CURRENT_LEXICAL_ARM: &str = "current_lexical";
const EXACT_OVERLAP_ARM: &str = "exact_overlap_resolved_scope";
const BM25_ARM: &str = "bm25_resolved_scope";

/// Frozen pass/fail gates for the retrieval probe.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetrievalQualityGates {
    /// Minimum production-arm recall at K over non-abstention queries.
    pub current_min_recall_at_k: f64,
    /// Minimum production-arm mean reciprocal rank over non-abstention queries.
    pub current_min_mrr: f64,
    /// Maximum results whose durable scope does not match the resolved query scope.
    pub max_scope_leakage: usize,
    /// Maximum inactive results returned by an arm.
    pub max_stale_results: usize,
    /// Maximum explicitly forbidden results returned by an arm.
    pub max_forbidden_results: usize,
    /// Maximum abstention queries on which an arm returned any result.
    pub max_abstention_false_positives: usize,
}

/// Additional frozen corpus record layered onto the engineering-context fixture.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetrievalCorpusRecord {
    /// Stable semantic key used for scoring.
    pub key: String,
    /// Memory kind accepted by `MemoryKind::parse`.
    pub kind: String,
    /// Human-readable title.
    pub title: String,
    /// Durable memory content.
    pub content: String,
    /// Project or repository boundary.
    pub scope: RetrievalScope,
    /// Retrieval tags.
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Scope supported by the frozen retrieval corpus.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RetrievalScope {
    /// Project-scoped record.
    Project {
        /// Stable project name.
        project_name: String,
    },
    /// Repository-scoped record.
    Repository {
        /// Canonical repository remote.
        remote_url: String,
    },
}

/// One frozen retrieval query and its judgments.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetrievalQuery {
    /// Stable query identifier.
    pub id: String,
    /// Failure-mode category.
    pub category: String,
    /// Natural-language or stable-key query.
    pub query: String,
    /// Resolved project boundary.
    pub project: String,
    /// Fixture URI for the production search path.
    pub cwd: String,
    /// Stable remote known after repository resolution.
    pub repository_remote: String,
    /// Relevant semantic keys accepted for this query.
    #[serde(default)]
    pub relevant_keys: Vec<String>,
    /// Keys that must never appear.
    #[serde(default)]
    pub forbidden_keys: Vec<String>,
    /// Whether the arm should return no result.
    #[serde(default)]
    pub abstain: bool,
}

/// Preregistered corpus-by-retrieval protocol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetrievalProtocol {
    /// Protocol schema version.
    pub schema_version: u32,
    /// Stable probe identifier.
    pub probe_id: String,
    /// Rank cutoff.
    pub top_k: usize,
    /// Production search minimum score.
    pub current_min_score: f32,
    /// Frozen quality and safety gates.
    pub quality_gates: RetrievalQualityGates,
    /// Deterministic additions to the base fixture corpus.
    pub corpus_expansions: Vec<RetrievalCorpusRecord>,
    /// Frozen queries and relevance judgments.
    pub queries: Vec<RetrievalQuery>,
}

/// One ranked result with its semantic key.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetrievalHit {
    /// One-based rank.
    pub rank: usize,
    /// Stable semantic key.
    pub key: String,
    /// Arm-specific score.
    pub score: f64,
}

/// One query result from one retrieval arm.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetrievalQueryRun {
    /// Retrieval arm.
    pub arm: String,
    /// Frozen query ID.
    pub query_id: String,
    /// Query category.
    pub category: String,
    /// Ranked results.
    pub hits: Vec<RetrievalHit>,
    /// Rank of the first judged-relevant result.
    pub first_relevant_rank: Option<usize>,
    /// Reciprocal rank for the query.
    pub reciprocal_rank: f64,
    /// Whether any judged-relevant result appeared in the top K.
    pub recalled_at_k: bool,
    /// Returned keys outside the resolved durable scope.
    pub scope_leakage_keys: Vec<String>,
    /// Returned keys that are not active.
    pub stale_keys: Vec<String>,
    /// Returned keys explicitly forbidden by the judgment.
    pub forbidden_keys: Vec<String>,
    /// Whether an abstention query produced any result.
    pub abstention_false_positive: bool,
}

/// Aggregate arm metrics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetrievalArmMetrics {
    /// Retrieval arm.
    pub arm: String,
    /// Number of non-abstention quality queries.
    pub quality_query_count: usize,
    /// Fraction of quality queries with a relevant top-K result.
    pub recall_at_k: f64,
    /// Mean reciprocal rank over quality queries.
    pub mean_reciprocal_rank: f64,
    /// Fraction of abstention queries returning no result.
    pub abstention_accuracy: f64,
    /// Total out-of-scope results.
    pub scope_leakage_count: usize,
    /// Total inactive results.
    pub stale_result_count: usize,
    /// Total explicitly forbidden results.
    pub forbidden_result_count: usize,
    /// Abstention queries that returned at least one result.
    pub abstention_false_positive_count: usize,
}

/// Reproducible provider-free retrieval report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetrievalReport {
    /// Probe identifier.
    pub probe_id: String,
    /// SHA-256 of exact preregistered protocol bytes.
    pub protocol_sha256: String,
    /// Frozen fixture content revision.
    pub fixture_revision: String,
    /// Number of all records in the isolated store.
    pub corpus_record_count: usize,
    /// Number of active candidate records.
    pub active_record_count: usize,
    /// Rank cutoff.
    pub top_k: usize,
    /// Frozen quality gates.
    pub quality_gates: RetrievalQualityGates,
    /// Aggregate metrics by arm.
    pub arms: Vec<RetrievalArmMetrics>,
    /// Query-level evidence.
    pub runs: Vec<RetrievalQueryRun>,
    /// Frozen gate violations.
    pub gate_violations: Vec<String>,
    /// True only when all frozen gates pass.
    pub passed: bool,
    /// Claims this probe cannot support.
    pub limitations: Vec<String>,
}

/// Materialize, seed, expand, run, and score the frozen provider-free retrieval probe.
pub async fn run_retrieval_probe(
    protocol_path: &Path,
    output: &Path,
) -> EvalResult<RetrievalReport> {
    require_empty_target(output)?;
    let protocol_bytes = fs::read(protocol_path)?;
    let protocol: RetrievalProtocol = serde_json::from_slice(&protocol_bytes)?;
    validate_protocol(&protocol)?;

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
    add_corpus_expansions(
        &memory,
        &protocol.corpus_expansions,
        &mut key_by_id,
        protocol_path,
    )
    .await?;

    let all_items = memory.list_memory(None, None).await.map_err(invalid)?;
    let active_items = all_items
        .iter()
        .filter(|item| item.status == MemoryStatus::Active)
        .cloned()
        .collect::<Vec<_>>();
    validate_judgments(&protocol, &all_items, &key_by_id)?;
    let item_by_key = all_items
        .iter()
        .filter_map(|item| {
            key_by_id
                .get(&item.id.to_string())
                .map(|key| (key.clone(), item.clone()))
        })
        .collect::<BTreeMap<_, _>>();
    let search = SearchService::new(db);

    let mut runs = Vec::with_capacity(protocol.queries.len() * 3);
    for query in &protocol.queries {
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
            .map(|(index, result)| {
                let key = key_by_id
                    .get(&result.id)
                    .cloned()
                    .unwrap_or_else(|| format!("unmapped:{}", result.id));
                RetrievalHit {
                    rank: index + 1,
                    key,
                    score: f64::from(result.score),
                }
            })
            .collect::<Vec<_>>();
        runs.push(score_query_run(
            CURRENT_LEXICAL_ARM,
            query,
            current_hits,
            &item_by_key,
        ));

        let scoped = active_items
            .iter()
            .filter(|item| resolved_scope_matches(item, query))
            .collect::<Vec<_>>();
        runs.push(score_query_run(
            EXACT_OVERLAP_ARM,
            query,
            exact_overlap_hits(&query.query, &scoped, &key_by_id, protocol.top_k),
            &item_by_key,
        ));
        runs.push(score_query_run(
            BM25_ARM,
            query,
            bm25_hits(&query.query, &scoped, &key_by_id, protocol.top_k),
            &item_by_key,
        ));
    }

    let arms = [CURRENT_LEXICAL_ARM, EXACT_OVERLAP_ARM, BM25_ARM]
        .into_iter()
        .map(|arm| aggregate_arm(arm, &protocol.queries, &runs))
        .collect::<Vec<_>>();
    let gate_violations = gate_violations(&protocol.quality_gates, &arms);
    let report = RetrievalReport {
        probe_id: protocol.probe_id,
        protocol_sha256: format!("{:x}", Sha256::digest(&protocol_bytes)),
        fixture_revision: seed.fixture_revision,
        corpus_record_count: all_items.len(),
        active_record_count: active_items.len(),
        top_k: protocol.top_k,
        quality_gates: protocol.quality_gates,
        arms,
        runs,
        passed: gate_violations.is_empty(),
        gate_violations,
        limitations: vec![
            "The BM25 and overlap arms receive the already-resolved repository remote; the production arm measures its current public project/cwd search contract.".to_string(),
            "No dense embedding, reranker, graph expansion, or agentic retrieval arm is included, so this report cannot select among those retrieval families.".to_string(),
            "The corpus is deterministic and adversarial but synthetic; host-native task success and natural production query distributions require separate evaluations.".to_string(),
            "Procedure retrieval is measured only as ranked recall. Applicability, prerequisites, evidence integrity, freshness, and execution remain separate hard gates.".to_string(),
        ],
    };
    fs::write(
        output.join("retrieval-report.json"),
        serde_json::to_string_pretty(&report)? + "\n",
    )?;
    Ok(report)
}

pub(crate) async fn add_corpus_expansions(
    memory: &MemoryService,
    records: &[RetrievalCorpusRecord],
    key_by_id: &mut BTreeMap<String, String>,
    protocol_path: &Path,
) -> EvalResult<()> {
    let writer = WriterProvenance::agent(
        Harness::Other("engram_eval".to_string()),
        ModelIdentity::new("deterministic", "retrieval-probe-seeder"),
    )
    .with_surface("engram-eval");
    for record in records {
        let scope = match &record.scope {
            RetrievalScope::Project { project_name } => MemoryScope::project(project_name.clone()),
            RetrievalScope::Repository { remote_url } => {
                MemoryScope::repository(Some(remote_url.clone()), None)
            }
        };
        let mut item = MemoryItem::new(
            MemoryKind::parse(&record.kind),
            &record.title,
            &record.content,
            scope,
            ClaimOrigin::AgentObserved,
            writer.clone(),
        )
        .with_tag(record.key.clone())
        .with_evidence(EvidenceRef::new(
            EvidenceKind::File,
            protocol_path.display().to_string(),
        ));
        for tag in &record.tags {
            item = item.with_tag(tag.clone());
        }
        let captured = memory.capture_memory(item).await.map_err(invalid)?;
        key_by_id.insert(captured.id.to_string(), record.key.clone());
    }
    Ok(())
}

fn exact_overlap_hits(
    query: &str,
    items: &[&MemoryItem],
    key_by_id: &BTreeMap<String, String>,
    top_k: usize,
) -> Vec<RetrievalHit> {
    let query_terms = token_counts(query);
    let mut scored = items
        .iter()
        .filter_map(|item| {
            let terms = memory_token_counts(item);
            let overlap = query_terms
                .keys()
                .filter(|term| terms.contains_key(*term))
                .count();
            if overlap == 0 {
                return None;
            }
            let score = overlap as f64 / query_terms.len().max(1) as f64;
            key_by_id
                .get(&item.id.to_string())
                .cloned()
                .map(|key| (key, score))
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

fn bm25_hits(
    query: &str,
    items: &[&MemoryItem],
    key_by_id: &BTreeMap<String, String>,
    top_k: usize,
) -> Vec<RetrievalHit> {
    let query_terms = token_counts(query).into_keys().collect::<BTreeSet<_>>();
    if query_terms.is_empty() || items.is_empty() {
        return Vec::new();
    }
    let documents = items
        .iter()
        .map(|item| memory_token_counts(item))
        .collect::<Vec<_>>();
    let average_length = documents
        .iter()
        .map(|document| document.values().sum::<usize>())
        .sum::<usize>() as f64
        / documents.len() as f64;
    let document_count = documents.len() as f64;
    let mut scored = items
        .iter()
        .zip(documents.iter())
        .filter_map(|(item, document)| {
            let document_length = document.values().sum::<usize>() as f64;
            let score = query_terms.iter().fold(0.0, |score, term| {
                let term_frequency = *document.get(term).unwrap_or(&0) as f64;
                if term_frequency == 0.0 {
                    return score;
                }
                let matching_documents = documents
                    .iter()
                    .filter(|candidate| candidate.contains_key(term))
                    .count() as f64;
                let inverse_document_frequency = ((document_count - matching_documents + 0.5)
                    / (matching_documents + 0.5)
                    + 1.0)
                    .ln();
                let k1 = 1.2;
                let b = 0.75;
                let denominator =
                    term_frequency + k1 * (1.0 - b + b * document_length / average_length.max(1.0));
                score + inverse_document_frequency * term_frequency * (k1 + 1.0) / denominator
            });
            if score <= 0.0 {
                return None;
            }
            key_by_id
                .get(&item.id.to_string())
                .cloned()
                .map(|key| (key, score))
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

fn memory_token_counts(item: &MemoryItem) -> HashMap<String, usize> {
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
    }
    token_counts(&text)
}

fn token_counts(value: &str) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for term in value
        .to_ascii_lowercase()
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|term| term.len() >= 3)
        .filter(|term| !is_stop_term(term))
    {
        *counts.entry(term.to_string()).or_insert(0) += 1;
    }
    counts
}

fn is_stop_term(term: &str) -> bool {
    matches!(
        term,
        "and"
            | "are"
            | "for"
            | "from"
            | "how"
            | "into"
            | "the"
            | "this"
            | "use"
            | "using"
            | "what"
            | "when"
            | "where"
            | "which"
            | "with"
    )
}

pub(crate) fn score_query_run(
    arm: &str,
    query: &RetrievalQuery,
    hits: Vec<RetrievalHit>,
    item_by_key: &BTreeMap<String, MemoryItem>,
) -> RetrievalQueryRun {
    let relevant = query.relevant_keys.iter().collect::<BTreeSet<_>>();
    let forbidden = query.forbidden_keys.iter().collect::<BTreeSet<_>>();
    let first_relevant_rank = hits
        .iter()
        .find(|hit| relevant.contains(&hit.key))
        .map(|hit| hit.rank);
    let scope_leakage_keys = hits
        .iter()
        .filter_map(|hit| {
            item_by_key
                .get(&hit.key)
                .filter(|item| !resolved_scope_matches(item, query))
                .map(|_| hit.key.clone())
        })
        .collect::<Vec<_>>();
    let stale_keys = hits
        .iter()
        .filter_map(|hit| {
            item_by_key
                .get(&hit.key)
                .filter(|item| item.status != MemoryStatus::Active)
                .map(|_| hit.key.clone())
        })
        .collect::<Vec<_>>();
    let forbidden_keys = hits
        .iter()
        .filter(|hit| forbidden.contains(&hit.key))
        .map(|hit| hit.key.clone())
        .collect::<Vec<_>>();
    let reciprocal_rank = first_relevant_rank.map_or(0.0, |rank| 1.0 / rank as f64);
    let recalled_at_k = first_relevant_rank.is_some();
    let abstention_false_positive = query.abstain && !hits.is_empty();
    RetrievalQueryRun {
        arm: arm.to_string(),
        query_id: query.id.clone(),
        category: query.category.clone(),
        hits,
        first_relevant_rank,
        reciprocal_rank,
        recalled_at_k,
        scope_leakage_keys,
        stale_keys,
        forbidden_keys,
        abstention_false_positive,
    }
}

pub(crate) fn aggregate_arm(
    arm: &str,
    queries: &[RetrievalQuery],
    runs: &[RetrievalQueryRun],
) -> RetrievalArmMetrics {
    let arm_runs = runs.iter().filter(|run| run.arm == arm).collect::<Vec<_>>();
    let quality_ids = queries
        .iter()
        .filter(|query| !query.abstain)
        .map(|query| query.id.as_str())
        .collect::<BTreeSet<_>>();
    let abstention_ids = queries
        .iter()
        .filter(|query| query.abstain)
        .map(|query| query.id.as_str())
        .collect::<BTreeSet<_>>();
    let quality_runs = arm_runs
        .iter()
        .filter(|run| quality_ids.contains(run.query_id.as_str()))
        .collect::<Vec<_>>();
    let recalled = quality_runs.iter().filter(|run| run.recalled_at_k).count();
    let reciprocal_rank_sum = quality_runs
        .iter()
        .map(|run| run.reciprocal_rank)
        .sum::<f64>();
    let abstention_runs = arm_runs
        .iter()
        .filter(|run| abstention_ids.contains(run.query_id.as_str()))
        .collect::<Vec<_>>();
    let abstention_correct = abstention_runs
        .iter()
        .filter(|run| !run.abstention_false_positive)
        .count();
    RetrievalArmMetrics {
        arm: arm.to_string(),
        quality_query_count: quality_runs.len(),
        recall_at_k: fraction(recalled, quality_runs.len()),
        mean_reciprocal_rank: if quality_runs.is_empty() {
            0.0
        } else {
            reciprocal_rank_sum / quality_runs.len() as f64
        },
        abstention_accuracy: fraction(abstention_correct, abstention_runs.len()),
        scope_leakage_count: arm_runs
            .iter()
            .map(|run| run.scope_leakage_keys.len())
            .sum(),
        stale_result_count: arm_runs.iter().map(|run| run.stale_keys.len()).sum(),
        forbidden_result_count: arm_runs.iter().map(|run| run.forbidden_keys.len()).sum(),
        abstention_false_positive_count: arm_runs
            .iter()
            .filter(|run| run.abstention_false_positive)
            .count(),
    }
}

fn gate_violations(gates: &RetrievalQualityGates, arms: &[RetrievalArmMetrics]) -> Vec<String> {
    let mut violations = Vec::new();
    let current = arms.iter().find(|arm| arm.arm == CURRENT_LEXICAL_ARM);
    match current {
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
        if arm.abstention_false_positive_count > gates.max_abstention_false_positives {
            violations.push(format!(
                "{}:abstention_false_positives:{}>{}",
                arm.arm, arm.abstention_false_positive_count, gates.max_abstention_false_positives
            ));
        }
    }
    violations
}

pub(crate) fn resolved_scope_matches(item: &MemoryItem, query: &RetrievalQuery) -> bool {
    match &item.scope {
        MemoryScope::Global | MemoryScope::User => true,
        MemoryScope::Project { project_name, .. } => {
            project_name.eq_ignore_ascii_case(&query.project)
        }
        MemoryScope::Task { project_name, .. } => project_name
            .as_deref()
            .is_some_and(|project| project.eq_ignore_ascii_case(&query.project)),
        MemoryScope::Repository { remote_url, .. } => remote_url.as_deref().is_some_and(|remote| {
            normalize_remote(remote) == normalize_remote(&query.repository_remote)
        }),
        MemoryScope::Entity { .. } | MemoryScope::Session { .. } | MemoryScope::Custom { .. } => {
            false
        }
    }
}

pub(crate) fn validate_protocol(protocol: &RetrievalProtocol) -> EvalResult<()> {
    if protocol.schema_version != RETRIEVAL_PROBE_SCHEMA_VERSION {
        return Err(EvalError::Invalid(format!(
            "unsupported retrieval probe schema version: {}",
            protocol.schema_version
        )));
    }
    if protocol.probe_id.trim().is_empty() || protocol.top_k == 0 || protocol.queries.is_empty() {
        return Err(EvalError::Invalid(
            "retrieval probe requires a probe ID, non-zero top_k, and queries".to_string(),
        ));
    }
    if !(0.0..=1.0).contains(&protocol.current_min_score)
        || !(0.0..=1.0).contains(&protocol.quality_gates.current_min_recall_at_k)
        || !(0.0..=1.0).contains(&protocol.quality_gates.current_min_mrr)
    {
        return Err(EvalError::Invalid(
            "retrieval score thresholds must be in [0, 1]".to_string(),
        ));
    }
    let mut keys = BTreeSet::new();
    for record in &protocol.corpus_expansions {
        if record.key.trim().is_empty()
            || record.title.trim().is_empty()
            || record.content.trim().is_empty()
            || !keys.insert(record.key.as_str())
        {
            return Err(EvalError::Invalid(format!(
                "invalid or duplicate corpus expansion key: {}",
                record.key
            )));
        }
    }
    let mut query_ids = BTreeSet::new();
    for query in &protocol.queries {
        if query.id.trim().is_empty()
            || query.query.trim().is_empty()
            || query.project.trim().is_empty()
            || query.repository_remote.trim().is_empty()
            || !query_ids.insert(query.id.as_str())
        {
            return Err(EvalError::Invalid(format!(
                "invalid or duplicate retrieval query: {}",
                query.id
            )));
        }
        if query.abstain && !query.relevant_keys.is_empty() {
            return Err(EvalError::Invalid(format!(
                "abstention query {} cannot declare relevant keys",
                query.id
            )));
        }
        if !query.abstain && query.relevant_keys.is_empty() {
            return Err(EvalError::Invalid(format!(
                "quality query {} requires at least one relevant key",
                query.id
            )));
        }
    }
    Ok(())
}

pub(crate) fn validate_judgments(
    protocol: &RetrievalProtocol,
    items: &[MemoryItem],
    key_by_id: &BTreeMap<String, String>,
) -> EvalResult<()> {
    let statuses = items
        .iter()
        .filter_map(|item| {
            key_by_id
                .get(&item.id.to_string())
                .map(|key| (key.as_str(), item.status))
        })
        .collect::<BTreeMap<_, _>>();
    for query in &protocol.queries {
        for key in query.relevant_keys.iter().chain(&query.forbidden_keys) {
            if !statuses.contains_key(key.as_str()) {
                return Err(EvalError::Invalid(format!(
                    "query {} references unknown key {key}",
                    query.id
                )));
            }
        }
        for key in &query.relevant_keys {
            if statuses.get(key.as_str()) != Some(&MemoryStatus::Active) {
                return Err(EvalError::Invalid(format!(
                    "query {} marks inactive key {key} relevant",
                    query.id
                )));
            }
        }
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

fn normalize_remote(value: &str) -> String {
    let value = value.trim().trim_end_matches('/');
    let value = value.strip_suffix(".git").unwrap_or(value);
    if let Some(rest) = value.strip_prefix("git@github.com:") {
        return format!("github.com/{}", rest.to_ascii_lowercase());
    }
    value
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .to_ascii_lowercase()
}

pub(crate) fn resolve_fixture_uri(layout: &FixtureLayout, uri: &str) -> EvalResult<PathBuf> {
    let (prefix, root) = layout
        .checkouts
        .iter()
        .filter(|(prefix, _)| uri == prefix.as_str() || uri.starts_with(&format!("{prefix}/")))
        .max_by_key(|(prefix, _)| prefix.len())
        .ok_or_else(|| EvalError::Invalid(format!("unmapped fixture URI: {uri}")))?;
    let suffix = uri
        .strip_prefix(prefix)
        .unwrap_or_default()
        .trim_start_matches('/');
    Ok(PathBuf::from(root).join(suffix))
}

pub(crate) fn require_empty_target(output: &Path) -> EvalResult<()> {
    if output.exists() && output.read_dir()?.next().is_some() {
        return Err(EvalError::Invalid(format!(
            "retrieval probe output is not empty: {}",
            output.display()
        )));
    }
    fs::create_dir_all(output)?;
    Ok(())
}

fn invalid(error: impl std::fmt::Display) -> EvalError {
    EvalError::Invalid(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bm25_prefers_the_specific_document() {
        let writer = WriterProvenance::agent(
            Harness::Other("test".to_string()),
            ModelIdentity::new("test", "test"),
        );
        let specific = MemoryItem::new(
            MemoryKind::Decision,
            "Queue retry policy",
            "Use full jitter for worker retries",
            MemoryScope::project("atlas"),
            ClaimOrigin::AgentObserved,
            writer.clone(),
        );
        let generic = MemoryItem::new(
            MemoryKind::Decision,
            "Worker policy",
            "Use a queue",
            MemoryScope::project("atlas"),
            ClaimOrigin::AgentObserved,
            writer,
        );
        let keys = BTreeMap::from([
            (specific.id.to_string(), "specific".to_string()),
            (generic.id.to_string(), "generic".to_string()),
        ]);
        let hits = bm25_hits(
            "queue worker retry full jitter",
            &[&specific, &generic],
            &keys,
            2,
        );
        assert_eq!(hits[0].key, "specific");
    }

    #[test]
    fn resolved_scope_excludes_lookalike_project() {
        let writer = WriterProvenance::agent(
            Harness::Other("test".to_string()),
            ModelIdentity::new("test", "test"),
        );
        let orbit = MemoryItem::new(
            MemoryKind::Decision,
            "Worker policy",
            "Orbit only",
            MemoryScope::project("orbit"),
            ClaimOrigin::AgentObserved,
            writer,
        );
        let query = RetrievalQuery {
            id: "scope".to_string(),
            category: "scope".to_string(),
            query: "worker".to_string(),
            project: "atlas".to_string(),
            cwd: "fixture://atlas/main".to_string(),
            repository_remote: "git@github.com:acme/atlas.git".to_string(),
            relevant_keys: Vec::new(),
            forbidden_keys: Vec::new(),
            abstain: true,
        };
        assert!(!resolved_scope_matches(&orbit, &query));
    }
}
