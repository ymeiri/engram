use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

use serde_json::{json, Value};
use tempfile::TempDir;

fn run(command: &mut Command) -> Output {
    let output = command.output().expect("command should run");
    if !output.status.success() {
        panic!(
            "command failed\nstatus: {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    output
}

fn write_minimal_workspace(repo: &Path) {
    fs::create_dir_all(repo.join("engram-cli/src")).unwrap();
    fs::write(
        repo.join("Cargo.toml"),
        r#"[workspace]
members = ["engram-cli"]
resolver = "2"

[workspace.package]
version = "0.2.0"
edition = "2021"
"#,
    )
    .unwrap();
    fs::write(
        repo.join("engram-cli/Cargo.toml"),
        r#"[package]
name = "engram-cli"
version.workspace = true
edition.workspace = true
"#,
    )
    .unwrap();
    fs::write(repo.join("engram-cli/src/lib.rs"), "").unwrap();
    run(Command::new("cargo")
        .arg("generate-lockfile")
        .current_dir(repo));
}

fn write_release_gate_script(repo: &Path) {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("scripts/release-gate-report.sh");
    let scripts_dir = repo.join("scripts");
    fs::create_dir_all(&scripts_dir).unwrap();
    let dest = scripts_dir.join("release-gate-report.sh");
    fs::copy(source, &dest).unwrap();
}

fn git(repo: &Path, args: &[&str]) {
    run(Command::new("git").args(args).current_dir(repo));
}

fn command_stdout(command: &mut Command) -> String {
    let output = run(command);
    String::from_utf8(output.stdout)
        .expect("command stdout should be UTF-8")
        .trim()
        .to_string()
}

fn git_stdout(repo: &Path, args: &[&str]) -> String {
    command_stdout(Command::new("git").args(args).current_dir(repo))
}

fn rust_host_triple() -> String {
    let output = run(Command::new("rustc").arg("-vV"));
    let stdout = String::from_utf8(output.stdout).expect("rustc -vV stdout should be UTF-8");

    stdout
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .expect("rustc -vV should include host triple")
        .to_string()
}

fn sha256(path: &Path) -> String {
    command_stdout(Command::new("shasum").args(["-a", "256"]).arg(path))
        .split_whitespace()
        .next()
        .expect("shasum output should include a digest")
        .to_string()
}

fn relative_path(repo: &Path, path: &Path) -> String {
    path.strip_prefix(repo)
        .expect("path should be under repo")
        .to_string_lossy()
        .to_string()
}

fn generated_output_json(repo: &Path, kind: &str, path: &Path, overwrite_env: &str) -> Value {
    let absolute_path = fs::canonicalize(path).unwrap();
    json!({
        "kind": kind,
        "path": relative_path(repo, path),
        "absolute_path": absolute_path.to_string_lossy(),
        "exists": true,
        "will_write": true,
        "overwrite_env": overwrite_env,
        "file_type": "file",
        "size_bytes": fs::metadata(path).unwrap().len(),
        "sha256": sha256(path),
    })
}

fn write_ga_generated_outputs(repo: &Path) -> Vec<Value> {
    let host_triple = rust_host_triple();
    let archive_name = format!("engram-0.2.0-{host_triple}");
    let archive = repo.join("dist").join(format!("{archive_name}.tar.gz"));
    let checksum = repo
        .join("dist")
        .join(format!("{archive_name}.tar.gz.sha256"));
    let formula = repo.join("dist/homebrew/Formula/engram.rb");

    fs::create_dir_all(archive.parent().unwrap()).unwrap();
    fs::create_dir_all(formula.parent().unwrap()).unwrap();
    fs::write(&archive, "fake release archive\n").unwrap();
    fs::write(&checksum, "fake release checksum\n").unwrap();
    fs::write(&formula, "class Engram < Formula\nend\n").unwrap();

    vec![
        generated_output_json(
            repo,
            "package_archive",
            &archive,
            "ALLOW_PACKAGE_ASSET_OVERWRITE",
        ),
        generated_output_json(
            repo,
            "package_checksum",
            &checksum,
            "ALLOW_PACKAGE_ASSET_OVERWRITE",
        ),
        generated_output_json(
            repo,
            "homebrew_formula",
            &formula,
            "ALLOW_HOMEBREW_FORMULA_OVERWRITE",
        ),
    ]
}

fn write_cleanup_manifest(repo: &Path, outputs: Vec<Value>) {
    let head = git_stdout(repo, &["rev-parse", "HEAD"]);
    let manifest = json!({
        "target": "ga",
        "package_version": "0.2.0",
        "release_version": "0.2.0",
        "branch": "main",
        "head": head,
        "release_gate_state": "generated_outputs_cleanup_required",
        "failure": {
            "kind": "generated_outputs_preflight",
        },
        "release_target": {
            "tag": "v0.2.0",
            "repository": "ymeiri/engram",
            "state": "available",
            "local_tag_exists": false,
            "remote_git_tag_exists": false,
            "github_release_exists": false,
        },
        "hosted_ci": {
            "state": "passing",
            "repository": "ymeiri/engram",
            "expected_workflow": "CI",
            "expected_event": "push",
            "run_id": 1,
            "run": {
                "status": "completed",
                "conclusion": "success",
                "headSha": head,
                "event": "push",
                "workflowName": "CI",
            },
        },
        "disk_space": {
            "state": "passed",
            "shortfall_kib": 0,
        },
        "generated_outputs": {
            "state": "cleanup_required",
            "outputs": outputs,
        },
        "generated_artifacts": {
            "state": "not_checked",
        },
        "ready_for_release_owner_review": false,
        "release_owner_decision_required": true,
        "hosted_ci_fallback_decision_required": false,
        "remaining_release_actions": [
            "remove_stale_generated_release_outputs_or_get_cleanup_approval",
            "rerun_full_release_gate_report_with_local_ci_and_package_smoke",
        ],
        "actions_performed": {
            "release_actions": false,
            "git_tag": false,
            "github_release": false,
            "package_asset_upload": false,
            "homebrew_tap_update": false,
            "generated_output_cleanup": false,
        },
        "release_actions_performed": false,
    });

    fs::write(
        repo.join("cleanup-manifest.json"),
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
}

fn init_synced_main(repo: &Path, remote: &Path) {
    git(repo, &["init", "--initial-branch=main"]);
    git(repo, &["config", "user.email", "release-test@example.com"]);
    git(repo, &["config", "user.name", "Release Test"]);
    git(repo, &["add", "."]);
    git(repo, &["commit", "-m", "initial"]);

    run(Command::new("git").args(["init", "--bare"]).arg(remote));
    git(repo, &["remote", "add", "origin", remote.to_str().unwrap()]);
    git(repo, &["push", "-u", "origin", "main"]);
}

fn run_ga_release_gate(repo: &Path) -> Output {
    Command::new("bash")
        .args([
            "scripts/release-gate-report.sh",
            "--target",
            "ga",
            "--verify-generated-output-cleanup",
            "cleanup-manifest.json",
            "--json",
        ])
        .current_dir(repo)
        .output()
        .expect("release gate should run")
}

fn parse_json_report(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|err| {
        panic!(
            "stdout should be JSON: {err}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

#[test]
fn ga_release_gate_branch_sync_failure_warns_against_bare_pull() {
    let temp = TempDir::new().unwrap();
    let repo = temp.path().join("repo");
    let remote = temp.path().join("remote.git");
    fs::create_dir(&repo).unwrap();

    write_minimal_workspace(&repo);
    write_release_gate_script(&repo);
    fs::write(repo.join("cleanup-manifest.json"), "{}").unwrap();

    git(&repo, &["init", "--initial-branch=main"]);
    git(&repo, &["config", "user.email", "release-test@example.com"]);
    git(&repo, &["config", "user.name", "Release Test"]);
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "initial"]);

    run(Command::new("git").args(["init", "--bare"]).arg(&remote));
    git(
        &repo,
        &["remote", "add", "origin", remote.to_str().unwrap()],
    );
    git(&repo, &["push", "-u", "origin", "main"]);

    fs::write(repo.join("local-only.txt"), "local release-head change\n").unwrap();
    git(&repo, &["add", "local-only.txt"]);
    git(&repo, &["commit", "-m", "local-only"]);

    let output = run_ga_release_gate(&repo);

    assert!(
        !output.status.success(),
        "branch-sync failure should stop the release gate"
    );

    let report = parse_json_report(&output);

    assert_eq!(report["release_gate_state"], "branch_sync_required");
    assert_eq!(report["failure"]["kind"], "branch_sync_preflight");
    assert_eq!(report["upstream"]["name"], "origin/main");
    assert_eq!(report["upstream"]["ahead"], 1);
    assert_eq!(report["upstream"]["behind"], 0);
    assert!(!report["release_actions_performed"].as_bool().unwrap());
    assert!(!report["actions_performed"]["generated_output_cleanup"]
        .as_bool()
        .unwrap());

    let failure_message = report["failure"]["message"].as_str().unwrap();
    assert!(failure_message.contains("do not use git pull as release approval"));
    assert!(failure_message.contains("fresh exact-head CI plus gate"));
}

#[test]
fn ga_release_gate_rejects_stale_remote_tracking_ref() {
    let temp = TempDir::new().unwrap();
    let repo = temp.path().join("repo");
    let remote = temp.path().join("remote.git");
    let peer = temp.path().join("peer");
    fs::create_dir(&repo).unwrap();

    write_minimal_workspace(&repo);
    write_release_gate_script(&repo);
    fs::write(repo.join("cleanup-manifest.json"), "{}").unwrap();

    git(&repo, &["init", "--initial-branch=main"]);
    git(&repo, &["config", "user.email", "release-test@example.com"]);
    git(&repo, &["config", "user.name", "Release Test"]);
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "initial"]);

    run(Command::new("git").args(["init", "--bare"]).arg(&remote));
    git(
        &repo,
        &["remote", "add", "origin", remote.to_str().unwrap()],
    );
    git(&repo, &["push", "-u", "origin", "main"]);

    run(Command::new("git")
        .args(["clone", "--branch", "main"])
        .arg(&remote)
        .arg(&peer));
    git(&peer, &["config", "user.email", "release-test@example.com"]);
    git(&peer, &["config", "user.name", "Release Test"]);
    fs::write(peer.join("remote-only.txt"), "remote release-head change\n").unwrap();
    git(&peer, &["add", "remote-only.txt"]);
    git(&peer, &["commit", "-m", "remote-only"]);
    git(&peer, &["push", "origin", "main"]);

    let output = run_ga_release_gate(&repo);

    assert!(
        !output.status.success(),
        "stale remote-tracking refs should stop the release gate"
    );

    let report = parse_json_report(&output);

    assert_eq!(report["release_gate_state"], "branch_sync_required");
    assert_eq!(report["failure"]["kind"], "branch_sync_preflight");
    assert_eq!(report["upstream"]["name"], "origin/main");
    assert_eq!(report["upstream"]["ahead"], 0);
    assert_eq!(report["upstream"]["behind"], 0);
    assert!(!report["upstream"]["matches_remote_head"].as_bool().unwrap());
    assert_ne!(report["upstream"]["remote_head"], report["head"]);
    assert!(!report["release_actions_performed"].as_bool().unwrap());
    assert!(!report["actions_performed"]["generated_output_cleanup"]
        .as_bool()
        .unwrap());

    let failure_message = report["failure"]["message"].as_str().unwrap();
    assert!(failure_message.contains("branch is not synced with remote origin/main"));
    assert!(failure_message.contains("do not use git pull as release approval"));
    assert!(failure_message.contains("fresh exact-head CI plus gate"));
}

#[test]
fn ga_release_gate_verifies_generated_output_cleanup_manifest() {
    let temp = TempDir::new().unwrap();
    let repo = temp.path().join("repo");
    let remote = temp.path().join("remote.git");
    fs::create_dir(&repo).unwrap();

    write_minimal_workspace(&repo);
    write_release_gate_script(&repo);
    init_synced_main(&repo, &remote);
    let outputs = write_ga_generated_outputs(&repo);
    write_cleanup_manifest(&repo, outputs);

    let output = run_ga_release_gate(&repo);

    assert!(
        output.status.success(),
        "matching cleanup manifest should pass:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let report = parse_json_report(&output);

    assert_eq!(
        report["release_gate_state"],
        "generated_output_cleanup_fingerprints_verified"
    );
    assert_eq!(
        report["generated_output_cleanup_verification"]["state"],
        "verified"
    );
    assert_eq!(
        report["generated_output_cleanup_verification"]["manifest_path"],
        "cleanup-manifest.json"
    );
    assert_eq!(
        report["generated_output_cleanup_verification"]["expected_outputs"],
        report["generated_output_cleanup_verification"]["current_outputs"]
    );
    assert!(!report["release_actions_performed"].as_bool().unwrap());
    assert!(!report["actions_performed"]["generated_output_cleanup"]
        .as_bool()
        .unwrap());
}

#[test]
fn ga_release_gate_cleanup_fingerprint_ignores_manifest_metadata_changes() {
    let temp = TempDir::new().unwrap();
    let repo = temp.path().join("repo");
    let remote = temp.path().join("remote.git");
    fs::create_dir(&repo).unwrap();

    write_minimal_workspace(&repo);
    write_release_gate_script(&repo);
    init_synced_main(&repo, &remote);
    let outputs = write_ga_generated_outputs(&repo);
    write_cleanup_manifest(&repo, outputs);

    let first_output = run_ga_release_gate(&repo);
    assert!(
        first_output.status.success(),
        "matching cleanup manifest should pass:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&first_output.stdout),
        String::from_utf8_lossy(&first_output.stderr)
    );
    let first_report = parse_json_report(&first_output);

    let manifest_path = repo.join("cleanup-manifest.json");
    let mut manifest: Value = serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    manifest["disk_space"]["min_required_kib"] = json!(10_485_760);
    manifest["disk_space"]["free_kib"] = json!(20_971_520);
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();

    let second_output = run_ga_release_gate(&repo);
    assert!(
        second_output.status.success(),
        "cleanup manifest with metadata-only changes should pass:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&second_output.stdout),
        String::from_utf8_lossy(&second_output.stderr)
    );
    let second_report = parse_json_report(&second_output);

    let first_verification = &first_report["generated_output_cleanup_verification"];
    let second_verification = &second_report["generated_output_cleanup_verification"];

    assert_ne!(
        first_verification["manifest_sha256"],
        second_verification["manifest_sha256"]
    );
    assert_eq!(
        first_verification["cleanup_fingerprint_sha256"],
        second_verification["cleanup_fingerprint_sha256"]
    );
    assert_eq!(
        first_verification["expected_outputs_sha256"],
        first_verification["current_outputs_sha256"]
    );
    assert_eq!(
        second_verification["expected_outputs_sha256"],
        second_verification["current_outputs_sha256"]
    );
    assert!(first_verification["cleanup_fingerprint_sha256"]
        .as_str()
        .unwrap()
        .chars()
        .all(|ch| ch.is_ascii_hexdigit()));
}

#[test]
fn ga_release_gate_rejects_cleanup_manifest_with_release_action_performed() {
    let temp = TempDir::new().unwrap();
    let repo = temp.path().join("repo");
    let remote = temp.path().join("remote.git");
    fs::create_dir(&repo).unwrap();

    write_minimal_workspace(&repo);
    write_release_gate_script(&repo);
    init_synced_main(&repo, &remote);
    let outputs = write_ga_generated_outputs(&repo);
    write_cleanup_manifest(&repo, outputs);

    let manifest_path = repo.join("cleanup-manifest.json");
    let mut manifest: Value = serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    manifest["actions_performed"]["generated_output_cleanup"] = json!(true);
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();

    let output = run_ga_release_gate(&repo);

    assert!(
        !output.status.success(),
        "cleanup manifest with prior cleanup action should fail"
    );

    let report = parse_json_report(&output);

    assert_eq!(
        report["release_gate_state"],
        "generated_output_cleanup_fingerprints_mismatch"
    );
    assert_eq!(
        report["failure"]["kind"],
        "generated_output_cleanup_verification"
    );
    assert_eq!(
        report["generated_output_cleanup_verification"]["state"],
        "mismatch"
    );

    let failure_message = report["failure"]["message"].as_str().unwrap();
    assert!(failure_message.contains("actions_performed release-action values"));
    assert!(!report["release_actions_performed"].as_bool().unwrap());
    assert!(!report["actions_performed"]["generated_output_cleanup"]
        .as_bool()
        .unwrap());
}

#[test]
fn ga_release_gate_rejects_cleanup_manifest_when_output_fingerprint_changes() {
    let temp = TempDir::new().unwrap();
    let repo = temp.path().join("repo");
    let remote = temp.path().join("remote.git");
    fs::create_dir(&repo).unwrap();

    write_minimal_workspace(&repo);
    write_release_gate_script(&repo);
    init_synced_main(&repo, &remote);
    let outputs = write_ga_generated_outputs(&repo);
    write_cleanup_manifest(&repo, outputs);

    let host_triple = rust_host_triple();
    let archive = repo
        .join("dist")
        .join(format!("engram-0.2.0-{host_triple}.tar.gz"));
    fs::write(archive, "changed fake release archive\n").unwrap();

    let output = run_ga_release_gate(&repo);

    assert!(
        !output.status.success(),
        "cleanup manifest with changed output fingerprint should fail"
    );

    let report = parse_json_report(&output);

    assert_eq!(
        report["release_gate_state"],
        "generated_output_cleanup_fingerprints_mismatch"
    );
    assert_eq!(
        report["generated_output_cleanup_verification"]["state"],
        "mismatch"
    );
    assert_eq!(
        report["failure"]["message"],
        "current generated outputs do not match the cleanup manifest"
    );
    assert_ne!(
        report["generated_output_cleanup_verification"]["expected_outputs"],
        report["generated_output_cleanup_verification"]["current_outputs"]
    );
    assert!(!report["release_actions_performed"].as_bool().unwrap());
    assert!(!report["actions_performed"]["generated_output_cleanup"]
        .as_bool()
        .unwrap());
}
