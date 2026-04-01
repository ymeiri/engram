//! Index error types.

use thiserror::Error;

/// Index error type.
#[derive(Debug, Error)]
pub enum IndexError {
    /// File not found.
    #[error("file not found: {0}")]
    FileNotFound(String),

    /// File already exists.
    #[error("file already exists: {0}")]
    FileExists(String),

    /// Entity/resource not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// Entity/resource already exists.
    #[error("already exists: {0}")]
    AlreadyExists(String),

    /// Invalid state for operation.
    #[error("invalid state: {0}")]
    InvalidState(String),

    /// Feature not configured.
    #[error("not configured: {0}")]
    NotConfigured(String),

    /// Parse error.
    #[error("parse error: {0}")]
    Parse(String),

    /// IO error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Store error.
    #[error("store error: {0}")]
    Store(#[from] engram_store::StoreError),

    /// Embed error.
    #[error("embed error: {0}")]
    Embed(#[from] engram_embed::EmbedError),
}

/// Index result type.
pub type IndexResult<T> = Result<T, IndexError>;
