//! Store configuration.

use std::path::PathBuf;

/// Optional state-root override for isolated hosts, tests, and portable installations.
pub const ENGRAM_HOME_ENV: &str = "ENGRAM_HOME";

/// Configuration for the engram store.
#[derive(Debug, Clone)]
pub struct StoreConfig {
    /// Storage backend (mem, rocksdb, etc.)
    pub backend: StorageBackend,

    /// Namespace to use.
    pub namespace: String,

    /// Database name to use.
    pub database: String,
}

/// Storage backend options.
#[derive(Debug, Clone)]
pub enum StorageBackend {
    /// In-memory storage (for testing).
    Memory,

    /// RocksDB storage (for persistence).
    RocksDb(PathBuf),

    /// Remote SurrealDB server (for concurrent access).
    Remote {
        /// Server URL (e.g., "ws://localhost:8000")
        url: String,
        /// Username for authentication
        username: String,
        /// Password for authentication
        password: String,
    },
}

impl Default for StoreConfig {
    fn default() -> Self {
        Self {
            backend: StorageBackend::Memory,
            namespace: "engram".to_string(),
            database: "main".to_string(),
        }
    }
}

impl StoreConfig {
    /// Create a config for in-memory storage.
    #[must_use]
    pub fn memory() -> Self {
        Self::default()
    }

    /// Create a config for RocksDB storage.
    #[must_use]
    pub fn rocksdb(path: impl Into<PathBuf>) -> Self {
        Self {
            backend: StorageBackend::RocksDb(path.into()),
            ..Default::default()
        }
    }

    /// Create a config for remote SurrealDB server.
    #[must_use]
    pub fn remote(url: String, username: String, password: String) -> Self {
        Self {
            backend: StorageBackend::Remote {
                url,
                username,
                password,
            },
            ..Default::default()
        }
    }

    /// Get the connection string for SurrealDB.
    #[must_use]
    pub fn connection_string(&self) -> String {
        match &self.backend {
            StorageBackend::Memory => "mem://".to_string(),
            StorageBackend::RocksDb(path) => format!("rocksdb://{}", path.display()),
            StorageBackend::Remote { url, .. } => url.clone(),
        }
    }

    /// Get the default data directory.
    #[must_use]
    pub fn default_data_dir() -> PathBuf {
        Self::state_dir().join("data")
    }

    /// Get Engram's state root, honoring `ENGRAM_HOME` when explicitly set.
    #[must_use]
    pub fn state_dir() -> PathBuf {
        state_dir_from(std::env::var_os(ENGRAM_HOME_ENV), dirs::home_dir())
    }

    /// Get the data directory for an isolated named project.
    #[must_use]
    pub fn project_data_dir(project: &str) -> PathBuf {
        Self::state_dir()
            .join("projects")
            .join(project)
            .join("data")
    }
}

fn state_dir_from(override_dir: Option<std::ffi::OsString>, user_home: Option<PathBuf>) -> PathBuf {
    override_dir
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            user_home
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".engram")
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_config() {
        let config = StoreConfig::memory();
        assert_eq!(config.connection_string(), "mem://");
    }

    #[test]
    fn test_rocksdb_config() {
        let config = StoreConfig::rocksdb("/tmp/engram-test");
        assert_eq!(config.connection_string(), "rocksdb:///tmp/engram-test");
    }

    #[test]
    fn state_dir_prefers_nonempty_override_without_touching_process_environment() {
        assert_eq!(
            state_dir_from(
                Some(std::ffi::OsString::from("/tmp/engram-isolated")),
                Some(PathBuf::from("/Users/example")),
            ),
            PathBuf::from("/tmp/engram-isolated")
        );
        assert_eq!(
            state_dir_from(
                Some(std::ffi::OsString::new()),
                Some(PathBuf::from("/Users/example"))
            ),
            PathBuf::from("/Users/example/.engram")
        );
    }
}
