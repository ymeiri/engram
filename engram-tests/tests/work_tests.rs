//! Integration tests for Layer 7: Work Management.
//!
//! Tests projects, tasks, PRs, work observations, entity connections,
//! and work context for sessions.

use engram_core::entity::EntityType;
use engram_core::id::Id;
use engram_core::work::{PrStatus, ProjectStatus, TaskPriority, TaskStatus};
use engram_index::{EntityService, WorkService};
use engram_store::{connect_and_init, StoreConfig};

// =============================================================================
// Test Fixtures
// =============================================================================

/// Create an in-memory database for testing.
async fn setup_db() -> engram_store::Db {
    let config = StoreConfig::memory();
    connect_and_init(&config)
        .await
        .expect("Failed to connect to test database")
}

/// Create a work service with initialized schema.
async fn setup_work_service() -> WorkService {
    let db = setup_db().await;
    let service = WorkService::with_defaults(db).expect("Failed to create work service");
    service
        .init()
        .await
        .expect("Failed to initialize work schema");
    service
}

/// Create both work and entity services for tests that need entity connections.
async fn setup_services() -> (WorkService, EntityService) {
    let db = setup_db().await;

    // Initialize entity schema FIRST
    let entity_service = EntityService::new(db.clone());
    entity_service
        .init()
        .await
        .expect("Failed to initialize entity schema");

    // Then initialize work schema
    let work_service = WorkService::with_defaults(db).expect("Failed to create work service");
    work_service
        .init()
        .await
        .expect("Failed to initialize work schema");

    (work_service, entity_service)
}

// =============================================================================
// Project CRUD Tests
// =============================================================================

#[tokio::test]
async fn test_create_project() {
    let service = setup_work_service().await;

    let project = service
        .create_project("my-feature", Some("Implement the new feature"))
        .await
        .expect("Failed to create project");

    assert_eq!(project.name, "my-feature");
    assert_eq!(
        project.description,
        Some("Implement the new feature".to_string())
    );
    assert_eq!(project.status, ProjectStatus::Active);
}

#[tokio::test]
async fn test_create_project_without_description() {
    let service = setup_work_service().await;

    let project = service
        .create_project("minimal-project", None)
        .await
        .expect("Failed to create project");

    assert_eq!(project.name, "minimal-project");
    assert!(project.description.is_none());
}

#[tokio::test]
async fn test_create_duplicate_project_fails() {
    let service = setup_work_service().await;

    service
        .create_project("unique-name", None)
        .await
        .expect("Failed to create first project");

    let result = service.create_project("unique-name", None).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));
}

#[tokio::test]
async fn test_get_project_by_name() {
    let service = setup_work_service().await;

    service
        .create_project("findable", Some("A project to find"))
        .await
        .expect("Failed to create project");

    let found = service
        .get_project("findable")
        .await
        .expect("Failed to get project")
        .expect("Project not found");

    assert_eq!(found.name, "findable");
    assert_eq!(found.description, Some("A project to find".to_string()));
}

#[tokio::test]
async fn test_get_nonexistent_project() {
    let service = setup_work_service().await;

    let result = service
        .get_project("does-not-exist")
        .await
        .expect("Query failed");
    assert!(result.is_none());
}

#[tokio::test]
async fn test_list_projects() {
    let service = setup_work_service().await;

    service.create_project("proj-1", None).await.unwrap();
    service.create_project("proj-2", None).await.unwrap();
    service.create_project("proj-3", None).await.unwrap();

    let all = service.list_projects(None).await.expect("Failed to list");
    assert_eq!(all.len(), 3);
}

#[tokio::test]
async fn test_list_projects_by_status() {
    let service = setup_work_service().await;

    service.create_project("active-1", None).await.unwrap();
    service.create_project("active-2", None).await.unwrap();

    // Create and complete one
    service.create_project("completed-1", None).await.unwrap();
    service
        .update_project_status("completed-1", ProjectStatus::Completed)
        .await
        .unwrap();

    let active = service
        .list_projects(Some(ProjectStatus::Active))
        .await
        .unwrap();
    assert_eq!(active.len(), 2);

    let completed = service
        .list_projects(Some(ProjectStatus::Completed))
        .await
        .unwrap();
    assert_eq!(completed.len(), 1);
    assert_eq!(completed[0].name, "completed-1");
}

#[tokio::test]
async fn test_update_project_status() {
    let service = setup_work_service().await;

    service.create_project("status-test", None).await.unwrap();

    // Update to Planning
    let project = service
        .update_project_status("status-test", ProjectStatus::Planning)
        .await
        .expect("Failed to update status");
    assert_eq!(project.status, ProjectStatus::Planning);

    // Update to Completed
    let project = service
        .update_project_status("status-test", ProjectStatus::Completed)
        .await
        .expect("Failed to update status");
    assert_eq!(project.status, ProjectStatus::Completed);

    // Update to Archived
    let project = service
        .update_project_status("status-test", ProjectStatus::Archived)
        .await
        .expect("Failed to update status");
    assert_eq!(project.status, ProjectStatus::Archived);
}

#[tokio::test]
async fn test_delete_project() {
    let service = setup_work_service().await;

    service.create_project("to-delete", None).await.unwrap();

    // Verify exists
    assert!(service.get_project("to-delete").await.unwrap().is_some());

    // Delete
    service
        .delete_project("to-delete")
        .await
        .expect("Failed to delete");

    // Verify gone
    assert!(service.get_project("to-delete").await.unwrap().is_none());
}

// =============================================================================
// Task CRUD Tests
// =============================================================================

#[tokio::test]
async fn test_create_task() {
    let service = setup_work_service().await;
    service.create_project("task-project", None).await.unwrap();

    let task = service
        .create_task(
            "task-project",
            "implement-auth",
            Some("Add OAuth support"),
            None,
        )
        .await
        .expect("Failed to create task");

    assert_eq!(task.name, "implement-auth");
    assert_eq!(task.description, Some("Add OAuth support".to_string()));
    assert_eq!(task.status, TaskStatus::Todo);
    assert_eq!(task.priority, TaskPriority::Medium);
}

#[tokio::test]
async fn test_create_task_with_jira_key() {
    let service = setup_work_service().await;
    service.create_project("jira-project", None).await.unwrap();

    let task = service
        .create_task("jira-project", "oauth-task", None, Some("IDEAI-235"))
        .await
        .expect("Failed to create task");

    assert_eq!(task.jira_key, Some("IDEAI-235".to_string()));
}

#[tokio::test]
async fn test_create_duplicate_task_in_project_fails() {
    let service = setup_work_service().await;
    service.create_project("dup-task-proj", None).await.unwrap();

    service
        .create_task("dup-task-proj", "unique-task", None, None)
        .await
        .unwrap();

    let result = service
        .create_task("dup-task-proj", "unique-task", None, None)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_duplicate_jira_key_fails() {
    let service = setup_work_service().await;
    service.create_project("jira-dup-proj", None).await.unwrap();

    service
        .create_task("jira-dup-proj", "task-1", None, Some("JIRA-100"))
        .await
        .unwrap();

    let result = service
        .create_task("jira-dup-proj", "task-2", None, Some("JIRA-100"))
        .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("JIRA"));
}

#[tokio::test]
async fn test_get_task_by_jira_key() {
    let service = setup_work_service().await;
    service.create_project("jira-lookup", None).await.unwrap();

    service
        .create_task(
            "jira-lookup",
            "my-task",
            Some("Task description"),
            Some("ABC-123"),
        )
        .await
        .unwrap();

    let found = service
        .get_task("ABC-123")
        .await
        .expect("Failed to get task")
        .expect("Task not found");

    assert_eq!(found.name, "my-task");
    assert_eq!(found.jira_key, Some("ABC-123".to_string()));
}

#[tokio::test]
async fn test_get_task_in_project() {
    let service = setup_work_service().await;
    service
        .create_project("task-lookup-proj", None)
        .await
        .unwrap();

    service
        .create_task("task-lookup-proj", "specific-task", None, None)
        .await
        .unwrap();

    let found = service
        .get_task_in_project("task-lookup-proj", "specific-task")
        .await
        .expect("Failed to get task")
        .expect("Task not found");

    assert_eq!(found.name, "specific-task");
}

#[tokio::test]
async fn test_list_tasks() {
    let service = setup_work_service().await;
    service
        .create_project("multi-task-proj", None)
        .await
        .unwrap();

    service
        .create_task("multi-task-proj", "task-1", None, None)
        .await
        .unwrap();
    service
        .create_task("multi-task-proj", "task-2", None, None)
        .await
        .unwrap();
    service
        .create_task("multi-task-proj", "task-3", None, None)
        .await
        .unwrap();

    let tasks = service.list_tasks("multi-task-proj", None).await.unwrap();
    assert_eq!(tasks.len(), 3);
}

#[tokio::test]
async fn test_list_tasks_by_status() {
    let service = setup_work_service().await;
    service
        .create_project("status-task-proj", None)
        .await
        .unwrap();

    service
        .create_task("status-task-proj", "todo-1", None, Some("T-1"))
        .await
        .unwrap();
    service
        .create_task("status-task-proj", "todo-2", None, Some("T-2"))
        .await
        .unwrap();
    service
        .create_task("status-task-proj", "done-1", None, Some("T-3"))
        .await
        .unwrap();
    service
        .update_task_status("T-3", TaskStatus::Done)
        .await
        .unwrap();

    let todo = service
        .list_tasks("status-task-proj", Some(TaskStatus::Todo))
        .await
        .unwrap();
    assert_eq!(todo.len(), 2);

    let done = service
        .list_tasks("status-task-proj", Some(TaskStatus::Done))
        .await
        .unwrap();
    assert_eq!(done.len(), 1);
}

#[tokio::test]
async fn test_update_task_status() {
    let service = setup_work_service().await;
    service
        .create_project("task-status-proj", None)
        .await
        .unwrap();
    service
        .create_task("task-status-proj", "status-task", None, Some("TS-1"))
        .await
        .unwrap();

    // Update to InProgress
    let task = service
        .update_task_status("TS-1", TaskStatus::InProgress)
        .await
        .unwrap();
    assert_eq!(task.status, TaskStatus::InProgress);

    // Update to Blocked
    let task = service
        .update_task_status("TS-1", TaskStatus::Blocked)
        .await
        .unwrap();
    assert_eq!(task.status, TaskStatus::Blocked);

    // Update to Done
    let task = service
        .update_task_status("TS-1", TaskStatus::Done)
        .await
        .unwrap();
    assert_eq!(task.status, TaskStatus::Done);
}

#[tokio::test]
async fn test_update_task_priority() {
    let service = setup_work_service().await;
    service.create_project("priority-proj", None).await.unwrap();
    service
        .create_task("priority-proj", "priority-task", None, Some("P-1"))
        .await
        .unwrap();

    // Default is Medium
    let task = service.get_task("P-1").await.unwrap().unwrap();
    assert_eq!(task.priority, TaskPriority::Medium);

    // Update to Critical
    let task = service
        .update_task_priority("P-1", TaskPriority::Critical)
        .await
        .unwrap();
    assert_eq!(task.priority, TaskPriority::Critical);

    // Update to Low
    let task = service
        .update_task_priority("P-1", TaskPriority::Low)
        .await
        .unwrap();
    assert_eq!(task.priority, TaskPriority::Low);
}

#[tokio::test]
async fn test_delete_task() {
    let service = setup_work_service().await;
    service
        .create_project("delete-task-proj", None)
        .await
        .unwrap();
    service
        .create_task("delete-task-proj", "to-delete", None, Some("DEL-1"))
        .await
        .unwrap();

    // Verify exists
    assert!(service.get_task("DEL-1").await.unwrap().is_some());

    // Delete
    service
        .delete_task("DEL-1")
        .await
        .expect("Failed to delete task");

    // Verify gone
    assert!(service.get_task("DEL-1").await.unwrap().is_none());
}

// =============================================================================
// PR Tests
// =============================================================================

#[tokio::test]
async fn test_add_pr() {
    let service = setup_work_service().await;
    service.create_project("pr-project", None).await.unwrap();

    let pr = service
        .add_pr(
            "pr-project",
            None,
            "https://github.com/org/repo/pull/123",
            Some("Add new feature"),
        )
        .await
        .expect("Failed to add PR");

    assert_eq!(pr.url, "https://github.com/org/repo/pull/123");
    assert_eq!(pr.repo, "repo");
    assert_eq!(pr.pr_number, 123);
    assert_eq!(pr.title, Some("Add new feature".to_string()));
    assert_eq!(pr.status, PrStatus::Open);
}

#[tokio::test]
async fn test_add_pr_linked_to_task() {
    let service = setup_work_service().await;
    service
        .create_project("pr-task-project", None)
        .await
        .unwrap();
    service
        .create_task("pr-task-project", "pr-task", None, Some("PT-1"))
        .await
        .unwrap();

    let pr = service
        .add_pr(
            "pr-task-project",
            Some("PT-1"),
            "https://github.com/org/repo/pull/456",
            None,
        )
        .await
        .expect("Failed to add PR");

    assert!(pr.task_id.is_some());
}

#[tokio::test]
async fn test_add_duplicate_pr_url_fails() {
    let service = setup_work_service().await;
    service.create_project("dup-pr-proj", None).await.unwrap();

    service
        .add_pr(
            "dup-pr-proj",
            None,
            "https://github.com/org/repo/pull/789",
            None,
        )
        .await
        .unwrap();

    let result = service
        .add_pr(
            "dup-pr-proj",
            None,
            "https://github.com/org/repo/pull/789",
            None,
        )
        .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));
}

#[tokio::test]
async fn test_get_pr_by_url() {
    let service = setup_work_service().await;
    service.create_project("pr-get-proj", None).await.unwrap();

    service
        .add_pr(
            "pr-get-proj",
            None,
            "https://github.com/org/repo/pull/100",
            Some("Test PR"),
        )
        .await
        .unwrap();

    let found = service
        .get_pr("https://github.com/org/repo/pull/100")
        .await
        .expect("Failed to get PR")
        .expect("PR not found");

    assert_eq!(found.pr_number, 100);
    assert_eq!(found.title, Some("Test PR".to_string()));
}

#[tokio::test]
async fn test_list_prs() {
    let service = setup_work_service().await;
    service.create_project("list-pr-proj", None).await.unwrap();

    service
        .add_pr(
            "list-pr-proj",
            None,
            "https://github.com/org/repo/pull/1",
            None,
        )
        .await
        .unwrap();
    service
        .add_pr(
            "list-pr-proj",
            None,
            "https://github.com/org/repo/pull/2",
            None,
        )
        .await
        .unwrap();
    service
        .add_pr(
            "list-pr-proj",
            None,
            "https://github.com/org/repo/pull/3",
            None,
        )
        .await
        .unwrap();

    let prs = service.list_prs("list-pr-proj", None).await.unwrap();
    assert_eq!(prs.len(), 3);
}

#[tokio::test]
async fn test_list_prs_by_task() {
    let service = setup_work_service().await;
    service
        .create_project("pr-filter-proj", None)
        .await
        .unwrap();
    service
        .create_task("pr-filter-proj", "task-a", None, Some("TF-A"))
        .await
        .unwrap();
    service
        .create_task("pr-filter-proj", "task-b", None, Some("TF-B"))
        .await
        .unwrap();

    service
        .add_pr(
            "pr-filter-proj",
            Some("TF-A"),
            "https://github.com/org/repo/pull/10",
            None,
        )
        .await
        .unwrap();
    service
        .add_pr(
            "pr-filter-proj",
            Some("TF-A"),
            "https://github.com/org/repo/pull/11",
            None,
        )
        .await
        .unwrap();
    service
        .add_pr(
            "pr-filter-proj",
            Some("TF-B"),
            "https://github.com/org/repo/pull/20",
            None,
        )
        .await
        .unwrap();

    let task_a_prs = service
        .list_prs("pr-filter-proj", Some("TF-A"))
        .await
        .unwrap();
    assert_eq!(task_a_prs.len(), 2);

    let task_b_prs = service
        .list_prs("pr-filter-proj", Some("TF-B"))
        .await
        .unwrap();
    assert_eq!(task_b_prs.len(), 1);
}

#[tokio::test]
async fn test_update_pr_status() {
    let service = setup_work_service().await;
    service
        .create_project("pr-status-proj", None)
        .await
        .unwrap();
    service
        .add_pr(
            "pr-status-proj",
            None,
            "https://github.com/org/repo/pull/50",
            None,
        )
        .await
        .unwrap();

    let pr = service
        .update_pr_status("https://github.com/org/repo/pull/50", PrStatus::Merged)
        .await
        .unwrap();
    assert_eq!(pr.status, PrStatus::Merged);

    let pr = service
        .update_pr_status("https://github.com/org/repo/pull/50", PrStatus::Closed)
        .await
        .unwrap();
    assert_eq!(pr.status, PrStatus::Closed);
}

// =============================================================================
// Entity Connection Tests
// =============================================================================

#[tokio::test]
async fn test_connect_project_to_entity() {
    let (work_service, entity_service) = setup_services().await;

    entity_service
        .create_entity("auth-service", EntityType::Service, None)
        .await
        .unwrap();
    work_service
        .create_project("entity-proj", None)
        .await
        .unwrap();

    work_service
        .connect_project_to_entity("entity-proj", "auth-service", Some("involves"))
        .await
        .expect("Failed to connect entity");

    let entities = work_service
        .get_project_entities("entity-proj")
        .await
        .unwrap();
    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].name, "auth-service");
}

#[tokio::test]
async fn test_connect_project_to_multiple_entities() {
    let (work_service, entity_service) = setup_services().await;

    entity_service
        .create_entity("svc-1", EntityType::Service, None)
        .await
        .unwrap();
    entity_service
        .create_entity("svc-2", EntityType::Service, None)
        .await
        .unwrap();
    entity_service
        .create_entity("svc-3", EntityType::Service, None)
        .await
        .unwrap();
    work_service
        .create_project("multi-entity-proj", None)
        .await
        .unwrap();

    work_service
        .connect_project_to_entity("multi-entity-proj", "svc-1", None)
        .await
        .unwrap();
    work_service
        .connect_project_to_entity("multi-entity-proj", "svc-2", None)
        .await
        .unwrap();
    work_service
        .connect_project_to_entity("multi-entity-proj", "svc-3", None)
        .await
        .unwrap();

    let entities = work_service
        .get_project_entities("multi-entity-proj")
        .await
        .unwrap();
    assert_eq!(entities.len(), 3);
}

#[tokio::test]
async fn test_connect_task_to_entity() {
    let (work_service, entity_service) = setup_services().await;

    entity_service
        .create_entity("database", EntityType::Service, None)
        .await
        .unwrap();
    work_service
        .create_project("task-entity-proj", None)
        .await
        .unwrap();
    work_service
        .create_task("task-entity-proj", "db-task", None, Some("TE-1"))
        .await
        .unwrap();

    work_service
        .connect_task_to_entity("TE-1", "database", Some("modifies"))
        .await
        .expect("Failed to connect entity to task");

    let entities = work_service.get_task_entities("TE-1").await.unwrap();
    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].name, "database");
}

#[tokio::test]
async fn test_connect_nonexistent_entity_fails() {
    let service = setup_work_service().await;
    service
        .create_project("no-entity-proj", None)
        .await
        .unwrap();

    let result = service
        .connect_project_to_entity("no-entity-proj", "nonexistent", None)
        .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

// =============================================================================
// Observation Tests
// =============================================================================

#[tokio::test]
async fn test_add_project_observation() {
    let service = setup_work_service().await;
    service.create_project("obs-proj", None).await.unwrap();

    let obs = service
        .add_project_observation(
            "obs-proj",
            "This project uses OAuth2 for authentication",
            Some("architecture.auth"),
        )
        .await
        .expect("Failed to add observation");

    assert_eq!(obs.content, "This project uses OAuth2 for authentication");
    assert_eq!(obs.key, Some("architecture.auth".to_string()));
    assert!(obs.embedding.is_some()); // Should have embedding
}

#[tokio::test]
async fn test_add_project_observation_without_key() {
    let service = setup_work_service().await;
    service
        .create_project("obs-nokey-proj", None)
        .await
        .unwrap();

    let obs = service
        .add_project_observation("obs-nokey-proj", "A random observation", None)
        .await
        .expect("Failed to add observation");

    assert!(obs.key.is_none());
}

#[tokio::test]
async fn test_get_project_observations() {
    let service = setup_work_service().await;
    service.create_project("get-obs-proj", None).await.unwrap();

    service
        .add_project_observation("get-obs-proj", "Obs 1", Some("key1"))
        .await
        .unwrap();
    service
        .add_project_observation("get-obs-proj", "Obs 2", Some("key2"))
        .await
        .unwrap();
    service
        .add_project_observation("get-obs-proj", "Obs 3", None)
        .await
        .unwrap();

    let observations = service
        .get_project_observations("get-obs-proj")
        .await
        .unwrap();
    assert_eq!(observations.len(), 3);
}

#[tokio::test]
async fn test_add_task_observation() {
    let service = setup_work_service().await;
    service.create_project("task-obs-proj", None).await.unwrap();
    service
        .create_task("task-obs-proj", "obs-task", None, Some("TO-1"))
        .await
        .unwrap();

    let obs = service
        .add_task_observation(
            "TO-1",
            "This task requires refactoring",
            Some("gotchas.refactor"),
        )
        .await
        .expect("Failed to add task observation");

    assert_eq!(obs.content, "This task requires refactoring");
    assert_eq!(obs.key, Some("gotchas.refactor".to_string()));
}

#[tokio::test]
async fn test_get_task_observations() {
    let service = setup_work_service().await;
    service
        .create_project("task-get-obs-proj", None)
        .await
        .unwrap();
    service
        .create_task("task-get-obs-proj", "multi-obs-task", None, Some("TGO-1"))
        .await
        .unwrap();

    service
        .add_task_observation("TGO-1", "Note 1", Some("note.1"))
        .await
        .unwrap();
    service
        .add_task_observation("TGO-1", "Note 2", Some("note.2"))
        .await
        .unwrap();

    let observations = service.get_task_observations("TGO-1").await.unwrap();
    assert_eq!(observations.len(), 2);
}

// =============================================================================
// Work Context Tests
// =============================================================================

#[tokio::test]
async fn test_join_work_project_only() {
    let service = setup_work_service().await;
    service.create_project("join-proj", None).await.unwrap();
    let session_id = Id::new();

    let ctx = service
        .join_work(&session_id, "join-proj", None)
        .await
        .expect("Failed to join work");

    assert_eq!(ctx.session_id, session_id);
    assert!(ctx.project_id.is_some());
    assert!(ctx.task_id.is_none());
}

#[tokio::test]
async fn test_join_work_with_task() {
    let service = setup_work_service().await;
    service
        .create_project("join-task-proj", None)
        .await
        .unwrap();
    service
        .create_task("join-task-proj", "join-task", None, Some("JT-1"))
        .await
        .unwrap();
    let session_id = Id::new();

    let ctx = service
        .join_work(&session_id, "join-task-proj", Some("JT-1"))
        .await
        .expect("Failed to join work");

    assert!(ctx.project_id.is_some());
    assert!(ctx.task_id.is_some());
}

#[tokio::test]
async fn test_get_work_context() {
    let service = setup_work_service().await;
    service.create_project("ctx-proj", None).await.unwrap();
    let session_id = Id::new();

    service
        .join_work(&session_id, "ctx-proj", None)
        .await
        .unwrap();

    let ctx = service
        .get_work_context(&session_id)
        .await
        .expect("Failed to get context")
        .expect("Context not found");

    assert_eq!(ctx.session_id, session_id);
}

#[tokio::test]
async fn test_leave_work() {
    let service = setup_work_service().await;
    service.create_project("leave-proj", None).await.unwrap();
    let session_id = Id::new();

    service
        .join_work(&session_id, "leave-proj", None)
        .await
        .unwrap();
    assert!(service
        .get_work_context(&session_id)
        .await
        .unwrap()
        .is_some());

    service
        .leave_work(&session_id)
        .await
        .expect("Failed to leave");

    assert!(service
        .get_work_context(&session_id)
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn test_get_full_context() {
    let (work_service, entity_service) = setup_services().await;

    // Create entity
    entity_service
        .create_entity("full-ctx-entity", EntityType::Service, None)
        .await
        .unwrap();

    // Create project
    work_service
        .create_project("full-ctx-proj", Some("Full context test"))
        .await
        .unwrap();

    // Create task
    work_service
        .create_task("full-ctx-proj", "full-ctx-task", None, Some("FC-1"))
        .await
        .unwrap();

    // Add PRs
    work_service
        .add_pr(
            "full-ctx-proj",
            Some("FC-1"),
            "https://github.com/org/repo/pull/999",
            Some("Full context PR"),
        )
        .await
        .unwrap();

    // Connect entity
    work_service
        .connect_project_to_entity("full-ctx-proj", "full-ctx-entity", None)
        .await
        .unwrap();

    // Add observations
    work_service
        .add_project_observation("full-ctx-proj", "Project observation", Some("test.obs"))
        .await
        .unwrap();
    work_service
        .add_task_observation("FC-1", "Task observation", Some("task.obs"))
        .await
        .unwrap();

    // Get full context
    let ctx = work_service
        .get_full_context("full-ctx-proj", Some("FC-1"))
        .await
        .expect("Failed to get full context");

    assert_eq!(ctx.project.name, "full-ctx-proj");
    assert!(ctx.task.is_some());
    assert_eq!(ctx.task.as_ref().unwrap().name, "full-ctx-task");
    assert_eq!(ctx.prs.len(), 1);
    assert_eq!(ctx.connected_entities.len(), 1);
    assert!(!ctx.project_observations.is_empty());
    assert!(!ctx.task_observations.is_empty());
}

#[tokio::test]
async fn test_get_full_context_project_only() {
    let service = setup_work_service().await;
    service
        .create_project("ctx-only-proj", Some("Context only project"))
        .await
        .unwrap();
    service
        .add_project_observation("ctx-only-proj", "Project note", None)
        .await
        .unwrap();

    let ctx = service
        .get_full_context("ctx-only-proj", None)
        .await
        .expect("Failed to get full context");

    assert_eq!(ctx.project.name, "ctx-only-proj");
    assert!(ctx.task.is_none());
    assert!(ctx.task_observations.is_empty());
}

// =============================================================================
// Statistics Tests
// =============================================================================

#[tokio::test]
async fn test_stats_empty() {
    let service = setup_work_service().await;

    let stats = service.stats().await.expect("Failed to get stats");

    assert_eq!(stats.project_count, 0);
    assert_eq!(stats.task_count, 0);
    assert_eq!(stats.pr_count, 0);
}

#[tokio::test]
async fn test_stats_with_data() {
    let service = setup_work_service().await;

    // Create projects
    service.create_project("stat-proj-1", None).await.unwrap();
    service.create_project("stat-proj-2", None).await.unwrap();

    // Create tasks - use JIRA key format for lookup
    let task1 = service
        .create_task("stat-proj-1", "STAT-1", None, None)
        .await
        .unwrap();
    service
        .create_task("stat-proj-1", "STAT-2", None, None)
        .await
        .unwrap();
    service
        .create_task("stat-proj-2", "STAT-3", None, None)
        .await
        .unwrap();

    // Create PRs
    service
        .add_pr(
            "stat-proj-1",
            None,
            "https://github.com/org/repo/pull/1000",
            None,
        )
        .await
        .unwrap();

    // Add observations - use task ID for lookup
    service
        .add_project_observation("stat-proj-1", "Obs", None)
        .await
        .unwrap();
    service
        .add_task_observation(&task1.id.to_string(), "Task obs", None)
        .await
        .unwrap();

    let stats = service.stats().await.expect("Failed to get stats");

    assert_eq!(stats.project_count, 2);
    assert_eq!(stats.task_count, 3);
    assert_eq!(stats.pr_count, 1);
    assert_eq!(stats.project_observation_count, 1);
    assert_eq!(stats.task_observation_count, 1);
}

// =============================================================================
// Edge Cases and Error Handling
// =============================================================================

#[tokio::test]
async fn test_create_task_for_nonexistent_project_fails() {
    let service = setup_work_service().await;

    let result = service
        .create_task("nonexistent-project", "orphan-task", None, None)
        .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_add_pr_for_nonexistent_project_fails() {
    let service = setup_work_service().await;

    let result = service
        .add_pr(
            "nonexistent",
            None,
            "https://github.com/org/repo/pull/1",
            None,
        )
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_add_pr_with_nonexistent_task_fails() {
    let service = setup_work_service().await;
    service
        .create_project("pr-no-task-proj", None)
        .await
        .unwrap();

    let result = service
        .add_pr(
            "pr-no-task-proj",
            Some("NONEXISTENT-123"),
            "https://github.com/org/repo/pull/2",
            None,
        )
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_status_for_nonexistent_project_fails() {
    let service = setup_work_service().await;

    let result = service
        .update_project_status("ghost-project", ProjectStatus::Completed)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_status_for_nonexistent_task_fails() {
    let service = setup_work_service().await;

    let result = service
        .update_task_status("GHOST-999", TaskStatus::Done)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_observation_for_nonexistent_project_fails() {
    let service = setup_work_service().await;

    let result = service
        .add_project_observation("ghost-proj", "observation", None)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_observation_for_nonexistent_task_fails() {
    let service = setup_work_service().await;

    let result = service
        .add_task_observation("GHOST-TASK", "observation", None)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_join_nonexistent_project_fails() {
    let service = setup_work_service().await;
    let session_id = Id::new();

    let result = service.join_work(&session_id, "ghost-project", None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_join_nonexistent_task_fails() {
    let service = setup_work_service().await;
    service.create_project("real-proj", None).await.unwrap();
    let session_id = Id::new();

    let result = service
        .join_work(&session_id, "real-proj", Some("GHOST-TASK"))
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_invalid_pr_url_fails() {
    let service = setup_work_service().await;
    service.create_project("bad-pr-proj", None).await.unwrap();

    let result = service
        .add_pr("bad-pr-proj", None, "not-a-valid-url", None)
        .await;
    assert!(result.is_err());

    let result = service
        .add_pr(
            "bad-pr-proj",
            None,
            "https://gitlab.com/org/repo/pull/1",
            None,
        )
        .await;
    assert!(result.is_err());
}

// =============================================================================
// Status Parsing Tests
// =============================================================================

#[tokio::test]
async fn test_project_status_parsing() {
    assert_eq!(ProjectStatus::parse("active"), ProjectStatus::Active);
    assert_eq!(ProjectStatus::parse("ACTIVE"), ProjectStatus::Active);
    assert_eq!(ProjectStatus::parse("planning"), ProjectStatus::Planning);
    assert_eq!(ProjectStatus::parse("completed"), ProjectStatus::Completed);
    assert_eq!(ProjectStatus::parse("archived"), ProjectStatus::Archived);
    // Unknown defaults to Active
    assert_eq!(ProjectStatus::parse("unknown"), ProjectStatus::Active);
}

#[tokio::test]
async fn test_task_status_parsing() {
    assert_eq!(TaskStatus::parse("todo"), TaskStatus::Todo);
    assert_eq!(TaskStatus::parse("in_progress"), TaskStatus::InProgress);
    assert_eq!(TaskStatus::parse("inprogress"), TaskStatus::InProgress);
    assert_eq!(TaskStatus::parse("blocked"), TaskStatus::Blocked);
    assert_eq!(TaskStatus::parse("done"), TaskStatus::Done);
    // Unknown defaults to Todo
    assert_eq!(TaskStatus::parse("unknown"), TaskStatus::Todo);
}

#[tokio::test]
async fn test_task_priority_parsing() {
    assert_eq!(TaskPriority::parse("low"), TaskPriority::Low);
    assert_eq!(TaskPriority::parse("medium"), TaskPriority::Medium);
    assert_eq!(TaskPriority::parse("high"), TaskPriority::High);
    assert_eq!(TaskPriority::parse("critical"), TaskPriority::Critical);
    // Unknown defaults to Medium
    assert_eq!(TaskPriority::parse("unknown"), TaskPriority::Medium);
}

#[tokio::test]
async fn test_pr_status_parsing() {
    assert_eq!(PrStatus::parse("open"), PrStatus::Open);
    assert_eq!(PrStatus::parse("merged"), PrStatus::Merged);
    assert_eq!(PrStatus::parse("closed"), PrStatus::Closed);
    // Unknown defaults to Open
    assert_eq!(PrStatus::parse("unknown"), PrStatus::Open);
}

// =============================================================================
// New Tool Tests: Delete PR
// =============================================================================

#[tokio::test]
async fn test_delete_pr() {
    let service = setup_work_service().await;
    service
        .create_project("pr-delete-proj", None)
        .await
        .unwrap();
    service
        .add_pr(
            "pr-delete-proj",
            None,
            "https://github.com/org/repo/pull/999",
            None,
        )
        .await
        .unwrap();

    // Verify exists
    assert!(service
        .get_pr("https://github.com/org/repo/pull/999")
        .await
        .unwrap()
        .is_some());

    // Delete
    service
        .delete_pr("https://github.com/org/repo/pull/999")
        .await
        .expect("Failed to delete PR");

    // Verify gone
    assert!(service
        .get_pr("https://github.com/org/repo/pull/999")
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn test_delete_nonexistent_pr_fails() {
    let service = setup_work_service().await;

    let result = service
        .delete_pr("https://github.com/org/repo/pull/nonexistent")
        .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

// =============================================================================
// New Tool Tests: Disconnect Entity
// =============================================================================

#[tokio::test]
async fn test_disconnect_project_from_entity() {
    let (work_service, entity_service) = setup_services().await;

    entity_service
        .create_entity("disconnect-svc", EntityType::Service, None)
        .await
        .unwrap();
    work_service
        .create_project("disconnect-proj", None)
        .await
        .unwrap();
    work_service
        .connect_project_to_entity("disconnect-proj", "disconnect-svc", None)
        .await
        .unwrap();

    // Verify connected
    let entities = work_service
        .get_project_entities("disconnect-proj")
        .await
        .unwrap();
    assert_eq!(entities.len(), 1);

    // Disconnect
    work_service
        .disconnect_project_from_entity("disconnect-proj", "disconnect-svc")
        .await
        .unwrap();

    // Verify disconnected
    let entities = work_service
        .get_project_entities("disconnect-proj")
        .await
        .unwrap();
    assert_eq!(entities.len(), 0);
}

#[tokio::test]
async fn test_disconnect_task_from_entity() {
    let (work_service, entity_service) = setup_services().await;

    entity_service
        .create_entity("task-disconnect-svc", EntityType::Service, None)
        .await
        .unwrap();
    work_service
        .create_project("task-disconnect-proj", None)
        .await
        .unwrap();
    work_service
        .create_task("task-disconnect-proj", "task-disc", None, Some("TD-1"))
        .await
        .unwrap();
    work_service
        .connect_task_to_entity("TD-1", "task-disconnect-svc", None)
        .await
        .unwrap();

    // Verify connected
    let entities = work_service.get_task_entities("TD-1").await.unwrap();
    assert_eq!(entities.len(), 1);

    // Disconnect
    work_service
        .disconnect_task_from_entity("TD-1", "task-disconnect-svc")
        .await
        .unwrap();

    // Verify disconnected
    let entities = work_service.get_task_entities("TD-1").await.unwrap();
    assert_eq!(entities.len(), 0);
}

#[tokio::test]
async fn test_disconnect_nonexistent_entity_fails() {
    let service = setup_work_service().await;
    service
        .create_project("no-entity-disconnect", None)
        .await
        .unwrap();

    let result = service
        .disconnect_project_from_entity("no-entity-disconnect", "ghost-entity")
        .await;
    assert!(result.is_err());
}

// =============================================================================
// New Tool Tests: Get Observation by Key
// =============================================================================

#[tokio::test]
async fn test_get_project_observation_by_key() {
    let service = setup_work_service().await;
    service.create_project("obs-key-proj", None).await.unwrap();

    service
        .add_project_observation(
            "obs-key-proj",
            "Architecture uses microservices",
            Some("architecture.style"),
        )
        .await
        .unwrap();

    // Get by key
    let obs = service
        .get_project_observation_by_key("obs-key-proj", "architecture.style")
        .await
        .expect("Failed to get observation")
        .expect("Observation not found");

    assert_eq!(obs.content, "Architecture uses microservices");
    assert_eq!(obs.key, Some("architecture.style".to_string()));
}

#[tokio::test]
async fn test_get_project_observation_by_key_not_found() {
    let service = setup_work_service().await;
    service
        .create_project("obs-key-notfound", None)
        .await
        .unwrap();

    let obs = service
        .get_project_observation_by_key("obs-key-notfound", "nonexistent.key")
        .await
        .expect("Query failed");

    assert!(obs.is_none());
}

#[tokio::test]
async fn test_get_task_observation_by_key() {
    let service = setup_work_service().await;
    service
        .create_project("task-obs-key-proj", None)
        .await
        .unwrap();
    service
        .create_task("task-obs-key-proj", "obs-task", None, Some("TOK-1"))
        .await
        .unwrap();

    service
        .add_task_observation("TOK-1", "This is a gotcha", Some("gotchas.edge-case"))
        .await
        .unwrap();

    // Get by key
    let obs = service
        .get_task_observation_by_key("TOK-1", "gotchas.edge-case")
        .await
        .expect("Failed to get observation")
        .expect("Observation not found");

    assert_eq!(obs.content, "This is a gotcha");
    assert_eq!(obs.key, Some("gotchas.edge-case".to_string()));
}

// =============================================================================
// New Tool Tests: Delete Observation by Key
// =============================================================================

#[tokio::test]
async fn test_delete_project_observation_by_key() {
    let service = setup_work_service().await;
    service.create_project("del-obs-proj", None).await.unwrap();

    service
        .add_project_observation("del-obs-proj", "To be deleted", Some("delete.me"))
        .await
        .unwrap();

    // Verify exists
    assert!(service
        .get_project_observation_by_key("del-obs-proj", "delete.me")
        .await
        .unwrap()
        .is_some());

    // Delete
    service
        .delete_project_observation_by_key("del-obs-proj", "delete.me")
        .await
        .unwrap();

    // Verify gone
    assert!(service
        .get_project_observation_by_key("del-obs-proj", "delete.me")
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn test_delete_task_observation_by_key() {
    let service = setup_work_service().await;
    service
        .create_project("del-task-obs-proj", None)
        .await
        .unwrap();
    service
        .create_task("del-task-obs-proj", "del-obs-task", None, Some("DTOK-1"))
        .await
        .unwrap();

    service
        .add_task_observation("DTOK-1", "To be deleted", Some("delete.task"))
        .await
        .unwrap();

    // Verify exists
    assert!(service
        .get_task_observation_by_key("DTOK-1", "delete.task")
        .await
        .unwrap()
        .is_some());

    // Delete
    service
        .delete_task_observation_by_key("DTOK-1", "delete.task")
        .await
        .unwrap();

    // Verify gone
    assert!(service
        .get_task_observation_by_key("DTOK-1", "delete.task")
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn test_delete_nonexistent_observation_fails() {
    let service = setup_work_service().await;
    service.create_project("no-obs-del", None).await.unwrap();

    let result = service
        .delete_project_observation_by_key("no-obs-del", "nonexistent.key")
        .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

// =============================================================================
// New Tool Tests: Work Stats
// =============================================================================

#[tokio::test]
async fn test_work_stats_comprehensive() {
    let (work_service, entity_service) = setup_services().await;

    // Create some data
    entity_service
        .create_entity("stats-entity", EntityType::Service, None)
        .await
        .unwrap();

    work_service
        .create_project("stats-proj-1", None)
        .await
        .unwrap();
    work_service
        .create_project("stats-proj-2", None)
        .await
        .unwrap();

    work_service
        .create_task("stats-proj-1", "stats-task-1", None, Some("ST-1"))
        .await
        .unwrap();
    work_service
        .create_task("stats-proj-1", "stats-task-2", None, Some("ST-2"))
        .await
        .unwrap();

    work_service
        .add_pr(
            "stats-proj-1",
            None,
            "https://github.com/org/repo/pull/1001",
            None,
        )
        .await
        .unwrap();
    work_service
        .add_pr(
            "stats-proj-1",
            None,
            "https://github.com/org/repo/pull/1002",
            None,
        )
        .await
        .unwrap();

    work_service
        .add_project_observation("stats-proj-1", "Project obs 1", Some("proj.obs1"))
        .await
        .unwrap();
    work_service
        .add_project_observation("stats-proj-1", "Project obs 2", Some("proj.obs2"))
        .await
        .unwrap();

    work_service
        .add_task_observation("ST-1", "Task obs", Some("task.obs"))
        .await
        .unwrap();

    // Get stats
    let stats = work_service.stats().await.expect("Failed to get stats");

    assert_eq!(stats.project_count, 2);
    assert_eq!(stats.task_count, 2);
    assert_eq!(stats.pr_count, 2);
    assert_eq!(stats.project_observation_count, 2);
    assert_eq!(stats.task_observation_count, 1);
}

// =============================================================================
// MCP Tool Handler Tests (Consolidated Action-Based API)
// =============================================================================

use engram_mcp::tools::{
    self, ToolState, WorkContextRequestNew, WorkObserveRequest, WorkPrRequest, WorkProjectRequest,
    WorkTaskRequest,
};

/// Create a ToolState with initialized WorkService for MCP tool tests.
async fn setup_tool_state() -> ToolState {
    let db = setup_db().await;
    let work_service = WorkService::with_defaults(db).expect("Failed to create work service");
    work_service
        .init()
        .await
        .expect("Failed to initialize work schema");

    let state = ToolState::new();
    state.init_work(work_service).await;
    state
}

#[tokio::test]
async fn test_mcp_work_project_create() {
    let state = setup_tool_state().await;

    let request = WorkProjectRequest {
        action: "create".to_string(),
        name: Some("mcp-test-project".to_string()),
        description: Some("Test via MCP".to_string()),
        status: None,
        entity: None,
        relation: None,
    };

    let result = tools::work_project(&state, request).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("mcp-test-project"));
    assert!(response.contains("active")); // Default status
}

#[tokio::test]
async fn test_mcp_work_project_get() {
    let state = setup_tool_state().await;

    // First create a project
    let create_req = WorkProjectRequest {
        action: "create".to_string(),
        name: Some("get-test-project".to_string()),
        description: Some("Project to get".to_string()),
        status: None,
        entity: None,
        relation: None,
    };
    tools::work_project(&state, create_req).await.unwrap();

    // Now get it
    let get_req = WorkProjectRequest {
        action: "get".to_string(),
        name: Some("get-test-project".to_string()),
        description: None,
        status: None,
        entity: None,
        relation: None,
    };
    let result = tools::work_project(&state, get_req).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("get-test-project"));
    assert!(response.contains("task_count"));
    assert!(response.contains("pr_count"));
}

#[tokio::test]
async fn test_mcp_work_project_list() {
    let state = setup_tool_state().await;

    // Create some projects
    for name in ["proj-a", "proj-b", "proj-c"] {
        let req = WorkProjectRequest {
            action: "create".to_string(),
            name: Some(name.to_string()),
            description: None,
            status: None,
            entity: None,
            relation: None,
        };
        tools::work_project(&state, req).await.unwrap();
    }

    // List all
    let list_req = WorkProjectRequest {
        action: "list".to_string(),
        name: None,
        description: None,
        status: None,
        entity: None,
        relation: None,
    };
    let result = tools::work_project(&state, list_req).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("\"count\": 3"));
}

#[tokio::test]
async fn test_mcp_work_project_update() {
    let state = setup_tool_state().await;

    // Create project
    let create_req = WorkProjectRequest {
        action: "create".to_string(),
        name: Some("update-test-project".to_string()),
        description: None,
        status: None,
        entity: None,
        relation: None,
    };
    tools::work_project(&state, create_req).await.unwrap();

    // Update status
    let update_req = WorkProjectRequest {
        action: "update".to_string(),
        name: Some("update-test-project".to_string()),
        description: None,
        status: Some("completed".to_string()),
        entity: None,
        relation: None,
    };
    let result = tools::work_project(&state, update_req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("completed"));
}

#[tokio::test]
async fn test_mcp_work_project_delete() {
    let state = setup_tool_state().await;

    // Create project
    let create_req = WorkProjectRequest {
        action: "create".to_string(),
        name: Some("delete-test-project".to_string()),
        description: None,
        status: None,
        entity: None,
        relation: None,
    };
    tools::work_project(&state, create_req).await.unwrap();

    // Delete
    let delete_req = WorkProjectRequest {
        action: "delete".to_string(),
        name: Some("delete-test-project".to_string()),
        description: None,
        status: None,
        entity: None,
        relation: None,
    };
    let result = tools::work_project(&state, delete_req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("success"));

    // Verify it's gone
    let get_req = WorkProjectRequest {
        action: "get".to_string(),
        name: Some("delete-test-project".to_string()),
        description: None,
        status: None,
        entity: None,
        relation: None,
    };
    let result = tools::work_project(&state, get_req).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn test_mcp_work_project_invalid_action() {
    let state = setup_tool_state().await;

    let request = WorkProjectRequest {
        action: "invalid_action".to_string(),
        name: Some("test".to_string()),
        description: None,
        status: None,
        entity: None,
        relation: None,
    };

    let result = tools::work_project(&state, request).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown action"));
}

#[tokio::test]
async fn test_mcp_work_task_create_get_list() {
    let state = setup_tool_state().await;

    // Create project first
    let proj_req = WorkProjectRequest {
        action: "create".to_string(),
        name: Some("task-test-project".to_string()),
        description: None,
        status: None,
        entity: None,
        relation: None,
    };
    tools::work_project(&state, proj_req).await.unwrap();

    // Create task
    let create_req = WorkTaskRequest {
        action: "create".to_string(),
        project: Some("task-test-project".to_string()),
        name: Some("my-task".to_string()),
        description: Some("Test task".to_string()),
        jira_key: Some("TEST-123".to_string()),
        status: None,
        entity: None,
        relation: None,
    };
    let result = tools::work_task(&state, create_req).await;
    assert!(result.is_ok(), "Task create failed: {:?}", result.err());
    let create_response = result.unwrap();
    assert!(create_response.contains("my-task"));

    // Get task by JIRA key (tasks are looked up by JIRA key when no project specified)
    let get_req = WorkTaskRequest {
        action: "get".to_string(),
        project: None,
        name: Some("TEST-123".to_string()), // JIRA key
        description: None,
        jira_key: None,
        status: None,
        entity: None,
        relation: None,
    };
    let result = tools::work_task(&state, get_req).await;
    assert!(result.is_ok(), "Task get failed: {:?}", result.err());
    let get_response = result.unwrap();
    assert!(get_response.contains("my-task"));
    assert!(get_response.contains("TEST-123"));

    // List tasks
    let list_req = WorkTaskRequest {
        action: "list".to_string(),
        project: Some("task-test-project".to_string()),
        name: None,
        description: None,
        jira_key: None,
        status: None,
        entity: None,
        relation: None,
    };
    let result = tools::work_task(&state, list_req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("\"count\": 1"));
}

#[tokio::test]
async fn test_mcp_work_task_update_delete() {
    let state = setup_tool_state().await;

    // Setup: project and task
    let proj_req = WorkProjectRequest {
        action: "create".to_string(),
        name: Some("task-update-project".to_string()),
        description: None,
        status: None,
        entity: None,
        relation: None,
    };
    tools::work_project(&state, proj_req).await.unwrap();

    let task_req = WorkTaskRequest {
        action: "create".to_string(),
        project: Some("task-update-project".to_string()),
        name: Some("update-task".to_string()),
        description: None,
        jira_key: Some("UPDATE-1".to_string()), // Add JIRA key for lookup
        status: None,
        entity: None,
        relation: None,
    };
    tools::work_task(&state, task_req).await.unwrap();

    // Update task status (lookup by JIRA key)
    let update_req = WorkTaskRequest {
        action: "update".to_string(),
        project: None,
        name: Some("UPDATE-1".to_string()), // Use JIRA key
        description: None,
        jira_key: None,
        status: Some("done".to_string()),
        entity: None,
        relation: None,
    };
    let result = tools::work_task(&state, update_req).await;
    assert!(result.is_ok(), "Task update failed: {:?}", result.err());
    assert!(result.unwrap().contains("done"));

    // Delete task (lookup by JIRA key)
    let delete_req = WorkTaskRequest {
        action: "delete".to_string(),
        project: None,
        name: Some("UPDATE-1".to_string()), // Use JIRA key
        description: None,
        jira_key: None,
        status: None,
        entity: None,
        relation: None,
    };
    let result = tools::work_task(&state, delete_req).await;
    assert!(result.is_ok(), "Task delete failed: {:?}", result.err());
    assert!(result.unwrap().contains("success"));
}

#[tokio::test]
async fn test_mcp_work_pr_add_get_list_update_delete() {
    let state = setup_tool_state().await;

    // Setup: project
    let proj_req = WorkProjectRequest {
        action: "create".to_string(),
        name: Some("pr-test-project".to_string()),
        description: None,
        status: None,
        entity: None,
        relation: None,
    };
    tools::work_project(&state, proj_req).await.unwrap();

    // Add PR
    let add_req = WorkPrRequest {
        action: "add".to_string(),
        project: Some("pr-test-project".to_string()),
        url: Some("https://github.com/org/repo/pull/42".to_string()),
        task: None,
        title: Some("Fix bug".to_string()),
        status: None,
    };
    let result = tools::work_pr(&state, add_req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("42"));

    // Get PR
    let get_req = WorkPrRequest {
        action: "get".to_string(),
        project: None,
        url: Some("https://github.com/org/repo/pull/42".to_string()),
        task: None,
        title: None,
        status: None,
    };
    let result = tools::work_pr(&state, get_req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("open")); // Default status

    // List PRs
    let list_req = WorkPrRequest {
        action: "list".to_string(),
        project: Some("pr-test-project".to_string()),
        url: None,
        task: None,
        title: None,
        status: None,
    };
    let result = tools::work_pr(&state, list_req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("\"count\": 1"));

    // Update PR status
    let update_req = WorkPrRequest {
        action: "update".to_string(),
        project: None,
        url: Some("https://github.com/org/repo/pull/42".to_string()),
        task: None,
        title: None,
        status: Some("merged".to_string()),
    };
    let result = tools::work_pr(&state, update_req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("merged"));

    // Delete PR
    let delete_req = WorkPrRequest {
        action: "delete".to_string(),
        project: None,
        url: Some("https://github.com/org/repo/pull/42".to_string()),
        task: None,
        title: None,
        status: None,
    };
    let result = tools::work_pr(&state, delete_req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("success"));
}

#[tokio::test]
async fn test_mcp_work_observe_add_get_list_delete() {
    let state = setup_tool_state().await;

    // Setup: project
    let proj_req = WorkProjectRequest {
        action: "create".to_string(),
        name: Some("observe-test-project".to_string()),
        description: None,
        status: None,
        entity: None,
        relation: None,
    };
    tools::work_project(&state, proj_req).await.unwrap();

    // Add observation
    let add_req = WorkObserveRequest {
        action: "add".to_string(),
        project: Some("observe-test-project".to_string()),
        task: None,
        content: Some("This is a test observation".to_string()),
        key: Some("test.observation".to_string()),
        key_pattern: None,
        limit: None,
    };
    let result = tools::work_observe(&state, add_req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("test.observation"));

    // Get observation by key
    let get_req = WorkObserveRequest {
        action: "get".to_string(),
        project: Some("observe-test-project".to_string()),
        task: None,
        content: None,
        key: Some("test.observation".to_string()),
        key_pattern: None,
        limit: None,
    };
    let result = tools::work_observe(&state, get_req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("This is a test observation"));

    // Add another observation
    let add_req2 = WorkObserveRequest {
        action: "add".to_string(),
        project: Some("observe-test-project".to_string()),
        task: None,
        content: Some("Another test".to_string()),
        key: Some("test.another".to_string()),
        key_pattern: None,
        limit: None,
    };
    tools::work_observe(&state, add_req2).await.unwrap();

    // List observations with pattern
    let list_req = WorkObserveRequest {
        action: "list".to_string(),
        project: Some("observe-test-project".to_string()),
        task: None,
        content: None,
        key: None,
        key_pattern: Some("test.*".to_string()),
        limit: None,
    };
    let result = tools::work_observe(&state, list_req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("\"count\": 2"));

    // Delete observation
    let delete_req = WorkObserveRequest {
        action: "delete".to_string(),
        project: Some("observe-test-project".to_string()),
        task: None,
        content: None,
        key: Some("test.observation".to_string()),
        key_pattern: None,
        limit: None,
    };
    let result = tools::work_observe(&state, delete_req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("success"));
}

#[tokio::test]
async fn test_mcp_work_observe_task_scope() {
    let state = setup_tool_state().await;

    // Setup: project and task
    let proj_req = WorkProjectRequest {
        action: "create".to_string(),
        name: Some("task-observe-project".to_string()),
        description: None,
        status: None,
        entity: None,
        relation: None,
    };
    tools::work_project(&state, proj_req).await.unwrap();

    let task_req = WorkTaskRequest {
        action: "create".to_string(),
        project: Some("task-observe-project".to_string()),
        name: Some("my-observed-task".to_string()),
        description: None,
        jira_key: Some("OBS-1".to_string()), // Need JIRA key for task lookup
        status: None,
        entity: None,
        relation: None,
    };
    tools::work_task(&state, task_req).await.unwrap();

    // Add task observation (task scope takes precedence)
    let add_req = WorkObserveRequest {
        action: "add".to_string(),
        project: Some("task-observe-project".to_string()), // This is ignored when task is set
        task: Some("OBS-1".to_string()),                   // Use JIRA key for task lookup
        content: Some("Task-specific observation".to_string()),
        key: Some("task.note".to_string()),
        key_pattern: None,
        limit: None,
    };
    let result = tools::work_observe(&state, add_req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("task_id")); // Task observation has task_id

    // Get task observation
    let get_req = WorkObserveRequest {
        action: "get".to_string(),
        project: None,
        task: Some("OBS-1".to_string()), // Use JIRA key for task lookup
        content: None,
        key: Some("task.note".to_string()),
        key_pattern: None,
        limit: None,
    };
    let result = tools::work_observe(&state, get_req).await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("Task-specific observation"));
}

#[tokio::test]
async fn test_mcp_work_context_direct_lookup() {
    let state = setup_tool_state().await;

    // Setup: project with observations
    let proj_req = WorkProjectRequest {
        action: "create".to_string(),
        name: Some("context-test-project".to_string()),
        description: Some("For context test".to_string()),
        status: None,
        entity: None,
        relation: None,
    };
    tools::work_project(&state, proj_req).await.unwrap();

    let obs_req = WorkObserveRequest {
        action: "add".to_string(),
        project: Some("context-test-project".to_string()),
        task: None,
        content: Some("Context observation".to_string()),
        key: Some("ctx.obs".to_string()),
        key_pattern: None,
        limit: None,
    };
    tools::work_observe(&state, obs_req).await.unwrap();

    // Get full context via direct project lookup
    let ctx_req = WorkContextRequestNew {
        session_id: None,
        project: Some("context-test-project".to_string()),
        task: None,
    };
    let result = tools::work_context_new(&state, ctx_req).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("context-test-project"));
    assert!(response.contains("project_observations"));
    assert!(response.contains("Context observation"));
}

#[tokio::test]
async fn test_mcp_work_context_missing_params() {
    let state = setup_tool_state().await;

    // Neither session_id nor project provided
    let ctx_req = WorkContextRequestNew {
        session_id: None,
        project: None,
        task: None,
    };
    let result = tools::work_context_new(&state, ctx_req).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("must be specified"));
}

#[tokio::test]
async fn test_mcp_work_project_missing_required_params() {
    let state = setup_tool_state().await;

    // Create without name
    let req = WorkProjectRequest {
        action: "create".to_string(),
        name: None, // Missing required field
        description: None,
        status: None,
        entity: None,
        relation: None,
    };
    let result = tools::work_project(&state, req).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("required"));

    // Update without status
    let proj_req = WorkProjectRequest {
        action: "create".to_string(),
        name: Some("missing-params-project".to_string()),
        description: None,
        status: None,
        entity: None,
        relation: None,
    };
    tools::work_project(&state, proj_req).await.unwrap();

    let update_req = WorkProjectRequest {
        action: "update".to_string(),
        name: Some("missing-params-project".to_string()),
        description: None,
        status: None, // Missing required field
        entity: None,
        relation: None,
    };
    let result = tools::work_project(&state, update_req).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("required"));
}
