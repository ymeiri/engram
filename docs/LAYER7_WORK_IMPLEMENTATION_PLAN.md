# Layer 7: Work Management - Implementation Plan

**Status**: Approved design, ready for implementation
**Date**: 2025-02-05
**Context**: Cross-repository project/task tracking with knowledge sharing

---

## Overview

Layer 7 adds work management to Engram:
- **Projects**: Logical groupings of work spanning multiple repos
- **Tasks**: Work items (JIRA tickets, features) within projects
- **PRs**: Pull requests implementing tasks
- **Work Observations**: Project/task-scoped knowledge (separate from entity observations)

**Key Design Principle**: Entities (Layer 1) remain standalone. Projects/Tasks reference entities via junction tables. Observations are stored in the appropriate scope (entity=global, project/task=contextual).

---

## Data Model

### New Tables (Layer 7)

```sql
-- Projects
CREATE TABLE work_project (
    id: string PRIMARY KEY,      -- UUID v7
    name: string NOT NULL UNIQUE,
    description: string,
    status: string DEFAULT 'active',  -- planning, active, completed, archived
    created_at: datetime,
    updated_at: datetime
);
CREATE INDEX idx_work_project_name ON work_project(name);
CREATE INDEX idx_work_project_status ON work_project(status);

-- Tasks
CREATE TABLE work_task (
    id: string PRIMARY KEY,
    project_id: string NOT NULL,  -- FK to work_project
    name: string NOT NULL,
    description: string,
    status: string DEFAULT 'todo',  -- todo, in_progress, blocked, done
    priority: string DEFAULT 'medium',  -- low, medium, high, critical
    jira_key: string,  -- e.g., "IDEAI-235"
    blocked_by: array<string>,  -- task IDs
    created_at: datetime,
    updated_at: datetime
);
CREATE INDEX idx_work_task_project ON work_task(project_id);
CREATE INDEX idx_work_task_status ON work_task(status);
CREATE INDEX idx_work_task_jira ON work_task(jira_key);

-- PRs
CREATE TABLE work_pr (
    id: string PRIMARY KEY,
    task_id: string,  -- FK to work_task (nullable - can be directly on project)
    project_id: string NOT NULL,  -- FK to work_project
    url: string NOT NULL UNIQUE,
    repo: string NOT NULL,  -- e.g., "my-project", "web-ui"
    pr_number: int NOT NULL,
    title: string,
    status: string DEFAULT 'open',  -- open, merged, closed
    blocked_by: array<string>,  -- PR IDs
    created_at: datetime,
    updated_at: datetime
);
CREATE INDEX idx_work_pr_task ON work_pr(task_id);
CREATE INDEX idx_work_pr_project ON work_pr(project_id);
CREATE INDEX idx_work_pr_repo ON work_pr(repo);
CREATE INDEX idx_work_pr_url ON work_pr(url);

-- Junction: Project <-> Entity (many-to-many)
CREATE TABLE work_project_entity (
    project_id: string NOT NULL,
    entity_id: string NOT NULL,
    relation: string DEFAULT 'involves',  -- involves, depends_on, produces
    PRIMARY KEY (project_id, entity_id)
);
CREATE INDEX idx_wpe_project ON work_project_entity(project_id);
CREATE INDEX idx_wpe_entity ON work_project_entity(entity_id);

-- Junction: Task <-> Entity (many-to-many)
CREATE TABLE work_task_entity (
    task_id: string NOT NULL,
    entity_id: string NOT NULL,
    relation: string DEFAULT 'touches',  -- touches, modifies, creates
    PRIMARY KEY (task_id, entity_id)
);
CREATE INDEX idx_wte_task ON work_task_entity(task_id);
CREATE INDEX idx_wte_entity ON work_task_entity(entity_id);

-- Project Observations (project-scoped knowledge)
CREATE TABLE work_project_observation (
    id: string PRIMARY KEY,
    project_id: string NOT NULL,
    key: string,  -- semantic key, same pattern as entity observations
    content: string NOT NULL,
    embedding: array<float>,  -- 384 dimensions
    source: string,
    created_at: datetime,
    updated_at: datetime
);
CREATE INDEX idx_wpo_project ON work_project_observation(project_id);
CREATE INDEX idx_wpo_key ON work_project_observation(key);
CREATE INDEX idx_wpo_project_key ON work_project_observation(project_id, key);

-- Task Observations (task-scoped knowledge)
CREATE TABLE work_task_observation (
    id: string PRIMARY KEY,
    task_id: string NOT NULL,
    key: string,
    content: string NOT NULL,
    embedding: array<float>,
    source: string,
    created_at: datetime,
    updated_at: datetime
);
CREATE INDEX idx_wto_task ON work_task_observation(task_id);
CREATE INDEX idx_wto_key ON work_task_observation(key);

-- Session <-> Work Context (which project/task a session is working on)
-- Extends existing coordination_session or separate table
CREATE TABLE work_session_context (
    session_id: string PRIMARY KEY,  -- coordination session ID
    project_id: string,
    task_id: string,
    joined_at: datetime
);
```

---

## Crate Changes

### 1. engram-core (new types)

**File**: `engram-core/src/work.rs` (new)

```rust
// Project status
pub enum ProjectStatus { Planning, Active, Completed, Archived }

// Task status
pub enum TaskStatus { Todo, InProgress, Blocked, Done }

// Task priority
pub enum TaskPriority { Low, Medium, High, Critical }

// PR status
pub enum PrStatus { Open, Merged, Closed }

// Project
pub struct Project {
    pub id: Id,
    pub name: String,
    pub description: Option<String>,
    pub status: ProjectStatus,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

// Task
pub struct Task {
    pub id: Id,
    pub project_id: Id,
    pub name: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub jira_key: Option<String>,
    pub blocked_by: Vec<Id>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

// PR
pub struct Pr {
    pub id: Id,
    pub task_id: Option<Id>,
    pub project_id: Id,
    pub url: String,
    pub repo: String,
    pub pr_number: u32,
    pub title: Option<String>,
    pub status: PrStatus,
    pub blocked_by: Vec<Id>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

// Project Observation
pub struct ProjectObservation {
    pub id: Id,
    pub project_id: Id,
    pub key: Option<String>,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub source: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

// Task Observation
pub struct TaskObservation {
    pub id: Id,
    pub task_id: Id,
    pub key: Option<String>,
    pub content: String,
    pub embedding: Option<Vec<f32>>,
    pub source: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

// Work Context (for sessions)
pub struct WorkContext {
    pub session_id: Id,
    pub project_id: Option<Id>,
    pub task_id: Option<Id>,
    pub joined_at: OffsetDateTime,
}
```

**Update**: `engram-core/src/lib.rs` - add `pub mod work;`

### 2. engram-store (new repo)

**File**: `engram-store/src/repos/work.rs` (new)

```rust
pub struct WorkRepo { db: Db }

impl WorkRepo {
    // Schema
    pub async fn init_schema(&self) -> StoreResult<()>;

    // Projects
    pub async fn create_project(&self, project: &Project) -> StoreResult<()>;
    pub async fn get_project(&self, id: &Id) -> StoreResult<Option<Project>>;
    pub async fn get_project_by_name(&self, name: &str) -> StoreResult<Option<Project>>;
    pub async fn list_projects(&self, status: Option<ProjectStatus>) -> StoreResult<Vec<Project>>;
    pub async fn update_project(&self, project: &Project) -> StoreResult<()>;

    // Tasks
    pub async fn create_task(&self, task: &Task) -> StoreResult<()>;
    pub async fn get_task(&self, id: &Id) -> StoreResult<Option<Task>>;
    pub async fn get_task_by_jira(&self, jira_key: &str) -> StoreResult<Option<Task>>;
    pub async fn list_tasks(&self, project_id: &Id, status: Option<TaskStatus>) -> StoreResult<Vec<Task>>;
    pub async fn update_task(&self, task: &Task) -> StoreResult<()>;

    // PRs
    pub async fn create_pr(&self, pr: &Pr) -> StoreResult<()>;
    pub async fn get_pr(&self, id: &Id) -> StoreResult<Option<Pr>>;
    pub async fn get_pr_by_url(&self, url: &str) -> StoreResult<Option<Pr>>;
    pub async fn list_prs(&self, project_id: Option<&Id>, task_id: Option<&Id>) -> StoreResult<Vec<Pr>>;
    pub async fn update_pr(&self, pr: &Pr) -> StoreResult<()>;

    // Entity connections
    pub async fn connect_project_entity(&self, project_id: &Id, entity_id: &Id, relation: &str) -> StoreResult<()>;
    pub async fn disconnect_project_entity(&self, project_id: &Id, entity_id: &Id) -> StoreResult<()>;
    pub async fn get_project_entities(&self, project_id: &Id) -> StoreResult<Vec<(Id, String)>>; // (entity_id, relation)
    pub async fn get_entity_projects(&self, entity_id: &Id) -> StoreResult<Vec<Id>>;

    pub async fn connect_task_entity(&self, task_id: &Id, entity_id: &Id, relation: &str) -> StoreResult<()>;
    pub async fn disconnect_task_entity(&self, task_id: &Id, entity_id: &Id) -> StoreResult<()>;
    pub async fn get_task_entities(&self, task_id: &Id) -> StoreResult<Vec<(Id, String)>>;

    // Observations
    pub async fn add_project_observation(&self, obs: &ProjectObservation) -> StoreResult<Option<ProjectObservation>>;
    pub async fn get_project_observations(&self, project_id: &Id) -> StoreResult<Vec<ProjectObservation>>;
    pub async fn search_project_observations(&self, project_id: &Id, query: &str) -> StoreResult<Vec<ProjectObservation>>;
    pub async fn search_project_observations_by_embedding(&self, project_id: &Id, embedding: &[f32], limit: usize, min_score: f32) -> StoreResult<Vec<(ProjectObservation, f32)>>;

    pub async fn add_task_observation(&self, obs: &TaskObservation) -> StoreResult<Option<TaskObservation>>;
    pub async fn get_task_observations(&self, task_id: &Id) -> StoreResult<Vec<TaskObservation>>;
    pub async fn search_task_observations_by_embedding(&self, task_id: &Id, embedding: &[f32], limit: usize, min_score: f32) -> StoreResult<Vec<(TaskObservation, f32)>>;

    // Work context
    pub async fn set_session_context(&self, ctx: &WorkContext) -> StoreResult<()>;
    pub async fn get_session_context(&self, session_id: &Id) -> StoreResult<Option<WorkContext>>;
    pub async fn clear_session_context(&self, session_id: &Id) -> StoreResult<()>;
}
```

**Update**: `engram-store/src/repos.rs` - add `pub mod work; pub use work::WorkRepo;`
**Update**: `engram-store/src/lib.rs` - add WorkRepo to exports

### 3. engram-index (new service)

**File**: `engram-index/src/work.rs` (new)

```rust
pub struct WorkService {
    repo: WorkRepo,
    entity_repo: EntityRepo,
    embedder: Option<Embedder>,
}

impl WorkService {
    pub fn new(db: Db) -> Self;
    pub fn with_embedder(db: Db, embedder: Embedder) -> Self;
    pub fn with_defaults(db: Db) -> IndexResult<Self>;

    pub async fn init(&self) -> IndexResult<()>;

    // Projects
    pub async fn create_project(&self, name: &str, description: Option<&str>) -> IndexResult<Project>;
    pub async fn get_project(&self, name: &str) -> IndexResult<Option<Project>>;
    pub async fn list_projects(&self, status: Option<ProjectStatus>) -> IndexResult<Vec<Project>>;
    pub async fn update_project_status(&self, name: &str, status: ProjectStatus) -> IndexResult<()>;

    // Tasks
    pub async fn create_task(&self, project_name: &str, task_name: &str, description: Option<&str>, jira_key: Option<&str>) -> IndexResult<Task>;
    pub async fn get_task(&self, name_or_jira: &str) -> IndexResult<Option<Task>>;
    pub async fn list_tasks(&self, project_name: &str, status: Option<TaskStatus>) -> IndexResult<Vec<Task>>;
    pub async fn update_task_status(&self, name_or_jira: &str, status: TaskStatus) -> IndexResult<()>;
    pub async fn set_task_blocked_by(&self, task: &str, blocked_by: &[&str]) -> IndexResult<()>;

    // PRs
    pub async fn add_pr(&self, project_name: &str, task_name: Option<&str>, url: &str, title: Option<&str>) -> IndexResult<Pr>;
    pub async fn get_pr(&self, url: &str) -> IndexResult<Option<Pr>>;
    pub async fn list_prs(&self, project_name: &str, task_name: Option<&str>) -> IndexResult<Vec<Pr>>;
    pub async fn update_pr_status(&self, url: &str, status: PrStatus) -> IndexResult<()>;
    pub async fn set_pr_blocked_by(&self, pr_url: &str, blocked_by_urls: &[&str]) -> IndexResult<()>;

    // Entity connections
    pub async fn connect_project_to_entity(&self, project_name: &str, entity_name: &str, relation: Option<&str>) -> IndexResult<()>;
    pub async fn connect_task_to_entity(&self, task_name: &str, entity_name: &str, relation: Option<&str>) -> IndexResult<()>;
    pub async fn get_project_entities(&self, project_name: &str) -> IndexResult<Vec<Entity>>;
    pub async fn get_task_entities(&self, task_name: &str) -> IndexResult<Vec<Entity>>;

    // Observations
    pub async fn add_project_observation(&self, project_name: &str, content: &str, key: Option<&str>) -> IndexResult<ProjectObservation>;
    pub async fn add_task_observation(&self, task_name: &str, content: &str, key: Option<&str>) -> IndexResult<TaskObservation>;
    pub async fn graduate_observation(&self, from: GraduateFrom, observation_id: &Id, to_entity: &str, key: Option<&str>) -> IndexResult<Observation>;

    // Work context for sessions
    pub async fn join_work(&self, session_id: &Id, project_name: &str, task_name: Option<&str>) -> IndexResult<WorkContext>;
    pub async fn leave_work(&self, session_id: &Id) -> IndexResult<()>;
    pub async fn get_work_context(&self, session_id: &Id) -> IndexResult<Option<WorkContext>>;

    // Aggregated context (for agent startup)
    pub async fn get_full_context(&self, project_name: &str, task_name: Option<&str>) -> IndexResult<FullWorkContext>;
}

pub enum GraduateFrom { Project, Task }

pub struct FullWorkContext {
    pub project: Project,
    pub task: Option<Task>,
    pub prs: Vec<Pr>,
    pub project_observations: Vec<ProjectObservation>,
    pub task_observations: Vec<TaskObservation>,
    pub connected_entities: Vec<Entity>,
    pub entity_observations: Vec<Observation>,  // from connected entities
}
```

**Update**: `engram-index/src/lib.rs` - add `pub mod work; pub use work::WorkService;`

### 4. engram-mcp (new tools)

**File**: `engram-mcp/src/tools.rs` - add work tools

New MCP tools to add:

```
# Project Management
- work_project_create(name, description?) -> Project
- work_project_get(name) -> Project with tasks, PRs, entities
- work_project_list(status?) -> Vec<Project>
- work_project_update_status(name, status) -> Project
- work_project_connect_entity(project, entity, relation?) -> ()

# Task Management
- work_task_create(project, name, description?, jira_key?) -> Task
- work_task_get(name_or_jira) -> Task with PRs, entities
- work_task_list(project, status?) -> Vec<Task>
- work_task_update_status(name_or_jira, status) -> Task
- work_task_set_blocked_by(task, blocked_by[]) -> Task
- work_task_connect_entity(task, entity, relation?) -> ()

# PR Management
- work_pr_add(project, url, task?, title?) -> Pr
- work_pr_get(url) -> Pr
- work_pr_list(project, task?) -> Vec<Pr>
- work_pr_update_status(url, status) -> Pr
- work_pr_set_blocked_by(url, blocked_by_urls[]) -> Pr

# Work Observations
- work_project_observe(project, content, key?) -> ProjectObservation
- work_task_observe(task, content, key?) -> TaskObservation
- work_graduate(from: "project"|"task", observation_id, to_entity, key?) -> Observation

# Session Work Context
- work_join(project, task?) -> FullWorkContext
- work_leave() -> ()
- work_context() -> current WorkContext or null

# Aggregated Query
- work_get_context(project, task?) -> FullWorkContext
```

### 5. engram-cli (new commands)

```
engram work project create <name> [--description <desc>]
engram work project list [--status <status>]
engram work project show <name>
engram work project connect <project> <entity> [--relation <rel>]

engram work task create <project> <name> [--jira <key>] [--description <desc>]
engram work task list <project> [--status <status>]
engram work task show <name_or_jira>
engram work task status <name_or_jira> <status>
engram work task connect <task> <entity>

engram work pr add <project> <url> [--task <task>] [--title <title>]
engram work pr list <project> [--task <task>]
engram work pr status <url> <status>

engram work observe project <project> <content> [--key <key>]
engram work observe task <task> <content> [--key <key>]
engram work graduate <project|task> <observation_id> --to-entity <entity> [--key <key>]
```

---

## Implementation Order

### Phase 1: Core Types & Storage
1. [ ] Add `engram-core/src/work.rs` with all types
2. [ ] Add `engram-store/src/repos/work.rs` with WorkRepo
3. [ ] Update exports in both crates
4. [ ] Write tests for WorkRepo

### Phase 2: Service Layer
5. [ ] Add `engram-index/src/work.rs` with WorkService
6. [ ] Implement embedding generation for work observations
7. [ ] Write tests for WorkService

### Phase 3: MCP Tools
8. [ ] Add work tool request/response types to `engram-mcp/src/tools.rs`
9. [ ] Add tool implementations
10. [ ] Update ToolState to include WorkService
11. [ ] Register tools in server.rs

### Phase 4: CLI
12. [ ] Add work subcommands to main.rs
13. [ ] Test CLI commands

### Phase 5: Integration
14. [ ] Update daemon initialization to include WorkService
15. [ ] Integration tests
16. [ ] Update CLAUDE.md documentation with work tools

---

## Migration

**No data migration required.** Layer 7 is purely additive:
- Existing entities remain unchanged
- Existing observations remain on entities
- New tables are created empty
- Users can start creating projects/tasks immediately

---

## Open Questions (resolved)

1. ✅ Entities are standalone, projects/tasks reference them (not vice versa)
2. ✅ Observations: entity=global, project/task=scoped
3. ✅ Graduation: explicit copy from project/task observation to entity observation
4. ✅ Session context: work_join/work_leave to set current project/task

---

## Notes for Implementation

- Use UUID v7 for all IDs (time-sortable)
- Generate embeddings for all observations (project and task)
- Semantic keys follow same pattern as entity observations: `category.subcategory`
- PR URLs should be normalized (handle various GitHub URL formats)
- JIRA keys are optional but useful for lookup
- blocked_by arrays contain IDs, resolve names in service layer
