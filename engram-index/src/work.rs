//! Work service for Layer 7: Work Management.
//!
//! Provides business logic for managing projects, tasks, PRs,
//! and work-scoped observations with entity connections.

use crate::entity::EntityService;
use crate::error::{IndexError, IndexResult};
use engram_core::entity::{Entity, Observation};
use engram_core::id::Id;
use engram_core::work::{
    Pr, PrStatus, Project, ProjectEntityRelation, ProjectObservation, ProjectStatus, Task,
    TaskEntityRelation, TaskObservation, TaskPriority, TaskStatus, WorkContext,
};
use engram_embed::Embedder;
use engram_store::{Db, EntityRepo, WorkRepo, WorkStats};
use time::OffsetDateTime;
use tracing::{debug, info};

/// Source for observation graduation.
#[derive(Debug, Clone, Copy)]
pub enum GraduateFrom {
    /// Graduate from a project observation.
    Project,
    /// Graduate from a task observation.
    Task,
}

/// Full work context for agent startup.
#[derive(Debug, Clone)]
pub struct FullWorkContext {
    /// The project.
    pub project: Project,
    /// The current task (if any).
    pub task: Option<Task>,
    /// All PRs for this project (or task if specified).
    pub prs: Vec<Pr>,
    /// Project-level observations.
    pub project_observations: Vec<ProjectObservation>,
    /// Task-level observations (if task specified).
    pub task_observations: Vec<TaskObservation>,
    /// Entities connected to the project/task.
    pub connected_entities: Vec<Entity>,
    /// Observations from connected entities (global knowledge).
    pub entity_observations: Vec<Observation>,
}

/// Service for work management.
#[derive(Clone)]
pub struct WorkService {
    repo: WorkRepo,
    entity_repo: EntityRepo,
    embedder: Option<Embedder>,
}

impl WorkService {
    /// Create a new work service without embedding support.
    pub fn new(db: Db) -> Self {
        Self {
            repo: WorkRepo::new(db.clone()),
            entity_repo: EntityRepo::new(db),
            embedder: None,
        }
    }

    /// Create a new work service with embedding support.
    pub fn with_embedder(db: Db, embedder: Embedder) -> Self {
        Self {
            repo: WorkRepo::new(db.clone()),
            entity_repo: EntityRepo::new(db),
            embedder: Some(embedder),
        }
    }

    /// Create a new work service with the default embedder.
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

    /// Initialize the work schema.
    pub async fn init(&self) -> IndexResult<()> {
        self.repo.init_schema().await?;
        Ok(())
    }

    // =========================================================================
    // Project Operations
    // =========================================================================

    /// Create a new project.
    pub async fn create_project(
        &self,
        name: &str,
        description: Option<&str>,
    ) -> IndexResult<Project> {
        info!("Creating project: {}", name);

        // Check if project with this name already exists
        if let Some(existing) = self.repo.get_project_by_name(name).await? {
            return Err(IndexError::AlreadyExists(format!(
                "Project with name '{}' already exists (id: {})",
                name, existing.id
            )));
        }

        let mut project = Project::new(name);
        if let Some(desc) = description {
            project = project.with_description(desc);
        }

        self.repo.create_project(&project).await?;

        info!("Created project: {} ({})", project.id, project.name);
        Ok(project)
    }

    /// Get a project by name.
    pub async fn get_project(&self, name: &str) -> IndexResult<Option<Project>> {
        Ok(self.repo.get_project_by_name(name).await?)
    }

    /// Get a project by ID.
    pub async fn get_project_by_id(&self, id: &Id) -> IndexResult<Option<Project>> {
        Ok(self.repo.get_project(id).await?)
    }

    /// List projects, optionally filtered by status.
    pub async fn list_projects(&self, status: Option<ProjectStatus>) -> IndexResult<Vec<Project>> {
        Ok(self.repo.list_projects(status).await?)
    }

    /// Update project status.
    pub async fn update_project_status(
        &self,
        name: &str,
        status: ProjectStatus,
    ) -> IndexResult<Project> {
        let mut project = self
            .repo
            .get_project_by_name(name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Project not found: {}", name)))?;

        project.status = status;
        project.updated_at = OffsetDateTime::now_utc();
        self.repo.update_project(&project).await?;

        info!("Updated project '{}' status to {}", name, status);
        Ok(project)
    }

    /// Delete a project and all related data.
    pub async fn delete_project(&self, name: &str) -> IndexResult<()> {
        let project = self
            .repo
            .get_project_by_name(name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Project not found: {}", name)))?;

        self.repo.delete_project(&project.id).await?;
        info!("Deleted project: {}", name);
        Ok(())
    }

    // =========================================================================
    // Task Operations
    // =========================================================================

    /// Create a new task.
    pub async fn create_task(
        &self,
        project_name: &str,
        task_name: &str,
        description: Option<&str>,
        jira_key: Option<&str>,
    ) -> IndexResult<Task> {
        info!("Creating task: {} (project: {})", task_name, project_name);

        let project = self
            .repo
            .get_project_by_name(project_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Project not found: {}", project_name)))?;

        // Check if task with this name already exists in project
        if let Some(existing) = self.repo.get_task_by_name(&project.id, task_name).await? {
            return Err(IndexError::AlreadyExists(format!(
                "Task with name '{}' already exists in project '{}' (id: {})",
                task_name, project_name, existing.id
            )));
        }

        // Check for duplicate JIRA key
        if let Some(key) = jira_key {
            if let Some(existing) = self.repo.get_task_by_jira(key).await? {
                return Err(IndexError::AlreadyExists(format!(
                    "Task with JIRA key '{}' already exists (id: {})",
                    key, existing.id
                )));
            }
        }

        let mut task = Task::new(project.id, task_name);
        if let Some(desc) = description {
            task = task.with_description(desc);
        }
        if let Some(key) = jira_key {
            task = task.with_jira_key(key);
        }

        self.repo.create_task(&task).await?;

        info!(
            "Created task: {} ({}) in project {}",
            task.id, task.name, project_name
        );
        Ok(task)
    }

    /// Get a task by ID, name, or JIRA key.
    pub async fn get_task(&self, id_name_or_jira: &str) -> IndexResult<Option<Task>> {
        // Try ID lookup first (UUID format)
        if let Ok(id) = Id::parse(id_name_or_jira) {
            if let Some(task) = self.repo.get_task(&id).await? {
                return Ok(Some(task));
            }
        }

        // Try JIRA key (format: ABC-123)
        if id_name_or_jira.contains('-') {
            if let Some(task) = self.repo.get_task_by_jira(id_name_or_jira).await? {
                return Ok(Some(task));
            }
        }

        // Fall back to searching by JIRA key
        // This is a limitation - we'd need project context to search by name
        Ok(self.repo.get_task_by_jira(id_name_or_jira).await?)
    }

    /// Get a task by name within a specific project.
    pub async fn get_task_in_project(
        &self,
        project_name: &str,
        task_name: &str,
    ) -> IndexResult<Option<Task>> {
        let project = self.repo.get_project_by_name(project_name).await?;
        if let Some(p) = project {
            Ok(self.repo.get_task_by_name(&p.id, task_name).await?)
        } else {
            Ok(None)
        }
    }

    /// Get a task by ID.
    pub async fn get_task_by_id(&self, id: &Id) -> IndexResult<Option<Task>> {
        Ok(self.repo.get_task(id).await?)
    }

    /// List tasks for a project, optionally filtered by status.
    pub async fn list_tasks(
        &self,
        project_name: &str,
        status: Option<TaskStatus>,
    ) -> IndexResult<Vec<Task>> {
        let project = self
            .repo
            .get_project_by_name(project_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Project not found: {}", project_name)))?;

        Ok(self.repo.list_tasks(&project.id, status).await?)
    }

    /// Update task status.
    pub async fn update_task_status(
        &self,
        name_or_jira: &str,
        status: TaskStatus,
    ) -> IndexResult<Task> {
        let mut task = self
            .get_task(name_or_jira)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Task not found: {}", name_or_jira)))?;

        task.status = status;
        task.updated_at = OffsetDateTime::now_utc();
        self.repo.update_task(&task).await?;

        info!("Updated task '{}' status to {}", name_or_jira, status);
        Ok(task)
    }

    /// Update task priority.
    pub async fn update_task_priority(
        &self,
        name_or_jira: &str,
        priority: TaskPriority,
    ) -> IndexResult<Task> {
        let mut task = self
            .get_task(name_or_jira)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Task not found: {}", name_or_jira)))?;

        task.priority = priority;
        task.updated_at = OffsetDateTime::now_utc();
        self.repo.update_task(&task).await?;

        info!("Updated task '{}' priority to {}", name_or_jira, priority);
        Ok(task)
    }

    /// Set tasks that block this task.
    pub async fn set_task_blocked_by(
        &self,
        task_name_or_jira: &str,
        blocked_by: &[&str],
    ) -> IndexResult<Task> {
        let mut task = self.get_task(task_name_or_jira).await?.ok_or_else(|| {
            IndexError::NotFound(format!("Task not found: {}", task_name_or_jira))
        })?;

        // Resolve blocking task names/jira keys to IDs
        let mut blocking_ids = Vec::new();
        for name in blocked_by {
            let blocking_task = self.get_task(name).await?.ok_or_else(|| {
                IndexError::NotFound(format!("Blocking task not found: {}", name))
            })?;
            blocking_ids.push(blocking_task.id);
        }

        task.blocked_by = blocking_ids;
        task.updated_at = OffsetDateTime::now_utc();
        self.repo.update_task(&task).await?;

        info!("Set blocked_by for task '{}'", task_name_or_jira);
        Ok(task)
    }

    /// Delete a task.
    pub async fn delete_task(&self, name_or_jira: &str) -> IndexResult<()> {
        let task = self
            .get_task(name_or_jira)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Task not found: {}", name_or_jira)))?;

        self.repo.delete_task(&task.id).await?;
        info!("Deleted task: {}", name_or_jira);
        Ok(())
    }

    // =========================================================================
    // PR Operations
    // =========================================================================

    /// Add a PR to a project (optionally linked to a task).
    pub async fn add_pr(
        &self,
        project_name: &str,
        task_name: Option<&str>,
        url: &str,
        title: Option<&str>,
    ) -> IndexResult<Pr> {
        info!("Adding PR: {} (project: {})", url, project_name);

        let project = self
            .repo
            .get_project_by_name(project_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Project not found: {}", project_name)))?;

        // Check if PR with this URL already exists
        if let Some(existing) = self.repo.get_pr_by_url(url).await? {
            return Err(IndexError::AlreadyExists(format!(
                "PR with URL '{}' already exists (id: {})",
                url, existing.id
            )));
        }

        // Parse repo and PR number from URL
        let (repo, pr_number) = parse_pr_url(url)?;

        let mut pr = Pr::new(project.id, url, repo, pr_number);
        if let Some(t) = title {
            pr = pr.with_title(t);
        }

        // Link to task if specified
        if let Some(task_ref) = task_name {
            let task = self
                .get_task(task_ref)
                .await?
                .ok_or_else(|| IndexError::NotFound(format!("Task not found: {}", task_ref)))?;
            pr = pr.with_task(task.id);
        }

        self.repo.create_pr(&pr).await?;

        info!("Added PR: {} ({})", pr.id, pr.url);
        Ok(pr)
    }

    /// Get a PR by URL.
    pub async fn get_pr(&self, url: &str) -> IndexResult<Option<Pr>> {
        Ok(self.repo.get_pr_by_url(url).await?)
    }

    /// List PRs for a project (optionally filtered by task).
    pub async fn list_prs(
        &self,
        project_name: &str,
        task_name: Option<&str>,
    ) -> IndexResult<Vec<Pr>> {
        let project = self
            .repo
            .get_project_by_name(project_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Project not found: {}", project_name)))?;

        let task_id = if let Some(task_ref) = task_name {
            Some(
                self.get_task(task_ref)
                    .await?
                    .ok_or_else(|| IndexError::NotFound(format!("Task not found: {}", task_ref)))?
                    .id,
            )
        } else {
            None
        };

        Ok(self
            .repo
            .list_prs(Some(&project.id), task_id.as_ref())
            .await?)
    }

    /// Update PR status.
    pub async fn update_pr_status(&self, url: &str, status: PrStatus) -> IndexResult<Pr> {
        let mut pr = self
            .repo
            .get_pr_by_url(url)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("PR not found: {}", url)))?;

        pr.status = status;
        pr.updated_at = OffsetDateTime::now_utc();
        self.repo.update_pr(&pr).await?;

        info!("Updated PR '{}' status to {}", url, status);
        Ok(pr)
    }

    /// Set PRs that block this PR.
    pub async fn set_pr_blocked_by(
        &self,
        pr_url: &str,
        blocked_by_urls: &[&str],
    ) -> IndexResult<Pr> {
        let mut pr = self
            .repo
            .get_pr_by_url(pr_url)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("PR not found: {}", pr_url)))?;

        // Resolve blocking PR URLs to IDs
        let mut blocking_ids = Vec::new();
        for url in blocked_by_urls {
            let blocking_pr =
                self.repo.get_pr_by_url(url).await?.ok_or_else(|| {
                    IndexError::NotFound(format!("Blocking PR not found: {}", url))
                })?;
            blocking_ids.push(blocking_pr.id);
        }

        pr.blocked_by = blocking_ids;
        pr.updated_at = OffsetDateTime::now_utc();
        self.repo.update_pr(&pr).await?;

        info!("Set blocked_by for PR '{}'", pr_url);
        Ok(pr)
    }

    /// Delete a PR by URL.
    pub async fn delete_pr(&self, url: &str) -> IndexResult<()> {
        let pr = self
            .repo
            .get_pr_by_url(url)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("PR not found: {}", url)))?;

        self.repo.delete_pr(&pr.id).await?;
        info!("Deleted PR: {}", url);
        Ok(())
    }

    // =========================================================================
    // Entity Connection Operations
    // =========================================================================

    /// Connect a project to an entity.
    pub async fn connect_project_to_entity(
        &self,
        project_name: &str,
        entity_name: &str,
        relation: Option<&str>,
    ) -> IndexResult<()> {
        let project = self
            .repo
            .get_project_by_name(project_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Project not found: {}", project_name)))?;

        let entity = self
            .entity_repo
            .get_entity_by_name(entity_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Entity not found: {}", entity_name)))?;

        let rel = relation
            .map(ProjectEntityRelation::parse)
            .unwrap_or_default();

        self.repo
            .connect_project_entity(&project.id, &entity.id, &rel)
            .await?;

        info!(
            "Connected project '{}' to entity '{}' ({})",
            project_name, entity_name, rel
        );
        Ok(())
    }

    /// Connect a task to an entity.
    pub async fn connect_task_to_entity(
        &self,
        task_name_or_jira: &str,
        entity_name: &str,
        relation: Option<&str>,
    ) -> IndexResult<()> {
        let task = self.get_task(task_name_or_jira).await?.ok_or_else(|| {
            IndexError::NotFound(format!("Task not found: {}", task_name_or_jira))
        })?;

        let entity = self
            .entity_repo
            .get_entity_by_name(entity_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Entity not found: {}", entity_name)))?;

        let rel = relation.map(TaskEntityRelation::parse).unwrap_or_default();

        self.repo
            .connect_task_entity(&task.id, &entity.id, &rel)
            .await?;

        info!(
            "Connected task '{}' to entity '{}' ({})",
            task_name_or_jira, entity_name, rel
        );
        Ok(())
    }

    /// Get entities connected to a project.
    pub async fn get_project_entities(&self, project_name: &str) -> IndexResult<Vec<Entity>> {
        let project = self
            .repo
            .get_project_by_name(project_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Project not found: {}", project_name)))?;

        let connections = self.repo.get_project_entities(&project.id).await?;

        let mut entities = Vec::new();
        for (entity_id, _relation) in connections {
            if let Some(entity) = self.entity_repo.get_entity(&entity_id).await? {
                entities.push(entity);
            }
        }

        Ok(entities)
    }

    /// Get entities connected to a task.
    pub async fn get_task_entities(&self, task_name_or_jira: &str) -> IndexResult<Vec<Entity>> {
        let task = self.get_task(task_name_or_jira).await?.ok_or_else(|| {
            IndexError::NotFound(format!("Task not found: {}", task_name_or_jira))
        })?;

        let connections = self.repo.get_task_entities(&task.id).await?;

        let mut entities = Vec::new();
        for (entity_id, _relation) in connections {
            if let Some(entity) = self.entity_repo.get_entity(&entity_id).await? {
                entities.push(entity);
            }
        }

        Ok(entities)
    }

    /// Disconnect a project from an entity.
    pub async fn disconnect_project_from_entity(
        &self,
        project_name: &str,
        entity_name: &str,
    ) -> IndexResult<()> {
        let project = self
            .repo
            .get_project_by_name(project_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Project not found: {}", project_name)))?;

        let entity = self
            .entity_repo
            .get_entity_by_name(entity_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Entity not found: {}", entity_name)))?;

        self.repo
            .disconnect_project_entity(&project.id, &entity.id)
            .await?;

        info!(
            "Disconnected project '{}' from entity '{}'",
            project_name, entity_name
        );
        Ok(())
    }

    /// Disconnect a task from an entity.
    pub async fn disconnect_task_from_entity(
        &self,
        task_name_or_jira: &str,
        entity_name: &str,
    ) -> IndexResult<()> {
        let task = self.get_task(task_name_or_jira).await?.ok_or_else(|| {
            IndexError::NotFound(format!("Task not found: {}", task_name_or_jira))
        })?;

        let entity = self
            .entity_repo
            .get_entity_by_name(entity_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Entity not found: {}", entity_name)))?;

        self.repo
            .disconnect_task_entity(&task.id, &entity.id)
            .await?;

        info!(
            "Disconnected task '{}' from entity '{}'",
            task_name_or_jira, entity_name
        );
        Ok(())
    }

    // =========================================================================
    // Observation Operations
    // =========================================================================

    /// Add or update a project observation.
    pub async fn add_project_observation(
        &self,
        project_name: &str,
        content: &str,
        key: Option<&str>,
    ) -> IndexResult<ProjectObservation> {
        let project = self
            .repo
            .get_project_by_name(project_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Project not found: {}", project_name)))?;

        let mut obs = ProjectObservation::new(project.id, content);
        if let Some(k) = key {
            obs = obs.with_key(k);
        }

        // Generate embedding
        if let Some(embedding) = self.generate_embedding(content) {
            obs = obs.with_embedding(embedding);
            debug!("Generated embedding for project observation");
        }

        self.repo.add_project_observation(&obs).await?;

        info!(
            "Added project observation for '{}' (key: {:?})",
            project_name, key
        );
        Ok(obs)
    }

    /// Add or update a task observation.
    pub async fn add_task_observation(
        &self,
        task_name_or_jira: &str,
        content: &str,
        key: Option<&str>,
    ) -> IndexResult<TaskObservation> {
        let task = self.get_task(task_name_or_jira).await?.ok_or_else(|| {
            IndexError::NotFound(format!("Task not found: {}", task_name_or_jira))
        })?;

        let mut obs = TaskObservation::new(task.id, content);
        if let Some(k) = key {
            obs = obs.with_key(k);
        }

        // Generate embedding
        if let Some(embedding) = self.generate_embedding(content) {
            obs = obs.with_embedding(embedding);
            debug!("Generated embedding for task observation");
        }

        self.repo.add_task_observation(&obs).await?;

        info!(
            "Added task observation for '{}' (key: {:?})",
            task_name_or_jira, key
        );
        Ok(obs)
    }

    /// Get observations for a project.
    pub async fn get_project_observations(
        &self,
        project_name: &str,
    ) -> IndexResult<Vec<ProjectObservation>> {
        let project = self
            .repo
            .get_project_by_name(project_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Project not found: {}", project_name)))?;

        Ok(self.repo.get_project_observations(&project.id).await?)
    }

    /// Get observations for a task.
    pub async fn get_task_observations(
        &self,
        task_name_or_jira: &str,
    ) -> IndexResult<Vec<TaskObservation>> {
        let task = self.get_task(task_name_or_jira).await?.ok_or_else(|| {
            IndexError::NotFound(format!("Task not found: {}", task_name_or_jira))
        })?;

        Ok(self.repo.get_task_observations(&task.id).await?)
    }

    /// Get a project observation by key.
    pub async fn get_project_observation_by_key(
        &self,
        project_name: &str,
        key: &str,
    ) -> IndexResult<Option<ProjectObservation>> {
        let project = self
            .repo
            .get_project_by_name(project_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Project not found: {}", project_name)))?;

        Ok(self
            .repo
            .get_project_observation_by_key(&project.id, key)
            .await?)
    }

    /// Get a task observation by key.
    pub async fn get_task_observation_by_key(
        &self,
        task_name_or_jira: &str,
        key: &str,
    ) -> IndexResult<Option<TaskObservation>> {
        let task = self.get_task(task_name_or_jira).await?.ok_or_else(|| {
            IndexError::NotFound(format!("Task not found: {}", task_name_or_jira))
        })?;

        Ok(self.repo.get_task_observation_by_key(&task.id, key).await?)
    }

    /// Delete a project observation by key.
    pub async fn delete_project_observation_by_key(
        &self,
        project_name: &str,
        key: &str,
    ) -> IndexResult<()> {
        let project = self
            .repo
            .get_project_by_name(project_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Project not found: {}", project_name)))?;

        let obs = self
            .repo
            .get_project_observation_by_key(&project.id, key)
            .await?
            .ok_or_else(|| {
                IndexError::NotFound(format!(
                    "Project observation not found: {} (key: {})",
                    project_name, key
                ))
            })?;

        self.repo.delete_project_observation(&obs.id).await?;

        info!(
            "Deleted project observation for '{}' (key: {})",
            project_name, key
        );
        Ok(())
    }

    /// Delete a task observation by key.
    pub async fn delete_task_observation_by_key(
        &self,
        task_name_or_jira: &str,
        key: &str,
    ) -> IndexResult<()> {
        let task = self.get_task(task_name_or_jira).await?.ok_or_else(|| {
            IndexError::NotFound(format!("Task not found: {}", task_name_or_jira))
        })?;

        let obs = self
            .repo
            .get_task_observation_by_key(&task.id, key)
            .await?
            .ok_or_else(|| {
                IndexError::NotFound(format!(
                    "Task observation not found: {} (key: {})",
                    task_name_or_jira, key
                ))
            })?;

        self.repo.delete_task_observation(&obs.id).await?;

        info!(
            "Deleted task observation for '{}' (key: {})",
            task_name_or_jira, key
        );
        Ok(())
    }

    /// Graduate an observation from project/task scope to entity scope.
    /// This copies the observation content to the target entity as a global observation.
    ///
    /// Note: Currently not implemented - requires observation lookup by ID which needs
    /// to be added to the repo layer. For now, users should manually copy observations.
    pub async fn graduate_observation(
        &self,
        _from: GraduateFrom,
        _observation_id: &Id,
        _to_entity: &str,
        _key: Option<&str>,
        _entity_service: &EntityService,
    ) -> IndexResult<Observation> {
        // TODO: Implement observation lookup by ID in repo layer
        // For now, return NotFound as this feature is not yet complete
        Err(IndexError::NotFound(
            "Observation graduation not yet implemented - use manual copy".to_string(),
        ))
    }

    // =========================================================================
    // Work Context Operations
    // =========================================================================

    /// Set session work context (join a project/task).
    pub async fn join_work(
        &self,
        session_id: &Id,
        project_name: &str,
        task_name: Option<&str>,
    ) -> IndexResult<WorkContext> {
        let project = self
            .repo
            .get_project_by_name(project_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Project not found: {}", project_name)))?;

        let task_id = if let Some(task_ref) = task_name {
            Some(
                self.get_task(task_ref)
                    .await?
                    .ok_or_else(|| IndexError::NotFound(format!("Task not found: {}", task_ref)))?
                    .id,
            )
        } else {
            None
        };

        let mut final_ctx = WorkContext::new(session_id.clone()).with_project(project.id);
        if let Some(tid) = task_id {
            final_ctx = final_ctx.with_task(tid);
        }

        self.repo.set_session_context(&final_ctx).await?;

        info!(
            "Session {} joined project '{}' (task: {:?})",
            session_id, project_name, task_name
        );
        Ok(final_ctx)
    }

    /// Clear session work context (leave work).
    pub async fn leave_work(&self, session_id: &Id) -> IndexResult<()> {
        self.repo.clear_session_context(session_id).await?;
        info!("Session {} left work context", session_id);
        Ok(())
    }

    /// Get session work context.
    pub async fn get_work_context(&self, session_id: &Id) -> IndexResult<Option<WorkContext>> {
        Ok(self.repo.get_session_context(session_id).await?)
    }

    // =========================================================================
    // Aggregated Context
    // =========================================================================

    /// Get full work context for agent startup.
    pub async fn get_full_context(
        &self,
        project_name: &str,
        task_name: Option<&str>,
    ) -> IndexResult<FullWorkContext> {
        let project = self
            .repo
            .get_project_by_name(project_name)
            .await?
            .ok_or_else(|| IndexError::NotFound(format!("Project not found: {}", project_name)))?;

        let task = if let Some(task_ref) = task_name {
            Some(
                self.get_task(task_ref)
                    .await?
                    .ok_or_else(|| IndexError::NotFound(format!("Task not found: {}", task_ref)))?,
            )
        } else {
            None
        };

        // Get PRs
        let prs = self
            .repo
            .list_prs(Some(&project.id), task.as_ref().map(|t| &t.id))
            .await?;

        // Get project observations
        let project_observations = self.repo.get_project_observations(&project.id).await?;

        // Get task observations (if task specified)
        let task_observations = if let Some(ref t) = task {
            self.repo.get_task_observations(&t.id).await?
        } else {
            Vec::new()
        };

        // Get connected entities (from both project and task)
        let mut entity_ids = Vec::new();
        let project_entities = self.repo.get_project_entities(&project.id).await?;
        for (entity_id, _) in project_entities {
            if !entity_ids.contains(&entity_id) {
                entity_ids.push(entity_id);
            }
        }
        if let Some(ref t) = task {
            let task_entities = self.repo.get_task_entities(&t.id).await?;
            for (entity_id, _) in task_entities {
                if !entity_ids.contains(&entity_id) {
                    entity_ids.push(entity_id);
                }
            }
        }

        // Fetch entities and their observations
        let mut connected_entities = Vec::new();
        let mut entity_observations = Vec::new();
        for entity_id in entity_ids {
            if let Some(entity) = self.entity_repo.get_entity(&entity_id).await? {
                connected_entities.push(entity);
                let obs = self.entity_repo.get_observations(&entity_id).await?;
                entity_observations.extend(obs);
            }
        }

        Ok(FullWorkContext {
            project,
            task,
            prs,
            project_observations,
            task_observations,
            connected_entities,
            entity_observations,
        })
    }

    // =========================================================================
    // Statistics
    // =========================================================================

    /// Get work statistics.
    pub async fn stats(&self) -> IndexResult<WorkStats> {
        Ok(self.repo.stats().await?)
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Parse a PR URL to extract repo name and PR number.
/// Supports GitHub URLs like:
/// - https://github.com/org/repo/pull/123
/// - github.com/org/repo/pull/123
fn parse_pr_url(url: &str) -> IndexResult<(String, u32)> {
    // Try to parse common GitHub URL formats
    let url_lower = url.to_lowercase();

    // Remove protocol if present
    let path = url_lower
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("www.");

    // Expected format: github.com/org/repo/pull/123
    if !path.starts_with("github.com/") {
        return Err(IndexError::Parse(format!(
            "Unsupported PR URL format: {}",
            url
        )));
    }

    let parts: Vec<&str> = path.split('/').collect();
    // parts: ["github.com", "org", "repo", "pull", "123"]
    if parts.len() < 5 || parts[3] != "pull" {
        return Err(IndexError::Parse(format!(
            "Invalid GitHub PR URL format: {}",
            url
        )));
    }

    let repo = parts[2].to_string();
    let pr_number: u32 = parts[4]
        .parse()
        .map_err(|_| IndexError::Parse(format!("Invalid PR number in URL: {}", url)))?;

    Ok((repo, pr_number))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pr_url() {
        let (repo, num) = parse_pr_url("https://github.com/org/my-repo/pull/123").unwrap();
        assert_eq!(repo, "my-repo");
        assert_eq!(num, 123);

        let (repo, num) = parse_pr_url("github.com/org/repo/pull/456").unwrap();
        assert_eq!(repo, "repo");
        assert_eq!(num, 456);
    }

    #[test]
    fn test_parse_pr_url_invalid() {
        assert!(parse_pr_url("https://gitlab.com/org/repo/pull/123").is_err());
        assert!(parse_pr_url("not-a-url").is_err());
    }
}
