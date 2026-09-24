#![cfg(unix)]

use engram_eval::native_audit::{AuditStatus, NativeLanePhase};
#[cfg(target_os = "macos")]
use engram_eval::native_instructions_control::PreparedNativeInstructionsControl;
#[cfg(target_os = "macos")]
use engram_eval::native_pilot::MemoryLayer;
use engram_eval::native_pilot::{
    CodexAuthenticationMode, NativePilotEvaluationPrerequisiteState, NativePilotProtocol,
    NativePilotResourceBudgets, NativePilotRunRef, PreparedNativePilot,
};
#[cfg(target_os = "macos")]
use engram_eval::native_runner::NativePilotPlanAttestation;
use engram_eval::native_runner::{
    prepare_native_memory_pilot_evaluation_recovery,
    prepare_native_memory_pilot_execution_recovery, NativePilotLaneExecution, NativePilotRunPhase,
    NativePilotRunReport,
};
use engram_eval::native_stale::{
    audit_native_stale_safety, audit_native_stale_safety_preparation, NativeStaleSafetyBoundary,
    NativeStaleSafetyCapability, NativeStaleSafetyCase, NativeStaleSafetyLane,
    NativeStaleSafetyProtocol, NativeStaleSafetySpec,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
#[cfg(target_os = "macos")]
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn sha256(bytes: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(bytes);
    format!("{:x}", digest.finalize())
}

fn protocol() -> NativeStaleSafetyProtocol {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../evals/native_memory_pilot_v1/protocol-schema-10-expired-procedure-forward.json");
    let mut base: NativePilotProtocol =
        serde_json::from_reader(fs::File::open(source).unwrap()).unwrap();
    base.pilot_id = "native-stale-safety-synthetic-preparation".to_string();
    base.codex_authentication_mode = CodexAuthenticationMode::ChatgptFileCache;
    base.resource_budgets = Some(NativePilotResourceBudgets {
        max_engram_result_bytes_per_call: 8_192,
        max_engram_result_bytes_per_lane: 16_384,
        max_incremental_total_tokens_per_lane: 50_000,
        max_incremental_runner_duration_ms_per_lane: 30_000,
    });
    let mut expired = base.cases[0].clone();
    expired.id = "case-1111111111111111".to_string();
    let mut missing = expired.clone();
    missing.id = "case-2222222222222222".to_string();
    missing.evaluation_prerequisite_state = NativePilotEvaluationPrerequisiteState::Missing;
    missing.condition_evidence_output_contains = None;
    missing.procedure_expiry_seconds = None;
    base.cases = vec![expired, missing];

    let ordered = [
        ("case-1111111111111111", "codex_native_memory"),
        ("case-2222222222222222", "claude_lean_engram"),
        ("case-1111111111111111", "codex_engram_plus_native"),
        ("case-2222222222222222", "claude_native_memory"),
        ("case-1111111111111111", "codex_lean_engram"),
        ("case-2222222222222222", "claude_engram_plus_native"),
        ("case-1111111111111111", "claude_native_memory"),
        ("case-2222222222222222", "codex_lean_engram"),
        ("case-1111111111111111", "claude_engram_plus_native"),
        ("case-2222222222222222", "codex_native_memory"),
        ("case-1111111111111111", "claude_lean_engram"),
        ("case-2222222222222222", "codex_engram_plus_native"),
    ];
    base.run_order = ordered
        .iter()
        .map(|(case_id, arm)| NativePilotRunRef {
            case_id: (*case_id).to_string(),
            arm: (*arm).to_string(),
            repetition: 1,
        })
        .collect();
    let layers = base
        .arms
        .iter()
        .map(|arm| (arm.id.as_str(), arm.memory_layer))
        .collect::<BTreeMap<_, _>>();
    let lanes = base
        .run_order
        .iter()
        .enumerate()
        .map(|(index, run)| {
            let order = u32::try_from(index + 1).unwrap();
            let uses_native = matches!(
                layers[run.arm.as_str()],
                engram_eval::native_pilot::MemoryLayer::Native
                    | engram_eval::native_pilot::MemoryLayer::Both
            );
            NativeStaleSafetyLane {
                order,
                case_id: run.case_id.clone(),
                arm: run.arm.clone(),
                repetition: 1,
                opaque_path_label: format!("lane-{order:016x}"),
                native_semantic_marker: uses_native.then(|| format!("MKR_{order:032x}")),
                native_ttl_marker: uses_native.then(|| format!("AUX_{:032x}", order + 100)),
                native_auxiliary_marker: None,
            }
        })
        .collect();
    NativeStaleSafetyProtocol {
        native_pilot: base,
        stale_safety: NativeStaleSafetySpec {
            family_version: 1,
            capabilities: vec![
                NativeStaleSafetyCapability::TraceCorrelatedCausalBoundary,
                NativeStaleSafetyCapability::ExactEngramDiagnostic,
                NativeStaleSafetyCapability::NativeRetentionMarkerCorrelation,
                NativeStaleSafetyCapability::CombinedNativeContentAttestation,
                NativeStaleSafetyCapability::SharedRetentionGate,
                NativeStaleSafetyCapability::BoundedProcedureMatch,
                NativeStaleSafetyCapability::ExactMissingSourceInspection,
                NativeStaleSafetyCapability::NoReplayLifecycle,
                NativeStaleSafetyCapability::RunnerControlledNoncredentialScan,
                NativeStaleSafetyCapability::ResourceAndDiskGates,
            ],
            shared_retention_gate_hours: 1,
            retention_absent_signal: "retention_absent".to_string(),
            max_scan_bytes_per_file: 64 * 1024,
            output_contract: None,
            signal_mapping: None,
            verification_time_contract: None,
            boundary_decision_table: None,
            result_precedence: None,
            first_relevant_action_contract: None,
            causal_rules: None,
            complete_output_marker_scan: None,
            contract_equivalence: None,
            cases: vec![
                NativeStaleSafetyCase {
                    case_id: "case-1111111111111111".to_string(),
                    boundary: NativeStaleSafetyBoundary::VerifiedProcedureExpired,
                    expected_boundary_signal: None,
                    native_marker_roles: Vec::new(),
                    evaluation_source_state: None,
                    verification_state_at_evaluation: None,
                    verification_expiry_rule: None,
                    verification_expiry_seconds: None,
                    procedure_query_max_chars: 512,
                    procedure_query_required_terms: vec![
                        "context".to_string(),
                        "probe".to_string(),
                    ],
                },
                NativeStaleSafetyCase {
                    case_id: "case-2222222222222222".to_string(),
                    boundary: NativeStaleSafetyBoundary::TrackedPrerequisiteSourceAbsent,
                    expected_boundary_signal: None,
                    native_marker_roles: Vec::new(),
                    evaluation_source_state: None,
                    verification_state_at_evaluation: None,
                    verification_expiry_rule: None,
                    verification_expiry_seconds: None,
                    procedure_query_max_chars: 512,
                    procedure_query_required_terms: vec![
                        "context".to_string(),
                        "probe".to_string(),
                    ],
                },
            ],
            lanes,
            adversarial_scorer_fixtures: Vec::new(),
        },
    }
}

fn write_executable(path: &Path, body: &str) {
    fs::write(path, body).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}

fn fake_binaries(root: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let engram = root.join("engram");
    write_executable(
        &engram,
        &format!(
            r#"#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "engram {}"
  exit 0
fi
if [ "$1" = "contract" ]; then
  script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)
  script_path="$script_dir/$(basename -- "$0")"
  if command -v shasum >/dev/null 2>&1; then
    script_sha=$(shasum -a 256 "$script_path" | awk '{{print $1}}')
  else
    script_sha=$(sha256sum "$script_path" | awk '{{print $1}}')
  fi
  printf '{{"schema_version":3,"cli_version":"{}","health_schema_version":1,"mcp_contract_version":1,"mcp_protocol_version":"test","profile":"agent","mcp_tool_count":6,"mcp_tools_sha256":"{}","profile_instructions_sha256":"{}","effective_runtime":{{"verified":true,"mcp_tool_count":6,"mcp_tools_sha256":"{}","profile_instructions_sha256":"{}","restricted_tool_rejected":true,"review_authority_rejected":true}},"executable_path":"%s","executable_sha256":"%s"}}\n' "$script_path" "$script_sha"
  exit 0
fi
exit 97
"#,
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_VERSION"),
            "a".repeat(64),
            "b".repeat(64),
            "a".repeat(64),
            "b".repeat(64),
        ),
    );
    let codex = root.join("codex");
    write_executable(
        &codex,
        "#!/bin/sh\n[ \"$1\" = \"--version\" ] && { echo 'codex-cli synthetic'; exit 0; }\nexit 97\n",
    );
    write_executable(
        &root.join("codex-code-mode-host"),
        "#!/bin/sh\n[ \"$1\" = \"--help\" ] && { echo 'Usage: codex-code-mode-host'; exit 0; }\nexit 97\n",
    );
    let claude = root.join("claude");
    write_executable(
        &claude,
        "#!/bin/sh\n[ \"$1\" = \"--version\" ] && { echo 'claude synthetic'; exit 0; }\nexit 97\n",
    );
    (engram, codex, claude)
}

#[cfg(target_os = "macos")]
fn runtime_test_library() -> PathBuf {
    let location = PathBuf::from(
        std::env::var("ORT_LIB_LOCATION")
            .expect("ORT_LIB_LOCATION must identify the pinned ONNX runtime for this test"),
    );
    [
        location.join("libonnxruntime.1.20.0.dylib"),
        location.join("lib/libonnxruntime.1.20.0.dylib"),
    ]
    .into_iter()
    .find(|path| path.is_file())
    .expect("pinned ONNX runtime library was not found under ORT_LIB_LOCATION")
}

#[cfg(target_os = "macos")]
fn set_mode(path: &Path, mode: u32) {
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
}

#[cfg(target_os = "macos")]
fn make_private_runtime_bundle(root: &Path) -> (PathBuf, PathBuf) {
    let bin = root.join("bin");
    let lib = root.join("lib");
    fs::create_dir_all(&bin).unwrap();
    fs::create_dir_all(&lib).unwrap();
    for directory in [root, bin.as_path(), lib.as_path()] {
        set_mode(directory, 0o700);
    }
    let library = lib.join("libonnxruntime.1.20.0.dylib");
    fs::copy(runtime_test_library(), &library).unwrap();
    set_mode(&library, 0o600);
    (bin, library)
}

#[cfg(target_os = "macos")]
fn normalize_test_executable_rpath(executable: &Path) {
    let output = Command::new("/usr/bin/otool")
        .args(["-l", executable.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    let mut expects_path = false;
    let mut rpaths = Vec::new();
    for line in String::from_utf8(output.stdout).unwrap().lines() {
        let line = line.trim();
        if line == "cmd LC_RPATH" {
            expects_path = true;
        } else if expects_path && line.starts_with("path ") {
            rpaths.push(
                line.strip_prefix("path ")
                    .unwrap()
                    .split_once(" (offset ")
                    .unwrap()
                    .0
                    .to_string(),
            );
            expects_path = false;
        }
    }
    for rpath in rpaths {
        let status = Command::new("/usr/bin/install_name_tool")
            .args(["-delete_rpath", &rpath])
            .arg(executable)
            .status()
            .unwrap();
        assert!(status.success());
    }
    let status = Command::new("/usr/bin/install_name_tool")
        .args(["-add_rpath", "@loader_path/../lib"])
        .arg(executable)
        .status()
        .unwrap();
    assert!(status.success());
    set_mode(executable, 0o700);
}

#[cfg(target_os = "macos")]
fn loader_closed_family_v3_binaries(root: &Path) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let evaluator_root = root.join("evaluator-bundle");
    fs::create_dir(&evaluator_root).unwrap();
    set_mode(&evaluator_root, 0o700);
    let (evaluator_bin, _) = make_private_runtime_bundle(&evaluator_root);
    let evaluator = evaluator_bin.join("engram-eval");
    fs::copy(env!("CARGO_BIN_EXE_engram-eval"), &evaluator).unwrap();
    set_mode(&evaluator, 0o700);
    normalize_test_executable_rpath(&evaluator);

    let engram_root = root.join("engram-bundle");
    fs::create_dir(&engram_root).unwrap();
    set_mode(&engram_root, 0o700);
    let (engram_bin, _) = make_private_runtime_bundle(&engram_root);
    let source = engram_root.join("fixture-engram.c");
    fs::write(
        &source,
        r#"#define _DARWIN_C_SOURCE
#include <limits.h>
#include <onnxruntime_c_api.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int self_sha256(const char *path, char digest[65]) {
    char command[PATH_MAX + 64];
    if (snprintf(command, sizeof(command), "/usr/bin/shasum -a 256 '%s'", path)
        >= (int)sizeof(command)) {
        return 0;
    }
    FILE *pipe = popen(command, "r");
    if (pipe == NULL) {
        return 0;
    }
    int matched = fscanf(pipe, "%64[0-9a-f]", digest);
    int status = pclose(pipe);
    digest[64] = '\0';
    return matched == 1 && status == 0;
}

int main(int argc, char **argv) {
    if (OrtGetApiBase() == NULL) {
        return 96;
    }
    if (argc == 2 && strcmp(argv[1], "--version") == 0) {
        puts("engram " ENGRAM_FIXTURE_VERSION);
        return 0;
    }
    if (argc == 6 && strcmp(argv[1], "contract") == 0
        && strcmp(argv[2], "--profile") == 0 && strcmp(argv[3], "agent") == 0
        && strcmp(argv[4], "--verify-runtime") == 0 && strcmp(argv[5], "--json") == 0) {
        char resolved[PATH_MAX];
        char digest[65];
        if (realpath(argv[0], resolved) == NULL || !self_sha256(resolved, digest)) {
            return 95;
        }
        printf("{\"schema_version\":3,\"cli_version\":\"%s\","
               "\"health_schema_version\":1,\"mcp_contract_version\":1,"
               "\"mcp_protocol_version\":\"test\",\"profile\":\"agent\","
               "\"mcp_tool_count\":6,\"mcp_tools_sha256\":\"%064d\","
               "\"profile_instructions_sha256\":\"%064d\","
               "\"effective_runtime\":{\"verified\":true,\"mcp_tool_count\":6,"
               "\"mcp_tools_sha256\":\"%064d\","
               "\"profile_instructions_sha256\":\"%064d\","
               "\"restricted_tool_rejected\":true,\"review_authority_rejected\":true},"
               "\"executable_path\":\"%s\",\"executable_sha256\":\"%s\"}\n",
               ENGRAM_FIXTURE_VERSION, 0, 0, 0, 0, resolved, digest);
        return 0;
    }
    return 97;
}
"#,
    )
    .unwrap();
    let library_dir = runtime_test_library().parent().unwrap().to_path_buf();
    let include = library_dir.parent().unwrap().join("include");
    let engram = engram_bin.join("engram");
    let status = Command::new("/usr/bin/clang")
        .args(["-std=c11", "-Wall", "-Werror"])
        .arg(format!(
            "-DENGRAM_FIXTURE_VERSION=\"{}\"",
            env!("CARGO_PKG_VERSION")
        ))
        .arg("-I")
        .arg(include)
        .arg(&source)
        .arg("-L")
        .arg(library_dir)
        .arg("-lonnxruntime")
        .arg("-Wl,-rpath,@loader_path/../lib")
        .arg("-o")
        .arg(&engram)
        .status()
        .unwrap();
    assert!(status.success());
    set_mode(&engram, 0o700);

    let tools = root.join("tools");
    fs::create_dir(&tools).unwrap();
    set_mode(&tools, 0o700);
    let (_, codex, claude) = fake_binaries(&tools);
    (evaluator, engram, codex, claude)
}

#[cfg(target_os = "macos")]
#[test]
fn family_v3_preparation_and_controls_bind_exact_inline_claude_config() {
    let root = tempfile::tempdir().unwrap();
    let protocol_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(
            "../evals/native_memory_pilot_v1/\
             protocol-native-stale-safety-v3-r2-base-schema-10-direct-host-file-cache.json",
        )
        .canonicalize()
        .unwrap();
    let control_protocol_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(
            "../evals/native_memory_pilot_v1/\
             protocol-native-stale-instructions-control-v1-draft.json",
        )
        .canonicalize()
        .unwrap();
    let (evaluator, engram, codex, claude) = loader_closed_family_v3_binaries(root.path());
    let treatment = root.path().join("treatment");
    let command = Command::new(&evaluator)
        .args([
            "prepare-native-stale-safety",
            "--protocol",
            protocol_path.to_str().unwrap(),
            "--output",
            treatment.to_str().unwrap(),
            "--engram-bin",
            engram.to_str().unwrap(),
            "--codex-bin",
            codex.to_str().unwrap(),
            "--claude-bin",
            claude.to_str().unwrap(),
        ])
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "commit.gpgsign")
        .env("GIT_CONFIG_VALUE_0", "false")
        .output()
        .unwrap();
    assert!(
        command.status.success(),
        "{}",
        String::from_utf8_lossy(&command.stderr)
    );

    let run_plan = treatment.join("run-plan.json");
    let plan: PreparedNativePilot =
        serde_json::from_reader(fs::File::open(&run_plan).unwrap()).unwrap();
    assert_eq!(plan.stale_safety.as_ref().unwrap().family_version, 3);
    assert_eq!(
        plan.runtime_libraries
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["onnxruntime"]
    );
    let runtime = &plan.runtime_libraries["onnxruntime"];
    assert_eq!(runtime.install_name, "@rpath/libonnxruntime.1.20.0.dylib");
    assert_eq!(
        runtime.sha256,
        sha256(&fs::read(runtime_test_library()).unwrap())
    );
    assert_eq!(
        runtime
            .consumers
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["engram", "engram_eval"]
    );
    let claude_lanes = plan
        .lanes
        .iter()
        .filter(|lane| lane.host == "claude_code")
        .collect::<Vec<_>>();
    assert_eq!(
        claude_lanes
            .iter()
            .map(|lane| lane.order)
            .collect::<Vec<_>>(),
        vec![2, 4, 6, 7, 9, 11]
    );
    assert_eq!(
        claude_lanes
            .iter()
            .filter(|lane| matches!(lane.memory_layer, MemoryLayer::Engram | MemoryLayer::Both))
            .map(|lane| lane.order)
            .collect::<Vec<_>>(),
        vec![2, 6, 9, 11]
    );
    assert_eq!(
        claude_lanes
            .iter()
            .filter(|lane| { matches!(lane.memory_layer, MemoryLayer::Native | MemoryLayer::Both) })
            .map(|lane| lane.order)
            .collect::<Vec<_>>(),
        vec![4, 6, 7, 9]
    );
    let exact_option = |argv: &[String], option: &str| {
        let values = argv
            .windows(2)
            .filter(|pair| pair[0] == option)
            .map(|pair| pair[1].clone())
            .collect::<Vec<_>>();
        assert_eq!(values.len(), 1, "expected exactly one {option} value");
        values.into_iter().next().unwrap()
    };
    for lane in claude_lanes {
        let lane_root = Path::new(&lane.acceptance_contract).parent().unwrap();
        let expected_settings = lane_root
            .join("claude-settings.json")
            .canonicalize()
            .unwrap();
        let expected_mcp = lane_root
            .join(
                if matches!(lane.memory_layer, MemoryLayer::Engram | MemoryLayer::Both) {
                    "claude-mcp.json"
                } else {
                    "claude-mcp-empty.json"
                },
            )
            .canonicalize()
            .unwrap();
        let settings_bytes = fs::read(&expected_settings).unwrap();
        let mcp_bytes = fs::read(&expected_mcp).unwrap();
        let settings: serde_json::Value = serde_json::from_slice(&settings_bytes).unwrap();
        let mcp: serde_json::Value = serde_json::from_slice(&mcp_bytes).unwrap();
        let settings_inline = serde_json::to_string(&settings).unwrap();
        let mcp_inline = serde_json::to_string(&mcp).unwrap();
        assert_eq!(settings_bytes, settings_inline.as_bytes());
        assert_eq!(mcp_bytes, mcp_inline.as_bytes());
        for argv in [&lane.teaching_argv, &lane.evaluation_argv] {
            assert_eq!(exact_option(argv, "--settings"), settings_inline);
            assert_eq!(exact_option(argv, "--mcp-config"), mcp_inline);
            assert_ne!(
                exact_option(argv, "--settings"),
                expected_settings.display().to_string()
            );
            assert_ne!(
                exact_option(argv, "--mcp-config"),
                expected_mcp.display().to_string()
            );
        }
        for path in [&expected_settings, &expected_mcp] {
            let metadata = fs::symlink_metadata(path).unwrap();
            assert!(metadata.is_file());
            assert!(!metadata.file_type().is_symlink());
            assert_eq!(metadata.uid(), unsafe { libc::geteuid() });
            assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
            assert_eq!(metadata.nlink(), 1);
        }
        let native_memory_enabled = [4, 6, 7, 9].contains(&lane.order);
        assert_eq!(
            settings,
            serde_json::json!({
                "autoMemoryEnabled": native_memory_enabled,
                "autoMemoryDirectory": lane_root
                    .join("claude-memory")
                    .canonicalize()
                    .unwrap()
                    .display()
                    .to_string(),
            })
        );
        if matches!(lane.memory_layer, MemoryLayer::Engram | MemoryLayer::Both) {
            assert_eq!(
                mcp,
                serde_json::json!({
                    "mcpServers": {
                        "engram": {
                            "type": "stdio",
                            "command": engram.canonicalize().unwrap().display().to_string(),
                            "args": [
                                "serve",
                                "--project",
                                format!("native-pilot-{:02}", lane.order),
                                "--profile",
                                "agent",
                            ],
                            "env": {
                                "ENGRAM_HOME": lane_root
                                    .join("engram-home")
                                    .canonicalize()
                                    .unwrap()
                                    .display()
                                    .to_string(),
                            },
                        }
                    }
                })
            );
        } else {
            assert_eq!(mcp, serde_json::json!({"mcpServers": {}}));
        }
    }
    let audit = audit_native_stale_safety_preparation(&protocol_path, &run_plan).unwrap();
    assert!(audit.valid, "{:?}", audit.failures);
    assert!(audit.provider_outputs_absent);
    assert!(audit.auth_caches_absent);
    let attestation = Command::new(&evaluator)
        .args([
            "attest-native-memory-pilot",
            "--plan",
            run_plan.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        attestation.status.success(),
        "{}",
        String::from_utf8_lossy(&attestation.stderr)
    );
    let attestation: NativePilotPlanAttestation =
        serde_json::from_slice(&attestation.stdout).unwrap();
    assert!(attestation.verified);
    assert_eq!(attestation.runtime_libraries, plan.runtime_libraries);

    let controls = root.path().join("controls");
    let control = Command::new(&evaluator)
        .args([
            "prepare-native-stale-instructions-control",
            "--protocol",
            control_protocol_path.to_str().unwrap(),
            "--treatment-protocol",
            protocol_path.to_str().unwrap(),
            "--treatment-plan",
            run_plan.to_str().unwrap(),
            "--output",
            controls.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        control.status.success(),
        "{}",
        String::from_utf8_lossy(&control.stderr)
    );
    let control_plan: PreparedNativeInstructionsControl =
        serde_json::from_reader(fs::File::open(controls.join("run-plan.json")).unwrap()).unwrap();
    assert_eq!(control_plan.lanes.len(), 4);
    assert!(!control_plan.execution_approved);
    assert_eq!(control_plan.runtime_libraries, plan.runtime_libraries);
    assert_eq!(
        control_plan
            .lanes
            .iter()
            .filter(|lane| lane.host == "claude_code")
            .map(|lane| (lane.admission_ordinal, lane.source_treatment_lane_order))
            .collect::<Vec<_>>(),
        vec![(32, 2), (33, 11)]
    );
    for lane in &control_plan.lanes {
        if lane.host != "claude_code" {
            assert!(!lane
                .evaluation_argv
                .iter()
                .any(|arg| { matches!(arg.as_str(), "--settings" | "--mcp-config") }));
            continue;
        }
        let lane_root = Path::new(&lane.effective_config_intent.path)
            .parent()
            .unwrap();
        let settings_path = lane_root.join("claude-settings.json");
        let mcp_path = lane_root.join("claude-mcp-empty.json");
        let settings_bytes = fs::read(&settings_path).unwrap();
        let mcp_bytes = fs::read(&mcp_path).unwrap();
        let settings: serde_json::Value = serde_json::from_slice(&settings_bytes).unwrap();
        let mcp: serde_json::Value = serde_json::from_slice(&mcp_bytes).unwrap();
        let settings_inline = serde_json::to_string(&settings).unwrap();
        let mcp_inline = serde_json::to_string(&mcp).unwrap();
        assert_eq!(settings_bytes, settings_inline.as_bytes());
        assert_eq!(mcp_bytes, mcp_inline.as_bytes());
        assert_eq!(
            exact_option(&lane.evaluation_argv, "--settings"),
            settings_inline
        );
        assert_eq!(
            exact_option(&lane.evaluation_argv, "--mcp-config"),
            mcp_inline
        );
        assert_ne!(
            exact_option(&lane.evaluation_argv, "--settings"),
            settings_path.display().to_string()
        );
        assert_ne!(
            exact_option(&lane.evaluation_argv, "--mcp-config"),
            mcp_path.display().to_string()
        );
        assert_eq!(
            settings,
            serde_json::json!({
                "autoMemoryEnabled": false,
                "autoMemoryDirectory": lane_root.join("claude-memory").display().to_string(),
            })
        );
        assert_eq!(mcp, serde_json::json!({"mcpServers": {}}));
        for path in [&settings_path, &mcp_path] {
            let metadata = fs::symlink_metadata(path).unwrap();
            assert!(metadata.is_file());
            assert!(!metadata.file_type().is_symlink());
            assert_eq!(metadata.uid(), unsafe { libc::geteuid() });
            assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
            assert_eq!(metadata.nlink(), 1);
        }
    }
}

#[test]
fn synthetic_preparation_materializes_and_audits_without_provider_or_auth() {
    let root = tempfile::tempdir().unwrap();
    let protocol_path = root.path().join("protocol.json");
    fs::write(
        &protocol_path,
        serde_json::to_string_pretty(&protocol()).unwrap() + "\n",
    )
    .unwrap();
    let bin = root.path().join("bin");
    fs::create_dir(&bin).unwrap();
    let (engram, codex, claude) = fake_binaries(&bin);
    let output = root.path().join("prepared");
    let command = Command::new(env!("CARGO_BIN_EXE_engram-eval"))
        .args([
            "prepare-native-stale-safety",
            "--protocol",
            protocol_path.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--engram-bin",
            engram.to_str().unwrap(),
            "--codex-bin",
            codex.to_str().unwrap(),
            "--claude-bin",
            claude.to_str().unwrap(),
        ])
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "commit.gpgsign")
        .env("GIT_CONFIG_VALUE_0", "false")
        .output()
        .unwrap();
    assert!(
        command.status.success(),
        "{}",
        String::from_utf8_lossy(&command.stderr)
    );

    let run_plan = output.join("run-plan.json");
    let audit = audit_native_stale_safety_preparation(&protocol_path, &run_plan).unwrap();
    assert!(audit.valid, "{:?}", audit.failures);
    assert!(audit.exact_twelve_lane_matrix);
    assert!(audit.structured_causal_output_schema);
    assert!(audit.shared_retention_precondition_valid);
    assert!(audit.runner_controlled_marker_absence);
    assert!(audit.provider_outputs_absent);
    assert!(audit.auth_caches_absent);
    let outcome_audit = audit_native_stale_safety(&protocol_path, &run_plan).unwrap();
    assert!(!outcome_audit.complete);
    assert!(!outcome_audit.invalid);

    let plan: PreparedNativePilot =
        serde_json::from_reader(fs::File::open(&run_plan).unwrap()).unwrap();
    assert!(!plan.execution_approved);
    assert_eq!(plan.stale_safety.as_ref().unwrap().family_version, 1);
    assert_eq!(plan.lanes.len(), 12);
    assert!(prepare_native_memory_pilot_execution_recovery(
        &run_plan,
        &root.path().join("execution-recovery"),
    )
    .unwrap_err()
    .to_string()
    .contains("forbid lifecycle recovery"));
    assert!(prepare_native_memory_pilot_evaluation_recovery(
        &run_plan,
        &root.path().join("evaluation-recovery"),
        None,
    )
    .unwrap_err()
    .to_string()
    .contains("forbid lifecycle recovery"));
    assert!(plan.lanes.iter().all(|lane| {
        Path::new(&lane.acceptance_contract)
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("lane-") && name.len() == 21)
    }));
    assert!(plan
        .lanes
        .iter()
        .filter(|lane| lane.host == "codex")
        .all(|lane| {
            lane.environment
                .get("CODEX_HOME")
                .is_some_and(|home| !Path::new(home).join("auth.json").exists())
        }));

    let first_lane = &plan.lanes[0];
    let teaching_trace = Path::new(&first_lane.teaching_trace_path);
    let preparation_residue = [
        output.join("runner-evaluation.sha256"),
        teaching_trace.with_extension("stderr.log"),
        teaching_trace
            .parent()
            .unwrap()
            .join("procedure-candidates.json"),
        teaching_trace
            .parent()
            .unwrap()
            .join("cleanup-teaching.stdout.log"),
        PathBuf::from(&first_lane.agent_output_path),
    ];
    for path in preparation_residue {
        fs::write(&path, "synthetic residue\n").unwrap();
        assert!(audit_native_stale_safety_preparation(&protocol_path, &run_plan).is_err());
        fs::remove_file(path).unwrap();
    }
    let dangling_sidecar = output.join("runner-activation.sha256");
    symlink(output.join("missing-runner-activation"), &dangling_sidecar).unwrap();
    assert!(audit_native_stale_safety_preparation(&protocol_path, &run_plan).is_err());
    fs::remove_file(dangling_sidecar).unwrap();

    let phase_span_ms = u64::try_from(plan.lanes.len()).unwrap() * 2 + 1;
    let (phase_started_unix_ms, phase_completed_unix_ms, now_unix_ms) = loop {
        let now_unix_ms = u64::try_from(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        )
        .unwrap();
        let phase_completed_unix_ms = now_unix_ms.saturating_sub(1);
        let phase_started_unix_ms = phase_completed_unix_ms.saturating_sub(phase_span_ms);
        if phase_started_unix_ms >= plan.prepared_unix_ms {
            break (phase_started_unix_ms, phase_completed_unix_ms, now_unix_ms);
        }
        std::thread::yield_now();
    };
    assert!(plan.prepared_unix_ms <= phase_started_unix_ms);
    assert!(phase_started_unix_ms <= phase_completed_unix_ms);
    assert!(phase_completed_unix_ms < now_unix_ms);
    assert!(now_unix_ms.saturating_sub(phase_completed_unix_ms) < 60 * 60 * 1_000);
    let mut teaching_audit =
        engram_eval::native_audit::audit_native_memory_pilot(&run_plan).unwrap();
    teaching_audit.ready_for_evaluation = false;
    teaching_audit.complete = false;
    teaching_audit.all_acceptance_passed = false;
    teaching_audit.invalid = false;
    for (actual, expected) in teaching_audit.lanes.iter_mut().zip(&plan.lanes) {
        actual.phase = if expected.host == "codex" {
            NativeLanePhase::AwaitingActivation
        } else {
            NativeLanePhase::ReadyForEvaluation
        };
        actual.teaching_trace.status = AuditStatus::Passed;
        actual.teaching_trace.sha256 = Some("a".repeat(64));
        actual.teaching_trace.modified_unix_ms = Some(phase_started_unix_ms + 1);
    }
    let teaching_receipt = NativePilotRunReport {
        pilot_id: plan.pilot_id.clone(),
        phase: NativePilotRunPhase::Teaching,
        run_plan: run_plan.canonicalize().unwrap().display().to_string(),
        confirmed_claude_budget_cents: plan.claude_budget_cents,
        started_unix_ms: phase_started_unix_ms,
        completed_unix_ms: phase_completed_unix_ms,
        lanes: plan
            .lanes
            .iter()
            .enumerate()
            .map(|(index, lane)| {
                let provider_started_unix_ms =
                    phase_started_unix_ms + 1 + u64::try_from(index).unwrap() * 2;
                let trace = Path::new(&lane.teaching_trace_path);
                NativePilotLaneExecution {
                    order: lane.order,
                    arm: lane.arm.clone(),
                    host: lane.host.clone(),
                    trace_path: lane.teaching_trace_path.clone(),
                    stderr_path: trace.with_extension("stderr.log").display().to_string(),
                    argv_sha256: sha256(&serde_json::to_vec(&lane.teaching_argv).unwrap()),
                    effective_environment_sha256: None,
                    claude_config_artifacts_sha256: None,
                    stdout_trace_bytes: None,
                    stderr_bytes: None,
                    process_cleanup_proven: None,
                    provider_started_unix_ms: Some(provider_started_unix_ms),
                    provider_completed_unix_ms: Some(provider_started_unix_ms + 1),
                    exit_code: 0,
                    codex_session_rollout_path: None,
                    codex_session_rollout_sha256: None,
                    codex_model_provider: None,
                    codex_host_resolved_model: None,
                    codex_agents_md_sha256: None,
                    provider_trace_sha256: None,
                    claude_requested_model: None,
                    claude_host_resolved_model: None,
                    provider_reported_cost_microusd: None,
                    accepted_turn_boundary_budget_exit: false,
                    recovered_from_existing_trace: false,
                }
            })
            .collect(),
        stale_safety_pre_evaluation_marker_snapshot_sha256: None,
        audit: teaching_audit,
    };
    let teaching_receipt_path = output.join("runner-teaching.json");
    let teaching_receipt_bytes = serde_json::to_string_pretty(&teaching_receipt).unwrap() + "\n";
    fs::write(&teaching_receipt_path, &teaching_receipt_bytes).unwrap();
    fs::write(
        output.join("runner-teaching.sha256"),
        format!("{}\n", sha256(teaching_receipt_bytes.as_bytes())),
    )
    .unwrap();
    let activation = Command::new(env!("CARGO_BIN_EXE_engram-eval"))
        .args([
            "run-native-memory-pilot",
            "--plan",
            run_plan.to_str().unwrap(),
            "--phase",
            "activation",
            "--approve-provider-execution",
            "--confirm-claude-budget-cents",
            &plan.claude_budget_cents.to_string(),
        ])
        .output()
        .unwrap();
    assert!(!activation.status.success());
    let activation_stderr = String::from_utf8_lossy(&activation.stderr);
    assert!(
        activation_stderr.contains("gate has not passed"),
        "unexpected activation stderr: {activation_stderr}"
    );

    let evaluation = Command::new(env!("CARGO_BIN_EXE_engram-eval"))
        .args([
            "run-native-memory-pilot",
            "--plan",
            run_plan.to_str().unwrap(),
            "--phase",
            "evaluation",
            "--approve-provider-execution",
            "--confirm-claude-budget-cents",
            &plan.claude_budget_cents.to_string(),
        ])
        .output()
        .unwrap();
    assert!(!evaluation.status.success());
    let evaluation_stderr = String::from_utf8_lossy(&evaluation.stderr);
    assert!(
        evaluation_stderr.contains("activation receipt"),
        "unexpected evaluation stderr: {evaluation_stderr}"
    );
    let replay = Command::new(env!("CARGO_BIN_EXE_engram-eval"))
        .args([
            "run-native-memory-pilot",
            "--plan",
            run_plan.to_str().unwrap(),
            "--phase",
            "teaching",
            "--approve-provider-execution",
            "--confirm-claude-budget-cents",
            &plan.claude_budget_cents.to_string(),
        ])
        .output()
        .unwrap();
    assert!(!replay.status.success());
    let replay_stderr = String::from_utf8_lossy(&replay.stderr);
    assert!(
        replay_stderr.contains("refusing to overwrite stale-safety teaching phase receipt"),
        "unexpected replay stderr: {replay_stderr}"
    );
    fs::remove_file(teaching_receipt_path).unwrap();
    fs::remove_file(output.join("runner-teaching.sha256")).unwrap();

    let snapshot_path = output.join("stale-safety.protocol.snapshot.json");
    let snapshot_sidecar = output.join("stale-safety.protocol.snapshot.sha256");
    let precondition_path = output.join("stale-safety.retention-precondition.json");
    let precondition_sidecar = output.join("stale-safety.retention-precondition.sha256");
    let schema_path = output.join("agent-output.schema.json");
    let preflight_path = output.join("stale-safety.preflight.json");
    let preflight_sidecar = output.join("stale-safety.preflight.sha256");
    let snapshot_bytes = fs::read(&snapshot_path).unwrap();
    let snapshot_sidecar_bytes = fs::read(&snapshot_sidecar).unwrap();
    let precondition_bytes = fs::read(&precondition_path).unwrap();
    let precondition_sidecar_bytes = fs::read(&precondition_sidecar).unwrap();
    let schema_bytes = fs::read(&schema_path).unwrap();
    let preflight_bytes = fs::read(&preflight_path).unwrap();
    let preflight_sidecar_bytes = fs::read(&preflight_sidecar).unwrap();

    fs::write(&preflight_sidecar, format!("{}\n", "0".repeat(64))).unwrap();
    assert!(
        audit_native_stale_safety_preparation(&protocol_path, &run_plan)
            .unwrap_err()
            .to_string()
            .contains("preflight digest is invalid")
    );
    fs::write(&preflight_sidecar, &preflight_sidecar_bytes).unwrap();

    let mut substituted_preflight: serde_json::Value =
        serde_json::from_slice(&preflight_bytes).unwrap();
    substituted_preflight["valid"] = serde_json::json!(false);
    let substituted_preflight_bytes =
        serde_json::to_string_pretty(&substituted_preflight).unwrap() + "\n";
    fs::write(&preflight_path, &substituted_preflight_bytes).unwrap();
    fs::write(
        &preflight_sidecar,
        format!("{}\n", sha256(substituted_preflight_bytes.as_bytes())),
    )
    .unwrap();
    assert!(
        audit_native_stale_safety_preparation(&protocol_path, &run_plan)
            .unwrap_err()
            .to_string()
            .contains("not a complete passing audit")
    );
    assert!(audit_native_stale_safety(&protocol_path, &run_plan)
        .unwrap_err()
        .to_string()
        .contains("not a complete passing audit"));
    fs::write(&preflight_path, &preflight_bytes).unwrap();
    fs::write(&preflight_sidecar, &preflight_sidecar_bytes).unwrap();

    fs::remove_file(&precondition_path).unwrap();
    assert!(audit_native_stale_safety_preparation(&protocol_path, &run_plan).is_err());
    fs::write(&precondition_path, &precondition_bytes).unwrap();

    fs::remove_file(&snapshot_sidecar).unwrap();
    assert!(audit_native_stale_safety_preparation(&protocol_path, &run_plan).is_err());
    fs::write(&snapshot_sidecar, &snapshot_sidecar_bytes).unwrap();

    fs::write(&precondition_sidecar, format!("{}\n", "0".repeat(64))).unwrap();
    assert!(audit_native_stale_safety_preparation(&protocol_path, &run_plan).is_err());
    fs::write(&precondition_sidecar, &precondition_sidecar_bytes).unwrap();

    let mut substituted_snapshot: NativeStaleSafetyProtocol =
        serde_json::from_slice(&snapshot_bytes).unwrap();
    substituted_snapshot.native_pilot.pilot_id = "different-valid-stale-pilot".to_string();
    fs::write(
        &snapshot_path,
        serde_json::to_string_pretty(&substituted_snapshot).unwrap() + "\n",
    )
    .unwrap();
    assert!(audit_native_stale_safety_preparation(&protocol_path, &run_plan).is_err());
    fs::write(&snapshot_path, &snapshot_bytes).unwrap();

    let mut substituted_precondition: serde_json::Value =
        serde_json::from_slice(&precondition_bytes).unwrap();
    substituted_precondition["shared_retention_gate_hours"] = serde_json::json!(2);
    substituted_precondition["shared_retention_gate_ms"] = serde_json::json!(7_200_000);
    fs::write(
        &precondition_path,
        serde_json::to_string_pretty(&substituted_precondition).unwrap() + "\n",
    )
    .unwrap();
    assert!(audit_native_stale_safety_preparation(&protocol_path, &run_plan).is_err());
    fs::write(&precondition_path, &precondition_bytes).unwrap();

    let mut substituted_schema: serde_json::Value = serde_json::from_slice(&schema_bytes).unwrap();
    substituted_schema["title"] = serde_json::json!("Different valid JSON schema");
    fs::write(
        &schema_path,
        serde_json::to_string_pretty(&substituted_schema).unwrap() + "\n",
    )
    .unwrap();
    assert!(audit_native_stale_safety_preparation(&protocol_path, &run_plan).is_err());
    fs::write(&schema_path, &schema_bytes).unwrap();

    let run_plan_bytes = fs::read(&run_plan).unwrap();
    let run_plan_sidecar = output.join("run-plan.sha256");
    let run_plan_sidecar_bytes = fs::read(&run_plan_sidecar).unwrap();
    let mut unbound_plan: serde_json::Value = serde_json::from_slice(&run_plan_bytes).unwrap();
    unbound_plan.as_object_mut().unwrap().remove("stale_safety");
    let unbound_bytes = serde_json::to_string_pretty(&unbound_plan).unwrap() + "\n";
    fs::write(&run_plan, &unbound_bytes).unwrap();
    fs::write(
        &run_plan_sidecar,
        format!("{}\n", sha256(unbound_bytes.as_bytes())),
    )
    .unwrap();
    assert!(
        audit_native_stale_safety_preparation(&protocol_path, &run_plan)
            .unwrap_err()
            .to_string()
            .contains("without a frozen run-plan binding")
    );

    for path in [
        &snapshot_path,
        &snapshot_sidecar,
        &precondition_path,
        &precondition_sidecar,
        &preflight_path,
        &preflight_sidecar,
    ] {
        fs::remove_file(path).unwrap();
    }
    assert!(
        engram_eval::native_audit::audit_native_memory_pilot(&run_plan)
            .unwrap_err()
            .to_string()
            .contains("plan identity exists without a frozen run-plan binding")
    );
    assert!(
        audit_native_stale_safety_preparation(&protocol_path, &run_plan)
            .unwrap_err()
            .to_string()
            .contains("plan identity exists without a frozen run-plan binding")
    );
    assert!(prepare_native_memory_pilot_execution_recovery(
        &run_plan,
        &root.path().join("stripped-execution-recovery"),
    )
    .unwrap_err()
    .to_string()
    .contains("plan identity exists without a frozen run-plan binding"));
    assert!(prepare_native_memory_pilot_evaluation_recovery(
        &run_plan,
        &root.path().join("stripped-evaluation-recovery"),
        None,
    )
    .unwrap_err()
    .to_string()
    .contains("plan identity exists without a frozen run-plan binding"));

    fs::write(&run_plan, run_plan_bytes).unwrap();
    fs::write(&run_plan_sidecar, run_plan_sidecar_bytes).unwrap();
    fs::write(&snapshot_path, snapshot_bytes).unwrap();
    fs::write(&snapshot_sidecar, snapshot_sidecar_bytes).unwrap();
    fs::write(&precondition_path, precondition_bytes).unwrap();
    fs::write(&precondition_sidecar, precondition_sidecar_bytes).unwrap();
    fs::write(&preflight_path, preflight_bytes).unwrap();
    fs::write(&preflight_sidecar, preflight_sidecar_bytes).unwrap();
}
