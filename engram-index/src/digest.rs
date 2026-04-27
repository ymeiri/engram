//! Digest source inventory for Memory OS.
//!
//! Daily email, Slack, AI, SWE, and notes digests can be valuable memory
//! evidence, but they are sensitive and noisy. This module only inventories
//! digest-like source files and classifies operational artifacts. It does not
//! read digest contents or promote facts into active memory.

use crate::error::{IndexError, IndexResult};
use engram_core::id::Id;
use engram_core::memory::{
    ClaimOrigin, EvidenceKind, EvidenceRef, KnowledgeCommit, MemoryChange, MemoryChangeType,
    MemoryItem, MemoryKind, MemoryScope, MemoryStatus, WriterProvenance,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use time::OffsetDateTime;

const DIGEST_REVIEW_MARKER: &str = "<!-- engram:generated:file digest-review-v1 -->";
const DIGEST_EXTRACTION_MARKER: &str = "<!-- engram:generated:file digest-extraction-v1 -->";
const DIGEST_MACHINE_RECORD_HEADING: &str = "## Machine Record";
const DIGEST_MACHINE_RECORD_FENCE: &str = "```json";
const DEFAULT_EXTRACTION_MAX_SOURCE_BYTES: usize = 256 * 1024;
const DEFAULT_EXTRACTION_MAX_CANDIDATES_PER_SOURCE: usize = 8;
const DEFAULT_EXTRACTION_MAX_CANDIDATE_CHARS: usize = 1600;
const DEFAULT_SOURCE_INDEX_MAX_SOURCE_BYTES: usize = 256 * 1024;
const MIN_EXTRACTION_CANDIDATE_CHARS: usize = 40;

/// Stateless service for digest source discovery.
#[derive(Debug, Default, Clone)]
pub struct DigestService;

impl DigestService {
    /// Create a digest service.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Inventory digest-like files under a root path.
    ///
    /// This is intentionally read-light: only filesystem metadata and path names
    /// are inspected. File contents are not read.
    pub fn inventory(&self, options: DigestInventoryOptions) -> IndexResult<DigestInventory> {
        inventory_digest_sources(options)
    }

    /// Export a generated review batch for digest source candidates.
    ///
    /// The export writes metadata-only review pages. It does not read digest file
    /// contents or write Memory OS records.
    pub fn export_review_batch(
        &self,
        output_path: impl AsRef<Path>,
        options: DigestInventoryOptions,
    ) -> IndexResult<DigestReviewExport> {
        export_digest_review_batch(output_path.as_ref(), options)
    }

    /// Apply human review decisions from a generated digest review batch.
    ///
    /// This parses only Engram-generated review pages. It does not read digest
    /// source contents, index source files, or write Memory OS records.
    pub fn apply_review_batch(&self, root: impl AsRef<Path>) -> IndexResult<DigestReviewApply> {
        apply_digest_review_batch(root.as_ref())
    }

    /// Build a review-gated extraction plan from accepted digest sources.
    ///
    /// Only sources explicitly accepted in a generated digest review batch are
    /// read. The plan writes generated candidate-memory review pages and does
    /// not write active Memory OS records.
    pub fn plan_extraction(
        &self,
        review_path: impl AsRef<Path>,
        output_path: impl AsRef<Path>,
        options: DigestExtractionOptions,
    ) -> IndexResult<DigestExtractionPlan> {
        plan_digest_extraction(review_path.as_ref(), output_path.as_ref(), options)
    }

    /// Build a review-gated source evidence indexing plan from source-only digest sources.
    ///
    /// Only sources explicitly marked `source_only` in a generated digest review
    /// batch are read. The returned plan carries source metadata and prepared
    /// document content for an explicit write step, but serialized reports omit
    /// digest contents.
    pub fn plan_source_index(
        &self,
        review_path: impl AsRef<Path>,
        options: DigestSourceIndexOptions,
    ) -> IndexResult<DigestSourceIndexPlan> {
        plan_digest_source_index(review_path.as_ref(), options)
    }
}

/// Options for digest source inventory.
#[derive(Debug, Clone)]
pub struct DigestInventoryOptions {
    /// Root directory to scan.
    pub root_path: PathBuf,
    /// Maximum included candidates to return.
    pub limit: Option<usize>,
    /// Include files that are normally treated as operational artifacts.
    pub include_operational: bool,
}

impl DigestInventoryOptions {
    /// Create inventory options.
    #[must_use]
    pub fn new(root_path: impl Into<PathBuf>) -> Self {
        Self {
            root_path: root_path.into(),
            limit: None,
            include_operational: false,
        }
    }
}

/// Digest source inventory result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestInventory {
    /// Inventory timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub generated_at: OffsetDateTime,
    /// Root path scanned.
    pub root_path: String,
    /// Files encountered under digest-like paths.
    pub files_scanned: usize,
    /// Total candidate digest files found before truncation.
    pub total_candidates: usize,
    /// Returned candidate count after limit.
    pub returned_candidates: usize,
    /// Whether candidate output was truncated by limit.
    pub truncated: bool,
    /// Excluded file count.
    pub excluded_count: usize,
    /// Candidate counts by digest source kind.
    pub by_source_kind: BTreeMap<String, usize>,
    /// Candidate counts by file format.
    pub by_format: BTreeMap<String, usize>,
    /// Candidate counts by collection directory.
    pub by_collection: BTreeMap<String, usize>,
    /// Candidate digest files.
    pub candidates: Vec<DigestSourceCandidate>,
    /// Excluded digest-adjacent files.
    pub exclusions: Vec<DigestExcludedPath>,
    /// Operator warnings.
    pub warnings: Vec<String>,
}

/// Generated digest review batch export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestReviewExport {
    /// Output directory for review files.
    pub output_path: String,
    /// Inventory used to build the batch.
    pub inventory: DigestInventory,
    /// Files created or updated, relative to output path.
    pub files_written: Vec<String>,
    /// Existing files skipped because they were not generated by Engram.
    pub files_skipped: Vec<String>,
}

/// Decision selected by the human reviewer for a digest source candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DigestReviewDecision {
    /// Allow a future extraction step to inspect this source and produce candidate memories.
    Accept,
    /// Keep this source available as evidence/index material, but do not extract memories.
    SourceOnly,
    /// Keep the source quarantined until a later review.
    Quarantine,
    /// Reject this source from future digest ingestion.
    Reject,
}

impl std::fmt::Display for DigestReviewDecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Accept => write!(f, "accept"),
            Self::SourceOnly => write!(f, "source_only"),
            Self::Quarantine => write!(f, "quarantine"),
            Self::Reject => write!(f, "reject"),
        }
    }
}

/// Reviewed digest source that can be handed to a later extraction/indexing step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestReviewedSource {
    /// Review page path relative to the batch root.
    pub review_path: String,
    /// Selected review decision.
    pub decision: DigestReviewDecision,
    /// Candidate source metadata copied from the generated machine record.
    pub candidate: DigestSourceCandidate,
    /// Optional intended memory kind from the review decision block.
    pub memory_kind: Option<String>,
    /// Optional intended scope type from the review decision block.
    pub scope_type: Option<String>,
    /// Optional intended scope name from the review decision block.
    pub scope_name: Option<String>,
    /// Optional reviewer title hint for future extraction.
    pub title: Option<String>,
    /// Optional reviewer notes for future extraction.
    pub notes: Option<String>,
}

/// Result of applying human review decisions from a digest review batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestReviewApply {
    /// Review batch root path.
    pub root: String,
    /// Candidate review files scanned.
    pub files_scanned: usize,
    /// Generated candidate files skipped with a reason.
    pub files_skipped: Vec<String>,
    /// Candidate files whose decision stayed pending or empty.
    pub files_with_no_decision: Vec<String>,
    /// Candidate files whose decision value was not recognized.
    pub files_with_invalid_decision: Vec<String>,
    /// Candidate files whose machine record could not be parsed.
    pub files_with_parse_errors: Vec<String>,
    /// Sources accepted for future review-gated extraction.
    pub accepted_count: usize,
    /// Sources accepted only as indexed/evidence sources.
    pub source_only_count: usize,
    /// Sources explicitly quarantined by review.
    pub quarantined_count: usize,
    /// Sources explicitly rejected by review.
    pub rejected_count: usize,
    /// Sources that would be passed to a future extraction/indexing stage.
    pub planned_sources: Vec<DigestReviewedSource>,
    /// Non-fatal warnings surfaced during parsing/apply.
    pub warnings: Vec<String>,
}

impl DigestReviewApply {
    /// Number of reviewed sources planned for future processing.
    #[must_use]
    pub fn planned_count(&self) -> usize {
        self.planned_sources.len()
    }
}

/// Options for building digest extraction plans from accepted reviewed sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestExtractionOptions {
    /// Maximum bytes allowed per accepted source before it is skipped.
    pub max_source_bytes: usize,
    /// Maximum candidate memory excerpts to generate per accepted source.
    pub max_candidates_per_source: usize,
    /// Maximum characters copied into each generated candidate excerpt.
    pub max_candidate_chars: usize,
}

impl Default for DigestExtractionOptions {
    fn default() -> Self {
        Self {
            max_source_bytes: DEFAULT_EXTRACTION_MAX_SOURCE_BYTES,
            max_candidates_per_source: DEFAULT_EXTRACTION_MAX_CANDIDATES_PER_SOURCE,
            max_candidate_chars: DEFAULT_EXTRACTION_MAX_CANDIDATE_CHARS,
        }
    }
}

/// Generated digest extraction plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestExtractionPlan {
    /// Review batch root path used as input.
    pub review_path: String,
    /// Output directory for generated extraction review files.
    pub output_path: String,
    /// Candidate review files scanned from the source review batch.
    pub review_files_scanned: usize,
    /// Sources accepted for extraction.
    pub accepted_sources: usize,
    /// Sources marked source-only and intentionally not read.
    pub source_only_sources: usize,
    /// Accepted sources whose content was read.
    pub sources_read: usize,
    /// Accepted sources skipped with a reason.
    pub sources_skipped: Vec<String>,
    /// Candidate memory review files created or updated.
    pub files_written: Vec<String>,
    /// Existing output files skipped because they were not generated by Engram.
    pub files_skipped: Vec<String>,
    /// Candidate memory summaries generated for review.
    pub candidates: Vec<DigestExtractionCandidateSummary>,
    /// Non-fatal warnings surfaced during planning.
    pub warnings: Vec<String>,
}

impl DigestExtractionPlan {
    /// Number of generated candidate memory excerpts.
    #[must_use]
    pub fn candidate_count(&self) -> usize {
        self.candidates.len()
    }
}

/// Options for planning source-only digest evidence indexing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestSourceIndexOptions {
    /// Maximum bytes allowed per source-only digest before it is skipped.
    pub max_source_bytes: usize,
}

impl Default for DigestSourceIndexOptions {
    fn default() -> Self {
        Self {
            max_source_bytes: DEFAULT_SOURCE_INDEX_MAX_SOURCE_BYTES,
        }
    }
}

/// Review-gated source-only digest evidence indexing plan.
#[derive(Debug, Clone, Serialize)]
pub struct DigestSourceIndexPlan {
    /// Review batch root path used as input.
    pub review_path: String,
    /// Candidate review files scanned from the source review batch.
    pub review_files_scanned: usize,
    /// Sources accepted for extraction and intentionally not indexed by this plan.
    pub accepted_sources: usize,
    /// Sources marked source-only and eligible for evidence indexing.
    pub source_only_sources: usize,
    /// Source-only sources whose content was read into prepared document content.
    pub sources_read: usize,
    /// Sources skipped with a reason.
    pub sources_skipped: Vec<String>,
    /// Source-only documents prepared for explicit indexing.
    pub documents: Vec<DigestSourceIndexDocument>,
    /// Non-fatal warnings surfaced during planning.
    pub warnings: Vec<String>,
}

impl DigestSourceIndexPlan {
    /// Number of source-only documents prepared for indexing.
    #[must_use]
    pub fn document_count(&self) -> usize {
        self.documents.len()
    }
}

/// Prepared source-only digest document metadata.
#[derive(Debug, Clone, Serialize)]
pub struct DigestSourceIndexDocument {
    /// Source review page path relative to the digest review batch root.
    pub source_review_path: String,
    /// Original digest source path relative to its inventory root.
    pub source_relative_path: String,
    /// Original digest source absolute path.
    pub source_absolute_path: String,
    /// Original digest source kind.
    pub source_kind: DigestSourceKind,
    /// Document title to store in Layer 3.
    pub title: String,
    /// Number of source text characters prepared for indexing.
    pub content_chars: usize,
    /// Document path or URL used as the Layer 3 source key.
    pub document_path: String,
    /// Markdown content prepared for Layer 3 indexing. Serialized reports omit this field.
    #[serde(skip)]
    pub indexed_content: String,
}

/// Metadata summary for a generated digest extraction candidate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestExtractionCandidateSummary {
    /// Generated candidate review page path relative to extraction output root.
    pub review_path: String,
    /// Source review page path relative to the digest review batch root.
    pub source_review_path: String,
    /// Original digest source path relative to its inventory root.
    pub source_relative_path: String,
    /// Original digest source kind.
    pub source_kind: DigestSourceKind,
    /// Generated candidate title.
    pub title: String,
    /// Suggested memory kind copied from source review, when provided.
    pub memory_kind: Option<String>,
    /// Suggested scope type copied from source review, when provided.
    pub scope_type: Option<String>,
    /// Suggested scope name copied from source review, when provided.
    pub scope_name: Option<String>,
    /// Number of characters copied into the generated candidate page.
    pub content_chars: usize,
}

/// Options for applying reviewed digest extraction candidates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestExtractionReviewApplyOptions {
    /// When true, parse and report the batch without writing memory records.
    pub dry_run: bool,
    /// Writer/importer provenance to attach to accepted memory records.
    pub writer: WriterProvenance,
    /// Create a knowledge commit for written records.
    pub create_commit: bool,
}

/// Result of applying, or dry-running, a digest extraction review batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestExtractionReviewApply {
    /// Extraction review batch root path.
    pub root: String,
    /// Whether this run avoided writes.
    pub dry_run: bool,
    /// Candidate memory review files scanned.
    pub files_scanned: usize,
    /// Generated candidate files skipped with a reason.
    pub files_skipped: Vec<String>,
    /// Candidate files whose decision stayed pending or empty.
    pub files_with_no_decision: Vec<String>,
    /// Candidate files whose decision value was not recognized.
    pub files_with_invalid_decision: Vec<String>,
    /// Candidate files whose reviewed memory record could not be parsed safely.
    pub files_with_parse_errors: Vec<String>,
    /// Accepted candidates planned for Memory OS import.
    pub accepted_count: usize,
    /// Candidates explicitly quarantined by review.
    pub quarantined_count: usize,
    /// Candidates explicitly rejected by review.
    pub rejected_count: usize,
    /// Accepted candidates skipped because their candidate review path was already imported.
    pub duplicate_count: usize,
    /// Items that would be written, or were written in non-dry-run mode.
    pub planned_items: Vec<MemoryItem>,
    /// Items written in non-dry-run mode.
    pub written_items: Vec<MemoryItem>,
    /// Knowledge commit created in non-dry-run mode.
    pub commit: Option<KnowledgeCommit>,
    /// Non-fatal warnings surfaced during parsing/apply.
    pub warnings: Vec<String>,
}

impl DigestExtractionReviewApply {
    /// Number of planned accepted memory records.
    #[must_use]
    pub fn planned_count(&self) -> usize {
        self.planned_items.len()
    }

    /// Number of written memory records.
    #[must_use]
    pub fn written_count(&self) -> usize {
        self.written_items.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DigestExtractionCandidateRecord {
    summary: DigestExtractionCandidateSummary,
    source_candidate: DigestSourceCandidate,
}

/// Candidate digest file that can later become review-gated source evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestSourceCandidate {
    /// Source kind inferred from path.
    pub source_kind: DigestSourceKind,
    /// Top-level digest collection directory, such as slack-digest.
    pub collection: String,
    /// Bucket within the collection, such as morning or ai-radar.
    pub bucket: Option<String>,
    /// File format.
    pub format: DigestFileFormat,
    /// Relative path from inventory root.
    pub relative_path: String,
    /// Absolute path.
    pub absolute_path: String,
    /// File name.
    pub file_name: String,
    /// Date or date range inferred from the path, when present.
    pub date_hint: Option<String>,
    /// File size from metadata.
    pub size_bytes: u64,
    /// Last modified timestamp from metadata, when available.
    #[serde(with = "time::serde::rfc3339::option")]
    pub modified_at: Option<OffsetDateTime>,
    /// Sensitivity classification.
    pub sensitivity: DigestSensitivity,
    /// Recommended next action.
    pub proposed_action: DigestProposedAction,
    /// Classification reasons.
    pub reasons: Vec<String>,
}

/// Digest-adjacent file excluded from source candidates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestExcludedPath {
    /// Relative path from inventory root.
    pub relative_path: String,
    /// Absolute path.
    pub absolute_path: String,
    /// Exclusion reason.
    pub reason: String,
}

/// Inferred digest source kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DigestSourceKind {
    /// Slack digest.
    Slack,
    /// Email digest.
    Email,
    /// AI/news digest.
    Ai,
    /// Software engineering digest.
    Swe,
    /// Notes digest.
    Notes,
    /// Unknown digest source.
    Unknown,
}

impl std::fmt::Display for DigestSourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Slack => write!(f, "slack"),
            Self::Email => write!(f, "email"),
            Self::Ai => write!(f, "ai"),
            Self::Swe => write!(f, "swe"),
            Self::Notes => write!(f, "notes"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// Supported digest file format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DigestFileFormat {
    /// Markdown digest.
    Markdown,
    /// HTML digest.
    Html,
}

impl std::fmt::Display for DigestFileFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Markdown => write!(f, "markdown"),
            Self::Html => write!(f, "html"),
        }
    }
}

/// Sensitivity classification for source material.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DigestSensitivity {
    /// Personal or workplace communication digest.
    SensitiveCommunication,
    /// General notes/news digest.
    PersonalNotes,
}

impl std::fmt::Display for DigestSensitivity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SensitiveCommunication => write!(f, "sensitive_communication"),
            Self::PersonalNotes => write!(f, "personal_notes"),
        }
    }
}

/// Recommended action for a candidate source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DigestProposedAction {
    /// Keep as evidence/index source; do not promote directly into memory.
    IndexAsSource,
    /// Generate candidate memories that need review before activation.
    ReviewGatedExtraction,
}

impl std::fmt::Display for DigestProposedAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IndexAsSource => write!(f, "index_as_source"),
            Self::ReviewGatedExtraction => write!(f, "review_gated_extraction"),
        }
    }
}

fn inventory_digest_sources(options: DigestInventoryOptions) -> IndexResult<DigestInventory> {
    if !options.root_path.exists() {
        return Err(IndexError::FileNotFound(
            options.root_path.display().to_string(),
        ));
    }
    if !options.root_path.is_dir() {
        return Err(IndexError::InvalidState(format!(
            "digest inventory root is not a directory: {}",
            options.root_path.display()
        )));
    }

    let root = fs::canonicalize(&options.root_path)?;
    let mut all_candidates = Vec::new();
    let mut exclusions = Vec::new();
    let mut files_scanned = 0;
    scan_directory(
        &root,
        &root,
        options.include_operational,
        &mut files_scanned,
        &mut all_candidates,
        &mut exclusions,
    )?;

    all_candidates.sort_by(|left, right| {
        left.relative_path
            .cmp(&right.relative_path)
            .then_with(|| left.absolute_path.cmp(&right.absolute_path))
    });
    exclusions.sort_by(|left, right| {
        left.relative_path
            .cmp(&right.relative_path)
            .then_with(|| left.absolute_path.cmp(&right.absolute_path))
    });

    let total_candidates = all_candidates.len();
    let limit = options.limit.unwrap_or(total_candidates);
    let truncated = total_candidates > limit;
    let candidates = all_candidates.into_iter().take(limit).collect::<Vec<_>>();
    let by_source_kind = count_by(
        candidates
            .iter()
            .map(|candidate| candidate.source_kind.to_string()),
    );
    let by_format = count_by(
        candidates
            .iter()
            .map(|candidate| candidate.format.to_string()),
    );
    let by_collection = count_by(
        candidates
            .iter()
            .map(|candidate| candidate.collection.clone()),
    );

    Ok(DigestInventory {
        generated_at: OffsetDateTime::now_utc(),
        root_path: root.display().to_string(),
        files_scanned,
        total_candidates,
        returned_candidates: candidates.len(),
        truncated,
        excluded_count: exclusions.len(),
        by_source_kind,
        by_format,
        by_collection,
        candidates,
        exclusions,
        warnings: vec![
            "Inventory only: no digest contents were read and no Memory OS records were written."
                .to_string(),
            "Digest-derived memories should be generated as review-gated candidates with source evidence."
                .to_string(),
        ],
    })
}

fn export_digest_review_batch(
    output_path: &Path,
    options: DigestInventoryOptions,
) -> IndexResult<DigestReviewExport> {
    let inventory = inventory_digest_sources(options)?;
    fs::create_dir_all(output_path)?;
    let mut export = DigestReviewExport {
        output_path: output_path.display().to_string(),
        inventory,
        files_written: Vec::new(),
        files_skipped: Vec::new(),
    };

    write_digest_review_file(
        output_path,
        Path::new("index.md").to_path_buf(),
        &digest_review_index(&export.inventory),
        &mut export,
    )?;

    let candidates = export.inventory.candidates.clone();
    for (index, candidate) in candidates.iter().enumerate() {
        write_digest_review_file(
            output_path,
            Path::new("candidates").join(format!(
                "{:04}-review-{}.md",
                index + 1,
                slugify(&candidate.relative_path)
            )),
            &digest_review_candidate_page(index + 1, candidate),
            &mut export,
        )?;
    }

    export.files_written.sort();
    export.files_skipped.sort();
    Ok(export)
}

fn apply_digest_review_batch(root: &Path) -> IndexResult<DigestReviewApply> {
    if !root.exists() {
        return Err(IndexError::FileNotFound(root.display().to_string()));
    }
    if !root.is_dir() {
        return Err(IndexError::InvalidState(format!(
            "digest review batch root is not a directory: {}",
            root.display()
        )));
    }

    let mut report = DigestReviewApply {
        root: root.display().to_string(),
        files_scanned: 0,
        files_skipped: Vec::new(),
        files_with_no_decision: Vec::new(),
        files_with_invalid_decision: Vec::new(),
        files_with_parse_errors: Vec::new(),
        accepted_count: 0,
        source_only_count: 0,
        quarantined_count: 0,
        rejected_count: 0,
        planned_sources: Vec::new(),
        warnings: Vec::new(),
    };

    for path in collect_digest_review_files(root)? {
        report.files_scanned += 1;
        let relative_path = relative_path(root, &path);
        let contents = fs::read_to_string(&path)?;
        let Some(reviewed) = parse_digest_review_candidate(&contents, &relative_path, &mut report)
        else {
            continue;
        };

        match reviewed.decision {
            DigestReviewDecision::Accept => {
                report.accepted_count += 1;
                report.planned_sources.push(reviewed);
            }
            DigestReviewDecision::SourceOnly => {
                report.source_only_count += 1;
                report.planned_sources.push(reviewed);
            }
            DigestReviewDecision::Quarantine => {
                report.quarantined_count += 1;
            }
            DigestReviewDecision::Reject => {
                report.rejected_count += 1;
            }
        }
    }

    report.files_skipped.sort();
    report.files_with_no_decision.sort();
    report.files_with_invalid_decision.sort();
    report.files_with_parse_errors.sort();
    report.warnings.sort();
    Ok(report)
}

fn plan_digest_extraction(
    review_path: &Path,
    output_path: &Path,
    options: DigestExtractionOptions,
) -> IndexResult<DigestExtractionPlan> {
    let options = normalize_extraction_options(options);
    let apply = apply_digest_review_batch(review_path)?;
    fs::create_dir_all(output_path)?;

    let mut plan = DigestExtractionPlan {
        review_path: review_path.display().to_string(),
        output_path: output_path.display().to_string(),
        review_files_scanned: apply.files_scanned,
        accepted_sources: 0,
        source_only_sources: apply.source_only_count,
        sources_read: 0,
        sources_skipped: Vec::new(),
        files_written: Vec::new(),
        files_skipped: Vec::new(),
        candidates: Vec::new(),
        warnings: apply.warnings.clone(),
    };

    let mut pending_pages = Vec::new();
    for source in apply.planned_sources {
        match source.decision {
            DigestReviewDecision::Accept => {
                plan.accepted_sources += 1;
                collect_extraction_pages_for_source(
                    &source,
                    &options,
                    &mut plan,
                    &mut pending_pages,
                );
            }
            DigestReviewDecision::SourceOnly => {
                plan.warnings.push(format!(
                    "{}: source_only decision; source content was not read for extraction",
                    source.review_path
                ));
            }
            DigestReviewDecision::Quarantine | DigestReviewDecision::Reject => {}
        }
    }

    write_digest_extraction_file(
        output_path,
        Path::new("index.md").to_path_buf(),
        &digest_extraction_index(&plan),
        &mut plan,
    )?;

    for (relative_path, contents) in pending_pages {
        write_digest_extraction_file(output_path, relative_path, &contents, &mut plan)?;
    }

    plan.files_written.sort();
    plan.files_skipped.sort();
    plan.sources_skipped.sort();
    plan.warnings.sort();
    Ok(plan)
}

fn plan_digest_source_index(
    review_path: &Path,
    options: DigestSourceIndexOptions,
) -> IndexResult<DigestSourceIndexPlan> {
    let options = normalize_source_index_options(options);
    let apply = apply_digest_review_batch(review_path)?;

    let mut plan = DigestSourceIndexPlan {
        review_path: review_path.display().to_string(),
        review_files_scanned: apply.files_scanned,
        accepted_sources: apply.accepted_count,
        source_only_sources: apply.source_only_count,
        sources_read: 0,
        sources_skipped: Vec::new(),
        documents: Vec::new(),
        warnings: apply.warnings.clone(),
    };

    for source in apply.planned_sources {
        match source.decision {
            DigestReviewDecision::SourceOnly => {
                collect_source_index_document(&source, &options, &mut plan);
            }
            DigestReviewDecision::Accept => {
                plan.warnings.push(format!(
                    "{}: accept decision is reserved for extraction; source was not indexed by source_only indexing",
                    source.review_path
                ));
            }
            DigestReviewDecision::Quarantine | DigestReviewDecision::Reject => {}
        }
    }

    plan.sources_skipped.sort();
    plan.warnings.sort();
    Ok(plan)
}

/// Parse a reviewed digest extraction batch. The caller owns persistence.
pub fn apply_digest_extraction_review_batch(
    root: &Path,
    options: DigestExtractionReviewApplyOptions,
    existing_candidate_tags: HashSet<String>,
) -> IndexResult<DigestExtractionReviewApply> {
    if !root.exists() {
        return Err(IndexError::FileNotFound(root.display().to_string()));
    }
    if !root.is_dir() {
        return Err(IndexError::InvalidState(format!(
            "digest extraction review batch root is not a directory: {}",
            root.display()
        )));
    }

    let mut report = DigestExtractionReviewApply {
        root: root.display().to_string(),
        dry_run: options.dry_run,
        files_scanned: 0,
        files_skipped: Vec::new(),
        files_with_no_decision: Vec::new(),
        files_with_invalid_decision: Vec::new(),
        files_with_parse_errors: Vec::new(),
        accepted_count: 0,
        quarantined_count: 0,
        rejected_count: 0,
        duplicate_count: 0,
        planned_items: Vec::new(),
        written_items: Vec::new(),
        commit: None,
        warnings: Vec::new(),
    };

    let mut seen_candidate_tags = existing_candidate_tags;
    for path in collect_digest_review_files(root)? {
        report.files_scanned += 1;
        let relative_path = relative_path(root, &path);
        let contents = fs::read_to_string(&path)?;
        let Some(parsed) =
            parse_digest_extraction_candidate(&contents, &relative_path, &mut report)
        else {
            continue;
        };

        apply_parsed_digest_extraction_candidate(
            parsed,
            &options,
            &mut seen_candidate_tags,
            &mut report,
            &relative_path,
        );
    }

    report.files_skipped.sort();
    report.files_with_no_decision.sort();
    report.files_with_invalid_decision.sort();
    report.files_with_parse_errors.sort();
    report.warnings.sort();
    Ok(report)
}

/// Build the knowledge commit for written digest extraction memory items.
#[must_use]
pub fn build_digest_extraction_commit(
    writer: &WriterProvenance,
    written_items: &[MemoryItem],
) -> KnowledgeCommit {
    let suffix = if written_items.len() == 1 { "" } else { "s" };
    let mut commit = KnowledgeCommit::new(
        writer.clone(),
        format!(
            "Apply reviewed digest extraction batch ({} item{})",
            written_items.len(),
            suffix
        ),
    );

    for item in written_items {
        commit = commit.with_change(
            MemoryChange::new(
                MemoryChangeType::Added,
                &item.title,
                "Imported a reviewed digest-derived memory item into Memory OS.",
            )
            .with_item(item.id),
        );
    }

    commit
}

fn digest_review_index(inventory: &DigestInventory) -> String {
    let mut output = digest_review_frontmatter("digest_review_index", Vec::new());
    output.push_str("# Digest Source Review Batch\n\n");
    output.push_str(
        "This generated batch is metadata-only. It does not include digest contents and ",
    );
    output.push_str("does not promote any digest-derived fact into active Memory OS records.\n\n");

    output.push_str("## Summary\n\n");
    output.push_str(&format!("- Root: `{}`\n", inventory.root_path));
    output.push_str(&format!("- Files scanned: {}\n", inventory.files_scanned));
    output.push_str(&format!(
        "- Total candidates: {}\n",
        inventory.total_candidates
    ));
    output.push_str(&format!(
        "- Returned candidates: {}\n",
        inventory.returned_candidates
    ));
    output.push_str(&format!("- Truncated: {}\n", inventory.truncated));
    output.push_str(&format!(
        "- Excluded files: {}\n\n",
        inventory.excluded_count
    ));

    output.push_str("## Candidate Counts\n\n");
    output.push_str("### By Source Kind\n\n");
    append_counts(&mut output, &inventory.by_source_kind);
    output.push_str("\n### By Format\n\n");
    append_counts(&mut output, &inventory.by_format);
    output.push_str("\n### By Collection\n\n");
    append_counts(&mut output, &inventory.by_collection);

    output.push_str("\n## Review Candidates\n\n");
    if inventory.candidates.is_empty() {
        output.push_str("No digest source candidates were found.\n");
    } else {
        for (index, candidate) in inventory.candidates.iter().enumerate() {
            output.push_str(&format!(
                "- [Candidate {:04}](candidates/{:04}-review-{}.md) - `{}` - {} - {}\n",
                index + 1,
                index + 1,
                slugify(&candidate.relative_path),
                candidate.relative_path,
                candidate.source_kind,
                candidate.proposed_action
            ));
        }
    }

    if !inventory.exclusions.is_empty() {
        output.push_str("\n## Exclusions\n\n");
        for exclusion in &inventory.exclusions {
            output.push_str(&format!(
                "- `{}` - {}\n",
                exclusion.relative_path, exclusion.reason
            ));
        }
    }

    output.push_str("\n## Safety\n\n");
    for warning in &inventory.warnings {
        output.push_str(&format!("- {}\n", warning));
    }
    output
}

fn digest_review_candidate_page(index: usize, candidate: &DigestSourceCandidate) -> String {
    let mut fields = vec![
        ("candidate_number".to_string(), index.to_string()),
        (
            "relative_path".to_string(),
            yaml_string(&candidate.relative_path),
        ),
        (
            "source_kind".to_string(),
            yaml_string(&candidate.source_kind.to_string()),
        ),
        ("collection".to_string(), yaml_string(&candidate.collection)),
        (
            "format".to_string(),
            yaml_string(&candidate.format.to_string()),
        ),
        (
            "sensitivity".to_string(),
            yaml_string(&candidate.sensitivity.to_string()),
        ),
        (
            "proposed_action".to_string(),
            yaml_string(&candidate.proposed_action.to_string()),
        ),
    ];
    if let Some(bucket) = &candidate.bucket {
        fields.push(("bucket".to_string(), yaml_string(bucket)));
    }
    if let Some(date_hint) = &candidate.date_hint {
        fields.push(("date_hint".to_string(), yaml_string(date_hint)));
    }

    let mut output = digest_review_frontmatter("digest_review_candidate", fields);
    output.push_str(&format!("# Digest Review: {}\n\n", candidate.relative_path));
    output.push_str("## Source\n\n");
    output.push_str(&format!("- Absolute path: `{}`\n", candidate.absolute_path));
    output.push_str(&format!("- Source kind: `{}`\n", candidate.source_kind));
    output.push_str(&format!("- Collection: `{}`\n", candidate.collection));
    if let Some(bucket) = &candidate.bucket {
        output.push_str(&format!("- Bucket: `{bucket}`\n"));
    }
    if let Some(date_hint) = &candidate.date_hint {
        output.push_str(&format!("- Date hint: `{date_hint}`\n"));
    }
    output.push_str(&format!("- Format: `{}`\n", candidate.format));
    output.push_str(&format!("- Sensitivity: `{}`\n", candidate.sensitivity));
    output.push_str(&format!(
        "- Proposed action: `{}`\n",
        candidate.proposed_action
    ));
    output.push_str(&format!("- Size: {} bytes\n", candidate.size_bytes));
    if let Some(modified_at) = candidate.modified_at {
        output.push_str(&format!("- Modified: {}\n", modified_at));
    }

    output.push_str("\n## Classification Reasons\n\n");
    for reason in &candidate.reasons {
        output.push_str(&format!("- {}\n", reason));
    }

    output.push_str("\n## Review Decision\n\n");
    output.push_str("Set `decision` before a future apply step consumes this file.\n\n");
    output.push_str("```yaml\n");
    output.push_str("decision: pending # accept | reject | quarantine | source_only\n");
    output.push_str("memory_kind: null # preference | rule | decision | limitation | ");
    output.push_str("project_fact | user_fact | task_fact | session_insight\n");
    output.push_str(
        "scope_type: null # global | user | project | task | entity | repository | custom\n",
    );
    output.push_str("scope_name: null\n");
    output.push_str("title: null\n");
    output.push_str("notes: null\n");
    output.push_str("```\n\n");

    output.push_str("## Safety\n\n");
    output.push_str("- This generated page intentionally omits digest contents.\n");
    output.push_str("- Use the source path as evidence if a later reviewed memory is created.\n");
    output.push_str(
        "- Communication digests require explicit review before active memory promotion.\n",
    );

    output.push_str("\n## Machine Record\n\n");
    output.push_str("```json\n");
    output.push_str(
        &serde_json::to_string_pretty(candidate)
            .expect("digest candidate JSON serialization should succeed"),
    );
    output.push_str("\n```\n");
    output
}

fn append_counts(output: &mut String, counts: &BTreeMap<String, usize>) {
    if counts.is_empty() {
        output.push_str("None.\n");
    } else {
        for (key, count) in counts {
            output.push_str(&format!("- {}: {}\n", key, count));
        }
    }
}

fn digest_review_frontmatter(page_type: &str, fields: Vec<(String, String)>) -> String {
    let mut output = String::new();
    output.push_str("---\n");
    output.push_str("generated_by: \"engram-memory-os\"\n");
    output.push_str(&format!("page_type: {}\n", yaml_string(page_type)));
    for (key, value) in fields {
        output.push_str(&format!("{key}: {value}\n"));
    }
    output.push_str("---\n\n");
    output.push_str(DIGEST_REVIEW_MARKER);
    output.push_str("\n\n");
    output
}

fn write_digest_review_file(
    root: &Path,
    relative_path: PathBuf,
    contents: &str,
    export: &mut DigestReviewExport,
) -> IndexResult<()> {
    let path = root.join(&relative_path);
    if path.exists() {
        let existing = fs::read(&path)?;
        if !String::from_utf8_lossy(&existing).contains(DIGEST_REVIEW_MARKER) {
            export.files_skipped.push(path_to_markdown(&relative_path));
            return Ok(());
        }
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, contents)?;
    export.files_written.push(path_to_markdown(&relative_path));
    Ok(())
}

fn collect_source_index_document(
    source: &DigestReviewedSource,
    options: &DigestSourceIndexOptions,
    plan: &mut DigestSourceIndexPlan,
) {
    let source_path = Path::new(&source.candidate.absolute_path);
    let source_label = format!(
        "{} ({})",
        source.candidate.relative_path, source.review_path
    );
    let metadata = match fs::metadata(source_path) {
        Ok(metadata) => metadata,
        Err(error) => {
            plan.sources_skipped.push(format!(
                "{source_label}: source metadata unavailable: {error}"
            ));
            return;
        }
    };

    if metadata.len() != source.candidate.size_bytes {
        plan.sources_skipped.push(format!(
            "{source_label}: source size changed from {} to {} bytes; re-run inventory/review",
            source.candidate.size_bytes,
            metadata.len()
        ));
        return;
    }
    let modified_at = metadata.modified().ok().map(OffsetDateTime::from);
    if source.candidate.modified_at.is_some() && modified_at != source.candidate.modified_at {
        plan.sources_skipped.push(format!(
            "{source_label}: source modified timestamp changed; re-run inventory/review"
        ));
        return;
    }

    if metadata.len() as usize > options.max_source_bytes {
        plan.sources_skipped.push(format!(
            "{source_label}: source is {} bytes, above max_source_bytes={}",
            metadata.len(),
            options.max_source_bytes
        ));
        return;
    }

    let raw = match fs::read_to_string(source_path) {
        Ok(raw) => raw,
        Err(error) => {
            plan.sources_skipped
                .push(format!("{source_label}: source text unreadable: {error}"));
            return;
        }
    };
    let source_text = digest_source_to_text(&raw, source.candidate.format);
    if source_text.trim().is_empty() {
        plan.warnings.push(format!(
            "{source_label}: no source text found after text normalization"
        ));
        return;
    }
    plan.sources_read += 1;

    let title = source
        .title
        .as_ref()
        .filter(|title| !title.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| format!("Digest source: {}", source.candidate.relative_path));
    let indexed_content = digest_source_index_content(source, &title, &source_text);
    plan.documents.push(DigestSourceIndexDocument {
        source_review_path: source.review_path.clone(),
        source_relative_path: source.candidate.relative_path.clone(),
        source_absolute_path: source.candidate.absolute_path.clone(),
        source_kind: source.candidate.source_kind,
        title,
        content_chars: source_text.chars().count(),
        document_path: source.candidate.absolute_path.clone(),
        indexed_content,
    });
}

fn collect_extraction_pages_for_source(
    source: &DigestReviewedSource,
    options: &DigestExtractionOptions,
    plan: &mut DigestExtractionPlan,
    pending_pages: &mut Vec<(PathBuf, String)>,
) {
    let source_path = Path::new(&source.candidate.absolute_path);
    let source_label = format!(
        "{} ({})",
        source.candidate.relative_path, source.review_path
    );
    let metadata = match fs::metadata(source_path) {
        Ok(metadata) => metadata,
        Err(error) => {
            plan.sources_skipped.push(format!(
                "{source_label}: source metadata unavailable: {error}"
            ));
            return;
        }
    };

    if metadata.len() != source.candidate.size_bytes {
        plan.sources_skipped.push(format!(
            "{source_label}: source size changed from {} to {} bytes; re-run inventory/review",
            source.candidate.size_bytes,
            metadata.len()
        ));
        return;
    }
    let modified_at = metadata.modified().ok().map(OffsetDateTime::from);
    if source.candidate.modified_at.is_some() && modified_at != source.candidate.modified_at {
        plan.sources_skipped.push(format!(
            "{source_label}: source modified timestamp changed; re-run inventory/review"
        ));
        return;
    }

    if metadata.len() as usize > options.max_source_bytes {
        plan.sources_skipped.push(format!(
            "{source_label}: source is {} bytes, above max_source_bytes={}",
            metadata.len(),
            options.max_source_bytes
        ));
        return;
    }

    let raw = match fs::read_to_string(source_path) {
        Ok(raw) => raw,
        Err(error) => {
            plan.sources_skipped
                .push(format!("{source_label}: source text unreadable: {error}"));
            return;
        }
    };
    plan.sources_read += 1;

    let text = digest_source_to_text(&raw, source.candidate.format);
    let excerpts = extraction_excerpts(
        &text,
        options.max_candidates_per_source,
        options.max_candidate_chars,
    );
    if excerpts.is_empty() {
        plan.warnings.push(format!(
            "{source_label}: no extraction candidates found after text normalization"
        ));
        return;
    }

    for excerpt in excerpts {
        let number = plan.candidates.len() + 1;
        let title = extraction_candidate_title(source, number);
        let relative_path =
            Path::new("candidates").join(format!("{:04}-candidate-{}.md", number, slugify(&title)));
        let review_path = path_to_markdown(&relative_path);
        let summary = DigestExtractionCandidateSummary {
            review_path: review_path.clone(),
            source_review_path: source.review_path.clone(),
            source_relative_path: source.candidate.relative_path.clone(),
            source_kind: source.candidate.source_kind,
            title: title.clone(),
            memory_kind: source.memory_kind.clone(),
            scope_type: source.scope_type.clone(),
            scope_name: source.scope_name.clone(),
            content_chars: excerpt.chars().count(),
        };
        pending_pages.push((
            relative_path,
            digest_extraction_candidate_page(&summary, source, &excerpt),
        ));
        plan.candidates.push(summary);
    }
}

fn digest_extraction_index(plan: &DigestExtractionPlan) -> String {
    let mut output = extraction_frontmatter("digest_extraction_index", Vec::new());
    output.push_str("# Digest Extraction Plan\n\n");
    output.push_str(
        "This generated batch contains review-gated candidate memory excerpts from sources ",
    );
    output.push_str(
        "explicitly accepted in a digest review batch. It does not write active memory.\n\n",
    );

    output.push_str("## Summary\n\n");
    output.push_str(&format!("- Review path: `{}`\n", plan.review_path));
    output.push_str(&format!("- Files scanned: {}\n", plan.review_files_scanned));
    output.push_str(&format!("- Accepted sources: {}\n", plan.accepted_sources));
    output.push_str(&format!(
        "- Source-only sources: {}\n",
        plan.source_only_sources
    ));
    output.push_str(&format!("- Sources read: {}\n", plan.sources_read));
    output.push_str(&format!(
        "- Candidate memories: {}\n\n",
        plan.candidate_count()
    ));

    output.push_str("## Candidate Memories\n\n");
    if plan.candidates.is_empty() {
        output.push_str("No candidate memories were generated.\n");
    } else {
        for candidate in &plan.candidates {
            output.push_str(&format!(
                "- [{}]({}) - `{}` - {} chars\n",
                escape_link_text(&candidate.title),
                candidate.review_path,
                candidate.source_relative_path,
                candidate.content_chars
            ));
        }
    }

    if !plan.sources_skipped.is_empty() {
        output.push_str("\n## Skipped Sources\n\n");
        for skipped in &plan.sources_skipped {
            output.push_str(&format!("- {}\n", skipped));
        }
    }

    if !plan.warnings.is_empty() {
        output.push_str("\n## Warnings\n\n");
        for warning in &plan.warnings {
            output.push_str(&format!("- {}\n", warning));
        }
    }

    output.push_str("\n## Safety\n\n");
    output.push_str("- Only `accept` decisions are read for extraction.\n");
    output.push_str("- `source_only`, `quarantine`, `reject`, and pending sources are not read.\n");
    output.push_str(
        "- Generated candidate memories require a later human review before activation.\n",
    );
    output
}

fn digest_extraction_candidate_page(
    summary: &DigestExtractionCandidateSummary,
    source: &DigestReviewedSource,
    excerpt: &str,
) -> String {
    let mut fields = vec![
        ("title".to_string(), yaml_string(&summary.title)),
        (
            "source_review_path".to_string(),
            yaml_string(&summary.source_review_path),
        ),
        (
            "source_relative_path".to_string(),
            yaml_string(&summary.source_relative_path),
        ),
        (
            "source_kind".to_string(),
            yaml_string(&summary.source_kind.to_string()),
        ),
    ];
    if let Some(memory_kind) = &summary.memory_kind {
        fields.push(("memory_kind".to_string(), yaml_string(memory_kind)));
    }
    if let Some(scope_type) = &summary.scope_type {
        fields.push(("scope_type".to_string(), yaml_string(scope_type)));
    }
    if let Some(scope_name) = &summary.scope_name {
        fields.push(("scope_name".to_string(), yaml_string(scope_name)));
    }

    let mut output = extraction_frontmatter("digest_extraction_candidate", fields);
    output.push_str(&format!("# Candidate Memory: {}\n\n", summary.title));
    output.push_str("## Candidate Content\n\n");
    output.push_str(excerpt.trim());
    output.push_str("\n\n## Review Decision\n\n");
    output.push_str("Set `decision` before any future apply step writes memory.\n\n");
    output.push_str("```yaml\n");
    output.push_str("decision: pending # accept | reject | quarantine\n");
    output.push_str(&format!(
        "memory_kind: {}\n",
        optional_yaml_string(summary.memory_kind.as_deref())
    ));
    output.push_str(&format!(
        "scope_type: {}\n",
        optional_yaml_string(summary.scope_type.as_deref())
    ));
    output.push_str(&format!(
        "scope_name: {}\n",
        optional_yaml_string(summary.scope_name.as_deref())
    ));
    output.push_str(&format!("title: {}\n", yaml_string(&summary.title)));
    output.push_str("notes: null\n");
    output.push_str("```\n\n");

    output.push_str("## Source Evidence\n\n");
    output.push_str(&format!("- Source review: `{}`\n", source.review_path));
    output.push_str(&format!(
        "- Source path: `{}`\n",
        source.candidate.relative_path
    ));
    output.push_str(&format!(
        "- Absolute path: `{}`\n",
        source.candidate.absolute_path
    ));
    output.push_str(&format!(
        "- Source kind: `{}`\n",
        source.candidate.source_kind
    ));
    if let Some(date_hint) = &source.candidate.date_hint {
        output.push_str(&format!("- Date hint: `{date_hint}`\n"));
    }
    if let Some(notes) = &source.notes {
        output.push_str(&format!("- Reviewer notes: {}\n", notes));
    }

    output.push_str("\n## Safety\n\n");
    output.push_str("- This is a candidate memory excerpt, not active memory.\n");
    output.push_str("- A future apply step must preserve source evidence and writer provenance.\n");

    output.push_str("\n## Machine Record\n\n");
    output.push_str("```json\n");
    let record = DigestExtractionCandidateRecord {
        summary: summary.clone(),
        source_candidate: source.candidate.clone(),
    };
    output.push_str(
        &serde_json::to_string_pretty(&record)
            .expect("digest extraction candidate JSON serialization should succeed"),
    );
    output.push_str("\n```\n");
    output
}

fn write_digest_extraction_file(
    root: &Path,
    relative_path: PathBuf,
    contents: &str,
    plan: &mut DigestExtractionPlan,
) -> IndexResult<()> {
    let path = root.join(&relative_path);
    if path.exists() {
        let existing = fs::read(&path)?;
        if !String::from_utf8_lossy(&existing).contains(DIGEST_EXTRACTION_MARKER) {
            plan.files_skipped.push(path_to_markdown(&relative_path));
            return Ok(());
        }
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, contents)?;
    plan.files_written.push(path_to_markdown(&relative_path));
    Ok(())
}

fn digest_source_index_content(
    source: &DigestReviewedSource,
    title: &str,
    source_text: &str,
) -> String {
    let mut output = String::new();
    output.push_str(&format!("# {title}\n\n"));
    output.push_str("## Source Metadata\n\n");
    output.push_str(&format!("- Source review: `{}`\n", source.review_path));
    output.push_str(&format!(
        "- Source path: `{}`\n",
        source.candidate.relative_path
    ));
    output.push_str(&format!(
        "- Absolute path: `{}`\n",
        source.candidate.absolute_path
    ));
    output.push_str(&format!(
        "- Source kind: `{}`\n",
        source.candidate.source_kind
    ));
    output.push_str(&format!("- Format: `{}`\n", source.candidate.format));
    if let Some(date_hint) = &source.candidate.date_hint {
        output.push_str(&format!("- Date hint: `{date_hint}`\n"));
    }
    if let Some(notes) = &source.notes {
        output.push_str(&format!("- Reviewer notes: {}\n", notes));
    }

    output.push_str("\n## Digest Content\n\n");
    output.push_str(source_text.trim());
    output.push('\n');
    output
}

fn extraction_frontmatter(page_type: &str, fields: Vec<(String, String)>) -> String {
    let mut output = String::new();
    output.push_str("---\n");
    output.push_str("generated_by: \"engram-memory-os\"\n");
    output.push_str(&format!("page_type: {}\n", yaml_string(page_type)));
    for (key, value) in fields {
        output.push_str(&format!("{key}: {value}\n"));
    }
    output.push_str("---\n\n");
    output.push_str(DIGEST_EXTRACTION_MARKER);
    output.push_str("\n\n");
    output
}

fn collect_digest_review_files(root: &Path) -> IndexResult<Vec<PathBuf>> {
    let candidates_dir = root.join("candidates");
    if !candidates_dir.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    for entry in fs::read_dir(candidates_dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|extension| extension == "md") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn parse_digest_review_candidate(
    contents: &str,
    relative_path: &str,
    report: &mut DigestReviewApply,
) -> Option<DigestReviewedSource> {
    if !contents.contains(DIGEST_REVIEW_MARKER) {
        report.files_skipped.push(relative_path.to_string());
        report.warnings.push(format!(
            "{relative_path}: skipped non-generated digest review file"
        ));
        return None;
    }

    let fields = review_decision_fields(contents);
    let decision_value = fields
        .get("decision")
        .and_then(|value| optional_value(value));
    let Some(decision_value) = decision_value else {
        report
            .files_with_no_decision
            .push(relative_path.to_string());
        return None;
    };
    if decision_value.eq_ignore_ascii_case("pending") {
        report
            .files_with_no_decision
            .push(relative_path.to_string());
        return None;
    }
    let Some(decision) = parse_digest_review_decision(&decision_value) else {
        report
            .files_with_invalid_decision
            .push(relative_path.to_string());
        report.warnings.push(format!(
            "{relative_path}: invalid digest review decision `{decision_value}`"
        ));
        return None;
    };

    let Some(candidate_json) = digest_machine_record_json(contents) else {
        report
            .files_with_parse_errors
            .push(relative_path.to_string());
        report
            .warnings
            .push(format!("{relative_path}: missing digest machine record"));
        return None;
    };
    let candidate = match serde_json::from_str::<DigestSourceCandidate>(candidate_json) {
        Ok(candidate) => candidate,
        Err(error) => {
            report
                .files_with_parse_errors
                .push(relative_path.to_string());
            report.warnings.push(format!(
                "{relative_path}: invalid digest machine record: {error}"
            ));
            return None;
        }
    };

    Some(DigestReviewedSource {
        review_path: relative_path.to_string(),
        decision,
        candidate,
        memory_kind: fields
            .get("memory_kind")
            .and_then(|value| optional_value(value)),
        scope_type: fields
            .get("scope_type")
            .and_then(|value| optional_value(value)),
        scope_name: fields
            .get("scope_name")
            .and_then(|value| optional_value(value)),
        title: fields.get("title").and_then(|value| optional_value(value)),
        notes: fields.get("notes").and_then(|value| optional_value(value)),
    })
}

fn review_decision_fields(contents: &str) -> BTreeMap<String, String> {
    let Some(block) = review_decision_yaml(contents) else {
        return BTreeMap::new();
    };

    block
        .lines()
        .filter_map(|line| {
            let (key, value) = line.split_once(':')?;
            let key = key.trim();
            if key.is_empty() {
                return None;
            }
            Some((
                key.to_string(),
                strip_inline_comment(value).trim().to_string(),
            ))
        })
        .collect()
}

fn review_decision_yaml(contents: &str) -> Option<&str> {
    let heading_start = contents.find("## Review Decision")?;
    let after_heading = &contents[heading_start..];
    let fence_start = after_heading.find("```yaml")?;
    let after_fence = &after_heading[fence_start + "```yaml".len()..];
    let block = after_fence.strip_prefix('\n').unwrap_or(after_fence);
    let fence_end = block.find("```")?;
    Some(block[..fence_end].trim())
}

fn strip_inline_comment(value: &str) -> &str {
    let mut in_quote = false;
    let mut escaped = false;
    for (index, ch) in value.char_indices() {
        match ch {
            '"' if !escaped => in_quote = !in_quote,
            '#' if !in_quote => return &value[..index],
            _ => {}
        }
        escaped = ch == '\\' && !escaped;
        if ch != '\\' {
            escaped = false;
        }
    }
    value
}

fn optional_value(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || value.eq_ignore_ascii_case("null") {
        return None;
    }
    if value.starts_with('"') && value.ends_with('"') {
        return serde_json::from_str::<String>(value).ok();
    }
    Some(value.to_string())
}

fn parse_digest_review_decision(value: &str) -> Option<DigestReviewDecision> {
    match value.trim().to_lowercase().as_str() {
        "accept" => Some(DigestReviewDecision::Accept),
        "source_only" | "source-only" | "source only" => Some(DigestReviewDecision::SourceOnly),
        "quarantine" => Some(DigestReviewDecision::Quarantine),
        "reject" => Some(DigestReviewDecision::Reject),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DigestExtractionReviewDecision {
    Accept,
    Quarantine,
    Reject,
}

struct ParsedDigestExtractionCandidate {
    decision: DigestExtractionReviewDecision,
    title: String,
    content: String,
    memory_kind: Option<String>,
    scope_type: Option<String>,
    scope_name: Option<String>,
    source_review_path: Option<String>,
    source_relative_path: Option<String>,
    absolute_path: Option<String>,
    source_kind: Option<String>,
    notes: Option<String>,
}

fn parse_digest_extraction_candidate(
    contents: &str,
    relative_path: &str,
    report: &mut DigestExtractionReviewApply,
) -> Option<ParsedDigestExtractionCandidate> {
    if !contents.contains(DIGEST_EXTRACTION_MARKER) {
        report.files_skipped.push(relative_path.to_string());
        report.warnings.push(format!(
            "{relative_path}: skipped non-generated digest extraction file"
        ));
        return None;
    }

    let fields = review_decision_fields(contents);
    let decision_value = fields
        .get("decision")
        .and_then(|value| optional_value(value));
    let Some(decision_value) = decision_value else {
        report
            .files_with_no_decision
            .push(relative_path.to_string());
        return None;
    };
    if decision_value.eq_ignore_ascii_case("pending") {
        report
            .files_with_no_decision
            .push(relative_path.to_string());
        return None;
    }
    let Some(decision) = parse_digest_extraction_review_decision(&decision_value) else {
        report
            .files_with_invalid_decision
            .push(relative_path.to_string());
        report.warnings.push(format!(
            "{relative_path}: invalid digest extraction review decision `{decision_value}`"
        ));
        return None;
    };

    let Some(content) = markdown_section(contents, "## Candidate Content") else {
        report
            .files_with_parse_errors
            .push(relative_path.to_string());
        report.warnings.push(format!(
            "{relative_path}: missing candidate content section"
        ));
        return None;
    };
    if content.trim().is_empty() {
        report
            .files_with_parse_errors
            .push(relative_path.to_string());
        report
            .warnings
            .push(format!("{relative_path}: candidate content is empty"));
        return None;
    }

    let record = match parse_digest_extraction_candidate_record(contents, relative_path, report) {
        Ok(record) => record,
        Err(()) => return None,
    };
    let evidence = record.as_ref().map(|record| {
        (
            Some(record.summary.source_review_path.clone()),
            Some(record.summary.source_relative_path.clone()),
            Some(record.source_candidate.absolute_path.clone()),
            Some(record.summary.source_kind.to_string()),
        )
    });
    let (record_review, record_source, record_absolute, record_kind) =
        evidence.unwrap_or((None, None, None, None));

    Some(ParsedDigestExtractionCandidate {
        decision,
        title: fields
            .get("title")
            .and_then(|value| optional_value(value))
            .or_else(|| candidate_memory_heading(contents))
            .unwrap_or_else(|| "Digest candidate memory".to_string()),
        content,
        memory_kind: fields
            .get("memory_kind")
            .and_then(|value| optional_value(value)),
        scope_type: fields
            .get("scope_type")
            .and_then(|value| optional_value(value)),
        scope_name: fields
            .get("scope_name")
            .and_then(|value| optional_value(value)),
        source_review_path: record_review
            .or_else(|| source_evidence_value(contents, "Source review")),
        source_relative_path: record_source
            .or_else(|| source_evidence_value(contents, "Source path")),
        absolute_path: record_absolute.or_else(|| source_evidence_value(contents, "Absolute path")),
        source_kind: record_kind.or_else(|| source_evidence_value(contents, "Source kind")),
        notes: fields.get("notes").and_then(|value| optional_value(value)),
    })
}

fn parse_digest_extraction_candidate_record(
    contents: &str,
    relative_path: &str,
    report: &mut DigestExtractionReviewApply,
) -> Result<Option<DigestExtractionCandidateRecord>, ()> {
    let Some(candidate_json) = digest_machine_record_json(contents) else {
        return Ok(None);
    };

    match serde_json::from_str::<DigestExtractionCandidateRecord>(candidate_json) {
        Ok(record) => Ok(Some(record)),
        Err(error) => {
            report
                .files_with_parse_errors
                .push(relative_path.to_string());
            report.warnings.push(format!(
                "{relative_path}: invalid digest extraction machine record: {error}"
            ));
            Err(())
        }
    }
}

fn parse_digest_extraction_review_decision(value: &str) -> Option<DigestExtractionReviewDecision> {
    match value.trim().to_lowercase().as_str() {
        "accept" => Some(DigestExtractionReviewDecision::Accept),
        "quarantine" => Some(DigestExtractionReviewDecision::Quarantine),
        "reject" => Some(DigestExtractionReviewDecision::Reject),
        _ => None,
    }
}

fn apply_parsed_digest_extraction_candidate(
    parsed: ParsedDigestExtractionCandidate,
    options: &DigestExtractionReviewApplyOptions,
    seen_candidate_tags: &mut HashSet<String>,
    report: &mut DigestExtractionReviewApply,
    relative_path: &str,
) {
    match parsed.decision {
        DigestExtractionReviewDecision::Accept => {
            let candidate_tag = digest_extraction_candidate_tag(&parsed, relative_path);
            let item = match memory_item_from_digest_extraction(
                parsed,
                options,
                relative_path,
                &candidate_tag,
            ) {
                Ok(item) => item,
                Err(error) => {
                    report
                        .files_with_parse_errors
                        .push(relative_path.to_string());
                    report.warnings.push(format!("{relative_path}: {error}"));
                    return;
                }
            };

            if !seen_candidate_tags.insert(candidate_tag.clone()) {
                report.duplicate_count += 1;
                report.warnings.push(format!(
                    "{relative_path}: candidate review was already imported; skipped duplicate"
                ));
                return;
            }

            report.accepted_count += 1;
            report.planned_items.push(item);
        }
        DigestExtractionReviewDecision::Quarantine => {
            report.quarantined_count += 1;
        }
        DigestExtractionReviewDecision::Reject => {
            report.rejected_count += 1;
        }
    }
}

fn memory_item_from_digest_extraction(
    parsed: ParsedDigestExtractionCandidate,
    options: &DigestExtractionReviewApplyOptions,
    relative_path: &str,
    candidate_tag: &str,
) -> Result<MemoryItem, String> {
    let memory_kind = parsed
        .memory_kind
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(MemoryKind::parse)
        .ok_or_else(|| "accepted candidate is missing memory_kind".to_string())?;
    let scope_type = parsed
        .scope_type
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "accepted candidate is missing scope_type".to_string())?;
    let scope = digest_scope_from_review(scope_type, parsed.scope_name.as_deref())?;
    let title = parsed.title.trim();
    if title.is_empty() {
        return Err("accepted candidate is missing title".to_string());
    }
    let content = parsed.content.trim();
    if content.is_empty() {
        return Err("accepted candidate content is empty".to_string());
    }

    let source_target = parsed
        .absolute_path
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .or(parsed.source_relative_path.as_deref())
        .ok_or_else(|| "accepted candidate is missing source evidence path".to_string())?;
    let source_label = parsed
        .source_relative_path
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(source_target);
    let source_kind = parsed.source_kind.as_deref().unwrap_or("unknown");
    let source_evidence = EvidenceRef::new(EvidenceKind::File, source_target)
        .with_summary(format!(
            "Reviewed digest source: {source_label} ({source_kind})"
        ))
        .with_excerpt(content.chars().take(500).collect::<String>());
    let mut review_summary =
        "Accepted from a generated Engram digest extraction review batch.".to_string();
    if let Some(notes) = parsed
        .notes
        .as_ref()
        .filter(|notes| !notes.trim().is_empty())
    {
        review_summary.push_str(" Reviewer notes: ");
        review_summary.push_str(notes.trim());
    }
    let review_evidence =
        EvidenceRef::new(EvidenceKind::ManualReview, relative_path).with_summary(review_summary);

    let mut item = MemoryItem::new(
        memory_kind,
        title,
        content,
        scope,
        ClaimOrigin::Imported,
        options.writer.clone(),
    )
    .with_status(MemoryStatus::Active)
    .with_evidence(source_evidence)
    .with_evidence(review_evidence)
    .with_tag("digest")
    .with_tag("digest-reviewed")
    .with_tag("digest-extraction")
    .with_tag(candidate_tag);

    if let Some(source_relative_path) = parsed
        .source_relative_path
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        item = item.with_tag(format!("digest-source:{source_relative_path}"));
    }
    if let Some(source_review_path) = parsed
        .source_review_path
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        item = item.with_tag(format!("digest-source-review:{source_review_path}"));
    }
    if source_kind != "unknown" {
        item = item.with_tag(format!("digest-source-kind:{source_kind}"));
    }

    Ok(item)
}

fn digest_scope_from_review(
    scope_type: &str,
    scope_name: Option<&str>,
) -> Result<MemoryScope, String> {
    let scope_name = scope_name.map(str::trim).filter(|value| !value.is_empty());
    match scope_type.trim().to_lowercase().replace('-', "_").as_str() {
        "global" => Ok(MemoryScope::Global),
        "user" => Ok(MemoryScope::User),
        "project" => Ok(MemoryScope::project(required_scope_name(
            scope_name,
            "project",
        )?)),
        "task" => Ok(MemoryScope::task(required_scope_name(scope_name, "task")?)),
        "entity" => Ok(MemoryScope::entity(required_scope_name(scope_name, "entity")?)),
        "repository" | "repo" => {
            let name = required_scope_name(scope_name, "repository")?;
            if looks_like_remote_url(name) {
                Ok(MemoryScope::repository(Some(name.to_string()), None))
            } else {
                Ok(MemoryScope::repository(None, Some(name.to_string())))
            }
        }
        "session" => {
            let name = required_scope_name(scope_name, "session")?;
            let session_id = Id::parse(name).map_err(|error| {
                format!("accepted candidate has invalid session scope id `{name}`: {error}")
            })?;
            Ok(MemoryScope::Session { session_id })
        }
        "custom" => Ok(MemoryScope::Custom {
            name: required_scope_name(scope_name, "custom")?.to_string(),
        }),
        other => Err(format!(
            "accepted candidate has unknown scope_type `{other}`; expected global, user, project, task, entity, repository, session, or custom"
        )),
    }
}

fn required_scope_name<'a>(
    scope_name: Option<&'a str>,
    scope_type: &str,
) -> Result<&'a str, String> {
    scope_name
        .ok_or_else(|| format!("accepted candidate with {scope_type} scope is missing scope_name"))
}

fn looks_like_remote_url(value: &str) -> bool {
    value.starts_with("http://")
        || value.starts_with("https://")
        || value.starts_with("ssh://")
        || value.starts_with("git@")
}

fn digest_extraction_candidate_tag(
    parsed: &ParsedDigestExtractionCandidate,
    relative_path: &str,
) -> String {
    let source = parsed
        .source_review_path
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("unknown-source-review");
    format!("digest-extraction-candidate:{source}:{relative_path}")
}

fn markdown_section(contents: &str, heading: &str) -> Option<String> {
    let heading_start = contents.find(heading)?;
    let after_heading = &contents[heading_start + heading.len()..];
    let section = after_heading.strip_prefix('\n').unwrap_or(after_heading);
    let end = section
        .find("\n## ")
        .or_else(|| section.find("\n# "))
        .unwrap_or(section.len());
    Some(section[..end].trim().to_string())
}

fn candidate_memory_heading(contents: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        line.trim()
            .strip_prefix("# Candidate Memory:")
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

fn source_evidence_value(contents: &str, label: &str) -> Option<String> {
    markdown_section(contents, "## Source Evidence")
        .and_then(|section| bullet_value(&section, label))
}

fn bullet_value(contents: &str, label: &str) -> Option<String> {
    let prefix = format!("- {label}:");
    contents.lines().find_map(|line| {
        let line = line.trim();
        let value = line.strip_prefix(&prefix)?;
        clean_markdown_value(value)
    })
}

fn clean_markdown_value(value: &str) -> Option<String> {
    let value = value.trim();
    let value = value
        .strip_prefix('`')
        .and_then(|rest| rest.strip_suffix('`'))
        .unwrap_or(value)
        .trim();
    if value.is_empty() || value.eq_ignore_ascii_case("null") {
        None
    } else {
        Some(value.to_string())
    }
}

fn digest_machine_record_json(contents: &str) -> Option<&str> {
    let heading_start = contents.find(DIGEST_MACHINE_RECORD_HEADING)?;
    let after_heading = &contents[heading_start + DIGEST_MACHINE_RECORD_HEADING.len()..];
    let fence_start = after_heading.find(DIGEST_MACHINE_RECORD_FENCE)?;
    let after_fence = &after_heading[fence_start + DIGEST_MACHINE_RECORD_FENCE.len()..];
    let json_start = after_fence.strip_prefix('\n').unwrap_or(after_fence);
    let fence_end = json_start.find("```")?;
    Some(json_start[..fence_end].trim())
}

fn normalize_extraction_options(mut options: DigestExtractionOptions) -> DigestExtractionOptions {
    if options.max_source_bytes == 0 {
        options.max_source_bytes = DEFAULT_EXTRACTION_MAX_SOURCE_BYTES;
    }
    if options.max_candidates_per_source == 0 {
        options.max_candidates_per_source = DEFAULT_EXTRACTION_MAX_CANDIDATES_PER_SOURCE;
    }
    if options.max_candidate_chars < MIN_EXTRACTION_CANDIDATE_CHARS {
        options.max_candidate_chars = DEFAULT_EXTRACTION_MAX_CANDIDATE_CHARS;
    }
    options
}

fn normalize_source_index_options(
    mut options: DigestSourceIndexOptions,
) -> DigestSourceIndexOptions {
    if options.max_source_bytes == 0 {
        options.max_source_bytes = DEFAULT_SOURCE_INDEX_MAX_SOURCE_BYTES;
    }
    options
}

fn digest_source_to_text(raw: &str, format: DigestFileFormat) -> String {
    match format {
        DigestFileFormat::Markdown => normalize_text(raw),
        DigestFileFormat::Html => normalize_text(&html_to_text(raw)),
    }
}

fn html_to_text(raw: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    let mut tag = String::new();
    for ch in raw.chars() {
        match ch {
            '<' => {
                in_tag = true;
                tag.clear();
            }
            '>' if in_tag => {
                in_tag = false;
                let lower = tag.trim().to_lowercase();
                if lower.starts_with("br")
                    || lower.starts_with("/p")
                    || lower.starts_with("/div")
                    || lower.starts_with("/li")
                    || lower.starts_with("/h")
                {
                    text.push('\n');
                } else {
                    text.push(' ');
                }
            }
            _ if in_tag => tag.push(ch),
            _ => text.push(ch),
        }
    }
    decode_basic_html_entities(&text)
}

fn decode_basic_html_entities(value: &str) -> String {
    value
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

fn normalize_text(value: &str) -> String {
    value
        .lines()
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join("\n")
}

fn extraction_excerpts(text: &str, limit: usize, max_chars: usize) -> Vec<String> {
    let mut excerpts = Vec::new();
    for paragraph in digest_paragraphs(text) {
        for excerpt in split_excerpt(&paragraph, max_chars) {
            if excerpt.chars().count() >= MIN_EXTRACTION_CANDIDATE_CHARS {
                excerpts.push(excerpt);
            }
            if excerpts.len() >= limit {
                return excerpts;
            }
        }
    }
    excerpts
}

fn digest_paragraphs(text: &str) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let mut current = Vec::new();
    for line in text.lines() {
        let line = clean_digest_line(line);
        if line.is_empty() {
            flush_paragraph(&mut paragraphs, &mut current);
        } else {
            current.push(line);
        }
    }
    flush_paragraph(&mut paragraphs, &mut current);
    paragraphs
}

fn flush_paragraph(paragraphs: &mut Vec<String>, current: &mut Vec<String>) {
    if current.is_empty() {
        return;
    }
    let paragraph = current.join("\n");
    current.clear();
    if paragraph.chars().count() >= MIN_EXTRACTION_CANDIDATE_CHARS {
        paragraphs.push(paragraph);
    }
}

fn clean_digest_line(line: &str) -> String {
    let trimmed = line.trim();
    let without_heading = trimmed.trim_start_matches('#').trim();
    strip_bullet_marker(without_heading).trim().to_string()
}

fn strip_bullet_marker(value: &str) -> &str {
    let trimmed = value.trim_start();
    for marker in ["- ", "* ", "+ ", "> "] {
        if let Some(rest) = trimmed.strip_prefix(marker) {
            return rest;
        }
    }

    let Some((number, rest)) = trimmed.split_once(". ") else {
        return trimmed;
    };
    if !number.is_empty() && number.chars().all(|ch| ch.is_ascii_digit()) {
        rest
    } else {
        trimmed
    }
}

fn split_excerpt(value: &str, max_chars: usize) -> Vec<String> {
    if value.chars().count() <= max_chars {
        return vec![value.to_string()];
    }

    let mut excerpts = Vec::new();
    let mut current = String::new();
    for line in value.lines() {
        let line_len = line.chars().count();
        let current_len = current.chars().count();
        if !current.is_empty() && current_len + line_len + 1 > max_chars {
            excerpts.push(current.trim().to_string());
            current.clear();
        }
        if line_len > max_chars {
            excerpts.extend(split_long_text(line, max_chars));
        } else {
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(line);
        }
    }
    if !current.trim().is_empty() {
        excerpts.push(current.trim().to_string());
    }
    excerpts
}

fn split_long_text(value: &str, max_chars: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    for word in value.split_whitespace() {
        let projected = current.chars().count() + word.chars().count() + 1;
        if !current.is_empty() && projected > max_chars {
            chunks.push(current.trim().to_string());
            current.clear();
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.trim().is_empty() {
        chunks.push(current.trim().to_string());
    }
    chunks
}

fn extraction_candidate_title(source: &DigestReviewedSource, number: usize) -> String {
    if let Some(title) = source
        .title
        .as_ref()
        .filter(|title| !title.trim().is_empty())
    {
        if number == 1 {
            return title.clone();
        }
        return format!("{title} #{number}");
    }

    let date = source
        .candidate
        .date_hint
        .as_deref()
        .map(|date| format!(" {date}"))
        .unwrap_or_default();
    format!(
        "{} digest candidate{} #{}",
        source.candidate.source_kind, date, number
    )
}

fn optional_yaml_string(value: Option<&str>) -> String {
    value.map(yaml_string).unwrap_or_else(|| "null".to_string())
}

fn scan_directory(
    root: &Path,
    current: &Path,
    include_operational: bool,
    files_scanned: &mut usize,
    candidates: &mut Vec<DigestSourceCandidate>,
    exclusions: &mut Vec<DigestExcludedPath>,
) -> IndexResult<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if file_type.is_dir() {
            if !include_operational && is_operational_directory(&name) {
                if path_has_digest_component(root, &path) {
                    exclusions.push(exclusion(root, &path, "operational directory"));
                }
                continue;
            }
            scan_directory(
                root,
                &path,
                include_operational,
                files_scanned,
                candidates,
                exclusions,
            )?;
        } else if file_type.is_file() && path_has_digest_component(root, &path) {
            *files_scanned += 1;
            match classify_file(root, &path, include_operational)? {
                FileClassification::Candidate(candidate) => candidates.push(candidate),
                FileClassification::Excluded(excluded) => exclusions.push(excluded),
            }
        }
    }
    Ok(())
}

enum FileClassification {
    Candidate(DigestSourceCandidate),
    Excluded(DigestExcludedPath),
}

fn classify_file(
    root: &Path,
    path: &Path,
    include_operational: bool,
) -> IndexResult<FileClassification> {
    let relative_path = relative_path(root, path);
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string();

    if !include_operational && is_operational_file(&file_name) {
        return Ok(FileClassification::Excluded(exclusion(
            root,
            path,
            "operational artifact",
        )));
    }

    let Some(format) = digest_file_format(path) else {
        return Ok(FileClassification::Excluded(exclusion(
            root,
            path,
            "unsupported digest-adjacent file format",
        )));
    };

    let metadata = fs::metadata(path)?;
    let modified_at = metadata.modified().ok().map(OffsetDateTime::from);
    let collection = digest_collection(root, path).unwrap_or_else(|| "digest".to_string());
    let source_kind = source_kind_from_collection(&collection);
    let bucket = digest_bucket(root, path, &collection);
    let sensitivity = sensitivity_for(source_kind);
    let proposed_action = match sensitivity {
        DigestSensitivity::SensitiveCommunication => DigestProposedAction::ReviewGatedExtraction,
        DigestSensitivity::PersonalNotes => DigestProposedAction::IndexAsSource,
    };
    let mut reasons = vec![
        "Path is under a digest-named collection.".to_string(),
        "Supported source format for future safe parsing.".to_string(),
        "Inventory does not read file contents.".to_string(),
    ];
    if sensitivity == DigestSensitivity::SensitiveCommunication {
        reasons.push(
            "Communication digests must remain review-gated before becoming active memory."
                .to_string(),
        );
    }

    Ok(FileClassification::Candidate(DigestSourceCandidate {
        source_kind,
        collection,
        bucket,
        format,
        relative_path: relative_path.clone(),
        absolute_path: path.display().to_string(),
        file_name,
        date_hint: extract_date_hint(&relative_path),
        size_bytes: metadata.len(),
        modified_at,
        sensitivity,
        proposed_action,
        reasons,
    }))
}

fn digest_file_format(path: &Path) -> Option<DigestFileFormat> {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_lowercase())
        .as_deref()
    {
        Some("md") | Some("markdown") => Some(DigestFileFormat::Markdown),
        Some("html") | Some("htm") => Some(DigestFileFormat::Html),
        _ => None,
    }
}

fn path_has_digest_component(root: &Path, path: &Path) -> bool {
    if root_digest_collection(root).is_some() {
        return true;
    }

    path.strip_prefix(root)
        .ok()
        .into_iter()
        .flat_map(Path::components)
        .any(|component| {
            component
                .as_os_str()
                .to_string_lossy()
                .to_lowercase()
                .contains("digest")
        })
}

fn digest_collection(root: &Path, path: &Path) -> Option<String> {
    let relative_collection = path
        .strip_prefix(root)
        .ok()?
        .components()
        .find_map(|component| {
            let value = component.as_os_str().to_string_lossy();
            if value.to_lowercase().contains("digest") {
                Some(value.to_string())
            } else {
                None
            }
        });

    relative_collection.or_else(|| root_digest_collection(root))
}

fn root_digest_collection(root: &Path) -> Option<String> {
    let name = root.file_name()?.to_string_lossy();
    if name.to_lowercase().contains("digest") {
        Some(name.to_string())
    } else {
        None
    }
}

fn digest_bucket(root: &Path, path: &Path, collection: &str) -> Option<String> {
    let relative = path.strip_prefix(root).ok()?;
    if root_digest_collection(root).as_deref() == Some(collection) {
        let parent = relative.parent()?;
        if parent.as_os_str().is_empty() {
            return None;
        }
        return Some(
            parent
                .components()
                .map(|component| component.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/"),
        );
    }

    let mut after_collection = false;
    let mut components = Vec::new();
    for component in relative.components() {
        let value = component.as_os_str().to_string_lossy();
        if !after_collection {
            after_collection = value == collection;
            continue;
        }
        components.push(value.to_string());
    }

    components.pop();
    if components.is_empty() {
        None
    } else {
        Some(components.join("/"))
    }
}

fn source_kind_from_collection(collection: &str) -> DigestSourceKind {
    let lower = collection.to_lowercase();
    if lower.contains("slack") {
        DigestSourceKind::Slack
    } else if lower.contains("mail") || lower.contains("email") {
        DigestSourceKind::Email
    } else if lower.contains("swe") {
        DigestSourceKind::Swe
    } else if lower.contains("ai") {
        DigestSourceKind::Ai
    } else if lower.contains("note") {
        DigestSourceKind::Notes
    } else {
        DigestSourceKind::Unknown
    }
}

fn sensitivity_for(source_kind: DigestSourceKind) -> DigestSensitivity {
    match source_kind {
        DigestSourceKind::Slack | DigestSourceKind::Email => {
            DigestSensitivity::SensitiveCommunication
        }
        DigestSourceKind::Ai
        | DigestSourceKind::Swe
        | DigestSourceKind::Notes
        | DigestSourceKind::Unknown => DigestSensitivity::PersonalNotes,
    }
}

fn is_operational_directory(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.starts_with('.') || lower == "node_modules" || lower == "target"
}

fn is_operational_file(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.starts_with('.')
        || lower == "_seen.json"
        || lower == "_queue.json"
        || lower == "skill.md"
        || lower == "settings.local.json"
        || lower.ends_with(".log")
        || lower.ends_with(".yml")
        || lower.ends_with(".yaml")
        || lower.ends_with(".json")
}

fn exclusion(root: &Path, path: &Path, reason: impl Into<String>) -> DigestExcludedPath {
    DigestExcludedPath {
        relative_path: relative_path(root, path),
        absolute_path: path.display().to_string(),
        reason: reason.into(),
    }
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn path_to_markdown(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut previous_dash = false;
    for ch in value.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            slug.push(ch);
            previous_dash = false;
        } else if !previous_dash {
            slug.push('-');
            previous_dash = true;
        }
        if slug.len() >= 80 {
            break;
        }
    }

    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "untitled".to_string()
    } else {
        slug
    }
}

fn yaml_string(value: &str) -> String {
    serde_json::to_string(value).expect("string serialization cannot fail")
}

fn escape_link_text(value: &str) -> String {
    value.replace('\\', "\\\\").replace(']', "\\]")
}

fn extract_date_hint(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut dates = Vec::new();
    for idx in 0..bytes.len().saturating_sub(9) {
        let candidate = &value[idx..idx + 10];
        if is_iso_date(candidate) {
            dates.push(candidate.to_string());
        }
    }

    match dates.as_slice() {
        [] => None,
        [one] => Some(one.clone()),
        [start, end, ..] => Some(format!("{start}..{end}")),
    }
}

fn is_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[..4].iter().all(u8::is_ascii_digit)
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[8..].iter().all(u8::is_ascii_digit)
}

fn count_by(items: impl Iterator<Item = String>) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for key in items {
        *counts.entry(key).or_insert(0) += 1;
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn inventory_classifies_digest_sources_without_reading_contents() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("slack-digest/morning")).unwrap();
        fs::create_dir_all(dir.path().join("mail-digest")).unwrap();
        fs::create_dir_all(dir.path().join("ai-digest/digests")).unwrap();
        fs::write(
            dir.path().join("slack-digest/morning/2026-04-26.md"),
            "private slack content",
        )
        .unwrap();
        fs::write(
            dir.path().join("mail-digest/digest-2026-04-26.html"),
            "<html>private mail content</html>",
        )
        .unwrap();
        fs::write(
            dir.path().join("ai-digest/digests/2026-04-26.md"),
            "AI digest",
        )
        .unwrap();
        fs::write(dir.path().join("ai-digest/_seen.json"), "{}").unwrap();
        fs::write(dir.path().join("ai-digest/SKILL.md"), "# skill").unwrap();

        let inventory = DigestService::new()
            .inventory(DigestInventoryOptions::new(dir.path()))
            .unwrap();

        assert_eq!(inventory.files_scanned, 5);
        assert_eq!(inventory.total_candidates, 3);
        assert_eq!(inventory.excluded_count, 2);
        assert_eq!(inventory.by_source_kind["slack"], 1);
        assert_eq!(inventory.by_source_kind["email"], 1);
        assert_eq!(inventory.by_source_kind["ai"], 1);
        assert!(inventory
            .candidates
            .iter()
            .any(|candidate| candidate.bucket.as_deref() == Some("morning")));
        assert!(inventory
            .exclusions
            .iter()
            .any(|excluded| excluded.relative_path.ends_with("_seen.json")));
        assert!(inventory
            .warnings
            .iter()
            .any(|warning| warning.contains("no digest contents were read")));
    }

    #[test]
    fn inventory_limit_truncates_candidates_after_counting() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("notes-digest")).unwrap();
        fs::write(dir.path().join("notes-digest/digest-2026-04-25.md"), "one").unwrap();
        fs::write(dir.path().join("notes-digest/digest-2026-04-26.md"), "two").unwrap();

        let mut options = DigestInventoryOptions::new(dir.path());
        options.limit = Some(1);
        let inventory = DigestService::new().inventory(options).unwrap();

        assert_eq!(inventory.total_candidates, 2);
        assert_eq!(inventory.returned_candidates, 1);
        assert!(inventory.truncated);
    }

    #[test]
    fn inventory_accepts_digest_directory_as_root() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("slack-digest");
        fs::create_dir_all(root.join("evening")).unwrap();
        fs::write(root.join("evening/2026-04-26.md"), "digest").unwrap();

        let inventory = DigestService::new()
            .inventory(DigestInventoryOptions::new(&root))
            .unwrap();

        assert_eq!(inventory.total_candidates, 1);
        assert_eq!(inventory.candidates[0].collection, "slack-digest");
        assert_eq!(inventory.candidates[0].bucket.as_deref(), Some("evening"));
    }

    #[test]
    fn inventory_prefers_child_collection_over_digest_named_root() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("digest-smoke-root");
        fs::create_dir_all(root.join("slack-digest/morning")).unwrap();
        fs::write(root.join("slack-digest/morning/2026-04-26.md"), "digest").unwrap();

        let inventory = DigestService::new()
            .inventory(DigestInventoryOptions::new(&root))
            .unwrap();

        assert_eq!(inventory.total_candidates, 1);
        assert_eq!(inventory.candidates[0].collection, "slack-digest");
        assert_eq!(inventory.candidates[0].source_kind, DigestSourceKind::Slack);
        assert_eq!(inventory.candidates[0].bucket.as_deref(), Some("morning"));
    }

    #[test]
    fn review_export_writes_index_and_candidate_pages_without_contents() {
        let dir = tempdir().unwrap();
        let output = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("slack-digest/morning")).unwrap();
        fs::create_dir_all(dir.path().join("mail-digest")).unwrap();
        fs::write(
            dir.path().join("slack-digest/morning/2026-04-26.md"),
            "private slack content",
        )
        .unwrap();
        fs::write(
            dir.path().join("mail-digest/digest-2026-04-26.html"),
            "<html>private mail content</html>",
        )
        .unwrap();
        fs::write(dir.path().join("mail-digest/_queue.json"), "{}").unwrap();

        let export = DigestService::new()
            .export_review_batch(output.path(), DigestInventoryOptions::new(dir.path()))
            .unwrap();

        assert_eq!(export.inventory.total_candidates, 2);
        assert!(export.files_written.iter().any(|path| path == "index.md"));
        assert!(export
            .files_written
            .iter()
            .any(|path| path.starts_with("candidates/")));

        let index = fs::read_to_string(output.path().join("index.md")).unwrap();
        assert!(index.contains("# Digest Source Review Batch"));
        assert!(index.contains(DIGEST_REVIEW_MARKER));
        assert!(!index.contains("private slack content"));
        assert!(!index.contains("private mail content"));

        let candidate_path = export
            .files_written
            .iter()
            .find(|path| path.starts_with("candidates/"))
            .expect("candidate review page should be written");
        let candidate = fs::read_to_string(output.path().join(candidate_path)).unwrap();
        assert!(candidate.contains("decision: pending"));
        assert!(candidate.contains("This generated page intentionally omits digest contents."));
        assert!(candidate.contains("## Machine Record"));
        assert!(!candidate.contains("private slack content"));
        assert!(!candidate.contains("private mail content"));
    }

    #[test]
    fn review_export_skips_user_owned_files() {
        let dir = tempdir().unwrap();
        let output = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("notes-digest")).unwrap();
        fs::write(
            dir.path().join("notes-digest/digest-2026-04-26.md"),
            "private notes content",
        )
        .unwrap();
        fs::write(
            output.path().join("index.md"),
            "# Human review notes\n\nDo not replace this file.\n",
        )
        .unwrap();

        let export = DigestService::new()
            .export_review_batch(output.path(), DigestInventoryOptions::new(dir.path()))
            .unwrap();

        assert!(export.files_skipped.iter().any(|path| path == "index.md"));
        let index = fs::read_to_string(output.path().join("index.md")).unwrap();
        assert!(index.contains("Do not replace this file."));
        assert!(!index.contains(DIGEST_REVIEW_MARKER));
    }

    #[test]
    fn review_apply_parses_decisions_without_reading_digest_contents() {
        let dir = tempdir().unwrap();
        let output = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("slack-digest/morning")).unwrap();
        fs::create_dir_all(dir.path().join("notes-digest")).unwrap();
        fs::write(
            dir.path().join("slack-digest/morning/2026-04-26.md"),
            "private slack content",
        )
        .unwrap();
        fs::write(
            dir.path().join("notes-digest/digest-2026-04-26.md"),
            "private notes content",
        )
        .unwrap();

        let export = DigestService::new()
            .export_review_batch(output.path(), DigestInventoryOptions::new(dir.path()))
            .unwrap();
        edit_review_decision(
            output.path(),
            &export,
            0,
            "accept",
            &[
                ("memory_kind", "project_fact"),
                ("title", "\"Useful digest\""),
            ],
        );
        edit_review_decision(output.path(), &export, 1, "source_only", &[]);

        let apply = DigestService::new()
            .apply_review_batch(output.path())
            .unwrap();

        assert_eq!(apply.files_scanned, 2);
        assert_eq!(apply.accepted_count, 1);
        assert_eq!(apply.source_only_count, 1);
        assert_eq!(apply.planned_count(), 2);
        assert_eq!(
            apply.planned_sources[0].memory_kind.as_deref(),
            Some("project_fact")
        );
        assert_eq!(
            apply.planned_sources[0].title.as_deref(),
            Some("Useful digest")
        );
        let serialized = serde_json::to_string(&apply).unwrap();
        assert!(!serialized.contains("private slack content"));
        assert!(!serialized.contains("private notes content"));
    }

    #[test]
    fn review_apply_reports_pending_invalid_and_non_generated_files() {
        let dir = tempdir().unwrap();
        let output = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("mail-digest")).unwrap();
        fs::create_dir_all(dir.path().join("swe-digest")).unwrap();
        fs::write(
            dir.path().join("mail-digest/digest-2026-04-26.html"),
            "<html>private mail content</html>",
        )
        .unwrap();
        fs::write(
            dir.path().join("swe-digest/digest-2026-04-26.md"),
            "private swe content",
        )
        .unwrap();

        let export = DigestService::new()
            .export_review_batch(output.path(), DigestInventoryOptions::new(dir.path()))
            .unwrap();
        edit_review_decision(output.path(), &export, 1, "maybe", &[]);
        fs::write(
            output.path().join("candidates/user-owned.md"),
            "# Human notes\n",
        )
        .unwrap();

        let apply = DigestService::new()
            .apply_review_batch(output.path())
            .unwrap();

        assert_eq!(apply.files_scanned, 3);
        assert_eq!(apply.planned_count(), 0);
        assert_eq!(apply.files_with_no_decision.len(), 1);
        assert_eq!(apply.files_with_invalid_decision.len(), 1);
        assert_eq!(apply.files_skipped, vec!["candidates/user-owned.md"]);
    }

    #[test]
    fn review_apply_rejects_missing_root() {
        let dir = tempdir().unwrap();
        let err = DigestService::new()
            .apply_review_batch(dir.path().join("missing"))
            .unwrap_err();

        assert!(err.to_string().contains("file not found"));
    }

    #[test]
    fn extraction_plan_reads_only_accepted_sources_and_writes_review_candidates() {
        let dir = tempdir().unwrap();
        let review = tempdir().unwrap();
        let output = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("slack-digest/morning")).unwrap();
        fs::create_dir_all(dir.path().join("notes-digest")).unwrap();
        fs::write(
            dir.path().join("slack-digest/morning/2026-04-26.md"),
            "accepted source body with enough detail for candidate memory extraction",
        )
        .unwrap();
        fs::write(
            dir.path().join("notes-digest/digest-2026-04-26.md"),
            "source only body should not be copied into extraction output",
        )
        .unwrap();

        let export = DigestService::new()
            .export_review_batch(review.path(), DigestInventoryOptions::new(dir.path()))
            .unwrap();
        edit_review_decision_by_source(review.path(), &export, "slack-digest", "accept", &[]);
        edit_review_decision_by_source(review.path(), &export, "notes-digest", "source_only", &[]);

        let plan = DigestService::new()
            .plan_extraction(
                review.path(),
                output.path(),
                DigestExtractionOptions::default(),
            )
            .unwrap();

        assert_eq!(plan.accepted_sources, 1);
        assert_eq!(plan.source_only_sources, 1);
        assert_eq!(plan.sources_read, 1);
        assert_eq!(plan.candidate_count(), 1);
        assert!(plan.files_written.iter().any(|path| path == "index.md"));
        let candidate =
            fs::read_to_string(output.path().join(&plan.candidates[0].review_path)).unwrap();
        assert!(candidate.contains(DIGEST_EXTRACTION_MARKER));
        assert!(candidate.contains("accepted source body"));
        assert!(!candidate.contains("source only body"));

        let serialized = serde_json::to_string(&plan).unwrap();
        assert!(!serialized.contains("accepted source body"));
        assert!(!serialized.contains("source only body"));
    }

    #[test]
    fn extraction_plan_skips_sources_above_size_limit() {
        let dir = tempdir().unwrap();
        let review = tempdir().unwrap();
        let output = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("slack-digest/morning")).unwrap();
        fs::write(
            dir.path().join("slack-digest/morning/2026-04-26.md"),
            "accepted private body that should stay unread when over the configured size limit",
        )
        .unwrap();

        let export = DigestService::new()
            .export_review_batch(review.path(), DigestInventoryOptions::new(dir.path()))
            .unwrap();
        edit_review_decision_by_source(review.path(), &export, "slack-digest", "accept", &[]);

        let plan = DigestService::new()
            .plan_extraction(
                review.path(),
                output.path(),
                DigestExtractionOptions {
                    max_source_bytes: 10,
                    ..DigestExtractionOptions::default()
                },
            )
            .unwrap();

        assert_eq!(plan.accepted_sources, 1);
        assert_eq!(plan.sources_read, 0);
        assert_eq!(plan.candidate_count(), 0);
        assert!(plan
            .sources_skipped
            .iter()
            .any(|skipped| skipped.contains("above max_source_bytes")));
        let output_text = fs::read_to_string(output.path().join("index.md")).unwrap();
        assert!(!output_text.contains("accepted private body"));
    }

    #[test]
    fn extraction_plan_skips_user_owned_output_files() {
        let dir = tempdir().unwrap();
        let review = tempdir().unwrap();
        let output = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("notes-digest")).unwrap();
        fs::write(
            dir.path().join("notes-digest/digest-2026-04-26.md"),
            "accepted source body with enough detail for candidate memory extraction",
        )
        .unwrap();
        fs::write(output.path().join("index.md"), "# Human extraction notes\n").unwrap();

        let export = DigestService::new()
            .export_review_batch(review.path(), DigestInventoryOptions::new(dir.path()))
            .unwrap();
        edit_review_decision_by_source(review.path(), &export, "notes-digest", "accept", &[]);

        let plan = DigestService::new()
            .plan_extraction(
                review.path(),
                output.path(),
                DigestExtractionOptions::default(),
            )
            .unwrap();

        assert!(plan.files_skipped.iter().any(|path| path == "index.md"));
        assert_eq!(
            fs::read_to_string(output.path().join("index.md")).unwrap(),
            "# Human extraction notes\n"
        );
        assert_eq!(plan.candidate_count(), 1);
    }

    #[test]
    fn source_index_plan_reads_only_source_only_sources_without_serializing_contents() {
        let dir = tempdir().unwrap();
        let review = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("slack-digest/morning")).unwrap();
        fs::create_dir_all(dir.path().join("notes-digest")).unwrap();
        fs::write(
            dir.path().join("slack-digest/morning/2026-04-26.md"),
            "accepted source body should not be indexed by source_only indexing",
        )
        .unwrap();
        fs::write(
            dir.path().join("notes-digest/digest-2026-04-26.md"),
            "source only body should be prepared for document evidence indexing",
        )
        .unwrap();

        let export = DigestService::new()
            .export_review_batch(review.path(), DigestInventoryOptions::new(dir.path()))
            .unwrap();
        edit_review_decision_by_source(review.path(), &export, "slack-digest", "accept", &[]);
        edit_review_decision_by_source(
            review.path(),
            &export,
            "notes-digest",
            "source_only",
            &[("title", "\"Reviewed digest source\"")],
        );

        let plan = DigestService::new()
            .plan_source_index(review.path(), DigestSourceIndexOptions::default())
            .unwrap();

        assert_eq!(plan.accepted_sources, 1);
        assert_eq!(plan.source_only_sources, 1);
        assert_eq!(plan.sources_read, 1);
        assert_eq!(plan.document_count(), 1);
        assert_eq!(plan.documents[0].title, "Reviewed digest source");
        assert!(plan.documents[0]
            .indexed_content
            .contains("source only body should be prepared"));
        assert!(plan
            .warnings
            .iter()
            .any(|warning| warning.contains("reserved for extraction")));

        let serialized = serde_json::to_string(&plan).unwrap();
        assert!(!serialized.contains("source only body should be prepared"));
        assert!(!serialized.contains("accepted source body"));
    }

    #[test]
    fn extraction_review_apply_builds_active_memory_items_after_accept() {
        let dir = tempdir().unwrap();
        let review = tempdir().unwrap();
        let output = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("slack-digest/morning")).unwrap();
        fs::write(
            dir.path().join("slack-digest/morning/2026-04-26.md"),
            "accepted digest source with enough specific detail for a reviewed memory candidate",
        )
        .unwrap();

        let export = DigestService::new()
            .export_review_batch(review.path(), DigestInventoryOptions::new(dir.path()))
            .unwrap();
        edit_review_decision_by_source(
            review.path(),
            &export,
            "slack-digest",
            "accept",
            &[
                ("memory_kind", "project_fact"),
                ("scope_type", "project"),
                ("scope_name", "\"Engram\""),
                ("title", "\"Reviewed digest fact\""),
            ],
        );

        let plan = DigestService::new()
            .plan_extraction(
                review.path(),
                output.path(),
                DigestExtractionOptions::default(),
            )
            .unwrap();
        edit_extraction_decision(output.path(), &plan, 0, "accept", &[]);

        let apply = apply_digest_extraction_review_batch(
            output.path(),
            DigestExtractionReviewApplyOptions {
                dry_run: true,
                writer: test_writer(),
                create_commit: true,
            },
            HashSet::new(),
        )
        .unwrap();

        assert_eq!(apply.files_scanned, 1);
        assert_eq!(apply.accepted_count, 1);
        assert_eq!(apply.planned_count(), 1);
        assert_eq!(apply.written_count(), 0);
        let item = &apply.planned_items[0];
        assert_eq!(item.status, MemoryStatus::Active);
        assert_eq!(item.origin, ClaimOrigin::Imported);
        assert_eq!(item.kind, MemoryKind::ProjectFact);
        assert!(matches!(item.scope, MemoryScope::Project { .. }));
        assert!(item
            .tags
            .iter()
            .any(|tag| tag.starts_with("digest-extraction-candidate:")));
        assert!(item
            .evidence
            .iter()
            .any(|evidence| evidence.kind == EvidenceKind::ManualReview));
        assert!(item.content.contains("accepted digest source"));
    }

    #[test]
    fn extraction_review_apply_rejects_accept_without_kind_or_scope() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("candidates")).unwrap();
        fs::write(
            dir.path().join("candidates/0001-candidate-missing.md"),
            format!(
                "---\ngenerated_by: \"engram-memory-os\"\npage_type: \"digest_extraction_candidate\"\n---\n\n{DIGEST_EXTRACTION_MARKER}\n\n# Candidate Memory: Missing metadata\n\n## Candidate Content\n\nReviewed content with enough detail to pass the length threshold.\n\n## Review Decision\n\n```yaml\ndecision: accept\nmemory_kind: null\nscope_type: null\nscope_name: null\ntitle: \"Missing metadata\"\nnotes: null\n```\n"
            ),
        )
        .unwrap();

        let apply = apply_digest_extraction_review_batch(
            dir.path(),
            DigestExtractionReviewApplyOptions {
                dry_run: true,
                writer: test_writer(),
                create_commit: true,
            },
            HashSet::new(),
        )
        .unwrap();

        assert_eq!(apply.accepted_count, 0);
        assert_eq!(apply.planned_count(), 0);
        assert_eq!(
            apply.files_with_parse_errors,
            vec!["candidates/0001-candidate-missing.md"]
        );
        assert!(apply
            .warnings
            .iter()
            .any(|warning| warning.contains("missing memory_kind")));
    }

    #[test]
    fn review_export_rejects_missing_root_without_creating_output() {
        let dir = tempdir().unwrap();
        let output = dir.path().join("review-output");

        let err = DigestService::new()
            .export_review_batch(
                &output,
                DigestInventoryOptions::new(dir.path().join("missing")),
            )
            .unwrap_err();

        assert!(err.to_string().contains("file not found"));
        assert!(!output.exists());
    }

    #[test]
    fn inventory_rejects_missing_root() {
        let dir = tempdir().unwrap();
        let err = DigestService::new()
            .inventory(DigestInventoryOptions::new(dir.path().join("missing")))
            .unwrap_err();

        assert!(err.to_string().contains("file not found"));
    }

    #[test]
    fn date_hint_extracts_single_dates_and_ranges() {
        assert_eq!(
            extract_date_hint("slack-digest/morning/2026-04-26.md"),
            Some("2026-04-26".to_string())
        );
        assert_eq!(
            extract_date_hint("catchup/2026-02-26-to-2026-03-29.md"),
            Some("2026-02-26..2026-03-29".to_string())
        );
        assert_eq!(extract_date_hint("handoff.md"), None);
    }

    fn edit_review_decision(
        root: &Path,
        export: &DigestReviewExport,
        candidate_index: usize,
        decision: &str,
        fields: &[(&str, &str)],
    ) {
        let candidate_path = export
            .files_written
            .iter()
            .filter(|path| path.starts_with("candidates/"))
            .nth(candidate_index)
            .expect("candidate review page should exist");
        let path = root.join(candidate_path);
        let mut contents = fs::read_to_string(&path).unwrap();
        contents = contents.replace(
            "decision: pending # accept | reject | quarantine | source_only",
            &format!("decision: {decision} # accept | reject | quarantine | source_only"),
        );
        for (key, value) in fields {
            contents = replace_review_field(&contents, key, value);
        }
        fs::write(path, contents).unwrap();
    }

    fn edit_review_decision_by_source(
        root: &Path,
        export: &DigestReviewExport,
        source_fragment: &str,
        decision: &str,
        fields: &[(&str, &str)],
    ) {
        let candidate_path = export
            .files_written
            .iter()
            .filter(|path| path.starts_with("candidates/"))
            .find(|path| {
                fs::read_to_string(root.join(path))
                    .is_ok_and(|contents| contents.contains(source_fragment))
            })
            .expect("candidate review page for source should exist");
        let path = root.join(candidate_path);
        let mut contents = fs::read_to_string(&path).unwrap();
        contents = contents.replace(
            "decision: pending # accept | reject | quarantine | source_only",
            &format!("decision: {decision} # accept | reject | quarantine | source_only"),
        );
        for (key, value) in fields {
            contents = replace_review_field(&contents, key, value);
        }
        fs::write(path, contents).unwrap();
    }

    fn replace_review_field(contents: &str, key: &str, value: &str) -> String {
        contents
            .lines()
            .map(|line| {
                if line.starts_with(&format!("{key}:")) {
                    format!("{key}: {value}")
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn edit_extraction_decision(
        root: &Path,
        plan: &DigestExtractionPlan,
        candidate_index: usize,
        decision: &str,
        fields: &[(&str, &str)],
    ) {
        let candidate_path = &plan.candidates[candidate_index].review_path;
        let path = root.join(candidate_path);
        let mut contents = fs::read_to_string(&path).unwrap();
        contents = contents.replace(
            "decision: pending # accept | reject | quarantine",
            &format!("decision: {decision} # accept | reject | quarantine"),
        );
        for (key, value) in fields {
            contents = replace_review_field(&contents, key, value);
        }
        fs::write(path, contents).unwrap();
    }

    fn test_writer() -> WriterProvenance {
        WriterProvenance::agent(
            engram_core::memory::Harness::Codex,
            engram_core::memory::ModelIdentity::new("openai", "gpt-5.5"),
        )
    }
}
