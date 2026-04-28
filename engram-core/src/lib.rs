//! # engram-core
//!
//! Core domain types and invariants for engram.
//!
//! This crate contains the fundamental data structures and business logic
//! for the engram knowledge system. It has **zero dependencies** on external
//! infrastructure (no database, no MCP, no embeddings) — making it easy to
//! test and ensuring the domain logic remains portable.
//!
//! ## Modules
//!
//! - [`entity`] - Entity knowledge (Layer 1): repos, tools, terminology
//! - [`session`] - Session history (Layer 2): decisions, events, rationale
//! - [`document`] - Document knowledge (Layer 3): chunks, sources
//! - [`tool`] - Tool intelligence (Layer 4): usage, recommendations
//! - [`coordination`] - Session coordination (Layer 5): parallel awareness
//! - [`knowledge`] - Document intelligence (Layer 6): canonical resolver
//! - [`work`] - Work management (Layer 7): projects, tasks, PRs
//! - [`memory`] - Memory OS ontology: provenance, scope, lifecycle, commits
//! - [`repository`] - Git repository topology and local checkout mapping
//! - [`harness`] - Agent harness policy and adapter contracts
//! - [`lint`] - Memory health and migration-safety linting
//! - [`graph`] - Memory OS graph traversal types
//! - [`id`] - ID types and generation
//! - [`error`] - Error types

pub mod coordination;
pub mod document;
pub mod entity;
pub mod error;
pub mod graph;
pub mod harness;
pub mod id;
pub mod knowledge;
pub mod lint;
pub mod memory;
pub mod repository;
pub mod search;
pub mod session;
pub mod tool;
pub mod work;

// Re-export commonly used types
pub use error::{Error, Result};
pub use id::Id;
