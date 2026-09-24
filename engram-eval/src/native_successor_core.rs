//! Private, non-runnable authority substrate for strict native successor phases.
//!
//! The only process transition is reachable from an in-memory, post-intent state. No public
//! module re-exports these types, and no production launch adapter is present in this slice.

use crate::native_document::{
    require_no_extended_acl, LoadedSuccessorPayload, NativeSuccessorDocumentKind,
    NativeSuccessorFamily, NativeSuccessorPayload, SuccessorDocumentAuthority,
};
use crate::native_execution::{
    NativeExecutionChildGuard, NativeExecutionChildStdin, NativeExecutionCleanupOutcome,
    NativeExecutionSpawnOutcome,
};
use crate::{EvalError, EvalResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
#[cfg(test)]
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{CString, OsString};
use std::fs::{self, File, Metadata, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const INTENT_FILE: &str = "execution-intent.json";
const INTENT_DIGEST_FILE: &str = "execution-intent.json.sha256";
const RECEIPT_FILE: &str = "terminal-receipt.json";
const RECEIPT_DIGEST_FILE: &str = "terminal-receipt.json.sha256";
const MAX_PHASE_INPUT_BYTES: usize = 1024 * 1024;
const MAX_PHASE_OUTPUT_BYTES: u64 = 8 * 1024 * 1024;
const MAX_PHASE_ARTIFACTS: usize = 4;
const CLEANUP_RESERVE_MAX: Duration = Duration::from_secs(2);

#[cfg(test)]
std::thread_local! {
    static DURABLE_WRITE_DELAY: Cell<Option<Duration>> = const { Cell::new(None) };
    static TERMINAL_FINALIZATION_DELAY: Cell<Option<Duration>> = const { Cell::new(None) };
    static TERMINAL_ARTIFACT_FAULT: Cell<Option<TerminalArtifactFault>> = const { Cell::new(None) };
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TerminalArtifactFault {
    ReplaceReceipt,
    TamperDigest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum NativeSuccessorPhase {
    StaleTeachingNative,
    StaleTeachingEngram,
    StaleActivation,
    StaleEvaluation,
    CorrectionProposal,
    CorrectionOperator,
    CorrectionRetrieval,
}

impl NativeSuccessorPhase {
    fn family(self) -> NativeSuccessorFamily {
        match self {
            Self::StaleTeachingNative
            | Self::StaleTeachingEngram
            | Self::StaleActivation
            | Self::StaleEvaluation => NativeSuccessorFamily::StaleIsolatedV1,
            Self::CorrectionProposal | Self::CorrectionOperator | Self::CorrectionRetrieval => {
                NativeSuccessorFamily::CorrectionV1
            }
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::StaleTeachingNative => "stale_teaching_native",
            Self::StaleTeachingEngram => "stale_teaching_engram",
            Self::StaleActivation => "stale_activation",
            Self::StaleEvaluation => "stale_evaluation",
            Self::CorrectionProposal => "correction_proposal",
            Self::CorrectionOperator => "correction_operator",
            Self::CorrectionRetrieval => "correction_retrieval",
        }
    }
}

/// A future family adapter must expose the exact execution binding frozen inside its run plan.
/// The core never accepts an independently selected artifact directory, execution ID, or phase.
trait NativeSuccessorRunPlanPayload: NativeSuccessorPayload {
    fn execution_id(&self) -> &str;
    fn phase(&self) -> NativeSuccessorPhase;
    fn artifact_directory(&self) -> &Path;
}

/// Process-local invocation binding. It intentionally has no serialization or cloning traits.
pub(crate) struct NativeInvocationConfirmation {
    execution_id: String,
    family: NativeSuccessorFamily,
    phase: NativeSuccessorPhase,
    plan_sha256: String,
}

/// Process-local authority minted only after the durable intent pair exists.
struct NativeExecutionSeal {
    directory: PrivateArtifactDirectory,
    plan: SuccessorDocumentAuthority,
    intent: SealedArtifact,
    intent_digest: SealedArtifact,
    evaluator_pid: u32,
    terminal_deadline: Instant,
}

/// Pre-intent state. It has no method capable of spawning a child.
#[allow(dead_code)]
pub(crate) struct PreparedExecution {
    directory: PrivateArtifactDirectory,
    plan: SuccessorDocumentAuthority,
    execution_id: String,
    phase: NativeSuccessorPhase,
    launch: NativeDerivedLaunch,
    response_deadline: Instant,
    terminal_deadline: Instant,
}

#[allow(dead_code)]
pub(crate) struct IntentCommittedExecution {
    seal: NativeExecutionSeal,
    execution_id: String,
    phase: NativeSuccessorPhase,
    launch: NativeDerivedLaunch,
    response_deadline: Instant,
}

#[allow(dead_code)]
pub(crate) struct RunningExecution {
    seal: Option<NativeExecutionSeal>,
    execution_id: String,
    phase: NativeSuccessorPhase,
    guard: Option<NativeExecutionChildGuard>,
    stdin: Option<NativeExecutionChildStdin>,
    stdout: Option<BoundedCollector>,
    stderr: Option<BoundedCollector>,
    response_deadline: Instant,
    started_unix_ms: u64,
    expected_json_object: bool,
    output_limit: u64,
    input: Vec<u8>,
    terminalization: TerminalizationState,
}

/// Internal terminal object. It deliberately has no Serde or Debug implementation.
#[allow(dead_code)]
pub(crate) struct TerminalExecutionReceipt {
    outcome: NativeTerminalOutcome,
    failure_code: Option<String>,
    receipt_sha256: String,
    captured_stdout: Vec<u8>,
    captured_stderr: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum NativeTerminalOutcome {
    Success,
    Failure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TerminalizationState {
    Active,
    Attempting,
    Finished,
}

/// A launch is derived in process by a future family adapter. There is intentionally no public or
/// serialized constructor in this provider-free slice.
struct NativeDerivedLaunch {
    binary: PathBuf,
    arguments: Vec<OsString>,
    cwd: PathBuf,
    environment: BTreeMap<OsString, OsString>,
    input: Vec<u8>,
    output_limit: u64,
    expected_json_object: bool,
}

struct PrivateArtifactDirectory {
    file: File,
    path: PathBuf,
    metadata: Metadata,
}

struct SealedArtifact {
    file: File,
    name: &'static str,
    metadata: Metadata,
    sha256: String,
}

struct BoundedCollector {
    receiver: Receiver<(std::io::Result<usize>, Vec<u8>)>,
    thread: Option<JoinHandle<()>>,
    label: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct SuccessorEnvelope<'a, T> {
    family: &'a str,
    schema_version: u32,
    payload: T,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ExecutionIntentPayload {
    document_kind: String,
    execution_id: String,
    plan_sha256: String,
    phase: String,
    command_contract_sha256: String,
    evaluator_pid: u32,
    created_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct TerminalReceiptPayload {
    document_kind: String,
    execution_id: String,
    plan_sha256: String,
    intent_sha256: String,
    phase: String,
    outcome: NativeTerminalOutcome,
    evaluator_pid: u32,
    process_id: Option<u32>,
    started_unix_ms: u64,
    completed_unix_ms: u64,
    exit_code: Option<i32>,
    stdout_sha256: String,
    stderr_sha256: String,
    stdout_bytes: u64,
    stderr_bytes: u64,
    failure_code: Option<String>,
}

struct TerminalReceiptObservation<'a> {
    execution_id: &'a str,
    phase: NativeSuccessorPhase,
    evaluator_pid: u32,
    process_id: Option<u32>,
    started_unix_ms: u64,
    completed_unix_ms: u64,
    exit_code: Option<i32>,
    stdout: &'a [u8],
    stderr: &'a [u8],
    outcome: NativeTerminalOutcome,
    failure_code: Option<&'a str>,
}

#[allow(dead_code)]
impl PreparedExecution {
    fn prepare<T: NativeSuccessorRunPlanPayload>(
        loaded: LoadedSuccessorPayload<T>,
        confirmation: NativeInvocationConfirmation,
        launch: NativeDerivedLaunch,
        minimum_disk_reserve_bytes: u64,
        timeout: Duration,
    ) -> EvalResult<Self> {
        let loaded_plan_sha256 = loaded.sha256().to_string();
        let (payload, plan) = loaded.into_parts();
        let execution_id = payload.execution_id().to_string();
        let phase = payload.phase();
        let artifact_directory = payload.artifact_directory().to_path_buf();
        if plan.document_kind != NativeSuccessorDocumentKind::RunPlan
            || plan.family != phase.family()
            || confirmation.execution_id != execution_id
            || confirmation.family != plan.family
            || confirmation.phase != phase
            || confirmation.plan_sha256 != plan.sha256
            || loaded_plan_sha256 != plan.sha256
        {
            return invalid(
                "successor invocation confirmation did not bind the exact plan and phase",
            );
        }
        validate_execution_id(&execution_id)?;
        plan.revalidate()?;
        let directory = PrivateArtifactDirectory::open(&artifact_directory)?;
        if plan.canonical_path().starts_with(directory.path()) {
            return invalid(
                "successor plan must remain outside its empty phase artifact directory",
            );
        }
        directory.require_exact_entries(&BTreeSet::new())?;
        require_disk_reserve(directory.path(), minimum_disk_reserve_bytes)?;
        if launch.input.len() > MAX_PHASE_INPUT_BYTES
            || launch.output_limit == 0
            || launch.output_limit > MAX_PHASE_OUTPUT_BYTES
        {
            return invalid(
                "successor derived launch exceeded its bounded input or output contract",
            );
        }
        let started = Instant::now();
        let terminal_deadline = started
            .checked_add(timeout)
            .ok_or_else(|| EvalError::Invalid("successor deadline overflowed".to_string()))?;
        let cleanup_reserve = (timeout / 4).min(CLEANUP_RESERVE_MAX);
        let response_deadline =
            terminal_deadline
                .checked_sub(cleanup_reserve)
                .ok_or_else(|| {
                    EvalError::Invalid("successor cleanup reserve underflowed".to_string())
                })?;
        if Instant::now() >= response_deadline {
            return invalid("successor timeout left no response interval before cleanup reserve");
        }
        Ok(Self {
            directory,
            plan,
            execution_id,
            phase,
            launch,
            response_deadline,
            terminal_deadline,
        })
    }

    fn commit_intent(self) -> EvalResult<IntentCommittedExecution> {
        self.plan.revalidate()?;
        self.directory.require_exact_entries(&BTreeSet::new())?;
        let evaluator_pid = std::process::id();
        let payload = ExecutionIntentPayload {
            document_kind: NativeSuccessorDocumentKind::ExecutionIntent
                .as_str()
                .to_string(),
            execution_id: self.execution_id.clone(),
            plan_sha256: self.plan.sha256.clone(),
            phase: self.phase.as_str().to_string(),
            command_contract_sha256: self.launch.contract_sha256(),
            evaluator_pid,
            created_unix_ms: unix_ms()?,
        };
        let (intent, intent_digest) = self.directory.write_enveloped_pair(
            INTENT_FILE,
            INTENT_DIGEST_FILE,
            self.plan.family,
            &payload,
            self.terminal_deadline,
        )?;
        let seal = NativeExecutionSeal {
            directory: self.directory,
            plan: self.plan,
            intent,
            intent_digest,
            evaluator_pid,
            terminal_deadline: self.terminal_deadline,
        };
        seal.revalidate_intent_state()?;
        Ok(IntentCommittedExecution {
            seal,
            execution_id: self.execution_id,
            phase: self.phase,
            launch: self.launch,
            response_deadline: self.response_deadline,
        })
    }
}

#[allow(dead_code)]
impl IntentCommittedExecution {
    /// This private consuming method is the only spawn edge in the successor state machine.
    fn spawn(self) -> EvalResult<RunningExecution> {
        self.seal.revalidate_intent_state()?;
        if Instant::now() >= self.response_deadline {
            return self.terminal_failure_without_child("pre_spawn_deadline");
        }
        let started_unix_ms = unix_ms()?;
        let mut command = Command::new(&self.launch.binary);
        command
            .args(&self.launch.arguments)
            .current_dir(&self.launch.cwd)
            .env_clear()
            .envs(&self.launch.environment)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut guard = match NativeExecutionChildGuard::spawn(command, self.seal.terminal_deadline)
        {
            NativeExecutionSpawnOutcome::Spawned(guard) => guard,
            NativeExecutionSpawnOutcome::NotSpawned(error) => {
                drop(error);
                return self.terminal_failure_without_child("spawn_failed_before_child");
            }
            NativeExecutionSpawnOutcome::PostSpawnTerminal {
                process_id,
                status,
                error,
            } => {
                drop(error);
                return self.terminal_failure_after_spawn(
                    "spawn_verification_failed_terminal",
                    process_id,
                    status.code(),
                );
            }
            NativeExecutionSpawnOutcome::PostSpawnCleanupUnproven {
                process_id,
                error,
                quarantine,
            } => {
                let _ = process_id;
                drop(error);
                drop(quarantine);
                return invalid(
                    "successor spawned process cleanup remained unproven; durable intent is frozen without a terminal receipt",
                );
            }
        };
        let process_id = match guard.process_id() {
            Ok(process_id) => process_id,
            Err(_) => {
                drop(guard);
                return invalid(
                    "successor process ownership changed before stdio acquisition; durable intent is frozen without a terminal receipt",
                );
            }
        };
        let stdin = match guard.take_stdin() {
            Ok(stdin) => stdin,
            Err(_) => {
                drop(guard);
                return invalid(
                    "successor process ownership changed during stdin acquisition; durable intent is frozen without a terminal receipt",
                );
            }
        };
        let stdout = match guard.take_stdout() {
            Ok(stdout) => stdout,
            Err(_) => {
                drop(stdin);
                drop(guard);
                return invalid(
                    "successor process ownership changed during stdout acquisition; durable intent is frozen without a terminal receipt",
                );
            }
        };
        let stderr = match guard.take_stderr() {
            Ok(stderr) => stderr,
            Err(_) => {
                drop(stdin);
                drop(stdout);
                drop(guard);
                return invalid(
                    "successor process ownership changed during stderr acquisition; durable intent is frozen without a terminal receipt",
                );
            }
        };
        let (Some(stdin), Some(stdout), Some(stderr)) = (stdin, stdout, stderr) else {
            return match guard
                .terminate_group_and_reap_with_status_until(self.seal.terminal_deadline)
            {
                NativeExecutionCleanupOutcome::ProvenTerminal(status) => self
                    .terminal_failure_after_spawn("stdio_unavailable", process_id, status.code()),
                NativeExecutionCleanupOutcome::CleanupUnproven(_) => {
                    drop(guard);
                    invalid(
                        "successor stdio setup cleanup remained unproven; durable intent is frozen without a terminal receipt",
                    )
                }
            };
        };
        if configure_stdin_nonblocking(&stdin).is_err() {
            drop(stdin);
            drop(stdout);
            drop(stderr);
            return match guard
                .terminate_group_and_reap_with_status_until(self.seal.terminal_deadline)
            {
                NativeExecutionCleanupOutcome::ProvenTerminal(status) => self
                    .terminal_failure_after_spawn(
                        "stdin_configuration_failed",
                        process_id,
                        status.code(),
                    ),
                NativeExecutionCleanupOutcome::CleanupUnproven(_) => {
                    drop(guard);
                    invalid(
                        "successor stdin setup cleanup remained unproven; durable intent is frozen without a terminal receipt",
                    )
                }
            };
        }
        let output_limit = self.launch.output_limit;
        Ok(RunningExecution {
            seal: Some(self.seal),
            execution_id: self.execution_id,
            phase: self.phase,
            guard: Some(guard),
            stdin: Some(stdin),
            stdout: Some(BoundedCollector::start(stdout, output_limit, "stdout")),
            stderr: Some(BoundedCollector::start(stderr, output_limit, "stderr")),
            response_deadline: self.response_deadline,
            started_unix_ms,
            expected_json_object: self.launch.expected_json_object,
            output_limit: self.launch.output_limit,
            input: self.launch.input,
            terminalization: TerminalizationState::Active,
        })
    }

    fn terminal_failure_without_child<T>(self, code: &str) -> EvalResult<T> {
        self.terminal_failure_after_observation(code, None, None)
    }

    fn terminal_failure_after_spawn<T>(
        self,
        code: &str,
        process_id: u32,
        exit_code: Option<i32>,
    ) -> EvalResult<T> {
        self.terminal_failure_after_observation(code, Some(process_id), exit_code)
    }

    fn terminal_failure_after_observation<T>(
        self,
        code: &str,
        process_id: Option<u32>,
        exit_code: Option<i32>,
    ) -> EvalResult<T> {
        let evaluator_pid = self.seal.evaluator_pid;
        let observed_unix_ms = unix_ms()?;
        let receipt = write_terminal_receipt(
            self.seal,
            TerminalReceiptObservation {
                execution_id: &self.execution_id,
                phase: self.phase,
                evaluator_pid,
                process_id,
                started_unix_ms: observed_unix_ms,
                completed_unix_ms: observed_unix_ms,
                exit_code,
                stdout: &[],
                stderr: &[],
                outcome: NativeTerminalOutcome::Failure,
                failure_code: Some(code),
            },
        );
        match receipt {
            Ok(_) => invalid("successor spawn failed after a durable terminal receipt was written"),
            Err(error) => Err(error),
        }
    }
}

#[allow(dead_code)]
impl RunningExecution {
    fn finish(mut self) -> EvalResult<TerminalExecutionReceipt> {
        self.finalize(None)
    }

    fn finalize(&mut self, forced_failure: Option<&str>) -> EvalResult<TerminalExecutionReceipt> {
        if self.terminalization != TerminalizationState::Active {
            return invalid("successor running execution was already terminalized");
        }
        self.terminalization = TerminalizationState::Attempting;
        self.seal
            .as_ref()
            .ok_or_else(|| {
                EvalError::Invalid("successor running execution omitted its seal".to_string())
            })?
            .revalidate_intent_state()?;
        let mut failure_code = forced_failure.map(ToOwned::to_owned);
        if failure_code.is_none() {
            match self.stdin.as_mut() {
                Some(stdin) => {
                    if write_stdin_until(stdin, &self.input, self.response_deadline).is_err() {
                        record_failure_code(&mut failure_code, "stdin_delivery_failed");
                    }
                }
                None => record_failure_code(&mut failure_code, "stdin_unavailable"),
            }
        }
        self.stdin.take();

        let (process_id, exit_code) = if let Some(mut guard) = self.guard.take() {
            let process_id = guard.process_id().map_err(|_| {
                EvalError::Invalid(
                    "successor process ownership changed before terminal observation; durable intent is frozen without a terminal receipt"
                        .to_string(),
                )
            })?;
            if failure_code.is_none() {
                loop {
                    match guard.exited_without_reap(self.response_deadline) {
                        Ok(true) => break,
                        Ok(false) if Instant::now() < self.response_deadline => {
                            let remaining = self
                                .response_deadline
                                .saturating_duration_since(Instant::now());
                            thread::sleep(remaining.min(Duration::from_millis(10)));
                        }
                        Ok(false) => {
                            record_failure_code(&mut failure_code, "process_timeout");
                            break;
                        }
                        Err(_) => {
                            record_failure_code(&mut failure_code, "process_observation_failed");
                            break;
                        }
                    }
                }
            }
            match guard.terminate_group_and_reap_with_status_until(
                self.seal
                    .as_ref()
                    .ok_or_else(|| {
                        EvalError::Invalid(
                            "successor running execution omitted its seal".to_string(),
                        )
                    })?
                    .terminal_deadline,
            ) {
                NativeExecutionCleanupOutcome::ProvenTerminal(status) => {
                    (Some(process_id), status.code())
                }
                NativeExecutionCleanupOutcome::CleanupUnproven(_) => {
                    drop(guard);
                    return invalid(
                        "successor process cleanup remained unproven; durable intent is frozen without a terminal receipt",
                    );
                }
            }
        } else {
            return invalid("successor running execution omitted its process guard");
        };

        let terminal_deadline = self
            .seal
            .as_ref()
            .ok_or_else(|| {
                EvalError::Invalid("successor running execution omitted its seal".to_string())
            })?
            .terminal_deadline;
        let stdout = self
            .stdout
            .take()
            .map(|collector| collector.finish(terminal_deadline))
            .transpose();
        let stderr = self
            .stderr
            .take()
            .map(|collector| collector.finish(terminal_deadline))
            .transpose();
        let stdout = match stdout {
            Ok(Some(bytes)) => bytes,
            _ => {
                record_failure_code(&mut failure_code, "stdout_collection_failed");
                Vec::new()
            }
        };
        let stderr = match stderr {
            Ok(Some(bytes)) => bytes,
            _ => {
                record_failure_code(&mut failure_code, "stderr_collection_failed");
                Vec::new()
            }
        };
        let output_limit = self.output_limit;
        if stdout.len() as u64 > output_limit
            || stderr.len() as u64 > output_limit
            || stdout.len().saturating_add(stderr.len()) as u64 > output_limit
        {
            record_failure_code(&mut failure_code, "output_overflow");
        }
        if failure_code.is_none() && exit_code != Some(0) {
            record_failure_code(&mut failure_code, "nonzero_exit");
        }
        if failure_code.is_none()
            && self.expected_json_object
            && !serde_json::from_slice::<serde_json::Value>(&stdout)
                .ok()
                .is_some_and(|value| value.is_object())
        {
            record_failure_code(&mut failure_code, "malformed_output");
        }
        let outcome = if failure_code.is_none() {
            NativeTerminalOutcome::Success
        } else {
            NativeTerminalOutcome::Failure
        };
        let seal = self.seal.take().ok_or_else(|| {
            EvalError::Invalid("successor running execution omitted its seal".to_string())
        })?;
        let receipt = write_terminal_receipt(
            seal,
            TerminalReceiptObservation {
                execution_id: &self.execution_id,
                phase: self.phase,
                evaluator_pid: std::process::id(),
                process_id,
                started_unix_ms: self.started_unix_ms,
                completed_unix_ms: unix_ms()?,
                exit_code,
                stdout: &stdout,
                stderr: &stderr,
                outcome,
                failure_code: failure_code.as_deref(),
            },
        )?;
        self.terminalization = TerminalizationState::Finished;
        Ok(receipt)
    }
}

impl Drop for RunningExecution {
    fn drop(&mut self) {
        if self.terminalization == TerminalizationState::Active
            && self
                .finalize(Some("unwind_or_abandoned_running_state"))
                .is_ok()
        {
            self.terminalization = TerminalizationState::Finished;
        }
    }
}

fn write_terminal_receipt(
    seal: NativeExecutionSeal,
    observation: TerminalReceiptObservation<'_>,
) -> EvalResult<TerminalExecutionReceipt> {
    seal.revalidate_intent_state()?;
    if observation.evaluator_pid != seal.evaluator_pid
        || observation.completed_unix_ms < observation.started_unix_ms
    {
        return invalid("terminal receipt chronology or evaluator identity was inconsistent");
    }
    let payload = TerminalReceiptPayload {
        document_kind: NativeSuccessorDocumentKind::TerminalReceipt
            .as_str()
            .to_string(),
        execution_id: observation.execution_id.to_string(),
        plan_sha256: seal.plan.sha256.clone(),
        intent_sha256: seal.intent.sha256.clone(),
        phase: observation.phase.as_str().to_string(),
        outcome: observation.outcome,
        evaluator_pid: observation.evaluator_pid,
        process_id: observation.process_id,
        started_unix_ms: observation.started_unix_ms,
        completed_unix_ms: observation.completed_unix_ms,
        exit_code: observation.exit_code,
        stdout_sha256: digest(observation.stdout),
        stderr_sha256: digest(observation.stderr),
        stdout_bytes: observation.stdout.len() as u64,
        stderr_bytes: observation.stderr.len() as u64,
        failure_code: observation.failure_code.map(ToOwned::to_owned),
    };
    let (receipt, receipt_digest) = seal.directory.write_enveloped_pair(
        RECEIPT_FILE,
        RECEIPT_DIGEST_FILE,
        seal.plan.family,
        &payload,
        seal.terminal_deadline,
    )?;
    #[cfg(test)]
    TERMINAL_ARTIFACT_FAULT.with(|fault| match fault.replace(None) {
        Some(TerminalArtifactFault::ReplaceReceipt) => {
            let receipt_path = seal.directory.path.join(RECEIPT_FILE);
            fs::rename(
                &receipt_path,
                seal.directory.path.join("terminal-receipt-moved.json"),
            )
            .unwrap();
            fs::write(&receipt_path, b"replacement").unwrap();
        }
        Some(TerminalArtifactFault::TamperDigest) => {
            fs::write(
                seal.directory.path.join(RECEIPT_DIGEST_FILE),
                b"0000000000000000000000000000000000000000000000000000000000000000\n",
            )
            .unwrap();
        }
        None => {}
    });
    let _receipt_bytes = receipt.read_verified(&seal.directory, MAX_PHASE_INPUT_BYTES as u64)?;
    let receipt_digest_bytes = receipt_digest.read_verified(&seal.directory, 65)?;
    let sidecar = parse_digest_sidecar(&receipt_digest_bytes)?;
    if sidecar != receipt.sha256 {
        return invalid(
            "successor terminal receipt digest sidecar did not bind the retained receipt",
        );
    }
    seal.directory.require_exact_entries(&BTreeSet::from([
        INTENT_FILE.to_string(),
        INTENT_DIGEST_FILE.to_string(),
        RECEIPT_FILE.to_string(),
        RECEIPT_DIGEST_FILE.to_string(),
    ]))?;
    let terminal_receipt = TerminalExecutionReceipt {
        outcome: observation.outcome,
        failure_code: observation.failure_code.map(ToOwned::to_owned),
        receipt_sha256: receipt.sha256,
        captured_stdout: observation.stdout.to_vec(),
        captured_stderr: observation.stderr.to_vec(),
    };
    #[cfg(test)]
    TERMINAL_FINALIZATION_DELAY.with(|delay| {
        if let Some(duration) = delay.replace(None) {
            thread::sleep(duration);
        }
    });
    require_before_deadline(
        seal.terminal_deadline,
        "successor terminal receipt finalization",
    )?;
    Ok(terminal_receipt)
}

impl NativeExecutionSeal {
    fn revalidate_intent_state(&self) -> EvalResult<()> {
        if std::process::id() != self.evaluator_pid || Instant::now() >= self.terminal_deadline {
            return invalid("successor execution seal was stale or crossed a process boundary");
        }
        self.plan.revalidate()?;
        self.directory.revalidate()?;
        self.intent
            .read_verified(&self.directory, MAX_PHASE_INPUT_BYTES as u64)?;
        let intent_digest_bytes = self.intent_digest.read_verified(&self.directory, 65)?;
        self.directory.require_exact_entries(&BTreeSet::from([
            INTENT_FILE.to_string(),
            INTENT_DIGEST_FILE.to_string(),
        ]))?;
        let sidecar = parse_digest_sidecar(&intent_digest_bytes)?;
        if sidecar != self.intent.sha256 {
            return invalid("successor intent digest sidecar did not bind the retained intent");
        }
        require_before_deadline(
            self.terminal_deadline,
            "successor execution seal revalidation",
        )?;
        Ok(())
    }
}

impl PrivateArtifactDirectory {
    #[cfg(not(unix))]
    fn open(_path: &Path) -> EvalResult<Self> {
        invalid("strict successor artifacts require a supported Unix host")
    }

    #[cfg(unix)]
    fn open(path: &Path) -> EvalResult<Self> {
        use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};

        if !path.is_absolute() || path.canonicalize()? != path {
            return invalid(
                "successor artifact directory must be absolute, canonical, and alias-free",
            );
        }
        let metadata = fs::symlink_metadata(path)?;
        let effective_uid = unsafe { libc::geteuid() };
        if metadata.file_type().is_symlink()
            || !metadata.is_dir()
            || metadata.uid() != effective_uid
            || metadata.permissions().mode() & 0o077 != 0
        {
            return invalid("successor artifact directory must be owner-private and link-free");
        }
        let mut options = OpenOptions::new();
        options
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC);
        let file = options.open(path)?;
        let opened = file.metadata()?;
        if !same_directory(&metadata, &opened) {
            return invalid("successor artifact directory identity changed while opening");
        }
        require_no_extended_acl(&file, "successor artifact directory")?;
        Ok(Self {
            file,
            path: path.to_path_buf(),
            metadata: opened,
        })
    }

    fn path(&self) -> &Path {
        &self.path
    }

    #[cfg(unix)]
    fn revalidate(&self) -> EvalResult<()> {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};

        let path_metadata = fs::symlink_metadata(&self.path)?;
        let handle_metadata = self.file.metadata()?;
        let effective_uid = unsafe { libc::geteuid() };
        if self.path.canonicalize()? != self.path
            || !same_directory(&self.metadata, &path_metadata)
            || !same_directory(&self.metadata, &handle_metadata)
            || path_metadata.file_type().is_symlink()
            || !path_metadata.is_dir()
            || path_metadata.uid() != effective_uid
            || path_metadata.permissions().mode() & 0o077 != 0
        {
            return invalid("successor artifact directory identity or permissions changed");
        }
        require_no_extended_acl(&self.file, "successor artifact directory")?;
        Ok(())
    }

    #[cfg(not(unix))]
    fn revalidate(&self) -> EvalResult<()> {
        invalid("strict successor artifacts require a supported Unix host")
    }

    fn require_exact_entries(&self, expected: &BTreeSet<String>) -> EvalResult<()> {
        self.revalidate()?;
        let observed = descriptor_relative_directory_entries(&self.file)?;
        if observed.len() > MAX_PHASE_ARTIFACTS {
            return invalid("successor artifact directory exceeded its closed-world boundary");
        }
        self.revalidate()?;
        if &observed != expected {
            return invalid("successor artifact directory contained partial or unexpected residue");
        }
        Ok(())
    }

    #[cfg(unix)]
    fn open_existing(&self, name: &'static str) -> EvalResult<File> {
        use std::os::fd::{AsRawFd, FromRawFd};

        validate_artifact_name(name)?;
        let name = CString::new(name).map_err(|_| {
            EvalError::Invalid("successor artifact name contained a NUL byte".to_string())
        })?;
        // SAFETY: the retained directory descriptor is live, the leaf is fixed and validated,
        // and O_NOFOLLOW forbids replacing it with a symbolic link.
        let descriptor = unsafe {
            libc::openat(
                self.file.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if descriptor < 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        // SAFETY: openat returned one newly owned descriptor on success.
        Ok(unsafe { File::from_raw_fd(descriptor) })
    }

    #[cfg(unix)]
    fn write_enveloped_pair<T: Serialize>(
        &self,
        primary_name: &'static str,
        digest_name: &'static str,
        family: NativeSuccessorFamily,
        payload: &T,
        deadline: Instant,
    ) -> EvalResult<(SealedArtifact, SealedArtifact)> {
        require_before_deadline(deadline, "successor durable artifact pair")?;
        self.revalidate()?;
        let envelope = SuccessorEnvelope {
            family: family.as_str(),
            schema_version: 1,
            payload,
        };
        let mut bytes = serde_json::to_vec_pretty(&envelope)?;
        bytes.push(b'\n');
        let sha256 = digest(&bytes);
        let primary = self.write_create_new(primary_name, &bytes, sha256.clone(), deadline)?;
        require_before_deadline(deadline, "successor durable primary artifact")?;
        self.file.sync_all()?;
        require_before_deadline(deadline, "successor durable primary directory entry")?;
        let digest_bytes = format!("{sha256}\n").into_bytes();
        let sidecar =
            self.write_create_new(digest_name, &digest_bytes, digest(&digest_bytes), deadline)?;
        require_before_deadline(deadline, "successor durable digest artifact")?;
        self.file.sync_all()?;
        #[cfg(test)]
        DURABLE_WRITE_DELAY.with(|delay| {
            if let Some(duration) = delay.replace(None) {
                thread::sleep(duration);
            }
        });
        require_before_deadline(deadline, "successor durable artifact pair")?;
        Ok((primary, sidecar))
    }

    #[cfg(not(unix))]
    fn write_enveloped_pair<T: Serialize>(
        &self,
        _primary_name: &'static str,
        _digest_name: &'static str,
        _family: NativeSuccessorFamily,
        _payload: &T,
        _deadline: Instant,
    ) -> EvalResult<(SealedArtifact, SealedArtifact)> {
        invalid("strict successor artifacts require a supported Unix host")
    }

    #[cfg(unix)]
    fn write_create_new(
        &self,
        name: &'static str,
        bytes: &[u8],
        sha256: String,
        deadline: Instant,
    ) -> EvalResult<SealedArtifact> {
        use std::os::fd::{AsRawFd, FromRawFd};

        require_before_deadline(deadline, "successor durable artifact")?;
        validate_artifact_name(name)?;
        let artifact_name = name;
        let name = CString::new(artifact_name).map_err(|_| {
            EvalError::Invalid("successor artifact name contained a NUL byte".to_string())
        })?;
        let descriptor = unsafe {
            libc::openat(
                self.file.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDWR | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                0o600,
            )
        };
        if descriptor < 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        let mut file = unsafe { File::from_raw_fd(descriptor) };
        file.write_all(bytes)?;
        require_before_deadline(deadline, "successor durable artifact write")?;
        file.sync_all()?;
        require_before_deadline(deadline, "successor durable artifact fsync")?;
        let metadata = file.metadata()?;
        require_no_extended_acl(&file, "successor durable artifact")?;
        require_before_deadline(deadline, "successor durable artifact metadata")?;
        Ok(SealedArtifact {
            file,
            name: artifact_name,
            metadata,
            sha256,
        })
    }
}

impl SealedArtifact {
    #[cfg(unix)]
    fn revalidate(&self, directory: &PrivateArtifactDirectory) -> EvalResult<()> {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};

        directory.revalidate()?;
        let handle = self.file.metadata()?;
        let relative_file = directory.open_existing(self.name)?;
        let path_metadata = relative_file.metadata()?;
        let effective_uid = unsafe { libc::geteuid() };
        if !same_file(&self.metadata, &handle)
            || !same_file(&self.metadata, &path_metadata)
            || !path_metadata.is_file()
            || path_metadata.uid() != effective_uid
            || path_metadata.nlink() != 1
            || path_metadata.permissions().mode() & 0o077 != 0
        {
            return invalid("successor sealed artifact identity or permissions changed");
        }
        require_no_extended_acl(&self.file, "successor sealed artifact")?;
        require_no_extended_acl(&relative_file, "successor sealed artifact")?;
        directory.revalidate()?;
        Ok(())
    }

    #[cfg(unix)]
    fn read_bounded(&self, max_bytes: u64) -> EvalResult<Vec<u8>> {
        use std::os::unix::fs::FileExt;

        let length = self.file.metadata()?.len();
        if length == 0 || length > max_bytes {
            return invalid("successor sealed artifact exceeded its bounded read contract");
        }
        let mut bytes = vec![
            0_u8;
            usize::try_from(length).map_err(|_| {
                EvalError::Invalid("successor sealed artifact length overflowed".to_string())
            })?
        ];
        let mut offset = 0usize;
        while offset < bytes.len() {
            let read = self.file.read_at(&mut bytes[offset..], offset as u64)?;
            if read == 0 {
                return invalid("successor sealed artifact ended before its accepted length");
            }
            offset = offset.saturating_add(read);
        }
        let mut trailing = [0_u8; 1];
        if self.file.read_at(&mut trailing, length)? != 0 {
            return invalid("successor sealed artifact grew during its retained-handle read");
        }
        Ok(bytes)
    }

    #[cfg(unix)]
    fn read_verified(
        &self,
        directory: &PrivateArtifactDirectory,
        max_bytes: u64,
    ) -> EvalResult<Vec<u8>> {
        self.revalidate(directory)?;
        let bytes = self.read_bounded(max_bytes)?;
        if digest(&bytes) != self.sha256 {
            return invalid("successor sealed artifact content did not match its retained digest");
        }
        self.revalidate(directory)?;
        Ok(bytes)
    }

    #[cfg(not(unix))]
    fn revalidate(&self, _directory: &PrivateArtifactDirectory) -> EvalResult<()> {
        invalid("strict successor artifacts require a supported Unix host")
    }

    #[cfg(not(unix))]
    fn read_bounded(&self, _max_bytes: u64) -> EvalResult<Vec<u8>> {
        invalid("strict successor artifacts require a supported Unix host")
    }

    #[cfg(not(unix))]
    fn read_verified(
        &self,
        _directory: &PrivateArtifactDirectory,
        _max_bytes: u64,
    ) -> EvalResult<Vec<u8>> {
        invalid("strict successor artifacts require a supported Unix host")
    }
}

impl NativeDerivedLaunch {
    fn contract_sha256(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.binary.as_os_str().as_encoded_bytes());
        hasher.update([0]);
        for argument in &self.arguments {
            hasher.update(argument.as_encoded_bytes());
            hasher.update([0]);
        }
        hasher.update(self.cwd.as_os_str().as_encoded_bytes());
        hasher.update([0]);
        for (key, value) in &self.environment {
            hasher.update(key.as_encoded_bytes());
            hasher.update([b'=']);
            hasher.update(value.as_encoded_bytes());
            hasher.update([0]);
        }
        hasher.update(Sha256::digest(&self.input));
        hasher.update(self.output_limit.to_be_bytes());
        hasher.update([u8::from(self.expected_json_object)]);
        format!("{:x}", hasher.finalize())
    }

    #[cfg(test)]
    fn shell_for_test(
        script: &str,
        arguments: &[&str],
        environment: &[(&str, &str)],
        input: &[u8],
        output_limit: u64,
        expected_json_object: bool,
    ) -> Self {
        let binary = Path::new("/bin/sh").canonicalize().unwrap();
        let cwd = Path::new("/private/tmp")
            .canonicalize()
            .or_else(|_| Path::new("/tmp").canonicalize())
            .unwrap();
        let mut argv = vec![
            OsString::from("-c"),
            OsString::from(script),
            OsString::from("stage-b"),
        ];
        argv.extend(arguments.iter().map(OsString::from));
        Self {
            binary,
            arguments: argv,
            cwd,
            environment: environment
                .iter()
                .map(|(key, value)| (OsString::from(key), OsString::from(value)))
                .collect(),
            input: input.to_vec(),
            output_limit,
            expected_json_object,
        }
    }
}

impl BoundedCollector {
    fn start(reader: impl Read + Send + 'static, limit: u64, label: &'static str) -> Self {
        let (sender, receiver) = mpsc::channel();
        let thread = thread::spawn(move || {
            let mut bytes = Vec::new();
            let result = reader.take(limit.saturating_add(1)).read_to_end(&mut bytes);
            let _ = sender.send((result, bytes));
        });
        Self {
            receiver,
            thread: Some(thread),
            label,
        }
    }

    fn finish(mut self, deadline: Instant) -> EvalResult<Vec<u8>> {
        let handle = self.thread.take().ok_or_else(|| {
            EvalError::Invalid(format!("{} collector handle was absent", self.label))
        })?;
        join_thread_until(handle, deadline, self.label)?;
        let (result, bytes) = self.receiver.try_recv().map_err(|_| {
            EvalError::Invalid(format!("{} collector omitted terminal output", self.label))
        })?;
        result?;
        Ok(bytes)
    }
}

#[cfg(unix)]
fn configure_stdin_nonblocking(stdin: &NativeExecutionChildStdin) -> EvalResult<()> {
    stdin.configure_nonblocking()?;
    Ok(())
}

#[cfg(not(unix))]
fn configure_stdin_nonblocking(_stdin: &NativeExecutionChildStdin) -> EvalResult<()> {
    invalid("bounded successor stdin requires a supported Unix host")
}

#[cfg(unix)]
fn write_stdin_until(
    stdin: &mut NativeExecutionChildStdin,
    bytes: &[u8],
    deadline: Instant,
) -> EvalResult<()> {
    let mut delivered = 0usize;
    while delivered < bytes.len() {
        if Instant::now() >= deadline {
            return invalid("successor stdin delivery exceeded its absolute response deadline");
        }
        match stdin.write(&bytes[delivered..]) {
            Ok(0) => return invalid("successor stdin accepted zero bytes"),
            Ok(written) => delivered = delivered.saturating_add(written),
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                let remaining =
                    deadline
                        .checked_duration_since(Instant::now())
                        .ok_or_else(|| {
                            EvalError::Invalid(
                                "successor stdin delivery exceeded its absolute response deadline"
                                    .to_string(),
                            )
                        })?;
                let timeout_ms = remaining.as_millis().clamp(1, i32::MAX as u128) as i32;
                let Some(observed) = stdin.poll_writable(timeout_ms)? else {
                    return invalid("successor stdin did not become writable before its deadline");
                };
                if observed & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL) != 0
                    || observed & libc::POLLOUT == 0
                {
                    return invalid("successor stdin poll reported a terminal descriptor state");
                }
            }
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

#[cfg(not(unix))]
fn write_stdin_until(
    _stdin: &mut NativeExecutionChildStdin,
    _bytes: &[u8],
    _deadline: Instant,
) -> EvalResult<()> {
    invalid("bounded successor stdin requires a supported Unix host")
}

fn join_thread_until(handle: JoinHandle<()>, deadline: Instant, label: &str) -> EvalResult<()> {
    let handle = handle;
    loop {
        if handle.is_finished() {
            return handle.join().map_err(|_| {
                EvalError::Invalid(format!("{label} collector panicked during bounded join"))
            });
        }
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .ok_or_else(|| {
                EvalError::Invalid(format!(
                    "{label} collector exceeded the absolute terminal deadline"
                ))
            })?;
        thread::sleep(remaining.min(Duration::from_millis(5)));
    }
}

fn require_before_deadline(deadline: Instant, label: &str) -> EvalResult<()> {
    if Instant::now() >= deadline {
        return invalid(format!("{label} crossed its absolute terminal deadline"));
    }
    Ok(())
}

fn parse_digest_sidecar(bytes: &[u8]) -> EvalResult<String> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| EvalError::Invalid("successor digest sidecar was not UTF-8".to_string()))?;
    let digest = text.strip_suffix('\n').ok_or_else(|| {
        EvalError::Invalid("successor digest sidecar omitted its exact newline".to_string())
    })?;
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return invalid("successor digest sidecar was not one lowercase SHA-256");
    }
    Ok(digest.to_string())
}

fn validate_execution_id(value: &str) -> EvalResult<()> {
    if value.is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return invalid("successor execution ID must be a bounded lowercase token");
    }
    Ok(())
}

fn validate_artifact_name(value: &str) -> EvalResult<()> {
    if value.is_empty()
        || value.len() > 64
        || value.contains('/')
        || value.contains('\\')
        || value == "."
        || value == ".."
    {
        return invalid("successor artifact name crossed its fixed leaf boundary");
    }
    Ok(())
}

#[cfg(unix)]
#[allow(clippy::useless_conversion)] // libc statvfs field widths vary across supported Unix ABIs.
fn require_disk_reserve(path: &Path, minimum_bytes: u64) -> EvalResult<()> {
    use std::os::unix::ffi::OsStrExt;

    let path = CString::new(path.as_os_str().as_bytes()).map_err(|_| {
        EvalError::Invalid("successor artifact path contained a NUL byte".to_string())
    })?;
    let mut stats = unsafe { std::mem::zeroed::<libc::statvfs>() };
    if unsafe { libc::statvfs(path.as_ptr(), &mut stats) } != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let available = u64::from(stats.f_bavail).saturating_mul(u64::from(stats.f_frsize));
    if available < minimum_bytes {
        return invalid("successor artifact filesystem has insufficient reserved free space");
    }
    Ok(())
}

#[cfg(not(unix))]
fn require_disk_reserve(_path: &Path, _minimum_bytes: u64) -> EvalResult<()> {
    invalid("strict successor disk attestation requires a supported Unix host")
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
struct DirectoryStream(*mut libc::DIR);

#[cfg(any(target_os = "macos", target_os = "linux"))]
impl Drop for DirectoryStream {
    fn drop(&mut self) {
        // SAFETY: fdopendir returned this live stream and it is closed exactly once here.
        unsafe {
            libc::closedir(self.0);
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn descriptor_relative_directory_entries(file: &File) -> EvalResult<BTreeSet<String>> {
    use std::ffi::CStr;
    use std::os::fd::AsRawFd;

    // A dup would share the retained directory's stream offset. Opening "." relative to the
    // retained descriptor creates an independent open-file description for this one enumeration.
    // SAFETY: the retained descriptor is live and the fixed NUL-terminated leaf cannot escape it.
    let enumeration_fd = unsafe {
        libc::openat(
            file.as_raw_fd(),
            c".".as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if enumeration_fd < 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    // SAFETY: fdopendir consumes the independent descriptor on success.
    let stream = unsafe { libc::fdopendir(enumeration_fd) };
    if stream.is_null() {
        let error = std::io::Error::last_os_error();
        // SAFETY: fdopendir did not consume the independent descriptor on failure.
        unsafe {
            libc::close(enumeration_fd);
        }
        return Err(error.into());
    }
    let stream = DirectoryStream(stream);
    let mut entries = BTreeSet::new();
    loop {
        set_directory_errno(0);
        // SAFETY: `stream` owns one live directory stream for the duration of this loop.
        let entry = unsafe { libc::readdir(stream.0) };
        if entry.is_null() {
            let errno = directory_errno();
            if errno != 0 {
                return Err(std::io::Error::from_raw_os_error(errno).into());
            }
            break;
        }
        // SAFETY: d_name is NUL-terminated for the live dirent returned by readdir. The bytes are
        // copied into an owned String before the next readdir call.
        let name = unsafe { CStr::from_ptr((*entry).d_name.as_ptr()) }
            .to_str()
            .map_err(|_| {
                EvalError::Invalid("successor artifact name was not valid UTF-8".to_string())
            })?;
        if name != "." && name != ".." {
            entries.insert(name.to_string());
            if entries.len() > MAX_PHASE_ARTIFACTS {
                return invalid("successor artifact directory exceeded its closed-world boundary");
            }
        }
    }
    Ok(entries)
}

#[cfg(all(unix, not(any(target_os = "macos", target_os = "linux"))))]
fn descriptor_relative_directory_entries(_file: &File) -> EvalResult<BTreeSet<String>> {
    invalid("descriptor-relative successor enumeration is unsupported on this Unix host")
}

#[cfg(not(unix))]
fn descriptor_relative_directory_entries(_file: &File) -> EvalResult<BTreeSet<String>> {
    invalid("descriptor-relative successor enumeration requires a supported Unix host")
}

#[cfg(target_os = "macos")]
fn set_directory_errno(value: libc::c_int) {
    // SAFETY: __error returns the calling thread's errno storage.
    unsafe {
        *libc::__error() = value;
    }
}

#[cfg(target_os = "macos")]
fn directory_errno() -> libc::c_int {
    // SAFETY: __error returns the calling thread's errno storage.
    unsafe { *libc::__error() }
}

#[cfg(target_os = "linux")]
fn set_directory_errno(value: libc::c_int) {
    // SAFETY: __errno_location returns the calling thread's errno storage.
    unsafe {
        *libc::__errno_location() = value;
    }
}

#[cfg(target_os = "linux")]
fn directory_errno() -> libc::c_int {
    // SAFETY: __errno_location returns the calling thread's errno storage.
    unsafe { *libc::__errno_location() }
}

#[cfg(unix)]
fn same_file(left: &Metadata, right: &Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;

    left.dev() == right.dev()
        && left.ino() == right.ino()
        && left.len() == right.len()
        && left.mtime() == right.mtime()
        && left.mtime_nsec() == right.mtime_nsec()
        && left.ctime() == right.ctime()
        && left.ctime_nsec() == right.ctime_nsec()
}

#[cfg(unix)]
fn same_directory(left: &Metadata, right: &Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;

    left.dev() == right.dev() && left.ino() == right.ino()
}

fn unix_ms() -> EvalResult<u64> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| EvalError::Invalid("system clock preceded Unix epoch".to_string()))?
        .as_millis();
    u64::try_from(millis)
        .map_err(|_| EvalError::Invalid("Unix millisecond timestamp overflowed".to_string()))
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn record_failure_code(target: &mut Option<String>, code: &str) {
    if target.is_none() {
        *target = Some(code.to_string());
    }
}

fn invalid<T>(message: impl Into<String>) -> EvalResult<T> {
    Err(EvalError::Invalid(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_document::load_successor_native_document;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct CorrectionRunPlan {
        document_kind: String,
        plan_id: String,
        execution_id: String,
        phase: NativeSuccessorPhase,
        artifact_directory: PathBuf,
    }

    impl NativeSuccessorPayload for CorrectionRunPlan {
        const FAMILY: NativeSuccessorFamily = NativeSuccessorFamily::CorrectionV1;
        const DOCUMENT_KIND: NativeSuccessorDocumentKind = NativeSuccessorDocumentKind::RunPlan;
    }

    impl NativeSuccessorRunPlanPayload for CorrectionRunPlan {
        fn execution_id(&self) -> &str {
            &self.execution_id
        }

        fn phase(&self) -> NativeSuccessorPhase {
            self.phase
        }

        fn artifact_directory(&self) -> &Path {
            &self.artifact_directory
        }
    }

    fn private_directory(path: &Path) {
        fs::create_dir(path).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
        }
    }

    fn fixture(
        script: &str,
        args: &[&str],
        environment: &[(&str, &str)],
        input: &[u8],
        output_limit: u64,
        expected_json: bool,
    ) -> (
        tempfile::TempDir,
        PathBuf,
        LoadedSuccessorPayload<CorrectionRunPlan>,
        NativeInvocationConfirmation,
        NativeDerivedLaunch,
    ) {
        let root = tempfile::tempdir().unwrap();
        let root_path = root.path().canonicalize().unwrap();
        let plan_path = root_path.join("plan.json");
        let artifact_path = root_path.join("artifacts");
        private_directory(&artifact_path);
        let plan_value = serde_json::json!({
            "family":"native_correction_v1",
            "schema_version":1,
            "payload":{
                "document_kind":"run_plan",
                "plan_id":"stage-b-test",
                "execution_id":"stage-b-one",
                "phase":"correction_proposal",
                "artifact_directory":artifact_path
            }
        });
        fs::write(&plan_path, serde_json::to_vec(&plan_value).unwrap()).unwrap();
        let loaded = load_successor_native_document::<CorrectionRunPlan>(&plan_path).unwrap();
        assert_eq!(loaded.payload.document_kind, "run_plan");
        assert_eq!(loaded.payload.plan_id, "stage-b-test");
        let confirmation = NativeInvocationConfirmation {
            execution_id: "stage-b-one".to_string(),
            family: NativeSuccessorFamily::CorrectionV1,
            phase: NativeSuccessorPhase::CorrectionProposal,
            plan_sha256: loaded.sha256().to_string(),
        };
        let launch = NativeDerivedLaunch::shell_for_test(
            script,
            args,
            environment,
            input,
            output_limit,
            expected_json,
        );
        (root, artifact_path, loaded, confirmation, launch)
    }

    fn prepare(
        loaded: LoadedSuccessorPayload<CorrectionRunPlan>,
        confirmation: NativeInvocationConfirmation,
        launch: NativeDerivedLaunch,
        timeout: Duration,
    ) -> PreparedExecution {
        PreparedExecution::prepare(loaded, confirmation, launch, 1, timeout).unwrap()
    }

    #[test]
    fn intent_is_durable_before_spawn_and_terminal_receipt_is_unique() {
        let (_root, artifacts, loaded, confirmation, _placeholder_launch) =
            fixture("printf '{}'", &[], &[], b"", 4096, true);
        let intent_path = artifacts.join(INTENT_FILE);
        let child_marker = artifacts.parent().unwrap().join("child-marker");
        let launch = NativeDerivedLaunch::shell_for_test(
            r#"test -f "$1" && : > "$2" && printf '{"ok":true}'"#,
            &[
                intent_path.to_str().unwrap(),
                child_marker.to_str().unwrap(),
            ],
            &[],
            b"",
            4096,
            true,
        );
        let prepared = prepare(loaded, confirmation, launch, Duration::from_secs(5));
        assert!(!intent_path.exists());
        assert!(!child_marker.exists());
        let committed = prepared.commit_intent().unwrap();
        assert!(intent_path.exists());
        assert!(!child_marker.exists());
        let receipt = committed.spawn().unwrap().finish().unwrap();
        assert_eq!(receipt.outcome, NativeTerminalOutcome::Success);
        assert!(receipt.failure_code.is_none());
        assert!(!receipt.receipt_sha256.is_empty());
        assert_eq!(receipt.captured_stdout, br#"{"ok":true}"#);
        assert!(receipt.captured_stderr.is_empty());
        assert!(child_marker.is_file());
        assert!(artifacts.join(RECEIPT_FILE).is_file());
        assert!(artifacts.join(RECEIPT_DIGEST_FILE).is_file());
        assert!(fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(artifacts.join(RECEIPT_FILE))
            .is_err());
    }

    #[test]
    fn intent_only_state_forbids_replay_and_unexpected_residue_invalidates_preparation() {
        let (_root, artifacts, loaded, confirmation, launch) =
            fixture("printf '{}';", &[], &[], b"", 4096, true);
        let committed = prepare(loaded, confirmation, launch, Duration::from_secs(5))
            .commit_intent()
            .unwrap();
        drop(committed);
        assert!(artifacts.join(INTENT_FILE).is_file());

        let plan_path = artifacts.parent().unwrap().join("plan.json");
        let loaded = load_successor_native_document::<CorrectionRunPlan>(&plan_path).unwrap();
        let confirmation = NativeInvocationConfirmation {
            execution_id: "stage-b-one".to_string(),
            family: NativeSuccessorFamily::CorrectionV1,
            phase: NativeSuccessorPhase::CorrectionProposal,
            plan_sha256: loaded.sha256().to_string(),
        };
        let error = PreparedExecution::prepare(
            loaded,
            confirmation,
            NativeDerivedLaunch::shell_for_test("printf '{}'", &[], &[], b"", 4096, true),
            1,
            Duration::from_secs(5),
        )
        .err()
        .expect("intent residue must reject replay")
        .to_string();
        assert!(error.contains("partial or unexpected residue"));
    }

    #[test]
    fn copied_plan_cannot_replay_into_an_alternate_empty_directory() {
        let (root, artifacts, loaded, confirmation, launch) =
            fixture("printf '{}'", &[], &[], b"", 4096, true);
        prepare(loaded, confirmation, launch, Duration::from_secs(5))
            .commit_intent()
            .unwrap();

        let alternate = root
            .path()
            .canonicalize()
            .unwrap()
            .join("alternate-artifacts");
        private_directory(&alternate);
        let source_plan = root.path().canonicalize().unwrap().join("plan.json");
        let copied_plan = root.path().canonicalize().unwrap().join("copied-plan.json");
        fs::copy(&source_plan, &copied_plan).unwrap();
        let loaded = load_successor_native_document::<CorrectionRunPlan>(&copied_plan).unwrap();
        let confirmation = NativeInvocationConfirmation {
            execution_id: "stage-b-one".to_string(),
            family: NativeSuccessorFamily::CorrectionV1,
            phase: NativeSuccessorPhase::CorrectionProposal,
            plan_sha256: loaded.sha256().to_string(),
        };
        let error = PreparedExecution::prepare(
            loaded,
            confirmation,
            NativeDerivedLaunch::shell_for_test("printf '{}'", &[], &[], b"", 4096, true),
            1,
            Duration::from_secs(5),
        )
        .err()
        .expect("the plan-bound directory must preserve no-replay state")
        .to_string();
        assert!(error.contains("partial or unexpected residue"));
        assert!(fs::read_dir(&alternate).unwrap().next().is_none());
        assert!(artifacts.join(INTENT_FILE).is_file());
    }

    #[test]
    fn cleared_environment_is_observed_without_ambient_secret_inheritance() {
        assert!(std::env::var_os("HOME").is_some());
        let (_root, _artifacts, loaded, confirmation, launch) = fixture(
            r#"if env | grep -q '^HOME='; then exit 91; fi; test "$ONLY_ALLOWED" = yes; printf '{"clean":true}'"#,
            &[],
            &[("ONLY_ALLOWED", "yes")],
            b"",
            4096,
            true,
        );
        let receipt = prepare(loaded, confirmation, launch, Duration::from_secs(5))
            .commit_intent()
            .unwrap()
            .spawn()
            .unwrap()
            .finish()
            .unwrap();
        assert_eq!(receipt.outcome, NativeTerminalOutcome::Success);
    }

    #[test]
    fn bounded_stdin_is_fully_delivered_before_success() {
        let (_root, _artifacts, loaded, confirmation, launch) = fixture(
            r#"IFS= read -r value; test "$value" = bounded-payload; printf '{"input":true}'"#,
            &[],
            &[],
            b"bounded-payload\n",
            4096,
            true,
        );
        let receipt = prepare(loaded, confirmation, launch, Duration::from_secs(5))
            .commit_intent()
            .unwrap()
            .spawn()
            .unwrap()
            .finish()
            .unwrap();
        assert_eq!(receipt.outcome, NativeTerminalOutcome::Success);
        assert_eq!(receipt.captured_stdout, br#"{"input":true}"#);
    }

    #[test]
    fn concurrent_prepared_states_produce_only_one_create_new_intent() {
        let (_root, artifacts, loaded_one, confirmation_one, launch_one) =
            fixture("printf '{}'", &[], &[], b"", 4096, true);
        let plan_path = artifacts.parent().unwrap().join("plan.json");
        let loaded_two = load_successor_native_document::<CorrectionRunPlan>(&plan_path).unwrap();
        let confirmation_two = NativeInvocationConfirmation {
            execution_id: "stage-b-one".to_string(),
            family: NativeSuccessorFamily::CorrectionV1,
            phase: NativeSuccessorPhase::CorrectionProposal,
            plan_sha256: loaded_two.sha256().to_string(),
        };
        let prepared_one = prepare(
            loaded_one,
            confirmation_one,
            launch_one,
            Duration::from_secs(5),
        );
        let prepared_two = prepare(
            loaded_two,
            confirmation_two,
            NativeDerivedLaunch::shell_for_test("printf '{}'", &[], &[], b"", 4096, true),
            Duration::from_secs(5),
        );
        let outcomes = thread::scope(|scope| {
            let first = scope.spawn(move || prepared_one.commit_intent().is_ok());
            let second = scope.spawn(move || prepared_two.commit_intent().is_ok());
            [first.join().unwrap(), second.join().unwrap()]
        });
        assert_eq!(outcomes.into_iter().filter(|outcome| *outcome).count(), 1);
        assert!(artifacts.join(INTENT_FILE).is_file());
        assert!(artifacts.join(INTENT_DIGEST_FILE).is_file());
    }

    #[test]
    fn plan_replacement_after_preparation_fails_before_intent_creation() {
        let (root, artifacts, loaded, confirmation, launch) =
            fixture("printf '{}'", &[], &[], b"", 4096, true);
        let prepared = prepare(loaded, confirmation, launch, Duration::from_secs(5));
        let plan_path = root.path().canonicalize().unwrap().join("plan.json");
        fs::rename(&plan_path, root.path().join("plan-moved.json")).unwrap();
        fs::write(&plan_path, b"replacement").unwrap();

        let error = prepared
            .commit_intent()
            .err()
            .expect("replaced plan must fail before intent creation")
            .to_string();
        assert!(error.contains("identity") || error.contains("safety metadata"));
        assert!(!artifacts.join(INTENT_FILE).exists());
        assert!(!artifacts.join(INTENT_DIGEST_FILE).exists());
    }

    #[test]
    fn tampered_intent_digest_fails_before_spawn() {
        let (_root, artifacts, loaded, confirmation, launch) =
            fixture("printf '{}'", &[], &[], b"", 4096, true);
        let committed = prepare(loaded, confirmation, launch, Duration::from_secs(5))
            .commit_intent()
            .unwrap();
        fs::write(
            artifacts.join(INTENT_DIGEST_FILE),
            b"0000000000000000000000000000000000000000000000000000000000000000\n",
        )
        .unwrap();

        let error = committed
            .spawn()
            .err()
            .expect("tampered intent digest must fail before spawn")
            .to_string();
        assert!(
            error.contains("identity")
                || error.contains("permissions")
                || error.contains("digest sidecar"),
            "{error}"
        );
        assert!(!artifacts.join(RECEIPT_FILE).exists());
    }

    #[test]
    fn post_spawn_cleanup_uncertainty_freezes_intent_without_terminal_receipt() {
        let (_root, artifacts, loaded, confirmation, launch) =
            fixture("sleep 30", &[], &[], b"", 4096, false);
        let committed = prepare(loaded, confirmation, launch, Duration::from_secs(5))
            .commit_intent()
            .unwrap();
        crate::native_execution::inject_next_post_spawn_cleanup_unproven_for_test();

        let error = committed
            .spawn()
            .err()
            .expect("cleanup uncertainty must not create a running or terminal state")
            .to_string();
        assert!(error.contains("cleanup remained unproven"), "{error}");
        assert!(artifacts.join(INTENT_FILE).is_file());
        assert!(artifacts.join(INTENT_DIGEST_FILE).is_file());
        assert!(!artifacts.join(RECEIPT_FILE).exists());
        assert!(!artifacts.join(RECEIPT_DIGEST_FILE).exists());
    }

    #[test]
    fn process_local_seal_is_checked_before_stdin_or_child_operations() {
        let (root, artifacts, loaded, confirmation, _launch) =
            fixture("printf '{}'", &[], &[], b"", 4096, true);
        let marker = root.path().join("stdin-observed");
        let launch = NativeDerivedLaunch::shell_for_test(
            r#"IFS= read -r _ && : > "$1""#,
            &[marker.to_str().unwrap()],
            &[],
            b"must-not-be-delivered\n",
            4096,
            false,
        );
        let mut running = prepare(loaded, confirmation, launch, Duration::from_secs(5))
            .commit_intent()
            .unwrap()
            .spawn()
            .unwrap();
        running.seal.as_mut().unwrap().evaluator_pid = std::process::id().saturating_add(1);

        let result = catch_unwind(AssertUnwindSafe(move || running.finish()));
        assert!(
            result.is_ok(),
            "authority failure must never panic during Drop"
        );
        assert!(result.unwrap().is_err());
        assert!(!marker.exists());
        assert!(!artifacts.join(RECEIPT_FILE).exists());
        assert!(!artifacts.join(RECEIPT_DIGEST_FILE).exists());
    }

    #[test]
    fn receipt_collision_fails_once_without_reentrant_drop_panic() {
        let (_root, artifacts, loaded, confirmation, launch) =
            fixture("sleep 30", &[], &[], b"", 4096, false);
        let running = prepare(loaded, confirmation, launch, Duration::from_secs(5))
            .commit_intent()
            .unwrap()
            .spawn()
            .unwrap();
        fs::write(artifacts.join(RECEIPT_FILE), b"collision").unwrap();

        let result = catch_unwind(AssertUnwindSafe(move || running.finish()));
        assert!(
            result.is_ok(),
            "failed terminalization must not re-enter and panic"
        );
        assert!(result.unwrap().is_err());
        assert!(!artifacts.join(RECEIPT_DIGEST_FILE).exists());
    }

    #[test]
    fn receipt_crossing_deadline_never_returns_authorizing_terminal_state() {
        let (_root, artifacts, loaded, confirmation, launch) =
            fixture("printf '{}'", &[], &[], b"", 4096, true);
        let running = prepare(loaded, confirmation, launch, Duration::from_millis(300))
            .commit_intent()
            .unwrap()
            .spawn()
            .unwrap();
        DURABLE_WRITE_DELAY.with(|delay| delay.set(Some(Duration::from_millis(500))));

        let result = catch_unwind(AssertUnwindSafe(move || running.finish()));
        assert!(
            result.is_ok(),
            "late durable receipt handling must not panic"
        );
        let error = result
            .unwrap()
            .err()
            .expect("late receipt must remain non-authorizing")
            .to_string();
        assert!(error.contains("absolute terminal deadline"), "{error}");
        assert!(artifacts.join(RECEIPT_FILE).is_file());
        assert!(artifacts.join(RECEIPT_DIGEST_FILE).is_file());
    }

    #[test]
    fn post_pair_finalization_crossing_deadline_remains_non_authorizing() {
        let (_root, artifacts, loaded, confirmation, launch) =
            fixture("printf '{}'", &[], &[], b"", 4096, true);
        let running = prepare(loaded, confirmation, launch, Duration::from_millis(300))
            .commit_intent()
            .unwrap()
            .spawn()
            .unwrap();
        TERMINAL_FINALIZATION_DELAY.with(|delay| delay.set(Some(Duration::from_millis(500))));

        let result = catch_unwind(AssertUnwindSafe(move || running.finish()));
        assert!(result.is_ok(), "late finalization must not panic");
        let error = result
            .unwrap()
            .err()
            .expect("post-pair deadline crossing must remain non-authorizing")
            .to_string();
        assert!(error.contains("absolute terminal deadline"), "{error}");
        assert!(artifacts.join(RECEIPT_FILE).is_file());
        assert!(artifacts.join(RECEIPT_DIGEST_FILE).is_file());
    }

    #[test]
    fn terminal_receipt_handles_reject_post_write_replacement_and_tampering() {
        for fault in [
            TerminalArtifactFault::ReplaceReceipt,
            TerminalArtifactFault::TamperDigest,
        ] {
            let (_root, artifacts, loaded, confirmation, launch) =
                fixture("printf '{}'", &[], &[], b"", 4096, true);
            let running = prepare(loaded, confirmation, launch, Duration::from_secs(5))
                .commit_intent()
                .unwrap()
                .spawn()
                .unwrap();
            TERMINAL_ARTIFACT_FAULT.with(|injected| injected.set(Some(fault)));

            let result = catch_unwind(AssertUnwindSafe(move || running.finish()));
            assert!(result.is_ok(), "artifact fault must not panic: {fault:?}");
            let error = result
                .unwrap()
                .err()
                .expect("artifact fault must remain non-authorizing")
                .to_string();
            assert!(
                error.contains("identity")
                    || error.contains("permissions")
                    || error.contains("digest"),
                "{fault:?}: {error}"
            );
            assert!(artifacts.join(INTENT_FILE).is_file());
        }
    }

    #[cfg(unix)]
    #[test]
    fn sealed_intent_and_directory_identity_attacks_fail_before_spawn() {
        use std::os::unix::fs::symlink;

        for attack in ["hardlink", "replace", "directory_swap"] {
            let (root, artifacts, loaded, confirmation, launch) =
                fixture("printf '{}'", &[], &[], b"", 4096, true);
            let committed = prepare(loaded, confirmation, launch, Duration::from_secs(5))
                .commit_intent()
                .unwrap();
            let intent = artifacts.join(INTENT_FILE);
            match attack {
                "hardlink" => fs::hard_link(&intent, root.path().join("intent-hardlink")).unwrap(),
                "replace" => {
                    let moved = root.path().join("intent-moved");
                    fs::rename(&intent, &moved).unwrap();
                    symlink(&moved, &intent).unwrap();
                }
                "directory_swap" => {
                    let moved = root.path().join("artifacts-moved");
                    fs::rename(&artifacts, &moved).unwrap();
                    private_directory(&artifacts);
                }
                _ => unreachable!(),
            }
            let error = committed
                .spawn()
                .err()
                .expect("identity attack must fail before spawn")
                .to_string();
            assert!(
                error.contains("identity")
                    || error.contains("permissions")
                    || error.contains("filesystem"),
                "{attack}: {error}"
            );
        }
    }

    #[test]
    fn partial_intent_residue_fails_before_plan_consumption_can_authorize_execution() {
        let (_root, artifacts, loaded, confirmation, launch) =
            fixture("printf '{}'", &[], &[], b"", 4096, true);
        fs::write(artifacts.join(INTENT_FILE), b"partial").unwrap();
        let error =
            PreparedExecution::prepare(loaded, confirmation, launch, 1, Duration::from_secs(5))
                .err()
                .expect("partial intent must reject preparation")
                .to_string();
        assert!(error.contains("partial or unexpected residue"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn artifact_directory_rejects_extended_acl_access() {
        let (_root, artifacts, loaded, confirmation, launch) =
            fixture("printf '{}'", &[], &[], b"", 4096, true);
        let status = std::process::Command::new("/bin/chmod")
            .args(["+a", "everyone allow write", artifacts.to_str().unwrap()])
            .status()
            .unwrap();
        assert!(status.success());
        let error =
            PreparedExecution::prepare(loaded, confirmation, launch, 1, Duration::from_secs(5))
                .err()
                .expect("artifact directory ACL must be rejected")
                .to_string();
        assert!(error.contains("extended ACL"), "{error}");
    }

    #[test]
    fn timeout_output_overflow_and_malformed_output_are_terminal_failures() {
        for (script, limit, expected_json, expected_failure, timeout) in [
            (
                "sleep 5",
                4096,
                false,
                "process_timeout",
                Duration::from_secs(1),
            ),
            (
                "printf '0123456789'",
                4,
                false,
                "output_overflow",
                Duration::from_secs(5),
            ),
            (
                "printf 'abcdef' >&2",
                4,
                false,
                "output_overflow",
                Duration::from_secs(5),
            ),
            (
                "printf 'abc'; printf 'def' >&2",
                4,
                false,
                "output_overflow",
                Duration::from_secs(5),
            ),
            (
                "printf 'not-json'",
                4096,
                true,
                "malformed_output",
                Duration::from_secs(5),
            ),
        ] {
            let (_root, artifacts, loaded, confirmation, launch) =
                fixture(script, &[], &[], b"", limit, expected_json);
            let receipt = prepare(loaded, confirmation, launch, timeout)
                .commit_intent()
                .unwrap()
                .spawn()
                .unwrap()
                .finish()
                .unwrap();
            assert_eq!(receipt.outcome, NativeTerminalOutcome::Failure);
            assert_eq!(receipt.failure_code.as_deref(), Some(expected_failure));
            assert!(artifacts.join(RECEIPT_FILE).is_file());
        }
    }

    #[test]
    fn blocked_stdin_is_bounded_and_terminalized_as_failure() {
        let input = vec![b'x'; MAX_PHASE_INPUT_BYTES];
        let (_root, artifacts, loaded, confirmation, launch) =
            fixture("sleep 5", &[], &[], &input, 4096, false);
        let receipt = prepare(loaded, confirmation, launch, Duration::from_secs(1))
            .commit_intent()
            .unwrap()
            .spawn()
            .unwrap()
            .finish()
            .unwrap();
        assert_eq!(receipt.outcome, NativeTerminalOutcome::Failure);
        assert_eq!(
            receipt.failure_code.as_deref(),
            Some("stdin_delivery_failed")
        );
        assert!(artifacts.join(RECEIPT_FILE).is_file());
    }

    #[test]
    fn panic_unwind_kills_owned_group_and_writes_terminal_failure() {
        let (_root, artifacts, loaded, confirmation, launch) =
            fixture("sleep 30 & wait", &[], &[], b"", 4096, false);
        let running = prepare(loaded, confirmation, launch, Duration::from_secs(5))
            .commit_intent()
            .unwrap()
            .spawn()
            .unwrap();
        let result = catch_unwind(AssertUnwindSafe(move || {
            let _running = running;
            panic!("injected unwind");
        }));
        assert!(result.is_err());
        assert!(artifacts.join(RECEIPT_FILE).is_file());
        let receipt = fs::read_to_string(artifacts.join(RECEIPT_FILE)).unwrap();
        assert!(receipt.contains("unwind_or_abandoned_running_state"));
    }
}
