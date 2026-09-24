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
//! ```no_run
//! use engram_store::{connect, StoreConfig, repos::DocumentRepo};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = StoreConfig::default();
//! let db = connect(&config).await?;
//!
//! let doc_repo = DocumentRepo::new(db.clone());
//! doc_repo.init_schema().await?;
//!
//! // Now ready to store and search documents
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod error;
pub mod repos;
mod secret;

pub use config::{StorageBackend, StoreConfig, ENGRAM_HOME_ENV};
pub use error::{StoreError, StoreResult};
pub use repos::{
    AliasSearchResult, ArchivedObservation, CoordinationRepo, CoordinationStats,
    DocumentDeletedOrphanSource, DocumentDetectedReference, DocumentOrphanChunkSample,
    DocumentOrphanDeleteResult, DocumentOrphanGroup, DocumentOrphanReport,
    DocumentRecoveryCandidateMatch, DocumentRecoveryClass, DocumentRecoverySummary, DocumentRepo,
    EntityRepo, EntitySearchResult, EntityStats, KnowledgeRepo, MemoryReferencePurge, MemoryRepo,
    ObligationRepo, ObservationSearchResult, ProjectObservationSearchResult, RepositoryRepo,
    SessionRepo, TaskObservationSearchResult, TelemetryMemoryPurge, TelemetryRepo, ToolIntelStats,
    ToolRepo, WorkRepo, WorkStats,
};

use std::path::Path;
use surrealdb::engine::any::Any;
use surrealdb::Surreal;
use tracing::info;

/// Absolute filesystem reserve required before Engram opens a persistent store.
pub const MIN_FREE_DISK_BYTES: u64 = 512 * 1024 * 1024;
/// Filesystem fraction reserved before Engram opens or advertises a persistent store as ready.
pub const MIN_FREE_DISK_FRACTION_DENOMINATOR: u64 = 50;

/// Measured filesystem headroom for a persistent store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiskHeadroom {
    pub available_bytes: u64,
    pub total_bytes: u64,
    pub required_bytes: u64,
}

impl DiskHeadroom {
    #[must_use]
    pub fn is_sufficient(self) -> bool {
        self.available_bytes >= self.required_bytes
    }
}

/// The main database connection.
pub type Db = Surreal<Any>;

/// Initialize the database connection.
///
/// # Errors
///
/// Returns an error if the connection cannot be established.
pub async fn connect(config: &StoreConfig) -> StoreResult<Db> {
    if let StorageBackend::RocksDb(path) = &config.backend {
        ensure_private_directory(path)?;
        ensure_disk_headroom(path)?;
    }
    info!("Connecting to SurrealDB: {}", config.connection_string());
    let db: Db = Surreal::init();
    db.connect(config.connection_string())
        .await
        .map_err(|err| StoreError::database_with_config(err, config))?;

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

/// Measure filesystem headroom using Engram's persistent-store reserve policy.
pub fn disk_headroom(path: &Path) -> StoreResult<DiskHeadroom> {
    let available_bytes = fs2::available_space(path)?;
    let total_bytes = fs2::total_space(path)?;
    Ok(disk_headroom_from_bytes(available_bytes, total_bytes))
}

/// Reject a persistent-store operation before it can exhaust the filesystem.
pub fn ensure_disk_headroom(path: &Path) -> StoreResult<DiskHeadroom> {
    let headroom = disk_headroom(path)?;
    if !headroom.is_sufficient() {
        return Err(StoreError::InsufficientDiskSpace {
            path: path.display().to_string(),
            available_bytes: headroom.available_bytes,
            required_bytes: headroom.required_bytes,
        });
    }
    Ok(headroom)
}

fn disk_headroom_from_bytes(available_bytes: u64, total_bytes: u64) -> DiskHeadroom {
    let fractional_reserve = total_bytes / MIN_FREE_DISK_FRACTION_DENOMINATOR;
    DiskHeadroom {
        available_bytes,
        total_bytes,
        required_bytes: MIN_FREE_DISK_BYTES.max(fractional_reserve),
    }
}

/// Create a directory when needed and restrict it to the owning user on Unix.
pub fn ensure_private_directory(path: &Path) -> StoreResult<()> {
    std::fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

/// Restrict an existing file to the owning user on Unix.
pub fn ensure_private_file(path: &Path) -> StoreResult<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
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

    // Initialize Memory OS schema
    let memory_repo = MemoryRepo::new(db.clone());
    memory_repo.init_schema().await?;

    // Initialize brain harness telemetry schema
    let telemetry_repo = TelemetryRepo::new(db.clone());
    telemetry_repo.init_schema().await?;

    // Initialize repository topology schema
    let repository_repo = RepositoryRepo::new(db.clone());
    repository_repo.init_schema().await?;

    // Initialize agent obligation schema
    let obligation_repo = ObligationRepo::new(db.clone());
    obligation_repo.init_schema().await?;

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

/// Verify that the datastore can commit a reversible write transaction.
///
/// The probe creates and deletes a unique canary record in one transaction, so a successful probe
/// leaves no row behind. This detects storage-engine failures that still permit read queries.
pub async fn probe_writable(db: &Db) -> StoreResult<()> {
    let probe_id = engram_core::id::Id::new().to_string();
    db.query(
        r#"
        BEGIN TRANSACTION;
        CREATE type::thing("engram_storage_probe", $id) SET checked_at = time::now();
        DELETE type::thing("engram_storage_probe", $id);
        COMMIT TRANSACTION;
        "#,
    )
    .bind(("id", probe_id))
    .await?
    .check()?;
    Ok(())
}

#[cfg(test)]
mod query_response_contract_tests {
    use std::fs;
    use std::path::Path;

    #[test]
    fn every_repository_query_checks_its_response() {
        let repos_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/repos");

        for entry in fs::read_dir(repos_dir).expect("repository source directory must exist") {
            let path = entry
                .expect("repository source entry must be readable")
                .path();
            if path.extension().and_then(|value| value.to_str()) != Some("rs") {
                continue;
            }

            let source = fs::read_to_string(&path).expect("repository source must be readable");
            let mut search_from = 0;
            while let Some(relative_query) = source[search_from..].find(".query(") {
                let query_offset = search_from + relative_query;
                let after_query = query_offset + ".query(".len();
                let next_query = source[after_query..]
                    .find(".query(")
                    .map_or(source.len(), |offset| after_query + offset);
                let segment = &source[after_query..next_query];
                let await_offset = segment
                    .find(".await")
                    .unwrap_or_else(|| panic!("{} has a query without an await", path.display()));
                let normalized: String = segment[await_offset + ".await".len()..]
                    .chars()
                    .filter(|character| !character.is_whitespace())
                    .take(80)
                    .collect();
                let checked = normalized.starts_with("?.check()?")
                    || normalized.starts_with(".unwrap().check()");

                assert!(
                    checked,
                    "{}:{} executes a SurrealDB query without checking its response",
                    path.display(),
                    source[..query_offset].lines().count()
                );
                search_from = after_query;
            }
        }
    }
}

#[cfg(test)]
mod storage_readiness_tests {
    use super::*;

    #[tokio::test]
    async fn writable_probe_commits_without_leaving_a_record() {
        let db = connect(&StoreConfig::memory()).await.unwrap();

        probe_writable(&db).await.unwrap();

        let mut response = db
            .query("SELECT * FROM engram_storage_probe")
            .await
            .unwrap()
            .check()
            .unwrap();
        let records: Vec<serde_json::Value> = response.take(0).unwrap();
        assert!(records.is_empty());
    }

    #[tokio::test]
    async fn writable_probe_surfaces_statement_level_failures() {
        let db = connect(&StoreConfig::memory()).await.unwrap();
        db.query(
            "DEFINE FIELD checked_at ON TABLE engram_storage_probe TYPE datetime ASSERT false",
        )
        .await
        .unwrap()
        .check()
        .unwrap();

        let error = probe_writable(&db)
            .await
            .expect_err("a failed canary transaction must make readiness fail");

        assert!(matches!(error, StoreError::Database(_)));
    }

    #[test]
    fn disk_headroom_uses_the_larger_absolute_or_fractional_reserve() {
        let small_filesystem = disk_headroom_from_bytes(600 * 1024 * 1024, 10 * 1024 * 1024 * 1024);
        assert_eq!(small_filesystem.required_bytes, MIN_FREE_DISK_BYTES);
        assert!(small_filesystem.is_sufficient());

        let large_filesystem = disk_headroom_from_bytes(10 * 1024 * 1024 * 1024, 1024_u64.pow(4));
        assert_eq!(large_filesystem.required_bytes, 1024_u64.pow(4) / 50);
        assert!(!large_filesystem.is_sufficient());
    }
}

#[cfg(all(test, unix))]
mod privacy_tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn private_directory_and_file_remove_group_and_other_access() {
        let root = tempfile::tempdir().expect("temp dir");
        let directory = root.path().join("data");
        ensure_private_directory(&directory).expect("private directory");
        let file = directory.join("runtime.json");
        std::fs::write(&file, "secret").expect("write file");
        ensure_private_file(&file).expect("private file");

        assert_eq!(
            std::fs::metadata(&directory).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            std::fs::metadata(&file).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}
