//! Daemon management for multi-session engram support.
//!
//! This module provides functionality to manage engram daemons that allow
//! multiple Claude/Cursor sessions to share the same knowledge base.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
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
        let base = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".engram");

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

    /// Get the data directory for this daemon.
    #[allow(dead_code)]
    pub fn data_dir(&self) -> PathBuf {
        self.daemon_dir().join("data")
    }
}

/// Spawn-time runtime metadata for a daemon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonSpawnMetadata {
    pub executable_path: String,
    pub executable_version: String,
    pub pid: u32,
    pub port: u16,
}

impl DaemonSpawnMetadata {
    fn new(executable_path: &Path, pid: u32, port: u16) -> Self {
        Self {
            executable_path: executable_path.display().to_string(),
            executable_version: env!("CARGO_PKG_VERSION").to_string(),
            pid,
            port,
        }
    }
}

/// Information about a running daemon.
#[derive(Debug, Clone)]
pub struct DaemonInfo {
    pub port: u16,
    pub pid: u32,
    pub healthy: bool,
    pub metadata: Option<DaemonSpawnMetadata>,
}

/// Ensure a daemon is running, starting one if necessary.
///
/// Returns the port the daemon is listening on.
pub async fn ensure_daemon_running(config: &DaemonConfig) -> Result<u16> {
    // Check if daemon is already running
    if let Ok(info) = get_daemon_info(config).await {
        if info.healthy {
            debug!(
                "Daemon already running on port {} (PID {})",
                info.port, info.pid
            );
            return Ok(info.port);
        }
        // Daemon files exist but not healthy - clean up and restart
        warn!("Daemon not responding, cleaning up stale files");
        cleanup_daemon_files(config)?;
    }

    // Find an available port
    let port = config.port.unwrap_or_else(find_available_port);
    info!("Starting daemon on port {}", port);

    // Spawn the daemon and make sure the spawned child is the process that stays alive.
    let exe = std::env::current_exe().context("Failed to get current executable path")?;
    let mut child = spawn_daemon(config, port, &exe)?;
    let pid = child.id();

    // Wait for daemon to be ready
    wait_for_spawned_daemon(port, &mut child, Duration::from_secs(30)).await?;

    // Save daemon info
    let metadata = DaemonSpawnMetadata::new(&exe, pid, port);
    save_daemon_info(config, port, pid, &metadata)?;

    Ok(port)
}

/// Get information about a running daemon.
pub async fn get_daemon_info(config: &DaemonConfig) -> Result<DaemonInfo> {
    let port = read_daemon_port(config)?;
    let pid = read_daemon_pid(config)?;
    let healthy = check_daemon_health(port).await.is_ok();
    let metadata = read_daemon_metadata(config);

    Ok(DaemonInfo {
        port,
        pid,
        healthy,
        metadata,
    })
}

/// Check if the daemon is healthy by hitting the health endpoint.
pub async fn check_daemon_health(port: u16) -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(500))
        .build()?;

    let url = format!("http://127.0.0.1:{}/health", port);
    let response = client.get(&url).send().await?;

    if response.status().is_success() {
        Ok(())
    } else {
        bail!("Daemon unhealthy: status {}", response.status())
    }
}

/// Wait for the daemon health endpoint while also requiring the spawned child to stay alive.
async fn wait_for_spawned_daemon(port: u16, child: &mut Child, timeout: Duration) -> Result<()> {
    let start = std::time::Instant::now();
    let mut interval = tokio::time::interval(Duration::from_millis(100));

    while start.elapsed() < timeout {
        interval.tick().await;
        ensure_spawned_child_still_running(child)?;

        if check_daemon_health(port).await.is_ok() {
            tokio::time::sleep(DAEMON_SPAWN_STABILITY_DELAY).await;
            ensure_spawned_child_still_running(child)?;
            info!("Daemon is ready on port {}", port);
            return Ok(());
        }
    }

    bail!(
        "Daemon failed to start within {} seconds",
        timeout.as_secs()
    )
}

fn ensure_spawned_child_still_running(child: &mut Child) -> Result<()> {
    if let Some(status) = child
        .try_wait()
        .context("Failed to inspect spawned daemon process")?
    {
        bail!(
            "Spawned daemon process {} exited before daemon readiness was confirmed: {}",
            child.id(),
            status
        );
    }
    Ok(())
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
fn spawn_daemon(config: &DaemonConfig, port: u16, exe: &Path) -> Result<Child> {
    // Ensure daemon directory exists
    let daemon_dir = config.daemon_dir();
    fs::create_dir_all(&daemon_dir).context("Failed to create daemon directory")?;

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

    // Spawn the daemon process
    #[cfg(unix)]
    let child = {
        use std::os::unix::process::CommandExt;

        // Create a new process group so the daemon survives parent exit
        Command::new(&exe)
            .args(&args)
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

/// Save daemon info to files.
fn save_daemon_info(
    config: &DaemonConfig,
    port: u16,
    pid: u32,
    metadata: &DaemonSpawnMetadata,
) -> Result<()> {
    let mut pid_file = fs::File::create(config.pid_file())?;
    writeln!(pid_file, "{}", pid)?;

    let mut port_file = fs::File::create(config.port_file())?;
    writeln!(port_file, "{}", port)?;

    if let Err(err) = write_daemon_metadata_file(&config.metadata_file(), metadata) {
        warn!(?err, "Failed to write daemon spawn metadata");
    }
    Ok(())
}

fn write_daemon_metadata_file(path: &Path, metadata: &DaemonSpawnMetadata) -> Result<()> {
    let contents = serde_json::to_string_pretty(metadata)
        .context("Failed to serialize daemon spawn metadata")?;
    fs::write(path, contents).context("Failed to write daemon spawn metadata")
}

fn read_daemon_metadata(config: &DaemonConfig) -> Option<DaemonSpawnMetadata> {
    read_daemon_metadata_file(&config.metadata_file())
}

fn read_daemon_metadata_file(path: &Path) -> Option<DaemonSpawnMetadata> {
    let contents = fs::read_to_string(path).ok()?;
    serde_json::from_str(&contents).ok()
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
    #[test]
    fn spawned_child_liveness_accepts_running_child() {
        let mut child = Command::new("/bin/sh")
            .args(["-c", "sleep 5"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn sleep child");

        assert!(ensure_spawned_child_still_running(&mut child).is_ok());

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

        let error = ensure_spawned_child_still_running(&mut child)
            .expect_err("exited child should fail liveness check");
        assert!(error
            .to_string()
            .contains("exited before daemon readiness was confirmed"));
    }

    #[test]
    fn daemon_spawn_metadata_round_trips_json() {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join("daemon.meta.json");
        let metadata = DaemonSpawnMetadata {
            executable_path: "/tmp/engram".to_string(),
            executable_version: "0.2.0-test".to_string(),
            pid: 42,
            port: 8765,
        };

        write_daemon_metadata_file(&path, &metadata).expect("write metadata");

        assert_eq!(read_daemon_metadata_file(&path), Some(metadata));
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
}
