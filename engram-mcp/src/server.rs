//! MCP server implementation for engram.
//!
//! Provides the MCP server that exposes engram's functionality to AI coding agents.

use crate::tools::{self, ToolState};
use engram_index::{
    CoordinationService, DocumentService, EntityService, GraphService, HandoffService,
    KnowledgeService, LintService, MemoryService, ObligationService, RepositoryService,
    SearchService, SessionService, TelemetryService, ToolIntelService, WorkService,
};
use rmcp::{
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
    transport::{StreamableHttpServerConfig, StreamableHttpService},
    ErrorData as McpError, ServerHandler, ServiceExt,
};
use sha2::{Digest, Sha256};
use std::{
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, OnceLock},
    time::Duration,
};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

/// Schema version for the daemon health response.
pub const DAEMON_HEALTH_SCHEMA_VERSION: u32 = 3;

/// Version of Engram's MCP-facing capability contract.
pub const MCP_CONTRACT_VERSION: u32 = 5;

/// MCP protocol version used by Engram's local proxy and daemon clients.
pub const MCP_PROTOCOL_VERSION: &str = "2024-11-05";
const STORAGE_READINESS_INTERVAL: Duration = Duration::from_secs(30);

// Re-export request types for external use (consolidated action-based API)
pub use crate::tools::{
    // Layer 5: Session coordination tools
    CoordRequestNew,
    CoordStatsRequest,
    // Layer 3: Document tools
    DigestRequest,
    DocsRequestNew,
    // Layer 1: Entity tools
    EntityObserveRequestNew,
    EntityRequestNew,
    EntityStatsRequest,
    // Memory OS tools
    GraphRequest,
    HandoffRequest,
    HarnessRequest,
    // Layer 6: Knowledge management tools
    KnowledgeRequestNew,
    KnowledgeStatsRequest,
    LintRequest,
    MemoryChangeRequest,
    MemoryEvidenceRequest,
    MemoryRequestNew,
    ObligationRequest,
    OrientRequest,
    RepoRequest,
    // Unified search
    SearchRequest,
    // Layer 2: Session tools
    SessionRequestNew,
    SessionStatsRequest,
    TelemetryRequest,
    // Layer 4: Tool intelligence tools
    ToolIntelStatsRequest,
    ToolRequestNew,
    VaultRequest,
    // Layer 7: Work management tools
    WorkContextRequestNew,
    WorkJoinRequest,
    WorkLeaveRequest,
    WorkObserveRequest,
    WorkPrRequest,
    WorkProjectRequest,
    WorkStatsRequest,
    WorkTaskRequest,
};

/// The engram MCP server.
#[derive(Clone)]
pub struct EngramServer {
    state: Arc<ToolState>,
    tool_router: ToolRouter<Self>,
    storage_path: Option<Arc<PathBuf>>,
}

impl EngramServer {
    /// Create a new engram server.
    pub fn new() -> Self {
        Self {
            state: Arc::new(ToolState::new()),
            tool_router: Self::tool_router(),
            storage_path: None,
        }
    }

    /// Attach the local persistent-store path used for disk-headroom readiness checks.
    #[must_use]
    pub fn with_storage_path(mut self, path: PathBuf) -> Self {
        self.storage_path = Some(Arc::new(path));
        self
    }

    /// Initialize the server with an entity service.
    pub async fn init_entity(&self, service: EntityService) {
        self.state.init_entity(service).await;
    }

    /// Initialize the server with a session service.
    pub async fn init_session(&self, service: SessionService) {
        self.state.init_session(service).await;
    }

    /// Initialize the server with a document service.
    pub async fn init(&self, service: DocumentService) {
        self.state.init(service).await;
    }

    /// Initialize the server with a tool intelligence service.
    pub async fn init_tool_intel(&self, service: ToolIntelService) {
        self.state.init_tool_intel(service).await;
    }

    /// Initialize the server with a coordination service.
    pub async fn init_coordination(&self, service: CoordinationService) {
        self.state.init_coordination(service).await;
    }

    /// Initialize the server with a knowledge service.
    pub async fn init_knowledge(&self, service: KnowledgeService) {
        self.state.init_knowledge(service).await;
    }

    /// Initialize the server with a work service.
    pub async fn init_work(&self, service: WorkService) {
        self.state.init_work(service).await;
    }

    /// Initialize the server with a Memory OS service.
    pub async fn init_memory(&self, service: MemoryService) {
        self.state.init_memory(service).await;
    }

    /// Initialize the server with a Memory OS lint service.
    pub async fn init_lint(&self, service: LintService) {
        self.state.init_lint(service).await;
    }

    /// Initialize the server with a Memory OS graph service.
    pub async fn init_graph(&self, service: GraphService) {
        self.state.init_graph(service).await;
    }

    /// Initialize the server with a rolling handoff service.
    pub async fn init_handoff(&self, service: HandoffService) {
        self.state.init_handoff(service).await;
    }

    /// Initialize the server with an agent obligation service.
    pub async fn init_obligation(&self, service: ObligationService) {
        self.state.init_obligation(service).await;
    }

    /// Initialize the server with a repository topology service.
    pub async fn init_repository(&self, service: RepositoryService) {
        self.state.init_repository(service).await;
    }

    /// Initialize the server with a search service.
    pub async fn init_search(&self, service: SearchService) {
        self.state.init_search(service).await;
    }

    /// Initialize the server with a brain harness telemetry service.
    pub async fn init_telemetry(&self, service: TelemetryService) {
        self.state.init_telemetry(service).await;
    }

    /// Start the server with stdio transport.
    ///
    /// # Errors
    ///
    /// Returns an error if the server fails to start.
    pub async fn serve_stdio(self) -> anyhow::Result<()> {
        info!("Starting engram MCP server on stdio");
        let service = self.serve(rmcp::transport::stdio()).await?;
        service.waiting().await?;
        Ok(())
    }

    /// Start the server with HTTP transport (daemon mode).
    ///
    /// This allows multiple clients to connect to a shared engram instance.
    ///
    /// # Arguments
    ///
    /// * `addr` - The socket address to listen on (e.g., "127.0.0.1:8765")
    ///
    /// # Errors
    ///
    /// Returns an error if the server fails to start or bind to the address.
    pub async fn serve_http(self, addr: SocketAddr) -> anyhow::Result<()> {
        use axum::{middleware, Router};
        use tower::ServiceBuilder;

        info!("Starting engram MCP HTTP server on {}", addr);

        let initial_readiness = storage_readiness(&self).await;
        if !initial_readiness.ready {
            anyhow::bail!(
                "Engram datastore is not ready: {}",
                initial_readiness.status
            );
        }

        // Create cancellation token for graceful shutdown
        let cancel_token = CancellationToken::new();
        let cancel_token_clone = cancel_token.clone();
        let readiness_cancel_token = cancel_token.clone();
        let readiness_server = self.clone();
        tokio::spawn(monitor_storage_readiness(
            readiness_server,
            readiness_cancel_token,
            STORAGE_READINESS_INTERVAL,
        ));

        // Create the HTTP service config
        let config = StreamableHttpServerConfig {
            cancellation_token: cancel_token.clone(),
            ..Default::default()
        };

        // Create the MCP HTTP service
        // The factory creates a new service for each session
        let server_clone = self.clone();
        let mcp_service = StreamableHttpService::new(
            move || {
                let server = server_clone.clone();
                Ok(server)
            },
            Arc::new(rmcp::transport::streamable_http_server::session::local::LocalSessionManager::default()),
            config,
        );

        // Protect daemon-managed MCP endpoints with a private bearer token. Direct explicit HTTP
        // mode remains available without a token, but reports that degraded mode via /health.
        let mut mcp_router =
            Router::new().nest_service("/mcp", ServiceBuilder::new().service(mcp_service));
        if let Some(token) = daemon_auth_token() {
            mcp_router = mcp_router.layer(middleware::from_fn(move |request, next| {
                let token = token.clone();
                async move { require_daemon_auth(request, next, &token).await }
            }));
        } else {
            warn!("Engram HTTP MCP endpoint is running without bearer authentication");
        }
        let health_server = self.clone();
        let app = Router::new().merge(mcp_router).route(
            "/health",
            axum::routing::get(move || {
                let server = health_server.clone();
                async move { health_handler(&server).await }
            }),
        );

        // Create the TCP listener
        let listener = tokio::net::TcpListener::bind(addr).await?;
        info!("Engram HTTP daemon listening on http://{}", addr);

        // Run the server with graceful shutdown
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                tokio::select! {
                    _ = cancel_token_clone.cancelled() => {
                        info!("Received cancellation signal");
                    }
                    _ = tokio::signal::ctrl_c() => {
                        info!("Received Ctrl+C, shutting down");
                    }
                }
            })
            .await?;

        Ok(())
    }
}

/// SHA-256 of the deterministic MCP tool contract exposed by this build.
#[must_use]
pub fn mcp_tools_sha256() -> &'static str {
    static DIGEST: OnceLock<String> = OnceLock::new();
    DIGEST.get_or_init(|| hash_mcp_tools(EngramServer::tool_router().list_all()))
}

/// Number of MCP tools exposed by the complete administrative profile.
#[must_use]
pub fn mcp_tool_count() -> usize {
    EngramServer::tool_router().list_all().len()
}

fn hash_mcp_tools(mut tools: Vec<rmcp::model::Tool>) -> String {
    tools.sort_by(|left, right| left.name.as_ref().cmp(right.name.as_ref()));
    let encoded = serde_json::to_vec(&tools).expect("MCP tool contract should serialize");
    format!("{:x}", Sha256::digest(encoded))
}

/// Health check handler for the HTTP server.
async fn health_handler(
    server: &EngramServer,
) -> (axum::http::StatusCode, axum::Json<serde_json::Value>) {
    let readiness = storage_readiness(server).await;
    if !readiness.ready {
        warn!(
            storage_status = readiness.status,
            "Engram datastore readiness probe failed"
        );
    }
    let (status_code, status) = if readiness.ready {
        (axum::http::StatusCode::OK, "ok")
    } else {
        (axum::http::StatusCode::SERVICE_UNAVAILABLE, "unavailable")
    };

    let body = axum::Json(serde_json::json!({
        "status": status,
        "service": "engram",
        "version": env!("CARGO_PKG_VERSION"),
        "build_sha": option_env!("ENGRAM_BUILD_SHA"),
        "pid": std::process::id(),
        "health_schema_version": DAEMON_HEALTH_SCHEMA_VERSION,
        "mcp_contract_version": MCP_CONTRACT_VERSION,
        "mcp_tools_sha256": mcp_tools_sha256(),
        "mcp_protocol_version": MCP_PROTOCOL_VERSION,
        "auth_required": daemon_auth_token().is_some(),
        "storage_status": readiness.status,
        "storage_ready": readiness.ready,
        "storage_available_bytes": readiness.available_bytes,
        "storage_required_bytes": readiness.required_bytes
    }));
    (status_code, body)
}

#[derive(Debug, Clone, Copy)]
struct StorageReadiness {
    ready: bool,
    status: &'static str,
    available_bytes: Option<u64>,
    required_bytes: Option<u64>,
}

async fn storage_readiness(server: &EngramServer) -> StorageReadiness {
    let mut available_bytes = None;
    let mut required_bytes = None;
    if let Some(path) = &server.storage_path {
        match engram_store::disk_headroom(path) {
            Ok(headroom) => {
                available_bytes = Some(headroom.available_bytes);
                required_bytes = Some(headroom.required_bytes);
                if let Some(failure) = low_disk_readiness(headroom) {
                    return failure;
                }
            }
            Err(error) => {
                warn!(error = %error, "Engram disk-headroom probe failed");
                return StorageReadiness {
                    ready: false,
                    status: "disk_probe_failed",
                    available_bytes,
                    required_bytes,
                };
            }
        }
    }

    let memory_service = server.state.memory_service.read().await.clone();
    let Some(service) = memory_service else {
        return StorageReadiness {
            ready: false,
            status: "uninitialized",
            available_bytes,
            required_bytes,
        };
    };
    if let Err(error) = service.probe_storage_writable().await {
        warn!(error = %error, "Engram datastore write probe failed");
        return StorageReadiness {
            ready: false,
            status: "write_probe_failed",
            available_bytes,
            required_bytes,
        };
    }

    StorageReadiness {
        ready: true,
        status: "ready",
        available_bytes,
        required_bytes,
    }
}

fn low_disk_readiness(headroom: engram_store::DiskHeadroom) -> Option<StorageReadiness> {
    (!headroom.is_sufficient()).then_some(StorageReadiness {
        ready: false,
        status: "low_disk",
        available_bytes: Some(headroom.available_bytes),
        required_bytes: Some(headroom.required_bytes),
    })
}

async fn monitor_storage_readiness(
    server: EngramServer,
    cancel_token: CancellationToken,
    interval_duration: Duration,
) {
    let mut interval = tokio::time::interval(interval_duration);
    interval.tick().await;
    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => break,
            _ = interval.tick() => {
                let readiness = storage_readiness(&server).await;
                if !readiness.ready {
                    error!(
                        storage_status = readiness.status,
                        "Engram datastore lost write readiness; shutting down daemon"
                    );
                    cancel_token.cancel();
                    break;
                }
            }
        }
    }
}

fn daemon_auth_token() -> Option<String> {
    std::env::var("ENGRAM_DAEMON_TOKEN")
        .ok()
        .map(|token| token.trim().to_string())
        .filter(|token| !token.is_empty())
}

async fn require_daemon_auth(
    request: axum::extract::Request,
    next: axum::middleware::Next,
    token: &str,
) -> axum::response::Response {
    use axum::response::IntoResponse;

    if !daemon_request_authorized(request.headers(), token) {
        return axum::http::StatusCode::UNAUTHORIZED.into_response();
    }
    next.run(request).await
}

fn daemon_request_authorized(headers: &axum::http::HeaderMap, token: &str) -> bool {
    let expected = format!("Bearer {token}");
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value == expected)
}

impl Default for EngramServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to convert Result<String, String> to Result<CallToolResult, McpError>
fn to_call_result(result: Result<String, String>) -> Result<CallToolResult, McpError> {
    match result {
        Ok(text) => Ok(CallToolResult::success(vec![Content::text(text)])),
        Err(e) => Ok(CallToolResult::error(vec![Content::text(e)])),
    }
}

// Implement the tool router for the server
#[tool_router]
impl EngramServer {
    // =========================================================================
    // Layer 3: Document Search Tools (Consolidated Action-based API)
    // =========================================================================

    /// Manage document indexing and search.
    #[tool(
        description = "Manage documents: search, index, plan, orphan_report, reindex_plan, reindex_execute, cleanup_plan, cleanup_execute, quarantine_review_export, quarantine_review_status, quarantine_review_prioritize, quarantine_review_apply, stats. Use 'action' parameter. search: semantic search (query, limit, min_score). index: add documents (path to file or directory). plan: dry-run ingestion policy and chunks (path). Administrative recovery, reindex, cleanup, quarantine, and stats actions require scope.relevance_mode=global because legacy document records lack project ownership. reindex_execute and cleanup_execute additionally retain their explicit write approvals."
    )]
    pub async fn docs(
        &self,
        params: Parameters<DocsRequestNew>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::docs_new(&self.state, params.0).await)
    }

    // =========================================================================
    // Layer 6: Knowledge Management Tools
    // =========================================================================

    /// Manage knowledge documents (consolidated action-based API).
    #[tool(
        description = "Manage knowledge documents: init, scan, register, import, list, duplicates, versions. Use 'action' parameter. init: create repo. scan: discover docs (needs path). register: reference doc (needs path, name, doc_type). import: copy to repo (needs path, name, doc_type). list, duplicates, and versions require scope.relevance_mode=global because legacy knowledge records lack project ownership. Doc types: adr, runbook, howto, research, design, readme, changelog."
    )]
    pub async fn knowledge(
        &self,
        params: Parameters<KnowledgeRequestNew>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::knowledge_new(&self.state, params.0).await)
    }

    /// Get knowledge statistics.
    #[tool(
        description = "Get statistics about the knowledge registry including document and sync counts."
    )]
    pub async fn knowledge_stats(
        &self,
        params: Parameters<KnowledgeStatsRequest>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::knowledge_stats(&self.state, params.0).await)
    }

    // =========================================================================
    // Layer 1: Entity Knowledge Tools (Consolidated Action-based API)
    // =========================================================================

    /// Manage entities in the knowledge graph.
    #[tool(
        description = "Manage entities: create, get, list, search, relate, alias, delete. Use 'action' parameter to specify operation. Entity types: repo, tool, concept, deployment, topic, workflow, person, team, service. Relation types: depends_on, uses, deployed_via, owned_by, documents, related_to."
    )]
    pub async fn entity(
        &self,
        params: Parameters<EntityRequestNew>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::entity_new(&self.state, params.0).await)
    }

    /// Manage entity observations.
    #[tool(
        description = "Manage entity observations: add, get, list, search, history. Use 'action' parameter. Observations store facts, notes, insights about entities. Use 'key' for semantic identification with format: category.subcategory (e.g., 'architecture.auth'). Categories: architecture, patterns, gotchas, decisions, dependencies, config, testing, performance, security. Keyed observations support upsert semantics."
    )]
    pub async fn entity_observe(
        &self,
        params: Parameters<EntityObserveRequestNew>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::entity_observe_new(&self.state, params.0).await)
    }

    /// Get entity statistics.
    #[tool(description = "Get statistics about the entity knowledge graph.")]
    pub async fn entity_stats(
        &self,
        params: Parameters<EntityStatsRequest>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::entity_stats(&self.state, params.0).await)
    }

    // =========================================================================
    // Layer 2: Session History Tools (Consolidated Action-based API)
    // =========================================================================

    /// Manage coding sessions.
    #[tool(
        description = "Manage sessions: start, end, get, list, log, search. Use 'action' parameter. start: begin session (agent, project, goal). end: finish session (session_id, summary). get: details (session_id). list: filter by status/agent/project. log: record event (session_id, event_type, content). search: find events (query). Event types: decision, observation, error, command, file_change, tool_use, milestone, prompt, plan, tool_result, test, preference, rule, limitation, handoff_update."
    )]
    pub async fn session(
        &self,
        params: Parameters<SessionRequestNew>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::session_new(&self.state, params.0).await)
    }

    /// Get session statistics.
    #[tool(description = "Get statistics about sessions and events.")]
    pub async fn session_stats(
        &self,
        params: Parameters<SessionStatsRequest>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::session_stats(&self.state, params.0).await)
    }

    // =========================================================================
    // Layer 4: Tool Intelligence Tools (Consolidated Action-based API)
    // =========================================================================

    /// Manage tool intelligence.
    #[tool(
        description = "Manage tool intelligence: log, recommend, stats, list, search. Use 'action' parameter. log: record usage (tool_name, context, outcome). recommend: get suggestions (context). stats: tool statistics (tool_name). list: recent usages (outcome_filter). search: find usages (query). Outcomes: success, partial, failed, switched."
    )]
    pub async fn tool(
        &self,
        params: Parameters<ToolRequestNew>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::tool_new(&self.state, params.0).await)
    }

    /// Get overall tool intelligence statistics.
    #[tool(
        description = "Get overall statistics about tool intelligence including usage count and learned preferences."
    )]
    pub async fn tool_intel_stats(
        &self,
        params: Parameters<ToolIntelStatsRequest>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::tool_intel_stats(&self.state, params.0).await)
    }

    // =========================================================================
    // Layer 5: Session Coordination Tools (Consolidated Action-based API)
    // =========================================================================

    /// Manage session coordination.
    #[tool(
        description = "Manage session coordination: register, unregister, heartbeat, set_file, set_components, check_conflicts, list. Use 'action' parameter. Conflict detection is isolated to the registered session's project. list defaults to local abstention; use scope.relevance_mode=related with a project/task/cwd boundary, or explicit global."
    )]
    pub async fn coord(
        &self,
        params: Parameters<CoordRequestNew>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::coord_new(&self.state, params.0).await)
    }

    /// Get coordination statistics.
    #[tool(description = "Get coordination statistics including number of active sessions.")]
    pub async fn coord_stats(
        &self,
        params: Parameters<CoordStatsRequest>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::coord_stats(&self.state, params.0).await)
    }

    // =========================================================================
    // Memory OS Tool
    // =========================================================================

    /// Manage Memory OS items and knowledge commits.
    #[tool(
        description = "Manage Memory OS records: add, propose_correction, get_correction_proposal, list_correction_proposals, verify_correction_procedure, apply_correction, procedure_match, capture_current_plan, get, list, review, promote, reject, supersede, correct, commit, cursor, changes_since, log, diff, writer_stats, archive, forget, export_vault, migration_inventory, migration_review_export, migration_review_status, migration_review_apply, digest_extraction_apply, distill_session. Agent proposals create inactive, digest-bound needs_review replacements, including unverified structured procedure replacements. Full-profile scoped verification attaches exact receipt proof while the replacement stays inactive and rotates its P0 digest to P1; apply_correction then atomically activates only a complete, unexpired, unchanged procedure proof. Operator selection is not authenticated human identity, intent, human review, or reviewer authority. Content retrieval accepts scope with local (default), related, or explicit global relevance; local returns only global/user and directly applicable project/task/cwd memory. Knowledge commit log/diff, writer aggregates, full-vault export, and editable review-batch status/apply require explicit global. procedure_match still requires verified, unexpired, exact-scope and exact-prerequisite applicability. Writes otherwise remain unchanged; forget is irreversible and requires an exact ID, reason, and confirmation."
    )]
    pub async fn memory(
        &self,
        params: Parameters<MemoryRequestNew>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::memory_new(&self.state, params.0).await)
    }

    /// Manage Memory OS agent harness policy and adapters.
    #[tool(
        description = "Manage the Memory OS agent harness contract: status, doctor, render_policy, render_adapter, install, hook_event. status and doctor accept scope with local (default), related project/task, or explicit global relevance. Local abstains before filesystem or host inspection. Related requires an explicit root that resolves through registered checkout topology to the same canonical project; the home-directory default and unrelated or ambiguous roots abstain. Global explicitly permits arbitrary/home roots. Supports claude_code, codex, gemini_cli, cursor, and generic. Installation is dry-run unless write=true; user-owned files require adopt_user_owned=true to replace. hook_event returns valid Claude hook JSON."
    )]
    pub async fn harness(
        &self,
        params: Parameters<HarnessRequest>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::harness_new(&self.state, params.0).await)
    }

    /// Run Memory OS lint checks and safe remediations.
    #[tool(
        description = "Run Memory OS health linting: run, list, apply_safe. Retrieval accepts scope with local (default), related project, or explicit global relevance. Local abstains; related includes global/user plus project-owned memory and project-scoped sessions/obligations, while exact-task scope abstains because every lint source cannot prove task ownership. Related apply_safe mutates only project-owned memory even when global/user findings are visible. vault_path requires explicit global because filesystem pages do not carry provable project ownership. Checks missing evidence, stale preferences, duplicate entity candidates, orphan project/task memory, stale active sessions, superseded active items, telemetry-flagged active memory, stale-feedback current-plan guidance, vault metadata, open obligations, and handoffs missing next actions. apply_safe writes only when write=true."
    )]
    pub async fn lint(&self, params: Parameters<LintRequest>) -> Result<CallToolResult, McpError> {
        to_call_result(tools::lint_new(&self.state, params.0).await)
    }

    /// Traverse the derived Memory OS graph.
    #[tool(
        description = "Traverse the derived Memory OS graph: around, path, subgraph, export. Retrieval accepts scope with local (default), related project/task, or explicit global relevance. Local abstains before graph access. Related filters memory and repository topology before traversal, excludes unprovable scopes and unrelated projects, and omits knowledge-commit nodes because commits lack project/task ownership. Global explicitly traverses all graph records."
    )]
    pub async fn graph(
        &self,
        params: Parameters<GraphRequest>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::graph_new(&self.state, params.0).await)
    }

    /// Manage rolling Memory OS handoffs.
    #[tool(
        description = "Manage rolling Memory OS handoffs: get, update, compile. get and compile accept scope with local (default), related project, or explicit global relevance. Local abstains; related verifies project-scoped handoffs directly and session-scoped handoffs against the authoritative Session project. Exact-task scope abstains because handoffs do not carry task ownership. update and compile default to dry-run unless dry_run=false and require writer_harness, model_provider, and model when writing/planning a handoff."
    )]
    pub async fn handoff(
        &self,
        params: Parameters<HandoffRequest>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::handoff_new(&self.state, params.0).await)
    }

    /// Manage agent-native session obligations.
    #[tool(
        description = "Manage agent-native obligations: detect, add, get, list, open, resolve, skip, doctor. get, list, open, and doctor accept scope with local (default), related project/task, or explicit global relevance. Local abstains before obligation-service access. Related includes global/user plus matching project guidance, narrows task obligations at an exact-task boundary, and verifies any cwd through registered checkout topology before Git inspection. Global explicitly permits cross-project retrieval and arbitrary cwd filters. detect is dry-run unless write=true; detect/add/resolve/skip behavior is unchanged."
    )]
    pub async fn obligations(
        &self,
        params: Parameters<ObligationRequest>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::obligations_new(&self.state, params.0).await)
    }

    /// Manage the generated Memory OS Markdown vault.
    #[tool(
        description = "Manage the generated Memory OS Markdown vault: init, compile, status, page. init only creates the directory skeleton. compile, status, and page require scope.relevance_mode=global because vault paths and the full-memory projection do not carry provable project/task ownership. Compile writes only Engram-generated files and skips existing user-owned files without the generated marker."
    )]
    pub async fn vault(
        &self,
        params: Parameters<VaultRequest>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::vault_new(&self.state, params.0).await)
    }

    /// Inventory digest-like sources, process review batches, or plan extraction.
    #[tool(
        description = "Inventory digest-like source files, export metadata-only review batches, parse review decisions, build review-gated extraction plans, or index reviewed source_only digests as document evidence. All actions require scope.relevance_mode=global because caller-selected filesystem sources and review batches do not carry provable project/task ownership. Extraction plans read only accepted sources and do not write active memory; source indexing defaults to dry-run unless write=true."
    )]
    pub async fn digest(
        &self,
        params: Parameters<DigestRequest>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::digest_new(&self.state, params.0).await)
    }

    /// Return a Memory OS orientation context packet for the current prompt.
    #[tool(
        description = "Return an orientation context packet for the current prompt. Includes a structured identity boundary that keeps repository/component identity separate from project authorization, plus a memory cursor, relevant memory, recommended actions, and ambiguities. Provide project only when authorized; cwd alone can resolve checkout identity while reporting project confirmation explicitly. Use response_shape='lean' for compact read-only/verification tasks that need identity, trace/cursor/scope, Brain Loop guidance, candidate memory IDs, and obligation summary/list."
    )]
    pub async fn orient(
        &self,
        params: Parameters<OrientRequest>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::orient(&self.state, params.0).await)
    }

    /// Manage brain-harness telemetry traces and agent feedback.
    #[tool(
        description = "Manage brain-harness telemetry and agent feedback: record_trace, get_trace, list_traces, submit_feedback, list_feedback, stats_by_intent, real_session_eval. Retrieval accepts scope with local (default), related project, or explicit global relevance. Local abstains because telemetry is not directly checkout-owned; related filters project-owned traces and derived feedback/reports, while exact-task scope abstains because traces do not carry task ownership. Traces may include free-form scenario_id/arm. Feedback should reference a trace_id returned by orient/search and may include outcome fields."
    )]
    pub async fn telemetry(
        &self,
        params: Parameters<TelemetryRequest>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::telemetry_new(&self.state, params.0).await)
    }

    /// Manage repository topology and local checkout mapping.
    #[tool(
        description = "Manage repository topology: detect, context, register, list, component_add, link_project, migration_inventory, migration_review_export, migration_review_status, migration_review_apply. Retrieval and migration-administration actions accept scope with local (default), related, or explicit global relevance. context is directly local to its cwd; list and migration inventory/export support project-related scope; review status/apply require explicit global because editable review batches do not prove project ownership. Use detect with cwd to register a Git checkout. migration_review_apply defaults to dry-run unless dry_run=false, and write mode requires writer_harness, model_provider, and model unless create_commit=false."
    )]
    pub async fn repo(&self, params: Parameters<RepoRequest>) -> Result<CallToolResult, McpError> {
        to_call_result(tools::repo_new(&self.state, params.0).await)
    }

    // =========================================================================
    // Unified Search Tool
    // =========================================================================

    /// Search across ALL knowledge layers with a single query.
    #[tool(
        description = "Search across ALL knowledge layers (memory items, entities, aliases, observations, session events, documents, tool usages) with a single query. Returns results sorted by relevance score. Use this for broad searches when you don't know which layer contains the information."
    )]
    pub async fn search(
        &self,
        params: Parameters<SearchRequest>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::search(&self.state, params.0).await)
    }

    // =========================================================================
    // Layer 7: Work Management Tools (Consolidated Action-Based API)
    // =========================================================================

    /// Manage projects with unified actions.
    #[tool(
        description = "Manage projects: create, get, list, update, delete, connect_entity, disconnect_entity, entities. Read actions default to local abstention; use scope.relevance_mode=related with a project/cwd boundary, or explicit global. Exact-task scope abstains from project-wide get/entities because those responses can contain sibling work. Write actions are unchanged."
    )]
    pub async fn work_project(
        &self,
        params: Parameters<WorkProjectRequest>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::work_project(&self.state, params.0).await)
    }

    /// Manage tasks with unified actions.
    #[tool(
        description = "Manage tasks: create, get, list, update, delete, connect_entity, disconnect_entity, entities. Read actions default to local abstention; related scope enforces the resolved project and, when supplied, exact task. Write actions are unchanged."
    )]
    pub async fn work_task(
        &self,
        params: Parameters<WorkTaskRequest>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::work_task(&self.state, params.0).await)
    }

    /// Manage pull requests with unified actions.
    #[tool(
        description = "Manage PRs: add, get, list, update, delete. Read actions default to local abstention; related scope enforces the resolved project and optional exact task, including URL lookup ownership checks. Write actions are unchanged."
    )]
    pub async fn work_pr(
        &self,
        params: Parameters<WorkPrRequest>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::work_pr(&self.state, params.0).await)
    }

    /// Manage work observations with unified actions.
    #[tool(
        description = "Manage project/task observations: add, get, list, delete. Read actions default to local abstention; related scope enforces the resolved project or exact task (task takes precedence). Add/delete writes are unchanged."
    )]
    pub async fn work_observe(
        &self,
        params: Parameters<WorkObserveRequest>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::work_observe(&self.state, params.0).await)
    }

    /// Join a work context.
    #[tool(
        description = "Join a project/task work context for a session. Returns full context including observations and entities."
    )]
    pub async fn work_join(
        &self,
        params: Parameters<WorkJoinRequest>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::work_join(&self.state, params.0).await)
    }

    /// Leave work context.
    #[tool(description = "Leave the current work context for a session.")]
    pub async fn work_leave(
        &self,
        params: Parameters<WorkLeaveRequest>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::work_leave(&self.state, params.0).await)
    }

    /// Get work context.
    #[tool(
        description = "Get work context. With session_id: returns the active project/task only when it matches the authorization boundary. With project and optional task: returns scoped full context. Defaults to local abstention; use related project/task/cwd scope or explicit global."
    )]
    pub async fn work_context(
        &self,
        params: Parameters<WorkContextRequestNew>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::work_context_new(&self.state, params.0).await)
    }

    /// Get work statistics.
    #[tool(
        description = "Get statistics about work management: project, task, PR, and observation counts."
    )]
    pub async fn work_stats(
        &self,
        params: Parameters<WorkStatsRequest>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::work_stats(&self.state, params.0).await)
    }
}

// Implement ServerHandler for the MCP protocol
#[tool_handler]
impl ServerHandler for EngramServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Engram is a local engineering-context service for AI coding agents. Resolve scope with orient, retrieve progressively with search, and treat ambiguous or mismatched scope as an abstention signal. Use memory procedure_match before replaying stored procedures. Store only durable facts with provenance and evidence; never store secrets. Administrative and migration operations are available in the full tool profile."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn health_response_attests_runtime_contract() {
        let storage_root = tempfile::tempdir().unwrap();
        let db = engram_store::connect(&engram_store::StoreConfig::memory())
            .await
            .unwrap();
        let server = EngramServer::new().with_storage_path(storage_root.path().to_path_buf());
        server.init_memory(MemoryService::new(db)).await;
        let (status, health) = health_handler(&server).await;
        let health = health.0;

        assert_eq!(status, axum::http::StatusCode::OK);
        assert_eq!(health["status"], "ok");
        assert_eq!(health["service"], "engram");
        assert_eq!(health["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(
            health["health_schema_version"],
            DAEMON_HEALTH_SCHEMA_VERSION
        );
        assert_eq!(health["mcp_contract_version"], MCP_CONTRACT_VERSION);
        assert_eq!(health["mcp_tools_sha256"], mcp_tools_sha256());
        assert_eq!(mcp_tools_sha256().len(), 64);
        assert_eq!(health["mcp_protocol_version"], MCP_PROTOCOL_VERSION);
        assert_eq!(health["pid"], std::process::id());
        assert!(health["auth_required"].is_boolean());
        assert_eq!(health["storage_status"], "ready");
        assert_eq!(health["storage_ready"], true);
        assert!(health["storage_available_bytes"].is_u64());
        assert!(health["storage_required_bytes"].is_u64());
    }

    #[tokio::test]
    async fn health_is_unavailable_when_the_datastore_cannot_commit() {
        let db = engram_store::connect(&engram_store::StoreConfig::memory())
            .await
            .unwrap();
        db.query(
            "DEFINE FIELD checked_at ON TABLE engram_storage_probe TYPE datetime ASSERT false",
        )
        .await
        .unwrap()
        .check()
        .unwrap();
        let server = EngramServer::new();
        server.init_memory(MemoryService::new(db)).await;

        let (status, health) = health_handler(&server).await;
        let health = health.0;

        assert_eq!(status, axum::http::StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(health["status"], "unavailable");
        assert_eq!(health["storage_status"], "write_probe_failed");
        assert_eq!(health["storage_ready"], false);
    }

    #[tokio::test]
    async fn http_daemon_refuses_to_bind_when_the_datastore_is_unwritable() {
        let db = engram_store::connect(&engram_store::StoreConfig::memory())
            .await
            .unwrap();
        db.query(
            "DEFINE FIELD checked_at ON TABLE engram_storage_probe TYPE datetime ASSERT false",
        )
        .await
        .unwrap()
        .check()
        .unwrap();
        let server = EngramServer::new();
        server.init_memory(MemoryService::new(db)).await;

        let error = server
            .serve_http("127.0.0.1:0".parse().unwrap())
            .await
            .expect_err("unwritable datastore must fail before binding HTTP");

        assert!(error.to_string().contains("write_probe_failed"));
    }

    #[test]
    fn insufficient_headroom_maps_to_low_disk_readiness() {
        let readiness = low_disk_readiness(engram_store::DiskHeadroom {
            available_bytes: 100,
            total_bytes: 1_000,
            required_bytes: 200,
        })
        .expect("insufficient headroom must fail readiness");

        assert!(!readiness.ready);
        assert_eq!(readiness.status, "low_disk");
        assert_eq!(readiness.available_bytes, Some(100));
        assert_eq!(readiness.required_bytes, Some(200));
    }

    #[tokio::test]
    async fn readiness_monitor_cancels_after_write_probe_failure() {
        let db = engram_store::connect(&engram_store::StoreConfig::memory())
            .await
            .unwrap();
        db.query(
            "DEFINE FIELD checked_at ON TABLE engram_storage_probe TYPE datetime ASSERT false",
        )
        .await
        .unwrap()
        .check()
        .unwrap();
        let server = EngramServer::new();
        server.init_memory(MemoryService::new(db)).await;
        let cancel_token = CancellationToken::new();
        let monitor = tokio::spawn(monitor_storage_readiness(
            server,
            cancel_token.clone(),
            Duration::from_millis(1),
        ));

        tokio::time::timeout(Duration::from_secs(1), cancel_token.cancelled())
            .await
            .expect("unwritable datastore must cancel the daemon token");
        monitor.await.unwrap();
    }

    #[test]
    fn mcp_tool_contract_hash_is_order_independent_and_content_sensitive() {
        let tools = EngramServer::tool_router().list_all();
        let expected = hash_mcp_tools(tools.clone());

        let mut reversed = tools.clone();
        reversed.reverse();
        assert_eq!(hash_mcp_tools(reversed), expected);

        let mut changed = tools;
        changed[0].description = Some("changed contract description".into());
        assert_ne!(hash_mcp_tools(changed), expected);
    }

    #[test]
    fn daemon_auth_requires_exact_bearer_token() {
        let mut headers = axum::http::HeaderMap::new();
        assert!(!daemon_request_authorized(&headers, "secret"));

        headers.insert(
            axum::http::header::AUTHORIZATION,
            axum::http::HeaderValue::from_static("Bearer wrong"),
        );
        assert!(!daemon_request_authorized(&headers, "secret"));

        headers.insert(
            axum::http::header::AUTHORIZATION,
            axum::http::HeaderValue::from_static("Bearer secret"),
        );
        assert!(daemon_request_authorized(&headers, "secret"));
    }
}
