//! Repository implementations.
//!
//! This module contains repository implementations for each domain type,
//! providing the persistence layer for engram.

pub mod coordination;
pub mod document;
pub mod entity;
pub mod knowledge;
pub mod session;
pub mod tool;
pub mod work;

pub use coordination::{CoordinationRepo, CoordinationStats};
pub use document::DocumentRepo;
pub use entity::{
    AliasSearchResult, ArchivedObservation, EntityRepo, EntitySearchResult, EntityStats,
    ObservationSearchResult,
};
pub use knowledge::KnowledgeRepo;
pub use session::SessionRepo;
pub use tool::{ToolIntelStats, ToolRepo};
pub use work::{ProjectObservationSearchResult, TaskObservationSearchResult, WorkRepo, WorkStats};
