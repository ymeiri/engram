//! Seed a materialized evaluation fixture into an isolated Engram data store.

use crate::fixture::FixtureLayout;
use crate::{EvalError, EvalResult};
use engram_core::memory::{
    ClaimOrigin, EvidenceKind, EvidenceRef, Harness, MemoryItem, MemoryKind, MemoryScope,
    MemoryStatus, ModelIdentity, ProcedureCard, ProcedurePrerequisite, ProcedureVerification,
    WriterProvenance,
};
use engram_core::repository::ProjectRepositoryRole;
use engram_core::Id;
use engram_index::{
    MemoryService, ProcedureMatchInput, ProcedureVerificationReceipt, RepositoryService,
    WorkService,
};
use engram_store::{connect_and_init, Db, StoreConfig};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use time::{Duration, OffsetDateTime};

/// IDs created while applying the semantic seed plan.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeedReport {
    /// Fixture content revision.
    pub fixture_revision: String,
    /// Isolated RocksDB directory.
    pub data_dir: String,
    /// Project name to generated Engram ID.
    pub projects: BTreeMap<String, String>,
    /// Semantic repository key to generated Engram ID.
    pub repositories: BTreeMap<String, String>,
    /// Stable context key to generated memory ID.
    pub memory: BTreeMap<String, String>,
    /// Procedure keys activated by receipt verification.
    pub verified_procedures: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SeedPlan {
    schema_version: u32,
    projects: Vec<SeedProject>,
    repositories: Vec<SeedRepository>,
    #[serde(default)]
    registered_checkout_lookalikes: Vec<SeedCheckout>,
    memory: Vec<SeedMemory>,
}

#[derive(Debug, Deserialize)]
struct SeedProject {
    name: String,
}

#[derive(Debug, Deserialize)]
struct SeedRepository {
    key: String,
    name: String,
    remote: String,
    checkout: String,
    project: String,
    #[serde(default)]
    components: Vec<SeedComponent>,
}

#[derive(Debug, Deserialize)]
struct SeedComponent {
    name: String,
    path: String,
    kind: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SeedCheckout {
    repository: String,
    checkout: String,
}

#[derive(Debug, Deserialize)]
struct SeedMemory {
    key: String,
    kind: String,
    status: String,
    scope: SeedScope,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    evidence: Option<String>,
    #[serde(default)]
    supersedes: Option<String>,
    #[serde(default)]
    task: Option<String>,
    #[serde(default)]
    commands: Vec<String>,
    #[serde(default)]
    prerequisites: BTreeMap<String, String>,
    #[serde(default)]
    failure_signatures: Vec<String>,
    #[serde(default)]
    verification_receipt: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum SeedScope {
    Project { project_name: String },
    Repository { remote_url: String },
}

/// Apply a fixture's semantic seed plan to a new isolated Engram store.
///
/// Existing store content is never replaced or merged.
pub async fn seed_engineering_context_fixture(
    fixture_map: &Path,
    data_dir: &Path,
) -> EvalResult<SeedReport> {
    let (report, _) = open_seeded_engineering_context_fixture(fixture_map, data_dir).await?;
    Ok(report)
}

/// Apply a fixture seed and retain the exact isolated database handle for immediate evaluation.
pub async fn open_seeded_engineering_context_fixture(
    fixture_map: &Path,
    data_dir: &Path,
) -> EvalResult<(SeedReport, Db)> {
    require_empty_target(data_dir)?;
    let db = connect_and_init(&StoreConfig::rocksdb(data_dir.to_path_buf()))
        .await
        .map_err(invalid)?;
    let report = seed_engineering_context_fixture_in_db(fixture_map, data_dir, db.clone()).await?;
    Ok((report, db))
}

async fn seed_engineering_context_fixture_in_db(
    fixture_map: &Path,
    data_dir: &Path,
    db: Db,
) -> EvalResult<SeedReport> {
    let layout: FixtureLayout = serde_json::from_reader(fs::File::open(fixture_map)?)?;
    let plan: SeedPlan = serde_json::from_reader(fs::File::open(&layout.seed_plan)?)?;
    if plan.schema_version != 1 || layout.schema_version != 1 {
        return Err(EvalError::Invalid(
            "unsupported fixture or seed-plan schema version".to_string(),
        ));
    }

    let work = WorkService::new(db.clone());
    work.init().await.map_err(invalid)?;
    let repositories = RepositoryService::new(db.clone());
    repositories.init_schema().await.map_err(invalid)?;
    let memory = MemoryService::new(db);
    memory.init_schema().await.map_err(invalid)?;

    let mut project_ids = BTreeMap::new();
    for project in &plan.projects {
        let created = work
            .create_project(
                &project.name,
                Some("Engram engineering-context evaluation fixture"),
            )
            .await
            .map_err(invalid)?;
        project_ids.insert(project.name.clone(), created.id.to_string());
    }

    let mut repository_ids = BTreeMap::new();
    let mut repository_ids_by_remote = BTreeMap::new();
    for repository in &plan.repositories {
        let checkout = resolve_fixture_uri(&layout, &repository.checkout)?;
        let detection = repositories
            .detect_repository(&checkout)
            .await
            .map_err(invalid)?;
        if detection.context.repository.name != repository.name {
            return Err(EvalError::Invalid(format!(
                "repository {} resolved as {}",
                repository.key, detection.context.repository.name
            )));
        }
        let actual_remote = detection
            .context
            .repository
            .remote_url
            .as_deref()
            .unwrap_or_default();
        if normalize_remote(actual_remote) != normalize_remote(&repository.remote) {
            return Err(EvalError::Invalid(format!(
                "repository {} remote mismatch: {actual_remote}",
                repository.key
            )));
        }
        let repository_id = detection.context.repository.id;
        for component in &repository.components {
            repositories
                .register_component(
                    Some(&repository_id),
                    None,
                    &component.name,
                    &component.path,
                    component.kind.as_deref(),
                    None,
                )
                .await
                .map_err(invalid)?;
        }
        repositories
            .link_project(
                &repository.project,
                Some(&repository_id),
                None,
                ProjectRepositoryRole::Primary,
                None,
            )
            .await
            .map_err(invalid)?;
        for component in &repository.components {
            repositories
                .link_project(
                    &repository.project,
                    Some(&repository_id),
                    None,
                    ProjectRepositoryRole::Primary,
                    Some(&component.path),
                )
                .await
                .map_err(invalid)?;
        }
        let repository_id = repository_id.to_string();
        repository_ids.insert(repository.key.clone(), repository_id.clone());
        if repository_ids_by_remote
            .insert(normalize_remote(&repository.remote), repository_id)
            .is_some()
        {
            return Err(EvalError::Invalid(format!(
                "duplicate normalized repository remote: {}",
                repository.remote
            )));
        }
    }

    // Register Atlas legacy and the two `api` lookalikes. Deliberately leave the moved checkout
    // unseen so that scenario proves remote-based identity on first encounter.
    repositories
        .detect_repository(&resolve_fixture_uri(&layout, "fixture://atlas/legacy")?)
        .await
        .map_err(invalid)?;
    for checkout in &plan.registered_checkout_lookalikes {
        let detection = repositories
            .detect_repository(&resolve_fixture_uri(&layout, &checkout.checkout)?)
            .await
            .map_err(invalid)?;
        let expected = repository_ids.get(&checkout.repository).ok_or_else(|| {
            EvalError::Invalid(format!("unknown repository key: {}", checkout.repository))
        })?;
        if detection.context.repository.id.to_string() != *expected {
            return Err(EvalError::Invalid(format!(
                "lookalike {} did not attach to {}",
                checkout.checkout, checkout.repository
            )));
        }
    }

    let writer = WriterProvenance::agent(
        Harness::Other("engram_eval".to_string()),
        ModelIdentity::new("deterministic", "fixture-seeder"),
    )
    .with_surface("engram-eval");
    let mut memory_ids: BTreeMap<String, Id> = BTreeMap::new();
    let mut verified_procedures = Vec::new();
    for seed in &plan.memory {
        let scope = seed_scope(seed, &project_ids, &repository_ids_by_remote)?;
        let kind = MemoryKind::parse(&seed.kind);
        let content = seed
            .content
            .clone()
            .unwrap_or_else(|| format!("Evaluation fixture record {}", seed.key));
        let mut item = MemoryItem::new(
            kind.clone(),
            seed.key.replace('-', " "),
            content,
            scope,
            ClaimOrigin::AgentObserved,
            writer.clone(),
        )
        .with_tag(seed.key.clone());
        if let Some(evidence) = &seed.evidence {
            item = item.with_evidence(EvidenceRef::new(
                EvidenceKind::File,
                resolve_fixture_uri(&layout, evidence)?
                    .display()
                    .to_string(),
            ));
        }
        if let Some(supersedes) = &seed.supersedes {
            let superseded_id = memory_ids.get(supersedes).ok_or_else(|| {
                EvalError::Invalid(format!(
                    "memory {} supersedes unknown earlier key {supersedes}",
                    seed.key
                ))
            })?;
            item = item.with_superseded_item(*superseded_id);
        }

        let receipt_path = seed
            .verification_receipt
            .as_deref()
            .map(|uri| resolve_fixture_uri(&layout, uri))
            .transpose()?;
        if kind == MemoryKind::Procedure {
            let receipt = receipt_path.as_deref().map(read_receipt).transpose()?;
            let command = receipt
                .as_ref()
                .map(|receipt| receipt.command.clone())
                .or_else(|| seed.commands.first().cloned())
                .unwrap_or_else(|| "unverified".to_string());
            let exit_code = receipt.as_ref().map_or(0, |receipt| receipt.exit_code);
            let marker = receipt
                .as_ref()
                .map(|receipt| receipt.output.trim().to_string())
                .unwrap_or_else(|| "UNVERIFIED".to_string());
            let mut procedure = ProcedureCard::new(
                seed.task.clone().unwrap_or_else(|| seed.key.clone()),
                seed.commands.clone(),
                ProcedureVerification::new(command, exit_code, marker),
            );
            for (key, expected) in &seed.prerequisites {
                procedure = procedure.with_prerequisite(ProcedurePrerequisite::new(key, expected));
            }
            for signature in &seed.failure_signatures {
                procedure = procedure.with_failure_signature(signature);
            }
            item = item.with_procedure(procedure);
        }

        let desired_status = parse_status(&seed.status)?;
        item = if receipt_path.is_some() {
            item.with_status(MemoryStatus::NeedsReview)
        } else {
            item.with_status(desired_status)
        };
        let captured = memory.capture_memory(item).await.map_err(invalid)?;
        let id = captured.id;
        if let Some(receipt_path) = receipt_path {
            if desired_status != MemoryStatus::Active {
                return Err(EvalError::Invalid(format!(
                    "verified procedure {} must request active status",
                    seed.key
                )));
            }
            memory
                .verify_procedure(
                    &id,
                    &receipt_path,
                    Some(OffsetDateTime::now_utc() + Duration::days(3650)),
                )
                .await
                .map_err(invalid)?;
            verified_procedures.push(seed.key.clone());
        }
        memory_ids.insert(seed.key.clone(), id);
    }

    validate_seeded_moved_checkout(&layout, &memory, &memory_ids).await?;

    Ok(SeedReport {
        fixture_revision: layout.fixture_revision,
        data_dir: data_dir.canonicalize()?.display().to_string(),
        projects: project_ids,
        repositories: repository_ids,
        memory: memory_ids
            .into_iter()
            .map(|(key, id)| (key, id.to_string()))
            .collect(),
        verified_procedures,
    })
}

async fn validate_seeded_moved_checkout(
    layout: &FixtureLayout,
    memory: &MemoryService,
    memory_ids: &BTreeMap<String, Id>,
) -> EvalResult<()> {
    let moved =
        resolve_fixture_uri(layout, "fixture://moved/arbitrary-name")?.join("services/worker");
    let matched = memory
        .match_procedures(ProcedureMatchInput {
            query: "run queue worker integration test".to_string(),
            project: Some("atlas".to_string()),
            cwd: Some(moved.display().to_string()),
            conditions: BTreeMap::from([("tool.version".to_string(), "3".to_string())]),
            limit: Some(5),
        })
        .await
        .map_err(invalid)?;
    let expected = memory_ids
        .get("procedure-atlas-integration-v3")
        .ok_or_else(|| EvalError::Invalid("missing moved-checkout procedure seed".to_string()))?;
    if matched.abstained || matched.procedures.len() != 1 || matched.procedures[0].id != *expected {
        return Err(EvalError::Invalid(format!(
            "seeded moved checkout did not resolve exactly the verified Atlas v3 procedure: {}",
            serde_json::to_string(&matched)?
        )));
    }
    let evidence_path = matched.procedures[0]
        .procedure
        .as_ref()
        .and_then(|procedure| procedure.verification.evidence_path.as_deref());
    if evidence_path != Some("evidence/integration-v3-success.json") {
        return Err(EvalError::Invalid(format!(
            "seeded repository procedure proof is not checkout-relative: {evidence_path:?}"
        )));
    }
    Ok(())
}

fn seed_scope(
    seed: &SeedMemory,
    projects: &BTreeMap<String, String>,
    repositories_by_remote: &BTreeMap<String, String>,
) -> EvalResult<MemoryScope> {
    match &seed.scope {
        SeedScope::Project { project_name } => {
            let project_id = projects
                .get(project_name)
                .ok_or_else(|| EvalError::Invalid(format!("unknown project: {project_name}")))?;
            Ok(MemoryScope::Project {
                project_id: Some(
                    Id::parse(project_id).map_err(|error| EvalError::Invalid(error.to_string()))?,
                ),
                project_name: project_name.clone(),
            })
        }
        SeedScope::Repository { remote_url } => {
            let repository_id = repositories_by_remote
                .get(&normalize_remote(remote_url))
                .ok_or_else(|| {
                    EvalError::Invalid(format!("unknown repository remote: {remote_url}"))
                })?;
            Ok(MemoryScope::Repository {
                repository_id: Some(
                    Id::parse(repository_id)
                        .map_err(|error| EvalError::Invalid(error.to_string()))?,
                ),
                remote_url: Some(remote_url.clone()),
                local_path: None,
            })
        }
    }
}

fn resolve_fixture_uri(layout: &FixtureLayout, uri: &str) -> EvalResult<PathBuf> {
    let (prefix, root) = layout
        .checkouts
        .iter()
        .filter(|(prefix, _)| uri == prefix.as_str() || uri.starts_with(&format!("{prefix}/")))
        .max_by_key(|(prefix, _)| prefix.len())
        .ok_or_else(|| EvalError::Invalid(format!("unmapped fixture URI: {uri}")))?;
    let suffix = uri
        .strip_prefix(prefix)
        .unwrap_or_default()
        .trim_start_matches('/');
    Ok(Path::new(root).join(suffix))
}

fn read_receipt(path: &Path) -> EvalResult<ProcedureVerificationReceipt> {
    Ok(serde_json::from_reader(fs::File::open(path)?)?)
}

fn parse_status(value: &str) -> EvalResult<MemoryStatus> {
    match value {
        "active" => Ok(MemoryStatus::Active),
        "needs_review" => Ok(MemoryStatus::NeedsReview),
        "superseded" => Ok(MemoryStatus::Superseded),
        "archived" => Ok(MemoryStatus::Archived),
        "rejected" => Ok(MemoryStatus::Rejected),
        _ => Err(EvalError::Invalid(format!(
            "unsupported seed memory status: {value}"
        ))),
    }
}

fn normalize_remote(value: &str) -> String {
    let trimmed = value.trim().trim_end_matches('/').trim_end_matches(".git");
    let lower = trimmed.to_ascii_lowercase();
    if let Some(rest) = lower.strip_prefix("git@") {
        return rest.replacen(':', "/", 1);
    }
    lower
        .strip_prefix("https://")
        .or_else(|| lower.strip_prefix("http://"))
        .or_else(|| lower.strip_prefix("ssh://git@"))
        .unwrap_or(&lower)
        .to_string()
}

fn require_empty_target(path: &Path) -> EvalResult<()> {
    if path.exists() && fs::read_dir(path)?.next().transpose()?.is_some() {
        return Err(EvalError::Invalid(format!(
            "seed data target is not empty: {}",
            path.display()
        )));
    }
    fs::create_dir_all(path)?;
    Ok(())
}

fn invalid(error: impl std::fmt::Display) -> EvalError {
    EvalError::Invalid(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture::materialize_engineering_context_fixture;

    #[tokio::test]
    async fn seeds_isolated_store_with_verified_scoped_context() {
        let temp = tempfile::tempdir().unwrap();
        let fixture_root = temp.path().join("fixture");
        let layout = materialize_engineering_context_fixture(&fixture_root).unwrap();
        let data_dir = temp.path().join("data");
        let report =
            seed_engineering_context_fixture(&fixture_root.join("fixture-map.json"), &data_dir)
                .await
                .unwrap();

        assert_eq!(report.fixture_revision, layout.fixture_revision);
        assert_eq!(report.projects.len(), 2);
        assert_eq!(report.repositories.len(), 2);
        assert_eq!(report.memory.len(), 13);
        assert_eq!(report.verified_procedures.len(), 3);

        assert!(report.memory.contains_key("procedure-atlas-integration-v3"));

        let second =
            seed_engineering_context_fixture(&fixture_root.join("fixture-map.json"), &fixture_root)
                .await
                .unwrap_err();
        assert!(second.to_string().contains("not empty"));
    }
}
