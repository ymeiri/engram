//! Entity service for Layer 1: Entity Knowledge.
//!
//! Provides business logic for managing the knowledge graph:
//! entities, relationships, aliases, and observations.

use crate::error::{IndexError, IndexResult};
use engram_core::entity::{Alias, Entity, EntityType, Observation, RelationType, Relationship};
use engram_core::id::Id;
use engram_embed::Embedder;
use engram_store::{
    ArchivedObservation, Db, EntityRepo, EntitySearchResult, EntityStats, ObservationSearchResult,
};
use tracing::{debug, info};

/// Service for entity knowledge management.
#[derive(Clone)]
pub struct EntityService {
    repo: EntityRepo,
    embedder: Option<Embedder>,
}

impl EntityService {
    /// Create a new entity service without embedding support.
    pub fn new(db: Db) -> Self {
        Self {
            repo: EntityRepo::new(db),
            embedder: None,
        }
    }

    /// Create a new entity service with embedding support for vector search.
    pub fn with_embedder(db: Db, embedder: Embedder) -> Self {
        Self {
            repo: EntityRepo::new(db),
            embedder: Some(embedder),
        }
    }

    /// Create a new entity service with the default embedder.
    pub fn with_defaults(db: Db) -> IndexResult<Self> {
        let embedder = Embedder::default_model()?;
        Ok(Self::with_embedder(db, embedder))
    }

    /// Generate embedding for text, returns None if no embedder configured.
    fn generate_embedding(&self, text: &str) -> Option<Vec<f32>> {
        self.embedder.as_ref().and_then(|e| match e.embed(text) {
            Ok(embedding) => Some(embedding),
            Err(err) => {
                debug!("Failed to generate embedding: {}", err);
                None
            }
        })
    }

    /// Initialize the entity schema.
    pub async fn init(&self) -> IndexResult<()> {
        self.repo.init_schema().await?;
        Ok(())
    }

    // =========================================================================
    // Entity CRUD
    // =========================================================================

    /// Create a new entity.
    pub async fn create_entity(
        &self,
        name: &str,
        entity_type: EntityType,
        description: Option<&str>,
    ) -> IndexResult<Entity> {
        info!("Creating entity: {} ({})", name, entity_type);

        // Check if entity with this name already exists
        if let Some(existing) = self.repo.get_entity_by_name(name).await? {
            return Err(IndexError::AlreadyExists(format!(
                "Entity with name '{}' already exists (id: {})",
                name, existing.id
            )));
        }

        let mut entity = Entity::new(name, entity_type);
        if let Some(desc) = description {
            entity = entity.with_description(desc);
        }

        // Generate embedding for name + description
        let embed_text = match &entity.description {
            Some(desc) => format!("{}: {}", name, desc),
            None => name.to_string(),
        };
        if let Some(embedding) = self.generate_embedding(&embed_text) {
            entity = entity.with_embedding(embedding);
            debug!("Generated embedding for entity '{}'", name);
        }

        self.repo.save_entity(&entity).await?;

        // Automatically add the entity name as an alias
        let alias = Alias::new(name, entity.id.clone());
        self.repo.add_alias(&alias).await?;

        info!("Created entity: {} ({})", entity.id, entity.name);
        Ok(entity)
    }

    /// Get an entity by ID.
    pub async fn get_entity(&self, id: &Id) -> IndexResult<Option<Entity>> {
        Ok(self.repo.get_entity(id).await?)
    }

    /// Get an entity by name.
    pub async fn get_entity_by_name(&self, name: &str) -> IndexResult<Option<Entity>> {
        Ok(self.repo.get_entity_by_name(name).await?)
    }

    /// Update an entity.
    pub async fn update_entity(&self, entity: &Entity) -> IndexResult<()> {
        self.repo.save_entity(entity).await?;
        Ok(())
    }

    /// Delete an entity (and all its relationships, aliases, observations).
    pub async fn delete_entity(&self, id: &Id) -> IndexResult<()> {
        info!("Deleting entity: {}", id);
        self.repo.delete_entity(id).await?;
        Ok(())
    }

    /// List entities, optionally filtered by type.
    pub async fn list_entities(
        &self,
        entity_type: Option<&EntityType>,
    ) -> IndexResult<Vec<Entity>> {
        Ok(self.repo.list_entities(entity_type).await?)
    }

    /// Search entities by name.
    pub async fn search_entities(&self, query: &str) -> IndexResult<Vec<Entity>> {
        Ok(self.repo.search_entities(query).await?)
    }

    /// Resolve an alias or name to an entity.
    pub async fn resolve(&self, name_or_alias: &str) -> IndexResult<Option<Entity>> {
        // First try to resolve as alias
        if let Some(entity_id) = self.repo.resolve_alias(name_or_alias).await? {
            return Ok(self.repo.get_entity(&entity_id).await?);
        }

        // Fall back to direct name lookup
        Ok(self.repo.get_entity_by_name(name_or_alias).await?)
    }

    // =========================================================================
    // Relationship Management
    // =========================================================================

    /// Create a relationship between two entities.
    pub async fn relate(
        &self,
        source_name: &str,
        relation_type: RelationType,
        target_name: &str,
    ) -> IndexResult<Relationship> {
        info!(
            "Creating relationship: {} --[{}]--> {}",
            source_name, relation_type, target_name
        );

        // Resolve source and target
        let source = self
            .resolve(source_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Entity not found: {}", source_name)))?;

        let target = self
            .resolve(target_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Entity not found: {}", target_name)))?;

        let rel = Relationship::new(source.id, target.id, relation_type);
        self.repo.create_relationship(&rel).await?;

        Ok(rel)
    }

    /// Get entities that this entity depends on / uses / etc.
    pub async fn get_related_from(
        &self,
        entity_id: &Id,
    ) -> IndexResult<Vec<(Relationship, Entity)>> {
        let relationships = self.repo.get_relationships_from(entity_id).await?;
        let mut results = Vec::new();

        for rel in relationships {
            if let Some(entity) = self.repo.get_entity(&rel.target_id).await? {
                results.push((rel, entity));
            }
        }

        Ok(results)
    }

    /// Get entities that depend on / use / etc. this entity.
    pub async fn get_related_to(&self, entity_id: &Id) -> IndexResult<Vec<(Relationship, Entity)>> {
        let relationships = self.repo.get_relationships_to(entity_id).await?;
        let mut results = Vec::new();

        for rel in relationships {
            if let Some(entity) = self.repo.get_entity(&rel.source_id).await? {
                results.push((rel, entity));
            }
        }

        Ok(results)
    }

    /// Delete a relationship.
    pub async fn unrelate(
        &self,
        source_name: &str,
        relation_type: RelationType,
        target_name: &str,
    ) -> IndexResult<()> {
        let source = self
            .resolve(source_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Entity not found: {}", source_name)))?;

        let target = self
            .resolve(target_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Entity not found: {}", target_name)))?;

        self.repo
            .delete_relationship(&source.id, &target.id, &relation_type)
            .await?;

        Ok(())
    }

    // =========================================================================
    // Alias Management
    // =========================================================================

    /// Add an alias for an entity.
    pub async fn add_alias(&self, entity_name: &str, alias_name: &str) -> IndexResult<()> {
        info!("Adding alias '{}' for entity '{}'", alias_name, entity_name);

        let entity = self
            .resolve(entity_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Entity not found: {}", entity_name)))?;

        // Check if alias already exists
        if let Some(existing_id) = self.repo.resolve_alias(alias_name).await? {
            if existing_id == entity.id {
                return Ok(()); // Already aliased to this entity
            }
            return Err(IndexError::AlreadyExists(format!(
                "Alias '{}' already exists for another entity",
                alias_name
            )));
        }

        let alias = Alias::new(alias_name, entity.id);
        self.repo.add_alias(&alias).await?;

        Ok(())
    }

    /// Get all aliases for an entity.
    pub async fn get_aliases(&self, entity_name: &str) -> IndexResult<Vec<String>> {
        let entity = self
            .resolve(entity_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Entity not found: {}", entity_name)))?;

        Ok(self.repo.get_aliases(&entity.id).await?)
    }

    /// Remove an alias.
    pub async fn remove_alias(&self, entity_name: &str, alias_name: &str) -> IndexResult<()> {
        let entity = self
            .resolve(entity_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Entity not found: {}", entity_name)))?;

        self.repo.delete_alias(alias_name, &entity.id).await?;
        Ok(())
    }

    // =========================================================================
    // Observation Management
    // =========================================================================

    /// Add or update an observation (fact/note) about an entity.
    ///
    /// If `key` is provided and an observation with that key already exists,
    /// the existing observation is archived and updated (upsert behavior).
    ///
    /// Returns the observation and optionally the previous content if updated.
    pub async fn add_observation(
        &self,
        entity_name: &str,
        content: &str,
        key: Option<&str>,
        source: Option<&str>,
    ) -> IndexResult<(Observation, Option<Observation>)> {
        info!(
            "Adding/updating observation for entity '{}' (key: {:?})",
            entity_name, key
        );

        let entity = self
            .resolve(entity_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Entity not found: {}", entity_name)))?;

        let mut obs = Observation::new(entity.id.clone(), content);
        if let Some(k) = key {
            obs = obs.with_key(k);
        }
        if let Some(src) = source {
            obs = obs.with_source(src);
        }

        // Generate embedding for observation content
        // Include entity name and key for better context
        let embed_text = match (&key, &entity.name) {
            (Some(k), name) => format!("{} [{}]: {}", name, k, content),
            (None, name) => format!("{}: {}", name, content),
        };
        if let Some(embedding) = self.generate_embedding(&embed_text) {
            obs = obs.with_embedding(embedding);
            debug!("Generated embedding for observation on '{}'", entity_name);
        }

        let previous = self.repo.add_observation(&obs).await?;

        // If we updated an existing observation, get the current state
        if previous.is_some() {
            if let Some(k) = key {
                if let Some(current) = self.repo.get_observation_by_key(&obs.entity_id, k).await? {
                    return Ok((current, previous));
                }
            }
        }

        Ok((obs, previous))
    }

    /// Get an observation by its semantic key.
    pub async fn get_observation_by_key(
        &self,
        entity_name: &str,
        key: &str,
    ) -> IndexResult<Option<Observation>> {
        let entity = self
            .resolve(entity_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Entity not found: {}", entity_name)))?;

        Ok(self.repo.get_observation_by_key(&entity.id, key).await?)
    }

    /// Get observations about an entity.
    pub async fn get_observations(&self, entity_name: &str) -> IndexResult<Vec<Observation>> {
        let entity = self
            .resolve(entity_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Entity not found: {}", entity_name)))?;

        Ok(self.repo.get_observations(&entity.id).await?)
    }

    /// List observations with optional key pattern filtering.
    ///
    /// Pattern uses glob-style syntax: `*` for wildcard (e.g., "architecture.*").
    pub async fn list_observations_by_pattern(
        &self,
        entity_name: &str,
        key_pattern: Option<&str>,
    ) -> IndexResult<Vec<Observation>> {
        let entity = self
            .resolve(entity_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Entity not found: {}", entity_name)))?;

        Ok(self
            .repo
            .list_observations_by_pattern(&entity.id, key_pattern)
            .await?)
    }

    /// Search observations by content.
    pub async fn search_observations(
        &self,
        entity_name: &str,
        query: &str,
        limit: usize,
    ) -> IndexResult<Vec<Observation>> {
        let entity = self
            .resolve(entity_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Entity not found: {}", entity_name)))?;

        Ok(self
            .repo
            .search_observations(&entity.id, query, limit)
            .await?)
    }

    /// Get the update history for an observation key.
    pub async fn get_observation_history(
        &self,
        entity_name: &str,
        key: &str,
    ) -> IndexResult<Vec<ArchivedObservation>> {
        let entity = self
            .resolve(entity_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Entity not found: {}", entity_name)))?;

        Ok(self.repo.get_observation_history(&entity.id, key).await?)
    }

    // =========================================================================
    // Vector Search
    // =========================================================================

    /// Search entities using vector similarity.
    ///
    /// Returns entities ordered by semantic similarity to the query.
    /// Requires embedder to be configured.
    pub async fn search_entities_semantic(
        &self,
        query: &str,
        limit: usize,
        min_score: f32,
    ) -> IndexResult<Vec<EntitySearchResult>> {
        let Some(ref embedder) = self.embedder else {
            return Err(IndexError::NotConfigured(
                "Embedder not configured for semantic search".into(),
            ));
        };

        let query_embedding = embedder.embed(query)?;
        Ok(self
            .repo
            .search_entities_by_embedding(&query_embedding, limit, min_score)
            .await?)
    }

    /// Search observations using vector similarity.
    ///
    /// Returns observations ordered by semantic similarity to the query.
    /// Requires embedder to be configured.
    pub async fn search_observations_semantic(
        &self,
        query: &str,
        limit: usize,
        min_score: f32,
    ) -> IndexResult<Vec<ObservationSearchResult>> {
        let Some(ref embedder) = self.embedder else {
            return Err(IndexError::NotConfigured(
                "Embedder not configured for semantic search".into(),
            ));
        };

        let query_embedding = embedder.embed(query)?;
        Ok(self
            .repo
            .search_observations_by_embedding(&query_embedding, limit, min_score)
            .await?)
    }

    // =========================================================================
    // Statistics
    // =========================================================================

    /// Get entity statistics.
    pub async fn stats(&self) -> IndexResult<EntityStats> {
        Ok(self.repo.stats().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_type_display() {
        assert_eq!(EntityType::Repo.to_string(), "repo");
        assert_eq!(EntityType::Service.to_string(), "service");
    }

    #[test]
    fn test_relation_type_display() {
        assert_eq!(RelationType::DependsOn.to_string(), "depends_on");
        assert_eq!(RelationType::Uses.to_string(), "uses");
    }
}
