//! Entity repository for Layer 1: Entity Knowledge.
//!
//! Handles persistence of Entity, Relationship, Alias, and Observation.
//! Uses SurrealDB's graph capabilities for relationship queries.

use crate::error::{StoreError, StoreResult};
use crate::Db;
use engram_core::entity::{Alias, Entity, EntityType, Observation, RelationType, Relationship};
use engram_core::id::Id;
use serde::Deserialize;
use std::collections::HashMap;
use tracing::{debug, info};

/// SurrealDB datetime representation (handles both string and native formats).
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum SurrealDateTime {
    /// ISO 8601 string format
    String(String),
    /// Native SurrealDB datetime format (array of integers)
    Native(serde_json::Value),
}

impl SurrealDateTime {
    fn to_offset_datetime(&self) -> StoreResult<time::OffsetDateTime> {
        match self {
            SurrealDateTime::String(s) => {
                time::OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339)
                    .map_err(|e| StoreError::Deserialization(format!("Invalid datetime: {}", e)))
            }
            SurrealDateTime::Native(v) => {
                // SurrealDB native format: [year, month, day, hour, min, sec, nano, offset_h, offset_m]
                if let Some(arr) = v.as_array() {
                    if arr.len() >= 6 {
                        let year = arr[0].as_i64().unwrap_or(2000) as i32;
                        let month = arr[1].as_i64().unwrap_or(1) as u8;
                        let day = arr[2].as_i64().unwrap_or(1) as u8;
                        let hour = arr[3].as_i64().unwrap_or(0) as u8;
                        let min = arr[4].as_i64().unwrap_or(0) as u8;
                        let sec = arr[5].as_i64().unwrap_or(0) as u8;

                        let date = time::Date::from_calendar_date(
                            year,
                            time::Month::try_from(month).unwrap_or(time::Month::January),
                            day,
                        )
                        .unwrap_or(
                            time::Date::from_calendar_date(2000, time::Month::January, 1).unwrap(),
                        );

                        let time_val =
                            time::Time::from_hms(hour, min, sec).unwrap_or(time::Time::MIDNIGHT);

                        return Ok(time::OffsetDateTime::new_utc(date, time_val));
                    }
                }
                Ok(time::OffsetDateTime::now_utc())
            }
        }
    }
}

/// Entity record from SurrealDB with proper deserialization.
#[derive(Debug, Deserialize)]
struct EntityRecord {
    name: String,
    entity_type: String,
    description: Option<String>,
    #[serde(default)]
    properties: HashMap<String, serde_json::Value>,
    #[serde(default)]
    embedding: Option<Vec<f32>>,
    created_at: SurrealDateTime,
    updated_at: SurrealDateTime,
}

/// Relationship record from SurrealDB.
#[derive(Debug, Deserialize)]
struct RelationshipRecord {
    source_id: String,
    target_id: String,
    relation_type: String,
    description: Option<String>,
    weight: Option<f32>,
    created_at: SurrealDateTime,
}

/// Alias record from SurrealDB.
#[derive(Debug, Deserialize)]
struct AliasRecord {
    name: String,
    entity_id: String,
}

/// Result of an alias search.
#[derive(Debug, Clone)]
pub struct AliasSearchResult {
    /// The alias name that matched.
    pub alias_name: String,
    /// The entity ID this alias points to.
    pub entity_id: Id,
}

/// Result of a vector similarity search for entities.
#[derive(Debug, Clone)]
pub struct EntitySearchResult {
    /// The matching entity.
    pub entity: Entity,
    /// Cosine similarity score (0.0 to 1.0).
    pub score: f32,
}

/// Result of a vector similarity search for observations.
#[derive(Debug, Clone)]
pub struct ObservationSearchResult {
    /// The matching observation.
    pub observation: Observation,
    /// Cosine similarity score (0.0 to 1.0).
    pub score: f32,
}

/// Compute cosine similarity between two vectors.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

/// Alias name only record for list queries.
#[derive(Debug, Deserialize)]
struct AliasNameRecord {
    name: String,
}

/// Observation record from SurrealDB.
#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Kept for consistency with other record types
struct ObservationRecord {
    entity_id: String,
    key: Option<String>,
    content: String,
    source: Option<String>,
    #[serde(default)]
    embedding: Option<Vec<f32>>,
    created_at: SurrealDateTime,
    updated_at: SurrealDateTime,
}

/// Archived observation record from SurrealDB.
#[derive(Debug, Deserialize)]
struct ObservationArchiveRecord {
    observation_id: String,
    entity_id: String,
    key: Option<String>,
    content: String,
    source: Option<String>,
    created_at: SurrealDateTime,
    archived_at: SurrealDateTime,
}

/// Table names for entity storage.
const TABLE_ENTITY: &str = "entity";
const TABLE_ALIAS: &str = "entity_alias";
const TABLE_OBSERVATION: &str = "entity_observation";
const TABLE_OBSERVATION_ARCHIVE: &str = "entity_observation_archive";
const TABLE_RELATIONSHIP: &str = "entity_relationship";

/// Repository for entity operations.
#[derive(Clone)]
pub struct EntityRepo {
    db: Db,
}

impl EntityRepo {
    /// Create a new entity repository.
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Initialize the entity schema.
    pub async fn init_schema(&self) -> StoreResult<()> {
        info!("Initializing entity schema (Layer 1)");

        // Entity table - SCHEMALESS for flexible properties
        self.db
            .query(format!(
                r#"
                DEFINE TABLE IF NOT EXISTS {TABLE_ENTITY} SCHEMALESS;
                DEFINE INDEX IF NOT EXISTS idx_entity_type ON {TABLE_ENTITY} FIELDS entity_type;
                DEFINE INDEX IF NOT EXISTS idx_entity_name ON {TABLE_ENTITY} FIELDS name;
                "#
            ))
            .await?;

        // Alias table for term detection
        self.db
            .query(format!(
                r#"
                DEFINE TABLE IF NOT EXISTS {TABLE_ALIAS} SCHEMALESS;
                DEFINE INDEX IF NOT EXISTS idx_alias_name ON {TABLE_ALIAS} FIELDS name;
                DEFINE INDEX IF NOT EXISTS idx_alias_entity ON {TABLE_ALIAS} FIELDS entity_id;
                "#
            ))
            .await?;

        // Observation table - uniqueness for keyed observations is handled in application logic
        // (we check for existing key and update rather than insert)
        self.db
            .query(format!(
                r#"
                DEFINE TABLE IF NOT EXISTS {TABLE_OBSERVATION} SCHEMALESS;
                DEFINE INDEX IF NOT EXISTS idx_obs_entity ON {TABLE_OBSERVATION} FIELDS entity_id;
                DEFINE INDEX IF NOT EXISTS idx_obs_key ON {TABLE_OBSERVATION} FIELDS key;
                DEFINE INDEX IF NOT EXISTS idx_obs_entity_key ON {TABLE_OBSERVATION} FIELDS entity_id, key;
                "#
            ))
            .await?;

        // Observation archive table for history tracking
        self.db
            .query(format!(
                r#"
                DEFINE TABLE IF NOT EXISTS {TABLE_OBSERVATION_ARCHIVE} SCHEMALESS;
                DEFINE INDEX IF NOT EXISTS idx_obs_archive_obs ON {TABLE_OBSERVATION_ARCHIVE} FIELDS observation_id;
                DEFINE INDEX IF NOT EXISTS idx_obs_archive_entity ON {TABLE_OBSERVATION_ARCHIVE} FIELDS entity_id;
                DEFINE INDEX IF NOT EXISTS idx_obs_archive_key ON {TABLE_OBSERVATION_ARCHIVE} FIELDS key;
                "#
            ))
            .await?;

        // Relationship table (simple edge table - we manage in/out manually for SurrealDB v2 compat)
        self.db
            .query(format!(
                r#"
                DEFINE TABLE IF NOT EXISTS {TABLE_RELATIONSHIP} SCHEMALESS;
                DEFINE INDEX IF NOT EXISTS idx_rel_type ON {TABLE_RELATIONSHIP} FIELDS relation_type;
                DEFINE INDEX IF NOT EXISTS idx_rel_source ON {TABLE_RELATIONSHIP} FIELDS source_id;
                DEFINE INDEX IF NOT EXISTS idx_rel_target ON {TABLE_RELATIONSHIP} FIELDS target_id;
                "#
            ))
            .await?;

        info!("Entity schema initialized");
        Ok(())
    }

    // =========================================================================
    // Entity Operations
    // =========================================================================

    /// Save an entity.
    pub async fn save_entity(&self, entity: &Entity) -> StoreResult<()> {
        debug!("Saving entity: {} ({})", entity.name, entity.entity_type);

        self.db
            .query(
                r#"
                UPSERT type::thing("entity", $id) SET
                    name = $name,
                    entity_type = $entity_type,
                    description = $description,
                    properties = $properties,
                    embedding = $embedding,
                    created_at = $created_at,
                    updated_at = $updated_at
            "#,
            )
            .bind(("id", entity.id.to_string()))
            .bind(("name", entity.name.clone()))
            .bind(("entity_type", entity.entity_type.to_string()))
            .bind(("description", entity.description.clone()))
            .bind(("properties", entity.properties.clone()))
            .bind(("embedding", entity.embedding.clone()))
            .bind((
                "created_at",
                entity
                    .created_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            ))
            .bind((
                "updated_at",
                entity
                    .updated_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            ))
            .await?;

        Ok(())
    }

    /// Get an entity by ID.
    pub async fn get_entity(&self, id: &Id) -> StoreResult<Option<Entity>> {
        debug!("Getting entity: {}", id);

        let mut result = self
            .db
            .query(r#"SELECT * FROM type::thing("entity", $id)"#)
            .bind(("id", id.to_string()))
            .await?;

        let records: Vec<EntityRecord> = result.take(0)?;

        if let Some(record) = records.into_iter().next() {
            Ok(Some(self.record_to_entity(id.clone(), record)?))
        } else {
            Ok(None)
        }
    }

    /// Get an entity by name.
    pub async fn get_entity_by_name(&self, name: &str) -> StoreResult<Option<Entity>> {
        debug!("Getting entity by name: {}", name);

        // Use meta::id(id) to ensure the id is returned as a string
        let mut result = self.db
            .query("SELECT meta::id(id) as id, name, entity_type, description, properties, embedding, created_at, updated_at FROM entity WHERE name = $name LIMIT 1")
            .bind(("name", name.to_string()))
            .await?;

        let records: Vec<EntityRecordWithId> = result.take(0)?;

        if let Some(record) = records.into_iter().next() {
            let id = self.parse_record_id(&record.id)?;
            Ok(Some(self.record_to_entity(id, record.into())?))
        } else {
            Ok(None)
        }
    }

    /// List all entities, optionally filtered by type.
    pub async fn list_entities(
        &self,
        entity_type: Option<&EntityType>,
    ) -> StoreResult<Vec<Entity>> {
        debug!("Listing entities (type filter: {:?})", entity_type);

        let query = match entity_type {
            Some(et) => format!(
                "SELECT meta::id(id) as id, name, entity_type, description, properties, created_at, updated_at FROM {} WHERE entity_type = '{}' ORDER BY name",
                TABLE_ENTITY,
                et
            ),
            None => format!("SELECT meta::id(id) as id, name, entity_type, description, properties, created_at, updated_at FROM {} ORDER BY name", TABLE_ENTITY),
        };

        let mut result = self.db.query(query).await?;
        let records: Vec<EntityRecordWithId> = result.take(0)?;

        let mut entities = Vec::new();
        for record in records {
            let id = self.parse_record_id(&record.id)?;
            entities.push(self.record_to_entity(id, record.into())?);
        }

        Ok(entities)
    }

    /// Search entities by name (partial match).
    pub async fn search_entities(&self, query: &str) -> StoreResult<Vec<Entity>> {
        debug!("Searching entities: {}", query);

        let mut result = self.db
            .query("SELECT meta::id(id) as id, name, entity_type, description, properties, created_at, updated_at FROM entity WHERE string::lowercase(name) CONTAINS $query ORDER BY name")
            .bind(("query", query.to_lowercase()))
            .await?;

        let records: Vec<EntityRecordWithId> = result.take(0)?;

        let mut entities = Vec::new();
        for record in records {
            let id = self.parse_record_id(&record.id)?;
            entities.push(self.record_to_entity(id, record.into())?);
        }

        Ok(entities)
    }

    /// Delete an entity by ID.
    pub async fn delete_entity(&self, id: &Id) -> StoreResult<()> {
        debug!("Deleting entity: {}", id);

        // Delete associated relationships
        self.db
            .query(r#"DELETE entity_relationship WHERE source_id = $id OR target_id = $id"#)
            .bind(("id", id.to_string()))
            .await?;

        // Delete associated aliases
        self.db
            .query("DELETE FROM entity_alias WHERE entity_id = $id")
            .bind(("id", id.to_string()))
            .await?;

        // Delete associated observations
        self.db
            .query("DELETE FROM entity_observation WHERE entity_id = $id")
            .bind(("id", id.to_string()))
            .await?;

        // Delete the entity - SurrealDB v2: use type::thing with double quotes
        self.db
            .query(r#"DELETE type::thing("entity", $id)"#)
            .bind(("id", id.to_string()))
            .await?;

        Ok(())
    }

    // =========================================================================
    // Relationship Operations (Graph)
    // =========================================================================

    /// Create a relationship between entities.
    pub async fn create_relationship(&self, rel: &Relationship) -> StoreResult<()> {
        debug!(
            "Creating relationship: {} --[{}]--> {}",
            rel.source_id, rel.relation_type, rel.target_id
        );

        // SurrealDB v2: Use simple table with source_id/target_id string fields
        // This avoids RELATE's syntax issues with dynamic record IDs
        self.db
            .query(
                r#"
                CREATE entity_relationship SET
                    source_id = $source_id,
                    target_id = $target_id,
                    relation_type = $relation_type,
                    description = $description,
                    weight = $weight,
                    created_at = $created_at
            "#,
            )
            .bind(("source_id", rel.source_id.to_string()))
            .bind(("target_id", rel.target_id.to_string()))
            .bind(("relation_type", rel.relation_type.to_string()))
            .bind(("description", rel.description.clone()))
            .bind(("weight", rel.weight))
            .bind((
                "created_at",
                rel.created_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            ))
            .await?;

        Ok(())
    }

    /// Get relationships from an entity (outgoing).
    pub async fn get_relationships_from(&self, entity_id: &Id) -> StoreResult<Vec<Relationship>> {
        debug!("Getting relationships from: {}", entity_id);

        // Query relationships where source_id matches
        let mut result = self
            .db
            .query(
                r#"SELECT source_id, target_id, relation_type, description, weight, created_at
                      FROM entity_relationship
                      WHERE source_id = $id"#,
            )
            .bind(("id", entity_id.to_string()))
            .await?;

        let records: Vec<RelationshipRecord> = result.take(0)?;
        self.records_to_relationships(records)
    }

    /// Get relationships to an entity (incoming).
    pub async fn get_relationships_to(&self, entity_id: &Id) -> StoreResult<Vec<Relationship>> {
        debug!("Getting relationships to: {}", entity_id);

        // Query relationships where target_id matches
        let mut result = self
            .db
            .query(
                r#"SELECT source_id, target_id, relation_type, description, weight, created_at
                      FROM entity_relationship
                      WHERE target_id = $id"#,
            )
            .bind(("id", entity_id.to_string()))
            .await?;

        let records: Vec<RelationshipRecord> = result.take(0)?;
        self.records_to_relationships(records)
    }

    /// Get all relationships for an entity (both directions).
    pub async fn get_all_relationships(&self, entity_id: &Id) -> StoreResult<Vec<Relationship>> {
        debug!("Getting all relationships for: {}", entity_id);

        // Query relationships where either source_id or target_id matches
        let mut result = self
            .db
            .query(
                r#"SELECT source_id, target_id, relation_type, description, weight, created_at
                      FROM entity_relationship
                      WHERE source_id = $id OR target_id = $id"#,
            )
            .bind(("id", entity_id.to_string()))
            .await?;

        let records: Vec<RelationshipRecord> = result.take(0)?;
        self.records_to_relationships(records)
    }

    /// Delete a relationship.
    pub async fn delete_relationship(
        &self,
        source_id: &Id,
        target_id: &Id,
        relation_type: &RelationType,
    ) -> StoreResult<()> {
        debug!(
            "Deleting relationship: {} --[{}]--> {}",
            source_id, relation_type, target_id
        );

        // Delete relationship matching source_id, target_id, and relation_type
        self.db
            .query(
                r#"DELETE entity_relationship
                      WHERE source_id = $source_id
                        AND target_id = $target_id
                        AND relation_type = $relation_type"#,
            )
            .bind(("source_id", source_id.to_string()))
            .bind(("target_id", target_id.to_string()))
            .bind(("relation_type", relation_type.to_string()))
            .await?;

        Ok(())
    }

    // =========================================================================
    // Alias Operations
    // =========================================================================

    /// Add an alias for an entity.
    pub async fn add_alias(&self, alias: &Alias) -> StoreResult<()> {
        debug!(
            "Adding alias '{}' for entity {}",
            alias.name, alias.entity_id
        );

        self.db
            .query(format!(
                "CREATE {} SET name = $name, entity_id = $entity_id",
                TABLE_ALIAS
            ))
            .bind(("name", alias.name.clone()))
            .bind(("entity_id", alias.entity_id.to_string()))
            .await?;

        Ok(())
    }

    /// Resolve an alias to an entity ID.
    pub async fn resolve_alias(&self, alias_name: &str) -> StoreResult<Option<Id>> {
        debug!("Resolving alias: {}", alias_name);

        let mut result = self.db
            .query("SELECT name, entity_id FROM entity_alias WHERE string::lowercase(name) = $name LIMIT 1")
            .bind(("name", alias_name.to_lowercase()))
            .await?;

        let records: Vec<AliasRecord> = result.take(0)?;

        if let Some(record) = records.into_iter().next() {
            let id = Id::parse(&record.entity_id)
                .map_err(|e| StoreError::Deserialization(format!("Invalid entity ID: {}", e)))?;
            Ok(Some(id))
        } else {
            Ok(None)
        }
    }

    /// Get all aliases for an entity.
    pub async fn get_aliases(&self, entity_id: &Id) -> StoreResult<Vec<String>> {
        debug!("Getting aliases for entity: {}", entity_id);

        let mut result = self
            .db
            .query("SELECT name FROM entity_alias WHERE entity_id = $id")
            .bind(("id", entity_id.to_string()))
            .await?;

        let records: Vec<AliasNameRecord> = result.take(0)?;
        Ok(records.into_iter().map(|r| r.name).collect())
    }

    /// Delete an alias.
    pub async fn delete_alias(&self, alias_name: &str, entity_id: &Id) -> StoreResult<()> {
        debug!("Deleting alias '{}' for entity {}", alias_name, entity_id);

        self.db
            .query("DELETE FROM entity_alias WHERE name = $name AND entity_id = $entity_id")
            .bind(("name", alias_name.to_string()))
            .bind(("entity_id", entity_id.to_string()))
            .await?;

        Ok(())
    }

    // =========================================================================
    // Observation Operations
    // =========================================================================

    /// Add or update an observation for an entity.
    /// If the observation has a key and one already exists with that key for the entity,
    /// the existing observation is archived and updated.
    pub async fn add_observation(&self, obs: &Observation) -> StoreResult<Option<Observation>> {
        debug!(
            "Adding/updating observation for entity {} (key: {:?})",
            obs.entity_id, obs.key
        );

        let mut previous: Option<Observation> = None;

        // If there's a key, check for existing observation and archive it
        if let Some(key) = &obs.key {
            if let Some(existing) = self.get_observation_by_key(&obs.entity_id, key).await? {
                // Archive the existing observation
                self.archive_observation(&existing).await?;
                previous = Some(existing.clone());

                // Update existing observation (reuse ID)
                self.db
                    .query(
                        r#"
                        UPDATE type::thing("entity_observation", $id) SET
                            content = $content,
                            source = $source,
                            embedding = $embedding,
                            updated_at = $updated_at
                    "#,
                    )
                    .bind(("id", existing.id.to_string()))
                    .bind(("content", obs.content.clone()))
                    .bind(("source", obs.source.clone()))
                    .bind(("embedding", obs.embedding.clone()))
                    .bind((
                        "updated_at",
                        obs.updated_at
                            .format(&time::format_description::well_known::Rfc3339)
                            .unwrap(),
                    ))
                    .await?;

                return Ok(previous);
            }
        }

        // Create new observation
        self.db
            .query(
                r#"
                CREATE type::thing("entity_observation", $id) SET
                    entity_id = $entity_id,
                    key = $key,
                    content = $content,
                    source = $source,
                    embedding = $embedding,
                    created_at = $created_at,
                    updated_at = $updated_at
            "#,
            )
            .bind(("id", obs.id.to_string()))
            .bind(("entity_id", obs.entity_id.to_string()))
            .bind(("key", obs.key.clone()))
            .bind(("content", obs.content.clone()))
            .bind(("source", obs.source.clone()))
            .bind(("embedding", obs.embedding.clone()))
            .bind((
                "created_at",
                obs.created_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            ))
            .bind((
                "updated_at",
                obs.updated_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            ))
            .await?;

        Ok(previous)
    }

    /// Archive an observation (for history tracking).
    async fn archive_observation(&self, obs: &Observation) -> StoreResult<()> {
        debug!("Archiving observation {} (key: {:?})", obs.id, obs.key);

        let now = time::OffsetDateTime::now_utc();
        self.db
            .query(
                r#"
                CREATE entity_observation_archive SET
                    observation_id = $observation_id,
                    entity_id = $entity_id,
                    key = $key,
                    content = $content,
                    source = $source,
                    created_at = $created_at,
                    archived_at = $archived_at
            "#,
            )
            .bind(("observation_id", obs.id.to_string()))
            .bind(("entity_id", obs.entity_id.to_string()))
            .bind(("key", obs.key.clone()))
            .bind(("content", obs.content.clone()))
            .bind(("source", obs.source.clone()))
            .bind((
                "created_at",
                obs.created_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            ))
            .bind((
                "archived_at",
                now.format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            ))
            .await?;

        Ok(())
    }

    /// Get an observation by key for an entity.
    pub async fn get_observation_by_key(
        &self,
        entity_id: &Id,
        key: &str,
    ) -> StoreResult<Option<Observation>> {
        debug!(
            "Getting observation by key: {} for entity {}",
            key, entity_id
        );

        let mut result = self.db
            .query("SELECT meta::id(id) as id, entity_id, key, content, source, created_at, updated_at FROM entity_observation WHERE entity_id = $entity_id AND key = $key LIMIT 1")
            .bind(("entity_id", entity_id.to_string()))
            .bind(("key", key.to_string()))
            .await?;

        let records: Vec<ObservationRecordWithId> = result.take(0)?;

        if let Some(record) = records.into_iter().next() {
            Ok(Some(self.record_to_observation(record)?))
        } else {
            Ok(None)
        }
    }

    /// Get observations for an entity.
    pub async fn get_observations(&self, entity_id: &Id) -> StoreResult<Vec<Observation>> {
        debug!("Getting observations for entity: {}", entity_id);

        let mut result = self.db
            .query("SELECT meta::id(id) as id, entity_id, key, content, source, created_at, updated_at FROM entity_observation WHERE entity_id = $id ORDER BY updated_at DESC")
            .bind(("id", entity_id.to_string()))
            .await?;

        let records: Vec<ObservationRecordWithId> = result.take(0)?;

        let mut observations = Vec::new();
        for record in records {
            observations.push(self.record_to_observation(record)?);
        }

        Ok(observations)
    }

    /// List observations for an entity with optional key pattern filtering.
    /// Pattern uses glob-style syntax (* for wildcard).
    pub async fn list_observations_by_pattern(
        &self,
        entity_id: &Id,
        key_pattern: Option<&str>,
    ) -> StoreResult<Vec<Observation>> {
        debug!(
            "Listing observations for entity {} with pattern {:?}",
            entity_id, key_pattern
        );

        let mut result = if let Some(pattern) = key_pattern {
            // Convert glob-style pattern (e.g., "architecture.*") to prefix match
            // SurrealDB uses string::starts_with for prefix matching
            let prefix = pattern.trim_end_matches('*').trim_end_matches('.');
            if prefix.is_empty() || prefix == pattern {
                // No wildcard or exact match - use equality
                self.db
                    .query("SELECT meta::id(id) as id, entity_id, key, content, source, created_at, updated_at FROM entity_observation WHERE entity_id = $entity_id AND key = $pattern ORDER BY key, updated_at DESC")
                    .bind(("entity_id", entity_id.to_string()))
                    .bind(("pattern", pattern.to_string()))
                    .await?
            } else {
                // Prefix match (e.g., "architecture.*" matches keys starting with "architecture.")
                let prefix_with_dot = format!("{}.", prefix);
                self.db
                    .query("SELECT meta::id(id) as id, entity_id, key, content, source, created_at, updated_at FROM entity_observation WHERE entity_id = $entity_id AND (key = $prefix OR string::starts_with(key, $prefix_dot)) ORDER BY key, updated_at DESC")
                    .bind(("entity_id", entity_id.to_string()))
                    .bind(("prefix", prefix.to_string()))
                    .bind(("prefix_dot", prefix_with_dot))
                    .await?
            }
        } else {
            self.db
                .query("SELECT meta::id(id) as id, entity_id, key, content, source, created_at, updated_at FROM entity_observation WHERE entity_id = $entity_id ORDER BY key, updated_at DESC")
                .bind(("entity_id", entity_id.to_string()))
                .await?
        };

        let records: Vec<ObservationRecordWithId> = result.take(0)?;

        let mut observations = Vec::new();
        for record in records {
            observations.push(self.record_to_observation(record)?);
        }

        Ok(observations)
    }

    /// Search observations by content (case-insensitive).
    pub async fn search_observations(
        &self,
        entity_id: &Id,
        query: &str,
        limit: usize,
    ) -> StoreResult<Vec<Observation>> {
        debug!(
            "Searching observations for entity {} with query '{}'",
            entity_id, query
        );

        let mut result = self.db
            .query("SELECT meta::id(id) as id, entity_id, key, content, source, created_at, updated_at FROM entity_observation WHERE entity_id = $entity_id AND content IS NOT NONE AND string::lowercase(content) CONTAINS $query ORDER BY updated_at DESC LIMIT $limit")
            .bind(("entity_id", entity_id.to_string()))
            .bind(("query", query.to_lowercase()))
            .bind(("limit", limit as i64))
            .await?;

        let records: Vec<ObservationRecordWithId> = result.take(0)?;

        let mut observations = Vec::new();
        for record in records {
            observations.push(self.record_to_observation(record)?);
        }

        Ok(observations)
    }

    /// Get observation history for a key (archived versions).
    pub async fn get_observation_history(
        &self,
        entity_id: &Id,
        key: &str,
    ) -> StoreResult<Vec<ArchivedObservation>> {
        debug!(
            "Getting observation history for key '{}' on entity {}",
            key, entity_id
        );

        let mut result = self.db
            .query("SELECT observation_id, entity_id, key, content, source, created_at, archived_at FROM entity_observation_archive WHERE entity_id = $entity_id AND key = $key ORDER BY archived_at DESC")
            .bind(("entity_id", entity_id.to_string()))
            .bind(("key", key.to_string()))
            .await?;

        let records: Vec<ObservationArchiveRecord> = result.take(0)?;

        let mut history = Vec::new();
        for record in records {
            let observation_id = Id::parse(&record.observation_id).map_err(|e| {
                StoreError::Deserialization(format!("Invalid observation ID: {}", e))
            })?;
            let entity_id = Id::parse(&record.entity_id)
                .map_err(|e| StoreError::Deserialization(format!("Invalid entity ID: {}", e)))?;
            history.push(ArchivedObservation {
                observation_id,
                entity_id,
                key: record.key,
                content: record.content,
                source: record.source,
                created_at: record.created_at.to_offset_datetime()?,
                archived_at: record.archived_at.to_offset_datetime()?,
            });
        }

        Ok(history)
    }

    /// Delete an observation by ID.
    pub async fn delete_observation(&self, id: &Id) -> StoreResult<()> {
        debug!("Deleting observation: {}", id);

        self.db
            .query(r#"DELETE type::thing("entity_observation", $id)"#)
            .bind(("id", id.to_string()))
            .await?;

        Ok(())
    }

    /// Helper to convert observation record to domain type.
    fn record_to_observation(&self, record: ObservationRecordWithId) -> StoreResult<Observation> {
        let id = self.parse_record_id(&record.id)?;
        let entity_id = Id::parse(&record.entity_id)
            .map_err(|e| StoreError::Deserialization(format!("Invalid entity ID: {}", e)))?;
        Ok(Observation {
            id,
            entity_id,
            key: record.key,
            content: record.content,
            source: record.source,
            embedding: record.embedding,
            created_at: record.created_at.to_offset_datetime()?,
            updated_at: record.updated_at.to_offset_datetime()?,
        })
    }

    // =========================================================================
    // Unified Search Operations
    // =========================================================================

    /// Search entities by name OR description (extended search).
    pub async fn search_entities_extended(
        &self,
        query: &str,
        limit: usize,
    ) -> StoreResult<Vec<Entity>> {
        debug!("Searching entities (extended): {}", query);

        let mut result = self.db
            .query("SELECT meta::id(id) as id, name, entity_type, description, properties, created_at, updated_at FROM entity WHERE string::lowercase(name) CONTAINS $query OR (description IS NOT NONE AND string::lowercase(description) CONTAINS $query) ORDER BY name LIMIT $limit")
            .bind(("query", query.to_lowercase()))
            .bind(("limit", limit as i64))
            .await?;

        let records: Vec<EntityRecordWithId> = result.take(0)?;

        let mut entities = Vec::new();
        for record in records {
            let id = self.parse_record_id(&record.id)?;
            entities.push(self.record_to_entity(id, record.into())?);
        }

        Ok(entities)
    }

    /// Search all observations globally (not scoped to a specific entity).
    pub async fn search_observations_global(
        &self,
        query: &str,
        limit: usize,
    ) -> StoreResult<Vec<Observation>> {
        debug!("Searching all observations: {}", query);

        let mut result = self.db
            .query("SELECT meta::id(id) as id, entity_id, key, content, source, created_at, updated_at FROM entity_observation WHERE content IS NOT NONE AND string::lowercase(content) CONTAINS $query ORDER BY updated_at DESC LIMIT $limit")
            .bind(("query", query.to_lowercase()))
            .bind(("limit", limit as i64))
            .await?;

        let records: Vec<ObservationRecordWithId> = result.take(0)?;

        let mut observations = Vec::new();
        for record in records {
            observations.push(self.record_to_observation(record)?);
        }

        Ok(observations)
    }

    /// Search aliases by name.
    pub async fn search_aliases(
        &self,
        query: &str,
        limit: usize,
    ) -> StoreResult<Vec<AliasSearchResult>> {
        debug!("Searching aliases: {}", query);

        let mut result = self.db
            .query("SELECT name, entity_id FROM entity_alias WHERE string::lowercase(name) CONTAINS $query LIMIT $limit")
            .bind(("query", query.to_lowercase()))
            .bind(("limit", limit as i64))
            .await?;

        let records: Vec<AliasRecord> = result.take(0)?;

        let mut results = Vec::new();
        for record in records {
            let entity_id = Id::parse(&record.entity_id)
                .map_err(|e| StoreError::Deserialization(format!("Invalid entity ID: {}", e)))?;
            results.push(AliasSearchResult {
                alias_name: record.name,
                entity_id,
            });
        }

        Ok(results)
    }

    // =========================================================================
    // Vector Search Operations
    // =========================================================================

    /// Search entities by embedding similarity.
    pub async fn search_entities_by_embedding(
        &self,
        query_embedding: &[f32],
        limit: usize,
        min_score: f32,
    ) -> StoreResult<Vec<EntitySearchResult>> {
        debug!("Searching entities by embedding (limit={})", limit);

        // Get all entities with embeddings
        let mut result = self.db
            .query("SELECT meta::id(id) as id, name, entity_type, description, properties, embedding, created_at, updated_at FROM entity WHERE embedding IS NOT NONE")
            .await?;

        let records: Vec<EntityRecordWithId> = result.take(0)?;

        // Compute cosine similarity for each entity
        let mut scored_results: Vec<EntitySearchResult> = Vec::new();
        for record in records {
            if let Some(ref embedding) = record.embedding {
                let score = cosine_similarity(query_embedding, embedding);
                if score >= min_score {
                    let id = self.parse_record_id(&record.id)?;
                    let entity = self.record_to_entity(id, record.into())?;
                    scored_results.push(EntitySearchResult { entity, score });
                }
            }
        }

        // Sort by score descending and take top-k
        scored_results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored_results.truncate(limit);

        Ok(scored_results)
    }

    /// Search observations by embedding similarity.
    pub async fn search_observations_by_embedding(
        &self,
        query_embedding: &[f32],
        limit: usize,
        min_score: f32,
    ) -> StoreResult<Vec<ObservationSearchResult>> {
        debug!("Searching observations by embedding (limit={})", limit);

        // Get all observations with embeddings
        let mut result = self.db
            .query("SELECT meta::id(id) as id, entity_id, key, content, source, embedding, created_at, updated_at FROM entity_observation WHERE embedding IS NOT NONE")
            .await?;

        let records: Vec<ObservationRecordWithId> = result.take(0)?;

        // Compute cosine similarity for each observation
        let mut scored_results: Vec<ObservationSearchResult> = Vec::new();
        for record in records {
            if let Some(ref embedding) = record.embedding {
                let score = cosine_similarity(query_embedding, embedding);
                if score >= min_score {
                    let observation = self.record_to_observation(record)?;
                    scored_results.push(ObservationSearchResult { observation, score });
                }
            }
        }

        // Sort by score descending and take top-k
        scored_results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored_results.truncate(limit);

        Ok(scored_results)
    }

    // =========================================================================
    // Statistics
    // =========================================================================

    /// Get entity statistics.
    pub async fn stats(&self) -> StoreResult<EntityStats> {
        let mut result = self
            .db
            .query(format!(
                r#"
                SELECT count() as count FROM {TABLE_ENTITY} GROUP ALL;
                SELECT count() as count FROM {TABLE_RELATIONSHIP} GROUP ALL;
                SELECT count() as count FROM {TABLE_ALIAS} GROUP ALL;
                SELECT count() as count FROM {TABLE_OBSERVATION} GROUP ALL;
                "#
            ))
            .await?;

        let entity_count: Option<CountResult> = result.take(0)?;
        let relationship_count: Option<CountResult> = result.take(1)?;
        let alias_count: Option<CountResult> = result.take(2)?;
        let observation_count: Option<CountResult> = result.take(3)?;

        Ok(EntityStats {
            entity_count: entity_count.map(|c| c.count).unwrap_or(0),
            relationship_count: relationship_count.map(|c| c.count).unwrap_or(0),
            alias_count: alias_count.map(|c| c.count).unwrap_or(0),
            observation_count: observation_count.map(|c| c.count).unwrap_or(0),
        })
    }

    // =========================================================================
    // Helper Methods
    // =========================================================================

    fn parse_record_id(&self, record_id: &str) -> StoreResult<Id> {
        // Record ID can be "table:id" or just "id" (when using meta::id())
        let id_str = if record_id.contains(':') {
            // Format: "table:id" - take the part after colon
            record_id.split(':').nth(1).unwrap_or(record_id)
        } else {
            // Format: just "id" (from meta::id())
            record_id
        };

        Id::parse(id_str).map_err(|e| StoreError::Deserialization(format!("Invalid ID: {}", e)))
    }

    fn record_to_entity(&self, id: Id, record: EntityRecord) -> StoreResult<Entity> {
        Ok(Entity {
            id,
            name: record.name,
            entity_type: EntityType::parse(&record.entity_type),
            description: record.description,
            properties: record.properties,
            embedding: record.embedding,
            created_at: record.created_at.to_offset_datetime()?,
            updated_at: record.updated_at.to_offset_datetime()?,
        })
    }

    fn records_to_relationships(
        &self,
        records: Vec<RelationshipRecord>,
    ) -> StoreResult<Vec<Relationship>> {
        let mut relationships = Vec::new();
        for record in records {
            let source_id = Id::parse(&record.source_id)
                .map_err(|e| StoreError::Deserialization(format!("Invalid source ID: {}", e)))?;
            let target_id = Id::parse(&record.target_id)
                .map_err(|e| StoreError::Deserialization(format!("Invalid target ID: {}", e)))?;
            relationships.push(Relationship {
                source_id,
                target_id,
                relation_type: RelationType::parse(&record.relation_type),
                description: record.description,
                weight: record.weight,
                created_at: record.created_at.to_offset_datetime()?,
            });
        }
        Ok(relationships)
    }
}

/// Entity record with ID for list queries.
#[derive(Debug, Deserialize)]
struct EntityRecordWithId {
    id: String,
    name: String,
    entity_type: String,
    description: Option<String>,
    #[serde(default)]
    properties: HashMap<String, serde_json::Value>,
    #[serde(default)]
    embedding: Option<Vec<f32>>,
    created_at: SurrealDateTime,
    updated_at: SurrealDateTime,
}

impl From<EntityRecordWithId> for EntityRecord {
    fn from(r: EntityRecordWithId) -> Self {
        EntityRecord {
            name: r.name,
            entity_type: r.entity_type,
            description: r.description,
            properties: r.properties,
            embedding: r.embedding,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

/// Observation record with ID.
#[derive(Debug, Deserialize)]
struct ObservationRecordWithId {
    id: String,
    entity_id: String,
    key: Option<String>,
    content: String,
    source: Option<String>,
    #[serde(default)]
    embedding: Option<Vec<f32>>,
    created_at: SurrealDateTime,
    updated_at: SurrealDateTime,
}

/// Archived observation for history.
#[derive(Debug, Clone)]
pub struct ArchivedObservation {
    pub observation_id: Id,
    pub entity_id: Id,
    pub key: Option<String>,
    pub content: String,
    pub source: Option<String>,
    pub created_at: time::OffsetDateTime,
    pub archived_at: time::OffsetDateTime,
}

/// Count result for stats queries.
#[derive(Debug, Deserialize)]
struct CountResult {
    count: u64,
}

/// Entity statistics.
#[derive(Debug, Clone, Default)]
pub struct EntityStats {
    pub entity_count: u64,
    pub relationship_count: u64,
    pub alias_count: u64,
    pub observation_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_stats_default() {
        let stats = EntityStats::default();
        assert_eq!(stats.entity_count, 0);
        assert_eq!(stats.relationship_count, 0);
    }

    #[test]
    fn test_entity_type_from_str() {
        assert_eq!(EntityType::parse("repo"), EntityType::Repo);
        assert_eq!(EntityType::parse("TOOL"), EntityType::Tool);
        assert_eq!(
            EntityType::parse("custom_type"),
            EntityType::Custom("custom_type".to_string())
        );
    }

    #[test]
    fn test_relation_type_from_str() {
        assert_eq!(RelationType::parse("depends_on"), RelationType::DependsOn);
        assert_eq!(RelationType::parse("USES"), RelationType::Uses);
        assert_eq!(
            RelationType::parse("custom_rel"),
            RelationType::Custom("custom_rel".to_string())
        );
    }
}
