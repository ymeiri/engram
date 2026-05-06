//! Unified search service for cross-layer search.
//!
//! Provides a single entry point for searching across all knowledge layers:
//! entities, aliases, observations, session events, documents, tool usages, and memory items.

use crate::error::IndexResult;
use engram_core::memory::{MemoryItem, MemoryScope, MemoryStatus};
use engram_core::search::{SearchLayer, SearchResultSource, UnifiedSearchResult};
use engram_embed::Embedder;
use engram_store::{Db, DocumentRepo, EntityRepo, MemoryRepo, SessionRepo, ToolRepo};
use std::collections::HashMap;
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
    embedder: Option<Embedder>,
}

/// Optional context for scoped search behavior.
#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    /// Project scope for MemoryItem filtering.
    pub project: Option<String>,
    /// Current working directory for repository-scoped MemoryItem filtering.
    pub cwd: Option<String>,
}

impl SearchService {
    /// Create a new search service.
    pub fn new(db: Db) -> Self {
        Self {
            entity_repo: EntityRepo::new(db.clone()),
            session_repo: SessionRepo::new(db.clone()),
            doc_repo: DocumentRepo::new(db.clone()),
            tool_repo: ToolRepo::new(db.clone()),
            memory_repo: MemoryRepo::new(db),
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
            memory_repo: MemoryRepo::new(db),
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

        // Filter by minimum score
        let mut results: Vec<_> = results
            .into_iter()
            .filter(|r| r.score >= min_score)
            .collect();

        // Sort by score (highest first)
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Remove duplicates (same id from same source)
        let mut seen: HashMap<String, ()> = HashMap::new();
        results.retain(|r| {
            let key = format!("{}:{}", r.source, r.id);
            seen.insert(key, ()).is_none()
        });

        info!("Unified search found {} results", results.len());
        Ok(results)
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

        // Search for similar chunks
        let results = self
            .doc_repo
            .search_similar(&query_embedding, limit)
            .await?;

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

        let mut results = self
            .memory_repo
            .list_memory_items(Some(MemoryStatus::Active), None)
            .await?
            .into_iter()
            .filter(|item| memory_scope_matches(item, options))
            .filter_map(|item| memory_result_for_query(item, query))
            .collect::<Vec<_>>();

        results.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.title.cmp(&right.title))
        });
        results.truncate(limit);
        Ok(results)
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

fn memory_result_for_query(item: MemoryItem, query: &str) -> Option<UnifiedSearchResult> {
    let score = memory_text_score(&item, query)?;
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

    Some(
        UnifiedSearchResult::new(
            SearchResultSource::Memory,
            score,
            item.title,
            snippet,
            item.id.to_string(),
        )
        .with_context(context)
        .with_memory_metadata(metadata),
    )
}

fn memory_text_score(item: &MemoryItem, query: &str) -> Option<f32> {
    let query = query.trim();
    if query.is_empty() {
        return None;
    }

    let query_lower = query.to_lowercase();
    let title_lower = item.title.to_lowercase();
    let content_lower = item.content.to_lowercase();
    let tags_lower = item.tags.join(" ").to_lowercase();
    let haystack = format!(
        "{} {} {} {} {}",
        title_lower,
        content_lower,
        tags_lower,
        item.kind,
        memory_scope_label(&item.scope).to_lowercase()
    );

    if title_lower == query_lower {
        return Some(0.96);
    }
    if title_lower.contains(&query_lower) {
        return Some(0.90);
    }
    if content_lower.contains(&query_lower) {
        return Some(0.84);
    }

    let terms = query_lower
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|term| term.len() >= 3)
        .collect::<Vec<_>>();
    if terms.is_empty() {
        return None;
    }

    let matched = terms
        .iter()
        .filter(|term| haystack.contains(**term))
        .count();
    if matched == 0 {
        return None;
    }

    let ratio = matched as f32 / terms.len() as f32;
    let title_hits = terms
        .iter()
        .filter(|term| title_lower.contains(**term))
        .count();
    let content_hits = terms
        .iter()
        .filter(|term| content_lower.contains(**term))
        .count();
    let mut score = 0.48 + ratio * 0.28 + title_hits as f32 * 0.04 + content_hits as f32 * 0.02;
    score += item.confidence.value() * 0.08;
    Some(score.min(0.88))
}

fn memory_scope_matches(item: &MemoryItem, options: &SearchOptions) -> bool {
    if options.project.is_none() && options.cwd.is_none() {
        return true;
    }

    match &item.scope {
        MemoryScope::Global | MemoryScope::User => true,
        MemoryScope::Project { project_name, .. } => options
            .project
            .as_deref()
            .is_some_and(|project| project_name.eq_ignore_ascii_case(project)),
        MemoryScope::Task { project_name, .. } => {
            match (options.project.as_deref(), project_name) {
                (Some(project), Some(item_project)) => item_project.eq_ignore_ascii_case(project),
                _ => false,
            }
        }
        MemoryScope::Repository { local_path, .. } => match (options.cwd.as_deref(), local_path) {
            (Some(cwd), Some(local_path)) => path_starts_with(
                &canonical_or_original(Path::new(cwd)),
                &canonical_or_original(Path::new(local_path)),
            ),
            _ => false,
        },
        MemoryScope::Entity { .. } | MemoryScope::Session { .. } | MemoryScope::Custom { .. } => {
            false
        }
    }
}

fn memory_scope_label(scope: &MemoryScope) -> String {
    match scope {
        MemoryScope::Global => "global".to_string(),
        MemoryScope::User => "user".to_string(),
        MemoryScope::Project { project_name, .. } => format!("project:{project_name}"),
        MemoryScope::Task {
            project_name,
            task_name,
            ..
        } => match project_name {
            Some(project_name) => format!("task:{project_name}/{task_name}"),
            None => format!("task:{task_name}"),
        },
        MemoryScope::Entity { entity_name, .. } => format!("entity:{entity_name}"),
        MemoryScope::Repository {
            remote_url,
            local_path,
            ..
        } => format!(
            "repository:{}",
            local_path
                .as_deref()
                .or(remote_url.as_deref())
                .unwrap_or("unknown")
        ),
        MemoryScope::Session { session_id } => format!("session:{session_id}"),
        MemoryScope::Custom { name } => format!("custom:{name}"),
    }
}

fn canonical_or_original(path: &Path) -> std::path::PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn path_starts_with(path: &Path, base: &Path) -> bool {
    path == base || path.starts_with(base)
}
