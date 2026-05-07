//! Shared MemoryItem ranking for orientation and search.

use engram_core::memory::{
    MemoryFreshness, MemoryItem, MemoryReviewState, MemoryScope, MemoryStatus,
};
use std::path::Path;
use time::OffsetDateTime;

/// Scope filtering policy for memory ranking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MemoryScopePolicy {
    /// Only rank memory that applies to the provided project/cwd, plus global/user memory.
    ScopedOnly,
    /// If project/cwd are both absent, rank memory from all scopes.
    AllWhenUnscoped,
}

/// Context for deterministic MemoryItem ranking.
#[derive(Debug, Clone, Copy)]
pub(crate) struct MemoryRankContext<'a> {
    /// Project scope, when known.
    pub project: Option<&'a str>,
    /// Current working directory, when known.
    pub cwd: Option<&'a str>,
    /// Prompt or search query.
    pub query: Option<&'a str>,
    /// Scope filtering behavior.
    pub scope_policy: MemoryScopePolicy,
    /// Whether a query text match is required.
    pub require_text_match: bool,
}

impl<'a> MemoryRankContext<'a> {
    /// Ranking context for orientation.
    pub(crate) fn orientation(
        project: Option<&'a str>,
        cwd: Option<&'a str>,
        query: Option<&'a str>,
    ) -> Self {
        Self {
            project,
            cwd,
            query,
            scope_policy: MemoryScopePolicy::ScopedOnly,
            require_text_match: false,
        }
    }

    /// Ranking context for memory search.
    pub(crate) fn search(
        project: Option<&'a str>,
        cwd: Option<&'a str>,
        query: Option<&'a str>,
    ) -> Self {
        Self {
            project,
            cwd,
            query,
            scope_policy: MemoryScopePolicy::AllWhenUnscoped,
            require_text_match: true,
        }
    }

    /// Ranking context for changes_since relevance.
    pub(crate) fn changes_since(
        project: Option<&'a str>,
        cwd: Option<&'a str>,
        query: Option<&'a str>,
    ) -> Self {
        Self {
            project,
            cwd,
            query,
            scope_policy: MemoryScopePolicy::AllWhenUnscoped,
            require_text_match: false,
        }
    }
}

/// A MemoryItem with its deterministic rank score and components.
#[derive(Debug, Clone)]
pub(crate) struct RankedMemoryItem {
    /// Ranked memory item.
    pub item: MemoryItem,
    /// Final deterministic rank score.
    pub score: f32,
    /// Score components used to produce the final rank.
    pub components: MemoryRankComponents,
}

/// Memory rank components exposed for tests and future tuning.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct MemoryRankComponents {
    /// Query/title/content/tag relevance, normalized 0.0-1.0.
    pub text: f32,
    /// Scope relevance, normalized 0.0-1.0.
    pub scope: f32,
    /// Recency relevance, normalized 0.0-1.0.
    pub recency: f32,
    /// Confidence value, normalized 0.0-1.0.
    pub confidence: f32,
    /// Lifecycle status contribution, normalized 0.0-1.0.
    pub status: f32,
    /// Review/trust contribution, normalized 0.0-1.0.
    pub review: f32,
    /// Freshness contribution, normalized 0.0-1.0.
    pub freshness: f32,
    /// Prompt-specific decision gate or guardrail contribution, normalized 0.0-1.0.
    pub guidance: f32,
}

/// Rank a collection of memory items for orientation or search.
pub(crate) fn rank_memory_items(
    items: Vec<MemoryItem>,
    context: MemoryRankContext<'_>,
) -> Vec<RankedMemoryItem> {
    let mut ranked = items
        .into_iter()
        .filter_map(|item| rank_memory_item(item, context))
        .collect::<Vec<_>>();
    sort_ranked_memory_items(&mut ranked);
    ranked
}

/// Rank a single memory item. Returns None when the item should not be returned.
pub(crate) fn rank_memory_item(
    item: MemoryItem,
    context: MemoryRankContext<'_>,
) -> Option<RankedMemoryItem> {
    if !memory_scope_matches(&item, context) {
        return None;
    }

    let text = text_match_score(&item, context.query);
    if context.require_text_match && text.is_none() {
        return None;
    }

    let text_score = text.unwrap_or(0.0);
    let guidance = if text.is_some() {
        guidance_score(&item, context.query)
    } else {
        0.0
    };

    let components = MemoryRankComponents {
        text: text_score,
        scope: normalized_scope_score(&item, context),
        recency: recency_score(item.updated_at),
        confidence: item.confidence.value(),
        status: status_score(item.status),
        review: review_score(item.trust_metadata().review_state),
        freshness: freshness_score(item.trust_metadata().freshness),
        guidance,
    };
    let score = rank_score(components);

    Some(RankedMemoryItem {
        item,
        score,
        components,
    })
}

/// Whether a memory item applies to the ranking context.
pub(crate) fn memory_scope_matches(item: &MemoryItem, context: MemoryRankContext<'_>) -> bool {
    if context.scope_policy == MemoryScopePolicy::AllWhenUnscoped
        && context.project.is_none()
        && context.cwd.is_none()
    {
        return true;
    }

    raw_scope_score(item, context.project, context.cwd) > 0.0
}

/// Human-readable memory scope label.
pub(crate) fn memory_scope_label(scope: &MemoryScope) -> String {
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

fn sort_ranked_memory_items(items: &mut [RankedMemoryItem]) {
    items.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| right.item.updated_at.cmp(&left.item.updated_at))
            .then_with(|| left.item.title.cmp(&right.item.title))
    });
}

fn rank_score(components: MemoryRankComponents) -> f32 {
    let score = components.text * 0.78
        + components.scope * 0.09
        + components.recency * 0.04
        + components.confidence * 0.04
        + components.status * 0.02
        + components.review * 0.02
        + components.freshness * 0.01
        + components.guidance * 0.10;
    score.clamp(0.0, 1.0)
}

fn normalized_scope_score(item: &MemoryItem, context: MemoryRankContext<'_>) -> f32 {
    if context.scope_policy == MemoryScopePolicy::AllWhenUnscoped
        && context.project.is_none()
        && context.cwd.is_none()
    {
        return 0.0;
    }
    raw_scope_score(item, context.project, context.cwd) / 4.0
}

fn raw_scope_score(item: &MemoryItem, project: Option<&str>, cwd: Option<&str>) -> f32 {
    match &item.scope {
        MemoryScope::Project { project_name, .. } => project
            .filter(|project| project_name.eq_ignore_ascii_case(project))
            .map(|_| 4.0)
            .unwrap_or(0.0),
        MemoryScope::Task { project_name, .. } => match (project, project_name) {
            (Some(project), Some(item_project)) if item_project.eq_ignore_ascii_case(project) => {
                3.5
            }
            _ => 0.0,
        },
        MemoryScope::Repository { local_path, .. } => match (cwd, local_path) {
            (Some(cwd), Some(local_path)) => {
                let cwd_path = canonical_or_original(Path::new(cwd));
                let local_path = canonical_or_original(Path::new(local_path));
                if path_starts_with(&cwd_path, &local_path) {
                    3.0
                } else {
                    0.0
                }
            }
            _ => 0.0,
        },
        MemoryScope::Global | MemoryScope::User => 1.0,
        MemoryScope::Entity { .. } | MemoryScope::Session { .. } | MemoryScope::Custom { .. } => {
            0.0
        }
    }
}

fn text_match_score(item: &MemoryItem, query: Option<&str>) -> Option<f32> {
    let query = query?.trim();
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
    let score = 0.48 + ratio * 0.28 + title_hits as f32 * 0.04 + content_hits as f32 * 0.02;
    Some(score.min(0.88))
}

fn recency_score(updated_at: OffsetDateTime) -> f32 {
    let age = OffsetDateTime::now_utc() - updated_at;
    if age < time::Duration::days(7) {
        1.0
    } else if age < time::Duration::days(30) {
        0.7
    } else if age < time::Duration::days(120) {
        0.3
    } else {
        0.0
    }
}

fn status_score(status: MemoryStatus) -> f32 {
    match status {
        MemoryStatus::Active => 1.0,
        MemoryStatus::NeedsReview => 0.35,
        MemoryStatus::Superseded | MemoryStatus::Archived | MemoryStatus::Rejected => 0.0,
    }
}

fn review_score(review_state: MemoryReviewState) -> f32 {
    match review_state {
        MemoryReviewState::Reviewed => 1.0,
        MemoryReviewState::ActiveUnreviewed => 0.7,
        MemoryReviewState::NeedsReview => 0.25,
        MemoryReviewState::Superseded
        | MemoryReviewState::Archived
        | MemoryReviewState::Rejected => 0.0,
    }
}

fn freshness_score(freshness: MemoryFreshness) -> f32 {
    match freshness {
        MemoryFreshness::ReviewScheduled => 1.0,
        MemoryFreshness::Unscheduled => 0.7,
        MemoryFreshness::ReviewDue => 0.0,
    }
}

fn guidance_score(item: &MemoryItem, query: Option<&str>) -> f32 {
    let Some(query) = query.map(str::to_lowercase) else {
        return 0.0;
    };
    if !asks_for_decision_gate(&query) {
        return 0.0;
    }

    let title = item.title.to_lowercase();
    let content = item.content.to_lowercase();
    let reviewed = item.trust_metadata().review_state == MemoryReviewState::Reviewed;
    if has_gate_language(&title) {
        return if reviewed { 1.0 } else { 0.6 };
    }
    if has_gate_language(&content) {
        return if reviewed { 0.8 } else { 0.4 };
    }
    0.0
}

fn asks_for_decision_gate(query: &str) -> bool {
    [
        "should", "proceed", "allowed", "allow", "apply", "gate", "safety", "block", "blocked",
        "must",
    ]
    .iter()
    .any(|term| query.contains(term))
}

fn has_gate_language(value: &str) -> bool {
    [
        "review-gated",
        "gate",
        "must",
        "do not",
        "should not",
        "blocked",
        "cannot",
        "never",
        "requires approval",
    ]
    .iter()
    .any(|term| value.contains(term))
}

fn canonical_or_original(path: &Path) -> std::path::PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn path_starts_with(path: &Path, base: &Path) -> bool {
    path == base || path.starts_with(base)
}
