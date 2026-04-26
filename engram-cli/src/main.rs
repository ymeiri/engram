//! # engram CLI
//!
//! Command-line interface for managing the engram knowledge system.

mod daemon;
mod proxy;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use engram_core::entity::{EntityType, RelationType};
use engram_core::knowledge::DocType;
use engram_core::memory::{
    Harness, MemoryCursor, MemoryItem, MemoryScope, MemoryStatus, ModelIdentity, WriterProvenance,
};
use engram_core::repository::{ProjectRepositoryRole, RepositoryContext};
use engram_core::session::{EventType, SessionStatus};
use engram_core::tool::ToolOutcome;
use engram_core::Id;
use engram_index::{
    CoordinationService, DocumentService, EntityService, KnowledgeService, MemoryService,
    MigrationInventory, MigrationInventoryOptions, MigrationReviewApply,
    MigrationReviewApplyOptions, MigrationReviewExport, RepositoryMigrationInventory,
    RepositoryMigrationOptions, RepositoryMigrationReviewApply,
    RepositoryMigrationReviewApplyOptions, RepositoryMigrationReviewExport, RepositoryService,
    SearchService, SessionService, ToolIntelService, WorkService,
};
use engram_mcp::EngramServer;
use engram_store::{connect_and_init, StoreConfig};
use time::OffsetDateTime;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

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
        /// Use in-memory storage (for testing)
        #[arg(long)]
        memory: bool,

        /// Connect to remote SurrealDB server (e.g., ws://localhost:8000)
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

        /// Port to listen on (default: auto-select from 8765-8774)
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

    /// Print a Memory OS cursor for later changes_since calls
    Cursor {
        /// Print cursor as JSON
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

fn cwd_or_current(cwd: Option<String>) -> Result<std::path::PathBuf> {
    cwd.map(std::path::PathBuf::from)
        .map(Ok)
        .unwrap_or_else(std::env::current_dir)
        .map_err(Into::into)
}

fn parse_optional_repo_id(repo_id: Option<&str>) -> Result<Option<Id>> {
    repo_id
        .map(Id::parse)
        .transpose()
        .map_err(|e| anyhow::anyhow!("Invalid repository ID: {}", e))
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
    println!("  Timestamp: {}", cursor.timestamp);
    if let Some(commit_id) = cursor.commit_id {
        println!("  Latest commit: {}", commit_id);
    } else {
        println!("  Latest commit: none");
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

                // Create repository topology service
                let repository_service = RepositoryService::new(db.clone());
                repository_service.init_schema().await?;

                // Create unified search service
                let search_service = SearchService::with_defaults(db)?;

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
                server.init_repository(repository_service).await;
                server.init_search(search_service).await;

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

        Commands::Index { path, recursive: _ } => {
            println!("Indexing: {}", path);

            // Connect to database using RocksDB for persistence
            let config = StoreConfig::rocksdb(StoreConfig::default_data_dir());
            let db = connect_and_init(&config).await?;

            // Create document service
            let service = DocumentService::with_defaults(db)?;
            service.init_schema().await?;

            // Index the path
            let path = std::path::Path::new(&path);
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
            // Connect to database
            let config = StoreConfig::default();
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
            // Connect to database
            let config = StoreConfig::default();
            let db = connect_and_init(&config).await?;

            // Create document service
            let service = DocumentService::with_defaults(db)?;

            // Get stats
            let stats = service.stats().await?;

            println!("Database statistics:");
            println!("  Document sources: {}", stats.source_count);
            println!("  Document chunks:  {}", stats.chunk_count);
            println!("  Embedding dim:    {}", stats.embedding_dimension);
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
                MemoryCommands::ExportVault { path } => {
                    let export = service.export_vault(std::path::Path::new(&path)).await?;

                    println!("✓ Memory vault exported");
                    println!("  Root:                 {}", export.root);
                    println!("  Files written:        {}", export.file_count());
                    println!("  Files skipped:        {}", export.files_skipped.len());
                    println!("  Memory items:         {}", export.memory_item_count);
                    println!("  Knowledge commits:    {}", export.knowledge_commit_count);
                    println!("  Repositories:         {}", export.repository_count);
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
                            },
                        )
                        .await?;

                    if json {
                        println!("{}", serde_json::to_string_pretty(&export)?);
                    } else {
                        print_migration_review_export(&export);
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
