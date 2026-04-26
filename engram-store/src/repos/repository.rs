//! Repository topology persistence.

use crate::error::{StoreError, StoreResult};
use crate::Db;
use engram_core::id::Id;
use engram_core::repository::{
    GitRepository, LocalCheckout, MonorepoComponent, ProjectRepositoryLink,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tracing::{debug, info};

#[derive(Debug, Clone, Deserialize)]
struct RepositoryRecord {
    record_id: String,
    repository: serde_json::Value,
}

impl RepositoryRecord {
    fn into_repository(self) -> StoreResult<GitRepository> {
        let mut repository: GitRepository = from_json(self.repository)?;
        repository.id = parse_id(&self.record_id, "repository")?;
        Ok(repository)
    }
}

#[derive(Debug, Clone, Deserialize)]
struct CheckoutRecord {
    record_id: String,
    checkout: serde_json::Value,
}

impl CheckoutRecord {
    fn into_checkout(self) -> StoreResult<LocalCheckout> {
        let mut checkout: LocalCheckout = from_json(self.checkout)?;
        checkout.id = parse_id(&self.record_id, "checkout")?;
        Ok(checkout)
    }
}

#[derive(Debug, Clone, Deserialize)]
struct ComponentRecord {
    record_id: String,
    component: serde_json::Value,
}

impl ComponentRecord {
    fn into_component(self) -> StoreResult<MonorepoComponent> {
        let mut component: MonorepoComponent = from_json(self.component)?;
        component.id = parse_id(&self.record_id, "component")?;
        Ok(component)
    }
}

#[derive(Debug, Clone, Deserialize)]
struct ProjectLinkRecord {
    record_id: String,
    link: serde_json::Value,
}

impl ProjectLinkRecord {
    fn into_link(self) -> StoreResult<ProjectRepositoryLink> {
        let mut link: ProjectRepositoryLink = from_json(self.link)?;
        link.id = parse_id(&self.record_id, "project repository link")?;
        Ok(link)
    }
}

/// Repository for Git topology records.
#[derive(Clone)]
pub struct RepositoryRepo {
    db: Db,
}

impl RepositoryRepo {
    /// Create a new repository topology repository.
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Initialize schema.
    pub async fn init_schema(&self) -> StoreResult<()> {
        info!("Initializing repository topology schema");

        self.db
            .query(
                r#"
                DEFINE TABLE IF NOT EXISTS git_repository SCHEMALESS;
                DEFINE INDEX IF NOT EXISTS idx_git_repository_name ON git_repository FIELDS name_key;
                DEFINE INDEX IF NOT EXISTS idx_git_repository_remote ON git_repository FIELDS remote_url;
                DEFINE INDEX IF NOT EXISTS idx_git_repository_provider ON git_repository FIELDS provider_key;
                DEFINE INDEX IF NOT EXISTS idx_git_repository_updated ON git_repository FIELDS updated_at;

                DEFINE TABLE IF NOT EXISTS local_checkout SCHEMALESS;
                DEFINE INDEX IF NOT EXISTS idx_local_checkout_repository ON local_checkout FIELDS repository_id;
                DEFINE INDEX IF NOT EXISTS idx_local_checkout_path ON local_checkout FIELDS local_path_key;
                DEFINE INDEX IF NOT EXISTS idx_local_checkout_last_seen ON local_checkout FIELDS last_seen_at;

                DEFINE TABLE IF NOT EXISTS monorepo_component SCHEMALESS;
                DEFINE INDEX IF NOT EXISTS idx_monorepo_component_repository ON monorepo_component FIELDS repository_id;
                DEFINE INDEX IF NOT EXISTS idx_monorepo_component_path ON monorepo_component FIELDS path_key;

                DEFINE TABLE IF NOT EXISTS project_repository_link SCHEMALESS;
                DEFINE INDEX IF NOT EXISTS idx_project_repository_link_repository ON project_repository_link FIELDS repository_id;
                DEFINE INDEX IF NOT EXISTS idx_project_repository_link_project ON project_repository_link FIELDS project_name_key;
                DEFINE INDEX IF NOT EXISTS idx_project_repository_link_component ON project_repository_link FIELDS component_path_key;
                "#,
            )
            .await?;

        info!("Repository topology schema initialized");
        Ok(())
    }

    /// Save a Git repository.
    pub async fn save_repository(&self, repository: &GitRepository) -> StoreResult<()> {
        debug!("Saving git repository: {}", repository.id);

        self.db
            .query(
                r#"
                UPSERT type::thing("git_repository", $id) SET
                    repository = $repository,
                    name_key = $name_key,
                    remote_url = $remote_url,
                    provider_key = $provider_key,
                    created_at = $created_at,
                    updated_at = $updated_at
                "#,
            )
            .bind(("id", repository.id.to_string()))
            .bind(("repository", to_json(repository)?))
            .bind(("name_key", repository.name.to_lowercase()))
            .bind(("remote_url", repository.remote_url.clone()))
            .bind(("provider_key", repository.provider.to_string()))
            .bind(("created_at", format_rfc3339(repository.created_at)?))
            .bind(("updated_at", format_rfc3339(repository.updated_at)?))
            .await?;

        Ok(())
    }

    /// Get a repository by ID.
    pub async fn get_repository(&self, id: &Id) -> StoreResult<Option<GitRepository>> {
        let mut result = self
            .db
            .query(
                r#"
                SELECT meta::id(id) AS record_id, repository
                FROM type::thing("git_repository", $id)
                "#,
            )
            .bind(("id", id.to_string()))
            .await?;

        let records: Vec<RepositoryRecord> = result.take(0)?;
        records
            .into_iter()
            .next()
            .map(RepositoryRecord::into_repository)
            .transpose()
    }

    /// Get a repository by case-insensitive name.
    pub async fn get_repository_by_name(&self, name: &str) -> StoreResult<Option<GitRepository>> {
        let mut result = self
            .db
            .query(
                r#"
                SELECT meta::id(id) AS record_id, repository
                FROM git_repository
                WHERE name_key = $name
                LIMIT 1
                "#,
            )
            .bind(("name", name.to_lowercase()))
            .await?;

        let records: Vec<RepositoryRecord> = result.take(0)?;
        records
            .into_iter()
            .next()
            .map(RepositoryRecord::into_repository)
            .transpose()
    }

    /// Get a repository by remote URL.
    pub async fn get_repository_by_remote_url(
        &self,
        remote_url: &str,
    ) -> StoreResult<Option<GitRepository>> {
        let mut result = self
            .db
            .query(
                r#"
                SELECT meta::id(id) AS record_id, repository
                FROM git_repository
                WHERE remote_url = $remote_url
                LIMIT 1
                "#,
            )
            .bind(("remote_url", remote_url.to_string()))
            .await?;

        let records: Vec<RepositoryRecord> = result.take(0)?;
        records
            .into_iter()
            .next()
            .map(RepositoryRecord::into_repository)
            .transpose()
    }

    /// List repositories newest-updated first.
    pub async fn list_repositories(&self, limit: Option<usize>) -> StoreResult<Vec<GitRepository>> {
        let mut query = r#"
            SELECT meta::id(id) AS record_id, repository, updated_at
            FROM git_repository
            ORDER BY updated_at DESC
        "#
        .to_string();
        if let Some(limit) = limit {
            query.push_str(&format!(" LIMIT {limit}"));
        }

        let mut result = self.db.query(query).await?;
        decode_repositories(result.take(0)?)
    }

    /// Save a local checkout.
    pub async fn save_checkout(&self, checkout: &LocalCheckout) -> StoreResult<()> {
        debug!("Saving local checkout: {}", checkout.id);

        self.db
            .query(
                r#"
                UPSERT type::thing("local_checkout", $id) SET
                    checkout = $checkout,
                    repository_id = $repository_id,
                    local_path_key = $local_path_key,
                    current_branch = $current_branch,
                    head_sha = $head_sha,
                    is_dirty = $is_dirty,
                    created_at = $created_at,
                    updated_at = $updated_at,
                    last_seen_at = $last_seen_at
                "#,
            )
            .bind(("id", checkout.id.to_string()))
            .bind(("checkout", to_json(checkout)?))
            .bind((
                "repository_id",
                checkout.repository_id.map(|id| id.to_string()),
            ))
            .bind(("local_path_key", checkout.local_path.clone()))
            .bind(("current_branch", checkout.current_branch.clone()))
            .bind(("head_sha", checkout.head_sha.clone()))
            .bind(("is_dirty", checkout.is_dirty))
            .bind(("created_at", format_rfc3339(checkout.created_at)?))
            .bind(("updated_at", format_rfc3339(checkout.updated_at)?))
            .bind(("last_seen_at", format_rfc3339(checkout.last_seen_at)?))
            .await?;

        Ok(())
    }

    /// Get a checkout by exact local path.
    pub async fn get_checkout_by_path(
        &self,
        local_path: &str,
    ) -> StoreResult<Option<LocalCheckout>> {
        let mut result = self
            .db
            .query(
                r#"
                SELECT meta::id(id) AS record_id, checkout
                FROM local_checkout
                WHERE local_path_key = $local_path
                LIMIT 1
                "#,
            )
            .bind(("local_path", local_path.to_string()))
            .await?;

        let records: Vec<CheckoutRecord> = result.take(0)?;
        records
            .into_iter()
            .next()
            .map(CheckoutRecord::into_checkout)
            .transpose()
    }

    /// List all known checkouts.
    pub async fn list_checkouts(&self) -> StoreResult<Vec<LocalCheckout>> {
        let mut result = self
            .db
            .query(
                r#"
                SELECT meta::id(id) AS record_id, checkout, last_seen_at
                FROM local_checkout
                ORDER BY last_seen_at DESC
                "#,
            )
            .await?;

        decode_checkouts(result.take(0)?)
    }

    /// Save a monorepo component.
    pub async fn save_component(&self, component: &MonorepoComponent) -> StoreResult<()> {
        debug!("Saving monorepo component: {}", component.id);

        self.db
            .query(
                r#"
                UPSERT type::thing("monorepo_component", $id) SET
                    component = $component,
                    repository_id = $repository_id,
                    name_key = $name_key,
                    path_key = $path_key,
                    kind = $kind,
                    created_at = $created_at,
                    updated_at = $updated_at
                "#,
            )
            .bind(("id", component.id.to_string()))
            .bind(("component", to_json(component)?))
            .bind(("repository_id", component.repository_id.to_string()))
            .bind(("name_key", component.name.to_lowercase()))
            .bind(("path_key", component.path.clone()))
            .bind(("kind", component.kind.clone()))
            .bind(("created_at", format_rfc3339(component.created_at)?))
            .bind(("updated_at", format_rfc3339(component.updated_at)?))
            .await?;

        Ok(())
    }

    /// Get a component by repository and path.
    pub async fn get_component_by_path(
        &self,
        repository_id: &Id,
        path: &str,
    ) -> StoreResult<Option<MonorepoComponent>> {
        let mut result = self
            .db
            .query(
                r#"
                SELECT meta::id(id) AS record_id, component
                FROM monorepo_component
                WHERE repository_id = $repository_id AND path_key = $path
                LIMIT 1
                "#,
            )
            .bind(("repository_id", repository_id.to_string()))
            .bind(("path", path.to_string()))
            .await?;

        let records: Vec<ComponentRecord> = result.take(0)?;
        records
            .into_iter()
            .next()
            .map(ComponentRecord::into_component)
            .transpose()
    }

    /// List components for a repository.
    pub async fn list_components(&self, repository_id: &Id) -> StoreResult<Vec<MonorepoComponent>> {
        let mut result = self
            .db
            .query(
                r#"
                SELECT meta::id(id) AS record_id, component, path_key
                FROM monorepo_component
                WHERE repository_id = $repository_id
                ORDER BY path_key ASC
                "#,
            )
            .bind(("repository_id", repository_id.to_string()))
            .await?;

        decode_components(result.take(0)?)
    }

    /// Save a project-repository link.
    pub async fn save_project_link(&self, link: &ProjectRepositoryLink) -> StoreResult<()> {
        debug!("Saving project repository link: {}", link.id);

        self.db
            .query(
                r#"
                UPSERT type::thing("project_repository_link", $id) SET
                    link = $link,
                    project_id = $project_id,
                    project_name_key = $project_name_key,
                    repository_id = $repository_id,
                    component_id = $component_id,
                    component_path_key = $component_path_key,
                    role = $role,
                    created_at = $created_at,
                    updated_at = $updated_at
                "#,
            )
            .bind(("id", link.id.to_string()))
            .bind(("link", to_json(link)?))
            .bind(("project_id", link.project_id.map(|id| id.to_string())))
            .bind(("project_name_key", link.project_name.to_lowercase()))
            .bind(("repository_id", link.repository_id.to_string()))
            .bind(("component_id", link.component_id.map(|id| id.to_string())))
            .bind(("component_path_key", link.component_path.clone()))
            .bind(("role", link.role.to_string()))
            .bind(("created_at", format_rfc3339(link.created_at)?))
            .bind(("updated_at", format_rfc3339(link.updated_at)?))
            .await?;

        Ok(())
    }

    /// Get a project-repository link by project name and repository.
    pub async fn get_project_link(
        &self,
        project_name: &str,
        repository_id: &Id,
    ) -> StoreResult<Option<ProjectRepositoryLink>> {
        Ok(self
            .list_project_links(repository_id)
            .await?
            .into_iter()
            .find(|link| link.project_name.eq_ignore_ascii_case(project_name)))
    }

    /// List project links for a repository.
    pub async fn list_project_links(
        &self,
        repository_id: &Id,
    ) -> StoreResult<Vec<ProjectRepositoryLink>> {
        let mut result = self
            .db
            .query(
                r#"
                SELECT meta::id(id) AS record_id, link, project_name_key, component_path_key
                FROM project_repository_link
                WHERE repository_id = $repository_id
                ORDER BY project_name_key ASC, component_path_key ASC
                "#,
            )
            .bind(("repository_id", repository_id.to_string()))
            .await?;

        decode_project_links(result.take(0)?)
    }
}

fn decode_repositories(records: Vec<RepositoryRecord>) -> StoreResult<Vec<GitRepository>> {
    records
        .into_iter()
        .map(RepositoryRecord::into_repository)
        .collect()
}

fn decode_checkouts(records: Vec<CheckoutRecord>) -> StoreResult<Vec<LocalCheckout>> {
    records
        .into_iter()
        .map(CheckoutRecord::into_checkout)
        .collect()
}

fn decode_components(records: Vec<ComponentRecord>) -> StoreResult<Vec<MonorepoComponent>> {
    records
        .into_iter()
        .map(ComponentRecord::into_component)
        .collect()
}

fn decode_project_links(
    records: Vec<ProjectLinkRecord>,
) -> StoreResult<Vec<ProjectRepositoryLink>> {
    records
        .into_iter()
        .map(ProjectLinkRecord::into_link)
        .collect()
}

fn parse_id(value: &str, label: &str) -> StoreResult<Id> {
    Id::parse(value).map_err(|e| StoreError::Deserialization(format!("Invalid {label} ID: {e}")))
}

fn format_rfc3339(value: OffsetDateTime) -> StoreResult<String> {
    value
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| StoreError::Deserialization(format!("Invalid timestamp: {e}")))
}

fn to_json<T: Serialize>(value: &T) -> StoreResult<serde_json::Value> {
    serde_json::to_value(value).map_err(StoreError::Serialization)
}

fn from_json<T: DeserializeOwned>(value: serde_json::Value) -> StoreResult<T> {
    serde_json::from_value(value).map_err(StoreError::Serialization)
}

#[cfg(test)]
mod tests {
    use super::*;
    use engram_core::repository::{ProjectRepositoryRole, RepositoryProvider};

    async fn setup_repo() -> RepositoryRepo {
        let config = crate::StoreConfig::memory();
        let db = crate::connect_and_init(&config).await.unwrap();
        let repo = RepositoryRepo::new(db);
        repo.init_schema().await.unwrap();
        repo
    }

    #[tokio::test]
    async fn save_and_get_repository_by_name_and_remote() {
        let repo = setup_repo().await;
        let repository =
            GitRepository::new("engram").with_remote_url("git@github.com:ymeiri/engram.git");

        repo.save_repository(&repository).await.unwrap();

        let by_name = repo
            .get_repository_by_name("Engram")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(by_name.id, repository.id);
        assert_eq!(by_name.provider, RepositoryProvider::GitHub);

        let by_remote = repo
            .get_repository_by_remote_url("git@github.com:ymeiri/engram.git")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(by_remote.id, repository.id);
    }

    #[tokio::test]
    async fn checkout_components_and_project_links_round_trip() {
        let repo = setup_repo().await;
        let repository = GitRepository::new("mono");
        repo.save_repository(&repository).await.unwrap();

        let checkout = LocalCheckout::new("/tmp/mono").with_repository(repository.id);
        repo.save_checkout(&checkout).await.unwrap();

        let component =
            MonorepoComponent::new(repository.id, "api", "services/api").with_kind("service");
        repo.save_component(&component).await.unwrap();

        let link = ProjectRepositoryLink::new(
            "Debug with AI",
            repository.id,
            ProjectRepositoryRole::Primary,
        );
        repo.save_project_link(&link).await.unwrap();

        let stored_checkout = repo
            .get_checkout_by_path("/tmp/mono")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(stored_checkout.repository_id, Some(repository.id));

        let components = repo.list_components(&repository.id).await.unwrap();
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].path, "services/api");

        let links = repo.list_project_links(&repository.id).await.unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].project_name, "Debug with AI");
    }
}
