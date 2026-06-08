//! Multi-session integration tests for the daemon/proxy architecture.
//!
//! These tests verify that multiple sessions can share state through the daemon,
//! that project isolation works correctly, and that error handling is robust.

use anyhow::{bail, Context, Result};
use serde_json::json;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command as StdCommand, Stdio};
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::OnceLock;
use std::time::Duration;
use tempfile::TempDir;
use tokio::process::{Child, Command};

/// Global port counter to ensure each test gets unique ports.
static PORT_COUNTER: AtomicU16 = AtomicU16::new(19000);
static ENGRAM_BIN: OnceLock<PathBuf> = OnceLock::new();

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

    let debug_bin = workspace_root.join("target").join("debug").join("engram");
    let status = StdCommand::new("cargo")
        .args(["build", "-p", "engram-cli", "--bin", "engram"])
        .current_dir(workspace_root)
        .status()
        .expect("Failed to build engram binary for multi-session tests");
    assert!(status.success(), "Failed to build engram binary");
    if debug_bin.exists() {
        return debug_bin;
    }

    let release_bin = workspace_root.join("target").join("release").join("engram");
    if release_bin.exists() {
        return release_bin;
    }
    // Fallback: assume it's in PATH
    PathBuf::from("engram")
}

/// Test daemon manager for integration tests.
struct TestDaemon {
    port: u16,
    child: Child,
    _data_dir: TempDir,
}

impl TestDaemon {
    /// Start a new test daemon with isolated data directory.
    async fn start() -> Result<Self> {
        let port = find_available_port();
        Self::start_on_port(port).await
    }

    /// Start a daemon on a specific port.
    async fn start_on_port(port: u16) -> Result<Self> {
        let data_dir = TempDir::new().context("Failed to create temp dir")?;

        let child = Command::new(engram_bin())
            .args(["serve", "--http", "--port", &port.to_string(), "--memory"])
            .env("ENGRAM_DATA_DIR", data_dir.path())
            .env("ENGRAM_EMBED_CACHE_DIR", test_embed_cache_dir())
            .env("RUST_LOG", "warn")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true) // Critical: cleanup on test failure
            .spawn()
            .context("Failed to spawn daemon")?;

        let daemon = Self {
            port,
            child,
            _data_dir: data_dir,
        };

        // Wait for daemon to be ready
        daemon.wait_for_health(Duration::from_secs(30)).await?;

        Ok(daemon)
    }

    /// Wait for the daemon to respond to health checks.
    async fn wait_for_health(&self, timeout_duration: Duration) -> Result<()> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(500))
            .build()?;

        let start = std::time::Instant::now();
        let url = format!("http://127.0.0.1:{}/health", self.port);

        while start.elapsed() < timeout_duration {
            if client.get(&url).send().await.is_ok() {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        bail!(
            "Daemon failed to become healthy within {:?}",
            timeout_duration
        )
    }

    /// Get the MCP endpoint URL.
    fn mcp_url(&self) -> String {
        format!("http://127.0.0.1:{}/mcp", self.port)
    }

    /// Stop the daemon gracefully.
    async fn stop(mut self) -> Result<()> {
        self.child.kill().await.ok();
        Ok(())
    }
}

impl Drop for TestDaemon {
    fn drop(&mut self) {
        // Ensure cleanup even if test panics
        let _ = self.child.start_kill();
    }
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
        self.call_tool("entity", json!({ "action": "list" })).await
    }

    /// Search entities by name.
    async fn search_entities(&mut self, query: &str) -> Result<serde_json::Value> {
        self.call_tool("entity", json!({ "action": "search", "query": query }))
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
async fn test_mcp_tools_list_lint_schema_exposes_project_filter() {
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
