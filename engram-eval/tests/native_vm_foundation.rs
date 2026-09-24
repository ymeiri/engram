#[path = "../src/native_vm.rs"]
mod native_vm;

use native_vm::{
    audit_native_vm_collection, audit_native_vm_collection_at, derive_vm_bundle_manifest,
    exact_colima_start_argv, execute_native_vm_collection,
    exercise_process_group_guard_failure_for_test, finalize_native_vm_contract,
    native_vm_contract_sha256, prepare_native_vm_collection, prepare_native_vm_collection_at,
    validate_native_vm_contract, validate_ustar_archive, NativeVmArchivePin,
    NativeVmCollectionContract, NativeVmCollectionReceipt, NativeVmCommandInput,
    NativeVmCommandReceipt, NativeVmExecutablePin, NativeVmRunnerTestFault,
    NATIVE_VM_SCHEMA_VERSION, REQUIRED_COLIMA_COMMIT, REQUIRED_COLIMA_VERSION,
    REQUIRED_LIMACTL_VERSION,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};
use tempfile::TempDir;

#[cfg(unix)]
use std::os::unix::fs::{symlink, MetadataExt, OpenOptionsExt, PermissionsExt};

const NOW: u64 = 1_800_000_000_000;

struct ContractFixture {
    _temporary: TempDir,
    contract: NativeVmCollectionContract,
    run_root: PathBuf,
    profile_config: PathBuf,
    bundle_root: PathBuf,
}

fn sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn file_sha256(path: &Path) -> String {
    sha256(&fs::read(path).unwrap())
}

fn contract_digest(contract: &NativeVmCollectionContract) -> String {
    native_vm_contract_sha256(contract).unwrap()
}

fn live_identity_sha256(contract: &NativeVmCollectionContract) -> (String, String) {
    let boot_sha256 = sha256(b"01234567-89ab-cdef-8123-456789abcdef");
    let identity = format!(
        "profile={}\ndisplay_name={}\ndriver=macOS Virtualization.Framework\narch=aarch64\nruntime=containerd\ndocker_socket=\nkubernetes=false\ndisk={}\nguest_boot_id_sha256={}\n",
        contract.profile_name,
        contract.profile_name,
        u64::from(contract.disk_gib) * 1024 * 1024 * 1024,
        boot_sha256,
    );
    (boot_sha256, sha256(identity.as_bytes()))
}

fn audit_fixture(fixture: &ContractFixture, now: u64) -> native_vm::NativeVmStructuralAudit {
    audit_native_vm_collection_at(
        &fixture.contract,
        &contract_digest(&fixture.contract),
        &fixture.run_root,
        now,
    )
    .unwrap()
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) {
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
}

#[cfg(unix)]
fn write_mode(path: &Path, bytes: &[u8], mode: u32) {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true).mode(mode);
    let mut file = options.open(path).unwrap();
    file.write_all(bytes).unwrap();
    file.sync_all().unwrap();
}

fn digest_pin(path: &str, name: &str, sha: String) -> NativeVmExecutablePin {
    let version = format!("{name} version frozen-1\n");
    NativeVmExecutablePin {
        path: path.to_string(),
        sha256: sha,
        version_argv: vec![path.to_string(), "--version".to_string()],
        version_output_sha256: sha256(version.as_bytes()),
        version_marker: format!("{name} version frozen-1"),
    }
}

#[cfg(unix)]
fn contract_fixture() -> ContractFixture {
    let temporary = tempfile::tempdir().unwrap();
    set_mode(temporary.path(), 0o700);
    let temporary_root = fs::canonicalize(temporary.path()).unwrap();
    let host_home = temporary_root.join("host");
    fs::create_dir(&host_home).unwrap();
    set_mode(&host_home, 0o700);
    let colima_home = host_home.join(".colima");
    fs::create_dir(&colima_home).unwrap();
    set_mode(&colima_home, 0o700);
    let profile_name = "engram-vm-foundation-01";
    let profile_directory = colima_home.join(profile_name);
    let profile_config = profile_directory.join("colima.yaml");

    let colima = temporary_root.join("colima");
    write_mode(&colima, b"frozen-colima-binary", 0o500);
    let colima_version =
        format!("colima version {REQUIRED_COLIMA_VERSION}\ngit commit: {REQUIRED_COLIMA_COMMIT}\n");
    let colima_pin = NativeVmExecutablePin {
        path: colima.display().to_string(),
        sha256: file_sha256(&colima),
        version_argv: vec!["version".to_string()],
        version_output_sha256: sha256(colima_version.as_bytes()),
        version_marker: format!(
            "colima version {REQUIRED_COLIMA_VERSION}\ngit commit: {REQUIRED_COLIMA_COMMIT}"
        ),
    };
    let limactl = temporary_root.join("limactl");
    write_mode(&limactl, b"frozen-limactl-binary", 0o500);
    let limactl_version = format!("limactl version {REQUIRED_LIMACTL_VERSION}\n");
    let limactl_pin = NativeVmExecutablePin {
        path: limactl.display().to_string(),
        sha256: file_sha256(&limactl),
        version_argv: vec!["--version".to_string()],
        version_output_sha256: sha256(limactl_version.as_bytes()),
        version_marker: format!("limactl version {REQUIRED_LIMACTL_VERSION}"),
    };

    let bundle_root = temporary_root.join("vm-bundle");
    fs::create_dir(&bundle_root).unwrap();
    set_mode(&bundle_root, 0o700);
    let bin = bundle_root.join("bin");
    fs::create_dir(&bin).unwrap();
    set_mode(&bin, 0o700);
    let runtime_names = [
        "cargo",
        "claude",
        "codex",
        "engram",
        "engram_eval",
        "node",
        "rustc",
    ];
    for name in runtime_names {
        write_mode(
            &bin.join(name),
            format!("frozen-runtime-{name}").as_bytes(),
            0o555,
        );
    }
    let manifest = derive_vm_bundle_manifest(&bundle_root).unwrap();

    let archive = temporary_root.join("vm-bundle.tar");
    let mut archive_command = Command::new("/usr/bin/tar");
    archive_command
        .arg("--format")
        .arg("ustar")
        .arg("-cf")
        .arg(&archive)
        .arg("-C")
        .arg(&bundle_root);
    for entry in &manifest.entries {
        archive_command.arg(&entry.relative_path);
    }
    assert!(archive_command.status().unwrap().success());
    set_mode(&archive, 0o600);

    let collection_id = "foundation-collection-01";
    let guest_root = format!("/var/lib/engram-eval/collections/{collection_id}/bundle");
    let required_runtimes = runtime_names
        .iter()
        .map(|name| {
            let relative = format!("bin/{name}");
            let entry = manifest
                .entries
                .iter()
                .find(|entry| entry.relative_path == relative)
                .unwrap();
            let path = format!("{guest_root}/{relative}");
            (
                (*name).to_string(),
                digest_pin(&path, name, entry.sha256.clone()),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let tool_names = [
        "aa_status",
        "bwrap",
        "cat",
        "env",
        "find",
        "findmnt",
        "getent",
        "id",
        "install",
        "openssl",
        "printenv",
        "readlink",
        "sha256sum",
        "stat",
        "sudo",
        "tar",
        "test",
        "uname",
        "useradd",
    ];
    let collector_tools = tool_names
        .iter()
        .map(|name| {
            let path = match *name {
                "aa_status" => "/usr/sbin/aa-status".to_string(),
                "useradd" => "/usr/sbin/useradd".to_string(),
                _ => format!("/usr/bin/{name}"),
            };
            let hash = sha256(format!("frozen-tool-{name}").as_bytes());
            let mut pin = digest_pin(&path, name, hash);
            if *name == "openssl" {
                pin.version_argv = vec![path.clone(), "version".to_string()];
            }
            ((*name).to_string(), pin)
        })
        .collect::<BTreeMap<_, _>>();
    let contract = NativeVmCollectionContract {
        schema_version: NATIVE_VM_SCHEMA_VERSION,
        collection_id: collection_id.to_string(),
        created_unix_ms: NOW - 1_000,
        expires_unix_ms: NOW + 60 * 60 * 1_000,
        profile_name: profile_name.to_string(),
        host_home: host_home.display().to_string(),
        colima_home: colima_home.display().to_string(),
        profile_config_path: profile_config.display().to_string(),
        colima: colima_pin,
        limactl: limactl_pin,
        vm_type: "vz".to_string(),
        architecture: "aarch64".to_string(),
        mount_mode: "none".to_string(),
        runtime: "containerd".to_string(),
        cpus: 4,
        memory_gib: 8,
        disk_gib: 40,
        root_disk_gib: 40,
        guest_bundle_root: guest_root,
        teaching_user: "engram-teach".to_string(),
        teaching_uid: 12_001,
        evaluation_user: "engram-eval".to_string(),
        evaluation_uid: 12_002,
        bundle_manifest: manifest,
        bundle_archive: NativeVmArchivePin {
            path: archive.display().to_string(),
            sha256: file_sha256(&archive),
            size_bytes: fs::metadata(&archive).unwrap().len(),
        },
        required_runtimes,
        collector_tools,
        command_plan: Vec::new(),
        command_plan_sha256: String::new(),
        provider_free: true,
        account_policy_nonclaim: "Claude managed policy remains account-scoped and may influence both hosts; this same-account VM replication does not isolate or attribute that influence.".to_string(),
    };
    let contract = finalize_native_vm_contract(contract).unwrap();
    ContractFixture {
        run_root: temporary_root.join("run"),
        _temporary: temporary,
        contract,
        profile_config,
        bundle_root,
    }
}

fn profile_config(contract: &NativeVmCollectionContract) -> String {
    format!(
        "cpu: {}\ndisk: {}\nmemory: {}\narch: aarch64\nruntime: containerd\nhostname: {}\nautoActivate: false\nnetwork:\n  address: false\n  mode: shared\n  preferredRoute: false\n  hostAddresses: false\nkubernetes:\n  enabled: false\nforwardAgent: false\nvmType: vz\nportForwarder: none\nrosetta: false\nbinfmt: false\nsshConfig: false\nmountInotify: false\nmounts: []\nprovision: null\nrootDisk: {}\nenv: {{}}\n",
        contract.cpus,
        contract.disk_gib,
        contract.memory_gib,
        contract.profile_name,
        contract.root_disk_gib
    )
}

fn expected_output(contract: &NativeVmCollectionContract, label: &str) -> Vec<u8> {
    let colima_version =
        format!("colima version {REQUIRED_COLIMA_VERSION}\ngit commit: {REQUIRED_COLIMA_COMMIT}\n");
    match label {
        "host_colima_version" => return colima_version.into_bytes(),
        "host_limactl_version" => {
            return format!("limactl version {REQUIRED_LIMACTL_VERSION}\n").into_bytes()
        }
        "host_profiles_before" => {
            return b"{\"name\":\"default\",\"status\":\"Stopped\"}\n".to_vec()
        }
        "host_colima_status" | "host_colima_status_after" => {
            return format!(
                "{{\"display_name\":\"{}\",\"driver\":\"macOS Virtualization.Framework\",\"arch\":\"aarch64\",\"runtime\":\"containerd\",\"docker_socket\":\"\",\"kubernetes\":false,\"disk\":{}}}\n",
                contract.profile_name,
                u64::from(contract.disk_gib) * 1024 * 1024 * 1024
            )
            .into_bytes()
        }
        "guest_uname" => return b"Linux aarch64\n".to_vec(),
        "guest_boot_id" | "guest_boot_id_after" => {
            return b"01234567-89ab-cdef-8123-456789abcdef\n".to_vec()
        }
        "guest_os_release" => return b"ID=ubuntu\nVERSION_ID=\"24.04\"\n".to_vec(),
        "guest_teaching_user" => {
            return format!(
                "{}:x:{}:{}::/home/{}:/bin/bash\n",
                contract.teaching_user,
                contract.teaching_uid,
                contract.teaching_uid,
                contract.teaching_user
            )
            .into_bytes()
        }
        "guest_evaluation_user" => {
            return format!(
                "{}:x:{}:{}::/home/{}:/bin/bash\n",
                contract.evaluation_user,
                contract.evaluation_uid,
                contract.evaluation_uid,
                contract.evaluation_user
            )
            .into_bytes()
        }
        "guest_teaching_identity" => {
            return format!("{}\n", contract.teaching_uid).into_bytes()
        }
        "guest_evaluation_identity" => {
            return format!("{}\n", contract.evaluation_uid).into_bytes()
        }
        "guest_teaching_home_stat" => {
            return format!(
                "directory|700|{}|{}|/home/{}\n",
                contract.teaching_uid, contract.teaching_uid, contract.teaching_user
            )
            .into_bytes()
        }
        "guest_evaluation_home_stat" => {
            return format!(
                "directory|700|{}|{}|/home/{}\n",
                contract.evaluation_uid, contract.evaluation_uid, contract.evaluation_user
            )
            .into_bytes()
        }
        "guest_findmnt" => {
            return b"{\"filesystems\":[{\"source\":\"/dev/vda1\",\"target\":\"/\",\"fstype\":\"ext4\",\"options\":\"rw,relatime\",\"children\":[{\"source\":\"proc\",\"target\":\"/proc\",\"fstype\":\"proc\",\"options\":\"rw\"}]}]}\n".to_vec()
        }
        "guest_sockets" => return b"/run/systemd/private\n".to_vec(),
        "guest_binfmt_entries" => return b"register\nstatus\n".to_vec(),
        "guest_apparmor_enabled" => return b"Y\n".to_vec(),
        "guest_bwrap_profile" => return b"bwrap-default (enforce)\n".to_vec(),
        "guest_bundle_filesystem" => return b"/dev/vda1 / ext4\n".to_vec(),
        "guest_root_filesystem_bytes" => {
            return format!("{}\n", u64::from(contract.root_disk_gib) * 1024 * 1024 * 1024)
                .into_bytes()
        }
        "guest_bundle_inventory" => {
            let mut rows = String::new();
            for entry in &contract.bundle_manifest.entries {
                rows.push_str(&format!(
                    "{:o}|1|0|0|{}|{}\n",
                    entry.mode, entry.size_bytes, entry.relative_path
                ));
            }
            return rows.into_bytes();
        }
        "guest_bundle_rehash" => {
            let mut rows = String::new();
            for entry in &contract.bundle_manifest.entries {
                rows.push_str(&format!(
                    "{}  {}/{}\n",
                    entry.sha256, contract.guest_bundle_root, entry.relative_path
                ));
            }
            return rows.into_bytes();
        }
        "guest_bundle_directories" => {
            let mut paths = std::collections::BTreeSet::from([String::new()]);
            for entry in &contract.bundle_manifest.entries {
                let mut parent = Path::new(&entry.relative_path).parent();
                while let Some(path) = parent {
                    if path.as_os_str().is_empty() {
                        break;
                    }
                    paths.insert(path.to_str().unwrap().to_string());
                    parent = path.parent();
                }
            }
            return paths
                .into_iter()
                .map(|path| format!("755|0|0|{path}\n"))
                .collect::<String>()
                .into_bytes();
        }
        "guest_sha256sum_crosscheck" => {
            let pin = &contract.collector_tools["sha256sum"];
            return format!("SHA2-256({})= {}\n", pin.path, pin.sha256).into_bytes();
        }
        "guest_openssl_crosscheck" => {
            let pin = &contract.collector_tools["openssl"];
            return format!("{}  {}\n", pin.sha256, pin.path).into_bytes();
        }
        _ => {}
    }
    for (namespace, pins) in [
        ("runtime", &contract.required_runtimes),
        ("tool", &contract.collector_tools),
    ] {
        for (name, pin) in pins {
            for suffix in ["path", "stat", "hash", "hash_alt", "version"] {
                if label != format!("guest_{namespace}_{name}_{suffix}") {
                    continue;
                }
                return match suffix {
                    "path" => format!("{}\n", pin.path).into_bytes(),
                    "stat" => format!("regular file|1|555|0|0|19|{}\n", pin.path).into_bytes(),
                    "hash" => format!("{}  {}\n", pin.sha256, pin.path).into_bytes(),
                    "hash_alt" => format!("SHA2-256({})= {}\n", pin.path, pin.sha256).into_bytes(),
                    "version" => format!("{name} version frozen-1\n").into_bytes(),
                    _ => unreachable!(),
                };
            }
        }
    }
    Vec::new()
}

#[cfg(unix)]
fn write_private(path: &Path, bytes: &[u8]) {
    write_mode(path, bytes, 0o600);
}

#[cfg(unix)]
fn write_json_digest<T: Serialize>(path: &Path, value: &T) {
    let mut bytes = serde_json::to_vec_pretty(value).unwrap();
    bytes.push(b'\n');
    write_private(path, &bytes);
    write_private(
        &path.with_file_name(format!(
            "{}.sha256",
            path.file_name().unwrap().to_str().unwrap()
        )),
        format!("{}\n", sha256(&bytes)).as_bytes(),
    );
}

#[cfg(unix)]
fn rewrite_json_digest<T: Serialize>(path: &Path, value: &T) {
    let mut bytes = serde_json::to_vec_pretty(value).unwrap();
    bytes.push(b'\n');
    fs::write(path, &bytes).unwrap();
    fs::write(
        path.with_file_name(format!(
            "{}.sha256",
            path.file_name().unwrap().to_str().unwrap()
        )),
        format!("{}\n", sha256(&bytes)),
    )
    .unwrap();
}

#[cfg(unix)]
fn forge_raw_and_rebind_receipt(fixture: &ContractFixture, label: &str, bytes: &[u8]) {
    let receipt_path = fixture.run_root.join("receipt.json");
    let mut receipt: NativeVmCollectionReceipt =
        serde_json::from_slice(&fs::read(&receipt_path).unwrap()).unwrap();
    let command = receipt
        .commands
        .iter_mut()
        .find(|command| command.label == label)
        .unwrap();
    fs::write(fixture.run_root.join(&command.stdout_path), bytes).unwrap();
    command.stdout_sha256 = sha256(bytes);
    command.stdout_bytes = bytes.len() as u64;
    receipt.command_transcript_sha256 = sha256(&serde_json::to_vec(&receipt.commands).unwrap());
    rewrite_json_digest(&receipt_path, &receipt);
}

#[cfg(unix)]
fn completed_fixture() -> ContractFixture {
    let fixture = contract_fixture();
    let prepared = prepare_native_vm_collection_at(
        &fixture.contract,
        &contract_digest(&fixture.contract),
        &fixture.run_root,
        NOW,
    )
    .unwrap();
    fs::create_dir(fixture.profile_config.parent().unwrap()).unwrap();
    set_mode(fixture.profile_config.parent().unwrap(), 0o700);
    let profile = profile_config(&fixture.contract);
    write_mode(&fixture.profile_config, profile.as_bytes(), 0o600);
    write_private(
        &fixture.run_root.join("profile-config.yaml"),
        profile.as_bytes(),
    );
    write_private(
        &fixture.run_root.join("profile-config.yaml.sha256"),
        format!("{}\n", sha256(profile.as_bytes())).as_bytes(),
    );

    let mut receipts = Vec::new();
    for (index, spec) in fixture.contract.command_plan.iter().enumerate() {
        let stem = format!("{:03}-{}", spec.ordinal, spec.label);
        let stdout_relative = format!("raw/{stem}.stdout");
        let stderr_relative = format!("raw/{stem}.stderr");
        let stdout = expected_output(&fixture.contract, &spec.label);
        write_private(&fixture.run_root.join(&stdout_relative), &stdout);
        write_private(&fixture.run_root.join(&stderr_relative), b"");
        let started = NOW + 10 + index as u64;
        receipts.push(NativeVmCommandReceipt {
            ordinal: spec.ordinal,
            label: spec.label.clone(),
            command_sha256: spec.command_sha256.clone(),
            expected_exit_code: spec.expected_exit_code,
            exit_code: spec.expected_exit_code,
            started_unix_ms: started,
            completed_unix_ms: started + 1,
            stdout_path: stdout_relative,
            stdout_sha256: sha256(&stdout),
            stdout_bytes: stdout.len() as u64,
            stderr_path: stderr_relative,
            stderr_sha256: sha256(b""),
            stderr_bytes: 0,
            stdin_sha256: (spec.stdin == NativeVmCommandInput::BundleArchive)
                .then(|| fixture.contract.bundle_archive.sha256.clone()),
            stdin_bytes: (spec.stdin == NativeVmCommandInput::BundleArchive)
                .then_some(fixture.contract.bundle_archive.size_bytes),
            timed_out: false,
            output_overflow: false,
        });
    }
    let canonical_run_root = fs::canonicalize(&fixture.run_root).unwrap();
    let run_root_metadata = fs::symlink_metadata(&canonical_run_root).unwrap();
    let raw_root_metadata = fs::symlink_metadata(canonical_run_root.join("raw")).unwrap();
    let (guest_boot_id_sha256, live_identity_sha256) = live_identity_sha256(&fixture.contract);
    let command_transcript_sha256 = sha256(&serde_json::to_vec(&receipts).unwrap());
    let receipt = NativeVmCollectionReceipt {
        schema_version: NATIVE_VM_SCHEMA_VERSION,
        collection_id: fixture.contract.collection_id.clone(),
        contract_sha256: native_vm_contract_sha256(&fixture.contract).unwrap(),
        intent_sha256: prepared.intent_sha256,
        command_plan_sha256: fixture.contract.command_plan_sha256.clone(),
        bundle_manifest_sha256: fixture.contract.bundle_manifest.manifest_sha256.clone(),
        bundle_archive_sha256: fixture.contract.bundle_archive.sha256.clone(),
        profile_config_sha256: sha256(profile.as_bytes()),
        canonical_run_root: canonical_run_root.display().to_string(),
        run_root_device: run_root_metadata.dev(),
        run_root_inode: run_root_metadata.ino(),
        raw_root_device: raw_root_metadata.dev(),
        raw_root_inode: raw_root_metadata.ino(),
        profile_name: fixture.contract.profile_name.clone(),
        guest_boot_id_sha256,
        live_identity_sha256,
        command_transcript_sha256,
        started_unix_ms: NOW + 1,
        completed_unix_ms: NOW + 1_000,
        collector_pid: 42,
        state: "complete".to_string(),
        no_replay: true,
        commands: receipts,
    };
    write_json_digest(&fixture.run_root.join("receipt.json"), &receipt);
    fixture
}

#[cfg(unix)]
#[test]
fn exact_colima_0101_start_disables_every_host_bridge_and_default() {
    let fixture = contract_fixture();
    let _provider_free_executor: fn(
        &NativeVmCollectionContract,
        &str,
        &Path,
    ) -> native_vm::NativeVmResult<
        native_vm::RunnerObservedNativeVmCollection,
    > = execute_native_vm_collection;
    let _structural_audit_accessor: fn(
        &native_vm::RunnerObservedNativeVmCollection,
    ) -> &native_vm::NativeVmStructuralAudit =
        native_vm::RunnerObservedNativeVmCollection::structural_audit;
    let _witness_consumer: fn(
        native_vm::RunnerObservedNativeVmCollection,
    ) -> native_vm::NativeVmExecutionWitness =
        native_vm::RunnerObservedNativeVmCollection::into_execution_witness;
    let _production_preparer: fn(
        &NativeVmCollectionContract,
        &str,
        &Path,
    ) -> native_vm::NativeVmResult<
        native_vm::PreparedNativeVmCollection,
    > = prepare_native_vm_collection;
    let _production_auditor: fn(
        &NativeVmCollectionContract,
        &str,
        &Path,
    )
        -> native_vm::NativeVmResult<native_vm::NativeVmStructuralAudit> =
        audit_native_vm_collection;
    let argv = exact_colima_start_argv(&fixture.contract);
    for required in [
        "--mount-inotify=false",
        "--vz-rosetta=false",
        "--binfmt=false",
        "--ssh-agent=false",
        "--activate=false",
        "--ssh-config=false",
        "--network-address=false",
        "--network-host-addresses=false",
        "--network-preferred-route=false",
        "--template=false",
    ] {
        assert_eq!(
            argv.iter()
                .filter(|value| value.as_str() == required)
                .count(),
            1
        );
    }
    assert!(argv.windows(2).any(|pair| pair == ["--mount", "none"]));
    assert!(argv
        .windows(2)
        .any(|pair| pair == ["--port-forwarder", "none"]));
    assert!(argv
        .windows(2)
        .any(|pair| pair == ["--runtime", "containerd"]));
    assert!(!argv.iter().any(|value| value == "default"));
}

#[cfg(unix)]
#[test]
fn bundle_manifest_rejects_links_permissive_modes_and_authentication_paths() {
    let temporary = tempfile::tempdir().unwrap();
    set_mode(temporary.path(), 0o700);
    let temporary_root = fs::canonicalize(temporary.path()).unwrap();
    let safe = temporary_root.join("safe");
    fs::create_dir(&safe).unwrap();
    set_mode(&safe, 0o700);
    write_mode(&safe.join("binary"), b"safe", 0o500);
    assert!(derive_vm_bundle_manifest(&safe).is_ok());

    let linked = temporary_root.join("linked");
    fs::create_dir(&linked).unwrap();
    set_mode(&linked, 0o700);
    write_mode(&linked.join("one"), b"same inode", 0o500);
    fs::hard_link(linked.join("one"), linked.join("two")).unwrap();
    assert!(derive_vm_bundle_manifest(&linked).is_err());

    let symlinked = temporary_root.join("symlinked");
    fs::create_dir(&symlinked).unwrap();
    set_mode(&symlinked, 0o700);
    symlink(safe.join("binary"), symlinked.join("binary")).unwrap();
    assert!(derive_vm_bundle_manifest(&symlinked).is_err());

    let permissive = temporary_root.join("permissive");
    fs::create_dir(&permissive).unwrap();
    set_mode(&permissive, 0o700);
    write_mode(&permissive.join("binary"), b"unsafe", 0o522);
    set_mode(&permissive.join("binary"), 0o522);
    assert!(derive_vm_bundle_manifest(&permissive).is_err());

    let authentication = temporary_root.join("authentication");
    fs::create_dir(&authentication).unwrap();
    set_mode(&authentication, 0o700);
    write_mode(&authentication.join("auth.json"), b"never read", 0o400);
    assert!(derive_vm_bundle_manifest(&authentication).is_err());
}

#[cfg(unix)]
#[test]
fn contract_rejects_default_profile_omitted_mount_and_unbound_runtime() {
    let fixture = contract_fixture();
    let mut bad = fixture.contract.clone();
    bad.profile_name = "default".to_string();
    assert!(validate_native_vm_contract(&bad).is_err());

    let mut bad = fixture.contract.clone();
    bad.mount_mode = "".to_string();
    assert!(validate_native_vm_contract(&bad).is_err());

    let mut bad = fixture.contract.clone();
    bad.required_runtimes.get_mut("codex").unwrap().sha256 = "f".repeat(64);
    assert!(validate_native_vm_contract(&bad).is_err());

    let mut bad = fixture.contract.clone();
    let duplicate_hash = bad.required_runtimes["claude"].sha256.clone();
    bad.required_runtimes.get_mut("codex").unwrap().sha256 = duplicate_hash;
    assert!(validate_native_vm_contract(&bad).is_err());

    let mut bad = fixture.contract.clone();
    bad.command_plan[0].argv.push("--verbose".to_string());
    assert!(validate_native_vm_contract(&bad).is_err());
}

#[cfg(unix)]
#[test]
fn preparation_is_create_new_fsynced_and_refuses_replay() {
    let fixture = contract_fixture();
    let digest = contract_digest(&fixture.contract);
    let auth_like = Path::new(&fixture.contract.host_home).join("auth.json");
    write_mode(&auth_like, b"synthetic forbidden fixture", 0o400);
    assert!(
        prepare_native_vm_collection_at(&fixture.contract, &digest, &fixture.run_root, NOW)
            .is_err()
    );
    assert!(!fixture.run_root.exists());
    fs::remove_file(auth_like).unwrap();
    assert!(prepare_native_vm_collection_at(
        &fixture.contract,
        &"f".repeat(64),
        &fixture.run_root,
        NOW
    )
    .is_err());
    assert!(!fixture.run_root.exists());
    let first = prepare_native_vm_collection_at(&fixture.contract, &digest, &fixture.run_root, NOW)
        .unwrap();
    let intent_text = fs::read_to_string(&first.intent_path).unwrap();
    assert!(!intent_text.contains("authorizes_provider_execution"));
    assert!(fixture.run_root.join("intent.json").is_file());
    assert!(fixture.run_root.join("intent.json.sha256").is_file());
    assert!(
        prepare_native_vm_collection_at(&fixture.contract, &digest, &fixture.run_root, NOW)
            .is_err()
    );
}

#[cfg(unix)]
#[test]
fn standalone_artifacts_are_consistency_only_and_never_execution_authority() {
    let fixture = completed_fixture();
    let audit = audit_fixture(&fixture, NOW + 2_000);
    assert!(audit.artifacts_consistent, "{:?}", audit.failures);
    assert_eq!(
        audit.claim,
        "structural artifact consistency only; no causal provenance or provider authority"
    );
    assert!(audit.account_policy_nonclaim.contains("account-scoped"));
    assert!(!audit
        .remaining_conditions_before_provider_execution
        .is_empty());
    let serialized = serde_json::to_string(&audit).unwrap();
    assert!(!serialized.contains("authorizes_provider_execution"));
    assert!(!serialized.contains("\"valid\""));
    assert!(serialized.contains("structural artifact consistency only"));
}

#[cfg(unix)]
#[test]
fn audit_rejects_stale_receipt_extra_residue_symlink_and_hardlink() {
    let fixture = completed_fixture();
    assert!(!audit_fixture(&fixture, fixture.contract.expires_unix_ms + 1).artifacts_consistent);

    let fixture = completed_fixture();
    write_private(
        &Path::new(&fixture.contract.host_home).join("unexpected"),
        b"host-home residue",
    );
    assert!(!audit_fixture(&fixture, NOW + 2_000).artifacts_consistent);

    let fixture = completed_fixture();
    write_private(&fixture.run_root.join("unexpected"), b"residue");
    assert!(!audit_fixture(&fixture, NOW + 2_000).artifacts_consistent);

    let fixture = completed_fixture();
    let raw = fixture.run_root.join(format!(
        "raw/{:03}-{}.stdout",
        fixture.contract.command_plan[0].ordinal, fixture.contract.command_plan[0].label
    ));
    let original = fs::read(&raw).unwrap();
    fs::remove_file(&raw).unwrap();
    let target = fixture.run_root.join(format!(
        "raw/{:03}-{}.stdout",
        fixture.contract.command_plan[1].ordinal, fixture.contract.command_plan[1].label
    ));
    fs::write(&target, &original).unwrap();
    symlink(&target, &raw).unwrap();
    assert!(!audit_fixture(&fixture, NOW + 2_000).artifacts_consistent);

    let fixture = completed_fixture();
    let raw = fixture.run_root.join(format!(
        "raw/{:03}-{}.stdout",
        fixture.contract.command_plan[0].ordinal, fixture.contract.command_plan[0].label
    ));
    let second = fixture.run_root.join(format!(
        "raw/{:03}-{}.stdout",
        fixture.contract.command_plan[1].ordinal, fixture.contract.command_plan[1].label
    ));
    fs::remove_file(&second).unwrap();
    fs::hard_link(&raw, &second).unwrap();
    assert!(!audit_fixture(&fixture, NOW + 2_000).artifacts_consistent);
}

#[cfg(unix)]
#[test]
fn semantic_forgery_and_single_hash_self_attestation_are_rejected() {
    let fixture = completed_fixture();
    let forged_status = b"{\"display_name\":\"forged\",\"driver\":\"qemu\",\"arch\":\"aarch64\",\"runtime\":\"containerd\",\"disk\":42949672960}\n";
    forge_raw_and_rebind_receipt(&fixture, "host_colima_status", forged_status);
    assert!(!audit_fixture(&fixture, NOW + 2_000).artifacts_consistent);

    let fixture = completed_fixture();
    let label = "guest_runtime_codex_hash_alt";
    let command = fixture
        .contract
        .command_plan
        .iter()
        .find(|command| command.label == label)
        .unwrap();
    let forged_hash = format!(
        "SHA2-256({})= {}\n",
        fixture.contract.required_runtimes["codex"].path,
        "f".repeat(64)
    );
    assert!(fixture
        .run_root
        .join(format!("raw/{:03}-{label}.stdout", command.ordinal))
        .is_file());
    forge_raw_and_rebind_receipt(&fixture, label, forged_hash.as_bytes());
    assert!(!audit_fixture(&fixture, NOW + 2_000).artifacts_consistent);

    let fixture = completed_fixture();
    let pin = &fixture.contract.required_runtimes["codex"];
    let forged_group = format!("regular file|1|555|0|17|19|{}\n", pin.path);
    forge_raw_and_rebind_receipt(
        &fixture,
        "guest_runtime_codex_stat",
        forged_group.as_bytes(),
    );
    assert!(!audit_fixture(&fixture, NOW + 2_000).artifacts_consistent);

    let fixture = completed_fixture();
    forge_raw_and_rebind_receipt(
        &fixture,
        "guest_bundle_filesystem",
        b"/dev/vdb1 /mnt/host ext4\n",
    );
    assert!(!audit_fixture(&fixture, NOW + 2_000).artifacts_consistent);

    let fixture = completed_fixture();
    forge_raw_and_rebind_receipt(&fixture, "guest_teaching_identity", b"12001 27\n");
    assert!(!audit_fixture(&fixture, NOW + 2_000).artifacts_consistent);
}

#[cfg(unix)]
#[test]
fn strict_ustar_scan_rejects_an_extra_file_before_streaming() {
    let fixture = contract_fixture();
    assert!(validate_ustar_archive(
        Path::new(&fixture.contract.bundle_archive.path),
        &fixture.contract.bundle_manifest
    )
    .is_ok());
    write_mode(
        &fixture.bundle_root.join("extra"),
        b"not in manifest",
        0o400,
    );
    let archive = fixture.bundle_root.parent().unwrap().join("extra.tar");
    let mut command = Command::new("/usr/bin/tar");
    command
        .arg("--format")
        .arg("ustar")
        .arg("-cf")
        .arg(&archive)
        .arg("-C")
        .arg(&fixture.bundle_root);
    for entry in &fixture.contract.bundle_manifest.entries {
        command.arg(&entry.relative_path);
    }
    command.arg("extra");
    assert!(command.status().unwrap().success());
    set_mode(&archive, 0o600);
    assert!(validate_ustar_archive(&archive, &fixture.contract.bundle_manifest).is_err());

    let extra_directory = fixture.bundle_root.join("empty-extra-directory");
    fs::create_dir(&extra_directory).unwrap();
    set_mode(&extra_directory, 0o700);
    let archive = fixture.bundle_root.parent().unwrap().join("extra-dir.tar");
    let mut command = Command::new("/usr/bin/tar");
    command
        .arg("--format")
        .arg("ustar")
        .arg("-cf")
        .arg(&archive)
        .arg("-C")
        .arg(&fixture.bundle_root);
    for entry in &fixture.contract.bundle_manifest.entries {
        command.arg(&entry.relative_path);
    }
    command.arg("empty-extra-directory");
    assert!(command.status().unwrap().success());
    set_mode(&archive, 0o600);
    assert!(validate_ustar_archive(&archive, &fixture.contract.bundle_manifest).is_err());
}

#[cfg(unix)]
#[test]
fn credential_paths_and_aliases_fail_before_any_content_hash() {
    let temporary = tempfile::tempdir().unwrap();
    set_mode(temporary.path(), 0o700);
    let root = fs::canonicalize(temporary.path()).unwrap();
    let bundle = root.join("bundle");
    fs::create_dir(&bundle).unwrap();
    set_mode(&bundle, 0o700);
    let credential = bundle.join(".credentials.json");
    write_mode(&credential, b"synthetic-never-opened", 0o000);
    let error = derive_vm_bundle_manifest(&bundle).unwrap_err().to_string();
    assert!(error.contains("authentication-like"), "{error}");

    let fixture = contract_fixture();
    let outside_credential = fixture
        .bundle_root
        .parent()
        .unwrap()
        .join(".credentials.json");
    write_mode(&outside_credential, b"synthetic-never-opened", 0o000);

    let alias_directory = fixture.bundle_root.parent().unwrap().join("host-alias");
    fs::create_dir(&alias_directory).unwrap();
    set_mode(&alias_directory, 0o700);
    let colima_alias = alias_directory.join("colima");
    symlink(&outside_credential, &colima_alias).unwrap();
    let mut bad = fixture.contract.clone();
    bad.colima.path = colima_alias.display().to_string();
    let error = validate_native_vm_contract(&bad).unwrap_err().to_string();
    assert!(error.contains("authentication-") || error.contains("symbolic-link"));

    let mut bad = fixture.contract.clone();
    bad.bundle_archive.path = outside_credential.display().to_string();
    let error = validate_native_vm_contract(&bad).unwrap_err().to_string();
    assert!(error.contains("authentication-"), "{error}");

    let fixture = completed_fixture();
    let credential = fixture
        .bundle_root
        .parent()
        .unwrap()
        .join(".credentials.json");
    write_mode(&credential, b"synthetic-never-opened", 0o000);
    let contract_path = fixture.run_root.join("contract.json");
    fs::remove_file(&contract_path).unwrap();
    symlink(&credential, &contract_path).unwrap();
    let audit = audit_fixture(&fixture, NOW + 2_000);
    assert!(!audit.artifacts_consistent);
    assert!(audit
        .failures
        .iter()
        .any(|failure| failure.contains("authentication-")));

    let fixture = completed_fixture();
    let credential = fixture
        .bundle_root
        .parent()
        .unwrap()
        .join(".credentials.json");
    write_mode(&credential, b"synthetic-never-opened", 0o000);
    let contract_path = fixture.run_root.join("contract.json");
    fs::remove_file(&contract_path).unwrap();
    fs::hard_link(&credential, &contract_path).unwrap();
    let audit = audit_fixture(&fixture, NOW + 2_000);
    assert!(!audit.artifacts_consistent);
    assert!(audit
        .failures
        .iter()
        .any(|failure| failure.contains("single-link") || failure.contains("metadata drifted")));
}

#[cfg(unix)]
#[test]
fn special_privilege_bits_fail_in_source_archive_and_guest_evidence() {
    let temporary = tempfile::tempdir().unwrap();
    set_mode(temporary.path(), 0o700);
    let root = fs::canonicalize(temporary.path()).unwrap();
    let bundle = root.join("bundle");
    fs::create_dir(&bundle).unwrap();
    set_mode(&bundle, 0o700);
    write_mode(&bundle.join("runtime"), b"runtime", 0o4555);
    set_mode(&bundle.join("runtime"), 0o4555);
    assert!(derive_vm_bundle_manifest(&bundle).is_err());

    let directory_bundle = root.join("directory-bundle");
    fs::create_dir(&directory_bundle).unwrap();
    set_mode(&directory_bundle, 0o2700);
    write_mode(&directory_bundle.join("runtime"), b"runtime", 0o500);
    assert!(derive_vm_bundle_manifest(&directory_bundle).is_err());

    let fixture = contract_fixture();
    let first = &fixture.contract.bundle_manifest.entries[0].relative_path;
    set_mode(&fixture.bundle_root.join(first), 0o4555);
    let archive = fixture
        .bundle_root
        .parent()
        .unwrap()
        .join("setuid-entry.tar");
    let mut command = Command::new("/usr/bin/tar");
    command
        .arg("--format")
        .arg("ustar")
        .arg("-cf")
        .arg(&archive)
        .arg("-C")
        .arg(&fixture.bundle_root);
    for entry in &fixture.contract.bundle_manifest.entries {
        command.arg(&entry.relative_path);
    }
    assert!(command.status().unwrap().success());
    set_mode(&archive, 0o600);
    assert!(validate_ustar_archive(&archive, &fixture.contract.bundle_manifest).is_err());

    let fixture = contract_fixture();
    set_mode(Path::new(&fixture.contract.bundle_archive.path), 0o4600);
    assert!(validate_ustar_archive(
        Path::new(&fixture.contract.bundle_archive.path),
        &fixture.contract.bundle_manifest
    )
    .is_err());

    let fixture = completed_fixture();
    let mut inventory =
        String::from_utf8(expected_output(&fixture.contract, "guest_bundle_inventory")).unwrap();
    inventory = inventory.replacen("555|1|", "4555|1|", 1);
    forge_raw_and_rebind_receipt(&fixture, "guest_bundle_inventory", inventory.as_bytes());
    assert!(!audit_fixture(&fixture, NOW + 2_000).artifacts_consistent);

    let fixture = completed_fixture();
    let mut directories = String::from_utf8(expected_output(
        &fixture.contract,
        "guest_bundle_directories",
    ))
    .unwrap();
    directories = directories.replacen("755|0|0|", "2755|0|0|", 1);
    forge_raw_and_rebind_receipt(&fixture, "guest_bundle_directories", directories.as_bytes());
    assert!(!audit_fixture(&fixture, NOW + 2_000).artifacts_consistent);

    let fixture = completed_fixture();
    let pin = &fixture.contract.required_runtimes["codex"];
    let forged = format!("regular file|1|4555|0|0|19|{}\n", pin.path);
    forge_raw_and_rebind_receipt(&fixture, "guest_runtime_codex_stat", forged.as_bytes());
    assert!(!audit_fixture(&fixture, NOW + 2_000).artifacts_consistent);
}

#[cfg(unix)]
#[test]
fn apparmor_requires_one_exact_enforcing_bwrap_profile() {
    for forged in [
        b"bwrap-default (complain)\n".as_slice(),
        b"bwrap-default (kill)\n".as_slice(),
        b"bwrap-default (enforce)\nextra (enforce)\n".as_slice(),
        b"unconfined\n".as_slice(),
    ] {
        let fixture = completed_fixture();
        forge_raw_and_rebind_receipt(&fixture, "guest_bwrap_profile", forged);
        let audit = audit_fixture(&fixture, NOW + 2_000);
        assert!(!audit.artifacts_consistent);
        assert!(audit
            .failures
            .iter()
            .any(|failure| failure.contains("AppArmor profile")));
    }
}

#[cfg(unix)]
#[test]
fn version_probes_and_guest_paths_are_exact_and_provider_free() {
    let fixture = contract_fixture();
    for (name, arguments) in [
        ("claude", vec!["--print", "contact provider"]),
        ("codex", vec!["exec", "contact provider"]),
    ] {
        let mut bad = fixture.contract.clone();
        let pin = bad.required_runtimes.get_mut(name).unwrap();
        pin.version_argv = std::iter::once(pin.path.clone())
            .chain(arguments.into_iter().map(str::to_string))
            .collect();
        let error = validate_native_vm_contract(&bad).unwrap_err().to_string();
        assert!(error.contains("provider-free probe"), "{error}");
    }

    let mut bad = fixture.contract.clone();
    let runtime_path = bad.required_runtimes["claude"].path.clone();
    let sudo = bad.collector_tools.get_mut("sudo").unwrap();
    sudo.version_argv = vec![sudo.path.clone(), runtime_path, "--print".to_string()];
    let error = validate_native_vm_contract(&bad).unwrap_err().to_string();
    assert!(error.contains("provider-free probe") || error.contains("unsafe"));

    let mut bad = fixture.contract.clone();
    let codex = bad.required_runtimes.get_mut("codex").unwrap();
    codex.path = format!("{}/bin/not-codex", bad.guest_bundle_root);
    let error = validate_native_vm_contract(&bad).unwrap_err().to_string();
    assert!(error.contains("exact frozen guest location"), "{error}");

    for command in &fixture.contract.command_plan {
        assert!(!command.executable.contains("credentials"));
        if command.label == "guest_runtime_claude_version"
            || command.label == "guest_runtime_codex_version"
        {
            assert_eq!(command.argv.last().map(String::as_str), Some("--version"));
            assert!(!command.argv.iter().any(|argument| {
                matches!(argument.as_str(), "--print" | "exec" | "login" | "auth")
            }));
        }
    }
}

#[cfg(unix)]
#[test]
fn fail_fast_plan_gates_transfer_and_runtime_execution_on_isolation_and_bundle_checks() {
    let fixture = contract_fixture();
    let index = |label: &str| {
        fixture
            .contract
            .command_plan
            .iter()
            .position(|command| command.label == label)
            .unwrap()
    };
    let transfer = index("guest_transfer_bundle");
    for preflight in [
        "guest_findmnt",
        "guest_sockets",
        "guest_ssh_auth_sock",
        "guest_teaching_user_absent",
        "guest_evaluation_user_absent",
        "guest_apparmor_enabled",
        "guest_bwrap_profile",
        "guest_bundle_filesystem",
    ] {
        assert!(
            index(preflight) < transfer,
            "{preflight} must gate transfer"
        );
    }
    let first_runtime_version = index("guest_runtime_cargo_version");
    for bundle_gate in [
        "guest_bundle_inventory",
        "guest_bundle_directories",
        "guest_bundle_rehash",
        "guest_special_entries",
        "guest_runtime_cargo_path",
        "guest_runtime_cargo_stat",
        "guest_runtime_cargo_hash",
        "guest_runtime_cargo_hash_alt",
    ] {
        assert!(
            index(bundle_gate) < first_runtime_version,
            "{bundle_gate} must gate bundled runtime execution"
        );
    }

    let fixture = completed_fixture();
    forge_raw_and_rebind_receipt(
        &fixture,
        "guest_findmnt",
        b"{\"filesystems\":[{\"source\":\"host\",\"target\":\"/host\",\"fstype\":\"virtiofs\",\"options\":\"rw\"}]}\n",
    );
    let receipt: NativeVmCollectionReceipt =
        serde_json::from_slice(&fs::read(fixture.run_root.join("receipt.json")).unwrap()).unwrap();
    let failed_index = index_in_contract(&fixture.contract, "guest_findmnt");
    let result = native_vm::validate_native_vm_checkpoint_for_test(
        &fixture.contract,
        &fixture.contract.command_plan[failed_index],
        &receipt.commands[..failed_index],
        &receipt.commands[failed_index],
        &fixture.run_root,
    );
    assert!(result.is_err());
    assert!(failed_index < index_in_contract(&fixture.contract, "guest_transfer_bundle"));
}

#[cfg(unix)]
#[test]
fn every_derived_command_has_an_incremental_semantic_checkpoint() {
    let fixture = completed_fixture();
    let receipt: NativeVmCollectionReceipt =
        serde_json::from_slice(&fs::read(fixture.run_root.join("receipt.json")).unwrap()).unwrap();
    for (index, (spec, observed)) in fixture
        .contract
        .command_plan
        .iter()
        .zip(&receipt.commands)
        .enumerate()
    {
        native_vm::validate_native_vm_checkpoint_for_test(
            &fixture.contract,
            spec,
            &receipt.commands[..index],
            observed,
            &fixture.run_root,
        )
        .unwrap_or_else(|error| panic!("missing checkpoint for {}: {error}", spec.label));
    }
}

#[cfg(unix)]
#[test]
fn every_injected_post_spawn_error_kills_the_group_and_reaps_the_direct_child() {
    for fault in [
        NativeVmRunnerTestFault::PipeAcquisition,
        NativeVmRunnerTestFault::TryWait,
        NativeVmRunnerTestFault::Wait,
        NativeVmRunnerTestFault::ThreadSetup,
        NativeVmRunnerTestFault::ThreadJoin,
    ] {
        let temporary = tempfile::tempdir().unwrap();
        set_mode(temporary.path(), 0o700);
        let output_root = fs::canonicalize(temporary.path()).unwrap();
        let child_pid = AtomicU32::new(0);
        let error = exercise_process_group_guard_failure_for_test(fault, &output_root, &child_pid)
            .unwrap_err();
        assert!(
            error.to_string().contains("injected")
                || matches!(fault, NativeVmRunnerTestFault::ThreadJoin)
        );
        assert_child_group_is_gone_and_reaped(child_pid.load(Ordering::SeqCst));
    }
}

#[cfg(unix)]
#[test]
fn cleanup_failure_is_reported_alongside_the_primary_failure() {
    let temporary = tempfile::tempdir().unwrap();
    set_mode(temporary.path(), 0o700);
    let output_root = fs::canonicalize(temporary.path()).unwrap();
    let child_pid = AtomicU32::new(0);
    let error = exercise_process_group_guard_failure_for_test(
        NativeVmRunnerTestFault::PrimaryAndCleanup,
        &output_root,
        &child_pid,
    )
    .unwrap_err()
    .to_string();
    assert!(
        error.contains("injected primary post-spawn failure"),
        "{error}"
    );
    assert!(error.contains("injected cleanup failure"), "{error}");
    assert_child_group_is_gone_and_reaped(child_pid.load(Ordering::SeqCst));
}

#[cfg(unix)]
#[test]
fn panic_unwind_guard_kills_the_group_and_reaps_the_direct_child() {
    let temporary = tempfile::tempdir().unwrap();
    set_mode(temporary.path(), 0o700);
    let output_root = fs::canonicalize(temporary.path()).unwrap();
    let child_pid = AtomicU32::new(0);
    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = exercise_process_group_guard_failure_for_test(
            NativeVmRunnerTestFault::PanicUnwind,
            &output_root,
            &child_pid,
        );
    }));
    assert!(unwind.is_err());
    assert_child_group_is_gone_and_reaped(child_pid.load(Ordering::SeqCst));
}

#[cfg(unix)]
fn assert_child_group_is_gone_and_reaped(pid: u32) {
    assert_ne!(pid, 0, "test runner did not observe the spawned child pid");
    let pid = libc::pid_t::try_from(pid).unwrap();
    let mut status = 0;
    let waited = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) };
    assert_eq!(waited, -1, "direct child remained waitable after cleanup");
    assert_eq!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(libc::ECHILD),
        "direct child was not synchronously reaped"
    );
    let group = unsafe { libc::killpg(pid, 0) };
    assert_eq!(group, -1, "child process group survived cleanup");
    assert_eq!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(libc::ESRCH),
        "unexpected process-group probe result"
    );
}

fn index_in_contract(contract: &NativeVmCollectionContract, label: &str) -> usize {
    contract
        .command_plan
        .iter()
        .position(|command| command.label == label)
        .unwrap()
}
