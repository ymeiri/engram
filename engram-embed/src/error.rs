//! Embedding error types.

use thiserror::Error;

/// Embedding error type.
#[derive(Debug, Error)]
pub enum EmbedError {
    /// Model loading error.
    #[error("failed to load model: {0}")]
    ModelLoad(String),

    /// Embedding generation error.
    #[error("embedding error: {0}")]
    Embedding(String),

    /// Invalid input.
    #[error("invalid input: {0}")]
    InvalidInput(String),
}

/// Embedding result type.
pub type EmbedResult<T> = Result<T, EmbedError>;
