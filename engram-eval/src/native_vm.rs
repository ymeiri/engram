//! Provider-free collection and auditing foundations for a mount-free Colima guest.
//!
//! This module deliberately does not authorize AI-provider execution. It freezes the exact
//! Colima launch and guest-inspection commands, creates a no-replay receipt boundary, and derives
//! evidence from runner-owned raw command artifacts instead of accepting observation booleans.
//! Integration with a provider runner is a separate, later-reviewed operation.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use thiserror::Error;

#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
#[cfg(unix)]
use std::os::unix::process::CommandExt;

pub const NATIVE_VM_SCHEMA_VERSION: u32 = 2;
pub const REQUIRED_COLIMA_VERSION: &str = "0.10.1";
pub const REQUIRED_COLIMA_COMMIT: &str = "ed905203afdbc6fd4eae6cc301918099ff31e86e";
pub const REQUIRED_LIMACTL_VERSION: &str = "2.0.3";
pub const MIN_VM_DISK_GIB: u32 = 40;
pub const MAX_VM_BUNDLE_FILES: usize = 16_384;
pub const MAX_VM_BUNDLE_BYTES: u64 = 4 * 1024 * 1024 * 1024;
pub const MAX_VM_FILE_BYTES: u64 = 1024 * 1024 * 1024;
pub const MAX_COMMAND_OUTPUT_BYTES: u64 = 4 * 1024 * 1024;
pub const MAX_COLLECTION_AGE_MS: u64 = 6 * 60 * 60 * 1_000;
const MAX_GUEST_ARG_BYTES: usize = 128 * 1024;
const MAX_CLOCK_SKEW_MS: u64 = 60_000;
const MAX_BUNDLE_DEPTH: usize = 32;
const SHA256_HEX_LEN: usize = 64;
const COLIMA_PROFILE_PREFIX: &str = "engram-vm-";
const GUEST_COLLECTION_PREFIX: &str = "/var/lib/engram-eval/collections";
const TEACHING_USER: &str = "engram-teach";
const EVALUATION_USER: &str = "engram-eval";
const TEACHING_UID: u32 = 12_001;
const EVALUATION_UID: u32 = 12_002;
const BWRAP_APPARMOR_PROFILE: &str = "bwrap-default (enforce)";
const ACCOUNT_POLICY_NONCLAIM: &str = "Claude managed policy remains account-scoped and may influence both hosts; this same-account VM replication does not isolate or attribute that influence.";
const STRUCTURAL_AUDIT_CLAIM: &str =
    "structural artifact consistency only; no causal provenance or provider authority";

const REQUIRED_RUNTIME_NAMES: [&str; 7] = [
    "cargo",
    "claude",
    "codex",
    "engram",
    "engram_eval",
    "node",
    "rustc",
];

const REQUIRED_TOOL_NAMES: [&str; 19] = [
    "aa_status",
    "bwrap",
    "cat",
    "env",
    "find",
    "findmnt",
    "getent",
    "id",
    "install",
    "openssl",
    "printenv",
    "readlink",
    "sha256sum",
    "stat",
    "sudo",
    "tar",
    "test",
    "uname",
    "useradd",
];

const REQUIRED_BASE_LABELS: [&str; 42] = [
    "host_colima_version",
    "host_limactl_version",
    "host_profiles_before",
    "host_colima_start",
    "host_colima_status",
    "guest_uname",
    "guest_boot_id",
    "guest_os_release",
    "guest_teaching_user_absent",
    "guest_evaluation_user_absent",
    "guest_create_teaching_user",
    "guest_create_evaluation_user",
    "guest_secure_teaching_home",
    "guest_secure_evaluation_home",
    "guest_teaching_home_stat",
    "guest_evaluation_home_stat",
    "guest_bundle_root_absent",
    "guest_prepare_bundle_root",
    "guest_transfer_bundle",
    "guest_findmnt",
    "guest_teaching_user",
    "guest_evaluation_user",
    "guest_teaching_identity",
    "guest_evaluation_identity",
    "guest_sockets",
    "guest_ssh_auth_sock",
    "guest_users_path_absent",
    "guest_volumes_path_absent",
    "guest_binfmt_entries",
    "guest_apparmor_enabled",
    "guest_apparmor_status",
    "guest_bwrap_profile",
    "guest_create_cross_uid_canary",
    "guest_cross_uid_probe",
    "guest_bundle_filesystem",
    "guest_root_filesystem_bytes",
    "guest_bundle_inventory",
    "guest_bundle_directories",
    "guest_sha256sum_crosscheck",
    "guest_openssl_crosscheck",
    "guest_boot_id_after",
    "host_colima_status_after",
];

#[derive(Debug, Error)]
pub enum NativeVmError {
    #[error("filesystem error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid native VM foundation: {0}")]
    Invalid(String),
}

pub type NativeVmResult<T> = Result<T, NativeVmError>;

fn invalid(message: impl Into<String>) -> NativeVmError {
    NativeVmError::Invalid(message.into())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NativeVmExecutablePin {
    pub path: String,
    pub sha256: String,
    pub version_argv: Vec<String>,
    pub version_output_sha256: String,
    pub version_marker: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NativeVmBundleEntry {
    pub relative_path: String,
    pub size_bytes: u64,
    pub mode: u32,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NativeVmBundleManifest {
    pub schema_version: u32,
    pub file_count: u32,
    pub total_bytes: u64,
    pub entries: Vec<NativeVmBundleEntry>,
    pub manifest_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NativeVmArchivePin {
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NativeVmCommandRealm {
    Host,
    Guest,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NativeVmCommandInput {
    None,
    BundleArchive,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NativeVmCommandSpec {
    pub ordinal: u32,
    pub label: String,
    pub realm: NativeVmCommandRealm,
    pub executable: String,
    pub argv: Vec<String>,
    pub environment: BTreeMap<String, String>,
    pub stdin: NativeVmCommandInput,
    pub expected_exit_code: i32,
    pub timeout_ms: u64,
    pub max_output_bytes: u64,
    pub command_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NativeVmCollectionContract {
    pub schema_version: u32,
    pub collection_id: String,
    pub created_unix_ms: u64,
    pub expires_unix_ms: u64,
    pub profile_name: String,
    pub host_home: String,
    pub colima_home: String,
    pub profile_config_path: String,
    pub colima: NativeVmExecutablePin,
    pub limactl: NativeVmExecutablePin,
    pub vm_type: String,
    pub architecture: String,
    pub mount_mode: String,
    pub runtime: String,
    pub cpus: u32,
    pub memory_gib: u32,
    pub disk_gib: u32,
    pub root_disk_gib: u32,
    pub guest_bundle_root: String,
    pub teaching_user: String,
    pub teaching_uid: u32,
    pub evaluation_user: String,
    pub evaluation_uid: u32,
    pub bundle_manifest: NativeVmBundleManifest,
    pub bundle_archive: NativeVmArchivePin,
    pub required_runtimes: BTreeMap<String, NativeVmExecutablePin>,
    pub collector_tools: BTreeMap<String, NativeVmExecutablePin>,
    pub command_plan: Vec<NativeVmCommandSpec>,
    pub command_plan_sha256: String,
    pub provider_free: bool,
    pub account_policy_nonclaim: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NativeVmCollectionIntent {
    pub schema_version: u32,
    pub collection_id: String,
    pub contract_sha256: String,
    pub command_plan_sha256: String,
    pub bundle_manifest_sha256: String,
    pub bundle_archive_sha256: String,
    pub created_unix_ms: u64,
    pub expires_unix_ms: u64,
    pub state: String,
    pub profile_path_absent_before_start: bool,
    pub isolated_host_home_empty_before_start: bool,
    pub no_replay: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NativeVmCommandReceipt {
    pub ordinal: u32,
    pub label: String,
    pub command_sha256: String,
    pub expected_exit_code: i32,
    pub exit_code: i32,
    pub started_unix_ms: u64,
    pub completed_unix_ms: u64,
    pub stdout_path: String,
    pub stdout_sha256: String,
    pub stdout_bytes: u64,
    pub stderr_path: String,
    pub stderr_sha256: String,
    pub stderr_bytes: u64,
    pub stdin_sha256: Option<String>,
    pub stdin_bytes: Option<u64>,
    pub timed_out: bool,
    pub output_overflow: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NativeVmCollectionReceipt {
    pub schema_version: u32,
    pub collection_id: String,
    pub contract_sha256: String,
    pub intent_sha256: String,
    pub command_plan_sha256: String,
    pub bundle_manifest_sha256: String,
    pub bundle_archive_sha256: String,
    pub profile_config_sha256: String,
    pub canonical_run_root: String,
    pub run_root_device: u64,
    pub run_root_inode: u64,
    pub raw_root_device: u64,
    pub raw_root_inode: u64,
    pub profile_name: String,
    pub guest_boot_id_sha256: String,
    pub live_identity_sha256: String,
    pub command_transcript_sha256: String,
    pub started_unix_ms: u64,
    pub completed_unix_ms: u64,
    pub collector_pid: u32,
    pub state: String,
    pub no_replay: bool,
    pub commands: Vec<NativeVmCommandReceipt>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NativeVmStructuralAudit {
    pub artifacts_consistent: bool,
    pub collection_id: String,
    pub claim: String,
    pub account_policy_nonclaim: String,
    pub command_count: usize,
    pub runtime_count: usize,
    pub collector_tool_count: usize,
    pub failures: Vec<String>,
    pub remaining_conditions_before_real_vm: Vec<String>,
    pub remaining_conditions_before_provider_execution: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PreparedNativeVmCollection {
    pub run_root: String,
    pub intent_path: String,
    pub intent_sha256: String,
    pub contract_sha256: String,
    pub command_count: usize,
}

/// An opaque, process-local witness that can only be constructed after this module's runner has
/// executed and reaped the exact command plan. It is deliberately neither cloneable nor
/// serializable. Persisted artifacts cannot recreate it and no provider gate exists in this
/// foundation module.
pub struct NativeVmExecutionWitness {
    _seal: NativeVmExecutionSeal,
    run_root_handle: File,
    raw_root_handle: File,
    canonical_run_root: PathBuf,
    contract_sha256: String,
    receipt_sha256: String,
    command_transcript_sha256: String,
    profile_name: String,
    guest_boot_id_sha256: String,
    live_identity_sha256: String,
}

struct NativeVmExecutionSeal;

/// Result available only from the real runner. The structural audit remains explicitly
/// non-authoritative; the opaque witness is intended for a separately reviewed, single-process
/// live recheck and provider gate, which is not implemented here.
pub struct RunnerObservedNativeVmCollection {
    structural_audit: NativeVmStructuralAudit,
    witness: NativeVmExecutionWitness,
}

impl RunnerObservedNativeVmCollection {
    pub fn structural_audit(&self) -> &NativeVmStructuralAudit {
        &self.structural_audit
    }

    pub fn into_execution_witness(self) -> NativeVmExecutionWitness {
        self.witness
    }
}

#[derive(Debug, Clone)]
struct BoundedCommandOutcome {
    status: ExitStatus,
    started_unix_ms: u64,
    completed_unix_ms: u64,
    stdout_bytes: u64,
    stderr_bytes: u64,
    stdout_overflow: bool,
    stderr_overflow: bool,
    timed_out: bool,
    stdin_sha256: Option<String>,
    stdin_bytes: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunnerPostSpawnFault {
    None,
    #[cfg(test)]
    PipeAcquisition,
    #[cfg(test)]
    TryWait,
    #[cfg(test)]
    Wait,
    #[cfg(test)]
    ThreadSetup,
    #[cfg(test)]
    ThreadJoin,
    #[cfg(test)]
    PanicUnwind,
    #[cfg(test)]
    PrimaryAndCleanup,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeVmRunnerTestFault {
    PipeAcquisition,
    TryWait,
    Wait,
    ThreadSetup,
    ThreadJoin,
    PanicUnwind,
    PrimaryAndCleanup,
}

#[cfg(test)]
impl From<NativeVmRunnerTestFault> for RunnerPostSpawnFault {
    fn from(value: NativeVmRunnerTestFault) -> Self {
        match value {
            NativeVmRunnerTestFault::PipeAcquisition => Self::PipeAcquisition,
            NativeVmRunnerTestFault::TryWait => Self::TryWait,
            NativeVmRunnerTestFault::Wait => Self::Wait,
            NativeVmRunnerTestFault::ThreadSetup => Self::ThreadSetup,
            NativeVmRunnerTestFault::ThreadJoin => Self::ThreadJoin,
            NativeVmRunnerTestFault::PanicUnwind => Self::PanicUnwind,
            NativeVmRunnerTestFault::PrimaryAndCleanup => Self::PrimaryAndCleanup,
        }
    }
}

/// Owns the direct child from the instant `spawn` succeeds until a terminal wait has reaped it.
///
/// Normal `Result` failures use `terminate_and_reap` explicitly so a cleanup failure can be
/// returned alongside the primary error. `Drop` is the panic-unwind backstop: it never panics or
/// replaces the active panic, but it reports a cleanup failure to stderr after attempting the same
/// synchronous process-group termination and direct-child reap.
struct DirectChildProcessGroupGuard {
    child: Child,
    process_group_id: u32,
    reaped: bool,
    armed: bool,
    #[cfg(test)]
    inject_cleanup_failure: bool,
    #[cfg(test)]
    inject_try_wait_failure: bool,
    #[cfg(test)]
    inject_wait_failure: bool,
}

impl DirectChildProcessGroupGuard {
    fn new(child: Child) -> Self {
        Self {
            process_group_id: child.id(),
            child,
            reaped: false,
            armed: true,
            #[cfg(test)]
            inject_cleanup_failure: false,
            #[cfg(test)]
            inject_try_wait_failure: false,
            #[cfg(test)]
            inject_wait_failure: false,
        }
    }

    fn id(&self) -> u32 {
        self.process_group_id
    }

    fn child_mut(&mut self) -> &mut Child {
        &mut self.child
    }

    fn try_wait(&mut self) -> std::io::Result<Option<ExitStatus>> {
        #[cfg(test)]
        if std::mem::take(&mut self.inject_try_wait_failure) {
            return Err(std::io::Error::other(
                "injected direct-child try_wait failure",
            ));
        }
        let status = self.child.try_wait()?;
        if status.is_some() {
            self.reaped = true;
        }
        Ok(status)
    }

    fn wait(&mut self) -> std::io::Result<ExitStatus> {
        #[cfg(test)]
        if std::mem::take(&mut self.inject_wait_failure) {
            return Err(std::io::Error::other("injected direct-child wait failure"));
        }
        loop {
            match self.child.wait() {
                Ok(status) => {
                    self.reaped = true;
                    return Ok(status);
                }
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(error),
            }
        }
    }

    fn kill_exact_process_group(&mut self) -> NativeVmResult<()> {
        #[cfg(unix)]
        {
            let process_group_id = libc::pid_t::try_from(self.process_group_id)
                .map_err(|_| invalid("child process-group id does not fit pid_t"))?;
            let result = unsafe { libc::killpg(process_group_id, libc::SIGKILL) };
            if result != 0 {
                let error = std::io::Error::last_os_error();
                if error.raw_os_error() != Some(libc::ESRCH) {
                    return Err(error.into());
                }
            }
            Ok(())
        }
        #[cfg(not(unix))]
        {
            if self.reaped {
                return Ok(());
            }
            match self.child.kill() {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == std::io::ErrorKind::InvalidInput => Ok(()),
                Err(error) => Err(error.into()),
            }
        }
    }

    fn kill_direct_child(&mut self) -> NativeVmResult<()> {
        if self.reaped {
            return Ok(());
        }
        match self.child.kill() {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::InvalidInput => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    fn terminate_and_reap(&mut self) -> NativeVmResult<()> {
        if !self.armed {
            return Ok(());
        }
        let mut failures = Vec::new();
        if let Err(error) = self.kill_exact_process_group() {
            failures.push(format!("process-group SIGKILL failed: {error}"));
        }
        if let Err(error) = self.kill_direct_child() {
            failures.push(format!("direct-child SIGKILL failed: {error}"));
        }
        if !self.reaped {
            if let Err(error) = self.wait() {
                failures.push(format!("direct-child reap failed: {error}"));
            }
        }
        #[cfg(test)]
        if self.inject_cleanup_failure {
            failures.push("injected cleanup failure".to_string());
        }
        if self.reaped {
            self.armed = false;
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(invalid(failures.join("; ")))
        }
    }

    fn disarm_after_confirmed_reap(&mut self) -> NativeVmResult<()> {
        if !self.reaped {
            return Err(invalid(
                "direct child cannot be disarmed before a confirmed terminal reap",
            ));
        }
        self.armed = false;
        Ok(())
    }
}

impl Drop for DirectChildProcessGroupGuard {
    fn drop(&mut self) {
        if self.armed {
            if let Err(error) = self.terminate_and_reap() {
                let _ = writeln!(
                    std::io::stderr().lock(),
                    "native VM child cleanup failure during error/panic unwind; primary failure remains authoritative: {error}"
                );
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct ColimaStatus {
    display_name: String,
    driver: String,
    arch: String,
    runtime: String,
    #[serde(default)]
    docker_socket: String,
    #[serde(default)]
    kubernetes: bool,
    disk: u64,
}

/// Scan an explicit bundle root without following links. Only regular, single-link files with
/// bounded sizes and non-writable group/other mode bits are accepted. Authentication-home names
/// are rejected before any file is opened.
pub fn derive_vm_bundle_manifest(root: &Path) -> NativeVmResult<NativeVmBundleManifest> {
    #[cfg(not(unix))]
    {
        let _ = root;
        return Err(invalid(
            "native VM bundle inspection requires Unix metadata",
        ));
    }
    #[cfg(unix)]
    {
        reject_authentication_lexical_path(root, "bundle root")?;
        let root = canonical_secure_directory(root, "bundle root")?;
        if path_contains_authentication_component(&root) {
            return Err(invalid(
                "VM bundle root traverses an authentication-like path",
            ));
        }
        let owner = fs::symlink_metadata(&root)?.uid();
        let mut entries = Vec::new();
        scan_bundle_directory(&root, &root, owner, 0, &mut entries)?;
        entries.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
        if entries.is_empty() {
            return Err(invalid("VM bundle must contain at least one regular file"));
        }
        if entries.len() > MAX_VM_BUNDLE_FILES {
            return Err(invalid("VM bundle exceeds the file-count bound"));
        }
        let total_bytes = entries.iter().try_fold(0_u64, |total, entry| {
            total
                .checked_add(entry.size_bytes)
                .ok_or_else(|| invalid("VM bundle size overflow"))
        })?;
        if total_bytes > MAX_VM_BUNDLE_BYTES {
            return Err(invalid("VM bundle exceeds the byte bound"));
        }
        let mut manifest = NativeVmBundleManifest {
            schema_version: NATIVE_VM_SCHEMA_VERSION,
            file_count: u32::try_from(entries.len())
                .map_err(|_| invalid("VM bundle file count does not fit in u32"))?,
            total_bytes,
            entries,
            manifest_sha256: String::new(),
        };
        manifest.manifest_sha256 = manifest_payload_digest(&manifest)?;
        Ok(manifest)
    }
}

#[cfg(unix)]
fn scan_bundle_directory(
    root: &Path,
    directory: &Path,
    owner: u32,
    depth: usize,
    entries: &mut Vec<NativeVmBundleEntry>,
) -> NativeVmResult<()> {
    if depth > MAX_BUNDLE_DEPTH {
        return Err(invalid("VM bundle exceeds the directory-depth bound"));
    }
    let mut children = fs::read_dir(directory)?.collect::<Result<Vec<_>, _>>()?;
    children.sort_by_key(|entry| entry.file_name());
    for child in children {
        let path = child.path();
        let relative = path
            .strip_prefix(root)
            .map_err(|_| invalid("bundle path escaped its root"))?;
        validate_relative_bundle_path(relative)?;
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.uid() != owner {
            return Err(invalid(format!(
                "bundle entry has a different owner: {}",
                relative.display()
            )));
        }
        let mode = metadata.permissions().mode() & 0o7777;
        if mode & 0o7000 != 0 {
            return Err(invalid(format!(
                "bundle entry has a setuid, setgid, or sticky mode: {}",
                relative.display()
            )));
        }
        if mode & 0o022 != 0 {
            return Err(invalid(format!(
                "bundle entry is group/other writable: {}",
                relative.display()
            )));
        }
        if metadata.is_dir() {
            scan_bundle_directory(root, &path, owner, depth + 1, entries)?;
            continue;
        }
        if !metadata.is_file() || metadata.nlink() != 1 {
            return Err(invalid(format!(
                "bundle entry is not a regular single-link file: {}",
                relative.display()
            )));
        }
        if metadata.len() > MAX_VM_FILE_BYTES {
            return Err(invalid(format!(
                "bundle file exceeds the per-file bound: {}",
                relative.display()
            )));
        }
        let binding = open_bound_regular_file(&path, owner, None)?;
        entries.push(NativeVmBundleEntry {
            relative_path: slash_path(relative)?,
            size_bytes: metadata.len(),
            mode,
            sha256: sha256_reader(binding)?,
        });
        if entries.len() > MAX_VM_BUNDLE_FILES {
            return Err(invalid("VM bundle exceeds the file-count bound"));
        }
    }
    Ok(())
}

fn validate_relative_bundle_path(path: &Path) -> NativeVmResult<()> {
    if path.as_os_str().is_empty() || path.is_absolute() || path.as_os_str().len() > 4_096 {
        return Err(invalid("bundle path must be a non-empty relative path"));
    }
    let mut depth = 0_usize;
    for component in path.components() {
        let Component::Normal(part) = component else {
            return Err(invalid("bundle path contains a non-normal component"));
        };
        let value = part
            .to_str()
            .ok_or_else(|| invalid("bundle path is not UTF-8"))?;
        if value.is_empty()
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"._+-".contains(&byte))
        {
            return Err(invalid(format!("unsafe bundle path component: {value}")));
        }
        if is_authentication_component(value) {
            return Err(invalid(format!(
                "authentication-like path is forbidden from the VM bundle: {value}"
            )));
        }
        depth += 1;
    }
    if depth > MAX_BUNDLE_DEPTH {
        return Err(invalid("bundle path exceeds the depth bound"));
    }
    Ok(())
}

fn is_authentication_component(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        ".ssh" | ".gnupg" | ".aws" | ".codex" | ".claude"
    ) {
        return true;
    }
    let normalized = lower.trim_start_matches('.');
    matches!(
        normalized,
        "auth"
            | "auth.json"
            | "authentication"
            | "authentication.json"
            | "credential"
            | "credential.json"
            | "credentials"
            | "credentials.json"
            | "keychain"
            | "secret"
            | "secret.json"
            | "secrets"
            | "secrets.json"
            | "token"
            | "token.json"
            | "tokens"
            | "tokens.json"
    ) || [
        "credential.",
        "credentials.",
        "secret.",
        "secrets.",
        "token.",
        "tokens.",
    ]
    .iter()
    .any(|prefix| normalized.starts_with(prefix))
}

fn path_contains_authentication_component(path: &Path) -> bool {
    path.components().any(|component| {
        let Component::Normal(value) = component else {
            return false;
        };
        value.to_str().is_some_and(is_authentication_component)
    })
}

fn reject_authentication_lexical_path(path: &Path, label: &str) -> NativeVmResult<()> {
    require_absolute_clean_path(path, label)?;
    if path_contains_authentication_component(path) {
        return Err(invalid(format!(
            "{label} traverses an authentication- or credential-like path"
        )));
    }
    Ok(())
}

#[cfg(unix)]
fn validate_existing_input_path_before_open(path: &Path, label: &str) -> NativeVmResult<()> {
    reject_authentication_lexical_path(path, label)?;
    let canonical = fs::canonicalize(path)?;
    if path_contains_authentication_component(&canonical) {
        return Err(invalid(format!(
            "{label} resolves to an authentication- or credential-like path"
        )));
    }
    if canonical != path {
        return Err(invalid(format!(
            "{label} traverses a symbolic-link or non-canonical alias"
        )));
    }
    Ok(())
}

/// The exact Colima 0.10.1 launch shape. Every safety-relevant boolean is explicit; there is no
/// inherited template, host activation, generated host SSH config, agent forwarding, Rosetta,
/// binfmt, host mount, or port-forwarder default.
pub fn exact_colima_start_argv(contract: &NativeVmCollectionContract) -> Vec<String> {
    vec![
        "start".to_string(),
        contract.profile_name.clone(),
        "--vm-type".to_string(),
        "vz".to_string(),
        "--arch".to_string(),
        "aarch64".to_string(),
        "--mount".to_string(),
        "none".to_string(),
        "--mount-inotify=false".to_string(),
        "--vz-rosetta=false".to_string(),
        "--binfmt=false".to_string(),
        "--ssh-agent=false".to_string(),
        "--activate=false".to_string(),
        "--ssh-config=false".to_string(),
        "--port-forwarder".to_string(),
        "none".to_string(),
        "--network-address=false".to_string(),
        "--network-host-addresses=false".to_string(),
        "--network-preferred-route=false".to_string(),
        "--network-mode".to_string(),
        "shared".to_string(),
        "--runtime".to_string(),
        "containerd".to_string(),
        "--kubernetes=false".to_string(),
        "--cpus".to_string(),
        contract.cpus.to_string(),
        "--memory".to_string(),
        contract.memory_gib.to_string(),
        "--disk".to_string(),
        contract.disk_gib.to_string(),
        "--root-disk".to_string(),
        contract.root_disk_gib.to_string(),
        "--hostname".to_string(),
        contract.profile_name.clone(),
        "--template=false".to_string(),
        "--save-config=true".to_string(),
    ]
}

fn collector_path(contract: &NativeVmCollectionContract, name: &str) -> NativeVmResult<String> {
    contract
        .collector_tools
        .get(name)
        .map(|pin| pin.path.clone())
        .ok_or_else(|| invalid(format!("missing {name} collector pin")))
}

fn guest_path_environment(contract: &NativeVmCollectionContract) -> NativeVmResult<String> {
    let node = contract
        .required_runtimes
        .get("node")
        .ok_or_else(|| invalid("missing node runtime pin"))?;
    let node_directory = Path::new(&node.path)
        .parent()
        .and_then(Path::to_str)
        .ok_or_else(|| invalid("node runtime path has no UTF-8 parent"))?;
    if node_directory.contains(':') {
        return Err(invalid("node runtime directory contains a PATH separator"));
    }
    Ok(format!(
        "PATH={node_directory}:/usr/sbin:/usr/bin:/sbin:/bin"
    ))
}

pub fn derive_native_vm_command_plan(
    contract: &NativeVmCollectionContract,
) -> NativeVmResult<Vec<NativeVmCommandSpec>> {
    let mut commands = Vec::new();
    push_host_command(
        &mut commands,
        contract,
        "host_colima_version",
        vec!["version".to_string()],
        NativeVmCommandInput::None,
        0,
        30_000,
    )?;
    push_specific_host_command(
        &mut commands,
        contract,
        "host_limactl_version",
        contract.limactl.path.clone(),
        vec!["--version".to_string()],
        NativeVmCommandInput::None,
        0,
        30_000,
    )?;
    push_host_command(
        &mut commands,
        contract,
        "host_profiles_before",
        vec!["list".to_string(), "--json".to_string()],
        NativeVmCommandInput::None,
        0,
        30_000,
    )?;
    push_host_command(
        &mut commands,
        contract,
        "host_colima_start",
        exact_colima_start_argv(contract),
        NativeVmCommandInput::None,
        0,
        15 * 60_000,
    )?;
    push_host_command(
        &mut commands,
        contract,
        "host_colima_status",
        vec![
            "status".to_string(),
            contract.profile_name.clone(),
            "--json".to_string(),
        ],
        NativeVmCommandInput::None,
        0,
        30_000,
    )?;

    push_guest_command(
        &mut commands,
        contract,
        "guest_uname",
        vec![
            collector_path(contract, "uname")?,
            "-s".to_string(),
            "-m".to_string(),
        ],
        NativeVmCommandInput::None,
        0,
        30_000,
    )?;
    push_guest_command(
        &mut commands,
        contract,
        "guest_boot_id",
        vec![
            collector_path(contract, "cat")?,
            "/proc/sys/kernel/random/boot_id".to_string(),
        ],
        NativeVmCommandInput::None,
        0,
        30_000,
    )?;
    push_guest_command(
        &mut commands,
        contract,
        "guest_root_filesystem_bytes",
        vec![
            collector_path(contract, "findmnt")?,
            "--bytes".to_string(),
            "--noheadings".to_string(),
            "--output".to_string(),
            "SIZE".to_string(),
            "--target".to_string(),
            "/".to_string(),
        ],
        NativeVmCommandInput::None,
        0,
        30_000,
    )?;
    push_guest_command(
        &mut commands,
        contract,
        "guest_os_release",
        vec![
            collector_path(contract, "cat")?,
            "/etc/os-release".to_string(),
        ],
        NativeVmCommandInput::None,
        0,
        30_000,
    )?;

    let teach_home = format!("/home/{}", contract.teaching_user);
    let eval_home = format!("/home/{}", contract.evaluation_user);
    push_guest_command(
        &mut commands,
        contract,
        "guest_teaching_user_absent",
        vec![
            collector_path(contract, "getent")?,
            "passwd".to_string(),
            contract.teaching_user.clone(),
        ],
        NativeVmCommandInput::None,
        2,
        30_000,
    )?;
    push_guest_command(
        &mut commands,
        contract,
        "guest_evaluation_user_absent",
        vec![
            collector_path(contract, "getent")?,
            "passwd".to_string(),
            contract.evaluation_user.clone(),
        ],
        NativeVmCommandInput::None,
        2,
        30_000,
    )?;
    for (label, user, uid) in [
        (
            "guest_create_teaching_user",
            contract.teaching_user.as_str(),
            contract.teaching_uid,
        ),
        (
            "guest_create_evaluation_user",
            contract.evaluation_user.as_str(),
            contract.evaluation_uid,
        ),
    ] {
        push_guest_command(
            &mut commands,
            contract,
            label,
            vec![
                collector_path(contract, "sudo")?,
                "-n".to_string(),
                collector_path(contract, "useradd")?,
                "--uid".to_string(),
                uid.to_string(),
                "--create-home".to_string(),
                "--shell".to_string(),
                "/bin/bash".to_string(),
                "--user-group".to_string(),
                user.to_string(),
            ],
            NativeVmCommandInput::None,
            0,
            30_000,
        )?;
    }
    for (label, user, home) in [
        (
            "guest_secure_teaching_home",
            contract.teaching_user.as_str(),
            teach_home.as_str(),
        ),
        (
            "guest_secure_evaluation_home",
            contract.evaluation_user.as_str(),
            eval_home.as_str(),
        ),
    ] {
        push_guest_command(
            &mut commands,
            contract,
            label,
            vec![
                collector_path(contract, "sudo")?,
                "-n".to_string(),
                collector_path(contract, "install")?,
                "-d".to_string(),
                "-m".to_string(),
                "0700".to_string(),
                "-o".to_string(),
                user.to_string(),
                "-g".to_string(),
                user.to_string(),
                home.to_string(),
            ],
            NativeVmCommandInput::None,
            0,
            30_000,
        )?;
    }
    for (label, home) in [
        ("guest_teaching_home_stat", teach_home.as_str()),
        ("guest_evaluation_home_stat", eval_home.as_str()),
    ] {
        push_guest_command(
            &mut commands,
            contract,
            label,
            vec![
                collector_path(contract, "stat")?,
                "--format=%F|%a|%u|%g|%n".to_string(),
                home.to_string(),
            ],
            NativeVmCommandInput::None,
            0,
            30_000,
        )?;
    }
    push_guest_command(
        &mut commands,
        contract,
        "guest_bundle_root_absent",
        vec![
            collector_path(contract, "test")?,
            "!".to_string(),
            "-e".to_string(),
            contract.guest_bundle_root.clone(),
        ],
        NativeVmCommandInput::None,
        0,
        30_000,
    )?;
    push_guest_command(
        &mut commands,
        contract,
        "guest_prepare_bundle_root",
        vec![
            collector_path(contract, "sudo")?,
            "-n".to_string(),
            collector_path(contract, "install")?,
            "-d".to_string(),
            "-m".to_string(),
            "0755".to_string(),
            "-o".to_string(),
            "root".to_string(),
            "-g".to_string(),
            "root".to_string(),
            contract.guest_bundle_root.clone(),
        ],
        NativeVmCommandInput::None,
        0,
        30_000,
    )?;
    push_guest_command(
        &mut commands,
        contract,
        "guest_transfer_bundle",
        vec![
            collector_path(contract, "sudo")?,
            "-n".to_string(),
            collector_path(contract, "tar")?,
            "--extract".to_string(),
            "--file".to_string(),
            "-".to_string(),
            "--directory".to_string(),
            contract.guest_bundle_root.clone(),
            "--no-same-owner".to_string(),
            "--no-overwrite-dir".to_string(),
            "--keep-old-files".to_string(),
        ],
        NativeVmCommandInput::BundleArchive,
        0,
        10 * 60_000,
    )?;
    push_guest_command(
        &mut commands,
        contract,
        "guest_findmnt",
        vec![
            collector_path(contract, "findmnt")?,
            "--json".to_string(),
            "--bytes".to_string(),
            "--output".to_string(),
            "SOURCE,TARGET,FSTYPE,OPTIONS".to_string(),
        ],
        NativeVmCommandInput::None,
        0,
        30_000,
    )?;
    for (label, user) in [
        ("guest_teaching_user", contract.teaching_user.as_str()),
        ("guest_evaluation_user", contract.evaluation_user.as_str()),
    ] {
        push_guest_command(
            &mut commands,
            contract,
            label,
            vec![
                collector_path(contract, "getent")?,
                "passwd".to_string(),
                user.to_string(),
            ],
            NativeVmCommandInput::None,
            0,
            30_000,
        )?;
    }
    for (label, user) in [
        ("guest_teaching_identity", contract.teaching_user.as_str()),
        (
            "guest_evaluation_identity",
            contract.evaluation_user.as_str(),
        ),
    ] {
        push_guest_command(
            &mut commands,
            contract,
            label,
            vec![
                collector_path(contract, "id")?,
                "--groups".to_string(),
                user.to_string(),
            ],
            NativeVmCommandInput::None,
            0,
            30_000,
        )?;
    }
    push_guest_command(
        &mut commands,
        contract,
        "guest_sockets",
        vec![
            collector_path(contract, "sudo")?,
            "-n".to_string(),
            collector_path(contract, "find")?,
            "/run".to_string(),
            teach_home.clone(),
            eval_home.clone(),
            "/tmp".to_string(),
            "-xdev".to_string(),
            "-type".to_string(),
            "s".to_string(),
            "-print".to_string(),
        ],
        NativeVmCommandInput::None,
        0,
        30_000,
    )?;
    push_guest_command(
        &mut commands,
        contract,
        "guest_ssh_auth_sock",
        vec![
            collector_path(contract, "printenv")?,
            "SSH_AUTH_SOCK".to_string(),
        ],
        NativeVmCommandInput::None,
        1,
        30_000,
    )?;
    for (label, path) in [
        ("guest_users_path_absent", "/Users"),
        ("guest_volumes_path_absent", "/Volumes"),
    ] {
        push_guest_command(
            &mut commands,
            contract,
            label,
            vec![
                collector_path(contract, "test")?,
                "!".to_string(),
                "-e".to_string(),
                path.to_string(),
            ],
            NativeVmCommandInput::None,
            0,
            30_000,
        )?;
    }
    push_guest_command(
        &mut commands,
        contract,
        "guest_binfmt_entries",
        vec![
            collector_path(contract, "find")?,
            "/proc/sys/fs/binfmt_misc".to_string(),
            "-mindepth".to_string(),
            "1".to_string(),
            "-maxdepth".to_string(),
            "1".to_string(),
            "-type".to_string(),
            "f".to_string(),
            "-printf".to_string(),
            "%f\\n".to_string(),
        ],
        NativeVmCommandInput::None,
        0,
        30_000,
    )?;
    push_guest_command(
        &mut commands,
        contract,
        "guest_apparmor_enabled",
        vec![
            collector_path(contract, "cat")?,
            "/sys/module/apparmor/parameters/enabled".to_string(),
        ],
        NativeVmCommandInput::None,
        0,
        30_000,
    )?;
    push_guest_command(
        &mut commands,
        contract,
        "guest_apparmor_status",
        vec![
            collector_path(contract, "aa_status")?,
            "--enabled".to_string(),
        ],
        NativeVmCommandInput::None,
        0,
        30_000,
    )?;
    let bwrap = contract
        .collector_tools
        .get("bwrap")
        .ok_or_else(|| invalid("missing bwrap collector pin"))?;
    push_guest_command(
        &mut commands,
        contract,
        "guest_bwrap_profile",
        vec![
            bwrap.path.clone(),
            "--unshare-all".to_string(),
            "--die-with-parent".to_string(),
            "--new-session".to_string(),
            "--ro-bind".to_string(),
            "/".to_string(),
            "/".to_string(),
            collector_path(contract, "cat")?,
            "/proc/self/attr/current".to_string(),
        ],
        NativeVmCommandInput::None,
        0,
        30_000,
    )?;
    let canary = format!("{teach_home}/.engram-cross-uid-canary");
    push_guest_command(
        &mut commands,
        contract,
        "guest_create_cross_uid_canary",
        vec![
            collector_path(contract, "sudo")?,
            "-n".to_string(),
            collector_path(contract, "install")?,
            "-m".to_string(),
            "0600".to_string(),
            "-o".to_string(),
            contract.teaching_user.clone(),
            "-g".to_string(),
            contract.teaching_user.clone(),
            "/dev/null".to_string(),
            canary.clone(),
        ],
        NativeVmCommandInput::None,
        0,
        30_000,
    )?;
    push_guest_command(
        &mut commands,
        contract,
        "guest_cross_uid_probe",
        vec![
            collector_path(contract, "sudo")?,
            "-n".to_string(),
            "-u".to_string(),
            contract.evaluation_user.clone(),
            collector_path(contract, "test")?,
            "!".to_string(),
            "-r".to_string(),
            canary,
        ],
        NativeVmCommandInput::None,
        0,
        30_000,
    )?;
    push_guest_command(
        &mut commands,
        contract,
        "guest_bundle_filesystem",
        vec![
            collector_path(contract, "findmnt")?,
            "--raw".to_string(),
            "--noheadings".to_string(),
            "--output".to_string(),
            "SOURCE,TARGET,FSTYPE".to_string(),
            "--target".to_string(),
            contract.guest_bundle_root.clone(),
        ],
        NativeVmCommandInput::None,
        0,
        30_000,
    )?;
    push_guest_command(
        &mut commands,
        contract,
        "guest_bundle_inventory",
        vec![
            collector_path(contract, "find")?,
            contract.guest_bundle_root.clone(),
            "-xdev".to_string(),
            "-type".to_string(),
            "f".to_string(),
            "-printf".to_string(),
            "%m|%n|%U|%G|%s|%P\\n".to_string(),
        ],
        NativeVmCommandInput::None,
        0,
        60_000,
    )?;
    push_guest_command(
        &mut commands,
        contract,
        "guest_bundle_directories",
        vec![
            collector_path(contract, "find")?,
            contract.guest_bundle_root.clone(),
            "-xdev".to_string(),
            "-type".to_string(),
            "d".to_string(),
            "-printf".to_string(),
            "%m|%U|%G|%P\\n".to_string(),
        ],
        NativeVmCommandInput::None,
        0,
        60_000,
    )?;

    append_executable_attestation_commands(
        &mut commands,
        contract,
        "runtime",
        &contract.required_runtimes,
    )?;
    append_executable_attestation_commands(
        &mut commands,
        contract,
        "tool",
        &contract.collector_tools,
    )?;
    let sha256sum = contract
        .collector_tools
        .get("sha256sum")
        .ok_or_else(|| invalid("missing sha256sum collector pin"))?;
    let openssl = contract
        .collector_tools
        .get("openssl")
        .ok_or_else(|| invalid("missing openssl collector pin"))?;
    push_guest_command(
        &mut commands,
        contract,
        "guest_sha256sum_crosscheck",
        vec![
            openssl.path.clone(),
            "dgst".to_string(),
            "-sha256".to_string(),
            sha256sum.path.clone(),
        ],
        NativeVmCommandInput::None,
        0,
        60_000,
    )?;
    push_guest_command(
        &mut commands,
        contract,
        "guest_openssl_crosscheck",
        vec![sha256sum.path.clone(), openssl.path.clone()],
        NativeVmCommandInput::None,
        0,
        60_000,
    )?;
    push_guest_command(
        &mut commands,
        contract,
        "guest_bundle_rehash",
        bundle_rehash_argv(contract)?,
        NativeVmCommandInput::None,
        0,
        5 * 60_000,
    )?;
    push_guest_command(
        &mut commands,
        contract,
        "guest_special_entries",
        vec![
            collector_path(contract, "find")?,
            contract.guest_bundle_root.clone(),
            "-xdev".to_string(),
            "!".to_string(),
            "-type".to_string(),
            "f".to_string(),
            "!".to_string(),
            "-type".to_string(),
            "d".to_string(),
            "-print".to_string(),
        ],
        NativeVmCommandInput::None,
        0,
        60_000,
    )?;
    push_guest_command(
        &mut commands,
        contract,
        "guest_boot_id_after",
        vec![
            collector_path(contract, "cat")?,
            "/proc/sys/kernel/random/boot_id".to_string(),
        ],
        NativeVmCommandInput::None,
        0,
        30_000,
    )?;
    push_host_command(
        &mut commands,
        contract,
        "host_colima_status_after",
        vec![
            "status".to_string(),
            contract.profile_name.clone(),
            "--json".to_string(),
        ],
        NativeVmCommandInput::None,
        0,
        30_000,
    )?;
    commands.sort_by_key(|command| command_plan_order(&command.label));
    if commands
        .iter()
        .any(|command| command_plan_order(&command.label) == usize::MAX)
    {
        return Err(invalid("command plan contains an unclassified stage"));
    }
    renumber_and_digest_commands(&mut commands)?;
    Ok(commands)
}

fn command_plan_order(label: &str) -> usize {
    let fixed = [
        "host_colima_version",
        "host_limactl_version",
        "host_profiles_before",
        "host_colima_start",
        "host_colima_status",
        "guest_uname",
        "guest_boot_id",
        "guest_root_filesystem_bytes",
        "guest_os_release",
        "guest_findmnt",
        "guest_sockets",
        "guest_ssh_auth_sock",
        "guest_users_path_absent",
        "guest_volumes_path_absent",
        "guest_binfmt_entries",
        "guest_apparmor_enabled",
        "guest_apparmor_status",
        "guest_bwrap_profile",
        "guest_teaching_user_absent",
        "guest_evaluation_user_absent",
        "guest_create_teaching_user",
        "guest_create_evaluation_user",
        "guest_secure_teaching_home",
        "guest_secure_evaluation_home",
        "guest_teaching_home_stat",
        "guest_evaluation_home_stat",
        "guest_teaching_user",
        "guest_evaluation_user",
        "guest_teaching_identity",
        "guest_evaluation_identity",
        "guest_create_cross_uid_canary",
        "guest_cross_uid_probe",
        "guest_bundle_root_absent",
        "guest_prepare_bundle_root",
        "guest_bundle_filesystem",
    ];
    if let Some(index) = fixed.iter().position(|candidate| *candidate == label) {
        return index;
    }
    let base = fixed.len();
    if label.starts_with("guest_tool_") {
        let suffix_rank = executable_attestation_suffix_rank(label);
        return base.saturating_add(suffix_rank);
    }
    match label {
        "guest_sha256sum_crosscheck" => base + 5,
        "guest_openssl_crosscheck" => base + 6,
        "guest_transfer_bundle" => base + 7,
        "guest_bundle_inventory" => base + 8,
        "guest_bundle_directories" => base + 9,
        "guest_bundle_rehash" => base + 10,
        "guest_special_entries" => base + 11,
        _ if label.starts_with("guest_runtime_") => {
            base + 12 + executable_attestation_suffix_rank(label)
        }
        "guest_boot_id_after" => base + 17,
        "host_colima_status_after" => base + 18,
        _ => usize::MAX,
    }
}

fn executable_attestation_suffix_rank(label: &str) -> usize {
    if label.ends_with("_path") {
        0
    } else if label.ends_with("_stat") {
        1
    } else if label.ends_with("_hash") {
        2
    } else if label.ends_with("_hash_alt") {
        3
    } else if label.ends_with("_version") {
        4
    } else {
        usize::MAX
    }
}

fn append_executable_attestation_commands(
    commands: &mut Vec<NativeVmCommandSpec>,
    contract: &NativeVmCollectionContract,
    namespace: &str,
    pins: &BTreeMap<String, NativeVmExecutablePin>,
) -> NativeVmResult<()> {
    let readlink = collector_path(contract, "readlink")?;
    let stat = collector_path(contract, "stat")?;
    let sha256sum = collector_path(contract, "sha256sum")?;
    let openssl = collector_path(contract, "openssl")?;
    for (name, pin) in pins {
        for (suffix, argv) in [
            (
                "path",
                vec![readlink.clone(), "-f".to_string(), pin.path.clone()],
            ),
            (
                "stat",
                vec![
                    stat.clone(),
                    "--format=%F|%h|%a|%u|%g|%s|%n".to_string(),
                    pin.path.clone(),
                ],
            ),
            ("hash", vec![sha256sum.clone(), pin.path.clone()]),
            (
                "hash_alt",
                vec![
                    openssl.clone(),
                    "dgst".to_string(),
                    "-sha256".to_string(),
                    pin.path.clone(),
                ],
            ),
            (
                "version",
                exact_safe_version_argv(namespace, name, &pin.path)?,
            ),
        ] {
            push_guest_command(
                commands,
                contract,
                &format!("guest_{namespace}_{name}_{suffix}"),
                argv,
                NativeVmCommandInput::None,
                0,
                60_000,
            )?;
        }
    }
    Ok(())
}

fn bundle_rehash_argv(contract: &NativeVmCollectionContract) -> NativeVmResult<Vec<String>> {
    if contract.bundle_manifest.entries.len() > MAX_VM_BUNDLE_FILES {
        return Err(invalid("bundle manifest exceeds the command bound"));
    }
    let mut argv = vec![collector_path(contract, "sha256sum")?];
    let mut argv_bytes = argv[0].len() + 1;
    for entry in &contract.bundle_manifest.entries {
        validate_relative_bundle_path(Path::new(&entry.relative_path))?;
        let path = format!("{}/{}", contract.guest_bundle_root, entry.relative_path);
        argv_bytes = argv_bytes
            .checked_add(path.len() + 1)
            .ok_or_else(|| invalid("guest rehash argv size overflow"))?;
        if argv_bytes > MAX_GUEST_ARG_BYTES {
            return Err(invalid("guest rehash argv exceeds its byte bound"));
        }
        argv.push(path);
    }
    Ok(argv)
}

fn push_host_command(
    commands: &mut Vec<NativeVmCommandSpec>,
    contract: &NativeVmCollectionContract,
    label: &str,
    argv: Vec<String>,
    stdin: NativeVmCommandInput,
    expected_exit_code: i32,
    timeout_ms: u64,
) -> NativeVmResult<()> {
    push_specific_host_command(
        commands,
        contract,
        label,
        contract.colima.path.clone(),
        argv,
        stdin,
        expected_exit_code,
        timeout_ms,
    )
}

#[allow(clippy::too_many_arguments)]
fn push_specific_host_command(
    commands: &mut Vec<NativeVmCommandSpec>,
    contract: &NativeVmCollectionContract,
    label: &str,
    executable: String,
    argv: Vec<String>,
    stdin: NativeVmCommandInput,
    expected_exit_code: i32,
    timeout_ms: u64,
) -> NativeVmResult<()> {
    let environment = host_command_environment(contract)?;
    commands.push(NativeVmCommandSpec {
        ordinal: 0,
        label: label.to_string(),
        realm: NativeVmCommandRealm::Host,
        executable,
        argv,
        environment,
        stdin,
        expected_exit_code,
        timeout_ms,
        max_output_bytes: MAX_COMMAND_OUTPUT_BYTES,
        command_sha256: String::new(),
    });
    Ok(())
}

fn push_guest_command(
    commands: &mut Vec<NativeVmCommandSpec>,
    contract: &NativeVmCollectionContract,
    label: &str,
    guest_argv: Vec<String>,
    stdin: NativeVmCommandInput,
    expected_exit_code: i32,
    timeout_ms: u64,
) -> NativeVmResult<()> {
    if guest_argv.is_empty() || !Path::new(&guest_argv[0]).is_absolute() {
        return Err(invalid("guest commands require an absolute executable"));
    }
    let mut argv = vec![
        "--profile".to_string(),
        contract.profile_name.clone(),
        "ssh".to_string(),
        "--".to_string(),
        collector_path(contract, "env")?,
        "-i".to_string(),
        guest_path_environment(contract)?,
        "LC_ALL=C".to_string(),
        "LANG=C".to_string(),
        "HOME=/var/empty".to_string(),
    ];
    argv.extend(guest_argv);
    push_host_command(
        commands,
        contract,
        label,
        argv,
        stdin,
        expected_exit_code,
        timeout_ms,
    )?;
    commands.last_mut().expect("command was just pushed").realm = NativeVmCommandRealm::Guest;
    Ok(())
}

fn host_command_environment(
    contract: &NativeVmCollectionContract,
) -> NativeVmResult<BTreeMap<String, String>> {
    require_absolute_clean_path(Path::new(&contract.host_home), "host home")?;
    require_absolute_clean_path(Path::new(&contract.colima_home), "Colima home")?;
    let limactl_parent = Path::new(&contract.limactl.path)
        .parent()
        .and_then(Path::to_str)
        .ok_or_else(|| invalid("limactl path has no UTF-8 parent"))?;
    let colima_parent = Path::new(&contract.colima.path)
        .parent()
        .and_then(Path::to_str)
        .ok_or_else(|| invalid("Colima path has no UTF-8 parent"))?;
    if limactl_parent.contains(':') || colima_parent.contains(':') {
        return Err(invalid("host runtime directory contains a PATH separator"));
    }
    Ok(BTreeMap::from([
        ("COLIMA_HOME".to_string(), contract.colima_home.clone()),
        ("HOME".to_string(), contract.host_home.clone()),
        ("LANG".to_string(), "C".to_string()),
        ("LC_ALL".to_string(), "C".to_string()),
        (
            "PATH".to_string(),
            format!("{limactl_parent}:{colima_parent}:/usr/bin:/bin"),
        ),
        ("TMPDIR".to_string(), "/tmp".to_string()),
    ]))
}

fn renumber_and_digest_commands(commands: &mut [NativeVmCommandSpec]) -> NativeVmResult<()> {
    let mut labels = BTreeSet::new();
    for (index, command) in commands.iter_mut().enumerate() {
        if !labels.insert(command.label.clone()) {
            return Err(invalid("command plan contains a duplicate label"));
        }
        command.ordinal = u32::try_from(index)
            .map_err(|_| invalid("command plan ordinal does not fit in u32"))?;
        command.command_sha256 = command_payload_digest(command)?;
    }
    Ok(())
}

fn command_payload_digest(command: &NativeVmCommandSpec) -> NativeVmResult<String> {
    let mut payload = command.clone();
    payload.command_sha256.clear();
    serialized_digest(&payload)
}

fn manifest_payload_digest(manifest: &NativeVmBundleManifest) -> NativeVmResult<String> {
    let mut payload = manifest.clone();
    payload.manifest_sha256.clear();
    serialized_digest(&payload)
}

fn serialized_digest<T: Serialize>(value: &T) -> NativeVmResult<String> {
    Ok(sha256_bytes(&serde_json::to_vec(value)?))
}

pub fn native_vm_contract_sha256(contract: &NativeVmCollectionContract) -> NativeVmResult<String> {
    serialized_digest(contract)
}

/// Recompute the derived command plan and validate every closed-world contract invariant.
pub fn finalize_native_vm_contract(
    mut contract: NativeVmCollectionContract,
) -> NativeVmResult<NativeVmCollectionContract> {
    contract.command_plan.clear();
    contract.command_plan_sha256.clear();
    let commands = derive_native_vm_command_plan(&contract)?;
    contract.command_plan_sha256 = serialized_digest(&commands)?;
    contract.command_plan = commands;
    validate_native_vm_contract(&contract)?;
    Ok(contract)
}

pub fn validate_native_vm_contract(contract: &NativeVmCollectionContract) -> NativeVmResult<()> {
    if contract.schema_version != NATIVE_VM_SCHEMA_VERSION {
        return Err(invalid("unsupported native VM schema"));
    }
    validate_id(&contract.collection_id, "collection id")?;
    validate_id(&contract.profile_name, "profile name")?;
    if contract.profile_name == "default"
        || !contract.profile_name.starts_with(COLIMA_PROFILE_PREFIX)
        || contract.profile_name == COLIMA_PROFILE_PREFIX
    {
        return Err(invalid(
            "Colima profile must be a named non-default engram-vm-* profile",
        ));
    }
    if contract.created_unix_ms == 0
        || contract.expires_unix_ms <= contract.created_unix_ms
        || contract.expires_unix_ms - contract.created_unix_ms > MAX_COLLECTION_AGE_MS
    {
        return Err(invalid("collection freshness window is invalid"));
    }
    validate_host_paths(contract)?;
    validate_colima_pin(&contract.colima)?;
    validate_limactl_pin(&contract.limactl)?;
    #[cfg(unix)]
    {
        validate_existing_regular_file_metadata(
            Path::new(&contract.colima.path),
            "Colima executable",
            None,
            true,
        )?;
        validate_existing_regular_file_metadata(
            Path::new(&contract.limactl.path),
            "limactl executable",
            None,
            true,
        )?;
    }
    if contract.colima.path == contract.limactl.path
        || contract.colima.sha256 == contract.limactl.sha256
    {
        return Err(invalid(
            "Colima and limactl must be distinct pinned host executables",
        ));
    }
    if contract.vm_type != "vz"
        || contract.architecture != "aarch64"
        || contract.mount_mode != "none"
        || contract.runtime != "containerd"
        || contract.cpus == 0
        || contract.memory_gib == 0
        || contract.disk_gib < MIN_VM_DISK_GIB
        || contract.root_disk_gib < MIN_VM_DISK_GIB
    {
        return Err(invalid(
            "VM shape must be VZ/aarch64/mount-none/containerd with positive CPU/memory and at least 40-GiB disks",
        ));
    }
    let expected_guest_root = format!(
        "{GUEST_COLLECTION_PREFIX}/{}/bundle",
        contract.collection_id
    );
    if contract.guest_bundle_root != expected_guest_root {
        return Err(invalid(
            "guest bundle root is not derived from collection id",
        ));
    }
    if contract.teaching_user != TEACHING_USER
        || contract.evaluation_user != EVALUATION_USER
        || contract.teaching_uid != TEACHING_UID
        || contract.evaluation_uid != EVALUATION_UID
        || contract.teaching_uid == 0
        || contract.evaluation_uid == 0
        || contract.teaching_uid == contract.evaluation_uid
    {
        return Err(invalid(
            "teaching and evaluation identities must be the frozen distinct unprivileged users",
        ));
    }
    validate_bundle_manifest(&contract.bundle_manifest)?;
    validate_archive_pin(&contract.bundle_archive)?;
    #[cfg(unix)]
    validate_existing_regular_file_metadata(
        Path::new(&contract.bundle_archive.path),
        "bundle archive",
        Some(contract.bundle_archive.size_bytes),
        false,
    )?;
    validate_pin_map(
        &contract.required_runtimes,
        &REQUIRED_RUNTIME_NAMES,
        "runtime",
        Some(&contract.guest_bundle_root),
    )?;
    validate_runtime_bundle_binding(contract)?;
    validate_pin_map(
        &contract.collector_tools,
        &REQUIRED_TOOL_NAMES,
        "collector tool",
        None,
    )?;
    if !contract.provider_free || contract.account_policy_nonclaim != ACCOUNT_POLICY_NONCLAIM {
        return Err(invalid(
            "foundation must remain provider-free and preserve the Claude managed-policy nonclaim",
        ));
    }
    let mut derivation_input = contract.clone();
    derivation_input.command_plan.clear();
    derivation_input.command_plan_sha256.clear();
    let derived = derive_native_vm_command_plan(&derivation_input)?;
    if contract.command_plan != derived
        || !is_sha256(&contract.command_plan_sha256)
        || serialized_digest(&derived)? != contract.command_plan_sha256
    {
        return Err(invalid(
            "native VM command plan is not the exact derived plan",
        ));
    }
    validate_command_plan(contract)?;
    Ok(())
}

fn validate_runtime_bundle_binding(contract: &NativeVmCollectionContract) -> NativeVmResult<()> {
    let prefix = format!("{}/", contract.guest_bundle_root);
    let entries = contract
        .bundle_manifest
        .entries
        .iter()
        .map(|entry| (entry.relative_path.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    for (name, pin) in &contract.required_runtimes {
        let relative = pin
            .path
            .strip_prefix(&prefix)
            .ok_or_else(|| invalid(format!("runtime {name} is not inside the streamed bundle")))?;
        let entry = entries
            .get(relative)
            .ok_or_else(|| invalid(format!("runtime {name} is absent from the bundle manifest")))?;
        if entry.sha256 != pin.sha256 || entry.size_bytes == 0 || entry.mode & 0o111 == 0 {
            return Err(invalid(format!(
                "runtime {name} is not bound to an executable manifest entry"
            )));
        }
    }
    Ok(())
}

fn validate_host_paths(contract: &NativeVmCollectionContract) -> NativeVmResult<()> {
    let host_home = Path::new(&contract.host_home);
    let colima_home = Path::new(&contract.colima_home);
    let profile_config = Path::new(&contract.profile_config_path);
    reject_authentication_lexical_path(host_home, "host home")?;
    reject_authentication_lexical_path(colima_home, "Colima home")?;
    reject_authentication_lexical_path(profile_config, "profile config")?;
    if colima_home != host_home.join(".colima")
        || profile_config != colima_home.join(&contract.profile_name).join("colima.yaml")
    {
        return Err(invalid(
            "Colima home/config path must be the exact .colima profile path under host home",
        ));
    }
    Ok(())
}

#[cfg(unix)]
fn validate_fresh_isolated_host_home(contract: &NativeVmCollectionContract) -> NativeVmResult<()> {
    let host_home = Path::new(&contract.host_home);
    let colima_home = Path::new(&contract.colima_home);
    require_canonical_owner_private_directory(host_home, "isolated host home")?;
    require_canonical_owner_private_directory(colima_home, "isolated Colima home")?;
    if directory_entry_names(host_home)? != BTreeSet::from([".colima".to_string()])
        || !directory_entry_names(colima_home)?.is_empty()
    {
        return Err(invalid(
            "isolated host home must be fresh and contain only an empty .colima directory",
        ));
    }
    Ok(())
}

fn validate_colima_pin(pin: &NativeVmExecutablePin) -> NativeVmResult<()> {
    validate_executable_pin(pin, "Colima")?;
    if Path::new(&pin.path)
        .file_name()
        .and_then(|value| value.to_str())
        != Some("colima")
        || pin.version_argv != ["version"]
        || pin.version_marker
            != format!(
                "colima version {REQUIRED_COLIMA_VERSION}\ngit commit: {REQUIRED_COLIMA_COMMIT}"
            )
    {
        return Err(invalid(
            "Colima pin is not exact version 0.10.1/official commit",
        ));
    }
    Ok(())
}

fn validate_limactl_pin(pin: &NativeVmExecutablePin) -> NativeVmResult<()> {
    validate_executable_pin(pin, "limactl")?;
    if Path::new(&pin.path)
        .file_name()
        .and_then(|value| value.to_str())
        != Some("limactl")
        || pin.version_argv != ["--version"]
        || pin.version_marker != format!("limactl version {REQUIRED_LIMACTL_VERSION}")
    {
        return Err(invalid("limactl pin is not exact version 2.0.3"));
    }
    Ok(())
}

fn validate_executable_pin(pin: &NativeVmExecutablePin, label: &str) -> NativeVmResult<()> {
    reject_authentication_lexical_path(Path::new(&pin.path), label)?;
    if !is_sha256(&pin.sha256)
        || !is_sha256(&pin.version_output_sha256)
        || pin.version_argv.is_empty()
        || pin.version_argv.len() > 32
        || pin.version_argv.iter().map(String::len).sum::<usize>() > 64 * 1024
        || pin.version_marker.trim().is_empty()
        || pin.version_marker.len() > 4_096
    {
        return Err(invalid(format!("{label} executable pin is incomplete")));
    }
    if pin.version_argv.iter().any(|value| {
        value.is_empty() || value.len() > 4_096 || contains_forbidden_runtime_word(value)
    }) {
        return Err(invalid(format!("{label} version argv is unsafe")));
    }
    Ok(())
}

fn validate_pin_map<const N: usize>(
    pins: &BTreeMap<String, NativeVmExecutablePin>,
    required: &[&str; N],
    label: &str,
    guest_bundle_root: Option<&str>,
) -> NativeVmResult<()> {
    let expected = required
        .iter()
        .map(|name| (*name).to_string())
        .collect::<BTreeSet<_>>();
    if pins.keys().cloned().collect::<BTreeSet<_>>() != expected {
        return Err(invalid(format!("{label} set is not exact")));
    }
    let mut paths = BTreeSet::new();
    let mut hashes = BTreeSet::new();
    for (name, pin) in pins {
        validate_executable_pin(pin, &format!("{label} {name}"))?;
        let expected_path = if let Some(root) = guest_bundle_root {
            format!("{root}/bin/{name}")
        } else {
            exact_collector_tool_path(name)?.to_string()
        };
        if pin.path != expected_path {
            return Err(invalid(format!(
                "{label} {name} path is not the exact frozen guest location"
            )));
        }
        if !paths.insert(pin.path.as_str()) {
            return Err(invalid(format!("{label} paths must be distinct")));
        }
        if !hashes.insert(pin.sha256.as_str()) {
            return Err(invalid(format!("{label} hashes must be distinct")));
        }
        let namespace = if guest_bundle_root.is_some() {
            "runtime"
        } else {
            "tool"
        };
        if pin.version_argv != exact_safe_version_argv(namespace, name, &pin.path)? {
            return Err(invalid(format!(
                "{label} {name} version command is not the exact provider-free probe"
            )));
        }
    }
    Ok(())
}

fn exact_collector_tool_path(name: &str) -> NativeVmResult<&'static str> {
    match name {
        "aa_status" => Ok("/usr/sbin/aa-status"),
        "bwrap" => Ok("/usr/bin/bwrap"),
        "cat" => Ok("/usr/bin/cat"),
        "env" => Ok("/usr/bin/env"),
        "find" => Ok("/usr/bin/find"),
        "findmnt" => Ok("/usr/bin/findmnt"),
        "getent" => Ok("/usr/bin/getent"),
        "id" => Ok("/usr/bin/id"),
        "install" => Ok("/usr/bin/install"),
        "openssl" => Ok("/usr/bin/openssl"),
        "printenv" => Ok("/usr/bin/printenv"),
        "readlink" => Ok("/usr/bin/readlink"),
        "sha256sum" => Ok("/usr/bin/sha256sum"),
        "stat" => Ok("/usr/bin/stat"),
        "sudo" => Ok("/usr/bin/sudo"),
        "tar" => Ok("/usr/bin/tar"),
        "test" => Ok("/usr/bin/test"),
        "uname" => Ok("/usr/bin/uname"),
        "useradd" => Ok("/usr/sbin/useradd"),
        _ => Err(invalid("unknown collector tool version/path contract")),
    }
}

fn exact_safe_version_argv(namespace: &str, name: &str, path: &str) -> NativeVmResult<Vec<String>> {
    let argument = match (namespace, name) {
        ("runtime", "cargo" | "claude" | "codex" | "engram" | "engram_eval" | "node" | "rustc") => {
            "--version"
        }
        ("tool", "openssl") => "version",
        (
            "tool",
            "aa_status" | "bwrap" | "cat" | "env" | "find" | "findmnt" | "getent" | "id"
            | "install" | "printenv" | "readlink" | "sha256sum" | "stat" | "sudo" | "tar" | "test"
            | "uname" | "useradd",
        ) => "--version",
        _ => return Err(invalid("unknown executable version probe")),
    };
    Ok(vec![path.to_string(), argument.to_string()])
}

fn validate_bundle_manifest(manifest: &NativeVmBundleManifest) -> NativeVmResult<()> {
    if manifest.schema_version != NATIVE_VM_SCHEMA_VERSION
        || manifest.entries.is_empty()
        || manifest.entries.len() > MAX_VM_BUNDLE_FILES
        || usize::try_from(manifest.file_count).ok() != Some(manifest.entries.len())
        || manifest.total_bytes > MAX_VM_BUNDLE_BYTES
        || !is_sha256(&manifest.manifest_sha256)
        || manifest_payload_digest(manifest)? != manifest.manifest_sha256
    {
        return Err(invalid("bundle manifest metadata or digest is invalid"));
    }
    let mut paths = BTreeSet::new();
    let mut total = 0_u64;
    for entry in &manifest.entries {
        validate_relative_bundle_path(Path::new(&entry.relative_path))?;
        if !paths.insert(entry.relative_path.as_str())
            || entry.size_bytes > MAX_VM_FILE_BYTES
            || entry.mode & 0o7000 != 0
            || entry.mode & 0o022 != 0
            || entry.mode == 0
            || entry.mode > 0o777
            || !is_sha256(&entry.sha256)
        {
            return Err(invalid(
                "bundle manifest contains an invalid or duplicate entry",
            ));
        }
        total = total
            .checked_add(entry.size_bytes)
            .ok_or_else(|| invalid("bundle manifest size overflow"))?;
    }
    if total != manifest.total_bytes {
        return Err(invalid("bundle manifest total does not match its entries"));
    }
    if !manifest
        .entries
        .windows(2)
        .all(|pair| pair[0].relative_path < pair[1].relative_path)
    {
        return Err(invalid("bundle manifest entries are not strictly sorted"));
    }
    Ok(())
}

fn validate_archive_pin(pin: &NativeVmArchivePin) -> NativeVmResult<()> {
    let path = Path::new(&pin.path);
    reject_authentication_lexical_path(path, "bundle archive")?;
    if pin.size_bytes == 0
        || pin.size_bytes > MAX_VM_BUNDLE_BYTES + (64 * 1024 * 1024)
        || !is_sha256(&pin.sha256)
    {
        return Err(invalid("bundle archive pin is invalid"));
    }
    Ok(())
}

fn validate_command_plan(contract: &NativeVmCollectionContract) -> NativeVmResult<()> {
    let labels = contract
        .command_plan
        .iter()
        .map(|command| command.label.as_str())
        .collect::<BTreeSet<_>>();
    if REQUIRED_BASE_LABELS
        .iter()
        .any(|label| !labels.contains(label))
        || !labels.contains("guest_bundle_rehash")
        || !labels.contains("guest_special_entries")
        || labels.len() != contract.command_plan.len()
    {
        return Err(invalid("command plan is not the required closed-world set"));
    }
    let transfer_count = contract
        .command_plan
        .iter()
        .filter(|command| command.stdin == NativeVmCommandInput::BundleArchive)
        .count();
    if transfer_count != 1 {
        return Err(invalid("command plan must stream the archive exactly once"));
    }
    for (index, command) in contract.command_plan.iter().enumerate() {
        if usize::try_from(command.ordinal).ok() != Some(index)
            || !safe_label(&command.label)
            || !matches!(
                command.executable.as_str(),
                path if path == contract.colima.path || path == contract.limactl.path
            )
            || command.environment != host_command_environment(contract)?
            || command.timeout_ms == 0
            || command.max_output_bytes == 0
            || command.max_output_bytes > MAX_COMMAND_OUTPUT_BYTES
            || command.command_sha256 != command_payload_digest(command)?
            || command_contains_forbidden_surface(command)
        {
            return Err(invalid(format!(
                "command plan entry is invalid: {}",
                command.label
            )));
        }
        if command.realm == NativeVmCommandRealm::Guest
            && (command.executable != contract.colima.path
                || !has_exact_guest_prefix(command, contract))
        {
            return Err(invalid(
                "guest command escaped the exact environment wrapper",
            ));
        }
    }
    let start = contract
        .command_plan
        .iter()
        .find(|command| command.label == "host_colima_start")
        .ok_or_else(|| invalid("missing Colima start command"))?;
    if start.argv != exact_colima_start_argv(contract) {
        return Err(invalid("Colima start argv drifted"));
    }
    let start_tokens = start.argv.iter().map(String::as_str).collect::<Vec<_>>();
    for required in [
        "--mount",
        "--mount-inotify=false",
        "--vz-rosetta=false",
        "--binfmt=false",
        "--ssh-agent=false",
        "--activate=false",
        "--ssh-config=false",
        "--port-forwarder",
        "--network-address=false",
        "--network-host-addresses=false",
        "--network-preferred-route=false",
        "--template=false",
    ] {
        if start_tokens
            .iter()
            .filter(|token| **token == required)
            .count()
            != 1
        {
            return Err(invalid(format!(
                "Colima start argv does not contain exactly one {required}"
            )));
        }
    }
    if start_tokens
        .windows(2)
        .filter(|pair| *pair == ["--mount", "none"])
        .count()
        != 1
        || start_tokens
            .windows(2)
            .filter(|pair| *pair == ["--port-forwarder", "none"])
            .count()
            != 1
    {
        return Err(invalid(
            "Colima start argv must explicitly freeze mount and port forwarding to none",
        ));
    }
    Ok(())
}

fn has_exact_guest_prefix(
    command: &NativeVmCommandSpec,
    contract: &NativeVmCollectionContract,
) -> bool {
    let Some(env) = contract.collector_tools.get("env") else {
        return false;
    };
    let Ok(path_environment) = guest_path_environment(contract) else {
        return false;
    };
    let expected = [
        "--profile",
        contract.profile_name.as_str(),
        "ssh",
        "--",
        env.path.as_str(),
        "-i",
        path_environment.as_str(),
        "LC_ALL=C",
        "LANG=C",
        "HOME=/var/empty",
    ];
    command
        .argv
        .iter()
        .take(expected.len())
        .map(String::as_str)
        .eq(expected)
        && command.argv.len() > expected.len()
}

fn contains_forbidden_runtime_word(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [
        "api.anthropic.com",
        "api.openai.com",
        "auth.json",
        "credentials.json",
        "ssh_auth_sock=",
        "anthropic_api_key",
        "openai_api_key",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn command_contains_forbidden_surface(command: &NativeVmCommandSpec) -> bool {
    contains_forbidden_runtime_word(&command.executable)
        || path_contains_authentication_component(Path::new(&command.executable))
        || command
            .argv
            .iter()
            .chain(command.environment.values())
            .any(|value| contains_forbidden_runtime_word(value))
        || command.environment.contains_key("SSH_AUTH_SOCK")
        || command.environment.keys().any(|key| {
            matches!(
                key.to_ascii_uppercase().as_str(),
                "ANTHROPIC_API_KEY" | "OPENAI_API_KEY" | "CODEX_ACCESS_TOKEN"
            )
        })
}

/// Create the local no-replay boundary. This writes only owner-private collection metadata; it
/// does not invoke Colima, access authentication state, or contact a provider.
pub fn prepare_native_vm_collection(
    contract: &NativeVmCollectionContract,
    expected_contract_sha256: &str,
    run_root: &Path,
) -> NativeVmResult<PreparedNativeVmCollection> {
    prepare_native_vm_collection_inner(
        contract,
        expected_contract_sha256,
        run_root,
        current_unix_ms()?,
    )
}

#[cfg(test)]
pub fn prepare_native_vm_collection_at(
    contract: &NativeVmCollectionContract,
    expected_contract_sha256: &str,
    run_root: &Path,
    now_unix_ms: u64,
) -> NativeVmResult<PreparedNativeVmCollection> {
    prepare_native_vm_collection_inner(contract, expected_contract_sha256, run_root, now_unix_ms)
}

fn prepare_native_vm_collection_inner(
    contract: &NativeVmCollectionContract,
    expected_contract_sha256: &str,
    run_root: &Path,
    now_unix_ms: u64,
) -> NativeVmResult<PreparedNativeVmCollection> {
    validate_native_vm_contract(contract)?;
    validate_expected_contract_digest(contract, expected_contract_sha256)?;
    validate_freshness(
        contract.created_unix_ms,
        contract.expires_unix_ms,
        now_unix_ms,
    )?;
    #[cfg(not(unix))]
    {
        let _ = run_root;
        return Err(invalid(
            "native VM collection requires Unix filesystem semantics",
        ));
    }
    #[cfg(unix)]
    {
        let parent = run_root
            .parent()
            .ok_or_else(|| invalid("collection root has no parent"))?;
        require_canonical_owner_private_directory(parent, "collection parent")?;
        validate_fresh_isolated_host_home(contract)?;
        let profile_directory = Path::new(&contract.colima_home).join(&contract.profile_name);
        if !fs::symlink_metadata(&profile_directory)
            .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound)
        {
            return Err(invalid(
                "fresh Colima profile path already exists before collection",
            ));
        }
        create_new_private_directory(run_root)?;
        create_new_private_directory(&run_root.join("raw"))?;

        let contract_path = run_root.join("contract.json");
        write_private_json_with_digest(&contract_path, contract)?;
        let contract_sha256 = native_vm_contract_sha256(contract)?;
        let manifest_path = run_root.join("bundle-manifest.json");
        write_private_json_with_digest(&manifest_path, &contract.bundle_manifest)?;
        let intent = NativeVmCollectionIntent {
            schema_version: NATIVE_VM_SCHEMA_VERSION,
            collection_id: contract.collection_id.clone(),
            contract_sha256: contract_sha256.clone(),
            command_plan_sha256: contract.command_plan_sha256.clone(),
            bundle_manifest_sha256: contract.bundle_manifest.manifest_sha256.clone(),
            bundle_archive_sha256: contract.bundle_archive.sha256.clone(),
            created_unix_ms: now_unix_ms,
            expires_unix_ms: contract.expires_unix_ms,
            state: "prepared".to_string(),
            profile_path_absent_before_start: true,
            isolated_host_home_empty_before_start: true,
            no_replay: true,
        };
        let intent_path = run_root.join("intent.json");
        let intent_sha256 = write_private_json_with_digest(&intent_path, &intent)?;
        sync_directory(run_root)?;
        sync_directory(parent)?;
        Ok(PreparedNativeVmCollection {
            run_root: run_root.display().to_string(),
            intent_path: intent_path.display().to_string(),
            intent_sha256,
            contract_sha256,
            command_count: contract.command_plan.len(),
        })
    }
}

/// Execute the provider-free collection plan against a new named VM. Callers must freeze and
/// independently review the contract before invoking this function. A failed or interrupted run
/// intentionally leaves its intent and partial artifacts in place and cannot be replayed.
pub fn execute_native_vm_collection(
    contract: &NativeVmCollectionContract,
    expected_contract_sha256: &str,
    run_root: &Path,
) -> NativeVmResult<RunnerObservedNativeVmCollection> {
    let prepared = prepare_native_vm_collection(contract, expected_contract_sha256, run_root)?;
    validate_bound_file_against_pin(
        Path::new(&contract.colima.path),
        &contract.colima.sha256,
        None,
    )?;
    validate_bound_file_against_pin(
        Path::new(&contract.limactl.path),
        &contract.limactl.sha256,
        None,
    )?;
    validate_bound_file_against_pin(
        Path::new(&contract.bundle_archive.path),
        &contract.bundle_archive.sha256,
        Some(contract.bundle_archive.size_bytes),
    )?;
    validate_ustar_archive(
        Path::new(&contract.bundle_archive.path),
        &contract.bundle_manifest,
    )?;
    let (canonical_run_root, run_root_handle, raw_root_handle) =
        open_collection_root_handles(run_root)?;
    let run_root_metadata = run_root_handle.metadata()?;
    let raw_root_metadata = raw_root_handle.metadata()?;
    let started_unix_ms = current_unix_ms()?;
    let mut receipts = Vec::with_capacity(contract.command_plan.len());
    for spec in &contract.command_plan {
        let receipt = execute_one_collection_command(contract, run_root, spec)?;
        validate_immediate_command_result(contract, spec, &receipts, &receipt, run_root)?;
        receipts.push(receipt);
    }
    sync_directory(&run_root.join("raw"))?;
    let profile_config_sha256 = snapshot_profile_config(contract, run_root)?;
    let command_transcript_sha256 = serialized_digest(&receipts)?;
    let (guest_boot_id_sha256, live_identity_sha256) =
        derive_live_identity(contract, &receipts, run_root)?;
    let receipt = NativeVmCollectionReceipt {
        schema_version: NATIVE_VM_SCHEMA_VERSION,
        collection_id: contract.collection_id.clone(),
        contract_sha256: prepared.contract_sha256.clone(),
        intent_sha256: prepared.intent_sha256,
        command_plan_sha256: contract.command_plan_sha256.clone(),
        bundle_manifest_sha256: contract.bundle_manifest.manifest_sha256.clone(),
        bundle_archive_sha256: contract.bundle_archive.sha256.clone(),
        profile_config_sha256,
        canonical_run_root: slash_path(&canonical_run_root)?,
        run_root_device: run_root_metadata.dev(),
        run_root_inode: run_root_metadata.ino(),
        raw_root_device: raw_root_metadata.dev(),
        raw_root_inode: raw_root_metadata.ino(),
        profile_name: contract.profile_name.clone(),
        guest_boot_id_sha256: guest_boot_id_sha256.clone(),
        live_identity_sha256: live_identity_sha256.clone(),
        command_transcript_sha256: command_transcript_sha256.clone(),
        started_unix_ms,
        completed_unix_ms: current_unix_ms()?,
        collector_pid: std::process::id(),
        state: "complete".to_string(),
        no_replay: true,
        commands: receipts,
    };
    let receipt_sha256 = write_private_json_with_digest(&run_root.join("receipt.json"), &receipt)?;
    sync_directory(run_root)?;
    let witness = NativeVmExecutionWitness {
        _seal: NativeVmExecutionSeal,
        run_root_handle,
        raw_root_handle,
        canonical_run_root,
        contract_sha256: prepared.contract_sha256,
        receipt_sha256,
        command_transcript_sha256,
        profile_name: contract.profile_name.clone(),
        guest_boot_id_sha256,
        live_identity_sha256,
    };
    let structural_audit = audit_native_vm_collection_inner(
        contract,
        expected_contract_sha256,
        run_root,
        current_unix_ms()?,
    )?;
    if !structural_audit.artifacts_consistent {
        return Err(invalid(format!(
            "runner artifacts failed structural audit: {}",
            structural_audit.failures.join("; ")
        )));
    }
    validate_execution_witness(&witness, contract, &receipt, run_root)?;
    Ok(RunnerObservedNativeVmCollection {
        structural_audit,
        witness,
    })
}

#[cfg(unix)]
fn open_collection_root_handles(run_root: &Path) -> NativeVmResult<(PathBuf, File, File)> {
    let canonical_run_root = canonical_secure_directory(run_root, "collection root")?;
    require_owner_private_directory(&canonical_run_root, "collection root")?;
    let raw_root = canonical_run_root.join("raw");
    require_canonical_owner_private_directory(&raw_root, "raw artifact directory")?;
    let run_handle = open_private_directory_handle(&canonical_run_root, "collection root")?;
    let raw_handle = open_private_directory_handle(&raw_root, "raw artifact directory")?;
    Ok((canonical_run_root, run_handle, raw_handle))
}

#[cfg(not(unix))]
fn open_collection_root_handles(_run_root: &Path) -> NativeVmResult<(PathBuf, File, File)> {
    Err(invalid(
        "native VM execution requires Unix directory binding",
    ))
}

#[cfg(unix)]
fn open_private_directory_handle(path: &Path, label: &str) -> NativeVmResult<File> {
    validate_existing_input_path_before_open(path, label)?;
    require_owner_private_directory(path, label)?;
    let before = fs::symlink_metadata(path)?;
    let mut options = OpenOptions::new();
    options
        .read(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_DIRECTORY);
    let handle = options.open(path)?;
    let after = fs::symlink_metadata(path)?;
    let bound = handle.metadata()?;
    if before.dev() != after.dev()
        || before.ino() != after.ino()
        || before.dev() != bound.dev()
        || before.ino() != bound.ino()
    {
        return Err(invalid(format!("{label} changed during handle binding")));
    }
    Ok(handle)
}

fn derive_live_identity(
    contract: &NativeVmCollectionContract,
    receipts: &[NativeVmCommandReceipt],
    run_root: &Path,
) -> NativeVmResult<(String, String)> {
    let output = |label: &str| {
        let receipt = receipts
            .iter()
            .find(|receipt| receipt.label == label)
            .ok_or_else(|| invalid(format!("missing live identity output: {label}")))?;
        read_bounded_secure_file(
            &run_root.join(&receipt.stdout_path),
            MAX_COMMAND_OUTPUT_BYTES,
        )
    };
    let status_before = parse_validated_colima_status(contract, &output("host_colima_status")?)?;
    let status_after =
        parse_validated_colima_status(contract, &output("host_colima_status_after")?)?;
    if status_before != status_after {
        return Err(invalid("Colima profile identity changed during collection"));
    }
    let boot_before = output("guest_boot_id")?;
    let boot_after = output("guest_boot_id_after")?;
    let boot_before = validate_guest_boot_id(&boot_before)?;
    let boot_after = validate_guest_boot_id(&boot_after)?;
    if boot_before != boot_after {
        return Err(invalid("guest boot identity changed during collection"));
    }
    let guest_boot_id_sha256 = sha256_bytes(boot_before.as_bytes());
    let live_identity_sha256 = live_identity_digest(
        contract.profile_name.as_str(),
        &status_before,
        guest_boot_id_sha256.as_str(),
    );
    Ok((guest_boot_id_sha256, live_identity_sha256))
}

fn live_identity_digest(
    profile_name: &str,
    status: &ColimaStatus,
    guest_boot_id_sha256: &str,
) -> String {
    sha256_bytes(
        format!(
            "profile={profile_name}\ndisplay_name={}\ndriver={}\narch={}\nruntime={}\ndocker_socket={}\nkubernetes={}\ndisk={}\nguest_boot_id_sha256={guest_boot_id_sha256}\n",
            status.display_name,
            status.driver,
            status.arch,
            status.runtime,
            status.docker_socket,
            status.kubernetes,
            status.disk,
        )
        .as_bytes(),
    )
}

#[cfg(unix)]
fn validate_execution_witness(
    witness: &NativeVmExecutionWitness,
    contract: &NativeVmCollectionContract,
    receipt: &NativeVmCollectionReceipt,
    run_root: &Path,
) -> NativeVmResult<()> {
    let canonical_run_root = fs::canonicalize(run_root)?;
    let run_path_metadata = fs::symlink_metadata(&canonical_run_root)?;
    let raw_path_metadata = fs::symlink_metadata(canonical_run_root.join("raw"))?;
    let run_handle_metadata = witness.run_root_handle.metadata()?;
    let raw_handle_metadata = witness.raw_root_handle.metadata()?;
    let receipt_bytes = read_bounded_secure_file(&run_root.join("receipt.json"), 16 * 1024 * 1024)?;
    let (boot_sha256, live_identity_sha256) =
        derive_live_identity(contract, &receipt.commands, run_root)?;
    if witness.canonical_run_root != canonical_run_root
        || run_path_metadata.dev() != run_handle_metadata.dev()
        || run_path_metadata.ino() != run_handle_metadata.ino()
        || raw_path_metadata.dev() != raw_handle_metadata.dev()
        || raw_path_metadata.ino() != raw_handle_metadata.ino()
        || witness.contract_sha256 != native_vm_contract_sha256(contract)?
        || witness.receipt_sha256 != sha256_bytes(&receipt_bytes)
        || witness.command_transcript_sha256 != serialized_digest(&receipt.commands)?
        || witness.profile_name != contract.profile_name
        || witness.guest_boot_id_sha256 != boot_sha256
        || witness.live_identity_sha256 != live_identity_sha256
    {
        return Err(invalid(
            "process-local execution witness lost its exact run/profile/boot binding",
        ));
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_execution_witness(
    _witness: &NativeVmExecutionWitness,
    _contract: &NativeVmCollectionContract,
    _receipt: &NativeVmCollectionReceipt,
    _run_root: &Path,
) -> NativeVmResult<()> {
    Err(invalid(
        "native VM execution witness requires Unix semantics",
    ))
}

fn execute_one_collection_command(
    contract: &NativeVmCollectionContract,
    run_root: &Path,
    spec: &NativeVmCommandSpec,
) -> NativeVmResult<NativeVmCommandReceipt> {
    let executable_sha256 = if spec.executable == contract.colima.path {
        &contract.colima.sha256
    } else if spec.executable == contract.limactl.path {
        &contract.limactl.sha256
    } else {
        return Err(invalid(
            "command executable is outside the frozen host runtime set",
        ));
    };
    validate_bound_file_against_pin(Path::new(&spec.executable), executable_sha256, None)?;
    let raw_root = run_root.join("raw");
    let stem = format!("{:03}-{}", spec.ordinal, spec.label);
    let stdout_path = raw_root.join(format!("{stem}.stdout"));
    let stderr_path = raw_root.join(format!("{stem}.stderr"));
    let stdin = match spec.stdin {
        NativeVmCommandInput::None => None,
        NativeVmCommandInput::BundleArchive => Some(open_pinned_archive(contract)?),
    };
    let outcome = run_bounded_command(spec, stdin, &stdout_path, &stderr_path)?;
    let exit_code = outcome.status.code().unwrap_or(-1);
    let receipt = NativeVmCommandReceipt {
        ordinal: spec.ordinal,
        label: spec.label.clone(),
        command_sha256: spec.command_sha256.clone(),
        expected_exit_code: spec.expected_exit_code,
        exit_code,
        started_unix_ms: outcome.started_unix_ms,
        completed_unix_ms: outcome.completed_unix_ms,
        stdout_path: format!("raw/{stem}.stdout"),
        stdout_sha256: sha256_secure_file(&stdout_path, spec.max_output_bytes)?,
        stdout_bytes: outcome.stdout_bytes,
        stderr_path: format!("raw/{stem}.stderr"),
        stderr_sha256: sha256_secure_file(&stderr_path, spec.max_output_bytes)?,
        stderr_bytes: outcome.stderr_bytes,
        stdin_sha256: outcome.stdin_sha256,
        stdin_bytes: outcome.stdin_bytes,
        timed_out: outcome.timed_out,
        output_overflow: outcome.stdout_overflow || outcome.stderr_overflow,
    };
    if receipt.timed_out {
        return Err(invalid(format!("command timed out: {}", spec.label)));
    }
    if receipt.output_overflow {
        return Err(invalid(format!(
            "command output exceeded its bound: {}",
            spec.label
        )));
    }
    if receipt.exit_code != spec.expected_exit_code {
        return Err(invalid(format!(
            "command exit mismatch: {} expected {} got {}",
            spec.label, spec.expected_exit_code, receipt.exit_code
        )));
    }
    if spec.stdin == NativeVmCommandInput::BundleArchive
        && (receipt.stdin_sha256.as_deref() != Some(contract.bundle_archive.sha256.as_str())
            || receipt.stdin_bytes != Some(contract.bundle_archive.size_bytes))
    {
        return Err(invalid(
            "SSH archive stream was not the exact fully observed pinned input",
        ));
    }
    Ok(receipt)
}

fn run_bounded_command(
    spec: &NativeVmCommandSpec,
    stdin: Option<File>,
    stdout_path: &Path,
    stderr_path: &Path,
) -> NativeVmResult<BoundedCommandOutcome> {
    run_bounded_command_inner(
        spec,
        stdin,
        stdout_path,
        stderr_path,
        RunnerPostSpawnFault::None,
        None,
    )
}

fn run_bounded_command_inner(
    spec: &NativeVmCommandSpec,
    stdin: Option<File>,
    stdout_path: &Path,
    stderr_path: &Path,
    fault: RunnerPostSpawnFault,
    child_pid_sink: Option<&std::sync::atomic::AtomicU32>,
) -> NativeVmResult<BoundedCommandOutcome> {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    #[cfg(not(test))]
    let _ = fault;

    let stdout_file = create_new_private_file(stdout_path)?;
    let stderr_file = create_new_private_file(stderr_path)?;
    let mut command = Command::new(&spec.executable);
    command
        .args(&spec.argv)
        .env_clear()
        .envs(&spec.environment)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    command.process_group(0);
    command.stdin(if stdin.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    });
    let started_unix_ms = current_unix_ms()?;
    let mut child = DirectChildProcessGroupGuard::new(command.spawn()?);
    if let Some(sink) = child_pid_sink {
        sink.store(child.id(), Ordering::SeqCst);
    }
    #[cfg(test)]
    {
        child.inject_try_wait_failure = fault == RunnerPostSpawnFault::TryWait;
        child.inject_wait_failure = fault == RunnerPostSpawnFault::Wait;
    }

    let result = (|| -> NativeVmResult<BoundedCommandOutcome> {
        #[cfg(test)]
        if fault == RunnerPostSpawnFault::PrimaryAndCleanup {
            child.inject_cleanup_failure = true;
            return Err(invalid("injected primary post-spawn failure"));
        }
        #[cfg(test)]
        if fault == RunnerPostSpawnFault::PanicUnwind {
            panic!("injected post-spawn panic");
        }
        #[cfg(test)]
        if fault == RunnerPostSpawnFault::PipeAcquisition {
            return Err(invalid("injected pipe-acquisition failure"));
        }

        let child_stdin = match stdin {
            Some(file) => Some((
                file,
                child
                    .child_mut()
                    .stdin
                    .take()
                    .ok_or_else(|| invalid("collector child stdin was not piped"))?,
            )),
            None => None,
        };
        let child_stdout = child
            .child_mut()
            .stdout
            .take()
            .ok_or_else(|| invalid("collector child stdout was not piped"))?;
        let child_stderr = child
            .child_mut()
            .stderr
            .take()
            .ok_or_else(|| invalid("collector child stderr was not piped"))?;

        #[cfg(test)]
        if fault == RunnerPostSpawnFault::ThreadSetup {
            return Err(invalid("injected collector-thread setup failure"));
        }

        let stdin_thread = match child_stdin {
            Some((file, child_stdin)) => Some(
                thread::Builder::new()
                    .name("native-vm-stdin".to_string())
                    .spawn(move || pump_archive(file, child_stdin))?,
            ),
            None => None,
        };
        let stdout_overflow = Arc::new(AtomicBool::new(false));
        let stderr_overflow = Arc::new(AtomicBool::new(false));
        let stdout_flag = Arc::clone(&stdout_overflow);
        let stderr_flag = Arc::clone(&stderr_overflow);
        let max_output = spec.max_output_bytes;
        #[cfg(test)]
        let inject_thread_join_panic = fault == RunnerPostSpawnFault::ThreadJoin;
        #[cfg(not(test))]
        let inject_thread_join_panic = false;
        let stdout_thread = thread::Builder::new()
            .name("native-vm-stdout".to_string())
            .spawn(move || {
                if inject_thread_join_panic {
                    panic!("injected stdout collector panic");
                }
                drain_bounded(child_stdout, stdout_file, max_output, &stdout_flag)
            })?;
        let stderr_thread = thread::Builder::new()
            .name("native-vm-stderr".to_string())
            .spawn(move || drain_bounded(child_stderr, stderr_file, max_output, &stderr_flag))?;
        let deadline = Instant::now() + Duration::from_millis(spec.timeout_ms);
        let mut timed_out = false;

        let status = loop {
            #[cfg(test)]
            if fault == RunnerPostSpawnFault::Wait {
                child.kill_exact_process_group()?;
                break child.wait()?;
            }
            if let Some(status) = child.try_wait()? {
                child.kill_exact_process_group()?;
                break status;
            }
            if Instant::now() >= deadline
                || stdout_overflow.load(Ordering::SeqCst)
                || stderr_overflow.load(Ordering::SeqCst)
            {
                timed_out = Instant::now() >= deadline;
                child.kill_exact_process_group()?;
                break child.wait()?;
            }
            thread::sleep(Duration::from_millis(10));
        };
        let stdout_bytes = stdout_thread
            .join()
            .map_err(|_| invalid("stdout collector thread panicked"))??;
        let stderr_bytes = stderr_thread
            .join()
            .map_err(|_| invalid("stderr collector thread panicked"))??;
        let (stdin_sha256, stdin_bytes) = match stdin_thread {
            Some(handle) => {
                let (digest, bytes) = handle
                    .join()
                    .map_err(|_| invalid("stdin collector thread panicked"))??;
                (Some(digest), Some(bytes))
            }
            None => (None, None),
        };
        Ok(BoundedCommandOutcome {
            status,
            started_unix_ms,
            completed_unix_ms: current_unix_ms()?,
            stdout_bytes,
            stderr_bytes,
            stdout_overflow: stdout_overflow.load(Ordering::SeqCst),
            stderr_overflow: stderr_overflow.load(Ordering::SeqCst),
            timed_out,
            stdin_sha256,
            stdin_bytes,
        })
    })();

    match result {
        Ok(outcome) => match child.disarm_after_confirmed_reap() {
            Ok(()) => Ok(outcome),
            Err(primary) => Err(combine_primary_and_cleanup_failure(&mut child, primary)),
        },
        Err(primary) => Err(combine_primary_and_cleanup_failure(&mut child, primary)),
    }
}

fn combine_primary_and_cleanup_failure(
    child: &mut DirectChildProcessGroupGuard,
    primary: NativeVmError,
) -> NativeVmError {
    match child.terminate_and_reap() {
        Ok(()) => primary,
        Err(cleanup) => invalid(format!(
            "primary post-spawn failure: {primary}; child cleanup failure: {cleanup}"
        )),
    }
}

#[cfg(test)]
pub fn exercise_process_group_guard_failure_for_test(
    fault: NativeVmRunnerTestFault,
    output_root: &Path,
    child_pid_sink: &std::sync::atomic::AtomicU32,
) -> NativeVmResult<()> {
    require_owner_private_directory(output_root, "test process-guard output root")?;
    let executable = if fault == NativeVmRunnerTestFault::ThreadJoin {
        "/usr/bin/true"
    } else {
        "/bin/sleep"
    };
    let argv = if fault == NativeVmRunnerTestFault::ThreadJoin {
        Vec::new()
    } else {
        vec!["30".to_string()]
    };
    let spec = NativeVmCommandSpec {
        ordinal: 0,
        label: "process_guard_test".to_string(),
        realm: NativeVmCommandRealm::Host,
        executable: executable.to_string(),
        argv,
        environment: BTreeMap::new(),
        stdin: NativeVmCommandInput::None,
        expected_exit_code: 0,
        timeout_ms: 60_000,
        max_output_bytes: 4_096,
        command_sha256: "0".repeat(SHA256_HEX_LEN),
    };
    run_bounded_command_inner(
        &spec,
        None,
        &output_root.join("stdout"),
        &output_root.join("stderr"),
        fault.into(),
        Some(child_pid_sink),
    )
    .map(|_| ())
}

fn pump_archive(mut input: File, mut output: impl Write) -> std::io::Result<(String, u64)> {
    let mut hasher = Sha256::new();
    let mut bytes = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = input.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        bytes = bytes
            .checked_add(read as u64)
            .ok_or_else(|| std::io::Error::other("archive stream size overflow"))?;
        if bytes > MAX_VM_BUNDLE_BYTES + (64 * 1024 * 1024) {
            return Err(std::io::Error::other(
                "archive stream exceeded the byte bound",
            ));
        }
        output.write_all(&buffer[..read])?;
        hasher.update(&buffer[..read]);
    }
    output.flush()?;
    drop(output);
    Ok((format!("{:x}", hasher.finalize()), bytes))
}

fn drain_bounded(
    mut input: impl Read,
    mut output: File,
    max_bytes: u64,
    overflow: &std::sync::atomic::AtomicBool,
) -> std::io::Result<u64> {
    use std::sync::atomic::Ordering;
    let mut retained = 0_u64;
    let mut buffer = [0_u8; 16 * 1024];
    loop {
        let read = input.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        let remaining = max_bytes.saturating_sub(retained);
        let keep = usize::try_from(remaining.min(read as u64)).unwrap_or(read);
        if keep > 0 {
            output.write_all(&buffer[..keep])?;
            retained += keep as u64;
        }
        if keep < read {
            overflow.store(true, Ordering::SeqCst);
        }
    }
    output.sync_all()?;
    Ok(retained)
}

fn open_pinned_archive(contract: &NativeVmCollectionContract) -> NativeVmResult<File> {
    #[cfg(not(unix))]
    {
        let _ = contract;
        Err(invalid("archive binding requires Unix metadata"))
    }
    #[cfg(unix)]
    {
        let path = Path::new(&contract.bundle_archive.path);
        let owner = unsafe { libc::geteuid() };
        let mut file =
            open_bound_regular_file(path, owner, Some(contract.bundle_archive.size_bytes))?;
        if sha256_reader(&mut file)? != contract.bundle_archive.sha256 {
            return Err(invalid("archive changed before the SSH stream"));
        }
        file.seek(SeekFrom::Start(0))?;
        Ok(file)
    }
}

fn validate_immediate_command_result(
    contract: &NativeVmCollectionContract,
    spec: &NativeVmCommandSpec,
    previous: &[NativeVmCommandReceipt],
    receipt: &NativeVmCommandReceipt,
    run_root: &Path,
) -> NativeVmResult<()> {
    let stdout =
        read_bounded_secure_file(&run_root.join(&receipt.stdout_path), spec.max_output_bytes)?;
    let stderr =
        read_bounded_secure_file(&run_root.join(&receipt.stderr_path), spec.max_output_bytes)?;
    if bytes_contain_forbidden_surface(&stdout) || bytes_contain_forbidden_surface(&stderr) {
        return Err(invalid(format!(
            "command output contains a forbidden provider/authentication surface: {}",
            spec.label
        )));
    }
    if spec.label != "host_colima_start" && !stderr.is_empty() {
        return Err(invalid(format!(
            "unexpected stderr from provider-free command: {}",
            spec.label
        )));
    }
    let require_empty = |label: &str| {
        if stdout.is_empty() {
            Ok(())
        } else {
            Err(invalid(format!(
                "provider-free command unexpectedly emitted output: {label}"
            )))
        }
    };
    match spec.label.as_str() {
        "host_colima_version" => validate_colima_version_output(contract, &stdout),
        "host_limactl_version" => validate_limactl_version_output(contract, &stdout),
        "host_profiles_before" => validate_profile_absent(contract, &stdout),
        "host_colima_start" => Ok(()),
        "host_colima_status" => validate_colima_status(contract, &stdout),
        "guest_uname" => validate_guest_uname(&stdout),
        "guest_boot_id" => validate_guest_boot_id(&stdout).map(|_| ()),
        "guest_root_filesystem_bytes" => validate_root_filesystem_size(contract, &stdout),
        "guest_os_release" => validate_guest_os_release(&stdout),
        "guest_findmnt" => validate_findmnt(&stdout).map(|_| ()),
        "guest_sockets" => validate_socket_inventory(&stdout),
        "guest_ssh_auth_sock"
        | "guest_users_path_absent"
        | "guest_volumes_path_absent"
        | "guest_apparmor_status"
        | "guest_teaching_user_absent"
        | "guest_evaluation_user_absent"
        | "guest_create_teaching_user"
        | "guest_create_evaluation_user"
        | "guest_secure_teaching_home"
        | "guest_secure_evaluation_home"
        | "guest_create_cross_uid_canary"
        | "guest_cross_uid_probe"
        | "guest_bundle_root_absent"
        | "guest_prepare_bundle_root"
        | "guest_transfer_bundle"
        | "guest_special_entries" => require_empty(&spec.label),
        "guest_binfmt_entries" => validate_binfmt(&stdout),
        "guest_apparmor_enabled" => {
            if trim_ascii(&stdout) == b"Y" {
                Ok(())
            } else {
                Err(invalid("AppArmor is not enabled in the guest kernel"))
            }
        }
        "guest_bwrap_profile" => validate_bwrap_profile(&stdout),
        "guest_teaching_home_stat" => {
            validate_guest_home(&stdout, &contract.teaching_user, contract.teaching_uid)
        }
        "guest_evaluation_home_stat" => {
            validate_guest_home(&stdout, &contract.evaluation_user, contract.evaluation_uid)
        }
        "guest_teaching_user" => {
            validate_guest_user(&stdout, &contract.teaching_user, contract.teaching_uid)
        }
        "guest_evaluation_user" => {
            validate_guest_user(&stdout, &contract.evaluation_user, contract.evaluation_uid)
        }
        "guest_teaching_identity" => validate_guest_identity_groups(&stdout, contract.teaching_uid),
        "guest_evaluation_identity" => {
            validate_guest_identity_groups(&stdout, contract.evaluation_uid)
        }
        "guest_bundle_filesystem" => {
            let findmnt = read_previous_command_output(previous, "guest_findmnt", run_root)?;
            let root_source = validate_findmnt(&findmnt)?;
            validate_bundle_filesystem(&stdout, &root_source)
        }
        "guest_bundle_inventory" => validate_bundle_inventory(contract, &stdout),
        "guest_bundle_directories" => validate_bundle_directories(contract, &stdout),
        "guest_bundle_rehash" => validate_guest_bundle_hashes(contract, &stdout),
        "guest_sha256sum_crosscheck" => {
            let pin = contract
                .collector_tools
                .get("sha256sum")
                .ok_or_else(|| invalid("missing sha256sum collector pin"))?;
            let (hash, path) = parse_openssl_sha256(&stdout)?;
            if hash == pin.sha256 && path == pin.path {
                Ok(())
            } else {
                Err(invalid("OpenSSL did not match the sha256sum pin"))
            }
        }
        "guest_openssl_crosscheck" => {
            let pin = contract
                .collector_tools
                .get("openssl")
                .ok_or_else(|| invalid("missing OpenSSL collector pin"))?;
            let (hash, path) = parse_sha256sum(&stdout)?;
            if hash == pin.sha256 && path == pin.path {
                Ok(())
            } else {
                Err(invalid("sha256sum did not match the OpenSSL pin"))
            }
        }
        "guest_boot_id_after" => {
            let before = read_previous_command_output(previous, "guest_boot_id", run_root)?;
            let before = validate_guest_boot_id(&before)?;
            let after = validate_guest_boot_id(&stdout)?;
            if before == after {
                Ok(())
            } else {
                Err(invalid("guest boot identity changed during collection"))
            }
        }
        "host_colima_status_after" => {
            let before = read_previous_command_output(previous, "host_colima_status", run_root)?;
            let before = parse_validated_colima_status(contract, &before)?;
            let after = parse_validated_colima_status(contract, &stdout)?;
            if before == after {
                Ok(())
            } else {
                Err(invalid(
                    "live Colima profile identity changed during collection",
                ))
            }
        }
        label => validate_one_executable_output(contract, label, &stdout),
    }
}

#[cfg(test)]
pub fn validate_native_vm_checkpoint_for_test(
    contract: &NativeVmCollectionContract,
    spec: &NativeVmCommandSpec,
    previous: &[NativeVmCommandReceipt],
    receipt: &NativeVmCommandReceipt,
    run_root: &Path,
) -> NativeVmResult<()> {
    validate_immediate_command_result(contract, spec, previous, receipt, run_root)
}

fn read_previous_command_output(
    previous: &[NativeVmCommandReceipt],
    label: &str,
    run_root: &Path,
) -> NativeVmResult<Vec<u8>> {
    let receipt = previous
        .iter()
        .find(|receipt| receipt.label == label)
        .ok_or_else(|| invalid(format!("required checkpoint has not run: {label}")))?;
    read_bounded_secure_file(
        &run_root.join(&receipt.stdout_path),
        MAX_COMMAND_OUTPUT_BYTES,
    )
}

fn validate_colima_version_output(
    contract: &NativeVmCollectionContract,
    output: &[u8],
) -> NativeVmResult<()> {
    if sha256_bytes(output) != contract.colima.version_output_sha256 {
        return Err(invalid("Colima version output digest drifted"));
    }
    let text =
        std::str::from_utf8(output).map_err(|_| invalid("Colima version output is not UTF-8"))?;
    if !text.contains(&contract.colima.version_marker)
        || !text.starts_with(&format!("colima version {REQUIRED_COLIMA_VERSION}\n"))
    {
        return Err(invalid("Colima version marker is not exact"));
    }
    Ok(())
}

fn validate_limactl_version_output(
    contract: &NativeVmCollectionContract,
    output: &[u8],
) -> NativeVmResult<()> {
    if sha256_bytes(output) != contract.limactl.version_output_sha256 {
        return Err(invalid("limactl version output digest drifted"));
    }
    let text =
        std::str::from_utf8(output).map_err(|_| invalid("limactl version output is not UTF-8"))?;
    if trim_ascii(text.as_bytes()) != contract.limactl.version_marker.as_bytes() {
        return Err(invalid("limactl version marker is not exact"));
    }
    Ok(())
}

fn validate_profile_absent(
    contract: &NativeVmCollectionContract,
    output: &[u8],
) -> NativeVmResult<()> {
    let text =
        std::str::from_utf8(output).map_err(|_| invalid("Colima profile listing is not UTF-8"))?;
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let value: Value = serde_json::from_str(line)?;
        let name = value
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid("Colima profile listing lacks a string name"))?;
        if name == contract.profile_name {
            return Err(invalid("fresh Colima profile already existed before start"));
        }
    }
    Ok(())
}

fn validate_colima_status(
    contract: &NativeVmCollectionContract,
    output: &[u8],
) -> NativeVmResult<()> {
    parse_validated_colima_status(contract, output).map(|_| ())
}

fn parse_validated_colima_status(
    contract: &NativeVmCollectionContract,
    output: &[u8],
) -> NativeVmResult<ColimaStatus> {
    let status: ColimaStatus = serde_json::from_slice(output)?;
    let min_disk_bytes = u64::from(contract.disk_gib) * 1024 * 1024 * 1024;
    if status.display_name != contract.profile_name
        || status.driver != "macOS Virtualization.Framework"
        || status.arch != contract.architecture
        || status.runtime != contract.runtime
        || status.kubernetes
        || !status.docker_socket.is_empty()
        || status.disk < min_disk_bytes
    {
        return Err(invalid(
            "live Colima status does not match the frozen VM shape",
        ));
    }
    Ok(status)
}

fn snapshot_profile_config(
    contract: &NativeVmCollectionContract,
    run_root: &Path,
) -> NativeVmResult<String> {
    #[cfg(not(unix))]
    {
        let _ = (contract, run_root);
        Err(invalid("profile config binding requires Unix metadata"))
    }
    #[cfg(unix)]
    {
        let source = Path::new(&contract.profile_config_path);
        validate_existing_input_path_before_open(source, "Colima profile config")?;
        let metadata = fs::symlink_metadata(source)?;
        if !metadata.is_file()
            || metadata.nlink() != 1
            || metadata.uid() != unsafe { libc::geteuid() }
            || metadata.permissions().mode() & 0o7000 != 0
            || metadata.permissions().mode() & 0o022 != 0
            || metadata.len() > 1024 * 1024
        {
            return Err(invalid(
                "Colima profile config is not an owner-bound regular file",
            ));
        }
        let mut input = open_bound_regular_file(source, metadata.uid(), Some(metadata.len()))?;
        let destination = run_root.join("profile-config.yaml");
        let mut output = create_new_private_file(&destination)?;
        std::io::copy(&mut input, &mut output)?;
        output.sync_all()?;
        let bytes = read_bounded_secure_file(&destination, 1024 * 1024)?;
        validate_profile_config_bytes(contract, &bytes)?;
        let digest = sha256_bytes(&bytes);
        write_private_text_create_new(
            &destination.with_file_name("profile-config.yaml.sha256"),
            &format!("{digest}\n"),
        )?;
        Ok(digest)
    }
}

fn validate_profile_config_bytes(
    contract: &NativeVmCollectionContract,
    bytes: &[u8],
) -> NativeVmResult<()> {
    let text =
        std::str::from_utf8(bytes).map_err(|_| invalid("Colima profile config is not UTF-8"))?;
    for (key, expected) in [
        ("arch", "aarch64".to_string()),
        ("cpu", contract.cpus.to_string()),
        ("memory", contract.memory_gib.to_string()),
        ("runtime", "containerd".to_string()),
        ("autoActivate", "false".to_string()),
        ("forwardAgent", "false".to_string()),
        ("vmType", "vz".to_string()),
        ("portForwarder", "none".to_string()),
        ("rosetta", "false".to_string()),
        ("binfmt", "false".to_string()),
        ("sshConfig", "false".to_string()),
        ("mountInotify", "false".to_string()),
        ("mounts", "[]".to_string()),
        ("provision", "null".to_string()),
        ("env", "{}".to_string()),
        ("hostname", contract.profile_name.clone()),
        ("disk", contract.disk_gib.to_string()),
        ("rootDisk", contract.root_disk_gib.to_string()),
    ] {
        if top_level_yaml_scalar(text, key)? != expected {
            return Err(invalid(format!(
                "Colima profile config has unexpected {key}"
            )));
        }
    }
    for (section, key, expected) in [
        ("network", "address", "false"),
        ("network", "mode", "shared"),
        ("network", "preferredRoute", "false"),
        ("network", "hostAddresses", "false"),
        ("kubernetes", "enabled", "false"),
    ] {
        if nested_yaml_scalar(text, section, key)? != expected {
            return Err(invalid(format!(
                "Colima profile config has unexpected {section}.{key}"
            )));
        }
    }
    Ok(())
}

fn nested_yaml_scalar(text: &str, section: &str, key: &str) -> NativeVmResult<String> {
    let section_prefix = format!("{section}:");
    let key_prefix = format!("  {key}:");
    let mut in_section = false;
    let mut values = Vec::new();
    for line in text.lines() {
        if !line.chars().next().is_some_and(char::is_whitespace) {
            in_section = line.trim_end() == section_prefix;
            continue;
        }
        if in_section {
            if let Some(value) = line.strip_prefix(&key_prefix) {
                values.push(value.trim().to_string());
            }
        }
    }
    if values.len() != 1 || values[0].is_empty() {
        return Err(invalid(format!(
            "Colima profile config must contain exactly one {section}.{key}"
        )));
    }
    Ok(values.remove(0))
}

fn top_level_yaml_scalar(text: &str, key: &str) -> NativeVmResult<String> {
    let prefix = format!("{key}:");
    let values = text
        .lines()
        .filter(|line| !line.chars().next().is_some_and(char::is_whitespace))
        .filter_map(|line| line.strip_prefix(&prefix))
        .map(|value| value.trim().to_string())
        .collect::<Vec<_>>();
    if values.len() != 1 || values[0].is_empty() {
        return Err(invalid(format!(
            "Colima profile config must contain exactly one top-level {key}"
        )));
    }
    Ok(values[0].clone())
}

/// Audit only runner-owned raw artifacts under the prepared collection root. There is no public
/// observation object whose booleans can substitute for command output.
pub fn audit_native_vm_collection(
    contract: &NativeVmCollectionContract,
    expected_contract_sha256: &str,
    run_root: &Path,
) -> NativeVmResult<NativeVmStructuralAudit> {
    audit_native_vm_collection_inner(
        contract,
        expected_contract_sha256,
        run_root,
        current_unix_ms()?,
    )
}

#[cfg(test)]
pub fn audit_native_vm_collection_at(
    contract: &NativeVmCollectionContract,
    expected_contract_sha256: &str,
    run_root: &Path,
    now_unix_ms: u64,
) -> NativeVmResult<NativeVmStructuralAudit> {
    audit_native_vm_collection_inner(contract, expected_contract_sha256, run_root, now_unix_ms)
}

fn audit_native_vm_collection_inner(
    contract: &NativeVmCollectionContract,
    expected_contract_sha256: &str,
    run_root: &Path,
    now_unix_ms: u64,
) -> NativeVmResult<NativeVmStructuralAudit> {
    let mut failures = Vec::new();
    if let Err(error) = validate_native_vm_contract(contract) {
        failures.push(error.to_string());
    }
    if let Err(error) = validate_expected_contract_digest(contract, expected_contract_sha256) {
        failures.push(error.to_string());
    }
    if let Err(error) = audit_collection_artifacts(contract, run_root, now_unix_ms) {
        failures.push(error.to_string());
    }
    Ok(NativeVmStructuralAudit {
        artifacts_consistent: failures.is_empty(),
        collection_id: contract.collection_id.clone(),
        claim: STRUCTURAL_AUDIT_CLAIM.to_string(),
        account_policy_nonclaim: ACCOUNT_POLICY_NONCLAIM.to_string(),
        command_count: contract.command_plan.len(),
        runtime_count: contract.required_runtimes.len(),
        collector_tool_count: contract.collector_tools.len(),
        failures,
        remaining_conditions_before_real_vm: vec![
            "independent adversarial review of this collector and exact frozen contract".to_string(),
            "a reviewed deterministic ustar bundle containing no credentials or host-specific state"
                .to_string(),
            "a preregistered Colima executable hash and guest base-image/package provenance"
                .to_string(),
            "a deterministic guest image whose collector utilities are regular single-link root-owned executables with no 0o7000 mode bits"
                .to_string(),
            "explicit operator selection of a fresh non-default profile name and collection root"
                .to_string(),
            "positive disk reserve and confirmation that repository target/debug is absent"
                .to_string(),
        ],
        remaining_conditions_before_provider_execution: vec![
            "offline artifacts are structurally auditable but can never recreate the process-local execution witness".to_string(),
            "shared runner/CLI integration with a separately reviewed provider authorization gate"
                .to_string(),
            "an independently anchored frozen contract plus guest image/package/tool provenance"
                .to_string(),
            "a fresh live status/config/boot/mount/socket/user/AppArmor/bundle recheck immediately before every provider phase"
                .to_string(),
            "subscription authentication performed inside the guest without importing host auth"
                .to_string(),
            "separate teaching and evaluation homes/state plus retention-gate evidence".to_string(),
            "provider trace, cost, identity, no-replay, scope-leakage, and stale-influence audits"
                .to_string(),
            "continued explicit non-attribution of account-scoped Claude managed policy".to_string(),
        ],
    })
}

fn validate_expected_contract_digest(
    contract: &NativeVmCollectionContract,
    expected_contract_sha256: &str,
) -> NativeVmResult<()> {
    if !is_sha256(expected_contract_sha256)
        || native_vm_contract_sha256(contract)? != expected_contract_sha256
    {
        return Err(invalid(
            "native VM contract does not match the preregistered digest",
        ));
    }
    Ok(())
}

fn audit_collection_artifacts(
    contract: &NativeVmCollectionContract,
    run_root: &Path,
    now_unix_ms: u64,
) -> NativeVmResult<()> {
    #[cfg(not(unix))]
    {
        let _ = (contract, run_root, now_unix_ms);
        return Err(invalid(
            "native VM collection audit requires Unix semantics",
        ));
    }
    #[cfg(unix)]
    {
        require_canonical_owner_private_directory(run_root, "collection root")?;
        require_canonical_owner_private_directory(&run_root.join("raw"), "raw artifact directory")?;
        validate_isolated_host_home_after_collection(contract)?;
        validate_closed_world_artifacts(contract, run_root)?;
        let (snapshot, _snapshot_sha256): (NativeVmCollectionContract, String) =
            read_private_json_with_digest(&run_root.join("contract.json"), 16 * 1024 * 1024)?;
        if &snapshot != contract {
            return Err(invalid(
                "contract snapshot does not match the audited contract",
            ));
        }
        let (manifest, _manifest_sha256): (NativeVmBundleManifest, String) =
            read_private_json_with_digest(
                &run_root.join("bundle-manifest.json"),
                16 * 1024 * 1024,
            )?;
        if manifest != contract.bundle_manifest {
            return Err(invalid("bundle manifest snapshot drifted"));
        }
        let (intent, intent_sha256): (NativeVmCollectionIntent, String) =
            read_private_json_with_digest(&run_root.join("intent.json"), 1024 * 1024)?;
        validate_intent(contract, &intent, now_unix_ms)?;
        let (receipt, _receipt_sha256): (NativeVmCollectionReceipt, String) =
            read_private_json_with_digest(&run_root.join("receipt.json"), 16 * 1024 * 1024)?;
        validate_receipt(
            contract,
            &intent,
            &receipt,
            &intent_sha256,
            run_root,
            now_unix_ms,
        )?;
        validate_command_receipts_and_raw(contract, &receipt, run_root)?;
        let (boot_sha256, live_identity_sha256) =
            derive_live_identity(contract, &receipt.commands, run_root)?;
        if receipt.guest_boot_id_sha256 != boot_sha256
            || receipt.live_identity_sha256 != live_identity_sha256
        {
            return Err(invalid(
                "receipt profile/boot identity binding does not match raw evidence",
            ));
        }
        let profile_snapshot =
            read_bounded_secure_file(&run_root.join("profile-config.yaml"), 1024 * 1024)?;
        validate_profile_config_bytes(contract, &profile_snapshot)?;
        if sha256_bytes(&profile_snapshot) != receipt.profile_config_sha256 {
            return Err(invalid(
                "profile config snapshot digest does not match receipt",
            ));
        }
        validate_digest_sidecar_for_bytes(
            &profile_snapshot,
            &run_root.join("profile-config.yaml.sha256"),
        )?;
        validate_bound_file_against_pin(
            Path::new(&contract.colima.path),
            &contract.colima.sha256,
            None,
        )?;
        validate_bound_file_against_pin(
            Path::new(&contract.limactl.path),
            &contract.limactl.sha256,
            None,
        )?;
        validate_bound_file_against_pin(
            Path::new(&contract.bundle_archive.path),
            &contract.bundle_archive.sha256,
            Some(contract.bundle_archive.size_bytes),
        )?;
        validate_ustar_archive(
            Path::new(&contract.bundle_archive.path),
            &contract.bundle_manifest,
        )?;
        validate_current_profile_config(contract, &profile_snapshot)?;
        validate_raw_guest_evidence(contract, &receipt, run_root)?;
        Ok(())
    }
}

#[cfg(unix)]
fn validate_isolated_host_home_after_collection(
    contract: &NativeVmCollectionContract,
) -> NativeVmResult<()> {
    let host_home = Path::new(&contract.host_home);
    let colima_home = Path::new(&contract.colima_home);
    require_canonical_owner_private_directory(host_home, "isolated host home")?;
    require_canonical_owner_private_directory(colima_home, "isolated Colima home")?;
    let names = directory_entry_names(host_home)?;
    if names != BTreeSet::from([".colima".to_string()]) {
        return Err(invalid(
            "isolated host home contains missing or unexpected top-level residue",
        ));
    }
    Ok(())
}

fn validate_intent(
    contract: &NativeVmCollectionContract,
    intent: &NativeVmCollectionIntent,
    now_unix_ms: u64,
) -> NativeVmResult<()> {
    validate_freshness(intent.created_unix_ms, intent.expires_unix_ms, now_unix_ms)?;
    if intent.schema_version != NATIVE_VM_SCHEMA_VERSION
        || intent.collection_id != contract.collection_id
        || intent.contract_sha256 != native_vm_contract_sha256(contract)?
        || intent.command_plan_sha256 != contract.command_plan_sha256
        || intent.bundle_manifest_sha256 != contract.bundle_manifest.manifest_sha256
        || intent.bundle_archive_sha256 != contract.bundle_archive.sha256
        || intent.state != "prepared"
        || !intent.profile_path_absent_before_start
        || !intent.isolated_host_home_empty_before_start
        || !intent.no_replay
        || intent.created_unix_ms < contract.created_unix_ms
        || intent.expires_unix_ms != contract.expires_unix_ms
    {
        return Err(invalid("collection intent is not exact or no-replay"));
    }
    Ok(())
}

fn validate_receipt(
    contract: &NativeVmCollectionContract,
    intent: &NativeVmCollectionIntent,
    receipt: &NativeVmCollectionReceipt,
    intent_sha256: &str,
    run_root: &Path,
    now_unix_ms: u64,
) -> NativeVmResult<()> {
    validate_freshness(intent.created_unix_ms, intent.expires_unix_ms, now_unix_ms)?;
    #[cfg(unix)]
    let binding_matches = {
        let canonical = fs::canonicalize(run_root)?;
        let run_metadata = fs::symlink_metadata(&canonical)?;
        let raw_metadata = fs::symlink_metadata(canonical.join("raw"))?;
        receipt.canonical_run_root == slash_path(&canonical)?
            && receipt.run_root_device == run_metadata.dev()
            && receipt.run_root_inode == run_metadata.ino()
            && receipt.raw_root_device == raw_metadata.dev()
            && receipt.raw_root_inode == raw_metadata.ino()
    };
    #[cfg(not(unix))]
    let binding_matches = false;
    if receipt.schema_version != NATIVE_VM_SCHEMA_VERSION
        || receipt.collection_id != contract.collection_id
        || receipt.contract_sha256 != native_vm_contract_sha256(contract)?
        || receipt.intent_sha256 != intent_sha256
        || receipt.command_plan_sha256 != contract.command_plan_sha256
        || receipt.bundle_manifest_sha256 != contract.bundle_manifest.manifest_sha256
        || receipt.bundle_archive_sha256 != contract.bundle_archive.sha256
        || !is_sha256(&receipt.profile_config_sha256)
        || !binding_matches
        || receipt.profile_name != contract.profile_name
        || !is_sha256(&receipt.guest_boot_id_sha256)
        || !is_sha256(&receipt.live_identity_sha256)
        || receipt.command_transcript_sha256 != serialized_digest(&receipt.commands)?
        || receipt.started_unix_ms < intent.created_unix_ms
        || receipt.completed_unix_ms < receipt.started_unix_ms
        || receipt.completed_unix_ms > now_unix_ms.saturating_add(MAX_CLOCK_SKEW_MS)
        || receipt.completed_unix_ms > intent.expires_unix_ms
        || receipt.collector_pid == 0
        || receipt.state != "complete"
        || !receipt.no_replay
        || receipt.commands.len() != contract.command_plan.len()
    {
        return Err(invalid(
            "collection receipt is stale, incomplete, or unbound",
        ));
    }
    Ok(())
}

fn validate_command_receipts_and_raw(
    contract: &NativeVmCollectionContract,
    receipt: &NativeVmCollectionReceipt,
    run_root: &Path,
) -> NativeVmResult<()> {
    let mut previous_completed = receipt.started_unix_ms;
    for (spec, observed) in contract.command_plan.iter().zip(&receipt.commands) {
        let stem = format!("{:03}-{}", spec.ordinal, spec.label);
        if observed.ordinal != spec.ordinal
            || observed.label != spec.label
            || observed.command_sha256 != spec.command_sha256
            || observed.expected_exit_code != spec.expected_exit_code
            || observed.exit_code != spec.expected_exit_code
            || observed.started_unix_ms < previous_completed
            || observed.completed_unix_ms < observed.started_unix_ms
            || observed.completed_unix_ms > receipt.completed_unix_ms
            || observed.stdout_path != format!("raw/{stem}.stdout")
            || observed.stderr_path != format!("raw/{stem}.stderr")
            || !is_sha256(&observed.stdout_sha256)
            || !is_sha256(&observed.stderr_sha256)
            || observed.stdout_bytes > spec.max_output_bytes
            || observed.stderr_bytes > spec.max_output_bytes
            || observed.timed_out
            || observed.output_overflow
        {
            return Err(invalid(format!(
                "command receipt is not exact: {}",
                spec.label
            )));
        }
        let expected_stdin = (spec.stdin == NativeVmCommandInput::BundleArchive)
            .then(|| contract.bundle_archive.sha256.clone());
        let expected_stdin_bytes = (spec.stdin == NativeVmCommandInput::BundleArchive)
            .then_some(contract.bundle_archive.size_bytes);
        if observed.stdin_sha256 != expected_stdin || observed.stdin_bytes != expected_stdin_bytes {
            return Err(invalid(format!(
                "command stdin binding drifted: {}",
                spec.label
            )));
        }
        let stdout_path = run_root.join(&observed.stdout_path);
        let stderr_path = run_root.join(&observed.stderr_path);
        let stdout = read_bounded_secure_file(&stdout_path, spec.max_output_bytes)?;
        let stderr = read_bounded_secure_file(&stderr_path, spec.max_output_bytes)?;
        if stdout.len() as u64 != observed.stdout_bytes
            || stderr.len() as u64 != observed.stderr_bytes
            || sha256_bytes(&stdout) != observed.stdout_sha256
            || sha256_bytes(&stderr) != observed.stderr_sha256
            || bytes_contain_forbidden_surface(&stdout)
            || bytes_contain_forbidden_surface(&stderr)
        {
            return Err(invalid(format!(
                "raw command artifact drifted: {}",
                spec.label
            )));
        }
        if spec.label != "host_colima_start" && !stderr.is_empty() {
            return Err(invalid(format!(
                "unexpected stderr in completed collection: {}",
                spec.label
            )));
        }
        previous_completed = observed.completed_unix_ms;
    }
    Ok(())
}

fn validate_raw_guest_evidence(
    contract: &NativeVmCollectionContract,
    receipt: &NativeVmCollectionReceipt,
    run_root: &Path,
) -> NativeVmResult<()> {
    let outputs = receipt
        .commands
        .iter()
        .map(|command| {
            Ok((
                command.label.as_str(),
                read_bounded_secure_file(
                    &run_root.join(&command.stdout_path),
                    MAX_COMMAND_OUTPUT_BYTES,
                )?,
            ))
        })
        .collect::<NativeVmResult<BTreeMap<_, _>>>()?;
    validate_colima_version_output(contract, required_output(&outputs, "host_colima_version")?)?;
    validate_limactl_version_output(contract, required_output(&outputs, "host_limactl_version")?)?;
    validate_profile_absent(contract, required_output(&outputs, "host_profiles_before")?)?;
    validate_colima_status(contract, required_output(&outputs, "host_colima_status")?)?;
    let status_before =
        parse_validated_colima_status(contract, required_output(&outputs, "host_colima_status")?)?;
    let status_after = parse_validated_colima_status(
        contract,
        required_output(&outputs, "host_colima_status_after")?,
    )?;
    if status_before != status_after {
        return Err(invalid(
            "live Colima profile identity changed during collection",
        ));
    }
    validate_guest_os(
        required_output(&outputs, "guest_uname")?,
        required_output(&outputs, "guest_os_release")?,
    )?;
    let boot_before = validate_guest_boot_id(required_output(&outputs, "guest_boot_id")?)?;
    let boot_after = validate_guest_boot_id(required_output(&outputs, "guest_boot_id_after")?)?;
    if boot_before != boot_after {
        return Err(invalid("guest boot identity changed during collection"));
    }
    let root_source = validate_findmnt(required_output(&outputs, "guest_findmnt")?)?;
    validate_guest_user(
        required_output(&outputs, "guest_teaching_user")?,
        &contract.teaching_user,
        contract.teaching_uid,
    )?;
    validate_guest_user(
        required_output(&outputs, "guest_evaluation_user")?,
        &contract.evaluation_user,
        contract.evaluation_uid,
    )?;
    validate_guest_identity_groups(
        required_output(&outputs, "guest_teaching_identity")?,
        contract.teaching_uid,
    )?;
    validate_guest_identity_groups(
        required_output(&outputs, "guest_evaluation_identity")?,
        contract.evaluation_uid,
    )?;
    validate_guest_home(
        required_output(&outputs, "guest_teaching_home_stat")?,
        &contract.teaching_user,
        contract.teaching_uid,
    )?;
    validate_guest_home(
        required_output(&outputs, "guest_evaluation_home_stat")?,
        &contract.evaluation_user,
        contract.evaluation_uid,
    )?;
    if !required_output(&outputs, "guest_teaching_user_absent")?.is_empty()
        || !required_output(&outputs, "guest_evaluation_user_absent")?.is_empty()
        || !required_output(&outputs, "guest_bundle_root_absent")?.is_empty()
        || !required_output(&outputs, "guest_ssh_auth_sock")?.is_empty()
        || !required_output(&outputs, "guest_users_path_absent")?.is_empty()
        || !required_output(&outputs, "guest_volumes_path_absent")?.is_empty()
        || !required_output(&outputs, "guest_apparmor_status")?.is_empty()
        || !required_output(&outputs, "guest_special_entries")?.is_empty()
    {
        return Err(invalid("an absence probe unexpectedly emitted output"));
    }
    validate_socket_inventory(required_output(&outputs, "guest_sockets")?)?;
    validate_binfmt(required_output(&outputs, "guest_binfmt_entries")?)?;
    if trim_ascii(required_output(&outputs, "guest_apparmor_enabled")?) != b"Y" {
        return Err(invalid("AppArmor is not enabled in the guest kernel"));
    }
    validate_bwrap_profile(required_output(&outputs, "guest_bwrap_profile")?)?;
    validate_bundle_filesystem(
        required_output(&outputs, "guest_bundle_filesystem")?,
        &root_source,
    )?;
    validate_root_filesystem_size(
        contract,
        required_output(&outputs, "guest_root_filesystem_bytes")?,
    )?;
    validate_bundle_inventory(
        contract,
        required_output(&outputs, "guest_bundle_inventory")?,
    )?;
    validate_bundle_directories(
        contract,
        required_output(&outputs, "guest_bundle_directories")?,
    )?;
    validate_executable_outputs(contract, &outputs, "runtime", &contract.required_runtimes)?;
    validate_executable_outputs(contract, &outputs, "tool", &contract.collector_tools)?;
    let sha256sum = contract
        .collector_tools
        .get("sha256sum")
        .ok_or_else(|| invalid("missing sha256sum collector pin"))?;
    let openssl = contract
        .collector_tools
        .get("openssl")
        .ok_or_else(|| invalid("missing openssl collector pin"))?;
    let (openssl_hash, openssl_path) =
        parse_sha256sum(required_output(&outputs, "guest_openssl_crosscheck")?)?;
    if openssl_hash != openssl.sha256 || openssl_path != openssl.path {
        return Err(invalid(
            "sha256sum did not independently match the OpenSSL pin",
        ));
    }
    let (sha_hash, sha_path) =
        parse_openssl_sha256(required_output(&outputs, "guest_sha256sum_crosscheck")?)?;
    if sha_hash != sha256sum.sha256 || sha_path != sha256sum.path {
        return Err(invalid(
            "OpenSSL did not independently match the sha256sum pin",
        ));
    }
    validate_guest_bundle_hashes(contract, required_output(&outputs, "guest_bundle_rehash")?)?;
    Ok(())
}

fn required_output<'a>(
    outputs: &'a BTreeMap<&str, Vec<u8>>,
    label: &str,
) -> NativeVmResult<&'a [u8]> {
    outputs
        .get(label)
        .map(Vec::as_slice)
        .ok_or_else(|| invalid(format!("missing raw command output: {label}")))
}

fn validate_guest_os(uname: &[u8], os_release: &[u8]) -> NativeVmResult<()> {
    validate_guest_uname(uname)?;
    validate_guest_os_release(os_release)
}

fn validate_guest_uname(uname: &[u8]) -> NativeVmResult<()> {
    if trim_ascii(uname) != b"Linux aarch64" {
        return Err(invalid("guest kernel/architecture is not Linux aarch64"));
    }
    Ok(())
}

fn validate_guest_os_release(os_release: &[u8]) -> NativeVmResult<()> {
    let text =
        std::str::from_utf8(os_release).map_err(|_| invalid("guest os-release is not UTF-8"))?;
    let fields = parse_key_value_lines(text)?;
    if fields.get("ID").map(String::as_str) != Some("ubuntu")
        || fields
            .get("VERSION_ID")
            .map(|value| value.trim_matches('"'))
            != Some("24.04")
    {
        return Err(invalid("guest distribution is not Ubuntu 24.04"));
    }
    Ok(())
}

fn validate_guest_boot_id(output: &[u8]) -> NativeVmResult<&str> {
    let value = std::str::from_utf8(trim_ascii(output))
        .map_err(|_| invalid("guest boot id is not UTF-8"))?;
    if value.len() != 36
        || value.bytes().enumerate().any(|(index, byte)| match index {
            8 | 13 | 18 | 23 => byte != b'-',
            _ => !byte.is_ascii_digit() && !(b'a'..=b'f').contains(&byte),
        })
        || value.bytes().all(|byte| byte == b'0' || byte == b'-')
    {
        return Err(invalid("guest boot id is not one canonical nonzero UUID"));
    }
    Ok(value)
}

fn parse_key_value_lines(text: &str) -> NativeVmResult<BTreeMap<String, String>> {
    let mut fields = BTreeMap::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| invalid("key/value output contains a malformed line"))?;
        if key.is_empty() || fields.insert(key.to_string(), value.to_string()).is_some() {
            return Err(invalid("key/value output contains an invalid duplicate"));
        }
    }
    Ok(fields)
}

fn validate_findmnt(output: &[u8]) -> NativeVmResult<String> {
    let value: Value = serde_json::from_slice(output)?;
    let filesystems = value
        .get("filesystems")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid("findmnt JSON has no filesystems array"))?;
    let mut mounts = Vec::new();
    collect_findmnt_rows(filesystems, &mut mounts)?;
    if mounts.is_empty() {
        return Err(invalid("findmnt returned no mounts"));
    }
    let roots = mounts
        .iter()
        .filter(|(_, target, _, _)| target == "/")
        .collect::<Vec<_>>();
    if roots.len() != 1 || roots[0].2 != "ext4" || !roots[0].0.starts_with("/dev/") {
        return Err(invalid(
            "guest root filesystem is not one VM block-device ext4 mount",
        ));
    }
    let mut targets = BTreeSet::new();
    for (source, target, filesystem, options) in &mounts {
        let lower = format!("{source}\n{target}\n{filesystem}\n{options}").to_ascii_lowercase();
        if !targets.insert(target.as_str())
            || matches!(
                filesystem.as_str(),
                "virtiofs"
                    | "9p"
                    | "fuse.sshfs"
                    | "sshfs"
                    | "nfs"
                    | "nfs4"
                    | "cifs"
                    | "smbfs"
                    | "afpfs"
                    | "vboxsf"
                    | "prl_fs"
            )
            || filesystem == "fuse"
            || filesystem.starts_with("fuse.")
            || target.starts_with("/Users")
            || target.starts_with("/Volumes")
            || source.contains("/Users/")
            || source.contains("/Volumes/")
            || target == "/host"
            || target.starts_with("/host/")
            || target == "/mnt/host"
            || target.starts_with("/mnt/host/")
            || lower.contains("docker.sock")
            || lower.contains("ssh_auth_sock")
        {
            return Err(invalid(
                "findmnt exposes a host mount, home, agent, or socket",
            ));
        }
    }
    Ok(roots[0].0.clone())
}

fn collect_findmnt_rows(
    rows: &[Value],
    mounts: &mut Vec<(String, String, String, String)>,
) -> NativeVmResult<()> {
    for row in rows {
        let object = row
            .as_object()
            .ok_or_else(|| invalid("findmnt row is not an object"))?;
        let source = required_json_string(object, "source")?;
        let target = required_json_string(object, "target")?;
        let filesystem = required_json_string(object, "fstype")?;
        let options = required_json_string(object, "options")?;
        mounts.push((source, target, filesystem, options));
        if let Some(children) = object.get("children") {
            collect_findmnt_rows(
                children
                    .as_array()
                    .ok_or_else(|| invalid("findmnt children is not an array"))?,
                mounts,
            )?;
        }
    }
    Ok(())
}

fn required_json_string(
    object: &serde_json::Map<String, Value>,
    key: &str,
) -> NativeVmResult<String> {
    object
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| invalid(format!("JSON field is missing or not a string: {key}")))
}

fn validate_guest_user(output: &[u8], name: &str, uid: u32) -> NativeVmResult<()> {
    let line = std::str::from_utf8(trim_ascii(output))
        .map_err(|_| invalid("getent output is not UTF-8"))?;
    let fields = line.split(':').collect::<Vec<_>>();
    if fields.len() != 7
        || fields[0] != name
        || fields[2] != uid.to_string()
        || fields[3] != uid.to_string()
        || fields[5] != format!("/home/{name}")
        || fields[6] != "/bin/bash"
        || uid == 0
    {
        return Err(invalid(format!("guest user is not exact: {name}")));
    }
    Ok(())
}

fn validate_guest_identity_groups(output: &[u8], uid: u32) -> NativeVmResult<()> {
    let groups = std::str::from_utf8(trim_ascii(output))
        .map_err(|_| invalid("guest group identity output is not UTF-8"))?;
    if groups != uid.to_string() {
        return Err(invalid(
            "guest identity has a supplementary or unexpected group",
        ));
    }
    Ok(())
}

fn validate_guest_home(output: &[u8], name: &str, uid: u32) -> NativeVmResult<()> {
    let text = std::str::from_utf8(trim_ascii(output))
        .map_err(|_| invalid("guest home stat output is not UTF-8"))?;
    let fields = text.split('|').collect::<Vec<_>>();
    let expected_uid = uid.to_string();
    let expected_home = format!("/home/{name}");
    if fields.len() != 5
        || fields[0] != "directory"
        || fields[1] != "700"
        || fields[2] != expected_uid
        || fields[3] != expected_uid
        || fields[4] != expected_home
    {
        return Err(invalid(format!(
            "guest home is not exact and owner-private: {name}"
        )));
    }
    Ok(())
}

fn validate_bundle_filesystem(output: &[u8], root_source: &str) -> NativeVmResult<()> {
    let text = std::str::from_utf8(trim_ascii(output))
        .map_err(|_| invalid("bundle filesystem output is not UTF-8"))?;
    let fields = text.split_ascii_whitespace().collect::<Vec<_>>();
    if fields != [root_source, "/", "ext4"] {
        return Err(invalid(
            "bundle root is not on the VM-owned ext4 root device",
        ));
    }
    Ok(())
}

fn validate_socket_inventory(output: &[u8]) -> NativeVmResult<()> {
    let text =
        std::str::from_utf8(output).map_err(|_| invalid("guest socket inventory is not UTF-8"))?;
    for line in text.lines().filter(|line| !line.is_empty()) {
        if !line.starts_with('/') || line.contains('\0') {
            return Err(invalid("guest socket inventory contains an invalid path"));
        }
        let lower = line.to_ascii_lowercase();
        if lower.contains("docker.sock")
            || lower.contains("ssh-agent")
            || lower.contains("ssh_auth_sock")
            || lower.contains("gpg-agent")
            || lower.contains("/tmp/ssh-")
            || lower.contains("/agent.")
            || lower.contains("/keyring/ssh")
            || lower.starts_with("/users/")
            || lower.starts_with("/volumes/")
            || lower.contains("/host/")
        {
            return Err(invalid(
                "guest exposes a forbidden host/agent/Docker socket",
            ));
        }
    }
    Ok(())
}

fn validate_binfmt(output: &[u8]) -> NativeVmResult<()> {
    let text = std::str::from_utf8(output).map_err(|_| invalid("binfmt inventory is not UTF-8"))?;
    let names = text
        .lines()
        .filter(|line| !line.is_empty())
        .collect::<BTreeSet<_>>();
    if names
        .iter()
        .any(|name| !matches!(*name, "register" | "status"))
    {
        return Err(invalid("foreign-architecture binfmt entry is registered"));
    }
    Ok(())
}

fn validate_bwrap_profile(output: &[u8]) -> NativeVmResult<()> {
    let text = std::str::from_utf8(trim_ascii(output))
        .map_err(|_| invalid("bubblewrap AppArmor profile output is not UTF-8"))?;
    if text != BWRAP_APPARMOR_PROFILE {
        return Err(invalid(
            "bubblewrap does not report the exact enforced AppArmor profile",
        ));
    }
    Ok(())
}

fn validate_bundle_inventory(
    contract: &NativeVmCollectionContract,
    output: &[u8],
) -> NativeVmResult<()> {
    let text =
        std::str::from_utf8(output).map_err(|_| invalid("guest bundle inventory is not UTF-8"))?;
    let expected = contract
        .bundle_manifest
        .entries
        .iter()
        .map(|entry| (entry.relative_path.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let mut observed = BTreeSet::new();
    for line in text.lines().filter(|line| !line.is_empty()) {
        let fields = line.split('|').collect::<Vec<_>>();
        if fields.len() != 6 {
            return Err(invalid("guest bundle inventory row is malformed"));
        }
        let relative = fields[5];
        validate_relative_bundle_path(Path::new(relative))?;
        let entry = expected
            .get(relative)
            .ok_or_else(|| invalid("guest bundle contains an unexpected file"))?;
        let mode = u32::from_str_radix(fields[0], 8)
            .map_err(|_| invalid("guest bundle mode is not octal"))?;
        let links = fields[1]
            .parse::<u64>()
            .map_err(|_| invalid("guest bundle link count is not numeric"))?;
        let owner = fields[2]
            .parse::<u32>()
            .map_err(|_| invalid("guest bundle owner is not numeric"))?;
        let group = fields[3]
            .parse::<u32>()
            .map_err(|_| invalid("guest bundle group is not numeric"))?;
        let size = fields[4]
            .parse::<u64>()
            .map_err(|_| invalid("guest bundle size is not numeric"))?;
        if !observed.insert(relative)
            || mode & 0o7000 != 0
            || mode != entry.mode
            || links != 1
            || owner != 0
            || group != 0
            || size != entry.size_bytes
        {
            return Err(invalid(
                "guest bundle metadata does not match the host manifest",
            ));
        }
    }
    if observed != expected.keys().copied().collect::<BTreeSet<_>>() {
        return Err(invalid("guest bundle inventory is incomplete"));
    }
    Ok(())
}

fn validate_root_filesystem_size(
    contract: &NativeVmCollectionContract,
    output: &[u8],
) -> NativeVmResult<()> {
    let text = std::str::from_utf8(trim_ascii(output))
        .map_err(|_| invalid("root filesystem size is not UTF-8"))?;
    let bytes = text
        .parse::<u64>()
        .map_err(|_| invalid("root filesystem size is not numeric"))?;
    let required = u64::from(contract.root_disk_gib) * 1024 * 1024 * 1024;
    if bytes < required {
        return Err(invalid(
            "VM-owned ext4 root is smaller than the frozen disk requirement",
        ));
    }
    Ok(())
}

fn validate_bundle_directories(
    contract: &NativeVmCollectionContract,
    output: &[u8],
) -> NativeVmResult<()> {
    let text = std::str::from_utf8(output)
        .map_err(|_| invalid("guest bundle directory inventory is not UTF-8"))?;
    let mut expected = BTreeSet::from([String::new()]);
    for entry in &contract.bundle_manifest.entries {
        let mut parent = Path::new(&entry.relative_path).parent();
        while let Some(path) = parent {
            if path.as_os_str().is_empty() {
                break;
            }
            expected.insert(slash_path(path)?);
            parent = path.parent();
        }
    }
    let mut observed = BTreeSet::new();
    for line in text.lines() {
        let fields = line.split('|').collect::<Vec<_>>();
        if fields.len() != 4 {
            return Err(invalid("guest bundle directory row is malformed"));
        }
        let mode = u32::from_str_radix(fields[0], 8)
            .map_err(|_| invalid("guest bundle directory mode is not octal"))?;
        let owner = fields[1]
            .parse::<u32>()
            .map_err(|_| invalid("guest bundle directory owner is not numeric"))?;
        let group = fields[2]
            .parse::<u32>()
            .map_err(|_| invalid("guest bundle directory group is not numeric"))?;
        let relative = fields[3];
        if !relative.is_empty() {
            validate_relative_bundle_path(Path::new(relative))?;
        }
        if mode & 0o7000 != 0
            || mode & 0o022 != 0
            || owner != 0
            || group != 0
            || !observed.insert(relative.to_string())
        {
            return Err(invalid("guest bundle directory metadata is unsafe"));
        }
    }
    if observed != expected {
        return Err(invalid(
            "guest bundle directory inventory is incomplete or has residue",
        ));
    }
    Ok(())
}

fn validate_executable_outputs(
    _contract: &NativeVmCollectionContract,
    outputs: &BTreeMap<&str, Vec<u8>>,
    namespace: &str,
    pins: &BTreeMap<String, NativeVmExecutablePin>,
) -> NativeVmResult<()> {
    for (name, pin) in pins {
        let prefix = format!("guest_{namespace}_{name}");
        let path = required_output(outputs, &format!("{prefix}_path"))?;
        if trim_ascii(path) != pin.path.as_bytes() {
            return Err(invalid(format!("{namespace} {name} resolved path drifted")));
        }
        validate_runtime_stat(
            required_output(outputs, &format!("{prefix}_stat"))?,
            &pin.path,
        )?;
        let (hash, hashed_path) =
            parse_sha256sum(required_output(outputs, &format!("{prefix}_hash"))?)?;
        if hash != pin.sha256 || hashed_path != pin.path {
            return Err(invalid(format!("{namespace} {name} hash drifted")));
        }
        let (alternate_hash, alternate_path) =
            parse_openssl_sha256(required_output(outputs, &format!("{prefix}_hash_alt"))?)?;
        if alternate_hash != pin.sha256 || alternate_path != pin.path {
            return Err(invalid(format!(
                "{namespace} {name} alternate hash drifted"
            )));
        }
        let version = required_output(outputs, &format!("{prefix}_version"))?;
        if sha256_bytes(version) != pin.version_output_sha256
            || !std::str::from_utf8(version)
                .map_err(|_| invalid("version output is not UTF-8"))?
                .contains(&pin.version_marker)
        {
            return Err(invalid(format!("{namespace} {name} version drifted")));
        }
    }
    Ok(())
}

fn validate_one_executable_output(
    contract: &NativeVmCollectionContract,
    label: &str,
    output: &[u8],
) -> NativeVmResult<()> {
    for (namespace, pins) in [
        ("runtime", &contract.required_runtimes),
        ("tool", &contract.collector_tools),
    ] {
        for (name, pin) in pins {
            let prefix = format!("guest_{namespace}_{name}");
            if label == format!("{prefix}_path") {
                return if trim_ascii(output) == pin.path.as_bytes() {
                    Ok(())
                } else {
                    Err(invalid(format!("{namespace} {name} resolved path drifted")))
                };
            }
            if label == format!("{prefix}_stat") {
                return validate_runtime_stat(output, &pin.path);
            }
            if label == format!("{prefix}_hash") {
                let (hash, path) = parse_sha256sum(output)?;
                return if hash == pin.sha256 && path == pin.path {
                    Ok(())
                } else {
                    Err(invalid(format!("{namespace} {name} hash drifted")))
                };
            }
            if label == format!("{prefix}_hash_alt") {
                let (hash, path) = parse_openssl_sha256(output)?;
                return if hash == pin.sha256 && path == pin.path {
                    Ok(())
                } else {
                    Err(invalid(format!(
                        "{namespace} {name} alternate hash drifted"
                    )))
                };
            }
            if label == format!("{prefix}_version") {
                return if sha256_bytes(output) == pin.version_output_sha256
                    && std::str::from_utf8(output)
                        .map_err(|_| invalid("version output is not UTF-8"))?
                        .contains(&pin.version_marker)
                {
                    Ok(())
                } else {
                    Err(invalid(format!("{namespace} {name} version drifted")))
                };
            }
        }
    }
    Err(invalid(format!(
        "command result has no exact semantic checkpoint: {label}"
    )))
}

fn validate_runtime_stat(output: &[u8], expected_path: &str) -> NativeVmResult<()> {
    let text = std::str::from_utf8(trim_ascii(output))
        .map_err(|_| invalid("runtime stat output is not UTF-8"))?;
    let fields = text.split('|').collect::<Vec<_>>();
    if fields.len() != 7
        || fields[0] != "regular file"
        || fields[1] != "1"
        || fields[6] != expected_path
    {
        return Err(invalid("runtime is not an exact regular single-link file"));
    }
    let mode =
        u32::from_str_radix(fields[2], 8).map_err(|_| invalid("runtime stat mode is not octal"))?;
    let owner = fields[3]
        .parse::<u32>()
        .map_err(|_| invalid("runtime stat owner is not numeric"))?;
    let group = fields[4]
        .parse::<u32>()
        .map_err(|_| invalid("runtime stat group is not numeric"))?;
    let links = fields[1]
        .parse::<u64>()
        .map_err(|_| invalid("runtime stat link count is not numeric"))?;
    let size = fields[5]
        .parse::<u64>()
        .map_err(|_| invalid("runtime stat size is not numeric"))?;
    if mode & 0o7000 != 0
        || mode & 0o022 != 0
        || mode & 0o111 == 0
        || owner != 0
        || group != 0
        || links != 1
        || size == 0
    {
        return Err(invalid(
            "runtime metadata is writable, unowned, linked, or empty",
        ));
    }
    Ok(())
}

fn validate_guest_bundle_hashes(
    contract: &NativeVmCollectionContract,
    output: &[u8],
) -> NativeVmResult<()> {
    let text = std::str::from_utf8(output)
        .map_err(|_| invalid("guest bundle hash output is not UTF-8"))?;
    let mut hashes = BTreeMap::new();
    for line in text.lines().filter(|line| !line.is_empty()) {
        let (hash, path) = parse_sha256sum(line.as_bytes())?;
        if hashes.insert(path, hash).is_some() {
            return Err(invalid(
                "guest bundle hash output contains a duplicate path",
            ));
        }
    }
    let expected = contract
        .bundle_manifest
        .entries
        .iter()
        .map(|entry| {
            (
                format!("{}/{}", contract.guest_bundle_root, entry.relative_path),
                entry.sha256.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    if hashes != expected {
        return Err(invalid(
            "guest rehash does not exactly match the host manifest",
        ));
    }
    Ok(())
}

fn parse_sha256sum(output: &[u8]) -> NativeVmResult<(String, String)> {
    let line = std::str::from_utf8(trim_ascii(output))
        .map_err(|_| invalid("sha256sum output is not UTF-8"))?;
    let (hash, path) = line
        .split_once("  ")
        .ok_or_else(|| invalid("sha256sum output is malformed"))?;
    if !is_sha256(hash) || path.is_empty() || path.contains('\n') {
        return Err(invalid("sha256sum output is invalid"));
    }
    Ok((hash.to_string(), path.to_string()))
}

fn parse_openssl_sha256(output: &[u8]) -> NativeVmResult<(String, String)> {
    let line = std::str::from_utf8(trim_ascii(output))
        .map_err(|_| invalid("OpenSSL digest output is not UTF-8"))?;
    let (prefix, hash) = line
        .rsplit_once("= ")
        .ok_or_else(|| invalid("OpenSSL digest output is malformed"))?;
    let path = prefix
        .strip_prefix("SHA2-256(")
        .and_then(|value| value.strip_suffix(')'))
        .ok_or_else(|| invalid("OpenSSL digest output has the wrong algorithm grammar"))?;
    if !is_sha256(hash) || path.is_empty() || path.contains('\n') {
        return Err(invalid("OpenSSL digest output is invalid"));
    }
    Ok((hash.to_string(), path.to_string()))
}

fn validate_current_profile_config(
    contract: &NativeVmCollectionContract,
    expected: &[u8],
) -> NativeVmResult<()> {
    #[cfg(not(unix))]
    {
        let _ = (contract, expected);
        Err(invalid("profile config binding requires Unix metadata"))
    }
    #[cfg(unix)]
    {
        let path = Path::new(&contract.profile_config_path);
        validate_existing_input_path_before_open(path, "live Colima profile config")?;
        let metadata = fs::symlink_metadata(path)?;
        if !metadata.is_file()
            || metadata.nlink() != 1
            || metadata.uid() != unsafe { libc::geteuid() }
            || metadata.permissions().mode() & 0o7000 != 0
            || metadata.permissions().mode() & 0o022 != 0
            || metadata.len() != expected.len() as u64
        {
            return Err(invalid("live Colima profile config metadata drifted"));
        }
        let observed = read_bounded_bound_file(path, metadata.uid(), 1024 * 1024)?;
        if observed != expected {
            return Err(invalid("live Colima profile config bytes drifted"));
        }
        Ok(())
    }
}

fn validate_closed_world_artifacts(
    contract: &NativeVmCollectionContract,
    run_root: &Path,
) -> NativeVmResult<()> {
    let expected_top = BTreeSet::from([
        "bundle-manifest.json".to_string(),
        "bundle-manifest.json.sha256".to_string(),
        "contract.json".to_string(),
        "contract.json.sha256".to_string(),
        "intent.json".to_string(),
        "intent.json.sha256".to_string(),
        "profile-config.yaml".to_string(),
        "profile-config.yaml.sha256".to_string(),
        "raw".to_string(),
        "receipt.json".to_string(),
        "receipt.json.sha256".to_string(),
    ]);
    let observed_top = directory_entry_names(run_root)?;
    if observed_top != expected_top {
        return Err(invalid("collection root contains missing or extra residue"));
    }
    let expected_raw = contract
        .command_plan
        .iter()
        .flat_map(|command| {
            let stem = format!("{:03}-{}", command.ordinal, command.label);
            [format!("{stem}.stdout"), format!("{stem}.stderr")]
        })
        .collect::<BTreeSet<_>>();
    if directory_entry_names(&run_root.join("raw"))? != expected_raw {
        return Err(invalid(
            "raw artifact directory contains missing or extra residue",
        ));
    }
    Ok(())
}

fn directory_entry_names(directory: &Path) -> NativeVmResult<BTreeSet<String>> {
    #[cfg(unix)]
    canonical_secure_directory(directory, "directory inventory root")?;
    fs::read_dir(directory)?
        .map(|entry| {
            let entry = entry?;
            entry
                .file_name()
                .into_string()
                .map_err(|_| invalid("artifact name is not UTF-8"))
        })
        .collect()
}

fn validate_freshness(created: u64, expires: u64, now: u64) -> NativeVmResult<()> {
    if created == 0
        || expires <= created
        || expires - created > MAX_COLLECTION_AGE_MS
        || now.saturating_add(MAX_CLOCK_SKEW_MS) < created
        || now > expires
    {
        return Err(invalid(
            "native VM evidence is stale or from an invalid clock window",
        ));
    }
    Ok(())
}

fn validate_id(value: &str, label: &str) -> NativeVmResult<()> {
    if value.len() < 8
        || value.len() > 96
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        || value.starts_with('-')
        || value.ends_with('-')
        || value.contains("--")
    {
        return Err(invalid(format!("{label} is not a safe stable identifier")));
    }
    Ok(())
}

fn safe_label(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn require_absolute_clean_path(path: &Path, label: &str) -> NativeVmResult<()> {
    if !path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
    {
        return Err(invalid(format!("{label} must be an absolute clean path")));
    }
    Ok(())
}

#[cfg(unix)]
fn canonical_secure_directory(path: &Path, label: &str) -> NativeVmResult<PathBuf> {
    reject_authentication_lexical_path(path, label)?;
    let canonical = fs::canonicalize(path)?;
    require_absolute_clean_path(&canonical, label)?;
    if path_contains_authentication_component(&canonical) {
        return Err(invalid(format!(
            "{label} resolves through an authentication- or credential-like path"
        )));
    }
    if canonical != path {
        return Err(invalid(format!(
            "{label} must not traverse a symbolic-link or non-canonical alias"
        )));
    }
    let metadata = fs::symlink_metadata(&canonical)?;
    if !metadata.is_dir()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.permissions().mode() & 0o7000 != 0
        || metadata.permissions().mode() & 0o022 != 0
    {
        return Err(invalid(format!(
            "{label} must be an owner-controlled non-writable-by-others directory"
        )));
    }
    Ok(canonical)
}

#[cfg(unix)]
fn require_owner_private_directory(path: &Path, label: &str) -> NativeVmResult<()> {
    reject_authentication_lexical_path(path, label)?;
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_dir()
        || metadata.file_type().is_symlink()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.permissions().mode() & 0o7777 != 0o700
    {
        return Err(invalid(format!(
            "{label} must be an owner-only mode-0700 directory"
        )));
    }
    Ok(())
}

#[cfg(unix)]
fn require_canonical_owner_private_directory(path: &Path, label: &str) -> NativeVmResult<()> {
    if fs::canonicalize(path)? != path {
        return Err(invalid(format!(
            "{label} must not traverse a symbolic-link alias"
        )));
    }
    require_owner_private_directory(path, label)
}

#[cfg(unix)]
fn create_new_private_directory(path: &Path) -> NativeVmResult<()> {
    use std::os::unix::fs::DirBuilderExt;
    let mut builder = fs::DirBuilder::new();
    builder.mode(0o700);
    builder.create(path)?;
    require_owner_private_directory(path, "new collection directory")?;
    sync_directory(path)?;
    Ok(())
}

#[cfg(unix)]
fn create_new_private_file(path: &Path) -> NativeVmResult<File> {
    reject_authentication_lexical_path(path, "private output file")?;
    let parent = path
        .parent()
        .ok_or_else(|| invalid("private file has no parent"))?;
    require_owner_private_directory(parent, "private file parent")?;
    let mut options = OpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW);
    let file = options.open(path)?;
    validate_open_private_file(path, &file)?;
    Ok(file)
}

#[cfg(unix)]
fn validate_open_private_file(path: &Path, file: &File) -> NativeVmResult<()> {
    let path_metadata = fs::symlink_metadata(path)?;
    let handle_metadata = file.metadata()?;
    if !path_metadata.is_file()
        || path_metadata.file_type().is_symlink()
        || path_metadata.uid() != unsafe { libc::geteuid() }
        || path_metadata.permissions().mode() & 0o7777 != 0o600
        || path_metadata.nlink() != 1
        || path_metadata.dev() != handle_metadata.dev()
        || path_metadata.ino() != handle_metadata.ino()
    {
        return Err(invalid("private output file failed path/handle binding"));
    }
    Ok(())
}

#[cfg(unix)]
fn open_bound_regular_file(
    path: &Path,
    owner: u32,
    expected_size: Option<u64>,
) -> NativeVmResult<File> {
    validate_existing_input_path_before_open(path, "input file")?;
    let before = fs::symlink_metadata(path)?;
    if !before.is_file()
        || before.file_type().is_symlink()
        || before.uid() != owner
        || before.nlink() != 1
        || before.permissions().mode() & 0o7000 != 0
        || before.permissions().mode() & 0o022 != 0
        || expected_size.is_some_and(|size| before.len() != size)
    {
        return Err(invalid("input is not a bound regular single-link file"));
    }
    let mut options = OpenOptions::new();
    options
        .read(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW);
    let file = options.open(path)?;
    let handle = file.metadata()?;
    let after = fs::symlink_metadata(path)?;
    if before.dev() != handle.dev()
        || before.ino() != handle.ino()
        || before.dev() != after.dev()
        || before.ino() != after.ino()
        || before.len() != handle.len()
        || before.len() != after.len()
        || handle.nlink() != 1
    {
        return Err(invalid("input path changed during binding"));
    }
    Ok(file)
}

#[cfg(unix)]
fn validate_existing_regular_file_metadata(
    path: &Path,
    label: &str,
    expected_size: Option<u64>,
    executable: bool,
) -> NativeVmResult<()> {
    validate_existing_input_path_before_open(path, label)?;
    let metadata = fs::symlink_metadata(path)?;
    let mode = metadata.permissions().mode() & 0o7777;
    if !metadata.is_file()
        || metadata.file_type().is_symlink()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.nlink() != 1
        || mode & 0o7000 != 0
        || mode & 0o022 != 0
        || executable && mode & 0o111 == 0
        || expected_size.is_some_and(|size| metadata.len() != size)
    {
        return Err(invalid(format!(
            "{label} metadata is not an exact owner-bound single-link file"
        )));
    }
    Ok(())
}

#[cfg(unix)]
fn validate_bound_file_against_pin(
    path: &Path,
    expected_sha256: &str,
    expected_size: Option<u64>,
) -> NativeVmResult<()> {
    validate_existing_input_path_before_open(path, "pinned file")?;
    let metadata = fs::symlink_metadata(path)?;
    let owner = unsafe { libc::geteuid() };
    if metadata.uid() != owner {
        return Err(invalid("pinned file is not owned by the collector user"));
    }
    let file = open_bound_regular_file(path, owner, expected_size)?;
    if sha256_reader(file)? != expected_sha256 {
        return Err(invalid("pinned file digest drifted"));
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_bound_file_against_pin(
    _path: &Path,
    _expected_sha256: &str,
    _expected_size: Option<u64>,
) -> NativeVmResult<()> {
    Err(invalid("pinned file validation requires Unix semantics"))
}

#[cfg(unix)]
fn read_bounded_bound_file(path: &Path, owner: u32, max_bytes: u64) -> NativeVmResult<Vec<u8>> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.len() > max_bytes {
        return Err(invalid("bounded input exceeds its byte limit"));
    }
    let mut file = open_bound_regular_file(path, owner, Some(metadata.len()))?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.read_to_end(&mut bytes)?;
    if file.metadata()?.len() != metadata.len() {
        return Err(invalid("bounded input changed while being read"));
    }
    Ok(bytes)
}

#[cfg(unix)]
fn read_bounded_secure_file(path: &Path, max_bytes: u64) -> NativeVmResult<Vec<u8>> {
    validate_existing_input_path_before_open(path, "private artifact")?;
    let metadata = fs::symlink_metadata(path)?;
    if metadata.uid() != unsafe { libc::geteuid() }
        || metadata.permissions().mode() & 0o7777 != 0o600
        || metadata.nlink() != 1
    {
        return Err(invalid("private artifact metadata drifted"));
    }
    read_bounded_bound_file(path, metadata.uid(), max_bytes)
}

#[cfg(not(unix))]
fn read_bounded_secure_file(_path: &Path, _max_bytes: u64) -> NativeVmResult<Vec<u8>> {
    Err(invalid("private artifact reading requires Unix semantics"))
}

#[cfg(unix)]
fn write_private_json_with_digest<T: Serialize>(path: &Path, value: &T) -> NativeVmResult<String> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    let digest = sha256_bytes(&bytes);
    let mut file = create_new_private_file(path)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    write_private_text_create_new(
        &path.with_file_name(format!(
            "{}.sha256",
            path.file_name()
                .and_then(|value| value.to_str())
                .ok_or_else(|| invalid("JSON artifact name is not UTF-8"))?
        )),
        &format!("{digest}\n"),
    )?;
    sync_directory(
        path.parent()
            .ok_or_else(|| invalid("JSON artifact has no parent"))?,
    )?;
    Ok(digest)
}

#[cfg(unix)]
fn write_private_text_create_new(path: &Path, value: &str) -> NativeVmResult<()> {
    let mut file = create_new_private_file(path)?;
    file.write_all(value.as_bytes())?;
    file.sync_all()?;
    Ok(())
}

#[cfg(unix)]
fn read_private_json_with_digest<T: for<'de> Deserialize<'de>>(
    path: &Path,
    max_bytes: u64,
) -> NativeVmResult<(T, String)> {
    let bytes = read_bounded_secure_file(path, max_bytes)?;
    let sidecar = path.with_file_name(format!(
        "{}.sha256",
        path.file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| invalid("JSON artifact name is not UTF-8"))?
    ));
    validate_digest_sidecar_for_bytes(&bytes, &sidecar)?;
    let digest = sha256_bytes(&bytes);
    Ok((serde_json::from_slice(&bytes)?, digest))
}

#[cfg(unix)]
fn validate_digest_sidecar_for_bytes(bytes: &[u8], sidecar: &Path) -> NativeVmResult<()> {
    let expected = read_bounded_secure_file(sidecar, 128)?;
    let expected = std::str::from_utf8(trim_ascii(&expected))
        .map_err(|_| invalid("digest sidecar is not UTF-8"))?;
    if !is_sha256(expected) || sha256_bytes(bytes) != expected {
        return Err(invalid("artifact digest sidecar does not match"));
    }
    Ok(())
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> NativeVmResult<()> {
    validate_existing_input_path_before_open(path, "directory to fsync")?;
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_dir()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.permissions().mode() & 0o7000 != 0
        || metadata.permissions().mode() & 0o022 != 0
    {
        return Err(invalid("directory to fsync is not owner-controlled"));
    }
    let mut options = OpenOptions::new();
    options
        .read(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_DIRECTORY);
    let directory = options.open(path)?;
    let handle = directory.metadata()?;
    if handle.dev() != metadata.dev() || handle.ino() != metadata.ino() {
        return Err(invalid("directory changed while being opened for fsync"));
    }
    directory.sync_all()?;
    Ok(())
}

#[cfg(unix)]
fn sha256_secure_file(path: &Path, max_bytes: u64) -> NativeVmResult<String> {
    Ok(sha256_bytes(&read_bounded_secure_file(path, max_bytes)?))
}

fn sha256_reader(mut reader: impl Read) -> std::io::Result<String> {
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn is_sha256(value: &str) -> bool {
    value.len() == SHA256_HEX_LEN
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn current_unix_ms() -> NativeVmResult<u64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| invalid("system clock precedes Unix epoch"))?;
    u64::try_from(duration.as_millis()).map_err(|_| invalid("Unix millisecond clock overflow"))
}

fn slash_path(path: &Path) -> NativeVmResult<String> {
    let value = path.to_str().ok_or_else(|| invalid("path is not UTF-8"))?;
    Ok(value.replace(std::path::MAIN_SEPARATOR, "/"))
}

fn trim_ascii(bytes: &[u8]) -> &[u8] {
    let start = bytes
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(bytes.len());
    let end = bytes
        .iter()
        .rposition(|byte| !byte.is_ascii_whitespace())
        .map_or(start, |index| index + 1);
    &bytes[start..end]
}

fn bytes_contain_forbidden_surface(bytes: &[u8]) -> bool {
    let lower = String::from_utf8_lossy(bytes).to_ascii_lowercase();
    [
        "api.anthropic.com",
        "api.openai.com",
        "anthropic_api_key",
        "openai_api_key",
        "codex_access_token",
        "auth.json",
        "credentials.json",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

/// Validate a deterministic POSIX ustar archive without extracting it. Every regular file must
/// exactly match the independently derived manifest; links, devices, PAX/GNU extensions, unsafe
/// paths, duplicates, trailing data, and unexpected files fail closed before the SSH stream.
pub fn validate_ustar_archive(
    archive: &Path,
    manifest: &NativeVmBundleManifest,
) -> NativeVmResult<()> {
    validate_bundle_manifest(manifest)?;
    #[cfg(not(unix))]
    {
        let _ = archive;
        return Err(invalid("ustar validation requires Unix file binding"));
    }
    #[cfg(unix)]
    {
        validate_existing_input_path_before_open(archive, "ustar archive")?;
        let metadata = fs::symlink_metadata(archive)?;
        if metadata.len() == 0 || metadata.len() > MAX_VM_BUNDLE_BYTES + (64 * 1024 * 1024) {
            return Err(invalid("ustar archive size is outside the bound"));
        }
        let owner = unsafe { libc::geteuid() };
        let mut file = open_bound_regular_file(archive, owner, Some(metadata.len()))?;
        let expected = manifest
            .entries
            .iter()
            .map(|entry| (entry.relative_path.as_str(), entry))
            .collect::<BTreeMap<_, _>>();
        let expected_directories = manifest
            .entries
            .iter()
            .flat_map(|entry| {
                Path::new(&entry.relative_path)
                    .ancestors()
                    .skip(1)
                    .filter(|path| !path.as_os_str().is_empty())
                    .map(Path::to_path_buf)
            })
            .map(|path| slash_path(&path))
            .collect::<NativeVmResult<BTreeSet<_>>>()?;
        let mut observed = BTreeSet::new();
        let mut observed_directories = BTreeSet::new();
        let mut zero_blocks = 0_u8;
        let mut consumed = 0_u64;
        loop {
            let mut header = [0_u8; 512];
            file.read_exact(&mut header)?;
            consumed += 512;
            if header.iter().all(|byte| *byte == 0) {
                zero_blocks += 1;
                if zero_blocks == 2 {
                    break;
                }
                continue;
            }
            if zero_blocks != 0 {
                return Err(invalid("ustar archive has data after a zero block"));
            }
            validate_tar_checksum(&header)?;
            if &header[257..263] != b"ustar\0" {
                return Err(invalid("archive is not strict POSIX ustar"));
            }
            let name = tar_path(&header)?;
            let normalized = name.trim_end_matches('/');
            validate_relative_bundle_path(Path::new(normalized))?;
            let kind = header[156];
            let size = parse_tar_octal(&header[124..136], "size")?;
            let raw_mode = u32::try_from(parse_tar_octal(&header[100..108], "mode")?)
                .map_err(|_| invalid("ustar mode does not fit in u32"))?;
            if raw_mode > 0o7777 || raw_mode & 0o7000 != 0 {
                return Err(invalid(
                    "ustar entry has a setuid, setgid, sticky, or non-permission mode",
                ));
            }
            let mode = raw_mode;
            if header[157..257].iter().any(|byte| *byte != 0) {
                return Err(invalid("ustar entry contains a link target"));
            }
            match kind {
                0 | b'0' => {
                    let entry = expected
                        .get(normalized)
                        .ok_or_else(|| invalid("ustar contains an unexpected regular file"))?;
                    if !observed.insert(normalized.to_string())
                        || size != entry.size_bytes
                        || mode != entry.mode
                    {
                        return Err(invalid("ustar file metadata drifted from manifest"));
                    }
                    let digest = hash_exact_bytes(&mut file, size)?;
                    if digest != entry.sha256 {
                        return Err(invalid("ustar file content drifted from manifest"));
                    }
                    consumed = consumed
                        .checked_add(size)
                        .ok_or_else(|| invalid("ustar byte count overflow"))?;
                    let padding = (512 - (size % 512)) % 512;
                    discard_exact_bytes(&mut file, padding)?;
                    consumed += padding;
                }
                b'5' => {
                    if size != 0
                        || mode & 0o022 != 0
                        || !expected_directories.contains(normalized)
                        || !observed_directories.insert(normalized.to_string())
                    {
                        return Err(invalid(
                            "ustar directory is unexpected, duplicate, writable, or nonempty",
                        ));
                    }
                }
                _ => return Err(invalid("ustar contains a non-file/non-directory entry")),
            }
            if consumed > metadata.len() {
                return Err(invalid("ustar structure exceeds the bound file"));
            }
        }
        if observed
            != expected
                .keys()
                .map(|path| (*path).to_string())
                .collect::<BTreeSet<_>>()
        {
            return Err(invalid("ustar is missing one or more manifest files"));
        }
        let mut trailing = Vec::new();
        file.read_to_end(&mut trailing)?;
        if trailing.iter().any(|byte| *byte != 0) {
            return Err(invalid("ustar contains nonzero trailing data"));
        }
        Ok(())
    }
}

fn validate_tar_checksum(header: &[u8; 512]) -> NativeVmResult<()> {
    let expected = parse_tar_octal(&header[148..156], "checksum")?;
    let actual = header.iter().enumerate().fold(0_u64, |sum, (index, byte)| {
        sum + if (148..156).contains(&index) {
            u64::from(b' ')
        } else {
            u64::from(*byte)
        }
    });
    if expected != actual {
        return Err(invalid("ustar header checksum is invalid"));
    }
    Ok(())
}

fn parse_tar_octal(bytes: &[u8], label: &str) -> NativeVmResult<u64> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| invalid(format!("ustar {label} is not ASCII")))?
        .trim_matches(|character: char| character == '\0' || character == ' ');
    if text.is_empty() || !text.bytes().all(|byte| (b'0'..=b'7').contains(&byte)) {
        return Err(invalid(format!("ustar {label} is not strict octal")));
    }
    u64::from_str_radix(text, 8).map_err(|_| invalid(format!("ustar {label} overflow")))
}

fn tar_path(header: &[u8; 512]) -> NativeVmResult<String> {
    let name = nul_terminated_ascii(&header[0..100], "name")?;
    let prefix = nul_terminated_ascii(&header[345..500], "prefix")?;
    let path = if prefix.is_empty() {
        name
    } else {
        format!("{prefix}/{name}")
    };
    if path.is_empty() {
        return Err(invalid("ustar entry path is empty"));
    }
    Ok(path)
}

fn nul_terminated_ascii(bytes: &[u8], label: &str) -> NativeVmResult<String> {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    if bytes[end..].iter().any(|byte| *byte != 0)
        || !bytes[..end].iter().all(|byte| byte.is_ascii())
    {
        return Err(invalid(format!("ustar {label} is not canonical ASCII")));
    }
    Ok(std::str::from_utf8(&bytes[..end])
        .map_err(|_| invalid(format!("ustar {label} is not UTF-8")))?
        .to_string())
}

fn hash_exact_bytes(reader: &mut impl Read, size: u64) -> NativeVmResult<String> {
    let mut remaining = size;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    while remaining > 0 {
        let wanted = usize::try_from(remaining.min(buffer.len() as u64))
            .map_err(|_| invalid("archive read size overflow"))?;
        reader.read_exact(&mut buffer[..wanted])?;
        hasher.update(&buffer[..wanted]);
        remaining -= wanted as u64;
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn discard_exact_bytes(reader: &mut impl Read, size: u64) -> NativeVmResult<()> {
    let mut remaining = size;
    let mut buffer = [0_u8; 512];
    while remaining > 0 {
        let wanted = usize::try_from(remaining.min(buffer.len() as u64))
            .map_err(|_| invalid("archive padding size overflow"))?;
        reader.read_exact(&mut buffer[..wanted])?;
        if buffer[..wanted].iter().any(|byte| *byte != 0) {
            return Err(invalid("ustar payload padding is nonzero"));
        }
        remaining -= wanted as u64;
    }
    Ok(())
}
