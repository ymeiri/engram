//! # engram-store
//!
//! SurrealDB storage adapter for engram.
//!
//! This crate provides the persistence layer for all engram data,
//! using SurrealDB's multi-model capabilities (relational + graph + vector).
//!
//! ## Features
//!
//! - Embedded SurrealDB with RocksDB backend
//! - Repository traits for clean abstraction
//! - Graph relationship queries
//! - Vector similarity search
//! - Schema migrations
//!
//! ## Example
//!
//! ```ignore
//! use engram_store::{connect, StoreConfig, repos::DocumentRepo};
//!
//! let config = StoreConfig::default();
//! let db = connect(&config).await?;
//!
//! let doc_repo = DocumentRepo::new(db.clone());
//! doc_repo.init_schema().await?;
//!
//! // Now ready to store and search documents
//! ```

pub mod config;
pub mod error;
pub mod repos;

pub use config::{StorageBackend, StoreConfig};
pub use error::{StoreError, StoreResult};
pub use repos::{
    AliasSearchResult, ArchivedObservation, CoordinationRepo, CoordinationStats, DocumentRepo,
    EntityRepo, EntitySearchResult, EntityStats, KnowledgeRepo, ObservationSearchResult,
    ProjectObservationSearchResult, SessionRepo, TaskObservationSearchResult, ToolIntelStats,
    ToolRepo, WorkRepo, WorkStats,
};

use surrealdb::engine::any::Any;
use surrealdb::Surreal;
use tracing::info;

/// The main database connection.
pub type Db = Surreal<Any>;

/// Initialize the database connection.
///
/// # Errors
///
/// Returns an error if the connection cannot be established.
pub async fn connect(config: &StoreConfig) -> StoreResult<Db> {
    info!("Connecting to SurrealDB: {}", config.connection_string());
    let db: Db = Surreal::init();
    db.connect(config.connection_string()).await?;

    // Handle authentication for remote connections
    if let StorageBackend::Remote {
        username, password, ..
    } = &config.backend
    {
        use surrealdb::opt::auth::Root;
        db.signin(Root {
            username: username.as_str(),
            password: password.as_str(),
        })
        .await?;
    }

    db.use_ns(&config.namespace)
        .use_db(&config.database)
        .await?;
    info!(
        "Connected to namespace={}, database={}",
        config.namespace, config.database
    );
    Ok(db)
}

/// Initialize all database schemas.
///
/// # Errors
///
/// Returns an error if schema creation fails.
pub async fn init_schema(db: &Db) -> StoreResult<()> {
    info!("Initializing all schemas");

    // Initialize entity schema (Layer 1)
    let entity_repo = EntityRepo::new(db.clone());
    entity_repo.init_schema().await?;

    // Initialize session schema (Layer 2)
    let session_repo = SessionRepo::new(db.clone());
    session_repo.init_schema().await?;

    // Initialize document schema (Layer 3)
    let doc_repo = DocumentRepo::new(db.clone());
    doc_repo.init_schema().await?;

    // Initialize tool schema (Layer 4)
    let tool_repo = ToolRepo::new(db.clone());
    tool_repo.init_schema().await?;

    // Initialize coordination schema (Layer 5)
    let coordination_repo = CoordinationRepo::new(db.clone());
    coordination_repo.init_schema().await?;

    // Initialize knowledge schema (Layer 6)
    let knowledge_repo = KnowledgeRepo::new(db.clone());
    knowledge_repo.init_schema().await?;

    // Initialize work schema (Layer 7)
    let work_repo = WorkRepo::new(db.clone());
    work_repo.init_schema().await?;

    info!("All schemas initialized");
    Ok(())
}

/// Connect and initialize in one step.
///
/// Convenience function that connects to the database and initializes all schemas.
///
/// # Errors
///
/// Returns an error if connection or schema initialization fails.
pub async fn connect_and_init(config: &StoreConfig) -> StoreResult<Db> {
    let db = connect(config).await?;
    init_schema(&db).await?;
    Ok(db)
}
