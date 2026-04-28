//! Derived Memory OS graph traversal.

use crate::error::IndexResult;
use engram_core::graph::{MemoryEdge, MemoryGraphPath, MemoryNode, MemorySubgraph};
use engram_core::memory::{EvidenceKind, KnowledgeCommit, MemoryItem, MemoryScope};
use engram_core::repository::{
    GitRepository, LocalCheckout, MonorepoComponent, ProjectRepositoryLink,
};
use engram_store::{Db, MemoryRepo, RepositoryRepo};
use std::collections::{HashMap, HashSet, VecDeque};

/// Service for graph traversal over Memory OS records.
#[derive(Clone)]
pub struct GraphService {
    repo: MemoryRepo,
    repository_repo: RepositoryRepo,
}

impl GraphService {
    /// Create a graph service.
    pub fn new(db: Db) -> Self {
        Self {
            repo: MemoryRepo::new(db.clone()),
            repository_repo: RepositoryRepo::new(db),
        }
    }

    /// Initialize graph-related schemas.
    pub async fn init_schema(&self) -> IndexResult<()> {
        self.repo.init_schema().await?;
        self.repository_repo.init_schema().await?;
        Ok(())
    }

    /// Return nodes around a starting node.
    pub async fn around(&self, start: &str, depth: usize) -> IndexResult<MemorySubgraph> {
        let graph = self.full_graph().await?;
        Ok(subgraph_around(
            graph,
            &normalize_node_id(start),
            depth.max(1),
        ))
    }

    /// Return a path between two nodes, if one exists.
    pub async fn path(
        &self,
        from: &str,
        to: &str,
        max_depth: usize,
    ) -> IndexResult<Option<MemoryGraphPath>> {
        let graph = self.full_graph().await?;
        Ok(path_between(
            graph,
            &normalize_node_id(from),
            &normalize_node_id(to),
            max_depth.max(1),
        ))
    }

    /// Return the full derived graph, optionally limited around a node.
    pub async fn subgraph(&self, start: Option<&str>, depth: usize) -> IndexResult<MemorySubgraph> {
        if let Some(start) = start {
            self.around(start, depth).await
        } else {
            self.full_graph().await
        }
    }

    /// Export a subgraph as Mermaid flowchart text.
    pub async fn export_mermaid(&self, start: Option<&str>, depth: usize) -> IndexResult<String> {
        let subgraph = self.subgraph(start, depth).await?;
        Ok(to_mermaid(&subgraph))
    }

    async fn full_graph(&self) -> IndexResult<MemorySubgraph> {
        let items = self.repo.list_memory_items(None, None).await?;
        let commits = self.repo.list_knowledge_commits(None).await?;
        let repositories = self.repository_repo.list_repositories(None).await?;
        let checkouts = self.repository_repo.list_checkouts().await?;
        let mut components = Vec::new();
        let mut project_links = Vec::new();
        for repository in &repositories {
            components.extend(self.repository_repo.list_components(&repository.id).await?);
            project_links.extend(
                self.repository_repo
                    .list_project_links(&repository.id)
                    .await?,
            );
        }
        Ok(build_graph(
            &items,
            &commits,
            &repositories,
            &checkouts,
            &components,
            &project_links,
        ))
    }
}

fn build_graph(
    items: &[MemoryItem],
    commits: &[KnowledgeCommit],
    repositories: &[GitRepository],
    checkouts: &[LocalCheckout],
    components: &[MonorepoComponent],
    project_links: &[ProjectRepositoryLink],
) -> MemorySubgraph {
    let mut nodes: HashMap<String, MemoryNode> = HashMap::new();
    let mut edges: HashSet<MemoryEdge> = HashSet::new();

    for item in items {
        let memory_id = memory_node_id(&item.id.to_string());
        insert_node(
            &mut nodes,
            MemoryNode::new(
                &memory_id,
                "memory",
                format!("{}: {}", item.kind, item.title),
            ),
        );
        add_scope(&mut nodes, &mut edges, &memory_id, &item.scope);
        for evidence in &item.evidence {
            let evidence_id = evidence_node_id(&evidence.kind, &evidence.target);
            insert_node(
                &mut nodes,
                MemoryNode::new(
                    &evidence_id,
                    "evidence",
                    evidence
                        .summary
                        .clone()
                        .unwrap_or_else(|| evidence.target.clone()),
                ),
            );
            edges.insert(MemoryEdge::new(&memory_id, evidence_id, "has_evidence"));
        }
        for superseded in &item.supersedes {
            let superseded_id = memory_node_id(&superseded.to_string());
            insert_node(
                &mut nodes,
                MemoryNode::new(&superseded_id, "memory", superseded.to_string()),
            );
            edges.insert(MemoryEdge::new(&memory_id, superseded_id, "supersedes"));
        }
        if let Some(session_id) = item.writer.session_id {
            let session_id = session_node_id(&session_id.to_string());
            insert_node(
                &mut nodes,
                MemoryNode::new(&session_id, "session", session_id.clone()),
            );
            edges.insert(MemoryEdge::new(&memory_id, session_id, "written_in"));
        }
    }

    for commit in commits {
        let commit_id = commit_node_id(&commit.id.to_string());
        insert_node(
            &mut nodes,
            MemoryNode::new(&commit_id, "commit", commit.message.clone()),
        );
        if let Some(parent_id) = commit.parent_id {
            let parent_id = commit_node_id(&parent_id.to_string());
            insert_node(
                &mut nodes,
                MemoryNode::new(&parent_id, "commit", parent_id.clone()),
            );
            edges.insert(MemoryEdge::new(&commit_id, parent_id, "parent"));
        }
        if let Some(session_id) = commit.session_id {
            let session_id = session_node_id(&session_id.to_string());
            insert_node(
                &mut nodes,
                MemoryNode::new(&session_id, "session", session_id.clone()),
            );
            edges.insert(MemoryEdge::new(&commit_id, session_id, "committed_in"));
        }
        for change in &commit.changes {
            if let Some(item_id) = change.item_id {
                let memory_id = memory_node_id(&item_id.to_string());
                insert_node(
                    &mut nodes,
                    MemoryNode::new(&memory_id, "memory", item_id.to_string()),
                );
                edges.insert(MemoryEdge::new(
                    &commit_id,
                    memory_id,
                    change.change_type.to_string(),
                ));
            }
        }
    }

    add_repository_topology(
        &mut nodes,
        &mut edges,
        repositories,
        checkouts,
        components,
        project_links,
    );

    let mut nodes = nodes.into_values().collect::<Vec<_>>();
    nodes.sort_by(|left, right| left.id.cmp(&right.id));
    let mut edges = edges.into_iter().collect::<Vec<_>>();
    edges.sort_by(|left, right| {
        left.from
            .cmp(&right.from)
            .then_with(|| left.to.cmp(&right.to))
            .then_with(|| left.relation.cmp(&right.relation))
    });
    MemorySubgraph::new(nodes, edges)
}

fn add_repository_topology(
    nodes: &mut HashMap<String, MemoryNode>,
    edges: &mut HashSet<MemoryEdge>,
    repositories: &[GitRepository],
    checkouts: &[LocalCheckout],
    components: &[MonorepoComponent],
    project_links: &[ProjectRepositoryLink],
) {
    for repository in repositories {
        let repository_id = repository_node_id(&repository.id.to_string());
        insert_node(
            nodes,
            MemoryNode::new(&repository_id, "repository", repository.name.clone()),
        );
        add_repository_alias(nodes, edges, &repository.name, &repository_id);
        if let Some(remote_url) = &repository.remote_url {
            add_repository_alias(nodes, edges, remote_url, &repository_id);
        }
    }

    for checkout in checkouts {
        let checkout_id = checkout_node_id(&checkout.id.to_string());
        insert_node(
            nodes,
            MemoryNode::new(&checkout_id, "checkout", checkout.local_path.clone()),
        );
        if let Some(repository_id) = checkout.repository_id {
            let repository_id = repository_node_id(&repository_id.to_string());
            insert_node(
                nodes,
                MemoryNode::new(&repository_id, "repository", repository_id.clone()),
            );
            edges.insert(MemoryEdge::new(&checkout_id, &repository_id, "checkout_of"));
            add_repository_alias(nodes, edges, &checkout.local_path, &repository_id);
        }
    }

    for component in components {
        let component_id = component_node_id(&component.id.to_string());
        insert_node(
            nodes,
            MemoryNode::new(&component_id, "component", component_label(component)),
        );
        let repository_id = repository_node_id(&component.repository_id.to_string());
        insert_node(
            nodes,
            MemoryNode::new(&repository_id, "repository", repository_id.clone()),
        );
        edges.insert(MemoryEdge::new(
            &component_id,
            &repository_id,
            "component_of",
        ));
    }

    for link in project_links {
        let project_id = project_node_id(&link.project_name);
        let repository_id = repository_node_id(&link.repository_id.to_string());
        insert_node(
            nodes,
            MemoryNode::new(&project_id, "project", link.project_name.clone()),
        );
        insert_node(
            nodes,
            MemoryNode::new(&repository_id, "repository", repository_id.clone()),
        );
        edges.insert(MemoryEdge::new(
            &project_id,
            &repository_id,
            format!("repository_{}", link.role),
        ));
        if let Some(component_id) = link.component_id {
            let component_id = component_node_id(&component_id.to_string());
            insert_node(
                nodes,
                MemoryNode::new(&component_id, "component", component_id.clone()),
            );
            edges.insert(MemoryEdge::new(
                &project_id,
                &component_id,
                format!("component_{}", link.role),
            ));
        } else if let Some(component_path) = &link.component_path {
            let component_id =
                component_path_node_id(&link.repository_id.to_string(), component_path);
            insert_node(
                nodes,
                MemoryNode::new(&component_id, "component", component_path.clone()),
            );
            edges.insert(MemoryEdge::new(
                &component_id,
                &repository_id,
                "component_of",
            ));
            edges.insert(MemoryEdge::new(
                &project_id,
                &component_id,
                format!("component_{}", link.role),
            ));
        }
    }
}

fn add_repository_alias(
    nodes: &mut HashMap<String, MemoryNode>,
    edges: &mut HashSet<MemoryEdge>,
    alias: &str,
    repository_id: &str,
) {
    let alias_id = repository_alias_node_id(alias);
    insert_node(
        nodes,
        MemoryNode::new(&alias_id, "repository_alias", alias.to_string()),
    );
    if alias_id != repository_id {
        edges.insert(MemoryEdge::new(alias_id, repository_id, "alias_of"));
    }
}

fn add_scope(
    nodes: &mut HashMap<String, MemoryNode>,
    edges: &mut HashSet<MemoryEdge>,
    memory_id: &str,
    scope: &MemoryScope,
) {
    let (node_id, kind, label) = match scope {
        MemoryScope::Global => ("scope:global".to_string(), "scope", "global".to_string()),
        MemoryScope::User => ("scope:user".to_string(), "scope", "user".to_string()),
        MemoryScope::Project { project_name, .. } => (
            project_node_id(project_name),
            "project",
            project_name.clone(),
        ),
        MemoryScope::Task {
            task_name,
            project_name,
            ..
        } => {
            let task_id = format!("task:{}", slug(task_name));
            if let Some(project_name) = project_name {
                let project_id = project_node_id(project_name);
                insert_node(nodes, MemoryNode::new(&project_id, "project", project_name));
                edges.insert(MemoryEdge::new(&task_id, project_id, "part_of"));
            }
            (task_id, "task", task_name.clone())
        }
        MemoryScope::Entity { entity_name, .. } => (
            format!("entity:{}", slug(entity_name)),
            "entity",
            entity_name.clone(),
        ),
        MemoryScope::Repository {
            repository_id,
            remote_url,
            local_path,
            ..
        } => {
            let label = local_path
                .clone()
                .or_else(|| remote_url.clone())
                .unwrap_or_else(|| "repository".to_string());
            let node_id = repository_id
                .map(|id| repository_node_id(&id.to_string()))
                .unwrap_or_else(|| repository_alias_node_id(&label));
            (node_id, "repository", label)
        }
        MemoryScope::Session { session_id } => (
            session_node_id(&session_id.to_string()),
            "session",
            session_id.to_string(),
        ),
        MemoryScope::Custom { name } => (format!("scope:{}", slug(name)), "scope", name.clone()),
    };
    insert_node(nodes, MemoryNode::new(&node_id, kind, label));
    edges.insert(MemoryEdge::new(memory_id, node_id, "scoped_to"));
}

fn subgraph_around(graph: MemorySubgraph, start: &str, depth: usize) -> MemorySubgraph {
    let adjacency = adjacency(&graph.edges);
    let mut seen = HashSet::new();
    let mut queue = VecDeque::from([(start.to_string(), 0usize)]);
    while let Some((node, distance)) = queue.pop_front() {
        if !seen.insert(node.clone()) || distance >= depth {
            continue;
        }
        for next in adjacency.get(&node).into_iter().flatten() {
            queue.push_back((next.clone(), distance + 1));
        }
    }

    let nodes = graph
        .nodes
        .into_iter()
        .filter(|node| seen.contains(&node.id))
        .collect();
    let edges = graph
        .edges
        .into_iter()
        .filter(|edge| seen.contains(&edge.from) && seen.contains(&edge.to))
        .collect();
    MemorySubgraph::new(nodes, edges)
}

fn path_between(
    graph: MemorySubgraph,
    from: &str,
    to: &str,
    max_depth: usize,
) -> Option<MemoryGraphPath> {
    let adjacency = adjacency_edges(&graph.edges);
    let mut queue = VecDeque::from([(from.to_string(), vec![from.to_string()], Vec::new())]);
    let mut seen = HashSet::new();
    while let Some((node, path_nodes, path_edges)) = queue.pop_front() {
        if node == to {
            return Some(MemoryGraphPath {
                nodes: path_nodes,
                edges: path_edges,
            });
        }
        if path_edges.len() >= max_depth || !seen.insert(node.clone()) {
            continue;
        }
        for edge in adjacency.get(&node).into_iter().flatten() {
            let mut next_nodes = path_nodes.clone();
            next_nodes.push(edge.to.clone());
            let mut next_edges = path_edges.clone();
            next_edges.push(edge.clone());
            queue.push_back((edge.to.clone(), next_nodes, next_edges));
        }
    }
    None
}

fn adjacency(edges: &[MemoryEdge]) -> HashMap<String, Vec<String>> {
    let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
    for edge in edges {
        adjacency
            .entry(edge.from.clone())
            .or_default()
            .push(edge.to.clone());
        adjacency
            .entry(edge.to.clone())
            .or_default()
            .push(edge.from.clone());
    }
    adjacency
}

fn adjacency_edges(edges: &[MemoryEdge]) -> HashMap<String, Vec<MemoryEdge>> {
    let mut adjacency: HashMap<String, Vec<MemoryEdge>> = HashMap::new();
    for edge in edges {
        adjacency
            .entry(edge.from.clone())
            .or_default()
            .push(edge.clone());
        adjacency
            .entry(edge.to.clone())
            .or_default()
            .push(MemoryEdge::new(&edge.to, &edge.from, &edge.relation));
    }
    adjacency
}

fn insert_node(nodes: &mut HashMap<String, MemoryNode>, node: MemoryNode) {
    nodes.entry(node.id.clone()).or_insert(node);
}

fn normalize_node_id(value: &str) -> String {
    if value.contains(':') {
        value.to_string()
    } else {
        memory_node_id(value)
    }
}

fn memory_node_id(id: &str) -> String {
    format!("memory:{id}")
}

fn repository_node_id(id: &str) -> String {
    format!("repository:{id}")
}

fn repository_alias_node_id(value: &str) -> String {
    format!("repository:{}", slug(value))
}

fn checkout_node_id(id: &str) -> String {
    format!("checkout:{id}")
}

fn component_node_id(id: &str) -> String {
    format!("component:{id}")
}

fn component_path_node_id(repository_id: &str, component_path: &str) -> String {
    format!("component:{repository_id}:{}", slug(component_path))
}

fn project_node_id(project_name: &str) -> String {
    format!("project:{}", slug(project_name))
}

fn component_label(component: &MonorepoComponent) -> String {
    if component.path == "." {
        component.name.clone()
    } else {
        format!("{} ({})", component.name, component.path)
    }
}

fn commit_node_id(id: &str) -> String {
    format!("commit:{id}")
}

fn session_node_id(id: &str) -> String {
    format!("session:{id}")
}

fn evidence_node_id(kind: &EvidenceKind, target: &str) -> String {
    format!("evidence:{}:{}", evidence_kind(kind), slug(target))
}

fn evidence_kind(kind: &EvidenceKind) -> String {
    match kind {
        EvidenceKind::SessionEvent => "session_event".to_string(),
        EvidenceKind::ToolCall => "tool_call".to_string(),
        EvidenceKind::File => "file".to_string(),
        EvidenceKind::GitCommit => "git_commit".to_string(),
        EvidenceKind::Url => "url".to_string(),
        EvidenceKind::Document => "document".to_string(),
        EvidenceKind::Observation => "observation".to_string(),
        EvidenceKind::ManualReview => "manual_review".to_string(),
        EvidenceKind::Custom(value) => value.clone(),
    }
}

fn slug(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn to_mermaid(graph: &MemorySubgraph) -> String {
    let mut lines = vec!["flowchart LR".to_string()];
    for node in &graph.nodes {
        lines.push(format!(
            "  {}[\"{}\"]",
            mermaid_id(&node.id),
            node.label.replace('"', "'")
        ));
    }
    for edge in &graph.edges {
        lines.push(format!(
            "  {} -- {} --> {}",
            mermaid_id(&edge.from),
            edge.relation,
            mermaid_id(&edge.to)
        ));
    }
    lines.join("\n")
}

fn mermaid_id(value: &str) -> String {
    value
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use engram_core::memory::{
        ClaimOrigin, EvidenceKind, EvidenceRef, Harness, MemoryKind, MemoryScope, ModelIdentity,
        WriterProvenance,
    };
    use engram_core::repository::{
        GitRepository, LocalCheckout, MonorepoComponent, ProjectRepositoryLink,
        ProjectRepositoryRole,
    };
    use engram_store::{connect_and_init, StoreConfig};

    async fn service() -> GraphService {
        let db = connect_and_init(&StoreConfig::memory()).await.unwrap();
        let service = GraphService::new(db);
        service.init_schema().await.unwrap();
        service
    }

    fn writer() -> WriterProvenance {
        WriterProvenance::agent(Harness::Codex, ModelIdentity::new("openai", "gpt-5.5"))
    }

    #[tokio::test]
    async fn graph_around_includes_scope_and_evidence() {
        let service = service().await;
        let item = MemoryItem::new(
            MemoryKind::Decision,
            "Graph decision",
            "Graph content.",
            MemoryScope::project("engram"),
            ClaimOrigin::UserStated,
            writer(),
        )
        .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "test"));
        let item_id = item.id;
        service.repo.save_memory_item(&item).await.unwrap();

        let graph = service.around(&item_id.to_string(), 1).await.unwrap();

        assert!(graph
            .nodes
            .iter()
            .any(|node| node.id == format!("memory:{item_id}")));
        assert!(graph.nodes.iter().any(|node| node.id == "project:engram"));
        assert!(graph
            .edges
            .iter()
            .any(|edge| edge.relation == "has_evidence"));
    }

    #[tokio::test]
    async fn graph_path_finds_supersedes_path() {
        let service = service().await;
        let old = MemoryItem::new(
            MemoryKind::Decision,
            "Old",
            "Old.",
            MemoryScope::project("engram"),
            ClaimOrigin::UserStated,
            writer(),
        );
        let new = MemoryItem::new(
            MemoryKind::Decision,
            "New",
            "New.",
            MemoryScope::project("engram"),
            ClaimOrigin::UserStated,
            writer(),
        )
        .with_superseded_item(old.id);
        service.repo.save_memory_item(&old).await.unwrap();
        service.repo.save_memory_item(&new).await.unwrap();

        let path = service
            .path(
                &format!("memory:{}", new.id),
                &format!("memory:{}", old.id),
                2,
            )
            .await
            .unwrap();

        assert!(path.is_some());
        assert_eq!(path.unwrap().edges[0].relation, "supersedes");
    }

    #[tokio::test]
    async fn graph_links_task_scope_to_parent_project() {
        let service = service().await;
        let item = MemoryItem::new(
            MemoryKind::TaskFact,
            "Task fact",
            "Task memory should connect to the parent project.",
            MemoryScope::Task {
                project_id: None,
                project_name: Some("engram".to_string()),
                task_id: None,
                task_name: "migration-safety".to_string(),
            },
            ClaimOrigin::UserStated,
            writer(),
        );
        service.repo.save_memory_item(&item).await.unwrap();

        let graph = service.subgraph(None, 1).await.unwrap();

        assert!(graph
            .nodes
            .iter()
            .any(|node| node.id == "task:migration-safety"));
        assert!(graph.nodes.iter().any(|node| node.id == "project:engram"));
        assert!(graph.edges.iter().any(|edge| {
            edge.from == "task:migration-safety"
                && edge.to == "project:engram"
                && edge.relation == "part_of"
        }));
    }

    #[tokio::test]
    async fn graph_includes_repository_topology() {
        let service = service().await;
        let repository =
            GitRepository::new("engram").with_remote_url("git@github.com:ymeiri/engram.git");
        let repository_id = repository.id;
        let checkout =
            LocalCheckout::new("/Users/yuval.meiri/projects/engram").with_repository(repository_id);
        let component = MonorepoComponent::new(repository_id, "engram-index", "engram-index");
        let link =
            ProjectRepositoryLink::new("engram", repository_id, ProjectRepositoryRole::Primary)
                .with_component(Some(component.id), component.path.clone());

        service
            .repository_repo
            .save_repository(&repository)
            .await
            .unwrap();
        service
            .repository_repo
            .save_checkout(&checkout)
            .await
            .unwrap();
        service
            .repository_repo
            .save_component(&component)
            .await
            .unwrap();
        service
            .repository_repo
            .save_project_link(&link)
            .await
            .unwrap();

        let graph = service
            .around(&format!("repository:{repository_id}"), 1)
            .await
            .unwrap();

        assert!(graph
            .nodes
            .iter()
            .any(|node| node.id == format!("repository:{repository_id}")));
        assert!(graph
            .nodes
            .iter()
            .any(|node| node.id == format!("checkout:{}", checkout.id)));
        assert!(graph
            .nodes
            .iter()
            .any(|node| node.id == format!("component:{}", component.id)));
        assert!(graph.nodes.iter().any(|node| node.id == "project:engram"));
        assert!(graph.edges.iter().any(|edge| {
            edge.from == "project:engram"
                && edge.to == format!("repository:{repository_id}")
                && edge.relation == "repository_primary"
        }));
        assert!(graph.edges.iter().any(|edge| {
            edge.from == format!("repository:{}", slug("git@github.com:ymeiri/engram.git"))
                && edge.to == format!("repository:{repository_id}")
                && edge.relation == "alias_of"
        }));

        let path = service
            .path("project:engram", &format!("repository:{repository_id}"), 2)
            .await
            .unwrap();
        assert!(path.is_some());
    }
}
