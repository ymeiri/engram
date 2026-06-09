//! Shared MemoryItem ranking for orientation and search.

use engram_core::memory::{
    MemoryFreshness, MemoryItem, MemoryKind, MemoryReviewState, MemoryScope, MemoryStatus,
};
use std::path::Path;
use time::OffsetDateTime;

const CURRENT_PLAN_TAG: &str = "current-plan";

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
    /// Prompt-specific decision gate, guardrail, or current-plan contribution, normalized 0.0-1.0.
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
    promote_current_plan_for_continuation_query(&mut ranked, context);
    promote_current_plan_for_exact_approval_command_query(&mut ranked, context);
    promote_migration_gate_for_explicit_apply_query(&mut ranked, context);
    promote_contextual_migration_gate_for_current_plan_query(&mut ranked, context);
    promote_approval_gate_items_for_gate_query(&mut ranked, context);
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
        guidance_score(&item, context)
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

fn promote_current_plan_for_continuation_query(
    items: &mut [RankedMemoryItem],
    context: MemoryRankContext<'_>,
) {
    if !should_promote_current_plan(context) {
        return;
    }

    let Some(best_index) = items
        .iter()
        .enumerate()
        .filter(|(_, ranked)| is_current_plan_guidance_item(&ranked.item))
        .max_by(|(_, left), (_, right)| {
            left.components
                .scope
                .total_cmp(&right.components.scope)
                .then_with(|| left.item.updated_at.cmp(&right.item.updated_at))
                .then_with(|| left.item.id.to_string().cmp(&right.item.id.to_string()))
        })
        .map(|(index, _)| index)
    else {
        return;
    };

    if best_index > 0 {
        let top_score = items.first().map(|ranked| ranked.score).unwrap_or(0.0);
        items[best_index].score = (top_score + 0.001).min(1.0);
        sort_ranked_memory_items(items);
    }
}

fn promote_current_plan_for_exact_approval_command_query(
    items: &mut [RankedMemoryItem],
    context: MemoryRankContext<'_>,
) {
    if !context.require_text_match || (context.project.is_none() && context.cwd.is_none()) {
        return;
    }

    let Some(command) = normalized_exact_approval_command(context.query) else {
        return;
    };

    let Some(best_index) = items
        .iter()
        .enumerate()
        .filter(|(_, ranked)| {
            is_current_plan_guidance_item(&ranked.item)
                && item_contains_normalized_approval_command(&ranked.item, &command)
        })
        .max_by(|(_, left), (_, right)| {
            left.components
                .scope
                .total_cmp(&right.components.scope)
                .then_with(|| left.item.updated_at.cmp(&right.item.updated_at))
                .then_with(|| left.item.id.to_string().cmp(&right.item.id.to_string()))
        })
        .map(|(index, _)| index)
    else {
        return;
    };

    if best_index > 0 {
        let top_score = items.first().map(|ranked| ranked.score).unwrap_or(0.0);
        items[best_index].score = (top_score + 0.001).min(1.0);
        sort_ranked_memory_items(items);
    }
}

fn promote_migration_gate_for_explicit_apply_query(
    items: &mut [RankedMemoryItem],
    context: MemoryRankContext<'_>,
) {
    if !asks_for_explicit_migration_apply_gate(context.query) {
        return;
    }

    let Some(best_index) = items
        .iter()
        .enumerate()
        .filter(|(_, ranked)| is_actionable_migration_gate_item(&ranked.item))
        .max_by(|(_, left), (_, right)| {
            migration_apply_gate_signal_score(&left.item)
                .cmp(&migration_apply_gate_signal_score(&right.item))
                .then_with(|| {
                    migration_gate_kind_score(&left.item.kind)
                        .cmp(&migration_gate_kind_score(&right.item.kind))
                })
                .then_with(|| {
                    memory_review_rank(left.item.trust_metadata().review_state).cmp(
                        &memory_review_rank(right.item.trust_metadata().review_state),
                    )
                })
                .then_with(|| left.components.scope.total_cmp(&right.components.scope))
                .then_with(|| left.item.updated_at.cmp(&right.item.updated_at))
                .then_with(|| left.item.id.to_string().cmp(&right.item.id.to_string()))
        })
        .map(|(index, _)| index)
    else {
        return;
    };

    if best_index > 0 {
        let top_score = items.first().map(|ranked| ranked.score).unwrap_or(0.0);
        items[best_index].score = (top_score + 0.001).min(1.0);
        sort_ranked_memory_items(items);
    }
}

fn promote_contextual_migration_gate_for_current_plan_query(
    items: &mut [RankedMemoryItem],
    context: MemoryRankContext<'_>,
) {
    if !asks_for_contextual_migration_gate_with_current_plan(context) {
        return;
    }

    let Some(current_plan_index) = items
        .iter()
        .position(|ranked| is_current_plan_guidance_item(&ranked.item))
    else {
        return;
    };
    if current_plan_index != 0 {
        return;
    }

    let Some(gate_index) = items
        .iter()
        .enumerate()
        .filter(|(_, ranked)| is_contextual_migration_gate_item(&ranked.item))
        .max_by(|(_, left), (_, right)| {
            contextual_migration_gate_signal_score(&left.item)
                .cmp(&contextual_migration_gate_signal_score(&right.item))
                .then_with(|| {
                    migration_gate_kind_score(&left.item.kind)
                        .cmp(&migration_gate_kind_score(&right.item.kind))
                })
                .then_with(|| {
                    memory_review_rank(left.item.trust_metadata().review_state).cmp(
                        &memory_review_rank(right.item.trust_metadata().review_state),
                    )
                })
                .then_with(|| left.components.scope.total_cmp(&right.components.scope))
                .then_with(|| left.item.updated_at.cmp(&right.item.updated_at))
                .then_with(|| left.item.id.to_string().cmp(&right.item.id.to_string()))
        })
        .map(|(index, _)| index)
    else {
        return;
    };

    if gate_index == current_plan_index {
        return;
    }

    items[current_plan_index].score = 1.0;
    items[gate_index].components.guidance = items[gate_index].components.guidance.max(0.9);
    items[gate_index].score = 0.999;
    sort_ranked_memory_items(items);
}

fn promote_approval_gate_items_for_gate_query(
    items: &mut [RankedMemoryItem],
    context: MemoryRankContext<'_>,
) {
    let Some(query) = context.query.map(str::to_lowercase) else {
        return;
    };
    let query = remove_continuation_gate_negations(&query);
    if !query.contains("approval gate") || !asks_for_decision_gate(&query) {
        return;
    }

    let top_score = items.first().map(|ranked| ranked.score).unwrap_or(0.0);
    let mut promoted = false;
    for ranked in items
        .iter_mut()
        .filter(|ranked| is_approval_gate_item(&ranked.item))
    {
        ranked.components.guidance = ranked.components.guidance.max(1.0);
        ranked.score = (top_score + 0.001).min(1.0);
        promoted = true;
    }

    if promoted {
        sort_ranked_memory_items(items);
    }
}

fn should_promote_current_plan(context: MemoryRankContext<'_>) -> bool {
    if context.project.is_none() && context.cwd.is_none() {
        return false;
    }

    let Some(query) = context.query.map(str::to_lowercase) else {
        return false;
    };

    !asks_for_decision_gate(&query) && asks_for_current_plan_guidance(&query)
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

fn guidance_score(item: &MemoryItem, context: MemoryRankContext<'_>) -> f32 {
    let Some(query) = context.query.map(str::to_lowercase) else {
        return 0.0;
    };

    if asks_for_decision_gate(&query) {
        let title = item.title.to_lowercase();
        let content = item.content.to_lowercase();
        let reviewed = item.trust_metadata().review_state == MemoryReviewState::Reviewed;
        if query.contains("approval gate") {
            if title.contains("approval gate") {
                return if reviewed { 1.0 } else { 0.95 };
            }
            if content.contains("approval gate") {
                return if reviewed { 0.95 } else { 0.9 };
            }
        }
        if has_gate_language(&title) {
            return if reviewed { 1.0 } else { 0.6 };
        }
        if has_gate_language(&content) {
            return if reviewed { 0.8 } else { 0.4 };
        }
        return 0.0;
    }

    current_plan_guidance_score(item, &query, context)
}

fn current_plan_guidance_score(
    item: &MemoryItem,
    query: &str,
    context: MemoryRankContext<'_>,
) -> f32 {
    if !asks_for_current_plan_guidance(query) {
        return 0.0;
    }
    if context.project.is_none() && context.cwd.is_none() {
        return 0.0;
    }
    if !is_current_plan_guidance_item(item) {
        return 0.0;
    }

    let reviewed = item.trust_metadata().review_state == MemoryReviewState::Reviewed;
    if reviewed {
        0.7
    } else {
        0.6
    }
}

fn asks_for_decision_gate(query: &str) -> bool {
    // "non-gated" is continuation vocabulary; only independent gate terms trigger gate mode.
    // Bare "gate" is often a milestone noun ("M6 gate") in continuation prompts, not an
    // approval request. Explicit action or permission terms still keep gate guidance first.
    // Approval-gate wording inside a continuation prompt is context unless the query is asking
    // for a gate/handoff summary. Bare "approval" is not a gate request.
    let query = remove_continuation_gate_negations(&query.to_ascii_lowercase());
    if has_modal_gate_action(&query) {
        return true;
    }
    if query.contains("approval gate") {
        return !asks_for_current_plan_guidance(&query) || has_gate_summary_intent(&query);
    }
    [
        "proceed", "allowed", "allow", "apply", "safety", "block", "blocked", "must",
    ]
    .iter()
    .any(|term| contains_ascii_word(&query, term))
}

fn has_gate_summary_intent(query: &str) -> bool {
    [
        "handoff",
        "gate summary",
        "gate summaries",
        "approval summary",
        "approval summaries",
        "prepare a compact",
        "prepare compact",
        "summarize approval",
        "summarise approval",
    ]
    .iter()
    .any(|term| query.contains(term))
}

fn has_modal_gate_action(query: &str) -> bool {
    let query = collapse_ascii_whitespace(query);
    [
        "should we proceed",
        "should i proceed",
        "should we apply",
        "should i apply",
        "should we run",
        "should i run",
        "should we execute",
        "should i execute",
        "should we export",
        "should i export",
        "should we migrate",
        "should i migrate",
        "should we approve",
        "should i approve",
        "should we do",
        "should i do",
        "can we proceed",
        "can i proceed",
        "can we apply",
        "can i apply",
        "can we run",
        "can i run",
        "can we execute",
        "can i execute",
        "can we export",
        "can i export",
        "can we migrate",
        "can i migrate",
        "could we proceed",
        "could i proceed",
        "could we apply",
        "could i apply",
        "could we run",
        "could i run",
        "could we execute",
        "could i execute",
        "whether we should proceed",
        "whether i should proceed",
        "whether we should apply",
        "whether i should apply",
        "whether we should run",
        "whether i should run",
        "whether we should execute",
        "whether i should execute",
        "whether we should export",
        "whether i should export",
        "whether we should migrate",
        "whether i should migrate",
        "if we should proceed",
        "if i should proceed",
        "if we should apply",
        "if i should apply",
        "if we should run",
        "if i should run",
        "if we should execute",
        "if i should execute",
        "if we should export",
        "if i should export",
        "if we should migrate",
        "if i should migrate",
        "do we have approval",
        "do i have approval",
    ]
    .iter()
    .any(|phrase| starts_at_gate_boundary(&query, phrase))
}

fn starts_at_gate_boundary(query: &str, phrase: &str) -> bool {
    phrase.starts_with("whether ") && query.contains(phrase)
        || phrase.starts_with("if ") && query.contains(phrase)
        || query.starts_with(phrase)
        || [". ", "? ", "! ", "; ", ": ", ", ", " and ", ", and "]
            .iter()
            .any(|prefix| query.contains(&format!("{prefix}{phrase}")))
}

fn collapse_ascii_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalized_exact_approval_command(query: Option<&str>) -> Option<String> {
    let query = normalize_approval_command_text(query?);
    if query.contains("approval gate") || !query.starts_with("approve ") {
        return None;
    }

    let after_approve = query.strip_prefix("approve ")?;
    let task_ref = after_approve.split(':').next()?.trim();
    let description = after_approve.split_once(':')?.1.trim();
    if description.is_empty() {
        return None;
    }

    if !is_task_reference(task_ref) {
        return None;
    }

    Some(format!("approve {task_ref}: {description}"))
}

fn is_task_reference(value: &str) -> bool {
    let Some(digits) = value.strip_prefix('t') else {
        return false;
    };

    !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())
}

fn item_contains_normalized_approval_command(item: &MemoryItem, command: &str) -> bool {
    let value = format!("{} {} {}", item.title, item.content, item.tags.join(" "));
    normalize_approval_command_text(&value).contains(command)
}

fn normalize_approval_command_text(value: &str) -> String {
    let value = collapse_ascii_whitespace(&value.to_ascii_lowercase());
    remove_spaces_before_colons(&value)
}

fn remove_spaces_before_colons(value: &str) -> String {
    let mut normalized = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch == ':' {
            while normalized.ends_with(' ') {
                normalized.pop();
            }
        }
        normalized.push(ch);
    }
    normalized
}

pub(crate) fn is_open_ended_plan_work_prompt(query: &str) -> bool {
    let query = query.to_ascii_lowercase();
    [
        "complete",
        "continue",
        "current plan",
        "end state",
        "mission",
        "move forward",
        "next step",
        "production-quality",
        "resume",
        "where we left off",
    ]
    .iter()
    .any(|term| query.contains(term))
}

fn asks_for_current_plan_guidance(query: &str) -> bool {
    if [
        "ignore current plan",
        "ignore the current plan",
        "not current plan",
        "not the current plan",
        "instead of current plan",
        "instead of the current plan",
    ]
    .iter()
    .any(|term| query.contains(term))
    {
        return false;
    }

    [
        "continue",
        "current plan",
        "current-plan",
        "keep going",
        "move forward",
        "next step",
        "next steps",
        "pick up where",
        "resume",
        "what should happen next",
        "what should i do next",
        "what should we do next",
        "what do we do next",
        "what's left",
        "what is left",
        "where we left off",
    ]
    .iter()
    .any(|term| query.contains(term))
}

fn asks_for_explicit_migration_apply_gate(query: Option<&str>) -> bool {
    let Some(query) = query.map(str::to_lowercase) else {
        return false;
    };
    let query = remove_continuation_gate_negations(&query);
    let mentions_migration = ["migration", "m6"].iter().any(|term| query.contains(term));
    let asks_apply_or_permission = [
        "apply", "proceed", "approve", "approval", "approved", "allowed", "allow", "run",
        "execute", "write",
    ]
    .iter()
    .any(|term| contains_ascii_word(&query, term));

    mentions_migration && asks_apply_or_permission && asks_for_decision_gate(&query)
}

fn asks_for_contextual_migration_gate_with_current_plan(context: MemoryRankContext<'_>) -> bool {
    if !context.require_text_match
        || !should_promote_current_plan(context)
        || asks_for_explicit_migration_apply_gate(context.query)
    {
        return false;
    }

    let Some(query) = context.query.map(str::to_lowercase) else {
        return false;
    };
    let query = remove_continuation_gate_negations(&query);

    has_migration_gate_domain(&query) && has_contextual_gate_mention(&query)
}

fn is_current_plan_guidance_item(item: &MemoryItem) -> bool {
    item.status == MemoryStatus::Active
        && matches!(item.kind, MemoryKind::Decision | MemoryKind::Rule)
        && item
            .tags
            .iter()
            .any(|tag| tag.eq_ignore_ascii_case(CURRENT_PLAN_TAG))
}

fn is_actionable_migration_gate_item(item: &MemoryItem) -> bool {
    if item.status != MemoryStatus::Active {
        return false;
    }
    if is_current_plan_guidance_item(item) || is_gate_query_noise_item(item) {
        return false;
    }

    let value = format!(
        "{} {}",
        item.title.to_lowercase(),
        item.content.to_lowercase()
    );
    has_migration_gate_domain(&value)
        && has_migration_apply_gate_detail(&value)
        && migration_apply_gate_signal_score(item) > 0
}

fn is_contextual_migration_gate_item(item: &MemoryItem) -> bool {
    if item.status != MemoryStatus::Active
        || is_current_plan_guidance_item(item)
        || is_gate_query_noise_item(item)
    {
        return false;
    }

    let value = format!(
        "{} {}",
        item.title.to_lowercase(),
        item.content.to_lowercase()
    );
    has_migration_gate_domain(&value)
        && has_contextual_gate_mention(&value)
        && migration_apply_gate_signal_score(item) > 0
}

fn is_approval_gate_item(item: &MemoryItem) -> bool {
    if item.status != MemoryStatus::Active
        || is_current_plan_guidance_item(item)
        || is_gate_query_noise_item(item)
    {
        return false;
    }

    let value = format!(
        "{} {}",
        item.title.to_lowercase(),
        item.content.to_lowercase()
    );
    value.contains("approval gate")
}

fn is_gate_query_noise_item(item: &MemoryItem) -> bool {
    let value = format!(
        "{} {}",
        item.title.to_lowercase(),
        item.content.to_lowercase()
    );
    [
        "non-gated continuation",
        "non gated continuation",
        "search calibration",
        "ranking calibration",
        "calibration landed",
    ]
    .iter()
    .any(|term| value.contains(term))
}

fn has_migration_gate_domain(value: &str) -> bool {
    ["migration", "m6"].iter().any(|term| value.contains(term))
}

fn has_migration_apply_gate_detail(value: &str) -> bool {
    [
        "migration apply",
        "write apply",
        "write-apply",
        "write approval",
        "reviewed candidates",
        "rollback",
    ]
    .iter()
    .any(|term| value.contains(term))
}

fn has_contextual_gate_mention(value: &str) -> bool {
    let value = remove_continuation_gate_negations(value);
    ["approval gate", "review-gated"]
        .iter()
        .any(|term| value.contains(term))
        || ["gate", "gated"]
            .iter()
            .any(|term| contains_ascii_word(&value, term))
}

fn contextual_migration_gate_signal_score(item: &MemoryItem) -> u16 {
    let value = format!(
        "{} {}",
        item.title.to_lowercase(),
        item.content.to_lowercase()
    );
    let approval_gate_bonus = if value.contains("approval gate") {
        20
    } else {
        0
    };
    approval_gate_bonus + migration_apply_gate_signal_score(item)
}

fn migration_apply_gate_signal_score(item: &MemoryItem) -> u16 {
    let title = item.title.to_lowercase();
    let value = format!("{title} {}", item.content.to_lowercase());
    let mut score = [
        ("must not proceed", 10),
        ("must not run", 9),
        ("cannot proceed", 10),
        ("do not mark", 9),
        ("do not run", 9),
        ("without reviewed candidates", 8),
        ("without human review", 8),
        ("review statuses remain pending", 7),
        ("pending/undecided", 7),
        ("explicit write approval", 6),
        ("requires explicit user approval", 5),
        ("explicit user approval", 4),
        ("explicit approval", 4),
        ("requires approval", 4),
    ]
    .iter()
    .filter_map(|(term, weight)| value.contains(term).then_some(*weight))
    .sum();

    if title.contains("paused") && title.contains("migration review gate") {
        score += 4;
    }

    score
}

fn migration_gate_kind_score(kind: &MemoryKind) -> u8 {
    match kind {
        MemoryKind::Rule => 5,
        MemoryKind::Decision => 4,
        MemoryKind::Limitation => 3,
        MemoryKind::ProjectFact | MemoryKind::Handoff => 2,
        _ => 1,
    }
}

fn memory_review_rank(review_state: MemoryReviewState) -> u8 {
    match review_state {
        MemoryReviewState::Reviewed => 4,
        MemoryReviewState::ActiveUnreviewed => 3,
        MemoryReviewState::NeedsReview => 2,
        MemoryReviewState::Superseded
        | MemoryReviewState::Archived
        | MemoryReviewState::Rejected => 1,
    }
}

fn has_gate_language(value: &str) -> bool {
    let value = remove_continuation_gate_negations(value);
    ["review-gated", "do not", "should not", "requires approval"]
        .iter()
        .any(|term| value.contains(term))
        || ["gate", "must", "blocked", "cannot", "never"]
            .iter()
            .any(|term| contains_ascii_word(&value, term))
}

fn remove_continuation_gate_negations(value: &str) -> String {
    value
        .replace("non-gated", "")
        .replace("non gated", "")
        .replace("un-gated", "")
        .replace("ungated", "")
        .replace("not gated", "")
        .replace("not a gate", "")
}

fn contains_ascii_word(value: &str, word: &str) -> bool {
    value.match_indices(word).any(|(index, _)| {
        let bytes = value.as_bytes();
        let before = index.checked_sub(1).and_then(|i| bytes.get(i));
        let after = bytes.get(index + word.len());
        let before_is_boundary = match before {
            Some(byte) => !is_ascii_word_byte(*byte),
            None => true,
        };
        let after_is_boundary = match after {
            Some(byte) => !is_ascii_word_byte(*byte),
            None => true,
        };

        before_is_boundary && after_is_boundary
    })
}

fn is_ascii_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn canonical_or_original(path: &Path) -> std::path::PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn path_starts_with(path: &Path, base: &Path) -> bool {
    path == base || path.starts_with(base)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approval_gate_phrase_triggers_gate_mode() {
        assert!(asks_for_decision_gate(
            "Prepare a compact Brain Harness handoff: current plan, approval gates, \
             evidence-quality state, and next non-gated work."
        ));
    }

    #[test]
    fn what_should_happen_next_is_continuation_not_gate_mode() {
        for query in [
            "Continue the Engram Brain Harness work. What is the current plan and what \
             should happen next?",
            "What should we do next for Engram?",
        ] {
            assert!(!asks_for_decision_gate(query), "{query}");
        }
    }

    #[test]
    fn what_should_happen_next_is_current_plan_guidance() {
        for query in [
            "what should happen next Engram Brain Harness",
            "what should we do next for Engram?",
            "what do we do next",
        ] {
            assert!(asks_for_current_plan_guidance(query), "{query}");
            assert!(!asks_for_decision_gate(query), "{query}");
        }

        assert!(!asks_for_current_plan_guidance(
            "what happened next in the old synthesis"
        ));
    }

    #[test]
    fn modal_action_prompts_still_trigger_gate_mode() {
        for query in [
            "Should we proceed with the migration?",
            "Should we run migration_review_export?",
            "What is the current plan, and should we proceed with the migration?",
            "Continue from the plan and tell me whether we should run migration_review_export.",
            "approved M6 write apply deletion cleanup legacy simplification now",
        ] {
            assert!(asks_for_decision_gate(query), "{query}");
            let context = MemoryRankContext::search(
                Some("engram"),
                Some("/Users/yuval.meiri/projects/engram"),
                Some(query),
            );
            assert!(!should_promote_current_plan(context), "{query}");
        }
    }

    #[test]
    fn continuation_with_approval_gate_context_promotes_current_plan() {
        for query in [
            "current plan next step continue move forward Engram Brain Harness after T139 T135 \
             T139 approval gate",
            "what is the current plan and next step after T139, considering approval gates",
            "continue the current plan after T139; include any approval gate constraints",
            "move forward with Engram Brain Harness from current plan after T139 approval gate",
        ] {
            let context = MemoryRankContext::search(
                Some("engram"),
                Some("/Users/yuval.meiri/projects/engram"),
                Some(query),
            );
            assert!(
                asks_for_current_plan_guidance(&query.to_lowercase()),
                "{query}"
            );
            assert!(should_promote_current_plan(context), "{query}");
        }
    }

    #[test]
    fn exact_approval_command_detector_is_narrow() {
        for (query, expected) in [
            (
                "Approve T70: index exact files T59, T68, and T69.",
                Some("approve t70: index exact files t59, t68, and t69."),
            ),
            (
                " approve   T70 :   index exact files ",
                Some("approve t70: index exact files"),
            ),
            ("approve t1: do the thing", Some("approve t1: do the thing")),
            ("do you approve of this?", None),
            ("the approval gate for T70", None),
            ("I approve T70: sure", None),
            ("approved T70: done", None),
            ("approve: something", None),
            ("approve T70 without colon", None),
            ("approve", None),
            ("approve tabc: not digits", None),
            ("Should we proceed with migration apply?", None),
        ] {
            assert_eq!(
                normalized_exact_approval_command(Some(query)).as_deref(),
                expected,
                "{query}"
            );
        }
    }

    #[test]
    fn contextual_m6_gate_prompt_stays_out_of_gate_mode() {
        assert!(!asks_for_decision_gate(
            "current plan next step M6 gate context and non-gated work"
        ));
    }

    #[test]
    fn ungated_continuation_words_do_not_create_gate_context() {
        for query in [
            "current plan next ungated Brain Harness feedback confidence M6",
            "current plan next un-gated Brain Harness feedback confidence M6",
            "current plan next not gated Brain Harness feedback confidence M6",
        ] {
            let context = MemoryRankContext::search(
                Some("engram"),
                Some("/Users/yuval.meiri/projects/engram"),
                Some(query),
            );

            assert!(!asks_for_decision_gate(query), "{query}");
            assert!(
                !asks_for_contextual_migration_gate_with_current_plan(context),
                "{query}"
            );
            assert!(!has_contextual_gate_mention(query), "{query}");
            assert!(!has_gate_language(query), "{query}");
        }
    }

    #[test]
    fn explicit_gate_actions_survive_ungated_wording() {
        for query in [
            "next ungated step, should we proceed with migration apply?",
            "next not gated step, should we run M6 write apply?",
        ] {
            let context = MemoryRankContext::search(
                Some("engram"),
                Some("/Users/yuval.meiri/projects/engram"),
                Some(query),
            );

            assert!(asks_for_decision_gate(query), "{query}");
            assert!(!should_promote_current_plan(context), "{query}");
        }
    }

    #[test]
    fn decision_gate_action_words_require_boundaries() {
        for query in [
            "current plan next M6 mustache formatting",
            "current plan next M6 blockchain status",
            "current plan next M6 unblocked status",
            "current plan next M6 allowance notes",
            "current plan next M6 safetybelt note",
        ] {
            let context = MemoryRankContext::search(
                Some("engram"),
                Some("/Users/yuval.meiri/projects/engram"),
                Some(query),
            );

            assert!(!asks_for_decision_gate(query), "{query}");
            assert!(should_promote_current_plan(context), "{query}");
        }
    }

    #[test]
    fn decision_gate_action_words_still_trigger_at_boundaries() {
        for query in [
            "must not proceed with M6",
            "M6 path is blocked",
            "block M6 write apply",
            "allowed M6 apply",
            "M6 safety gate",
            "M6 write-apply gate",
        ] {
            assert!(asks_for_decision_gate(query), "{query}");
        }
    }

    #[test]
    fn gateway_words_do_not_create_gate_context() {
        for query in [
            "current plan next M6 gateway routing confidence",
            "current plan next M6 gatekeeper note",
            "current plan next M6 gatedness wording",
        ] {
            let context = MemoryRankContext::search(
                Some("engram"),
                Some("/Users/yuval.meiri/projects/engram"),
                Some(query),
            );

            assert!(
                !asks_for_contextual_migration_gate_with_current_plan(context),
                "{query}"
            );
            assert!(!has_contextual_gate_mention(query), "{query}");
            assert!(!has_gate_language(query), "{query}");
        }
    }

    #[test]
    fn gate_boundary_words_still_trigger_gate_context() {
        for query in [
            "current plan next M6 gate context",
            "current plan next M6 gated state",
            "current plan next M6 review-gated status",
        ] {
            let context = MemoryRankContext::search(
                Some("engram"),
                Some("/Users/yuval.meiri/projects/engram"),
                Some(query),
            );

            assert!(
                asks_for_contextual_migration_gate_with_current_plan(context),
                "{query}"
            );
            assert!(has_contextual_gate_mention(query), "{query}");
        }

        for query in [
            "must not proceed",
            "blocked path",
            "cannot proceed",
            "never run",
        ] {
            assert!(has_gate_language(query), "{query}");
        }
    }

    #[test]
    fn contextual_m6_gate_prompt_triggers_current_plan_gate_context() {
        let context = MemoryRankContext::search(
            Some("engram"),
            Some("/Users/yuval.meiri/projects/engram"),
            Some("current plan next non-gated Brain Harness feedback confidence M6 gate"),
        );

        assert!(asks_for_contextual_migration_gate_with_current_plan(
            context
        ));
    }

    #[test]
    fn contextual_m6_gate_promotion_is_search_only() {
        let context = MemoryRankContext::orientation(
            Some("engram"),
            Some("/Users/yuval.meiri/projects/engram"),
            Some("current plan next non-gated Brain Harness feedback confidence M6 gate"),
        );

        assert!(!asks_for_contextual_migration_gate_with_current_plan(
            context
        ));
    }

    #[test]
    fn explicit_m6_apply_prompt_does_not_trigger_current_plan_gate_context() {
        let context = MemoryRankContext::search(
            Some("engram"),
            Some("/Users/yuval.meiri/projects/engram"),
            Some("approved M6 write apply deletion cleanup legacy simplification now"),
        );

        assert!(!asks_for_contextual_migration_gate_with_current_plan(
            context
        ));
    }

    #[test]
    fn pure_current_plan_prompt_does_not_trigger_migration_gate_context() {
        let context = MemoryRankContext::search(
            Some("engram"),
            Some("/Users/yuval.meiri/projects/engram"),
            Some("current plan next non-gated Brain Harness feedback confidence"),
        );

        assert!(!asks_for_contextual_migration_gate_with_current_plan(
            context
        ));
    }
}
