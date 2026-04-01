//! Error types for engram-core.
//!
//! This module defines the error types used throughout the engram system.
//! We use `thiserror` for ergonomic error definition.

use thiserror::Error;

/// The main error type for engram operations.
#[derive(Debug, Error)]
pub enum Error {
    /// Entity not found.
    #[error("entity not found: {0}")]
    EntityNotFound(String),

    /// Document not found.
    #[error("document not found: {0}")]
    DocumentNotFound(String),

    /// Session not found.
    #[error("session not found: {0}")]
    SessionNotFound(String),

    /// Invalid input provided.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// Validation error.
    #[error("validation error: {0}")]
    Validation(String),

    /// Duplicate entry.
    #[error("duplicate entry: {0}")]
    Duplicate(String),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// ID parsing error.
    #[error("invalid id: {0}")]
    InvalidId(#[from] uuid::Error),
}

/// A specialized Result type for engram operations.
pub type Result<T> = std::result::Result<T, Error>;
