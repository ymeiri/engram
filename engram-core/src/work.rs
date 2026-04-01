//! Work management types (Layer 7).
//!
//! Work management provides cross-repository project and task tracking:
//! - Projects: Logical groupings of work spanning multiple repos
//! - Tasks: Work items (JIRA tickets, features) within projects
//! - PRs: Pull requests implementing tasks
//! - Work Observations: Project/task-scoped knowledge (separate from entity observations)
//!
//! Key Design Principle: Entities (Layer 1) remain standalone. Projects/Tasks
//! reference entities via junction tables. Observations are stored in the
//! appropriate scope (entity=global, project/task=contextual).

use crate::id::Id;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

// ============================================================================
// Project
// ============================================================================

/// Status of a project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    /// Initial planning phase.
    Planning,
    /// Actively being worked on.
    #[default]
    Active,
    /// Work completed.
    Completed,
    /// No longer active but preserved.
    Archived,
}

impl std::fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Planning => write!(f, "planning"),
            Self::Active => write!(f, "active"),
            Self::Completed => write!(f, "completed"),
            Self::Archived => write!(f, "archived"),
        }
    }
}

impl ProjectStatus {
    /// Parse from string.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "planning" => Self::Planning,
            "active" => Self::Active,
            "completed" => Self::Completed,
            "archived" => Self::Archived,
            _ => Self::Active,
        }
    }
}

/// A project representing a logical grouping of work.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Unique identifier.
    pub id: Id,

    /// Human-readable unique name.
    pub name: String,

    /// Description of the project.
    pub description: Option<String>,

    /// Current status.
    pub status: ProjectStatus,

    /// Creation timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,

    /// Last update timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl Project {
    /// Create a new project.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: Id::new(),
            name: name.into(),
            description: None,
            status: ProjectStatus::default(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Set the description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the status.
    #[must_use]
    pub fn with_status(mut self, status: ProjectStatus) -> Self {
        self.status = status;
        self
    }
}

// ============================================================================
// Task
// ============================================================================

/// Status of a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// Not yet started.
    #[default]
    Todo,
    /// Currently being worked on.
    InProgress,
    /// Waiting on dependencies.
    Blocked,
    /// Completed.
    Done,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Todo => write!(f, "todo"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Blocked => write!(f, "blocked"),
            Self::Done => write!(f, "done"),
        }
    }
}

impl TaskStatus {
    /// Parse from string.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "todo" => Self::Todo,
            "in_progress" | "inprogress" => Self::InProgress,
            "blocked" => Self::Blocked,
            "done" => Self::Done,
            _ => Self::Todo,
        }
    }
}

/// Priority of a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    /// Low priority.
    Low,
    /// Medium priority (default).
    #[default]
    Medium,
    /// High priority.
    High,
    /// Critical priority.
    Critical,
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

impl TaskPriority {
    /// Parse from string.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "low" => Self::Low,
            "medium" => Self::Medium,
            "high" => Self::High,
            "critical" => Self::Critical,
            _ => Self::Medium,
        }
    }
}

/// A task within a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique identifier.
    pub id: Id,

    /// Parent project ID.
    pub project_id: Id,

    /// Human-readable name.
    pub name: String,

    /// Description of the task.
    pub description: Option<String>,

    /// Current status.
    pub status: TaskStatus,

    /// Priority level.
    pub priority: TaskPriority,

    /// External tracker key (e.g., "IDEAI-235").
    pub jira_key: Option<String>,

    /// IDs of tasks that block this one.
    #[serde(default)]
    pub blocked_by: Vec<Id>,

    /// Creation timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,

    /// Last update timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl Task {
    /// Create a new task.
    #[must_use]
    pub fn new(project_id: Id, name: impl Into<String>) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: Id::new(),
            project_id,
            name: name.into(),
            description: None,
            status: TaskStatus::default(),
            priority: TaskPriority::default(),
            jira_key: None,
            blocked_by: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Set the description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the JIRA key.
    #[must_use]
    pub fn with_jira_key(mut self, jira_key: impl Into<String>) -> Self {
        self.jira_key = Some(jira_key.into());
        self
    }

    /// Set the priority.
    #[must_use]
    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set the status.
    #[must_use]
    pub fn with_status(mut self, status: TaskStatus) -> Self {
        self.status = status;
        self
    }

    /// Set blocked_by task IDs.
    #[must_use]
    pub fn with_blocked_by(mut self, blocked_by: Vec<Id>) -> Self {
        self.blocked_by = blocked_by;
        self
    }
}

// ============================================================================
// PR (Pull Request)
// ============================================================================

/// Status of a pull request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PrStatus {
    /// Open and awaiting review/merge.
    #[default]
    Open,
    /// Successfully merged.
    Merged,
    /// Closed without merging.
    Closed,
}

impl std::fmt::Display for PrStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open => write!(f, "open"),
            Self::Merged => write!(f, "merged"),
            Self::Closed => write!(f, "closed"),
        }
    }
}

impl PrStatus {
    /// Parse from string.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "open" => Self::Open,
            "merged" => Self::Merged,
            "closed" => Self::Closed,
            _ => Self::Open,
        }
    }
}

/// A pull request associated with a project/task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pr {
    /// Unique identifier.
    pub id: Id,

    /// Associated task ID (optional - can be directly on project).
    pub task_id: Option<Id>,

    /// Parent project ID.
    pub project_id: Id,

    /// PR URL (unique identifier).
    pub url: String,

    /// Repository name (e.g., "my-project", "web-ui").
    pub repo: String,

    /// PR number within the repository.
    pub pr_number: u32,

    /// PR title.
    pub title: Option<String>,

    /// Current status.
    pub status: PrStatus,

    /// IDs of PRs that block this one.
    #[serde(default)]
    pub blocked_by: Vec<Id>,

    /// Creation timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,

    /// Last update timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl Pr {
    /// Create a new PR.
    #[must_use]
    pub fn new(
        project_id: Id,
        url: impl Into<String>,
        repo: impl Into<String>,
        pr_number: u32,
    ) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: Id::new(),
            task_id: None,
            project_id,
            url: url.into(),
            repo: repo.into(),
            pr_number,
            title: None,
            status: PrStatus::default(),
            blocked_by: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Set the associated task.
    #[must_use]
    pub fn with_task(mut self, task_id: Id) -> Self {
        self.task_id = Some(task_id);
        self
    }

    /// Set the title.
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the status.
    #[must_use]
    pub fn with_status(mut self, status: PrStatus) -> Self {
        self.status = status;
        self
    }

    /// Set blocked_by PR IDs.
    #[must_use]
    pub fn with_blocked_by(mut self, blocked_by: Vec<Id>) -> Self {
        self.blocked_by = blocked_by;
        self
    }
}

// ============================================================================
// Project Observation
// ============================================================================

/// An observation (fact/note) scoped to a project.
///
/// Project observations are contextual knowledge that applies to a specific
/// project rather than being universally true about an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectObservation {
    /// Unique identifier.
    pub id: Id,

    /// Parent project ID.
    pub project_id: Id,

    /// Semantic key for updates (optional).
    /// Format: `category.subcategory` (e.g., "architecture.auth", "decisions.api-design")
    pub key: Option<String>,

    /// The observation content.
    pub content: String,

    /// Embedding vector for semantic search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,

    /// Source of the observation.
    pub source: Option<String>,

    /// Creation timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,

    /// Last update timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl ProjectObservation {
    /// Create a new project observation.
    #[must_use]
    pub fn new(project_id: Id, content: impl Into<String>) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: Id::new(),
            project_id,
            key: None,
            content: content.into(),
            embedding: None,
            source: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Set the semantic key.
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

// ============================================================================
// Task Observation
// ============================================================================

/// An observation (fact/note) scoped to a task.
///
/// Task observations are contextual knowledge that applies to a specific
/// task rather than being universally true about an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskObservation {
    /// Unique identifier.
    pub id: Id,

    /// Parent task ID.
    pub task_id: Id,

    /// Semantic key for updates (optional).
    /// Format: `category.subcategory` (e.g., "gotchas.edge-case", "decisions.approach")
    pub key: Option<String>,

    /// The observation content.
    pub content: String,

    /// Embedding vector for semantic search.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,

    /// Source of the observation.
    pub source: Option<String>,

    /// Creation timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,

    /// Last update timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl TaskObservation {
    /// Create a new task observation.
    #[must_use]
    pub fn new(task_id: Id, content: impl Into<String>) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: Id::new(),
            task_id,
            key: None,
            content: content.into(),
            embedding: None,
            source: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Set the semantic key.
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

// ============================================================================
// Work Context
// ============================================================================

/// Session work context (which project/task a session is working on).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkContext {
    /// The session ID (from coordination layer).
    pub session_id: Id,

    /// Current project (optional).
    pub project_id: Option<Id>,

    /// Current task (optional).
    pub task_id: Option<Id>,

    /// When the session joined this work context.
    #[serde(with = "time::serde::rfc3339")]
    pub joined_at: OffsetDateTime,
}

impl WorkContext {
    /// Create a new work context.
    #[must_use]
    pub fn new(session_id: Id) -> Self {
        Self {
            session_id,
            project_id: None,
            task_id: None,
            joined_at: OffsetDateTime::now_utc(),
        }
    }

    /// Set the project.
    #[must_use]
    pub fn with_project(mut self, project_id: Id) -> Self {
        self.project_id = Some(project_id);
        self
    }

    /// Set the task.
    #[must_use]
    pub fn with_task(mut self, task_id: Id) -> Self {
        self.task_id = Some(task_id);
        self
    }
}

// ============================================================================
// Entity Connections
// ============================================================================

/// Relation type for project-entity connections.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProjectEntityRelation {
    /// Project involves this entity.
    #[default]
    Involves,
    /// Project depends on this entity.
    DependsOn,
    /// Project produces this entity.
    Produces,
    /// Custom relation.
    Custom(String),
}

impl std::fmt::Display for ProjectEntityRelation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Involves => write!(f, "involves"),
            Self::DependsOn => write!(f, "depends_on"),
            Self::Produces => write!(f, "produces"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl ProjectEntityRelation {
    /// Parse from string.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "involves" => Self::Involves,
            "depends_on" => Self::DependsOn,
            "produces" => Self::Produces,
            other => Self::Custom(other.to_string()),
        }
    }
}

/// Relation type for task-entity connections.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaskEntityRelation {
    /// Task touches this entity.
    #[default]
    Touches,
    /// Task modifies this entity.
    Modifies,
    /// Task creates this entity.
    Creates,
    /// Custom relation.
    Custom(String),
}

impl std::fmt::Display for TaskEntityRelation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Touches => write!(f, "touches"),
            Self::Modifies => write!(f, "modifies"),
            Self::Creates => write!(f, "creates"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl TaskEntityRelation {
    /// Parse from string.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "touches" => Self::Touches,
            "modifies" => Self::Modifies,
            "creates" => Self::Creates,
            other => Self::Custom(other.to_string()),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_creation() {
        let project = Project::new("my-project")
            .with_description("A test project")
            .with_status(ProjectStatus::Active);

        assert_eq!(project.name, "my-project");
        assert_eq!(project.description, Some("A test project".to_string()));
        assert_eq!(project.status, ProjectStatus::Active);
    }

    #[test]
    fn test_task_creation() {
        let project_id = Id::new();
        let task = Task::new(project_id, "implement-feature")
            .with_description("Implement the new feature")
            .with_jira_key("IDEAI-235")
            .with_priority(TaskPriority::High);

        assert_eq!(task.project_id, project_id);
        assert_eq!(task.name, "implement-feature");
        assert_eq!(task.jira_key, Some("IDEAI-235".to_string()));
        assert_eq!(task.priority, TaskPriority::High);
        assert_eq!(task.status, TaskStatus::Todo);
    }

    #[test]
    fn test_pr_creation() {
        let project_id = Id::new();
        let pr = Pr::new(
            project_id,
            "https://github.com/org/repo/pull/123",
            "repo",
            123,
        )
        .with_title("Add new feature");

        assert_eq!(pr.project_id, project_id);
        assert_eq!(pr.url, "https://github.com/org/repo/pull/123");
        assert_eq!(pr.repo, "repo");
        assert_eq!(pr.pr_number, 123);
        assert_eq!(pr.title, Some("Add new feature".to_string()));
        assert_eq!(pr.status, PrStatus::Open);
    }

    #[test]
    fn test_project_observation() {
        let project_id = Id::new();
        let obs = ProjectObservation::new(project_id, "Uses microservices architecture")
            .with_key("architecture.overview");

        assert_eq!(obs.project_id, project_id);
        assert_eq!(obs.content, "Uses microservices architecture");
        assert_eq!(obs.key, Some("architecture.overview".to_string()));
    }

    #[test]
    fn test_task_observation() {
        let task_id = Id::new();
        let obs = TaskObservation::new(task_id, "Watch out for race conditions")
            .with_key("gotchas.concurrency");

        assert_eq!(obs.task_id, task_id);
        assert_eq!(obs.content, "Watch out for race conditions");
        assert_eq!(obs.key, Some("gotchas.concurrency".to_string()));
    }

    #[test]
    fn test_status_parsing() {
        assert_eq!(ProjectStatus::parse("active"), ProjectStatus::Active);
        assert_eq!(ProjectStatus::parse("COMPLETED"), ProjectStatus::Completed);
        assert_eq!(TaskStatus::parse("in_progress"), TaskStatus::InProgress);
        assert_eq!(TaskStatus::parse("inprogress"), TaskStatus::InProgress);
        assert_eq!(PrStatus::parse("merged"), PrStatus::Merged);
    }
}
