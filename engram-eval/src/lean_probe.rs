//! Provider-free measurement of the real lean orientation wire packet.

use crate::fixture::{materialize_engineering_context_fixture, FixtureLayout};
use crate::native_runner::write_private_json;
use crate::pilot::require_empty_target;
use crate::seed::{open_seeded_engineering_context_fixture, SeedReport};
use crate::{load_suite, EvalError, EvalResult, Journey, Scenario};
use engram_index::{MemoryService, RepositoryService};
use engram_mcp::tools::{self, OrientRequest, OrientResponseShape, ToolState};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

const LEAN_PROBE_SCHEMA_VERSION: u32 = 1;

/// Aggregate bounds frozen before a provider-free probe runs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LeanOrientTargets {
    /// Maximum median lean orientation latency.
    pub latency_p50_ms: u64,
    /// Maximum nearest-rank p95 lean orientation latency.
    pub latency_p95_ms: u64,
    /// Maximum median serialized lean response size.
    pub packet_p50_bytes: u64,
    /// Maximum nearest-rank p95 serialized lean response size.
    pub packet_p95_bytes: u64,
}

/// Preregistered provider-free lean orientation probe.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LeanOrientProtocol {
    /// Protocol schema version.
    pub schema_version: u32,
    /// Stable probe identifier.
    pub probe_id: String,
    /// Frozen engineering-context manifest, relative to this protocol.
    pub manifest: String,
    /// Scenarios whose first lean packet can be scored without an agent.
    pub scenario_ids: Vec<String>,
    /// Number of measurements per scenario.
    pub repetitions: u32,
    /// Aggregate latency and packet targets declared before execution.
    pub targets: LeanOrientTargets,
}

/// One measured lean orientation call.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LeanOrientRun {
    /// Frozen scenario ID.
    pub scenario_id: String,
    /// One-based repetition.
    pub repetition: u32,
    /// Serialized lean MCP response size.
    pub packet_bytes: u64,
    /// End-to-end in-process MCP tool latency.
    pub latency_ms: u64,
    /// Repository remote resolved from the same isolated store after orientation.
    pub repository_remote: Option<String>,
    /// Project selected by orientation.
    pub project: Option<String>,
    /// Components selected by orientation.
    pub components: Vec<String>,
    /// Semantic memory keys present in the bounded packet.
    pub returned_context_keys: Vec<String>,
    /// Whether every returned item explains why it appeared.
    pub explanations_complete: bool,
    /// Whether material unresolved identity was surfaced to the caller.
    pub ambiguity_reported: bool,
    /// Deterministic gate failures for this call.
    pub failures: Vec<String>,
}

/// Reproducible provider-free lean orientation report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LeanOrientReport {
    /// Probe identifier.
    pub probe_id: String,
    /// SHA-256 of the preregistered protocol bytes.
    pub protocol_sha256: String,
    /// Frozen fixture content revision.
    pub fixture_revision: String,
    /// Number of measured calls.
    pub run_count: usize,
    /// Observed median latency.
    pub latency_p50_ms: u64,
    /// Observed nearest-rank p95 latency.
    pub latency_p95_ms: u64,
    /// Observed median lean packet size.
    pub packet_p50_bytes: u64,
    /// Observed nearest-rank p95 lean packet size.
    pub packet_p95_bytes: u64,
    /// Frozen targets used to score this report.
    pub targets: LeanOrientTargets,
    /// Aggregate target violations.
    pub aggregate_violations: Vec<String>,
    /// True only when every semantic and resource gate passed.
    pub passed: bool,
    /// Per-call evidence.
    pub runs: Vec<LeanOrientRun>,
    /// Claims this provider-free probe deliberately does not make.
    pub limitations: Vec<String>,
}

/// Materialize, seed, measure, and score the real lean orientation MCP response without a host.
pub async fn run_lean_orient_probe(
    protocol_path: &Path,
    output: &Path,
) -> EvalResult<LeanOrientReport> {
    require_empty_target(output)?;
    let protocol_bytes = fs::read(protocol_path)?;
    let protocol: LeanOrientProtocol = serde_json::from_slice(&protocol_bytes)?;
    let manifest_path = resolve_relative(protocol_path, &protocol.manifest)?;
    let (_, scenarios) = load_suite(&manifest_path)?;
    let selected = validate_protocol(&protocol, &scenarios)?;

    let fixture_root = output.join("fixture");
    let data_dir = output.join("data");
    let layout = materialize_engineering_context_fixture(&fixture_root)?;
    let (seed, db) =
        open_seeded_engineering_context_fixture(&fixture_root.join("fixture-map.json"), &data_dir)
            .await?;
    let memory = MemoryService::new(db.clone());
    memory.init_schema().await.map_err(invalid)?;
    let repositories = RepositoryService::new(db);
    repositories.init_schema().await.map_err(invalid)?;
    let state = ToolState::new();
    state.init_memory(memory).await;

    let reverse_memory = seed
        .memory
        .iter()
        .map(|(key, id)| (id.clone(), key.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut runs = Vec::with_capacity(selected.len() * protocol.repetitions as usize);
    for repetition in 1..=protocol.repetitions {
        for scenario in &selected {
            runs.push(
                measure_scenario(
                    scenario,
                    repetition,
                    &layout,
                    &seed,
                    &state,
                    &repositories,
                    &reverse_memory,
                )
                .await?,
            );
        }
    }

    let latencies = runs.iter().map(|run| run.latency_ms).collect::<Vec<_>>();
    let packet_sizes = runs.iter().map(|run| run.packet_bytes).collect::<Vec<_>>();
    let latency_p50_ms = percentile(&latencies, 0.50);
    let latency_p95_ms = percentile(&latencies, 0.95);
    let packet_p50_bytes = percentile(&packet_sizes, 0.50);
    let packet_p95_bytes = percentile(&packet_sizes, 0.95);
    let aggregate_violations = aggregate_violations(
        &protocol.targets,
        latency_p50_ms,
        latency_p95_ms,
        packet_p50_bytes,
        packet_p95_bytes,
    );
    let passed = aggregate_violations.is_empty() && runs.iter().all(|run| run.failures.is_empty());
    let report = LeanOrientReport {
        probe_id: protocol.probe_id,
        protocol_sha256: format!("{:x}", Sha256::digest(&protocol_bytes)),
        fixture_revision: seed.fixture_revision,
        run_count: runs.len(),
        latency_p50_ms,
        latency_p95_ms,
        packet_p50_bytes,
        packet_p95_bytes,
        targets: protocol.targets,
        aggregate_violations,
        passed,
        runs,
        limitations: vec![
            "This probe measures Engram's in-process lean MCP response, not coding-agent task success, token usage, or provider latency.".to_string(),
            "Repository remote correctness is checked from the same isolated repository service; the lean packet intentionally requires progressive repository evidence retrieval.".to_string(),
            "The compaction/resume scenario sends a resume_session intent to a freshly seeded in-process store; it does not compact a host session, restart Engram, or cross a process boundary.".to_string(),
            "Verified procedure, secret-capture, deletion, and host-native-memory claims remain covered by their dedicated tests and execution-gated evaluations.".to_string(),
        ],
    };
    write_private_json(&output.join("lean-orient-report.json"), &report)?;
    Ok(report)
}

async fn measure_scenario(
    scenario: &Scenario,
    repetition: u32,
    layout: &FixtureLayout,
    seed: &SeedReport,
    state: &ToolState,
    repositories: &RepositoryService,
    reverse_memory: &BTreeMap<String, String>,
) -> EvalResult<LeanOrientRun> {
    let cwd = scenario
        .cwd
        .as_deref()
        .map(|uri| resolve_fixture_uri(layout, uri))
        .transpose()?;
    let started = Instant::now();
    let response = tools::orient(
        state,
        OrientRequest {
            cwd: cwd.as_ref().map(|path| path.display().to_string()),
            prompt: Some(scenario.prompt.clone()),
            project: scenario.project.clone(),
            task: None,
            agent: Some("provider_free_lean_probe".to_string()),
            external_session_id: None,
            intent: Some(scenario_intent(&scenario.journey).to_string()),
            scenario_id: Some(scenario.id.clone()),
            arm: Some("provider_free_lean_orient".to_string()),
            include_recent_commits: Some(true),
            limit: Some(5),
            response_shape: Some(OrientResponseShape::Lean),
        },
    )
    .await
    .map_err(|error| EvalError::Invalid(format!("orient {} failed: {error}", scenario.id)))?;
    let latency_ms = started.elapsed().as_millis() as u64;
    let packet_bytes = response.len() as u64;
    let value: Value = serde_json::from_str(&response)?;
    let repository_context = match cwd.as_deref() {
        Some(cwd) => repositories.resolve_cwd(cwd).await.map_err(invalid)?,
        None => None,
    };
    let repository_remote = repository_context
        .as_ref()
        .and_then(|context| context.repository.remote_url.clone());
    let project = value
        .pointer("/resolution/selected_project")
        .and_then(Value::as_str)
        .map(str::to_string);
    let components = value
        .pointer("/resolution/component_names")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect::<Vec<_>>();
    let returned_context_keys = returned_context_keys(&value, reverse_memory);
    let explanations_complete = explanations_complete(&value);
    let ambiguities = value
        .get("ambiguities")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    let requires_confirmation = value
        .pointer("/resolution/requires_confirmation")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let ambiguity_reported = requires_confirmation || ambiguities > 0;
    let mut failures = Vec::new();

    if scenario.expected.unresolved_identity {
        if repository_context.is_some() || project.is_some() || !components.is_empty() {
            failures.push("unresolved_identity_was_fabricated".to_string());
        }
        if !ambiguity_reported {
            failures.push("material_identity_ambiguity_not_reported".to_string());
        }
    } else {
        if let Some(expected) = scenario.expected.repository_remote.as_deref() {
            if repository_remote.as_deref().map(normalize_remote)
                != Some(normalize_remote(expected))
            {
                failures.push("repository_remote_mismatch".to_string());
            }
        }
        if scenario.expected.project != project {
            failures.push("project_mismatch".to_string());
        }
        if let Some(expected) = scenario.expected.component.as_deref() {
            if !components.iter().any(|component| component == expected) {
                failures.push("component_mismatch".to_string());
            }
        }
    }

    for required in &scenario.expected.required_context_keys {
        if !returned_context_keys.contains(required) {
            failures.push(format!("missing_context:{required}"));
        }
    }
    for forbidden in &scenario.expected.forbidden_context_keys {
        if returned_context_keys.contains(forbidden) {
            failures.push(format!("forbidden_context:{forbidden}"));
        }
    }
    if scenario.journey == Journey::NoResult && !returned_context_keys.is_empty() {
        failures.push("no_result_returned_context".to_string());
    }
    if !explanations_complete {
        failures.push("missing_why_relevant".to_string());
    }
    if scenario
        .budgets
        .context_packet_bytes
        .is_some_and(|limit| packet_bytes > limit)
    {
        failures.push("scenario_packet_budget_exceeded".to_string());
    }
    if value.get("response_shape").and_then(Value::as_str) != Some("lean") {
        failures.push("response_shape_not_lean".to_string());
    }
    if seed.fixture_revision != layout.fixture_revision {
        failures.push("fixture_revision_drift".to_string());
    }

    Ok(LeanOrientRun {
        scenario_id: scenario.id.clone(),
        repetition,
        packet_bytes,
        latency_ms,
        repository_remote,
        project,
        components,
        returned_context_keys,
        explanations_complete,
        ambiguity_reported,
        failures,
    })
}

fn scenario_intent(journey: &Journey) -> &'static str {
    match journey {
        Journey::CompactionResume => "resume_session",
        Journey::Decision | Journey::StaleMemory | Journey::NoResult => "answer_question",
        Journey::RepositoryIdentity => "answer_question",
        _ => "plan_work",
    }
}

fn validate_protocol<'a>(
    protocol: &LeanOrientProtocol,
    scenarios: &'a [Scenario],
) -> EvalResult<Vec<&'a Scenario>> {
    if protocol.schema_version != LEAN_PROBE_SCHEMA_VERSION {
        return Err(EvalError::Invalid(format!(
            "unsupported lean probe schema version: {}",
            protocol.schema_version
        )));
    }
    if protocol.probe_id.trim().is_empty() || protocol.repetitions == 0 {
        return Err(EvalError::Invalid(
            "lean probe requires a non-empty ID and positive repetitions".to_string(),
        ));
    }
    if protocol.scenario_ids.is_empty()
        || protocol.targets.latency_p50_ms == 0
        || protocol.targets.latency_p95_ms == 0
        || protocol.targets.packet_p50_bytes == 0
        || protocol.targets.packet_p95_bytes == 0
    {
        return Err(EvalError::Invalid(
            "lean probe requires scenarios and positive aggregate targets".to_string(),
        ));
    }
    let mut seen = BTreeSet::new();
    protocol
        .scenario_ids
        .iter()
        .map(|id| {
            if !seen.insert(id) {
                return Err(EvalError::Invalid(format!(
                    "lean probe contains duplicate scenario: {id}"
                )));
            }
            let scenario = scenarios
                .iter()
                .find(|scenario| scenario.id == *id)
                .ok_or_else(|| EvalError::Invalid(format!("unknown probe scenario: {id}")))?;
            if !matches!(
                scenario.journey,
                Journey::RepositoryIdentity
                    | Journey::Decision
                    | Journey::StaleMemory
                    | Journey::NoResult
                    | Journey::CompactionResume
            ) {
                return Err(EvalError::Invalid(format!(
                    "scenario {id} cannot be scored from orientation alone"
                )));
            }
            Ok(scenario)
        })
        .collect()
}

fn returned_context_keys(
    response: &Value,
    reverse_memory: &BTreeMap<String, String>,
) -> Vec<String> {
    let mut ids = BTreeSet::new();
    for pointer in ["/used_memory_candidate_ids", "/hot_context_ids"] {
        ids.extend(
            response
                .pointer(pointer)
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_string),
        );
    }
    ids.into_iter()
        .filter_map(|id| reverse_memory.get(&id).cloned())
        .collect()
}

fn explanations_complete(response: &Value) -> bool {
    ["/hot_context_items", "/brain_loop/top_items"]
        .iter()
        .flat_map(|pointer| {
            response
                .pointer(pointer)
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .all(|item| {
            item.get("why_relevant")
                .and_then(Value::as_str)
                .is_some_and(|why| !why.trim().is_empty())
        })
}

fn aggregate_violations(
    targets: &LeanOrientTargets,
    latency_p50_ms: u64,
    latency_p95_ms: u64,
    packet_p50_bytes: u64,
    packet_p95_bytes: u64,
) -> Vec<String> {
    let mut violations = Vec::new();
    if latency_p50_ms > targets.latency_p50_ms {
        violations.push("latency_p50_ms".to_string());
    }
    if latency_p95_ms > targets.latency_p95_ms {
        violations.push("latency_p95_ms".to_string());
    }
    if packet_p50_bytes > targets.packet_p50_bytes {
        violations.push("packet_p50_bytes".to_string());
    }
    if packet_p95_bytes > targets.packet_p95_bytes {
        violations.push("packet_p95_bytes".to_string());
    }
    violations
}

fn percentile(values: &[u64], percentile: f64) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let mut values = values.to_vec();
    values.sort_unstable();
    let rank = (percentile * values.len() as f64).ceil() as usize;
    values[rank.saturating_sub(1).min(values.len() - 1)]
}

fn resolve_relative(base: &Path, relative: &str) -> EvalResult<PathBuf> {
    let path = base
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(relative);
    Ok(path.canonicalize()?)
}

fn resolve_fixture_uri(layout: &FixtureLayout, uri: &str) -> EvalResult<PathBuf> {
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
    Ok(Path::new(root).join(suffix))
}

fn normalize_remote(value: &str) -> String {
    let trimmed = value.trim().trim_end_matches('/').trim_end_matches(".git");
    let lower = trimmed.to_ascii_lowercase();
    if let Some(rest) = lower.strip_prefix("git@") {
        return rest.replacen(':', "/", 1);
    }
    lower
        .strip_prefix("https://")
        .or_else(|| lower.strip_prefix("http://"))
        .or_else(|| lower.strip_prefix("ssh://git@"))
        .unwrap_or(&lower)
        .to_string()
}

fn invalid(error: impl std::fmt::Display) -> EvalError {
    EvalError::Invalid(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_rank_percentiles_are_deterministic() {
        let values = [5, 1, 4, 2, 3];
        assert_eq!(percentile(&values, 0.50), 3);
        assert_eq!(percentile(&values, 0.95), 5);
        assert_eq!(percentile(&[], 0.95), 0);
    }

    #[test]
    fn aggregate_targets_report_each_independent_violation() {
        let targets = LeanOrientTargets {
            latency_p50_ms: 10,
            latency_p95_ms: 20,
            packet_p50_bytes: 30,
            packet_p95_bytes: 40,
        };
        assert_eq!(
            aggregate_violations(&targets, 11, 20, 31, 41),
            vec!["latency_p50_ms", "packet_p50_bytes", "packet_p95_bytes"]
        );
    }

    #[test]
    fn semantic_key_extraction_uses_only_seeded_ids() {
        let response = serde_json::json!({
            "used_memory_candidate_ids": ["id-current", "unknown"],
            "hot_context_ids": ["id-current", "id-plan"]
        });
        let keys = returned_context_keys(
            &response,
            &BTreeMap::from([
                ("id-current".to_string(), "decision-current".to_string()),
                ("id-plan".to_string(), "plan-current".to_string()),
            ]),
        );
        assert_eq!(keys, vec!["decision-current", "plan-current"]);
    }

    #[test]
    fn explanation_gate_rejects_empty_why_relevant() {
        let complete = serde_json::json!({
            "hot_context_items": [{"why_relevant": "matched scope"}],
            "brain_loop": {"top_items": [{"why_relevant": "matched task"}]}
        });
        let incomplete = serde_json::json!({
            "hot_context_items": [{"why_relevant": ""}],
            "brain_loop": {"top_items": []}
        });
        assert!(explanations_complete(&complete));
        assert!(!explanations_complete(&incomplete));
    }
}
