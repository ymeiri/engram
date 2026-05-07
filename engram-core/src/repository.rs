//! Repository topology types.
//!
//! This module models Git repositories, local checkouts, monorepo components,
//! and their project connections. It is deliberately separate from Layer 1
//! `EntityType::Repo`: entities remain the general knowledge graph, while these
//! records capture source-control facts that orientation can resolve from `cwd`.

use crate::id::Id;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Source-control hosting provider.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RepositoryProvider {
    /// GitHub.
    GitHub,
    /// GitLab.
    GitLab,
    /// Bitbucket.
    Bitbucket,
    /// Unknown provider.
    #[default]
    Unknown,
    /// Custom provider label.
    Other(String),
}

impl std::fmt::Display for RepositoryProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GitHub => write!(f, "github"),
            Self::GitLab => write!(f, "gitlab"),
            Self::Bitbucket => write!(f, "bitbucket"),
            Self::Unknown => write!(f, "unknown"),
            Self::Other(value) => write!(f, "{value}"),
        }
    }
}

impl RepositoryProvider {
    /// Parse a provider label.
    #[must_use]
    pub fn parse(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "github" | "github.com" => Self::GitHub,
            "gitlab" | "gitlab.com" => Self::GitLab,
            "bitbucket" | "bitbucket.org" => Self::Bitbucket,
            "unknown" | "" => Self::Unknown,
            other => Self::Other(other.to_string()),
        }
    }

    /// Infer a provider from a remote URL.
    #[must_use]
    pub fn infer_from_remote(remote_url: &str) -> Self {
        let value = remote_url.to_lowercase();
        if value.contains("github.com") {
            Self::GitHub
        } else if value.contains("gitlab.com") {
            Self::GitLab
        } else if value.contains("bitbucket.org") {
            Self::Bitbucket
        } else {
            Self::Unknown
        }
    }
}

/// Canonical Git repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitRepository {
    /// Unique identifier.
    pub id: Id,
    /// Stable human-readable repository name.
    pub name: String,
    /// Canonical remote URL when known.
    pub remote_url: Option<String>,
    /// Hosting provider when known.
    pub provider: RepositoryProvider,
    /// Default branch when known.
    pub default_branch: Option<String>,
    /// Optional description.
    pub description: Option<String>,
    /// Creation timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// Last update timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl GitRepository {
    /// Create a repository record.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: Id::new(),
            name: name.into(),
            remote_url: None,
            provider: RepositoryProvider::Unknown,
            default_branch: None,
            description: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Set the remote URL and infer provider.
    #[must_use]
    pub fn with_remote_url(mut self, remote_url: impl Into<String>) -> Self {
        let remote_url = remote_url.into();
        self.provider = RepositoryProvider::infer_from_remote(&remote_url);
        self.remote_url = Some(remote_url);
        self
    }

    /// Set the provider explicitly.
    #[must_use]
    pub fn with_provider(mut self, provider: RepositoryProvider) -> Self {
        self.provider = provider;
        self
    }

    /// Set the default branch.
    #[must_use]
    pub fn with_default_branch(mut self, branch: impl Into<String>) -> Self {
        self.default_branch = Some(branch.into());
        self
    }

    /// Set the description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Mark the record as updated now.
    pub fn touch(&mut self) {
        self.updated_at = OffsetDateTime::now_utc();
    }
}

/// Local checkout of a Git repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalCheckout {
    /// Unique identifier.
    pub id: Id,
    /// Repository ID when associated.
    pub repository_id: Option<Id>,
    /// Absolute local checkout path.
    pub local_path: String,
    /// Current branch when known.
    pub current_branch: Option<String>,
    /// Current HEAD SHA when known.
    pub head_sha: Option<String>,
    /// Whether the checkout has uncommitted changes when last detected.
    pub is_dirty: Option<bool>,
    /// Creation timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// Last update timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    /// Last time this checkout was detected.
    #[serde(with = "time::serde::rfc3339")]
    pub last_seen_at: OffsetDateTime,
}

impl LocalCheckout {
    /// Create a local checkout record.
    #[must_use]
    pub fn new(local_path: impl Into<String>) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: Id::new(),
            repository_id: None,
            local_path: local_path.into(),
            current_branch: None,
            head_sha: None,
            is_dirty: None,
            created_at: now,
            updated_at: now,
            last_seen_at: now,
        }
    }

    /// Attach this checkout to a repository.
    #[must_use]
    pub fn with_repository(mut self, repository_id: Id) -> Self {
        self.repository_id = Some(repository_id);
        self
    }

    /// Update detected Git state.
    pub fn update_detected_state(
        &mut self,
        current_branch: Option<String>,
        head_sha: Option<String>,
        is_dirty: Option<bool>,
    ) {
        let now = OffsetDateTime::now_utc();
        self.current_branch = current_branch;
        self.head_sha = head_sha;
        self.is_dirty = is_dirty;
        self.updated_at = now;
        self.last_seen_at = now;
    }
}

/// Recent Git commit context for a local checkout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentGitCommit {
    /// Full commit SHA.
    pub sha: String,
    /// Commit subject line.
    pub summary: String,
    /// Changed paths reported for the commit, capped by the caller.
    pub changed_paths: Vec<String>,
}

impl RecentGitCommit {
    /// Create recent Git commit context.
    #[must_use]
    pub fn new(
        sha: impl Into<String>,
        summary: impl Into<String>,
        changed_paths: Vec<String>,
    ) -> Self {
        Self {
            sha: sha.into(),
            summary: summary.into(),
            changed_paths,
        }
    }
}

/// A component within a monorepo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonorepoComponent {
    /// Unique identifier.
    pub id: Id,
    /// Repository ID.
    pub repository_id: Id,
    /// Component name.
    pub name: String,
    /// Repository-relative component path.
    pub path: String,
    /// Optional component kind, such as service, app, package, or crate.
    pub kind: Option<String>,
    /// Optional description.
    pub description: Option<String>,
    /// Creation timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// Last update timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl MonorepoComponent {
    /// Create a monorepo component.
    #[must_use]
    pub fn new(repository_id: Id, name: impl Into<String>, path: impl Into<String>) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: Id::new(),
            repository_id,
            name: name.into(),
            path: normalize_component_path(path.into()),
            kind: None,
            description: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Set component kind.
    #[must_use]
    pub fn with_kind(mut self, kind: impl Into<String>) -> Self {
        self.kind = Some(kind.into());
        self
    }

    /// Set description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Relationship between a project and a repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProjectRepositoryRole {
    /// Main repository for the project.
    Primary,
    /// Repository the project depends on.
    Dependency,
    /// Repository produced by the project.
    Produces,
    /// Related repository.
    #[default]
    Related,
}

impl std::fmt::Display for ProjectRepositoryRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Primary => write!(f, "primary"),
            Self::Dependency => write!(f, "dependency"),
            Self::Produces => write!(f, "produces"),
            Self::Related => write!(f, "related"),
        }
    }
}

impl ProjectRepositoryRole {
    /// Parse from string.
    #[must_use]
    pub fn parse(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "primary" => Self::Primary,
            "dependency" | "depends_on" => Self::Dependency,
            "produces" | "output" => Self::Produces,
            "related" | "" => Self::Related,
            _ => Self::Related,
        }
    }
}

/// Project-to-repository connection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectRepositoryLink {
    /// Unique identifier.
    pub id: Id,
    /// Project ID when known.
    pub project_id: Option<Id>,
    /// Stable project name.
    pub project_name: String,
    /// Repository ID.
    pub repository_id: Id,
    /// Component ID when this link applies only to a monorepo component.
    #[serde(default)]
    pub component_id: Option<Id>,
    /// Repository-relative component path when this link applies only to a component.
    #[serde(default)]
    pub component_path: Option<String>,
    /// Role the repository plays for this project.
    pub role: ProjectRepositoryRole,
    /// Creation timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// Last update timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl ProjectRepositoryLink {
    /// Create a project-repository link.
    #[must_use]
    pub fn new(
        project_name: impl Into<String>,
        repository_id: Id,
        role: ProjectRepositoryRole,
    ) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: Id::new(),
            project_id: None,
            project_name: project_name.into(),
            repository_id,
            component_id: None,
            component_path: None,
            role,
            created_at: now,
            updated_at: now,
        }
    }

    /// Attach the link to a known project ID.
    #[must_use]
    pub fn with_project_id(mut self, project_id: Id) -> Self {
        self.project_id = Some(project_id);
        self
    }

    /// Scope the link to a monorepo component.
    #[must_use]
    pub fn with_component(mut self, component_id: Option<Id>, component_path: String) -> Self {
        self.component_id = component_id;
        self.component_path = Some(normalize_component_path(component_path));
        self
    }
}

/// Repository context resolved from a current working directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryContext {
    /// Matched repository.
    pub repository: GitRepository,
    /// Matched checkout.
    pub checkout: Option<LocalCheckout>,
    /// Recent current-branch commits, when requested by the caller.
    #[serde(default)]
    pub recent_commits: Vec<RecentGitCommit>,
    /// Monorepo components containing the current path.
    pub matching_components: Vec<MonorepoComponent>,
    /// Projects linked to this repository.
    pub linked_projects: Vec<ProjectRepositoryLink>,
}

fn normalize_component_path(path: String) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() || trimmed == "." {
        return ".".to_string();
    }
    trimmed
        .trim_start_matches("./")
        .trim_end_matches('/')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_infers_from_remote_url() {
        assert_eq!(
            RepositoryProvider::infer_from_remote("git@github.com:org/repo.git"),
            RepositoryProvider::GitHub
        );
        assert_eq!(
            RepositoryProvider::infer_from_remote("https://gitlab.com/org/repo"),
            RepositoryProvider::GitLab
        );
    }

    #[test]
    fn component_path_is_normalized() {
        let component = MonorepoComponent::new(Id::new(), "api", "./services/api/");
        assert_eq!(component.path, "services/api");
    }
}
