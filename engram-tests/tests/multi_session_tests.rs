//! Multi-session integration tests for the daemon/proxy architecture.
//!
//! These tests verify that multiple sessions can share state through the daemon,
//! that project isolation works correctly, and that error handling is robust.

use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command as StdCommand, Stdio};
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tempfile::TempDir;
use tokio::process::{Child, Command};
use tokio::sync::{Mutex as TokioMutex, OwnedMutexGuard};

/// Global port counter to ensure each test gets unique ports.
static PORT_COUNTER: AtomicU16 = AtomicU16::new(19000);
static ENGRAM_BIN: OnceLock<PathBuf> = OnceLock::new();
static DAEMON_TEST_LOCK: OnceLock<Arc<TokioMutex<()>>> = OnceLock::new();
const DAEMON_HEALTH_TIMEOUT: Duration = Duration::from_secs(120);
const DAEMON_HEALTH_STABILITY_DELAY: Duration = Duration::from_millis(250);
const DAEMON_LOG_TAIL_LINES: usize = 40;

fn mcp_tools_sha256(tools: &[Value]) -> String {
    let mut tools = tools.to_vec();
    tools.sort_by(|left, right| left["name"].as_str().cmp(&right["name"].as_str()));
    let encoded = serde_json::to_vec(&tools).expect("MCP tools should serialize");
    format!("{:x}", Sha256::digest(encoded))
}

#[tokio::test]
async fn test_agent_profile_contract_attests_effective_stdio_runtime() -> Result<()> {
    let binary = engram_bin();
    let embed_cache = test_embed_cache_dir();
    let command_binary = binary.clone();
    let output = tokio::task::spawn_blocking(move || {
        StdCommand::new(command_binary)
            .args([
                "contract",
                "--profile",
                "agent",
                "--verify-runtime",
                "--json",
            ])
            .env("ENGRAM_EMBED_CACHE_DIR", embed_cache)
            .output()
    })
    .await??;
    if !output.status.success() {
        bail!(
            "agent-profile runtime attestation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let report: Value = serde_json::from_slice(&output.stdout)?;
    let runtime = report
        .get("effective_runtime")
        .context("runtime attestation should be present")?;

    assert_eq!(report["schema_version"], 4);
    assert_eq!(report["profile"], "agent");
    assert_eq!(report["mcp_tool_count"], 6);
    assert_eq!(runtime["verified"], true);
    assert_eq!(runtime["restricted_tool_rejected"], true);
    assert_eq!(runtime["review_authority_rejected"], true);
    assert_eq!(runtime["correction_verification_unavailable"], true);
    assert_eq!(runtime["mcp_tool_count"], report["mcp_tool_count"]);
    assert_eq!(runtime["mcp_tools_sha256"], report["mcp_tools_sha256"]);
    assert_eq!(
        runtime["profile_instructions_sha256"],
        report["profile_instructions_sha256"]
    );
    assert_eq!(
        Path::new(report["executable_path"].as_str().unwrap()),
        binary.canonicalize()?.as_path()
    );
    assert_eq!(report["executable_sha256"].as_str().unwrap().len(), 64);
    Ok(())
}

fn parse_mcp_tool_json(response: &Value) -> Result<Value> {
    if response
        .pointer("/result/isError")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        bail!("MCP tool returned an error result: {response}");
    }
    let text = response
        .pointer("/result/content/0/text")
        .and_then(Value::as_str)
        .context("MCP tool result should contain JSON text")?;
    serde_json::from_str(text).context("MCP tool result text should be valid JSON")
}

fn resolve_schema_ref<'a>(root: &'a Value, schema: &'a Value) -> Result<&'a Value> {
    let Some(reference) = schema.get("$ref").and_then(Value::as_str) else {
        return Ok(schema);
    };
    let pointer = reference
        .strip_prefix('#')
        .context("fixture schemas may reference only the current tool schema")?;
    root.pointer(pointer)
        .with_context(|| format!("tool schema reference should resolve: {reference}"))
}

fn validate_fixture_value(root: &Value, schema: &Value, value: &Value, path: &str) -> Result<()> {
    let schema = resolve_schema_ref(root, schema)?;
    if let Some(branches) = schema.get("anyOf").and_then(Value::as_array) {
        if branches
            .iter()
            .any(|branch| validate_fixture_value(root, branch, value, path).is_ok())
        {
            return Ok(());
        }
        bail!("{path} does not match any live schema branch");
    }

    if let Some(allowed) = schema.get("enum").and_then(Value::as_array) {
        if !allowed.contains(value) {
            bail!("{path} value {value} is not in the live schema enum");
        }
    }

    if let Some(schema_type) = schema.get("type").and_then(Value::as_str) {
        let matches = match schema_type {
            "object" => value.is_object(),
            "array" => value.is_array(),
            "string" => value.is_string(),
            "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
            "number" => value.is_number(),
            "boolean" => value.is_boolean(),
            "null" => value.is_null(),
            _ => true,
        };
        if !matches {
            bail!("{path} value {value} does not have live schema type {schema_type}");
        }
    }

    if let Some(object) = value.as_object() {
        let properties = schema.get("properties").and_then(Value::as_object);
        for required in schema
            .get("required")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            if !object.contains_key(required) {
                bail!("{path} is missing live required argument '{required}'");
            }
        }

        for (name, child) in object {
            if let Some(child_schema) = properties.and_then(|items| items.get(name)) {
                validate_fixture_value(root, child_schema, child, &format!("{path}.{name}"))?;
                continue;
            }
            if let Some(additional) = schema
                .get("additionalProperties")
                .filter(|item| item.is_object())
            {
                validate_fixture_value(root, additional, child, &format!("{path}.{name}"))?;
                continue;
            }
            bail!("{path} contains undeclared live argument '{name}'");
        }
    }

    if let (Some(items), Some(values)) = (schema.get("items"), value.as_array()) {
        for (index, child) in values.iter().enumerate() {
            validate_fixture_value(root, items, child, &format!("{path}[{index}]"))?;
        }
    }

    Ok(())
}

/// Get a unique port for testing.
fn get_test_port() -> u16 {
    PORT_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Find an available port by binding.
fn find_available_port() -> u16 {
    let port = get_test_port();
    // Try binding to verify it's available
    if TcpListener::bind(("127.0.0.1", port)).is_ok() {
        port
    } else {
        // Fallback: let OS pick
        TcpListener::bind("127.0.0.1:0")
            .and_then(|l| l.local_addr())
            .map(|a| a.port())
            .unwrap_or_else(|_| get_test_port())
    }
}

/// Get the path to the engram binary.
fn engram_bin() -> PathBuf {
    ENGRAM_BIN.get_or_init(resolve_engram_bin).clone()
}

fn test_embed_cache_dir() -> PathBuf {
    for key in ["ENGRAM_EMBED_CACHE_DIR", "FASTEMBED_CACHE_DIR"] {
        if let Some(path) = std::env::var_os(key).filter(|value| !value.as_os_str().is_empty()) {
            return PathBuf::from(path);
        }
    }

    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".fastembed_cache")
}

fn daemon_test_lock() -> Arc<TokioMutex<()>> {
    DAEMON_TEST_LOCK
        .get_or_init(|| Arc::new(TokioMutex::new(())))
        .clone()
}

fn resolve_engram_bin() -> PathBuf {
    // Use env var if set (for CI), otherwise find relative to workspace
    if let Ok(path) = std::env::var("ENGRAM_BIN") {
        return PathBuf::from(path);
    }

    // Try to find the binary in target directory
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));

    let workspace_root = manifest_dir.parent().unwrap_or(&manifest_dir);

    let target_root = std::env::var_os("CARGO_TARGET_DIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .map(|path| {
            if path.is_absolute() {
                path
            } else {
                workspace_root.join(path)
            }
        })
        .unwrap_or_else(|| workspace_root.join("target"));
    let debug_bin = target_root.join("debug").join("engram");
    let status = StdCommand::new("cargo")
        .args(["build", "-p", "engram-cli", "--bin", "engram"])
        .current_dir(workspace_root)
        .status()
        .expect("Failed to build engram binary for multi-session tests");
    assert!(status.success(), "Failed to build engram binary");
    if debug_bin.exists() {
        return debug_bin;
    }

    let release_bin = target_root.join("release").join("engram");
    if release_bin.exists() {
        return release_bin;
    }
    // Fallback: assume it's in PATH
    PathBuf::from("engram")
}

/// Test daemon manager for integration tests.
struct TestDaemon {
    port: u16,
    pid: u32,
    child: Child,
    _data_dir: Option<TempDir>,
    stdout_path: PathBuf,
    stderr_path: PathBuf,
    _lock: OwnedMutexGuard<()>,
}

impl TestDaemon {
    /// Start a new test daemon with isolated data directory.
    async fn start() -> Result<Self> {
        let port = find_available_port();
        Self::start_on_port(port).await
    }

    /// Start a daemon on a specific port.
    async fn start_on_port(port: u16) -> Result<Self> {
        let lock = daemon_test_lock().lock_owned().await;
        let data_dir = TempDir::new().context("Failed to create temp dir")?;
        let stdout_path = data_dir.path().join("daemon.stdout.log");
        let stderr_path = data_dir.path().join("daemon.stderr.log");
        let stdout =
            fs::File::create(&stdout_path).context("Failed to create daemon stdout log")?;
        let stderr =
            fs::File::create(&stderr_path).context("Failed to create daemon stderr log")?;

        let child = Command::new(engram_bin())
            .args(["serve", "--http", "--port", &port.to_string(), "--memory"])
            .env("ENGRAM_DATA_DIR", data_dir.path())
            .env("ENGRAM_EMBED_CACHE_DIR", test_embed_cache_dir())
            .env("RUST_LOG", "warn")
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .kill_on_drop(true) // Critical: cleanup on test failure
            .spawn()
            .context("Failed to spawn daemon")?;

        let pid = child.id().unwrap_or_default();
        let mut daemon = Self {
            port,
            pid,
            child,
            _data_dir: Some(data_dir),
            stdout_path,
            stderr_path,
            _lock: lock,
        };

        // Wait for daemon to be ready
        daemon.wait_for_health(DAEMON_HEALTH_TIMEOUT).await?;

        Ok(daemon)
    }

    /// Start a daemon backed by a caller-owned persistent state root.
    async fn start_persistent(state_root: &Path) -> Result<Self> {
        let port = find_available_port();
        let lock = daemon_test_lock().lock_owned().await;
        let stdout_path = state_root.join(format!("daemon-{port}.stdout.log"));
        let stderr_path = state_root.join(format!("daemon-{port}.stderr.log"));
        let stdout =
            fs::File::create(&stdout_path).context("Failed to create daemon stdout log")?;
        let stderr =
            fs::File::create(&stderr_path).context("Failed to create daemon stderr log")?;

        let child = Command::new(engram_bin())
            .args(["serve", "--http", "--port", &port.to_string()])
            .env("ENGRAM_HOME", state_root)
            .env("ENGRAM_EMBED_CACHE_DIR", test_embed_cache_dir())
            .env("RUST_LOG", "warn")
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .kill_on_drop(true)
            .spawn()
            .context("Failed to spawn persistent daemon")?;

        let pid = child.id().unwrap_or_default();
        let mut daemon = Self {
            port,
            pid,
            child,
            _data_dir: None,
            stdout_path,
            stderr_path,
            _lock: lock,
        };
        daemon.wait_for_health(DAEMON_HEALTH_TIMEOUT).await?;
        Ok(daemon)
    }

    /// Wait for the daemon to respond to health checks.
    async fn wait_for_health(&mut self, timeout_duration: Duration) -> Result<()> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(500))
            .build()?;

        let start = std::time::Instant::now();
        let url = format!("http://127.0.0.1:{}/health", self.port);
        let mut last_health_error = None;

        while start.elapsed() < timeout_duration {
            self.ensure_child_running()?;

            match client.get(&url).send().await {
                Ok(response) if response.status().is_success() => {
                    tokio::time::sleep(DAEMON_HEALTH_STABILITY_DELAY).await;
                    self.ensure_child_running()?;
                    return Ok(());
                }
                Ok(response) => {
                    last_health_error =
                        Some(format!("health endpoint returned {}", response.status()));
                }
                Err(err) => {
                    last_health_error = Some(err.to_string());
                }
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        let health_context = last_health_error
            .map(|err| format!("\nlast health error: {err}"))
            .unwrap_or_default();
        bail!(
            "Daemon failed to become healthy within {:?}{}{}",
            timeout_duration,
            health_context,
            self.daemon_output_tail()
        )
    }

    fn ensure_child_running(&mut self) -> Result<()> {
        if let Some(status) = self
            .child
            .try_wait()
            .context("Failed to inspect daemon process")?
        {
            bail!(
                "Daemon process {} exited before health check succeeded: {}{}",
                self.pid,
                status,
                self.daemon_output_tail()
            );
        }
        Ok(())
    }

    fn daemon_output_tail(&self) -> String {
        let mut output = String::new();
        if let Some(stdout) = recent_log_tail(&self.stdout_path, DAEMON_LOG_TAIL_LINES) {
            output.push_str("\n\nRecent daemon stdout:\n");
            output.push_str(&stdout);
        }
        if let Some(stderr) = recent_log_tail(&self.stderr_path, DAEMON_LOG_TAIL_LINES) {
            output.push_str("\n\nRecent daemon stderr:\n");
            output.push_str(&stderr);
        }
        output
    }

    /// Get the MCP endpoint URL.
    fn mcp_url(&self) -> String {
        format!("http://127.0.0.1:{}/mcp", self.port)
    }

    /// Stop the daemon gracefully.
    async fn stop(mut self) -> Result<()> {
        self.child.kill().await.ok();
        self.child.wait().await.ok();
        Ok(())
    }
}

impl Drop for TestDaemon {
    fn drop(&mut self) {
        // Ensure cleanup even if test panics
        let _ = self.child.start_kill();
    }
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

/// Test HTTP client for sending MCP requests directly to daemon.
struct TestHttpClient {
    client: reqwest::Client,
    mcp_url: String,
    session_id: Option<String>,
    initialized: bool,
}

impl TestHttpClient {
    /// Create a new HTTP client connected to the daemon.
    fn new(daemon_port: u16) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            mcp_url: format!("http://127.0.0.1:{}/mcp", daemon_port),
            session_id: None,
            initialized: false,
        }
    }

    /// Send a JSON-RPC request and get the response.
    async fn send_request(&self, request: serde_json::Value) -> Result<serde_json::Value> {
        let mut req_builder = self
            .client
            .post(&self.mcp_url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .json(&request);

        if let Some(ref sid) = self.session_id {
            req_builder = req_builder.header("mcp-session-id", sid);
        }

        let response = req_builder.send().await.context("Failed to send request")?;

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if content_type.contains("text/event-stream") {
            // Parse SSE response
            let body = response.text().await?;
            parse_sse_response(&body)
        } else {
            response
                .json()
                .await
                .context("Failed to parse JSON response")
        }
    }

    /// Send a raw request and capture response headers (for initialize).
    async fn send_request_with_headers(
        &self,
        request: serde_json::Value,
    ) -> Result<(serde_json::Value, Option<String>)> {
        let mut req_builder = self
            .client
            .post(&self.mcp_url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .json(&request);

        if let Some(ref sid) = self.session_id {
            req_builder = req_builder.header("mcp-session-id", sid);
        }

        let response = req_builder.send().await.context("Failed to send request")?;

        // Extract session ID from headers
        let session_id = response
            .headers()
            .get("mcp-session-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let body = if content_type.contains("text/event-stream") {
            let body = response.text().await?;
            parse_sse_response(&body)?
        } else {
            response
                .json()
                .await
                .context("Failed to parse JSON response")?
        };

        Ok((body, session_id))
    }

    /// Send a notification (no response expected).
    async fn send_notification(&self, notification: serde_json::Value) -> Result<()> {
        let mut req_builder = self
            .client
            .post(&self.mcp_url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .json(&notification);

        if let Some(ref sid) = self.session_id {
            req_builder = req_builder.header("mcp-session-id", sid);
        }

        req_builder
            .send()
            .await
            .context("Failed to send notification")?;
        Ok(())
    }

    /// Initialize the MCP session with full handshake.
    async fn initialize(&mut self) -> Result<serde_json::Value> {
        // Step 1: Send initialize request
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "engram-test",
                    "version": "1.0.0"
                }
            }
        });

        let (response, session_id) = self.send_request_with_headers(request).await?;

        // Store session ID
        if let Some(sid) = session_id {
            self.session_id = Some(sid);
        }

        // Step 2: Send initialized notification
        let notification = json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        self.send_notification(notification).await?;

        self.initialized = true;
        Ok(response)
    }

    /// Ensure client is initialized before making requests.
    async fn ensure_initialized(&mut self) -> Result<()> {
        if !self.initialized {
            self.initialize().await?;
        }
        Ok(())
    }

    /// Call an MCP tool (ensures client is initialized first).
    async fn call_tool(
        &mut self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value> {
        self.ensure_initialized().await?;

        let request = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": tool_name,
                "arguments": arguments
            }
        });

        let response = self.send_request(request).await?;
        if response.get("error").is_some() {
            bail!("MCP tool call failed: {}", response);
        }
        Ok(response)
    }

    /// Create an entity.
    async fn create_entity(&mut self, name: &str, entity_type: &str) -> Result<serde_json::Value> {
        self.call_tool(
            "entity",
            json!({
                "action": "create",
                "name": name,
                "entity_type": entity_type
            }),
        )
        .await
    }

    /// List all entities.
    async fn list_entities(&mut self) -> Result<serde_json::Value> {
        self.call_tool(
            "entity",
            json!({
                "action": "list",
                "scope": { "relevance_mode": "global" }
            }),
        )
        .await
    }

    /// Search entities by name.
    async fn search_entities(&mut self, query: &str) -> Result<serde_json::Value> {
        self.call_tool(
            "entity",
            json!({
                "action": "search",
                "query": query,
                "search_scope": { "relevance_mode": "global" }
            }),
        )
        .await
    }
}

/// Parse SSE response to extract the last data event.
fn parse_sse_response(body: &str) -> Result<serde_json::Value> {
    let mut last_data = None;

    for line in body.lines() {
        if let Some(data) = line.strip_prefix("data:") {
            let data = data.trim();
            if !data.is_empty() {
                last_data = Some(data.to_string());
            }
        }
    }

    match last_data {
        Some(data) => serde_json::from_str(&data).context("Failed to parse SSE data as JSON"),
        None => bail!("No data in SSE response"),
    }
}

fn git_available() -> bool {
    StdCommand::new("git")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn run_git(cwd: &Path, args: &[&str]) {
    let output = StdCommand::new("git")
        .arg("-C")
        .arg(cwd)
        .args(args)
        .output()
        .expect("git should run");
    assert!(
        output.status.success(),
        "git {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
}

// =============================================================================
// Multi-Session State Sharing Tests
// =============================================================================

#[tokio::test]
async fn test_daemon_starts_and_responds_to_health() {
    let daemon = TestDaemon::start().await.expect("Failed to start daemon");

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://127.0.0.1:{}/health", daemon.port))
        .send()
        .await
        .expect("Health check failed");

    assert!(response.status().is_success());

    let body: serde_json::Value = response
        .json()
        .await
        .expect("Failed to parse health response");
    assert_eq!(body["status"], "ok");

    daemon.stop().await.expect("Failed to stop daemon");
}

#[tokio::test]
async fn test_resume_orientation_survives_real_daemon_restart() -> Result<()> {
    let state_root = TempDir::new().context("persistent state root should be created")?;
    let daemon = TestDaemon::start_persistent(state_root.path()).await?;
    let mut client = TestHttpClient::new(daemon.port);

    let captured = parse_mcp_tool_json(
        &client
            .call_tool(
                "memory",
                json!({
                    "action": "capture_current_plan",
                    "kind": "decision",
                    "title": "Persistent daemon resume plan",
                    "content": "Resume by validating the source-bound schema-14 route.",
                    "project_name": "engram",
                    "origin": "tool_result",
                    "message": "Capture persistent daemon resume plan",
                    "writer_harness": "codex",
                    "model_provider": "openai",
                    "model": "gpt-5.6-sol",
                    "surface": "test",
                    "evidence": [{
                        "kind": "tool_call",
                        "target": "persistent-daemon-resume-fixture",
                        "summary": "Current plan captured before the daemon restart."
                    }]
                }),
            )
            .await?,
    )?;
    let plan_id = captured["item"]["id"]
        .as_str()
        .context("captured plan should have an ID")?
        .to_string();
    let handoff = parse_mcp_tool_json(
        &client
            .call_tool(
                "handoff",
                json!({
                    "action": "update",
                    "project": "engram",
                    "content": "Schema 14 is prepared; authentication is the next boundary.",
                    "next_actions": [
                        "Provision only after explicit credential-copy authorization."
                    ],
                    "dry_run": false,
                    "writer_harness": "codex",
                    "model_provider": "openai",
                    "model": "gpt-5.6-sol",
                    "surface": "test"
                }),
            )
            .await?,
    )?;
    let handoff_id = handoff["item"]["id"]
        .as_str()
        .context("captured handoff should have an ID")?
        .to_string();
    daemon.stop().await?;

    assert!(
        state_root.path().join("data/CURRENT").is_file(),
        "test must cross a persistent RocksDB and daemon-process boundary"
    );

    let daemon = TestDaemon::start_persistent(state_root.path()).await?;
    let mut resumed_client = TestHttpClient::new(daemon.port);
    let orientation = parse_mcp_tool_json(
        &resumed_client
            .call_tool(
                "orient",
                json!({
                    "project": "engram",
                    "prompt": "Continue from where we left off.",
                    "agent": "codex",
                    "external_session_id": "post-restart-session",
                    "intent": "resume_session",
                    "scenario_id": "persistent_daemon_resume",
                    "arm": "engram",
                    "include_recent_commits": false,
                    "limit": 5,
                    "response_shape": "lean"
                }),
            )
            .await?,
    )?;

    let top_items = orientation["brain_loop"]["top_items"]
        .as_array()
        .context("resume orientation should contain ranked items")?;
    assert!(
        top_items
            .iter()
            .any(|item| item["id"].as_str() == Some(handoff_id.as_str())),
        "resume orientation: {orientation:#}"
    );
    assert!(
        top_items
            .iter()
            .any(|item| item["id"].as_str() == Some(plan_id.as_str())),
        "resume orientation: {orientation:#}"
    );
    let used_ids = orientation["used_memory_candidate_ids"]
        .as_array()
        .context("resume orientation should report used memory IDs")?;
    assert!(
        used_ids
            .iter()
            .any(|id| id.as_str() == Some(handoff_id.as_str())),
        "resume orientation: {orientation:#}"
    );
    assert!(
        used_ids
            .iter()
            .any(|id| id.as_str() == Some(plan_id.as_str())),
        "resume orientation: {orientation:#}"
    );

    daemon.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_mcp_initialize_returns_capabilities() {
    let daemon = TestDaemon::start().await.expect("Failed to start daemon");
    let mut client = TestHttpClient::new(daemon.port);

    let response = client.initialize().await.expect("Initialize failed");

    // Verify it's a valid JSON-RPC response
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["result"].is_object());

    // Verify capabilities include tools
    let result = &response["result"];
    assert!(result["capabilities"]["tools"].is_object());

    daemon.stop().await.expect("Failed to stop daemon");
}

#[tokio::test]
async fn test_mcp_tools_list_lint_schema_exposes_project_filter_and_scope() {
    let daemon = TestDaemon::start().await.expect("Failed to start daemon");
    let mut client = TestHttpClient::new(daemon.port);
    client.initialize().await.expect("Initialize failed");

    let request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });
    let response = client
        .send_request(request)
        .await
        .expect("tools/list should respond");
    let tools = response["result"]["tools"]
        .as_array()
        .expect("tools/list should return a tools array");
    let lint = tools
        .iter()
        .find(|tool| tool["name"] == "lint")
        .expect("tools/list should expose lint");
    let properties = lint["inputSchema"]["properties"]
        .as_object()
        .expect("lint input schema should expose properties");

    assert!(properties.contains_key("project"));
    assert_eq!(
        properties["project"]["description"],
        "Optional project scope to lint."
    );
    assert_eq!(
        properties["scope"]["description"],
        "Authorization boundary for retrieval actions"
    );
    assert!(!properties.contains_key("search_scope"));

    daemon.stop().await.expect("Failed to stop daemon");
}

#[tokio::test]
async fn test_mcp_tools_list_obligations_schema_exposes_scope_filters() {
    let daemon = TestDaemon::start().await.expect("Failed to start daemon");
    let mut client = TestHttpClient::new(daemon.port);
    client.initialize().await.expect("Initialize failed");

    let request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });
    let response = client
        .send_request(request)
        .await
        .expect("tools/list should respond");
    let tools = response["result"]["tools"]
        .as_array()
        .expect("tools/list should return a tools array");
    let obligations = tools
        .iter()
        .find(|tool| tool["name"] == "obligations")
        .expect("tools/list should expose obligations");
    let properties = obligations["inputSchema"]["properties"]
        .as_object()
        .expect("obligations input schema should expose properties");

    assert_eq!(
        properties["project"]["description"],
        "Optional project scope for detect, add, list, open, and doctor."
    );
    assert_eq!(
        properties["cwd"]["description"],
        "Current working directory for detect/list/open/doctor scoping."
    );

    daemon.stop().await.expect("Failed to stop daemon");
}

#[tokio::test]
async fn test_mcp_tools_list_telemetry_schema_exposes_project_filter_and_scope() {
    let daemon = TestDaemon::start().await.expect("Failed to start daemon");
    let mut client = TestHttpClient::new(daemon.port);
    client.initialize().await.expect("Initialize failed");

    let request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });
    let response = client
        .send_request(request)
        .await
        .expect("tools/list should respond");
    let tools = response["result"]["tools"]
        .as_array()
        .expect("tools/list should return a tools array");
    let telemetry = tools
        .iter()
        .find(|tool| tool["name"] == "telemetry")
        .expect("tools/list should expose telemetry");
    let properties = telemetry["inputSchema"]["properties"]
        .as_object()
        .expect("telemetry input schema should expose properties");

    assert_eq!(
        properties["project"]["description"],
        "Optional project scope for record_trace, list_traces, list_feedback, stats_by_intent, and real_session_eval."
    );
    assert_eq!(
        properties["scope"]["description"],
        "Authorization boundary for retrieval actions"
    );
    assert!(!properties.contains_key("search_scope"));

    daemon.stop().await.expect("Failed to stop daemon");
}

#[tokio::test]
async fn test_mcp_tools_list_exposes_orientation_and_retrieval_scope_contracts() {
    let daemon = TestDaemon::start().await.expect("Failed to start daemon");
    let mut client = TestHttpClient::new(daemon.port);
    client.initialize().await.expect("Initialize failed");

    let request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });
    let response = client
        .send_request(request)
        .await
        .expect("tools/list should respond");
    let tools = response["result"]["tools"]
        .as_array()
        .expect("tools/list should return a tools array");
    let exposed_tools_sha256 = mcp_tools_sha256(tools);
    let health: Value = reqwest::get(format!("http://127.0.0.1:{}/health", daemon.port))
        .await
        .expect("health should respond")
        .json()
        .await
        .expect("health should return JSON");
    assert_eq!(
        health["mcp_tools_sha256"], exposed_tools_sha256,
        "health must attest the exact live tools/list contract"
    );
    let orient = tools
        .iter()
        .find(|tool| tool["name"] == "orient")
        .expect("tools/list should expose orient");
    let properties = orient["inputSchema"]["properties"]
        .as_object()
        .expect("orient input schema should expose properties");

    assert_eq!(
        properties["cwd"]["description"],
        "Current working directory for deterministic repository/component identity and scoped memory selection. A cwd never authorizes a project by directory basename."
    );
    assert_eq!(
        properties["project"]["description"],
        "Explicit project authorization for project-scoped memory selection. Omit when unknown; the response identity reports candidates or requires_confirmation without inventing scope."
    );
    assert_eq!(
        properties["response_shape"]["description"],
        "Response shape: full (default) or lean for compact identity/trace/cursor/Brain Loop guidance. When omitted by Claude Code agents, defaults to lean to avoid oversized hook/tool output."
    );

    for tool_name in [
        "entity",
        "entity_observe",
        "session",
        "docs",
        "tool",
        "knowledge",
        "coord",
        "work_project",
        "work_task",
        "work_pr",
        "work_observe",
        "work_context",
        "repo",
        "memory",
        "telemetry",
        "handoff",
        "harness",
        "obligations",
        "lint",
        "graph",
        "vault",
        "digest",
    ] {
        let retrieval_tool = tools
            .iter()
            .find(|tool| tool["name"] == tool_name)
            .unwrap_or_else(|| panic!("tools/list should expose {tool_name}"));
        let retrieval_properties = retrieval_tool["inputSchema"]["properties"]
            .as_object()
            .unwrap_or_else(|| panic!("{tool_name} input schema should expose properties"));
        assert!(retrieval_properties.contains_key("scope"));
        assert!(!retrieval_properties.contains_key("search_scope"));
        let description = retrieval_properties["scope"]["description"]
            .as_str()
            .expect("scope description should be text");
        assert!(description.starts_with("Authorization boundary for retrieval actions"));
        if tool_name == "memory" {
            assert!(description.contains("procedure_match always applies local scope"));
        }
    }

    for tool_name in [
        "knowledge_stats",
        "entity_stats",
        "session_stats",
        "tool_intel_stats",
        "coord_stats",
        "work_stats",
    ] {
        let stats_tool = tools
            .iter()
            .find(|tool| tool["name"] == tool_name)
            .unwrap_or_else(|| panic!("tools/list should expose {tool_name}"));
        let stats_properties = stats_tool["inputSchema"]["properties"]
            .as_object()
            .unwrap_or_else(|| panic!("{tool_name} input schema should expose properties"));
        assert!(stats_properties.contains_key("scope"));
        assert!(!stats_properties.contains_key("search_scope"));
        assert_eq!(
            stats_properties["scope"]["description"],
            "Authorization boundary for retrieval actions"
        );
    }

    daemon.stop().await.expect("Failed to stop daemon");
}

#[tokio::test]
async fn test_generated_codex_and_claude_adapter_fixtures_execute_against_live_schema() {
    let daemon = TestDaemon::start().await.expect("Failed to start daemon");
    let mut client = TestHttpClient::new(daemon.port);
    client.initialize().await.expect("Initialize failed");

    let response = client
        .send_request(json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        }))
        .await
        .expect("tools/list should respond");
    let tools = response["result"]["tools"]
        .as_array()
        .expect("tools/list should return a tools array");

    let fixture_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../evals/adapter_contract_v1/fixtures.json");
    let fixture: Value = serde_json::from_str(
        &fs::read_to_string(&fixture_path).expect("adapter contract fixture should be readable"),
    )
    .expect("adapter contract fixture should be valid JSON");
    assert_eq!(fixture["schema_version"], 1);
    let project = fixture["project"]
        .as_str()
        .expect("fixture should declare a project");

    let created = client
        .call_tool(
            "work_project",
            json!({
                "action": "create",
                "name": project,
                "description": "Ephemeral native adapter contract fixture"
            }),
        )
        .await
        .expect("fixture project creation should succeed");
    parse_mcp_tool_json(&created).expect("fixture project response should be valid");

    for harness in fixture["harnesses"]
        .as_array()
        .expect("fixture should declare harnesses")
    {
        let harness_name = harness["harness"]
            .as_str()
            .expect("harness fixture should declare a harness");
        let adapter_name = harness["adapter"]
            .as_str()
            .expect("harness fixture should declare an adapter");
        let rendered = client
            .call_tool(
                "harness",
                json!({
                    "action": "render_adapter",
                    "harness": harness_name,
                    "adapter": adapter_name
                }),
            )
            .await
            .expect("live adapter rendering should succeed");
        let rendered = parse_mcp_tool_json(&rendered)
            .expect("live adapter rendering should return valid JSON");
        let contents = rendered["adapters"][0]["contents"]
            .as_str()
            .expect("live adapter rendering should include contents");
        for fragment in harness["required_adapter_fragments"]
            .as_array()
            .expect("harness fixture should declare adapter fragments")
            .iter()
            .filter_map(Value::as_str)
        {
            assert!(
                contents.contains(fragment),
                "{harness_name} adapter should contain contract fragment: {fragment}"
            );
        }

        for call in harness["calls"]
            .as_array()
            .expect("harness fixture should declare calls")
        {
            let tool_name = call["tool"]
                .as_str()
                .expect("contract call should declare a tool");
            assert!(
                ["orient", "search", "memory"].contains(&tool_name),
                "adapter contract should stay within the compact agent retrieval surface"
            );
            let live_tool = tools
                .iter()
                .find(|tool| tool["name"] == tool_name)
                .unwrap_or_else(|| panic!("live tools/list should expose {tool_name}"));
            let arguments = &call["arguments"];
            validate_fixture_value(
                &live_tool["inputSchema"],
                &live_tool["inputSchema"],
                arguments,
                tool_name,
            )
            .unwrap_or_else(|error| {
                panic!("{harness_name} {tool_name} fixture should match live schema: {error}")
            });

            let response = client
                .call_tool(tool_name, arguments.clone())
                .await
                .unwrap_or_else(|error| {
                    panic!("{harness_name} {tool_name} fixture should execute: {error}")
                });
            let result = parse_mcp_tool_json(&response).unwrap_or_else(|error| {
                panic!("{harness_name} {tool_name} fixture should succeed: {error}")
            });
            for (pointer, expected) in call["expected"]
                .as_object()
                .expect("contract call should declare expected response values")
            {
                assert_eq!(
                    result.pointer(pointer),
                    Some(expected),
                    "{harness_name} {tool_name} response should satisfy {pointer}"
                );
            }
        }
    }

    let memory_tool = tools
        .iter()
        .find(|tool| tool["name"] == "memory")
        .expect("live tools/list should expose memory");
    let mut stale_arguments = fixture["harnesses"][0]["calls"][3]["arguments"].clone();
    let stale_object = stale_arguments
        .as_object_mut()
        .expect("memory fixture arguments should be an object");
    let scope = stale_object
        .remove("scope")
        .expect("memory fixture should declare scope");
    stale_object.insert("search_scope".to_string(), scope);
    let stale_error = validate_fixture_value(
        &memory_tool["inputSchema"],
        &memory_tool["inputSchema"],
        &stale_arguments,
        "memory",
    )
    .expect_err("a stale search_scope adapter call should fail the live schema contract");
    assert!(stale_error.to_string().contains("search_scope"));

    let mut missing_action = fixture["harnesses"][0]["calls"][3]["arguments"].clone();
    missing_action
        .as_object_mut()
        .expect("memory fixture arguments should be an object")
        .remove("action");
    let missing_error = validate_fixture_value(
        &memory_tool["inputSchema"],
        &memory_tool["inputSchema"],
        &missing_action,
        "memory",
    )
    .expect_err("a memory adapter call without action should fail the live schema contract");
    assert!(missing_error
        .to_string()
        .contains("required argument 'action'"));

    daemon.stop().await.expect("Failed to stop daemon");
}

#[tokio::test]
async fn test_two_sessions_share_entity_state() {
    let daemon = TestDaemon::start().await.expect("Failed to start daemon");

    // Create two independent clients (simulating two Claude sessions)
    let mut client1 = TestHttpClient::new(daemon.port);
    let mut client2 = TestHttpClient::new(daemon.port);

    // Session 1 creates an entity
    let create_result = client1
        .create_entity("shared-test-repo", "repo")
        .await
        .expect("Failed to create entity");

    // Verify creation succeeded
    assert!(
        create_result["result"].is_object() || create_result["result"].is_array(),
        "Entity creation should return result"
    );

    // Session 2 should see the entity when listing
    let list_result = client2
        .list_entities()
        .await
        .expect("Failed to list entities");

    // The result should contain the entity we created
    let result_str = serde_json::to_string(&list_result).unwrap();
    assert!(
        result_str.contains("shared-test-repo"),
        "Session 2 should see entity created by Session 1. Got: {}",
        result_str
    );

    daemon.stop().await.expect("Failed to stop daemon");
}

#[tokio::test]
async fn test_two_sessions_share_search_results() {
    let daemon = TestDaemon::start().await.expect("Failed to start daemon");

    let mut client1 = TestHttpClient::new(daemon.port);
    let mut client2 = TestHttpClient::new(daemon.port);

    // Session 1 creates multiple entities
    client1
        .create_entity("search-test-alpha", "repo")
        .await
        .expect("Failed to create entity 1");
    client1
        .create_entity("search-test-beta", "tool")
        .await
        .expect("Failed to create entity 2");
    client1
        .create_entity("other-entity", "service")
        .await
        .expect("Failed to create entity 3");

    // Session 2 searches for entities
    let search_result = client2
        .search_entities("search-test")
        .await
        .expect("Failed to search entities");

    let result_str = serde_json::to_string(&search_result).unwrap();

    // Should find the two matching entities
    assert!(
        result_str.contains("search-test-alpha"),
        "Should find alpha entity"
    );
    assert!(
        result_str.contains("search-test-beta"),
        "Should find beta entity"
    );

    daemon.stop().await.expect("Failed to stop daemon");
}

#[tokio::test]
async fn test_memory_tool_smoke_over_http_daemon() {
    let daemon = TestDaemon::start().await.expect("Failed to start daemon");
    let mut client = TestHttpClient::new(daemon.port);

    let add_result = client
        .call_tool(
            "memory",
            json!({
                "action": "add",
                "kind": "decision",
                "title": "HTTP memory smoke",
                "content": "The daemon exposes the memory MCP tool over HTTP.",
                "origin": "user_stated",
                "scope_type": "project",
                "project_name": "engram",
                "writer_harness": "codex",
                "model_provider": "openai",
                "model": "gpt-5.5",
                "surface": "test"
            }),
        )
        .await
        .expect("Failed to add memory over HTTP");
    let add_str = serde_json::to_string(&add_result).unwrap();
    assert!(add_str.contains("HTTP memory smoke"));

    let orient_result = client
        .call_tool(
            "orient",
            json!({
                "cwd": "/Users/yuval.meiri/projects/engram",
                "project": "engram",
                "agent": "codex",
                "prompt": "daemon smoke test",
                "include_recent_commits": true
            }),
        )
        .await
        .expect("Failed to orient over HTTP");
    let orient_str = serde_json::to_string(&orient_result).unwrap();
    assert!(orient_str.contains("HTTP memory smoke"));
    assert!(orient_str.contains("memory_cursor"));

    if git_available() {
        let repo_dir = TempDir::new().expect("Failed to create repo temp dir");
        run_git(repo_dir.path(), &["init"]);
        run_git(
            repo_dir.path(),
            &[
                "remote",
                "add",
                "origin",
                "git@github.com:ymeiri/engram.git",
            ],
        );

        let detect_result = client
            .call_tool(
                "repo",
                json!({
                    "action": "detect",
                    "cwd": repo_dir.path().display().to_string()
                }),
            )
            .await
            .expect("Failed to detect repo over HTTP");
        let detect_str = serde_json::to_string(&detect_result).unwrap();
        assert!(detect_str.contains("engram"));
        assert!(detect_str.contains("detected_root"));

        let repo_orient_result = client
            .call_tool(
                "orient",
                json!({
                    "cwd": repo_dir.path().display().to_string(),
                    "agent": "codex",
                    "prompt": "repo daemon smoke test",
                    "include_recent_commits": false
                }),
            )
            .await
            .expect("Failed to orient with repo context over HTTP");
        let repo_orient_str = serde_json::to_string(&repo_orient_result).unwrap();
        assert!(repo_orient_str.contains("repository_context"));
        assert!(repo_orient_str.contains("engram"));
    }

    daemon.stop().await.expect("Failed to stop daemon");
}

// =============================================================================
// Concurrent Access Tests
// =============================================================================

#[tokio::test]
async fn test_concurrent_entity_creation_different_names() {
    let daemon = TestDaemon::start().await.expect("Failed to start daemon");

    // Spawn 5 concurrent entity creations with different names
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let port = daemon.port;
            tokio::spawn(async move {
                let mut client = TestHttpClient::new(port);
                client
                    .create_entity(&format!("concurrent-entity-{}", i), "repo")
                    .await
            })
        })
        .collect();

    // Wait for all to complete
    let results: Vec<_> = futures::future::join_all(handles).await;

    // All should succeed
    for (i, result) in results.iter().enumerate() {
        let inner = result.as_ref().expect("Task panicked");
        assert!(inner.is_ok(), "Entity {} creation failed: {:?}", i, inner);
    }

    // Verify all entities exist
    let mut client = TestHttpClient::new(daemon.port);
    let list_result = client.list_entities().await.expect("Failed to list");
    let result_str = serde_json::to_string(&list_result).unwrap();

    for i in 0..5 {
        assert!(
            result_str.contains(&format!("concurrent-entity-{}", i)),
            "Entity {} should exist",
            i
        );
    }

    daemon.stop().await.expect("Failed to stop daemon");
}

#[tokio::test]
async fn test_rapid_sequential_requests() {
    let daemon = TestDaemon::start().await.expect("Failed to start daemon");
    let mut client = TestHttpClient::new(daemon.port);

    // Send 20 rapid sequential requests
    for i in 0..20 {
        let result = client
            .create_entity(&format!("rapid-entity-{}", i), "repo")
            .await;

        assert!(result.is_ok(), "Request {} failed: {:?}", i, result);
    }

    // Verify all exist
    let list_result = client.list_entities().await.expect("Failed to list");
    let result_str = serde_json::to_string(&list_result).unwrap();

    for i in 0..20 {
        assert!(
            result_str.contains(&format!("rapid-entity-{}", i)),
            "Entity {} should exist",
            i
        );
    }

    daemon.stop().await.expect("Failed to stop daemon");
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[tokio::test]
async fn test_malformed_json_returns_error() {
    let daemon = TestDaemon::start().await.expect("Failed to start daemon");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();

    let response = client
        .post(daemon.mcp_url())
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .body("{ invalid json }")
        .send()
        .await
        .expect("Request failed");

    // Should return an error status or JSON-RPC error
    // The exact behavior depends on implementation
    assert!(
        !response.status().is_success() || {
            let body: serde_json::Value = response.json().await.unwrap_or_default();
            body.get("error").is_some()
        },
        "Malformed JSON should be rejected"
    );

    daemon.stop().await.expect("Failed to stop daemon");
}

#[tokio::test]
async fn test_unknown_method_returns_error() {
    let daemon = TestDaemon::start().await.expect("Failed to start daemon");
    let mut client = TestHttpClient::new(daemon.port);

    // Initialize first
    client.initialize().await.expect("Failed to initialize");

    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "nonexistent/method",
        "params": {}
    });

    let response = client.send_request(request).await;

    // Should either error or return JSON-RPC error
    match response {
        Ok(val) => {
            assert!(
                val.get("error").is_some(),
                "Unknown method should return error: {:?}",
                val
            );
        }
        Err(_) => {
            // Connection error is also acceptable for unknown method
        }
    }

    daemon.stop().await.expect("Failed to stop daemon");
}

// =============================================================================
// Daemon Lifecycle Tests
// =============================================================================

#[tokio::test]
async fn test_daemon_health_endpoint() {
    let daemon = TestDaemon::start().await.expect("Failed to start daemon");

    // Multiple health checks should succeed
    let client = reqwest::Client::new();
    for _ in 0..5 {
        let response = client
            .get(format!("http://127.0.0.1:{}/health", daemon.port))
            .send()
            .await
            .expect("Health check failed");

        assert!(response.status().is_success());
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    daemon.stop().await.expect("Failed to stop daemon");
}

#[tokio::test]
async fn test_daemon_handles_connection_after_client_disconnect() {
    let daemon = TestDaemon::start().await.expect("Failed to start daemon");

    // Client 1 connects and creates data
    {
        let mut client1 = TestHttpClient::new(daemon.port);
        client1
            .create_entity("persist-test", "repo")
            .await
            .expect("Failed to create");
        // client1 dropped here
    }

    // New client should still see the data
    let mut client2 = TestHttpClient::new(daemon.port);
    let list_result = client2.list_entities().await.expect("Failed to list");
    let result_str = serde_json::to_string(&list_result).unwrap();

    assert!(
        result_str.contains("persist-test"),
        "Data should persist after client disconnect"
    );

    daemon.stop().await.expect("Failed to stop daemon");
}

// =============================================================================
// Session Coordination Tests (Cross-Session)
// =============================================================================

#[tokio::test]
async fn test_coordination_conflict_detection_across_sessions() {
    let daemon = TestDaemon::start().await.expect("Failed to start daemon");

    let mut client1 = TestHttpClient::new(daemon.port);
    let mut client2 = TestHttpClient::new(daemon.port);

    // Use proper UUIDs for session IDs
    let session_id_1 = "019c0000-0000-7000-8000-000000000001";
    let session_id_2 = "019c0000-0000-7000-8000-000000000002";

    // Session 1 registers for coordination
    let reg1 = client1
        .call_tool(
            "coord",
            json!({
                "action": "register",
                "session_id": session_id_1,
                "agent": "claude-1",
                "project": "shared-project",
                "goal": "Implement feature A",
                "components": ["auth", "api"]
            }),
        )
        .await;

    // Session 2 registers with overlapping component
    let reg2 = client2
        .call_tool(
            "coord",
            json!({
                "action": "register",
                "session_id": session_id_2,
                "agent": "claude-2",
                "project": "shared-project",
                "goal": "Implement feature B",
                "components": ["api", "database"]
            }),
        )
        .await;

    // Both registrations should succeed
    assert!(reg1.is_ok(), "Session 1 registration failed: {:?}", reg1);
    assert!(reg2.is_ok(), "Session 2 registration failed: {:?}", reg2);

    // Check conflicts from session 1's perspective
    let conflicts = client1
        .call_tool(
            "coord",
            json!({
                "action": "check_conflicts",
                "session_id": session_id_1
            }),
        )
        .await
        .expect("Failed to check conflicts");

    let result_str = serde_json::to_string(&conflicts).unwrap();

    // Should detect conflict on "api" component (session 2 id or "api" should appear)
    assert!(
        result_str.contains(session_id_2)
            || result_str.contains("api")
            || result_str.contains("conflict"),
        "Should detect conflict with session-2 on api component. Got: {}",
        result_str
    );

    // Cleanup
    let _ = client1
        .call_tool(
            "coord",
            json!({ "action": "unregister", "session_id": session_id_1 }),
        )
        .await;
    let _ = client2
        .call_tool(
            "coord",
            json!({ "action": "unregister", "session_id": session_id_2 }),
        )
        .await;

    daemon.stop().await.expect("Failed to stop daemon");
}

// =============================================================================
// Timeout and Long Operation Tests
// =============================================================================

#[tokio::test]
async fn test_request_does_not_timeout_quickly() {
    let daemon = TestDaemon::start().await.expect("Failed to start daemon");
    let mut client = TestHttpClient::new(daemon.port);

    // Create multiple entities - should complete well within timeout
    let start = std::time::Instant::now();

    for i in 0..10 {
        client
            .create_entity(&format!("timeout-test-{}", i), "repo")
            .await
            .expect("Request timed out unexpectedly");
    }

    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_secs(10),
        "Operations took too long: {:?}",
        elapsed
    );

    daemon.stop().await.expect("Failed to stop daemon");
}
