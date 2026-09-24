//! Unified search service for cross-layer search.
//!
//! Provides a single entry point for searching across all knowledge layers:
//! entities, aliases, observations, session events, documents, tool usages, and memory items.

use crate::document_search::merge_document_results;
use crate::error::{IndexError, IndexResult};
use crate::memory_ranker::{
    memory_scope_label, rank_memory_items, MemoryRankContext, RankedMemoryItem,
};
use crate::repository::RepositoryService;
use crate::service::DocumentStats;
use crate::tool_intel::ToolUsageInfo;
use engram_core::entity::{Entity, Observation};
use engram_core::id::Id;
use engram_core::memory::MemoryStatus;
use engram_core::search::{SearchLayer, SearchResultSource, UnifiedSearchResult};
use engram_core::session::{Event, SessionStats, SessionStatus};
use engram_core::tool::{ToolOutcome, ToolStats};
use engram_core::work::{Project, Task};
use engram_embed::Embedder;
use engram_store::{
    Db, DocumentRepo, EntityRepo, EntityStats, MemoryRepo, SessionRepo, ToolIntelStats, ToolRepo,
    WorkRepo,
};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use tracing::{debug, info};

/// Truncate a string to at most `max_bytes` bytes at a valid UTF-8 char boundary.
fn truncate_snippet(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }
    // Walk back from max_bytes to find a char boundary
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}...", &s[..end])
}

/// Service for unified cross-layer search.
#[derive(Clone)]
pub struct SearchService {
    entity_repo: EntityRepo,
    session_repo: SessionRepo,
    doc_repo: DocumentRepo,
    tool_repo: ToolRepo,
    memory_repo: MemoryRepo,
    work_repo: WorkRepo,
    repository_service: RepositoryService,
    embedder: Option<Embedder>,
}

/// Optional context for scoped search behavior.
#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    /// Project scope for MemoryItem filtering and fail-closed related legacy retrieval.
    pub project: Option<String>,
    /// Current working directory for repository/project scope resolution.
    pub cwd: Option<String>,
}

/// Result of a fail-closed related search.
#[derive(Debug, Clone)]
pub struct RelatedSearchOutcome {
    /// Scoped and ranked results.
    pub results: Vec<UnifiedSearchResult>,
    /// Canonical project selected for the search.
    pub project: String,
    /// Canonical exact task selected for the search, when requested.
    pub task: Option<String>,
    /// Requested layers searched with enforceable ownership.
    pub scoped_layers: Vec<SearchLayer>,
    /// Requested layers omitted because their ownership cannot be proven.
    pub omitted_layers: Vec<SearchLayer>,
}

#[derive(Debug, Clone)]
pub struct RelatedSearchScope {
    /// Canonical project that bounds retrieval.
    pub project: Project,
    /// Exact task that further narrows retrieval, when requested.
    pub task: Option<Task>,
    /// Entity IDs owned by the project or exact task.
    pub entity_ids: Vec<Id>,
    /// Project-owned session IDs; empty for exact-task scope because sessions lack task ownership.
    pub session_ids: Vec<Id>,
}

/// Tool recommendation derived only from project-owned session usages.
#[derive(Debug, Clone)]
pub struct RelatedToolRecommendation {
    /// Recommended tool name.
    pub tool_name: String,
    /// Success rate among matching project-owned usages.
    pub confidence: f32,
    /// Scope-aware explanation for the recommendation.
    pub reason: String,
}

impl SearchService {
    /// Create a new search service.
    pub fn new(db: Db) -> Self {
        Self {
            entity_repo: EntityRepo::new(db.clone()),
            session_repo: SessionRepo::new(db.clone()),
            doc_repo: DocumentRepo::new(db.clone()),
            tool_repo: ToolRepo::new(db.clone()),
            memory_repo: MemoryRepo::new(db.clone()),
            work_repo: WorkRepo::new(db.clone()),
            repository_service: RepositoryService::new(db),
            embedder: None,
        }
    }

    /// Create a search service with document search support.
    pub fn with_embedder(db: Db, embedder: Embedder) -> Self {
        Self {
            entity_repo: EntityRepo::new(db.clone()),
            session_repo: SessionRepo::new(db.clone()),
            doc_repo: DocumentRepo::new(db.clone()),
            tool_repo: ToolRepo::new(db.clone()),
            memory_repo: MemoryRepo::new(db.clone()),
            work_repo: WorkRepo::new(db.clone()),
            repository_service: RepositoryService::new(db),
            embedder: Some(embedder),
        }
    }

    /// Create a search service with default embedder for document search.
    pub fn with_defaults(db: Db) -> IndexResult<Self> {
        let embedder = Embedder::default_model()?;
        Ok(Self::with_embedder(db, embedder))
    }

    /// Search across all layers with a single query.
    ///
    /// # Arguments
    ///
    /// * `query` - The search query (text match for most layers, semantic for documents)
    /// * `limit_per_layer` - Maximum results per layer (default: 5)
    /// * `min_score` - Minimum score threshold (default: 0.3)
    /// * `layers` - Optional filter to specific layers (default: all layers)
    ///
    /// # Returns
    ///
    /// A vector of unified search results sorted by score (highest first).
    pub async fn search(
        &self,
        query: &str,
        limit_per_layer: usize,
        min_score: Option<f32>,
        layers: Option<&[SearchLayer]>,
    ) -> IndexResult<Vec<UnifiedSearchResult>> {
        self.search_with_options(
            query,
            limit_per_layer,
            min_score,
            layers,
            SearchOptions::default(),
        )
        .await
    }

    /// Search across all layers with optional context.
    pub async fn search_with_options(
        &self,
        query: &str,
        limit_per_layer: usize,
        min_score: Option<f32>,
        layers: Option<&[SearchLayer]>,
        options: SearchOptions,
    ) -> IndexResult<Vec<UnifiedSearchResult>> {
        let has_scope_boundary = options
            .project
            .as_deref()
            .is_some_and(|project| !project.trim().is_empty())
            || options
                .cwd
                .as_deref()
                .is_some_and(|cwd| !cwd.trim().is_empty());
        let requests_legacy_layer = layers
            .map(|layers| layers.iter().any(|layer| *layer != SearchLayer::Memory))
            .unwrap_or(true);
        if has_scope_boundary && requests_legacy_layer {
            return Ok(self
                .search_related(query, limit_per_layer, min_score, layers, &options, None)
                .await?
                .results);
        }

        info!(
            "Unified search: query='{}', limit={}, layers={:?}",
            query, limit_per_layer, layers
        );

        let min_score = min_score.unwrap_or(0.3);
        let layers = layers.map(|l| l.to_vec()).unwrap_or_else(SearchLayer::all);

        // Run searches in parallel using tokio::join!
        let (entities, aliases, observations, events, docs, tool_usages, memory_items) = tokio::join!(
            self.search_entities_if_enabled(&layers, query, limit_per_layer),
            self.search_aliases_if_enabled(&layers, query, limit_per_layer),
            self.search_observations_if_enabled(&layers, query, limit_per_layer),
            self.search_events_if_enabled(&layers, query, limit_per_layer),
            self.search_docs_if_enabled(&layers, query, limit_per_layer),
            self.search_tool_usages_if_enabled(&layers, query, limit_per_layer),
            self.search_memory_if_enabled(&layers, query, limit_per_layer, &options),
        );

        // Collect all results
        let mut results = Vec::new();
        results.extend(entities?);
        results.extend(aliases?);
        results.extend(observations?);
        results.extend(events?);
        results.extend(docs?);
        results.extend(tool_usages?);
        results.extend(memory_items?);

        let results = finalize_results(results, min_score);

        info!("Unified search found {} results", results.len());
        Ok(results)
    }

    /// Search only MemoryItems that apply to the supplied local scope.
    ///
    /// With no project, task, or cwd this deliberately returns only global/user memory.
    pub async fn search_local_memory(
        &self,
        query: &str,
        limit: usize,
        min_score: Option<f32>,
        options: &SearchOptions,
        task: Option<&str>,
    ) -> IndexResult<Vec<UnifiedSearchResult>> {
        let results = self
            .search_scoped_memory(
                query,
                limit,
                options.project.as_deref(),
                None,
                task,
                None,
                options.cwd.as_deref(),
            )
            .await?;
        Ok(finalize_results(results, min_score.unwrap_or(0.3)))
    }

    /// Search related records within one deterministically resolved authorization scope.
    ///
    /// Every requested layer is either searched with provable project/task ownership or listed
    /// in `omitted_layers`. The method fails closed when no single project can be resolved.
    pub async fn search_related(
        &self,
        query: &str,
        limit_per_layer: usize,
        min_score: Option<f32>,
        layers: Option<&[SearchLayer]>,
        options: &SearchOptions,
        task: Option<&str>,
    ) -> IndexResult<RelatedSearchOutcome> {
        let scope = self.resolve_related_scope(options, task).await?;
        let requested_layers = layers
            .map(|value| value.to_vec())
            .unwrap_or_else(SearchLayer::all);
        let exact_task = scope.task.is_some();
        let mut scoped_layers = Vec::new();
        let mut omitted_layers = Vec::new();
        for layer in &requested_layers {
            let ownership_is_provable = match layer {
                SearchLayer::Document => false,
                SearchLayer::SessionEvent | SearchLayer::ToolUsage if exact_task => false,
                _ => true,
            };
            if ownership_is_provable {
                scoped_layers.push(*layer);
            } else {
                omitted_layers.push(*layer);
            }
        }

        let query_embedding = self
            .embedder
            .as_ref()
            .and_then(|embedder| embedder.embed(query).ok());
        let mut results = Vec::new();
        if scoped_layers.contains(&SearchLayer::Entity) {
            results.extend(
                self.search_scoped_entities(
                    &scope.entity_ids,
                    query,
                    query_embedding.as_deref(),
                    limit_per_layer,
                )
                .await?,
            );
        }
        if scoped_layers.contains(&SearchLayer::Alias) {
            results.extend(
                self.search_scoped_aliases(&scope.entity_ids, query, limit_per_layer)
                    .await?,
            );
        }
        if scoped_layers.contains(&SearchLayer::Observation) {
            results.extend(
                self.search_scoped_observations(
                    &scope.entity_ids,
                    query,
                    query_embedding.as_deref(),
                    limit_per_layer,
                )
                .await?,
            );
        }
        if scoped_layers.contains(&SearchLayer::SessionEvent) {
            results.extend(
                self.search_related_session_events(&scope, query, limit_per_layer)
                    .await?
                    .into_iter()
                    .map(event_search_result),
            );
        }
        if scoped_layers.contains(&SearchLayer::ToolUsage) {
            results.extend(
                self.search_related_tool_usages(&scope, query, limit_per_layer)
                    .await?
                    .into_iter()
                    .map(tool_usage_search_result),
            );
        }
        if scoped_layers.contains(&SearchLayer::Memory) {
            results.extend(
                self.search_scoped_memory(
                    query,
                    limit_per_layer,
                    Some(&scope.project.name),
                    Some(&scope.project.id),
                    scope.task.as_ref().map(|task| task.name.as_str()),
                    scope.task.as_ref().map(|task| &task.id),
                    options.cwd.as_deref(),
                )
                .await?,
            );
        }

        let results = finalize_results(results, min_score.unwrap_or(0.3));
        Ok(RelatedSearchOutcome {
            results,
            project: scope.project.name,
            task: scope.task.map(|task| task.name),
            scoped_layers,
            omitted_layers,
        })
    }

    /// Resolve one fail-closed project/task authorization boundary for related retrieval.
    pub async fn resolve_related_scope(
        &self,
        options: &SearchOptions,
        task_ref: Option<&str>,
    ) -> IndexResult<RelatedSearchScope> {
        let explicit_project = options
            .project
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let task_ref = task_ref.map(str::trim).filter(|value| !value.is_empty());

        let mut task = if let Some(task_ref) = task_ref {
            if let Ok(id) = Id::parse(task_ref) {
                self.work_repo.get_task(&id).await?
            } else {
                self.work_repo.get_task_by_jira(task_ref).await?
            }
        } else {
            None
        };

        let project = if let Some(project_name) = explicit_project {
            self.work_repo
                .get_project_by_name(project_name)
                .await?
                .ok_or_else(|| {
                    IndexError::NotFound(format!(
                        "related search project not found: {project_name}"
                    ))
                })?
        } else if let Some(task) = &task {
            self.work_repo
                .get_project(&task.project_id)
                .await?
                .ok_or_else(|| {
                    IndexError::NotFound(format!(
                        "project {} for task '{}' was not found",
                        task.project_id, task.name
                    ))
                })?
        } else if let Some(cwd) = options.cwd.as_deref() {
            self.resolve_related_project_from_cwd(cwd).await?
        } else {
            return Err(IndexError::InvalidState(
                "related search requires a project, an ID/JIRA task reference, or a cwd that resolves to exactly one linked project"
                    .to_string(),
            ));
        };

        if let Some(task_ref) = task_ref {
            if task.is_none() {
                task = self
                    .work_repo
                    .get_task_by_name(&project.id, task_ref)
                    .await?;
            }
            let resolved_task = task.as_ref().ok_or_else(|| {
                IndexError::NotFound(format!(
                    "related search task '{}' was not found in project '{}'",
                    task_ref, project.name
                ))
            })?;
            if resolved_task.project_id != project.id {
                return Err(IndexError::InvalidState(format!(
                    "task '{}' belongs to project {}, not explicitly selected project '{}' ({})",
                    resolved_task.name, resolved_task.project_id, project.name, project.id
                )));
            }
        }

        let mut entity_ids = HashSet::new();
        for (entity_id, _) in self.work_repo.get_project_entities(&project.id).await? {
            entity_ids.insert(entity_id);
        }
        if let Some(task) = &task {
            for (entity_id, _) in self.work_repo.get_task_entities(&task.id).await? {
                entity_ids.insert(entity_id);
            }
        }
        let mut entity_ids: Vec<_> = entity_ids.into_iter().collect();
        entity_ids.sort_by_key(ToString::to_string);

        let session_ids = if task.is_none() {
            self.session_repo
                .list_sessions(None, None, Some(&project.name), None)
                .await?
                .into_iter()
                .map(|session| session.id)
                .collect()
        } else {
            Vec::new()
        };

        Ok(RelatedSearchScope {
            project,
            task,
            entity_ids,
            session_ids,
        })
    }

    /// Search events owned by the resolved related project.
    ///
    /// Exact-task scopes return no events because legacy sessions do not carry task ownership.
    pub async fn search_related_session_events(
        &self,
        scope: &RelatedSearchScope,
        query: &str,
        limit: usize,
    ) -> IndexResult<Vec<Event>> {
        let query = query.to_lowercase();
        let mut events = Vec::new();
        for session_id in &scope.session_ids {
            events.extend(
                self.session_repo
                    .get_events(session_id)
                    .await?
                    .into_iter()
                    .filter(|event| {
                        contains_query(
                            &query,
                            [
                                Some(event.content.as_str()),
                                event.context.as_deref(),
                                event.source.as_deref(),
                            ],
                        )
                    }),
            );
        }
        events.sort_by_key(|event| std::cmp::Reverse(event.timestamp));
        events.truncate(limit);
        Ok(events)
    }

    /// Search tool usages owned by sessions in the resolved related project.
    ///
    /// Exact-task scopes return no usages because legacy sessions do not carry task ownership.
    pub async fn search_related_tool_usages(
        &self,
        scope: &RelatedSearchScope,
        query: &str,
        limit: usize,
    ) -> IndexResult<Vec<ToolUsageInfo>> {
        let query = query.to_lowercase();
        let mut usages = Vec::new();
        for session_id in &scope.session_ids {
            usages.extend(
                self.tool_repo
                    .get_usages_for_session(session_id)
                    .await?
                    .into_iter()
                    .filter(|usage| usage.context.to_lowercase().contains(&query)),
            );
        }
        usages.sort_by_key(|usage| std::cmp::Reverse(usage.timestamp));
        usages.truncate(limit);

        let mut results = Vec::new();
        for usage in usages {
            let tool_name = self
                .entity_repo
                .get_entity(&usage.tool_id)
                .await?
                .map(|entity| entity.name)
                .unwrap_or_else(|| usage.tool_id.to_string());
            results.push(ToolUsageInfo {
                id: usage.id,
                tool_name,
                context: usage.context,
                outcome: usage.outcome,
                timestamp: usage.timestamp,
            });
        }
        Ok(results)
    }

    /// Recommend tools using only usages owned by the resolved related project.
    ///
    /// Global learned preferences are intentionally excluded because legacy preferences do not
    /// carry project or task ownership.
    pub async fn recommend_related_tools(
        &self,
        scope: &RelatedSearchScope,
        context: &str,
        limit: usize,
    ) -> IndexResult<Vec<RelatedToolRecommendation>> {
        let usages = self.search_related_tool_usages(scope, context, 20).await?;
        let mut stats: HashMap<String, (usize, usize)> = HashMap::new();
        for usage in usages {
            let counts = stats.entry(usage.tool_name).or_default();
            counts.0 += 1;
            if usage.outcome == ToolOutcome::Success {
                counts.1 += 1;
            }
        }

        let mut recommendations: Vec<_> = stats
            .into_iter()
            .map(|(tool_name, (total, successes))| {
                let confidence = successes as f32 / total as f32;
                RelatedToolRecommendation {
                    tool_name,
                    confidence,
                    reason: format!(
                        "Based on {total} similar usages owned by project '{}' with {:.0}% success rate",
                        scope.project.name,
                        confidence * 100.0
                    ),
                }
            })
            .collect();
        recommendations.sort_by(|left, right| {
            right
                .confidence
                .partial_cmp(&left.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.tool_name.cmp(&right.tool_name))
        });
        recommendations.truncate(limit);
        Ok(recommendations)
    }

    /// Calculate tool statistics using only usages owned by the resolved related project.
    ///
    /// `preferences_count` is zero because legacy learned preferences do not carry ownership.
    pub async fn related_tool_stats(
        &self,
        scope: &RelatedSearchScope,
        tool_name: &str,
    ) -> IndexResult<ToolStats> {
        let usages = self
            .search_related_tool_usages(scope, "", usize::MAX)
            .await?;
        let usages: Vec<_> = usages
            .into_iter()
            .filter(|usage| usage.tool_name.eq_ignore_ascii_case(tool_name))
            .collect();
        let total_usages = usages.len();
        let success_count = usages
            .iter()
            .filter(|usage| usage.outcome == ToolOutcome::Success)
            .count();
        let failure_count = usages
            .iter()
            .filter(|usage| usage.outcome == ToolOutcome::Failed)
            .count();
        let success_rate = if total_usages == 0 {
            0.0
        } else {
            success_count as f32 / total_usages as f32
        };
        Ok(ToolStats {
            total_usages,
            success_count,
            failure_count,
            success_rate,
            preferences_count: 0,
        })
    }

    /// Calculate entity statistics only from entities owned by the related scope.
    ///
    /// A relationship counts only when both endpoints are owned by the scope.
    pub async fn related_entity_stats(
        &self,
        scope: &RelatedSearchScope,
    ) -> IndexResult<EntityStats> {
        let owned: HashSet<_> = scope.entity_ids.iter().cloned().collect();
        let mut stats = EntityStats::default();
        for entity_id in &scope.entity_ids {
            if self.entity_repo.get_entity(entity_id).await?.is_none() {
                continue;
            }
            stats.entity_count += 1;
            stats.alias_count += self.entity_repo.get_aliases(entity_id).await?.len() as u64;
            stats.observation_count +=
                self.entity_repo.get_observations(entity_id).await?.len() as u64;
            stats.relationship_count += self
                .entity_repo
                .get_relationships_from(entity_id)
                .await?
                .into_iter()
                .filter(|relationship| owned.contains(&relationship.target_id))
                .count() as u64;
        }
        Ok(stats)
    }

    /// Calculate session statistics only from sessions owned by the related project.
    ///
    /// Exact-task scopes have no owned session IDs and therefore return zeroes.
    pub async fn related_session_stats(
        &self,
        scope: &RelatedSearchScope,
    ) -> IndexResult<SessionStats> {
        let mut stats = SessionStats::default();
        for session_id in &scope.session_ids {
            let Some(session) = self.session_repo.get_session(session_id).await? else {
                continue;
            };
            stats.total_sessions += 1;
            match session.status {
                SessionStatus::Active => stats.active_sessions += 1,
                SessionStatus::Completed => stats.completed_sessions += 1,
                SessionStatus::Abandoned => stats.abandoned_sessions += 1,
            }
            for event in self.session_repo.get_events(session_id).await? {
                stats.total_events += 1;
                *stats
                    .events_by_type
                    .entry(event.event_type.to_string())
                    .or_default() += 1;
            }
        }
        Ok(stats)
    }

    /// Calculate overall tool intelligence statistics from project-owned session usages.
    ///
    /// Legacy learned preferences are excluded because they do not carry ownership metadata.
    pub async fn related_tool_intel_stats(
        &self,
        scope: &RelatedSearchScope,
    ) -> IndexResult<ToolIntelStats> {
        let usages = self
            .search_related_tool_usages(scope, "", usize::MAX)
            .await?;
        Ok(ToolIntelStats {
            usage_count: usages.len() as u64,
            preference_count: 0,
        })
    }

    /// Return global document-index statistics for an explicitly global administrative read.
    pub async fn document_stats(&self) -> IndexResult<DocumentStats> {
        let stats = self.doc_repo.stats().await?;
        Ok(DocumentStats {
            source_count: stats.source_count,
            chunk_count: stats.chunk_count,
            searchable_chunk_count: stats.searchable_chunk_count,
            orphan_chunk_count: stats.orphan_chunk_count,
            embedding_dimension: self.embedder.as_ref().map_or(0, Embedder::dimension),
        })
    }

    async fn resolve_related_project_from_cwd(&self, cwd: &str) -> IndexResult<Project> {
        let context = self
            .repository_service
            .resolve_cwd(Path::new(cwd))
            .await?
            .ok_or_else(|| {
                IndexError::InvalidState(format!(
                    "related search cwd '{}' did not match a registered checkout",
                    cwd
                ))
            })?;
        let matching_paths: Vec<_> = context
            .matching_components
            .iter()
            .map(|component| component.path.as_str())
            .collect();
        let component_links: Vec<_> = context
            .linked_projects
            .iter()
            .filter(|link| {
                link.component_path
                    .as_deref()
                    .is_some_and(|path| matching_paths.contains(&path))
            })
            .collect();
        let candidate_links = if !component_links.is_empty() {
            component_links
        } else {
            let repository_links: Vec<_> = context
                .linked_projects
                .iter()
                .filter(|link| link.component_path.is_none())
                .collect();
            if repository_links.is_empty() {
                context.linked_projects.iter().collect()
            } else {
                repository_links
            }
        };
        let mut candidates: Vec<_> = candidate_links
            .into_iter()
            .map(|link| link.project_name.clone())
            .collect();
        candidates.sort();
        candidates.dedup_by(|left, right| left.eq_ignore_ascii_case(right));

        match candidates.as_slice() {
            [project_name] => self
                .work_repo
                .get_project_by_name(project_name)
                .await?
                .ok_or_else(|| {
                    IndexError::NotFound(format!(
                        "cwd resolved linked project '{}' but no work project exists",
                        project_name
                    ))
                }),
            [] => Err(IndexError::InvalidState(format!(
                "related search cwd '{}' matched repository '{}' with no linked project",
                cwd, context.repository.name
            ))),
            _ => Err(IndexError::InvalidState(format!(
                "related search cwd '{}' is ambiguous across projects: {}",
                cwd,
                candidates.join(", ")
            ))),
        }
    }

    // =========================================================================
    // Layer-specific search implementations
    // =========================================================================

    async fn search_entities_if_enabled(
        &self,
        layers: &[SearchLayer],
        query: &str,
        limit: usize,
    ) -> IndexResult<Vec<UnifiedSearchResult>> {
        if !layers.contains(&SearchLayer::Entity) {
            return Ok(Vec::new());
        }
        debug!("Searching entities: {}", query);

        let mut results = Vec::new();

        // Vector search if embedder is available
        if let Some(ref embedder) = self.embedder {
            if let Ok(query_embedding) = embedder.embed(query) {
                let vector_results = self
                    .entity_repo
                    .search_entities_by_embedding(&query_embedding, limit, 0.3)
                    .await?;

                for r in vector_results {
                    results.push(UnifiedSearchResult::new(
                        SearchResultSource::Entity,
                        r.score,
                        &r.entity.name,
                        r.entity
                            .description
                            .as_deref()
                            .unwrap_or(&format!("Entity of type {}", r.entity.entity_type)),
                        r.entity.id.to_string(),
                    ));
                }
            }
        }

        // Also do text search for exact/substring matches
        let text_results = self
            .entity_repo
            .search_entities_extended(query, limit)
            .await?;
        for e in text_results {
            // Higher score for name match vs description match
            let name_lower = e.name.to_lowercase();
            let query_lower = query.to_lowercase();
            let score = if name_lower.contains(&query_lower) {
                if name_lower == query_lower {
                    0.95
                } else {
                    0.85
                }
            } else {
                0.70 // Description match
            };

            results.push(UnifiedSearchResult::new(
                SearchResultSource::Entity,
                score,
                &e.name,
                e.description
                    .as_deref()
                    .unwrap_or(&format!("Entity of type {}", e.entity_type)),
                e.id.to_string(),
            ));
        }

        Ok(results)
    }

    async fn search_aliases_if_enabled(
        &self,
        layers: &[SearchLayer],
        query: &str,
        limit: usize,
    ) -> IndexResult<Vec<UnifiedSearchResult>> {
        if !layers.contains(&SearchLayer::Alias) {
            return Ok(Vec::new());
        }
        debug!("Searching aliases: {}", query);

        let aliases = self.entity_repo.search_aliases(query, limit).await?;

        // Look up entity names for context
        let mut results = Vec::new();
        for alias in aliases {
            let score = {
                let alias_lower = alias.alias_name.to_lowercase();
                let query_lower = query.to_lowercase();
                if alias_lower == query_lower {
                    0.90
                } else {
                    0.80
                }
            };

            // Try to get entity name for context
            let context = match self.entity_repo.get_entity(&alias.entity_id).await {
                Ok(Some(entity)) => Some(format!("alias for entity '{}'", entity.name)),
                _ => None,
            };

            let mut result = UnifiedSearchResult::new(
                SearchResultSource::Alias,
                score,
                &alias.alias_name,
                format!("Alias pointing to entity {}", alias.entity_id),
                alias.entity_id.to_string(),
            );
            if let Some(ctx) = context {
                result = result.with_context(ctx);
            }
            results.push(result);
        }

        Ok(results)
    }

    async fn search_observations_if_enabled(
        &self,
        layers: &[SearchLayer],
        query: &str,
        limit: usize,
    ) -> IndexResult<Vec<UnifiedSearchResult>> {
        if !layers.contains(&SearchLayer::Observation) {
            return Ok(Vec::new());
        }
        debug!("Searching observations globally: {}", query);

        let mut results = Vec::new();

        // Vector search if embedder is available
        if let Some(ref embedder) = self.embedder {
            if let Ok(query_embedding) = embedder.embed(query) {
                let vector_results = self
                    .entity_repo
                    .search_observations_by_embedding(&query_embedding, limit, 0.3)
                    .await?;

                for r in vector_results {
                    let obs = &r.observation;
                    // Try to get entity name for context
                    let context = match self.entity_repo.get_entity(&obs.entity_id).await {
                        Ok(Some(entity)) => {
                            let key_info = obs
                                .key
                                .as_ref()
                                .map(|k| format!(" [{}]", k))
                                .unwrap_or_default();
                            Some(format!("observation on '{}'{}", entity.name, key_info))
                        }
                        _ => obs.key.clone().map(|k| format!("key: {}", k)),
                    };

                    let snippet = truncate_snippet(&obs.content, 200);

                    let title = obs.key.clone().unwrap_or_else(|| "observation".to_string());

                    let mut result = UnifiedSearchResult::new(
                        SearchResultSource::Observation,
                        r.score,
                        title,
                        snippet,
                        obs.id.to_string(),
                    );
                    if let Some(ctx) = context {
                        result = result.with_context(ctx);
                    }
                    results.push(result);
                }
            }
        }

        // Also do text search for exact/substring matches
        let text_results = self
            .entity_repo
            .search_observations_global(query, limit)
            .await?;
        for obs in text_results {
            let score = {
                let content_lower = obs.content.to_lowercase();
                let query_lower = query.to_lowercase();
                if content_lower.contains(&query_lower) {
                    0.75
                } else {
                    0.70
                }
            };

            // Try to get entity name for context
            let context = match self.entity_repo.get_entity(&obs.entity_id).await {
                Ok(Some(entity)) => {
                    let key_info = obs
                        .key
                        .as_ref()
                        .map(|k| format!(" [{}]", k))
                        .unwrap_or_default();
                    Some(format!("observation on '{}'{}", entity.name, key_info))
                }
                _ => obs.key.clone().map(|k| format!("key: {}", k)),
            };

            let snippet = truncate_snippet(&obs.content, 200);

            let title = obs.key.clone().unwrap_or_else(|| "observation".to_string());

            let mut result = UnifiedSearchResult::new(
                SearchResultSource::Observation,
                score,
                title,
                snippet,
                obs.id.to_string(),
            );
            if let Some(ctx) = context {
                result = result.with_context(ctx);
            }
            results.push(result);
        }

        Ok(results)
    }

    async fn search_events_if_enabled(
        &self,
        layers: &[SearchLayer],
        query: &str,
        limit: usize,
    ) -> IndexResult<Vec<UnifiedSearchResult>> {
        if !layers.contains(&SearchLayer::SessionEvent) {
            return Ok(Vec::new());
        }
        debug!("Searching session events: {}", query);

        let events = self.session_repo.search_events(query, Some(limit)).await?;

        Ok(events
            .into_iter()
            .map(|e| {
                let score = 0.65; // Base score for event matches

                let snippet = truncate_snippet(&e.content, 200);

                UnifiedSearchResult::new(
                    SearchResultSource::SessionEvent,
                    score,
                    format!("{} event", e.event_type),
                    snippet,
                    e.id.to_string(),
                )
                .with_context(format!("session {}", e.session_id))
            })
            .collect())
    }

    async fn search_docs_if_enabled(
        &self,
        layers: &[SearchLayer],
        query: &str,
        limit: usize,
    ) -> IndexResult<Vec<UnifiedSearchResult>> {
        if !layers.contains(&SearchLayer::Document) {
            return Ok(Vec::new());
        }

        // Skip if no embedder available
        let Some(embedder) = &self.embedder else {
            debug!("Document search skipped: no embedder configured");
            return Ok(Vec::new());
        };

        debug!("Searching documents (semantic): {}", query);

        // Generate query embedding
        let query_embedding = embedder.embed(query)?;

        // Search chunks semantically and source metadata lexically for known-item lookups.
        let semantic_results = self
            .doc_repo
            .search_similar(&query_embedding, limit)
            .await?;
        let lexical_results = self.doc_repo.search_source_metadata(query, limit).await?;
        let results = merge_document_results(semantic_results, lexical_results, limit);

        Ok(results
            .into_iter()
            .map(|r| {
                // Score is already cosine similarity (0.0-1.0)
                let score = r.score;

                let snippet = truncate_snippet(&r.chunk.content, 200);

                let title = r
                    .source
                    .title
                    .unwrap_or_else(|| r.source.path_or_url.clone());

                UnifiedSearchResult::new(
                    SearchResultSource::Document,
                    score,
                    title,
                    snippet,
                    r.chunk.id.to_string(),
                )
                .with_context(format!("path: {}", r.source.path_or_url))
            })
            .collect())
    }

    async fn search_tool_usages_if_enabled(
        &self,
        layers: &[SearchLayer],
        query: &str,
        limit: usize,
    ) -> IndexResult<Vec<UnifiedSearchResult>> {
        if !layers.contains(&SearchLayer::ToolUsage) {
            return Ok(Vec::new());
        }
        debug!("Searching tool usages: {}", query);

        let usages = self.tool_repo.search_usages(query, Some(limit)).await?;

        // Look up tool names for better display
        let mut results = Vec::new();
        for u in usages {
            let score = 0.60; // Base score for tool usage matches

            // Try to get tool name
            let tool_name = match self.entity_repo.get_entity(&u.tool_id).await {
                Ok(Some(entity)) => entity.name,
                _ => u.tool_id.to_string(),
            };

            let snippet = truncate_snippet(&u.context, 200);

            results.push(
                UnifiedSearchResult::new(
                    SearchResultSource::ToolUsage,
                    score,
                    format!("{} ({})", tool_name, u.outcome),
                    snippet,
                    u.id.to_string(),
                )
                .with_context(format!("outcome: {}", u.outcome)),
            );
        }

        Ok(results)
    }

    async fn search_scoped_entities(
        &self,
        entity_ids: &[Id],
        query: &str,
        query_embedding: Option<&[f32]>,
        limit: usize,
    ) -> IndexResult<Vec<UnifiedSearchResult>> {
        let mut results = Vec::new();
        for entity_id in entity_ids {
            let Some(entity) = self.entity_repo.get_entity(entity_id).await? else {
                continue;
            };
            let Some(score) = scoped_entity_score(&entity, query, query_embedding) else {
                continue;
            };
            let content = entity
                .description
                .clone()
                .unwrap_or_else(|| format!("Entity of type {}", entity.entity_type));
            results.push(UnifiedSearchResult::new(
                SearchResultSource::Entity,
                score,
                entity.name,
                content,
                entity.id.to_string(),
            ));
        }
        sort_and_truncate(&mut results, limit);
        Ok(results)
    }

    async fn search_scoped_aliases(
        &self,
        entity_ids: &[Id],
        query: &str,
        limit: usize,
    ) -> IndexResult<Vec<UnifiedSearchResult>> {
        let query = query.to_lowercase();
        let mut results = Vec::new();
        for entity_id in entity_ids {
            let entity_name = self
                .entity_repo
                .get_entity(entity_id)
                .await?
                .map(|entity| entity.name)
                .unwrap_or_else(|| entity_id.to_string());
            for alias in self.entity_repo.get_aliases(entity_id).await? {
                let alias_lower = alias.to_lowercase();
                if !alias_lower.contains(&query) {
                    continue;
                }
                let score = if alias_lower == query { 0.90 } else { 0.80 };
                results.push(
                    UnifiedSearchResult::new(
                        SearchResultSource::Alias,
                        score,
                        alias,
                        format!("Alias pointing to entity {entity_id}"),
                        entity_id.to_string(),
                    )
                    .with_context(format!("alias for entity '{entity_name}'")),
                );
            }
        }
        sort_and_truncate(&mut results, limit);
        Ok(results)
    }

    async fn search_scoped_observations(
        &self,
        entity_ids: &[Id],
        query: &str,
        query_embedding: Option<&[f32]>,
        limit: usize,
    ) -> IndexResult<Vec<UnifiedSearchResult>> {
        let mut results = Vec::new();
        for entity_id in entity_ids {
            let entity_name = self
                .entity_repo
                .get_entity(entity_id)
                .await?
                .map(|entity| entity.name)
                .unwrap_or_else(|| entity_id.to_string());
            for observation in self.entity_repo.get_observations(entity_id).await? {
                let Some(score) = scoped_observation_score(&observation, query, query_embedding)
                else {
                    continue;
                };
                let key_info = observation
                    .key
                    .as_ref()
                    .map(|key| format!(" [{key}]"))
                    .unwrap_or_default();
                results.push(
                    UnifiedSearchResult::new(
                        SearchResultSource::Observation,
                        score,
                        observation
                            .key
                            .clone()
                            .unwrap_or_else(|| "observation".to_string()),
                        truncate_snippet(&observation.content, 200),
                        observation.id.to_string(),
                    )
                    .with_context(format!("observation on '{entity_name}'{key_info}")),
                );
            }
        }
        sort_and_truncate(&mut results, limit);
        Ok(results)
    }

    async fn search_scoped_memory(
        &self,
        query: &str,
        limit: usize,
        project: Option<&str>,
        project_id: Option<&Id>,
        task: Option<&str>,
        task_id: Option<&Id>,
        cwd: Option<&str>,
    ) -> IndexResult<Vec<UnifiedSearchResult>> {
        let (repository_id, repository_remote) = self
            .resolve_memory_repository_identity(project, project_id, cwd)
            .await?;
        let ranked = rank_memory_items(
            self.memory_repo
                .list_memory_items(Some(MemoryStatus::Active), None)
                .await?,
            MemoryRankContext::scoped_search(project, project_id, task, task_id, cwd, Some(query))
                .with_repository(repository_id.as_ref(), repository_remote.as_deref()),
        );
        Ok(ranked
            .into_iter()
            .take(limit)
            .map(memory_result_for_ranked)
            .collect())
    }

    async fn search_memory_if_enabled(
        &self,
        layers: &[SearchLayer],
        query: &str,
        limit: usize,
        options: &SearchOptions,
    ) -> IndexResult<Vec<UnifiedSearchResult>> {
        if !layers.contains(&SearchLayer::Memory) {
            return Ok(Vec::new());
        }
        debug!("Searching memory items: {}", query);

        let (repository_id, repository_remote) = self
            .resolve_memory_repository_identity(
                options.project.as_deref(),
                None,
                options.cwd.as_deref(),
            )
            .await?;
        let ranked = rank_memory_items(
            self.memory_repo
                .list_memory_items(Some(MemoryStatus::Active), None)
                .await?,
            MemoryRankContext::search(
                options.project.as_deref(),
                options.cwd.as_deref(),
                Some(query),
            )
            .with_repository(repository_id.as_ref(), repository_remote.as_deref()),
        );

        Ok(ranked
            .into_iter()
            .take(limit)
            .map(memory_result_for_ranked)
            .collect())
    }

    async fn resolve_memory_repository_identity(
        &self,
        project: Option<&str>,
        project_id: Option<&Id>,
        cwd: Option<&str>,
    ) -> IndexResult<(Option<Id>, Option<String>)> {
        let Some(cwd) = cwd else {
            return Ok((None, None));
        };
        let Some(context) = self.repository_service.resolve_cwd(Path::new(cwd)).await? else {
            return Ok((None, None));
        };

        let has_project_boundary = project.is_some() || project_id.is_some();
        let repository_is_owned_by_project = !has_project_boundary
            || context.linked_projects.iter().any(|link| {
                match (project_id, link.project_id.as_ref()) {
                    (Some(expected), Some(actual)) => expected == actual,
                    (Some(_), None) => project
                        .is_some_and(|project| link.project_name.eq_ignore_ascii_case(project)),
                    (None, _) => project
                        .is_some_and(|project| link.project_name.eq_ignore_ascii_case(project)),
                }
            });
        if !repository_is_owned_by_project {
            return Ok((None, None));
        }

        Ok((
            Some(context.repository.id),
            context.repository.remote_url.clone(),
        ))
    }

    /// Get statistics about what can be searched.
    pub async fn stats(&self) -> IndexResult<SearchStats> {
        let entity_stats = self.entity_repo.stats().await?;
        let session_stats = self.session_repo.stats().await?;
        let doc_stats = self.doc_repo.stats().await?;
        let tool_stats = self.tool_repo.stats().await?;
        let memory_count = self
            .memory_repo
            .list_memory_items(Some(MemoryStatus::Active), None)
            .await?
            .len() as u64;

        Ok(SearchStats {
            entity_count: entity_stats.entity_count,
            alias_count: entity_stats.alias_count,
            observation_count: entity_stats.observation_count,
            session_event_count: session_stats.total_events as u64,
            document_chunk_count: doc_stats.chunk_count,
            tool_usage_count: tool_stats.usage_count,
            memory_item_count: memory_count,
        })
    }
}

fn scoped_entity_score(
    entity: &Entity,
    query: &str,
    query_embedding: Option<&[f32]>,
) -> Option<f32> {
    let query_lower = query.to_lowercase();
    let name_lower = entity.name.to_lowercase();
    let lexical_score = if name_lower == query_lower {
        Some(0.95)
    } else if name_lower.contains(&query_lower) {
        Some(0.85)
    } else if entity
        .description
        .as_deref()
        .is_some_and(|description| description.to_lowercase().contains(&query_lower))
    {
        Some(0.70)
    } else {
        None
    };
    let vector_score = query_embedding
        .zip(entity.embedding.as_deref())
        .map(|(query, embedding)| cosine_similarity(query, embedding))
        .filter(|score| *score >= 0.3);
    max_optional_score(lexical_score, vector_score)
}

fn scoped_observation_score(
    observation: &Observation,
    query: &str,
    query_embedding: Option<&[f32]>,
) -> Option<f32> {
    let query_lower = query.to_lowercase();
    let content_match = observation.content.to_lowercase().contains(&query_lower);
    let metadata_match = contains_query(
        &query_lower,
        [observation.key.as_deref(), observation.source.as_deref()],
    );
    let lexical_score = if content_match {
        Some(0.75)
    } else if metadata_match {
        Some(0.70)
    } else {
        None
    };
    let vector_score = query_embedding
        .zip(observation.embedding.as_deref())
        .map(|(query, embedding)| cosine_similarity(query, embedding))
        .filter(|score| *score >= 0.3);
    max_optional_score(lexical_score, vector_score)
}

fn max_optional_score(left: Option<f32>, right: Option<f32>) -> Option<f32> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (Some(score), None) | (None, Some(score)) => Some(score),
        (None, None) => None,
    }
}

fn event_search_result(event: Event) -> UnifiedSearchResult {
    UnifiedSearchResult::new(
        SearchResultSource::SessionEvent,
        0.65,
        format!("{} event", event.event_type),
        truncate_snippet(&event.content, 200),
        event.id.to_string(),
    )
    .with_context(format!("session {}", event.session_id))
}

fn tool_usage_search_result(usage: ToolUsageInfo) -> UnifiedSearchResult {
    UnifiedSearchResult::new(
        SearchResultSource::ToolUsage,
        0.60,
        format!("{} ({})", usage.tool_name, usage.outcome),
        truncate_snippet(&usage.context, 200),
        usage.id.to_string(),
    )
    .with_context(format!("outcome: {}", usage.outcome))
}

fn contains_query<'a>(
    query_lower: &str,
    values: impl IntoIterator<Item = Option<&'a str>>,
) -> bool {
    values
        .into_iter()
        .flatten()
        .any(|value| value.to_lowercase().contains(query_lower))
}

fn cosine_similarity(left: &[f32], right: &[f32]) -> f32 {
    if left.len() != right.len() || left.is_empty() {
        return 0.0;
    }
    let dot: f32 = left.iter().zip(right).map(|(a, b)| a * b).sum();
    let left_norm = left.iter().map(|value| value * value).sum::<f32>().sqrt();
    let right_norm = right.iter().map(|value| value * value).sum::<f32>().sqrt();
    if left_norm == 0.0 || right_norm == 0.0 {
        0.0
    } else {
        dot / (left_norm * right_norm)
    }
}

fn sort_and_truncate(results: &mut Vec<UnifiedSearchResult>, limit: usize) {
    results.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results.truncate(limit);
}

fn finalize_results(results: Vec<UnifiedSearchResult>, min_score: f32) -> Vec<UnifiedSearchResult> {
    let mut results: Vec<_> = results
        .into_iter()
        .filter(|result| result.score >= min_score)
        .collect();
    results.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut seen: HashMap<String, ()> = HashMap::new();
    results.retain(|result| {
        let key = format!("{}:{}", result.source, result.id);
        seen.insert(key, ()).is_none()
    });
    results
}

/// Statistics about searchable content.
#[derive(Debug, Clone)]
pub struct SearchStats {
    /// Number of entities.
    pub entity_count: u64,
    /// Number of aliases.
    pub alias_count: u64,
    /// Number of observations.
    pub observation_count: u64,
    /// Number of session events.
    pub session_event_count: u64,
    /// Number of document chunks.
    pub document_chunk_count: u64,
    /// Number of tool usages.
    pub tool_usage_count: u64,
    /// Number of active Memory OS items.
    pub memory_item_count: u64,
}

fn memory_result_for_ranked(ranked: RankedMemoryItem) -> UnifiedSearchResult {
    let item = ranked.item;
    let snippet = truncate_snippet(&item.content, 200);
    let metadata = item.trust_metadata();
    let context = format!(
        "memory: {}, status: {}, review_state: {}, freshness: {}, scope: {}, evidence_count: {}, writer: {}/{}",
        item.kind,
        metadata.status,
        metadata.review_state,
        metadata.freshness,
        memory_scope_label(&item.scope),
        metadata.evidence_count,
        metadata.writer.harness,
        metadata.writer.model
    );

    UnifiedSearchResult::new(
        SearchResultSource::Memory,
        ranked.score,
        item.title,
        snippet,
        item.id.to_string(),
    )
    .with_context(context)
    .with_memory_metadata(metadata)
}
