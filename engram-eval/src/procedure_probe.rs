//! Provider-free evaluation of the real verified-procedure matcher.

use crate::fixture::{materialize_engineering_context_fixture, FixtureLayout};
use crate::native_runner::write_private_json;
use crate::pilot::require_empty_target;
use crate::retrieval_probe_v2::BaseProtocolReference;
use crate::seed::open_seeded_engineering_context_fixture;
use crate::{EvalError, EvalResult};
use engram_core::memory::{
    ClaimOrigin, Harness, MemoryItem, MemoryKind, MemoryScope, MemoryStatus, ModelIdentity,
    ProcedureCard, ProcedurePrerequisite, ProcedureVerification, WriterProvenance,
};
use engram_core::Id;
use engram_index::{MemoryService, ProcedureMatchInput, ProcedureVerificationReceipt};
use engram_store::MemoryRepo;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use time::{Duration, OffsetDateTime};

const PROCEDURE_PROBE_SCHEMA_VERSION: u32 = 1;
const PROCEDURE_REPLAY_SCHEMA_VERSION: u32 = 2;
const INTEGRATION_KEY: &str = "procedure-atlas-integration-v3";
const INTEGRATION_RECEIPT_URI: &str = "fixture://atlas/main/evidence/integration-v3-success.json";

const fn one_repetition() -> usize {
    1
}

/// Exact source file attested before the procedure baseline runs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceFileReference {
    /// Path relative to the protocol.
    pub path: String,
    /// Expected SHA-256.
    pub sha256: String,
}

/// One extra verified procedure used to exercise failure-signature matching.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdditionalProcedure {
    /// Stable evaluation key.
    pub key: String,
    /// Human-readable title.
    pub title: String,
    /// Durable content.
    pub content: String,
    /// Structured task.
    pub task: String,
    /// Successful commands.
    pub commands: Vec<String>,
    /// Exact prerequisites.
    pub prerequisites: BTreeMap<String, String>,
    /// Known failure signatures.
    pub failure_signatures: Vec<String>,
    /// Receipt command.
    pub verification_command: String,
    /// Expected receipt exit code.
    pub verification_exit_code: i32,
    /// Receipt output body containing the expected marker.
    pub verification_output: String,
    /// Conditions captured by the verification receipt.
    pub receipt_conditions: BTreeMap<String, String>,
    /// Project scope.
    pub project: String,
    /// Verification lifetime.
    pub expires_in_days: i64,
}

/// Frozen outcome, safety, evidence, and latency gates.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcedureQualityGates {
    /// Minimum exact returned-key and abstention outcome accuracy.
    pub min_exact_outcome_accuracy: f64,
    /// Minimum accuracy across expected-success scenarios.
    pub min_success_accuracy: f64,
    /// Minimum accuracy across expected-abstention scenarios.
    pub min_abstention_accuracy: f64,
    /// Minimum expected diagnostic-substring accuracy.
    pub min_diagnostic_accuracy: f64,
    /// Minimum receipt-hash evidence accuracy for returned procedures.
    pub min_evidence_accuracy: f64,
    /// Maximum returned procedures in any scenario.
    pub max_returned_per_scenario: usize,
    /// Maximum explicitly forbidden returned procedures.
    pub max_forbidden_results: usize,
    /// Maximum p95 matcher latency.
    pub query_p95_ms: u64,
}

/// One preregistered procedure-match scenario.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcedureScenario {
    /// Stable scenario ID.
    pub id: String,
    /// Failure-mode category.
    pub category: String,
    /// Task or failure text passed to the real matcher.
    pub query: String,
    /// Explicit project, if supplied.
    pub project: Option<String>,
    /// Fixture URI for cwd, if supplied.
    pub cwd: Option<String>,
    /// Exact caller-observed conditions.
    #[serde(default)]
    pub conditions: BTreeMap<String, String>,
    /// Isolated pre-query mutation.
    pub setup: String,
    /// Exact semantic keys that should be returned.
    #[serde(default)]
    pub expected_keys: Vec<String>,
    /// Semantic keys that must not be returned.
    #[serde(default)]
    pub forbidden_keys: Vec<String>,
    /// Expected matcher abstention state.
    pub expect_abstain: bool,
    /// Required fragments across deterministic diagnostic reasons.
    #[serde(default)]
    pub expected_diagnostic_substrings: Vec<String>,
}

/// Frozen real-matcher evaluation protocol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcedureProbeProtocol {
    /// Protocol schema version.
    pub schema_version: u32,
    /// Stable probe identifier.
    pub probe_id: String,
    /// Checked-in result that motivated this narrower probe.
    pub trigger_results: BaseProtocolReference,
    /// Exact production and fixture source files.
    pub source_files: Vec<SourceFileReference>,
    /// Matcher result limit.
    pub limit: usize,
    /// Extra verified failure-signature procedure.
    pub additional_procedure: AdditionalProcedure,
    /// Frozen quality and safety gates.
    pub quality_gates: ProcedureQualityGates,
    /// Frozen scenarios.
    pub scenarios: Vec<ProcedureScenario>,
}

/// Source-attested replay of an existing procedure matrix after one correction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcedureReplayProtocol {
    /// Replay schema version.
    pub schema_version: u32,
    /// Stable replay identifier.
    pub probe_id: String,
    /// Exact baseline protocol whose matrix and gates are inherited.
    pub baseline_protocol: BaseProtocolReference,
    /// Exact checked-in baseline result that justified the correction.
    pub trigger_results: BaseProtocolReference,
    /// Exact corrected source hashes replacing baseline entries by path.
    pub source_overrides: Vec<SourceFileReference>,
    /// Frozen correction policy.
    pub correction_policy: String,
    /// Number of complete inherited-matrix repetitions.
    #[serde(default = "one_repetition")]
    pub repetitions: usize,
}

/// Diagnostic decision for one candidate considered by the real matcher.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProcedureDiagnosticEvidence {
    /// Semantic key, when mapped.
    pub key: String,
    /// Whether all matcher checks passed.
    pub applicable: bool,
    /// Deterministic applicability reasons.
    pub reasons: Vec<String>,
}

/// One scored real-matcher scenario.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcedureScenarioRun {
    /// Scenario ID.
    pub scenario_id: String,
    /// One-based complete-matrix repetition.
    pub repetition: usize,
    /// Scenario category.
    pub category: String,
    /// Isolated setup mutation.
    pub setup: String,
    /// Resolved cwd used by the matcher.
    pub cwd: Option<String>,
    /// Returned semantic keys in matcher order.
    pub returned_keys: Vec<String>,
    /// Explicitly forbidden returned keys.
    pub forbidden_keys: Vec<String>,
    /// Matcher abstention state.
    pub abstained: bool,
    /// Whether returned keys and abstention exactly matched the judgment.
    pub exact_outcome: bool,
    /// Whether expected diagnostic fragments were present.
    pub diagnostics_accurate: bool,
    /// Whether every returned procedure still had complete, hash-matching evidence.
    pub evidence_accurate: bool,
    /// Matcher diagnostic evidence.
    pub diagnostics: Vec<ProcedureDiagnosticEvidence>,
    /// Matcher latency.
    pub latency_ms: f64,
    /// Matcher summary.
    pub message: String,
    /// Bounded follow-up actions returned by the matcher.
    pub next_actions: Vec<String>,
}

/// Aggregate real-matcher metrics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcedureProbeMetrics {
    /// Number of scenarios.
    pub scenario_count: usize,
    /// Fraction with exact returned keys and abstention.
    pub exact_outcome_accuracy: f64,
    /// Fraction of expected-success scenarios exactly correct.
    pub success_accuracy: f64,
    /// Fraction of expected-abstention scenarios exactly correct.
    pub abstention_accuracy: f64,
    /// Fraction satisfying every expected diagnostic fragment.
    pub diagnostic_accuracy: f64,
    /// Fraction of successful returned-procedure scenarios with valid evidence.
    pub evidence_accuracy: f64,
    /// Largest number of returned procedures.
    pub max_returned_per_scenario: usize,
    /// Total explicitly forbidden returned procedures.
    pub forbidden_result_count: usize,
    /// Median matcher latency.
    pub query_p50_ms: f64,
    /// 95th percentile matcher latency.
    pub query_p95_ms: f64,
}

/// Reproducible provider-free real-matcher report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcedureProbeReport {
    /// Probe identifier.
    pub probe_id: String,
    /// SHA-256 of exact protocol bytes.
    pub protocol_sha256: String,
    /// Baseline protocol inherited by a schema-v2 replay.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline_protocol_sha256: Option<String>,
    /// Frozen correction policy for a schema-v2 replay.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correction_policy: Option<String>,
    /// SHA-256 of the motivating v5 results.
    pub trigger_results_sha256: String,
    /// Observed source-file hashes.
    pub observed_source_files: BTreeMap<String, String>,
    /// Frozen fixture revision.
    pub fixture_revision: String,
    /// Number of complete matrix repetitions.
    pub repetitions: usize,
    /// Total scenario executions across repetitions.
    pub run_count: usize,
    /// Number of memory records in the isolated store.
    pub corpus_record_count: usize,
    /// Number of active verified procedures after the extra seed.
    pub active_verified_procedure_count: usize,
    /// Frozen gates.
    pub quality_gates: ProcedureQualityGates,
    /// Aggregate metrics.
    pub metrics: ProcedureProbeMetrics,
    /// Scenario-level evidence.
    pub runs: Vec<ProcedureScenarioRun>,
    /// Frozen gate violations.
    pub gate_violations: Vec<String>,
    /// True only when every frozen gate passes.
    pub passed: bool,
    /// Claims this probe cannot support.
    pub limitations: Vec<String>,
}

/// Run the frozen provider-free baseline against the real procedure matcher.
pub async fn run_procedure_match_probe(
    protocol_path: &Path,
    output: &Path,
) -> EvalResult<ProcedureProbeReport> {
    let protocol_bytes = fs::read(protocol_path)?;
    let protocol_sha256 = sha256_bytes(&protocol_bytes);
    let owner = protocol_path.parent().unwrap_or_else(|| Path::new("."));
    let (protocol, baseline_protocol_sha256, correction_policy, repetitions) =
        load_protocol(owner, &protocol_bytes)?;

    let trigger_path = owner.join(&protocol.trigger_results.path);
    let trigger_bytes = fs::read(&trigger_path)?;
    let trigger_sha256 = sha256_bytes(&trigger_bytes);
    require_hash(
        "trigger results",
        &trigger_sha256,
        &protocol.trigger_results.sha256,
    )?;
    let mut observed_source_files = BTreeMap::new();
    for source in &protocol.source_files {
        let bytes = fs::read(owner.join(&source.path))?;
        let observed = sha256_bytes(&bytes);
        require_hash(&source.path, &observed, &source.sha256)?;
        observed_source_files.insert(source.path.clone(), observed);
    }

    require_empty_target(output)?;
    let fixture_root = output.join("fixture");
    let data_dir = output.join("data");
    let layout = materialize_engineering_context_fixture(&fixture_root)?;
    let (mut seed, db) =
        open_seeded_engineering_context_fixture(&fixture_root.join("fixture-map.json"), &data_dir)
            .await?;
    let service = MemoryService::new(db.clone());
    service.init_schema().await.map_err(invalid)?;
    let repo = MemoryRepo::new(db);
    repo.init_schema().await.map_err(invalid)?;

    let additional =
        seed_additional_procedure(&service, &layout, &protocol.additional_procedure, output)
            .await?;
    seed.memory.insert(
        protocol.additional_procedure.key.clone(),
        additional.id.to_string(),
    );
    seed.verified_procedures
        .push(protocol.additional_procedure.key.clone());
    let key_by_id = seed
        .memory
        .iter()
        .map(|(key, id)| (id.clone(), key.clone()))
        .collect::<BTreeMap<_, _>>();

    let integration_id = seed
        .memory
        .get(INTEGRATION_KEY)
        .ok_or_else(|| EvalError::Invalid("missing integration v3 seed key".to_string()))?;
    let integration_id = Id::parse(integration_id).map_err(invalid)?;
    let baseline_integration = repo
        .get_memory_item(&integration_id)
        .await
        .map_err(invalid)?
        .ok_or_else(|| EvalError::Invalid("missing integration v3 memory item".to_string()))?;
    let integration_receipt = resolve_fixture_uri(&layout, INTEGRATION_RECEIPT_URI)?;
    let baseline_receipt_bytes = fs::read(&integration_receipt)?;

    let mut runs = Vec::new();
    for repetition in 1..=repetitions {
        for scenario in &protocol.scenarios {
            restore_baseline(
                &repo,
                &baseline_integration,
                &integration_receipt,
                &baseline_receipt_bytes,
            )
            .await?;
            apply_setup(
                &repo,
                &baseline_integration,
                &integration_receipt,
                &scenario.setup,
            )
            .await?;
            let cwd = scenario
                .cwd
                .as_deref()
                .map(|uri| resolve_fixture_uri(&layout, uri))
                .transpose()?;
            let started = Instant::now();
            let matched = service
                .match_procedures(ProcedureMatchInput {
                    query: scenario.query.clone(),
                    project: scenario.project.clone(),
                    cwd: cwd.as_ref().map(|path| path.display().to_string()),
                    conditions: scenario.conditions.clone(),
                    limit: Some(protocol.limit),
                })
                .await;
            let latency_ms = started.elapsed().as_secs_f64() * 1_000.0;
            let restore = restore_baseline(
                &repo,
                &baseline_integration,
                &integration_receipt,
                &baseline_receipt_bytes,
            )
            .await;
            let matched = matched.map_err(invalid)?;
            restore?;
            runs.push(score_run(
                scenario,
                repetition,
                &matched,
                &key_by_id,
                &layout,
                cwd.as_deref(),
                latency_ms,
            )?);
        }
    }

    let metrics = aggregate_metrics(&protocol.scenarios, &runs);
    let mut gate_violations = gate_violations(&protocol.quality_gates, &metrics);
    if correction_policy.as_deref()
        == Some("attest_current_procedure_boundary_and_bounded_no_result_actions")
    {
        let no_result_runs = runs
            .iter()
            .filter(|run| run.category == "no_result")
            .collect::<Vec<_>>();
        let bounded_no_result = no_result_runs.len() == repetitions
            && no_result_runs.into_iter().all(|run| {
                run.abstained
                    && run.next_actions.len() == 1
                    && [
                        "current_checkout_root",
                        "bounded read-only local evidence-closure",
                        "canonical repository identity",
                        "Git-tracked project or component",
                        "operation-specific runbook",
                        "Do not broaden beyond the current checkout",
                        "execute candidate commands",
                        "invent a project",
                    ]
                    .into_iter()
                    .all(|fragment| run.next_actions[0].contains(fragment))
            });
        if !bounded_no_result {
            gate_violations.push("bounded_no_result_next_actions".to_string());
        }
    }
    let all_items = service.list_memory(None, None).await.map_err(invalid)?;
    let active_verified_procedure_count = all_items
        .iter()
        .filter(|item| item.status == MemoryStatus::Active)
        .filter_map(|item| item.procedure.as_ref())
        .filter(|procedure| procedure.is_verified_at(OffsetDateTime::now_utc()))
        .count();
    let report = ProcedureProbeReport {
        probe_id: protocol.probe_id,
        protocol_sha256,
        baseline_protocol_sha256,
        correction_policy,
        trigger_results_sha256: trigger_sha256,
        observed_source_files,
        fixture_revision: seed.fixture_revision,
        repetitions,
        run_count: runs.len(),
        corpus_record_count: all_items.len(),
        active_verified_procedure_count,
        quality_gates: protocol.quality_gates,
        metrics,
        runs,
        passed: gate_violations.is_empty(),
        gate_violations,
        limitations: vec![
            "This provider-free probe calls the real MemoryService matcher but does not exercise a Codex or Claude host boundary.".to_string(),
            "The corpus is synthetic and small; exact accuracy here is necessary but not sufficient for production generalization.".to_string(),
            "Scenario mutations occur only in the isolated evaluation store and are restored after each query.".to_string(),
            "Latency is local matcher, repository-resolution, database, and receipt-I/O time, not a host-native end-to-end SLO.".to_string(),
            "The probe verifies returned receipt hashes but does not execute stored commands.".to_string(),
        ],
    };
    write_private_json(&output.join("procedure-report.json"), &report)?;
    Ok(report)
}

fn load_protocol(
    owner: &Path,
    bytes: &[u8],
) -> EvalResult<(
    ProcedureProbeProtocol,
    Option<String>,
    Option<String>,
    usize,
)> {
    load_protocol_with_depth(owner, bytes, 0)
}

fn load_protocol_with_depth(
    owner: &Path,
    bytes: &[u8],
    depth: usize,
) -> EvalResult<(
    ProcedureProbeProtocol,
    Option<String>,
    Option<String>,
    usize,
)> {
    if depth > 4 {
        return Err(EvalError::Invalid(
            "procedure replay chain exceeds four corrections".to_string(),
        ));
    }
    let value: serde_json::Value = serde_json::from_slice(bytes)?;
    let schema_version = value
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| EvalError::Invalid("procedure protocol omits schema_version".to_string()))?;
    if schema_version == u64::from(PROCEDURE_PROBE_SCHEMA_VERSION) {
        let protocol: ProcedureProbeProtocol = serde_json::from_value(value)?;
        validate_protocol(&protocol)?;
        return Ok((protocol, None, None, 1));
    }
    if schema_version != u64::from(PROCEDURE_REPLAY_SCHEMA_VERSION) {
        return Err(EvalError::Invalid(format!(
            "unsupported procedure probe schema version: {schema_version}"
        )));
    }
    let replay: ProcedureReplayProtocol = serde_json::from_value(value)?;
    let supported_policy = matches!(
        replay.correction_policy.as_str(),
        "fail_closed_on_material_project_repository_ambiguity"
            | "include_procedure_prerequisites_in_text_matching"
            | "attest_current_procedure_boundary_and_bounded_no_result_actions"
    );
    let source_override_paths = replay
        .source_overrides
        .iter()
        .map(|source| source.path.as_str())
        .collect::<BTreeSet<_>>();
    let source_overrides_valid = if replay.correction_policy
        == "attest_current_procedure_boundary_and_bounded_no_result_actions"
    {
        source_override_paths
            == BTreeSet::from([
                "../../engram-index/src/memory.rs",
                "../../engram-core/src/memory.rs",
                "../../engram-eval/src/fixture.rs",
            ])
    } else {
        replay.source_overrides.len() == 1
    };
    if replay.probe_id.trim().is_empty()
        || !supported_policy
        || replay.baseline_protocol.sha256.len() != 64
        || replay.trigger_results.sha256.len() != 64
        || !(1..=20).contains(&replay.repetitions)
        || !source_overrides_valid
    {
        return Err(EvalError::Invalid(
            "invalid procedure replay header or correction policy".to_string(),
        ));
    }
    let baseline_path = owner.join(&replay.baseline_protocol.path);
    let baseline_bytes = fs::read(&baseline_path)?;
    let baseline_sha256 = sha256_bytes(&baseline_bytes);
    require_hash(
        "baseline procedure protocol",
        &baseline_sha256,
        &replay.baseline_protocol.sha256,
    )?;
    let baseline_owner = baseline_path.parent().unwrap_or_else(|| Path::new("."));
    let owner_parent = fs::canonicalize(owner)?.parent().map(Path::to_path_buf);
    let baseline_owner_parent = fs::canonicalize(baseline_owner)?
        .parent()
        .map(Path::to_path_buf);
    if owner_parent != baseline_owner_parent {
        return Err(EvalError::Invalid(
            "procedure replay and baseline must be sibling protocol directories".to_string(),
        ));
    }
    let (mut protocol, _, _, _) =
        load_protocol_with_depth(baseline_owner, &baseline_bytes, depth + 1)?;
    for source_override in &replay.source_overrides {
        let source = protocol
            .source_files
            .iter_mut()
            .find(|source| source.path == source_override.path)
            .ok_or_else(|| {
                EvalError::Invalid(format!(
                    "source override is absent from baseline: {}",
                    source_override.path
                ))
            })?;
        if source.sha256 == source_override.sha256 {
            return Err(EvalError::Invalid(format!(
                "source override does not change the baseline hash: {}",
                source_override.path
            )));
        }
        *source = source_override.clone();
    }
    protocol.probe_id = replay.probe_id;
    protocol.trigger_results = replay.trigger_results;
    Ok((
        protocol,
        Some(baseline_sha256),
        Some(replay.correction_policy),
        replay.repetitions,
    ))
}

fn validate_protocol(protocol: &ProcedureProbeProtocol) -> EvalResult<()> {
    if protocol.schema_version != PROCEDURE_PROBE_SCHEMA_VERSION
        || protocol.probe_id.trim().is_empty()
        || protocol.limit == 0
        || protocol.scenarios.is_empty()
        || protocol.source_files.is_empty()
    {
        return Err(EvalError::Invalid(
            "invalid procedure probe header".to_string(),
        ));
    }
    for value in [
        protocol.quality_gates.min_exact_outcome_accuracy,
        protocol.quality_gates.min_success_accuracy,
        protocol.quality_gates.min_abstention_accuracy,
        protocol.quality_gates.min_diagnostic_accuracy,
        protocol.quality_gates.min_evidence_accuracy,
    ] {
        if !(0.0..=1.0).contains(&value) {
            return Err(EvalError::Invalid(
                "procedure accuracy gates must be in [0, 1]".to_string(),
            ));
        }
    }
    if protocol.quality_gates.max_returned_per_scenario == 0
        || protocol.trigger_results.sha256.len() != 64
        || protocol
            .source_files
            .iter()
            .any(|source| source.path.trim().is_empty() || source.sha256.len() != 64)
    {
        return Err(EvalError::Invalid(
            "invalid procedure source hashes or budgets".to_string(),
        ));
    }
    let success_count = protocol
        .scenarios
        .iter()
        .filter(|scenario| !scenario.expect_abstain)
        .count();
    let abstention_count = protocol.scenarios.len() - success_count;
    if success_count < 5 || abstention_count < 10 {
        return Err(EvalError::Invalid(
            "procedure probe requires at least five success and ten abstention scenarios"
                .to_string(),
        ));
    }
    let allowed_setups = BTreeSet::from([
        "none",
        "remove_integration_receipt",
        "tamper_integration_receipt",
        "expire_integration_verification",
        "clear_integration_verification_hash",
        "supersede_integration_procedure",
    ]);
    let mut ids = BTreeSet::new();
    for scenario in &protocol.scenarios {
        if scenario.id.trim().is_empty()
            || scenario.query.trim().is_empty()
            || !ids.insert(scenario.id.as_str())
            || !allowed_setups.contains(scenario.setup.as_str())
            || scenario.expect_abstain != scenario.expected_keys.is_empty()
        {
            return Err(EvalError::Invalid(format!(
                "invalid procedure scenario: {}",
                scenario.id
            )));
        }
    }
    let additional = &protocol.additional_procedure;
    if additional.key.trim().is_empty()
        || additional.commands.is_empty()
        || additional.failure_signatures.is_empty()
        || additional.expires_in_days <= 0
        || additional.verification_output.trim().is_empty()
    {
        return Err(EvalError::Invalid(
            "invalid additional failure-signature procedure".to_string(),
        ));
    }
    Ok(())
}

async fn seed_additional_procedure(
    service: &MemoryService,
    layout: &FixtureLayout,
    procedure: &AdditionalProcedure,
    output: &Path,
) -> EvalResult<MemoryItem> {
    let receipt_path = output.join("context-probe-success.json");
    let receipt = ProcedureVerificationReceipt {
        command: procedure.verification_command.clone(),
        exit_code: procedure.verification_exit_code,
        output: procedure.verification_output.clone(),
        conditions: procedure.receipt_conditions.clone(),
    };
    write_private_json(&receipt_path, &receipt)?;
    let writer = WriterProvenance::agent(
        Harness::Other("engram_eval".to_string()),
        ModelIdentity::new("deterministic", "procedure-probe-seeder"),
    )
    .with_surface("engram-eval");
    let mut card = ProcedureCard::new(
        procedure.task.clone(),
        procedure.commands.clone(),
        ProcedureVerification::new(
            procedure.verification_command.clone(),
            procedure.verification_exit_code,
            procedure.verification_output.clone(),
        ),
    );
    for (key, expected) in &procedure.prerequisites {
        card = card.with_prerequisite(ProcedurePrerequisite::new(key, expected));
    }
    for signature in &procedure.failure_signatures {
        card = card.with_failure_signature(signature);
    }
    let candidate = service
        .capture_memory(
            MemoryItem::new(
                MemoryKind::Procedure,
                procedure.title.clone(),
                procedure.content.clone(),
                MemoryScope::project(procedure.project.clone()),
                ClaimOrigin::AgentObserved,
                writer,
            )
            .with_procedure(card),
        )
        .await
        .map_err(invalid)?;
    let verified = service
        .verify_procedure(
            &candidate.id,
            &receipt_path,
            Some(OffsetDateTime::now_utc() + Duration::days(procedure.expires_in_days)),
        )
        .await
        .map_err(invalid)?;
    if resolve_checkout_root(layout, "fixture://atlas/main/services/worker").is_none() {
        return Err(EvalError::Invalid(
            "fixture omits Atlas checkout root".to_string(),
        ));
    }
    Ok(verified)
}

async fn restore_baseline(
    repo: &MemoryRepo,
    baseline: &MemoryItem,
    receipt_path: &Path,
    receipt_bytes: &[u8],
) -> EvalResult<()> {
    repo.save_memory_item(baseline).await.map_err(invalid)?;
    fs::write(receipt_path, receipt_bytes)?;
    Ok(())
}

async fn apply_setup(
    repo: &MemoryRepo,
    baseline: &MemoryItem,
    receipt_path: &Path,
    setup: &str,
) -> EvalResult<()> {
    match setup {
        "none" => {}
        "remove_integration_receipt" => fs::remove_file(receipt_path)?,
        "tamper_integration_receipt" => fs::write(receipt_path, b"tampered receipt")?,
        "expire_integration_verification" => {
            let mut item = baseline.clone();
            item.procedure
                .as_mut()
                .ok_or_else(|| {
                    EvalError::Invalid("integration procedure card missing".to_string())
                })?
                .expires_at = Some(OffsetDateTime::now_utc() - Duration::days(1));
            repo.save_memory_item(&item).await.map_err(invalid)?;
        }
        "clear_integration_verification_hash" => {
            let mut item = baseline.clone();
            item.procedure
                .as_mut()
                .ok_or_else(|| {
                    EvalError::Invalid("integration procedure card missing".to_string())
                })?
                .verification
                .evidence_sha256 = None;
            repo.save_memory_item(&item).await.map_err(invalid)?;
        }
        "supersede_integration_procedure" => {
            let mut item = baseline.clone();
            item.status = MemoryStatus::Superseded;
            repo.save_memory_item(&item).await.map_err(invalid)?;
        }
        other => {
            return Err(EvalError::Invalid(format!(
                "unsupported procedure scenario setup: {other}"
            )))
        }
    }
    Ok(())
}

fn score_run(
    scenario: &ProcedureScenario,
    repetition: usize,
    matched: &engram_index::ProcedureMatchReport,
    key_by_id: &BTreeMap<String, String>,
    layout: &FixtureLayout,
    cwd: Option<&Path>,
    latency_ms: f64,
) -> EvalResult<ProcedureScenarioRun> {
    let returned_keys = matched
        .procedures
        .iter()
        .map(|item| semantic_key(item, key_by_id))
        .collect::<Vec<_>>();
    let forbidden = scenario.forbidden_keys.iter().collect::<BTreeSet<_>>();
    let forbidden_keys = returned_keys
        .iter()
        .filter(|key| forbidden.contains(key))
        .cloned()
        .collect::<Vec<_>>();
    let returned_set = returned_keys.iter().collect::<BTreeSet<_>>();
    let expected_set = scenario.expected_keys.iter().collect::<BTreeSet<_>>();
    let exact_outcome = returned_set == expected_set
        && matched.abstained == scenario.expect_abstain
        && forbidden_keys.is_empty();
    let diagnostic_text = matched
        .diagnostics
        .iter()
        .flat_map(|diagnostic| &diagnostic.reasons)
        .map(|reason| reason.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join("\n");
    let diagnostics_accurate = scenario
        .expected_diagnostic_substrings
        .iter()
        .all(|expected| diagnostic_text.contains(&expected.to_ascii_lowercase()));
    let checkout_root = scenario
        .cwd
        .as_deref()
        .and_then(|uri| resolve_checkout_root(layout, uri))
        .or_else(|| cwd.and_then(git_checkout_root));
    let evidence_accurate = matched
        .procedures
        .iter()
        .all(|item| procedure_evidence_matches(item, checkout_root.as_deref()));
    let diagnostics = matched
        .diagnostics
        .iter()
        .map(|diagnostic| ProcedureDiagnosticEvidence {
            key: key_by_id
                .get(&diagnostic.memory_id.to_string())
                .cloned()
                .unwrap_or_else(|| format!("unmapped:{}", diagnostic.memory_id)),
            applicable: diagnostic.applicable,
            reasons: diagnostic.reasons.clone(),
        })
        .collect();
    Ok(ProcedureScenarioRun {
        scenario_id: scenario.id.clone(),
        repetition,
        category: scenario.category.clone(),
        setup: scenario.setup.clone(),
        cwd: cwd.map(|path| path.display().to_string()),
        returned_keys,
        forbidden_keys,
        abstained: matched.abstained,
        exact_outcome,
        diagnostics_accurate,
        evidence_accurate,
        diagnostics,
        latency_ms,
        message: matched.message.clone(),
        next_actions: matched.next_actions.clone(),
    })
}

fn aggregate_metrics(
    scenarios: &[ProcedureScenario],
    runs: &[ProcedureScenarioRun],
) -> ProcedureProbeMetrics {
    let success_ids = scenarios
        .iter()
        .filter(|scenario| !scenario.expect_abstain)
        .map(|scenario| scenario.id.as_str())
        .collect::<BTreeSet<_>>();
    let abstention_ids = scenarios
        .iter()
        .filter(|scenario| scenario.expect_abstain)
        .map(|scenario| scenario.id.as_str())
        .collect::<BTreeSet<_>>();
    let exact_count = runs.iter().filter(|run| run.exact_outcome).count();
    let success_run_count = runs
        .iter()
        .filter(|run| success_ids.contains(run.scenario_id.as_str()))
        .count();
    let abstention_run_count = runs
        .iter()
        .filter(|run| abstention_ids.contains(run.scenario_id.as_str()))
        .count();
    let success_correct = runs
        .iter()
        .filter(|run| success_ids.contains(run.scenario_id.as_str()) && run.exact_outcome)
        .count();
    let abstention_correct = runs
        .iter()
        .filter(|run| abstention_ids.contains(run.scenario_id.as_str()) && run.exact_outcome)
        .count();
    let diagnostics_correct = runs.iter().filter(|run| run.diagnostics_accurate).count();
    let returned_successes = runs
        .iter()
        .filter(|run| !run.returned_keys.is_empty())
        .collect::<Vec<_>>();
    let evidence_correct = returned_successes
        .iter()
        .filter(|run| run.evidence_accurate)
        .count();
    let mut latencies = runs.iter().map(|run| run.latency_ms).collect::<Vec<_>>();
    latencies.sort_by(f64::total_cmp);
    ProcedureProbeMetrics {
        scenario_count: scenarios.len(),
        exact_outcome_accuracy: fraction(exact_count, runs.len()),
        success_accuracy: fraction(success_correct, success_run_count),
        abstention_accuracy: fraction(abstention_correct, abstention_run_count),
        diagnostic_accuracy: fraction(diagnostics_correct, runs.len()),
        evidence_accuracy: fraction(evidence_correct, returned_successes.len()),
        max_returned_per_scenario: runs
            .iter()
            .map(|run| run.returned_keys.len())
            .max()
            .unwrap_or_default(),
        forbidden_result_count: runs.iter().map(|run| run.forbidden_keys.len()).sum(),
        query_p50_ms: percentile(&latencies, 0.50),
        query_p95_ms: percentile(&latencies, 0.95),
    }
}

fn gate_violations(gates: &ProcedureQualityGates, metrics: &ProcedureProbeMetrics) -> Vec<String> {
    let mut violations = Vec::new();
    for (name, observed, expected) in [
        (
            "exact_outcome_accuracy",
            metrics.exact_outcome_accuracy,
            gates.min_exact_outcome_accuracy,
        ),
        (
            "success_accuracy",
            metrics.success_accuracy,
            gates.min_success_accuracy,
        ),
        (
            "abstention_accuracy",
            metrics.abstention_accuracy,
            gates.min_abstention_accuracy,
        ),
        (
            "diagnostic_accuracy",
            metrics.diagnostic_accuracy,
            gates.min_diagnostic_accuracy,
        ),
        (
            "evidence_accuracy",
            metrics.evidence_accuracy,
            gates.min_evidence_accuracy,
        ),
    ] {
        if observed < expected {
            violations.push(format!("{name}:{observed:.4}<{expected}"));
        }
    }
    if metrics.max_returned_per_scenario > gates.max_returned_per_scenario {
        violations.push(format!(
            "max_returned_per_scenario:{}>{}",
            metrics.max_returned_per_scenario, gates.max_returned_per_scenario
        ));
    }
    if metrics.forbidden_result_count > gates.max_forbidden_results {
        violations.push(format!(
            "forbidden_results:{}>{}",
            metrics.forbidden_result_count, gates.max_forbidden_results
        ));
    }
    if metrics.query_p95_ms > gates.query_p95_ms as f64 {
        violations.push(format!(
            "query_p95_ms:{:.4}>{}",
            metrics.query_p95_ms, gates.query_p95_ms
        ));
    }
    violations
}

fn procedure_evidence_matches(item: &MemoryItem, checkout_root: Option<&Path>) -> bool {
    let Some(procedure) = &item.procedure else {
        return false;
    };
    if !procedure.is_verified_at(OffsetDateTime::now_utc()) {
        return false;
    }
    let (Some(path), Some(expected)) = (
        procedure.verification.evidence_path.as_deref(),
        procedure.verification.evidence_sha256.as_deref(),
    ) else {
        return false;
    };
    let path = PathBuf::from(path);
    let resolved = if path.is_absolute() {
        path
    } else if let Some(root) = checkout_root {
        root.join(path)
    } else {
        return false;
    };
    fs::read(resolved)
        .map(|bytes| sha256_bytes(&bytes) == expected)
        .unwrap_or(false)
}

fn semantic_key(item: &MemoryItem, key_by_id: &BTreeMap<String, String>) -> String {
    key_by_id
        .get(&item.id.to_string())
        .cloned()
        .unwrap_or_else(|| format!("unmapped:{}", item.id))
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
    Ok(PathBuf::from(root).join(suffix))
}

fn resolve_checkout_root(layout: &FixtureLayout, uri: &str) -> Option<PathBuf> {
    layout
        .checkouts
        .iter()
        .filter(|(prefix, _)| uri == prefix.as_str() || uri.starts_with(&format!("{prefix}/")))
        .max_by_key(|(prefix, _)| prefix.len())
        .map(|(_, root)| PathBuf::from(root))
}

fn git_checkout_root(path: &Path) -> Option<PathBuf> {
    path.ancestors()
        .find(|candidate| candidate.join(".git").exists())
        .map(Path::to_path_buf)
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
    fn exact_outcome_requires_expected_set_abstention_and_no_forbidden_results() {
        let scenario = scenario(false);
        let run = ProcedureScenarioRun {
            scenario_id: scenario.id.clone(),
            repetition: 1,
            category: scenario.category.clone(),
            setup: "none".to_string(),
            cwd: None,
            returned_keys: vec!["expected".to_string(), "forbidden".to_string()],
            forbidden_keys: vec!["forbidden".to_string()],
            abstained: false,
            exact_outcome: false,
            diagnostics_accurate: true,
            evidence_accurate: true,
            diagnostics: Vec::new(),
            latency_ms: 1.0,
            message: "matched".to_string(),
            next_actions: Vec::new(),
        };
        let metrics = aggregate_metrics(&[scenario], &[run]);
        assert_eq!(metrics.exact_outcome_accuracy, 0.0);
        assert_eq!(metrics.forbidden_result_count, 1);
    }

    #[test]
    fn gate_checks_success_and_abstention_as_separate_populations() {
        let scenarios = vec![scenario(false), scenario(true)];
        let runs = vec![
            run("scenario-success", true),
            run("scenario-abstain", false),
        ];
        let metrics = aggregate_metrics(&scenarios, &runs);
        assert_eq!(metrics.success_accuracy, 1.0);
        assert_eq!(metrics.abstention_accuracy, 0.0);
    }

    #[test]
    fn aggregate_accuracy_uses_repeated_run_denominators() {
        let scenarios = vec![scenario(false), scenario(true)];
        let runs = vec![
            run("scenario-success", true),
            run("scenario-abstain", true),
            run("scenario-success", true),
            run("scenario-abstain", false),
        ];
        let metrics = aggregate_metrics(&scenarios, &runs);
        assert_eq!(metrics.exact_outcome_accuracy, 0.75);
        assert_eq!(metrics.success_accuracy, 1.0);
        assert_eq!(metrics.abstention_accuracy, 0.5);
    }

    fn scenario(abstain: bool) -> ProcedureScenario {
        ProcedureScenario {
            id: if abstain {
                "scenario-abstain".to_string()
            } else {
                "scenario-success".to_string()
            },
            category: "test".to_string(),
            query: "query".to_string(),
            project: Some("atlas".to_string()),
            cwd: None,
            conditions: BTreeMap::new(),
            setup: "none".to_string(),
            expected_keys: if abstain {
                Vec::new()
            } else {
                vec!["expected".to_string()]
            },
            forbidden_keys: vec!["forbidden".to_string()],
            expect_abstain: abstain,
            expected_diagnostic_substrings: Vec::new(),
        }
    }

    fn run(id: &str, exact: bool) -> ProcedureScenarioRun {
        ProcedureScenarioRun {
            scenario_id: id.to_string(),
            repetition: 1,
            category: "test".to_string(),
            setup: "none".to_string(),
            cwd: None,
            returned_keys: Vec::new(),
            forbidden_keys: Vec::new(),
            abstained: true,
            exact_outcome: exact,
            diagnostics_accurate: true,
            evidence_accurate: true,
            diagnostics: Vec::new(),
            latency_ms: 1.0,
            message: "message".to_string(),
            next_actions: Vec::new(),
        }
    }
}
