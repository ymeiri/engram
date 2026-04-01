//! # engram-embed
//!
//! Embedding generation for engram using fastembed.
//!
//! This crate provides local embedding generation without external API calls,
//! using the fastembed library with ONNX models.
//!
//! ## Features
//!
//! - Local embedding generation (no API calls)
//! - Configurable embedding models
//! - Batch processing for efficiency
//! - Caching for repeated queries

pub mod config;
pub mod embedder;
pub mod error;

pub use config::EmbedConfig;
pub use embedder::Embedder;
pub use error::{EmbedError, EmbedResult};

/// The dimension of the default embedding model (all-MiniLM-L6-v2).
pub const DEFAULT_DIMENSION: usize = 384;
