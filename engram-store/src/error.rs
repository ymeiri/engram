//! Store error types.

use thiserror::Error;

/// Store error type.
#[derive(Debug, Error)]
pub enum StoreError {
    /// Database error.
    #[error("database error: {0}")]
    Database(#[from] surrealdb::Error),

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

/// Store result type.
pub type StoreResult<T> = Result<T, StoreError>;
