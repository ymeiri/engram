//! Integration tests for authorization-scoped Memory OS graph traversal.

use engram_core::memory::{
    ClaimOrigin, Harness, KnowledgeCommit, MemoryChange, MemoryChangeType, MemoryItem, MemoryKind,
    MemoryScope, ModelIdentity, WriterProvenance,
};
use engram_core::repository::{
    GitRepository, LocalCheckout, MonorepoComponent, ProjectRepositoryLink, ProjectRepositoryRole,
};
use engram_index::{GraphService, SearchService, WorkService};
use engram_mcp::tools::{self, GraphRequest, RetrievalScopeRequest, ToolState};
use engram_store::{connect_and_init, MemoryRepo, RepositoryRepo, StoreConfig};
use serde_json::Value;

async fn setup() -> (ToolState, MemoryRepo, RepositoryRepo, WorkService) {
    let db = connect_and_init(&StoreConfig::memory())
        .await
        .expect("Failed to connect");
    let graph = GraphService::new(db.clone());
    graph
        .init_schema()
        .await
        .expect("Failed to initialize graph schema");
    let work = WorkService::new(db.clone());
    work.init().await.expect("Failed to initialize work schema");

    let state = ToolState::new();
    state.init_graph(graph).await;
    state.init_search(SearchService::new(db.clone())).await;
    (
        state,
        MemoryRepo::new(db.clone()),
        RepositoryRepo::new(db),
        work,
    )
}

fn writer() -> WriterProvenance {
    WriterProvenance::agent(Harness::Codex, ModelIdentity::new("openai", "gpt-5.5"))
        .with_surface("test")
}

fn graph_request(action: &str, scope: Option<RetrievalScopeRequest>) -> GraphRequest {
    GraphRequest {
        action: action.to_string(),
        node: None,
        from: None,
        to: None,
        depth: None,
        max_depth: None,
        scope,
    }
}

fn related_scope(project: &str) -> Option<RetrievalScopeRequest> {
    Some(RetrievalScopeRequest {
        relevance_mode: Some("related".to_string()),
        project: Some(project.to_string()),
        ..RetrievalScopeRequest::default()
    })
}

fn related_task_scope(project: &str, task: &str) -> Option<RetrievalScopeRequest> {
    Some(RetrievalScopeRequest {
        relevance_mode: Some("related".to_string()),
        project: Some(project.to_string()),
        task: Some(task.to_string()),
        ..RetrievalScopeRequest::default()
    })
}

fn global_scope() -> Option<RetrievalScopeRequest> {
    Some(RetrievalScopeRequest {
        relevance_mode: Some("global".to_string()),
        ..RetrievalScopeRequest::default()
    })
}

fn parse_json(response: &str) -> Value {
    serde_json::from_str(response).expect("response should be valid JSON")
}

fn node_ids(response: &Value) -> Vec<&str> {
    response["nodes"]
        .as_array()
        .expect("graph response should contain nodes")
        .iter()
        .map(|node| node["id"].as_str().expect("node should have an ID"))
        .collect()
}

#[tokio::test]
async fn mcp_graph_reads_abstain_locally_before_service_access() {
    let state = ToolState::new();

    for action in ["around", "path", "subgraph", "export"] {
        let mut request = graph_request(action, None);
        request.node = Some("memory:local".to_string());
        request.from = Some("memory:local".to_string());
        request.to = Some("memory:other".to_string());
        let response = tools::graph_new(&state, request)
            .await
            .unwrap_or_else(|error| {
                panic!("{action} should abstain before service access: {error}")
            });
        let json = parse_json(&response);
        assert_eq!(json["executed"], false, "unexpected response for {action}");
        assert_eq!(json["relevance_mode"], "local");
        assert_eq!(json["authorization_scope_enforced"], true);
        assert_eq!(json["omitted_layers"], serde_json::json!(["graph"]));
    }

    let error = tools::graph_new(&state, graph_request("unknown", None))
        .await
        .expect_err("unknown actions should be rejected before scope handling");
    assert!(error.contains("Unknown action"));
}

#[tokio::test]
async fn mcp_graph_related_scope_filters_sources_before_traversal() {
    let (state, memory_repo, repository_repo, work) = setup().await;
    let alpha = work.create_project("alpha", None).await.unwrap();
    let beta = work.create_project("beta", None).await.unwrap();
    let alpha_task = work
        .create_task("alpha", "alpha-one", None, Some("ALPHA-1"))
        .await
        .unwrap();
    let alpha_sibling = work
        .create_task("alpha", "alpha-two", None, Some("ALPHA-2"))
        .await
        .unwrap();
    work.create_task("beta", "beta-one", None, Some("BETA-1"))
        .await
        .unwrap();

    let alpha_repository =
        GitRepository::new("alpha-repo").with_remote_url("git@github.com:test/alpha.git");
    let beta_repository =
        GitRepository::new("beta-repo").with_remote_url("git@github.com:test/beta.git");
    repository_repo
        .save_repository(&alpha_repository)
        .await
        .unwrap();
    repository_repo
        .save_repository(&beta_repository)
        .await
        .unwrap();
    let alpha_checkout = LocalCheckout::new("/work/alpha").with_repository(alpha_repository.id);
    let beta_checkout = LocalCheckout::new("/work/beta").with_repository(beta_repository.id);
    repository_repo
        .save_checkout(&alpha_checkout)
        .await
        .unwrap();
    repository_repo.save_checkout(&beta_checkout).await.unwrap();

    let alpha_component = MonorepoComponent::new(alpha_repository.id, "alpha-api", "api");
    let unrelated_alpha_component =
        MonorepoComponent::new(alpha_repository.id, "alpha-admin", "admin");
    let beta_component = MonorepoComponent::new(beta_repository.id, "beta-api", "api");
    for component in [
        &alpha_component,
        &unrelated_alpha_component,
        &beta_component,
    ] {
        repository_repo.save_component(component).await.unwrap();
    }
    repository_repo
        .save_project_link(
            &ProjectRepositoryLink::new(
                "alpha",
                alpha_repository.id,
                ProjectRepositoryRole::Primary,
            )
            .with_project_id(alpha.id)
            .with_component(Some(alpha_component.id), alpha_component.path.clone()),
        )
        .await
        .unwrap();
    repository_repo
        .save_project_link(
            &ProjectRepositoryLink::new("beta", beta_repository.id, ProjectRepositoryRole::Primary)
                .with_project_id(beta.id)
                .with_component(Some(beta_component.id), beta_component.path.clone()),
        )
        .await
        .unwrap();

    let beta_memory = MemoryItem::new(
        MemoryKind::Decision,
        "Beta decision",
        "Only beta may read this.",
        MemoryScope::Project {
            project_id: Some(beta.id),
            project_name: beta.name.clone(),
        },
        ClaimOrigin::UserStated,
        writer(),
    );
    let alpha_memory = MemoryItem::new(
        MemoryKind::Decision,
        "Alpha decision",
        "Only alpha may read this.",
        MemoryScope::Project {
            project_id: Some(alpha.id),
            project_name: alpha.name.clone(),
        },
        ClaimOrigin::UserStated,
        writer(),
    )
    .with_superseded_item(beta_memory.id);
    let global_memory = MemoryItem::new(
        MemoryKind::Preference,
        "Global preference",
        "Applies to every project.",
        MemoryScope::Global,
        ClaimOrigin::UserStated,
        writer(),
    );
    let alpha_task_memory = MemoryItem::new(
        MemoryKind::TaskFact,
        "Alpha task",
        "Exact alpha task context.",
        MemoryScope::Task {
            project_id: Some(alpha.id),
            project_name: Some(alpha.name.clone()),
            task_id: Some(alpha_task.id),
            task_name: alpha_task.name.clone(),
        },
        ClaimOrigin::UserStated,
        writer(),
    );
    let alpha_sibling_memory = MemoryItem::new(
        MemoryKind::TaskFact,
        "Alpha sibling task",
        "Sibling task context.",
        MemoryScope::Task {
            project_id: Some(alpha.id),
            project_name: Some(alpha.name.clone()),
            task_id: Some(alpha_sibling.id),
            task_name: alpha_sibling.name.clone(),
        },
        ClaimOrigin::UserStated,
        writer(),
    );
    let unowned_memory = MemoryItem::new(
        MemoryKind::Decision,
        "Unowned entity decision",
        "No project ownership can be proven.",
        MemoryScope::entity("unowned"),
        ClaimOrigin::UserStated,
        writer(),
    );
    let alpha_repository_memory = MemoryItem::new(
        MemoryKind::RepositoryFact,
        "Alpha repository fact",
        "Owned through the alpha repository link.",
        MemoryScope::Repository {
            repository_id: Some(alpha_repository.id),
            remote_url: None,
            local_path: None,
        },
        ClaimOrigin::UserStated,
        writer(),
    );
    let beta_repository_memory = MemoryItem::new(
        MemoryKind::RepositoryFact,
        "Beta repository fact",
        "Owned through the beta repository link.",
        MemoryScope::Repository {
            repository_id: Some(beta_repository.id),
            remote_url: None,
            local_path: None,
        },
        ClaimOrigin::UserStated,
        writer(),
    );

    for item in [
        &beta_memory,
        &alpha_memory,
        &global_memory,
        &alpha_task_memory,
        &alpha_sibling_memory,
        &unowned_memory,
        &alpha_repository_memory,
        &beta_repository_memory,
    ] {
        memory_repo.save_memory_item(item).await.unwrap();
    }
    let commit = KnowledgeCommit::new(writer(), "Mixed-project commit message").with_change(
        MemoryChange::new(MemoryChangeType::Added, "Alpha", "Added alpha memory")
            .with_item(alpha_memory.id),
    );
    memory_repo.save_knowledge_commit(&commit).await.unwrap();

    let related = parse_json(
        &tools::graph_new(&state, graph_request("subgraph", related_scope("alpha")))
            .await
            .unwrap(),
    );
    let ids = node_ids(&related);
    assert_eq!(related["relevance_mode"], "related");
    assert_eq!(related["resolved_project"], "alpha");
    assert_eq!(related["authorization_scope_enforced"], true);
    assert!(ids.contains(&format!("memory:{}", global_memory.id).as_str()));
    assert!(ids.contains(&format!("memory:{}", alpha_memory.id).as_str()));
    assert!(ids.contains(&format!("memory:{}", alpha_task_memory.id).as_str()));
    assert!(ids.contains(&format!("memory:{}", alpha_sibling_memory.id).as_str()));
    assert!(ids.contains(&format!("memory:{}", alpha_repository_memory.id).as_str()));
    assert!(!ids.contains(&format!("memory:{}", beta_memory.id).as_str()));
    assert!(!ids.contains(&format!("memory:{}", unowned_memory.id).as_str()));
    assert!(!ids.contains(&format!("memory:{}", beta_repository_memory.id).as_str()));
    assert!(ids.contains(&format!("repository:{}", alpha_repository.id).as_str()));
    assert!(!ids.contains(&format!("repository:{}", beta_repository.id).as_str()));
    assert!(ids.contains(&format!("component:{}", alpha_component.id).as_str()));
    assert!(!ids.contains(&format!("component:{}", unrelated_alpha_component.id).as_str()));
    assert!(!ids.iter().any(|id| id.starts_with("commit:")));
    assert!(!related["edges"].as_array().unwrap().iter().any(|edge| {
        edge["from"] == format!("memory:{}", alpha_memory.id)
            && edge["to"] == format!("memory:{}", beta_memory.id)
    }));

    let exact_task = parse_json(
        &tools::graph_new(
            &state,
            graph_request("subgraph", related_task_scope("alpha", "ALPHA-1")),
        )
        .await
        .unwrap(),
    );
    let exact_ids = node_ids(&exact_task);
    assert_eq!(exact_task["resolved_task"], "alpha-one");
    assert!(exact_ids.contains(&format!("memory:{}", alpha_memory.id).as_str()));
    assert!(exact_ids.contains(&format!("memory:{}", alpha_task_memory.id).as_str()));
    assert!(!exact_ids.contains(&format!("memory:{}", alpha_sibling_memory.id).as_str()));

    let mut around_beta = graph_request("around", related_scope("alpha"));
    around_beta.node = Some(beta_memory.id.to_string());
    let around_beta = parse_json(&tools::graph_new(&state, around_beta).await.unwrap());
    assert!(node_ids(&around_beta).is_empty());

    let mut path_to_beta = graph_request("path", related_scope("alpha"));
    path_to_beta.from = Some(alpha_memory.id.to_string());
    path_to_beta.to = Some(beta_memory.id.to_string());
    let path_to_beta = parse_json(&tools::graph_new(&state, path_to_beta).await.unwrap());
    assert!(path_to_beta["path"].is_null());
    assert_eq!(path_to_beta["authorization_scope_enforced"], true);

    let mismatch = tools::graph_new(
        &state,
        graph_request("subgraph", related_task_scope("alpha", "BETA-1")),
    )
    .await
    .expect_err("a task from another project should be rejected");
    assert!(mismatch.contains("belongs to project"));

    let global = parse_json(
        &tools::graph_new(&state, graph_request("subgraph", global_scope()))
            .await
            .unwrap(),
    );
    let global_ids = node_ids(&global);
    assert_eq!(global["relevance_mode"], "global");
    assert!(global_ids.contains(&format!("memory:{}", beta_memory.id).as_str()));
    assert!(global_ids.contains(&format!("memory:{}", unowned_memory.id).as_str()));
    assert!(global_ids.contains(&format!("repository:{}", beta_repository.id).as_str()));
    assert!(global_ids.contains(&format!("commit:{}", commit.id).as_str()));
}
