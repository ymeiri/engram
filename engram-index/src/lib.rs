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
pub mod entity;
pub mod error;
pub mod knowledge;
pub mod parser;
pub mod pipeline;
pub mod search;
pub mod service;
pub mod session;
pub mod tool_intel;
pub mod version;
pub mod work;

pub use chunker::ChunkerConfig;
pub use coordination::{ConflictCheckResult, CoordinationService};
pub use entity::EntityService;
pub use error::{IndexError, IndexResult};
pub use knowledge::{
    DuplicateGroup, KnowledgeConfig, KnowledgeService, KnowledgeStats, ScanResult, VersionChain,
    VersionedFile,
};
pub use parser::{parse_content, parse_file, ParsedDocument, Section};
pub use pipeline::{IndexedChunk, IndexedDocument, Pipeline, PipelineConfig};
pub use search::{SearchService, SearchStats};
pub use service::{DocumentService, DocumentStats};
pub use session::SessionService;
pub use tool_intel::{ToolIntelService, ToolUsageInfo};
pub use version::{VersionDetector, VersionInfo, VersionSource};
pub use work::{FullWorkContext, GraduateFrom, WorkService};
