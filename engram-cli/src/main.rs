//! # engram CLI
//!
//! Command-line interface for managing the engram knowledge system.

mod daemon;
mod proxy;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use engram_core::entity::{EntityType, RelationType};
use engram_core::graph::MemorySubgraph;
use engram_core::harness::{
    HarnessAdapterStatus, HarnessInstallReport, HarnessKind, HarnessStatusReport,
};
use engram_core::knowledge::DocType;
use engram_core::lint::{LintFinding, LintReport, LintSeverity};
use engram_core::memory::{
    Harness, MemoryCursor, MemoryItem, MemoryScope, MemoryStatus, ModelIdentity, WriterProvenance,
};
use engram_core::obligation::{
    AgentObligation, AgentObligationKind, AgentObligationResolution, AgentObligationResolutionKind,
    AgentObligationStatus, AgentObligationTrigger,
};
use engram_core::repository::{ProjectRepositoryRole, RepositoryContext};
use engram_core::session::{EventType, SessionStatus};
use engram_core::tool::ToolOutcome;
use engram_core::Id;
use engram_index::{
    ChunkingStrategy, CoordinationService, DigestExtractionOptions, DigestExtractionPlan,
    DigestExtractionReviewApply, DigestExtractionReviewApplyOptions, DigestInventory,
    DigestInventoryOptions, DigestReviewApply, DigestReviewExport, DigestService,
    DigestSourceIndexOptions, DigestSourceIndexPlan, DocumentIngestionPlan,
    DocumentOrphanCleanupAction, DocumentOrphanCleanupExecutionOptions,
    DocumentOrphanCleanupExecutionReport, DocumentOrphanCleanupExecutionStatus,
    DocumentOrphanCleanupPlan, DocumentOrphanCleanupPlanOptions,
    DocumentOrphanQuarantineReviewApply, DocumentOrphanQuarantineReviewApplyOptions,
    DocumentOrphanQuarantineReviewExport, DocumentOrphanQuarantineReviewOptions,
    DocumentOrphanQuarantineReviewPrioritization,
    DocumentOrphanQuarantineReviewPrioritizationOptions, DocumentOrphanQuarantineReviewStatus,
    DocumentOrphanReport, DocumentRecoveryClass, DocumentRecoveryOptions, DocumentReindexAction,
    DocumentReindexExecutionOptions, DocumentReindexExecutionReport,
    DocumentReindexExecutionStatus, DocumentReindexPlan, DocumentService, EntityService,
    GraphService, HandoffService, HarnessHookEvent, HarnessHookServices, HarnessInstallOptions,
    HarnessService, HarnessSettingsTarget, KnowledgeService, LintOptions, LintService,
    MemoryChanges, MemoryChangesSinceOptions, MemoryService, MigrationInventory,
    MigrationInventoryOptions, MigrationReviewApply, MigrationReviewApplyOptions,
    MigrationReviewExport, MigrationReviewStatus, ObligationDetectOptions, ObligationDetection,
    ObligationDoctorReport, ObligationService, OrientInput, OrientationPacket, Pipeline,
    PipelineConfig, RepositoryMigrationInventory, RepositoryMigrationOptions,
    RepositoryMigrationReviewApply, RepositoryMigrationReviewApplyOptions,
    RepositoryMigrationReviewExport, RepositoryMigrationReviewStatus, RepositoryService,
    SearchService, SessionService, TelemetryService, ToolIntelService, WorkService,
};
use engram_mcp::EngramServer;
use engram_store::{connect_and_init, StoreConfig};
use time::OffsetDateTime;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

const EXTERNAL_SESSION_ID_ENV: &str = "ENGRAM_EXTERNAL_SESSION_ID";
const CLAUDE_CODE_SESSION_ID_ENV: &str = "CLAUDE_CODE_SESSION_ID";
const CLAUDE_CODE_MARKER_ENV: &str = "CLAUDECODE";
const CODEX_THREAD_ID_ENV: &str = "CODEX_THREAD_ID";
const CODEX_SHELL_ENV: &str = "CODEX_SHELL";
const CODEX_ORIGINATOR_ENV: &str = "CODEX_INTERNAL_ORIGINATOR_OVERRIDE";
const CODEX_BUNDLE_ID_ENV: &str = "__CFBundleIdentifier";
const CLAUDE_CODE_EXTERNAL_SESSION_PREFIX: &str = "claude-code://sessions/";
const CODEX_THREAD_EXTERNAL_SESSION_PREFIX: &str = "codex://threads/";
const MAX_CLAUDE_CODE_SESSION_ID_LEN: usize = 128;
const MAX_CODEX_THREAD_ID_LEN: usize = 128;

/// engram - Personal Knowledge Augmentation System for AI coding agents
#[derive(Parser)]
#[command(name = "engram")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize the engram database
    Init {
        /// Data directory path
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Start the MCP server
    Serve {
        /// Use in-memory storage in --http daemon mode (for testing)
        #[arg(long)]
        memory: bool,

        /// Connect to remote SurrealDB server in --http daemon mode (e.g., ws://localhost:8000)
        /// Enables concurrent access from multiple engram sessions
        #[arg(long)]
        remote: Option<String>,

        /// Username for remote server authentication
        #[arg(long)]
        username: Option<String>,

        /// Password for remote server authentication
        #[arg(long)]
        password: Option<String>,

        /// Run as HTTP server (daemon mode) instead of stdio proxy
        #[arg(long)]
        http: bool,

        /// Port to listen on in --http mode (default: 8765)
        #[arg(long)]
        port: Option<u16>,

        /// Project-specific mode (isolated data store per project)
        #[arg(long)]
        project: Option<String>,
    },

    /// Manage the engram daemon
    Daemon {
        #[command(subcommand)]
        command: DaemonCommands,
    },

    /// Add an entity to the knowledge base
    Add {
        #[command(subcommand)]
        what: AddCommands,
    },

    /// Search the knowledge base
    Search {
        /// Search query
        query: String,

        /// Entity type filter
        #[arg(short, long)]
        r#type: Option<String>,
    },

    /// Index documentation
    Index {
        /// Path to index
        path: String,

        /// Recursive indexing
        #[arg(short, long)]
        recursive: bool,

        /// Show the ingestion plan without writing document sources or chunks
        #[arg(long)]
        plan: bool,
    },

    /// Search documentation
    SearchDocs {
        /// Search query
        query: String,

        /// Maximum results
        #[arg(short, long, default_value = "5")]
        limit: usize,

        /// Minimum score threshold (0.0 - 1.0)
        #[arg(short, long, default_value = "0.3")]
        score: f32,
    },

    /// Show database statistics
    Stats,

    /// Report orphan document chunks without changing the store
    DocOrphans {
        /// Maximum orphan source groups to return
        #[arg(short, long, default_value = "20")]
        limit: usize,

        /// Include all orphan source groups in the report
        #[arg(long)]
        all: bool,

        /// Sample chunks to include per orphan source group
        #[arg(short, long, default_value = "3")]
        samples: usize,

        /// Current file or directory path to scan for fingerprint matches
        #[arg(long = "scan-path")]
        scan_paths: Vec<String>,

        /// Digest review batch root to scan for reviewed source matches
        #[arg(long = "digest-review-path")]
        digest_review_paths: Vec<String>,

        /// Maximum candidate files or digest sources to read
        #[arg(long, default_value = "5000")]
        max_candidate_files: usize,

        /// Maximum bytes to read per candidate file
        #[arg(long, default_value = "1048576")]
        max_file_bytes: usize,

        /// Write the report to a file
        #[arg(short, long)]
        output: Option<String>,

        /// Export file format
        #[arg(long, value_enum, default_value = "markdown")]
        format: OrphanExportFormat,
    },

    /// Build a read-only source-level reindex plan for recoverable orphan chunks
    DocReindexPlan {
        /// Maximum orphan source groups to analyze
        #[arg(short, long, default_value = "20")]
        limit: usize,

        /// Analyze all orphan source groups
        #[arg(long)]
        all: bool,

        /// Sample chunks to include per orphan source group
        #[arg(short, long, default_value = "3")]
        samples: usize,

        /// Current file or directory path to scan for fingerprint matches
        #[arg(long = "scan-path")]
        scan_paths: Vec<String>,

        /// Digest review batch root to scan for reviewed source matches
        #[arg(long = "digest-review-path")]
        digest_review_paths: Vec<String>,

        /// Maximum candidate files or digest sources to read
        #[arg(long, default_value = "5000")]
        max_candidate_files: usize,

        /// Maximum bytes to read per candidate file
        #[arg(long, default_value = "1048576")]
        max_file_bytes: usize,

        /// Write the plan to a file
        #[arg(short, long)]
        output: Option<String>,

        /// Export file format
        #[arg(long, value_enum, default_value = "markdown")]
        format: OrphanExportFormat,
    },

    /// Execute or dry-run a guarded source-level orphan reindex plan
    DocReindexExecute {
        /// JSON plan file produced by doc-reindex-plan --format json
        #[arg(long = "plan")]
        plan_path: String,

        /// Perform writes. Default mode is a dry-run.
        #[arg(long)]
        execute: bool,

        /// Approve all selected source actions in write mode
        #[arg(long)]
        all: bool,

        /// Exact source path to include. Repeat to approve a subset.
        #[arg(long = "source")]
        source_paths: Vec<String>,

        /// Action kind to include: reindex_file, reindex_digest_reviewed_source, inspect_existing_source
        #[arg(long = "action")]
        actions: Vec<String>,

        /// Digest review batch root to resolve digest reviewed source actions
        #[arg(long = "digest-review-path")]
        digest_review_paths: Vec<String>,

        /// Maximum bytes to read per digest source
        #[arg(long, default_value = "1048576")]
        max_source_bytes: usize,

        /// Maximum selected source actions to process
        #[arg(long)]
        max_actions: Option<usize>,

        /// Write the execution report to a file
        #[arg(short, long)]
        output: Option<String>,

        /// Export file format
        #[arg(long, value_enum, default_value = "markdown")]
        format: OrphanExportFormat,
    },

    /// Build a read-only cleanup/quarantine plan for remaining orphan chunks
    DocOrphanCleanupPlan {
        /// Maximum orphan source groups to analyze
        #[arg(short, long, default_value = "20")]
        limit: usize,

        /// Analyze all orphan source groups
        #[arg(long)]
        all: bool,

        /// Sample chunks to include per orphan source group
        #[arg(short, long, default_value = "3")]
        samples: usize,

        /// Current file or directory path to scan for fingerprint matches
        #[arg(long = "scan-path")]
        scan_paths: Vec<String>,

        /// Digest review batch root to scan for reviewed source matches
        #[arg(long = "digest-review-path")]
        digest_review_paths: Vec<String>,

        /// Maximum candidate files or digest sources to read
        #[arg(long, default_value = "5000")]
        max_candidate_files: usize,

        /// Maximum bytes to read per candidate file
        #[arg(long, default_value = "1048576")]
        max_file_bytes: usize,

        /// JSON source-level reindex plan used to map sources back to orphan groups
        #[arg(long = "reindex-plan")]
        reindex_plan_path: Option<String>,

        /// JSON write execution report used to prove successful reindex coverage
        #[arg(long = "execution-report")]
        execution_report_path: Option<String>,

        /// Write the cleanup/quarantine plan to a file
        #[arg(short, long)]
        output: Option<String>,

        /// Export file format
        #[arg(long, value_enum, default_value = "markdown")]
        format: OrphanExportFormat,
    },

    /// Execute or dry-run deletion for cleanup-plan delete candidates
    DocOrphanCleanupExecute {
        /// JSON cleanup plan file produced by doc-orphan-cleanup-plan --format json
        #[arg(long = "plan")]
        plan_path: String,

        /// Perform deletion. Default mode is a dry-run.
        #[arg(long)]
        execute: bool,

        /// Explicitly approve deleting delete_after_successful_reindex groups in write mode
        #[arg(long)]
        delete_candidates: bool,

        /// Approve all delete candidates in write mode
        #[arg(long)]
        all_delete_candidates: bool,

        /// Exact missing source ID to include. Repeat to approve a subset.
        #[arg(long = "source-id")]
        source_ids: Vec<String>,

        /// Maximum selected delete groups to process
        #[arg(long)]
        max_groups: Option<usize>,

        /// Export quarantine groups to a separate file without deleting them
        #[arg(long)]
        quarantine_output: Option<String>,

        /// Write the execution report to a file
        #[arg(short, long)]
        output: Option<String>,

        /// Export file format
        #[arg(long, value_enum, default_value = "markdown")]
        format: OrphanExportFormat,
    },

    /// Export retained quarantine orphan chunks into a generated Markdown review batch
    DocOrphanQuarantineReviewExport {
        /// JSON cleanup plan file with quarantine groups
        #[arg(long = "plan")]
        plan_path: String,

        /// Output directory for generated review pages
        #[arg(long = "output-dir")]
        output_dir: String,

        /// Maximum quarantine groups to export
        #[arg(long)]
        max_groups: Option<usize>,

        /// Maximum chunks to include per group
        #[arg(long)]
        max_chunks_per_group: Option<usize>,

        /// Maximum content bytes per chunk before truncation
        #[arg(long, default_value = "16384")]
        max_chunk_bytes: usize,
    },

    /// Inspect decision status for a generated document orphan quarantine review batch
    DocOrphanQuarantineReviewStatus {
        /// Review batch directory produced by doc-orphan-quarantine-review-export
        #[arg(long = "review-path")]
        review_path: String,
    },

    /// Rank generated document orphan quarantine review pages for a small review pilot
    DocOrphanQuarantineReviewPrioritize {
        /// Review batch directory produced by doc-orphan-quarantine-review-export
        #[arg(long = "review-path")]
        review_path: String,

        /// Maximum prioritized pages to print
        #[arg(short, long, default_value = "10")]
        limit: usize,

        /// Include already decided pages in the prioritization output
        #[arg(long)]
        include_decided: bool,

        /// Include duplicate content fingerprints in the prioritization output
        #[arg(long)]
        include_duplicate_fingerprints: bool,

        /// Maximum excerpt bytes to include per item
        #[arg(long, default_value = "800")]
        max_excerpt_bytes: usize,
    },

    /// Dry-run actions implied by a generated document orphan quarantine review batch
    DocOrphanQuarantineReviewApply {
        /// Review batch directory produced by doc-orphan-quarantine-review-export
        #[arg(long = "review-path")]
        review_path: String,
    },

    /// Manage Layer 6: Knowledge Documents
    Knowledge {
        #[command(subcommand)]
        command: KnowledgeCommands,
    },

    /// Manage Layer 1: Entity Knowledge (repos, tools, concepts, services)
    Entity {
        #[command(subcommand)]
        command: EntityCommands,
    },

    /// Manage Layer 2: Session History (decisions, events, rationale)
    Session {
        #[command(subcommand)]
        command: SessionCommands,
    },

    /// Manage Layer 4: Tool Intelligence (usage tracking, recommendations)
    Tool {
        #[command(subcommand)]
        command: ToolCommands,
    },

    /// Manage Layer 5: Session Coordination (parallel session awareness)
    Coord {
        #[command(subcommand)]
        command: CoordCommands,
    },

    /// Run database migrations
    Migrate {
        #[command(subcommand)]
        command: MigrateCommands,
    },

    /// Manage Layer 7: Work Management (projects, tasks, PRs)
    Work {
        #[command(subcommand)]
        command: WorkCommands,
    },

    /// Manage Memory OS records and vault projections
    Memory {
        /// Use a project-specific Engram data store
        #[arg(long)]
        project: Option<String>,

        /// Use an explicit RocksDB data directory
        #[arg(long)]
        data_dir: Option<String>,

        #[command(subcommand)]
        command: MemoryCommands,
    },

    /// Build a Memory OS orientation context packet for the current workspace
    Orient {
        /// Explicit Memory OS project/scope name
        #[arg(long)]
        project: Option<String>,

        /// Current working directory, defaults to the process cwd
        #[arg(long)]
        cwd: Option<String>,

        /// Prompt or task that triggered orientation
        #[arg(long)]
        prompt: Option<String>,

        /// Agent or harness name
        #[arg(long)]
        agent: Option<String>,

        /// Host conversation/session label for telemetry; falls back to ENGRAM_EXTERNAL_SESSION_ID,
        /// guarded CLAUDE_CODE_SESSION_ID, then guarded CODEX_THREAD_ID
        #[arg(long)]
        external_session_id: Option<String>,

        /// Include recent knowledge commits in the orientation packet
        #[arg(long)]
        include_recent_commits: bool,

        /// Maximum memory items per grouped bucket
        #[arg(long)]
        limit: Option<usize>,

        /// Use a project-specific Engram data store
        #[arg(long)]
        store_project: Option<String>,

        /// Use an explicit RocksDB data directory
        #[arg(long)]
        data_dir: Option<String>,

        /// Print the full orientation packet as JSON
        #[arg(long)]
        json: bool,
    },

    /// Manage Memory OS agent harness policy and adapters
    Harness {
        #[command(subcommand)]
        command: HarnessCommands,
    },

    /// Run Memory OS health linting
    Lint {
        /// Use a project-specific Engram data store
        #[arg(long)]
        project: Option<String>,

        /// Use an explicit RocksDB data directory
        #[arg(long)]
        data_dir: Option<String>,

        #[command(subcommand)]
        command: LintCommands,
    },

    /// Traverse the derived Memory OS graph
    Graph {
        /// Use a project-specific Engram data store
        #[arg(long)]
        project: Option<String>,

        /// Use an explicit RocksDB data directory
        #[arg(long)]
        data_dir: Option<String>,

        #[command(subcommand)]
        command: GraphCommands,
    },

    /// Manage rolling Memory OS handoffs
    Handoff {
        /// Use a project-specific Engram data store
        #[arg(long)]
        project: Option<String>,

        /// Use an explicit RocksDB data directory
        #[arg(long)]
        data_dir: Option<String>,

        #[command(subcommand)]
        command: HandoffCommands,
    },

    /// Manage agent-native session obligations
    Obligations {
        /// Use a project-specific Engram data store
        #[arg(long)]
        project: Option<String>,

        /// Use an explicit RocksDB data directory
        #[arg(long)]
        data_dir: Option<String>,

        #[command(subcommand)]
        command: ObligationCommands,
    },

    /// Manage the generated Memory OS Markdown vault
    Vault {
        /// Use a project-specific Engram data store
        #[arg(long)]
        project: Option<String>,

        /// Use an explicit RocksDB data directory
        #[arg(long)]
        data_dir: Option<String>,

        #[command(subcommand)]
        command: VaultCommands,
    },

    /// Inventory scheduled digest source files without reading contents
    Digest {
        #[command(subcommand)]
        command: DigestCommands,
    },

    /// Manage repository topology and local checkout mapping
    Repo {
        /// Use a project-specific Engram data store
        #[arg(long)]
        project: Option<String>,

        /// Use an explicit RocksDB data directory
        #[arg(long)]
        data_dir: Option<String>,

        #[command(subcommand)]
        command: RepoCommands,
    },
}

/// Export format for document orphan recovery reports.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum OrphanExportFormat {
    Markdown,
    Json,
}

/// Document type for knowledge management.
#[derive(Debug, Clone, ValueEnum)]
enum DocTypeArg {
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

#[derive(Subcommand)]
enum KnowledgeCommands {
    /// Initialize the knowledge system (creates directories and git repo)
    Init,

    /// Scan a directory for documents
    Scan {
        /// Path to scan
        path: String,

        /// Repository name (identifier for tracking)
        #[arg(short, long, default_value = "default")]
        repo: String,
    },

    /// Import a document to the personal knowledge repo
    Import {
        /// Source file path
        source: String,

        /// Document name
        #[arg(short, long)]
        name: String,

        /// Document type
        #[arg(short = 't', long, value_enum)]
        doc_type: DocTypeArg,
    },

    /// Register a document (reference only, doesn't copy)
    Register {
        /// File path
        path: String,

        /// Document name
        #[arg(short, long)]
        name: String,

        /// Document type
        #[arg(short = 't', long, value_enum)]
        doc_type: DocTypeArg,
    },

    /// List all knowledge documents
    List,

    /// Find duplicate documents
    Duplicates,

    /// Detect version chains
    Versions,

    /// Show knowledge statistics
    Stats,
}

/// Entity type for CLI.
#[derive(Debug, Clone, ValueEnum)]
enum EntityTypeArg {
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

/// Relation type for CLI.
#[derive(Debug, Clone, ValueEnum)]
enum RelationTypeArg {
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

/// Event type for CLI.
#[derive(Debug, Clone, ValueEnum)]
enum EventTypeArg {
    Decision,
    Command,
    FileChange,
    ToolUse,
    Error,
    Milestone,
    Observation,
    Prompt,
    Plan,
    ToolResult,
    Test,
    Preference,
    Rule,
    Limitation,
    HandoffUpdate,
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
            EventTypeArg::Prompt => EventType::Prompt,
            EventTypeArg::Plan => EventType::Plan,
            EventTypeArg::ToolResult => EventType::ToolResult,
            EventTypeArg::Test => EventType::Test,
            EventTypeArg::Preference => EventType::Preference,
            EventTypeArg::Rule => EventType::Rule,
            EventTypeArg::Limitation => EventType::Limitation,
            EventTypeArg::HandoffUpdate => EventType::HandoffUpdate,
        }
    }
}

/// Session status for CLI.
#[derive(Debug, Clone, ValueEnum)]
enum SessionStatusArg {
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

#[derive(Subcommand)]
enum SessionCommands {
    /// Start a new coding session
    Start {
        /// Agent type (e.g., "claude-code", "cursor")
        #[arg(short, long)]
        agent: Option<String>,

        /// Project name or directory
        #[arg(short, long)]
        project: Option<String>,

        /// Goal of this session
        #[arg(short, long)]
        goal: Option<String>,
    },

    /// End a session
    End {
        /// Session ID to end (uses most recent active if not specified)
        session_id: Option<String>,

        /// Summary of what was accomplished
        #[arg(short, long)]
        summary: Option<String>,
    },

    /// List sessions
    List {
        /// Filter by status
        #[arg(short, long, value_enum)]
        status: Option<SessionStatusArg>,

        /// Filter by agent
        #[arg(short, long)]
        agent: Option<String>,

        /// Filter by project
        #[arg(short, long)]
        project: Option<String>,

        /// Maximum number of sessions
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// Show session details with events
    Show {
        /// Session ID
        session_id: String,
    },

    /// Log an event to a session
    Log {
        /// Event type
        #[arg(short = 't', long, value_enum)]
        event_type: EventTypeArg,

        /// Event content
        content: String,

        /// Session ID (uses most recent active if not specified)
        #[arg(short, long)]
        session: Option<String>,

        /// Additional context or rationale
        #[arg(short, long)]
        context: Option<String>,

        /// Source of the event
        #[arg(long)]
        source: Option<String>,
    },

    /// Search events across sessions
    Search {
        /// Search query
        query: String,

        /// Maximum results
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// Show session statistics
    Stats,
}

#[derive(Subcommand)]
enum EntityCommands {
    /// Create a new entity
    Create {
        /// Entity name
        name: String,

        /// Entity type
        #[arg(short = 't', long, value_enum)]
        entity_type: EntityTypeArg,

        /// Description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// List all entities
    List {
        /// Filter by entity type
        #[arg(short = 't', long, value_enum)]
        entity_type: Option<EntityTypeArg>,
    },

    /// Show entity details
    Show {
        /// Entity name or alias
        name: String,
    },

    /// Search entities by name
    Search {
        /// Search query
        query: String,
    },

    /// Create a relationship between entities
    Relate {
        /// Source entity name
        source: String,

        /// Relationship type
        #[arg(short = 't', long, value_enum)]
        relation: RelationTypeArg,

        /// Target entity name
        target: String,
    },

    /// Add an alias for an entity
    Alias {
        /// Entity name
        entity: String,

        /// Alias to add
        alias: String,
    },

    /// Add an observation (fact/note) about an entity
    Observe {
        /// Entity name
        entity: String,

        /// Observation content
        content: String,

        /// Semantic key for updates (e.g., 'architecture.auth', 'gotchas.race-conditions')
        /// If the key exists, the existing observation will be updated.
        #[arg(short, long)]
        key: Option<String>,

        /// Source of the observation
        #[arg(short, long)]
        source: Option<String>,
    },

    /// Delete an entity
    Delete {
        /// Entity name
        name: String,
    },

    /// Show entity statistics
    Stats,
}

#[derive(Subcommand)]
enum AddCommands {
    /// Add an entity (shortcut for 'entity create')
    Entity {
        /// Entity name
        name: String,

        /// Entity type
        #[arg(short = 't', long, value_enum)]
        entity_type: EntityTypeArg,

        /// Description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Add an alias (shortcut for 'entity alias')
    Alias {
        /// Alias text
        alias: String,

        /// Entity name
        #[arg(short, long)]
        entity: String,
    },
}

/// Tool outcome for CLI.
#[derive(Debug, Clone, ValueEnum)]
enum ToolOutcomeArg {
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

#[derive(Subcommand)]
enum ToolCommands {
    /// Log a tool usage with outcome
    Log {
        /// Tool name (must be registered as an entity of type 'tool')
        tool_name: String,

        /// Outcome of the tool usage
        #[arg(short, long, value_enum)]
        outcome: ToolOutcomeArg,

        /// Context (what was the tool used for?)
        #[arg(short, long)]
        context: String,

        /// Session ID to link this usage to
        #[arg(short, long)]
        session: Option<String>,
    },

    /// Get tool recommendations for a context
    Recommend {
        /// Context to get recommendations for
        context: String,
    },

    /// Show statistics for a specific tool
    Stats {
        /// Tool name (optional, shows overall stats if not specified)
        tool_name: Option<String>,
    },

    /// List recent tool usages
    List {
        /// Filter by outcome
        #[arg(short, long, value_enum)]
        outcome: Option<ToolOutcomeArg>,

        /// Maximum number of results
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// Search tool usage history
    Search {
        /// Search query
        query: String,

        /// Maximum number of results
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },
}

#[derive(Subcommand)]
enum DaemonCommands {
    /// Show daemon status
    Status {
        /// Project name (for project-specific daemon)
        #[arg(long)]
        project: Option<String>,
    },

    /// Start the daemon (if not running)
    Start {
        /// Project name (for project-specific daemon)
        #[arg(long)]
        project: Option<String>,

        /// Port to listen on
        #[arg(long)]
        port: Option<u16>,
    },

    /// Stop a running daemon
    Stop {
        /// Project name (for project-specific daemon)
        #[arg(long)]
        project: Option<String>,
    },

    /// Show daemon logs
    Logs {
        /// Project name (for project-specific daemon)
        #[arg(long)]
        project: Option<String>,

        /// Number of lines to show
        #[arg(short, long, default_value = "50")]
        lines: usize,
    },
}

#[derive(Subcommand)]
enum CoordCommands {
    /// Register a session for coordination
    Register {
        /// Session ID to register
        session_id: String,

        /// Agent type (e.g., "claude-code", "cursor")
        #[arg(short, long)]
        agent: String,

        /// Project being worked on
        #[arg(short, long)]
        project: String,

        /// Goal of the session
        #[arg(short, long)]
        goal: String,

        /// Components being worked on
        #[arg(short, long)]
        components: Option<Vec<String>>,
    },

    /// Unregister a session
    Unregister {
        /// Session ID to unregister
        session_id: String,
    },

    /// Send a heartbeat for a session
    Heartbeat {
        /// Session ID
        session_id: String,
    },

    /// Set the current file being edited
    SetFile {
        /// Session ID
        session_id: String,

        /// File path being edited (or empty to clear)
        #[arg(short, long)]
        file: Option<String>,
    },

    /// Set components being worked on
    SetComponents {
        /// Session ID
        session_id: String,

        /// Components being worked on
        #[arg(short, long)]
        components: Vec<String>,
    },

    /// Check for conflicts with other sessions
    Conflicts {
        /// Session ID to check
        session_id: String,
    },

    /// List active sessions
    List {
        /// Filter by project
        #[arg(short, long)]
        project: Option<String>,
    },

    /// Show coordination statistics
    Stats,
}

#[derive(Subcommand)]
enum MigrateCommands {
    /// Generate embeddings for all existing entities and observations
    Embeddings {
        /// Batch size for processing
        #[arg(short, long, default_value = "100")]
        batch_size: usize,
    },
}

// =========================================================================
// Layer 7: Work Management CLI Types
// =========================================================================

use engram_core::work::{PrStatus, ProjectStatus, TaskPriority, TaskStatus};

/// Project status for CLI.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum ProjectStatusArg {
    Planning,
    Active,
    Completed,
    Archived,
}

impl From<ProjectStatusArg> for ProjectStatus {
    fn from(arg: ProjectStatusArg) -> Self {
        match arg {
            ProjectStatusArg::Planning => ProjectStatus::Planning,
            ProjectStatusArg::Active => ProjectStatus::Active,
            ProjectStatusArg::Completed => ProjectStatus::Completed,
            ProjectStatusArg::Archived => ProjectStatus::Archived,
        }
    }
}

/// Task status for CLI.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum TaskStatusArg {
    Todo,
    InProgress,
    Blocked,
    Done,
}

impl From<TaskStatusArg> for TaskStatus {
    fn from(arg: TaskStatusArg) -> Self {
        match arg {
            TaskStatusArg::Todo => TaskStatus::Todo,
            TaskStatusArg::InProgress => TaskStatus::InProgress,
            TaskStatusArg::Blocked => TaskStatus::Blocked,
            TaskStatusArg::Done => TaskStatus::Done,
        }
    }
}

/// Task priority for CLI.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum TaskPriorityArg {
    Low,
    Medium,
    High,
    Critical,
}

impl From<TaskPriorityArg> for TaskPriority {
    fn from(arg: TaskPriorityArg) -> Self {
        match arg {
            TaskPriorityArg::Low => TaskPriority::Low,
            TaskPriorityArg::Medium => TaskPriority::Medium,
            TaskPriorityArg::High => TaskPriority::High,
            TaskPriorityArg::Critical => TaskPriority::Critical,
        }
    }
}

/// PR status for CLI.
#[derive(Debug, Clone, Copy, ValueEnum)]
enum PrStatusArg {
    Open,
    Merged,
    Closed,
}

impl From<PrStatusArg> for PrStatus {
    fn from(arg: PrStatusArg) -> Self {
        match arg {
            PrStatusArg::Open => PrStatus::Open,
            PrStatusArg::Merged => PrStatus::Merged,
            PrStatusArg::Closed => PrStatus::Closed,
        }
    }
}

#[derive(Subcommand)]
enum WorkCommands {
    /// Manage projects
    Project {
        #[command(subcommand)]
        command: WorkProjectCommands,
    },

    /// Manage tasks
    Task {
        #[command(subcommand)]
        command: WorkTaskCommands,
    },

    /// Manage pull requests
    Pr {
        #[command(subcommand)]
        command: WorkPrCommands,
    },

    /// Add observations to projects or tasks
    Observe {
        #[command(subcommand)]
        command: WorkObserveCommands,
    },

    /// Join a project/task work context
    Join {
        /// Project name
        project: String,

        /// Task name (optional)
        #[arg(short, long)]
        task: Option<String>,

        /// Session ID (uses coordination session if available)
        #[arg(short, long)]
        session: Option<String>,
    },

    /// Leave the current work context
    Leave {
        /// Session ID
        #[arg(short, long)]
        session: Option<String>,
    },

    /// Show current work context
    Context {
        /// Project name
        project: String,

        /// Task name (optional)
        #[arg(short, long)]
        task: Option<String>,
    },

    /// Show work statistics
    Stats,
}

#[derive(Subcommand)]
enum WorkProjectCommands {
    /// Create a new project
    Create {
        /// Project name
        name: String,

        /// Project description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// List projects
    List {
        /// Filter by status
        #[arg(short, long, value_enum)]
        status: Option<ProjectStatusArg>,
    },

    /// Show project details
    Show {
        /// Project name
        name: String,
    },

    /// Update project status
    Status {
        /// Project name
        name: String,

        /// New status
        #[arg(value_enum)]
        status: ProjectStatusArg,
    },

    /// Connect an entity to a project
    Connect {
        /// Project name
        project: String,

        /// Entity name
        entity: String,

        /// Relationship type (involves, depends_on, produces)
        #[arg(short, long, default_value = "involves")]
        relation: String,
    },
}

#[derive(Subcommand)]
enum WorkTaskCommands {
    /// Create a new task
    Create {
        /// Project name
        project: String,

        /// Task name
        name: String,

        /// Task description
        #[arg(short, long)]
        description: Option<String>,

        /// JIRA key (e.g., IDEAI-235)
        #[arg(short, long)]
        jira: Option<String>,

        /// Priority
        #[arg(short, long, value_enum, default_value = "medium")]
        priority: TaskPriorityArg,
    },

    /// List tasks for a project
    List {
        /// Project name
        project: String,

        /// Filter by status
        #[arg(short, long, value_enum)]
        status: Option<TaskStatusArg>,
    },

    /// Show task details
    Show {
        /// Task name or JIRA key
        name: String,
    },

    /// Update task status
    Status {
        /// Task name or JIRA key
        name: String,

        /// New status
        #[arg(value_enum)]
        status: TaskStatusArg,
    },

    /// Connect an entity to a task
    Connect {
        /// Task name or JIRA key
        task: String,

        /// Entity name
        entity: String,

        /// Relationship type (touches, modifies, creates)
        #[arg(short, long, default_value = "touches")]
        relation: String,
    },
}

#[derive(Subcommand)]
enum WorkPrCommands {
    /// Add a PR to a project/task
    Add {
        /// Project name
        project: String,

        /// PR URL
        url: String,

        /// Task name or JIRA key (optional)
        #[arg(short, long)]
        task: Option<String>,

        /// PR title
        #[arg(long)]
        title: Option<String>,
    },

    /// List PRs
    List {
        /// Project name
        project: String,

        /// Task name (optional)
        #[arg(short, long)]
        task: Option<String>,
    },

    /// Update PR status
    Status {
        /// PR URL
        url: String,

        /// New status
        #[arg(value_enum)]
        status: PrStatusArg,
    },
}

#[derive(Subcommand)]
enum WorkObserveCommands {
    /// Add an observation to a project
    Project {
        /// Project name
        project: String,

        /// Observation content
        content: String,

        /// Semantic key (e.g., 'architecture.auth', 'gotchas.race-conditions')
        #[arg(short, long)]
        key: Option<String>,

        /// Source of the observation
        #[arg(short, long)]
        source: Option<String>,
    },

    /// Add an observation to a task
    Task {
        /// Task name or JIRA key
        task: String,

        /// Observation content
        content: String,

        /// Semantic key
        #[arg(short, long)]
        key: Option<String>,

        /// Source of the observation
        #[arg(short, long)]
        source: Option<String>,
    },
}

/// Repository-project relationship role for CLI.
#[derive(Debug, Clone, ValueEnum)]
enum RepoRoleArg {
    Primary,
    Dependency,
    Produces,
    Related,
}

impl From<RepoRoleArg> for ProjectRepositoryRole {
    fn from(arg: RepoRoleArg) -> Self {
        match arg {
            RepoRoleArg::Primary => ProjectRepositoryRole::Primary,
            RepoRoleArg::Dependency => ProjectRepositoryRole::Dependency,
            RepoRoleArg::Produces => ProjectRepositoryRole::Produces,
            RepoRoleArg::Related => ProjectRepositoryRole::Related,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum MemoryStatusArg {
    Active,
    NeedsReview,
    Superseded,
    Archived,
    Rejected,
}

impl From<MemoryStatusArg> for MemoryStatus {
    fn from(arg: MemoryStatusArg) -> Self {
        match arg {
            MemoryStatusArg::Active => MemoryStatus::Active,
            MemoryStatusArg::NeedsReview => MemoryStatus::NeedsReview,
            MemoryStatusArg::Superseded => MemoryStatus::Superseded,
            MemoryStatusArg::Archived => MemoryStatus::Archived,
            MemoryStatusArg::Rejected => MemoryStatus::Rejected,
        }
    }
}

#[derive(Subcommand)]
enum MemoryCommands {
    /// List Memory OS records
    List {
        /// Optional lifecycle status filter
        #[arg(long)]
        status: Option<MemoryStatusArg>,

        /// Maximum memory items to return
        #[arg(short, long)]
        limit: Option<usize>,

        /// Print records as JSON
        #[arg(long)]
        json: bool,
    },

    /// Get a Memory OS record by ID
    Get {
        /// Memory item ID
        id: String,

        /// Print record as JSON
        #[arg(long)]
        json: bool,
    },

    /// List Memory OS records needing review
    Review {
        /// Maximum memory items to return
        #[arg(short, long)]
        limit: Option<usize>,

        /// Print records as JSON
        #[arg(long)]
        json: bool,
    },

    /// Print a Memory OS cursor for later changes-since calls
    Cursor {
        /// Print cursor as JSON
        #[arg(long)]
        json: bool,
    },

    /// List memory and knowledge commits written after memory_cursor.timestamp
    ChangesSince {
        /// Cursor timestamp from orient memory_cursor.timestamp or `engram memory cursor`
        #[arg(long)]
        timestamp: String,

        /// Cursor commit ID from memory_cursor.commit_id, when known; timestamp is still required
        #[arg(long)]
        commit_id: Option<String>,

        /// Maximum memory items and commits to return
        #[arg(short, long)]
        limit: Option<usize>,

        /// Filter changed memory by writer harness
        #[arg(long)]
        writer_harness: Option<String>,

        /// Filter changed memory by writer model
        #[arg(long)]
        model: Option<String>,

        /// Filter changed memory by writer surface
        #[arg(long)]
        surface: Option<String>,

        /// Filter changed memory by writer session ID
        #[arg(long)]
        writer_session_id: Option<String>,

        /// Project for relevance scoring
        #[arg(long)]
        relevance_project: Option<String>,

        /// Current working directory for relevance scoring
        #[arg(long)]
        cwd: Option<String>,

        /// Prompt/query for relevance scoring
        #[arg(long)]
        query: Option<String>,

        /// Host conversation/session label for telemetry; falls back to ENGRAM_EXTERNAL_SESSION_ID,
        /// guarded CLAUDE_CODE_SESSION_ID, then guarded CODEX_THREAD_ID
        #[arg(long)]
        external_session_id: Option<String>,

        /// Print changes as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show knowledge commit log
    Log {
        /// Maximum commits to return
        #[arg(short, long)]
        limit: Option<usize>,

        /// Print records as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show one knowledge commit and its recorded changes
    Diff {
        /// Knowledge commit ID
        commit_id: String,

        /// Print record as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show memory writer statistics
    WriterStats {
        /// Print records as JSON
        #[arg(long)]
        json: bool,
    },

    /// Archive a memory item
    Archive {
        /// Memory item ID
        id: String,

        /// Archive reason
        #[arg(long)]
        reason: String,

        /// Actor/harness archiving the item
        #[arg(long)]
        archived_by: Option<String>,

        /// Print item as JSON
        #[arg(long)]
        json: bool,
    },

    /// Export Memory OS records into a generated Markdown vault
    ExportVault {
        /// Vault root path to write
        path: String,
    },

    /// Inventory existing Engram data for future Memory OS migration without writing records
    MigrationInventory {
        /// Restrict inventory to a project name where source data supports project scoping
        #[arg(long)]
        project_filter: Option<String>,

        /// Maximum candidates to return
        #[arg(short, long)]
        limit: Option<usize>,

        /// Exclude sources already decided in generated review batches under this path
        #[arg(long)]
        exclude_reviewed_path: Option<String>,

        /// Print the full inventory as JSON
        #[arg(long)]
        json: bool,

        /// Exclude Layer 1 entity observations
        #[arg(long)]
        no_entity_observations: bool,

        /// Exclude Layer 2 session history
        #[arg(long)]
        no_session_history: bool,

        /// Exclude Layer 7 work observations
        #[arg(long)]
        no_work_observations: bool,
    },

    /// Export a generated Markdown review batch for migration candidates
    MigrationReviewExport {
        /// Output directory for generated review files
        path: String,

        /// Restrict inventory to a project name where source data supports project scoping
        #[arg(long)]
        project_filter: Option<String>,

        /// Maximum candidates to include in the review batch
        #[arg(short, long)]
        limit: Option<usize>,

        /// Exclude sources already decided in generated review batches under this path
        #[arg(long)]
        exclude_reviewed_path: Option<String>,

        /// Print the full export result as JSON
        #[arg(long)]
        json: bool,

        /// Exclude Layer 1 entity observations
        #[arg(long)]
        no_entity_observations: bool,

        /// Exclude Layer 2 session history
        #[arg(long)]
        no_session_history: bool,

        /// Exclude Layer 7 work observations
        #[arg(long)]
        no_work_observations: bool,
    },

    /// Validate a generated migration review batch without planning or writing records
    MigrationReviewStatus {
        /// Review batch directory containing index.md and candidates/
        path: String,

        /// Print the full status report as JSON
        #[arg(long)]
        json: bool,
    },

    /// Apply accepted items from a generated migration review batch
    MigrationReviewApply {
        /// Review batch directory containing index.md and candidates/
        path: String,

        /// Actually write accepted memory records; omitted means dry-run
        #[arg(long)]
        write: bool,

        /// Print the full apply report as JSON
        #[arg(long)]
        json: bool,

        /// Do not create a knowledge commit when writing accepted records
        #[arg(long)]
        no_commit: bool,

        /// Writer harness/interface recorded on migrated records
        #[arg(long, default_value = "engram_cli")]
        writer_harness: String,

        /// Model/provider label recorded on migrated records
        #[arg(long, default_value = "engram")]
        model_provider: String,

        /// Model/tool label recorded on migrated records
        #[arg(long, default_value = "migration-review-apply")]
        model: String,
    },

    /// Apply accepted items from a generated digest extraction review batch
    DigestExtractionApply {
        /// Extraction review batch directory containing index.md and candidates/
        path: String,

        /// Actually write accepted memory records; omitted means dry-run
        #[arg(long)]
        write: bool,

        /// Print the full apply report as JSON
        #[arg(long)]
        json: bool,

        /// Do not create a knowledge commit when writing accepted records
        #[arg(long)]
        no_commit: bool,

        /// Writer harness/interface recorded on imported records
        #[arg(long, default_value = "engram_cli")]
        writer_harness: String,

        /// Model/provider label recorded on imported records
        #[arg(long, default_value = "engram")]
        model_provider: String,

        /// Model/tool label recorded on imported records
        #[arg(long, default_value = "digest-extraction-apply")]
        model: String,
    },

    /// Generate review candidates from a session event stream without writing memory
    DistillSession {
        /// Session ID to distill
        session_id: String,

        /// Print result as JSON
        #[arg(long)]
        json: bool,

        /// Writer harness/interface recorded on generated candidates
        #[arg(long, default_value = "engram_cli")]
        writer_harness: String,

        /// Model/provider label recorded on generated candidates
        #[arg(long, default_value = "engram")]
        model_provider: String,

        /// Model/tool label recorded on generated candidates
        #[arg(long, default_value = "distill-session")]
        model: String,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum HarnessKindArg {
    ClaudeCode,
    Codex,
    GeminiCli,
    Cursor,
    Generic,
}

impl From<HarnessKindArg> for HarnessKind {
    fn from(value: HarnessKindArg) -> Self {
        match value {
            HarnessKindArg::ClaudeCode => Self::ClaudeCode,
            HarnessKindArg::Codex => Self::Codex,
            HarnessKindArg::GeminiCli => Self::GeminiCli,
            HarnessKindArg::Cursor => Self::Cursor,
            HarnessKindArg::Generic => Self::Generic,
        }
    }
}

fn parse_harness_settings_target(value: &str) -> Result<HarnessSettingsTarget, String> {
    HarnessSettingsTarget::parse(value)
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
enum HarnessCommands {
    /// Check whether harness adapters are present
    Status {
        /// Harness to check
        #[arg(long, value_enum, default_value = "generic")]
        harness: HarnessKindArg,

        /// Install root, defaults to home directory
        #[arg(long)]
        root: Option<String>,

        /// Print report as JSON
        #[arg(long)]
        json: bool,
    },

    /// Run harness diagnostics
    Doctor {
        /// Harness to check
        #[arg(long, value_enum, default_value = "generic")]
        harness: HarnessKindArg,

        /// Install root, defaults to home directory
        #[arg(long)]
        root: Option<String>,

        /// Print report as JSON
        #[arg(long)]
        json: bool,
    },

    /// Render the policy or one/all adapters without writing files
    Render {
        /// Harness to render
        #[arg(long, value_enum, default_value = "generic")]
        harness: HarnessKindArg,

        /// Render a specific adapter by name instead of the policy JSON
        #[arg(long)]
        adapter: Option<String>,

        /// Print adapter metadata and contents as JSON
        #[arg(long)]
        json: bool,
    },

    /// Install harness adapters. Dry-run unless --write is supplied.
    Install {
        /// Harness to install
        #[arg(long, value_enum, default_value = "generic")]
        harness: HarnessKindArg,

        /// Install root, defaults to home directory
        #[arg(long)]
        root: Option<String>,

        /// Actually write generated adapters
        #[arg(long)]
        write: bool,

        /// Back up and replace user-owned adapters; only active with --write
        #[arg(long)]
        adopt_user_owned: bool,

        /// Claude settings target: settings.json, settings.local.json, or snippet-only
        #[arg(long, default_value = "settings.json", value_parser = parse_harness_settings_target)]
        settings_target: HarnessSettingsTarget,

        /// Print report as JSON
        #[arg(long)]
        json: bool,
    },

    /// Handle one agent hook event and print hook JSON
    Hook {
        /// Harness handling the hook
        #[arg(long, value_enum, default_value = "claude-code")]
        harness: HarnessKindArg,

        /// Hook event name, e.g. UserPromptSubmit, PostToolUseFailure, Stop
        #[arg(long)]
        event: String,

        /// Claude session ID
        #[arg(long)]
        session_id: Option<String>,

        /// Current working directory
        #[arg(long)]
        cwd: Option<String>,

        /// Transcript path
        #[arg(long)]
        transcript_path: Option<String>,

        /// User prompt
        #[arg(long)]
        prompt: Option<String>,

        /// Tool name
        #[arg(long)]
        tool_name: Option<String>,

        /// Tool error
        #[arg(long)]
        tool_error: Option<String>,

        /// Tool input command
        #[arg(long)]
        tool_input_command: Option<String>,

        /// File path touched by a tool
        #[arg(long)]
        file_path: Option<String>,

        /// Last assistant message
        #[arg(long)]
        last_assistant_message: Option<String>,

        /// Compact summary
        #[arg(long)]
        compact_summary: Option<String>,

        /// Hook trigger/matcher
        #[arg(long)]
        trigger: Option<String>,

        /// Session end or permission reason
        #[arg(long)]
        reason: Option<String>,

        /// Whether Stop is already active
        #[arg(long)]
        stop_hook_active: bool,

        /// Hook write policy: durable or nudge
        #[arg(long, default_value = "durable")]
        write_policy: String,

        /// Project scope override
        #[arg(long)]
        project: Option<String>,

        /// Writer model provider
        #[arg(long, default_value = "anthropic")]
        model_provider: String,

        /// Writer model
        #[arg(long, default_value = "claude-code")]
        model: String,

        /// Surface label
        #[arg(long, default_value = "claude-code")]
        surface: String,

        /// Actor label
        #[arg(long, default_value = "agent")]
        actor: String,

        /// Store project scope for CLI database access
        #[arg(long = "store-project")]
        store_project: Option<String>,

        /// Override data directory
        #[arg(long)]
        data_dir: Option<String>,
    },
}

#[derive(Subcommand)]
enum LintCommands {
    /// Run lint checks
    Run {
        /// Optional Memory OS vault root to scan
        #[arg(long)]
        vault_path: Option<String>,

        /// Maximum findings to print
        #[arg(short, long)]
        limit: Option<usize>,

        /// Print report as JSON
        #[arg(long)]
        json: bool,
    },

    /// Alias for run
    List {
        /// Optional Memory OS vault root to scan
        #[arg(long)]
        vault_path: Option<String>,

        /// Maximum findings to print
        #[arg(short, long)]
        limit: Option<usize>,

        /// Print report as JSON
        #[arg(long)]
        json: bool,
    },

    /// Apply safe lint actions only
    ApplySafe {
        /// Optional Memory OS vault root to scan
        #[arg(long)]
        vault_path: Option<String>,

        /// Maximum findings to inspect
        #[arg(short, long)]
        limit: Option<usize>,

        /// Actually write safe actions. Omitted means dry-run.
        #[arg(long)]
        write: bool,

        /// Print report as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum GraphCommands {
    /// Return a subgraph around a node ID
    Around {
        /// Node ID. Plain UUIDs are treated as `memory:<id>`.
        node: String,

        /// Traversal depth
        #[arg(short, long, default_value = "2")]
        depth: usize,

        /// Print graph as JSON
        #[arg(long)]
        json: bool,
    },

    /// Find a graph path between two node IDs
    Path {
        /// Start node ID
        from: String,

        /// End node ID
        to: String,

        /// Maximum traversal depth
        #[arg(long, default_value = "6")]
        max_depth: usize,

        /// Print path as JSON
        #[arg(long)]
        json: bool,
    },

    /// Return the full graph or a bounded graph around a node
    Subgraph {
        /// Optional start node ID
        #[arg(long)]
        node: Option<String>,

        /// Traversal depth when --node is supplied
        #[arg(short, long, default_value = "2")]
        depth: usize,

        /// Print graph as JSON
        #[arg(long)]
        json: bool,
    },

    /// Export graph as Mermaid
    Export {
        /// Optional start node ID
        #[arg(long)]
        node: Option<String>,

        /// Traversal depth when --node is supplied
        #[arg(short, long, default_value = "2")]
        depth: usize,
    },
}

#[derive(Subcommand)]
enum HandoffCommands {
    /// Get the latest active handoff
    Get {
        /// Project scope
        #[arg(long)]
        project: Option<String>,

        /// Session scope
        #[arg(long)]
        session_id: Option<String>,

        /// Print result as JSON
        #[arg(long)]
        json: bool,
    },

    /// Update the rolling handoff
    Update {
        /// Project scope
        #[arg(long)]
        project: Option<String>,

        /// Session scope
        #[arg(long)]
        session_id: Option<String>,

        /// Handoff Markdown content
        content: String,

        /// Next action line; may be repeated
        #[arg(long = "next-action")]
        next_actions: Vec<String>,

        /// Actually write the handoff. Omitted means dry-run.
        #[arg(long)]
        write: bool,

        /// Print result as JSON
        #[arg(long)]
        json: bool,

        /// Writer harness/interface recorded on the handoff
        #[arg(long, default_value = "engram_cli")]
        writer_harness: String,

        /// Model/provider label recorded on the handoff
        #[arg(long, default_value = "engram")]
        model_provider: String,

        /// Model/tool label recorded on the handoff
        #[arg(long, default_value = "handoff-update")]
        model: String,
    },

    /// Compile a handoff from session events
    Compile {
        /// Session ID
        session_id: String,

        /// Project scope for the written handoff
        #[arg(long)]
        project: Option<String>,

        /// Actually write the compiled handoff. Omitted means dry-run.
        #[arg(long)]
        write: bool,

        /// Print result as JSON
        #[arg(long)]
        json: bool,

        /// Writer harness/interface recorded on the handoff
        #[arg(long, default_value = "engram_cli")]
        writer_harness: String,

        /// Model/provider label recorded on the handoff
        #[arg(long, default_value = "engram")]
        model_provider: String,

        /// Model/tool label recorded on the handoff
        #[arg(long, default_value = "handoff-compile")]
        model: String,
    },
}

#[derive(Subcommand)]
enum ObligationCommands {
    /// Detect obligations from prompt and git status. Dry-run unless --write is supplied.
    Detect {
        /// Current working directory, defaults to the process cwd
        #[arg(long)]
        cwd: Option<String>,

        /// Prompt or task text used for source/design/tool-failure cues
        #[arg(long)]
        prompt: Option<String>,

        /// Project scope for generated obligations
        #[arg(long)]
        scope_project: Option<String>,

        /// Actually write detected obligations. Omitted means dry-run.
        #[arg(long)]
        write: bool,

        /// Maximum candidate obligations
        #[arg(short, long)]
        limit: Option<usize>,

        /// Print result as JSON
        #[arg(long)]
        json: bool,

        /// Writer harness/interface recorded on generated obligations
        #[arg(long, default_value = "engram_cli")]
        writer_harness: String,

        /// Model/provider label recorded on generated obligations
        #[arg(long, default_value = "engram")]
        model_provider: String,

        /// Model/tool label recorded on generated obligations
        #[arg(long, default_value = "obligation-detect")]
        model: String,
    },

    /// Add an explicit obligation
    Add {
        /// Obligation kind
        #[arg(long)]
        kind: String,

        /// Short title
        #[arg(long)]
        title: String,

        /// Obligation details
        #[arg(long)]
        description: String,

        /// Project scope
        #[arg(long)]
        scope_project: Option<String>,

        /// Trigger kind
        #[arg(long, default_value = "agent_decision")]
        trigger_kind: String,

        /// Trigger summary
        #[arg(long)]
        trigger_summary: String,

        /// Optional trigger target
        #[arg(long)]
        trigger_target: Option<String>,

        /// Expected resolution; may be repeated
        #[arg(long = "required-resolution")]
        required_resolutions: Vec<String>,

        /// Print result as JSON
        #[arg(long)]
        json: bool,

        /// Writer harness/interface recorded on generated obligations
        #[arg(long, default_value = "engram_cli")]
        writer_harness: String,

        /// Model/provider label recorded on generated obligations
        #[arg(long, default_value = "engram")]
        model_provider: String,

        /// Model/tool label recorded on generated obligations
        #[arg(long, default_value = "obligation-add")]
        model: String,
    },

    /// List obligations
    List {
        /// Optional status filter: open, resolved, skipped
        #[arg(long)]
        status: Option<String>,

        /// Maximum obligations to print
        #[arg(short, long)]
        limit: Option<usize>,

        /// Print result as JSON
        #[arg(long)]
        json: bool,
    },

    /// Run obligation doctor checks
    Doctor {
        /// Maximum open obligations to inspect
        #[arg(short, long)]
        limit: Option<usize>,

        /// Print result as JSON
        #[arg(long)]
        json: bool,
    },

    /// Resolve an obligation
    Resolve {
        /// Obligation ID
        id: String,

        /// Resolution kind
        #[arg(long)]
        resolution: String,

        /// Resolution summary
        #[arg(long)]
        summary: String,

        /// Actor resolving the obligation
        #[arg(long, default_value = "agent")]
        actor: String,

        /// Print result as JSON
        #[arg(long)]
        json: bool,
    },

    /// Skip an obligation with an explicit reason
    Skip {
        /// Obligation ID
        id: String,

        /// Skip reason
        #[arg(long)]
        reason: String,

        /// Actor skipping the obligation
        #[arg(long, default_value = "agent")]
        actor: String,

        /// Print result as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum VaultCommands {
    /// Create the generated vault directory skeleton
    Init {
        /// Vault root path
        path: String,

        /// Print result as JSON
        #[arg(long)]
        json: bool,
    },

    /// Compile Memory OS records into generated Markdown pages
    Compile {
        /// Vault root path
        path: String,

        /// Print result as JSON
        #[arg(long)]
        json: bool,
    },

    /// Inspect a vault without writing files
    Status {
        /// Vault root path
        path: String,

        /// Print result as JSON
        #[arg(long)]
        json: bool,
    },

    /// Print a page from the vault
    Page {
        /// Vault root path
        path: String,

        /// Page path relative to the vault root
        page: String,

        /// Print result as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum DigestCommands {
    /// Inventory digest-like source files without reading contents or writing memory
    Inventory {
        /// Root directory to scan, such as ~/notes
        root_path: String,

        /// Maximum candidate digest files to return
        #[arg(short, long)]
        limit: Option<usize>,

        /// Include files normally treated as operational artifacts
        #[arg(long)]
        include_operational: bool,

        /// Print inventory as JSON
        #[arg(long)]
        json: bool,
    },

    /// Export metadata-only digest review files without reading contents
    ReviewExport {
        /// Root directory to scan, such as ~/notes
        root_path: String,

        /// Output directory for generated review files
        output_path: String,

        /// Maximum candidate digest files to return
        #[arg(short, long)]
        limit: Option<usize>,

        /// Include files normally treated as operational artifacts
        #[arg(long)]
        include_operational: bool,

        /// Print export result as JSON
        #[arg(long)]
        json: bool,
    },

    /// Parse human decisions from a generated digest review batch
    ReviewApply {
        /// Review batch directory containing index.md and candidates/
        path: String,

        /// Print apply report as JSON
        #[arg(long)]
        json: bool,
    },

    /// Build review-gated candidate memory excerpts from accepted digest sources
    ExtractionPlan {
        /// Review batch directory containing accepted digest source decisions
        review_path: String,

        /// Output directory for generated extraction review files
        output_path: String,

        /// Maximum bytes to read from any accepted source
        #[arg(long)]
        max_source_bytes: Option<usize>,

        /// Maximum candidate memory excerpts per accepted source
        #[arg(long)]
        max_candidates_per_source: Option<usize>,

        /// Maximum characters copied into each generated candidate excerpt
        #[arg(long)]
        max_candidate_chars: Option<usize>,

        /// Print extraction plan as JSON
        #[arg(long)]
        json: bool,
    },

    /// Plan or write source-only digest evidence into the document index
    SourceIndex {
        /// Review batch directory containing source_only digest source decisions
        review_path: String,

        /// Actually index source-only digest documents; omitted means dry-run
        #[arg(long)]
        write: bool,

        /// Use a project-specific Engram data store when writing
        #[arg(long)]
        project: Option<String>,

        /// Use an explicit RocksDB data directory when writing
        #[arg(long)]
        data_dir: Option<String>,

        /// Maximum bytes to read from any source-only digest
        #[arg(long)]
        max_source_bytes: Option<usize>,

        /// Print source index plan/result as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum RepoCommands {
    /// Detect the Git checkout at or above cwd and register it
    Detect {
        /// Current working directory, defaults to the process cwd
        cwd: Option<String>,
    },

    /// Resolve repository context for a cwd
    Context {
        /// Current working directory, defaults to the process cwd
        cwd: Option<String>,
    },

    /// Register or update a canonical repository
    Register {
        /// Repository name
        name: String,

        /// Canonical remote URL
        #[arg(long)]
        remote: Option<String>,

        /// Default branch
        #[arg(long)]
        default_branch: Option<String>,

        /// Description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// List known repositories
    List {
        /// Maximum number of repositories
        #[arg(short, long)]
        limit: Option<usize>,
    },

    /// Add or update a monorepo component
    ComponentAdd {
        /// Repository name
        #[arg(long)]
        repo: Option<String>,

        /// Repository ID
        #[arg(long)]
        repo_id: Option<String>,

        /// Component name
        name: String,

        /// Repository-relative component path
        path: String,

        /// Component kind, such as service, app, package, or crate
        #[arg(long)]
        kind: Option<String>,

        /// Description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Link a project to a repository or monorepo component
    LinkProject {
        /// Project name
        project: String,

        /// Repository name
        #[arg(long)]
        repo: Option<String>,

        /// Repository ID
        #[arg(long)]
        repo_id: Option<String>,

        /// Relationship role
        #[arg(long, value_enum, default_value = "related")]
        role: RepoRoleArg,

        /// Optional repository-relative component path
        #[arg(long)]
        component_path: Option<String>,
    },

    /// Inventory legacy Engram data for repository topology references without writing records
    MigrationInventory {
        /// Restrict inventory to a project name where source data supports project scoping
        #[arg(long)]
        project_filter: Option<String>,

        /// Maximum candidates to return
        #[arg(short, long)]
        limit: Option<usize>,

        /// Print the full inventory as JSON
        #[arg(long)]
        json: bool,

        /// Exclude Layer 1 entity descriptions and observations
        #[arg(long)]
        no_entity_observations: bool,

        /// Exclude Layer 2 session history
        #[arg(long)]
        no_session_history: bool,

        /// Exclude Layer 7 work records
        #[arg(long)]
        no_work_records: bool,
    },

    /// Export a generated Markdown review batch for repository topology migration
    MigrationReviewExport {
        /// Output directory for generated review files
        path: String,

        /// Restrict inventory to a project name where source data supports project scoping
        #[arg(long)]
        project_filter: Option<String>,

        /// Maximum candidates to include in the review batch
        #[arg(short, long)]
        limit: Option<usize>,

        /// Print the full export result as JSON
        #[arg(long)]
        json: bool,

        /// Exclude Layer 1 entity descriptions and observations
        #[arg(long)]
        no_entity_observations: bool,

        /// Exclude Layer 2 session history
        #[arg(long)]
        no_session_history: bool,

        /// Exclude Layer 7 work records
        #[arg(long)]
        no_work_records: bool,
    },

    /// Validate a generated repository migration review batch without writing records
    MigrationReviewStatus {
        /// Review batch directory containing index.md and candidates/
        path: String,

        /// Print the full status report as JSON
        #[arg(long)]
        json: bool,
    },

    /// Apply accepted topology records from a generated repository migration review batch
    MigrationReviewApply {
        /// Review batch directory containing index.md and candidates/
        path: String,

        /// Actually write accepted repository topology records; omitted means dry-run
        #[arg(long)]
        write: bool,

        /// Print the full apply report as JSON
        #[arg(long)]
        json: bool,

        /// Do not create a knowledge commit when writing accepted topology records
        #[arg(long)]
        no_commit: bool,

        /// Writer harness/interface recorded on migration audit commits
        #[arg(long, default_value = "engram_cli")]
        writer_harness: String,

        /// Model/provider label recorded on migration audit commits
        #[arg(long, default_value = "engram")]
        model_provider: String,

        /// Model/tool label recorded on migration audit commits
        #[arg(long, default_value = "repository-migration-review-apply")]
        model: String,
    },
}

fn scoped_store_config(project: Option<&str>, data_dir: Option<&str>) -> Result<StoreConfig> {
    if let Some(data_dir) = data_dir {
        return Ok(StoreConfig::rocksdb(data_dir));
    }

    if let Some(project) = project {
        let base = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".engram")
            .join("projects")
            .join(project)
            .join("data");
        std::fs::create_dir_all(&base)?;
        return Ok(StoreConfig::rocksdb(base));
    }

    Ok(StoreConfig::rocksdb(StoreConfig::default_data_dir()))
}

async fn handle_harness_hook_via_daemon(
    hook_event: &HarnessHookEvent,
    store_project: Option<&str>,
) -> Result<Option<serde_json::Value>> {
    let daemon_config = match store_project {
        Some(project) => daemon::DaemonConfig::project(project),
        None => daemon::DaemonConfig::global(),
    };
    let info = match daemon::get_daemon_info(&daemon_config).await {
        Ok(info) if info.healthy => info,
        _ => return Ok(None),
    };

    let arguments = harness_hook_daemon_arguments(hook_event);
    let response = proxy::call_tool_once(info.port, "harness", arguments).await?;
    extract_mcp_tool_json_text(response).map(Some)
}

fn harness_hook_daemon_arguments(hook_event: &HarnessHookEvent) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert("action".to_string(), serde_json::json!("hook_event"));
    map.insert(
        "harness".to_string(),
        serde_json::json!(hook_event.harness.to_string()),
    );
    map.insert(
        "hook_event_name".to_string(),
        serde_json::json!(hook_event.hook_event_name),
    );
    map.insert(
        "stop_hook_active".to_string(),
        serde_json::json!(hook_event.stop_hook_active.to_string()),
    );
    insert_optional_string(&mut map, "session_id", &hook_event.session_id);
    insert_optional_string(&mut map, "cwd", &hook_event.cwd);
    insert_optional_string(&mut map, "transcript_path", &hook_event.transcript_path);
    insert_optional_string(&mut map, "prompt", &hook_event.prompt);
    insert_optional_string(&mut map, "tool_name", &hook_event.tool_name);
    insert_optional_string(&mut map, "tool_error", &hook_event.tool_error);
    insert_optional_string(
        &mut map,
        "tool_input_command",
        &hook_event.tool_input_command,
    );
    insert_optional_string(&mut map, "file_path", &hook_event.file_path);
    insert_optional_string(
        &mut map,
        "last_assistant_message",
        &hook_event.last_assistant_message,
    );
    insert_optional_string(&mut map, "compact_summary", &hook_event.compact_summary);
    insert_optional_string(&mut map, "trigger", &hook_event.trigger);
    insert_optional_string(&mut map, "reason", &hook_event.reason);
    insert_optional_string(&mut map, "write_policy", &hook_event.write_policy);
    insert_optional_string(&mut map, "project", &hook_event.project);
    insert_optional_string(&mut map, "model_provider", &hook_event.model_provider);
    insert_optional_string(&mut map, "model", &hook_event.model);
    insert_optional_string(&mut map, "surface", &hook_event.surface);
    insert_optional_string(&mut map, "actor", &hook_event.actor);
    serde_json::Value::Object(map)
}

fn insert_optional_string(
    map: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: &Option<String>,
) {
    if let Some(value) = value {
        map.insert(key.to_string(), serde_json::json!(value));
    }
}

fn extract_mcp_tool_json_text(response: serde_json::Value) -> Result<serde_json::Value> {
    if let Some(error) = response.get("error") {
        return Err(anyhow::anyhow!("daemon MCP tool error: {}", error));
    }

    let text = response
        .pointer("/result/content/0/text")
        .and_then(|value| value.as_str())
        .ok_or_else(|| anyhow::anyhow!("daemon MCP tool response did not include text content"))?;
    serde_json::from_str(text)
        .map_err(|error| anyhow::anyhow!("daemon MCP tool response was not JSON: {}", error))
}

fn validate_serve_options(
    memory: bool,
    remote: Option<&str>,
    username: Option<&str>,
    password: Option<&str>,
    http: bool,
    port: Option<u16>,
) -> Result<()> {
    if memory && remote.is_some() {
        anyhow::bail!("--memory and --remote cannot be used together");
    }
    if remote.is_none() && (username.is_some() || password.is_some()) {
        anyhow::bail!("--username and --password require --remote");
    }
    if !http {
        if memory {
            anyhow::bail!(
                "--memory is only honored with --http; use `engram serve --http --memory` \
                 or omit --memory for the default persistent stdio proxy"
            );
        }
        if remote.is_some() || username.is_some() || password.is_some() {
            anyhow::bail!(
                "--remote/--username/--password are only honored with --http; use \
                 `engram serve --http --remote ...` or omit them for the default stdio proxy"
            );
        }
        if port.is_some() {
            anyhow::bail!(
                "--port is only honored with --http; omit it for the default stdio proxy"
            );
        }
    }
    Ok(())
}

fn cwd_or_current(cwd: Option<String>) -> Result<std::path::PathBuf> {
    cwd.map(std::path::PathBuf::from)
        .map(Ok)
        .unwrap_or_else(std::env::current_dir)
        .map_err(Into::into)
}

fn external_session_id_from_cli(cli_value: Option<String>) -> Option<String> {
    resolve_external_session_id_with_envs(
        cli_value,
        std::env::var(EXTERNAL_SESSION_ID_ENV).ok(),
        std::env::var(CLAUDE_CODE_SESSION_ID_ENV).ok(),
        claude_code_host_marker_from_env(),
        std::env::var(CODEX_THREAD_ID_ENV).ok(),
        codex_host_marker_from_env(),
    )
}

#[cfg(test)]
fn resolve_external_session_id(
    cli_value: Option<String>,
    env_value: Option<String>,
) -> Option<String> {
    resolve_external_session_id_with_envs(cli_value, env_value, None, false, None, false)
}

fn resolve_external_session_id_with_envs(
    cli_value: Option<String>,
    env_value: Option<String>,
    claude_code_session_id: Option<String>,
    claude_code_host_detected: bool,
    codex_thread_id: Option<String>,
    codex_host_detected: bool,
) -> Option<String> {
    match cli_value {
        Some(value) => normalize_external_session_id(Some(value)),
        None => normalize_external_session_id(env_value)
            .or_else(|| {
                claude_code_session_external_session_id(
                    claude_code_session_id,
                    claude_code_host_detected,
                )
            })
            .or_else(|| codex_thread_external_session_id(codex_thread_id, codex_host_detected)),
    }
}

fn normalize_external_session_id(value: Option<String>) -> Option<String> {
    let value = value?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn claude_code_host_marker_from_env() -> bool {
    claude_code_host_marker_present(std::env::var(CLAUDE_CODE_MARKER_ENV).ok())
}

fn claude_code_host_marker_present(claudecode_marker: Option<String>) -> bool {
    normalize_external_session_id(claudecode_marker)
        .map(|value| value == "1")
        .unwrap_or(false)
}

fn codex_host_marker_from_env() -> bool {
    codex_host_marker_present(
        std::env::var(CODEX_SHELL_ENV).ok(),
        std::env::var(CODEX_ORIGINATOR_ENV).ok(),
        std::env::var(CODEX_BUNDLE_ID_ENV).ok(),
    )
}

fn codex_host_marker_present(
    codex_shell: Option<String>,
    codex_originator: Option<String>,
    bundle_id: Option<String>,
) -> bool {
    normalize_external_session_id(codex_shell).is_some()
        || normalize_external_session_id(codex_originator)
            .map(|value| value.to_ascii_lowercase().contains("codex"))
            .unwrap_or(false)
        || normalize_external_session_id(bundle_id)
            .map(|value| value == "com.openai.codex")
            .unwrap_or(false)
}

fn codex_thread_external_session_id(
    codex_thread_id: Option<String>,
    codex_host_detected: bool,
) -> Option<String> {
    // Codex Desktop exposes a host thread ID; require a Codex marker to avoid generic env leakage.
    if !codex_host_detected {
        return None;
    }

    let thread_id = safe_host_session_token(codex_thread_id, MAX_CODEX_THREAD_ID_LEN)?;

    Some(format!("{CODEX_THREAD_EXTERNAL_SESSION_PREFIX}{thread_id}"))
}

fn claude_code_session_external_session_id(
    claude_code_session_id: Option<String>,
    claude_code_host_detected: bool,
) -> Option<String> {
    // Claude Code exposes its session ID to MCP/Bash subprocesses; require its subprocess marker.
    if !claude_code_host_detected {
        return None;
    }

    let session_id =
        safe_host_session_token(claude_code_session_id, MAX_CLAUDE_CODE_SESSION_ID_LEN)?;

    Some(format!("{CLAUDE_CODE_EXTERNAL_SESSION_PREFIX}{session_id}"))
}

fn safe_host_session_token(value: Option<String>, max_len: usize) -> Option<String> {
    let token = normalize_external_session_id(value)?;
    if token.len() > max_len
        || !token
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return None;
    }

    Some(token)
}

fn parse_optional_repo_id(repo_id: Option<&str>) -> Result<Option<Id>> {
    repo_id
        .map(Id::parse)
        .transpose()
        .map_err(|e| anyhow::anyhow!("Invalid repository ID: {}", e))
}

fn parse_rfc3339_timestamp(value: &str) -> Result<OffsetDateTime> {
    OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339).map_err(|e| {
        anyhow::anyhow!(
            "Invalid RFC3339 timestamp: {}. Pass memory_cursor.timestamp from orient or \
                 `engram memory cursor`.",
            e
        )
    })
}

fn format_rfc3339_timestamp(value: OffsetDateTime) -> Result<String> {
    value
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| anyhow::anyhow!("Invalid timestamp: {}", e))
}

fn print_orientation_packet(packet: &OrientationPacket) {
    println!("{}", packet.context_pack);
}

fn print_harness_status(report: &HarnessStatusReport) {
    println!("Harness: {}", report.harness);
    println!("Root:    {}", report.root);
    println!("Ready:   {}", report.ready);
    println!("Adapters:");
    for adapter in &report.adapters {
        let marker = match adapter.status {
            HarnessAdapterStatus::Installed => "installed",
            HarnessAdapterStatus::Missing => "missing",
            HarnessAdapterStatus::Drifted => "drifted",
            HarnessAdapterStatus::UserOwned => "user-owned",
        };
        println!("  - {} [{}] {}", adapter.name, marker, adapter.path);
    }
    if !report.missing_mcp_tools.is_empty() {
        println!("Missing MCP tools: {}", report.missing_mcp_tools.join(", "));
    }
    if !report.settings.is_empty() {
        println!("Settings:");
        for check in &report.settings {
            let locations = if check.locations.is_empty() {
                "missing".to_string()
            } else {
                check.locations.join(", ")
            };
            println!("  - {} [{}] {}", check.name, check.kind, locations);
        }
    }
    if !report.warnings.is_empty() {
        println!("Warnings:");
        for warning in &report.warnings {
            println!("  - {warning}");
        }
    }
}

fn print_harness_install(report: &HarnessInstallReport) {
    println!("Harness install: {}", report.harness);
    println!("Root:            {}", report.root);
    println!("Dry-run:         {}", report.dry_run);
    println!("Planned files:   {}", report.planned.len());
    println!("Written files:   {}", report.written.len());
    println!("Skipped files:   {}", report.skipped.len());

    if !report.written.is_empty() {
        println!("Written:");
        for file in &report.written {
            println!("  - {}", file.path);
        }
    }
    if !report.skipped.is_empty() {
        println!("Skipped:");
        for file in &report.skipped {
            println!("  - {} ({})", file.path, file.message);
        }
    }
    if !report.warnings.is_empty() {
        println!("Warnings:");
        for warning in &report.warnings {
            println!("  - {warning}");
        }
    }
}

fn print_obligation_detection(detection: &ObligationDetection) {
    println!("Obligation detection");
    println!("  Dry-run:           {}", detection.dry_run);
    println!("  Candidates:        {}", detection.candidates.len());
    println!("  Written:           {}", detection.written.len());
    println!("  Skipped existing:  {}", detection.skipped_existing.len());
    if !detection.candidates.is_empty() {
        println!("Candidates:");
        for obligation in &detection.candidates {
            println!("  - [{}] {}", obligation.kind, obligation.title);
            if let Some(target) = &obligation.trigger.target {
                println!("    target: {target}");
            }
        }
    }
    if !detection.warnings.is_empty() {
        println!("Warnings:");
        for warning in &detection.warnings {
            println!("  - {warning}");
        }
    }
}

fn print_obligation_list(obligations: &[AgentObligation]) {
    println!("Obligations: {}", obligations.len());
    for obligation in obligations {
        print_obligation(obligation);
    }
}

fn print_obligation(obligation: &AgentObligation) {
    println!(
        "- [{}] {} ({})",
        obligation.status, obligation.title, obligation.kind
    );
    println!("  id: {}", obligation.id);
    println!("  {}", obligation.description);
    println!("  trigger: {}", obligation.trigger.summary);
    if let Some(target) = &obligation.trigger.target {
        println!("  target: {target}");
    }
    if !obligation.required_resolution.is_empty() {
        let resolutions: Vec<_> = obligation
            .required_resolution
            .iter()
            .map(ToString::to_string)
            .collect();
        println!("  required: {}", resolutions.join(", "));
    }
    if let Some(resolution) = &obligation.resolution {
        println!("  resolution: {} - {}", resolution.kind, resolution.summary);
    }
}

fn print_obligation_doctor(report: &ObligationDoctorReport) {
    println!("Open obligations: {}", report.open.len());
    for warning in &report.warnings {
        println!("  - {warning}");
    }
}

fn print_lint_report(report: &LintReport, dry_run: bool) {
    println!("Lint findings: {}", report.findings.len());
    if report.applied_safe_actions > 0 || dry_run {
        println!("Safe actions applied: {}", report.applied_safe_actions);
        if dry_run {
            println!("Dry-run: safe actions were not written");
        }
    }
    for finding in &report.findings {
        print_lint_finding(finding);
    }
}

fn print_lint_finding(finding: &LintFinding) {
    let severity = match finding.severity {
        LintSeverity::Info => "info",
        LintSeverity::Warning => "warning",
        LintSeverity::Error => "error",
    };
    println!("- [{}] {}: {}", severity, finding.rule, finding.title);
    println!("  {}", finding.message);
    if let Some(item_id) = finding.item_id {
        println!("  item: {item_id}");
    }
    if let Some(session_id) = finding.session_id {
        println!("  session: {session_id}");
    }
    if let Some(obligation_id) = finding.obligation_id {
        println!("  obligation: {obligation_id}");
    }
    if let Some(path) = &finding.path {
        println!("  path: {path}");
    }
}

fn print_subgraph(graph: &MemorySubgraph) {
    println!("Nodes: {}", graph.nodes.len());
    for node in &graph.nodes {
        println!("  - {} [{}] {}", node.id, node.kind, node.label);
    }
    println!("Edges: {}", graph.edges.len());
    for edge in &graph.edges {
        println!("  - {} --{}--> {}", edge.from, edge.relation, edge.to);
    }
}

fn print_repository_context(context: &RepositoryContext) {
    println!("Repository:");
    println!("  ID:   {}", context.repository.id);
    println!("  Name: {}", context.repository.name);
    if let Some(remote_url) = &context.repository.remote_url {
        println!("  Remote: {}", remote_url);
    }
    println!("  Provider: {}", context.repository.provider);
    if let Some(default_branch) = &context.repository.default_branch {
        println!("  Default branch: {}", default_branch);
    }

    if let Some(checkout) = &context.checkout {
        println!("Checkout:");
        println!("  Path: {}", checkout.local_path);
        if let Some(branch) = &checkout.current_branch {
            println!("  Branch: {}", branch);
        }
        if let Some(head_sha) = &checkout.head_sha {
            println!("  HEAD: {}", head_sha);
        }
        if let Some(is_dirty) = checkout.is_dirty {
            println!("  Dirty: {}", is_dirty);
        }
    }

    println!("Components:");
    if context.matching_components.is_empty() {
        println!("  none");
    } else {
        for component in &context.matching_components {
            let kind = component.kind.as_deref().unwrap_or("unknown");
            println!("  {} ({}, {})", component.name, component.path, kind);
        }
    }

    println!("Linked projects:");
    if context.linked_projects.is_empty() {
        println!("  none");
    } else {
        for link in &context.linked_projects {
            let component = link
                .component_path
                .as_deref()
                .map(|path| format!(" component={path}"))
                .unwrap_or_default();
            println!("  {} ({}){}", link.project_name, link.role, component);
        }
    }
}

fn print_memory_items(title: &str, items: &[MemoryItem]) {
    println!("{} ({})", title, items.len());
    if items.is_empty() {
        println!("  none");
        return;
    }

    for item in items {
        println!("  {} [{}; {}]", item.title, item.kind, item.status);
        println!("    ID:      {}", item.id);
        println!("    Scope:   {}", memory_scope_label(&item.scope));
        println!("    Origin:  {:?}", item.origin);
        println!(
            "    Writer:  {} / {}",
            item.writer.harness, item.writer.model.model
        );
        if !item.tags.is_empty() {
            println!("    Tags:    {}", item.tags.join(", "));
        }
        println!("    Content: {}", item.content.replace('\n', " "));
    }
}

fn print_memory_item(item: &MemoryItem) {
    print_memory_items("Memory item", std::slice::from_ref(item));
}

fn print_memory_cursor(cursor: &MemoryCursor) {
    println!("Memory cursor");
    let timestamp =
        format_rfc3339_timestamp(cursor.timestamp).unwrap_or_else(|_| cursor.timestamp.to_string());
    println!("  Timestamp: {}", timestamp);
    if let Some(commit_id) = cursor.commit_id {
        println!("  Latest commit: {}", commit_id);
    } else {
        println!("  Latest commit: none");
    }
}

fn print_memory_changes(changes: &MemoryChanges) {
    println!("Memory changes");
    let since_timestamp = format_rfc3339_timestamp(changes.since.timestamp)
        .unwrap_or_else(|_| changes.since.timestamp.to_string());
    let next_timestamp = format_rfc3339_timestamp(changes.next_cursor.timestamp)
        .unwrap_or_else(|_| changes.next_cursor.timestamp.to_string());
    println!("  Since timestamp: {}", since_timestamp);
    if let Some(commit_id) = changes.since.commit_id {
        println!("  Since commit:    {}", commit_id);
    }
    println!("  Next timestamp:  {}", next_timestamp);
    if let Some(commit_id) = changes.next_cursor.commit_id {
        println!("  Next commit:     {}", commit_id);
    }
    if let Some(trace_id) = changes.trace_id {
        println!("  Trace ID:        {}", trace_id);
    }
    println!("  Memory items:    {}", changes.items.len());
    println!("  Commits:         {}", changes.commits.len());

    if !changes.items.is_empty() {
        print_memory_items("Changed memory items", &changes.items);
        if !changes.item_relevance.is_empty() {
            println!("Relevance");
            for relevance in &changes.item_relevance {
                println!(
                    "  {} score {:.2}: {}",
                    relevance.item_id,
                    relevance.score,
                    relevance.reasons.join(", ")
                );
            }
        }
    }
    if !changes.commits.is_empty() {
        println!("Knowledge commits");
        for commit in &changes.commits {
            println!("  {} - {}", commit.id, commit.message);
        }
    }
}

fn print_digest_inventory(inventory: &DigestInventory) {
    println!("Digest source inventory");
    println!("  Root:                {}", inventory.root_path);
    println!("  Files scanned:       {}", inventory.files_scanned);
    println!("  Total candidates:    {}", inventory.total_candidates);
    println!("  Returned candidates: {}", inventory.returned_candidates);
    println!("  Truncated:           {}", inventory.truncated);
    println!("  Excluded files:      {}", inventory.excluded_count);

    if !inventory.by_source_kind.is_empty() {
        println!("By source kind:");
        for (kind, count) in &inventory.by_source_kind {
            println!("  - {}: {}", kind, count);
        }
    }
    if !inventory.by_format.is_empty() {
        println!("By format:");
        for (format, count) in &inventory.by_format {
            println!("  - {}: {}", format, count);
        }
    }

    if inventory.candidates.is_empty() {
        println!("Candidates: none");
    } else {
        println!("Candidates:");
        for candidate in &inventory.candidates {
            let bucket = candidate
                .bucket
                .as_deref()
                .map(|bucket| format!(" bucket={bucket}"))
                .unwrap_or_default();
            let date = candidate
                .date_hint
                .as_deref()
                .map(|date| format!(" date={date}"))
                .unwrap_or_default();
            println!(
                "  - {} [{}; {}; {}{}{}]",
                candidate.relative_path,
                candidate.source_kind,
                candidate.format,
                candidate.proposed_action,
                bucket,
                date
            );
        }
    }

    if !inventory.exclusions.is_empty() {
        println!("Exclusions:");
        for exclusion in &inventory.exclusions {
            println!("  - {} ({})", exclusion.relative_path, exclusion.reason);
        }
    }
}

fn print_digest_review_export(export: &DigestReviewExport) {
    println!("Digest review batch export");
    println!("  Output path:         {}", export.output_path);
    println!("  Files written:       {}", export.files_written.len());
    println!("  Files skipped:       {}", export.files_skipped.len());
    println!(
        "  Total candidates:    {}",
        export.inventory.total_candidates
    );
    println!(
        "  Returned candidates: {}",
        export.inventory.returned_candidates
    );
    println!("  Excluded files:      {}", export.inventory.excluded_count);

    if !export.files_written.is_empty() {
        println!("Written files:");
        for path in &export.files_written {
            println!("  - {}", path);
        }
    }

    if !export.files_skipped.is_empty() {
        println!("Skipped user-owned files:");
        for path in &export.files_skipped {
            println!("  - {}", path);
        }
    }

    if !export.inventory.warnings.is_empty() {
        println!("Warnings:");
        for warning in &export.inventory.warnings {
            println!("  - {}", warning);
        }
    }
}

fn print_digest_review_apply(apply: &DigestReviewApply) {
    println!("Digest review apply");
    println!("  Root:                  {}", apply.root);
    println!("  Files scanned:         {}", apply.files_scanned);
    println!("  Planned sources:       {}", apply.planned_count());
    println!("  Accepted:              {}", apply.accepted_count);
    println!("  Source-only:           {}", apply.source_only_count);
    println!("  Quarantined:           {}", apply.quarantined_count);
    println!("  Rejected:              {}", apply.rejected_count);

    if !apply.planned_sources.is_empty() {
        println!("Planned sources:");
        for source in &apply.planned_sources {
            println!(
                "  - {} [{}; {}]",
                source.candidate.relative_path, source.decision, source.candidate.source_kind
            );
        }
    }

    if !apply.files_with_no_decision.is_empty() {
        println!("Files with no review decision:");
        for path in &apply.files_with_no_decision {
            println!("  - {}", path);
        }
    }
    if !apply.files_with_invalid_decision.is_empty() {
        println!("Files with invalid review decisions:");
        for path in &apply.files_with_invalid_decision {
            println!("  - {}", path);
        }
    }
    if !apply.files_with_parse_errors.is_empty() {
        println!("Files with parse errors:");
        for path in &apply.files_with_parse_errors {
            println!("  - {}", path);
        }
    }
    if !apply.files_skipped.is_empty() {
        println!("Skipped files:");
        for path in &apply.files_skipped {
            println!("  - {}", path);
        }
    }
    if !apply.warnings.is_empty() {
        println!("Warnings:");
        for warning in &apply.warnings {
            println!("  - {}", warning);
        }
    }
}

fn print_digest_extraction_plan(plan: &DigestExtractionPlan) {
    println!("Digest extraction plan");
    println!("  Review path:            {}", plan.review_path);
    println!("  Output path:            {}", plan.output_path);
    println!("  Review files scanned:   {}", plan.review_files_scanned);
    println!("  Accepted sources:       {}", plan.accepted_sources);
    println!("  Source-only sources:    {}", plan.source_only_sources);
    println!("  Sources read:           {}", plan.sources_read);
    println!("  Candidate memories:     {}", plan.candidate_count());
    println!("  Files written:          {}", plan.files_written.len());
    println!("  Files skipped:          {}", plan.files_skipped.len());

    if !plan.candidates.is_empty() {
        println!("Candidate memories:");
        for candidate in &plan.candidates {
            println!(
                "  - {} [{}; {} chars]",
                candidate.title, candidate.source_kind, candidate.content_chars
            );
        }
    }

    if !plan.sources_skipped.is_empty() {
        println!("Skipped sources:");
        for skipped in &plan.sources_skipped {
            println!("  - {}", skipped);
        }
    }
    if !plan.files_written.is_empty() {
        println!("Written files:");
        for path in &plan.files_written {
            println!("  - {}", path);
        }
    }
    if !plan.files_skipped.is_empty() {
        println!("Skipped output files:");
        for path in &plan.files_skipped {
            println!("  - {}", path);
        }
    }
    if !plan.warnings.is_empty() {
        println!("Warnings:");
        for warning in &plan.warnings {
            println!("  - {}", warning);
        }
    }
}

fn print_digest_source_index_plan(plan: &DigestSourceIndexPlan, indexed_documents: usize) {
    println!("Digest source index plan");
    println!("  Review path:            {}", plan.review_path);
    println!("  Review files scanned:   {}", plan.review_files_scanned);
    println!("  Accepted sources:       {}", plan.accepted_sources);
    println!("  Source-only sources:    {}", plan.source_only_sources);
    println!("  Sources read:           {}", plan.sources_read);
    println!("  Documents planned:      {}", plan.document_count());
    println!("  Documents indexed:      {}", indexed_documents);

    if !plan.documents.is_empty() {
        println!("Prepared documents:");
        for document in &plan.documents {
            println!(
                "  - {} [{}; {} chars]",
                document.title, document.source_kind, document.content_chars
            );
        }
    }
    if !plan.sources_skipped.is_empty() {
        println!("Skipped sources:");
        for skipped in &plan.sources_skipped {
            println!("  - {}", skipped);
        }
    }
    if !plan.warnings.is_empty() {
        println!("Warnings:");
        for warning in &plan.warnings {
            println!("  - {}", warning);
        }
    }
}

fn print_document_ingestion_plan(plan: &DocumentIngestionPlan) {
    println!("Document ingestion plan");
    println!("  Documents planned:      {}", plan.documents.len());
    println!("  Chunks planned:         {}", plan.total_chunks());

    if !plan.documents.is_empty() {
        println!("Documents:");
        for document in &plan.documents {
            println!(
                "  - {} [{}; {} chars; {} sections; {} chunks]",
                document.title,
                chunking_strategy_label(document.chunking_strategy),
                document.content_chars,
                document.section_count,
                document.chunk_count
            );
            println!("    Path: {}", document.path);
        }
    }

    if !plan.warnings.is_empty() {
        println!("Warnings:");
        for warning in &plan.warnings {
            println!("  - {}", warning);
        }
    }
}

fn print_document_orphan_report(report: &DocumentOrphanReport) {
    print_document_orphan_report_summary(report);

    if report.groups.is_empty() {
        println!("No orphan groups returned.");
        return;
    }

    println!("Groups:");
    for group in &report.groups {
        println!(
            "  - {} [{} chunks; {}; {}]",
            group.missing_source_id,
            group.chunk_count,
            recovery_class_label(group.recovery_class),
            group.recovery_hint
        );
        if !group.detected_references.is_empty() {
            println!("    References:");
            for reference in &group.detected_references {
                match &reference.existing_source_id {
                    Some(source_id) => println!(
                        "      - {}: {} (matches source {})",
                        reference.reference_type, reference.value, source_id
                    ),
                    None => println!("      - {}: {}", reference.reference_type, reference.value),
                }
            }
        }
        if !group.candidate_matches.is_empty() {
            println!("    Candidate matches:");
            for candidate in &group.candidate_matches {
                println!(
                    "      - {} [{}; score {:.2}; {}/{} anchors]",
                    candidate.path,
                    candidate.match_type,
                    candidate.score,
                    candidate.matched_anchors,
                    candidate.total_anchors
                );
            }
        }
        if !group.samples.is_empty() {
            println!("    Samples:");
            for sample in &group.samples {
                println!(
                    "      - {} :: {}",
                    sample.heading_path, sample.content_preview
                );
            }
        }
    }

    if !report.candidate_scan_warnings.is_empty() {
        println!("Warnings:");
        for warning in &report.candidate_scan_warnings {
            println!("  - {}", warning);
        }
    }
}

fn print_document_orphan_report_summary(report: &DocumentOrphanReport) {
    println!("Document orphan recovery report");
    println!("  Orphan chunks:            {}", report.orphan_chunk_count);
    println!("  Missing source IDs:       {}", report.orphan_source_count);
    println!("  Groups returned:          {}", report.groups_returned);
    println!(
        "  Known source matches:     {}",
        report.groups_with_known_source_match
    );
    println!(
        "  Recoverable groups:       {}",
        report.recovery_summary.recoverable
    );
    println!(
        "  Unknown groups:           {}",
        report.recovery_summary.unknown
    );
    println!(
        "  Safe-to-quarantine:       {}",
        report.recovery_summary.safe_to_quarantine
    );
    println!(
        "  Candidate matches:        {}",
        report.groups_with_candidate_matches
    );
    println!(
        "  Candidate files scanned:  {}",
        report.candidate_files_scanned
    );
    println!(
        "  Candidate files skipped:  {}",
        report.candidate_files_skipped
    );
    println!(
        "  Samples per group:        {}",
        report.sample_limit_per_group
    );
}

fn write_document_orphan_report(
    report: &DocumentOrphanReport,
    output: &str,
    format: OrphanExportFormat,
) -> Result<()> {
    let path = std::path::Path::new(output);
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    let contents = match format {
        OrphanExportFormat::Markdown => document_orphan_report_markdown(report),
        OrphanExportFormat::Json => serde_json::to_string_pretty(report)?,
    };
    std::fs::write(path, contents)?;
    Ok(())
}

fn document_orphan_report_markdown(report: &DocumentOrphanReport) -> String {
    let mut output = String::new();
    output.push_str("# Document Orphan Recovery Report\n\n");
    output.push_str("## Summary\n\n");
    output.push_str(&format!("- Orphan chunks: {}\n", report.orphan_chunk_count));
    output.push_str(&format!(
        "- Missing source IDs: {}\n",
        report.orphan_source_count
    ));
    output.push_str(&format!("- Groups returned: {}\n", report.groups_returned));
    output.push_str(&format!(
        "- Recoverable groups: {}\n",
        report.recovery_summary.recoverable
    ));
    output.push_str(&format!(
        "- Unknown groups: {}\n",
        report.recovery_summary.unknown
    ));
    output.push_str(&format!(
        "- Safe-to-quarantine groups: {}\n",
        report.recovery_summary.safe_to_quarantine
    ));
    output.push_str(&format!(
        "- Known source matches: {}\n",
        report.groups_with_known_source_match
    ));
    output.push_str(&format!(
        "- Candidate matches: {}\n",
        report.groups_with_candidate_matches
    ));
    output.push_str(&format!(
        "- Candidate files scanned: {}\n",
        report.candidate_files_scanned
    ));
    output.push_str(&format!(
        "- Candidate files skipped: {}\n",
        report.candidate_files_skipped
    ));

    output.push_str("\n## Groups\n\n");
    for group in &report.groups {
        output.push_str(&format!("### `{}`\n\n", group.missing_source_id));
        output.push_str(&format!("- Chunks: {}\n", group.chunk_count));
        output.push_str(&format!(
            "- Recovery class: `{}`\n",
            recovery_class_label(group.recovery_class)
        ));
        output.push_str(&format!("- Recovery hint: `{}`\n", group.recovery_hint));
        output.push_str(&format!(
            "- Content fingerprint: `{}`\n",
            group.content_fingerprint
        ));
        output.push_str(&format!(
            "- Content anchors: {}\n",
            group.content_anchor_count
        ));

        if !group.detected_references.is_empty() {
            output.push_str("\nReferences:\n\n");
            for reference in &group.detected_references {
                if let Some(source_id) = &reference.existing_source_id {
                    output.push_str(&format!(
                        "- `{}`: `{}` (matches source `{}`)\n",
                        reference.reference_type, reference.value, source_id
                    ));
                } else {
                    output.push_str(&format!(
                        "- `{}`: `{}`\n",
                        reference.reference_type, reference.value
                    ));
                }
            }
        }

        if !group.candidate_matches.is_empty() {
            output.push_str("\nCandidate matches:\n\n");
            for candidate in &group.candidate_matches {
                output.push_str(&format!(
                    "- `{}` (`{}`, score {:.2}, {}/{} anchors, exact: {})\n",
                    candidate.path,
                    candidate.match_type,
                    candidate.score,
                    candidate.matched_anchors,
                    candidate.total_anchors,
                    candidate.exact_fingerprint_match
                ));
                for evidence in &candidate.evidence {
                    output.push_str(&format!("  - Evidence: {}\n", evidence));
                }
            }
        }

        if !group.samples.is_empty() {
            output.push_str("\nSamples:\n\n");
            for sample in &group.samples {
                output.push_str(&format!(
                    "- `{}`: {}\n",
                    sample.heading_path, sample.content_preview
                ));
            }
        }

        output.push('\n');
    }

    if !report.candidate_scan_warnings.is_empty() {
        output.push_str("## Warnings\n\n");
        for warning in &report.candidate_scan_warnings {
            output.push_str(&format!("- {}\n", warning));
        }
    }

    output
}

fn print_document_reindex_plan(plan: &DocumentReindexPlan) {
    print_document_reindex_plan_summary(plan);

    if plan.sources.is_empty() {
        println!("No source-level reindex actions planned.");
    } else {
        println!("Sources:");
        for source in &plan.sources {
            println!(
                "  - {} [{}; {}; {} groups; {} orphan chunks]",
                source.source_path,
                reindex_action_label(source.action),
                source.match_type,
                source.group_count,
                source.orphan_chunk_count
            );
            println!(
                "    Score range: {:.2} - {:.2}",
                source.min_score, source.max_score
            );
            if !source.existing_source_ids.is_empty() {
                println!("    Existing source IDs:");
                for source_id in &source.existing_source_ids {
                    println!("      - {}", source_id);
                }
            }
            if !source.notes.is_empty() {
                println!("    Notes:");
                for note in &source.notes {
                    println!("      - {}", note);
                }
            }
            if !source.groups.is_empty() {
                println!("    Groups:");
                for group in &source.groups {
                    println!(
                        "      - {} [{} chunks; score {:.2}; {}/{} anchors; exact: {}]",
                        group.missing_source_id,
                        group.orphan_chunk_count,
                        group.score,
                        group.matched_anchors,
                        group.total_anchors,
                        group.exact_fingerprint_match
                    );
                }
            }
        }
    }

    if !plan.review_only.is_empty() {
        println!("Review-only groups:");
        for group in &plan.review_only {
            println!(
                "  - {} [{} chunks; {}]",
                group.missing_source_id, group.orphan_chunk_count, group.reason
            );
        }
    }
}

fn print_document_reindex_plan_summary(plan: &DocumentReindexPlan) {
    println!("Document orphan reindex plan");
    println!("  Read-only:                 {}", plan.read_only);
    println!("  Orphan chunks:             {}", plan.orphan_chunk_count);
    println!("  Missing source IDs:        {}", plan.orphan_source_count);
    println!("  Recoverable groups:        {}", plan.recoverable_groups);
    println!("  Unknown groups:            {}", plan.unknown_groups);
    println!(
        "  Safe-to-quarantine:        {}",
        plan.safe_to_quarantine_groups
    );
    println!("  Planned source actions:    {}", plan.sources.len());
    println!("  Planned groups:            {}", plan.planned_groups);
    println!(
        "  Planned orphan chunks:     {}",
        plan.planned_orphan_chunks
    );
    println!("  Review-only groups:        {}", plan.review_only_groups);
}

fn write_document_reindex_plan(
    plan: &DocumentReindexPlan,
    output: &str,
    format: OrphanExportFormat,
) -> Result<()> {
    let path = std::path::Path::new(output);
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    let contents = match format {
        OrphanExportFormat::Markdown => document_reindex_plan_markdown(plan),
        OrphanExportFormat::Json => serde_json::to_string_pretty(plan)?,
    };
    std::fs::write(path, contents)?;
    Ok(())
}

fn document_reindex_plan_markdown(plan: &DocumentReindexPlan) -> String {
    let mut output = String::new();
    output.push_str("# Document Orphan Reindex Plan\n\n");
    output.push_str("## Summary\n\n");
    output.push_str(&format!("- Read-only: {}\n", plan.read_only));
    output.push_str(&format!("- Orphan chunks: {}\n", plan.orphan_chunk_count));
    output.push_str(&format!(
        "- Missing source IDs: {}\n",
        plan.orphan_source_count
    ));
    output.push_str(&format!(
        "- Recoverable groups: {}\n",
        plan.recoverable_groups
    ));
    output.push_str(&format!("- Unknown groups: {}\n", plan.unknown_groups));
    output.push_str(&format!(
        "- Safe-to-quarantine groups: {}\n",
        plan.safe_to_quarantine_groups
    ));
    output.push_str(&format!(
        "- Planned source actions: {}\n",
        plan.sources.len()
    ));
    output.push_str(&format!("- Planned groups: {}\n", plan.planned_groups));
    output.push_str(&format!(
        "- Planned orphan chunks: {}\n",
        plan.planned_orphan_chunks
    ));
    output.push_str(&format!(
        "- Review-only groups: {}\n",
        plan.review_only_groups
    ));

    output.push_str("\n## Source Actions\n\n");
    for source in &plan.sources {
        output.push_str(&format!("### `{}`\n\n", source.source_path));
        output.push_str(&format!(
            "- Action: `{}`\n",
            reindex_action_label(source.action)
        ));
        output.push_str(&format!("- Match type: `{}`\n", source.match_type));
        output.push_str(&format!("- Groups: {}\n", source.group_count));
        output.push_str(&format!("- Orphan chunks: {}\n", source.orphan_chunk_count));
        output.push_str(&format!(
            "- Score range: {:.2} - {:.2}\n",
            source.min_score, source.max_score
        ));

        if !source.existing_source_ids.is_empty() {
            output.push_str("\nExisting source IDs:\n\n");
            for source_id in &source.existing_source_ids {
                output.push_str(&format!("- `{}`\n", source_id));
            }
        }

        if !source.notes.is_empty() {
            output.push_str("\nNotes:\n\n");
            for note in &source.notes {
                output.push_str(&format!("- {}\n", note));
            }
        }

        if !source.groups.is_empty() {
            output.push_str("\nCovered orphan groups:\n\n");
            for group in &source.groups {
                output.push_str(&format!(
                    "- `{}`: {} chunks, score {:.2}, {}/{} anchors, exact: {}\n",
                    group.missing_source_id,
                    group.orphan_chunk_count,
                    group.score,
                    group.matched_anchors,
                    group.total_anchors,
                    group.exact_fingerprint_match
                ));
                for evidence in &group.evidence {
                    output.push_str(&format!("  - Evidence: {}\n", evidence));
                }
            }
        }

        output.push('\n');
    }

    if !plan.review_only.is_empty() {
        output.push_str("## Review-Only Groups\n\n");
        for group in &plan.review_only {
            output.push_str(&format!("### `{}`\n\n", group.missing_source_id));
            output.push_str(&format!("- Chunks: {}\n", group.orphan_chunk_count));
            output.push_str(&format!("- Recovery hint: `{}`\n", group.recovery_hint));
            output.push_str(&format!("- Reason: {}\n\n", group.reason));
        }
    }

    output
}

fn read_document_reindex_plan(path: &str) -> Result<DocumentReindexPlan> {
    let contents = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&contents)?)
}

fn read_document_reindex_execution_report(path: &str) -> Result<DocumentReindexExecutionReport> {
    let contents = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&contents)?)
}

fn read_document_orphan_cleanup_plan(path: &str) -> Result<DocumentOrphanCleanupPlan> {
    let contents = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&contents)?)
}

fn parse_reindex_actions(actions: &[String]) -> Result<Vec<DocumentReindexAction>> {
    actions
        .iter()
        .map(|action| {
            DocumentReindexAction::parse(action).ok_or_else(|| {
                anyhow::anyhow!(
                    "Unknown reindex action '{}'. Valid: reindex_file, reindex_digest_reviewed_source, inspect_existing_source",
                    action
                )
            })
        })
        .collect()
}

fn print_document_reindex_execution_report(report: &DocumentReindexExecutionReport) {
    print_document_reindex_execution_summary(report);

    if report.actions.is_empty() {
        println!("No source actions in execution report.");
        return;
    }

    println!("Actions:");
    for action in &report.actions {
        println!(
            "  - {} [{}; {}; {}]",
            action.source_path,
            reindex_action_label(action.action),
            reindex_execution_status_label(action.status),
            if action.dry_run { "dry-run" } else { "write" }
        );
        println!(
            "    Groups: {}, orphan chunks: {}",
            action.group_count, action.orphan_chunk_count
        );
        if let Some(chunk_count) = action.chunk_count {
            println!("    Indexed/planned chunks: {}", chunk_count);
        }
        if let Some(title) = &action.title {
            println!("    Title: {}", title);
        }
        if let Some(reason) = &action.reason {
            println!("    Reason: {}", reason);
        }
    }

    if !report.warnings.is_empty() {
        println!("Warnings:");
        for warning in &report.warnings {
            println!("  - {}", warning);
        }
    }
}

fn print_document_reindex_execution_summary(report: &DocumentReindexExecutionReport) {
    println!("Document orphan reindex execution report");
    println!("  Dry-run:                   {}", report.dry_run);
    println!(
        "  Orphan cleanup performed:  {}",
        report.orphan_cleanup_performed
    );
    println!(
        "  Plan source actions:       {}",
        report.plan_source_actions
    );
    println!(
        "  Selected source actions:   {}",
        report.selected_source_actions
    );
    println!(
        "  Planned source actions:    {}",
        report.planned_source_actions
    );
    println!(
        "  Reindexed source actions:  {}",
        report.reindexed_source_actions
    );
    println!(
        "  Already indexed actions:   {}",
        report.already_indexed_source_actions
    );
    println!(
        "  Inspection actions:        {}",
        report.inspection_source_actions
    );
    println!(
        "  Skipped source actions:    {}",
        report.skipped_source_actions
    );
    println!(
        "  Failed source actions:     {}",
        report.failed_source_actions
    );
    println!(
        "  Reindexed documents:       {}",
        report.reindexed_documents
    );
    println!("  Planned chunks:            {}", report.planned_chunks);
    println!("  Indexed chunks:            {}", report.indexed_chunks);
}

fn write_document_reindex_execution_report(
    report: &DocumentReindexExecutionReport,
    output: &str,
    format: OrphanExportFormat,
) -> Result<()> {
    let path = std::path::Path::new(output);
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    let contents = match format {
        OrphanExportFormat::Markdown => document_reindex_execution_markdown(report),
        OrphanExportFormat::Json => serde_json::to_string_pretty(report)?,
    };
    std::fs::write(path, contents)?;
    Ok(())
}

fn document_reindex_execution_markdown(report: &DocumentReindexExecutionReport) -> String {
    let mut output = String::new();
    output.push_str("# Document Orphan Reindex Execution Report\n\n");
    output.push_str("## Summary\n\n");
    output.push_str(&format!("- Dry-run: {}\n", report.dry_run));
    output.push_str(&format!(
        "- Orphan cleanup performed: {}\n",
        report.orphan_cleanup_performed
    ));
    output.push_str(&format!(
        "- Plan source actions: {}\n",
        report.plan_source_actions
    ));
    output.push_str(&format!(
        "- Selected source actions: {}\n",
        report.selected_source_actions
    ));
    output.push_str(&format!(
        "- Planned source actions: {}\n",
        report.planned_source_actions
    ));
    output.push_str(&format!(
        "- Reindexed source actions: {}\n",
        report.reindexed_source_actions
    ));
    output.push_str(&format!(
        "- Already indexed actions: {}\n",
        report.already_indexed_source_actions
    ));
    output.push_str(&format!(
        "- Inspection actions: {}\n",
        report.inspection_source_actions
    ));
    output.push_str(&format!(
        "- Skipped source actions: {}\n",
        report.skipped_source_actions
    ));
    output.push_str(&format!(
        "- Failed source actions: {}\n",
        report.failed_source_actions
    ));
    output.push_str(&format!(
        "- Reindexed documents: {}\n",
        report.reindexed_documents
    ));
    output.push_str(&format!("- Planned chunks: {}\n", report.planned_chunks));
    output.push_str(&format!("- Indexed chunks: {}\n", report.indexed_chunks));

    output.push_str("\n## Actions\n\n");
    for action in &report.actions {
        output.push_str(&format!("### `{}`\n\n", action.source_path));
        output.push_str(&format!(
            "- Action: `{}`\n",
            reindex_action_label(action.action)
        ));
        output.push_str(&format!(
            "- Status: `{}`\n",
            reindex_execution_status_label(action.status)
        ));
        output.push_str(&format!("- Dry-run: {}\n", action.dry_run));
        output.push_str(&format!("- Groups: {}\n", action.group_count));
        output.push_str(&format!("- Orphan chunks: {}\n", action.orphan_chunk_count));
        if let Some(chunk_count) = action.chunk_count {
            output.push_str(&format!("- Indexed/planned chunks: {}\n", chunk_count));
        }
        if let Some(title) = &action.title {
            output.push_str(&format!("- Title: `{}`\n", title));
        }
        if let Some(reason) = &action.reason {
            output.push_str(&format!("- Reason: {}\n", reason));
        }
        if !action.existing_source_ids.is_empty() {
            output.push_str("\nExisting source IDs:\n\n");
            for source_id in &action.existing_source_ids {
                output.push_str(&format!("- `{}`\n", source_id));
            }
        }
        if !action.notes.is_empty() {
            output.push_str("\nNotes:\n\n");
            for note in &action.notes {
                output.push_str(&format!("- {}\n", note));
            }
        }
        output.push('\n');
    }

    if !report.warnings.is_empty() {
        output.push_str("## Warnings\n\n");
        for warning in &report.warnings {
            output.push_str(&format!("- {}\n", warning));
        }
    }

    output
}

fn print_document_orphan_cleanup_plan(plan: &DocumentOrphanCleanupPlan) {
    print_document_orphan_cleanup_plan_summary(plan);

    if plan.groups.is_empty() {
        println!("No orphan cleanup groups returned.");
        return;
    }

    println!("Groups:");
    for group in &plan.groups {
        println!(
            "  - {} [{}; {} chunks; {}]",
            group.missing_source_id,
            cleanup_action_label(group.cleanup_action),
            group.orphan_chunk_count,
            group.reason
        );
        if let Some(source_path) = &group.reindex_source_path {
            println!("    Reindex source: {}", source_path);
        }
        if let Some(status) = group.reindex_status {
            println!(
                "    Reindex status: {}",
                reindex_execution_status_label(status)
            );
        }
        if !group.samples.is_empty() {
            println!("    Samples:");
            for sample in &group.samples {
                println!(
                    "      - {} :: {}",
                    sample.heading_path, sample.content_preview
                );
            }
        }
    }

    if !plan.warnings.is_empty() {
        println!("Warnings:");
        for warning in &plan.warnings {
            println!("  - {}", warning);
        }
    }
}

fn print_document_orphan_cleanup_plan_summary(plan: &DocumentOrphanCleanupPlan) {
    println!("Document orphan cleanup/quarantine plan");
    println!("  Read-only:                 {}", plan.read_only);
    println!("  Orphan chunks:             {}", plan.orphan_chunk_count);
    println!("  Missing source IDs:        {}", plan.orphan_source_count);
    println!("  Groups returned:           {}", plan.groups_returned);
    println!("  Recoverable groups:        {}", plan.recoverable_groups);
    println!("  Unknown groups:            {}", plan.unknown_groups);
    println!(
        "  Safe-to-quarantine:        {}",
        plan.safe_to_quarantine_groups
    );
    println!(
        "  Delete candidate groups:   {}",
        plan.delete_candidate_groups
    );
    println!(
        "  Delete candidate chunks:   {}",
        plan.delete_candidate_chunks
    );
    println!(
        "  Quarantine groups:         {}",
        plan.quarantine_candidate_groups
    );
    println!(
        "  Quarantine chunks:         {}",
        plan.quarantine_candidate_chunks
    );
    println!("  Manual review groups:      {}", plan.manual_review_groups);
    println!("  Manual review chunks:      {}", plan.manual_review_chunks);
}

fn write_document_orphan_cleanup_plan(
    plan: &DocumentOrphanCleanupPlan,
    output: &str,
    format: OrphanExportFormat,
) -> Result<()> {
    let path = std::path::Path::new(output);
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    let contents = match format {
        OrphanExportFormat::Markdown => document_orphan_cleanup_plan_markdown(plan),
        OrphanExportFormat::Json => serde_json::to_string_pretty(plan)?,
    };
    std::fs::write(path, contents)?;
    Ok(())
}

fn document_orphan_cleanup_plan_markdown(plan: &DocumentOrphanCleanupPlan) -> String {
    let mut output = String::new();
    output.push_str("# Document Orphan Cleanup/Quarantine Plan\n\n");
    output.push_str("## Summary\n\n");
    output.push_str(&format!("- Read-only: {}\n", plan.read_only));
    output.push_str(&format!("- Orphan chunks: {}\n", plan.orphan_chunk_count));
    output.push_str(&format!(
        "- Missing source IDs: {}\n",
        plan.orphan_source_count
    ));
    output.push_str(&format!("- Groups returned: {}\n", plan.groups_returned));
    output.push_str(&format!(
        "- Recoverable groups: {}\n",
        plan.recoverable_groups
    ));
    output.push_str(&format!("- Unknown groups: {}\n", plan.unknown_groups));
    output.push_str(&format!(
        "- Safe-to-quarantine groups: {}\n",
        plan.safe_to_quarantine_groups
    ));
    output.push_str(&format!(
        "- Delete candidate groups: {}\n",
        plan.delete_candidate_groups
    ));
    output.push_str(&format!(
        "- Delete candidate chunks: {}\n",
        plan.delete_candidate_chunks
    ));
    output.push_str(&format!(
        "- Quarantine candidate groups: {}\n",
        plan.quarantine_candidate_groups
    ));
    output.push_str(&format!(
        "- Quarantine candidate chunks: {}\n",
        plan.quarantine_candidate_chunks
    ));
    output.push_str(&format!(
        "- Manual review groups: {}\n",
        plan.manual_review_groups
    ));
    output.push_str(&format!(
        "- Manual review chunks: {}\n",
        plan.manual_review_chunks
    ));

    output.push_str("\n## Groups\n\n");
    for group in &plan.groups {
        output.push_str(&format!("### `{}`\n\n", group.missing_source_id));
        output.push_str(&format!(
            "- Cleanup action: `{}`\n",
            cleanup_action_label(group.cleanup_action)
        ));
        output.push_str(&format!("- Orphan chunks: {}\n", group.orphan_chunk_count));
        output.push_str(&format!(
            "- Recovery class: `{}`\n",
            recovery_class_label(group.recovery_class)
        ));
        output.push_str(&format!("- Recovery hint: `{}`\n", group.recovery_hint));
        output.push_str(&format!("- Reason: {}\n", group.reason));
        output.push_str(&format!(
            "- Content fingerprint: `{}`\n",
            group.content_fingerprint
        ));
        if let Some(source_path) = &group.reindex_source_path {
            output.push_str(&format!("- Reindex source: `{}`\n", source_path));
        }
        if let Some(action) = group.reindex_action {
            output.push_str(&format!(
                "- Reindex action: `{}`\n",
                reindex_action_label(action)
            ));
        }
        if let Some(status) = group.reindex_status {
            output.push_str(&format!(
                "- Reindex status: `{}`\n",
                reindex_execution_status_label(status)
            ));
        }

        if !group.existing_source_ids.is_empty() {
            output.push_str("\nExisting source IDs:\n\n");
            for source_id in &group.existing_source_ids {
                output.push_str(&format!("- `{}`\n", source_id));
            }
        }

        if !group.candidate_matches.is_empty() {
            output.push_str("\nCandidate matches:\n\n");
            for candidate in &group.candidate_matches {
                output.push_str(&format!(
                    "- `{}` (`{}`, score {:.2}, {}/{} anchors, exact: {})\n",
                    candidate.path,
                    candidate.match_type,
                    candidate.score,
                    candidate.matched_anchors,
                    candidate.total_anchors,
                    candidate.exact_fingerprint_match
                ));
            }
        }

        if !group.samples.is_empty() {
            output.push_str("\nSamples:\n\n");
            for sample in &group.samples {
                output.push_str(&format!(
                    "- `{}`: {}\n",
                    sample.heading_path, sample.content_preview
                ));
            }
        }

        output.push('\n');
    }

    if !plan.warnings.is_empty() {
        output.push_str("## Warnings\n\n");
        for warning in &plan.warnings {
            output.push_str(&format!("- {}\n", warning));
        }
    }

    output
}

fn cleanup_action_label(action: DocumentOrphanCleanupAction) -> &'static str {
    match action {
        DocumentOrphanCleanupAction::DeleteAfterSuccessfulReindex => {
            "delete_after_successful_reindex"
        }
        DocumentOrphanCleanupAction::Quarantine => "quarantine",
        DocumentOrphanCleanupAction::ManualReview => "manual_review",
    }
}

fn print_document_orphan_cleanup_execution_report(report: &DocumentOrphanCleanupExecutionReport) {
    print_document_orphan_cleanup_execution_summary(report);

    if report.actions.is_empty() {
        println!("No cleanup execution actions returned.");
        return;
    }

    println!("Actions:");
    for action in &report.actions {
        println!(
            "  - {} [{}; {} chunks; {}]",
            action.missing_source_id,
            cleanup_execution_status_label(action.status),
            action.planned_orphan_chunks,
            action.reason
        );
    }

    if !report.warnings.is_empty() {
        println!("Warnings:");
        for warning in &report.warnings {
            println!("  - {}", warning);
        }
    }
}

fn print_document_orphan_cleanup_execution_summary(report: &DocumentOrphanCleanupExecutionReport) {
    println!("Document orphan cleanup execution report");
    println!("  Dry-run:                   {}", report.dry_run);
    println!(
        "  Orphan cleanup performed:  {}",
        report.orphan_cleanup_performed
    );
    println!("  Plan groups:               {}", report.plan_groups);
    println!(
        "  Plan delete candidates:    {}",
        report.plan_delete_candidate_groups
    );
    println!(
        "  Plan quarantine groups:    {}",
        report.plan_quarantine_groups
    );
    println!(
        "  Selected delete groups:    {}",
        report.selected_delete_groups
    );
    println!(
        "  Planned delete groups:     {}",
        report.planned_delete_groups
    );
    println!(
        "  Planned delete chunks:     {}",
        report.planned_delete_chunks
    );
    println!("  Deleted groups:            {}", report.deleted_groups);
    println!("  Deleted chunks:            {}", report.deleted_chunks);
    println!(
        "  Quarantine groups retained: {}",
        report.quarantine_groups_retained
    );
    println!(
        "  Quarantine chunks retained: {}",
        report.quarantine_chunks_retained
    );
    println!(
        "  Manual review groups:      {}",
        report.manual_review_groups
    );
    println!("  Skipped groups:            {}", report.skipped_groups);
    println!("  Protected groups:          {}", report.protected_groups);
}

fn write_document_orphan_cleanup_execution_report(
    report: &DocumentOrphanCleanupExecutionReport,
    output: &str,
    format: OrphanExportFormat,
) -> Result<()> {
    let path = std::path::Path::new(output);
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    let contents = match format {
        OrphanExportFormat::Markdown => document_orphan_cleanup_execution_markdown(report),
        OrphanExportFormat::Json => serde_json::to_string_pretty(report)?,
    };
    std::fs::write(path, contents)?;
    Ok(())
}

fn document_orphan_cleanup_execution_markdown(
    report: &DocumentOrphanCleanupExecutionReport,
) -> String {
    let mut output = String::new();
    output.push_str("# Document Orphan Cleanup Execution Report\n\n");
    output.push_str("## Summary\n\n");
    output.push_str(&format!("- Dry-run: {}\n", report.dry_run));
    output.push_str(&format!(
        "- Orphan cleanup performed: {}\n",
        report.orphan_cleanup_performed
    ));
    output.push_str(&format!("- Plan groups: {}\n", report.plan_groups));
    output.push_str(&format!(
        "- Plan delete candidates: {}\n",
        report.plan_delete_candidate_groups
    ));
    output.push_str(&format!(
        "- Plan quarantine groups: {}\n",
        report.plan_quarantine_groups
    ));
    output.push_str(&format!(
        "- Selected delete groups: {}\n",
        report.selected_delete_groups
    ));
    output.push_str(&format!(
        "- Planned delete groups: {}\n",
        report.planned_delete_groups
    ));
    output.push_str(&format!(
        "- Planned delete chunks: {}\n",
        report.planned_delete_chunks
    ));
    output.push_str(&format!("- Deleted groups: {}\n", report.deleted_groups));
    output.push_str(&format!("- Deleted chunks: {}\n", report.deleted_chunks));
    output.push_str(&format!(
        "- Quarantine groups retained: {}\n",
        report.quarantine_groups_retained
    ));
    output.push_str(&format!(
        "- Quarantine chunks retained: {}\n",
        report.quarantine_chunks_retained
    ));
    output.push_str(&format!(
        "- Manual review groups: {}\n",
        report.manual_review_groups
    ));
    output.push_str(&format!("- Skipped groups: {}\n", report.skipped_groups));
    output.push_str(&format!(
        "- Protected groups: {}\n",
        report.protected_groups
    ));

    output.push_str("\n## Actions\n\n");
    for action in &report.actions {
        output.push_str(&format!("### `{}`\n\n", action.missing_source_id));
        output.push_str(&format!(
            "- Cleanup action: `{}`\n",
            cleanup_action_label(action.cleanup_action)
        ));
        output.push_str(&format!(
            "- Status: `{}`\n",
            cleanup_execution_status_label(action.status)
        ));
        output.push_str(&format!("- Dry-run: {}\n", action.dry_run));
        output.push_str(&format!(
            "- Planned orphan chunks: {}\n",
            action.planned_orphan_chunks
        ));
        output.push_str(&format!("- Deleted chunks: {}\n", action.deleted_chunks));
        output.push_str(&format!("- Reason: {}\n\n", action.reason));
    }

    if !report.warnings.is_empty() {
        output.push_str("## Warnings\n\n");
        for warning in &report.warnings {
            output.push_str(&format!("- {}\n", warning));
        }
    }

    output
}

fn write_document_orphan_quarantine_export(
    plan: &DocumentOrphanCleanupPlan,
    output: &str,
    format: OrphanExportFormat,
) -> Result<()> {
    let mut quarantine_plan = plan.clone();
    quarantine_plan
        .groups
        .retain(|group| group.cleanup_action == DocumentOrphanCleanupAction::Quarantine);
    let quarantine_chunks = quarantine_plan
        .groups
        .iter()
        .map(|group| group.orphan_chunk_count)
        .sum();
    quarantine_plan.orphan_chunk_count = quarantine_chunks;
    quarantine_plan.orphan_source_count = quarantine_plan.groups.len();
    quarantine_plan.groups_returned = quarantine_plan.groups.len();
    quarantine_plan.recoverable_groups = 0;
    quarantine_plan.unknown_groups = 0;
    quarantine_plan.safe_to_quarantine_groups = quarantine_plan.groups.len();
    quarantine_plan.delete_candidate_groups = 0;
    quarantine_plan.delete_candidate_chunks = 0;
    quarantine_plan.quarantine_candidate_groups = quarantine_plan.groups.len();
    quarantine_plan.quarantine_candidate_chunks = quarantine_chunks;
    quarantine_plan.manual_review_groups = 0;
    quarantine_plan.manual_review_chunks = 0;

    write_document_orphan_cleanup_plan(&quarantine_plan, output, format)
}

fn print_document_orphan_quarantine_review_export(export: &DocumentOrphanQuarantineReviewExport) {
    println!("Document orphan quarantine review export");
    println!("  Root:                    {}", export.root);
    println!("  Plan groups:             {}", export.plan_groups);
    println!(
        "  Plan quarantine groups:  {}",
        export.plan_quarantine_groups
    );
    println!(
        "  Plan quarantine chunks:  {}",
        export.plan_quarantine_chunks
    );
    println!("  Selected groups:         {}", export.selected_groups);
    println!(
        "  Selected orphan chunks:  {}",
        export.selected_orphan_chunks
    );
    println!("  Loaded chunks:           {}", export.loaded_chunks);
    println!("  Truncated groups:        {}", export.truncated_groups);
    println!("  Truncated chunks:        {}", export.truncated_chunks);
    println!("  Files written:           {}", export.files_written.len());
    println!("  Files skipped:           {}", export.files_skipped.len());
    if !export.warnings.is_empty() {
        println!("Warnings:");
        for warning in &export.warnings {
            println!("  - {}", warning);
        }
    }
}

fn print_document_orphan_quarantine_review_status(status: &DocumentOrphanQuarantineReviewStatus) {
    println!("Document orphan quarantine review status");
    println!("  Root:                       {}", status.root);
    println!("  Files scanned:              {}", status.files_scanned);
    println!("  Generated files:            {}", status.generated_files);
    println!("  Index pages:                {}", status.index_pages);
    println!("  Group pages:                {}", status.group_pages);
    println!("  User-owned files:           {}", status.user_owned_files);
    println!("  Pending:                    {}", status.pending_count);
    println!(
        "  Retain quarantine:          {}",
        status.retain_quarantine_count
    );
    println!(
        "  Promote to memory review:   {}",
        status.promote_to_memory_review_count
    );
    println!(
        "  Archive legacy:             {}",
        status.archive_legacy_count
    );
    println!(
        "  Delete later:               {}",
        status.delete_later_count
    );
    println!("  Invalid:                    {}", status.invalid_count);
    println!("  Parse errors:               {}", status.parse_error_count);
    println!("  Ready to apply:             {}", status.ready_to_apply);
    if !status.warnings.is_empty() {
        println!("Warnings:");
        for warning in &status.warnings {
            println!("  - {}", warning);
        }
    }
}

fn print_document_orphan_quarantine_review_prioritization(
    report: &DocumentOrphanQuarantineReviewPrioritization,
) {
    println!("Document orphan quarantine review prioritization");
    println!("  Root:                       {}", report.root);
    println!("  Files scanned:              {}", report.files_scanned);
    println!("  Group pages:                {}", report.group_pages);
    println!("  Pending:                    {}", report.pending_count);
    println!(
        "  Decided skipped:            {}",
        report.decided_skipped_count
    );
    println!(
        "  Invalid/parse skipped:      {}",
        report.invalid_or_parse_error_count
    );
    println!("  Candidates ranked:          {}", report.candidate_count);
    println!(
        "  Candidates after dedupe:    {}",
        report.ranked_candidate_count
    );
    println!("  Returned:                   {}", report.returned_count);
    println!(
        "  Duplicate fingerprint groups: {}",
        report.duplicate_fingerprint_group_count
    );
    println!(
        "  Duplicate fingerprint candidates: {}",
        report.duplicate_fingerprint_candidate_count
    );
    println!(
        "  Duplicate fingerprint skipped: {}",
        report.duplicate_fingerprint_skipped_count
    );
    println!(
        "  High priority:              {}",
        report.high_priority_count
    );
    println!(
        "  Medium priority:            {}",
        report.medium_priority_count
    );
    println!(
        "  Low priority:               {}",
        report.low_priority_count
    );

    for (index, item) in report.items.iter().enumerate() {
        println!();
        println!(
            "{}. {:?} score={} `{}`",
            index + 1,
            item.priority,
            item.score,
            item.relative_path
        );
        println!("   Missing source: {}", item.missing_source_id);
        println!("   Suggested step: {:?}", item.suggested_next_step);
        println!(
            "   Chunks: {} planned, {} exported",
            item.orphan_chunk_count, item.exported_chunk_count
        );
        if let Some(title) = &item.title_hint {
            println!("   Title hint: {}", title);
        }
        if let Some(reason) = &item.reason {
            println!("   Reason: {}", reason);
        }
        if item.fingerprint_group_size > 1 {
            println!(
                "   Fingerprint group: rank {}/{}",
                item.fingerprint_group_rank, item.fingerprint_group_size
            );
            println!(
                "   Duplicate paths: {}",
                item.fingerprint_duplicate_paths.join(", ")
            );
        }
        if !item.detected_signals.is_empty() {
            println!("   Signals: {}", item.detected_signals.join(", "));
        }
        if !item.score_reasons.is_empty() {
            println!("   Score reasons:");
            for reason in &item.score_reasons {
                println!("     - {}", reason);
            }
        }
        if !item.excerpt.is_empty() {
            println!("   Excerpt:");
            for line in item.excerpt.lines().take(6) {
                println!("     {}", line);
            }
        }
    }

    if !report.warnings.is_empty() {
        println!("Warnings:");
        for warning in &report.warnings {
            println!("  - {}", warning);
        }
    }
}

fn print_document_orphan_quarantine_review_apply(apply: &DocumentOrphanQuarantineReviewApply) {
    println!("Document orphan quarantine review apply dry-run");
    println!("  Root:                       {}", apply.root);
    println!("  Dry-run:                    {}", apply.dry_run);
    println!("  Files scanned:              {}", apply.files_scanned);
    println!("  Group pages:                {}", apply.group_pages);
    println!("  Pending:                    {}", apply.pending_count);
    println!("  Invalid:                    {}", apply.invalid_count);
    println!("  Parse errors:               {}", apply.parse_error_count);
    println!(
        "  Would retain quarantine:    {}",
        apply.retain_quarantine_count
    );
    println!(
        "  Would promote to review:    {}",
        apply.promote_to_memory_review_count
    );
    println!(
        "  Would archive legacy:       {}",
        apply.archive_legacy_count
    );
    println!("  Would mark delete later:    {}", apply.delete_later_count);
    println!(
        "  Ready for future write:     {}",
        apply.ready_for_future_write
    );
    if !apply.warnings.is_empty() {
        println!("Warnings:");
        for warning in &apply.warnings {
            println!("  - {}", warning);
        }
    }
}

fn cleanup_execution_status_label(status: DocumentOrphanCleanupExecutionStatus) -> &'static str {
    match status {
        DocumentOrphanCleanupExecutionStatus::PlannedDelete => "planned_delete",
        DocumentOrphanCleanupExecutionStatus::Deleted => "deleted",
        DocumentOrphanCleanupExecutionStatus::QuarantineRetained => "quarantine_retained",
        DocumentOrphanCleanupExecutionStatus::ManualReviewRequired => "manual_review_required",
        DocumentOrphanCleanupExecutionStatus::Skipped => "skipped",
        DocumentOrphanCleanupExecutionStatus::Protected => "protected",
    }
}

fn reindex_action_label(action: DocumentReindexAction) -> &'static str {
    action.as_str()
}

fn reindex_execution_status_label(status: DocumentReindexExecutionStatus) -> &'static str {
    match status {
        DocumentReindexExecutionStatus::Planned => "planned",
        DocumentReindexExecutionStatus::Reindexed => "reindexed",
        DocumentReindexExecutionStatus::AlreadyIndexed => "already_indexed",
        DocumentReindexExecutionStatus::RequiresInspection => "requires_inspection",
        DocumentReindexExecutionStatus::Skipped => "skipped",
        DocumentReindexExecutionStatus::Failed => "failed",
    }
}

fn recovery_class_label(class: DocumentRecoveryClass) -> &'static str {
    match class {
        DocumentRecoveryClass::Recoverable => "recoverable",
        DocumentRecoveryClass::Unknown => "unknown",
        DocumentRecoveryClass::SafeToQuarantine => "safe_to_quarantine",
    }
}

fn chunking_strategy_label(strategy: ChunkingStrategy) -> &'static str {
    match strategy {
        ChunkingStrategy::Empty => "empty",
        ChunkingStrategy::WholeDocument => "whole-document",
        ChunkingStrategy::HeadingSections => "heading-sections",
        ChunkingStrategy::SyntheticSections => "synthetic-sections",
    }
}

fn memory_scope_label(scope: &MemoryScope) -> String {
    match scope {
        MemoryScope::Global => "global".to_string(),
        MemoryScope::User => "user".to_string(),
        MemoryScope::Project { project_name, .. } => format!("project:{project_name}"),
        MemoryScope::Task {
            project_name,
            task_name,
            ..
        } => {
            let project = project_name.as_deref().unwrap_or("(unknown-project)");
            format!("task:{project}/{task_name}")
        }
        MemoryScope::Entity { entity_name, .. } => format!("entity:{entity_name}"),
        MemoryScope::Repository {
            remote_url,
            local_path,
            ..
        } => format!(
            "repository:{}",
            remote_url
                .as_deref()
                .or(local_path.as_deref())
                .unwrap_or("(unknown)")
        ),
        MemoryScope::Session { session_id } => format!("session:{session_id}"),
        MemoryScope::Custom { name } => format!("custom:{name}"),
    }
}

fn print_migration_inventory(inventory: &MigrationInventory) {
    println!("Migration inventory dry-run");
    if let Some(project_filter) = &inventory.project_filter {
        println!("  Project filter:       {}", project_filter);
    }
    println!("  Sources scanned:      {}", inventory.sources_scanned);
    println!("  Total candidates:     {}", inventory.total_candidates);
    println!("  Returned candidates:  {}", inventory.returned_candidates);
    println!("  Truncated:            {}", inventory.truncated);

    println!("By disposition:");
    if inventory.by_disposition.is_empty() {
        println!("  none");
    } else {
        for (disposition, count) in &inventory.by_disposition {
            println!("  {}: {}", disposition, count);
        }
    }

    println!("By source:");
    if inventory.by_source_kind.is_empty() {
        println!("  none");
    } else {
        for (source, count) in &inventory.by_source_kind {
            println!("  {}: {}", source, count);
        }
    }

    println!("By memory kind:");
    if inventory.by_memory_kind.is_empty() {
        println!("  none");
    } else {
        for (kind, count) in &inventory.by_memory_kind {
            println!("  {}: {}", kind, count);
        }
    }

    println!("By confidence:");
    if inventory.by_confidence.is_empty() {
        println!("  none");
    } else {
        for (bucket, count) in &inventory.by_confidence {
            println!("  {}: {}", bucket, count);
        }
    }

    if !inventory.warnings.is_empty() {
        println!("Warnings:");
        for warning in &inventory.warnings {
            println!("  - {}", warning);
        }
    }

    if !inventory.candidates.is_empty() {
        println!("Candidates:");
        for candidate in &inventory.candidates {
            println!(
                "  - [{}] {} -> {} ({:.2})",
                candidate.disposition,
                candidate.source_kind,
                candidate.proposed_kind,
                candidate.confidence
            );
            println!("    Title: {}", candidate.title);
            println!("    Source: {}", candidate.source_label);
            if let Some(key) = &candidate.source_key {
                println!("    Key: {}", key);
            }
            if !candidate.reasons.is_empty() {
                println!("    Reason: {}", candidate.reasons.join("; "));
            }
        }
    }
}

fn print_migration_review_export(export: &MigrationReviewExport) {
    println!("Migration review batch exported");
    println!("  Root:                {}", export.root);
    println!("  Files written:       {}", export.file_count());
    println!("  Files skipped:       {}", export.files_skipped.len());
    println!(
        "  Sources scanned:     {}",
        export.inventory.sources_scanned
    );
    println!(
        "  Total candidates:    {}",
        export.inventory.total_candidates
    );
    println!(
        "  Returned candidates: {}",
        export.inventory.returned_candidates
    );
    println!("  Truncated:           {}", export.inventory.truncated);

    if !export.files_skipped.is_empty() {
        println!("Skipped non-generated files:");
        for path in &export.files_skipped {
            println!("  - {}", path);
        }
    }

    if !export.inventory.warnings.is_empty() {
        println!("Warnings:");
        for warning in &export.inventory.warnings {
            println!("  - {}", warning);
        }
    }
}

fn print_migration_review_status(status: &MigrationReviewStatus) {
    println!("Migration review batch status");
    println!("  Root:                  {}", status.root);
    println!("  Ready to apply:        {}", status.ready_to_apply);
    println!("  Files scanned:         {}", status.files_scanned);
    println!("  Planned items:         {}", status.planned_count);
    println!("  Accepted:              {}", status.accepted_count);
    println!(
        "  Accepted with edits:   {}",
        status.accepted_with_edits_count
    );
    println!("  Quarantined:           {}", status.quarantined_count);
    println!("  Rejected:              {}", status.rejected_count);
    println!("  Duplicates skipped:    {}", status.duplicate_count);

    print_review_file_list(
        "Files with no review decision",
        &status.files_with_no_decision,
    );
    print_review_file_list(
        "Files with conflicting decisions",
        &status.files_with_conflicts,
    );
    print_review_file_list("Files not listed in index.md", &status.files_not_in_index);
    print_review_file_list(
        "Indexed files missing on disk",
        &status.indexed_files_missing,
    );
    print_review_file_list("Accepted files", &status.accepted_files);
    print_review_file_list("Quarantined files", &status.quarantined_files);
    print_review_file_list("Rejected files", &status.rejected_files);
    print_review_file_list("Skipped files", &status.files_skipped);
    print_review_file_list("Warnings", &status.warnings);
}

fn print_migration_review_apply(apply: &MigrationReviewApply) {
    if apply.dry_run {
        println!("Migration review apply dry-run");
    } else {
        println!("Migration review applied");
    }
    println!("  Root:                  {}", apply.root);
    println!("  Files scanned:         {}", apply.files_scanned);
    println!("  Planned items:         {}", apply.planned_count());
    println!("  Written items:         {}", apply.written_count());
    println!("  Accepted:              {}", apply.accepted_count);
    println!(
        "  Accepted with edits:   {}",
        apply.accepted_with_edits_count
    );
    println!("  Quarantined:           {}", apply.quarantined_count);
    println!("  Rejected:              {}", apply.rejected_count);
    println!("  Duplicates skipped:    {}", apply.duplicate_count);
    if let Some(commit) = &apply.commit {
        println!("  Knowledge commit:      {}", commit.id);
    }

    if !apply.files_with_no_decision.is_empty() {
        println!("Files with no review decision:");
        for path in &apply.files_with_no_decision {
            println!("  - {}", path);
        }
    }
    if !apply.files_with_conflicts.is_empty() {
        println!("Files with conflicting decisions:");
        for path in &apply.files_with_conflicts {
            println!("  - {}", path);
        }
    }
    print_review_file_list("Files not listed in index.md", &apply.files_not_in_index);
    print_review_file_list(
        "Indexed files missing on disk",
        &apply.indexed_files_missing,
    );
    if !apply.accepted_files.is_empty() {
        println!("Accepted files:");
        for path in &apply.accepted_files {
            println!("  - {}", path);
        }
    }
    if !apply.quarantined_files.is_empty() {
        println!("Quarantined files:");
        for path in &apply.quarantined_files {
            println!("  - {}", path);
        }
    }
    if !apply.rejected_files.is_empty() {
        println!("Rejected files:");
        for path in &apply.rejected_files {
            println!("  - {}", path);
        }
    }
    if !apply.files_skipped.is_empty() {
        println!("Skipped files:");
        for path in &apply.files_skipped {
            println!("  - {}", path);
        }
    }
    if !apply.warnings.is_empty() {
        println!("Warnings:");
        for warning in &apply.warnings {
            println!("  - {}", warning);
        }
    }
}

fn print_review_file_list(title: &str, files: &[String]) {
    if files.is_empty() {
        return;
    }
    println!("{title}:");
    for path in files {
        println!("  - {}", path);
    }
}

fn print_digest_extraction_review_apply(apply: &DigestExtractionReviewApply) {
    if apply.dry_run {
        println!("Digest extraction review apply dry-run");
    } else {
        println!("Digest extraction review applied");
    }
    println!("  Root:                  {}", apply.root);
    println!("  Files scanned:         {}", apply.files_scanned);
    println!("  Planned items:         {}", apply.planned_count());
    println!("  Written items:         {}", apply.written_count());
    println!("  Accepted:              {}", apply.accepted_count);
    println!("  Quarantined:           {}", apply.quarantined_count);
    println!("  Rejected:              {}", apply.rejected_count);
    println!("  Duplicates skipped:    {}", apply.duplicate_count);
    if let Some(commit) = &apply.commit {
        println!("  Knowledge commit:      {}", commit.id);
    }

    if !apply.files_with_no_decision.is_empty() {
        println!("Files with no review decision:");
        for path in &apply.files_with_no_decision {
            println!("  - {}", path);
        }
    }
    if !apply.files_with_invalid_decision.is_empty() {
        println!("Files with invalid review decision:");
        for path in &apply.files_with_invalid_decision {
            println!("  - {}", path);
        }
    }
    if !apply.files_with_parse_errors.is_empty() {
        println!("Files with parse errors:");
        for path in &apply.files_with_parse_errors {
            println!("  - {}", path);
        }
    }
    if !apply.files_skipped.is_empty() {
        println!("Skipped files:");
        for path in &apply.files_skipped {
            println!("  - {}", path);
        }
    }
    if !apply.warnings.is_empty() {
        println!("Warnings:");
        for warning in &apply.warnings {
            println!("  - {}", warning);
        }
    }
}

fn print_repository_migration_inventory(inventory: &RepositoryMigrationInventory) {
    println!("Repository migration inventory dry-run");
    if let Some(project_filter) = &inventory.project_filter {
        println!("  Project filter:       {}", project_filter);
    }
    println!("  Sources scanned:      {}", inventory.sources_scanned);
    println!("  Total candidates:     {}", inventory.total_candidates);
    println!("  Returned candidates:  {}", inventory.returned_candidates);
    println!("  Truncated:            {}", inventory.truncated);

    println!("By reference kind:");
    if inventory.by_reference_kind.is_empty() {
        println!("  none");
    } else {
        for (kind, count) in &inventory.by_reference_kind {
            println!("  {}: {}", kind, count);
        }
    }

    println!("By disposition:");
    if inventory.by_disposition.is_empty() {
        println!("  none");
    } else {
        for (disposition, count) in &inventory.by_disposition {
            println!("  {}: {}", disposition, count);
        }
    }

    println!("By project:");
    if inventory.by_project.is_empty() {
        println!("  none");
    } else {
        for (project, count) in &inventory.by_project {
            println!("  {}: {}", project, count);
        }
    }

    println!("By confidence:");
    if inventory.by_confidence.is_empty() {
        println!("  none");
    } else {
        for (bucket, count) in &inventory.by_confidence {
            println!("  {}: {}", bucket, count);
        }
    }

    if !inventory.warnings.is_empty() {
        println!("Warnings:");
        for warning in &inventory.warnings {
            println!("  - {}", warning);
        }
    }

    if !inventory.candidates.is_empty() {
        println!("Candidates:");
        for candidate in &inventory.candidates {
            println!(
                "  - [{}] {} ({:.2})",
                candidate.disposition, candidate.reference_kind, candidate.confidence
            );
            if let Some(name) = &candidate.repository_name {
                println!("    Repository: {}", name);
            }
            if let Some(remote) = &candidate.normalized_remote {
                println!("    Remote: {}", remote);
            }
            if let Some(path) = &candidate.local_path {
                println!("    Local path: {}", path);
            }
            if let Some(project) = &candidate.project_name {
                println!("    Project: {}", project);
            }
            if let Some(component_path) = &candidate.component_path {
                println!("    Possible component: {}", component_path);
            }
            println!("    Evidence records: {}", candidate.evidence.len());
        }
    }
}

fn print_repository_migration_review_export(export: &RepositoryMigrationReviewExport) {
    println!("Repository migration review batch exported");
    println!("  Root:                {}", export.root);
    println!("  Files written:       {}", export.file_count());
    println!("  Files skipped:       {}", export.files_skipped.len());
    println!(
        "  Sources scanned:     {}",
        export.inventory.sources_scanned
    );
    println!(
        "  Total candidates:    {}",
        export.inventory.total_candidates
    );
    println!(
        "  Returned candidates: {}",
        export.inventory.returned_candidates
    );
    println!("  Truncated:           {}", export.inventory.truncated);

    if !export.files_skipped.is_empty() {
        println!("Skipped non-generated files:");
        for path in &export.files_skipped {
            println!("  - {}", path);
        }
    }

    if !export.inventory.warnings.is_empty() {
        println!("Warnings:");
        for warning in &export.inventory.warnings {
            println!("  - {}", warning);
        }
    }
}

fn print_repository_migration_review_status(status: &RepositoryMigrationReviewStatus) {
    println!("Repository migration review batch status");
    println!("  Root:                  {}", status.root);
    println!("  Ready to apply:        {}", status.ready_to_apply);
    println!("  Files scanned:         {}", status.files_scanned);
    println!("  Planned records:       {}", status.planned_record_count);
    println!("  Accepted:              {}", status.accepted_count);
    println!(
        "  Accepted with edits:   {}",
        status.accepted_with_edits_count
    );
    println!("  Quarantined:           {}", status.quarantined_count);
    println!("  Rejected:              {}", status.rejected_count);
    println!("  Already existed:       {}", status.existing_record_count);

    print_review_file_list(
        "Files with no review decision",
        &status.files_with_no_decision,
    );
    print_review_file_list(
        "Files with conflicting decisions",
        &status.files_with_conflicts,
    );
    print_review_file_list("Files not listed in index.md", &status.files_not_in_index);
    print_review_file_list(
        "Indexed files missing on disk",
        &status.indexed_files_missing,
    );
    print_review_file_list("Accepted files", &status.accepted_files);
    print_review_file_list("Quarantined files", &status.quarantined_files);
    print_review_file_list("Rejected files", &status.rejected_files);
    print_review_file_list("Skipped files", &status.files_skipped);
    print_review_file_list("Warnings", &status.warnings);
}

fn print_repository_migration_review_apply(apply: &RepositoryMigrationReviewApply) {
    if apply.dry_run {
        println!("Repository migration review apply dry-run");
    } else {
        println!("Repository migration review applied");
    }
    println!("  Root:                  {}", apply.root);
    println!("  Files scanned:         {}", apply.files_scanned);
    println!("  Planned records:       {}", apply.planned_count());
    println!("  Written records:       {}", apply.written_count());
    println!("  Accepted:              {}", apply.accepted_count);
    println!(
        "  Accepted with edits:   {}",
        apply.accepted_with_edits_count
    );
    println!("  Quarantined:           {}", apply.quarantined_count);
    println!("  Rejected:              {}", apply.rejected_count);
    println!("  Already existed:       {}", apply.existing_record_count);
    if let Some(commit) = &apply.commit {
        println!("  Knowledge commit:      {}", commit.id);
    }

    if !apply.files_with_no_decision.is_empty() {
        println!("Files with no review decision:");
        for path in &apply.files_with_no_decision {
            println!("  - {}", path);
        }
    }
    if !apply.files_with_conflicts.is_empty() {
        println!("Files with conflicting decisions:");
        for path in &apply.files_with_conflicts {
            println!("  - {}", path);
        }
    }
    print_review_file_list("Files not listed in index.md", &apply.files_not_in_index);
    print_review_file_list(
        "Indexed files missing on disk",
        &apply.indexed_files_missing,
    );
    if !apply.accepted_files.is_empty() {
        println!("Accepted files:");
        for path in &apply.accepted_files {
            println!("  - {}", path);
        }
    }
    if !apply.quarantined_files.is_empty() {
        println!("Quarantined files:");
        for path in &apply.quarantined_files {
            println!("  - {}", path);
        }
    }
    if !apply.rejected_files.is_empty() {
        println!("Rejected files:");
        for path in &apply.rejected_files {
            println!("  - {}", path);
        }
    }
    if !apply.files_skipped.is_empty() {
        println!("Skipped files:");
        for path in &apply.files_skipped {
            println!("  - {}", path);
        }
    }
    if !apply.warnings.is_empty() {
        println!("Warnings:");
        for warning in &apply.warnings {
            println!("  - {}", warning);
        }
    }
}

fn cli_migration_writer(
    writer_harness: &str,
    model_provider: &str,
    model: &str,
) -> WriterProvenance {
    WriterProvenance {
        harness: Harness::parse(writer_harness),
        harness_version: None,
        model: ModelIdentity::new(model_provider, model),
        surface: Some("cli".to_string()),
        actor: "importer".to_string(),
        session_id: None,
        written_at: OffsetDateTime::now_utc(),
    }
}

fn cli_agent_writer(writer_harness: &str, model_provider: &str, model: &str) -> WriterProvenance {
    WriterProvenance {
        harness: Harness::parse(writer_harness),
        harness_version: None,
        model: ModelIdentity::new(model_provider, model),
        surface: Some("cli".to_string()),
        actor: "agent".to_string(),
        session_id: None,
        written_at: OffsetDateTime::now_utc(),
    }
}

fn setup_logging(verbose: bool) {
    let filter = if verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("warn")
    };

    tracing_subscriber::registry()
        .with(fmt::layer().with_writer(std::io::stderr))
        .with(filter)
        .init();
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    setup_logging(cli.verbose);

    match cli.command {
        Commands::Init { path } => {
            let path = path
                .map(std::path::PathBuf::from)
                .unwrap_or_else(StoreConfig::default_data_dir);
            println!("Initializing engram database at: {}", path.display());

            // Create the directory
            std::fs::create_dir_all(&path)?;

            // Initialize database
            let config = StoreConfig::rocksdb(path);
            let _db = connect_and_init(&config).await?;

            println!("✓ Database initialized successfully!");
        }

        Commands::Serve {
            memory,
            remote,
            username,
            password,
            http,
            port,
            project,
        } => {
            validate_serve_options(
                memory,
                remote.as_deref(),
                username.as_deref(),
                password.as_deref(),
                http,
                port,
            )?;

            // If http mode is requested, run the HTTP server directly (daemon mode)
            if http {
                // Determine storage configuration
                let store_config = match (memory, remote.clone()) {
                    (true, _) => {
                        info!("Starting MCP HTTP server (in-memory)");
                        StoreConfig::memory()
                    }
                    (false, Some(url)) => {
                        let username = username.ok_or_else(|| {
                            anyhow::anyhow!("--username required when using --remote")
                        })?;
                        let password = password.ok_or_else(|| {
                            anyhow::anyhow!("--password required when using --remote")
                        })?;
                        info!("Starting MCP HTTP server (remote: {})", url);
                        StoreConfig::remote(url, username, password)
                    }
                    (false, None) => {
                        // Use project-specific or global data directory
                        let data_dir = if let Some(proj) = &project {
                            let base = dirs::home_dir()
                                .unwrap_or_else(|| std::path::PathBuf::from("."))
                                .join(".engram")
                                .join("projects")
                                .join(proj)
                                .join("data");
                            std::fs::create_dir_all(&base)?;
                            base
                        } else {
                            StoreConfig::default_data_dir()
                        };
                        info!(
                            "Starting MCP HTTP server (RocksDB at {})",
                            data_dir.display()
                        );
                        StoreConfig::rocksdb(data_dir)
                    }
                };

                let db = connect_and_init(&store_config).await?;

                // Create entity service (Layer 1) - with embeddings for vector search
                let entity_service = EntityService::with_defaults(db.clone())?;
                entity_service.init().await?;

                // Create session service (Layer 2)
                let session_service = SessionService::new(db.clone());
                session_service.init().await?;

                // Create document service (Layer 3)
                let doc_service = DocumentService::with_defaults(db.clone())?;
                doc_service.init_schema().await?;

                // Create tool intelligence service (Layer 4)
                let tool_intel_service = ToolIntelService::new(db.clone());
                tool_intel_service.init().await?;

                // Create coordination service (Layer 5)
                let coordination_service = CoordinationService::new(db.clone());
                coordination_service.init().await?;

                // Create knowledge service (Layer 6)
                let knowledge_service = KnowledgeService::with_defaults(db.clone());
                knowledge_service.init().await?;

                // Create work service (Layer 7)
                let work_service = WorkService::with_defaults(db.clone())?;
                work_service.init().await?;

                // Create Memory OS service
                let memory_service = MemoryService::new(db.clone());
                memory_service.init_schema().await?;

                // Create Memory OS lint service
                let lint_service = LintService::new(db.clone());
                lint_service.init_schema().await?;

                // Create Memory OS graph service
                let graph_service = GraphService::new(db.clone());
                graph_service.init_schema().await?;

                // Create rolling handoff service
                let handoff_service = HandoffService::new(db.clone());
                handoff_service.init_schema().await?;

                // Create agent obligation service
                let obligation_service = ObligationService::new(db.clone());
                obligation_service.init_schema().await?;

                // Create repository topology service
                let repository_service = RepositoryService::new(db.clone());
                repository_service.init_schema().await?;

                // Create unified search service
                let search_service = SearchService::with_defaults(db.clone())?;

                // Create brain harness telemetry service
                let telemetry_service = TelemetryService::new(db);
                telemetry_service.init_schema().await?;

                // Start MCP HTTP server
                let server = EngramServer::new();
                server.init_entity(entity_service).await;
                server.init_session(session_service).await;
                server.init(doc_service).await;
                server.init_tool_intel(tool_intel_service).await;
                server.init_coordination(coordination_service).await;
                server.init_knowledge(knowledge_service).await;
                server.init_work(work_service).await;
                server.init_memory(memory_service).await;
                server.init_lint(lint_service).await;
                server.init_graph(graph_service).await;
                server.init_handoff(handoff_service).await;
                server.init_obligation(obligation_service).await;
                server.init_repository(repository_service).await;
                server.init_search(search_service).await;
                server.init_telemetry(telemetry_service).await;

                let listen_port = port.unwrap_or(daemon::DEFAULT_DAEMON_PORT);
                let addr = std::net::SocketAddr::from(([127, 0, 0, 1], listen_port));
                server.serve_http(addr).await?;
            } else {
                // Default: stdio proxy mode with auto-started daemon
                // Build daemon config
                let daemon_config = if let Some(proj) = project {
                    daemon::DaemonConfig::project(proj)
                } else {
                    daemon::DaemonConfig::global()
                };

                // Ensure daemon is running (starts one if needed)
                let daemon_port = daemon::ensure_daemon_running(&daemon_config).await?;
                info!("Connected to daemon on port {}", daemon_port);

                // Run the stdio-to-HTTP proxy
                let proxy_config = proxy::ProxyConfig::new(daemon_port);
                proxy::run_proxy(proxy_config).await?;
            }
        }

        Commands::Daemon { command } => {
            match command {
                DaemonCommands::Status { project } => {
                    let config = match project {
                        Some(p) => daemon::DaemonConfig::project(p),
                        None => daemon::DaemonConfig::global(),
                    };

                    match daemon::get_daemon_info(&config).await {
                        Ok(info) => {
                            let status = if info.healthy {
                                "🟢 running"
                            } else {
                                "🔴 not responding"
                            };
                            println!("Daemon status: {}", status);
                            println!("  Port: {}", info.port);
                            println!("  PID:  {}", info.pid);
                            if let Some(project) = &config.project {
                                println!("  Project: {}", project);
                            }
                            let current_exe = std::env::current_exe().ok();
                            let current_version = env!("CARGO_PKG_VERSION");
                            if let Some(metadata) = &info.metadata {
                                println!("  Spawned by: {}", metadata.executable_path);
                                println!("  Spawn version: {}", metadata.executable_version);
                                if let Some(path) = &current_exe {
                                    let current_path = path.display().to_string();
                                    println!("  Current CLI: {}", current_path);
                                    if metadata.executable_path != current_path {
                                        println!(
                                            "  Warning: daemon was spawned by a different executable path; restart it after updating Engram if runtime drift is suspected"
                                        );
                                    }
                                }
                                if metadata.executable_version != current_version {
                                    println!(
                                        "  Warning: daemon version {} differs from current CLI version {}",
                                        metadata.executable_version, current_version
                                    );
                                }
                                if metadata.pid != info.pid || metadata.port != info.port {
                                    println!(
                                        "  Warning: daemon spawn metadata does not match pid/port files"
                                    );
                                }
                            } else {
                                println!(
                                    "  Spawn metadata: unavailable (daemon may have been started by an older Engram binary)"
                                );
                                if let Some(path) = &current_exe {
                                    println!("  Current CLI: {}", path.display());
                                }
                            }
                        }
                        Err(_) => {
                            println!("Daemon status: 🔴 not running");
                            if let Some(proj) = config.project {
                                println!("  Project: {}", proj);
                            }
                        }
                    }
                }

                DaemonCommands::Start { project, port } => {
                    let mut config = match project {
                        Some(p) => daemon::DaemonConfig::project(p),
                        None => daemon::DaemonConfig::global(),
                    };
                    config.port = port;

                    let daemon_port = daemon::ensure_daemon_running(&config).await?;
                    println!("✓ Daemon running on port {}", daemon_port);
                }

                DaemonCommands::Stop { project } => {
                    let config = match project {
                        Some(p) => daemon::DaemonConfig::project(p),
                        None => daemon::DaemonConfig::global(),
                    };

                    daemon::stop_daemon(&config).await?;
                    println!("✓ Daemon stopped");
                }

                DaemonCommands::Logs { project, lines } => {
                    let config = match project {
                        Some(p) => daemon::DaemonConfig::project(p),
                        None => daemon::DaemonConfig::global(),
                    };

                    let log_file = config.log_file();
                    if log_file.exists() {
                        // Read last N lines from log file
                        let content = std::fs::read_to_string(&log_file)?;
                        let all_lines: Vec<&str> = content.lines().collect();
                        let start = if all_lines.len() > lines {
                            all_lines.len() - lines
                        } else {
                            0
                        };
                        for line in &all_lines[start..] {
                            println!("{}", line);
                        }
                    } else {
                        println!("No log file found at: {}", log_file.display());
                    }
                }
            }
        }

        Commands::Add { what } => {
            // Connect to database using RocksDB for persistence
            let config = StoreConfig::rocksdb(StoreConfig::default_data_dir());
            let db = connect_and_init(&config).await?;

            // Create entity service
            let service = EntityService::new(db);
            service.init().await?;

            match what {
                AddCommands::Entity {
                    name,
                    entity_type,
                    description,
                } => {
                    let entity = service
                        .create_entity(&name, entity_type.into(), description.as_deref())
                        .await?;

                    println!("✓ Entity created:");
                    println!("  ID:   {}", entity.id);
                    println!("  Name: {}", entity.name);
                    println!("  Type: {}", entity.entity_type);
                    if let Some(desc) = &entity.description {
                        println!("  Desc: {}", desc);
                    }
                }
                AddCommands::Alias { alias, entity } => {
                    service.add_alias(&entity, &alias).await?;
                    println!("✓ Alias '{}' added for entity '{}'", alias, entity);
                }
            }
        }

        Commands::Search { query, r#type } => {
            // Connect to database using RocksDB for persistence
            let config = StoreConfig::rocksdb(StoreConfig::default_data_dir());
            let db = connect_and_init(&config).await?;

            // Create entity service
            let service = EntityService::new(db);
            service.init().await?;

            // Search entities
            let entities = service.search_entities(&query).await?;

            // Filter by type if specified
            let entities: Vec<_> = if let Some(type_filter) = r#type {
                let filter_type = EntityType::parse(&type_filter);
                entities
                    .into_iter()
                    .filter(|e| e.entity_type == filter_type)
                    .collect()
            } else {
                entities
            };

            if entities.is_empty() {
                println!("No entities found matching: {}", query);
            } else {
                println!("Found {} entities:\n", entities.len());
                for entity in entities {
                    println!("  {} ({})", entity.name, entity.entity_type);
                    if let Some(desc) = &entity.description {
                        println!("    {}", desc);
                    }
                }
            }
        }

        Commands::Index {
            path,
            recursive: _,
            plan,
        } => {
            println!("Indexing: {}", path);

            let path = std::path::Path::new(&path);
            if plan {
                let ingestion_plan =
                    Pipeline::plan_path_with_config(path, &PipelineConfig::default())?;
                print_document_ingestion_plan(&ingestion_plan);
                return Ok(());
            }

            // Connect to database using RocksDB for persistence
            let config = StoreConfig::rocksdb(StoreConfig::default_data_dir());
            let db = connect_and_init(&config).await?;

            // Create document service
            let service = DocumentService::with_defaults(db)?;
            service.init_schema().await?;

            // Index the path
            if path.is_dir() {
                let results = service.index_directory(path).await?;
                let chunks: usize = results.iter().map(|d| d.chunks.len()).sum();
                println!(
                    "✓ Indexed {} documents with {} chunks",
                    results.len(),
                    chunks
                );
            } else if path.is_file() {
                let result = service.index_file(path).await?;
                println!(
                    "✓ Indexed '{}' with {} chunks",
                    result.parsed.title,
                    result.chunks.len()
                );
            } else {
                println!("✗ Path not found: {}", path.display());
            }
        }

        Commands::SearchDocs {
            query,
            limit,
            score,
        } => {
            // Connect to the persistent document database.
            let config = StoreConfig::rocksdb(StoreConfig::default_data_dir());
            let db = connect_and_init(&config).await?;

            // Create document service
            let service = DocumentService::with_defaults(db)?;

            // Search
            let results = service.search_threshold(&query, limit, score).await?;

            if results.is_empty() {
                println!("No results found for: {}", query);
            } else {
                println!("Found {} results for: {}\n", results.len(), query);

                for (i, result) in results.iter().enumerate() {
                    println!(
                        "{}. {} (score: {:.2})",
                        i + 1,
                        result.source.title.as_deref().unwrap_or("Untitled"),
                        result.score
                    );
                    println!("   Path: {}", result.source.path_or_url);
                    println!("   Section: {}", result.chunk.heading_path);
                    if let (Some(start), Some(end)) =
                        (result.chunk.start_line, result.chunk.end_line)
                    {
                        println!("   Lines: {}-{}", start, end);
                    }
                    // Show truncated content
                    let content = &result.chunk.content;
                    let preview = if content.len() > 200 {
                        let mut end = 200;
                        while end > 0 && !content.is_char_boundary(end) {
                            end -= 1;
                        }
                        format!("{}...", &content[..end])
                    } else {
                        content.clone()
                    };
                    println!("   Content: {}\n", preview.replace('\n', " "));
                }
            }
        }

        Commands::Stats => {
            // Connect to the persistent document database.
            let config = StoreConfig::rocksdb(StoreConfig::default_data_dir());
            let db = connect_and_init(&config).await?;

            // Create document service
            let service = DocumentService::with_defaults(db)?;

            // Get stats
            let stats = service.stats().await?;

            println!("Database statistics:");
            println!("  Document sources: {}", stats.source_count);
            println!("  Document chunks:  {}", stats.chunk_count);
            println!("  Searchable chunks: {}", stats.searchable_chunk_count);
            println!("  Orphan chunks:     {}", stats.orphan_chunk_count);
            println!("  Embedding dim:    {}", stats.embedding_dimension);
        }

        Commands::DocOrphans {
            limit,
            all,
            samples,
            scan_paths,
            digest_review_paths,
            max_candidate_files,
            max_file_bytes,
            output,
            format,
        } => {
            // Connect to the persistent document database.
            let config = StoreConfig::rocksdb(StoreConfig::default_data_dir());
            let db = connect_and_init(&config).await?;

            let service = DocumentService::with_defaults(db)?;
            let group_limit = if all { usize::MAX } else { limit };
            let report = service
                .orphan_recovery_report(DocumentRecoveryOptions {
                    group_limit,
                    sample_limit_per_group: samples,
                    scan_paths: scan_paths.into_iter().map(Into::into).collect(),
                    digest_review_paths: digest_review_paths.into_iter().map(Into::into).collect(),
                    max_candidate_files,
                    max_file_bytes,
                    ..Default::default()
                })
                .await?;
            let wrote_output = output.is_some();
            if let Some(output) = output {
                write_document_orphan_report(&report, &output, format)?;
                println!("Wrote document orphan recovery report: {}", output);
            }
            if wrote_output {
                print_document_orphan_report_summary(&report);
            } else {
                print_document_orphan_report(&report);
            }
        }

        Commands::DocReindexPlan {
            limit,
            all,
            samples,
            scan_paths,
            digest_review_paths,
            max_candidate_files,
            max_file_bytes,
            output,
            format,
        } => {
            // Connect to the persistent document database.
            let config = StoreConfig::rocksdb(StoreConfig::default_data_dir());
            let db = connect_and_init(&config).await?;

            let service = DocumentService::with_defaults(db)?;
            let group_limit = if all { usize::MAX } else { limit };
            let plan = service
                .orphan_reindex_plan(DocumentRecoveryOptions {
                    group_limit,
                    sample_limit_per_group: samples,
                    scan_paths: scan_paths.into_iter().map(Into::into).collect(),
                    digest_review_paths: digest_review_paths.into_iter().map(Into::into).collect(),
                    max_candidate_files,
                    max_file_bytes,
                    ..Default::default()
                })
                .await?;
            let wrote_output = output.is_some();
            if let Some(output) = output {
                write_document_reindex_plan(&plan, &output, format)?;
                println!("Wrote document orphan reindex plan: {}", output);
            }
            if wrote_output {
                print_document_reindex_plan_summary(&plan);
            } else {
                print_document_reindex_plan(&plan);
            }
        }

        Commands::DocReindexExecute {
            plan_path,
            execute,
            all,
            source_paths,
            actions,
            digest_review_paths,
            max_source_bytes,
            max_actions,
            output,
            format,
        } => {
            if execute && !all && source_paths.is_empty() {
                anyhow::bail!(
                    "write mode requires explicit approval: pass --all or one or more --source values"
                );
            }

            let plan = read_document_reindex_plan(&plan_path)?;
            let parsed_actions = parse_reindex_actions(&actions)?;

            // Connect to the persistent document database.
            let config = StoreConfig::rocksdb(StoreConfig::default_data_dir());
            let db = connect_and_init(&config).await?;

            let service = DocumentService::with_defaults(db)?;
            let report = service
                .execute_orphan_reindex_plan(
                    &plan,
                    DocumentReindexExecutionOptions {
                        dry_run: !execute,
                        source_paths,
                        actions: parsed_actions,
                        digest_review_paths: digest_review_paths
                            .into_iter()
                            .map(Into::into)
                            .collect(),
                        max_source_bytes,
                        max_actions,
                    },
                )
                .await?;

            let wrote_output = output.is_some();
            if let Some(output) = output {
                write_document_reindex_execution_report(&report, &output, format)?;
                println!("Wrote document orphan reindex execution report: {}", output);
            }
            if wrote_output {
                print_document_reindex_execution_summary(&report);
            } else {
                print_document_reindex_execution_report(&report);
            }
        }

        Commands::DocOrphanCleanupPlan {
            limit,
            all,
            samples,
            scan_paths,
            digest_review_paths,
            max_candidate_files,
            max_file_bytes,
            reindex_plan_path,
            execution_report_path,
            output,
            format,
        } => {
            // Connect to the persistent document database.
            let config = StoreConfig::rocksdb(StoreConfig::default_data_dir());
            let db = connect_and_init(&config).await?;

            let service = DocumentService::with_defaults(db)?;
            let group_limit = if all { usize::MAX } else { limit };
            let reindex_plan = reindex_plan_path
                .as_deref()
                .map(read_document_reindex_plan)
                .transpose()?;
            let execution_report = execution_report_path
                .as_deref()
                .map(read_document_reindex_execution_report)
                .transpose()?;
            let plan = service
                .orphan_cleanup_plan(DocumentOrphanCleanupPlanOptions {
                    recovery: DocumentRecoveryOptions {
                        group_limit,
                        sample_limit_per_group: samples,
                        scan_paths: scan_paths.into_iter().map(Into::into).collect(),
                        digest_review_paths: digest_review_paths
                            .into_iter()
                            .map(Into::into)
                            .collect(),
                        max_candidate_files,
                        max_file_bytes,
                        ..Default::default()
                    },
                    reindex_plan,
                    execution_report,
                })
                .await?;

            let wrote_output = output.is_some();
            if let Some(output) = output {
                write_document_orphan_cleanup_plan(&plan, &output, format)?;
                println!("Wrote document orphan cleanup/quarantine plan: {}", output);
            }
            if wrote_output {
                print_document_orphan_cleanup_plan_summary(&plan);
            } else {
                print_document_orphan_cleanup_plan(&plan);
            }
        }

        Commands::DocOrphanCleanupExecute {
            plan_path,
            execute,
            delete_candidates,
            all_delete_candidates,
            source_ids,
            max_groups,
            quarantine_output,
            output,
            format,
        } => {
            if execute && (!delete_candidates || (!all_delete_candidates && source_ids.is_empty()))
            {
                anyhow::bail!(
                    "write mode requires explicit approval: pass --delete-candidates and either --all-delete-candidates or one or more --source-id values"
                );
            }

            let plan = read_document_orphan_cleanup_plan(&plan_path)?;
            if let Some(quarantine_output) = quarantine_output.as_ref() {
                write_document_orphan_quarantine_export(&plan, quarantine_output, format)?;
                println!(
                    "Wrote document orphan quarantine export: {}",
                    quarantine_output
                );
            }

            // Connect to the persistent document database.
            let config = StoreConfig::rocksdb(StoreConfig::default_data_dir());
            let db = connect_and_init(&config).await?;

            let service = DocumentService::with_defaults(db)?;
            let report = service
                .execute_orphan_cleanup_plan(
                    &plan,
                    DocumentOrphanCleanupExecutionOptions {
                        dry_run: !execute,
                        approve_delete_candidates: delete_candidates,
                        missing_source_ids: source_ids,
                        max_groups,
                    },
                )
                .await?;

            let wrote_output = output.is_some();
            if let Some(output) = output {
                write_document_orphan_cleanup_execution_report(&report, &output, format)?;
                println!("Wrote document orphan cleanup execution report: {}", output);
            }
            if wrote_output {
                print_document_orphan_cleanup_execution_summary(&report);
            } else {
                print_document_orphan_cleanup_execution_report(&report);
            }
        }

        Commands::DocOrphanQuarantineReviewExport {
            plan_path,
            output_dir,
            max_groups,
            max_chunks_per_group,
            max_chunk_bytes,
        } => {
            let plan = read_document_orphan_cleanup_plan(&plan_path)?;

            // Connect to the persistent document database.
            let config = StoreConfig::rocksdb(StoreConfig::default_data_dir());
            let db = connect_and_init(&config).await?;

            let service = DocumentService::with_defaults(db)?;
            let export = service
                .export_orphan_quarantine_review(
                    &plan,
                    &output_dir,
                    DocumentOrphanQuarantineReviewOptions {
                        max_groups,
                        max_chunks_per_group,
                        max_chunk_bytes,
                    },
                )
                .await?;

            print_document_orphan_quarantine_review_export(&export);
        }

        Commands::DocOrphanQuarantineReviewStatus { review_path } => {
            let status = DocumentService::orphan_quarantine_review_status_for_dir(&review_path)?;
            print_document_orphan_quarantine_review_status(&status);
        }

        Commands::DocOrphanQuarantineReviewPrioritize {
            review_path,
            limit,
            include_decided,
            include_duplicate_fingerprints,
            max_excerpt_bytes,
        } => {
            let report = DocumentService::prioritize_orphan_quarantine_review_for_dir(
                &review_path,
                DocumentOrphanQuarantineReviewPrioritizationOptions {
                    limit: Some(limit),
                    include_decided,
                    include_duplicate_fingerprints,
                    max_excerpt_bytes,
                },
            )?;
            print_document_orphan_quarantine_review_prioritization(&report);
        }

        Commands::DocOrphanQuarantineReviewApply { review_path } => {
            let apply = DocumentService::apply_orphan_quarantine_review_for_dir(
                &review_path,
                DocumentOrphanQuarantineReviewApplyOptions::default(),
            )?;
            print_document_orphan_quarantine_review_apply(&apply);
        }

        Commands::Knowledge { command } => {
            // Connect to database using RocksDB for persistence
            let config = StoreConfig::rocksdb(StoreConfig::default_data_dir());
            let db = connect_and_init(&config).await?;

            // Create knowledge service
            let service = KnowledgeService::with_defaults(db);

            match command {
                KnowledgeCommands::Init => {
                    println!("Initializing knowledge system...");
                    service.init().await?;
                    println!("✓ Knowledge system initialized!");
                    println!(
                        "  Personal repo: {}",
                        service.knowledge_repo_path().display()
                    );
                }

                KnowledgeCommands::Scan { path, repo } => {
                    // Initialize first to ensure schema exists
                    service.init().await?;

                    println!("Scanning: {} (repo: {})", path, repo);
                    let path = std::path::Path::new(&path);
                    let result = service.scan_directory(path, &repo).await?;

                    println!("✓ Scan complete:");
                    println!("  Files found:   {}", result.files_found);
                    println!("  New files:     {}", result.files_new);
                    println!("  Updated files: {}", result.files_updated);
                }

                KnowledgeCommands::Import {
                    source,
                    name,
                    doc_type,
                } => {
                    // Initialize first to ensure schema exists
                    service.init().await?;

                    println!("Importing: {} as '{}'", source, name);
                    let source_path = std::path::Path::new(&source);
                    let doc = service
                        .import_doc(source_path, &name, doc_type.into())
                        .await?;

                    println!("✓ Document imported:");
                    println!("  ID:   {}", doc.id);
                    println!("  Name: {}", doc.name);
                    println!("  Type: {}", doc.doc_type);
                    if let Some(path) = &doc.canonical_path {
                        println!("  Path: {}", path);
                    }
                }

                KnowledgeCommands::Register {
                    path,
                    name,
                    doc_type,
                } => {
                    // Initialize first to ensure schema exists
                    service.init().await?;

                    println!("Registering: {} as '{}'", path, name);
                    let file_path = std::path::Path::new(&path);
                    let doc = service
                        .register_doc(file_path, &name, doc_type.into())
                        .await?;

                    println!("✓ Document registered:");
                    println!("  ID:   {}", doc.id);
                    println!("  Name: {}", doc.name);
                    println!("  Type: {}", doc.doc_type);
                }

                KnowledgeCommands::List => {
                    // Initialize first to ensure schema exists
                    service.init().await?;

                    let docs = service.list_docs().await?;

                    if docs.is_empty() {
                        println!("No knowledge documents found.");
                        println!("Use 'engram knowledge import' or 'engram knowledge register' to add documents.");
                    } else {
                        println!("Knowledge documents ({}):\n", docs.len());
                        for doc in docs {
                            println!("  {} ({})", doc.name, doc.doc_type);
                            println!("    ID:     {}", doc.id);
                            println!("    Status: {:?}", doc.status);
                            if let Some(path) = &doc.canonical_path {
                                println!("    Path:   {}", path);
                            }
                            println!();
                        }
                    }
                }

                KnowledgeCommands::Duplicates => {
                    // Initialize first to ensure schema exists
                    service.init().await?;

                    println!("Finding duplicates...\n");
                    let duplicates = service.find_duplicates().await?;

                    if duplicates.is_empty() {
                        println!("No duplicate documents found.");
                    } else {
                        println!("Found {} duplicate groups:\n", duplicates.len());
                        for (i, group) in duplicates.iter().enumerate() {
                            println!("Group {} (hash: {}...):", i + 1, &group.content_hash[..12]);
                            for file in &group.files {
                                println!("  - {}", file.path);
                            }
                            println!();
                        }
                    }
                }

                KnowledgeCommands::Versions => {
                    // Initialize first to ensure schema exists
                    service.init().await?;

                    println!("Detecting version chains...\n");
                    let chains = service.detect_versions().await?;

                    if chains.is_empty() {
                        println!("No version chains found.");
                    } else {
                        println!("Found {} version chains:\n", chains.len());
                        for chain in chains {
                            println!("'{}' ({} versions):", chain.base_name, chain.versions.len());
                            for v in &chain.versions {
                                let version_str = v
                                    .version
                                    .map(|n| format!("v{}", n))
                                    .unwrap_or_else(|| "(no version)".to_string());
                                println!("  {} - {}", version_str, v.path);
                            }

                            // Show canonical recommendation
                            if let Ok(Some(canonical)) = service.resolve_canonical(&chain).await {
                                println!("  → Recommended canonical: {}", canonical);
                            }
                            println!();
                        }
                    }
                }

                KnowledgeCommands::Stats => {
                    // Initialize first to ensure schema exists
                    service.init().await?;

                    let stats = service.stats().await?;

                    println!("Knowledge statistics:");
                    println!("  Documents:     {}", stats.doc_count);
                    println!("  Synced files:  {}", stats.file_sync_count);
                    println!("  Aliases:       {}", stats.alias_count);
                }
            }
        }

        Commands::Entity { command } => {
            // Connect to database using RocksDB for persistence
            let config = StoreConfig::rocksdb(StoreConfig::default_data_dir());
            let db = connect_and_init(&config).await?;

            // Create entity service
            let service = EntityService::new(db);
            service.init().await?;

            match command {
                EntityCommands::Create {
                    name,
                    entity_type,
                    description,
                } => {
                    let entity = service
                        .create_entity(&name, entity_type.into(), description.as_deref())
                        .await?;

                    println!("✓ Entity created:");
                    println!("  ID:   {}", entity.id);
                    println!("  Name: {}", entity.name);
                    println!("  Type: {}", entity.entity_type);
                    if let Some(desc) = &entity.description {
                        println!("  Desc: {}", desc);
                    }
                }

                EntityCommands::List { entity_type } => {
                    let type_filter = entity_type.map(EntityType::from);
                    let entities = service.list_entities(type_filter.as_ref()).await?;

                    if entities.is_empty() {
                        println!("No entities found.");
                        println!(
                            "Use 'engram entity create' or 'engram add entity' to create one."
                        );
                    } else {
                        println!("Entities ({}):\n", entities.len());
                        for entity in entities {
                            println!("  {} ({})", entity.name, entity.entity_type);
                            println!("    ID: {}", entity.id);
                            if let Some(desc) = &entity.description {
                                println!("    Description: {}", desc);
                            }
                            println!();
                        }
                    }
                }

                EntityCommands::Show { name } => {
                    let entity = service
                        .resolve(&name)
                        .await?
                        .ok_or_else(|| anyhow::anyhow!("Entity not found: {}", name))?;

                    println!("Entity: {}\n", entity.name);
                    println!("  ID:      {}", entity.id);
                    println!("  Type:    {}", entity.entity_type);
                    if let Some(desc) = &entity.description {
                        println!("  Desc:    {}", desc);
                    }
                    println!("  Created: {}", entity.created_at);

                    // Show aliases
                    let aliases = service.get_aliases(&entity.name).await?;
                    if !aliases.is_empty() {
                        println!("\n  Aliases: {}", aliases.join(", "));
                    }

                    // Show outgoing relationships
                    let related_from = service.get_related_from(&entity.id).await?;
                    if !related_from.is_empty() {
                        println!("\n  Relationships (outgoing):");
                        for (rel, target) in related_from {
                            println!(
                                "    --[{}]--> {} ({})",
                                rel.relation_type, target.name, target.entity_type
                            );
                        }
                    }

                    // Show incoming relationships
                    let related_to = service.get_related_to(&entity.id).await?;
                    if !related_to.is_empty() {
                        println!("\n  Relationships (incoming):");
                        for (rel, source) in related_to {
                            println!(
                                "    <--[{}]-- {} ({})",
                                rel.relation_type, source.name, source.entity_type
                            );
                        }
                    }

                    // Show observations
                    let observations = service.get_observations(&entity.name).await?;
                    if !observations.is_empty() {
                        println!("\n  Observations:");
                        for obs in observations {
                            let source_str = obs.source.as_deref().unwrap_or("unknown");
                            println!("    - {} (from: {})", obs.content, source_str);
                        }
                    }
                }

                EntityCommands::Search { query } => {
                    let entities = service.search_entities(&query).await?;

                    if entities.is_empty() {
                        println!("No entities found matching: {}", query);
                    } else {
                        println!("Found {} entities:\n", entities.len());
                        for entity in entities {
                            println!("  {} ({})", entity.name, entity.entity_type);
                            if let Some(desc) = &entity.description {
                                println!("    {}", desc);
                            }
                        }
                    }
                }

                EntityCommands::Relate {
                    source,
                    relation,
                    target,
                } => {
                    let rel = service.relate(&source, relation.into(), &target).await?;

                    println!("✓ Relationship created:");
                    println!("  {} --[{}]--> {}", source, rel.relation_type, target);
                }

                EntityCommands::Alias { entity, alias } => {
                    service.add_alias(&entity, &alias).await?;
                    println!("✓ Alias '{}' added for entity '{}'", alias, entity);
                }

                EntityCommands::Observe {
                    entity,
                    content,
                    key,
                    source,
                } => {
                    let (obs, previous) = service
                        .add_observation(&entity, &content, key.as_deref(), source.as_deref())
                        .await?;

                    let action = if previous.is_some() {
                        "updated"
                    } else {
                        "added"
                    };
                    println!("✓ Observation {}:", action);
                    println!("  Entity: {}", entity);
                    if let Some(k) = &obs.key {
                        println!("  Key: {}", k);
                    }
                    println!("  Content: {}", obs.content);
                    if let Some(src) = &obs.source {
                        println!("  Source: {}", src);
                    }
                    if let Some(prev) = previous {
                        println!("  Previous: {}", prev.content);
                    }
                }

                EntityCommands::Delete { name } => {
                    let entity = service
                        .resolve(&name)
                        .await?
                        .ok_or_else(|| anyhow::anyhow!("Entity not found: {}", name))?;

                    service.delete_entity(&entity.id).await?;
                    println!("✓ Entity '{}' deleted", name);
                }

                EntityCommands::Stats => {
                    let stats = service.stats().await?;

                    println!("Entity statistics:");
                    println!("  Entities:      {}", stats.entity_count);
                    println!("  Relationships: {}", stats.relationship_count);
                    println!("  Aliases:       {}", stats.alias_count);
                    println!("  Observations:  {}", stats.observation_count);
                }
            }
        }

        // =========================================================================
        // Layer 2: Session Commands
        // =========================================================================
        Commands::Session { command } => {
            let config = StoreConfig::rocksdb(StoreConfig::default_data_dir());
            let db = connect_and_init(&config).await?;
            let service = SessionService::new(db);
            service.init().await?;

            match command {
                SessionCommands::Start {
                    agent,
                    project,
                    goal,
                } => {
                    let session = service
                        .start_session(agent.as_deref(), project.as_deref(), goal.as_deref())
                        .await?;

                    println!("✓ Session started:");
                    println!("  ID:      {}", session.id);
                    if let Some(a) = &session.agent {
                        println!("  Agent:   {}", a);
                    }
                    if let Some(p) = &session.project {
                        println!("  Project: {}", p);
                    }
                    if let Some(g) = &session.goal {
                        println!("  Goal:    {}", g);
                    }
                    println!(
                        "\nTip: Use 'engram session log' to record events during your session."
                    );
                }

                SessionCommands::End {
                    session_id,
                    summary,
                } => {
                    let id = if let Some(sid) = session_id {
                        engram_core::id::Id::parse(&sid)?
                    } else {
                        // Find the most recent active session
                        let active = service.get_active_sessions(None).await?;
                        active
                            .first()
                            .ok_or_else(|| anyhow::anyhow!("No active session found"))?
                            .id
                    };

                    service.end_session(&id, summary.as_deref()).await?;
                    println!("✓ Session {} ended", id);
                    if let Some(s) = summary {
                        println!("  Summary: {}", s);
                    }
                }

                SessionCommands::List {
                    status,
                    agent,
                    project,
                    limit,
                } => {
                    let status_filter: Option<SessionStatus> = status.map(|s| s.into());
                    let sessions = service
                        .list_sessions(
                            status_filter.as_ref(),
                            agent.as_deref(),
                            project.as_deref(),
                            Some(limit),
                        )
                        .await?;

                    if sessions.is_empty() {
                        println!("No sessions found.");
                    } else {
                        println!("Sessions ({}):\n", sessions.len());
                        for s in sessions {
                            let status_icon = match s.status {
                                SessionStatus::Active => "🟢",
                                SessionStatus::Completed => "✅",
                                SessionStatus::Abandoned => "❌",
                            };
                            println!("  {} {} [{}]", status_icon, s.id, s.status);
                            if let Some(a) = &s.agent {
                                print!("    Agent: {}", a);
                            }
                            if let Some(p) = &s.project {
                                print!("  Project: {}", p);
                            }
                            println!();
                            if let Some(g) = &s.goal {
                                println!("    Goal: {}", g);
                            }
                        }
                    }
                }

                SessionCommands::Show { session_id } => {
                    let id = engram_core::id::Id::parse(&session_id)?;
                    let (session, events) = service.get_session_with_events(&id).await?;

                    let status_icon = match session.status {
                        SessionStatus::Active => "🟢",
                        SessionStatus::Completed => "✅",
                        SessionStatus::Abandoned => "❌",
                    };

                    println!("Session: {} {}", status_icon, session.id);
                    println!("  Status:  {}", session.status);
                    if let Some(a) = &session.agent {
                        println!("  Agent:   {}", a);
                    }
                    if let Some(p) = &session.project {
                        println!("  Project: {}", p);
                    }
                    if let Some(g) = &session.goal {
                        println!("  Goal:    {}", g);
                    }
                    if let Some(s) = &session.summary {
                        println!("  Summary: {}", s);
                    }

                    if !events.is_empty() {
                        println!("\n  Events ({}):", events.len());
                        for e in events {
                            let type_icon = match e.event_type {
                                EventType::Decision => "💡",
                                EventType::Observation => "👁️",
                                EventType::Error => "❗",
                                EventType::Command => "⚡",
                                EventType::FileChange => "📝",
                                EventType::ToolUse => "🔧",
                                EventType::Milestone => "🎯",
                                EventType::Prompt => "💬",
                                EventType::Plan => "📋",
                                EventType::ToolResult => "🧰",
                                EventType::Test => "✅",
                                EventType::Preference => "⚙️",
                                EventType::Rule => "📏",
                                EventType::Limitation => "⛔",
                                EventType::HandoffUpdate => "📦",
                                EventType::Custom(_) => "📌",
                            };
                            println!("    {} [{}] {}", type_icon, e.event_type, e.content);
                            if let Some(ctx) = &e.context {
                                println!("       Context: {}", ctx);
                            }
                        }
                    } else {
                        println!("\n  No events logged.");
                    }
                }

                SessionCommands::Log {
                    event_type,
                    content,
                    session,
                    context,
                    source,
                } => {
                    let session_id = if let Some(sid) = session {
                        engram_core::id::Id::parse(&sid)?
                    } else {
                        // Find the most recent active session
                        let active = service.get_active_sessions(None).await?;
                        active
                            .first()
                            .ok_or_else(|| anyhow::anyhow!("No active session found. Start one with 'engram session start'"))?
                            .id
                    };

                    let event = service
                        .log_event(
                            &session_id,
                            event_type.into(),
                            &content,
                            context.as_deref(),
                            source.as_deref(),
                        )
                        .await?;

                    println!("✓ Event logged:");
                    println!("  Type:    {}", event.event_type);
                    println!("  Content: {}", event.content);
                    if let Some(ctx) = &event.context {
                        println!("  Context: {}", ctx);
                    }
                }

                SessionCommands::Search { query, limit } => {
                    let events = service.search_events(&query, Some(limit)).await?;

                    if events.is_empty() {
                        println!("No events found matching: {}", query);
                    } else {
                        println!("Found {} events:\n", events.len());
                        for e in events {
                            let type_icon = match e.event_type {
                                EventType::Decision => "💡",
                                EventType::Observation => "👁️",
                                EventType::Error => "❗",
                                EventType::Command => "⚡",
                                EventType::FileChange => "📝",
                                EventType::ToolUse => "🔧",
                                EventType::Milestone => "🎯",
                                EventType::Prompt => "💬",
                                EventType::Plan => "📋",
                                EventType::ToolResult => "🧰",
                                EventType::Test => "✅",
                                EventType::Preference => "⚙️",
                                EventType::Rule => "📏",
                                EventType::Limitation => "⛔",
                                EventType::HandoffUpdate => "📦",
                                EventType::Custom(_) => "📌",
                            };
                            println!(
                                "  {} [{}] (session: {})",
                                type_icon, e.event_type, e.session_id
                            );
                            println!("    {}", e.content);
                            if let Some(ctx) = &e.context {
                                println!("    Context: {}", ctx);
                            }
                            println!();
                        }
                    }
                }

                SessionCommands::Stats => {
                    let stats = service.stats().await?;

                    println!("Session statistics:");
                    println!("  Total sessions:     {}", stats.total_sessions);
                    println!("  Active sessions:    {}", stats.active_sessions);
                    println!("  Completed sessions: {}", stats.completed_sessions);
                    println!("  Abandoned sessions: {}", stats.abandoned_sessions);
                    println!("  Total events:       {}", stats.total_events);

                    if !stats.events_by_type.is_empty() {
                        println!("\n  Events by type:");
                        for (event_type, count) in &stats.events_by_type {
                            println!("    {}: {}", event_type, count);
                        }
                    }
                }
            }
        }

        // =========================================================================
        // Layer 4: Tool Intelligence Commands
        // =========================================================================
        Commands::Tool { command } => {
            let config = StoreConfig::rocksdb(StoreConfig::default_data_dir());
            let db = connect_and_init(&config).await?;
            let service = ToolIntelService::new(db);
            service.init().await?;

            match command {
                ToolCommands::Log {
                    tool_name,
                    outcome,
                    context,
                    session,
                } => {
                    let session_id = if let Some(sid) = session {
                        Some(engram_core::id::Id::parse(&sid)?)
                    } else {
                        None
                    };

                    let usage = service
                        .log_usage(&tool_name, &context, outcome.into(), session_id.as_ref())
                        .await?;

                    println!("✓ Tool usage logged:");
                    println!("  ID:      {}", usage.id);
                    println!("  Tool:    {}", tool_name);
                    println!("  Outcome: {}", usage.outcome);
                    println!("  Context: {}", usage.context);
                }

                ToolCommands::Recommend { context } => {
                    let recommendations = service.get_recommendations(&context).await?;

                    if recommendations.is_empty() {
                        println!("No recommendations found for context: {}", context);
                        println!("\nTip: Log some tool usages first with 'engram tool log'");
                    } else {
                        println!("Tool recommendations for: {}\n", context);
                        for (i, rec) in recommendations.iter().enumerate() {
                            let confidence_pct = (rec.confidence * 100.0) as u32;
                            let bar = "█".repeat((confidence_pct / 10) as usize);
                            println!(
                                "  {}. {} ({}% confidence)",
                                i + 1,
                                rec.tool_name,
                                confidence_pct
                            );
                            println!("     {} {}", bar, rec.reason);
                            println!();
                        }
                    }
                }

                ToolCommands::Stats { tool_name } => {
                    if let Some(name) = tool_name {
                        let stats = service.get_tool_stats(&name).await?;

                        println!("Statistics for tool: {}\n", name);
                        println!("  Total usages:  {}", stats.total_usages);
                        println!("  Successes:     {}", stats.success_count);
                        println!("  Failures:      {}", stats.failure_count);
                        println!("  Success rate:  {:.1}%", stats.success_rate * 100.0);
                        println!("  Preferences:   {}", stats.preferences_count);
                    } else {
                        let stats = service.stats().await?;

                        println!("Tool intelligence statistics:\n");
                        println!("  Total usages:      {}", stats.usage_count);
                        println!("  Learned prefs:     {}", stats.preference_count);
                    }
                }

                ToolCommands::List { outcome, limit } => {
                    let outcome_filter: Option<ToolOutcome> = outcome.map(|o| o.into());
                    let usages = service
                        .list_usages(outcome_filter.as_ref(), Some(limit))
                        .await?;

                    if usages.is_empty() {
                        println!("No tool usages found.");
                        println!("\nTip: Log tool usages with 'engram tool log <tool-name> -o success -c \"context\"'");
                    } else {
                        println!("Recent tool usages ({}):\n", usages.len());
                        for usage in usages {
                            let outcome_icon = match usage.outcome {
                                ToolOutcome::Success => "✅",
                                ToolOutcome::Partial => "⚡",
                                ToolOutcome::Failed => "❌",
                                ToolOutcome::Switched => "🔄",
                            };
                            println!("  {} {} [{}]", outcome_icon, usage.tool_name, usage.outcome);
                            println!("    Context: {}", usage.context);
                            println!();
                        }
                    }
                }

                ToolCommands::Search { query, limit } => {
                    let usages = service.search_usages(&query, Some(limit)).await?;

                    if usages.is_empty() {
                        println!("No tool usages found matching: {}", query);
                    } else {
                        println!("Found {} usages matching: {}\n", usages.len(), query);
                        for usage in usages {
                            let outcome_icon = match usage.outcome {
                                ToolOutcome::Success => "✅",
                                ToolOutcome::Partial => "⚡",
                                ToolOutcome::Failed => "❌",
                                ToolOutcome::Switched => "🔄",
                            };
                            println!("  {} {} [{}]", outcome_icon, usage.tool_name, usage.outcome);
                            println!("    Context: {}", usage.context);
                            println!();
                        }
                    }
                }
            }
        }

        // =========================================================================
        // Layer 5: Session Coordination Commands
        // =========================================================================
        Commands::Coord { command } => {
            let config = StoreConfig::rocksdb(StoreConfig::default_data_dir());
            let db = connect_and_init(&config).await?;
            let service = CoordinationService::new(db);
            service.init().await?;

            match command {
                CoordCommands::Register {
                    session_id,
                    agent,
                    project,
                    goal,
                    components,
                } => {
                    let id = engram_core::id::Id::parse(&session_id)?;
                    let components = components.unwrap_or_default();

                    let session = if components.is_empty() {
                        service.register(&id, &agent, &project, &goal).await?
                    } else {
                        service
                            .register_with_components(&id, &agent, &project, &goal, components)
                            .await?
                    };

                    println!("✓ Session registered for coordination:");
                    println!("  Session: {}", session.session_id);
                    println!("  Agent:   {}", session.agent);
                    println!("  Project: {}", session.project);
                    println!("  Goal:    {}", session.goal);
                    if !session.components.is_empty() {
                        println!("  Components: {}", session.components.join(", "));
                    }
                }

                CoordCommands::Unregister { session_id } => {
                    let id = engram_core::id::Id::parse(&session_id)?;
                    service.unregister(&id).await?;
                    println!("✓ Session {} unregistered from coordination", session_id);
                }

                CoordCommands::Heartbeat { session_id } => {
                    let id = engram_core::id::Id::parse(&session_id)?;
                    service.heartbeat(&id).await?;
                    println!("✓ Heartbeat recorded for session {}", session_id);
                }

                CoordCommands::SetFile { session_id, file } => {
                    let id = engram_core::id::Id::parse(&session_id)?;
                    let conflicts = service.set_current_file(&id, file.as_deref()).await?;

                    if let Some(f) = &file {
                        println!("✓ Current file set to: {}", f);
                    } else {
                        println!("✓ Current file cleared");
                    }

                    if !conflicts.is_empty() {
                        println!("\n⚠️  Conflicts detected with other sessions:");
                        for conflict in conflicts {
                            println!(
                                "  - Session {} ({}) is also editing this file",
                                conflict.other_session_id, conflict.other_agent
                            );
                            println!("    Goal: {}", conflict.other_goal);
                        }
                    }
                }

                CoordCommands::SetComponents {
                    session_id,
                    components,
                } => {
                    let id = engram_core::id::Id::parse(&session_id)?;
                    let conflicts = service.set_components(&id, &components).await?;

                    println!("✓ Components set: {}", components.join(", "));

                    if !conflicts.is_empty() {
                        println!("\n⚠️  Conflicts detected with other sessions:");
                        for conflict in conflicts {
                            println!(
                                "  - Session {} ({}) has overlapping components: {}",
                                conflict.other_session_id,
                                conflict.other_agent,
                                conflict.overlapping_components.join(", ")
                            );
                            println!("    Goal: {}", conflict.other_goal);
                        }
                    }
                }

                CoordCommands::Conflicts { session_id } => {
                    let id = engram_core::id::Id::parse(&session_id)?;

                    // Get component conflicts
                    let component_conflicts = service.check_conflicts(&id).await?;

                    // Get session to check file conflicts
                    let session = service.get(&id).await?;
                    let file_conflicts = if let Some(s) = &session {
                        if let Some(file) = &s.current_file {
                            service.check_file_conflicts(&id, file).await?
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    };

                    let has_conflicts =
                        !component_conflicts.is_empty() || !file_conflicts.is_empty();

                    if !has_conflicts {
                        println!("✅ No conflicts detected for session {}", session_id);
                    } else {
                        println!("⚠️  Conflicts detected for session {}:\n", session_id);

                        if !component_conflicts.is_empty() {
                            println!("Component conflicts:");
                            for conflict in &component_conflicts {
                                println!(
                                    "  - Session {} ({}) has overlapping components: {}",
                                    conflict.other_session_id,
                                    conflict.other_agent,
                                    conflict.overlapping_components.join(", ")
                                );
                                println!("    Goal: {}", conflict.other_goal);
                            }
                        }

                        if !file_conflicts.is_empty() {
                            if !component_conflicts.is_empty() {
                                println!();
                            }
                            println!("File conflicts:");
                            for conflict in &file_conflicts {
                                println!(
                                    "  - Session {} ({}) is editing: {}",
                                    conflict.other_session_id,
                                    conflict.other_agent,
                                    conflict.other_current_file.as_deref().unwrap_or("unknown")
                                );
                                println!("    Goal: {}", conflict.other_goal);
                            }
                        }
                    }
                }

                CoordCommands::List { project } => {
                    let sessions = if let Some(p) = &project {
                        service.list_for_project(p).await?
                    } else {
                        service.list_active().await?
                    };

                    if sessions.is_empty() {
                        println!("No active sessions.");
                    } else {
                        println!("Active sessions ({}):\n", sessions.len());
                        for s in sessions {
                            println!("  {} ({})", s.session_id, s.agent);
                            println!("    Project: {}", s.project);
                            println!("    Goal:    {}", s.goal);
                            if !s.components.is_empty() {
                                println!("    Components: {}", s.components.join(", "));
                            }
                            if let Some(file) = &s.current_file {
                                println!("    Editing: {}", file);
                            }
                            println!();
                        }
                    }
                }

                CoordCommands::Stats => {
                    let stats = service.stats().await?;

                    println!("Coordination statistics:");
                    println!("  Active sessions: {}", stats.active_session_count);
                }
            }
        }

        // =========================================================================
        // Migration Commands
        // =========================================================================
        Commands::Migrate { command } => {
            match command {
                MigrateCommands::Embeddings { batch_size } => {
                    use engram_embed::Embedder;

                    println!("Starting embeddings migration...");
                    println!("Batch size: {}", batch_size);

                    let config = StoreConfig::rocksdb(StoreConfig::default_data_dir());
                    let db = connect_and_init(&config).await?;

                    // Initialize embedder
                    println!("Loading embedding model...");
                    let embedder = Embedder::default_model()?;
                    println!("✓ Model loaded");

                    // Get entity repo
                    let entity_repo = engram_store::EntityRepo::new(db.clone());

                    // Process entities
                    println!("\nProcessing entities...");
                    let entities = entity_repo.list_entities(None).await?;
                    let total_entities = entities.len();
                    let mut entities_updated = 0;
                    let mut entities_skipped = 0;

                    for (i, mut entity) in entities.into_iter().enumerate() {
                        // Skip if already has embedding
                        if entity.embedding.is_some() {
                            entities_skipped += 1;
                            continue;
                        }

                        // Generate embedding text
                        let embed_text = match &entity.description {
                            Some(desc) => format!("{}: {}", entity.name, desc),
                            None => entity.name.clone(),
                        };

                        // Generate embedding
                        match embedder.embed(&embed_text) {
                            Ok(embedding) => {
                                entity.embedding = Some(embedding);
                                entity.updated_at = OffsetDateTime::now_utc();
                                entity_repo.save_entity(&entity).await?;
                                entities_updated += 1;
                            }
                            Err(e) => {
                                eprintln!(
                                    "  Warning: Failed to embed entity '{}': {}",
                                    entity.name, e
                                );
                            }
                        }

                        // Progress update
                        if (i + 1) % batch_size == 0 || i + 1 == total_entities {
                            println!(
                                "  Entities: {}/{} processed ({} updated, {} skipped)",
                                i + 1,
                                total_entities,
                                entities_updated,
                                entities_skipped
                            );
                        }
                    }

                    // Process observations
                    println!("\nProcessing observations...");

                    // Get all entities to process their observations
                    let all_entities = entity_repo.list_entities(None).await?;
                    let mut total_observations = 0;
                    let mut observations_updated = 0;
                    let mut observations_skipped = 0;

                    for entity in &all_entities {
                        let observations = entity_repo.get_observations(&entity.id).await?;

                        for mut obs in observations {
                            total_observations += 1;

                            // Skip if already has embedding
                            if obs.embedding.is_some() {
                                observations_skipped += 1;
                                continue;
                            }

                            // Generate embedding text
                            let embed_text = match &obs.key {
                                Some(k) => format!("{} [{}]: {}", entity.name, k, obs.content),
                                None => format!("{}: {}", entity.name, obs.content),
                            };

                            // Generate embedding
                            match embedder.embed(&embed_text) {
                                Ok(embedding) => {
                                    obs.embedding = Some(embedding);
                                    obs.updated_at = OffsetDateTime::now_utc();
                                    entity_repo.add_observation(&obs).await?;
                                    observations_updated += 1;
                                }
                                Err(e) => {
                                    eprintln!("  Warning: Failed to embed observation: {}", e);
                                }
                            }
                        }
                    }

                    println!(
                        "  Observations: {} total ({} updated, {} skipped)",
                        total_observations, observations_updated, observations_skipped
                    );

                    println!("\n✓ Migration complete!");
                    println!(
                        "  Entities:     {} updated, {} skipped",
                        entities_updated, entities_skipped
                    );
                    println!(
                        "  Observations: {} updated, {} skipped",
                        observations_updated, observations_skipped
                    );
                }
            }
        }

        // =========================================================================
        // Layer 7: Work Management Commands
        // =========================================================================
        Commands::Work { command } => {
            let config = StoreConfig::rocksdb(StoreConfig::default_data_dir());
            let db = connect_and_init(&config).await?;
            let service = WorkService::with_defaults(db.clone())?;
            service.init().await?;

            match command {
                WorkCommands::Project { command } => match command {
                    WorkProjectCommands::Create { name, description } => {
                        let project = service
                            .create_project(&name, description.as_deref())
                            .await?;

                        println!("✓ Project created:");
                        println!("  ID:     {}", project.id);
                        println!("  Name:   {}", project.name);
                        println!("  Status: {}", project.status);
                        if let Some(desc) = &project.description {
                            println!("  Desc:   {}", desc);
                        }
                    }

                    WorkProjectCommands::List { status } => {
                        let status_filter: Option<ProjectStatus> = status.map(|s| s.into());
                        let projects = service.list_projects(status_filter).await?;

                        if projects.is_empty() {
                            println!("No projects found.");
                            println!("Use 'engram work project create <name>' to create one.");
                        } else {
                            println!("Projects ({}):\n", projects.len());
                            for p in projects {
                                let status_icon = match p.status {
                                    ProjectStatus::Planning => "📋",
                                    ProjectStatus::Active => "🟢",
                                    ProjectStatus::Completed => "✅",
                                    ProjectStatus::Archived => "📦",
                                };
                                println!("  {} {} [{}]", status_icon, p.name, p.status);
                                if let Some(desc) = &p.description {
                                    println!("    {}", desc);
                                }
                            }
                        }
                    }

                    WorkProjectCommands::Show { name } => {
                        let ctx = service.get_full_context(&name, None).await?;
                        let project = ctx.project;

                        let status_icon = match project.status {
                            ProjectStatus::Planning => "📋",
                            ProjectStatus::Active => "🟢",
                            ProjectStatus::Completed => "✅",
                            ProjectStatus::Archived => "📦",
                        };

                        println!("Project: {} {}", status_icon, project.name);
                        println!("  ID:      {}", project.id);
                        println!("  Status:  {}", project.status);
                        if let Some(desc) = &project.description {
                            println!("  Desc:    {}", desc);
                        }
                        println!("  Created: {}", project.created_at);

                        // Show tasks
                        let tasks = service.list_tasks(&name, None).await?;
                        if !tasks.is_empty() {
                            println!("\n  Tasks ({}):", tasks.len());
                            for t in &tasks {
                                let status_icon = match t.status {
                                    TaskStatus::Todo => "⬜",
                                    TaskStatus::InProgress => "🔄",
                                    TaskStatus::Blocked => "🚫",
                                    TaskStatus::Done => "✅",
                                };
                                println!("    {} {} [{}]", status_icon, t.name, t.status);
                                if let Some(jira) = &t.jira_key {
                                    println!("      JIRA: {}", jira);
                                }
                            }
                        }

                        // Show PRs
                        if !ctx.prs.is_empty() {
                            println!("\n  PRs ({}):", ctx.prs.len());
                            for pr in &ctx.prs {
                                let status_icon = match pr.status {
                                    PrStatus::Open => "🟡",
                                    PrStatus::Merged => "🟢",
                                    PrStatus::Closed => "🔴",
                                };
                                println!("    {} {} [{}]", status_icon, pr.url, pr.status);
                                if let Some(title) = &pr.title {
                                    println!("      {}", title);
                                }
                            }
                        }

                        // Show connected entities
                        if !ctx.connected_entities.is_empty() {
                            println!("\n  Connected Entities ({}):", ctx.connected_entities.len());
                            for e in &ctx.connected_entities {
                                println!("    {} ({})", e.name, e.entity_type);
                            }
                        }

                        // Show observations
                        if !ctx.project_observations.is_empty() {
                            println!("\n  Observations ({}):", ctx.project_observations.len());
                            for obs in &ctx.project_observations {
                                let key_str = obs.key.as_deref().unwrap_or("(no key)");
                                println!("    [{}] {}", key_str, obs.content);
                            }
                        }
                    }

                    WorkProjectCommands::Status { name, status } => {
                        service.update_project_status(&name, status.into()).await?;
                        println!(
                            "✓ Project '{}' status updated to: {}",
                            name,
                            ProjectStatus::from(status)
                        );
                    }

                    WorkProjectCommands::Connect {
                        project,
                        entity,
                        relation,
                    } => {
                        service
                            .connect_project_to_entity(&project, &entity, Some(&relation))
                            .await?;
                        println!(
                            "✓ Connected entity '{}' to project '{}' ({})",
                            entity, project, relation
                        );
                    }
                },

                WorkCommands::Task { command } => match command {
                    WorkTaskCommands::Create {
                        project,
                        name,
                        description,
                        jira,
                        priority,
                    } => {
                        let task = service
                            .create_task(&project, &name, description.as_deref(), jira.as_deref())
                            .await?;

                        // Update priority if not default
                        let task = if priority as u8 != TaskPriorityArg::Medium as u8 {
                            service
                                .update_task_priority(&task.name, priority.into())
                                .await?
                        } else {
                            task
                        };

                        println!("✓ Task created:");
                        println!("  ID:       {}", task.id);
                        println!("  Name:     {}", task.name);
                        println!("  Project:  {}", project);
                        println!("  Status:   {}", task.status);
                        println!("  Priority: {}", task.priority);
                        if let Some(jira) = &task.jira_key {
                            println!("  JIRA:     {}", jira);
                        }
                        if let Some(desc) = &task.description {
                            println!("  Desc:     {}", desc);
                        }
                    }

                    WorkTaskCommands::List { project, status } => {
                        let status_filter: Option<TaskStatus> = status.map(|s| s.into());
                        let tasks = service.list_tasks(&project, status_filter).await?;

                        if tasks.is_empty() {
                            println!("No tasks found for project '{}'.", project);
                            println!(
                                "Use 'engram work task create {} <name>' to create one.",
                                project
                            );
                        } else {
                            println!("Tasks for '{}' ({}):\n", project, tasks.len());
                            for t in tasks {
                                let status_icon = match t.status {
                                    TaskStatus::Todo => "⬜",
                                    TaskStatus::InProgress => "🔄",
                                    TaskStatus::Blocked => "🚫",
                                    TaskStatus::Done => "✅",
                                };
                                let priority_icon = match t.priority {
                                    TaskPriority::Low => "🔵",
                                    TaskPriority::Medium => "🟡",
                                    TaskPriority::High => "🟠",
                                    TaskPriority::Critical => "🔴",
                                };
                                println!(
                                    "  {} {} {} [{}]",
                                    status_icon, priority_icon, t.name, t.status
                                );
                                if let Some(jira) = &t.jira_key {
                                    print!("    JIRA: {}", jira);
                                }
                                if let Some(desc) = &t.description {
                                    print!("  {}", desc);
                                }
                                println!();
                            }
                        }
                    }

                    WorkTaskCommands::Show { name } => {
                        let task = service
                            .get_task(&name)
                            .await?
                            .ok_or_else(|| anyhow::anyhow!("Task not found: {}", name))?;

                        let status_icon = match task.status {
                            TaskStatus::Todo => "⬜",
                            TaskStatus::InProgress => "🔄",
                            TaskStatus::Blocked => "🚫",
                            TaskStatus::Done => "✅",
                        };

                        println!("Task: {} {}", status_icon, task.name);
                        println!("  ID:       {}", task.id);
                        println!("  Project:  {}", task.project_id);
                        println!("  Status:   {}", task.status);
                        println!("  Priority: {}", task.priority);
                        if let Some(jira) = &task.jira_key {
                            println!("  JIRA:     {}", jira);
                        }
                        if let Some(desc) = &task.description {
                            println!("  Desc:     {}", desc);
                        }
                        println!("  Created:  {}", task.created_at);

                        // Show connected entities
                        let entities = service.get_task_entities(&task.name).await?;
                        if !entities.is_empty() {
                            println!("\n  Connected Entities ({}):", entities.len());
                            for e in &entities {
                                println!("    {} ({})", e.name, e.entity_type);
                            }
                        }

                        // Show observations
                        let observations = service.get_task_observations(&task.name).await?;
                        if !observations.is_empty() {
                            println!("\n  Observations ({}):", observations.len());
                            for obs in &observations {
                                let key_str = obs.key.as_deref().unwrap_or("(no key)");
                                println!("    [{}] {}", key_str, obs.content);
                            }
                        }
                    }

                    WorkTaskCommands::Status { name, status } => {
                        service.update_task_status(&name, status.into()).await?;
                        println!(
                            "✓ Task '{}' status updated to: {}",
                            name,
                            TaskStatus::from(status)
                        );
                    }

                    WorkTaskCommands::Connect {
                        task,
                        entity,
                        relation,
                    } => {
                        service
                            .connect_task_to_entity(&task, &entity, Some(&relation))
                            .await?;
                        println!(
                            "✓ Connected entity '{}' to task '{}' ({})",
                            entity, task, relation
                        );
                    }
                },

                WorkCommands::Pr { command } => match command {
                    WorkPrCommands::Add {
                        project,
                        url,
                        task,
                        title,
                    } => {
                        let pr = service
                            .add_pr(&project, task.as_deref(), &url, title.as_deref())
                            .await?;

                        println!("✓ PR added:");
                        println!("  ID:      {}", pr.id);
                        println!("  URL:     {}", pr.url);
                        println!("  Repo:    {}", pr.repo);
                        println!("  PR #:    {}", pr.pr_number);
                        println!("  Status:  {}", pr.status);
                        if let Some(t) = &pr.title {
                            println!("  Title:   {}", t);
                        }
                    }

                    WorkPrCommands::List { project, task } => {
                        let prs = service.list_prs(&project, task.as_deref()).await?;

                        if prs.is_empty() {
                            println!("No PRs found.");
                            println!("Use 'engram work pr add {} <url>' to add one.", project);
                        } else {
                            println!("PRs ({}):\n", prs.len());
                            for pr in prs {
                                let status_icon = match pr.status {
                                    PrStatus::Open => "🟡",
                                    PrStatus::Merged => "🟢",
                                    PrStatus::Closed => "🔴",
                                };
                                println!("  {} {} [{}]", status_icon, pr.url, pr.status);
                                if let Some(title) = &pr.title {
                                    println!("    {}", title);
                                }
                            }
                        }
                    }

                    WorkPrCommands::Status { url, status } => {
                        service.update_pr_status(&url, status.into()).await?;
                        println!(
                            "✓ PR '{}' status updated to: {}",
                            url,
                            PrStatus::from(status)
                        );
                    }
                },

                WorkCommands::Observe { command } => match command {
                    WorkObserveCommands::Project {
                        project,
                        content,
                        key,
                        source: _,
                    } => {
                        let obs = service
                            .add_project_observation(&project, &content, key.as_deref())
                            .await?;

                        println!("✓ Observation added:");
                        println!("  Project: {}", project);
                        if let Some(k) = &obs.key {
                            println!("  Key:     {}", k);
                        }
                        println!("  Content: {}", obs.content);
                    }

                    WorkObserveCommands::Task {
                        task,
                        content,
                        key,
                        source: _,
                    } => {
                        let obs = service
                            .add_task_observation(&task, &content, key.as_deref())
                            .await?;

                        println!("✓ Observation added:");
                        println!("  Task:    {}", task);
                        if let Some(k) = &obs.key {
                            println!("  Key:     {}", k);
                        }
                        println!("  Content: {}", obs.content);
                    }
                },

                WorkCommands::Join {
                    project,
                    task,
                    session,
                } => {
                    let session_id = if let Some(sid) = session {
                        engram_core::id::Id::parse(&sid)?
                    } else {
                        // Generate a new session ID for CLI usage
                        engram_core::id::Id::new()
                    };

                    let ctx = service
                        .join_work(&session_id, &project, task.as_deref())
                        .await?;

                    println!("✓ Joined work context:");
                    println!("  Session: {}", ctx.session_id);
                    println!("  Project: {}", project);
                    if let Some(t) = task {
                        println!("  Task:    {}", t);
                    }
                }

                WorkCommands::Leave { session } => {
                    let session_id = if let Some(sid) = session {
                        engram_core::id::Id::parse(&sid)?
                    } else {
                        return Err(anyhow::anyhow!("Session ID required for leave"));
                    };

                    service.leave_work(&session_id).await?;
                    println!("✓ Left work context for session {}", session_id);
                }

                WorkCommands::Context { project, task } => {
                    let ctx = service.get_full_context(&project, task.as_deref()).await?;

                    println!("Work Context:\n");
                    println!("Project: {}", ctx.project.name);
                    println!("  Status: {}", ctx.project.status);
                    if let Some(desc) = &ctx.project.description {
                        println!("  Description: {}", desc);
                    }

                    if let Some(t) = &ctx.task {
                        println!("\nTask: {}", t.name);
                        println!("  Status: {}", t.status);
                        println!("  Priority: {}", t.priority);
                        if let Some(jira) = &t.jira_key {
                            println!("  JIRA: {}", jira);
                        }
                    }

                    if !ctx.prs.is_empty() {
                        println!("\nPRs:");
                        for pr in &ctx.prs {
                            println!("  {} [{}]", pr.url, pr.status);
                        }
                    }

                    if !ctx.connected_entities.is_empty() {
                        println!("\nConnected Entities:");
                        for e in &ctx.connected_entities {
                            println!("  {} ({})", e.name, e.entity_type);
                        }
                    }

                    if !ctx.project_observations.is_empty() {
                        println!("\nProject Observations:");
                        for obs in &ctx.project_observations {
                            let key_str = obs.key.as_deref().unwrap_or("(no key)");
                            println!("  [{}] {}", key_str, obs.content);
                        }
                    }

                    if !ctx.task_observations.is_empty() {
                        println!("\nTask Observations:");
                        for obs in &ctx.task_observations {
                            let key_str = obs.key.as_deref().unwrap_or("(no key)");
                            println!("  [{}] {}", key_str, obs.content);
                        }
                    }
                }

                WorkCommands::Stats => {
                    let stats = service.stats().await?;

                    println!("Work statistics:");
                    println!("  Projects:             {}", stats.project_count);
                    println!("  Tasks:                {}", stats.task_count);
                    println!("  PRs:                  {}", stats.pr_count);
                    println!(
                        "  Project Observations: {}",
                        stats.project_observation_count
                    );
                    println!("  Task Observations:    {}", stats.task_observation_count);
                }
            }
        }

        // =========================================================================
        // Memory OS Commands
        // =========================================================================
        Commands::Orient {
            project,
            cwd,
            prompt,
            agent,
            external_session_id,
            include_recent_commits,
            limit,
            store_project,
            data_dir,
            json,
        } => {
            let config = scoped_store_config(store_project.as_deref(), data_dir.as_deref())?;
            let db = connect_and_init(&config).await?;
            let service = MemoryService::new(db);
            service.init_schema().await?;
            let cwd = cwd_or_current(cwd)?.display().to_string();

            let packet = service
                .orient(OrientInput {
                    cwd: Some(cwd),
                    prompt,
                    project,
                    agent,
                    external_session_id: external_session_id_from_cli(external_session_id),
                    intent: None,
                    scenario_id: None,
                    arm: None,
                    include_recent_commits,
                    limit,
                })
                .await?;

            if json {
                println!("{}", serde_json::to_string_pretty(&packet)?);
            } else {
                print_orientation_packet(&packet);
            }
        }

        Commands::Harness { command } => {
            let service = HarnessService::new();
            match command {
                HarnessCommands::Status {
                    harness,
                    root,
                    json,
                } => {
                    let report = service.status(
                        harness.into(),
                        root.as_deref().map(std::path::Path::new),
                        &[],
                    )?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&report)?);
                    } else {
                        print_harness_status(&report);
                    }
                }
                HarnessCommands::Doctor {
                    harness,
                    root,
                    json,
                } => {
                    let report = service.doctor(
                        harness.into(),
                        root.as_deref().map(std::path::Path::new),
                        &[],
                    )?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&report)?);
                    } else {
                        print_harness_status(&report);
                    }
                }
                HarnessCommands::Render {
                    harness,
                    adapter,
                    json,
                } => {
                    let harness = harness.into();
                    if let Some(adapter) = adapter {
                        let adapters = service.render_adapters(harness, Some(&adapter));
                        if json {
                            println!("{}", serde_json::to_string_pretty(&adapters)?);
                        } else if adapters.is_empty() {
                            return Err(anyhow::anyhow!("No adapter matched '{}'", adapter));
                        } else {
                            for adapter in adapters {
                                println!(
                                    "# {} ({})\n{}",
                                    adapter.name, adapter.relative_path, adapter.contents
                                );
                            }
                        }
                    } else {
                        let policy = service.render_policy(harness)?;
                        println!("{policy}");
                    }
                }
                HarnessCommands::Install {
                    harness,
                    root,
                    write,
                    adopt_user_owned,
                    settings_target,
                    json,
                } => {
                    let report = service.install_with_options(
                        harness.into(),
                        root.as_deref().map(std::path::Path::new),
                        HarnessInstallOptions {
                            write,
                            adopt_user_owned,
                            settings_target,
                        },
                    )?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&report)?);
                    } else {
                        print_harness_install(&report);
                    }
                }
                HarnessCommands::Hook {
                    harness,
                    event,
                    session_id,
                    cwd,
                    transcript_path,
                    prompt,
                    tool_name,
                    tool_error,
                    tool_input_command,
                    file_path,
                    last_assistant_message,
                    compact_summary,
                    trigger,
                    reason,
                    stop_hook_active,
                    write_policy,
                    project,
                    model_provider,
                    model,
                    surface,
                    actor,
                    store_project,
                    data_dir,
                } => {
                    let hook_event = HarnessHookEvent {
                        harness: harness.into(),
                        hook_event_name: event,
                        session_id,
                        cwd,
                        transcript_path,
                        prompt,
                        tool_name,
                        tool_error,
                        tool_input_command,
                        file_path,
                        last_assistant_message,
                        compact_summary,
                        trigger,
                        reason,
                        stop_hook_active,
                        write_policy: Some(write_policy),
                        project,
                        model_provider: Some(model_provider),
                        model: Some(model),
                        surface: Some(surface),
                        actor: Some(actor),
                    };

                    if data_dir.is_none() {
                        if let Some(response) =
                            handle_harness_hook_via_daemon(&hook_event, store_project.as_deref())
                                .await?
                        {
                            println!("{}", serde_json::to_string_pretty(&response)?);
                            return Ok(());
                        }
                    }

                    let config =
                        scoped_store_config(store_project.as_deref(), data_dir.as_deref())?;
                    let db = connect_and_init(&config).await?;
                    let memory_service = MemoryService::new(db.clone());
                    memory_service.init_schema().await?;
                    let obligation_service = ObligationService::new(db.clone());
                    obligation_service.init_schema().await?;
                    let handoff_service = HandoffService::new(db);
                    handoff_service.init_schema().await?;

                    let outcome = service
                        .handle_hook_event(
                            hook_event,
                            HarnessHookServices {
                                memory: Some(&memory_service),
                                obligations: Some(&obligation_service),
                                handoff: Some(&handoff_service),
                            },
                        )
                        .await?;
                    println!("{}", serde_json::to_string_pretty(&outcome.response)?);
                }
            }
        }

        Commands::Lint {
            project,
            data_dir,
            command,
        } => {
            let config = scoped_store_config(project.as_deref(), data_dir.as_deref())?;
            let db = connect_and_init(&config).await?;
            let service = LintService::new(db);
            service.init_schema().await?;

            match command {
                LintCommands::Run {
                    vault_path,
                    limit,
                    json,
                }
                | LintCommands::List {
                    vault_path,
                    limit,
                    json,
                } => {
                    let report = service.run(LintOptions { vault_path, limit }).await?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&report)?);
                    } else {
                        print_lint_report(&report, false);
                    }
                }
                LintCommands::ApplySafe {
                    vault_path,
                    limit,
                    write,
                    json,
                } => {
                    let mut report = if write {
                        service
                            .apply_safe(LintOptions { vault_path, limit })
                            .await?
                    } else {
                        service.run(LintOptions { vault_path, limit }).await?
                    };
                    if !write {
                        report.applied_safe_actions = 0;
                    }
                    if json {
                        println!("{}", serde_json::to_string_pretty(&report)?);
                    } else {
                        print_lint_report(&report, !write);
                    }
                }
            }
        }

        Commands::Graph {
            project,
            data_dir,
            command,
        } => {
            let config = scoped_store_config(project.as_deref(), data_dir.as_deref())?;
            let db = connect_and_init(&config).await?;
            let service = GraphService::new(db);
            service.init_schema().await?;

            match command {
                GraphCommands::Around { node, depth, json } => {
                    let graph = service.around(&node, depth).await?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&graph)?);
                    } else {
                        print_subgraph(&graph);
                    }
                }
                GraphCommands::Path {
                    from,
                    to,
                    max_depth,
                    json,
                } => {
                    let path = service.path(&from, &to, max_depth).await?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&path)?);
                    } else if let Some(path) = path {
                        println!("Path nodes: {}", path.nodes.join(" -> "));
                        for edge in path.edges {
                            println!("  - {} --{}--> {}", edge.from, edge.relation, edge.to);
                        }
                    } else {
                        println!("No path found");
                    }
                }
                GraphCommands::Subgraph { node, depth, json } => {
                    let graph = service.subgraph(node.as_deref(), depth).await?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&graph)?);
                    } else {
                        print_subgraph(&graph);
                    }
                }
                GraphCommands::Export { node, depth } => {
                    let output = service.export_mermaid(node.as_deref(), depth).await?;
                    println!("{output}");
                }
            }
        }

        Commands::Handoff {
            project,
            data_dir,
            command,
        } => {
            let config = scoped_store_config(project.as_deref(), data_dir.as_deref())?;
            let db = connect_and_init(&config).await?;
            let service = HandoffService::new(db);
            service.init_schema().await?;

            match command {
                HandoffCommands::Get {
                    project,
                    session_id,
                    json,
                } => {
                    let session_id = session_id
                        .as_deref()
                        .map(Id::parse)
                        .transpose()
                        .map_err(|e| anyhow::anyhow!("Invalid session ID: {}", e))?;
                    let result = service.get(project.as_deref(), session_id).await?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    } else if let Some(item) = result.item {
                        print_memory_item(&item);
                    } else {
                        println!("No active handoff found");
                    }
                }
                HandoffCommands::Update {
                    project,
                    session_id,
                    content,
                    next_actions,
                    write,
                    json,
                    writer_harness,
                    model_provider,
                    model,
                } => {
                    let session_id = session_id
                        .as_deref()
                        .map(Id::parse)
                        .transpose()
                        .map_err(|e| anyhow::anyhow!("Invalid session ID: {}", e))?;
                    let result = service
                        .update(
                            project,
                            session_id,
                            content,
                            next_actions,
                            cli_agent_writer(&writer_harness, &model_provider, &model),
                            !write,
                        )
                        .await?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    } else {
                        println!(
                            "Handoff {}",
                            if result.written { "written" } else { "planned" }
                        );
                        println!("  Item: {}", result.item.id);
                        if let Some(previous_id) = result.previous_id {
                            println!("  Supersedes: {}", previous_id);
                        }
                    }
                }
                HandoffCommands::Compile {
                    session_id,
                    project,
                    write,
                    json,
                    writer_harness,
                    model_provider,
                    model,
                } => {
                    let session_id = Id::parse(&session_id)
                        .map_err(|e| anyhow::anyhow!("Invalid session ID: {}", e))?;
                    let result = service
                        .compile(
                            session_id,
                            project,
                            cli_agent_writer(&writer_harness, &model_provider, &model),
                            !write,
                        )
                        .await?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    } else {
                        println!("{}", result.content);
                        if let Some(update) = result.update {
                            println!("\nWritten handoff: {}", update.item.id);
                        }
                    }
                }
            }
        }

        Commands::Obligations {
            project,
            data_dir,
            command,
        } => {
            let config = scoped_store_config(project.as_deref(), data_dir.as_deref())?;
            let db = connect_and_init(&config).await?;
            let service = ObligationService::new(db);
            service.init_schema().await?;

            match command {
                ObligationCommands::Detect {
                    cwd,
                    prompt,
                    scope_project,
                    write,
                    limit,
                    json,
                    writer_harness,
                    model_provider,
                    model,
                } => {
                    let cwd = cwd_or_current(cwd)?.display().to_string();
                    let detection = service
                        .detect(ObligationDetectOptions {
                            cwd: Some(cwd),
                            prompt,
                            project: scope_project.or(project),
                            writer: cli_agent_writer(&writer_harness, &model_provider, &model),
                            write,
                            limit,
                        })
                        .await?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&detection)?);
                    } else {
                        print_obligation_detection(&detection);
                    }
                }
                ObligationCommands::Add {
                    kind,
                    title,
                    description,
                    scope_project,
                    trigger_kind,
                    trigger_summary,
                    trigger_target,
                    required_resolutions,
                    json,
                    writer_harness,
                    model_provider,
                    model,
                } => {
                    let scope = scope_project
                        .or(project)
                        .map(MemoryScope::project)
                        .unwrap_or(MemoryScope::Global);
                    let mut trigger = AgentObligationTrigger::new(trigger_kind, trigger_summary);
                    if let Some(target) = trigger_target {
                        trigger = trigger.with_target(target);
                    }
                    let mut obligation = AgentObligation::new(
                        AgentObligationKind::parse(&kind),
                        title,
                        description,
                        scope,
                        trigger,
                        cli_agent_writer(&writer_harness, &model_provider, &model),
                    );
                    for resolution in required_resolutions {
                        obligation = obligation.with_required_resolution(
                            AgentObligationResolutionKind::parse(&resolution),
                        );
                    }
                    let obligation = service.add(obligation).await?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&obligation)?);
                    } else {
                        print_obligation(&obligation);
                    }
                }
                ObligationCommands::List {
                    status,
                    limit,
                    json,
                } => {
                    let status = status
                        .as_deref()
                        .map(|value| {
                            AgentObligationStatus::parse(value)
                                .ok_or_else(|| anyhow::anyhow!("Invalid status: {}", value))
                        })
                        .transpose()?;
                    let obligations = service.list(status, None, None, limit).await?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&obligations)?);
                    } else {
                        print_obligation_list(&obligations);
                    }
                }
                ObligationCommands::Doctor { limit, json } => {
                    let report = service.doctor(None, None, limit).await?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&report)?);
                    } else {
                        print_obligation_doctor(&report);
                    }
                }
                ObligationCommands::Resolve {
                    id,
                    resolution,
                    summary,
                    actor,
                    json,
                } => {
                    let id = Id::parse(&id)
                        .map_err(|e| anyhow::anyhow!("Invalid obligation ID: {}", e))?;
                    let resolution = AgentObligationResolution::new(
                        AgentObligationResolutionKind::parse(&resolution),
                        summary,
                        actor,
                    );
                    let obligation = service.resolve(id, resolution).await?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&obligation)?);
                    } else {
                        print_obligation(&obligation);
                    }
                }
                ObligationCommands::Skip {
                    id,
                    reason,
                    actor,
                    json,
                } => {
                    let id = Id::parse(&id)
                        .map_err(|e| anyhow::anyhow!("Invalid obligation ID: {}", e))?;
                    let resolution = AgentObligationResolution::new(
                        AgentObligationResolutionKind::SkippedWithReason,
                        reason,
                        actor,
                    );
                    let obligation = service.skip(id, resolution).await?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&obligation)?);
                    } else {
                        print_obligation(&obligation);
                    }
                }
            }
        }

        Commands::Memory {
            project,
            data_dir,
            command,
        } => {
            let config = scoped_store_config(project.as_deref(), data_dir.as_deref())?;
            let db = connect_and_init(&config).await?;
            let service = MemoryService::new(db);
            service.init_schema().await?;

            match command {
                MemoryCommands::List {
                    status,
                    limit,
                    json,
                } => {
                    let items = service.list_memory(status.map(Into::into), limit).await?;

                    if json {
                        println!("{}", serde_json::to_string_pretty(&items)?);
                    } else {
                        print_memory_items("Memory items", &items);
                    }
                }
                MemoryCommands::Get { id, json } => {
                    let id = Id::parse(&id)
                        .map_err(|e| anyhow::anyhow!("Invalid memory item ID: {}", e))?;
                    let item = service
                        .get_memory(&id)
                        .await?
                        .ok_or_else(|| anyhow::anyhow!("Memory item not found: {}", id))?;

                    if json {
                        println!("{}", serde_json::to_string_pretty(&item)?);
                    } else {
                        print_memory_item(&item);
                    }
                }
                MemoryCommands::Review { limit, json } => {
                    let items = service.list_memory_needing_review(limit).await?;

                    if json {
                        println!("{}", serde_json::to_string_pretty(&items)?);
                    } else {
                        print_memory_items("Memory items needing review", &items);
                    }
                }
                MemoryCommands::Cursor { json } => {
                    let cursor = service.current_cursor().await?;

                    if json {
                        println!("{}", serde_json::to_string_pretty(&cursor)?);
                    } else {
                        print_memory_cursor(&cursor);
                    }
                }
                MemoryCommands::ChangesSince {
                    timestamp,
                    commit_id,
                    limit,
                    writer_harness,
                    model,
                    surface,
                    writer_session_id,
                    relevance_project,
                    cwd,
                    query,
                    external_session_id,
                    json,
                } => {
                    let timestamp = parse_rfc3339_timestamp(&timestamp)?;
                    let commit_id = commit_id
                        .as_deref()
                        .map(Id::parse)
                        .transpose()
                        .map_err(|e| anyhow::anyhow!("Invalid commit ID: {}", e))?;
                    let writer_session_id = writer_session_id
                        .as_deref()
                        .map(Id::parse)
                        .transpose()
                        .map_err(|e| anyhow::anyhow!("Invalid writer session ID: {}", e))?;
                    let changes = service
                        .changes_since_with_options(
                            MemoryCursor {
                                commit_id,
                                timestamp,
                            },
                            limit,
                            MemoryChangesSinceOptions {
                                writer_harness,
                                model,
                                surface,
                                writer_session_id,
                                project: relevance_project,
                                cwd,
                                query,
                                intent: None,
                                external_session_id: external_session_id_from_cli(
                                    external_session_id,
                                ),
                            },
                        )
                        .await?;

                    if json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "since": changes.since,
                                "next_cursor": changes.next_cursor,
                                "trace_id": changes.trace_id,
                                "item_count": changes.items.len(),
                                "commit_count": changes.commits.len(),
                                "item_relevance": changes.item_relevance,
                                "items": changes.items,
                                "commits": changes.commits
                            }))?
                        );
                    } else {
                        print_memory_changes(&changes);
                    }
                }
                MemoryCommands::Log { limit, json } => {
                    let commits = service.list_commits(limit).await?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&commits)?);
                    } else {
                        println!("Knowledge commits");
                        for commit in commits {
                            println!("  {} - {}", commit.id, commit.message);
                        }
                    }
                }
                MemoryCommands::Diff { commit_id, json } => {
                    let commit_id = Id::parse(&commit_id)
                        .map_err(|e| anyhow::anyhow!("Invalid commit ID: {}", e))?;
                    let commit = service.get_commit(&commit_id).await?.ok_or_else(|| {
                        anyhow::anyhow!("Knowledge commit not found: {}", commit_id)
                    })?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&commit)?);
                    } else {
                        println!("Commit: {} - {}", commit.id, commit.message);
                        for change in commit.changes {
                            println!(
                                "  - {} {}: {}",
                                change.change_type, change.title, change.summary
                            );
                        }
                    }
                }
                MemoryCommands::WriterStats { json } => {
                    let stats = service.writer_stats().await?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&stats)?);
                    } else {
                        println!("Memory writer stats");
                        for stat in stats {
                            println!(
                                "  {} / {} / {} / {}: {}",
                                stat.harness,
                                stat.model_provider,
                                stat.model,
                                stat.surface.as_deref().unwrap_or("unknown"),
                                stat.count
                            );
                        }
                    }
                }
                MemoryCommands::Archive {
                    id,
                    reason,
                    archived_by,
                    json,
                } => {
                    let id = Id::parse(&id)
                        .map_err(|e| anyhow::anyhow!("Invalid memory item ID: {}", e))?;
                    let item = service.archive_memory(&id, reason, archived_by).await?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&item)?);
                    } else {
                        println!("Archived memory item: {}", item.id);
                    }
                }
                MemoryCommands::ExportVault { path } => {
                    let export = service.export_vault(std::path::Path::new(&path)).await?;

                    println!("✓ Memory vault exported");
                    println!("  Root:                 {}", export.root);
                    println!("  Files written:        {}", export.file_count());
                    println!("  Files skipped:        {}", export.files_skipped.len());
                    println!("  Memory items:         {}", export.memory_item_count);
                    println!("  Knowledge commits:    {}", export.knowledge_commit_count);
                    println!("  Repositories:         {}", export.repository_count);
                    println!("  Entities:             {}", export.entity_count);
                    println!("  Projects:             {}", export.project_count);
                    if !export.files_skipped.is_empty() {
                        println!("Skipped non-generated files:");
                        for path in export.files_skipped {
                            println!("  - {}", path);
                        }
                    }
                }
                MemoryCommands::MigrationInventory {
                    project_filter,
                    limit,
                    exclude_reviewed_path,
                    json,
                    no_entity_observations,
                    no_session_history,
                    no_work_observations,
                } => {
                    if no_entity_observations && no_session_history && no_work_observations {
                        return Err(anyhow::anyhow!(
                            "At least one migration inventory source layer must be included"
                        ));
                    }
                    let inventory = service
                        .migration_inventory(MigrationInventoryOptions {
                            project_filter,
                            limit,
                            include_entity_observations: !no_entity_observations,
                            include_session_history: !no_session_history,
                            include_work_observations: !no_work_observations,
                            exclude_reviewed_path,
                        })
                        .await?;

                    if json {
                        println!("{}", serde_json::to_string_pretty(&inventory)?);
                    } else {
                        print_migration_inventory(&inventory);
                    }
                }
                MemoryCommands::MigrationReviewExport {
                    path,
                    project_filter,
                    limit,
                    exclude_reviewed_path,
                    json,
                    no_entity_observations,
                    no_session_history,
                    no_work_observations,
                } => {
                    if no_entity_observations && no_session_history && no_work_observations {
                        return Err(anyhow::anyhow!(
                            "At least one migration review source layer must be included"
                        ));
                    }
                    let export = service
                        .export_migration_review(
                            std::path::Path::new(&path),
                            MigrationInventoryOptions {
                                project_filter,
                                limit,
                                include_entity_observations: !no_entity_observations,
                                include_session_history: !no_session_history,
                                include_work_observations: !no_work_observations,
                                exclude_reviewed_path,
                            },
                        )
                        .await?;

                    if json {
                        println!("{}", serde_json::to_string_pretty(&export)?);
                    } else {
                        print_migration_review_export(&export);
                    }
                }
                MemoryCommands::MigrationReviewStatus { path, json } => {
                    let status = service
                        .migration_review_status(std::path::Path::new(&path))
                        .await?;

                    if json {
                        println!("{}", serde_json::to_string_pretty(&status)?);
                    } else {
                        print_migration_review_status(&status);
                    }
                }
                MemoryCommands::MigrationReviewApply {
                    path,
                    write,
                    json,
                    no_commit,
                    writer_harness,
                    model_provider,
                    model,
                } => {
                    let apply = service
                        .apply_migration_review(
                            std::path::Path::new(&path),
                            MigrationReviewApplyOptions {
                                dry_run: !write,
                                writer: cli_migration_writer(
                                    &writer_harness,
                                    &model_provider,
                                    &model,
                                ),
                                create_commit: !no_commit,
                            },
                        )
                        .await?;

                    if json {
                        println!("{}", serde_json::to_string_pretty(&apply)?);
                    } else {
                        print_migration_review_apply(&apply);
                    }
                }
                MemoryCommands::DigestExtractionApply {
                    path,
                    write,
                    json,
                    no_commit,
                    writer_harness,
                    model_provider,
                    model,
                } => {
                    let apply = service
                        .apply_digest_extraction_review(
                            std::path::Path::new(&path),
                            DigestExtractionReviewApplyOptions {
                                dry_run: !write,
                                writer: cli_migration_writer(
                                    &writer_harness,
                                    &model_provider,
                                    &model,
                                ),
                                create_commit: !no_commit,
                            },
                        )
                        .await?;

                    if json {
                        println!("{}", serde_json::to_string_pretty(&apply)?);
                    } else {
                        print_digest_extraction_review_apply(&apply);
                    }
                }
                MemoryCommands::DistillSession {
                    session_id,
                    json,
                    writer_harness,
                    model_provider,
                    model,
                } => {
                    let session_id = Id::parse(&session_id)
                        .map_err(|e| anyhow::anyhow!("Invalid session ID: {}", e))?;
                    let distillation = service
                        .distill_session(
                            session_id,
                            cli_agent_writer(&writer_harness, &model_provider, &model),
                        )
                        .await?;

                    if json {
                        println!("{}", serde_json::to_string_pretty(&distillation)?);
                    } else {
                        println!("Session distillation candidates");
                        println!("  Session:    {}", distillation.session_id);
                        println!("  Candidates: {}", distillation.candidates.len());
                        println!("  Warning:    {}", distillation.warning);
                        print_memory_items("Candidates needing review", &distillation.candidates);
                    }
                }
            }
        }

        // =========================================================================
        // Memory OS Vault Commands
        // =========================================================================
        Commands::Vault {
            project,
            data_dir,
            command,
        } => {
            let config = scoped_store_config(project.as_deref(), data_dir.as_deref())?;
            let db = connect_and_init(&config).await?;
            let service = MemoryService::new(db);
            service.init_schema().await?;

            match command {
                VaultCommands::Init { path, json } => {
                    let init = service.init_vault(std::path::Path::new(&path)).await?;

                    if json {
                        println!("{}", serde_json::to_string_pretty(&init)?);
                    } else {
                        println!("✓ Memory vault initialized");
                        println!("  Root: {}", init.root);
                        println!("  Directories created: {}", init.directories_created.len());
                        println!(
                            "  Directories existing: {}",
                            init.directories_existing.len()
                        );
                        if !init.directories_created.is_empty() {
                            println!("Created:");
                            for path in init.directories_created {
                                println!("  - {}", path);
                            }
                        }
                    }
                }
                VaultCommands::Compile { path, json } => {
                    let export = service.export_vault(std::path::Path::new(&path)).await?;

                    if json {
                        println!("{}", serde_json::to_string_pretty(&export)?);
                    } else {
                        println!("✓ Memory vault compiled");
                        println!("  Root:                 {}", export.root);
                        println!("  Files written:        {}", export.file_count());
                        println!("  Files skipped:        {}", export.files_skipped.len());
                        println!("  Memory items:         {}", export.memory_item_count);
                        println!("  Knowledge commits:    {}", export.knowledge_commit_count);
                        println!("  Repositories:         {}", export.repository_count);
                        println!("  Entities:             {}", export.entity_count);
                        println!("  Projects:             {}", export.project_count);
                        if !export.files_skipped.is_empty() {
                            println!("Skipped non-generated files:");
                            for path in export.files_skipped {
                                println!("  - {}", path);
                            }
                        }
                    }
                }
                VaultCommands::Status { path, json } => {
                    let status = service.vault_status(std::path::Path::new(&path)).await?;

                    if json {
                        println!("{}", serde_json::to_string_pretty(&status)?);
                    } else {
                        println!("Memory vault status");
                        println!("  Root:              {}", status.root);
                        println!("  Exists:            {}", status.exists);
                        println!("  Initialized:       {}", status.initialized);
                        println!("  Total files:       {}", status.total_file_count);
                        println!("  Generated files:   {}", status.generated_file_count);
                        println!("  User files:        {}", status.user_file_count);
                        println!(
                            "  Expected generated files: {}",
                            status.expected_generated_file_count
                        );
                        println!("  Memory items:      {}", status.memory_item_count);
                        println!("  Knowledge commits: {}", status.knowledge_commit_count);
                        println!("  Repositories:      {}", status.repository_count);
                        println!("  Entities:          {}", status.entity_count);
                        println!("  Projects:          {}", status.project_count);
                        if !status.missing_directories.is_empty() {
                            println!("Missing directories:");
                            for path in status.missing_directories {
                                println!("  - {}", path);
                            }
                        }
                    }
                }
                VaultCommands::Page { path, page, json } => {
                    let page = service
                        .vault_page(std::path::Path::new(&path), &page)
                        .await?
                        .ok_or_else(|| anyhow::anyhow!("Vault page not found"))?;

                    if json {
                        println!("{}", serde_json::to_string_pretty(&page)?);
                    } else {
                        print!("{}", page.contents);
                    }
                }
            }
        }

        // =========================================================================
        // Digest Source Commands
        // =========================================================================
        Commands::Digest { command } => match command {
            DigestCommands::Inventory {
                root_path,
                limit,
                include_operational,
                json,
            } => {
                let mut options = DigestInventoryOptions::new(std::path::PathBuf::from(root_path));
                options.limit = limit;
                options.include_operational = include_operational;
                let inventory = DigestService::new().inventory(options)?;

                if json {
                    println!("{}", serde_json::to_string_pretty(&inventory)?);
                } else {
                    print_digest_inventory(&inventory);
                }
            }
            DigestCommands::ReviewExport {
                root_path,
                output_path,
                limit,
                include_operational,
                json,
            } => {
                let mut options = DigestInventoryOptions::new(std::path::PathBuf::from(root_path));
                options.limit = limit;
                options.include_operational = include_operational;
                let export = DigestService::new()
                    .export_review_batch(std::path::PathBuf::from(output_path), options)?;

                if json {
                    println!("{}", serde_json::to_string_pretty(&export)?);
                } else {
                    print_digest_review_export(&export);
                }
            }
            DigestCommands::ReviewApply { path, json } => {
                let apply =
                    DigestService::new().apply_review_batch(std::path::PathBuf::from(path))?;

                if json {
                    println!("{}", serde_json::to_string_pretty(&apply)?);
                } else {
                    print_digest_review_apply(&apply);
                }
            }
            DigestCommands::ExtractionPlan {
                review_path,
                output_path,
                max_source_bytes,
                max_candidates_per_source,
                max_candidate_chars,
                json,
            } => {
                let defaults = DigestExtractionOptions::default();
                let plan = DigestService::new().plan_extraction(
                    std::path::PathBuf::from(review_path),
                    std::path::PathBuf::from(output_path),
                    DigestExtractionOptions {
                        max_source_bytes: max_source_bytes.unwrap_or(defaults.max_source_bytes),
                        max_candidates_per_source: max_candidates_per_source
                            .unwrap_or(defaults.max_candidates_per_source),
                        max_candidate_chars: max_candidate_chars
                            .unwrap_or(defaults.max_candidate_chars),
                    },
                )?;

                if json {
                    println!("{}", serde_json::to_string_pretty(&plan)?);
                } else {
                    print_digest_extraction_plan(&plan);
                }
            }
            DigestCommands::SourceIndex {
                review_path,
                write,
                project,
                data_dir,
                max_source_bytes,
                json,
            } => {
                let defaults = DigestSourceIndexOptions::default();
                let plan = DigestService::new().plan_source_index(
                    std::path::PathBuf::from(&review_path),
                    DigestSourceIndexOptions {
                        max_source_bytes: max_source_bytes.unwrap_or(defaults.max_source_bytes),
                    },
                )?;

                let mut indexed_documents = 0usize;
                if write {
                    let config = scoped_store_config(project.as_deref(), data_dir.as_deref())?;
                    let db = connect_and_init(&config).await?;
                    let service = DocumentService::with_defaults(db)?;
                    service.init_schema().await?;
                    for document in &plan.documents {
                        service
                            .index_content(
                                &document.document_path,
                                Some(document.title.clone()),
                                document.indexed_content.clone(),
                            )
                            .await?;
                        indexed_documents += 1;
                    }
                }

                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "plan": plan,
                            "dry_run": !write,
                            "indexed_documents": indexed_documents
                        }))?
                    );
                } else {
                    print_digest_source_index_plan(&plan, indexed_documents);
                }
            }
        },

        // =========================================================================
        // Repository Topology Commands
        // =========================================================================
        Commands::Repo {
            project,
            data_dir,
            command,
        } => {
            let config = scoped_store_config(project.as_deref(), data_dir.as_deref())?;
            let db = connect_and_init(&config).await?;
            let service = RepositoryService::new(db);
            service.init_schema().await?;

            match command {
                RepoCommands::Detect { cwd } => {
                    let cwd = cwd_or_current(cwd)?;
                    let detection = service.detect_repository(&cwd).await?;

                    println!("✓ Repository detected and registered");
                    println!("Detected root: {}", detection.detected_root);
                    print_repository_context(&detection.context);
                    if !detection.warnings.is_empty() {
                        println!("Warnings:");
                        for warning in detection.warnings {
                            println!("  - {}", warning);
                        }
                    }
                }

                RepoCommands::Context { cwd } => {
                    let cwd = cwd_or_current(cwd)?;
                    match service.resolve_cwd(&cwd).await? {
                        Some(context) => print_repository_context(&context),
                        None => {
                            println!("No registered repository context matched {}", cwd.display());
                        }
                    }
                }

                RepoCommands::Register {
                    name,
                    remote,
                    default_branch,
                    description,
                } => {
                    let repository = service
                        .register_repository(
                            &name,
                            remote.as_deref(),
                            default_branch.as_deref(),
                            description.as_deref(),
                        )
                        .await?;

                    println!("✓ Repository registered:");
                    println!("  ID:   {}", repository.id);
                    println!("  Name: {}", repository.name);
                    if let Some(remote_url) = repository.remote_url {
                        println!("  Remote: {}", remote_url);
                    }
                    println!("  Provider: {}", repository.provider);
                    if let Some(default_branch) = repository.default_branch {
                        println!("  Default branch: {}", default_branch);
                    }
                }

                RepoCommands::List { limit } => {
                    let repositories = service.list_repositories(limit).await?;
                    if repositories.is_empty() {
                        println!("No repositories registered.");
                    } else {
                        println!("Repositories ({}):\n", repositories.len());
                        for repository in repositories {
                            println!("  {} ({})", repository.name, repository.id);
                            if let Some(remote_url) = repository.remote_url {
                                println!("    Remote: {}", remote_url);
                            }
                            println!("    Provider: {}", repository.provider);
                        }
                    }
                }

                RepoCommands::ComponentAdd {
                    repo,
                    repo_id,
                    name,
                    path,
                    kind,
                    description,
                } => {
                    let repo_id = parse_optional_repo_id(repo_id.as_deref())?;
                    let component = service
                        .register_component(
                            repo_id.as_ref(),
                            repo.as_deref(),
                            &name,
                            &path,
                            kind.as_deref(),
                            description.as_deref(),
                        )
                        .await?;

                    println!("✓ Component registered:");
                    println!("  ID:   {}", component.id);
                    println!("  Name: {}", component.name);
                    println!("  Path: {}", component.path);
                    if let Some(kind) = component.kind {
                        println!("  Kind: {}", kind);
                    }
                }

                RepoCommands::LinkProject {
                    project,
                    repo,
                    repo_id,
                    role,
                    component_path,
                } => {
                    let repo_id = parse_optional_repo_id(repo_id.as_deref())?;
                    let link = service
                        .link_project(
                            &project,
                            repo_id.as_ref(),
                            repo.as_deref(),
                            role.into(),
                            component_path.as_deref(),
                        )
                        .await?;

                    println!("✓ Project linked to repository:");
                    println!("  Project: {}", link.project_name);
                    println!("  Repository ID: {}", link.repository_id);
                    println!("  Role: {}", link.role);
                    if let Some(component_path) = link.component_path {
                        println!("  Component path: {}", component_path);
                    }
                }
                RepoCommands::MigrationInventory {
                    project_filter,
                    limit,
                    json,
                    no_entity_observations,
                    no_session_history,
                    no_work_records,
                } => {
                    if no_entity_observations && no_session_history && no_work_records {
                        return Err(anyhow::anyhow!(
                            "At least one repository migration inventory source layer must be included"
                        ));
                    }
                    let inventory = service
                        .migration_inventory(RepositoryMigrationOptions {
                            project_filter,
                            limit,
                            include_entity_observations: !no_entity_observations,
                            include_session_history: !no_session_history,
                            include_work_records: !no_work_records,
                        })
                        .await?;

                    if json {
                        println!("{}", serde_json::to_string_pretty(&inventory)?);
                    } else {
                        print_repository_migration_inventory(&inventory);
                    }
                }
                RepoCommands::MigrationReviewExport {
                    path,
                    project_filter,
                    limit,
                    json,
                    no_entity_observations,
                    no_session_history,
                    no_work_records,
                } => {
                    if no_entity_observations && no_session_history && no_work_records {
                        return Err(anyhow::anyhow!(
                            "At least one repository migration review source layer must be included"
                        ));
                    }
                    let export = service
                        .export_migration_review(
                            std::path::Path::new(&path),
                            RepositoryMigrationOptions {
                                project_filter,
                                limit,
                                include_entity_observations: !no_entity_observations,
                                include_session_history: !no_session_history,
                                include_work_records: !no_work_records,
                            },
                        )
                        .await?;

                    if json {
                        println!("{}", serde_json::to_string_pretty(&export)?);
                    } else {
                        print_repository_migration_review_export(&export);
                    }
                }
                RepoCommands::MigrationReviewStatus { path, json } => {
                    let status = service
                        .migration_review_status(std::path::Path::new(&path))
                        .await?;

                    if json {
                        println!("{}", serde_json::to_string_pretty(&status)?);
                    } else {
                        print_repository_migration_review_status(&status);
                    }
                }
                RepoCommands::MigrationReviewApply {
                    path,
                    write,
                    json,
                    no_commit,
                    writer_harness,
                    model_provider,
                    model,
                } => {
                    let apply = service
                        .apply_migration_review(
                            std::path::Path::new(&path),
                            RepositoryMigrationReviewApplyOptions {
                                dry_run: !write,
                                writer: Some(cli_migration_writer(
                                    &writer_harness,
                                    &model_provider,
                                    &model,
                                )),
                                create_commit: !no_commit,
                            },
                        )
                        .await?;

                    if json {
                        println!("{}", serde_json::to_string_pretty(&apply)?);
                    } else {
                        print_repository_migration_review_apply(&apply);
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_rfc3339_timestamp_error_names_cursor_timestamp() {
        let error = parse_rfc3339_timestamp("not-a-timestamp")
            .expect_err("invalid timestamp should fail")
            .to_string();

        assert!(error.contains("Invalid RFC3339 timestamp"));
        assert!(error.contains("memory_cursor.timestamp"));
        assert!(error.contains("engram memory cursor"));
    }

    #[test]
    fn external_session_id_resolution_uses_flag_before_env() {
        assert_eq!(
            resolve_external_session_id(
                Some(" codex://threads/cli ".to_string()),
                Some("codex://threads/env".to_string()),
            )
            .as_deref(),
            Some("codex://threads/cli")
        );
    }

    #[test]
    fn external_session_id_resolution_uses_env_when_flag_omitted() {
        assert_eq!(
            resolve_external_session_id(None, Some(" codex://threads/env ".to_string())).as_deref(),
            Some("codex://threads/env")
        );
    }

    #[test]
    fn external_session_id_resolution_uses_flag_before_codex_thread_id() {
        assert_eq!(
            resolve_external_session_id_with_envs(
                Some(" codex://threads/cli ".to_string()),
                None,
                Some("claude-session".to_string()),
                true,
                Some("codex-thread".to_string()),
                true,
            )
            .as_deref(),
            Some("codex://threads/cli")
        );
    }

    #[test]
    fn external_session_id_resolution_uses_env_before_codex_thread_id() {
        assert_eq!(
            resolve_external_session_id_with_envs(
                None,
                Some(" codex://threads/env ".to_string()),
                Some("claude-session".to_string()),
                true,
                Some("codex-thread".to_string()),
                true,
            )
            .as_deref(),
            Some("codex://threads/env")
        );
    }

    #[test]
    fn external_session_id_resolution_uses_claude_code_session_id_when_guarded() {
        assert_eq!(
            resolve_external_session_id_with_envs(
                None,
                None,
                Some(" claude-session_123 ".to_string()),
                true,
                None,
                false,
            )
            .as_deref(),
            Some("claude-code://sessions/claude-session_123")
        );
    }

    #[test]
    fn external_session_id_resolution_rejects_unguarded_claude_code_session_id() {
        assert_eq!(
            resolve_external_session_id_with_envs(
                None,
                None,
                Some("claude-session".to_string()),
                false,
                None,
                false,
            ),
            None
        );
    }

    #[test]
    fn external_session_id_resolution_rejects_unsafe_claude_code_session_ids() {
        assert_eq!(
            resolve_external_session_id_with_envs(
                None,
                None,
                Some(" \t\n ".to_string()),
                true,
                None,
                false,
            ),
            None
        );
        assert_eq!(
            resolve_external_session_id_with_envs(
                None,
                None,
                Some("session/one".to_string()),
                true,
                None,
                false,
            ),
            None
        );
        assert_eq!(
            resolve_external_session_id_with_envs(
                None,
                None,
                Some("x".repeat(MAX_CLAUDE_CODE_SESSION_ID_LEN + 1)),
                true,
                None,
                false,
            ),
            None
        );
    }

    #[test]
    fn external_session_id_resolution_uses_claude_code_session_id_before_codex_thread_id() {
        assert_eq!(
            resolve_external_session_id_with_envs(
                None,
                None,
                Some("claude-session".to_string()),
                true,
                Some("codex-thread".to_string()),
                true,
            )
            .as_deref(),
            Some("claude-code://sessions/claude-session")
        );
    }

    #[test]
    fn external_session_id_resolution_uses_codex_thread_id_when_claude_code_id_is_invalid() {
        assert_eq!(
            resolve_external_session_id_with_envs(
                None,
                None,
                Some("claude/session".to_string()),
                true,
                Some("codex-thread".to_string()),
                true,
            )
            .as_deref(),
            Some("codex://threads/codex-thread")
        );
    }

    #[test]
    fn external_session_id_resolution_uses_codex_thread_id_when_guarded() {
        assert_eq!(
            resolve_external_session_id_with_envs(
                None,
                None,
                None,
                false,
                Some(" codex-thread_123 ".to_string()),
                true,
            )
            .as_deref(),
            Some("codex://threads/codex-thread_123")
        );
    }

    #[test]
    fn external_session_id_resolution_rejects_unguarded_codex_thread_id() {
        assert_eq!(
            resolve_external_session_id_with_envs(
                None,
                None,
                None,
                false,
                Some("codex-thread".to_string()),
                false,
            ),
            None
        );
    }

    #[test]
    fn external_session_id_resolution_rejects_unsafe_codex_thread_ids() {
        assert_eq!(
            resolve_external_session_id_with_envs(
                None,
                None,
                None,
                false,
                Some(" \t\n ".to_string()),
                true,
            ),
            None
        );
        assert_eq!(
            resolve_external_session_id_with_envs(
                None,
                None,
                None,
                false,
                Some("thread/one".to_string()),
                true,
            ),
            None
        );
        assert_eq!(
            resolve_external_session_id_with_envs(
                None,
                None,
                None,
                false,
                Some("x".repeat(MAX_CODEX_THREAD_ID_LEN + 1)),
                true,
            ),
            None
        );
    }

    #[test]
    fn external_session_id_resolution_whitespace_flag_does_not_use_codex_thread_id() {
        assert_eq!(
            resolve_external_session_id_with_envs(
                Some("   ".to_string()),
                None,
                Some("claude-session".to_string()),
                true,
                Some("codex-thread".to_string()),
                true,
            ),
            None
        );
    }

    #[test]
    fn external_session_id_resolution_detects_codex_host_markers() {
        assert!(codex_host_marker_present(Some("1".to_string()), None, None));
        assert!(codex_host_marker_present(
            None,
            Some("Codex Desktop".to_string()),
            None,
        ));
        assert!(codex_host_marker_present(
            None,
            None,
            Some("com.openai.codex".to_string()),
        ));
        assert!(!codex_host_marker_present(
            None,
            Some("other host".to_string()),
            Some("com.example.other".to_string()),
        ));
    }

    #[test]
    fn external_session_id_resolution_detects_claude_code_host_marker() {
        assert!(claude_code_host_marker_present(Some("1".to_string())));
        assert!(!claude_code_host_marker_present(Some("true".to_string())));
        assert!(!claude_code_host_marker_present(Some("0".to_string())));
        assert!(!claude_code_host_marker_present(None));
    }

    #[test]
    fn external_session_id_resolution_treats_whitespace_as_unset() {
        assert_eq!(
            resolve_external_session_id(Some("   ".to_string()), Some("env".to_string())),
            None
        );
        assert_eq!(
            resolve_external_session_id(None, Some("   ".to_string())),
            None
        );
    }

    #[test]
    fn orient_parses_external_session_id_flag() {
        let cli = Cli::try_parse_from([
            "engram",
            "orient",
            "--external-session-id",
            "codex://threads/orient-cli",
        ])
        .expect("orient command should parse");

        match cli.command {
            Commands::Orient {
                external_session_id,
                ..
            } => assert_eq!(
                external_session_id.as_deref(),
                Some("codex://threads/orient-cli")
            ),
            _ => panic!("expected orient command"),
        }
    }

    #[test]
    fn memory_changes_since_parses_external_session_id_flag() {
        let cli = Cli::try_parse_from([
            "engram",
            "memory",
            "changes-since",
            "--timestamp",
            "2026-06-04T05:00:00Z",
            "--external-session-id",
            "codex://threads/changes-cli",
        ])
        .expect("memory changes-since command should parse");

        match cli.command {
            Commands::Memory { command, .. } => match command {
                MemoryCommands::ChangesSince {
                    external_session_id,
                    ..
                } => assert_eq!(
                    external_session_id.as_deref(),
                    Some("codex://threads/changes-cli")
                ),
                _ => panic!("expected changes-since command"),
            },
            _ => panic!("expected memory command"),
        }
    }

    #[test]
    fn serve_stdio_rejects_http_only_storage_flags() {
        let err = validate_serve_options(true, None, None, None, false, None).unwrap_err();
        assert!(err
            .to_string()
            .contains("--memory is only honored with --http"));

        let err = validate_serve_options(
            false,
            Some("ws://localhost:8000"),
            Some("root"),
            Some("root"),
            false,
            None,
        )
        .unwrap_err();
        assert!(err
            .to_string()
            .contains("--remote/--username/--password are only honored with --http"));

        let err = validate_serve_options(false, None, None, None, false, Some(8766)).unwrap_err();
        assert!(err
            .to_string()
            .contains("--port is only honored with --http"));
    }

    #[test]
    fn serve_http_accepts_storage_flags() {
        validate_serve_options(true, None, None, None, true, Some(8766))
            .expect("--http --memory should be valid");
        validate_serve_options(
            false,
            Some("ws://localhost:8000"),
            Some("root"),
            Some("root"),
            true,
            Some(8766),
        )
        .expect("--http --remote with credentials should be valid");
    }

    #[test]
    fn serve_rejects_conflicting_or_dangling_storage_credentials() {
        let err = validate_serve_options(
            true,
            Some("ws://localhost:8000"),
            Some("root"),
            Some("root"),
            true,
            None,
        )
        .unwrap_err();
        assert!(err.to_string().contains("--memory and --remote"));

        let err = validate_serve_options(false, None, Some("root"), None, true, None).unwrap_err();
        assert!(err
            .to_string()
            .contains("--username and --password require --remote"));
    }
}
