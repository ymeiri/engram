//! MCP error types.

use thiserror::Error;

/// MCP error type.
#[derive(Debug, Error)]
pub enum McpError {
    /// Tool execution error.
    #[error("tool error: {0}")]
    Tool(String),

    /// Invalid input.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// Store error.
    #[error("store error: {0}")]
    Store(#[from] engram_store::StoreError),

    /// Index error.
    #[error("index error: {0}")]
    Index(#[from] engram_index::IndexError),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// MCP result type.
pub type McpResult<T> = Result<T, McpError>;
