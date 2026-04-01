//! Stdio-to-HTTP proxy for transparent daemon access.
//!
//! This module bridges MCP stdio clients to the HTTP daemon, making the daemon
//! transparent to clients that only support stdio transport.

use anyhow::{bail, Context, Result};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;
use tracing::{debug, error, info, trace};

/// Configuration for the proxy.
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    /// The daemon port to connect to.
    pub daemon_port: u16,
    /// Request timeout.
    pub timeout: Duration,
}

impl ProxyConfig {
    pub fn new(daemon_port: u16) -> Self {
        Self {
            daemon_port,
            timeout: Duration::from_secs(300), // 5 minute timeout for long operations
        }
    }
}

/// Proxy state for managing MCP session.
struct ProxyState {
    session_id: Option<String>,
    initialized: bool,
    /// Queue of requests received before initialization completed.
    pending_requests: Vec<serde_json::Value>,
}

/// Run the stdio-to-HTTP proxy.
///
/// This function reads JSON-RPC messages from stdin, forwards them to the
/// HTTP daemon, and writes responses to stdout. It runs until stdin is closed.
pub async fn run_proxy(config: ProxyConfig) -> Result<()> {
    info!("Starting proxy to daemon on port {}", config.daemon_port);

    let client = reqwest::Client::builder()
        .timeout(config.timeout)
        .build()
        .context("Failed to create HTTP client")?;

    let mcp_url = format!("http://127.0.0.1:{}/mcp", config.daemon_port);

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin);
    let mut writer = stdout;

    // Proxy state for session management
    let state = Arc::new(Mutex::new(ProxyState {
        session_id: None,
        initialized: false,
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
                            Ok((response, session_id)) => {
                                // Store session ID
                                {
                                    let mut s = state.lock().await;
                                    s.session_id = session_id;
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
                                    let session_id = state.lock().await.session_id.clone();
                                    for pending_req in pending {
                                        let is_notification = pending_req.get("id").is_none();
                                        match forward_request(
                                            &client,
                                            &mcp_url,
                                            &session_id,
                                            &pending_req,
                                        )
                                        .await
                                        {
                                            Ok(response) => {
                                                if !is_notification {
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
                        // Regular request - check if initialized first
                        let (initialized, session_id) = {
                            let s = state.lock().await;
                            (s.initialized, s.session_id.clone())
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

                        match forward_request(&client, &mcp_url, &session_id, &request).await {
                            Ok(response) => {
                                if !is_notification {
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
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        bail!("Daemon returned error {}: {}", status, body);
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
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        bail!("Daemon returned error {}: {}", status, body);
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
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        bail!("Daemon returned error {}: {}", status, body);
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
