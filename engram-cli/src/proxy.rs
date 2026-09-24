//! Stdio-to-HTTP proxy for transparent daemon access.
//!
//! This module bridges MCP stdio clients to the HTTP daemon, making the daemon
//! transparent to clients that only support stdio transport.

use crate::daemon;
use anyhow::{bail, Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;
use tracing::{debug, error, info, trace};

const AGENT_PROFILE_INSTRUCTIONS: &str = "Engram agent profile v1 supplies scoped, evidence-backed engineering context. For non-actionable tasks, call orient at task start with cwd, project when known, the full user prompt, and response_shape=lean. Pass task as an exact task name or tracker key when known; include project when using a task name. Treat ambiguity or a scope mismatch as a reason to abstain from project/task memory and ask the user. Every actionable repository task has a mandatory single-call identity and procedure route, even without remembered/learned wording: call memory action=procedure_match directly with a task-focused query of at most 512 characters copied from the current user request and the exact current cwd before native memory, memory list, orient, or command exploration. Preserve concrete operation nouns, identifiers, and failure text verbatim; omit unrelated meta/output instructions and secret values. The query is retrieval text, not an authorization channel. Requests to handle, run, debug, test, build, deploy, modify, determine how to perform a repository operation, or use durable procedure memory are actionable. memory action=list is not a substitute for procedure_match. Project ambiguity does not block repository-local matching. procedure_match returns structured repository/project/component identity and only active, verified, unexpired procedures whose scope and exact prerequisites match. Engram resolves source-backed prerequisites itself from Git-tracked files in the current checkout and returns value-redacted condition observations; caller conditions apply only to unsourced prerequisites. Treat suggested_operation_evidence as a required host-action protocol, not an optional suggestion. When required_before_final_abstention=true, the next tool call must read its canonical absolute resolved_path unchanged before interpreting abstained, asking for project confirmation, or returning final output, including for durable-memory-only requests. allowed_when_project_requires_confirmation=true authorizes only that repository-local evidence read; authorizes_procedure_execution=false forbids executing a remembered procedure. Treat the relative path as provenance only. Use search or repo for progressive retrieval when the identity boundary is insufficient. Store only durable facts with provenance and source evidence, never secrets. Agent-profile memory adds must truthfully use agent_observed or agent_inferred origin and cannot assert user provenance or manual review. The explicit agent-profile correction API can propose non-procedure guidance or an unverified structured procedure replacement, but cannot inspect, verify, or apply a proposal. For a correction, call memory action=propose_correction with the exact obsolete ID, proposed title/content, source evidence, writer identity, an optional structured procedure card, and a local or related project/task/cwd selector. Engram derives kind and scope, creates an inactive needs_review replacement, and returns a digest for an operator; the obsolete item remains active. Full-profile verification is required before a procedure correction can be applied. This profile restriction is not human authentication and does not prevent semantically conflicting autonomous adds or deletion through other permitted APIs. Global relevance is forbidden, and an empty local boundary is valid only for global- or user-scoped obsolete memory. memory action=forget is irreversible and requires an exact ID, reason, and confirmation.";
const AGENT_TOOL_NAMES: [&str; 6] = [
    "orient",
    "memory",
    "repo",
    "search",
    "harness",
    "obligations",
];
const PROFILE_PROBE_TIMEOUT: Duration = Duration::from_secs(90);

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct AgentProfileRuntimeAttestation {
    pub verified: bool,
    pub mcp_tool_count: usize,
    pub mcp_tools_sha256: String,
    pub profile_instructions_sha256: String,
    pub restricted_tool_rejected: bool,
    pub review_authority_rejected: bool,
    pub correction_proposal_path_verified: bool,
    pub direct_correction_unavailable: bool,
    pub correction_apply_unavailable: bool,
    pub correction_verification_unavailable: bool,
    pub correction_inspection_unavailable: bool,
}

/// MCP surface exposed by the stdio proxy.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ToolProfile {
    /// Compact surface intended for coding-agent sessions.
    Agent,
    /// Complete administrative and migration surface.
    #[default]
    Full,
}

/// Configuration for the proxy.
#[derive(Clone)]
pub struct ProxyConfig {
    /// The daemon port to connect to.
    pub daemon_port: u16,
    /// Request timeout.
    pub timeout: Duration,
    /// Bearer token for the private daemon MCP endpoint.
    pub auth_token: Option<String>,
    /// MCP tools and actions presented to the client.
    pub tool_profile: ToolProfile,
}

impl std::fmt::Debug for ProxyConfig {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ProxyConfig")
            .field("daemon_port", &self.daemon_port)
            .field("timeout", &self.timeout)
            .field(
                "auth_token",
                &self.auth_token.as_ref().map(|_| "<redacted>"),
            )
            .field("tool_profile", &self.tool_profile)
            .finish()
    }
}

impl ProxyConfig {
    pub fn new(daemon_port: u16, auth_token: Option<String>) -> Self {
        Self {
            daemon_port,
            timeout: Duration::from_secs(300), // 5 minute timeout for long operations
            auth_token,
            tool_profile: ToolProfile::Full,
        }
    }

    pub fn with_tool_profile(mut self, tool_profile: ToolProfile) -> Self {
        self.tool_profile = tool_profile;
        self
    }
}

/// Proxy state for managing MCP session.
struct ProxyState {
    session_id: Option<String>,
    initialized: bool,
    initialize_request: Option<serde_json::Value>,
    /// Queue of requests received before initialization completed.
    pending_requests: Vec<serde_json::Value>,
}

/// Run the stdio-to-HTTP proxy.
///
/// This function reads JSON-RPC messages from stdin, forwards them to the
/// HTTP daemon, and writes validated responses to stdout. Calls can mutate daemon state within the
/// selected tool profile. When stdin closes, the proxy sends a DELETE for its MCP session. Native
/// evaluators may use this only with an evaluator-owned isolated `Child` and store plus
/// phase-constrained MCP traffic.
pub async fn run_proxy(config: ProxyConfig) -> Result<()> {
    info!("Starting proxy to daemon on port {}", config.daemon_port);

    let client = daemon_http_client(config.timeout, config.auth_token.as_deref())?;

    let mcp_url = format!("http://127.0.0.1:{}/mcp", config.daemon_port);

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin);
    let mut writer = stdout;

    // Proxy state for session management
    let state = Arc::new(Mutex::new(ProxyState {
        session_id: None,
        initialized: false,
        initialize_request: None,
        pending_requests: Vec::new(),
    }));

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                // EOF - stdin closed
                debug!("Stdin closed, exiting proxy");
                break;
            }
            Ok(_) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                trace!("Received from stdin: {}", line);

                // Parse the JSON-RPC message
                let request: serde_json::Value = match serde_json::from_str(line) {
                    Ok(v) => v,
                    Err(e) => {
                        error!("Failed to parse JSON-RPC request: {}", e);
                        continue;
                    }
                };

                // Check if this is an initialize request or initialized notification
                let method = request.get("method").and_then(|m| m.as_str());

                match method {
                    Some("initialize") => {
                        // Forward initialize and capture session ID
                        match forward_initialize(&client, &mcp_url, &request).await {
                            Ok((mut response, session_id)) => {
                                apply_profile_response(
                                    config.tool_profile,
                                    &request,
                                    &mut response,
                                );
                                // Store session ID
                                {
                                    let mut s = state.lock().await;
                                    s.session_id = session_id;
                                    s.initialize_request = Some(request.clone());
                                    debug!("Captured session ID: {:?}", s.session_id);
                                }

                                let response_str = serde_json::to_string(&response)?;
                                trace!("Sending to stdout: {}", response_str);
                                writer.write_all(response_str.as_bytes()).await?;
                                writer.write_all(b"\n").await?;
                                writer.flush().await?;
                            }
                            Err(e) => {
                                error!("Failed to forward initialize: {}", e);
                                let error_response =
                                    create_error_response(&request, &e.to_string());
                                let response_str = serde_json::to_string(&error_response)?;
                                writer.write_all(response_str.as_bytes()).await?;
                                writer.write_all(b"\n").await?;
                                writer.flush().await?;
                            }
                        }
                    }
                    Some("notifications/initialized") => {
                        // Forward initialized notification
                        let session_id = state.lock().await.session_id.clone();
                        match forward_notification(&client, &mcp_url, &session_id, &request).await {
                            Ok(_) => {
                                // Mark as initialized and get pending requests
                                let pending = {
                                    let mut s = state.lock().await;
                                    s.initialized = true;
                                    debug!("Session initialized");
                                    std::mem::take(&mut s.pending_requests)
                                };

                                // Process any requests that were queued before initialization
                                if !pending.is_empty() {
                                    debug!("Processing {} pending requests", pending.len());
                                    for pending_req in pending {
                                        let is_notification = pending_req.get("id").is_none();
                                        match forward_request_with_recovery(
                                            &client,
                                            &mcp_url,
                                            &state,
                                            &pending_req,
                                        )
                                        .await
                                        {
                                            Ok(mut response) => {
                                                if !is_notification {
                                                    apply_profile_response(
                                                        config.tool_profile,
                                                        &pending_req,
                                                        &mut response,
                                                    );
                                                    let response_str =
                                                        serde_json::to_string(&response)?;
                                                    trace!(
                                                        "Sending queued response to stdout: {}",
                                                        response_str
                                                    );
                                                    writer
                                                        .write_all(response_str.as_bytes())
                                                        .await?;
                                                    writer.write_all(b"\n").await?;
                                                    writer.flush().await?;
                                                }
                                            }
                                            Err(e) => {
                                                error!("Failed to forward queued request: {}", e);
                                                if !is_notification {
                                                    let error_response = create_error_response(
                                                        &pending_req,
                                                        &e.to_string(),
                                                    );
                                                    let response_str =
                                                        serde_json::to_string(&error_response)?;
                                                    writer
                                                        .write_all(response_str.as_bytes())
                                                        .await?;
                                                    writer.write_all(b"\n").await?;
                                                    writer.flush().await?;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to forward initialized notification: {}", e);
                            }
                        }
                        // Notifications don't expect a response
                    }
                    _ => {
                        if let Err(message) =
                            validate_profile_request(config.tool_profile, &request)
                        {
                            if request.get("id").is_some() {
                                let error_response =
                                    create_profile_error_response(&request, &message);
                                let response_str = serde_json::to_string(&error_response)?;
                                writer.write_all(response_str.as_bytes()).await?;
                                writer.write_all(b"\n").await?;
                                writer.flush().await?;
                            }
                            continue;
                        }

                        // Regular request - check if initialized first
                        let initialized = {
                            let s = state.lock().await;
                            s.initialized
                        };

                        if !initialized {
                            // Queue request until initialization completes
                            debug!(
                                "Queueing request until initialized: {:?}",
                                request.get("method")
                            );
                            state.lock().await.pending_requests.push(request.clone());
                            continue;
                        }

                        // Check if this is a notification (no id field)
                        let is_notification = request.get("id").is_none();

                        match forward_request_with_recovery(&client, &mcp_url, &state, &request)
                            .await
                        {
                            Ok(mut response) => {
                                if !is_notification {
                                    apply_profile_response(
                                        config.tool_profile,
                                        &request,
                                        &mut response,
                                    );
                                    let response_str = serde_json::to_string(&response)?;
                                    trace!("Sending to stdout: {}", response_str);
                                    writer.write_all(response_str.as_bytes()).await?;
                                    writer.write_all(b"\n").await?;
                                    writer.flush().await?;
                                }
                            }
                            Err(e) => {
                                error!("Failed to forward request: {}", e);
                                if !is_notification {
                                    let error_response =
                                        create_error_response(&request, &e.to_string());
                                    let response_str = serde_json::to_string(&error_response)?;
                                    writer.write_all(response_str.as_bytes()).await?;
                                    writer.write_all(b"\n").await?;
                                    writer.flush().await?;
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                error!("Error reading from stdin: {}", e);
                bail!("Stdin read error: {}", e);
            }
        }
    }

    // Clean up session
    let session_id = state.lock().await.session_id.clone();
    if let Some(sid) = session_id {
        let _ = delete_session(&client, &mcp_url, &sid).await;
    }

    Ok(())
}

/// Call a single MCP tool through the HTTP daemon.
pub async fn call_tool_once(
    daemon_port: u16,
    auth_token: Option<&str>,
    tool_name: &str,
    arguments: serde_json::Value,
) -> Result<serde_json::Value> {
    let client = daemon_http_client(Duration::from_secs(30), auth_token)?;
    let mcp_url = format!("http://127.0.0.1:{daemon_port}/mcp");
    let initialize_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "engram-cli",
                "version": env!("CARGO_PKG_VERSION")
            }
        }
    });

    let (_, session_id) = forward_initialize(&client, &mcp_url, &initialize_request).await?;
    let initialized_notification = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    forward_notification(&client, &mcp_url, &session_id, &initialized_notification).await?;

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": tool_name,
            "arguments": arguments
        }
    });
    let response = forward_request(&client, &mcp_url, &session_id, &request).await;

    if let Some(sid) = session_id {
        let _ = delete_session(&client, &mcp_url, &sid).await;
    }

    response
}

fn validate_profile_request(
    profile: ToolProfile,
    request: &serde_json::Value,
) -> std::result::Result<(), String> {
    if profile == ToolProfile::Full
        || request.get("method").and_then(|method| method.as_str()) != Some("tools/call")
    {
        return Ok(());
    }

    let name = request
        .pointer("/params/name")
        .and_then(|name| name.as_str())
        .ok_or_else(|| "tools/call requires params.name".to_string())?;
    if !matches!(
        name,
        "orient" | "memory" | "repo" | "search" | "harness" | "obligations"
    ) {
        return Err(format!(
            "tool '{name}' is unavailable in MCP profile 'agent'; use --profile full for administrative APIs"
        ));
    }

    let action = request
        .pointer("/params/arguments/action")
        .and_then(|action| action.as_str())
        .map(str::to_ascii_lowercase);
    let allowed = match name {
        "memory" => action.as_deref().is_some_and(|action| {
            matches!(
                action,
                "add"
                    | "propose_correction"
                    | "procedure_match"
                    | "get"
                    | "list"
                    | "archive"
                    | "forget"
            )
        }),
        "repo" => action.as_deref().is_some_and(|action| {
            matches!(
                action,
                "detect" | "context" | "register" | "list" | "link_project"
            )
        }),
        "harness" => action.as_deref() == Some("hook_event"),
        "obligations" => action
            .as_deref()
            .is_some_and(|action| matches!(action, "detect" | "doctor" | "resolve" | "skip")),
        _ => true,
    };
    if !allowed {
        return Err(format!(
            "action '{}' is unavailable for tool '{name}' in MCP profile 'agent'",
            action.as_deref().unwrap_or("<missing>")
        ));
    }

    if name == "memory" {
        match action.as_deref() {
            Some("add") => validate_agent_memory_add(request)?,
            Some("propose_correction") => validate_agent_correction_proposal(request)?,
            Some("archive") if request.pointer("/params/arguments/archived_by").is_some() => {
                return Err(
                    "caller archived_by attribution is unavailable for memory archive in MCP profile 'agent'"
                        .to_string(),
                );
            }
            _ => {}
        }
    }

    Ok(())
}

fn validate_agent_memory_add(request: &serde_json::Value) -> std::result::Result<(), String> {
    if request_asserts_manual_review(request) {
        return Err(
            "manual_review evidence is unavailable in MCP profile 'agent'; coding agents may propose guidance with source evidence but cannot assert reviewer authority"
                .to_string(),
        );
    }
    if request.pointer("/params/arguments/actor").is_some() {
        return Err(
            "caller actor overrides are unavailable for memory add in MCP profile 'agent'"
                .to_string(),
        );
    }
    let origin = request
        .pointer("/params/arguments/origin")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            "memory add in MCP profile 'agent' requires truthful origin=agent_observed or origin=agent_inferred"
                .to_string()
        })?;
    if !matches!(
        origin.to_ascii_lowercase().as_str(),
        "agent_observed" | "agent_inferred"
    ) {
        return Err(
            "memory add in MCP profile 'agent' only accepts origin=agent_observed or origin=agent_inferred; agents cannot manufacture user or tool provenance"
                .to_string(),
        );
    }
    Ok(())
}

fn validate_agent_correction_proposal(
    request: &serde_json::Value,
) -> std::result::Result<(), String> {
    let arguments = request
        .pointer("/params/arguments")
        .and_then(serde_json::Value::as_object);
    let asserts_server_derived_fields = arguments.is_some_and(|arguments| {
        ["actor", "origin", "status", "kind", "scope_type"]
            .iter()
            .any(|field| arguments.contains_key(*field))
    });
    if asserts_server_derived_fields || request_asserts_manual_review(request) {
        return Err(
            "memory propose_correction derives actor, origin, status, kind, and memory scope server-side; structured procedure replacements remain inactive and unverified, and agents cannot assert manual-review authority"
                .to_string(),
        );
    }
    Ok(())
}

fn request_asserts_manual_review(request: &serde_json::Value) -> bool {
    request
        .pointer("/params/arguments/evidence")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|evidence| {
            evidence.iter().any(|item| {
                item.get("kind")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|kind| kind.eq_ignore_ascii_case("manual_review"))
            })
        })
}

fn apply_profile_response(
    profile: ToolProfile,
    request: &serde_json::Value,
    response: &mut serde_json::Value,
) {
    if profile == ToolProfile::Full {
        return;
    }

    match request.get("method").and_then(|method| method.as_str()) {
        Some("initialize") => {
            if let Some(result) = response
                .get_mut("result")
                .and_then(|result| result.as_object_mut())
            {
                result.insert(
                    "instructions".to_string(),
                    serde_json::Value::String(AGENT_PROFILE_INSTRUCTIONS.to_string()),
                );
            }
        }
        Some("tools/list") => {
            if let Some(tools) = response
                .pointer_mut("/result/tools")
                .and_then(|tools| tools.as_array_mut())
            {
                let filtered = std::mem::take(tools)
                    .into_iter()
                    .filter_map(agent_tool_declaration)
                    .collect();
                *tools = filtered;
            }
        }
        _ => {}
    }
}

pub(crate) fn agent_profile_tool_contract() -> Vec<serde_json::Value> {
    let mut tools = AGENT_TOOL_NAMES
        .iter()
        .filter_map(|name| agent_tool_declaration(serde_json::json!({"name": name})))
        .collect::<Vec<_>>();
    tools.sort_by(|left, right| left["name"].as_str().cmp(&right["name"].as_str()));
    tools
}

pub(crate) fn agent_profile_tools_sha256() -> String {
    json_tools_sha256(&agent_profile_tool_contract())
}

pub(crate) fn agent_profile_instructions_sha256() -> String {
    format!(
        "{:x}",
        Sha256::digest(AGENT_PROFILE_INSTRUCTIONS.as_bytes())
    )
}

pub(crate) async fn probe_agent_profile_runtime(
    executable: &Path,
) -> Result<AgentProfileRuntimeAttestation> {
    let executable = executable.canonicalize()?;
    let root = tempfile::Builder::new()
        .prefix("engram-agent-contract-")
        .tempdir()
        .context("Failed to create isolated agent-profile probe directory")?;
    let project = "mcp-contract-probe";
    let listener = std::net::TcpListener::bind("127.0.0.1:0")
        .context("Failed to reserve an agent-profile probe port")?;
    let port = listener.local_addr()?.port();
    drop(listener);
    let auth_token = format!(
        "{:x}",
        Sha256::digest(engram_core::id::Id::new().to_string().as_bytes())
    );
    let daemon_dir = root.path().join("projects").join(project);
    let port_file = daemon_dir.join("daemon.port");
    let pid_file = daemon_dir.join("daemon.pid");
    let metadata_file = daemon_dir.join("daemon.meta.json");
    let token_file = daemon_dir.join("daemon.token");
    engram_store::ensure_private_directory(&daemon_dir)?;
    std::fs::write(&port_file, format!("{port}\n"))?;
    std::fs::write(&token_file, &auth_token)?;
    engram_store::ensure_private_file(&port_file)?;
    engram_store::ensure_private_file(&token_file)?;

    let daemon_stderr_path = root.path().join("daemon.stderr.log");
    let daemon_stderr = std::fs::File::create(&daemon_stderr_path)?;
    let mut daemon_child = tokio::process::Command::new(&executable)
        .args([
            "serve",
            "--http",
            "--memory",
            "--port",
            &port.to_string(),
            "--project",
            project,
        ])
        .env(engram_store::ENGRAM_HOME_ENV, root.path())
        .env("ENGRAM_DAEMON_TOKEN", &auth_token)
        .env("RUST_LOG", "warn")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(daemon_stderr))
        .kill_on_drop(true)
        .spawn()
        .context("Failed to launch isolated in-memory agent-profile daemon")?;
    let daemon_pid = daemon_child
        .id()
        .context("Agent-profile probe daemon has no process ID")?;
    std::fs::write(&pid_file, format!("{daemon_pid}\n"))?;
    engram_store::ensure_private_file(&pid_file)?;
    write_profile_probe_daemon_metadata(&metadata_file, &executable, daemon_pid, port)?;

    let daemon_ready = tokio::time::timeout(PROFILE_PROBE_TIMEOUT, async {
        loop {
            if daemon::fetch_daemon_health(port)
                .await
                .is_ok_and(|health| daemon::daemon_health_matches_pid(&health, daemon_pid))
            {
                return Ok(());
            }
            if let Some(status) = daemon_child.try_wait()? {
                bail!("In-memory agent-profile daemon exited early with {status}");
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .context("Timed out waiting for isolated in-memory agent-profile daemon")?;
    if let Err(error) = daemon_ready {
        let _ = daemon_child.kill().await;
        let _ = daemon_child.wait().await;
        let stderr = std::fs::read_to_string(&daemon_stderr_path).unwrap_or_default();
        return Err(error).with_context(|| {
            format!("Failed to start isolated in-memory agent-profile daemon: {stderr}")
        });
    }

    let stderr_path = root.path().join("proxy.stderr.log");
    let stderr = std::fs::File::create(&stderr_path)?;
    let mut child = tokio::process::Command::new(&executable)
        .args(["serve", "--project", project, "--profile", "agent"])
        .env(engram_store::ENGRAM_HOME_ENV, root.path())
        .env("RUST_LOG", "warn")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::from(stderr))
        .kill_on_drop(true)
        .spawn()
        .context("Failed to launch isolated agent-profile proxy")?;
    let mut stdin = child
        .stdin
        .take()
        .context("Profile probe stdin is unavailable")?;
    let stdout = child
        .stdout
        .take()
        .context("Profile probe stdout is unavailable")?;
    let mut stdout = BufReader::new(stdout);

    let probe = async {
        write_probe_message(
            &mut stdin,
            &serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": engram_mcp::server::MCP_PROTOCOL_VERSION,
                    "capabilities": {},
                    "clientInfo": {"name": "engram-contract-probe", "version": "1"}
                }
            }),
        )
        .await?;
        let initialize = read_probe_response(&mut stdout, "initialize").await?;
        let instructions = initialize
            .pointer("/result/instructions")
            .and_then(serde_json::Value::as_str)
            .context("Agent-profile initialize response omitted instructions")?;

        write_probe_message(
            &mut stdin,
            &serde_json::json!({
                "jsonrpc": "2.0",
                "method": "notifications/initialized"
            }),
        )
        .await?;
        write_probe_message(
            &mut stdin,
            &serde_json::json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/list",
                "params": {}
            }),
        )
        .await?;
        let tools_response = read_probe_response(&mut stdout, "tools/list").await?;
        let tools = tools_response
            .pointer("/result/tools")
            .and_then(serde_json::Value::as_array)
            .context("Agent-profile tools/list response omitted tools")?;

        write_probe_message(
            &mut stdin,
            &serde_json::json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {"name": "docs", "arguments": {"action": "stats"}}
            }),
        )
        .await?;
        let restricted = read_probe_response(&mut stdout, "restricted tools/call").await?;
        let restricted_tool_rejected = restricted
            .pointer("/error/code")
            .and_then(serde_json::Value::as_i64)
            == Some(-32601);
        if !restricted_tool_rejected {
            bail!("Agent profile did not reject a restricted administrative tool");
        }

        write_probe_message(
            &mut stdin,
            &serde_json::json!({
                "jsonrpc": "2.0",
                "id": 4,
                "method": "tools/call",
                "params": {
                    "name": "memory",
                    "arguments": {
                        "action": "add",
                        "evidence": [{"kind": "manual_review", "target": "claimed-reviewer"}]
                    }
                }
            }),
        )
        .await?;
        let review = read_probe_response(&mut stdout, "review-authority tools/call").await?;
        let review_authority_rejected = review
            .pointer("/error/code")
            .and_then(serde_json::Value::as_i64)
            == Some(-32601);
        if !review_authority_rejected {
            bail!("Agent profile did not reject caller-asserted reviewer authority");
        }

        for (id, arguments, label) in [
            (
                5,
                serde_json::json!({
                    "action": "add",
                    "origin": "user_corrected"
                }),
                "user-corrected add",
            ),
            (
                6,
                serde_json::json!({
                    "action": "add",
                    "origin": "user_stated"
                }),
                "user-stated add",
            ),
            (
                7,
                serde_json::json!({
                    "action": "add",
                    "actor": "user"
                }),
                "actor override add",
            ),
            (
                8,
                serde_json::json!({
                    "action": "archive",
                    "id": "00000000-0000-0000-0000-000000000000",
                    "archived_by": "yuval"
                }),
                "archive attribution override",
            ),
        ] {
            write_probe_message(
                &mut stdin,
                &serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "method": "tools/call",
                    "params": {"name": "memory", "arguments": arguments}
                }),
            )
            .await?;
            let response = read_probe_response(&mut stdout, label).await?;
            if response
                .pointer("/error/code")
                .and_then(serde_json::Value::as_i64)
                != Some(-32601)
            {
                bail!("Agent profile did not reject {label}");
            }
        }

        let obsolete_marker = "Use the obsolete contract-probe correction path.";
        write_probe_message(
            &mut stdin,
            &serde_json::json!({
                "jsonrpc": "2.0",
                "id": 9,
                "method": "tools/call",
                "params": {
                    "name": "memory",
                    "arguments": {
                        "action": "add",
                        "kind": "decision",
                        "title": "Obsolete contract-probe correction",
                        "content": obsolete_marker,
                        "origin": "agent_observed",
                        "status": "active",
                        "scope_type": "project",
                        "project_name": project,
                        "writer_harness": "codex",
                        "model_provider": "openai",
                        "model": "contract-probe",
                        "evidence": [{
                            "kind": "tool_call",
                            "target": "agent-profile-contract-probe/obsolete"
                        }]
                    }
                }
            }),
        )
        .await?;
        let obsolete_response = read_probe_response(&mut stdout, "obsolete memory add").await?;
        let obsolete = probe_content_json(&obsolete_response, "obsolete memory add")?;
        let obsolete_id = obsolete
            .pointer("/item/id")
            .and_then(serde_json::Value::as_str)
            .context("Obsolete memory add response omitted item ID")?;

        let replacement_marker = "Use the proposed contract-probe correction path.";
        write_probe_message(
            &mut stdin,
            &serde_json::json!({
                "jsonrpc": "2.0",
                "id": 10,
                "method": "tools/call",
                "params": {
                    "name": "memory",
                    "arguments": {
                        "action": "propose_correction",
                        "id": obsolete_id,
                        "title": "Proposed contract-probe correction",
                        "content": replacement_marker,
                        "writer_harness": "codex",
                        "model_provider": "openai",
                        "model": "contract-probe",
                        "evidence": [{
                            "kind": "session_event",
                            "target": "agent-profile-contract-probe/proposal"
                        }],
                        "scope": {"relevance_mode": "local", "project": project}
                    }
                }
            }),
        )
        .await?;
        let proposal_response = read_probe_response(&mut stdout, "correction proposal").await?;
        let proposal = probe_content_json(&proposal_response, "correction proposal")?;
        let proposal_id = proposal
            .pointer("/proposal_id")
            .and_then(serde_json::Value::as_str)
            .context("Correction proposal response omitted proposal ID")?;
        let replacement_id = proposal
            .pointer("/replacement_id")
            .and_then(serde_json::Value::as_str)
            .context("Correction proposal response omitted replacement ID")?;
        let expected_digest = proposal
            .pointer("/canonical_digest")
            .and_then(serde_json::Value::as_str)
            .context("Correction proposal response omitted canonical digest")?;

        let mut direct_correction_unavailable = false;
        let mut correction_apply_unavailable = false;
        let mut correction_verification_unavailable = false;
        for (id, arguments, label, outcome) in [
            (
                11,
                serde_json::json!({
                    "action": "correct",
                    "id": obsolete_id,
                    "replacement_id": replacement_id,
                    "confirm_correction": true
                }),
                "restricted direct correction",
                &mut direct_correction_unavailable,
            ),
            (
                12,
                serde_json::json!({
                    "action": "apply_correction",
                    "proposal_id": proposal_id,
                    "expected_digest": expected_digest,
                    "scope": {"relevance_mode": "local", "project": project}
                }),
                "restricted correction apply",
                &mut correction_apply_unavailable,
            ),
            (
                13,
                serde_json::json!({
                    "action": "verify_correction_procedure",
                    "proposal_id": proposal_id,
                    "expected_digest": expected_digest,
                    "receipt": "/tmp/not-read-by-agent-profile.json",
                    "expires_at": "2099-01-01T00:00:00Z",
                    "scope": {"relevance_mode": "local", "project": project}
                }),
                "restricted correction procedure verification",
                &mut correction_verification_unavailable,
            ),
        ] {
            write_probe_message(
                &mut stdin,
                &serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "method": "tools/call",
                    "params": {"name": "memory", "arguments": arguments}
                }),
            )
            .await?;
            let response = read_probe_response(&mut stdout, label).await?;
            *outcome = response
                .pointer("/error/code")
                .and_then(serde_json::Value::as_i64)
                == Some(-32601);
            if !*outcome {
                bail!("Agent profile did not reject {label}");
            }
        }

        for (id, action) in [
            (14, "get_correction_proposal"),
            (15, "list_correction_proposals"),
        ] {
            write_probe_message(
                &mut stdin,
                &serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "method": "tools/call",
                    "params": {
                        "name": "memory",
                        "arguments": {
                            "action": action,
                            "proposal_id": proposal_id,
                            "scope": {"relevance_mode": "local", "project": project}
                        }
                    }
                }),
            )
            .await?;
            let response =
                read_probe_response(&mut stdout, "restricted proposal inspection").await?;
            if response
                .pointer("/error/code")
                .and_then(serde_json::Value::as_i64)
                != Some(-32601)
            {
                bail!(
                    "Agent profile did not reject restricted proposal inspection action {action}"
                );
            }
        }
        let correction_inspection_unavailable = true;

        write_probe_message(
            &mut stdin,
            &serde_json::json!({
                "jsonrpc": "2.0",
                "id": 16,
                "method": "tools/call",
                "params": {
                    "name": "memory",
                    "arguments": {
                        "action": "list",
                        "status_filter": "active",
                        "project_name": project,
                        "scope": {"relevance_mode": "local", "project": project}
                    }
                }
            }),
        )
        .await?;
        let active_response = read_probe_response(&mut stdout, "post-proposal active list").await?;
        let active = probe_content_json(&active_response, "post-proposal active list")?;
        let active_text = serde_json::to_string(&active)?;
        let correction_proposal_path_verified = proposal
            .pointer("/replacement/status")
            .and_then(serde_json::Value::as_str)
            == Some("needs_review")
            && proposal.pointer("/obsolete_id")
                == Some(&serde_json::Value::String(obsolete_id.to_string()))
            && proposal.pointer("/replacement_id")
                == Some(&serde_json::Value::String(replacement_id.to_string()))
            && proposal
                .pointer("/intent_verified")
                .and_then(serde_json::Value::as_bool)
                == Some(false)
            && proposal
                .pointer("/reviewer_authority_conferred")
                .and_then(serde_json::Value::as_bool)
                == Some(false)
            && proposal
                .pointer("/scope_selector_match_enforced")
                .and_then(serde_json::Value::as_bool)
                == Some(true)
            && expected_digest.len() == 64
            && active_text.contains(obsolete_marker)
            && !active_text.contains(replacement_marker);
        if !correction_proposal_path_verified {
            bail!("Agent profile did not preserve the inactive correction proposal boundary");
        }

        Ok(AgentProfileRuntimeAttestation {
            verified: true,
            mcp_tool_count: tools.len(),
            mcp_tools_sha256: json_tools_sha256(tools),
            profile_instructions_sha256: format!("{:x}", Sha256::digest(instructions.as_bytes())),
            restricted_tool_rejected,
            review_authority_rejected,
            correction_proposal_path_verified,
            direct_correction_unavailable,
            correction_apply_unavailable,
            correction_verification_unavailable,
            correction_inspection_unavailable,
        })
    }
    .await;

    let _ = child.kill().await;
    let _ = child.wait().await;
    let _ = daemon_child.kill().await;
    let _ = daemon_child.wait().await;

    match probe {
        Ok(attestation) => Ok(attestation),
        Err(error) => Err(error).with_context(|| {
            let stderr = std::fs::read_to_string(&stderr_path).unwrap_or_default();
            format!("Agent-profile runtime probe failed; proxy stderr: {stderr}")
        }),
    }
}

fn write_profile_probe_daemon_metadata(
    path: &Path,
    executable: &Path,
    pid: u32,
    port: u16,
) -> Result<()> {
    let metadata = daemon::DaemonSpawnMetadata::new(executable, pid, port);
    let contents = serde_json::to_vec_pretty(&metadata)
        .context("Failed to serialize agent-profile probe daemon metadata")?;
    std::fs::write(path, contents)
        .context("Failed to write agent-profile probe daemon metadata")?;
    engram_store::ensure_private_file(path)
        .context("Failed to restrict agent-profile probe daemon metadata permissions")?;
    Ok(())
}

async fn write_probe_message(
    stdin: &mut tokio::process::ChildStdin,
    message: &serde_json::Value,
) -> Result<()> {
    stdin
        .write_all(serde_json::to_string(message)?.as_bytes())
        .await?;
    stdin.write_all(b"\n").await?;
    stdin.flush().await?;
    Ok(())
}

async fn read_probe_response(
    stdout: &mut BufReader<tokio::process::ChildStdout>,
    label: &str,
) -> Result<serde_json::Value> {
    let mut line = String::new();
    let read = tokio::time::timeout(PROFILE_PROBE_TIMEOUT, stdout.read_line(&mut line))
        .await
        .with_context(|| format!("Timed out waiting for {label} response"))??;
    if read == 0 {
        bail!("Agent-profile proxy exited before {label} response");
    }
    serde_json::from_str(line.trim())
        .with_context(|| format!("Agent-profile {label} response was not valid JSON"))
}

fn probe_content_json(response: &serde_json::Value, label: &str) -> Result<serde_json::Value> {
    if response
        .pointer("/result/isError")
        .and_then(serde_json::Value::as_bool)
        == Some(true)
    {
        bail!("Agent-profile {label} returned an MCP tool error: {response}");
    }
    let text = response
        .pointer("/result/content/0/text")
        .and_then(serde_json::Value::as_str)
        .with_context(|| format!("Agent-profile {label} response omitted text content"))?;
    serde_json::from_str(text)
        .with_context(|| format!("Agent-profile {label} text was not valid JSON"))
}

fn json_tools_sha256(tools: &[serde_json::Value]) -> String {
    let mut tools = tools.to_vec();
    tools.sort_by(|left, right| left["name"].as_str().cmp(&right["name"].as_str()));
    let encoded = serde_json::to_vec(&tools).expect("agent MCP tool contract should serialize");
    format!("{:x}", Sha256::digest(encoded))
}

fn agent_tool_declaration(tool: serde_json::Value) -> Option<serde_json::Value> {
    let name = tool.get("name")?.as_str()?;
    match name {
        "orient" => Some(serde_json::json!({
            "name": "orient",
            "description": "Resolve the current repository/project/task and return compact, scoped engineering context. Call at task start with the full user prompt. Ambiguity or a task/checkout mismatch blocks project/task memory, but not repository-local procedure_match.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "cwd": {"type": "string", "description": "Current working directory."},
                    "project": {"type": "string", "description": "Project name when known."},
                    "task": {"type": "string", "description": "Exact task name or tracker key when known; task names require project."},
                    "prompt": {"type": "string", "description": "Current user task."},
                    "agent": {"type": "string"},
                    "intent": {"type": "string"},
                    "external_session_id": {"type": "string"},
                    "scenario_id": {"type": "string"},
                    "arm": {"type": "string"},
                    "include_recent_commits": {"type": "boolean"},
                    "limit": {"type": "integer", "minimum": 1},
                    "response_shape": {"type": "string", "enum": ["lean", "full"], "default": "lean"}
                },
                "additionalProperties": false
            }
        })),
        "memory" => Some(serde_json::json!({
            "name": "memory",
            "description": "Match verified procedures, or add/read/propose/archive/forget durable scoped memory. Agent adds must use truthful agent_observed or agent_inferred origin and cannot assert user provenance or manual review. propose_correction creates an inactive needs_review replacement whose kind and memory scope are derived server-side from one exact active item matching a caller-selected local or related project/task/cwd selector. A structured procedure replacement may be proposed but remains unverified and requires full-profile verification and apply. The profile is not an authentication boundary and does not prevent semantically conflicting autonomous adds or deletion. procedure_match reads declarative Git-tracked prerequisite sources from the current checkout and abstains unless evidence, scope, expiry, and prerequisites all pass.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "action": {"type": "string", "enum": ["add", "propose_correction", "procedure_match", "get", "list", "archive", "forget"]},
                    "id": {"type": "string"},
                    "kind": {"type": "string", "enum": ["preference", "rule", "decision", "limitation", "project_fact", "repository_fact", "task_fact", "user_fact", "session_insight", "handoff", "procedure"]},
                    "title": {"type": "string"},
                    "content": {"type": "string"},
                    "origin": {"type": "string", "enum": ["agent_observed", "agent_inferred"]},
                    "status": {"type": "string", "enum": ["active", "needs_review"]},
                    "confidence": {"type": "number", "minimum": 0, "maximum": 1},
                    "tags": {"type": "array", "items": {"type": "string"}},
                    "scope_type": {"type": "string", "enum": ["global", "user", "project", "task", "entity", "repository", "session", "custom"]},
                    "project_name": {"type": "string"},
                    "task_name": {"type": "string"},
                    "entity_name": {"type": "string"},
                    "repository_id": {"type": "string"},
                    "remote_url": {"type": "string"},
                    "local_path": {"type": "string"},
                    "scope_session_id": {"type": "string"},
                    "scope_name": {"type": "string"},
                    "writer_harness": {"type": "string"},
                    "model_provider": {"type": "string"},
                    "model": {"type": "string"},
                    "evidence": {"type": "array", "items": {"type": "object", "properties": {"kind": {"type": "string", "enum": ["session_event", "tool_call", "file", "git_commit", "url", "document", "observation"]}, "target": {"type": "string"}, "summary": {"type": "string"}, "excerpt": {"type": "string"}}, "required": ["kind", "target"], "additionalProperties": false}},
                    "procedure": {"type": "object", "properties": {"task": {"type": "string"}, "commands": {"type": "array", "items": {"type": "string"}}, "prerequisites": {"type": "object", "additionalProperties": {"type": "string"}}, "prerequisite_sources": {"type": "object", "additionalProperties": {"type": "object", "properties": {"format": {"type": "string", "enum": ["toml"]}, "relative_path": {"type": "string"}, "key_path": {"type": "array", "items": {"type": "string"}}}, "required": ["format", "relative_path", "key_path"], "additionalProperties": false}}, "failure_signatures": {"type": "array", "items": {"type": "string"}}, "verification_command": {"type": "string"}, "verification_exit_code": {"type": "integer"}, "verification_output_contains": {"type": "string"}}, "required": ["task", "commands", "verification_command", "verification_output_contains"], "additionalProperties": false},
                    "query": {"type": "string", "maxLength": 512, "description": "Required for procedure_match; copy a bounded task-focused excerpt from the current user request. Preserve concrete operation terms and identifiers verbatim; omit unrelated instructions and secret values. This is retrieval text, not authorization. memory action=list is not a substitute."},
                    "conditions": {"type": "object", "additionalProperties": {"type": "string"}},
                    "scope": {"type": "object", "properties": {
                        "relevance_mode": {"type": "string", "enum": ["local", "related", "global"]},
                        "project": {"type": "string"},
                        "task": {"type": "string"},
                        "cwd": {"type": "string"}
                    }, "additionalProperties": false},
                    "cwd": {"type": "string"},
                    "relevance_project": {"type": "string"},
                    "status_filter": {"type": "string"},
                    "limit": {"type": "integer", "minimum": 1},
                    "archive_reason": {"type": "string"},
                    "confirm_forget": {"type": "boolean"},
                    "vault_path": {"type": "string"}
                },
                "required": ["action"],
                "additionalProperties": false
            }
        })),
        "repo" => Some(serde_json::json!({
            "name": "repo",
            "description": "Resolve or register stable repository identity and link it to a project. Prefer remote identity over checkout path.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "action": {"type": "string", "enum": ["detect", "context", "register", "list", "link_project"]},
                    "cwd": {"type": "string"},
                    "repository_id": {"type": "string"},
                    "repository_name": {"type": "string"},
                    "remote_url": {"type": "string"},
                    "default_branch": {"type": "string"},
                    "description": {"type": "string"},
                    "project_name": {"type": "string"},
                    "role": {"type": "string", "enum": ["primary", "dependency", "produces", "related"]},
                    "limit": {"type": "integer", "minimum": 1}
                },
                "required": ["action"],
                "additionalProperties": false
            }
        })),
        "search" => Some(serde_json::json!({
            "name": "search",
            "description": "Progressively search Engram evidence when orientation is insufficient. Scope memory results with project when known.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": {"type": "string"},
                    "project": {"type": "string"},
                    "task": {"type": "string"},
                    "cwd": {"type": "string"},
                    "relevance_mode": {"type": "string", "enum": ["local", "related", "global"]},
                    "layers": {"type": "array", "items": {"type": "string", "enum": ["entity", "alias", "observation", "session_event", "document", "tool_usage", "memory"]}},
                    "limit": {"type": "integer", "minimum": 1, "default": 5},
                    "min_score": {"type": "number", "minimum": 0, "maximum": 1},
                    "intent": {"type": "string"},
                    "scenario_id": {"type": "string"},
                    "arm": {"type": "string"},
                    "agent": {"type": "string"},
                    "external_session_id": {"type": "string"}
                },
                "required": ["query"],
                "additionalProperties": false
            }
        })),
        "harness" => Some(serde_json::json!({
            "name": "harness",
            "description": "Process a host lifecycle hook. Intended for generated Engram adapters; normal coding-agent work should start with orient.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "action": {"type": "string", "enum": ["hook_event"]},
                    "harness": {"type": "string"},
                    "enforcement": {"type": "string", "enum": ["soft", "graduated", "strict"]},
                    "hook_event_name": {"type": "string"},
                    "session_id": {"type": "string"},
                    "cwd": {"type": "string"},
                    "transcript_path": {"type": "string"},
                    "prompt": {"type": "string"},
                    "tool_name": {"type": "string"},
                    "tool_error": {"type": "string"},
                    "tool_input_command": {"type": "string"},
                    "file_path": {"type": "string"},
                    "last_assistant_message": {"type": "string"},
                    "compact_summary": {"type": "string"},
                    "trigger": {"type": "string"},
                    "reason": {"type": "string"},
                    "stop_hook_active": {"type": "boolean"},
                    "write_policy": {"type": "string", "enum": ["nudge", "durable"]},
                    "project": {"type": "string"},
                    "model_provider": {"type": "string"},
                    "model": {"type": "string"},
                    "surface": {"type": "string"},
                    "actor": {"type": "string"}
                },
                "required": ["action", "harness", "hook_event_name"],
                "additionalProperties": false
            }
        })),
        "obligations" => Some(serde_json::json!({
            "name": "obligations",
            "description": "Detect, inspect, resolve, or explicitly skip high-value lifecycle obligations. Used by graduated/strict host adapters.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "action": {"type": "string", "enum": ["detect", "doctor", "resolve", "skip"]},
                    "cwd": {"type": "string"},
                    "prompt": {"type": "string"},
                    "project": {"type": "string"},
                    "limit": {"type": "integer", "minimum": 1},
                    "write": {"type": "boolean"},
                    "id": {"type": "string"},
                    "resolution": {"type": "string"},
                    "summary": {"type": "string"},
                    "reason": {"type": "string"},
                    "evidence": {"type": "array", "items": {"type": "object", "properties": {"kind": {"type": "string"}, "target": {"type": "string"}, "summary": {"type": "string"}, "excerpt": {"type": "string"}}, "required": ["kind", "target"], "additionalProperties": false}},
                    "writer_harness": {"type": "string"},
                    "model_provider": {"type": "string"},
                    "model": {"type": "string"},
                    "surface": {"type": "string"},
                    "actor": {"type": "string"},
                    "writer_session_id": {"type": "string"}
                },
                "required": ["action"],
                "additionalProperties": false
            }
        })),
        _ => None,
    }
}

fn daemon_http_client(timeout: Duration, auth_token: Option<&str>) -> Result<reqwest::Client> {
    let mut builder = reqwest::Client::builder()
        .timeout(timeout)
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none());
    if let Some(token) = auth_token {
        let mut headers = reqwest::header::HeaderMap::new();
        let mut value = reqwest::header::HeaderValue::from_str(&format!("Bearer {token}"))
            .context("Daemon bearer token is not a valid HTTP header value")?;
        value.set_sensitive(true);
        headers.insert(reqwest::header::AUTHORIZATION, value);
        builder = builder.default_headers(headers);
    }
    builder.build().context("Failed to create HTTP client")
}

/// Forward an initialize request and capture the session ID from response headers.
async fn forward_initialize(
    client: &reqwest::Client,
    mcp_url: &str,
    request: &serde_json::Value,
) -> Result<(serde_json::Value, Option<String>)> {
    let response = client
        .post(mcp_url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(request)
        .send()
        .await
        .context("Failed to send initialize request to daemon")?;

    // Capture session ID from response headers
    let session_id = response
        .headers()
        .get("mcp-session-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    if !response.status().is_success() {
        return Err(daemon_http_status_error("initialize request", &response));
    }

    // Parse response body
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let body = if content_type.contains("text/event-stream") {
        handle_sse_response(response).await?
    } else {
        response
            .json()
            .await
            .context("Failed to parse daemon response")?
    };

    Ok((body, session_id))
}

/// Forward a notification (no response expected).
async fn forward_notification(
    client: &reqwest::Client,
    mcp_url: &str,
    session_id: &Option<String>,
    request: &serde_json::Value,
) -> Result<()> {
    let mut req_builder = client
        .post(mcp_url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(request);

    if let Some(sid) = session_id {
        req_builder = req_builder.header("mcp-session-id", sid);
    }

    let response = req_builder
        .send()
        .await
        .context("Failed to send notification to daemon")?;

    if !response.status().is_success() {
        return Err(daemon_http_status_error("notification", &response));
    }

    Ok(())
}

/// Forward a JSON-RPC request to the daemon.
async fn forward_request(
    client: &reqwest::Client,
    mcp_url: &str,
    session_id: &Option<String>,
    request: &serde_json::Value,
) -> Result<serde_json::Value> {
    let mut req_builder = client
        .post(mcp_url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(request);

    // Add session ID header if we have one
    if let Some(sid) = session_id {
        req_builder = req_builder.header("mcp-session-id", sid);
    }

    let response = req_builder
        .send()
        .await
        .context("Failed to send request to daemon")?;

    if !response.status().is_success() {
        return Err(daemon_http_status_error("request", &response));
    }

    // Check content type
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if content_type.contains("text/event-stream") {
        // SSE response - need to handle streaming
        handle_sse_response(response).await
    } else {
        // JSON response
        response
            .json()
            .await
            .context("Failed to parse daemon response")
    }
}

/// Forward a JSON-RPC request and recover once from a stale HTTP MCP session.
async fn forward_request_with_recovery(
    client: &reqwest::Client,
    mcp_url: &str,
    state: &Arc<Mutex<ProxyState>>,
    request: &serde_json::Value,
) -> Result<serde_json::Value> {
    let session_id = state.lock().await.session_id.clone();
    match forward_request(client, mcp_url, &session_id, request).await {
        Ok(response) => Ok(response),
        Err(error) if is_stale_session_error(&error) => {
            info!("HTTP MCP session is stale; refreshing proxy session");
            refresh_session(client, mcp_url, state).await?;
            let session_id = state.lock().await.session_id.clone();
            forward_request(client, mcp_url, &session_id, request).await
        }
        Err(error) => Err(error),
    }
}

async fn refresh_session(
    client: &reqwest::Client,
    mcp_url: &str,
    state: &Arc<Mutex<ProxyState>>,
) -> Result<()> {
    let initialize_request = {
        let s = state.lock().await;
        s.initialize_request
            .clone()
            .context("Cannot refresh HTTP MCP session before initialize")?
    };

    let (_, session_id) = forward_initialize(client, mcp_url, &initialize_request).await?;
    {
        let mut s = state.lock().await;
        s.session_id = session_id;
        s.initialized = false;
    }

    let initialized_notification = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    let session_id = state.lock().await.session_id.clone();
    forward_notification(client, mcp_url, &session_id, &initialized_notification).await?;

    let mut s = state.lock().await;
    s.initialized = true;
    Ok(())
}

fn is_stale_session_error(error: &anyhow::Error) -> bool {
    let message = error.to_string();
    message.contains("request failed: HTTP 401 Unauthorized")
}

fn daemon_http_status_error(operation: &str, response: &reqwest::Response) -> anyhow::Error {
    let response_length = response
        .content_length()
        .map_or_else(|| "unknown".to_string(), |length| length.to_string());
    anyhow::anyhow!(
        "Daemon {operation} failed: HTTP {}; response_length={response_length}",
        response.status()
    )
}

/// Handle an SSE (Server-Sent Events) response.
async fn handle_sse_response(response: reqwest::Response) -> Result<serde_json::Value> {
    let body = response.text().await?;

    // Parse SSE events - look for the last "data:" line with content
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

/// Delete a session.
async fn delete_session(client: &reqwest::Client, mcp_url: &str, session_id: &str) -> Result<()> {
    let _ = client
        .delete(mcp_url)
        .header("mcp-session-id", session_id)
        .send()
        .await;
    Ok(())
}

/// Create an error response for a failed request.
fn create_error_response(request: &serde_json::Value, error_message: &str) -> serde_json::Value {
    let id = request
        .get("id")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": -32603,
            "message": error_message
        }
    })
}

fn create_profile_error_response(
    request: &serde_json::Value,
    error_message: &str,
) -> serde_json::Value {
    let id = request
        .get("id")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": -32601,
            "message": error_message
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_profile_probe_writes_complete_private_daemon_metadata() {
        let directory = tempfile::tempdir().expect("create metadata fixture directory");
        let path = directory.path().join("daemon.meta.json");
        let executable = std::env::current_exe()
            .expect("resolve test executable")
            .canonicalize()
            .expect("canonicalize test executable");

        write_profile_probe_daemon_metadata(&path, &executable, 12_345, 9_876)
            .expect("write profile-probe daemon metadata");

        let bytes = std::fs::read(&path).expect("read profile-probe daemon metadata");
        let decoded: daemon::DaemonSpawnMetadata =
            serde_json::from_slice(&bytes).expect("decode profile-probe daemon metadata");
        assert_eq!(
            decoded,
            daemon::DaemonSpawnMetadata::new(&executable, 12_345, 9_876)
        );
        let object = serde_json::from_slice::<serde_json::Value>(&bytes)
            .expect("decode exact profile-probe daemon metadata")
            .as_object()
            .expect("profile-probe daemon metadata should be an object")
            .clone();
        assert_eq!(object.len(), 6);
        assert_eq!(object["schema_version"], 1);
        assert_eq!(object["pid"], 12_345);
        assert_eq!(object["port"], 9_876);
        assert!(object["executable_sha256"].is_string());

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&path)
                    .expect("stat profile-probe daemon metadata")
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }
    }

    fn validate_adapter_fixture_against_tools(
        fixture: &serde_json::Value,
        tools: &[serde_json::Value],
    ) -> std::result::Result<(), String> {
        for harness in fixture["harnesses"]
            .as_array()
            .ok_or_else(|| "fixture should declare harnesses".to_string())?
        {
            for call in harness["calls"]
                .as_array()
                .ok_or_else(|| "harness fixture should declare calls".to_string())?
            {
                let tool_name = call["tool"]
                    .as_str()
                    .ok_or_else(|| "fixture call should declare a tool".to_string())?;
                let tool = tools
                    .iter()
                    .find(|tool| tool["name"] == tool_name)
                    .ok_or_else(|| format!("effective profile omits fixture tool '{tool_name}'"))?;
                let properties = tool["inputSchema"]["properties"]
                    .as_object()
                    .ok_or_else(|| format!("{tool_name} should expose input properties"))?;
                let arguments = call["arguments"]
                    .as_object()
                    .ok_or_else(|| format!("{tool_name} fixture arguments should be an object"))?;
                for name in arguments.keys() {
                    if !properties.contains_key(name) {
                        return Err(format!(
                            "effective {tool_name} profile omits fixture argument '{name}'"
                        ));
                    }
                }
                for required in tool["inputSchema"]["required"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(serde_json::Value::as_str)
                {
                    if !arguments.contains_key(required) {
                        return Err(format!(
                            "{tool_name} fixture omits required argument '{required}'"
                        ));
                    }
                }
                if let Some(scope) = arguments
                    .get("scope")
                    .and_then(serde_json::Value::as_object)
                {
                    let scope_properties = properties["scope"]["properties"]
                        .as_object()
                        .ok_or_else(|| {
                            format!("effective {tool_name} profile should expose scope properties")
                        })?;
                    for name in scope.keys() {
                        if !scope_properties.contains_key(name) {
                            return Err(format!(
                                "effective {tool_name}.scope profile omits fixture argument '{name}'"
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn effective_agent_tools() -> Vec<serde_json::Value> {
        let request = serde_json::json!({"method": "tools/list"});
        let mut response = serde_json::json!({
            "result": {
                "tools": [
                    {"name": "harness"},
                    {"name": "memory"},
                    {"name": "obligations"},
                    {"name": "orient"},
                    {"name": "repo"},
                    {"name": "search"}
                ]
            }
        });
        apply_profile_response(ToolProfile::Agent, &request, &mut response);
        response["result"]["tools"].as_array().unwrap().clone()
    }

    #[test]
    fn agent_profile_replaces_instructions_and_exposes_six_compact_tools() {
        let request = serde_json::json!({"method": "initialize"});
        let mut response = serde_json::json!({"result": {"instructions": "long"}});
        apply_profile_response(ToolProfile::Agent, &request, &mut response);
        assert_eq!(
            response
                .pointer("/result/instructions")
                .and_then(|v| v.as_str()),
            Some(AGENT_PROFILE_INSTRUCTIONS)
        );

        let request = serde_json::json!({"method": "tools/list"});
        let mut response = serde_json::json!({
            "result": {
                "tools": [
                    {"name": "docs", "description": "admin", "inputSchema": {}},
                    {"name": "harness", "description": "large", "inputSchema": {}},
                    {"name": "memory", "description": "large", "inputSchema": {}},
                    {"name": "obligations", "description": "large", "inputSchema": {}},
                    {"name": "orient", "description": "large", "inputSchema": {}},
                    {"name": "repo", "description": "large", "inputSchema": {}},
                    {"name": "search", "description": "large", "inputSchema": {}}
                ]
            }
        });
        apply_profile_response(ToolProfile::Agent, &request, &mut response);

        let tools = response
            .pointer("/result/tools")
            .and_then(|tools| tools.as_array())
            .unwrap();
        let names: Vec<_> = tools
            .iter()
            .filter_map(|tool| tool.get("name").and_then(|name| name.as_str()))
            .collect();
        assert_eq!(
            names,
            [
                "harness",
                "memory",
                "obligations",
                "orient",
                "repo",
                "search"
            ]
        );
        assert_eq!(tools.as_slice(), agent_profile_tool_contract().as_slice());
        let orient = tools
            .iter()
            .find(|tool| tool["name"] == "orient")
            .expect("agent profile should expose orient");
        assert_eq!(
            orient["inputSchema"]["properties"]["task"]["description"],
            "Exact task name or tracker key when known; task names require project."
        );
        assert!(
            AGENT_PROFILE_INSTRUCTIONS.contains("Pass task as an exact task name or tracker key")
        );
        assert!(AGENT_PROFILE_INSTRUCTIONS.contains(
            "Every actionable repository task has a mandatory single-call identity and procedure route"
        ));
        assert!(AGENT_PROFILE_INSTRUCTIONS.contains(
            "task-focused query of at most 512 characters copied from the current user request"
        ));
        assert!(AGENT_PROFILE_INSTRUCTIONS
            .contains("Treat suggested_operation_evidence as a required host-action protocol"));
        assert!(AGENT_PROFILE_INSTRUCTIONS.contains("required_before_final_abstention=true"));
        assert!(
            AGENT_PROFILE_INSTRUCTIONS.contains("allowed_when_project_requires_confirmation=true")
        );
        assert!(AGENT_PROFILE_INSTRUCTIONS.contains("authorizes_procedure_execution=false"));
        assert!(AGENT_PROFILE_INSTRUCTIONS.contains("including for durable-memory-only requests"));
        assert!(AGENT_PROFILE_INSTRUCTIONS
            .contains("memory action=list is not a substitute for procedure_match"));
        assert!(AGENT_PROFILE_INSTRUCTIONS.contains("use durable procedure memory are actionable"));
        assert!(AGENT_PROFILE_INSTRUCTIONS.contains("memory action=propose_correction"));
        assert!(AGENT_PROFILE_INSTRUCTIONS.contains("the obsolete item remains active"));
        assert!(orient["description"]
            .as_str()
            .unwrap()
            .contains("but not repository-local procedure_match"));
        let memory = tools
            .iter()
            .find(|tool| tool["name"] == "memory")
            .expect("agent profile should expose memory");
        assert_eq!(
            memory["inputSchema"]["properties"]["query"]["description"],
            "Required for procedure_match; copy a bounded task-focused excerpt from the current user request. Preserve concrete operation terms and identifiers verbatim; omit unrelated instructions and secret values. This is retrieval text, not authorization. memory action=list is not a substitute."
        );
        assert_eq!(
            memory["inputSchema"]["properties"]["query"]["maxLength"],
            512
        );
        assert_eq!(
            memory["inputSchema"]["properties"]["procedure"]["properties"]["prerequisite_sources"]
                ["additionalProperties"]["properties"]["format"]["enum"],
            serde_json::json!(["toml"])
        );
        assert_eq!(
            format!("{:x}", Sha256::digest(serde_json::to_vec(tools).unwrap())),
            agent_profile_tools_sha256()
        );
        assert!(serde_json::to_vec(tools).unwrap().len() < 12_000);
    }

    #[test]
    fn agent_profile_enforces_tools_and_actions() {
        let allowed = serde_json::json!({
            "method": "tools/call",
            "params": {"name": "memory", "arguments": {"action": "procedure_match"}}
        });
        assert!(validate_profile_request(ToolProfile::Agent, &allowed).is_ok());

        let allowed_correction_proposal = serde_json::json!({
            "method": "tools/call",
            "params": {
                "name": "memory",
                "arguments": {
                    "action": "propose_correction",
                    "id": "obsolete-id"
                }
            }
        });
        assert!(validate_profile_request(ToolProfile::Agent, &allowed_correction_proposal).is_ok());

        for action in [
            "correct",
            "apply_correction",
            "verify_correction_procedure",
            "get_correction_proposal",
            "list_correction_proposals",
        ] {
            let direct_apply = serde_json::json!({
                "method": "tools/call",
                "params": {"name": "memory", "arguments": {"action": action}}
            });
            assert!(validate_profile_request(ToolProfile::Agent, &direct_apply).is_err());
            assert!(validate_profile_request(ToolProfile::Full, &direct_apply).is_ok());
        }

        let legacy_supersede = serde_json::json!({
            "method": "tools/call",
            "params": {"name": "memory", "arguments": {"action": "supersede"}}
        });
        assert!(validate_profile_request(ToolProfile::Agent, &legacy_supersede).is_err());

        let hidden_tool = serde_json::json!({
            "method": "tools/call",
            "params": {"name": "docs", "arguments": {"action": "search"}}
        });
        assert!(validate_profile_request(ToolProfile::Agent, &hidden_tool).is_err());

        let hidden_action = serde_json::json!({
            "method": "tools/call",
            "params": {"name": "memory", "arguments": {"action": "migration_review_apply"}}
        });
        assert!(validate_profile_request(ToolProfile::Agent, &hidden_action).is_err());
        assert!(validate_profile_request(ToolProfile::Full, &hidden_action).is_ok());

        let source_evidence = serde_json::json!({
            "method": "tools/call",
            "params": {
                "name": "memory",
                "arguments": {
                    "action": "add",
                    "origin": "agent_observed",
                    "status": "active",
                    "evidence": [{"kind": "file", "target": "docs/decision.md"}]
                }
            }
        });
        assert!(validate_profile_request(ToolProfile::Agent, &source_evidence).is_ok());

        for (field, value) in [
            ("origin", serde_json::json!("user_corrected")),
            ("origin", serde_json::json!("user_stated")),
            ("origin", serde_json::json!("tool_result")),
            ("actor", serde_json::json!("user")),
        ] {
            let mut unsafe_add = serde_json::json!({
                "method": "tools/call",
                "params": {"name": "memory", "arguments": {"action": "add"}}
            });
            unsafe_add["params"]["arguments"][field] = value;
            assert!(validate_profile_request(ToolProfile::Agent, &unsafe_add).is_err());
            assert!(validate_profile_request(ToolProfile::Full, &unsafe_add).is_ok());
        }

        let asserted_review = serde_json::json!({
            "method": "tools/call",
            "params": {
                "name": "memory",
                "arguments": {
                    "action": "add",
                    "evidence": [{"kind": "MANUAL_REVIEW", "target": "claimed-reviewer"}]
                }
            }
        });
        let error = validate_profile_request(ToolProfile::Agent, &asserted_review).unwrap_err();
        assert!(error.contains("cannot assert reviewer authority"));
        assert!(validate_profile_request(ToolProfile::Full, &asserted_review).is_ok());

        let attributed_archive = serde_json::json!({
            "method": "tools/call",
            "params": {
                "name": "memory",
                "arguments": {
                    "action": "archive",
                    "id": "memory-id",
                    "archived_by": "yuval"
                }
            }
        });
        let error = validate_profile_request(ToolProfile::Agent, &attributed_archive).unwrap_err();
        assert!(error.contains("archived_by attribution"));
        assert!(validate_profile_request(ToolProfile::Full, &attributed_archive).is_ok());

        let proposal_with_derived_fields = serde_json::json!({
            "method": "tools/call",
            "params": {
                "name": "memory",
                "arguments": {
                    "action": "propose_correction",
                    "origin": "user_corrected"
                }
            }
        });
        let error = validate_profile_request(ToolProfile::Agent, &proposal_with_derived_fields)
            .unwrap_err();
        assert!(error.contains("derives actor, origin, status, kind"));

        let procedure_proposal = serde_json::json!({
            "method": "tools/call",
            "params": {
                "name": "memory",
                "arguments": {
                    "action": "propose_correction",
                    "procedure": {
                        "task": "run focused tests",
                        "commands": ["cargo test -p engram-index"],
                        "verification_command": "cargo test -p engram-index",
                        "verification_output_contains": "test result: ok"
                    }
                }
            }
        });
        assert!(validate_profile_request(ToolProfile::Agent, &procedure_proposal).is_ok());

        let memory_schema = agent_profile_tool_contract()
            .into_iter()
            .find(|tool| tool["name"] == "memory")
            .unwrap();
        let evidence_kinds = memory_schema
            .pointer("/inputSchema/properties/evidence/items/properties/kind/enum")
            .and_then(serde_json::Value::as_array)
            .unwrap();
        assert!(!evidence_kinds
            .iter()
            .any(|kind| kind.as_str() == Some("manual_review")));
        assert_eq!(
            memory_schema["inputSchema"]["properties"]["origin"]["enum"],
            serde_json::json!(["agent_observed", "agent_inferred"])
        );
        let missing_origin = serde_json::json!({
            "method": "tools/call",
            "params": {"name": "memory", "arguments": {"action": "add"}}
        });
        let error = validate_profile_request(ToolProfile::Agent, &missing_origin).unwrap_err();
        assert!(error.contains("requires truthful origin"));
        let actions = memory_schema
            .pointer("/inputSchema/properties/action/enum")
            .and_then(serde_json::Value::as_array)
            .unwrap();
        assert!(actions
            .iter()
            .any(|action| action.as_str() == Some("propose_correction")));
        assert!(!actions.iter().any(|action| {
            matches!(
                action.as_str(),
                Some(
                    "correct"
                        | "apply_correction"
                        | "verify_correction_procedure"
                        | "get_correction_proposal"
                        | "list_correction_proposals"
                )
            )
        }));
        assert!(!memory_schema["inputSchema"]["properties"]
            .as_object()
            .unwrap()
            .contains_key("replacement_id"));
        for operator_only_field in ["proposal_id", "expected_digest", "receipt", "expires_at"] {
            assert!(!memory_schema["inputSchema"]["properties"]
                .as_object()
                .unwrap()
                .contains_key(operator_only_field));
        }
        assert!(!memory_schema["inputSchema"]["properties"]
            .as_object()
            .unwrap()
            .contains_key("archived_by"));
    }

    #[test]
    fn generated_adapter_fixtures_match_effective_agent_profile() {
        let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../evals/adapter_contract_v1/fixtures.json");
        let fixture: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(fixture_path).expect("adapter fixture should be readable"),
        )
        .expect("adapter fixture should be valid JSON");
        let tools = effective_agent_tools();
        validate_adapter_fixture_against_tools(&fixture, &tools)
            .expect("generated adapter fixtures should match the effective agent profile");

        for (tool_name, field) in [("memory", "scope"), ("search", "relevance_mode")] {
            let mut drifted = tools.clone();
            let tool = drifted
                .iter_mut()
                .find(|tool| tool["name"] == tool_name)
                .unwrap();
            tool["inputSchema"]["properties"]
                .as_object_mut()
                .unwrap()
                .remove(field);
            let error = validate_adapter_fixture_against_tools(&fixture, &drifted)
                .expect_err("an unscoped effective profile should fail the adapter contract");
            assert!(error.contains(field));
        }
    }

    #[tokio::test]
    async fn bearer_loopback_client_bypasses_poisoned_system_proxy_environment() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let direct = tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .expect("bind direct MCP endpoint");
        let direct_port = direct.local_addr().expect("direct address").port();
        let direct_request = tokio::spawn(async move {
            let (mut stream, _) = direct.accept().await.expect("accept direct request");
            let mut request = Vec::new();
            let mut byte = [0_u8; 1];
            while !request.ends_with(b"\r\n\r\n") {
                stream
                    .read_exact(&mut byte)
                    .await
                    .expect("read direct request");
                request.push(byte[0]);
                assert!(request.len() <= 16 * 1024);
            }
            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: 2\r\nconnection: close\r\n\r\n{}",
                )
                .await
                .expect("write direct response");
            String::from_utf8(request).expect("request headers are UTF-8")
        });
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
        let proxy_environment = crate::daemon::PoisonedProxyEnvironment::install(&poison_url);

        let response = daemon_http_client(Duration::from_secs(1), Some("secret-token"))
            .expect("build direct bearer client")
            .get(format!("http://127.0.0.1:{direct_port}/mcp"))
            .send()
            .await
            .expect("bearer request must go directly to loopback");
        drop(proxy_environment);

        assert_eq!(response.status(), reqwest::StatusCode::OK);
        let request = direct_request.await.expect("direct request capture");
        assert!(request
            .to_ascii_lowercase()
            .contains("authorization: bearer secret-token\r\n"));
        assert!(!poison_attempt.await.expect("poison proxy observation"));
    }

    #[tokio::test]
    async fn authenticated_client_debug_and_reflected_error_never_expose_bearer() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let bearer = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .expect("bind malicious daemon");
        let port = listener
            .local_addr()
            .expect("malicious daemon address")
            .port();
        let response_body = format!("malicious reflection: Bearer {bearer}");
        let response_length = response_body.len();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept malicious request");
            let mut request = Vec::new();
            let mut byte = [0_u8; 1];
            while !request.ends_with(b"\r\n\r\n") {
                stream
                    .read_exact(&mut byte)
                    .await
                    .expect("read malicious request headers");
                request.push(byte[0]);
                assert!(request.len() <= 16 * 1024);
            }
            let header = format!(
                "HTTP/1.1 500 Internal Server Error\r\ncontent-type: text/plain\r\n\
                 content-length: {response_length}\r\nconnection: close\r\n\r\n"
            );
            stream
                .write_all(header.as_bytes())
                .await
                .expect("write malicious response header");
            stream
                .write_all(response_body.as_bytes())
                .await
                .expect("write malicious reflected body");
        });
        let client = daemon_http_client(Duration::from_secs(1), Some(bearer))
            .expect("build authenticated client");

        let client_debug = format!("{client:?}");
        assert!(!client_debug.contains(bearer));
        let error = forward_request(
            &client,
            &format!("http://127.0.0.1:{port}/mcp"),
            &None,
            &serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": "ping"}),
        )
        .await
        .expect_err("malicious daemon response must fail structurally");
        server.await.expect("malicious daemon completed");

        let rendered = error.to_string();
        assert!(rendered.contains("HTTP 500 Internal Server Error"));
        assert!(rendered.contains(&format!("response_length={response_length}")));
        assert!(!rendered.contains(bearer));
        assert!(!rendered.contains("malicious reflection"));
    }

    #[test]
    fn proxy_config_debug_redacts_bearer_token() {
        let rendered = format!(
            "{:?}",
            ProxyConfig::new(8765, Some("do-not-render-this-token".to_string()))
        );
        assert!(rendered.contains("<redacted>"));
        assert!(!rendered.contains("do-not-render-this-token"));
    }
}
