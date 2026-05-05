//! # engram-index
//!
//! Document indexing, chunking, and ingestion for engram.
//!
//! This crate handles the document processing pipeline:
//! 1. Parse documents (Markdown, etc.)
//! 2. Chunk by section headings
//! 3. Generate embeddings
//! 4. Store in database
//!
//! ## Features
//!
//! - Markdown parsing with heading hierarchy
//! - Section-based chunking
//! - Duplicate detection
//! - Version detection
//! - Incremental indexing
//! - Knowledge document management (Layer 6)
//!
//! ## Example
//!
//! ```ignore
//! use engram_index::pipeline::Pipeline;
//!
//! let pipeline = Pipeline::with_defaults()?;
//! let docs = pipeline.index_directory("./docs")?;
//! ```

pub mod chunker;
pub mod coordination;
pub mod digest;
pub mod entity;
pub mod error;
pub mod graph;
pub mod handoff;
pub mod harness;
pub mod knowledge;
pub mod lint;
pub mod memory;
pub mod migration;
pub mod obligation;
pub mod parser;
pub mod pipeline;
pub mod repository;
pub mod search;
pub mod service;
pub mod session;
pub mod telemetry;
pub mod tool_intel;
pub mod vault;
pub mod version;
pub mod work;

pub use chunker::{ChunkerConfig, ChunkingStrategy};
pub use coordination::{ConflictCheckResult, CoordinationService};
pub use digest::{
    DigestExcludedPath, DigestExtractionCandidateSummary, DigestExtractionOptions,
    DigestExtractionPlan, DigestExtractionReviewApply, DigestExtractionReviewApplyOptions,
    DigestFileFormat, DigestInventory, DigestInventoryOptions, DigestProposedAction,
    DigestReviewApply, DigestReviewDecision, DigestReviewExport, DigestReviewedSource,
    DigestSensitivity, DigestService, DigestSourceCandidate, DigestSourceIndexDocument,
    DigestSourceIndexOptions, DigestSourceIndexPlan, DigestSourceKind,
};
pub use entity::EntityService;
pub use error::{IndexError, IndexResult};
pub use graph::GraphService;
pub use handoff::{HandoffCompile, HandoffGet, HandoffService, HandoffUpdate};
pub use harness::HarnessService;
pub use knowledge::{
    DuplicateGroup, KnowledgeConfig, KnowledgeService, KnowledgeStats, ScanResult, VersionChain,
    VersionedFile,
};
pub use lint::{LintOptions, LintService};
pub use memory::{
    MemoryChangeRelevance, MemoryChanges, MemoryChangesSinceOptions, MemoryService,
    MemoryWriterStat, OrientInput, OrientationPacket, OrientationResolution, SessionDistillation,
};
pub use migration::{
    MigrationCandidate, MigrationDisposition, MigrationInventory, MigrationInventoryOptions,
    MigrationReviewApply, MigrationReviewApplyOptions, MigrationReviewExport,
    MigrationReviewStatus, MigrationService, MigrationSourceKind,
};
pub use obligation::{
    obligation_writer, ObligationDetectOptions, ObligationDetection, ObligationDoctorReport,
    ObligationService,
};
pub use parser::{parse_content, parse_file, ParsedDocument, Section};
pub use pipeline::{
    DocumentIngestionPlan, IndexedChunk, IndexedDocument, Pipeline, PipelineConfig, PlannedDocument,
};
pub use repository::{
    RepositoryDetection, RepositoryMigrationCandidate, RepositoryMigrationDisposition,
    RepositoryMigrationEvidence, RepositoryMigrationInventory, RepositoryMigrationOptions,
    RepositoryMigrationReviewApply, RepositoryMigrationReviewApplyOptions,
    RepositoryMigrationReviewExport, RepositoryMigrationReviewStatus,
    RepositoryMigrationSourceKind, RepositoryReferenceKind, RepositoryService,
};
pub use search::{SearchOptions, SearchService, SearchStats};
pub use service::{
    DocumentOrphanCleanupAction, DocumentOrphanCleanupExecutionAction,
    DocumentOrphanCleanupExecutionOptions, DocumentOrphanCleanupExecutionReport,
    DocumentOrphanCleanupExecutionStatus, DocumentOrphanCleanupGroupPlan,
    DocumentOrphanCleanupPlan, DocumentOrphanCleanupPlanOptions,
    DocumentOrphanQuarantineMemoryReviewPlan, DocumentOrphanQuarantineReviewApply,
    DocumentOrphanQuarantineReviewApplyOptions, DocumentOrphanQuarantineReviewDecision,
    DocumentOrphanQuarantineReviewExport, DocumentOrphanQuarantineReviewFileState,
    DocumentOrphanQuarantineReviewFileStatus, DocumentOrphanQuarantineReviewOptions,
    DocumentOrphanQuarantineReviewPrioritization,
    DocumentOrphanQuarantineReviewPrioritizationOptions, DocumentOrphanQuarantineReviewPriority,
    DocumentOrphanQuarantineReviewPriorityItem, DocumentOrphanQuarantineReviewStatus,
    DocumentOrphanQuarantineReviewSuggestedStep, DocumentRecoveryOptions, DocumentReindexAction,
    DocumentReindexExecutionAction, DocumentReindexExecutionOptions,
    DocumentReindexExecutionReport, DocumentReindexExecutionStatus, DocumentReindexGroupRef,
    DocumentReindexPlan, DocumentReindexReviewOnlyGroup, DocumentReindexSourcePlan,
    DocumentService, DocumentStats,
};
pub use session::SessionService;
pub use telemetry::TelemetryService;
pub use tool_intel::{ToolIntelService, ToolUsageInfo};
pub use vault::{
    MemoryVaultExport, MemoryVaultInit, MemoryVaultPage, MemoryVaultStatus, RepositoryVaultSnapshot,
};
pub use version::{VersionDetector, VersionInfo, VersionSource};
pub use work::{FullWorkContext, GraduateFrom, WorkService};

pub use engram_store::{
    DocumentDetectedReference, DocumentOrphanChunkSample, DocumentOrphanGroup,
    DocumentOrphanReport, DocumentRecoveryCandidateMatch, DocumentRecoveryClass,
    DocumentRecoverySummary,
};
