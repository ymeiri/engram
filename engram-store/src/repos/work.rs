//! Work repository for Layer 7: Work Management.
//!
//! Handles persistence of Projects, Tasks, PRs, and Work Observations.
//! Provides entity connections via junction tables.

use crate::error::{StoreError, StoreResult};
use crate::Db;
use engram_core::id::Id;
use engram_core::work::{
    Pr, PrStatus, Project, ProjectEntityRelation, ProjectObservation, ProjectStatus, Task,
    TaskEntityRelation, TaskObservation, TaskPriority, TaskStatus, WorkContext,
};
use serde::Deserialize;
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

// ============================================================================
// Record Types
// ============================================================================

#[derive(Debug, Deserialize)]
struct ProjectRecord {
    name: String,
    description: Option<String>,
    status: String,
    created_at: SurrealDateTime,
    updated_at: SurrealDateTime,
}

#[derive(Debug, Deserialize)]
struct ProjectRecordWithId {
    id: String,
    name: String,
    description: Option<String>,
    status: String,
    created_at: SurrealDateTime,
    updated_at: SurrealDateTime,
}

#[derive(Debug, Deserialize)]
struct TaskRecord {
    project_id: String,
    name: String,
    description: Option<String>,
    status: String,
    priority: String,
    jira_key: Option<String>,
    #[serde(default)]
    blocked_by: Vec<String>,
    created_at: SurrealDateTime,
    updated_at: SurrealDateTime,
}

#[derive(Debug, Deserialize)]
struct TaskRecordWithId {
    id: String,
    project_id: String,
    name: String,
    description: Option<String>,
    status: String,
    priority: String,
    jira_key: Option<String>,
    #[serde(default)]
    blocked_by: Vec<String>,
    created_at: SurrealDateTime,
    updated_at: SurrealDateTime,
}

#[derive(Debug, Deserialize)]
struct PrRecord {
    task_id: Option<String>,
    project_id: String,
    url: String,
    repo: String,
    pr_number: u32,
    title: Option<String>,
    status: String,
    #[serde(default)]
    blocked_by: Vec<String>,
    created_at: SurrealDateTime,
    updated_at: SurrealDateTime,
}

#[derive(Debug, Deserialize)]
struct PrRecordWithId {
    id: String,
    task_id: Option<String>,
    project_id: String,
    url: String,
    repo: String,
    pr_number: u32,
    title: Option<String>,
    status: String,
    #[serde(default)]
    blocked_by: Vec<String>,
    created_at: SurrealDateTime,
    updated_at: SurrealDateTime,
}

#[derive(Debug, Deserialize)]
struct ProjectObservationRecordWithId {
    id: String,
    project_id: String,
    key: Option<String>,
    content: String,
    #[serde(default)]
    embedding: Option<Vec<f32>>,
    source: Option<String>,
    created_at: SurrealDateTime,
    updated_at: SurrealDateTime,
}

#[derive(Debug, Deserialize)]
struct TaskObservationRecordWithId {
    id: String,
    task_id: String,
    key: Option<String>,
    content: String,
    #[serde(default)]
    embedding: Option<Vec<f32>>,
    source: Option<String>,
    created_at: SurrealDateTime,
    updated_at: SurrealDateTime,
}

#[derive(Debug, Deserialize)]
struct EntityConnectionRecord {
    entity_id: String,
    relation: String,
}

#[derive(Debug, Deserialize)]
struct WorkContextRecord {
    session_id: String,
    project_id: Option<String>,
    task_id: Option<String>,
    joined_at: SurrealDateTime,
}

#[derive(Debug, Deserialize)]
struct CountResult {
    count: u64,
}

// ============================================================================
// Search Results
// ============================================================================

/// Result of a vector similarity search for project observations.
#[derive(Debug, Clone)]
pub struct ProjectObservationSearchResult {
    /// The matching observation.
    pub observation: ProjectObservation,
    /// Cosine similarity score (0.0 to 1.0).
    pub score: f32,
}

/// Result of a vector similarity search for task observations.
#[derive(Debug, Clone)]
pub struct TaskObservationSearchResult {
    /// The matching observation.
    pub observation: TaskObservation,
    /// Cosine similarity score (0.0 to 1.0).
    pub score: f32,
}

/// Statistics for work management.
#[derive(Debug, Clone, Default)]
pub struct WorkStats {
    pub project_count: u64,
    pub task_count: u64,
    pub pr_count: u64,
    pub project_observation_count: u64,
    pub task_observation_count: u64,
}

// ============================================================================
// Table Names
// ============================================================================

const TABLE_PROJECT: &str = "work_project";
const TABLE_TASK: &str = "work_task";
const TABLE_PR: &str = "work_pr";
const TABLE_PROJECT_ENTITY: &str = "work_project_entity";
const TABLE_TASK_ENTITY: &str = "work_task_entity";
const TABLE_PROJECT_OBSERVATION: &str = "work_project_observation";
const TABLE_TASK_OBSERVATION: &str = "work_task_observation";
const TABLE_SESSION_CONTEXT: &str = "work_session_context";

// ============================================================================
// Helper Functions
// ============================================================================

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

// ============================================================================
// WorkRepo
// ============================================================================

/// Repository for work management operations.
#[derive(Clone)]
pub struct WorkRepo {
    db: Db,
}

impl WorkRepo {
    /// Create a new work repository.
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Initialize the work schema.
    pub async fn init_schema(&self) -> StoreResult<()> {
        info!("Initializing work schema (Layer 7)");

        // Project table
        self.db
            .query(format!(
                r#"
                DEFINE TABLE IF NOT EXISTS {TABLE_PROJECT} SCHEMALESS;
                DEFINE INDEX IF NOT EXISTS idx_work_project_name ON {TABLE_PROJECT} FIELDS name UNIQUE;
                DEFINE INDEX IF NOT EXISTS idx_work_project_status ON {TABLE_PROJECT} FIELDS status;
                "#
            ))
            .await?;

        // Task table
        self.db
            .query(format!(
                r#"
                DEFINE TABLE IF NOT EXISTS {TABLE_TASK} SCHEMALESS;
                DEFINE INDEX IF NOT EXISTS idx_work_task_project ON {TABLE_TASK} FIELDS project_id;
                DEFINE INDEX IF NOT EXISTS idx_work_task_status ON {TABLE_TASK} FIELDS status;
                DEFINE INDEX IF NOT EXISTS idx_work_task_jira ON {TABLE_TASK} FIELDS jira_key;
                "#
            ))
            .await?;

        // PR table
        self.db
            .query(format!(
                r#"
                DEFINE TABLE IF NOT EXISTS {TABLE_PR} SCHEMALESS;
                DEFINE INDEX IF NOT EXISTS idx_work_pr_task ON {TABLE_PR} FIELDS task_id;
                DEFINE INDEX IF NOT EXISTS idx_work_pr_project ON {TABLE_PR} FIELDS project_id;
                DEFINE INDEX IF NOT EXISTS idx_work_pr_repo ON {TABLE_PR} FIELDS repo;
                DEFINE INDEX IF NOT EXISTS idx_work_pr_url ON {TABLE_PR} FIELDS url UNIQUE;
                "#
            ))
            .await?;

        // Project-Entity junction table
        self.db
            .query(format!(
                r#"
                DEFINE TABLE IF NOT EXISTS {TABLE_PROJECT_ENTITY} SCHEMALESS;
                DEFINE INDEX IF NOT EXISTS idx_wpe_project ON {TABLE_PROJECT_ENTITY} FIELDS project_id;
                DEFINE INDEX IF NOT EXISTS idx_wpe_entity ON {TABLE_PROJECT_ENTITY} FIELDS entity_id;
                DEFINE INDEX IF NOT EXISTS idx_wpe_project_entity ON {TABLE_PROJECT_ENTITY} FIELDS project_id, entity_id UNIQUE;
                "#
            ))
            .await?;

        // Task-Entity junction table
        self.db
            .query(format!(
                r#"
                DEFINE TABLE IF NOT EXISTS {TABLE_TASK_ENTITY} SCHEMALESS;
                DEFINE INDEX IF NOT EXISTS idx_wte_task ON {TABLE_TASK_ENTITY} FIELDS task_id;
                DEFINE INDEX IF NOT EXISTS idx_wte_entity ON {TABLE_TASK_ENTITY} FIELDS entity_id;
                DEFINE INDEX IF NOT EXISTS idx_wte_task_entity ON {TABLE_TASK_ENTITY} FIELDS task_id, entity_id UNIQUE;
                "#
            ))
            .await?;

        // Project Observation table
        self.db
            .query(format!(
                r#"
                DEFINE TABLE IF NOT EXISTS {TABLE_PROJECT_OBSERVATION} SCHEMALESS;
                DEFINE INDEX IF NOT EXISTS idx_wpo_project ON {TABLE_PROJECT_OBSERVATION} FIELDS project_id;
                DEFINE INDEX IF NOT EXISTS idx_wpo_key ON {TABLE_PROJECT_OBSERVATION} FIELDS key;
                DEFINE INDEX IF NOT EXISTS idx_wpo_project_key ON {TABLE_PROJECT_OBSERVATION} FIELDS project_id, key;
                "#
            ))
            .await?;

        // Task Observation table
        self.db
            .query(format!(
                r#"
                DEFINE TABLE IF NOT EXISTS {TABLE_TASK_OBSERVATION} SCHEMALESS;
                DEFINE INDEX IF NOT EXISTS idx_wto_task ON {TABLE_TASK_OBSERVATION} FIELDS task_id;
                DEFINE INDEX IF NOT EXISTS idx_wto_key ON {TABLE_TASK_OBSERVATION} FIELDS key;
                "#
            ))
            .await?;

        // Session Work Context table
        self.db
            .query(format!(
                r#"
                DEFINE TABLE IF NOT EXISTS {TABLE_SESSION_CONTEXT} SCHEMALESS;
                "#
            ))
            .await?;

        info!("Work schema initialized");
        Ok(())
    }

    // =========================================================================
    // Project Operations
    // =========================================================================

    /// Create a project.
    pub async fn create_project(&self, project: &Project) -> StoreResult<()> {
        debug!("Creating project: {}", project.name);

        self.db
            .query(
                r#"
                CREATE type::thing("work_project", $id) SET
                    name = $name,
                    description = $description,
                    status = $status,
                    created_at = $created_at,
                    updated_at = $updated_at
            "#,
            )
            .bind(("id", project.id.to_string()))
            .bind(("name", project.name.clone()))
            .bind(("description", project.description.clone()))
            .bind(("status", project.status.to_string()))
            .bind((
                "created_at",
                project
                    .created_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            ))
            .bind((
                "updated_at",
                project
                    .updated_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            ))
            .await?;

        Ok(())
    }

    /// Get a project by ID.
    pub async fn get_project(&self, id: &Id) -> StoreResult<Option<Project>> {
        debug!("Getting project: {}", id);

        let mut result = self
            .db
            .query(r#"SELECT * FROM type::thing("work_project", $id)"#)
            .bind(("id", id.to_string()))
            .await?;

        let records: Vec<ProjectRecord> = result.take(0)?;

        if let Some(record) = records.into_iter().next() {
            Ok(Some(self.record_to_project(id.clone(), record)?))
        } else {
            Ok(None)
        }
    }

    /// Get a project by name.
    pub async fn get_project_by_name(&self, name: &str) -> StoreResult<Option<Project>> {
        debug!("Getting project by name: {}", name);

        let mut result = self
            .db
            .query("SELECT meta::id(id) as id, * FROM work_project WHERE name = $name LIMIT 1")
            .bind(("name", name.to_string()))
            .await?;

        let records: Vec<ProjectRecordWithId> = result.take(0)?;

        if let Some(record) = records.into_iter().next() {
            let id = self.parse_record_id(&record.id)?;
            Ok(Some(self.record_to_project(id, record.into())?))
        } else {
            Ok(None)
        }
    }

    /// List projects, optionally filtered by status.
    pub async fn list_projects(&self, status: Option<ProjectStatus>) -> StoreResult<Vec<Project>> {
        debug!("Listing projects (status filter: {:?})", status);

        let query = match status {
            Some(s) => format!(
                "SELECT meta::id(id) as id, * FROM {} WHERE status = '{}' ORDER BY name",
                TABLE_PROJECT, s
            ),
            None => format!(
                "SELECT meta::id(id) as id, * FROM {} ORDER BY name",
                TABLE_PROJECT
            ),
        };

        let mut result = self.db.query(query).await?;
        let records: Vec<ProjectRecordWithId> = result.take(0)?;

        let mut projects = Vec::new();
        for record in records {
            let id = self.parse_record_id(&record.id)?;
            projects.push(self.record_to_project(id, record.into())?);
        }

        Ok(projects)
    }

    /// Update a project.
    pub async fn update_project(&self, project: &Project) -> StoreResult<()> {
        debug!("Updating project: {}", project.name);

        self.db
            .query(
                r#"
                UPDATE type::thing("work_project", $id) SET
                    name = $name,
                    description = $description,
                    status = $status,
                    updated_at = $updated_at
            "#,
            )
            .bind(("id", project.id.to_string()))
            .bind(("name", project.name.clone()))
            .bind(("description", project.description.clone()))
            .bind(("status", project.status.to_string()))
            .bind((
                "updated_at",
                project
                    .updated_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            ))
            .await?;

        Ok(())
    }

    /// Delete a project and all related data.
    pub async fn delete_project(&self, id: &Id) -> StoreResult<()> {
        debug!("Deleting project: {}", id);

        // Delete tasks (and their observations, entity connections)
        let tasks = self.list_tasks(id, None).await?;
        for task in tasks {
            self.delete_task(&task.id).await?;
        }

        // Delete PRs
        self.db
            .query("DELETE FROM work_pr WHERE project_id = $id")
            .bind(("id", id.to_string()))
            .await?;

        // Delete entity connections
        self.db
            .query("DELETE FROM work_project_entity WHERE project_id = $id")
            .bind(("id", id.to_string()))
            .await?;

        // Delete project observations
        self.db
            .query("DELETE FROM work_project_observation WHERE project_id = $id")
            .bind(("id", id.to_string()))
            .await?;

        // Delete the project
        self.db
            .query(r#"DELETE type::thing("work_project", $id)"#)
            .bind(("id", id.to_string()))
            .await?;

        Ok(())
    }

    // =========================================================================
    // Task Operations
    // =========================================================================

    /// Create a task.
    pub async fn create_task(&self, task: &Task) -> StoreResult<()> {
        debug!(
            "Creating task: {} (project: {})",
            task.name, task.project_id
        );

        let blocked_by: Vec<String> = task.blocked_by.iter().map(|id| id.to_string()).collect();

        self.db
            .query(
                r#"
                CREATE type::thing("work_task", $id) SET
                    project_id = $project_id,
                    name = $name,
                    description = $description,
                    status = $status,
                    priority = $priority,
                    jira_key = $jira_key,
                    blocked_by = $blocked_by,
                    created_at = $created_at,
                    updated_at = $updated_at
            "#,
            )
            .bind(("id", task.id.to_string()))
            .bind(("project_id", task.project_id.to_string()))
            .bind(("name", task.name.clone()))
            .bind(("description", task.description.clone()))
            .bind(("status", task.status.to_string()))
            .bind(("priority", task.priority.to_string()))
            .bind(("jira_key", task.jira_key.clone()))
            .bind(("blocked_by", blocked_by))
            .bind((
                "created_at",
                task.created_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            ))
            .bind((
                "updated_at",
                task.updated_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            ))
            .await?;

        Ok(())
    }

    /// Get a task by ID.
    pub async fn get_task(&self, id: &Id) -> StoreResult<Option<Task>> {
        debug!("Getting task: {}", id);

        let mut result = self
            .db
            .query(r#"SELECT * FROM type::thing("work_task", $id)"#)
            .bind(("id", id.to_string()))
            .await?;

        let records: Vec<TaskRecord> = result.take(0)?;

        if let Some(record) = records.into_iter().next() {
            Ok(Some(self.record_to_task(id.clone(), record)?))
        } else {
            Ok(None)
        }
    }

    /// Get a task by JIRA key.
    pub async fn get_task_by_jira(&self, jira_key: &str) -> StoreResult<Option<Task>> {
        debug!("Getting task by JIRA key: {}", jira_key);

        let mut result = self
            .db
            .query("SELECT meta::id(id) as id, * FROM work_task WHERE jira_key = $jira_key LIMIT 1")
            .bind(("jira_key", jira_key.to_string()))
            .await?;

        let records: Vec<TaskRecordWithId> = result.take(0)?;

        if let Some(record) = records.into_iter().next() {
            let id = self.parse_record_id(&record.id)?;
            Ok(Some(self.record_to_task(id, record.into())?))
        } else {
            Ok(None)
        }
    }

    /// Get a task by name within a project.
    pub async fn get_task_by_name(&self, project_id: &Id, name: &str) -> StoreResult<Option<Task>> {
        debug!("Getting task by name: {} (project: {})", name, project_id);

        let mut result = self
            .db
            .query("SELECT meta::id(id) as id, * FROM work_task WHERE project_id = $project_id AND name = $name LIMIT 1")
            .bind(("project_id", project_id.to_string()))
            .bind(("name", name.to_string()))
            .await?;

        let records: Vec<TaskRecordWithId> = result.take(0)?;

        if let Some(record) = records.into_iter().next() {
            let id = self.parse_record_id(&record.id)?;
            Ok(Some(self.record_to_task(id, record.into())?))
        } else {
            Ok(None)
        }
    }

    /// List tasks for a project, optionally filtered by status.
    pub async fn list_tasks(
        &self,
        project_id: &Id,
        status: Option<TaskStatus>,
    ) -> StoreResult<Vec<Task>> {
        debug!(
            "Listing tasks for project {} (status filter: {:?})",
            project_id, status
        );

        let query = match status {
            Some(s) => format!(
                "SELECT meta::id(id) as id, * FROM {} WHERE project_id = $project_id AND status = '{}' ORDER BY priority DESC, name",
                TABLE_TASK, s
            ),
            None => format!(
                "SELECT meta::id(id) as id, * FROM {} WHERE project_id = $project_id ORDER BY priority DESC, name",
                TABLE_TASK
            ),
        };

        let mut result = self
            .db
            .query(query)
            .bind(("project_id", project_id.to_string()))
            .await?;

        let records: Vec<TaskRecordWithId> = result.take(0)?;

        let mut tasks = Vec::new();
        for record in records {
            let id = self.parse_record_id(&record.id)?;
            tasks.push(self.record_to_task(id, record.into())?);
        }

        Ok(tasks)
    }

    /// Update a task.
    pub async fn update_task(&self, task: &Task) -> StoreResult<()> {
        debug!("Updating task: {}", task.name);

        let blocked_by: Vec<String> = task.blocked_by.iter().map(|id| id.to_string()).collect();

        self.db
            .query(
                r#"
                UPDATE type::thing("work_task", $id) SET
                    name = $name,
                    description = $description,
                    status = $status,
                    priority = $priority,
                    jira_key = $jira_key,
                    blocked_by = $blocked_by,
                    updated_at = $updated_at
            "#,
            )
            .bind(("id", task.id.to_string()))
            .bind(("name", task.name.clone()))
            .bind(("description", task.description.clone()))
            .bind(("status", task.status.to_string()))
            .bind(("priority", task.priority.to_string()))
            .bind(("jira_key", task.jira_key.clone()))
            .bind(("blocked_by", blocked_by))
            .bind((
                "updated_at",
                task.updated_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            ))
            .await?;

        Ok(())
    }

    /// Delete a task and related data.
    pub async fn delete_task(&self, id: &Id) -> StoreResult<()> {
        debug!("Deleting task: {}", id);

        // Delete task observations
        self.db
            .query("DELETE FROM work_task_observation WHERE task_id = $id")
            .bind(("id", id.to_string()))
            .await?;

        // Delete entity connections
        self.db
            .query("DELETE FROM work_task_entity WHERE task_id = $id")
            .bind(("id", id.to_string()))
            .await?;

        // Update PRs to remove task reference (keep PRs on project)
        self.db
            .query("UPDATE work_pr SET task_id = NONE WHERE task_id = $id")
            .bind(("id", id.to_string()))
            .await?;

        // Delete the task
        self.db
            .query(r#"DELETE type::thing("work_task", $id)"#)
            .bind(("id", id.to_string()))
            .await?;

        Ok(())
    }

    // =========================================================================
    // PR Operations
    // =========================================================================

    /// Create a PR.
    pub async fn create_pr(&self, pr: &Pr) -> StoreResult<()> {
        debug!("Creating PR: {} (project: {})", pr.url, pr.project_id);

        let blocked_by: Vec<String> = pr.blocked_by.iter().map(|id| id.to_string()).collect();

        self.db
            .query(
                r#"
                CREATE type::thing("work_pr", $id) SET
                    task_id = $task_id,
                    project_id = $project_id,
                    url = $url,
                    repo = $repo,
                    pr_number = $pr_number,
                    title = $title,
                    status = $status,
                    blocked_by = $blocked_by,
                    created_at = $created_at,
                    updated_at = $updated_at
            "#,
            )
            .bind(("id", pr.id.to_string()))
            .bind(("task_id", pr.task_id.as_ref().map(|id| id.to_string())))
            .bind(("project_id", pr.project_id.to_string()))
            .bind(("url", pr.url.clone()))
            .bind(("repo", pr.repo.clone()))
            .bind(("pr_number", pr.pr_number))
            .bind(("title", pr.title.clone()))
            .bind(("status", pr.status.to_string()))
            .bind(("blocked_by", blocked_by))
            .bind((
                "created_at",
                pr.created_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            ))
            .bind((
                "updated_at",
                pr.updated_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            ))
            .await?;

        Ok(())
    }

    /// Get a PR by ID.
    pub async fn get_pr(&self, id: &Id) -> StoreResult<Option<Pr>> {
        debug!("Getting PR: {}", id);

        let mut result = self
            .db
            .query(r#"SELECT * FROM type::thing("work_pr", $id)"#)
            .bind(("id", id.to_string()))
            .await?;

        let records: Vec<PrRecord> = result.take(0)?;

        if let Some(record) = records.into_iter().next() {
            Ok(Some(self.record_to_pr(id.clone(), record)?))
        } else {
            Ok(None)
        }
    }

    /// Get a PR by URL.
    pub async fn get_pr_by_url(&self, url: &str) -> StoreResult<Option<Pr>> {
        debug!("Getting PR by URL: {}", url);

        let mut result = self
            .db
            .query("SELECT meta::id(id) as id, * FROM work_pr WHERE url = $url LIMIT 1")
            .bind(("url", url.to_string()))
            .await?;

        let records: Vec<PrRecordWithId> = result.take(0)?;

        if let Some(record) = records.into_iter().next() {
            let id = self.parse_record_id(&record.id)?;
            Ok(Some(self.record_to_pr(id, record.into())?))
        } else {
            Ok(None)
        }
    }

    /// List PRs, optionally filtered by project and/or task.
    pub async fn list_prs(
        &self,
        project_id: Option<&Id>,
        task_id: Option<&Id>,
    ) -> StoreResult<Vec<Pr>> {
        debug!(
            "Listing PRs (project: {:?}, task: {:?})",
            project_id, task_id
        );

        let (query, binds) = match (project_id, task_id) {
            (Some(pid), Some(tid)) => (
                format!(
                    "SELECT meta::id(id) as id, * FROM {} WHERE project_id = $project_id AND task_id = $task_id ORDER BY created_at DESC",
                    TABLE_PR
                ),
                vec![
                    ("project_id".to_string(), pid.to_string()),
                    ("task_id".to_string(), tid.to_string()),
                ],
            ),
            (Some(pid), None) => (
                format!(
                    "SELECT meta::id(id) as id, * FROM {} WHERE project_id = $project_id ORDER BY created_at DESC",
                    TABLE_PR
                ),
                vec![("project_id".to_string(), pid.to_string())],
            ),
            (None, Some(tid)) => (
                format!(
                    "SELECT meta::id(id) as id, * FROM {} WHERE task_id = $task_id ORDER BY created_at DESC",
                    TABLE_PR
                ),
                vec![("task_id".to_string(), tid.to_string())],
            ),
            (None, None) => (
                format!(
                    "SELECT meta::id(id) as id, * FROM {} ORDER BY created_at DESC",
                    TABLE_PR
                ),
                vec![],
            ),
        };

        let mut q = self.db.query(query);
        for (key, value) in binds {
            q = q.bind((key, value));
        }

        let mut result = q.await?;
        let records: Vec<PrRecordWithId> = result.take(0)?;

        let mut prs = Vec::new();
        for record in records {
            let id = self.parse_record_id(&record.id)?;
            prs.push(self.record_to_pr(id, record.into())?);
        }

        Ok(prs)
    }

    /// Update a PR.
    pub async fn update_pr(&self, pr: &Pr) -> StoreResult<()> {
        debug!("Updating PR: {}", pr.url);

        let blocked_by: Vec<String> = pr.blocked_by.iter().map(|id| id.to_string()).collect();

        self.db
            .query(
                r#"
                UPDATE type::thing("work_pr", $id) SET
                    task_id = $task_id,
                    title = $title,
                    status = $status,
                    blocked_by = $blocked_by,
                    updated_at = $updated_at
            "#,
            )
            .bind(("id", pr.id.to_string()))
            .bind(("task_id", pr.task_id.as_ref().map(|id| id.to_string())))
            .bind(("title", pr.title.clone()))
            .bind(("status", pr.status.to_string()))
            .bind(("blocked_by", blocked_by))
            .bind((
                "updated_at",
                pr.updated_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            ))
            .await?;

        Ok(())
    }

    /// Delete a PR.
    pub async fn delete_pr(&self, id: &Id) -> StoreResult<()> {
        debug!("Deleting PR: {}", id);

        self.db
            .query(r#"DELETE type::thing("work_pr", $id)"#)
            .bind(("id", id.to_string()))
            .await?;

        Ok(())
    }

    // =========================================================================
    // Entity Connection Operations
    // =========================================================================

    /// Connect a project to an entity.
    pub async fn connect_project_entity(
        &self,
        project_id: &Id,
        entity_id: &Id,
        relation: &ProjectEntityRelation,
    ) -> StoreResult<()> {
        debug!(
            "Connecting project {} to entity {} ({})",
            project_id, entity_id, relation
        );

        // Use raw strings without parameter binding to avoid SurrealDB auto-conversion
        // Add RETURN NONE to prevent SurrealDB from returning the created record
        let delete_query = format!(
            r#"DELETE FROM work_project_entity WHERE project_id = "{}" AND entity_id = "{}" RETURN NONE"#,
            project_id, entity_id
        );
        self.db.query(&delete_query).await?;

        let create_query = format!(
            r#"CREATE work_project_entity SET project_id = "{}", entity_id = "{}", relation = "{}" RETURN NONE"#,
            project_id, entity_id, relation
        );
        self.db.query(&create_query).await?;

        Ok(())
    }

    /// Disconnect a project from an entity.
    pub async fn disconnect_project_entity(
        &self,
        project_id: &Id,
        entity_id: &Id,
    ) -> StoreResult<()> {
        debug!(
            "Disconnecting project {} from entity {}",
            project_id, entity_id
        );

        let query = format!(
            r#"DELETE FROM work_project_entity WHERE project_id = "{}" AND entity_id = "{}""#,
            project_id, entity_id
        );
        self.db.query(&query).await?;

        Ok(())
    }

    /// Get entities connected to a project.
    pub async fn get_project_entities(
        &self,
        project_id: &Id,
    ) -> StoreResult<Vec<(Id, ProjectEntityRelation)>> {
        debug!("Getting entities for project: {}", project_id);

        let query = format!(
            r#"SELECT entity_id, relation FROM work_project_entity WHERE project_id = "{}""#,
            project_id
        );
        let mut result = self.db.query(&query).await?;

        let records: Vec<EntityConnectionRecord> = result.take(0)?;

        let mut connections = Vec::new();
        for record in records {
            let entity_id = Id::parse(&record.entity_id)
                .map_err(|e| StoreError::Deserialization(format!("Invalid entity ID: {}", e)))?;
            let relation = ProjectEntityRelation::parse(&record.relation);
            connections.push((entity_id, relation));
        }

        Ok(connections)
    }

    /// Get projects that reference an entity.
    pub async fn get_entity_projects(&self, entity_id: &Id) -> StoreResult<Vec<Id>> {
        debug!("Getting projects for entity: {}", entity_id);

        #[derive(Deserialize)]
        struct ProjectIdRecord {
            project_id: String,
        }

        let query = format!(
            r#"SELECT project_id FROM work_project_entity WHERE entity_id = "{}""#,
            entity_id
        );
        let mut result = self.db.query(&query).await?;

        let records: Vec<ProjectIdRecord> = result.take(0)?;

        let mut project_ids = Vec::new();
        for record in records {
            let project_id = Id::parse(&record.project_id)
                .map_err(|e| StoreError::Deserialization(format!("Invalid project ID: {}", e)))?;
            project_ids.push(project_id);
        }

        Ok(project_ids)
    }

    /// Connect a task to an entity.
    pub async fn connect_task_entity(
        &self,
        task_id: &Id,
        entity_id: &Id,
        relation: &TaskEntityRelation,
    ) -> StoreResult<()> {
        debug!(
            "Connecting task {} to entity {} ({})",
            task_id, entity_id, relation
        );

        // Use raw strings without parameter binding to avoid SurrealDB auto-conversion
        // Add RETURN NONE to prevent SurrealDB from returning the created record
        let delete_query = format!(
            r#"DELETE FROM work_task_entity WHERE task_id = "{}" AND entity_id = "{}" RETURN NONE"#,
            task_id, entity_id
        );
        self.db.query(&delete_query).await?;

        let create_query = format!(
            r#"CREATE work_task_entity SET task_id = "{}", entity_id = "{}", relation = "{}" RETURN NONE"#,
            task_id, entity_id, relation
        );
        self.db.query(&create_query).await?;

        Ok(())
    }

    /// Disconnect a task from an entity.
    pub async fn disconnect_task_entity(&self, task_id: &Id, entity_id: &Id) -> StoreResult<()> {
        debug!("Disconnecting task {} from entity {}", task_id, entity_id);

        let query = format!(
            r#"DELETE FROM work_task_entity WHERE task_id = "{}" AND entity_id = "{}""#,
            task_id, entity_id
        );
        self.db.query(&query).await?;

        Ok(())
    }

    /// Get entities connected to a task.
    pub async fn get_task_entities(
        &self,
        task_id: &Id,
    ) -> StoreResult<Vec<(Id, TaskEntityRelation)>> {
        debug!("Getting entities for task: {}", task_id);

        let mut result = self
            .db
            .query(&format!(
                r#"SELECT entity_id, relation FROM work_task_entity WHERE task_id = "{}""#,
                task_id
            ))
            .await?;

        let records: Vec<EntityConnectionRecord> = result.take(0)?;

        let mut connections = Vec::new();
        for record in records {
            let entity_id = Id::parse(&record.entity_id)
                .map_err(|e| StoreError::Deserialization(format!("Invalid entity ID: {}", e)))?;
            let relation = TaskEntityRelation::parse(&record.relation);
            connections.push((entity_id, relation));
        }

        Ok(connections)
    }

    // =========================================================================
    // Project Observation Operations
    // =========================================================================

    /// Add or update a project observation.
    pub async fn add_project_observation(
        &self,
        obs: &ProjectObservation,
    ) -> StoreResult<Option<ProjectObservation>> {
        debug!(
            "Adding/updating project observation for {} (key: {:?})",
            obs.project_id, obs.key
        );

        let mut previous: Option<ProjectObservation> = None;

        // If there's a key, check for existing and update
        if let Some(key) = &obs.key {
            if let Some(existing) = self
                .get_project_observation_by_key(&obs.project_id, key)
                .await?
            {
                previous = Some(existing.clone());

                self.db
                    .query(
                        r#"
                        UPDATE type::thing("work_project_observation", $id) SET
                            content = $content,
                            embedding = $embedding,
                            source = $source,
                            updated_at = $updated_at
                    "#,
                    )
                    .bind(("id", existing.id.to_string()))
                    .bind(("content", obs.content.clone()))
                    .bind(("embedding", obs.embedding.clone()))
                    .bind(("source", obs.source.clone()))
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
                CREATE type::thing("work_project_observation", $id) SET
                    project_id = $project_id,
                    key = $key,
                    content = $content,
                    embedding = $embedding,
                    source = $source,
                    created_at = $created_at,
                    updated_at = $updated_at
            "#,
            )
            .bind(("id", obs.id.to_string()))
            .bind(("project_id", obs.project_id.to_string()))
            .bind(("key", obs.key.clone()))
            .bind(("content", obs.content.clone()))
            .bind(("embedding", obs.embedding.clone()))
            .bind(("source", obs.source.clone()))
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

    /// Get a project observation by key.
    pub async fn get_project_observation_by_key(
        &self,
        project_id: &Id,
        key: &str,
    ) -> StoreResult<Option<ProjectObservation>> {
        debug!(
            "Getting project observation by key: {} for project {}",
            key, project_id
        );

        let mut result = self
            .db
            .query("SELECT meta::id(id) as id, * FROM work_project_observation WHERE project_id = $project_id AND key = $key LIMIT 1")
            .bind(("project_id", project_id.to_string()))
            .bind(("key", key.to_string()))
            .await?;

        let records: Vec<ProjectObservationRecordWithId> = result.take(0)?;

        if let Some(record) = records.into_iter().next() {
            Ok(Some(self.record_to_project_observation(record)?))
        } else {
            Ok(None)
        }
    }

    /// Get all observations for a project.
    pub async fn get_project_observations(
        &self,
        project_id: &Id,
    ) -> StoreResult<Vec<ProjectObservation>> {
        debug!("Getting observations for project: {}", project_id);

        let mut result = self
            .db
            .query("SELECT meta::id(id) as id, * FROM work_project_observation WHERE project_id = $project_id ORDER BY key, updated_at DESC")
            .bind(("project_id", project_id.to_string()))
            .await?;

        let records: Vec<ProjectObservationRecordWithId> = result.take(0)?;

        let mut observations = Vec::new();
        for record in records {
            observations.push(self.record_to_project_observation(record)?);
        }

        Ok(observations)
    }

    /// Search project observations by content.
    pub async fn search_project_observations(
        &self,
        project_id: &Id,
        query: &str,
    ) -> StoreResult<Vec<ProjectObservation>> {
        debug!(
            "Searching project observations for {} with query '{}'",
            project_id, query
        );

        let mut result = self
            .db
            .query("SELECT meta::id(id) as id, * FROM work_project_observation WHERE project_id = $project_id AND content IS NOT NONE AND string::lowercase(content) CONTAINS $query ORDER BY updated_at DESC")
            .bind(("project_id", project_id.to_string()))
            .bind(("query", query.to_lowercase()))
            .await?;

        let records: Vec<ProjectObservationRecordWithId> = result.take(0)?;

        let mut observations = Vec::new();
        for record in records {
            observations.push(self.record_to_project_observation(record)?);
        }

        Ok(observations)
    }

    /// Search project observations by embedding similarity.
    pub async fn search_project_observations_by_embedding(
        &self,
        project_id: &Id,
        query_embedding: &[f32],
        limit: usize,
        min_score: f32,
    ) -> StoreResult<Vec<ProjectObservationSearchResult>> {
        debug!(
            "Searching project observations by embedding for {} (limit={})",
            project_id, limit
        );

        let mut result = self
            .db
            .query("SELECT meta::id(id) as id, * FROM work_project_observation WHERE project_id = $project_id AND embedding IS NOT NONE")
            .bind(("project_id", project_id.to_string()))
            .await?;

        let records: Vec<ProjectObservationRecordWithId> = result.take(0)?;

        let mut scored_results: Vec<ProjectObservationSearchResult> = Vec::new();
        for record in records {
            if let Some(ref embedding) = record.embedding {
                let score = cosine_similarity(query_embedding, embedding);
                if score >= min_score {
                    let observation = self.record_to_project_observation(record)?;
                    scored_results.push(ProjectObservationSearchResult { observation, score });
                }
            }
        }

        scored_results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored_results.truncate(limit);

        Ok(scored_results)
    }

    /// Delete a project observation.
    pub async fn delete_project_observation(&self, id: &Id) -> StoreResult<()> {
        debug!("Deleting project observation: {}", id);

        self.db
            .query(r#"DELETE type::thing("work_project_observation", $id)"#)
            .bind(("id", id.to_string()))
            .await?;

        Ok(())
    }

    // =========================================================================
    // Task Observation Operations
    // =========================================================================

    /// Add or update a task observation.
    pub async fn add_task_observation(
        &self,
        obs: &TaskObservation,
    ) -> StoreResult<Option<TaskObservation>> {
        debug!(
            "Adding/updating task observation for {} (key: {:?})",
            obs.task_id, obs.key
        );

        let mut previous: Option<TaskObservation> = None;

        // If there's a key, check for existing and update
        if let Some(key) = &obs.key {
            if let Some(existing) = self.get_task_observation_by_key(&obs.task_id, key).await? {
                previous = Some(existing.clone());

                self.db
                    .query(
                        r#"
                        UPDATE type::thing("work_task_observation", $id) SET
                            content = $content,
                            embedding = $embedding,
                            source = $source,
                            updated_at = $updated_at
                    "#,
                    )
                    .bind(("id", existing.id.to_string()))
                    .bind(("content", obs.content.clone()))
                    .bind(("embedding", obs.embedding.clone()))
                    .bind(("source", obs.source.clone()))
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
                CREATE type::thing("work_task_observation", $id) SET
                    task_id = $task_id,
                    key = $key,
                    content = $content,
                    embedding = $embedding,
                    source = $source,
                    created_at = $created_at,
                    updated_at = $updated_at
            "#,
            )
            .bind(("id", obs.id.to_string()))
            .bind(("task_id", obs.task_id.to_string()))
            .bind(("key", obs.key.clone()))
            .bind(("content", obs.content.clone()))
            .bind(("embedding", obs.embedding.clone()))
            .bind(("source", obs.source.clone()))
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

    /// Get a task observation by key.
    pub async fn get_task_observation_by_key(
        &self,
        task_id: &Id,
        key: &str,
    ) -> StoreResult<Option<TaskObservation>> {
        debug!(
            "Getting task observation by key: {} for task {}",
            key, task_id
        );

        let mut result = self
            .db
            .query("SELECT meta::id(id) as id, * FROM work_task_observation WHERE task_id = $task_id AND key = $key LIMIT 1")
            .bind(("task_id", task_id.to_string()))
            .bind(("key", key.to_string()))
            .await?;

        let records: Vec<TaskObservationRecordWithId> = result.take(0)?;

        if let Some(record) = records.into_iter().next() {
            Ok(Some(self.record_to_task_observation(record)?))
        } else {
            Ok(None)
        }
    }

    /// Get all observations for a task.
    pub async fn get_task_observations(&self, task_id: &Id) -> StoreResult<Vec<TaskObservation>> {
        debug!("Getting observations for task: {}", task_id);

        let mut result = self
            .db
            .query("SELECT meta::id(id) as id, * FROM work_task_observation WHERE task_id = $task_id ORDER BY key, updated_at DESC")
            .bind(("task_id", task_id.to_string()))
            .await?;

        let records: Vec<TaskObservationRecordWithId> = result.take(0)?;

        let mut observations = Vec::new();
        for record in records {
            observations.push(self.record_to_task_observation(record)?);
        }

        Ok(observations)
    }

    /// Search task observations by embedding similarity.
    pub async fn search_task_observations_by_embedding(
        &self,
        task_id: &Id,
        query_embedding: &[f32],
        limit: usize,
        min_score: f32,
    ) -> StoreResult<Vec<TaskObservationSearchResult>> {
        debug!(
            "Searching task observations by embedding for {} (limit={})",
            task_id, limit
        );

        let mut result = self
            .db
            .query("SELECT meta::id(id) as id, * FROM work_task_observation WHERE task_id = $task_id AND embedding IS NOT NONE")
            .bind(("task_id", task_id.to_string()))
            .await?;

        let records: Vec<TaskObservationRecordWithId> = result.take(0)?;

        let mut scored_results: Vec<TaskObservationSearchResult> = Vec::new();
        for record in records {
            if let Some(ref embedding) = record.embedding {
                let score = cosine_similarity(query_embedding, embedding);
                if score >= min_score {
                    let observation = self.record_to_task_observation(record)?;
                    scored_results.push(TaskObservationSearchResult { observation, score });
                }
            }
        }

        scored_results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored_results.truncate(limit);

        Ok(scored_results)
    }

    /// Delete a task observation.
    pub async fn delete_task_observation(&self, id: &Id) -> StoreResult<()> {
        debug!("Deleting task observation: {}", id);

        self.db
            .query(r#"DELETE type::thing("work_task_observation", $id)"#)
            .bind(("id", id.to_string()))
            .await?;

        Ok(())
    }

    // =========================================================================
    // Work Context Operations
    // =========================================================================

    /// Set session work context.
    pub async fn set_session_context(&self, ctx: &WorkContext) -> StoreResult<()> {
        debug!(
            "Setting work context for session {} (project: {:?}, task: {:?})",
            ctx.session_id, ctx.project_id, ctx.task_id
        );

        self.db
            .query(
                r#"
                UPSERT type::thing("work_session_context", $session_id) SET
                    session_id = $session_id,
                    project_id = $project_id,
                    task_id = $task_id,
                    joined_at = $joined_at
            "#,
            )
            .bind(("session_id", ctx.session_id.to_string()))
            .bind((
                "project_id",
                ctx.project_id.as_ref().map(|id| id.to_string()),
            ))
            .bind(("task_id", ctx.task_id.as_ref().map(|id| id.to_string())))
            .bind((
                "joined_at",
                ctx.joined_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            ))
            .await?;

        Ok(())
    }

    /// Get session work context.
    pub async fn get_session_context(&self, session_id: &Id) -> StoreResult<Option<WorkContext>> {
        debug!("Getting work context for session: {}", session_id);

        let mut result = self
            .db
            .query(r#"SELECT * FROM type::thing("work_session_context", $session_id)"#)
            .bind(("session_id", session_id.to_string()))
            .await?;

        let records: Vec<WorkContextRecord> = result.take(0)?;

        if let Some(record) = records.into_iter().next() {
            Ok(Some(self.record_to_work_context(record)?))
        } else {
            Ok(None)
        }
    }

    /// Clear session work context.
    pub async fn clear_session_context(&self, session_id: &Id) -> StoreResult<()> {
        debug!("Clearing work context for session: {}", session_id);

        self.db
            .query(r#"DELETE type::thing("work_session_context", $session_id)"#)
            .bind(("session_id", session_id.to_string()))
            .await?;

        Ok(())
    }

    // =========================================================================
    // Statistics
    // =========================================================================

    /// Get work statistics.
    pub async fn stats(&self) -> StoreResult<WorkStats> {
        let mut result = self
            .db
            .query(format!(
                r#"
                SELECT count() as count FROM {TABLE_PROJECT} GROUP ALL;
                SELECT count() as count FROM {TABLE_TASK} GROUP ALL;
                SELECT count() as count FROM {TABLE_PR} GROUP ALL;
                SELECT count() as count FROM {TABLE_PROJECT_OBSERVATION} GROUP ALL;
                SELECT count() as count FROM {TABLE_TASK_OBSERVATION} GROUP ALL;
                "#
            ))
            .await?;

        let project_count: Option<CountResult> = result.take(0)?;
        let task_count: Option<CountResult> = result.take(1)?;
        let pr_count: Option<CountResult> = result.take(2)?;
        let project_obs_count: Option<CountResult> = result.take(3)?;
        let task_obs_count: Option<CountResult> = result.take(4)?;

        Ok(WorkStats {
            project_count: project_count.map(|c| c.count).unwrap_or(0),
            task_count: task_count.map(|c| c.count).unwrap_or(0),
            pr_count: pr_count.map(|c| c.count).unwrap_or(0),
            project_observation_count: project_obs_count.map(|c| c.count).unwrap_or(0),
            task_observation_count: task_obs_count.map(|c| c.count).unwrap_or(0),
        })
    }

    // =========================================================================
    // Helper Methods
    // =========================================================================

    fn parse_record_id(&self, record_id: &str) -> StoreResult<Id> {
        let id_str = if record_id.contains(':') {
            record_id.split(':').nth(1).unwrap_or(record_id)
        } else {
            record_id
        };

        Id::parse(id_str).map_err(|e| StoreError::Deserialization(format!("Invalid ID: {}", e)))
    }

    fn record_to_project(&self, id: Id, record: ProjectRecord) -> StoreResult<Project> {
        Ok(Project {
            id,
            name: record.name,
            description: record.description,
            status: ProjectStatus::parse(&record.status),
            created_at: record.created_at.to_offset_datetime()?,
            updated_at: record.updated_at.to_offset_datetime()?,
        })
    }

    fn record_to_task(&self, id: Id, record: TaskRecord) -> StoreResult<Task> {
        let blocked_by: Vec<Id> = record
            .blocked_by
            .iter()
            .filter_map(|s| Id::parse(s).ok())
            .collect();

        Ok(Task {
            id,
            project_id: Id::parse(&record.project_id)
                .map_err(|e| StoreError::Deserialization(format!("Invalid project ID: {}", e)))?,
            name: record.name,
            description: record.description,
            status: TaskStatus::parse(&record.status),
            priority: TaskPriority::parse(&record.priority),
            jira_key: record.jira_key,
            blocked_by,
            created_at: record.created_at.to_offset_datetime()?,
            updated_at: record.updated_at.to_offset_datetime()?,
        })
    }

    fn record_to_pr(&self, id: Id, record: PrRecord) -> StoreResult<Pr> {
        let blocked_by: Vec<Id> = record
            .blocked_by
            .iter()
            .filter_map(|s| Id::parse(s).ok())
            .collect();

        let task_id = record
            .task_id
            .map(|s| {
                Id::parse(&s)
                    .map_err(|e| StoreError::Deserialization(format!("Invalid task ID: {}", e)))
            })
            .transpose()?;

        Ok(Pr {
            id,
            task_id,
            project_id: Id::parse(&record.project_id)
                .map_err(|e| StoreError::Deserialization(format!("Invalid project ID: {}", e)))?,
            url: record.url,
            repo: record.repo,
            pr_number: record.pr_number,
            title: record.title,
            status: PrStatus::parse(&record.status),
            blocked_by,
            created_at: record.created_at.to_offset_datetime()?,
            updated_at: record.updated_at.to_offset_datetime()?,
        })
    }

    fn record_to_project_observation(
        &self,
        record: ProjectObservationRecordWithId,
    ) -> StoreResult<ProjectObservation> {
        let id = self.parse_record_id(&record.id)?;
        let project_id = Id::parse(&record.project_id)
            .map_err(|e| StoreError::Deserialization(format!("Invalid project ID: {}", e)))?;

        Ok(ProjectObservation {
            id,
            project_id,
            key: record.key,
            content: record.content,
            embedding: record.embedding,
            source: record.source,
            created_at: record.created_at.to_offset_datetime()?,
            updated_at: record.updated_at.to_offset_datetime()?,
        })
    }

    fn record_to_task_observation(
        &self,
        record: TaskObservationRecordWithId,
    ) -> StoreResult<TaskObservation> {
        let id = self.parse_record_id(&record.id)?;
        let task_id = Id::parse(&record.task_id)
            .map_err(|e| StoreError::Deserialization(format!("Invalid task ID: {}", e)))?;

        Ok(TaskObservation {
            id,
            task_id,
            key: record.key,
            content: record.content,
            embedding: record.embedding,
            source: record.source,
            created_at: record.created_at.to_offset_datetime()?,
            updated_at: record.updated_at.to_offset_datetime()?,
        })
    }

    fn record_to_work_context(&self, record: WorkContextRecord) -> StoreResult<WorkContext> {
        let session_id = Id::parse(&record.session_id)
            .map_err(|e| StoreError::Deserialization(format!("Invalid session ID: {}", e)))?;

        let project_id = record
            .project_id
            .map(|s| {
                Id::parse(&s)
                    .map_err(|e| StoreError::Deserialization(format!("Invalid project ID: {}", e)))
            })
            .transpose()?;

        let task_id = record
            .task_id
            .map(|s| {
                Id::parse(&s)
                    .map_err(|e| StoreError::Deserialization(format!("Invalid task ID: {}", e)))
            })
            .transpose()?;

        Ok(WorkContext {
            session_id,
            project_id,
            task_id,
            joined_at: record.joined_at.to_offset_datetime()?,
        })
    }
}

// ============================================================================
// Record Conversions
// ============================================================================

impl From<ProjectRecordWithId> for ProjectRecord {
    fn from(r: ProjectRecordWithId) -> Self {
        ProjectRecord {
            name: r.name,
            description: r.description,
            status: r.status,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

impl From<TaskRecordWithId> for TaskRecord {
    fn from(r: TaskRecordWithId) -> Self {
        TaskRecord {
            project_id: r.project_id,
            name: r.name,
            description: r.description,
            status: r.status,
            priority: r.priority,
            jira_key: r.jira_key,
            blocked_by: r.blocked_by,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

impl From<PrRecordWithId> for PrRecord {
    fn from(r: PrRecordWithId) -> Self {
        PrRecord {
            task_id: r.task_id,
            project_id: r.project_id,
            url: r.url,
            repo: r.repo,
            pr_number: r.pr_number,
            title: r.title,
            status: r.status,
            blocked_by: r.blocked_by,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_work_stats_default() {
        let stats = WorkStats::default();
        assert_eq!(stats.project_count, 0);
        assert_eq!(stats.task_count, 0);
        assert_eq!(stats.pr_count, 0);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0, 0.0];
        assert!(cosine_similarity(&a, &c).abs() < 0.001);
    }
}
