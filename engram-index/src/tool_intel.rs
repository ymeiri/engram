//! Tool Intelligence service for Layer 4.
//!
//! Provides business logic for tracking tool usage, learning preferences,
//! and generating recommendations based on historical data.

use crate::error::{IndexError, IndexResult};
use crate::EntityService;
use engram_core::entity::EntityType;
use engram_core::id::Id;
use engram_core::tool::{ToolOutcome, ToolPreference, ToolRecommendation, ToolStats, ToolUsage};
use engram_store::{Db, ToolIntelStats, ToolRepo};
use tracing::{debug, info};

/// Service for tool intelligence management.
#[derive(Clone)]
pub struct ToolIntelService {
    repo: ToolRepo,
    entity_service: EntityService,
}

impl ToolIntelService {
    /// Create a new tool intelligence service.
    pub fn new(db: Db) -> Self {
        Self {
            repo: ToolRepo::new(db.clone()),
            entity_service: EntityService::new(db),
        }
    }

    /// Initialize the tool intelligence schema.
    pub async fn init(&self) -> IndexResult<()> {
        self.repo.init_schema().await?;
        self.entity_service.init().await?;
        Ok(())
    }

    // =========================================================================
    // Tool Usage Logging
    // =========================================================================

    /// Log a tool usage with outcome.
    ///
    /// The tool must be registered as an entity first. If not found by name,
    /// we try to resolve it as an alias.
    pub async fn log_usage(
        &self,
        tool_name: &str,
        context: &str,
        outcome: ToolOutcome,
        session_id: Option<&Id>,
    ) -> IndexResult<ToolUsage> {
        info!(
            "Logging tool usage: {} ({}) in context: {}",
            tool_name, outcome, context
        );

        // Resolve tool name to entity
        let tool_entity = self
            .entity_service
            .resolve(tool_name)
            .await?
            .ok_or_else(|| {
                IndexError::NotFound(format!(
                    "Tool '{}' not found. Register it first with 'entity create {} -t tool'",
                    tool_name, tool_name
                ))
            })?;

        // Verify it's a tool type
        if tool_entity.entity_type != EntityType::Tool {
            return Err(IndexError::InvalidState(format!(
                "Entity '{}' is not a tool (type: {})",
                tool_name, tool_entity.entity_type
            )));
        }

        let mut usage = ToolUsage::new(tool_entity.id, context, outcome);

        if let Some(sid) = session_id {
            usage = usage.with_session(sid.clone());
        }

        self.repo.save_usage(&usage).await?;

        // Update learned preferences based on this usage
        self.update_preferences_from_usage(&usage).await?;

        info!("Tool usage logged: {} ({})", usage.id, usage.outcome);
        Ok(usage)
    }

    /// Log a tool switch (user switched from one tool to another).
    pub async fn log_switch(
        &self,
        from_tool: &str,
        to_tool: &str,
        context: &str,
        session_id: Option<&Id>,
    ) -> IndexResult<ToolUsage> {
        info!(
            "Logging tool switch: {} -> {} in context: {}",
            from_tool, to_tool, context
        );

        // Resolve both tools
        let from_entity = self
            .entity_service
            .resolve(from_tool)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Tool not found: {}", from_tool)))?;

        let to_entity = self
            .entity_service
            .resolve(to_tool)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Tool not found: {}", to_tool)))?;

        let mut usage = ToolUsage::new(from_entity.id, context, ToolOutcome::Switched)
            .with_switched_to(to_entity.id);

        if let Some(sid) = session_id {
            usage = usage.with_session(sid.clone());
        }

        self.repo.save_usage(&usage).await?;

        info!("Tool switch logged: {} -> {}", from_tool, to_tool);
        Ok(usage)
    }

    // =========================================================================
    // Recommendations
    // =========================================================================

    /// Get tool recommendations for a given context.
    ///
    /// Returns tools sorted by confidence/success rate for the given context.
    pub async fn get_recommendations(&self, context: &str) -> IndexResult<Vec<ToolRecommendation>> {
        debug!("Getting recommendations for context: {}", context);

        let mut recommendations = Vec::new();

        // First, check learned preferences
        let preferences = self.repo.get_preferences_for_context(context).await?;

        for pref in preferences {
            // Get tool entity for name
            if let Some(entity) = self
                .entity_service
                .get_entity(&pref.preferred_tool_id)
                .await?
            {
                recommendations.push(ToolRecommendation {
                    tool_id: pref.preferred_tool_id,
                    tool_name: entity.name.clone(),
                    confidence: pref.confidence,
                    reason: format!(
                        "Based on {} previous usages with {:.0}% success rate",
                        pref.sample_count,
                        pref.confidence * 100.0
                    ),
                });
            }
        }

        // If no preferences found, search usage history for similar contexts
        if recommendations.is_empty() {
            let similar_usages = self.repo.search_usages(context, Some(20)).await?;

            // Group by tool and calculate success rate
            let mut tool_stats: std::collections::HashMap<Id, (usize, usize, String)> =
                std::collections::HashMap::new();

            for usage in similar_usages {
                let entry =
                    tool_stats
                        .entry(usage.tool_id.clone())
                        .or_insert((0, 0, String::new()));
                entry.0 += 1; // total
                if usage.outcome == ToolOutcome::Success {
                    entry.1 += 1; // success
                }
                // Get tool name if we don't have it
                if entry.2.is_empty() {
                    if let Some(entity) = self.entity_service.get_entity(&usage.tool_id).await? {
                        entry.2 = entity.name;
                    }
                }
            }

            // Convert to recommendations
            for (tool_id, (total, success, name)) in tool_stats {
                if !name.is_empty() {
                    let rate = success as f32 / total as f32;
                    recommendations.push(ToolRecommendation {
                        tool_id,
                        tool_name: name,
                        confidence: rate,
                        reason: format!(
                            "Based on {} similar usages with {:.0}% success rate",
                            total,
                            rate * 100.0
                        ),
                    });
                }
            }
        }

        // Sort by confidence
        recommendations.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(recommendations)
    }

    // =========================================================================
    // Statistics
    // =========================================================================

    /// Get statistics for a specific tool.
    pub async fn get_tool_stats(&self, tool_name: &str) -> IndexResult<ToolStats> {
        debug!("Getting stats for tool: {}", tool_name);

        let tool_entity = self
            .entity_service
            .resolve(tool_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Tool not found: {}", tool_name)))?;

        Ok(self.repo.tool_stats(&tool_entity.id).await?)
    }

    /// Get success rate for a tool.
    pub async fn get_success_rate(&self, tool_name: &str) -> IndexResult<f32> {
        debug!("Getting success rate for tool: {}", tool_name);

        let tool_entity = self
            .entity_service
            .resolve(tool_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Tool not found: {}", tool_name)))?;

        Ok(self.repo.calculate_success_rate(&tool_entity.id).await?)
    }

    /// Get overall tool intelligence statistics.
    pub async fn stats(&self) -> IndexResult<ToolIntelStats> {
        Ok(self.repo.stats().await?)
    }

    // =========================================================================
    // Usage Queries
    // =========================================================================

    /// List recent tool usages.
    pub async fn list_usages(
        &self,
        outcome: Option<&ToolOutcome>,
        limit: Option<usize>,
    ) -> IndexResult<Vec<ToolUsageInfo>> {
        debug!("Listing tool usages");

        let usages = self.repo.list_usages(outcome, limit).await?;

        let mut results = Vec::new();
        for usage in usages {
            // Get tool name
            let tool_name =
                if let Some(entity) = self.entity_service.get_entity(&usage.tool_id).await? {
                    entity.name
                } else {
                    usage.tool_id.to_string()
                };

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

    /// Search usage history.
    pub async fn search_usages(
        &self,
        query: &str,
        limit: Option<usize>,
    ) -> IndexResult<Vec<ToolUsageInfo>> {
        debug!("Searching tool usages: {}", query);

        let usages = self.repo.search_usages(query, limit).await?;

        let mut results = Vec::new();
        for usage in usages {
            let tool_name =
                if let Some(entity) = self.entity_service.get_entity(&usage.tool_id).await? {
                    entity.name
                } else {
                    usage.tool_id.to_string()
                };

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

    // =========================================================================
    // Learning
    // =========================================================================

    /// Learn preferences from all usage data.
    ///
    /// Analyzes usage history to find patterns and update preferences.
    pub async fn learn_preferences(&self) -> IndexResult<usize> {
        info!("Learning preferences from usage data");

        // Get all tools
        let tools = self
            .entity_service
            .list_entities(Some(&EntityType::Tool))
            .await?;

        let mut learned = 0;

        for tool in tools {
            let usages = self.repo.get_usages_for_tool(&tool.id).await?;

            if usages.is_empty() {
                continue;
            }

            // Group by context pattern (simplified: use exact context)
            let mut context_stats: std::collections::HashMap<String, (usize, usize)> =
                std::collections::HashMap::new();

            for usage in usages {
                let entry = context_stats.entry(usage.context.clone()).or_insert((0, 0));
                entry.0 += 1; // total
                if usage.outcome == ToolOutcome::Success {
                    entry.1 += 1; // success
                }
            }

            // Create preferences for contexts with enough samples
            for (context, (total, success)) in context_stats {
                if total >= 2 {
                    let confidence = success as f32 / total as f32;

                    // Only create preference if success rate is decent
                    if confidence >= 0.5 {
                        let pref = ToolPreference::new(
                            &context,
                            tool.id.clone(),
                            confidence,
                            total as u32,
                        );
                        self.repo.save_preference(&pref).await?;
                        learned += 1;
                    }
                }
            }
        }

        info!("Learned {} preferences", learned);
        Ok(learned)
    }

    /// Update preferences based on a single usage.
    async fn update_preferences_from_usage(&self, usage: &ToolUsage) -> IndexResult<()> {
        // Check if we have enough samples for this context
        let usages = self.repo.search_usages(&usage.context, Some(100)).await?;

        // Filter to usages with the same tool
        let tool_usages: Vec<_> = usages
            .iter()
            .filter(|u| u.tool_id == usage.tool_id)
            .collect();

        if tool_usages.len() >= 2 {
            let success_count = tool_usages
                .iter()
                .filter(|u| u.outcome == ToolOutcome::Success)
                .count();

            let confidence = success_count as f32 / tool_usages.len() as f32;

            if confidence >= 0.5 {
                let pref = ToolPreference::new(
                    &usage.context,
                    usage.tool_id.clone(),
                    confidence,
                    tool_usages.len() as u32,
                );
                self.repo.save_preference(&pref).await?;
            }
        }

        Ok(())
    }
}

/// Tool usage information with resolved names.
#[derive(Debug, Clone)]
pub struct ToolUsageInfo {
    /// Usage ID.
    pub id: Id,
    /// Tool name.
    pub tool_name: String,
    /// Context of usage.
    pub context: String,
    /// Outcome.
    pub outcome: ToolOutcome,
    /// Timestamp.
    pub timestamp: time::OffsetDateTime,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_usage_info_fields() {
        // Verify the struct has the expected fields
        let info = ToolUsageInfo {
            id: Id::new(),
            tool_name: "bzl".to_string(),
            context: "building go service".to_string(),
            outcome: ToolOutcome::Success,
            timestamp: time::OffsetDateTime::now_utc(),
        };

        assert_eq!(info.tool_name, "bzl");
        assert_eq!(info.outcome, ToolOutcome::Success);
    }
}
