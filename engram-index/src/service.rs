//! Document service - coordinates indexing and search.
//!
//! The service layer provides a high-level API for document operations,
//! combining the ingestion pipeline with the storage repository.

use crate::digest::{DigestService, DigestSourceIndexDocument, DigestSourceIndexOptions};
use crate::error::{IndexError, IndexResult};
use crate::pipeline::{DocumentIngestionPlan, IndexedDocument, Pipeline, PipelineConfig};
use engram_core::document::{DocChunk, DocSearchResult};
use engram_core::id::Id;
use engram_embed::Embedder;
use engram_store::repos::document::{normalize_for_fingerprint, stable_fingerprint};
use engram_store::{
    Db, DocumentOrphanChunkSample, DocumentOrphanDeleteResult, DocumentOrphanGroup,
    DocumentOrphanReport, DocumentRecoveryCandidateMatch, DocumentRecoveryClass, DocumentRepo,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

const DEFAULT_ORPHAN_GROUP_LIMIT: usize = 20;
const DEFAULT_ORPHAN_SAMPLE_LIMIT: usize = 3;
const DEFAULT_RECOVERY_MAX_CANDIDATE_FILES: usize = 5000;
const DEFAULT_RECOVERY_MAX_FILE_BYTES: usize = 1024 * 1024;
const DEFAULT_RECOVERY_MAX_MATCHES_PER_GROUP: usize = 5;
const DEFAULT_RECOVERY_MIN_MATCH_SCORE: f32 = 0.15;
const DEFAULT_QUARANTINE_REVIEW_MAX_CHUNK_BYTES: usize = 16 * 1024;
const DOCUMENT_ORPHAN_QUARANTINE_REVIEW_MARKER: &str =
    "<!-- engram:document-orphan-quarantine-review:v1 -->";

/// The document service for indexing and searching documents.
pub struct DocumentService {
    pipeline: Pipeline,
    repo: DocumentRepo,
    embedder: Embedder,
}

impl DocumentService {
    /// Create a new document service.
    pub fn new(db: Db, embedder: Embedder, config: PipelineConfig) -> Self {
        let pipeline = Pipeline::new(embedder.clone(), config);
        let repo = DocumentRepo::new(db);
        Self {
            pipeline,
            repo,
            embedder,
        }
    }

    /// Create a document service with default configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the embedder cannot be initialized.
    pub fn with_defaults(db: Db) -> IndexResult<Self> {
        let embedder = Embedder::default_model()?;
        Ok(Self::new(db, embedder, PipelineConfig::default()))
    }

    /// Initialize the document schema in the database.
    ///
    /// # Errors
    ///
    /// Returns an error if schema creation fails.
    pub async fn init_schema(&self) -> IndexResult<()> {
        self.repo.init_schema().await?;
        Ok(())
    }

    /// Index a file and store it in the database.
    ///
    /// # Errors
    ///
    /// Returns an error if indexing or storage fails.
    pub async fn index_file(&self, path: impl AsRef<Path>) -> IndexResult<IndexedDocument> {
        let path = path.as_ref();
        info!("Indexing file: {}", path.display());

        // Check if already indexed
        let path_str = path.display().to_string();
        if let Some(existing) = self.repo.find_source_by_path(&path_str).await? {
            if !existing.needs_reindex() {
                debug!("File already indexed and fresh: {}", path.display());
                // Return existing document info
                let chunks = self.repo.get_chunks_for_source(&existing.id).await?;
                let parsed = crate::parser::parse_file(path)?;
                return Ok(IndexedDocument {
                    source: existing,
                    parsed,
                    chunks: chunks
                        .into_iter()
                        .map(|chunk| crate::pipeline::IndexedChunk {
                            chunk,
                            embedding: Vec::new(), // Not loading embeddings for existing
                        })
                        .collect(),
                });
            }
            debug!("Re-indexing stale file: {}", path.display());
        }

        // Index the file
        let mut result = self.pipeline.index_file(path)?;

        // Mark as indexed and save
        result.source.mark_indexed();
        self.repo.save_source(&result.source).await?;

        // Save chunks with embeddings
        let chunks_with_embeddings: Vec<_> = result
            .chunks
            .iter()
            .map(|ic| (ic.chunk.clone(), ic.embedding.clone()))
            .collect();
        self.repo
            .save_chunks(&result.source.id, chunks_with_embeddings)
            .await?;

        info!(
            "Indexed {} with {} chunks",
            path.display(),
            result.chunks.len()
        );

        Ok(result)
    }

    /// Index caller-supplied markdown content and store it in the database.
    ///
    /// # Errors
    ///
    /// Returns an error if indexing or storage fails.
    pub async fn index_content(
        &self,
        path_or_url: impl Into<String>,
        title: Option<String>,
        content: impl Into<String>,
    ) -> IndexResult<IndexedDocument> {
        let path_or_url = path_or_url.into();
        info!("Indexing supplied content: {}", path_or_url);

        if let Some(existing) = self.repo.find_source_by_path(&path_or_url).await? {
            if !existing.needs_reindex() {
                debug!("Source already indexed and fresh: {}", path_or_url);
                let chunks = self.repo.get_chunks_for_source(&existing.id).await?;
                let mut parsed = crate::parser::parse_content(path_or_url, content.into())?;
                if let Some(title) = title.filter(|title| !title.trim().is_empty()) {
                    parsed.title = title;
                }
                return Ok(IndexedDocument {
                    source: existing,
                    parsed,
                    chunks: chunks
                        .into_iter()
                        .map(|chunk| crate::pipeline::IndexedChunk {
                            chunk,
                            embedding: Vec::new(),
                        })
                        .collect(),
                });
            }
            debug!("Re-indexing stale source: {}", path_or_url);
        }

        let mut result = self.pipeline.index_content(path_or_url, content, title)?;
        result.source.mark_indexed();
        self.repo.save_source(&result.source).await?;

        let chunks_with_embeddings: Vec<_> = result
            .chunks
            .iter()
            .map(|ic| (ic.chunk.clone(), ic.embedding.clone()))
            .collect();
        self.repo
            .save_chunks(&result.source.id, chunks_with_embeddings)
            .await?;

        info!(
            "Indexed supplied content with {} chunks",
            result.chunks.len()
        );
        Ok(result)
    }

    /// Index a directory and store all documents.
    ///
    /// # Errors
    ///
    /// Returns an error if indexing fails.
    pub async fn index_directory(
        &self,
        path: impl AsRef<Path>,
    ) -> IndexResult<Vec<IndexedDocument>> {
        let path = path.as_ref();
        info!("Indexing directory: {}", path.display());

        let results = self.pipeline.index_directory(path)?;

        // Save all to database
        for doc in &results {
            let mut source = doc.source.clone();
            source.mark_indexed();
            self.repo.save_source(&source).await?;

            let chunks_with_embeddings: Vec<_> = doc
                .chunks
                .iter()
                .map(|ic| (ic.chunk.clone(), ic.embedding.clone()))
                .collect();
            self.repo
                .save_chunks(&source.id, chunks_with_embeddings)
                .await?;
        }

        Ok(results)
    }

    /// Build a dry-run ingestion plan for a file or directory without writing to the store.
    ///
    /// # Errors
    ///
    /// Returns an error if the path cannot be read.
    pub async fn plan_path(&self, path: impl AsRef<Path>) -> IndexResult<DocumentIngestionPlan> {
        let path = path.as_ref();
        if path.is_dir() {
            self.pipeline.plan_directory(path)
        } else {
            Ok(DocumentIngestionPlan {
                documents: vec![self.pipeline.plan_file(path)?],
                warnings: Vec::new(),
            })
        }
    }

    /// Search for documents by semantic similarity.
    ///
    /// # Arguments
    ///
    /// * `query` - The search query text
    /// * `limit` - Maximum number of results to return
    ///
    /// # Errors
    ///
    /// Returns an error if search fails.
    pub async fn search(&self, query: &str, limit: usize) -> IndexResult<Vec<DocSearchResult>> {
        debug!("Searching for: {}", query);

        // Generate embedding for the query
        let query_embedding = self.embedder.embed(query)?;

        // Search in database
        let results = self.repo.search_similar(&query_embedding, limit).await?;

        info!("Found {} results for query", results.len());
        Ok(results)
    }

    /// Search with minimum score threshold.
    ///
    /// # Errors
    ///
    /// Returns an error if search fails.
    pub async fn search_threshold(
        &self,
        query: &str,
        limit: usize,
        min_score: f32,
    ) -> IndexResult<Vec<DocSearchResult>> {
        let results = self.search(query, limit).await?;
        Ok(results
            .into_iter()
            .filter(|r| r.score >= min_score)
            .collect())
    }

    /// Get statistics about indexed documents.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn stats(&self) -> IndexResult<DocumentStats> {
        let db_stats = self.repo.stats().await?;
        Ok(DocumentStats {
            source_count: db_stats.source_count,
            chunk_count: db_stats.chunk_count,
            searchable_chunk_count: db_stats.searchable_chunk_count,
            orphan_chunk_count: db_stats.orphan_chunk_count,
            embedding_dimension: self.pipeline.embedding_dimension(),
        })
    }

    /// Build a read-only recovery report for orphan document chunks.
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails.
    pub async fn orphan_report(
        &self,
        group_limit: usize,
        sample_limit_per_group: usize,
    ) -> IndexResult<DocumentOrphanReport> {
        self.orphan_recovery_report(DocumentRecoveryOptions {
            group_limit,
            sample_limit_per_group,
            ..Default::default()
        })
        .await
    }

    /// Build a read-only orphan recovery report, optionally fingerprinting candidate files.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query or candidate scan fails.
    pub async fn orphan_recovery_report(
        &self,
        options: DocumentRecoveryOptions,
    ) -> IndexResult<DocumentOrphanReport> {
        let mut report = self
            .repo
            .orphan_report(options.group_limit, options.sample_limit_per_group, 240)
            .await?;

        enrich_orphan_report_with_candidate_matches(&mut report, &options)?;
        Ok(report)
    }

    /// Build a read-only plan for future reindexing of recoverable orphan groups.
    ///
    /// # Errors
    ///
    /// Returns an error if the recovery report cannot be built.
    pub async fn orphan_reindex_plan(
        &self,
        options: DocumentRecoveryOptions,
    ) -> IndexResult<DocumentReindexPlan> {
        let report = self.orphan_recovery_report(options).await?;
        Ok(build_reindex_plan_from_report(&report))
    }

    /// Execute or dry-run a guarded source-level orphan reindex plan.
    ///
    /// The execution path never deletes orphan chunks. Write mode only reindexes the selected
    /// source actions; orphan cleanup must be handled by a separate explicit migration step.
    ///
    /// # Errors
    ///
    /// Returns an error if shared digest review planning fails. Per-source indexing failures are
    /// recorded in the returned report and do not abort the entire batch.
    pub async fn execute_orphan_reindex_plan(
        &self,
        plan: &DocumentReindexPlan,
        options: DocumentReindexExecutionOptions,
    ) -> IndexResult<DocumentReindexExecutionReport> {
        let mut report = DocumentReindexExecutionReport::new(plan, &options);
        let selected_indexes = selected_reindex_source_indexes(plan, &options);
        report.selected_source_actions = selected_indexes.len();

        let digest_selected = selected_indexes.iter().any(|index| {
            plan.sources[*index].action == DocumentReindexAction::ReindexDigestReviewedSource
        });
        let mut digest_documents = BTreeMap::new();
        if digest_selected {
            if options.digest_review_paths.is_empty() {
                report.warnings.push(
                    "digest reindex actions require digest_review_paths to resolve reviewed source content"
                        .to_string(),
                );
            } else {
                let (documents, warnings) = collect_digest_source_documents(&options)?;
                digest_documents = documents;
                report.warnings.extend(warnings);
            }
        }

        let selected_set = selected_indexes.into_iter().collect::<BTreeSet<_>>();
        let mut executable_seen = 0usize;

        for (index, source) in plan.sources.iter().enumerate() {
            if !selected_set.contains(&index) {
                report.actions.push(DocumentReindexExecutionAction::skipped(
                    source,
                    options.dry_run,
                    "source action not selected",
                ));
                report.skipped_source_actions += 1;
                continue;
            }

            if let Some(max_actions) = options.max_actions {
                if executable_seen >= max_actions {
                    report.actions.push(DocumentReindexExecutionAction::skipped(
                        source,
                        options.dry_run,
                        "max_actions limit reached",
                    ));
                    report.skipped_source_actions += 1;
                    continue;
                }
            }
            executable_seen += 1;

            let action = match source.action {
                DocumentReindexAction::ReindexFile => {
                    self.execute_file_reindex_action(source, &options).await
                }
                DocumentReindexAction::ReindexDigestReviewedSource => {
                    self.execute_digest_reindex_action(source, &options, &digest_documents)
                        .await
                }
                DocumentReindexAction::InspectExistingSource => {
                    DocumentReindexExecutionAction::requires_inspection(source, options.dry_run)
                }
            };

            match action.status {
                DocumentReindexExecutionStatus::Planned => {
                    report.planned_source_actions += 1;
                    report.planned_chunks += action.chunk_count.unwrap_or(0);
                }
                DocumentReindexExecutionStatus::Reindexed => {
                    report.reindexed_source_actions += 1;
                    report.reindexed_documents += 1;
                    report.indexed_chunks += action.chunk_count.unwrap_or(0);
                }
                DocumentReindexExecutionStatus::AlreadyIndexed => {
                    report.already_indexed_source_actions += 1;
                    report.indexed_chunks += action.chunk_count.unwrap_or(0);
                }
                DocumentReindexExecutionStatus::RequiresInspection => {
                    report.inspection_source_actions += 1;
                }
                DocumentReindexExecutionStatus::Skipped => {
                    report.skipped_source_actions += 1;
                }
                DocumentReindexExecutionStatus::Failed => {
                    report.failed_source_actions += 1;
                }
            }
            report.actions.push(action);
        }

        Ok(report)
    }

    /// Build a read-only cleanup/quarantine plan for orphan chunks.
    ///
    /// Delete candidates are only recoverable groups that are covered by a successful write-mode
    /// reindex execution report. Safe-to-quarantine groups remain quarantine candidates; this
    /// method never deletes or archives chunks.
    ///
    /// # Errors
    ///
    /// Returns an error if the recovery report cannot be built.
    pub async fn orphan_cleanup_plan(
        &self,
        options: DocumentOrphanCleanupPlanOptions,
    ) -> IndexResult<DocumentOrphanCleanupPlan> {
        let report = self.orphan_recovery_report(options.recovery).await?;
        Ok(build_orphan_cleanup_plan(
            &report,
            options.reindex_plan.as_ref(),
            options.execution_report.as_ref(),
        ))
    }

    /// Execute or dry-run deletion for cleanup-plan delete candidates.
    ///
    /// This executor can only delete groups marked `delete_after_successful_reindex`.
    /// Quarantine and manual-review groups are retained and reported.
    ///
    /// # Errors
    ///
    /// Returns an error if write mode is requested without explicit delete approval, or if the
    /// store deletion fails.
    pub async fn execute_orphan_cleanup_plan(
        &self,
        plan: &DocumentOrphanCleanupPlan,
        options: DocumentOrphanCleanupExecutionOptions,
    ) -> IndexResult<DocumentOrphanCleanupExecutionReport> {
        if !options.dry_run && !options.approve_delete_candidates {
            return Err(IndexError::InvalidState(
                "write mode requires approve_delete_candidates=true".to_string(),
            ));
        }

        let mut report = DocumentOrphanCleanupExecutionReport::new(plan, &options);
        let selected_delete_ids = selected_cleanup_delete_ids(plan, &options);
        report.selected_delete_groups = selected_delete_ids.len();

        if options.dry_run {
            for group in &plan.groups {
                report.actions.push(cleanup_execution_action_for_dry_run(
                    group,
                    &selected_delete_ids,
                ));
            }
            refresh_cleanup_execution_summary(&mut report);
            return Ok(report);
        }

        let delete_result = self
            .repo
            .delete_orphan_chunks_for_sources(&selected_delete_ids)
            .await?;
        apply_cleanup_delete_result(plan, &options, delete_result, &mut report);
        Ok(report)
    }

    /// Export a human-reviewable batch for orphan groups retained in quarantine.
    ///
    /// This is intentionally non-destructive: it reads retained orphan chunks and writes generated
    /// Markdown review pages. A later apply step must make migration or archive decisions
    /// explicitly from those reviewed pages.
    ///
    /// # Errors
    ///
    /// Returns an error if the output directory cannot be written or chunk lookup fails.
    pub async fn export_orphan_quarantine_review(
        &self,
        plan: &DocumentOrphanCleanupPlan,
        output_dir: impl AsRef<Path>,
        options: DocumentOrphanQuarantineReviewOptions,
    ) -> IndexResult<DocumentOrphanQuarantineReviewExport> {
        export_orphan_quarantine_review(&self.repo, plan, output_dir.as_ref(), options).await
    }

    /// Summarize decisions in a generated document orphan quarantine review batch.
    ///
    /// # Errors
    ///
    /// Returns an error if the review directory cannot be read.
    pub fn orphan_quarantine_review_status(
        &self,
        review_dir: impl AsRef<Path>,
    ) -> IndexResult<DocumentOrphanQuarantineReviewStatus> {
        Self::orphan_quarantine_review_status_for_dir(review_dir)
    }

    /// Summarize decisions in a generated document orphan quarantine review batch without
    /// requiring a document database handle.
    ///
    /// # Errors
    ///
    /// Returns an error if the review directory cannot be read.
    pub fn orphan_quarantine_review_status_for_dir(
        review_dir: impl AsRef<Path>,
    ) -> IndexResult<DocumentOrphanQuarantineReviewStatus> {
        orphan_quarantine_review_status(review_dir.as_ref())
    }

    /// Prioritize pending pages in a generated document orphan quarantine review batch.
    ///
    /// This is read-only. It ranks review pages with deterministic, transparent signals so a
    /// human can choose a small pilot set before any future write-mode apply step exists.
    ///
    /// # Errors
    ///
    /// Returns an error if the review directory cannot be read.
    pub fn prioritize_orphan_quarantine_review(
        &self,
        review_dir: impl AsRef<Path>,
        options: DocumentOrphanQuarantineReviewPrioritizationOptions,
    ) -> IndexResult<DocumentOrphanQuarantineReviewPrioritization> {
        Self::prioritize_orphan_quarantine_review_for_dir(review_dir, options)
    }

    /// Prioritize pending pages in a generated document orphan quarantine review batch without
    /// requiring a document database handle.
    ///
    /// # Errors
    ///
    /// Returns an error if the review directory cannot be read.
    pub fn prioritize_orphan_quarantine_review_for_dir(
        review_dir: impl AsRef<Path>,
        options: DocumentOrphanQuarantineReviewPrioritizationOptions,
    ) -> IndexResult<DocumentOrphanQuarantineReviewPrioritization> {
        prioritize_orphan_quarantine_review(review_dir.as_ref(), options)
    }

    /// Dry-run the actions implied by a generated document orphan quarantine review batch.
    ///
    /// This method currently supports dry-run only. Write-mode migration/archive/delete behavior is
    /// intentionally left for a later guarded step after status validation is in place.
    ///
    /// # Errors
    ///
    /// Returns an error if write mode is requested or the review directory cannot be read.
    pub fn apply_orphan_quarantine_review(
        &self,
        review_dir: impl AsRef<Path>,
        options: DocumentOrphanQuarantineReviewApplyOptions,
    ) -> IndexResult<DocumentOrphanQuarantineReviewApply> {
        Self::apply_orphan_quarantine_review_for_dir(review_dir, options)
    }

    /// Dry-run the actions implied by a generated document orphan quarantine review batch without
    /// requiring a document database handle.
    ///
    /// # Errors
    ///
    /// Returns an error if write mode is requested or the review directory cannot be read.
    pub fn apply_orphan_quarantine_review_for_dir(
        review_dir: impl AsRef<Path>,
        options: DocumentOrphanQuarantineReviewApplyOptions,
    ) -> IndexResult<DocumentOrphanQuarantineReviewApply> {
        if !options.dry_run {
            return Err(IndexError::InvalidState(
                "document orphan quarantine review apply currently supports dry-run only"
                    .to_string(),
            ));
        }
        apply_orphan_quarantine_review(review_dir.as_ref(), options)
    }

    async fn execute_file_reindex_action(
        &self,
        source: &DocumentReindexSourcePlan,
        options: &DocumentReindexExecutionOptions,
    ) -> DocumentReindexExecutionAction {
        if options.dry_run {
            return match self.pipeline.plan_file(Path::new(&source.source_path)) {
                Ok(document) => DocumentReindexExecutionAction::planned(
                    source,
                    document.chunk_count,
                    document.title,
                ),
                Err(error) => {
                    DocumentReindexExecutionAction::failed(source, true, error.to_string())
                }
            };
        }

        let already_fresh = match self.repo.find_source_by_path(&source.source_path).await {
            Ok(existing) => existing.is_some_and(|source| !source.needs_reindex()),
            Err(error) => {
                return DocumentReindexExecutionAction::failed(source, false, error.to_string());
            }
        };

        match self.index_file(Path::new(&source.source_path)).await {
            Ok(document) => {
                if already_fresh {
                    DocumentReindexExecutionAction::already_indexed(
                        source,
                        document.chunks.len(),
                        document.parsed.title,
                    )
                } else {
                    DocumentReindexExecutionAction::reindexed(
                        source,
                        document.chunks.len(),
                        document.parsed.title,
                    )
                }
            }
            Err(error) => DocumentReindexExecutionAction::failed(source, false, error.to_string()),
        }
    }

    async fn execute_digest_reindex_action(
        &self,
        source: &DocumentReindexSourcePlan,
        options: &DocumentReindexExecutionOptions,
        digest_documents: &BTreeMap<String, DigestSourceIndexDocument>,
    ) -> DocumentReindexExecutionAction {
        let Some(document) = digest_documents.get(&source.source_path) else {
            return DocumentReindexExecutionAction::failed(
                source,
                options.dry_run,
                "source was not found in the supplied digest review batches",
            );
        };

        if options.dry_run {
            return match self.pipeline.plan_content(
                &document.document_path,
                document.indexed_content.clone(),
                Some(document.title.clone()),
            ) {
                Ok(document) => DocumentReindexExecutionAction::planned(
                    source,
                    document.chunk_count,
                    document.title,
                ),
                Err(error) => {
                    DocumentReindexExecutionAction::failed(source, true, error.to_string())
                }
            };
        }

        let already_fresh = match self.repo.find_source_by_path(&document.document_path).await {
            Ok(existing) => existing.is_some_and(|source| !source.needs_reindex()),
            Err(error) => {
                return DocumentReindexExecutionAction::failed(source, false, error.to_string());
            }
        };

        match self
            .index_content(
                &document.document_path,
                Some(document.title.clone()),
                document.indexed_content.clone(),
            )
            .await
        {
            Ok(indexed) => {
                if already_fresh {
                    DocumentReindexExecutionAction::already_indexed(
                        source,
                        indexed.chunks.len(),
                        indexed.parsed.title,
                    )
                } else {
                    DocumentReindexExecutionAction::reindexed(
                        source,
                        indexed.chunks.len(),
                        indexed.parsed.title,
                    )
                }
            }
            Err(error) => DocumentReindexExecutionAction::failed(source, false, error.to_string()),
        }
    }
}

/// Options for read-only orphan recovery analysis.
#[derive(Debug, Clone)]
pub struct DocumentRecoveryOptions {
    /// Maximum orphan source groups to include.
    pub group_limit: usize,
    /// Sample chunks to include per group.
    pub sample_limit_per_group: usize,
    /// File or directory paths to scan for current-file matches.
    pub scan_paths: Vec<PathBuf>,
    /// Digest review batch roots whose reviewed sources should be scanned.
    pub digest_review_paths: Vec<PathBuf>,
    /// Maximum candidate files or digest sources to read.
    pub max_candidate_files: usize,
    /// Maximum bytes to read per candidate.
    pub max_file_bytes: usize,
    /// Maximum candidate matches retained per orphan group.
    pub max_matches_per_group: usize,
    /// Minimum anchor-match score to keep a candidate.
    pub min_match_score: f32,
}

impl Default for DocumentRecoveryOptions {
    fn default() -> Self {
        Self {
            group_limit: DEFAULT_ORPHAN_GROUP_LIMIT,
            sample_limit_per_group: DEFAULT_ORPHAN_SAMPLE_LIMIT,
            scan_paths: Vec::new(),
            digest_review_paths: Vec::new(),
            max_candidate_files: DEFAULT_RECOVERY_MAX_CANDIDATE_FILES,
            max_file_bytes: DEFAULT_RECOVERY_MAX_FILE_BYTES,
            max_matches_per_group: DEFAULT_RECOVERY_MAX_MATCHES_PER_GROUP,
            min_match_score: DEFAULT_RECOVERY_MIN_MATCH_SCORE,
        }
    }
}

#[derive(Debug, Clone)]
struct RecoveryCandidate {
    match_type: String,
    path: String,
    normalized_content: String,
    fingerprint: String,
}

/// Read-only reindex plan built from an orphan recovery report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentReindexPlan {
    /// Always true. The plan itself performs no writes.
    pub read_only: bool,
    /// Number of orphan chunks in the source report.
    pub orphan_chunk_count: u64,
    /// Number of missing source groups in the source report.
    pub orphan_source_count: usize,
    /// Recoverable groups in the source report.
    pub recoverable_groups: usize,
    /// Unknown groups in the source report.
    pub unknown_groups: usize,
    /// Safe-to-quarantine groups in the source report.
    pub safe_to_quarantine_groups: usize,
    /// Recoverable groups covered by planned actions.
    pub planned_groups: usize,
    /// Orphan chunks covered by planned actions.
    pub planned_orphan_chunks: u64,
    /// Recoverable groups that still need manual review before action selection.
    pub review_only_groups: usize,
    /// Planned source-level actions.
    pub sources: Vec<DocumentReindexSourcePlan>,
    /// Recoverable groups that could not be assigned to a source-level action.
    pub review_only: Vec<DocumentReindexReviewOnlyGroup>,
}

/// Source-level action in a read-only reindex plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentReindexSourcePlan {
    /// Planned action kind.
    pub action: DocumentReindexAction,
    /// Source path or source identifier.
    pub source_path: String,
    /// Candidate match type used to choose this source.
    pub match_type: String,
    /// Number of orphan groups covered by this source.
    pub group_count: usize,
    /// Number of orphan chunks covered by this source.
    pub orphan_chunk_count: u64,
    /// Highest candidate score among covered groups.
    pub max_score: f32,
    /// Lowest candidate score among covered groups.
    pub min_score: f32,
    /// Existing doc_source IDs detected in the orphan evidence.
    pub existing_source_ids: Vec<String>,
    /// Covered orphan groups.
    pub groups: Vec<DocumentReindexGroupRef>,
    /// Human-readable caution notes.
    pub notes: Vec<String>,
}

/// Planned action kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentReindexAction {
    /// Reindex a normal file path with the document indexer.
    ReindexFile,
    /// Reindex through the digest source-index flow to preserve source metadata wrappers.
    ReindexDigestReviewedSource,
    /// Existing source record appears to cover this orphan group; inspect before mutating.
    InspectExistingSource,
}

impl DocumentReindexAction {
    /// Parse an action label accepted by CLI and MCP filters.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "reindex_file" | "reindex-file" | "file" => Some(Self::ReindexFile),
            "reindex_digest_reviewed_source"
            | "reindex-digest-reviewed-source"
            | "digest_reviewed_source"
            | "digest-reviewed-source"
            | "digest" => Some(Self::ReindexDigestReviewedSource),
            "inspect_existing_source" | "inspect-existing-source" | "inspect" => {
                Some(Self::InspectExistingSource)
            }
            _ => None,
        }
    }

    /// Stable snake_case action label.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReindexFile => "reindex_file",
            Self::ReindexDigestReviewedSource => "reindex_digest_reviewed_source",
            Self::InspectExistingSource => "inspect_existing_source",
        }
    }
}

/// Orphan group covered by a source-level plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentReindexGroupRef {
    /// Missing source ID.
    pub missing_source_id: String,
    /// Orphan chunk count in the group.
    pub orphan_chunk_count: u64,
    /// Recovery hint from the orphan report.
    pub recovery_hint: String,
    /// Match score used for planning.
    pub score: f32,
    /// Number of anchors matched.
    pub matched_anchors: usize,
    /// Total anchors tested.
    pub total_anchors: usize,
    /// Whether the normalized full-content fingerprint matched.
    pub exact_fingerprint_match: bool,
    /// Evidence snippets from the candidate match.
    pub evidence: Vec<String>,
}

/// Recoverable group that still needs manual review.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentReindexReviewOnlyGroup {
    /// Missing source ID.
    pub missing_source_id: String,
    /// Orphan chunk count in the group.
    pub orphan_chunk_count: u64,
    /// Recovery hint from the orphan report.
    pub recovery_hint: String,
    /// Reason no source-level action was selected.
    pub reason: String,
}

/// Guardrails for executing a source-level orphan reindex plan.
#[derive(Debug, Clone)]
pub struct DocumentReindexExecutionOptions {
    /// When true, only validate and estimate selected actions.
    pub dry_run: bool,
    /// Exact source paths to include. Empty means all sources are eligible.
    pub source_paths: Vec<String>,
    /// Action kinds to include. Empty means all action kinds are eligible.
    pub actions: Vec<DocumentReindexAction>,
    /// Digest review batch roots used to resolve digest source-index actions.
    pub digest_review_paths: Vec<PathBuf>,
    /// Maximum bytes allowed per digest source.
    pub max_source_bytes: usize,
    /// Maximum selected source actions to process.
    pub max_actions: Option<usize>,
}

impl Default for DocumentReindexExecutionOptions {
    fn default() -> Self {
        Self {
            dry_run: true,
            source_paths: Vec::new(),
            actions: Vec::new(),
            digest_review_paths: Vec::new(),
            max_source_bytes: DigestSourceIndexOptions::default().max_source_bytes,
            max_actions: None,
        }
    }
}

/// Execution report for a guarded source-level orphan reindex plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentReindexExecutionReport {
    /// Whether the run was dry-run only.
    pub dry_run: bool,
    /// Whether orphan cleanup was intentionally not performed.
    pub orphan_cleanup_performed: bool,
    /// Source-level actions in the input plan.
    pub plan_source_actions: usize,
    /// Source-level actions selected by filters before max_actions is applied.
    pub selected_source_actions: usize,
    /// Selected actions successfully planned in dry-run mode.
    pub planned_source_actions: usize,
    /// Selected actions that wrote or refreshed index content.
    pub reindexed_source_actions: usize,
    /// Selected actions that already had fresh indexed content.
    pub already_indexed_source_actions: usize,
    /// Selected actions that require manual inspection and do not write.
    pub inspection_source_actions: usize,
    /// Actions skipped by filters or max_actions.
    pub skipped_source_actions: usize,
    /// Actions that failed validation or indexing.
    pub failed_source_actions: usize,
    /// Documents written or refreshed in write mode.
    pub reindexed_documents: usize,
    /// Chunks estimated in dry-run mode.
    pub planned_chunks: usize,
    /// Chunks present after write-mode indexing.
    pub indexed_chunks: usize,
    /// Per-source action results.
    pub actions: Vec<DocumentReindexExecutionAction>,
    /// Non-fatal warnings.
    pub warnings: Vec<String>,
}

impl DocumentReindexExecutionReport {
    fn new(plan: &DocumentReindexPlan, options: &DocumentReindexExecutionOptions) -> Self {
        Self {
            dry_run: options.dry_run,
            orphan_cleanup_performed: false,
            plan_source_actions: plan.sources.len(),
            selected_source_actions: 0,
            planned_source_actions: 0,
            reindexed_source_actions: 0,
            already_indexed_source_actions: 0,
            inspection_source_actions: 0,
            skipped_source_actions: 0,
            failed_source_actions: 0,
            reindexed_documents: 0,
            planned_chunks: 0,
            indexed_chunks: 0,
            actions: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

/// Per-source result from guarded reindex execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentReindexExecutionAction {
    /// Planned action kind.
    pub action: DocumentReindexAction,
    /// Source path or source identifier.
    pub source_path: String,
    /// Candidate match type that selected this source.
    pub match_type: String,
    /// Number of orphan groups covered by this source.
    pub group_count: usize,
    /// Number of orphan chunks covered by this source.
    pub orphan_chunk_count: u64,
    /// Execution status.
    pub status: DocumentReindexExecutionStatus,
    /// Whether this action was dry-run only.
    pub dry_run: bool,
    /// Chunk count estimated or observed for the indexed document.
    pub chunk_count: Option<usize>,
    /// Indexed document title, when available.
    pub title: Option<String>,
    /// Skip/failure/manual-review reason, when applicable.
    pub reason: Option<String>,
    /// Existing doc_source IDs detected in the plan evidence.
    pub existing_source_ids: Vec<String>,
    /// Caution notes copied from the plan.
    pub notes: Vec<String>,
}

impl DocumentReindexExecutionAction {
    fn base(
        source: &DocumentReindexSourcePlan,
        dry_run: bool,
        status: DocumentReindexExecutionStatus,
    ) -> Self {
        Self {
            action: source.action,
            source_path: source.source_path.clone(),
            match_type: source.match_type.clone(),
            group_count: source.group_count,
            orphan_chunk_count: source.orphan_chunk_count,
            status,
            dry_run,
            chunk_count: None,
            title: None,
            reason: None,
            existing_source_ids: source.existing_source_ids.clone(),
            notes: source.notes.clone(),
        }
    }

    fn planned(source: &DocumentReindexSourcePlan, chunk_count: usize, title: String) -> Self {
        let mut action = Self::base(source, true, DocumentReindexExecutionStatus::Planned);
        action.chunk_count = Some(chunk_count);
        action.title = Some(title);
        action
    }

    fn reindexed(source: &DocumentReindexSourcePlan, chunk_count: usize, title: String) -> Self {
        let mut action = Self::base(source, false, DocumentReindexExecutionStatus::Reindexed);
        action.chunk_count = Some(chunk_count);
        action.title = Some(title);
        action
    }

    fn already_indexed(
        source: &DocumentReindexSourcePlan,
        chunk_count: usize,
        title: String,
    ) -> Self {
        let mut action = Self::base(
            source,
            false,
            DocumentReindexExecutionStatus::AlreadyIndexed,
        );
        action.chunk_count = Some(chunk_count);
        action.title = Some(title);
        action.reason = Some("source already had fresh indexed content".to_string());
        action
    }

    fn requires_inspection(source: &DocumentReindexSourcePlan, dry_run: bool) -> Self {
        let mut action = Self::base(
            source,
            dry_run,
            DocumentReindexExecutionStatus::RequiresInspection,
        );
        action.reason = Some(
            "existing source references require manual inspection before mutation".to_string(),
        );
        action
    }

    fn skipped(
        source: &DocumentReindexSourcePlan,
        dry_run: bool,
        reason: impl Into<String>,
    ) -> Self {
        let mut action = Self::base(source, dry_run, DocumentReindexExecutionStatus::Skipped);
        action.reason = Some(reason.into());
        action
    }

    fn failed(
        source: &DocumentReindexSourcePlan,
        dry_run: bool,
        reason: impl Into<String>,
    ) -> Self {
        let mut action = Self::base(source, dry_run, DocumentReindexExecutionStatus::Failed);
        action.reason = Some(reason.into());
        action
    }
}

/// Per-source status from guarded reindex execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentReindexExecutionStatus {
    /// Dry-run validation and chunk planning succeeded.
    Planned,
    /// Source was indexed in write mode.
    Reindexed,
    /// Source already had fresh indexed content in write mode.
    AlreadyIndexed,
    /// Source maps to existing records and needs manual inspection.
    RequiresInspection,
    /// Source was not selected or was limited out.
    Skipped,
    /// Source validation or indexing failed.
    Failed,
}

/// Inputs for building a read-only orphan cleanup/quarantine plan.
#[derive(Debug, Clone, Default)]
pub struct DocumentOrphanCleanupPlanOptions {
    /// Recovery report options, including optional candidate scans.
    pub recovery: DocumentRecoveryOptions,
    /// Optional source-level reindex plan used to map source actions back to orphan groups.
    pub reindex_plan: Option<DocumentReindexPlan>,
    /// Optional write execution report used to prove reindex coverage before delete candidacy.
    pub execution_report: Option<DocumentReindexExecutionReport>,
}

/// Read-only orphan cleanup/quarantine plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentOrphanCleanupPlan {
    /// Always true. This plan performs no writes.
    pub read_only: bool,
    /// Number of orphan chunks in the current recovery report.
    pub orphan_chunk_count: u64,
    /// Number of missing source IDs in the current recovery report.
    pub orphan_source_count: usize,
    /// Number of orphan groups included in this plan.
    pub groups_returned: usize,
    /// Recoverable groups in the current recovery report.
    pub recoverable_groups: usize,
    /// Unknown groups in the current recovery report.
    pub unknown_groups: usize,
    /// Safe-to-quarantine groups in the current recovery report.
    pub safe_to_quarantine_groups: usize,
    /// Groups that can be deleted after successful reindex coverage.
    pub delete_candidate_groups: usize,
    /// Orphan chunks covered by delete candidates.
    pub delete_candidate_chunks: u64,
    /// Groups that should be moved to quarantine review, not deleted.
    pub quarantine_candidate_groups: usize,
    /// Orphan chunks covered by quarantine candidates.
    pub quarantine_candidate_chunks: u64,
    /// Groups requiring manual review.
    pub manual_review_groups: usize,
    /// Orphan chunks covered by manual-review groups.
    pub manual_review_chunks: u64,
    /// Per-group cleanup/quarantine decisions.
    pub groups: Vec<DocumentOrphanCleanupGroupPlan>,
    /// Non-fatal warnings.
    pub warnings: Vec<String>,
}

/// Per-group cleanup/quarantine decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentOrphanCleanupGroupPlan {
    /// Proposed cleanup action.
    pub cleanup_action: DocumentOrphanCleanupAction,
    /// Missing source ID referenced by orphan chunks.
    pub missing_source_id: String,
    /// Number of orphan chunks in the group.
    pub orphan_chunk_count: u64,
    /// Current recovery classification.
    pub recovery_class: DocumentRecoveryClass,
    /// Current recovery hint.
    pub recovery_hint: String,
    /// Stable fingerprint over normalized orphan group content.
    pub content_fingerprint: String,
    /// Why this action was selected.
    pub reason: String,
    /// Reindex source path that covers this group, if any.
    pub reindex_source_path: Option<String>,
    /// Reindex action kind that covers this group, if any.
    pub reindex_action: Option<DocumentReindexAction>,
    /// Write execution status for the covering reindex source, if any.
    pub reindex_status: Option<DocumentReindexExecutionStatus>,
    /// Existing doc_source IDs detected in the orphan evidence.
    pub existing_source_ids: Vec<String>,
    /// Candidate matches retained for human review.
    pub candidate_matches: Vec<DocumentRecoveryCandidateMatch>,
    /// Bounded chunk samples for human review.
    pub samples: Vec<DocumentOrphanChunkSample>,
}

/// Proposed cleanup action for an orphan group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentOrphanCleanupAction {
    /// Covered by a successful reindex execution; eligible for explicit future delete.
    DeleteAfterSuccessfulReindex,
    /// No source clues; move to a quarantine review bucket before any deletion.
    Quarantine,
    /// Requires manual review before any mutation.
    ManualReview,
}

/// Guardrails for executing a cleanup/quarantine plan.
#[derive(Debug, Clone)]
pub struct DocumentOrphanCleanupExecutionOptions {
    /// When true, no chunks are deleted.
    pub dry_run: bool,
    /// Required for write mode. Only delete candidates are ever deleted.
    pub approve_delete_candidates: bool,
    /// Optional exact missing source IDs to include. Empty means all delete candidates are eligible.
    pub missing_source_ids: Vec<String>,
    /// Maximum selected delete groups to process.
    pub max_groups: Option<usize>,
}

impl Default for DocumentOrphanCleanupExecutionOptions {
    fn default() -> Self {
        Self {
            dry_run: true,
            approve_delete_candidates: false,
            missing_source_ids: Vec::new(),
            max_groups: None,
        }
    }
}

/// Execution report for a guarded cleanup/quarantine plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentOrphanCleanupExecutionReport {
    /// Whether the run was dry-run only.
    pub dry_run: bool,
    /// Whether orphan cleanup deletion was performed.
    pub orphan_cleanup_performed: bool,
    /// Number of groups in the cleanup plan.
    pub plan_groups: usize,
    /// Delete candidate groups in the cleanup plan.
    pub plan_delete_candidate_groups: usize,
    /// Quarantine groups in the cleanup plan.
    pub plan_quarantine_groups: usize,
    /// Manual-review groups in the cleanup plan.
    pub plan_manual_review_groups: usize,
    /// Delete candidate groups selected for dry-run or deletion.
    pub selected_delete_groups: usize,
    /// Delete candidate groups planned in dry-run mode.
    pub planned_delete_groups: usize,
    /// Chunks planned for deletion in dry-run mode.
    pub planned_delete_chunks: u64,
    /// Groups deleted in write mode.
    pub deleted_groups: usize,
    /// Chunks deleted in write mode.
    pub deleted_chunks: u64,
    /// Quarantine groups retained and never deleted by this executor.
    pub quarantine_groups_retained: usize,
    /// Quarantine chunks retained and never deleted by this executor.
    pub quarantine_chunks_retained: u64,
    /// Groups requiring manual review.
    pub manual_review_groups: usize,
    /// Groups skipped by filters or limits.
    pub skipped_groups: usize,
    /// Groups protected because their source ID exists in doc_source at execution time.
    pub protected_groups: usize,
    /// Per-group execution results.
    pub actions: Vec<DocumentOrphanCleanupExecutionAction>,
    /// Non-fatal warnings.
    pub warnings: Vec<String>,
}

impl DocumentOrphanCleanupExecutionReport {
    fn new(
        plan: &DocumentOrphanCleanupPlan,
        options: &DocumentOrphanCleanupExecutionOptions,
    ) -> Self {
        Self {
            dry_run: options.dry_run,
            orphan_cleanup_performed: false,
            plan_groups: plan.groups.len(),
            plan_delete_candidate_groups: plan.delete_candidate_groups,
            plan_quarantine_groups: plan.quarantine_candidate_groups,
            plan_manual_review_groups: plan.manual_review_groups,
            selected_delete_groups: 0,
            planned_delete_groups: 0,
            planned_delete_chunks: 0,
            deleted_groups: 0,
            deleted_chunks: 0,
            quarantine_groups_retained: 0,
            quarantine_chunks_retained: 0,
            manual_review_groups: 0,
            skipped_groups: 0,
            protected_groups: 0,
            actions: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

/// Per-group cleanup execution result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentOrphanCleanupExecutionAction {
    /// Missing source ID referenced by orphan chunks.
    pub missing_source_id: String,
    /// Cleanup action from the plan.
    pub cleanup_action: DocumentOrphanCleanupAction,
    /// Execution status.
    pub status: DocumentOrphanCleanupExecutionStatus,
    /// Whether this action was dry-run only.
    pub dry_run: bool,
    /// Number of orphan chunks in the plan for this group.
    pub planned_orphan_chunks: u64,
    /// Number of chunks deleted in write mode.
    pub deleted_chunks: u64,
    /// Human-readable reason.
    pub reason: String,
}

/// Per-group cleanup execution status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentOrphanCleanupExecutionStatus {
    /// Delete candidate selected in dry-run mode.
    PlannedDelete,
    /// Delete candidate deleted in write mode.
    Deleted,
    /// Quarantine group retained and not deleted.
    QuarantineRetained,
    /// Manual review group retained and not deleted.
    ManualReviewRequired,
    /// Group skipped by filter or max_groups.
    Skipped,
    /// Group protected because its source now exists.
    Protected,
}

/// Options for exporting a quarantine review batch.
#[derive(Debug, Clone)]
pub struct DocumentOrphanQuarantineReviewOptions {
    /// Maximum quarantine groups to export. Empty means all groups.
    pub max_groups: Option<usize>,
    /// Maximum chunks to include per group. Empty means all chunks for each group.
    pub max_chunks_per_group: Option<usize>,
    /// Maximum bytes of content to include for each chunk before truncating.
    pub max_chunk_bytes: usize,
}

impl Default for DocumentOrphanQuarantineReviewOptions {
    fn default() -> Self {
        Self {
            max_groups: None,
            max_chunks_per_group: None,
            max_chunk_bytes: DEFAULT_QUARANTINE_REVIEW_MAX_CHUNK_BYTES,
        }
    }
}

/// Export summary for a generated quarantine review batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentOrphanQuarantineReviewExport {
    /// Output directory.
    pub root: String,
    /// UTC timestamp string when the export was produced.
    pub generated_at: String,
    /// Total groups in the supplied cleanup plan.
    pub plan_groups: usize,
    /// Quarantine groups in the supplied cleanup plan.
    pub plan_quarantine_groups: usize,
    /// Quarantine chunks in the supplied cleanup plan.
    pub plan_quarantine_chunks: u64,
    /// Quarantine groups selected for export after limits.
    pub selected_groups: usize,
    /// Orphan chunks represented by selected groups according to the plan.
    pub selected_orphan_chunks: u64,
    /// Full chunks loaded from the database and written to generated pages.
    pub loaded_chunks: usize,
    /// Groups where chunk output was truncated by chunk count or content byte limits.
    pub truncated_groups: usize,
    /// Chunks whose content was truncated by max_chunk_bytes.
    pub truncated_chunks: usize,
    /// Generated files written relative to root.
    pub files_written: Vec<String>,
    /// Existing user-owned files skipped relative to root.
    pub files_skipped: Vec<String>,
    /// Non-fatal warnings.
    pub warnings: Vec<String>,
}

impl DocumentOrphanQuarantineReviewExport {
    fn new(root: &Path, plan: &DocumentOrphanCleanupPlan) -> Self {
        Self {
            root: root.display().to_string(),
            generated_at: current_utc_string(),
            plan_groups: plan.groups.len(),
            plan_quarantine_groups: plan.quarantine_candidate_groups,
            plan_quarantine_chunks: plan.quarantine_candidate_chunks,
            selected_groups: 0,
            selected_orphan_chunks: 0,
            loaded_chunks: 0,
            truncated_groups: 0,
            truncated_chunks: 0,
            files_written: Vec::new(),
            files_skipped: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

/// Options for applying a quarantine review batch.
#[derive(Debug, Clone)]
pub struct DocumentOrphanQuarantineReviewApplyOptions {
    /// When true, only reports planned actions. Write mode is not implemented yet.
    pub dry_run: bool,
}

impl Default for DocumentOrphanQuarantineReviewApplyOptions {
    fn default() -> Self {
        Self { dry_run: true }
    }
}

/// Options for prioritizing a quarantine review batch.
#[derive(Debug, Clone)]
pub struct DocumentOrphanQuarantineReviewPrioritizationOptions {
    /// Maximum prioritized items to return.
    pub limit: Option<usize>,
    /// Include already decided pages in the ranked output.
    pub include_decided: bool,
    /// Include multiple pages with the same content fingerprint in returned pilot items.
    pub include_duplicate_fingerprints: bool,
    /// Maximum excerpt bytes to include per item.
    pub max_excerpt_bytes: usize,
}

impl Default for DocumentOrphanQuarantineReviewPrioritizationOptions {
    fn default() -> Self {
        Self {
            limit: Some(10),
            include_decided: false,
            include_duplicate_fingerprints: false,
            max_excerpt_bytes: 800,
        }
    }
}

/// Read-only prioritization report for a quarantine review batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentOrphanQuarantineReviewPrioritization {
    /// Review batch root directory.
    pub root: String,
    /// Markdown files scanned under the review directory.
    pub files_scanned: usize,
    /// Generated group pages found.
    pub group_pages: usize,
    /// Pending group pages found.
    pub pending_count: usize,
    /// Decided pages skipped because include_decided was false.
    pub decided_skipped_count: usize,
    /// Invalid or parse-error pages skipped.
    pub invalid_or_parse_error_count: usize,
    /// Candidate pages considered for ranking.
    pub candidate_count: usize,
    /// Candidate pages retained after duplicate fingerprint filtering.
    pub ranked_candidate_count: usize,
    /// Items returned after limit.
    pub returned_count: usize,
    /// Content fingerprint groups with more than one candidate page.
    pub duplicate_fingerprint_group_count: usize,
    /// Candidate pages that belong to duplicate fingerprint groups.
    pub duplicate_fingerprint_candidate_count: usize,
    /// Candidate pages skipped because duplicate fingerprints are excluded.
    pub duplicate_fingerprint_skipped_count: usize,
    /// High-priority candidates in the full candidate set.
    pub high_priority_count: usize,
    /// Medium-priority candidates in the full candidate set.
    pub medium_priority_count: usize,
    /// Low-priority candidates in the full candidate set.
    pub low_priority_count: usize,
    /// Ranked pilot candidates.
    pub items: Vec<DocumentOrphanQuarantineReviewPriorityItem>,
    /// Non-fatal warnings.
    pub warnings: Vec<String>,
}

impl DocumentOrphanQuarantineReviewPrioritization {
    fn new(root: &Path) -> Self {
        Self {
            root: root.display().to_string(),
            files_scanned: 0,
            group_pages: 0,
            pending_count: 0,
            decided_skipped_count: 0,
            invalid_or_parse_error_count: 0,
            candidate_count: 0,
            ranked_candidate_count: 0,
            returned_count: 0,
            duplicate_fingerprint_group_count: 0,
            duplicate_fingerprint_candidate_count: 0,
            duplicate_fingerprint_skipped_count: 0,
            high_priority_count: 0,
            medium_priority_count: 0,
            low_priority_count: 0,
            items: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

/// Ranked quarantine review page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentOrphanQuarantineReviewPriorityItem {
    /// File path relative to the review root.
    pub relative_path: String,
    /// Missing source ID represented by the review page.
    pub missing_source_id: String,
    /// Current parsed page state.
    pub status: DocumentOrphanQuarantineReviewFileState,
    /// Existing decision, when present.
    pub decision: Option<DocumentOrphanQuarantineReviewDecision>,
    /// Deterministic review priority.
    pub priority: DocumentOrphanQuarantineReviewPriority,
    /// Numeric score used for sorting.
    pub score: i32,
    /// Suggested next human action, not an automatic decision.
    pub suggested_next_step: DocumentOrphanQuarantineReviewSuggestedStep,
    /// Plan orphan chunks from frontmatter.
    pub orphan_chunk_count: u64,
    /// Exported chunks from frontmatter.
    pub exported_chunk_count: usize,
    /// Recovery class extracted from the generated summary.
    pub recovery_class: Option<String>,
    /// Recovery hint extracted from the generated summary.
    pub recovery_hint: Option<String>,
    /// Generated reason extracted from the generated summary.
    pub reason: Option<String>,
    /// Content fingerprint from frontmatter.
    pub content_fingerprint: Option<String>,
    /// Number of candidate pages sharing this content fingerprint, including this page.
    pub fingerprint_group_size: usize,
    /// This page's 1-based rank within its content fingerprint group.
    pub fingerprint_group_rank: usize,
    /// Other review pages with the same content fingerprint.
    pub fingerprint_duplicate_paths: Vec<String>,
    /// First meaningful heading extracted from chunk metadata.
    pub title_hint: Option<String>,
    /// Short content excerpt for quick triage.
    pub excerpt: String,
    /// Transparent reasons contributing to the score.
    pub score_reasons: Vec<String>,
    /// Keyword/topic signals detected in headings or content.
    pub detected_signals: Vec<String>,
}

/// Coarse priority bucket.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentOrphanQuarantineReviewPriority {
    /// Review early in a pilot batch.
    High,
    /// Review after high-priority pages.
    Medium,
    /// Lower-value or low-signal page.
    Low,
}

/// Suggested human action for a prioritized page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentOrphanQuarantineReviewSuggestedStep {
    /// Inspect first for possible Memory OS promotion.
    InspectForMemoryPromotion,
    /// Inspect as supporting archival/reference material.
    InspectForArchiveOrRetention,
    /// Leave for later after higher-signal pages.
    Defer,
}

/// Status summary for a generated quarantine review batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentOrphanQuarantineReviewStatus {
    /// Review batch root directory.
    pub root: String,
    /// Markdown files scanned under the review directory.
    pub files_scanned: usize,
    /// Generated Engram review files detected by marker.
    pub generated_files: usize,
    /// Generated index pages detected.
    pub index_pages: usize,
    /// Generated group pages detected.
    pub group_pages: usize,
    /// Non-generated Markdown files skipped as user-owned.
    pub user_owned_files: usize,
    /// Pages with pending or missing decisions.
    pub pending_count: usize,
    /// Pages decided to remain in quarantine.
    pub retain_quarantine_count: usize,
    /// Pages decided for future Memory OS review promotion.
    pub promote_to_memory_review_count: usize,
    /// Pages decided for future legacy archive.
    pub archive_legacy_count: usize,
    /// Pages decided for future delete consideration.
    pub delete_later_count: usize,
    /// Generated group pages with invalid decisions or missing required fields.
    pub invalid_count: usize,
    /// Generated pages with missing/invalid generated metadata.
    pub parse_error_count: usize,
    /// True when all group pages are decided and valid.
    pub ready_to_apply: bool,
    /// Per-file group-page status.
    pub files: Vec<DocumentOrphanQuarantineReviewFileStatus>,
    /// Non-fatal warnings.
    pub warnings: Vec<String>,
}

impl DocumentOrphanQuarantineReviewStatus {
    fn new(root: &Path) -> Self {
        Self {
            root: root.display().to_string(),
            files_scanned: 0,
            generated_files: 0,
            index_pages: 0,
            group_pages: 0,
            user_owned_files: 0,
            pending_count: 0,
            retain_quarantine_count: 0,
            promote_to_memory_review_count: 0,
            archive_legacy_count: 0,
            delete_later_count: 0,
            invalid_count: 0,
            parse_error_count: 0,
            ready_to_apply: false,
            files: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

/// Per-file status in a quarantine review batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentOrphanQuarantineReviewFileStatus {
    /// File path relative to the review root.
    pub relative_path: String,
    /// Missing source ID from generated frontmatter.
    pub missing_source_id: Option<String>,
    /// Parsed decision, when available.
    pub decision: Option<DocumentOrphanQuarantineReviewDecision>,
    /// Parsed memory kind for promotion decisions.
    pub memory_kind: Option<String>,
    /// Parsed scope type for promotion decisions.
    pub scope_type: Option<String>,
    /// Parsed scope name for promotion decisions.
    pub scope_name: Option<String>,
    /// Parsed title for promotion decisions.
    pub title: Option<String>,
    /// Parsed notes.
    pub notes: Option<String>,
    /// Status category.
    pub status: DocumentOrphanQuarantineReviewFileState,
    /// Human-readable status detail.
    pub message: Option<String>,
}

/// Per-file status category for quarantine review pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentOrphanQuarantineReviewFileState {
    /// Review decision is pending.
    Pending,
    /// Review decision is explicit and valid.
    Decided,
    /// Decision is invalid or missing required fields.
    Invalid,
    /// Generated metadata could not be parsed.
    ParseError,
    /// Markdown file is not generated by this review exporter.
    UserOwned,
    /// Generated page is not a group decision page.
    NonGroupGenerated,
}

/// Explicit decision in a quarantine review page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentOrphanQuarantineReviewDecision {
    /// No decision yet.
    Pending,
    /// Keep the group quarantined.
    RetainQuarantine,
    /// Create a later Memory OS migration review item.
    PromoteToMemoryReview,
    /// Archive as legacy material in a later write step.
    ArchiveLegacy,
    /// Consider for deletion in a later guarded cleanup step.
    DeleteLater,
}

/// Dry-run apply report for a quarantine review batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentOrphanQuarantineReviewApply {
    /// Review batch root directory.
    pub root: String,
    /// Always true for now.
    pub dry_run: bool,
    /// Files scanned by status.
    pub files_scanned: usize,
    /// Generated group pages.
    pub group_pages: usize,
    /// Pages still pending.
    pub pending_count: usize,
    /// Invalid pages that block future write mode.
    pub invalid_count: usize,
    /// Parse errors that block future write mode.
    pub parse_error_count: usize,
    /// Groups that would remain quarantined.
    pub retain_quarantine_count: usize,
    /// Groups that would be promoted to Memory OS review.
    pub promote_to_memory_review_count: usize,
    /// Groups that would be archived as legacy material.
    pub archive_legacy_count: usize,
    /// Groups that would be marked for later guarded deletion.
    pub delete_later_count: usize,
    /// True when there are no pending, invalid, or parse-error group pages.
    pub ready_for_future_write: bool,
    /// Planned Memory OS review promotions.
    pub planned_memory_review_items: Vec<DocumentOrphanQuarantineMemoryReviewPlan>,
    /// Groups that would remain quarantined.
    pub retained_quarantine_groups: Vec<String>,
    /// Groups that would be archived later.
    pub archive_legacy_groups: Vec<String>,
    /// Groups that would be considered for later deletion.
    pub delete_later_groups: Vec<String>,
    /// Non-fatal warnings.
    pub warnings: Vec<String>,
}

/// Planned Memory OS review item from a quarantine review page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentOrphanQuarantineMemoryReviewPlan {
    /// Review file path relative to root.
    pub relative_path: String,
    /// Missing source ID represented by the review page.
    pub missing_source_id: String,
    /// Requested memory kind.
    pub memory_kind: String,
    /// Requested memory scope type.
    pub scope_type: String,
    /// Requested memory scope name.
    pub scope_name: Option<String>,
    /// Requested title.
    pub title: String,
    /// Optional notes.
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct PreparedQuarantineReviewGroup {
    missing_source_id: String,
    relative_path: PathBuf,
    plan: DocumentOrphanCleanupGroupPlan,
    chunks: Vec<PreparedQuarantineReviewChunk>,
    database_chunk_count: usize,
    truncated_by_chunk_limit: bool,
}

#[derive(Debug, Clone, Serialize)]
struct PreparedQuarantineReviewChunk {
    chunk_id: String,
    heading_path: String,
    start_line: Option<u32>,
    end_line: Option<u32>,
    content: String,
    content_bytes: usize,
    content_truncated: bool,
}

async fn export_orphan_quarantine_review(
    repo: &DocumentRepo,
    plan: &DocumentOrphanCleanupPlan,
    root: &Path,
    options: DocumentOrphanQuarantineReviewOptions,
) -> IndexResult<DocumentOrphanQuarantineReviewExport> {
    fs::create_dir_all(root)?;

    let mut export = DocumentOrphanQuarantineReviewExport::new(root, plan);
    let mut groups = plan
        .groups
        .iter()
        .filter(|group| group.cleanup_action == DocumentOrphanCleanupAction::Quarantine)
        .collect::<Vec<_>>();

    groups.sort_by(|left, right| {
        right
            .orphan_chunk_count
            .cmp(&left.orphan_chunk_count)
            .then_with(|| left.missing_source_id.cmp(&right.missing_source_id))
    });

    let total_quarantine_groups = groups.len();
    if let Some(max_groups) = options.max_groups {
        if groups.len() > max_groups {
            export.warnings.push(format!(
                "quarantine review export limited to {max_groups} of {total_quarantine_groups} groups"
            ));
            groups.truncate(max_groups);
        }
    }

    let mut prepared_groups = Vec::new();
    for (index, group) in groups.into_iter().enumerate() {
        let source_id = Id::parse(&group.missing_source_id).map_err(|error| {
            IndexError::Parse(format!(
                "invalid missing source ID {}: {error}",
                group.missing_source_id
            ))
        })?;
        let mut chunks = repo.get_chunks_for_source(&source_id).await?;
        chunks.sort_by(|left, right| {
            left.start_line
                .cmp(&right.start_line)
                .then_with(|| left.heading_path.cmp(&right.heading_path))
                .then_with(|| left.id.to_string().cmp(&right.id.to_string()))
        });

        if chunks.len() as u64 != group.orphan_chunk_count {
            export.warnings.push(format!(
                "group {} planned {} chunks but database returned {} chunks",
                group.missing_source_id,
                group.orphan_chunk_count,
                chunks.len()
            ));
        }

        let database_chunk_count = chunks.len();
        let truncated_by_chunk_limit = options
            .max_chunks_per_group
            .is_some_and(|limit| chunks.len() > limit);
        if let Some(limit) = options.max_chunks_per_group {
            chunks.truncate(limit);
        }

        let prepared_chunks = chunks
            .into_iter()
            .map(|chunk| prepare_quarantine_review_chunk(chunk, options.max_chunk_bytes))
            .collect::<Vec<_>>();
        let relative_path = quarantine_review_group_path(index + 1, total_quarantine_groups, group);

        export.selected_groups += 1;
        export.selected_orphan_chunks += group.orphan_chunk_count;
        export.loaded_chunks += prepared_chunks.len();
        if truncated_by_chunk_limit || prepared_chunks.iter().any(|chunk| chunk.content_truncated) {
            export.truncated_groups += 1;
        }
        export.truncated_chunks += prepared_chunks
            .iter()
            .filter(|chunk| chunk.content_truncated)
            .count();

        prepared_groups.push(PreparedQuarantineReviewGroup {
            missing_source_id: group.missing_source_id.clone(),
            relative_path,
            plan: group.clone(),
            chunks: prepared_chunks,
            database_chunk_count,
            truncated_by_chunk_limit,
        });
    }

    write_quarantine_review_file(
        root,
        Path::new("index.md").to_path_buf(),
        &quarantine_review_index_page(plan, &export, &prepared_groups),
        &mut export,
    )?;

    for (index, group) in prepared_groups.iter().enumerate() {
        write_quarantine_review_file(
            root,
            group.relative_path.clone(),
            &quarantine_review_group_page(index + 1, group, &options),
            &mut export,
        )?;
    }

    export.files_written.sort();
    export.files_skipped.sort();
    Ok(export)
}

fn prepare_quarantine_review_chunk(
    chunk: DocChunk,
    max_chunk_bytes: usize,
) -> PreparedQuarantineReviewChunk {
    let content_bytes = chunk.content.len();
    let (content, content_truncated) = truncate_to_byte_limit(&chunk.content, max_chunk_bytes);
    PreparedQuarantineReviewChunk {
        chunk_id: chunk.id.to_string(),
        heading_path: chunk.heading_path,
        start_line: chunk.start_line,
        end_line: chunk.end_line,
        content,
        content_bytes,
        content_truncated,
    }
}

fn quarantine_review_group_path(
    index: usize,
    total_groups: usize,
    group: &DocumentOrphanCleanupGroupPlan,
) -> PathBuf {
    let width = total_groups.max(1).to_string().len().max(4);
    let prefix = format!("{index:0width$}", width = width);
    Path::new("groups").join(format!("{prefix}-{}.md", group.missing_source_id))
}

fn quarantine_review_index_page(
    plan: &DocumentOrphanCleanupPlan,
    export: &DocumentOrphanQuarantineReviewExport,
    groups: &[PreparedQuarantineReviewGroup],
) -> String {
    let mut output = quarantine_review_frontmatter(
        "document_orphan_quarantine_review_index",
        vec![
            ("generated_at", export.generated_at.clone()),
            ("plan_groups", plan.groups.len().to_string()),
            (
                "plan_quarantine_groups",
                plan.quarantine_candidate_groups.to_string(),
            ),
            (
                "plan_quarantine_chunks",
                plan.quarantine_candidate_chunks.to_string(),
            ),
        ],
    );

    output.push_str("# Document Orphan Quarantine Review\n\n");
    output.push_str("## Summary\n\n");
    output.push_str(&format!(
        "- Plan quarantine groups: {}\n",
        plan.quarantine_candidate_groups
    ));
    output.push_str(&format!(
        "- Plan quarantine chunks: {}\n",
        plan.quarantine_candidate_chunks
    ));
    output.push_str(&format!("- Selected groups: {}\n", export.selected_groups));
    output.push_str(&format!(
        "- Selected orphan chunks: {}\n",
        export.selected_orphan_chunks
    ));
    output.push_str(&format!("- Loaded chunks: {}\n", export.loaded_chunks));
    output.push_str(&format!(
        "- Truncated groups: {}\n",
        export.truncated_groups
    ));
    output.push_str(&format!(
        "- Truncated chunks: {}\n\n",
        export.truncated_chunks
    ));

    output.push_str("## Review Contract\n\n");
    output.push_str("- This batch is generated from retained orphan chunks only.\n");
    output.push_str("- No memory, archive, or deletion action is performed by this export.\n");
    output
        .push_str("- Each group page has a pending decision block for explicit human review.\n\n");

    output.push_str("## Groups\n\n");
    for group in groups {
        output.push_str(&format!(
            "- [{}]({}) - {} plan chunks, {} loaded chunks\n",
            group.missing_source_id,
            markdown_path(&group.relative_path),
            group.plan.orphan_chunk_count,
            group.chunks.len()
        ));
    }

    if !export.warnings.is_empty() {
        output.push_str("\n## Warnings\n\n");
        for warning in &export.warnings {
            output.push_str(&format!("- {warning}\n"));
        }
    }

    output
}

fn quarantine_review_group_page(
    index: usize,
    group: &PreparedQuarantineReviewGroup,
    options: &DocumentOrphanQuarantineReviewOptions,
) -> String {
    let mut output = quarantine_review_frontmatter(
        "document_orphan_quarantine_review_group",
        vec![
            ("missing_source_id", group.missing_source_id.clone()),
            (
                "content_fingerprint",
                group.plan.content_fingerprint.clone(),
            ),
            (
                "orphan_chunk_count",
                group.plan.orphan_chunk_count.to_string(),
            ),
            (
                "database_chunk_count",
                group.database_chunk_count.to_string(),
            ),
            ("exported_chunk_count", group.chunks.len().to_string()),
        ],
    );

    output.push_str(&format!(
        "# Quarantine Group {}: `{}`\n\n",
        index, group.missing_source_id
    ));
    output.push_str("## Summary\n\n");
    output.push_str(&format!(
        "- Missing source ID: `{}`\n",
        group.missing_source_id
    ));
    output.push_str(&format!(
        "- Plan orphan chunks: {}\n",
        group.plan.orphan_chunk_count
    ));
    output.push_str(&format!(
        "- Database chunks loaded: {}\n",
        group.database_chunk_count
    ));
    output.push_str(&format!("- Exported chunks: {}\n", group.chunks.len()));
    output.push_str(&format!(
        "- Recovery class: `{:?}`\n",
        group.plan.recovery_class
    ));
    output.push_str(&format!(
        "- Recovery hint: `{}`\n",
        group.plan.recovery_hint
    ));
    output.push_str(&format!("- Reason: {}\n", group.plan.reason));
    output.push_str(&format!(
        "- Content fingerprint: `{}`\n",
        group.plan.content_fingerprint
    ));
    if group.truncated_by_chunk_limit {
        output.push_str(&format!(
            "- Chunk list truncated by max_chunks_per_group: {:?}\n",
            options.max_chunks_per_group
        ));
    }

    output.push_str("\n## Review Decision\n\n");
    output.push_str("Set `decision` before a future apply step consumes this file.\n\n");
    output.push_str("```yaml\n");
    output.push_str("decision: pending # retain_quarantine | promote_to_memory_review | archive_legacy | delete_later\n");
    output.push_str("memory_kind: null # preference | rule | decision | limitation | project_fact | repository_fact | task_fact | user_fact | session_insight\n");
    output.push_str("scope_type: null # global | user | project | task | entity | repository | session | custom\n");
    output.push_str("scope_name: null\n");
    output.push_str("title: null\n");
    output.push_str("notes: null\n");
    output.push_str("```\n\n");

    output.push_str("## Chunk Contents\n\n");
    for (chunk_index, chunk) in group.chunks.iter().enumerate() {
        output.push_str(&format!("### Chunk {}\n\n", chunk_index + 1));
        output.push_str(&format!("- Chunk ID: `{}`\n", chunk.chunk_id));
        output.push_str(&format!("- Heading: `{}`\n", chunk.heading_path));
        if let Some(start_line) = chunk.start_line {
            output.push_str(&format!("- Start line: {}\n", start_line));
        }
        if let Some(end_line) = chunk.end_line {
            output.push_str(&format!("- End line: {}\n", end_line));
        }
        output.push_str(&format!("- Original bytes: {}\n", chunk.content_bytes));
        output.push_str(&format!(
            "- Content truncated: {}\n\n",
            chunk.content_truncated
        ));
        output.push_str(&markdown_code_block("markdown", &chunk.content));
        output.push_str("\n\n");
    }

    output.push_str("## Machine Record\n\n");
    output.push_str(&markdown_code_block(
        "json",
        &serde_json::to_string_pretty(group)
            .expect("quarantine review group JSON serialization should succeed"),
    ));
    output.push('\n');

    output
}

fn write_quarantine_review_file(
    root: &Path,
    relative_path: PathBuf,
    contents: &str,
    export: &mut DocumentOrphanQuarantineReviewExport,
) -> IndexResult<()> {
    let path = root.join(&relative_path);
    if path.exists() {
        let existing = fs::read(&path)?;
        if !String::from_utf8_lossy(&existing).contains(DOCUMENT_ORPHAN_QUARANTINE_REVIEW_MARKER) {
            export.files_skipped.push(markdown_path(&relative_path));
            return Ok(());
        }
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, contents)?;
    export.files_written.push(markdown_path(&relative_path));
    Ok(())
}

fn quarantine_review_frontmatter(page_type: &str, fields: Vec<(&str, String)>) -> String {
    let mut output = String::new();
    output.push_str("---\n");
    output.push_str("generated_by: \"engram-memory-os\"\n");
    output.push_str(&format!("page_type: {}\n", yaml_string(page_type)));
    for (key, value) in fields {
        output.push_str(&format!("{key}: {}\n", yaml_string(&value)));
    }
    output.push_str("---\n\n");
    output.push_str(DOCUMENT_ORPHAN_QUARANTINE_REVIEW_MARKER);
    output.push_str("\n\n");
    output
}

fn truncate_to_byte_limit(value: &str, max_bytes: usize) -> (String, bool) {
    if value.len() <= max_bytes {
        return (value.to_string(), false);
    }

    let mut end = max_bytes.min(value.len());
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    let mut truncated = value[..end].to_string();
    truncated.push_str(&format!("\n\n[truncated at {max_bytes} bytes]"));
    (truncated, true)
}

fn markdown_code_block(language: &str, content: &str) -> String {
    let fence_len = longest_backtick_run(content).max(2) + 1;
    let fence = "`".repeat(fence_len);
    format!("{fence}{language}\n{content}\n{fence}")
}

fn longest_backtick_run(content: &str) -> usize {
    let mut longest = 0;
    let mut current = 0;
    for character in content.chars() {
        if character == '`' {
            current += 1;
            longest = longest.max(current);
        } else {
            current = 0;
        }
    }
    longest
}

fn yaml_string(value: &str) -> String {
    serde_json::to_string(value).expect("string JSON serialization should succeed")
}

fn markdown_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn current_utc_string() -> String {
    let now = time::OffsetDateTime::now_utc();
    now.format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| now.to_string())
}

fn orphan_quarantine_review_status(
    root: &Path,
) -> IndexResult<DocumentOrphanQuarantineReviewStatus> {
    let mut status = DocumentOrphanQuarantineReviewStatus::new(root);
    let files = collect_markdown_files(root)?;
    status.files_scanned = files.len();

    for path in files {
        let relative_path = relative_path_string(root, &path);
        let contents = fs::read_to_string(&path)?;
        let page = parse_quarantine_review_page(&contents, &relative_path);

        match page.state {
            DocumentOrphanQuarantineReviewFileState::UserOwned => {
                status.user_owned_files += 1;
            }
            DocumentOrphanQuarantineReviewFileState::NonGroupGenerated => {
                status.generated_files += 1;
                status.index_pages += usize::from(
                    page.page_type.as_deref() == Some("document_orphan_quarantine_review_index"),
                );
            }
            DocumentOrphanQuarantineReviewFileState::ParseError => {
                status.generated_files += 1;
                status.group_pages += 1;
                status.parse_error_count += 1;
                status.warnings.push(format!(
                    "{relative_path}: {}",
                    page.message
                        .as_deref()
                        .unwrap_or("failed to parse generated review page")
                ));
                status.files.push(page.into_file_status());
            }
            DocumentOrphanQuarantineReviewFileState::Pending
            | DocumentOrphanQuarantineReviewFileState::Decided
            | DocumentOrphanQuarantineReviewFileState::Invalid => {
                status.generated_files += 1;
                status.group_pages += 1;
                update_quarantine_review_status_counts(&mut status, &page);
                if page.state == DocumentOrphanQuarantineReviewFileState::Invalid {
                    status.warnings.push(format!(
                        "{relative_path}: {}",
                        page.message.as_deref().unwrap_or("invalid review decision")
                    ));
                }
                status.files.push(page.into_file_status());
            }
        }
    }

    status.files.sort_by(|left, right| {
        left.relative_path
            .cmp(&right.relative_path)
            .then_with(|| left.missing_source_id.cmp(&right.missing_source_id))
    });
    status.ready_to_apply = status.group_pages > 0
        && status.pending_count == 0
        && status.invalid_count == 0
        && status.parse_error_count == 0;
    Ok(status)
}

fn update_quarantine_review_status_counts(
    status: &mut DocumentOrphanQuarantineReviewStatus,
    page: &ParsedQuarantineReviewPage,
) {
    match page.state {
        DocumentOrphanQuarantineReviewFileState::Pending => status.pending_count += 1,
        DocumentOrphanQuarantineReviewFileState::Invalid => status.invalid_count += 1,
        DocumentOrphanQuarantineReviewFileState::Decided => match page.decision {
            Some(DocumentOrphanQuarantineReviewDecision::RetainQuarantine) => {
                status.retain_quarantine_count += 1;
            }
            Some(DocumentOrphanQuarantineReviewDecision::PromoteToMemoryReview) => {
                status.promote_to_memory_review_count += 1;
            }
            Some(DocumentOrphanQuarantineReviewDecision::ArchiveLegacy) => {
                status.archive_legacy_count += 1;
            }
            Some(DocumentOrphanQuarantineReviewDecision::DeleteLater) => {
                status.delete_later_count += 1;
            }
            Some(DocumentOrphanQuarantineReviewDecision::Pending) | None => {
                status.pending_count += 1;
            }
        },
        DocumentOrphanQuarantineReviewFileState::ParseError => status.parse_error_count += 1,
        DocumentOrphanQuarantineReviewFileState::UserOwned
        | DocumentOrphanQuarantineReviewFileState::NonGroupGenerated => {}
    }
}

fn apply_orphan_quarantine_review(
    root: &Path,
    options: DocumentOrphanQuarantineReviewApplyOptions,
) -> IndexResult<DocumentOrphanQuarantineReviewApply> {
    let status = orphan_quarantine_review_status(root)?;
    let mut report = DocumentOrphanQuarantineReviewApply {
        root: status.root.clone(),
        dry_run: options.dry_run,
        files_scanned: status.files_scanned,
        group_pages: status.group_pages,
        pending_count: status.pending_count,
        invalid_count: status.invalid_count,
        parse_error_count: status.parse_error_count,
        retain_quarantine_count: 0,
        promote_to_memory_review_count: 0,
        archive_legacy_count: 0,
        delete_later_count: 0,
        ready_for_future_write: false,
        planned_memory_review_items: Vec::new(),
        retained_quarantine_groups: Vec::new(),
        archive_legacy_groups: Vec::new(),
        delete_later_groups: Vec::new(),
        warnings: status.warnings,
    };

    for file in status.files {
        if file.status != DocumentOrphanQuarantineReviewFileState::Decided {
            continue;
        }
        let Some(decision) = file.decision else {
            continue;
        };
        let Some(missing_source_id) = file.missing_source_id.clone() else {
            report.invalid_count += 1;
            report.warnings.push(format!(
                "{}: decided page is missing source ID",
                file.relative_path
            ));
            continue;
        };

        match decision {
            DocumentOrphanQuarantineReviewDecision::RetainQuarantine => {
                report.retain_quarantine_count += 1;
                report.retained_quarantine_groups.push(missing_source_id);
            }
            DocumentOrphanQuarantineReviewDecision::PromoteToMemoryReview => {
                report.promote_to_memory_review_count += 1;
                report
                    .planned_memory_review_items
                    .push(DocumentOrphanQuarantineMemoryReviewPlan {
                        relative_path: file.relative_path,
                        missing_source_id,
                        memory_kind: file.memory_kind.unwrap_or_default(),
                        scope_type: file.scope_type.unwrap_or_default(),
                        scope_name: file.scope_name,
                        title: file.title.unwrap_or_default(),
                        notes: file.notes,
                    });
            }
            DocumentOrphanQuarantineReviewDecision::ArchiveLegacy => {
                report.archive_legacy_count += 1;
                report.archive_legacy_groups.push(missing_source_id);
            }
            DocumentOrphanQuarantineReviewDecision::DeleteLater => {
                report.delete_later_count += 1;
                report.delete_later_groups.push(missing_source_id);
            }
            DocumentOrphanQuarantineReviewDecision::Pending => {}
        }
    }

    report.retained_quarantine_groups.sort();
    report.archive_legacy_groups.sort();
    report.delete_later_groups.sort();
    report.planned_memory_review_items.sort_by(|left, right| {
        left.relative_path
            .cmp(&right.relative_path)
            .then_with(|| left.missing_source_id.cmp(&right.missing_source_id))
    });
    report.ready_for_future_write = report.group_pages > 0
        && report.pending_count == 0
        && report.invalid_count == 0
        && report.parse_error_count == 0;
    Ok(report)
}

fn prioritize_orphan_quarantine_review(
    root: &Path,
    options: DocumentOrphanQuarantineReviewPrioritizationOptions,
) -> IndexResult<DocumentOrphanQuarantineReviewPrioritization> {
    let mut report = DocumentOrphanQuarantineReviewPrioritization::new(root);
    let files = collect_markdown_files(root)?;
    report.files_scanned = files.len();

    let mut candidates = Vec::new();
    for path in files {
        let relative_path = relative_path_string(root, &path);
        let contents = fs::read_to_string(&path)?;
        let page = parse_quarantine_review_page(&contents, &relative_path);
        match page.state {
            DocumentOrphanQuarantineReviewFileState::UserOwned
            | DocumentOrphanQuarantineReviewFileState::NonGroupGenerated => continue,
            DocumentOrphanQuarantineReviewFileState::ParseError
            | DocumentOrphanQuarantineReviewFileState::Invalid => {
                report.group_pages += 1;
                report.invalid_or_parse_error_count += 1;
                report.warnings.push(format!(
                    "{relative_path}: {}",
                    page.message
                        .as_deref()
                        .unwrap_or("cannot prioritize invalid review page")
                ));
                continue;
            }
            DocumentOrphanQuarantineReviewFileState::Pending => {
                report.group_pages += 1;
                report.pending_count += 1;
            }
            DocumentOrphanQuarantineReviewFileState::Decided => {
                report.group_pages += 1;
                if !options.include_decided {
                    report.decided_skipped_count += 1;
                    continue;
                }
            }
        }

        if let Some(item) = prioritize_quarantine_review_page(&contents, page, &options) {
            candidates.push(item);
        }
    }

    candidates.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| right.orphan_chunk_count.cmp(&left.orphan_chunk_count))
            .then_with(|| left.relative_path.cmp(&right.relative_path))
    });

    let (duplicate_group_count, duplicate_candidate_count) =
        annotate_quarantine_review_fingerprint_groups(&mut candidates);
    report.candidate_count = candidates.len();
    report.duplicate_fingerprint_group_count = duplicate_group_count;
    report.duplicate_fingerprint_candidate_count = duplicate_candidate_count;
    for item in &candidates {
        match item.priority {
            DocumentOrphanQuarantineReviewPriority::High => report.high_priority_count += 1,
            DocumentOrphanQuarantineReviewPriority::Medium => report.medium_priority_count += 1,
            DocumentOrphanQuarantineReviewPriority::Low => report.low_priority_count += 1,
        }
    }

    if !options.include_duplicate_fingerprints {
        let before = candidates.len();
        let mut seen_fingerprints = BTreeSet::new();
        candidates.retain(|item| {
            let Some(fingerprint) = normalized_review_fingerprint(item) else {
                return true;
            };
            seen_fingerprints.insert(fingerprint)
        });
        report.duplicate_fingerprint_skipped_count = before - candidates.len();
    }
    report.ranked_candidate_count = candidates.len();

    if let Some(limit) = options.limit {
        candidates.truncate(limit);
    }
    report.returned_count = candidates.len();
    report.items = candidates;
    Ok(report)
}

fn prioritize_quarantine_review_page(
    contents: &str,
    page: ParsedQuarantineReviewPage,
    options: &DocumentOrphanQuarantineReviewPrioritizationOptions,
) -> Option<DocumentOrphanQuarantineReviewPriorityItem> {
    let missing_source_id = page.missing_source_id.clone()?;
    let frontmatter = frontmatter_fields(contents);
    let orphan_chunk_count = frontmatter
        .get("orphan_chunk_count")
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(0);
    let exported_chunk_count = frontmatter
        .get("exported_chunk_count")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);
    let content_fingerprint = frontmatter.get("content_fingerprint").cloned();
    let recovery_class = summary_value(contents, "Recovery class");
    let recovery_hint = summary_value(contents, "Recovery hint");
    let reason = summary_value(contents, "Reason");
    let headings = chunk_headings(contents);
    let content_region = quarantine_review_content_region(contents);
    let text_for_signals = format!(
        "{}\n{}\n{}\n{}",
        headings.join("\n"),
        content_region,
        recovery_class.as_deref().unwrap_or_default(),
        reason.as_deref().unwrap_or_default()
    );
    let detected_signals = review_priority_signals(&text_for_signals);
    let mut score_reasons = Vec::new();
    let mut score = 0;

    if page.state == DocumentOrphanQuarantineReviewFileState::Pending {
        score += 20;
        score_reasons.push("pending page needs an explicit human decision".to_string());
    }
    if orphan_chunk_count >= 50 {
        score += 24;
        score_reasons.push(format!("large orphan group ({orphan_chunk_count} chunks)"));
    } else if orphan_chunk_count >= 20 {
        score += 16;
        score_reasons.push(format!(
            "substantial orphan group ({orphan_chunk_count} chunks)"
        ));
    } else if orphan_chunk_count >= 5 {
        score += 8;
        score_reasons.push(format!(
            "multi-chunk orphan group ({orphan_chunk_count} chunks)"
        ));
    }

    let signal_score = (detected_signals.len().min(6) as i32) * 7;
    if signal_score > 0 {
        score += signal_score;
        score_reasons.push(format!(
            "detected {} high-signal topic(s): {}",
            detected_signals.len(),
            detected_signals.join(", ")
        ));
    }

    if headings.iter().any(|heading| heading.contains("IMPORTANT")) {
        score += 10;
        score_reasons.push("heading contains IMPORTANT".to_string());
    }
    if content_region.trim().is_empty() {
        score -= 15;
        score_reasons.push("page has no chunk content excerpt".to_string());
    }
    if recovery_hint.as_deref() == Some("unknown_source") {
        score -= 4;
        score_reasons.push("source is unknown, so provenance needs extra care".to_string());
    }

    let priority = if score >= 70 {
        DocumentOrphanQuarantineReviewPriority::High
    } else if score >= 40 {
        DocumentOrphanQuarantineReviewPriority::Medium
    } else {
        DocumentOrphanQuarantineReviewPriority::Low
    };
    let suggested_next_step = match priority {
        DocumentOrphanQuarantineReviewPriority::High => {
            DocumentOrphanQuarantineReviewSuggestedStep::InspectForMemoryPromotion
        }
        DocumentOrphanQuarantineReviewPriority::Medium => {
            DocumentOrphanQuarantineReviewSuggestedStep::InspectForArchiveOrRetention
        }
        DocumentOrphanQuarantineReviewPriority::Low => {
            DocumentOrphanQuarantineReviewSuggestedStep::Defer
        }
    };
    let title_hint = headings
        .iter()
        .find(|heading| meaningful_heading(heading))
        .cloned();
    let (excerpt, _) = truncate_to_byte_limit(
        &compact_review_excerpt(&content_region),
        options.max_excerpt_bytes,
    );

    Some(DocumentOrphanQuarantineReviewPriorityItem {
        relative_path: page.relative_path,
        missing_source_id,
        status: page.state,
        decision: page.decision,
        priority,
        score,
        suggested_next_step,
        orphan_chunk_count,
        exported_chunk_count,
        recovery_class,
        recovery_hint,
        reason,
        content_fingerprint,
        fingerprint_group_size: 1,
        fingerprint_group_rank: 1,
        fingerprint_duplicate_paths: Vec::new(),
        title_hint,
        excerpt,
        score_reasons,
        detected_signals,
    })
}

fn annotate_quarantine_review_fingerprint_groups(
    candidates: &mut [DocumentOrphanQuarantineReviewPriorityItem],
) -> (usize, usize) {
    let mut groups: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (index, item) in candidates.iter().enumerate() {
        if let Some(fingerprint) = normalized_review_fingerprint(item) {
            groups.entry(fingerprint).or_default().push(index);
        }
    }

    let mut duplicate_group_count = 0;
    let mut duplicate_candidate_count = 0;
    for indexes in groups.values() {
        let group_size = indexes.len();
        if group_size < 2 {
            continue;
        }

        duplicate_group_count += 1;
        duplicate_candidate_count += group_size;
        let paths = indexes
            .iter()
            .map(|index| candidates[*index].relative_path.clone())
            .collect::<Vec<_>>();

        for (rank, index) in indexes.iter().enumerate() {
            let relative_path = candidates[*index].relative_path.clone();
            candidates[*index].fingerprint_group_size = group_size;
            candidates[*index].fingerprint_group_rank = rank + 1;
            candidates[*index].fingerprint_duplicate_paths = paths
                .iter()
                .filter(|path| path.as_str() != relative_path.as_str())
                .cloned()
                .collect();
            candidates[*index].score_reasons.push(format!(
                "same content fingerprint appears in {group_size} candidate pages"
            ));
        }
    }

    (duplicate_group_count, duplicate_candidate_count)
}

fn normalized_review_fingerprint(
    item: &DocumentOrphanQuarantineReviewPriorityItem,
) -> Option<String> {
    let fingerprint = item.content_fingerprint.as_deref()?.trim();
    if fingerprint.is_empty() {
        None
    } else {
        Some(fingerprint.to_string())
    }
}

fn summary_value(contents: &str, label: &str) -> Option<String> {
    let prefix = format!("- {label}:");
    contents.lines().find_map(|line| {
        let value = line.trim().strip_prefix(&prefix)?.trim();
        optional_review_value(value).map(clean_review_inline_value)
    })
}

fn chunk_headings(contents: &str) -> Vec<String> {
    contents
        .lines()
        .filter_map(|line| {
            let value = line.trim().strip_prefix("- Heading:")?.trim();
            optional_review_value(value).map(clean_review_inline_value)
        })
        .collect()
}

fn clean_review_inline_value(value: String) -> String {
    value
        .trim()
        .trim_matches('`')
        .trim_matches('"')
        .trim()
        .to_string()
}

fn quarantine_review_content_region(contents: &str) -> String {
    contents
        .split_once("## Chunk Contents")
        .map(|(_, content)| content)
        .unwrap_or("")
        .to_string()
}

fn review_priority_signals(contents: &str) -> Vec<String> {
    let lowercase = contents.to_lowercase();
    let signal_groups = [
        ("decision", ["decision", "decided", "rationale"].as_slice()),
        (
            "rule",
            ["rule", "must", "should", "always", "never"].as_slice(),
        ),
        (
            "preference",
            ["preference", "user likes", "user prefers", "tone"].as_slice(),
        ),
        (
            "limitation",
            ["limitation", "blocker", "known issue", "gotcha", "risk"].as_slice(),
        ),
        (
            "workflow",
            ["workflow", "runbook", "process", "steps", "next step"].as_slice(),
        ),
        (
            "architecture",
            ["architecture", "design", "component", "graph", "schema"].as_slice(),
        ),
        (
            "memory-os",
            ["memory os", "memory-os", "engram", "mcp", "migration"].as_slice(),
        ),
        (
            "debugging",
            ["debug", "eval", "incident", "hotdog", "staging"].as_slice(),
        ),
        ("important", ["important", "critical", "warning"].as_slice()),
    ];

    signal_groups
        .iter()
        .filter(|(_, needles)| needles.iter().any(|needle| lowercase.contains(needle)))
        .map(|(name, _)| (*name).to_string())
        .collect()
}

fn meaningful_heading(heading: &str) -> bool {
    let trimmed = heading.trim();
    !trimmed.is_empty() && trimmed != "#" && !trimmed.ends_with(" > ## CLI Tools")
}

fn compact_review_excerpt(contents: &str) -> String {
    contents
        .lines()
        .map(str::trim)
        .filter(|line| {
            !line.is_empty()
                && !line.starts_with("### Chunk ")
                && !line.starts_with("- Chunk ID:")
                && !line.starts_with("- Heading:")
                && !line.starts_with("- Start line:")
                && !line.starts_with("- End line:")
                && !line.starts_with("- Original bytes:")
                && !line.starts_with("- Content truncated:")
                && !line.starts_with("```")
        })
        .take(30)
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Debug, Clone)]
struct ParsedQuarantineReviewPage {
    relative_path: String,
    page_type: Option<String>,
    missing_source_id: Option<String>,
    decision: Option<DocumentOrphanQuarantineReviewDecision>,
    memory_kind: Option<String>,
    scope_type: Option<String>,
    scope_name: Option<String>,
    title: Option<String>,
    notes: Option<String>,
    state: DocumentOrphanQuarantineReviewFileState,
    message: Option<String>,
}

impl ParsedQuarantineReviewPage {
    fn into_file_status(self) -> DocumentOrphanQuarantineReviewFileStatus {
        DocumentOrphanQuarantineReviewFileStatus {
            relative_path: self.relative_path,
            missing_source_id: self.missing_source_id,
            decision: self.decision,
            memory_kind: self.memory_kind,
            scope_type: self.scope_type,
            scope_name: self.scope_name,
            title: self.title,
            notes: self.notes,
            status: self.state,
            message: self.message,
        }
    }
}

fn parse_quarantine_review_page(contents: &str, relative_path: &str) -> ParsedQuarantineReviewPage {
    if !contents.contains(DOCUMENT_ORPHAN_QUARANTINE_REVIEW_MARKER) {
        return ParsedQuarantineReviewPage {
            relative_path: relative_path.to_string(),
            page_type: None,
            missing_source_id: None,
            decision: None,
            memory_kind: None,
            scope_type: None,
            scope_name: None,
            title: None,
            notes: None,
            state: DocumentOrphanQuarantineReviewFileState::UserOwned,
            message: Some("non-generated markdown file skipped".to_string()),
        };
    }

    let frontmatter = frontmatter_fields(contents);
    let page_type = frontmatter.get("page_type").cloned();
    if page_type.as_deref() != Some("document_orphan_quarantine_review_group") {
        return ParsedQuarantineReviewPage {
            relative_path: relative_path.to_string(),
            page_type,
            missing_source_id: frontmatter.get("missing_source_id").cloned(),
            decision: None,
            memory_kind: None,
            scope_type: None,
            scope_name: None,
            title: None,
            notes: None,
            state: DocumentOrphanQuarantineReviewFileState::NonGroupGenerated,
            message: None,
        };
    }

    let missing_source_id = frontmatter.get("missing_source_id").cloned();
    if match missing_source_id.as_deref() {
        Some(source_id) => Id::parse(source_id).is_err(),
        None => true,
    } {
        return ParsedQuarantineReviewPage {
            relative_path: relative_path.to_string(),
            page_type,
            missing_source_id,
            decision: None,
            memory_kind: None,
            scope_type: None,
            scope_name: None,
            title: None,
            notes: None,
            state: DocumentOrphanQuarantineReviewFileState::ParseError,
            message: Some("missing or invalid missing_source_id frontmatter".to_string()),
        };
    }

    let fields = quarantine_review_decision_fields(contents);
    let decision_value = fields
        .get("decision")
        .and_then(|value| optional_review_value(value));
    let Some(decision_value) = decision_value else {
        return ParsedQuarantineReviewPage {
            relative_path: relative_path.to_string(),
            page_type,
            missing_source_id,
            decision: Some(DocumentOrphanQuarantineReviewDecision::Pending),
            memory_kind: fields
                .get("memory_kind")
                .and_then(|value| optional_review_value(value)),
            scope_type: fields
                .get("scope_type")
                .and_then(|value| optional_review_value(value)),
            scope_name: fields
                .get("scope_name")
                .and_then(|value| optional_review_value(value)),
            title: fields
                .get("title")
                .and_then(|value| optional_review_value(value)),
            notes: fields
                .get("notes")
                .and_then(|value| optional_review_value(value)),
            state: DocumentOrphanQuarantineReviewFileState::Pending,
            message: Some("decision is pending or missing".to_string()),
        };
    };

    let Some(decision) = parse_quarantine_review_decision(&decision_value) else {
        return ParsedQuarantineReviewPage {
            relative_path: relative_path.to_string(),
            page_type,
            missing_source_id,
            decision: None,
            memory_kind: fields
                .get("memory_kind")
                .and_then(|value| optional_review_value(value)),
            scope_type: fields
                .get("scope_type")
                .and_then(|value| optional_review_value(value)),
            scope_name: fields
                .get("scope_name")
                .and_then(|value| optional_review_value(value)),
            title: fields
                .get("title")
                .and_then(|value| optional_review_value(value)),
            notes: fields
                .get("notes")
                .and_then(|value| optional_review_value(value)),
            state: DocumentOrphanQuarantineReviewFileState::Invalid,
            message: Some(format!("invalid decision `{decision_value}`")),
        };
    };

    let memory_kind = fields
        .get("memory_kind")
        .and_then(|value| optional_review_value(value));
    let scope_type = fields
        .get("scope_type")
        .and_then(|value| optional_review_value(value));
    let scope_name = fields
        .get("scope_name")
        .and_then(|value| optional_review_value(value));
    let title = fields
        .get("title")
        .and_then(|value| optional_review_value(value));
    let notes = fields
        .get("notes")
        .and_then(|value| optional_review_value(value));

    if decision == DocumentOrphanQuarantineReviewDecision::Pending {
        return ParsedQuarantineReviewPage {
            relative_path: relative_path.to_string(),
            page_type,
            missing_source_id,
            decision: Some(decision),
            memory_kind,
            scope_type,
            scope_name,
            title,
            notes,
            state: DocumentOrphanQuarantineReviewFileState::Pending,
            message: Some("decision is pending".to_string()),
        };
    }

    if let Err(message) = validate_quarantine_review_decision(
        decision,
        &memory_kind,
        &scope_type,
        &scope_name,
        &title,
    ) {
        return ParsedQuarantineReviewPage {
            relative_path: relative_path.to_string(),
            page_type,
            missing_source_id,
            decision: Some(decision),
            memory_kind,
            scope_type,
            scope_name,
            title,
            notes,
            state: DocumentOrphanQuarantineReviewFileState::Invalid,
            message: Some(message),
        };
    }

    ParsedQuarantineReviewPage {
        relative_path: relative_path.to_string(),
        page_type,
        missing_source_id,
        decision: Some(decision),
        memory_kind,
        scope_type,
        scope_name,
        title,
        notes,
        state: DocumentOrphanQuarantineReviewFileState::Decided,
        message: None,
    }
}

fn validate_quarantine_review_decision(
    decision: DocumentOrphanQuarantineReviewDecision,
    memory_kind: &Option<String>,
    scope_type: &Option<String>,
    scope_name: &Option<String>,
    title: &Option<String>,
) -> Result<(), String> {
    if decision != DocumentOrphanQuarantineReviewDecision::PromoteToMemoryReview {
        return Ok(());
    }

    let memory_kind = memory_kind
        .as_deref()
        .ok_or_else(|| "promote_to_memory_review requires memory_kind".to_string())?;
    if !is_allowed_quarantine_memory_kind(memory_kind) {
        return Err(format!("unsupported memory_kind `{memory_kind}`"));
    }

    let scope_type = scope_type
        .as_deref()
        .ok_or_else(|| "promote_to_memory_review requires scope_type".to_string())?;
    if !is_allowed_quarantine_scope_type(scope_type) {
        return Err(format!("unsupported scope_type `{scope_type}`"));
    }

    if !matches!(scope_type, "global" | "user") && scope_name.as_deref().is_none() {
        return Err(format!("scope_type `{scope_type}` requires scope_name"));
    }

    if title.as_deref().is_none() {
        return Err("promote_to_memory_review requires title".to_string());
    }

    Ok(())
}

fn is_allowed_quarantine_memory_kind(value: &str) -> bool {
    matches!(
        value,
        "preference"
            | "rule"
            | "decision"
            | "limitation"
            | "project_fact"
            | "repository_fact"
            | "task_fact"
            | "user_fact"
            | "session_insight"
    )
}

fn is_allowed_quarantine_scope_type(value: &str) -> bool {
    matches!(
        value,
        "global" | "user" | "project" | "task" | "entity" | "repository" | "session" | "custom"
    )
}

fn parse_quarantine_review_decision(value: &str) -> Option<DocumentOrphanQuarantineReviewDecision> {
    match value.trim().to_lowercase().as_str() {
        "pending" => Some(DocumentOrphanQuarantineReviewDecision::Pending),
        "retain_quarantine" | "retain-quarantine" | "retain quarantine" => {
            Some(DocumentOrphanQuarantineReviewDecision::RetainQuarantine)
        }
        "promote_to_memory_review" | "promote-to-memory-review" | "promote to memory review" => {
            Some(DocumentOrphanQuarantineReviewDecision::PromoteToMemoryReview)
        }
        "archive_legacy" | "archive-legacy" | "archive legacy" => {
            Some(DocumentOrphanQuarantineReviewDecision::ArchiveLegacy)
        }
        "delete_later" | "delete-later" | "delete later" => {
            Some(DocumentOrphanQuarantineReviewDecision::DeleteLater)
        }
        _ => None,
    }
}

fn quarantine_review_decision_fields(contents: &str) -> BTreeMap<String, String> {
    let Some(block) = quarantine_review_decision_yaml(contents) else {
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

fn quarantine_review_decision_yaml(contents: &str) -> Option<&str> {
    let heading_start = contents.find("## Review Decision")?;
    let after_heading = &contents[heading_start..];
    let fence_start = after_heading.find("```yaml")?;
    let after_fence = &after_heading[fence_start + "```yaml".len()..];
    let block = after_fence.strip_prefix('\n').unwrap_or(after_fence);
    let fence_end = closing_fence_index(block)?;
    Some(block[..fence_end].trim())
}

fn frontmatter_fields(contents: &str) -> BTreeMap<String, String> {
    let Some(frontmatter) = frontmatter_block(contents) else {
        return BTreeMap::new();
    };

    frontmatter
        .lines()
        .filter_map(|line| {
            let (key, value) = line.split_once(':')?;
            let key = key.trim();
            if key.is_empty() {
                return None;
            }
            optional_review_value(value).map(|value| (key.to_string(), value))
        })
        .collect()
}

fn frontmatter_block(contents: &str) -> Option<&str> {
    let rest = contents.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    Some(&rest[..end])
}

fn optional_review_value(value: &str) -> Option<String> {
    let value = strip_inline_comment(value).trim();
    if value.is_empty() || value.eq_ignore_ascii_case("null") {
        return None;
    }
    if value.starts_with('"') && value.ends_with('"') {
        return serde_json::from_str::<String>(value).ok();
    }
    Some(value.to_string())
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

fn collect_markdown_files(root: &Path) -> IndexResult<Vec<PathBuf>> {
    if !root.exists() {
        return Err(IndexError::FileNotFound(root.display().to_string()));
    }
    let mut files = Vec::new();
    collect_markdown_files_inner(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_markdown_files_inner(path: &Path, files: &mut Vec<PathBuf>) -> IndexResult<()> {
    if path.is_file() {
        if path.extension().is_some_and(|extension| extension == "md") {
            files.push(path.to_path_buf());
        }
        return Ok(());
    }

    for entry in fs::read_dir(path)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_markdown_files_inner(&path, files)?;
        } else if path.extension().is_some_and(|extension| extension == "md") {
            files.push(path);
        }
    }
    Ok(())
}

fn relative_path_string(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn closing_fence_index(block: &str) -> Option<usize> {
    let mut offset = 0;
    for line in block.split_inclusive('\n') {
        let line_without_newline = line.trim_end_matches('\n').trim_end_matches('\r');
        if line_without_newline.trim().starts_with("```") {
            return Some(offset);
        }
        offset += line.len();
    }
    None
}

fn build_reindex_plan_from_report(report: &DocumentOrphanReport) -> DocumentReindexPlan {
    let mut by_source: BTreeMap<
        (DocumentReindexAction, String, String),
        DocumentReindexSourcePlan,
    > = BTreeMap::new();
    let mut review_only = Vec::new();

    for group in &report.groups {
        if group.recovery_class != DocumentRecoveryClass::Recoverable {
            continue;
        }

        if let Some(candidate) = select_reindex_candidate(group) {
            let action = match candidate.match_type.as_str() {
                "digest_reviewed_source" => DocumentReindexAction::ReindexDigestReviewedSource,
                _ => DocumentReindexAction::ReindexFile,
            };
            let key = (action, candidate.path.clone(), candidate.match_type.clone());
            let entry = by_source
                .entry(key)
                .or_insert_with(|| DocumentReindexSourcePlan {
                    action,
                    source_path: candidate.path.clone(),
                    match_type: candidate.match_type.clone(),
                    group_count: 0,
                    orphan_chunk_count: 0,
                    max_score: candidate.score,
                    min_score: candidate.score,
                    existing_source_ids: Vec::new(),
                    groups: Vec::new(),
                    notes: reindex_notes(action),
                });
            entry.group_count += 1;
            entry.orphan_chunk_count += group.chunk_count;
            entry.max_score = entry.max_score.max(candidate.score);
            entry.min_score = entry.min_score.min(candidate.score);
            for source_id in existing_source_ids(group) {
                if !entry.existing_source_ids.contains(&source_id) {
                    entry.existing_source_ids.push(source_id);
                }
            }
            entry.groups.push(DocumentReindexGroupRef {
                missing_source_id: group.missing_source_id.clone(),
                orphan_chunk_count: group.chunk_count,
                recovery_hint: group.recovery_hint.clone(),
                score: candidate.score,
                matched_anchors: candidate.matched_anchors,
                total_anchors: candidate.total_anchors,
                exact_fingerprint_match: candidate.exact_fingerprint_match,
                evidence: candidate.evidence.clone(),
            });
        } else if let Some((source_id, source_path)) = existing_source_reference(group) {
            let action = DocumentReindexAction::InspectExistingSource;
            let key = (action, source_path.clone(), "existing_source".to_string());
            let entry = by_source
                .entry(key)
                .or_insert_with(|| DocumentReindexSourcePlan {
                    action,
                    source_path,
                    match_type: "existing_source".to_string(),
                    group_count: 0,
                    orphan_chunk_count: 0,
                    max_score: 1.0,
                    min_score: 1.0,
                    existing_source_ids: Vec::new(),
                    groups: Vec::new(),
                    notes: reindex_notes(action),
                });
            entry.group_count += 1;
            entry.orphan_chunk_count += group.chunk_count;
            if !entry.existing_source_ids.contains(&source_id) {
                entry.existing_source_ids.push(source_id);
            }
            entry.groups.push(DocumentReindexGroupRef {
                missing_source_id: group.missing_source_id.clone(),
                orphan_chunk_count: group.chunk_count,
                recovery_hint: group.recovery_hint.clone(),
                score: 1.0,
                matched_anchors: 0,
                total_anchors: group.content_anchor_count,
                exact_fingerprint_match: false,
                evidence: Vec::new(),
            });
        } else {
            review_only.push(DocumentReindexReviewOnlyGroup {
                missing_source_id: group.missing_source_id.clone(),
                orphan_chunk_count: group.chunk_count,
                recovery_hint: group.recovery_hint.clone(),
                reason: "recoverable group has no candidate match or existing source reference"
                    .to_string(),
            });
        }
    }

    let mut sources = by_source.into_values().collect::<Vec<_>>();
    sources.sort_by(|left, right| {
        right
            .orphan_chunk_count
            .cmp(&left.orphan_chunk_count)
            .then_with(|| left.source_path.cmp(&right.source_path))
    });
    for source in &mut sources {
        source.existing_source_ids.sort();
        source.groups.sort_by(|left, right| {
            right
                .orphan_chunk_count
                .cmp(&left.orphan_chunk_count)
                .then_with(|| left.missing_source_id.cmp(&right.missing_source_id))
        });
    }

    let planned_groups = sources.iter().map(|source| source.group_count).sum();
    let planned_orphan_chunks = sources.iter().map(|source| source.orphan_chunk_count).sum();

    DocumentReindexPlan {
        read_only: true,
        orphan_chunk_count: report.orphan_chunk_count,
        orphan_source_count: report.orphan_source_count,
        recoverable_groups: report.recovery_summary.recoverable,
        unknown_groups: report.recovery_summary.unknown,
        safe_to_quarantine_groups: report.recovery_summary.safe_to_quarantine,
        planned_groups,
        planned_orphan_chunks,
        review_only_groups: review_only.len(),
        sources,
        review_only,
    }
}

fn select_reindex_candidate(
    group: &DocumentOrphanGroup,
) -> Option<&DocumentRecoveryCandidateMatch> {
    let mut candidates = group.candidate_matches.iter().collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        candidate_preference(right)
            .cmp(&candidate_preference(left))
            .then_with(|| right.score.total_cmp(&left.score))
            .then_with(|| right.matched_anchors.cmp(&left.matched_anchors))
            .then_with(|| left.path.cmp(&right.path))
    });
    candidates.into_iter().next()
}

fn candidate_preference(candidate: &DocumentRecoveryCandidateMatch) -> u8 {
    match candidate.match_type.as_str() {
        "digest_reviewed_source" => 2,
        "file" => 1,
        _ => 0,
    }
}

fn existing_source_ids(group: &DocumentOrphanGroup) -> Vec<String> {
    group
        .detected_references
        .iter()
        .filter_map(|reference| reference.existing_source_id.clone())
        .collect()
}

fn existing_source_reference(group: &DocumentOrphanGroup) -> Option<(String, String)> {
    group.detected_references.iter().find_map(|reference| {
        reference
            .existing_source_id
            .as_ref()
            .map(|source_id| (source_id.clone(), reference.value.clone()))
    })
}

fn reindex_notes(action: DocumentReindexAction) -> Vec<String> {
    match action {
        DocumentReindexAction::ReindexFile => vec![
            "Read-only plan only; future execution must explicitly reindex this file before orphan cleanup.".to_string(),
        ],
        DocumentReindexAction::ReindexDigestReviewedSource => vec![
            "Use the digest source-index flow rather than raw file indexing so source metadata wrappers are preserved.".to_string(),
        ],
        DocumentReindexAction::InspectExistingSource => vec![
            "A current doc_source appears to cover this group; inspect existing chunks before reindexing or deleting orphan chunks.".to_string(),
        ],
    }
}

fn selected_reindex_source_indexes(
    plan: &DocumentReindexPlan,
    options: &DocumentReindexExecutionOptions,
) -> Vec<usize> {
    let selected_paths = options.source_paths.iter().collect::<BTreeSet<_>>();
    let selected_actions = options.actions.iter().copied().collect::<BTreeSet<_>>();

    plan.sources
        .iter()
        .enumerate()
        .filter_map(|(index, source)| {
            if !selected_paths.is_empty() && !selected_paths.contains(&source.source_path) {
                return None;
            }
            if !selected_actions.is_empty() && !selected_actions.contains(&source.action) {
                return None;
            }
            Some(index)
        })
        .collect()
}

fn collect_digest_source_documents(
    options: &DocumentReindexExecutionOptions,
) -> IndexResult<(BTreeMap<String, DigestSourceIndexDocument>, Vec<String>)> {
    let digest_service = DigestService::new();
    let mut documents = BTreeMap::new();
    let mut warnings = Vec::new();

    for review_path in &options.digest_review_paths {
        let plan = digest_service.plan_source_index(
            review_path,
            DigestSourceIndexOptions {
                max_source_bytes: options.max_source_bytes,
            },
        )?;
        warnings.extend(plan.warnings);
        warnings.extend(
            plan.sources_skipped
                .into_iter()
                .map(|warning| format!("{}: {}", plan.review_path, warning)),
        );
        for document in plan.documents {
            let document_path = document.document_path.clone();
            if documents.insert(document_path.clone(), document).is_some() {
                warnings.push(format!(
                    "duplicate digest source document path encountered: {document_path}"
                ));
            }
        }
    }

    Ok((documents, warnings))
}

#[derive(Debug, Clone)]
struct ReindexGroupCoverage {
    source_path: String,
    action: DocumentReindexAction,
    match_type: String,
    status: Option<DocumentReindexExecutionStatus>,
    status_reason: Option<String>,
}

fn build_orphan_cleanup_plan(
    report: &DocumentOrphanReport,
    reindex_plan: Option<&DocumentReindexPlan>,
    execution_report: Option<&DocumentReindexExecutionReport>,
) -> DocumentOrphanCleanupPlan {
    let coverage_by_group = reindex_group_coverage(reindex_plan, execution_report);
    let mut groups = Vec::new();
    let mut warnings = Vec::new();

    if report.groups_returned != report.orphan_source_count {
        warnings.push(format!(
            "cleanup plan is partial: {} of {} orphan source groups returned",
            report.groups_returned, report.orphan_source_count
        ));
    }
    if reindex_plan.is_none() {
        warnings.push(
            "no reindex plan supplied; recoverable groups cannot become delete candidates"
                .to_string(),
        );
    }
    if execution_report.is_none() {
        warnings.push(
            "no write execution report supplied; recoverable groups cannot become delete candidates"
                .to_string(),
        );
    }

    for group in &report.groups {
        groups.push(cleanup_group_plan(
            group,
            coverage_by_group.get(&group.missing_source_id),
        ));
    }

    groups.sort_by(|left, right| {
        cleanup_action_sort_key(left.cleanup_action)
            .cmp(&cleanup_action_sort_key(right.cleanup_action))
            .then_with(|| right.orphan_chunk_count.cmp(&left.orphan_chunk_count))
            .then_with(|| left.missing_source_id.cmp(&right.missing_source_id))
    });

    let delete_candidate_groups = groups
        .iter()
        .filter(|group| {
            group.cleanup_action == DocumentOrphanCleanupAction::DeleteAfterSuccessfulReindex
        })
        .count();
    let delete_candidate_chunks = groups
        .iter()
        .filter(|group| {
            group.cleanup_action == DocumentOrphanCleanupAction::DeleteAfterSuccessfulReindex
        })
        .map(|group| group.orphan_chunk_count)
        .sum();
    let quarantine_candidate_groups = groups
        .iter()
        .filter(|group| group.cleanup_action == DocumentOrphanCleanupAction::Quarantine)
        .count();
    let quarantine_candidate_chunks = groups
        .iter()
        .filter(|group| group.cleanup_action == DocumentOrphanCleanupAction::Quarantine)
        .map(|group| group.orphan_chunk_count)
        .sum();
    let manual_review_groups = groups
        .iter()
        .filter(|group| group.cleanup_action == DocumentOrphanCleanupAction::ManualReview)
        .count();
    let manual_review_chunks = groups
        .iter()
        .filter(|group| group.cleanup_action == DocumentOrphanCleanupAction::ManualReview)
        .map(|group| group.orphan_chunk_count)
        .sum();

    DocumentOrphanCleanupPlan {
        read_only: true,
        orphan_chunk_count: report.orphan_chunk_count,
        orphan_source_count: report.orphan_source_count,
        groups_returned: report.groups_returned,
        recoverable_groups: report.recovery_summary.recoverable,
        unknown_groups: report.recovery_summary.unknown,
        safe_to_quarantine_groups: report.recovery_summary.safe_to_quarantine,
        delete_candidate_groups,
        delete_candidate_chunks,
        quarantine_candidate_groups,
        quarantine_candidate_chunks,
        manual_review_groups,
        manual_review_chunks,
        groups,
        warnings,
    }
}

fn reindex_group_coverage(
    reindex_plan: Option<&DocumentReindexPlan>,
    execution_report: Option<&DocumentReindexExecutionReport>,
) -> BTreeMap<String, ReindexGroupCoverage> {
    let execution_by_source = execution_report
        .map(|report| {
            report
                .actions
                .iter()
                .map(|action| {
                    (
                        (
                            action.action,
                            action.source_path.clone(),
                            action.match_type.clone(),
                        ),
                        action,
                    )
                })
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    let mut coverage = BTreeMap::new();
    let Some(reindex_plan) = reindex_plan else {
        return coverage;
    };

    for source in &reindex_plan.sources {
        let execution = execution_by_source.get(&(
            source.action,
            source.source_path.clone(),
            source.match_type.clone(),
        ));
        for group in &source.groups {
            coverage.insert(
                group.missing_source_id.clone(),
                ReindexGroupCoverage {
                    source_path: source.source_path.clone(),
                    action: source.action,
                    match_type: source.match_type.clone(),
                    status: execution.map(|action| action.status),
                    status_reason: execution.and_then(|action| action.reason.clone()),
                },
            );
        }
    }

    coverage
}

fn cleanup_group_plan(
    group: &DocumentOrphanGroup,
    coverage: Option<&ReindexGroupCoverage>,
) -> DocumentOrphanCleanupGroupPlan {
    let (cleanup_action, reason) = match group.recovery_class {
        DocumentRecoveryClass::Recoverable => {
            if let Some(coverage) = coverage {
                match coverage.status {
                    Some(
                        DocumentReindexExecutionStatus::Reindexed
                        | DocumentReindexExecutionStatus::AlreadyIndexed,
                    ) => (
                        DocumentOrphanCleanupAction::DeleteAfterSuccessfulReindex,
                        format!(
                            "recoverable group covered by successful {} reindex action with status {}",
                            coverage.match_type,
                            reindex_execution_status_name(coverage.status)
                        ),
                    ),
                    Some(status) => (
                        DocumentOrphanCleanupAction::ManualReview,
                        format!(
                            "recoverable group has reindex coverage but execution status is {}; {}",
                            reindex_execution_status_name(Some(status)),
                            coverage
                                .status_reason
                                .as_deref()
                                .unwrap_or("manual review required")
                        ),
                    ),
                    None => (
                        DocumentOrphanCleanupAction::ManualReview,
                        "recoverable group has reindex plan coverage but no execution status"
                            .to_string(),
                    ),
                }
            } else {
                (
                    DocumentOrphanCleanupAction::ManualReview,
                    "recoverable group is not covered by a supplied successful reindex execution"
                        .to_string(),
                )
            }
        }
        DocumentRecoveryClass::Unknown => (
            DocumentOrphanCleanupAction::ManualReview,
            "group has source clues but no verified current source coverage".to_string(),
        ),
        DocumentRecoveryClass::SafeToQuarantine => (
            DocumentOrphanCleanupAction::Quarantine,
            "group has no source clues or candidate matches; quarantine review before deletion"
                .to_string(),
        ),
    };

    DocumentOrphanCleanupGroupPlan {
        cleanup_action,
        missing_source_id: group.missing_source_id.clone(),
        orphan_chunk_count: group.chunk_count,
        recovery_class: group.recovery_class,
        recovery_hint: group.recovery_hint.clone(),
        content_fingerprint: group.content_fingerprint.clone(),
        reason,
        reindex_source_path: coverage.map(|coverage| coverage.source_path.clone()),
        reindex_action: coverage.map(|coverage| coverage.action),
        reindex_status: coverage.and_then(|coverage| coverage.status),
        existing_source_ids: existing_source_ids(group),
        candidate_matches: group.candidate_matches.clone(),
        samples: group.samples.clone(),
    }
}

fn cleanup_action_sort_key(action: DocumentOrphanCleanupAction) -> u8 {
    match action {
        DocumentOrphanCleanupAction::DeleteAfterSuccessfulReindex => 0,
        DocumentOrphanCleanupAction::Quarantine => 1,
        DocumentOrphanCleanupAction::ManualReview => 2,
    }
}

fn reindex_execution_status_name(status: Option<DocumentReindexExecutionStatus>) -> &'static str {
    match status {
        Some(DocumentReindexExecutionStatus::Planned) => "planned",
        Some(DocumentReindexExecutionStatus::Reindexed) => "reindexed",
        Some(DocumentReindexExecutionStatus::AlreadyIndexed) => "already_indexed",
        Some(DocumentReindexExecutionStatus::RequiresInspection) => "requires_inspection",
        Some(DocumentReindexExecutionStatus::Skipped) => "skipped",
        Some(DocumentReindexExecutionStatus::Failed) => "failed",
        None => "missing",
    }
}

fn selected_cleanup_delete_ids(
    plan: &DocumentOrphanCleanupPlan,
    options: &DocumentOrphanCleanupExecutionOptions,
) -> Vec<String> {
    let selected_ids = options.missing_source_ids.iter().collect::<BTreeSet<_>>();
    let mut selected = Vec::new();

    for group in &plan.groups {
        if group.cleanup_action != DocumentOrphanCleanupAction::DeleteAfterSuccessfulReindex {
            continue;
        }
        if !selected_ids.is_empty() && !selected_ids.contains(&group.missing_source_id) {
            continue;
        }
        if options
            .max_groups
            .is_some_and(|max_groups| selected.len() >= max_groups)
        {
            break;
        }
        selected.push(group.missing_source_id.clone());
    }

    selected
}

fn cleanup_execution_action_for_dry_run(
    group: &DocumentOrphanCleanupGroupPlan,
    selected_delete_ids: &[String],
) -> DocumentOrphanCleanupExecutionAction {
    let selected_for_delete = selected_delete_ids
        .iter()
        .any(|source_id| source_id == &group.missing_source_id);
    match group.cleanup_action {
        DocumentOrphanCleanupAction::DeleteAfterSuccessfulReindex if selected_for_delete => {
            DocumentOrphanCleanupExecutionAction {
                missing_source_id: group.missing_source_id.clone(),
                cleanup_action: group.cleanup_action,
                status: DocumentOrphanCleanupExecutionStatus::PlannedDelete,
                dry_run: true,
                planned_orphan_chunks: group.orphan_chunk_count,
                deleted_chunks: 0,
                reason: "delete candidate selected; dry-run only".to_string(),
            }
        }
        DocumentOrphanCleanupAction::DeleteAfterSuccessfulReindex => {
            DocumentOrphanCleanupExecutionAction {
                missing_source_id: group.missing_source_id.clone(),
                cleanup_action: group.cleanup_action,
                status: DocumentOrphanCleanupExecutionStatus::Skipped,
                dry_run: true,
                planned_orphan_chunks: group.orphan_chunk_count,
                deleted_chunks: 0,
                reason: "delete candidate not selected by filters or max_groups".to_string(),
            }
        }
        DocumentOrphanCleanupAction::Quarantine => DocumentOrphanCleanupExecutionAction {
            missing_source_id: group.missing_source_id.clone(),
            cleanup_action: group.cleanup_action,
            status: DocumentOrphanCleanupExecutionStatus::QuarantineRetained,
            dry_run: true,
            planned_orphan_chunks: group.orphan_chunk_count,
            deleted_chunks: 0,
            reason: "quarantine group retained; this executor never deletes quarantine groups"
                .to_string(),
        },
        DocumentOrphanCleanupAction::ManualReview => DocumentOrphanCleanupExecutionAction {
            missing_source_id: group.missing_source_id.clone(),
            cleanup_action: group.cleanup_action,
            status: DocumentOrphanCleanupExecutionStatus::ManualReviewRequired,
            dry_run: true,
            planned_orphan_chunks: group.orphan_chunk_count,
            deleted_chunks: 0,
            reason: "manual-review group retained".to_string(),
        },
    }
}

fn apply_cleanup_delete_result(
    plan: &DocumentOrphanCleanupPlan,
    options: &DocumentOrphanCleanupExecutionOptions,
    delete_result: DocumentOrphanDeleteResult,
    report: &mut DocumentOrphanCleanupExecutionReport,
) {
    let selected_delete_ids = selected_cleanup_delete_ids(plan, options)
        .into_iter()
        .collect::<BTreeSet<_>>();
    let deleted_by_source = delete_result
        .deleted_sources
        .iter()
        .map(|source| (source.missing_source_id.clone(), source.deleted_chunks))
        .collect::<BTreeMap<_, _>>();
    let protected_source_ids = delete_result
        .protected_source_ids
        .iter()
        .collect::<BTreeSet<_>>();

    report.orphan_cleanup_performed = !selected_delete_ids.is_empty();
    report.deleted_chunks = delete_result.deleted_chunk_count;
    report.warnings.push(
        "quarantine groups were retained; export/review them separately before any deletion"
            .to_string(),
    );

    for group in &plan.groups {
        let action = match group.cleanup_action {
            DocumentOrphanCleanupAction::DeleteAfterSuccessfulReindex
                if protected_source_ids.contains(&group.missing_source_id) =>
            {
                DocumentOrphanCleanupExecutionAction {
                    missing_source_id: group.missing_source_id.clone(),
                    cleanup_action: group.cleanup_action,
                    status: DocumentOrphanCleanupExecutionStatus::Protected,
                    dry_run: false,
                    planned_orphan_chunks: group.orphan_chunk_count,
                    deleted_chunks: 0,
                    reason: "source ID now exists in doc_source; chunks were protected".to_string(),
                }
            }
            DocumentOrphanCleanupAction::DeleteAfterSuccessfulReindex
                if selected_delete_ids.contains(&group.missing_source_id) =>
            {
                let deleted_chunks = deleted_by_source
                    .get(&group.missing_source_id)
                    .copied()
                    .unwrap_or(0);
                DocumentOrphanCleanupExecutionAction {
                    missing_source_id: group.missing_source_id.clone(),
                    cleanup_action: group.cleanup_action,
                    status: DocumentOrphanCleanupExecutionStatus::Deleted,
                    dry_run: false,
                    planned_orphan_chunks: group.orphan_chunk_count,
                    deleted_chunks,
                    reason: "delete candidate executed; store deleted only still-orphaned chunks"
                        .to_string(),
                }
            }
            DocumentOrphanCleanupAction::DeleteAfterSuccessfulReindex => {
                DocumentOrphanCleanupExecutionAction {
                    missing_source_id: group.missing_source_id.clone(),
                    cleanup_action: group.cleanup_action,
                    status: DocumentOrphanCleanupExecutionStatus::Skipped,
                    dry_run: false,
                    planned_orphan_chunks: group.orphan_chunk_count,
                    deleted_chunks: 0,
                    reason: "delete candidate not selected by filters or max_groups".to_string(),
                }
            }
            DocumentOrphanCleanupAction::Quarantine => DocumentOrphanCleanupExecutionAction {
                missing_source_id: group.missing_source_id.clone(),
                cleanup_action: group.cleanup_action,
                status: DocumentOrphanCleanupExecutionStatus::QuarantineRetained,
                dry_run: false,
                planned_orphan_chunks: group.orphan_chunk_count,
                deleted_chunks: 0,
                reason: "quarantine group retained; this executor never deletes quarantine groups"
                    .to_string(),
            },
            DocumentOrphanCleanupAction::ManualReview => DocumentOrphanCleanupExecutionAction {
                missing_source_id: group.missing_source_id.clone(),
                cleanup_action: group.cleanup_action,
                status: DocumentOrphanCleanupExecutionStatus::ManualReviewRequired,
                dry_run: false,
                planned_orphan_chunks: group.orphan_chunk_count,
                deleted_chunks: 0,
                reason: "manual-review group retained".to_string(),
            },
        };
        report.actions.push(action);
    }

    refresh_cleanup_execution_summary(report);
}

fn refresh_cleanup_execution_summary(report: &mut DocumentOrphanCleanupExecutionReport) {
    report.planned_delete_groups = 0;
    report.planned_delete_chunks = 0;
    report.deleted_groups = 0;
    report.deleted_chunks = 0;
    report.quarantine_groups_retained = 0;
    report.quarantine_chunks_retained = 0;
    report.manual_review_groups = 0;
    report.skipped_groups = 0;
    report.protected_groups = 0;

    for action in &report.actions {
        match action.status {
            DocumentOrphanCleanupExecutionStatus::PlannedDelete => {
                report.planned_delete_groups += 1;
                report.planned_delete_chunks += action.planned_orphan_chunks;
            }
            DocumentOrphanCleanupExecutionStatus::Deleted => {
                report.deleted_groups += 1;
                report.deleted_chunks += action.deleted_chunks;
            }
            DocumentOrphanCleanupExecutionStatus::QuarantineRetained => {
                report.quarantine_groups_retained += 1;
                report.quarantine_chunks_retained += action.planned_orphan_chunks;
            }
            DocumentOrphanCleanupExecutionStatus::ManualReviewRequired => {
                report.manual_review_groups += 1;
            }
            DocumentOrphanCleanupExecutionStatus::Skipped => {
                report.skipped_groups += 1;
            }
            DocumentOrphanCleanupExecutionStatus::Protected => {
                report.protected_groups += 1;
            }
        }
    }
}

fn enrich_orphan_report_with_candidate_matches(
    report: &mut DocumentOrphanReport,
    options: &DocumentRecoveryOptions,
) -> IndexResult<()> {
    if options.scan_paths.is_empty() && options.digest_review_paths.is_empty() {
        return Ok(());
    }

    let mut warnings = Vec::new();
    let mut skipped = 0usize;
    let mut candidates = Vec::new();

    collect_file_candidates(options, &mut candidates, &mut skipped, &mut warnings)?;
    collect_digest_review_candidates(options, &mut candidates, &mut skipped, &mut warnings)?;

    if candidates.len() > options.max_candidate_files {
        skipped += candidates.len() - options.max_candidate_files;
        candidates.truncate(options.max_candidate_files);
    }

    for group in &mut report.groups {
        let total_anchors = group.content_anchors.len();
        if total_anchors == 0 {
            continue;
        }

        let mut matches = candidates
            .iter()
            .filter_map(|candidate| {
                match_candidate_to_group(
                    candidate,
                    &group.content_fingerprint,
                    &group.content_anchors,
                    options.min_match_score,
                )
            })
            .collect::<Vec<_>>();

        matches.sort_by(|left, right| {
            right
                .score
                .total_cmp(&left.score)
                .then_with(|| right.matched_anchors.cmp(&left.matched_anchors))
                .then_with(|| left.path.cmp(&right.path))
        });
        matches.truncate(options.max_matches_per_group);

        if !matches.is_empty() && group.recovery_hint == "unknown_source" {
            group.recovery_hint = "candidate_match".to_string();
        }
        group.candidate_matches = matches;
    }

    report.groups_with_candidate_matches = report
        .groups
        .iter()
        .filter(|group| !group.candidate_matches.is_empty())
        .count();
    report.candidate_files_scanned = candidates.len();
    report.candidate_files_skipped = skipped;
    report.candidate_scan_warnings = warnings;
    report.refresh_recovery_summary();
    Ok(())
}

fn match_candidate_to_group(
    candidate: &RecoveryCandidate,
    group_fingerprint: &str,
    anchors: &[String],
    min_match_score: f32,
) -> Option<DocumentRecoveryCandidateMatch> {
    let exact_fingerprint_match = candidate.fingerprint == group_fingerprint;
    let matched = anchors
        .iter()
        .filter(|anchor| candidate.normalized_content.contains(anchor.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let matched_anchors = matched.len();
    let total_anchors = anchors.len();
    let score = if exact_fingerprint_match {
        1.0
    } else {
        matched_anchors as f32 / total_anchors as f32
    };
    let required_matches = if total_anchors <= 2 { 1 } else { 2 };

    if !exact_fingerprint_match && (matched_anchors < required_matches || score < min_match_score) {
        return None;
    }

    Some(DocumentRecoveryCandidateMatch {
        match_type: candidate.match_type.clone(),
        path: candidate.path.clone(),
        score,
        matched_anchors,
        total_anchors,
        exact_fingerprint_match,
        evidence: matched
            .into_iter()
            .take(3)
            .map(|anchor| truncate_chars(&anchor, 160))
            .collect(),
    })
}

fn collect_file_candidates(
    options: &DocumentRecoveryOptions,
    candidates: &mut Vec<RecoveryCandidate>,
    skipped: &mut usize,
    warnings: &mut Vec<String>,
) -> IndexResult<()> {
    for path in &options.scan_paths {
        collect_path_candidates(path, options, candidates, skipped, warnings)?;
        if candidates.len() >= options.max_candidate_files {
            return Ok(());
        }
    }
    Ok(())
}

fn collect_path_candidates(
    path: &Path,
    options: &DocumentRecoveryOptions,
    candidates: &mut Vec<RecoveryCandidate>,
    skipped: &mut usize,
    warnings: &mut Vec<String>,
) -> IndexResult<()> {
    if candidates.len() >= options.max_candidate_files {
        return Ok(());
    }
    if path.is_file() {
        if is_recovery_candidate_file(path) {
            match read_recovery_candidate(path, "file", options.max_file_bytes) {
                Ok(Some(candidate)) => candidates.push(candidate),
                Ok(None) => *skipped += 1,
                Err(error) => {
                    *skipped += 1;
                    warnings.push(format!("{}: {}", path.display(), error));
                }
            }
        } else {
            *skipped += 1;
        }
        return Ok(());
    }
    if !path.is_dir() {
        warnings.push(format!("scan path not found: {}", path.display()));
        return Ok(());
    }

    let entries = fs::read_dir(path)?;
    for entry in entries {
        let entry = entry?;
        let child = entry.path();
        if child.is_dir() {
            if should_skip_recovery_dir(&child) {
                continue;
            }
            collect_path_candidates(&child, options, candidates, skipped, warnings)?;
        } else {
            collect_path_candidates(&child, options, candidates, skipped, warnings)?;
        }
        if candidates.len() >= options.max_candidate_files {
            return Ok(());
        }
    }
    Ok(())
}

fn collect_digest_review_candidates(
    options: &DocumentRecoveryOptions,
    candidates: &mut Vec<RecoveryCandidate>,
    skipped: &mut usize,
    warnings: &mut Vec<String>,
) -> IndexResult<()> {
    let digest_service = DigestService::new();
    for review_path in &options.digest_review_paths {
        let reviewed = match digest_service.apply_review_batch(review_path) {
            Ok(reviewed) => reviewed,
            Err(error) => {
                warnings.push(format!("{}: {}", review_path.display(), error));
                continue;
            }
        };
        for source in reviewed.planned_sources {
            if candidates.len() >= options.max_candidate_files {
                return Ok(());
            }
            let source_path = Path::new(&source.candidate.absolute_path);
            match read_recovery_candidate(
                source_path,
                "digest_reviewed_source",
                options.max_file_bytes,
            ) {
                Ok(Some(candidate)) => candidates.push(candidate),
                Ok(None) => *skipped += 1,
                Err(error) => {
                    *skipped += 1;
                    warnings.push(format!(
                        "{} ({}): {}",
                        source.candidate.absolute_path, source.review_path, error
                    ));
                }
            }
        }
    }
    Ok(())
}

fn read_recovery_candidate(
    path: &Path,
    match_type: &str,
    max_file_bytes: usize,
) -> IndexResult<Option<RecoveryCandidate>> {
    let metadata = fs::metadata(path)?;
    if metadata.len() as usize > max_file_bytes {
        return Ok(None);
    }

    let bytes = fs::read(path)?;
    let content = String::from_utf8_lossy(&bytes);
    let normalized_content = normalize_for_fingerprint(&content);
    if normalized_content.is_empty() {
        return Ok(None);
    }
    let fingerprint = stable_fingerprint(&normalized_content);

    Ok(Some(RecoveryCandidate {
        match_type: match_type.to_string(),
        path: path.display().to_string(),
        normalized_content,
        fingerprint,
    }))
}

fn is_recovery_candidate_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "md" | "markdown" | "txt" | "json" | "jsonl" | "yaml" | "yml" | "csv"
            )
        })
        .unwrap_or(false)
}

fn should_skip_recovery_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| {
            matches!(
                name,
                ".git" | "target" | "node_modules" | ".venv" | "venv" | ".direnv"
            )
        })
        .unwrap_or(false)
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    let mut truncated = value.chars().take(max_chars).collect::<String>();
    if value.chars().count() > max_chars {
        truncated.push_str("...");
    }
    truncated
}

/// Statistics about the document index.
#[derive(Debug, Clone)]
pub struct DocumentStats {
    /// Number of indexed document sources.
    pub source_count: u64,
    /// Number of document chunks.
    pub chunk_count: u64,
    /// Number of chunks attached to a persisted source.
    pub searchable_chunk_count: u64,
    /// Number of chunks whose source record is missing.
    pub orphan_chunk_count: u64,
    /// Embedding vector dimension.
    pub embedding_dimension: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use engram_store::{
        DocumentDetectedReference, DocumentOrphanGroup, DocumentOrphanReport,
        DocumentRecoveryCandidateMatch, DocumentRecoveryClass, DocumentRecoverySummary,
    };
    use std::fs;

    #[test]
    fn recovery_matching_finds_current_file_by_content_anchor() {
        let dir = tempfile::TempDir::new().unwrap();
        let candidate_path = dir.path().join("candidate.md");
        fs::write(
            &candidate_path,
            "# Candidate\n\nThis document contains a durable recovery anchor about dynamic mcp authentication for long running agents and session renewal.",
        )
        .unwrap();

        let anchor =
            "durable recovery anchor about dynamic mcp authentication for long running agents"
                .to_string();
        let mut report = DocumentOrphanReport {
            orphan_chunk_count: 1,
            orphan_source_count: 1,
            groups_returned: 1,
            sample_limit_per_group: 1,
            recovery_summary: DocumentRecoverySummary {
                recoverable: 0,
                unknown: 0,
                safe_to_quarantine: 1,
            },
            groups_with_known_source_match: 0,
            groups_with_candidate_matches: 0,
            candidate_files_scanned: 0,
            candidate_files_skipped: 0,
            candidate_scan_warnings: Vec::new(),
            groups: vec![DocumentOrphanGroup {
                missing_source_id: "missing-source".to_string(),
                chunk_count: 1,
                recovery_class: DocumentRecoveryClass::SafeToQuarantine,
                recovery_hint: "unknown_source".to_string(),
                content_fingerprint: "not-an-exact-match".to_string(),
                content_anchor_count: 1,
                content_anchors: vec![anchor],
                detected_references: Vec::new(),
                candidate_matches: Vec::new(),
                samples: Vec::new(),
            }],
        };

        enrich_orphan_report_with_candidate_matches(
            &mut report,
            &DocumentRecoveryOptions {
                scan_paths: vec![dir.path().to_path_buf()],
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(report.candidate_files_scanned, 1);
        assert_eq!(report.groups_with_candidate_matches, 1);
        assert_eq!(report.recovery_summary.recoverable, 1);
        assert_eq!(
            report.groups[0].recovery_class,
            DocumentRecoveryClass::Recoverable
        );
        assert_eq!(report.groups[0].recovery_hint, "candidate_match");
        assert_eq!(report.groups[0].candidate_matches.len(), 1);
        assert_eq!(
            report.groups[0].candidate_matches[0].path,
            candidate_path.display().to_string()
        );
    }

    #[test]
    fn reindex_plan_dedupes_groups_by_selected_source() {
        let report = DocumentOrphanReport {
            orphan_chunk_count: 7,
            orphan_source_count: 2,
            groups_returned: 2,
            sample_limit_per_group: 1,
            recovery_summary: DocumentRecoverySummary {
                recoverable: 2,
                unknown: 0,
                safe_to_quarantine: 0,
            },
            groups_with_known_source_match: 0,
            groups_with_candidate_matches: 2,
            candidate_files_scanned: 1,
            candidate_files_skipped: 0,
            candidate_scan_warnings: Vec::new(),
            groups: vec![
                recoverable_group_with_candidate("missing-a", 3, "file", "/tmp/source.md", 0.75),
                recoverable_group_with_candidate("missing-b", 4, "file", "/tmp/source.md", 0.95),
            ],
        };

        let plan = build_reindex_plan_from_report(&report);

        assert!(plan.read_only);
        assert_eq!(plan.planned_groups, 2);
        assert_eq!(plan.planned_orphan_chunks, 7);
        assert_eq!(plan.sources.len(), 1);
        assert_eq!(plan.sources[0].action, DocumentReindexAction::ReindexFile);
        assert_eq!(plan.sources[0].source_path, "/tmp/source.md");
        assert_eq!(plan.sources[0].group_count, 2);
        assert_eq!(plan.sources[0].orphan_chunk_count, 7);
        assert_eq!(plan.sources[0].min_score, 0.75);
        assert_eq!(plan.sources[0].max_score, 0.95);
    }

    #[test]
    fn reindex_plan_prefers_digest_reviewed_source_over_raw_file() {
        let mut group =
            recoverable_group_with_candidate("missing-digest", 5, "file", "/tmp/digest.md", 0.99);
        group.candidate_matches.push(candidate_match(
            "digest_reviewed_source",
            "/tmp/review.json",
            0.4,
        ));

        let report = DocumentOrphanReport {
            orphan_chunk_count: 5,
            orphan_source_count: 1,
            groups_returned: 1,
            sample_limit_per_group: 1,
            recovery_summary: DocumentRecoverySummary {
                recoverable: 1,
                unknown: 0,
                safe_to_quarantine: 0,
            },
            groups_with_known_source_match: 0,
            groups_with_candidate_matches: 1,
            candidate_files_scanned: 2,
            candidate_files_skipped: 0,
            candidate_scan_warnings: Vec::new(),
            groups: vec![group],
        };

        let plan = build_reindex_plan_from_report(&report);

        assert_eq!(plan.sources.len(), 1);
        assert_eq!(
            plan.sources[0].action,
            DocumentReindexAction::ReindexDigestReviewedSource
        );
        assert_eq!(plan.sources[0].source_path, "/tmp/review.json");
        assert_eq!(plan.sources[0].match_type, "digest_reviewed_source");
    }

    #[test]
    fn reindex_plan_tracks_existing_source_references_for_inspection() {
        let report = DocumentOrphanReport {
            orphan_chunk_count: 2,
            orphan_source_count: 1,
            groups_returned: 1,
            sample_limit_per_group: 1,
            recovery_summary: DocumentRecoverySummary {
                recoverable: 1,
                unknown: 0,
                safe_to_quarantine: 0,
            },
            groups_with_known_source_match: 1,
            groups_with_candidate_matches: 0,
            candidate_files_scanned: 0,
            candidate_files_skipped: 0,
            candidate_scan_warnings: Vec::new(),
            groups: vec![DocumentOrphanGroup {
                missing_source_id: "missing-existing".to_string(),
                chunk_count: 2,
                recovery_class: DocumentRecoveryClass::Recoverable,
                recovery_hint: "known_source_reference".to_string(),
                content_fingerprint: "fingerprint".to_string(),
                content_anchor_count: 0,
                content_anchors: Vec::new(),
                detected_references: vec![DocumentDetectedReference {
                    reference_type: "absolute_path".to_string(),
                    value: "/tmp/current.md".to_string(),
                    existing_source_id: Some("doc_source:current".to_string()),
                }],
                candidate_matches: Vec::new(),
                samples: Vec::new(),
            }],
        };

        let plan = build_reindex_plan_from_report(&report);

        assert_eq!(plan.sources.len(), 1);
        assert_eq!(
            plan.sources[0].action,
            DocumentReindexAction::InspectExistingSource
        );
        assert_eq!(plan.sources[0].source_path, "/tmp/current.md");
        assert_eq!(
            plan.sources[0].existing_source_ids,
            vec!["doc_source:current".to_string()]
        );
    }

    #[test]
    fn reindex_execution_selection_filters_by_action_and_source() {
        let report = DocumentOrphanReport {
            orphan_chunk_count: 8,
            orphan_source_count: 2,
            groups_returned: 2,
            sample_limit_per_group: 1,
            recovery_summary: DocumentRecoverySummary {
                recoverable: 2,
                unknown: 0,
                safe_to_quarantine: 0,
            },
            groups_with_known_source_match: 0,
            groups_with_candidate_matches: 2,
            candidate_files_scanned: 2,
            candidate_files_skipped: 0,
            candidate_scan_warnings: Vec::new(),
            groups: vec![
                recoverable_group_with_candidate("missing-a", 3, "file", "/tmp/source-a.md", 0.9),
                recoverable_group_with_candidate(
                    "missing-b",
                    5,
                    "digest_reviewed_source",
                    "/tmp/source-b.md",
                    0.9,
                ),
            ],
        };
        let plan = build_reindex_plan_from_report(&report);

        let selected = selected_reindex_source_indexes(
            &plan,
            &DocumentReindexExecutionOptions {
                source_paths: vec!["/tmp/source-b.md".to_string()],
                actions: vec![DocumentReindexAction::ReindexDigestReviewedSource],
                ..Default::default()
            },
        );

        assert_eq!(selected.len(), 1);
        assert_eq!(plan.sources[selected[0]].source_path, "/tmp/source-b.md");
    }

    #[test]
    fn reindex_action_parse_accepts_cli_and_mcp_labels() {
        assert_eq!(
            DocumentReindexAction::parse("reindex-file"),
            Some(DocumentReindexAction::ReindexFile)
        );
        assert_eq!(
            DocumentReindexAction::parse("digest_reviewed_source"),
            Some(DocumentReindexAction::ReindexDigestReviewedSource)
        );
        assert_eq!(
            DocumentReindexAction::parse("inspect-existing-source"),
            Some(DocumentReindexAction::InspectExistingSource)
        );
        assert_eq!(DocumentReindexAction::parse("unknown"), None);
    }

    #[test]
    fn cleanup_plan_marks_successfully_reindexed_recoverable_group_as_delete_candidate() {
        let report = DocumentOrphanReport {
            orphan_chunk_count: 3,
            orphan_source_count: 1,
            groups_returned: 1,
            sample_limit_per_group: 1,
            recovery_summary: DocumentRecoverySummary {
                recoverable: 1,
                unknown: 0,
                safe_to_quarantine: 0,
            },
            groups_with_known_source_match: 0,
            groups_with_candidate_matches: 1,
            candidate_files_scanned: 1,
            candidate_files_skipped: 0,
            candidate_scan_warnings: Vec::new(),
            groups: vec![recoverable_group_with_candidate(
                "missing-a",
                3,
                "file",
                "/tmp/source.md",
                0.9,
            )],
        };
        let reindex_plan = build_reindex_plan_from_report(&report);
        let execution_report =
            execution_report_for_plan(&reindex_plan, DocumentReindexExecutionStatus::Reindexed);

        let cleanup_plan =
            build_orphan_cleanup_plan(&report, Some(&reindex_plan), Some(&execution_report));

        assert_eq!(cleanup_plan.delete_candidate_groups, 1);
        assert_eq!(cleanup_plan.delete_candidate_chunks, 3);
        assert_eq!(cleanup_plan.quarantine_candidate_groups, 0);
        assert_eq!(
            cleanup_plan.groups[0].cleanup_action,
            DocumentOrphanCleanupAction::DeleteAfterSuccessfulReindex
        );
    }

    #[test]
    fn cleanup_plan_keeps_dry_run_reindex_coverage_in_manual_review() {
        let report = DocumentOrphanReport {
            orphan_chunk_count: 3,
            orphan_source_count: 1,
            groups_returned: 1,
            sample_limit_per_group: 1,
            recovery_summary: DocumentRecoverySummary {
                recoverable: 1,
                unknown: 0,
                safe_to_quarantine: 0,
            },
            groups_with_known_source_match: 0,
            groups_with_candidate_matches: 1,
            candidate_files_scanned: 1,
            candidate_files_skipped: 0,
            candidate_scan_warnings: Vec::new(),
            groups: vec![recoverable_group_with_candidate(
                "missing-a",
                3,
                "file",
                "/tmp/source.md",
                0.9,
            )],
        };
        let reindex_plan = build_reindex_plan_from_report(&report);
        let execution_report =
            execution_report_for_plan(&reindex_plan, DocumentReindexExecutionStatus::Planned);

        let cleanup_plan =
            build_orphan_cleanup_plan(&report, Some(&reindex_plan), Some(&execution_report));

        assert_eq!(cleanup_plan.delete_candidate_groups, 0);
        assert_eq!(cleanup_plan.manual_review_groups, 1);
        assert_eq!(
            cleanup_plan.groups[0].cleanup_action,
            DocumentOrphanCleanupAction::ManualReview
        );
    }

    #[test]
    fn cleanup_plan_marks_safe_to_quarantine_groups_as_quarantine_candidates() {
        let report = DocumentOrphanReport {
            orphan_chunk_count: 2,
            orphan_source_count: 1,
            groups_returned: 1,
            sample_limit_per_group: 1,
            recovery_summary: DocumentRecoverySummary {
                recoverable: 0,
                unknown: 0,
                safe_to_quarantine: 1,
            },
            groups_with_known_source_match: 0,
            groups_with_candidate_matches: 0,
            candidate_files_scanned: 0,
            candidate_files_skipped: 0,
            candidate_scan_warnings: Vec::new(),
            groups: vec![DocumentOrphanGroup {
                missing_source_id: "missing-safe".to_string(),
                chunk_count: 2,
                recovery_class: DocumentRecoveryClass::SafeToQuarantine,
                recovery_hint: "unknown_source".to_string(),
                content_fingerprint: "fingerprint".to_string(),
                content_anchor_count: 0,
                content_anchors: Vec::new(),
                detected_references: Vec::new(),
                candidate_matches: Vec::new(),
                samples: Vec::new(),
            }],
        };

        let cleanup_plan = build_orphan_cleanup_plan(&report, None, None);

        assert_eq!(cleanup_plan.quarantine_candidate_groups, 1);
        assert_eq!(cleanup_plan.quarantine_candidate_chunks, 2);
        assert_eq!(cleanup_plan.delete_candidate_groups, 0);
        assert_eq!(
            cleanup_plan.groups[0].cleanup_action,
            DocumentOrphanCleanupAction::Quarantine
        );
    }

    #[test]
    fn cleanup_execution_dry_run_plans_delete_candidates_and_retains_quarantine() {
        let mut plan = DocumentOrphanCleanupPlan {
            read_only: true,
            orphan_chunk_count: 5,
            orphan_source_count: 2,
            groups_returned: 2,
            recoverable_groups: 1,
            unknown_groups: 0,
            safe_to_quarantine_groups: 1,
            delete_candidate_groups: 1,
            delete_candidate_chunks: 3,
            quarantine_candidate_groups: 1,
            quarantine_candidate_chunks: 2,
            manual_review_groups: 0,
            manual_review_chunks: 0,
            groups: vec![
                cleanup_group(
                    "delete-source",
                    3,
                    DocumentOrphanCleanupAction::DeleteAfterSuccessfulReindex,
                ),
                cleanup_group(
                    "quarantine-source",
                    2,
                    DocumentOrphanCleanupAction::Quarantine,
                ),
            ],
            warnings: Vec::new(),
        };
        plan.groups.sort_by(|left, right| {
            cleanup_action_sort_key(left.cleanup_action)
                .cmp(&cleanup_action_sort_key(right.cleanup_action))
        });
        let options = DocumentOrphanCleanupExecutionOptions::default();
        let selected = selected_cleanup_delete_ids(&plan, &options);
        let mut report = DocumentOrphanCleanupExecutionReport::new(&plan, &options);
        report.selected_delete_groups = selected.len();
        report.actions = plan
            .groups
            .iter()
            .map(|group| cleanup_execution_action_for_dry_run(group, &selected))
            .collect();
        refresh_cleanup_execution_summary(&mut report);

        assert_eq!(report.selected_delete_groups, 1);
        assert_eq!(report.planned_delete_groups, 1);
        assert_eq!(report.planned_delete_chunks, 3);
        assert_eq!(report.quarantine_groups_retained, 1);
        assert_eq!(report.quarantine_chunks_retained, 2);
        assert_eq!(report.deleted_chunks, 0);
    }

    #[test]
    fn quarantine_review_chunk_truncation_respects_utf8_boundaries() {
        let source_id = Id::new();
        let chunk = DocChunk::new(source_id, "# Test", 1, "alpha βeta gamma");

        let prepared = prepare_quarantine_review_chunk(chunk, 8);

        assert!(prepared.content_truncated);
        assert!(prepared.content.contains("[truncated at 8 bytes]"));
        assert!(prepared.content.is_char_boundary(prepared.content.len()));
    }

    #[test]
    fn quarantine_review_group_page_uses_generated_marker_and_safe_fence() {
        let group = PreparedQuarantineReviewGroup {
            missing_source_id: "019c60b9-1327-73c2-9eb3-dbbe1853d01c".to_string(),
            relative_path: Path::new("groups/test.md").to_path_buf(),
            plan: cleanup_group(
                "019c60b9-1327-73c2-9eb3-dbbe1853d01c",
                1,
                DocumentOrphanCleanupAction::Quarantine,
            ),
            chunks: vec![PreparedQuarantineReviewChunk {
                chunk_id: "chunk-1".to_string(),
                heading_path: "# Test".to_string(),
                start_line: Some(1),
                end_line: Some(3),
                content: "```markdown\nnested fence\n```".to_string(),
                content_bytes: 27,
                content_truncated: false,
            }],
            database_chunk_count: 1,
            truncated_by_chunk_limit: false,
        };

        let page = quarantine_review_group_page(
            1,
            &group,
            &DocumentOrphanQuarantineReviewOptions::default(),
        );

        assert!(page.contains(DOCUMENT_ORPHAN_QUARANTINE_REVIEW_MARKER));
        assert!(page.contains("decision: pending"));
        assert!(page.contains("````markdown"));
        assert!(page.contains("## Machine Record"));
    }

    #[test]
    fn quarantine_review_status_counts_pending_decided_and_user_owned_pages() {
        let dir = tempfile::TempDir::new().unwrap();
        let pending_id = Id::new().to_string();
        let retained_id = Id::new().to_string();
        let pending_page = test_quarantine_review_page(&pending_id);
        let retained_page = test_quarantine_review_page(&retained_id).replace(
            "decision: pending # retain_quarantine | promote_to_memory_review | archive_legacy | delete_later",
            "decision: retain_quarantine # retain_quarantine | promote_to_memory_review | archive_legacy | delete_later",
        );
        fs::write(dir.path().join("pending.md"), pending_page).unwrap();
        fs::write(dir.path().join("retained.md"), retained_page).unwrap();
        fs::write(dir.path().join("notes.md"), "# User note\n").unwrap();

        let status = orphan_quarantine_review_status(dir.path()).unwrap();

        assert_eq!(status.files_scanned, 3);
        assert_eq!(status.generated_files, 2);
        assert_eq!(status.group_pages, 2);
        assert_eq!(status.user_owned_files, 1);
        assert_eq!(status.pending_count, 1);
        assert_eq!(status.retain_quarantine_count, 1);
        assert!(!status.ready_to_apply);
    }

    #[test]
    fn quarantine_review_apply_dry_run_plans_memory_review_promotions() {
        let dir = tempfile::TempDir::new().unwrap();
        let source_id = Id::new().to_string();
        let page = test_quarantine_review_page(&source_id)
            .replace(
                "decision: pending # retain_quarantine | promote_to_memory_review | archive_legacy | delete_later",
                "decision: promote_to_memory_review # retain_quarantine | promote_to_memory_review | archive_legacy | delete_later",
            )
            .replace(
                "memory_kind: null # preference | rule | decision | limitation | project_fact | repository_fact | task_fact | user_fact | session_insight",
                "memory_kind: project_fact # preference | rule | decision | limitation | project_fact | repository_fact | task_fact | user_fact | session_insight",
            )
            .replace(
                "scope_type: null # global | user | project | task | entity | repository | session | custom",
                "scope_type: project # global | user | project | task | entity | repository | session | custom",
            )
            .replace("scope_name: null", "scope_name: engram")
            .replace("title: null", "title: Recovered project fact");
        fs::write(dir.path().join("promote.md"), page).unwrap();

        let apply = apply_orphan_quarantine_review(
            dir.path(),
            DocumentOrphanQuarantineReviewApplyOptions::default(),
        )
        .unwrap();

        assert!(apply.dry_run);
        assert_eq!(apply.pending_count, 0);
        assert_eq!(apply.invalid_count, 0);
        assert_eq!(apply.promote_to_memory_review_count, 1);
        assert_eq!(apply.planned_memory_review_items.len(), 1);
        assert!(apply.ready_for_future_write);
        assert_eq!(
            apply.planned_memory_review_items[0].missing_source_id,
            source_id
        );
        assert_eq!(
            apply.planned_memory_review_items[0].title,
            "Recovered project fact"
        );
    }

    #[test]
    fn quarantine_review_prioritization_ranks_high_signal_pending_pages() {
        let dir = tempfile::TempDir::new().unwrap();
        let high_id = Id::new().to_string();
        let low_id = Id::new().to_string();
        let high_page = test_quarantine_review_page(&high_id)
            .replace("orphan_chunk_count: \"1\"", "orphan_chunk_count: \"60\"")
            .replace("- Plan orphan chunks: 1", "- Plan orphan chunks: 60")
            .replace(
                "content",
                "IMPORTANT architecture decision: Engram Memory OS migration rule and workflow",
            );
        let low_page = test_quarantine_review_page(&low_id).replace("content", "misc note");
        fs::write(dir.path().join("high.md"), high_page).unwrap();
        fs::write(dir.path().join("low.md"), low_page).unwrap();

        let report = prioritize_orphan_quarantine_review(
            dir.path(),
            DocumentOrphanQuarantineReviewPrioritizationOptions {
                limit: Some(2),
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(report.candidate_count, 2);
        assert_eq!(report.returned_count, 2);
        assert_eq!(report.items[0].missing_source_id, high_id);
        assert_eq!(
            report.items[0].priority,
            DocumentOrphanQuarantineReviewPriority::High
        );
        assert!(report.items[0]
            .detected_signals
            .contains(&"memory-os".to_string()));
        assert!(report.items[0].score > report.items[1].score);
    }

    #[test]
    fn quarantine_review_prioritization_skips_decided_pages_by_default() {
        let dir = tempfile::TempDir::new().unwrap();
        let pending_id = Id::new().to_string();
        let decided_id = Id::new().to_string();
        let decided_page = test_quarantine_review_page(&decided_id).replace(
            "decision: pending # retain_quarantine | promote_to_memory_review | archive_legacy | delete_later",
            "decision: retain_quarantine # retain_quarantine | promote_to_memory_review | archive_legacy | delete_later",
        );
        fs::write(
            dir.path().join("pending.md"),
            test_quarantine_review_page(&pending_id),
        )
        .unwrap();
        fs::write(dir.path().join("decided.md"), decided_page).unwrap();

        let default_report = prioritize_orphan_quarantine_review(
            dir.path(),
            DocumentOrphanQuarantineReviewPrioritizationOptions::default(),
        )
        .unwrap();
        assert_eq!(default_report.candidate_count, 1);
        assert_eq!(default_report.decided_skipped_count, 1);
        assert_eq!(default_report.items[0].missing_source_id, pending_id);

        let include_report = prioritize_orphan_quarantine_review(
            dir.path(),
            DocumentOrphanQuarantineReviewPrioritizationOptions {
                include_decided: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(include_report.candidate_count, 2);
        assert_eq!(include_report.decided_skipped_count, 0);
    }

    #[test]
    fn quarantine_review_prioritization_dedupes_fingerprints_by_default() {
        let dir = tempfile::TempDir::new().unwrap();
        let first_id = Id::new().to_string();
        let second_id = Id::new().to_string();
        let unique_id = Id::new().to_string();
        let duplicate_fingerprint = "duplicate-review-content";

        fs::write(
            dir.path().join("duplicate-a.md"),
            review_page_with_fingerprint(
                test_quarantine_review_page(&first_id),
                &first_id,
                duplicate_fingerprint,
            ),
        )
        .unwrap();
        fs::write(
            dir.path().join("duplicate-b.md"),
            review_page_with_fingerprint(
                test_quarantine_review_page(&second_id),
                &second_id,
                duplicate_fingerprint,
            ),
        )
        .unwrap();
        fs::write(
            dir.path().join("unique.md"),
            test_quarantine_review_page(&unique_id),
        )
        .unwrap();

        let report = prioritize_orphan_quarantine_review(
            dir.path(),
            DocumentOrphanQuarantineReviewPrioritizationOptions {
                limit: Some(10),
                ..Default::default()
            },
        )
        .unwrap();

        assert_eq!(report.candidate_count, 3);
        assert_eq!(report.ranked_candidate_count, 2);
        assert_eq!(report.returned_count, 2);
        assert_eq!(report.duplicate_fingerprint_group_count, 1);
        assert_eq!(report.duplicate_fingerprint_candidate_count, 2);
        assert_eq!(report.duplicate_fingerprint_skipped_count, 1);

        let duplicate_item = report
            .items
            .iter()
            .find(|item| item.content_fingerprint.as_deref() == Some(duplicate_fingerprint))
            .unwrap();
        assert_eq!(duplicate_item.fingerprint_group_size, 2);
        assert_eq!(duplicate_item.fingerprint_group_rank, 1);
        assert_eq!(duplicate_item.fingerprint_duplicate_paths.len(), 1);

        let include_report = prioritize_orphan_quarantine_review(
            dir.path(),
            DocumentOrphanQuarantineReviewPrioritizationOptions {
                limit: Some(10),
                include_duplicate_fingerprints: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(include_report.candidate_count, 3);
        assert_eq!(include_report.ranked_candidate_count, 3);
        assert_eq!(include_report.returned_count, 3);
        assert_eq!(include_report.duplicate_fingerprint_skipped_count, 0);
    }

    fn recoverable_group_with_candidate(
        missing_source_id: &str,
        chunk_count: u64,
        match_type: &str,
        path: &str,
        score: f32,
    ) -> DocumentOrphanGroup {
        DocumentOrphanGroup {
            missing_source_id: missing_source_id.to_string(),
            chunk_count,
            recovery_class: DocumentRecoveryClass::Recoverable,
            recovery_hint: "candidate_match".to_string(),
            content_fingerprint: "fingerprint".to_string(),
            content_anchor_count: 2,
            content_anchors: Vec::new(),
            detected_references: Vec::new(),
            candidate_matches: vec![candidate_match(match_type, path, score)],
            samples: Vec::new(),
        }
    }

    fn test_quarantine_review_page(source_id: &str) -> String {
        let group = PreparedQuarantineReviewGroup {
            missing_source_id: source_id.to_string(),
            relative_path: Path::new("groups/test.md").to_path_buf(),
            plan: cleanup_group(source_id, 1, DocumentOrphanCleanupAction::Quarantine),
            chunks: vec![PreparedQuarantineReviewChunk {
                chunk_id: "chunk-1".to_string(),
                heading_path: "# Test".to_string(),
                start_line: Some(1),
                end_line: Some(2),
                content: "content".to_string(),
                content_bytes: 7,
                content_truncated: false,
            }],
            database_chunk_count: 1,
            truncated_by_chunk_limit: false,
        };
        review_page_with_fingerprint(
            quarantine_review_group_page(
                1,
                &group,
                &DocumentOrphanQuarantineReviewOptions::default(),
            ),
            source_id,
            &format!("fingerprint-{source_id}"),
        )
    }

    fn review_page_with_fingerprint(page: String, source_id: &str, fingerprint: &str) -> String {
        let replacement = format!("content_fingerprint: \"{fingerprint}\"");
        for target in [
            format!("content_fingerprint: \"fingerprint-{source_id}\""),
            format!("content_fingerprint: fingerprint-{source_id}"),
            "content_fingerprint: \"fingerprint\"".to_string(),
            "content_fingerprint: fingerprint".to_string(),
        ] {
            if page.contains(&target) {
                return page.replace(&target, &replacement);
            }
        }
        page
    }

    fn candidate_match(match_type: &str, path: &str, score: f32) -> DocumentRecoveryCandidateMatch {
        DocumentRecoveryCandidateMatch {
            match_type: match_type.to_string(),
            path: path.to_string(),
            score,
            matched_anchors: 2,
            total_anchors: 2,
            exact_fingerprint_match: false,
            evidence: vec!["matched anchor".to_string()],
        }
    }

    fn cleanup_group(
        missing_source_id: &str,
        orphan_chunk_count: u64,
        cleanup_action: DocumentOrphanCleanupAction,
    ) -> DocumentOrphanCleanupGroupPlan {
        DocumentOrphanCleanupGroupPlan {
            cleanup_action,
            missing_source_id: missing_source_id.to_string(),
            orphan_chunk_count,
            recovery_class: match cleanup_action {
                DocumentOrphanCleanupAction::DeleteAfterSuccessfulReindex => {
                    DocumentRecoveryClass::Recoverable
                }
                DocumentOrphanCleanupAction::Quarantine => DocumentRecoveryClass::SafeToQuarantine,
                DocumentOrphanCleanupAction::ManualReview => DocumentRecoveryClass::Unknown,
            },
            recovery_hint: "test".to_string(),
            content_fingerprint: "fingerprint".to_string(),
            reason: "test".to_string(),
            reindex_source_path: None,
            reindex_action: None,
            reindex_status: None,
            existing_source_ids: Vec::new(),
            candidate_matches: Vec::new(),
            samples: Vec::new(),
        }
    }

    fn execution_report_for_plan(
        plan: &DocumentReindexPlan,
        status: DocumentReindexExecutionStatus,
    ) -> DocumentReindexExecutionReport {
        let actions = plan
            .sources
            .iter()
            .map(|source| {
                let mut action = DocumentReindexExecutionAction::base(source, false, status);
                action.chunk_count = Some(1);
                action
            })
            .collect::<Vec<_>>();
        DocumentReindexExecutionReport {
            dry_run: status == DocumentReindexExecutionStatus::Planned,
            orphan_cleanup_performed: false,
            plan_source_actions: plan.sources.len(),
            selected_source_actions: plan.sources.len(),
            planned_source_actions: usize::from(status == DocumentReindexExecutionStatus::Planned),
            reindexed_source_actions: usize::from(
                status == DocumentReindexExecutionStatus::Reindexed,
            ),
            already_indexed_source_actions: usize::from(
                status == DocumentReindexExecutionStatus::AlreadyIndexed,
            ),
            inspection_source_actions: usize::from(
                status == DocumentReindexExecutionStatus::RequiresInspection,
            ),
            skipped_source_actions: usize::from(status == DocumentReindexExecutionStatus::Skipped),
            failed_source_actions: usize::from(status == DocumentReindexExecutionStatus::Failed),
            reindexed_documents: usize::from(status == DocumentReindexExecutionStatus::Reindexed),
            planned_chunks: 0,
            indexed_chunks: 0,
            actions,
            warnings: Vec::new(),
        }
    }
}
