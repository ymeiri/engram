//! Entity knowledge types (Layer 1).
//!
//! Entities represent the fundamental knowledge objects in engram:
//! repositories, tools, terminology, concepts, services, etc.
//!
//! Entities can have:
//! - Properties (flexible key-value attributes)
//! - Relationships (graph edges to other entities)
//! - Aliases (alternative names for term detection)
//! - Observations (timestamped notes/facts)

use crate::id::Id;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::OffsetDateTime;

/// The type of an entity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    /// A code repository.
    Repo,
    /// A tool or CLI command.
    Tool,
    /// A concept or term (jargon).
    Concept,
    /// A deployment method or target.
    Deployment,
    /// A topic (for documentation shortcuts).
    Topic,
    /// A workflow or process.
    Workflow,
    /// A person.
    Person,
    /// A team.
    Team,
    /// A service or API.
    Service,
    /// Custom type.
    Custom(String),
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Repo => write!(f, "repo"),
            Self::Tool => write!(f, "tool"),
            Self::Concept => write!(f, "concept"),
            Self::Deployment => write!(f, "deployment"),
            Self::Topic => write!(f, "topic"),
            Self::Workflow => write!(f, "workflow"),
            Self::Person => write!(f, "person"),
            Self::Team => write!(f, "team"),
            Self::Service => write!(f, "service"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl EntityType {
    /// Parse from string.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "repo" => Self::Repo,
            "tool" => Self::Tool,
            "concept" => Self::Concept,
            "deployment" => Self::Deployment,
            "topic" => Self::Topic,
            "workflow" => Self::Workflow,
            "person" => Self::Person,
            "team" => Self::Team,
            "service" => Self::Service,
            other => Self::Custom(other.to_string()),
        }
    }
}

/// An entity in the knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Unique identifier.
    pub id: Id,

    /// Human-readable name.
    pub name: String,

    /// Type of entity.
    pub entity_type: EntityType,

    /// Description of the entity.
    pub description: Option<String>,

    /// Flexible properties (key-value).
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,

    /// Embedding vector for semantic search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,

    /// Creation timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,

    /// Last update timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl Entity {
    /// Create a new entity.
    #[must_use]
    pub fn new(name: impl Into<String>, entity_type: EntityType) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: Id::new(),
            name: name.into(),
            entity_type,
            description: None,
            properties: HashMap::new(),
            embedding: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Set the embedding.
    #[must_use]
    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }

    /// Set the description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a property.
    #[must_use]
    pub fn with_property(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.properties.insert(key.into(), value);
        self
    }

    /// Get a property value.
    pub fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.properties.get(key)
    }

    /// Get a property as a string.
    pub fn get_property_str(&self, key: &str) -> Option<&str> {
        self.properties.get(key).and_then(|v| v.as_str())
    }
}

/// The type of relationship between entities.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    /// Source depends on target.
    DependsOn,
    /// Source uses target.
    Uses,
    /// Source is deployed via target.
    DeployedVia,
    /// Source is owned by target.
    OwnedBy,
    /// Source documents target.
    Documents,
    /// Source is related to target (generic).
    RelatedTo,
    /// Custom relationship.
    Custom(String),
}

impl std::fmt::Display for RelationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DependsOn => write!(f, "depends_on"),
            Self::Uses => write!(f, "uses"),
            Self::DeployedVia => write!(f, "deployed_via"),
            Self::OwnedBy => write!(f, "owned_by"),
            Self::Documents => write!(f, "documents"),
            Self::RelatedTo => write!(f, "related_to"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl RelationType {
    /// Parse from string.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "depends_on" => Self::DependsOn,
            "uses" => Self::Uses,
            "deployed_via" => Self::DeployedVia,
            "owned_by" => Self::OwnedBy,
            "documents" => Self::Documents,
            "related_to" => Self::RelatedTo,
            other => Self::Custom(other.to_string()),
        }
    }
}

/// A relationship between two entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    /// Source entity ID.
    pub source_id: Id,

    /// Target entity ID.
    pub target_id: Id,

    /// Type of relationship.
    pub relation_type: RelationType,

    /// Optional description.
    pub description: Option<String>,

    /// Optional weight (for ranking).
    pub weight: Option<f32>,

    /// Creation timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

impl Relationship {
    /// Create a new relationship.
    #[must_use]
    pub fn new(source_id: Id, target_id: Id, relation_type: RelationType) -> Self {
        Self {
            source_id,
            target_id,
            relation_type,
            description: None,
            weight: None,
            created_at: OffsetDateTime::now_utc(),
        }
    }
}

/// An alias for an entity (for term detection).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alias {
    /// The alias text.
    pub name: String,

    /// The canonical entity ID.
    pub entity_id: Id,
}

impl Alias {
    /// Create a new alias.
    #[must_use]
    pub fn new(name: impl Into<String>, entity_id: Id) -> Self {
        Self {
            name: name.into(),
            entity_id,
        }
    }
}

/// An observation (fact/note) attached to an entity.
///
/// Observations can have an optional `key` for semantic identification.
/// When a key is provided, observations with the same (entity_id, key) pair
/// will be updated (upsert) rather than creating duplicates.
///
/// Recommended key format: `<category>.<subcategory>.<specific>`
/// Categories: architecture, patterns, gotchas, decisions, dependencies,
///             config, testing, performance, security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    /// Unique identifier.
    pub id: Id,

    /// The entity this observation is about.
    pub entity_id: Id,

    /// Semantic key for updates (optional).
    /// Format: `category.subcategory` (e.g., "architecture.auth", "gotchas.race-conditions")
    /// When provided, same key on same entity = update existing observation.
    pub key: Option<String>,

    /// The observation content.
    pub content: String,

    /// Source of the observation.
    pub source: Option<String>,

    /// Embedding vector for semantic search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,

    /// Creation timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,

    /// Last update timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl Observation {
    /// Create a new observation.
    #[must_use]
    pub fn new(entity_id: Id, content: impl Into<String>) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: Id::new(),
            entity_id,
            key: None,
            content: content.into(),
            source: None,
            embedding: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Set the semantic key for updates.
    /// Format: `category.subcategory` (e.g., "architecture.auth")
    #[must_use]
    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Set the source.
    #[must_use]
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Set the embedding.
    #[must_use]
    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_creation() {
        let entity = Entity::new("web-ui", EntityType::Repo)
            .with_description("Main frontend repository")
            .with_property(
                "github_url",
                serde_json::json!("https://github.com/example/web-ui"),
            );

        assert_eq!(entity.name, "web-ui");
        assert_eq!(entity.entity_type, EntityType::Repo);
        assert_eq!(
            entity.description,
            Some("Main frontend repository".to_string())
        );
        assert_eq!(
            entity.get_property_str("github_url"),
            Some("https://github.com/example/web-ui")
        );
    }

    #[test]
    fn test_relationship_creation() {
        let source = Id::new();
        let target = Id::new();
        let rel = Relationship::new(source, target, RelationType::DependsOn);

        assert_eq!(rel.source_id, source);
        assert_eq!(rel.target_id, target);
        assert_eq!(rel.relation_type, RelationType::DependsOn);
    }
}
