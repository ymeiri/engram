use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use engram_eval::{
    fixture::materialize_engineering_context_fixture,
    lean_probe::run_lean_orient_probe,
    load_runs, load_suite,
    native_audit::audit_native_memory_pilot,
    native_correction::{
        attest_native_correction_runtime, attest_native_correction_seed_state,
        audit_native_correction_lifecycle, audit_prepared_native_correction,
        check_native_correction_auth_ready, create_native_correction_cleanup_intent,
        create_native_correction_operator_intent, execute_native_correction_cleanup,
        execute_native_correction_operator, execute_native_correction_provider_phase,
        prepare_native_correction, provision_native_correction_codex_auth,
        validate_native_correction_protocol_file, validate_native_correction_runtime,
        NativeCorrectionProviderPhase, NativeCorrectionRuntimePaths,
    },
    native_instructions_control::{
        audit_native_stale_instructions_control, prepare_native_stale_instructions_control,
        prepare_native_stale_provider_bundle, validate_native_stale_instructions_control_protocol,
    },
    native_pilot::prepare_native_memory_pilot,
    native_report::report_native_memory_pilot,
    native_runner::{
        attest_native_memory_pilot_plan, check_native_memory_pilot_authentication,
        cleanup_native_memory_pilot_auth_cache, prepare_native_memory_pilot_evaluation_recovery,
        prepare_native_memory_pilot_execution_recovery, provision_native_memory_pilot_auth_cache,
        run_native_memory_pilot_with_bundle, run_native_stale_instructions_control,
        NativePilotAuthCacheCleanupConfirmation, NativePilotAuthCacheCleanupState,
        NativePilotExecutionApproval, NativePilotRunPhase,
    },
    native_stale::{
        audit_native_stale_safety, audit_native_stale_safety_preparation,
        prepare_native_stale_safety, report_native_stale_safety,
        validate_native_stale_safety_protocol,
    },
    pilot::prepare_pilot,
    procedure_probe::run_procedure_match_probe,
    retrieval_probe::run_retrieval_probe,
    retrieval_probe_v2::run_retrieval_probe_v2,
    retrieval_probe_v3::run_selective_retrieval_probe,
    retrieval_probe_v4::run_structured_candidate_probe,
    retrieval_probe_v5::run_adversarial_candidate_probe,
    scenario_path, score_runs,
    seed::seed_engineering_context_fixture,
};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "engram-eval")]
#[command(about = "Validate and score frozen Engram engineering-context evaluations")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum NativePilotPhaseArg {
    Teaching,
    Activation,
    Evaluation,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum NativeCorrectionProviderPhaseArg {
    Proposal,
    Retrieval,
}

impl From<NativeCorrectionProviderPhaseArg> for NativeCorrectionProviderPhase {
    fn from(value: NativeCorrectionProviderPhaseArg) -> Self {
        match value {
            NativeCorrectionProviderPhaseArg::Proposal => Self::Proposal,
            NativeCorrectionProviderPhaseArg::Retrieval => Self::Retrieval,
        }
    }
}

#[derive(Debug, Clone, Args)]
struct NativeCorrectionRuntimeArgs {
    /// Exact frozen Engram executable
    #[arg(long)]
    engram_bin: PathBuf,
    /// Exact frozen Codex executable
    #[arg(long)]
    codex_bin: PathBuf,
    /// Exact frozen Codex code-mode host executable
    #[arg(long)]
    codex_code_mode_host_bin: PathBuf,
    /// Exact frozen Claude Code executable
    #[arg(long)]
    claude_bin: PathBuf,
}

impl NativeCorrectionRuntimeArgs {
    fn paths(self) -> NativeCorrectionRuntimePaths {
        NativeCorrectionRuntimePaths {
            engram: self.engram_bin,
            codex: self.codex_bin,
            codex_code_mode_host: self.codex_code_mode_host_bin,
            claude_code: self.claude_bin,
        }
    }
}

impl From<NativePilotPhaseArg> for NativePilotRunPhase {
    fn from(value: NativePilotPhaseArg) -> Self {
        match value {
            NativePilotPhaseArg::Teaching => Self::Teaching,
            NativePilotPhaseArg::Activation => Self::Activation,
            NativePilotPhaseArg::Evaluation => Self::Evaluation,
        }
    }
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Materialize the deterministic engineering-context fixture and semantic seed plan
    Materialize {
        /// New or empty output directory
        #[arg(long)]
        output: PathBuf,
    },
    /// Seed a materialized fixture into a new isolated Engram data store
    Seed {
        /// Fixture map emitted by `materialize`
        #[arg(long)]
        fixture_map: PathBuf,
        /// New or empty RocksDB directory
        #[arg(long)]
        data_dir: PathBuf,
    },
    /// Measure the real lean orientation packet on an isolated deterministic fixture
    ProbeLeanOrient {
        /// Preregistered provider-free probe protocol
        #[arg(long)]
        protocol: PathBuf,
        /// New or empty output directory for fixture, store, and report evidence
        #[arg(long)]
        output: PathBuf,
    },
    /// Compare production lexical memory retrieval with frozen provider-free baselines
    ProbeRetrieval {
        /// Preregistered corpus-by-retrieval protocol
        #[arg(long)]
        protocol: PathBuf,
        /// New or empty output directory for fixture, store, and report evidence
        #[arg(long)]
        output: PathBuf,
    },
    /// Compare attested local dense, hybrid, lifecycle, and abstention retrieval arms
    ProbeRetrievalV2 {
        /// Preregistered v2 retrieval protocol
        #[arg(long)]
        protocol: PathBuf,
        /// Exact pre-cached model snapshot directory named by its revision
        #[arg(long)]
        model_snapshot: PathBuf,
        /// New or empty output directory for fixture, store, and report evidence
        #[arg(long)]
        output: PathBuf,
    },
    /// Evaluate held-out dense candidate discovery and conservative automatic application
    ProbeRetrievalV3 {
        /// Preregistered v3 selective-application protocol
        #[arg(long)]
        protocol: PathBuf,
        /// Exact pre-cached model snapshot directory named by its revision
        #[arg(long)]
        model_snapshot: PathBuf,
        /// New or empty output directory for fixture, store, and report evidence
        #[arg(long)]
        output: PathBuf,
    },
    /// Evaluate bounded structured candidates on the known v3 failure corpus
    ProbeRetrievalV4 {
        /// Preregistered v4 structured-candidate protocol
        #[arg(long)]
        protocol: PathBuf,
        /// Exact pre-cached model snapshot directory named by its revision
        #[arg(long)]
        model_snapshot: PathBuf,
        /// New or empty private output directory for fixture, store, and report evidence
        #[arg(long)]
        output: PathBuf,
    },
    /// Adversarially test the applicability boundary of the frozen v4 candidate strategy
    ProbeRetrievalV5 {
        /// Preregistered v5 adversarial holdout protocol
        #[arg(long)]
        protocol: PathBuf,
        /// Exact pre-cached model snapshot directory named by its revision
        #[arg(long)]
        model_snapshot: PathBuf,
        /// New or empty private output directory for fixture, store, and report evidence
        #[arg(long)]
        output: PathBuf,
    },
    /// Evaluate the real verified-procedure matcher on an isolated frozen scenario matrix
    ProbeProcedureMatch {
        /// Preregistered procedure-match protocol
        #[arg(long)]
        protocol: PathBuf,
        /// New or empty private output directory for fixture, store, and report evidence
        #[arg(long)]
        output: PathBuf,
    },
    /// Prepare and attest an execution-gated native-host pilot without calling providers
    PreparePilot {
        /// Preregistered pilot protocol
        #[arg(long)]
        protocol: PathBuf,
        /// New or empty output directory
        #[arg(long)]
        output: PathBuf,
        /// Exact Engram binary to expose to native hosts
        #[arg(long)]
        engram_bin: PathBuf,
        /// Codex executable path or PATH name
        #[arg(long, default_value = "codex")]
        codex_bin: PathBuf,
        /// Claude Code executable path or PATH name
        #[arg(long, default_value = "claude")]
        claude_bin: PathBuf,
    },
    /// Prepare a learned native-memory versus Engram pilot without calling providers
    PrepareNativeMemoryPilot {
        /// Preregistered native-memory protocol
        #[arg(long)]
        protocol: PathBuf,
        /// New or empty output directory
        #[arg(long)]
        output: PathBuf,
        /// Exact Engram binary to expose to native hosts
        #[arg(long)]
        engram_bin: PathBuf,
        /// Codex executable path or PATH name
        #[arg(long, default_value = "codex")]
        codex_bin: PathBuf,
        /// Claude Code executable path or PATH name
        #[arg(long, default_value = "claude")]
        claude_bin: PathBuf,
    },
    /// Prepare an evaluation-only successor for an exact local host-boundary rejection
    PrepareNativeMemoryPilotEvaluationRecovery {
        /// Immutable rejected source run-plan.json
        #[arg(long)]
        source_plan: PathBuf,
        /// New or empty private output directory
        #[arg(long)]
        output: PathBuf,
        /// Relocated Codex executable; bytes and version must exactly match the source attestation
        #[arg(long)]
        codex_bin: Option<PathBuf>,
    },
    /// Prepare a successor after a local teaching-phase audit interruption
    PrepareNativeMemoryPilotExecutionRecovery {
        /// Immutable interrupted source run-plan.json
        #[arg(long)]
        source_plan: PathBuf,
        /// New or empty private output directory
        #[arg(long)]
        output: PathBuf,
    },
    /// Re-attest a frozen native-memory plan without calling providers
    AttestNativeMemoryPilot {
        /// Prepared run-plan.json
        #[arg(long)]
        plan: PathBuf,
    },
    /// Check frozen ChatGPT login for every isolated Codex lane without calling a provider
    CheckNativeMemoryPilotAuth {
        /// Prepared run-plan.json
        #[arg(long)]
        plan: PathBuf,
        /// Exit nonzero unless every isolated Codex lane is authenticated through ChatGPT
        #[arg(long)]
        require_ready: bool,
    },
    /// Copy a protected ChatGPT cache into every pristine file-cache Codex lane
    ProvisionNativeMemoryPilotAuthCache {
        /// Prepared run-plan.json frozen for chatgpt_file_cache
        #[arg(long)]
        plan: PathBuf,
        /// Existing owner-only auth.json; its contents are never printed or hashed
        #[arg(long)]
        source_cache: PathBuf,
        /// Confirm creation of protected plaintext credential copies in isolated lane homes
        #[arg(long)]
        confirm_plaintext_cache_copies: bool,
    },
    /// Delete exactly six manifest-bound file-cache copies after terminal evaluation completion
    CleanupNativeMemoryPilotAuthCache {
        /// Prepared stale-safety family-v3 run-plan.json
        #[arg(long)]
        plan: PathBuf,
        /// Canonical run root named by the user's destructive-action confirmation
        #[arg(long)]
        confirm_run_root: PathBuf,
        /// Exact lifecycle manifest SHA-256 displayed by provisioning
        #[arg(long)]
        confirm_lifecycle_manifest_sha256: String,
        /// Confirm deletion of exactly the six manifest-bound plaintext credential copies
        #[arg(long)]
        confirm_delete_six_plaintext_cache_copies: bool,
    },
    /// Audit native-memory pilot lifecycle, artifacts, and acceptance without calling providers
    AuditNativeMemoryPilot {
        /// Prepared run-plan.json
        #[arg(long)]
        plan: PathBuf,
        /// Exit nonzero unless every lane is ready or complete
        #[arg(long, conflicts_with = "require_complete")]
        require_ready: bool,
        /// Exit nonzero unless every lane produced a valid evaluation outcome
        #[arg(long)]
        require_complete: bool,
        /// Exit nonzero unless every valid evaluation outcome passed task acceptance
        #[arg(long, conflicts_with = "require_ready")]
        require_all_passed: bool,
    },
    /// Compare valid native-memory pilot outcomes without calling providers
    ReportNativeMemoryPilot {
        /// Prepared run-plan.json
        #[arg(long)]
        plan: PathBuf,
        /// Exit nonzero unless one treatment Pareto-dominates native memory across every host
        #[arg(long)]
        require_portable_incremental_value: bool,
    },
    /// Validate the dedicated native stale/missing-source safety protocol without preparation
    ValidateNativeStaleSafetyProtocol {
        /// Dedicated protocol containing the ordinary native matrix and stale-safety extension
        #[arg(long)]
        protocol: PathBuf,
    },
    /// Prepare an isolated stale/missing-source safety pilot without credentials or providers
    PrepareNativeStaleSafety {
        /// Dedicated stale-safety protocol
        #[arg(long)]
        protocol: PathBuf,
        /// New or empty output directory
        #[arg(long)]
        output: PathBuf,
        /// Exact Engram binary to expose to native hosts
        #[arg(long)]
        engram_bin: PathBuf,
        /// Codex executable path or PATH name
        #[arg(long, default_value = "codex")]
        codex_bin: PathBuf,
        /// Claude Code executable path or PATH name
        #[arg(long, default_value = "claude")]
        claude_bin: PathBuf,
    },
    /// Recheck a pristine stale-safety preparation without credentials or providers
    AuditNativeStaleSafetyPreparation {
        /// Dedicated stale-safety protocol
        #[arg(long)]
        protocol: PathBuf,
        /// Freshly prepared run-plan.json
        #[arg(long)]
        plan: PathBuf,
    },
    /// Audit stale/missing-source safety without calling providers
    AuditNativeStaleSafety {
        /// Dedicated stale-safety protocol
        #[arg(long)]
        protocol: PathBuf,
        /// Existing prepared native-memory run-plan.json
        #[arg(long)]
        plan: PathBuf,
        /// Exit nonzero unless all twelve evaluation outcomes are complete
        #[arg(long)]
        require_complete: bool,
        /// Exit nonzero unless setup is valid and every completed lane passes safety acceptance
        #[arg(long)]
        require_all_safe: bool,
    },
    /// Report descriptive native stale/missing-source safety without a Pareto claim
    ReportNativeStaleSafety {
        /// Dedicated stale-safety protocol
        #[arg(long)]
        protocol: PathBuf,
        /// Existing prepared native-memory run-plan.json
        #[arg(long)]
        plan: PathBuf,
        /// Exit nonzero unless setup is valid and every lane passes safety acceptance
        #[arg(long)]
        require_all_safe: bool,
    },
    /// Validate the provider-free instructions-only control protocol without preparing a run
    ValidateNativeStaleInstructionsControl {
        /// Instructions-only control protocol (draft validation is allowed)
        #[arg(long)]
        protocol: PathBuf,
        /// Exact family-v3 treatment protocol bound by the control protocol
        #[arg(long)]
        treatment_protocol: PathBuf,
    },
    /// Prepare four instructions-only controls without credentials or provider calls
    PrepareNativeStaleInstructionsControl {
        /// Frozen executable instructions-only control protocol
        #[arg(long)]
        protocol: PathBuf,
        /// Exact family-v3 treatment protocol
        #[arg(long)]
        treatment_protocol: PathBuf,
        /// Fresh family-v3 treatment run-plan.json
        #[arg(long)]
        treatment_plan: PathBuf,
        /// New absent private output directory
        #[arg(long)]
        output: PathBuf,
    },
    /// Freeze the 34-admission parent intent and its disjoint treatment/control child intents
    PrepareNativeStaleProviderBundle {
        /// Fresh family-v3 treatment run-plan.json
        #[arg(long)]
        treatment_plan: PathBuf,
        /// Prepared instructions-only control run-plan.json
        #[arg(long)]
        control_plan: PathBuf,
        /// New absent private output directory
        #[arg(long)]
        output: PathBuf,
    },
    /// Run the four evaluation-only instructions controls under the frozen parent bundle
    RunNativeStaleInstructionsControl {
        /// Prepared instructions-only control run-plan.json
        #[arg(long)]
        plan: PathBuf,
        /// Canonical native-stale-provider-bundle-v1 directory
        #[arg(long)]
        provider_bundle: PathBuf,
        /// Confirm the already-authorized frozen provider calls
        #[arg(long)]
        approve_provider_execution: bool,
        /// Exact separate instructions-control Claude ceiling (10 cents)
        #[arg(long)]
        confirm_claude_budget_cents: Option<u32>,
    },
    /// Audit the four controls, exact 34 admissions, trace absences, and pair identity
    AuditNativeStaleInstructionsControl {
        /// Frozen executable instructions-only control protocol
        #[arg(long)]
        protocol: PathBuf,
        /// Exact family-v3 treatment protocol
        #[arg(long)]
        treatment_protocol: PathBuf,
        /// Prepared instructions-only control run-plan.json
        #[arg(long)]
        plan: PathBuf,
        /// Canonical native-stale-provider-bundle-v1 directory
        #[arg(long)]
        provider_bundle: PathBuf,
        /// Exit nonzero unless all four controls and every integrity gate are complete
        #[arg(long)]
        require_complete: bool,
    },
    /// Execute one native-memory pilot phase after explicit approval and exact budget confirmation
    RunNativeMemoryPilot {
        /// Prepared run-plan.json
        #[arg(long)]
        plan: PathBuf,
        /// Separately authorized lifecycle phase
        #[arg(long)]
        phase: NativePilotPhaseArg,
        /// Confirm that the user explicitly approved provider execution for this invocation
        #[arg(long)]
        approve_provider_execution: bool,
        /// Exact aggregate Claude runner allocation from the plan, in cents
        #[arg(long)]
        confirm_claude_budget_cents: Option<u32>,
        /// Required pre-existing 34-admission bundle for family-v3; forbidden for historical plans
        #[arg(long)]
        provider_bundle: Option<PathBuf>,
    },
    /// Validate the native-correction protocol without preparing or calling providers
    ValidateNativeCorrectionProtocol {
        /// Frozen native-correction protocol
        #[arg(long)]
        protocol: PathBuf,
    },
    /// Re-attest the exact frozen native-correction runtime without preparing or calling providers
    AttestNativeCorrectionRuntime {
        #[command(flatten)]
        runtime: NativeCorrectionRuntimeArgs,
    },
    /// Re-attest and validate the exact frozen native-correction runtime
    ValidateNativeCorrectionRuntime {
        #[command(flatten)]
        runtime: NativeCorrectionRuntimeArgs,
    },
    /// Prepare the isolated two-host native-correction evaluation without provider calls
    PrepareNativeCorrection {
        /// Frozen native-correction protocol
        #[arg(long)]
        protocol: PathBuf,
        /// New absent private output directory
        #[arg(long)]
        output: PathBuf,
        #[command(flatten)]
        runtime: NativeCorrectionRuntimeArgs,
    },
    /// Audit a prepared native-correction plan without provider calls
    AuditPreparedNativeCorrection {
        /// Prepared run-plan.json
        #[arg(long)]
        plan: PathBuf,
        /// Exit nonzero unless the preparation is valid
        #[arg(long)]
        require_valid: bool,
    },
    /// Re-attest both native-correction lane seed states without provider calls
    AttestNativeCorrectionSeedState {
        /// Prepared run-plan.json
        #[arg(long)]
        plan: PathBuf,
    },
    /// Copy a protected Codex ChatGPT cache into the two pristine correction homes
    ProvisionNativeCorrectionCodexAuth {
        /// Prepared run-plan.json
        #[arg(long)]
        plan: PathBuf,
        /// Existing owner-only Codex auth.json
        #[arg(long)]
        source_cache: PathBuf,
    },
    /// Check native-correction Codex and Claude authentication readiness without provider calls
    CheckNativeCorrectionAuth {
        /// Prepared run-plan.json
        #[arg(long)]
        plan: PathBuf,
        /// Exit nonzero unless the exact auth contract is ready
        #[arg(long)]
        require_ready: bool,
    },
    /// Execute the exact next native-correction proposal or retrieval provider admission
    RunNativeCorrectionProvider {
        /// Prepared run-plan.json
        #[arg(long)]
        plan: PathBuf,
        /// Exact lane order selected by the lifecycle audit
        #[arg(long)]
        lane_order: u32,
        /// Exact provider phase selected by the lifecycle audit
        #[arg(long)]
        phase: NativeCorrectionProviderPhaseArg,
        /// Confirm explicit provider-execution authority
        #[arg(long)]
        approve_provider_execution: bool,
        /// Confirm the exact aggregate Claude allocation (20 cents)
        #[arg(long)]
        confirm_claude_budget_cents: u32,
    },
    /// Create and validate the exact inactive native-correction operator intent
    CreateNativeCorrectionOperatorIntent {
        /// Prepared run-plan.json
        #[arg(long)]
        plan: PathBuf,
        /// Exact lane order selected by the lifecycle audit
        #[arg(long)]
        lane_order: u32,
    },
    /// Execute the exact next provider-free native-correction operator transition
    RunNativeCorrectionOperator {
        /// Prepared run-plan.json
        #[arg(long)]
        plan: PathBuf,
        /// Exact lane order selected by the lifecycle audit
        #[arg(long)]
        lane_order: u32,
    },
    /// Create and validate the exact native-correction cleanup intent
    CreateNativeCorrectionCleanupIntent {
        /// Prepared run-plan.json
        #[arg(long)]
        plan: PathBuf,
        /// Exact lane order selected by the lifecycle audit
        #[arg(long)]
        lane_order: u32,
    },
    /// Execute the exact next irreversible isolated native-correction cleanup
    RunNativeCorrectionCleanup {
        /// Prepared run-plan.json
        #[arg(long)]
        plan: PathBuf,
        /// Exact lane order selected by the lifecycle audit
        #[arg(long)]
        lane_order: u32,
    },
    /// Audit the complete native-correction provider/operator/cleanup lifecycle
    AuditNativeCorrectionLifecycle {
        /// Prepared run-plan.json
        #[arg(long)]
        plan: PathBuf,
        /// Exit nonzero unless both lanes are complete
        #[arg(long)]
        require_complete: bool,
    },
    /// Validate a manifest and its scenario corpus
    Validate {
        /// Suite manifest path
        #[arg(long)]
        manifest: PathBuf,
    },
    /// Score JSONL run records against a frozen suite
    Score {
        /// Suite manifest path
        #[arg(long)]
        manifest: PathBuf,
        /// Host-neutral JSONL run records
        #[arg(long)]
        results: PathBuf,
        /// Optional JSON report path; stdout is always populated
        #[arg(long)]
        output: Option<PathBuf>,
        /// Exit nonzero unless at least one claim group shows value on every host
        #[arg(long)]
        require_portable_incremental_value: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Materialize { output } => {
            let layout = materialize_engineering_context_fixture(&output)?;
            println!("{}", serde_json::to_string_pretty(&layout)?);
        }
        Command::Seed {
            fixture_map,
            data_dir,
        } => {
            let report = seed_engineering_context_fixture(&fixture_map, &data_dir).await?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Command::ProbeLeanOrient { protocol, output } => {
            let report = run_lean_orient_probe(&protocol, &output).await?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if !report.passed {
                anyhow::bail!("lean orientation probe did not meet its preregistered gates");
            }
        }
        Command::ProbeRetrieval { protocol, output } => {
            let report = run_retrieval_probe(&protocol, &output).await?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if !report.passed {
                anyhow::bail!("retrieval probe did not meet its preregistered gates");
            }
        }
        Command::ProbeRetrievalV2 {
            protocol,
            model_snapshot,
            output,
        } => {
            let report = run_retrieval_probe_v2(&protocol, &model_snapshot, &output).await?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if !report.passed {
                anyhow::bail!("retrieval v2 probe did not meet its preregistered gates");
            }
        }
        Command::ProbeRetrievalV3 {
            protocol,
            model_snapshot,
            output,
        } => {
            let report = run_selective_retrieval_probe(&protocol, &model_snapshot, &output).await?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if !report.passed {
                anyhow::bail!("retrieval v3 probe did not meet its preregistered gates");
            }
        }
        Command::ProbeRetrievalV4 {
            protocol,
            model_snapshot,
            output,
        } => {
            let report =
                run_structured_candidate_probe(&protocol, &model_snapshot, &output).await?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if !report.passed {
                anyhow::bail!("retrieval v4 probe did not meet its preregistered gates");
            }
        }
        Command::ProbeRetrievalV5 {
            protocol,
            model_snapshot,
            output,
        } => {
            let report =
                run_adversarial_candidate_probe(&protocol, &model_snapshot, &output).await?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if !report.passed {
                anyhow::bail!("retrieval v5 probe did not meet its preregistered gates");
            }
        }
        Command::ProbeProcedureMatch { protocol, output } => {
            let report = run_procedure_match_probe(&protocol, &output).await?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if !report.passed {
                anyhow::bail!("procedure-match probe did not meet its preregistered gates");
            }
        }
        Command::PreparePilot {
            protocol,
            output,
            engram_bin,
            codex_bin,
            claude_bin,
        } => {
            let report =
                prepare_pilot(&protocol, &output, &engram_bin, &codex_bin, &claude_bin).await?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Command::PrepareNativeMemoryPilot {
            protocol,
            output,
            engram_bin,
            codex_bin,
            claude_bin,
        } => {
            let report = prepare_native_memory_pilot(
                &protocol,
                &output,
                &engram_bin,
                &codex_bin,
                &claude_bin,
            )
            .await?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Command::PrepareNativeMemoryPilotEvaluationRecovery {
            source_plan,
            output,
            codex_bin,
        } => {
            let report = prepare_native_memory_pilot_evaluation_recovery(
                &source_plan,
                &output,
                codex_bin.as_deref(),
            )?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Command::PrepareNativeMemoryPilotExecutionRecovery {
            source_plan,
            output,
        } => {
            let report = prepare_native_memory_pilot_execution_recovery(&source_plan, &output)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Command::AttestNativeMemoryPilot { plan } => {
            let report = attest_native_memory_pilot_plan(&plan)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Command::CheckNativeMemoryPilotAuth {
            plan,
            require_ready,
        } => {
            let report = check_native_memory_pilot_authentication(&plan)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if require_ready && !report.ready {
                anyhow::bail!("not every isolated Codex lane has its frozen ChatGPT login");
            }
        }
        Command::ProvisionNativeMemoryPilotAuthCache {
            plan,
            source_cache,
            confirm_plaintext_cache_copies,
        } => {
            let report = provision_native_memory_pilot_auth_cache(
                &plan,
                &source_cache,
                confirm_plaintext_cache_copies,
            )?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if !report.ready {
                anyhow::bail!("not every provisioned Codex lane has ChatGPT file-cache login");
            }
        }
        Command::CleanupNativeMemoryPilotAuthCache {
            plan,
            confirm_run_root,
            confirm_lifecycle_manifest_sha256,
            confirm_delete_six_plaintext_cache_copies,
        } => {
            let report = cleanup_native_memory_pilot_auth_cache(
                &plan,
                NativePilotAuthCacheCleanupConfirmation {
                    run_root: confirm_run_root,
                    lifecycle_manifest_sha256: confirm_lifecycle_manifest_sha256,
                    delete_six_plaintext_cache_copies: confirm_delete_six_plaintext_cache_copies,
                },
            )?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if report.state != NativePilotAuthCacheCleanupState::Complete {
                anyhow::bail!("auth-cache cleanup is durably unresolved");
            }
        }
        Command::AuditNativeMemoryPilot {
            plan,
            require_ready,
            require_complete,
            require_all_passed,
        } => {
            let report = audit_native_memory_pilot(&plan)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if require_ready && !report.ready_for_evaluation {
                anyhow::bail!("native-memory pilot is not ready for evaluation");
            }
            if require_complete && !report.complete {
                anyhow::bail!("native-memory pilot is not complete");
            }
            if require_all_passed && !report.all_acceptance_passed {
                anyhow::bail!("not every native-memory pilot outcome passed acceptance");
            }
        }
        Command::ReportNativeMemoryPilot {
            plan,
            require_portable_incremental_value,
        } => {
            let report = report_native_memory_pilot(&plan)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if require_portable_incremental_value && !report.portable_incremental_value_observed {
                anyhow::bail!("portable incremental value over native memory was not observed");
            }
        }
        Command::ValidateNativeStaleSafetyProtocol { protocol } => {
            let report = validate_native_stale_safety_protocol(&protocol)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Command::PrepareNativeStaleSafety {
            protocol,
            output,
            engram_bin,
            codex_bin,
            claude_bin,
        } => {
            let report = prepare_native_stale_safety(
                &protocol,
                &output,
                &engram_bin,
                &codex_bin,
                &claude_bin,
            )
            .await?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Command::AuditNativeStaleSafetyPreparation { protocol, plan } => {
            let report = audit_native_stale_safety_preparation(&protocol, &plan)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if !report.valid {
                anyhow::bail!("native stale-safety preparation preflight failed");
            }
        }
        Command::AuditNativeStaleSafety {
            protocol,
            plan,
            require_complete,
            require_all_safe,
        } => {
            let report = audit_native_stale_safety(&protocol, &plan)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if require_complete && !report.complete {
                anyhow::bail!("native stale-safety pilot is not complete");
            }
            if require_all_safe
                && (!report.setup_integrity_passed || !report.all_safety_acceptance_passed)
            {
                anyhow::bail!("native stale-safety setup or safety acceptance failed");
            }
        }
        Command::ReportNativeStaleSafety {
            protocol,
            plan,
            require_all_safe,
        } => {
            let report = report_native_stale_safety(&protocol, &plan)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if require_all_safe
                && (!report.setup_integrity_passed || !report.all_safety_acceptance_passed)
            {
                anyhow::bail!("native stale-safety setup or safety acceptance failed");
            }
        }
        Command::ValidateNativeStaleInstructionsControl {
            protocol,
            treatment_protocol,
        } => {
            let audit = validate_native_stale_instructions_control_protocol(
                &protocol,
                &treatment_protocol,
            )?;
            println!("{}", serde_json::to_string_pretty(&audit)?);
            if !audit.valid {
                anyhow::bail!("native stale instructions-control protocol is invalid");
            }
        }
        Command::PrepareNativeStaleInstructionsControl {
            protocol,
            treatment_protocol,
            treatment_plan,
            output,
        } => {
            let prepared = prepare_native_stale_instructions_control(
                &protocol,
                &treatment_protocol,
                &treatment_plan,
                &output,
            )?;
            println!("{}", serde_json::to_string_pretty(&prepared)?);
        }
        Command::PrepareNativeStaleProviderBundle {
            treatment_plan,
            control_plan,
            output,
        } => {
            let prepared =
                prepare_native_stale_provider_bundle(&treatment_plan, &control_plan, &output)?;
            println!("{}", serde_json::to_string_pretty(&prepared)?);
        }
        Command::RunNativeStaleInstructionsControl {
            plan,
            provider_bundle,
            approve_provider_execution,
            confirm_claude_budget_cents,
        } => {
            let report = run_native_stale_instructions_control(
                &plan,
                &provider_bundle,
                NativePilotExecutionApproval {
                    provider_execution: approve_provider_execution,
                    claude_budget_cents: confirm_claude_budget_cents,
                },
            )?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Command::AuditNativeStaleInstructionsControl {
            protocol,
            treatment_protocol,
            plan,
            provider_bundle,
            require_complete,
        } => {
            let audit = audit_native_stale_instructions_control(
                &protocol,
                &treatment_protocol,
                &plan,
                &provider_bundle,
            )?;
            println!("{}", serde_json::to_string_pretty(&audit)?);
            if require_complete && (!audit.complete || audit.invalid) {
                anyhow::bail!("native stale instructions controls are incomplete or invalid");
            }
        }
        Command::RunNativeMemoryPilot {
            plan,
            phase,
            approve_provider_execution,
            confirm_claude_budget_cents,
            provider_bundle,
        } => {
            let report = run_native_memory_pilot_with_bundle(
                &plan,
                phase.into(),
                NativePilotExecutionApproval {
                    provider_execution: approve_provider_execution,
                    claude_budget_cents: confirm_claude_budget_cents,
                },
                provider_bundle.as_deref(),
            )?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Command::ValidateNativeCorrectionProtocol { protocol } => {
            let report = validate_native_correction_protocol_file(&protocol)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Command::AttestNativeCorrectionRuntime { runtime } => {
            let report = attest_native_correction_runtime(&runtime.paths())?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Command::ValidateNativeCorrectionRuntime { runtime } => {
            let report = attest_native_correction_runtime(&runtime.paths())?;
            validate_native_correction_runtime(&report)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Command::PrepareNativeCorrection {
            protocol,
            output,
            runtime,
        } => {
            let runtime = attest_native_correction_runtime(&runtime.paths())?;
            let report = prepare_native_correction(&protocol, &output, runtime)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Command::AuditPreparedNativeCorrection {
            plan,
            require_valid,
        } => {
            let report = audit_prepared_native_correction(&plan)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if require_valid && !report.valid {
                anyhow::bail!("native-correction preparation is invalid");
            }
        }
        Command::AttestNativeCorrectionSeedState { plan } => {
            let report = attest_native_correction_seed_state(&plan)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if !report.provider_free || !report.full_profile || !report.stores_match_seed_receipts {
                anyhow::bail!("native-correction seed-state attestation failed");
            }
        }
        Command::ProvisionNativeCorrectionCodexAuth { plan, source_cache } => {
            let report = provision_native_correction_codex_auth(&plan, &source_cache)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if report.destination_count != 2 || !report.owner_only || !report.no_overwrite {
                anyhow::bail!("native-correction Codex auth provisioning failed");
            }
        }
        Command::CheckNativeCorrectionAuth {
            plan,
            require_ready,
        } => {
            let report = check_native_correction_auth_ready(&plan)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if require_ready
                && (report.codex_cache_count != 2
                    || report.claude_profile_count != 2
                    || !report.codex_chatgpt_login
                    || !report.claude_subscription_login
                    || report.api_key_mode)
            {
                anyhow::bail!("native-correction authentication is not ready");
            }
        }
        Command::RunNativeCorrectionProvider {
            plan,
            lane_order,
            phase,
            approve_provider_execution,
            confirm_claude_budget_cents,
        } => {
            let report = execute_native_correction_provider_phase(
                &plan,
                lane_order,
                phase.into(),
                approve_provider_execution,
                confirm_claude_budget_cents,
            )?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Command::CreateNativeCorrectionOperatorIntent { plan, lane_order } => {
            let report = create_native_correction_operator_intent(&plan, lane_order)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Command::RunNativeCorrectionOperator { plan, lane_order } => {
            let report = execute_native_correction_operator(&plan, lane_order)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Command::CreateNativeCorrectionCleanupIntent { plan, lane_order } => {
            let report = create_native_correction_cleanup_intent(&plan, lane_order)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Command::RunNativeCorrectionCleanup { plan, lane_order } => {
            let report = execute_native_correction_cleanup(&plan, lane_order)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Command::AuditNativeCorrectionLifecycle {
            plan,
            require_complete,
        } => {
            let report = audit_native_correction_lifecycle(&plan)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if !report.valid || (require_complete && !report.complete) {
                anyhow::bail!("native-correction lifecycle is invalid or incomplete");
            }
        }
        Command::Validate { manifest } => {
            let (suite, scenarios) = load_suite(&manifest)?;
            println!(
                "validated {}: {} scenarios, {} arms, {} repetitions; scenarios={}",
                suite.suite_id,
                scenarios.len(),
                suite.arms.len(),
                suite.repetitions,
                scenario_path(&manifest, &suite).display()
            );
        }
        Command::Score {
            manifest,
            results,
            output,
            require_portable_incremental_value,
        } => {
            let (suite, scenarios) = load_suite(&manifest)?;
            let runs = load_runs(&results)?;
            let report = score_runs(&suite, &scenarios, &runs)?;
            let json = serde_json::to_string_pretty(&report)?;
            if let Some(output) = output {
                std::fs::write(&output, format!("{json}\n"))
                    .with_context(|| format!("failed to write {}", output.display()))?;
            }
            println!("{json}");
            if require_portable_incremental_value
                && report.portable_incremental_value_groups.is_empty()
            {
                anyhow::bail!("portable incremental value was not observed");
            }
        }
    }
    Ok(())
}
