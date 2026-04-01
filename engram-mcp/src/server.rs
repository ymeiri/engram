//! MCP server implementation for engram.
//!
//! Provides the MCP server that exposes engram's functionality to AI coding agents.

use crate::tools::{self, ToolState};
use engram_index::{
    CoordinationService, DocumentService, EntityService, KnowledgeService, SearchService,
    SessionService, ToolIntelService, WorkService,
};
use rmcp::{
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
    transport::{StreamableHttpServerConfig, StreamableHttpService},
    ErrorData as McpError, ServerHandler, ServiceExt,
};
use std::{net::SocketAddr, sync::Arc};
use tokio_util::sync::CancellationToken;
use tracing::info;

// Re-export request types for external use (consolidated action-based API)
pub use crate::tools::{
    // Layer 5: Session coordination tools
    CoordRequestNew,
    CoordStatsRequest,
    // Layer 3: Document tools
    DocsRequestNew,
    // Layer 1: Entity tools
    EntityObserveRequestNew,
    EntityRequestNew,
    EntityStatsRequest,
    // Layer 6: Knowledge management tools
    KnowledgeRequestNew,
    KnowledgeStatsRequest,
    // Unified search
    SearchRequest,
    // Layer 2: Session tools
    SessionRequestNew,
    SessionStatsRequest,
    // Layer 4: Tool intelligence tools
    ToolIntelStatsRequest,
    ToolRequestNew,
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
}

impl EngramServer {
    /// Create a new engram server.
    pub fn new() -> Self {
        Self {
            state: Arc::new(ToolState::new()),
            tool_router: Self::tool_router(),
        }
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

    /// Initialize the server with a search service.
    pub async fn init_search(&self, service: SearchService) {
        self.state.init_search(service).await;
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
        use axum::Router;
        use tower::ServiceBuilder;

        info!("Starting engram MCP HTTP server on {}", addr);

        // Create cancellation token for graceful shutdown
        let cancel_token = CancellationToken::new();
        let cancel_token_clone = cancel_token.clone();

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

        // Build the axum router
        let app = Router::new()
            // MCP endpoint at /mcp
            .nest_service("/mcp", ServiceBuilder::new().service(mcp_service))
            // Health check endpoint
            .route("/health", axum::routing::get(health_handler));

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

/// Health check handler for the HTTP server.
async fn health_handler() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "ok",
        "service": "engram",
        "version": env!("CARGO_PKG_VERSION")
    }))
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
        description = "Manage documents: search, index, stats. Use 'action' parameter. search: semantic search (query, limit, min_score). index: add documents (path to file or directory). stats: index statistics."
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
        description = "Manage knowledge documents: init, scan, register, import, list, duplicates, versions. Use 'action' parameter. init: create repo. scan: discover docs (needs path). register: reference doc (needs path, name, doc_type). import: copy to repo (needs path, name, doc_type). list: show all docs. duplicates: find dupes. versions: detect chains. Doc types: adr, runbook, howto, research, design, readme, changelog."
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
        description = "Manage sessions: start, end, get, list, log, search. Use 'action' parameter. start: begin session (agent, project, goal). end: finish session (session_id, summary). get: details (session_id). list: filter by status/agent/project. log: record event (session_id, event_type, content). search: find events (query). Event types: decision, observation, error, command, file_change, tool_use, milestone."
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
        description = "Manage session coordination: register, unregister, heartbeat, set_file, set_components, check_conflicts, list. Use 'action' parameter. Enables conflict detection when multiple agents work on the same project."
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
    // Unified Search Tool
    // =========================================================================

    /// Search across ALL knowledge layers with a single query.
    #[tool(
        description = "Search across ALL knowledge layers (entities, aliases, observations, session events, documents, tool usages) with a single query. Returns results sorted by relevance score. Use this for broad searches when you don't know which layer contains the information."
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
        description = "Manage projects: create, get, list, update, delete, connect_entity, disconnect_entity, entities. Use 'action' to specify the operation. Examples: {action: 'create', name: 'my-project'}, {action: 'list', status: 'active'}, {action: 'connect_entity', name: 'my-project', entity: 'api-service'}."
    )]
    pub async fn work_project(
        &self,
        params: Parameters<WorkProjectRequest>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::work_project(&self.state, params.0).await)
    }

    /// Manage tasks with unified actions.
    #[tool(
        description = "Manage tasks: create, get, list, update, delete, connect_entity, disconnect_entity, entities. Use 'action' to specify the operation. Examples: {action: 'create', project: 'my-project', name: 'my-task'}, {action: 'update', name: 'my-task', status: 'done'}."
    )]
    pub async fn work_task(
        &self,
        params: Parameters<WorkTaskRequest>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::work_task(&self.state, params.0).await)
    }

    /// Manage pull requests with unified actions.
    #[tool(
        description = "Manage PRs: add, get, list, update, delete. Use 'action' to specify the operation. Examples: {action: 'add', project: 'my-project', url: 'https://github.com/org/repo/pull/123'}, {action: 'update', url: '...', status: 'merged'}."
    )]
    pub async fn work_pr(
        &self,
        params: Parameters<WorkPrRequest>,
    ) -> Result<CallToolResult, McpError> {
        to_call_result(tools::work_pr(&self.state, params.0).await)
    }

    /// Manage work observations with unified actions.
    #[tool(
        description = "Manage project/task observations: add, get, list, delete. Scope via 'project' or 'task' (task takes precedence). Examples: {action: 'add', project: 'my-project', key: 'decisions.api', content: '...'}, {action: 'list', project: 'my-project', key_pattern: 'architecture.*'}."
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
        description = "Get work context. With session_id: returns which project/task is active. With project (and optional task): returns full context with all details."
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
                "Engram is a Personal Knowledge Augmentation System (PKAS) for AI coding agents.\n\n\
                 **Layer 1 - Entity Knowledge (Action-Based API):**\n\
                 - entity: Manage entities (actions: create, get, list, search, relate, alias, delete)\n\
                 - entity_observe: Manage observations (actions: add, get, list, search, history)\n\
                 - entity_stats: Get entity statistics\n\n\
                 **Layer 2 - Session History (Action-Based API):**\n\
                 - session: Manage sessions (actions: start, end, get, list, log, search)\n\
                 - session_stats: Get session statistics\n\n\
                 **Layer 3 - Document Search (Action-Based API):**\n\
                 - docs: Manage documents (actions: search, index, stats)\n\n\
                 **Layer 4 - Tool Intelligence (Action-Based API):**\n\
                 - tool: Manage tool usage (actions: log, recommend, stats, list, search)\n\
                 - tool_intel_stats: Get overall tool intelligence statistics\n\n\
                 **Layer 5 - Session Coordination (Action-Based API):**\n\
                 - coord: Manage coordination (actions: register, unregister, heartbeat, set_file, set_components, check_conflicts, list)\n\
                 - coord_stats: Get coordination statistics\n\n\
                 **Layer 6 - Knowledge Management (Action-Based API):**\n\
                 - knowledge: Manage knowledge docs (actions: init, scan, register, import, list, duplicates, versions)\n\
                 - knowledge_stats: Get knowledge statistics\n\n\
                 **Layer 7 - Work Management (Action-Based API):**\n\
                 - work_project: Manage projects (actions: create, get, list, update, delete, connect_entity, disconnect_entity, entities)\n\
                 - work_task: Manage tasks (actions: create, get, list, update, delete, connect_entity, disconnect_entity, entities)\n\
                 - work_pr: Manage pull requests (actions: add, get, list, update, delete)\n\
                 - work_observe: Manage observations (actions: add, get, list, delete) - scope via project or task\n\
                 - work_join: Join a work context for a session\n\
                 - work_leave: Leave work context\n\
                 - work_context: Get work context (session-based or direct lookup)\n\
                 - work_stats: Get work statistics\n\n\
                 **Unified Search:**\n\
                 - search: Search across ALL layers with a single query (entities, aliases, observations, sessions, documents, tool usages)"
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
