//! Daemon management for multi-session engram support.
//!
//! This module provides functionality to manage engram daemons that allow
//! multiple Claude/Cursor sessions to share the same knowledge base.

use anyhow::{bail, Context, Result};
use serde::de::{DeserializeOwned, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tracing::{debug, info, warn};

/// Default port for the global engram daemon.
pub const DEFAULT_DAEMON_PORT: u16 = 8765;

/// Port range to search for available ports.
const PORT_RANGE: std::ops::Range<u16> = 8765..8775;

/// Extra time after a successful health check to catch children that exit
/// immediately while another daemon instance is satisfying the probe.
const DAEMON_SPAWN_STABILITY_DELAY: Duration = Duration::from_millis(250);
const DAEMON_ERROR_LOG_LINES: usize = 20;
const DAEMON_SPAWN_METADATA_SCHEMA_VERSION: u32 = 1;
const MAX_EXACT_CONTROL_FILE_BYTES: usize = 16 * 1024;
const MAX_EXACT_TOKEN_BYTES: usize = 4 * 1024;
const MAX_EXACT_HEALTH_BYTES: usize = 64 * 1024;

#[cfg(test)]
static PROXY_ENV_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
pub(crate) struct PoisonedProxyEnvironment {
    _lock: std::sync::MutexGuard<'static, ()>,
    saved: Vec<(&'static str, Option<std::ffi::OsString>)>,
}

#[cfg(test)]
impl PoisonedProxyEnvironment {
    pub(crate) fn install(proxy_url: &str) -> Self {
        let lock = PROXY_ENV_TEST_LOCK.lock().expect("proxy environment lock");
        let saved = ["HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "NO_PROXY"]
            .into_iter()
            .map(|name| (name, std::env::var_os(name)))
            .collect();
        std::env::set_var("HTTP_PROXY", proxy_url);
        std::env::set_var("HTTPS_PROXY", proxy_url);
        std::env::set_var("ALL_PROXY", proxy_url);
        std::env::set_var("NO_PROXY", "");
        Self { _lock: lock, saved }
    }
}

#[cfg(test)]
impl Drop for PoisonedProxyEnvironment {
    fn drop(&mut self) {
        for (name, value) in &self.saved {
            if let Some(value) = value {
                std::env::set_var(name, value);
            } else {
                std::env::remove_var(name);
            }
        }
    }
}

/// Configuration for daemon management.
#[derive(Debug, Clone, Default)]
pub struct DaemonConfig {
    /// Project name (None for global daemon).
    pub project: Option<String>,
    /// Port to use (None for auto-select).
    pub port: Option<u16>,
}

impl DaemonConfig {
    /// Create a config for the global daemon.
    pub fn global() -> Self {
        Self::default()
    }

    /// Create a config for a project-specific daemon.
    pub fn project(name: impl Into<String>) -> Self {
        Self {
            project: Some(name.into()),
            port: None,
        }
    }

    /// Get the directory for daemon files.
    fn daemon_dir(&self) -> PathBuf {
        let base = engram_store::StoreConfig::state_dir();

        match &self.project {
            Some(name) => base.join("projects").join(name),
            None => base,
        }
    }

    /// Get the path to the daemon port file.
    pub fn port_file(&self) -> PathBuf {
        self.daemon_dir().join("daemon.port")
    }

    /// Get the path to the daemon PID file.
    pub fn pid_file(&self) -> PathBuf {
        self.daemon_dir().join("daemon.pid")
    }

    /// Get the path to the daemon log file.
    pub fn log_file(&self) -> PathBuf {
        self.daemon_dir().join("daemon.log")
    }

    /// Get the path to the daemon spawn metadata file.
    pub fn metadata_file(&self) -> PathBuf {
        self.daemon_dir().join("daemon.meta.json")
    }

    /// Get the path to the bearer token protecting the daemon MCP endpoint.
    pub fn token_file(&self) -> PathBuf {
        self.daemon_dir().join("daemon.token")
    }

    /// Get the path used to serialize concurrent daemon startup attempts.
    fn start_lock_file(&self) -> PathBuf {
        self.daemon_dir().join("daemon.start.lock")
    }

    /// Get the data directory for this daemon.
    #[allow(dead_code)]
    pub fn data_dir(&self) -> PathBuf {
        self.daemon_dir().join("data")
    }
}

/// Spawn-time runtime metadata for a daemon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonSpawnMetadata {
    #[serde(default)]
    pub schema_version: u32,
    pub executable_path: String,
    pub executable_version: String,
    #[serde(default)]
    pub executable_sha256: Option<String>,
    pub pid: u32,
    pub port: u16,
}

impl DaemonSpawnMetadata {
    pub(crate) fn new(executable_path: &Path, pid: u32, port: u16) -> Self {
        Self {
            schema_version: DAEMON_SPAWN_METADATA_SCHEMA_VERSION,
            executable_path: executable_path.display().to_string(),
            executable_version: env!("CARGO_PKG_VERSION").to_string(),
            executable_sha256: executable_sha256(executable_path).ok(),
            pid,
            port,
        }
    }
}

/// Live identity returned by the daemon health endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonHealth {
    pub status: String,
    pub service: String,
    pub version: String,
    #[serde(default)]
    pub build_sha: Option<String>,
    #[serde(default)]
    pub pid: Option<u32>,
    #[serde(default)]
    pub health_schema_version: Option<u32>,
    #[serde(default)]
    pub mcp_contract_version: Option<u32>,
    #[serde(default)]
    pub mcp_tools_sha256: Option<String>,
    #[serde(default)]
    pub mcp_protocol_version: Option<String>,
    #[serde(default)]
    pub auth_required: Option<bool>,
    #[serde(default)]
    pub storage_status: Option<String>,
    #[serde(default)]
    pub storage_ready: Option<bool>,
    #[serde(default)]
    pub storage_available_bytes: Option<u64>,
    #[serde(default)]
    pub storage_required_bytes: Option<u64>,
}

impl DaemonHealth {
    /// Whether the daemon is ready to serve write-capable MCP traffic.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.status == "ok" && self.service == "engram" && self.storage_ready.unwrap_or(true)
    }
}

/// Result of comparing the invoking CLI, spawn record, and live daemon identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonAttestationStatus {
    Matched,
    Degraded,
    Drifted,
}

impl std::fmt::Display for DaemonAttestationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Matched => write!(f, "matched"),
            Self::Degraded => write!(f, "degraded"),
            Self::Drifted => write!(f, "drifted"),
        }
    }
}

/// Structured runtime attestation diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonRuntimeAttestation {
    pub status: DaemonAttestationStatus,
    pub warnings: Vec<String>,
}

/// Information about a running daemon.
#[derive(Debug, Clone)]
pub struct DaemonInfo {
    pub port: u16,
    pub pid: u32,
    pub healthy: bool,
    pub health: Option<DaemonHealth>,
    pub metadata: Option<DaemonSpawnMetadata>,
}

/// Transport evidence for one existing daemon.
///
/// This is deliberately crate-private, non-cloneable, and non-serializable. It proves only that
/// the local transport controls were coherent during one bounded observation; it is not process
/// authority. A caller that needs execution authority must retain and monitor the original
/// `Child` and its exact launch record for the complete operation.
pub(crate) struct ExistingDaemonTransport {
    port: u16,
    auth_token: String,
}

impl ExistingDaemonTransport {
    /// Consume the observation immediately before starting the stdio proxy.
    pub(crate) fn into_proxy_parts(self) -> (u16, String) {
        (self.port, self.auth_token)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExactDaemonSpawnMetadata {
    schema_version: u32,
    executable_path: String,
    executable_version: String,
    executable_sha256: String,
    pid: u32,
    port: u16,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExactDaemonHealth {
    status: String,
    service: String,
    version: String,
    build_sha: Option<String>,
    pid: u32,
    health_schema_version: u32,
    mcp_contract_version: u32,
    mcp_tools_sha256: String,
    mcp_protocol_version: String,
    auth_required: bool,
    storage_status: String,
    storage_ready: bool,
    storage_available_bytes: u64,
    storage_required_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExactFileIdentity {
    device: u64,
    inode: u64,
    length: u64,
    mode: u32,
    owner: u32,
    links: u64,
    modified_seconds: i64,
    modified_nanoseconds: i64,
    changed_seconds: i64,
    changed_nanoseconds: i64,
}

struct ExactFileSnapshot {
    name: &'static str,
    label: &'static str,
    identity: ExactFileIdentity,
    bytes: Vec<u8>,
    file: fs::File,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExactDirectoryIdentity {
    device: u64,
    inode: u64,
    length: u64,
    mode: u32,
    owner: u32,
    links: u64,
    modified_seconds: i64,
    modified_nanoseconds: i64,
    changed_seconds: i64,
    changed_nanoseconds: i64,
}

#[cfg(unix)]
struct PinnedExactDirectory {
    file: fs::File,
    identity: ExactDirectoryIdentity,
    entry_name: Option<std::ffi::OsString>,
}

#[cfg(unix)]
struct PinnedExactDaemonDirectory {
    ancestors: Vec<PinnedExactDirectory>,
    projects: PinnedExactDirectory,
    daemon: PinnedExactDirectory,
}

#[cfg(unix)]
struct PinnedExactExecutable {
    ancestors: Vec<PinnedExactDirectory>,
    file: fs::File,
    identity: ExactFileIdentity,
    canonical_path: PathBuf,
    entry_name: std::ffi::OsString,
}

#[cfg(unix)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExactTransportValidationPhase {
    ControlsPinned,
    HealthValidated,
}

/// Ensure a daemon is running, starting one if necessary.
///
/// Returns the port the daemon is listening on.
pub async fn ensure_daemon_running(config: &DaemonConfig) -> Result<u16> {
    let daemon_directory = config.daemon_dir();
    if existing_controls_require_read_only_resolution(&daemon_directory)? {
        return reuse_existing_daemon_or_block(config).await;
    }
    engram_store::ensure_private_directory(&config.daemon_dir())
        .context("Failed to restrict daemon directory permissions")?;
    let _start_lock = acquire_daemon_start_lock(config, Duration::from_secs(35)).await?;
    require_absent_or_complete_daemon_controls(&config.daemon_dir())?;
    if !present_daemon_control_names(&config.daemon_dir())?.is_empty() {
        return reuse_existing_daemon_or_block(config).await;
    }
    require_no_unresolved_daemon_controls(&config.daemon_dir())?;
    ensure_private_daemon_paths(config)?;
    engram_store::ensure_private_directory(&config.data_dir())
        .context("Failed to create private daemon data directory before startup fencing")?;
    fs::File::create(config.log_file())
        .context("Failed to create daemon log before startup fencing")?;
    engram_store::ensure_private_file(&config.log_file())
        .context("Failed to restrict daemon log permissions before startup fencing")?;
    let auth_token = load_or_create_daemon_token(config)?;

    // Find an available port
    let port = config.port.unwrap_or_else(find_available_port);
    info!("Starting daemon on port {}", port);

    // Spawn the daemon and make sure the spawned child is the process that stays alive.
    let exe = std::env::current_exe().context("Failed to get current executable path")?;
    require_no_unresolved_daemon_controls(&config.daemon_dir())?;
    let prepared = prepare_spawned_daemon_control_transaction(&config.daemon_dir())?;
    let mut child = spawn_daemon(config, port, &exe, &auth_token).with_context(|| {
        format!(
            "Failed to spawn daemon; the durable startup control was preserved in {} for \
             explicit operator recovery",
            config.daemon_dir().display()
        )
    })?;
    let pid = child.id();
    let metadata = DaemonSpawnMetadata::new(&exe, pid, port);

    // Persist the child identity before waiting. MCP hosts may start the same stdio server more
    // than once and terminate one launcher while its detached daemon child is still becoming
    // ready. A later launcher can now wait for and reuse that exact child instead of spawning a
    // second RocksDB owner.
    let persisted = persist_spawned_daemon_info(prepared, &mut child, port, pid, &metadata)?;

    // Wait for daemon to be ready
    if let Err(start_error) = wait_for_spawned_daemon(
        port,
        &mut child,
        Duration::from_secs(30),
        &config.log_file(),
    )
    .await
    {
        if let Err(cleanup_error) =
            terminate_then_rollback_spawned_daemon(&mut child, &persisted, |_| Ok(()))
        {
            bail!("{start_error}; spawned-daemon cleanup failed: {cleanup_error}");
        }
        return Err(start_error);
    }
    if let Err(control_error) = validate_persisted_daemon_controls(&persisted) {
        if let Err(cleanup_error) =
            terminate_then_rollback_spawned_daemon(&mut child, &persisted, |_| Ok(()))
        {
            bail!(
                "Spawned daemon controls changed before readiness completion: {control_error}; \
                 spawned-daemon cleanup failed: {cleanup_error}"
            );
        }
        bail!("Spawned daemon controls changed before readiness completion: {control_error}");
    }

    Ok(port)
}

async fn reuse_existing_daemon_or_block(config: &DaemonConfig) -> Result<u16> {
    let info = get_daemon_info(config).await.with_context(|| {
        "Existing daemon controls could not be validated; refusing automatic cleanup or a second \
         daemon start"
    })?;
    if info.healthy {
        debug!(
            "Daemon already running on port {} (PID {})",
            info.port, info.pid
        );
        return Ok(info.port);
    }
    if let Some(health) = &info.health {
        bail!(
            "Engram daemon PID {} is responding on port {} but is not ready (datastore: {}). \
             Do not start a second daemon while this process owns the store. Check free disk \
             space and `engram daemon logs`, then explicitly establish safe recovery.",
            info.pid,
            info.port,
            health.storage_status.as_deref().unwrap_or("unknown")
        );
    }
    if wait_for_recorded_daemon(info.port, info.pid, Duration::from_secs(3)).await {
        debug!(
            "Daemon became ready on port {} (PID {}) while startup state was being checked",
            info.port, info.pid
        );
        return Ok(info.port);
    }
    warn!("Recorded daemon is not responding; refusing automatic identity-control cleanup");
    bail!(
        "Existing daemon controls may belong to a live or unreaped process; refusing automatic \
         cleanup or a second daemon start without positive process identity and termination \
         authority"
    )
}

async fn wait_for_recorded_daemon(port: u16, pid: u32, timeout: Duration) -> bool {
    let started = std::time::Instant::now();
    while started.elapsed() < timeout {
        if fetch_daemon_health(port)
            .await
            .is_ok_and(|health| daemon_health_matches_pid(&health, pid))
        {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    false
}

async fn acquire_daemon_start_lock(config: &DaemonConfig, timeout: Duration) -> Result<fs::File> {
    let path = config.start_lock_file();
    acquire_daemon_start_lock_file(&path, timeout).await
}

async fn acquire_daemon_start_lock_file(path: &Path, timeout: Duration) -> Result<fs::File> {
    let file = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(path)
        .with_context(|| format!("Failed to open daemon startup lock: {}", path.display()))?;
    engram_store::ensure_private_file(path)
        .with_context(|| format!("Failed to restrict daemon startup lock: {}", path.display()))?;

    let started = std::time::Instant::now();
    loop {
        match fs2::FileExt::try_lock_exclusive(&file) {
            Ok(()) => return Ok(file),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                if started.elapsed() >= timeout {
                    bail!(
                        "Timed out waiting for another Engram process to finish daemon startup: {}",
                        path.display()
                    );
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("Failed to lock daemon startup file: {}", path.display())
                })
            }
        }
    }
}

/// Get information about a running daemon.
pub async fn get_daemon_info(config: &DaemonConfig) -> Result<DaemonInfo> {
    let port = read_daemon_port(config)?;
    let pid = read_daemon_pid(config)?;
    let health = fetch_daemon_health(port).await.ok();
    let healthy = health
        .as_ref()
        .is_some_and(|health| daemon_health_matches_pid(health, pid));
    let metadata = read_daemon_metadata(config);

    Ok(DaemonInfo {
        port,
        pid,
        healthy,
        health,
        metadata,
    })
}

/// Observe one exact project-daemon transport without starting, repairing, or cleaning state.
///
/// The `/health` observation is not a read-only datastore operation: the daemon commits a unique
/// canary creation and deletion in one transaction. Subsequent proxy traffic may mutate within its
/// selected MCP profile, and proxy shutdown deletes the proxy's MCP session.
///
/// This does not establish process authority. The native evaluator must separately retain the
/// original isolated daemon `Child` and store, validate its semantic behavior for the full
/// operation lifetime, and constrain MCP traffic to the intended phase.
pub(crate) async fn connect_existing_exact_transport(
    config: &DaemonConfig,
    expected_port: u16,
    expected_pid: u32,
) -> Result<ExistingDaemonTransport> {
    let project = config
        .project
        .as_deref()
        .context("Exact existing-daemon transport requires an explicit project")?;
    validate_exact_project_name(project)?;
    let current_executable = std::env::current_exe()
        .context("Failed to resolve current executable for exact daemon validation")?
        .canonicalize()
        .context("Failed to canonicalize current executable for exact daemon validation")?;
    connect_existing_exact_transport_at(
        &engram_store::StoreConfig::state_dir(),
        project,
        expected_port,
        expected_pid,
        &current_executable,
    )
    .await
}

pub(crate) fn validate_exact_project_name(project: &str) -> Result<()> {
    if project.contains('/') || project.contains('\\') || project.as_bytes().contains(&0) {
        bail!("Exact existing-daemon project must be one nonempty normal path component");
    }
    let mut components = Path::new(project).components();
    let Some(std::path::Component::Normal(component)) = components.next() else {
        bail!("Exact existing-daemon project must be one nonempty normal path component");
    };
    if components.next().is_some()
        || component != Path::new(project).as_os_str()
        || project.trim() != project
    {
        bail!("Exact existing-daemon project must be one nonempty normal path component");
    }
    Ok(())
}

#[cfg(unix)]
async fn connect_existing_exact_transport_at(
    state_root: &Path,
    project: &str,
    expected_port: u16,
    expected_pid: u32,
    current_executable: &Path,
) -> Result<ExistingDaemonTransport> {
    connect_existing_exact_transport_at_with_hook(
        state_root,
        project,
        expected_port,
        expected_pid,
        current_executable,
        |_| Ok(()),
    )
    .await
}

#[cfg(unix)]
async fn connect_existing_exact_transport_at_with_hook(
    state_root: &Path,
    project: &str,
    expected_port: u16,
    expected_pid: u32,
    current_executable: &Path,
    mut after_phase: impl FnMut(ExactTransportValidationPhase) -> Result<()>,
) -> Result<ExistingDaemonTransport> {
    if expected_port == 0 || expected_pid == 0 {
        bail!("Expected daemon port and PID must both be nonzero");
    }
    validate_exact_project_name(project)?;

    let mut executable = open_pinned_exact_executable(current_executable)?;
    let directory = open_pinned_exact_daemon_directory(state_root, project)?;

    let mut port_snapshot = read_exact_owner_file_at(
        &directory.daemon.file,
        "daemon.port",
        "daemon port control",
        MAX_EXACT_CONTROL_FILE_BYTES,
    )?;
    let mut pid_snapshot = read_exact_owner_file_at(
        &directory.daemon.file,
        "daemon.pid",
        "daemon PID control",
        MAX_EXACT_CONTROL_FILE_BYTES,
    )?;
    let mut metadata_snapshot = read_exact_owner_file_at(
        &directory.daemon.file,
        "daemon.meta.json",
        "daemon spawn metadata",
        MAX_EXACT_CONTROL_FILE_BYTES,
    )?;
    let mut token_snapshot = read_exact_owner_file_at(
        &directory.daemon.file,
        "daemon.token",
        "daemon bearer token",
        MAX_EXACT_TOKEN_BYTES,
    )?;
    after_phase(ExactTransportValidationPhase::ControlsPinned)?;

    let observed_port =
        parse_exact_decimal_line::<u16>(&port_snapshot.bytes, "daemon port control")?;
    let observed_pid = parse_exact_decimal_line::<u32>(&pid_snapshot.bytes, "daemon PID control")?;
    if observed_port != expected_port || observed_pid != expected_pid {
        bail!("Existing daemon controls do not match the expected port and PID");
    }

    let metadata: ExactDaemonSpawnMetadata = decode_strict_json(
        &metadata_snapshot.bytes,
        "Daemon spawn metadata is not exact schema-1 JSON",
    )?;
    let current_executable_sha256 = digest_pinned_executable(&mut executable)?;
    if metadata.schema_version != DAEMON_SPAWN_METADATA_SCHEMA_VERSION
        || metadata.executable_path != executable.canonical_path.display().to_string()
        || metadata.executable_version != env!("CARGO_PKG_VERSION")
        || metadata.executable_sha256 != current_executable_sha256
        || metadata.pid != expected_pid
        || metadata.port != expected_port
    {
        bail!("Daemon spawn metadata does not match the current executable and exact controls");
    }

    if token_snapshot.bytes.len() != 64
        || !token_snapshot
            .bytes
            .iter()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte))
    {
        bail!("Daemon bearer token does not match the exact generated token format");
    }
    let auth_token = String::from_utf8(token_snapshot.bytes.clone())
        .context("Daemon bearer token is not UTF-8")?;

    require_process_liveness_observation(expected_pid)?;
    let health = fetch_exact_daemon_health(expected_port).await?;
    validate_exact_daemon_health(&health, expected_pid)?;
    require_process_liveness_observation(expected_pid)?;
    after_phase(ExactTransportValidationPhase::HealthValidated)?;

    if digest_pinned_executable(&mut executable)? != current_executable_sha256 {
        bail!("Current executable changed during exact daemon validation");
    }
    require_pinned_directories_unchanged(&directory)?;
    require_pinned_directories_unchanged_for_executable(&executable)?;
    require_unchanged_exact_file_at(&directory.daemon.file, &mut port_snapshot)?;
    require_unchanged_exact_file_at(&directory.daemon.file, &mut pid_snapshot)?;
    require_unchanged_exact_file_at(&directory.daemon.file, &mut metadata_snapshot)?;
    require_unchanged_exact_file_at(&directory.daemon.file, &mut token_snapshot)?;

    Ok(ExistingDaemonTransport {
        port: expected_port,
        auth_token,
    })
}

#[cfg(not(unix))]
async fn connect_existing_exact_transport_at(
    _state_root: &Path,
    _project: &str,
    _expected_port: u16,
    _expected_pid: u32,
    _current_executable: &Path,
) -> Result<ExistingDaemonTransport> {
    bail!("Exact existing-daemon transport requires Unix descriptor validation")
}

fn parse_exact_decimal_line<T>(bytes: &[u8], label: &str) -> Result<T>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    if bytes.len() < 2
        || bytes.last() != Some(&b'\n')
        || !bytes[..bytes.len() - 1].iter().all(u8::is_ascii_digit)
    {
        bail!("{label} is not an exact decimal line");
    }
    std::str::from_utf8(&bytes[..bytes.len() - 1])
        .with_context(|| format!("{label} is not UTF-8"))?
        .parse::<T>()
        .with_context(|| format!("{label} is out of range"))
}

async fn fetch_exact_daemon_health(port: u16) -> Result<ExactDaemonHealth> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(500))
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;
    let url = format!("http://127.0.0.1:{port}/health");
    let mut response = client.get(&url).send().await?;
    if response.status() != reqwest::StatusCode::OK {
        bail!(
            "Exact daemon health endpoint returned HTTP {}",
            response.status()
        );
    }
    if response
        .content_length()
        .is_some_and(|length| length > MAX_EXACT_HEALTH_BYTES as u64)
    {
        bail!("Exact daemon health response exceeded its byte ceiling");
    }

    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if body.len().saturating_add(chunk.len()) > MAX_EXACT_HEALTH_BYTES {
            bail!("Exact daemon health response exceeded its byte ceiling");
        }
        body.extend_from_slice(&chunk);
    }

    decode_exact_daemon_health(&body)
}

fn decode_exact_daemon_health(body: &[u8]) -> Result<ExactDaemonHealth> {
    let (health, value): (ExactDaemonHealth, serde_json::Value) =
        decode_strict_json_with_value(body, "Exact daemon health response is not exact JSON")?;
    let object = value
        .as_object()
        .context("Exact daemon health response is not an object")?;
    let expected_keys = BTreeSet::from([
        "auth_required",
        "build_sha",
        "health_schema_version",
        "mcp_contract_version",
        "mcp_protocol_version",
        "mcp_tools_sha256",
        "pid",
        "service",
        "status",
        "storage_available_bytes",
        "storage_ready",
        "storage_required_bytes",
        "storage_status",
        "version",
    ]);
    if object.keys().map(String::as_str).collect::<BTreeSet<_>>() != expected_keys {
        bail!("Exact daemon health response key set drifted");
    }
    Ok(health)
}

struct StrictJsonValue(serde_json::Value);

impl<'de> Deserialize<'de> for StrictJsonValue {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(StrictJsonVisitor)
    }
}

struct StrictJsonVisitor;

impl<'de> Visitor<'de> for StrictJsonVisitor {
    type Value = StrictJsonValue;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("JSON without duplicate object keys")
    }

    fn visit_bool<E>(self, value: bool) -> std::result::Result<Self::Value, E> {
        Ok(StrictJsonValue(serde_json::Value::Bool(value)))
    }

    fn visit_i64<E>(self, value: i64) -> std::result::Result<Self::Value, E> {
        Ok(StrictJsonValue(value.into()))
    }

    fn visit_u64<E>(self, value: u64) -> std::result::Result<Self::Value, E> {
        Ok(StrictJsonValue(value.into()))
    }

    fn visit_f64<E>(self, value: f64) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        serde_json::Number::from_f64(value)
            .map(serde_json::Value::Number)
            .map(StrictJsonValue)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E> {
        Ok(StrictJsonValue(value.into()))
    }

    fn visit_string<E>(self, value: String) -> std::result::Result<Self::Value, E> {
        Ok(StrictJsonValue(value.into()))
    }

    fn visit_none<E>(self) -> std::result::Result<Self::Value, E> {
        Ok(StrictJsonValue(serde_json::Value::Null))
    }

    fn visit_unit<E>(self) -> std::result::Result<Self::Value, E> {
        Ok(StrictJsonValue(serde_json::Value::Null))
    }

    fn visit_some<D>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        StrictJsonValue::deserialize(deserializer)
    }

    fn visit_seq<A>(self, mut sequence: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element::<StrictJsonValue>()? {
            values.push(value.0);
        }
        Ok(StrictJsonValue(serde_json::Value::Array(values)))
    }

    fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = serde_json::Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(serde::de::Error::custom(format!(
                    "duplicate JSON object key: {key}"
                )));
            }
            let value = map.next_value::<StrictJsonValue>()?;
            values.insert(key, value.0);
        }
        Ok(StrictJsonValue(serde_json::Value::Object(values)))
    }
}

fn decode_strict_json<T>(bytes: &[u8], label: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    decode_strict_json_with_value(bytes, label).map(|(typed, _)| typed)
}

fn decode_strict_json_with_value<T>(bytes: &[u8], label: &str) -> Result<(T, serde_json::Value)>
where
    T: DeserializeOwned,
{
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let value = StrictJsonValue::deserialize(&mut deserializer)
        .with_context(|| label.to_string())?
        .0;
    deserializer
        .end()
        .with_context(|| format!("{label}: trailing JSON data"))?;
    let typed = serde_json::from_value(value.clone()).with_context(|| label.to_string())?;
    Ok((typed, value))
}

fn validate_exact_daemon_health(health: &ExactDaemonHealth, expected_pid: u32) -> Result<()> {
    if health.status != "ok"
        || health.service != "engram"
        || health.version != env!("CARGO_PKG_VERSION")
        || health.build_sha.as_deref() != option_env!("ENGRAM_BUILD_SHA")
        || health.pid != expected_pid
        || health.health_schema_version != engram_mcp::server::DAEMON_HEALTH_SCHEMA_VERSION
        || health.mcp_contract_version != engram_mcp::server::MCP_CONTRACT_VERSION
        || health.mcp_tools_sha256 != engram_mcp::server::mcp_tools_sha256()
        || health.mcp_protocol_version != engram_mcp::server::MCP_PROTOCOL_VERSION
        || !health.auth_required
        || health.storage_status != "ready"
        || !health.storage_ready
        || health.storage_required_bytes == 0
        || health.storage_available_bytes < health.storage_required_bytes
    {
        bail!("Daemon health identity/storage/auth contract is not ready and exact");
    }
    Ok(())
}

#[cfg(unix)]
fn open_pinned_exact_daemon_directory(
    state_root: &Path,
    project: &str,
) -> Result<PinnedExactDaemonDirectory> {
    use std::ffi::OsStr;

    let ancestors = open_absolute_directory_chain(state_root, "Engram state root")?;
    require_private_directory(
        ancestors
            .last()
            .context("Engram state root did not resolve to a directory")?,
        "Engram state root",
    )?;
    let projects = open_pinned_directory_at(
        &ancestors.last().expect("checked state root").file,
        OsStr::new("projects"),
        "Engram projects directory",
    )?;
    require_private_directory(&projects, "Engram projects directory")?;
    let daemon = open_pinned_directory_at(
        &projects.file,
        OsStr::new(project),
        "project daemon control directory",
    )?;
    require_private_directory(&daemon, "project daemon control directory")?;
    Ok(PinnedExactDaemonDirectory {
        ancestors,
        projects,
        daemon,
    })
}

#[cfg(unix)]
fn open_absolute_directory_chain(path: &Path, label: &str) -> Result<Vec<PinnedExactDirectory>> {
    use std::ffi::OsStr;
    use std::os::fd::FromRawFd;

    if !path.is_absolute() {
        bail!("{label} must be absolute");
    }
    let root_fd = unsafe {
        nix::libc::open(
            c"/".as_ptr(),
            nix::libc::O_RDONLY
                | nix::libc::O_DIRECTORY
                | nix::libc::O_NOFOLLOW
                | nix::libc::O_CLOEXEC,
        )
    };
    if root_fd < 0 {
        return Err(std::io::Error::last_os_error()).context("Failed to pin filesystem root");
    }
    // SAFETY: `open` returned a new owned descriptor on success.
    let root = unsafe { fs::File::from_raw_fd(root_fd) };
    let mut pinned = vec![pinned_directory_from_file(root, "filesystem root", None)?];
    for component in path.components() {
        match component {
            std::path::Component::RootDir => {}
            std::path::Component::Normal(name) => {
                let next = open_pinned_directory_at(
                    &pinned.last().expect("filesystem root is pinned").file,
                    name,
                    label,
                )?;
                pinned.push(next);
            }
            std::path::Component::CurDir
            | std::path::Component::ParentDir
            | std::path::Component::Prefix(_) => {
                bail!("{label} must be a canonical absolute path")
            }
        }
    }
    if pinned.len() == 1 && path != Path::new(OsStr::new("/")) {
        bail!("{label} must be a canonical absolute path");
    }
    Ok(pinned)
}

#[cfg(unix)]
fn open_pinned_directory_at(
    parent: &fs::File,
    name: &std::ffi::OsStr,
    label: &str,
) -> Result<PinnedExactDirectory> {
    let file = openat_file(
        parent,
        name,
        nix::libc::O_RDONLY | nix::libc::O_DIRECTORY | nix::libc::O_NOFOLLOW | nix::libc::O_CLOEXEC,
    )
    .with_context(|| format!("Failed to pin {label}"))?;
    pinned_directory_from_file(file, label, Some(name.to_os_string()))
}

#[cfg(unix)]
fn pinned_directory_from_file(
    file: fs::File,
    label: &str,
    entry_name: Option<std::ffi::OsString>,
) -> Result<PinnedExactDirectory> {
    let metadata = file
        .metadata()
        .with_context(|| format!("Failed to inspect pinned {label}"))?;
    if !metadata.is_dir() {
        bail!("Pinned {label} is not a directory");
    }
    Ok(PinnedExactDirectory {
        identity: exact_directory_identity(&metadata),
        file,
        entry_name,
    })
}

#[cfg(unix)]
fn require_private_directory(directory: &PinnedExactDirectory, label: &str) -> Result<()> {
    // SAFETY: `geteuid` has no preconditions and does not access caller memory.
    let effective_uid = unsafe { nix::libc::geteuid() };
    if directory.identity.mode != 0o700 || directory.identity.owner != effective_uid {
        bail!("{label} must be an owner-controlled mode-0700 directory");
    }
    Ok(())
}

#[cfg(unix)]
fn open_pinned_exact_executable(path: &Path) -> Result<PinnedExactExecutable> {
    use std::ffi::OsString;

    if !path.is_absolute() {
        bail!("Current executable path must be absolute");
    }
    let mut parent = PathBuf::from("/");
    let mut file_name: Option<OsString> = None;
    for component in path.components() {
        match component {
            std::path::Component::RootDir => {}
            std::path::Component::Normal(name) => {
                if let Some(previous) = file_name.replace(name.to_os_string()) {
                    parent.push(previous);
                }
            }
            std::path::Component::CurDir
            | std::path::Component::ParentDir
            | std::path::Component::Prefix(_) => {
                bail!("Current executable path must be canonical")
            }
        }
    }
    let file_name = file_name.context("Current executable path has no file name")?;
    let ancestors = open_absolute_directory_chain(&parent, "current executable parent")?;
    let parent_file = &ancestors
        .last()
        .expect("absolute executable parent pins filesystem root")
        .file;
    let entry_identity = exact_file_identity_at(parent_file, &file_name, "current executable")?;
    let file = openat_file(
        parent_file,
        &file_name,
        nix::libc::O_RDONLY | nix::libc::O_NOFOLLOW | nix::libc::O_CLOEXEC | nix::libc::O_NONBLOCK,
    )
    .context("Failed to pin current executable")?;
    let metadata = file
        .metadata()
        .context("Failed to inspect current executable")?;
    if !metadata.is_file() {
        bail!("Current executable is not a regular file");
    }
    let identity = exact_file_identity(&metadata);
    if identity != entry_identity {
        bail!("Current executable changed while it was pinned");
    }
    Ok(PinnedExactExecutable {
        ancestors,
        file,
        identity,
        canonical_path: path.to_path_buf(),
        entry_name: file_name,
    })
}

#[cfg(unix)]
fn digest_pinned_executable(executable: &mut PinnedExactExecutable) -> Result<String> {
    if exact_file_identity(&executable.file.metadata()?) != executable.identity {
        bail!("Pinned current executable identity changed");
    }
    executable.file.seek(SeekFrom::Start(0))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    let mut length = 0_u64;
    loop {
        let read = executable.file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        length = length.saturating_add(read as u64);
        hasher.update(&buffer[..read]);
    }
    if length != executable.identity.length
        || exact_file_identity(&executable.file.metadata()?) != executable.identity
    {
        bail!("Pinned current executable changed while it was hashed");
    }
    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(unix)]
fn read_exact_owner_file_at(
    directory: &fs::File,
    name: &'static str,
    label: &'static str,
    max_bytes: usize,
) -> Result<ExactFileSnapshot> {
    use std::ffi::OsStr;

    let entry_identity = exact_file_identity_at(directory, OsStr::new(name), label)?;
    let mut file = openat_file(
        directory,
        OsStr::new(name),
        nix::libc::O_RDONLY | nix::libc::O_NOFOLLOW | nix::libc::O_CLOEXEC | nix::libc::O_NONBLOCK,
    )
    .with_context(|| format!("Failed to open {label}"))?;
    let metadata = file
        .metadata()
        .with_context(|| format!("Failed to inspect open {label}"))?;
    if !metadata.is_file() {
        bail!("{label} must be a regular file, not a symlink");
    }
    let identity = exact_file_identity(&metadata);
    // SAFETY: `geteuid` has no preconditions and does not access caller memory.
    let effective_uid = unsafe { nix::libc::geteuid() };
    if identity != entry_identity
        || identity.mode != 0o600
        || identity.owner != effective_uid
        || identity.links != 1
        || identity.length > max_bytes as u64
    {
        bail!("{label} must be an unchanged owner-controlled mode-0600 single-link file");
    }

    let mut bytes = Vec::new();
    (&mut file)
        .take((max_bytes as u64).saturating_add(1))
        .read_to_end(&mut bytes)
        .with_context(|| format!("Failed to read {label}"))?;
    if bytes.len() > max_bytes
        || bytes.len() as u64 != identity.length
        || exact_file_identity(&file.metadata()?) != identity
    {
        bail!("{label} changed while it was being read");
    }
    Ok(ExactFileSnapshot {
        name,
        label,
        identity,
        bytes,
        file,
    })
}

#[cfg(unix)]
fn require_unchanged_exact_file_at(
    directory: &fs::File,
    expected: &mut ExactFileSnapshot,
) -> Result<()> {
    use std::ffi::OsStr;

    if exact_file_identity_at(directory, OsStr::new(expected.name), expected.label)?
        != expected.identity
        || exact_file_identity(&expected.file.metadata()?) != expected.identity
    {
        bail!("{} changed during exact daemon validation", expected.label);
    }
    expected.file.seek(SeekFrom::Start(0))?;
    let mut observed = Vec::new();
    (&mut expected.file)
        .take((expected.bytes.len() as u64).saturating_add(1))
        .read_to_end(&mut observed)?;
    if observed != expected.bytes
        || exact_file_identity(&expected.file.metadata()?) != expected.identity
    {
        bail!("{} changed during exact daemon validation", expected.label);
    }
    Ok(())
}

#[cfg(unix)]
fn require_pinned_directories_unchanged(directory: &PinnedExactDaemonDirectory) -> Result<()> {
    for pinned in directory
        .ancestors
        .iter()
        .chain([&directory.projects, &directory.daemon])
    {
        if !same_pinned_directory(
            &exact_directory_identity(&pinned.file.metadata()?),
            &pinned.identity,
        ) {
            bail!("Pinned daemon directory identity changed during validation");
        }
    }
    require_pinned_directory_chain(&directory.ancestors)?;
    let state_root = directory.ancestors.last().expect("state root is pinned");
    require_pinned_directory_entry(state_root, &directory.projects)?;
    require_pinned_directory_entry(&directory.projects, &directory.daemon)?;
    Ok(())
}

#[cfg(unix)]
fn require_pinned_directories_unchanged_for_executable(
    executable: &PinnedExactExecutable,
) -> Result<()> {
    for pinned in &executable.ancestors {
        if !same_pinned_directory(
            &exact_directory_identity(&pinned.file.metadata()?),
            &pinned.identity,
        ) {
            bail!("Pinned executable directory identity changed during validation");
        }
    }
    require_pinned_directory_chain(&executable.ancestors)?;
    let parent = &executable
        .ancestors
        .last()
        .expect("executable parent is pinned")
        .file;
    if exact_file_identity_at(parent, &executable.entry_name, "current executable")?
        != executable.identity
    {
        bail!("Current executable directory entry changed during validation");
    }
    Ok(())
}

#[cfg(unix)]
fn require_pinned_directory_chain(chain: &[PinnedExactDirectory]) -> Result<()> {
    for pair in chain.windows(2) {
        require_pinned_directory_entry(&pair[0], &pair[1])?;
    }
    Ok(())
}

#[cfg(unix)]
fn require_pinned_directory_entry(
    parent: &PinnedExactDirectory,
    child: &PinnedExactDirectory,
) -> Result<()> {
    let name = child
        .entry_name
        .as_deref()
        .context("Pinned child directory has no entry name")?;
    let entry_identity = exact_directory_identity_at(&parent.file, name)?;
    let handle_identity = exact_directory_identity(&child.file.metadata()?);
    if !same_pinned_directory(&entry_identity, &handle_identity)
        || !same_pinned_directory(&handle_identity, &child.identity)
    {
        bail!("Pinned directory entry changed during validation");
    }
    Ok(())
}

fn same_pinned_directory(
    observed: &ExactDirectoryIdentity,
    expected: &ExactDirectoryIdentity,
) -> bool {
    observed.device == expected.device
        && observed.inode == expected.inode
        && observed.mode == expected.mode
        && observed.owner == expected.owner
}

#[cfg(unix)]
fn openat_file(parent: &fs::File, name: &std::ffi::OsStr, flags: i32) -> Result<fs::File> {
    use std::os::fd::{AsRawFd, FromRawFd};
    use std::os::unix::ffi::OsStrExt;

    let name = std::ffi::CString::new(name.as_bytes()).context("Path component contains NUL")?;
    // SAFETY: pointers are valid for the call and a successful descriptor is immediately owned.
    let fd = unsafe { nix::libc::openat(parent.as_raw_fd(), name.as_ptr(), flags) };
    if fd < 0 {
        return Err(std::io::Error::last_os_error()).context("openat failed");
    }
    // SAFETY: `openat` returned a new owned descriptor on success.
    Ok(unsafe { fs::File::from_raw_fd(fd) })
}

#[cfg(unix)]
fn exact_file_identity_at(
    directory: &fs::File,
    name: &std::ffi::OsStr,
    label: &str,
) -> Result<ExactFileIdentity> {
    use std::os::fd::AsRawFd;
    use std::os::unix::ffi::OsStrExt;

    let name = std::ffi::CString::new(name.as_bytes()).context("Path component contains NUL")?;
    let mut stat = std::mem::MaybeUninit::<nix::libc::stat>::uninit();
    // SAFETY: `stat` points to writable storage and `name` is NUL terminated for this call.
    let result = unsafe {
        nix::libc::fstatat(
            directory.as_raw_fd(),
            name.as_ptr(),
            stat.as_mut_ptr(),
            nix::libc::AT_SYMLINK_NOFOLLOW,
        )
    };
    if result != 0 {
        return Err(std::io::Error::last_os_error())
            .with_context(|| format!("Failed to inspect {label} relative to pinned directory"));
    }
    // SAFETY: successful `fstatat` initialized the complete value.
    let stat = unsafe { stat.assume_init() };
    if stat.st_mode & nix::libc::S_IFMT != nix::libc::S_IFREG {
        bail!("{label} must be a regular file, not a symlink");
    }
    exact_file_identity_from_stat(&stat)
}

#[cfg(unix)]
fn exact_directory_identity_at(
    directory: &fs::File,
    name: &std::ffi::OsStr,
) -> Result<ExactDirectoryIdentity> {
    use std::os::fd::AsRawFd;
    use std::os::unix::ffi::OsStrExt;

    let name = std::ffi::CString::new(name.as_bytes()).context("Path component contains NUL")?;
    let mut stat = std::mem::MaybeUninit::<nix::libc::stat>::uninit();
    // SAFETY: `stat` points to writable storage and `name` is NUL terminated for this call.
    let result = unsafe {
        nix::libc::fstatat(
            directory.as_raw_fd(),
            name.as_ptr(),
            stat.as_mut_ptr(),
            nix::libc::AT_SYMLINK_NOFOLLOW,
        )
    };
    if result != 0 {
        return Err(std::io::Error::last_os_error())
            .context("Failed to reinspect pinned directory entry");
    }
    // SAFETY: successful `fstatat` initialized the complete value.
    let stat = unsafe { stat.assume_init() };
    if stat.st_mode & nix::libc::S_IFMT != nix::libc::S_IFDIR {
        bail!("Pinned directory entry is no longer a directory");
    }
    exact_directory_identity_from_stat(&stat)
}

#[cfg(target_os = "macos")]
fn exact_file_identity_from_stat(stat: &nix::libc::stat) -> Result<ExactFileIdentity> {
    Ok(ExactFileIdentity {
        device: stat.st_dev as u64,
        inode: stat.st_ino,
        length: stat.st_size as u64,
        mode: u32::from(stat.st_mode) & 0o777,
        owner: stat.st_uid,
        links: stat.st_nlink as u64,
        modified_seconds: stat.st_mtime,
        modified_nanoseconds: stat.st_mtime_nsec,
        changed_seconds: stat.st_ctime,
        changed_nanoseconds: stat.st_ctime_nsec,
    })
}

#[cfg(target_os = "macos")]
fn exact_directory_identity_from_stat(stat: &nix::libc::stat) -> Result<ExactDirectoryIdentity> {
    Ok(ExactDirectoryIdentity {
        device: stat.st_dev as u64,
        inode: stat.st_ino,
        length: stat.st_size as u64,
        mode: u32::from(stat.st_mode) & 0o777,
        owner: stat.st_uid,
        links: stat.st_nlink as u64,
        modified_seconds: stat.st_mtime,
        modified_nanoseconds: stat.st_mtime_nsec,
        changed_seconds: stat.st_ctime,
        changed_nanoseconds: stat.st_ctime_nsec,
    })
}

#[cfg(target_os = "linux")]
#[allow(clippy::unnecessary_cast)] // libc stat field aliases vary across Linux targets.
fn exact_file_identity_from_stat(stat: &nix::libc::stat) -> Result<ExactFileIdentity> {
    Ok(ExactFileIdentity {
        device: stat.st_dev as u64,
        inode: stat.st_ino as u64,
        length: stat.st_size as u64,
        mode: stat.st_mode as u32 & 0o777,
        owner: stat.st_uid as u32,
        links: stat.st_nlink as u64,
        modified_seconds: stat.st_mtime as i64,
        modified_nanoseconds: stat.st_mtime_nsec as i64,
        changed_seconds: stat.st_ctime as i64,
        changed_nanoseconds: stat.st_ctime_nsec as i64,
    })
}

#[cfg(target_os = "linux")]
#[allow(clippy::unnecessary_cast)] // libc stat field aliases vary across Linux targets.
fn exact_directory_identity_from_stat(stat: &nix::libc::stat) -> Result<ExactDirectoryIdentity> {
    Ok(ExactDirectoryIdentity {
        device: stat.st_dev as u64,
        inode: stat.st_ino as u64,
        length: stat.st_size as u64,
        mode: stat.st_mode as u32 & 0o777,
        owner: stat.st_uid as u32,
        links: stat.st_nlink as u64,
        modified_seconds: stat.st_mtime as i64,
        modified_nanoseconds: stat.st_mtime_nsec as i64,
        changed_seconds: stat.st_ctime as i64,
        changed_nanoseconds: stat.st_ctime_nsec as i64,
    })
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn exact_file_identity_from_stat(_stat: &nix::libc::stat) -> Result<ExactFileIdentity> {
    bail!("Exact descriptor identity is supported only on macOS and Linux")
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn exact_directory_identity_from_stat(_stat: &nix::libc::stat) -> Result<ExactDirectoryIdentity> {
    bail!("Exact descriptor identity is supported only on macOS and Linux")
}

#[cfg(unix)]
fn exact_file_identity(metadata: &fs::Metadata) -> ExactFileIdentity {
    use std::os::unix::fs::MetadataExt;

    ExactFileIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
        length: metadata.len(),
        mode: metadata.mode() & 0o777,
        owner: metadata.uid(),
        links: metadata.nlink(),
        modified_seconds: metadata.mtime(),
        modified_nanoseconds: metadata.mtime_nsec(),
        changed_seconds: metadata.ctime(),
        changed_nanoseconds: metadata.ctime_nsec(),
    }
}

#[cfg(unix)]
fn exact_directory_identity(metadata: &fs::Metadata) -> ExactDirectoryIdentity {
    use std::os::unix::fs::MetadataExt;

    ExactDirectoryIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
        length: metadata.len(),
        mode: metadata.mode() & 0o777,
        owner: metadata.uid(),
        links: metadata.nlink(),
        modified_seconds: metadata.mtime(),
        modified_nanoseconds: metadata.mtime_nsec(),
        changed_seconds: metadata.ctime(),
        changed_nanoseconds: metadata.ctime_nsec(),
    }
}

#[cfg(unix)]
fn require_process_liveness_observation(pid: u32) -> Result<()> {
    use nix::sys::signal;
    use nix::unistd::Pid;

    let pid = i32::try_from(pid).context("Expected daemon PID is out of platform range")?;
    signal::kill(Pid::from_raw(pid), None).context("Expected daemon process is not alive")
}

#[cfg(not(unix))]
fn require_process_liveness_observation(_pid: u32) -> Result<()> {
    bail!("Exact existing-daemon connections require Unix process identity checks")
}

/// Fetch and validate the live daemon identity from its health endpoint.
pub async fn fetch_daemon_health(port: u16) -> Result<DaemonHealth> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(500))
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let url = format!("http://127.0.0.1:{}/health", port);
    let response = client.get(&url).send().await?;

    let health: DaemonHealth = response
        .json()
        .await
        .context("Daemon health response is not valid attestation JSON")?;
    if health.service != "engram" {
        bail!(
            "Daemon health identity mismatch: service={}",
            health.service,
        );
    }
    Ok(health)
}

pub(crate) fn daemon_health_matches_pid(health: &DaemonHealth, expected_pid: u32) -> bool {
    health.is_ready() && health.pid == Some(expected_pid)
}

/// Compare the current CLI with the daemon's live and spawn-time identities.
pub fn attest_daemon_runtime(
    info: &DaemonInfo,
    current_executable: Option<&Path>,
    current_version: &str,
    current_build_sha: Option<&str>,
) -> DaemonRuntimeAttestation {
    let mut drift = Vec::new();
    let mut degraded = Vec::new();

    match &info.health {
        Some(health) => {
            if health.version != current_version {
                drift.push(format!(
                    "live daemon version {} differs from current CLI version {}",
                    health.version, current_version
                ));
            }
            if health.pid.is_some_and(|pid| pid != info.pid) {
                drift.push("live daemon pid differs from daemon.pid".to_string());
            }
            match health.health_schema_version {
                Some(version) if version == engram_mcp::server::DAEMON_HEALTH_SCHEMA_VERSION => {}
                Some(version) => drift.push(format!(
                    "daemon health schema {} differs from supported schema {}",
                    version,
                    engram_mcp::server::DAEMON_HEALTH_SCHEMA_VERSION
                )),
                None => {
                    degraded.push("live daemon does not expose a health schema version".to_string())
                }
            }
            match health.mcp_contract_version {
                Some(version) if version == engram_mcp::server::MCP_CONTRACT_VERSION => {}
                Some(version) => drift.push(format!(
                    "daemon MCP contract {} differs from supported contract {}",
                    version,
                    engram_mcp::server::MCP_CONTRACT_VERSION
                )),
                None => {
                    degraded.push("live daemon does not expose an MCP contract version".to_string())
                }
            }
            match health.mcp_tools_sha256.as_deref() {
                Some(hash) if hash == engram_mcp::server::mcp_tools_sha256() => {}
                Some(hash) => drift.push(format!(
                    "live daemon MCP tools SHA-256 {} differs from current contract {}",
                    hash,
                    engram_mcp::server::mcp_tools_sha256()
                )),
                None => {
                    degraded.push("live daemon does not expose an MCP tools SHA-256".to_string())
                }
            }
            match health.mcp_protocol_version.as_deref() {
                Some(version) if version == engram_mcp::server::MCP_PROTOCOL_VERSION => {}
                Some(version) => drift.push(format!(
                    "daemon MCP protocol {} differs from supported protocol {}",
                    version,
                    engram_mcp::server::MCP_PROTOCOL_VERSION
                )),
                None => {
                    degraded.push("live daemon does not expose an MCP protocol version".to_string())
                }
            }
            match health.auth_required {
                Some(true) => {}
                Some(false) => drift.push(
                    "daemon MCP endpoint does not require the local bearer token".to_string(),
                ),
                None => degraded
                    .push("live daemon does not report its MCP authentication mode".to_string()),
            }
            match health.storage_ready {
                Some(true) => {}
                Some(false) => {
                    drift.push("live daemon datastore is not writable".to_string());
                }
                None => degraded
                    .push("live daemon does not report datastore write readiness".to_string()),
            }
            match health.storage_status.as_deref() {
                Some("ready") => {}
                Some(status) => {
                    drift.push(format!("live daemon datastore status is {status}"));
                }
                None => degraded.push("live daemon does not report datastore status".to_string()),
            }
            if let (Some(live), Some(current)) = (health.build_sha.as_deref(), current_build_sha) {
                if live != current {
                    drift.push(format!(
                        "live daemon build SHA {} differs from current CLI build SHA {}",
                        live, current
                    ));
                }
            }
        }
        None => degraded.push("live daemon identity is unavailable".to_string()),
    }

    match &info.metadata {
        Some(metadata) => {
            if metadata.schema_version != DAEMON_SPAWN_METADATA_SCHEMA_VERSION {
                degraded.push(format!(
                    "spawn metadata schema {} differs from current schema {}",
                    metadata.schema_version, DAEMON_SPAWN_METADATA_SCHEMA_VERSION
                ));
            }
            if metadata.pid != info.pid || metadata.port != info.port {
                drift.push("spawn metadata does not match daemon pid/port files".to_string());
            }
            if let Some(health) = &info.health {
                if metadata.executable_version != health.version {
                    drift.push(format!(
                        "spawn version {} differs from live daemon version {}",
                        metadata.executable_version, health.version
                    ));
                }
            }
            match current_executable {
                Some(path) => {
                    let current_path = path.display().to_string();
                    if metadata.executable_path != current_path {
                        drift.push(format!(
                            "daemon executable {} differs from current CLI {}",
                            metadata.executable_path, current_path
                        ));
                    }
                    match (&metadata.executable_sha256, executable_sha256(path)) {
                        (Some(spawn_hash), Ok(current_hash)) if spawn_hash != &current_hash => {
                            drift.push(
                                "daemon spawn hash differs from the current executable bytes"
                                    .to_string(),
                            );
                        }
                        (Some(_), Ok(_)) => {}
                        (None, _) => degraded
                            .push("spawn metadata does not include an executable hash".to_string()),
                        (_, Err(error)) => degraded.push(format!(
                            "current CLI executable could not be hashed: {error}"
                        )),
                    }
                }
                None => degraded.push("current CLI executable path is unavailable".to_string()),
            }
        }
        None => degraded.push("daemon spawn metadata is unavailable".to_string()),
    }

    let status = if !drift.is_empty() {
        DaemonAttestationStatus::Drifted
    } else if !degraded.is_empty() {
        DaemonAttestationStatus::Degraded
    } else {
        DaemonAttestationStatus::Matched
    };
    drift.extend(degraded);
    DaemonRuntimeAttestation {
        status,
        warnings: drift,
    }
}

pub(crate) fn executable_sha256(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path)
        .with_context(|| format!("Failed to open executable {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .with_context(|| format!("Failed to read executable {}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Wait for the daemon health endpoint while also requiring the spawned child to stay alive.
async fn wait_for_spawned_daemon(
    port: u16,
    child: &mut Child,
    timeout: Duration,
    log_file: &Path,
) -> Result<()> {
    let start = std::time::Instant::now();
    let mut interval = tokio::time::interval(Duration::from_millis(100));

    while start.elapsed() < timeout {
        interval.tick().await;
        ensure_spawned_child_still_running(child, Some(log_file))?;

        if let Ok(health) = fetch_daemon_health(port).await {
            let child_matches = daemon_health_matches_pid(&health, child.id());
            if child_matches {
                tokio::time::sleep(DAEMON_SPAWN_STABILITY_DELAY).await;
                ensure_spawned_child_still_running(child, Some(log_file))?;
                info!("Daemon is ready on port {}", port);
                return Ok(());
            }
            if !health.is_ready() && health.pid == Some(child.id()) {
                bail!(
                    "Spawned Engram daemon is responding but not ready (datastore: {})",
                    health.storage_status.as_deref().unwrap_or("unknown")
                );
            }
        }
    }

    let log_tail = recent_log_tail(log_file, DAEMON_ERROR_LOG_LINES)
        .map(|tail| format!("\n\nRecent daemon log:\n{}", tail))
        .unwrap_or_default();
    bail!(
        "Daemon failed to start within {} seconds{}",
        timeout.as_secs(),
        log_tail
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChildTerminationPhase {
    TryWait,
    Kill,
    Wait,
}

#[cfg(test)]
fn terminate_spawned_child(child: &mut Child) -> Result<()> {
    terminate_spawned_child_with_hook(child, |_| Ok(()))
}

fn terminate_spawned_child_with_hook(
    child: &mut Child,
    mut before_phase: impl FnMut(ChildTerminationPhase) -> Result<()>,
) -> Result<()> {
    before_phase(ChildTerminationPhase::TryWait)?;
    if child
        .try_wait()
        .context("Failed to inspect unready daemon process")?
        .is_some()
    {
        return Ok(());
    }

    before_phase(ChildTerminationPhase::Kill)?;
    child
        .kill()
        .context("Failed to terminate unready daemon process")?;
    before_phase(ChildTerminationPhase::Wait)?;
    child
        .wait()
        .context("Failed to reap unready daemon process")?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DaemonControlPersistencePhase {
    Pid,
    Port,
    Metadata,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DaemonControlPublicationPhase {
    BeforeParentSync,
    AfterParentSyncBeforeWrite,
}

struct PersistedDaemonControls {
    directory_path: PathBuf,
    #[cfg(unix)]
    directory: fs::File,
    #[cfg(unix)]
    directory_identity: ExactDirectoryIdentity,
    controls: Vec<PersistedDaemonControl>,
}

struct PersistedDaemonControl {
    name: &'static str,
    path: PathBuf,
    expected_bytes: Vec<u8>,
    file: fs::File,
    #[cfg(unix)]
    identity: Option<ExactFileIdentity>,
}

fn persist_spawned_daemon_info(
    persisted: PersistedDaemonControls,
    child: &mut Child,
    port: u16,
    pid: u32,
    metadata: &DaemonSpawnMetadata,
) -> Result<PersistedDaemonControls> {
    persist_prepared_daemon_info_with_hooks(
        persisted,
        child,
        port,
        pid,
        metadata,
        |_, _| Ok(()),
        |_| Ok(()),
    )
}

#[cfg(test)]
fn persist_spawned_daemon_info_with_hook(
    daemon_directory: &Path,
    child: &mut Child,
    port: u16,
    pid: u32,
    metadata: &DaemonSpawnMetadata,
    after_phase: impl FnMut(DaemonControlPersistencePhase, &Path) -> Result<()>,
) -> Result<PersistedDaemonControls> {
    persist_spawned_daemon_info_with_hooks(
        daemon_directory,
        child,
        port,
        pid,
        metadata,
        after_phase,
        |_| Ok(()),
    )
}

#[cfg(test)]
fn persist_spawned_daemon_info_with_hooks(
    daemon_directory: &Path,
    child: &mut Child,
    port: u16,
    pid: u32,
    metadata: &DaemonSpawnMetadata,
    after_phase: impl FnMut(DaemonControlPersistencePhase, &Path) -> Result<()>,
    mut before_termination_phase: impl FnMut(ChildTerminationPhase) -> Result<()>,
) -> Result<PersistedDaemonControls> {
    let persisted = match prepare_spawned_daemon_control_transaction(daemon_directory) {
        Ok(persisted) => persisted,
        Err(persist_error) => {
            return match terminate_spawned_child_with_hook(child, &mut before_termination_phase) {
                Ok(()) => Err(persist_error).context(
                    "Failed to begin spawned daemon identity persistence; spawned child was \
                     terminated and reaped",
                ),
                Err(termination_error) => bail!(
                    "Failed to begin spawned daemon identity persistence: {persist_error}; \
                     termination/reap was not established: {termination_error}"
                ),
            };
        }
    };
    persist_prepared_daemon_info_with_hooks(
        persisted,
        child,
        port,
        pid,
        metadata,
        after_phase,
        before_termination_phase,
    )
}

fn persist_prepared_daemon_info_with_hooks(
    mut persisted: PersistedDaemonControls,
    child: &mut Child,
    port: u16,
    pid: u32,
    metadata: &DaemonSpawnMetadata,
    mut after_phase: impl FnMut(DaemonControlPersistencePhase, &Path) -> Result<()>,
    mut before_termination_phase: impl FnMut(ChildTerminationPhase) -> Result<()>,
) -> Result<PersistedDaemonControls> {
    let persist_result =
        write_spawned_daemon_controls(port, pid, metadata, &mut persisted, &mut after_phase);
    if let Err(persist_error) = persist_result {
        return match terminate_then_rollback_spawned_daemon(
            child,
            &persisted,
            &mut before_termination_phase,
        ) {
            Ok(()) => Err(persist_error).context(
                "Failed to persist spawned daemon identity; spawned child was terminated and \
                 reaped and daemon controls were rolled back",
            ),
            Err(cleanup_error) => {
                bail!("Failed to persist spawned daemon identity: {persist_error}; {cleanup_error}")
            }
        };
    }
    Ok(persisted)
}

fn terminate_then_rollback_spawned_daemon(
    child: &mut Child,
    persisted: &PersistedDaemonControls,
    before_termination_phase: impl FnMut(ChildTerminationPhase) -> Result<()>,
) -> Result<()> {
    if let Err(error) = terminate_spawned_child_with_hook(child, before_termination_phase) {
        match sync_preserved_daemon_control_fence(persisted) {
            Ok(()) => bail!(
                "Spawned child termination/reap was not positively established; daemon controls \
                 were preserved and synced as a retry fence: {error}"
            ),
            Err(sync_error) => bail!(
                "Spawned child termination/reap was not positively established; daemon controls \
                 were preserved, but retry-fence sync also failed: {error}; {sync_error}"
            ),
        }
    }
    if let Err(error) = rollback_spawned_daemon_controls(persisted) {
        bail!(
            "Spawned child was terminated and reaped, but exact daemon-control rollback failed: \
             {error}"
        );
    }
    Ok(())
}

#[cfg(unix)]
fn sync_preserved_daemon_control_fence(persisted: &PersistedDaemonControls) -> Result<()> {
    let mut failures = Vec::new();
    for control in &persisted.controls {
        if let Err(error) = control.file.sync_all() {
            failures.push(format!("{}: {error}", control.path.display()));
        }
    }
    if let Err(error) = persisted.directory.sync_all() {
        failures.push(format!("{}: {error}", persisted.directory_path.display()));
    }
    if failures.is_empty() {
        Ok(())
    } else {
        bail!(
            "Failed to sync preserved daemon retry fence: {}",
            failures.join("; ")
        )
    }
}

#[cfg(not(unix))]
fn sync_preserved_daemon_control_fence(persisted: &PersistedDaemonControls) -> Result<()> {
    let mut failures = Vec::new();
    for control in &persisted.controls {
        if let Err(error) = control.file.sync_all() {
            failures.push(format!("{}: {error}", control.path.display()));
        }
    }
    match fs::File::open(&persisted.directory_path).and_then(|directory| directory.sync_all()) {
        Ok(()) => {}
        Err(error) => failures.push(format!("{}: {error}", persisted.directory_path.display())),
    }
    if failures.is_empty() {
        Ok(())
    } else {
        bail!(
            "Failed to sync preserved daemon retry fence: {}",
            failures.join("; ")
        )
    }
}

fn ensure_spawned_child_still_running(child: &mut Child, log_file: Option<&Path>) -> Result<()> {
    if let Some(status) = child
        .try_wait()
        .context("Failed to inspect spawned daemon process")?
    {
        let log_tail = log_file
            .and_then(|path| recent_log_tail(path, DAEMON_ERROR_LOG_LINES))
            .map(|tail| format!("\n\nRecent daemon log:\n{}", tail))
            .unwrap_or_default();
        bail!(
            "Spawned daemon process {} exited before daemon readiness was confirmed: {}{}",
            child.id(),
            status,
            log_tail
        );
    }
    Ok(())
}

fn recent_log_tail(path: &Path, max_lines: usize) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return None;
    }
    let start = lines.len().saturating_sub(max_lines);
    Some(lines[start..].join("\n"))
}

/// Find an available port in the configured range.
fn find_available_port() -> u16 {
    for port in PORT_RANGE {
        if TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return port;
        }
    }
    // Fallback to OS-assigned port
    TcpListener::bind("127.0.0.1:0")
        .and_then(|l| l.local_addr())
        .map(|a| a.port())
        .unwrap_or(DEFAULT_DAEMON_PORT)
}

/// Spawn a daemon process.
fn spawn_daemon(config: &DaemonConfig, port: u16, exe: &Path, auth_token: &str) -> Result<Child> {
    // Ensure daemon directory exists
    let daemon_dir = config.daemon_dir();
    engram_store::ensure_private_directory(&daemon_dir)
        .context("Failed to create private daemon directory")?;
    engram_store::ensure_disk_headroom(&daemon_dir)
        .context("Refusing to start Engram daemon without filesystem headroom")?;

    // Build command arguments
    let mut args = vec![
        "serve".to_string(),
        "--http".to_string(),
        "--port".to_string(),
        port.to_string(),
    ];

    // Add project flag if project-specific
    if let Some(project) = &config.project {
        args.push("--project".to_string());
        args.push(project.clone());
    }

    // Open log file
    let log_file = fs::File::create(config.log_file()).context("Failed to create log file")?;
    engram_store::ensure_private_file(&config.log_file())
        .context("Failed to restrict daemon log permissions")?;

    // Spawn the daemon process
    #[cfg(unix)]
    let child = {
        use std::os::unix::process::CommandExt;

        // Create a new process group so the daemon survives parent exit
        Command::new(exe)
            .args(&args)
            .env("ENGRAM_DAEMON_TOKEN", auth_token)
            .stdin(Stdio::null())
            .stdout(log_file.try_clone()?)
            .stderr(log_file)
            .process_group(0) // Create new process group
            .spawn()
            .context("Failed to spawn daemon")?
    };

    #[cfg(windows)]
    let child = {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        const DETACHED_PROCESS: u32 = 0x00000008;

        Command::new(&exe)
            .args(&args)
            .env("ENGRAM_DAEMON_TOKEN", auth_token)
            .stdin(Stdio::null())
            .stdout(log_file.try_clone()?)
            .stderr(log_file)
            .creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS)
            .spawn()
            .context("Failed to spawn daemon")?
    };

    let pid = child.id();
    info!("Spawned daemon with PID {}", pid);

    Ok(child)
}

/// Stop a running daemon.
pub async fn stop_daemon(config: &DaemonConfig) -> Result<()> {
    let info = get_daemon_info(config).await?;

    #[cfg(unix)]
    {
        use nix::sys::signal::{self, Signal};
        use nix::unistd::Pid;

        let pid = Pid::from_raw(info.pid as i32);
        signal::kill(pid, Signal::SIGTERM).context("Failed to send SIGTERM to daemon")?;
    }

    #[cfg(windows)]
    {
        // On Windows, we'll try to terminate the process
        Command::new("taskkill")
            .args(&["/PID", &info.pid.to_string(), "/F"])
            .output()
            .context("Failed to terminate daemon")?;
    }

    // Wait a moment for cleanup
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Clean up files
    cleanup_daemon_files(config)?;

    info!("Daemon stopped");
    Ok(())
}

/// Read the daemon port from file.
fn read_daemon_port(config: &DaemonConfig) -> Result<u16> {
    let port_file = config.port_file();
    let content = fs::read_to_string(&port_file).context("Failed to read daemon port file")?;
    content
        .trim()
        .parse()
        .context("Failed to parse daemon port")
}

/// Read the daemon PID from file.
fn read_daemon_pid(config: &DaemonConfig) -> Result<u32> {
    let pid_file = config.pid_file();
    let content = fs::read_to_string(&pid_file).context("Failed to read daemon PID file")?;
    content.trim().parse().context("Failed to parse daemon PID")
}

#[cfg(unix)]
fn begin_spawned_daemon_control_transaction(
    daemon_directory: &Path,
) -> Result<PersistedDaemonControls> {
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt};

    engram_store::ensure_private_directory(daemon_directory)
        .context("Failed to restrict daemon control directory")?;
    let directory = fs::OpenOptions::new()
        .read(true)
        .custom_flags(nix::libc::O_DIRECTORY | nix::libc::O_NOFOLLOW | nix::libc::O_CLOEXEC)
        .open(daemon_directory)
        .with_context(|| {
            format!(
                "Failed to pin daemon control directory {}",
                daemon_directory.display()
            )
        })?;
    let metadata = directory
        .metadata()
        .context("Failed to inspect pinned daemon control directory")?;
    // SAFETY: `geteuid` has no preconditions and does not access caller memory.
    let effective_uid = unsafe { nix::libc::geteuid() };
    if !metadata.is_dir() || metadata.mode() & 0o777 != 0o700 || metadata.uid() != effective_uid {
        bail!("Daemon control directory must be an owner-controlled mode-0700 directory");
    }
    Ok(PersistedDaemonControls {
        directory_path: daemon_directory.to_path_buf(),
        directory_identity: exact_directory_identity(&metadata),
        directory,
        controls: Vec::new(),
    })
}

#[cfg(not(unix))]
fn begin_spawned_daemon_control_transaction(
    daemon_directory: &Path,
) -> Result<PersistedDaemonControls> {
    engram_store::ensure_private_directory(daemon_directory)
        .context("Failed to restrict daemon control directory")?;
    Ok(PersistedDaemonControls {
        directory_path: daemon_directory.to_path_buf(),
        controls: Vec::new(),
    })
}

fn prepare_spawned_daemon_control_transaction(
    daemon_directory: &Path,
) -> Result<PersistedDaemonControls> {
    let mut persisted = begin_spawned_daemon_control_transaction(daemon_directory)?;
    create_spawned_daemon_control(&mut persisted, "daemon.pid", b"startup-pending\n".to_vec())?;
    sync_persisted_daemon_directory(&persisted)?;
    Ok(persisted)
}

fn write_spawned_daemon_controls(
    port: u16,
    pid: u32,
    metadata: &DaemonSpawnMetadata,
    persisted: &mut PersistedDaemonControls,
    after_phase: &mut impl FnMut(DaemonControlPersistencePhase, &Path) -> Result<()>,
) -> Result<()> {
    let metadata =
        serde_json::to_vec_pretty(metadata).context("Failed to serialize daemon spawn metadata")?;
    rewrite_spawned_daemon_control(persisted, "daemon.pid", format!("{pid}\n").into_bytes())?;
    sync_persisted_daemon_directory(persisted)?;
    let pid_path = persisted.directory_path.join("daemon.pid");
    after_phase(DaemonControlPersistencePhase::Pid, &pid_path)?;
    validate_persisted_daemon_controls(persisted)?;

    for (phase, name, contents) in [
        (
            DaemonControlPersistencePhase::Port,
            "daemon.port",
            format!("{port}\n").into_bytes(),
        ),
        (
            DaemonControlPersistencePhase::Metadata,
            "daemon.meta.json",
            metadata,
        ),
    ] {
        create_spawned_daemon_control(persisted, name, contents)?;
        // Make each published name crash-durable before a later phase or child-cleanup failure can
        // leave it as the retry fence.
        sync_persisted_daemon_directory(persisted)?;
        let path = persisted.directory_path.join(name);
        after_phase(phase, &path)?;
        validate_persisted_daemon_controls(persisted)?;
    }
    Ok(())
}

#[cfg(unix)]
fn create_spawned_daemon_control(
    persisted: &mut PersistedDaemonControls,
    name: &'static str,
    expected_bytes: Vec<u8>,
) -> Result<()> {
    create_spawned_daemon_control_with_hook(persisted, name, expected_bytes, |_, _| Ok(()))
}

#[cfg(unix)]
fn create_spawned_daemon_control_with_hook(
    persisted: &mut PersistedDaemonControls,
    name: &'static str,
    expected_bytes: Vec<u8>,
    mut publication_hook: impl FnMut(DaemonControlPublicationPhase, &Path) -> Result<()>,
) -> Result<()> {
    require_persisted_daemon_directory_identity(persisted)?;
    let path = persisted.directory_path.join(name);
    let file = create_private_file_at(&persisted.directory, name)
        .with_context(|| format!("Failed to create daemon control {}", path.display()))?;
    persisted.controls.push(PersistedDaemonControl {
        name,
        path: path.clone(),
        expected_bytes,
        file,
        identity: None,
    });
    if let Err(hook_error) =
        publication_hook(DaemonControlPublicationPhase::BeforeParentSync, &path)
    {
        return match persisted.directory.sync_all() {
            Ok(()) => Err(hook_error).context(
                "Daemon-control publication failed before its scheduled parent sync; the parent \
                 was synced before returning",
            ),
            Err(sync_error) => bail!(
                "Daemon-control publication failed before parent sync: {hook_error}; emergency \
                 parent sync also failed: {sync_error}"
            ),
        };
    }
    persisted.directory.sync_all().with_context(|| {
        format!(
            "Failed to make newly published daemon control {} durable",
            path.display()
        )
    })?;
    publication_hook(
        DaemonControlPublicationPhase::AfterParentSyncBeforeWrite,
        &path,
    )?;
    persisted.directory_identity = exact_directory_identity(
        &persisted
            .directory
            .metadata()
            .context("Failed to inspect pinned daemon control directory after creation")?,
    );

    let control = persisted.controls.last_mut().expect("control was pushed");
    control
        .file
        .write_all(&control.expected_bytes)
        .with_context(|| format!("Failed to write daemon control {}", path.display()))?;
    control
        .file
        .sync_all()
        .with_context(|| format!("Failed to sync daemon control {}", path.display()))?;
    let identity = exact_file_identity(&control.file.metadata().with_context(|| {
        format!(
            "Failed to inspect written daemon control {}",
            path.display()
        )
    })?);
    // SAFETY: `geteuid` has no preconditions and does not access caller memory.
    let effective_uid = unsafe { nix::libc::geteuid() };
    if identity.mode != 0o600
        || identity.owner != effective_uid
        || identity.links != 1
        || identity.length != control.expected_bytes.len() as u64
    {
        bail!("New daemon control is not an exact owner-only single-link file");
    }
    control.identity = Some(identity);
    require_persisted_control_bytes(control)?;
    Ok(())
}

#[cfg(not(unix))]
fn create_spawned_daemon_control(
    persisted: &mut PersistedDaemonControls,
    name: &'static str,
    expected_bytes: Vec<u8>,
) -> Result<()> {
    let path = persisted.directory_path.join(name);
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    let file = options
        .open(&path)
        .with_context(|| format!("Failed to create daemon control {}", path.display()))?;
    persisted.controls.push(PersistedDaemonControl {
        name,
        path: path.clone(),
        expected_bytes,
        file,
    });
    fs::File::open(&persisted.directory_path)
        .context("Failed to open daemon control directory for durability sync")?
        .sync_all()
        .context("Failed to make newly published daemon control durable")?;
    let control = persisted.controls.last_mut().expect("control was pushed");
    control
        .file
        .write_all(&control.expected_bytes)
        .with_context(|| format!("Failed to write daemon control {}", path.display()))?;
    control
        .file
        .sync_all()
        .with_context(|| format!("Failed to sync daemon control {}", path.display()))?;
    Ok(())
}

#[cfg(unix)]
fn rewrite_spawned_daemon_control(
    persisted: &mut PersistedDaemonControls,
    name: &'static str,
    expected_bytes: Vec<u8>,
) -> Result<()> {
    let control = persisted
        .controls
        .iter_mut()
        .find(|control| control.name == name)
        .with_context(|| format!("Prepared daemon control {name} is missing"))?;
    control.expected_bytes = expected_bytes;
    control.file.set_len(0)?;
    control.file.seek(SeekFrom::Start(0))?;
    control.file.write_all(&control.expected_bytes)?;
    control.file.sync_all()?;
    let identity = exact_file_identity(&control.file.metadata()?);
    // SAFETY: `geteuid` has no preconditions and does not access caller memory.
    let effective_uid = unsafe { nix::libc::geteuid() };
    if identity.mode != 0o600
        || identity.owner != effective_uid
        || identity.links != 1
        || identity.length != control.expected_bytes.len() as u64
    {
        bail!("Rewritten daemon control is not an exact owner-only single-link file");
    }
    control.identity = Some(identity);
    require_persisted_control_bytes(control)
}

#[cfg(not(unix))]
fn rewrite_spawned_daemon_control(
    persisted: &mut PersistedDaemonControls,
    name: &'static str,
    expected_bytes: Vec<u8>,
) -> Result<()> {
    let control = persisted
        .controls
        .iter_mut()
        .find(|control| control.name == name)
        .with_context(|| format!("Prepared daemon control {name} is missing"))?;
    control.expected_bytes = expected_bytes;
    control.file.set_len(0)?;
    control.file.seek(SeekFrom::Start(0))?;
    control.file.write_all(&control.expected_bytes)?;
    control.file.sync_all()?;
    Ok(())
}

#[cfg(unix)]
fn create_private_file_at(directory: &fs::File, name: &str) -> Result<fs::File> {
    use std::os::fd::{AsRawFd, FromRawFd};

    let name = std::ffi::CString::new(name).context("Daemon control name contains NUL")?;
    // SAFETY: pointers are valid for this call and a successful descriptor is immediately owned.
    let fd = unsafe {
        nix::libc::openat(
            directory.as_raw_fd(),
            name.as_ptr(),
            nix::libc::O_RDWR
                | nix::libc::O_CREAT
                | nix::libc::O_EXCL
                | nix::libc::O_NOFOLLOW
                | nix::libc::O_CLOEXEC,
            0o600,
        )
    };
    if fd < 0 {
        return Err(std::io::Error::last_os_error()).context("openat create failed");
    }
    // SAFETY: `openat` returned a new owned descriptor on success.
    Ok(unsafe { fs::File::from_raw_fd(fd) })
}

#[cfg(unix)]
fn require_persisted_control_bytes(control: &PersistedDaemonControl) -> Result<()> {
    let identity = control
        .identity
        .as_ref()
        .context("Daemon control was not completely persisted; it was preserved")?;
    if exact_file_identity(&control.file.metadata()?) != *identity {
        bail!("Daemon control identity changed before exact byte validation");
    }
    let mut file = control.file.try_clone()?;
    file.seek(SeekFrom::Start(0))?;
    let mut observed = Vec::new();
    (&mut file)
        .take((control.expected_bytes.len() as u64).saturating_add(1))
        .read_to_end(&mut observed)?;
    if observed != control.expected_bytes
        || exact_file_identity(&control.file.metadata()?) != *identity
    {
        bail!("Daemon control bytes or identity changed during exact validation");
    }
    Ok(())
}

#[cfg(unix)]
fn require_persisted_daemon_directory_identity(persisted: &PersistedDaemonControls) -> Result<()> {
    let held_identity = exact_directory_identity(&persisted.directory.metadata()?);
    let path_metadata = fs::symlink_metadata(&persisted.directory_path)?;
    let path_identity = exact_directory_identity(&path_metadata);
    if path_metadata.file_type().is_symlink()
        || !path_metadata.is_dir()
        || !same_persisted_daemon_directory(&held_identity, &persisted.directory_identity)
        || !same_persisted_daemon_directory(&path_identity, &persisted.directory_identity)
    {
        bail!("Pinned daemon control directory changed during the persistence transaction");
    }
    Ok(())
}

#[cfg(unix)]
fn same_persisted_daemon_directory(
    observed: &ExactDirectoryIdentity,
    expected: &ExactDirectoryIdentity,
) -> bool {
    observed.device == expected.device
        && observed.inode == expected.inode
        && observed.mode == expected.mode
        && observed.owner == expected.owner
        && observed.links == expected.links
}

#[cfg(unix)]
fn require_persisted_control_unchanged(
    persisted: &PersistedDaemonControls,
    control: &PersistedDaemonControl,
) -> Result<()> {
    let identity = exact_file_identity_at(
        &persisted.directory,
        std::ffi::OsStr::new(control.name),
        "spawned daemon control",
    )?;
    let expected_identity = control
        .identity
        .as_ref()
        .context("daemon control was not completely persisted")?;
    if identity != *expected_identity
        || exact_file_identity(&control.file.metadata()?) != *expected_identity
    {
        bail!("daemon control entry or held file identity changed");
    }
    require_persisted_control_bytes(control)
}

#[cfg(unix)]
fn rollback_spawned_daemon_controls(persisted: &PersistedDaemonControls) -> Result<()> {
    rollback_spawned_daemon_controls_with_hook(persisted, |_| Ok(()))
}

#[cfg(unix)]
fn rollback_spawned_daemon_controls_with_hook(
    persisted: &PersistedDaemonControls,
    mut after_validation_before_removal: impl FnMut(&PersistedDaemonControl) -> Result<()>,
) -> Result<()> {
    // POSIX unlink is name-based: a same-owner replacement can always land after the final
    // identity check. There is no compare-(device,inode)-and-unlink primitive on macOS or Linux,
    // so automatic rollback must preserve every control and fail closed.
    let mut failures = Vec::new();
    if require_persisted_daemon_directory_identity(persisted).is_err() {
        failures.push(format!(
            "pinned daemon control directory changed; controls preserved in {}",
            persisted.directory_path.display()
        ));
    }

    if failures.is_empty() {
        for control in persisted.controls.iter().rev() {
            if let Err(error) = require_persisted_control_unchanged(persisted, control) {
                failures.push(format!(
                    "changed daemon control {} was preserved: {error}",
                    control.path.display()
                ));
                continue;
            }
            if let Err(error) = after_validation_before_removal(control) {
                failures.push(format!(
                    "daemon control {} was preserved after the pre-removal check failed: {error}",
                    control.path.display()
                ));
                continue;
            }
            if let Err(error) = require_persisted_control_unchanged(persisted, control) {
                failures.push(format!(
                    "daemon control {} changed after validation and was preserved: {error}",
                    control.path.display()
                ));
            }
        }
    }
    failures.push(format!(
        "automatic daemon-control removal is disabled because this platform provides no atomic \
         compare-identity-and-unlink operation; controls were preserved in {} for explicit \
         operator recovery after process termination is independently established",
        persisted.directory_path.display()
    ));
    if let Err(error) = persisted.directory.sync_all() {
        failures.push(format!(
            "failed to sync pinned daemon control directory {}: {error}",
            persisted.directory_path.display()
        ));
    }
    if failures.is_empty() {
        Ok(())
    } else {
        bail!(failures.join("; "))
    }
}

#[cfg(not(unix))]
fn rollback_spawned_daemon_controls(persisted: &PersistedDaemonControls) -> Result<()> {
    let controls = persisted
        .controls
        .iter()
        .map(|control| {
            format!(
                "{}:{}:{}",
                control.name,
                control.path.display(),
                control.expected_bytes.len()
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    bail!(
        "Exact replacement-safe daemon-control rollback is unavailable on this platform; \
         created controls were preserved in {}: [{}]",
        persisted.directory_path.display(),
        controls
    )
}

#[cfg(unix)]
fn sync_persisted_daemon_directory(persisted: &PersistedDaemonControls) -> Result<()> {
    validate_persisted_daemon_controls(persisted)?;
    persisted.directory.sync_all().with_context(|| {
        format!(
            "Failed to sync pinned daemon control directory {}",
            persisted.directory_path.display()
        )
    })
}

#[cfg(unix)]
fn validate_persisted_daemon_controls(persisted: &PersistedDaemonControls) -> Result<()> {
    require_persisted_daemon_directory_identity(persisted)?;
    for control in &persisted.controls {
        require_persisted_control_unchanged(persisted, control)?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_persisted_daemon_controls(_persisted: &PersistedDaemonControls) -> Result<()> {
    Ok(())
}

#[cfg(not(unix))]
fn sync_persisted_daemon_directory(persisted: &PersistedDaemonControls) -> Result<()> {
    fs::File::open(&persisted.directory_path)
        .with_context(|| {
            format!(
                "Failed to open directory {} for sync",
                persisted.directory_path.display()
            )
        })?
        .sync_all()
        .with_context(|| {
            format!(
                "Failed to sync directory {}",
                persisted.directory_path.display()
            )
        })
}

#[cfg(test)]
fn write_daemon_metadata_file(path: &Path, metadata: &DaemonSpawnMetadata) -> Result<()> {
    let contents = serde_json::to_string_pretty(metadata)
        .context("Failed to serialize daemon spawn metadata")?;
    fs::write(path, contents).context("Failed to write daemon spawn metadata")?;
    engram_store::ensure_private_file(path)
        .context("Failed to restrict daemon spawn metadata permissions")?;
    Ok(())
}

fn ensure_private_daemon_paths(config: &DaemonConfig) -> Result<()> {
    engram_store::ensure_private_directory(&config.daemon_dir())
        .context("Failed to restrict daemon directory permissions")?;
    for path in [
        config.port_file(),
        config.pid_file(),
        config.log_file(),
        config.metadata_file(),
        config.token_file(),
        config.start_lock_file(),
    ] {
        if path.exists() {
            engram_store::ensure_private_file(&path).with_context(|| {
                format!(
                    "Failed to restrict daemon file permissions: {}",
                    path.display()
                )
            })?;
        }
    }
    Ok(())
}

fn load_or_create_daemon_token(config: &DaemonConfig) -> Result<String> {
    load_or_create_token_file(&config.token_file())
}

fn load_or_create_token_file(path: &Path) -> Result<String> {
    if let Some(token) = read_token_file(path)? {
        return Ok(token);
    }

    let seed = format!(
        "{}:{}",
        engram_core::id::Id::new(),
        engram_core::id::Id::new()
    );
    let token = format!("{:x}", Sha256::digest(seed.as_bytes()));
    fs::write(path, &token).context("Failed to write daemon bearer token")?;
    engram_store::ensure_private_file(path)
        .context("Failed to restrict daemon bearer token permissions")?;
    Ok(token)
}

/// Read the local bearer token for an existing daemon.
pub fn read_daemon_token(config: &DaemonConfig) -> Result<Option<String>> {
    read_token_file(&config.token_file())
}

fn read_token_file(path: &Path) -> Result<Option<String>> {
    let token = match fs::read_to_string(path) {
        Ok(token) => token,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).context("Failed to read daemon bearer token"),
    };
    let token = token.trim();
    if token.is_empty() {
        bail!("Daemon bearer token file is empty")
    }
    Ok(Some(token.to_string()))
}

fn read_daemon_metadata(config: &DaemonConfig) -> Option<DaemonSpawnMetadata> {
    read_daemon_metadata_file(&config.metadata_file())
}

fn read_daemon_metadata_file(path: &Path) -> Option<DaemonSpawnMetadata> {
    let contents = fs::read_to_string(path).ok()?;
    serde_json::from_str(&contents).ok()
}

fn require_no_unresolved_daemon_controls(daemon_directory: &Path) -> Result<()> {
    let present = present_daemon_control_names(daemon_directory)?;
    if !present.is_empty() {
        bail!(
            "Existing daemon identity controls may belong to a live or unreaped process; refusing \
             automatic cleanup or a second daemon start without positive process identity and \
             termination authority: {}",
            present.join(", ")
        );
    }
    Ok(())
}

fn existing_controls_require_read_only_resolution(daemon_directory: &Path) -> Result<bool> {
    let present = present_daemon_control_names(daemon_directory)?;
    if present.is_empty() {
        return Ok(false);
    }
    require_absent_or_complete_daemon_controls(daemon_directory)?;
    Ok(true)
}

fn require_absent_or_complete_daemon_controls(daemon_directory: &Path) -> Result<()> {
    let present = present_daemon_control_names(daemon_directory)?;
    if !present.is_empty() && present.len() != 3 {
        bail!(
            "Partial daemon identity controls may belong to a live or unreaped process; refusing \
             to inspect, repair, or start a daemon until process identity and termination are \
             independently established: {}",
            present.join(", ")
        );
    }
    Ok(())
}

fn present_daemon_control_names(daemon_directory: &Path) -> Result<Vec<&'static str>> {
    let mut present = Vec::new();
    for name in ["daemon.pid", "daemon.port", "daemon.meta.json"] {
        match fs::symlink_metadata(daemon_directory.join(name)) {
            Ok(_) => present.push(name),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| {
                    format!(
                        "Failed to inspect unresolved daemon control {}",
                        daemon_directory.join(name).display()
                    )
                })
            }
        }
    }
    Ok(present)
}

/// Clean up daemon files.
fn cleanup_daemon_files(config: &DaemonConfig) -> Result<()> {
    let _ = fs::remove_file(config.port_file());
    let _ = fs::remove_file(config.pid_file());
    let _ = fs::remove_file(config.metadata_file());
    Ok(())
}

/// Check if a daemon is running for the given config.
#[allow(dead_code)]
pub async fn is_daemon_running(config: &DaemonConfig) -> bool {
    if let Ok(info) = get_daemon_info(config).await {
        info.healthy
    } else {
        false
    }
}

/// Get the MCP endpoint URL for a daemon.
#[allow(dead_code)]
pub fn daemon_mcp_url(port: u16) -> String {
    format!("http://127.0.0.1:{}/mcp", port)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    use std::collections::BTreeMap;
    #[cfg(unix)]
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    #[cfg(unix)]
    struct ExactConnectionFixture {
        _root: tempfile::TempDir,
        state_root: PathBuf,
        project: String,
        daemon_directory: PathBuf,
        executable: PathBuf,
        port: u16,
        pid: u32,
    }

    #[cfg(unix)]
    fn write_private(path: &Path, contents: impl AsRef<[u8]>) {
        fs::write(path, contents).expect("write private fixture");
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
            .expect("set private fixture mode");
    }

    #[cfg(unix)]
    fn exact_connection_fixture(port: u16, pid: u32) -> ExactConnectionFixture {
        let root = tempfile::tempdir().expect("temp dir");
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700))
            .expect("set state root mode");
        let state_root = root.path().canonicalize().expect("canonical state root");
        let projects = state_root.join("projects");
        fs::create_dir(&projects).expect("create projects directory");
        fs::set_permissions(&projects, fs::Permissions::from_mode(0o700))
            .expect("set projects directory mode");
        let project = "exact-fixture".to_string();
        let daemon_directory = projects.join(&project);
        fs::create_dir(&daemon_directory).expect("create daemon directory");
        fs::set_permissions(&daemon_directory, fs::Permissions::from_mode(0o700))
            .expect("set daemon directory mode");
        let daemon_directory = daemon_directory
            .canonicalize()
            .expect("canonical daemon directory");
        let executable = root.path().join("engram-fixture");
        fs::write(&executable, b"exact executable bytes").expect("write executable fixture");
        let executable = executable
            .canonicalize()
            .expect("canonical fixture executable");
        let metadata = serde_json::json!({
            "schema_version": DAEMON_SPAWN_METADATA_SCHEMA_VERSION,
            "executable_path": executable.display().to_string(),
            "executable_version": env!("CARGO_PKG_VERSION"),
            "executable_sha256": executable_sha256(&executable).expect("hash fixture executable"),
            "pid": pid,
            "port": port,
        });
        write_private(&daemon_directory.join("daemon.port"), format!("{port}\n"));
        write_private(&daemon_directory.join("daemon.pid"), format!("{pid}\n"));
        write_private(
            &daemon_directory.join("daemon.meta.json"),
            serde_json::to_vec(&metadata).expect("serialize metadata fixture"),
        );
        write_private(
            &daemon_directory.join("daemon.token"),
            b"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        );
        ExactConnectionFixture {
            _root: root,
            state_root,
            project,
            daemon_directory,
            executable,
            port,
            pid,
        }
    }

    #[cfg(unix)]
    async fn connect_fixture(fixture: &ExactConnectionFixture) -> Result<ExistingDaemonTransport> {
        connect_existing_exact_transport_at(
            &fixture.state_root,
            &fixture.project,
            fixture.port,
            fixture.pid,
            &fixture.executable,
        )
        .await
    }

    #[cfg(unix)]
    fn exact_health(pid: u32) -> serde_json::Value {
        serde_json::json!({
            "status": "ok",
            "service": "engram",
            "version": env!("CARGO_PKG_VERSION"),
            "build_sha": option_env!("ENGRAM_BUILD_SHA"),
            "pid": pid,
            "health_schema_version": engram_mcp::server::DAEMON_HEALTH_SCHEMA_VERSION,
            "mcp_contract_version": engram_mcp::server::MCP_CONTRACT_VERSION,
            "mcp_tools_sha256": engram_mcp::server::mcp_tools_sha256(),
            "mcp_protocol_version": engram_mcp::server::MCP_PROTOCOL_VERSION,
            "auth_required": true,
            "storage_status": "ready",
            "storage_ready": true,
            "storage_available_bytes": 2_000_000_000_u64,
            "storage_required_bytes": 1_000_000_000_u64,
        })
    }

    #[cfg(unix)]
    async fn start_fake_health_server(
        body: serde_json::Value,
    ) -> (u16, tokio::task::JoinHandle<()>) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .expect("bind fake health server");
        let port = listener.local_addr().expect("fake server address").port();
        let response_body = serde_json::to_vec(&body).expect("serialize fake health");
        let handle = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept health request");
            let mut request = Vec::new();
            let mut byte = [0_u8; 1];
            while !request.ends_with(b"\r\n\r\n") {
                stream
                    .read_exact(&mut byte)
                    .await
                    .expect("read health request");
                request.push(byte[0]);
                assert!(request.len() <= 16 * 1024);
            }
            assert!(request.starts_with(b"GET /health HTTP/1.1\r\n"));
            let header = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                response_body.len()
            );
            stream
                .write_all(header.as_bytes())
                .await
                .expect("write health header");
            stream
                .write_all(&response_body)
                .await
                .expect("write health body");
        });
        (port, handle)
    }

    #[cfg(unix)]
    fn exact_control_snapshot(directory: &Path) -> BTreeMap<String, (Vec<u8>, u32, u64, u64)> {
        fs::read_dir(directory)
            .expect("read control directory")
            .map(|entry| {
                let entry = entry.expect("read control entry");
                let path = entry.path();
                let metadata = fs::symlink_metadata(&path).expect("inspect control entry");
                (
                    entry.file_name().to_string_lossy().into_owned(),
                    (
                        fs::read(&path).expect("read control entry"),
                        metadata.permissions().mode() & 0o777,
                        metadata.ino(),
                        metadata.nlink(),
                    ),
                )
            })
            .collect()
    }

    #[cfg(unix)]
    #[test]
    fn spawned_child_liveness_accepts_running_child() {
        let mut child = Command::new("/bin/sh")
            .args(["-c", "sleep 5"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn sleep child");

        assert!(ensure_spawned_child_still_running(&mut child, None).is_ok());

        let _ = child.kill();
        let _ = child.wait();
    }

    #[cfg(unix)]
    #[test]
    fn spawned_child_liveness_rejects_exited_child() {
        let mut child = Command::new("/bin/sh")
            .args(["-c", "exit 7"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn exiting child");

        let _ = child.wait();

        let error = ensure_spawned_child_still_running(&mut child, None)
            .expect_err("exited child should fail liveness check");
        assert!(error
            .to_string()
            .contains("exited before daemon readiness was confirmed"));
    }

    #[cfg(unix)]
    #[test]
    fn unready_spawned_child_is_terminated_and_reaped() {
        let mut child = Command::new("/bin/sh")
            .args(["-c", "sleep 5"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn sleep child");

        terminate_spawned_child(&mut child).expect("terminate unready child");

        assert!(child.try_wait().unwrap().is_some());
    }

    #[cfg(unix)]
    #[test]
    fn daemon_identity_persistence_failures_terminate_reap_and_fence_retry() {
        let cases: &[(DaemonControlPersistencePhase, &[&str])] = &[
            (DaemonControlPersistencePhase::Pid, &["daemon.pid"]),
            (
                DaemonControlPersistencePhase::Port,
                &["daemon.pid", "daemon.port"],
            ),
            (
                DaemonControlPersistencePhase::Metadata,
                &["daemon.pid", "daemon.port", "daemon.meta.json"],
            ),
        ];
        for &(failure_phase, expected_controls) in cases {
            let directory = tempfile::tempdir().expect("daemon control directory");
            let mut child = Command::new("/bin/sh")
                .args(["-c", "sleep 5"])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .expect("spawn sleep child");
            let pid = child.id();
            let metadata = DaemonSpawnMetadata::new(Path::new("/bin/sh"), pid, 9876);

            let error = persist_spawned_daemon_info_with_hook(
                directory.path(),
                &mut child,
                9876,
                pid,
                &metadata,
                |phase, _| {
                    if phase == failure_phase {
                        bail!("injected {phase:?} control write failure")
                    }
                    Ok(())
                },
            )
            .err()
            .expect("control write failure must abort startup");

            assert!(error.to_string().contains("terminated and reaped"));
            assert!(error.to_string().contains("controls were preserved"));
            assert!(child.try_wait().expect("inspect child").is_some());
            for name in ["daemon.pid", "daemon.port", "daemon.meta.json"] {
                assert_eq!(
                    directory.path().join(name).exists(),
                    expected_controls.contains(&name),
                    "unexpected control state for {name} after {failure_phase:?}"
                );
            }
            let before_retry = exact_control_snapshot(directory.path());
            let retry_error = require_no_unresolved_daemon_controls(directory.path())
                .expect_err("preserved controls must fence a retry");
            assert!(retry_error
                .to_string()
                .contains("refusing automatic cleanup or a second daemon start"));
            assert_eq!(exact_control_snapshot(directory.path()), before_retry);
        }
    }

    #[cfg(unix)]
    #[test]
    fn daemon_identity_rollback_rejects_entry_replacement_without_deleting_it() {
        let cases = [
            (DaemonControlPersistencePhase::Pid, "daemon.pid"),
            (DaemonControlPersistencePhase::Port, "daemon.port"),
            (DaemonControlPersistencePhase::Metadata, "daemon.meta.json"),
        ];
        let replacement = b"replacement-not-created-for-child";
        for (failure_phase, target_name) in cases {
            let directory = tempfile::tempdir().expect("daemon control directory");
            let mut child = Command::new("/bin/sh")
                .args(["-c", "sleep 5"])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .expect("spawn sleep child");
            let pid = child.id();
            let metadata = DaemonSpawnMetadata::new(Path::new("/bin/sh"), pid, 9876);

            let error = persist_spawned_daemon_info_with_hook(
                directory.path(),
                &mut child,
                9876,
                pid,
                &metadata,
                |phase, path| {
                    if phase == failure_phase {
                        fs::remove_file(path)?;
                        write_private(path, replacement);
                    }
                    Ok(())
                },
            )
            .err()
            .expect("replacement race must abort startup");

            assert!(error.to_string().contains("controls were preserved"));
            assert!(child.try_wait().expect("inspect child").is_some());
            assert_eq!(
                fs::read(directory.path().join(target_name)).expect("replacement survives"),
                replacement
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn daemon_identity_rollback_rejects_parent_replacement_and_preserves_controls() {
        for failure_phase in [
            DaemonControlPersistencePhase::Pid,
            DaemonControlPersistencePhase::Port,
            DaemonControlPersistencePhase::Metadata,
        ] {
            let directory = tempfile::tempdir().expect("daemon control directory");
            let directory_path = directory.path().to_path_buf();
            let moved_path = directory_path.parent().expect("temp parent").join(format!(
                "{}-held",
                directory_path
                    .file_name()
                    .expect("temp directory name")
                    .to_string_lossy()
            ));
            let mut child = Command::new("/bin/sh")
                .args(["-c", "sleep 5"])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .expect("spawn sleep child");
            let pid = child.id();
            let metadata = DaemonSpawnMetadata::new(Path::new("/bin/sh"), pid, 9876);

            let error = persist_spawned_daemon_info_with_hook(
                &directory_path,
                &mut child,
                9876,
                pid,
                &metadata,
                |phase, _| {
                    if phase == failure_phase {
                        fs::rename(&directory_path, &moved_path)?;
                        fs::create_dir(&directory_path)?;
                        fs::set_permissions(&directory_path, fs::Permissions::from_mode(0o700))?;
                    }
                    Ok(())
                },
            )
            .err()
            .expect("parent ABA must abort persistence");

            assert!(error
                .to_string()
                .contains("pinned daemon control directory changed"));
            assert!(child.try_wait().expect("inspect child").is_some());
            fs::remove_dir(&directory_path).expect("remove adversarial replacement");
            fs::rename(&moved_path, &directory_path).expect("restore held fixture directory");
            assert!(directory_path.join("daemon.pid").exists());
        }
    }

    #[cfg(unix)]
    #[test]
    fn daemon_persistence_allows_trusted_entries_without_losing_directory_binding() {
        let directory = tempfile::tempdir().expect("daemon control directory");
        fs::create_dir(directory.path().join("data")).expect("create trusted data directory");
        write_private(&directory.path().join("daemon.log"), b"");
        let mut persisted = begin_spawned_daemon_control_transaction(directory.path())
            .expect("begin control transaction");
        create_spawned_daemon_control(&mut persisted, "daemon.pid", b"startup-pending\n".to_vec())
            .expect("create pending PID control");
        fs::write(directory.path().join("daemon.log"), b"trusted log output")
            .expect("write trusted log output");
        require_persisted_daemon_directory_identity(&persisted)
            .expect("trusted entry creation must preserve the pinned directory binding");
    }

    #[cfg(unix)]
    #[test]
    fn daemon_persistence_rejects_unexpected_directory_link_count_change() {
        let directory = tempfile::tempdir().expect("daemon control directory");
        let persisted = begin_spawned_daemon_control_transaction(directory.path())
            .expect("begin control transaction");

        fs::create_dir(directory.path().join("unexpected"))
            .expect("create unexpected directory entry");
        let error = require_persisted_daemon_directory_identity(&persisted)
            .expect_err("unexpected link-count change must invalidate the transaction");
        assert!(error
            .to_string()
            .contains("Pinned daemon control directory changed"));
    }

    #[cfg(unix)]
    #[test]
    fn daemon_identity_rollback_rejects_same_inode_byte_mutation() {
        let cases = [
            (DaemonControlPersistencePhase::Pid, "daemon.pid"),
            (DaemonControlPersistencePhase::Port, "daemon.port"),
            (DaemonControlPersistencePhase::Metadata, "daemon.meta.json"),
        ];
        let mutation = b"same-inode-mutation";
        for (failure_phase, target_name) in cases {
            let directory = tempfile::tempdir().expect("daemon control directory");
            let mut child = Command::new("/bin/sh")
                .args(["-c", "sleep 5"])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .expect("spawn sleep child");
            let pid = child.id();
            let metadata = DaemonSpawnMetadata::new(Path::new("/bin/sh"), pid, 9876);

            let error = persist_spawned_daemon_info_with_hook(
                directory.path(),
                &mut child,
                9876,
                pid,
                &metadata,
                |phase, path| {
                    if phase == failure_phase {
                        let mut file = fs::OpenOptions::new()
                            .write(true)
                            .truncate(true)
                            .open(path)?;
                        file.write_all(mutation)?;
                        file.sync_all()?;
                    }
                    Ok(())
                },
            )
            .err()
            .expect("same-inode mutation must abort persistence");

            assert!(error.to_string().contains("changed daemon control"));
            assert!(error.to_string().contains("preserved"));
            assert!(child.try_wait().expect("inspect child").is_some());
            assert_eq!(
                fs::read(directory.path().join(target_name)).expect("mutated control survives"),
                mutation
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn daemon_identity_controls_survive_unproven_child_termination_or_reap() {
        for persistence_phase in [
            DaemonControlPersistencePhase::Pid,
            DaemonControlPersistencePhase::Port,
            DaemonControlPersistencePhase::Metadata,
        ] {
            for termination_phase in [
                ChildTerminationPhase::TryWait,
                ChildTerminationPhase::Kill,
                ChildTerminationPhase::Wait,
            ] {
                let directory = tempfile::tempdir().expect("daemon control directory");
                let mut child = Command::new("/bin/sh")
                    .args(["-c", "sleep 5"])
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .expect("spawn sleep child");
                let pid = child.id();
                let metadata = DaemonSpawnMetadata::new(Path::new("/bin/sh"), pid, 9876);

                let error = persist_spawned_daemon_info_with_hooks(
                    directory.path(),
                    &mut child,
                    9876,
                    pid,
                    &metadata,
                    |phase, _| {
                        if phase == persistence_phase {
                            bail!("injected {phase:?} persistence failure")
                        }
                        Ok(())
                    },
                    |phase| {
                        if phase == termination_phase {
                            bail!("injected {phase:?} termination failure")
                        }
                        Ok(())
                    },
                )
                .err()
                .expect("unproven cleanup must fail closed");

                assert!(error.to_string().contains("controls were preserved"));
                let before_retry = exact_control_snapshot(directory.path());
                assert!(!before_retry.is_empty());
                let retry_error = if before_retry.len() == 3 {
                    require_no_unresolved_daemon_controls(directory.path())
                } else {
                    require_absent_or_complete_daemon_controls(directory.path())
                }
                .expect_err("retry must stop before cleanup or spawn");
                assert!(retry_error.to_string().contains("live or unreaped process"));
                assert_eq!(exact_control_snapshot(directory.path()), before_retry);
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn daemon_identity_rollback_preserves_swaps_after_validation_for_every_control() {
        for target_name in ["daemon.pid", "daemon.port", "daemon.meta.json"] {
            let directory = tempfile::tempdir().expect("daemon control directory");
            let mut persisted = prepare_spawned_daemon_control_transaction(directory.path())
                .expect("prepare control transaction");
            let metadata = DaemonSpawnMetadata::new(Path::new("/bin/sh"), 12345, 9876);
            write_spawned_daemon_controls(9876, 12345, &metadata, &mut persisted, &mut |_, _| {
                Ok(())
            })
            .expect("write exact controls");
            let before = exact_control_snapshot(directory.path());
            let replacement = format!("replacement-after-validation:{target_name}").into_bytes();
            let mut replacement_identity = None;

            let error = rollback_spawned_daemon_controls_with_hook(&persisted, |control| {
                if control.name == target_name {
                    fs::remove_file(&control.path)?;
                    write_private(&control.path, &replacement);
                    replacement_identity = Some(exact_file_identity(
                        &fs::symlink_metadata(&control.path)
                            .context("inspect injected replacement")?,
                    ));
                }
                Ok(())
            })
            .expect_err("rollback cannot safely remove pathname controls");

            assert!(error
                .to_string()
                .contains("changed after validation and was preserved"));
            assert_eq!(
                fs::read(directory.path().join(target_name)).expect("replacement survives"),
                replacement
            );
            assert_eq!(
                exact_file_identity(
                    &fs::symlink_metadata(directory.path().join(target_name))
                        .expect("inspect surviving replacement")
                ),
                replacement_identity.expect("replacement identity recorded")
            );
            for (name, snapshot) in &before {
                if name != target_name {
                    let after = exact_control_snapshot(directory.path());
                    assert_eq!(after.get(name), Some(snapshot));
                }
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn daemon_startup_preparation_fences_retry_before_child_spawn() {
        let directory = tempfile::tempdir().expect("daemon control directory");
        let prepared = prepare_spawned_daemon_control_transaction(directory.path())
            .expect("prepare durable startup control");
        drop(prepared);

        assert_eq!(
            fs::read(directory.path().join("daemon.pid")).expect("read pending PID control"),
            b"startup-pending\n"
        );
        let before_retry = exact_control_snapshot(directory.path());
        let error = require_absent_or_complete_daemon_controls(directory.path())
            .expect_err("pending pre-spawn control must fence a retry");
        assert!(error.to_string().contains("live or unreaped process"));
        assert_eq!(exact_control_snapshot(directory.path()), before_retry);
    }

    #[cfg(unix)]
    #[test]
    fn every_control_publication_failure_preserves_a_durable_retry_fence() {
        for publication_phase in [
            DaemonControlPublicationPhase::BeforeParentSync,
            DaemonControlPublicationPhase::AfterParentSyncBeforeWrite,
        ] {
            for target_name in ["daemon.pid", "daemon.port", "daemon.meta.json"] {
                let directory = tempfile::tempdir().expect("daemon control directory");
                let mut persisted = begin_spawned_daemon_control_transaction(directory.path())
                    .expect("begin control transaction");
                if target_name != "daemon.pid" {
                    create_spawned_daemon_control(
                        &mut persisted,
                        "daemon.pid",
                        b"startup-pending\n".to_vec(),
                    )
                    .expect("create pending PID control");
                }
                if target_name == "daemon.meta.json" {
                    create_spawned_daemon_control(
                        &mut persisted,
                        "daemon.port",
                        b"9876\n".to_vec(),
                    )
                    .expect("create port control");
                }

                let error = create_spawned_daemon_control_with_hook(
                    &mut persisted,
                    target_name,
                    b"unwritten".to_vec(),
                    |phase, _| {
                        if phase == publication_phase {
                            bail!("injected {phase:?} publication failure")
                        }
                        Ok(())
                    },
                )
                .expect_err("pre-write failure must escape after durable name publication");
                assert!(format!("{error:#}").contains("injected"));
                drop(persisted);

                assert_eq!(
                    fs::metadata(directory.path().join(target_name))
                        .expect("durably published control remains")
                        .len(),
                    0
                );
                let before_retry = exact_control_snapshot(directory.path());
                let retry_error = if before_retry.len() == 3 {
                    require_no_unresolved_daemon_controls(directory.path())
                } else {
                    require_absent_or_complete_daemon_controls(directory.path())
                }
                .expect_err("published control must fence retry");
                assert!(retry_error.to_string().contains("live or unreaped process"));
                assert_eq!(exact_control_snapshot(directory.path()), before_retry);
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn initial_partial_control_preflight_is_read_only() {
        let root = tempfile::tempdir().expect("preflight root");
        let directory = root.path().join("real-daemon-directory");
        fs::create_dir(&directory).expect("create daemon control directory");
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o755))
            .expect("set deliberately nonprivate directory mode");
        let daemon_link = root.path().join("daemon-directory-link");
        std::os::unix::fs::symlink(&directory, &daemon_link)
            .expect("create swapped daemon-directory symlink");
        let pid_path = directory.join("daemon.pid");
        fs::write(&pid_path, b"123\n").expect("write partial PID control");
        fs::set_permissions(&pid_path, fs::Permissions::from_mode(0o644))
            .expect("set deliberately nonprivate control mode");
        let directory_mode = fs::symlink_metadata(&directory)
            .expect("inspect directory")
            .permissions()
            .mode()
            & 0o777;
        let control_mode = fs::symlink_metadata(&pid_path)
            .expect("inspect PID control")
            .permissions()
            .mode()
            & 0o777;

        let error = existing_controls_require_read_only_resolution(&daemon_link)
            .expect_err("partial controls must stop before permission repair");

        assert!(error.to_string().contains("live or unreaped process"));
        assert!(fs::symlink_metadata(&daemon_link)
            .expect("reinspect daemon link")
            .file_type()
            .is_symlink());
        assert_eq!(
            fs::symlink_metadata(&directory)
                .expect("reinspect directory")
                .permissions()
                .mode()
                & 0o777,
            directory_mode
        );
        assert_eq!(
            fs::symlink_metadata(&pid_path)
                .expect("reinspect PID control")
                .permissions()
                .mode()
                & 0o777,
            control_mode
        );
    }

    #[test]
    fn daemon_start_fence_allows_only_absent_or_complete_control_sets() {
        let directory = tempfile::tempdir().expect("daemon control directory");
        require_absent_or_complete_daemon_controls(directory.path())
            .expect("empty control set may proceed to ordinary startup");

        fs::write(directory.path().join("daemon.pid"), b"123\n").expect("write PID control");
        assert!(require_absent_or_complete_daemon_controls(directory.path()).is_err());

        fs::write(directory.path().join("daemon.port"), b"9876\n").expect("write port control");
        assert!(require_absent_or_complete_daemon_controls(directory.path()).is_err());

        fs::write(directory.path().join("daemon.meta.json"), b"{}")
            .expect("write metadata control");
        require_absent_or_complete_daemon_controls(directory.path())
            .expect("complete controls may be inspected for exact healthy reuse");
        assert!(require_no_unresolved_daemon_controls(directory.path()).is_err());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn exact_existing_transport_is_consumed_without_authority_or_persistence() {
        let pid = std::process::id();
        let (port, server) = start_fake_health_server(exact_health(pid)).await;
        let fixture = exact_connection_fixture(port, pid);
        let before = exact_control_snapshot(&fixture.daemon_directory);

        let connection = connect_fixture(&fixture)
            .await
            .expect("connect to exact fake daemon");
        server.await.expect("fake health server completed");

        let (connection_port, auth_token) = connection.into_proxy_parts();
        assert_eq!(connection_port, fixture.port);
        assert_eq!(
            auth_token,
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        );
        assert_eq!(exact_control_snapshot(&fixture.daemon_directory), before);
        let error = fetch_exact_daemon_health(port)
            .await
            .expect_err("disappeared daemon must remain unavailable");
        assert!(!error.to_string().is_empty());
        assert_eq!(exact_control_snapshot(&fixture.daemon_directory), before);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn exact_existing_connection_rejects_invalid_controls_without_mutation() {
        let fixture = exact_connection_fixture(9, std::process::id());
        write_private(&fixture.daemon_directory.join("daemon.port"), b"9 \n");
        let before = exact_control_snapshot(&fixture.daemon_directory);

        let error = connect_fixture(&fixture)
            .await
            .err()
            .expect("non-exact control must fail before connecting");

        assert!(error.to_string().contains("exact decimal line"));
        assert_eq!(exact_control_snapshot(&fixture.daemon_directory), before);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn exact_existing_connection_rejects_partial_controls_without_cleanup() {
        let fixture = exact_connection_fixture(9, std::process::id());
        fs::remove_file(fixture.daemon_directory.join("daemon.meta.json"))
            .expect("remove metadata fixture");
        let before = exact_control_snapshot(&fixture.daemon_directory);

        let error = connect_fixture(&fixture)
            .await
            .err()
            .expect("partial controls must fail");

        assert!(error.to_string().contains("daemon spawn metadata"));
        assert_eq!(exact_control_snapshot(&fixture.daemon_directory), before);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn exact_existing_connection_rejects_stale_metadata_without_mutation() {
        let fixture = exact_connection_fixture(9, std::process::id());
        let path = fixture.daemon_directory.join("daemon.meta.json");
        let mut metadata: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).expect("read metadata fixture"))
                .expect("parse metadata fixture");
        metadata["executable_sha256"] = serde_json::json!("stale");
        write_private(
            &path,
            serde_json::to_vec(&metadata).expect("serialize metadata"),
        );
        let before = exact_control_snapshot(&fixture.daemon_directory);

        let error = connect_fixture(&fixture)
            .await
            .err()
            .expect("stale metadata must fail");

        assert!(error.to_string().contains("spawn metadata does not match"));
        assert_eq!(exact_control_snapshot(&fixture.daemon_directory), before);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn exact_existing_connection_rejects_nonprivate_token_without_chmod() {
        let fixture = exact_connection_fixture(9, std::process::id());
        let token = fixture.daemon_directory.join("daemon.token");
        fs::set_permissions(&token, fs::Permissions::from_mode(0o644))
            .expect("make token nonprivate");
        let before = exact_control_snapshot(&fixture.daemon_directory);

        let error = connect_fixture(&fixture)
            .await
            .err()
            .expect("nonprivate token must fail");

        assert!(error.to_string().contains("mode-0600"));
        assert_eq!(exact_control_snapshot(&fixture.daemon_directory), before);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn exact_existing_connection_rejects_noncanonical_token_bytes() {
        for token_bytes in [
            b"short".as_slice(),
            b"0123456789ABCDEF0123456789abcdef0123456789abcdef0123456789abcdef".as_slice(),
        ] {
            let fixture = exact_connection_fixture(9, std::process::id());
            write_private(&fixture.daemon_directory.join("daemon.token"), token_bytes);
            let before = exact_control_snapshot(&fixture.daemon_directory);

            let error = connect_fixture(&fixture)
                .await
                .err()
                .expect("noncanonical token must fail before connecting");

            assert!(error.to_string().contains("exact generated token format"));
            assert_eq!(exact_control_snapshot(&fixture.daemon_directory), before);
        }
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn exact_existing_connection_rejects_replaced_control_paths() {
        let fixture = exact_connection_fixture(9, std::process::id());
        let token = fixture.daemon_directory.join("daemon.token");
        let replacement = fixture.daemon_directory.join("replacement.token");
        fs::rename(&token, &replacement).expect("move original token fixture");
        std::os::unix::fs::symlink(&replacement, &token).expect("replace token with symlink");
        let before = exact_control_snapshot(&fixture.daemon_directory);

        let error = connect_fixture(&fixture)
            .await
            .err()
            .expect("replaced token path must fail");

        assert!(error.to_string().contains("not a symlink"));
        assert_eq!(exact_control_snapshot(&fixture.daemon_directory), before);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn exact_existing_connection_rejects_inexact_health_without_mutation() {
        let pid = std::process::id();
        let mut health = exact_health(pid);
        health["auth_required"] = serde_json::json!(false);
        let (port, server) = start_fake_health_server(health).await;
        let fixture = exact_connection_fixture(port, pid);
        let before = exact_control_snapshot(&fixture.daemon_directory);

        let error = connect_fixture(&fixture)
            .await
            .err()
            .expect("unauthenticated health must fail");
        server.await.expect("fake health server completed");

        assert!(error.to_string().contains("not ready and exact"));
        assert_eq!(exact_control_snapshot(&fixture.daemon_directory), before);
    }

    #[cfg(unix)]
    #[test]
    fn exact_health_validation_rejects_each_required_contract_class() {
        let pid = std::process::id();
        let valid: ExactDaemonHealth =
            serde_json::from_value(exact_health(pid)).expect("parse valid health fixture");
        validate_exact_daemon_health(&valid, pid).expect("valid exact health");

        let mut wrong_pid = exact_health(pid);
        wrong_pid["pid"] = serde_json::json!(pid.saturating_add(1));
        let wrong_pid = serde_json::from_value(wrong_pid).expect("parse wrong PID health");
        assert!(validate_exact_daemon_health(&wrong_pid, pid).is_err());

        let mut wrong_storage = exact_health(pid);
        wrong_storage["storage_available_bytes"] = serde_json::json!(1_u64);
        let wrong_storage = serde_json::from_value(wrong_storage).expect("parse storage health");
        assert!(validate_exact_daemon_health(&wrong_storage, pid).is_err());

        let mut wrong_schema = exact_health(pid);
        wrong_schema["health_schema_version"] = serde_json::json!(0);
        let wrong_schema = serde_json::from_value(wrong_schema).expect("parse schema health");
        assert!(validate_exact_daemon_health(&wrong_schema, pid).is_err());

        let mut wrong_contract = exact_health(pid);
        wrong_contract["mcp_contract_version"] = serde_json::json!(0);
        let wrong_contract = serde_json::from_value(wrong_contract).expect("parse contract health");
        assert!(validate_exact_daemon_health(&wrong_contract, pid).is_err());

        let mut wrong_tools = exact_health(pid);
        wrong_tools["mcp_tools_sha256"] = serde_json::json!("wrong");
        let wrong_tools = serde_json::from_value(wrong_tools).expect("parse tools health");
        assert!(validate_exact_daemon_health(&wrong_tools, pid).is_err());

        let mut wrong_protocol = exact_health(pid);
        wrong_protocol["mcp_protocol_version"] = serde_json::json!("wrong");
        let wrong_protocol = serde_json::from_value(wrong_protocol).expect("parse protocol health");
        assert!(validate_exact_daemon_health(&wrong_protocol, pid).is_err());

        let mut wrong_auth = exact_health(pid);
        wrong_auth["auth_required"] = serde_json::json!(false);
        let wrong_auth = serde_json::from_value(wrong_auth).expect("parse auth health");
        assert!(validate_exact_daemon_health(&wrong_auth, pid).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn exact_health_shape_rejects_missing_and_additional_fields() {
        let pid = std::process::id();
        let mut missing = exact_health(pid);
        missing
            .as_object_mut()
            .expect("health object")
            .remove("build_sha");
        assert!(decode_exact_daemon_health(
            &serde_json::to_vec(&missing).expect("serialize missing health")
        )
        .is_err());

        let mut additional = exact_health(pid);
        additional["untrusted_extension"] = serde_json::json!(true);
        assert!(decode_exact_daemon_health(
            &serde_json::to_vec(&additional).expect("serialize additional health")
        )
        .is_err());
    }

    #[cfg(unix)]
    #[test]
    fn strict_metadata_and_health_json_reject_duplicates_polyglots_and_malformed_input() {
        let metadata = format!(
            concat!(
                "{{\"schema_version\":1,\"executable_path\":\"/tmp/engram\",",
                "\"executable_version\":\"{}\",\"executable_sha256\":\"abc\",",
                "\"pid\":42,\"port\":8765}}"
            ),
            env!("CARGO_PKG_VERSION")
        );
        let duplicate_metadata = metadata.replacen("\"pid\":42", "\"pid\":42,\"pid\":42", 1);
        let recursive_duplicate_metadata = metadata.replacen(
            "\"pid\":42",
            "\"untrusted\":{\"nested\":1,\"nested\":2},\"pid\":42",
            1,
        );
        let unknown_metadata = metadata.replacen("\"pid\":42", "\"untrusted\":1,\"pid\":42", 1);
        for invalid in [
            duplicate_metadata,
            recursive_duplicate_metadata,
            unknown_metadata,
            format!("{metadata}\nnull"),
            "{\"schema_version\":1".to_string(),
        ] {
            assert!(decode_strict_json::<ExactDaemonSpawnMetadata>(
                invalid.as_bytes(),
                "invalid metadata"
            )
            .is_err());
        }

        let health = serde_json::to_string(&exact_health(std::process::id()))
            .expect("serialize exact health");
        let pid_fragment = format!("\"pid\":{}", std::process::id());
        let duplicate_health =
            health.replacen(&pid_fragment, &format!("{pid_fragment},{pid_fragment}"), 1);
        let recursive_duplicate_health = health.replacen(
            &pid_fragment,
            &format!("\"untrusted\":{{\"nested\":1,\"nested\":2}},{pid_fragment}"),
            1,
        );
        for invalid in [
            duplicate_health,
            recursive_duplicate_health,
            format!("{health}[]"),
            "[".to_string(),
        ] {
            assert!(decode_exact_daemon_health(invalid.as_bytes()).is_err());
        }
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn exact_existing_connection_holds_parent_and_rejects_control_entry_aba() {
        for replace_parent in [true, false] {
            let pid = std::process::id();
            let (port, server) = start_fake_health_server(exact_health(pid)).await;
            let fixture = exact_connection_fixture(port, pid);
            let state_root = fixture.state_root.clone();
            let projects = state_root.join("projects");
            let moved_projects = state_root.join("projects-held");
            let token = fixture.daemon_directory.join("daemon.token");
            let moved_token = fixture.daemon_directory.join("daemon.token-held");

            let result = connect_existing_exact_transport_at_with_hook(
                &fixture.state_root,
                &fixture.project,
                fixture.port,
                fixture.pid,
                &fixture.executable,
                |phase| {
                    if phase != ExactTransportValidationPhase::ControlsPinned {
                        return Ok(());
                    }
                    if replace_parent {
                        fs::rename(&projects, &moved_projects)?;
                        fs::create_dir(&projects)?;
                        fs::set_permissions(&projects, fs::Permissions::from_mode(0o700))?;
                        fs::remove_dir(&projects)?;
                        fs::rename(&moved_projects, &projects)?;
                    } else {
                        fs::rename(&token, &moved_token)?;
                        write_private(
                            &token,
                            b"abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
                        );
                        fs::remove_file(&token)?;
                        fs::rename(&moved_token, &token)?;
                    }
                    Ok(())
                },
            )
            .await;
            server.await.expect("fake health server completed");

            if replace_parent {
                let (_, token) = result
                    .expect("held descriptor chain must not switch during restored parent ABA")
                    .into_proxy_parts();
                assert_eq!(
                    token,
                    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                );
            } else {
                let error = result
                    .err()
                    .expect("control-entry ABA must invalidate exact identity");
                assert!(error
                    .to_string()
                    .contains("changed during exact daemon validation"));
            }
        }
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn exact_existing_connection_rejects_symlinked_parent_component() {
        let fixture = exact_connection_fixture(9, std::process::id());
        let projects = fixture.state_root.join("projects");
        let real_projects = fixture.state_root.join("projects-real");
        fs::rename(&projects, &real_projects).expect("move projects directory");
        std::os::unix::fs::symlink(&real_projects, &projects).expect("symlink projects directory");

        let error = connect_fixture(&fixture)
            .await
            .err()
            .expect("symlinked parent must fail before network access");

        assert!(error
            .to_string()
            .contains("Failed to pin Engram projects directory"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn loopback_health_clients_bypass_poisoned_system_proxy_environment() {
        let poison = tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .expect("bind poison proxy");
        let poison_url = format!(
            "http://{}",
            poison.local_addr().expect("poison proxy address")
        );
        let poison_attempt = tokio::spawn(async move {
            tokio::time::timeout(Duration::from_millis(250), poison.accept())
                .await
                .is_ok()
        });
        let pid = std::process::id();
        let (exact_port, exact_server) = start_fake_health_server(exact_health(pid)).await;
        let (generic_port, generic_server) = start_fake_health_server(exact_health(pid)).await;
        let proxy_environment = PoisonedProxyEnvironment::install(&poison_url);

        let exact_health = fetch_exact_daemon_health(exact_port)
            .await
            .expect("exact health must connect directly to loopback");
        let generic_health = fetch_daemon_health(generic_port)
            .await
            .expect("generic health must connect directly to loopback");
        drop(proxy_environment);
        exact_server
            .await
            .expect("direct exact-health server completed");
        generic_server
            .await
            .expect("direct generic-health server completed");

        assert_eq!(exact_health.pid, pid);
        assert_eq!(generic_health.pid, Some(pid));
        assert!(!poison_attempt.await.expect("poison proxy observation"));
    }

    #[test]
    fn recent_log_tail_returns_last_lines() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("daemon.log");
        fs::write(&path, "one\ntwo\nthree\n").expect("write log");

        assert_eq!(recent_log_tail(&path, 2).as_deref(), Some("two\nthree"));
    }

    #[test]
    fn daemon_spawn_metadata_round_trips_json() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("daemon.meta.json");
        let metadata = DaemonSpawnMetadata {
            schema_version: DAEMON_SPAWN_METADATA_SCHEMA_VERSION,
            executable_path: "/tmp/engram".to_string(),
            executable_version: "0.2.0-test".to_string(),
            executable_sha256: Some("abc123".to_string()),
            pid: 42,
            port: 8765,
        };

        write_daemon_metadata_file(&path, &metadata).expect("write metadata");

        assert_eq!(read_daemon_metadata_file(&path), Some(metadata));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }

    #[test]
    fn daemon_spawn_metadata_missing_or_invalid_is_absent() {
        let dir = tempfile::tempdir().expect("temp dir");
        let missing = dir.path().join("missing.json");
        assert_eq!(read_daemon_metadata_file(&missing), None);

        let invalid = dir.path().join("invalid.json");
        fs::write(&invalid, "not json").expect("write invalid metadata");
        assert_eq!(read_daemon_metadata_file(&invalid), None);
    }

    fn current_health(pid: u32) -> DaemonHealth {
        DaemonHealth {
            status: "ok".to_string(),
            service: "engram".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            build_sha: None,
            pid: Some(pid),
            health_schema_version: Some(engram_mcp::server::DAEMON_HEALTH_SCHEMA_VERSION),
            mcp_contract_version: Some(engram_mcp::server::MCP_CONTRACT_VERSION),
            mcp_tools_sha256: Some(engram_mcp::server::mcp_tools_sha256().to_string()),
            mcp_protocol_version: Some(engram_mcp::server::MCP_PROTOCOL_VERSION.to_string()),
            auth_required: Some(true),
            storage_status: Some("ready".to_string()),
            storage_ready: Some(true),
            storage_available_bytes: Some(10 * 1024 * 1024 * 1024),
            storage_required_bytes: Some(512 * 1024 * 1024),
        }
    }

    #[test]
    fn ordinary_daemon_readiness_requires_exact_health_pid() {
        let expected_pid = 42;
        let exact = current_health(expected_pid);
        assert!(daemon_health_matches_pid(&exact, expected_pid));

        let mut missing = exact.clone();
        missing.pid = None;
        assert!(!daemon_health_matches_pid(&missing, expected_pid));

        let mut wrong = exact;
        wrong.pid = Some(expected_pid + 1);
        assert!(!daemon_health_matches_pid(&wrong, expected_pid));
    }

    fn daemon_info(path: &Path, pid: u32, health: DaemonHealth) -> DaemonInfo {
        DaemonInfo {
            port: 8765,
            pid,
            healthy: true,
            health: Some(health),
            metadata: Some(DaemonSpawnMetadata {
                schema_version: DAEMON_SPAWN_METADATA_SCHEMA_VERSION,
                executable_path: path.display().to_string(),
                executable_version: env!("CARGO_PKG_VERSION").to_string(),
                executable_sha256: Some(executable_sha256(path).expect("hash executable")),
                pid,
                port: 8765,
            }),
        }
    }

    #[test]
    fn runtime_attestation_matches_identical_cli_spawn_and_live_daemon() {
        let dir = tempfile::tempdir().expect("temp dir");
        let executable = dir.path().join("engram");
        fs::write(&executable, b"same executable bytes").expect("write executable");
        let info = daemon_info(&executable, 42, current_health(42));

        let attestation =
            attest_daemon_runtime(&info, Some(&executable), env!("CARGO_PKG_VERSION"), None);

        assert_eq!(attestation.status, DaemonAttestationStatus::Matched);
        assert!(attestation.warnings.is_empty());
    }

    #[test]
    fn runtime_attestation_rejects_unwritable_datastore() {
        let dir = tempfile::tempdir().expect("temp dir");
        let executable = dir.path().join("engram");
        fs::write(&executable, b"same executable bytes").expect("write executable");
        let mut health = current_health(42);
        health.storage_status = Some("write_probe_failed".to_string());
        health.storage_ready = Some(false);
        let info = daemon_info(&executable, 42, health);

        let attestation =
            attest_daemon_runtime(&info, Some(&executable), env!("CARGO_PKG_VERSION"), None);

        assert_eq!(attestation.status, DaemonAttestationStatus::Drifted);
        assert!(attestation
            .warnings
            .iter()
            .any(|warning| warning.contains("not writable")));
    }

    #[test]
    fn runtime_attestation_reports_legacy_runtime_as_degraded() {
        let dir = tempfile::tempdir().expect("temp dir");
        let executable = dir.path().join("engram");
        fs::write(&executable, b"legacy executable bytes").expect("write executable");
        let info = DaemonInfo {
            port: 8765,
            pid: 42,
            healthy: true,
            health: Some(DaemonHealth {
                status: "ok".to_string(),
                service: "engram".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                build_sha: None,
                pid: None,
                health_schema_version: None,
                mcp_contract_version: None,
                mcp_tools_sha256: None,
                mcp_protocol_version: None,
                auth_required: None,
                storage_status: None,
                storage_ready: None,
                storage_available_bytes: None,
                storage_required_bytes: None,
            }),
            metadata: Some(DaemonSpawnMetadata {
                schema_version: 0,
                executable_path: executable.display().to_string(),
                executable_version: env!("CARGO_PKG_VERSION").to_string(),
                executable_sha256: None,
                pid: 42,
                port: 8765,
            }),
        };

        let attestation =
            attest_daemon_runtime(&info, Some(&executable), env!("CARGO_PKG_VERSION"), None);

        assert_eq!(attestation.status, DaemonAttestationStatus::Degraded);
        assert!(attestation
            .warnings
            .iter()
            .any(|warning| warning.contains("health schema")));
        assert!(attestation
            .warnings
            .iter()
            .any(|warning| warning.contains("executable hash")));
    }

    #[test]
    fn runtime_attestation_detects_stale_live_daemon_version() {
        let dir = tempfile::tempdir().expect("temp dir");
        let executable = dir.path().join("engram");
        fs::write(&executable, b"same executable bytes").expect("write executable");
        let mut health = current_health(42);
        health.version = "0.1.0".to_string();
        let mut info = daemon_info(&executable, 42, health);
        info.metadata
            .as_mut()
            .expect("spawn metadata")
            .executable_version = "0.1.0".to_string();

        let attestation =
            attest_daemon_runtime(&info, Some(&executable), env!("CARGO_PKG_VERSION"), None);

        assert_eq!(attestation.status, DaemonAttestationStatus::Drifted);
        assert!(attestation
            .warnings
            .iter()
            .any(|warning| warning.contains("live daemon version 0.1.0")));
    }

    #[test]
    fn runtime_attestation_detects_live_mcp_tool_contract_drift() {
        let dir = tempfile::tempdir().expect("temp dir");
        let executable = dir.path().join("engram");
        fs::write(&executable, b"same executable bytes").expect("write executable");
        let mut health = current_health(42);
        health.mcp_tools_sha256 = Some("stale-schema-hash".to_string());
        let info = daemon_info(&executable, 42, health);

        let attestation =
            attest_daemon_runtime(&info, Some(&executable), env!("CARGO_PKG_VERSION"), None);

        assert_eq!(attestation.status, DaemonAttestationStatus::Drifted);
        assert!(attestation
            .warnings
            .iter()
            .any(|warning| warning.contains("MCP tools SHA-256")));
    }

    #[test]
    fn runtime_attestation_detects_executable_replaced_after_spawn() {
        let dir = tempfile::tempdir().expect("temp dir");
        let executable = dir.path().join("engram");
        fs::write(&executable, b"spawn-time executable bytes").expect("write executable");
        let info = daemon_info(&executable, 42, current_health(42));
        fs::write(&executable, b"replacement executable bytes").expect("replace executable");

        let attestation =
            attest_daemon_runtime(&info, Some(&executable), env!("CARGO_PKG_VERSION"), None);

        assert_eq!(attestation.status, DaemonAttestationStatus::Drifted);
        assert!(attestation
            .warnings
            .iter()
            .any(|warning| warning.contains("spawn hash")));
    }

    #[test]
    fn daemon_token_is_stable_and_owner_only() {
        let root = tempfile::tempdir().expect("temp dir");
        let path = root.path().join("daemon.token");
        let token = load_or_create_token_file(&path).expect("create token");
        let reread = load_or_create_token_file(&path).expect("read token");

        assert_eq!(token, reread);
        assert_eq!(token.len(), 64);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }

    #[tokio::test]
    async fn daemon_start_lock_serializes_concurrent_startup_attempts() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("daemon.start.lock");
        let first = acquire_daemon_start_lock_file(&path, Duration::from_secs(1))
            .await
            .unwrap();

        let error = acquire_daemon_start_lock_file(&path, Duration::from_millis(100))
            .await
            .expect_err("second startup must wait for the first lock holder");
        assert!(error.to_string().contains("Timed out waiting"));

        drop(first);
        acquire_daemon_start_lock_file(&path, Duration::from_secs(1))
            .await
            .expect("startup lock should become available after release");
    }
}
