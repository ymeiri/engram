//! Store error types.

use crate::config::{StorageBackend, StoreConfig};
use std::path::Path;
use thiserror::Error;

/// Store error type.
#[derive(Debug, Error)]
pub enum StoreError {
    /// Database error.
    #[error("database error: {0}")]
    Database(#[source] Box<surrealdb::Error>),

    /// Database error with user-facing recovery guidance.
    #[error("{message}")]
    DatabaseWithHint {
        /// User-facing error and recovery hint.
        message: String,
        /// Original database error.
        #[source]
        source: Box<surrealdb::Error>,
    },

    /// Entity not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Deserialization error (for custom parsing).
    #[error("deserialization error: {0}")]
    Deserialization(String),

    /// Core domain error.
    #[error("domain error: {0}")]
    Domain(#[from] engram_core::Error),
}

impl From<surrealdb::Error> for StoreError {
    fn from(value: surrealdb::Error) -> Self {
        Self::database(value)
    }
}

impl StoreError {
    /// Create a database error without storage-backend context.
    #[must_use]
    pub fn database(value: surrealdb::Error) -> Self {
        Self::Database(Box::new(value))
    }

    /// Create a database error with backend-specific recovery guidance when possible.
    #[must_use]
    pub fn database_with_config(value: surrealdb::Error, config: &StoreConfig) -> Self {
        if let StorageBackend::RocksDb(path) = &config.backend {
            if let Some(message) = rocksdb_lock_conflict_message(&value, path) {
                return Self::DatabaseWithHint {
                    message,
                    source: Box::new(value),
                };
            }
        }

        Self::database(value)
    }
}

fn rocksdb_lock_conflict_message(error: &surrealdb::Error, data_dir: &Path) -> Option<String> {
    let original = error.to_string();
    if !looks_like_rocksdb_lock_conflict(&original) {
        return None;
    }

    let lock_path = data_dir.join("LOCK");
    Some(format!(
        "database lock conflict opening RocksDB store at {}\n\n\
         Engram could not acquire the RocksDB lock. This usually means another Engram daemon or \
         CLI process is using the store, or a previous crash left a stale lock file.\n\n\
         What to do:\n\
           1. Run `engram daemon status` to check for a running daemon.\n\
           2. If a daemon is running, use it through `engram serve` or stop it with \
         `engram daemon stop` before running direct database commands.\n\
           3. If no Engram process is using this store and a crash left a stale lock, remove \
         `{}` and retry.\n\
           4. Check `engram daemon logs` for the original startup error.\n\n\
         Original database error: {}",
        data_dir.display(),
        lock_path.display(),
        original
    ))
}

fn looks_like_rocksdb_lock_conflict(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("lock")
        && (lower.contains("rocksdb")
            || lower.contains("resource temporarily unavailable")
            || lower.contains("temporarily unavailable")
            || lower.contains("io error")
            || lower.contains("already held")
            || lower.contains("database"))
}

/// Store result type.
pub type StoreResult<T> = Result<T, StoreError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rocksdb_lock_detector_matches_common_lock_errors() {
        assert!(looks_like_rocksdb_lock_conflict(
            "IO error: lock /Users/me/.engram/data/LOCK: Resource temporarily unavailable"
        ));
        assert!(looks_like_rocksdb_lock_conflict(
            "rocksdb lock already held by another process"
        ));
    }

    #[test]
    fn rocksdb_lock_detector_ignores_unrelated_errors() {
        assert!(!looks_like_rocksdb_lock_conflict(
            "network connection failed"
        ));
        assert!(!looks_like_rocksdb_lock_conflict(
            "database schema failed to parse"
        ));
    }
}
