//! # engram-mcp
//!
//! MCP (Model Context Protocol) server implementation for engram.
//!
//! This crate exposes engram's functionality as MCP tools that can be
//! used by AI coding agents like Claude Code and Cursor.
//!
//! ## Features
//!
//! - MCP server with stdio transport
//! - Document search and indexing tools
//! - JSON Schema generation for tool inputs
//! - Structured error responses
//!
//! ## Example
//!
//! ```ignore
//! use engram_mcp::server::EngramServer;
//! use engram_index::DocumentService;
//! use engram_store::{connect_and_init, StoreConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Initialize storage
//!     let db = connect_and_init(&StoreConfig::default()).await?;
//!
//!     // Create document service
//!     let service = DocumentService::with_defaults(db)?;
//!
//!     // Start MCP server
//!     let server = EngramServer::new();
//!     server.init(service).await;
//!     server.serve_stdio().await?;
//!
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod server;
pub mod tools;

pub use error::{McpError, McpResult};
pub use server::EngramServer;
pub use tools::{
    ActiveSessionInfoMcp,
    ConflictInfoMcp,
    CoordCheckConflictsRequest,
    CoordCheckConflictsResponse,
    CoordHeartbeatRequest,
    CoordListRequest,
    CoordListResponse,
    // Layer 5: Session Coordination
    CoordRegisterRequest,
    CoordRegisterResponse,
    CoordSetComponentsRequest,
    CoordSetComponentsResponse,
    CoordSetFileRequest,
    CoordSetFileResponse,
    CoordStatsRequest,
    CoordStatsResponse,
    CoordUnregisterRequest,
    DocTypeArg,
    DuplicateGroupInfo,
    EntityAliasRequest,
    // Layer 1: Entity Knowledge
    EntityCreateRequest,
    EntityCreateResponse,
    EntityGetRequest,
    EntityGetResponse,
    EntityInfo,
    EntityListRequest,
    EntityListResponse,
    EntityObserveRequest,
    EntityRelateRequest,
    EntityRelateResponse,
    EntitySearchRequest,
    EntitySearchResponse,
    EntityStatsRequest,
    EntityStatsResponse,
    EntityTypeArg,
    EventInfo,
    EventTypeArg,
    // Layer 3: Document Search
    GetStatsRequest,
    GetStatsResponse,
    IndexDocsRequest,
    IndexDocsResponse,
    KnowledgeDocInfo,
    KnowledgeDocResponse,
    KnowledgeDuplicatesRequest,
    KnowledgeDuplicatesResponse,
    KnowledgeImportRequest,
    // Layer 6: Knowledge Management
    KnowledgeInitRequest,
    KnowledgeInitResponse,
    KnowledgeListRequest,
    KnowledgeListResponse,
    KnowledgeRegisterRequest,
    KnowledgeScanRequest,
    KnowledgeScanResponse,
    KnowledgeStatsRequest,
    KnowledgeStatsResponse,
    KnowledgeVersionsRequest,
    KnowledgeVersionsResponse,
    ObservationInfo,
    RelationTypeArg,
    RelationshipInfo,
    SearchDocsRequest,
    SearchDocsResponse,
    // Unified Search
    SearchRequest,
    SearchResponse,
    SearchResult,
    SessionEndRequest,
    SessionGetRequest,
    SessionGetResponse,
    SessionInfo,
    SessionListRequest,
    SessionListResponse,
    SessionLogRequest,
    SessionLogResponse,
    SessionSearchRequest,
    SessionSearchResponse,
    // Layer 2: Session History
    SessionStartRequest,
    SessionStartResponse,
    SessionStatsRequest,
    SessionStatsResponse,
    SessionStatusArg,
    ToolGetStatsRequest,
    ToolGetStatsResponse,
    ToolIntelStatsRequest,
    ToolIntelStatsResponse,
    ToolListUsagesRequest,
    ToolListUsagesResponse,
    // Layer 4: Tool Intelligence
    ToolLogUsageRequest,
    ToolLogUsageResponse,
    ToolOutcomeArg,
    ToolRecommendRequest,
    ToolRecommendResponse,
    ToolRecommendationInfo,
    ToolSearchRequest,
    ToolUsageInfoMcp,
    UnifiedSearchResultInfo,
    VersionChainInfo,
    VersionInfo,
};
