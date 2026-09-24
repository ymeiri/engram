//! Deterministic filesystem fixture for the engineering-context evaluation suite.

use crate::{EvalError, EvalResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;

const FIXED_GIT_DATE: &str = "2026-01-02T03:04:05Z";
const BASE_AGENT_INSTRUCTIONS: &str = "# Evaluation repository guidance\n\nUse checked-in code, Git metadata, ADRs, and runbooks as authoritative. Do not guess missing identity or context; report material ambiguity. Do not use network or external sources. For requested commands, inspect local scripts and execute only an applicable safe local command. Cite checkout-relative evidence paths. Do not read files outside this checkout.\n";

/// Materialized checkout paths and their portable fixture revision.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FixtureLayout {
    /// Fixture schema version.
    pub schema_version: u32,
    /// SHA-256 over logical paths, file bodies, and the semantic seed plan.
    pub fixture_revision: String,
    /// Materialization root.
    pub root: String,
    /// Mapping from `fixture://` checkout URI to absolute path.
    pub checkouts: BTreeMap<String, String>,
    /// Machine-readable semantic seed plan.
    pub seed_plan: String,
}

struct FixtureFile {
    path: &'static str,
    body: &'static str,
    executable: bool,
}

/// Materialize a fresh deterministic fixture tree.
///
/// The target may be absent or an existing empty directory. Existing content is never replaced.
pub fn materialize_engineering_context_fixture(root: &Path) -> EvalResult<FixtureLayout> {
    if root.exists() && fs::read_dir(root)?.next().transpose()?.is_some() {
        return Err(EvalError::Invalid(format!(
            "fixture target is not empty: {}",
            root.display()
        )));
    }
    fs::create_dir_all(root)?;

    let atlas_main = root.join("atlas/main");
    let atlas_legacy = root.join("atlas/legacy");
    let orbit_main = root.join("orbit/main");
    let ambiguous = root.join("ambiguous/api");
    let moved = root.join("moved/arbitrary-name");
    let registered_atlas = root.join("registered/atlas/api");
    let registered_orbit = root.join("registered/orbit/api");

    let atlas_main_files = atlas_main_files();
    write_files(&atlas_main, &atlas_main_files)?;
    init_git_repository(
        &atlas_main,
        "git@github.com:acme/atlas.git",
        "atlas main fixture",
    )?;

    let atlas_legacy_files = atlas_legacy_files();
    write_files(&atlas_legacy, &atlas_legacy_files)?;
    init_git_repository(
        &atlas_legacy,
        "git@github.com:acme/atlas.git",
        "atlas legacy fixture",
    )?;

    let orbit_files = orbit_files();
    write_files(&orbit_main, &orbit_files)?;
    init_git_repository(
        &orbit_main,
        "git@github.com:acme/orbit.git",
        "orbit fixture",
    )?;

    clone_repository(&atlas_main, &moved)?;
    git(
        &moved,
        &[
            "remote",
            "set-url",
            "origin",
            "https://github.com/acme/atlas",
        ],
    )?;
    clone_repository(&atlas_main, &registered_atlas)?;
    git(
        &registered_atlas,
        &[
            "remote",
            "set-url",
            "origin",
            "git@github.com:acme/atlas.git",
        ],
    )?;
    clone_repository(&orbit_main, &registered_orbit)?;
    git(
        &registered_orbit,
        &[
            "remote",
            "set-url",
            "origin",
            "git@github.com:acme/orbit.git",
        ],
    )?;

    fs::create_dir_all(&ambiguous)?;
    fs::write(
        ambiguous.join("README.md"),
        "# API checkout\n\nThis intentionally has no Git metadata or stable remote.\n",
    )?;

    let seed_plan_value = seed_plan();
    let seed_plan_body = serde_json::to_string_pretty(&seed_plan_value)? + "\n";
    let seed_plan_path = root.join("seed-plan.json");
    fs::write(&seed_plan_path, &seed_plan_body)?;

    let fixture_revision = fixture_revision(
        &[
            ("atlas/main", &atlas_main_files),
            ("atlas/legacy", &atlas_legacy_files),
            ("orbit/main", &orbit_files),
        ],
        &seed_plan_body,
    );
    let checkouts = BTreeMap::from([
        ("fixture://atlas/main".to_string(), absolute(&atlas_main)?),
        (
            "fixture://atlas/legacy".to_string(),
            absolute(&atlas_legacy)?,
        ),
        ("fixture://orbit/main".to_string(), absolute(&orbit_main)?),
        ("fixture://ambiguous/api".to_string(), absolute(&ambiguous)?),
        (
            "fixture://moved/arbitrary-name".to_string(),
            absolute(&moved)?,
        ),
        (
            "fixture://registered/atlas/api".to_string(),
            absolute(&registered_atlas)?,
        ),
        (
            "fixture://registered/orbit/api".to_string(),
            absolute(&registered_orbit)?,
        ),
    ]);
    let layout = FixtureLayout {
        schema_version: 1,
        fixture_revision,
        root: absolute(root)?,
        checkouts,
        seed_plan: absolute(&seed_plan_path)?,
    };
    fs::write(
        root.join("fixture-map.json"),
        serde_json::to_string_pretty(&layout)? + "\n",
    )?;
    Ok(layout)
}

fn atlas_main_files() -> Vec<FixtureFile> {
    let mut files = agent_instruction_files();
    files.extend([
        FixtureFile {
            path: "services/worker/README.md",
            body: "# Queue worker\n\nComponent: queue-worker\n",
            executable: false,
        },
        FixtureFile {
            path: "services/worker/component.json",
            body: "{\"name\":\"queue-worker\",\"kind\":\"service\"}\n",
            executable: false,
        },
        FixtureFile {
            path: "docs/adr/0042-queue.md",
            body: "# ADR 0042: Queue backend\n\nStatus: accepted\n\nUse NATS JetStream for the queue worker. It provides durable consumers and bounded redelivery without operating Kafka for this workload. This supersedes the Redis Streams proposal.\n",
            executable: false,
        },
        FixtureFile {
            path: "docs/adr/0048-http-client.md",
            body: "# ADR 0048: HTTP client\n\nStatus: accepted\n\nNew Rust code uses reqwest with the shared timeout middleware. This supersedes the isahc wrapper.\n",
            executable: false,
        },
        FixtureFile {
            path: "runbooks/deploy-worker.md",
            body: "# Deploy the Atlas queue worker\n\nRun `./bin/deploy-worker atlas queue-worker` and verify `ATLAS_WORKER_READY`.\n",
            executable: false,
        },
        FixtureFile {
            path: "evidence/integration-v3-success.json",
            body: "{\"command\":\"./bin/integration-test --worker queue\",\"exit_code\":0,\"output\":\"ATLAS_INTEGRATION_V3_OK\",\"conditions\":{\"tool.version\":\"3\"}}\n",
            executable: false,
        },
        FixtureFile {
            path: "evidence/deploy-worker-success.json",
            body: "{\"command\":\"./bin/deploy-worker atlas queue-worker\",\"exit_code\":0,\"output\":\"ATLAS_WORKER_READY\",\"conditions\":{\"deployer.version\":\"2\"}}\n",
            executable: false,
        },
        FixtureFile {
            path: "toolchain.toml",
            body: "[tools]\nversion = \"3\"\n",
            executable: false,
        },
        FixtureFile {
            path: "bin/integration-test",
            body: "#!/bin/sh\nset -eu\n[ \"${1:-}\" = \"--worker\" ]\n[ \"${2:-}\" = \"queue\" ]\nprintf '%s\\n' ATLAS_INTEGRATION_V3_OK\n",
            executable: true,
        },
        FixtureFile {
            path: "bin/deploy-worker",
            body: "#!/bin/sh\nset -eu\n[ \"${1:-}\" = \"atlas\" ]\n[ \"${2:-}\" = \"queue-worker\" ]\nprintf '%s\\n' ATLAS_WORKER_READY\n",
            executable: true,
        },
        FixtureFile {
            path: "bin/context-probe",
            body: "#!/bin/sh\nset -eu\nactual=$(printf '%s' \"${1:-} ${2:-}\" | cksum)\nif [ \"$actual\" != \"1070561821 16\" ]; then\n  printf '%s\\n' ATLAS_CONTEXT_PROBE_REJECTED >&2\n  exit 2\nfi\nprintf '%s\\n' ATLAS_CONTEXT_PROBE_OK\n",
            executable: true,
        },
    ]);
    files
}

fn atlas_legacy_files() -> Vec<FixtureFile> {
    let mut files = agent_instruction_files();
    files.extend([
        FixtureFile {
            path: "services/worker/README.md",
            body: "# Queue worker legacy checkout\n\nComponent: queue-worker\n",
            executable: false,
        },
        FixtureFile {
            path: "services/worker/component.json",
            body: "{\"name\":\"queue-worker\",\"kind\":\"service\"}\n",
            executable: false,
        },
        FixtureFile {
            path: "toolchain.toml",
            body: "[tools]\nversion = \"2\"\n",
            executable: false,
        },
        FixtureFile {
            path: "bin/integration-test",
            body: "#!/bin/sh\nset -eu\nprintf '%s\\n' ATLAS_INTEGRATION_V2_ONLY\n",
            executable: true,
        },
    ]);
    files
}

fn orbit_files() -> Vec<FixtureFile> {
    let mut files = agent_instruction_files();
    files.extend([
        FixtureFile {
            path: "services/worker/README.md",
            body: "# Orbit worker\n\nComponent: worker\n",
            executable: false,
        },
        FixtureFile {
            path: "services/worker/component.json",
            body: "{\"name\":\"queue-worker\",\"kind\":\"service\"}\n",
            executable: false,
        },
        FixtureFile {
            path: "runbooks/deploy-worker.md",
            body: "# Deploy the Orbit worker\n\nORBIT_ONLY_CANARY. Run `orbitctl worker deploy --fast`. This procedure never applies to Atlas.\n",
            executable: false,
        },
        FixtureFile {
            path: "evidence/deploy-worker-success.json",
            body: "{\"command\":\"orbitctl worker deploy --fast\",\"exit_code\":0,\"output\":\"ORBIT_DEPLOY_OK\",\"conditions\":{\"orbitctl.version\":\"9\"}}\n",
            executable: false,
        },
    ]);
    files
}

fn agent_instruction_files() -> Vec<FixtureFile> {
    vec![
        FixtureFile {
            path: "AGENTS.md",
            body: BASE_AGENT_INSTRUCTIONS,
            executable: false,
        },
        FixtureFile {
            path: "CLAUDE.md",
            body: BASE_AGENT_INSTRUCTIONS,
            executable: false,
        },
    ]
}

fn seed_plan() -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "projects": [
            {"name": "atlas", "status": "active"},
            {"name": "orbit", "status": "active"}
        ],
        "repositories": [
            {
                "key": "repo-atlas",
                "name": "atlas",
                "remote": "git@github.com:acme/atlas.git",
                "checkout": "fixture://atlas/main",
                "project": "atlas",
                "components": [{"name": "queue-worker", "path": "services/worker", "kind": "service"}]
            },
            {
                "key": "repo-orbit",
                "name": "orbit",
                "remote": "git@github.com:acme/orbit.git",
                "checkout": "fixture://orbit/main",
                "project": "orbit",
                "components": [{"name": "worker", "path": "services/worker", "kind": "service"}]
            }
        ],
        "registered_checkout_lookalikes": [
            {"repository": "repo-atlas", "checkout": "fixture://registered/atlas/api"},
            {"repository": "repo-orbit", "checkout": "fixture://registered/orbit/api"}
        ],
        "memory": [
            {"key": "decision-atlas-queue-v1", "kind": "decision", "status": "superseded", "scope": {"type": "project", "project_name": "atlas"}, "content": "Use Redis Streams.", "evidence": "fixture://atlas/main/docs/adr/0042-queue.md"},
            {"key": "decision-atlas-queue-v2", "kind": "decision", "status": "active", "scope": {"type": "project", "project_name": "atlas"}, "content": "Use NATS JetStream for durable consumers and bounded redelivery.", "evidence": "fixture://atlas/main/docs/adr/0042-queue.md", "supersedes": "decision-atlas-queue-v1"},
            {"key": "decision-atlas-http-client-v1", "kind": "decision", "status": "superseded", "scope": {"type": "repository", "remote_url": "git@github.com:acme/atlas.git"}, "content": "Use isahc.", "evidence": "fixture://atlas/main/docs/adr/0048-http-client.md"},
            {"key": "decision-atlas-http-client-v2", "kind": "decision", "status": "active", "scope": {"type": "repository", "remote_url": "git@github.com:acme/atlas.git"}, "content": "Use reqwest with shared timeout middleware.", "evidence": "fixture://atlas/main/docs/adr/0048-http-client.md", "supersedes": "decision-atlas-http-client-v1"},
            {"key": "decision-orbit-runtime", "kind": "decision", "status": "active", "scope": {"type": "project", "project_name": "orbit"}, "content": "Orbit runtime context only."},
            {"key": "procedure-atlas-worker-deploy", "kind": "procedure", "status": "active", "scope": {"type": "repository", "remote_url": "git@github.com:acme/atlas.git"}, "task": "deploy queue worker", "commands": ["./bin/deploy-worker atlas queue-worker"], "prerequisites": {}, "verification_receipt": "fixture://atlas/main/evidence/deploy-worker-success.json", "evidence": "fixture://atlas/main/runbooks/deploy-worker.md"},
            {"key": "procedure-orbit-worker-deploy", "kind": "procedure", "status": "active", "scope": {"type": "repository", "remote_url": "git@github.com:acme/orbit.git"}, "task": "deploy worker fast", "commands": ["orbitctl worker deploy --fast"], "prerequisites": {"orbitctl.version": "9"}, "content": "ORBIT_ONLY_CANARY", "verification_receipt": "fixture://orbit/main/evidence/deploy-worker-success.json", "evidence": "fixture://orbit/main/runbooks/deploy-worker.md"},
            {"key": "procedure-atlas-integration-v2", "kind": "procedure", "status": "superseded", "scope": {"type": "repository", "remote_url": "git@github.com:acme/atlas.git"}, "task": "run queue worker integration test", "commands": ["./bin/integration-test --worker queue"], "prerequisites": {"tool.version": "2"}},
            {"key": "procedure-atlas-integration-v3", "kind": "procedure", "status": "active", "scope": {"type": "repository", "remote_url": "git@github.com:acme/atlas.git"}, "task": "run queue worker integration test", "commands": ["./bin/integration-test --worker queue"], "prerequisites": {"tool.version": "3"}, "verification_receipt": "fixture://atlas/main/evidence/integration-v3-success.json", "supersedes": "procedure-atlas-integration-v2"},
            {"key": "plan-atlas-worker-previous", "kind": "handoff", "status": "superseded", "scope": {"type": "project", "project_name": "atlas"}, "content": "Previous plan: investigate Redis Streams."},
            {"key": "plan-atlas-worker-current", "kind": "handoff", "status": "active", "scope": {"type": "project", "project_name": "atlas"}, "content": "Next action: update the queue worker consumer for JetStream and run the integration test.", "evidence": "fixture://atlas/main/.git/HEAD", "supersedes": "plan-atlas-worker-previous"},
            {"key": "plan-orbit-current", "kind": "handoff", "status": "active", "scope": {"type": "project", "project_name": "orbit"}, "content": "Continue Orbit worker migration."},
            {"key": "deleted-canary-memory", "kind": "project_fact", "status": "active", "scope": {"type": "project", "project_name": "atlas"}, "content": "ENGRAM_DELETE_CANARY_b48c22d1"}
        ]
    })
}

fn write_files(root: &Path, files: &[FixtureFile]) -> EvalResult<()> {
    for file in files {
        let path = root.join(file.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, file.body)?;
        if file.executable {
            set_executable(&path)?;
        }
    }
    Ok(())
}

fn init_git_repository(root: &Path, remote: &str, message: &str) -> EvalResult<()> {
    git(root, &["init", "-q", "-b", "main"])?;
    git(root, &["config", "user.name", "Engram Eval"])?;
    git(
        root,
        &["config", "user.email", "engram-eval@example.invalid"],
    )?;
    git(root, &["remote", "add", "origin", remote])?;
    git(root, &["add", "."])?;
    git_with_fixed_identity(root, &["commit", "-q", "-m", message])
}

fn clone_repository(source: &Path, target: &Path) -> EvalResult<()> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    let output = Command::new("git")
        .args(["clone", "-q", "--no-hardlinks"])
        .arg(source)
        .arg(target)
        .output()?;
    command_succeeded("git clone", output)
}

fn git(root: &Path, args: &[&str]) -> EvalResult<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()?;
    command_succeeded("git", output)
}

fn git_with_fixed_identity(root: &Path, args: &[&str]) -> EvalResult<()> {
    let output = Command::new("git")
        .args(["-c", "commit.gpgsign=false"])
        .arg("-C")
        .arg(root)
        .args(args)
        .env("GIT_AUTHOR_DATE", FIXED_GIT_DATE)
        .env("GIT_COMMITTER_DATE", FIXED_GIT_DATE)
        .output()?;
    command_succeeded("git", output)
}

fn command_succeeded(label: &str, output: std::process::Output) -> EvalResult<()> {
    if output.status.success() {
        return Ok(());
    }
    Err(EvalError::Invalid(format!(
        "{label} failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    )))
}

fn fixture_revision(groups: &[(&str, &[FixtureFile])], seed_plan: &str) -> String {
    let mut entries = groups
        .iter()
        .flat_map(|(prefix, files)| {
            files
                .iter()
                .map(move |file| (format!("{prefix}/{}", file.path), file.body))
        })
        .collect::<Vec<_>>();
    entries.push(("seed-plan.json".to_string(), seed_plan));
    entries.sort_by(|left, right| left.0.cmp(&right.0));

    let mut hasher = Sha256::new();
    for (path, body) in entries {
        hasher.update(path.as_bytes());
        hasher.update([0]);
        hasher.update(body.as_bytes());
        hasher.update([0]);
    }
    format!("{:x}", hasher.finalize())
}

fn absolute(path: &Path) -> EvalResult<String> {
    Ok(path.canonicalize()?.display().to_string())
}

#[cfg(unix)]
fn set_executable(path: &Path) -> EvalResult<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> EvalResult<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn materializes_reproducible_git_fixture_without_overwriting() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("fixture");
        let layout = materialize_engineering_context_fixture(&root).unwrap();

        assert_eq!(layout.schema_version, 1);
        assert_eq!(layout.fixture_revision.len(), 64);
        assert_eq!(layout.checkouts.len(), 7);
        assert!(Path::new(&layout.seed_plan).is_file());

        let atlas = PathBuf::from(&layout.checkouts["fixture://atlas/main"]);
        let moved = PathBuf::from(&layout.checkouts["fixture://moved/arbitrary-name"]);
        assert_eq!(
            git_output(&atlas, &["rev-parse", "HEAD"]),
            git_output(&moved, &["rev-parse", "HEAD"])
        );
        assert_eq!(
            git_output(&moved, &["config", "--get", "remote.origin.url"]),
            "https://github.com/acme/atlas"
        );
        assert!(atlas.join("evidence/integration-v3-success.json").is_file());
        let orbit = PathBuf::from(&layout.checkouts["fixture://orbit/main"]);
        assert_eq!(
            fs::read_to_string(orbit.join("services/worker/component.json")).unwrap(),
            "{\"name\":\"queue-worker\",\"kind\":\"service\"}\n"
        );
        assert_eq!(
            git_output(
                &orbit,
                &[
                    "ls-files",
                    "--error-unmatch",
                    "services/worker/component.json"
                ]
            ),
            "services/worker/component.json"
        );

        let second = materialize_engineering_context_fixture(&root).unwrap_err();
        assert!(second.to_string().contains("not empty"));
    }

    #[test]
    fn fixture_revision_is_independent_of_materialization_path() {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        let first = materialize_engineering_context_fixture(&first.path().join("one")).unwrap();
        let second = materialize_engineering_context_fixture(&second.path().join("two")).unwrap();
        assert_eq!(first.fixture_revision, second.fixture_revision);
    }

    fn git_output(root: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .unwrap();
        assert!(output.status.success());
        String::from_utf8(output.stdout).unwrap().trim().to_string()
    }
}
