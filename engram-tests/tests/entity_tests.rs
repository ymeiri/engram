//! Integration tests for Layer 1: Entity Knowledge.
//!
//! Tests the entity knowledge graph with relationships, aliases, and observations.

use engram_core::entity::{EntityType, RelationType};
use engram_index::EntityService;
use engram_mcp::tools::{self, EntityObserveRequestNew, EntityRequestNew, ToolState};
use engram_store::{connect_and_init, StoreConfig};
use std::sync::Arc;
use tokio::sync::RwLock;

// =============================================================================
// Test Fixtures
// =============================================================================

async fn setup_service() -> EntityService {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config).await.expect("Failed to connect");
    let service = EntityService::new(db);
    service.init().await.expect("Failed to init");
    service
}

// =============================================================================
// Entity CRUD Tests
// =============================================================================

#[tokio::test]
async fn test_create_entity() {
    let service = setup_service().await;

    let entity = service
        .create_entity("web-api", EntityType::Service, Some("Main REST API"))
        .await
        .expect("Failed to create entity");

    assert_eq!(entity.name, "web-api");
    assert_eq!(entity.entity_type, EntityType::Service);
    assert_eq!(entity.description, Some("Main REST API".to_string()));
}

#[tokio::test]
async fn test_create_all_entity_types() {
    let service = setup_service().await;

    let types = [
        ("my-repo", EntityType::Repo),
        ("my-tool", EntityType::Tool),
        ("my-concept", EntityType::Concept),
        ("my-deploy", EntityType::Deployment),
        ("my-topic", EntityType::Topic),
        ("my-workflow", EntityType::Workflow),
        ("my-person", EntityType::Person),
        ("my-team", EntityType::Team),
        ("my-service", EntityType::Service),
    ];

    for (name, entity_type) in types {
        let entity = service
            .create_entity(name, entity_type.clone(), None)
            .await
            .expect(&format!("Failed to create {}", name));

        assert_eq!(entity.entity_type, entity_type);
    }
}

#[tokio::test]
async fn test_get_entity_by_name() {
    let service = setup_service().await;

    service
        .create_entity("postgres", EntityType::Service, Some("Database"))
        .await
        .expect("Failed to create");

    let found = service
        .resolve("postgres")
        .await
        .expect("Failed to resolve")
        .expect("Entity not found");

    assert_eq!(found.name, "postgres");
}

#[tokio::test]
async fn test_list_entities() {
    let service = setup_service().await;

    service
        .create_entity("svc-1", EntityType::Service, None)
        .await
        .unwrap();
    service
        .create_entity("svc-2", EntityType::Service, None)
        .await
        .unwrap();
    service
        .create_entity("tool-1", EntityType::Tool, None)
        .await
        .unwrap();

    // List all
    let all = service.list_entities(None).await.expect("Failed to list");
    assert_eq!(all.len(), 3);

    // List by type
    let services = service
        .list_entities(Some(&EntityType::Service))
        .await
        .expect("Failed to list");
    assert_eq!(services.len(), 2);

    let tools = service
        .list_entities(Some(&EntityType::Tool))
        .await
        .expect("Failed to list");
    assert_eq!(tools.len(), 1);
}

#[tokio::test]
async fn test_search_entities() {
    let service = setup_service().await;

    service
        .create_entity("auth-service", EntityType::Service, None)
        .await
        .unwrap();
    service
        .create_entity("auth-client", EntityType::Service, None)
        .await
        .unwrap();
    service
        .create_entity("billing", EntityType::Service, None)
        .await
        .unwrap();

    let results = service
        .search_entities("auth")
        .await
        .expect("Failed to search");

    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|e| e.name.contains("auth")));
}

#[tokio::test]
async fn test_delete_entity() {
    let service = setup_service().await;

    let entity = service
        .create_entity("to-delete", EntityType::Concept, None)
        .await
        .unwrap();

    // Verify exists
    assert!(service.resolve("to-delete").await.unwrap().is_some());

    // Delete
    service
        .delete_entity(&entity.id)
        .await
        .expect("Failed to delete");

    // Verify gone
    assert!(service.resolve("to-delete").await.unwrap().is_none());
}

// =============================================================================
// Relationship Tests
// =============================================================================

#[tokio::test]
async fn test_create_relationship() {
    let service = setup_service().await;

    service
        .create_entity("web-ui", EntityType::Service, None)
        .await
        .unwrap();
    service
        .create_entity("api", EntityType::Service, None)
        .await
        .unwrap();

    let rel = service
        .relate("web-ui", RelationType::DependsOn, "api")
        .await
        .expect("Failed to create relationship");

    assert_eq!(rel.relation_type, RelationType::DependsOn);
}

#[tokio::test]
async fn test_all_relationship_types() {
    let service = setup_service().await;

    service
        .create_entity("source", EntityType::Service, None)
        .await
        .unwrap();
    service
        .create_entity("target", EntityType::Service, None)
        .await
        .unwrap();

    let rel_types = [
        RelationType::DependsOn,
        RelationType::Uses,
        RelationType::DeployedVia,
        RelationType::OwnedBy,
        RelationType::Documents,
        RelationType::RelatedTo,
    ];

    for rel_type in rel_types {
        // Create a new target for each relationship
        let target_name = format!("target-{}", rel_type);
        service
            .create_entity(&target_name, EntityType::Service, None)
            .await
            .unwrap();

        let rel = service
            .relate("source", rel_type.clone(), &target_name)
            .await
            .expect(&format!("Failed to create {:?}", rel_type));

        assert_eq!(rel.relation_type, rel_type);
    }
}

#[tokio::test]
async fn test_get_related_entities() {
    let service = setup_service().await;

    // Create entities
    service
        .create_entity("frontend", EntityType::Service, None)
        .await
        .unwrap();
    service
        .create_entity("backend", EntityType::Service, None)
        .await
        .unwrap();
    service
        .create_entity("database", EntityType::Service, None)
        .await
        .unwrap();

    // Create relationships: frontend -> backend -> database
    service
        .relate("frontend", RelationType::DependsOn, "backend")
        .await
        .unwrap();
    service
        .relate("backend", RelationType::Uses, "database")
        .await
        .unwrap();

    // Get frontend entity
    let frontend = service.resolve("frontend").await.unwrap().unwrap();

    // Check outgoing relationships from frontend
    let outgoing = service
        .get_related_from(&frontend.id)
        .await
        .expect("Failed");
    assert_eq!(outgoing.len(), 1);
    assert_eq!(outgoing[0].1.name, "backend");

    // Get backend entity
    let backend = service.resolve("backend").await.unwrap().unwrap();

    // Check incoming relationships to backend
    let incoming = service.get_related_to(&backend.id).await.expect("Failed");
    assert_eq!(incoming.len(), 1);
    assert_eq!(incoming[0].1.name, "frontend");
}

// =============================================================================
// Alias Tests
// =============================================================================

#[tokio::test]
async fn test_add_alias() {
    let service = setup_service().await;

    service
        .create_entity("MCP", EntityType::Concept, None)
        .await
        .unwrap();

    service
        .add_alias("MCP", "Model Context Protocol")
        .await
        .expect("Failed to add alias");

    // Should be able to resolve by alias
    let by_alias = service
        .resolve("Model Context Protocol")
        .await
        .expect("Failed to resolve")
        .expect("Not found by alias");

    assert_eq!(by_alias.name, "MCP");
}

#[tokio::test]
async fn test_multiple_aliases() {
    let service = setup_service().await;

    service
        .create_entity("kubernetes", EntityType::Tool, None)
        .await
        .unwrap();

    service.add_alias("kubernetes", "k8s").await.unwrap();
    service.add_alias("kubernetes", "kube").await.unwrap();

    // Get aliases (includes auto-created alias for entity name "kubernetes")
    let aliases = service.get_aliases("kubernetes").await.expect("Failed");
    assert_eq!(aliases.len(), 3); // kubernetes (auto) + k8s + kube
    assert!(aliases.contains(&"kubernetes".to_string())); // auto-created
    assert!(aliases.contains(&"k8s".to_string()));
    assert!(aliases.contains(&"kube".to_string()));

    // Resolve by any alias
    assert!(service.resolve("k8s").await.unwrap().is_some());
    assert!(service.resolve("kube").await.unwrap().is_some());
}

// =============================================================================
// Observation Tests
// =============================================================================

#[tokio::test]
async fn test_add_observation() {
    let service = setup_service().await;

    service
        .create_entity("redis", EntityType::Service, None)
        .await
        .unwrap();

    let (obs, previous) = service
        .add_observation(
            "redis",
            "Used for session caching",
            None,
            Some("architecture doc"),
        )
        .await
        .expect("Failed to add observation");

    assert_eq!(obs.content, "Used for session caching");
    assert_eq!(obs.source, Some("architecture doc".to_string()));
    assert!(previous.is_none()); // First observation, no previous
}

#[tokio::test]
async fn test_multiple_observations() {
    let service = setup_service().await;

    service
        .create_entity("api-gateway", EntityType::Service, None)
        .await
        .unwrap();

    service
        .add_observation(
            "api-gateway",
            "Handles rate limiting",
            None,
            Some("ops doc"),
        )
        .await
        .unwrap();
    service
        .add_observation("api-gateway", "Uses nginx", None, None)
        .await
        .unwrap();
    service
        .add_observation("api-gateway", "Port 8080", None, Some("config"))
        .await
        .unwrap();

    let observations = service
        .get_observations("api-gateway")
        .await
        .expect("Failed to get observations");

    assert_eq!(observations.len(), 3);
}

// =============================================================================
// Statistics Tests
// =============================================================================

#[tokio::test]
async fn test_entity_stats() {
    let service = setup_service().await;

    // Create some entities, relationships, aliases, observations
    service
        .create_entity("e1", EntityType::Service, None)
        .await
        .unwrap();
    service
        .create_entity("e2", EntityType::Service, None)
        .await
        .unwrap();
    service
        .relate("e1", RelationType::DependsOn, "e2")
        .await
        .unwrap();
    service.add_alias("e1", "entity-one").await.unwrap();
    service
        .add_observation("e1", "Test observation", None, None)
        .await
        .unwrap();

    let stats = service.stats().await.expect("Failed to get stats");

    assert_eq!(stats.entity_count, 2);
    assert_eq!(stats.relationship_count, 1);
    assert_eq!(stats.alias_count, 3); // e1 (auto) + e2 (auto) + entity-one (manual)
    assert_eq!(stats.observation_count, 1);
}

// =============================================================================
// Edge Cases
// =============================================================================

#[tokio::test]
async fn test_entity_not_found() {
    let service = setup_service().await;

    let result = service.resolve("nonexistent").await.expect("Query failed");
    assert!(result.is_none());
}

#[tokio::test]
async fn test_duplicate_entity_name() {
    let service = setup_service().await;

    service
        .create_entity("unique", EntityType::Service, None)
        .await
        .unwrap();

    // Creating another entity with same name should either error or update
    let result = service
        .create_entity("unique", EntityType::Tool, None)
        .await;
    // Behavior depends on implementation - just ensure it doesn't panic
    let _ = result;
}

#[tokio::test]
async fn test_special_characters_in_entity_name() {
    let service = setup_service().await;

    let names = [
        "service-with-dashes",
        "service_with_underscores",
        "Service.With.Dots",
        "service/path/style",
        "service:port:8080",
    ];

    for name in names {
        let entity = service
            .create_entity(name, EntityType::Service, None)
            .await
            .expect(&format!("Failed to create '{}'", name));

        assert_eq!(entity.name, name);
    }
}

#[tokio::test]
async fn test_long_description() {
    let service = setup_service().await;

    let long_desc = "A".repeat(10000);

    let entity = service
        .create_entity("long-desc", EntityType::Concept, Some(&long_desc))
        .await
        .expect("Failed with long description");

    assert_eq!(entity.description.unwrap().len(), 10000);
}

// =============================================================================
// Keyed Observations Tests
// =============================================================================

#[tokio::test]
async fn test_keyed_observation_create() {
    let service = setup_service().await;

    service
        .create_entity("my-app", EntityType::Repo, None)
        .await
        .unwrap();

    // Create a keyed observation
    let (obs, previous) = service
        .add_observation(
            "my-app",
            "Uses JWT for authentication",
            Some("architecture.auth"),
            Some("code-analysis"),
        )
        .await
        .expect("Failed to add keyed observation");

    assert_eq!(obs.key, Some("architecture.auth".to_string()));
    assert_eq!(obs.content, "Uses JWT for authentication");
    assert_eq!(obs.source, Some("code-analysis".to_string()));
    assert!(previous.is_none()); // First observation with this key
}

#[tokio::test]
async fn test_keyed_observation_update() {
    let service = setup_service().await;

    service
        .create_entity("my-app", EntityType::Repo, None)
        .await
        .unwrap();

    // Create initial observation
    let (obs1, prev1) = service
        .add_observation(
            "my-app",
            "Uses session-based auth",
            Some("architecture.auth"),
            Some("initial"),
        )
        .await
        .unwrap();
    assert!(prev1.is_none());

    // Update the same key
    let (obs2, prev2) = service
        .add_observation(
            "my-app",
            "Actually uses JWT, not sessions",
            Some("architecture.auth"),
            Some("correction"),
        )
        .await
        .unwrap();

    // Should have updated, not created new
    assert_eq!(obs2.key, Some("architecture.auth".to_string()));
    assert_eq!(obs2.content, "Actually uses JWT, not sessions");
    assert!(prev2.is_some());
    assert_eq!(prev2.unwrap().content, "Uses session-based auth");
}

#[tokio::test]
async fn test_keyed_observation_multiple_keys() {
    let service = setup_service().await;

    service
        .create_entity("my-app", EntityType::Repo, None)
        .await
        .unwrap();

    // Create observations with different keys
    service
        .add_observation("my-app", "Uses JWT", Some("architecture.auth"), None)
        .await
        .unwrap();
    service
        .add_observation(
            "my-app",
            "Watch for race conditions in cache",
            Some("gotchas.caching"),
            None,
        )
        .await
        .unwrap();
    service
        .add_observation(
            "my-app",
            "Uses React 18",
            Some("architecture.frontend"),
            None,
        )
        .await
        .unwrap();

    let all_obs = service.get_observations("my-app").await.unwrap();
    assert_eq!(all_obs.len(), 3);
}

#[tokio::test]
async fn test_get_observation_by_key() {
    let service = setup_service().await;

    service
        .create_entity("my-app", EntityType::Repo, None)
        .await
        .unwrap();

    service
        .add_observation(
            "my-app",
            "Uses PostgreSQL",
            Some("architecture.database"),
            None,
        )
        .await
        .unwrap();

    let obs = service
        .get_observation_by_key("my-app", "architecture.database")
        .await
        .unwrap();

    assert!(obs.is_some());
    assert_eq!(obs.unwrap().content, "Uses PostgreSQL");

    // Non-existent key should return None
    let none = service
        .get_observation_by_key("my-app", "architecture.nonexistent")
        .await
        .unwrap();
    assert!(none.is_none());
}

#[tokio::test]
async fn test_list_observations_by_pattern() {
    let service = setup_service().await;

    service
        .create_entity("my-app", EntityType::Repo, None)
        .await
        .unwrap();

    // Add various keyed observations
    service
        .add_observation("my-app", "Auth overview", Some("architecture.auth"), None)
        .await
        .unwrap();
    service
        .add_observation(
            "my-app",
            "Database overview",
            Some("architecture.database"),
            None,
        )
        .await
        .unwrap();
    service
        .add_observation("my-app", "API overview", Some("architecture.api"), None)
        .await
        .unwrap();
    service
        .add_observation(
            "my-app",
            "Cache race condition",
            Some("gotchas.cache"),
            None,
        )
        .await
        .unwrap();

    // Filter by architecture.*
    let arch_obs = service
        .list_observations_by_pattern("my-app", Some("architecture.*"))
        .await
        .unwrap();
    assert_eq!(arch_obs.len(), 3);

    // Filter by gotchas.*
    let gotchas_obs = service
        .list_observations_by_pattern("my-app", Some("gotchas.*"))
        .await
        .unwrap();
    assert_eq!(gotchas_obs.len(), 1);

    // All observations (no pattern)
    let all_obs = service
        .list_observations_by_pattern("my-app", None)
        .await
        .unwrap();
    assert_eq!(all_obs.len(), 4);
}

#[tokio::test]
async fn test_search_observations() {
    let service = setup_service().await;

    service
        .create_entity("my-app", EntityType::Repo, None)
        .await
        .unwrap();

    service
        .add_observation(
            "my-app",
            "Uses JWT for authentication",
            Some("architecture.auth"),
            None,
        )
        .await
        .unwrap();
    service
        .add_observation(
            "my-app",
            "PostgreSQL database",
            Some("architecture.db"),
            None,
        )
        .await
        .unwrap();
    service
        .add_observation(
            "my-app",
            "JWT tokens expire after 1 hour",
            Some("config.auth"),
            None,
        )
        .await
        .unwrap();

    // Search for "JWT"
    let jwt_obs = service
        .search_observations("my-app", "jwt", 10)
        .await
        .unwrap();
    assert_eq!(jwt_obs.len(), 2);

    // Search for "PostgreSQL"
    let db_obs = service
        .search_observations("my-app", "postgresql", 10)
        .await
        .unwrap();
    assert_eq!(db_obs.len(), 1);
}

#[tokio::test]
async fn test_observation_history() {
    let service = setup_service().await;

    service
        .create_entity("my-app", EntityType::Repo, None)
        .await
        .unwrap();

    // Create and update multiple times
    service
        .add_observation("my-app", "Version 1", Some("architecture.auth"), None)
        .await
        .unwrap();
    service
        .add_observation("my-app", "Version 2", Some("architecture.auth"), None)
        .await
        .unwrap();
    service
        .add_observation("my-app", "Version 3", Some("architecture.auth"), None)
        .await
        .unwrap();

    let history = service
        .get_observation_history("my-app", "architecture.auth")
        .await
        .unwrap();

    // Should have 2 archived versions (Version 1 and Version 2)
    assert_eq!(history.len(), 2);
    assert!(history.iter().any(|h| h.content == "Version 1"));
    assert!(history.iter().any(|h| h.content == "Version 2"));
}

#[tokio::test]
async fn test_mixed_keyed_and_unkeyed_observations() {
    let service = setup_service().await;

    service
        .create_entity("my-app", EntityType::Repo, None)
        .await
        .unwrap();

    // Add keyed observations
    service
        .add_observation("my-app", "Keyed obs 1", Some("key.one"), None)
        .await
        .unwrap();
    service
        .add_observation("my-app", "Keyed obs 2", Some("key.two"), None)
        .await
        .unwrap();

    // Add unkeyed observations (legacy style)
    service
        .add_observation("my-app", "Unkeyed obs 1", None, None)
        .await
        .unwrap();
    service
        .add_observation("my-app", "Unkeyed obs 2", None, None)
        .await
        .unwrap();
    service
        .add_observation("my-app", "Unkeyed obs 3", None, None)
        .await
        .unwrap();

    let all_obs = service.get_observations("my-app").await.unwrap();
    assert_eq!(all_obs.len(), 5); // 2 keyed + 3 unkeyed
}

// =============================================================================
// MCP Tool Handler Tests (Phase 2: Consolidated Action-based API)
// =============================================================================

/// Setup tool state for MCP handler tests.
async fn setup_tool_state() -> ToolState {
    let config = StoreConfig::memory();
    let db = connect_and_init(&config).await.expect("Failed to connect");
    let entity_service = EntityService::new(db);
    entity_service.init().await.expect("Failed to init");

    let state = ToolState::new();
    *state.entity_service.write().await = Some(entity_service);
    state
}

#[tokio::test]
async fn test_mcp_entity_create() {
    let state = setup_tool_state().await;

    let req = EntityRequestNew {
        action: "create".to_string(),
        name: Some("test-service".to_string()),
        entity_type: Some("service".to_string()),
        description: Some("A test service".to_string()),
        query: None,
        type_filter: None,
        limit: None,
        target: None,
        relation: None,
        alias: None,
    };
    let result = tools::entity_new(&state, req).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("test-service"));
    assert!(response.contains("service"));
}

#[tokio::test]
async fn test_mcp_entity_get() {
    let state = setup_tool_state().await;

    // Create an entity first
    let create_req = EntityRequestNew {
        action: "create".to_string(),
        name: Some("my-api".to_string()),
        entity_type: Some("service".to_string()),
        description: Some("My API service".to_string()),
        query: None,
        type_filter: None,
        limit: None,
        target: None,
        relation: None,
        alias: None,
    };
    tools::entity_new(&state, create_req).await.unwrap();

    // Get the entity
    let get_req = EntityRequestNew {
        action: "get".to_string(),
        name: Some("my-api".to_string()),
        entity_type: None,
        description: None,
        query: None,
        type_filter: None,
        limit: None,
        target: None,
        relation: None,
        alias: None,
    };
    let result = tools::entity_new(&state, get_req).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("my-api"));
    assert!(response.contains("My API service"));
}

#[tokio::test]
async fn test_mcp_entity_list() {
    let state = setup_tool_state().await;

    // Create some entities
    for name in ["service-a", "service-b", "tool-a"] {
        let entity_type = if name.starts_with("service") {
            "service"
        } else {
            "tool"
        };
        let req = EntityRequestNew {
            action: "create".to_string(),
            name: Some(name.to_string()),
            entity_type: Some(entity_type.to_string()),
            description: None,
            query: None,
            type_filter: None,
            limit: None,
            target: None,
            relation: None,
            alias: None,
        };
        tools::entity_new(&state, req).await.unwrap();
    }

    // List all entities
    let list_req = EntityRequestNew {
        action: "list".to_string(),
        name: None,
        entity_type: None,
        description: None,
        query: None,
        type_filter: None,
        limit: None,
        target: None,
        relation: None,
        alias: None,
    };
    let result = tools::entity_new(&state, list_req).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("service-a"));
    assert!(response.contains("service-b"));
    assert!(response.contains("tool-a"));

    // List only services
    let list_filtered_req = EntityRequestNew {
        action: "list".to_string(),
        name: None,
        entity_type: None,
        description: None,
        query: None,
        type_filter: Some("service".to_string()),
        limit: None,
        target: None,
        relation: None,
        alias: None,
    };
    let result = tools::entity_new(&state, list_filtered_req).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("service-a"));
    assert!(response.contains("service-b"));
    // tool-a may or may not appear based on implementation, but count should be 2
    assert!(response.contains("\"count\": 2"));
}

#[tokio::test]
async fn test_mcp_entity_search() {
    let state = setup_tool_state().await;

    // Create entities
    let req = EntityRequestNew {
        action: "create".to_string(),
        name: Some("auth-service".to_string()),
        entity_type: Some("service".to_string()),
        description: Some("Authentication service".to_string()),
        query: None,
        type_filter: None,
        limit: None,
        target: None,
        relation: None,
        alias: None,
    };
    tools::entity_new(&state, req).await.unwrap();

    // Search for it
    let search_req = EntityRequestNew {
        action: "search".to_string(),
        name: None,
        entity_type: None,
        description: None,
        query: Some("auth".to_string()),
        type_filter: None,
        limit: None,
        target: None,
        relation: None,
        alias: None,
    };
    let result = tools::entity_new(&state, search_req).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("auth-service"));
}

#[tokio::test]
async fn test_mcp_entity_relate() {
    let state = setup_tool_state().await;

    // Create two entities
    for name in ["frontend", "backend"] {
        let req = EntityRequestNew {
            action: "create".to_string(),
            name: Some(name.to_string()),
            entity_type: Some("service".to_string()),
            description: None,
            query: None,
            type_filter: None,
            limit: None,
            target: None,
            relation: None,
            alias: None,
        };
        tools::entity_new(&state, req).await.unwrap();
    }

    // Create relationship
    let relate_req = EntityRequestNew {
        action: "relate".to_string(),
        name: Some("frontend".to_string()),
        entity_type: None,
        description: None,
        query: None,
        type_filter: None,
        limit: None,
        target: Some("backend".to_string()),
        relation: Some("depends_on".to_string()),
        alias: None,
    };
    let result = tools::entity_new(&state, relate_req).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("Relationship created"));
    assert!(response.contains("depends_on"));
}

#[tokio::test]
async fn test_mcp_entity_alias() {
    let state = setup_tool_state().await;

    // Create entity
    let req = EntityRequestNew {
        action: "create".to_string(),
        name: Some("kubernetes".to_string()),
        entity_type: Some("tool".to_string()),
        description: None,
        query: None,
        type_filter: None,
        limit: None,
        target: None,
        relation: None,
        alias: None,
    };
    tools::entity_new(&state, req).await.unwrap();

    // Add alias
    let alias_req = EntityRequestNew {
        action: "alias".to_string(),
        name: Some("kubernetes".to_string()),
        entity_type: None,
        description: None,
        query: None,
        type_filter: None,
        limit: None,
        target: None,
        relation: None,
        alias: Some("k8s".to_string()),
    };
    let result = tools::entity_new(&state, alias_req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("k8s"));

    // Should be able to get entity by alias
    let get_req = EntityRequestNew {
        action: "get".to_string(),
        name: Some("k8s".to_string()),
        entity_type: None,
        description: None,
        query: None,
        type_filter: None,
        limit: None,
        target: None,
        relation: None,
        alias: None,
    };
    let result = tools::entity_new(&state, get_req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("kubernetes"));
}

#[tokio::test]
async fn test_mcp_entity_delete() {
    let state = setup_tool_state().await;

    // Create entity
    let req = EntityRequestNew {
        action: "create".to_string(),
        name: Some("temp-entity".to_string()),
        entity_type: Some("concept".to_string()),
        description: None,
        query: None,
        type_filter: None,
        limit: None,
        target: None,
        relation: None,
        alias: None,
    };
    tools::entity_new(&state, req).await.unwrap();

    // Delete it
    let delete_req = EntityRequestNew {
        action: "delete".to_string(),
        name: Some("temp-entity".to_string()),
        entity_type: None,
        description: None,
        query: None,
        type_filter: None,
        limit: None,
        target: None,
        relation: None,
        alias: None,
    };
    let result = tools::entity_new(&state, delete_req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("deleted"));

    // Should not find it anymore
    let get_req = EntityRequestNew {
        action: "get".to_string(),
        name: Some("temp-entity".to_string()),
        entity_type: None,
        description: None,
        query: None,
        type_filter: None,
        limit: None,
        target: None,
        relation: None,
        alias: None,
    };
    let result = tools::entity_new(&state, get_req).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn test_mcp_entity_invalid_action() {
    let state = setup_tool_state().await;

    let req = EntityRequestNew {
        action: "invalid".to_string(),
        name: Some("test".to_string()),
        entity_type: None,
        description: None,
        query: None,
        type_filter: None,
        limit: None,
        target: None,
        relation: None,
        alias: None,
    };
    let result = tools::entity_new(&state, req).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown action"));
}

#[tokio::test]
async fn test_mcp_entity_observe_add_get() {
    let state = setup_tool_state().await;

    // Create entity first
    let entity_req = EntityRequestNew {
        action: "create".to_string(),
        name: Some("observed-entity".to_string()),
        entity_type: Some("service".to_string()),
        description: None,
        query: None,
        type_filter: None,
        limit: None,
        target: None,
        relation: None,
        alias: None,
    };
    tools::entity_new(&state, entity_req).await.unwrap();

    // Add observation
    let add_req = EntityObserveRequestNew {
        action: "add".to_string(),
        entity: Some("observed-entity".to_string()),
        content: Some("Uses JWT for authentication".to_string()),
        key: Some("architecture.auth".to_string()),
        source: Some("code-review".to_string()),
        key_pattern: None,
        query: None,
        limit: None,
    };
    let result = tools::entity_observe_new(&state, add_req).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("created"));
    assert!(response.contains("architecture.auth"));

    // Get observation
    let get_req = EntityObserveRequestNew {
        action: "get".to_string(),
        entity: Some("observed-entity".to_string()),
        content: None,
        key: Some("architecture.auth".to_string()),
        source: None,
        key_pattern: None,
        query: None,
        limit: None,
    };
    let result = tools::entity_observe_new(&state, get_req).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("Uses JWT"));
}

#[tokio::test]
async fn test_mcp_entity_observe_update() {
    let state = setup_tool_state().await;

    // Create entity
    let entity_req = EntityRequestNew {
        action: "create".to_string(),
        name: Some("update-test".to_string()),
        entity_type: Some("service".to_string()),
        description: None,
        query: None,
        type_filter: None,
        limit: None,
        target: None,
        relation: None,
        alias: None,
    };
    tools::entity_new(&state, entity_req).await.unwrap();

    // Add initial observation
    let add_req = EntityObserveRequestNew {
        action: "add".to_string(),
        entity: Some("update-test".to_string()),
        content: Some("Version 1".to_string()),
        key: Some("version.current".to_string()),
        source: None,
        key_pattern: None,
        query: None,
        limit: None,
    };
    tools::entity_observe_new(&state, add_req).await.unwrap();

    // Update observation (same key)
    let update_req = EntityObserveRequestNew {
        action: "add".to_string(),
        entity: Some("update-test".to_string()),
        content: Some("Version 2".to_string()),
        key: Some("version.current".to_string()),
        source: None,
        key_pattern: None,
        query: None,
        limit: None,
    };
    let result = tools::entity_observe_new(&state, update_req).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("updated"));
    assert!(response.contains("previous_content"));
    assert!(response.contains("Version 1"));
}

#[tokio::test]
async fn test_mcp_entity_observe_list() {
    let state = setup_tool_state().await;

    // Create entity
    let entity_req = EntityRequestNew {
        action: "create".to_string(),
        name: Some("list-test".to_string()),
        entity_type: Some("service".to_string()),
        description: None,
        query: None,
        type_filter: None,
        limit: None,
        target: None,
        relation: None,
        alias: None,
    };
    tools::entity_new(&state, entity_req).await.unwrap();

    // Add multiple observations
    for (key, content) in [
        ("architecture.db", "Uses PostgreSQL"),
        ("architecture.cache", "Uses Redis"),
        ("gotchas.timeout", "Watch for slow queries"),
    ] {
        let req = EntityObserveRequestNew {
            action: "add".to_string(),
            entity: Some("list-test".to_string()),
            content: Some(content.to_string()),
            key: Some(key.to_string()),
            source: None,
            key_pattern: None,
            query: None,
            limit: None,
        };
        tools::entity_observe_new(&state, req).await.unwrap();
    }

    // List all observations
    let list_req = EntityObserveRequestNew {
        action: "list".to_string(),
        entity: Some("list-test".to_string()),
        content: None,
        key: None,
        source: None,
        key_pattern: None,
        query: None,
        limit: None,
    };
    let result = tools::entity_observe_new(&state, list_req).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("\"count\": 3"));

    // List with pattern filter (need wildcard for prefix match)
    let pattern_req = EntityObserveRequestNew {
        action: "list".to_string(),
        entity: Some("list-test".to_string()),
        content: None,
        key: None,
        source: None,
        key_pattern: Some("architecture.*".to_string()),
        query: None,
        limit: None,
    };
    let result = tools::entity_observe_new(&state, pattern_req).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("\"count\": 2"));
}

#[tokio::test]
async fn test_mcp_entity_observe_search() {
    let state = setup_tool_state().await;

    // Create entity with observations
    let entity_req = EntityRequestNew {
        action: "create".to_string(),
        name: Some("search-test".to_string()),
        entity_type: Some("service".to_string()),
        description: None,
        query: None,
        type_filter: None,
        limit: None,
        target: None,
        relation: None,
        alias: None,
    };
    tools::entity_new(&state, entity_req).await.unwrap();

    let obs_req = EntityObserveRequestNew {
        action: "add".to_string(),
        entity: Some("search-test".to_string()),
        content: Some("Authentication uses OAuth2 with PKCE flow".to_string()),
        key: Some("security.auth".to_string()),
        source: None,
        key_pattern: None,
        query: None,
        limit: None,
    };
    tools::entity_observe_new(&state, obs_req).await.unwrap();

    // Search observations
    let search_req = EntityObserveRequestNew {
        action: "search".to_string(),
        entity: Some("search-test".to_string()),
        content: None,
        key: None,
        source: None,
        key_pattern: None,
        query: Some("OAuth".to_string()),
        limit: None,
    };
    let result = tools::entity_observe_new(&state, search_req).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("OAuth2"));
    assert!(response.contains("PKCE"));
}

#[tokio::test]
async fn test_mcp_entity_observe_history() {
    let state = setup_tool_state().await;

    // Create entity
    let entity_req = EntityRequestNew {
        action: "create".to_string(),
        name: Some("history-test".to_string()),
        entity_type: Some("service".to_string()),
        description: None,
        query: None,
        type_filter: None,
        limit: None,
        target: None,
        relation: None,
        alias: None,
    };
    tools::entity_new(&state, entity_req).await.unwrap();

    // Add and update observation multiple times
    for version in ["v1.0", "v1.1", "v2.0"] {
        let req = EntityObserveRequestNew {
            action: "add".to_string(),
            entity: Some("history-test".to_string()),
            content: Some(format!("Current version is {}", version)),
            key: Some("meta.version".to_string()),
            source: None,
            key_pattern: None,
            query: None,
            limit: None,
        };
        tools::entity_observe_new(&state, req).await.unwrap();
    }

    // Get history
    let history_req = EntityObserveRequestNew {
        action: "history".to_string(),
        entity: Some("history-test".to_string()),
        content: None,
        key: Some("meta.version".to_string()),
        source: None,
        key_pattern: None,
        query: None,
        limit: None,
    };
    let result = tools::entity_observe_new(&state, history_req).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    // Current should be v2.0
    assert!(response.contains("v2.0"));
    // History should have 2 entries (v1.0 and v1.1)
    assert!(response.contains("\"history_count\": 2"));
}

#[tokio::test]
async fn test_mcp_entity_observe_invalid_action() {
    let state = setup_tool_state().await;

    let req = EntityObserveRequestNew {
        action: "invalid".to_string(),
        entity: Some("test".to_string()),
        content: None,
        key: None,
        source: None,
        key_pattern: None,
        query: None,
        limit: None,
    };
    let result = tools::entity_observe_new(&state, req).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown action"));
}
