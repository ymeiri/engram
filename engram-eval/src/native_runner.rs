//! Approval-gated execution for the learned native-memory pilot.

use crate::native_audit::{
    audit_native_memory_pilot, validate_teaching_command_evidence, AuditStatus, NativeLanePhase,
    NativePilotAudit,
};
use crate::native_document::{
    load_historical_native_plan, load_historical_native_plan_with_identity,
    read_bounded_owned_text_file, LoadedHistoricalNativePlan,
};
use crate::native_execution::{
    NativeExecutionChildGuard, NativeExecutionCleanupOutcome, NativeExecutionSpawnOutcome,
};
use crate::native_instructions_control::{
    complete_codex_rollout_forbidden_scan, complete_forbidden_trace_scan, load_control_plan,
    open_control_claude_phase_artifacts, seal_control_effective_config, seal_control_lane_terminal,
    seal_control_run_report, treatment_admission_ordinal,
    treatment_dynamic_forbidden_trace_needles, validate_control_lane_configured_absence,
    validate_prepared_control, validated_control_bundle_pre_provider_receipt_evidence,
    validated_control_bundle_predecessor_evidence, NativeInstructionsControlClaudeArtifactGuard,
    NativeInstructionsControlExecution, NativeInstructionsControlRunReport,
    NativeProviderAdmissionOutcome, NativeProviderBundleAdmission, NativeProviderBundleJournal,
    NativeProviderBundlePreProviderReceiptEvidence, NativeProviderBundlePredecessorEvidence,
    PreparedInstructionsControlLane,
};
use crate::native_pilot::{
    attest_engram_mcp_contract, claude_allowed_tools,
    validate_native_pilot_family_v3_runtime_libraries, CodexAuthenticationMode,
    EngramMcpContractAttestation, NativePilotPrerequisiteSource,
    NativePilotRuntimeLibraryAttestation, PreparedCodexAuthentication,
    PreparedNativeEvaluationRecovery, PreparedNativeExecutionRecovery, PreparedNativeInheritedFile,
    PreparedNativeLane, PreparedNativePilot, CODEX_CHATGPT_LOGIN_STATUS, CODEX_FILE_CONFIG,
    CODEX_KEYRING_CONFIG, OPERATION_EVIDENCE_ROUTE_NATIVE_PILOT_SCHEMA_VERSION,
    STRUCTURED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION,
};
#[cfg(test)]
use crate::native_pilot::{
    EngramMcpRuntimeAttestation, EXPIRED_PROCEDURE_NATIVE_PILOT_SCHEMA_VERSION,
};
use crate::native_stale::{
    materialize_native_stale_evaluation_marker_snapshot, trace_native_stale_family_version,
    validate_native_stale_evaluation_marker_snapshot, validate_native_stale_plan_binding,
    validate_native_stale_v3_claude_config_artifacts,
    validate_native_stale_v3_codex_action_lifecycle, NativeStalePreparationPrecondition,
    NativeStaleV3ClaudeConfigAttestation,
};
use crate::pilot::{
    attest_binary, attest_codex_code_mode_host, create_private_dir_all, format_budget_usd,
    require_empty_target, PilotBinaryAttestation, AGENT_OUTPUT_SCHEMA,
};
use crate::{normalize_remote, EvalError, EvalResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use zeroize::Zeroizing;

const PROCEDURE_ID_PLACEHOLDER: &str = "${PROCEDURE_MEMORY_ID_FROM_TEACHING_TRACE}";
const CLAUDE_SCHEMA_REJECTION: &str = "Error: --json-schema is not a valid JSON Schema: no schema with key or ref \"https://json-schema.org/draft/2020-12/schema\"";
const CODEX_TERMINAL_OUTPUT_REJECTION: &str =
    "Codex emitted multiple schema-shaped agent messages before its terminal final response";
const CODEX_FINAL_RESPONSE_HELP: &str =
    "Path to a JSON Schema file describing the model's final response shape";
const CODEX_PRE_CONSOLIDATION_AUDIT_REJECTION: &str =
    "Codex pre-consolidation memory scaffold omitted MEMORY.md and memory_summary.md";
const PROCEDURE_TEACHING_SCOPE_REJECTION: &str =
    "Trusted procedure selector compared the teaching candidate against the evaluation repository";
const CLAUDE_TEACHING_COMMAND_WRAPPER_REJECTION: &str =
    "Claude teaching command used a repository-root wrapper with an explicit inner exit report";
const MAX_CODEX_AUTH_FILE_BYTES: u64 = 1024 * 1024;
const CODEX_AUTH_CACHE_LIFECYCLE_FAMILY_VERSION: u32 = 3;
const CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION: u32 = 1;
const CODEX_AUTH_CACHE_LIFECYCLE_DIRECTORY: &str = "codex-auth-cache-lifecycle-v1";
const CODEX_AUTH_CACHE_LIFECYCLE_LOCK: &str = "lifecycle.lock";
const CODEX_AUTH_CACHE_CLEANUP_AUTHORITY: &str =
    "explicit_exact_six_plaintext_cache_cleanup_confirmation";
const CODEX_AUTH_STATUS_TIMEOUT: Duration = Duration::from_secs(15);
const CODEX_AUTH_STATUS_CLEANUP_RESERVE: Duration = Duration::from_secs(5);
const MAX_CODEX_AUTH_STATUS_OUTPUT_BYTES: usize = 8 * 1024;
const MAX_CODEX_SESSION_ROLLOUT_BYTES: u64 = 32 * 1024 * 1024;
const MAX_CODEX_SESSION_SCAN_ENTRIES: usize = 4_096;
const MAX_CODEX_SESSION_SCAN_DEPTH: usize = 8;
const PROVIDER_WALL_TIMEOUT: Duration = Duration::from_millis(900_000);
const PROVIDER_CLEANUP_GRACE: Duration = Duration::from_millis(10_000);
const PROVIDER_STDOUT_LIMIT_BYTES: u64 = 67_108_864;
const PROVIDER_STDERR_LIMIT_BYTES: u64 = 8_388_608;
const CONTROL_AGENT_OUTPUT_LIMIT_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EvaluationRecoveryKind {
    ClaudeSchemaRejection,
    CodexTerminalOutput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExecutionRecoveryKind {
    CodexPreConsolidationAudit,
    ProcedureTeachingScope,
    ClaudeTeachingCommandWrapper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeAgentOutputContract {
    Base,
    StaleSafetyV1,
    StaleSafetyV3,
}

impl ExecutionRecoveryKind {
    fn failure_reason(self) -> &'static str {
        match self {
            Self::CodexPreConsolidationAudit => CODEX_PRE_CONSOLIDATION_AUDIT_REJECTION,
            Self::ProcedureTeachingScope => PROCEDURE_TEACHING_SCOPE_REJECTION,
            Self::ClaudeTeachingCommandWrapper => CLAUDE_TEACHING_COMMAND_WRAPPER_REJECTION,
        }
    }
}

fn execution_recovery_kind(reason: &str) -> EvalResult<ExecutionRecoveryKind> {
    match reason {
        CODEX_PRE_CONSOLIDATION_AUDIT_REJECTION => {
            Ok(ExecutionRecoveryKind::CodexPreConsolidationAudit)
        }
        PROCEDURE_TEACHING_SCOPE_REJECTION => Ok(ExecutionRecoveryKind::ProcedureTeachingScope),
        CLAUDE_TEACHING_COMMAND_WRAPPER_REJECTION => {
            Ok(ExecutionRecoveryKind::ClaudeTeachingCommandWrapper)
        }
        other => Err(EvalError::Invalid(format!(
            "unsupported execution recovery reason: {other}"
        ))),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InterruptedTeachingRecovery {
    kind: ExecutionRecoveryKind,
    completed_lane_orders: Vec<u32>,
}

/// One separately authorized provider-execution phase.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NativePilotRunPhase {
    /// Teach every preregistered lane and verify Engram candidates.
    Teaching,
    /// Run the matched Codex calls after the frozen idle interval.
    Activation,
    /// Run fresh-session evaluation after every artifact gate passes.
    Evaluation,
}

impl NativePilotRunPhase {
    fn label(self) -> &'static str {
        match self {
            Self::Teaching => "teaching",
            Self::Activation => "activation",
            Self::Evaluation => "evaluation",
        }
    }
}

/// Explicit execution authorization supplied for one runner invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativePilotExecutionApproval {
    /// Must be true for any host process to run.
    pub provider_execution: bool,
    /// Must exactly match the aggregate Claude budget frozen in the plan.
    pub claude_budget_cents: Option<u32>,
}

/// One completed host invocation in a phase report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NativePilotLaneExecution {
    /// Fixed lane order.
    pub order: u32,
    /// Arm identifier.
    pub arm: String,
    /// Host harness.
    pub host: String,
    /// Raw provider trace path.
    pub trace_path: String,
    /// Provider stderr path.
    pub stderr_path: String,
    /// SHA-256 of the exact argv from the frozen plan.
    pub argv_sha256: String,
    /// SHA-256 of the exact closed-world environment for family-v3 provider dispatches.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_environment_sha256: Option<String>,
    /// SHA-256 of the aggregate Claude configuration root/file identity attestation containing
    /// content hashes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claude_config_artifacts_sha256: Option<String>,
    /// Bounded family-v3 stdout/trace byte count after terminal process-group cleanup.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdout_trace_bytes: Option<u64>,
    /// Bounded family-v3 stderr byte count after terminal process-group cleanup.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stderr_bytes: Option<u64>,
    /// True only when the evaluator proved the owned provider process group terminal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_cleanup_proven: Option<bool>,
    /// Runner-observed start of the provider process in Unix milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_started_unix_ms: Option<u64>,
    /// Runner-observed completion of the provider process in Unix milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_completed_unix_ms: Option<u64>,
    /// Process exit code. Claude may return 1 after a completed turn-boundary budget stop.
    pub exit_code: i32,
    /// Canonical isolated Codex rollout created by this exact provider admission.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_session_rollout_path: Option<String>,
    /// SHA-256 of the exact bounded Codex rollout bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_session_rollout_sha256: Option<String>,
    /// Host-reported model provider from the Codex session metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_model_provider: Option<String>,
    /// Host-resolved Codex model identifier; this is not a provider backend snapshot claim.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_host_resolved_model: Option<String>,
    /// SHA-256 of the concordant full-world-state `agents_md` object in the Codex rollout.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_agents_md_sha256: Option<String>,
    /// SHA-256 of the complete provider trace bytes sealed immediately after the call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_trace_sha256: Option<String>,
    /// Requested Claude model reported by the unique init event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claude_requested_model: Option<String>,
    /// Concordant host-resolved Claude model reported by every assistant event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claude_host_resolved_model: Option<String>,
    /// Provider-reported Claude cost, rounded up to whole micro-USD.
    pub provider_reported_cost_microusd: Option<u64>,
    /// Whether an exact Claude completed-turn or structured-output budget stop was accepted.
    pub accepted_turn_boundary_budget_exit: bool,
    /// Whether local post-processing resumed from a previously captured provider trace.
    pub recovered_from_existing_trace: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LaneRunAction {
    Execute,
    RecoverTeaching,
    RecoverEvaluation,
    Skip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ProviderCommandOutcome {
    exit_code: i32,
    provider_reported_cost_microusd: Option<u64>,
    accepted_turn_boundary_budget_exit: bool,
    safety: Option<ProviderSafetyEvidence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ProviderSafetyEvidence {
    stdout_trace_bytes: u64,
    stderr_bytes: u64,
    process_cleanup_proven: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProviderFailureSealIdentity {
    argv_sha256: String,
    environment_sha256: String,
    trace_path: PathBuf,
    stderr_path: PathBuf,
}

fn provider_failure_seal_identity(
    argv: &[String],
    environment: &BTreeMap<String, String>,
    trace_path: PathBuf,
    stderr_path: PathBuf,
) -> EvalResult<ProviderFailureSealIdentity> {
    if argv.is_empty() || trace_path.as_os_str().is_empty() || stderr_path.as_os_str().is_empty() {
        return Err(EvalError::Invalid(
            "provider failure seal identity is incomplete".to_string(),
        ));
    }
    Ok(ProviderFailureSealIdentity {
        argv_sha256: argv_sha256(argv)?,
        environment_sha256: canonical_environment_sha256(environment)?,
        trace_path,
        stderr_path,
    })
}

fn reconcile_pre_provider_receipt_failure(
    error: EvalError,
    reported_receipt_sha256: Option<&str>,
    evidence: Option<&NativeProviderBundlePreProviderReceiptEvidence>,
) -> EvalError {
    if reported_receipt_sha256 == evidence.map(|value| value.receipt_sha256.as_str()) {
        return error;
    }
    EvalError::Invalid(format!(
        "provider failure receipt evidence is inconsistent; the consumed bundle admission remains permanently non-replayable; original_failure_sha256={}",
        sha256_bytes(error.to_string().as_bytes())
    ))
}

enum ProviderLaneFailure {
    PreDispatchConfigArtifactDrift {
        error: EvalError,
        pre_dispatch_receipt_sha256: Option<String>,
    },
    PreDispatchRunnerFailure {
        error: EvalError,
        pre_dispatch_receipt_sha256: Option<String>,
    },
    PostDispatchFailure {
        error: EvalError,
        pre_dispatch_receipt_sha256: Option<String>,
        quarantine: Option<NativeExecutionChildGuard>,
    },
    PostSpawnOutputSetupFailure {
        error: EvalError,
        pre_dispatch_receipt_sha256: Option<String>,
        process_cleanup_proven: bool,
        quarantine: Option<NativeExecutionChildGuard>,
    },
}

enum BoundedProviderCommandFailure {
    PreSpawn(EvalError),
    PostSpawn(EvalError),
    PostSpawnCleanupUnproven {
        error: EvalError,
        quarantine: NativeExecutionChildGuard,
    },
    PostSpawnOutputSetup {
        error: EvalError,
        process_cleanup_proven: bool,
        quarantine: Option<NativeExecutionChildGuard>,
    },
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InjectedProviderOutputSetupFailure {
    StdoutFile,
    StderrFile,
    StdoutPipe,
    StderrPipe,
    TerminalCleanup,
}

#[cfg(test)]
thread_local! {
    static INJECT_NEXT_PROVIDER_OUTPUT_SETUP_FAILURE:
        std::cell::Cell<Option<InjectedProviderOutputSetupFailure>> = const {
            std::cell::Cell::new(None)
        };
}

#[cfg(test)]
fn inject_next_provider_output_setup_failure_for_test(failure: InjectedProviderOutputSetupFailure) {
    INJECT_NEXT_PROVIDER_OUTPUT_SETUP_FAILURE.with(|next| next.set(Some(failure)));
}

#[cfg(test)]
fn take_injected_provider_output_setup_failure(
    expected: InjectedProviderOutputSetupFailure,
) -> bool {
    INJECT_NEXT_PROVIDER_OUTPUT_SETUP_FAILURE.with(|next| {
        if next.get() == Some(expected) {
            next.set(None);
            true
        } else {
            false
        }
    })
}

impl BoundedProviderCommandFailure {
    fn into_error(self) -> EvalError {
        match self {
            Self::PreSpawn(error) | Self::PostSpawn(error) => error,
            Self::PostSpawnCleanupUnproven { error, .. } => error,
            Self::PostSpawnOutputSetup { error, .. } => error,
        }
    }

    fn into_lane_failure(self) -> ProviderLaneFailure {
        match self {
            Self::PreSpawn(error) => ProviderLaneFailure::pre_dispatch_runner_failure(error),
            Self::PostSpawn(error) => ProviderLaneFailure::post_dispatch_failure(error),
            Self::PostSpawnCleanupUnproven { error, quarantine } => {
                ProviderLaneFailure::PostDispatchFailure {
                    error,
                    pre_dispatch_receipt_sha256: None,
                    quarantine: Some(quarantine),
                }
            }
            Self::PostSpawnOutputSetup {
                error,
                process_cleanup_proven,
                quarantine,
            } => ProviderLaneFailure::PostSpawnOutputSetupFailure {
                error,
                pre_dispatch_receipt_sha256: None,
                process_cleanup_proven,
                quarantine,
            },
        }
    }
}

impl From<EvalError> for BoundedProviderCommandFailure {
    fn from(error: EvalError) -> Self {
        Self::PostSpawn(error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProviderLaneFailureStage {
    ConfigArtifactPreDispatch,
    RunnerPreDispatch,
    PostDispatch,
    PostSpawnOutputSetup { process_cleanup_proven: bool },
}

impl ProviderLaneFailure {
    fn pre_dispatch_config_artifact_drift(error: EvalError) -> Self {
        Self::PreDispatchConfigArtifactDrift {
            error,
            pre_dispatch_receipt_sha256: None,
        }
    }

    fn pre_dispatch_runner_failure(error: EvalError) -> Self {
        Self::PreDispatchRunnerFailure {
            error,
            pre_dispatch_receipt_sha256: None,
        }
    }

    fn post_dispatch_failure(error: EvalError) -> Self {
        Self::PostDispatchFailure {
            error,
            pre_dispatch_receipt_sha256: None,
            quarantine: None,
        }
    }

    fn with_pre_dispatch_receipt_sha256(self, pre_dispatch_receipt_sha256: Option<String>) -> Self {
        match self {
            Self::PreDispatchConfigArtifactDrift { error, .. } => {
                Self::PreDispatchConfigArtifactDrift {
                    error,
                    pre_dispatch_receipt_sha256,
                }
            }
            Self::PreDispatchRunnerFailure { error, .. } => Self::PreDispatchRunnerFailure {
                error,
                pre_dispatch_receipt_sha256,
            },
            Self::PostDispatchFailure {
                error, quarantine, ..
            } => Self::PostDispatchFailure {
                error,
                pre_dispatch_receipt_sha256,
                quarantine,
            },
            Self::PostSpawnOutputSetupFailure {
                error,
                process_cleanup_proven,
                quarantine,
                ..
            } => Self::PostSpawnOutputSetupFailure {
                error,
                pre_dispatch_receipt_sha256,
                process_cleanup_proven,
                quarantine,
            },
        }
    }

    fn into_parts(
        self,
    ) -> (
        EvalError,
        ProviderLaneFailureStage,
        Option<String>,
        Option<NativeExecutionChildGuard>,
    ) {
        match self {
            Self::PreDispatchConfigArtifactDrift {
                error,
                pre_dispatch_receipt_sha256,
            } => (
                error,
                ProviderLaneFailureStage::ConfigArtifactPreDispatch,
                pre_dispatch_receipt_sha256,
                None,
            ),
            Self::PreDispatchRunnerFailure {
                error,
                pre_dispatch_receipt_sha256,
            } => (
                error,
                ProviderLaneFailureStage::RunnerPreDispatch,
                pre_dispatch_receipt_sha256,
                None,
            ),
            Self::PostDispatchFailure {
                error,
                pre_dispatch_receipt_sha256,
                quarantine,
            } => (
                error,
                ProviderLaneFailureStage::PostDispatch,
                pre_dispatch_receipt_sha256,
                quarantine,
            ),
            Self::PostSpawnOutputSetupFailure {
                error,
                pre_dispatch_receipt_sha256,
                process_cleanup_proven,
                quarantine,
            } => (
                error,
                ProviderLaneFailureStage::PostSpawnOutputSetup {
                    process_cleanup_proven,
                },
                pre_dispatch_receipt_sha256,
                quarantine,
            ),
        }
    }
}

impl From<EvalError> for ProviderLaneFailure {
    fn from(error: EvalError) -> Self {
        Self::post_dispatch_failure(error)
    }
}

fn bind_post_dispatch_failure(error: EvalError, safety: ProviderSafetyEvidence) -> EvalError {
    let detail = error.to_string();
    if detail.contains("bounded provider limit triggered; ")
        || detail.contains("provider terminal ambiguous; ")
    {
        return error;
    }
    EvalError::Invalid(format!(
        "provider terminal ambiguous; reason=post_dispatch_evidence_or_terminal_failure; stdout_bytes={}; stderr_bytes={}; process_cleanup_proven={}; failure_sha256={}",
        safety.stdout_trace_bytes,
        safety.stderr_bytes,
        safety.process_cleanup_proven,
        sha256_bytes(detail.as_bytes()),
    ))
}

/// Durable report emitted only after an entire phase passes its postconditions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NativePilotRunReport {
    /// Pilot identifier.
    pub pilot_id: String,
    /// Executed phase.
    pub phase: NativePilotRunPhase,
    /// Canonical frozen run plan.
    pub run_plan: String,
    /// Exact aggregate Claude runner allocation explicitly confirmed by the caller.
    pub confirmed_claude_budget_cents: u32,
    /// Phase start in Unix milliseconds.
    pub started_unix_ms: u64,
    /// Phase completion in Unix milliseconds.
    pub completed_unix_ms: u64,
    /// Host invocations in preregistered order.
    pub lanes: Vec<NativePilotLaneExecution>,
    /// Digest of the runner-captured, pre-provider native marker snapshot for stale evaluation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stale_safety_pre_evaluation_marker_snapshot_sha256: Option<String>,
    /// Provider-free post-phase audit.
    pub audit: NativePilotAudit,
}

/// Sanitized no-secret evidence that the family-v3 auth lifecycle durably admitted and sealed
/// each exact treatment provider call. A runner report alone is not admission evidence.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NativeFamilyV3LifecycleAudit {
    pub exact_admission_count: u32,
    pub all_phase_receipts_valid: bool,
    pub claude_reported_cost_microusd: u64,
    pub claude_authorized_ceiling_microusd: u64,
    pub claude_turn_boundary_overshoot_microusd: u64,
    pub bindings: Vec<NativeFamilyV3LifecycleBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct NativeFamilyV3LifecycleBinding {
    pub admission_ordinal: u32,
    pub phase: NativePilotRunPhase,
    pub lane_order: u32,
    pub phase_receipt_path: String,
    pub phase_receipt_sha256: String,
    pub terminal_receipt_path: String,
    pub terminal_receipt_sha256: String,
    pub provider_started_unix_ms: u64,
    pub provider_completed_unix_ms: u64,
    pub provider_trace_sha256: Option<String>,
    pub effective_environment_sha256: Option<String>,
    pub claude_config_artifacts_sha256: Option<String>,
    pub stdout_trace_bytes: Option<u64>,
    pub stderr_bytes: Option<u64>,
    pub process_cleanup_proven: Option<bool>,
    pub provider_reported_cost_microusd: Option<u64>,
    pub accepted_turn_boundary_budget_exit: bool,
    pub codex_session_rollout_path: Option<String>,
    pub codex_session_rollout_sha256: Option<String>,
    pub codex_model_provider: Option<String>,
    pub codex_host_resolved_model: Option<String>,
    pub codex_agents_md_sha256: Option<String>,
    pub claude_requested_model: Option<String>,
    pub claude_host_resolved_model: Option<String>,
}

/// Provider-free re-attestation of a frozen native-memory pilot plan.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NativePilotPlanAttestation {
    /// True only after the plan digest, binaries, loader closure, argv, and live MCP runtime match.
    pub verified: bool,
    /// Pilot identifier from the frozen plan.
    pub pilot_id: String,
    /// Canonical run-plan path.
    pub run_plan: String,
    /// Re-attested executable identities.
    pub binaries: BTreeMap<String, PilotBinaryAttestation>,
    /// Re-attested loader-closed runtime-library identities for family-v3 plans.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub runtime_libraries: BTreeMap<String, NativePilotRuntimeLibraryAttestation>,
    /// Re-attested embedded and effective-runtime Engram MCP contract.
    pub engram_mcp_contract: EngramMcpContractAttestation,
}

/// Sanitized result of one provider-free Codex login-status check.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NativePilotCodexAuthState {
    /// The isolated lane can resolve ChatGPT through its frozen credential store.
    Ready,
    /// No login is available to the isolated lane.
    NotLoggedIn,
    /// Codex reported a different authentication mode.
    WrongAuthenticationMode,
    /// The provider-free status command failed unexpectedly.
    CheckFailed,
}

/// Authentication readiness for one isolated Codex lane.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NativePilotCodexLaneAuth {
    /// Fixed lane order.
    pub order: u32,
    /// Arm identifier.
    pub arm: String,
    /// Isolated Codex state root; file-cache plans may contain protected `auth.json`.
    pub codex_home: String,
    /// Sanitized readiness state. Raw status output is never persisted.
    pub state: NativePilotCodexAuthState,
}

/// Provider-free authentication readiness report for a frozen pilot.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NativePilotAuthReport {
    /// Stable pilot identifier.
    pub pilot_id: String,
    /// Canonical frozen run-plan path.
    pub run_plan: String,
    /// Always true: login status does not invoke a model provider.
    pub provider_free: bool,
    /// True only when every Codex lane reports its frozen ChatGPT authentication mode.
    pub ready: bool,
    /// Per-lane sanitized results.
    pub lanes: Vec<NativePilotCodexLaneAuth>,
    /// SHA-256 of the exact owner-only lifecycle receipt that binds cleanup authority.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_cache_lifecycle_manifest_sha256: Option<String>,
    /// Exact generated cache destinations. Credential bytes and source paths are never included.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub auth_cache_destinations: Vec<String>,
}

/// Exact destructive authority supplied to terminal auth-cache cleanup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativePilotAuthCacheCleanupConfirmation {
    /// Must equal the canonical parent directory of the frozen run plan.
    pub run_root: PathBuf,
    /// Must equal the lifecycle manifest digest displayed by provisioning.
    pub lifecycle_manifest_sha256: String,
    /// Must explicitly authorize deletion of the six manifest-bound plaintext copies.
    pub delete_six_plaintext_cache_copies: bool,
}

/// Terminal state of one guarded auth-cache cleanup attempt.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NativePilotAuthCacheCleanupState {
    /// All six exact files were unlinked and their absence was verified.
    Complete,
    /// Cleanup stopped without path-based deletion after an identity or lifecycle failure.
    Unresolved,
}

/// Sanitized durable cleanup result. It never contains credential bytes or a credential digest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NativePilotAuthCacheCleanupReport {
    pub schema_version: u32,
    pub pilot_id: String,
    pub run_plan: String,
    pub run_root: String,
    pub lifecycle_manifest_sha256: String,
    pub cleanup_intent_sha256: String,
    pub boot_id: String,
    pub state: NativePilotAuthCacheCleanupState,
    pub destination_count: usize,
    pub removed_destinations: Vec<String>,
    pub unresolved_reason: Option<String>,
    pub completed_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CodexAuthCacheFileIdentity {
    device: u64,
    inode: u64,
    owner: u32,
    mode: u32,
    link_count: u64,
    size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CodexAuthCacheDestinationPlan {
    lane_order: u32,
    auth_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CodexAuthCacheDestinationReceipt {
    lane_order: u32,
    auth_json: String,
    identity: CodexAuthCacheFileIdentity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CodexAuthCacheProvisionIntent {
    schema_version: u32,
    pilot_id: String,
    run_plan_sha256: String,
    boot_id: String,
    destinations: Vec<CodexAuthCacheDestinationPlan>,
    created_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CodexAuthCacheProvisionReceipt {
    schema_version: u32,
    pilot_id: String,
    run_plan_sha256: String,
    provision_intent_sha256: String,
    boot_id: String,
    destinations: Vec<CodexAuthCacheDestinationReceipt>,
    authentication_ready: bool,
    completed_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CodexAuthCacheUnresolvedReceipt {
    schema_version: u32,
    pilot_id: String,
    run_plan_sha256: String,
    provision_intent_sha256: String,
    boot_id: String,
    created_destination_count: usize,
    failure_code: String,
    completed_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CodexAuthCachePhaseIntent {
    schema_version: u32,
    pilot_id: String,
    run_plan_sha256: String,
    lifecycle_manifest_sha256: String,
    boot_id: String,
    phase: NativePilotRunPhase,
    lane_orders: Vec<u32>,
    created_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CodexSessionPreDispatchInventory {
    sessions_root: String,
    relative_file_paths: Vec<String>,
    captured_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CodexAuthCacheLaneAdmission {
    schema_version: u32,
    pilot_id: String,
    run_plan_sha256: String,
    lifecycle_manifest_sha256: String,
    boot_id: String,
    phase: NativePilotRunPhase,
    lane_order: u32,
    argv_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    claude_config_artifacts: Option<NativeStaleV3ClaudeConfigAttestation>,
    consumption_state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    codex_pre_dispatch_inventory: Option<CodexSessionPreDispatchInventory>,
    admitted_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CodexAuthCacheLaneTerminalReceipt {
    schema_version: u32,
    pilot_id: String,
    run_plan_sha256: String,
    lifecycle_manifest_sha256: String,
    boot_id: String,
    phase: NativePilotRunPhase,
    lane_order: u32,
    argv_sha256: String,
    lane_admission_sha256: String,
    provider_started_unix_ms: Option<u64>,
    provider_completed_unix_ms: Option<u64>,
    exit_code: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    codex_session_rollout_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    codex_session_rollout_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    codex_model_provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    codex_host_resolved_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    codex_agents_md_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    provider_trace_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    effective_environment_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    claude_config_artifacts_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    stdout_trace_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    stderr_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    process_cleanup_proven: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    provider_reported_cost_microusd: Option<u64>,
    #[serde(default)]
    accepted_turn_boundary_budget_exit: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    claude_requested_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    claude_host_resolved_model: Option<String>,
    sealed_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CodexAuthCachePhaseLaneBinding {
    lane_order: u32,
    terminal_receipt_sha256: String,
    provider_started_unix_ms: Option<u64>,
    provider_completed_unix_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    codex_session_rollout_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    codex_session_rollout_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    codex_model_provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    codex_host_resolved_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    codex_agents_md_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    provider_trace_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    effective_environment_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    claude_config_artifacts_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    stdout_trace_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    stderr_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    process_cleanup_proven: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    provider_reported_cost_microusd: Option<u64>,
    #[serde(default)]
    accepted_turn_boundary_budget_exit: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    claude_requested_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    claude_host_resolved_model: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CodexSessionRolloutAttestation {
    path: String,
    sha256: String,
    model_provider: String,
    host_resolved_model: String,
    agents_md_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ClaudeTraceModelAttestation {
    trace_sha256: String,
    requested_model: String,
    host_resolved_model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CodexAuthCachePhaseReceipt {
    schema_version: u32,
    pilot_id: String,
    run_plan_sha256: String,
    lifecycle_manifest_sha256: String,
    boot_id: String,
    phase: NativePilotRunPhase,
    phase_intent_sha256: String,
    runner_report_sha256: String,
    lane_orders: Vec<u32>,
    lane_bindings: Vec<CodexAuthCachePhaseLaneBinding>,
    completed_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CodexAuthCacheCleanupIntent {
    schema_version: u32,
    pilot_id: String,
    run_plan_sha256: String,
    lifecycle_manifest_sha256: String,
    run_root: String,
    boot_id: String,
    destinations: Vec<CodexAuthCacheDestinationReceipt>,
    authority: String,
    created_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CodexAuthCacheCleanupLaneIntent {
    schema_version: u32,
    cleanup_intent_sha256: String,
    boot_id: String,
    lane_order: u32,
    auth_json: String,
    identity: CodexAuthCacheFileIdentity,
    authority: String,
    created_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct CodexAuthCacheCleanupLaneReceipt {
    schema_version: u32,
    cleanup_intent_sha256: String,
    cleanup_lane_intent_sha256: String,
    boot_id: String,
    lane_order: u32,
    auth_json: String,
    identity: CodexAuthCacheFileIdentity,
    absent_after_unlink_and_parent_fsync: bool,
    completed_unix_ms: u64,
}

#[derive(Debug, Deserialize)]
struct RunnerAcceptanceContract {
    expected_repository_remote: String,
    required_context_keys: Vec<String>,
    required_task: String,
    required_conditions: BTreeMap<String, String>,
    #[serde(default)]
    required_prerequisite_sources: BTreeMap<String, NativePilotPrerequisiteSource>,
    forbidden_commands: Vec<String>,
    required_command: String,
    expected_exit_code: i32,
    expected_output_contains: String,
}

/// Re-attest a native-memory pilot plan without invoking Codex or Claude Code providers.
pub fn attest_native_memory_pilot_plan(run_plan: &Path) -> EvalResult<NativePilotPlanAttestation> {
    let plan = load_historical_native_plan(run_plan)?;
    if plan.execution_approved {
        return Err(EvalError::Invalid(
            "prepared native-memory plans must retain execution_approved=false".to_string(),
        ));
    }
    validate_plan_digest(run_plan)?;
    validate_native_stale_plan_binding(&plan, run_plan)?;
    validate_evaluation_recovery(&plan, run_plan)?;
    validate_execution_recovery(&plan)?;
    validate_plan_attestations(&plan)?;
    Ok(NativePilotPlanAttestation {
        verified: true,
        pilot_id: plan.pilot_id.clone(),
        run_plan: run_plan.canonicalize()?.display().to_string(),
        binaries: plan.binaries,
        runtime_libraries: plan.runtime_libraries,
        engram_mcp_contract: plan.engram_mcp_contract,
    })
}

/// Check isolated Codex homes for their frozen ChatGPT login mode without invoking a provider.
pub fn check_native_memory_pilot_authentication(
    run_plan: &Path,
) -> EvalResult<NativePilotAuthReport> {
    let plan = load_historical_native_plan(run_plan)?;
    validate_plan_digest(run_plan)?;
    validate_native_stale_plan_binding(&plan, run_plan)?;
    validate_evaluation_recovery(&plan, run_plan)?;
    validate_execution_recovery(&plan)?;
    validate_plan_attestations(&plan)?;
    let run_plan = run_plan.canonicalize()?;
    if requires_exact_six_codex_auth_lifecycle(&plan)? {
        if !auth_lifecycle_path(&run_plan)?.exists() {
            return Err(EvalError::Invalid(
                "family-v3 Codex authentication requires a provisioned exact-six lifecycle"
                    .to_string(),
            ));
        }
        let lifecycle = CodexAuthCacheLifecycle::open(&plan, &run_plan)?;
        lifecycle.validate_current_destinations(&plan)?;
        let report = inspect_codex_authentication_with_manifest(
            &plan,
            &run_plan,
            Some(lifecycle.manifest_sha256.clone()),
        )?;
        lifecycle.validate_current_destinations(&plan)?;
        Ok(report)
    } else {
        inspect_codex_authentication(&plan, &run_plan)
    }
}

/// Copy one already-authenticated ChatGPT cache into every pristine file-cache Codex lane.
///
/// The source contents and any content digest are intentionally absent from the returned report.
/// Existing destination files are never reused or overwritten.
pub fn provision_native_memory_pilot_auth_cache(
    run_plan: &Path,
    source_cache: &Path,
    confirm_plaintext_cache_copies: bool,
) -> EvalResult<NativePilotAuthReport> {
    if !confirm_plaintext_cache_copies {
        return Err(EvalError::Invalid(
            "auth-cache provisioning requires --confirm-plaintext-cache-copies".to_string(),
        ));
    }

    let plan = load_historical_native_plan(run_plan)?;
    validate_plan_digest(run_plan)?;
    validate_native_stale_plan_binding(&plan, run_plan)?;
    validate_evaluation_recovery(&plan, run_plan)?;
    validate_execution_recovery(&plan)?;
    validate_plan_attestations(&plan)?;
    if plan.evaluation_recovery.is_some() || plan.execution_recovery.is_some() {
        return Err(EvalError::Invalid(
            "auth-cache provisioning is allowed only for a fresh prepared plan".to_string(),
        ));
    }
    let audit = audit_native_memory_pilot(run_plan)?;
    if audit.invalid
        || audit
            .lanes
            .iter()
            .any(|lane| lane.phase != NativeLanePhase::Prepared || !lane.failures.is_empty())
    {
        return Err(EvalError::Invalid(
            "auth-cache provisioning requires pristine prepared lanes with zero failures"
                .to_string(),
        ));
    }

    let run_plan = run_plan.canonicalize()?;
    if requires_exact_six_codex_auth_lifecycle(&plan)? {
        return provision_exact_six_codex_auth_cache_lifecycle(&plan, &run_plan, source_cache);
    }

    let mut destinations = Vec::new();
    for lane in plan.lanes.iter().filter(|lane| lane.host == "codex") {
        let authentication = lane.codex_authentication.as_ref().ok_or_else(|| {
            EvalError::Invalid(format!(
                "Codex lane {} is missing an authentication contract",
                lane.order
            ))
        })?;
        if authentication.mode != CodexAuthenticationMode::ChatgptFileCache {
            return Err(EvalError::Invalid(format!(
                "Codex lane {} is not frozen for chatgpt_file_cache",
                lane.order
            )));
        }
        let home = lane.environment.get("CODEX_HOME").ok_or_else(|| {
            EvalError::Invalid(format!("Codex lane {} is missing CODEX_HOME", lane.order))
        })?;
        let canonical_home = Path::new(home).canonicalize()?;
        if validate_codex_file_cache(&canonical_home)? {
            return Err(EvalError::Invalid(format!(
                "refusing to overwrite an existing Codex auth cache in lane {}",
                lane.order
            )));
        }
        destinations.push(canonical_home.join("auth.json"));
    }
    if destinations.is_empty() {
        return Err(EvalError::Invalid(
            "auth-cache provisioning requires at least one Codex lane".to_string(),
        ));
    }

    let canonical_source = source_cache.canonicalize()?;
    if destinations.iter().any(|path| {
        path.parent()
            .is_some_and(|home| canonical_source.starts_with(home))
    }) {
        return Err(EvalError::Invalid(
            "source auth cache must remain outside the isolated lane homes".to_string(),
        ));
    }
    for destination in &destinations {
        match fs::symlink_metadata(destination) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Ok(_) => {
                return Err(EvalError::Invalid(format!(
                    "refusing to overwrite Codex auth cache: {}",
                    destination.display()
                )))
            }
            Err(error) => return Err(EvalError::Io(error)),
        }
    }

    let source = open_validated_codex_auth_cache(source_cache, None)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;

        let source_owner = source.metadata()?.uid();
        for destination in &destinations {
            let home_owner = fs::symlink_metadata(destination.parent().ok_or_else(|| {
                EvalError::Invalid("Codex auth cache destination has no parent".to_string())
            })?)?
            .uid();
            if source_owner != home_owner {
                return Err(EvalError::Invalid(
                    "source auth cache owner must match every isolated Codex home".to_string(),
                ));
            }
        }
    }
    let cache = read_validated_codex_auth_cache(source, source_cache)?;
    let mut installed = Vec::new();
    for destination in &destinations {
        if let Err(error) = write_private_codex_auth_cache(destination, &cache) {
            for path in &installed {
                let _ = fs::remove_file(path);
            }
            return Err(error);
        }
        installed.push(destination.clone());
    }

    validate_plan_attestations(&plan)?;
    inspect_codex_authentication(&plan, &run_plan)
}

/// Remove the exact six family-v3 plaintext cache copies after terminal evaluation completion.
///
/// Cleanup is manifest-, identity-, and boot-bound. Once any cleanup receipt exists, subsequent
/// calls are read-only and return that durable terminal state.
pub fn cleanup_native_memory_pilot_auth_cache(
    run_plan: &Path,
    confirmation: NativePilotAuthCacheCleanupConfirmation,
) -> EvalResult<NativePilotAuthCacheCleanupReport> {
    if !confirmation.delete_six_plaintext_cache_copies {
        return Err(EvalError::Invalid(
            "auth-cache cleanup requires --confirm-delete-six-plaintext-cache-copies".to_string(),
        ));
    }
    let plan = load_historical_native_plan(run_plan)?;
    validate_plan_digest(run_plan)?;
    validate_native_stale_plan_binding(&plan, run_plan)?;
    validate_evaluation_recovery(&plan, run_plan)?;
    validate_execution_recovery(&plan)?;
    validate_plan_attestations(&plan)?;
    if !requires_exact_six_codex_auth_lifecycle(&plan)? {
        return Err(EvalError::Invalid(
            "guarded auth-cache cleanup is available only for stale-safety family version 3"
                .to_string(),
        ));
    }

    let run_plan = run_plan.canonicalize()?;
    let run_root = run_plan
        .parent()
        .ok_or_else(|| EvalError::Invalid("run plan has no parent".to_string()))?
        .canonicalize()?;
    if confirmation.run_root.canonicalize()? != run_root {
        return Err(EvalError::Invalid(
            "auth-cache cleanup confirmation does not name the canonical run root".to_string(),
        ));
    }
    if !is_sha256(&confirmation.lifecycle_manifest_sha256) {
        return Err(EvalError::Invalid(
            "auth-cache cleanup confirmation has an invalid lifecycle manifest digest".to_string(),
        ));
    }

    let mut lifecycle = CodexAuthCacheLifecycle::open(&plan, &run_plan)?;
    if lifecycle.manifest_sha256 != confirmation.lifecycle_manifest_sha256 {
        return Err(EvalError::Invalid(
            "auth-cache cleanup confirmation does not bind the provision receipt".to_string(),
        ));
    }
    lifecycle.cleanup(&plan, &run_plan, &run_root)
}

fn requires_exact_six_codex_auth_lifecycle(plan: &PreparedNativePilot) -> EvalResult<bool> {
    let Some(binding) = plan.stale_safety.as_ref() else {
        return Ok(false);
    };
    match binding.family_version {
        1 => Ok(false),
        CODEX_AUTH_CACHE_LIFECYCLE_FAMILY_VERSION => {
            let codex = plan
                .lanes
                .iter()
                .filter(|lane| lane.host == "codex")
                .collect::<Vec<_>>();
            if codex.len() != 6
                || codex.iter().any(|lane| {
                    lane.codex_authentication.as_ref().map(|auth| auth.mode)
                        != Some(CodexAuthenticationMode::ChatgptFileCache)
                })
            {
                return Err(EvalError::Invalid(
                    "stale-safety family version 3 requires exactly six Codex chatgpt_file_cache lanes"
                        .to_string(),
                ));
            }
            Ok(true)
        }
        version => Err(EvalError::Invalid(format!(
            "unsupported auth-cache lifecycle stale-safety family version {version}"
        ))),
    }
}

fn auth_lifecycle_path(run_plan: &Path) -> EvalResult<PathBuf> {
    Ok(run_plan
        .parent()
        .ok_or_else(|| EvalError::Invalid("run plan has no parent".to_string()))?
        .join(CODEX_AUTH_CACHE_LIFECYCLE_DIRECTORY))
}

#[derive(Debug)]
struct CodexAuthCacheTarget {
    lane_order: u32,
    home: PathBuf,
    auth_json: PathBuf,
}

fn exact_six_codex_auth_targets(
    plan: &PreparedNativePilot,
) -> EvalResult<Vec<CodexAuthCacheTarget>> {
    if !requires_exact_six_codex_auth_lifecycle(plan)? {
        return Err(EvalError::Invalid(
            "exact-six auth-cache targets require stale-safety family version 3".to_string(),
        ));
    }
    let mut targets = Vec::new();
    let mut homes = BTreeSet::new();
    for lane in plan.lanes.iter().filter(|lane| lane.host == "codex") {
        let home = lane.environment.get("CODEX_HOME").ok_or_else(|| {
            EvalError::Invalid(format!("Codex lane {} is missing CODEX_HOME", lane.order))
        })?;
        let home = Path::new(home).canonicalize()?;
        if !homes.insert(home.clone()) {
            return Err(EvalError::Invalid(
                "family-v3 Codex lanes must have six distinct homes".to_string(),
            ));
        }
        targets.push(CodexAuthCacheTarget {
            lane_order: lane.order,
            auth_json: home.join("auth.json"),
            home,
        });
    }
    targets.sort_by_key(|target| target.lane_order);
    Ok(targets)
}

fn expected_phase_lane_orders(plan: &PreparedNativePilot, phase: NativePilotRunPhase) -> Vec<u32> {
    plan.lanes
        .iter()
        .filter(|lane| phase != NativePilotRunPhase::Activation || lane.host == "codex")
        .map(|lane| lane.order)
        .collect()
}

fn phase_predecessors(phase: NativePilotRunPhase) -> Vec<NativePilotRunPhase> {
    match phase {
        NativePilotRunPhase::Teaching => Vec::new(),
        NativePilotRunPhase::Activation => vec![NativePilotRunPhase::Teaching],
        NativePilotRunPhase::Evaluation => vec![
            NativePilotRunPhase::Teaching,
            NativePilotRunPhase::Activation,
        ],
    }
}

fn phase_lifecycle_stem(phase: NativePilotRunPhase, suffix: &str) -> String {
    format!("phase-{}-{suffix}", phase.label())
}

fn lane_lifecycle_stem(phase: NativePilotRunPhase, lane_order: u32, suffix: &str) -> String {
    format!("phase-{}-lane-{lane_order:03}-{suffix}", phase.label())
}

fn cleanup_lane_lifecycle_stem(lane_order: u32, suffix: &str) -> String {
    format!("cleanup-lane-{lane_order:03}-{suffix}")
}

fn lane_phase_argv(lane: &PreparedNativeLane, phase: NativePilotRunPhase) -> EvalResult<&[String]> {
    match phase {
        NativePilotRunPhase::Teaching => Ok(&lane.teaching_argv),
        NativePilotRunPhase::Activation => lane.activation_argv.as_deref().ok_or_else(|| {
            EvalError::Invalid(format!("lane {} has no activation command", lane.order))
        }),
        NativePilotRunPhase::Evaluation => Ok(&lane.evaluation_argv),
    }
}

fn provision_exact_six_codex_auth_cache_lifecycle(
    plan: &PreparedNativePilot,
    run_plan: &Path,
    source_cache: &Path,
) -> EvalResult<NativePilotAuthReport> {
    #[cfg(not(unix))]
    {
        let _ = (plan, run_plan, source_cache);
        return Err(EvalError::Invalid(
            "Codex auth-cache lifecycle requires a supported Unix host".to_string(),
        ));
    }

    #[cfg(unix)]
    {
        let targets = exact_six_codex_auth_targets(plan)?;
        let canonical_source = source_cache.canonicalize().map_err(|_| {
            EvalError::Invalid("source auth cache could not be resolved".to_string())
        })?;
        if targets
            .iter()
            .any(|target| canonical_source.starts_with(&target.home))
        {
            return Err(EvalError::Invalid(
                "source auth cache must remain outside all isolated lane homes".to_string(),
            ));
        }
        let source = open_source_codex_auth_cache(source_cache)?;
        let source_identity = codex_auth_identity(&source.metadata()?)?;
        let effective_uid = unsafe { libc::geteuid() };
        if source_identity.owner != effective_uid {
            return Err(EvalError::Invalid(
                "source auth cache must be owned by the effective user".to_string(),
            ));
        }
        let source_bytes = read_source_codex_auth_cache(source)?;

        let mut destination_plans = Vec::with_capacity(6);
        for target in &targets {
            let home = open_private_codex_home(&target.home, effective_uid)?;
            require_auth_entry_absent(&home)?;
            destination_plans.push(CodexAuthCacheDestinationPlan {
                lane_order: target.lane_order,
                auth_json: target.auth_json.display().to_string(),
            });
        }

        let run_plan_sha256 = sha256_file(run_plan)?;
        let boot_id = current_boot_id()?;
        let mut lifecycle = CodexAuthCacheLifecycle::create_new(run_plan)?;
        let intent = CodexAuthCacheProvisionIntent {
            schema_version: CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION,
            pilot_id: plan.pilot_id.clone(),
            run_plan_sha256: run_plan_sha256.clone(),
            boot_id: boot_id.clone(),
            destinations: destination_plans,
            created_unix_ms: unix_ms(SystemTime::now())?,
        };
        let intent_sha256 = lifecycle
            .directory
            .write_json_pair_exclusive("provision-intent", &intent)?;

        let mut created = Vec::with_capacity(6);
        let result = (|| -> EvalResult<NativePilotAuthReport> {
            for target in &targets {
                let home = open_private_codex_home(&target.home, effective_uid)?;
                let identity = create_auth_cache_relative(&home, &source_bytes)?;
                created.push(CodexAuthCacheDestinationReceipt {
                    lane_order: target.lane_order,
                    auth_json: target.auth_json.display().to_string(),
                    identity,
                });
            }

            let auth_report = inspect_codex_authentication_bounded(plan, run_plan)?;
            if !auth_report.ready {
                return Err(EvalError::Invalid(
                    "bounded Codex authentication status did not confirm all six copies"
                        .to_string(),
                ));
            }
            validate_destination_receipts(&targets, &created, effective_uid)?;

            let source_after = open_source_codex_auth_cache(source_cache)?;
            let source_after_identity = codex_auth_identity(&source_after.metadata()?)?;
            let source_after_bytes = read_source_codex_auth_cache(source_after)?;
            if source_after_identity != source_identity
                || source_after_bytes.as_slice() != source_bytes.as_slice()
            {
                return Err(EvalError::Invalid(
                    "source auth cache identity or bytes changed during provisioning".to_string(),
                ));
            }

            let receipt = CodexAuthCacheProvisionReceipt {
                schema_version: CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION,
                pilot_id: plan.pilot_id.clone(),
                run_plan_sha256,
                provision_intent_sha256: intent_sha256.clone(),
                boot_id,
                destinations: created.clone(),
                authentication_ready: true,
                completed_unix_ms: unix_ms(SystemTime::now())?,
            };
            let manifest_sha256 = lifecycle
                .directory
                .write_json_pair_exclusive("provision-receipt", &receipt)?;
            lifecycle.receipt = Some(receipt);
            lifecycle.manifest_sha256 = manifest_sha256.clone();
            lifecycle.validate_current_destinations(plan)?;
            inspect_codex_authentication_with_manifest(plan, run_plan, Some(manifest_sha256))
        })();

        match result {
            Ok(report) => Ok(report),
            Err(error) => {
                let unresolved = CodexAuthCacheUnresolvedReceipt {
                    schema_version: CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION,
                    pilot_id: plan.pilot_id.clone(),
                    run_plan_sha256: sha256_file(run_plan)?,
                    provision_intent_sha256: intent_sha256,
                    boot_id: current_boot_id().unwrap_or_else(|_| "unavailable".to_string()),
                    created_destination_count: created.len(),
                    failure_code: "provisioning_unresolved".to_string(),
                    completed_unix_ms: unix_ms(SystemTime::now())?,
                };
                let _ = lifecycle
                    .directory
                    .write_json_pair_exclusive("provision-unresolved", &unresolved);
                Err(EvalError::Invalid(format!(
                    "exact-six auth-cache provisioning is unresolved and retained all residue; no destination was rolled back: {error}"
                )))
            }
        }
    }
}

/// Prepare a successor after a local teaching-phase audit interruption.
///
/// Completed provider traces and trusted evaluator outputs remain at their original paths and are
/// hash-attested. The successor keeps the source lanes unchanged, skips those completed lanes, and
/// budgets only provider calls that have not yet occurred.
pub fn prepare_native_memory_pilot_execution_recovery(
    source_run_plan: &Path,
    output: &Path,
) -> EvalResult<PreparedNativePilot> {
    let loaded_source = load_historical_native_plan_with_identity(source_run_plan)?;
    validate_loaded_plan_digest(&loaded_source)?;
    let source_run_plan = loaded_source.canonical_path;
    let source_run_plan_sha256 = loaded_source.sha256;
    let source = loaded_source.plan;
    if source.stale_safety.is_some()
        || validate_native_stale_plan_binding(&source, &source_run_plan)?
    {
        return Err(EvalError::Invalid(
            "stale-safety plans forbid lifecycle recovery and provider replay".to_string(),
        ));
    }
    if source.evaluation_recovery.is_some() || source.execution_recovery.is_some() {
        return Err(EvalError::Invalid(
            "lifecycle recovery plans cannot be chained".to_string(),
        ));
    }
    validate_plan_attestations(&source)?;
    let source_audit = audit_native_memory_pilot(&source_run_plan)?;
    let interruption =
        validate_interrupted_teaching_audit(&source, &source_run_plan, &source_audit)?;
    let inherited_files = attest_inherited_teaching_files(
        &source,
        &interruption.completed_lane_orders,
        interruption.kind,
    )?;
    let (recovery_budget_cents, prior_spend, claude_remaining_calls) =
        execution_recovery_accounting(&source, &interruption.completed_lane_orders)?;

    let mut binaries = source.binaries.clone();
    let source_eval = binaries.get("engram_eval").ok_or_else(|| {
        EvalError::Invalid("source plan is missing engram_eval attestation".to_string())
    })?;
    let current_eval = attest_binary(&std::env::current_exe().map_err(EvalError::Io)?)?;
    if current_eval.version != source_eval.version || current_eval.sha256 == source_eval.sha256 {
        return Err(EvalError::Invalid(
            "execution recovery requires the same evaluator version with a distinct repaired binary"
                .to_string(),
        ));
    }
    binaries.insert("engram_eval".to_string(), current_eval);

    let accounted = prior_spend
        .checked_add(u64::from(recovery_budget_cents) * 10_000)
        .ok_or_else(|| EvalError::Invalid("Claude recovery accounting overflow".to_string()))?;
    if accounted > u64::from(source.claude_authorized_ceiling_cents) * 10_000 {
        return Err(EvalError::Invalid(
            "Claude execution-recovery allocation exceeds the existing cumulative ceiling"
                .to_string(),
        ));
    }

    require_empty_target(output)?;
    let output = output.canonicalize()?;
    let recovery = PreparedNativeExecutionRecovery {
        source_run_plan: source_run_plan.display().to_string(),
        source_run_plan_sha256,
        interrupted_phase: NativePilotRunPhase::Teaching.label().to_string(),
        failure_reason: interruption.kind.failure_reason().to_string(),
        recovery_prepared_unix_ms: unix_ms(SystemTime::now())?,
        completed_lane_orders: interruption.completed_lane_orders,
        inherited_files,
        claude_remaining_calls,
    };
    let prepared = PreparedNativePilot {
        protocol_schema_version: source.protocol_schema_version,
        pilot_id: execution_recovery_pilot_id(&source, &binaries),
        execution_approved: false,
        prepared_unix_ms: source.prepared_unix_ms,
        codex_min_idle_hours: source.codex_min_idle_hours,
        resource_budgets: source.resource_budgets.clone(),
        claude_budget_cents: recovery_budget_cents,
        claude_prior_spend_microusd: prior_spend,
        claude_authorized_ceiling_cents: source.claude_authorized_ceiling_cents,
        claude_model: source.claude_model.clone(),
        claude_max_turns: source.claude_max_turns,
        native_memory_references: source.native_memory_references.clone(),
        binaries,
        runtime_libraries: source.runtime_libraries.clone(),
        engram_mcp_contract: source.engram_mcp_contract.clone(),
        evaluation_recovery: None,
        execution_recovery: Some(recovery),
        stale_safety: None,
        lanes: source.lanes.clone(),
    };
    let run_plan = output.join("run-plan.json");
    write_private_json(&run_plan, &prepared)?;
    write_private_text(
        &output.join("run-plan.sha256"),
        &format!("{}\n", sha256_file(&run_plan)?),
    )?;
    validate_execution_recovery(&prepared)?;
    validate_plan_attestations(&prepared)?;
    let audit = audit_native_memory_pilot(&run_plan)?;
    if audit.invalid
        || audit
            .lanes
            .iter()
            .map(|lane| lane.phase)
            .ne(source_audit.lanes.iter().map(|lane| lane.phase))
    {
        return Err(EvalError::Invalid(
            "prepared execution recovery did not preserve the source lifecycle".to_string(),
        ));
    }
    Ok(prepared)
}

/// Prepare an evaluation-only successor after an exact, locally attested host-boundary failure.
///
/// The successor references the immutable teaching, activation, memory, and acceptance artifacts
/// from the source plan. It creates new evaluation destinations and never copies, repairs,
/// overwrites, or replays a provider artifact.
pub fn prepare_native_memory_pilot_evaluation_recovery(
    source_run_plan: &Path,
    output: &Path,
    codex_binary: Option<&Path>,
) -> EvalResult<PreparedNativePilot> {
    let loaded_source = load_historical_native_plan_with_identity(source_run_plan)?;
    validate_loaded_plan_digest(&loaded_source)?;
    let source_run_plan = loaded_source.canonical_path;
    let source_run_plan_sha256 = loaded_source.sha256;
    let source = loaded_source.plan;
    if source.stale_safety.is_some()
        || validate_native_stale_plan_binding(&source, &source_run_plan)?
    {
        return Err(EvalError::Invalid(
            "stale-safety plans forbid lifecycle recovery and provider replay".to_string(),
        ));
    }
    if source.evaluation_recovery.is_some() || source.execution_recovery.is_some() {
        return Err(EvalError::Invalid(
            "evaluation recovery plans cannot be chained".to_string(),
        ));
    }
    validate_plan_contracts(&source)?;
    let (recovery_kind, rejected_order) =
        classify_source_evaluation_rejection(&source, &source_run_plan)?;
    let binaries = attest_evaluation_recovery_binaries(&source, codex_binary, recovery_kind)?;

    require_empty_target(output)?;
    let output = output.canonicalize()?;
    create_private_dir_all(&output)?;
    let schema_path = output.join("agent-output.schema.json");
    write_private_text(&schema_path, AGENT_OUTPUT_SCHEMA)?;

    let (recovery_budget_cents, prior_spend, claude_remaining_calls) =
        evaluation_recovery_accounting(&source, recovery_kind)?;
    let accounted = prior_spend
        .checked_add(u64::from(recovery_budget_cents) * 10_000)
        .ok_or_else(|| EvalError::Invalid("Claude recovery accounting overflow".to_string()))?;
    if accounted > u64::from(source.claude_authorized_ceiling_cents) * 10_000 {
        return Err(EvalError::Invalid(
            "Claude recovery allocation exceeds the existing cumulative ceiling".to_string(),
        ));
    }

    let mut lanes = source.lanes.clone();
    for lane in &mut lanes {
        let lane_dir = output.join(format!("lane-{:02}-{}", lane.order, lane.arm));
        create_private_dir_all(&lane_dir)?;
        lane.evaluation_argv = rewrite_evaluation_argv_for_recovery(
            &lane.evaluation_argv,
            &lane.host,
            &schema_path,
            &binaries[&lane.host],
            recovery_kind,
        )?;
        if lane.host == "codex" {
            rewrite_codex_authentication_for_recovery(lane, &binaries["codex"])?;
        }
        lane.evaluation_trace_path = lane_dir
            .join("evaluation-trace.jsonl")
            .display()
            .to_string();
        lane.agent_output_path = lane_dir.join("agent-output.json").display().to_string();
    }

    let rejected_lane = source
        .lanes
        .iter()
        .find(|lane| lane.order == rejected_order)
        .ok_or_else(|| EvalError::Invalid("rejected source lane disappeared".to_string()))?;
    let rejected_trace = Path::new(&rejected_lane.evaluation_trace_path);
    let rejected_stderr = stderr_path(rejected_trace);
    let recovery = PreparedNativeEvaluationRecovery {
        source_run_plan: source_run_plan.display().to_string(),
        source_run_plan_sha256,
        rejected_evaluation_trace: rejected_trace.display().to_string(),
        rejected_evaluation_trace_sha256: sha256_file(rejected_trace)?,
        rejected_evaluation_stderr: rejected_stderr.display().to_string(),
        rejected_evaluation_stderr_sha256: sha256_file(&rejected_stderr)?,
        rejection_reason: recovery_rejection_reason(recovery_kind).to_string(),
        recovery_prepared_unix_ms: unix_ms(SystemTime::now())?,
        claude_remaining_calls,
        agent_output_schema: schema_path.display().to_string(),
    };
    let prepared = PreparedNativePilot {
        protocol_schema_version: source.protocol_schema_version,
        pilot_id: evaluation_recovery_pilot_id(&source, &binaries, recovery_kind),
        execution_approved: false,
        prepared_unix_ms: source.prepared_unix_ms,
        codex_min_idle_hours: source.codex_min_idle_hours,
        resource_budgets: source.resource_budgets.clone(),
        claude_budget_cents: recovery_budget_cents,
        claude_prior_spend_microusd: prior_spend,
        claude_authorized_ceiling_cents: source.claude_authorized_ceiling_cents,
        claude_model: source.claude_model.clone(),
        claude_max_turns: source.claude_max_turns,
        native_memory_references: source.native_memory_references.clone(),
        binaries,
        runtime_libraries: source.runtime_libraries.clone(),
        engram_mcp_contract: source.engram_mcp_contract.clone(),
        evaluation_recovery: Some(recovery),
        execution_recovery: None,
        stale_safety: None,
        lanes,
    };
    let run_plan = output.join("run-plan.json");
    write_private_json(&run_plan, &prepared)?;
    write_private_text(
        &output.join("run-plan.sha256"),
        &format!("{}\n", sha256_file(&run_plan)?),
    )?;
    validate_evaluation_recovery(&prepared, &run_plan)?;
    validate_plan_attestations(&prepared)?;
    let audit = audit_native_memory_pilot(&run_plan)?;
    if audit.invalid || !audit.ready_for_evaluation {
        return Err(EvalError::Invalid(
            "prepared evaluation recovery did not preserve a ready lifecycle".to_string(),
        ));
    }
    Ok(prepared)
}

/// Execute exactly one phase after all approval and lifecycle gates pass.
pub fn run_native_memory_pilot(
    run_plan: &Path,
    phase: NativePilotRunPhase,
    approval: NativePilotExecutionApproval,
) -> EvalResult<NativePilotRunReport> {
    run_native_memory_pilot_with_bundle(run_plan, phase, approval, None)
}

/// Execute the exact four evaluation-only instructions controls after the family-v3 treatment
/// has a complete, independently validated 30-admission lifecycle. Every provider call consumes
/// its parent-bundle ordinal before dispatch and becomes permanently non-replayable.
pub fn run_native_stale_instructions_control(
    control_run_plan: &Path,
    provider_bundle: &Path,
    approval: NativePilotExecutionApproval,
) -> EvalResult<NativeInstructionsControlRunReport> {
    if !approval.provider_execution || approval.claude_budget_cents != Some(10) {
        return Err(EvalError::Invalid(
            "instructions-control execution requires provider approval and the exact 10 cent Claude ceiling"
                .to_string(),
        ));
    }
    let control_plan = load_control_plan(control_run_plan)?;
    validate_prepared_control(&control_plan, control_run_plan, true)?;
    let control_run_plan = control_run_plan.canonicalize()?;
    let report_path = control_run_plan
        .parent()
        .ok_or_else(|| EvalError::Invalid("control run plan has no parent".to_string()))?
        .join("runner-evaluation.json");
    require_absent(&report_path, "instructions-control runner report")?;
    require_absent(
        &report_path.with_extension("sha256"),
        "instructions-control runner report digest",
    )?;

    let treatment_run_plan = Path::new(&control_plan.treatment_run_plan).canonicalize()?;
    let treatment_plan = load_historical_native_plan(&treatment_run_plan)?;
    validate_plan_digest(&treatment_run_plan)?;
    validate_native_stale_plan_binding(&treatment_plan, &treatment_run_plan)?;
    validate_plan_attestations(&treatment_plan)?;
    let treatment_receipt = read_validated_stale_phase_receipt(
        &treatment_plan,
        &treatment_run_plan,
        NativePilotRunPhase::Evaluation,
    )?;
    if !treatment_receipt.audit.complete || treatment_receipt.audit.invalid {
        return Err(EvalError::Invalid(
            "instructions controls require a complete valid treatment evaluation receipt"
                .to_string(),
        ));
    }
    let lifecycle_audit = audit_native_family_v3_lifecycle(&treatment_run_plan)?;
    if lifecycle_audit.exact_admission_count != 30
        || !lifecycle_audit.all_phase_receipts_valid
        || lifecycle_audit.bindings.len() != 30
    {
        return Err(EvalError::Invalid(
            "instructions controls require exactly 30 sealed treatment lifecycle admissions"
                .to_string(),
        ));
    }
    let dynamic_forbidden_trace_needles =
        treatment_dynamic_forbidden_trace_needles(&treatment_plan, &treatment_run_plan)?;
    let mut claude_artifact_guards = control_plan
        .lanes
        .iter()
        .map(|lane| open_control_claude_phase_artifacts(&control_plan, lane))
        .collect::<EvalResult<Vec<_>>>()?;

    // Both locks intentionally span every control dispatch. This prevents auth cleanup, another
    // treatment phase, or another control runner from interleaving with the four admissions.
    let auth_lifecycle = CodexAuthCacheLifecycle::open(&treatment_plan, &treatment_run_plan)?;
    auth_lifecycle.validate_current_destinations(&treatment_plan)?;
    require_codex_authentication_ready(&treatment_plan, &treatment_run_plan, true)?;
    auth_lifecycle.validate_current_destinations(&treatment_plan)?;
    let bundle = NativeProviderBundleJournal::open_control(provider_bundle, &control_run_plan)?;
    let treatment_bundle_predecessors = lifecycle_audit
        .bindings
        .iter()
        .map(|binding| NativeProviderBundlePredecessorEvidence {
            ordinal: binding.admission_ordinal,
            child: "treatment".to_string(),
            terminal_receipt_sha256: binding.terminal_receipt_sha256.clone(),
            provider_started_unix_ms: binding.provider_started_unix_ms,
            provider_completed_unix_ms: binding.provider_completed_unix_ms,
        })
        .collect::<Vec<_>>();

    let started_unix_ms = unix_ms(SystemTime::now())?;
    let mut executions: Vec<NativeInstructionsControlExecution> = Vec::with_capacity(4);
    let mut claude_cost_microusd = 0_u64;
    for (lane, claude_artifact_guard) in control_plan
        .lanes
        .iter()
        .zip(claude_artifact_guards.iter_mut())
    {
        if claude_cost_microusd >= 100_000 {
            return Err(EvalError::Invalid(
                "instructions-control Claude aggregate ceiling was reached before the next provider admission"
                    .to_string(),
            ));
        }
        validate_control_lane_configured_absence(lane)?;
        auth_lifecycle.validate_current_destinations(&treatment_plan)?;
        let source = treatment_plan
            .lanes
            .iter()
            .find(|source| source.order == lane.source_treatment_lane_order)
            .ok_or_else(|| {
                EvalError::Invalid("control source treatment lane is absent".to_string())
            })?;
        validate_control_native_artifact_absence(source)?;
        let paired = treatment_receipt
            .lanes
            .iter()
            .find(|execution| execution.order == lane.source_treatment_lane_order)
            .ok_or_else(|| {
                EvalError::Invalid("paired treatment execution is absent".to_string())
            })?;
        let codex_inventory = if lane.host == "codex" {
            Some(capture_codex_pre_dispatch_inventory(source)?)
        } else {
            None
        };
        if let Some(guard) = claude_artifact_guard.as_mut() {
            guard.revalidate()?;
        }
        let mut bundle_predecessors = treatment_bundle_predecessors.clone();
        for prior_execution in &executions {
            let prior_lane = control_plan
                .lanes
                .iter()
                .find(|prepared| prepared.admission_ordinal == prior_execution.admission_ordinal)
                .ok_or_else(|| {
                    EvalError::Invalid(
                        "completed control predecessor is absent from the frozen plan".to_string(),
                    )
                })?;
            bundle_predecessors.push(validated_control_bundle_predecessor_evidence(
                &treatment_plan,
                &control_plan,
                prior_lane,
                prior_execution,
                &dynamic_forbidden_trace_needles,
            )?);
        }
        let failure_seal_identity = provider_failure_seal_identity(
            &lane.evaluation_argv,
            &lane.environment,
            PathBuf::from(&lane.trace_path),
            PathBuf::from(&lane.stderr_path),
        )?;
        let bundle_admission = NativeProviderBundleAdmission {
            ordinal: lane.admission_ordinal,
            child: "control".to_string(),
            phase: "evaluation".to_string(),
            lane_order: lane.source_treatment_lane_order,
            host: lane.host.clone(),
            case_id: lane.case_id.clone(),
            repetition: lane.repetition,
            arm: lane.arm.clone(),
        };
        let admission_sha256 = bundle.admit(&bundle_admission, &bundle_predecessors)?;
        let mut pre_provider_receipt_evidence = None;
        let result = execute_control_lane(
            &control_plan,
            lane,
            source,
            paired,
            (codex_inventory.as_ref(), claude_artifact_guard.as_mut()),
            &dynamic_forbidden_trace_needles,
            &mut pre_provider_receipt_evidence,
            || {
                bundle.revalidate_dispatch_boundary(
                    lane.admission_ordinal,
                    &admission_sha256,
                    &bundle_predecessors,
                )
            },
        );
        match result {
            Ok((execution, terminal_sha256)) => {
                bundle.seal(
                    lane.admission_ordinal,
                    &admission_sha256,
                    NativeProviderAdmissionOutcome::Terminal,
                    Some(terminal_sha256),
                    None,
                    Some(execution.provider_started_unix_ms),
                    Some(execution.provider_completed_unix_ms),
                )?;
                if lane.host == "claude_code" {
                    let cost = execution.provider_reported_cost_microusd.ok_or_else(|| {
                        EvalError::Invalid(
                            "Claude control omitted provider-reported cost".to_string(),
                        )
                    })?;
                    claude_cost_microusd =
                        claude_cost_microusd.checked_add(cost).ok_or_else(|| {
                            EvalError::Invalid("Claude control cost overflow".to_string())
                        })?;
                }
                executions.push(execution);
            }
            Err(failure) => {
                let (error, stage, pre_dispatch_receipt_sha256, quarantine) = failure.into_parts();
                let error = reconcile_pre_provider_receipt_failure(
                    error,
                    pre_dispatch_receipt_sha256.as_deref(),
                    pre_provider_receipt_evidence.as_ref(),
                );
                let seal_result = match stage {
                    ProviderLaneFailureStage::ConfigArtifactPreDispatch => bundle
                        .seal_pre_dispatch_rejection(
                            lane.admission_ordinal,
                            &admission_sha256,
                            failure_seal_identity.argv_sha256.clone(),
                            failure_seal_identity.environment_sha256.clone(),
                            pre_provider_receipt_evidence.as_ref(),
                            &error,
                        ),
                    ProviderLaneFailureStage::RunnerPreDispatch => bundle
                        .seal_pre_dispatch_runner_failure(
                            lane.admission_ordinal,
                            &admission_sha256,
                            failure_seal_identity.argv_sha256.clone(),
                            failure_seal_identity.environment_sha256.clone(),
                            pre_provider_receipt_evidence.as_ref(),
                            &error,
                        ),
                    ProviderLaneFailureStage::PostDispatch => bundle.seal_ambiguous(
                        lane.admission_ordinal,
                        &admission_sha256,
                        (
                            &failure_seal_identity.argv_sha256,
                            &failure_seal_identity.environment_sha256,
                        ),
                        (
                            &failure_seal_identity.trace_path,
                            &failure_seal_identity.stderr_path,
                        ),
                        pre_provider_receipt_evidence.as_ref(),
                        "control_provider_or_terminal_evidence_failed",
                        &error,
                    ),
                    ProviderLaneFailureStage::PostSpawnOutputSetup {
                        process_cleanup_proven,
                    } => bundle.seal_post_spawn_output_setup_failure(
                        lane.admission_ordinal,
                        &admission_sha256,
                        (
                            &failure_seal_identity.argv_sha256,
                            &failure_seal_identity.environment_sha256,
                        ),
                        (
                            &failure_seal_identity.trace_path,
                            &failure_seal_identity.stderr_path,
                        ),
                        pre_provider_receipt_evidence.as_ref(),
                        process_cleanup_proven,
                        &error,
                    ),
                };
                // An unproven-cleanup guard remains owned through the durable bundle freeze.
                // Dropping it only after sealing makes one final deadline-bounded cleanup attempt.
                drop(quarantine);
                if let Err(seal_error) = seal_result {
                    return Err(EvalError::Invalid(format!(
                        "control admission failed and durable ambiguous sealing also failed; original_failure_sha256={}; sealing_error={seal_error}",
                        sha256_bytes(error.to_string().as_bytes())
                    )));
                }
                return Err(error);
            }
        }
    }
    auth_lifecycle.validate_current_destinations(&treatment_plan)?;
    let report = NativeInstructionsControlRunReport {
        schema_version: 1,
        control_id: control_plan.control_id,
        run_plan: control_run_plan.display().to_string(),
        bundle: provider_bundle.canonicalize()?.display().to_string(),
        confirmed_claude_budget_cents: 10,
        started_unix_ms,
        completed_unix_ms: unix_ms(SystemTime::now())?,
        lanes: executions,
    };
    seal_control_run_report(&control_run_plan, &report)?;
    Ok(report)
}

fn execute_control_lane(
    plan: &crate::native_instructions_control::PreparedNativeInstructionsControl,
    lane: &PreparedInstructionsControlLane,
    source: &PreparedNativeLane,
    paired_treatment: &NativePilotLaneExecution,
    host_evidence: (
        Option<&CodexSessionPreDispatchInventory>,
        Option<&mut NativeInstructionsControlClaudeArtifactGuard>,
    ),
    dynamic_forbidden_trace_needles: &[String],
    pre_provider_receipt_evidence: &mut Option<NativeProviderBundlePreProviderReceiptEvidence>,
    revalidate_bundle_dispatch_boundary: impl FnOnce() -> EvalResult<()>,
) -> Result<(NativeInstructionsControlExecution, String), ProviderLaneFailure> {
    let (codex_inventory, claude_artifact_guard) = host_evidence;
    let mut claude_artifact_guard = claude_artifact_guard;
    let expected_claude_artifacts = match (lane.host.as_str(), claude_artifact_guard.as_deref_mut())
    {
        ("claude_code", Some(guard)) => Some(
            guard
                .revalidate()
                .map_err(ProviderLaneFailure::pre_dispatch_config_artifact_drift)?
                .clone(),
        ),
        ("claude_code", None) => {
            return Err(ProviderLaneFailure::pre_dispatch_config_artifact_drift(
                EvalError::Invalid(
                    "Claude control omitted its phase-spanning config artifact guard".to_string(),
                ),
            ))
        }
        (_, Some(_)) => {
            return Err(ProviderLaneFailure::pre_dispatch_config_artifact_drift(
                EvalError::Invalid(
                    "non-Claude control received a Claude config artifact guard".to_string(),
                ),
            ))
        }
        (_, None) => None,
    };
    validate_control_native_artifact_absence(source)
        .map_err(ProviderLaneFailure::pre_dispatch_runner_failure)?;
    for path in [
        &lane.trace_path,
        &lane.stderr_path,
        &lane.agent_output_path,
        &lane.effective_config_receipt_path,
        &lane.terminal_receipt_path,
    ] {
        require_absent(Path::new(path), "control lane output")
            .map_err(ProviderLaneFailure::pre_dispatch_runner_failure)?;
    }
    let effective_config_receipt_sha256 =
        match seal_control_effective_config(plan, lane, claude_artifact_guard.as_deref_mut()) {
            Ok(sha256) => sha256,
            Err(error) => {
                let artifact_drifted = claude_artifact_guard
                    .as_deref_mut()
                    .is_some_and(|guard| guard.revalidate().is_err());
                return Err(if artifact_drifted {
                    ProviderLaneFailure::pre_dispatch_config_artifact_drift(error)
                } else {
                    ProviderLaneFailure::pre_dispatch_runner_failure(error)
                });
            }
        };
    let pre_dispatch_receipt_sha256 = Some(effective_config_receipt_sha256.clone());
    *pre_provider_receipt_evidence = Some(
        validated_control_bundle_pre_provider_receipt_evidence(
            plan,
            lane,
            &effective_config_receipt_sha256,
        )
        .map_err(ProviderLaneFailure::pre_dispatch_runner_failure)
        .map_err(|failure| {
            failure.with_pre_dispatch_receipt_sha256(pre_dispatch_receipt_sha256.clone())
        })?,
    );
    if let (Some(expected), Some(guard)) = (
        expected_claude_artifacts.as_ref(),
        claude_artifact_guard.as_deref_mut(),
    ) {
        if guard
            .revalidate()
            .map_err(ProviderLaneFailure::pre_dispatch_config_artifact_drift)
            .map_err(|failure| {
                failure.with_pre_dispatch_receipt_sha256(pre_dispatch_receipt_sha256.clone())
            })?
            != expected
        {
            return Err(ProviderLaneFailure::pre_dispatch_config_artifact_drift(
                EvalError::Invalid(
                    "Claude control config artifact binding changed before provider spawn"
                        .to_string(),
                ),
            )
            .with_pre_dispatch_receipt_sha256(pre_dispatch_receipt_sha256));
        }
    }
    let provider_started_unix_ms = unix_ms(SystemTime::now())
        .map_err(ProviderLaneFailure::pre_dispatch_runner_failure)
        .map_err(|failure| {
            failure.with_pre_dispatch_receipt_sha256(pre_dispatch_receipt_sha256.clone())
        })?;
    let outcome = run_control_provider_command(
        lane,
        plan.treatment_protocol_schema_version,
        revalidate_bundle_dispatch_boundary,
    )
    .map_err(|failure| {
        failure.with_pre_dispatch_receipt_sha256(pre_dispatch_receipt_sha256.clone())
    })?;
    let provider_completed_unix_ms = unix_ms(SystemTime::now())
        .map_err(ProviderLaneFailure::post_dispatch_failure)
        .map_err(|failure| {
            failure.with_pre_dispatch_receipt_sha256(pre_dispatch_receipt_sha256.clone())
        })?;
    let post_provider = (|| -> EvalResult<(NativeInstructionsControlExecution, String)> {
        validate_native_pilot_family_v3_runtime_libraries(&plan.binaries, &plan.runtime_libraries)?;
        if let (Some(expected), Some(guard)) = (
            expected_claude_artifacts.as_ref(),
            claude_artifact_guard.as_mut(),
        ) {
            if guard.revalidate()? != expected {
                return Err(EvalError::Invalid(
                    "Claude control config artifact binding changed during provider execution"
                        .to_string(),
                ));
            }
        }
        extract_control_agent_output(
            &lane.host,
            Path::new(&lane.trace_path),
            Path::new(&lane.agent_output_path),
            plan.treatment_protocol_schema_version,
        )?;
        complete_forbidden_trace_scan(
            lane,
            Path::new(&lane.trace_path),
            dynamic_forbidden_trace_needles,
        )?;
        validate_control_native_artifact_absence(source)?;

        let trace_sha256 = sha256_file(Path::new(&lane.trace_path))?;
        let repository_instruction_sha256 = lane
            .repository_instructions
            .first()
            .ok_or_else(|| EvalError::Invalid("control instructions are absent".to_string()))?
            .sha256
            .clone();
        let mut codex_rollout = None;
        let mut claude_model = None;
        if lane.host == "codex" {
            let observed = attest_codex_session_rollout(
                source,
                NativePilotRunPhase::Evaluation,
                Path::new(&lane.trace_path),
                codex_inventory.ok_or_else(|| {
                    EvalError::Invalid(
                        "Codex control omitted pre-dispatch rollout inventory".to_string(),
                    )
                })?,
            )?;
            complete_codex_rollout_forbidden_scan(
                lane,
                Path::new(&observed.path),
                dynamic_forbidden_trace_needles,
            )?;
            if paired_treatment.codex_host_resolved_model.as_deref()
                != Some(observed.host_resolved_model.as_str())
                || paired_treatment.codex_agents_md_sha256.as_deref()
                    != Some(observed.agents_md_sha256.as_str())
            {
                return Err(EvalError::Invalid(
                "Codex control and paired treatment differ in model or loaded-instruction prefix evidence"
                    .to_string(),
            ));
            }
            codex_rollout = Some(observed);
        } else {
            let observed =
                attest_claude_trace_model(Path::new(&lane.trace_path), &plan.claude_model)?;
            if paired_treatment.claude_host_resolved_model.as_deref()
                != Some(observed.host_resolved_model.as_str())
            {
                return Err(EvalError::Invalid(
                    "Claude control and paired treatment resolved different model identifiers"
                        .to_string(),
                ));
            }
            claude_model = Some(observed);
        }
        let loaded_instruction_evidence_sha256 = if let Some(observed) = codex_rollout.as_ref() {
            sha256_bytes(&serde_json::to_vec(&serde_json::json!({
                "repository_instruction_sha256": repository_instruction_sha256,
                "rollout_sha256": observed.sha256,
                "agents_md_sha256": observed.agents_md_sha256,
                "directory": lane.evaluation_cwd,
            }))?)
        } else {
            let observed = claude_model.as_ref().expect("Claude model was attested");
            sha256_bytes(&serde_json::to_vec(&serde_json::json!({
                "repository_instruction_sha256": repository_instruction_sha256,
                "argv_sha256": lane.argv_sha256,
                "trace_sha256": observed.trace_sha256,
                "requested_model": observed.requested_model,
                "host_resolved_model": observed.host_resolved_model,
            }))?)
        };
        let execution = NativeInstructionsControlExecution {
            admission_ordinal: lane.admission_ordinal,
            source_treatment_lane_order: lane.source_treatment_lane_order,
            host: lane.host.clone(),
            case_id: lane.case_id.clone(),
            repetition: lane.repetition,
            arm: lane.arm.clone(),
            trace_path: lane.trace_path.clone(),
            trace_sha256,
            stderr_path: lane.stderr_path.clone(),
            stderr_sha256: sha256_file(Path::new(&lane.stderr_path))?,
            agent_output_path: lane.agent_output_path.clone(),
            agent_output_sha256: sha256_file(Path::new(&lane.agent_output_path))?,
            effective_config_receipt_path: lane.effective_config_receipt_path.clone(),
            effective_config_receipt_sha256,
            argv_sha256: argv_sha256(&lane.evaluation_argv)?,
            environment_sha256: canonical_environment_sha256(&lane.environment)?,
            forbidden_trace_needles_sha256: sha256_bytes(&serde_json::to_vec(
                dynamic_forbidden_trace_needles,
            )?),
            repository_instruction_sha256: repository_instruction_sha256.clone(),
            loaded_instruction_evidence_sha256,
            provider_started_unix_ms,
            provider_completed_unix_ms,
            exit_code: outcome.exit_code,
            stdout_trace_bytes: outcome
                .safety
                .expect("control provider always uses bounded execution")
                .stdout_trace_bytes,
            stderr_bytes: outcome
                .safety
                .expect("control provider always uses bounded execution")
                .stderr_bytes,
            process_cleanup_proven: outcome
                .safety
                .expect("control provider always uses bounded execution")
                .process_cleanup_proven,
            native_memory_artifact_absence_before_dispatch_proven: true,
            native_memory_artifact_absence_after_dispatch_proven: true,
            limit_triggered: false,
            provider_reported_cost_microusd: outcome.provider_reported_cost_microusd,
            accepted_turn_boundary_budget_exit: outcome.accepted_turn_boundary_budget_exit,
            codex_session_rollout_path: codex_rollout.as_ref().map(|value| value.path.clone()),
            codex_session_rollout_sha256: codex_rollout.as_ref().map(|value| value.sha256.clone()),
            codex_model_provider: codex_rollout
                .as_ref()
                .map(|value| value.model_provider.clone()),
            codex_host_resolved_model: codex_rollout
                .as_ref()
                .map(|value| value.host_resolved_model.clone()),
            codex_agents_md_sha256: codex_rollout
                .as_ref()
                .map(|value| value.agents_md_sha256.clone()),
            claude_requested_model: claude_model
                .as_ref()
                .map(|value| value.requested_model.clone()),
            claude_host_resolved_model: claude_model
                .as_ref()
                .map(|value| value.host_resolved_model.clone()),
        };
        let terminal_sha256 = seal_control_lane_terminal(lane, &execution)?;
        Ok((execution, terminal_sha256))
    })();
    post_provider
        .map_err(|error| {
            ProviderLaneFailure::post_dispatch_failure(bind_post_dispatch_failure(
                error,
                outcome
                    .safety
                    .expect("control provider always uses bounded execution"),
            ))
        })
        .map_err(|failure| failure.with_pre_dispatch_receipt_sha256(pre_dispatch_receipt_sha256))
}

pub(crate) fn validate_control_host_evidence(
    lane: &PreparedInstructionsControlLane,
    source: &PreparedNativeLane,
    execution: &NativeInstructionsControlExecution,
    protocol_schema_version: u32,
    claude_model: &str,
    dynamic_forbidden_trace_needles: &[String],
) -> EvalResult<()> {
    let trace = Path::new(&execution.trace_path);
    let stderr = Path::new(&execution.stderr_path);
    let agent_output = Path::new(&execution.agent_output_path);
    validate_private_control_artifact(trace, PROVIDER_STDOUT_LIMIT_BYTES, false)?;
    validate_private_control_artifact(stderr, PROVIDER_STDERR_LIMIT_BYTES, true)?;
    validate_private_control_artifact(agent_output, CONTROL_AGENT_OUTPUT_LIMIT_BYTES, false)?;
    if execution.trace_path != lane.trace_path
        || execution.stderr_path != lane.stderr_path
        || execution.agent_output_path != lane.agent_output_path
        || execution.effective_config_receipt_path != lane.effective_config_receipt_path
        || execution.argv_sha256 != argv_sha256(&lane.evaluation_argv)?
        || execution.environment_sha256 != canonical_environment_sha256(&lane.environment)?
        || execution.forbidden_trace_needles_sha256
            != sha256_bytes(&serde_json::to_vec(dynamic_forbidden_trace_needles)?)
        || execution.trace_sha256 != sha256_file(trace)?
        || execution.stderr_sha256 != sha256_file(stderr)?
        || execution.agent_output_sha256 != sha256_file(agent_output)?
        || execution.stdout_trace_bytes != fs::metadata(trace)?.len()
        || execution.stderr_bytes != fs::metadata(stderr)?.len()
        || execution.stdout_trace_bytes > PROVIDER_STDOUT_LIMIT_BYTES
        || execution.stderr_bytes > PROVIDER_STDERR_LIMIT_BYTES
        || !execution.process_cleanup_proven
        || !execution.native_memory_artifact_absence_before_dispatch_proven
        || !execution.native_memory_artifact_absence_after_dispatch_proven
        || execution.limit_triggered
        || execution.provider_started_unix_ms == 0
        || execution.provider_completed_unix_ms < execution.provider_started_unix_ms
    {
        return Err(EvalError::Invalid(
            "control terminal path, digest, environment, timing, or safety evidence drifted"
                .to_string(),
        ));
    }
    let expected_instruction = lane
        .repository_instructions
        .first()
        .ok_or_else(|| EvalError::Invalid("control instructions are absent".to_string()))?
        .sha256
        .clone();
    if execution.repository_instruction_sha256 != expected_instruction {
        return Err(EvalError::Invalid(
            "control repository instruction receipt drifted".to_string(),
        ));
    }
    let expected_loaded_digest = if lane.host == "codex" {
        let recorded = CodexSessionRolloutAttestation {
            path: execution
                .codex_session_rollout_path
                .clone()
                .ok_or_else(|| {
                    EvalError::Invalid("Codex control omitted rollout path".to_string())
                })?,
            sha256: execution
                .codex_session_rollout_sha256
                .clone()
                .ok_or_else(|| {
                    EvalError::Invalid("Codex control omitted rollout digest".to_string())
                })?,
            model_provider: execution.codex_model_provider.clone().ok_or_else(|| {
                EvalError::Invalid("Codex control omitted model provider".to_string())
            })?,
            host_resolved_model: execution.codex_host_resolved_model.clone().ok_or_else(|| {
                EvalError::Invalid("Codex control omitted host-resolved model".to_string())
            })?,
            agents_md_sha256: execution.codex_agents_md_sha256.clone().ok_or_else(|| {
                EvalError::Invalid("Codex control omitted agents_md digest".to_string())
            })?,
        };
        validate_recorded_codex_session_rollout(
            source,
            NativePilotRunPhase::Evaluation,
            trace,
            &recorded,
        )?;
        complete_codex_rollout_forbidden_scan(
            lane,
            Path::new(&recorded.path),
            dynamic_forbidden_trace_needles,
        )?;
        sha256_bytes(&serde_json::to_vec(&serde_json::json!({
            "repository_instruction_sha256": expected_instruction,
            "rollout_sha256": recorded.sha256,
            "agents_md_sha256": recorded.agents_md_sha256,
            "directory": lane.evaluation_cwd,
        }))?)
    } else {
        if execution.codex_session_rollout_path.is_some()
            || execution.codex_session_rollout_sha256.is_some()
            || execution.codex_model_provider.is_some()
            || execution.codex_host_resolved_model.is_some()
            || execution.codex_agents_md_sha256.is_some()
        {
            return Err(EvalError::Invalid(
                "Claude control contains Codex rollout evidence".to_string(),
            ));
        }
        let observed = attest_claude_trace_model(trace, claude_model)?;
        if execution.claude_requested_model.as_deref() != Some(observed.requested_model.as_str())
            || execution.claude_host_resolved_model.as_deref()
                != Some(observed.host_resolved_model.as_str())
        {
            return Err(EvalError::Invalid(
                "Claude control model evidence drifted".to_string(),
            ));
        }
        sha256_bytes(&serde_json::to_vec(&serde_json::json!({
            "repository_instruction_sha256": expected_instruction,
            "argv_sha256": lane.argv_sha256,
            "trace_sha256": observed.trace_sha256,
            "requested_model": observed.requested_model,
            "host_resolved_model": observed.host_resolved_model,
        }))?)
    };
    if execution.loaded_instruction_evidence_sha256 != expected_loaded_digest {
        return Err(EvalError::Invalid(
            "control loaded-instruction evidence digest drifted".to_string(),
        ));
    }
    match lane.host.as_str() {
        "codex" => {
            if execution.exit_code != 0
                || execution.provider_reported_cost_microusd.is_some()
                || execution.accepted_turn_boundary_budget_exit
                || execution.claude_requested_model.is_some()
                || execution.claude_host_resolved_model.is_some()
            {
                return Err(EvalError::Invalid(
                    "Codex control terminal outcome is invalid".to_string(),
                ));
            }
        }
        "claude_code" => {
            let summary = claude_trace_summary(trace)?;
            let reported_cost = execution.provider_reported_cost_microusd.ok_or_else(|| {
                EvalError::Invalid("Claude control omitted provider-reported cost".to_string())
            })?;
            if summary.provider_reported_cost_microusd != Some(reported_cost)
                || summary.accepted_turn_boundary_budget_exit
                    != execution.accepted_turn_boundary_budget_exit
            {
                return Err(EvalError::Invalid(
                    "Claude control cost or budget evidence drifted".to_string(),
                ));
            }
            let terminal = unique_claude_terminal_result(trace)?;
            if execution.accepted_turn_boundary_budget_exit {
                if execution.exit_code == 0
                    || !exact_control_budget_exhaustion(&terminal)
                    || (terminal.get("stop_reason").and_then(Value::as_str) == Some("tool_use")
                        && unique_agent_output_with_contract(
                            trace,
                            protocol_schema_version,
                            NativeAgentOutputContract::StaleSafetyV3,
                        )
                        .is_err())
                {
                    return Err(EvalError::Invalid(
                        "Claude control budget terminal is invalid".to_string(),
                    ));
                }
            } else if execution.exit_code != 0
                || terminal.get("subtype").and_then(Value::as_str) != Some("success")
                || terminal.get("is_error").and_then(Value::as_bool) != Some(false)
                || terminal.get("stop_reason").and_then(Value::as_str) != Some("end_turn")
            {
                return Err(EvalError::Invalid(
                    "Claude control successful terminal is invalid".to_string(),
                ));
            }
        }
        _ => {
            return Err(EvalError::Invalid(
                "control terminal has unsupported host".to_string(),
            ))
        }
    }
    Ok(())
}

fn validate_control_native_artifact_absence(source: &PreparedNativeLane) -> EvalResult<()> {
    let lane_root = Path::new(&source.acceptance_contract)
        .parent()
        .ok_or_else(|| EvalError::Invalid("treatment lane has no root".to_string()))?;
    let expected = match source.host.as_str() {
        "codex" => lane_root.join("codex-home/memories"),
        "claude_code" => lane_root.join("claude-memory/MEMORY.md"),
        _ => {
            return Err(EvalError::Invalid(
                "control source has unsupported host".to_string(),
            ))
        }
    };
    match fs::symlink_metadata(&expected) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Ok(_) => Err(EvalError::Invalid(format!(
            "instructions control observed a native-memory artifact: {}",
            expected.display()
        ))),
        Err(error) => Err(EvalError::Io(error)),
    }
}

fn validate_private_control_artifact(path: &Path, limit: u64, allow_empty: bool) -> EvalResult<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let parent = path.parent().ok_or_else(|| {
            EvalError::Invalid("control artifact has no private parent".to_string())
        })?;
        let (_, owner) = open_private_directory(parent)?;
        let metadata = fs::symlink_metadata(path)?;
        if metadata.file_type().is_symlink()
            || !metadata.is_file()
            || metadata.uid() != owner
            || metadata.mode() & 0o777 != 0o600
            || metadata.nlink() != 1
            || (!allow_empty && metadata.len() == 0)
            || metadata.len() > limit
        {
            return Err(EvalError::Invalid(format!(
                "control artifact is not a bounded owner-only single-link regular file: {}",
                path.display()
            )));
        }
        Ok(())
    }
    #[cfg(not(unix))]
    {
        let _ = (path, limit, allow_empty);
        Err(EvalError::Invalid(
            "control artifact validation requires Unix metadata".to_string(),
        ))
    }
}

fn unique_claude_terminal_result(trace: &Path) -> EvalResult<Value> {
    let reader = BufReader::new(File::open(trace)?);
    let mut terminal = None;
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(&line)?;
        if value.get("type").and_then(Value::as_str) == Some("result")
            && terminal.replace(value).is_some()
        {
            return Err(EvalError::Invalid(
                "Claude control trace contains multiple terminal results".to_string(),
            ));
        }
    }
    terminal.ok_or_else(|| {
        EvalError::Invalid(format!(
            "Claude control trace has no terminal result: {}",
            trace.display()
        ))
    })
}

fn exact_control_budget_exhaustion(result: &Value) -> bool {
    result.get("type").and_then(Value::as_str) == Some("result")
        && result.get("subtype").and_then(Value::as_str) == Some("error_max_budget_usd")
        && result.get("is_error").and_then(Value::as_bool) == Some(true)
        && result.get("terminal_reason").and_then(Value::as_str) == Some("budget_exhausted")
        && matches!(
            result.get("stop_reason").and_then(Value::as_str),
            Some("end_turn" | "tool_use")
        )
        && result
            .get("errors")
            .and_then(Value::as_array)
            .is_some_and(|errors| {
                errors.len() == 1 && errors[0].as_str() == Some("Reached maximum budget ($0.041)")
            })
}

fn run_control_provider_command(
    lane: &PreparedInstructionsControlLane,
    protocol_schema_version: u32,
    revalidate_bundle_dispatch_boundary: impl FnOnce() -> EvalResult<()>,
) -> Result<ProviderCommandOutcome, ProviderLaneFailure> {
    let (exit_code, safety) = run_control_command_status(lane, revalidate_bundle_dispatch_boundary)
        .map_err(BoundedProviderCommandFailure::into_lane_failure)?;
    let result = (|| -> EvalResult<ProviderCommandOutcome> {
        let summary = if lane.host == "claude_code" {
            claude_trace_summary(Path::new(&lane.trace_path))?
        } else {
            ProviderCommandOutcome {
                exit_code,
                provider_reported_cost_microusd: None,
                accepted_turn_boundary_budget_exit: false,
                safety: Some(safety),
            }
        };
        if exit_code == 0 {
            return Ok(ProviderCommandOutcome {
                exit_code,
                provider_reported_cost_microusd: summary.provider_reported_cost_microusd,
                accepted_turn_boundary_budget_exit: false,
                safety: Some(safety),
            });
        }
        if lane.host == "claude_code" && summary.accepted_turn_boundary_budget_exit {
            return Ok(ProviderCommandOutcome {
                exit_code,
                safety: Some(safety),
                ..summary
            });
        }
        if lane.host == "claude_code" {
            let terminal = claude_terminal_result(Path::new(&lane.trace_path))?;
            if is_structured_output_budget_exhaustion(&terminal) {
                unique_agent_output_with_contract(
                    Path::new(&lane.trace_path),
                    protocol_schema_version,
                    NativeAgentOutputContract::StaleSafetyV3,
                )?;
                return Ok(ProviderCommandOutcome {
                    exit_code,
                    accepted_turn_boundary_budget_exit: true,
                    safety: Some(safety),
                    ..summary
                });
            }
        }
        Err(EvalError::Invalid(format!(
            "control provider exited with status {exit_code}; stderr is {}",
            lane.stderr_path
        )))
    })();
    result.map_err(|error| {
        ProviderLaneFailure::post_dispatch_failure(bind_post_dispatch_failure(error, safety))
    })
}

fn run_control_command_status(
    lane: &PreparedInstructionsControlLane,
    revalidate_bundle_dispatch_boundary: impl FnOnce() -> EvalResult<()>,
) -> Result<(i32, ProviderSafetyEvidence), BoundedProviderCommandFailure> {
    run_bounded_provider_command_status(
        &lane.evaluation_argv,
        Path::new(&lane.evaluation_cwd),
        &lane.environment,
        &lane.environment_remove,
        Path::new(&lane.trace_path),
        Path::new(&lane.stderr_path),
        revalidate_bundle_dispatch_boundary,
    )
}

/// Execute a native-memory phase under the frozen 34-admission parent bundle when family-v3 is
/// selected. Historical and family-v1 plans reject an unrelated bundle and retain their existing
/// execution path.
pub fn run_native_memory_pilot_with_bundle(
    run_plan: &Path,
    phase: NativePilotRunPhase,
    approval: NativePilotExecutionApproval,
    provider_bundle: Option<&Path>,
) -> EvalResult<NativePilotRunReport> {
    let plan = load_historical_native_plan(run_plan)?;
    validate_execution_approval(&plan, approval)?;
    validate_plan_digest(run_plan)?;
    validate_native_stale_plan_binding(&plan, run_plan)?;
    validate_evaluation_recovery(&plan, run_plan)?;
    validate_execution_recovery(&plan)?;
    validate_plan_attestations(&plan)?;
    validate_required_secrets(&plan)?;
    validate_execution_phase_scope(&plan, phase)?;

    let run_plan = run_plan.canonicalize()?;
    let family_v3 = plan
        .stale_safety
        .as_ref()
        .is_some_and(|binding| binding.family_version == CODEX_AUTH_CACHE_LIFECYCLE_FAMILY_VERSION);
    let provider_bundle = match (family_v3, provider_bundle) {
        (true, Some(bundle)) => Some(NativeProviderBundleJournal::open_treatment(
            bundle, &run_plan,
        )?),
        (true, None) => {
            return Err(EvalError::Invalid(
                "family-v3 provider execution requires the pre-existing 34-admission parent bundle intent"
                    .to_string(),
            ))
        }
        (false, Some(_)) => {
            return Err(EvalError::Invalid(
                "historical native-memory plans do not accept a family-v3 provider bundle"
                    .to_string(),
            ))
        }
        (false, None) => None,
    };
    let mut auth_lifecycle = if requires_exact_six_codex_auth_lifecycle(&plan)? {
        let lifecycle = CodexAuthCacheLifecycle::open(&plan, &run_plan)?;
        lifecycle.validate_current_destinations(&plan)?;
        Some(lifecycle)
    } else {
        None
    };
    if plan.stale_safety.is_some() {
        validate_stale_lifecycle_residue_before_phase(&plan, &run_plan, phase)?;
    } else {
        require_absent(&phase_report_path(&run_plan, phase)?, "phase report")?;
    }
    validate_stale_shared_retention_precondition_at(
        &plan,
        &run_plan,
        phase,
        unix_ms(SystemTime::now())?,
    )?;
    let report_path = phase_report_path(&run_plan, phase)?;
    let before = audit_native_memory_pilot(&run_plan)?;
    validate_phase_preconditions(&plan, &before, phase)?;
    validate_phase_targets_absent(&plan, &before, phase)?;
    validate_engram_disk_headroom(&plan, &before, phase)?;
    require_codex_authentication_ready(&plan, &run_plan, auth_lifecycle.is_some())?;
    if let Some(lifecycle) = auth_lifecycle.as_ref() {
        lifecycle.validate_current_destinations(&plan)?;
    }
    let (
        family_v3_claude_config_artifacts,
        started_unix_ms,
        stale_safety_pre_evaluation_marker_snapshot_sha256,
    ) = with_family_v3_claude_config_preflight(&plan, |config_artifacts| {
        let started_unix_ms = unix_ms(SystemTime::now())?;
        let marker_snapshot_sha256 = if phase == NativePilotRunPhase::Evaluation {
            materialize_native_stale_evaluation_marker_snapshot(&plan, &run_plan, started_unix_ms)?
        } else {
            None
        };
        if let Some(lifecycle) = auth_lifecycle.as_mut() {
            lifecycle.begin_phase(&plan, &run_plan, phase)?;
        }
        Ok((config_artifacts, started_unix_ms, marker_snapshot_sha256))
    })?;
    let mut lanes = Vec::new();
    for lane in &plan.lanes {
        if phase == NativePilotRunPhase::Activation && lane.host != "codex" {
            continue;
        }
        let current = before
            .lanes
            .iter()
            .find(|current| current.order == lane.order)
            .ok_or_else(|| {
                EvalError::Invalid(format!("pilot audit omitted lane {}", lane.order))
            })?;
        match lane_run_action(&plan, lane, current.phase, phase)? {
            LaneRunAction::Execute => {
                if let Some(lifecycle) = auth_lifecycle.as_ref() {
                    let ordinal = treatment_admission_ordinal(&plan, phase, lane.order)?;
                    let prior_spend =
                        lifecycle.prior_claude_spend_microusd(&plan, &run_plan, ordinal)?;
                    let allocation = u64::from(plan.claude_budget_cents) * 10_000;
                    if prior_spend >= allocation {
                        return Err(EvalError::Invalid(format!(
                            "family-v3 treatment Claude aggregate allocation was reached before provider admission {ordinal}; prior receipt-bound spend is {prior_spend} micro-USD"
                        )));
                    }
                }
                let claude_config_artifacts = if family_v3 {
                    revalidate_family_v3_lane_claude_config_artifacts(
                        lane,
                        &family_v3_claude_config_artifacts,
                    )?
                } else {
                    None
                };
                let bundle_admission = if let Some(bundle) = provider_bundle.as_ref() {
                    let trace = trace_path(lane, phase)?.to_path_buf();
                    let failure_seal_identity = provider_failure_seal_identity(
                        lane_phase_argv(lane, phase)?,
                        &lane.environment,
                        trace.clone(),
                        stderr_path(&trace),
                    )?;
                    let ordinal = treatment_admission_ordinal(&plan, phase, lane.order)?;
                    let predecessors = auth_lifecycle
                        .as_ref()
                        .ok_or_else(|| {
                            EvalError::Invalid(
                                "family-v3 provider bundle admission omitted its active lifecycle"
                                    .to_string(),
                            )
                        })?
                        .validated_treatment_predecessor_evidence(&plan, &run_plan, ordinal)?;
                    let requested = NativeProviderBundleAdmission {
                        ordinal,
                        child: "treatment".to_string(),
                        phase: phase.label().to_string(),
                        lane_order: lane.order,
                        host: lane.host.clone(),
                        case_id: lane.case_id.clone(),
                        repetition: lane.repetition,
                        arm: lane.arm.clone(),
                    };
                    let digest = bundle.admit(&requested, &predecessors)?;
                    Some((ordinal, digest, predecessors, failure_seal_identity))
                } else {
                    None
                };
                let mut pre_provider_receipt_evidence = None;
                let lane_result =
                    (|| -> Result<(NativePilotLaneExecution, Option<String>), ProviderLaneFailure> {
                    let pre_dispatch_receipt_sha256 = if let Some(lifecycle) = auth_lifecycle.as_ref()
                    {
                        let receipt_sha256 = lifecycle.admit_lane(
                            &plan,
                            &run_plan,
                            lane,
                            phase,
                            claude_config_artifacts,
                        )?;
                        let admission_ordinal = bundle_admission
                            .as_ref()
                            .map(|(ordinal, _, _, _)| *ordinal)
                            .ok_or_else(|| {
                                ProviderLaneFailure::pre_dispatch_runner_failure(
                                    EvalError::Invalid(
                                        "family-v3 lifecycle admission omitted its provider bundle admission"
                                            .to_string(),
                                    ),
                                )
                            })?;
                        let evidence = lifecycle
                            .validated_bundle_pre_provider_receipt_evidence(
                                &plan,
                                &run_plan,
                                lane,
                                phase,
                                admission_ordinal,
                                &receipt_sha256,
                            )
                            .map_err(ProviderLaneFailure::pre_dispatch_runner_failure)
                            .map_err(|failure| {
                                failure.with_pre_dispatch_receipt_sha256(Some(
                                    receipt_sha256.clone(),
                                ))
                            })?;
                        pre_provider_receipt_evidence = Some(evidence);
                        Some(receipt_sha256)
                    } else {
                        None
                    };
                    let mut execution = execute_lane(
                        &plan,
                        lane,
                        phase,
                        auth_lifecycle.as_ref(),
                        claude_config_artifacts,
                        || match (provider_bundle.as_ref(), bundle_admission.as_ref()) {
                            (
                                Some(bundle),
                                Some((ordinal, admission_sha256, predecessors, _)),
                            ) => {
                                validate_native_pilot_family_v3_runtime_libraries(
                                    &plan.binaries,
                                    &plan.runtime_libraries,
                                )?;
                                bundle.revalidate_dispatch_boundary(
                                    *ordinal,
                                    admission_sha256,
                                    predecessors,
                                )
                            }
                            (None, None) if family_v3 => Err(EvalError::Invalid(
                                "family-v3 provider dispatch omitted its live bundle journal"
                                    .to_string(),
                            )),
                            (None, None) => Ok(()),
                            _ => Err(EvalError::Invalid(
                                "provider bundle admission state is internally inconsistent"
                                    .to_string(),
                            )),
                        },
                    )
                    .map_err(|failure| {
                        failure.with_pre_dispatch_receipt_sha256(
                            pre_dispatch_receipt_sha256.clone(),
                        )
                    })?;
                    let lifecycle_terminal_sha256 = if let Some(lifecycle) = auth_lifecycle.as_ref()
                    {
                        let digest = lifecycle
                            .seal_lane(&plan, &run_plan, &mut execution, phase)
                            .map_err(ProviderLaneFailure::post_dispatch_failure)
                            .map_err(|failure| {
                                failure.with_pre_dispatch_receipt_sha256(
                                    pre_dispatch_receipt_sha256.clone(),
                                )
                            })?;
                        lifecycle
                            .validate_current_destinations(&plan)
                            .map_err(ProviderLaneFailure::post_dispatch_failure)
                            .map_err(|failure| {
                                failure.with_pre_dispatch_receipt_sha256(
                                    pre_dispatch_receipt_sha256.clone(),
                                )
                            })?;
                        Some(digest)
                    } else {
                        None
                    };
                    let incremental_audit = audit_native_memory_pilot(&run_plan)
                        .map_err(ProviderLaneFailure::post_dispatch_failure)
                        .map_err(|failure| {
                            failure.with_pre_dispatch_receipt_sha256(
                                pre_dispatch_receipt_sha256.clone(),
                            )
                        })?;
                    validate_executed_lane_postcondition(&incremental_audit, lane, phase)
                        .map_err(ProviderLaneFailure::post_dispatch_failure)
                        .map_err(|failure| {
                            failure.with_pre_dispatch_receipt_sha256(
                                pre_dispatch_receipt_sha256.clone(),
                            )
                        })?;
                    Ok((execution, lifecycle_terminal_sha256))
                })();
                match lane_result {
                    Ok((execution, lifecycle_terminal_sha256)) => {
                        if let (Some(bundle), Some((ordinal, admission_sha256, _, _))) =
                            (provider_bundle.as_ref(), bundle_admission.as_ref())
                        {
                            bundle.seal(
                                *ordinal,
                                admission_sha256,
                                NativeProviderAdmissionOutcome::Terminal,
                                lifecycle_terminal_sha256,
                                None,
                                execution.provider_started_unix_ms,
                                execution.provider_completed_unix_ms,
                            )?;
                        }
                        lanes.push(execution);
                    }
                    Err(failure) => {
                        let (error, stage, pre_dispatch_receipt_sha256, quarantine) =
                            failure.into_parts();
                        let error = reconcile_pre_provider_receipt_failure(
                            error,
                            pre_dispatch_receipt_sha256.as_deref(),
                            pre_provider_receipt_evidence.as_ref(),
                        );
                        if let (
                            Some(bundle),
                            Some((ordinal, admission_sha256, _, failure_seal_identity)),
                        ) = (provider_bundle.as_ref(), bundle_admission.as_ref())
                        {
                            let seal_result = match stage {
                                ProviderLaneFailureStage::ConfigArtifactPreDispatch => bundle
                                    .seal_pre_dispatch_rejection(
                                        *ordinal,
                                        admission_sha256,
                                        failure_seal_identity.argv_sha256.clone(),
                                        failure_seal_identity.environment_sha256.clone(),
                                        pre_provider_receipt_evidence.as_ref(),
                                        &error,
                                    ),
                                ProviderLaneFailureStage::RunnerPreDispatch => bundle
                                    .seal_pre_dispatch_runner_failure(
                                        *ordinal,
                                        admission_sha256,
                                        failure_seal_identity.argv_sha256.clone(),
                                        failure_seal_identity.environment_sha256.clone(),
                                        pre_provider_receipt_evidence.as_ref(),
                                        &error,
                                    ),
                                ProviderLaneFailureStage::PostDispatch => bundle.seal_ambiguous(
                                    *ordinal,
                                    admission_sha256,
                                    (
                                        &failure_seal_identity.argv_sha256,
                                        &failure_seal_identity.environment_sha256,
                                    ),
                                    (
                                        &failure_seal_identity.trace_path,
                                        &failure_seal_identity.stderr_path,
                                    ),
                                    pre_provider_receipt_evidence.as_ref(),
                                    "provider_or_postcondition_failed",
                                    &error,
                                ),
                                ProviderLaneFailureStage::PostSpawnOutputSetup {
                                    process_cleanup_proven,
                                } => bundle.seal_post_spawn_output_setup_failure(
                                    *ordinal,
                                    admission_sha256,
                                    (
                                        &failure_seal_identity.argv_sha256,
                                        &failure_seal_identity.environment_sha256,
                                    ),
                                    (
                                        &failure_seal_identity.trace_path,
                                        &failure_seal_identity.stderr_path,
                                    ),
                                    pre_provider_receipt_evidence.as_ref(),
                                    process_cleanup_proven,
                                    &error,
                                ),
                            };
                            // Keep any unproven-cleanup child guard alive through the durable
                            // freeze, then let Drop make its final bounded cleanup attempt.
                            drop(quarantine);
                            if let Err(seal_error) = seal_result {
                                return Err(EvalError::Invalid(format!(
                                    "treatment admission failed and durable ambiguous sealing also failed; original_failure_sha256={}; sealing_error={seal_error}",
                                    sha256_bytes(error.to_string().as_bytes())
                                )));
                            }
                        }
                        return Err(error);
                    }
                }
            }
            LaneRunAction::RecoverTeaching => {
                lanes.push(recover_interrupted_teaching_lane(&plan, lane)?)
            }
            LaneRunAction::RecoverEvaluation => lanes.push(recover_interrupted_evaluation_lane(
                &plan,
                lane,
                plan.protocol_schema_version,
            )?),
            LaneRunAction::Skip => continue,
        }
        let incremental_audit = audit_native_memory_pilot(&run_plan)?;
        validate_executed_lane_postcondition(&incremental_audit, lane, phase)?;
    }

    let audit = audit_native_memory_pilot(&run_plan)?;
    validate_phase_postconditions(&audit, phase)?;
    let report = NativePilotRunReport {
        pilot_id: plan.pilot_id.clone(),
        phase,
        run_plan: run_plan.display().to_string(),
        confirmed_claude_budget_cents: plan.claude_budget_cents,
        started_unix_ms,
        completed_unix_ms: unix_ms(SystemTime::now())?,
        lanes,
        stale_safety_pre_evaluation_marker_snapshot_sha256,
        audit,
    };
    write_private_json(&report_path, &report)?;
    write_stale_phase_report_digest(&plan, &report_path)?;
    if let Some(lifecycle) = auth_lifecycle.as_mut() {
        lifecycle.complete_phase(&plan, &run_plan, phase, &report_path)?;
    }
    Ok(report)
}

fn validate_stale_lifecycle_residue_before_phase(
    plan: &PreparedNativePilot,
    run_plan: &Path,
    phase: NativePilotRunPhase,
) -> EvalResult<()> {
    let root = run_plan
        .parent()
        .ok_or_else(|| EvalError::Invalid("run plan has no parent".to_string()))?;
    let require_completed = |completed_phase| {
        read_validated_stale_phase_receipt(plan, run_plan, completed_phase).map(|_| ())
    };
    let require_phase_absent = |absent_phase: NativePilotRunPhase| -> EvalResult<()> {
        let label = absent_phase.label();
        require_absent(
            &root.join(format!("runner-{label}.json")),
            &format!("stale-safety {label} phase receipt"),
        )?;
        require_absent(
            &root.join(format!("runner-{label}.sha256")),
            &format!("stale-safety {label} phase receipt digest"),
        )
    };

    match phase {
        NativePilotRunPhase::Teaching => {
            require_phase_absent(NativePilotRunPhase::Teaching)?;
            require_phase_absent(NativePilotRunPhase::Activation)?;
            require_phase_absent(NativePilotRunPhase::Evaluation)?;
        }
        NativePilotRunPhase::Activation => {
            require_completed(NativePilotRunPhase::Teaching)?;
            require_phase_absent(NativePilotRunPhase::Activation)?;
            require_phase_absent(NativePilotRunPhase::Evaluation)?;
        }
        NativePilotRunPhase::Evaluation => {
            require_completed(NativePilotRunPhase::Teaching)?;
            require_completed(NativePilotRunPhase::Activation)?;
            require_phase_absent(NativePilotRunPhase::Evaluation)?;
        }
    }
    require_absent(
        &root.join("stale-safety.pre-evaluation-native-markers.json"),
        "stale-safety pre-evaluation native marker snapshot",
    )?;
    require_absent(
        &root.join("stale-safety.pre-evaluation-native-markers.sha256"),
        "stale-safety pre-evaluation native marker snapshot digest",
    )?;
    Ok(())
}

pub(crate) fn validate_stale_shared_retention_precondition_at(
    plan: &PreparedNativePilot,
    run_plan: &Path,
    phase: NativePilotRunPhase,
    now_unix_ms: u64,
) -> EvalResult<()> {
    let root = run_plan
        .parent()
        .ok_or_else(|| EvalError::Invalid("run plan has no parent".to_string()))?;
    if !validate_native_stale_plan_binding(plan, run_plan)? {
        return Ok(());
    }
    let binding = plan.stale_safety.as_ref().expect("validated stale binding");
    let precondition_path = root.join(&binding.retention_precondition_file);
    let digest_path = root.join("stale-safety.retention-precondition.sha256");
    let digest = fs::read_to_string(&digest_path)?;
    if digest.trim() != sha256_file(&precondition_path)? {
        return Err(EvalError::Invalid(
            "stale-safety shared retention precondition digest is invalid".to_string(),
        ));
    }
    let precondition: NativeStalePreparationPrecondition =
        serde_json::from_reader(File::open(&precondition_path)?)?;
    if precondition.schema_version != 1
        || precondition.shared_retention_gate_hours < 1
        || precondition.shared_retention_gate_ms
            != u64::from(precondition.shared_retention_gate_hours) * 60 * 60 * 1_000
        || precondition.deadline_source != "runner-teaching.json.completed_unix_ms"
        || precondition.teaching_completed_unix_ms.is_some()
        || precondition.not_before_unix_ms.is_some()
        || precondition.required_before_phases != ["activation", "evaluation"]
    {
        return Err(EvalError::Invalid(
            "stale-safety shared retention precondition metadata is invalid".to_string(),
        ));
    }
    if phase == NativePilotRunPhase::Teaching {
        return Ok(());
    }
    let teaching =
        read_validated_stale_phase_receipt(plan, run_plan, NativePilotRunPhase::Teaching)?;
    if phase == NativePilotRunPhase::Evaluation {
        read_validated_stale_phase_receipt(plan, run_plan, NativePilotRunPhase::Activation)?;
    }
    let completed = teaching.completed_unix_ms;
    let deadline = completed
        .checked_add(precondition.shared_retention_gate_ms)
        .ok_or_else(|| {
            EvalError::Invalid("stale-safety retention deadline overflow".to_string())
        })?;
    if now_unix_ms < deadline {
        return Err(EvalError::Invalid(format!(
            "stale-safety shared retention gate has not passed: now={now_unix_ms}, not_before={deadline}"
        )));
    }
    Ok(())
}

pub(crate) fn read_validated_stale_phase_receipt(
    plan: &PreparedNativePilot,
    run_plan: &Path,
    phase: NativePilotRunPhase,
) -> EvalResult<NativePilotRunReport> {
    let root = run_plan
        .parent()
        .ok_or_else(|| EvalError::Invalid("run plan has no parent".to_string()))?;
    let phase_name = phase.label();
    let receipt_path = root.join(format!("runner-{phase_name}.json"));
    let metadata = fs::symlink_metadata(&receipt_path).map_err(|error| {
        EvalError::Invalid(format!(
            "stale-safety lifecycle requires the completed {phase_name} receipt: {error}"
        ))
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(EvalError::Invalid(format!(
            "stale-safety {phase_name} receipt must be a regular non-symlink file"
        )));
    }
    let digest_path = root.join(format!("runner-{phase_name}.sha256"));
    let digest_metadata = fs::symlink_metadata(&digest_path).map_err(|error| {
        EvalError::Invalid(format!(
            "stale-safety {phase_name} receipt digest is unavailable: {error}"
        ))
    })?;
    if digest_metadata.file_type().is_symlink() || !digest_metadata.is_file() {
        return Err(EvalError::Invalid(format!(
            "stale-safety {phase_name} receipt digest must be a regular non-symlink file"
        )));
    }
    let receipt_digest = fs::read_to_string(&digest_path)?;
    if receipt_digest.trim() != sha256_file(&receipt_path)? {
        return Err(EvalError::Invalid(format!(
            "stale-safety {phase_name} receipt digest is invalid"
        )));
    }
    let receipt: NativePilotRunReport = serde_json::from_reader(File::open(&receipt_path)?)?;
    validate_stale_phase_receipt(plan, run_plan, phase, &receipt)?;
    Ok(receipt)
}

pub(crate) fn validate_stale_phase_receipt(
    plan: &PreparedNativePilot,
    run_plan: &Path,
    phase: NativePilotRunPhase,
    receipt: &NativePilotRunReport,
) -> EvalResult<()> {
    let canonical_run_plan = run_plan.canonicalize()?.display().to_string();
    let marker_snapshot = match phase {
        NativePilotRunPhase::Evaluation => {
            let digest = receipt
                .stale_safety_pre_evaluation_marker_snapshot_sha256
                .as_deref()
                .ok_or_else(|| {
                    EvalError::Invalid(
                        "stale-safety evaluation receipt omits its pre-evaluation marker snapshot"
                            .to_string(),
                    )
                })?;
            Some(validate_native_stale_evaluation_marker_snapshot(
                plan, run_plan, digest,
            )?)
        }
        NativePilotRunPhase::Teaching | NativePilotRunPhase::Activation => {
            if receipt
                .stale_safety_pre_evaluation_marker_snapshot_sha256
                .is_some()
            {
                return Err(EvalError::Invalid(format!(
                    "stale-safety {} receipt unexpectedly references an evaluation marker snapshot",
                    phase.label()
                )));
            }
            None
        }
    };
    let expected_lanes = plan
        .lanes
        .iter()
        .filter(|lane| phase != NativePilotRunPhase::Activation || lane.host == "codex")
        .collect::<Vec<_>>();
    if receipt.pilot_id != plan.pilot_id
        || receipt.phase != phase
        || receipt.run_plan != canonical_run_plan
        || receipt.started_unix_ms < plan.prepared_unix_ms
        || receipt.started_unix_ms > receipt.completed_unix_ms
        || receipt.confirmed_claude_budget_cents != plan.claude_budget_cents
        || receipt.lanes.len() != expected_lanes.len()
        || !stale_phase_receipt_audit_is_valid(plan, &canonical_run_plan, phase, &receipt.audit)
    {
        return Err(EvalError::Invalid(format!(
            "stale-safety {} receipt is not a valid frozen lifecycle receipt",
            phase.label()
        )));
    }
    let mut previous_completed = receipt.started_unix_ms;
    for (actual, expected) in receipt.lanes.iter().zip(expected_lanes) {
        if plan.stale_safety.as_ref().is_some_and(|binding| {
            binding.family_version == CODEX_AUTH_CACHE_LIFECYCLE_FAMILY_VERSION
        }) {
            validate_family_v3_lane_rollout_evidence(actual, expected, phase, &plan.claude_model)?;
        }
        if !stale_phase_lane_lifecycle_is_valid(
            actual,
            expected,
            phase,
            receipt.started_unix_ms,
            receipt.completed_unix_ms,
            previous_completed,
        )? {
            return Err(EvalError::Invalid(format!(
                "stale-safety {} receipt has invalid lifecycle evidence for lane {}",
                phase.label(),
                expected.order
            )));
        }
        previous_completed = actual.provider_completed_unix_ms.ok_or_else(|| {
            EvalError::Invalid(format!(
                "stale-safety {} receipt lane {} omits provider completion",
                phase.label(),
                expected.order
            ))
        })?;
    }
    if marker_snapshot.is_some_and(|snapshot| {
        let first_provider_start = receipt
            .lanes
            .first()
            .and_then(|lane| lane.provider_started_unix_ms)
            .unwrap_or(receipt.completed_unix_ms);
        snapshot.captured_unix_ms < receipt.started_unix_ms
            || snapshot.captured_unix_ms > first_provider_start
    }) {
        return Err(EvalError::Invalid(
            "stale-safety evaluation marker snapshot was not captured before provider execution"
                .to_string(),
        ));
    }
    Ok(())
}

fn validate_family_v3_lane_rollout_evidence(
    actual: &NativePilotLaneExecution,
    expected: &PreparedNativeLane,
    phase: NativePilotRunPhase,
    claude_model: &str,
) -> EvalResult<()> {
    let expected_environment_sha256 = canonical_environment_sha256(&expected.environment)?;
    if actual.effective_environment_sha256.as_deref() != Some(expected_environment_sha256.as_str())
    {
        return Err(EvalError::Invalid(format!(
            "family-v3 lane {} omitted or drifted its closed-world environment digest",
            expected.order
        )));
    }
    let stdout_bytes = fs::metadata(&actual.trace_path)?.len();
    let stderr_bytes = fs::metadata(&actual.stderr_path)?.len();
    if actual.stdout_trace_bytes != Some(stdout_bytes)
        || actual.stderr_bytes != Some(stderr_bytes)
        || actual.process_cleanup_proven != Some(true)
        || stdout_bytes > PROVIDER_STDOUT_LIMIT_BYTES
        || stderr_bytes > PROVIDER_STDERR_LIMIT_BYTES
    {
        return Err(EvalError::Invalid(format!(
            "family-v3 lane {} omitted or drifted its bounded process evidence",
            expected.order
        )));
    }
    let trace_digest = sha256_file(Path::new(&actual.trace_path))?;
    if actual.provider_trace_sha256.as_deref() != Some(trace_digest.as_str()) {
        return Err(EvalError::Invalid(format!(
            "family-v3 lane {} omitted or drifted its complete provider trace digest",
            expected.order
        )));
    }
    if expected.host != "codex" {
        if actual.codex_session_rollout_path.is_some()
            || actual.codex_session_rollout_sha256.is_some()
            || actual.codex_model_provider.is_some()
            || actual.codex_host_resolved_model.is_some()
            || actual.codex_agents_md_sha256.is_some()
        {
            return Err(EvalError::Invalid(format!(
                "family-v3 non-Codex lane {} contains Codex rollout evidence",
                expected.order
            )));
        }
        let attestation = attest_claude_trace_model(Path::new(&actual.trace_path), claude_model)?;
        let config_artifacts = validate_native_stale_v3_claude_config_artifacts(expected)?;
        if actual.claude_requested_model.as_deref() != Some(attestation.requested_model.as_str())
            || actual.claude_host_resolved_model.as_deref()
                != Some(attestation.host_resolved_model.as_str())
            || actual.provider_trace_sha256.as_deref() != Some(attestation.trace_sha256.as_str())
            || actual.claude_config_artifacts_sha256.as_deref()
                != Some(config_artifacts.aggregate_sha256.as_str())
        {
            return Err(EvalError::Invalid(format!(
                "family-v3 Claude lane {} omitted or drifted model/config evidence",
                expected.order
            )));
        }
        return Ok(());
    }
    if actual.claude_config_artifacts_sha256.is_some()
        || actual.claude_requested_model.is_some()
        || actual.claude_host_resolved_model.is_some()
    {
        return Err(EvalError::Invalid(format!(
            "family-v3 Codex lane {} contains Claude model evidence",
            expected.order
        )));
    }
    let recorded = CodexSessionRolloutAttestation {
        path: actual.codex_session_rollout_path.clone().ok_or_else(|| {
            EvalError::Invalid(format!(
                "family-v3 Codex lane {} omitted its session rollout path",
                expected.order
            ))
        })?,
        sha256: actual.codex_session_rollout_sha256.clone().ok_or_else(|| {
            EvalError::Invalid(format!(
                "family-v3 Codex lane {} omitted its session rollout digest",
                expected.order
            ))
        })?,
        model_provider: actual.codex_model_provider.clone().ok_or_else(|| {
            EvalError::Invalid(format!(
                "family-v3 Codex lane {} omitted its model provider",
                expected.order
            ))
        })?,
        host_resolved_model: actual.codex_host_resolved_model.clone().ok_or_else(|| {
            EvalError::Invalid(format!(
                "family-v3 Codex lane {} omitted its host-resolved model",
                expected.order
            ))
        })?,
        agents_md_sha256: actual.codex_agents_md_sha256.clone().ok_or_else(|| {
            EvalError::Invalid(format!(
                "family-v3 Codex lane {} omitted its agents instruction digest",
                expected.order
            ))
        })?,
    };
    validate_recorded_codex_session_rollout(
        expected,
        phase,
        Path::new(&actual.trace_path),
        &recorded,
    )
}

fn stale_phase_receipt_audit_is_valid(
    plan: &PreparedNativePilot,
    canonical_run_plan: &str,
    phase: NativePilotRunPhase,
    audit: &NativePilotAudit,
) -> bool {
    if audit.pilot_id != plan.pilot_id
        || audit.run_plan != canonical_run_plan
        || audit.invalid
        || audit.lanes.len() != plan.lanes.len()
    {
        return false;
    }
    let lifecycle_is_valid = audit
        .lanes
        .iter()
        .zip(&plan.lanes)
        .all(|(actual, expected)| {
            let expected_phase = match phase {
                NativePilotRunPhase::Teaching if expected.host == "codex" => {
                    NativeLanePhase::AwaitingActivation
                }
                NativePilotRunPhase::Teaching | NativePilotRunPhase::Activation => {
                    NativeLanePhase::ReadyForEvaluation
                }
                NativePilotRunPhase::Evaluation => NativeLanePhase::EvaluationComplete,
            };
            actual.order == expected.order
                && actual.arm == expected.arm
                && actual.phase == expected_phase
                && actual.failures.is_empty()
                && actual.native_memory_write_attempts.is_empty()
                && actual.teaching_trace.status == AuditStatus::Passed
                && match (&actual.activation_trace, &expected.activation_trace_path) {
                    (None, None) => true,
                    (Some(activation), Some(_)) => {
                        activation.status
                            == if phase == NativePilotRunPhase::Teaching {
                                AuditStatus::Pending
                            } else {
                                AuditStatus::Passed
                            }
                    }
                    _ => false,
                }
                && actual.evaluation_trace.status
                    == if phase == NativePilotRunPhase::Evaluation {
                        AuditStatus::Passed
                    } else {
                        AuditStatus::Pending
                    }
                && actual.agent_output.status
                    == if phase == NativePilotRunPhase::Evaluation {
                        AuditStatus::Passed
                    } else {
                        AuditStatus::Pending
                    }
        });
    lifecycle_is_valid
        && match phase {
            NativePilotRunPhase::Teaching => {
                !audit.ready_for_evaluation && !audit.complete && !audit.all_acceptance_passed
            }
            NativePilotRunPhase::Activation => {
                audit.ready_for_evaluation && !audit.complete && !audit.all_acceptance_passed
            }
            NativePilotRunPhase::Evaluation => audit.ready_for_evaluation && audit.complete,
        }
}

fn stale_phase_lane_lifecycle_is_valid(
    actual: &NativePilotLaneExecution,
    expected: &PreparedNativeLane,
    phase: NativePilotRunPhase,
    phase_started_unix_ms: u64,
    phase_completed_unix_ms: u64,
    previous_completed_unix_ms: u64,
) -> EvalResult<bool> {
    let (expected_trace, expected_argv) = match phase {
        NativePilotRunPhase::Teaching => (
            Path::new(&expected.teaching_trace_path),
            &expected.teaching_argv,
        ),
        NativePilotRunPhase::Activation => (
            Path::new(expected.activation_trace_path.as_deref().ok_or_else(|| {
                EvalError::Invalid(format!(
                    "stale-safety activation lane {} has no trace path",
                    expected.order
                ))
            })?),
            expected.activation_argv.as_ref().ok_or_else(|| {
                EvalError::Invalid(format!(
                    "stale-safety activation lane {} has no argv",
                    expected.order
                ))
            })?,
        ),
        NativePilotRunPhase::Evaluation => (
            Path::new(&expected.evaluation_trace_path),
            &expected.evaluation_argv,
        ),
    };
    let Some(provider_started) = actual.provider_started_unix_ms else {
        return Ok(false);
    };
    let Some(provider_completed) = actual.provider_completed_unix_ms else {
        return Ok(false);
    };
    Ok(actual.order == expected.order
        && actual.arm == expected.arm
        && actual.host == expected.host
        && actual.trace_path == expected_trace.display().to_string()
        && actual.stderr_path == stderr_path(expected_trace).display().to_string()
        && actual.argv_sha256 == argv_sha256(expected_argv)?
        && phase_started_unix_ms <= provider_started
        && previous_completed_unix_ms <= provider_started
        && provider_started <= provider_completed
        && provider_completed <= phase_completed_unix_ms
        && !actual.recovered_from_existing_trace)
}

fn write_stale_phase_report_digest(
    plan: &PreparedNativePilot,
    report_path: &Path,
) -> EvalResult<()> {
    if plan.stale_safety.is_none() {
        return Ok(());
    }
    let sidecar = report_path.with_extension("sha256");
    write_private_text(&sidecar, &format!("{}\n", sha256_file(report_path)?))
}

fn validate_execution_approval(
    plan: &PreparedNativePilot,
    approval: NativePilotExecutionApproval,
) -> EvalResult<()> {
    if plan.execution_approved {
        return Err(EvalError::Invalid(
            "prepared native-memory plans must retain execution_approved=false".to_string(),
        ));
    }
    if !approval.provider_execution {
        return Err(EvalError::Invalid(
            "provider execution is disabled; pass --approve-provider-execution only after explicit user approval"
                .to_string(),
        ));
    }
    if approval.claude_budget_cents != Some(plan.claude_budget_cents) {
        return Err(EvalError::Invalid(format!(
            "Claude budget confirmation must exactly match the frozen {} cent runner allocation",
            plan.claude_budget_cents
        )));
    }
    Ok(())
}

fn validate_execution_phase_scope(
    plan: &PreparedNativePilot,
    phase: NativePilotRunPhase,
) -> EvalResult<()> {
    if plan.evaluation_recovery.is_some() && phase != NativePilotRunPhase::Evaluation {
        return Err(EvalError::Invalid(
            "evaluation recovery plans authorize only the evaluation phase".to_string(),
        ));
    }
    Ok(())
}

fn validate_source_schema_rejection(
    source: &PreparedNativePilot,
    source_run_plan: &Path,
) -> EvalResult<u32> {
    let audit = audit_native_memory_pilot(source_run_plan)?;
    let invalid = audit
        .lanes
        .iter()
        .filter(|lane| lane.phase == NativeLanePhase::Invalid)
        .collect::<Vec<_>>();
    if !audit.invalid || invalid.len() != 1 {
        return Err(EvalError::Invalid(
            "evaluation recovery requires exactly one invalid source lane".to_string(),
        ));
    }
    let rejected = invalid[0];
    let lane = source
        .lanes
        .iter()
        .find(|lane| lane.order == rejected.order)
        .ok_or_else(|| EvalError::Invalid("source audit lane is not in the plan".to_string()))?;
    if lane.host != "claude_code"
        || rejected.evaluation_trace.status != AuditStatus::Failed
        || rejected.agent_output.status != AuditStatus::Pending
        || rejected.evaluation_trace.detail != "evaluation trace is not a non-empty file"
        || !rejected
            .failures
            .iter()
            .any(|failure| failure == "evaluation trace is not a non-empty file")
        || !rejected
            .failures
            .iter()
            .any(|failure| failure == "evaluation trace and structured output are incomplete")
    {
        return Err(EvalError::Invalid(
            "source invalidity is not the exact Claude pre-provider schema rejection".to_string(),
        ));
    }
    if audit.lanes.iter().any(|observed| {
        observed.order != rejected.order && observed.phase != NativeLanePhase::ReadyForEvaluation
    }) {
        return Err(EvalError::Invalid(
            "source plan has another lane outside ready_for_evaluation".to_string(),
        ));
    }
    let trace = Path::new(&lane.evaluation_trace_path);
    if fs::metadata(trace)?.len() != 0 {
        return Err(EvalError::Invalid(
            "rejected Claude evaluation trace is not empty".to_string(),
        ));
    }
    let stderr = stderr_path(trace);
    if fs::read_to_string(&stderr)?.trim_end() != CLAUDE_SCHEMA_REJECTION {
        return Err(EvalError::Invalid(
            "source stderr does not match the exact Claude schema rejection".to_string(),
        ));
    }
    if phase_report_path(source_run_plan, NativePilotRunPhase::Evaluation)?.exists() {
        return Err(EvalError::Invalid(
            "source plan unexpectedly has a completed evaluation phase report".to_string(),
        ));
    }
    Ok(rejected.order)
}

fn classify_source_evaluation_rejection(
    source: &PreparedNativePilot,
    source_run_plan: &Path,
) -> EvalResult<(EvaluationRecoveryKind, u32)> {
    let audit = audit_native_memory_pilot(source_run_plan)?;
    let invalid = audit
        .lanes
        .iter()
        .filter(|lane| lane.phase == NativeLanePhase::Invalid)
        .collect::<Vec<_>>();
    if invalid.len() != 1 {
        return Err(EvalError::Invalid(
            "evaluation recovery requires exactly one invalid source lane".to_string(),
        ));
    }
    let lane = source
        .lanes
        .iter()
        .find(|lane| lane.order == invalid[0].order)
        .ok_or_else(|| EvalError::Invalid("invalid source lane disappeared".to_string()))?;
    if lane.host == "claude_code" {
        return validate_source_schema_rejection(source, source_run_plan)
            .map(|order| (EvaluationRecoveryKind::ClaudeSchemaRejection, order));
    }
    if lane.host == "codex" {
        return validate_source_codex_terminal_output_rejection(source, source_run_plan)
            .map(|order| (EvaluationRecoveryKind::CodexTerminalOutput, order));
    }
    Err(EvalError::Invalid(format!(
        "unsupported recovery host for lane {}: {}",
        lane.order, lane.host
    )))
}

fn validate_source_codex_terminal_output_rejection(
    source: &PreparedNativePilot,
    source_run_plan: &Path,
) -> EvalResult<u32> {
    let audit = audit_native_memory_pilot(source_run_plan)?;
    let invalid = audit
        .lanes
        .iter()
        .filter(|lane| lane.phase == NativeLanePhase::Invalid)
        .collect::<Vec<_>>();
    if !audit.invalid || invalid.len() != 1 {
        return Err(EvalError::Invalid(
            "Codex terminal-output recovery requires exactly one invalid source lane".to_string(),
        ));
    }
    let rejected = invalid[0];
    let lane = source
        .lanes
        .iter()
        .find(|lane| lane.order == rejected.order)
        .ok_or_else(|| EvalError::Invalid("rejected source lane disappeared".to_string()))?;
    if lane.host != "codex"
        || rejected.evaluation_trace.status != AuditStatus::Passed
        || rejected.agent_output.status != AuditStatus::Pending
        || rejected.failures != ["evaluation trace and structured output are incomplete"]
    {
        return Err(EvalError::Invalid(
            "source invalidity is not an exact Codex structured-output lifecycle rejection"
                .to_string(),
        ));
    }
    if audit.lanes.iter().any(|observed| {
        observed.order != rejected.order && observed.phase != NativeLanePhase::EvaluationComplete
    }) {
        return Err(EvalError::Invalid(
            "Codex terminal-output recovery requires every other source lane to be complete"
                .to_string(),
        ));
    }
    if phase_report_path(source_run_plan, NativePilotRunPhase::Evaluation)?.exists() {
        return Err(EvalError::Invalid(
            "source plan unexpectedly has a completed evaluation phase report".to_string(),
        ));
    }
    let trace = Path::new(&lane.evaluation_trace_path);
    if distinct_agent_output_count(trace, source.protocol_schema_version)? <= 1 {
        return Err(EvalError::Invalid(
            "Codex source trace does not contain competing schema-shaped agent messages"
                .to_string(),
        ));
    }
    codex_terminal_agent_output(trace, source.protocol_schema_version)?;
    attest_codex_final_response_contract(Path::new(&source.binaries["codex"].path))?;
    Ok(rejected.order)
}

fn recovery_rejection_reason(kind: EvaluationRecoveryKind) -> &'static str {
    match kind {
        EvaluationRecoveryKind::ClaudeSchemaRejection => CLAUDE_SCHEMA_REJECTION,
        EvaluationRecoveryKind::CodexTerminalOutput => CODEX_TERMINAL_OUTPUT_REJECTION,
    }
}

fn rewrite_evaluation_argv_for_recovery(
    source: &[String],
    host: &str,
    schema_path: &Path,
    binary: &PilotBinaryAttestation,
    recovery_kind: EvaluationRecoveryKind,
) -> EvalResult<Vec<String>> {
    let mut argv = source.to_vec();
    let program = argv
        .first_mut()
        .ok_or_else(|| EvalError::Invalid("recovery evaluation argv has no program".to_string()))?;
    *program = binary.path.clone();
    let option = match host {
        "claude_code" => "--json-schema",
        "codex" => "--output-schema",
        _ => {
            return Err(EvalError::Invalid(format!(
                "unsupported recovery host: {host}"
            )))
        }
    };
    let matches = argv
        .iter()
        .enumerate()
        .filter(|(_, value)| value.as_str() == option)
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    if matches.len() != 1 || matches[0] + 1 >= argv.len() {
        return Err(EvalError::Invalid(format!(
            "recovery evaluation argv must contain exactly one {option} value"
        )));
    }
    let value_index = matches[0] + 1;
    if host == "claude_code" {
        match recovery_kind {
            EvaluationRecoveryKind::ClaudeSchemaRejection => {
                if !argv[value_index].contains("https://json-schema.org/draft/2020-12/schema") {
                    return Err(EvalError::Invalid(
                        "Claude recovery source does not contain the rejected schema dialect"
                            .to_string(),
                    ));
                }
            }
            EvaluationRecoveryKind::CodexTerminalOutput => {
                if argv[value_index] != AGENT_OUTPUT_SCHEMA {
                    return Err(EvalError::Invalid(
                        "Codex terminal-output recovery source does not contain the portable Claude schema"
                            .to_string(),
                    ));
                }
            }
        }
        argv[value_index] = AGENT_OUTPUT_SCHEMA.to_string();
    } else {
        argv[value_index] = schema_path.canonicalize()?.display().to_string();
    }
    Ok(argv)
}

fn rewrite_codex_authentication_for_recovery(
    lane: &mut PreparedNativeLane,
    codex: &PilotBinaryAttestation,
) -> EvalResult<()> {
    let authentication = lane.codex_authentication.as_mut().ok_or_else(|| {
        EvalError::Invalid(format!(
            "Codex recovery lane {} has no authentication contract",
            lane.order
        ))
    })?;
    for argv in [
        &mut authentication.login_argv,
        &mut authentication.status_argv,
    ] {
        let program = argv.first_mut().ok_or_else(|| {
            EvalError::Invalid(format!(
                "Codex recovery lane {} has an empty authentication argv",
                lane.order
            ))
        })?;
        *program = codex.path.clone();
    }
    Ok(())
}

fn attest_evaluation_recovery_binaries(
    source: &PreparedNativePilot,
    codex_binary: Option<&Path>,
    recovery_kind: EvaluationRecoveryKind,
) -> EvalResult<BTreeMap<String, PilotBinaryAttestation>> {
    let mut binaries = BTreeMap::new();
    for &name in required_binary_attestation_names(source.protocol_schema_version) {
        let expected = source.binaries.get(name).ok_or_else(|| {
            EvalError::Invalid(format!("source plan is missing {name} binary attestation"))
        })?;
        let source_actual = attest_binary(Path::new(&expected.path))?;
        if &source_actual != expected {
            return Err(EvalError::Invalid(format!(
                "source {name} binary drifted before recovery preparation"
            )));
        }
        let path = if name == "codex" {
            codex_binary
                .unwrap_or_else(|| Path::new(&expected.path))
                .to_path_buf()
        } else if name == "engram_eval"
            && recovery_kind == EvaluationRecoveryKind::CodexTerminalOutput
        {
            std::env::current_exe().map_err(EvalError::Io)?
        } else {
            PathBuf::from(&expected.path)
        };
        let actual = attest_binary(&path)?;
        let matches = if name == "codex" {
            actual.version == expected.version && actual.sha256 == expected.sha256
        } else if name == "engram_eval"
            && recovery_kind == EvaluationRecoveryKind::CodexTerminalOutput
        {
            actual.version == expected.version && actual.sha256 != expected.sha256
        } else {
            &actual == expected
        };
        if !matches {
            return Err(EvalError::Invalid(format!(
                "{name} recovery binary does not match the source attestation: expected path={}, version={}, sha256={}; actual path={}, version={}, sha256={}",
                expected.path,
                expected.version,
                expected.sha256,
                actual.path,
                actual.version,
                actual.sha256
            )));
        }
        binaries.insert(name.to_string(), actual);
    }
    if let Some(expected) = source.binaries.get("codex_code_mode_host") {
        let source_codex = source.binaries.get("codex").ok_or_else(|| {
            EvalError::Invalid("source plan is missing codex binary attestation".to_string())
        })?;
        let source_actual = attest_codex_code_mode_host(source_codex)?;
        if &source_actual != expected {
            return Err(EvalError::Invalid(
                "source codex_code_mode_host binary drifted before recovery preparation"
                    .to_string(),
            ));
        }
        let actual = attest_codex_code_mode_host(&binaries["codex"])?;
        if actual.version != expected.version || actual.sha256 != expected.sha256 {
            return Err(EvalError::Invalid(format!(
                "codex_code_mode_host recovery binary does not match the source attestation: expected version={}, sha256={}; actual version={}, sha256={}",
                expected.version, expected.sha256, actual.version, actual.sha256
            )));
        }
        binaries.insert("codex_code_mode_host".to_string(), actual);
    }
    if recovery_kind == EvaluationRecoveryKind::CodexTerminalOutput {
        attest_codex_final_response_contract(Path::new(&binaries["codex"].path))?;
    }
    Ok(binaries)
}

fn evaluation_recovery_binaries_match(
    source: &BTreeMap<String, PilotBinaryAttestation>,
    recovery: &BTreeMap<String, PilotBinaryAttestation>,
    recovery_kind: EvaluationRecoveryKind,
) -> bool {
    source.len() == recovery.len()
        && source.iter().all(|(name, expected)| {
            recovery.get(name).is_some_and(|actual| {
                if matches!(name.as_str(), "codex" | "codex_code_mode_host") {
                    actual.version == expected.version && actual.sha256 == expected.sha256
                } else if name == "engram_eval"
                    && recovery_kind == EvaluationRecoveryKind::CodexTerminalOutput
                {
                    actual.version == expected.version && actual.sha256 != expected.sha256
                } else {
                    actual == expected
                }
            })
        })
}

fn attest_codex_final_response_contract(codex: &Path) -> EvalResult<()> {
    let output = Command::new(codex)
        .args(["exec", "--help"])
        .output()
        .map_err(EvalError::Io)?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success()
        || !stdout.contains("--output-schema <FILE>")
        || !stdout.contains(CODEX_FINAL_RESPONSE_HELP)
    {
        return Err(EvalError::Invalid(
            "attested Codex binary does not declare --output-schema as its final-response shape"
                .to_string(),
        ));
    }
    Ok(())
}

fn evaluation_recovery_pilot_id(
    source: &PreparedNativePilot,
    binaries: &BTreeMap<String, PilotBinaryAttestation>,
    recovery_kind: EvaluationRecoveryKind,
) -> String {
    match recovery_kind {
        EvaluationRecoveryKind::ClaudeSchemaRejection => {
            let generation = if &source.binaries == binaries { 1 } else { 2 };
            format!("{}-evaluation-recovery-{generation}", source.pilot_id)
        }
        EvaluationRecoveryKind::CodexTerminalOutput => {
            format!("{}-codex-terminal-recovery-1", source.pilot_id)
        }
    }
}

fn evaluation_recovery_accounting(
    source: &PreparedNativePilot,
    recovery_kind: EvaluationRecoveryKind,
) -> EvalResult<(u32, u64, u32)> {
    let claude_lanes = source
        .lanes
        .iter()
        .filter(|lane| lane.host == "claude_code")
        .count();
    let source_calls = claude_lanes
        .checked_mul(2)
        .ok_or_else(|| EvalError::Invalid("Claude source-call accounting overflow".to_string()))?;
    if source_calls == 0 {
        return Err(EvalError::Invalid(
            "evaluation recovery source has no Claude calls".to_string(),
        ));
    }
    let source_budget_millis = u64::from(source.claude_budget_cents)
        .checked_mul(10)
        .ok_or_else(|| EvalError::Invalid("Claude source budget overflow".to_string()))?
        / source_calls as u64;
    let remaining_budget_millis = source_budget_millis
        .checked_mul(claude_lanes as u64)
        .ok_or_else(|| EvalError::Invalid("Claude recovery budget overflow".to_string()))?;
    if remaining_budget_millis % 10 != 0 {
        return Err(EvalError::Invalid(
            "Claude recovery allocation is not an exact number of cents".to_string(),
        ));
    }
    let recovery_budget_cents = u32::try_from(remaining_budget_millis / 10)
        .map_err(|_| EvalError::Invalid("Claude recovery allocation is too large".to_string()))?;
    let teaching_spend = source
        .lanes
        .iter()
        .filter(|lane| lane.host == "claude_code")
        .try_fold(0_u64, |total, lane| {
            let cost = claude_trace_summary(Path::new(&lane.teaching_trace_path))?
                .provider_reported_cost_microusd
                .ok_or_else(|| {
                    EvalError::Invalid(format!(
                        "Claude teaching trace for lane {} omitted provider cost",
                        lane.order
                    ))
                })?;
            total.checked_add(cost).ok_or_else(|| {
                EvalError::Invalid("Claude teaching-spend accounting overflow".to_string())
            })
        })?;
    let evaluation_spend = if recovery_kind == EvaluationRecoveryKind::CodexTerminalOutput {
        source
            .lanes
            .iter()
            .filter(|lane| lane.host == "claude_code")
            .try_fold(0_u64, |total, lane| {
                let cost = claude_trace_summary(Path::new(&lane.evaluation_trace_path))?
                    .provider_reported_cost_microusd
                    .ok_or_else(|| {
                        EvalError::Invalid(format!(
                            "Claude evaluation trace for lane {} omitted provider cost",
                            lane.order
                        ))
                    })?;
                total.checked_add(cost).ok_or_else(|| {
                    EvalError::Invalid("Claude evaluation-spend accounting overflow".to_string())
                })
            })?
    } else {
        0
    };
    let prior_spend = source
        .claude_prior_spend_microusd
        .checked_add(teaching_spend)
        .and_then(|total| total.checked_add(evaluation_spend))
        .ok_or_else(|| EvalError::Invalid("Claude prior-spend accounting overflow".to_string()))?;
    Ok((
        recovery_budget_cents,
        prior_spend,
        u32::try_from(claude_lanes)
            .map_err(|_| EvalError::Invalid("too many Claude recovery calls".to_string()))?,
    ))
}

fn validate_interrupted_teaching_audit(
    source: &PreparedNativePilot,
    source_run_plan: &Path,
    audit: &NativePilotAudit,
) -> EvalResult<InterruptedTeachingRecovery> {
    if audit.invalid || audit.complete || audit.ready_for_evaluation {
        return Err(EvalError::Invalid(
            "execution recovery requires a valid, incomplete teaching lifecycle".to_string(),
        ));
    }
    for phase in [
        NativePilotRunPhase::Teaching,
        NativePilotRunPhase::Activation,
        NativePilotRunPhase::Evaluation,
    ] {
        if phase_report_path(source_run_plan, phase)?.exists() {
            return Err(EvalError::Invalid(format!(
                "execution recovery source unexpectedly has a completed {} report",
                phase.label()
            )));
        }
    }

    for lane in &source.lanes {
        let observed = audit
            .lanes
            .iter()
            .find(|candidate| candidate.order == lane.order)
            .ok_or_else(|| {
                EvalError::Invalid(format!("source audit omitted lane {}", lane.order))
            })?;
        if !observed.failures.is_empty()
            || observed.evaluation_trace.status != AuditStatus::Pending
            || observed.agent_output.status != AuditStatus::Pending
            || observed
                .activation_trace
                .as_ref()
                .is_some_and(|trace| trace.status != AuditStatus::Pending)
        {
            return Err(EvalError::Invalid(format!(
                "lane {} contains output outside the interrupted teaching prefix",
                lane.order
            )));
        }
    }

    let awaiting_verification = audit
        .lanes
        .iter()
        .filter(|lane| lane.phase == NativeLanePhase::AwaitingVerification)
        .collect::<Vec<_>>();
    if !awaiting_verification.is_empty() {
        return validate_procedure_teaching_scope_interruption(
            source,
            audit,
            &awaiting_verification,
        );
    }

    let mut completed = Vec::new();
    let mut reached_prepared = false;
    for lane in &source.lanes {
        let observed = audit
            .lanes
            .iter()
            .find(|candidate| candidate.order == lane.order)
            .ok_or_else(|| {
                EvalError::Invalid(format!("source audit omitted lane {}", lane.order))
            })?;
        match observed.phase {
            NativeLanePhase::Prepared => reached_prepared = true,
            NativeLanePhase::AwaitingActivation if lane.host == "codex" => {
                if reached_prepared || observed.teaching_trace.status != AuditStatus::Passed {
                    return Err(EvalError::Invalid(
                        "completed teaching lanes are not a strict execution-order prefix"
                            .to_string(),
                    ));
                }
                completed.push(lane.order);
            }
            NativeLanePhase::ReadyForEvaluation if lane.host != "codex" => {
                if reached_prepared || observed.teaching_trace.status != AuditStatus::Passed {
                    return Err(EvalError::Invalid(
                        "completed teaching lanes are not a strict execution-order prefix"
                            .to_string(),
                    ));
                }
                completed.push(lane.order);
            }
            _ => {
                return Err(EvalError::Invalid(format!(
                    "lane {} is not in a recoverable teaching state: {:?}",
                    lane.order, observed.phase
                )));
            }
        }
    }
    if completed.is_empty() || completed.len() == source.lanes.len() {
        return Err(EvalError::Invalid(
            "execution recovery requires a non-empty, incomplete teaching prefix".to_string(),
        ));
    }

    let last_order = *completed
        .last()
        .ok_or_else(|| EvalError::Invalid("completed teaching prefix disappeared".to_string()))?;
    let last_lane = source
        .lanes
        .iter()
        .find(|lane| lane.order == last_order)
        .ok_or_else(|| EvalError::Invalid("last completed lane disappeared".to_string()))?;
    let last_audit = audit
        .lanes
        .iter()
        .find(|lane| lane.order == last_order)
        .ok_or_else(|| EvalError::Invalid("last completed audit lane disappeared".to_string()))?;
    if last_lane.host == "claude_code" {
        if source.protocol_schema_version != OPERATION_EVIDENCE_ROUTE_NATIVE_PILOT_SCHEMA_VERSION
            || last_lane.arm != "claude_native_memory"
            || last_lane.memory_layer.uses_engram()
            || last_audit.phase != NativeLanePhase::ReadyForEvaluation
        {
            return Err(EvalError::Invalid(
                "source does not match the exact Claude teaching-command wrapper rejection"
                    .to_string(),
            ));
        }
        validate_completed_claude_teaching_trace(Path::new(&last_lane.teaching_trace_path))?;
        validate_teaching_command_evidence(last_lane, source.protocol_schema_version)?;
        if validate_teaching_command_evidence(
            last_lane,
            OPERATION_EVIDENCE_ROUTE_NATIVE_PILOT_SCHEMA_VERSION - 1,
        )
        .is_ok()
        {
            return Err(EvalError::Invalid(
                "Claude teaching trace does not require the repaired command-wrapper semantics"
                    .to_string(),
            ));
        }
        let native = last_audit
            .artifacts
            .iter()
            .find(|artifact| artifact.layer == "claude_code_auto_memory")
            .ok_or_else(|| {
                EvalError::Invalid(
                    "Claude teaching-command wrapper recovery omitted native memory evidence"
                        .to_string(),
                )
            })?;
        if native.status != AuditStatus::Passed
            || !native.observed
            || !Path::new(&native.path).is_file()
        {
            return Err(EvalError::Invalid(
                "Claude teaching-command wrapper recovery requires the observed native memory artifact"
                    .to_string(),
            ));
        }
        return Ok(InterruptedTeachingRecovery {
            kind: ExecutionRecoveryKind::ClaudeTeachingCommandWrapper,
            completed_lane_orders: completed,
        });
    }
    let native = last_audit
        .artifacts
        .iter()
        .find(|artifact| artifact.layer == "codex_native_memory")
        .ok_or_else(|| {
            EvalError::Invalid(
                "interrupted teaching prefix did not end at a Codex native-memory lane".to_string(),
            )
        })?;
    let memory = Path::new(&native.path);
    if last_lane.host != "codex"
        || native.status != AuditStatus::Pending
        || native.observed
        || memory.join("MEMORY.md").exists()
        || memory.join("memory_summary.md").exists()
        || !memory.join("phase2_workspace_diff.md").is_file()
        || !memory.join("raw_memories.md").is_file()
        || !memory.join("extensions/ad_hoc/instructions.md").is_file()
    {
        return Err(EvalError::Invalid(
            "source does not match the exact Codex pre-consolidation audit rejection".to_string(),
        ));
    }
    Ok(InterruptedTeachingRecovery {
        kind: ExecutionRecoveryKind::CodexPreConsolidationAudit,
        completed_lane_orders: completed,
    })
}

fn validate_procedure_teaching_scope_interruption(
    source: &PreparedNativePilot,
    audit: &NativePilotAudit,
    awaiting_verification: &[&crate::native_audit::NativeLaneAudit],
) -> EvalResult<InterruptedTeachingRecovery> {
    if awaiting_verification.len() != 1 {
        return Err(EvalError::Invalid(
            "procedure teaching-scope recovery requires exactly one interrupted lane".to_string(),
        ));
    }
    let interrupted = awaiting_verification[0];
    let interrupted_index = source
        .lanes
        .iter()
        .position(|lane| lane.order == interrupted.order)
        .ok_or_else(|| EvalError::Invalid("interrupted teaching lane disappeared".to_string()))?;
    if interrupted_index + 1 == source.lanes.len() {
        return Err(EvalError::Invalid(
            "procedure teaching-scope recovery requires remaining provider calls".to_string(),
        ));
    }

    for (index, lane) in source.lanes.iter().enumerate() {
        let observed = audit
            .lanes
            .iter()
            .find(|candidate| candidate.order == lane.order)
            .ok_or_else(|| {
                EvalError::Invalid(format!("source audit omitted lane {}", lane.order))
            })?;
        let valid = if index < interrupted_index {
            (lane.host == "codex" && observed.phase == NativeLanePhase::AwaitingActivation)
                || (lane.host != "codex" && observed.phase == NativeLanePhase::ReadyForEvaluation)
        } else if index == interrupted_index {
            observed.phase == NativeLanePhase::AwaitingVerification
                && observed.teaching_trace.status == AuditStatus::Passed
                && observed
                    .procedure_verification
                    .as_ref()
                    .is_some_and(|verification| verification.status == AuditStatus::Pending)
        } else {
            observed.phase == NativeLanePhase::Prepared
        };
        if !valid {
            return Err(EvalError::Invalid(
                "procedure teaching-scope interruption is not a strict execution-order prefix"
                    .to_string(),
            ));
        }
    }

    let lane = &source.lanes[interrupted_index];
    if lane.host != "claude_code" || !lane.memory_layer.uses_engram() {
        return Err(EvalError::Invalid(
            "procedure teaching-scope recovery requires an interrupted Claude Engram lane"
                .to_string(),
        ));
    }
    validate_completed_claude_teaching_trace(Path::new(&lane.teaching_trace_path))?;
    let trace = fs::read_to_string(&lane.teaching_trace_path)?;
    let candidates = candidate_path(lane)?;
    require_existing_file(&candidates, "procedure candidate review")?;
    require_existing_file(&candidate_stderr_path(lane)?, "procedure candidate stderr")?;
    if lane.cleanup_argv.is_some() {
        require_existing_file(
            &cleanup_stdout_path(lane, NativePilotRunPhase::Teaching)?,
            "Engram cleanup stdout",
        )?;
        require_existing_file(
            &cleanup_stderr_path(lane, NativePilotRunPhase::Teaching)?,
            "Engram cleanup stderr",
        )?;
    }
    let verification = Path::new(
        lane.post_teaching_verification_output_path
            .as_deref()
            .ok_or_else(|| {
                EvalError::Invalid(format!(
                    "lane {} is missing procedure verification output path",
                    lane.order
                ))
            })?,
    );
    require_absent(verification, "procedure verification output")?;
    require_absent(
        &verification_stderr_path(verification),
        "verification stderr",
    )?;

    let contract: RunnerAcceptanceContract =
        serde_json::from_reader(File::open(&lane.acceptance_contract)?)?;
    let teaching_remote = repository_remote(Path::new(&lane.teaching_cwd))?;
    if normalize_remote(&teaching_remote) == normalize_remote(&contract.expected_repository_remote)
    {
        return Err(EvalError::Invalid(
            "procedure teaching-scope recovery requires distinct teaching and evaluation repositories"
                .to_string(),
        ));
    }
    let candidate_id = select_procedure_candidate(&candidates, &contract, &teaching_remote)?;
    if !trace.contains(&candidate_id) {
        return Err(EvalError::Invalid(format!(
            "lane {} selected candidate ID does not occur in the teaching trace",
            lane.order
        )));
    }
    if select_procedure_candidate(&candidates, &contract, &contract.expected_repository_remote)
        .is_ok()
    {
        return Err(EvalError::Invalid(
            "procedure candidate unexpectedly matched the evaluation repository".to_string(),
        ));
    }

    Ok(InterruptedTeachingRecovery {
        kind: ExecutionRecoveryKind::ProcedureTeachingScope,
        completed_lane_orders: source
            .lanes
            .iter()
            .take(interrupted_index + 1)
            .map(|lane| lane.order)
            .collect(),
    })
}

fn attest_inherited_teaching_files(
    source: &PreparedNativePilot,
    completed_lane_orders: &[u32],
    recovery_kind: ExecutionRecoveryKind,
) -> EvalResult<Vec<PreparedNativeInheritedFile>> {
    let completed = completed_lane_orders
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    if completed.len() != completed_lane_orders.len() {
        return Err(EvalError::Invalid(
            "execution recovery contains duplicate completed lane orders".to_string(),
        ));
    }
    let mut paths = Vec::new();
    let interrupted_order = (recovery_kind == ExecutionRecoveryKind::ProcedureTeachingScope)
        .then(|| completed_lane_orders.last().copied())
        .flatten();
    for lane in source
        .lanes
        .iter()
        .filter(|lane| completed.contains(&lane.order))
    {
        let trace = Path::new(&lane.teaching_trace_path);
        paths.push(trace.to_path_buf());
        paths.push(stderr_path(trace));
        if lane.memory_layer.uses_engram() {
            paths.push(candidate_path(lane)?);
            paths.push(candidate_stderr_path(lane)?);
            if interrupted_order != Some(lane.order) {
                let verification = lane
                    .post_teaching_verification_output_path
                    .as_deref()
                    .ok_or_else(|| {
                        EvalError::Invalid(format!(
                            "completed Engram lane {} has no verification output",
                            lane.order
                        ))
                    })?;
                paths.push(PathBuf::from(verification));
                paths.push(verification_stderr_path(Path::new(verification)));
            }
            if lane.cleanup_argv.is_some() {
                paths.push(cleanup_stdout_path(lane, NativePilotRunPhase::Teaching)?);
                paths.push(cleanup_stderr_path(lane, NativePilotRunPhase::Teaching)?);
            }
        }
    }
    paths.sort();
    paths.dedup();
    paths
        .into_iter()
        .map(|path| {
            let canonical = path.canonicalize().map_err(|error| {
                EvalError::Invalid(format!(
                    "inherited teaching file is missing at {}: {error}",
                    path.display()
                ))
            })?;
            Ok(PreparedNativeInheritedFile {
                path: canonical.display().to_string(),
                sha256: sha256_file(&canonical)?,
            })
        })
        .collect()
}

fn execution_recovery_accounting(
    source: &PreparedNativePilot,
    completed_lane_orders: &[u32],
) -> EvalResult<(u32, u64, u32)> {
    let claude_lanes = source
        .lanes
        .iter()
        .filter(|lane| lane.host == "claude_code")
        .count();
    let source_calls = claude_lanes
        .checked_mul(2)
        .ok_or_else(|| EvalError::Invalid("Claude source-call accounting overflow".to_string()))?;
    let completed = completed_lane_orders
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let completed_claude = source
        .lanes
        .iter()
        .filter(|lane| lane.host == "claude_code" && completed.contains(&lane.order))
        .collect::<Vec<_>>();
    let remaining_calls = source_calls
        .checked_sub(completed_claude.len())
        .ok_or_else(|| {
            EvalError::Invalid("Claude remaining-call accounting underflow".to_string())
        })?;
    if source_calls == 0 || remaining_calls == 0 {
        return Err(EvalError::Invalid(
            "execution recovery source has no remaining Claude calls".to_string(),
        ));
    }
    let per_call_budget_millis = u64::from(source.claude_budget_cents)
        .checked_mul(10)
        .ok_or_else(|| EvalError::Invalid("Claude source budget overflow".to_string()))?
        / source_calls as u64;
    let remaining_budget_millis = per_call_budget_millis
        .checked_mul(remaining_calls as u64)
        .ok_or_else(|| EvalError::Invalid("Claude recovery budget overflow".to_string()))?;
    let recovery_budget_cents = u32::try_from(remaining_budget_millis.div_ceil(10))
        .map_err(|_| EvalError::Invalid("Claude recovery allocation is too large".to_string()))?;
    if u64::from(recovery_budget_cents) * 10 / remaining_calls as u64 != per_call_budget_millis {
        return Err(EvalError::Invalid(
            "Claude recovery allocation cannot preserve the frozen per-call budget".to_string(),
        ));
    }
    let completed_spend = completed_claude
        .into_iter()
        .try_fold(0_u64, |total, lane| {
            let cost = claude_trace_summary(Path::new(&lane.teaching_trace_path))?
                .provider_reported_cost_microusd
                .ok_or_else(|| {
                    EvalError::Invalid(format!(
                        "Claude teaching trace for lane {} omitted provider cost",
                        lane.order
                    ))
                })?;
            total.checked_add(cost).ok_or_else(|| {
                EvalError::Invalid("Claude completed-spend accounting overflow".to_string())
            })
        })?;
    let prior_spend = source
        .claude_prior_spend_microusd
        .checked_add(completed_spend)
        .ok_or_else(|| EvalError::Invalid("Claude prior-spend accounting overflow".to_string()))?;
    Ok((
        recovery_budget_cents,
        prior_spend,
        u32::try_from(remaining_calls)
            .map_err(|_| EvalError::Invalid("too many Claude recovery calls".to_string()))?,
    ))
}

fn execution_recovery_pilot_id(
    source: &PreparedNativePilot,
    binaries: &BTreeMap<String, PilotBinaryAttestation>,
) -> String {
    let evaluator = &binaries["engram_eval"].sha256;
    format!("{}-teaching-recovery-{}", source.pilot_id, &evaluator[..12])
}

fn execution_recovery_binaries_match(
    source: &BTreeMap<String, PilotBinaryAttestation>,
    recovery: &BTreeMap<String, PilotBinaryAttestation>,
) -> bool {
    source.len() == recovery.len()
        && source.iter().all(|(name, expected)| {
            recovery.get(name).is_some_and(|actual| {
                if name == "engram_eval" {
                    actual.version == expected.version && actual.sha256 != expected.sha256
                } else {
                    actual == expected
                }
            })
        })
}

fn validate_execution_recovery(plan: &PreparedNativePilot) -> EvalResult<()> {
    let Some(recovery) = &plan.execution_recovery else {
        return Ok(());
    };
    if plan.evaluation_recovery.is_some() {
        return Err(EvalError::Invalid(
            "execution and evaluation recovery cannot be combined".to_string(),
        ));
    }
    let loaded_source =
        load_historical_native_plan_with_identity(Path::new(&recovery.source_run_plan))?;
    validate_loaded_plan_digest(&loaded_source)?;
    if loaded_source.sha256 != recovery.source_run_plan_sha256 {
        return Err(EvalError::Invalid(
            "execution recovery source plan hash drifted".to_string(),
        ));
    }
    let source_path = loaded_source.canonical_path;
    let source = loaded_source.plan;
    if source.stale_safety.is_some() || validate_native_stale_plan_binding(&source, &source_path)? {
        return Err(EvalError::Invalid(
            "stale-safety plans forbid lifecycle recovery and provider replay".to_string(),
        ));
    }
    if source.evaluation_recovery.is_some() || source.execution_recovery.is_some() {
        return Err(EvalError::Invalid(
            "lifecycle recovery plans cannot be chained".to_string(),
        ));
    }
    validate_plan_attestations(&source)?;
    let recovery_kind = execution_recovery_kind(&recovery.failure_reason)?;
    let expected_orders = source
        .lanes
        .iter()
        .take(recovery.completed_lane_orders.len())
        .map(|lane| lane.order)
        .collect::<Vec<_>>();
    if recovery.completed_lane_orders.is_empty()
        || recovery.completed_lane_orders.len() == source.lanes.len()
        || recovery.completed_lane_orders != expected_orders
    {
        return Err(EvalError::Invalid(
            "execution recovery completed lanes are not a strict source-order prefix".to_string(),
        ));
    }
    let inherited =
        attest_inherited_teaching_files(&source, &recovery.completed_lane_orders, recovery_kind)?;
    let (budget_cents, prior_spend, remaining_calls) =
        execution_recovery_accounting(&source, &recovery.completed_lane_orders)?;
    if recovery.interrupted_phase != NativePilotRunPhase::Teaching.label()
        || recovery.failure_reason != recovery_kind.failure_reason()
        || recovery.inherited_files != inherited
        || recovery.claude_remaining_calls != remaining_calls
        || recovery.recovery_prepared_unix_ms <= source.prepared_unix_ms
        || plan.pilot_id != execution_recovery_pilot_id(&source, &plan.binaries)
        || plan.execution_approved
        || plan.prepared_unix_ms != source.prepared_unix_ms
        || plan.codex_min_idle_hours != source.codex_min_idle_hours
        || plan.claude_budget_cents != budget_cents
        || plan.claude_prior_spend_microusd != prior_spend
        || plan.claude_authorized_ceiling_cents != source.claude_authorized_ceiling_cents
        || plan.claude_model != source.claude_model
        || plan.claude_max_turns != source.claude_max_turns
        || plan.native_memory_references != source.native_memory_references
        || !execution_recovery_binaries_match(&source.binaries, &plan.binaries)
        || plan.runtime_libraries != source.runtime_libraries
        || plan.engram_mcp_contract != source.engram_mcp_contract
        || plan.lanes != source.lanes
    {
        return Err(EvalError::Invalid(
            "execution recovery metadata does not match its immutable source".to_string(),
        ));
    }
    Ok(())
}

fn validate_evaluation_recovery(plan: &PreparedNativePilot, run_plan: &Path) -> EvalResult<()> {
    let Some(recovery) = &plan.evaluation_recovery else {
        return Ok(());
    };
    let loaded_source =
        load_historical_native_plan_with_identity(Path::new(&recovery.source_run_plan))?;
    validate_loaded_plan_digest(&loaded_source)?;
    if loaded_source.sha256 != recovery.source_run_plan_sha256 {
        return Err(EvalError::Invalid(
            "evaluation recovery source plan hash drifted".to_string(),
        ));
    }
    let source_path = loaded_source.canonical_path;
    let source = loaded_source.plan;
    if source.stale_safety.is_some() || validate_native_stale_plan_binding(&source, &source_path)? {
        return Err(EvalError::Invalid(
            "stale-safety plans forbid lifecycle recovery and provider replay".to_string(),
        ));
    }
    if source.evaluation_recovery.is_some() || source.execution_recovery.is_some() {
        return Err(EvalError::Invalid(
            "evaluation recovery plans cannot be chained".to_string(),
        ));
    }
    let (recovery_kind, rejected_order) =
        classify_source_evaluation_rejection(&source, &source_path)?;
    let rejected_lane = source
        .lanes
        .iter()
        .find(|lane| lane.order == rejected_order)
        .ok_or_else(|| EvalError::Invalid("rejected source lane disappeared".to_string()))?;
    let rejected_trace = Path::new(&rejected_lane.evaluation_trace_path).canonicalize()?;
    let rejected_stderr = stderr_path(&rejected_trace).canonicalize()?;
    if rejected_trace != Path::new(&recovery.rejected_evaluation_trace)
        || rejected_stderr != Path::new(&recovery.rejected_evaluation_stderr)
        || sha256_file(&rejected_trace)? != recovery.rejected_evaluation_trace_sha256
        || sha256_file(&rejected_stderr)? != recovery.rejected_evaluation_stderr_sha256
        || recovery.rejection_reason != recovery_rejection_reason(recovery_kind)
    {
        return Err(EvalError::Invalid(
            "evaluation recovery rejection provenance drifted".to_string(),
        ));
    }

    let run_root = run_plan
        .canonicalize()?
        .parent()
        .ok_or_else(|| EvalError::Invalid("recovery run plan has no parent".to_string()))?
        .to_path_buf();
    let schema_path = Path::new(&recovery.agent_output_schema).canonicalize()?;
    if schema_path != run_root.join("agent-output.schema.json")
        || fs::read_to_string(&schema_path)? != AGENT_OUTPUT_SCHEMA
    {
        return Err(EvalError::Invalid(
            "evaluation recovery output schema drifted".to_string(),
        ));
    }
    let (budget_cents, prior_spend, remaining_calls) =
        evaluation_recovery_accounting(&source, recovery_kind)?;
    if plan.pilot_id != evaluation_recovery_pilot_id(&source, &plan.binaries, recovery_kind)
        || plan.execution_approved
        || plan.prepared_unix_ms != source.prepared_unix_ms
        || plan.codex_min_idle_hours != source.codex_min_idle_hours
        || plan.claude_budget_cents != budget_cents
        || plan.claude_prior_spend_microusd != prior_spend
        || plan.claude_authorized_ceiling_cents != source.claude_authorized_ceiling_cents
        || plan.claude_model != source.claude_model
        || plan.claude_max_turns != source.claude_max_turns
        || plan.native_memory_references != source.native_memory_references
        || !evaluation_recovery_binaries_match(&source.binaries, &plan.binaries, recovery_kind)
        || plan.runtime_libraries != source.runtime_libraries
        || plan.engram_mcp_contract != source.engram_mcp_contract
        || recovery.claude_remaining_calls != remaining_calls
        || recovery.recovery_prepared_unix_ms <= source.prepared_unix_ms
        || plan.lanes.len() != source.lanes.len()
    {
        return Err(EvalError::Invalid(
            "evaluation recovery metadata does not match its immutable source".to_string(),
        ));
    }
    if recovery_kind == EvaluationRecoveryKind::CodexTerminalOutput {
        attest_codex_final_response_contract(Path::new(&plan.binaries["codex"].path))?;
    }
    let current = plan
        .lanes
        .iter()
        .map(|lane| (lane.order, lane))
        .collect::<BTreeMap<_, _>>();
    if current.len() != plan.lanes.len() {
        return Err(EvalError::Invalid(
            "evaluation recovery contains duplicate lane orders".to_string(),
        ));
    }
    for source_lane in &source.lanes {
        let mut expected = source_lane.clone();
        let lane_dir = run_root.join(format!("lane-{:02}-{}", source_lane.order, source_lane.arm));
        expected.evaluation_argv = rewrite_evaluation_argv_for_recovery(
            &source_lane.evaluation_argv,
            &source_lane.host,
            &schema_path,
            &plan.binaries[&source_lane.host],
            recovery_kind,
        )?;
        if source_lane.host == "codex" {
            rewrite_codex_authentication_for_recovery(&mut expected, &plan.binaries["codex"])?;
        }
        expected.evaluation_trace_path = lane_dir
            .join("evaluation-trace.jsonl")
            .display()
            .to_string();
        expected.agent_output_path = lane_dir.join("agent-output.json").display().to_string();
        if current.get(&source_lane.order).copied() != Some(&expected) {
            return Err(EvalError::Invalid(format!(
                "evaluation recovery lane {} drifted from its source",
                source_lane.order
            )));
        }
    }
    Ok(())
}

fn validate_plan_attestations(plan: &PreparedNativePilot) -> EvalResult<()> {
    let family_v3 = is_family_v3_plan(plan);
    if family_v3 {
        validate_native_pilot_family_v3_runtime_libraries(&plan.binaries, &plan.runtime_libraries)?;
    } else if !plan.runtime_libraries.is_empty() {
        return Err(EvalError::Invalid(
            "runtime-library attestations are accepted only for stale-safety family v3".to_string(),
        ));
    }
    for &name in required_binary_attestation_names(plan.protocol_schema_version) {
        let expected = plan.binaries.get(name).ok_or_else(|| {
            EvalError::Invalid(format!("run plan is missing {name} binary attestation"))
        })?;
        let actual = attest_binary(Path::new(&expected.path))?;
        if &actual != expected {
            return Err(EvalError::Invalid(format!(
                "{name} binary drifted after preparation: expected path={}, version={}, sha256={}; actual path={}, version={}, sha256={}",
                expected.path,
                expected.version,
                expected.sha256,
                actual.path,
                actual.version,
                actual.sha256
            )));
        }
    }
    if let Some(expected) = plan.binaries.get("codex_code_mode_host") {
        let codex = plan.binaries.get("codex").ok_or_else(|| {
            EvalError::Invalid("run plan is missing codex binary attestation".to_string())
        })?;
        let actual = attest_codex_code_mode_host(codex)?;
        if &actual != expected {
            return Err(EvalError::Invalid(format!(
                "codex_code_mode_host binary drifted after preparation: expected path={}, version={}, sha256={}; actual path={}, version={}, sha256={}",
                expected.path,
                expected.version,
                expected.sha256,
                actual.path,
                actual.version,
                actual.sha256
            )));
        }
    }
    validate_plan_contracts(plan)?;
    if family_v3 {
        validate_native_pilot_family_v3_runtime_libraries(&plan.binaries, &plan.runtime_libraries)?;
    }
    Ok(())
}

fn required_binary_attestation_names(protocol_schema_version: u32) -> &'static [&'static str] {
    if protocol_schema_version >= 5 {
        &["engram", "codex", "claude_code", "engram_eval"]
    } else {
        &["engram", "codex", "claude_code"]
    }
}

fn validate_plan_contracts(plan: &PreparedNativePilot) -> EvalResult<()> {
    crate::native_pilot::validate_native_pilot_schema_version(plan.protocol_schema_version)?;
    crate::native_pilot::validate_native_pilot_resource_budgets(plan.resource_budgets.as_ref())?;
    let live_contract = attest_engram_mcp_contract(&plan.binaries["engram"])?;
    validate_frozen_engram_contract(&plan.engram_mcp_contract, &live_contract)?;
    validate_codex_authentication_contract(plan)?;
    validate_claude_execution_contract(plan)?;
    for lane in &plan.lanes {
        let expected = plan.binaries.get(&lane.host).ok_or_else(|| {
            EvalError::Invalid(format!(
                "lane {} has unattested host {}",
                lane.order, lane.host
            ))
        })?;
        let commands = if plan.evaluation_recovery.is_some() {
            vec![&lane.evaluation_argv]
        } else {
            vec![&lane.teaching_argv, &lane.evaluation_argv]
        };
        for argv in commands {
            validate_program(argv, expected, &format!("lane {} host command", lane.order))?;
        }
        if plan.evaluation_recovery.is_none() {
            if let Some(argv) = &lane.activation_argv {
                validate_program(argv, expected, &format!("lane {} activation", lane.order))?;
            }
        }
        if let Some(argv) = &lane.cleanup_argv {
            validate_program(argv, &plan.binaries["engram"], "Engram cleanup")?;
        }
        if let Some(argv) = &lane.post_teaching_verification_argv {
            validate_program(
                argv,
                &plan.binaries["engram"],
                "Engram procedure verification",
            )?;
        }
    }
    Ok(())
}

fn validate_claude_execution_contract(plan: &PreparedNativePilot) -> EvalResult<()> {
    if plan.claude_model.is_empty() {
        return Ok(());
    }
    let allocated_microusd = u64::from(plan.claude_budget_cents) * 10_000;
    let authorized_microusd = u64::from(plan.claude_authorized_ceiling_cents) * 10_000;
    let accounted_microusd = plan
        .claude_prior_spend_microusd
        .checked_add(allocated_microusd)
        .ok_or_else(|| EvalError::Invalid("Claude budget accounting overflow".to_string()))?;
    if accounted_microusd > authorized_microusd {
        return Err(EvalError::Invalid(
            "Claude predecessor spend plus replacement allocation exceeds the authorized ceiling"
                .to_string(),
        ));
    }
    let claude_lane_count = plan
        .lanes
        .iter()
        .filter(|lane| lane.host == "claude_code")
        .count();
    let claude_calls = if let Some(recovery) = &plan.evaluation_recovery {
        let declared = usize::try_from(recovery.claude_remaining_calls).map_err(|_| {
            EvalError::Invalid("Claude recovery call count is too large".to_string())
        })?;
        if declared != claude_lane_count {
            return Err(EvalError::Invalid(
                "Claude recovery call count does not match the evaluation lanes".to_string(),
            ));
        }
        declared
    } else if let Some(recovery) = &plan.execution_recovery {
        usize::try_from(recovery.claude_remaining_calls).map_err(|_| {
            EvalError::Invalid("Claude recovery call count is too large".to_string())
        })?
    } else {
        claude_lane_count * 2
    };
    if claude_calls == 0 || plan.claude_max_turns == 0 {
        return Err(EvalError::Invalid(
            "frozen Claude execution contract has no calls or max-turn limit".to_string(),
        ));
    }
    let budget_millis = u64::from(plan.claude_budget_cents) * 10 / claude_calls as u64;
    for lane in plan.lanes.iter().filter(|lane| lane.host == "claude_code") {
        let native_memory_root = lane
            .artifact_gates
            .iter()
            .find(|gate| gate.layer == "claude_code_auto_memory")
            .map(|gate| Path::new(&gate.path))
            .and_then(Path::parent);
        let expected = [
            ("--permission-mode", "dontAsk".to_string()),
            (
                "--tools",
                if native_memory_root.is_some() {
                    "Read,Bash,Write,Edit"
                } else {
                    "Read,Bash"
                }
                .to_string(),
            ),
            (
                "--disallowed-tools",
                if native_memory_root.is_some() {
                    "WebFetch,WebSearch,NotebookEdit,Task"
                } else {
                    "Write,Edit,WebFetch,WebSearch,NotebookEdit,Task"
                }
                .to_string(),
            ),
            ("--model", plan.claude_model.clone()),
            ("--max-turns", plan.claude_max_turns.to_string()),
            ("--max-budget-usd", format_budget_usd(budget_millis)),
        ];
        if lane.claude_teaching_bash_commands.is_empty()
            || lane.claude_teaching_bash_commands.iter().any(|command| {
                command.trim().is_empty()
                    || command
                        .chars()
                        .any(|character| matches!(character, '\n' | '\r' | '*'))
                    || !command.starts_with("cd ")
                    || !command.contains(" && ")
            })
        {
            return Err(EvalError::Invalid(format!(
                "Claude lane {} does not freeze exact teaching-only Bash commands",
                lane.order
            )));
        }
        let mut commands = vec![(
            &lane.evaluation_argv,
            claude_allowed_tools(native_memory_root, &[])?,
        )];
        if plan.evaluation_recovery.is_none() {
            commands.insert(
                0,
                (
                    &lane.teaching_argv,
                    claude_allowed_tools(native_memory_root, &lane.claude_teaching_bash_commands)?,
                ),
            );
        }
        for (argv, allowed_tools) in commands {
            if argv
                .windows(2)
                .filter(|pair| pair[0] == "--allowed-tools" && pair[1] == allowed_tools.as_str())
                .count()
                != 1
            {
                return Err(EvalError::Invalid(format!(
                    "Claude lane {} argv does not freeze --allowed-tools={allowed_tools}",
                    lane.order
                )));
            }
            for (option, value) in &expected {
                if argv
                    .windows(2)
                    .filter(|pair| pair[0] == *option && pair[1] == value.as_str())
                    .count()
                    != 1
                {
                    return Err(EvalError::Invalid(format!(
                        "Claude lane {} argv does not freeze {option}={value}",
                        lane.order
                    )));
                }
            }
        }
    }
    Ok(())
}

fn validate_codex_authentication_contract(plan: &PreparedNativePilot) -> EvalResult<()> {
    let codex = plan.binaries.get("codex").ok_or_else(|| {
        EvalError::Invalid("run plan is missing codex binary attestation".to_string())
    })?;
    let mut codex_homes = BTreeSet::new();
    let mut requires_code_mode_host = false;
    for lane in &plan.lanes {
        if lane.host != "codex" {
            if lane.codex_authentication.is_some() {
                return Err(EvalError::Invalid(format!(
                    "non-Codex lane {} contains a Codex authentication contract",
                    lane.order
                )));
            }
            continue;
        }
        let Some(authentication) = lane.codex_authentication.as_ref() else {
            if lane
                .required_secret_environment
                .iter()
                .any(|name| name == "CODEX_ACCESS_TOKEN")
            {
                continue;
            }
            return Err(EvalError::Invalid(format!(
                "Codex lane {} is missing an authentication contract",
                lane.order
            )));
        };
        validate_codex_lane_authentication(lane, authentication, codex)?;
        let home = lane.environment.get("CODEX_HOME").ok_or_else(|| {
            EvalError::Invalid(format!("Codex lane {} is missing CODEX_HOME", lane.order))
        })?;
        let canonical_home = Path::new(home).canonicalize()?;
        if !codex_homes.insert(canonical_home.clone()) {
            return Err(EvalError::Invalid(format!(
                "Codex lane {} reuses an isolated CODEX_HOME",
                lane.order
            )));
        }
        if authentication.mode == CodexAuthenticationMode::ChatgptFileCache {
            requires_code_mode_host = true;
            validate_codex_file_cache(&canonical_home)?;
        } else if canonical_home.join("auth.json").exists() {
            return Err(EvalError::Invalid(format!(
                "Codex lane {} contains auth.json in a keyring authentication plan",
                lane.order
            )));
        }
        let credential_store_config = authentication.mode.credential_store_config();
        for argv in [
            &lane.teaching_argv,
            lane.activation_argv.as_ref().ok_or_else(|| {
                EvalError::Invalid(format!(
                    "Codex lane {} has no activation command",
                    lane.order
                ))
            })?,
            &lane.evaluation_argv,
        ] {
            if argv
                .windows(2)
                .filter(|pair| pair[0] == "--config" && pair[1] == credential_store_config)
                .count()
                != 1
            {
                return Err(EvalError::Invalid(format!(
                    "Codex lane {} provider argv does not freeze {} credential storage",
                    lane.order, authentication.credential_store,
                )));
            }
        }
        if lane
            .required_secret_environment
            .iter()
            .any(|name| matches!(name.as_str(), "CODEX_ACCESS_TOKEN" | "OPENAI_API_KEY"))
        {
            return Err(EvalError::Invalid(format!(
                "Codex lane {} mixes ChatGPT login with token or API-key requirements",
                lane.order
            )));
        }
    }
    if requires_code_mode_host && !plan.binaries.contains_key("codex_code_mode_host") {
        return Err(EvalError::Invalid(
            "chatgpt_file_cache plans must attest the sibling codex_code_mode_host binary"
                .to_string(),
        ));
    }
    Ok(())
}

fn validate_codex_file_cache(codex_home: &Path) -> EvalResult<bool> {
    #[cfg(not(unix))]
    {
        let _ = codex_home;
        return Err(EvalError::Invalid(
            "Codex file-cache authentication requires Unix permission checks".to_string(),
        ));
    }

    #[cfg(unix)]
    {
        use std::io::ErrorKind;
        use std::os::unix::fs::MetadataExt;

        let home_metadata = fs::symlink_metadata(codex_home)?;
        if home_metadata.file_type().is_symlink() || !home_metadata.is_dir() {
            return Err(EvalError::Invalid(format!(
                "Codex file-cache home is not a real directory: {}",
                codex_home.display()
            )));
        }
        if home_metadata.mode() & 0o777 != 0o700 {
            return Err(EvalError::Invalid(format!(
                "Codex file-cache home must have mode 0700: {}",
                codex_home.display()
            )));
        }

        let auth_path = codex_home.join("auth.json");
        match fs::symlink_metadata(&auth_path) {
            Ok(_) => {
                validate_codex_auth_cache_file(&auth_path, Some(home_metadata.uid()))?;
                Ok(true)
            }
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
            Err(error) => Err(EvalError::Io(error)),
        }
    }
}

fn validate_codex_auth_cache_file(path: &Path, expected_owner: Option<u32>) -> EvalResult<()> {
    #[cfg(not(unix))]
    {
        let _ = (path, expected_owner);
        return Err(EvalError::Invalid(
            "Codex file-cache authentication requires Unix permission checks".to_string(),
        ));
    }

    #[cfg(unix)]
    {
        let file = open_validated_codex_auth_cache(path, expected_owner)?;
        drop(read_validated_codex_auth_cache(file, path)?);
        Ok(())
    }
}

#[cfg(unix)]
fn open_validated_codex_auth_cache(path: &Path, expected_owner: Option<u32>) -> EvalResult<File> {
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt};

    let mut options = OpenOptions::new();
    options
        .read(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW);
    let file = options.open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(EvalError::Invalid(format!(
            "Codex auth cache must be a regular non-symlink file: {}",
            path.display()
        )));
    }
    if metadata.mode() & 0o777 != 0o600 {
        return Err(EvalError::Invalid(format!(
            "Codex auth cache must have mode 0600: {}",
            path.display()
        )));
    }
    if expected_owner.is_some_and(|owner| metadata.uid() != owner) || metadata.nlink() != 1 {
        return Err(EvalError::Invalid(format!(
            "Codex auth cache must have the expected owner and exactly one hard link: {}",
            path.display()
        )));
    }
    if metadata.len() == 0 || metadata.len() > MAX_CODEX_AUTH_FILE_BYTES {
        return Err(EvalError::Invalid(format!(
            "Codex auth cache must be non-empty and at most {MAX_CODEX_AUTH_FILE_BYTES} bytes: {}",
            path.display()
        )));
    }
    Ok(file)
}

#[cfg(unix)]
fn read_validated_codex_auth_cache(mut file: File, path: &Path) -> EvalResult<Zeroizing<Vec<u8>>> {
    let mut contents = Zeroizing::new(Vec::new());
    (&mut file)
        .take(MAX_CODEX_AUTH_FILE_BYTES + 1)
        .read_to_end(&mut contents)?;
    if contents.is_empty() || contents.len() as u64 > MAX_CODEX_AUTH_FILE_BYTES {
        return Err(EvalError::Invalid(format!(
            "Codex auth cache must be non-empty and at most {MAX_CODEX_AUTH_FILE_BYTES} bytes: {}",
            path.display()
        )));
    }
    let value: Value = serde_json::from_slice(&contents)?;
    if !value.is_object() {
        return Err(EvalError::Invalid(format!(
            "Codex auth cache must contain a JSON object: {}",
            path.display()
        )));
    }
    Ok(contents)
}

fn validate_codex_lane_authentication(
    lane: &PreparedNativeLane,
    authentication: &PreparedCodexAuthentication,
    codex: &PilotBinaryAttestation,
) -> EvalResult<()> {
    let base_argv = vec![
        codex.path.clone(),
        "login".to_string(),
        "--config".to_string(),
        authentication.mode.credential_store_config().to_string(),
    ];
    let mut expected_login = base_argv.clone();
    if authentication.mode == CodexAuthenticationMode::ChatgptDeviceKeyring {
        expected_login.push("--device-auth".to_string());
    }
    let mut expected_status = base_argv;
    expected_status.push("status".to_string());
    if authentication.credential_store != authentication.mode.credential_store()
        || authentication.expected_status != CODEX_CHATGPT_LOGIN_STATUS
        || authentication.login_argv != expected_login
        || authentication.status_argv != expected_status
    {
        return Err(EvalError::Invalid(format!(
            "Codex lane {} has an incompatible ChatGPT authentication contract",
            lane.order
        )));
    }
    validate_program(
        &authentication.login_argv,
        codex,
        &format!("lane {} Codex ChatGPT login", lane.order),
    )?;
    validate_program(
        &authentication.status_argv,
        codex,
        &format!("lane {} Codex login status", lane.order),
    )
}

fn inspect_codex_authentication(
    plan: &PreparedNativePilot,
    run_plan: &Path,
) -> EvalResult<NativePilotAuthReport> {
    inspect_codex_authentication_impl(plan, run_plan, None, false)
}

fn inspect_codex_authentication_bounded(
    plan: &PreparedNativePilot,
    run_plan: &Path,
) -> EvalResult<NativePilotAuthReport> {
    inspect_codex_authentication_impl(plan, run_plan, None, true)
}

fn inspect_codex_authentication_with_manifest(
    plan: &PreparedNativePilot,
    run_plan: &Path,
    manifest_sha256: Option<String>,
) -> EvalResult<NativePilotAuthReport> {
    inspect_codex_authentication_impl(plan, run_plan, manifest_sha256, true)
}

fn inspect_codex_authentication_impl(
    plan: &PreparedNativePilot,
    run_plan: &Path,
    manifest_sha256: Option<String>,
    bounded_status: bool,
) -> EvalResult<NativePilotAuthReport> {
    let mut lanes = Vec::new();
    for lane in plan.lanes.iter().filter(|lane| lane.host == "codex") {
        let authentication = lane.codex_authentication.as_ref().ok_or_else(|| {
            EvalError::Invalid(format!(
                "Codex lane {} does not use a supported ChatGPT authentication contract",
                lane.order
            ))
        })?;
        let codex_home = lane.environment.get("CODEX_HOME").ok_or_else(|| {
            EvalError::Invalid(format!("Codex lane {} is missing CODEX_HOME", lane.order))
        })?;
        lanes.push(NativePilotCodexLaneAuth {
            order: lane.order,
            arm: lane.arm.clone(),
            codex_home: codex_home.clone(),
            state: inspect_codex_lane_authentication(lane, authentication, bounded_status),
        });
    }
    let ready = lanes
        .iter()
        .all(|lane| lane.state == NativePilotCodexAuthState::Ready);
    Ok(NativePilotAuthReport {
        pilot_id: plan.pilot_id.clone(),
        run_plan: run_plan.display().to_string(),
        provider_free: true,
        ready,
        auth_cache_destinations: if manifest_sha256.is_some() {
            lanes
                .iter()
                .map(|lane| {
                    Path::new(&lane.codex_home)
                        .join("auth.json")
                        .display()
                        .to_string()
                })
                .collect()
        } else {
            Vec::new()
        },
        auth_cache_lifecycle_manifest_sha256: manifest_sha256,
        lanes,
    })
}

fn inspect_codex_lane_authentication(
    lane: &PreparedNativeLane,
    authentication: &PreparedCodexAuthentication,
    bounded_status: bool,
) -> NativePilotCodexAuthState {
    if authentication.mode == CodexAuthenticationMode::ChatgptFileCache {
        let Some(codex_home) = lane.environment.get("CODEX_HOME") else {
            return NativePilotCodexAuthState::CheckFailed;
        };
        match validate_codex_file_cache(Path::new(codex_home)) {
            Ok(true) => {}
            Ok(false) => return NativePilotCodexAuthState::NotLoggedIn,
            Err(_) => return NativePilotCodexAuthState::CheckFailed,
        }
    }
    let Some((program, args)) = authentication.status_argv.split_first() else {
        return NativePilotCodexAuthState::CheckFailed;
    };
    if bounded_status {
        let output = run_bounded_codex_auth_status(
            program,
            args,
            Path::new(&lane.teaching_cwd),
            &lane.environment,
            CODEX_AUTH_STATUS_TIMEOUT,
            MAX_CODEX_AUTH_STATUS_OUTPUT_BYTES,
        );
        let Ok(output) = output else {
            return NativePilotCodexAuthState::CheckFailed;
        };
        classify_codex_authentication(output.success, output.stdout.trim(), output.stderr.trim())
    } else {
        let output = Command::new(program)
            .args(args)
            .current_dir(&lane.teaching_cwd)
            .envs(&lane.environment)
            .env_remove("CODEX_ACCESS_TOKEN")
            .env_remove("OPENAI_API_KEY")
            .stdin(Stdio::null())
            .output();
        let Ok(output) = output else {
            return NativePilotCodexAuthState::CheckFailed;
        };
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        classify_codex_authentication(output.status.success(), stdout.trim(), stderr.trim())
    }
}

#[derive(Debug, PartialEq, Eq)]
struct BoundedCodexAuthStatus {
    success: bool,
    stdout: String,
    stderr: String,
}

fn run_bounded_codex_auth_status(
    program: &str,
    args: &[String],
    cwd: &Path,
    environment: &BTreeMap<String, String>,
    timeout: Duration,
    output_limit: usize,
) -> EvalResult<BoundedCodexAuthStatus> {
    if timeout.is_zero() || output_limit == 0 || output_limit > MAX_CODEX_AUTH_STATUS_OUTPUT_BYTES {
        return Err(EvalError::Invalid(
            "Codex authentication status bounds are invalid".to_string(),
        ));
    }
    let response_deadline = Instant::now()
        .checked_add(timeout)
        .ok_or_else(|| EvalError::Invalid("Codex auth status deadline overflow".to_string()))?;
    let terminal_deadline = response_deadline
        .checked_add(CODEX_AUTH_STATUS_CLEANUP_RESERVE)
        .ok_or_else(|| EvalError::Invalid("Codex auth cleanup deadline overflow".to_string()))?;
    let mut command = Command::new(program);
    command
        .args(args)
        .current_dir(cwd)
        .env_clear()
        .envs(environment)
        .env_remove("CODEX_ACCESS_TOKEN")
        .env_remove("OPENAI_API_KEY")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut guard = match NativeExecutionChildGuard::spawn(command, terminal_deadline) {
        NativeExecutionSpawnOutcome::Spawned(guard) => guard,
        NativeExecutionSpawnOutcome::NotSpawned(_)
        | NativeExecutionSpawnOutcome::PostSpawnTerminal { .. }
        | NativeExecutionSpawnOutcome::PostSpawnCleanupUnproven { .. } => {
            return Err(EvalError::Invalid(
                "Codex authentication status process could not be started safely".to_string(),
            ))
        }
    };
    let stdout = guard
        .take_stdout()
        .map_err(|_| EvalError::Invalid("Codex auth stdout ownership failed".to_string()))?
        .ok_or_else(|| EvalError::Invalid("Codex auth stdout was unavailable".to_string()))?;
    let stderr = guard
        .take_stderr()
        .map_err(|_| EvalError::Invalid("Codex auth stderr ownership failed".to_string()))?
        .ok_or_else(|| EvalError::Invalid("Codex auth stderr was unavailable".to_string()))?;
    let stdout_reader =
        std::thread::spawn(move || read_bounded_status_stream(stdout, output_limit));
    let stderr_reader =
        std::thread::spawn(move || read_bounded_status_stream(stderr, output_limit));

    let mut timed_out = false;
    loop {
        match guard.exited_without_reap(response_deadline) {
            Ok(true) => break,
            Ok(false) if Instant::now() < response_deadline => {
                std::thread::sleep(Duration::from_millis(10));
            }
            Ok(false) => {
                timed_out = true;
                break;
            }
            Err(_) => {
                return Err(EvalError::Invalid(
                    "Codex authentication status observation failed".to_string(),
                ))
            }
        }
    }
    let status = match guard.terminate_group_and_reap_with_status_until(terminal_deadline) {
        NativeExecutionCleanupOutcome::ProvenTerminal(status) => status,
        NativeExecutionCleanupOutcome::CleanupUnproven(_) => {
            return Err(EvalError::Invalid(
                "Codex authentication status cleanup remained unproven".to_string(),
            ))
        }
    };
    let stdout = stdout_reader
        .join()
        .map_err(|_| EvalError::Invalid("Codex auth stdout collector failed".to_string()))??;
    let stderr = stderr_reader
        .join()
        .map_err(|_| EvalError::Invalid("Codex auth stderr collector failed".to_string()))??;
    if timed_out || stdout.len().saturating_add(stderr.len()) > output_limit {
        return Err(EvalError::Invalid(
            "Codex authentication status exceeded its time or output bound".to_string(),
        ));
    }
    let stdout = String::from_utf8(stdout)
        .map_err(|_| EvalError::Invalid("Codex auth stdout was not UTF-8".to_string()))?;
    let stderr = String::from_utf8(stderr)
        .map_err(|_| EvalError::Invalid("Codex auth stderr was not UTF-8".to_string()))?;
    Ok(BoundedCodexAuthStatus {
        success: status.success(),
        stdout,
        stderr,
    })
}

fn read_bounded_status_stream(mut stream: impl Read, output_limit: usize) -> EvalResult<Vec<u8>> {
    let mut output = Vec::new();
    (&mut stream)
        .take(output_limit as u64 + 1)
        .read_to_end(&mut output)?;
    if output.len() > output_limit {
        return Err(EvalError::Invalid(
            "Codex authentication status output exceeded its bound".to_string(),
        ));
    }
    Ok(output)
}

fn classify_codex_authentication(
    success: bool,
    stdout: &str,
    stderr: &str,
) -> NativePilotCodexAuthState {
    if success
        && [stdout, stderr]
            .into_iter()
            .any(|value| value == CODEX_CHATGPT_LOGIN_STATUS)
    {
        NativePilotCodexAuthState::Ready
    } else if [stdout, stderr]
        .into_iter()
        .any(|value| value == "Not logged in")
    {
        NativePilotCodexAuthState::NotLoggedIn
    } else if success {
        NativePilotCodexAuthState::WrongAuthenticationMode
    } else {
        NativePilotCodexAuthState::CheckFailed
    }
}

fn require_codex_authentication_ready(
    plan: &PreparedNativePilot,
    run_plan: &Path,
    bounded_status: bool,
) -> EvalResult<()> {
    let report = if bounded_status {
        inspect_codex_authentication_bounded(plan, run_plan)?
    } else {
        inspect_codex_authentication(plan, run_plan)?
    };
    if report.ready {
        return Ok(());
    }
    let blocked = report
        .lanes
        .iter()
        .filter(|lane| lane.state != NativePilotCodexAuthState::Ready)
        .map(|lane| format!("{}:{:?}", lane.order, lane.state))
        .collect::<Vec<_>>()
        .join(", ");
    Err(EvalError::Invalid(format!(
        "provider execution is blocked until every isolated Codex lane reports its frozen ChatGPT authentication mode ready; lanes={blocked}"
    )))
}

fn validate_frozen_engram_contract(
    expected: &crate::native_pilot::EngramMcpContractAttestation,
    actual: &crate::native_pilot::EngramMcpContractAttestation,
) -> EvalResult<()> {
    if actual != expected {
        return Err(EvalError::Invalid(format!(
            "Engram agent-profile MCP contract drifted after preparation: expected tools_sha256={}, instructions_sha256={:?}; actual tools_sha256={}, instructions_sha256={:?}",
            expected.mcp_tools_sha256,
            expected.profile_instructions_sha256,
            actual.mcp_tools_sha256,
            actual.profile_instructions_sha256
        )));
    }
    Ok(())
}

/// Legacy historical entrypoints call the strict document loader before this path-based check.
/// That ordering is a firewall-before-side-effects guarantee, not immutable-plan authority against
/// a same-UID process racing later path reads. Recovery source plans use
/// `validate_loaded_plan_digest` instead so their already-bounded bytes are never read again by a
/// path-following digest operation.
fn validate_plan_digest(run_plan: &Path) -> EvalResult<()> {
    let expected_path = run_plan
        .parent()
        .ok_or_else(|| EvalError::Invalid("run plan has no parent".to_string()))?
        .join("run-plan.sha256");
    let expected = fs::read_to_string(&expected_path).map_err(|error| {
        EvalError::Invalid(format!(
            "missing frozen run-plan digest {}: {error}",
            expected_path.display()
        ))
    })?;
    let expected = expected.trim();
    if expected.len() != 64 || !expected.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(EvalError::Invalid(format!(
            "invalid run-plan digest in {}",
            expected_path.display()
        )));
    }
    let actual = sha256_file(run_plan)?;
    if !expected.eq_ignore_ascii_case(&actual) {
        return Err(EvalError::Invalid(format!(
            "run plan changed after preparation: expected sha256={expected}, actual sha256={actual}"
        )));
    }
    Ok(())
}

fn validate_loaded_plan_digest(loaded: &LoadedHistoricalNativePlan) -> EvalResult<()> {
    let expected_path = loaded
        .canonical_path
        .parent()
        .ok_or_else(|| EvalError::Invalid("run plan has no parent".to_string()))?
        .join("run-plan.sha256");
    let expected =
        read_bounded_owned_text_file(&expected_path, 128, "frozen run-plan digest sidecar")
            .map_err(|error| {
                EvalError::Invalid(format!(
                    "invalid frozen run-plan digest {}: {error}",
                    expected_path.display()
                ))
            })?;
    let expected = expected.trim();
    if expected.len() != 64 || !expected.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(EvalError::Invalid(format!(
            "invalid run-plan digest in {}",
            expected_path.display()
        )));
    }
    if !expected.eq_ignore_ascii_case(&loaded.sha256) {
        return Err(EvalError::Invalid(format!(
            "run plan changed after preparation: expected sha256={expected}, actual sha256={}",
            loaded.sha256
        )));
    }
    Ok(())
}

fn validate_program(
    argv: &[String],
    expected: &PilotBinaryAttestation,
    label: &str,
) -> EvalResult<()> {
    let program = argv
        .first()
        .ok_or_else(|| EvalError::Invalid(format!("{label} argv is empty")))?;
    let resolved = Path::new(program).canonicalize()?;
    if resolved != Path::new(&expected.path) {
        return Err(EvalError::Invalid(format!(
            "{label} program does not match attested binary: {} != {}",
            resolved.display(),
            expected.path
        )));
    }
    Ok(())
}

fn validate_required_secrets(plan: &PreparedNativePilot) -> EvalResult<()> {
    let missing = plan
        .lanes
        .iter()
        .flat_map(|lane| lane.required_secret_environment.iter())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|name| std::env::var_os(name.as_str()).map_or(true, |value| value.is_empty()))
        .cloned()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(EvalError::Invalid(format!(
            "required secret environment is missing or empty: {}",
            missing.join(", ")
        )));
    }
    Ok(())
}

fn validate_engram_disk_headroom(
    plan: &PreparedNativePilot,
    audit: &NativePilotAudit,
    phase: NativePilotRunPhase,
) -> EvalResult<()> {
    let mut checked_filesystems = BTreeSet::new();
    for lane in &plan.lanes {
        if !lane.memory_layer.uses_engram()
            || (phase == NativePilotRunPhase::Activation && lane.host != "codex")
        {
            continue;
        }
        let current = audit
            .lanes
            .iter()
            .find(|current| current.order == lane.order)
            .ok_or_else(|| {
                EvalError::Invalid(format!("pilot audit omitted lane {}", lane.order))
            })?;
        if lane_run_action(plan, lane, current.phase, phase)? != LaneRunAction::Execute {
            continue;
        }
        let engram_home = lane.engram_home.as_deref().ok_or_else(|| {
            EvalError::Invalid(format!("Engram lane {} is missing ENGRAM_HOME", lane.order))
        })?;
        let intended_path = Path::new(engram_home);
        let probe_path = intended_path
            .ancestors()
            .find(|candidate| candidate.exists())
            .ok_or_else(|| {
                EvalError::Invalid(format!(
                    "Engram disk-headroom preflight could not resolve an existing filesystem for lane {} at {}",
                    lane.order,
                    intended_path.display()
                ))
            })?;
        let probe_path = probe_path.canonicalize()?;
        if !checked_filesystems.insert(probe_path.clone()) {
            continue;
        }
        let headroom = engram_store::disk_headroom(&probe_path).map_err(|error| {
            EvalError::Invalid(format!(
                "Engram disk-headroom preflight failed before provider execution for lane {} at {}: {error}",
                lane.order,
                intended_path.display()
            ))
        })?;
        require_engram_disk_headroom(lane.order, intended_path, headroom)?;
    }
    Ok(())
}

fn require_engram_disk_headroom(
    lane_order: u32,
    intended_path: &Path,
    headroom: engram_store::DiskHeadroom,
) -> EvalResult<()> {
    if !headroom.is_sufficient() {
        return Err(EvalError::Invalid(format!(
            "provider execution is blocked before lane {lane_order}: insufficient free disk space at {} ({} bytes available; Engram requires at least {} bytes)",
            intended_path.display(),
            headroom.available_bytes,
            headroom.required_bytes
        )));
    }
    Ok(())
}

fn validate_phase_preconditions(
    plan: &PreparedNativePilot,
    audit: &NativePilotAudit,
    phase: NativePilotRunPhase,
) -> EvalResult<()> {
    if audit.invalid {
        if phase == NativePilotRunPhase::Evaluation {
            validate_recoverable_evaluation_audit(plan, audit)?;
        } else {
            return Err(EvalError::Invalid(
                "pilot audit is invalid; provider execution is blocked".to_string(),
            ));
        }
    }
    let stale_safety = plan.stale_safety.is_some();
    for lane in &plan.lanes {
        if phase == NativePilotRunPhase::Activation && lane.host != "codex" {
            continue;
        }
        let current = audit
            .lanes
            .iter()
            .find(|current| current.order == lane.order)
            .ok_or_else(|| {
                EvalError::Invalid(format!("pilot audit omitted lane {}", lane.order))
            })?;
        let action = lane_run_action(plan, lane, current.phase, phase)?;
        if stale_safety && action != LaneRunAction::Execute {
            return Err(EvalError::Invalid(format!(
                "stale-safety {} phase forbids lane recovery or replay; lane {} is already {:?}",
                phase.label(),
                lane.order,
                current.phase
            )));
        }
    }
    Ok(())
}

fn validate_phase_targets_absent(
    plan: &PreparedNativePilot,
    audit: &NativePilotAudit,
    phase: NativePilotRunPhase,
) -> EvalResult<()> {
    for lane in &plan.lanes {
        if phase == NativePilotRunPhase::Activation && lane.host != "codex" {
            continue;
        }
        let current = audit
            .lanes
            .iter()
            .find(|current| current.order == lane.order)
            .ok_or_else(|| {
                EvalError::Invalid(format!("pilot audit omitted lane {}", lane.order))
            })?;
        if lane_run_action(plan, lane, current.phase, phase)? != LaneRunAction::Execute {
            continue;
        }
        let trace = trace_path(lane, phase)?;
        require_absent(trace, "provider trace")?;
        require_absent(&stderr_path(trace), "provider stderr")?;
        match phase {
            NativePilotRunPhase::Teaching if lane.memory_layer.uses_engram() => {
                let verification = lane
                    .post_teaching_verification_output_path
                    .as_deref()
                    .ok_or_else(|| {
                        EvalError::Invalid(format!(
                            "lane {} is missing procedure verification output path",
                            lane.order
                        ))
                    })?;
                require_absent(Path::new(verification), "procedure verification output")?;
                require_absent(
                    &verification_stderr_path(Path::new(verification)),
                    "verification stderr",
                )?;
                require_absent(&candidate_path(lane)?, "procedure candidate review")?;
                require_absent(&candidate_stderr_path(lane)?, "procedure candidate stderr")?;
            }
            NativePilotRunPhase::Evaluation => {
                require_absent(Path::new(&lane.agent_output_path), "agent output")?;
            }
            _ => {}
        }
        if lane.cleanup_argv.is_some() {
            require_absent(&cleanup_stdout_path(lane, phase)?, "Engram cleanup stdout")?;
            require_absent(&cleanup_stderr_path(lane, phase)?, "Engram cleanup stderr")?;
        }
    }
    Ok(())
}

fn lane_run_action(
    plan: &PreparedNativePilot,
    lane: &PreparedNativeLane,
    current: NativeLanePhase,
    phase: NativePilotRunPhase,
) -> EvalResult<LaneRunAction> {
    let action = match phase {
        NativePilotRunPhase::Teaching => match current {
            NativeLanePhase::Prepared => LaneRunAction::Execute,
            NativeLanePhase::AwaitingVerification if lane.memory_layer.uses_engram() => {
                if is_procedure_teaching_scope_recovery(plan, lane) {
                    validate_completed_claude_teaching_trace(Path::new(&lane.teaching_trace_path))?;
                } else {
                    validate_recoverable_claude_budget_trace(Path::new(&lane.teaching_trace_path))?;
                }
                LaneRunAction::RecoverTeaching
            }
            NativeLanePhase::AwaitingActivation if lane.host == "codex" => LaneRunAction::Skip,
            NativeLanePhase::ReadyForEvaluation if lane.host != "codex" => LaneRunAction::Skip,
            _ => {
                return Err(EvalError::Invalid(format!(
                    "{} phase precondition failed for lane {}: phase is {:?}",
                    phase.label(),
                    lane.order,
                    current
                )))
            }
        },
        NativePilotRunPhase::Activation => {
            if lane.host != "codex" {
                if current == NativeLanePhase::ReadyForEvaluation {
                    LaneRunAction::Skip
                } else {
                    return Err(EvalError::Invalid(format!(
                        "{} phase precondition failed for lane {}: phase is {:?}",
                        phase.label(),
                        lane.order,
                        current
                    )));
                }
            } else {
                match current {
                    NativeLanePhase::AwaitingActivation => LaneRunAction::Execute,
                    NativeLanePhase::ReadyForEvaluation => LaneRunAction::Skip,
                    _ => {
                        return Err(EvalError::Invalid(format!(
                            "{} phase precondition failed for lane {}: phase is {:?}",
                            phase.label(),
                            lane.order,
                            current
                        )))
                    }
                }
            }
        }
        NativePilotRunPhase::Evaluation => match current {
            NativeLanePhase::ReadyForEvaluation => LaneRunAction::Execute,
            NativeLanePhase::EvaluationComplete => LaneRunAction::Skip,
            NativeLanePhase::Invalid if lane.host == "claude_code" => {
                validate_recoverable_claude_evaluation_trace(
                    Path::new(&lane.evaluation_trace_path),
                    plan.protocol_schema_version,
                )?;
                LaneRunAction::RecoverEvaluation
            }
            _ => {
                return Err(EvalError::Invalid(format!(
                    "{} phase precondition failed for lane {}: phase is {:?}",
                    phase.label(),
                    lane.order,
                    current
                )))
            }
        },
    };
    Ok(action)
}

fn is_procedure_teaching_scope_recovery(
    plan: &PreparedNativePilot,
    lane: &PreparedNativeLane,
) -> bool {
    plan.execution_recovery.as_ref().is_some_and(|recovery| {
        recovery.failure_reason == PROCEDURE_TEACHING_SCOPE_REJECTION
            && recovery.completed_lane_orders.contains(&lane.order)
    })
}

fn is_family_v3_plan(plan: &PreparedNativePilot) -> bool {
    plan.stale_safety
        .as_ref()
        .is_some_and(|binding| binding.family_version == CODEX_AUTH_CACHE_LIFECYCLE_FAMILY_VERSION)
}

fn attest_family_v3_claude_config_artifacts(
    plan: &PreparedNativePilot,
) -> EvalResult<BTreeMap<u32, NativeStaleV3ClaudeConfigAttestation>> {
    if !is_family_v3_plan(plan) {
        return Ok(BTreeMap::new());
    }
    let mut attestations = BTreeMap::new();
    for lane in &plan.lanes {
        if lane.host != "claude_code" {
            continue;
        }
        let attestation = validate_native_stale_v3_claude_config_artifacts(lane)?;
        if attestations.insert(lane.order, attestation).is_some() {
            return Err(EvalError::Invalid(format!(
                "family-v3 Claude config attestation duplicated lane {}",
                lane.order
            )));
        }
    }
    if attestations.len()
        != plan
            .lanes
            .iter()
            .filter(|lane| lane.host == "claude_code")
            .count()
    {
        return Err(EvalError::Invalid(
            "family-v3 Claude config attestation matrix is incomplete".to_string(),
        ));
    }
    Ok(attestations)
}

/// Run a continuation only after every static family-v3 Claude configuration has passed the
/// phase-wide preflight. This keeps phase intent creation and every later admission behind one
/// testable all-lane barrier.
fn with_family_v3_claude_config_preflight<T>(
    plan: &PreparedNativePilot,
    continuation: impl FnOnce(BTreeMap<u32, NativeStaleV3ClaudeConfigAttestation>) -> EvalResult<T>,
) -> EvalResult<T> {
    let attestations = attest_family_v3_claude_config_artifacts(plan)?;
    continuation(attestations)
}

fn revalidate_family_v3_lane_claude_config_artifacts<'a>(
    lane: &PreparedNativeLane,
    attestations: &'a BTreeMap<u32, NativeStaleV3ClaudeConfigAttestation>,
) -> EvalResult<Option<&'a NativeStaleV3ClaudeConfigAttestation>> {
    if lane.host != "claude_code" {
        if attestations.contains_key(&lane.order) {
            return Err(EvalError::Invalid(format!(
                "non-Claude lane {} has a Claude config attestation",
                lane.order
            )));
        }
        return Ok(None);
    }
    let expected = attestations.get(&lane.order).ok_or_else(|| {
        EvalError::Invalid(format!(
            "Claude lane {} omitted its family-v3 config attestation",
            lane.order
        ))
    })?;
    if validate_native_stale_v3_claude_config_artifacts(lane)? != *expected {
        return Err(EvalError::Invalid(format!(
            "Claude lane {} config artifacts changed after phase preflight",
            lane.order
        )));
    }
    Ok(Some(expected))
}

fn execute_lane(
    plan: &PreparedNativePilot,
    lane: &PreparedNativeLane,
    phase: NativePilotRunPhase,
    auth_lifecycle: Option<&CodexAuthCacheLifecycle>,
    claude_config_artifacts: Option<&NativeStaleV3ClaudeConfigAttestation>,
    revalidate_bundle_dispatch_boundary: impl FnOnce() -> EvalResult<()>,
) -> Result<NativePilotLaneExecution, ProviderLaneFailure> {
    if lane.host == "codex" {
        let authentication = lane.codex_authentication.as_ref().ok_or_else(|| {
            EvalError::Invalid(format!(
                "Codex lane {} is missing an authentication contract",
                lane.order
            ))
        })?;
        if inspect_codex_lane_authentication(lane, authentication, auth_lifecycle.is_some())
            != NativePilotCodexAuthState::Ready
        {
            return Err(ProviderLaneFailure::pre_dispatch_runner_failure(
                EvalError::Invalid(format!(
                    "Codex lane {} lost its frozen ChatGPT authentication before provider execution",
                    lane.order
                )),
            ));
        }
        if let Some(lifecycle) = auth_lifecycle {
            lifecycle
                .validate_current_destinations(plan)
                .map_err(ProviderLaneFailure::pre_dispatch_runner_failure)?;
        }
    }
    if phase == NativePilotRunPhase::Activation {
        validate_activation_interval(lane)
            .map_err(ProviderLaneFailure::pre_dispatch_runner_failure)?;
    }
    let argv = match phase {
        NativePilotRunPhase::Teaching => &lane.teaching_argv,
        NativePilotRunPhase::Activation => lane
            .activation_argv
            .as_ref()
            .ok_or_else(|| {
                EvalError::Invalid(format!("lane {} has no activation command", lane.order))
            })
            .map_err(ProviderLaneFailure::pre_dispatch_runner_failure)?,
        NativePilotRunPhase::Evaluation => &lane.evaluation_argv,
    };
    let cwd = match phase {
        NativePilotRunPhase::Teaching => Path::new(&lane.teaching_cwd),
        NativePilotRunPhase::Activation | NativePilotRunPhase::Evaluation => {
            Path::new(&lane.evaluation_cwd)
        }
    };
    let trace =
        trace_path(lane, phase).map_err(ProviderLaneFailure::pre_dispatch_runner_failure)?;
    let stderr = stderr_path(trace);
    let family_v3 = is_family_v3_plan(plan);
    let claude_config_artifacts_sha256 = match (family_v3, lane.host.as_str()) {
        (true, "claude_code") => {
            let expected = claude_config_artifacts.ok_or_else(|| {
                ProviderLaneFailure::pre_dispatch_config_artifact_drift(EvalError::Invalid(
                    format!(
                        "Claude lane {} omitted its pre-admission config attestation",
                        lane.order
                    ),
                ))
            })?;
            let observed = validate_native_stale_v3_claude_config_artifacts(lane)
                .map_err(ProviderLaneFailure::pre_dispatch_config_artifact_drift)?;
            if observed != *expected {
                return Err(ProviderLaneFailure::pre_dispatch_config_artifact_drift(
                    EvalError::Invalid(format!(
                        "Claude lane {} config artifacts changed before provider spawn",
                        lane.order
                    )),
                ));
            }
            Some(expected.aggregate_sha256.clone())
        }
        (_, _) if claude_config_artifacts.is_some() => {
            return Err(ProviderLaneFailure::pre_dispatch_config_artifact_drift(
                EvalError::Invalid(format!(
                    "lane {} received an inapplicable Claude config attestation",
                    lane.order
                )),
            ))
        }
        _ => None,
    };
    let effective_environment_sha256 = if family_v3 {
        Some(
            canonical_environment_sha256(&lane.environment)
                .map_err(ProviderLaneFailure::pre_dispatch_runner_failure)?,
        )
    } else {
        None
    };
    let provider_started_unix_ms =
        unix_ms(SystemTime::now()).map_err(ProviderLaneFailure::pre_dispatch_runner_failure)?;
    let outcome = run_provider_command(
        lane,
        phase,
        argv,
        cwd,
        (trace, &stderr),
        plan.protocol_schema_version,
        family_v3,
        revalidate_bundle_dispatch_boundary,
    )?;
    let provider_completed_unix_ms =
        unix_ms(SystemTime::now()).map_err(ProviderLaneFailure::post_dispatch_failure)?;
    let post_provider = (|| -> EvalResult<NativePilotLaneExecution> {
        if family_v3 {
            validate_native_pilot_family_v3_runtime_libraries(
                &plan.binaries,
                &plan.runtime_libraries,
            )?;
        }
        if let Some(expected) = claude_config_artifacts {
            if validate_native_stale_v3_claude_config_artifacts(lane)? != *expected {
                return Err(EvalError::Invalid(format!(
                    "Claude lane {} config artifacts changed during provider execution",
                    lane.order
                )));
            }
        }
        if phase == NativePilotRunPhase::Teaching {
            validate_teaching_command_evidence(lane, plan.protocol_schema_version)?;
        }
        if lane.cleanup_argv.is_some() {
            run_cleanup(lane, phase, family_v3)?;
        }
        if phase == NativePilotRunPhase::Teaching && lane.memory_layer.uses_engram() {
            verify_taught_procedure(plan, lane, false, family_v3)?;
        }
        if phase == NativePilotRunPhase::Evaluation {
            extract_agent_output(
                &lane.host,
                trace,
                Path::new(&lane.agent_output_path),
                plan.protocol_schema_version,
            )?;
        }

        let provider_trace_sha256 = family_v3.then(|| sha256_file(trace)).transpose()?;
        let claude_model = if family_v3 && lane.host == "claude_code" {
            Some(attest_claude_trace_model(trace, &plan.claude_model)?)
        } else {
            None
        };

        Ok(NativePilotLaneExecution {
            order: lane.order,
            arm: lane.arm.clone(),
            host: lane.host.clone(),
            trace_path: trace.display().to_string(),
            stderr_path: stderr.display().to_string(),
            argv_sha256: argv_sha256(argv)?,
            effective_environment_sha256,
            claude_config_artifacts_sha256,
            stdout_trace_bytes: outcome.safety.map(|value| value.stdout_trace_bytes),
            stderr_bytes: outcome.safety.map(|value| value.stderr_bytes),
            process_cleanup_proven: outcome.safety.map(|value| value.process_cleanup_proven),
            provider_started_unix_ms: Some(provider_started_unix_ms),
            provider_completed_unix_ms: Some(provider_completed_unix_ms),
            exit_code: outcome.exit_code,
            codex_session_rollout_path: None,
            codex_session_rollout_sha256: None,
            codex_model_provider: None,
            codex_host_resolved_model: None,
            codex_agents_md_sha256: None,
            provider_trace_sha256,
            claude_requested_model: claude_model
                .as_ref()
                .map(|attestation| attestation.requested_model.clone()),
            claude_host_resolved_model: claude_model
                .as_ref()
                .map(|attestation| attestation.host_resolved_model.clone()),
            provider_reported_cost_microusd: outcome.provider_reported_cost_microusd,
            accepted_turn_boundary_budget_exit: outcome.accepted_turn_boundary_budget_exit,
            recovered_from_existing_trace: false,
        })
    })();
    if family_v3 {
        post_provider.map_err(|error| {
            ProviderLaneFailure::post_dispatch_failure(bind_post_dispatch_failure(
                error,
                outcome
                    .safety
                    .expect("family-v3 provider always uses bounded execution"),
            ))
        })
    } else {
        post_provider.map_err(ProviderLaneFailure::post_dispatch_failure)
    }
}

fn validate_recoverable_evaluation_audit(
    plan: &PreparedNativePilot,
    audit: &NativePilotAudit,
) -> EvalResult<()> {
    let invalid = audit
        .lanes
        .iter()
        .filter(|lane| lane.phase == NativeLanePhase::Invalid)
        .collect::<Vec<_>>();
    if invalid.len() != 1 {
        return Err(EvalError::Invalid(
            "evaluation recovery requires exactly one interrupted lane".to_string(),
        ));
    }
    let observed = invalid[0];
    let lane = plan
        .lanes
        .iter()
        .find(|lane| lane.order == observed.order)
        .ok_or_else(|| EvalError::Invalid("interrupted lane is not in the plan".to_string()))?;
    if lane.host != "claude_code"
        || observed.evaluation_trace.status != AuditStatus::Passed
        || observed.agent_output.status != AuditStatus::Pending
        || observed.failures != ["evaluation trace and structured output are incomplete"]
    {
        return Err(EvalError::Invalid(
            "pilot audit is invalid; provider execution is blocked".to_string(),
        ));
    }
    if audit.lanes.iter().any(|candidate| {
        candidate.order != observed.order
            && !matches!(
                candidate.phase,
                NativeLanePhase::ReadyForEvaluation | NativeLanePhase::EvaluationComplete
            )
    }) {
        return Err(EvalError::Invalid(
            "another lane is outside the evaluation lifecycle".to_string(),
        ));
    }
    validate_recoverable_claude_evaluation_trace(
        Path::new(&lane.evaluation_trace_path),
        plan.protocol_schema_version,
    )?;
    Ok(())
}

fn recover_interrupted_evaluation_lane(
    plan: &PreparedNativePilot,
    lane: &PreparedNativeLane,
    protocol_schema_version: u32,
) -> EvalResult<NativePilotLaneExecution> {
    let trace = Path::new(&lane.evaluation_trace_path);
    let summary = validate_recoverable_claude_evaluation_trace(trace, protocol_schema_version)?;
    require_absent(Path::new(&lane.agent_output_path), "agent output")?;
    if lane.cleanup_argv.is_some() {
        require_absent(
            &cleanup_stdout_path(lane, NativePilotRunPhase::Evaluation)?,
            "Engram cleanup stdout",
        )?;
        require_absent(
            &cleanup_stderr_path(lane, NativePilotRunPhase::Evaluation)?,
            "Engram cleanup stderr",
        )?;
        run_cleanup(
            lane,
            NativePilotRunPhase::Evaluation,
            is_family_v3_plan(plan),
        )?;
    }
    extract_agent_output(
        &lane.host,
        trace,
        Path::new(&lane.agent_output_path),
        protocol_schema_version,
    )?;

    Ok(NativePilotLaneExecution {
        order: lane.order,
        arm: lane.arm.clone(),
        host: lane.host.clone(),
        trace_path: trace.display().to_string(),
        stderr_path: stderr_path(trace).display().to_string(),
        argv_sha256: argv_sha256(&lane.evaluation_argv)?,
        effective_environment_sha256: None,
        claude_config_artifacts_sha256: None,
        stdout_trace_bytes: None,
        stderr_bytes: None,
        process_cleanup_proven: None,
        provider_started_unix_ms: None,
        provider_completed_unix_ms: None,
        exit_code: 1,
        codex_session_rollout_path: None,
        codex_session_rollout_sha256: None,
        codex_model_provider: None,
        codex_host_resolved_model: None,
        codex_agents_md_sha256: None,
        provider_trace_sha256: None,
        claude_requested_model: None,
        claude_host_resolved_model: None,
        provider_reported_cost_microusd: summary.provider_reported_cost_microusd,
        accepted_turn_boundary_budget_exit: true,
        recovered_from_existing_trace: true,
    })
}

fn recover_interrupted_teaching_lane(
    plan: &PreparedNativePilot,
    lane: &PreparedNativeLane,
) -> EvalResult<NativePilotLaneExecution> {
    let trace = Path::new(&lane.teaching_trace_path);
    let reuse_candidates = is_procedure_teaching_scope_recovery(plan, lane);
    let (summary, exit_code, accepted_turn_boundary_budget_exit) = if reuse_candidates {
        let summary = validate_completed_claude_teaching_trace(trace)?;
        if lane.cleanup_argv.is_some() {
            require_existing_file(
                &cleanup_stdout_path(lane, NativePilotRunPhase::Teaching)?,
                "Engram cleanup stdout",
            )?;
            require_existing_file(
                &cleanup_stderr_path(lane, NativePilotRunPhase::Teaching)?,
                "Engram cleanup stderr",
            )?;
        }
        require_existing_file(&candidate_path(lane)?, "procedure candidate review")?;
        require_existing_file(&candidate_stderr_path(lane)?, "procedure candidate stderr")?;
        (summary, 0, false)
    } else {
        let summary = validate_recoverable_claude_budget_trace(trace)?;
        if lane.cleanup_argv.is_some() {
            require_absent(
                &cleanup_stdout_path(lane, NativePilotRunPhase::Teaching)?,
                "Engram cleanup stdout",
            )?;
            require_absent(
                &cleanup_stderr_path(lane, NativePilotRunPhase::Teaching)?,
                "Engram cleanup stderr",
            )?;
            run_cleanup(lane, NativePilotRunPhase::Teaching, is_family_v3_plan(plan))?;
        }
        require_absent(&candidate_path(lane)?, "procedure candidate review")?;
        require_absent(&candidate_stderr_path(lane)?, "procedure candidate stderr")?;
        (summary, 1, true)
    };
    validate_teaching_command_evidence(lane, plan.protocol_schema_version)?;
    let verification = lane
        .post_teaching_verification_output_path
        .as_deref()
        .ok_or_else(|| {
            EvalError::Invalid(format!(
                "lane {} is missing procedure verification output path",
                lane.order
            ))
        })?;
    require_absent(Path::new(verification), "procedure verification output")?;
    require_absent(
        &verification_stderr_path(Path::new(verification)),
        "verification stderr",
    )?;
    verify_taught_procedure(plan, lane, reuse_candidates, is_family_v3_plan(plan))?;

    Ok(NativePilotLaneExecution {
        order: lane.order,
        arm: lane.arm.clone(),
        host: lane.host.clone(),
        trace_path: trace.display().to_string(),
        stderr_path: stderr_path(trace).display().to_string(),
        argv_sha256: argv_sha256(&lane.teaching_argv)?,
        effective_environment_sha256: None,
        claude_config_artifacts_sha256: None,
        stdout_trace_bytes: None,
        stderr_bytes: None,
        process_cleanup_proven: None,
        provider_started_unix_ms: None,
        provider_completed_unix_ms: None,
        exit_code,
        codex_session_rollout_path: None,
        codex_session_rollout_sha256: None,
        codex_model_provider: None,
        codex_host_resolved_model: None,
        codex_agents_md_sha256: None,
        provider_trace_sha256: None,
        claude_requested_model: None,
        claude_host_resolved_model: None,
        provider_reported_cost_microusd: summary.provider_reported_cost_microusd,
        accepted_turn_boundary_budget_exit,
        recovered_from_existing_trace: true,
    })
}

fn run_cleanup(
    lane: &PreparedNativeLane,
    phase: NativePilotRunPhase,
    bounded_closed_world: bool,
) -> EvalResult<()> {
    let argv = lane.cleanup_argv.as_ref().ok_or_else(|| {
        EvalError::Invalid(format!("lane {} is missing cleanup command", lane.order))
    })?;
    run_auxiliary_command(
        argv,
        Path::new(&lane.teaching_cwd),
        &lane.environment,
        &cleanup_stdout_path(lane, phase)?,
        &cleanup_stderr_path(lane, phase)?,
        bounded_closed_world,
    )?;
    Ok(())
}

fn verify_taught_procedure(
    plan: &PreparedNativePilot,
    lane: &PreparedNativeLane,
    reuse_candidates: bool,
    bounded_closed_world: bool,
) -> EvalResult<()> {
    let project = lane.engram_project.as_deref().ok_or_else(|| {
        EvalError::Invalid(format!("lane {} is missing Engram project", lane.order))
    })?;
    let review_argv = vec![
        plan.binaries["engram"].path.clone(),
        "memory".to_string(),
        "--project".to_string(),
        project.to_string(),
        "review".to_string(),
        "--json".to_string(),
    ];
    let candidates = candidate_path(lane)?;
    if reuse_candidates {
        require_existing_file(&candidates, "procedure candidate review")?;
        require_existing_file(&candidate_stderr_path(lane)?, "procedure candidate stderr")?;
    } else {
        run_auxiliary_command(
            &review_argv,
            Path::new(&lane.teaching_cwd),
            &lane.environment,
            &candidates,
            &candidate_stderr_path(lane)?,
            bounded_closed_world,
        )?;
    }
    let contract: RunnerAcceptanceContract =
        serde_json::from_reader(File::open(&lane.acceptance_contract)?)?;
    let teaching_remote = repository_remote(Path::new(&lane.teaching_cwd))?;
    let candidate_id = select_procedure_candidate(&candidates, &contract, &teaching_remote)?;
    if !fs::read_to_string(&lane.teaching_trace_path)?.contains(&candidate_id) {
        return Err(EvalError::Invalid(format!(
            "lane {} selected candidate ID does not occur in the teaching trace",
            lane.order
        )));
    }

    let template = lane
        .post_teaching_verification_argv
        .as_ref()
        .ok_or_else(|| {
            EvalError::Invalid(format!(
                "lane {} is missing verification command",
                lane.order
            ))
        })?;
    if template
        .iter()
        .filter(|arg| arg.as_str() == PROCEDURE_ID_PLACEHOLDER)
        .count()
        != 1
    {
        return Err(EvalError::Invalid(format!(
            "lane {} verification template must contain exactly one candidate placeholder",
            lane.order
        )));
    }
    let argv = template
        .iter()
        .map(|arg| {
            if arg == PROCEDURE_ID_PLACEHOLDER {
                candidate_id.clone()
            } else {
                arg.clone()
            }
        })
        .collect::<Vec<_>>();
    if argv.iter().any(|arg| arg.contains("${")) {
        return Err(EvalError::Invalid(format!(
            "lane {} verification command contains an unresolved placeholder",
            lane.order
        )));
    }
    let output = Path::new(
        lane.post_teaching_verification_output_path
            .as_deref()
            .ok_or_else(|| {
                EvalError::Invalid(format!(
                    "lane {} is missing verification output path",
                    lane.order
                ))
            })?,
    );
    run_auxiliary_command(
        &argv,
        Path::new(&lane.teaching_cwd),
        &lane.environment,
        output,
        &verification_stderr_path(output),
        bounded_closed_world,
    )?;
    Ok(())
}

fn select_procedure_candidate(
    path: &Path,
    contract: &RunnerAcceptanceContract,
    expected_repository_remote: &str,
) -> EvalResult<String> {
    let value: Value = serde_json::from_reader(File::open(path)?)?;
    let items = value.as_array().ok_or_else(|| {
        EvalError::Invalid("Engram review output must be a JSON array".to_string())
    })?;
    let context_keys = contract
        .required_context_keys
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let matches = items
        .iter()
        .filter(|item| {
            item.get("kind").and_then(Value::as_str) == Some("procedure")
                && item.get("status").and_then(Value::as_str) == Some("needs_review")
                && item.pointer("/scope/type").and_then(Value::as_str) == Some("repository")
                && item
                    .pointer("/scope/remote_url")
                    .and_then(Value::as_str)
                    .is_some_and(|remote| {
                        normalize_remote(remote) == normalize_remote(expected_repository_remote)
                    })
                && item.pointer("/procedure/task").and_then(Value::as_str)
                    == Some(contract.required_task.as_str())
                && item
                    .pointer("/procedure/commands")
                    .and_then(Value::as_array)
                    .is_some_and(|commands| {
                        commands.iter().any(|command| {
                            command.as_str() == Some(contract.required_command.as_str())
                        })
                    })
                && item
                    .pointer("/procedure/verification/expected_exit_code")
                    .and_then(Value::as_i64)
                    == Some(i64::from(contract.expected_exit_code))
                && item
                    .pointer("/procedure/verification/expected_output_contains")
                    .and_then(Value::as_str)
                    == Some(contract.expected_output_contains.as_str())
                && procedure_prerequisites(item)
                    .is_some_and(|actual| actual == contract.required_conditions)
                && procedure_prerequisite_sources(item)
                    .is_some_and(|actual| actual == contract.required_prerequisite_sources)
                && item
                    .pointer("/procedure/failure_signatures")
                    .and_then(Value::as_array)
                    .is_some_and(|signatures| {
                        contract.forbidden_commands.iter().all(|command| {
                            signatures
                                .iter()
                                .any(|signature| signature.as_str() == Some(command.as_str()))
                        })
                    })
                && serde_json::to_string(item).ok().is_some_and(|serialized| {
                    context_keys.iter().all(|key| serialized.contains(key))
                })
        })
        .filter_map(|item| item.get("id").and_then(Value::as_str))
        .map(str::to_string)
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(EvalError::Invalid(format!(
            "trusted evaluator expected exactly one matching procedure candidate, found {}",
            matches.len()
        )));
    }
    Ok(matches[0].clone())
}

fn repository_remote(cwd: &Path) -> EvalResult<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(["config", "--get", "remote.origin.url"])
        .output()?;
    if !output.status.success() {
        return Err(EvalError::Invalid(format!(
            "failed to resolve teaching repository remote at {}: {}",
            cwd.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    let remote = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if remote.is_empty() {
        return Err(EvalError::Invalid(format!(
            "teaching repository remote is empty at {}",
            cwd.display()
        )));
    }
    Ok(remote)
}

fn procedure_prerequisites(item: &Value) -> Option<BTreeMap<String, String>> {
    item.pointer("/procedure/prerequisites")?
        .as_array()?
        .iter()
        .map(|prerequisite| {
            Some((
                prerequisite.get("key")?.as_str()?.to_string(),
                prerequisite.get("expected")?.as_str()?.to_string(),
            ))
        })
        .collect()
}

fn procedure_prerequisite_sources(
    item: &Value,
) -> Option<BTreeMap<String, NativePilotPrerequisiteSource>> {
    let prerequisites = item.pointer("/procedure/prerequisites")?.as_array()?;
    let mut sources = BTreeMap::new();
    for prerequisite in prerequisites {
        let Some(source) = prerequisite.get("source") else {
            continue;
        };
        sources.insert(
            prerequisite.get("key")?.as_str()?.to_string(),
            serde_json::from_value(source.clone()).ok()?,
        );
    }
    Some(sources)
}

fn extract_agent_output(
    host: &str,
    trace: &Path,
    output: &Path,
    protocol_schema_version: u32,
) -> EvalResult<()> {
    let value = if host == "codex" {
        codex_terminal_agent_output(trace, protocol_schema_version)?
    } else {
        unique_agent_output(trace, protocol_schema_version)?
    };
    write_private_json(output, &value)
}

fn extract_control_agent_output(
    host: &str,
    trace: &Path,
    output: &Path,
    protocol_schema_version: u32,
) -> EvalResult<()> {
    let value = if host == "codex" {
        codex_terminal_agent_output_with_contract(
            trace,
            protocol_schema_version,
            NativeAgentOutputContract::StaleSafetyV3,
        )?
    } else {
        unique_agent_output_with_contract(
            trace,
            protocol_schema_version,
            NativeAgentOutputContract::StaleSafetyV3,
        )?
    };
    write_private_json(output, &value)
}

fn unique_agent_output(trace: &Path, protocol_schema_version: u32) -> EvalResult<Value> {
    let output_contract = agent_output_contract(trace, protocol_schema_version)?;
    unique_agent_output_with_contract(trace, protocol_schema_version, output_contract)
}

fn unique_agent_output_with_contract(
    trace: &Path,
    protocol_schema_version: u32,
    output_contract: NativeAgentOutputContract,
) -> EvalResult<Value> {
    let reader = BufReader::new(File::open(trace)?);
    let mut candidates = Vec::new();
    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(&line).map_err(|error| {
            EvalError::Invalid(format!(
                "evaluation trace line {} is invalid JSON: {error}",
                index + 1
            ))
        })?;
        collect_agent_outputs(
            &value,
            &mut candidates,
            protocol_schema_version,
            output_contract,
        );
    }
    let unique = distinct_agent_outputs(candidates)?;
    if unique.len() != 1 {
        return Err(EvalError::Invalid(format!(
            "evaluation trace must contain exactly one distinct structured agent output, found {}",
            unique.len()
        )));
    }
    Ok(unique.into_values().next().expect("checked one value"))
}

fn distinct_agent_output_count(trace: &Path, protocol_schema_version: u32) -> EvalResult<usize> {
    let output_contract = agent_output_contract(trace, protocol_schema_version)?;
    let reader = BufReader::new(File::open(trace)?);
    let mut candidates = Vec::new();
    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(&line).map_err(|error| {
            EvalError::Invalid(format!(
                "evaluation trace line {} is invalid JSON: {error}",
                index + 1
            ))
        })?;
        collect_agent_outputs(
            &value,
            &mut candidates,
            protocol_schema_version,
            output_contract,
        );
    }
    Ok(distinct_agent_outputs(candidates)?.len())
}

fn distinct_agent_outputs(candidates: Vec<Value>) -> EvalResult<BTreeMap<String, Value>> {
    let mut unique = BTreeMap::new();
    for candidate in candidates {
        unique.insert(serde_json::to_string(&candidate)?, candidate);
    }
    Ok(unique)
}

fn codex_terminal_agent_output(trace: &Path, protocol_schema_version: u32) -> EvalResult<Value> {
    let output_contract = agent_output_contract(trace, protocol_schema_version)?;
    codex_terminal_agent_output_with_contract(trace, protocol_schema_version, output_contract)
}

fn codex_terminal_agent_output_with_contract(
    trace: &Path,
    protocol_schema_version: u32,
    output_contract: NativeAgentOutputContract,
) -> EvalResult<Value> {
    if output_contract == NativeAgentOutputContract::StaleSafetyV3 {
        validate_native_stale_v3_codex_action_lifecycle(trace)?;
    }
    let reader = BufReader::new(File::open(trace)?);
    let mut turn_started = 0_u32;
    let mut turn_completed = 0_u32;
    let mut terminal_index = None;
    let mut last_event_index = None;
    let mut last_item_index = None;
    let mut last_agent_message_index = None;
    let mut last_agent_output = None;
    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(&line).map_err(|error| {
            EvalError::Invalid(format!(
                "evaluation trace line {} is invalid JSON: {error}",
                index + 1
            ))
        })?;
        last_event_index = Some(index);
        if matches!(
            value.get("type").and_then(Value::as_str),
            Some("item.started" | "item.completed")
        ) {
            last_item_index = Some(index);
        }
        match value.get("type").and_then(Value::as_str) {
            Some("turn.started") => turn_started += 1,
            Some("turn.completed") => {
                turn_completed += 1;
                terminal_index = Some(index);
            }
            Some("item.completed")
                if value.pointer("/item/type").and_then(Value::as_str) == Some("agent_message") =>
            {
                last_agent_message_index = Some(index);
                last_agent_output = value
                    .pointer("/item/text")
                    .and_then(Value::as_str)
                    .and_then(|text| serde_json::from_str::<Value>(text.trim()).ok())
                    .filter(|value| {
                        is_agent_output(value, protocol_schema_version, output_contract)
                    });
            }
            _ => {}
        }
    }
    let terminal_index = terminal_index.ok_or_else(|| {
        EvalError::Invalid(
            "Codex evaluation trace has no terminal turn.completed event".to_string(),
        )
    })?;
    if turn_started != 1 || turn_completed != 1 {
        return Err(EvalError::Invalid(format!(
            "Codex evaluation trace must contain exactly one turn lifecycle; started={turn_started}, completed={turn_completed}"
        )));
    }
    if last_event_index != Some(terminal_index)
        || last_agent_message_index != last_item_index
        || last_agent_message_index.map_or(true, |index| index >= terminal_index)
    {
        return Err(EvalError::Invalid(
            "Codex evaluation trace has no terminal agent message immediately before turn.completed"
                .to_string(),
        ));
    }
    last_agent_output.ok_or_else(|| {
        EvalError::Invalid(
            "Codex terminal agent message does not satisfy the frozen structured-output schema"
                .to_string(),
        )
    })
}

fn agent_output_contract(
    trace: &Path,
    protocol_schema_version: u32,
) -> EvalResult<NativeAgentOutputContract> {
    match trace_native_stale_family_version(trace, protocol_schema_version)? {
        None => Ok(NativeAgentOutputContract::Base),
        Some(1) => Ok(NativeAgentOutputContract::StaleSafetyV1),
        Some(3) => Ok(NativeAgentOutputContract::StaleSafetyV3),
        Some(version) => Err(EvalError::Invalid(format!(
            "unsupported native stale-safety structured-output family version {version}"
        ))),
    }
}

fn collect_agent_outputs(
    value: &Value,
    output: &mut Vec<Value>,
    protocol_schema_version: u32,
    output_contract: NativeAgentOutputContract,
) {
    if is_agent_output(value, protocol_schema_version, output_contract) {
        output.push(value.clone());
    }
    match value {
        Value::Array(values) => {
            for value in values {
                collect_agent_outputs(value, output, protocol_schema_version, output_contract);
            }
        }
        Value::Object(values) => {
            for value in values.values() {
                collect_agent_outputs(value, output, protocol_schema_version, output_contract);
            }
        }
        Value::String(value) => {
            if let Ok(parsed) = serde_json::from_str::<Value>(value.trim()) {
                collect_agent_outputs(&parsed, output, protocol_schema_version, output_contract);
            }
        }
        _ => {}
    }
}

fn is_agent_output(
    value: &Value,
    protocol_schema_version: u32,
    output_contract: NativeAgentOutputContract,
) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    let mut exact_keys = vec![
        "answer",
        "repository_remote",
        "project",
        "component",
        "first_action",
        "returned_context_keys",
        "applied_context_keys",
        "evidence_targets",
        "abstained",
    ];
    if protocol_schema_version >= STRUCTURED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION {
        exact_keys.extend([
            "checkout_root",
            "project_status",
            "project_confirmation_required",
        ]);
    }
    if output_contract == NativeAgentOutputContract::StaleSafetyV1 {
        exact_keys.extend(["boundary_signal", "native_retention_signal"]);
    } else if output_contract == NativeAgentOutputContract::StaleSafetyV3 {
        exact_keys.extend([
            "boundary_signal",
            "native_evidence_signal",
            "causal_result",
            "retrieved_native_markers",
        ]);
    }
    object.len() == exact_keys.len()
        && exact_keys.iter().all(|key| object.contains_key(*key))
        && object["answer"].is_string()
        && ["repository_remote", "project", "component", "first_action"]
            .iter()
            .all(|key| object[*key].is_null() || object[*key].is_string())
        && [
            "returned_context_keys",
            "applied_context_keys",
            "evidence_targets",
        ]
        .iter()
        .all(|key| {
            object[*key]
                .as_array()
                .is_some_and(|values| values.iter().all(Value::is_string))
        })
        && object["abstained"].is_boolean()
        && (protocol_schema_version < STRUCTURED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION
            || ((object["checkout_root"].is_null() || object["checkout_root"].is_string())
                && object["project_status"].as_str().is_some_and(|status| {
                    matches!(
                        status,
                        "authorized" | "requires_confirmation" | "unavailable"
                    )
                })
                && object["project_confirmation_required"].is_boolean()))
        && match output_contract {
            NativeAgentOutputContract::Base => true,
            NativeAgentOutputContract::StaleSafetyV1 => {
                valid_stale_boundary_signal(&object["boundary_signal"])
                    && object["native_retention_signal"]
                        .as_str()
                        .is_some_and(|signal| {
                            matches!(signal, "retained" | "retention_absent" | "not_applicable")
                        })
            }
            NativeAgentOutputContract::StaleSafetyV3 => {
                valid_stale_boundary_signal(&object["boundary_signal"])
                    && object["native_evidence_signal"]
                        .as_str()
                        .is_some_and(|signal| {
                            matches!(
                                signal,
                                "not_applicable"
                                    | "source_markers_correlated"
                                    | "expiry_markers_correlated"
                                    | "native_evidence_insufficient"
                            )
                        })
                    && object["causal_result"].as_str().is_some_and(|result| {
                        matches!(result, "FAIL" | "CAUSAL_PASS" | "SAFE_INCONCLUSIVE")
                    })
                    && object["retrieved_native_markers"]
                        .as_array()
                        .is_some_and(|markers| {
                            markers.len() <= 2
                                && markers.iter().all(Value::is_string)
                                && markers
                                    .iter()
                                    .filter_map(Value::as_str)
                                    .collect::<BTreeSet<_>>()
                                    .len()
                                    == markers.len()
                        })
            }
        }
}

fn valid_stale_boundary_signal(value: &Value) -> bool {
    value.as_str().is_some_and(|signal| {
        matches!(
            signal,
            "boundary:source_unavailable" | "boundary:verification_expired"
        )
    })
}

fn attest_codex_session_rollout(
    lane: &PreparedNativeLane,
    phase: NativePilotRunPhase,
    provider_trace: &Path,
    before: &CodexSessionPreDispatchInventory,
) -> EvalResult<CodexSessionRolloutAttestation> {
    #[cfg(not(unix))]
    {
        let _ = (lane, phase, provider_trace, before);
        return Err(EvalError::Invalid(
            "Codex session rollout attestation requires Unix metadata".to_string(),
        ));
    }

    #[cfg(unix)]
    {
        let thread_id = codex_provider_trace_thread_id(provider_trace)?;
        let after = capture_codex_pre_dispatch_inventory(lane)?;
        if before.sessions_root != after.sessions_root {
            return Err(EvalError::Invalid(
                "Codex session inventory root drifted after provider dispatch".to_string(),
            ));
        }
        let before_paths = before
            .relative_file_paths
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let after_paths = after
            .relative_file_paths
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        if before_paths.len() != before.relative_file_paths.len()
            || after_paths.len() != after.relative_file_paths.len()
            || !before_paths.is_subset(&after_paths)
        {
            return Err(EvalError::Invalid(
                "Codex session inventory is duplicated or regressed after provider dispatch"
                    .to_string(),
            ));
        }
        let created = after_paths
            .difference(&before_paths)
            .cloned()
            .collect::<Vec<_>>();
        if created.len() != 1 {
            return Err(EvalError::Invalid(format!(
                "Codex provider dispatch must create exactly one new session rollout path; found {}",
                created.len()
            )));
        }
        let relative = Path::new(&created[0]);
        if !relative
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(&format!("-{thread_id}.jsonl")))
        {
            return Err(EvalError::Invalid(
                "new Codex session rollout path does not match the provider thread identity"
                    .to_string(),
            ));
        }
        attest_codex_session_rollout_file(
            Path::new(&after.sessions_root),
            &Path::new(&after.sessions_root).join(relative),
            &thread_id,
            lane,
            phase,
        )
    }
}

fn validate_recorded_codex_session_rollout(
    lane: &PreparedNativeLane,
    phase: NativePilotRunPhase,
    provider_trace: &Path,
    recorded: &CodexSessionRolloutAttestation,
) -> EvalResult<()> {
    if !is_sha256(&recorded.sha256)
        || !is_sha256(&recorded.agents_md_sha256)
        || recorded.model_provider != "openai"
        || recorded.host_resolved_model.trim().is_empty()
    {
        return Err(EvalError::Invalid(
            "recorded Codex session rollout attestation is malformed".to_string(),
        ));
    }
    let thread_id = codex_provider_trace_thread_id(provider_trace)?;
    let current = capture_codex_pre_dispatch_inventory(lane)?;
    let matching = current
        .relative_file_paths
        .iter()
        .filter(|relative| {
            Path::new(relative)
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(&format!("-{thread_id}.jsonl")))
        })
        .collect::<Vec<_>>();
    if matching.len() != 1 {
        return Err(EvalError::Invalid(format!(
            "recorded Codex provider trace must resolve exactly one session rollout; found {}",
            matching.len()
        )));
    }
    let path = Path::new(&current.sessions_root).join(matching[0]);
    if recorded.path != path.display().to_string() {
        return Err(EvalError::Invalid(
            "recorded Codex session rollout path drifted".to_string(),
        ));
    }
    let observed = attest_codex_session_rollout_file(
        Path::new(&current.sessions_root),
        &path,
        &thread_id,
        lane,
        phase,
    )?;
    if &observed != recorded {
        return Err(EvalError::Invalid(
            "Codex host-resolved model or session rollout attestation drifted".to_string(),
        ));
    }
    Ok(())
}

fn capture_codex_pre_dispatch_inventory(
    lane: &PreparedNativeLane,
) -> EvalResult<CodexSessionPreDispatchInventory> {
    #[cfg(not(unix))]
    {
        let _ = lane;
        return Err(EvalError::Invalid(
            "Codex session inventory requires Unix metadata".to_string(),
        ));
    }

    #[cfg(unix)]
    {
        let codex_home = lane.environment.get("CODEX_HOME").ok_or_else(|| {
            EvalError::Invalid(format!("Codex lane {} is missing CODEX_HOME", lane.order))
        })?;
        let (_, owner) = open_private_directory(Path::new(codex_home))?;
        let codex_home = Path::new(codex_home).canonicalize()?;
        let sessions_root = codex_home.join("sessions");
        let mut relative_file_paths = Vec::new();
        let mut entry_count = 0;
        match fs::symlink_metadata(&sessions_root) {
            Ok(_) => collect_codex_session_inventory(
                &sessions_root,
                &sessions_root,
                owner,
                0,
                &mut entry_count,
                &mut relative_file_paths,
            )?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(EvalError::Io(error)),
        }
        relative_file_paths.sort();
        Ok(CodexSessionPreDispatchInventory {
            sessions_root: sessions_root.display().to_string(),
            relative_file_paths,
            captured_unix_ms: unix_ms(SystemTime::now())?,
        })
    }
}

fn collect_codex_session_inventory(
    sessions_root: &Path,
    directory: &Path,
    expected_owner: u32,
    depth: usize,
    entry_count: &mut usize,
    relative_file_paths: &mut Vec<String>,
) -> EvalResult<()> {
    #[cfg(not(unix))]
    {
        let _ = (
            sessions_root,
            directory,
            expected_owner,
            depth,
            entry_count,
            relative_file_paths,
        );
        return Err(EvalError::Invalid(
            "Codex session inventory requires Unix metadata".to_string(),
        ));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;

        if depth > MAX_CODEX_SESSION_SCAN_DEPTH {
            return Err(EvalError::Invalid(
                "Codex session scan exceeded its depth bound".to_string(),
            ));
        }
        let directory_metadata = fs::symlink_metadata(directory)?;
        if directory_metadata.file_type().is_symlink()
            || !directory_metadata.is_dir()
            || directory_metadata.uid() != expected_owner
            || directory_metadata.mode() & 0o077 != 0
        {
            return Err(EvalError::Invalid(
                "Codex session directory is not private and owner-controlled".to_string(),
            ));
        }
        let mut entries = fs::read_dir(directory)?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            *entry_count = entry_count.saturating_add(1);
            if *entry_count > MAX_CODEX_SESSION_SCAN_ENTRIES {
                return Err(EvalError::Invalid(
                    "Codex session scan exceeded its entry bound".to_string(),
                ));
            }
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)?;
            if metadata.file_type().is_symlink() {
                return Err(EvalError::Invalid(
                    "Codex session scan encountered a symlink".to_string(),
                ));
            }
            if metadata.is_dir() {
                collect_codex_session_inventory(
                    sessions_root,
                    &path,
                    expected_owner,
                    depth + 1,
                    entry_count,
                    relative_file_paths,
                )?;
            } else if metadata.is_file() {
                if metadata.uid() != expected_owner
                    || metadata.mode() & 0o077 != 0
                    || metadata.nlink() != 1
                    || metadata.len() > MAX_CODEX_SESSION_ROLLOUT_BYTES
                {
                    return Err(EvalError::Invalid(
                        "Codex session file failed its private bounded identity contract"
                            .to_string(),
                    ));
                }
                let relative = path.strip_prefix(sessions_root).map_err(|_| {
                    EvalError::Invalid(
                        "Codex session file escaped its isolated session root".to_string(),
                    )
                })?;
                let relative = relative.to_str().ok_or_else(|| {
                    EvalError::Invalid("Codex session path is not UTF-8".to_string())
                })?;
                relative_file_paths.push(relative.to_string());
            } else {
                return Err(EvalError::Invalid(
                    "Codex session scan encountered a special filesystem entry".to_string(),
                ));
            }
        }
        Ok(())
    }
}

fn attest_codex_session_rollout_file(
    sessions_root: &Path,
    path: &Path,
    thread_id: &str,
    lane: &PreparedNativeLane,
    phase: NativePilotRunPhase,
) -> EvalResult<CodexSessionRolloutAttestation> {
    #[cfg(not(unix))]
    {
        let _ = (sessions_root, path, thread_id, lane, phase);
        return Err(EvalError::Invalid(
            "Codex session rollout attestation requires Unix metadata".to_string(),
        ));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, OpenOptionsExt};

        if !path.starts_with(sessions_root) {
            return Err(EvalError::Invalid(
                "Codex session rollout escapes the isolated session directory".to_string(),
            ));
        }
        let mut options = OpenOptions::new();
        options
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
        let mut file = options.open(path)?;
        let metadata = file.metadata()?;
        if !metadata.is_file()
            || metadata.uid() != unsafe { libc::geteuid() }
            || metadata.mode() & 0o077 != 0
            || metadata.nlink() != 1
            || metadata.len() == 0
            || metadata.len() > MAX_CODEX_SESSION_ROLLOUT_BYTES
        {
            return Err(EvalError::Invalid(
                "Codex session rollout failed its private bounded file identity contract"
                    .to_string(),
            ));
        }
        let mut bytes = Vec::new();
        (&mut file)
            .take(MAX_CODEX_SESSION_ROLLOUT_BYTES + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 != metadata.len() {
            return Err(EvalError::Invalid(
                "Codex session rollout changed while reading".to_string(),
            ));
        }

        let (expected_agents_directory, expected_agents_instructions) =
            expected_codex_agents_context(lane, phase)?;

        let mut session_ids = BTreeSet::new();
        let mut providers = BTreeSet::new();
        let mut world_models = BTreeSet::new();
        let mut turn_models = BTreeSet::new();
        let mut agents_md_digests = BTreeSet::new();
        let mut session_meta_count = 0_usize;
        let mut world_state_count = 0_usize;
        let mut full_agents_world_state_count = 0_usize;
        let mut turn_context_count = 0_usize;
        for (index, line) in bytes.split(|byte| *byte == b'\n').enumerate() {
            if line.iter().all(u8::is_ascii_whitespace) {
                continue;
            }
            let value: Value = serde_json::from_slice(line).map_err(|_| {
                EvalError::Invalid(format!(
                    "Codex session rollout line {} is invalid JSON",
                    index + 1
                ))
            })?;
            match value.get("type").and_then(Value::as_str) {
                Some("session_meta") => {
                    session_meta_count += 1;
                    let mut found_session_id = false;
                    for pointer in ["/payload/session_id", "/payload/id"] {
                        if let Some(value) = value.pointer(pointer).and_then(Value::as_str) {
                            found_session_id = true;
                            session_ids.insert(value.to_string());
                        }
                    }
                    if !found_session_id {
                        return Err(EvalError::Invalid(
                            "Codex session_meta record omitted its session identity".to_string(),
                        ));
                    }
                    let provider = value
                        .pointer("/payload/model_provider")
                        .and_then(Value::as_str)
                        .ok_or_else(|| {
                            EvalError::Invalid(
                                "Codex session_meta record omitted its model provider".to_string(),
                            )
                        })?;
                    providers.insert(provider.to_string());
                }
                Some("world_state") => {
                    world_state_count += 1;
                    let is_full = match value.pointer("/payload/full") {
                        Some(Value::Bool(full)) => *full,
                        Some(_) => {
                            return Err(EvalError::Invalid(
                                "Codex world_state full flag is not boolean".to_string(),
                            ));
                        }
                        None => true,
                    };
                    let mut found_model = false;
                    for pointer in [
                        "/payload/state/model",
                        "/payload/state/collaboration_mode/model",
                    ] {
                        match value.pointer(pointer) {
                            Some(Value::String(model)) => {
                                found_model = true;
                                world_models.insert(model.to_string());
                            }
                            Some(_) => {
                                return Err(EvalError::Invalid(
                                    "Codex world_state model field is not a string".to_string(),
                                ));
                            }
                            None => {}
                        }
                    }
                    if is_full && !found_model {
                        return Err(EvalError::Invalid(
                            "Codex world_state record omitted its model".to_string(),
                        ));
                    }
                    if is_full && value.pointer("/payload/state/agents_md").is_none() {
                        return Err(EvalError::Invalid(
                            "Codex full world_state omitted agents_md".to_string(),
                        ));
                    }
                    if let Some(agents_md) = value.pointer("/payload/state/agents_md") {
                        let directory = agents_md
                            .get("directory")
                            .and_then(Value::as_str)
                            .ok_or_else(|| {
                                EvalError::Invalid(
                                    "Codex full world_state agents_md omitted directory"
                                        .to_string(),
                                )
                            })?;
                        let text =
                            agents_md
                                .get("text")
                                .and_then(Value::as_str)
                                .ok_or_else(|| {
                                    EvalError::Invalid(
                                        "Codex full world_state agents_md omitted text".to_string(),
                                    )
                                })?;
                        if directory != expected_agents_directory
                            || !text.ends_with(&expected_agents_instructions)
                        {
                            return Err(EvalError::Invalid(
                                "Codex rollout does not bind the exact fixture repository instructions"
                                    .to_string(),
                            ));
                        }
                        if is_full {
                            full_agents_world_state_count += 1;
                        }
                        agents_md_digests.insert(sha256_bytes(&serde_json::to_vec(agents_md)?));
                    }
                }
                Some("turn_context") => {
                    turn_context_count += 1;
                    let model = value
                        .pointer("/payload/model")
                        .and_then(Value::as_str)
                        .ok_or_else(|| {
                            EvalError::Invalid(
                                "Codex turn_context record omitted its model".to_string(),
                            )
                        })?;
                    turn_models.insert(model.to_string());
                }
                _ => {}
            }
        }
        if session_meta_count != 1
            || world_state_count == 0
            || full_agents_world_state_count == 0
            || turn_context_count == 0
            || session_ids != BTreeSet::from([thread_id.to_string()])
            || providers != BTreeSet::from(["openai".to_string()])
            || world_models.len() != 1
            || turn_models.len() != 1
            || world_models != turn_models
            || agents_md_digests.len() != 1
        {
            return Err(EvalError::Invalid(
                "Codex session rollout lacks concordant host-resolved model evidence".to_string(),
            ));
        }
        let host_resolved_model = world_models
            .into_iter()
            .next()
            .expect("checked one world-state model");
        if host_resolved_model.trim().is_empty() {
            return Err(EvalError::Invalid(
                "Codex host-resolved model identifier is empty".to_string(),
            ));
        }
        let path = path.canonicalize()?;
        if !path.starts_with(sessions_root) {
            return Err(EvalError::Invalid(
                "Codex session rollout escapes the isolated session directory".to_string(),
            ));
        }
        Ok(CodexSessionRolloutAttestation {
            path: path.display().to_string(),
            sha256: sha256_bytes(&bytes),
            model_provider: "openai".to_string(),
            host_resolved_model,
            agents_md_sha256: agents_md_digests
                .into_iter()
                .next()
                .expect("checked one agents_md digest"),
        })
    }
}

fn codex_provider_trace_thread_id(trace: &Path) -> EvalResult<String> {
    let metadata = fs::symlink_metadata(trace)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > MAX_CODEX_SESSION_ROLLOUT_BYTES
    {
        return Err(EvalError::Invalid(
            "Codex provider trace failed its bounded file contract".to_string(),
        ));
    }
    let reader = BufReader::new(File::open(trace)?);
    let mut thread_ids = BTreeSet::new();
    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(&line).map_err(|_| {
            EvalError::Invalid(format!(
                "Codex provider trace line {} is invalid JSON",
                index + 1
            ))
        })?;
        if value.get("type").and_then(Value::as_str) == Some("thread.started") {
            if let Some(thread_id) = value.get("thread_id").and_then(Value::as_str) {
                thread_ids.insert(thread_id.to_string());
            }
        }
    }
    if thread_ids.len() != 1 {
        return Err(EvalError::Invalid(
            "Codex provider trace does not contain exactly one thread identity".to_string(),
        ));
    }
    let thread_id = thread_ids
        .into_iter()
        .next()
        .expect("checked one thread id");
    validate_uuid(&thread_id, "Codex thread identity")?;
    Ok(thread_id)
}

fn expected_codex_agents_context(
    lane: &PreparedNativeLane,
    phase: NativePilotRunPhase,
) -> EvalResult<(String, String)> {
    let phase_cwd = match phase {
        NativePilotRunPhase::Teaching => &lane.teaching_cwd,
        NativePilotRunPhase::Activation | NativePilotRunPhase::Evaluation => &lane.evaluation_cwd,
    };
    let cwd = Path::new(phase_cwd).canonicalize()?;
    let mut checkout = cwd.clone();
    loop {
        let dot_git = checkout.join(".git");
        match fs::symlink_metadata(&dot_git) {
            Ok(metadata)
                if !metadata.file_type().is_symlink()
                    && (metadata.is_dir() || metadata.is_file()) =>
            {
                break;
            }
            Ok(_) => {
                return Err(EvalError::Invalid(
                    "fixture checkout .git identity is not a regular file or directory".to_string(),
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(EvalError::Io(error)),
        }
        if !checkout.pop() {
            return Err(EvalError::Invalid(
                "Codex phase cwd is not contained in a fixture checkout".to_string(),
            ));
        }
    }
    let mut current = cwd.clone();
    let instructions = loop {
        let candidate = current.join("AGENTS.md");
        match fs::symlink_metadata(&candidate) {
            Ok(metadata)
                if !metadata.file_type().is_symlink()
                    && metadata.is_file()
                    && metadata.len() > 0
                    && metadata.len() <= MAX_CODEX_AUTH_FILE_BYTES =>
            {
                let canonical = candidate.canonicalize()?;
                if !canonical.starts_with(&checkout) {
                    return Err(EvalError::Invalid(
                        "fixture AGENTS.md escaped the exact checkout root".to_string(),
                    ));
                }
                let bytes = fs::read(&canonical)?;
                if bytes.len() as u64 != metadata.len() {
                    return Err(EvalError::Invalid(
                        "fixture AGENTS.md changed while reading".to_string(),
                    ));
                }
                break String::from_utf8(bytes).map_err(|_| {
                    EvalError::Invalid("fixture AGENTS.md is not UTF-8".to_string())
                })?;
            }
            Ok(_) => {
                return Err(EvalError::Invalid(
                    "fixture AGENTS.md is not a bounded regular non-symlink file".to_string(),
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(EvalError::Io(error)),
        }
        if current == checkout || !current.pop() || !current.starts_with(&checkout) {
            return Err(EvalError::Invalid(
                "Codex phase checkout has no authoritative fixture AGENTS.md".to_string(),
            ));
        }
    };
    if instructions.is_empty() {
        return Err(EvalError::Invalid(
            "fixture AGENTS.md must not be empty".to_string(),
        ));
    }
    Ok((cwd.display().to_string(), instructions))
}

fn validate_uuid(value: &str, label: &str) -> EvalResult<()> {
    let bytes = value.as_bytes();
    if bytes.len() != 36
        || [8, 13, 18, 23]
            .into_iter()
            .any(|index| bytes[index] != b'-')
        || bytes
            .iter()
            .enumerate()
            .any(|(index, byte)| ![8, 13, 18, 23].contains(&index) && !byte.is_ascii_hexdigit())
        || value
            .chars()
            .filter(|character| *character != '-')
            .all(|character| character == '0')
    {
        return Err(EvalError::Invalid(format!(
            "{label} is not one nonzero UUID"
        )));
    }
    Ok(())
}

fn validate_activation_interval(lane: &PreparedNativeLane) -> EvalResult<()> {
    let teaching = fs::metadata(&lane.teaching_trace_path)?
        .modified()
        .map_err(EvalError::Io)?;
    let elapsed = SystemTime::now()
        .duration_since(teaching)
        .map_err(|error| {
            EvalError::Invalid(format!(
                "teaching trace timestamp is in the future: {error}"
            ))
        })?;
    let required = u64::from(lane.activation_wait_hours) * 60 * 60;
    if elapsed.as_secs() < required {
        return Err(EvalError::Invalid(format!(
            "lane {} requires {} idle hours before activation; {} seconds remain",
            lane.order,
            lane.activation_wait_hours,
            required.saturating_sub(elapsed.as_secs())
        )));
    }
    Ok(())
}

fn validate_phase_postconditions(
    audit: &NativePilotAudit,
    phase: NativePilotRunPhase,
) -> EvalResult<()> {
    if audit.invalid {
        return Err(EvalError::Invalid(format!(
            "{} phase produced an invalid audit",
            phase.label()
        )));
    }
    let valid = match phase {
        NativePilotRunPhase::Teaching => audit.lanes.iter().all(|lane| {
            matches!(
                lane.phase,
                NativeLanePhase::AwaitingActivation | NativeLanePhase::ReadyForEvaluation
            )
        }),
        NativePilotRunPhase::Activation => audit.ready_for_evaluation,
        NativePilotRunPhase::Evaluation => audit.complete,
    };
    if !valid {
        return Err(EvalError::Invalid(format!(
            "{} phase did not reach its required lifecycle state",
            phase.label()
        )));
    }
    Ok(())
}

fn validate_executed_lane_postcondition(
    audit: &NativePilotAudit,
    planned: &PreparedNativeLane,
    phase: NativePilotRunPhase,
) -> EvalResult<()> {
    let lane = audit
        .lanes
        .iter()
        .find(|lane| lane.order == planned.order)
        .ok_or_else(|| {
            EvalError::Invalid(format!(
                "post-execution audit omitted lane {}",
                planned.order
            ))
        })?;
    let expected = match phase {
        NativePilotRunPhase::Teaching if planned.host == "codex" => {
            NativeLanePhase::AwaitingActivation
        }
        NativePilotRunPhase::Teaching | NativePilotRunPhase::Activation => {
            NativeLanePhase::ReadyForEvaluation
        }
        NativePilotRunPhase::Evaluation => NativeLanePhase::EvaluationComplete,
    };
    if lane.phase != expected {
        return Err(EvalError::Invalid(format!(
            "{} execution for lane {} reached {:?}, expected {:?}; stopping before the next provider call",
            phase.label(),
            lane.order,
            lane.phase,
            expected
        )));
    }
    Ok(())
}

fn run_command(
    argv: &[String],
    cwd: &Path,
    environment: &BTreeMap<String, String>,
    stdout_path: &Path,
    stderr_path: &Path,
) -> EvalResult<i32> {
    let exit_code = run_command_status(argv, cwd, environment, stdout_path, stderr_path)?;
    if exit_code != 0 {
        return Err(EvalError::Invalid(format!(
            "attested command exited with exit status: {exit_code}; stderr is {}",
            stderr_path.display()
        )));
    }
    Ok(exit_code)
}

fn run_auxiliary_command(
    argv: &[String],
    cwd: &Path,
    environment: &BTreeMap<String, String>,
    stdout_path: &Path,
    stderr_path: &Path,
    bounded_closed_world: bool,
) -> EvalResult<i32> {
    if !bounded_closed_world {
        return run_command(argv, cwd, environment, stdout_path, stderr_path);
    }
    let (exit_code, safety) = run_bounded_provider_command_status(
        argv,
        cwd,
        environment,
        &[
            "CODEX_ACCESS_TOKEN".to_string(),
            "OPENAI_API_KEY".to_string(),
            "ANTHROPIC_API_KEY".to_string(),
            "ANTHROPIC_AUTH_TOKEN".to_string(),
            "CLAUDE_CODE_OAUTH_TOKEN".to_string(),
        ],
        stdout_path,
        stderr_path,
        || Ok(()),
    )
    .map_err(BoundedProviderCommandFailure::into_error)?;
    if !safety.process_cleanup_proven {
        return Err(EvalError::Invalid(
            "bounded family-v3 auxiliary command did not prove process-group cleanup".to_string(),
        ));
    }
    if exit_code != 0 {
        return Err(EvalError::Invalid(format!(
            "bounded family-v3 auxiliary command exited with status {exit_code}; stderr is {}",
            stderr_path.display()
        )));
    }
    Ok(exit_code)
}

fn run_provider_command(
    lane: &PreparedNativeLane,
    phase: NativePilotRunPhase,
    argv: &[String],
    cwd: &Path,
    artifacts: (&Path, &Path),
    protocol_schema_version: u32,
    closed_world_environment: bool,
    revalidate_bundle_dispatch_boundary: impl FnOnce() -> EvalResult<()>,
) -> Result<ProviderCommandOutcome, ProviderLaneFailure> {
    let (stdout_path, stderr_path) = artifacts;
    let (exit_code, safety) = if closed_world_environment {
        let (exit_code, evidence) = run_bounded_provider_command_status(
            argv,
            cwd,
            &lane.environment,
            &[
                "CODEX_ACCESS_TOKEN".to_string(),
                "OPENAI_API_KEY".to_string(),
                "ANTHROPIC_API_KEY".to_string(),
                "ANTHROPIC_AUTH_TOKEN".to_string(),
                "CLAUDE_CODE_OAUTH_TOKEN".to_string(),
            ],
            stdout_path,
            stderr_path,
            revalidate_bundle_dispatch_boundary,
        )
        .map_err(BoundedProviderCommandFailure::into_lane_failure)?;
        (exit_code, Some(evidence))
    } else {
        (
            run_command_status(argv, cwd, &lane.environment, stdout_path, stderr_path)
                .map_err(ProviderLaneFailure::post_dispatch_failure)?,
            None,
        )
    };
    let result = (|| -> EvalResult<ProviderCommandOutcome> {
        let summary = if lane.host == "claude_code" {
            claude_trace_summary(stdout_path)?
        } else {
            ProviderCommandOutcome {
                exit_code,
                provider_reported_cost_microusd: None,
                accepted_turn_boundary_budget_exit: false,
                safety,
            }
        };
        if exit_code == 0 {
            return Ok(ProviderCommandOutcome {
                exit_code,
                provider_reported_cost_microusd: summary.provider_reported_cost_microusd,
                accepted_turn_boundary_budget_exit: false,
                safety,
            });
        }
        if lane.host == "claude_code" && summary.accepted_turn_boundary_budget_exit {
            return Ok(ProviderCommandOutcome {
                exit_code,
                safety,
                ..summary
            });
        }
        if lane.host == "claude_code" && phase == NativePilotRunPhase::Evaluation {
            if let Ok(summary) =
                validate_recoverable_claude_evaluation_trace(stdout_path, protocol_schema_version)
            {
                return Ok(ProviderCommandOutcome {
                    exit_code,
                    safety,
                    ..summary
                });
            }
        }
        Err(EvalError::Invalid(format!(
            "attested command exited with exit status: {exit_code}; stderr is {}",
            stderr_path.display()
        )))
    })();
    if let Some(safety) = safety {
        result.map_err(|error| {
            ProviderLaneFailure::post_dispatch_failure(bind_post_dispatch_failure(error, safety))
        })
    } else {
        result.map_err(ProviderLaneFailure::post_dispatch_failure)
    }
}

fn run_command_status(
    argv: &[String],
    cwd: &Path,
    environment: &BTreeMap<String, String>,
    stdout_path: &Path,
    stderr_path: &Path,
) -> EvalResult<i32> {
    let (program, args) = argv
        .split_first()
        .ok_or_else(|| EvalError::Invalid("refusing to execute an empty argv".to_string()))?;
    let stdout = create_private_file(stdout_path)?;
    let stderr = create_private_file(stderr_path)?;
    let mut command = Command::new(program);
    command.args(args).current_dir(cwd);
    command.envs(environment);
    if argv.windows(2).any(|pair| {
        pair[0] == "--config"
            && matches!(pair[1].as_str(), CODEX_KEYRING_CONFIG | CODEX_FILE_CONFIG)
    }) {
        command
            .env_remove("CODEX_ACCESS_TOKEN")
            .env_remove("OPENAI_API_KEY");
    }
    let status = command
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .status()
        .map_err(|error| {
            EvalError::Invalid(format!(
                "failed to execute attested command in {}: {error}",
                cwd.display()
            ))
        })?;
    status.code().ok_or_else(|| {
        EvalError::Invalid("attested command exited without a numeric status".to_string())
    })
}

fn finish_post_spawn_output_setup_failure(
    mut guard: NativeExecutionChildGuard,
    terminal_deadline: Instant,
    stage: &str,
    primary: EvalError,
) -> BoundedProviderCommandFailure {
    match guard.terminate_group_and_reap_with_status_until(terminal_deadline) {
        NativeExecutionCleanupOutcome::ProvenTerminal(_) => {
            BoundedProviderCommandFailure::PostSpawnOutputSetup {
                error: EvalError::Invalid(format!(
                    "post-spawn provider output setup failed; stage={stage}; process_cleanup_proven=true; failure_sha256={}",
                    sha256_bytes(primary.to_string().as_bytes())
                )),
                process_cleanup_proven: true,
                quarantine: None,
            }
        }
        NativeExecutionCleanupOutcome::CleanupUnproven(cleanup_error) => {
            BoundedProviderCommandFailure::PostSpawnOutputSetup {
                error: EvalError::Invalid(format!(
                    "post-spawn provider output setup failed; stage={stage}; process_cleanup_proven=false; failure_sha256={}; cleanup_failure_sha256={}",
                    sha256_bytes(primary.to_string().as_bytes()),
                    sha256_bytes(cleanup_error.to_string().as_bytes())
                )),
                process_cleanup_proven: false,
                quarantine: Some(guard),
            }
        }
    }
}

fn run_bounded_provider_command_status(
    argv: &[String],
    cwd: &Path,
    environment: &BTreeMap<String, String>,
    environment_remove: &[String],
    stdout_path: &Path,
    stderr_path: &Path,
    revalidate_dispatch_boundary: impl FnOnce() -> EvalResult<()>,
) -> Result<(i32, ProviderSafetyEvidence), BoundedProviderCommandFailure> {
    let (program, args) = argv
        .split_first()
        .ok_or_else(|| EvalError::Invalid("refusing to execute an empty argv".to_string()))
        .map_err(BoundedProviderCommandFailure::PreSpawn)?;
    let started = Instant::now();
    let response_deadline = started
        .checked_add(PROVIDER_WALL_TIMEOUT)
        .ok_or_else(|| EvalError::Invalid("provider wall deadline overflow".to_string()))
        .map_err(BoundedProviderCommandFailure::PreSpawn)?;
    let terminal_deadline = response_deadline
        .checked_add(PROVIDER_CLEANUP_GRACE)
        .ok_or_else(|| EvalError::Invalid("provider cleanup deadline overflow".to_string()))
        .map_err(BoundedProviderCommandFailure::PreSpawn)?;
    let mut command = Command::new(program);
    command
        .args(args)
        .current_dir(cwd)
        .env_clear()
        .envs(environment)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for key in environment_remove {
        command.env_remove(key);
    }
    // This is the final evaluator-controlled fallible step before the kernel spawn boundary.
    // A same-UID actor or OS scheduling can still race after this returns and before spawn/exec;
    // the receipt proves this check, not elimination of that irreducible interval.
    revalidate_dispatch_boundary().map_err(BoundedProviderCommandFailure::PreSpawn)?;
    let mut guard = match NativeExecutionChildGuard::spawn(command, terminal_deadline) {
        NativeExecutionSpawnOutcome::Spawned(guard) => guard,
        NativeExecutionSpawnOutcome::NotSpawned(error) => {
            return Err(BoundedProviderCommandFailure::PreSpawn(EvalError::Invalid(
                format!("bounded provider was not spawned: {error}"),
            )))
        }
        NativeExecutionSpawnOutcome::PostSpawnTerminal { error, .. } => {
            return Err(BoundedProviderCommandFailure::PostSpawnOutputSetup {
                error: EvalError::Invalid(format!(
                    "post-spawn provider output setup failed; stage=spawn_verification; process_cleanup_proven=true; failure_sha256={}",
                    sha256_bytes(error.to_string().as_bytes())
                )),
                process_cleanup_proven: true,
                quarantine: None,
            });
        }
        NativeExecutionSpawnOutcome::PostSpawnCleanupUnproven {
            error, quarantine, ..
        } => {
            return Err(BoundedProviderCommandFailure::PostSpawnOutputSetup {
                error: EvalError::Invalid(format!(
                    "post-spawn provider output setup failed; stage=spawn_verification; process_cleanup_proven=false; failure_sha256={}",
                    sha256_bytes(error.to_string().as_bytes())
                )),
                process_cleanup_proven: false,
                quarantine: Some(quarantine),
            });
        }
    };
    #[cfg(test)]
    if take_injected_provider_output_setup_failure(InjectedProviderOutputSetupFailure::StdoutFile) {
        return Err(finish_post_spawn_output_setup_failure(
            guard,
            terminal_deadline,
            "stdout_file",
            EvalError::Invalid("injected stdout output-file creation failure".to_string()),
        ));
    }
    let stdout_file = match create_private_file(stdout_path) {
        Ok(file) => file,
        Err(error) => {
            return Err(finish_post_spawn_output_setup_failure(
                guard,
                terminal_deadline,
                "stdout_file",
                error,
            ))
        }
    };
    #[cfg(test)]
    if take_injected_provider_output_setup_failure(InjectedProviderOutputSetupFailure::StderrFile) {
        return Err(finish_post_spawn_output_setup_failure(
            guard,
            terminal_deadline,
            "stderr_file",
            EvalError::Invalid("injected stderr output-file creation failure".to_string()),
        ));
    }
    let stderr_file = match create_private_file(stderr_path) {
        Ok(file) => file,
        Err(error) => {
            return Err(finish_post_spawn_output_setup_failure(
                guard,
                terminal_deadline,
                "stderr_file",
                error,
            ))
        }
    };
    #[cfg(test)]
    if take_injected_provider_output_setup_failure(InjectedProviderOutputSetupFailure::StdoutPipe) {
        return Err(finish_post_spawn_output_setup_failure(
            guard,
            terminal_deadline,
            "stdout_pipe",
            EvalError::Invalid("injected stdout pipe ownership failure".to_string()),
        ));
    }
    let stdout = match guard.take_stdout() {
        Ok(Some(stdout)) => stdout,
        Ok(None) => {
            return Err(finish_post_spawn_output_setup_failure(
                guard,
                terminal_deadline,
                "stdout_pipe",
                EvalError::Invalid("provider stdout pipe is absent".to_string()),
            ))
        }
        Err(error) => {
            return Err(finish_post_spawn_output_setup_failure(
                guard,
                terminal_deadline,
                "stdout_pipe",
                EvalError::Invalid(format!("provider stdout ownership failed: {error}")),
            ))
        }
    };
    #[cfg(test)]
    if take_injected_provider_output_setup_failure(InjectedProviderOutputSetupFailure::StderrPipe) {
        return Err(finish_post_spawn_output_setup_failure(
            guard,
            terminal_deadline,
            "stderr_pipe",
            EvalError::Invalid("injected stderr pipe ownership failure".to_string()),
        ));
    }
    let stderr = match guard.take_stderr() {
        Ok(Some(stderr)) => stderr,
        Ok(None) => {
            return Err(finish_post_spawn_output_setup_failure(
                guard,
                terminal_deadline,
                "stderr_pipe",
                EvalError::Invalid("provider stderr pipe is absent".to_string()),
            ))
        }
        Err(error) => {
            return Err(finish_post_spawn_output_setup_failure(
                guard,
                terminal_deadline,
                "stderr_pipe",
                EvalError::Invalid(format!("provider stderr ownership failed: {error}")),
            ))
        }
    };
    #[cfg(test)]
    if take_injected_provider_output_setup_failure(
        InjectedProviderOutputSetupFailure::TerminalCleanup,
    ) {
        guard.inject_next_cleanup_attempt_failure_for_test();
    }
    let stdout_overflow = Arc::new(AtomicBool::new(false));
    let stderr_overflow = Arc::new(AtomicBool::new(false));
    let stdout_flag = Arc::clone(&stdout_overflow);
    let stderr_flag = Arc::clone(&stderr_overflow);
    let stdout_collector = std::thread::spawn(move || {
        copy_bounded_provider_stream(
            stdout,
            stdout_file,
            PROVIDER_STDOUT_LIMIT_BYTES,
            stdout_flag,
        )
    });
    let stderr_collector = std::thread::spawn(move || {
        copy_bounded_provider_stream(
            stderr,
            stderr_file,
            PROVIDER_STDERR_LIMIT_BYTES,
            stderr_flag,
        )
    });

    let mut timed_out = false;
    let mut limit_triggered = false;
    let mut observation_error = None;
    loop {
        if stdout_overflow.load(Ordering::Acquire) || stderr_overflow.load(Ordering::Acquire) {
            limit_triggered = true;
            break;
        }
        match guard.exited_without_reap(response_deadline) {
            Ok(true) => break,
            Ok(false) if Instant::now() < response_deadline => {
                std::thread::sleep(Duration::from_millis(10));
            }
            Ok(false) => {
                timed_out = true;
                break;
            }
            Err(error) => {
                observation_error = Some(EvalError::Invalid(format!(
                    "bounded provider observation failed: {error}"
                )));
                break;
            }
        }
    }
    let status = match guard.terminate_group_and_reap_with_status_until(terminal_deadline) {
        NativeExecutionCleanupOutcome::ProvenTerminal(status) => status,
        NativeExecutionCleanupOutcome::CleanupUnproven(error) => {
            return Err(BoundedProviderCommandFailure::PostSpawnCleanupUnproven {
                error: EvalError::Invalid(format!(
                    "bounded provider process-group cleanup remained unproven: {error}"
                )),
                quarantine: guard,
            })
        }
    };
    let (stdout_trace_bytes, stdout_limited) = stdout_collector
        .join()
        .map_err(|_| EvalError::Invalid("provider stdout collector panicked".to_string()))??;
    let (stderr_bytes, stderr_limited) = stderr_collector
        .join()
        .map_err(|_| EvalError::Invalid("provider stderr collector panicked".to_string()))??;
    limit_triggered |= stdout_limited || stderr_limited;
    if let Some(error) = observation_error {
        return Err(BoundedProviderCommandFailure::PostSpawn(error));
    }
    if timed_out || limit_triggered {
        return Err(BoundedProviderCommandFailure::PostSpawn(EvalError::Invalid(format!(
            "bounded provider limit triggered; timeout={timed_out}; stdout_bytes={stdout_trace_bytes}; stderr_bytes={stderr_bytes}; process_cleanup_proven=true"
        ))));
    }
    let exit_code = status.code().ok_or_else(|| {
        EvalError::Invalid("bounded provider exited without a numeric status".to_string())
    })?;
    Ok((
        exit_code,
        ProviderSafetyEvidence {
            stdout_trace_bytes,
            stderr_bytes,
            process_cleanup_proven: true,
        },
    ))
}

fn copy_bounded_provider_stream(
    mut stream: impl Read,
    mut output: File,
    limit: u64,
    overflow: Arc<AtomicBool>,
) -> EvalResult<(u64, bool)> {
    let mut observed = 0_u64;
    let mut written = 0_u64;
    let mut buffer = [0_u8; 16 * 1024];
    loop {
        let count = stream.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        observed = observed.saturating_add(count as u64);
        let remaining = limit.saturating_sub(written);
        let keep = usize::try_from(remaining.min(count as u64)).unwrap_or(count);
        if keep > 0 {
            output.write_all(&buffer[..keep])?;
            written += keep as u64;
        }
        if observed > limit {
            overflow.store(true, Ordering::Release);
        }
    }
    output.sync_all()?;
    Ok((observed, observed > limit))
}

fn validate_recoverable_claude_budget_trace(trace: &Path) -> EvalResult<ProviderCommandOutcome> {
    let summary = claude_trace_summary(trace)?;
    if !summary.accepted_turn_boundary_budget_exit {
        return Err(EvalError::Invalid(format!(
            "refusing to recover teaching trace without an exact Claude end-of-turn budget envelope: {}",
            trace.display()
        )));
    }
    Ok(summary)
}

fn validate_completed_claude_teaching_trace(trace: &Path) -> EvalResult<ProviderCommandOutcome> {
    let result = claude_terminal_result(trace)?;
    if result.get("type").and_then(Value::as_str) != Some("result")
        || result.get("subtype").and_then(Value::as_str) != Some("success")
        || result.get("is_error").and_then(Value::as_bool) != Some(false)
        || result.get("stop_reason").and_then(Value::as_str) != Some("end_turn")
    {
        return Err(EvalError::Invalid(format!(
            "refusing to recover teaching trace without an exact successful Claude terminal result: {}",
            trace.display()
        )));
    }
    claude_trace_summary(trace)
}

fn validate_recoverable_claude_evaluation_trace(
    trace: &Path,
    protocol_schema_version: u32,
) -> EvalResult<ProviderCommandOutcome> {
    let mut summary = claude_trace_summary(trace)?;
    if summary.accepted_turn_boundary_budget_exit {
        return Ok(summary);
    }
    let result = claude_terminal_result(trace)?;
    if !is_structured_output_budget_exhaustion(&result) {
        return Err(EvalError::Invalid(format!(
            "refusing to recover evaluation trace without an exact Claude structured-output budget envelope: {}",
            trace.display()
        )));
    }
    unique_structured_output_tool_input(trace, protocol_schema_version)?;
    summary.accepted_turn_boundary_budget_exit = true;
    Ok(summary)
}

fn claude_trace_summary(trace: &Path) -> EvalResult<ProviderCommandOutcome> {
    let result = claude_terminal_result(trace)?;
    let provider_reported_cost_microusd = result
        .get("total_cost_usd")
        .and_then(Value::as_f64)
        .map(cost_usd_to_microusd)
        .transpose()?;
    Ok(ProviderCommandOutcome {
        exit_code: 0,
        provider_reported_cost_microusd,
        accepted_turn_boundary_budget_exit: is_turn_boundary_budget_exhaustion(&result),
        safety: None,
    })
}

fn claude_terminal_result(trace: &Path) -> EvalResult<Value> {
    let reader = BufReader::new(File::open(trace)?);
    let mut final_result = None;
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(&line)?;
        if value.get("type").and_then(Value::as_str) == Some("result") {
            final_result = Some(value);
        }
    }
    final_result.ok_or_else(|| {
        EvalError::Invalid(format!(
            "Claude trace has no terminal result event: {}",
            trace.display()
        ))
    })
}

fn attest_claude_trace_model(
    trace: &Path,
    expected_requested_model: &str,
) -> EvalResult<ClaudeTraceModelAttestation> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;

        let metadata = fs::symlink_metadata(trace)?;
        if metadata.file_type().is_symlink()
            || !metadata.is_file()
            || metadata.uid() != unsafe { libc::geteuid() }
            || metadata.mode() & 0o077 != 0
            || metadata.nlink() != 1
            || metadata.len() == 0
            || metadata.len() > MAX_CODEX_SESSION_ROLLOUT_BYTES
        {
            return Err(EvalError::Invalid(
                "Claude trace failed its private bounded identity contract".to_string(),
            ));
        }
        let bytes = fs::read(trace)?;
        let after = fs::symlink_metadata(trace)?;
        if bytes.len() as u64 != metadata.len()
            || after.dev() != metadata.dev()
            || after.ino() != metadata.ino()
            || after.uid() != metadata.uid()
            || after.mode() != metadata.mode()
            || after.nlink() != metadata.nlink()
            || after.len() != metadata.len()
        {
            return Err(EvalError::Invalid(
                "Claude trace identity changed while model evidence was read".to_string(),
            ));
        }

        let mut init_models = Vec::new();
        let mut assistant_models = Vec::new();
        for (index, line) in bytes.split(|byte| *byte == b'\n').enumerate() {
            if line.iter().all(u8::is_ascii_whitespace) {
                continue;
            }
            let value: Value = serde_json::from_slice(line).map_err(|_| {
                EvalError::Invalid(format!("Claude trace line {} is invalid JSON", index + 1))
            })?;
            if value.get("type").and_then(Value::as_str) == Some("system")
                && value.get("subtype").and_then(Value::as_str) == Some("init")
            {
                let model = value.get("model").and_then(Value::as_str).ok_or_else(|| {
                    EvalError::Invalid("Claude init event omitted requested model".to_string())
                })?;
                if model.trim().is_empty() {
                    return Err(EvalError::Invalid(
                        "Claude init event reported an empty requested model".to_string(),
                    ));
                }
                init_models.push(model.to_string());
            }
            if value.get("type").and_then(Value::as_str) == Some("assistant") {
                let model = value
                    .pointer("/message/model")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        EvalError::Invalid(
                            "Claude assistant event omitted its host-resolved model".to_string(),
                        )
                    })?;
                if model.trim().is_empty() {
                    return Err(EvalError::Invalid(
                        "Claude assistant event reported an empty host-resolved model".to_string(),
                    ));
                }
                assistant_models.push(model.to_string());
            }
        }
        if init_models.len() != 1 || init_models[0] != expected_requested_model {
            return Err(EvalError::Invalid(format!(
                "Claude trace must contain exactly one init model equal to {expected_requested_model}"
            )));
        }
        let resolved = assistant_models.first().ok_or_else(|| {
            EvalError::Invalid("Claude trace contains no assistant model evidence".to_string())
        })?;
        if assistant_models.iter().any(|model| model != resolved) {
            return Err(EvalError::Invalid(
                "Claude assistant events disagree on the host-resolved model".to_string(),
            ));
        }
        Ok(ClaudeTraceModelAttestation {
            trace_sha256: sha256_bytes(&bytes),
            requested_model: init_models.remove(0),
            host_resolved_model: resolved.clone(),
        })
    }
    #[cfg(not(unix))]
    {
        let _ = (trace, expected_requested_model);
        Err(EvalError::Invalid(
            "Claude model attestation requires Unix trace ownership".to_string(),
        ))
    }
}

fn cost_usd_to_microusd(cost: f64) -> EvalResult<u64> {
    if !cost.is_finite() || cost < 0.0 || cost > u64::MAX as f64 / 1_000_000.0 {
        return Err(EvalError::Invalid(
            "Claude provider reported an invalid total_cost_usd".to_string(),
        ));
    }
    Ok((cost * 1_000_000.0).ceil() as u64)
}

fn is_turn_boundary_budget_exhaustion(result: &Value) -> bool {
    is_budget_exhaustion_with_stop_reason(result, "end_turn")
}

fn is_structured_output_budget_exhaustion(result: &Value) -> bool {
    is_budget_exhaustion_with_stop_reason(result, "tool_use")
}

fn is_budget_exhaustion_with_stop_reason(result: &Value, stop_reason: &str) -> bool {
    result.get("type").and_then(Value::as_str) == Some("result")
        && result.get("subtype").and_then(Value::as_str) == Some("error_max_budget_usd")
        && result.get("is_error").and_then(Value::as_bool) == Some(true)
        && result.get("terminal_reason").and_then(Value::as_str) == Some("budget_exhausted")
        && result.get("stop_reason").and_then(Value::as_str) == Some(stop_reason)
        && result
            .get("errors")
            .and_then(Value::as_array)
            .is_some_and(|errors| {
                errors.len() == 1
                    && errors[0]
                        .as_str()
                        .is_some_and(|error| error.starts_with("Reached maximum budget ($"))
            })
}

fn unique_structured_output_tool_input(
    trace: &Path,
    protocol_schema_version: u32,
) -> EvalResult<Value> {
    let output_contract = agent_output_contract(trace, protocol_schema_version)?;
    let reader = BufReader::new(File::open(trace)?);
    let mut candidates = Vec::new();
    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(&line).map_err(|error| {
            EvalError::Invalid(format!(
                "evaluation trace line {} is invalid JSON: {error}",
                index + 1
            ))
        })?;
        collect_structured_output_tool_inputs(
            &value,
            &mut candidates,
            protocol_schema_version,
            output_contract,
        );
    }
    let mut unique = BTreeMap::new();
    for candidate in candidates {
        unique.insert(serde_json::to_string(&candidate)?, candidate);
    }
    if unique.len() != 1 {
        return Err(EvalError::Invalid(format!(
            "evaluation trace must contain exactly one distinct StructuredOutput tool input, found {}",
            unique.len()
        )));
    }
    Ok(unique.into_values().next().expect("checked one value"))
}

fn collect_structured_output_tool_inputs(
    value: &Value,
    output: &mut Vec<Value>,
    protocol_schema_version: u32,
    output_contract: NativeAgentOutputContract,
) {
    if value.get("type").and_then(Value::as_str) == Some("tool_use")
        && value.get("name").and_then(Value::as_str) == Some("StructuredOutput")
    {
        if let Some(input) = value
            .get("input")
            .filter(|input| is_agent_output(input, protocol_schema_version, output_contract))
        {
            output.push(input.clone());
        }
    }
    match value {
        Value::Array(values) => {
            for value in values {
                collect_structured_output_tool_inputs(
                    value,
                    output,
                    protocol_schema_version,
                    output_contract,
                );
            }
        }
        Value::Object(values) => {
            for value in values.values() {
                collect_structured_output_tool_inputs(
                    value,
                    output,
                    protocol_schema_version,
                    output_contract,
                );
            }
        }
        _ => {}
    }
}

fn trace_path(lane: &PreparedNativeLane, phase: NativePilotRunPhase) -> EvalResult<&Path> {
    match phase {
        NativePilotRunPhase::Teaching => Ok(Path::new(&lane.teaching_trace_path)),
        NativePilotRunPhase::Activation => lane
            .activation_trace_path
            .as_deref()
            .map(Path::new)
            .ok_or_else(|| {
                EvalError::Invalid(format!(
                    "lane {} is missing activation trace path",
                    lane.order
                ))
            }),
        NativePilotRunPhase::Evaluation => Ok(Path::new(&lane.evaluation_trace_path)),
    }
}

pub(crate) fn stderr_path(trace: &Path) -> PathBuf {
    trace.with_extension("stderr.log")
}

fn candidate_path(lane: &PreparedNativeLane) -> EvalResult<PathBuf> {
    Ok(Path::new(&lane.teaching_trace_path)
        .parent()
        .ok_or_else(|| EvalError::Invalid("teaching trace has no parent".to_string()))?
        .join("procedure-candidates.json"))
}

fn candidate_stderr_path(lane: &PreparedNativeLane) -> EvalResult<PathBuf> {
    Ok(candidate_path(lane)?.with_extension("stderr.log"))
}

fn verification_stderr_path(output: &Path) -> PathBuf {
    output.with_extension("stderr.log")
}

fn cleanup_stdout_path(
    lane: &PreparedNativeLane,
    phase: NativePilotRunPhase,
) -> EvalResult<PathBuf> {
    Ok(trace_path(lane, phase)?
        .parent()
        .ok_or_else(|| EvalError::Invalid("phase trace has no parent".to_string()))?
        .join(format!("cleanup-{}.stdout.log", phase.label())))
}

fn cleanup_stderr_path(
    lane: &PreparedNativeLane,
    phase: NativePilotRunPhase,
) -> EvalResult<PathBuf> {
    Ok(trace_path(lane, phase)?
        .parent()
        .ok_or_else(|| EvalError::Invalid("phase trace has no parent".to_string()))?
        .join(format!("cleanup-{}.stderr.log", phase.label())))
}

fn phase_report_path(run_plan: &Path, phase: NativePilotRunPhase) -> EvalResult<PathBuf> {
    Ok(run_plan
        .parent()
        .ok_or_else(|| EvalError::Invalid("run plan has no parent".to_string()))?
        .join(format!("runner-{}.json", phase.label())))
}

fn require_absent(path: &Path, label: &str) -> EvalResult<()> {
    match fs::symlink_metadata(path) {
        Ok(_) => Err(EvalError::Invalid(format!(
            "refusing to overwrite {label}: {}",
            path.display()
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn require_existing_file(path: &Path, label: &str) -> EvalResult<()> {
    if !path.is_file() {
        return Err(EvalError::Invalid(format!(
            "required inherited {label} is missing: {}",
            path.display()
        )));
    }
    Ok(())
}

fn exact_recorded_claude_config_option<'a>(
    argv: &'a [String],
    option: &str,
) -> EvalResult<&'a str> {
    if argv
        .iter()
        .any(|argument| argument.starts_with(&format!("{option}=")))
    {
        return Err(EvalError::Invalid(format!(
            "recorded Claude config uses a non-canonical {option} form"
        )));
    }
    let positions = argv
        .iter()
        .enumerate()
        .filter_map(|(index, argument)| (argument == option).then_some(index))
        .collect::<Vec<_>>();
    if positions.len() != 1 {
        return Err(EvalError::Invalid(format!(
            "recorded Claude config requires exactly one {option}"
        )));
    }
    argv.get(positions[0] + 1)
        .filter(|value| !value.is_empty())
        .map(String::as_str)
        .ok_or_else(|| {
            EvalError::Invalid(format!(
                "recorded Claude config {option} has no inline JSON value"
            ))
        })
}

fn validate_recorded_claude_config_attestation(
    lane: &PreparedNativeLane,
    phase: NativePilotRunPhase,
    attestation: &NativeStaleV3ClaudeConfigAttestation,
) -> EvalResult<()> {
    let lane_root = Path::new(&lane.acceptance_contract)
        .parent()
        .ok_or_else(|| EvalError::Invalid("stale-safety lane has no root".to_string()))?;
    let argv = lane_phase_argv(lane, phase)?;
    let settings_inline = exact_recorded_claude_config_option(argv, "--settings")?;
    let mcp_inline = exact_recorded_claude_config_option(argv, "--mcp-config")?;
    let expected_settings = lane_root.join("claude-settings.json");
    let expected_mcp = lane_root.join(if lane.memory_layer.uses_engram() {
        "claude-mcp.json"
    } else {
        "claude-mcp-empty.json"
    });
    let owner_uid = current_effective_uid()?;
    let artifact_is_exact =
        |artifact: &crate::native_stale::NativeStaleV3ClaudeConfigArtifactAttestation,
         expected_path: &Path,
         inline: &str| {
            artifact.path == expected_path.display().to_string()
                && is_sha256(&artifact.sha256)
                && artifact.sha256 == sha256_bytes(inline.as_bytes())
                && artifact.device == attestation.lane_root.device
                && artifact.owner_uid == owner_uid
                && artifact.owner_uid == attestation.lane_root.owner_uid
                && artifact.owner_gid == attestation.lane_root.owner_gid
                && artifact.mode & 0o777 == 0o600
                && artifact.link_count == 1
                && artifact.size_bytes == inline.len() as u64
                && artifact.size_bytes > 0
        };
    if attestation.lane_root.path != lane_root.display().to_string()
        || attestation.lane_root.owner_uid != owner_uid
        || attestation.lane_root.mode & 0o777 != 0o700
        || !artifact_is_exact(&attestation.settings, &expected_settings, settings_inline)
        || !artifact_is_exact(&attestation.mcp, &expected_mcp, mcp_inline)
        || !is_sha256(&attestation.aggregate_sha256)
        || attestation.aggregate_sha256
            != sha256_bytes(&serde_json::to_vec(&(
                &attestation.lane_root,
                &attestation.settings,
                &attestation.mcp,
            ))?)
    {
        return Err(EvalError::Invalid(
            "recorded Claude config attestation is incomplete or drifted".to_string(),
        ));
    }
    Ok(())
}

#[derive(Debug)]
struct PrivateAuthLifecycleDirectory {
    directory: File,
    owner: u32,
    _lock: File,
}

impl PrivateAuthLifecycleDirectory {
    fn create_new(run_plan: &Path) -> EvalResult<Self> {
        #[cfg(not(unix))]
        {
            let _ = run_plan;
            return Err(EvalError::Invalid(
                "Codex auth-cache lifecycle requires a supported Unix host".to_string(),
            ));
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;

            let path = auth_lifecycle_path(run_plan)?;
            let mut builder = fs::DirBuilder::new();
            builder.mode(0o700);
            builder.create(&path).map_err(|error| {
                if error.kind() == std::io::ErrorKind::AlreadyExists {
                    EvalError::Invalid(
                        "auth-cache lifecycle residue exists; provisioning is non-replayable"
                            .to_string(),
                    )
                } else {
                    EvalError::Io(error)
                }
            })?;
            sync_directory(
                path.parent()
                    .ok_or_else(|| EvalError::Invalid("lifecycle has no parent".to_string()))?,
            )?;
            let (directory, owner) = open_private_directory(&path)?;
            let lock = create_relative_file(&directory, CODEX_AUTH_CACHE_LIFECYCLE_LOCK, 0o600)?;
            lock.sync_all()?;
            directory.sync_all()?;
            acquire_exclusive_file_lock(&lock)?;
            Ok(Self {
                directory,
                owner,
                _lock: lock,
            })
        }
    }

    fn open(run_plan: &Path) -> EvalResult<Self> {
        #[cfg(not(unix))]
        {
            let _ = run_plan;
            return Err(EvalError::Invalid(
                "Codex auth-cache lifecycle requires a supported Unix host".to_string(),
            ));
        }

        #[cfg(unix)]
        {
            let path = auth_lifecycle_path(run_plan)?;
            let (directory, owner) = open_private_directory(&path)?;
            let lock = open_relative_regular_file(
                &directory,
                CODEX_AUTH_CACHE_LIFECYCLE_LOCK,
                owner,
                true,
            )?;
            acquire_exclusive_file_lock(&lock)?;
            Ok(Self {
                directory,
                owner,
                _lock: lock,
            })
        }
    }

    fn open_without_lock_for_validation(run_plan: &Path) -> EvalResult<Self> {
        #[cfg(not(unix))]
        {
            let _ = run_plan;
            return Err(EvalError::Invalid(
                "Codex auth-cache lifecycle requires a supported Unix host".to_string(),
            ));
        }

        #[cfg(unix)]
        {
            let path = auth_lifecycle_path(run_plan)?;
            let (directory, owner) = open_private_directory(&path)?;
            let lock = open_relative_regular_file(
                &directory,
                CODEX_AUTH_CACHE_LIFECYCLE_LOCK,
                owner,
                true,
            )?;
            Ok(Self {
                directory,
                owner,
                _lock: lock,
            })
        }
    }

    fn write_json_pair_exclusive(&self, stem: &str, value: &impl Serialize) -> EvalResult<String> {
        let json_name = format!("{stem}.json");
        let digest_name = format!("{stem}.sha256");
        validate_lifecycle_entry_name(&json_name)?;
        validate_lifecycle_entry_name(&digest_name)?;
        let mut bytes = serde_json::to_vec_pretty(value)?;
        bytes.push(b'\n');
        let digest = sha256_bytes(&bytes);
        let mut file = create_relative_file(&self.directory, &json_name, 0o600)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        self.directory.sync_all()?;
        let mut digest_file = create_relative_file(&self.directory, &digest_name, 0o600)?;
        digest_file.write_all(format!("{digest}\n").as_bytes())?;
        digest_file.sync_all()?;
        self.directory.sync_all()?;
        Ok(digest)
    }

    fn read_json_pair<T: for<'de> Deserialize<'de>>(&self, stem: &str) -> EvalResult<(T, String)> {
        let json_name = format!("{stem}.json");
        let digest_name = format!("{stem}.sha256");
        validate_lifecycle_entry_name(&json_name)?;
        validate_lifecycle_entry_name(&digest_name)?;
        let mut json = open_relative_regular_file(&self.directory, &json_name, self.owner, false)?;
        let metadata = json.metadata()?;
        if metadata.len() == 0 || metadata.len() > MAX_CODEX_AUTH_FILE_BYTES {
            return Err(EvalError::Invalid(format!(
                "auth-cache lifecycle entry {json_name} has an invalid size"
            )));
        }
        let mut bytes = Vec::new();
        (&mut json)
            .take(MAX_CODEX_AUTH_FILE_BYTES + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 != metadata.len() {
            return Err(EvalError::Invalid(format!(
                "auth-cache lifecycle entry {json_name} changed while reading"
            )));
        }
        let observed = sha256_bytes(&bytes);
        let mut digest_file =
            open_relative_regular_file(&self.directory, &digest_name, self.owner, false)?;
        let mut digest_bytes = Vec::new();
        (&mut digest_file).take(66).read_to_end(&mut digest_bytes)?;
        let expected = std::str::from_utf8(&digest_bytes)
            .map_err(|_| EvalError::Invalid("lifecycle digest is not UTF-8".to_string()))?
            .trim();
        if !is_sha256(expected) || expected != observed {
            return Err(EvalError::Invalid(format!(
                "auth-cache lifecycle entry {json_name} failed digest validation"
            )));
        }
        Ok((serde_json::from_slice(&bytes)?, observed))
    }

    fn entry_exists(&self, name: &str) -> EvalResult<bool> {
        validate_lifecycle_entry_name(name)?;
        relative_entry_exists(&self.directory, name)
    }
}

#[derive(Debug)]
struct CodexAuthCacheLifecycle {
    directory: PrivateAuthLifecycleDirectory,
    receipt: Option<CodexAuthCacheProvisionReceipt>,
    manifest_sha256: String,
    active_phase: Option<(NativePilotRunPhase, String, Vec<u32>)>,
}

impl CodexAuthCacheLifecycle {
    fn create_new(run_plan: &Path) -> EvalResult<Self> {
        Ok(Self {
            directory: PrivateAuthLifecycleDirectory::create_new(run_plan)?,
            receipt: None,
            manifest_sha256: String::new(),
            active_phase: None,
        })
    }

    fn open(plan: &PreparedNativePilot, run_plan: &Path) -> EvalResult<Self> {
        Self::from_directory(
            plan,
            run_plan,
            PrivateAuthLifecycleDirectory::open(run_plan)?,
        )
    }

    fn open_without_lock_for_validation(
        plan: &PreparedNativePilot,
        run_plan: &Path,
    ) -> EvalResult<Self> {
        Self::from_directory(
            plan,
            run_plan,
            PrivateAuthLifecycleDirectory::open_without_lock_for_validation(run_plan)?,
        )
    }

    fn from_directory(
        plan: &PreparedNativePilot,
        run_plan: &Path,
        directory: PrivateAuthLifecycleDirectory,
    ) -> EvalResult<Self> {
        if directory.entry_exists("provision-unresolved.json")? {
            return Err(EvalError::Invalid(
                "auth-cache provisioning has durable unresolved residue".to_string(),
            ));
        }
        let (intent, intent_sha256): (CodexAuthCacheProvisionIntent, String) =
            directory.read_json_pair("provision-intent")?;
        let (receipt, manifest_sha256): (CodexAuthCacheProvisionReceipt, String) =
            directory.read_json_pair("provision-receipt")?;
        let run_plan_sha256 = sha256_file(run_plan)?;
        if intent.schema_version != CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION
            || receipt.schema_version != CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION
            || intent.pilot_id != plan.pilot_id
            || receipt.pilot_id != plan.pilot_id
            || intent.run_plan_sha256 != run_plan_sha256
            || receipt.run_plan_sha256 != run_plan_sha256
            || normalize_boot_id(&intent.boot_id).ok().as_deref() != Some(intent.boot_id.as_str())
            || receipt.boot_id != intent.boot_id
            || receipt.provision_intent_sha256 != intent_sha256
            || !receipt.authentication_ready
            || receipt.destinations.len() != 6
        {
            return Err(EvalError::Invalid(
                "auth-cache lifecycle provision receipt is invalid or from another boot"
                    .to_string(),
            ));
        }
        let expected = exact_six_codex_auth_targets(plan)?;
        if intent.destinations.len() != 6
            || intent
                .destinations
                .iter()
                .zip(&expected)
                .any(|(recorded, target)| {
                    recorded.lane_order != target.lane_order
                        || recorded.auth_json != target.auth_json.display().to_string()
                })
            || receipt
                .destinations
                .iter()
                .zip(&expected)
                .any(|(recorded, target)| {
                    recorded.lane_order != target.lane_order
                        || recorded.auth_json != target.auth_json.display().to_string()
                })
        {
            return Err(EvalError::Invalid(
                "auth-cache lifecycle destination manifest does not bind the exact six lanes"
                    .to_string(),
            ));
        }
        Ok(Self {
            directory,
            receipt: Some(receipt),
            manifest_sha256,
            active_phase: None,
        })
    }

    fn receipt(&self) -> EvalResult<&CodexAuthCacheProvisionReceipt> {
        self.receipt.as_ref().ok_or_else(|| {
            EvalError::Invalid("auth-cache lifecycle has no provision receipt".to_string())
        })
    }

    fn validate_current_destinations(&self, plan: &PreparedNativePilot) -> EvalResult<()> {
        if lifecycle_pair_state(&self.directory, "cleanup-intent")? != LifecyclePairState::Absent
            || lifecycle_pair_state(&self.directory, "cleanup-receipt")?
                != LifecyclePairState::Absent
        {
            return Err(EvalError::Invalid(
                "auth-cache lifecycle cleanup has started; authentication cannot be reused"
                    .to_string(),
            ));
        }
        require_same_boot(&self.receipt()?.boot_id, &current_boot_id()?)?;
        let targets = exact_six_codex_auth_targets(plan)?;
        let effective_uid = current_effective_uid()?;
        validate_destination_receipts(&targets, &self.receipt()?.destinations, effective_uid)
    }

    fn begin_phase(
        &mut self,
        plan: &PreparedNativePilot,
        run_plan: &Path,
        phase: NativePilotRunPhase,
    ) -> EvalResult<()> {
        if self.active_phase.is_some() {
            return Err(EvalError::Invalid(
                "auth-cache lifecycle already has an active provider phase".to_string(),
            ));
        }
        self.validate_current_destinations(plan)?;
        for predecessor in phase_predecessors(phase) {
            self.validate_phase_receipt(plan, run_plan, predecessor)?;
        }
        let stem = phase_lifecycle_stem(phase, "intent");
        let lane_orders = expected_phase_lane_orders(plan, phase);
        let mut residue_stems = vec![stem.clone(), phase_lifecycle_stem(phase, "receipt")];
        for lane_order in &lane_orders {
            residue_stems.push(lane_lifecycle_stem(phase, *lane_order, "admission"));
            residue_stems.push(lane_lifecycle_stem(phase, *lane_order, "terminal"));
        }
        for entry in residue_stems {
            if lifecycle_pair_state(&self.directory, &entry)? != LifecyclePairState::Absent {
                return Err(EvalError::Invalid(format!(
                    "{} provider phase has durable admission residue and is non-replayable",
                    phase.label()
                )));
            }
        }
        let intent = CodexAuthCachePhaseIntent {
            schema_version: CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION,
            pilot_id: plan.pilot_id.clone(),
            run_plan_sha256: sha256_file(run_plan)?,
            lifecycle_manifest_sha256: self.manifest_sha256.clone(),
            boot_id: current_boot_id()?,
            phase,
            lane_orders: lane_orders.clone(),
            created_unix_ms: unix_ms(SystemTime::now())?,
        };
        let digest = self.directory.write_json_pair_exclusive(&stem, &intent)?;
        self.active_phase = Some((phase, digest, lane_orders));
        Ok(())
    }

    fn admit_lane(
        &self,
        plan: &PreparedNativePilot,
        run_plan: &Path,
        lane: &PreparedNativeLane,
        phase: NativePilotRunPhase,
        claude_config_artifacts: Option<&NativeStaleV3ClaudeConfigAttestation>,
    ) -> Result<String, ProviderLaneFailure> {
        self.validate_current_destinations(plan)
            .map_err(ProviderLaneFailure::pre_dispatch_runner_failure)?;
        let (active_phase, _, expected_orders) = self
            .active_phase
            .as_ref()
            .ok_or_else(|| {
                EvalError::Invalid("provider lane admission has no active phase intent".to_string())
            })
            .map_err(ProviderLaneFailure::pre_dispatch_runner_failure)?;
        if *active_phase != phase || !expected_orders.contains(&lane.order) {
            return Err(ProviderLaneFailure::pre_dispatch_runner_failure(
                EvalError::Invalid(
                    "provider lane admission is outside the active phase intent".to_string(),
                ),
            ));
        }
        let argv = lane_phase_argv(lane, phase)
            .map_err(ProviderLaneFailure::pre_dispatch_runner_failure)?;
        let claude_config_artifacts = match lane.host.as_str() {
            "claude_code" => {
                let expected = claude_config_artifacts.ok_or_else(|| {
                    ProviderLaneFailure::pre_dispatch_config_artifact_drift(EvalError::Invalid(
                        format!(
                            "Claude lane {} admission omitted its config attestation",
                            lane.order
                        ),
                    ))
                })?;
                let observed = validate_native_stale_v3_claude_config_artifacts(lane)
                    .map_err(ProviderLaneFailure::pre_dispatch_config_artifact_drift)?;
                if observed != *expected {
                    return Err(ProviderLaneFailure::pre_dispatch_config_artifact_drift(
                        EvalError::Invalid(format!(
                            "Claude lane {} config artifacts changed before lifecycle admission",
                            lane.order
                        )),
                    ));
                }
                Some(expected.clone())
            }
            _ if claude_config_artifacts.is_some() => {
                return Err(ProviderLaneFailure::pre_dispatch_config_artifact_drift(
                    EvalError::Invalid(format!(
                        "non-Claude lane {} admission received a Claude config attestation",
                        lane.order
                    )),
                ))
            }
            _ => None,
        };
        let codex_pre_dispatch_inventory = if lane.host == "codex" {
            Some(
                capture_codex_pre_dispatch_inventory(lane)
                    .map_err(ProviderLaneFailure::pre_dispatch_runner_failure)?,
            )
        } else {
            None
        };
        let admission = CodexAuthCacheLaneAdmission {
            schema_version: CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION,
            pilot_id: plan.pilot_id.clone(),
            run_plan_sha256: sha256_file(run_plan)
                .map_err(ProviderLaneFailure::pre_dispatch_runner_failure)?,
            lifecycle_manifest_sha256: self.manifest_sha256.clone(),
            boot_id: current_boot_id().map_err(ProviderLaneFailure::pre_dispatch_runner_failure)?,
            phase,
            lane_order: lane.order,
            argv_sha256: argv_sha256(argv)
                .map_err(ProviderLaneFailure::pre_dispatch_runner_failure)?,
            claude_config_artifacts,
            consumption_state: "dispatch_authorized_consumption_not_inferred".to_string(),
            codex_pre_dispatch_inventory,
            admitted_unix_ms: unix_ms(SystemTime::now())
                .map_err(ProviderLaneFailure::pre_dispatch_runner_failure)?,
        };
        self.directory
            .write_json_pair_exclusive(
                &lane_lifecycle_stem(phase, lane.order, "admission"),
                &admission,
            )
            .map_err(ProviderLaneFailure::pre_dispatch_runner_failure)
    }

    fn seal_lane(
        &self,
        plan: &PreparedNativePilot,
        run_plan: &Path,
        execution: &mut NativePilotLaneExecution,
        phase: NativePilotRunPhase,
    ) -> EvalResult<String> {
        let lane = plan
            .lanes
            .iter()
            .find(|lane| lane.order == execution.order)
            .ok_or_else(|| {
                EvalError::Invalid("provider lane is absent from the plan".to_string())
            })?;
        let (admission, admission_sha256) =
            self.read_validated_lane_admission(plan, run_plan, lane, phase)?;
        if admission.argv_sha256 != execution.argv_sha256 {
            return Err(EvalError::Invalid(
                "provider lane execution argv drifted from its admission".to_string(),
            ));
        }
        let expected_claude_config_sha256 = admission
            .claude_config_artifacts
            .as_ref()
            .map(|attestation| attestation.aggregate_sha256.as_str());
        if execution.claude_config_artifacts_sha256.as_deref() != expected_claude_config_sha256 {
            return Err(EvalError::Invalid(
                "provider lane execution Claude config drifted from its admission".to_string(),
            ));
        }
        let codex_rollout = if execution.host == "codex" {
            Some(attest_codex_session_rollout(
                lane,
                phase,
                Path::new(&execution.trace_path),
                admission
                    .codex_pre_dispatch_inventory
                    .as_ref()
                    .ok_or_else(|| {
                        EvalError::Invalid(
                            "Codex lane admission omitted its pre-dispatch session inventory"
                                .to_string(),
                        )
                    })?,
            )?)
        } else {
            None
        };
        if let Some(attestation) = codex_rollout.as_ref() {
            execution.codex_session_rollout_path = Some(attestation.path.clone());
            execution.codex_session_rollout_sha256 = Some(attestation.sha256.clone());
            execution.codex_model_provider = Some(attestation.model_provider.clone());
            execution.codex_host_resolved_model = Some(attestation.host_resolved_model.clone());
            execution.codex_agents_md_sha256 = Some(attestation.agents_md_sha256.clone());
        }
        let provider_trace_sha256 = sha256_file(Path::new(&execution.trace_path))?;
        if execution.provider_trace_sha256.as_deref() != Some(provider_trace_sha256.as_str()) {
            return Err(EvalError::Invalid(
                "provider trace digest changed before terminal sealing".to_string(),
            ));
        }
        if execution.host == "claude_code" {
            let attestation =
                attest_claude_trace_model(Path::new(&execution.trace_path), &plan.claude_model)?;
            if execution.claude_requested_model.as_deref()
                != Some(attestation.requested_model.as_str())
                || execution.claude_host_resolved_model.as_deref()
                    != Some(attestation.host_resolved_model.as_str())
                || provider_trace_sha256 != attestation.trace_sha256
            {
                return Err(EvalError::Invalid(
                    "Claude model evidence changed before terminal sealing".to_string(),
                ));
            }
        }
        let terminal = CodexAuthCacheLaneTerminalReceipt {
            schema_version: CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION,
            pilot_id: plan.pilot_id.clone(),
            run_plan_sha256: admission.run_plan_sha256,
            lifecycle_manifest_sha256: self.manifest_sha256.clone(),
            boot_id: admission.boot_id,
            phase,
            lane_order: execution.order,
            argv_sha256: execution.argv_sha256.clone(),
            lane_admission_sha256: admission_sha256,
            provider_started_unix_ms: execution.provider_started_unix_ms,
            provider_completed_unix_ms: execution.provider_completed_unix_ms,
            exit_code: execution.exit_code,
            codex_session_rollout_path: execution.codex_session_rollout_path.clone(),
            codex_session_rollout_sha256: execution.codex_session_rollout_sha256.clone(),
            codex_model_provider: execution.codex_model_provider.clone(),
            codex_host_resolved_model: execution.codex_host_resolved_model.clone(),
            codex_agents_md_sha256: execution.codex_agents_md_sha256.clone(),
            provider_trace_sha256: execution.provider_trace_sha256.clone(),
            effective_environment_sha256: execution.effective_environment_sha256.clone(),
            claude_config_artifacts_sha256: execution.claude_config_artifacts_sha256.clone(),
            stdout_trace_bytes: execution.stdout_trace_bytes,
            stderr_bytes: execution.stderr_bytes,
            process_cleanup_proven: execution.process_cleanup_proven,
            provider_reported_cost_microusd: execution.provider_reported_cost_microusd,
            accepted_turn_boundary_budget_exit: execution.accepted_turn_boundary_budget_exit,
            claude_requested_model: execution.claude_requested_model.clone(),
            claude_host_resolved_model: execution.claude_host_resolved_model.clone(),
            sealed_unix_ms: unix_ms(SystemTime::now())?,
        };
        self.directory.write_json_pair_exclusive(
            &lane_lifecycle_stem(phase, execution.order, "terminal"),
            &terminal,
        )
    }

    fn read_validated_lane_admission(
        &self,
        plan: &PreparedNativePilot,
        run_plan: &Path,
        lane: &PreparedNativeLane,
        phase: NativePilotRunPhase,
    ) -> EvalResult<(CodexAuthCacheLaneAdmission, String)> {
        let (admission, admission_sha256) =
            self.read_validated_lane_admission_recorded(plan, run_plan, lane, phase)?;
        let expected_claude_config_artifacts = if lane.host == "claude_code" {
            Some(validate_native_stale_v3_claude_config_artifacts(lane)?)
        } else {
            None
        };
        if admission.claude_config_artifacts != expected_claude_config_artifacts {
            return Err(EvalError::Invalid(
                "provider lane admission Claude config provenance changed".to_string(),
            ));
        }
        Ok((admission, admission_sha256))
    }

    fn read_validated_lane_admission_recorded(
        &self,
        plan: &PreparedNativePilot,
        run_plan: &Path,
        lane: &PreparedNativeLane,
        phase: NativePilotRunPhase,
    ) -> EvalResult<(CodexAuthCacheLaneAdmission, String)> {
        let intent_stem = phase_lifecycle_stem(phase, "intent");
        let (intent, intent_sha256): (CodexAuthCachePhaseIntent, String) =
            self.directory.read_json_pair(&intent_stem)?;
        let expected_orders = expected_phase_lane_orders(plan, phase);
        let (admission, admission_sha256): (CodexAuthCacheLaneAdmission, String) =
            self.directory
                .read_json_pair(&lane_lifecycle_stem(phase, lane.order, "admission"))?;
        let recorded_claude_config_is_valid = match (
            lane.host.as_str(),
            admission.claude_config_artifacts.as_ref(),
        ) {
            ("claude_code", Some(attestation)) => {
                validate_recorded_claude_config_attestation(lane, phase, attestation).is_ok()
            }
            ("claude_code", None) => false,
            (_, None) => true,
            (_, Some(_)) => false,
        };
        let active_phase_is_valid = match self.active_phase.as_ref() {
            None => true,
            Some((active_phase, active_intent_sha256, active_orders)) => {
                *active_phase != phase
                    || (active_intent_sha256 == &intent_sha256 && active_orders == &expected_orders)
            }
        };
        if intent.schema_version != CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION
            || intent.pilot_id != plan.pilot_id
            || intent.run_plan_sha256 != sha256_file(run_plan)?
            || intent.lifecycle_manifest_sha256 != self.manifest_sha256
            || intent.boot_id != current_boot_id()?
            || intent.phase != phase
            || intent.lane_orders != expected_orders
            || !intent.lane_orders.contains(&lane.order)
            || !active_phase_is_valid
            || admission.schema_version != CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION
            || admission.pilot_id != plan.pilot_id
            || admission.run_plan_sha256 != sha256_file(run_plan)?
            || admission.lifecycle_manifest_sha256 != self.manifest_sha256
            || admission.boot_id != current_boot_id()?
            || admission.phase != phase
            || admission.lane_order != lane.order
            || admission.argv_sha256 != argv_sha256(lane_phase_argv(lane, phase)?)?
            || !recorded_claude_config_is_valid
            || admission.consumption_state != "dispatch_authorized_consumption_not_inferred"
            || (lane.host == "codex") != admission.codex_pre_dispatch_inventory.is_some()
            || intent.created_unix_ms == 0
            || admission.admitted_unix_ms < intent.created_unix_ms
        {
            return Err(EvalError::Invalid(
                "provider lane admission is incomplete or drifted".to_string(),
            ));
        }
        Ok((admission, admission_sha256))
    }

    fn validated_bundle_pre_provider_receipt_evidence(
        &self,
        plan: &PreparedNativePilot,
        run_plan: &Path,
        lane: &PreparedNativeLane,
        phase: NativePilotRunPhase,
        admission_ordinal: u32,
        expected_receipt_sha256: &str,
    ) -> EvalResult<NativeProviderBundlePreProviderReceiptEvidence> {
        if family_v3_treatment_phase_lane(plan, admission_ordinal)? != (phase, lane.order) {
            return Err(EvalError::Invalid(
                "family-v3 pre-provider receipt ordinal does not map to its phase lane".to_string(),
            ));
        }
        let (admission, receipt_sha256) =
            self.read_validated_lane_admission_recorded(plan, run_plan, lane, phase)?;
        if receipt_sha256 != expected_receipt_sha256 {
            return Err(EvalError::Invalid(
                "family-v3 pre-provider evidence does not bind the exact admission receipt bytes"
                    .to_string(),
            ));
        }
        Ok(NativeProviderBundlePreProviderReceiptEvidence {
            ordinal: admission_ordinal,
            child: "treatment".to_string(),
            phase: phase.label().to_string(),
            lane_order: lane.order,
            receipt_sha256,
            argv_sha256: admission.argv_sha256.clone(),
            receipt_unix_ms: admission.admitted_unix_ms,
            typed_binding_sha256: sha256_bytes(&serde_json::to_vec(&admission)?),
        })
    }

    fn complete_phase(
        &mut self,
        plan: &PreparedNativePilot,
        run_plan: &Path,
        phase: NativePilotRunPhase,
        report_path: &Path,
    ) -> EvalResult<()> {
        self.validate_current_destinations(plan)?;
        let (active_phase, phase_intent_sha256, lane_orders) =
            self.active_phase.take().ok_or_else(|| {
                EvalError::Invalid("provider phase completion has no active intent".to_string())
            })?;
        if active_phase != phase {
            return Err(EvalError::Invalid(
                "provider phase completion does not match its active intent".to_string(),
            ));
        }
        let report: NativePilotRunReport = serde_json::from_reader(File::open(report_path)?)?;
        validate_stale_phase_receipt(plan, run_plan, phase, &report)?;
        let mut lane_bindings = Vec::with_capacity(lane_orders.len());
        for lane_order in &lane_orders {
            let (terminal, terminal_receipt_sha256): (CodexAuthCacheLaneTerminalReceipt, String) =
                self.directory.read_json_pair(&lane_lifecycle_stem(
                    phase,
                    *lane_order,
                    "terminal",
                ))?;
            self.validate_lane_terminal_receipt(plan, run_plan, phase, *lane_order, &terminal)?;
            let execution = report
                .lanes
                .iter()
                .find(|execution| execution.order == *lane_order)
                .ok_or_else(|| {
                    EvalError::Invalid(
                        "provider phase report omitted a terminal lane binding".to_string(),
                    )
                })?;
            if execution.argv_sha256 != terminal.argv_sha256
                || execution.provider_started_unix_ms != terminal.provider_started_unix_ms
                || execution.provider_completed_unix_ms != terminal.provider_completed_unix_ms
                || execution.exit_code != terminal.exit_code
                || execution.codex_session_rollout_path != terminal.codex_session_rollout_path
                || execution.codex_session_rollout_sha256 != terminal.codex_session_rollout_sha256
                || execution.codex_model_provider != terminal.codex_model_provider
                || execution.codex_host_resolved_model != terminal.codex_host_resolved_model
                || execution.codex_agents_md_sha256 != terminal.codex_agents_md_sha256
                || execution.provider_trace_sha256 != terminal.provider_trace_sha256
                || execution.effective_environment_sha256 != terminal.effective_environment_sha256
                || execution.claude_config_artifacts_sha256
                    != terminal.claude_config_artifacts_sha256
                || execution.stdout_trace_bytes != terminal.stdout_trace_bytes
                || execution.stderr_bytes != terminal.stderr_bytes
                || execution.process_cleanup_proven != terminal.process_cleanup_proven
                || execution.provider_reported_cost_microusd
                    != terminal.provider_reported_cost_microusd
                || execution.accepted_turn_boundary_budget_exit
                    != terminal.accepted_turn_boundary_budget_exit
                || execution.claude_requested_model != terminal.claude_requested_model
                || execution.claude_host_resolved_model != terminal.claude_host_resolved_model
            {
                return Err(EvalError::Invalid(
                    "provider phase report drifted from its terminal lane receipt".to_string(),
                ));
            }
            lane_bindings.push(phase_lane_binding(&terminal, terminal_receipt_sha256));
        }
        let receipt = CodexAuthCachePhaseReceipt {
            schema_version: CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION,
            pilot_id: plan.pilot_id.clone(),
            run_plan_sha256: sha256_file(run_plan)?,
            lifecycle_manifest_sha256: self.manifest_sha256.clone(),
            boot_id: current_boot_id()?,
            phase,
            phase_intent_sha256,
            runner_report_sha256: sha256_file(report_path)?,
            lane_orders,
            lane_bindings,
            completed_unix_ms: unix_ms(SystemTime::now())?,
        };
        self.directory
            .write_json_pair_exclusive(&phase_lifecycle_stem(phase, "receipt"), &receipt)?;
        Ok(())
    }

    fn validate_lane_terminal_receipt(
        &self,
        plan: &PreparedNativePilot,
        run_plan: &Path,
        phase: NativePilotRunPhase,
        lane_order: u32,
        terminal: &CodexAuthCacheLaneTerminalReceipt,
    ) -> EvalResult<()> {
        let lane = plan
            .lanes
            .iter()
            .find(|lane| lane.order == lane_order)
            .ok_or_else(|| {
                EvalError::Invalid("terminal receipt lane is not planned".to_string())
            })?;
        let (admission, admission_sha256) =
            self.read_validated_lane_admission(plan, run_plan, lane, phase)?;
        let expected_claude_config_sha256 = admission
            .claude_config_artifacts
            .as_ref()
            .map(|attestation| attestation.aggregate_sha256.as_str());
        if terminal.schema_version != CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION
            || terminal.pilot_id != plan.pilot_id
            || terminal.run_plan_sha256 != sha256_file(run_plan)?
            || terminal.lifecycle_manifest_sha256 != self.manifest_sha256
            || terminal.boot_id != current_boot_id()?
            || terminal.phase != phase
            || terminal.lane_order != lane_order
            || terminal.argv_sha256 != admission.argv_sha256
            || terminal.lane_admission_sha256 != admission_sha256
            || terminal.provider_started_unix_ms.is_none()
            || terminal.provider_completed_unix_ms.is_none()
            || terminal.provider_started_unix_ms > terminal.provider_completed_unix_ms
            || terminal.effective_environment_sha256.as_deref()
                != Some(canonical_environment_sha256(&lane.environment)?.as_str())
            || terminal.claude_config_artifacts_sha256.as_deref() != expected_claude_config_sha256
            || terminal.stdout_trace_bytes != Some(fs::metadata(trace_path(lane, phase)?)?.len())
            || terminal.stderr_bytes
                != Some(fs::metadata(stderr_path(trace_path(lane, phase)?))?.len())
            || terminal.process_cleanup_proven != Some(true)
            || terminal.provider_trace_sha256.as_deref()
                != Some(sha256_file(trace_path(lane, phase)?)?.as_str())
        {
            return Err(EvalError::Invalid(
                "provider lane terminal receipt is incomplete or drifted".to_string(),
            ));
        }
        if lane.host == "codex" {
            if terminal.claude_config_artifacts_sha256.is_some()
                || terminal.provider_reported_cost_microusd.is_some()
                || terminal.accepted_turn_boundary_budget_exit
            {
                return Err(EvalError::Invalid(
                    "Codex terminal receipt contains Claude cost evidence".to_string(),
                ));
            }
            let recorded = CodexSessionRolloutAttestation {
                path: terminal.codex_session_rollout_path.clone().ok_or_else(|| {
                    EvalError::Invalid("Codex terminal receipt omitted rollout path".to_string())
                })?,
                sha256: terminal
                    .codex_session_rollout_sha256
                    .clone()
                    .ok_or_else(|| {
                        EvalError::Invalid(
                            "Codex terminal receipt omitted rollout digest".to_string(),
                        )
                    })?,
                model_provider: terminal.codex_model_provider.clone().ok_or_else(|| {
                    EvalError::Invalid("Codex terminal receipt omitted model provider".to_string())
                })?,
                host_resolved_model: terminal.codex_host_resolved_model.clone().ok_or_else(
                    || {
                        EvalError::Invalid(
                            "Codex terminal receipt omitted host-resolved model".to_string(),
                        )
                    },
                )?,
                agents_md_sha256: terminal.codex_agents_md_sha256.clone().ok_or_else(|| {
                    EvalError::Invalid(
                        "Codex terminal receipt omitted agents instruction digest".to_string(),
                    )
                })?,
            };
            validate_recorded_codex_session_rollout(
                lane,
                phase,
                trace_path(lane, phase)?,
                &recorded,
            )?;
            if terminal.claude_requested_model.is_some()
                || terminal.claude_host_resolved_model.is_some()
            {
                return Err(EvalError::Invalid(
                    "Codex terminal receipt contains Claude model evidence".to_string(),
                ));
            }
        } else if terminal.codex_session_rollout_path.is_some()
            || terminal.codex_session_rollout_sha256.is_some()
            || terminal.codex_model_provider.is_some()
            || terminal.codex_host_resolved_model.is_some()
            || terminal.codex_agents_md_sha256.is_some()
        {
            return Err(EvalError::Invalid(
                "non-Codex lane contains a Codex session attestation".to_string(),
            ));
        } else {
            if terminal.claude_config_artifacts_sha256.is_none() {
                return Err(EvalError::Invalid(
                    "Claude terminal receipt omitted its config artifact binding".to_string(),
                ));
            }
            let observed = attest_claude_trace_model(trace_path(lane, phase)?, &plan.claude_model)?;
            let summary = claude_trace_summary(trace_path(lane, phase)?)?;
            if terminal.claude_requested_model.as_deref() != Some(observed.requested_model.as_str())
                || terminal.claude_host_resolved_model.as_deref()
                    != Some(observed.host_resolved_model.as_str())
                || terminal.provider_trace_sha256.as_deref() != Some(observed.trace_sha256.as_str())
                || terminal.provider_reported_cost_microusd
                    != summary.provider_reported_cost_microusd
                || terminal.accepted_turn_boundary_budget_exit
                    != summary.accepted_turn_boundary_budget_exit
                || terminal.provider_reported_cost_microusd.is_none()
            {
                return Err(EvalError::Invalid(
                    "Claude terminal receipt model or complete-trace evidence drifted".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn validated_treatment_predecessor_evidence(
        &self,
        plan: &PreparedNativePilot,
        run_plan: &Path,
        next_admission_ordinal: u32,
    ) -> EvalResult<Vec<NativeProviderBundlePredecessorEvidence>> {
        if !(1..=30).contains(&next_admission_ordinal) {
            return Err(EvalError::Invalid(
                "family-v3 treatment admission ordinal is outside 1..=30".to_string(),
            ));
        }
        let mut predecessors = Vec::with_capacity(
            usize::try_from(next_admission_ordinal - 1)
                .map_err(|_| EvalError::Invalid("predecessor count overflow".to_string()))?,
        );
        for ordinal in 1..next_admission_ordinal {
            let (phase, lane_order) = family_v3_treatment_phase_lane(plan, ordinal)?;
            let terminal_stem = lane_lifecycle_stem(phase, lane_order, "terminal");
            let (terminal, terminal_receipt_sha256): (CodexAuthCacheLaneTerminalReceipt, String) =
                self.directory.read_json_pair(&terminal_stem)?;
            self.validate_lane_terminal_receipt(plan, run_plan, phase, lane_order, &terminal)?;
            predecessors.push(NativeProviderBundlePredecessorEvidence {
                ordinal,
                child: "treatment".to_string(),
                terminal_receipt_sha256,
                provider_started_unix_ms: terminal.provider_started_unix_ms.ok_or_else(|| {
                    EvalError::Invalid(
                        "family-v3 treatment predecessor omitted provider start".to_string(),
                    )
                })?,
                provider_completed_unix_ms: terminal.provider_completed_unix_ms.ok_or_else(
                    || {
                        EvalError::Invalid(
                            "family-v3 treatment predecessor omitted provider completion"
                                .to_string(),
                        )
                    },
                )?,
            });
        }
        Ok(predecessors)
    }

    fn validate_phase_receipt(
        &self,
        plan: &PreparedNativePilot,
        run_plan: &Path,
        phase: NativePilotRunPhase,
    ) -> EvalResult<()> {
        let (intent, intent_sha256): (CodexAuthCachePhaseIntent, String) = self
            .directory
            .read_json_pair(&phase_lifecycle_stem(phase, "intent"))?;
        let (receipt, _): (CodexAuthCachePhaseReceipt, String) = self
            .directory
            .read_json_pair(&phase_lifecycle_stem(phase, "receipt"))?;
        let expected_orders = expected_phase_lane_orders(plan, phase);
        let report_path = phase_report_path(run_plan, phase)?;
        if intent.schema_version != CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION
            || receipt.schema_version != CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION
            || intent.pilot_id != plan.pilot_id
            || receipt.pilot_id != plan.pilot_id
            || intent.run_plan_sha256 != sha256_file(run_plan)?
            || receipt.run_plan_sha256 != intent.run_plan_sha256
            || intent.lifecycle_manifest_sha256 != self.manifest_sha256
            || receipt.lifecycle_manifest_sha256 != self.manifest_sha256
            || intent.boot_id != current_boot_id()?
            || receipt.boot_id != intent.boot_id
            || intent.phase != phase
            || receipt.phase != phase
            || intent.lane_orders != expected_orders
            || receipt.lane_orders != expected_orders
            || receipt.phase_intent_sha256 != intent_sha256
            || receipt.runner_report_sha256 != sha256_file(&report_path)?
        {
            return Err(EvalError::Invalid(format!(
                "{} auth-cache lifecycle phase receipt is invalid",
                phase.label()
            )));
        }
        let mut expected_bindings = Vec::with_capacity(expected_orders.len());
        for lane_order in &expected_orders {
            let (terminal, terminal_receipt_sha256): (CodexAuthCacheLaneTerminalReceipt, String) =
                self.directory.read_json_pair(&lane_lifecycle_stem(
                    phase,
                    *lane_order,
                    "terminal",
                ))?;
            self.validate_lane_terminal_receipt(plan, run_plan, phase, *lane_order, &terminal)?;
            expected_bindings.push(phase_lane_binding(&terminal, terminal_receipt_sha256));
        }
        if receipt.lane_bindings != expected_bindings {
            return Err(EvalError::Invalid(format!(
                "{} auth-cache lifecycle phase lane bindings drifted",
                phase.label()
            )));
        }
        Ok(())
    }

    fn prior_claude_spend_microusd(
        &self,
        plan: &PreparedNativePilot,
        run_plan: &Path,
        current_ordinal: u32,
    ) -> EvalResult<u64> {
        if !(1..=30).contains(&current_ordinal) {
            return Err(EvalError::Invalid(
                "family-v3 treatment admission ordinal is outside 1..=30".to_string(),
            ));
        }
        let mut ordered = Vec::with_capacity(30);
        for lane in &plan.lanes {
            ordered.push((lane.order, NativePilotRunPhase::Teaching, lane));
        }
        let mut activation_ordinal = 13_u32;
        for lane in plan.lanes.iter().filter(|lane| lane.host == "codex") {
            ordered.push((activation_ordinal, NativePilotRunPhase::Activation, lane));
            activation_ordinal += 1;
        }
        for lane in &plan.lanes {
            ordered.push((18 + lane.order, NativePilotRunPhase::Evaluation, lane));
        }
        ordered.sort_by_key(|(ordinal, _, _)| *ordinal);
        if ordered.len() != 30
            || ordered
                .iter()
                .map(|(ordinal, _, _)| *ordinal)
                .collect::<Vec<_>>()
                != (1..=30).collect::<Vec<_>>()
        {
            return Err(EvalError::Invalid(
                "family-v3 treatment cost journal matrix is not exact".to_string(),
            ));
        }
        let mut total = 0_u64;
        for (ordinal, phase, lane) in ordered {
            if ordinal >= current_ordinal {
                break;
            }
            let (terminal, _): (CodexAuthCacheLaneTerminalReceipt, String) = self
                .directory
                .read_json_pair(&lane_lifecycle_stem(phase, lane.order, "terminal"))?;
            self.validate_lane_terminal_receipt(plan, run_plan, phase, lane.order, &terminal)?;
            if lane.host == "claude_code" {
                let cost = terminal.provider_reported_cost_microusd.ok_or_else(|| {
                    EvalError::Invalid(format!(
                        "Claude treatment admission {ordinal} omitted receipt-bound cost"
                    ))
                })?;
                total = total.checked_add(cost).ok_or_else(|| {
                    EvalError::Invalid("Claude treatment cost accounting overflow".to_string())
                })?;
            }
        }
        Ok(total)
    }

    fn cleanup(
        &mut self,
        plan: &PreparedNativePilot,
        run_plan: &Path,
        run_root: &Path,
    ) -> EvalResult<NativePilotAuthCacheCleanupReport> {
        if self.directory.entry_exists("cleanup-receipt.json")? {
            let (report, _): (NativePilotAuthCacheCleanupReport, String) =
                self.directory.read_json_pair("cleanup-receipt")?;
            let (intent, intent_sha256): (CodexAuthCacheCleanupIntent, String) =
                self.directory.read_json_pair("cleanup-intent")?;
            validate_existing_cleanup_report(
                &report,
                &intent,
                &intent_sha256,
                self.receipt()?,
                plan,
                run_plan,
                run_root,
                &self.manifest_sha256,
            )?;
            if report.state == NativePilotAuthCacheCleanupState::Complete {
                let targets = exact_six_codex_auth_targets(plan)?;
                let effective_uid = current_effective_uid()?;
                for target in targets {
                    let home = open_private_codex_home(&target.home, effective_uid)?;
                    if auth_identity_relative(&home)?.is_some() {
                        return Err(EvalError::Invalid(
                            "completed auth-cache cleanup receipt has a present destination"
                                .to_string(),
                        ));
                    }
                }
            }
            return Ok(report);
        }
        let cleanup_intent_json = self.directory.entry_exists("cleanup-intent.json")?;
        let cleanup_intent_digest = self.directory.entry_exists("cleanup-intent.sha256")?;
        if cleanup_intent_json || cleanup_intent_digest {
            if !(cleanup_intent_json && cleanup_intent_digest) {
                return Err(EvalError::Invalid(
                    "auth-cache cleanup has an incomplete global intent pair; cleanup is unresolved and non-replayable"
                        .to_string(),
                ));
            }
            return self.seal_interrupted_cleanup(plan, run_plan, run_root);
        }

        let boot_id = current_boot_id()?;
        let receipt = self.receipt()?.clone();
        let targets = exact_six_codex_auth_targets(plan)?;
        let effective_uid = current_effective_uid()?;

        let intent = CodexAuthCacheCleanupIntent {
            schema_version: CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION,
            pilot_id: plan.pilot_id.clone(),
            run_plan_sha256: sha256_file(run_plan)?,
            lifecycle_manifest_sha256: self.manifest_sha256.clone(),
            run_root: run_root.display().to_string(),
            boot_id: boot_id.clone(),
            destinations: receipt.destinations.clone(),
            authority: CODEX_AUTH_CACHE_CLEANUP_AUTHORITY.to_string(),
            created_unix_ms: unix_ms(SystemTime::now())?,
        };
        let cleanup_intent_sha256 = self
            .directory
            .write_json_pair_exclusive("cleanup-intent", &intent)?;

        let mut removed = Vec::new();
        let cleanup_result = (|| -> EvalResult<()> {
            require_same_boot(&receipt.boot_id, &boot_id)
                .map_err(|_| EvalError::Invalid("boot_identity_changed".to_string()))?;
            self.validate_phase_receipt(plan, run_plan, NativePilotRunPhase::Evaluation)
                .map_err(|_| {
                    EvalError::Invalid(
                        "evaluation_dispatch_or_terminal_state_ambiguous".to_string(),
                    )
                })?;
            let evaluation =
                read_validated_stale_phase_receipt(plan, run_plan, NativePilotRunPhase::Evaluation)
                    .map_err(|_| {
                        EvalError::Invalid("evaluation_receipt_invalid_or_absent".to_string())
                    })?;
            let audit = audit_native_memory_pilot(run_plan)
                .map_err(|_| EvalError::Invalid("evaluation_audit_unavailable".to_string()))?;
            if !evaluation.audit.complete || !audit.complete || audit.invalid {
                return Err(EvalError::Invalid(
                    "evaluation_not_terminal_and_complete".to_string(),
                ));
            }
            delete_exact_destination_receipts(
                &targets,
                &receipt.destinations,
                effective_uid,
                &self.directory,
                &cleanup_intent_sha256,
                &boot_id,
                &mut removed,
            )?;
            Ok(())
        })();

        if cleanup_result.is_err() {
            if let Ok(observed_absent) = observe_absent_destinations(&targets, effective_uid) {
                removed = observed_absent;
            }
        }

        let (state, unresolved_reason) = match cleanup_result {
            Ok(()) => (NativePilotAuthCacheCleanupState::Complete, None),
            Err(error) => (
                NativePilotAuthCacheCleanupState::Unresolved,
                Some(match error {
                    EvalError::Invalid(code)
                        if matches!(
                            code.as_str(),
                            "boot_identity_changed"
                                | "evaluation_dispatch_or_terminal_state_ambiguous"
                                | "evaluation_receipt_invalid_or_absent"
                                | "evaluation_audit_unavailable"
                                | "evaluation_not_terminal_and_complete"
                        ) =>
                    {
                        code
                    }
                    _ => "identity_or_unlink_verification_failed".to_string(),
                }),
            ),
        };
        let report = NativePilotAuthCacheCleanupReport {
            schema_version: CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION,
            pilot_id: plan.pilot_id.clone(),
            run_plan: run_plan.display().to_string(),
            run_root: run_root.display().to_string(),
            lifecycle_manifest_sha256: self.manifest_sha256.clone(),
            cleanup_intent_sha256,
            boot_id,
            state,
            destination_count: receipt.destinations.len(),
            removed_destinations: removed,
            unresolved_reason,
            completed_unix_ms: unix_ms(SystemTime::now())?,
        };
        self.directory
            .write_json_pair_exclusive("cleanup-receipt", &report)?;
        Ok(report)
    }

    fn seal_interrupted_cleanup(
        &self,
        plan: &PreparedNativePilot,
        run_plan: &Path,
        run_root: &Path,
    ) -> EvalResult<NativePilotAuthCacheCleanupReport> {
        let provision = self.receipt()?;
        let (intent, cleanup_intent_sha256): (CodexAuthCacheCleanupIntent, String) =
            self.directory.read_json_pair("cleanup-intent")?;
        if intent.schema_version != CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION
            || intent.pilot_id != plan.pilot_id
            || intent.run_plan_sha256 != sha256_file(run_plan)?
            || intent.lifecycle_manifest_sha256 != self.manifest_sha256
            || intent.run_root != run_root.display().to_string()
            || normalize_boot_id(&intent.boot_id).ok().as_deref() != Some(intent.boot_id.as_str())
            || intent.destinations != provision.destinations
            || intent.authority != CODEX_AUTH_CACHE_CLEANUP_AUTHORITY
        {
            return Err(EvalError::Invalid(
                "interrupted auth-cache cleanup has an invalid global intent".to_string(),
            ));
        }

        let targets = exact_six_codex_auth_targets(plan)?;
        let effective_uid = current_effective_uid()?;
        let current_boot = current_boot_id()?;
        let inspection = inspect_interrupted_cleanup_residue(
            &self.directory,
            &targets,
            &provision.destinations,
            effective_uid,
            &cleanup_intent_sha256,
            &intent.boot_id,
        );
        let mut reasons = vec!["interrupted_cleanup_inspection_only"];
        if current_boot != intent.boot_id {
            reasons.push("boot_identity_changed");
        }
        if !inspection.lane_residue_valid {
            reasons.push("lane_residue_invalid");
        }
        if !inspection.destination_inspection_complete {
            reasons.push("destination_inspection_incomplete");
        }
        let report = NativePilotAuthCacheCleanupReport {
            schema_version: CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION,
            pilot_id: plan.pilot_id.clone(),
            run_plan: run_plan.display().to_string(),
            run_root: run_root.display().to_string(),
            lifecycle_manifest_sha256: self.manifest_sha256.clone(),
            cleanup_intent_sha256,
            boot_id: intent.boot_id,
            state: NativePilotAuthCacheCleanupState::Unresolved,
            destination_count: provision.destinations.len(),
            removed_destinations: inspection.observed_absent_destinations,
            unresolved_reason: Some(format!(
                "{};lane_intents={};lane_receipts={}",
                reasons.join("+"),
                inspection.lane_intent_count,
                inspection.lane_receipt_count
            )),
            completed_unix_ms: unix_ms(SystemTime::now())?,
        };
        self.directory
            .write_json_pair_exclusive("cleanup-receipt", &report)?;
        Ok(report)
    }
}

/// Revalidate one treatment pre-provider receipt against the fully typed auth lifecycle without
/// acquiring its flock. Callers must already hold the parent bundle lock, or run after the
/// treatment lifecycle has released its lock; the receipt/digest checks remain fail-closed.
pub(crate) fn validate_native_family_v3_pre_provider_receipt_evidence(
    run_plan: &Path,
    evidence: &NativeProviderBundlePreProviderReceiptEvidence,
) -> EvalResult<()> {
    let plan = load_historical_native_plan(run_plan)?;
    validate_plan_digest(run_plan)?;
    if !validate_native_stale_plan_binding(&plan, run_plan)?
        || plan
            .stale_safety
            .as_ref()
            .map(|binding| binding.family_version)
            != Some(CODEX_AUTH_CACHE_LIFECYCLE_FAMILY_VERSION)
    {
        return Err(EvalError::Invalid(
            "pre-provider receipt evidence requires the exact family-v3 treatment plan".to_string(),
        ));
    }
    let run_plan = run_plan.canonicalize()?;
    let (phase, lane_order) = family_v3_treatment_phase_lane(&plan, evidence.ordinal)?;
    let lane = plan
        .lanes
        .iter()
        .find(|lane| lane.order == lane_order)
        .ok_or_else(|| {
            EvalError::Invalid("family-v3 pre-provider receipt evidence lane is absent".to_string())
        })?;
    let lifecycle = CodexAuthCacheLifecycle::open_without_lock_for_validation(&plan, &run_plan)?;
    let observed = lifecycle.validated_bundle_pre_provider_receipt_evidence(
        &plan,
        &run_plan,
        lane,
        phase,
        evidence.ordinal,
        &evidence.receipt_sha256,
    )?;
    if &observed != evidence {
        return Err(EvalError::Invalid(
            "family-v3 pre-provider receipt evidence is incomplete or drifted".to_string(),
        ));
    }
    Ok(())
}

fn family_v3_treatment_phase_lane(
    plan: &PreparedNativePilot,
    admission_ordinal: u32,
) -> EvalResult<(NativePilotRunPhase, u32)> {
    let mapped = match admission_ordinal {
        1..=12 => (NativePilotRunPhase::Teaching, admission_ordinal),
        13..=18 => {
            let codex_index = usize::try_from(admission_ordinal - 13)
                .map_err(|_| EvalError::Invalid("activation ordinal overflow".to_string()))?;
            let lane_order = plan
                .lanes
                .iter()
                .filter(|lane| lane.host == "codex")
                .nth(codex_index)
                .map(|lane| lane.order)
                .ok_or_else(|| {
                    EvalError::Invalid(
                        "family-v3 activation admission has no matching Codex lane".to_string(),
                    )
                })?;
            (NativePilotRunPhase::Activation, lane_order)
        }
        19..=30 => (NativePilotRunPhase::Evaluation, admission_ordinal - 18),
        _ => {
            return Err(EvalError::Invalid(
                "family-v3 treatment admission ordinal is outside 1..=30".to_string(),
            ))
        }
    };
    let lane = plan
        .lanes
        .iter()
        .find(|lane| lane.order == mapped.1)
        .ok_or_else(|| {
            EvalError::Invalid("family-v3 treatment admission maps to an absent lane".to_string())
        })?;
    if mapped.0 == NativePilotRunPhase::Activation && lane.host != "codex" {
        return Err(EvalError::Invalid(
            "family-v3 activation admission maps to a non-Codex lane".to_string(),
        ));
    }
    if treatment_admission_ordinal(plan, mapped.0, mapped.1)? != admission_ordinal {
        return Err(EvalError::Invalid(
            "family-v3 treatment admission mapping is not reversible".to_string(),
        ));
    }
    Ok(mapped)
}

fn phase_lane_binding(
    terminal: &CodexAuthCacheLaneTerminalReceipt,
    terminal_receipt_sha256: String,
) -> CodexAuthCachePhaseLaneBinding {
    CodexAuthCachePhaseLaneBinding {
        lane_order: terminal.lane_order,
        terminal_receipt_sha256,
        provider_started_unix_ms: terminal.provider_started_unix_ms,
        provider_completed_unix_ms: terminal.provider_completed_unix_ms,
        codex_session_rollout_path: terminal.codex_session_rollout_path.clone(),
        codex_session_rollout_sha256: terminal.codex_session_rollout_sha256.clone(),
        codex_model_provider: terminal.codex_model_provider.clone(),
        codex_host_resolved_model: terminal.codex_host_resolved_model.clone(),
        codex_agents_md_sha256: terminal.codex_agents_md_sha256.clone(),
        provider_trace_sha256: terminal.provider_trace_sha256.clone(),
        effective_environment_sha256: terminal.effective_environment_sha256.clone(),
        claude_config_artifacts_sha256: terminal.claude_config_artifacts_sha256.clone(),
        stdout_trace_bytes: terminal.stdout_trace_bytes,
        stderr_bytes: terminal.stderr_bytes,
        process_cleanup_proven: terminal.process_cleanup_proven,
        provider_reported_cost_microusd: terminal.provider_reported_cost_microusd,
        accepted_turn_boundary_budget_exit: terminal.accepted_turn_boundary_budget_exit,
        claude_requested_model: terminal.claude_requested_model.clone(),
        claude_host_resolved_model: terminal.claude_host_resolved_model.clone(),
    }
}

pub(crate) fn audit_native_family_v3_lifecycle(
    run_plan: &Path,
) -> EvalResult<NativeFamilyV3LifecycleAudit> {
    let plan = load_historical_native_plan(run_plan)?;
    if plan
        .stale_safety
        .as_ref()
        .map(|binding| binding.family_version)
        != Some(CODEX_AUTH_CACHE_LIFECYCLE_FAMILY_VERSION)
        || !requires_exact_six_codex_auth_lifecycle(&plan)?
    {
        return Err(EvalError::Invalid(
            "sanitized lifecycle audit requires exact stale-safety family version 3".to_string(),
        ));
    }
    validate_plan_digest(run_plan)?;
    let run_plan = run_plan.canonicalize()?;
    let lifecycle = CodexAuthCacheLifecycle::open(&plan, &run_plan)?;
    let lifecycle_root = run_plan
        .parent()
        .ok_or_else(|| EvalError::Invalid("run plan has no parent".to_string()))?
        .join(CODEX_AUTH_CACHE_LIFECYCLE_DIRECTORY);
    let mut bindings = Vec::with_capacity(30);
    for phase in [
        NativePilotRunPhase::Teaching,
        NativePilotRunPhase::Activation,
        NativePilotRunPhase::Evaluation,
    ] {
        lifecycle.validate_phase_receipt(&plan, &run_plan, phase)?;
        let phase_receipt_stem = phase_lifecycle_stem(phase, "receipt");
        let phase_receipt_path = lifecycle_root.join(format!("{phase_receipt_stem}.json"));
        let (receipt, phase_receipt_sha256): (CodexAuthCachePhaseReceipt, String) =
            lifecycle.directory.read_json_pair(&phase_receipt_stem)?;
        for binding in receipt.lane_bindings {
            let admission_ordinal = match phase {
                NativePilotRunPhase::Teaching => binding.lane_order,
                NativePilotRunPhase::Activation => {
                    let index = plan
                        .lanes
                        .iter()
                        .filter(|lane| lane.host == "codex")
                        .position(|lane| lane.order == binding.lane_order)
                        .ok_or_else(|| {
                            EvalError::Invalid(
                                "lifecycle activation receipt contains a non-Codex lane"
                                    .to_string(),
                            )
                        })?;
                    13 + u32::try_from(index).map_err(|_| {
                        EvalError::Invalid("lifecycle activation ordinal overflow".to_string())
                    })?
                }
                NativePilotRunPhase::Evaluation => 18 + binding.lane_order,
            };
            let terminal_receipt_path = lifecycle_root.join(format!(
                "{}.json",
                lane_lifecycle_stem(phase, binding.lane_order, "terminal")
            ));
            bindings.push(NativeFamilyV3LifecycleBinding {
                admission_ordinal,
                phase,
                lane_order: binding.lane_order,
                phase_receipt_path: phase_receipt_path.display().to_string(),
                phase_receipt_sha256: phase_receipt_sha256.clone(),
                terminal_receipt_path: terminal_receipt_path.display().to_string(),
                terminal_receipt_sha256: binding.terminal_receipt_sha256,
                provider_started_unix_ms: binding.provider_started_unix_ms.ok_or_else(|| {
                    EvalError::Invalid(
                        "family-v3 lifecycle binding omitted provider start".to_string(),
                    )
                })?,
                provider_completed_unix_ms: binding.provider_completed_unix_ms.ok_or_else(
                    || {
                        EvalError::Invalid(
                            "family-v3 lifecycle binding omitted provider completion".to_string(),
                        )
                    },
                )?,
                provider_trace_sha256: binding.provider_trace_sha256,
                effective_environment_sha256: binding.effective_environment_sha256,
                claude_config_artifacts_sha256: binding.claude_config_artifacts_sha256,
                stdout_trace_bytes: binding.stdout_trace_bytes,
                stderr_bytes: binding.stderr_bytes,
                process_cleanup_proven: binding.process_cleanup_proven,
                provider_reported_cost_microusd: binding.provider_reported_cost_microusd,
                accepted_turn_boundary_budget_exit: binding.accepted_turn_boundary_budget_exit,
                codex_session_rollout_path: binding.codex_session_rollout_path,
                codex_session_rollout_sha256: binding.codex_session_rollout_sha256,
                codex_model_provider: binding.codex_model_provider,
                codex_host_resolved_model: binding.codex_host_resolved_model,
                codex_agents_md_sha256: binding.codex_agents_md_sha256,
                claude_requested_model: binding.claude_requested_model,
                claude_host_resolved_model: binding.claude_host_resolved_model,
            });
        }
    }
    bindings.sort_by_key(|binding| binding.admission_ordinal);
    let ordinals = bindings
        .iter()
        .map(|binding| binding.admission_ordinal)
        .collect::<Vec<_>>();
    if ordinals != (1..=30).collect::<Vec<_>>() {
        return Err(EvalError::Invalid(
            "family-v3 lifecycle does not contain exactly 30 sealed treatment admissions"
                .to_string(),
        ));
    }
    let claude_reported_cost_microusd = bindings.iter().try_fold(0_u64, |total, binding| {
        match binding.provider_reported_cost_microusd {
            Some(cost) => total.checked_add(cost).ok_or_else(|| {
                EvalError::Invalid("family-v3 Claude cost accounting overflow".to_string())
            }),
            None => Ok(total),
        }
    })?;
    let claude_authorized_ceiling_microusd = u64::from(plan.claude_budget_cents) * 10_000;
    Ok(NativeFamilyV3LifecycleAudit {
        exact_admission_count: 30,
        all_phase_receipts_valid: true,
        claude_reported_cost_microusd,
        claude_authorized_ceiling_microusd,
        claude_turn_boundary_overshoot_microusd: claude_reported_cost_microusd
            .saturating_sub(claude_authorized_ceiling_microusd),
        bindings,
    })
}

#[derive(Debug)]
struct InterruptedCleanupInspection {
    observed_absent_destinations: Vec<String>,
    lane_intent_count: usize,
    lane_receipt_count: usize,
    lane_residue_valid: bool,
    destination_inspection_complete: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LifecyclePairState {
    Absent,
    Complete,
    Partial,
}

fn lifecycle_pair_state(
    lifecycle: &PrivateAuthLifecycleDirectory,
    stem: &str,
) -> EvalResult<LifecyclePairState> {
    let json = lifecycle.entry_exists(&format!("{stem}.json"))?;
    let digest = lifecycle.entry_exists(&format!("{stem}.sha256"))?;
    Ok(match (json, digest) {
        (false, false) => LifecyclePairState::Absent,
        (true, true) => LifecyclePairState::Complete,
        _ => LifecyclePairState::Partial,
    })
}

fn inspect_interrupted_cleanup_residue(
    lifecycle: &PrivateAuthLifecycleDirectory,
    targets: &[CodexAuthCacheTarget],
    destinations: &[CodexAuthCacheDestinationReceipt],
    expected_owner: u32,
    cleanup_intent_sha256: &str,
    boot_id: &str,
) -> InterruptedCleanupInspection {
    let mut lane_residue_valid = targets.len() == 6 && destinations.len() == 6;
    let mut lane_intent_orders = Vec::new();
    let mut lane_receipt_orders = Vec::new();
    for (target, destination) in targets.iter().zip(destinations) {
        let intent_stem = cleanup_lane_lifecycle_stem(target.lane_order, "intent");
        let receipt_stem = cleanup_lane_lifecycle_stem(target.lane_order, "receipt");
        let intent_state = lifecycle_pair_state(lifecycle, &intent_stem);
        let receipt_state = lifecycle_pair_state(lifecycle, &receipt_stem);
        if matches!(&intent_state, Ok(LifecyclePairState::Partial) | Err(_))
            || matches!(&receipt_state, Ok(LifecyclePairState::Partial) | Err(_))
        {
            lane_residue_valid = false;
        }

        let mut lane_intent_sha256 = None;
        if matches!(&intent_state, Ok(LifecyclePairState::Complete)) {
            lane_intent_orders.push(target.lane_order);
            match lifecycle.read_json_pair::<CodexAuthCacheCleanupLaneIntent>(&intent_stem) {
                Ok((intent, digest)) => {
                    if intent.schema_version != CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION
                        || intent.cleanup_intent_sha256 != cleanup_intent_sha256
                        || intent.boot_id != boot_id
                        || intent.lane_order != target.lane_order
                        || intent.auth_json != destination.auth_json
                        || intent.identity != destination.identity
                        || intent.authority != CODEX_AUTH_CACHE_CLEANUP_AUTHORITY
                    {
                        lane_residue_valid = false;
                    }
                    lane_intent_sha256 = Some(digest);
                }
                Err(_) => lane_residue_valid = false,
            }
        }

        if matches!(&receipt_state, Ok(LifecyclePairState::Complete)) {
            lane_receipt_orders.push(target.lane_order);
            match lifecycle.read_json_pair::<CodexAuthCacheCleanupLaneReceipt>(&receipt_stem) {
                Ok((receipt, _)) => {
                    if receipt.schema_version != CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION
                        || receipt.cleanup_intent_sha256 != cleanup_intent_sha256
                        || Some(receipt.cleanup_lane_intent_sha256.as_str())
                            != lane_intent_sha256.as_deref()
                        || receipt.boot_id != boot_id
                        || receipt.lane_order != target.lane_order
                        || receipt.auth_json != destination.auth_json
                        || receipt.identity != destination.identity
                        || !receipt.absent_after_unlink_and_parent_fsync
                    {
                        lane_residue_valid = false;
                    }
                }
                Err(_) => lane_residue_valid = false,
            }
        } else if !matches!(&receipt_state, Ok(LifecyclePairState::Absent)) {
            lane_residue_valid = false;
        }
    }

    let (observed_absent_destinations, destination_inspection_complete) =
        inspect_absent_destinations(targets, expected_owner);
    let expected_orders = targets
        .iter()
        .map(|target| target.lane_order)
        .collect::<Vec<_>>();
    let observed_absent_orders = targets
        .iter()
        .filter(|target| {
            observed_absent_destinations
                .iter()
                .any(|path| path == &target.auth_json.display().to_string())
        })
        .map(|target| target.lane_order)
        .collect::<Vec<_>>();
    if lane_intent_orders != expected_orders[..lane_intent_orders.len()]
        || lane_receipt_orders != expected_orders[..lane_receipt_orders.len()]
        || lane_receipt_orders.len() > lane_intent_orders.len()
        || lane_intent_orders.len() > lane_receipt_orders.len().saturating_add(1)
        || observed_absent_orders != expected_orders[..observed_absent_orders.len()]
        || observed_absent_orders.len() < lane_receipt_orders.len()
        || observed_absent_orders.len() > lane_intent_orders.len()
    {
        lane_residue_valid = false;
    }

    InterruptedCleanupInspection {
        observed_absent_destinations,
        lane_intent_count: lane_intent_orders.len(),
        lane_receipt_count: lane_receipt_orders.len(),
        lane_residue_valid,
        destination_inspection_complete,
    }
}

fn validate_existing_cleanup_report(
    report: &NativePilotAuthCacheCleanupReport,
    intent: &CodexAuthCacheCleanupIntent,
    intent_sha256: &str,
    provision: &CodexAuthCacheProvisionReceipt,
    plan: &PreparedNativePilot,
    run_plan: &Path,
    run_root: &Path,
    manifest_sha256: &str,
) -> EvalResult<()> {
    if report.schema_version != CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION
        || intent.schema_version != CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION
        || report.pilot_id != plan.pilot_id
        || intent.pilot_id != plan.pilot_id
        || report.run_plan != run_plan.display().to_string()
        || report.run_root != run_root.display().to_string()
        || intent.run_root != report.run_root
        || report.lifecycle_manifest_sha256 != manifest_sha256
        || intent.lifecycle_manifest_sha256 != manifest_sha256
        || report.cleanup_intent_sha256 != intent_sha256
        || !is_sha256(intent_sha256)
        || intent.run_plan_sha256 != sha256_file(run_plan)?
        || intent.boot_id != report.boot_id
        || normalize_boot_id(&report.boot_id).ok().as_deref() != Some(report.boot_id.as_str())
        || intent.destinations != provision.destinations
        || intent.authority != CODEX_AUTH_CACHE_CLEANUP_AUTHORITY
        || report.destination_count != 6
        || (report.state == NativePilotAuthCacheCleanupState::Complete
            && (report.removed_destinations
                != provision
                    .destinations
                    .iter()
                    .map(|destination| destination.auth_json.clone())
                    .collect::<Vec<_>>()
                || report.unresolved_reason.is_some()))
        || (report.state == NativePilotAuthCacheCleanupState::Unresolved
            && (report.unresolved_reason.is_none()
                || report.removed_destinations.iter().any(|removed| {
                    !provision
                        .destinations
                        .iter()
                        .any(|destination| &destination.auth_json == removed)
                })))
    {
        return Err(EvalError::Invalid(
            "existing auth-cache cleanup receipt is invalid".to_string(),
        ));
    }
    Ok(())
}

#[derive(Debug)]
struct PrivateCodexHome {
    directory: File,
    owner: u32,
}

fn open_private_codex_home(path: &Path, expected_owner: u32) -> EvalResult<PrivateCodexHome> {
    let (directory, owner) = open_private_directory(path)?;
    if owner != expected_owner {
        return Err(EvalError::Invalid(
            "Codex home must be owned by the effective user".to_string(),
        ));
    }
    Ok(PrivateCodexHome { directory, owner })
}

fn require_auth_entry_absent(home: &PrivateCodexHome) -> EvalResult<()> {
    if relative_entry_exists(&home.directory, "auth.json")? {
        return Err(EvalError::Invalid(
            "refusing to overwrite an existing Codex auth cache".to_string(),
        ));
    }
    Ok(())
}

fn create_auth_cache_relative(
    home: &PrivateCodexHome,
    contents: &[u8],
) -> EvalResult<CodexAuthCacheFileIdentity> {
    let mut file = create_relative_file(&home.directory, "auth.json", 0o600)?;
    file.write_all(contents)?;
    file.sync_all()?;
    home.directory.sync_all()?;
    let identity = codex_auth_identity(&file.metadata()?)?;
    if identity.owner != home.owner || identity.mode != 0o600 || identity.link_count != 1 {
        return Err(EvalError::Invalid(
            "created Codex auth cache failed its private identity contract".to_string(),
        ));
    }
    Ok(identity)
}

fn auth_identity_relative(
    home: &PrivateCodexHome,
) -> EvalResult<Option<CodexAuthCacheFileIdentity>> {
    match open_relative_regular_file(&home.directory, "auth.json", home.owner, false) {
        Ok(file) => Ok(Some(codex_auth_identity(&file.metadata()?)?)),
        Err(EvalError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn validate_destination_receipts(
    targets: &[CodexAuthCacheTarget],
    receipts: &[CodexAuthCacheDestinationReceipt],
    expected_owner: u32,
) -> EvalResult<()> {
    if targets.len() != 6 || receipts.len() != 6 {
        return Err(EvalError::Invalid(
            "auth-cache lifecycle requires exactly six destinations".to_string(),
        ));
    }
    for (target, receipt) in targets.iter().zip(receipts) {
        if target.lane_order != receipt.lane_order
            || target.auth_json.display().to_string() != receipt.auth_json
        {
            return Err(EvalError::Invalid(
                "auth-cache destination receipt does not match its frozen lane".to_string(),
            ));
        }
        let home = open_private_codex_home(&target.home, expected_owner)?;
        let identity = auth_identity_relative(&home)?.ok_or_else(|| {
            EvalError::Invalid("manifest-bound Codex auth cache is absent".to_string())
        })?;
        if identity != receipt.identity {
            return Err(EvalError::Invalid(
                "manifest-bound Codex auth cache identity drifted".to_string(),
            ));
        }
    }
    Ok(())
}

fn delete_exact_destination_receipts(
    targets: &[CodexAuthCacheTarget],
    receipts: &[CodexAuthCacheDestinationReceipt],
    expected_owner: u32,
    lifecycle: &PrivateAuthLifecycleDirectory,
    cleanup_intent_sha256: &str,
    boot_id: &str,
    removed: &mut Vec<String>,
) -> EvalResult<()> {
    // Complete the identity preflight before the first unlink, so known drift never causes a
    // partial path-based cleanup.
    validate_destination_receipts(targets, receipts, expected_owner)?;
    for (target, receipt) in targets.iter().zip(receipts) {
        let intent = CodexAuthCacheCleanupLaneIntent {
            schema_version: CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION,
            cleanup_intent_sha256: cleanup_intent_sha256.to_string(),
            boot_id: boot_id.to_string(),
            lane_order: receipt.lane_order,
            auth_json: receipt.auth_json.clone(),
            identity: receipt.identity.clone(),
            authority: CODEX_AUTH_CACHE_CLEANUP_AUTHORITY.to_string(),
            created_unix_ms: unix_ms(SystemTime::now())?,
        };
        let lane_intent_sha256 = lifecycle.write_json_pair_exclusive(
            &cleanup_lane_lifecycle_stem(receipt.lane_order, "intent"),
            &intent,
        )?;
        let home = open_private_codex_home(&target.home, expected_owner)?;
        let current = auth_identity_relative(&home)?.ok_or_else(|| {
            EvalError::Invalid("manifest-bound Codex auth cache became absent".to_string())
        })?;
        if current != receipt.identity {
            return Err(EvalError::Invalid(
                "manifest-bound Codex auth cache identity changed immediately before unlink"
                    .to_string(),
            ));
        }
        unlink_auth_cache_relative(&home)?;
        removed.push(receipt.auth_json.clone());
        let terminal = CodexAuthCacheCleanupLaneReceipt {
            schema_version: CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION,
            cleanup_intent_sha256: cleanup_intent_sha256.to_string(),
            cleanup_lane_intent_sha256: lane_intent_sha256,
            boot_id: boot_id.to_string(),
            lane_order: receipt.lane_order,
            auth_json: receipt.auth_json.clone(),
            identity: receipt.identity.clone(),
            absent_after_unlink_and_parent_fsync: true,
            completed_unix_ms: unix_ms(SystemTime::now())?,
        };
        lifecycle.write_json_pair_exclusive(
            &cleanup_lane_lifecycle_stem(receipt.lane_order, "receipt"),
            &terminal,
        )?;
    }
    Ok(())
}

fn observe_absent_destinations(
    targets: &[CodexAuthCacheTarget],
    expected_owner: u32,
) -> EvalResult<Vec<String>> {
    let (absent, complete) = inspect_absent_destinations(targets, expected_owner);
    if !complete {
        return Err(EvalError::Invalid(
            "could not inspect every auth-cache cleanup destination".to_string(),
        ));
    }
    Ok(absent)
}

fn inspect_absent_destinations(
    targets: &[CodexAuthCacheTarget],
    expected_owner: u32,
) -> (Vec<String>, bool) {
    let mut absent = Vec::new();
    let mut complete = true;
    for target in targets {
        match open_private_codex_home(&target.home, expected_owner)
            .and_then(|home| relative_entry_exists(&home.directory, "auth.json"))
        {
            Ok(false) => absent.push(target.auth_json.display().to_string()),
            Ok(true) => {}
            Err(_) => complete = false,
        }
    }
    (absent, complete)
}

fn unlink_auth_cache_relative(home: &PrivateCodexHome) -> EvalResult<()> {
    #[cfg(not(unix))]
    {
        let _ = home;
        return Err(EvalError::Invalid(
            "Codex auth-cache cleanup requires Unix unlinkat".to_string(),
        ));
    }

    #[cfg(unix)]
    {
        use std::ffi::CString;
        use std::os::fd::AsRawFd;

        let name = CString::new("auth.json").expect("static entry name");
        let result = unsafe { libc::unlinkat(home.directory.as_raw_fd(), name.as_ptr(), 0) };
        if result != 0 {
            return Err(EvalError::Io(std::io::Error::last_os_error()));
        }
        home.directory.sync_all()?;
        if relative_entry_exists(&home.directory, "auth.json")? {
            return Err(EvalError::Invalid(
                "Codex auth cache remained present after unlinkat".to_string(),
            ));
        }
        Ok(())
    }
}

fn open_source_codex_auth_cache(path: &Path) -> EvalResult<File> {
    #[cfg(not(unix))]
    {
        let _ = path;
        return Err(EvalError::Invalid(
            "Codex auth-cache provisioning requires Unix permission checks".to_string(),
        ));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;

        let mut options = OpenOptions::new();
        options
            .read(true)
            .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW);
        let file = options.open(path).map_err(|error| {
            EvalError::Invalid(format!(
                "source auth cache could not be opened safely: {error}"
            ))
        })?;
        codex_auth_identity(&file.metadata()?)?;
        Ok(file)
    }
}

fn read_source_codex_auth_cache(mut file: File) -> EvalResult<Zeroizing<Vec<u8>>> {
    let mut contents = Zeroizing::new(Vec::new());
    (&mut file)
        .take(MAX_CODEX_AUTH_FILE_BYTES + 1)
        .read_to_end(&mut contents)?;
    if contents.is_empty() || contents.len() as u64 > MAX_CODEX_AUTH_FILE_BYTES {
        return Err(EvalError::Invalid(
            "source auth cache has an invalid bounded size".to_string(),
        ));
    }
    let value: Value = serde_json::from_slice(&contents)
        .map_err(|_| EvalError::Invalid("source auth cache is not a JSON object".to_string()))?;
    if !value.is_object() {
        return Err(EvalError::Invalid(
            "source auth cache is not a JSON object".to_string(),
        ));
    }
    Ok(contents)
}

fn codex_auth_identity(metadata: &fs::Metadata) -> EvalResult<CodexAuthCacheFileIdentity> {
    #[cfg(not(unix))]
    {
        let _ = metadata;
        return Err(EvalError::Invalid(
            "Codex auth-cache identity requires Unix metadata".to_string(),
        ));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;

        if !metadata.is_file()
            || metadata.mode() & 0o777 != 0o600
            || metadata.nlink() != 1
            || metadata.len() == 0
            || metadata.len() > MAX_CODEX_AUTH_FILE_BYTES
        {
            return Err(EvalError::Invalid(
                "Codex auth cache failed its regular owner-only single-link size contract"
                    .to_string(),
            ));
        }
        Ok(CodexAuthCacheFileIdentity {
            device: metadata.dev(),
            inode: metadata.ino(),
            owner: metadata.uid(),
            mode: metadata.mode() & 0o777,
            link_count: metadata.nlink(),
            size: metadata.len(),
        })
    }
}

fn open_private_directory(path: &Path) -> EvalResult<(File, u32)> {
    #[cfg(not(unix))]
    {
        let _ = path;
        return Err(EvalError::Invalid(
            "private directory validation requires Unix metadata".to_string(),
        ));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, OpenOptionsExt};

        let mut options = OpenOptions::new();
        options
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC);
        let directory = options.open(path)?;
        let metadata = directory.metadata()?;
        let effective_uid = unsafe { libc::geteuid() };
        if !metadata.is_dir() || metadata.uid() != effective_uid || metadata.mode() & 0o777 != 0o700
        {
            return Err(EvalError::Invalid(format!(
                "private directory must be real, owner-only, and owned by the effective user: {}",
                path.display()
            )));
        }
        Ok((directory, metadata.uid()))
    }
}

fn create_relative_file(directory: &File, name: &str, mode: u32) -> EvalResult<File> {
    #[cfg(not(unix))]
    {
        let _ = (directory, name, mode);
        return Err(EvalError::Invalid(
            "directory-relative file creation requires Unix openat".to_string(),
        ));
    }

    #[cfg(unix)]
    {
        use std::ffi::CString;
        use std::os::fd::{AsRawFd, FromRawFd};

        validate_lifecycle_entry_name(name)?;
        let name = CString::new(name)
            .map_err(|_| EvalError::Invalid("entry name contains NUL".to_string()))?;
        let descriptor = unsafe {
            libc::openat(
                directory.as_raw_fd(),
                name.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                libc::c_uint::from(mode as u16),
            )
        };
        if descriptor < 0 {
            return Err(EvalError::Io(std::io::Error::last_os_error()));
        }
        Ok(unsafe { File::from_raw_fd(descriptor) })
    }
}

fn open_relative_regular_file(
    directory: &File,
    name: &str,
    expected_owner: u32,
    allow_empty: bool,
) -> EvalResult<File> {
    #[cfg(not(unix))]
    {
        let _ = (directory, name, expected_owner, allow_empty);
        return Err(EvalError::Invalid(
            "directory-relative file opening requires Unix openat".to_string(),
        ));
    }

    #[cfg(unix)]
    {
        use std::ffi::CString;
        use std::os::fd::{AsRawFd, FromRawFd};
        use std::os::unix::fs::MetadataExt;

        validate_lifecycle_entry_name(name)?;
        let name = CString::new(name)
            .map_err(|_| EvalError::Invalid("entry name contains NUL".to_string()))?;
        let descriptor = unsafe {
            libc::openat(
                directory.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if descriptor < 0 {
            return Err(EvalError::Io(std::io::Error::last_os_error()));
        }
        let file = unsafe { File::from_raw_fd(descriptor) };
        let metadata = file.metadata()?;
        if !metadata.is_file()
            || metadata.uid() != expected_owner
            || metadata.mode() & 0o777 != 0o600
            || metadata.nlink() != 1
            || (!allow_empty && metadata.len() == 0)
        {
            return Err(EvalError::Invalid(
                "private lifecycle file failed its identity contract".to_string(),
            ));
        }
        Ok(file)
    }
}

fn relative_entry_exists(directory: &File, name: &str) -> EvalResult<bool> {
    #[cfg(not(unix))]
    {
        let _ = (directory, name);
        return Err(EvalError::Invalid(
            "directory-relative existence checks require Unix fstatat".to_string(),
        ));
    }

    #[cfg(unix)]
    {
        use std::ffi::CString;
        use std::mem::MaybeUninit;
        use std::os::fd::AsRawFd;

        validate_lifecycle_entry_name(name)?;
        let name = CString::new(name)
            .map_err(|_| EvalError::Invalid("entry name contains NUL".to_string()))?;
        let mut metadata = MaybeUninit::<libc::stat>::uninit();
        let result = unsafe {
            libc::fstatat(
                directory.as_raw_fd(),
                name.as_ptr(),
                metadata.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        };
        if result == 0 {
            return Ok(true);
        }
        let error = std::io::Error::last_os_error();
        if error.kind() == std::io::ErrorKind::NotFound {
            Ok(false)
        } else {
            Err(EvalError::Io(error))
        }
    }
}

fn acquire_exclusive_file_lock(file: &File) -> EvalResult<()> {
    #[cfg(not(unix))]
    {
        let _ = file;
        return Err(EvalError::Invalid(
            "auth-cache lifecycle locking requires Unix flock".to_string(),
        ));
    }

    #[cfg(unix)]
    {
        use std::os::fd::AsRawFd;

        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return Err(EvalError::Invalid(
                "another auth-cache lifecycle operation holds the exclusive lock".to_string(),
            ));
        }
        Ok(())
    }
}

fn sync_directory(path: &Path) -> EvalResult<()> {
    let (directory, _) = open_private_directory(path)?;
    directory.sync_all()?;
    Ok(())
}

fn current_effective_uid() -> EvalResult<u32> {
    #[cfg(unix)]
    {
        Ok(unsafe { libc::geteuid() })
    }
    #[cfg(not(unix))]
    {
        Err(EvalError::Invalid(
            "auth-cache lifecycle requires Unix ownership".to_string(),
        ))
    }
}

fn current_boot_id() -> EvalResult<String> {
    #[cfg(target_os = "linux")]
    {
        let value = fs::read_to_string("/proc/sys/kernel/random/boot_id")?;
        normalize_boot_id(&value)
    }

    #[cfg(target_os = "macos")]
    {
        use std::ffi::CString;

        let name = CString::new("kern.bootsessionuuid").expect("static sysctl name");
        let mut length: libc::size_t = 0;
        if unsafe {
            libc::sysctlbyname(
                name.as_ptr(),
                std::ptr::null_mut(),
                &mut length,
                std::ptr::null_mut(),
                0,
            )
        } != 0
            || length == 0
            || length > 128
        {
            return Err(EvalError::Invalid(
                "boot identity is unavailable".to_string(),
            ));
        }
        let mut bytes = vec![0_u8; length];
        if unsafe {
            libc::sysctlbyname(
                name.as_ptr(),
                bytes.as_mut_ptr().cast(),
                &mut length,
                std::ptr::null_mut(),
                0,
            )
        } != 0
        {
            return Err(EvalError::Invalid(
                "boot identity is unavailable".to_string(),
            ));
        }
        bytes.truncate(length);
        while bytes.last() == Some(&0) {
            bytes.pop();
        }
        let value = std::str::from_utf8(&bytes)
            .map_err(|_| EvalError::Invalid("boot identity is not UTF-8".to_string()))?;
        normalize_boot_id(value)
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    Err(EvalError::Invalid(
        "boot identity is unsupported on this operating system".to_string(),
    ))
}

fn normalize_boot_id(value: &str) -> EvalResult<String> {
    let value = value.trim().to_ascii_lowercase();
    let bytes = value.as_bytes();
    if bytes.len() != 36
        || [8, 13, 18, 23]
            .into_iter()
            .any(|index| bytes[index] != b'-')
        || bytes
            .iter()
            .enumerate()
            .any(|(index, byte)| ![8, 13, 18, 23].contains(&index) && !byte.is_ascii_hexdigit())
        || value
            .chars()
            .filter(|character| *character != '-')
            .all(|character| character == '0')
    {
        return Err(EvalError::Invalid(
            "boot identity is not one canonical nonzero UUID".to_string(),
        ));
    }
    Ok(value)
}

fn require_same_boot(expected: &str, observed: &str) -> EvalResult<()> {
    if normalize_boot_id(expected)? != normalize_boot_id(observed)? || expected != observed {
        return Err(EvalError::Invalid(
            "auth-cache lifecycle belongs to another boot".to_string(),
        ));
    }
    Ok(())
}

fn validate_lifecycle_entry_name(name: &str) -> EvalResult<()> {
    if name.is_empty()
        || name.len() > 128
        || name == "."
        || name == ".."
        || name.contains('/')
        || name.contains('\\')
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(EvalError::Invalid(
            "auth-cache lifecycle entry name is invalid".to_string(),
        ));
    }
    Ok(())
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn create_private_file(path: &Path) -> EvalResult<File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path).map_err(EvalError::Io)
}

fn write_private_codex_auth_cache(path: &Path, contents: &[u8]) -> EvalResult<()> {
    #[cfg(not(unix))]
    {
        let _ = (path, contents);
        return Err(EvalError::Invalid(
            "Codex file-cache authentication requires Unix permission checks".to_string(),
        ));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, OpenOptionsExt};

        let parent = path.parent().ok_or_else(|| {
            EvalError::Invalid("Codex auth cache destination has no parent".to_string())
        })?;
        let parent_metadata = fs::symlink_metadata(parent)?;
        let mut options = OpenOptions::new();
        options.write(true).create_new(true).mode(0o600);
        let mut file = options.open(path)?;
        let result = file
            .write_all(contents)
            .and_then(|()| file.sync_all())
            .map_err(EvalError::Io);
        drop(file);
        if let Err(error) = result {
            let _ = fs::remove_file(path);
            return Err(error);
        }
        if let Err(error) = validate_codex_auth_cache_file(path, Some(parent_metadata.uid())) {
            let _ = fs::remove_file(path);
            return Err(error);
        }
        Ok(())
    }
}

pub(crate) fn write_private_json(path: &Path, value: &impl Serialize) -> EvalResult<()> {
    let mut file = create_private_file(path)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    Ok(())
}

pub(crate) fn write_private_text(path: &Path, value: &str) -> EvalResult<()> {
    let mut file = create_private_file(path)?;
    file.write_all(value.as_bytes())?;
    file.sync_all()?;
    Ok(())
}

pub(crate) fn argv_sha256(argv: &[String]) -> EvalResult<String> {
    let mut digest = Sha256::new();
    digest.update(serde_json::to_vec(argv)?);
    Ok(format!("{:x}", digest.finalize()))
}

fn canonical_environment_sha256(environment: &BTreeMap<String, String>) -> EvalResult<String> {
    Ok(sha256_bytes(&serde_json::to_vec(environment)?))
}

fn sha256_file(path: &Path) -> EvalResult<String> {
    let mut digest = Sha256::new();
    digest.update(fs::read(path)?);
    Ok(format!("{:x}", digest.finalize()))
}

fn unix_ms(time: SystemTime) -> EvalResult<u64> {
    let duration = time
        .duration_since(UNIX_EPOCH)
        .map_err(|error| EvalError::Invalid(format!("timestamp predates Unix epoch: {error}")))?;
    u64::try_from(duration.as_millis())
        .map_err(|_| EvalError::Invalid("timestamp overflow".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_audit::{FileAudit, NativeLaneAudit};
    use crate::native_pilot::MemoryLayer;

    #[cfg(unix)]
    fn write_test_executable(path: &Path, contents: &str) {
        use std::os::unix::fs::PermissionsExt;

        fs::write(path, contents).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
    }

    fn plan(budget: u32) -> PreparedNativePilot {
        PreparedNativePilot {
            protocol_schema_version: 4,
            pilot_id: "pilot".to_string(),
            execution_approved: false,
            prepared_unix_ms: 1,
            codex_min_idle_hours: 1,
            resource_budgets: None,
            claude_budget_cents: budget,
            claude_prior_spend_microusd: 0,
            claude_authorized_ceiling_cents: budget,
            claude_model: "claude-haiku-4-5".to_string(),
            claude_max_turns: 12,
            native_memory_references: BTreeMap::new(),
            binaries: BTreeMap::new(),
            runtime_libraries: BTreeMap::new(),
            engram_mcp_contract: EngramMcpContractAttestation {
                schema_version: 3,
                cli_version: env!("CARGO_PKG_VERSION").to_string(),
                health_schema_version: 1,
                mcp_contract_version: 1,
                mcp_protocol_version: "test".to_string(),
                profile: "agent".to_string(),
                mcp_tool_count: 6,
                mcp_tools_sha256: "a".repeat(64),
                profile_instructions_sha256: Some("b".repeat(64)),
                effective_runtime: Some(EngramMcpRuntimeAttestation {
                    verified: true,
                    mcp_tool_count: 6,
                    mcp_tools_sha256: "a".repeat(64),
                    profile_instructions_sha256: "b".repeat(64),
                    restricted_tool_rejected: true,
                    review_authority_rejected: true,
                    correction_proposal_path_verified: None,
                    direct_correction_unavailable: None,
                    correction_apply_unavailable: None,
                    correction_verification_unavailable: None,
                    correction_inspection_unavailable: None,
                }),
                executable_path: "engram".to_string(),
                executable_sha256: "c".repeat(64),
            },
            evaluation_recovery: None,
            execution_recovery: None,
            stale_safety: None,
            lanes: Vec::new(),
        }
    }

    fn receipt_test_lane(root: &Path, order: u32, host: &str) -> PreparedNativeLane {
        let lane_root = root.join(format!("lane-{order}"));
        PreparedNativeLane {
            order,
            case_id: format!("case-{order}"),
            arm: format!("arm-{order}"),
            host: host.to_string(),
            memory_layer: MemoryLayer::Native,
            repetition: 1,
            fixture_revision: "fixture".to_string(),
            teaching_cwd: lane_root.join("teaching").display().to_string(),
            evaluation_cwd: lane_root.join("evaluation").display().to_string(),
            environment: BTreeMap::new(),
            required_secret_environment: Vec::new(),
            codex_authentication: None,
            claude_teaching_bash_commands: Vec::new(),
            teaching_argv: vec![host.to_string(), "teach".to_string()],
            activation_argv: (host == "codex")
                .then(|| vec![host.to_string(), "activate".to_string()]),
            post_teaching_verification_argv: None,
            activation_wait_hours: 1,
            evaluation_argv: vec![host.to_string(), "evaluate".to_string()],
            artifact_gates: Vec::new(),
            acceptance_contract: lane_root.join("acceptance.json").display().to_string(),
            adapter_sha256: None,
            engram_home: None,
            engram_project: None,
            cleanup_argv: None,
            teaching_trace_path: lane_root.join("teaching.jsonl").display().to_string(),
            activation_trace_path: (host == "codex")
                .then(|| lane_root.join("activation.jsonl").display().to_string()),
            evaluation_trace_path: lane_root.join("evaluation.jsonl").display().to_string(),
            post_teaching_verification_output_path: None,
            agent_output_path: lane_root.join("agent-output.json").display().to_string(),
        }
    }

    fn receipt_file_audit(path: &str) -> FileAudit {
        receipt_file_audit_with_status(path, AuditStatus::Passed)
    }

    fn receipt_file_audit_with_status(path: &str, status: AuditStatus) -> FileAudit {
        FileAudit {
            status,
            path: path.to_string(),
            sha256: (status == AuditStatus::Passed).then(|| "a".repeat(64)),
            modified_unix_ms: (status == AuditStatus::Passed).then_some(1_500),
            detail: "synthetic".to_string(),
        }
    }

    fn receipt_lane_audit(
        lane: &PreparedNativeLane,
        phase: NativePilotRunPhase,
    ) -> NativeLaneAudit {
        NativeLaneAudit {
            order: lane.order,
            arm: lane.arm.clone(),
            phase: match phase {
                NativePilotRunPhase::Teaching if lane.host == "codex" => {
                    NativeLanePhase::AwaitingActivation
                }
                NativePilotRunPhase::Teaching | NativePilotRunPhase::Activation => {
                    NativeLanePhase::ReadyForEvaluation
                }
                NativePilotRunPhase::Evaluation => NativeLanePhase::EvaluationComplete,
            },
            teaching_trace: receipt_file_audit(&lane.teaching_trace_path),
            activation_trace: lane.activation_trace_path.as_deref().map(|path| {
                receipt_file_audit_with_status(
                    path,
                    if phase == NativePilotRunPhase::Teaching {
                        AuditStatus::Pending
                    } else {
                        AuditStatus::Passed
                    },
                )
            }),
            procedure_verification: None,
            artifacts: Vec::new(),
            evaluation_trace: receipt_file_audit_with_status(
                &lane.evaluation_trace_path,
                if phase == NativePilotRunPhase::Evaluation {
                    AuditStatus::Passed
                } else {
                    AuditStatus::Pending
                },
            ),
            agent_output: receipt_file_audit_with_status(
                &lane.agent_output_path,
                if phase == NativePilotRunPhase::Evaluation {
                    AuditStatus::Passed
                } else {
                    AuditStatus::Pending
                },
            ),
            acceptance: None,
            native_memory_write_attempts: Vec::new(),
            failures: Vec::new(),
        }
    }

    fn receipt_for(
        plan: &PreparedNativePilot,
        run_plan: &Path,
        phase: NativePilotRunPhase,
    ) -> NativePilotRunReport {
        let phase_lanes = plan
            .lanes
            .iter()
            .filter(|lane| phase != NativePilotRunPhase::Activation || lane.host == "codex")
            .collect::<Vec<_>>();
        let lanes = phase_lanes
            .iter()
            .enumerate()
            .map(|(index, lane)| {
                let (trace, argv) = match phase {
                    NativePilotRunPhase::Teaching => {
                        (Path::new(&lane.teaching_trace_path), &lane.teaching_argv)
                    }
                    NativePilotRunPhase::Activation => (
                        Path::new(lane.activation_trace_path.as_deref().unwrap()),
                        lane.activation_argv.as_ref().unwrap(),
                    ),
                    NativePilotRunPhase::Evaluation => (
                        Path::new(&lane.evaluation_trace_path),
                        &lane.evaluation_argv,
                    ),
                };
                let started = 1_100 + u64::try_from(index).unwrap() * 50;
                NativePilotLaneExecution {
                    order: lane.order,
                    arm: lane.arm.clone(),
                    host: lane.host.clone(),
                    trace_path: trace.display().to_string(),
                    stderr_path: stderr_path(trace).display().to_string(),
                    argv_sha256: argv_sha256(argv).unwrap(),
                    effective_environment_sha256: None,
                    claude_config_artifacts_sha256: None,
                    stdout_trace_bytes: None,
                    stderr_bytes: None,
                    process_cleanup_proven: None,
                    provider_started_unix_ms: Some(started),
                    provider_completed_unix_ms: Some(started + 25),
                    exit_code: 0,
                    codex_session_rollout_path: None,
                    codex_session_rollout_sha256: None,
                    codex_model_provider: None,
                    codex_host_resolved_model: None,
                    codex_agents_md_sha256: None,
                    provider_trace_sha256: None,
                    claude_requested_model: None,
                    claude_host_resolved_model: None,
                    provider_reported_cost_microusd: None,
                    accepted_turn_boundary_budget_exit: false,
                    recovered_from_existing_trace: false,
                }
            })
            .collect();
        NativePilotRunReport {
            pilot_id: plan.pilot_id.clone(),
            phase,
            run_plan: run_plan.canonicalize().unwrap().display().to_string(),
            confirmed_claude_budget_cents: plan.claude_budget_cents,
            started_unix_ms: 1_000,
            completed_unix_ms: 2_000,
            lanes,
            stale_safety_pre_evaluation_marker_snapshot_sha256: None,
            audit: NativePilotAudit {
                pilot_id: plan.pilot_id.clone(),
                run_plan: run_plan.canonicalize().unwrap().display().to_string(),
                ready_for_evaluation: phase != NativePilotRunPhase::Teaching,
                complete: phase == NativePilotRunPhase::Evaluation,
                all_acceptance_passed: false,
                invalid: false,
                lanes: plan
                    .lanes
                    .iter()
                    .map(|lane| receipt_lane_audit(lane, phase))
                    .collect(),
            },
        }
    }

    #[test]
    fn stale_teaching_deadline_source_requires_digest_and_exact_lifecycle_receipt() {
        let root = tempfile::tempdir().unwrap();
        let run_plan = root.path().join("run-plan.json");
        fs::write(&run_plan, "{}\n").unwrap();
        let mut plan = plan(30);
        plan.lanes = vec![
            receipt_test_lane(root.path(), 1, "codex"),
            receipt_test_lane(root.path(), 2, "claude_code"),
        ];
        let receipt_path = root.path().join("runner-teaching.json");
        let receipt = receipt_for(&plan, &run_plan, NativePilotRunPhase::Teaching);
        write_private_json(&receipt_path, &receipt).unwrap();
        fs::write(
            root.path().join("runner-teaching.sha256"),
            format!("{}\n", sha256_file(&receipt_path).unwrap()),
        )
        .unwrap();
        assert_eq!(
            read_validated_stale_phase_receipt(&plan, &run_plan, NativePilotRunPhase::Teaching,)
                .unwrap()
                .completed_unix_ms,
            2_000
        );

        fs::write(&receipt_path, "{}\n").unwrap();
        assert!(read_validated_stale_phase_receipt(
            &plan,
            &run_plan,
            NativePilotRunPhase::Teaching,
        )
        .unwrap_err()
        .to_string()
        .contains("digest is invalid"));

        let reject_lifecycle = |candidate: &NativePilotRunReport| {
            fs::write(
                &receipt_path,
                serde_json::to_string_pretty(candidate).unwrap() + "\n",
            )
            .unwrap();
            fs::write(
                root.path().join("runner-teaching.sha256"),
                format!("{}\n", sha256_file(&receipt_path).unwrap()),
            )
            .unwrap();
            assert!(read_validated_stale_phase_receipt(
                &plan,
                &run_plan,
                NativePilotRunPhase::Teaching,
            )
            .unwrap_err()
            .to_string()
            .contains("lifecycle"));
        };

        let mut wrong_phase = receipt.clone();
        wrong_phase.phase = NativePilotRunPhase::Activation;
        reject_lifecycle(&wrong_phase);
        let mut wrong_pilot = receipt.clone();
        wrong_pilot.pilot_id = "different-pilot".to_string();
        reject_lifecycle(&wrong_pilot);
        let mut wrong_plan = receipt.clone();
        wrong_plan.run_plan = root.path().join("other-plan.json").display().to_string();
        reject_lifecycle(&wrong_plan);
        let mut reversed_phase_time = receipt.clone();
        reversed_phase_time.started_unix_ms = reversed_phase_time.completed_unix_ms + 1;
        reject_lifecycle(&reversed_phase_time);
        let mut wrong_budget = receipt.clone();
        wrong_budget.confirmed_claude_budget_cents += 1;
        reject_lifecycle(&wrong_budget);

        let mut reordered = receipt.clone();
        reordered.lanes.swap(0, 1);
        reject_lifecycle(&reordered);
        let mut wrong_arm = receipt.clone();
        wrong_arm.lanes[0].arm = "other-arm".to_string();
        reject_lifecycle(&wrong_arm);
        let mut wrong_host = receipt.clone();
        wrong_host.lanes[0].host = "claude_code".to_string();
        reject_lifecycle(&wrong_host);
        let mut wrong_trace = receipt.clone();
        wrong_trace.lanes[0].trace_path = "other-trace".to_string();
        reject_lifecycle(&wrong_trace);
        let mut wrong_argv = receipt.clone();
        wrong_argv.lanes[0].argv_sha256 = "0".repeat(64);
        reject_lifecycle(&wrong_argv);
        let mut missing_completion = receipt.clone();
        missing_completion.lanes[0].provider_completed_unix_ms = None;
        reject_lifecycle(&missing_completion);
        let mut before_phase = receipt.clone();
        before_phase.lanes[0].provider_started_unix_ms = Some(999);
        reject_lifecycle(&before_phase);
        let mut after_phase = receipt.clone();
        after_phase.lanes[1].provider_completed_unix_ms = Some(2_001);
        reject_lifecycle(&after_phase);
        let mut overlapping = receipt.clone();
        overlapping.lanes[1].provider_started_unix_ms = Some(1_110);
        reject_lifecycle(&overlapping);
        let mut recovered = receipt.clone();
        recovered.lanes[0].recovered_from_existing_trace = true;
        reject_lifecycle(&recovered);
        let mut wrong_audit = receipt.clone();
        wrong_audit.audit.lanes[0].phase = NativeLanePhase::ReadyForEvaluation;
        reject_lifecycle(&wrong_audit);
        let mut audit_with_hidden_failure = receipt.clone();
        audit_with_hidden_failure.audit.lanes[0]
            .failures
            .push("hidden failure".to_string());
        reject_lifecycle(&audit_with_hidden_failure);
        let mut audit_with_future_evaluation = receipt;
        audit_with_future_evaluation.audit.lanes[0]
            .evaluation_trace
            .status = AuditStatus::Passed;
        reject_lifecycle(&audit_with_future_evaluation);
    }

    #[test]
    fn stale_phase_preconditions_forbid_recovery_and_replay() {
        let root = tempfile::tempdir().unwrap();
        let run_plan = root.path().join("run-plan.json");
        fs::write(&run_plan, "{}\n").unwrap();
        let mut pilot = plan(30);
        pilot.pilot_id = "native-stale-safety-replay-fixture".to_string();
        pilot.lanes = vec![receipt_test_lane(root.path(), 1, "codex")];
        pilot.stale_safety = Some(crate::native_pilot::PreparedNativeStaleSafetyBinding {
            family_version: 1,
            protocol_snapshot_file: "stale-safety.protocol.snapshot.json".to_string(),
            protocol_snapshot_sha256: "a".repeat(64),
            retention_precondition_file: "stale-safety.retention-precondition.json".to_string(),
            retention_precondition_sha256: "b".repeat(64),
            agent_output_schema_sha256: "c".repeat(64),
        });

        for phase in [
            NativePilotRunPhase::Teaching,
            NativePilotRunPhase::Activation,
            NativePilotRunPhase::Evaluation,
        ] {
            let audit = receipt_for(&pilot, &run_plan, phase).audit;
            let error = validate_phase_preconditions(&pilot, &audit, phase)
                .unwrap_err()
                .to_string();
            assert!(error.contains("forbids lane recovery or replay"));
        }

        pilot.stale_safety = None;
        let historical = receipt_for(&pilot, &run_plan, NativePilotRunPhase::Teaching).audit;
        validate_phase_preconditions(&pilot, &historical, NativePilotRunPhase::Teaching).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn stale_lifecycle_residue_matrix_rejects_future_sidecars_and_dangling_entries() {
        use std::os::unix::fs::symlink;

        let root = tempfile::tempdir().unwrap();
        let run_plan = root.path().join("run-plan.json");
        fs::write(&run_plan, "{}\n").unwrap();
        let mut pilot = plan(30);
        pilot.pilot_id = "native-stale-safety-residue-fixture".to_string();
        pilot.lanes = vec![
            receipt_test_lane(root.path(), 1, "codex"),
            receipt_test_lane(root.path(), 2, "claude_code"),
        ];
        pilot.stale_safety = Some(crate::native_pilot::PreparedNativeStaleSafetyBinding {
            family_version: 1,
            protocol_snapshot_file: "stale-safety.protocol.snapshot.json".to_string(),
            protocol_snapshot_sha256: "a".repeat(64),
            retention_precondition_file: "stale-safety.retention-precondition.json".to_string(),
            retention_precondition_sha256: "b".repeat(64),
            agent_output_schema_sha256: "c".repeat(64),
        });

        let future_digest = root.path().join("runner-evaluation.sha256");
        fs::write(&future_digest, format!("{}\n", "0".repeat(64))).unwrap();
        assert!(validate_stale_lifecycle_residue_before_phase(
            &pilot,
            &run_plan,
            NativePilotRunPhase::Teaching,
        )
        .unwrap_err()
        .to_string()
        .contains("evaluation phase receipt digest"));
        fs::remove_file(&future_digest).unwrap();

        let dangling_receipt = root.path().join("runner-activation.json");
        symlink(root.path().join("missing-receipt"), &dangling_receipt).unwrap();
        assert!(validate_stale_lifecycle_residue_before_phase(
            &pilot,
            &run_plan,
            NativePilotRunPhase::Teaching,
        )
        .unwrap_err()
        .to_string()
        .contains("activation phase receipt"));
        fs::remove_file(&dangling_receipt).unwrap();

        let marker_sidecar = root
            .path()
            .join("stale-safety.pre-evaluation-native-markers.sha256");
        symlink(root.path().join("missing-marker"), &marker_sidecar).unwrap();
        assert!(validate_stale_lifecycle_residue_before_phase(
            &pilot,
            &run_plan,
            NativePilotRunPhase::Teaching,
        )
        .unwrap_err()
        .to_string()
        .contains("marker snapshot digest"));
        fs::remove_file(&marker_sidecar).unwrap();

        let teaching_path = root.path().join("runner-teaching.json");
        write_private_json(
            &teaching_path,
            &receipt_for(&pilot, &run_plan, NativePilotRunPhase::Teaching),
        )
        .unwrap();
        fs::write(
            root.path().join("runner-teaching.sha256"),
            format!("{}\n", sha256_file(&teaching_path).unwrap()),
        )
        .unwrap();
        validate_stale_lifecycle_residue_before_phase(
            &pilot,
            &run_plan,
            NativePilotRunPhase::Activation,
        )
        .unwrap();
        assert!(validate_stale_lifecycle_residue_before_phase(
            &pilot,
            &run_plan,
            NativePilotRunPhase::Evaluation,
        )
        .unwrap_err()
        .to_string()
        .contains("completed activation receipt"));
    }

    #[test]
    fn execution_requires_explicit_approval_and_exact_budget() {
        let plan = plan(50);
        let disabled = NativePilotExecutionApproval {
            provider_execution: false,
            claude_budget_cents: Some(50),
        };
        assert!(validate_execution_approval(&plan, disabled)
            .unwrap_err()
            .to_string()
            .contains("provider execution is disabled"));

        let wrong_budget = NativePilotExecutionApproval {
            provider_execution: true,
            claude_budget_cents: Some(49),
        };
        assert!(validate_execution_approval(&plan, wrong_budget)
            .unwrap_err()
            .to_string()
            .contains("exactly match"));

        validate_execution_approval(
            &plan,
            NativePilotExecutionApproval {
                provider_execution: true,
                claude_budget_cents: Some(50),
            },
        )
        .unwrap();
    }

    #[test]
    fn claude_only_diagnostic_has_no_codex_authentication_prerequisite() {
        let report = inspect_codex_authentication(&plan(50), Path::new("run-plan.json")).unwrap();

        assert!(report.ready);
        assert!(report.lanes.is_empty());
    }

    #[test]
    fn insufficient_engram_disk_headroom_blocks_before_provider_execution() {
        let error = require_engram_disk_headroom(
            3,
            Path::new("/private/tmp/pilot/engram-home"),
            engram_store::DiskHeadroom {
                available_bytes: 15_000,
                total_bytes: 1_000_000,
                required_bytes: 20_000,
            },
        )
        .unwrap_err();

        let message = error.to_string();
        assert!(message.contains("provider execution is blocked before lane 3"));
        assert!(message.contains("15000 bytes available"));
        assert!(message.contains("requires at least 20000 bytes"));
    }

    #[test]
    fn phase_target_validation_refuses_every_provider_output_class() {
        let root = tempfile::tempdir().unwrap();
        let lane = PreparedNativeLane {
            order: 1,
            case_id: "case".to_string(),
            arm: "codex_engram_plus_native".to_string(),
            host: "codex".to_string(),
            memory_layer: MemoryLayer::Both,
            repetition: 1,
            fixture_revision: "fixture".to_string(),
            teaching_cwd: root.path().display().to_string(),
            evaluation_cwd: root.path().display().to_string(),
            environment: BTreeMap::new(),
            required_secret_environment: Vec::new(),
            codex_authentication: None,
            claude_teaching_bash_commands: Vec::new(),
            teaching_argv: Vec::new(),
            activation_argv: Some(Vec::new()),
            post_teaching_verification_argv: None,
            activation_wait_hours: 1,
            evaluation_argv: Vec::new(),
            artifact_gates: Vec::new(),
            acceptance_contract: root
                .path()
                .join("acceptance-contract.json")
                .display()
                .to_string(),
            adapter_sha256: None,
            engram_home: Some(root.path().join("engram-home").display().to_string()),
            engram_project: Some("pilot".to_string()),
            cleanup_argv: Some(vec![
                "engram".to_string(),
                "daemon".to_string(),
                "stop".to_string(),
            ]),
            teaching_trace_path: root.path().join("teaching.jsonl").display().to_string(),
            activation_trace_path: Some(root.path().join("activation.jsonl").display().to_string()),
            evaluation_trace_path: root.path().join("evaluation.jsonl").display().to_string(),
            post_teaching_verification_output_path: None,
            agent_output_path: root.path().join("agent-output.json").display().to_string(),
        };
        let pending = |path: &Path| FileAudit {
            status: AuditStatus::Pending,
            path: path.display().to_string(),
            sha256: None,
            modified_unix_ms: None,
            detail: "pending".to_string(),
        };
        let activation_trace = Path::new(lane.activation_trace_path.as_deref().unwrap());
        let evaluation_trace = Path::new(&lane.evaluation_trace_path);
        let lane_audit = NativeLaneAudit {
            order: lane.order,
            arm: lane.arm.clone(),
            phase: NativeLanePhase::AwaitingActivation,
            teaching_trace: pending(Path::new(&lane.teaching_trace_path)),
            activation_trace: Some(pending(activation_trace)),
            procedure_verification: None,
            artifacts: Vec::new(),
            evaluation_trace: pending(evaluation_trace),
            agent_output: pending(Path::new(&lane.agent_output_path)),
            acceptance: None,
            native_memory_write_attempts: Vec::new(),
            failures: Vec::new(),
        };
        let mut pilot = plan(50);
        pilot.lanes.push(lane.clone());
        let mut audit = NativePilotAudit {
            pilot_id: pilot.pilot_id.clone(),
            run_plan: root.path().join("run-plan.json").display().to_string(),
            ready_for_evaluation: false,
            complete: false,
            all_acceptance_passed: false,
            invalid: false,
            lanes: vec![lane_audit],
        };

        validate_phase_targets_absent(&pilot, &audit, NativePilotRunPhase::Activation).unwrap();
        for (target, label) in [
            (activation_trace.to_path_buf(), "provider trace"),
            (stderr_path(activation_trace), "provider stderr"),
            (
                cleanup_stdout_path(&lane, NativePilotRunPhase::Activation).unwrap(),
                "Engram cleanup stdout",
            ),
            (
                cleanup_stderr_path(&lane, NativePilotRunPhase::Activation).unwrap(),
                "Engram cleanup stderr",
            ),
        ] {
            fs::write(&target, "occupied").unwrap();
            let message =
                validate_phase_targets_absent(&pilot, &audit, NativePilotRunPhase::Activation)
                    .unwrap_err()
                    .to_string();
            assert!(message.contains(label), "{message}");
            fs::remove_file(target).unwrap();
        }

        audit.lanes[0].phase = NativeLanePhase::ReadyForEvaluation;
        validate_phase_targets_absent(&pilot, &audit, NativePilotRunPhase::Evaluation).unwrap();
        for (target, label) in [
            (evaluation_trace.to_path_buf(), "provider trace"),
            (stderr_path(evaluation_trace), "provider stderr"),
            (
                Path::new(&lane.agent_output_path).to_path_buf(),
                "agent output",
            ),
            (
                cleanup_stdout_path(&lane, NativePilotRunPhase::Evaluation).unwrap(),
                "Engram cleanup stdout",
            ),
            (
                cleanup_stderr_path(&lane, NativePilotRunPhase::Evaluation).unwrap(),
                "Engram cleanup stderr",
            ),
        ] {
            fs::write(&target, "occupied").unwrap();
            let message =
                validate_phase_targets_absent(&pilot, &audit, NativePilotRunPhase::Evaluation)
                    .unwrap_err()
                    .to_string();
            assert!(message.contains(label), "{message}");
            fs::remove_file(target).unwrap();
        }
    }

    #[test]
    fn schema_five_attests_the_runner_while_legacy_schema_four_remains_exact() {
        assert_eq!(
            required_binary_attestation_names(4),
            &["engram", "codex", "claude_code"]
        );
        assert_eq!(
            required_binary_attestation_names(5),
            &["engram", "codex", "claude_code", "engram_eval"]
        );
    }

    #[test]
    fn evaluation_recovery_rewrites_only_the_host_schema_value() {
        let root = tempfile::tempdir().unwrap();
        let schema = root.path().join("agent-output.schema.json");
        fs::write(&schema, AGENT_OUTPUT_SCHEMA).unwrap();
        let claude = vec![
            "claude".to_string(),
            "--json-schema".to_string(),
            r#"{"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object"}"#
                .to_string(),
            "--print".to_string(),
        ];
        let codex = vec![
            "codex".to_string(),
            "--output-schema".to_string(),
            "/frozen/rejected-schema.json".to_string(),
            "exec".to_string(),
        ];
        let claude_binary = PilotBinaryAttestation {
            path: "/relocated/claude".to_string(),
            version: "claude".to_string(),
            sha256: "a".repeat(64),
        };
        let codex_binary = PilotBinaryAttestation {
            path: "/relocated/codex".to_string(),
            version: "codex".to_string(),
            sha256: "b".repeat(64),
        };

        let rewritten_claude = rewrite_evaluation_argv_for_recovery(
            &claude,
            "claude_code",
            &schema,
            &claude_binary,
            EvaluationRecoveryKind::ClaudeSchemaRejection,
        )
        .unwrap();
        let rewritten_codex = rewrite_evaluation_argv_for_recovery(
            &codex,
            "codex",
            &schema,
            &codex_binary,
            EvaluationRecoveryKind::ClaudeSchemaRejection,
        )
        .unwrap();

        assert_eq!(rewritten_claude[0], claude_binary.path);
        assert_eq!(rewritten_claude[1], claude[1]);
        assert_eq!(rewritten_claude[2], AGENT_OUTPUT_SCHEMA);
        assert_eq!(rewritten_claude[3], claude[3]);
        assert_eq!(rewritten_codex[0], codex_binary.path);
        assert_eq!(rewritten_codex[1], codex[1]);
        assert_eq!(
            rewritten_codex[2],
            schema.canonicalize().unwrap().display().to_string()
        );
        assert_eq!(rewritten_codex[3], codex[3]);
    }

    #[test]
    fn codex_terminal_recovery_requires_the_existing_portable_claude_schema() {
        let root = tempfile::tempdir().unwrap();
        let schema = root.path().join("agent-output.schema.json");
        fs::write(&schema, AGENT_OUTPUT_SCHEMA).unwrap();
        let binary = PilotBinaryAttestation {
            path: "/relocated/claude".to_string(),
            version: "claude".to_string(),
            sha256: "a".repeat(64),
        };
        let portable = vec![
            "claude".to_string(),
            "--json-schema".to_string(),
            AGENT_OUTPUT_SCHEMA.to_string(),
        ];
        let rewritten = rewrite_evaluation_argv_for_recovery(
            &portable,
            "claude_code",
            &schema,
            &binary,
            EvaluationRecoveryKind::CodexTerminalOutput,
        )
        .unwrap();
        assert_eq!(rewritten[2], AGENT_OUTPUT_SCHEMA);

        let mut drifted = portable;
        drifted[2] = r#"{"type":"object"}"#.to_string();
        assert!(rewrite_evaluation_argv_for_recovery(
            &drifted,
            "claude_code",
            &schema,
            &binary,
            EvaluationRecoveryKind::CodexTerminalOutput,
        )
        .is_err());
    }

    #[test]
    fn evaluation_recovery_allows_only_an_identical_codex_relocation() {
        let attestation = |path: &str, version: &str, marker: char| PilotBinaryAttestation {
            path: path.to_string(),
            version: version.to_string(),
            sha256: marker.to_string().repeat(64),
        };
        let source = BTreeMap::from([
            (
                "engram".to_string(),
                attestation("/source/engram", "engram", 'a'),
            ),
            (
                "codex".to_string(),
                attestation("/source/codex", "codex", 'b'),
            ),
            (
                "claude_code".to_string(),
                attestation("/source/claude", "claude", 'c'),
            ),
        ]);
        let mut relocated = source.clone();
        relocated.get_mut("codex").unwrap().path = "/recovered/codex".to_string();
        assert!(evaluation_recovery_binaries_match(
            &source,
            &relocated,
            EvaluationRecoveryKind::ClaudeSchemaRejection,
        ));

        let mut wrong_codex = relocated.clone();
        wrong_codex.get_mut("codex").unwrap().sha256 = "d".repeat(64);
        assert!(!evaluation_recovery_binaries_match(
            &source,
            &wrong_codex,
            EvaluationRecoveryKind::ClaudeSchemaRejection,
        ));

        let mut relocated_claude = relocated;
        relocated_claude.get_mut("claude_code").unwrap().path = "/recovered/claude".to_string();
        assert!(!evaluation_recovery_binaries_match(
            &source,
            &relocated_claude,
            EvaluationRecoveryKind::ClaudeSchemaRejection,
        ));
    }

    #[test]
    fn evaluation_recovery_authorizes_no_earlier_phase() {
        let mut plan = plan(15);
        plan.evaluation_recovery = Some(PreparedNativeEvaluationRecovery {
            source_run_plan: "source.json".to_string(),
            source_run_plan_sha256: "a".repeat(64),
            rejected_evaluation_trace: "trace.jsonl".to_string(),
            rejected_evaluation_trace_sha256: "b".repeat(64),
            rejected_evaluation_stderr: "trace.stderr.log".to_string(),
            rejected_evaluation_stderr_sha256: "c".repeat(64),
            rejection_reason: CLAUDE_SCHEMA_REJECTION.to_string(),
            recovery_prepared_unix_ms: 2,
            claude_remaining_calls: 3,
            agent_output_schema: "schema.json".to_string(),
        });

        assert!(validate_execution_phase_scope(&plan, NativePilotRunPhase::Teaching).is_err());
        assert!(validate_execution_phase_scope(&plan, NativePilotRunPhase::Activation).is_err());
        validate_execution_phase_scope(&plan, NativePilotRunPhase::Evaluation).unwrap();
    }

    #[test]
    fn accepts_only_exact_claude_turn_boundary_budget_envelope() {
        let result = serde_json::json!({
            "type": "result",
            "subtype": "error_max_budget_usd",
            "is_error": true,
            "terminal_reason": "budget_exhausted",
            "stop_reason": "end_turn",
            "total_cost_usd": 0.0534471,
            "errors": ["Reached maximum budget ($0.05)"]
        });
        assert!(is_turn_boundary_budget_exhaustion(&result));
        assert_eq!(cost_usd_to_microusd(0.0534471).unwrap(), 53_448);

        for (field, value) in [
            ("subtype", serde_json::json!("error_during_execution")),
            ("terminal_reason", serde_json::json!("provider_error")),
            ("stop_reason", serde_json::json!("max_turns")),
            ("is_error", serde_json::json!(false)),
        ] {
            let mut changed = result.clone();
            changed[field] = value;
            assert!(!is_turn_boundary_budget_exhaustion(&changed));
        }
    }

    #[test]
    fn completed_claude_teaching_recovery_requires_successful_terminal_result() {
        let root = tempfile::tempdir().unwrap();
        let trace = root.path().join("trace.jsonl");
        let success = serde_json::json!({
            "type": "result",
            "subtype": "success",
            "is_error": false,
            "terminal_reason": "completed",
            "stop_reason": "end_turn",
            "total_cost_usd": 0.0466201
        });
        fs::write(&trace, format!("{success}\n")).unwrap();

        let summary = validate_completed_claude_teaching_trace(&trace).unwrap();
        assert!(!summary.accepted_turn_boundary_budget_exit);
        assert_eq!(summary.provider_reported_cost_microusd, Some(46_621));

        let mut rejected = success;
        rejected["subtype"] = serde_json::json!("error_max_budget_usd");
        fs::write(&trace, format!("{rejected}\n")).unwrap();
        assert!(validate_completed_claude_teaching_trace(&trace).is_err());
    }

    #[test]
    fn evaluation_recovery_requires_one_completed_structured_output_tool() {
        let root = tempfile::tempdir().unwrap();
        let trace = root.path().join("trace.jsonl");
        let output = serde_json::json!({
            "answer": "abstained",
            "repository_remote": "atlas",
            "project": null,
            "component": null,
            "first_action": "orient",
            "returned_context_keys": [],
            "applied_context_keys": [],
            "evidence_targets": [],
            "abstained": true
        });
        let terminal = serde_json::json!({
            "type": "result",
            "subtype": "error_max_budget_usd",
            "is_error": true,
            "terminal_reason": "budget_exhausted",
            "stop_reason": "tool_use",
            "total_cost_usd": 0.0513728,
            "errors": ["Reached maximum budget ($0.05)"]
        });
        fs::write(
            &trace,
            format!(
                "{}\n{}\n",
                serde_json::json!({
                    "type": "assistant",
                    "message": {"content": [{
                        "type": "tool_use",
                        "name": "StructuredOutput",
                        "input": output
                    }]}
                }),
                terminal
            ),
        )
        .unwrap();

        let summary = validate_recoverable_claude_evaluation_trace(
            &trace,
            EXPIRED_PROCEDURE_NATIVE_PILOT_SCHEMA_VERSION,
        )
        .unwrap();
        assert!(summary.accepted_turn_boundary_budget_exit);
        assert_eq!(summary.provider_reported_cost_microusd, Some(51_373));

        fs::write(&trace, format!("{}\n{}\n", output, terminal)).unwrap();
        assert!(validate_recoverable_claude_evaluation_trace(
            &trace,
            EXPIRED_PROCEDURE_NATIVE_PILOT_SCHEMA_VERSION,
        )
        .is_err());
    }

    #[test]
    fn frozen_agent_profile_contract_detects_schema_drift() {
        let expected = plan(50).engram_mcp_contract;
        validate_frozen_engram_contract(&expected, &expected).unwrap();

        let mut drifted = expected.clone();
        drifted.mcp_tools_sha256 = "d".repeat(64);
        assert!(validate_frozen_engram_contract(&expected, &drifted)
            .unwrap_err()
            .to_string()
            .contains("contract drifted after preparation"));
    }

    #[test]
    fn codex_authentication_status_is_strict_and_sanitized() {
        assert_eq!(
            classify_codex_authentication(true, CODEX_CHATGPT_LOGIN_STATUS, ""),
            NativePilotCodexAuthState::Ready
        );
        assert_eq!(
            classify_codex_authentication(false, "", "Not logged in"),
            NativePilotCodexAuthState::NotLoggedIn
        );
        assert_eq!(
            classify_codex_authentication(true, "Logged in using an API key", ""),
            NativePilotCodexAuthState::WrongAuthenticationMode
        );
        assert_eq!(
            classify_codex_authentication(false, "", "keyring unavailable"),
            NativePilotCodexAuthState::CheckFailed
        );
    }

    #[cfg(unix)]
    fn synthetic_six_auth_targets(
        root: &Path,
        contents: &[u8],
    ) -> (
        Vec<CodexAuthCacheTarget>,
        Vec<CodexAuthCacheDestinationReceipt>,
    ) {
        let owner = current_effective_uid().unwrap();
        let mut targets = Vec::new();
        let mut receipts = Vec::new();
        for lane_order in 1..=6 {
            let home_path = root.join(format!("codex-home-{lane_order}"));
            create_private_dir_all(&home_path).unwrap();
            let home = open_private_codex_home(&home_path, owner).unwrap();
            let identity = create_auth_cache_relative(&home, contents).unwrap();
            let auth_json = home_path.join("auth.json");
            targets.push(CodexAuthCacheTarget {
                lane_order,
                home: home_path,
                auth_json: auth_json.clone(),
            });
            receipts.push(CodexAuthCacheDestinationReceipt {
                lane_order,
                auth_json: auth_json.display().to_string(),
                identity,
            });
        }
        (targets, receipts)
    }

    fn family_v3_auth_plan(root: &Path, codex_lane_count: u32) -> PreparedNativePilot {
        let mut pilot = plan(0);
        pilot.stale_safety = Some(crate::native_pilot::PreparedNativeStaleSafetyBinding {
            family_version: 3,
            protocol_snapshot_file: "stale-safety.protocol.snapshot.json".to_string(),
            protocol_snapshot_sha256: "a".repeat(64),
            retention_precondition_file: "stale-safety.retention-precondition.json".to_string(),
            retention_precondition_sha256: "b".repeat(64),
            agent_output_schema_sha256: "c".repeat(64),
        });
        pilot.lanes = (1..=codex_lane_count)
            .map(|order| {
                let mut lane = receipt_test_lane(root, order, "codex");
                let home = root.join(format!("family-v3-home-{order}"));
                create_private_dir_all(&home).unwrap();
                lane.environment
                    .insert("CODEX_HOME".to_string(), home.display().to_string());
                lane.codex_authentication = Some(PreparedCodexAuthentication {
                    mode: CodexAuthenticationMode::ChatgptFileCache,
                    credential_store: "file".to_string(),
                    login_argv: vec!["codex".to_string(), "login".to_string()],
                    status_argv: vec![
                        "codex".to_string(),
                        "login".to_string(),
                        "status".to_string(),
                    ],
                    expected_status: CODEX_CHATGPT_LOGIN_STATUS.to_string(),
                });
                lane
            })
            .collect();
        pilot
    }

    #[cfg(unix)]
    fn family_v3_claude_test_lane(
        root: &Path,
        order: u32,
        executable: &Path,
    ) -> PreparedNativeLane {
        use std::os::unix::fs::PermissionsExt;

        let lane_root = root.join(format!("lane-{order}"));
        for directory in [
            "teaching",
            "evaluation",
            "claude-memory",
            "provider-home",
            "provider-tmp",
            "claude-config",
        ] {
            create_private_dir_all(&lane_root.join(directory)).unwrap();
        }
        fs::set_permissions(&lane_root, fs::Permissions::from_mode(0o700)).unwrap();
        let lane_root = lane_root.canonicalize().unwrap();
        let mut lane = receipt_test_lane(root, order, "claude_code");
        lane.arm = "claude_code_auto_memory".to_string();
        lane.memory_layer = MemoryLayer::Native;
        lane.teaching_cwd = lane_root.join("teaching").display().to_string();
        lane.evaluation_cwd = lane_root.join("evaluation").display().to_string();
        lane.acceptance_contract = lane_root.join("acceptance.json").display().to_string();
        lane.teaching_trace_path = lane_root.join("teaching.jsonl").display().to_string();
        lane.evaluation_trace_path = lane_root.join("evaluation.jsonl").display().to_string();
        lane.agent_output_path = lane_root.join("agent-output.json").display().to_string();
        lane.environment = BTreeMap::from([
            (
                "CLAUDE_CODE_DISABLE_AUTO_MEMORY".to_string(),
                "1".to_string(),
            ),
            (
                "CLAUDE_CONFIG_DIR".to_string(),
                lane_root.join("claude-config").display().to_string(),
            ),
            ("DISABLE_TELEMETRY".to_string(), "1".to_string()),
            (
                "HOME".to_string(),
                lane_root.join("provider-home").display().to_string(),
            ),
            ("LANG".to_string(), "C.UTF-8".to_string()),
            ("LC_ALL".to_string(), "C.UTF-8".to_string()),
            (
                "PATH".to_string(),
                "/usr/bin:/bin:/usr/sbin:/sbin".to_string(),
            ),
            ("SHELL".to_string(), "/bin/bash".to_string()),
            (
                "TMPDIR".to_string(),
                lane_root.join("provider-tmp").display().to_string(),
            ),
        ]);
        let settings_value = serde_json::json!({
            "autoMemoryEnabled": true,
            "autoMemoryDirectory": lane_root.join("claude-memory").display().to_string(),
        });
        let mcp_value = serde_json::json!({"mcpServers": {}});
        let settings = serde_json::to_string(&settings_value).unwrap();
        let mcp = serde_json::to_string(&mcp_value).unwrap();
        write_private_text(&lane_root.join("claude-settings.json"), &settings).unwrap();
        write_private_text(&lane_root.join("claude-mcp-empty.json"), &mcp).unwrap();
        let argv = vec![
            executable.display().to_string(),
            "--settings".to_string(),
            settings,
            "--mcp-config".to_string(),
            mcp,
            "prompt".to_string(),
        ];
        lane.teaching_argv = argv.clone();
        lane.evaluation_argv = argv;
        lane
    }

    #[cfg(unix)]
    fn family_v3_test_lifecycle(
        plan: &PreparedNativePilot,
        run_plan: &Path,
    ) -> CodexAuthCacheLifecycle {
        let owner = current_effective_uid().unwrap();
        let destinations: Vec<CodexAuthCacheDestinationReceipt> =
            exact_six_codex_auth_targets(plan)
                .unwrap()
                .into_iter()
                .map(|target| {
                    let home = open_private_codex_home(&target.home, owner).unwrap();
                    CodexAuthCacheDestinationReceipt {
                        lane_order: target.lane_order,
                        auth_json: target.auth_json.display().to_string(),
                        identity: create_auth_cache_relative(&home, b"{}\n").unwrap(),
                    }
                })
                .collect();
        let directory = PrivateAuthLifecycleDirectory::create_new(run_plan).unwrap();
        let boot_id = current_boot_id().unwrap();
        let run_plan_sha256 = sha256_file(run_plan).unwrap();
        let intent = CodexAuthCacheProvisionIntent {
            schema_version: CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION,
            pilot_id: plan.pilot_id.clone(),
            run_plan_sha256: run_plan_sha256.clone(),
            boot_id: boot_id.clone(),
            destinations: destinations
                .iter()
                .map(|destination| CodexAuthCacheDestinationPlan {
                    lane_order: destination.lane_order,
                    auth_json: destination.auth_json.clone(),
                })
                .collect(),
            created_unix_ms: unix_ms(SystemTime::now()).unwrap(),
        };
        let provision_intent_sha256 = directory
            .write_json_pair_exclusive("provision-intent", &intent)
            .unwrap();
        let receipt = CodexAuthCacheProvisionReceipt {
            schema_version: CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION,
            pilot_id: plan.pilot_id.clone(),
            run_plan_sha256,
            provision_intent_sha256,
            boot_id,
            destinations,
            authentication_ready: true,
            completed_unix_ms: unix_ms(SystemTime::now()).unwrap(),
        };
        let manifest_sha256 = directory
            .write_json_pair_exclusive("provision-receipt", &receipt)
            .unwrap();
        CodexAuthCacheLifecycle {
            directory,
            receipt: Some(receipt),
            manifest_sha256,
            active_phase: None,
        }
    }

    #[cfg(unix)]
    fn replace_private_file_with_same_bytes(path: &Path) {
        let bytes = fs::read(path).unwrap();
        fs::remove_file(path).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        write_private_text(path, &text).unwrap();
    }

    #[cfg(unix)]
    fn shell_single_quote(value: &str) -> String {
        format!("'{}'", value.replace('\'', "'\"'\"'"))
    }

    fn unwrap_provider_lane_result<T>(result: Result<T, ProviderLaneFailure>) -> T {
        match result {
            Ok(value) => value,
            Err(failure) => panic!(
                "unexpected provider-lane failure: {}",
                failure.into_parts().0
            ),
        }
    }

    #[cfg(unix)]
    #[test]
    fn family_v3_auth_lifecycle_requires_exactly_six_file_cache_lanes() {
        let root = tempfile::tempdir().unwrap();
        assert!(
            requires_exact_six_codex_auth_lifecycle(&family_v3_auth_plan(root.path(), 6)).unwrap()
        );
        assert!(
            requires_exact_six_codex_auth_lifecycle(&family_v3_auth_plan(root.path(), 5))
                .unwrap_err()
                .to_string()
                .contains("exactly six")
        );

        let mut future = family_v3_auth_plan(root.path(), 6);
        future.stale_safety.as_mut().unwrap().family_version = 4;
        assert!(requires_exact_six_codex_auth_lifecycle(&future)
            .unwrap_err()
            .to_string()
            .contains("unsupported"));
        let mut historical = future;
        historical.stale_safety.as_mut().unwrap().family_version = 1;
        assert!(!requires_exact_six_codex_auth_lifecycle(&historical).unwrap());
    }

    #[cfg(unix)]
    #[test]
    fn family_v3_bad_later_config_blocks_the_post_preflight_admission_boundary() {
        use std::os::unix::fs::PermissionsExt;

        let root = tempfile::tempdir().unwrap();
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let provider = root.path().join("provider");
        write_test_executable(&provider, "#!/bin/sh\nexit 97\n");
        let mut pilot = family_v3_auth_plan(root.path(), 6);
        pilot
            .lanes
            .push(family_v3_claude_test_lane(root.path(), 7, &provider));
        pilot
            .lanes
            .push(family_v3_claude_test_lane(root.path(), 8, &provider));
        let bad_lane = pilot.lanes.last_mut().unwrap();
        let mcp_index = bad_lane
            .evaluation_argv
            .iter()
            .position(|argument| argument == "--mcp-config")
            .unwrap();
        bad_lane.evaluation_argv[mcp_index + 1].push(' ');
        let run_plan = root.path().join("run-plan.json");
        write_private_json(&run_plan, &pilot).unwrap();
        let mut lifecycle = family_v3_test_lifecycle(&pilot, &run_plan);
        let bundle_admission_marker = root.path().join("bundle-admission-called");
        let result = with_family_v3_claude_config_preflight(&pilot, |_| {
            lifecycle.begin_phase(&pilot, &run_plan, NativePilotRunPhase::Teaching)?;
            fs::write(&bundle_admission_marker, b"called")?;
            Ok(())
        });
        assert!(result.is_err());
        assert!(!bundle_admission_marker.exists());
        assert!(!lifecycle
            .directory
            .entry_exists("phase-teaching-intent.json")
            .unwrap());
        for lane_order in 1..=8 {
            assert!(!lifecycle
                .directory
                .entry_exists(&format!(
                    "{}.json",
                    lane_lifecycle_stem(NativePilotRunPhase::Teaching, lane_order, "admission")
                ))
                .unwrap());
        }
    }

    #[cfg(unix)]
    #[test]
    fn family_v3_config_drift_preserves_pre_dispatch_stage_and_receipt_boundary() {
        use std::os::unix::fs::PermissionsExt;

        let pre_admission_root = tempfile::tempdir().unwrap();
        fs::set_permissions(pre_admission_root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let pre_admission_marker = pre_admission_root.path().join("provider-spawned");
        let pre_admission_provider = pre_admission_root.path().join("provider");
        write_test_executable(
            &pre_admission_provider,
            &format!(
                "#!/bin/sh\nprintf spawned > {}\nprintf '%s\\n' '{{\"type\":\"result\",\"total_cost_usd\":0.0}}'\n",
                shell_single_quote(pre_admission_marker.to_str().unwrap())
            ),
        );
        let mut pilot = family_v3_auth_plan(pre_admission_root.path(), 6);
        pilot.lanes.push(family_v3_claude_test_lane(
            pre_admission_root.path(),
            7,
            &pre_admission_provider,
        ));
        let run_plan = pre_admission_root.path().join("run-plan.json");
        write_private_json(&run_plan, &pilot).unwrap();
        let mut lifecycle = family_v3_test_lifecycle(&pilot, &run_plan);
        let attestations = with_family_v3_claude_config_preflight(&pilot, |attestations| {
            lifecycle.begin_phase(&pilot, &run_plan, NativePilotRunPhase::Teaching)?;
            Ok(attestations)
        })
        .unwrap();
        let lane = &pilot.lanes[6];
        let expected = &attestations[&lane.order];
        replace_private_file_with_same_bytes(Path::new(&expected.settings.path));
        let failure = match lifecycle.admit_lane(
            &pilot,
            &run_plan,
            lane,
            NativePilotRunPhase::Teaching,
            Some(expected),
        ) {
            Ok(_) => panic!("pre-admission config drift unexpectedly admitted"),
            Err(failure) => failure,
        };
        let (_, stage, receipt_sha256, _) = failure.into_parts();
        assert_eq!(stage, ProviderLaneFailureStage::ConfigArtifactPreDispatch);
        assert_eq!(receipt_sha256, None);
        assert!(!pre_admission_marker.exists());
        assert!(!lifecycle
            .directory
            .entry_exists(&format!(
                "{}.json",
                lane_lifecycle_stem(NativePilotRunPhase::Teaching, lane.order, "admission")
            ))
            .unwrap());

        let pre_spawn_root = tempfile::tempdir().unwrap();
        fs::set_permissions(pre_spawn_root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let pre_spawn_marker = pre_spawn_root.path().join("provider-spawned");
        let pre_spawn_provider = pre_spawn_root.path().join("provider");
        write_test_executable(
            &pre_spawn_provider,
            &format!(
                "#!/bin/sh\nprintf spawned > {}\nprintf '%s\\n' '{{\"type\":\"result\",\"total_cost_usd\":0.0}}'\n",
                shell_single_quote(pre_spawn_marker.to_str().unwrap())
            ),
        );
        let mut pilot = family_v3_auth_plan(pre_spawn_root.path(), 6);
        pilot.lanes.push(family_v3_claude_test_lane(
            pre_spawn_root.path(),
            7,
            &pre_spawn_provider,
        ));
        let run_plan = pre_spawn_root.path().join("run-plan.json");
        write_private_json(&run_plan, &pilot).unwrap();
        let mut lifecycle = family_v3_test_lifecycle(&pilot, &run_plan);
        let attestations = with_family_v3_claude_config_preflight(&pilot, |attestations| {
            lifecycle.begin_phase(&pilot, &run_plan, NativePilotRunPhase::Teaching)?;
            Ok(attestations)
        })
        .unwrap();
        let lane = &pilot.lanes[6];
        let expected = &attestations[&lane.order];
        let admission_sha256 = unwrap_provider_lane_result(lifecycle.admit_lane(
            &pilot,
            &run_plan,
            lane,
            NativePilotRunPhase::Teaching,
            Some(expected),
        ));
        replace_private_file_with_same_bytes(Path::new(&expected.settings.path));
        let failure = match execute_lane(
            &pilot,
            lane,
            NativePilotRunPhase::Teaching,
            Some(&lifecycle),
            Some(expected),
            || Ok(()),
        ) {
            Ok(_) => panic!("pre-spawn config drift unexpectedly executed"),
            Err(failure) => failure,
        }
        .with_pre_dispatch_receipt_sha256(Some(admission_sha256.clone()));
        let (_, stage, receipt_sha256, _) = failure.into_parts();
        assert_eq!(stage, ProviderLaneFailureStage::ConfigArtifactPreDispatch);
        assert_eq!(receipt_sha256, Some(admission_sha256));
        assert!(!pre_spawn_marker.exists());
        assert!(!Path::new(&lane.teaching_trace_path).exists());
        assert!(!stderr_path(Path::new(&lane.teaching_trace_path)).exists());
    }

    #[cfg(unix)]
    #[test]
    fn family_v3_post_provider_config_drift_is_generic_ambiguous_failure() {
        use std::os::unix::fs::PermissionsExt;

        let root = tempfile::tempdir().unwrap();
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let provider = root.path().join("provider");
        let marker = root.path().join("provider-spawned");
        let mut pilot = family_v3_auth_plan(root.path(), 6);
        pilot
            .lanes
            .push(family_v3_claude_test_lane(root.path(), 7, &provider));
        let lane = &pilot.lanes[6];
        let settings_path = Path::new(&lane.acceptance_contract)
            .parent()
            .unwrap()
            .join("claude-settings.json");
        let replacement = root.path().join("replacement-settings.json");
        write_private_text(&replacement, &fs::read_to_string(&settings_path).unwrap()).unwrap();
        write_test_executable(
            &provider,
            &format!(
                "#!/bin/sh\nprintf spawned > {}\n/bin/mv -f {} {}\nprintf '%s\\n' '{{\"type\":\"result\",\"total_cost_usd\":0.0}}'\n",
                shell_single_quote(marker.to_str().unwrap()),
                shell_single_quote(replacement.to_str().unwrap()),
                shell_single_quote(settings_path.to_str().unwrap()),
            ),
        );
        let run_plan = root.path().join("run-plan.json");
        write_private_json(&run_plan, &pilot).unwrap();
        let mut lifecycle = family_v3_test_lifecycle(&pilot, &run_plan);
        let attestations = with_family_v3_claude_config_preflight(&pilot, |attestations| {
            lifecycle.begin_phase(&pilot, &run_plan, NativePilotRunPhase::Teaching)?;
            Ok(attestations)
        })
        .unwrap();
        let lane = &pilot.lanes[6];
        let expected = &attestations[&lane.order];
        let admission_sha256 = unwrap_provider_lane_result(lifecycle.admit_lane(
            &pilot,
            &run_plan,
            lane,
            NativePilotRunPhase::Teaching,
            Some(expected),
        ));
        let expected_receipt_sha256 = admission_sha256.clone();
        let failure = match execute_lane(
            &pilot,
            lane,
            NativePilotRunPhase::Teaching,
            Some(&lifecycle),
            Some(expected),
            || Ok(()),
        ) {
            Ok(_) => panic!("provider-time config drift unexpectedly completed"),
            Err(failure) => failure,
        }
        .with_pre_dispatch_receipt_sha256(Some(admission_sha256));
        let (error, stage, receipt_sha256, _) = failure.into_parts();
        assert_eq!(stage, ProviderLaneFailureStage::PostDispatch);
        assert_eq!(receipt_sha256, Some(expected_receipt_sha256));
        assert!(error.to_string().contains("provider terminal ambiguous;"));
        assert!(marker.exists());
        assert!(Path::new(&lane.teaching_trace_path).is_file());
        assert!(stderr_path(Path::new(&lane.teaching_trace_path)).is_file());
    }

    #[cfg(unix)]
    #[test]
    fn family_v3_bounded_dispatch_distinguishes_not_spawned_from_post_spawn_failure() {
        use std::os::unix::fs::PermissionsExt;

        let not_spawned_root = tempfile::tempdir().unwrap();
        fs::set_permissions(not_spawned_root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let mut pilot = family_v3_auth_plan(not_spawned_root.path(), 6);
        pilot.lanes.push(family_v3_claude_test_lane(
            not_spawned_root.path(),
            7,
            &not_spawned_root.path().join("missing-provider"),
        ));
        let lane = &pilot.lanes[6];
        let attestation = validate_native_stale_v3_claude_config_artifacts(lane).unwrap();
        let receipt_sha256 = "d".repeat(64);
        let failure = match execute_lane(
            &pilot,
            lane,
            NativePilotRunPhase::Teaching,
            None,
            Some(&attestation),
            || Ok(()),
        ) {
            Ok(_) => panic!("missing provider executable unexpectedly spawned"),
            Err(failure) => failure,
        }
        .with_pre_dispatch_receipt_sha256(Some(receipt_sha256.clone()));
        let (error, stage, observed_receipt, _) = failure.into_parts();
        assert_eq!(stage, ProviderLaneFailureStage::RunnerPreDispatch);
        assert_eq!(observed_receipt, Some(receipt_sha256));
        assert!(error
            .to_string()
            .contains("bounded provider was not spawned"));
        assert!(!Path::new(&lane.teaching_trace_path).exists());
        assert!(!stderr_path(Path::new(&lane.teaching_trace_path)).exists());

        let post_spawn_root = tempfile::tempdir().unwrap();
        fs::set_permissions(post_spawn_root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let provider = post_spawn_root.path().join("local-stub");
        write_test_executable(&provider, "#!/bin/sh\nexit 0\n");
        let mut pilot = family_v3_auth_plan(post_spawn_root.path(), 6);
        pilot.lanes.push(family_v3_claude_test_lane(
            post_spawn_root.path(),
            7,
            &provider,
        ));
        let lane = &pilot.lanes[6];
        let attestation = validate_native_stale_v3_claude_config_artifacts(lane).unwrap();
        crate::native_execution::inject_next_post_spawn_cleanup_unproven_for_test();
        let receipt_sha256 = "e".repeat(64);
        let failure = match execute_lane(
            &pilot,
            lane,
            NativePilotRunPhase::Teaching,
            None,
            Some(&attestation),
            || Ok(()),
        ) {
            Ok(_) => panic!("injected post-spawn failure unexpectedly completed"),
            Err(failure) => failure,
        }
        .with_pre_dispatch_receipt_sha256(Some(receipt_sha256.clone()));
        let (error, stage, observed_receipt, quarantine) = failure.into_parts();
        assert_eq!(
            stage,
            ProviderLaneFailureStage::PostSpawnOutputSetup {
                process_cleanup_proven: false
            }
        );
        assert_eq!(observed_receipt, Some(receipt_sha256));
        assert!(error
            .to_string()
            .contains("post-spawn provider output setup failed"));
        assert!(quarantine.is_some());
        assert!(!Path::new(&lane.teaching_trace_path).exists());
        assert!(!stderr_path(Path::new(&lane.teaching_trace_path)).exists());
        drop(quarantine);
    }

    #[cfg(unix)]
    #[test]
    fn family_v3_post_spawn_output_setup_failures_preserve_exact_artifact_states() {
        use std::os::unix::fs::PermissionsExt;

        let cases = [
            (
                InjectedProviderOutputSetupFailure::StdoutFile,
                "stdout_file",
                false,
                false,
            ),
            (
                InjectedProviderOutputSetupFailure::StderrFile,
                "stderr_file",
                true,
                false,
            ),
            (
                InjectedProviderOutputSetupFailure::StdoutPipe,
                "stdout_pipe",
                true,
                true,
            ),
            (
                InjectedProviderOutputSetupFailure::StderrPipe,
                "stderr_pipe",
                true,
                true,
            ),
        ];
        for (fault, expected_stage, trace_exists, stderr_exists) in cases {
            let root = tempfile::tempdir().unwrap();
            fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
            let provider = root.path().join("provider");
            write_test_executable(&provider, "#!/bin/sh\n/bin/sleep 30\n");
            let mut pilot = family_v3_auth_plan(root.path(), 6);
            pilot
                .lanes
                .push(family_v3_claude_test_lane(root.path(), 7, &provider));
            let lane = &pilot.lanes[6];
            let attestation = validate_native_stale_v3_claude_config_artifacts(lane).unwrap();
            inject_next_provider_output_setup_failure_for_test(fault);
            let receipt_sha256 = "a".repeat(64);
            let failure = match execute_lane(
                &pilot,
                lane,
                NativePilotRunPhase::Teaching,
                None,
                Some(&attestation),
                || Ok(()),
            ) {
                Ok(_) => panic!("injected post-spawn setup failure unexpectedly completed"),
                Err(failure) => failure,
            }
            .with_pre_dispatch_receipt_sha256(Some(receipt_sha256.clone()));
            let (error, stage, observed_receipt, quarantine) = failure.into_parts();
            assert_eq!(
                stage,
                ProviderLaneFailureStage::PostSpawnOutputSetup {
                    process_cleanup_proven: true,
                }
            );
            assert_eq!(observed_receipt, Some(receipt_sha256));
            assert!(error
                .to_string()
                .contains(&format!("stage={expected_stage}")));
            assert!(quarantine.is_none());
            assert_eq!(Path::new(&lane.teaching_trace_path).exists(), trace_exists);
            assert_eq!(
                stderr_path(Path::new(&lane.teaching_trace_path)).exists(),
                stderr_exists
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn family_v3_terminal_cleanup_uncertainty_retains_guard_for_durable_freeze() {
        use std::os::unix::fs::PermissionsExt;

        let root = tempfile::tempdir().unwrap();
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let provider = root.path().join("provider");
        write_test_executable(&provider, "#!/bin/sh\nexit 0\n");
        let mut pilot = family_v3_auth_plan(root.path(), 6);
        pilot
            .lanes
            .push(family_v3_claude_test_lane(root.path(), 7, &provider));
        let lane = &pilot.lanes[6];
        let attestation = validate_native_stale_v3_claude_config_artifacts(lane).unwrap();
        inject_next_provider_output_setup_failure_for_test(
            InjectedProviderOutputSetupFailure::TerminalCleanup,
        );
        let failure = match execute_lane(
            &pilot,
            lane,
            NativePilotRunPhase::Teaching,
            None,
            Some(&attestation),
            || Ok(()),
        ) {
            Ok(_) => panic!("injected cleanup uncertainty unexpectedly completed"),
            Err(failure) => failure,
        };
        let (error, stage, receipt, quarantine) = failure.into_parts();
        assert_eq!(stage, ProviderLaneFailureStage::PostDispatch);
        assert_eq!(receipt, None);
        assert!(error.to_string().contains("cleanup remained unproven"));
        assert!(quarantine.is_some());
        assert!(Path::new(&lane.teaching_trace_path).is_file());
        assert!(stderr_path(Path::new(&lane.teaching_trace_path)).is_file());
        drop(quarantine);
    }

    #[cfg(unix)]
    #[test]
    fn family_v3_final_bundle_revalidation_blocks_test_double_dispatch() {
        use std::cell::Cell;
        use std::os::unix::fs::PermissionsExt;

        let root = tempfile::tempdir().unwrap();
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let provider = root.path().join("provider");
        let spawn_marker = root.path().join("provider-spawned");
        write_test_executable(
            &provider,
            &format!(
                "#!/bin/sh\nprintf spawned > {}\nexit 0\n",
                shell_single_quote(spawn_marker.to_str().unwrap())
            ),
        );
        let mut pilot = family_v3_auth_plan(root.path(), 6);
        pilot
            .lanes
            .push(family_v3_claude_test_lane(root.path(), 7, &provider));
        let lane = &pilot.lanes[6];
        let attestation = validate_native_stale_v3_claude_config_artifacts(lane).unwrap();
        let receipt_sha256 = "f".repeat(64);
        let dispatch_boundary_calls = Cell::new(0_u32);
        let failure = match execute_lane(
            &pilot,
            lane,
            NativePilotRunPhase::Teaching,
            None,
            Some(&attestation),
            || {
                dispatch_boundary_calls.set(dispatch_boundary_calls.get() + 1);
                Err(EvalError::Invalid(
                    "provider bundle dispatch-boundary identity drifted".to_string(),
                ))
            },
        ) {
            Ok(_) => panic!("drifted bundle unexpectedly allowed provider dispatch"),
            Err(failure) => failure,
        }
        .with_pre_dispatch_receipt_sha256(Some(receipt_sha256.clone()));
        let (error, stage, observed_receipt, _) = failure.into_parts();
        assert_eq!(stage, ProviderLaneFailureStage::RunnerPreDispatch);
        assert_eq!(observed_receipt, Some(receipt_sha256));
        assert!(error
            .to_string()
            .contains("provider bundle dispatch-boundary identity drifted"));
        assert_eq!(dispatch_boundary_calls.get(), 1);
        assert!(!spawn_marker.exists());
        assert!(!Path::new(&lane.teaching_trace_path).exists());
        assert!(!stderr_path(Path::new(&lane.teaching_trace_path)).exists());
    }

    #[test]
    fn family_v3_treatment_ordinals_map_to_exact_phase_lanes() {
        let root = tempfile::tempdir().unwrap();
        let mut pilot = plan(0);
        pilot.lanes = (1..=12)
            .map(|order| {
                receipt_test_lane(
                    root.path(),
                    order,
                    if order % 2 == 1 {
                        "codex"
                    } else {
                        "claude_code"
                    },
                )
            })
            .collect();

        assert_eq!(
            family_v3_treatment_phase_lane(&pilot, 1).unwrap(),
            (NativePilotRunPhase::Teaching, 1)
        );
        assert_eq!(
            family_v3_treatment_phase_lane(&pilot, 12).unwrap(),
            (NativePilotRunPhase::Teaching, 12)
        );
        assert_eq!(
            family_v3_treatment_phase_lane(&pilot, 13).unwrap(),
            (NativePilotRunPhase::Activation, 1)
        );
        assert_eq!(
            family_v3_treatment_phase_lane(&pilot, 18).unwrap(),
            (NativePilotRunPhase::Activation, 11)
        );
        assert_eq!(
            family_v3_treatment_phase_lane(&pilot, 19).unwrap(),
            (NativePilotRunPhase::Evaluation, 1)
        );
        assert_eq!(
            family_v3_treatment_phase_lane(&pilot, 30).unwrap(),
            (NativePilotRunPhase::Evaluation, 12)
        );
        assert!(family_v3_treatment_phase_lane(&pilot, 0).is_err());
        assert!(family_v3_treatment_phase_lane(&pilot, 31).is_err());
    }

    #[test]
    fn family_v3_failure_seal_inputs_and_receipt_tokens_fail_closed() {
        let trace = PathBuf::from("trace.jsonl");
        let stderr = PathBuf::from("trace.stderr.log");
        assert!(provider_failure_seal_identity(
            &[],
            &BTreeMap::new(),
            trace.clone(),
            stderr.clone(),
        )
        .unwrap_err()
        .to_string()
        .contains("seal identity is incomplete"));
        let identity = provider_failure_seal_identity(
            &["provider".to_string()],
            &BTreeMap::new(),
            trace,
            stderr,
        )
        .unwrap();
        assert_eq!(
            identity.argv_sha256,
            argv_sha256(&["provider".to_string()]).unwrap()
        );

        let evidence = NativeProviderBundlePreProviderReceiptEvidence {
            ordinal: 1,
            child: "treatment".to_string(),
            phase: "teaching".to_string(),
            lane_order: 1,
            receipt_sha256: "a".repeat(64),
            argv_sha256: "b".repeat(64),
            receipt_unix_ms: 1,
            typed_binding_sha256: "c".repeat(64),
        };
        let original = reconcile_pre_provider_receipt_failure(
            EvalError::Invalid("original".to_string()),
            Some(&evidence.receipt_sha256),
            Some(&evidence),
        );
        assert_eq!(original.to_string(), "invalid evaluation data: original");
        let inconsistent = reconcile_pre_provider_receipt_failure(
            EvalError::Invalid("original".to_string()),
            Some(&"d".repeat(64)),
            Some(&evidence),
        );
        assert!(inconsistent
            .to_string()
            .contains("consumed bundle admission remains permanently non-replayable"));
    }

    #[cfg(unix)]
    #[test]
    fn family_v3_lifecycle_rejects_admission_and_terminal_digest_mutation() {
        use std::os::unix::fs::PermissionsExt;

        let admission_root = tempfile::tempdir().unwrap();
        fs::set_permissions(admission_root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let provider = admission_root.path().join("provider");
        write_test_executable(&provider, "#!/bin/sh\nexit 97\n");
        let mut pilot = family_v3_auth_plan(admission_root.path(), 6);
        pilot.lanes.push(family_v3_claude_test_lane(
            admission_root.path(),
            7,
            &provider,
        ));
        let run_plan = admission_root.path().join("run-plan.json");
        write_private_json(&run_plan, &pilot).unwrap();
        let mut lifecycle = family_v3_test_lifecycle(&pilot, &run_plan);
        let attestations = with_family_v3_claude_config_preflight(&pilot, |attestations| {
            lifecycle.begin_phase(&pilot, &run_plan, NativePilotRunPhase::Teaching)?;
            Ok(attestations)
        })
        .unwrap();
        let lane = &pilot.lanes[6];
        let admission_sha256 = unwrap_provider_lane_result(lifecycle.admit_lane(
            &pilot,
            &run_plan,
            lane,
            NativePilotRunPhase::Teaching,
            Some(&attestations[&lane.order]),
        ));
        assert_eq!(
            lifecycle
                .read_validated_lane_admission(
                    &pilot,
                    &run_plan,
                    lane,
                    NativePilotRunPhase::Teaching,
                )
                .unwrap()
                .1,
            admission_sha256
        );
        let lifecycle_root = auth_lifecycle_path(&run_plan).unwrap();
        let admission_digest_path = lifecycle_root.join(format!(
            "{}.sha256",
            lane_lifecycle_stem(NativePilotRunPhase::Teaching, lane.order, "admission")
        ));
        fs::write(&admission_digest_path, format!("{}\n", "0".repeat(64))).unwrap();
        assert!(lifecycle
            .read_validated_lane_admission(&pilot, &run_plan, lane, NativePilotRunPhase::Teaching,)
            .unwrap_err()
            .to_string()
            .contains("failed digest validation"));

        let terminal_root = tempfile::tempdir().unwrap();
        fs::set_permissions(terminal_root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let provider = terminal_root.path().join("provider");
        write_test_executable(&provider, "#!/bin/sh\nexit 97\n");
        let mut pilot = family_v3_auth_plan(terminal_root.path(), 6);
        pilot.lanes.push(family_v3_claude_test_lane(
            terminal_root.path(),
            7,
            &provider,
        ));
        let run_plan = terminal_root.path().join("run-plan.json");
        write_private_json(&run_plan, &pilot).unwrap();
        let mut lifecycle = family_v3_test_lifecycle(&pilot, &run_plan);
        let attestations = with_family_v3_claude_config_preflight(&pilot, |attestations| {
            lifecycle.begin_phase(&pilot, &run_plan, NativePilotRunPhase::Teaching)?;
            Ok(attestations)
        })
        .unwrap();
        let lane = &pilot.lanes[6];
        let expected_config = &attestations[&lane.order];
        let admission_sha256 = unwrap_provider_lane_result(lifecycle.admit_lane(
            &pilot,
            &run_plan,
            lane,
            NativePilotRunPhase::Teaching,
            Some(expected_config),
        ));
        let (admission, observed_admission_sha256) = lifecycle
            .read_validated_lane_admission(&pilot, &run_plan, lane, NativePilotRunPhase::Teaching)
            .unwrap();
        assert_eq!(observed_admission_sha256, admission_sha256);
        let trace = Path::new(&lane.teaching_trace_path);
        let trace_text = [
            serde_json::json!({
                "type": "system",
                "subtype": "init",
                "model": pilot.claude_model,
            })
            .to_string(),
            serde_json::json!({
                "type": "assistant",
                "message": {"model": pilot.claude_model},
            })
            .to_string(),
            serde_json::json!({
                "type": "result",
                "total_cost_usd": 0.0,
            })
            .to_string(),
        ]
        .join("\n")
            + "\n";
        write_private_text(trace, &trace_text).unwrap();
        write_private_text(&stderr_path(trace), "").unwrap();
        let now = unix_ms(SystemTime::now())
            .unwrap()
            .max(admission.admitted_unix_ms);
        let mut execution = NativePilotLaneExecution {
            order: lane.order,
            arm: lane.arm.clone(),
            host: lane.host.clone(),
            trace_path: trace.display().to_string(),
            stderr_path: stderr_path(trace).display().to_string(),
            argv_sha256: admission.argv_sha256.clone(),
            effective_environment_sha256: Some(
                canonical_environment_sha256(&lane.environment).unwrap(),
            ),
            claude_config_artifacts_sha256: Some(expected_config.aggregate_sha256.clone()),
            stdout_trace_bytes: Some(fs::metadata(trace).unwrap().len()),
            stderr_bytes: Some(0),
            process_cleanup_proven: Some(true),
            provider_started_unix_ms: Some(now),
            provider_completed_unix_ms: Some(now),
            exit_code: 0,
            codex_session_rollout_path: None,
            codex_session_rollout_sha256: None,
            codex_model_provider: None,
            codex_host_resolved_model: None,
            codex_agents_md_sha256: None,
            provider_trace_sha256: Some(sha256_file(trace).unwrap()),
            claude_requested_model: Some(pilot.claude_model.clone()),
            claude_host_resolved_model: Some(pilot.claude_model.clone()),
            provider_reported_cost_microusd: Some(0),
            accepted_turn_boundary_budget_exit: false,
            recovered_from_existing_trace: false,
        };
        let terminal_sha256 = lifecycle
            .seal_lane(
                &pilot,
                &run_plan,
                &mut execution,
                NativePilotRunPhase::Teaching,
            )
            .unwrap();
        let terminal_stem =
            lane_lifecycle_stem(NativePilotRunPhase::Teaching, lane.order, "terminal");
        let (terminal, observed_terminal_sha256): (CodexAuthCacheLaneTerminalReceipt, String) =
            lifecycle.directory.read_json_pair(&terminal_stem).unwrap();
        assert_eq!(observed_terminal_sha256, terminal_sha256);
        lifecycle
            .validate_lane_terminal_receipt(
                &pilot,
                &run_plan,
                NativePilotRunPhase::Teaching,
                lane.order,
                &terminal,
            )
            .unwrap();
        fs::write(
            auth_lifecycle_path(&run_plan)
                .unwrap()
                .join(format!("{terminal_stem}.sha256")),
            format!("{}\n", "0".repeat(64)),
        )
        .unwrap();
        let result: EvalResult<(CodexAuthCacheLaneTerminalReceipt, String)> =
            lifecycle.directory.read_json_pair(&terminal_stem);
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("failed digest validation"));
    }

    #[cfg(unix)]
    #[test]
    fn lifecycle_receipts_never_persist_auth_bytes_or_source_path() {
        use std::os::unix::fs::PermissionsExt;

        const CANARY: &str = "SYNTHETIC_SECRET_AUTH_BYTES_7f5489";
        let root = tempfile::tempdir().unwrap();
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let run_plan = root.path().join("run-plan.json");
        fs::write(&run_plan, "{}\n").unwrap();
        let lifecycle = PrivateAuthLifecycleDirectory::create_new(&run_plan).unwrap();
        let (targets, receipts) = synthetic_six_auth_targets(
            root.path(),
            format!("{{\"token\":\"{CANARY}\"}}\n").as_bytes(),
        );
        let boot_id = current_boot_id().unwrap();
        let intent = CodexAuthCacheProvisionIntent {
            schema_version: 1,
            pilot_id: "synthetic".to_string(),
            run_plan_sha256: "a".repeat(64),
            boot_id: boot_id.clone(),
            destinations: targets
                .iter()
                .map(|target| CodexAuthCacheDestinationPlan {
                    lane_order: target.lane_order,
                    auth_json: target.auth_json.display().to_string(),
                })
                .collect(),
            created_unix_ms: 1,
        };
        let intent_sha = lifecycle
            .write_json_pair_exclusive("provision-intent", &intent)
            .unwrap();
        let receipt = CodexAuthCacheProvisionReceipt {
            schema_version: 1,
            pilot_id: "synthetic".to_string(),
            run_plan_sha256: "a".repeat(64),
            provision_intent_sha256: intent_sha,
            boot_id,
            destinations: receipts,
            authentication_ready: true,
            completed_unix_ms: 2,
        };
        lifecycle
            .write_json_pair_exclusive("provision-receipt", &receipt)
            .unwrap();

        let lifecycle_root = root.path().join(CODEX_AUTH_CACHE_LIFECYCLE_DIRECTORY);
        for entry in fs::read_dir(lifecycle_root).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_file() {
                let bytes = fs::read(entry.path()).unwrap();
                assert!(!String::from_utf8_lossy(&bytes).contains(CANARY));
                assert!(!String::from_utf8_lossy(&bytes).contains("source-auth"));
                assert!(!String::from_utf8_lossy(&bytes).contains("source_identity"));
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn lifecycle_creation_receipts_and_locking_are_exclusive() {
        use std::os::unix::fs::PermissionsExt;

        let root = tempfile::tempdir().unwrap();
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let run_plan = root.path().join("run-plan.json");
        fs::write(&run_plan, "{}\n").unwrap();
        let lifecycle = PrivateAuthLifecycleDirectory::create_new(&run_plan).unwrap();
        assert!(PrivateAuthLifecycleDirectory::open(&run_plan)
            .unwrap_err()
            .to_string()
            .contains("exclusive lock"));
        lifecycle
            .write_json_pair_exclusive("provision-intent", &serde_json::json!({"once": true}))
            .unwrap();
        assert!(lifecycle
            .write_json_pair_exclusive("provision-intent", &serde_json::json!({"once": true}))
            .is_err());
        drop(lifecycle);
        assert!(PrivateAuthLifecycleDirectory::create_new(&run_plan)
            .unwrap_err()
            .to_string()
            .contains("non-replayable"));
    }

    #[cfg(unix)]
    #[test]
    fn lifecycle_digest_only_residue_blocks_before_provider_admission() {
        use std::io::Write;
        use std::os::unix::fs::PermissionsExt;

        let root = tempfile::tempdir().unwrap();
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let plan = family_v3_auth_plan(root.path(), 6);
        let targets = exact_six_codex_auth_targets(&plan).unwrap();
        let owner = current_effective_uid().unwrap();
        let destinations = targets
            .iter()
            .map(|target| {
                let home = open_private_codex_home(&target.home, owner).unwrap();
                CodexAuthCacheDestinationReceipt {
                    lane_order: target.lane_order,
                    auth_json: target.auth_json.display().to_string(),
                    identity: create_auth_cache_relative(&home, b"{}\n").unwrap(),
                }
            })
            .collect::<Vec<_>>();
        let run_plan = root.path().join("run-plan.json");
        fs::write(&run_plan, "{}\n").unwrap();
        let directory = PrivateAuthLifecycleDirectory::create_new(&run_plan).unwrap();
        let boot_id = current_boot_id().unwrap();
        let mut lifecycle = CodexAuthCacheLifecycle {
            directory,
            receipt: Some(CodexAuthCacheProvisionReceipt {
                schema_version: CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION,
                pilot_id: plan.pilot_id.clone(),
                run_plan_sha256: sha256_file(&run_plan).unwrap(),
                provision_intent_sha256: "p".repeat(64),
                boot_id,
                destinations,
                authentication_ready: true,
                completed_unix_ms: 1,
            }),
            manifest_sha256: "m".repeat(64),
            active_phase: None,
        };

        let mut phase_digest = create_relative_file(
            &lifecycle.directory.directory,
            "phase-teaching-receipt.sha256",
            0o600,
        )
        .unwrap();
        phase_digest
            .write_all(format!("{}\n", "a".repeat(64)).as_bytes())
            .unwrap();
        phase_digest.sync_all().unwrap();
        lifecycle.directory.directory.sync_all().unwrap();
        assert!(lifecycle
            .begin_phase(&plan, &run_plan, NativePilotRunPhase::Teaching)
            .unwrap_err()
            .to_string()
            .contains("non-replayable"));
        assert!(!lifecycle
            .directory
            .entry_exists("phase-teaching-intent.json")
            .unwrap());

        let mut cleanup_digest = create_relative_file(
            &lifecycle.directory.directory,
            "cleanup-intent.sha256",
            0o600,
        )
        .unwrap();
        cleanup_digest
            .write_all(format!("{}\n", "b".repeat(64)).as_bytes())
            .unwrap();
        cleanup_digest.sync_all().unwrap();
        lifecycle.directory.directory.sync_all().unwrap();
        assert!(lifecycle
            .validate_current_destinations(&plan)
            .unwrap_err()
            .to_string()
            .contains("cleanup has started"));
    }

    #[cfg(unix)]
    #[test]
    fn exact_cleanup_preflights_drift_and_terminal_cleanup_is_complete() {
        use std::os::unix::fs::PermissionsExt;

        let root = tempfile::tempdir().unwrap();
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let owner = current_effective_uid().unwrap();
        let (targets, receipts) = synthetic_six_auth_targets(root.path(), b"{}\n");
        let run_plan = root.path().join("run-plan.json");
        fs::write(&run_plan, "{}\n").unwrap();
        let lifecycle = PrivateAuthLifecycleDirectory::create_new(&run_plan).unwrap();
        let cleanup_intent_sha256 = "a".repeat(64);
        let boot_id = current_boot_id().unwrap();
        let mut removed = Vec::new();
        fs::set_permissions(&targets[4].auth_json, fs::Permissions::from_mode(0o400)).unwrap();
        assert!(delete_exact_destination_receipts(
            &targets,
            &receipts,
            owner,
            &lifecycle,
            &cleanup_intent_sha256,
            &boot_id,
            &mut removed,
        )
        .is_err());
        assert!(removed.is_empty());
        assert!(targets.iter().all(|target| target.auth_json.exists()));

        fs::set_permissions(&targets[4].auth_json, fs::Permissions::from_mode(0o600)).unwrap();
        delete_exact_destination_receipts(
            &targets,
            &receipts,
            owner,
            &lifecycle,
            &cleanup_intent_sha256,
            &boot_id,
            &mut removed,
        )
        .unwrap();
        assert_eq!(removed.len(), 6);
        assert!(targets.iter().all(|target| !target.auth_json.exists()));
        for receipt in &receipts {
            let (lane_intent, lane_intent_sha256): (CodexAuthCacheCleanupLaneIntent, String) =
                lifecycle
                    .read_json_pair(&cleanup_lane_lifecycle_stem(receipt.lane_order, "intent"))
                    .unwrap();
            let (lane_receipt, _): (CodexAuthCacheCleanupLaneReceipt, String) = lifecycle
                .read_json_pair(&cleanup_lane_lifecycle_stem(receipt.lane_order, "receipt"))
                .unwrap();
            assert_eq!(lane_intent.identity, receipt.identity);
            assert_eq!(lane_receipt.cleanup_lane_intent_sha256, lane_intent_sha256);
            assert!(lane_receipt.absent_after_unlink_and_parent_fsync);
        }
    }

    #[cfg(unix)]
    #[test]
    fn partial_cleanup_preserves_exact_removed_paths_and_durable_lane_residue() {
        use std::os::unix::fs::PermissionsExt;

        let root = tempfile::tempdir().unwrap();
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let owner = current_effective_uid().unwrap();
        let (targets, receipts) = synthetic_six_auth_targets(root.path(), b"{}\n");
        let run_plan = root.path().join("run-plan.json");
        fs::write(&run_plan, "{}\n").unwrap();
        let lifecycle = PrivateAuthLifecycleDirectory::create_new(&run_plan).unwrap();
        let cleanup_intent_sha256 = "a".repeat(64);
        let boot_id = current_boot_id().unwrap();
        lifecycle
            .write_json_pair_exclusive(
                &cleanup_lane_lifecycle_stem(2, "intent"),
                &serde_json::json!({"synthetic_collision": true}),
            )
            .unwrap();
        let mut removed = Vec::new();

        assert!(delete_exact_destination_receipts(
            &targets,
            &receipts,
            owner,
            &lifecycle,
            &cleanup_intent_sha256,
            &boot_id,
            &mut removed,
        )
        .is_err());
        assert_eq!(removed, vec![receipts[0].auth_json.clone()]);
        assert_eq!(
            observe_absent_destinations(&targets, owner).unwrap(),
            removed
        );
        assert!(!targets[0].auth_json.exists());
        assert!(targets[1..].iter().all(|target| target.auth_json.exists()));
        assert!(lifecycle
            .entry_exists(&format!(
                "{}.json",
                cleanup_lane_lifecycle_stem(1, "receipt")
            ))
            .unwrap());
        assert!(!lifecycle
            .entry_exists(&format!(
                "{}.json",
                cleanup_lane_lifecycle_stem(2, "receipt")
            ))
            .unwrap());
    }

    #[cfg(unix)]
    fn interrupted_cleanup_fixture() -> (
        tempfile::TempDir,
        PreparedNativePilot,
        PathBuf,
        CodexAuthCacheLifecycle,
        Vec<CodexAuthCacheTarget>,
        String,
        String,
    ) {
        use std::os::unix::fs::PermissionsExt;

        let root = tempfile::tempdir().unwrap();
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let plan = family_v3_auth_plan(root.path(), 6);
        let targets = exact_six_codex_auth_targets(&plan).unwrap();
        let owner = current_effective_uid().unwrap();
        let mut destinations = Vec::new();
        for target in &targets {
            let home = open_private_codex_home(&target.home, owner).unwrap();
            destinations.push(CodexAuthCacheDestinationReceipt {
                lane_order: target.lane_order,
                auth_json: target.auth_json.display().to_string(),
                identity: create_auth_cache_relative(&home, b"{}\n").unwrap(),
            });
        }
        let run_plan = root.path().join("run-plan.json");
        fs::write(&run_plan, "{}\n").unwrap();
        let directory = PrivateAuthLifecycleDirectory::create_new(&run_plan).unwrap();
        let manifest_sha256 = "m".repeat(64);
        let boot_id = current_boot_id().unwrap();
        let provision = CodexAuthCacheProvisionReceipt {
            schema_version: CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION,
            pilot_id: plan.pilot_id.clone(),
            run_plan_sha256: sha256_file(&run_plan).unwrap(),
            provision_intent_sha256: "p".repeat(64),
            boot_id: boot_id.clone(),
            destinations: destinations.clone(),
            authentication_ready: true,
            completed_unix_ms: 1,
        };
        let lifecycle = CodexAuthCacheLifecycle {
            directory,
            receipt: Some(provision),
            manifest_sha256: manifest_sha256.clone(),
            active_phase: None,
        };
        let intent = CodexAuthCacheCleanupIntent {
            schema_version: CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION,
            pilot_id: plan.pilot_id.clone(),
            run_plan_sha256: sha256_file(&run_plan).unwrap(),
            lifecycle_manifest_sha256: manifest_sha256,
            run_root: root.path().display().to_string(),
            boot_id: boot_id.clone(),
            destinations,
            authority: CODEX_AUTH_CACHE_CLEANUP_AUTHORITY.to_string(),
            created_unix_ms: 2,
        };
        let cleanup_intent_sha256 = lifecycle
            .directory
            .write_json_pair_exclusive("cleanup-intent", &intent)
            .unwrap();
        (
            root,
            plan,
            run_plan,
            lifecycle,
            targets,
            cleanup_intent_sha256,
            boot_id,
        )
    }

    #[cfg(unix)]
    fn write_cleanup_lane_intent_for_test(
        lifecycle: &CodexAuthCacheLifecycle,
        destination: &CodexAuthCacheDestinationReceipt,
        cleanup_intent_sha256: &str,
        boot_id: &str,
    ) -> String {
        lifecycle
            .directory
            .write_json_pair_exclusive(
                &cleanup_lane_lifecycle_stem(destination.lane_order, "intent"),
                &CodexAuthCacheCleanupLaneIntent {
                    schema_version: CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION,
                    cleanup_intent_sha256: cleanup_intent_sha256.to_string(),
                    boot_id: boot_id.to_string(),
                    lane_order: destination.lane_order,
                    auth_json: destination.auth_json.clone(),
                    identity: destination.identity.clone(),
                    authority: CODEX_AUTH_CACHE_CLEANUP_AUTHORITY.to_string(),
                    created_unix_ms: 3,
                },
            )
            .unwrap()
    }

    #[cfg(unix)]
    #[test]
    fn interrupted_cleanup_crash_points_seal_inspection_only_unresolved_receipts() {
        for crash_point in [
            "after_global_intent",
            "after_lane_intent",
            "after_unlink_before_lane_receipt",
            "after_some_lane_receipts",
        ] {
            let (root, plan, run_plan, mut lifecycle, targets, cleanup_intent_sha256, boot_id) =
                interrupted_cleanup_fixture();
            let destinations = lifecycle.receipt().unwrap().destinations.clone();
            if crash_point != "after_global_intent" {
                let lane_intent_sha256 = write_cleanup_lane_intent_for_test(
                    &lifecycle,
                    &destinations[0],
                    &cleanup_intent_sha256,
                    &boot_id,
                );
                if matches!(
                    crash_point,
                    "after_unlink_before_lane_receipt" | "after_some_lane_receipts"
                ) {
                    let home =
                        open_private_codex_home(&targets[0].home, current_effective_uid().unwrap())
                            .unwrap();
                    unlink_auth_cache_relative(&home).unwrap();
                }
                if crash_point == "after_some_lane_receipts" {
                    lifecycle
                        .directory
                        .write_json_pair_exclusive(
                            &cleanup_lane_lifecycle_stem(1, "receipt"),
                            &CodexAuthCacheCleanupLaneReceipt {
                                schema_version: CODEX_AUTH_CACHE_LIFECYCLE_SCHEMA_VERSION,
                                cleanup_intent_sha256: cleanup_intent_sha256.clone(),
                                cleanup_lane_intent_sha256: lane_intent_sha256,
                                boot_id: boot_id.clone(),
                                lane_order: destinations[0].lane_order,
                                auth_json: destinations[0].auth_json.clone(),
                                identity: destinations[0].identity.clone(),
                                absent_after_unlink_and_parent_fsync: true,
                                completed_unix_ms: 4,
                            },
                        )
                        .unwrap();
                    write_cleanup_lane_intent_for_test(
                        &lifecycle,
                        &destinations[1],
                        &cleanup_intent_sha256,
                        &boot_id,
                    );
                    let home =
                        open_private_codex_home(&targets[1].home, current_effective_uid().unwrap())
                            .unwrap();
                    unlink_auth_cache_relative(&home).unwrap();
                }
            }

            let report = lifecycle.cleanup(&plan, &run_plan, root.path()).unwrap();
            assert_eq!(report.state, NativePilotAuthCacheCleanupState::Unresolved);
            let expected_absent = match crash_point {
                "after_global_intent" | "after_lane_intent" => 0,
                "after_unlink_before_lane_receipt" => 1,
                "after_some_lane_receipts" => 2,
                _ => unreachable!(),
            };
            assert_eq!(report.removed_destinations.len(), expected_absent);
            assert!(report
                .unresolved_reason
                .as_deref()
                .unwrap()
                .contains("interrupted_cleanup_inspection_only"));
            assert_eq!(
                targets
                    .iter()
                    .filter(|target| !target.auth_json.exists())
                    .count(),
                expected_absent
            );
            let (persisted, _): (NativePilotAuthCacheCleanupReport, String) = lifecycle
                .directory
                .read_json_pair("cleanup-receipt")
                .unwrap();
            assert_eq!(persisted, report);
            assert_eq!(
                lifecycle.cleanup(&plan, &run_plan, root.path()).unwrap(),
                report
            );
        }
    }

    #[test]
    fn lifecycle_boot_binding_rejects_mismatch() {
        let first = "11111111-1111-1111-1111-111111111111";
        let second = "22222222-2222-2222-2222-222222222222";
        require_same_boot(first, first).unwrap();
        assert!(require_same_boot(first, second).is_err());
        assert!(normalize_boot_id("00000000-0000-0000-0000-000000000000").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn unresolved_cleanup_receipt_is_durable_and_non_overwritable() {
        use std::os::unix::fs::PermissionsExt;

        let root = tempfile::tempdir().unwrap();
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let run_plan = root.path().join("run-plan.json");
        fs::write(&run_plan, "{}\n").unwrap();
        let lifecycle = PrivateAuthLifecycleDirectory::create_new(&run_plan).unwrap();
        let report = NativePilotAuthCacheCleanupReport {
            schema_version: 1,
            pilot_id: "synthetic".to_string(),
            run_plan: run_plan.display().to_string(),
            run_root: root.path().display().to_string(),
            lifecycle_manifest_sha256: "a".repeat(64),
            cleanup_intent_sha256: "b".repeat(64),
            boot_id: current_boot_id().unwrap(),
            state: NativePilotAuthCacheCleanupState::Unresolved,
            destination_count: 6,
            removed_destinations: Vec::new(),
            unresolved_reason: Some("boot_identity_changed".to_string()),
            completed_unix_ms: 1,
        };
        lifecycle
            .write_json_pair_exclusive("cleanup-receipt", &report)
            .unwrap();
        let (read_back, _): (NativePilotAuthCacheCleanupReport, String) =
            lifecycle.read_json_pair("cleanup-receipt").unwrap();
        assert_eq!(read_back, report);
        assert!(lifecycle
            .write_json_pair_exclusive("cleanup-receipt", &report)
            .is_err());
    }

    #[cfg(unix)]
    #[test]
    fn bounded_auth_status_enforces_timeout_output_and_sanitized_environment() {
        let root = tempfile::tempdir().unwrap();
        let success = root.path().join("success");
        write_test_executable(
            &success,
            "#!/bin/sh\nif [ -n \"$OPENAI_API_KEY\" ]; then exit 91; fi\nprintf 'Logged in using ChatGPT\\n'\n",
        );
        let environment = BTreeMap::from([
            ("OPENAI_API_KEY".to_string(), "SYNTHETIC_SECRET".to_string()),
            (
                "CODEX_ACCESS_TOKEN".to_string(),
                "SYNTHETIC_SECRET".to_string(),
            ),
        ]);
        let output = run_bounded_codex_auth_status(
            success.to_str().unwrap(),
            &[],
            root.path(),
            &environment,
            Duration::from_secs(5),
            256,
        )
        .unwrap();
        assert!(output.success);
        assert_eq!(output.stdout.trim(), CODEX_CHATGPT_LOGIN_STATUS);

        let timeout = root.path().join("timeout");
        write_test_executable(&timeout, "#!/bin/sh\n/bin/sleep 2\n");
        assert!(run_bounded_codex_auth_status(
            timeout.to_str().unwrap(),
            &[],
            root.path(),
            &environment,
            Duration::from_millis(50),
            256,
        )
        .is_err());

        let overflow = root.path().join("overflow");
        write_test_executable(
            &overflow,
            "#!/bin/sh\ni=0; while [ $i -lt 300 ]; do printf x; i=$((i+1)); done\n",
        );
        assert!(run_bounded_codex_auth_status(
            overflow.to_str().unwrap(),
            &[],
            root.path(),
            &environment,
            Duration::from_secs(1),
            128,
        )
        .is_err());
    }

    #[cfg(unix)]
    #[test]
    fn codex_rollout_attestation_binds_unique_trace_and_host_resolved_model() {
        use std::os::unix::fs::PermissionsExt;

        let root = tempfile::tempdir().unwrap();
        let fixture_instructions = "# Synthetic fixture instructions\n";
        fs::create_dir(root.path().join(".git")).unwrap();
        fs::write(root.path().join("AGENTS.md"), fixture_instructions).unwrap();
        let codex_home = root.path().join("codex-home");
        let sessions = codex_home.join("sessions/2026/09/10");
        for directory in [
            codex_home.clone(),
            codex_home.join("sessions"),
            codex_home.join("sessions/2026"),
            codex_home.join("sessions/2026/09"),
            sessions.clone(),
        ] {
            create_private_dir_all(&directory).unwrap();
        }
        let mut lane = receipt_test_lane(root.path(), 1, "codex");
        lane.evaluation_cwd = root.path().canonicalize().unwrap().display().to_string();
        lane.environment
            .insert("CODEX_HOME".to_string(), codex_home.display().to_string());
        let thread_id = "11111111-2222-7333-8444-555555555555";
        let trace = root.path().join("provider.jsonl");
        fs::write(
            &trace,
            format!(
                "{}\n",
                serde_json::json!({"type": "thread.started", "thread_id": thread_id})
            ),
        )
        .unwrap();
        let before = capture_codex_pre_dispatch_inventory(&lane).unwrap();
        let rollout = sessions.join(format!("rollout-2026-09-10T00-00-00-{thread_id}.jsonl"));
        let agents_md = serde_json::json!({
            "directory": root.path().canonicalize().unwrap().display().to_string(),
            "text": format!("# Global prelude\n{fixture_instructions}"),
        });
        let rollout_bytes = format!(
            "{}\n{}\n{}\n{}\n",
            serde_json::json!({
                "type": "session_meta",
                "payload": {
                    "id": thread_id,
                    "session_id": thread_id,
                    "model_provider": "openai"
                }
            }),
            serde_json::json!({
                "type": "world_state",
                "payload": {
                    "full": true,
                    "state": {"model": "gpt-5.6-sol", "agents_md": agents_md.clone()}
                }
            }),
            serde_json::json!({
                "type": "turn_context",
                "payload": {"model": "gpt-5.6-sol"}
            }),
            serde_json::json!({
                "type": "world_state",
                "payload": {"full": false, "state": {"plugins_instructions": "delta"}}
            })
        );
        fs::write(&rollout, rollout_bytes.as_bytes()).unwrap();
        fs::set_permissions(&rollout, fs::Permissions::from_mode(0o600)).unwrap();

        let attestation =
            attest_codex_session_rollout(&lane, NativePilotRunPhase::Evaluation, &trace, &before)
                .unwrap();
        assert_eq!(
            attestation.path,
            rollout.canonicalize().unwrap().display().to_string()
        );
        assert_eq!(attestation.sha256, sha256_bytes(rollout_bytes.as_bytes()));
        assert_eq!(attestation.model_provider, "openai");
        assert_eq!(attestation.host_resolved_model, "gpt-5.6-sol");
        validate_recorded_codex_session_rollout(
            &lane,
            NativePilotRunPhase::Evaluation,
            &trace,
            &attestation,
        )
        .unwrap();

        let replayed_before = capture_codex_pre_dispatch_inventory(&lane).unwrap();
        assert!(attest_codex_session_rollout(
            &lane,
            NativePilotRunPhase::Evaluation,
            &trace,
            &replayed_before,
        )
        .unwrap_err()
        .to_string()
        .contains("exactly one new session rollout path; found 0"));

        let mismatched = format!(
            "{}\n{}\n{}\n",
            serde_json::json!({
                "type": "session_meta",
                "payload": {"id": thread_id, "model_provider": "openai"}
            }),
            serde_json::json!({
                "type": "world_state",
                "payload": {"state": {"model": "gpt-5.6-sol", "agents_md": agents_md.clone()}}
            }),
            serde_json::json!({
                "type": "turn_context",
                "payload": {"model": "gpt-5.6-terra"}
            })
        );
        fs::write(&rollout, mismatched).unwrap();
        assert!(attest_codex_session_rollout(
            &lane,
            NativePilotRunPhase::Evaluation,
            &trace,
            &before,
        )
        .unwrap_err()
        .to_string()
        .contains("concordant host-resolved model"));

        fs::write(&rollout, rollout_bytes.as_bytes()).unwrap();
        fs::set_permissions(&rollout, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(attest_codex_session_rollout(
            &lane,
            NativePilotRunPhase::Evaluation,
            &trace,
            &before,
        )
        .unwrap_err()
        .to_string()
        .contains("private bounded identity"));

        let missing_turn_model = format!(
            "{}\n{}\n{}\n",
            serde_json::json!({
                "type": "session_meta",
                "payload": {"id": thread_id, "model_provider": "openai"}
            }),
            serde_json::json!({
                "type": "world_state",
                "payload": {"state": {"model": "gpt-5.6-sol", "agents_md": agents_md.clone()}}
            }),
            serde_json::json!({"type": "turn_context", "payload": {}})
        );
        fs::write(&rollout, missing_turn_model).unwrap();
        fs::set_permissions(&rollout, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(attest_codex_session_rollout(
            &lane,
            NativePilotRunPhase::Evaluation,
            &trace,
            &before,
        )
        .unwrap_err()
        .to_string()
        .contains("turn_context record omitted its model"));

        let missing_full_model = format!(
            "{}\n{}\n{}\n",
            serde_json::json!({
                "type": "session_meta",
                "payload": {"id": thread_id, "model_provider": "openai"}
            }),
            serde_json::json!({
                "type": "world_state",
                "payload": {"full": true, "state": {"agents_md": agents_md.clone()}}
            }),
            serde_json::json!({
                "type": "turn_context",
                "payload": {"model": "gpt-5.6-sol"}
            })
        );
        fs::write(&rollout, missing_full_model).unwrap();
        assert!(attest_codex_session_rollout(
            &lane,
            NativePilotRunPhase::Evaluation,
            &trace,
            &before,
        )
        .unwrap_err()
        .to_string()
        .contains("world_state record omitted its model"));

        let malformed_full_flag = format!(
            "{}\n{}\n{}\n",
            serde_json::json!({
                "type": "session_meta",
                "payload": {"id": thread_id, "model_provider": "openai"}
            }),
            serde_json::json!({
                "type": "world_state",
                "payload": {
                    "full": "false",
                    "state": {"model": "gpt-5.6-sol", "agents_md": agents_md.clone()}
                }
            }),
            serde_json::json!({
                "type": "turn_context",
                "payload": {"model": "gpt-5.6-sol"}
            })
        );
        fs::write(&rollout, malformed_full_flag).unwrap();
        assert!(attest_codex_session_rollout(
            &lane,
            NativePilotRunPhase::Evaluation,
            &trace,
            &before,
        )
        .unwrap_err()
        .to_string()
        .contains("world_state full flag is not boolean"));

        let conflicting_delta_model = format!(
            "{}\n{}\n{}\n{}\n",
            serde_json::json!({
                "type": "session_meta",
                "payload": {"id": thread_id, "model_provider": "openai"}
            }),
            serde_json::json!({
                "type": "world_state",
                "payload": {
                    "full": true,
                    "state": {"model": "gpt-5.6-sol", "agents_md": agents_md.clone()}
                }
            }),
            serde_json::json!({
                "type": "world_state",
                "payload": {"full": false, "state": {"model": "gpt-5.6-terra"}}
            }),
            serde_json::json!({
                "type": "turn_context",
                "payload": {"model": "gpt-5.6-sol"}
            })
        );
        fs::write(&rollout, conflicting_delta_model).unwrap();
        assert!(attest_codex_session_rollout(
            &lane,
            NativePilotRunPhase::Evaluation,
            &trace,
            &before,
        )
        .unwrap_err()
        .to_string()
        .contains("concordant host-resolved model"));

        let matching_delta_model = format!(
            "{}\n{}\n{}\n{}\n",
            serde_json::json!({
                "type": "session_meta",
                "payload": {"id": thread_id, "model_provider": "openai"}
            }),
            serde_json::json!({
                "type": "world_state",
                "payload": {
                    "full": true,
                    "state": {"model": "gpt-5.6-sol", "agents_md": agents_md.clone()}
                }
            }),
            serde_json::json!({
                "type": "world_state",
                "payload": {"full": false, "state": {"model": "gpt-5.6-sol"}}
            }),
            serde_json::json!({
                "type": "turn_context",
                "payload": {"model": "gpt-5.6-sol"}
            })
        );
        fs::write(&rollout, matching_delta_model).unwrap();
        assert_eq!(
            attest_codex_session_rollout(&lane, NativePilotRunPhase::Evaluation, &trace, &before,)
                .unwrap()
                .host_resolved_model,
            "gpt-5.6-sol"
        );

        let malformed_delta_model = format!(
            "{}\n{}\n{}\n{}\n",
            serde_json::json!({
                "type": "session_meta",
                "payload": {"id": thread_id, "model_provider": "openai"}
            }),
            serde_json::json!({
                "type": "world_state",
                "payload": {
                    "full": true,
                    "state": {"model": "gpt-5.6-sol", "agents_md": agents_md.clone()}
                }
            }),
            serde_json::json!({
                "type": "world_state",
                "payload": {"full": false, "state": {"model": null}}
            }),
            serde_json::json!({
                "type": "turn_context",
                "payload": {"model": "gpt-5.6-sol"}
            })
        );
        fs::write(&rollout, malformed_delta_model).unwrap();
        assert!(attest_codex_session_rollout(
            &lane,
            NativePilotRunPhase::Evaluation,
            &trace,
            &before,
        )
        .unwrap_err()
        .to_string()
        .contains("world_state model field is not a string"));

        let malformed_collaboration_model = format!(
            "{}\n{}\n{}\n",
            serde_json::json!({
                "type": "session_meta",
                "payload": {"id": thread_id, "model_provider": "openai"}
            }),
            serde_json::json!({
                "type": "world_state",
                "payload": {
                    "full": true,
                    "state": {
                        "model": "gpt-5.6-sol",
                        "collaboration_mode": {"model": {}},
                        "agents_md": agents_md.clone()
                    }
                }
            }),
            serde_json::json!({
                "type": "turn_context",
                "payload": {"model": "gpt-5.6-sol"}
            })
        );
        fs::write(&rollout, malformed_collaboration_model).unwrap();
        assert!(attest_codex_session_rollout(
            &lane,
            NativePilotRunPhase::Evaluation,
            &trace,
            &before,
        )
        .unwrap_err()
        .to_string()
        .contains("world_state model field is not a string"));

        let later_full_without_agents = format!(
            "{}\n{}\n{}\n{}\n",
            serde_json::json!({
                "type": "session_meta",
                "payload": {"id": thread_id, "model_provider": "openai"}
            }),
            serde_json::json!({
                "type": "world_state",
                "payload": {
                    "full": true,
                    "state": {"model": "gpt-5.6-sol", "agents_md": agents_md.clone()}
                }
            }),
            serde_json::json!({
                "type": "world_state",
                "payload": {"full": true, "state": {"model": "gpt-5.6-sol"}}
            }),
            serde_json::json!({
                "type": "turn_context",
                "payload": {"model": "gpt-5.6-sol"}
            })
        );
        fs::write(&rollout, later_full_without_agents).unwrap();
        assert!(attest_codex_session_rollout(
            &lane,
            NativePilotRunPhase::Evaluation,
            &trace,
            &before,
        )
        .unwrap_err()
        .to_string()
        .contains("full world_state omitted agents_md"));

        let divergent_delta_agents = format!(
            "{}\n{}\n{}\n{}\n",
            serde_json::json!({
                "type": "session_meta",
                "payload": {"id": thread_id, "model_provider": "openai"}
            }),
            serde_json::json!({
                "type": "world_state",
                "payload": {
                    "full": true,
                    "state": {"model": "gpt-5.6-sol", "agents_md": agents_md.clone()}
                }
            }),
            serde_json::json!({
                "type": "world_state",
                "payload": {
                    "full": false,
                    "state": {
                        "agents_md": {
                            "directory": root.path().canonicalize().unwrap().display().to_string(),
                            "text": format!("# Different global prelude\n{fixture_instructions}")
                        }
                    }
                }
            }),
            serde_json::json!({
                "type": "turn_context",
                "payload": {"model": "gpt-5.6-sol"}
            })
        );
        fs::write(&rollout, divergent_delta_agents).unwrap();
        assert!(attest_codex_session_rollout(
            &lane,
            NativePilotRunPhase::Evaluation,
            &trace,
            &before,
        )
        .unwrap_err()
        .to_string()
        .contains("concordant host-resolved model"));
    }

    #[cfg(unix)]
    #[test]
    fn codex_rollout_attestation_rejects_ambiguous_session_files() {
        use std::os::unix::fs::PermissionsExt;

        let root = tempfile::tempdir().unwrap();
        let codex_home = root.path().join("codex-home");
        let sessions = codex_home.join("sessions");
        for directory in [
            codex_home.clone(),
            sessions.clone(),
            sessions.join("first"),
            sessions.join("second"),
        ] {
            create_private_dir_all(&directory).unwrap();
        }
        let mut lane = receipt_test_lane(root.path(), 1, "codex");
        lane.environment
            .insert("CODEX_HOME".to_string(), codex_home.display().to_string());
        let thread_id = "aaaaaaaa-bbbb-7ccc-8ddd-eeeeeeeeeeee";
        let trace = root.path().join("provider.jsonl");
        fs::write(
            &trace,
            format!(
                "{}\n",
                serde_json::json!({"type": "thread.started", "thread_id": thread_id})
            ),
        )
        .unwrap();
        let before = capture_codex_pre_dispatch_inventory(&lane).unwrap();
        for directory in ["first", "second"] {
            let rollout = sessions
                .join(directory)
                .join(format!("rollout-{thread_id}.jsonl"));
            fs::write(&rollout, "{}\n").unwrap();
            fs::set_permissions(&rollout, fs::Permissions::from_mode(0o600)).unwrap();
        }

        assert!(attest_codex_session_rollout(
            &lane,
            NativePilotRunPhase::Evaluation,
            &trace,
            &before,
        )
        .unwrap_err()
        .to_string()
        .contains("exactly one new session rollout path; found 2"));
    }

    #[cfg(unix)]
    #[test]
    fn codex_file_cache_requires_private_regular_json() {
        use std::os::unix::fs::PermissionsExt;

        let root = tempfile::tempdir().unwrap();
        let codex_home = root.path().join("codex-home");
        fs::create_dir(&codex_home).unwrap();
        fs::set_permissions(&codex_home, fs::Permissions::from_mode(0o700)).unwrap();

        assert!(!validate_codex_file_cache(&codex_home).unwrap());

        let auth_path = codex_home.join("auth.json");
        fs::write(&auth_path, "{}\n").unwrap();
        fs::set_permissions(&auth_path, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(validate_codex_file_cache(&codex_home).unwrap());

        fs::set_permissions(&auth_path, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(validate_codex_file_cache(&codex_home)
            .unwrap_err()
            .to_string()
            .contains("mode 0600"));

        fs::set_permissions(&auth_path, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&auth_path, "[]\n").unwrap();
        assert!(validate_codex_file_cache(&codex_home)
            .unwrap_err()
            .to_string()
            .contains("JSON object"));
    }

    #[test]
    fn auth_cache_provisioning_requires_explicit_confirmation_before_file_access() {
        let error = provision_native_memory_pilot_auth_cache(
            Path::new("/does/not/exist/run-plan.json"),
            Path::new("/does/not/exist/auth.json"),
            false,
        )
        .unwrap_err();

        assert!(error
            .to_string()
            .contains("--confirm-plaintext-cache-copies"));
    }

    #[cfg(unix)]
    #[test]
    fn auth_cache_provisioning_completes_one_frozen_canary_transaction() {
        use std::os::unix::fs::PermissionsExt;

        const CANARY: &str = "SYNTHETIC_AUTH_CACHE_CANARY";
        let root = tempfile::tempdir().unwrap();
        let bin = root.path().join("bin");
        let run = root.path().join("run");
        let lane_dir = run.join("lane-1");
        let codex_home = lane_dir.join("codex-home");
        let teaching_cwd = root.path().join("teaching-checkout");
        let evaluation_cwd = root.path().join("evaluation-checkout");
        for path in [
            &bin,
            &run,
            &lane_dir,
            &codex_home,
            &teaching_cwd,
            &evaluation_cwd,
        ] {
            create_private_dir_all(path).unwrap();
        }

        let engram = bin.join("engram");
        write_test_executable(
            &engram,
            &format!(
                r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "engram-test"
  exit 0
fi
path="$(cd "$(dirname "$0")" && pwd -P)/$(basename "$0")"
sha="$(shasum -a 256 "$path" | awk '{{print $1}}')"
printf '%s\n' "{{\"schema_version\":3,\"cli_version\":\"{}\",\"health_schema_version\":1,\"mcp_contract_version\":1,\"mcp_protocol_version\":\"test\",\"profile\":\"agent\",\"mcp_tool_count\":6,\"mcp_tools_sha256\":\"{}\",\"profile_instructions_sha256\":\"{}\",\"effective_runtime\":{{\"verified\":true,\"mcp_tool_count\":6,\"mcp_tools_sha256\":\"{}\",\"profile_instructions_sha256\":\"{}\",\"restricted_tool_rejected\":true,\"review_authority_rejected\":true}},\"executable_path\":\"$path\",\"executable_sha256\":\"$sha\"}}"
"#,
                env!("CARGO_PKG_VERSION"),
                "a".repeat(64),
                "b".repeat(64),
                "a".repeat(64),
                "b".repeat(64),
            ),
        );
        let codex = bin.join("codex");
        write_test_executable(
            &codex,
            r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "codex-test"
  exit 0
fi
last=""
for arg in "$@"; do last="$arg"; done
if [ "$last" = "status" ]; then
  echo "Logged in using ChatGPT"
  exit 0
fi
exit 64
"#,
        );
        let codex_code_mode_host = bin.join("codex-code-mode-host");
        write_test_executable(
            &codex_code_mode_host,
            r#"#!/bin/sh
if [ "$1" = "--help" ]; then
  echo "Usage: codex-code-mode-host [OPTIONS]"
  exit 0
fi
exit 64
"#,
        );
        let claude = bin.join("claude");
        write_test_executable(
            &claude,
            r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "claude-test"
  exit 0
fi
exit 64
"#,
        );

        let engram_attestation = attest_binary(&engram).unwrap();
        let codex_attestation = attest_binary(&codex).unwrap();
        let codex_code_mode_host_attestation =
            attest_codex_code_mode_host(&codex_attestation).unwrap();
        let claude_attestation = attest_binary(&claude).unwrap();
        let engram_contract = attest_engram_mcp_contract(&engram_attestation).unwrap();
        let codex_path = codex_attestation.path.clone();
        let provider_argv = vec![
            codex_path.clone(),
            "--config".to_string(),
            CODEX_FILE_CONFIG.to_string(),
            "exec".to_string(),
        ];
        let login_argv = vec![
            codex_path.clone(),
            "login".to_string(),
            "--config".to_string(),
            CODEX_FILE_CONFIG.to_string(),
        ];
        let mut status_argv = login_argv.clone();
        status_argv.push("status".to_string());

        let acceptance_contract = lane_dir.join("acceptance-contract.json");
        write_private_json(
            &acceptance_contract,
            &serde_json::json!({
                "case_id": "canary-case",
                "expected_repository_remote": "https://github.com/example/canary",
                "expected_project": null,
                "expected_component": null,
                "expected_first_action": "abstain",
                "required_command": "true",
                "expected_exit_code": 0,
                "expected_output_contains": "unused",
                "evidence_path": lane_dir.join("unused-evidence.json"),
                "evidence_sha256": "d".repeat(64),
            }),
        )
        .unwrap();

        let lane = PreparedNativeLane {
            order: 1,
            case_id: "canary-case".to_string(),
            arm: "codex_native_memory".to_string(),
            host: "codex".to_string(),
            memory_layer: MemoryLayer::Native,
            repetition: 1,
            fixture_revision: "canary-fixture".to_string(),
            teaching_cwd: teaching_cwd.display().to_string(),
            evaluation_cwd: evaluation_cwd.display().to_string(),
            environment: BTreeMap::from([(
                "CODEX_HOME".to_string(),
                codex_home.display().to_string(),
            )]),
            required_secret_environment: Vec::new(),
            codex_authentication: Some(PreparedCodexAuthentication {
                mode: CodexAuthenticationMode::ChatgptFileCache,
                credential_store: "file".to_string(),
                login_argv,
                status_argv,
                expected_status: CODEX_CHATGPT_LOGIN_STATUS.to_string(),
            }),
            claude_teaching_bash_commands: Vec::new(),
            teaching_argv: provider_argv.clone(),
            activation_argv: Some(provider_argv.clone()),
            post_teaching_verification_argv: None,
            activation_wait_hours: 1,
            evaluation_argv: provider_argv,
            artifact_gates: Vec::new(),
            acceptance_contract: acceptance_contract.display().to_string(),
            adapter_sha256: None,
            engram_home: None,
            engram_project: None,
            cleanup_argv: None,
            teaching_trace_path: lane_dir.join("teaching.jsonl").display().to_string(),
            activation_trace_path: Some(lane_dir.join("activation.jsonl").display().to_string()),
            evaluation_trace_path: lane_dir.join("evaluation.jsonl").display().to_string(),
            post_teaching_verification_output_path: None,
            agent_output_path: lane_dir.join("agent-output.json").display().to_string(),
        };
        let mut pilot = plan(0);
        pilot.claude_model.clear();
        pilot.claude_max_turns = 0;
        pilot.binaries = BTreeMap::from([
            ("engram".to_string(), engram_attestation),
            ("codex".to_string(), codex_attestation),
            (
                "codex_code_mode_host".to_string(),
                codex_code_mode_host_attestation,
            ),
            ("claude_code".to_string(), claude_attestation),
        ]);
        pilot.engram_mcp_contract = engram_contract;
        pilot.lanes = vec![lane];
        let mut missing_companion = pilot.clone();
        missing_companion
            .binaries
            .remove("codex_code_mode_host")
            .unwrap();
        assert!(validate_codex_authentication_contract(&missing_companion)
            .unwrap_err()
            .to_string()
            .contains("must attest the sibling codex_code_mode_host"));
        let run_plan = run.join("run-plan.json");
        write_private_json(&run_plan, &pilot).unwrap();
        write_private_text(
            &run.join("run-plan.sha256"),
            &format!("{}\n", sha256_file(&run_plan).unwrap()),
        )
        .unwrap();

        let source = root.path().join("source-auth.json");
        let source_contents = format!("{{\"token\":\"{CANARY}\"}}\n");
        fs::write(&source, &source_contents).unwrap();
        fs::set_permissions(&source, fs::Permissions::from_mode(0o600)).unwrap();

        let report = provision_native_memory_pilot_auth_cache(&run_plan, &source, true).unwrap();
        assert!(report.ready);
        assert_eq!(report.lanes.len(), 1);
        assert_eq!(report.lanes[0].state, NativePilotCodexAuthState::Ready);
        assert!(!serde_json::to_string(&report).unwrap().contains(CANARY));

        let destination = codex_home.join("auth.json");
        assert_eq!(fs::read_to_string(&destination).unwrap(), source_contents);
        assert_eq!(
            fs::symlink_metadata(&destination)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        let error = provision_native_memory_pilot_auth_cache(&run_plan, &source, true).unwrap_err();
        assert!(error.to_string().contains("refusing to overwrite"));
        assert!(!error.to_string().contains(CANARY));
        assert_eq!(fs::read_to_string(destination).unwrap(), source_contents);
    }

    #[cfg(unix)]
    #[test]
    fn private_auth_cache_copy_never_overwrites_or_discloses_contents() {
        use std::os::unix::fs::PermissionsExt;

        const CANARY: &str = "AUTH_CACHE_SECRET_CANARY";
        let root = tempfile::tempdir().unwrap();
        let codex_home = root.path().join("codex-home");
        fs::create_dir(&codex_home).unwrap();
        fs::set_permissions(&codex_home, fs::Permissions::from_mode(0o700)).unwrap();
        let destination = codex_home.join("auth.json");
        let contents = format!("{{\"token\":\"{CANARY}\"}}\n");

        write_private_codex_auth_cache(&destination, contents.as_bytes()).unwrap();
        assert_eq!(fs::read_to_string(&destination).unwrap(), contents);
        assert_eq!(
            fs::symlink_metadata(&destination)
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        assert!(validate_codex_file_cache(&codex_home).unwrap());

        let error = write_private_codex_auth_cache(&destination, contents.as_bytes()).unwrap_err();
        assert!(!error.to_string().contains(CANARY));
        assert_eq!(fs::read_to_string(&destination).unwrap(), contents);

        let invalid_destination = codex_home.join("invalid-auth.json");
        let error =
            write_private_codex_auth_cache(&invalid_destination, CANARY.as_bytes()).unwrap_err();
        assert!(!error.to_string().contains(CANARY));
        assert!(!invalid_destination.exists());
    }

    #[cfg(unix)]
    #[test]
    fn source_auth_cache_rejects_hard_links_and_sanitizes_json_errors() {
        use std::os::unix::fs::PermissionsExt;

        const CANARY: &str = "INVALID_AUTH_CACHE_SECRET_CANARY";
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("auth.json");
        fs::write(&source, "{}\n").unwrap();
        fs::set_permissions(&source, fs::Permissions::from_mode(0o600)).unwrap();
        let linked = root.path().join("linked-auth.json");
        fs::hard_link(&source, &linked).unwrap();
        assert!(validate_codex_auth_cache_file(&source, None)
            .unwrap_err()
            .to_string()
            .contains("exactly one hard link"));

        let invalid_source = root.path().join("invalid-auth.json");
        fs::write(&invalid_source, CANARY).unwrap();
        fs::set_permissions(&invalid_source, fs::Permissions::from_mode(0o600)).unwrap();
        let error = validate_codex_auth_cache_file(&invalid_source, None).unwrap_err();
        assert!(!error.to_string().contains(CANARY));
    }

    #[cfg(unix)]
    #[test]
    fn source_auth_cache_reads_the_validated_handle_without_following_symlinks() {
        use std::os::unix::fs::{symlink, PermissionsExt};

        const ORIGINAL: &str = "{\"token\":\"ORIGINAL_AUTH_CACHE_CANARY\"}\n";
        const REPLACEMENT: &str = "{\"token\":\"REPLACEMENT_AUTH_CACHE_CANARY\"}\n";
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("auth.json");
        fs::write(&source, ORIGINAL).unwrap();
        fs::set_permissions(&source, fs::Permissions::from_mode(0o600)).unwrap();

        let file = open_validated_codex_auth_cache(&source, None).unwrap();
        let moved = root.path().join("moved-auth.json");
        fs::rename(&source, &moved).unwrap();
        fs::write(&source, REPLACEMENT).unwrap();
        fs::set_permissions(&source, fs::Permissions::from_mode(0o600)).unwrap();

        let contents = read_validated_codex_auth_cache(file, &source).unwrap();
        assert_eq!(contents.as_slice(), ORIGINAL.as_bytes());

        let linked = root.path().join("symlinked-auth.json");
        symlink(&source, &linked).unwrap();
        let error = open_validated_codex_auth_cache(&linked, None).unwrap_err();
        assert!(!error.to_string().contains(REPLACEMENT));
    }

    #[cfg(unix)]
    #[test]
    fn source_auth_cache_read_remains_bounded_after_open() {
        use std::os::unix::fs::PermissionsExt;

        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("auth.json");
        fs::write(&source, "{}\n").unwrap();
        fs::set_permissions(&source, fs::Permissions::from_mode(0o600)).unwrap();

        let file = open_validated_codex_auth_cache(&source, None).unwrap();
        OpenOptions::new()
            .write(true)
            .open(&source)
            .unwrap()
            .set_len(MAX_CODEX_AUTH_FILE_BYTES + 1)
            .unwrap();

        let error = read_validated_codex_auth_cache(file, &source).unwrap_err();
        assert!(error.to_string().contains("at most"));
    }

    #[test]
    fn extracts_one_distinct_agent_output_from_nested_jsonl() {
        let root = tempfile::tempdir().unwrap();
        let trace = root.path().join("trace.jsonl");
        let output = root.path().join("output.json");
        let result = serde_json::json!({
            "answer": "done",
            "repository_remote": "https://github.com/acme/atlas",
            "project": "atlas",
            "component": "worker",
            "first_action": "run_verified_procedure",
            "returned_context_keys": ["procedure.atlas"],
            "applied_context_keys": ["procedure.atlas"],
            "evidence_targets": ["/tmp/receipt.json"],
            "abstained": false
        });
        let lines = format!(
            "{}\n{}\n",
            serde_json::json!({"type": "start"}),
            serde_json::json!({"type": "result", "structured_output": result})
        );
        fs::write(&trace, lines).unwrap();

        extract_agent_output(
            "claude_code",
            &trace,
            &output,
            EXPIRED_PROCEDURE_NATIVE_PILOT_SCHEMA_VERSION,
        )
        .unwrap();
        let actual: Value = serde_json::from_reader(File::open(output).unwrap()).unwrap();
        assert_eq!(actual, result);
    }

    fn materialize_synthetic_stale_output_contract(root: &Path) -> u32 {
        let protocol = crate::native_stale::tests::protocol();
        let schema =
            crate::native_stale::stale_agent_output_schema(&protocol.native_pilot).unwrap();
        materialize_synthetic_stale_output_contract_from(root, protocol, schema)
    }

    fn materialize_synthetic_stale_output_contract_from(
        root: &Path,
        protocol: crate::native_stale::NativeStaleSafetyProtocol,
        schema: String,
    ) -> u32 {
        let snapshot = root.join("stale-safety.protocol.snapshot.json");
        fs::write(
            &snapshot,
            serde_json::to_string_pretty(&protocol).unwrap() + "\n",
        )
        .unwrap();
        fs::write(
            root.join("stale-safety.protocol.snapshot.sha256"),
            format!("{}\n", sha256_file(&snapshot).unwrap()),
        )
        .unwrap();
        let precondition = NativeStalePreparationPrecondition {
            schema_version: 1,
            shared_retention_gate_hours: protocol.stale_safety.shared_retention_gate_hours,
            shared_retention_gate_ms: u64::from(protocol.stale_safety.shared_retention_gate_hours)
                * 60
                * 60
                * 1_000,
            deadline_source: "runner-teaching.json.completed_unix_ms".to_string(),
            teaching_completed_unix_ms: None,
            not_before_unix_ms: None,
            required_before_phases: vec!["activation".to_string(), "evaluation".to_string()],
        };
        let precondition_path = root.join("stale-safety.retention-precondition.json");
        fs::write(
            &precondition_path,
            serde_json::to_string_pretty(&precondition).unwrap() + "\n",
        )
        .unwrap();
        fs::write(
            root.join("stale-safety.retention-precondition.sha256"),
            format!("{}\n", sha256_file(&precondition_path).unwrap()),
        )
        .unwrap();
        fs::write(root.join("agent-output.schema.json"), &schema).unwrap();
        fs::write(
            root.join("protocol.snapshot.json"),
            serde_json::to_string_pretty(&protocol.native_pilot).unwrap() + "\n",
        )
        .unwrap();

        let binding = crate::native_pilot::PreparedNativeStaleSafetyBinding {
            family_version: protocol.stale_safety.family_version,
            protocol_snapshot_file: "stale-safety.protocol.snapshot.json".to_string(),
            protocol_snapshot_sha256: sha256_file(&snapshot).unwrap(),
            retention_precondition_file: "stale-safety.retention-precondition.json".to_string(),
            retention_precondition_sha256: sha256_file(&precondition_path).unwrap(),
            agent_output_schema_sha256: {
                let mut digest = Sha256::new();
                digest.update(schema.as_bytes());
                format!("{:x}", digest.finalize())
            },
        };
        let arms = protocol
            .native_pilot
            .arms
            .iter()
            .map(|arm| (arm.id.as_str(), arm))
            .collect::<BTreeMap<_, _>>();
        let mut prepared = plan(protocol.native_pilot.claude_budget_cents);
        prepared.protocol_schema_version = protocol.native_pilot.schema_version;
        prepared.pilot_id = protocol.native_pilot.pilot_id.clone();
        prepared.codex_min_idle_hours = protocol.native_pilot.codex_min_idle_hours;
        prepared.resource_budgets = protocol.native_pilot.resource_budgets.clone();
        prepared.claude_budget_cents = protocol.native_pilot.claude_budget_cents;
        prepared.claude_prior_spend_microusd = protocol.native_pilot.claude_prior_spend_microusd;
        prepared.claude_authorized_ceiling_cents =
            protocol.native_pilot.claude_authorized_ceiling_cents;
        prepared.claude_model = protocol.native_pilot.claude_model.clone();
        prepared.claude_max_turns = protocol.native_pilot.claude_max_turns;
        prepared.stale_safety = Some(binding);
        prepared.lanes = protocol
            .stale_safety
            .lanes
            .iter()
            .map(|lane| {
                let arm = arms[lane.arm.as_str()];
                let lane_dir = root.join("lanes").join(&lane.opaque_path_label);
                PreparedNativeLane {
                    order: lane.order,
                    case_id: lane.case_id.clone(),
                    arm: lane.arm.clone(),
                    host: arm.host.clone(),
                    memory_layer: arm.memory_layer,
                    repetition: lane.repetition,
                    fixture_revision: "synthetic".to_string(),
                    teaching_cwd: lane_dir.join("teaching").display().to_string(),
                    evaluation_cwd: lane_dir.join("evaluation").display().to_string(),
                    environment: BTreeMap::new(),
                    required_secret_environment: Vec::new(),
                    codex_authentication: (arm.host == "codex").then(|| {
                        PreparedCodexAuthentication {
                            mode: CodexAuthenticationMode::ChatgptFileCache,
                            credential_store: "file".to_string(),
                            login_argv: vec!["codex".to_string(), "login".to_string()],
                            status_argv: vec![
                                "codex".to_string(),
                                "login".to_string(),
                                "status".to_string(),
                            ],
                            expected_status: CODEX_CHATGPT_LOGIN_STATUS.to_string(),
                        }
                    }),
                    claude_teaching_bash_commands: Vec::new(),
                    teaching_argv: Vec::new(),
                    activation_argv: (arm.host == "codex").then(Vec::new),
                    post_teaching_verification_argv: None,
                    activation_wait_hours: 1,
                    evaluation_argv: Vec::new(),
                    artifact_gates: Vec::new(),
                    acceptance_contract: lane_dir
                        .join("acceptance-contract.json")
                        .display()
                        .to_string(),
                    adapter_sha256: None,
                    engram_home: None,
                    engram_project: None,
                    cleanup_argv: None,
                    teaching_trace_path: lane_dir
                        .join("teaching-trace.jsonl")
                        .display()
                        .to_string(),
                    activation_trace_path: (arm.host == "codex").then(|| {
                        lane_dir
                            .join("activation-trace.jsonl")
                            .display()
                            .to_string()
                    }),
                    evaluation_trace_path: lane_dir
                        .join("evaluation-trace.jsonl")
                        .display()
                        .to_string(),
                    post_teaching_verification_output_path: None,
                    agent_output_path: lane_dir.join("agent-output.json").display().to_string(),
                }
            })
            .collect();
        let run_plan = root.join("run-plan.json");
        fs::write(
            &run_plan,
            serde_json::to_string_pretty(&prepared).unwrap() + "\n",
        )
        .unwrap();
        fs::write(
            root.join("run-plan.sha256"),
            format!("{}\n", sha256_file(&run_plan).unwrap()),
        )
        .unwrap();
        protocol.native_pilot.schema_version
    }

    #[test]
    fn stale_evaluation_receipt_binds_pre_provider_marker_snapshot() {
        let root = tempfile::tempdir().unwrap();
        materialize_synthetic_stale_output_contract(root.path());
        let run_plan = root.path().join("run-plan.json");
        let plan: PreparedNativePilot =
            serde_json::from_reader(File::open(&run_plan).unwrap()).unwrap();
        for lane in &plan.lanes {
            fs::create_dir_all(Path::new(&lane.acceptance_contract).parent().unwrap()).unwrap();
        }
        let captured_unix_ms = plan.prepared_unix_ms.max(1_050);
        let digest =
            materialize_native_stale_evaluation_marker_snapshot(&plan, &run_plan, captured_unix_ms)
                .unwrap()
                .unwrap();
        let snapshot =
            validate_native_stale_evaluation_marker_snapshot(&plan, &run_plan, &digest).unwrap();
        assert_eq!(snapshot.captured_unix_ms, captured_unix_ms);
        assert_eq!(snapshot.lanes.len(), 12);
        assert!(materialize_native_stale_evaluation_marker_snapshot(
            &plan,
            &run_plan,
            captured_unix_ms,
        )
        .unwrap_err()
        .to_string()
        .contains("refusing to overwrite"));

        let mut receipt = receipt_for(&plan, &run_plan, NativePilotRunPhase::Evaluation);
        receipt.started_unix_ms = captured_unix_ms;
        for (index, lane) in receipt.lanes.iter_mut().enumerate() {
            let started = captured_unix_ms + 1 + u64::try_from(index).unwrap() * 2;
            lane.provider_started_unix_ms = Some(started);
            lane.provider_completed_unix_ms = Some(started + 1);
        }
        receipt.completed_unix_ms = captured_unix_ms + 100;
        receipt.stale_safety_pre_evaluation_marker_snapshot_sha256 = Some(digest.clone());
        validate_stale_phase_receipt(&plan, &run_plan, NativePilotRunPhase::Evaluation, &receipt)
            .unwrap();

        let activation_path = root.path().join("runner-activation.json");
        write_private_json(
            &activation_path,
            &receipt_for(&plan, &run_plan, NativePilotRunPhase::Activation),
        )
        .unwrap();
        let activation_digest_path = root.path().join("runner-activation.sha256");
        fs::write(
            &activation_digest_path,
            format!("{}\n", sha256_file(&activation_path).unwrap()),
        )
        .unwrap();
        read_validated_stale_phase_receipt(&plan, &run_plan, NativePilotRunPhase::Activation)
            .unwrap();
        fs::write(&activation_digest_path, format!("{}\n", "0".repeat(64))).unwrap();
        assert!(read_validated_stale_phase_receipt(
            &plan,
            &run_plan,
            NativePilotRunPhase::Activation,
        )
        .unwrap_err()
        .to_string()
        .contains("digest is invalid"));

        let evaluation_path = root.path().join("runner-evaluation.json");
        write_private_json(&evaluation_path, &receipt).unwrap();
        let evaluation_digest_path = root.path().join("runner-evaluation.sha256");
        fs::write(
            &evaluation_digest_path,
            format!("{}\n", sha256_file(&evaluation_path).unwrap()),
        )
        .unwrap();
        read_validated_stale_phase_receipt(&plan, &run_plan, NativePilotRunPhase::Evaluation)
            .unwrap();
        fs::write(&evaluation_digest_path, format!("{}\n", "0".repeat(64))).unwrap();
        assert!(read_validated_stale_phase_receipt(
            &plan,
            &run_plan,
            NativePilotRunPhase::Evaluation,
        )
        .unwrap_err()
        .to_string()
        .contains("digest is invalid"));

        let mut missing_snapshot = receipt.clone();
        missing_snapshot.stale_safety_pre_evaluation_marker_snapshot_sha256 = None;
        assert!(validate_stale_phase_receipt(
            &plan,
            &run_plan,
            NativePilotRunPhase::Evaluation,
            &missing_snapshot,
        )
        .unwrap_err()
        .to_string()
        .contains("omits its pre-evaluation marker snapshot"));

        let snapshot_path = root
            .path()
            .join("stale-safety.pre-evaluation-native-markers.json");
        let mut late_snapshot: Value =
            serde_json::from_reader(File::open(&snapshot_path).unwrap()).unwrap();
        late_snapshot["captured_unix_ms"] = serde_json::json!(captured_unix_ms + 2);
        fs::write(
            &snapshot_path,
            serde_json::to_string_pretty(&late_snapshot).unwrap() + "\n",
        )
        .unwrap();
        let late_digest = sha256_file(&snapshot_path).unwrap();
        fs::write(
            root.path()
                .join("stale-safety.pre-evaluation-native-markers.sha256"),
            format!("{late_digest}\n"),
        )
        .unwrap();
        receipt.stale_safety_pre_evaluation_marker_snapshot_sha256 = Some(late_digest);
        assert!(validate_stale_phase_receipt(
            &plan,
            &run_plan,
            NativePilotRunPhase::Evaluation,
            &receipt,
        )
        .unwrap_err()
        .to_string()
        .contains("not captured before provider execution"));
    }

    fn stale_agent_output() -> Value {
        serde_json::json!({
            "answer": "boundary:verification_expired retention_absent",
            "repository_remote": "https://github.com/acme/atlas",
            "project": null,
            "component": "queue-worker",
            "first_action": null,
            "returned_context_keys": [],
            "applied_context_keys": [],
            "evidence_targets": ["toolchain.toml"],
            "abstained": true,
            "boundary_signal": "boundary:verification_expired",
            "native_retention_signal": "retention_absent"
        })
    }

    fn stale_v3_agent_output_schema(
        protocol: &crate::native_stale::NativeStaleSafetyProtocol,
    ) -> String {
        let mut schema: Value = serde_json::from_str(
            &crate::native_pilot::native_agent_output_schema(&protocol.native_pilot).unwrap(),
        )
        .unwrap();
        let properties = schema
            .get_mut("properties")
            .and_then(Value::as_object_mut)
            .unwrap();
        properties.insert(
            "boundary_signal".to_string(),
            serde_json::json!({
                "type": "string",
                "enum": ["boundary:source_unavailable", "boundary:verification_expired"],
                "description": "Exact causal boundary proven from current source or expiry evidence."
            }),
        );
        properties.remove("native_retention_signal");
        properties.insert(
            "native_evidence_signal".to_string(),
            serde_json::json!({
                "type": "string",
                "enum": [
                    "not_applicable",
                    "source_markers_correlated",
                    "expiry_markers_correlated",
                    "native_evidence_insufficient"
                ],
                "description": "Typed native-memory evidence correlated to the proven boundary."
            }),
        );
        properties.insert(
            "causal_result".to_string(),
            serde_json::json!({
                "type": "string",
                "enum": ["FAIL", "CAUSAL_PASS", "SAFE_INCONCLUSIVE"],
                "description": "Preregistered causal classification for this lane."
            }),
        );
        properties.insert(
            "retrieved_native_markers".to_string(),
            serde_json::json!({
                "type": "array",
                "items": {"type": "string"},
                "maxItems": 2,
                "uniqueItems": true,
                "description": "Ordered exact own marker values actually retrieved; empty when none were retrieved."
            }),
        );
        let required = schema
            .get_mut("required")
            .and_then(Value::as_array_mut)
            .unwrap();
        required.push(Value::String("boundary_signal".to_string()));
        required.extend(
            [
                "native_evidence_signal",
                "causal_result",
                "retrieved_native_markers",
            ]
            .into_iter()
            .map(|key| Value::String(key.to_string())),
        );
        serde_json::to_string_pretty(&schema).unwrap() + "\n"
    }

    fn stale_v3_agent_output() -> Value {
        serde_json::json!({
            "answer": "boundary:source_unavailable",
            "repository_remote": "https://github.com/acme/atlas",
            "project": null,
            "component": "queue-worker",
            "first_action": null,
            "returned_context_keys": [],
            "applied_context_keys": [],
            "evidence_targets": ["toolchain.toml"],
            "abstained": true,
            "boundary_signal": "boundary:source_unavailable",
            "native_evidence_signal": "native_evidence_insufficient",
            "causal_result": "SAFE_INCONCLUSIVE",
            "retrieved_native_markers": []
        })
    }

    fn materialize_synthetic_v3_output_contract(root: &Path) -> u32 {
        let protocol: crate::native_stale::NativeStaleSafetyProtocol = serde_json::from_str(
            include_str!(
                "../../evals/native_memory_pilot_v1/protocol-native-stale-safety-v3-r2-base-schema-10-direct-host-file-cache.json"
            ),
        )
        .unwrap();
        let schema = stale_v3_agent_output_schema(&protocol);
        materialize_synthetic_stale_output_contract_from(root, protocol, schema)
    }

    #[test]
    fn stale_v3_extracts_exact_synthetic_codex_and_claude_outputs() {
        let root = tempfile::tempdir().unwrap();
        let schema_version = materialize_synthetic_v3_output_contract(root.path());
        let expected = stale_v3_agent_output();
        for (index, host) in ["codex", "claude_code"].into_iter().enumerate() {
            let lane = root
                .path()
                .join("lanes")
                .join(format!("synthetic-v3-lane-{index}"));
            fs::create_dir_all(&lane).unwrap();
            let trace = lane.join("evaluation-trace.jsonl");
            let output = lane.join("agent-output.json");
            let contents = if host == "codex" {
                [
                    serde_json::json!({"type": "turn.started"}),
                    serde_json::json!({
                        "type": "item.completed",
                        "item": {"type": "agent_message", "text": expected.to_string()}
                    }),
                    serde_json::json!({"type": "turn.completed"}),
                ]
                .into_iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
                .join("\n")
            } else {
                format!(
                    "{}\n{}",
                    serde_json::json!({
                        "type": "assistant",
                        "message": {"content": [{
                            "type": "tool_use",
                            "name": "StructuredOutput",
                            "input": expected
                        }]}
                    }),
                    serde_json::json!({"type": "result", "structured_output": expected})
                )
            };
            fs::write(&trace, format!("{contents}\n")).unwrap();
            extract_agent_output(host, &trace, &output, schema_version).unwrap();
            assert_eq!(
                serde_json::from_reader::<_, Value>(File::open(output).unwrap()).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn stale_v3_codex_terminal_extraction_requires_exact_external_action_pairs() {
        let root = tempfile::tempdir().unwrap();
        let schema_version = materialize_synthetic_v3_output_contract(root.path());
        let lane = root.path().join("lanes/synthetic-v3-action-lifecycle");
        fs::create_dir_all(&lane).unwrap();
        let trace = lane.join("evaluation-trace.jsonl");
        let output = stale_v3_agent_output();
        assert!(is_agent_output(
            &output,
            schema_version,
            NativeAgentOutputContract::StaleSafetyV3,
        ));
        let started = serde_json::json!({
            "type": "item.started",
            "item": {
                "id": "item_0",
                "type": "command_execution",
                "command": "git rev-parse --show-toplevel",
                "aggregated_output": "",
                "exit_code": null,
                "status": "in_progress"
            }
        });
        let completed = serde_json::json!({
            "type": "item.completed",
            "item": {
                "id": "item_0",
                "type": "command_execution",
                "command": "git rev-parse --show-toplevel",
                "aggregated_output": "/tmp/checkout\n",
                "exit_code": 0,
                "status": "completed"
            }
        });
        let terminal = |actions: Vec<Value>| {
            let mut events = vec![serde_json::json!({"type": "turn.started"})];
            events.extend(actions);
            events.push(serde_json::json!({
                "type": "item.completed",
                "item": {"type": "agent_message", "text": output.to_string()}
            }));
            events.push(serde_json::json!({"type": "turn.completed"}));
            events
                .into_iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
                .join("\n")
                + "\n"
        };

        fs::write(&trace, terminal(vec![started.clone(), completed.clone()])).unwrap();
        let valid = codex_terminal_agent_output(&trace, schema_version);
        assert!(
            valid.is_ok(),
            "{valid:?}; trace={}",
            fs::read_to_string(&trace).unwrap()
        );

        fs::write(&trace, terminal(vec![started.clone()])).unwrap();
        assert!(codex_terminal_agent_output(&trace, schema_version).is_err());

        fs::write(&trace, terminal(vec![completed.clone()])).unwrap();
        assert!(codex_terminal_agent_output(&trace, schema_version).is_err());

        let mut mismatched = completed.clone();
        mismatched["item"]["command"] = Value::String("curl https://example.invalid".into());
        fs::write(&trace, terminal(vec![started.clone(), mismatched])).unwrap();
        assert!(codex_terminal_agent_output(&trace, schema_version).is_err());

        fs::write(
            &trace,
            terminal(vec![started, completed.clone(), completed]),
        )
        .unwrap();
        assert!(codex_terminal_agent_output(&trace, schema_version).is_err());
    }

    #[test]
    fn stale_v3_rejects_extra_legacy_keys_invalid_enums_and_duplicate_markers() {
        let root = tempfile::tempdir().unwrap();
        let schema_version = materialize_synthetic_v3_output_contract(root.path());
        let lane = root.path().join("lanes/synthetic-v3-invalid");
        fs::create_dir_all(&lane).unwrap();
        let trace = lane.join("evaluation-trace.jsonl");

        let mut candidates = Vec::new();
        let mut legacy = stale_v3_agent_output();
        legacy["native_retention_signal"] = Value::String("retention_absent".to_string());
        candidates.push(legacy);
        for (key, value) in [
            ("boundary_signal", "boundary:evidence_insufficient"),
            ("native_evidence_signal", "invented"),
            ("causal_result", "PASS"),
        ] {
            let mut invalid = stale_v3_agent_output();
            invalid[key] = Value::String(value.to_string());
            candidates.push(invalid);
        }
        let mut duplicate = stale_v3_agent_output();
        duplicate["retrieved_native_markers"] = serde_json::json!(["same", "same"]);
        candidates.push(duplicate);
        let mut too_many = stale_v3_agent_output();
        too_many["retrieved_native_markers"] = serde_json::json!(["one", "two", "three"]);
        candidates.push(too_many);

        for candidate in candidates {
            fs::write(
                &trace,
                [
                    serde_json::json!({"type": "turn.started"}),
                    serde_json::json!({
                        "type": "item.completed",
                        "item": {"type": "agent_message", "text": candidate.to_string()}
                    }),
                    serde_json::json!({"type": "turn.completed"}),
                ]
                .into_iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
                .join("\n"),
            )
            .unwrap();
            assert!(codex_terminal_agent_output(&trace, schema_version).is_err());
        }
    }

    #[test]
    fn stale_capability_extracts_synthetic_codex_terminal_output() {
        let root = tempfile::tempdir().unwrap();
        let schema_version = materialize_synthetic_stale_output_contract(root.path());
        let lane = root.path().join("lanes/lane-0000000000000001");
        fs::create_dir_all(&lane).unwrap();
        let trace = lane.join("evaluation-trace.jsonl");
        let output = lane.join("agent-output.json");
        let expected = stale_agent_output();
        fs::write(
            &trace,
            [
                serde_json::json!({"type": "thread.started"}),
                serde_json::json!({"type": "turn.started"}),
                serde_json::json!({
                    "type": "item.completed",
                    "item": {"type": "agent_message", "text": expected.to_string()}
                }),
                serde_json::json!({"type": "turn.completed"}),
            ]
            .into_iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join("\n"),
        )
        .unwrap();

        extract_agent_output("codex", &trace, &output, schema_version).unwrap();
        let actual: Value = serde_json::from_reader(File::open(output).unwrap()).unwrap();
        assert_eq!(actual, expected);

        let mut invalid = expected;
        invalid["native_retention_signal"] = Value::String("invented".to_string());
        fs::write(
            &trace,
            [
                serde_json::json!({"type": "turn.started"}),
                serde_json::json!({
                    "type": "item.completed",
                    "item": {"type": "agent_message", "text": invalid.to_string()}
                }),
                serde_json::json!({"type": "turn.completed"}),
            ]
            .into_iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join("\n"),
        )
        .unwrap();
        assert!(codex_terminal_agent_output(&trace, schema_version).is_err());
    }

    #[test]
    fn stale_capability_extracts_synthetic_claude_outputs() {
        let root = tempfile::tempdir().unwrap();
        let schema_version = materialize_synthetic_stale_output_contract(root.path());
        let lane = root.path().join("lanes/lane-0000000000000002");
        fs::create_dir_all(&lane).unwrap();
        let trace = lane.join("evaluation-trace.jsonl");
        let output = lane.join("agent-output.json");
        let expected = stale_agent_output();
        fs::write(
            &trace,
            format!(
                "{}\n{}\n",
                serde_json::json!({
                    "type": "assistant",
                    "message": {"content": [{
                        "type": "tool_use",
                        "name": "StructuredOutput",
                        "input": expected
                    }]}
                }),
                serde_json::json!({"type": "result", "structured_output": expected})
            ),
        )
        .unwrap();

        extract_agent_output("claude_code", &trace, &output, schema_version).unwrap();
        assert_eq!(
            serde_json::from_reader::<_, Value>(File::open(output).unwrap()).unwrap(),
            expected
        );
        assert_eq!(
            unique_structured_output_tool_input(&trace, schema_version).unwrap(),
            expected
        );

        let run_plan = root.path().join("run-plan.json");
        let snapshot = root.path().join("stale-safety.protocol.snapshot.json");
        let snapshot_bytes = fs::read(&snapshot).unwrap();
        fs::remove_file(&snapshot).unwrap();
        assert!(unique_agent_output(&trace, schema_version).is_err());
        fs::write(&snapshot, snapshot_bytes).unwrap();

        let mut unbound: Value = serde_json::from_reader(File::open(&run_plan).unwrap()).unwrap();
        unbound.as_object_mut().unwrap().remove("stale_safety");
        fs::write(
            &run_plan,
            serde_json::to_string_pretty(&unbound).unwrap() + "\n",
        )
        .unwrap();
        fs::write(
            root.path().join("run-plan.sha256"),
            format!("{}\n", sha256_file(&run_plan).unwrap()),
        )
        .unwrap();
        for name in [
            "stale-safety.protocol.snapshot.json",
            "stale-safety.protocol.snapshot.sha256",
            "stale-safety.retention-precondition.json",
            "stale-safety.retention-precondition.sha256",
            "stale-safety.preflight.json",
            "stale-safety.preflight.sha256",
        ] {
            let path = root.path().join(name);
            if path.exists() {
                fs::remove_file(path).unwrap();
            }
        }
        assert!(unique_agent_output(&trace, schema_version)
            .unwrap_err()
            .to_string()
            .contains("without a frozen run-plan binding"));
    }

    #[test]
    fn historical_unbound_plan_keeps_base_output_contract() {
        let root = tempfile::tempdir().unwrap();
        let lane = root.path().join("lanes/historical-lane");
        fs::create_dir_all(&lane).unwrap();
        let run_plan = root.path().join("run-plan.json");
        fs::write(
            &run_plan,
            serde_json::to_string_pretty(&plan(30)).unwrap() + "\n",
        )
        .unwrap();
        fs::write(
            root.path().join("run-plan.sha256"),
            format!("{}\n", sha256_file(&run_plan).unwrap()),
        )
        .unwrap();
        let trace = lane.join("evaluation-trace.jsonl");
        fs::write(&trace, "{\"type\":\"historical\"}\n").unwrap();
        assert_eq!(
            trace_native_stale_family_version(&trace, plan(30).protocol_schema_version).unwrap(),
            None
        );

        fs::write(
            root.path().join("stale-safety.protocol.snapshot.sha256"),
            format!("{}\n", "0".repeat(64)),
        )
        .unwrap();
        assert!(
            trace_native_stale_family_version(&trace, plan(30).protocol_schema_version)
                .unwrap_err()
                .to_string()
                .contains("artifacts exist without a frozen run-plan binding")
        );
    }

    #[test]
    fn schema_twelve_agent_output_recovery_requires_exact_structured_identity_keys() {
        let legacy = serde_json::json!({
            "answer": "abstained",
            "repository_remote": "https://github.com/acme/orbit",
            "project": null,
            "component": "queue-worker",
            "first_action": "resolve_checkout_identity",
            "returned_context_keys": [],
            "applied_context_keys": [],
            "evidence_targets": ["services/worker/component.json"],
            "abstained": true
        });
        assert!(!is_agent_output(
            &legacy,
            STRUCTURED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION,
            NativeAgentOutputContract::Base,
        ));

        let mut structured = legacy.as_object().unwrap().clone();
        structured.insert(
            "checkout_root".to_string(),
            Value::String("/tmp/orbit".to_string()),
        );
        structured.insert(
            "project_status".to_string(),
            Value::String("requires_confirmation".to_string()),
        );
        structured.insert(
            "project_confirmation_required".to_string(),
            Value::Bool(true),
        );
        let structured = Value::Object(structured);
        assert!(is_agent_output(
            &structured,
            STRUCTURED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION,
            NativeAgentOutputContract::Base,
        ));

        let mut extra = structured.as_object().unwrap().clone();
        extra.insert(
            "project_guess".to_string(),
            Value::String("orbit".to_string()),
        );
        assert!(!is_agent_output(
            &Value::Object(extra),
            STRUCTURED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION,
            NativeAgentOutputContract::Base,
        ));
    }

    #[test]
    fn extracts_json_encoded_agent_output_and_deduplicates_repeats() {
        let root = tempfile::tempdir().unwrap();
        let trace = root.path().join("trace.jsonl");
        let output = root.path().join("output.json");
        let result = serde_json::json!({
            "answer": "abstained",
            "repository_remote": null,
            "project": null,
            "component": null,
            "first_action": null,
            "returned_context_keys": [],
            "applied_context_keys": [],
            "evidence_targets": [],
            "abstained": true
        });
        let encoded = serde_json::to_string(&result).unwrap();
        fs::write(
            &trace,
            format!(
                "{}\n{}\n",
                serde_json::json!({"item": {"text": encoded}}),
                serde_json::json!({"result": result})
            ),
        )
        .unwrap();

        extract_agent_output(
            "claude_code",
            &trace,
            &output,
            EXPIRED_PROCEDURE_NATIVE_PILOT_SCHEMA_VERSION,
        )
        .unwrap();
        let actual: Value = serde_json::from_reader(File::open(output).unwrap()).unwrap();
        assert_eq!(actual, result);
    }

    #[test]
    fn codex_extracts_only_the_terminal_agent_message_from_one_completed_turn() {
        let root = tempfile::tempdir().unwrap();
        let trace = root.path().join("trace.jsonl");
        let output = root.path().join("output.json");
        let progress = serde_json::json!({
            "answer": "orienting",
            "repository_remote": null,
            "project": null,
            "component": null,
            "first_action": null,
            "returned_context_keys": [],
            "applied_context_keys": [],
            "evidence_targets": [],
            "abstained": false
        });
        let final_output = serde_json::json!({
            "answer": "done",
            "repository_remote": "https://github.com/acme/atlas",
            "project": null,
            "component": "queue-worker",
            "first_action": "./bin/context-probe --channel cobalt",
            "returned_context_keys": ["procedure.atlas"],
            "applied_context_keys": ["procedure.atlas"],
            "evidence_targets": ["toolchain.toml"],
            "abstained": false
        });
        fs::write(
            &trace,
            [
                serde_json::json!({"type": "thread.started"}),
                serde_json::json!({"type": "turn.started"}),
                serde_json::json!({
                    "type": "item.completed",
                    "item": {"type": "agent_message", "text": progress.to_string()}
                }),
                serde_json::json!({
                    "type": "item.completed",
                    "item": {"type": "command_execution", "exit_code": 0}
                }),
                serde_json::json!({
                    "type": "item.completed",
                    "item": {"type": "agent_message", "text": final_output.to_string()}
                }),
                serde_json::json!({"type": "turn.completed"}),
            ]
            .into_iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join("\n"),
        )
        .unwrap();

        assert_eq!(
            distinct_agent_output_count(&trace, EXPIRED_PROCEDURE_NATIVE_PILOT_SCHEMA_VERSION,)
                .unwrap(),
            2
        );
        assert!(
            unique_agent_output(&trace, EXPIRED_PROCEDURE_NATIVE_PILOT_SCHEMA_VERSION,).is_err()
        );
        extract_agent_output(
            "codex",
            &trace,
            &output,
            EXPIRED_PROCEDURE_NATIVE_PILOT_SCHEMA_VERSION,
        )
        .unwrap();
        let actual: Value = serde_json::from_reader(File::open(output).unwrap()).unwrap();
        assert_eq!(actual, final_output);
    }

    #[test]
    fn candidate_selection_requires_exact_structured_procedure() {
        let root = tempfile::tempdir().unwrap();
        let candidates = root.path().join("candidates.json");
        fs::write(
            &candidates,
            serde_json::to_vec(&serde_json::json!([{
                "id": "019example",
                "kind": "procedure",
                "status": "needs_review",
                "content": "procedure.atlas",
                "scope": {
                    "type": "repository",
                    "remote_url": "git@github.com:acme/atlas.git"
                },
                "procedure": {
                    "task": "run probe",
                    "commands": ["./bin/probe --channel cobalt"],
                    "prerequisites": [{
                        "key": "probe.version",
                        "expected": "1.0",
                        "source": {
                            "format": "toml",
                            "relative_path": "toolchain.toml",
                            "key_path": ["probe", "version"]
                        }
                    }],
                    "failure_signatures": ["./bin/probe --channel amber"],
                    "verification": {
                        "expected_exit_code": 0,
                        "expected_output_contains": "ATLAS_OK"
                    }
                }
            }]))
            .unwrap(),
        )
        .unwrap();
        let contract = RunnerAcceptanceContract {
            expected_repository_remote: "https://github.com/acme/orbit".to_string(),
            required_context_keys: vec!["procedure.atlas".to_string()],
            required_task: "run probe".to_string(),
            required_conditions: BTreeMap::from([("probe.version".to_string(), "1.0".to_string())]),
            required_prerequisite_sources: BTreeMap::from([(
                "probe.version".to_string(),
                NativePilotPrerequisiteSource {
                    format: "toml".to_string(),
                    relative_path: "toolchain.toml".to_string(),
                    key_path: vec!["probe".to_string(), "version".to_string()],
                },
            )]),
            forbidden_commands: vec!["./bin/probe --channel amber".to_string()],
            required_command: "./bin/probe --channel cobalt".to_string(),
            expected_exit_code: 0,
            expected_output_contains: "ATLAS_OK".to_string(),
        };

        assert_eq!(
            select_procedure_candidate(&candidates, &contract, "git@github.com:acme/atlas.git",)
                .unwrap(),
            "019example"
        );
        assert!(select_procedure_candidate(
            &candidates,
            &contract,
            "git@github.com:acme/orbit.git",
        )
        .is_err());
    }

    #[test]
    fn private_json_never_overwrites() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("report.json");
        write_private_json(&path, &serde_json::json!({"first": true})).unwrap();
        assert!(write_private_json(&path, &serde_json::json!({"second": true})).is_err());
        assert!(fs::read_to_string(path).unwrap().contains("first"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            assert_eq!(
                fs::metadata(root.path().join("report.json"))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }
    }

    #[test]
    fn frozen_run_plan_digest_detects_drift() {
        let root = tempfile::tempdir().unwrap();
        let plan = root.path().join("run-plan.json");
        fs::write(&plan, "{\"pilot_id\":\"one\"}\n").unwrap();
        fs::write(
            root.path().join("run-plan.sha256"),
            format!("{}\n", sha256_file(&plan).unwrap()),
        )
        .unwrap();
        validate_plan_digest(&plan).unwrap();

        fs::write(&plan, "{\"pilot_id\":\"two\"}\n").unwrap();
        assert!(validate_plan_digest(&plan)
            .unwrap_err()
            .to_string()
            .contains("changed after preparation"));
    }

    #[cfg(unix)]
    #[test]
    fn recovery_preparation_firewalls_source_before_digest_and_output_paths() {
        use std::os::unix::fs::symlink;

        let root = tempfile::tempdir().unwrap();
        let target_root = root.path().join("target");
        fs::create_dir(&target_root).unwrap();
        let target = target_root.join("run-plan.json");
        fs::write(
            &target,
            serde_json::to_string_pretty(&plan(1)).unwrap() + "\n",
        )
        .unwrap();
        fs::create_dir(target_root.join("run-plan.sha256")).unwrap();
        let alias = root.path().join("source-plan-alias.json");
        symlink(&target, &alias).unwrap();

        let oversized_root = root.path().join("oversized");
        fs::create_dir(&oversized_root).unwrap();
        let oversized = oversized_root.join("run-plan.json");
        fs::write(&oversized, vec![b' '; 4 * 1024 * 1024 + 1]).unwrap();
        fs::create_dir(oversized_root.join("run-plan.sha256")).unwrap();

        for (label, source, expected) in [
            ("symlink", alias.as_path(), "single-link regular file"),
            (
                "oversized",
                oversized.as_path(),
                "no larger than 4194304 bytes",
            ),
        ] {
            let execution_output = root.path().join(format!("execution-{label}"));
            let error = prepare_native_memory_pilot_execution_recovery(source, &execution_output)
                .unwrap_err()
                .to_string();
            assert!(error.contains(expected), "{error}");
            assert!(!execution_output.exists());

            let evaluation_output = root.path().join(format!("evaluation-{label}"));
            let binary_sentinel = root.path().join(format!("binary-sentinel-{label}"));
            let error = prepare_native_memory_pilot_evaluation_recovery(
                source,
                &evaluation_output,
                Some(&binary_sentinel),
            )
            .unwrap_err()
            .to_string();
            assert!(error.contains(expected), "{error}");
            assert!(!evaluation_output.exists());
            assert!(!binary_sentinel.exists());
        }
    }

    #[cfg(unix)]
    #[test]
    fn embedded_recovery_validation_firewalls_source_before_digest_and_other_paths() {
        use std::os::unix::fs::symlink;

        let root = tempfile::tempdir().unwrap();
        let target_root = root.path().join("target");
        fs::create_dir(&target_root).unwrap();
        let target = target_root.join("run-plan.json");
        fs::write(
            &target,
            serde_json::to_string_pretty(&plan(1)).unwrap() + "\n",
        )
        .unwrap();
        fs::create_dir(target_root.join("run-plan.sha256")).unwrap();
        let alias = root.path().join("source-plan-alias.json");
        symlink(&target, &alias).unwrap();

        let oversized_root = root.path().join("oversized");
        fs::create_dir(&oversized_root).unwrap();
        let oversized = oversized_root.join("run-plan.json");
        fs::write(&oversized, vec![b' '; 4 * 1024 * 1024 + 1]).unwrap();
        fs::create_dir(oversized_root.join("run-plan.sha256")).unwrap();

        for (source, expected) in [
            (alias.as_path(), "single-link regular file"),
            (oversized.as_path(), "no larger than 4194304 bytes"),
        ] {
            let mut execution = plan(1);
            execution.execution_recovery = Some(PreparedNativeExecutionRecovery {
                source_run_plan: source.display().to_string(),
                source_run_plan_sha256: "0".repeat(64),
                interrupted_phase: "teaching".to_string(),
                failure_reason: PROCEDURE_TEACHING_SCOPE_REJECTION.to_string(),
                recovery_prepared_unix_ms: 2,
                completed_lane_orders: vec![1],
                inherited_files: Vec::new(),
                claude_remaining_calls: 1,
            });
            let error = validate_execution_recovery(&execution)
                .unwrap_err()
                .to_string();
            assert!(error.contains(expected), "{error}");

            let mut evaluation = plan(1);
            evaluation.evaluation_recovery = Some(PreparedNativeEvaluationRecovery {
                source_run_plan: source.display().to_string(),
                source_run_plan_sha256: "0".repeat(64),
                rejected_evaluation_trace: root
                    .path()
                    .join("trace-must-not-be-read")
                    .display()
                    .to_string(),
                rejected_evaluation_trace_sha256: "0".repeat(64),
                rejected_evaluation_stderr: root
                    .path()
                    .join("stderr-must-not-be-read")
                    .display()
                    .to_string(),
                rejected_evaluation_stderr_sha256: "0".repeat(64),
                rejection_reason: CLAUDE_SCHEMA_REJECTION.to_string(),
                recovery_prepared_unix_ms: 2,
                claude_remaining_calls: 1,
                agent_output_schema: root
                    .path()
                    .join("schema-must-not-be-read")
                    .display()
                    .to_string(),
            });
            let current_plan_sentinel = root.path().join("current-plan-must-not-be-read");
            let error = validate_evaluation_recovery(&evaluation, &current_plan_sentinel)
                .unwrap_err()
                .to_string();
            assert!(error.contains(expected), "{error}");
            assert!(!current_plan_sentinel.exists());
        }
    }
}
