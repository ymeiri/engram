//! Reproducible preparation for a learned native-memory versus Engram pilot.

use crate::fixture::materialize_engineering_context_fixture;
use crate::native_document::load_historical_native_protocol;
use crate::pilot::{
    attest_binary, attest_codex_code_mode_host, create_private_dir_all, format_budget_usd,
    install_eval_adapter, require_empty_target, resolve_checkout_root, resolve_fixture_uri,
    write_claude_mcp_config, PilotBinaryAttestation, AGENT_OUTPUT_SCHEMA,
};
use crate::{EvalError, EvalResult};
use engram_store::ENGRAM_HOME_ENV;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
#[cfg(target_os = "macos")]
use std::fs::{File, OpenOptions};
#[cfg(target_os = "macos")]
use std::io::Read;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) const SOURCE_BACKED_PREREQUISITE_NATIVE_PILOT_SCHEMA_VERSION: u32 = 8;
pub(crate) const UNAVAILABLE_PREREQUISITE_NATIVE_PILOT_SCHEMA_VERSION: u32 = 9;
pub(crate) const EXPIRED_PROCEDURE_NATIVE_PILOT_SCHEMA_VERSION: u32 = 10;
pub(crate) const TARGETED_HOST_DIAGNOSTIC_NATIVE_PILOT_SCHEMA_VERSION: u32 = 11;
pub(crate) const STRUCTURED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION: u32 = 12;
pub(crate) const OPERATION_EVIDENCE_ROUTE_NATIVE_PILOT_SCHEMA_VERSION: u32 = 13;
pub(crate) const SOURCE_BOUND_OPERATION_EVIDENCE_NATIVE_PILOT_SCHEMA_VERSION: u32 = 14;
pub(crate) const REQUIRED_HOST_ACTION_NATIVE_PILOT_SCHEMA_VERSION: u32 = 15;
pub(crate) const TRUSTED_SCOPED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION: u32 = 7;
#[cfg(test)]
const NATIVE_PILOT_SCHEMA_VERSION: u32 = EXPIRED_PROCEDURE_NATIVE_PILOT_SCHEMA_VERSION;
const ABSTENTION_NATIVE_PILOT_SCHEMA_VERSION: u32 = 6;
const IDENTITY_NATIVE_PILOT_SCHEMA_VERSION: u32 = 5;
const LEGACY_NATIVE_PILOT_SCHEMA_VERSION: u32 = 4;
pub(crate) const CODEX_KEYRING_CONFIG: &str = "cli_auth_credentials_store=\"keyring\"";
pub(crate) const CODEX_FILE_CONFIG: &str = "cli_auth_credentials_store=\"file\"";
pub(crate) const CODEX_CHATGPT_LOGIN_STATUS: &str = "Logged in using ChatGPT";
pub(crate) const NATIVE_PILOT_ONNXRUNTIME_LIBRARY_KEY: &str = "onnxruntime";
pub(crate) const NATIVE_PILOT_ENGRAM_RUNTIME_CONSUMER_KEY: &str = "engram";
pub(crate) const NATIVE_PILOT_EVALUATOR_RUNTIME_CONSUMER_KEY: &str = "engram_eval";
const NATIVE_PILOT_ONNXRUNTIME_DYLIB: &str = "libonnxruntime.1.20.0.dylib";
const NATIVE_PILOT_ONNXRUNTIME_INSTALL_NAME: &str = "@rpath/libonnxruntime.1.20.0.dylib";
const NATIVE_PILOT_ONNXRUNTIME_LOADER_RPATH: &str = "@loader_path/../lib";
const NATIVE_PILOT_RUNTIME_LIBRARY_MAX_BYTES: u64 = 256 * 1024 * 1024;
const NATIVE_PILOT_RUNTIME_EXECUTABLE_MAX_BYTES: u64 = 512 * 1024 * 1024;
const NATIVE_PILOT_OTOOL_MAX_BYTES: usize = 2 * 1024 * 1024;
pub(crate) const CLAUDE_ALLOWED_TOOLS: &str = concat!(
    "Read,mcp__engram__harness,mcp__engram__memory,",
    "mcp__engram__obligations,mcp__engram__orient,mcp__engram__repo,mcp__engram__search"
);

/// Memory system available to one controlled comparison arm.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum MemoryLayer {
    /// Host-native memory only.
    Native,
    /// Engram only, with host-native memory disabled.
    Engram,
    /// Engram and host-native memory together.
    Both,
}

/// Authentication contract used by every isolated Codex lane.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodexAuthenticationMode {
    /// Interactive ChatGPT browser login with credentials stored in the OS keyring.
    ChatgptBrowserKeyring,
    /// ChatGPT device-code login with credentials stored in the OS keyring.
    ChatgptDeviceKeyring,
    /// ChatGPT credentials read from an owner-only `auth.json` in the isolated Codex home.
    ChatgptFileCache,
}

impl CodexAuthenticationMode {
    pub(crate) const fn credential_store(self) -> &'static str {
        match self {
            Self::ChatgptBrowserKeyring | Self::ChatgptDeviceKeyring => "keyring",
            Self::ChatgptFileCache => "file",
        }
    }

    pub(crate) const fn credential_store_config(self) -> &'static str {
        match self {
            Self::ChatgptBrowserKeyring | Self::ChatgptDeviceKeyring => CODEX_KEYRING_CONFIG,
            Self::ChatgptFileCache => CODEX_FILE_CONFIG,
        }
    }
}

/// Host-visible resource limits declared before native-pilot provider execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NativePilotResourceBudgets {
    /// Maximum serialized bytes returned by any one Engram tool call.
    pub max_engram_result_bytes_per_call: u64,
    /// Maximum serialized Engram result bytes returned during one evaluation lane.
    pub max_engram_result_bytes_per_lane: u64,
    /// Maximum additional host-reported input plus output tokens over the matched native lane.
    pub max_incremental_total_tokens_per_lane: u64,
    /// Maximum additional runner-observed provider time over the matched native lane.
    pub max_incremental_runner_duration_ms_per_lane: u64,
}

/// Frozen task outcome expected after the fresh-session applicability check.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NativePilotExpectedOutcome {
    /// The learned procedure is applicable and must execute successfully.
    #[default]
    ExecuteProcedure,
    /// Current conditions make the learned procedure inapplicable, so execution must abstain.
    Abstain,
}

/// Project-authorization state expected from the structured identity boundary.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NativePilotExpectedProjectStatus {
    /// A caller, task, or correlated project link authorized the project identity.
    Authorized,
    /// Repository identity is known, but selecting a project requires user confirmation.
    RequiresConfirmation,
    /// Project authorization could not be evaluated.
    Unavailable,
}

impl MemoryLayer {
    pub(crate) fn uses_native(self) -> bool {
        matches!(self, Self::Native | Self::Both)
    }

    pub(crate) fn uses_engram(self) -> bool {
        matches!(self, Self::Engram | Self::Both)
    }
}

/// One host/layer arm in the native-memory comparison matrix.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NativePilotArm {
    /// Stable arm identifier.
    pub id: String,
    /// Native harness: `codex` or `claude_code`.
    pub host: String,
    /// Durable memory layer available in both teaching and evaluation.
    pub memory_layer: MemoryLayer,
}

/// Procedure facts taught through a real provider session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LearnedProcedure {
    /// Stable semantic key the teaching session should preserve.
    pub context_key: String,
    /// Task wording used for procedure retrieval.
    pub task: String,
    /// Known failed commands that must not be repeated during evaluation.
    pub failed_commands: Vec<String>,
    /// Correct command learned after the failed attempt.
    pub command: String,
    /// Expected successful process exit code.
    pub expected_exit_code: i32,
    /// Marker proving successful execution.
    pub expected_output_contains: String,
    /// Exact environment prerequisites for applicability.
    #[serde(default)]
    pub conditions: BTreeMap<String, String>,
    /// Declarative current-checkout sources for exact prerequisite keys in schema 8 and later.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub prerequisite_sources: BTreeMap<String, NativePilotPrerequisiteSource>,
}

/// Declarative source for one learned procedure prerequisite.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NativePilotPrerequisiteSource {
    /// Source format. Schema 8 supports `toml` only.
    pub format: String,
    /// Safe path relative to the resolved checkout root.
    pub relative_path: String,
    /// TOML table/key names leading to one scalar value.
    pub key_path: Vec<String>,
}

/// Expected value-redacted condition observation from `procedure_match`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NativePilotConditionObservation {
    /// Prerequisite key resolved from the declarative source.
    pub condition_key: String,
    /// Frozen source descriptor.
    pub source: NativePilotPrerequisiteSource,
    /// Expected applicability status.
    pub status: NativePilotConditionObservationStatus,
    /// SHA-256 of the evaluation-checkout source bytes.
    pub source_sha256: Option<String>,
    /// Exact source excerpt required only for native-host direct-read parity.
    pub source_output_contains: Option<String>,
    /// Bounded redacted detail required for an unavailable observation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail_contains: Option<String>,
}

/// Frozen source-backed applicability outcome.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NativePilotConditionObservationStatus {
    /// The current source matches the verified prerequisite.
    Matched,
    /// The current source differs from the verified prerequisite.
    Mismatched,
    /// The current source cannot be inspected safely.
    Unavailable,
}

/// Deterministic evaluation-checkout state for one source-backed prerequisite.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NativePilotEvaluationPrerequisiteState {
    /// Leave the materialized evaluation checkout unchanged.
    #[default]
    Unchanged,
    /// Remove the tracked source from the evaluation worktree after materialization.
    Missing,
}

/// One teaching/evaluation transfer case.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NativePilotCase {
    /// Stable case identifier.
    pub id: String,
    /// Fixture URI used for the teaching session.
    pub teaching_cwd: String,
    /// Distinct fixture URI used for the fresh evaluation session.
    pub evaluation_cwd: String,
    /// User-facing evaluation request. It must not reveal learned values.
    pub evaluation_prompt: String,
    /// Expected normalized repository remote in the evaluation checkout.
    pub expected_repository_remote: String,
    /// Expected project identity. `None` means the agent must report unresolved project ambiguity.
    pub expected_project: Option<String>,
    /// Expected structured project-authorization status in schema 12 and later.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_project_status: Option<NativePilotExpectedProjectStatus>,
    /// Whether schema 12 and later must explicitly request project confirmation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_project_confirmation_required: Option<bool>,
    /// Expected component identity. `None` means no component should be resolved.
    pub expected_component: Option<String>,
    /// Expected first externally visible action label.
    pub expected_first_action: String,
    /// Whether current checkout conditions require execution or safe abstention.
    #[serde(default)]
    pub expected_outcome: NativePilotExpectedOutcome,
    /// Checkout-local evidence target that must be inspected before expected abstention.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition_evidence_target: Option<String>,
    /// Exact excerpt a successful condition-source read must return before expected abstention.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition_evidence_output_contains: Option<String>,
    /// Maximum Unicode-scalar length of the task-focused procedure retrieval query in schema 15.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub procedure_query_max_chars: Option<u32>,
    /// Concrete task terms that the bounded procedure retrieval query must preserve in schema 15.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub procedure_query_required_terms: Vec<String>,
    /// Controlled source state used only by schema 9 unavailable-source cases.
    #[serde(default, skip_serializing_if = "is_unchanged_prerequisite_state")]
    pub evaluation_prerequisite_state: NativePilotEvaluationPrerequisiteState,
    /// Short verified lifetime used only by schema 10 expiry cases.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub procedure_expiry_seconds: Option<u64>,
    /// Checkout-relative source that must prove a positive project identity in schema 7.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_evidence_target: Option<String>,
    /// Exact excerpt the project-identity source read must return in schema 7.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_evidence_output_contains: Option<String>,
    /// Checkout-relative source that must prove a positive component identity in schema 7.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component_evidence_target: Option<String>,
    /// Exact excerpt the component-identity source read must return in schema 7.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component_evidence_output_contains: Option<String>,
    /// Frozen learned procedure and acceptance contract.
    pub procedure: LearnedProcedure,
}

/// One preregistered lane in fixed execution order.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct NativePilotRunRef {
    /// Case identifier.
    pub case_id: String,
    /// Arm identifier.
    pub arm: String,
    /// One-based repetition.
    pub repetition: u32,
}

/// Frozen native-memory comparison protocol.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NativePilotProtocol {
    /// Protocol schema version.
    pub schema_version: u32,
    /// Stable pilot identifier.
    pub pilot_id: String,
    /// Transfer cases declared before any provider execution.
    pub cases: Vec<NativePilotCase>,
    /// Required host/layer comparison matrix.
    pub arms: Vec<NativePilotArm>,
    /// Required repetitions for each case/arm pair.
    pub repetitions: u32,
    /// Fixed mixed lane order.
    pub run_order: Vec<NativePilotRunRef>,
    /// Minimum idle interval before Codex memory activation.
    pub codex_min_idle_hours: u32,
    /// Provider authentication mode shared by the isolated Codex lanes.
    pub codex_authentication_mode: CodexAuthenticationMode,
    /// Optional preregistered resource limits. Historical plans without these remain readable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_budgets: Option<NativePilotResourceBudgets>,
    /// Aggregate Claude Code budget across teaching and evaluation calls.
    pub claude_budget_cents: u32,
    /// Provider-reported Claude spend from invalidated predecessor attempts, rounded up.
    pub claude_prior_spend_microusd: u64,
    /// Operator-approved cumulative Claude ceiling across predecessor and replacement plans.
    pub claude_authorized_ceiling_cents: u32,
    /// Exact Claude model shared by all Claude Code lanes.
    pub claude_model: String,
    /// Maximum agentic turns in each Claude Code call.
    pub claude_max_turns: u32,
    /// Provider calls remain disabled until the user explicitly approves them.
    pub requires_explicit_execution_approval: bool,
}

/// Artifact observed from a selected memory system or required from evaluation infrastructure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NativeArtifactGate {
    /// Memory layer that owns this artifact.
    pub layer: String,
    /// Expected file or directory path.
    pub path: String,
    /// Whether absence after the producing phase invalidates the lane.
    #[serde(default = "required_artifact_by_default")]
    pub required: bool,
    /// Condition required before evaluation can start.
    pub requirement: String,
}

const fn required_artifact_by_default() -> bool {
    true
}

fn is_unchanged_prerequisite_state(state: &NativePilotEvaluationPrerequisiteState) -> bool {
    *state == NativePilotEvaluationPrerequisiteState::Unchanged
}

/// Frozen, secret-free authentication commands for one isolated Codex lane.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PreparedCodexAuthentication {
    /// Authentication mode required by the protocol.
    pub mode: CodexAuthenticationMode,
    /// Codex credential-store selector: OS `keyring` or lane-local owner-only `file`.
    pub credential_store: String,
    /// Exact interactive ChatGPT-login argv. The runner never invokes it.
    pub login_argv: Vec<String>,
    /// Exact provider-free login-status argv invoked by readiness checks and the runner.
    pub status_argv: Vec<String>,
    /// Exact successful status required before provider execution.
    pub expected_status: String,
}

/// Prepared, but never executed, teaching/evaluation lane.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PreparedNativeLane {
    /// One-based fixed execution order.
    pub order: u32,
    /// Case identifier.
    pub case_id: String,
    /// Arm identifier.
    pub arm: String,
    /// Host harness.
    pub host: String,
    /// Durable memory layer.
    pub memory_layer: MemoryLayer,
    /// One-based repetition.
    pub repetition: u32,
    /// Path-independent fixture content revision.
    pub fixture_revision: String,
    /// Teaching checkout cwd.
    pub teaching_cwd: String,
    /// Fresh moved-checkout evaluation cwd.
    pub evaluation_cwd: String,
    /// Isolated environment values that are safe to record.
    pub environment: BTreeMap<String, String>,
    /// Secret environment names required at execution time; values are never read or recorded.
    #[serde(default)]
    pub required_secret_environment: Vec<String>,
    /// Secret-free Codex authentication contract. Absent for non-Codex lanes and legacy plans.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codex_authentication: Option<PreparedCodexAuthentication>,
    /// Exact Bash commands pre-approved only for Claude's controlled teaching phase.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub claude_teaching_bash_commands: Vec<String>,
    /// Exact provider teaching argv.
    pub teaching_argv: Vec<String>,
    /// Exact matched activation argv. Codex lanes use this after the idle interval.
    pub activation_argv: Option<Vec<String>>,
    /// Trusted evaluator command template that verifies an Engram procedure candidate.
    pub post_teaching_verification_argv: Option<Vec<String>>,
    /// Minimum wait before activation, in hours.
    pub activation_wait_hours: u32,
    /// Exact fresh-session evaluation argv.
    pub evaluation_argv: Vec<String>,
    /// Generated-memory observations and required state gates checked before evaluation.
    pub artifact_gates: Vec<NativeArtifactGate>,
    /// Frozen acceptance contract, kept outside the evaluation checkout.
    pub acceptance_contract: String,
    /// Native adapter content hash for Engram arms.
    pub adapter_sha256: Option<String>,
    /// Engram isolated state root, when used.
    pub engram_home: Option<String>,
    /// Engram daemon project, when used.
    pub engram_project: Option<String>,
    /// Cleanup command for the isolated Engram daemon, when used.
    pub cleanup_argv: Option<Vec<String>>,
    /// Future raw teaching trace path.
    pub teaching_trace_path: String,
    /// Future raw activation trace path.
    pub activation_trace_path: Option<String>,
    /// Future raw evaluation trace path.
    pub evaluation_trace_path: String,
    /// Future trusted CLI procedure-verification output path.
    pub post_teaching_verification_output_path: Option<String>,
    /// Future structured evaluation result path.
    pub agent_output_path: String,
}

/// Provenance for an evaluation-only successor after an exact host-boundary rejection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PreparedNativeEvaluationRecovery {
    /// Immutable source plan whose teaching and activation evidence is reused.
    pub source_run_plan: String,
    /// SHA-256 of the immutable source plan.
    pub source_run_plan_sha256: String,
    /// Immutable source evaluation trace at the rejected host boundary.
    pub rejected_evaluation_trace: String,
    /// SHA-256 of the rejected evaluation trace.
    pub rejected_evaluation_trace_sha256: String,
    /// Exact stderr artifact captured alongside the rejected trace.
    pub rejected_evaluation_stderr: String,
    /// SHA-256 of the immutable rejection stderr.
    pub rejected_evaluation_stderr_sha256: String,
    /// Exact frozen rejection classification.
    pub rejection_reason: String,
    /// Successor preparation time; the inherited artifact freshness baseline remains unchanged.
    pub recovery_prepared_unix_ms: u64,
    /// Remaining Claude evaluation calls covered by this successor allocation.
    pub claude_remaining_calls: u32,
    /// Portable schema emitted for both host output boundaries.
    pub agent_output_schema: String,
}

/// One immutable file inherited by a lifecycle-recovery successor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PreparedNativeInheritedFile {
    /// Canonical path to the provider or trusted-evaluator artifact.
    pub path: String,
    /// SHA-256 of the inherited bytes.
    pub sha256: String,
}

/// Provenance for a successor that resumes an interrupted lifecycle phase.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PreparedNativeExecutionRecovery {
    /// Immutable source plan whose completed lane outputs are inherited.
    pub source_run_plan: String,
    /// SHA-256 of the immutable source plan.
    pub source_run_plan_sha256: String,
    /// Exact lifecycle phase interrupted after one or more completed lanes.
    pub interrupted_phase: String,
    /// Local failure classification that caused the successor to be prepared.
    pub failure_reason: String,
    /// Successor preparation time; artifact freshness still uses the source preparation time.
    pub recovery_prepared_unix_ms: u64,
    /// Lanes whose completed provider calls must never be replayed.
    pub completed_lane_orders: Vec<u32>,
    /// Immutable traces and trusted evaluator outputs inherited from those lanes.
    pub inherited_files: Vec<PreparedNativeInheritedFile>,
    /// Claude calls still covered by the successor allocation.
    pub claude_remaining_calls: u32,
}

/// Agent-profile contract observed through an isolated real stdio MCP handshake.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EngramMcpRuntimeAttestation {
    pub verified: bool,
    pub mcp_tool_count: usize,
    pub mcp_tools_sha256: String,
    pub profile_instructions_sha256: String,
    pub restricted_tool_rejected: bool,
    pub review_authority_rejected: bool,
    /// Schema-v4 proof that the agent profile can create only inactive correction proposals.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correction_proposal_path_verified: Option<bool>,
    /// Schema-v4 proof that direct correction is unavailable to the agent profile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direct_correction_unavailable: Option<bool>,
    /// Schema-v4 proof that applying a proposal is unavailable to the agent profile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correction_apply_unavailable: Option<bool>,
    /// Schema-v4 proof that procedure-correction verification is unavailable to the agent profile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correction_verification_unavailable: Option<bool>,
    /// Schema-v4 proof that proposal inspection is unavailable to the agent profile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correction_inspection_unavailable: Option<bool>,
}

/// Effective agent-profile MCP contract reported by the exact Engram executable.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EngramMcpContractAttestation {
    /// Contract attestation document schema.
    pub schema_version: u32,
    /// Engram CLI package version.
    pub cli_version: String,
    /// Daemon health document schema supported by the binary.
    pub health_schema_version: u32,
    /// MCP contract version supported by the binary.
    pub mcp_contract_version: u32,
    /// MCP protocol version supported by the binary.
    pub mcp_protocol_version: String,
    /// Effective proxy profile.
    pub profile: String,
    /// Number of tools exposed to the host.
    pub mcp_tool_count: usize,
    /// SHA-256 of the effective host-facing tool declarations.
    pub mcp_tools_sha256: String,
    /// SHA-256 of the effective host-facing initialization instructions.
    pub profile_instructions_sha256: Option<String>,
    /// Contract observed through an isolated real stdio MCP handshake.
    pub effective_runtime: Option<EngramMcpRuntimeAttestation>,
    /// Canonical executable path that produced the report.
    pub executable_path: String,
    /// SHA-256 of the executable bytes that produced the report.
    pub executable_sha256: String,
}

/// Fully prepared native-memory pilot. Preparation never invokes a provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PreparedNativePilot {
    /// Frozen protocol semantics. Legacy plans without this field use schema 4 exactly.
    #[serde(default = "legacy_native_pilot_schema_version")]
    pub protocol_schema_version: u32,
    /// Stable pilot identifier.
    pub pilot_id: String,
    /// Always false; preparation cannot authorize provider execution.
    pub execution_approved: bool,
    /// Preparation timestamp used as the generated-artifact freshness baseline.
    pub prepared_unix_ms: u64,
    /// Minimum Codex idle interval frozen by the protocol.
    pub codex_min_idle_hours: u32,
    /// Preregistered host-visible resource limits, when declared by the protocol.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_budgets: Option<NativePilotResourceBudgets>,
    /// Aggregate Claude Code runner allocation.
    pub claude_budget_cents: u32,
    /// Provider-reported Claude spend from invalidated predecessor attempts, rounded up.
    #[serde(default)]
    pub claude_prior_spend_microusd: u64,
    /// Operator-approved cumulative Claude ceiling across predecessor and replacement plans.
    #[serde(default)]
    pub claude_authorized_ceiling_cents: u32,
    /// Exact Claude model shared by all Claude Code lanes.
    #[serde(default)]
    pub claude_model: String,
    /// Maximum agentic turns in each Claude Code call.
    #[serde(default)]
    pub claude_max_turns: u32,
    /// Provider lifecycle documentation used to design the protocol.
    pub native_memory_references: BTreeMap<String, String>,
    /// Attested local executable identities.
    pub binaries: BTreeMap<String, PilotBinaryAttestation>,
    /// Loader-closed runtime-library identities. Required exactly for stale-safety family v3.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub runtime_libraries: BTreeMap<String, NativePilotRuntimeLibraryAttestation>,
    /// Agent-profile MCP contract emitted by the attested Engram executable.
    pub engram_mcp_contract: EngramMcpContractAttestation,
    /// Evaluation-only recovery provenance. Absent from ordinary prepared plans.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evaluation_recovery: Option<PreparedNativeEvaluationRecovery>,
    /// Lifecycle recovery provenance. Absent from ordinary and evaluation-only plans.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_recovery: Option<PreparedNativeExecutionRecovery>,
    /// Frozen stale-safety capability binding. Absent from ordinary and historical plans.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stale_safety: Option<PreparedNativeStaleSafetyBinding>,
    /// Fixed isolated teaching/evaluation lanes.
    pub lanes: Vec<PreparedNativeLane>,
}

/// One immutable native library shared by one or more loader-bound executable consumers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NativePilotRuntimeLibraryAttestation {
    pub install_name: String,
    pub sha256: String,
    pub byte_length: u64,
    pub consumers: BTreeMap<String, NativePilotRuntimeLibraryConsumerBinding>,
}

/// Exact loader edge and resolved sibling file used by one executable consumer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NativePilotRuntimeLibraryConsumerBinding {
    pub executable_path: String,
    pub dependency_install_name: String,
    pub loader_rpath: String,
    pub resolved_library_path: String,
    pub resolved_library_sha256: String,
    pub resolved_library_identity: NativePilotRuntimeFileIdentity,
}

/// Stable Unix identity for one held, owner-private runtime-library file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NativePilotRuntimeFileIdentity {
    pub device: u64,
    pub inode: u64,
    pub uid: u32,
    pub gid: u32,
    pub mode: u32,
    pub link_count: u64,
    pub byte_length: u64,
    pub modified_unix_seconds: i64,
    pub modified_nanoseconds: i64,
    pub changed_unix_seconds: i64,
    pub changed_nanoseconds: i64,
}

#[cfg(target_os = "macos")]
fn native_pilot_runtime_file_identity(metadata: &fs::Metadata) -> NativePilotRuntimeFileIdentity {
    use std::os::unix::fs::MetadataExt;

    NativePilotRuntimeFileIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
        uid: metadata.uid(),
        gid: metadata.gid(),
        mode: metadata.mode() & 0o7777,
        link_count: metadata.nlink(),
        byte_length: metadata.len(),
        modified_unix_seconds: metadata.mtime(),
        modified_nanoseconds: metadata.mtime_nsec(),
        changed_unix_seconds: metadata.ctime(),
        changed_nanoseconds: metadata.ctime_nsec(),
    }
}

#[cfg(target_os = "macos")]
fn open_native_pilot_runtime_directory(
    path: &Path,
    label: &str,
) -> EvalResult<(File, NativePilotRuntimeFileIdentity)> {
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};

    if !path.is_absolute() || path.canonicalize()? != path {
        return Err(EvalError::Invalid(format!(
            "{label} is not one canonical absolute directory"
        )));
    }
    let before = fs::symlink_metadata(path)?;
    if before.file_type().is_symlink()
        || !before.is_dir()
        || before.uid() != unsafe { libc::geteuid() }
        || before.permissions().mode() & 0o777 != 0o700
    {
        return Err(EvalError::Invalid(format!(
            "{label} is not an owner-private 0700 directory"
        )));
    }
    let mut options = OpenOptions::new();
    options
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC);
    let file = options.open(path)?;
    let opened = file.metadata()?;
    let before = native_pilot_runtime_file_identity(&before);
    let opened = native_pilot_runtime_file_identity(&opened);
    if before != opened {
        return Err(EvalError::Invalid(format!(
            "{label} changed while its directory descriptor was opened"
        )));
    }
    Ok((file, opened))
}

#[cfg(target_os = "macos")]
fn revalidate_native_pilot_runtime_path(
    path: &Path,
    descriptor: &File,
    expected: &NativePilotRuntimeFileIdentity,
    label: &str,
) -> EvalResult<()> {
    let path_metadata = fs::symlink_metadata(path)?;
    if path_metadata.file_type().is_symlink()
        || native_pilot_runtime_file_identity(&path_metadata) != *expected
        || native_pilot_runtime_file_identity(&descriptor.metadata()?) != *expected
        || path.canonicalize()? != path
    {
        return Err(EvalError::Invalid(format!(
            "{label} changed during runtime-closure attestation"
        )));
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn open_native_pilot_runtime_file(
    path: &Path,
    label: &str,
    expected_mode: u32,
    max_bytes: u64,
    require_executable: bool,
) -> EvalResult<(File, NativePilotRuntimeFileIdentity)> {
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};

    if !path.is_absolute() || path.canonicalize()? != path {
        return Err(EvalError::Invalid(format!(
            "{label} is not one canonical absolute file"
        )));
    }
    let before = fs::symlink_metadata(path)?;
    let permissions = before.permissions().mode() & 0o777;
    if before.file_type().is_symlink()
        || !before.is_file()
        || before.uid() != unsafe { libc::geteuid() }
        || permissions != expected_mode
        || require_executable && permissions & 0o100 == 0
        || before.nlink() != 1
        || before.len() == 0
        || before.len() > max_bytes
    {
        return Err(EvalError::Invalid(format!(
            "{label} is not the exact owner-private single-link regular file"
        )));
    }
    let mut options = OpenOptions::new();
    options
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    let file = options.open(path)?;
    let before = native_pilot_runtime_file_identity(&before);
    let opened = native_pilot_runtime_file_identity(&file.metadata()?);
    if before != opened {
        return Err(EvalError::Invalid(format!(
            "{label} changed while its file descriptor was opened"
        )));
    }
    Ok((file, opened))
}

#[cfg(target_os = "macos")]
fn run_native_pilot_otool(arguments: &[&str], label: &str) -> EvalResult<String> {
    let output = Command::new("/usr/bin/otool")
        .args(arguments)
        .env_clear()
        .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .output()?;
    if output.stdout.len() > NATIVE_PILOT_OTOOL_MAX_BYTES
        || output.stderr.len() > NATIVE_PILOT_OTOOL_MAX_BYTES
        || !output.status.success()
        || !output.stderr.is_empty()
    {
        return Err(EvalError::Invalid(format!(
            "{label} Mach-O inspection failed its bounded clean-output contract"
        )));
    }
    String::from_utf8(output.stdout)
        .map_err(|_| EvalError::Invalid(format!("{label} Mach-O output is not UTF-8")))
}

#[cfg(target_os = "macos")]
fn validate_native_pilot_runtime_loader_outputs(
    linked: &str,
    load: &str,
    install: &str,
    library_text: &str,
) -> EvalResult<()> {
    let onnx_dependencies = linked
        .lines()
        .skip(1)
        .map(str::trim)
        .filter(|line| line.contains("onnxruntime"))
        .collect::<Vec<_>>();
    let install_lines = install
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    if onnx_dependencies.len() != 1
        || !onnx_dependencies[0].starts_with(&format!("{NATIVE_PILOT_ONNXRUNTIME_INSTALL_NAME} ("))
        || load.matches("cmd LC_RPATH").count() != 1
        || load
            .matches(&format!(
                "path {NATIVE_PILOT_ONNXRUNTIME_LOADER_RPATH} (offset "
            ))
            .count()
            != 1
        || install_lines.len() != 2
        || install_lines[0] != format!("{library_text}:")
        || install_lines[1] != NATIVE_PILOT_ONNXRUNTIME_INSTALL_NAME
    {
        return Err(EvalError::Invalid(
            "family-v3 executable/library pair lacks the exact loader-relative ONNX closure"
                .to_string(),
        ));
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn validate_native_pilot_runtime_loader_contract(
    executable: &Path,
    library: &Path,
) -> EvalResult<()> {
    let executable_text = executable.to_str().ok_or_else(|| {
        EvalError::Invalid("family-v3 runtime executable path is not UTF-8".to_string())
    })?;
    let library_text = library.to_str().ok_or_else(|| {
        EvalError::Invalid("family-v3 runtime library path is not UTF-8".to_string())
    })?;
    let linked = run_native_pilot_otool(&["-L", executable_text], "family-v3 executable")?;
    let load = run_native_pilot_otool(&["-l", executable_text], "family-v3 executable")?;
    let install = run_native_pilot_otool(&["-D", library_text], "family-v3 runtime library")?;
    validate_native_pilot_runtime_loader_outputs(&linked, &load, &install, library_text)
}

#[cfg(target_os = "macos")]
fn attest_native_pilot_runtime_consumer(
    executable: &Path,
    expected_name: &str,
) -> EvalResult<(String, u64, NativePilotRuntimeLibraryConsumerBinding)> {
    let executable = executable.canonicalize()?;
    if executable.file_name().and_then(|name| name.to_str()) != Some(expected_name)
        || executable
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            != Some("bin")
    {
        return Err(EvalError::Invalid(format!(
            "family-v3 {expected_name} must be the exact <bundle>/bin/{expected_name} executable"
        )));
    }
    let bin = executable.parent().expect("validated executable parent");
    let root = bin.parent().ok_or_else(|| {
        EvalError::Invalid(format!("family-v3 {expected_name} bundle has no root"))
    })?;
    let lib = root.join("lib");
    let library = lib.join(NATIVE_PILOT_ONNXRUNTIME_DYLIB);
    let (root_fd, root_identity) =
        open_native_pilot_runtime_directory(root, "family-v3 runtime root")?;
    let (bin_fd, bin_identity) =
        open_native_pilot_runtime_directory(bin, "family-v3 runtime bin directory")?;
    let (lib_fd, lib_identity) =
        open_native_pilot_runtime_directory(&lib, "family-v3 runtime lib directory")?;
    let executable_mode = fs::symlink_metadata(&executable)?.permissions();
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    let executable_mode = executable_mode.mode() & 0o777;
    if !matches!(executable_mode, 0o500 | 0o700) {
        return Err(EvalError::Invalid(format!(
            "family-v3 {expected_name} executable mode is not exactly 0500 or 0700"
        )));
    }
    let (executable_fd, executable_identity) = open_native_pilot_runtime_file(
        &executable,
        "family-v3 runtime executable",
        executable_mode,
        NATIVE_PILOT_RUNTIME_EXECUTABLE_MAX_BYTES,
        true,
    )?;
    let (mut library_fd, library_identity) = open_native_pilot_runtime_file(
        &library,
        "family-v3 ONNX runtime library",
        0o600,
        NATIVE_PILOT_RUNTIME_LIBRARY_MAX_BYTES,
        false,
    )?;
    let mut bytes = Vec::new();
    (&mut library_fd)
        .take(NATIVE_PILOT_RUNTIME_LIBRARY_MAX_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 != library_identity.byte_length {
        return Err(EvalError::Invalid(
            "family-v3 ONNX runtime changed during its bounded read".to_string(),
        ));
    }
    let sha256 = {
        let mut digest = Sha256::new();
        digest.update(&bytes);
        format!("{:x}", digest.finalize())
    };
    validate_native_pilot_runtime_loader_contract(&executable, &library)?;
    revalidate_native_pilot_runtime_path(root, &root_fd, &root_identity, "family-v3 runtime root")?;
    revalidate_native_pilot_runtime_path(
        bin,
        &bin_fd,
        &bin_identity,
        "family-v3 runtime bin directory",
    )?;
    revalidate_native_pilot_runtime_path(
        &lib,
        &lib_fd,
        &lib_identity,
        "family-v3 runtime lib directory",
    )?;
    revalidate_native_pilot_runtime_path(
        &executable,
        &executable_fd,
        &executable_identity,
        "family-v3 runtime executable",
    )?;
    revalidate_native_pilot_runtime_path(
        &library,
        &library_fd,
        &library_identity,
        "family-v3 ONNX runtime library",
    )?;
    Ok((
        sha256.clone(),
        library_identity.byte_length,
        NativePilotRuntimeLibraryConsumerBinding {
            executable_path: executable.display().to_string(),
            dependency_install_name: NATIVE_PILOT_ONNXRUNTIME_INSTALL_NAME.to_string(),
            loader_rpath: NATIVE_PILOT_ONNXRUNTIME_LOADER_RPATH.to_string(),
            resolved_library_path: library.display().to_string(),
            resolved_library_sha256: sha256,
            resolved_library_identity: library_identity,
        },
    ))
}

#[cfg(not(target_os = "macos"))]
fn attest_native_pilot_runtime_consumer(
    _executable: &Path,
    _expected_name: &str,
) -> EvalResult<(String, u64, NativePilotRuntimeLibraryConsumerBinding)> {
    Err(EvalError::Invalid(
        "family-v3 loader-closed runtime attestation requires macOS".to_string(),
    ))
}

/// Attest the exact ONNX runtime content plus its two loader-relative executable consumers.
pub(crate) fn attest_native_pilot_family_v3_runtime_libraries(
    engram: &Path,
    evaluator: &Path,
) -> EvalResult<BTreeMap<String, NativePilotRuntimeLibraryAttestation>> {
    let (engram_sha256, engram_length, engram_binding) =
        attest_native_pilot_runtime_consumer(engram, "engram")?;
    let (evaluator_sha256, evaluator_length, evaluator_binding) =
        attest_native_pilot_runtime_consumer(evaluator, "engram-eval")?;
    if engram_sha256 != evaluator_sha256 || engram_length != evaluator_length {
        return Err(EvalError::Invalid(
            "family-v3 Engram and evaluator do not resolve byte-identical ONNX runtimes"
                .to_string(),
        ));
    }
    Ok(BTreeMap::from([(
        NATIVE_PILOT_ONNXRUNTIME_LIBRARY_KEY.to_string(),
        NativePilotRuntimeLibraryAttestation {
            install_name: NATIVE_PILOT_ONNXRUNTIME_INSTALL_NAME.to_string(),
            sha256: engram_sha256,
            byte_length: engram_length,
            consumers: BTreeMap::from([
                (
                    NATIVE_PILOT_ENGRAM_RUNTIME_CONSUMER_KEY.to_string(),
                    engram_binding,
                ),
                (
                    NATIVE_PILOT_EVALUATOR_RUNTIME_CONSUMER_KEY.to_string(),
                    evaluator_binding,
                ),
            ]),
        },
    )]))
}

/// Re-attest the live family-v3 runtime closure and compare every path, byte and file identity.
pub(crate) fn validate_native_pilot_family_v3_runtime_libraries(
    binaries: &BTreeMap<String, PilotBinaryAttestation>,
    expected: &BTreeMap<String, NativePilotRuntimeLibraryAttestation>,
) -> EvalResult<()> {
    let engram = binaries
        .get(NATIVE_PILOT_ENGRAM_RUNTIME_CONSUMER_KEY)
        .ok_or_else(|| {
            EvalError::Invalid("family-v3 plan omitted its Engram executable".to_string())
        })?;
    let evaluator = binaries
        .get(NATIVE_PILOT_EVALUATOR_RUNTIME_CONSUMER_KEY)
        .ok_or_else(|| {
            EvalError::Invalid("family-v3 plan omitted its evaluator executable".to_string())
        })?;
    let observed = attest_native_pilot_family_v3_runtime_libraries(
        Path::new(&engram.path),
        Path::new(&evaluator.path),
    )?;
    if observed != *expected {
        return Err(EvalError::Invalid(
            "family-v3 loader-closed runtime library identity drifted".to_string(),
        ));
    }
    Ok(())
}

/// Run-plan-bound stale-safety artifacts required before any provider phase can execute.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PreparedNativeStaleSafetyBinding {
    pub family_version: u32,
    pub protocol_snapshot_file: String,
    pub protocol_snapshot_sha256: String,
    pub retention_precondition_file: String,
    pub retention_precondition_sha256: String,
    pub agent_output_schema_sha256: String,
}

/// Narrow provider-free input overrides for specialized native-pilot families.
///
/// The default native pilot does not use these. A specialized family can freeze opaque lane
/// paths and family-specific prompts, contracts, and output schema while retaining the same
/// fixture, binary-attestation, authentication, and lane construction code.
#[derive(Debug, Clone, Default)]
pub(crate) struct NativePilotPreparationOverrides {
    pub agent_output_schema: Option<String>,
    pub stale_safety: Option<PreparedNativeStaleSafetyBinding>,
    pub lanes: BTreeMap<u32, NativePilotLanePreparationOverride>,
}

/// Specialized inputs for one prepared lane. Callers validate these values before preparation.
#[derive(Debug, Clone, Default)]
pub(crate) struct NativePilotLanePreparationOverride {
    pub directory_name: Option<String>,
    pub teaching_prompt_suffix: Option<String>,
    pub evaluation_prompt_suffix: Option<String>,
    pub verification_expiry_seconds: Option<u64>,
    pub persist_codex_evaluation_session: bool,
    pub closed_world_provider_environment: bool,
    pub acceptance_contract_fields: BTreeMap<String, serde_json::Value>,
}

/// Compile an isolated native-memory pilot without executing Codex or Claude Code.
pub async fn prepare_native_memory_pilot(
    protocol_path: &Path,
    output: &Path,
    engram_binary: &Path,
    codex_binary: &Path,
    claude_binary: &Path,
) -> EvalResult<PreparedNativePilot> {
    prepare_native_memory_pilot_with_overrides(
        protocol_path,
        output,
        engram_binary,
        codex_binary,
        claude_binary,
        None,
    )
    .await
}

pub(crate) async fn prepare_native_memory_pilot_with_overrides(
    protocol_path: &Path,
    output: &Path,
    engram_binary: &Path,
    codex_binary: &Path,
    claude_binary: &Path,
    overrides: Option<&NativePilotPreparationOverrides>,
) -> EvalResult<PreparedNativePilot> {
    let protocol = load_historical_native_protocol(protocol_path)?;
    prepare_native_memory_pilot_from_protocol_with_overrides(
        protocol,
        output,
        engram_binary,
        codex_binary,
        claude_binary,
        overrides,
    )
    .await
}

pub(crate) async fn prepare_native_memory_pilot_from_protocol_with_overrides(
    protocol: NativePilotProtocol,
    output: &Path,
    engram_binary: &Path,
    codex_binary: &Path,
    claude_binary: &Path,
    overrides: Option<&NativePilotPreparationOverrides>,
) -> EvalResult<PreparedNativePilot> {
    validate_protocol(&protocol)?;
    require_empty_target(output)?;
    let output = output.canonicalize()?;

    let runner_path = std::env::current_exe().map_err(EvalError::Io)?;
    let family_v3 = overrides
        .and_then(|value| value.stale_safety.as_ref())
        .is_some_and(|binding| binding.family_version == 3);
    // The family-v3 Engram process is invoked below for version and MCP-contract probes. Close its
    // non-system loader dependency first; the evaluator's already-loaded mapping remains an
    // explicitly documented prelaunch boundary, while its sibling artifact is still bound here.
    let runtime_libraries = if family_v3 {
        attest_native_pilot_family_v3_runtime_libraries(engram_binary, &runner_path)?
    } else {
        BTreeMap::new()
    };
    let engram = attest_binary(engram_binary)?;
    if !engram.version.contains(env!("CARGO_PKG_VERSION")) {
        return Err(EvalError::Invalid(format!(
            "Engram binary version '{}' does not match evaluation crate {}",
            engram.version,
            env!("CARGO_PKG_VERSION")
        )));
    }
    let codex = attest_binary(codex_binary)?;
    let claude = attest_binary(claude_binary)?;
    let runner = attest_binary(&runner_path)?;
    let engram_mcp_contract = attest_engram_mcp_contract(&engram)?;
    let mut binaries = BTreeMap::from([
        ("claude_code".to_string(), claude.clone()),
        ("codex".to_string(), codex.clone()),
        ("engram".to_string(), engram.clone()),
        ("engram_eval".to_string(), runner),
    ]);
    if protocol.codex_authentication_mode == CodexAuthenticationMode::ChatgptFileCache {
        binaries.insert(
            "codex_code_mode_host".to_string(),
            attest_codex_code_mode_host(&codex)?,
        );
    }
    if family_v3 {
        validate_native_pilot_family_v3_runtime_libraries(&binaries, &runtime_libraries)?;
    }

    create_private_dir_all(&output)?;
    let schema_path = output.join("agent-output.schema.json");
    let agent_output_schema = match overrides.and_then(|value| value.agent_output_schema.as_ref()) {
        Some(schema) => {
            let _: serde_json::Value = serde_json::from_str(schema).map_err(|error| {
                EvalError::Invalid(format!(
                    "specialized native-pilot output schema is invalid JSON: {error}"
                ))
            })?;
            schema.clone()
        }
        None => native_agent_output_schema(&protocol)?,
    };
    fs::write(&schema_path, agent_output_schema)?;
    fs::write(
        output.join("protocol.snapshot.json"),
        serde_json::to_string_pretty(&protocol)? + "\n",
    )?;

    let arms = protocol
        .arms
        .iter()
        .map(|arm| (arm.id.as_str(), arm))
        .collect::<BTreeMap<_, _>>();
    let cases = protocol
        .cases
        .iter()
        .map(|case| (case.id.as_str(), case))
        .collect::<BTreeMap<_, _>>();
    let claude_calls = protocol
        .run_order
        .iter()
        .filter(|run| arms[run.arm.as_str()].host == "claude_code")
        .count()
        * 2;
    let claude_budget_millis = if claude_calls == 0 {
        0
    } else {
        u64::from(protocol.claude_budget_cents) * 10 / claude_calls as u64
    };

    let mut lanes = Vec::with_capacity(protocol.run_order.len());
    for (index, run_ref) in protocol.run_order.iter().enumerate() {
        let order = u32::try_from(index + 1)
            .map_err(|_| EvalError::Invalid("native pilot has too many lanes".to_string()))?;
        lanes.push(prepare_lane(
            &output,
            order,
            run_ref,
            arms[run_ref.arm.as_str()],
            cases[run_ref.case_id.as_str()],
            &engram,
            &codex,
            &claude,
            &schema_path,
            protocol.schema_version,
            protocol.codex_min_idle_hours,
            protocol.codex_authentication_mode,
            &protocol.claude_model,
            protocol.claude_max_turns,
            claude_budget_millis,
            overrides.and_then(|overrides| overrides.lanes.get(&order)),
        )?);
    }

    let prepared = PreparedNativePilot {
        protocol_schema_version: protocol.schema_version,
        pilot_id: protocol.pilot_id,
        execution_approved: false,
        prepared_unix_ms: system_time_unix_ms(SystemTime::now())?,
        codex_min_idle_hours: protocol.codex_min_idle_hours,
        resource_budgets: protocol.resource_budgets,
        claude_budget_cents: protocol.claude_budget_cents,
        claude_prior_spend_microusd: protocol.claude_prior_spend_microusd,
        claude_authorized_ceiling_cents: protocol.claude_authorized_ceiling_cents,
        claude_model: protocol.claude_model,
        claude_max_turns: protocol.claude_max_turns,
        native_memory_references: BTreeMap::from([
            (
                "claude_code".to_string(),
                "https://code.claude.com/docs/en/memory".to_string(),
            ),
            (
                "codex".to_string(),
                "https://learn.chatgpt.com/docs/customization/memories".to_string(),
            ),
        ]),
        binaries,
        runtime_libraries,
        engram_mcp_contract,
        evaluation_recovery: None,
        execution_recovery: None,
        stale_safety: overrides.and_then(|value| value.stale_safety.clone()),
        lanes,
    };
    let run_plan_path = output.join("run-plan.json");
    fs::write(
        &run_plan_path,
        serde_json::to_string_pretty(&prepared)? + "\n",
    )?;
    fs::write(
        output.join("run-plan.sha256"),
        format!("{}\n", sha256_file(&run_plan_path)?),
    )?;
    Ok(prepared)
}

pub(crate) fn attest_engram_mcp_contract(
    engram: &PilotBinaryAttestation,
) -> EvalResult<EngramMcpContractAttestation> {
    let output = Command::new(&engram.path)
        .args([
            "contract",
            "--profile",
            "agent",
            "--verify-runtime",
            "--json",
        ])
        .output()?;
    if !output.status.success() {
        return Err(EvalError::Invalid(format!(
            "Engram agent-profile contract attestation failed with status {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    let contract: EngramMcpContractAttestation =
        serde_json::from_slice(&output.stdout).map_err(|error| {
            EvalError::Invalid(format!(
                "Engram agent-profile contract output is invalid JSON: {error}"
            ))
        })?;
    validate_engram_mcp_contract(&contract, engram)?;
    Ok(contract)
}

fn validate_engram_mcp_contract(
    contract: &EngramMcpContractAttestation,
    engram: &PilotBinaryAttestation,
) -> EvalResult<()> {
    let is_sha256 =
        |value: &str| value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit());
    let runtime_matches = contract.effective_runtime.as_ref().is_some_and(|runtime| {
        runtime.verified
            && runtime.restricted_tool_rejected
            && runtime.review_authority_rejected
            && runtime.mcp_tool_count == contract.mcp_tool_count
            && runtime.mcp_tools_sha256 == contract.mcp_tools_sha256
            && Some(runtime.profile_instructions_sha256.as_str())
                == contract.profile_instructions_sha256.as_deref()
    });
    let schema_shape_matches = match contract.schema_version {
        3 => contract.effective_runtime.as_ref().is_some_and(|runtime| {
            runtime.correction_proposal_path_verified.is_none()
                && runtime.direct_correction_unavailable.is_none()
                && runtime.correction_apply_unavailable.is_none()
                && runtime.correction_verification_unavailable.is_none()
                && runtime.correction_inspection_unavailable.is_none()
        }),
        4 => {
            contract.health_schema_version == 3
                && contract.mcp_contract_version == 5
                && contract.mcp_protocol_version == "2024-11-05"
                && contract.effective_runtime.as_ref().is_some_and(|runtime| {
                    runtime.correction_proposal_path_verified == Some(true)
                        && runtime.direct_correction_unavailable == Some(true)
                        && runtime.correction_apply_unavailable == Some(true)
                        && runtime.correction_verification_unavailable == Some(true)
                        && runtime.correction_inspection_unavailable == Some(true)
                })
        }
        _ => false,
    };
    if !schema_shape_matches
        || contract.cli_version != env!("CARGO_PKG_VERSION")
        || contract.profile != "agent"
        || contract.mcp_tool_count != 6
        || contract.health_schema_version == 0
        || contract.mcp_contract_version == 0
        || contract.mcp_protocol_version.is_empty()
        || !is_sha256(&contract.mcp_tools_sha256)
        || !contract
            .profile_instructions_sha256
            .as_deref()
            .is_some_and(is_sha256)
        || !runtime_matches
        || contract.executable_path != engram.path
        || contract.executable_sha256 != engram.sha256
    {
        return Err(EvalError::Invalid(
            "Engram executable reported an incompatible agent-profile MCP contract".to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn validate_protocol(protocol: &NativePilotProtocol) -> EvalResult<()> {
    validate_native_pilot_schema_version(protocol.schema_version)?;
    validate_native_pilot_resource_budgets(protocol.resource_budgets.as_ref())?;
    if protocol.schema_version >= STRUCTURED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION {
        let expected = NativePilotResourceBudgets {
            max_engram_result_bytes_per_call: 8_192,
            max_engram_result_bytes_per_lane: 16_384,
            max_incremental_total_tokens_per_lane: 50_000,
            max_incremental_runner_duration_ms_per_lane: 30_000,
        };
        if protocol.resource_budgets.as_ref() != Some(&expected) {
            return Err(EvalError::Invalid(format!(
                "native pilot schema {STRUCTURED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION} requires the frozen structured-identity resource budgets"
            )));
        }
    }
    if protocol.pilot_id.trim().is_empty() {
        return Err(EvalError::Invalid("pilot_id must not be empty".to_string()));
    }
    if protocol.repetitions == 0 {
        return Err(EvalError::Invalid(
            "native pilot repetitions must be positive".to_string(),
        ));
    }
    if protocol.codex_min_idle_hours < 1 {
        return Err(EvalError::Invalid(
            "Codex native-memory idle interval must be at least one hour".to_string(),
        ));
    }
    if !protocol.requires_explicit_execution_approval {
        return Err(EvalError::Invalid(
            "native pilot must require explicit provider execution approval".to_string(),
        ));
    }
    if protocol.cases.is_empty() {
        return Err(EvalError::Invalid(
            "native pilot must declare at least one transfer case".to_string(),
        ));
    }

    let case_ids = protocol
        .cases
        .iter()
        .map(|case| case.id.as_str())
        .collect::<BTreeSet<_>>();
    if case_ids.len() != protocol.cases.len() {
        return Err(EvalError::Invalid(
            "native pilot case IDs must be unique".to_string(),
        ));
    }
    for case in &protocol.cases {
        validate_case(case, protocol.schema_version)?;
        if case
            .procedure_expiry_seconds
            .is_some_and(|seconds| seconds >= u64::from(protocol.codex_min_idle_hours) * 60 * 60)
        {
            return Err(EvalError::Invalid(format!(
                "native pilot case {} procedure expiry must precede the frozen Codex activation interval",
                case.id
            )));
        }
    }

    let arm_ids = protocol
        .arms
        .iter()
        .map(|arm| arm.id.as_str())
        .collect::<BTreeSet<_>>();
    if arm_ids.len() != protocol.arms.len() {
        return Err(EvalError::Invalid(
            "native pilot arm IDs must be unique".to_string(),
        ));
    }
    let actual_matrix = protocol
        .arms
        .iter()
        .map(|arm| (arm.host.as_str(), arm.memory_layer))
        .collect::<BTreeSet<_>>();
    let expected_hosts =
        if protocol.schema_version == TARGETED_HOST_DIAGNOSTIC_NATIVE_PILOT_SCHEMA_VERSION {
            vec!["claude_code"]
        } else {
            vec!["codex", "claude_code"]
        };
    let expected_matrix = expected_hosts
        .into_iter()
        .flat_map(|host| {
            [MemoryLayer::Native, MemoryLayer::Engram, MemoryLayer::Both]
                .into_iter()
                .map(move |layer| (host, layer))
        })
        .collect::<BTreeSet<_>>();
    if actual_matrix != expected_matrix || protocol.arms.len() != expected_matrix.len() {
        let expected = if protocol.schema_version
            == TARGETED_HOST_DIAGNOSTIC_NATIVE_PILOT_SCHEMA_VERSION
        {
            "native pilot schema 11 arms must contain native, Engram, and combined layers for Claude Code only"
        } else {
            "native pilot arms must contain native, Engram, and combined layers for both Codex and Claude Code"
        };
        return Err(EvalError::Invalid(expected.to_string()));
    }

    let expected_runs = protocol
        .cases
        .iter()
        .flat_map(|case| {
            protocol.arms.iter().flat_map(move |arm| {
                (1..=protocol.repetitions).map(move |repetition| NativePilotRunRef {
                    case_id: case.id.clone(),
                    arm: arm.id.clone(),
                    repetition,
                })
            })
        })
        .collect::<BTreeSet<_>>();
    let actual_runs = protocol.run_order.iter().cloned().collect::<BTreeSet<_>>();
    if actual_runs.len() != protocol.run_order.len() {
        return Err(EvalError::Invalid(
            "native pilot run_order contains duplicate lanes".to_string(),
        ));
    }
    if actual_runs != expected_runs {
        return Err(EvalError::Invalid(
            "native pilot run_order must contain the complete case/arm/repetition matrix"
                .to_string(),
        ));
    }
    if protocol.arms.iter().any(|arm| arm.host == "claude_code")
        && protocol.claude_budget_cents == 0
    {
        return Err(EvalError::Invalid(
            "Claude Code lanes require a positive aggregate budget".to_string(),
        ));
    }
    let requested_microusd = u64::from(protocol.claude_budget_cents) * 10_000;
    let ceiling_microusd = u64::from(protocol.claude_authorized_ceiling_cents) * 10_000;
    let accounted_microusd = protocol
        .claude_prior_spend_microusd
        .checked_add(requested_microusd)
        .ok_or_else(|| EvalError::Invalid("Claude budget accounting overflow".to_string()))?;
    if accounted_microusd > ceiling_microusd {
        return Err(EvalError::Invalid(
            "Claude predecessor spend plus replacement allocation exceeds the authorized ceiling"
                .to_string(),
        ));
    }
    if protocol.claude_model.trim().is_empty() || protocol.claude_max_turns == 0 {
        return Err(EvalError::Invalid(
            "Claude Code lanes require an explicit model and positive max-turn limit".to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn validate_native_pilot_resource_budgets(
    budgets: Option<&NativePilotResourceBudgets>,
) -> EvalResult<()> {
    let Some(budgets) = budgets else {
        return Ok(());
    };
    if budgets.max_engram_result_bytes_per_call == 0
        || budgets.max_engram_result_bytes_per_lane == 0
        || budgets.max_incremental_total_tokens_per_lane == 0
        || budgets.max_incremental_runner_duration_ms_per_lane == 0
        || budgets.max_engram_result_bytes_per_call > budgets.max_engram_result_bytes_per_lane
    {
        return Err(EvalError::Invalid(
            "native pilot resource budgets must be positive and the per-call Engram limit must not exceed the per-lane limit"
                .to_string(),
        ));
    }
    Ok(())
}

const fn legacy_native_pilot_schema_version() -> u32 {
    LEGACY_NATIVE_PILOT_SCHEMA_VERSION
}

pub(crate) fn validate_native_pilot_schema_version(version: u32) -> EvalResult<()> {
    if matches!(
        version,
        LEGACY_NATIVE_PILOT_SCHEMA_VERSION
            | IDENTITY_NATIVE_PILOT_SCHEMA_VERSION
            | ABSTENTION_NATIVE_PILOT_SCHEMA_VERSION
            | TRUSTED_SCOPED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION
            | SOURCE_BACKED_PREREQUISITE_NATIVE_PILOT_SCHEMA_VERSION
            | UNAVAILABLE_PREREQUISITE_NATIVE_PILOT_SCHEMA_VERSION
            | EXPIRED_PROCEDURE_NATIVE_PILOT_SCHEMA_VERSION
            | TARGETED_HOST_DIAGNOSTIC_NATIVE_PILOT_SCHEMA_VERSION
            | STRUCTURED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION
            | OPERATION_EVIDENCE_ROUTE_NATIVE_PILOT_SCHEMA_VERSION
            | SOURCE_BOUND_OPERATION_EVIDENCE_NATIVE_PILOT_SCHEMA_VERSION
            | REQUIRED_HOST_ACTION_NATIVE_PILOT_SCHEMA_VERSION
    ) {
        return Ok(());
    }
    Err(EvalError::Invalid(format!(
        "unsupported native pilot schema version: {version}; expected {LEGACY_NATIVE_PILOT_SCHEMA_VERSION}, {IDENTITY_NATIVE_PILOT_SCHEMA_VERSION}, {ABSTENTION_NATIVE_PILOT_SCHEMA_VERSION}, {TRUSTED_SCOPED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION}, {SOURCE_BACKED_PREREQUISITE_NATIVE_PILOT_SCHEMA_VERSION}, {UNAVAILABLE_PREREQUISITE_NATIVE_PILOT_SCHEMA_VERSION}, {EXPIRED_PROCEDURE_NATIVE_PILOT_SCHEMA_VERSION}, {TARGETED_HOST_DIAGNOSTIC_NATIVE_PILOT_SCHEMA_VERSION}, {STRUCTURED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION}, {OPERATION_EVIDENCE_ROUTE_NATIVE_PILOT_SCHEMA_VERSION}, {SOURCE_BOUND_OPERATION_EVIDENCE_NATIVE_PILOT_SCHEMA_VERSION}, or {REQUIRED_HOST_ACTION_NATIVE_PILOT_SCHEMA_VERSION}"
    )))
}

const fn schema_requires_source_backed_prerequisite(version: u32) -> bool {
    matches!(
        version,
        SOURCE_BACKED_PREREQUISITE_NATIVE_PILOT_SCHEMA_VERSION
            | UNAVAILABLE_PREREQUISITE_NATIVE_PILOT_SCHEMA_VERSION
            | EXPIRED_PROCEDURE_NATIVE_PILOT_SCHEMA_VERSION
            | TARGETED_HOST_DIAGNOSTIC_NATIVE_PILOT_SCHEMA_VERSION
    )
}

fn validate_case(case: &NativePilotCase, protocol_schema_version: u32) -> EvalResult<()> {
    if case.id.trim().is_empty()
        || case.teaching_cwd.trim().is_empty()
        || case.evaluation_cwd.trim().is_empty()
        || case.evaluation_prompt.trim().is_empty()
        || case.expected_repository_remote.trim().is_empty()
        || case
            .expected_project
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        || case
            .expected_component
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        || case.expected_first_action.trim().is_empty()
        || case.procedure.context_key.trim().is_empty()
        || case.procedure.task.trim().is_empty()
        || case.procedure.command.trim().is_empty()
        || case.procedure.expected_output_contains.trim().is_empty()
    {
        return Err(EvalError::Invalid(format!(
            "native pilot case {} contains an empty required field",
            case.id
        )));
    }
    if protocol_schema_version < IDENTITY_NATIVE_PILOT_SCHEMA_VERSION
        && (case.expected_project.is_none() || case.expected_component.is_none())
    {
        return Err(EvalError::Invalid(format!(
            "legacy native pilot case {} requires concrete project and component identities",
            case.id
        )));
    }
    if protocol_schema_version < ABSTENTION_NATIVE_PILOT_SCHEMA_VERSION
        && case.expected_outcome == NativePilotExpectedOutcome::Abstain
    {
        return Err(EvalError::Invalid(format!(
            "native pilot case {} requires schema {ABSTENTION_NATIVE_PILOT_SCHEMA_VERSION} for expected abstention",
            case.id
        )));
    }
    if protocol_schema_version < STRUCTURED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION
        && (case.expected_project_status.is_some()
            || case.expected_project_confirmation_required.is_some())
    {
        return Err(EvalError::Invalid(format!(
            "native pilot case {} requires schema {STRUCTURED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION} for structured project authorization",
            case.id
        )));
    }
    if protocol_schema_version >= STRUCTURED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION {
        if case.expected_first_action != "resolve_checkout_identity" {
            return Err(EvalError::Invalid(format!(
                "native pilot schema {STRUCTURED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION} case {} must use resolve_checkout_identity as the first action",
                case.id
            )));
        }
        let status = case.expected_project_status.ok_or_else(|| {
            EvalError::Invalid(format!(
                "native pilot schema {STRUCTURED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION} case {} requires expected_project_status",
                case.id
            ))
        })?;
        let confirmation_required = case.expected_project_confirmation_required.ok_or_else(|| {
            EvalError::Invalid(format!(
                "native pilot schema {STRUCTURED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION} case {} requires expected_project_confirmation_required",
                case.id
            ))
        })?;
        match (case.expected_project.as_deref(), status, confirmation_required) {
            (Some(_), NativePilotExpectedProjectStatus::Authorized, false) => {}
            (None, NativePilotExpectedProjectStatus::RequiresConfirmation, true)
                if case.expected_outcome == NativePilotExpectedOutcome::Abstain => {}
            _ => {
                return Err(EvalError::Invalid(format!(
                    "native pilot schema {STRUCTURED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION} case {} has inconsistent project identity, authorization status, confirmation requirement, or outcome",
                    case.id
                )))
            }
        }
    }
    if protocol_schema_version >= OPERATION_EVIDENCE_ROUTE_NATIVE_PILOT_SCHEMA_VERSION
        && (case.expected_outcome != NativePilotExpectedOutcome::Abstain
            || !case
                .condition_evidence_target
                .as_deref()
                .is_some_and(|target| !target.trim().is_empty())
            || !case
                .condition_evidence_output_contains
                .as_deref()
                .is_some_and(|excerpt| !excerpt.trim().is_empty()))
    {
        return Err(EvalError::Invalid(format!(
            "native pilot schema {OPERATION_EVIDENCE_ROUTE_NATIVE_PILOT_SCHEMA_VERSION} case {} requires abstention plus one frozen operation-evidence target and excerpt",
            case.id
        )));
    }
    if protocol_schema_version >= REQUIRED_HOST_ACTION_NATIVE_PILOT_SCHEMA_VERSION {
        let required_terms = case
            .procedure_query_required_terms
            .iter()
            .map(|term| term.trim().to_ascii_lowercase())
            .collect::<BTreeSet<_>>();
        let prompt = case.evaluation_prompt.to_ascii_lowercase();
        if case.procedure_query_max_chars != Some(512)
            || required_terms.is_empty()
            || required_terms.iter().any(|term| {
                term.is_empty() || term.chars().count() > 64 || !prompt.contains(term.as_str())
            })
            || required_terms.len() != case.procedure_query_required_terms.len()
        {
            return Err(EvalError::Invalid(format!(
                "native pilot schema {REQUIRED_HOST_ACTION_NATIVE_PILOT_SCHEMA_VERSION} case {} requires a 512-character procedure-query bound and unique non-empty task terms present in the evaluation prompt",
                case.id
            )));
        }
    } else if case.procedure_query_max_chars.is_some()
        || !case.procedure_query_required_terms.is_empty()
    {
        return Err(EvalError::Invalid(format!(
            "native pilot case {} requires schema {REQUIRED_HOST_ACTION_NATIVE_PILOT_SCHEMA_VERSION} for a bounded procedure-query contract",
            case.id
        )));
    }
    if protocol_schema_version < TRUSTED_SCOPED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION
        && (case.project_evidence_target.is_some()
            || case.project_evidence_output_contains.is_some()
            || case.component_evidence_target.is_some()
            || case.component_evidence_output_contains.is_some())
    {
        return Err(EvalError::Invalid(format!(
            "native pilot case {} requires schema {TRUSTED_SCOPED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION} for trusted scoped-identity evidence",
            case.id
        )));
    }
    if protocol_schema_version >= TRUSTED_SCOPED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION {
        validate_identity_evidence(
            case,
            "project",
            case.expected_project.as_deref(),
            case.project_evidence_target.as_deref(),
            case.project_evidence_output_contains.as_deref(),
        )?;
        validate_identity_evidence(
            case,
            "component",
            case.expected_component.as_deref(),
            case.component_evidence_target.as_deref(),
            case.component_evidence_output_contains.as_deref(),
        )?;
    }
    if protocol_schema_version < SOURCE_BACKED_PREREQUISITE_NATIVE_PILOT_SCHEMA_VERSION
        && !case.procedure.prerequisite_sources.is_empty()
    {
        return Err(EvalError::Invalid(format!(
            "native pilot case {} requires schema {SOURCE_BACKED_PREREQUISITE_NATIVE_PILOT_SCHEMA_VERSION} for prerequisite sources",
            case.id
        )));
    }
    if protocol_schema_version >= STRUCTURED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION
        && (!case.procedure.prerequisite_sources.is_empty()
            || !case.procedure.conditions.is_empty())
    {
        return Err(EvalError::Invalid(format!(
            "native pilot schema {STRUCTURED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION} case {} must keep the focused wrong-scope identity procedure free of applicability prerequisites",
            case.id
        )));
    }
    if schema_requires_source_backed_prerequisite(protocol_schema_version) {
        if case.procedure.prerequisite_sources.len() != 1 {
            return Err(EvalError::Invalid(format!(
                "native pilot schema {SOURCE_BACKED_PREREQUISITE_NATIVE_PILOT_SCHEMA_VERSION} case {} requires exactly one prerequisite source",
                case.id
            )));
        }
        for (condition_key, source) in &case.procedure.prerequisite_sources {
            if !case.procedure.conditions.contains_key(condition_key) {
                return Err(EvalError::Invalid(format!(
                    "native pilot case {} prerequisite source key {condition_key} has no matching condition",
                    case.id
                )));
            }
            if !source.format.trim().eq_ignore_ascii_case("toml")
                || !is_safe_relative_path(Path::new(&source.relative_path))
                || source.key_path.is_empty()
                || source.key_path.iter().any(|key| key.trim().is_empty())
            {
                return Err(EvalError::Invalid(format!(
                    "native pilot case {} contains an invalid TOML prerequisite source",
                    case.id
                )));
            }
        }
    }
    if protocol_schema_version < UNAVAILABLE_PREREQUISITE_NATIVE_PILOT_SCHEMA_VERSION
        && case.evaluation_prerequisite_state != NativePilotEvaluationPrerequisiteState::Unchanged
    {
        return Err(EvalError::Invalid(format!(
            "native pilot case {} requires schema {UNAVAILABLE_PREREQUISITE_NATIVE_PILOT_SCHEMA_VERSION} for an unavailable prerequisite source",
            case.id
        )));
    }
    if protocol_schema_version < EXPIRED_PROCEDURE_NATIVE_PILOT_SCHEMA_VERSION
        && case.procedure_expiry_seconds.is_some()
    {
        return Err(EvalError::Invalid(format!(
            "native pilot case {} requires schema {EXPIRED_PROCEDURE_NATIVE_PILOT_SCHEMA_VERSION} for procedure expiry",
            case.id
        )));
    }
    if case.procedure_expiry_seconds == Some(0)
        || case.procedure_expiry_seconds.is_some()
            && (case.expected_outcome != NativePilotExpectedOutcome::Abstain
                || case.evaluation_prerequisite_state
                    != NativePilotEvaluationPrerequisiteState::Unchanged)
    {
        return Err(EvalError::Invalid(format!(
            "native pilot case {} procedure expiry requires positive seconds, expected abstention, and an unchanged prerequisite source",
            case.id
        )));
    }
    if case.evaluation_prerequisite_state == NativePilotEvaluationPrerequisiteState::Missing
        && case.expected_outcome != NativePilotExpectedOutcome::Abstain
    {
        return Err(EvalError::Invalid(format!(
            "native pilot case {} can remove its evaluation prerequisite source only when abstention is expected",
            case.id
        )));
    }
    if case.expected_outcome == NativePilotExpectedOutcome::Abstain
        && !case
            .condition_evidence_target
            .as_deref()
            .is_some_and(|target| !target.trim().is_empty())
    {
        return Err(EvalError::Invalid(format!(
            "native pilot abstention case {} requires a non-empty condition_evidence_target",
            case.id
        )));
    }
    if case.expected_outcome == NativePilotExpectedOutcome::Abstain
        && case.evaluation_prerequisite_state == NativePilotEvaluationPrerequisiteState::Unchanged
        && !case
            .condition_evidence_output_contains
            .as_deref()
            .is_some_and(|excerpt| !excerpt.trim().is_empty())
    {
        return Err(EvalError::Invalid(format!(
            "native pilot abstention case {} requires a non-empty condition_evidence_output_contains",
            case.id
        )));
    }
    if case.expected_outcome == NativePilotExpectedOutcome::ExecuteProcedure
        && (case.evaluation_prerequisite_state != NativePilotEvaluationPrerequisiteState::Unchanged
            || case.condition_evidence_target.is_some()
            || case.condition_evidence_output_contains.is_some())
    {
        return Err(EvalError::Invalid(format!(
            "native pilot execution case {} cannot declare abstention condition evidence",
            case.id
        )));
    }
    if case.teaching_cwd == case.evaluation_cwd {
        return Err(EvalError::Invalid(format!(
            "native pilot case {} must use distinct teaching and evaluation checkouts",
            case.id
        )));
    }
    if case.procedure.failed_commands.is_empty()
        || case
            .procedure
            .failed_commands
            .iter()
            .any(|command| command.trim().is_empty() || *command == case.procedure.command)
    {
        return Err(EvalError::Invalid(format!(
            "native pilot case {} needs a distinct non-empty failed command",
            case.id
        )));
    }
    let leaked_values = [
        case.procedure.context_key.as_str(),
        case.procedure.command.as_str(),
        case.procedure.expected_output_contains.as_str(),
    ];
    if leaked_values
        .iter()
        .any(|value| case.evaluation_prompt.contains(value))
    {
        return Err(EvalError::Invalid(format!(
            "native pilot case {} leaks learned ground truth in its evaluation prompt",
            case.id
        )));
    }
    Ok(())
}

fn is_safe_relative_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, std::path::Component::Normal(_)))
}

fn validate_identity_evidence(
    case: &NativePilotCase,
    field: &str,
    expected: Option<&str>,
    target: Option<&str>,
    excerpt: Option<&str>,
) -> EvalResult<()> {
    match (expected, target, excerpt) {
        (Some(expected), Some(target), Some(excerpt))
            if !target.trim().is_empty() && !excerpt.trim().is_empty() =>
        {
            if excerpt
                .to_ascii_lowercase()
                .contains(&expected.to_ascii_lowercase())
            {
                Ok(())
            } else {
                Err(EvalError::Invalid(format!(
                    "native pilot schema {TRUSTED_SCOPED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION} case {} {field} evidence excerpt must contain expected {field} identity",
                    case.id
                )))
            }
        }
        (Some(_), _, _) => Err(EvalError::Invalid(format!(
            "native pilot schema {TRUSTED_SCOPED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION} case {} requires non-empty {field}_evidence_target and {field}_evidence_output_contains for positive {field} identity",
            case.id
        ))),
        (None, None, None) => Ok(()),
        (None, _, _) => Err(EvalError::Invalid(format!(
            "native pilot schema {TRUSTED_SCOPED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION} case {} cannot declare {field} identity evidence when expected_{field} is null",
            case.id
        ))),
    }
}

#[allow(clippy::too_many_arguments)]
fn prepare_lane(
    output: &Path,
    order: u32,
    run_ref: &NativePilotRunRef,
    arm: &NativePilotArm,
    case: &NativePilotCase,
    engram: &PilotBinaryAttestation,
    codex: &PilotBinaryAttestation,
    claude: &PilotBinaryAttestation,
    schema_path: &Path,
    protocol_schema_version: u32,
    codex_min_idle_hours: u32,
    codex_authentication_mode: CodexAuthenticationMode,
    claude_model: &str,
    claude_max_turns: u32,
    claude_budget_millis: u64,
    preparation_override: Option<&NativePilotLanePreparationOverride>,
) -> EvalResult<PreparedNativeLane> {
    let default_directory_name =
        format!("{order:02}-{}-{}-r{}", case.id, arm.id, run_ref.repetition);
    let directory_name = preparation_override
        .and_then(|value| value.directory_name.as_deref())
        .unwrap_or(&default_directory_name);
    let lane_dir = output.join("lanes").join(directory_name);
    if preparation_override.is_some_and(|value| value.closed_world_provider_environment) {
        create_private_dir_all(&lane_dir)?;
    }
    let fixture_root = lane_dir.join("fixture");
    let layout = materialize_engineering_context_fixture(&fixture_root)?;
    let teaching_cwd = resolve_fixture_uri(&layout, &case.teaching_cwd)?;
    let evaluation_cwd = resolve_fixture_uri(&layout, &case.evaluation_cwd)?;
    let teaching_checkout = resolve_checkout_root(&layout, &case.teaching_cwd)?;
    let evaluation_checkout = resolve_checkout_root(&layout, &case.evaluation_cwd)?;
    let teaching_repository_remote = repository_remote(&teaching_checkout)?;
    apply_evaluation_prerequisite_state(case, protocol_schema_version, &evaluation_checkout)?;
    let condition_evidence_sha256 = prepare_condition_evidence(case, &evaluation_checkout)?;
    let prerequisite_source_observation = prepare_prerequisite_source_observation(
        case,
        protocol_schema_version,
        &teaching_checkout,
        &evaluation_checkout,
    )?;
    let project_evidence_sha256 = prepare_identity_evidence(
        case,
        protocol_schema_version,
        "project",
        case.project_evidence_target.as_deref(),
        case.project_evidence_output_contains.as_deref(),
        &evaluation_checkout,
    )?;
    let component_evidence_sha256 = prepare_identity_evidence(
        case,
        protocol_schema_version,
        "component",
        case.component_evidence_target.as_deref(),
        case.component_evidence_output_contains.as_deref(),
        &evaluation_checkout,
    )?;

    let evidence_dir = lane_dir.join("attested-evidence");
    create_private_dir_all(&evidence_dir)?;
    let receipt_path = evidence_dir.join("procedure-success.json");
    let receipt = serde_json::json!({
        "command": case.procedure.command,
        "exit_code": case.procedure.expected_exit_code,
        "output": case.procedure.expected_output_contains,
        "conditions": case.procedure.conditions,
    });
    fs::write(
        &receipt_path,
        serde_json::to_string_pretty(&receipt)? + "\n",
    )?;
    let receipt_sha256 = sha256_file(&receipt_path)?;
    let contract_path = lane_dir.join("acceptance-contract.json");
    let mut contract = serde_json::json!({
        "case_id": case.id,
        "expected_repository_remote": case.expected_repository_remote,
        "expected_checkout_root": evaluation_checkout.canonicalize()?.display().to_string(),
        "expected_project": case.expected_project,
        "expected_project_status": case.expected_project_status,
        "expected_project_confirmation_required": case.expected_project_confirmation_required,
        "expected_component": case.expected_component,
        "expected_first_action": case.expected_first_action,
        "expected_outcome": case.expected_outcome,
        "procedure_expiry_seconds": case.procedure_expiry_seconds,
        "condition_evidence_target": case.condition_evidence_target,
        "condition_evidence_output_contains": case.condition_evidence_output_contains,
        "condition_evidence_sha256": condition_evidence_sha256,
        "procedure_query_max_chars": case.procedure_query_max_chars,
        "procedure_query_required_terms": case.procedure_query_required_terms,
        "prerequisite_source_observation": prerequisite_source_observation,
        "project_evidence_target": case.project_evidence_target,
        "project_evidence_output_contains": case.project_evidence_output_contains,
        "project_evidence_sha256": project_evidence_sha256,
        "component_evidence_target": case.component_evidence_target,
        "component_evidence_output_contains": case.component_evidence_output_contains,
        "component_evidence_sha256": component_evidence_sha256,
        "required_context_keys": if case.expected_outcome == NativePilotExpectedOutcome::ExecuteProcedure {
            vec![case.procedure.context_key.clone()]
        } else {
            Vec::new()
        },
        "forbidden_context_keys": if case.expected_outcome == NativePilotExpectedOutcome::Abstain {
            vec![case.procedure.context_key.clone()]
        } else {
            Vec::new()
        },
        "required_task": case.procedure.task,
        "required_conditions": case.procedure.conditions,
        "required_prerequisite_sources": case.procedure.prerequisite_sources,
        "forbidden_commands": case.procedure.failed_commands,
        "required_command": case.procedure.command,
        "expected_exit_code": case.procedure.expected_exit_code,
        "expected_output_contains": case.procedure.expected_output_contains,
        "evidence_path": receipt_path.canonicalize()?.display().to_string(),
        "evidence_sha256": receipt_sha256,
    });
    if let Some(preparation_override) = preparation_override {
        let object = contract
            .as_object_mut()
            .expect("acceptance contract object");
        for (key, value) in &preparation_override.acceptance_contract_fields {
            if object.insert(key.clone(), value.clone()).is_some() {
                return Err(EvalError::Invalid(format!(
                    "specialized native-pilot acceptance field {key} collides with a base field"
                )));
            }
        }
    }
    fs::write(
        &contract_path,
        serde_json::to_string_pretty(&contract)? + "\n",
    )?;

    let closed_world_provider_environment =
        preparation_override.is_some_and(|value| value.closed_world_provider_environment);
    let mut environment = BTreeMap::new();
    if closed_world_provider_environment {
        let process_home = lane_dir.join("provider-home");
        let process_tmp = lane_dir.join("provider-tmp");
        create_private_dir_all(&process_home)?;
        create_private_dir_all(&process_tmp)?;
        environment.extend([
            (
                "HOME".to_string(),
                process_home.canonicalize()?.display().to_string(),
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
                process_tmp.canonicalize()?.display().to_string(),
            ),
        ]);
    }
    let required_secret_environment = Vec::new();
    let mut codex_authentication = None;
    let mut adapter_sha256 = None;
    let mut adapter_root = None;
    let mut engram_home = None;
    let mut engram_project = None;
    let mut cleanup_argv = None;
    let mut post_teaching_verification_argv = None;
    let mut claude_mcp_config = None;
    let mut claude_mcp_inline_json = None;
    if arm.memory_layer.uses_engram() {
        let generated_adapter_root = lane_dir.join("adapter");
        adapter_sha256 = Some(install_eval_adapter(
            &generated_adapter_root,
            &teaching_checkout,
            &arm.host,
        )?);
        adapter_root = Some(generated_adapter_root.canonicalize()?);
        let state_root = lane_dir.join("engram-home");
        create_private_dir_all(&state_root)?;
        let project = format!("native-pilot-{order:02}");
        environment.insert(
            ENGRAM_HOME_ENV.to_string(),
            state_root.canonicalize()?.display().to_string(),
        );
        engram_home = Some(state_root.canonicalize()?);
        engram_project = Some(project.clone());
        cleanup_argv = Some(vec![
            engram.path.clone(),
            "daemon".to_string(),
            "stop".to_string(),
            "--project".to_string(),
            project.clone(),
        ]);
        let verification_expiry_seconds = preparation_override
            .and_then(|value| value.verification_expiry_seconds)
            .or(case.procedure_expiry_seconds);
        let (expiry_flag, expiry_value) = verification_expiry_seconds.map_or_else(
            || ("--expires-in-days".to_string(), "30".to_string()),
            |seconds| ("--expires-in-seconds".to_string(), seconds.to_string()),
        );
        post_teaching_verification_argv = Some(vec![
            engram.path.clone(),
            "memory".to_string(),
            "--project".to_string(),
            project.clone(),
            "verify-procedure".to_string(),
            "${PROCEDURE_MEMORY_ID_FROM_TEACHING_TRACE}".to_string(),
            "--receipt".to_string(),
            receipt_path.canonicalize()?.display().to_string(),
            expiry_flag,
            expiry_value,
            "--confirm".to_string(),
            "--json".to_string(),
        ]);
        if arm.host == "claude_code" {
            let config_path = lane_dir.join("claude-mcp.json");
            let mcp_environment = BTreeMap::from([(
                ENGRAM_HOME_ENV.to_string(),
                state_root.canonicalize()?.display().to_string(),
            )]);
            if closed_world_provider_environment {
                let config = serde_json::json!({
                    "mcpServers": {
                        "engram": {
                            "type": "stdio",
                            "command": engram.path,
                            "args": ["serve", "--project", project, "--profile", "agent"],
                            "env": mcp_environment,
                        }
                    }
                });
                let compact = serde_json::to_string(&config)?;
                write_family_v3_private_file_exclusive(&config_path, compact.as_bytes())?;
                claude_mcp_inline_json = Some(compact);
            } else {
                write_claude_mcp_config(&config_path, engram, &project, &mcp_environment)?;
            }
            claude_mcp_config = Some(config_path.canonicalize()?);
        }
    }

    let claude_teaching_bash_commands = if arm.host == "claude_code" {
        let teaching_checkout = teaching_checkout.canonicalize()?;
        case.procedure
            .failed_commands
            .iter()
            .chain(std::iter::once(&case.procedure.command))
            .map(|command| format!("cd {} && {command}", teaching_checkout.display()))
            .collect()
    } else {
        Vec::new()
    };
    let writer_model = if arm.host == "claude_code" {
        claude_model
    } else {
        "codex-cli"
    };
    let mut teaching_prompt = teaching_prompt(
        case,
        arm.memory_layer,
        &receipt_path,
        &teaching_checkout,
        &teaching_repository_remote,
        &arm.host,
        writer_model,
        &claude_teaching_bash_commands,
    )?;
    if let Some(suffix) =
        preparation_override.and_then(|value| value.teaching_prompt_suffix.as_deref())
    {
        teaching_prompt.push_str("\n\n");
        teaching_prompt.push_str(suffix);
    }
    let mut evaluation_prompt = evaluation_prompt(case);
    if let Some(suffix) =
        preparation_override.and_then(|value| value.evaluation_prompt_suffix.as_deref())
    {
        evaluation_prompt.push_str("\n\n");
        evaluation_prompt.push_str(suffix);
    }
    let teaching_trace_path = lane_dir.join("teaching-trace.jsonl");
    let evaluation_trace_path = lane_dir.join("evaluation-trace.jsonl");
    let post_teaching_verification_output_path = lane_dir.join("procedure-verification.json");
    let agent_output_path = lane_dir.join("agent-output.json");
    let mut artifact_gates = Vec::new();

    let (teaching_argv, activation_argv, evaluation_argv, activation_wait_hours) = match arm
        .host
        .as_str()
    {
        "codex" => {
            let codex_home = lane_dir.join("codex-home");
            create_private_dir_all(&codex_home)?;
            environment.insert(
                "CODEX_HOME".to_string(),
                codex_home.canonicalize()?.display().to_string(),
            );
            codex_authentication = Some(prepared_codex_authentication(
                codex,
                codex_authentication_mode,
            ));
            if arm.memory_layer.uses_native() {
                artifact_gates.push(NativeArtifactGate {
                        layer: "codex_native_memory".to_string(),
                        path: codex_home.join("memories").display().to_string(),
                        required: false,
                        requirement: "after the idle interval and activation, record whether the provider generated non-empty memory artifacts; absence is a valid native-memory outcome, and shell commands must never create or edit them"
                            .to_string(),
                    });
            }
            (
                codex_argv(
                    codex,
                    engram,
                    codex_authentication_mode,
                    arm.memory_layer,
                    &teaching_cwd,
                    &teaching_prompt,
                    None,
                    engram_project.as_deref(),
                    engram_home.as_deref(),
                    adapter_root.as_deref(),
                    codex_min_idle_hours,
                    CodexPhase::Teaching,
                )?,
                Some(codex_argv(
                    codex,
                    engram,
                    codex_authentication_mode,
                    arm.memory_layer,
                    &evaluation_cwd,
                    "Reply exactly NATIVE_MEMORY_ACTIVATION_COMPLETE.",
                    None,
                    engram_project.as_deref(),
                    engram_home.as_deref(),
                    adapter_root.as_deref(),
                    codex_min_idle_hours,
                    CodexPhase::Activation,
                )?),
                codex_argv(
                    codex,
                    engram,
                    codex_authentication_mode,
                    arm.memory_layer,
                    &evaluation_cwd,
                    &evaluation_prompt,
                    Some(schema_path),
                    engram_project.as_deref(),
                    engram_home.as_deref(),
                    adapter_root.as_deref(),
                    codex_min_idle_hours,
                    if preparation_override
                        .is_some_and(|value| value.persist_codex_evaluation_session)
                    {
                        CodexPhase::EvaluationPersisted
                    } else {
                        CodexPhase::Evaluation
                    },
                )?,
                codex_min_idle_hours,
            )
        }
        "claude_code" => {
            let config_root = lane_dir.join("claude-config");
            let memory_root = lane_dir.join("claude-memory");
            create_private_dir_all(&config_root)?;
            create_private_dir_all(&memory_root)?;
            environment.insert(
                "CLAUDE_CONFIG_DIR".to_string(),
                config_root.canonicalize()?.display().to_string(),
            );
            environment.insert(
                "CLAUDE_CODE_DISABLE_AUTO_MEMORY".to_string(),
                if arm.memory_layer.uses_native() {
                    "0"
                } else {
                    "1"
                }
                .to_string(),
            );
            environment.insert("DISABLE_TELEMETRY".to_string(), "1".to_string());
            if arm.memory_layer.uses_native() {
                artifact_gates.push(NativeArtifactGate {
                        layer: "claude_code_auto_memory".to_string(),
                        path: memory_root.join("MEMORY.md").display().to_string(),
                        required: false,
                        requirement: "record whether Claude Code generated a non-empty MEMORY.md through its normal auto-memory behavior; absence is a valid native-memory outcome, and shell commands must never create or edit it"
                            .to_string(),
                    });
            }
            let settings_path = lane_dir.join("claude-settings.json");
            let settings = serde_json::json!({
                "autoMemoryEnabled": arm.memory_layer.uses_native(),
                "autoMemoryDirectory": memory_root.canonicalize()?.display().to_string(),
            });
            let settings_inline_json = closed_world_provider_environment
                .then(|| serde_json::to_string(&settings))
                .transpose()?;
            if let Some(compact) = settings_inline_json.as_deref() {
                write_family_v3_private_file_exclusive(&settings_path, compact.as_bytes())?;
            } else {
                fs::write(
                    &settings_path,
                    serde_json::to_string_pretty(&settings)? + "\n",
                )?;
            }
            let empty_mcp_path = lane_dir.join("claude-mcp-empty.json");
            let empty_mcp = serde_json::json!({"mcpServers": {}});
            let empty_mcp_inline_json = closed_world_provider_environment
                .then(|| serde_json::to_string(&empty_mcp))
                .transpose()?;
            if !arm.memory_layer.uses_engram() {
                if let Some(compact) = empty_mcp_inline_json.as_deref() {
                    write_family_v3_private_file_exclusive(&empty_mcp_path, compact.as_bytes())?;
                } else {
                    fs::write(&empty_mcp_path, "{\"mcpServers\":{}}\n")?;
                }
            }
            let mcp_path = claude_mcp_config
                .as_deref()
                .unwrap_or(empty_mcp_path.as_path());
            let inline_config_json = if closed_world_provider_environment {
                Some((
                    settings_inline_json.as_deref().ok_or_else(|| {
                        EvalError::Invalid(
                            "family-v3 Claude settings JSON was not materialized".to_string(),
                        )
                    })?,
                    if arm.memory_layer.uses_engram() {
                        claude_mcp_inline_json.as_deref().ok_or_else(|| {
                            EvalError::Invalid(
                                "family-v3 Claude Engram MCP JSON was not materialized".to_string(),
                            )
                        })?
                    } else {
                        empty_mcp_inline_json.as_deref().ok_or_else(|| {
                            EvalError::Invalid(
                                "family-v3 Claude empty MCP JSON was not materialized".to_string(),
                            )
                        })?
                    },
                ))
            } else {
                None
            };
            let instruction_path = if arm.memory_layer.uses_engram() {
                adapter_root
                    .as_deref()
                    .ok_or_else(|| EvalError::Invalid("missing Claude Engram adapter".to_string()))?
                    .join("claude-eval-instructions.md")
            } else {
                teaching_checkout.join("CLAUDE.md")
            };
            (
                claude_argv(
                    claude,
                    &teaching_cwd,
                    &teaching_prompt,
                    None,
                    &settings_path,
                    mcp_path,
                    inline_config_json,
                    &instruction_path,
                    claude_model,
                    claude_max_turns,
                    claude_budget_millis,
                    false,
                    arm.memory_layer
                        .uses_native()
                        .then_some(memory_root.as_path()),
                    &claude_teaching_bash_commands,
                )?,
                None,
                claude_argv(
                    claude,
                    &evaluation_cwd,
                    &evaluation_prompt,
                    Some(schema_path),
                    &settings_path,
                    mcp_path,
                    inline_config_json,
                    &instruction_path,
                    claude_model,
                    claude_max_turns,
                    claude_budget_millis,
                    true,
                    arm.memory_layer
                        .uses_native()
                        .then_some(memory_root.as_path()),
                    &[],
                )?,
                0,
            )
        }
        other => {
            return Err(EvalError::Invalid(format!(
                "unsupported native pilot host: {other}"
            )))
        }
    };

    if let (Some(home), Some(project)) = (&engram_home, &engram_project) {
        artifact_gates.push(NativeArtifactGate {
            layer: "engram".to_string(),
            path: home.join("projects").join(project).join("data").display().to_string(),
            required: true,
            requirement: "the teaching trace must contain one structured procedure candidate add; the trusted evaluator must substitute that exact returned ID into post_teaching_verification_argv, verify the pre-attested receipt, and require a fresh procedure_match to revalidate it before evaluation"
                .to_string(),
        });
    }

    Ok(PreparedNativeLane {
        order,
        case_id: case.id.clone(),
        arm: arm.id.clone(),
        host: arm.host.clone(),
        memory_layer: arm.memory_layer,
        repetition: run_ref.repetition,
        fixture_revision: layout.fixture_revision,
        teaching_cwd: teaching_cwd.display().to_string(),
        evaluation_cwd: evaluation_cwd.display().to_string(),
        environment,
        required_secret_environment,
        codex_authentication,
        claude_teaching_bash_commands,
        teaching_argv,
        activation_argv,
        post_teaching_verification_argv,
        activation_wait_hours,
        evaluation_argv,
        artifact_gates,
        acceptance_contract: contract_path.canonicalize()?.display().to_string(),
        adapter_sha256,
        engram_home: engram_home.map(|path| path.display().to_string()),
        engram_project,
        cleanup_argv,
        teaching_trace_path: teaching_trace_path.display().to_string(),
        activation_trace_path: (arm.host == "codex").then(|| {
            lane_dir
                .join("activation-trace.jsonl")
                .display()
                .to_string()
        }),
        evaluation_trace_path: evaluation_trace_path.display().to_string(),
        post_teaching_verification_output_path: arm
            .memory_layer
            .uses_engram()
            .then(|| post_teaching_verification_output_path.display().to_string()),
        agent_output_path: agent_output_path.display().to_string(),
    })
}

fn teaching_prompt(
    case: &NativePilotCase,
    layer: MemoryLayer,
    receipt_path: &Path,
    teaching_checkout: &Path,
    teaching_repository_remote: &str,
    host: &str,
    writer_model: &str,
    claude_teaching_bash_commands: &[String],
) -> EvalResult<String> {
    let failures = case.procedure.failed_commands.join("; ");
    let conditions = serde_json::to_string(&case.procedure.conditions)?;
    let expiry_instruction = case.procedure_expiry_seconds.map_or_else(String::new, |seconds| {
        format!(
            " The verified procedure expires {seconds} seconds after trusted verification; preserve that freshness limit."
        )
    });
    let memory_instruction = match layer {
        MemoryLayer::Native => "Rely only on the host's normal automatic native-memory behavior. Let the host's own memory instructions decide whether and how to persist the lesson; do not use shell commands or repository files to emulate native memory."
            .to_string(),
        MemoryLayer::Engram | MemoryLayer::Both => {
            let (writer_harness, model_provider, surface) = match host {
                "claude_code" => ("claude_code", "anthropic", "claude-code"),
                "codex" => ("codex", "openai", "codex-cli"),
                other => {
                    return Err(EvalError::Invalid(format!(
                        "unsupported native pilot host in teaching prompt: {other}"
                    )))
                }
            };
            let request = serde_json::json!({
                "action": "add",
                "kind": "procedure",
                "title": case.procedure.context_key,
                "content": format!(
                    "Verified candidate for durable context key `{}`: {}",
                    case.procedure.context_key, case.procedure.task
                ),
                "origin": "agent_observed",
                "scope_type": "repository",
                "remote_url": teaching_repository_remote,
                "local_path": teaching_checkout.canonicalize()?.display().to_string(),
                "writer_harness": writer_harness,
                "model_provider": model_provider,
                "model": writer_model,
                "surface": surface,
                "actor": "agent",
                "tags": [case.procedure.context_key.clone()],
                "procedure": {
                    "task": case.procedure.task,
                    "commands": [case.procedure.command.clone()],
                    "prerequisites": case.procedure.conditions.clone(),
                    "prerequisite_sources": case.procedure.prerequisite_sources.clone(),
                    "failure_signatures": case.procedure.failed_commands.clone(),
                    "verification_command": case.procedure.command,
                    "verification_exit_code": case.procedure.expected_exit_code,
                    "verification_output_contains": case.procedure.expected_output_contains,
                },
                "evidence": [
                    {
                        "kind": "tool_call",
                        "target": case.procedure.command,
                        "summary": "Successful command observed during the controlled teaching session"
                    },
                    {
                        "kind": "document",
                        "target": receipt_path.canonicalize()?.display().to_string(),
                        "summary": "Machine-generated verification receipt supplied by the trusted evaluator"
                    }
                ]
            });
            let native_instruction = if layer == MemoryLayer::Both {
                " Also allow the host's normal automatic native-memory behavior. Let the host's own memory instructions decide whether and how to persist the lesson; do not use shell commands or repository files to emulate native memory."
            } else {
                " Native memory is disabled."
            };
            format!(
                "{native_instruction} After observing the successful command, submit exactly one call to the Engram memory tool using this exact JSON object as its arguments; do not add, omit, rename, infer, or rewrite any field: {} Do not promote or verify the returned candidate yourself; the trusted evaluator will verify that exact returned ID against the supplied receipt.",
                serde_json::to_string(&request)?
            )
        }
    };
    let claude_bash_instruction =
        if host == "claude_code" && !claude_teaching_bash_commands.is_empty() {
            format!(
                " Use exactly these pre-approved Bash invocations and no other Bash command: {}.",
                claude_teaching_bash_commands
                    .iter()
                    .map(|command| format!("`{command}`"))
                    .collect::<Vec<_>>()
                    .join("; ")
            )
        } else {
            String::new()
        };
    Ok(format!(
        "Controlled teaching session for a later fresh-session evaluation. {memory_instruction}{claude_bash_instruction}\n\nLearn durable context key `{}` for task `{}`. First run the known failed command `{}` and observe its failure. Then run the correct command `{}` from the repository root and confirm output containing `{}` with exit code {}. Applicability conditions are {}.{expiry_instruction} The trusted evaluator's machine-generated verification receipt is `{}`. Preserve the failed invocation as a failure signature, but never as the recommended command. Do not add this information to repository files. Finish only after the successful command and the selected memory system's normal capture step are complete.",
        case.procedure.context_key,
        case.procedure.task,
        failures,
        case.procedure.command,
        case.procedure.expected_output_contains,
        case.procedure.expected_exit_code,
        conditions,
        receipt_path.canonicalize()?.display()
    ))
}

fn evaluation_prompt(case: &NativePilotCase) -> String {
    format!(
        "{} This is a fresh-session memory evaluation. Do not read outside the checkout, inspect benchmark plans or acceptance contracts, use the network, or discover the answer by trial and error. Inspect repository identity and local applicability conditions, then use only durable context available through the configured memory layer. If no applicable durable procedure is available, abstain instead of guessing. Return only the required structured result.",
        case.evaluation_prompt.trim()
    )
}

fn repository_remote(checkout: &Path) -> EvalResult<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(checkout)
        .args(["config", "--get", "remote.origin.url"])
        .output()?;
    if !output.status.success() {
        return Err(EvalError::Invalid(format!(
            "failed to resolve teaching repository remote at {}: {}",
            checkout.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    let remote = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if remote.is_empty() {
        return Err(EvalError::Invalid(format!(
            "teaching repository remote is empty at {}",
            checkout.display()
        )));
    }
    Ok(remote)
}

pub(crate) fn native_agent_output_schema(protocol: &NativePilotProtocol) -> EvalResult<String> {
    let mut schema: serde_json::Value = serde_json::from_str(AGENT_OUTPUT_SCHEMA)?;
    let actions = protocol
        .cases
        .iter()
        .map(|case| case.expected_first_action.as_str())
        .collect::<BTreeSet<_>>();
    let allowed = actions
        .into_iter()
        .map(|action| serde_json::Value::String(action.to_string()))
        .chain(std::iter::once(serde_json::Value::Null))
        .collect::<Vec<_>>();
    let first_action = schema
        .pointer_mut("/properties/first_action")
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| {
            EvalError::Invalid("agent output schema omitted first_action property".to_string())
        })?;
    first_action.insert("enum".to_string(), serde_json::Value::Array(allowed));
    first_action.insert(
        "description".to_string(),
        serde_json::Value::String(
            "Preregistered semantic label for the first externally visible task action."
                .to_string(),
        ),
    );
    schema["properties"]["repository_remote"]["description"] = serde_json::Value::String(
        "Canonical origin remote observed through a trusted host or Engram tool result; null only when unresolved."
            .to_string(),
    );
    schema["properties"]["project"]["description"] = serde_json::Value::String(
        "Canonical project selected without material ambiguity; null when unresolved or confirmation is required."
            .to_string(),
    );
    schema["properties"]["component"]["description"] = serde_json::Value::String(
        "Current component resolved from authoritative checkout evidence; null when none is resolved."
            .to_string(),
    );
    if protocol.schema_version >= STRUCTURED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION {
        let properties = schema
            .get_mut("properties")
            .and_then(serde_json::Value::as_object_mut)
            .ok_or_else(|| {
                EvalError::Invalid("agent output schema omitted properties".to_string())
            })?;
        properties.insert(
            "checkout_root".to_string(),
            serde_json::json!({
                "type": ["string", "null"],
                "description": "Canonical checkout root observed through trusted identity evidence; null only when unresolved."
            }),
        );
        properties.insert(
            "project_status".to_string(),
            serde_json::json!({
                "type": "string",
                "enum": ["authorized", "requires_confirmation", "unavailable"],
                "description": "Structured project-authorization state; repository prose never authorizes a project."
            }),
        );
        properties.insert(
            "project_confirmation_required".to_string(),
            serde_json::json!({
                "type": "boolean",
                "description": "Whether selecting a project requires explicit user confirmation."
            }),
        );
        let required = schema
            .get_mut("required")
            .and_then(serde_json::Value::as_array_mut)
            .ok_or_else(|| {
                EvalError::Invalid("agent output schema omitted required keys".to_string())
            })?;
        required.extend(
            [
                "checkout_root",
                "project_status",
                "project_confirmation_required",
            ]
            .into_iter()
            .map(|key| serde_json::Value::String(key.to_string())),
        );
    }
    Ok(serde_json::to_string_pretty(&schema)? + "\n")
}

#[derive(Clone, Copy)]
enum CodexPhase {
    Teaching,
    Activation,
    Evaluation,
    EvaluationPersisted,
}

#[allow(clippy::too_many_arguments)]
fn codex_argv(
    codex: &PilotBinaryAttestation,
    engram: &PilotBinaryAttestation,
    authentication_mode: CodexAuthenticationMode,
    layer: MemoryLayer,
    cwd: &Path,
    prompt: &str,
    schema_path: Option<&Path>,
    engram_project: Option<&str>,
    engram_home: Option<&Path>,
    adapter_root: Option<&Path>,
    min_idle_hours: u32,
    phase: CodexPhase,
) -> EvalResult<Vec<String>> {
    let mut argv = vec![
        codex.path.clone(),
        "exec".to_string(),
        "--ignore-user-config".to_string(),
        "--ignore-rules".to_string(),
        "--strict-config".to_string(),
        "--sandbox".to_string(),
        "read-only".to_string(),
        "--cd".to_string(),
        cwd.canonicalize()?.display().to_string(),
        "--json".to_string(),
        "--config".to_string(),
        "feedback.enabled=false".to_string(),
        "--config".to_string(),
        authentication_mode.credential_store_config().to_string(),
    ];
    if matches!(phase, CodexPhase::Evaluation) {
        argv.push("--ephemeral".to_string());
    }
    if let Some(schema_path) = schema_path {
        argv.extend([
            "--output-schema".to_string(),
            schema_path.canonicalize()?.display().to_string(),
        ]);
    }

    let native = layer.uses_native();
    let (generate, use_memories) = match phase {
        CodexPhase::Teaching => (native, false),
        CodexPhase::Activation => (native, false),
        CodexPhase::Evaluation | CodexPhase::EvaluationPersisted => (false, native),
    };
    argv.extend([
        "--config".to_string(),
        format!("features.memories={native}"),
        "--config".to_string(),
        format!("memories.generate_memories={generate}"),
        "--config".to_string(),
        format!("memories.use_memories={use_memories}"),
        "--config".to_string(),
        "memories.disable_on_external_context=false".to_string(),
        "--config".to_string(),
        format!("memories.min_rollout_idle_hours={min_idle_hours}"),
    ]);

    if layer.uses_engram() {
        let project = engram_project.ok_or_else(|| {
            EvalError::Invalid("Codex Engram lane is missing daemon project".to_string())
        })?;
        let state_root = engram_home.ok_or_else(|| {
            EvalError::Invalid("Codex Engram lane is missing state root".to_string())
        })?;
        let skill_path = adapter_root
            .ok_or_else(|| EvalError::Invalid("Codex Engram lane is missing adapter".to_string()))?
            .join(".codex/skills/engram-memory-session/SKILL.md")
            .canonicalize()?;
        let developer_instructions = fs::read_to_string(&skill_path)?;
        let args = vec!["serve", "--project", project, "--profile", "agent"];
        argv.extend([
            "--config".to_string(),
            format!(
                "mcp_servers.engram.command={}",
                serde_json::to_string(&engram.path)?
            ),
            "--config".to_string(),
            format!("mcp_servers.engram.args={}", serde_json::to_string(&args)?),
            "--config".to_string(),
            format!(
                "mcp_servers.engram.env={{ENGRAM_HOME={}}}",
                serde_json::to_string(&state_root.display().to_string())?
            ),
            "--config".to_string(),
            "mcp_servers.engram.required=true".to_string(),
            "--config".to_string(),
            "mcp_servers.engram.startup_timeout_sec=60".to_string(),
            "--config".to_string(),
            format!(
                "skills.config=[{{path={},enabled=true}}]",
                serde_json::to_string(&skill_path.display().to_string())?
            ),
            "--config".to_string(),
            format!(
                "developer_instructions={}",
                serde_json::to_string(&developer_instructions)?
            ),
        ]);
    }
    argv.push(prompt.to_string());
    Ok(argv)
}

fn prepared_codex_authentication(
    codex: &PilotBinaryAttestation,
    mode: CodexAuthenticationMode,
) -> PreparedCodexAuthentication {
    let base_argv = vec![
        codex.path.clone(),
        "login".to_string(),
        "--config".to_string(),
        mode.credential_store_config().to_string(),
    ];
    let mut login_argv = base_argv.clone();
    if mode == CodexAuthenticationMode::ChatgptDeviceKeyring {
        login_argv.push("--device-auth".to_string());
    }
    let mut status_argv = base_argv;
    status_argv.push("status".to_string());
    PreparedCodexAuthentication {
        mode,
        credential_store: mode.credential_store().to_string(),
        login_argv,
        status_argv,
        expected_status: CODEX_CHATGPT_LOGIN_STATUS.to_string(),
    }
}

#[allow(clippy::too_many_arguments)]
fn claude_argv(
    claude: &PilotBinaryAttestation,
    cwd: &Path,
    prompt: &str,
    schema_path: Option<&Path>,
    settings_path: &Path,
    mcp_path: &Path,
    inline_config_json: Option<(&str, &str)>,
    instruction_path: &Path,
    model: &str,
    max_turns: u32,
    budget_millis: u64,
    no_session_persistence: bool,
    native_memory_root: Option<&Path>,
    teaching_bash_commands: &[String],
) -> EvalResult<Vec<String>> {
    let allowed_tools = claude_allowed_tools(native_memory_root, teaching_bash_commands)?;
    let tools = if native_memory_root.is_some() {
        "Read,Bash,Write,Edit"
    } else {
        "Read,Bash"
    };
    let disallowed_tools = if native_memory_root.is_some() {
        "WebFetch,WebSearch,NotebookEdit,Task"
    } else {
        "Write,Edit,WebFetch,WebSearch,NotebookEdit,Task"
    };
    let (settings_argument, mcp_argument) = if let Some((settings, mcp)) = inline_config_json {
        (settings.to_string(), mcp.to_string())
    } else {
        (
            settings_path.canonicalize()?.display().to_string(),
            mcp_path.canonicalize()?.display().to_string(),
        )
    };
    let mut argv = vec![
        claude.path.clone(),
        "--print".to_string(),
        "--output-format".to_string(),
        "stream-json".to_string(),
        "--verbose".to_string(),
        "--permission-mode".to_string(),
        "dontAsk".to_string(),
        "--allowed-tools".to_string(),
        allowed_tools,
        "--tools".to_string(),
        tools.to_string(),
        "--disallowed-tools".to_string(),
        disallowed_tools.to_string(),
        "--setting-sources".to_string(),
        "project".to_string(),
        "--settings".to_string(),
        settings_argument,
        "--strict-mcp-config".to_string(),
        "--mcp-config".to_string(),
        mcp_argument,
        "--model".to_string(),
        model.to_string(),
        "--max-turns".to_string(),
        max_turns.to_string(),
        "--max-budget-usd".to_string(),
        format_budget_usd(budget_millis),
        "--add-dir".to_string(),
        cwd.canonicalize()?.display().to_string(),
        "--append-system-prompt-file".to_string(),
        instruction_path.canonicalize()?.display().to_string(),
    ];
    if no_session_persistence {
        argv.push("--no-session-persistence".to_string());
    }
    if let Some(schema_path) = schema_path {
        argv.extend([
            "--json-schema".to_string(),
            fs::read_to_string(schema_path)?,
        ]);
    }
    argv.push(prompt.to_string());
    Ok(argv)
}

pub(crate) fn claude_allowed_tools(
    native_memory_root: Option<&Path>,
    teaching_bash_commands: &[String],
) -> EvalResult<String> {
    let mut rules = vec![CLAUDE_ALLOWED_TOOLS.to_string()];
    rules.extend(
        teaching_bash_commands
            .iter()
            .map(|command| format!("Bash({command})")),
    );
    if let Some(path) = native_memory_root {
        rules.push(format!(
            "Edit(//{}/**)",
            path.canonicalize()?
                .display()
                .to_string()
                .trim_start_matches('/')
        ));
    }
    Ok(rules.join(","))
}

fn sha256_file(path: &Path) -> EvalResult<String> {
    let mut digest = Sha256::new();
    digest.update(fs::read(path)?);
    Ok(format!("{:x}", digest.finalize()))
}

fn apply_evaluation_prerequisite_state(
    case: &NativePilotCase,
    protocol_schema_version: u32,
    evaluation_checkout: &Path,
) -> EvalResult<()> {
    if case.evaluation_prerequisite_state == NativePilotEvaluationPrerequisiteState::Unchanged {
        return Ok(());
    }
    if protocol_schema_version < UNAVAILABLE_PREREQUISITE_NATIVE_PILOT_SCHEMA_VERSION {
        return Err(EvalError::Invalid(format!(
            "native pilot case {} cannot mutate prerequisite state before schema {UNAVAILABLE_PREREQUISITE_NATIVE_PILOT_SCHEMA_VERSION}",
            case.id
        )));
    }
    let source = case
        .procedure
        .prerequisite_sources
        .values()
        .next()
        .expect("validated source-backed prerequisite");
    let relative = Path::new(&source.relative_path);
    let checkout = evaluation_checkout.canonicalize()?;
    let path = checkout.join(relative);
    let metadata = fs::symlink_metadata(&path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(EvalError::Invalid(format!(
            "native pilot case {} can remove only a regular prerequisite source",
            case.id
        )));
    }
    let tracked = Command::new("git")
        .arg("-C")
        .arg(&checkout)
        .args(["--literal-pathspecs", "ls-files", "--error-unmatch", "--"])
        .arg(relative)
        .output()?;
    if !tracked.status.success() {
        return Err(EvalError::Invalid(format!(
            "native pilot case {} prerequisite source must be tracked before removal",
            case.id
        )));
    }
    fs::remove_file(&path)?;
    Ok(())
}

fn prepare_condition_evidence(
    case: &NativePilotCase,
    evaluation_checkout: &Path,
) -> EvalResult<Option<String>> {
    if case.expected_outcome == NativePilotExpectedOutcome::ExecuteProcedure {
        return Ok(None);
    }
    if case.evaluation_prerequisite_state == NativePilotEvaluationPrerequisiteState::Missing {
        let source = case
            .procedure
            .prerequisite_sources
            .values()
            .next()
            .expect("validated source-backed prerequisite");
        if case.condition_evidence_target.as_deref() != Some(source.relative_path.as_str())
            || evaluation_checkout.join(&source.relative_path).exists()
        {
            return Err(EvalError::Invalid(format!(
                "native pilot case {} missing-source evidence must target the absent prerequisite source",
                case.id
            )));
        }
        return Ok(None);
    }
    prepare_tracked_checkout_evidence(
        case,
        "condition",
        case.condition_evidence_target
            .as_deref()
            .expect("validated abstention condition target"),
        case.condition_evidence_output_contains
            .as_deref()
            .expect("validated abstention condition excerpt"),
        evaluation_checkout,
    )
}

fn prepare_prerequisite_source_observation(
    case: &NativePilotCase,
    protocol_schema_version: u32,
    teaching_checkout: &Path,
    evaluation_checkout: &Path,
) -> EvalResult<Option<NativePilotConditionObservation>> {
    if !schema_requires_source_backed_prerequisite(protocol_schema_version) {
        return Ok(None);
    }
    let (condition_key, source) = case
        .procedure
        .prerequisite_sources
        .iter()
        .next()
        .expect("validated schema-8 prerequisite source");
    let expected = case
        .procedure
        .conditions
        .get(condition_key)
        .expect("validated schema-8 prerequisite condition");
    let (teaching_value, _, _) =
        read_prerequisite_source(case, "teaching", teaching_checkout, source)?;
    if !teaching_value.trim().eq_ignore_ascii_case(expected.trim()) {
        return Err(EvalError::Invalid(format!(
            "native pilot case {} teaching prerequisite source does not match condition {condition_key}",
            case.id
        )));
    }
    if case.evaluation_prerequisite_state == NativePilotEvaluationPrerequisiteState::Missing {
        let missing = evaluation_checkout.join(&source.relative_path);
        if missing.exists() {
            return Err(EvalError::Invalid(format!(
                "native pilot case {} expected a missing evaluation prerequisite source",
                case.id
            )));
        }
        return Ok(Some(NativePilotConditionObservation {
            condition_key: condition_key.clone(),
            source: source.clone(),
            status: NativePilotConditionObservationStatus::Unavailable,
            source_sha256: None,
            source_output_contains: None,
            detail_contains: Some("configured source file is unavailable".to_string()),
        }));
    }
    let (evaluation_value, source_sha256, source_output_contains) =
        read_prerequisite_source(case, "evaluation", evaluation_checkout, source)?;
    let status = if evaluation_value
        .trim()
        .eq_ignore_ascii_case(expected.trim())
    {
        NativePilotConditionObservationStatus::Matched
    } else {
        NativePilotConditionObservationStatus::Mismatched
    };
    let expected_status = match (case.expected_outcome, case.procedure_expiry_seconds) {
        (NativePilotExpectedOutcome::Abstain, Some(_)) => {
            NativePilotConditionObservationStatus::Matched
        }
        (NativePilotExpectedOutcome::ExecuteProcedure, _) => {
            NativePilotConditionObservationStatus::Matched
        }
        (NativePilotExpectedOutcome::Abstain, None) => {
            NativePilotConditionObservationStatus::Mismatched
        }
    };
    if status != expected_status {
        return Err(EvalError::Invalid(format!(
            "native pilot case {} evaluation prerequisite source status {status:?} conflicts with expected outcome {:?}",
            case.id, case.expected_outcome
        )));
    }
    Ok(Some(NativePilotConditionObservation {
        condition_key: condition_key.clone(),
        source: source.clone(),
        status,
        source_sha256: Some(source_sha256),
        source_output_contains: Some(source_output_contains),
        detail_contains: None,
    }))
}

fn read_prerequisite_source(
    case: &NativePilotCase,
    phase: &str,
    checkout: &Path,
    source: &NativePilotPrerequisiteSource,
) -> EvalResult<(String, String, String)> {
    let relative_path = Path::new(&source.relative_path);
    if !is_safe_relative_path(relative_path) {
        return Err(EvalError::Invalid(format!(
            "native pilot case {} {phase} prerequisite source path is unsafe",
            case.id
        )));
    }
    let checkout = checkout.canonicalize()?;
    let path = checkout.join(relative_path);
    let metadata = fs::symlink_metadata(&path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > 64 * 1024 {
        return Err(EvalError::Invalid(format!(
            "native pilot case {} {phase} prerequisite source must be a regular non-symlink file no larger than 64 KiB",
            case.id
        )));
    }
    let path = path.canonicalize()?;
    if !path.starts_with(&checkout) {
        return Err(EvalError::Invalid(format!(
            "native pilot case {} {phase} prerequisite source escapes its checkout",
            case.id
        )));
    }
    let tracked = Command::new("git")
        .arg("-C")
        .arg(&checkout)
        .args(["--literal-pathspecs", "ls-files", "--error-unmatch", "--"])
        .arg(relative_path)
        .output()?;
    if !tracked.status.success() {
        return Err(EvalError::Invalid(format!(
            "native pilot case {} {phase} prerequisite source is not Git-tracked",
            case.id
        )));
    }
    let body = fs::read_to_string(&path)?;
    let document: toml::Value = toml::from_str(&body).map_err(|error| {
        EvalError::Invalid(format!(
            "native pilot case {} {phase} prerequisite source is invalid TOML: {error}",
            case.id
        ))
    })?;
    let mut selected = &document;
    for key in &source.key_path {
        selected = selected.get(key).ok_or_else(|| {
            EvalError::Invalid(format!(
                "native pilot case {} {phase} prerequisite source key path is absent",
                case.id
            ))
        })?;
    }
    let value = match selected {
        toml::Value::String(value) => value.clone(),
        toml::Value::Integer(value) => value.to_string(),
        toml::Value::Float(value) => value.to_string(),
        toml::Value::Boolean(value) => value.to_string(),
        toml::Value::Datetime(value) => value.to_string(),
        toml::Value::Array(_) | toml::Value::Table(_) => {
            return Err(EvalError::Invalid(format!(
                "native pilot case {} {phase} prerequisite source key path is not scalar",
                case.id
            )))
        }
    };
    let excerpt = body.trim().to_string();
    if excerpt.is_empty() {
        return Err(EvalError::Invalid(format!(
            "native pilot case {} {phase} prerequisite source is empty",
            case.id
        )));
    }
    Ok((value, sha256_file(&path)?, excerpt))
}

fn prepare_identity_evidence(
    case: &NativePilotCase,
    protocol_schema_version: u32,
    field: &str,
    target: Option<&str>,
    excerpt: Option<&str>,
    evaluation_checkout: &Path,
) -> EvalResult<Option<String>> {
    if protocol_schema_version < TRUSTED_SCOPED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION {
        return Ok(None);
    }
    let (Some(target), Some(excerpt)) = (target, excerpt) else {
        return Ok(None);
    };
    prepare_tracked_checkout_evidence(case, field, target, excerpt, evaluation_checkout)
}

fn prepare_tracked_checkout_evidence(
    case: &NativePilotCase,
    evidence_kind: &str,
    target: &str,
    excerpt: &str,
    evaluation_checkout: &Path,
) -> EvalResult<Option<String>> {
    let target = Path::new(target);
    if target.as_os_str().is_empty()
        || !target
            .components()
            .all(|component| matches!(component, std::path::Component::Normal(_)))
    {
        return Err(EvalError::Invalid(format!(
            "native pilot case {} {evidence_kind} evidence target must be a safe checkout-relative path",
            case.id,
        )));
    }
    let checkout = evaluation_checkout.canonicalize()?;
    let source = checkout.join(target);
    let metadata = fs::symlink_metadata(&source)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(EvalError::Invalid(format!(
            "native pilot case {} {evidence_kind} evidence must be a regular non-symlink file",
            case.id,
        )));
    }
    let source = source.canonicalize()?;
    if !source.starts_with(&checkout) {
        return Err(EvalError::Invalid(format!(
            "native pilot case {} {evidence_kind} evidence escapes its evaluation checkout",
            case.id,
        )));
    }
    let tracked = Command::new("git")
        .arg("-C")
        .arg(&checkout)
        .args(["ls-files", "--error-unmatch", "--"])
        .arg(target)
        .output()?;
    if !tracked.status.success() {
        return Err(EvalError::Invalid(format!(
            "native pilot case {} {evidence_kind} evidence is not Git-tracked",
            case.id,
        )));
    }
    if !fs::read_to_string(&source)?.contains(excerpt) {
        return Err(EvalError::Invalid(format!(
            "native pilot case {} {evidence_kind} evidence does not contain its frozen excerpt",
            case.id,
        )));
    }
    Ok(Some(sha256_file(&source)?))
}

fn system_time_unix_ms(value: SystemTime) -> EvalResult<u64> {
    let duration = value.duration_since(UNIX_EPOCH).map_err(|error| {
        EvalError::Invalid(format!("preparation time predates Unix epoch: {error}"))
    })?;
    u64::try_from(duration.as_millis())
        .map_err(|_| EvalError::Invalid("preparation timestamp overflow".to_string()))
}

#[cfg(not(unix))]
fn write_family_v3_private_file_exclusive(_path: &Path, _bytes: &[u8]) -> EvalResult<()> {
    Err(EvalError::Invalid(
        "family-v3 private Claude artifacts require a Unix host".to_string(),
    ))
}

#[cfg(unix)]
fn write_family_v3_private_file_exclusive(path: &Path, bytes: &[u8]) -> EvalResult<()> {
    use std::ffi::CString;
    use std::fs::File;
    use std::io::Write;
    use std::os::fd::{AsRawFd, FromRawFd};
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};

    let parent = path.parent().ok_or_else(|| {
        EvalError::Invalid("family-v3 Claude artifact has no parent directory".to_string())
    })?;
    let name = path.file_name().ok_or_else(|| {
        EvalError::Invalid("family-v3 Claude artifact has no file name".to_string())
    })?;
    let name = CString::new(name.as_bytes()).map_err(|_| {
        EvalError::Invalid("family-v3 Claude artifact name contains a NUL byte".to_string())
    })?;
    let effective_uid = unsafe { libc::geteuid() };
    let parent_before = fs::symlink_metadata(parent)?;
    if parent_before.file_type().is_symlink()
        || !parent_before.is_dir()
        || parent_before.uid() != effective_uid
        || parent_before.permissions().mode() & 0o077 != 0
    {
        return Err(EvalError::Invalid(format!(
            "family-v3 Claude artifact parent is not a private owner-only directory: {}",
            parent.display()
        )));
    }

    let mut parent_options = fs::OpenOptions::new();
    parent_options
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC);
    let parent_file = parent_options.open(parent)?;
    let parent_opened = parent_file.metadata()?;
    if parent_before.dev() != parent_opened.dev()
        || parent_before.ino() != parent_opened.ino()
        || parent_before.uid() != parent_opened.uid()
        || parent_before.mode() != parent_opened.mode()
    {
        return Err(EvalError::Invalid(format!(
            "family-v3 Claude artifact parent changed while opening: {}",
            parent.display()
        )));
    }

    let descriptor = unsafe {
        libc::openat(
            parent_file.as_raw_fd(),
            name.as_ptr(),
            libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            0o600,
        )
    };
    if descriptor < 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let mut file = unsafe { File::from_raw_fd(descriptor) };
    file.set_permissions(fs::Permissions::from_mode(0o600))?;
    file.write_all(bytes)?;
    file.sync_all()?;

    let opened = file.metadata()?;
    if !opened.is_file()
        || opened.uid() != effective_uid
        || opened.permissions().mode() & 0o777 != 0o600
        || opened.nlink() != 1
        || opened.len() != bytes.len() as u64
    {
        return Err(EvalError::Invalid(format!(
            "family-v3 Claude artifact is not an exact private owner-only single-link file: {}",
            path.display()
        )));
    }

    let reopened_descriptor = unsafe {
        libc::openat(
            parent_file.as_raw_fd(),
            name.as_ptr(),
            libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if reopened_descriptor < 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let reopened = unsafe { File::from_raw_fd(reopened_descriptor) };
    let reopened_metadata = reopened.metadata()?;
    if opened.dev() != reopened_metadata.dev()
        || opened.ino() != reopened_metadata.ino()
        || opened.len() != reopened_metadata.len()
        || opened.uid() != reopened_metadata.uid()
        || opened.mode() != reopened_metadata.mode()
        || opened.nlink() != reopened_metadata.nlink()
    {
        return Err(EvalError::Invalid(format!(
            "family-v3 Claude artifact identity changed after creation: {}",
            path.display()
        )));
    }
    parent_file.sync_all()?;

    let parent_after = fs::symlink_metadata(parent)?;
    if parent_after.file_type().is_symlink()
        || parent_before.dev() != parent_after.dev()
        || parent_before.ino() != parent_after.ino()
        || parent_before.uid() != parent_after.uid()
        || parent_before.mode() != parent_after.mode()
    {
        return Err(EvalError::Invalid(format!(
            "family-v3 Claude artifact parent changed after creation: {}",
            parent.display()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[cfg(target_os = "macos")]
    fn set_mode(path: &Path, mode: u32) {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
    }

    #[cfg(target_os = "macos")]
    fn runtime_test_library() -> PathBuf {
        let location = PathBuf::from(
            std::env::var("ORT_LIB_LOCATION")
                .expect("ORT_LIB_LOCATION must identify the pinned ONNX runtime for this test"),
        );
        [
            location.join(NATIVE_PILOT_ONNXRUNTIME_DYLIB),
            location.join("lib").join(NATIVE_PILOT_ONNXRUNTIME_DYLIB),
        ]
        .into_iter()
        .find(|path| path.is_file())
        .expect("pinned ONNX runtime library was not found under ORT_LIB_LOCATION")
    }

    #[cfg(target_os = "macos")]
    fn materialize_runtime_test_consumer(root: &Path, name: &str) -> PathBuf {
        let bin = root.join("bin");
        let lib = root.join("lib");
        fs::create_dir_all(&bin).unwrap();
        fs::create_dir_all(&lib).unwrap();
        for directory in [root, bin.as_path(), lib.as_path()] {
            set_mode(directory, 0o700);
        }
        let executable = bin.join(name);
        fs::copy(std::env::current_exe().unwrap(), &executable).unwrap();
        set_mode(&executable, 0o700);
        let load_commands = Command::new("/usr/bin/otool")
            .args(["-l", executable.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(load_commands.status.success());
        if !String::from_utf8(load_commands.stdout)
            .unwrap()
            .contains(NATIVE_PILOT_ONNXRUNTIME_LOADER_RPATH)
        {
            let status = Command::new("/usr/bin/install_name_tool")
                .args([
                    "-add_rpath",
                    NATIVE_PILOT_ONNXRUNTIME_LOADER_RPATH,
                    executable.to_str().unwrap(),
                ])
                .status()
                .unwrap();
            assert!(status.success());
            set_mode(&executable, 0o700);
        }
        let library = lib.join(NATIVE_PILOT_ONNXRUNTIME_DYLIB);
        fs::copy(runtime_test_library(), &library).unwrap();
        set_mode(&library, 0o600);
        executable
    }

    #[cfg(unix)]
    fn assert_private_owner_only_single_link(path: &Path) {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};

        let metadata = fs::symlink_metadata(path).unwrap();
        assert!(metadata.is_file());
        assert!(!metadata.file_type().is_symlink());
        assert_eq!(metadata.uid(), unsafe { libc::geteuid() });
        assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
        assert_eq!(metadata.nlink(), 1);
    }

    fn exact_argv_option<'a>(argv: &'a [String], option: &str) -> &'a str {
        let values = argv
            .windows(2)
            .filter(|pair| pair[0] == option)
            .map(|pair| pair[1].as_str())
            .collect::<Vec<_>>();
        assert_eq!(values.len(), 1, "expected one {option} value");
        values[0]
    }

    fn assert_family_v3_inline_claude_config(lane: &PreparedNativeLane) -> (PathBuf, PathBuf) {
        let lane_root = Path::new(&lane.acceptance_contract).parent().unwrap();
        let settings_path = lane_root.join("claude-settings.json");
        let mcp_path = lane_root.join(if lane.memory_layer.uses_engram() {
            "claude-mcp.json"
        } else {
            "claude-mcp-empty.json"
        });
        let settings_bytes = fs::read(&settings_path).unwrap();
        let mcp_bytes = fs::read(&mcp_path).unwrap();
        let settings_value: serde_json::Value = serde_json::from_slice(&settings_bytes).unwrap();
        let mcp_value: serde_json::Value = serde_json::from_slice(&mcp_bytes).unwrap();
        let settings_compact = serde_json::to_string(&settings_value).unwrap();
        let mcp_compact = serde_json::to_string(&mcp_value).unwrap();

        assert_eq!(settings_bytes, settings_compact.as_bytes());
        assert_eq!(mcp_bytes, mcp_compact.as_bytes());
        for argv in [&lane.teaching_argv, &lane.evaluation_argv] {
            assert_eq!(exact_argv_option(argv, "--settings"), settings_compact);
            assert_eq!(exact_argv_option(argv, "--mcp-config"), mcp_compact);
            assert_ne!(
                exact_argv_option(argv, "--settings"),
                settings_path.display().to_string()
            );
            assert_ne!(
                exact_argv_option(argv, "--mcp-config"),
                mcp_path.display().to_string()
            );
        }
        assert_eq!(
            settings_value,
            serde_json::json!({
                "autoMemoryEnabled": lane.memory_layer.uses_native(),
                "autoMemoryDirectory": lane_root
                    .join("claude-memory")
                    .canonicalize()
                    .unwrap()
                    .display()
                    .to_string(),
            })
        );
        if lane.memory_layer.uses_engram() {
            assert_eq!(
                mcp_value,
                serde_json::json!({
                    "mcpServers": {
                        "engram": {
                            "type": "stdio",
                            "command": lane.cleanup_argv.as_ref().unwrap()[0].as_str(),
                            "args": [
                                "serve",
                                "--project",
                                lane.engram_project.as_deref().unwrap(),
                                "--profile",
                                "agent",
                            ],
                            "env": {
                                "ENGRAM_HOME": lane.engram_home.as_deref().unwrap(),
                            },
                        }
                    }
                })
            );
        } else {
            assert_eq!(mcp_value, serde_json::json!({"mcpServers": {}}));
        }
        #[cfg(unix)]
        {
            assert_private_owner_only_single_link(&settings_path);
            assert_private_owner_only_single_link(&mcp_path);
        }
        (settings_path, mcp_path)
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn family_v3_runtime_loader_output_contract_is_exact() {
        let executable = "/private/tmp/runtime/bin/engram";
        let library = "/private/tmp/runtime/lib/libonnxruntime.1.20.0.dylib";
        let linked = format!(
            "{executable}:\n\t{NATIVE_PILOT_ONNXRUNTIME_INSTALL_NAME} (compatibility version 0.0.0, current version 1.20.0)\n\t/usr/lib/libSystem.B.dylib (compatibility version 1.0.0, current version 1.0.0)\n"
        );
        let load = format!(
            "Load command 1\n          cmd LC_RPATH\n      cmdsize 40\n         path {NATIVE_PILOT_ONNXRUNTIME_LOADER_RPATH} (offset 12)\n"
        );
        let install = format!("{library}:\n{NATIVE_PILOT_ONNXRUNTIME_INSTALL_NAME}\n");
        validate_native_pilot_runtime_loader_outputs(&linked, &load, &install, library).unwrap();

        for (mutated_linked, mutated_load, mutated_install) in [
            (
                linked.replace("@rpath/", "/private/tmp/foreign/"),
                load.clone(),
                install.clone(),
            ),
            (
                format!(
                    "{linked}\t{NATIVE_PILOT_ONNXRUNTIME_INSTALL_NAME} (compatibility version 0.0.0, current version 1.20.0)\n"
                ),
                load.clone(),
                install.clone(),
            ),
            (
                linked.clone(),
                load.replace(NATIVE_PILOT_ONNXRUNTIME_LOADER_RPATH, "@executable_path/../lib"),
                install.clone(),
            ),
            (
                linked.clone(),
                format!("{load}{load}"),
                install.clone(),
            ),
            (
                linked.clone(),
                load.clone(),
                install.replace(NATIVE_PILOT_ONNXRUNTIME_INSTALL_NAME, "@rpath/libother.dylib"),
            ),
        ] {
            assert!(validate_native_pilot_runtime_loader_outputs(
                &mutated_linked,
                &mutated_load,
                &mutated_install,
                library,
            )
            .is_err());
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn family_v3_runtime_library_attestation_binds_two_consumers_and_live_identity() {
        let root = tempfile::tempdir().unwrap();
        let engram_root = root.path().join("engram-bundle");
        let evaluator_root = root.path().join("evaluator-bundle");
        fs::create_dir(&engram_root).unwrap();
        fs::create_dir(&evaluator_root).unwrap();
        set_mode(&engram_root, 0o700);
        set_mode(&evaluator_root, 0o700);
        let engram = materialize_runtime_test_consumer(&engram_root, "engram");
        let evaluator = materialize_runtime_test_consumer(&evaluator_root, "engram-eval");

        let attestation =
            attest_native_pilot_family_v3_runtime_libraries(&engram, &evaluator).unwrap();
        assert_eq!(
            attestation.keys().map(String::as_str).collect::<Vec<_>>(),
            [NATIVE_PILOT_ONNXRUNTIME_LIBRARY_KEY]
        );
        let runtime = &attestation[NATIVE_PILOT_ONNXRUNTIME_LIBRARY_KEY];
        assert_eq!(runtime.install_name, NATIVE_PILOT_ONNXRUNTIME_INSTALL_NAME);
        assert_eq!(
            runtime.sha256,
            sha256_file(&runtime_test_library()).unwrap()
        );
        assert_eq!(
            runtime
                .consumers
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            [
                NATIVE_PILOT_ENGRAM_RUNTIME_CONSUMER_KEY,
                NATIVE_PILOT_EVALUATOR_RUNTIME_CONSUMER_KEY,
            ]
        );
        let binaries = BTreeMap::from([
            (
                NATIVE_PILOT_ENGRAM_RUNTIME_CONSUMER_KEY.to_string(),
                PilotBinaryAttestation {
                    path: engram.display().to_string(),
                    version: "test".to_string(),
                    sha256: sha256_file(&engram).unwrap(),
                },
            ),
            (
                NATIVE_PILOT_EVALUATOR_RUNTIME_CONSUMER_KEY.to_string(),
                PilotBinaryAttestation {
                    path: evaluator.display().to_string(),
                    version: "test".to_string(),
                    sha256: sha256_file(&evaluator).unwrap(),
                },
            ),
        ]);
        validate_native_pilot_family_v3_runtime_libraries(&binaries, &attestation).unwrap();

        let mut drifted_attestation = attestation.clone();
        drifted_attestation
            .get_mut(NATIVE_PILOT_ONNXRUNTIME_LIBRARY_KEY)
            .unwrap()
            .consumers
            .get_mut(NATIVE_PILOT_EVALUATOR_RUNTIME_CONSUMER_KEY)
            .unwrap()
            .loader_rpath = "@executable_path/../lib".to_string();
        assert!(
            validate_native_pilot_family_v3_runtime_libraries(&binaries, &drifted_attestation,)
                .is_err()
        );

        let evaluator_library = evaluator_root
            .join("lib")
            .join(NATIVE_PILOT_ONNXRUNTIME_DYLIB);
        let mut bytes = fs::read(&evaluator_library).unwrap();
        bytes.push(0);
        fs::write(&evaluator_library, bytes).unwrap();
        set_mode(&evaluator_library, 0o600);
        assert!(
            validate_native_pilot_family_v3_runtime_libraries(&binaries, &attestation).is_err()
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn family_v3_runtime_library_attestation_rejects_unsafe_file_shapes() {
        use std::os::unix::fs::symlink;

        let root = tempfile::tempdir().unwrap();
        let engram_root = root.path().join("engram-bundle");
        let evaluator_root = root.path().join("evaluator-bundle");
        fs::create_dir(&engram_root).unwrap();
        fs::create_dir(&evaluator_root).unwrap();
        set_mode(&engram_root, 0o700);
        set_mode(&evaluator_root, 0o700);
        let engram = materialize_runtime_test_consumer(&engram_root, "engram");
        let evaluator = materialize_runtime_test_consumer(&evaluator_root, "engram-eval");
        let evaluator_library = evaluator_root
            .join("lib")
            .join(NATIVE_PILOT_ONNXRUNTIME_DYLIB);

        set_mode(&evaluator_library, 0o644);
        assert!(attest_native_pilot_family_v3_runtime_libraries(&engram, &evaluator).is_err());
        set_mode(&evaluator_library, 0o600);

        let linked = evaluator_root.join("lib/linked.dylib");
        fs::hard_link(&evaluator_library, &linked).unwrap();
        assert!(attest_native_pilot_family_v3_runtime_libraries(&engram, &evaluator).is_err());
        fs::remove_file(&linked).unwrap();

        let original = evaluator_root.join("lib/original.dylib");
        fs::rename(&evaluator_library, &original).unwrap();
        symlink(&original, &evaluator_library).unwrap();
        assert!(attest_native_pilot_family_v3_runtime_libraries(&engram, &evaluator).is_err());
    }

    #[test]
    fn agent_profile_contract_validation_accepts_exact_v3_and_v4_shapes() {
        let binary = PilotBinaryAttestation {
            path: "/tmp/engram".to_string(),
            version: format!("engram {}", env!("CARGO_PKG_VERSION")),
            sha256: "a".repeat(64),
        };
        let contract = EngramMcpContractAttestation {
            schema_version: 3,
            cli_version: env!("CARGO_PKG_VERSION").to_string(),
            health_schema_version: 2,
            mcp_contract_version: 2,
            mcp_protocol_version: "2024-11-05".to_string(),
            profile: "agent".to_string(),
            mcp_tool_count: 6,
            mcp_tools_sha256: "b".repeat(64),
            profile_instructions_sha256: Some("c".repeat(64)),
            effective_runtime: Some(EngramMcpRuntimeAttestation {
                verified: true,
                mcp_tool_count: 6,
                mcp_tools_sha256: "b".repeat(64),
                profile_instructions_sha256: "c".repeat(64),
                restricted_tool_rejected: true,
                review_authority_rejected: true,
                correction_proposal_path_verified: None,
                direct_correction_unavailable: None,
                correction_apply_unavailable: None,
                correction_verification_unavailable: None,
                correction_inspection_unavailable: None,
            }),
            executable_path: binary.path.clone(),
            executable_sha256: binary.sha256.clone(),
        };
        validate_engram_mcp_contract(&contract, &binary).unwrap();
        let legacy_fixture: EngramMcpContractAttestation =
            serde_json::from_value(serde_json::to_value(&contract).unwrap()).unwrap();
        assert_eq!(legacy_fixture, contract);

        let mut drifted = contract.clone();
        drifted.mcp_tool_count = 5;
        assert!(validate_engram_mcp_contract(&drifted, &binary).is_err());

        let mut drifted = contract.clone();
        drifted.effective_runtime.as_mut().unwrap().mcp_tools_sha256 = "e".repeat(64);
        assert!(validate_engram_mcp_contract(&drifted, &binary).is_err());

        let mut current = contract.clone();
        current.schema_version = 4;
        current.health_schema_version = 3;
        current.mcp_contract_version = 5;
        current.mcp_protocol_version = "2024-11-05".to_string();
        let runtime = current.effective_runtime.as_mut().unwrap();
        runtime.correction_proposal_path_verified = Some(true);
        runtime.direct_correction_unavailable = Some(true);
        runtime.correction_apply_unavailable = Some(true);
        runtime.correction_verification_unavailable = Some(true);
        runtime.correction_inspection_unavailable = Some(true);
        validate_engram_mcp_contract(&current, &binary).unwrap();
        let current_fixture: EngramMcpContractAttestation =
            serde_json::from_value(serde_json::to_value(&current).unwrap()).unwrap();
        assert_eq!(current_fixture, current);

        let mut missing_v4_proof = current.clone();
        missing_v4_proof
            .effective_runtime
            .as_mut()
            .unwrap()
            .correction_apply_unavailable = None;
        assert!(validate_engram_mcp_contract(&missing_v4_proof, &binary).is_err());

        let mut false_verification_proof = current.clone();
        false_verification_proof
            .effective_runtime
            .as_mut()
            .unwrap()
            .correction_verification_unavailable = Some(false);
        assert!(validate_engram_mcp_contract(&false_verification_proof, &binary).is_err());

        let mut missing_verification_proof = current.clone();
        missing_verification_proof
            .effective_runtime
            .as_mut()
            .unwrap()
            .correction_verification_unavailable = None;
        assert!(validate_engram_mcp_contract(&missing_verification_proof, &binary).is_err());
        let mut wrong_v4_contract = current.clone();
        wrong_v4_contract.mcp_contract_version = 3;
        assert!(validate_engram_mcp_contract(&wrong_v4_contract, &binary).is_err());

        let mut legacy_with_v4_fields = contract.clone();
        legacy_with_v4_fields
            .effective_runtime
            .as_mut()
            .unwrap()
            .correction_proposal_path_verified = Some(true);
        assert!(validate_engram_mcp_contract(&legacy_with_v4_fields, &binary).is_err());

        let mut legacy_with_verification_proof = contract.clone();
        legacy_with_verification_proof
            .effective_runtime
            .as_mut()
            .unwrap()
            .correction_verification_unavailable = Some(true);
        assert!(validate_engram_mcp_contract(&legacy_with_verification_proof, &binary).is_err());

        let mut unknown_field = serde_json::to_value(&current).unwrap();
        unknown_field["effective_runtime"]["unattested_boundary"] = serde_json::json!(true);
        assert!(serde_json::from_value::<EngramMcpContractAttestation>(unknown_field).is_err());

        let mut drifted = contract;
        drifted.executable_sha256 = "d".repeat(64);
        assert!(validate_engram_mcp_contract(&drifted, &binary).is_err());
    }

    fn case() -> NativePilotCase {
        NativePilotCase {
            id: "procedure".to_string(),
            teaching_cwd: "fixture://atlas/main/services/worker".to_string(),
            evaluation_cwd: "fixture://moved/arbitrary-name/services/worker".to_string(),
            evaluation_prompt: "Run the context probe using the learned procedure.".to_string(),
            expected_repository_remote: "https://github.com/acme/atlas".to_string(),
            expected_project: Some("atlas".to_string()),
            expected_project_status: None,
            expected_project_confirmation_required: None,
            expected_component: Some("queue-worker".to_string()),
            expected_first_action: "run_verified_procedure".to_string(),
            expected_outcome: NativePilotExpectedOutcome::ExecuteProcedure,
            condition_evidence_target: None,
            condition_evidence_output_contains: None,
            procedure_query_max_chars: None,
            procedure_query_required_terms: Vec::new(),
            evaluation_prerequisite_state: NativePilotEvaluationPrerequisiteState::Unchanged,
            procedure_expiry_seconds: None,
            project_evidence_target: Some("runbooks/deploy-worker.md".to_string()),
            project_evidence_output_contains: Some("Atlas".to_string()),
            component_evidence_target: Some("services/worker/component.json".to_string()),
            component_evidence_output_contains: Some("queue-worker".to_string()),
            procedure: LearnedProcedure {
                context_key: "procedure-atlas-context-probe-v1".to_string(),
                task: "run context probe".to_string(),
                failed_commands: vec!["./bin/context-probe --channel amber".to_string()],
                command: "./bin/context-probe --channel cobalt".to_string(),
                expected_exit_code: 0,
                expected_output_contains: "ATLAS_CONTEXT_PROBE_OK".to_string(),
                conditions: BTreeMap::from([("tool.version".to_string(), "3".to_string())]),
                prerequisite_sources: BTreeMap::from([(
                    "tool.version".to_string(),
                    NativePilotPrerequisiteSource {
                        format: "toml".to_string(),
                        relative_path: "toolchain.toml".to_string(),
                        key_path: vec!["tools".to_string(), "version".to_string()],
                    },
                )]),
            },
        }
    }

    fn arms() -> Vec<NativePilotArm> {
        ["codex", "claude_code"]
            .into_iter()
            .flat_map(|host| {
                [MemoryLayer::Native, MemoryLayer::Engram, MemoryLayer::Both]
                    .into_iter()
                    .map(move |memory_layer| NativePilotArm {
                        id: format!("{host}-{memory_layer:?}"),
                        host: host.to_string(),
                        memory_layer,
                    })
            })
            .collect()
    }

    fn protocol() -> NativePilotProtocol {
        let arms = arms();
        let run_order = arms
            .iter()
            .map(|arm| NativePilotRunRef {
                case_id: "procedure".to_string(),
                arm: arm.id.clone(),
                repetition: 1,
            })
            .collect();
        NativePilotProtocol {
            schema_version: NATIVE_PILOT_SCHEMA_VERSION,
            pilot_id: "native-pilot".to_string(),
            cases: vec![case()],
            arms,
            repetitions: 1,
            run_order,
            codex_min_idle_hours: 1,
            codex_authentication_mode: CodexAuthenticationMode::ChatgptBrowserKeyring,
            resource_budgets: None,
            claude_budget_cents: 50,
            claude_prior_spend_microusd: 0,
            claude_authorized_ceiling_cents: 50,
            claude_model: "claude-haiku-4-5".to_string(),
            claude_max_turns: 12,
            requires_explicit_execution_approval: true,
        }
    }

    fn clear_trusted_identity_evidence(case: &mut NativePilotCase) {
        case.project_evidence_target = None;
        case.project_evidence_output_contains = None;
        case.component_evidence_target = None;
        case.component_evidence_output_contains = None;
    }

    fn clear_prerequisite_sources(case: &mut NativePilotCase) {
        case.procedure.prerequisite_sources.clear();
    }

    #[test]
    fn protocol_requires_complete_six_arm_matrix_and_provider_gate() {
        validate_protocol(&protocol()).unwrap();

        let mut schema_seven_source = protocol();
        schema_seven_source.schema_version = TRUSTED_SCOPED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION;
        assert!(validate_protocol(&schema_seven_source)
            .unwrap_err()
            .to_string()
            .contains("requires schema 8 for prerequisite sources"));

        let mut unsafe_source = protocol();
        unsafe_source.cases[0]
            .procedure
            .prerequisite_sources
            .get_mut("tool.version")
            .unwrap()
            .relative_path = "../toolchain.toml".to_string();
        assert!(validate_protocol(&unsafe_source)
            .unwrap_err()
            .to_string()
            .contains("invalid TOML prerequisite source"));

        let mut unknown_source = protocol();
        let source = unknown_source.cases[0]
            .procedure
            .prerequisite_sources
            .remove("tool.version")
            .unwrap();
        unknown_source.cases[0]
            .procedure
            .prerequisite_sources
            .insert("tool.channel".to_string(), source);
        assert!(validate_protocol(&unknown_source)
            .unwrap_err()
            .to_string()
            .contains("has no matching condition"));

        let mut legacy = protocol();
        legacy.schema_version = LEGACY_NATIVE_PILOT_SCHEMA_VERSION;
        clear_trusted_identity_evidence(&mut legacy.cases[0]);
        clear_prerequisite_sources(&mut legacy.cases[0]);
        validate_protocol(&legacy).unwrap();

        let mut identity_schema = protocol();
        identity_schema.schema_version = IDENTITY_NATIVE_PILOT_SCHEMA_VERSION;
        clear_trusted_identity_evidence(&mut identity_schema.cases[0]);
        clear_prerequisite_sources(&mut identity_schema.cases[0]);
        validate_protocol(&identity_schema).unwrap();

        let mut ambiguous = protocol();
        ambiguous.cases[0].expected_project = None;
        ambiguous.cases[0].project_evidence_target = None;
        ambiguous.cases[0].project_evidence_output_contains = None;
        validate_protocol(&ambiguous).unwrap();
        ambiguous.schema_version = LEGACY_NATIVE_PILOT_SCHEMA_VERSION;
        clear_trusted_identity_evidence(&mut ambiguous.cases[0]);
        clear_prerequisite_sources(&mut ambiguous.cases[0]);
        assert!(validate_protocol(&ambiguous)
            .unwrap_err()
            .to_string()
            .contains("requires concrete project and component identities"));

        let mut abstention = protocol();
        abstention.cases[0].expected_outcome = NativePilotExpectedOutcome::Abstain;
        abstention.cases[0].evaluation_cwd = "fixture://atlas/legacy/services/worker".to_string();
        abstention.cases[0].expected_first_action = "inspect_procedure_prerequisites".to_string();
        abstention.cases[0].condition_evidence_target = Some("toolchain.toml".to_string());
        abstention.cases[0].condition_evidence_output_contains =
            Some("version = \"2\"".to_string());
        validate_protocol(&abstention).unwrap();
        abstention.schema_version = IDENTITY_NATIVE_PILOT_SCHEMA_VERSION;
        clear_trusted_identity_evidence(&mut abstention.cases[0]);
        clear_prerequisite_sources(&mut abstention.cases[0]);
        assert!(validate_protocol(&abstention)
            .unwrap_err()
            .to_string()
            .contains("expected abstention"));

        let mut missing_condition_evidence = protocol();
        missing_condition_evidence.cases[0].expected_outcome = NativePilotExpectedOutcome::Abstain;
        assert!(validate_protocol(&missing_condition_evidence)
            .unwrap_err()
            .to_string()
            .contains("condition_evidence_target"));

        let mut missing_condition_excerpt = protocol();
        missing_condition_excerpt.cases[0].expected_outcome = NativePilotExpectedOutcome::Abstain;
        missing_condition_excerpt.cases[0].condition_evidence_target =
            Some("toolchain.toml".to_string());
        assert!(validate_protocol(&missing_condition_excerpt)
            .unwrap_err()
            .to_string()
            .contains("condition_evidence_output_contains"));

        let mut unavailable = protocol();
        unavailable.cases[0].expected_outcome = NativePilotExpectedOutcome::Abstain;
        unavailable.cases[0].evaluation_cwd = "fixture://atlas/legacy/services/worker".to_string();
        unavailable.cases[0].expected_first_action = "inspect_procedure_prerequisites".to_string();
        unavailable.cases[0].condition_evidence_target = Some("toolchain.toml".to_string());
        unavailable.cases[0].condition_evidence_output_contains = None;
        unavailable.cases[0].evaluation_prerequisite_state =
            NativePilotEvaluationPrerequisiteState::Missing;
        validate_protocol(&unavailable).unwrap();

        let mut schema_eight_unavailable = unavailable.clone();
        schema_eight_unavailable.schema_version =
            SOURCE_BACKED_PREREQUISITE_NATIVE_PILOT_SCHEMA_VERSION;
        assert!(validate_protocol(&schema_eight_unavailable)
            .unwrap_err()
            .to_string()
            .contains("requires schema 9"));

        let mut executing_without_source = protocol();
        executing_without_source.cases[0].evaluation_prerequisite_state =
            NativePilotEvaluationPrerequisiteState::Missing;
        assert!(validate_protocol(&executing_without_source)
            .unwrap_err()
            .to_string()
            .contains("only when abstention is expected"));

        let mut missing_component_evidence = protocol();
        missing_component_evidence.cases[0].component_evidence_target = None;
        assert!(validate_protocol(&missing_component_evidence)
            .unwrap_err()
            .to_string()
            .contains("component_evidence_target"));

        let mut legacy_identity_evidence = protocol();
        legacy_identity_evidence.schema_version = ABSTENTION_NATIVE_PILOT_SCHEMA_VERSION;
        clear_prerequisite_sources(&mut legacy_identity_evidence.cases[0]);
        assert!(validate_protocol(&legacy_identity_evidence)
            .unwrap_err()
            .to_string()
            .contains("trusted scoped-identity evidence"));

        let mut unrelated_component_evidence = protocol();
        unrelated_component_evidence.cases[0].component_evidence_output_contains =
            Some("unrelated-service".to_string());
        assert!(validate_protocol(&unrelated_component_evidence)
            .unwrap_err()
            .to_string()
            .contains("must contain expected component identity"));

        let mut unsupported = protocol();
        unsupported.schema_version = REQUIRED_HOST_ACTION_NATIVE_PILOT_SCHEMA_VERSION + 1;
        assert!(validate_protocol(&unsupported)
            .unwrap_err()
            .to_string()
            .contains("unsupported native pilot schema version"));

        let mut incomplete = protocol();
        let removed = incomplete.arms.pop().unwrap();
        incomplete.run_order.retain(|run| run.arm != removed.id);
        assert!(validate_protocol(&incomplete)
            .unwrap_err()
            .to_string()
            .contains("both Codex and Claude Code"));

        let mut ungated = protocol();
        ungated.requires_explicit_execution_approval = false;
        assert!(validate_protocol(&ungated)
            .unwrap_err()
            .to_string()
            .contains("explicit provider execution approval"));

        let mut over_budget = protocol();
        over_budget.claude_prior_spend_microusd = 1;
        assert!(validate_protocol(&over_budget)
            .unwrap_err()
            .to_string()
            .contains("exceeds the authorized ceiling"));
    }

    #[test]
    fn schema_twelve_requires_consistent_structured_project_authorization() {
        let mut structured_case = case();
        structured_case.expected_project = None;
        structured_case.expected_project_status =
            Some(NativePilotExpectedProjectStatus::RequiresConfirmation);
        structured_case.expected_project_confirmation_required = Some(true);
        structured_case.expected_first_action = "resolve_checkout_identity".to_string();
        structured_case.expected_outcome = NativePilotExpectedOutcome::Abstain;
        structured_case.condition_evidence_target = Some("runbooks/deploy-worker.md".to_string());
        structured_case.condition_evidence_output_contains = Some("ORBIT_ONLY_CANARY".to_string());
        structured_case.project_evidence_target = None;
        structured_case.project_evidence_output_contains = None;
        structured_case.procedure.conditions.clear();
        structured_case.procedure.prerequisite_sources.clear();
        validate_case(
            &structured_case,
            STRUCTURED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION,
        )
        .unwrap();

        let mut missing_status = structured_case.clone();
        missing_status.expected_project_status = None;
        assert!(validate_case(
            &missing_status,
            STRUCTURED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION,
        )
        .unwrap_err()
        .to_string()
        .contains("requires expected_project_status"));

        let mut invented_project = structured_case;
        invented_project.expected_project = Some("orbit".to_string());
        assert!(validate_case(
            &invented_project,
            STRUCTURED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION,
        )
        .unwrap_err()
        .to_string()
        .contains("inconsistent project identity"));

        validate_case(&case(), EXPIRED_PROCEDURE_NATIVE_PILOT_SCHEMA_VERSION).unwrap();
    }

    #[test]
    fn schema_eleven_allows_only_a_complete_claude_diagnostic_matrix() {
        let mut diagnostic = protocol();
        diagnostic.schema_version = TARGETED_HOST_DIAGNOSTIC_NATIVE_PILOT_SCHEMA_VERSION;
        diagnostic.arms.retain(|arm| arm.host == "claude_code");
        diagnostic
            .run_order
            .retain(|run| diagnostic.arms.iter().any(|arm| arm.id == run.arm));
        validate_protocol(&diagnostic).unwrap();

        let mut incomplete = diagnostic.clone();
        let removed = incomplete.arms.pop().unwrap();
        incomplete.run_order.retain(|run| run.arm != removed.id);
        assert!(validate_protocol(&incomplete)
            .unwrap_err()
            .to_string()
            .contains("native, Engram, and combined layers for Claude Code only"));

        let mut cross_host = protocol();
        cross_host.schema_version = TARGETED_HOST_DIAGNOSTIC_NATIVE_PILOT_SCHEMA_VERSION;
        assert!(validate_protocol(&cross_host)
            .unwrap_err()
            .to_string()
            .contains("for Claude Code only"));
    }

    #[test]
    fn schema_ten_procedure_expiry_contract_is_strict() {
        let mut expiring = protocol();
        expiring.cases[0].expected_outcome = NativePilotExpectedOutcome::Abstain;
        expiring.cases[0].expected_first_action = "inspect_procedure_prerequisites".to_string();
        expiring.cases[0].condition_evidence_target = Some("toolchain.toml".to_string());
        expiring.cases[0].condition_evidence_output_contains = Some("version = \"3\"".to_string());
        expiring.cases[0].procedure_expiry_seconds = Some(300);
        validate_protocol(&expiring).unwrap();

        let mut schema_nine = expiring.clone();
        schema_nine.schema_version = UNAVAILABLE_PREREQUISITE_NATIVE_PILOT_SCHEMA_VERSION;
        assert!(validate_protocol(&schema_nine)
            .unwrap_err()
            .to_string()
            .contains("requires schema 10 for procedure expiry"));

        let mut zero = expiring.clone();
        zero.cases[0].procedure_expiry_seconds = Some(0);
        assert!(validate_protocol(&zero)
            .unwrap_err()
            .to_string()
            .contains("requires positive seconds"));

        let mut executing = expiring.clone();
        executing.cases[0].expected_outcome = NativePilotExpectedOutcome::ExecuteProcedure;
        assert!(validate_protocol(&executing)
            .unwrap_err()
            .to_string()
            .contains("expected abstention"));

        let mut missing_source = expiring.clone();
        missing_source.cases[0].evaluation_prerequisite_state =
            NativePilotEvaluationPrerequisiteState::Missing;
        missing_source.cases[0].condition_evidence_output_contains = None;
        assert!(validate_protocol(&missing_source)
            .unwrap_err()
            .to_string()
            .contains("unchanged prerequisite source"));

        let mut after_activation = expiring;
        after_activation.cases[0].procedure_expiry_seconds = Some(60 * 60);
        assert!(validate_protocol(&after_activation)
            .unwrap_err()
            .to_string()
            .contains("must precede the frozen Codex activation interval"));
    }

    #[test]
    fn schema_ten_expired_procedure_protocol_is_valid() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../evals/native_memory_pilot_v1/protocol-schema-10-expired-procedure-forward.json",
        );
        let protocol: NativePilotProtocol =
            serde_json::from_reader(fs::File::open(path).unwrap()).unwrap();

        validate_protocol(&protocol).unwrap();
        assert_eq!(protocol.schema_version, 10);
        assert_eq!(protocol.cases[0].procedure_expiry_seconds, Some(300));
        assert_eq!(
            protocol.cases[0].expected_outcome,
            NativePilotExpectedOutcome::Abstain
        );
        assert_eq!(
            protocol.cases[0].evaluation_prerequisite_state,
            NativePilotEvaluationPrerequisiteState::Unchanged
        );
        assert_eq!(protocol.claude_budget_cents, 50);
        assert_eq!(protocol.claude_prior_spend_microusd, 6_752_912);
        assert_eq!(protocol.claude_authorized_ceiling_cents, 750);
    }

    #[test]
    fn schema_twelve_protocol_freezes_structured_identity_no_result_contract() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../evals/native_memory_pilot_v1/protocol-schema-12-structured-identity-no-result-forward-file-cache-bounded.json",
        );
        let protocol: NativePilotProtocol =
            serde_json::from_reader(fs::File::open(path).unwrap()).unwrap();

        validate_protocol(&protocol).unwrap();
        assert_eq!(
            protocol.pilot_id,
            "native-memory-pilot-v1-structured-identity-no-result-forward-file-cache-bounded"
        );
        assert_eq!(
            protocol.schema_version,
            STRUCTURED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION
        );
        assert_eq!(protocol.cases.len(), 1);
        assert_eq!(protocol.repetitions, 1);
        assert_eq!(protocol.run_order.len(), 6);
        let case = &protocol.cases[0];
        assert_eq!(case.expected_outcome, NativePilotExpectedOutcome::Abstain);
        assert_eq!(
            case.expected_repository_remote,
            "https://github.com/acme/orbit"
        );
        assert_eq!(case.expected_project, None);
        assert_eq!(
            case.expected_project_status,
            Some(NativePilotExpectedProjectStatus::RequiresConfirmation)
        );
        assert_eq!(case.expected_project_confirmation_required, Some(true));
        assert_eq!(case.expected_component.as_deref(), Some("queue-worker"));
        assert_eq!(case.expected_first_action, "resolve_checkout_identity");
        assert!(case.procedure.conditions.is_empty());
        assert!(case.procedure.prerequisite_sources.is_empty());
        assert_eq!(
            protocol.codex_authentication_mode,
            CodexAuthenticationMode::ChatgptFileCache
        );
        assert_eq!(
            protocol.resource_budgets,
            Some(NativePilotResourceBudgets {
                max_engram_result_bytes_per_call: 8_192,
                max_engram_result_bytes_per_lane: 16_384,
                max_incremental_total_tokens_per_lane: 50_000,
                max_incremental_runner_duration_ms_per_lane: 30_000,
            })
        );
        assert_eq!(protocol.claude_prior_spend_microusd, 5_936_287);
        assert_eq!(protocol.claude_budget_cents, 50);
        assert_eq!(protocol.claude_authorized_ceiling_cents, 700);
    }

    #[test]
    fn schema_thirteen_requires_a_frozen_operation_evidence_route() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../evals/native_memory_pilot_v1/protocol-schema-13-operation-evidence-route-forward-file-cache-bounded.json",
        );
        let mut protocol: NativePilotProtocol =
            serde_json::from_reader(fs::File::open(path).unwrap()).unwrap();

        validate_protocol(&protocol).unwrap();
        assert_eq!(
            protocol.pilot_id,
            "native-memory-pilot-v1-operation-evidence-route-forward-file-cache-bounded"
        );
        assert_eq!(
            protocol.schema_version,
            OPERATION_EVIDENCE_ROUTE_NATIVE_PILOT_SCHEMA_VERSION
        );
        assert_eq!(protocol.claude_prior_spend_microusd, 6_192_445);
        assert_eq!(protocol.claude_budget_cents, 50);
        assert_eq!(protocol.claude_authorized_ceiling_cents, 700);

        protocol.cases[0].condition_evidence_target = None;
        assert!(validate_protocol(&protocol)
            .unwrap_err()
            .to_string()
            .contains("one frozen operation-evidence target and excerpt"));
    }

    #[test]
    fn schema_fourteen_freezes_source_bound_single_call_route_and_spend() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../evals/native_memory_pilot_v1/protocol-schema-14-source-bound-single-call-forward-file-cache-bounded.json",
        );
        let protocol: NativePilotProtocol =
            serde_json::from_reader(fs::File::open(path).unwrap()).unwrap();

        validate_protocol(&protocol).unwrap();
        assert_eq!(
            protocol.pilot_id,
            "native-memory-pilot-v1-source-bound-single-call-forward-file-cache-bounded"
        );
        assert_eq!(
            protocol.schema_version,
            SOURCE_BOUND_OPERATION_EVIDENCE_NATIVE_PILOT_SCHEMA_VERSION
        );
        assert_eq!(protocol.cases.len(), 1);
        assert_eq!(protocol.repetitions, 1);
        assert_eq!(protocol.run_order.len(), 6);
        assert_eq!(protocol.claude_prior_spend_microusd, 6_488_734);
        assert_eq!(protocol.claude_budget_cents, 50);
        assert_eq!(protocol.claude_authorized_ceiling_cents, 700);
        assert_eq!(
            protocol.codex_authentication_mode,
            CodexAuthenticationMode::ChatgptFileCache
        );
        assert_eq!(
            protocol.resource_budgets,
            Some(NativePilotResourceBudgets {
                max_engram_result_bytes_per_call: 8_192,
                max_engram_result_bytes_per_lane: 16_384,
                max_incremental_total_tokens_per_lane: 50_000,
                max_incremental_runner_duration_ms_per_lane: 30_000,
            })
        );
    }

    #[test]
    fn schema_fifteen_freezes_required_host_action_bounded_query_and_repetitions() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../evals/native_memory_pilot_v1/protocol-schema-15-required-host-action-bounded-query-repeated-file-cache.json",
        );
        let protocol: NativePilotProtocol =
            serde_json::from_reader(fs::File::open(path).unwrap()).unwrap();

        validate_protocol(&protocol).unwrap();
        assert_eq!(
            protocol.pilot_id,
            "native-memory-pilot-v1-required-host-action-bounded-query-repeated-file-cache"
        );
        assert_eq!(
            protocol.schema_version,
            REQUIRED_HOST_ACTION_NATIVE_PILOT_SCHEMA_VERSION
        );
        assert_eq!(protocol.cases.len(), 1);
        assert_eq!(protocol.repetitions, 3);
        assert_eq!(protocol.run_order.len(), 18);
        assert_eq!(protocol.cases[0].procedure_query_max_chars, Some(512));
        assert_eq!(
            protocol.cases[0].procedure_query_required_terms,
            ["worker".to_string(), "procedure".to_string()]
        );
        assert_eq!(protocol.claude_prior_spend_microusd, 6_821_219);
        assert_eq!(protocol.claude_budget_cents, 150);
        assert_eq!(protocol.claude_authorized_ceiling_cents, 850);

        let mut invalid = protocol.clone();
        invalid.cases[0].procedure_query_max_chars = Some(1024);
        assert!(validate_protocol(&invalid)
            .unwrap_err()
            .to_string()
            .contains("512-character procedure-query bound"));

        let mut invalid = protocol;
        invalid.cases[0].procedure_query_required_terms = vec!["unrelated".to_string()];
        assert!(validate_protocol(&invalid)
            .unwrap_err()
            .to_string()
            .contains("task terms present in the evaluation prompt"));
    }

    #[test]
    fn topology_repair_protocol_freezes_identity_matrix_and_spend_accounting() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../evals/native_memory_pilot_v1/protocol-schema-5-topology.json");
        let protocol: NativePilotProtocol =
            serde_json::from_reader(fs::File::open(path).unwrap()).unwrap();

        validate_protocol(&protocol).unwrap();
        assert_eq!(protocol.pilot_id, "native-memory-pilot-v1-topology-repair");
        assert_eq!(protocol.cases.len(), 1);
        assert_eq!(protocol.cases[0].expected_project, None);
        assert_eq!(
            protocol.cases[0].expected_component.as_deref(),
            Some("queue-worker")
        );
        assert_eq!(
            protocol
                .run_order
                .iter()
                .map(|run| run.arm.as_str())
                .collect::<Vec<_>>(),
            [
                "claude_engram_plus_native",
                "codex_native_memory",
                "claude_lean_engram",
                "codex_engram_plus_native",
                "claude_native_memory",
                "codex_lean_engram"
            ]
        );
        assert_eq!(protocol.claude_prior_spend_microusd, 627_092);
        assert_eq!(protocol.claude_budget_cents, 60);
        assert_eq!(protocol.claude_authorized_ceiling_cents, 130);
    }

    #[test]
    fn schema_eight_protocol_freezes_source_backed_mismatch_contract() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../evals/native_memory_pilot_v1/protocol-schema-8-source-backed-prerequisite.json",
        );
        let protocol: NativePilotProtocol =
            serde_json::from_reader(fs::File::open(path).unwrap()).unwrap();

        validate_protocol(&protocol).unwrap();
        assert_eq!(
            protocol.pilot_id,
            "native-memory-pilot-v1-source-backed-prerequisite"
        );
        assert_eq!(protocol.schema_version, 8);
        assert_eq!(protocol.cases.len(), 1);
        let case = &protocol.cases[0];
        assert_eq!(case.expected_outcome, NativePilotExpectedOutcome::Abstain);
        assert_eq!(case.expected_project, None);
        assert_eq!(case.expected_component.as_deref(), Some("queue-worker"));
        assert_eq!(
            case.procedure.prerequisite_sources["tool.version"].relative_path,
            "toolchain.toml"
        );
        assert_eq!(protocol.claude_prior_spend_microusd, 1_496_461);
        assert_eq!(protocol.claude_budget_cents, 50);
        assert_eq!(protocol.claude_authorized_ceiling_cents, 200);
    }

    #[test]
    fn schema_nine_protocol_freezes_unavailable_source_contract() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../evals/native_memory_pilot_v1/protocol-schema-9-source-unavailable-forward.json",
        );
        let protocol: NativePilotProtocol =
            serde_json::from_reader(fs::File::open(path).unwrap()).unwrap();

        validate_protocol(&protocol).unwrap();
        assert_eq!(
            protocol.pilot_id,
            "native-memory-pilot-v1-source-unavailable-forward"
        );
        assert_eq!(protocol.schema_version, 9);
        assert_eq!(protocol.cases.len(), 1);
        let case = &protocol.cases[0];
        assert_eq!(case.expected_outcome, NativePilotExpectedOutcome::Abstain);
        assert_eq!(
            case.evaluation_prerequisite_state,
            NativePilotEvaluationPrerequisiteState::Missing
        );
        assert_eq!(
            case.condition_evidence_target.as_deref(),
            Some("toolchain.toml")
        );
        assert_eq!(case.condition_evidence_output_contains, None);
        assert_eq!(protocol.claude_prior_spend_microusd, 2_752_912);
        assert_eq!(protocol.claude_budget_cents, 50);
        assert_eq!(protocol.claude_authorized_ceiling_cents, 350);
    }

    #[test]
    fn schema_seven_protocol_freezes_wrong_scope_no_result_contract() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../evals/native_memory_pilot_v1/protocol-schema-7-wrong-scope-no-result-forward.json",
        );
        let protocol: NativePilotProtocol =
            serde_json::from_reader(fs::File::open(path).unwrap()).unwrap();

        validate_protocol(&protocol).unwrap();
        assert_eq!(
            protocol.pilot_id,
            "native-memory-pilot-v1-wrong-scope-no-result-forward"
        );
        assert_eq!(protocol.schema_version, 7);
        assert_eq!(protocol.cases.len(), 1);
        let case = &protocol.cases[0];
        assert_eq!(case.expected_outcome, NativePilotExpectedOutcome::Abstain);
        assert_eq!(
            case.expected_repository_remote,
            "https://github.com/acme/orbit"
        );
        assert_eq!(case.expected_project.as_deref(), Some("orbit"));
        assert_eq!(case.expected_component.as_deref(), Some("worker"));
        assert!(case.procedure.prerequisite_sources.is_empty());
        assert_eq!(
            case.condition_evidence_target.as_deref(),
            Some("runbooks/deploy-worker.md")
        );
        assert_eq!(protocol.claude_prior_spend_microusd, 3_252_912);
        assert_eq!(protocol.claude_budget_cents, 50);
        assert_eq!(protocol.claude_authorized_ceiling_cents, 400);
    }

    #[test]
    fn schema_seven_current_protocol_repeats_wrong_scope_across_every_arm() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../evals/native_memory_pilot_v1/protocol-schema-7-unconditional-route-wrong-scope-repeated-current.json",
        );
        let protocol: NativePilotProtocol =
            serde_json::from_reader(fs::File::open(path).unwrap()).unwrap();

        validate_protocol(&protocol).unwrap();
        assert_eq!(
            protocol.pilot_id,
            "native-memory-pilot-v1-unconditional-route-wrong-scope-repeated-current"
        );
        assert_eq!(protocol.schema_version, 7);
        assert_eq!(protocol.cases.len(), 1);
        assert_eq!(protocol.repetitions, 3);
        assert_eq!(protocol.run_order.len(), 18);
        let unique_runs = protocol
            .run_order
            .iter()
            .map(|entry| (&entry.case_id, &entry.arm, entry.repetition))
            .collect::<BTreeSet<_>>();
        assert_eq!(unique_runs.len(), 18);
        for repetition in 1..=3 {
            assert_eq!(
                protocol
                    .run_order
                    .iter()
                    .filter(|entry| entry.repetition == repetition)
                    .count(),
                6
            );
        }
        let case = &protocol.cases[0];
        assert_eq!(case.expected_outcome, NativePilotExpectedOutcome::Abstain);
        assert_eq!(
            case.expected_repository_remote,
            "https://github.com/acme/orbit"
        );
        assert_eq!(case.expected_project.as_deref(), Some("orbit"));
        assert_eq!(case.expected_component.as_deref(), Some("worker"));
        assert_eq!(
            case.condition_evidence_target.as_deref(),
            Some("runbooks/deploy-worker.md")
        );
        assert_eq!(protocol.claude_prior_spend_microusd, 4_528_914);
        assert_eq!(protocol.claude_budget_cents, 100);
        assert_eq!(protocol.claude_authorized_ceiling_cents, 700);
    }

    #[test]
    fn schema_seven_protocol_freezes_no_result_evidence_closure_diagnostic() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../evals/native_memory_pilot_v1/protocol-schema-7-no-result-evidence-closure-forward.json",
        );
        let protocol: NativePilotProtocol =
            serde_json::from_reader(fs::File::open(path).unwrap()).unwrap();

        validate_protocol(&protocol).unwrap();
        assert_eq!(
            protocol.pilot_id,
            "native-memory-pilot-v1-no-result-evidence-closure-forward"
        );
        assert_eq!(protocol.schema_version, 7);
        assert_eq!(protocol.cases.len(), 1);
        assert_eq!(protocol.repetitions, 1);
        assert_eq!(protocol.run_order.len(), 6);
        let case = &protocol.cases[0];
        assert_eq!(case.expected_outcome, NativePilotExpectedOutcome::Abstain);
        assert_eq!(
            case.expected_repository_remote,
            "https://github.com/acme/orbit"
        );
        assert_eq!(case.expected_project.as_deref(), Some("orbit"));
        assert_eq!(case.expected_component.as_deref(), Some("worker"));
        assert_eq!(
            case.condition_evidence_target.as_deref(),
            Some("runbooks/deploy-worker.md")
        );
        assert_eq!(protocol.claude_prior_spend_microusd, 5_245_707);
        assert_eq!(protocol.claude_budget_cents, 50);
        assert_eq!(protocol.claude_authorized_ceiling_cents, 700);
    }

    #[test]
    fn schema_seven_device_auth_protocol_preserves_the_forward_diagnostic() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../evals/native_memory_pilot_v1/protocol-schema-7-no-result-evidence-closure-forward-device-auth.json",
        );
        let protocol: NativePilotProtocol =
            serde_json::from_reader(fs::File::open(path).unwrap()).unwrap();

        validate_protocol(&protocol).unwrap();
        assert_eq!(
            protocol.pilot_id,
            "native-memory-pilot-v1-no-result-evidence-closure-forward-device-auth"
        );
        assert_eq!(protocol.schema_version, 7);
        assert_eq!(protocol.cases.len(), 1);
        assert_eq!(protocol.repetitions, 1);
        assert_eq!(protocol.run_order.len(), 6);
        assert_eq!(
            protocol.codex_authentication_mode,
            CodexAuthenticationMode::ChatgptDeviceKeyring
        );
        assert_eq!(protocol.claude_prior_spend_microusd, 5_245_707);
        assert_eq!(protocol.claude_budget_cents, 50);
        assert_eq!(protocol.claude_authorized_ceiling_cents, 700);
    }

    #[test]
    fn schema_seven_bounded_protocol_freezes_host_visible_resource_limits() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../evals/native_memory_pilot_v1/protocol-schema-7-no-result-evidence-closure-forward-device-auth-bounded.json",
        );
        let protocol: NativePilotProtocol =
            serde_json::from_reader(fs::File::open(path).unwrap()).unwrap();

        validate_protocol(&protocol).unwrap();
        assert_eq!(
            protocol.pilot_id,
            "native-memory-pilot-v1-no-result-evidence-closure-forward-device-auth-bounded"
        );
        assert_eq!(
            protocol.resource_budgets,
            Some(NativePilotResourceBudgets {
                max_engram_result_bytes_per_call: 8_192,
                max_engram_result_bytes_per_lane: 16_384,
                max_incremental_total_tokens_per_lane: 50_000,
                max_incremental_runner_duration_ms_per_lane: 30_000,
            })
        );

        let mut invalid = protocol;
        invalid
            .resource_budgets
            .as_mut()
            .unwrap()
            .max_engram_result_bytes_per_lane = 0;
        assert!(validate_protocol(&invalid)
            .unwrap_err()
            .to_string()
            .contains("resource budgets must be positive"));
    }

    #[test]
    fn bounded_protocol_accepts_explicit_file_cache_authentication() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../evals/native_memory_pilot_v1/protocol-schema-7-no-result-evidence-closure-forward-device-auth-bounded.json",
        );
        let mut protocol: NativePilotProtocol =
            serde_json::from_reader(fs::File::open(path).unwrap()).unwrap();
        protocol.codex_authentication_mode = CodexAuthenticationMode::ChatgptFileCache;

        validate_protocol(&protocol).unwrap();
        assert_eq!(
            serde_json::to_string(&protocol.codex_authentication_mode).unwrap(),
            "\"chatgpt_file_cache\""
        );
    }

    #[test]
    fn codex_authentication_freezes_browser_login_and_keyring_status() {
        let binary = PilotBinaryAttestation {
            path: "/tmp/codex".to_string(),
            version: "codex-cli test".to_string(),
            sha256: "a".repeat(64),
        };
        let authentication =
            prepared_codex_authentication(&binary, CodexAuthenticationMode::ChatgptBrowserKeyring);
        assert_eq!(authentication.credential_store, "keyring");
        assert_eq!(
            authentication.login_argv,
            [
                "/tmp/codex",
                "login",
                "--config",
                "cli_auth_credentials_store=\"keyring\""
            ]
        );
        assert_eq!(
            authentication.status_argv,
            [
                "/tmp/codex",
                "login",
                "--config",
                "cli_auth_credentials_store=\"keyring\"",
                "status"
            ]
        );
        assert_eq!(authentication.expected_status, "Logged in using ChatGPT");
    }

    #[test]
    fn codex_authentication_freezes_device_login_and_keyring_status() {
        let binary = PilotBinaryAttestation {
            path: "/tmp/codex".to_string(),
            version: "codex-cli test".to_string(),
            sha256: "a".repeat(64),
        };
        let authentication =
            prepared_codex_authentication(&binary, CodexAuthenticationMode::ChatgptDeviceKeyring);
        assert_eq!(authentication.credential_store, "keyring");
        assert_eq!(
            authentication.login_argv,
            [
                "/tmp/codex",
                "login",
                "--config",
                "cli_auth_credentials_store=\"keyring\"",
                "--device-auth"
            ]
        );
        assert_eq!(
            authentication.status_argv,
            [
                "/tmp/codex",
                "login",
                "--config",
                "cli_auth_credentials_store=\"keyring\"",
                "status"
            ]
        );
        assert_eq!(authentication.expected_status, "Logged in using ChatGPT");
    }

    #[test]
    fn codex_authentication_freezes_file_cache_status() {
        let binary = PilotBinaryAttestation {
            path: "/tmp/codex".to_string(),
            version: "codex-cli test".to_string(),
            sha256: "a".repeat(64),
        };
        let authentication =
            prepared_codex_authentication(&binary, CodexAuthenticationMode::ChatgptFileCache);
        assert_eq!(authentication.credential_store, "file");
        assert_eq!(
            authentication.login_argv,
            [
                "/tmp/codex",
                "login",
                "--config",
                "cli_auth_credentials_store=\"file\""
            ]
        );
        assert_eq!(
            authentication.status_argv,
            [
                "/tmp/codex",
                "login",
                "--config",
                "cli_auth_credentials_store=\"file\"",
                "status"
            ]
        );
        assert_eq!(authentication.expected_status, "Logged in using ChatGPT");
    }

    #[test]
    fn codex_file_cache_lane_freezes_file_provider_config() {
        let root = tempfile::tempdir().unwrap();
        let binary = PilotBinaryAttestation {
            path: "/tmp/codex".to_string(),
            version: "codex-cli test".to_string(),
            sha256: "a".repeat(64),
        };
        let argv = codex_argv(
            &binary,
            &binary,
            CodexAuthenticationMode::ChatgptFileCache,
            MemoryLayer::Native,
            root.path(),
            "remember the verified procedure",
            None,
            None,
            None,
            None,
            1,
            CodexPhase::Teaching,
        )
        .unwrap();

        assert_eq!(
            argv.windows(2)
                .filter(|pair| pair[0] == "--config" && pair[1] == CODEX_FILE_CONFIG)
                .count(),
            1
        );
        assert!(!argv
            .windows(2)
            .any(|pair| pair[0] == "--config" && pair[1] == CODEX_KEYRING_CONFIG));
    }

    #[test]
    fn codex_persisted_evaluation_omits_only_ephemeral_flag() {
        let root = tempfile::tempdir().unwrap();
        let binary = PilotBinaryAttestation {
            path: "/tmp/codex".to_string(),
            version: "codex-cli test".to_string(),
            sha256: "a".repeat(64),
        };
        let build = |phase| {
            codex_argv(
                &binary,
                &binary,
                CodexAuthenticationMode::ChatgptFileCache,
                MemoryLayer::Native,
                root.path(),
                "evaluate stale memory",
                None,
                None,
                None,
                None,
                1,
                phase,
            )
            .unwrap()
        };
        let ephemeral = build(CodexPhase::Evaluation);
        let persisted = build(CodexPhase::EvaluationPersisted);

        assert_eq!(
            ephemeral
                .iter()
                .filter(|argument| argument.as_str() == "--ephemeral")
                .count(),
            1
        );
        assert!(!persisted.iter().any(|argument| argument == "--ephemeral"));
        assert_eq!(
            ephemeral
                .into_iter()
                .filter(|argument| argument != "--ephemeral")
                .collect::<Vec<_>>(),
            persisted
        );
    }

    #[test]
    fn claude_command_freezes_model_turn_budget_and_explicit_tool_approval() {
        let root = tempfile::tempdir().unwrap();
        let settings = root.path().join("settings.json");
        let mcp = root.path().join("mcp.json");
        let instructions = root.path().join("instructions.md");
        fs::write(&settings, "{}").unwrap();
        fs::write(&mcp, "{}").unwrap();
        fs::write(&instructions, "instructions").unwrap();
        let binary = PilotBinaryAttestation {
            path: "/tmp/claude".to_string(),
            version: "claude test".to_string(),
            sha256: "a".repeat(64),
        };
        let argv = claude_argv(
            &binary,
            root.path(),
            "prompt",
            None,
            &settings,
            &mcp,
            None,
            &instructions,
            "claude-haiku-4-5",
            12,
            50,
            false,
            None,
            &[],
        )
        .unwrap();
        let settings_path = settings.canonicalize().unwrap().display().to_string();
        let mcp_path = mcp.canonicalize().unwrap().display().to_string();
        for expected in [
            ("--permission-mode", "dontAsk"),
            ("--allowed-tools", CLAUDE_ALLOWED_TOOLS),
            ("--settings", settings_path.as_str()),
            ("--mcp-config", mcp_path.as_str()),
            ("--model", "claude-haiku-4-5"),
            ("--max-turns", "12"),
            ("--max-budget-usd", "0.050"),
        ] {
            assert!(argv
                .windows(2)
                .any(|pair| pair[0] == expected.0 && pair[1] == expected.1));
        }
    }

    #[test]
    fn claude_teaching_scopes_bash_to_exact_frozen_commands() {
        let commands = vec![
            "cd /tmp/atlas && ./bin/context-probe --channel amber".to_string(),
            "cd /tmp/atlas && ./bin/context-probe --channel cobalt".to_string(),
        ];
        let teaching = claude_allowed_tools(None, &commands).unwrap();
        let evaluation = claude_allowed_tools(None, &[]).unwrap();

        assert!(teaching.contains("Bash(cd /tmp/atlas && ./bin/context-probe --channel amber)"));
        assert!(teaching.contains("Bash(cd /tmp/atlas && ./bin/context-probe --channel cobalt)"));
        assert!(!teaching.split(',').any(|rule| rule == "Bash"));
        assert!(!evaluation.contains("Bash("));
        assert!(!evaluation.split(',').any(|rule| rule == "Bash"));
    }

    #[test]
    fn claude_native_memory_only_allows_edits_in_isolated_memory_root() {
        let root = tempfile::tempdir().unwrap();
        let settings = root.path().join("settings.json");
        let mcp = root.path().join("mcp.json");
        let instructions = root.path().join("instructions.md");
        let memory = root.path().join("memory");
        fs::create_dir(&memory).unwrap();
        fs::write(&settings, "{}").unwrap();
        fs::write(&mcp, "{}").unwrap();
        fs::write(&instructions, "instructions").unwrap();
        let binary = PilotBinaryAttestation {
            path: "/tmp/claude".to_string(),
            version: "claude test".to_string(),
            sha256: "a".repeat(64),
        };
        let argv = claude_argv(
            &binary,
            root.path(),
            "prompt",
            None,
            &settings,
            &mcp,
            None,
            &instructions,
            "claude-haiku-4-5",
            12,
            50,
            false,
            Some(&memory),
            &[],
        )
        .unwrap();
        let memory_rule = format!(
            "{CLAUDE_ALLOWED_TOOLS},Edit(//{}/**)",
            memory
                .canonicalize()
                .unwrap()
                .display()
                .to_string()
                .trim_start_matches('/')
        );
        for expected in [
            ["--allowed-tools", memory_rule.as_str()],
            ["--tools", "Read,Bash,Write,Edit"],
            ["--disallowed-tools", "WebFetch,WebSearch,NotebookEdit,Task"],
        ] {
            assert!(argv
                .windows(2)
                .any(|pair| pair[0] == expected[0] && pair[1] == expected[1]));
        }
    }

    #[test]
    fn evaluation_prompt_cannot_reveal_learned_values() {
        let mut leaked = case();
        leaked.evaluation_prompt = format!("Run {}", leaked.procedure.command);
        assert!(validate_case(&leaked, NATIVE_PILOT_SCHEMA_VERSION)
            .unwrap_err()
            .to_string()
            .contains("leaks learned ground truth"));
    }

    #[test]
    fn native_output_schema_exposes_only_frozen_first_action_vocabulary() {
        let mut protocol = protocol();
        let mut abstention = protocol.cases[0].clone();
        abstention.id = "prerequisite-mismatch".to_string();
        abstention.expected_first_action = "inspect_procedure_prerequisites".to_string();
        protocol.cases.push(abstention);

        let schema: serde_json::Value =
            serde_json::from_str(&native_agent_output_schema(&protocol).unwrap()).unwrap();
        assert_eq!(
            schema.pointer("/properties/first_action/enum"),
            Some(&serde_json::json!([
                "inspect_procedure_prerequisites",
                "run_verified_procedure",
                null
            ]))
        );
        assert!(schema.get("$schema").is_none());
        assert!(schema["properties"]["project"]["description"]
            .as_str()
            .unwrap()
            .contains("null when unresolved"));
    }

    #[test]
    fn schema_twelve_output_requires_structured_identity_fields() {
        let mut structured_protocol = protocol();
        structured_protocol.schema_version = STRUCTURED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION;
        structured_protocol.cases[0].expected_project = None;
        structured_protocol.cases[0].expected_project_status =
            Some(NativePilotExpectedProjectStatus::RequiresConfirmation);
        structured_protocol.cases[0].expected_project_confirmation_required = Some(true);
        structured_protocol.cases[0].expected_first_action =
            "resolve_checkout_identity".to_string();
        structured_protocol.cases[0].expected_outcome = NativePilotExpectedOutcome::Abstain;
        structured_protocol.cases[0].condition_evidence_target =
            Some("runbooks/deploy-worker.md".to_string());
        structured_protocol.cases[0].condition_evidence_output_contains =
            Some("ORBIT_ONLY_CANARY".to_string());
        structured_protocol.cases[0].project_evidence_target = None;
        structured_protocol.cases[0].project_evidence_output_contains = None;
        structured_protocol.cases[0].procedure.conditions.clear();
        structured_protocol.cases[0]
            .procedure
            .prerequisite_sources
            .clear();
        structured_protocol.resource_budgets = Some(NativePilotResourceBudgets {
            max_engram_result_bytes_per_call: 8_192,
            max_engram_result_bytes_per_lane: 16_384,
            max_incremental_total_tokens_per_lane: 50_000,
            max_incremental_runner_duration_ms_per_lane: 30_000,
        });
        validate_protocol(&structured_protocol).unwrap();

        let schema: serde_json::Value =
            serde_json::from_str(&native_agent_output_schema(&structured_protocol).unwrap())
                .unwrap();
        let required = schema["required"].as_array().unwrap();
        for key in [
            "checkout_root",
            "project_status",
            "project_confirmation_required",
        ] {
            assert!(required.contains(&serde_json::Value::String(key.to_string())));
        }
        assert_eq!(
            schema.pointer("/properties/project_status/enum"),
            Some(&serde_json::json!([
                "authorized",
                "requires_confirmation",
                "unavailable"
            ]))
        );

        let mut drifted_budget = structured_protocol.clone();
        drifted_budget
            .resource_budgets
            .as_mut()
            .unwrap()
            .max_engram_result_bytes_per_call += 1;
        assert!(validate_protocol(&drifted_budget)
            .unwrap_err()
            .to_string()
            .contains("frozen structured-identity resource budgets"));

        let historical = native_agent_output_schema(&protocol()).unwrap();
        assert!(!historical.contains("project_confirmation_required"));
    }

    #[test]
    fn schema_seven_identity_evidence_is_tracked_and_hashed_at_preparation() {
        let root = tempfile::tempdir().unwrap();
        let component = root.path().join("services/worker/component.json");
        fs::create_dir_all(component.parent().unwrap()).unwrap();
        fs::write(&component, r#"{"name":"queue-worker","kind":"service"}"#).unwrap();
        assert!(Command::new("git")
            .args(["init", "--quiet"])
            .arg(root.path())
            .status()
            .unwrap()
            .success());
        assert!(Command::new("git")
            .arg("-C")
            .arg(root.path())
            .args(["add", "services/worker/component.json"])
            .status()
            .unwrap()
            .success());

        let frozen = prepare_identity_evidence(
            &case(),
            TRUSTED_SCOPED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION,
            "component",
            Some("services/worker/component.json"),
            Some("queue-worker"),
            root.path(),
        )
        .unwrap();
        assert_eq!(frozen, Some(sha256_file(&component).unwrap()));

        fs::write(root.path().join("untracked-project.txt"), "Atlas").unwrap();
        assert!(prepare_identity_evidence(
            &case(),
            TRUSTED_SCOPED_IDENTITY_NATIVE_PILOT_SCHEMA_VERSION,
            "project",
            Some("untracked-project.txt"),
            Some("Atlas"),
            root.path(),
        )
        .unwrap_err()
        .to_string()
        .contains("not Git-tracked"));
    }

    #[test]
    fn schema_eight_prerequisite_source_freezes_current_checkout_status() {
        let root = tempfile::tempdir().unwrap();
        let teaching = root.path().join("teaching");
        let evaluation = root.path().join("evaluation");
        for (checkout, version) in [(&teaching, "3"), (&evaluation, "2")] {
            fs::create_dir_all(checkout).unwrap();
            fs::write(
                checkout.join("toolchain.toml"),
                format!("[tools]\nversion = \"{version}\"\n"),
            )
            .unwrap();
            assert!(Command::new("git")
                .args(["init", "--quiet"])
                .arg(checkout)
                .status()
                .unwrap()
                .success());
            assert!(Command::new("git")
                .arg("-C")
                .arg(checkout)
                .args(["add", "toolchain.toml"])
                .status()
                .unwrap()
                .success());
        }
        let mut mismatch = case();
        mismatch.expected_outcome = NativePilotExpectedOutcome::Abstain;
        let observation = prepare_prerequisite_source_observation(
            &mismatch,
            SOURCE_BACKED_PREREQUISITE_NATIVE_PILOT_SCHEMA_VERSION,
            &teaching,
            &evaluation,
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            observation.status,
            NativePilotConditionObservationStatus::Mismatched
        );
        assert_eq!(
            observation.source_sha256,
            Some(sha256_file(&evaluation.join("toolchain.toml")).unwrap())
        );
        assert_eq!(
            observation.source_output_contains,
            Some("[tools]\nversion = \"2\"".to_string())
        );

        let mut missing = mismatch.clone();
        missing.evaluation_prerequisite_state = NativePilotEvaluationPrerequisiteState::Missing;
        fs::remove_file(evaluation.join("toolchain.toml")).unwrap();
        let unavailable = prepare_prerequisite_source_observation(
            &missing,
            UNAVAILABLE_PREREQUISITE_NATIVE_PILOT_SCHEMA_VERSION,
            &teaching,
            &evaluation,
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            unavailable.status,
            NativePilotConditionObservationStatus::Unavailable
        );
        assert_eq!(unavailable.source_sha256, None);
        assert_eq!(unavailable.source_output_contains, None);
        assert_eq!(
            unavailable.detail_contains.as_deref(),
            Some("configured source file is unavailable")
        );

        fs::write(
            evaluation.join("toolchain.toml"),
            "[tools]\nversion = \"3\"\n",
        )
        .unwrap();
        let mut expired = case();
        expired.expected_outcome = NativePilotExpectedOutcome::Abstain;
        expired.procedure_expiry_seconds = Some(300);
        let matched = prepare_prerequisite_source_observation(
            &expired,
            EXPIRED_PROCEDURE_NATIVE_PILOT_SCHEMA_VERSION,
            &teaching,
            &evaluation,
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            matched.status,
            NativePilotConditionObservationStatus::Matched
        );

        fs::write(
            teaching.join("toolchain.toml"),
            "[tools]\nversion = \"2\"\n",
        )
        .unwrap();
        assert!(prepare_prerequisite_source_observation(
            &mismatch,
            SOURCE_BACKED_PREREQUISITE_NATIVE_PILOT_SCHEMA_VERSION,
            &teaching,
            &evaluation,
        )
        .unwrap_err()
        .to_string()
        .contains("teaching prerequisite source does not match"));
    }

    #[test]
    fn teaching_prompt_forbids_hand_written_native_memory() {
        let dir = tempfile::tempdir().unwrap();
        let receipt = dir.path().join("receipt.json");
        fs::write(&receipt, "{}").unwrap();
        let prompt = teaching_prompt(
            &case(),
            MemoryLayer::Native,
            &receipt,
            dir.path(),
            "https://github.com/acme/atlas",
            "codex",
            "codex-cli",
            &[],
        )
        .unwrap();
        assert!(prompt.contains("do not use shell commands or repository files"));
        assert!(prompt.contains("--channel amber"));
        assert!(prompt.contains("--channel cobalt"));
    }

    #[test]
    fn engram_teaching_prompt_freezes_complete_write_request() {
        let dir = tempfile::tempdir().unwrap();
        let receipt = dir.path().join("receipt.json");
        fs::write(&receipt, "{}").unwrap();
        let prompt = teaching_prompt(
            &case(),
            MemoryLayer::Both,
            &receipt,
            dir.path(),
            "https://github.com/acme/atlas",
            "claude_code",
            "claude-haiku-4-5",
            &[],
        )
        .unwrap();
        for expected in [
            "submit exactly one call",
            "\"scope_type\":\"repository\"",
            "\"remote_url\":\"https://github.com/acme/atlas\"",
            "\"writer_harness\":\"claude_code\"",
            "\"model_provider\":\"anthropic\"",
            "\"model\":\"claude-haiku-4-5\"",
            "\"prerequisite_sources\":{\"tool.version\":{\"format\":\"toml\",\"relative_path\":\"toolchain.toml\",\"key_path\":[\"tools\",\"version\"]}}",
            "\"verification_output_contains\":\"ATLAS_CONTEXT_PROBE_OK\"",
        ] {
            assert!(
                prompt.contains(expected),
                "missing `{expected}` in {prompt}"
            );
        }
        assert!(prompt.contains(&receipt.canonicalize().unwrap().display().to_string()));
        assert!(prompt.contains(&dir.path().canonicalize().unwrap().display().to_string()));
    }

    #[test]
    fn engram_teaching_prompt_scopes_memory_to_teaching_remote() {
        let dir = tempfile::tempdir().unwrap();
        let receipt = dir.path().join("receipt.json");
        fs::write(&receipt, "{}").unwrap();
        let mut wrong_scope = case();
        wrong_scope.expected_repository_remote = "https://github.com/acme/orbit".to_string();

        let prompt = teaching_prompt(
            &wrong_scope,
            MemoryLayer::Engram,
            &receipt,
            dir.path(),
            "git@github.com:acme/atlas.git",
            "codex",
            "codex-cli",
            &[],
        )
        .unwrap();

        assert!(prompt.contains("\"remote_url\":\"git@github.com:acme/atlas.git\""));
        assert!(!prompt.contains("\"remote_url\":\"https://github.com/acme/orbit\""));
    }

    #[test]
    fn teaching_repository_remote_comes_from_the_teaching_checkout() {
        let root = tempfile::tempdir().unwrap();
        let layout = materialize_engineering_context_fixture(&root.path().join("fixture")).unwrap();
        let teaching =
            resolve_checkout_root(&layout, "fixture://atlas/main/services/worker").unwrap();
        assert_eq!(
            repository_remote(&teaching).unwrap(),
            "git@github.com:acme/atlas.git"
        );
    }

    #[test]
    fn codex_engram_lane_injects_generated_adapter_as_developer_instructions() {
        let root = tempfile::tempdir().unwrap();
        let adapter = root
            .path()
            .join("adapter/.codex/skills/engram-memory-session");
        fs::create_dir_all(&adapter).unwrap();
        fs::write(
            adapter.join("SKILL.md"),
            "---\nname: engram-memory-session\ndescription: learned procedure\n---\nCall orient, then procedure_match before shell exploration.\n",
        )
        .unwrap();
        let state = root.path().join("engram-home");
        let cwd = root.path().join("checkout");
        fs::create_dir_all(&state).unwrap();
        fs::create_dir_all(&cwd).unwrap();
        let binary = PilotBinaryAttestation {
            path: "/tmp/binary".to_string(),
            version: "test".to_string(),
            sha256: "a".repeat(64),
        };

        let argv = codex_argv(
            &binary,
            &binary,
            CodexAuthenticationMode::ChatgptBrowserKeyring,
            MemoryLayer::Engram,
            &cwd,
            "run learned procedure",
            None,
            Some("test-project"),
            Some(&state),
            Some(&root.path().join("adapter")),
            1,
            CodexPhase::Evaluation,
        )
        .unwrap();
        let instructions = argv
            .windows(2)
            .find(|pair| pair[0] == "--config" && pair[1].starts_with("developer_instructions="))
            .map(|pair| pair[1].as_str())
            .expect("developer instructions config");
        assert!(instructions.contains("Call orient, then procedure_match before shell exploration"));
    }

    #[test]
    fn family_v3_claude_engram_and_both_lanes_inline_private_config() {
        for (memory_layer, arm_id) in [
            (MemoryLayer::Engram, "claude_lean_engram"),
            (MemoryLayer::Both, "claude_engram_plus_native"),
        ] {
            let root = tempfile::tempdir().unwrap();
            let output = root.path().join("prepared");
            fs::create_dir(&output).unwrap();
            let schema_path = output.join("agent-output.schema.json");
            fs::write(&schema_path, AGENT_OUTPUT_SCHEMA).unwrap();
            let arm = NativePilotArm {
                id: arm_id.to_string(),
                host: "claude_code".to_string(),
                memory_layer,
            };
            let run_ref = NativePilotRunRef {
                case_id: "procedure".to_string(),
                arm: arm.id.clone(),
                repetition: 1,
            };
            let binary = PilotBinaryAttestation {
                path: "/tmp/binary".to_string(),
                version: "test".to_string(),
                sha256: "a".repeat(64),
            };
            let preparation_override = NativePilotLanePreparationOverride {
                closed_world_provider_environment: true,
                ..NativePilotLanePreparationOverride::default()
            };

            let lane = prepare_lane(
                &output,
                1,
                &run_ref,
                &arm,
                &case(),
                &binary,
                &binary,
                &binary,
                &schema_path,
                EXPIRED_PROCEDURE_NATIVE_PILOT_SCHEMA_VERSION,
                1,
                CodexAuthenticationMode::ChatgptFileCache,
                "claude-haiku-4-5",
                12,
                41,
                Some(&preparation_override),
            )
            .unwrap();
            crate::native_stale::validate_native_stale_v3_claude_config_artifacts(&lane).unwrap();

            assert_eq!(
                lane.environment
                    .keys()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
                vec![
                    "CLAUDE_CODE_DISABLE_AUTO_MEMORY",
                    "CLAUDE_CONFIG_DIR",
                    "DISABLE_TELEMETRY",
                    "ENGRAM_HOME",
                    "HOME",
                    "LANG",
                    "LC_ALL",
                    "PATH",
                    "SHELL",
                    "TMPDIR",
                ]
            );
            let (_, mcp_path) = assert_family_v3_inline_claude_config(&lane);
            assert_eq!(mcp_path.file_name().unwrap(), "claude-mcp.json");
            assert!(!mcp_path
                .parent()
                .unwrap()
                .join("claude-mcp-empty.json")
                .exists());
        }
    }

    #[test]
    fn family_v3_claude_native_lane_creates_private_empty_mcp_and_settings() {
        let root = tempfile::tempdir().unwrap();
        let output = root.path().join("prepared");
        fs::create_dir(&output).unwrap();
        let schema_path = output.join("agent-output.schema.json");
        fs::write(&schema_path, AGENT_OUTPUT_SCHEMA).unwrap();
        let arm = NativePilotArm {
            id: "claude_native_memory".to_string(),
            host: "claude_code".to_string(),
            memory_layer: MemoryLayer::Native,
        };
        let run_ref = NativePilotRunRef {
            case_id: "procedure".to_string(),
            arm: arm.id.clone(),
            repetition: 1,
        };
        let binary = PilotBinaryAttestation {
            path: "/tmp/binary".to_string(),
            version: "test".to_string(),
            sha256: "a".repeat(64),
        };
        let preparation_override = NativePilotLanePreparationOverride {
            closed_world_provider_environment: true,
            ..NativePilotLanePreparationOverride::default()
        };

        let lane = prepare_lane(
            &output,
            1,
            &run_ref,
            &arm,
            &case(),
            &binary,
            &binary,
            &binary,
            &schema_path,
            EXPIRED_PROCEDURE_NATIVE_PILOT_SCHEMA_VERSION,
            1,
            CodexAuthenticationMode::ChatgptFileCache,
            "claude-haiku-4-5",
            12,
            41,
            Some(&preparation_override),
        )
        .unwrap();
        crate::native_stale::validate_native_stale_v3_claude_config_artifacts(&lane).unwrap();

        let (settings_path, mcp_path) = assert_family_v3_inline_claude_config(&lane);
        assert_eq!(settings_path.file_name().unwrap(), "claude-settings.json");
        assert_eq!(mcp_path.file_name().unwrap(), "claude-mcp-empty.json");
        assert_eq!(fs::read(&mcp_path).unwrap(), b"{\"mcpServers\":{}}");
        assert!(!mcp_path.parent().unwrap().join("claude-mcp.json").exists());
    }

    #[cfg(unix)]
    #[test]
    fn family_v3_private_file_creation_is_exclusive_and_rejects_symlinks() {
        use std::os::unix::fs::symlink;

        let root = tempfile::tempdir().unwrap();
        let lane = root.path().join("lane");
        create_private_dir_all(&lane).unwrap();
        let artifact = lane.join("claude-settings.json");
        write_family_v3_private_file_exclusive(&artifact, b"first\n").unwrap();
        assert_private_owner_only_single_link(&artifact);
        assert!(write_family_v3_private_file_exclusive(&artifact, b"second\n").is_err());
        assert_eq!(fs::read(&artifact).unwrap(), b"first\n");

        let target = lane.join("target.json");
        fs::write(&target, b"target\n").unwrap();
        let link = lane.join("claude-mcp.json");
        symlink(&target, &link).unwrap();
        assert!(write_family_v3_private_file_exclusive(&link, b"replacement\n").is_err());
        assert_eq!(fs::read(&target).unwrap(), b"target\n");
    }
}
