//! Provider-free adversarial tests for the stale-safety isolation successor foundation.

pub use engram_eval::{EvalError, EvalResult};

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
#[path = "../src/native_isolation.rs"]
mod native_isolation;

use native_isolation::*;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn serialized_digest(value: &impl serde::Serialize) -> String {
    digest(&serde_json::to_vec(value).unwrap())
}

fn mkdir_private(path: &Path) {
    fs::create_dir_all(path).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
    }
}

fn write_mode(path: &Path, bytes: &[u8], mode: u32) {
    if let Some(parent) = path.parent() {
        mkdir_private(parent);
    }
    fs::write(path, bytes).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
    }
}

fn sandbox_audit(label: &str, succeeded: bool, denial: bool) -> IsolationSandboxCommandAudit {
    let stderr = if denial {
        b"Operation not permitted".as_slice()
    } else {
        b"".as_slice()
    };
    IsolationSandboxCommandAudit {
        label: label.to_string(),
        argv_sha256: digest(label.as_bytes()),
        exit_code: if succeeded { 0 } else { 1 },
        succeeded,
        sandbox_denial_evidence: denial,
        stdout_sha256: digest(b""),
        stderr_sha256: digest(stderr),
    }
}

fn sandbox_audit_with_identity(
    label: &str,
    identity: &str,
    succeeded: bool,
    denial: bool,
) -> IsolationSandboxCommandAudit {
    let mut audit = sandbox_audit(label, succeeded, denial);
    audit.argv_sha256 = digest(identity.as_bytes());
    audit
}

fn successful_network_control() -> IsolationSandboxCommandAudit {
    sandbox_audit("network_control", true, false)
}

fn codex_probe(profile: &CodexPermissionProfile, profile_bytes: &[u8]) -> CodexSandboxProbeAudit {
    CodexSandboxProbeAudit {
        valid: true,
        profile_sha256: digest(profile_bytes),
        profile: CodexPermissionProbe {
            config_layer_name: profile.config_layer_name.clone(),
            permission_profile_name: profile.permission_profile_name.clone(),
            allowed_read_succeeded: true,
            sibling_traversal_denied: true,
            direct_forbidden_read_denied: true,
            child_shell_forbidden_read_denied: true,
            write_denied: true,
            network_denied: true,
            network_sandbox_denial_observed: true,
        },
        network_control: successful_network_control(),
        commands: vec![
            sandbox_audit("allowed_read", true, false),
            sandbox_audit("sibling_traversal", false, true),
            sandbox_audit("direct_forbidden_read", false, true),
            sandbox_audit("child_shell_forbidden_read", false, true),
            sandbox_audit("write_denial", false, true),
            sandbox_audit("network_denial", false, false),
            sandbox_audit("network_policy_denial", false, false),
            sandbox_audit("network_errno_denial", false, true),
        ],
    }
}

fn claude_probe(spec: &ClaudeSeatbeltSpec, profile_bytes: &[u8]) -> ClaudeSeatbeltProbeAudit {
    let mut commands = vec![
        sandbox_audit_with_identity("profile_compile", "profile_compile", true, false),
        sandbox_audit_with_identity("allowed_read", "allowed_read", true, false),
    ];
    commands.extend(
        spec.forbidden_read_paths
            .iter()
            .enumerate()
            .map(|(index, _)| {
                sandbox_audit_with_identity(
                    "forbidden_read",
                    &format!("forbidden_read:{index}"),
                    false,
                    true,
                )
            }),
    );
    commands.extend(
        spec.forbidden_write_paths
            .iter()
            .enumerate()
            .map(|(index, _)| {
                sandbox_audit_with_identity(
                    "forbidden_write",
                    &format!("forbidden_write:{index}"),
                    false,
                    true,
                )
            }),
    );
    commands.push(sandbox_audit_with_identity(
        "provider_proxy_allowed",
        "provider_proxy_allowed",
        true,
        false,
    ));
    if spec.transport.engram_daemon_port.is_some() {
        commands.push(sandbox_audit_with_identity(
            "engram_loopback_allowed",
            "engram_loopback_allowed",
            true,
            false,
        ));
    }
    commands.push(sandbox_audit_with_identity(
        "network_denial",
        "network_denial",
        false,
        false,
    ));
    commands.push(sandbox_audit_with_identity(
        "network_errno_denial",
        "network_errno_denial",
        false,
        true,
    ));
    commands.push(sandbox_audit_with_identity(
        "external_network_denial",
        "external_network_denial",
        false,
        true,
    ));
    commands.push(sandbox_audit_with_identity(
        "unexpected_bind_denial",
        "unexpected_bind_denial",
        false,
        true,
    ));
    ClaudeSeatbeltProbeAudit {
        valid: true,
        profile_sha256: digest(profile_bytes),
        profile: ClaudeSeatbeltProbe {
            sandbox_exec_present: true,
            profile_compiled: true,
            allowed_evaluation_canary_read: true,
            provider_proxy_loopback_allowed: true,
            engram_loopback_allowed: spec.transport.engram_daemon_port.map(|_| true),
            forbidden_reads: spec
                .forbidden_read_paths
                .iter()
                .map(|path| (path.clone(), IsolationProbeOutcome::PermissionDenied))
                .collect(),
            forbidden_writes: spec
                .forbidden_write_paths
                .iter()
                .map(|path| (path.clone(), IsolationProbeOutcome::PermissionDenied))
                .collect(),
            network_denied: true,
            adjacent_loopback_denied: true,
            external_network_denied: true,
            unexpected_bind_denied: true,
            network_sandbox_denial_observed: true,
        },
        network_control: successful_network_control(),
        commands,
    }
}

struct ManifestFixture {
    _temp: tempfile::TempDir,
    source: PathBuf,
    evaluation: PathBuf,
    boundary: IsolationBoundaryContract,
    manifest: IsolationCopyManifest,
}

fn manifest_fixture() -> ManifestFixture {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let evaluation = temp.path().join("evaluation-enclave");
    mkdir_private(&source);
    mkdir_private(&source.join("run"));
    mkdir_private(&source.join("teaching"));
    mkdir_private(&evaluation);
    write_mode(
        &evaluation.join("codex-home/auth.json"),
        b"synthetic-fixture-only\n",
        0o600,
    );
    let source = source.canonicalize().unwrap();
    let evaluation = evaluation.canonicalize().unwrap();
    let marker = "STALE-MARKER-0001".to_string();
    let boundary = IsolationBoundaryContract {
        schema_version: 1,
        host: IsolationHost::Codex,
        native_memory_enabled: true,
        uses_engram: true,
        source_root: source.display().to_string(),
        evaluation_root: evaluation.display().to_string(),
        forbidden_paths: vec![
            IsolationBoundaryPath {
                class: IsolationBoundaryPathClass::RunArtifacts,
                path: source.join("run").display().to_string(),
            },
            IsolationBoundaryPath {
                class: IsolationBoundaryPathClass::TeachingArtifacts,
                path: source.join("teaching").display().to_string(),
            },
            IsolationBoundaryPath {
                class: IsolationBoundaryPathClass::CredentialMaterial,
                path: evaluation
                    .join("codex-home/auth.json")
                    .display()
                    .to_string(),
            },
            IsolationBoundaryPath {
                class: IsolationBoundaryPathClass::NativeState,
                path: evaluation.join("codex-home/memories").display().to_string(),
            },
            IsolationBoundaryPath {
                class: IsolationBoundaryPathClass::EngramState,
                path: evaluation
                    .join("engram-home/projects/isolated-stale-eval")
                    .display()
                    .to_string(),
            },
        ],
        forbidden_markers: vec![marker.clone()],
    };
    let files = [
        (
            IsolationCopyClass::EvaluationCheckout,
            "fresh-checkout/README.md",
            "checkout/README.md",
            b"fresh evaluation checkout\n".as_slice(),
        ),
        (
            IsolationCopyClass::OutputSchema,
            "marker-free/schema/output.json",
            "contracts/schema/output.json",
            b"{\"type\":\"object\"}\n".as_slice(),
        ),
        (
            IsolationCopyClass::Settings,
            "marker-free/settings/settings.json",
            "contracts/settings/settings.json",
            b"{}\n".as_slice(),
        ),
        (
            IsolationCopyClass::McpConfig,
            "marker-free/mcp/mcp.json",
            "contracts/mcp/mcp.json",
            b"{\"mcpServers\":{}}\n".as_slice(),
        ),
        (
            IsolationCopyClass::Instructions,
            "marker-free/instructions/AGENTS.md",
            "contracts/instructions/AGENTS.md",
            b"Use only evaluation evidence.\n".as_slice(),
        ),
        (
            IsolationCopyClass::CodexMemories,
            "codex-home/memories/MEMORY.md",
            "codex-home/memories/MEMORY.md",
            b"Current safe native memory.\n".as_slice(),
        ),
        (
            IsolationCopyClass::EngramState,
            "engram-home/projects/isolated-stale-eval/db.bin",
            "engram-home/projects/isolated-stale-eval/db.bin",
            b"isolated-engram-state\n".as_slice(),
        ),
    ];
    let mut entries = Vec::new();
    for (class, source_path, destination, bytes) in files {
        write_mode(&source.join(source_path), bytes, 0o600);
        entries.push(IsolationCopyEntry {
            class,
            source: source_path.to_string(),
            destination: destination.to_string(),
            sha256: digest(bytes),
            size_bytes: bytes.len() as u64,
            mode: 0o600,
        });
    }
    let zones = vec![
        (
            IsolationCopyClass::EvaluationCheckout,
            "fresh-checkout",
            "checkout",
        ),
        (
            IsolationCopyClass::OutputSchema,
            "marker-free/schema",
            "contracts/schema",
        ),
        (
            IsolationCopyClass::Settings,
            "marker-free/settings",
            "contracts/settings",
        ),
        (
            IsolationCopyClass::McpConfig,
            "marker-free/mcp",
            "contracts/mcp",
        ),
        (
            IsolationCopyClass::Instructions,
            "marker-free/instructions",
            "contracts/instructions",
        ),
        (
            IsolationCopyClass::CodexMemories,
            "codex-home/memories",
            "codex-home/memories",
        ),
        (
            IsolationCopyClass::EngramState,
            "engram-home/projects/isolated-stale-eval",
            "engram-home/projects/isolated-stale-eval",
        ),
    ]
    .into_iter()
    .map(
        |(class, source_prefix, destination_prefix)| IsolationCopyZone {
            class,
            source_prefix: source_prefix.to_string(),
            destination_prefix: destination_prefix.to_string(),
        },
    )
    .collect();
    let manifest = IsolationCopyManifest {
        schema_version: 1,
        host: IsolationHost::Codex,
        native_memory_enabled: true,
        uses_engram: true,
        source_root: source.display().to_string(),
        evaluation_root: evaluation.display().to_string(),
        max_file_bytes: 1024,
        max_total_bytes: 4096,
        zones,
        entries,
        forbidden_markers: vec![marker],
        forbidden_content_literals: vec!["source-only-run-plan".to_string()],
        boundary_sha256: serialized_digest(&boundary),
    };
    ManifestFixture {
        _temp: temp,
        source,
        evaluation,
        boundary,
        manifest,
    }
}

#[test]
fn materializes_exact_closed_world_state_and_detects_drift() {
    let fixture = manifest_fixture();
    let audit = materialize_isolation_enclave(&fixture.manifest, &fixture.boundary).unwrap();
    assert_eq!(audit.file_count, 7);
    let after = snapshot_isolation_state(&fixture.manifest, &fixture.boundary).unwrap();
    require_isolation_state_unchanged(&audit.state, &after).unwrap();
    write_mode(
        &fixture.evaluation.join("codex-home/memories/MEMORY.md"),
        b"drift\n",
        0o600,
    );
    assert!(snapshot_isolation_state(&fixture.manifest, &fixture.boundary).is_err());
}

#[test]
fn semantic_state_gate_rejects_caller_fabricated_engram_evidence() {
    let fixture = manifest_fixture();
    materialize_isolation_enclave(&fixture.manifest, &fixture.boundary).unwrap();
    assert!(build_engram_semantic_evidence(
        "isolated-stale-eval",
        &digest(b"claimed read-only semantic export contract"),
        &digest(b"claimed canonical memory records"),
        7,
        &digest(b"claimed raw RocksDB state"),
        1_780_000_000_000,
    )
    .is_err());
    assert!(snapshot_isolation_semantic_state(
        &fixture.manifest,
        &fixture.boundary,
        Some("isolated-stale-eval"),
        None,
    )
    .is_err());

    let mut fixture = manifest_fixture();
    fixture.boundary.uses_engram = false;
    fixture
        .boundary
        .forbidden_paths
        .retain(|path| path.class != IsolationBoundaryPathClass::EngramState);
    fixture.manifest.uses_engram = false;
    fixture
        .manifest
        .zones
        .retain(|zone| zone.class != IsolationCopyClass::EngramState);
    fixture
        .manifest
        .entries
        .retain(|entry| entry.class != IsolationCopyClass::EngramState);
    fixture.manifest.boundary_sha256 = serialized_digest(&fixture.boundary);
    materialize_isolation_enclave(&fixture.manifest, &fixture.boundary).unwrap();
    let before =
        snapshot_isolation_semantic_state(&fixture.manifest, &fixture.boundary, None, None)
            .unwrap();
    let after = snapshot_isolation_semantic_state(&fixture.manifest, &fixture.boundary, None, None)
        .unwrap();
    require_isolation_semantic_state_unchanged(&before, &after).unwrap();
}

#[test]
fn rejects_empty_or_mismatched_markers_and_empty_required_zones() {
    let mut fixture = manifest_fixture();
    fixture.manifest.forbidden_markers.clear();
    assert!(materialize_isolation_enclave(&fixture.manifest, &fixture.boundary).is_err());
    let mut fixture = manifest_fixture();
    fixture
        .manifest
        .entries
        .retain(|entry| entry.class != IsolationCopyClass::Settings);
    assert!(materialize_isolation_enclave(&fixture.manifest, &fixture.boundary).is_err());
    let mut fixture = manifest_fixture();
    fixture.manifest.forbidden_markers[0] = "OTHER-MARKER-0002".to_string();
    assert!(materialize_isolation_enclave(&fixture.manifest, &fixture.boundary).is_err());
}

#[test]
fn rejects_native_or_engram_copy_zones_not_bound_to_exact_launch_state_roots() {
    let mut fixture = manifest_fixture();
    let zone = fixture
        .manifest
        .zones
        .iter_mut()
        .find(|zone| zone.class == IsolationCopyClass::CodexMemories)
        .unwrap();
    zone.destination_prefix = "other/memories".to_string();
    assert!(materialize_isolation_enclave(&fixture.manifest, &fixture.boundary).is_err());

    let mut fixture = manifest_fixture();
    let zone = fixture
        .manifest
        .zones
        .iter_mut()
        .find(|zone| zone.class == IsolationCopyClass::EngramState)
        .unwrap();
    zone.source_prefix = "dummy/projects/isolated-stale-eval".to_string();
    assert!(materialize_isolation_enclave(&fixture.manifest, &fixture.boundary).is_err());
}

#[test]
fn rejects_overlap_traversal_oversize_unlisted_credentials_and_markers() {
    let mut fixture = manifest_fixture();
    fixture.manifest.evaluation_root = fixture.source.join("nested").display().to_string();
    mkdir_private(Path::new(&fixture.manifest.evaluation_root));
    assert!(materialize_isolation_enclave(&fixture.manifest, &fixture.boundary).is_err());
    let mut fixture = manifest_fixture();
    fixture.manifest.entries[0].source = "fresh-checkout/../README.md".to_string();
    assert!(materialize_isolation_enclave(&fixture.manifest, &fixture.boundary).is_err());
    let mut fixture = manifest_fixture();
    fixture.manifest.max_file_bytes = 8;
    fixture.manifest.max_total_bytes = 64;
    assert!(materialize_isolation_enclave(&fixture.manifest, &fixture.boundary).is_err());
    let fixture = manifest_fixture();
    write_mode(&fixture.source.join("fresh-checkout/unlisted"), b"x", 0o600);
    assert!(materialize_isolation_enclave(&fixture.manifest, &fixture.boundary).is_err());
    let mut fixture = manifest_fixture();
    fs::rename(
        fixture.source.join("fresh-checkout/README.md"),
        fixture.source.join("fresh-checkout/auth.json"),
    )
    .unwrap();
    fixture.manifest.entries[0].source = "fresh-checkout/auth.json".to_string();
    fixture.manifest.entries[0].destination = "checkout/auth.json".to_string();
    assert!(materialize_isolation_enclave(&fixture.manifest, &fixture.boundary).is_err());
    let mut fixture = manifest_fixture();
    fs::rename(
        fixture.source.join("fresh-checkout/README.md"),
        fixture.source.join("fresh-checkout/.credentials.json"),
    )
    .unwrap();
    fixture.manifest.entries[0].source = "fresh-checkout/.credentials.json".to_string();
    fixture.manifest.entries[0].destination = "checkout/.credentials.json".to_string();
    assert!(materialize_isolation_enclave(&fixture.manifest, &fixture.boundary).is_err());
    let mut fixture = manifest_fixture();
    let bytes = b"contains STALE-MARKER-0001\n";
    write_mode(
        &fixture.source.join("fresh-checkout/README.md"),
        bytes,
        0o600,
    );
    fixture.manifest.entries[0].sha256 = digest(bytes);
    fixture.manifest.entries[0].size_bytes = bytes.len() as u64;
    assert!(materialize_isolation_enclave(&fixture.manifest, &fixture.boundary).is_err());
}

#[cfg(unix)]
#[test]
fn rejects_symlink_hardlink_nonregular_and_public_enclave() {
    use std::os::unix::fs::{symlink, PermissionsExt};
    use std::os::unix::net::UnixListener;
    let fixture = manifest_fixture();
    symlink(
        fixture.source.join("fresh-checkout/README.md"),
        fixture.source.join("fresh-checkout/link"),
    )
    .unwrap();
    assert!(materialize_isolation_enclave(&fixture.manifest, &fixture.boundary).is_err());
    let fixture = manifest_fixture();
    fs::hard_link(
        fixture.source.join("fresh-checkout/README.md"),
        fixture.source.join("hardlink-outside-zone"),
    )
    .unwrap();
    assert!(materialize_isolation_enclave(&fixture.manifest, &fixture.boundary).is_err());
    let fixture = manifest_fixture();
    let _socket = UnixListener::bind(fixture.source.join("fresh-checkout/socket")).unwrap();
    assert!(materialize_isolation_enclave(&fixture.manifest, &fixture.boundary).is_err());
    let fixture = manifest_fixture();
    fs::set_permissions(&fixture.evaluation, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(materialize_isolation_enclave(&fixture.manifest, &fixture.boundary).is_err());
}

struct LaunchFixture {
    _temp: tempfile::TempDir,
    source: PathBuf,
    evaluation: PathBuf,
    codex_executable: PathBuf,
    claude_executable: PathBuf,
    engram_executable: PathBuf,
    skill: PathBuf,
    output_schema: PathBuf,
    settings: PathBuf,
    mcp: PathBuf,
    seatbelt: PathBuf,
    provider_proxy_port: u16,
    engram_daemon_port: u16,
}

fn launch_fixture() -> LaunchFixture {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let evaluation = temp.path().join("evaluation-enclave");
    mkdir_private(&source);
    mkdir_private(&source.join("run"));
    mkdir_private(&source.join("teaching"));
    mkdir_private(&evaluation);
    for relative in ["home", "tmp", "checkout", "config", "bin", "traces"] {
        mkdir_private(&evaluation.join(relative));
    }
    let source = source.canonicalize().unwrap();
    let evaluation = evaluation.canonicalize().unwrap();
    let codex_executable = evaluation.join("bin/codex-mock");
    let claude_executable = evaluation.join("bin/claude-mock");
    let engram_executable = evaluation.join("bin/engram");
    write_mode(
        &codex_executable,
        b"#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then printf '%s\\n' 'codex-cli 0.153.3'; else printf '%s\\n' 'Logged in using ChatGPT'; fi\n",
        0o700,
    );
    write_mode(
        &claude_executable,
        b"#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then printf '%s\\n' '2.1.260 (Claude Code)'; else printf '%s\\n' '{\"loggedIn\":true,\"authMethod\":\"claude.ai\",\"apiProvider\":\"firstParty\"}'; fi\n",
        0o700,
    );
    fs::copy("/bin/sleep", &engram_executable).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&engram_executable, fs::Permissions::from_mode(0o700)).unwrap();
    }
    let skill = evaluation.join("config/engram-skill.md");
    let output_schema = evaluation.join("config/output-schema.json");
    let settings = evaluation.join("config/settings.json");
    let mcp = evaluation.join("config/mcp.json");
    let seatbelt = evaluation.join("config/claude.sb");
    let provider_proxy_reservation = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let engram_daemon_reservation = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let provider_proxy_port = provider_proxy_reservation.local_addr().unwrap().port();
    let engram_daemon_port = engram_daemon_reservation.local_addr().unwrap().port();
    drop(provider_proxy_reservation);
    drop(engram_daemon_reservation);
    write_mode(&skill, b"Use exact Engram retrieval only.\n", 0o600);
    write_mode(&output_schema, b"{\"type\":\"object\"}\n", 0o600);
    LaunchFixture {
        _temp: temp,
        source,
        evaluation,
        codex_executable,
        claude_executable,
        engram_executable,
        skill,
        output_schema,
        settings,
        mcp,
        seatbelt,
        provider_proxy_port,
        engram_daemon_port,
    }
}

fn base_environment(
    fixture: &LaunchFixture,
    host: IsolationHost,
    native: bool,
    uses_engram: bool,
) -> BTreeMap<String, String> {
    let mut environment = BTreeMap::from([
        (
            "HOME".to_string(),
            fixture.evaluation.join("home").display().to_string(),
        ),
        (
            "TMPDIR".to_string(),
            fixture.evaluation.join("tmp").display().to_string(),
        ),
        ("PATH".to_string(), "/usr/bin:/bin".to_string()),
        ("LANG".to_string(), "C.UTF-8".to_string()),
        ("LC_ALL".to_string(), "C.UTF-8".to_string()),
        ("NO_COLOR".to_string(), "1".to_string()),
        ("DO_NOT_TRACK".to_string(), "1".to_string()),
        ("OTEL_SDK_DISABLED".to_string(), "true".to_string()),
    ]);
    match host {
        IsolationHost::Codex => {
            environment.insert(
                "CODEX_HOME".to_string(),
                fixture.evaluation.join("codex-home").display().to_string(),
            );
        }
        IsolationHost::ClaudeCode => {
            environment.insert(
                "CLAUDE_CONFIG_DIR".to_string(),
                fixture
                    .evaluation
                    .join("claude-config")
                    .display()
                    .to_string(),
            );
            environment.insert(
                "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".to_string(),
                "1".to_string(),
            );
            environment.insert(
                "CLAUDE_CODE_DISABLE_AUTO_MEMORY".to_string(),
                if native { "0" } else { "1" }.to_string(),
            );
            environment.insert(
                "CLAUDE_CODE_PROXY_RESOLVES_HOSTS".to_string(),
                "1".to_string(),
            );
            environment.insert("DISABLE_TELEMETRY".to_string(), "1".to_string());
            environment.insert("DISABLE_UPDATES".to_string(), "1".to_string());
            environment.insert(
                "ENABLE_CLAUDEAI_MCP_SERVERS".to_string(),
                "false".to_string(),
            );
            environment.insert(
                "HTTPS_PROXY".to_string(),
                format!(
                    "http://engram-proxy:REDACTED_PER_INVOCATION_SECRET@127.0.0.1:{}",
                    claude_transport(fixture, uses_engram).provider_proxy_port
                ),
            );
        }
    }
    if uses_engram {
        environment.insert(
            "ENGRAM_HOME".to_string(),
            fixture.evaluation.join("engram-home").display().to_string(),
        );
    }
    environment
}

fn boundary(
    fixture: &LaunchFixture,
    host: IsolationHost,
    native: bool,
    uses_engram: bool,
) -> IsolationBoundaryContract {
    let credential_path = match host {
        IsolationHost::Codex => {
            mkdir_private(&fixture.evaluation.join("codex-home"));
            fixture.evaluation.join("codex-home/auth.json")
        }
        IsolationHost::ClaudeCode => {
            mkdir_private(&fixture.evaluation.join("claude-config"));
            fixture.evaluation.join("claude-config/.credentials.json")
        }
    };
    write_mode(&credential_path, b"synthetic-fixture-only\n", 0o600);
    if native {
        mkdir_private(&match host {
            IsolationHost::Codex => fixture.evaluation.join("codex-home/memories"),
            IsolationHost::ClaudeCode => fixture.evaluation.join("claude-config/auto-memory"),
        });
    }
    if uses_engram {
        mkdir_private(&fixture.evaluation.join("engram-home"));
        mkdir_private(&fixture.evaluation.join("engram-home/projects"));
        mkdir_private(
            &fixture
                .evaluation
                .join("engram-home/projects/isolated-stale-eval"),
        );
    }
    let mut forbidden_paths = vec![
        IsolationBoundaryPath {
            class: IsolationBoundaryPathClass::RunArtifacts,
            path: fixture.source.join("run").display().to_string(),
        },
        IsolationBoundaryPath {
            class: IsolationBoundaryPathClass::TeachingArtifacts,
            path: fixture.source.join("teaching").display().to_string(),
        },
        IsolationBoundaryPath {
            class: IsolationBoundaryPathClass::CredentialMaterial,
            path: credential_path.display().to_string(),
        },
    ];
    if native {
        forbidden_paths.push(IsolationBoundaryPath {
            class: IsolationBoundaryPathClass::NativeState,
            path: match host {
                IsolationHost::Codex => fixture.evaluation.join("codex-home/memories"),
                IsolationHost::ClaudeCode => fixture.evaluation.join("claude-config/auto-memory"),
            }
            .display()
            .to_string(),
        });
    }
    if uses_engram {
        forbidden_paths.push(IsolationBoundaryPath {
            class: IsolationBoundaryPathClass::EngramState,
            path: fixture
                .evaluation
                .join("engram-home/projects/isolated-stale-eval")
                .display()
                .to_string(),
        });
    }
    IsolationBoundaryContract {
        schema_version: 1,
        host,
        native_memory_enabled: native,
        uses_engram,
        source_root: fixture.source.display().to_string(),
        evaluation_root: fixture.evaluation.display().to_string(),
        forbidden_paths,
        forbidden_markers: vec!["NATIVE-TEACHING-MARKER-01".to_string()],
    }
}

fn engram_surface(fixture: &LaunchFixture) -> IsolationEngramSurface {
    IsolationEngramSurface {
        executable: fixture.engram_executable.display().to_string(),
        executable_sha256: digest(&fs::read(&fixture.engram_executable).unwrap()),
        executable_version: "0.2.3-test".to_string(),
        mcp_tools_sha256: digest(b"fixture MCP tools"),
        home: fixture.evaluation.join("engram-home").display().to_string(),
        project: "isolated-stale-eval".to_string(),
        skill_path: fixture.skill.display().to_string(),
        skill_sha256: digest(&fs::read(&fixture.skill).unwrap()),
    }
}

fn permission_profile(fixture: &LaunchFixture) -> CodexPermissionProfile {
    CodexPermissionProfile {
        config_layer_name: "isolation".to_string(),
        permission_profile_name: "engram-eval-read".to_string(),
        root_access: "deny".to_string(),
        minimal_access: "read".to_string(),
        tmpdir_access: "deny".to_string(),
        slash_tmp_access: "deny".to_string(),
        evaluation_checkout: fixture.evaluation.join("checkout").display().to_string(),
        evaluation_checkout_access: "read".to_string(),
        additional_entries: Vec::new(),
        network_enabled: false,
    }
}

fn claude_transport(fixture: &LaunchFixture, uses_engram: bool) -> ClaudeNarrowTransportContract {
    ClaudeNarrowTransportContract {
        provider_proxy_host: "127.0.0.1".to_string(),
        provider_proxy_port: fixture.provider_proxy_port,
        provider_connect_authority: "api.anthropic.com:443".to_string(),
        engram_daemon_port: uses_engram.then_some(fixture.engram_daemon_port),
        max_connect_header_bytes: 16 * 1024,
        max_connect_header_count: 64,
        max_connect_line_bytes: 2048,
        max_connect_requests: 16,
        max_resolved_addresses: 16,
        connect_timeout_ms: 5_000,
        total_timeout_ms: 60_000,
        max_tunnel_bytes_each_direction: 64 * 1024 * 1024,
        idle_timeout_ms: 30_000,
    }
}

fn claude_engram_daemon_attestation(
    fixture: &LaunchFixture,
    surface: &IsolationEngramSurface,
    transport: &ClaudeNarrowTransportContract,
) -> ClaudeEngramDaemonAttestation {
    let port = transport.engram_daemon_port.unwrap();
    let mut process = Command::new(&surface.executable).arg("10").spawn().unwrap();
    let process_id = process.id();
    let daemon_directory = fixture
        .evaluation
        .join("engram-home/projects/isolated-stale-eval");
    mkdir_private(&daemon_directory);
    write_mode(
        &daemon_directory.join("daemon.port"),
        format!("{port}\n").as_bytes(),
        0o600,
    );
    write_mode(
        &daemon_directory.join("daemon.pid"),
        format!("{process_id}\n").as_bytes(),
        0o600,
    );
    let metadata = serde_json::json!({
        "schema_version": 1,
        "executable_path": surface.executable,
        "executable_version": "0.2.3-test",
        "executable_sha256": surface.executable_sha256,
        "pid": process_id,
        "port": port,
    });
    write_mode(
        &daemon_directory.join("daemon.meta.json"),
        serde_json::to_vec(&metadata).unwrap().as_slice(),
        0o600,
    );
    let listener = TcpListener::bind(("127.0.0.1", port)).unwrap();
    let body = serde_json::to_vec(&serde_json::json!({
        "status": "ok",
        "service": "engram",
        "version": surface.executable_version,
        "build_sha": null,
        "pid": process_id,
        "health_schema_version": 3,
        "mcp_contract_version": 5,
        "mcp_tools_sha256": surface.mcp_tools_sha256,
        "mcp_protocol_version": "2024-11-05",
        "auth_required": true,
        "storage_status": "ready",
        "storage_ready": true,
        "storage_available_bytes": 1_000_000,
        "storage_required_bytes": 1_000,
    }))
    .unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(1)))
            .unwrap();
        let mut request = [0_u8; 256];
        let length = stream.read(&mut request).unwrap();
        assert_eq!(
            &request[..length],
            format!("GET /health HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n")
                .as_bytes()
        );
        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        stream.write_all(header.as_bytes()).unwrap();
        stream.write_all(&body).unwrap();
    });
    let attestation = observe_claude_engram_daemon_attestation(surface, transport).unwrap();
    process.kill().unwrap();
    process.wait().unwrap();
    server.join().unwrap();
    attestation
}

fn codex_launch(
    fixture: &LaunchFixture,
    native: bool,
    uses_engram: bool,
) -> (IsolationLaunchContract, IsolationBoundaryContract) {
    let boundary = boundary(fixture, IsolationHost::Codex, native, uses_engram);
    let profile = permission_profile(fixture);
    let arm = CodexEvaluationArm {
        native_memory_enabled: native,
        min_rollout_idle_hours: 0,
        engram: uses_engram.then(|| engram_surface(fixture)),
    };
    let profile_bytes = generate_codex_evaluation_config_toml(&profile, &arm).unwrap();
    write_mode(
        &fixture.evaluation.join("codex-home/isolation.config.toml"),
        profile_bytes.as_bytes(),
        0o600,
    );
    let launch = build_codex_isolation_launch(
        &CodexIsolationLaunchSpec {
            executable: fixture.codex_executable.display().to_string(),
            executable_sha256: digest(&fs::read(&fixture.codex_executable).unwrap()),
            cli_version: "codex-cli 0.153.3".to_string(),
            cwd: fixture.evaluation.join("checkout").display().to_string(),
            environment: base_environment(fixture, IsolationHost::Codex, native, uses_engram),
            prompt: "Evaluate the bounded stale case.".to_string(),
            output_schema: fixture.output_schema.display().to_string(),
            permission_profile: profile.clone(),
            arm,
            permission_probe: codex_probe(&profile, profile_bytes.as_bytes()),
        },
        &boundary,
    )
    .unwrap();
    (launch, boundary)
}

fn claude_launch_spec(
    fixture: &LaunchFixture,
    native: bool,
    uses_engram: bool,
) -> (ClaudeIsolationLaunchSpec, IsolationBoundaryContract) {
    let boundary = boundary(fixture, IsolationHost::ClaudeCode, native, uses_engram);
    let transport = claude_transport(fixture, uses_engram);
    let seatbelt_spec = claude_seatbelt_spec_for_boundary(&boundary, &transport).unwrap();
    let profile = generate_claude_seatbelt_profile(&seatbelt_spec).unwrap();
    write_mode(&fixture.seatbelt, profile.as_bytes(), 0o600);
    let auto_memory = native.then(|| {
        fixture
            .evaluation
            .join("claude-config/auto-memory")
            .display()
            .to_string()
    });
    let settings = if let Some(directory) = &auto_memory {
        serde_json::json!({"autoMemoryDirectory": directory, "autoMemoryEnabled": true})
    } else {
        serde_json::json!({"autoMemoryEnabled": false})
    };
    write_mode(
        &fixture.settings,
        format!("{}\n", serde_json::to_string(&settings).unwrap()).as_bytes(),
        0o600,
    );
    let engram = uses_engram.then(|| engram_surface(fixture));
    let engram_daemon = engram
        .as_ref()
        .map(|surface| claude_engram_daemon_attestation(fixture, surface, &transport));
    let mcp = if let Some(engram) = &engram {
        serde_json::json!({"mcpServers": {"engram": {"command": engram.executable, "args": ["serve", "--project", engram.project, "--profile", "agent"], "env": {"ENGRAM_HOME": engram.home}}}})
    } else {
        serde_json::json!({"mcpServers": {}})
    };
    write_mode(
        &fixture.mcp,
        format!("{}\n", serde_json::to_string(&mcp).unwrap()).as_bytes(),
        0o600,
    );
    (
        ClaudeIsolationLaunchSpec {
            executable: fixture.claude_executable.display().to_string(),
            executable_sha256: digest(&fs::read(&fixture.claude_executable).unwrap()),
            cli_version: "2.1.260 (Claude Code)".to_string(),
            cwd: fixture.evaluation.join("checkout").display().to_string(),
            environment: base_environment(fixture, IsolationHost::ClaudeCode, native, uses_engram),
            prompt: "Evaluate the bounded stale case.".to_string(),
            output_schema: fixture.output_schema.display().to_string(),
            settings: fixture.settings.display().to_string(),
            mcp_config: fixture.mcp.display().to_string(),
            seatbelt_profile: fixture.seatbelt.display().to_string(),
            seatbelt_spec: seatbelt_spec.clone(),
            seatbelt_probe: claude_probe(&seatbelt_spec, profile.as_bytes()),
            auto_memory_directory: auto_memory,
            engram,
            engram_daemon,
            limits: ClaudeEvaluationLimits {
                model: "claude-test-model-1".to_string(),
                max_turns: 12,
                max_budget_micro_usd: 50_000,
            },
        },
        boundary,
    )
}

#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn claude_launch(
    fixture: &LaunchFixture,
    native: bool,
    uses_engram: bool,
) -> (IsolationLaunchContract, IsolationBoundaryContract) {
    let (spec, boundary) = claude_launch_spec(fixture, native, uses_engram);
    let launch = build_claude_isolation_launch(&spec, &boundary).unwrap();
    (launch, boundary)
}

#[test]
fn typed_launches_bind_exact_host_argv_native_and_engram_surfaces() {
    for native in [false, true] {
        for uses_engram in [false, true] {
            let fixture = launch_fixture();
            let (codex, codex_boundary) = codex_launch(&fixture, native, uses_engram);
            validate_isolation_launch(&codex, &codex_boundary).unwrap();
            assert!(!codex.argv.iter().any(|arg| arg == "--ask-for-approval"));
            assert!(!codex.argv.iter().any(|arg| arg == "--sandbox"));
            assert_eq!(
                codex
                    .expected_tools
                    .iter()
                    .any(|tool| tool.starts_with("mcp__engram__")),
                uses_engram
            );
            #[cfg(target_os = "macos")]
            {
                let fixture = launch_fixture();
                let (claude, claude_boundary) = claude_launch(&fixture, native, uses_engram);
                validate_isolation_launch(&claude, &claude_boundary).unwrap();
                assert_eq!(
                    claude
                        .argv
                        .iter()
                        .any(|arg| arg == "--append-system-prompt-file"),
                    uses_engram
                );
                assert!(!claude.expected_tools.iter().any(|tool| tool == "Bash"));
            }
        }
    }
}

#[test]
fn exact_argv_rejects_equals_forms_duplicates_extras_and_invalid_approval_placement() {
    let fixture = launch_fixture();
    let (launch, boundary) = codex_launch(&fixture, false, false);
    for mutation in 0..4 {
        let mut bad = launch.clone();
        match mutation {
            0 => bad.argv[2] = "--profile=isolation".to_string(),
            1 => {
                bad.argv
                    .splice(4..4, ["--profile".to_string(), "isolation".to_string()]);
            }
            2 => bad.argv.insert(bad.argv.len() - 1, "--search".to_string()),
            _ => {
                bad.argv.splice(
                    bad.argv.len() - 1..bad.argv.len() - 1,
                    ["--ask-for-approval".to_string(), "never".to_string()],
                );
            }
        }
        assert!(validate_isolation_launch(&bad, &boundary).is_err());
    }
    let mut bad = launch;
    bad.prompt = "--help".to_string();
    *bad.argv.last_mut().unwrap() = "--help".to_string();
    assert!(validate_isolation_launch(&bad, &boundary).is_err());
    #[cfg(target_os = "macos")]
    {
        let fixture = launch_fixture();
        let (launch, boundary) = claude_launch(&fixture, false, false);
        for mutation in 0..3 {
            let mut bad = launch.clone();
            match mutation {
                0 => {
                    let index = bad.argv.iter().position(|arg| arg == "--settings").unwrap();
                    bad.argv[index] = format!("--settings={}", fixture.settings.display());
                }
                1 => bad.argv.insert(8, "--restricted".to_string()),
                _ => bad
                    .argv
                    .insert(bad.argv.len() - 1, "--plugin-dir".to_string()),
            }
            assert!(validate_isolation_launch(&bad, &boundary).is_err());
        }
    }
}

#[cfg(target_os = "macos")]
#[test]
fn claude_exact_contract_requires_verbose_and_separates_builtin_from_mcp_tools() {
    let fixture = launch_fixture();
    let (engram_launch, engram_boundary) = claude_launch(&fixture, false, true);
    let expected_mcp = engram_launch
        .expected_tools
        .iter()
        .filter(|tool| tool.starts_with("mcp__engram__"))
        .cloned()
        .collect::<Vec<_>>()
        .join(",");
    assert!(engram_launch
        .argv
        .windows(2)
        .any(|pair| pair == ["--tools", "Read"]));
    assert!(engram_launch
        .argv
        .windows(2)
        .any(|pair| { pair[0] == "--allowed-tools" && pair[1] == expected_mcp }));
    assert!(engram_launch.argv.iter().any(|arg| arg == "--verbose"));
    assert!(engram_launch
        .argv
        .windows(2)
        .any(|pair| pair == ["--permission-mode", "dontAsk"]));
    assert!(engram_launch.argv.windows(2).any(|pair| {
        pair == [
            "--disallowed-tools",
            "Bash,Write,Edit,WebFetch,WebSearch,NotebookEdit,Task",
        ]
    }));
    assert!(engram_launch
        .argv
        .windows(2)
        .any(|pair| pair == ["--model", "claude-test-model-1"]));
    assert!(engram_launch
        .argv
        .windows(2)
        .any(|pair| pair == ["--max-turns", "12"]));
    assert!(engram_launch
        .argv
        .windows(2)
        .any(|pair| pair == ["--max-budget-usd", "0.050000"]));
    assert_eq!(
        engram_launch
            .environment
            .get("CLAUDE_CODE_DISABLE_AUTO_MEMORY")
            .map(String::as_str),
        Some("1")
    );
    assert_eq!(
        engram_launch
            .environment
            .get("HTTPS_PROXY")
            .map(String::as_str),
        Some(
            format!(
                "http://engram-proxy:REDACTED_PER_INVOCATION_SECRET@127.0.0.1:{}",
                fixture.provider_proxy_port
            )
            .as_str(),
        )
    );
    assert!(engram_launch
        .environment
        .get("HTTPS_PROXY")
        .unwrap()
        .contains("REDACTED_PER_INVOCATION_SECRET"));
    for (key, value) in [
        ("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC", "1"),
        ("CLAUDE_CODE_PROXY_RESOLVES_HOSTS", "1"),
        ("DISABLE_TELEMETRY", "1"),
        ("DISABLE_UPDATES", "1"),
        ("ENABLE_CLAUDEAI_MCP_SERVERS", "false"),
    ] {
        assert_eq!(
            engram_launch.environment.get(key).map(String::as_str),
            Some(value)
        );
    }
    assert!(!engram_launch.environment.contains_key("HTTP_PROXY"));
    assert!(!engram_launch.environment.contains_key("ALL_PROXY"));
    assert!(!engram_launch.environment.contains_key("NO_PROXY"));
    assert!(!engram_launch.authorizes_provider_execution);

    let mut bad = engram_launch.clone();
    bad.argv.retain(|arg| arg != "--verbose");
    assert!(validate_isolation_launch(&bad, &engram_boundary).is_err());
    let mut bad = engram_launch.clone();
    bad.environment.insert(
        "HTTPS_PROXY".to_string(),
        format!("http://127.0.0.1:{}", fixture.engram_daemon_port),
    );
    assert!(validate_isolation_launch(&bad, &engram_boundary).is_err());
    let mut bad = engram_launch.clone();
    bad.environment.insert(
        "HTTP_PROXY".to_string(),
        format!("http://127.0.0.1:{}", fixture.provider_proxy_port),
    );
    assert!(validate_isolation_launch(&bad, &engram_boundary).is_err());
    let mut bad = engram_launch.clone();
    bad.prompt = "--help".to_string();
    *bad.argv.last_mut().unwrap() = "--help".to_string();
    assert!(validate_isolation_launch(&bad, &engram_boundary).is_err());
    let mut bad = engram_launch.clone();
    let tools = bad.argv.iter().position(|arg| arg == "Read").unwrap();
    bad.argv[tools] = format!("Read,{expected_mcp}");
    assert!(validate_isolation_launch(&bad, &engram_boundary).is_err());
    let mut bad = engram_launch.clone();
    let allowed = bad
        .argv
        .iter()
        .position(|arg| arg == "--allowed-tools")
        .unwrap();
    bad.argv.drain(allowed..=allowed + 1);
    assert!(validate_isolation_launch(&bad, &engram_boundary).is_err());

    let fixture = launch_fixture();
    let (native_only, native_boundary) = claude_launch(&fixture, true, false);
    assert!(!native_only.argv.iter().any(|arg| arg == "--allowed-tools"));
    let mut bad = native_only;
    bad.argv.splice(
        bad.argv.len() - 3..bad.argv.len() - 3,
        ["--allowed-tools".to_string(), "Read".to_string()],
    );
    assert!(validate_isolation_launch(&bad, &native_boundary).is_err());
}

#[cfg(target_os = "macos")]
#[test]
fn claude_trace_attestation_binds_init_terminal_turn_and_cost_evidence() {
    let fixture = launch_fixture();
    let (launch, boundary) = claude_launch(&fixture, true, true);
    assert_eq!(
        launch
            .environment
            .get("CLAUDE_CODE_DISABLE_AUTO_MEMORY")
            .map(String::as_str),
        Some("0")
    );
    let attestation = ClaudeIsolationTraceAttestation {
        trace_sha256: "a".repeat(64),
        init_present: true,
        terminal_present: true,
        init_claude_code_version: "2.1.260".to_string(),
        init_cwd: launch.cwd.clone(),
        init_output_style: "default".to_string(),
        init_model: "claude-test-model-1".to_string(),
        init_mcp_servers: vec!["engram".to_string()],
        init_mcp_server_statuses: BTreeMap::from([("engram".to_string(), "connected".to_string())]),
        init_tools: launch.expected_tools.clone(),
        init_permission_mode: "dontAsk".to_string(),
        init_plugins: Vec::new(),
        init_subagents: vec![
            "claude".to_string(),
            "Explore".to_string(),
            "general-purpose".to_string(),
            "Plan".to_string(),
            "statusline-setup".to_string(),
        ],
        init_skills: Vec::new(),
        init_slash_commands: Vec::new(),
        init_terminal_slash_commands: Vec::new(),
        init_capabilities: vec![
            "interrupt_receipt_v1".to_string(),
            "interrupt_cancel_queued_v1".to_string(),
            "msg_lifecycle_v1".to_string(),
        ],
        init_api_key_source: "apiKeyHelper".to_string(),
        init_analytics_disabled: true,
        init_product_feedback_disabled: false,
        managed_api_key_helper_attempted: true,
        managed_api_key_helper_succeeded: true,
        local_subscription_pure: false,
        confounded_exploratory_only: true,
        configured_max_turns: 12,
        configured_max_budget_micro_usd: 50_000,
        observed_turns: 2,
        reported_cost_micro_usd: 40_000,
        terminal_subtype: "success".to_string(),
        terminal_budget_exhausted: false,
    };
    validate_claude_trace_attestation(&launch, &boundary, &attestation).unwrap();
    for mutation in 0..12 {
        let mut bad = attestation.clone();
        match mutation {
            0 => bad.init_model = "other-model".to_string(),
            1 => bad.init_mcp_servers.clear(),
            2 => {
                bad.init_tools.pop();
            }
            3 => bad.init_permission_mode = "manual".to_string(),
            4 => bad.init_plugins.push("ambient-plugin".to_string()),
            5 => bad.init_subagents.clear(),
            6 => bad.init_capabilities.reverse(),
            7 => bad.init_skills.push("ambient-skill".to_string()),
            8 => bad.init_api_key_source = "ANTHROPIC_API_KEY".to_string(),
            9 => bad.local_subscription_pure = true,
            10 => bad.terminal_subtype = "error_during_execution".to_string(),
            _ => bad.observed_turns = 13,
        }
        assert!(validate_claude_trace_attestation(&launch, &boundary, &bad).is_err());
    }
    let mut overshoot = attestation.clone();
    overshoot.reported_cost_micro_usd = 50_001;
    assert!(validate_claude_trace_attestation(&launch, &boundary, &overshoot).is_err());
    overshoot.terminal_subtype = "error_max_budget_usd".to_string();
    overshoot.terminal_budget_exhausted = true;
    validate_claude_trace_attestation(&launch, &boundary, &overshoot).unwrap();
}

#[test]
fn rejects_arm_config_tool_skill_profile_hash_and_boundary_drift() {
    let fixture = launch_fixture();
    let (launch, boundary) = codex_launch(&fixture, true, true);
    let mut bad = launch.clone();
    bad.authorizes_provider_execution = true;
    assert!(validate_isolation_launch(&bad, &boundary).is_err());
    let mut bad = launch.clone();
    bad.expected_tools.pop();
    assert!(validate_isolation_launch(&bad, &boundary).is_err());
    let mut bad = launch.clone();
    bad.native_memory_enabled = false;
    assert!(validate_isolation_launch(&bad, &boundary).is_err());
    let mut bad = launch.clone();
    bad.boundary_sha256 = "0".repeat(64);
    assert!(validate_isolation_launch(&bad, &boundary).is_err());
    let mut bad = launch.clone();
    bad.config_files[0].sha256 = "0".repeat(64);
    assert!(validate_isolation_launch(&bad, &boundary).is_err());
    let mut bad = launch.clone();
    if let IsolationLaunchSemantics::Codex {
        permission_probe, ..
    } = &mut bad.semantics
    {
        permission_probe.profile_sha256 = "0".repeat(64);
    }
    assert!(validate_isolation_launch(&bad, &boundary).is_err());
    write_mode(
        &fixture.evaluation.join("codex-home/isolation.config.toml"),
        b"approval_policy = \"never\"\n",
        0o600,
    );
    assert!(validate_isolation_launch(&launch, &boundary).is_err());
    #[cfg(target_os = "macos")]
    {
        let fixture = launch_fixture();
        let (launch, boundary) = claude_launch(&fixture, true, true);
        let mut bad = launch.clone();
        if let IsolationLaunchSemantics::ClaudeCode { seatbelt_probe, .. } = &mut bad.semantics {
            seatbelt_probe.profile_sha256 = "0".repeat(64);
        }
        assert!(validate_isolation_launch(&bad, &boundary).is_err());
        let mut bad = launch.clone();
        if let IsolationLaunchSemantics::ClaudeCode {
            managed_settings_outside_exact_environment_claim,
            ..
        } = &mut bad.semantics
        {
            *managed_settings_outside_exact_environment_claim = false;
        }
        assert!(validate_isolation_launch(&bad, &boundary).is_err());
        let mut bad = launch.clone();
        if let IsolationLaunchSemantics::ClaudeCode { engram_daemon, .. } = &mut bad.semantics {
            engram_daemon.as_mut().unwrap().healthy = false;
        }
        assert!(validate_isolation_launch(&bad, &boundary).is_err());
        let mut bad = launch.clone();
        if let IsolationLaunchSemantics::ClaudeCode { seatbelt_spec, .. } = &mut bad.semantics {
            seatbelt_spec.transport.max_connect_requests = 0;
        }
        assert!(validate_isolation_launch(&bad, &boundary).is_err());
        write_mode(&fixture.settings, b"{\"autoMemoryEnabled\":false}\n", 0o600);
        assert!(validate_isolation_launch(&launch, &boundary).is_err());
    }
}

#[test]
fn rejects_hollow_credential_engram_and_disabled_native_state_boundaries() {
    let fixture = launch_fixture();
    let (launch, boundary) = codex_launch(&fixture, true, true);
    let mut hollow = boundary.clone();
    let dummy_credential = fixture.evaluation.join("dummy-auth.json");
    write_mode(&dummy_credential, b"synthetic-fixture-only\n", 0o600);
    hollow
        .forbidden_paths
        .iter_mut()
        .find(|path| path.class == IsolationBoundaryPathClass::CredentialMaterial)
        .unwrap()
        .path = dummy_credential.display().to_string();
    let mut bad = launch.clone();
    bad.boundary_sha256 = serialized_digest(&hollow);
    assert!(validate_isolation_launch(&bad, &hollow).is_err());

    let mut hollow = boundary;
    let dummy_engram = fixture.evaluation.join("engram-home/dummy-state");
    mkdir_private(&dummy_engram);
    hollow
        .forbidden_paths
        .iter_mut()
        .find(|path| path.class == IsolationBoundaryPathClass::EngramState)
        .unwrap()
        .path = dummy_engram.display().to_string();
    let mut bad = launch;
    bad.boundary_sha256 = serialized_digest(&hollow);
    assert!(validate_isolation_launch(&bad, &hollow).is_err());

    let fixture = launch_fixture();
    let (launch, boundary) = codex_launch(&fixture, false, false);
    mkdir_private(&fixture.evaluation.join("codex-home/memories"));
    assert!(validate_isolation_launch(&launch, &boundary).is_err());

    #[cfg(target_os = "macos")]
    {
        let fixture = launch_fixture();
        let (launch, boundary) = claude_launch(&fixture, false, false);
        mkdir_private(&fixture.evaluation.join("codex-home"));
        assert!(validate_isolation_launch(&launch, &boundary).is_err());
    }
}

#[test]
fn rejects_every_claude_credential_alias_before_content_access() {
    #[cfg(target_os = "macos")]
    {
        type LaunchPathMutator = fn(&mut ClaudeIsolationLaunchSpec, String);

        let simple_mutators: [(&str, LaunchPathMutator); 5] = [
            ("provider executable", |spec, path| spec.executable = path),
            ("output schema", |spec, path| spec.output_schema = path),
            ("settings", |spec, path| spec.settings = path),
            ("MCP config", |spec, path| spec.mcp_config = path),
            ("Seatbelt profile", |spec, path| {
                spec.seatbelt_profile = path
            }),
        ];
        for (label, mutate) in simple_mutators {
            let fixture = launch_fixture();
            let (mut spec, boundary) = claude_launch_spec(&fixture, false, false);
            let credential = fixture
                .evaluation
                .join("claude-config/.credentials.json")
                .display()
                .to_string();
            mutate(&mut spec, credential);
            let error = build_claude_isolation_launch(&spec, &boundary).unwrap_err();
            assert!(
                error.to_string().contains("before content access"),
                "{label} reached a later validator: {error}"
            );
        }

        let engram_mutators: [(&str, LaunchPathMutator); 3] = [
            ("Engram executable", |spec, path| {
                spec.engram.as_mut().unwrap().executable = path;
            }),
            ("Engram skill", |spec, path| {
                spec.engram.as_mut().unwrap().skill_path = path;
            }),
            ("daemon receipt", |spec, path| {
                spec.engram_daemon.as_mut().unwrap().daemon_port_file.path = path;
            }),
        ];
        for (label, mutate) in engram_mutators {
            let fixture = launch_fixture();
            let (mut spec, boundary) = claude_launch_spec(&fixture, false, true);
            let credential = fixture
                .evaluation
                .join("claude-config/.credentials.json")
                .display()
                .to_string();
            mutate(&mut spec, credential);
            let error = build_claude_isolation_launch(&spec, &boundary).unwrap_err();
            assert!(
                error.to_string().contains("before content access"),
                "{label} reached a later validator: {error}"
            );
        }

        let fixture = launch_fixture();
        let (mut spec, boundary) = claude_launch_spec(&fixture, false, false);
        let credential = fixture.evaluation.join("claude-config/.credentials.json");
        spec.output_schema = credential.parent().unwrap().display().to_string();
        let error = build_claude_isolation_launch(&spec, &boundary).unwrap_err();
        assert!(error.to_string().contains("before content access"));

        let fixture = launch_fixture();
        let (mut spec, boundary) = claude_launch_spec(&fixture, false, false);
        let credential = fixture.evaluation.join("claude-config/.credentials.json");
        spec.output_schema = credential.join("descendant").display().to_string();
        let error = build_claude_isolation_launch(&spec, &boundary).unwrap_err();
        assert!(error.to_string().contains("before content access"));

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;

            let fixture = launch_fixture();
            let (mut spec, boundary) = claude_launch_spec(&fixture, false, false);
            let credential = fixture.evaluation.join("claude-config/.credentials.json");
            let alias = fixture.evaluation.join("config/credential-alias");
            symlink(&credential, &alias).unwrap();
            spec.output_schema = alias.display().to_string();
            let error = build_claude_isolation_launch(&spec, &boundary).unwrap_err();
            assert!(error.to_string().contains("before content access"));
        }

        let fixture = launch_fixture();
        let (mut launch, boundary) = claude_launch(&fixture, false, false);
        let credential = fixture
            .evaluation
            .join("claude-config/.credentials.json")
            .display()
            .to_string();
        launch.config_files[0].path = credential;
        let error = validate_isolation_launch(&launch, &boundary).unwrap_err();
        assert!(error.to_string().contains("before content access"));
    }

    let fixture = launch_fixture();
    let (mut launch, boundary) = codex_launch(&fixture, false, false);
    launch.argv[0] = fixture
        .evaluation
        .join("codex-home/auth.json")
        .display()
        .to_string();
    let error = validate_isolation_launch(&launch, &boundary).unwrap_err();
    assert!(error.to_string().contains("before content access"));
}

#[test]
fn rejects_self_hashed_provider_substitutes_with_wrong_observed_version() {
    let fixture = launch_fixture();
    let (mut launch, boundary) = codex_launch(&fixture, false, false);
    write_mode(
        &fixture.codex_executable,
        b"#!/bin/sh\nprintf '%s\\n' 'attacker-controlled'\n",
        0o700,
    );
    launch.provider_executable_sha256 = digest(&fs::read(&fixture.codex_executable).unwrap());
    assert!(validate_isolation_launch(&launch, &boundary).is_err());
}

#[test]
fn rejects_incomplete_forged_or_internally_inconsistent_probe_receipts() {
    let fixture = launch_fixture();
    let (launch, boundary) = codex_launch(&fixture, false, false);
    let mutate_codex = |mutation: fn(&mut CodexSandboxProbeAudit)| {
        let mut bad = launch.clone();
        let IsolationLaunchSemantics::Codex {
            permission_probe, ..
        } = &mut bad.semantics
        else {
            unreachable!();
        };
        mutation(permission_probe);
        assert!(validate_isolation_launch(&bad, &boundary).is_err());
    };
    mutate_codex(|probe| {
        probe.commands.pop();
    });
    mutate_codex(|probe| {
        probe.commands[1].argv_sha256 = probe.commands[0].argv_sha256.clone();
    });
    mutate_codex(|probe| {
        probe.commands[4].sandbox_denial_evidence = false;
    });
    mutate_codex(|probe| {
        probe.network_control.stdout_sha256 = "not-a-digest".to_string();
    });
    mutate_codex(|probe| {
        probe.profile.network_sandbox_denial_observed = false;
    });

    #[cfg(target_os = "macos")]
    {
        let fixture = launch_fixture();
        let (launch, boundary) = claude_launch(&fixture, true, true);
        let mutate_claude = |mutation: fn(&mut ClaudeSeatbeltProbeAudit)| {
            let mut bad = launch.clone();
            let IsolationLaunchSemantics::ClaudeCode { seatbelt_probe, .. } = &mut bad.semantics
            else {
                unreachable!();
            };
            mutation(seatbelt_probe);
            assert!(validate_isolation_launch(&bad, &boundary).is_err());
        };
        mutate_claude(|probe| {
            probe.commands.pop();
        });
        mutate_claude(|probe| {
            probe.commands[2].argv_sha256 = probe.commands[1].argv_sha256.clone();
        });
        mutate_claude(|probe| {
            probe.commands.last_mut().unwrap().exit_code = 0;
        });
        mutate_claude(|probe| {
            probe.profile.forbidden_writes.clear();
        });
    }
}

#[test]
fn rejects_ambient_environment_source_marker_and_public_private_paths() {
    let fixture = launch_fixture();
    let (launch, boundary) = codex_launch(&fixture, false, false);
    let mut bad = launch.clone();
    bad.environment
        .insert("USER".to_string(), "ambient".to_string());
    assert!(validate_isolation_launch(&bad, &boundary).is_err());
    let mut bad = launch.clone();
    bad.argv.push(fixture.source.display().to_string());
    assert!(validate_isolation_launch(&bad, &boundary).is_err());
    let mut bad = launch.clone();
    bad.prompt = "NATIVE-TEACHING-MARKER-01".to_string();
    assert!(validate_isolation_launch(&bad, &boundary).is_err());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(
            fixture.evaluation.join("codex-home"),
            fs::Permissions::from_mode(0o755),
        )
        .unwrap();
        assert!(validate_isolation_launch(&launch, &boundary).is_err());
    }
}

#[cfg(target_os = "macos")]
#[test]
fn claude_seatbelt_protects_state_but_allows_trusted_client_auth_access() {
    let fixture = launch_fixture();
    let (_, boundary) = claude_launch(&fixture, true, true);
    let spec =
        claude_seatbelt_spec_for_boundary(&boundary, &claude_transport(&fixture, true)).unwrap();
    assert!(spec.forbidden_read_paths.contains(&boundary.source_root));
    assert!(spec.forbidden_write_paths.contains(&boundary.source_root));
    for protected in &boundary.forbidden_paths {
        assert_eq!(
            spec.forbidden_write_paths.contains(&protected.path),
            protected.class != IsolationBoundaryPathClass::CredentialMaterial
        );
        if matches!(
            protected.class,
            IsolationBoundaryPathClass::RunArtifacts
                | IsolationBoundaryPathClass::TeachingArtifacts
        ) {
            assert!(spec.forbidden_read_paths.contains(&protected.path));
        } else if protected.class == IsolationBoundaryPathClass::CredentialMaterial {
            assert!(!spec.forbidden_read_paths.contains(&protected.path));
        }
    }
    let profile = generate_claude_seatbelt_profile(&spec).unwrap();
    for path in &spec.forbidden_write_paths {
        assert!(profile.contains(&format!("(deny file-write* (subpath \"{path}\"))")));
    }
    let credential = boundary
        .forbidden_paths
        .iter()
        .find(|path| path.class == IsolationBoundaryPathClass::CredentialMaterial)
        .unwrap();
    assert!(!spec.forbidden_read_paths.contains(&credential.path));
    assert!(!spec.forbidden_write_paths.contains(&credential.path));
    assert!(!profile.contains(&credential.path));
    let mut bad = spec.clone();
    bad.forbidden_write_paths.push(credential.path.clone());
    assert!(validate_claude_seatbelt_boundary(&bad, &boundary).is_err());
    let mut bad = spec.clone();
    bad.forbidden_write_paths.pop();
    assert!(validate_claude_seatbelt_boundary(&bad, &boundary).is_err());
}

#[test]
fn connect_only_proxy_parser_fails_closed() {
    let fixture = launch_fixture();
    let transport = claude_transport(&fixture, false);
    let valid = b"CONNECT api.anthropic.com:443 HTTP/1.1\r\nHost: api.anthropic.com:443\r\nUser-Agent: bounded-test\r\n\r\n";
    let accepted = inspect_claude_connect_request(&transport, valid).unwrap();
    assert!(accepted.accepted);
    assert!(!accepted.raw_request_retained);

    for rejected in [
        b"GET api.anthropic.com:443 HTTP/1.1\r\nHost: api.anthropic.com:443\r\n\r\n".as_slice(),
        b"CONNECT example.com:443 HTTP/1.1\r\nHost: example.com:443\r\n\r\n".as_slice(),
        b"CONNECT api.anthropic.com:443 HTTP/1.1\r\nHost: api.anthropic.com:443\r\nProxy-Authorization: secret\r\n\r\n".as_slice(),
        b"CONNECT api.anthropic.com:443 HTTP/1.1\r\nHost: api.anthropic.com:443\r\nContent-Length: 1\r\n\r\nx".as_slice(),
    ] {
        assert!(!inspect_claude_connect_request(&transport, rejected)
            .unwrap()
            .accepted);
    }
    for malformed in [
        b"CONNECT api.anthropic.com:443 HTTP/1.1\r\n Host: api.anthropic.com:443\r\n\r\n".as_slice(),
        b"CONNECT api.anthropic.com:443 HTTP/1.1\r\nBad Name: value\r\nHost: api.anthropic.com:443\r\n\r\n".as_slice(),
        b"CONNECT api.anthropic.com:443 HTTP/1.1\r\nHost: api.anthropic.com:443\x01\r\n\r\n".as_slice(),
    ] {
        assert!(inspect_claude_connect_request(&transport, malformed).is_err());
    }
    let mut too_many =
        String::from("CONNECT api.anthropic.com:443 HTTP/1.1\r\nHost: api.anthropic.com:443\r\n");
    for _ in 0..transport.max_connect_header_count {
        too_many.push_str("X-Test: value\r\n");
    }
    too_many.push_str("\r\n");
    assert!(inspect_claude_connect_request(&transport, too_many.as_bytes()).is_err());
}

#[cfg(target_os = "macos")]
#[test]
#[ignore = "explicit provider-free local managed-policy attestation gate"]
fn observes_every_local_claude_managed_policy_source_without_claiming_remote_policy_closure() {
    assert_eq!(
        known_claude_macos_managed_policy_paths(),
        vec![
            "/Library/Application Support/ClaudeCode/managed-settings.json".to_string(),
            "/Library/Application Support/ClaudeCode/managed-mcp.json".to_string(),
            "/Library/Managed Preferences/com.anthropic.claudecode.plist".to_string(),
            "/Library/Application Support/ClaudeCode/managed-settings.d".to_string(),
        ]
    );
    let attestation = observe_claude_macos_managed_policy().unwrap();
    validate_claude_macos_managed_policy_attestation(&attestation).unwrap();
    assert!(!attestation.all_known_sources_observed);
    assert!(attestation.server_managed_effective_policy_unobserved);
    assert!(!attestation.safe_for_local_provider_execution);
    assert!(attestation.confounded_exploratory_only);
    assert!(!attestation.raw_policy_contents_retained);
}

#[cfg(target_os = "macos")]
#[test]
fn production_transport_rejects_special_duplicate_addresses_and_future_receipts() {
    use std::net::SocketAddr;
    use std::time::{SystemTime, UNIX_EPOCH};

    let fixture = launch_fixture();
    let transport = claude_transport(&fixture, false);
    let public = [
        "93.184.216.34:443".parse::<SocketAddr>().unwrap(),
        "[2606:4700:4700::1111]:443".parse::<SocketAddr>().unwrap(),
    ];
    let normalized = validate_claude_provider_resolution_candidates(&transport, &public).unwrap();
    assert_eq!(normalized.len(), 2);

    for rejected in [
        vec!["127.0.0.1:443".parse().unwrap()],
        vec!["10.0.0.1:443".parse().unwrap()],
        vec!["192.0.2.1:443".parse().unwrap()],
        vec!["93.184.216.34:80".parse().unwrap()],
        vec![public[0], public[0]],
    ] {
        assert!(validate_claude_provider_resolution_candidates(&transport, &rejected).is_err());
    }
    let mut too_many = Vec::new();
    for suffix in 1..=17 {
        too_many.push(format!("8.8.4.{suffix}:443").parse().unwrap());
    }
    assert!(validate_claude_provider_resolution_candidates(&transport, &too_many).is_err());

    let first = generate_claude_proxy_invocation_secret("resolution-policy-test-1").unwrap();
    let second = generate_claude_proxy_invocation_secret("resolution-policy-test-1").unwrap();
    assert_ne!(first.salted_sha256(), second.salted_sha256());
    let runtime_proxy = first.runtime_https_proxy(&transport).unwrap();
    assert!(runtime_proxy.starts_with("http://engram-proxy:"));
    assert!(runtime_proxy.ends_with(&format!("@127.0.0.1:{}", transport.provider_proxy_port)));
    assert!(!format!("{first:?}").contains(runtime_proxy.as_str()));

    let now = u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    )
    .unwrap();
    let session = ClaudeProviderProxySessionBinding {
        invocation_id: "timestamp-binding-1".to_string(),
        launch_sha256: "a".repeat(64),
        seatbelt_profile_sha256: "b".repeat(64),
        proxy_auth_sha256: "c".repeat(64),
        provider_process_id: 42,
        proxy_process_id: 43,
        proxy_started_unix_ms: now.saturating_sub(500),
        provider_started_unix_ms: now.saturating_sub(400),
    };
    let mut receipt = ClaudeProviderProxyReceipt {
        valid: true,
        target: "api.anthropic.com:443".to_string(),
        accepted_connect_count: 1,
        rejected_request_count: 0,
        client_to_upstream_bytes: 10,
        upstream_to_client_bytes: 20,
        started_unix_ms: now.saturating_sub(300),
        finished_unix_ms: now.saturating_sub(200),
        raw_headers_or_body_retained: false,
        session: session.clone(),
        receipt_sha256: String::new(),
    };
    receipt.receipt_sha256 = serialized_digest(&(
        &receipt.target,
        receipt.accepted_connect_count,
        receipt.rejected_request_count,
        receipt.client_to_upstream_bytes,
        receipt.upstream_to_client_bytes,
        receipt.started_unix_ms,
        receipt.finished_unix_ms,
        receipt.raw_headers_or_body_retained,
        &receipt.session,
    ));
    validate_claude_provider_proxy_receipt(&transport, &session, &receipt).unwrap();
    receipt.started_unix_ms = now.saturating_add(10_000);
    receipt.finished_unix_ms = now.saturating_add(10_100);
    receipt.receipt_sha256 = serialized_digest(&(
        &receipt.target,
        receipt.accepted_connect_count,
        receipt.rejected_request_count,
        receipt.client_to_upstream_bytes,
        receipt.upstream_to_client_bytes,
        receipt.started_unix_ms,
        receipt.finished_unix_ms,
        receipt.raw_headers_or_body_retained,
        &receipt.session,
    ));
    assert!(validate_claude_provider_proxy_receipt(&transport, &session, &receipt).is_err());

    // Compile/link the real resolver+CONNECT+bounded-relay path without invoking DNS or provider.
    let _production_path: fn(
        &ClaudeNarrowTransportContract,
        TcpListener,
        &ClaudeProviderProxySessionBinding,
        &Path,
        &str,
        &ClaudeProxyInvocationSecret,
    ) -> EvalResult<ClaudeProductionRelayAudit> = execute_claude_production_connect_relay_once;
}

struct PreparedClaudeProbe {
    _provider_proxy_listener: TcpListener,
    _engram_listener: TcpListener,
    _denied_listener: TcpListener,
    spec: ClaudeSeatbeltSpec,
    commands: Vec<ClaudeSeatbeltProbeCommand>,
    environment: BTreeMap<String, String>,
}

fn prepare_claude_probe(fixture: &LaunchFixture) -> PreparedClaudeProbe {
    let boundary = boundary(fixture, IsolationHost::ClaudeCode, true, true);
    let provider_proxy_listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let engram_listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let denied_listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let unexpected_bind_listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let unexpected_bind_port = unexpected_bind_listener.local_addr().unwrap().port();
    drop(unexpected_bind_listener);
    let transport = ClaudeNarrowTransportContract {
        provider_proxy_host: "127.0.0.1".to_string(),
        provider_proxy_port: provider_proxy_listener.local_addr().unwrap().port(),
        provider_connect_authority: "api.anthropic.com:443".to_string(),
        engram_daemon_port: Some(engram_listener.local_addr().unwrap().port()),
        max_connect_header_bytes: 16 * 1024,
        max_connect_header_count: 64,
        max_connect_line_bytes: 2048,
        max_connect_requests: 16,
        max_resolved_addresses: 16,
        connect_timeout_ms: 5_000,
        total_timeout_ms: 60_000,
        max_tunnel_bytes_each_direction: 64 * 1024 * 1024,
        idle_timeout_ms: 30_000,
    };
    let spec = claude_seatbelt_spec_for_boundary(&boundary, &transport).unwrap();
    let read_targets = spec
        .forbidden_read_paths
        .iter()
        .map(|protected| {
            let canary = Path::new(protected).join("read-canary.txt");
            write_mode(&canary, b"denied\n", 0o600);
            ClaudeSeatbeltProbeTarget {
                protected_path: protected.clone(),
                canary_path: canary.display().to_string(),
            }
        })
        .collect::<Vec<_>>();
    let write_targets = spec
        .forbidden_write_paths
        .iter()
        .map(|protected| ClaudeSeatbeltProbeTarget {
            protected_path: protected.clone(),
            canary_path: Path::new(protected)
                .join("write-probe")
                .display()
                .to_string(),
        })
        .collect::<Vec<_>>();
    let allowed = fixture.evaluation.join("checkout/allowed-seatbelt.txt");
    write_mode(&allowed, b"allowed\n", 0o600);
    let profile = generate_claude_seatbelt_profile(&spec).unwrap();
    write_mode(&fixture.seatbelt, profile.as_bytes(), 0o600);
    let commands = claude_seatbelt_probe_commands(
        &spec,
        &fixture.seatbelt,
        &allowed,
        &read_targets,
        &write_targets,
        denied_listener.local_addr().unwrap().port(),
        unexpected_bind_port,
    )
    .unwrap();
    PreparedClaudeProbe {
        _provider_proxy_listener: provider_proxy_listener,
        _engram_listener: engram_listener,
        _denied_listener: denied_listener,
        spec,
        commands,
        environment: BTreeMap::from([
            ("LANG".to_string(), "C.UTF-8".to_string()),
            ("LC_ALL".to_string(), "C.UTF-8".to_string()),
            ("NO_COLOR".to_string(), "1".to_string()),
            ("PATH".to_string(), "/usr/bin:/bin".to_string()),
        ]),
    }
}

#[test]
fn rejects_tampered_network_controls_and_seatbelt_probe_commands_before_execution() {
    let fixture = launch_fixture();
    let mut prepared = prepare_claude_probe(&fixture);
    *prepared
        .commands
        .last_mut()
        .unwrap()
        .argv
        .last_mut()
        .unwrap() = "0".to_string();
    assert!(execute_claude_seatbelt_probes(
        &prepared.spec,
        &fixture.seatbelt,
        &prepared.commands,
        &prepared.environment
    )
    .is_err());
    let profile = permission_profile(&fixture);
    let arm = CodexEvaluationArm {
        native_memory_enabled: false,
        min_rollout_idle_hours: 0,
        engram: None,
    };
    write_mode(
        &fixture.evaluation.join("codex-home/isolation.config.toml"),
        generate_codex_evaluation_config_toml(&profile, &arm)
            .unwrap()
            .as_bytes(),
        0o600,
    );
    let allowed = fixture.evaluation.join("checkout/allowed.txt");
    let sibling = fixture.evaluation.join("sibling.txt");
    let forbidden = fixture.source.join("forbidden.txt");
    write_mode(&allowed, b"allowed", 0o600);
    write_mode(&sibling, b"denied", 0o600);
    write_mode(&forbidden, b"denied", 0o600);
    let commands = codex_sandbox_probe_commands(
        Path::new("/Applications/ChatGPT.app/Contents/Resources/codex"),
        &profile,
        &allowed,
        &sibling,
        &forbidden,
        &fixture.evaluation.join("checkout/write-probe"),
        9,
    )
    .unwrap();
    let mut bad = commands;
    *bad.last_mut().unwrap().argv.last_mut().unwrap() = "0".to_string();
    assert!(execute_codex_sandbox_probes(
        &profile,
        &arm,
        &fixture.evaluation,
        &bad,
        &base_environment(&fixture, IsolationHost::Codex, false, false)
    )
    .is_err());
}

#[cfg(target_os = "macos")]
#[test]
#[ignore = "explicit provider-free local enforcement gate"]
fn executes_real_provider_free_codex_sandbox_matrix() {
    let fixture = launch_fixture();
    let profile = permission_profile(&fixture);
    let arm = CodexEvaluationArm {
        native_memory_enabled: false,
        min_rollout_idle_hours: 0,
        engram: None,
    };
    write_mode(
        &fixture.evaluation.join("codex-home/isolation.config.toml"),
        generate_codex_evaluation_config_toml(&profile, &arm)
            .unwrap()
            .as_bytes(),
        0o600,
    );
    let checkout = fixture.evaluation.join("checkout");
    let allowed = checkout.join("allowed-codex.txt");
    let sibling = fixture.evaluation.join("sibling-codex.txt");
    let forbidden = fixture.source.join("forbidden-codex.txt");
    write_mode(&allowed, b"allowed\n", 0o600);
    write_mode(&sibling, b"denied\n", 0o600);
    write_mode(&forbidden, b"denied\n", 0o600);
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let commands = codex_sandbox_probe_commands(
        Path::new("/Applications/ChatGPT.app/Contents/Resources/codex"),
        &profile,
        &allowed,
        &sibling,
        &forbidden,
        &checkout.join("write-probe"),
        listener.local_addr().unwrap().port(),
    )
    .unwrap();
    let audit = execute_codex_sandbox_probes(
        &profile,
        &arm,
        &fixture.evaluation,
        &commands,
        &base_environment(&fixture, IsolationHost::Codex, false, false),
    )
    .unwrap();
    assert!(audit.network_control.succeeded);
    eprintln!("{}", serde_json::to_string_pretty(&audit).unwrap());
}

#[cfg(target_os = "macos")]
#[test]
#[ignore = "explicit provider-free local enforcement gate"]
fn executes_real_provider_free_claude_seatbelt_matrix() {
    let fixture = launch_fixture();
    let prepared = prepare_claude_probe(&fixture);
    let audit = execute_claude_seatbelt_probes(
        &prepared.spec,
        &fixture.seatbelt,
        &prepared.commands,
        &prepared.environment,
    )
    .unwrap();
    assert!(audit.network_control.succeeded);
    eprintln!("{}", serde_json::to_string_pretty(&audit).unwrap());
}

#[cfg(target_os = "macos")]
#[test]
#[ignore = "explicit provider-free local enforcement gate"]
fn executes_runner_owned_fake_upstream_through_the_exact_claude_seatbelt() {
    let fixture = launch_fixture();
    let boundary = boundary(&fixture, IsolationHost::ClaudeCode, false, true);
    let proxy_reservation = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let proxy_port = proxy_reservation.local_addr().unwrap().port();
    drop(proxy_reservation);
    let engram_listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let transport = ClaudeNarrowTransportContract {
        provider_proxy_host: "127.0.0.1".to_string(),
        provider_proxy_port: proxy_port,
        provider_connect_authority: "api.anthropic.com:443".to_string(),
        engram_daemon_port: Some(engram_listener.local_addr().unwrap().port()),
        max_connect_header_bytes: 16 * 1024,
        max_connect_header_count: 64,
        max_connect_line_bytes: 2048,
        max_connect_requests: 16,
        max_resolved_addresses: 16,
        connect_timeout_ms: 5_000,
        total_timeout_ms: 60_000,
        max_tunnel_bytes_each_direction: 64 * 1024 * 1024,
        idle_timeout_ms: 30_000,
    };
    let spec = claude_seatbelt_spec_for_boundary(&boundary, &transport).unwrap();
    let profile = generate_claude_seatbelt_profile(&spec).unwrap();
    write_mode(&fixture.seatbelt, profile.as_bytes(), 0o600);
    let environment = BTreeMap::from([
        ("LANG".to_string(), "C.UTF-8".to_string()),
        ("LC_ALL".to_string(), "C.UTF-8".to_string()),
        ("NO_COLOR".to_string(), "1".to_string()),
        ("PATH".to_string(), "/usr/bin:/bin".to_string()),
    ]);
    let audit = execute_claude_provider_free_transport_probe(
        &spec,
        &fixture.seatbelt,
        &"c".repeat(64),
        "provider-free-transport-0001",
        &environment,
    )
    .unwrap();
    validate_claude_provider_free_transport_probe(
        &spec,
        &"c".repeat(64),
        "provider-free-transport-0001",
        &audit,
    )
    .unwrap();
    assert_eq!(audit.receipt.accepted_connect_count, 1);
    assert_eq!(audit.receipt.rejected_request_count, 0);
    assert_ne!(
        audit.session.provider_process_id,
        audit.session.proxy_process_id
    );
    let mut bad = audit.clone();
    bad.receipt.rejected_request_count = 1;
    assert!(validate_claude_provider_free_transport_probe(
        &spec,
        &"c".repeat(64),
        "provider-free-transport-0001",
        &bad,
    )
    .is_err());
    let mut bad = audit;
    bad.session.provider_process_id += 1;
    assert!(validate_claude_provider_free_transport_probe(
        &spec,
        &"c".repeat(64),
        "provider-free-transport-0001",
        &bad,
    )
    .is_err());
}

#[cfg(target_os = "macos")]
#[test]
fn runner_generates_stdio_fstats_and_detects_an_ambient_inheritable_fd() {
    use std::os::fd::AsRawFd;
    #[cfg(target_os = "macos")]
    use std::os::fd::{FromRawFd, OwnedFd};
    let fixture = launch_fixture();
    let prepared = prepare_isolation_stdio(
        &fixture.evaluation,
        &fixture.evaluation.join("traces/stdout.jsonl"),
        &fixture.evaluation.join("traces/stderr.log"),
    )
    .unwrap();
    assert!(prepared.audit.valid);
    assert_eq!(prepared.audit.child_descriptors.len(), 3);
    assert_eq!(prepared.audit.child_descriptors[1].link_count, 1);
    let mut child = std::process::Command::new("/usr/bin/true");
    let (mut child, spawn_audit) = prepared.spawn(&mut child).unwrap();
    assert!(child.wait().unwrap().success());
    assert!(spawn_audit.valid);
    assert!(spawn_audit.ambient_inheritable_descriptors.len() <= 3);
    let file = fs::File::open("/dev/null").unwrap();
    let fd = file.as_raw_fd();
    assert_eq!(unsafe { libc::fcntl(fd, libc::F_SETFD, 0) }, 0);
    let ambient = inspect_ambient_inheritable_fds().unwrap();
    assert!(ambient.iter().any(|entry| entry.fd == fd));
    assert_eq!(
        unsafe { libc::fcntl(fd, libc::F_SETFD, libc::FD_CLOEXEC) },
        0
    );

    #[cfg(target_os = "macos")]
    {
        let high_fd = unsafe { libc::fcntl(fd, libc::F_DUPFD, 70_000) };
        assert!(
            high_fd >= 70_000,
            "host could not allocate the required high descriptor: {}",
            std::io::Error::last_os_error()
        );
        // SAFETY: `F_DUPFD` returned a new owned descriptor.
        let high_fd = unsafe { OwnedFd::from_raw_fd(high_fd) };
        assert_eq!(
            unsafe { libc::fcntl(high_fd.as_raw_fd(), libc::F_SETFD, 0) },
            0
        );
        let ambient = inspect_ambient_inheritable_fds().unwrap();
        assert!(ambient.iter().any(|entry| entry.fd == high_fd.as_raw_fd()));
        let high_fixture = launch_fixture();
        assert!(prepare_isolation_stdio(
            &high_fixture.evaluation,
            &high_fixture.evaluation.join("traces/stdout.jsonl"),
            &high_fixture.evaluation.join("traces/stderr.log"),
        )
        .is_err());
    }
}

#[test]
fn executes_bounded_codex_auth_status_and_rejects_wrapper_or_mode_mismatch() {
    let fixture = launch_fixture();
    let (launch, boundary) = codex_launch(&fixture, false, false);
    let contract = build_auth_status_contract(&launch, &boundary).unwrap();
    let attestation = execute_auth_status_contract(&launch, &boundary).unwrap();
    assert!(attestation.subscription_login);
    assert!(!attestation.raw_output_retained);
    assert!(attestation.output_bytes < contract.max_output_bytes);
    let mut wrong = contract.clone();
    wrong.argv.insert(1, "--profile=isolation".to_string());
    assert!(validate_auth_status_attestation(&wrong, &attestation, &launch, &boundary).is_err());
    let mut credential_alias = contract.clone();
    credential_alias.argv[0] = fixture
        .evaluation
        .join("codex-home/auth.json")
        .display()
        .to_string();
    let error =
        validate_auth_status_attestation(&credential_alias, &attestation, &launch, &boundary)
            .unwrap_err();
    assert!(error.to_string().contains("caller paths were not accessed"));
    let mut credential_launch = launch.clone();
    credential_launch.argv[0] = fixture
        .evaluation
        .join("codex-home/auth.json")
        .display()
        .to_string();
    let error = build_auth_status_contract(&credential_launch, &boundary).unwrap_err();
    assert!(error.to_string().contains("before content access"));
    let fixture = launch_fixture();
    write_mode(
        &fixture.codex_executable,
        b"#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then printf '%s\\n' 'codex-cli 0.153.3'; else printf '%s\\n' 'Logged in using an API key'; fi\n",
        0o700,
    );
    let (launch, boundary) = codex_launch(&fixture, false, false);
    assert!(execute_auth_status_contract(&launch, &boundary).is_err());
}

#[cfg(target_os = "macos")]
#[test]
fn executes_bounded_claude_auth_status_through_the_exact_seatbelt_wrapper() {
    let fixture = launch_fixture();
    write_mode(
        &fixture.claude_executable,
        b"#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then printf '%s\\n' '2.1.260 (Claude Code)'; else printf '%s\\n' '{\"loggedIn\":true,\"authMethod\":\"claude.ai\",\"apiProvider\":\"firstParty\"}'; fi\n",
        0o700,
    );
    let (launch, boundary) = claude_launch(&fixture, false, false);
    let contract = build_auth_status_contract(&launch, &boundary).unwrap();
    let attestation = execute_auth_status_contract(&launch, &boundary).unwrap();
    assert!(attestation.account_ready);
    assert!(attestation.subscription_login);
    assert!(!attestation.api_key_mode);
    assert!(!attestation.raw_output_retained);

    let mut bad = contract.clone();
    bad.argv.remove(3);
    assert!(validate_auth_status_attestation(&bad, &attestation, &launch, &boundary).is_err());
    let credential = fixture
        .evaluation
        .join("claude-config/.credentials.json")
        .display()
        .to_string();
    let mut provider_alias = contract.clone();
    provider_alias.argv[4] = credential.clone();
    let error = validate_auth_status_attestation(&provider_alias, &attestation, &launch, &boundary)
        .unwrap_err();
    assert!(error.to_string().contains("caller paths were not accessed"));
    let mut profile_alias = contract.clone();
    profile_alias.argv[2] = credential.clone();
    let error = validate_auth_status_attestation(&profile_alias, &attestation, &launch, &boundary)
        .unwrap_err();
    assert!(error.to_string().contains("caller paths were not accessed"));
    let mut credential_launch = launch.clone();
    credential_launch.argv[4] = credential;
    let error = build_auth_status_contract(&credential_launch, &boundary).unwrap_err();
    assert!(error.to_string().contains("before content access"));

    let fixture = launch_fixture();
    write_mode(
        &fixture.claude_executable,
        b"#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then printf '%s\\n' '2.1.260 (Claude Code)'; else printf '%s\\n' '{\"loggedIn\":true,\"authMethod\":\"apiKey\",\"apiProvider\":\"anthropicApi\"}'; fi\n",
        0o700,
    );
    let (launch, boundary) = claude_launch(&fixture, false, false);
    assert!(execute_auth_status_contract(&launch, &boundary).is_err());
}

fn runtime(name: &str, index: u8) -> ColimaRuntimeAttestation {
    ColimaRuntimeAttestation {
        path: format!("/opt/engram-pilot/bin/{name}"),
        version: format!("{name}-frozen-1"),
        sha256: format!("{:064x}", u64::from(index) + 1),
    }
}

fn colima_commands() -> Vec<ColimaCommandContract> {
    [
        "bwrap_probe",
        "colima_start",
        "findmnt",
        "guest_rehash",
        "runtime_attestation",
        "ssh_config",
        "transfer_bundle",
    ]
    .into_iter()
    .map(|label| {
        let argv = vec![format!("/usr/bin/{label}"), "--frozen".to_string()];
        ColimaCommandContract {
            label: label.to_string(),
            argv_sha256: serialized_digest(&argv),
            argv,
        }
    })
    .collect()
}

fn colima_contract() -> ColimaIsolationContract {
    let names = [
        "cargo",
        "claude",
        "codex",
        "engram",
        "engram_eval",
        "node",
        "rustc",
    ];
    ColimaIsolationContract {
        schema_version: 1,
        profile_name: "engram-stale-isolated-01".to_string(),
        vm_type: "vz".to_string(),
        architecture: "aarch64".to_string(),
        mount_mode: "none".to_string(),
        rosetta: false,
        forward_ssh_agent: false,
        disk_gib: 40,
        transfer_mode: "ssh_stream_hash_manifest".to_string(),
        teaching_user: "engram-teach".to_string(),
        evaluation_user: "engram-eval".to_string(),
        required_runtimes: names
            .iter()
            .enumerate()
            .map(|(index, name)| ((*name).to_string(), runtime(name, index as u8)))
            .collect(),
        colima_profile_sha256: "a".repeat(64),
        bwrap_profile_sha256: "b".repeat(64),
        command_contracts: colima_commands(),
    }
}

fn colima_observation(contract: &ColimaIsolationContract) -> ColimaIsolationObservation {
    ColimaIsolationObservation {
        observed_in_disposable_guest: true,
        environment_id: "colima-engram-stale-isolated-01".to_string(),
        operating_system: "linux".to_string(),
        distribution: "ubuntu-24.04.4".to_string(),
        architecture: "aarch64".to_string(),
        root_filesystem: "ext4".to_string(),
        mounts: vec![ColimaMountObservation {
            source: "/dev/vda1".to_string(),
            target: "/".to_string(),
            filesystem: "ext4".to_string(),
            options: vec!["rw".to_string()],
        }],
        host_home_visible: false,
        rosetta_registered: false,
        ssh_auth_sock_present: false,
        ssh_forward_agent: false,
        docker_socket_present: false,
        teaching_uid: 1001,
        evaluation_uid: 1002,
        bwrap: Some(ColimaRuntimeAttestation {
            path: "/usr/bin/bwrap".to_string(),
            version: "bubblewrap 0.9.0".to_string(),
            sha256: "c".repeat(64),
        }),
        apparmor_enabled: true,
        bwrap_apparmor_profile_enforced: true,
        bwrap_cross_uid_probe_passed: true,
        runtimes: contract.required_runtimes.clone(),
        transfer_manifest_sha256: "d".repeat(64),
        guest_manifest_sha256: "d".repeat(64),
        guest_bundle_on_vm_owned_ext4: true,
        authentication_performed_inside_guest: true,
        api_key_fallback_used: false,
        colima_profile_sha256: contract.colima_profile_sha256.clone(),
        bwrap_profile_sha256: contract.bwrap_profile_sha256.clone(),
        command_receipts: contract
            .command_contracts
            .iter()
            .map(|command| ColimaCommandReceipt {
                label: command.label.clone(),
                argv_sha256: command.argv_sha256.clone(),
                exit_code: 0,
            })
            .collect(),
    }
}

#[test]
fn colima_model_rejects_unobserved_mount_runtime_and_receipt_bypasses_and_never_authorizes() {
    let contract = colima_contract();
    let observation = colima_observation(&contract);
    let audit = validate_colima_isolation(&contract, &observation);
    assert!(audit.valid, "{:?}", audit.failures);
    assert!(!audit.authorizes_provider_execution);
    let mut bad = observation.clone();
    bad.observed_in_disposable_guest = false;
    assert!(!validate_colima_isolation(&contract, &bad).valid);
    let mut bad = observation.clone();
    bad.mounts.clear();
    assert!(!validate_colima_isolation(&contract, &bad).valid);
    let mut bad = observation.clone();
    bad.mounts.push(ColimaMountObservation {
        source: "tmpfs".to_string(),
        target: "/unrecognized".to_string(),
        filesystem: "tmpfs".to_string(),
        options: vec![],
    });
    assert!(!validate_colima_isolation(&contract, &bad).valid);
    let mut bad = observation.clone();
    let rustc = bad.runtimes["rustc"].path.clone();
    bad.runtimes.get_mut("node").unwrap().path = rustc;
    assert!(!validate_colima_isolation(&contract, &bad).valid);
    let mut bad = observation.clone();
    let rustc = bad.runtimes["rustc"].sha256.clone();
    bad.runtimes.get_mut("node").unwrap().sha256 = rustc;
    assert!(!validate_colima_isolation(&contract, &bad).valid);
    let mut bad = observation.clone();
    bad.runtimes.remove("rustc");
    assert!(!validate_colima_isolation(&contract, &bad).valid);
    let mut bad = observation.clone();
    bad.bwrap = None;
    assert!(!validate_colima_isolation(&contract, &bad).valid);
    let mut bad = observation.clone();
    bad.apparmor_enabled = false;
    assert!(!validate_colima_isolation(&contract, &bad).valid);
    let mut bad = observation.clone();
    bad.bwrap_apparmor_profile_enforced = false;
    assert!(!validate_colima_isolation(&contract, &bad).valid);
    let mut bad = observation.clone();
    bad.bwrap_cross_uid_probe_passed = false;
    assert!(!validate_colima_isolation(&contract, &bad).valid);
    let mut bad = observation.clone();
    bad.command_receipts[0].exit_code = 1;
    assert!(!validate_colima_isolation(&contract, &bad).valid);
    let mut bad = observation.clone();
    bad.bwrap_profile_sha256 = "e".repeat(64);
    assert!(!validate_colima_isolation(&contract, &bad).valid);
    let mut bad = observation.clone();
    bad.rosetta_registered = true;
    assert!(!validate_colima_isolation(&contract, &bad).valid);
    let mut bad = observation.clone();
    bad.docker_socket_present = true;
    assert!(!validate_colima_isolation(&contract, &bad).valid);
    let mut bad = observation.clone();
    bad.api_key_fallback_used = true;
    assert!(!validate_colima_isolation(&contract, &bad).valid);
    let mut bad = observation;
    bad.ssh_auth_sock_present = true;
    assert!(!validate_colima_isolation(&contract, &bad).valid);
}

#[cfg(unix)]
#[test]
fn phase_outputs_are_derived_nonempty_and_target_debug_symlinks_fail_closed() {
    use std::os::unix::fs::symlink;
    let temp = tempfile::tempdir().unwrap();
    let phase_root = temp.path().join("phase");
    mkdir_private(&phase_root);
    let contract =
        derive_isolation_phase_outputs(IsolationExecutablePhase::Evaluation, &phase_root, 1)
            .unwrap();
    assert_eq!(contract.output_paths.len(), 4);
    let audit = inspect_isolation_local_gates(temp.path(), &contract, temp.path()).unwrap();
    assert!(audit.valid, "{:?}", audit.failures);
    assert!(audit.phase_root_empty);
    let mut forged = contract.clone();
    forged.output_paths.clear();
    assert!(inspect_isolation_local_gates(temp.path(), &forged, temp.path()).is_err());
    assert!(
        derive_isolation_phase_outputs(IsolationExecutablePhase::Evaluation, &phase_root, 0)
            .is_err()
    );
    write_mode(&phase_root.join("partial.intent"), b"interrupted\n", 0o600);
    let audit = inspect_isolation_local_gates(temp.path(), &contract, temp.path()).unwrap();
    assert!(!audit.valid);
    assert!(!audit.phase_root_empty);
    mkdir_private(&temp.path().join("target"));
    symlink(
        temp.path().join("missing-debug-target"),
        temp.path().join("target/debug"),
    )
    .unwrap();
    let audit = inspect_isolation_local_gates(temp.path(), &contract, temp.path()).unwrap();
    assert!(!audit.repository_target_debug_absent);
}
