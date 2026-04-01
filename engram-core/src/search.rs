//! Unified search types for cross-layer search.
//!
//! Provides types for the unified search tool that searches
//! across all knowledge layers with a single query.

use serde::{Deserialize, Serialize};

/// Source layer for a search result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchResultSource {
    /// Entity name match
    Entity,
    /// Entity alias match
    Alias,
    /// Entity observation content match
    Observation,
    /// Session event content match
    SessionEvent,
    /// Document chunk match (semantic search)
    Document,
    /// Tool usage context match
    ToolUsage,
}

impl std::fmt::Display for SearchResultSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Entity => write!(f, "entity"),
            Self::Alias => write!(f, "alias"),
            Self::Observation => write!(f, "observation"),
            Self::SessionEvent => write!(f, "session_event"),
            Self::Document => write!(f, "document"),
            Self::ToolUsage => write!(f, "tool_usage"),
        }
    }
}

/// A unified search result from any layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSearchResult {
    /// Which layer the result came from
    pub source: SearchResultSource,
    /// Relevance score (0.0-1.0, higher is better)
    pub score: f32,
    /// Primary title (entity name, doc title, etc.)
    pub title: String,
    /// Matching content snippet
    pub content: String,
    /// Context information (parent entity, session ID, etc.)
    pub context: Option<String>,
    /// ID for follow-up queries
    pub id: String,
}

impl UnifiedSearchResult {
    /// Create a new unified search result.
    pub fn new(
        source: SearchResultSource,
        score: f32,
        title: impl Into<String>,
        content: impl Into<String>,
        id: impl Into<String>,
    ) -> Self {
        Self {
            source,
            score,
            title: title.into(),
            content: content.into(),
            context: None,
            id: id.into(),
        }
    }

    /// Add context to the result.
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

/// Layers that can be searched.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchLayer {
    /// Entity names and descriptions
    Entity,
    /// Entity aliases
    Alias,
    /// Entity observations
    Observation,
    /// Session events
    SessionEvent,
    /// Indexed documents (semantic search)
    Document,
    /// Tool usage history
    ToolUsage,
}

impl SearchLayer {
    /// Parse a layer from string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "entity" => Some(Self::Entity),
            "alias" => Some(Self::Alias),
            "observation" => Some(Self::Observation),
            "session_event" | "session" => Some(Self::SessionEvent),
            "document" | "doc" => Some(Self::Document),
            "tool_usage" | "tool" => Some(Self::ToolUsage),
            _ => None,
        }
    }

    /// All available layers.
    pub fn all() -> Vec<Self> {
        vec![
            Self::Entity,
            Self::Alias,
            Self::Observation,
            Self::SessionEvent,
            Self::Document,
            Self::ToolUsage,
        ]
    }
}

impl std::fmt::Display for SearchLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Entity => write!(f, "entity"),
            Self::Alias => write!(f, "alias"),
            Self::Observation => write!(f, "observation"),
            Self::SessionEvent => write!(f, "session_event"),
            Self::Document => write!(f, "document"),
            Self::ToolUsage => write!(f, "tool_usage"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_result_source_display() {
        assert_eq!(SearchResultSource::Entity.to_string(), "entity");
        assert_eq!(SearchResultSource::Observation.to_string(), "observation");
    }

    #[test]
    fn test_search_layer_parse() {
        assert_eq!(SearchLayer::parse("entity"), Some(SearchLayer::Entity));
        assert_eq!(SearchLayer::parse("doc"), Some(SearchLayer::Document));
        assert_eq!(
            SearchLayer::parse("session"),
            Some(SearchLayer::SessionEvent)
        );
        assert_eq!(SearchLayer::parse("unknown"), None);
    }

    #[test]
    fn test_unified_search_result_builder() {
        let result = UnifiedSearchResult::new(
            SearchResultSource::Entity,
            0.85,
            "my-service",
            "A microservice",
            "abc123",
        )
        .with_context("repo: my-project");

        assert_eq!(result.title, "my-service");
        assert_eq!(result.context, Some("repo: my-project".to_string()));
    }
}
