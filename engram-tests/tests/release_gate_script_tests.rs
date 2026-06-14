use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

use serde_json::Value;
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

    let output = Command::new("bash")
        .args([
            "scripts/release-gate-report.sh",
            "--target",
            "ga",
            "--verify-generated-output-cleanup",
            "cleanup-manifest.json",
            "--json",
        ])
        .current_dir(&repo)
        .output()
        .expect("release gate should run");

    assert!(
        !output.status.success(),
        "branch-sync failure should stop the release gate"
    );

    let report: Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|err| {
        panic!(
            "stdout should be JSON: {err}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    });

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
