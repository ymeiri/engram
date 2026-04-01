//! Store configuration.

use std::path::PathBuf;

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
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".engram")
            .join("data")
    }
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
}
