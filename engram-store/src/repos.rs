//! Repository implementations.
//!
//! This module contains repository implementations for each domain type,
//! providing the persistence layer for engram.

pub mod coordination;
pub mod document;
pub mod entity;
pub mod knowledge;
pub mod memory;
pub mod obligation;
pub mod repository;
pub mod session;
pub mod telemetry;
pub mod tool;
pub mod work;

pub use coordination::{CoordinationRepo, CoordinationStats};
pub use document::{
    DocumentDeletedOrphanSource, DocumentDetectedReference, DocumentOrphanChunkSample,
    DocumentOrphanDeleteResult, DocumentOrphanGroup, DocumentOrphanReport,
    DocumentRecoveryCandidateMatch, DocumentRecoveryClass, DocumentRecoverySummary, DocumentRepo,
};
pub use entity::{
    AliasSearchResult, ArchivedObservation, EntityRepo, EntitySearchResult, EntityStats,
    ObservationSearchResult,
};
pub use knowledge::KnowledgeRepo;
pub use memory::MemoryRepo;
pub use obligation::ObligationRepo;
pub use repository::RepositoryRepo;
pub use session::SessionRepo;
pub use telemetry::TelemetryRepo;
pub use tool::{ToolIntelStats, ToolRepo};
pub use work::{ProjectObservationSearchResult, TaskObservationSearchResult, WorkRepo, WorkStats};
