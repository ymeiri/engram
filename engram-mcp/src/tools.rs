//! MCP tool definitions for engram.
//!
//! Exposes engram's document search, indexing, and knowledge management capabilities as MCP tools.

use engram_core::document::DocSearchResult;
use engram_core::entity::{EntityType, RelationType};
use engram_core::knowledge::DocType;
use engram_core::search::SearchLayer;
use engram_core::session::{EventType, SessionStatus};
use engram_core::tool::ToolOutcome;
use engram_index::{
    CoordinationService, DocumentService, EntityService, KnowledgeService, SearchService,
    SessionService, ToolIntelService, WorkService,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

/// Shared state for MCP tools.
pub struct ToolState {
    /// The entity service for knowledge graph (Layer 1).
    pub entity_service: Arc<RwLock<Option<EntityService>>>,
    /// The session service for session history (Layer 2).
    pub session_service: Arc<RwLock<Option<SessionService>>>,
    /// The document service for indexing and search (Layer 3).
    pub doc_service: Arc<RwLock<Option<DocumentService>>>,
    /// The tool intelligence service (Layer 4).
    pub tool_intel_service: Arc<RwLock<Option<ToolIntelService>>>,
    /// The coordination service for session coordination (Layer 5).
    pub coordination_service: Arc<RwLock<Option<CoordinationService>>>,
    /// The knowledge service for document intelligence (Layer 6).
    pub knowledge_service: Arc<RwLock<Option<KnowledgeService>>>,
    /// The work service for project/task management (Layer 7).
    pub work_service: Arc<RwLock<Option<WorkService>>>,
    /// The unified search service (cross-layer search).
    pub search_service: Arc<RwLock<Option<SearchService>>>,
}

impl ToolState {
    /// Create new tool state (uninitialized).
    pub fn new() -> Self {
        Self {
            entity_service: Arc::new(RwLock::new(None)),
            session_service: Arc::new(RwLock::new(None)),
            doc_service: Arc::new(RwLock::new(None)),
            tool_intel_service: Arc::new(RwLock::new(None)),
            coordination_service: Arc::new(RwLock::new(None)),
            knowledge_service: Arc::new(RwLock::new(None)),
            work_service: Arc::new(RwLock::new(None)),
            search_service: Arc::new(RwLock::new(None)),
        }
    }

    /// Initialize with an entity service.
    pub async fn init_entity(&self, service: EntityService) {
        let mut guard = self.entity_service.write().await;
        *guard = Some(service);
    }

    /// Initialize with a session service.
    pub async fn init_session(&self, service: SessionService) {
        let mut guard = self.session_service.write().await;
        *guard = Some(service);
    }

    /// Initialize with a document service.
    pub async fn init(&self, service: DocumentService) {
        let mut guard = self.doc_service.write().await;
        *guard = Some(service);
    }

    /// Initialize with a tool intelligence service.
    pub async fn init_tool_intel(&self, service: ToolIntelService) {
        let mut guard = self.tool_intel_service.write().await;
        *guard = Some(service);
    }

    /// Initialize with a coordination service.
    pub async fn init_coordination(&self, service: CoordinationService) {
        let mut guard = self.coordination_service.write().await;
        *guard = Some(service);
    }

    /// Initialize with a knowledge service.
    pub async fn init_knowledge(&self, service: KnowledgeService) {
        let mut guard = self.knowledge_service.write().await;
        *guard = Some(service);
    }

    /// Initialize with a work service.
    pub async fn init_work(&self, service: WorkService) {
        let mut guard = self.work_service.write().await;
        *guard = Some(service);
    }

    /// Initialize with a search service.
    pub async fn init_search(&self, service: SearchService) {
        let mut guard = self.search_service.write().await;
        *guard = Some(service);
    }
}

impl Default for ToolState {
    fn default() -> Self {
        Self::new()
    }
}

/// Request to search documents by semantic similarity.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchDocsRequest {
    /// The query to search for.
    #[schemars(description = "Natural language query to search for in indexed documents")]
    pub query: String,

    /// Maximum number of results to return.
    #[schemars(description = "Maximum number of search results (default: 5)")]
    #[serde(default = "default_limit")]
    pub limit: usize,

    /// Minimum similarity score threshold (0.0 - 1.0).
    #[schemars(description = "Minimum similarity score threshold, 0.0-1.0 (default: 0.3)")]
    #[serde(default = "default_min_score")]
    pub min_score: Option<f32>,
}

fn default_limit() -> usize {
    5
}

fn default_min_score() -> Option<f32> {
    Some(0.3)
}

/// Result of a document search.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchDocsResponse {
    /// The search results.
    pub results: Vec<SearchResult>,
    /// Total number of results returned.
    pub count: usize,
}

/// A single search result.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchResult {
    /// The document title.
    pub title: Option<String>,
    /// The document path or URL.
    pub path: String,
    /// The heading path within the document.
    pub heading_path: String,
    /// The matching content.
    pub content: String,
    /// Similarity score (0.0 - 1.0).
    pub score: f32,
    /// Start line in the source document.
    pub start_line: Option<u32>,
    /// End line in the source document.
    pub end_line: Option<u32>,
}

impl From<DocSearchResult> for SearchResult {
    fn from(result: DocSearchResult) -> Self {
        Self {
            title: result.source.title,
            path: result.source.path_or_url,
            heading_path: result.chunk.heading_path,
            content: result.chunk.content,
            score: result.score,
            start_line: result.chunk.start_line,
            end_line: result.chunk.end_line,
        }
    }
}

/// Request to index a file or directory.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IndexDocsRequest {
    /// Path to the file or directory to index.
    #[schemars(description = "Path to the file or directory to index")]
    pub path: String,
}

/// Result of indexing documents.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IndexDocsResponse {
    /// Number of documents indexed.
    pub documents_indexed: usize,
    /// Number of chunks created.
    pub chunks_created: usize,
    /// Any warnings or errors.
    pub warnings: Vec<String>,
}

/// Request to get index statistics.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetStatsRequest {}

/// Index statistics response.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetStatsResponse {
    /// Number of indexed document sources.
    pub source_count: u64,
    /// Number of document chunks.
    pub chunk_count: u64,
    /// Embedding vector dimension.
    pub embedding_dimension: usize,
}

/// Execute a document search.
pub async fn search_docs(state: &ToolState, request: SearchDocsRequest) -> Result<String, String> {
    debug!("search_docs: query={}", request.query);

    let service_guard = state.doc_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Document service not initialized".to_string())?;

    let results = if let Some(min_score) = request.min_score {
        service
            .search_threshold(&request.query, request.limit, min_score)
            .await
            .map_err(|e| e.to_string())?
    } else {
        service
            .search(&request.query, request.limit)
            .await
            .map_err(|e| e.to_string())?
    };

    let response = SearchDocsResponse {
        count: results.len(),
        results: results.into_iter().map(SearchResult::from).collect(),
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Index documents from a path.
pub async fn index_docs(state: &ToolState, request: IndexDocsRequest) -> Result<String, String> {
    debug!("index_docs: path={}", request.path);

    let service_guard = state.doc_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Document service not initialized".to_string())?;

    let path = std::path::Path::new(&request.path);
    let mut warnings = Vec::new();

    let (docs_indexed, chunks_created) = if path.is_dir() {
        let results = service
            .index_directory(path)
            .await
            .map_err(|e| e.to_string())?;

        let docs = results.len();
        let chunks: usize = results.iter().map(|d| d.chunks.len()).sum();
        (docs, chunks)
    } else if path.is_file() {
        let result = service.index_file(path).await.map_err(|e| e.to_string())?;

        (1, result.chunks.len())
    } else {
        warnings.push(format!("Path not found: {}", request.path));
        (0, 0)
    };

    let response = IndexDocsResponse {
        documents_indexed: docs_indexed,
        chunks_created,
        warnings,
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Get index statistics.
pub async fn get_stats(state: &ToolState, _request: GetStatsRequest) -> Result<String, String> {
    debug!("get_stats");

    let service_guard = state.doc_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Document service not initialized".to_string())?;

    let stats = service.stats().await.map_err(|e| e.to_string())?;

    let response = GetStatsResponse {
        source_count: stats.source_count,
        chunk_count: stats.chunk_count,
        embedding_dimension: stats.embedding_dimension,
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

// =============================================================================
// Layer 6: Knowledge Management Tools
// =============================================================================

/// Request to initialize the knowledge system.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeInitRequest {}

/// Response after initializing the knowledge system.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeInitResponse {
    /// Path to the personal knowledge repository.
    pub repo_path: String,
    /// Whether initialization was successful.
    pub success: bool,
}

/// Request to scan a directory for documents.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeScanRequest {
    /// Path to the directory to scan.
    #[schemars(description = "Path to the directory to scan for markdown files")]
    pub path: String,

    /// Repository name (identifier for tracking).
    #[schemars(description = "Repository name for tracking (default: 'default')")]
    #[serde(default = "default_repo_name")]
    pub repo_name: String,
}

fn default_repo_name() -> String {
    "default".to_string()
}

/// Response after scanning a directory.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeScanResponse {
    /// Total files found.
    pub files_found: usize,
    /// New files added.
    pub files_new: usize,
    /// Files with updated content.
    pub files_updated: usize,
}

/// Document type for registration/import.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DocTypeArg {
    Adr,
    Runbook,
    Howto,
    Research,
    Design,
    Readme,
    Changelog,
}

impl From<DocTypeArg> for DocType {
    fn from(arg: DocTypeArg) -> Self {
        match arg {
            DocTypeArg::Adr => DocType::Adr,
            DocTypeArg::Runbook => DocType::Runbook,
            DocTypeArg::Howto => DocType::Howto,
            DocTypeArg::Research => DocType::Research,
            DocTypeArg::Design => DocType::Design,
            DocTypeArg::Readme => DocType::Readme,
            DocTypeArg::Changelog => DocType::Changelog,
        }
    }
}

/// Request to register a document (reference only, doesn't copy).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeRegisterRequest {
    /// Path to the document to register.
    #[schemars(description = "Path to the document to register")]
    pub path: String,

    /// Human-friendly name for the document.
    #[schemars(description = "Name for the document")]
    pub name: String,

    /// Document type.
    #[schemars(
        description = "Type of document (adr, runbook, howto, research, design, readme, changelog)"
    )]
    pub doc_type: DocTypeArg,
}

/// Request to import a document (copies to personal repo).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeImportRequest {
    /// Path to the source document.
    #[schemars(description = "Path to the source document to import")]
    pub source_path: String,

    /// Human-friendly name for the document.
    #[schemars(description = "Name for the document")]
    pub name: String,

    /// Document type.
    #[schemars(
        description = "Type of document (adr, runbook, howto, research, design, readme, changelog)"
    )]
    pub doc_type: DocTypeArg,
}

/// Response after registering or importing a document.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeDocResponse {
    /// Document ID.
    pub id: String,
    /// Document name.
    pub name: String,
    /// Document type.
    pub doc_type: String,
    /// Path to the document.
    pub path: Option<String>,
}

/// Request to list knowledge documents.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeListRequest {}

/// A knowledge document in the list response.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeDocInfo {
    /// Document ID.
    pub id: String,
    /// Document name.
    pub name: String,
    /// Document type.
    pub doc_type: String,
    /// Document status.
    pub status: String,
    /// Path to the document.
    pub path: Option<String>,
}

/// Response listing knowledge documents.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeListResponse {
    /// List of documents.
    pub documents: Vec<KnowledgeDocInfo>,
    /// Total count.
    pub count: usize,
}

/// Request to find duplicate documents.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeDuplicatesRequest {}

/// A group of duplicate files.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuplicateGroupInfo {
    /// Content hash.
    pub hash: String,
    /// Paths of duplicate files.
    pub paths: Vec<String>,
}

/// Response with duplicate groups.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeDuplicatesResponse {
    /// Duplicate groups.
    pub groups: Vec<DuplicateGroupInfo>,
    /// Number of groups.
    pub count: usize,
}

/// Request to detect version chains.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeVersionsRequest {}

/// A version of a document.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VersionInfo {
    /// File path.
    pub path: String,
    /// Version number (if detected).
    pub version: Option<u32>,
}

/// A chain of document versions.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VersionChainInfo {
    /// Base name shared by versions.
    pub base_name: String,
    /// Versions in the chain.
    pub versions: Vec<VersionInfo>,
    /// Recommended canonical path.
    pub recommended_canonical: Option<String>,
}

/// Response with version chains.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeVersionsResponse {
    /// Version chains.
    pub chains: Vec<VersionChainInfo>,
    /// Number of chains.
    pub count: usize,
}

/// Request to get knowledge statistics.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeStatsRequest {}

/// Knowledge statistics response.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeStatsResponse {
    /// Number of knowledge documents.
    pub doc_count: u64,
    /// Number of synced files.
    pub file_sync_count: u64,
    /// Number of aliases.
    pub alias_count: u64,
}

// =============================================================================
// Layer 6: Knowledge Tool Implementations
// =============================================================================

/// Initialize the knowledge system.
pub async fn knowledge_init(
    state: &ToolState,
    _request: KnowledgeInitRequest,
) -> Result<String, String> {
    debug!("knowledge_init");

    let service_guard = state.knowledge_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Knowledge service not initialized".to_string())?;

    service.init().await.map_err(|e| e.to_string())?;

    let response = KnowledgeInitResponse {
        repo_path: service.knowledge_repo_path().display().to_string(),
        success: true,
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Scan a directory for documents.
pub async fn knowledge_scan(
    state: &ToolState,
    request: KnowledgeScanRequest,
) -> Result<String, String> {
    debug!("knowledge_scan: path={}", request.path);

    let service_guard = state.knowledge_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Knowledge service not initialized".to_string())?;

    // Ensure initialized
    service.init().await.map_err(|e| e.to_string())?;

    let path = Path::new(&request.path);
    let result = service
        .scan_directory(path, &request.repo_name)
        .await
        .map_err(|e| e.to_string())?;

    let response = KnowledgeScanResponse {
        files_found: result.files_found,
        files_new: result.files_new,
        files_updated: result.files_updated,
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Register a document (reference only).
pub async fn knowledge_register(
    state: &ToolState,
    request: KnowledgeRegisterRequest,
) -> Result<String, String> {
    debug!(
        "knowledge_register: path={}, name={}",
        request.path, request.name
    );

    let service_guard = state.knowledge_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Knowledge service not initialized".to_string())?;

    // Ensure initialized
    service.init().await.map_err(|e| e.to_string())?;

    let path = Path::new(&request.path);
    let doc = service
        .register_doc(path, &request.name, request.doc_type.into())
        .await
        .map_err(|e| e.to_string())?;

    let response = KnowledgeDocResponse {
        id: doc.id.to_string(),
        name: doc.name,
        doc_type: doc.doc_type.to_string(),
        path: doc.canonical_path,
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Import a document to the personal knowledge repo.
pub async fn knowledge_import(
    state: &ToolState,
    request: KnowledgeImportRequest,
) -> Result<String, String> {
    debug!(
        "knowledge_import: source={}, name={}",
        request.source_path, request.name
    );

    let service_guard = state.knowledge_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Knowledge service not initialized".to_string())?;

    // Ensure initialized
    service.init().await.map_err(|e| e.to_string())?;

    let source_path = Path::new(&request.source_path);
    let doc = service
        .import_doc(source_path, &request.name, request.doc_type.into())
        .await
        .map_err(|e| e.to_string())?;

    let response = KnowledgeDocResponse {
        id: doc.id.to_string(),
        name: doc.name,
        doc_type: doc.doc_type.to_string(),
        path: doc.canonical_path,
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// List all knowledge documents.
pub async fn knowledge_list(
    state: &ToolState,
    _request: KnowledgeListRequest,
) -> Result<String, String> {
    debug!("knowledge_list");

    let service_guard = state.knowledge_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Knowledge service not initialized".to_string())?;

    // Ensure initialized
    service.init().await.map_err(|e| e.to_string())?;

    let docs = service.list_docs().await.map_err(|e| e.to_string())?;

    let documents: Vec<KnowledgeDocInfo> = docs
        .into_iter()
        .map(|doc| KnowledgeDocInfo {
            id: doc.id.to_string(),
            name: doc.name,
            doc_type: doc.doc_type.to_string(),
            status: format!("{:?}", doc.status),
            path: doc.canonical_path,
        })
        .collect();

    let response = KnowledgeListResponse {
        count: documents.len(),
        documents,
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Find duplicate documents.
pub async fn knowledge_find_duplicates(
    state: &ToolState,
    _request: KnowledgeDuplicatesRequest,
) -> Result<String, String> {
    debug!("knowledge_find_duplicates");

    let service_guard = state.knowledge_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Knowledge service not initialized".to_string())?;

    // Ensure initialized
    service.init().await.map_err(|e| e.to_string())?;

    let duplicates = service.find_duplicates().await.map_err(|e| e.to_string())?;

    let groups: Vec<DuplicateGroupInfo> = duplicates
        .into_iter()
        .map(|g| DuplicateGroupInfo {
            hash: g.content_hash,
            paths: g.files.into_iter().map(|f| f.path).collect(),
        })
        .collect();

    let response = KnowledgeDuplicatesResponse {
        count: groups.len(),
        groups,
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Detect version chains.
pub async fn knowledge_detect_versions(
    state: &ToolState,
    _request: KnowledgeVersionsRequest,
) -> Result<String, String> {
    debug!("knowledge_detect_versions");

    let service_guard = state.knowledge_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Knowledge service not initialized".to_string())?;

    // Ensure initialized
    service.init().await.map_err(|e| e.to_string())?;

    let chains = service.detect_versions().await.map_err(|e| e.to_string())?;

    let mut chain_infos = Vec::new();
    for chain in chains {
        let recommended = service.resolve_canonical(&chain).await.ok().flatten();

        chain_infos.push(VersionChainInfo {
            base_name: chain.base_name,
            versions: chain
                .versions
                .into_iter()
                .map(|v| VersionInfo {
                    path: v.path,
                    version: v.version,
                })
                .collect(),
            recommended_canonical: recommended,
        });
    }

    let response = KnowledgeVersionsResponse {
        count: chain_infos.len(),
        chains: chain_infos,
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Get knowledge statistics.
pub async fn knowledge_stats(
    state: &ToolState,
    _request: KnowledgeStatsRequest,
) -> Result<String, String> {
    debug!("knowledge_stats");

    let service_guard = state.knowledge_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Knowledge service not initialized".to_string())?;

    // Ensure initialized
    service.init().await.map_err(|e| e.to_string())?;

    let stats = service.stats().await.map_err(|e| e.to_string())?;

    let response = KnowledgeStatsResponse {
        doc_count: stats.doc_count,
        file_sync_count: stats.file_sync_count,
        alias_count: stats.alias_count,
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

// =============================================================================
// Layer 1: Entity Knowledge Types
// =============================================================================

/// Entity type for MCP tools.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EntityTypeArg {
    Repo,
    Tool,
    Concept,
    Deployment,
    Topic,
    Workflow,
    Person,
    Team,
    Service,
}

impl From<EntityTypeArg> for EntityType {
    fn from(arg: EntityTypeArg) -> Self {
        match arg {
            EntityTypeArg::Repo => EntityType::Repo,
            EntityTypeArg::Tool => EntityType::Tool,
            EntityTypeArg::Concept => EntityType::Concept,
            EntityTypeArg::Deployment => EntityType::Deployment,
            EntityTypeArg::Topic => EntityType::Topic,
            EntityTypeArg::Workflow => EntityType::Workflow,
            EntityTypeArg::Person => EntityType::Person,
            EntityTypeArg::Team => EntityType::Team,
            EntityTypeArg::Service => EntityType::Service,
        }
    }
}

/// Relation type for MCP tools.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RelationTypeArg {
    DependsOn,
    Uses,
    DeployedVia,
    OwnedBy,
    Documents,
    RelatedTo,
}

impl From<RelationTypeArg> for RelationType {
    fn from(arg: RelationTypeArg) -> Self {
        match arg {
            RelationTypeArg::DependsOn => RelationType::DependsOn,
            RelationTypeArg::Uses => RelationType::Uses,
            RelationTypeArg::DeployedVia => RelationType::DeployedVia,
            RelationTypeArg::OwnedBy => RelationType::OwnedBy,
            RelationTypeArg::Documents => RelationType::Documents,
            RelationTypeArg::RelatedTo => RelationType::RelatedTo,
        }
    }
}

/// Request to create an entity.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityCreateRequest {
    /// Entity name.
    #[schemars(description = "Name of the entity (e.g., 'web-ui', 'postgres-db', 'MCP')")]
    pub name: String,

    /// Entity type.
    #[schemars(description = "Type of entity")]
    pub entity_type: EntityTypeArg,

    /// Optional description.
    #[schemars(description = "Description of the entity")]
    pub description: Option<String>,
}

/// Response after creating an entity.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityCreateResponse {
    /// Entity ID.
    pub id: String,
    /// Entity name.
    pub name: String,
    /// Entity type.
    pub entity_type: String,
    /// Description.
    pub description: Option<String>,
}

/// Request to list entities.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityListRequest {
    /// Optional entity type filter.
    #[schemars(description = "Filter by entity type (optional)")]
    pub entity_type: Option<EntityTypeArg>,
}

/// Response with list of entities.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityListResponse {
    /// The entities.
    pub entities: Vec<EntityInfo>,
    /// Total count.
    pub count: usize,
}

/// Entity info for list responses.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityInfo {
    /// Entity ID.
    pub id: String,
    /// Entity name.
    pub name: String,
    /// Entity type.
    pub entity_type: String,
    /// Description.
    pub description: Option<String>,
}

/// Request to search entities.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntitySearchRequest {
    /// Search query.
    #[schemars(description = "Search query to find entities by name")]
    pub query: String,
}

/// Response with search results.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntitySearchResponse {
    /// Matching entities.
    pub entities: Vec<EntityInfo>,
    /// Total count.
    pub count: usize,
}

/// Request to get entity details.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityGetRequest {
    /// Entity name or alias.
    #[schemars(description = "Entity name or alias to look up")]
    pub name: String,
}

/// Full entity details response.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityGetResponse {
    /// Entity info.
    pub entity: EntityInfo,
    /// Aliases for this entity.
    pub aliases: Vec<String>,
    /// Outgoing relationships.
    pub relationships_out: Vec<RelationshipInfo>,
    /// Incoming relationships.
    pub relationships_in: Vec<RelationshipInfo>,
    /// Observations about this entity.
    pub observations: Vec<ObservationInfo>,
}

/// Relationship info.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RelationshipInfo {
    /// Relationship type.
    pub relation_type: String,
    /// Related entity name.
    pub entity_name: String,
    /// Related entity type.
    pub entity_type: String,
}

/// Observation info for responses.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ObservationInfo {
    /// Observation ID.
    pub id: String,
    /// Semantic key (optional).
    pub key: Option<String>,
    /// Observation content.
    pub content: String,
    /// Source of the observation.
    pub source: Option<String>,
    /// Created timestamp.
    pub created_at: String,
    /// Last updated timestamp.
    pub updated_at: String,
}

/// Archived observation info for history responses.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ArchivedObservationInfo {
    /// Content at the time of archiving.
    pub content: String,
    /// Source.
    pub source: Option<String>,
    /// When this version was created.
    pub created_at: String,
    /// When this version was archived (replaced).
    pub archived_at: String,
}

/// Request to create a relationship.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityRelateRequest {
    /// Source entity name.
    #[schemars(description = "Source entity name or alias")]
    pub source: String,

    /// Relationship type.
    #[schemars(description = "Type of relationship")]
    pub relation_type: RelationTypeArg,

    /// Target entity name.
    #[schemars(description = "Target entity name or alias")]
    pub target: String,
}

/// Response after creating a relationship.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityRelateResponse {
    /// Success message.
    pub message: String,
    /// Source entity name.
    pub source: String,
    /// Relationship type.
    pub relation_type: String,
    /// Target entity name.
    pub target: String,
}

/// Request to add an alias.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityAliasRequest {
    /// Entity name.
    #[schemars(description = "Entity name to add alias for")]
    pub entity: String,

    /// Alias to add.
    #[schemars(description = "Alias text to add")]
    pub alias: String,
}

/// Request to add or update an observation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityObserveRequest {
    /// Entity name.
    #[schemars(description = "Entity name to add observation for")]
    pub entity: String,

    /// Semantic key for the observation (optional).
    /// Format: category.subcategory (e.g., "architecture.auth", "gotchas.race-conditions")
    /// Categories: architecture, patterns, gotchas, decisions, dependencies, config, testing, performance, security
    /// When provided, if an observation with this key already exists for the entity, it will be updated.
    #[schemars(
        description = "Semantic key for updates (e.g., 'architecture.auth'). If key exists, updates existing observation."
    )]
    pub key: Option<String>,

    /// Observation content.
    #[schemars(description = "The observation/fact/note")]
    pub content: String,

    /// Source of the observation.
    #[schemars(
        description = "Source of this observation (optional, e.g., 'code-analysis', 'bug-fix')"
    )]
    pub source: Option<String>,
}

/// Request to get an observation by key.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityObserveGetRequest {
    /// Entity name.
    #[schemars(description = "Entity name")]
    pub entity: String,

    /// Semantic key.
    #[schemars(description = "Semantic key of the observation")]
    pub key: String,
}

/// Request to list observations with optional filtering.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityObserveListRequest {
    /// Entity name.
    #[schemars(description = "Entity name")]
    pub entity: String,

    /// Key pattern filter (optional).
    /// Uses glob-style: * for wildcard (e.g., "architecture.*")
    #[schemars(description = "Key pattern filter (e.g., 'architecture.*'). Omit to list all.")]
    pub key_pattern: Option<String>,

    /// Maximum results.
    #[schemars(description = "Maximum number of results (default: 50)")]
    pub limit: Option<usize>,
}

/// Request to search observations by content.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityObserveSearchRequest {
    /// Entity name.
    #[schemars(description = "Entity name")]
    pub entity: String,

    /// Search query.
    #[schemars(description = "Search query (case-insensitive content match)")]
    pub query: String,

    /// Maximum results.
    #[schemars(description = "Maximum number of results (default: 20)")]
    pub limit: Option<usize>,
}

/// Request to get observation history.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityObserveHistoryRequest {
    /// Entity name.
    #[schemars(description = "Entity name")]
    pub entity: String,

    /// Semantic key.
    #[schemars(description = "Semantic key of the observation")]
    pub key: String,
}

/// Request for entity statistics.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityStatsRequest {}

/// Entity statistics response.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityStatsResponse {
    /// Number of entities.
    pub entity_count: u64,
    /// Number of relationships.
    pub relationship_count: u64,
    /// Number of aliases.
    pub alias_count: u64,
    /// Number of observations.
    pub observation_count: u64,
}

// =============================================================================
// Layer 1: Entity Tool Implementations
// =============================================================================

/// Create an entity.
pub async fn entity_create(
    state: &ToolState,
    request: EntityCreateRequest,
) -> Result<String, String> {
    debug!(
        "entity_create: name={}, type={:?}",
        request.name, request.entity_type
    );

    let service_guard = state.entity_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Entity service not initialized".to_string())?;

    let entity = service
        .create_entity(
            &request.name,
            request.entity_type.into(),
            request.description.as_deref(),
        )
        .await
        .map_err(|e| e.to_string())?;

    let response = EntityCreateResponse {
        id: entity.id.to_string(),
        name: entity.name,
        entity_type: entity.entity_type.to_string(),
        description: entity.description,
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// List entities.
pub async fn entity_list(state: &ToolState, request: EntityListRequest) -> Result<String, String> {
    debug!("entity_list: type_filter={:?}", request.entity_type);

    let service_guard = state.entity_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Entity service not initialized".to_string())?;

    let type_filter = request.entity_type.map(EntityType::from);
    let entities = service
        .list_entities(type_filter.as_ref())
        .await
        .map_err(|e| e.to_string())?;

    let entity_infos: Vec<EntityInfo> = entities
        .into_iter()
        .map(|e| EntityInfo {
            id: e.id.to_string(),
            name: e.name,
            entity_type: e.entity_type.to_string(),
            description: e.description,
        })
        .collect();

    let count = entity_infos.len();
    let response = EntityListResponse {
        entities: entity_infos,
        count,
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Search entities by name.
pub async fn entity_search(
    state: &ToolState,
    request: EntitySearchRequest,
) -> Result<String, String> {
    debug!("entity_search: query={}", request.query);

    let service_guard = state.entity_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Entity service not initialized".to_string())?;

    let entities = service
        .search_entities(&request.query)
        .await
        .map_err(|e| e.to_string())?;

    let entity_infos: Vec<EntityInfo> = entities
        .into_iter()
        .map(|e| EntityInfo {
            id: e.id.to_string(),
            name: e.name,
            entity_type: e.entity_type.to_string(),
            description: e.description,
        })
        .collect();

    let count = entity_infos.len();
    let response = EntitySearchResponse {
        entities: entity_infos,
        count,
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Get entity details.
pub async fn entity_get(state: &ToolState, request: EntityGetRequest) -> Result<String, String> {
    debug!("entity_get: name={}", request.name);

    let service_guard = state.entity_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Entity service not initialized".to_string())?;

    let entity = service
        .resolve(&request.name)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Entity not found: {}", request.name))?;

    // Get aliases
    let aliases = service
        .get_aliases(&entity.name)
        .await
        .map_err(|e| e.to_string())?;

    // Get outgoing relationships
    let related_from = service
        .get_related_from(&entity.id)
        .await
        .map_err(|e| e.to_string())?;

    let relationships_out: Vec<RelationshipInfo> = related_from
        .into_iter()
        .map(|(rel, target)| RelationshipInfo {
            relation_type: rel.relation_type.to_string(),
            entity_name: target.name,
            entity_type: target.entity_type.to_string(),
        })
        .collect();

    // Get incoming relationships
    let related_to = service
        .get_related_to(&entity.id)
        .await
        .map_err(|e| e.to_string())?;

    let relationships_in: Vec<RelationshipInfo> = related_to
        .into_iter()
        .map(|(rel, source)| RelationshipInfo {
            relation_type: rel.relation_type.to_string(),
            entity_name: source.name,
            entity_type: source.entity_type.to_string(),
        })
        .collect();

    // Get observations
    let observations = service
        .get_observations(&entity.name)
        .await
        .map_err(|e| e.to_string())?;

    let observation_infos: Vec<ObservationInfo> = observations
        .into_iter()
        .map(|o| ObservationInfo {
            id: o.id.to_string(),
            key: o.key,
            content: o.content,
            source: o.source,
            created_at: o
                .created_at
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap(),
            updated_at: o
                .updated_at
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap(),
        })
        .collect();

    let response = EntityGetResponse {
        entity: EntityInfo {
            id: entity.id.to_string(),
            name: entity.name,
            entity_type: entity.entity_type.to_string(),
            description: entity.description,
        },
        aliases,
        relationships_out,
        relationships_in,
        observations: observation_infos,
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Create a relationship between entities.
pub async fn entity_relate(
    state: &ToolState,
    request: EntityRelateRequest,
) -> Result<String, String> {
    debug!(
        "entity_relate: {} --[{:?}]--> {}",
        request.source, request.relation_type, request.target
    );

    let service_guard = state.entity_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Entity service not initialized".to_string())?;

    let rel = service
        .relate(
            &request.source,
            request.relation_type.into(),
            &request.target,
        )
        .await
        .map_err(|e| e.to_string())?;

    let response = EntityRelateResponse {
        message: "Relationship created".to_string(),
        source: request.source,
        relation_type: rel.relation_type.to_string(),
        target: request.target,
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Add an alias for an entity.
pub async fn entity_alias(
    state: &ToolState,
    request: EntityAliasRequest,
) -> Result<String, String> {
    debug!(
        "entity_alias: entity={}, alias={}",
        request.entity, request.alias
    );

    let service_guard = state.entity_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Entity service not initialized".to_string())?;

    service
        .add_alias(&request.entity, &request.alias)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_string_pretty(&serde_json::json!({
        "message": format!("Alias '{}' added for entity '{}'", request.alias, request.entity)
    }))
    .map_err(|e| e.to_string())
}

/// Add or update an observation about an entity.
/// If key is provided and exists, updates the existing observation (archives old version).
pub async fn entity_observe(
    state: &ToolState,
    request: EntityObserveRequest,
) -> Result<String, String> {
    debug!(
        "entity_observe: entity={}, key={:?}",
        request.entity, request.key
    );

    let service_guard = state.entity_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Entity service not initialized".to_string())?;

    let (obs, previous) = service
        .add_observation(
            &request.entity,
            &request.content,
            request.key.as_deref(),
            request.source.as_deref(),
        )
        .await
        .map_err(|e| e.to_string())?;

    let action = if previous.is_some() {
        "updated"
    } else {
        "created"
    };

    let mut response = serde_json::json!({
        "message": format!("Observation {}", action),
        "action": action,
        "entity": request.entity,
        "key": obs.key,
        "content": obs.content,
        "source": obs.source
    });

    if let Some(prev) = previous {
        response["previous_content"] = serde_json::json!(prev.content);
    }

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Get an observation by its semantic key.
pub async fn entity_observe_get(
    state: &ToolState,
    request: EntityObserveGetRequest,
) -> Result<String, String> {
    debug!(
        "entity_observe_get: entity={}, key={}",
        request.entity, request.key
    );

    let service_guard = state.entity_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Entity service not initialized".to_string())?;

    let obs = service
        .get_observation_by_key(&request.entity, &request.key)
        .await
        .map_err(|e| e.to_string())?;

    match obs {
        Some(o) => {
            let info = ObservationInfo {
                id: o.id.to_string(),
                key: o.key,
                content: o.content,
                source: o.source,
                created_at: o.created_at.format(&time::format_description::well_known::Rfc3339).unwrap(),
                updated_at: o.updated_at.format(&time::format_description::well_known::Rfc3339).unwrap(),
            };
            serde_json::to_string_pretty(&info).map_err(|e| e.to_string())
        }
        None => {
            serde_json::to_string_pretty(&serde_json::json!({
                "error": format!("No observation found with key '{}' for entity '{}'", request.key, request.entity)
            })).map_err(|e| e.to_string())
        }
    }
}

/// List observations with optional key pattern filtering.
pub async fn entity_observe_list(
    state: &ToolState,
    request: EntityObserveListRequest,
) -> Result<String, String> {
    debug!(
        "entity_observe_list: entity={}, pattern={:?}",
        request.entity, request.key_pattern
    );

    let service_guard = state.entity_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Entity service not initialized".to_string())?;

    let observations = service
        .list_observations_by_pattern(&request.entity, request.key_pattern.as_deref())
        .await
        .map_err(|e| e.to_string())?;

    let limit = request.limit.unwrap_or(50);
    let observations: Vec<ObservationInfo> = observations
        .into_iter()
        .take(limit)
        .map(|o| ObservationInfo {
            id: o.id.to_string(),
            key: o.key,
            content: o.content,
            source: o.source,
            created_at: o
                .created_at
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap(),
            updated_at: o
                .updated_at
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap(),
        })
        .collect();

    serde_json::to_string_pretty(&serde_json::json!({
        "entity": request.entity,
        "pattern": request.key_pattern,
        "count": observations.len(),
        "observations": observations
    }))
    .map_err(|e| e.to_string())
}

/// Search observations by content.
pub async fn entity_observe_search(
    state: &ToolState,
    request: EntityObserveSearchRequest,
) -> Result<String, String> {
    debug!(
        "entity_observe_search: entity={}, query={}",
        request.entity, request.query
    );

    let service_guard = state.entity_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Entity service not initialized".to_string())?;

    let limit = request.limit.unwrap_or(20);
    let observations = service
        .search_observations(&request.entity, &request.query, limit)
        .await
        .map_err(|e| e.to_string())?;

    let results: Vec<ObservationInfo> = observations
        .into_iter()
        .map(|o| ObservationInfo {
            id: o.id.to_string(),
            key: o.key,
            content: o.content,
            source: o.source,
            created_at: o
                .created_at
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap(),
            updated_at: o
                .updated_at
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap(),
        })
        .collect();

    serde_json::to_string_pretty(&serde_json::json!({
        "entity": request.entity,
        "query": request.query,
        "count": results.len(),
        "results": results
    }))
    .map_err(|e| e.to_string())
}

/// Get the update history for an observation.
pub async fn entity_observe_history(
    state: &ToolState,
    request: EntityObserveHistoryRequest,
) -> Result<String, String> {
    debug!(
        "entity_observe_history: entity={}, key={}",
        request.entity, request.key
    );

    let service_guard = state.entity_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Entity service not initialized".to_string())?;

    let history = service
        .get_observation_history(&request.entity, &request.key)
        .await
        .map_err(|e| e.to_string())?;

    let archived: Vec<ArchivedObservationInfo> = history
        .into_iter()
        .map(|a| ArchivedObservationInfo {
            content: a.content,
            source: a.source,
            created_at: a
                .created_at
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap(),
            archived_at: a
                .archived_at
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap(),
        })
        .collect();

    // Also get the current observation
    let current = service
        .get_observation_by_key(&request.entity, &request.key)
        .await
        .map_err(|e| e.to_string())?;

    let current_info = current.map(|o| ObservationInfo {
        id: o.id.to_string(),
        key: o.key,
        content: o.content,
        source: o.source,
        created_at: o
            .created_at
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap(),
        updated_at: o
            .updated_at
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap(),
    });

    serde_json::to_string_pretty(&serde_json::json!({
        "entity": request.entity,
        "key": request.key,
        "current": current_info,
        "history_count": archived.len(),
        "history": archived
    }))
    .map_err(|e| e.to_string())
}

/// Get entity statistics.
pub async fn entity_stats(
    state: &ToolState,
    _request: EntityStatsRequest,
) -> Result<String, String> {
    debug!("entity_stats");

    let service_guard = state.entity_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Entity service not initialized".to_string())?;

    let stats = service.stats().await.map_err(|e| e.to_string())?;

    let response = EntityStatsResponse {
        entity_count: stats.entity_count,
        relationship_count: stats.relationship_count,
        alias_count: stats.alias_count,
        observation_count: stats.observation_count,
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

// =============================================================================
// Layer 2: Session History Tools
// =============================================================================

/// Event type argument for MCP tools.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EventTypeArg {
    Decision,
    Command,
    FileChange,
    ToolUse,
    Error,
    Milestone,
    Observation,
}

impl From<EventTypeArg> for EventType {
    fn from(arg: EventTypeArg) -> Self {
        match arg {
            EventTypeArg::Decision => EventType::Decision,
            EventTypeArg::Command => EventType::Command,
            EventTypeArg::FileChange => EventType::FileChange,
            EventTypeArg::ToolUse => EventType::ToolUse,
            EventTypeArg::Error => EventType::Error,
            EventTypeArg::Milestone => EventType::Milestone,
            EventTypeArg::Observation => EventType::Observation,
        }
    }
}

/// Session status argument for MCP tools.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatusArg {
    Active,
    Completed,
    Abandoned,
}

impl From<SessionStatusArg> for SessionStatus {
    fn from(arg: SessionStatusArg) -> Self {
        match arg {
            SessionStatusArg::Active => SessionStatus::Active,
            SessionStatusArg::Completed => SessionStatus::Completed,
            SessionStatusArg::Abandoned => SessionStatus::Abandoned,
        }
    }
}

/// Request to start a new session.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionStartRequest {
    /// The agent type (e.g., "claude-code", "cursor").
    #[schemars(description = "Agent type like 'claude-code', 'cursor', 'gemini-cli'")]
    pub agent: Option<String>,

    /// The project being worked on.
    #[schemars(description = "Project name or working directory")]
    pub project: Option<String>,

    /// Goal of this session.
    #[schemars(description = "What you're trying to accomplish in this session")]
    pub goal: Option<String>,
}

/// Response from starting a session.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionStartResponse {
    /// The session ID.
    pub id: String,
    /// The agent type.
    pub agent: Option<String>,
    /// The project.
    pub project: Option<String>,
    /// The goal.
    pub goal: Option<String>,
    /// When the session started.
    pub started_at: String,
}

/// Request to end a session.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionEndRequest {
    /// The session ID to end.
    #[schemars(description = "Session ID to end")]
    pub session_id: String,

    /// Summary of what was accomplished.
    #[schemars(description = "Summary of what was accomplished in this session")]
    pub summary: Option<String>,
}

/// Request to get a session.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionGetRequest {
    /// The session ID.
    #[schemars(description = "Session ID to retrieve")]
    pub session_id: String,
}

/// Session information.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionInfo {
    pub id: String,
    pub agent: Option<String>,
    pub project: Option<String>,
    pub goal: Option<String>,
    pub status: String,
    pub summary: Option<String>,
    pub started_at: String,
    pub ended_at: Option<String>,
}

/// Event information.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EventInfo {
    pub id: String,
    pub event_type: String,
    pub actor: String,
    pub content: String,
    pub context: Option<String>,
    pub source: Option<String>,
    pub timestamp: String,
}

/// Response with session details and events.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionGetResponse {
    pub session: SessionInfo,
    pub events: Vec<EventInfo>,
}

/// Request to list sessions.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionListRequest {
    /// Filter by status.
    #[schemars(description = "Filter by status: active, completed, abandoned")]
    pub status: Option<SessionStatusArg>,

    /// Filter by agent.
    #[schemars(description = "Filter by agent type")]
    pub agent: Option<String>,

    /// Filter by project.
    #[schemars(description = "Filter by project")]
    pub project: Option<String>,

    /// Maximum number of sessions.
    #[schemars(description = "Maximum sessions to return (default: 20)")]
    pub limit: Option<usize>,
}

/// Response with list of sessions.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionListResponse {
    pub sessions: Vec<SessionInfo>,
    pub count: usize,
}

/// Request to log an event.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionLogRequest {
    /// The session ID to log to.
    #[schemars(description = "Session ID to log the event to")]
    pub session_id: String,

    /// The type of event.
    #[schemars(
        description = "Event type: decision, observation, error, command, file_change, tool_use, milestone"
    )]
    pub event_type: EventTypeArg,

    /// The event content.
    #[schemars(description = "The main content/message of the event")]
    pub content: String,

    /// Additional context.
    #[schemars(description = "Why this happened, rationale, additional context")]
    pub context: Option<String>,

    /// Source of the event.
    #[schemars(description = "Where this event came from (file path, tool name, etc.)")]
    pub source: Option<String>,
}

/// Response from logging an event.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionLogResponse {
    pub event: EventInfo,
}

/// Request to search events.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionSearchRequest {
    /// Search query.
    #[schemars(description = "Search query to find in event content")]
    pub query: String,

    /// Maximum results.
    #[schemars(description = "Maximum results to return (default: 20)")]
    pub limit: Option<usize>,
}

/// Response with search results.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionSearchResponse {
    pub events: Vec<EventInfo>,
    pub count: usize,
}

/// Request for session statistics.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionStatsRequest {}

/// Response with session statistics.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionStatsResponse {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub completed_sessions: usize,
    pub abandoned_sessions: usize,
    pub total_events: usize,
    pub events_by_type: std::collections::HashMap<String, usize>,
}

// =============================================================================
// Session Tool Implementations
// =============================================================================

/// Start a new session.
pub async fn session_start(
    state: &ToolState,
    request: SessionStartRequest,
) -> Result<String, String> {
    debug!(
        "session_start: agent={:?}, project={:?}",
        request.agent, request.project
    );

    let service_guard = state.session_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Session service not initialized".to_string())?;

    let session = service
        .start_session(
            request.agent.as_deref(),
            request.project.as_deref(),
            request.goal.as_deref(),
        )
        .await
        .map_err(|e| e.to_string())?;

    let response = SessionStartResponse {
        id: session.id.to_string(),
        agent: session.agent,
        project: session.project,
        goal: session.goal,
        started_at: session
            .started_at
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default(),
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// End a session.
pub async fn session_end(state: &ToolState, request: SessionEndRequest) -> Result<String, String> {
    debug!("session_end: session_id={}", request.session_id);

    let service_guard = state.session_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Session service not initialized".to_string())?;

    let id = engram_core::id::Id::parse(&request.session_id)
        .map_err(|e| format!("Invalid session ID: {}", e))?;

    service
        .end_session(&id, request.summary.as_deref())
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_string_pretty(&serde_json::json!({
        "message": format!("Session {} ended", request.session_id),
        "summary": request.summary
    }))
    .map_err(|e| e.to_string())
}

/// Get a session with its events.
pub async fn session_get(state: &ToolState, request: SessionGetRequest) -> Result<String, String> {
    debug!("session_get: session_id={}", request.session_id);

    let service_guard = state.session_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Session service not initialized".to_string())?;

    let id = engram_core::id::Id::parse(&request.session_id)
        .map_err(|e| format!("Invalid session ID: {}", e))?;

    let (session, events) = service
        .get_session_with_events(&id)
        .await
        .map_err(|e| e.to_string())?;

    let response = SessionGetResponse {
        session: SessionInfo {
            id: session.id.to_string(),
            agent: session.agent,
            project: session.project,
            goal: session.goal,
            status: session.status.to_string(),
            summary: session.summary,
            started_at: session
                .started_at
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default(),
            ended_at: session.ended_at.map(|dt| {
                dt.format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_default()
            }),
        },
        events: events
            .into_iter()
            .map(|e| EventInfo {
                id: e.id.to_string(),
                event_type: e.event_type.to_string(),
                actor: e.actor,
                content: e.content,
                context: e.context,
                source: e.source,
                timestamp: e
                    .timestamp
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_default(),
            })
            .collect(),
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// List sessions.
pub async fn session_list(
    state: &ToolState,
    request: SessionListRequest,
) -> Result<String, String> {
    debug!(
        "session_list: status={:?}, agent={:?}",
        request.status, request.agent
    );

    let service_guard = state.session_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Session service not initialized".to_string())?;

    let status: Option<SessionStatus> = request.status.map(|s| s.into());

    let sessions = service
        .list_sessions(
            status.as_ref(),
            request.agent.as_deref(),
            request.project.as_deref(),
            Some(request.limit.unwrap_or(20)),
        )
        .await
        .map_err(|e| e.to_string())?;

    let response = SessionListResponse {
        count: sessions.len(),
        sessions: sessions
            .into_iter()
            .map(|s| SessionInfo {
                id: s.id.to_string(),
                agent: s.agent,
                project: s.project,
                goal: s.goal,
                status: s.status.to_string(),
                summary: s.summary,
                started_at: s
                    .started_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_default(),
                ended_at: s.ended_at.map(|dt| {
                    dt.format(&time::format_description::well_known::Rfc3339)
                        .unwrap_or_default()
                }),
            })
            .collect(),
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Log an event to a session.
pub async fn session_log(state: &ToolState, request: SessionLogRequest) -> Result<String, String> {
    debug!(
        "session_log: session_id={}, event_type={:?}",
        request.session_id, request.event_type
    );

    let service_guard = state.session_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Session service not initialized".to_string())?;

    let session_id = engram_core::id::Id::parse(&request.session_id)
        .map_err(|e| format!("Invalid session ID: {}", e))?;

    let event = service
        .log_event(
            &session_id,
            request.event_type.into(),
            &request.content,
            request.context.as_deref(),
            request.source.as_deref(),
        )
        .await
        .map_err(|e| e.to_string())?;

    let response = SessionLogResponse {
        event: EventInfo {
            id: event.id.to_string(),
            event_type: event.event_type.to_string(),
            actor: event.actor,
            content: event.content,
            context: event.context,
            source: event.source,
            timestamp: event
                .timestamp
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default(),
        },
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Search events across sessions.
pub async fn session_search(
    state: &ToolState,
    request: SessionSearchRequest,
) -> Result<String, String> {
    debug!("session_search: query={}", request.query);

    let service_guard = state.session_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Session service not initialized".to_string())?;

    let events = service
        .search_events(&request.query, Some(request.limit.unwrap_or(20)))
        .await
        .map_err(|e| e.to_string())?;

    let response = SessionSearchResponse {
        count: events.len(),
        events: events
            .into_iter()
            .map(|e| EventInfo {
                id: e.id.to_string(),
                event_type: e.event_type.to_string(),
                actor: e.actor,
                content: e.content,
                context: e.context,
                source: e.source,
                timestamp: e
                    .timestamp
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_default(),
            })
            .collect(),
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Get session statistics.
pub async fn session_stats(
    state: &ToolState,
    _request: SessionStatsRequest,
) -> Result<String, String> {
    debug!("session_stats");

    let service_guard = state.session_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Session service not initialized".to_string())?;

    let stats = service.stats().await.map_err(|e| e.to_string())?;

    let response = SessionStatsResponse {
        total_sessions: stats.total_sessions,
        active_sessions: stats.active_sessions,
        completed_sessions: stats.completed_sessions,
        abandoned_sessions: stats.abandoned_sessions,
        total_events: stats.total_events,
        events_by_type: stats.events_by_type,
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

// =============================================================================
// Layer 4: Tool Intelligence Types
// =============================================================================

/// Tool outcome argument for MCP tools.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ToolOutcomeArg {
    Success,
    Partial,
    Failed,
    Switched,
}

impl From<ToolOutcomeArg> for ToolOutcome {
    fn from(arg: ToolOutcomeArg) -> Self {
        match arg {
            ToolOutcomeArg::Success => ToolOutcome::Success,
            ToolOutcomeArg::Partial => ToolOutcome::Partial,
            ToolOutcomeArg::Failed => ToolOutcome::Failed,
            ToolOutcomeArg::Switched => ToolOutcome::Switched,
        }
    }
}

/// Request to log a tool usage.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolLogUsageRequest {
    /// Tool name (must be registered as an entity first).
    #[schemars(description = "Name of the tool (must be a registered entity of type 'tool')")]
    pub tool_name: String,

    /// Context of the usage (what was the user trying to do?).
    #[schemars(description = "Context/description of what the tool was used for")]
    pub context: String,

    /// Outcome of the tool usage.
    #[schemars(description = "Outcome: success, partial, failed, or switched")]
    pub outcome: ToolOutcomeArg,

    /// Optional session ID to associate with this usage.
    #[schemars(description = "Session ID to link this usage to (optional)")]
    pub session_id: Option<String>,
}

/// Response from logging a tool usage.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolLogUsageResponse {
    /// Usage ID.
    pub id: String,
    /// Tool name.
    pub tool_name: String,
    /// Context.
    pub context: String,
    /// Outcome.
    pub outcome: String,
}

/// Request to get tool recommendations.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolRecommendRequest {
    /// Context to get recommendations for.
    #[schemars(description = "Description of what you're trying to do")]
    pub context: String,
}

/// A single tool recommendation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolRecommendationInfo {
    /// Tool name.
    pub tool_name: String,
    /// Confidence score (0.0 - 1.0).
    pub confidence: f32,
    /// Reason for the recommendation.
    pub reason: String,
}

/// Response with tool recommendations.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolRecommendResponse {
    /// Recommendations sorted by confidence.
    pub recommendations: Vec<ToolRecommendationInfo>,
    /// Number of recommendations.
    pub count: usize,
}

/// Request to get tool statistics.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolGetStatsRequest {
    /// Tool name.
    #[schemars(description = "Name of the tool to get statistics for")]
    pub tool_name: String,
}

/// Response with tool statistics.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolGetStatsResponse {
    /// Tool name.
    pub tool_name: String,
    /// Total number of usages.
    pub total_usages: usize,
    /// Number of successful usages.
    pub success_count: usize,
    /// Number of failed usages.
    pub failure_count: usize,
    /// Success rate (0.0 - 1.0).
    pub success_rate: f32,
    /// Number of preferences involving this tool.
    pub preferences_count: usize,
}

/// Request to list tool usages.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolListUsagesRequest {
    /// Optional outcome filter.
    #[schemars(description = "Filter by outcome (optional)")]
    pub outcome: Option<ToolOutcomeArg>,

    /// Maximum number of results.
    #[schemars(description = "Maximum results to return (default: 20)")]
    pub limit: Option<usize>,
}

/// Tool usage information.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolUsageInfoMcp {
    /// Usage ID.
    pub id: String,
    /// Tool name.
    pub tool_name: String,
    /// Context.
    pub context: String,
    /// Outcome.
    pub outcome: String,
    /// Timestamp.
    pub timestamp: String,
}

/// Response with list of tool usages.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolListUsagesResponse {
    /// Tool usages.
    pub usages: Vec<ToolUsageInfoMcp>,
    /// Count.
    pub count: usize,
}

/// Request to search tool usages.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolSearchRequest {
    /// Search query.
    #[schemars(description = "Search query to find in usage context")]
    pub query: String,

    /// Maximum results.
    #[schemars(description = "Maximum results to return (default: 20)")]
    pub limit: Option<usize>,
}

/// Request for overall tool intelligence statistics.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolIntelStatsRequest {}

/// Response with overall tool intelligence statistics.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolIntelStatsResponse {
    /// Number of usage records.
    pub usage_count: u64,
    /// Number of learned preferences.
    pub preference_count: u64,
}

// =============================================================================
// Layer 4: Tool Intelligence Implementations
// =============================================================================

/// Log a tool usage.
pub async fn tool_log_usage(
    state: &ToolState,
    request: ToolLogUsageRequest,
) -> Result<String, String> {
    debug!(
        "tool_log_usage: tool={}, outcome={:?}",
        request.tool_name, request.outcome
    );

    let service_guard = state.tool_intel_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Tool intelligence service not initialized".to_string())?;

    let session_id = if let Some(sid) = &request.session_id {
        Some(engram_core::id::Id::parse(sid).map_err(|e| format!("Invalid session ID: {}", e))?)
    } else {
        None
    };

    let usage = service
        .log_usage(
            &request.tool_name,
            &request.context,
            request.outcome.into(),
            session_id.as_ref(),
        )
        .await
        .map_err(|e| e.to_string())?;

    let response = ToolLogUsageResponse {
        id: usage.id.to_string(),
        tool_name: request.tool_name,
        context: usage.context,
        outcome: usage.outcome.to_string(),
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Get tool recommendations for a context.
pub async fn tool_recommend(
    state: &ToolState,
    request: ToolRecommendRequest,
) -> Result<String, String> {
    debug!("tool_recommend: context={}", request.context);

    let service_guard = state.tool_intel_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Tool intelligence service not initialized".to_string())?;

    let recommendations = service
        .get_recommendations(&request.context)
        .await
        .map_err(|e| e.to_string())?;

    let response = ToolRecommendResponse {
        count: recommendations.len(),
        recommendations: recommendations
            .into_iter()
            .map(|r| ToolRecommendationInfo {
                tool_name: r.tool_name,
                confidence: r.confidence,
                reason: r.reason,
            })
            .collect(),
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Get statistics for a specific tool.
pub async fn tool_get_stats(
    state: &ToolState,
    request: ToolGetStatsRequest,
) -> Result<String, String> {
    debug!("tool_get_stats: tool={}", request.tool_name);

    let service_guard = state.tool_intel_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Tool intelligence service not initialized".to_string())?;

    let stats = service
        .get_tool_stats(&request.tool_name)
        .await
        .map_err(|e| e.to_string())?;

    let response = ToolGetStatsResponse {
        tool_name: request.tool_name,
        total_usages: stats.total_usages,
        success_count: stats.success_count,
        failure_count: stats.failure_count,
        success_rate: stats.success_rate,
        preferences_count: stats.preferences_count,
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// List recent tool usages.
pub async fn tool_list_usages(
    state: &ToolState,
    request: ToolListUsagesRequest,
) -> Result<String, String> {
    debug!(
        "tool_list_usages: outcome={:?}, limit={:?}",
        request.outcome, request.limit
    );

    let service_guard = state.tool_intel_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Tool intelligence service not initialized".to_string())?;

    let outcome: Option<ToolOutcome> = request.outcome.map(|o| o.into());

    let usages = service
        .list_usages(outcome.as_ref(), request.limit)
        .await
        .map_err(|e| e.to_string())?;

    let response = ToolListUsagesResponse {
        count: usages.len(),
        usages: usages
            .into_iter()
            .map(|u| ToolUsageInfoMcp {
                id: u.id.to_string(),
                tool_name: u.tool_name,
                context: u.context,
                outcome: u.outcome.to_string(),
                timestamp: u
                    .timestamp
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_default(),
            })
            .collect(),
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Search tool usage history.
pub async fn tool_search(state: &ToolState, request: ToolSearchRequest) -> Result<String, String> {
    debug!("tool_search: query={}", request.query);

    let service_guard = state.tool_intel_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Tool intelligence service not initialized".to_string())?;

    let usages = service
        .search_usages(&request.query, request.limit)
        .await
        .map_err(|e| e.to_string())?;

    let response = ToolListUsagesResponse {
        count: usages.len(),
        usages: usages
            .into_iter()
            .map(|u| ToolUsageInfoMcp {
                id: u.id.to_string(),
                tool_name: u.tool_name,
                context: u.context,
                outcome: u.outcome.to_string(),
                timestamp: u
                    .timestamp
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_default(),
            })
            .collect(),
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Get overall tool intelligence statistics.
pub async fn tool_intel_stats(
    state: &ToolState,
    _request: ToolIntelStatsRequest,
) -> Result<String, String> {
    debug!("tool_intel_stats");

    let service_guard = state.tool_intel_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Tool intelligence service not initialized".to_string())?;

    let stats = service.stats().await.map_err(|e| e.to_string())?;

    let response = ToolIntelStatsResponse {
        usage_count: stats.usage_count,
        preference_count: stats.preference_count,
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

// =============================================================================
// Layer 5: Session Coordination Types
// =============================================================================

/// Request to register a session for coordination.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoordRegisterRequest {
    /// Session ID to register.
    #[schemars(description = "Session ID to register for coordination")]
    pub session_id: String,

    /// Agent type (e.g., "claude-code", "cursor").
    #[schemars(description = "Agent type like 'claude-code', 'cursor', 'gemini-cli'")]
    pub agent: String,

    /// Project being worked on.
    #[schemars(description = "Project name or working directory")]
    pub project: String,

    /// Goal of this session.
    #[schemars(description = "What you're trying to accomplish in this session")]
    pub goal: String,

    /// Components being worked on.
    #[schemars(description = "Components/modules being touched (for conflict detection)")]
    #[serde(default)]
    pub components: Vec<String>,
}

/// Response from registering a session.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoordRegisterResponse {
    /// Session ID.
    pub session_id: String,
    /// Agent type.
    pub agent: String,
    /// Project.
    pub project: String,
    /// Goal.
    pub goal: String,
    /// Components.
    pub components: Vec<String>,
    /// When registered.
    pub started_at: String,
}

/// Request to unregister a session.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoordUnregisterRequest {
    /// Session ID to unregister.
    #[schemars(description = "Session ID to unregister")]
    pub session_id: String,
}

/// Request to send a heartbeat.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoordHeartbeatRequest {
    /// Session ID to heartbeat.
    #[schemars(description = "Session ID to send heartbeat for")]
    pub session_id: String,
}

/// Request to set the current file being edited.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoordSetFileRequest {
    /// Session ID.
    #[schemars(description = "Session ID")]
    pub session_id: String,

    /// Current file path (or null to clear).
    #[schemars(description = "File path being edited, or null to clear")]
    pub file: Option<String>,
}

/// Response from setting file (includes any conflicts detected).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoordSetFileResponse {
    /// Whether the file was set.
    pub success: bool,
    /// Any conflicts detected.
    pub conflicts: Vec<ConflictInfoMcp>,
}

/// Request to update components being worked on.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoordSetComponentsRequest {
    /// Session ID.
    #[schemars(description = "Session ID")]
    pub session_id: String,

    /// Components being worked on.
    #[schemars(description = "Components/modules being touched")]
    pub components: Vec<String>,
}

/// Response from setting components (includes any conflicts detected).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoordSetComponentsResponse {
    /// Whether components were set.
    pub success: bool,
    /// Any conflicts detected.
    pub conflicts: Vec<ConflictInfoMcp>,
}

/// Request to check for conflicts.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoordCheckConflictsRequest {
    /// Session ID.
    #[schemars(description = "Session ID to check conflicts for")]
    pub session_id: String,
}

/// Conflict information for MCP.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConflictInfoMcp {
    /// The other session ID.
    pub other_session_id: String,
    /// Agent of the other session.
    pub other_agent: String,
    /// Goal of the other session.
    pub other_goal: String,
    /// Overlapping components.
    pub overlapping_components: Vec<String>,
    /// Current file of the other session.
    pub other_current_file: Option<String>,
}

/// Response with conflict check results.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoordCheckConflictsResponse {
    /// Whether there are any conflicts.
    pub has_conflicts: bool,
    /// Component-based conflicts.
    pub component_conflicts: Vec<ConflictInfoMcp>,
    /// File-based conflicts.
    pub file_conflicts: Vec<ConflictInfoMcp>,
}

/// Request to list active sessions.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoordListRequest {
    /// Optional project filter.
    #[schemars(description = "Filter by project (optional)")]
    pub project: Option<String>,
}

/// Active session information for MCP.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ActiveSessionInfoMcp {
    /// Session ID.
    pub session_id: String,
    /// Agent type.
    pub agent: String,
    /// Project.
    pub project: String,
    /// Goal.
    pub goal: String,
    /// Components being worked on.
    pub components: Vec<String>,
    /// Current file being edited.
    pub current_file: Option<String>,
    /// When the session started.
    pub started_at: String,
    /// Last heartbeat.
    pub last_heartbeat: String,
}

/// Response with list of active sessions.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoordListResponse {
    /// Active sessions.
    pub sessions: Vec<ActiveSessionInfoMcp>,
    /// Count.
    pub count: usize,
}

/// Request for coordination statistics.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoordStatsRequest {}

/// Response with coordination statistics.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoordStatsResponse {
    /// Number of active sessions.
    pub active_sessions: u64,
}

// =============================================================================
// Layer 5: Session Coordination Implementations
// =============================================================================

/// Register a session for coordination.
pub async fn coord_register(
    state: &ToolState,
    request: CoordRegisterRequest,
) -> Result<String, String> {
    debug!(
        "coord_register: session_id={}, agent={}",
        request.session_id, request.agent
    );

    let service_guard = state.coordination_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Coordination service not initialized".to_string())?;

    let session_id = engram_core::id::Id::parse(&request.session_id)
        .map_err(|e| format!("Invalid session ID: {}", e))?;

    let session = if request.components.is_empty() {
        service
            .register(&session_id, &request.agent, &request.project, &request.goal)
            .await
            .map_err(|e| e.to_string())?
    } else {
        service
            .register_with_components(
                &session_id,
                &request.agent,
                &request.project,
                &request.goal,
                request.components,
            )
            .await
            .map_err(|e| e.to_string())?
    };

    let response = CoordRegisterResponse {
        session_id: session.session_id.to_string(),
        agent: session.agent,
        project: session.project,
        goal: session.goal,
        components: session.components,
        started_at: session
            .started_at
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default(),
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Unregister a session.
pub async fn coord_unregister(
    state: &ToolState,
    request: CoordUnregisterRequest,
) -> Result<String, String> {
    debug!("coord_unregister: session_id={}", request.session_id);

    let service_guard = state.coordination_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Coordination service not initialized".to_string())?;

    let session_id = engram_core::id::Id::parse(&request.session_id)
        .map_err(|e| format!("Invalid session ID: {}", e))?;

    service
        .unregister(&session_id)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_string_pretty(&serde_json::json!({
        "message": format!("Session {} unregistered", request.session_id)
    }))
    .map_err(|e| e.to_string())
}

/// Send heartbeat for a session.
pub async fn coord_heartbeat(
    state: &ToolState,
    request: CoordHeartbeatRequest,
) -> Result<String, String> {
    debug!("coord_heartbeat: session_id={}", request.session_id);

    let service_guard = state.coordination_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Coordination service not initialized".to_string())?;

    let session_id = engram_core::id::Id::parse(&request.session_id)
        .map_err(|e| format!("Invalid session ID: {}", e))?;

    service
        .heartbeat(&session_id)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_string_pretty(&serde_json::json!({
        "message": "Heartbeat recorded",
        "session_id": request.session_id
    }))
    .map_err(|e| e.to_string())
}

/// Set the current file being edited.
pub async fn coord_set_file(
    state: &ToolState,
    request: CoordSetFileRequest,
) -> Result<String, String> {
    debug!(
        "coord_set_file: session_id={}, file={:?}",
        request.session_id, request.file
    );

    let service_guard = state.coordination_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Coordination service not initialized".to_string())?;

    let session_id = engram_core::id::Id::parse(&request.session_id)
        .map_err(|e| format!("Invalid session ID: {}", e))?;

    let conflicts = service
        .set_current_file(&session_id, request.file.as_deref())
        .await
        .map_err(|e| e.to_string())?;

    let response = CoordSetFileResponse {
        success: true,
        conflicts: conflicts
            .into_iter()
            .map(|c| ConflictInfoMcp {
                other_session_id: c.other_session_id.to_string(),
                other_agent: c.other_agent,
                other_goal: c.other_goal,
                overlapping_components: c.overlapping_components,
                other_current_file: c.other_current_file,
            })
            .collect(),
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Update components being worked on.
pub async fn coord_set_components(
    state: &ToolState,
    request: CoordSetComponentsRequest,
) -> Result<String, String> {
    debug!(
        "coord_set_components: session_id={}, components={:?}",
        request.session_id, request.components
    );

    let service_guard = state.coordination_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Coordination service not initialized".to_string())?;

    let session_id = engram_core::id::Id::parse(&request.session_id)
        .map_err(|e| format!("Invalid session ID: {}", e))?;

    let conflicts = service
        .set_components(&session_id, &request.components)
        .await
        .map_err(|e| e.to_string())?;

    let response = CoordSetComponentsResponse {
        success: true,
        conflicts: conflicts
            .into_iter()
            .map(|c| ConflictInfoMcp {
                other_session_id: c.other_session_id.to_string(),
                other_agent: c.other_agent,
                other_goal: c.other_goal,
                overlapping_components: c.overlapping_components,
                other_current_file: c.other_current_file,
            })
            .collect(),
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Check for conflicts with other sessions.
pub async fn coord_check_conflicts(
    state: &ToolState,
    request: CoordCheckConflictsRequest,
) -> Result<String, String> {
    debug!("coord_check_conflicts: session_id={}", request.session_id);

    let service_guard = state.coordination_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Coordination service not initialized".to_string())?;

    let session_id = engram_core::id::Id::parse(&request.session_id)
        .map_err(|e| format!("Invalid session ID: {}", e))?;

    // Get component conflicts
    let component_conflicts = service
        .check_conflicts(&session_id)
        .await
        .map_err(|e| e.to_string())?;

    // Get file conflicts (need to get current file first)
    let session = service.get(&session_id).await.map_err(|e| e.to_string())?;

    let file_conflicts = if let Some(s) = &session {
        if let Some(file) = &s.current_file {
            service
                .check_file_conflicts(&session_id, file)
                .await
                .map_err(|e| e.to_string())?
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let has_conflicts = !component_conflicts.is_empty() || !file_conflicts.is_empty();

    let response = CoordCheckConflictsResponse {
        has_conflicts,
        component_conflicts: component_conflicts
            .into_iter()
            .map(|c| ConflictInfoMcp {
                other_session_id: c.other_session_id.to_string(),
                other_agent: c.other_agent,
                other_goal: c.other_goal,
                overlapping_components: c.overlapping_components,
                other_current_file: c.other_current_file,
            })
            .collect(),
        file_conflicts: file_conflicts
            .into_iter()
            .map(|c| ConflictInfoMcp {
                other_session_id: c.other_session_id.to_string(),
                other_agent: c.other_agent,
                other_goal: c.other_goal,
                overlapping_components: c.overlapping_components,
                other_current_file: c.other_current_file,
            })
            .collect(),
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// List active sessions.
pub async fn coord_list(state: &ToolState, request: CoordListRequest) -> Result<String, String> {
    debug!("coord_list: project={:?}", request.project);

    let service_guard = state.coordination_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Coordination service not initialized".to_string())?;

    let sessions = if let Some(project) = &request.project {
        service
            .list_for_project(project)
            .await
            .map_err(|e| e.to_string())?
    } else {
        service.list_active().await.map_err(|e| e.to_string())?
    };

    let response = CoordListResponse {
        count: sessions.len(),
        sessions: sessions
            .into_iter()
            .map(|s| ActiveSessionInfoMcp {
                session_id: s.session_id.to_string(),
                agent: s.agent,
                project: s.project,
                goal: s.goal,
                components: s.components,
                current_file: s.current_file,
                started_at: s
                    .started_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_default(),
                last_heartbeat: s
                    .last_heartbeat
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_default(),
            })
            .collect(),
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Get coordination statistics.
pub async fn coord_stats(state: &ToolState, _request: CoordStatsRequest) -> Result<String, String> {
    debug!("coord_stats");

    let service_guard = state.coordination_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Coordination service not initialized".to_string())?;

    let stats = service.stats().await.map_err(|e| e.to_string())?;

    let response = CoordStatsResponse {
        active_sessions: stats.active_session_count,
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

// =============================================================================
// Unified Search Tool
// =============================================================================

/// Request to search across all knowledge layers.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchRequest {
    /// The search query.
    #[schemars(
        description = "Search query - searches entity names, descriptions, observations, session events, documents, and tool usages"
    )]
    pub query: String,

    /// Maximum results per layer.
    #[schemars(description = "Maximum results per layer (default: 5)")]
    #[serde(default = "default_search_limit")]
    pub limit: usize,

    /// Minimum score threshold (0.0 - 1.0).
    #[schemars(description = "Minimum similarity score threshold, 0.0-1.0 (default: 0.3)")]
    #[serde(default)]
    pub min_score: Option<f32>,

    /// Filter to specific layers.
    #[schemars(
        description = "Filter to specific layers: entity, alias, observation, session_event, document, tool_usage"
    )]
    pub layers: Option<Vec<String>>,
}

fn default_search_limit() -> usize {
    5
}

/// A single unified search result.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UnifiedSearchResultInfo {
    /// Which layer the result came from.
    pub source: String,
    /// Relevance score (0.0-1.0, higher is better).
    pub score: f32,
    /// Primary title (entity name, doc title, etc.).
    pub title: String,
    /// Matching content snippet.
    pub content: String,
    /// Context information (parent entity, session ID, etc.).
    pub context: Option<String>,
    /// ID for follow-up queries.
    pub id: String,
}

/// Response from unified search.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchResponse {
    /// Search results sorted by score.
    pub results: Vec<UnifiedSearchResultInfo>,
    /// Total number of results.
    pub count: usize,
    /// Results count by layer.
    pub by_layer: std::collections::HashMap<String, usize>,
}

/// Execute a unified search across all layers.
pub async fn search(state: &ToolState, request: SearchRequest) -> Result<String, String> {
    debug!(
        "search: query='{}', limit={}, layers={:?}",
        request.query, request.limit, request.layers
    );

    let service_guard = state.search_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Search service not initialized".to_string())?;

    // Parse layers if specified
    let layers: Option<Vec<SearchLayer>> = request.layers.as_ref().map(|layer_strs| {
        layer_strs
            .iter()
            .filter_map(|s| SearchLayer::parse(s))
            .collect()
    });

    let results = service
        .search(
            &request.query,
            request.limit,
            request.min_score,
            layers.as_deref(),
        )
        .await
        .map_err(|e| e.to_string())?;

    // Count results by layer
    let mut by_layer: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for result in &results {
        *by_layer.entry(result.source.to_string()).or_insert(0) += 1;
    }

    let response = SearchResponse {
        count: results.len(),
        results: results
            .into_iter()
            .map(|r| UnifiedSearchResultInfo {
                source: r.source.to_string(),
                score: r.score,
                title: r.title,
                content: r.content,
                context: r.context,
                id: r.id,
            })
            .collect(),
        by_layer,
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

// ============================================================================
// Layer 7: Work Management Tools
// ============================================================================

// ----------------------------------------------------------------------------
// Project Management
// ----------------------------------------------------------------------------

/// Request to create a new project.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkProjectCreateRequest {
    /// Project name (must be unique).
    #[schemars(description = "Unique project name")]
    pub name: String,
    /// Optional project description.
    #[schemars(description = "Project description")]
    pub description: Option<String>,
}

/// Response from project creation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkProjectResponse {
    /// Project ID.
    pub id: String,
    /// Project name.
    pub name: String,
    /// Project description.
    pub description: Option<String>,
    /// Project status.
    pub status: String,
    /// Creation timestamp.
    pub created_at: String,
}

/// Create a new project.
pub async fn work_project_create(
    state: &ToolState,
    request: WorkProjectCreateRequest,
) -> Result<String, String> {
    debug!("work_project_create: name={}", request.name);

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    let project = service
        .create_project(&request.name, request.description.as_deref())
        .await
        .map_err(|e| e.to_string())?;

    let response = WorkProjectResponse {
        id: project.id.to_string(),
        name: project.name,
        description: project.description,
        status: project.status.to_string(),
        created_at: project
            .created_at
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap(),
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Request to get a project.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkProjectGetRequest {
    /// Project name.
    #[schemars(description = "Project name")]
    pub name: String,
}

/// Get a project by name.
pub async fn work_project_get(
    state: &ToolState,
    request: WorkProjectGetRequest,
) -> Result<String, String> {
    debug!("work_project_get: name={}", request.name);

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    let project = service
        .get_project(&request.name)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Project not found: {}", request.name))?;

    // Get additional details
    let tasks = service
        .list_tasks(&request.name, None)
        .await
        .map_err(|e| e.to_string())?;

    let prs = service
        .list_prs(&request.name, None)
        .await
        .map_err(|e| e.to_string())?;

    let entities = service
        .get_project_entities(&request.name)
        .await
        .map_err(|e| e.to_string())?;

    let response = serde_json::json!({
        "id": project.id.to_string(),
        "name": project.name,
        "description": project.description,
        "status": project.status.to_string(),
        "created_at": project.created_at.format(&time::format_description::well_known::Rfc3339).unwrap(),
        "task_count": tasks.len(),
        "tasks": tasks.iter().map(|t| serde_json::json!({
            "id": t.id.to_string(),
            "name": t.name,
            "status": t.status.to_string(),
            "priority": t.priority.to_string(),
            "jira_key": t.jira_key,
        })).collect::<Vec<_>>(),
        "pr_count": prs.len(),
        "prs": prs.iter().map(|p| serde_json::json!({
            "url": p.url,
            "repo": p.repo,
            "status": p.status.to_string(),
        })).collect::<Vec<_>>(),
        "entity_count": entities.len(),
        "entities": entities.iter().map(|e| e.name.clone()).collect::<Vec<_>>(),
    });

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Request to list projects.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkProjectListRequest {
    /// Optional status filter.
    #[schemars(description = "Filter by status: planning, active, completed, archived")]
    pub status: Option<String>,
}

/// List all projects.
pub async fn work_project_list(
    state: &ToolState,
    request: WorkProjectListRequest,
) -> Result<String, String> {
    debug!("work_project_list: status={:?}", request.status);

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    let status = request
        .status
        .map(|s| engram_core::work::ProjectStatus::parse(&s));

    let projects = service
        .list_projects(status)
        .await
        .map_err(|e| e.to_string())?;

    let response = serde_json::json!({
        "count": projects.len(),
        "projects": projects.iter().map(|p| serde_json::json!({
            "id": p.id.to_string(),
            "name": p.name,
            "description": p.description,
            "status": p.status.to_string(),
        })).collect::<Vec<_>>(),
    });

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Request to update project status.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkProjectUpdateStatusRequest {
    /// Project name.
    #[schemars(description = "Project name")]
    pub name: String,
    /// New status.
    #[schemars(description = "New status: planning, active, completed, archived")]
    pub status: String,
}

/// Update project status.
pub async fn work_project_update_status(
    state: &ToolState,
    request: WorkProjectUpdateStatusRequest,
) -> Result<String, String> {
    debug!(
        "work_project_update_status: name={}, status={}",
        request.name, request.status
    );

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    let status = engram_core::work::ProjectStatus::parse(&request.status);

    let project = service
        .update_project_status(&request.name, status)
        .await
        .map_err(|e| e.to_string())?;

    let response = WorkProjectResponse {
        id: project.id.to_string(),
        name: project.name,
        description: project.description,
        status: project.status.to_string(),
        created_at: project
            .created_at
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap(),
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Request to connect project to entity.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkProjectConnectEntityRequest {
    /// Project name.
    #[schemars(description = "Project name")]
    pub project: String,
    /// Entity name.
    #[schemars(description = "Entity name to connect")]
    pub entity: String,
    /// Relation type.
    #[schemars(description = "Relation type: involves, depends_on, produces")]
    pub relation: Option<String>,
}

/// Connect project to entity.
pub async fn work_project_connect_entity(
    state: &ToolState,
    request: WorkProjectConnectEntityRequest,
) -> Result<String, String> {
    debug!(
        "work_project_connect_entity: project={}, entity={}",
        request.project, request.entity
    );

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    service
        .connect_project_to_entity(
            &request.project,
            &request.entity,
            request.relation.as_deref(),
        )
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_string_pretty(&serde_json::json!({
        "success": true,
        "message": format!("Connected project '{}' to entity '{}'", request.project, request.entity),
    }))
    .map_err(|e| e.to_string())
}

// ----------------------------------------------------------------------------
// Task Management
// ----------------------------------------------------------------------------

/// Request to create a new task.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkTaskCreateRequest {
    /// Parent project name.
    #[schemars(description = "Parent project name")]
    pub project: String,
    /// Task name.
    #[schemars(description = "Task name")]
    pub name: String,
    /// Optional description.
    #[schemars(description = "Task description")]
    pub description: Option<String>,
    /// Optional JIRA key.
    #[schemars(description = "JIRA key (e.g., IDEAI-235)")]
    pub jira_key: Option<String>,
}

/// Response from task creation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkTaskResponse {
    /// Task ID.
    pub id: String,
    /// Task name.
    pub name: String,
    /// Project ID.
    pub project_id: String,
    /// Task description.
    pub description: Option<String>,
    /// Task status.
    pub status: String,
    /// Task priority.
    pub priority: String,
    /// JIRA key.
    pub jira_key: Option<String>,
}

/// Create a new task.
pub async fn work_task_create(
    state: &ToolState,
    request: WorkTaskCreateRequest,
) -> Result<String, String> {
    debug!(
        "work_task_create: project={}, name={}",
        request.project, request.name
    );

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    let task = service
        .create_task(
            &request.project,
            &request.name,
            request.description.as_deref(),
            request.jira_key.as_deref(),
        )
        .await
        .map_err(|e| e.to_string())?;

    let response = WorkTaskResponse {
        id: task.id.to_string(),
        name: task.name,
        project_id: task.project_id.to_string(),
        description: task.description,
        status: task.status.to_string(),
        priority: task.priority.to_string(),
        jira_key: task.jira_key,
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Request to get a task.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkTaskGetRequest {
    /// Task name or JIRA key.
    #[schemars(description = "Task name or JIRA key")]
    pub name_or_jira: String,
}

/// Get a task by name or JIRA key.
pub async fn work_task_get(
    state: &ToolState,
    request: WorkTaskGetRequest,
) -> Result<String, String> {
    debug!("work_task_get: name_or_jira={}", request.name_or_jira);

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    let task = service
        .get_task(&request.name_or_jira)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Task not found: {}", request.name_or_jira))?;

    // Get PRs for this task
    let project = service
        .get_project_by_id(&task.project_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Parent project not found".to_string())?;

    let prs = service
        .list_prs(&project.name, task.jira_key.as_deref())
        .await
        .unwrap_or_default();

    let entities = service
        .get_task_entities(&request.name_or_jira)
        .await
        .unwrap_or_default();

    let response = serde_json::json!({
        "id": task.id.to_string(),
        "name": task.name,
        "project": project.name,
        "description": task.description,
        "status": task.status.to_string(),
        "priority": task.priority.to_string(),
        "jira_key": task.jira_key,
        "blocked_by": task.blocked_by.iter().map(|id| id.to_string()).collect::<Vec<_>>(),
        "prs": prs.iter().map(|p| serde_json::json!({
            "url": p.url,
            "status": p.status.to_string(),
        })).collect::<Vec<_>>(),
        "entities": entities.iter().map(|e| e.name.clone()).collect::<Vec<_>>(),
    });

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Request to list tasks.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkTaskListRequest {
    /// Project name.
    #[schemars(description = "Project name")]
    pub project: String,
    /// Optional status filter.
    #[schemars(description = "Filter by status: todo, in_progress, blocked, done")]
    pub status: Option<String>,
}

/// List tasks for a project.
pub async fn work_task_list(
    state: &ToolState,
    request: WorkTaskListRequest,
) -> Result<String, String> {
    debug!(
        "work_task_list: project={}, status={:?}",
        request.project, request.status
    );

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    let status = request
        .status
        .map(|s| engram_core::work::TaskStatus::parse(&s));

    let tasks = service
        .list_tasks(&request.project, status)
        .await
        .map_err(|e| e.to_string())?;

    let response = serde_json::json!({
        "count": tasks.len(),
        "tasks": tasks.iter().map(|t| serde_json::json!({
            "id": t.id.to_string(),
            "name": t.name,
            "status": t.status.to_string(),
            "priority": t.priority.to_string(),
            "jira_key": t.jira_key,
        })).collect::<Vec<_>>(),
    });

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Request to update task status.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkTaskUpdateStatusRequest {
    /// Task name or JIRA key.
    #[schemars(description = "Task name or JIRA key")]
    pub name_or_jira: String,
    /// New status.
    #[schemars(description = "New status: todo, in_progress, blocked, done")]
    pub status: String,
}

/// Update task status.
pub async fn work_task_update_status(
    state: &ToolState,
    request: WorkTaskUpdateStatusRequest,
) -> Result<String, String> {
    debug!(
        "work_task_update_status: name_or_jira={}, status={}",
        request.name_or_jira, request.status
    );

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    let status = engram_core::work::TaskStatus::parse(&request.status);

    let task = service
        .update_task_status(&request.name_or_jira, status)
        .await
        .map_err(|e| e.to_string())?;

    let response = WorkTaskResponse {
        id: task.id.to_string(),
        name: task.name,
        project_id: task.project_id.to_string(),
        description: task.description,
        status: task.status.to_string(),
        priority: task.priority.to_string(),
        jira_key: task.jira_key,
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Request to connect task to entity.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkTaskConnectEntityRequest {
    /// Task name or JIRA key.
    #[schemars(description = "Task name or JIRA key")]
    pub task: String,
    /// Entity name.
    #[schemars(description = "Entity name to connect")]
    pub entity: String,
    /// Relation type.
    #[schemars(description = "Relation type: touches, modifies, creates")]
    pub relation: Option<String>,
}

/// Connect task to entity.
pub async fn work_task_connect_entity(
    state: &ToolState,
    request: WorkTaskConnectEntityRequest,
) -> Result<String, String> {
    debug!(
        "work_task_connect_entity: task={}, entity={}",
        request.task, request.entity
    );

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    service
        .connect_task_to_entity(&request.task, &request.entity, request.relation.as_deref())
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_string_pretty(&serde_json::json!({
        "success": true,
        "message": format!("Connected task '{}' to entity '{}'", request.task, request.entity),
    }))
    .map_err(|e| e.to_string())
}

// ----------------------------------------------------------------------------
// PR Management
// ----------------------------------------------------------------------------

/// Request to add a PR.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkPrAddRequest {
    /// Project name.
    #[schemars(description = "Project name")]
    pub project: String,
    /// PR URL.
    #[schemars(description = "PR URL (e.g., https://github.com/org/repo/pull/123)")]
    pub url: String,
    /// Optional task name or JIRA key.
    #[schemars(description = "Task name or JIRA key to link this PR to")]
    pub task: Option<String>,
    /// Optional PR title.
    #[schemars(description = "PR title")]
    pub title: Option<String>,
}

/// Response from PR addition.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkPrResponse {
    /// PR ID.
    pub id: String,
    /// PR URL.
    pub url: String,
    /// Repository name.
    pub repo: String,
    /// PR number.
    pub pr_number: u32,
    /// PR status.
    pub status: String,
}

/// Add a PR to a project.
pub async fn work_pr_add(state: &ToolState, request: WorkPrAddRequest) -> Result<String, String> {
    debug!(
        "work_pr_add: project={}, url={}",
        request.project, request.url
    );

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    let pr = service
        .add_pr(
            &request.project,
            request.task.as_deref(),
            &request.url,
            request.title.as_deref(),
        )
        .await
        .map_err(|e| e.to_string())?;

    let response = WorkPrResponse {
        id: pr.id.to_string(),
        url: pr.url,
        repo: pr.repo,
        pr_number: pr.pr_number,
        status: pr.status.to_string(),
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Request to list PRs.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkPrListRequest {
    /// Project name.
    #[schemars(description = "Project name")]
    pub project: String,
    /// Optional task filter.
    #[schemars(description = "Filter by task name or JIRA key")]
    pub task: Option<String>,
}

/// List PRs for a project.
pub async fn work_pr_list(state: &ToolState, request: WorkPrListRequest) -> Result<String, String> {
    debug!(
        "work_pr_list: project={}, task={:?}",
        request.project, request.task
    );

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    let prs = service
        .list_prs(&request.project, request.task.as_deref())
        .await
        .map_err(|e| e.to_string())?;

    let response = serde_json::json!({
        "count": prs.len(),
        "prs": prs.iter().map(|p| serde_json::json!({
            "id": p.id.to_string(),
            "url": p.url,
            "repo": p.repo,
            "pr_number": p.pr_number,
            "title": p.title,
            "status": p.status.to_string(),
        })).collect::<Vec<_>>(),
    });

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Request to update PR status.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkPrUpdateStatusRequest {
    /// PR URL.
    #[schemars(description = "PR URL")]
    pub url: String,
    /// New status.
    #[schemars(description = "New status: open, merged, closed")]
    pub status: String,
}

/// Update PR status.
pub async fn work_pr_update_status(
    state: &ToolState,
    request: WorkPrUpdateStatusRequest,
) -> Result<String, String> {
    debug!(
        "work_pr_update_status: url={}, status={}",
        request.url, request.status
    );

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    let status = engram_core::work::PrStatus::parse(&request.status);

    let pr = service
        .update_pr_status(&request.url, status)
        .await
        .map_err(|e| e.to_string())?;

    let response = WorkPrResponse {
        id: pr.id.to_string(),
        url: pr.url,
        repo: pr.repo,
        pr_number: pr.pr_number,
        status: pr.status.to_string(),
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

// ----------------------------------------------------------------------------
// Work Observations
// ----------------------------------------------------------------------------

/// Request to add a project observation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkProjectObserveRequest {
    /// Project name.
    #[schemars(description = "Project name")]
    pub project: String,
    /// Observation content.
    #[schemars(description = "Observation content")]
    pub content: String,
    /// Optional semantic key.
    #[schemars(description = "Semantic key (e.g., architecture.auth, decisions.api)")]
    pub key: Option<String>,
}

/// Add a project observation.
pub async fn work_project_observe(
    state: &ToolState,
    request: WorkProjectObserveRequest,
) -> Result<String, String> {
    debug!(
        "work_project_observe: project={}, key={:?}",
        request.project, request.key
    );

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    let obs = service
        .add_project_observation(&request.project, &request.content, request.key.as_deref())
        .await
        .map_err(|e| e.to_string())?;

    let response = serde_json::json!({
        "id": obs.id.to_string(),
        "project_id": obs.project_id.to_string(),
        "key": obs.key,
        "content": obs.content,
        "created_at": obs.created_at.format(&time::format_description::well_known::Rfc3339).unwrap(),
    });

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Request to add a task observation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkTaskObserveRequest {
    /// Task name or JIRA key.
    #[schemars(description = "Task name or JIRA key")]
    pub task: String,
    /// Observation content.
    #[schemars(description = "Observation content")]
    pub content: String,
    /// Optional semantic key.
    #[schemars(description = "Semantic key (e.g., gotchas.edge-case, decisions.approach)")]
    pub key: Option<String>,
}

/// Add a task observation.
pub async fn work_task_observe(
    state: &ToolState,
    request: WorkTaskObserveRequest,
) -> Result<String, String> {
    debug!(
        "work_task_observe: task={}, key={:?}",
        request.task, request.key
    );

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    let obs = service
        .add_task_observation(&request.task, &request.content, request.key.as_deref())
        .await
        .map_err(|e| e.to_string())?;

    let response = serde_json::json!({
        "id": obs.id.to_string(),
        "task_id": obs.task_id.to_string(),
        "key": obs.key,
        "content": obs.content,
        "created_at": obs.created_at.format(&time::format_description::well_known::Rfc3339).unwrap(),
    });

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

// ----------------------------------------------------------------------------
// Session Work Context
// ----------------------------------------------------------------------------

/// Request to join work context.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkJoinRequest {
    /// Coordination session ID.
    #[schemars(description = "Session ID from coord_register")]
    pub session_id: String,
    /// Project name to join.
    #[schemars(description = "Project name to join")]
    pub project: String,
    /// Optional task to join.
    #[schemars(description = "Task name or JIRA key to join")]
    pub task: Option<String>,
}

/// Join a work context.
pub async fn work_join(state: &ToolState, request: WorkJoinRequest) -> Result<String, String> {
    debug!(
        "work_join: session={}, project={}, task={:?}",
        request.session_id, request.project, request.task
    );

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    let session_id = engram_core::id::Id::parse(&request.session_id)
        .map_err(|e| format!("Invalid session ID: {}", e))?;

    let ctx = service
        .join_work(&session_id, &request.project, request.task.as_deref())
        .await
        .map_err(|e| e.to_string())?;

    // Get full context for the response
    let full_ctx = service
        .get_full_context(&request.project, request.task.as_deref())
        .await
        .map_err(|e| e.to_string())?;

    let response = serde_json::json!({
        "session_id": ctx.session_id.to_string(),
        "project": full_ctx.project.name,
        "task": full_ctx.task.as_ref().map(|t| &t.name),
        "project_observations": full_ctx.project_observations.len(),
        "task_observations": full_ctx.task_observations.len(),
        "prs": full_ctx.prs.len(),
        "connected_entities": full_ctx.connected_entities.iter().map(|e| e.name.clone()).collect::<Vec<_>>(),
    });

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Request to leave work context.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkLeaveRequest {
    /// Session ID.
    #[schemars(description = "Session ID from coord_register")]
    pub session_id: String,
}

/// Leave work context.
pub async fn work_leave(state: &ToolState, request: WorkLeaveRequest) -> Result<String, String> {
    debug!("work_leave: session={}", request.session_id);

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    let session_id = engram_core::id::Id::parse(&request.session_id)
        .map_err(|e| format!("Invalid session ID: {}", e))?;

    service
        .leave_work(&session_id)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_string_pretty(&serde_json::json!({
        "success": true,
        "message": "Left work context",
    }))
    .map_err(|e| e.to_string())
}

/// Request to get current work context.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkContextRequest {
    /// Session ID.
    #[schemars(description = "Session ID from coord_register")]
    pub session_id: String,
}

/// Get current work context.
pub async fn work_context(
    state: &ToolState,
    request: WorkContextRequest,
) -> Result<String, String> {
    debug!("work_context: session={}", request.session_id);

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    let session_id = engram_core::id::Id::parse(&request.session_id)
        .map_err(|e| format!("Invalid session ID: {}", e))?;

    let ctx = service
        .get_work_context(&session_id)
        .await
        .map_err(|e| e.to_string())?;

    match ctx {
        Some(c) => {
            // Get project and task names
            let project_name = if let Some(pid) = &c.project_id {
                service
                    .get_project_by_id(pid)
                    .await
                    .ok()
                    .flatten()
                    .map(|p| p.name)
            } else {
                None
            };

            let task_name = if let Some(tid) = &c.task_id {
                service
                    .get_task_by_id(tid)
                    .await
                    .ok()
                    .flatten()
                    .map(|t| t.name)
            } else {
                None
            };

            let response = serde_json::json!({
                "session_id": c.session_id.to_string(),
                "project_id": c.project_id.map(|id| id.to_string()),
                "project_name": project_name,
                "task_id": c.task_id.map(|id| id.to_string()),
                "task_name": task_name,
                "joined_at": c.joined_at.format(&time::format_description::well_known::Rfc3339).unwrap(),
            });
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        None => serde_json::to_string_pretty(&serde_json::json!({
            "session_id": request.session_id,
            "project_id": null,
            "task_id": null,
            "message": "Not currently in a work context",
        }))
        .map_err(|e| e.to_string()),
    }
}

/// Request to get full work context.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkGetContextRequest {
    /// Project name.
    #[schemars(description = "Project name")]
    pub project: String,
    /// Optional task name or JIRA key.
    #[schemars(description = "Task name or JIRA key")]
    pub task: Option<String>,
}

/// Get full work context with all details.
pub async fn work_get_context(
    state: &ToolState,
    request: WorkGetContextRequest,
) -> Result<String, String> {
    debug!(
        "work_get_context: project={}, task={:?}",
        request.project, request.task
    );

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    let ctx = service
        .get_full_context(&request.project, request.task.as_deref())
        .await
        .map_err(|e| e.to_string())?;

    let response = serde_json::json!({
        "project": {
            "id": ctx.project.id.to_string(),
            "name": ctx.project.name,
            "description": ctx.project.description,
            "status": ctx.project.status.to_string(),
        },
        "task": ctx.task.as_ref().map(|t| serde_json::json!({
            "id": t.id.to_string(),
            "name": t.name,
            "description": t.description,
            "status": t.status.to_string(),
            "priority": t.priority.to_string(),
            "jira_key": t.jira_key,
        })),
        "prs": ctx.prs.iter().map(|p| serde_json::json!({
            "url": p.url,
            "repo": p.repo,
            "status": p.status.to_string(),
            "title": p.title,
        })).collect::<Vec<_>>(),
        "project_observations": ctx.project_observations.iter().map(|o| serde_json::json!({
            "key": o.key,
            "content": o.content,
        })).collect::<Vec<_>>(),
        "task_observations": ctx.task_observations.iter().map(|o| serde_json::json!({
            "key": o.key,
            "content": o.content,
        })).collect::<Vec<_>>(),
        "connected_entities": ctx.connected_entities.iter().map(|e| serde_json::json!({
            "name": e.name,
            "type": e.entity_type.to_string(),
            "description": e.description,
        })).collect::<Vec<_>>(),
        "entity_observations": ctx.entity_observations.iter().map(|o| serde_json::json!({
            "key": o.key,
            "content": o.content,
        })).collect::<Vec<_>>(),
    });

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

// ----------------------------------------------------------------------------
// Additional Work Management Tools
// ----------------------------------------------------------------------------

/// Request to delete a project.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkProjectDeleteRequest {
    /// Project name.
    #[schemars(description = "Project name to delete")]
    pub name: String,
}

/// Delete a project.
pub async fn work_project_delete(
    state: &ToolState,
    request: WorkProjectDeleteRequest,
) -> Result<String, String> {
    debug!("work_project_delete: name={}", request.name);

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    service
        .delete_project(&request.name)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_string_pretty(&serde_json::json!({
        "success": true,
        "message": format!("Deleted project '{}'", request.name),
    }))
    .map_err(|e| e.to_string())
}

/// Request to delete a task.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkTaskDeleteRequest {
    /// Task name or JIRA key.
    #[schemars(description = "Task name or JIRA key to delete")]
    pub name_or_jira: String,
}

/// Delete a task.
pub async fn work_task_delete(
    state: &ToolState,
    request: WorkTaskDeleteRequest,
) -> Result<String, String> {
    debug!("work_task_delete: name_or_jira={}", request.name_or_jira);

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    service
        .delete_task(&request.name_or_jira)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_string_pretty(&serde_json::json!({
        "success": true,
        "message": format!("Deleted task '{}'", request.name_or_jira),
    }))
    .map_err(|e| e.to_string())
}

/// Request to delete a PR.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkPrDeleteRequest {
    /// PR URL.
    #[schemars(description = "PR URL to delete")]
    pub url: String,
}

/// Delete a PR.
pub async fn work_pr_delete(
    state: &ToolState,
    request: WorkPrDeleteRequest,
) -> Result<String, String> {
    debug!("work_pr_delete: url={}", request.url);

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    service
        .delete_pr(&request.url)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_string_pretty(&serde_json::json!({
        "success": true,
        "message": format!("Deleted PR '{}'", request.url),
    }))
    .map_err(|e| e.to_string())
}

/// Request to get a PR.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkPrGetRequest {
    /// PR URL.
    #[schemars(description = "PR URL to get")]
    pub url: String,
}

/// Get a PR by URL.
pub async fn work_pr_get(state: &ToolState, request: WorkPrGetRequest) -> Result<String, String> {
    debug!("work_pr_get: url={}", request.url);

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    let pr = service
        .get_pr(&request.url)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("PR not found: {}", request.url))?;

    let response = WorkPrResponse {
        id: pr.id.to_string(),
        url: pr.url,
        repo: pr.repo,
        pr_number: pr.pr_number,
        status: pr.status.to_string(),
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

/// Request to disconnect project from entity.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkProjectDisconnectEntityRequest {
    /// Project name.
    #[schemars(description = "Project name")]
    pub project: String,
    /// Entity name.
    #[schemars(description = "Entity name to disconnect")]
    pub entity: String,
}

/// Disconnect project from entity.
pub async fn work_project_disconnect_entity(
    state: &ToolState,
    request: WorkProjectDisconnectEntityRequest,
) -> Result<String, String> {
    debug!(
        "work_project_disconnect_entity: project={}, entity={}",
        request.project, request.entity
    );

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    service
        .disconnect_project_from_entity(&request.project, &request.entity)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_string_pretty(&serde_json::json!({
        "success": true,
        "message": format!("Disconnected project '{}' from entity '{}'", request.project, request.entity),
    }))
    .map_err(|e| e.to_string())
}

/// Request to disconnect task from entity.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkTaskDisconnectEntityRequest {
    /// Task name or JIRA key.
    #[schemars(description = "Task name or JIRA key")]
    pub task: String,
    /// Entity name.
    #[schemars(description = "Entity name to disconnect")]
    pub entity: String,
}

/// Disconnect task from entity.
pub async fn work_task_disconnect_entity(
    state: &ToolState,
    request: WorkTaskDisconnectEntityRequest,
) -> Result<String, String> {
    debug!(
        "work_task_disconnect_entity: task={}, entity={}",
        request.task, request.entity
    );

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    service
        .disconnect_task_from_entity(&request.task, &request.entity)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::to_string_pretty(&serde_json::json!({
        "success": true,
        "message": format!("Disconnected task '{}' from entity '{}'", request.task, request.entity),
    }))
    .map_err(|e| e.to_string())
}

/// Request to get a work observation by key.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkObserveGetRequest {
    /// Project name (required if task is not specified).
    #[schemars(description = "Project name")]
    pub project: Option<String>,
    /// Task name or JIRA key (overrides project scope).
    #[schemars(
        description = "Task name or JIRA key (if specified, gets task observation instead of project)"
    )]
    pub task: Option<String>,
    /// Observation key.
    #[schemars(description = "Observation key to get")]
    pub key: String,
}

/// Get a work observation by key.
pub async fn work_observe_get(
    state: &ToolState,
    request: WorkObserveGetRequest,
) -> Result<String, String> {
    debug!(
        "work_observe_get: project={:?}, task={:?}, key={}",
        request.project, request.task, request.key
    );

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    // Task scope takes precedence
    if let Some(task) = &request.task {
        let obs = service
            .get_task_observation_by_key(task, &request.key)
            .await
            .map_err(|e| e.to_string())?;

        match obs {
            Some(o) => {
                let response = serde_json::json!({
                    "id": o.id.to_string(),
                    "task_id": o.task_id.to_string(),
                    "key": o.key,
                    "content": o.content,
                    "created_at": o.created_at.format(&time::format_description::well_known::Rfc3339).unwrap(),
                });
                serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
            }
            None => serde_json::to_string_pretty(&serde_json::json!({
                "found": false,
                "message": format!("No observation found for task '{}' with key '{}'", task, request.key),
            }))
            .map_err(|e| e.to_string()),
        }
    } else if let Some(project) = &request.project {
        let obs = service
            .get_project_observation_by_key(project, &request.key)
            .await
            .map_err(|e| e.to_string())?;

        match obs {
            Some(o) => {
                let response = serde_json::json!({
                    "id": o.id.to_string(),
                    "project_id": o.project_id.to_string(),
                    "key": o.key,
                    "content": o.content,
                    "created_at": o.created_at.format(&time::format_description::well_known::Rfc3339).unwrap(),
                });
                serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
            }
            None => serde_json::to_string_pretty(&serde_json::json!({
                "found": false,
                "message": format!("No observation found for project '{}' with key '{}'", project, request.key),
            }))
            .map_err(|e| e.to_string()),
        }
    } else {
        Err("Either 'project' or 'task' must be specified".to_string())
    }
}

/// Request to list work observations.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkObserveListRequest {
    /// Project name (required if task is not specified).
    #[schemars(description = "Project name")]
    pub project: Option<String>,
    /// Task name or JIRA key (overrides project scope).
    #[schemars(
        description = "Task name or JIRA key (if specified, lists task observations instead of project)"
    )]
    pub task: Option<String>,
    /// Optional key pattern to filter (e.g., 'architecture.*').
    #[schemars(description = "Key pattern for filtering (e.g., 'architecture.*')")]
    pub key_pattern: Option<String>,
    /// Maximum number of results.
    #[schemars(description = "Maximum results to return (default: 50)")]
    pub limit: Option<usize>,
}

/// List work observations.
pub async fn work_observe_list(
    state: &ToolState,
    request: WorkObserveListRequest,
) -> Result<String, String> {
    debug!(
        "work_observe_list: project={:?}, task={:?}, key_pattern={:?}",
        request.project, request.task, request.key_pattern
    );

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    let limit = request.limit.unwrap_or(50);

    // Task scope takes precedence
    if let Some(task) = &request.task {
        let observations = service
            .get_task_observations(task)
            .await
            .map_err(|e| e.to_string())?;

        // Filter by key pattern if specified
        let filtered: Vec<_> = if let Some(pattern) = &request.key_pattern {
            observations
                .into_iter()
                .filter(|o| {
                    o.key.as_ref().map_or(false, |k| {
                        if pattern.ends_with('*') {
                            k.starts_with(&pattern[..pattern.len() - 1])
                        } else {
                            k == pattern
                        }
                    })
                })
                .take(limit)
                .collect()
        } else {
            observations.into_iter().take(limit).collect()
        };

        let response = serde_json::json!({
            "count": filtered.len(),
            "observations": filtered.iter().map(|o| serde_json::json!({
                "id": o.id.to_string(),
                "key": o.key,
                "content": o.content,
                "created_at": o.created_at.format(&time::format_description::well_known::Rfc3339).unwrap(),
            })).collect::<Vec<_>>(),
        });
        serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
    } else if let Some(project) = &request.project {
        let observations = service
            .get_project_observations(project)
            .await
            .map_err(|e| e.to_string())?;

        // Filter by key pattern if specified
        let filtered: Vec<_> = if let Some(pattern) = &request.key_pattern {
            observations
                .into_iter()
                .filter(|o| {
                    o.key.as_ref().map_or(false, |k| {
                        if pattern.ends_with('*') {
                            k.starts_with(&pattern[..pattern.len() - 1])
                        } else {
                            k == pattern
                        }
                    })
                })
                .take(limit)
                .collect()
        } else {
            observations.into_iter().take(limit).collect()
        };

        let response = serde_json::json!({
            "count": filtered.len(),
            "observations": filtered.iter().map(|o| serde_json::json!({
                "id": o.id.to_string(),
                "key": o.key,
                "content": o.content,
                "created_at": o.created_at.format(&time::format_description::well_known::Rfc3339).unwrap(),
            })).collect::<Vec<_>>(),
        });
        serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
    } else {
        Err("Either 'project' or 'task' must be specified".to_string())
    }
}

/// Request to delete a work observation.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkObserveDeleteRequest {
    /// Project name (required if task is not specified).
    #[schemars(description = "Project name")]
    pub project: Option<String>,
    /// Task name or JIRA key (overrides project scope).
    #[schemars(
        description = "Task name or JIRA key (if specified, deletes task observation instead of project)"
    )]
    pub task: Option<String>,
    /// Observation key to delete.
    #[schemars(description = "Observation key to delete")]
    pub key: String,
}

/// Delete a work observation.
pub async fn work_observe_delete(
    state: &ToolState,
    request: WorkObserveDeleteRequest,
) -> Result<String, String> {
    debug!(
        "work_observe_delete: project={:?}, task={:?}, key={}",
        request.project, request.task, request.key
    );

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    // Task scope takes precedence
    if let Some(task) = &request.task {
        service
            .delete_task_observation_by_key(task, &request.key)
            .await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "success": true,
            "message": format!("Deleted task observation for '{}' (key: {})", task, request.key),
        }))
        .map_err(|e| e.to_string())
    } else if let Some(project) = &request.project {
        service
            .delete_project_observation_by_key(project, &request.key)
            .await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "success": true,
            "message": format!("Deleted project observation for '{}' (key: {})", project, request.key),
        }))
        .map_err(|e| e.to_string())
    } else {
        Err("Either 'project' or 'task' must be specified".to_string())
    }
}

/// Request for work statistics.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkStatsRequest {}

/// Response with work statistics.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkStatsResponse {
    /// Number of projects.
    pub project_count: u64,
    /// Number of tasks.
    pub task_count: u64,
    /// Number of PRs.
    pub pr_count: u64,
    /// Number of project observations.
    pub project_observation_count: u64,
    /// Number of task observations.
    pub task_observation_count: u64,
}

/// Get work statistics.
pub async fn work_stats(state: &ToolState, _request: WorkStatsRequest) -> Result<String, String> {
    debug!("work_stats");

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    let stats = service.stats().await.map_err(|e| e.to_string())?;

    let response = WorkStatsResponse {
        project_count: stats.project_count,
        task_count: stats.task_count,
        pr_count: stats.pr_count,
        project_observation_count: stats.project_observation_count,
        task_observation_count: stats.task_observation_count,
    };

    serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
}

// =============================================================================
// Layer 7: Consolidated Work Management Tools (Action-Based API)
// =============================================================================

/// Unified request for project operations.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkProjectRequest {
    /// Action: create, get, list, update, delete, connect_entity, disconnect_entity, entities
    #[schemars(
        description = "Action to perform: create, get, list, update, delete, connect_entity, disconnect_entity, entities"
    )]
    pub action: String,
    /// Project name (required for most actions).
    #[schemars(description = "Project name")]
    pub name: Option<String>,
    /// Project description (for create).
    #[schemars(description = "Project description (for create action)")]
    pub description: Option<String>,
    /// Status filter or new status (for list/update).
    #[schemars(
        description = "Status: planning, active, completed, archived (for list filter or update)"
    )]
    pub status: Option<String>,
    /// Entity name (for connect_entity/disconnect_entity).
    #[schemars(description = "Entity name (for connect_entity/disconnect_entity)")]
    pub entity: Option<String>,
    /// Relation type (for connect_entity).
    #[schemars(description = "Relation type: involves, depends_on, produces (for connect_entity)")]
    pub relation: Option<String>,
}

/// Unified project management handler.
pub async fn work_project(
    state: &ToolState,
    request: WorkProjectRequest,
) -> Result<String, String> {
    debug!("work_project: action={}", request.action);

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    match request.action.as_str() {
        "create" => {
            let name = request.name.ok_or("'name' required for create")?;
            let project = service
                .create_project(&name, request.description.as_deref())
                .await
                .map_err(|e| e.to_string())?;

            let response = WorkProjectResponse {
                id: project.id.to_string(),
                name: project.name,
                description: project.description,
                status: project.status.to_string(),
                created_at: project
                    .created_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "get" => {
            let name = request.name.ok_or("'name' required for get")?;
            let project = service
                .get_project(&name)
                .await
                .map_err(|e| e.to_string())?
                .ok_or_else(|| format!("Project not found: {}", name))?;

            let tasks = service
                .list_tasks(&name, None)
                .await
                .map_err(|e| e.to_string())?;
            let prs = service
                .list_prs(&name, None)
                .await
                .map_err(|e| e.to_string())?;
            let entities = service
                .get_project_entities(&name)
                .await
                .map_err(|e| e.to_string())?;

            let response = serde_json::json!({
                "id": project.id.to_string(),
                "name": project.name,
                "description": project.description,
                "status": project.status.to_string(),
                "created_at": project.created_at.format(&time::format_description::well_known::Rfc3339).unwrap(),
                "task_count": tasks.len(),
                "tasks": tasks.iter().map(|t| serde_json::json!({
                    "id": t.id.to_string(),
                    "name": t.name,
                    "status": t.status.to_string(),
                    "priority": t.priority.to_string(),
                    "jira_key": t.jira_key,
                })).collect::<Vec<_>>(),
                "pr_count": prs.len(),
                "prs": prs.iter().map(|p| serde_json::json!({
                    "url": p.url,
                    "repo": p.repo,
                    "status": p.status.to_string(),
                })).collect::<Vec<_>>(),
                "entity_count": entities.len(),
                "entities": entities.iter().map(|e| e.name.clone()).collect::<Vec<_>>(),
            });
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "list" => {
            let status = request
                .status
                .map(|s| engram_core::work::ProjectStatus::parse(&s));

            let projects = service
                .list_projects(status)
                .await
                .map_err(|e| e.to_string())?;

            let response = serde_json::json!({
                "count": projects.len(),
                "projects": projects.iter().map(|p| serde_json::json!({
                    "id": p.id.to_string(),
                    "name": p.name,
                    "description": p.description,
                    "status": p.status.to_string(),
                })).collect::<Vec<_>>(),
            });
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "update" => {
            let name = request.name.ok_or("'name' required for update")?;
            let status_str = request.status.ok_or("'status' required for update")?;
            let status = engram_core::work::ProjectStatus::parse(&status_str);

            let project = service
                .update_project_status(&name, status)
                .await
                .map_err(|e| e.to_string())?;

            let response = WorkProjectResponse {
                id: project.id.to_string(),
                name: project.name,
                description: project.description,
                status: project.status.to_string(),
                created_at: project
                    .created_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "delete" => {
            let name = request.name.ok_or("'name' required for delete")?;
            service
                .delete_project(&name)
                .await
                .map_err(|e| e.to_string())?;

            serde_json::to_string_pretty(&serde_json::json!({
                "success": true,
                "message": format!("Deleted project '{}'", name),
            }))
            .map_err(|e| e.to_string())
        }
        "connect_entity" => {
            let name = request.name.ok_or("'name' required for connect_entity")?;
            let entity = request.entity.ok_or("'entity' required for connect_entity")?;

            service
                .connect_project_to_entity(&name, &entity, request.relation.as_deref())
                .await
                .map_err(|e| e.to_string())?;

            serde_json::to_string_pretty(&serde_json::json!({
                "success": true,
                "message": format!("Connected project '{}' to entity '{}'", name, entity),
            }))
            .map_err(|e| e.to_string())
        }
        "disconnect_entity" => {
            let name = request.name.ok_or("'name' required for disconnect_entity")?;
            let entity = request.entity.ok_or("'entity' required for disconnect_entity")?;

            service
                .disconnect_project_from_entity(&name, &entity)
                .await
                .map_err(|e| e.to_string())?;

            serde_json::to_string_pretty(&serde_json::json!({
                "success": true,
                "message": format!("Disconnected project '{}' from entity '{}'", name, entity),
            }))
            .map_err(|e| e.to_string())
        }
        "entities" => {
            let name = request.name.ok_or("'name' required for entities")?;
            let entities = service
                .get_project_entities(&name)
                .await
                .map_err(|e| e.to_string())?;

            let response = serde_json::json!({
                "count": entities.len(),
                "entities": entities.iter().map(|e| serde_json::json!({
                    "name": e.name,
                    "type": e.entity_type.to_string(),
                    "description": e.description,
                })).collect::<Vec<_>>(),
            });
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        _ => Err(format!(
            "Unknown action: '{}'. Valid actions: create, get, list, update, delete, connect_entity, disconnect_entity, entities",
            request.action
        )),
    }
}

/// Unified request for task operations.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkTaskRequest {
    /// Action: create, get, list, update, delete, connect_entity, disconnect_entity, entities
    #[schemars(
        description = "Action to perform: create, get, list, update, delete, connect_entity, disconnect_entity, entities"
    )]
    pub action: String,
    /// Project name (required for create, list).
    #[schemars(description = "Project name (required for create, list)")]
    pub project: Option<String>,
    /// Task name or JIRA key (required for most actions).
    #[schemars(description = "Task name or JIRA key")]
    pub name: Option<String>,
    /// Task description (for create).
    #[schemars(description = "Task description (for create)")]
    pub description: Option<String>,
    /// JIRA key (for create).
    #[schemars(description = "JIRA key e.g. IDEAI-235 (for create)")]
    pub jira_key: Option<String>,
    /// Status filter or new status (for list/update).
    #[schemars(
        description = "Status: todo, in_progress, blocked, done (for list filter or update)"
    )]
    pub status: Option<String>,
    /// Entity name (for connect_entity/disconnect_entity).
    #[schemars(description = "Entity name (for connect_entity/disconnect_entity)")]
    pub entity: Option<String>,
    /// Relation type (for connect_entity).
    #[schemars(description = "Relation type: touches, modifies, creates (for connect_entity)")]
    pub relation: Option<String>,
}

/// Unified task management handler.
pub async fn work_task(state: &ToolState, request: WorkTaskRequest) -> Result<String, String> {
    debug!("work_task: action={}", request.action);

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    match request.action.as_str() {
        "create" => {
            let project = request.project.ok_or("'project' required for create")?;
            let name = request.name.ok_or("'name' required for create")?;

            let task = service
                .create_task(
                    &project,
                    &name,
                    request.description.as_deref(),
                    request.jira_key.as_deref(),
                )
                .await
                .map_err(|e| e.to_string())?;

            let response = WorkTaskResponse {
                id: task.id.to_string(),
                name: task.name,
                project_id: task.project_id.to_string(),
                description: task.description,
                status: task.status.to_string(),
                priority: task.priority.to_string(),
                jira_key: task.jira_key,
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "get" => {
            let name = request.name.ok_or("'name' required for get")?;

            let task = service
                .get_task(&name)
                .await
                .map_err(|e| e.to_string())?
                .ok_or_else(|| format!("Task not found: {}", name))?;

            let project = service
                .get_project_by_id(&task.project_id)
                .await
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "Parent project not found".to_string())?;

            let prs = service
                .list_prs(&project.name, task.jira_key.as_deref())
                .await
                .unwrap_or_default();

            let entities = service
                .get_task_entities(&name)
                .await
                .unwrap_or_default();

            let response = serde_json::json!({
                "id": task.id.to_string(),
                "name": task.name,
                "project": project.name,
                "description": task.description,
                "status": task.status.to_string(),
                "priority": task.priority.to_string(),
                "jira_key": task.jira_key,
                "blocked_by": task.blocked_by.iter().map(|id| id.to_string()).collect::<Vec<_>>(),
                "prs": prs.iter().map(|p| serde_json::json!({
                    "url": p.url,
                    "status": p.status.to_string(),
                })).collect::<Vec<_>>(),
                "entities": entities.iter().map(|e| e.name.clone()).collect::<Vec<_>>(),
            });
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "list" => {
            let project = request.project.ok_or("'project' required for list")?;
            let status = request
                .status
                .map(|s| engram_core::work::TaskStatus::parse(&s));

            let tasks = service
                .list_tasks(&project, status)
                .await
                .map_err(|e| e.to_string())?;

            let response = serde_json::json!({
                "count": tasks.len(),
                "tasks": tasks.iter().map(|t| serde_json::json!({
                    "id": t.id.to_string(),
                    "name": t.name,
                    "status": t.status.to_string(),
                    "priority": t.priority.to_string(),
                    "jira_key": t.jira_key,
                })).collect::<Vec<_>>(),
            });
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "update" => {
            let name = request.name.ok_or("'name' required for update")?;
            let status_str = request.status.ok_or("'status' required for update")?;
            let status = engram_core::work::TaskStatus::parse(&status_str);

            let task = service
                .update_task_status(&name, status)
                .await
                .map_err(|e| e.to_string())?;

            let response = WorkTaskResponse {
                id: task.id.to_string(),
                name: task.name,
                project_id: task.project_id.to_string(),
                description: task.description,
                status: task.status.to_string(),
                priority: task.priority.to_string(),
                jira_key: task.jira_key,
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "delete" => {
            let name = request.name.ok_or("'name' required for delete")?;

            service
                .delete_task(&name)
                .await
                .map_err(|e| e.to_string())?;

            serde_json::to_string_pretty(&serde_json::json!({
                "success": true,
                "message": format!("Deleted task '{}'", name),
            }))
            .map_err(|e| e.to_string())
        }
        "connect_entity" => {
            let name = request.name.ok_or("'name' required for connect_entity")?;
            let entity = request.entity.ok_or("'entity' required for connect_entity")?;

            service
                .connect_task_to_entity(&name, &entity, request.relation.as_deref())
                .await
                .map_err(|e| e.to_string())?;

            serde_json::to_string_pretty(&serde_json::json!({
                "success": true,
                "message": format!("Connected task '{}' to entity '{}'", name, entity),
            }))
            .map_err(|e| e.to_string())
        }
        "disconnect_entity" => {
            let name = request.name.ok_or("'name' required for disconnect_entity")?;
            let entity = request.entity.ok_or("'entity' required for disconnect_entity")?;

            service
                .disconnect_task_from_entity(&name, &entity)
                .await
                .map_err(|e| e.to_string())?;

            serde_json::to_string_pretty(&serde_json::json!({
                "success": true,
                "message": format!("Disconnected task '{}' from entity '{}'", name, entity),
            }))
            .map_err(|e| e.to_string())
        }
        "entities" => {
            let name = request.name.ok_or("'name' required for entities")?;
            let entities = service
                .get_task_entities(&name)
                .await
                .map_err(|e| e.to_string())?;

            let response = serde_json::json!({
                "count": entities.len(),
                "entities": entities.iter().map(|e| serde_json::json!({
                    "name": e.name,
                    "type": e.entity_type.to_string(),
                    "description": e.description,
                })).collect::<Vec<_>>(),
            });
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        _ => Err(format!(
            "Unknown action: '{}'. Valid actions: create, get, list, update, delete, connect_entity, disconnect_entity, entities",
            request.action
        )),
    }
}

/// Unified request for PR operations.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkPrRequest {
    /// Action: add, get, list, update, delete
    #[schemars(description = "Action to perform: add, get, list, update, delete")]
    pub action: String,
    /// Project name (required for add, list).
    #[schemars(description = "Project name (required for add, list)")]
    pub project: Option<String>,
    /// PR URL (required for add, get, update, delete).
    #[schemars(description = "PR URL e.g. https://github.com/org/repo/pull/123")]
    pub url: Option<String>,
    /// Task name or JIRA key (optional for add, filter for list).
    #[schemars(description = "Task name or JIRA key to link PR to")]
    pub task: Option<String>,
    /// PR title (for add).
    #[schemars(description = "PR title (for add)")]
    pub title: Option<String>,
    /// New status (for update).
    #[schemars(description = "Status: open, merged, closed (for update)")]
    pub status: Option<String>,
}

/// Unified PR management handler.
pub async fn work_pr(state: &ToolState, request: WorkPrRequest) -> Result<String, String> {
    debug!("work_pr: action={}", request.action);

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    match request.action.as_str() {
        "add" => {
            let project = request.project.ok_or("'project' required for add")?;
            let url = request.url.ok_or("'url' required for add")?;

            let pr = service
                .add_pr(
                    &project,
                    request.task.as_deref(),
                    &url,
                    request.title.as_deref(),
                )
                .await
                .map_err(|e| e.to_string())?;

            let response = WorkPrResponse {
                id: pr.id.to_string(),
                url: pr.url,
                repo: pr.repo,
                pr_number: pr.pr_number,
                status: pr.status.to_string(),
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "get" => {
            let url = request.url.ok_or("'url' required for get")?;

            let pr = service
                .get_pr(&url)
                .await
                .map_err(|e| e.to_string())?
                .ok_or_else(|| format!("PR not found: {}", url))?;

            let response = WorkPrResponse {
                id: pr.id.to_string(),
                url: pr.url,
                repo: pr.repo,
                pr_number: pr.pr_number,
                status: pr.status.to_string(),
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "list" => {
            let project = request.project.ok_or("'project' required for list")?;

            let prs = service
                .list_prs(&project, request.task.as_deref())
                .await
                .map_err(|e| e.to_string())?;

            let response = serde_json::json!({
                "count": prs.len(),
                "prs": prs.iter().map(|p| serde_json::json!({
                    "id": p.id.to_string(),
                    "url": p.url,
                    "repo": p.repo,
                    "pr_number": p.pr_number,
                    "title": p.title,
                    "status": p.status.to_string(),
                })).collect::<Vec<_>>(),
            });
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "update" => {
            let url = request.url.ok_or("'url' required for update")?;
            let status_str = request.status.ok_or("'status' required for update")?;
            let status = engram_core::work::PrStatus::parse(&status_str);

            let pr = service
                .update_pr_status(&url, status)
                .await
                .map_err(|e| e.to_string())?;

            let response = WorkPrResponse {
                id: pr.id.to_string(),
                url: pr.url,
                repo: pr.repo,
                pr_number: pr.pr_number,
                status: pr.status.to_string(),
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "delete" => {
            let url = request.url.ok_or("'url' required for delete")?;

            service.delete_pr(&url).await.map_err(|e| e.to_string())?;

            serde_json::to_string_pretty(&serde_json::json!({
                "success": true,
                "message": format!("Deleted PR '{}'", url),
            }))
            .map_err(|e| e.to_string())
        }
        _ => Err(format!(
            "Unknown action: '{}'. Valid actions: add, get, list, update, delete",
            request.action
        )),
    }
}

/// Unified request for work observations.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkObserveRequest {
    /// Action: add, get, list, delete
    #[schemars(description = "Action to perform: add, get, list, delete")]
    pub action: String,
    /// Project name (scope - one of project/task required).
    #[schemars(description = "Project name (scope - use project OR task)")]
    pub project: Option<String>,
    /// Task name or JIRA key (scope - takes precedence over project).
    #[schemars(description = "Task name or JIRA key (takes precedence over project)")]
    pub task: Option<String>,
    /// Observation content (for add).
    #[schemars(description = "Observation content (for add)")]
    pub content: Option<String>,
    /// Semantic key (for add, get, delete).
    #[schemars(description = "Semantic key e.g. architecture.auth, decisions.api")]
    pub key: Option<String>,
    /// Key pattern for filtering (for list).
    #[schemars(description = "Key pattern for filtering e.g. 'architecture.*' (for list)")]
    pub key_pattern: Option<String>,
    /// Max results (for list).
    #[schemars(description = "Maximum results (default: 50, for list)")]
    pub limit: Option<usize>,
}

/// Unified work observation handler.
pub async fn work_observe(
    state: &ToolState,
    request: WorkObserveRequest,
) -> Result<String, String> {
    debug!("work_observe: action={}", request.action);

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    match request.action.as_str() {
        "add" => {
            let content = request.content.ok_or("'content' required for add")?;

            // Task scope takes precedence
            if let Some(task) = &request.task {
                let obs = service
                    .add_task_observation(task, &content, request.key.as_deref())
                    .await
                    .map_err(|e| e.to_string())?;

                let response = serde_json::json!({
                    "id": obs.id.to_string(),
                    "task_id": obs.task_id.to_string(),
                    "key": obs.key,
                    "content": obs.content,
                    "created_at": obs.created_at.format(&time::format_description::well_known::Rfc3339).unwrap(),
                });
                serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
            } else if let Some(project) = &request.project {
                let obs = service
                    .add_project_observation(project, &content, request.key.as_deref())
                    .await
                    .map_err(|e| e.to_string())?;

                let response = serde_json::json!({
                    "id": obs.id.to_string(),
                    "project_id": obs.project_id.to_string(),
                    "key": obs.key,
                    "content": obs.content,
                    "created_at": obs.created_at.format(&time::format_description::well_known::Rfc3339).unwrap(),
                });
                serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
            } else {
                Err("Either 'project' or 'task' must be specified".to_string())
            }
        }
        "get" => {
            let key = request.key.ok_or("'key' required for get")?;

            // Task scope takes precedence
            if let Some(task) = &request.task {
                let obs = service
                    .get_task_observation_by_key(task, &key)
                    .await
                    .map_err(|e| e.to_string())?;

                match obs {
                    Some(o) => {
                        let response = serde_json::json!({
                            "id": o.id.to_string(),
                            "task_id": o.task_id.to_string(),
                            "key": o.key,
                            "content": o.content,
                            "created_at": o.created_at.format(&time::format_description::well_known::Rfc3339).unwrap(),
                        });
                        serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
                    }
                    None => serde_json::to_string_pretty(&serde_json::json!({
                        "found": false,
                        "message": format!("No observation found for task '{}' with key '{}'", task, key),
                    }))
                    .map_err(|e| e.to_string()),
                }
            } else if let Some(project) = &request.project {
                let obs = service
                    .get_project_observation_by_key(project, &key)
                    .await
                    .map_err(|e| e.to_string())?;

                match obs {
                    Some(o) => {
                        let response = serde_json::json!({
                            "id": o.id.to_string(),
                            "project_id": o.project_id.to_string(),
                            "key": o.key,
                            "content": o.content,
                            "created_at": o.created_at.format(&time::format_description::well_known::Rfc3339).unwrap(),
                        });
                        serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
                    }
                    None => serde_json::to_string_pretty(&serde_json::json!({
                        "found": false,
                        "message": format!("No observation found for project '{}' with key '{}'", project, key),
                    }))
                    .map_err(|e| e.to_string()),
                }
            } else {
                Err("Either 'project' or 'task' must be specified".to_string())
            }
        }
        "list" => {
            let limit = request.limit.unwrap_or(50);

            // Task scope takes precedence
            if let Some(task) = &request.task {
                let observations = service
                    .get_task_observations(task)
                    .await
                    .map_err(|e| e.to_string())?;

                let filtered: Vec<_> = if let Some(pattern) = &request.key_pattern {
                    observations
                        .into_iter()
                        .filter(|o| {
                            o.key.as_ref().map_or(false, |k| {
                                if pattern.ends_with('*') {
                                    k.starts_with(&pattern[..pattern.len() - 1])
                                } else {
                                    k == pattern
                                }
                            })
                        })
                        .take(limit)
                        .collect()
                } else {
                    observations.into_iter().take(limit).collect()
                };

                let response = serde_json::json!({
                    "count": filtered.len(),
                    "observations": filtered.iter().map(|o| serde_json::json!({
                        "id": o.id.to_string(),
                        "key": o.key,
                        "content": o.content,
                        "created_at": o.created_at.format(&time::format_description::well_known::Rfc3339).unwrap(),
                    })).collect::<Vec<_>>(),
                });
                serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
            } else if let Some(project) = &request.project {
                let observations = service
                    .get_project_observations(project)
                    .await
                    .map_err(|e| e.to_string())?;

                let filtered: Vec<_> = if let Some(pattern) = &request.key_pattern {
                    observations
                        .into_iter()
                        .filter(|o| {
                            o.key.as_ref().map_or(false, |k| {
                                if pattern.ends_with('*') {
                                    k.starts_with(&pattern[..pattern.len() - 1])
                                } else {
                                    k == pattern
                                }
                            })
                        })
                        .take(limit)
                        .collect()
                } else {
                    observations.into_iter().take(limit).collect()
                };

                let response = serde_json::json!({
                    "count": filtered.len(),
                    "observations": filtered.iter().map(|o| serde_json::json!({
                        "id": o.id.to_string(),
                        "key": o.key,
                        "content": o.content,
                        "created_at": o.created_at.format(&time::format_description::well_known::Rfc3339).unwrap(),
                    })).collect::<Vec<_>>(),
                });
                serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
            } else {
                Err("Either 'project' or 'task' must be specified".to_string())
            }
        }
        "delete" => {
            let key = request.key.ok_or("'key' required for delete")?;

            // Task scope takes precedence
            if let Some(task) = &request.task {
                service
                    .delete_task_observation_by_key(task, &key)
                    .await
                    .map_err(|e| e.to_string())?;

                serde_json::to_string_pretty(&serde_json::json!({
                    "success": true,
                    "message": format!("Deleted task observation for '{}' (key: {})", task, key),
                }))
                .map_err(|e| e.to_string())
            } else if let Some(project) = &request.project {
                service
                    .delete_project_observation_by_key(project, &key)
                    .await
                    .map_err(|e| e.to_string())?;

                serde_json::to_string_pretty(&serde_json::json!({
                    "success": true,
                    "message": format!("Deleted project observation for '{}' (key: {})", project, key),
                }))
                .map_err(|e| e.to_string())
            } else {
                Err("Either 'project' or 'task' must be specified".to_string())
            }
        }
        _ => Err(format!(
            "Unknown action: '{}'. Valid actions: add, get, list, delete",
            request.action
        )),
    }
}

/// Unified request for work context operations (merges work_context + work_get_context).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkContextRequestNew {
    /// Session ID (for session-based context lookup).
    #[schemars(description = "Session ID from coord_register (for session-based lookup)")]
    pub session_id: Option<String>,
    /// Project name (for direct context lookup - returns full context).
    #[schemars(description = "Project name (for direct full context lookup)")]
    pub project: Option<String>,
    /// Task name or JIRA key (for direct context lookup).
    #[schemars(description = "Task name or JIRA key (for direct context lookup)")]
    pub task: Option<String>,
}

/// Unified work context handler (merges work_context and work_get_context).
pub async fn work_context_new(
    state: &ToolState,
    request: WorkContextRequestNew,
) -> Result<String, String> {
    debug!(
        "work_context: session_id={:?}, project={:?}",
        request.session_id, request.project
    );

    let service_guard = state.work_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Work service not initialized".to_string())?;

    // If project is provided, get direct full context
    if let Some(project) = &request.project {
        let ctx = service
            .get_full_context(project, request.task.as_deref())
            .await
            .map_err(|e| e.to_string())?;

        let response = serde_json::json!({
            "project": {
                "id": ctx.project.id.to_string(),
                "name": ctx.project.name,
                "description": ctx.project.description,
                "status": ctx.project.status.to_string(),
            },
            "task": ctx.task.as_ref().map(|t| serde_json::json!({
                "id": t.id.to_string(),
                "name": t.name,
                "description": t.description,
                "status": t.status.to_string(),
                "priority": t.priority.to_string(),
                "jira_key": t.jira_key,
            })),
            "prs": ctx.prs.iter().map(|p| serde_json::json!({
                "url": p.url,
                "repo": p.repo,
                "status": p.status.to_string(),
                "title": p.title,
            })).collect::<Vec<_>>(),
            "project_observations": ctx.project_observations.iter().map(|o| serde_json::json!({
                "key": o.key,
                "content": o.content,
            })).collect::<Vec<_>>(),
            "task_observations": ctx.task_observations.iter().map(|o| serde_json::json!({
                "key": o.key,
                "content": o.content,
            })).collect::<Vec<_>>(),
            "connected_entities": ctx.connected_entities.iter().map(|e| serde_json::json!({
                "name": e.name,
                "type": e.entity_type.to_string(),
                "description": e.description,
            })).collect::<Vec<_>>(),
            "entity_observations": ctx.entity_observations.iter().map(|o| serde_json::json!({
                "key": o.key,
                "content": o.content,
            })).collect::<Vec<_>>(),
        });

        return serde_json::to_string_pretty(&response).map_err(|e| e.to_string());
    }

    // If session_id is provided, get session-based context
    if let Some(session_id_str) = &request.session_id {
        let session_id = engram_core::id::Id::parse(session_id_str)
            .map_err(|e| format!("Invalid session ID: {}", e))?;

        let ctx = service
            .get_work_context(&session_id)
            .await
            .map_err(|e| e.to_string())?;

        match ctx {
            Some(c) => {
                // Get project and task names
                let project_name = if let Some(pid) = &c.project_id {
                    service
                        .get_project_by_id(pid)
                        .await
                        .ok()
                        .flatten()
                        .map(|p| p.name)
                } else {
                    None
                };

                let task_name = if let Some(tid) = &c.task_id {
                    service
                        .get_task_by_id(tid)
                        .await
                        .ok()
                        .flatten()
                        .map(|t| t.name)
                } else {
                    None
                };

                let response = serde_json::json!({
                    "session_id": c.session_id.to_string(),
                    "project_id": c.project_id.map(|id| id.to_string()),
                    "project_name": project_name,
                    "task_id": c.task_id.map(|id| id.to_string()),
                    "task_name": task_name,
                    "joined_at": c.joined_at.format(&time::format_description::well_known::Rfc3339).unwrap(),
                });
                serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
            }
            None => serde_json::to_string_pretty(&serde_json::json!({
                "session_id": session_id_str,
                "project_id": null,
                "task_id": null,
                "message": "Not currently in a work context",
            }))
            .map_err(|e| e.to_string()),
        }
    } else {
        Err("Either 'session_id' or 'project' must be specified".to_string())
    }
}

// =============================================================================
// Phase 2: Consolidated Entity Tools (Action-based API)
// =============================================================================

/// Consolidated entity request - replaces entity_create, entity_get, entity_list, entity_search, entity_relate, entity_alias
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityRequestNew {
    /// Action to perform: create, get, list, search, relate, alias, delete
    #[schemars(description = "Action: create, get, list, search, relate, alias, delete")]
    pub action: String,

    /// Entity name (required for most actions)
    #[schemars(description = "Entity name (required for create, get, relate, alias, delete)")]
    pub name: Option<String>,

    /// Entity type (for create action)
    #[schemars(
        description = "Entity type for create: repo, tool, concept, deployment, topic, workflow, person, team, service"
    )]
    pub entity_type: Option<String>,

    /// Entity description (for create action)
    #[schemars(description = "Entity description (for create)")]
    pub description: Option<String>,

    /// Search query (for search action)
    #[schemars(description = "Search query (for search action)")]
    pub query: Option<String>,

    /// Type filter (for list action)
    #[schemars(description = "Filter by entity type (for list)")]
    pub type_filter: Option<String>,

    /// Max results (for list, search)
    #[schemars(description = "Maximum results (for list, search)")]
    pub limit: Option<usize>,

    /// Target entity name (for relate action)
    #[schemars(description = "Target entity name (for relate)")]
    pub target: Option<String>,

    /// Relation type (for relate action)
    #[schemars(
        description = "Relation type for relate: depends_on, uses, deployed_via, owned_by, documents, related_to"
    )]
    pub relation: Option<String>,

    /// Alias to add (for alias action)
    #[schemars(description = "Alias to add (for alias action)")]
    pub alias: Option<String>,
}

/// Consolidated entity observation request - replaces entity_observe, entity_observe_get, entity_observe_list, entity_observe_search, entity_observe_history
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityObserveRequestNew {
    /// Action to perform: add, get, list, search, history
    #[schemars(description = "Action: add, get, list, search, history")]
    pub action: String,

    /// Entity name (required for all actions)
    #[schemars(description = "Entity name")]
    pub entity: Option<String>,

    /// Observation content (for add action)
    #[schemars(description = "Observation content (for add)")]
    pub content: Option<String>,

    /// Semantic key (for add, get, history)
    #[schemars(
        description = "Semantic key (for add, get, history). Categories: architecture, patterns, gotchas, decisions, dependencies, config, testing, performance, security"
    )]
    pub key: Option<String>,

    /// Source of observation (for add action)
    #[schemars(description = "Source of observation (for add, e.g., 'code-analysis')")]
    pub source: Option<String>,

    /// Key pattern filter (for list action)
    #[schemars(description = "Key pattern filter, e.g., 'architecture.*' (for list)")]
    pub key_pattern: Option<String>,

    /// Search query (for search action)
    #[schemars(description = "Search query (for search)")]
    pub query: Option<String>,

    /// Max results (for list, search)
    #[schemars(description = "Maximum results (for list, search)")]
    pub limit: Option<usize>,
}

/// Consolidated entity handler - manages entities with action-based API
pub async fn entity_new(state: &ToolState, request: EntityRequestNew) -> Result<String, String> {
    debug!("entity: action={}", request.action);

    let service_guard = state.entity_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Entity service not initialized".to_string())?;

    match request.action.to_lowercase().as_str() {
        "create" => {
            let name = request.name.ok_or("name required for create")?;
            let entity_type_str = request.entity_type.as_deref().unwrap_or("concept");
            let entity_type = parse_entity_type(entity_type_str)?;

            let entity = service
                .create_entity(&name, entity_type, request.description.as_deref())
                .await
                .map_err(|e| e.to_string())?;

            let response = EntityCreateResponse {
                id: entity.id.to_string(),
                name: entity.name,
                entity_type: entity.entity_type.to_string(),
                description: entity.description,
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "get" => {
            let name = request.name.ok_or("name required for get")?;

            let entity = service
                .resolve(&name)
                .await
                .map_err(|e| e.to_string())?
                .ok_or_else(|| format!("Entity not found: {}", name))?;

            // Get aliases
            let aliases = service
                .get_aliases(&entity.name)
                .await
                .map_err(|e| e.to_string())?;

            // Get outgoing relationships
            let related_from = service
                .get_related_from(&entity.id)
                .await
                .map_err(|e| e.to_string())?;

            let relationships_out: Vec<RelationshipInfo> = related_from
                .into_iter()
                .map(|(rel, target)| RelationshipInfo {
                    relation_type: rel.relation_type.to_string(),
                    entity_name: target.name,
                    entity_type: target.entity_type.to_string(),
                })
                .collect();

            // Get incoming relationships
            let related_to = service
                .get_related_to(&entity.id)
                .await
                .map_err(|e| e.to_string())?;

            let relationships_in: Vec<RelationshipInfo> = related_to
                .into_iter()
                .map(|(rel, source)| RelationshipInfo {
                    relation_type: rel.relation_type.to_string(),
                    entity_name: source.name,
                    entity_type: source.entity_type.to_string(),
                })
                .collect();

            // Get observations
            let observations = service
                .get_observations(&entity.name)
                .await
                .map_err(|e| e.to_string())?;

            let observation_infos: Vec<ObservationInfo> = observations
                .into_iter()
                .map(|o| ObservationInfo {
                    id: o.id.to_string(),
                    key: o.key,
                    content: o.content,
                    source: o.source,
                    created_at: o
                        .created_at
                        .format(&time::format_description::well_known::Rfc3339)
                        .unwrap(),
                    updated_at: o
                        .updated_at
                        .format(&time::format_description::well_known::Rfc3339)
                        .unwrap(),
                })
                .collect();

            let response = EntityGetResponse {
                entity: EntityInfo {
                    id: entity.id.to_string(),
                    name: entity.name,
                    entity_type: entity.entity_type.to_string(),
                    description: entity.description,
                },
                aliases,
                relationships_out,
                relationships_in,
                observations: observation_infos,
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "list" => {
            let type_filter = if let Some(tf) = &request.type_filter {
                Some(parse_entity_type(tf)?)
            } else {
                None
            };

            let entities = service
                .list_entities(type_filter.as_ref())
                .await
                .map_err(|e| e.to_string())?;

            let limit = request.limit.unwrap_or(100);
            let entity_infos: Vec<EntityInfo> = entities
                .into_iter()
                .take(limit)
                .map(|e| EntityInfo {
                    id: e.id.to_string(),
                    name: e.name,
                    entity_type: e.entity_type.to_string(),
                    description: e.description,
                })
                .collect();

            let count = entity_infos.len();
            let response = EntityListResponse {
                entities: entity_infos,
                count,
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "search" => {
            let query = request.query.ok_or("query required for search")?;

            let entities = service
                .search_entities(&query)
                .await
                .map_err(|e| e.to_string())?;

            let limit = request.limit.unwrap_or(50);
            let entity_infos: Vec<EntityInfo> = entities
                .into_iter()
                .take(limit)
                .map(|e| EntityInfo {
                    id: e.id.to_string(),
                    name: e.name,
                    entity_type: e.entity_type.to_string(),
                    description: e.description,
                })
                .collect();

            let count = entity_infos.len();
            let response = EntitySearchResponse {
                entities: entity_infos,
                count,
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "relate" => {
            let name = request.name.ok_or("name (source) required for relate")?;
            let target = request.target.ok_or("target required for relate")?;
            let relation_str = request.relation.ok_or("relation required for relate")?;
            let relation_type = parse_relation_type(&relation_str)?;

            let rel = service
                .relate(&name, relation_type, &target)
                .await
                .map_err(|e| e.to_string())?;

            let response = EntityRelateResponse {
                message: "Relationship created".to_string(),
                source: name,
                relation_type: rel.relation_type.to_string(),
                target,
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "alias" => {
            let name = request.name.ok_or("name required for alias")?;
            let alias = request.alias.ok_or("alias required for alias")?;

            service
                .add_alias(&name, &alias)
                .await
                .map_err(|e| e.to_string())?;

            serde_json::to_string_pretty(&serde_json::json!({
                "message": format!("Alias '{}' added for entity '{}'", alias, name)
            }))
            .map_err(|e| e.to_string())
        }
        "delete" => {
            let name = request.name.ok_or("name required for delete")?;

            // First resolve the entity to get its ID
            let entity = service
                .resolve(&name)
                .await
                .map_err(|e| e.to_string())?
                .ok_or_else(|| format!("Entity not found: {}", name))?;

            service
                .delete_entity(&entity.id)
                .await
                .map_err(|e| e.to_string())?;

            serde_json::to_string_pretty(&serde_json::json!({
                "message": format!("Entity '{}' deleted", name),
                "id": entity.id.to_string()
            }))
            .map_err(|e| e.to_string())
        }
        _ => Err(format!(
            "Unknown action: '{}'. Valid actions: create, get, list, search, relate, alias, delete",
            request.action
        )),
    }
}

/// Consolidated entity observation handler - manages observations with action-based API
pub async fn entity_observe_new(
    state: &ToolState,
    request: EntityObserveRequestNew,
) -> Result<String, String> {
    debug!("entity_observe: action={}", request.action);

    let service_guard = state.entity_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Entity service not initialized".to_string())?;

    let entity_name = request.entity.ok_or("entity required")?;

    match request.action.to_lowercase().as_str() {
        "add" => {
            let content = request.content.ok_or("content required for add")?;

            let (obs, previous) = service
                .add_observation(
                    &entity_name,
                    &content,
                    request.key.as_deref(),
                    request.source.as_deref(),
                )
                .await
                .map_err(|e| e.to_string())?;

            let action = if previous.is_some() {
                "updated"
            } else {
                "created"
            };

            let mut response = serde_json::json!({
                "message": format!("Observation {}", action),
                "action": action,
                "entity": entity_name,
                "key": obs.key,
                "content": obs.content,
                "source": obs.source
            });

            if let Some(prev) = previous {
                response["previous_content"] = serde_json::json!(prev.content);
            }

            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "get" => {
            let key = request.key.ok_or("key required for get")?;

            let obs = service
                .get_observation_by_key(&entity_name, &key)
                .await
                .map_err(|e| e.to_string())?;

            match obs {
                Some(o) => {
                    let info = ObservationInfo {
                        id: o.id.to_string(),
                        key: o.key,
                        content: o.content,
                        source: o.source,
                        created_at: o
                            .created_at
                            .format(&time::format_description::well_known::Rfc3339)
                            .unwrap(),
                        updated_at: o
                            .updated_at
                            .format(&time::format_description::well_known::Rfc3339)
                            .unwrap(),
                    };
                    serde_json::to_string_pretty(&info).map_err(|e| e.to_string())
                }
                None => serde_json::to_string_pretty(&serde_json::json!({
                    "error": format!("No observation found with key '{}' for entity '{}'", key, entity_name)
                }))
                .map_err(|e| e.to_string()),
            }
        }
        "list" => {
            let observations = service
                .list_observations_by_pattern(&entity_name, request.key_pattern.as_deref())
                .await
                .map_err(|e| e.to_string())?;

            let limit = request.limit.unwrap_or(50);
            let observation_infos: Vec<ObservationInfo> = observations
                .into_iter()
                .take(limit)
                .map(|o| ObservationInfo {
                    id: o.id.to_string(),
                    key: o.key,
                    content: o.content,
                    source: o.source,
                    created_at: o
                        .created_at
                        .format(&time::format_description::well_known::Rfc3339)
                        .unwrap(),
                    updated_at: o
                        .updated_at
                        .format(&time::format_description::well_known::Rfc3339)
                        .unwrap(),
                })
                .collect();

            serde_json::to_string_pretty(&serde_json::json!({
                "entity": entity_name,
                "pattern": request.key_pattern,
                "count": observation_infos.len(),
                "observations": observation_infos
            }))
            .map_err(|e| e.to_string())
        }
        "search" => {
            let query = request.query.ok_or("query required for search")?;
            let limit = request.limit.unwrap_or(20);

            let observations = service
                .search_observations(&entity_name, &query, limit)
                .await
                .map_err(|e| e.to_string())?;

            let results: Vec<ObservationInfo> = observations
                .into_iter()
                .map(|o| ObservationInfo {
                    id: o.id.to_string(),
                    key: o.key,
                    content: o.content,
                    source: o.source,
                    created_at: o
                        .created_at
                        .format(&time::format_description::well_known::Rfc3339)
                        .unwrap(),
                    updated_at: o
                        .updated_at
                        .format(&time::format_description::well_known::Rfc3339)
                        .unwrap(),
                })
                .collect();

            serde_json::to_string_pretty(&serde_json::json!({
                "entity": entity_name,
                "query": query,
                "count": results.len(),
                "results": results
            }))
            .map_err(|e| e.to_string())
        }
        "history" => {
            let key = request.key.ok_or("key required for history")?;

            let history = service
                .get_observation_history(&entity_name, &key)
                .await
                .map_err(|e| e.to_string())?;

            let archived: Vec<ArchivedObservationInfo> = history
                .into_iter()
                .map(|a| ArchivedObservationInfo {
                    content: a.content,
                    source: a.source,
                    created_at: a
                        .created_at
                        .format(&time::format_description::well_known::Rfc3339)
                        .unwrap(),
                    archived_at: a
                        .archived_at
                        .format(&time::format_description::well_known::Rfc3339)
                        .unwrap(),
                })
                .collect();

            // Also get the current observation
            let current = service
                .get_observation_by_key(&entity_name, &key)
                .await
                .map_err(|e| e.to_string())?;

            let current_info = current.map(|o| ObservationInfo {
                id: o.id.to_string(),
                key: o.key,
                content: o.content,
                source: o.source,
                created_at: o
                    .created_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
                updated_at: o
                    .updated_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            });

            serde_json::to_string_pretty(&serde_json::json!({
                "entity": entity_name,
                "key": key,
                "current": current_info,
                "history_count": archived.len(),
                "history": archived
            }))
            .map_err(|e| e.to_string())
        }
        _ => Err(format!(
            "Unknown action: '{}'. Valid actions: add, get, list, search, history",
            request.action
        )),
    }
}

/// Parse entity type from string
fn parse_entity_type(s: &str) -> Result<EntityType, String> {
    match s.to_lowercase().as_str() {
        "repo" => Ok(EntityType::Repo),
        "tool" => Ok(EntityType::Tool),
        "concept" => Ok(EntityType::Concept),
        "deployment" => Ok(EntityType::Deployment),
        "topic" => Ok(EntityType::Topic),
        "workflow" => Ok(EntityType::Workflow),
        "person" => Ok(EntityType::Person),
        "team" => Ok(EntityType::Team),
        "service" => Ok(EntityType::Service),
        _ => Err(format!(
            "Unknown entity type: '{}'. Valid types: repo, tool, concept, deployment, topic, workflow, person, team, service",
            s
        )),
    }
}

/// Parse relation type from string
fn parse_relation_type(s: &str) -> Result<RelationType, String> {
    match s.to_lowercase().as_str() {
        "depends_on" => Ok(RelationType::DependsOn),
        "uses" => Ok(RelationType::Uses),
        "deployed_via" => Ok(RelationType::DeployedVia),
        "owned_by" => Ok(RelationType::OwnedBy),
        "documents" => Ok(RelationType::Documents),
        "related_to" => Ok(RelationType::RelatedTo),
        _ => Err(format!(
            "Unknown relation type: '{}'. Valid types: depends_on, uses, deployed_via, owned_by, documents, related_to",
            s
        )),
    }
}

// =============================================================================
// Phase 3: Consolidated Coordination Tools (Action-based API)
// =============================================================================

/// Consolidated coordination request - replaces coord_register, coord_unregister, coord_heartbeat, coord_set_file, coord_set_components, coord_check_conflicts, coord_list
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoordRequestNew {
    /// Action to perform: register, unregister, heartbeat, set_file, set_components, check_conflicts, list
    #[schemars(
        description = "Action: register, unregister, heartbeat, set_file, set_components, check_conflicts, list"
    )]
    pub action: String,

    /// Session ID (required for most actions)
    #[schemars(description = "Session ID")]
    pub session_id: Option<String>,

    /// Agent type (for register)
    #[schemars(description = "Agent type like 'claude-code', 'cursor' (for register)")]
    pub agent: Option<String>,

    /// Project (for register, list filter)
    #[schemars(description = "Project name or working directory")]
    pub project: Option<String>,

    /// Goal (for register)
    #[schemars(description = "Session goal (for register)")]
    pub goal: Option<String>,

    /// Components (for register, set_components)
    #[schemars(description = "Components/modules being worked on")]
    #[serde(default)]
    pub components: Vec<String>,

    /// File path (for set_file)
    #[schemars(description = "File path being edited, or null to clear (for set_file)")]
    pub file: Option<String>,
}

/// Consolidated coordination handler - manages session coordination with action-based API
pub async fn coord_new(state: &ToolState, request: CoordRequestNew) -> Result<String, String> {
    debug!("coord: action={}", request.action);

    let service_guard = state.coordination_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Coordination service not initialized".to_string())?;

    match request.action.to_lowercase().as_str() {
        "register" => {
            let session_id_str = request.session_id.ok_or("session_id required for register")?;
            let agent = request.agent.ok_or("agent required for register")?;
            let project = request.project.ok_or("project required for register")?;
            let goal = request.goal.ok_or("goal required for register")?;

            let session_id = engram_core::id::Id::parse(&session_id_str)
                .map_err(|e| format!("Invalid session ID: {}", e))?;

            let session = if request.components.is_empty() {
                service
                    .register(&session_id, &agent, &project, &goal)
                    .await
                    .map_err(|e| e.to_string())?
            } else {
                service
                    .register_with_components(
                        &session_id,
                        &agent,
                        &project,
                        &goal,
                        request.components,
                    )
                    .await
                    .map_err(|e| e.to_string())?
            };

            let response = CoordRegisterResponse {
                session_id: session.session_id.to_string(),
                agent: session.agent,
                project: session.project,
                goal: session.goal,
                components: session.components,
                started_at: session
                    .started_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_default(),
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "unregister" => {
            let session_id_str = request
                .session_id
                .ok_or("session_id required for unregister")?;
            let session_id = engram_core::id::Id::parse(&session_id_str)
                .map_err(|e| format!("Invalid session ID: {}", e))?;

            service
                .unregister(&session_id)
                .await
                .map_err(|e| e.to_string())?;

            serde_json::to_string_pretty(&serde_json::json!({
                "message": format!("Session {} unregistered", session_id_str)
            }))
            .map_err(|e| e.to_string())
        }
        "heartbeat" => {
            let session_id_str = request
                .session_id
                .ok_or("session_id required for heartbeat")?;
            let session_id = engram_core::id::Id::parse(&session_id_str)
                .map_err(|e| format!("Invalid session ID: {}", e))?;

            service
                .heartbeat(&session_id)
                .await
                .map_err(|e| e.to_string())?;

            serde_json::to_string_pretty(&serde_json::json!({
                "message": "Heartbeat recorded",
                "session_id": session_id_str
            }))
            .map_err(|e| e.to_string())
        }
        "set_file" => {
            let session_id_str = request
                .session_id
                .ok_or("session_id required for set_file")?;
            let session_id = engram_core::id::Id::parse(&session_id_str)
                .map_err(|e| format!("Invalid session ID: {}", e))?;

            let conflicts = service
                .set_current_file(&session_id, request.file.as_deref())
                .await
                .map_err(|e| e.to_string())?;

            let response = CoordSetFileResponse {
                success: true,
                conflicts: conflicts
                    .into_iter()
                    .map(|c| ConflictInfoMcp {
                        other_session_id: c.other_session_id.to_string(),
                        other_agent: c.other_agent,
                        other_goal: c.other_goal,
                        overlapping_components: c.overlapping_components,
                        other_current_file: c.other_current_file,
                    })
                    .collect(),
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "set_components" => {
            let session_id_str = request
                .session_id
                .ok_or("session_id required for set_components")?;
            let session_id = engram_core::id::Id::parse(&session_id_str)
                .map_err(|e| format!("Invalid session ID: {}", e))?;

            let conflicts = service
                .set_components(&session_id, &request.components)
                .await
                .map_err(|e| e.to_string())?;

            let response = CoordSetComponentsResponse {
                success: true,
                conflicts: conflicts
                    .into_iter()
                    .map(|c| ConflictInfoMcp {
                        other_session_id: c.other_session_id.to_string(),
                        other_agent: c.other_agent,
                        other_goal: c.other_goal,
                        overlapping_components: c.overlapping_components,
                        other_current_file: c.other_current_file,
                    })
                    .collect(),
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "check_conflicts" => {
            let session_id_str = request
                .session_id
                .ok_or("session_id required for check_conflicts")?;
            let session_id = engram_core::id::Id::parse(&session_id_str)
                .map_err(|e| format!("Invalid session ID: {}", e))?;

            // Get component conflicts
            let component_conflicts = service
                .check_conflicts(&session_id)
                .await
                .map_err(|e| e.to_string())?;

            // Get file conflicts (need to get current file first)
            let session = service.get(&session_id).await.map_err(|e| e.to_string())?;

            let file_conflicts = if let Some(s) = &session {
                if let Some(file) = &s.current_file {
                    service
                        .check_file_conflicts(&session_id, file)
                        .await
                        .map_err(|e| e.to_string())?
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };

            let has_conflicts = !component_conflicts.is_empty() || !file_conflicts.is_empty();

            let response = CoordCheckConflictsResponse {
                has_conflicts,
                component_conflicts: component_conflicts
                    .into_iter()
                    .map(|c| ConflictInfoMcp {
                        other_session_id: c.other_session_id.to_string(),
                        other_agent: c.other_agent,
                        other_goal: c.other_goal,
                        overlapping_components: c.overlapping_components,
                        other_current_file: c.other_current_file,
                    })
                    .collect(),
                file_conflicts: file_conflicts
                    .into_iter()
                    .map(|c| ConflictInfoMcp {
                        other_session_id: c.other_session_id.to_string(),
                        other_agent: c.other_agent,
                        other_goal: c.other_goal,
                        overlapping_components: c.overlapping_components,
                        other_current_file: c.other_current_file,
                    })
                    .collect(),
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "list" => {
            let sessions = if let Some(project) = &request.project {
                service
                    .list_for_project(project)
                    .await
                    .map_err(|e| e.to_string())?
            } else {
                service.list_active().await.map_err(|e| e.to_string())?
            };

            let session_infos: Vec<ActiveSessionInfoMcp> = sessions
                .into_iter()
                .map(|s| ActiveSessionInfoMcp {
                    session_id: s.session_id.to_string(),
                    agent: s.agent,
                    project: s.project,
                    goal: s.goal,
                    components: s.components,
                    current_file: s.current_file,
                    started_at: s
                        .started_at
                        .format(&time::format_description::well_known::Rfc3339)
                        .unwrap_or_default(),
                    last_heartbeat: s
                        .last_heartbeat
                        .format(&time::format_description::well_known::Rfc3339)
                        .unwrap_or_default(),
                })
                .collect();

            let count = session_infos.len();
            let response = CoordListResponse {
                sessions: session_infos,
                count,
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        _ => Err(format!(
            "Unknown action: '{}'. Valid actions: register, unregister, heartbeat, set_file, set_components, check_conflicts, list",
            request.action
        )),
    }
}

// =============================================================================
// Phase 4: Consolidated Knowledge Management Tool
// =============================================================================

/// Consolidated request for all knowledge management operations.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeRequestNew {
    /// Action to perform: init, scan, register, import, list, duplicates, versions
    #[schemars(description = "Action: init, scan, register, import, list, duplicates, versions")]
    pub action: String,

    /// Path for scan (directory) or register/import (document file)
    #[schemars(description = "Path to directory (scan) or document file (register/import)")]
    pub path: Option<String>,

    /// Repository name for scan (default: 'default')
    #[schemars(description = "Repository name for tracking (scan only, default: 'default')")]
    pub repo_name: Option<String>,

    /// Document name for register/import
    #[schemars(description = "Human-friendly name for the document (register/import)")]
    pub name: Option<String>,

    /// Document type for register/import: adr, runbook, howto, research, design, readme, changelog
    #[schemars(
        description = "Document type: adr, runbook, howto, research, design, readme, changelog"
    )]
    pub doc_type: Option<String>,
}

/// Parse doc_type string to DocType enum
fn parse_doc_type(doc_type: &str) -> Result<DocType, String> {
    match doc_type.to_lowercase().as_str() {
        "adr" => Ok(DocType::Adr),
        "runbook" => Ok(DocType::Runbook),
        "howto" => Ok(DocType::Howto),
        "research" => Ok(DocType::Research),
        "design" => Ok(DocType::Design),
        "readme" => Ok(DocType::Readme),
        "changelog" => Ok(DocType::Changelog),
        _ => Err(format!(
            "Unknown doc_type: '{}'. Valid: adr, runbook, howto, research, design, readme, changelog",
            doc_type
        )),
    }
}

/// Consolidated knowledge management handler.
pub async fn knowledge_new(
    state: &ToolState,
    request: KnowledgeRequestNew,
) -> Result<String, String> {
    debug!("knowledge_new: action={}", request.action);

    let service_guard = state.knowledge_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Knowledge service not initialized".to_string())?;

    match request.action.as_str() {
        "init" => {
            service.init().await.map_err(|e| e.to_string())?;

            let response = KnowledgeInitResponse {
                repo_path: service.knowledge_repo_path().display().to_string(),
                success: true,
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "scan" => {
            let path = request.path.ok_or("path required for scan")?;
            let repo_name = request.repo_name.as_deref().unwrap_or("default");

            // Ensure initialized
            service.init().await.map_err(|e| e.to_string())?;

            let result = service
                .scan_directory(Path::new(&path), repo_name)
                .await
                .map_err(|e| e.to_string())?;

            let response = KnowledgeScanResponse {
                files_found: result.files_found,
                files_new: result.files_new,
                files_updated: result.files_updated,
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "register" => {
            let path = request.path.ok_or("path required for register")?;
            let name = request.name.ok_or("name required for register")?;
            let doc_type_str = request.doc_type.ok_or("doc_type required for register")?;
            let doc_type = parse_doc_type(&doc_type_str)?;

            // Ensure initialized
            service.init().await.map_err(|e| e.to_string())?;

            let doc = service
                .register_doc(Path::new(&path), &name, doc_type)
                .await
                .map_err(|e| e.to_string())?;

            let response = KnowledgeDocResponse {
                id: doc.id.to_string(),
                name: doc.name,
                doc_type: doc.doc_type.to_string(),
                path: doc.canonical_path,
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "import" => {
            let path = request.path.ok_or("path required for import")?;
            let name = request.name.ok_or("name required for import")?;
            let doc_type_str = request.doc_type.ok_or("doc_type required for import")?;
            let doc_type = parse_doc_type(&doc_type_str)?;

            // Ensure initialized
            service.init().await.map_err(|e| e.to_string())?;

            let doc = service
                .import_doc(Path::new(&path), &name, doc_type)
                .await
                .map_err(|e| e.to_string())?;

            let response = KnowledgeDocResponse {
                id: doc.id.to_string(),
                name: doc.name,
                doc_type: doc.doc_type.to_string(),
                path: doc.canonical_path,
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "list" => {
            // Ensure initialized
            service.init().await.map_err(|e| e.to_string())?;

            let docs = service.list_docs().await.map_err(|e| e.to_string())?;

            let documents: Vec<KnowledgeDocInfo> = docs
                .into_iter()
                .map(|doc| KnowledgeDocInfo {
                    id: doc.id.to_string(),
                    name: doc.name,
                    doc_type: doc.doc_type.to_string(),
                    status: format!("{:?}", doc.status),
                    path: doc.canonical_path,
                })
                .collect();

            let response = KnowledgeListResponse {
                count: documents.len(),
                documents,
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "duplicates" => {
            // Ensure initialized
            service.init().await.map_err(|e| e.to_string())?;

            let duplicates = service.find_duplicates().await.map_err(|e| e.to_string())?;

            let groups: Vec<DuplicateGroupInfo> = duplicates
                .into_iter()
                .map(|g| DuplicateGroupInfo {
                    hash: g.content_hash,
                    paths: g.files.into_iter().map(|f| f.path).collect(),
                })
                .collect();

            let response = KnowledgeDuplicatesResponse {
                count: groups.len(),
                groups,
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "versions" => {
            // Ensure initialized
            service.init().await.map_err(|e| e.to_string())?;

            let chains = service.detect_versions().await.map_err(|e| e.to_string())?;

            let mut chain_infos = Vec::new();
            for chain in chains {
                let recommended = service.resolve_canonical(&chain).await.ok().flatten();

                chain_infos.push(VersionChainInfo {
                    base_name: chain.base_name,
                    versions: chain
                        .versions
                        .into_iter()
                        .map(|v| VersionInfo {
                            path: v.path,
                            version: v.version,
                        })
                        .collect(),
                    recommended_canonical: recommended,
                });
            }

            let response = KnowledgeVersionsResponse {
                count: chain_infos.len(),
                chains: chain_infos,
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        _ => Err(format!(
            "Unknown action: '{}'. Valid actions: init, scan, register, import, list, duplicates, versions",
            request.action
        )),
    }
}

// =============================================================================
// Phase 5: Consolidated Session Tool
// =============================================================================

/// Consolidated request for all session operations.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionRequestNew {
    /// Action to perform: start, end, get, list, log, search
    #[schemars(description = "Action: start, end, get, list, log, search")]
    pub action: String,

    /// Session ID (for end, get, log actions)
    #[schemars(description = "Session ID (required for end, get, log)")]
    pub session_id: Option<String>,

    /// Agent name (for start action)
    #[schemars(description = "Agent type (for start)")]
    pub agent: Option<String>,

    /// Project path (for start, list actions)
    #[schemars(description = "Project path")]
    pub project: Option<String>,

    /// Goal description (for start action)
    #[schemars(description = "Session goal (for start)")]
    pub goal: Option<String>,

    /// Summary (for end action)
    #[schemars(description = "Session summary (for end)")]
    pub summary: Option<String>,

    /// Event type (for log action): decision, observation, error, command, file_change, tool_use, milestone
    #[schemars(description = "Event type for log action")]
    pub event_type: Option<String>,

    /// Content (for log action)
    #[schemars(description = "Event content (for log)")]
    pub content: Option<String>,

    /// Context (for log action)
    #[schemars(description = "Event context/rationale (for log)")]
    pub context: Option<String>,

    /// Source (for log action)
    #[schemars(description = "Event source (for log)")]
    pub source: Option<String>,

    /// Query (for search action)
    #[schemars(description = "Search query (for search)")]
    pub query: Option<String>,

    /// Status filter (for list action): active, completed, abandoned
    #[schemars(description = "Status filter (for list)")]
    pub status: Option<String>,

    /// Max results (for list, search actions)
    #[schemars(description = "Maximum results")]
    pub limit: Option<usize>,
}

/// Parse session status string
fn parse_session_status(status: &str) -> Result<SessionStatus, String> {
    match status.to_lowercase().as_str() {
        "active" => Ok(SessionStatus::Active),
        "completed" => Ok(SessionStatus::Completed),
        "abandoned" => Ok(SessionStatus::Abandoned),
        _ => Err(format!(
            "Unknown status: '{}'. Valid: active, completed, abandoned",
            status
        )),
    }
}

/// Parse event type string
fn parse_event_type(event_type: &str) -> Result<EventType, String> {
    match event_type.to_lowercase().as_str() {
        "decision" => Ok(EventType::Decision),
        "observation" => Ok(EventType::Observation),
        "error" => Ok(EventType::Error),
        "command" => Ok(EventType::Command),
        "file_change" => Ok(EventType::FileChange),
        "tool_use" => Ok(EventType::ToolUse),
        "milestone" => Ok(EventType::Milestone),
        _ => Err(format!(
            "Unknown event_type: '{}'. Valid: decision, observation, error, command, file_change, tool_use, milestone",
            event_type
        )),
    }
}

/// Consolidated session handler.
pub async fn session_new(state: &ToolState, request: SessionRequestNew) -> Result<String, String> {
    debug!("session_new: action={}", request.action);

    let service_guard = state.session_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Session service not initialized".to_string())?;

    match request.action.as_str() {
        "start" => {
            let session = service
                .start_session(
                    request.agent.as_deref(),
                    request.project.as_deref(),
                    request.goal.as_deref(),
                )
                .await
                .map_err(|e| e.to_string())?;

            let response = SessionStartResponse {
                id: session.id.to_string(),
                agent: session.agent,
                project: session.project,
                goal: session.goal,
                started_at: session
                    .started_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_default(),
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "end" => {
            let session_id = request.session_id.ok_or("session_id required for end")?;
            let id = engram_core::id::Id::parse(&session_id)
                .map_err(|e| format!("Invalid session ID: {}", e))?;

            service
                .end_session(&id, request.summary.as_deref())
                .await
                .map_err(|e| e.to_string())?;

            serde_json::to_string_pretty(&serde_json::json!({
                "message": format!("Session {} ended", session_id),
                "summary": request.summary
            }))
            .map_err(|e| e.to_string())
        }
        "get" => {
            let session_id = request.session_id.ok_or("session_id required for get")?;
            let id = engram_core::id::Id::parse(&session_id)
                .map_err(|e| format!("Invalid session ID: {}", e))?;

            let (session, events) = service
                .get_session_with_events(&id)
                .await
                .map_err(|e| e.to_string())?;

            let response = SessionGetResponse {
                session: SessionInfo {
                    id: session.id.to_string(),
                    agent: session.agent,
                    project: session.project,
                    goal: session.goal,
                    status: session.status.to_string(),
                    summary: session.summary,
                    started_at: session
                        .started_at
                        .format(&time::format_description::well_known::Rfc3339)
                        .unwrap_or_default(),
                    ended_at: session.ended_at.map(|dt| {
                        dt.format(&time::format_description::well_known::Rfc3339)
                            .unwrap_or_default()
                    }),
                },
                events: events
                    .into_iter()
                    .map(|e| EventInfo {
                        id: e.id.to_string(),
                        event_type: e.event_type.to_string(),
                        actor: e.actor,
                        content: e.content,
                        context: e.context,
                        source: e.source,
                        timestamp: e
                            .timestamp
                            .format(&time::format_description::well_known::Rfc3339)
                            .unwrap_or_default(),
                    })
                    .collect(),
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "list" => {
            let status: Option<SessionStatus> = if let Some(s) = &request.status {
                Some(parse_session_status(s)?)
            } else {
                None
            };

            let sessions = service
                .list_sessions(
                    status.as_ref(),
                    request.agent.as_deref(),
                    request.project.as_deref(),
                    Some(request.limit.unwrap_or(20)),
                )
                .await
                .map_err(|e| e.to_string())?;

            let response = SessionListResponse {
                count: sessions.len(),
                sessions: sessions
                    .into_iter()
                    .map(|s| SessionInfo {
                        id: s.id.to_string(),
                        agent: s.agent,
                        project: s.project,
                        goal: s.goal,
                        status: s.status.to_string(),
                        summary: s.summary,
                        started_at: s
                            .started_at
                            .format(&time::format_description::well_known::Rfc3339)
                            .unwrap_or_default(),
                        ended_at: s.ended_at.map(|dt| {
                            dt.format(&time::format_description::well_known::Rfc3339)
                                .unwrap_or_default()
                        }),
                    })
                    .collect(),
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "log" => {
            let session_id = request.session_id.ok_or("session_id required for log")?;
            let event_type_str = request.event_type.ok_or("event_type required for log")?;
            let content = request.content.ok_or("content required for log")?;

            let id = engram_core::id::Id::parse(&session_id)
                .map_err(|e| format!("Invalid session ID: {}", e))?;
            let event_type = parse_event_type(&event_type_str)?;

            let event = service
                .log_event(
                    &id,
                    event_type,
                    &content,
                    request.context.as_deref(),
                    request.source.as_deref(),
                )
                .await
                .map_err(|e| e.to_string())?;

            let response = SessionLogResponse {
                event: EventInfo {
                    id: event.id.to_string(),
                    event_type: event.event_type.to_string(),
                    actor: event.actor,
                    content: event.content,
                    context: event.context,
                    source: event.source,
                    timestamp: event
                        .timestamp
                        .format(&time::format_description::well_known::Rfc3339)
                        .unwrap_or_default(),
                },
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "search" => {
            let query = request.query.ok_or("query required for search")?;

            let events = service
                .search_events(&query, Some(request.limit.unwrap_or(20)))
                .await
                .map_err(|e| e.to_string())?;

            let response = SessionSearchResponse {
                count: events.len(),
                events: events
                    .into_iter()
                    .map(|e| EventInfo {
                        id: e.id.to_string(),
                        event_type: e.event_type.to_string(),
                        actor: e.actor,
                        content: e.content,
                        context: e.context,
                        source: e.source,
                        timestamp: e
                            .timestamp
                            .format(&time::format_description::well_known::Rfc3339)
                            .unwrap_or_default(),
                    })
                    .collect(),
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        _ => Err(format!(
            "Unknown action: '{}'. Valid actions: start, end, get, list, log, search",
            request.action
        )),
    }
}

// =============================================================================
// Phase 5: Consolidated Document Tool
// =============================================================================

/// Consolidated request for all document operations.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DocsRequestNew {
    /// Action to perform: search, index, stats
    #[schemars(description = "Action: search, index, stats")]
    pub action: String,

    /// Search query (for search action)
    #[schemars(description = "Search query (for search)")]
    pub query: Option<String>,

    /// Path to index (for index action)
    #[schemars(description = "Path to file or directory (for index)")]
    pub path: Option<String>,

    /// Max results (for search action)
    #[schemars(description = "Maximum results (for search)")]
    pub limit: Option<usize>,

    /// Minimum score threshold (for search action)
    #[schemars(description = "Minimum similarity score 0.0-1.0 (for search)")]
    pub min_score: Option<f32>,
}

/// Consolidated document handler.
pub async fn docs_new(state: &ToolState, request: DocsRequestNew) -> Result<String, String> {
    debug!("docs_new: action={}", request.action);

    let service_guard = state.doc_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Document service not initialized".to_string())?;

    match request.action.as_str() {
        "search" => {
            let query = request.query.ok_or("query required for search")?;
            let limit = request.limit.unwrap_or(5);

            let results = if let Some(min_score) = request.min_score {
                service
                    .search_threshold(&query, limit, min_score)
                    .await
                    .map_err(|e| e.to_string())?
            } else {
                service
                    .search(&query, limit)
                    .await
                    .map_err(|e| e.to_string())?
            };

            let response = SearchDocsResponse {
                count: results.len(),
                results: results.into_iter().map(SearchResult::from).collect(),
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "index" => {
            let path_str = request.path.ok_or("path required for index")?;
            let path = std::path::Path::new(&path_str);
            let mut warnings = Vec::new();

            let (docs_indexed, chunks_created) = if path.is_dir() {
                let results = service
                    .index_directory(path)
                    .await
                    .map_err(|e| e.to_string())?;

                let docs = results.len();
                let chunks: usize = results.iter().map(|d| d.chunks.len()).sum();
                (docs, chunks)
            } else if path.is_file() {
                let result = service.index_file(path).await.map_err(|e| e.to_string())?;
                (1, result.chunks.len())
            } else {
                warnings.push(format!("Path not found: {}", path_str));
                (0, 0)
            };

            let response = IndexDocsResponse {
                documents_indexed: docs_indexed,
                chunks_created,
                warnings,
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "stats" => {
            let stats = service.stats().await.map_err(|e| e.to_string())?;

            let response = GetStatsResponse {
                source_count: stats.source_count,
                chunk_count: stats.chunk_count,
                embedding_dimension: stats.embedding_dimension,
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        _ => Err(format!(
            "Unknown action: '{}'. Valid actions: search, index, stats",
            request.action
        )),
    }
}

// =============================================================================
// Phase 5: Consolidated Tool Intelligence Tool
// =============================================================================

/// Consolidated request for all tool intelligence operations.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolRequestNew {
    /// Action to perform: log, recommend, stats, list, search
    #[schemars(description = "Action: log, recommend, stats, list, search")]
    pub action: String,

    /// Tool name (for log, stats actions)
    #[schemars(description = "Tool name")]
    pub tool_name: Option<String>,

    /// Context (for log, recommend actions)
    #[schemars(description = "Usage context or recommendation context")]
    pub context: Option<String>,

    /// Outcome (for log action): success, partial, failed, switched
    #[schemars(description = "Outcome for log: success, partial, failed, switched")]
    pub outcome: Option<String>,

    /// Session ID (for log action)
    #[schemars(description = "Session ID to link usage to")]
    pub session_id: Option<String>,

    /// Query (for search action)
    #[schemars(description = "Search query (for search)")]
    pub query: Option<String>,

    /// Outcome filter (for list action)
    #[schemars(description = "Filter by outcome (for list)")]
    pub outcome_filter: Option<String>,

    /// Max results (for list, search actions)
    #[schemars(description = "Maximum results")]
    pub limit: Option<usize>,
}

/// Parse tool outcome string
fn parse_tool_outcome(outcome: &str) -> Result<ToolOutcome, String> {
    match outcome.to_lowercase().as_str() {
        "success" => Ok(ToolOutcome::Success),
        "partial" => Ok(ToolOutcome::Partial),
        "failed" => Ok(ToolOutcome::Failed),
        "switched" => Ok(ToolOutcome::Switched),
        _ => Err(format!(
            "Unknown outcome: '{}'. Valid: success, partial, failed, switched",
            outcome
        )),
    }
}

/// Consolidated tool intelligence handler.
pub async fn tool_new(state: &ToolState, request: ToolRequestNew) -> Result<String, String> {
    debug!("tool_new: action={}", request.action);

    let service_guard = state.tool_intel_service.read().await;
    let service = service_guard
        .as_ref()
        .ok_or_else(|| "Tool intelligence service not initialized".to_string())?;

    match request.action.as_str() {
        "log" => {
            let tool_name = request.tool_name.ok_or("tool_name required for log")?;
            let context = request.context.ok_or("context required for log")?;
            let outcome_str = request.outcome.ok_or("outcome required for log")?;
            let outcome = parse_tool_outcome(&outcome_str)?;

            let session_id = if let Some(sid) = &request.session_id {
                Some(
                    engram_core::id::Id::parse(sid)
                        .map_err(|e| format!("Invalid session ID: {}", e))?,
                )
            } else {
                None
            };

            let usage = service
                .log_usage(&tool_name, &context, outcome, session_id.as_ref())
                .await
                .map_err(|e| e.to_string())?;

            let response = ToolLogUsageResponse {
                id: usage.id.to_string(),
                tool_name,
                context: usage.context,
                outcome: usage.outcome.to_string(),
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "recommend" => {
            let context = request.context.ok_or("context required for recommend")?;

            let recommendations = service
                .get_recommendations(&context)
                .await
                .map_err(|e| e.to_string())?;

            let response = ToolRecommendResponse {
                count: recommendations.len(),
                recommendations: recommendations
                    .into_iter()
                    .map(|r| ToolRecommendationInfo {
                        tool_name: r.tool_name,
                        confidence: r.confidence,
                        reason: r.reason,
                    })
                    .collect(),
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "stats" => {
            let tool_name = request.tool_name.ok_or("tool_name required for stats")?;

            let stats = service
                .get_tool_stats(&tool_name)
                .await
                .map_err(|e| e.to_string())?;

            let response = ToolGetStatsResponse {
                tool_name,
                total_usages: stats.total_usages,
                success_count: stats.success_count,
                failure_count: stats.failure_count,
                success_rate: stats.success_rate,
                preferences_count: stats.preferences_count,
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "list" => {
            let outcome: Option<ToolOutcome> = if let Some(o) = &request.outcome_filter {
                Some(parse_tool_outcome(o)?)
            } else {
                None
            };

            let usages = service
                .list_usages(outcome.as_ref(), request.limit)
                .await
                .map_err(|e| e.to_string())?;

            let response = ToolListUsagesResponse {
                count: usages.len(),
                usages: usages
                    .into_iter()
                    .map(|u| ToolUsageInfoMcp {
                        id: u.id.to_string(),
                        tool_name: u.tool_name,
                        context: u.context,
                        outcome: u.outcome.to_string(),
                        timestamp: u
                            .timestamp
                            .format(&time::format_description::well_known::Rfc3339)
                            .unwrap_or_default(),
                    })
                    .collect(),
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        "search" => {
            let query = request.query.ok_or("query required for search")?;

            let usages = service
                .search_usages(&query, request.limit)
                .await
                .map_err(|e| e.to_string())?;

            let response = ToolListUsagesResponse {
                count: usages.len(),
                usages: usages
                    .into_iter()
                    .map(|u| ToolUsageInfoMcp {
                        id: u.id.to_string(),
                        tool_name: u.tool_name,
                        context: u.context,
                        outcome: u.outcome.to_string(),
                        timestamp: u
                            .timestamp
                            .format(&time::format_description::well_known::Rfc3339)
                            .unwrap_or_default(),
                    })
                    .collect(),
            };
            serde_json::to_string_pretty(&response).map_err(|e| e.to_string())
        }
        _ => Err(format!(
            "Unknown action: '{}'. Valid actions: log, recommend, stats, list, search",
            request.action
        )),
    }
}
