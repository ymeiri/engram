//! Provider-free isolation foundations for a native stale-safety successor.
//!
//! This module deliberately stops before provider execution. It validates and materializes a
//! closed-world semantic allowlist into a disjoint evaluation enclave, validates the exact host
//! launch boundary, and models the later mount-free Colima replication attestation. The macOS
//! boundary is limited to the named filesystem paths and model-command surfaces proved by the
//! probes; it is not a sandbox for a malicious provider client or a trusted MCP server.

use crate::{EvalError, EvalResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::CString;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr};
#[cfg(target_os = "macos")]
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
#[cfg(target_os = "macos")]
use std::time::{Duration, Instant};
use std::time::{SystemTime, UNIX_EPOCH};
use zeroize::Zeroizing;

const ISOLATION_SCHEMA_VERSION: u32 = 1;
const COLIMA_ISOLATION_SCHEMA_VERSION: u32 = 1;
const MAX_MANIFEST_FILE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_MANIFEST_TOTAL_BYTES: u64 = 512 * 1024 * 1024;
const MIN_VM_DISK_GIB: u32 = 40;
const CODEX_CLI_VERSION: &str = "codex-cli 0.153.3";
const CLAUDE_CLI_VERSION: &str = "2.1.260 (Claude Code)";
const CLAUDE_TRACE_CLI_VERSION: &str = "2.1.260";
const CODEX_CONFIG_LAYER: &str = "isolation";
const CODEX_PERMISSION_PROFILE: &str = "engram-eval-read";
const CLAUDE_PERMISSION_MODE: &str = "dontAsk";
const CLAUDE_DISALLOWED_TOOLS: &str = "Bash,Write,Edit,WebFetch,WebSearch,NotebookEdit,Task";
const CLAUDE_PROVIDER_AUTHORITY: &str = "api.anthropic.com:443";
const CLAUDE_PROVIDER_HOST: &str = "api.anthropic.com";
const CLAUDE_PROVIDER_PORT: u16 = 443;
const CLAUDE_REDACTED_PROXY_PASSWORD: &str = "REDACTED_PER_INVOCATION_SECRET";
const PROVIDER_VERSION_TIMEOUT_MS: u64 = 10_000;
const ENGRAM_HEALTH_SCHEMA_VERSION: u64 = 3;
const ENGRAM_MCP_CONTRACT_VERSION: u64 = 5;
const ENGRAM_MCP_PROTOCOL_VERSION: &str = "2024-11-05";
const MAX_ENGRAM_HEALTH_RESPONSE_BYTES: usize = 64 * 1024;
const CLAUDE_MANAGED_SETTINGS_PATH: &str =
    "/Library/Application Support/ClaudeCode/managed-settings.json";
const CLAUDE_MANAGED_MCP_PATH: &str = "/Library/Application Support/ClaudeCode/managed-mcp.json";
const CLAUDE_MANAGED_PREFERENCES_PATH: &str =
    "/Library/Managed Preferences/com.anthropic.claudecode.plist";
const CLAUDE_MANAGED_SETTINGS_DROP_IN_DIRECTORY: &str =
    "/Library/Application Support/ClaudeCode/managed-settings.d";
const CLAUDE_EXTERNAL_PROBE_ADDRESS: &str = "192.0.2.1";
const CLAUDE_EXTERNAL_PROBE_PORT: &str = "443";
const ENGRAM_AGENT_TOOLS: [&str; 6] = [
    "mcp__engram__harness",
    "mcp__engram__memory",
    "mcp__engram__obligations",
    "mcp__engram__orient",
    "mcp__engram__repo",
    "mcp__engram__search",
];
const CLAUDE_BUILT_IN_AGENTS: [&str; 5] = [
    "claude",
    "Explore",
    "general-purpose",
    "Plan",
    "statusline-setup",
];
const CLAUDE_CAPABILITIES: [&str; 3] = [
    "interrupt_receipt_v1",
    "interrupt_cancel_queued_v1",
    "msg_lifecycle_v1",
];

/// Native host whose evaluation process is isolated.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IsolationHost {
    Codex,
    ClaudeCode,
}

/// The only semantic file classes that may cross from teaching state into evaluation state.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum IsolationCopyClass {
    EvaluationCheckout,
    OutputSchema,
    Settings,
    McpConfig,
    Instructions,
    CodexMemories,
    ClaudeAutoMemory,
    EngramState,
}

/// One closed-world subtree in the source and its exact destination in the enclave.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IsolationCopyZone {
    pub class: IsolationCopyClass,
    pub source_prefix: String,
    pub destination_prefix: String,
}

/// One hash-bound regular file in a closed-world copy zone.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IsolationCopyEntry {
    pub class: IsolationCopyClass,
    pub source: String,
    pub destination: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub mode: u32,
}

/// Explicit semantic allowlist copied at the trusted retention transition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IsolationCopyManifest {
    pub schema_version: u32,
    pub host: IsolationHost,
    pub native_memory_enabled: bool,
    pub uses_engram: bool,
    pub source_root: String,
    pub evaluation_root: String,
    pub max_file_bytes: u64,
    pub max_total_bytes: u64,
    pub zones: Vec<IsolationCopyZone>,
    pub entries: Vec<IsolationCopyEntry>,
    /// Exact marker values which must not occur in any copied byte surface.
    pub forbidden_markers: Vec<String>,
    /// Additional exact source/run/teaching literals forbidden in copied content.
    #[serde(default)]
    pub forbidden_content_literals: Vec<String>,
    /// Digest of the exact lane boundary, including the exact marker set.
    pub boundary_sha256: String,
}

/// Content fingerprint for a materialized non-credential file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IsolationStateFile {
    pub class: IsolationCopyClass,
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub mode: u32,
}

/// Closed-world state snapshot used before and after provider execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IsolationStateSnapshot {
    pub schema_version: u32,
    pub evaluation_root: String,
    pub files: Vec<IsolationStateFile>,
    pub tree_sha256: String,
}

/// Sanitized output of an exact, runner-controlled semantic Engram export.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IsolationEngramSemanticEvidence {
    pub valid: bool,
    pub project: String,
    pub query_contract_sha256: String,
    pub canonical_records_sha256: String,
    pub record_count: u64,
    pub raw_database_sha256: String,
    pub collected_from_lane_api: bool,
    pub raw_response_retained: bool,
    pub collected_unix_ms: u64,
    pub evidence_sha256: String,
    /// Deserialization and caller-side construction cannot assert runner ownership. The future
    /// integrated collector must set this immediately after the exact live API extraction.
    #[serde(skip)]
    runner_observed_live: bool,
}

/// Native/config bytes plus semantic Engram contents at one evaluation boundary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IsolationSemanticStateSnapshot {
    pub schema_version: u32,
    pub evaluation_root: String,
    pub immutable_files: Vec<IsolationStateFile>,
    pub immutable_tree_sha256: String,
    pub engram: Option<IsolationEngramSemanticEvidence>,
    pub snapshot_sha256: String,
}

/// Provider-free result of materializing a fresh enclave.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IsolationMaterializationAudit {
    pub valid: bool,
    pub source_root: String,
    pub evaluation_root: String,
    pub file_count: usize,
    pub byte_count: u64,
    pub manifest_sha256: String,
    pub state: IsolationStateSnapshot,
}

#[derive(Debug, Clone)]
struct ValidatedSourceFile {
    entry: IsolationCopyEntry,
    source: PathBuf,
    destination: PathBuf,
}

/// Validate every source and copy exactly the hash-manifested allowlist into a fresh enclave.
///
/// Credential-looking paths are rejected before they are opened. Every declared copy zone is
/// closed-world: an unlisted file, symlink, non-regular node, or cross-root hard link fails the
/// transition. Destinations are created with no-overwrite semantics.
pub fn materialize_isolation_enclave(
    manifest: &IsolationCopyManifest,
    boundary: &IsolationBoundaryContract,
) -> EvalResult<IsolationMaterializationAudit> {
    validate_manifest_boundary_binding(manifest, boundary)?;
    let validated = validate_copy_manifest(manifest)?;
    for file in &validated {
        let bytes = read_validated_source(file, manifest)?;
        write_materialized_file(file, &bytes)?;
    }
    let state = snapshot_isolation_state(manifest, boundary)?;
    let byte_count = state.files.iter().map(|file| file.size_bytes).sum();
    Ok(IsolationMaterializationAudit {
        valid: true,
        source_root: canonical_directory(Path::new(&manifest.source_root))?
            .display()
            .to_string(),
        evaluation_root: canonical_directory(Path::new(&manifest.evaluation_root))?
            .display()
            .to_string(),
        file_count: state.files.len(),
        byte_count,
        manifest_sha256: sha256_serialized(manifest)?,
        state,
    })
}

/// Hash the exact copied semantic state without traversing credential homes or other enclave data.
pub fn snapshot_isolation_state(
    manifest: &IsolationCopyManifest,
    boundary: &IsolationBoundaryContract,
) -> EvalResult<IsolationStateSnapshot> {
    validate_manifest_boundary_binding(manifest, boundary)?;
    validate_manifest_structure(manifest)?;
    let evaluation_root = canonical_directory(Path::new(&manifest.evaluation_root))?;
    let zones = validated_zone_map(manifest)?;
    let expected = expected_destination_entries(manifest)?;
    validate_closed_world_destinations(manifest, &evaluation_root, &expected)?;

    let mut files = Vec::with_capacity(manifest.entries.len());
    for entry in &manifest.entries {
        validate_manifest_entry(entry, manifest)?;
        let zone = zones
            .get(&entry.class)
            .ok_or_else(|| invalid(format!("entry has no copy zone: {}", entry.destination)))?;
        require_zone_mapping(entry, zone)?;
        let relative = clean_relative(&entry.destination, "manifest destination")?;
        let path = evaluation_root.join(&relative);
        reject_credential_semantic_path(&relative, entry.class)?;
        let bytes =
            read_exact_regular_file(&path, entry.size_bytes, entry.mode, manifest.max_file_bytes)?;
        let sha256 = sha256_bytes(&bytes);
        if sha256 != entry.sha256 {
            return Err(invalid(format!(
                "materialized file digest changed: {}",
                path.display()
            )));
        }
        files.push(IsolationStateFile {
            class: entry.class,
            path: entry.destination.clone(),
            sha256,
            size_bytes: entry.size_bytes,
            mode: entry.mode,
        });
    }
    files.sort_by(|left, right| left.path.cmp(&right.path));
    let tree_sha256 = sha256_serialized(&files)?;
    Ok(IsolationStateSnapshot {
        schema_version: ISOLATION_SCHEMA_VERSION,
        evaluation_root: evaluation_root.display().to_string(),
        files,
        tree_sha256,
    })
}

/// Bind copy authorization to the exact host/arm/roots/marker boundary bytes.
pub fn validate_manifest_boundary_binding(
    manifest: &IsolationCopyManifest,
    boundary: &IsolationBoundaryContract,
) -> EvalResult<()> {
    if boundary.schema_version != ISOLATION_SCHEMA_VERSION
        || boundary.forbidden_markers.is_empty()
        || boundary
            .forbidden_markers
            .iter()
            .any(|marker| marker.len() < 8)
    {
        return Err(invalid("copy boundary has an invalid schema or marker set"));
    }
    require_unique_strings(boundary.forbidden_markers.iter(), "copy boundary marker")?;
    if manifest.boundary_sha256 != sha256_serialized(boundary)?
        || manifest.host != boundary.host
        || manifest.native_memory_enabled != boundary.native_memory_enabled
        || manifest.uses_engram != boundary.uses_engram
        || manifest.source_root != boundary.source_root
        || manifest.evaluation_root != boundary.evaluation_root
        || manifest.forbidden_markers != boundary.forbidden_markers
    {
        return Err(invalid(
            "copy manifest is not bound to the exact lane boundary/marker set",
        ));
    }
    validate_manifest_state_zone_binding(manifest, boundary)?;
    Ok(())
}

fn validate_manifest_state_zone_binding(
    manifest: &IsolationCopyManifest,
    boundary: &IsolationBoundaryContract,
) -> EvalResult<()> {
    let evaluation_root = Path::new(&boundary.evaluation_root);
    let zones = validated_zone_map(manifest)?;
    for (copy_class, boundary_class) in [
        (
            match boundary.host {
                IsolationHost::Codex => IsolationCopyClass::CodexMemories,
                IsolationHost::ClaudeCode => IsolationCopyClass::ClaudeAutoMemory,
            },
            IsolationBoundaryPathClass::NativeState,
        ),
        (
            IsolationCopyClass::EngramState,
            IsolationBoundaryPathClass::EngramState,
        ),
    ] {
        let zone = zones.get(&copy_class);
        let protected = boundary
            .forbidden_paths
            .iter()
            .find(|path| path.class == boundary_class);
        match (zone, protected) {
            (None, None) => {}
            (Some(zone), Some(protected)) => {
                let destination = Path::new(&protected.path)
                    .strip_prefix(evaluation_root)
                    .map_err(|_| invalid("state boundary path is outside the evaluation root"))?;
                let source_prefix = clean_relative(&zone.source_prefix, "state source prefix")?;
                let destination_prefix =
                    clean_relative(&zone.destination_prefix, "state destination prefix")?;
                if destination_prefix != destination || source_prefix != destination_prefix {
                    return Err(invalid(
                        "native/Engram copy zone is not the exact mirrored launch-state root",
                    ));
                }
            }
            _ => {
                return Err(invalid(
                    "native/Engram copy zone does not match its exact boundary state root",
                ))
            }
        }
    }
    Ok(())
}

/// Require exact native and Engram semantic state equality across evaluation.
pub fn require_isolation_state_unchanged(
    before: &IsolationStateSnapshot,
    after: &IsolationStateSnapshot,
) -> EvalResult<()> {
    if before.schema_version != ISOLATION_SCHEMA_VERSION
        || after.schema_version != ISOLATION_SCHEMA_VERSION
        || before.evaluation_root != after.evaluation_root
        || before.tree_sha256 != after.tree_sha256
        || before.files != after.files
    {
        return Err(invalid(
            "native or Engram evaluation state changed after the pre-evaluation snapshot",
        ));
    }
    Ok(())
}

/// Snapshot exact native/config state while representing read-mutating Engram state semantically.
///
/// Engram retrieval records a telemetry trace, so RocksDB/WAL byte equality is not a valid
/// no-stale-influence invariant. The trusted runner must instead collect the supplied evidence
/// through a separately frozen, read-only semantic export contract. The raw database digest is
/// retained as an audit signal but is not required to remain equal.
pub fn snapshot_isolation_semantic_state(
    manifest: &IsolationCopyManifest,
    boundary: &IsolationBoundaryContract,
    expected_engram_project: Option<&str>,
    engram: Option<IsolationEngramSemanticEvidence>,
) -> EvalResult<IsolationSemanticStateSnapshot> {
    validate_manifest_boundary_binding(manifest, boundary)?;
    validate_manifest_structure(manifest)?;
    let evaluation_root = canonical_directory(Path::new(&manifest.evaluation_root))?;
    let expected = expected_destination_entries(manifest)?;
    validate_closed_world_immutable_destinations(manifest, &evaluation_root, &expected)?;
    if manifest.uses_engram != engram.is_some()
        || manifest.uses_engram != expected_engram_project.is_some()
    {
        return Err(invalid(
            "semantic Engram evidence does not match the exact lane arm",
        ));
    }
    if let Some(evidence) = &engram {
        validate_engram_semantic_evidence(evidence)?;
        if Some(evidence.project.as_str()) != expected_engram_project {
            return Err(invalid(
                "semantic Engram evidence does not match the frozen project",
            ));
        }
    }
    let mut immutable_files = Vec::new();
    for entry in manifest
        .entries
        .iter()
        .filter(|entry| entry.class != IsolationCopyClass::EngramState)
    {
        let relative = clean_relative(&entry.destination, "manifest destination")?;
        let path = evaluation_root.join(relative);
        let bytes =
            read_exact_regular_file(&path, entry.size_bytes, entry.mode, manifest.max_file_bytes)?;
        let sha256 = sha256_bytes(&bytes);
        if sha256 != entry.sha256 {
            return Err(invalid(format!(
                "immutable native/config state changed: {}",
                path.display()
            )));
        }
        immutable_files.push(IsolationStateFile {
            class: entry.class,
            path: entry.destination.clone(),
            sha256,
            size_bytes: entry.size_bytes,
            mode: entry.mode,
        });
    }
    immutable_files.sort_by(|left, right| left.path.cmp(&right.path));
    let immutable_tree_sha256 = sha256_serialized(&immutable_files)?;
    let snapshot_sha256 = sha256_serialized(&(
        ISOLATION_SCHEMA_VERSION,
        evaluation_root.display().to_string(),
        &immutable_files,
        &immutable_tree_sha256,
        &engram,
    ))?;
    Ok(IsolationSemanticStateSnapshot {
        schema_version: ISOLATION_SCHEMA_VERSION,
        evaluation_root: evaluation_root.display().to_string(),
        immutable_files,
        immutable_tree_sha256,
        engram,
        snapshot_sha256,
    })
}

/// Require exact native/config equality and exact semantic Engram equality across evaluation.
pub fn require_isolation_semantic_state_unchanged(
    before: &IsolationSemanticStateSnapshot,
    after: &IsolationSemanticStateSnapshot,
) -> EvalResult<()> {
    validate_semantic_state_snapshot(before)?;
    validate_semantic_state_snapshot(after)?;
    let engram_unchanged = match (&before.engram, &after.engram) {
        (None, None) => true,
        (Some(before), Some(after)) => {
            validate_engram_semantic_evidence(before)?;
            validate_engram_semantic_evidence(after)?;
            before.project == after.project
                && before.query_contract_sha256 == after.query_contract_sha256
                && before.canonical_records_sha256 == after.canonical_records_sha256
                && before.record_count == after.record_count
        }
        _ => false,
    };
    if before.schema_version != ISOLATION_SCHEMA_VERSION
        || after.schema_version != ISOLATION_SCHEMA_VERSION
        || before.evaluation_root != after.evaluation_root
        || before.immutable_tree_sha256 != after.immutable_tree_sha256
        || before.immutable_files != after.immutable_files
        || !engram_unchanged
    {
        return Err(invalid(
            "native/config bytes or semantic Engram records changed across evaluation",
        ));
    }
    Ok(())
}

fn validate_semantic_state_snapshot(snapshot: &IsolationSemanticStateSnapshot) -> EvalResult<()> {
    let expected_immutable = sha256_serialized(&snapshot.immutable_files)?;
    let expected_snapshot = sha256_serialized(&(
        snapshot.schema_version,
        &snapshot.evaluation_root,
        &snapshot.immutable_files,
        &snapshot.immutable_tree_sha256,
        &snapshot.engram,
    ))?;
    if snapshot.schema_version != ISOLATION_SCHEMA_VERSION
        || !Path::new(&snapshot.evaluation_root).is_absolute()
        || snapshot.immutable_files.is_empty()
        || snapshot.immutable_tree_sha256 != expected_immutable
        || snapshot.snapshot_sha256 != expected_snapshot
    {
        return Err(invalid("semantic state snapshot digest/shape drifted"));
    }
    if let Some(evidence) = &snapshot.engram {
        validate_engram_semantic_evidence(evidence)?;
    }
    Ok(())
}

pub fn build_engram_semantic_evidence(
    project: &str,
    query_contract_sha256: &str,
    canonical_records_sha256: &str,
    record_count: u64,
    raw_database_sha256: &str,
    collected_unix_ms: u64,
) -> EvalResult<IsolationEngramSemanticEvidence> {
    let mut evidence = IsolationEngramSemanticEvidence {
        valid: true,
        project: project.to_string(),
        query_contract_sha256: query_contract_sha256.to_string(),
        canonical_records_sha256: canonical_records_sha256.to_string(),
        record_count,
        raw_database_sha256: raw_database_sha256.to_string(),
        collected_from_lane_api: true,
        raw_response_retained: false,
        collected_unix_ms,
        evidence_sha256: String::new(),
        runner_observed_live: false,
    };
    evidence.evidence_sha256 = engram_semantic_evidence_digest(&evidence)?;
    validate_engram_semantic_evidence(&evidence)?;
    Ok(evidence)
}

fn validate_engram_semantic_evidence(evidence: &IsolationEngramSemanticEvidence) -> EvalResult<()> {
    if !evidence.valid
        || evidence.project.is_empty()
        || evidence.project.len() > 128
        || !evidence
            .project
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        || matches!(evidence.project.as_str(), "." | "..")
        || !is_sha256(&evidence.query_contract_sha256)
        || !is_sha256(&evidence.canonical_records_sha256)
        || evidence.record_count == 0
        || !is_sha256(&evidence.raw_database_sha256)
        || !evidence.collected_from_lane_api
        || evidence.raw_response_retained
        || evidence.collected_unix_ms == 0
        || !evidence.runner_observed_live
        || !timestamp_is_recent(evidence.collected_unix_ms, 30_000)?
        || evidence.evidence_sha256 != engram_semantic_evidence_digest(evidence)?
    {
        return Err(invalid(
            "Engram semantic-state evidence is incomplete or not exact",
        ));
    }
    Ok(())
}

fn engram_semantic_evidence_digest(
    evidence: &IsolationEngramSemanticEvidence,
) -> EvalResult<String> {
    sha256_serialized(&(
        evidence.valid,
        &evidence.project,
        &evidence.query_contract_sha256,
        &evidence.canonical_records_sha256,
        evidence.record_count,
        &evidence.raw_database_sha256,
        evidence.collected_from_lane_api,
        evidence.raw_response_retained,
        evidence.collected_unix_ms,
    ))
}

fn validate_copy_manifest(
    manifest: &IsolationCopyManifest,
) -> EvalResult<Vec<ValidatedSourceFile>> {
    validate_manifest_structure(manifest)?;
    let source_root = canonical_directory(Path::new(&manifest.source_root))?;
    let evaluation_root = canonical_directory(Path::new(&manifest.evaluation_root))?;
    require_disjoint_roots(&source_root, &evaluation_root)?;

    let zone_map = validated_zone_map(manifest)?;
    validate_closed_world_sources(manifest, &source_root, &zone_map)?;
    validate_fresh_destinations(manifest, &evaluation_root)?;

    let mut total_bytes = 0_u64;
    let mut files = Vec::with_capacity(manifest.entries.len());
    for entry in &manifest.entries {
        validate_manifest_entry(entry, manifest)?;
        let source_relative = clean_relative(&entry.source, "manifest source")?;
        let destination_relative = clean_relative(&entry.destination, "manifest destination")?;
        reject_credential_semantic_path(&source_relative, entry.class)?;
        reject_credential_semantic_path(&destination_relative, entry.class)?;
        let zone = zone_map
            .get(&entry.class)
            .ok_or_else(|| invalid(format!("entry has no copy zone: {}", entry.source)))?;
        require_zone_mapping(entry, zone)?;
        total_bytes = total_bytes
            .checked_add(entry.size_bytes)
            .ok_or_else(|| invalid("manifest byte count overflow"))?;
        if total_bytes > manifest.max_total_bytes {
            return Err(invalid("manifest exceeds its total byte limit"));
        }

        let source = source_root.join(source_relative);
        validate_existing_ancestors(&source_root, &source)?;
        let bytes = read_exact_regular_file(
            &source,
            entry.size_bytes,
            entry.mode,
            manifest.max_file_bytes,
        )?;
        validate_copied_content(&bytes, manifest, &source_root)?;
        if sha256_bytes(&bytes) != entry.sha256 {
            return Err(invalid(format!(
                "source digest does not match manifest: {}",
                source.display()
            )));
        }
        files.push(ValidatedSourceFile {
            entry: entry.clone(),
            source,
            destination: evaluation_root.join(destination_relative),
        });
    }
    Ok(files)
}

fn validate_manifest_structure(manifest: &IsolationCopyManifest) -> EvalResult<()> {
    if manifest.schema_version != ISOLATION_SCHEMA_VERSION {
        return Err(invalid("unsupported isolation manifest schema version"));
    }
    if !is_sha256(&manifest.boundary_sha256) {
        return Err(invalid("copy manifest has no boundary digest"));
    }
    if manifest.max_file_bytes == 0
        || manifest.max_file_bytes > MAX_MANIFEST_FILE_BYTES
        || manifest.max_total_bytes < manifest.max_file_bytes
        || manifest.max_total_bytes > MAX_MANIFEST_TOTAL_BYTES
    {
        return Err(invalid("invalid isolation manifest byte limits"));
    }
    if manifest.zones.is_empty() || manifest.entries.is_empty() {
        return Err(invalid("isolation manifest must contain zones and files"));
    }
    let mut expected_classes = BTreeSet::from([
        IsolationCopyClass::EvaluationCheckout,
        IsolationCopyClass::OutputSchema,
        IsolationCopyClass::Settings,
        IsolationCopyClass::McpConfig,
        IsolationCopyClass::Instructions,
    ]);
    if manifest.native_memory_enabled {
        expected_classes.insert(match manifest.host {
            IsolationHost::Codex => IsolationCopyClass::CodexMemories,
            IsolationHost::ClaudeCode => IsolationCopyClass::ClaudeAutoMemory,
        });
    }
    if manifest.uses_engram {
        expected_classes.insert(IsolationCopyClass::EngramState);
    }
    let actual_classes = manifest
        .zones
        .iter()
        .map(|zone| zone.class)
        .collect::<BTreeSet<_>>();
    if actual_classes != expected_classes {
        return Err(invalid(
            "copy zones do not match the lane host/native/Engram arm",
        ));
    }
    let entry_counts = manifest.entries.iter().fold(
        BTreeMap::<IsolationCopyClass, usize>::new(),
        |mut counts, entry| {
            *counts.entry(entry.class).or_default() += 1;
            counts
        },
    );
    if expected_classes
        .iter()
        .any(|class| entry_counts.get(class).copied().unwrap_or_default() == 0)
    {
        return Err(invalid(
            "every required semantic copy zone must contain at least one file",
        ));
    }
    if manifest.forbidden_markers.is_empty()
        || manifest
            .forbidden_markers
            .iter()
            .any(|marker| marker.len() < 8)
    {
        return Err(invalid(
            "forbidden markers must contain at least eight bytes",
        ));
    }
    require_unique_strings(
        manifest
            .forbidden_markers
            .iter()
            .chain(manifest.forbidden_content_literals.iter()),
        "forbidden marker/content literal",
    )?;
    let source = Path::new(&manifest.source_root);
    let evaluation = Path::new(&manifest.evaluation_root);
    if !source.is_absolute() || !evaluation.is_absolute() {
        return Err(invalid("isolation roots must be absolute"));
    }
    require_private_directory(evaluation, "evaluation enclave")?;
    Ok(())
}

fn validated_zone_map(
    manifest: &IsolationCopyManifest,
) -> EvalResult<BTreeMap<IsolationCopyClass, IsolationCopyZone>> {
    let mut zones = BTreeMap::new();
    let mut source_prefixes = Vec::new();
    let mut destination_prefixes = Vec::new();
    for zone in &manifest.zones {
        let source = clean_relative(&zone.source_prefix, "copy-zone source prefix")?;
        let destination = clean_relative(&zone.destination_prefix, "copy-zone destination prefix")?;
        validate_native_zone_semantics(zone.class, &source, &destination)?;
        if zones.insert(zone.class, zone.clone()).is_some() {
            return Err(invalid("a semantic copy class has more than one zone"));
        }
        source_prefixes.push(source);
        destination_prefixes.push(destination);
    }
    require_nonoverlapping_relative_roots(&source_prefixes, "source copy zones")?;
    require_nonoverlapping_relative_roots(&destination_prefixes, "destination copy zones")?;
    Ok(zones)
}

fn validate_native_zone_semantics(
    class: IsolationCopyClass,
    source: &Path,
    destination: &Path,
) -> EvalResult<()> {
    let final_source = source.file_name().and_then(|name| name.to_str());
    let final_destination = destination.file_name().and_then(|name| name.to_str());
    match class {
        IsolationCopyClass::CodexMemories
            if final_source != Some("memories") || final_destination != Some("memories") =>
        {
            Err(invalid(
                "Codex native state copy zones must be exact memories directories",
            ))
        }
        IsolationCopyClass::ClaudeAutoMemory
            if !matches!(final_source, Some("memory" | "auto-memory"))
                || !matches!(final_destination, Some("memory" | "auto-memory")) =>
        {
            Err(invalid(
                "Claude native state copy zones must be exact memory directories",
            ))
        }
        _ => Ok(()),
    }
}

fn validate_manifest_entry(
    entry: &IsolationCopyEntry,
    manifest: &IsolationCopyManifest,
) -> EvalResult<()> {
    if !is_sha256(&entry.sha256) {
        return Err(invalid(format!(
            "manifest entry has an invalid SHA-256: {}",
            entry.source
        )));
    }
    if entry.size_bytes > manifest.max_file_bytes {
        return Err(invalid(format!(
            "manifest entry exceeds the per-file limit: {}",
            entry.source
        )));
    }
    if entry.mode > 0o777 || entry.mode & 0o022 != 0 {
        return Err(invalid(format!(
            "manifest mode must not grant group/other write access: {}",
            entry.source
        )));
    }
    Ok(())
}

fn require_zone_mapping(entry: &IsolationCopyEntry, zone: &IsolationCopyZone) -> EvalResult<()> {
    let source = clean_relative(&entry.source, "manifest source")?;
    let destination = clean_relative(&entry.destination, "manifest destination")?;
    let source_prefix = clean_relative(&zone.source_prefix, "copy-zone source prefix")?;
    let destination_prefix =
        clean_relative(&zone.destination_prefix, "copy-zone destination prefix")?;
    let source_suffix = source.strip_prefix(&source_prefix).map_err(|_| {
        invalid(format!(
            "source is outside its semantic zone: {}",
            entry.source
        ))
    })?;
    let destination_suffix = destination.strip_prefix(&destination_prefix).map_err(|_| {
        invalid(format!(
            "destination is outside its semantic zone: {}",
            entry.destination
        ))
    })?;
    if source_suffix.as_os_str().is_empty()
        || destination_suffix.as_os_str().is_empty()
        || source_suffix != destination_suffix
    {
        return Err(invalid(format!(
            "copy entry must preserve its exact zone-relative path: {}",
            entry.source
        )));
    }
    Ok(())
}

fn validate_closed_world_sources(
    manifest: &IsolationCopyManifest,
    source_root: &Path,
    zones: &BTreeMap<IsolationCopyClass, IsolationCopyZone>,
) -> EvalResult<()> {
    let expected = manifest
        .entries
        .iter()
        .map(|entry| (entry.source.clone(), entry.class))
        .collect::<BTreeSet<_>>();
    if expected.len() != manifest.entries.len() {
        return Err(invalid("manifest contains duplicate source paths"));
    }
    let destinations = manifest
        .entries
        .iter()
        .map(|entry| entry.destination.as_str())
        .collect::<BTreeSet<_>>();
    if destinations.len() != manifest.entries.len() {
        return Err(invalid("manifest contains duplicate destination paths"));
    }

    let mut observed = BTreeSet::new();
    for zone in zones.values() {
        let relative = clean_relative(&zone.source_prefix, "copy-zone source prefix")?;
        let root = source_root.join(&relative);
        validate_existing_ancestors(source_root, &root)?;
        walk_closed_world_files(source_root, &root, &relative, zone.class, &mut observed)?;
    }
    if observed != expected {
        let unlisted = observed.difference(&expected).next();
        let missing = expected.difference(&observed).next();
        return Err(invalid(format!(
            "closed-world source manifest mismatch (unlisted={unlisted:?}, missing={missing:?})"
        )));
    }
    Ok(())
}

fn walk_closed_world_files(
    source_root: &Path,
    directory: &Path,
    relative: &Path,
    class: IsolationCopyClass,
    observed: &mut BTreeSet<(String, IsolationCopyClass)>,
) -> EvalResult<()> {
    let metadata = fs::symlink_metadata(directory)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(invalid(format!(
            "copy zone must be a real directory: {}",
            directory.display()
        )));
    }
    let mut children = fs::read_dir(directory)?.collect::<Result<Vec<_>, _>>()?;
    children.sort_by_key(|entry| entry.file_name());
    for child in children {
        let path = child.path();
        let child_relative = relative.join(child.file_name());
        reject_credential_semantic_path(&child_relative, class)?;
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            return Err(invalid(format!(
                "copy zones may not contain symlinks: {}",
                path.display()
            )));
        }
        if metadata.is_dir() {
            walk_closed_world_files(source_root, &path, &child_relative, class, observed)?;
        } else if metadata.is_file() {
            require_single_link(&metadata, &path)?;
            let canonical = path.canonicalize()?;
            if !canonical.starts_with(source_root) {
                return Err(invalid("copy-zone file escapes the source root"));
            }
            observed.insert((child_relative.to_string_lossy().into_owned(), class));
        } else {
            return Err(invalid(format!(
                "copy zones may contain only directories and regular files: {}",
                path.display()
            )));
        }
    }
    Ok(())
}

fn validate_fresh_destinations(
    manifest: &IsolationCopyManifest,
    evaluation_root: &Path,
) -> EvalResult<()> {
    for zone in &manifest.zones {
        let relative = clean_relative(&zone.destination_prefix, "copy-zone destination prefix")?;
        let destination = evaluation_root.join(relative);
        match fs::symlink_metadata(&destination) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Ok(_) => {
                return Err(invalid(format!(
                    "copy-zone destination already exists: {}",
                    destination.display()
                )))
            }
            Err(error) => return Err(EvalError::Io(error)),
        }
    }
    Ok(())
}

fn expected_destination_entries(
    manifest: &IsolationCopyManifest,
) -> EvalResult<BTreeSet<(String, IsolationCopyClass)>> {
    let expected = manifest
        .entries
        .iter()
        .map(|entry| (entry.destination.clone(), entry.class))
        .collect::<BTreeSet<_>>();
    if expected.len() != manifest.entries.len() {
        return Err(invalid("manifest contains duplicate destination paths"));
    }
    Ok(expected)
}

fn validate_closed_world_destinations(
    manifest: &IsolationCopyManifest,
    evaluation_root: &Path,
    expected: &BTreeSet<(String, IsolationCopyClass)>,
) -> EvalResult<()> {
    let mut observed = BTreeSet::new();
    for zone in &manifest.zones {
        let relative = clean_relative(&zone.destination_prefix, "copy-zone destination prefix")?;
        let root = evaluation_root.join(&relative);
        validate_existing_ancestors(evaluation_root, &root)?;
        walk_closed_world_files(evaluation_root, &root, &relative, zone.class, &mut observed)?;
    }
    if &observed != expected {
        return Err(invalid(
            "materialized destination contains missing or unlisted files",
        ));
    }
    Ok(())
}

fn validate_closed_world_immutable_destinations(
    manifest: &IsolationCopyManifest,
    evaluation_root: &Path,
    expected: &BTreeSet<(String, IsolationCopyClass)>,
) -> EvalResult<()> {
    let expected = expected
        .iter()
        .filter(|(_, class)| *class != IsolationCopyClass::EngramState)
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut observed = BTreeSet::new();
    for zone in manifest
        .zones
        .iter()
        .filter(|zone| zone.class != IsolationCopyClass::EngramState)
    {
        let relative = clean_relative(&zone.destination_prefix, "copy-zone destination prefix")?;
        let root = evaluation_root.join(&relative);
        validate_existing_ancestors(evaluation_root, &root)?;
        walk_closed_world_files(evaluation_root, &root, &relative, zone.class, &mut observed)?;
    }
    if observed != expected {
        return Err(invalid(
            "immutable destination contains missing or unlisted files",
        ));
    }
    Ok(())
}

fn read_validated_source(
    file: &ValidatedSourceFile,
    manifest: &IsolationCopyManifest,
) -> EvalResult<Vec<u8>> {
    let bytes = read_exact_regular_file(
        &file.source,
        file.entry.size_bytes,
        file.entry.mode,
        manifest.max_file_bytes,
    )?;
    validate_copied_content(
        &bytes,
        manifest,
        &canonical_directory(Path::new(&manifest.source_root))?,
    )?;
    if sha256_bytes(&bytes) != file.entry.sha256 {
        return Err(invalid(format!(
            "source changed after manifest validation: {}",
            file.source.display()
        )));
    }
    Ok(bytes)
}

fn write_materialized_file(file: &ValidatedSourceFile, bytes: &[u8]) -> EvalResult<()> {
    let parent = file
        .destination
        .parent()
        .ok_or_else(|| invalid("materialized destination has no parent"))?;
    create_private_directory_tree(parent)?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(file.entry.mode);
    }
    let mut output = options.open(&file.destination)?;
    output.write_all(bytes)?;
    output.sync_all()?;
    drop(output);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(
            &file.destination,
            fs::Permissions::from_mode(file.entry.mode),
        )?;
    }
    let copied = read_exact_regular_file(
        &file.destination,
        file.entry.size_bytes,
        file.entry.mode,
        file.entry.size_bytes,
    )?;
    if sha256_bytes(&copied) != file.entry.sha256 {
        return Err(invalid(format!(
            "materialized destination digest mismatch: {}",
            file.destination.display()
        )));
    }
    Ok(())
}

fn read_exact_regular_file(
    path: &Path,
    expected_size: u64,
    expected_mode: u32,
    max_bytes: u64,
) -> EvalResult<Vec<u8>> {
    #[cfg(not(unix))]
    {
        let _ = (path, expected_size, expected_mode, max_bytes);
        return Err(invalid(
            "native isolation materialization requires Unix no-follow checks",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, OpenOptionsExt};

        let mut options = OpenOptions::new();
        options
            .read(true)
            .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW);
        let mut file = options.open(path)?;
        let metadata = file.metadata()?;
        if !metadata.is_file() || metadata.nlink() != 1 {
            return Err(invalid(format!(
                "manifest files must be regular and have exactly one hard link: {}",
                path.display()
            )));
        }
        if metadata.len() != expected_size || metadata.len() > max_bytes {
            return Err(invalid(format!(
                "manifest file size changed or exceeds its limit: {}",
                path.display()
            )));
        }
        if metadata.mode() & 0o777 != expected_mode {
            return Err(invalid(format!(
                "manifest file mode changed: {}",
                path.display()
            )));
        }
        let capacity = usize::try_from(expected_size)
            .map_err(|_| invalid("manifest file cannot fit in memory on this host"))?;
        let mut bytes = Vec::with_capacity(capacity);
        Read::by_ref(&mut file)
            .take(max_bytes + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 != expected_size {
            return Err(invalid(format!(
                "manifest file changed while it was read: {}",
                path.display()
            )));
        }
        Ok(bytes)
    }
}

fn validate_copied_content(
    bytes: &[u8],
    manifest: &IsolationCopyManifest,
    canonical_source_root: &Path,
) -> EvalResult<()> {
    let source_literal = canonical_source_root.to_string_lossy();
    if contains_bytes(bytes, source_literal.as_bytes()) {
        return Err(invalid(
            "copied content contains the canonical source-root path",
        ));
    }
    for literal in manifest
        .forbidden_markers
        .iter()
        .chain(manifest.forbidden_content_literals.iter())
    {
        if !literal.is_empty() && contains_bytes(bytes, literal.as_bytes()) {
            return Err(invalid(
                "copied content contains a forbidden marker or source literal",
            ));
        }
    }
    Ok(())
}

fn reject_credential_semantic_path(path: &Path, class: IsolationCopyClass) -> EvalResult<()> {
    for component in path.components() {
        let Component::Normal(component) = component else {
            continue;
        };
        let name = component.to_string_lossy().to_ascii_lowercase();
        let universally_forbidden = matches!(
            name.as_str(),
            "auth"
                | "auth.json"
                | "credential"
                | "credentials"
                | "credentials.json"
                | ".credentials.json"
                | "keychain"
                | "keyring"
                | "oauth"
                | "token"
                | "tokens"
                | "cookies"
                | ".netrc"
                | ".git-credentials"
                | ".claude.json"
                | ".ssh"
        ) || name.ends_with(".pem")
            || name.ends_with(".p12")
            || name.ends_with(".key")
            || name.ends_with(".token")
            || name.ends_with(".secret");
        let forbidden_native_history =
            matches!(
                class,
                IsolationCopyClass::CodexMemories | IsolationCopyClass::ClaudeAutoMemory
            ) && matches!(name.as_str(), "sessions" | "session" | "history");
        if universally_forbidden || forbidden_native_history {
            return Err(invalid(format!(
                "credential/session semantic path is not copyable: {}",
                path.display()
            )));
        }
    }
    Ok(())
}

/// Structured Codex permission profile frozen for the evaluation lane.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexPermissionProfile {
    pub config_layer_name: String,
    pub permission_profile_name: String,
    pub root_access: String,
    pub minimal_access: String,
    pub tmpdir_access: String,
    pub slash_tmp_access: String,
    pub evaluation_checkout: String,
    pub evaluation_checkout_access: String,
    #[serde(default)]
    pub additional_entries: Vec<CodexPermissionEntry>,
    pub network_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexPermissionEntry {
    pub path: String,
    pub access: String,
}

/// Exact Engram MCP/skill/instruction surface for one Engram-enabled evaluation lane.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IsolationEngramSurface {
    pub executable: String,
    pub executable_sha256: String,
    pub executable_version: String,
    pub mcp_tools_sha256: String,
    pub home: String,
    pub project: String,
    pub skill_path: String,
    pub skill_sha256: String,
}

/// Exact native-memory and Engram semantics appended to the Codex permission layer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexEvaluationArm {
    pub native_memory_enabled: bool,
    pub min_rollout_idle_hours: u32,
    pub engram: Option<IsolationEngramSurface>,
}

/// Provider-free model-command probe captured using the exact frozen permission profile.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexPermissionProbe {
    pub config_layer_name: String,
    pub permission_profile_name: String,
    pub allowed_read_succeeded: bool,
    pub sibling_traversal_denied: bool,
    pub direct_forbidden_read_denied: bool,
    pub child_shell_forbidden_read_denied: bool,
    pub write_denied: bool,
    pub network_denied: bool,
    pub network_sandbox_denial_observed: bool,
}

pub fn validate_codex_permission_profile(
    profile: &CodexPermissionProfile,
    evaluation_root: &Path,
    probe: &CodexPermissionProbe,
) -> EvalResult<()> {
    if profile.config_layer_name.trim().is_empty()
        || profile.permission_profile_name.trim().is_empty()
        || profile.root_access != "deny"
        || profile.minimal_access != "read"
        || profile.tmpdir_access != "deny"
        || profile.slash_tmp_access != "deny"
        || profile.evaluation_checkout_access != "read"
        || profile.network_enabled
        || probe.config_layer_name != profile.config_layer_name
        || probe.permission_profile_name != profile.permission_profile_name
    {
        return Err(invalid(
            "Codex permission profile is not the frozen deny/read shape",
        ));
    }
    let evaluation_root = canonical_directory(evaluation_root)?;
    let checkout = canonical_directory(Path::new(&profile.evaluation_checkout))?;
    if !checkout.starts_with(&evaluation_root) {
        return Err(invalid("Codex permission checkout is outside the enclave"));
    }
    if !profile.additional_entries.is_empty() {
        return Err(invalid(
            "Codex permission profile must grant only :minimal and the exact checkout",
        ));
    }
    if !(probe.allowed_read_succeeded
        && probe.sibling_traversal_denied
        && probe.direct_forbidden_read_denied
        && probe.child_shell_forbidden_read_denied
        && probe.write_denied
        && probe.network_denied
        && probe.network_sandbox_denial_observed)
    {
        return Err(invalid(
            "Codex permission-profile negative probe did not pass",
        ));
    }
    Ok(())
}

/// Render the exact config-layer bytes used to select the named Codex permission profile.
pub fn generate_codex_permission_profile_toml(
    profile: &CodexPermissionProfile,
) -> EvalResult<String> {
    if profile.config_layer_name.is_empty()
        || profile.permission_profile_name.is_empty()
        || !profile
            .config_layer_name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        || !profile
            .permission_profile_name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        || profile.root_access != "deny"
        || profile.minimal_access != "read"
        || profile.tmpdir_access != "deny"
        || profile.slash_tmp_access != "deny"
        || profile.evaluation_checkout_access != "read"
        || !profile.additional_entries.is_empty()
        || profile.network_enabled
        || !Path::new(&profile.evaluation_checkout).is_absolute()
    {
        return Err(invalid(
            "invalid Codex permission profile for TOML generation",
        ));
    }
    let checkout = toml_basic_string(&profile.evaluation_checkout)?;
    Ok(format!(
        "approval_policy = \"never\"\n\
         default_permissions = \"{}\"\n\
         \n\
         [permissions.{}.filesystem]\n\
         \":root\" = \"deny\"\n\
         \":minimal\" = \"read\"\n\
         \":tmpdir\" = \"deny\"\n\
         \":slash_tmp\" = \"deny\"\n\
         \"{}\" = \"read\"\n\
         \n\
         [permissions.{}.network]\n\
         enabled = false\n",
        profile.permission_profile_name,
        profile.permission_profile_name,
        checkout,
        profile.permission_profile_name
    ))
}

/// Render the complete Codex 0.153.3 evaluation layer, including exact lane semantics.
pub fn generate_codex_evaluation_config_toml(
    profile: &CodexPermissionProfile,
    arm: &CodexEvaluationArm,
) -> EvalResult<String> {
    if arm.min_rollout_idle_hours != 0 {
        return Err(invalid(
            "Codex evaluation must freeze min_rollout_idle_hours to zero",
        ));
    }
    let mut root =
        String::from("approval_policy = \"never\"\ncli_auth_credentials_store = \"file\"\n");
    if let Some(engram) = &arm.engram {
        validate_engram_surface(engram)?;
        let instructions = read_bound_file(Path::new(&engram.skill_path), &engram.skill_sha256)?;
        let instructions = String::from_utf8(instructions)
            .map_err(|_| invalid("Engram skill/instructions must be UTF-8"))?;
        root.push_str(&format!(
            "developer_instructions = {}\n",
            serde_json::to_string(&instructions)?
        ));
    }
    root.push_str(&format!(
        "default_permissions = {}\n\n\
         [features]\n\
         memories = {}\n\n\
         [memories]\n\
         generate_memories = false\n\
         use_memories = {}\n\
         disable_on_external_context = false\n\
         min_rollout_idle_hours = 0\n",
        serde_json::to_string(&profile.permission_profile_name)?,
        arm.native_memory_enabled,
        arm.native_memory_enabled,
    ));
    if let Some(engram) = &arm.engram {
        let args = ["serve", "--project", &engram.project, "--profile", "agent"];
        root.push_str(&format!(
            "\n[mcp_servers.engram]\n\
             command = {}\n\
             args = {}\n\
             env = {{ ENGRAM_HOME = {} }}\n\
             required = true\n\
             startup_timeout_sec = 60\n\n\
             [[skills.config]]\n\
             path = {}\n\
             enabled = true\n",
            serde_json::to_string(&engram.executable)?,
            serde_json::to_string(&args)?,
            serde_json::to_string(&engram.home)?,
            serde_json::to_string(&engram.skill_path)?,
        ));
    }
    let permission = generate_codex_permission_profile_toml(profile)?;
    let permission_header = format!(
        "[permissions.{}.filesystem]",
        profile.permission_profile_name
    );
    let (_, permission) = permission
        .split_once(&permission_header)
        .ok_or_else(|| invalid("Codex permission profile section drifted"))?;
    root.push('\n');
    root.push_str(&permission_header);
    root.push_str(permission);
    toml::from_str::<toml::Value>(&root).map_err(|error| {
        invalid(format!(
            "generated Codex evaluation TOML is invalid: {error}"
        ))
    })?;
    Ok(root)
}

fn validate_engram_surface(surface: &IsolationEngramSurface) -> EvalResult<()> {
    require_normalized_absolute_path(Path::new(&surface.executable), "Engram executable")?;
    require_normalized_absolute_path(Path::new(&surface.home), "Engram home")?;
    require_normalized_absolute_path(Path::new(&surface.skill_path), "Engram skill")?;
    if surface.project.trim().is_empty()
        || surface.project.len() > 128
        || !surface
            .project
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        || matches!(surface.project.as_str(), "." | "..")
        || surface
            .project
            .contains(|character: char| character.is_control())
        || !is_sha256(&surface.executable_sha256)
        || surface.executable_version.trim().is_empty()
        || surface.executable_version != surface.executable_version.trim()
        || !is_sha256(&surface.mcp_tools_sha256)
        || !is_sha256(&surface.skill_sha256)
    {
        return Err(invalid("invalid exact Engram launch surface"));
    }
    let actual_executable = read_regular_file_digest(Path::new(&surface.executable))?;
    if actual_executable != surface.executable_sha256 {
        return Err(invalid("Engram executable digest drifted"));
    }
    let _ = read_bound_file(Path::new(&surface.skill_path), &surface.skill_sha256)?;
    Ok(())
}

/// One exact provider-free `codex sandbox` enforcement probe.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexSandboxProbeCommand {
    pub label: String,
    /// Canonical canary path bound to this probe; network probes have no file canary.
    pub probe_path: Option<String>,
    pub argv: Vec<String>,
    pub expect_success: bool,
    pub require_sandbox_denial_evidence: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub must_remain_absent: Option<String>,
}

/// Build the exact provider-free filesystem/write/network probe sequence for Codex 0.153.3.
///
/// The required curl probe is paired with a localhost socket probe: DNS failure alone is not
/// accepted as evidence that the permission profile caused the network failure.
#[allow(clippy::too_many_arguments)]
pub fn codex_sandbox_probe_commands(
    codex_executable: &Path,
    profile: &CodexPermissionProfile,
    allowed_canary: &Path,
    sibling_traversal_canary: &Path,
    forbidden_source_canary: &Path,
    write_probe: &Path,
    network_policy_port: u16,
) -> EvalResult<Vec<CodexSandboxProbeCommand>> {
    if !codex_executable.is_absolute()
        || !allowed_canary.is_absolute()
        || !sibling_traversal_canary.is_absolute()
        || !forbidden_source_canary.is_absolute()
        || !write_probe.is_absolute()
    {
        return Err(invalid("Codex sandbox probe paths must be absolute"));
    }
    let checkout = Path::new(&profile.evaluation_checkout);
    let canonical_checkout = canonical_directory(checkout)?;
    if canonical_checkout.display().to_string() != profile.evaluation_checkout {
        return Err(invalid("Codex sandbox checkout must be canonical"));
    }
    for canary in [
        allowed_canary,
        sibling_traversal_canary,
        forbidden_source_canary,
    ] {
        validate_probe_read_canary(canary)?;
    }
    require_normalized_absolute_path(write_probe, "Codex write probe")?;
    let write_parent = canonical_directory(
        write_probe
            .parent()
            .ok_or_else(|| invalid("Codex write probe has no parent"))?,
    )?;
    if !allowed_canary.starts_with(checkout)
        || !write_probe.starts_with(checkout)
        || forbidden_source_canary.starts_with(checkout)
        || write_parent != canonical_checkout
        || write_probe.exists()
        || network_policy_port == 0
    {
        return Err(invalid(
            "Codex sandbox probe canaries violate the boundary contract",
        ));
    }
    let traversal = lexical_relative_path(checkout, sibling_traversal_canary)?;
    if !traversal
        .components()
        .any(|component| component == Component::ParentDir)
    {
        return Err(invalid("Codex sibling probe does not exercise traversal"));
    }
    let prefix = vec![
        codex_executable.display().to_string(),
        "sandbox".to_string(),
        "--profile".to_string(),
        profile.config_layer_name.clone(),
        "--permission-profile".to_string(),
        profile.permission_profile_name.clone(),
        "--cd".to_string(),
        profile.evaluation_checkout.clone(),
        "--log-denials".to_string(),
        "--".to_string(),
    ];
    let command = |label: &str,
                   probe_path: Option<String>,
                   tail: Vec<String>,
                   expect_success: bool,
                   denial: bool,
                   absent: Option<String>| {
        let mut argv = prefix.clone();
        argv.extend(tail);
        CodexSandboxProbeCommand {
            label: label.to_string(),
            probe_path,
            argv,
            expect_success,
            require_sandbox_denial_evidence: denial,
            must_remain_absent: absent,
        }
    };
    Ok(vec![
        command(
            "allowed_read",
            Some(allowed_canary.display().to_string()),
            vec!["/bin/cat".to_string(), allowed_canary.display().to_string()],
            true,
            false,
            None,
        ),
        command(
            "sibling_traversal",
            Some(sibling_traversal_canary.display().to_string()),
            vec!["/bin/cat".to_string(), traversal.display().to_string()],
            false,
            true,
            None,
        ),
        command(
            "direct_forbidden_read",
            Some(forbidden_source_canary.display().to_string()),
            vec![
                "/bin/cat".to_string(),
                forbidden_source_canary.display().to_string(),
            ],
            false,
            true,
            None,
        ),
        command(
            "child_shell_forbidden_read",
            Some(forbidden_source_canary.display().to_string()),
            vec![
                "/bin/sh".to_string(),
                "-c".to_string(),
                "exec /bin/cat \"$1\"".to_string(),
                "probe".to_string(),
                forbidden_source_canary.display().to_string(),
            ],
            false,
            true,
            None,
        ),
        command(
            "write_denial",
            Some(write_probe.display().to_string()),
            vec![
                "/usr/bin/touch".to_string(),
                write_probe.display().to_string(),
            ],
            false,
            true,
            Some(write_probe.display().to_string()),
        ),
        command(
            "network_denial",
            None,
            vec![
                "/usr/bin/curl".to_string(),
                "-fsS".to_string(),
                "--connect-timeout".to_string(),
                "2".to_string(),
                "https://example.com".to_string(),
            ],
            false,
            false,
            None,
        ),
        command(
            "network_policy_denial",
            None,
            vec![
                "/usr/bin/nc".to_string(),
                "-z".to_string(),
                "-w".to_string(),
                "1".to_string(),
                "127.0.0.1".to_string(),
                network_policy_port.to_string(),
            ],
            false,
            false,
            None,
        ),
        command(
            "network_errno_denial",
            None,
            vec![
                "/usr/bin/nc".to_string(),
                "-z".to_string(),
                "-v".to_string(),
                "-w".to_string(),
                "1".to_string(),
                "127.0.0.1".to_string(),
                network_policy_port.to_string(),
            ],
            false,
            true,
            None,
        ),
    ])
}

/// Sanitized result of one real provider-free sandbox command.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IsolationSandboxCommandAudit {
    pub label: String,
    pub argv_sha256: String,
    pub exit_code: i32,
    pub succeeded: bool,
    pub sandbox_denial_evidence: bool,
    pub stdout_sha256: String,
    pub stderr_sha256: String,
}

/// Real provider-free Codex sandbox probe result. Command output bytes are never retained.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexSandboxProbeAudit {
    pub valid: bool,
    /// SHA-256 of the exact complete selected evaluation config layer used by the probes.
    pub profile_sha256: String,
    pub profile: CodexPermissionProbe,
    /// Exact unwrapped `/usr/bin/nc` control under the same cleared environment.
    pub network_control: IsolationSandboxCommandAudit,
    pub commands: Vec<IsolationSandboxCommandAudit>,
}

/// Execute the exact Codex sandbox probe matrix without invoking `codex exec` or a provider.
pub fn execute_codex_sandbox_probes(
    profile: &CodexPermissionProfile,
    arm: &CodexEvaluationArm,
    evaluation_root: &Path,
    commands: &[CodexSandboxProbeCommand],
    environment: &BTreeMap<String, String>,
) -> EvalResult<CodexSandboxProbeAudit> {
    let evaluation_root = canonical_directory(evaluation_root)?;
    validate_codex_probe_environment(environment, &evaluation_root)?;
    validate_codex_probe_command_set(profile, commands)?;
    let profile_path = Path::new(&environment["CODEX_HOME"])
        .join(format!("{}.config.toml", profile.config_layer_name));
    let expected_profile = generate_codex_evaluation_config_toml(profile, arm)?;
    require_exact_file_bytes(&profile_path, expected_profile.as_bytes(), true)?;

    let mut audits = Vec::with_capacity(commands.len());
    let mut network_control = None;
    for command in commands {
        if command.label == "network_policy_denial" {
            network_control = Some(execute_exact_network_control(
                &command.argv,
                codex_probe_prefix_len(profile),
                environment,
            )?);
        }
        let audit = run_sanitized_probe_command(
            &command.label,
            &command.argv,
            environment,
            command.require_sandbox_denial_evidence,
        )?;
        if audit.succeeded != command.expect_success {
            return Err(invalid(format!(
                "Codex sandbox probe had the wrong status: {}",
                command.label
            )));
        }
        if command.require_sandbox_denial_evidence && !audit.sandbox_denial_evidence {
            return Err(invalid(format!(
                "Codex sandbox probe lacked policy-denial evidence: {}",
                command.label
            )));
        }
        if command
            .must_remain_absent
            .as_ref()
            .is_some_and(|path| Path::new(path).exists())
        {
            return Err(invalid("Codex denied write probe created its target"));
        }
        audits.push(audit);
    }
    let result = |label: &str| {
        audits
            .iter()
            .find(|audit| audit.label == label)
            .ok_or_else(|| invalid(format!("missing Codex sandbox probe result: {label}")))
    };
    let permission_probe = CodexPermissionProbe {
        config_layer_name: profile.config_layer_name.clone(),
        permission_profile_name: profile.permission_profile_name.clone(),
        allowed_read_succeeded: result("allowed_read")?.succeeded,
        sibling_traversal_denied: !result("sibling_traversal")?.succeeded,
        direct_forbidden_read_denied: !result("direct_forbidden_read")?.succeeded,
        child_shell_forbidden_read_denied: !result("child_shell_forbidden_read")?.succeeded,
        write_denied: !result("write_denial")?.succeeded,
        network_denied: !result("network_denial")?.succeeded
            && !result("network_policy_denial")?.succeeded
            && !result("network_errno_denial")?.succeeded,
        network_sandbox_denial_observed: result("network_errno_denial")?.sandbox_denial_evidence,
    };
    validate_codex_permission_profile(profile, &evaluation_root, &permission_probe)?;
    Ok(CodexSandboxProbeAudit {
        valid: true,
        profile_sha256: sha256_bytes(expected_profile.as_bytes()),
        profile: permission_probe,
        network_control: network_control
            .ok_or_else(|| invalid("Codex sandbox probe lacked its network control"))?,
        commands: audits,
    })
}

fn codex_probe_prefix_len(profile: &CodexPermissionProfile) -> usize {
    // executable, sandbox, --profile, layer, --permission-profile, permission, --cd, cwd,
    // --log-denials, --
    let _ = profile;
    10
}

fn validate_codex_probe_environment(
    environment: &BTreeMap<String, String>,
    evaluation_root: &Path,
) -> EvalResult<()> {
    let expected_keys = BTreeSet::from([
        "CODEX_HOME",
        "DO_NOT_TRACK",
        "HOME",
        "LANG",
        "LC_ALL",
        "NO_COLOR",
        "OTEL_SDK_DISABLED",
        "PATH",
        "TMPDIR",
    ]);
    if environment
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>()
        != expected_keys
        || environment.get("PATH").map(String::as_str) != Some("/usr/bin:/bin")
        || environment.get("LANG").map(String::as_str) != Some("C.UTF-8")
        || environment.get("LC_ALL").map(String::as_str) != Some("C.UTF-8")
        || environment.get("NO_COLOR").map(String::as_str) != Some("1")
        || environment.get("DO_NOT_TRACK").map(String::as_str) != Some("1")
        || environment.get("OTEL_SDK_DISABLED").map(String::as_str) != Some("true")
    {
        return Err(invalid(
            "Codex sandbox probe environment is not the exact cleared allowlist",
        ));
    }
    for (key, relative) in [
        ("HOME", "home"),
        ("TMPDIR", "tmp"),
        ("CODEX_HOME", "codex-home"),
    ] {
        let value = environment
            .get(key)
            .ok_or_else(|| invalid(format!("Codex sandbox probe environment lacks {key}")))?;
        let path = canonical_directory(Path::new(value))?;
        if path != evaluation_root.join(relative) || path.display().to_string() != *value {
            return Err(invalid(format!(
                "Codex sandbox probe {key} is not its exact enclave directory"
            )));
        }
    }
    Ok(())
}

fn validate_codex_probe_command_set(
    profile: &CodexPermissionProfile,
    commands: &[CodexSandboxProbeCommand],
) -> EvalResult<()> {
    let labels = [
        "allowed_read",
        "sibling_traversal",
        "direct_forbidden_read",
        "child_shell_forbidden_read",
        "write_denial",
        "network_denial",
        "network_policy_denial",
        "network_errno_denial",
    ];
    if commands.len() != labels.len()
        || commands
            .iter()
            .map(|command| command.label.as_str())
            .ne(labels)
    {
        return Err(invalid("Codex sandbox probe matrix is not exact"));
    }
    let expected_prefix = [
        "sandbox",
        "--profile",
        profile.config_layer_name.as_str(),
        "--permission-profile",
        profile.permission_profile_name.as_str(),
        "--cd",
        profile.evaluation_checkout.as_str(),
        "--log-denials",
        "--",
    ];
    let mut executable = None;
    for command in commands {
        let first = command
            .argv
            .first()
            .ok_or_else(|| invalid("Codex sandbox probe argv is empty"))?;
        if !Path::new(first).is_absolute()
            || *executable.get_or_insert(first) != first
            || command.argv.len() <= expected_prefix.len()
            || command.argv[1..=expected_prefix.len()]
                .iter()
                .map(String::as_str)
                .ne(expected_prefix)
        {
            return Err(invalid("Codex sandbox probe prefix is not frozen"));
        }
        let tail = &command.argv[expected_prefix.len() + 1..];
        let valid_tail = match command.label.as_str() {
            "allowed_read" | "sibling_traversal" | "direct_forbidden_read" => {
                tail.len() == 2 && tail[0] == "/bin/cat"
            }
            "child_shell_forbidden_read" => {
                tail.len() == 5
                    && tail[0] == "/bin/sh"
                    && tail[1] == "-c"
                    && tail[2] == "exec /bin/cat \"$1\""
                    && tail[3] == "probe"
            }
            "write_denial" => tail.len() == 2 && tail[0] == "/usr/bin/touch",
            "network_denial" => {
                tail == [
                    "/usr/bin/curl",
                    "-fsS",
                    "--connect-timeout",
                    "2",
                    "https://example.com",
                ]
            }
            "network_policy_denial" => {
                tail.len() == 6
                    && tail[..5] == ["/usr/bin/nc", "-z", "-w", "1", "127.0.0.1"]
                    && tail[5].parse::<u16>().is_ok_and(|port| port != 0)
            }
            "network_errno_denial" => {
                tail.len() == 7
                    && tail[..6] == ["/usr/bin/nc", "-z", "-v", "-w", "1", "127.0.0.1"]
                    && tail[6].parse::<u16>().is_ok_and(|port| port != 0)
            }
            _ => false,
        };
        if !valid_tail {
            return Err(invalid("Codex sandbox probe command tail is not frozen"));
        }
        validate_codex_probe_path(profile, command, tail)?;
    }
    if !commands[0].expect_success
        || commands[0].require_sandbox_denial_evidence
        || commands[1..].iter().any(|command| command.expect_success)
        || commands[1..5]
            .iter()
            .any(|command| !command.require_sandbox_denial_evidence)
        || commands[5].require_sandbox_denial_evidence
        || commands[6].require_sandbox_denial_evidence
        || !commands[7].require_sandbox_denial_evidence
        || commands[4].must_remain_absent.as_deref() != commands[4].argv.last().map(String::as_str)
        || commands
            .iter()
            .enumerate()
            .any(|(index, command)| index != 4 && command.must_remain_absent.is_some())
    {
        return Err(invalid("Codex sandbox probe expectations are not frozen"));
    }
    if commands[2].probe_path != commands[3].probe_path
        || commands[6].argv.last() != commands[7].argv.last()
    {
        return Err(invalid(
            "paired Codex probes do not target the same forbidden canary or endpoint",
        ));
    }
    Ok(())
}

fn validate_codex_probe_path(
    profile: &CodexPermissionProfile,
    command: &CodexSandboxProbeCommand,
    tail: &[String],
) -> EvalResult<()> {
    let checkout = Path::new(&profile.evaluation_checkout);
    match command.label.as_str() {
        "allowed_read" => {
            let path = command
                .probe_path
                .as_deref()
                .ok_or_else(|| invalid("Codex allowed probe lacks its canary path"))?;
            if tail.get(1).map(String::as_str) != Some(path)
                || !Path::new(path).starts_with(checkout)
            {
                return Err(invalid(
                    "Codex allowed probe canary is not bound to checkout",
                ));
            }
            validate_probe_read_canary(Path::new(path))
        }
        "sibling_traversal" => {
            let path = command
                .probe_path
                .as_deref()
                .ok_or_else(|| invalid("Codex sibling probe lacks its canary path"))?;
            let path = Path::new(path);
            validate_probe_read_canary(path)?;
            let relative = lexical_relative_path(checkout, path)?;
            if path.starts_with(checkout)
                || tail.get(1).map(Path::new) != Some(relative.as_path())
                || !relative
                    .components()
                    .any(|component| component == Component::ParentDir)
            {
                return Err(invalid("Codex sibling traversal canary is not exact"));
            }
            Ok(())
        }
        "direct_forbidden_read" | "child_shell_forbidden_read" => {
            let path = command
                .probe_path
                .as_deref()
                .ok_or_else(|| invalid("Codex forbidden probe lacks its canary path"))?;
            if tail.last().map(String::as_str) != Some(path)
                || Path::new(path).starts_with(checkout)
            {
                return Err(invalid("Codex forbidden canary is not outside checkout"));
            }
            validate_probe_read_canary(Path::new(path))
        }
        "write_denial" => {
            let path = command
                .probe_path
                .as_deref()
                .ok_or_else(|| invalid("Codex write probe lacks its target path"))?;
            let path = Path::new(path);
            if tail.last().map(Path::new) != Some(path)
                || command.must_remain_absent.as_deref() != path.to_str()
                || !path.starts_with(checkout)
                || path.exists()
            {
                return Err(invalid("Codex write probe target is not exact"));
            }
            Ok(())
        }
        "network_denial" | "network_policy_denial" | "network_errno_denial" => {
            if command.probe_path.is_some() {
                return Err(invalid("Codex network probes may not name a file canary"));
            }
            Ok(())
        }
        _ => Err(invalid("unknown Codex sandbox probe")),
    }
}

/// Inputs used to generate the raw macOS Seatbelt wrapper for Claude only.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeSeatbeltSpec {
    pub schema_version: u32,
    pub sandbox_exec: String,
    pub transport: ClaudeNarrowTransportContract,
    /// Exact source/run/teaching paths denied for raw reads in this lane. The trusted Claude client
    /// must retain credential access; the restricted CLI tool surface, not Seatbelt, mediates it.
    pub forbidden_read_paths: Vec<String>,
    /// Exact source/run/teaching/native/Engram paths denied for raw writes in this lane.
    pub forbidden_write_paths: Vec<String>,
}

/// Exact loopback-only transports allowed through the outer Claude Seatbelt boundary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeNarrowTransportContract {
    pub provider_proxy_host: String,
    pub provider_proxy_port: u16,
    pub provider_connect_authority: String,
    pub engram_daemon_port: Option<u16>,
    pub max_connect_header_bytes: u64,
    pub max_connect_header_count: u16,
    pub max_connect_line_bytes: u16,
    pub max_connect_requests: u32,
    /// Maximum number of exact system-resolver results accepted for the provider hostname.
    pub max_resolved_addresses: u16,
    /// Per-address TCP connect deadline for the runner-owned provider relay.
    pub connect_timeout_ms: u64,
    /// One wall-clock ceiling for accept, header, DNS, all connects, and both tunnel directions.
    pub total_timeout_ms: u64,
    pub max_tunnel_bytes_each_direction: u64,
    pub idle_timeout_ms: u64,
}

/// Exact invocation/process identity that a live runner must generate before accepting CONNECT.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeProviderProxySessionBinding {
    pub invocation_id: String,
    pub launch_sha256: String,
    pub seatbelt_profile_sha256: String,
    /// Salted digest of the per-invocation Basic proxy credential; never the raw credential.
    pub proxy_auth_sha256: String,
    pub provider_process_id: u32,
    pub proxy_process_id: u32,
    pub proxy_started_unix_ms: u64,
    pub provider_started_unix_ms: u64,
}

/// Non-serializable, zeroizing per-invocation proxy credential.
pub struct ClaudeProxyInvocationSecret {
    invocation_id: String,
    secret_hex: Zeroizing<String>,
    salted_sha256: String,
}

impl std::fmt::Debug for ClaudeProxyInvocationSecret {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ClaudeProxyInvocationSecret")
            .field("invocation_id", &self.invocation_id)
            .field("secret_hex", &"<redacted>")
            .field("salted_sha256", &self.salted_sha256)
            .finish()
    }
}

impl ClaudeProxyInvocationSecret {
    #[must_use]
    pub fn salted_sha256(&self) -> &str {
        &self.salted_sha256
    }

    /// Build the only runtime HTTPS proxy value. The returned value zeroizes on drop and must not
    /// be copied into `IsolationLaunchContract`, argv, logs, or persisted receipts.
    pub fn runtime_https_proxy(
        &self,
        transport: &ClaudeNarrowTransportContract,
    ) -> EvalResult<Zeroizing<String>> {
        validate_claude_transport(transport)?;
        Ok(Zeroizing::new(format!(
            "http://engram-proxy:{}@127.0.0.1:{}",
            self.secret_hex.as_str(),
            transport.provider_proxy_port
        )))
    }

    fn proxy_authorization_value(&self) -> Zeroizing<String> {
        Zeroizing::new(format!(
            "Basic {}",
            base64_standard(format!("engram-proxy:{}", self.secret_hex.as_str()).as_bytes())
        ))
    }
}

/// Generate exactly 256 bits from `/dev/urandom`; the raw secret never enters a serializable type.
#[cfg(target_os = "macos")]
pub fn generate_claude_proxy_invocation_secret(
    invocation_id: &str,
) -> EvalResult<ClaudeProxyInvocationSecret> {
    use std::os::unix::fs::FileTypeExt;

    validate_invocation_id(invocation_id)?;
    let mut random = Zeroizing::new([0_u8; 32]);
    let mut urandom = OpenOptions::new().read(true).open("/dev/urandom")?;
    let metadata = urandom.metadata()?;
    if !metadata.file_type().is_char_device() {
        return Err(invalid("/dev/urandom is not a character device"));
    }
    urandom.read_exact(random.as_mut())?;
    let secret_hex: Zeroizing<String> =
        Zeroizing::new(random.iter().map(|byte| format!("{byte:02x}")).collect());
    let salted_sha256 = proxy_secret_digest(invocation_id, secret_hex.as_bytes());
    Ok(ClaudeProxyInvocationSecret {
        invocation_id: invocation_id.to_string(),
        secret_hex,
        salted_sha256,
    })
}

#[cfg(not(target_os = "macos"))]
pub fn generate_claude_proxy_invocation_secret(
    _invocation_id: &str,
) -> EvalResult<ClaudeProxyInvocationSecret> {
    Err(invalid(
        "Claude proxy secret generation is currently frozen for macOS only",
    ))
}

/// Hash-bound evidence that the lane's Engram daemon was prestarted outside Claude's Seatbelt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeEngramDaemonAttestation {
    pub valid: bool,
    pub host: String,
    pub port: u16,
    pub project: String,
    pub home: String,
    pub process_id: u32,
    pub process_executable_path: String,
    pub executable_sha256: String,
    pub daemon_port_file: IsolationFileBinding,
    pub daemon_pid_file: IsolationFileBinding,
    pub daemon_metadata_file: IsolationFileBinding,
    pub started_outside_seatbelt: bool,
    pub process_identity_observed_by_runner: bool,
    pub healthy: bool,
    pub health_schema_version: u64,
    pub mcp_contract_version: u64,
    pub mcp_tools_sha256: String,
    pub mcp_protocol_version: String,
    pub health_build_sha: Option<String>,
    pub auth_required: bool,
    pub storage_status: String,
    pub storage_ready: bool,
    pub storage_available_bytes: u64,
    pub storage_required_bytes: u64,
    pub health_response_sha256: String,
    pub health_contract_sha256: String,
    pub health_observed_by_runner: bool,
    pub observed_unix_ms: u64,
    pub raw_bearer_token_read: bool,
    pub attestation_sha256: String,
}

/// Sanitized CONNECT-only proxy receipt. No request headers, credential bytes, or body are fields.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeProviderProxyReceipt {
    pub valid: bool,
    pub target: String,
    pub accepted_connect_count: u32,
    pub rejected_request_count: u32,
    pub client_to_upstream_bytes: u64,
    pub upstream_to_client_bytes: u64,
    pub started_unix_ms: u64,
    pub finished_unix_ms: u64,
    pub raw_headers_or_body_retained: bool,
    pub session: ClaudeProviderProxySessionBinding,
    pub receipt_sha256: String,
}

/// Execution-owned, provider-free proof of the proxy/Seatbelt/fake-upstream composition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeProviderFreeTransportProbeAudit {
    pub valid: bool,
    pub session: ClaudeProviderProxySessionBinding,
    pub receipt: ClaudeProviderProxyReceipt,
    pub sandboxed_client: IsolationSandboxCommandAudit,
    pub missing_proxy_secret_rejected: bool,
    pub wrong_proxy_secret_rejected: bool,
    /// Intentional negative connections are outside the accepted-session receipt by design.
    pub intentional_negative_attempt_count: u32,
    pub fake_upstream_received_bytes: u64,
    pub fake_upstream_sent_bytes: u64,
    pub raw_transport_bytes_retained: bool,
    pub audit_sha256: String,
}

/// Exact, content-free DNS and TCP-selection evidence generated by the production relay.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeProviderResolutionAudit {
    pub valid: bool,
    pub host: String,
    pub port: u16,
    pub resolver_argv_sha256: String,
    pub resolver_stdout_sha256: String,
    pub resolver_stderr_sha256: String,
    pub resolver_exit_code: i32,
    pub resolver_output_bytes: u64,
    pub resolver_observed_by_runner: bool,
    pub resolved_addresses: Vec<String>,
    pub selected_address: String,
    pub resolution_started_unix_ms: u64,
    pub resolution_finished_unix_ms: u64,
    pub raw_dns_response_retained: bool,
    pub audit_sha256: String,
}

/// Execution-owned result of one production CONNECT relay session.
///
/// This receipt deliberately remains non-authorizing until it is bound into the shared phase
/// runner and independently audited. It contains no HTTP headers, tunnel bytes, or credentials.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeProductionRelayAudit {
    pub valid: bool,
    pub authorizes_provider_execution: bool,
    pub provider_executable_path: String,
    pub provider_executable_sha256: String,
    pub session: ClaudeProviderProxySessionBinding,
    pub resolution: ClaudeProviderResolutionAudit,
    pub receipt: ClaudeProviderProxyReceipt,
    pub runner_observed_provider_process: bool,
    /// Credential possession is not kernel peer-PID attribution; malicious same-UID processes are
    /// explicitly outside this local-host claim.
    pub malicious_same_uid_process_excluded: bool,
    pub total_timeout_ms: u64,
    pub deadline_unix_ms: u64,
    pub completed_before_deadline: bool,
    pub raw_transport_bytes_retained: bool,
    pub audit_sha256: String,
}

/// One exact macOS source through which Claude managed policy may bypass cleared user settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeManagedPolicySourceAudit {
    pub role: String,
    pub path: String,
    pub present: bool,
    pub sha256: Option<String>,
    pub size_bytes: Option<u64>,
    pub mode: Option<u32>,
    pub owner_uid: Option<u32>,
    /// Security-relevant JSON keys only; values and policy content are never retained here.
    pub security_relevant_keys: Vec<String>,
}

/// Runner-observed closed list of macOS Claude managed-policy sources.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeManagedPolicyAttestation {
    pub valid: bool,
    pub sources: Vec<ClaudeManagedPolicySourceAudit>,
    pub all_known_sources_observed: bool,
    pub server_managed_effective_policy_unobserved: bool,
    pub opaque_managed_preferences_present: bool,
    pub api_key_helper_present: bool,
    pub policy_helper_present: bool,
    pub external_execution_policy_present: bool,
    pub safe_for_local_provider_execution: bool,
    pub confounded_exploratory_only: bool,
    pub raw_policy_contents_retained: bool,
    pub observed_unix_ms: u64,
    pub attestation_sha256: String,
}

/// Sanitized classification of one bounded proxy request header.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeConnectRequestAudit {
    pub accepted: bool,
    pub target: String,
    pub method: String,
    pub header_bytes: u64,
    pub body_bytes: u64,
    pub raw_request_retained: bool,
}

/// Sanitized provider-free result of compiling and probing the generated Seatbelt profile.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeSeatbeltProbe {
    pub sandbox_exec_present: bool,
    pub profile_compiled: bool,
    pub allowed_evaluation_canary_read: bool,
    pub provider_proxy_loopback_allowed: bool,
    pub engram_loopback_allowed: Option<bool>,
    pub forbidden_reads: BTreeMap<String, IsolationProbeOutcome>,
    pub forbidden_writes: BTreeMap<String, IsolationProbeOutcome>,
    pub network_denied: bool,
    pub adjacent_loopback_denied: bool,
    pub external_network_denied: bool,
    pub unexpected_bind_denied: bool,
    pub network_sandbox_denial_observed: bool,
}

/// A harmless canary tied to one exact Seatbelt-protected path.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeSeatbeltProbeTarget {
    pub protected_path: String,
    pub canary_path: String,
}

/// One exact provider-free `/usr/bin/sandbox-exec` probe.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeSeatbeltProbeCommand {
    pub label: String,
    pub protected_path: Option<String>,
    pub argv: Vec<String>,
    pub expect_success: bool,
    pub require_sandbox_denial_evidence: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub must_remain_absent: Option<String>,
}

/// Real provider-free Claude Seatbelt probe result. Command output bytes are never retained.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeSeatbeltProbeAudit {
    pub valid: bool,
    pub profile_sha256: String,
    pub profile: ClaudeSeatbeltProbe,
    /// Exact unwrapped `/usr/bin/nc` control under the same cleared environment.
    pub network_control: IsolationSandboxCommandAudit,
    pub commands: Vec<IsolationSandboxCommandAudit>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IsolationProbeOutcome {
    Allowed,
    PermissionDenied,
    Missing,
    OtherFailure,
}

/// Generate a pinned, fail-closed-on-parse Seatbelt profile for the Claude wrapper.
///
/// The profile denies network, the exact source-side read paths, and writes to copied semantic
/// state. Other reads remain governed by Claude's `--restricted` file-tool boundary. This explicit
/// composition is intentional: raw Seatbelt must not wrap Codex because Codex installs its own
/// sandbox.
pub fn generate_claude_seatbelt_profile(spec: &ClaudeSeatbeltSpec) -> EvalResult<String> {
    validate_seatbelt_spec(spec)?;
    let mut profile = String::from("(version 1)\n(allow default)\n(deny network*)\n");
    profile.push_str(&format!(
        "(allow network-outbound (remote ip \"localhost:{}\"))\n",
        spec.transport.provider_proxy_port
    ));
    if let Some(port) = spec.transport.engram_daemon_port {
        profile.push_str(&format!(
            "(allow network-outbound (remote ip \"localhost:{port}\"))\n"
        ));
    }
    for path in &spec.forbidden_read_paths {
        profile.push_str(&format!(
            "(deny file-read* (subpath \"{}\"))\n",
            seatbelt_escape(path)?
        ));
    }
    for path in &spec.forbidden_write_paths {
        profile.push_str(&format!(
            "(deny file-write* (subpath \"{}\"))\n",
            seatbelt_escape(path)?
        ));
    }
    Ok(profile)
}

/// Build the exact compile/read/write/network probe matrix for the generated Claude profile.
pub fn claude_seatbelt_probe_commands(
    spec: &ClaudeSeatbeltSpec,
    profile_path: &Path,
    allowed_canary: &Path,
    forbidden_read_targets: &[ClaudeSeatbeltProbeTarget],
    forbidden_write_targets: &[ClaudeSeatbeltProbeTarget],
    network_policy_port: u16,
    unexpected_bind_port: u16,
) -> EvalResult<Vec<ClaudeSeatbeltProbeCommand>> {
    validate_seatbelt_spec(spec)?;
    let allowed_ports = [
        Some(spec.transport.provider_proxy_port),
        spec.transport.engram_daemon_port,
    ]
    .into_iter()
    .flatten()
    .collect::<BTreeSet<_>>();
    if network_policy_port == 0
        || unexpected_bind_port == 0
        || network_policy_port == unexpected_bind_port
        || allowed_ports.contains(&network_policy_port)
        || allowed_ports.contains(&unexpected_bind_port)
    {
        return Err(invalid(
            "Seatbelt denied network probe ports must be nonzero, distinct, and unallowed",
        ));
    }
    require_normalized_absolute_path(profile_path, "Seatbelt profile")?;
    require_normalized_absolute_path(allowed_canary, "Seatbelt allowed canary")?;
    let allowed_metadata = fs::symlink_metadata(allowed_canary)?;
    if allowed_metadata.file_type().is_symlink() || !allowed_metadata.is_file() {
        return Err(invalid("Seatbelt allowed canary is not a regular file"));
    }
    require_single_link(&allowed_metadata, allowed_canary)?;
    if spec
        .forbidden_read_paths
        .iter()
        .any(|path| allowed_canary.starts_with(path))
    {
        return Err(invalid(
            "Seatbelt allowed canary is under a denied read path",
        ));
    }
    validate_seatbelt_probe_targets(&spec.forbidden_read_paths, forbidden_read_targets, true)?;
    validate_seatbelt_probe_targets(&spec.forbidden_write_paths, forbidden_write_targets, false)?;

    let prefix = [
        spec.sandbox_exec.clone(),
        "-f".to_string(),
        profile_path.display().to_string(),
    ];
    let command = |label: String,
                   protected_path: Option<String>,
                   tail: Vec<String>,
                   expect_success: bool,
                   denial: bool,
                   absent: Option<String>| {
        let mut argv = prefix.to_vec();
        argv.extend(tail);
        ClaudeSeatbeltProbeCommand {
            label,
            protected_path,
            argv,
            expect_success,
            require_sandbox_denial_evidence: denial,
            must_remain_absent: absent,
        }
    };
    let mut commands = vec![
        command(
            "profile_compile".to_string(),
            None,
            vec!["/usr/bin/true".to_string()],
            true,
            false,
            None,
        ),
        command(
            "allowed_read".to_string(),
            None,
            vec!["/bin/cat".to_string(), allowed_canary.display().to_string()],
            true,
            false,
            None,
        ),
    ];
    for target in forbidden_read_targets {
        commands.push(command(
            "forbidden_read".to_string(),
            Some(target.protected_path.clone()),
            vec!["/bin/cat".to_string(), target.canary_path.clone()],
            false,
            true,
            None,
        ));
    }
    for target in forbidden_write_targets {
        commands.push(command(
            "forbidden_write".to_string(),
            Some(target.protected_path.clone()),
            vec!["/usr/bin/touch".to_string(), target.canary_path.clone()],
            false,
            true,
            Some(target.canary_path.clone()),
        ));
    }
    commands.push(command(
        "provider_proxy_allowed".to_string(),
        None,
        vec![
            "/usr/bin/nc".to_string(),
            "-z".to_string(),
            "-w".to_string(),
            "1".to_string(),
            spec.transport.provider_proxy_host.clone(),
            spec.transport.provider_proxy_port.to_string(),
        ],
        true,
        false,
        None,
    ));
    if let Some(port) = spec.transport.engram_daemon_port {
        commands.push(command(
            "engram_loopback_allowed".to_string(),
            None,
            vec![
                "/usr/bin/nc".to_string(),
                "-z".to_string(),
                "-w".to_string(),
                "1".to_string(),
                "127.0.0.1".to_string(),
                port.to_string(),
            ],
            true,
            false,
            None,
        ));
    }
    commands.push(command(
        "network_denial".to_string(),
        None,
        vec![
            "/usr/bin/nc".to_string(),
            "-z".to_string(),
            "-w".to_string(),
            "1".to_string(),
            "127.0.0.1".to_string(),
            network_policy_port.to_string(),
        ],
        false,
        false,
        None,
    ));
    commands.push(command(
        "network_errno_denial".to_string(),
        None,
        vec![
            "/usr/bin/nc".to_string(),
            "-z".to_string(),
            "-v".to_string(),
            "-w".to_string(),
            "1".to_string(),
            "127.0.0.1".to_string(),
            network_policy_port.to_string(),
        ],
        false,
        true,
        None,
    ));
    commands.push(command(
        "external_network_denial".to_string(),
        None,
        vec![
            "/usr/bin/nc".to_string(),
            "-z".to_string(),
            "-v".to_string(),
            "-w".to_string(),
            "1".to_string(),
            CLAUDE_EXTERNAL_PROBE_ADDRESS.to_string(),
            CLAUDE_EXTERNAL_PROBE_PORT.to_string(),
        ],
        false,
        true,
        None,
    ));
    commands.push(command(
        "unexpected_bind_denial".to_string(),
        None,
        vec![
            "/usr/bin/python3".to_string(),
            "-c".to_string(),
            format!(
                "import socket; s=socket.socket(); s.bind(('127.0.0.1', {unexpected_bind_port}))"
            ),
        ],
        false,
        true,
        None,
    ));
    Ok(commands)
}

/// Execute the real Seatbelt matrix without invoking Claude or inspecting credentials.
pub fn execute_claude_seatbelt_probes(
    spec: &ClaudeSeatbeltSpec,
    profile_path: &Path,
    commands: &[ClaudeSeatbeltProbeCommand],
    environment: &BTreeMap<String, String>,
) -> EvalResult<ClaudeSeatbeltProbeAudit> {
    validate_seatbelt_spec(spec)?;
    validate_claude_probe_environment(environment)?;
    validate_claude_probe_command_set(spec, profile_path, commands)?;
    let expected_profile = generate_claude_seatbelt_profile(spec)?;
    let metadata = fs::symlink_metadata(profile_path)?;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    #[cfg(unix)]
    let mode = metadata.permissions().mode() & 0o777;
    #[cfg(not(unix))]
    let mode = 0;
    let actual_profile =
        read_exact_regular_file(profile_path, metadata.len(), mode, MAX_MANIFEST_FILE_BYTES)?;
    if actual_profile != expected_profile.as_bytes() {
        return Err(invalid(
            "executed Seatbelt profile differs from generated bytes",
        ));
    }

    let mut audits = Vec::with_capacity(commands.len());
    let mut forbidden_reads = BTreeMap::new();
    let mut forbidden_writes = BTreeMap::new();
    let mut network_control = None;
    for command in commands {
        if command.label == "network_denial" {
            network_control = Some(execute_exact_network_control(
                &command.argv,
                3,
                environment,
            )?);
        }
        let audit = run_sanitized_probe_command(
            &command.label,
            &command.argv,
            environment,
            command.require_sandbox_denial_evidence,
        )?;
        if audit.succeeded != command.expect_success
            || (command.require_sandbox_denial_evidence && !audit.sandbox_denial_evidence)
        {
            return Err(invalid(format!(
                "Claude Seatbelt probe did not prove policy enforcement: {}",
                command.label
            )));
        }
        if command
            .must_remain_absent
            .as_ref()
            .is_some_and(|path| Path::new(path).exists())
        {
            return Err(invalid("Claude denied write probe created its target"));
        }
        if let Some(path) = &command.protected_path {
            let outcome = if audit.succeeded {
                IsolationProbeOutcome::Allowed
            } else if audit.sandbox_denial_evidence {
                IsolationProbeOutcome::PermissionDenied
            } else {
                IsolationProbeOutcome::OtherFailure
            };
            match command.label.as_str() {
                "forbidden_read" => {
                    forbidden_reads.insert(path.clone(), outcome);
                }
                "forbidden_write" => {
                    forbidden_writes.insert(path.clone(), outcome);
                }
                _ => return Err(invalid("protected Seatbelt probe has an invalid label")),
            }
        }
        audits.push(audit);
    }
    let succeeded = |label: &str| {
        audits
            .iter()
            .find(|audit| audit.label == label)
            .map(|audit| audit.succeeded)
            .unwrap_or(false)
    };
    let probe = ClaudeSeatbeltProbe {
        sandbox_exec_present: Path::new(&spec.sandbox_exec).is_file(),
        profile_compiled: succeeded("profile_compile"),
        allowed_evaluation_canary_read: succeeded("allowed_read"),
        provider_proxy_loopback_allowed: succeeded("provider_proxy_allowed"),
        engram_loopback_allowed: spec
            .transport
            .engram_daemon_port
            .map(|_| succeeded("engram_loopback_allowed")),
        forbidden_reads,
        forbidden_writes,
        network_denied: !succeeded("network_denial") && !succeeded("network_errno_denial"),
        adjacent_loopback_denied: !succeeded("network_denial")
            && !succeeded("network_errno_denial"),
        external_network_denied: !succeeded("external_network_denial"),
        unexpected_bind_denied: !succeeded("unexpected_bind_denial"),
        network_sandbox_denial_observed: audits.iter().any(|audit| {
            matches!(
                audit.label.as_str(),
                "network_errno_denial" | "external_network_denial" | "unexpected_bind_denial"
            ) && audit.sandbox_denial_evidence
        }),
    };
    validate_claude_seatbelt_probe(spec, &probe)?;
    Ok(ClaudeSeatbeltProbeAudit {
        valid: true,
        profile_sha256: sha256_bytes(&actual_profile),
        profile: probe,
        network_control: network_control
            .ok_or_else(|| invalid("Claude Seatbelt probe lacked its network control"))?,
        commands: audits,
    })
}

pub fn validate_claude_seatbelt_probe(
    spec: &ClaudeSeatbeltSpec,
    probe: &ClaudeSeatbeltProbe,
) -> EvalResult<()> {
    validate_seatbelt_spec(spec)?;
    if !probe.sandbox_exec_present
        || !probe.profile_compiled
        || !probe.allowed_evaluation_canary_read
        || !probe.provider_proxy_loopback_allowed
        || probe.engram_loopback_allowed != spec.transport.engram_daemon_port.map(|_| true)
        || !probe.network_denied
        || !probe.adjacent_loopback_denied
        || !probe.external_network_denied
        || !probe.unexpected_bind_denied
        || !probe.network_sandbox_denial_observed
    {
        return Err(invalid(
            "Claude Seatbelt availability/positive probe failed",
        ));
    }
    require_exact_denials(
        &spec.forbidden_read_paths,
        &probe.forbidden_reads,
        "Seatbelt forbidden read",
    )?;
    require_exact_denials(
        &spec.forbidden_write_paths,
        &probe.forbidden_writes,
        "Seatbelt forbidden write",
    )
}

fn validate_seatbelt_spec(spec: &ClaudeSeatbeltSpec) -> EvalResult<()> {
    if spec.schema_version != ISOLATION_SCHEMA_VERSION
        || spec.sandbox_exec != "/usr/bin/sandbox-exec"
        || spec.forbidden_read_paths.is_empty()
    {
        return Err(invalid("invalid Claude Seatbelt contract"));
    }
    validate_claude_transport(&spec.transport)?;
    require_unique_absolute_paths(&spec.forbidden_read_paths, "Seatbelt read deny")?;
    if spec.forbidden_write_paths.is_empty() {
        return Err(invalid("Claude Seatbelt must deny at least one write path"));
    }
    require_unique_absolute_paths(&spec.forbidden_write_paths, "Seatbelt write deny")?;
    Ok(())
}

fn validate_claude_transport(transport: &ClaudeNarrowTransportContract) -> EvalResult<()> {
    if transport.provider_proxy_host != "127.0.0.1"
        || transport.provider_proxy_port == 0
        || transport.provider_connect_authority != CLAUDE_PROVIDER_AUTHORITY
        || transport.engram_daemon_port == Some(0)
        || transport.engram_daemon_port == Some(transport.provider_proxy_port)
        || transport.max_connect_header_bytes == 0
        || transport.max_connect_header_bytes > 16 * 1024
        || transport.max_connect_header_count == 0
        || transport.max_connect_header_count > 128
        || transport.max_connect_line_bytes == 0
        || u64::from(transport.max_connect_line_bytes) > transport.max_connect_header_bytes
        || transport.max_connect_requests == 0
        || transport.max_connect_requests > 64
        || transport.max_resolved_addresses == 0
        || transport.max_resolved_addresses > 64
        || transport.connect_timeout_ms == 0
        || transport.connect_timeout_ms > 30_000
        || transport.total_timeout_ms < transport.connect_timeout_ms
        || transport.total_timeout_ms > 15 * 60 * 1_000
        || transport.max_tunnel_bytes_each_direction == 0
        || transport.max_tunnel_bytes_each_direction > 1024 * 1024 * 1024
        || transport.idle_timeout_ms == 0
        || transport.idle_timeout_ms > 5 * 60 * 1_000
    {
        return Err(invalid(
            "Claude narrow transport is not an exact bounded loopback/CONNECT contract",
        ));
    }
    Ok(())
}

/// Parse one bounded proxy request without retaining its headers or any following bytes.
pub fn inspect_claude_connect_request(
    transport: &ClaudeNarrowTransportContract,
    request: &[u8],
) -> EvalResult<ClaudeConnectRequestAudit> {
    inspect_claude_connect_request_with_auth(transport, request, None)
}

fn inspect_claude_connect_request_with_auth(
    transport: &ClaudeNarrowTransportContract,
    request: &[u8],
    expected_proxy_authorization: Option<&[u8]>,
) -> EvalResult<ClaudeConnectRequestAudit> {
    validate_claude_transport(transport)?;
    let request_len = u64::try_from(request.len())
        .map_err(|_| invalid("Claude proxy request length overflow"))?;
    if request_len > transport.max_connect_header_bytes {
        return Err(invalid(
            "Claude proxy CONNECT header exceeds its byte bound",
        ));
    }
    let terminator = request
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| invalid("Claude proxy request has no complete HTTP header"))?;
    let header_end = terminator + 4;
    let header = std::str::from_utf8(&request[..header_end])
        .map_err(|_| invalid("Claude proxy request header is not UTF-8"))?;
    if header.contains('\0') || header.contains("\r\n ") || header.contains("\r\n\t") {
        return Err(invalid(
            "Claude proxy request contains an unsafe folded header",
        ));
    }
    let mut lines = header[..terminator].split("\r\n");
    let first = lines
        .next()
        .ok_or_else(|| invalid("Claude proxy request line is absent"))?;
    let parts = first.split(' ').collect::<Vec<_>>();
    if first.len() > usize::from(transport.max_connect_line_bytes)
        || parts.len() != 3
        || parts.iter().any(|part| part.is_empty())
        || parts[0].len() > 16
        || parts[1].len() > 255
        || !parts[0].bytes().all(is_http_token_byte)
        || !parts[1].bytes().all(|byte| byte.is_ascii_graphic())
        || parts[2] != "HTTP/1.1"
    {
        return Err(invalid("Claude proxy request line is malformed"));
    }
    let mut host_count = 0_u32;
    let mut host_matches = false;
    let mut proxy_authorization_count = 0_u32;
    let mut proxy_authorization_matches = false;
    let mut forbidden_framing = false;
    let mut header_count = 0_u16;
    for line in lines {
        header_count = header_count
            .checked_add(1)
            .ok_or_else(|| invalid("Claude proxy header count overflow"))?;
        if header_count > transport.max_connect_header_count
            || line.len() > usize::from(transport.max_connect_line_bytes)
        {
            return Err(invalid("Claude proxy header count/line bound exceeded"));
        }
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| invalid("Claude proxy header line is malformed"))?;
        if name.is_empty()
            || !name.bytes().all(is_http_token_byte)
            || value.bytes().any(|byte| !(0x20..=0x7e).contains(&byte))
        {
            return Err(invalid(
                "Claude proxy header contains invalid token/CTL bytes",
            ));
        }
        let name = name.to_ascii_lowercase();
        let value = value.trim();
        if name == "host" {
            host_count += 1;
            host_matches = value == transport.provider_connect_authority;
        }
        if name == "proxy-authorization" {
            proxy_authorization_count += 1;
            proxy_authorization_matches = expected_proxy_authorization
                .is_some_and(|expected| constant_time_eq(value.as_bytes(), expected));
        }
        if matches!(
            name.as_str(),
            "authorization" | "content-length" | "cookie" | "transfer-encoding"
        ) {
            forbidden_framing = true;
        }
    }
    let body_bytes = request_len
        .checked_sub(u64::try_from(header_end).map_err(|_| invalid("header length overflow"))?)
        .ok_or_else(|| invalid("Claude proxy body length underflow"))?;
    let accepted = parts[0] == "CONNECT"
        && parts[1] == transport.provider_connect_authority
        && parts[2] == "HTTP/1.1"
        && host_count == 1
        && host_matches
        && match expected_proxy_authorization {
            Some(_) => proxy_authorization_count == 1 && proxy_authorization_matches,
            None => proxy_authorization_count == 0,
        }
        && !forbidden_framing
        && body_bytes == 0;
    Ok(ClaudeConnectRequestAudit {
        accepted,
        target: parts[1].to_string(),
        method: parts[0].to_string(),
        header_bytes: u64::try_from(header_end).map_err(|_| invalid("header length overflow"))?,
        body_bytes,
        raw_request_retained: false,
    })
}

fn is_http_token_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
        || matches!(
            byte,
            b'!' | b'#'
                | b'$'
                | b'%'
                | b'&'
                | b'\''
                | b'*'
                | b'+'
                | b'-'
                | b'.'
                | b'^'
                | b'_'
                | b'`'
                | b'|'
                | b'~'
        )
}

fn validate_proxy_session_binding(session: &ClaudeProviderProxySessionBinding) -> EvalResult<()> {
    validate_invocation_id(&session.invocation_id)?;
    if !is_sha256(&session.launch_sha256)
        || !is_sha256(&session.seatbelt_profile_sha256)
        || !is_sha256(&session.proxy_auth_sha256)
        || session.provider_process_id == 0
        || session.proxy_process_id == 0
        || session.provider_process_id == session.proxy_process_id
        || session.proxy_started_unix_ms == 0
        || session.provider_started_unix_ms < session.proxy_started_unix_ms
        || !timestamp_is_recent(session.proxy_started_unix_ms, 15 * 60 * 1_000)?
        || !timestamp_is_recent(session.provider_started_unix_ms, 15 * 60 * 1_000)?
    {
        return Err(invalid(
            "Claude provider proxy session is not bound to an exact launch/process identity",
        ));
    }
    Ok(())
}

fn validate_invocation_id(invocation_id: &str) -> EvalResult<()> {
    if invocation_id.is_empty()
        || invocation_id.len() > 128
        || !invocation_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(invalid("Claude invocation identifier is invalid"));
    }
    Ok(())
}

fn proxy_secret_digest(invocation_id: &str, secret: &[u8]) -> String {
    let mut digest = Sha256::new();
    digest.update(b"engram-claude-proxy-secret-v1\0");
    digest.update(invocation_id.as_bytes());
    digest.update(b"\0");
    digest.update(secret);
    format!("{:x}", digest.finalize())
}

fn base64_standard(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);
        output.push(TABLE[usize::from(first >> 2)] as char);
        output.push(TABLE[usize::from(((first & 0x03) << 4) | (second >> 4))] as char);
        if chunk.len() > 1 {
            output.push(TABLE[usize::from(((second & 0x0f) << 2) | (third >> 6))] as char);
        } else {
            output.push('=');
        }
        if chunk.len() > 2 {
            output.push(TABLE[usize::from(third & 0x3f)] as char);
        } else {
            output.push('=');
        }
    }
    output
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    let mut difference = left.len() ^ right.len();
    let max = left.len().max(right.len());
    for index in 0..max {
        let left = left.get(index).copied().unwrap_or(0);
        let right = right.get(index).copied().unwrap_or(0);
        difference |= usize::from(left ^ right);
    }
    difference == 0
}

/// Observe an already-running, lane-private Engram daemon from the trusted runner.
///
/// This function does not accept caller-supplied PID, liveness, or health claims. It reads the
/// owner-private daemon receipts, observes the live process executable through macOS, performs one
/// bounded token-free `GET /health`, and binds the exact response contract. The caller must invoke
/// it before starting the Seatbelt-wrapped provider and later enforce that chronology.
#[cfg(target_os = "macos")]
pub fn observe_claude_engram_daemon_attestation(
    surface: &IsolationEngramSurface,
    transport: &ClaudeNarrowTransportContract,
) -> EvalResult<ClaudeEngramDaemonAttestation> {
    validate_engram_surface(surface)?;
    validate_claude_transport(transport)?;
    let port = transport
        .engram_daemon_port
        .ok_or_else(|| invalid("Engram daemon attestation lacks its frozen loopback port"))?;
    let daemon_directory = Path::new(&surface.home)
        .join("projects")
        .join(&surface.project);
    require_private_directory(&daemon_directory, "Engram daemon directory")?;
    let daemon_port_file = bound_file(&daemon_directory.join("daemon.port"), true)?;
    let daemon_pid_file = bound_file(&daemon_directory.join("daemon.pid"), true)?;
    let daemon_metadata_file = bound_file(&daemon_directory.join("daemon.meta.json"), true)?;
    let port_bytes = read_bound_file(Path::new(&daemon_port_file.path), &daemon_port_file.sha256)?;
    let pid_bytes = read_bound_file(Path::new(&daemon_pid_file.path), &daemon_pid_file.sha256)?;
    let observed_port = parse_exact_decimal::<u16>(&port_bytes, "Engram daemon port")?;
    let process_id = parse_exact_decimal::<u32>(&pid_bytes, "Engram daemon PID")?;
    if observed_port != port || process_id == 0 {
        return Err(invalid("Engram daemon receipt port/PID drifted"));
    }
    require_process_alive(process_id)?;
    let expected_executable = Path::new(&surface.executable).canonicalize()?;
    let process_executable = observed_process_executable(process_id)?;
    if process_executable != expected_executable
        || read_regular_file_digest(&process_executable)? != surface.executable_sha256
    {
        return Err(invalid(
            "live Engram daemon process executable does not match its frozen binary",
        ));
    }
    let observed_unix_ms = now_unix_ms()?;
    let (health, health_response_sha256) = fetch_exact_engram_health(surface, port, process_id)?;
    require_process_alive(process_id)?;
    let health_contract_sha256 = sha256_serialized(&health)?;
    let mut attestation = ClaudeEngramDaemonAttestation {
        valid: true,
        host: "127.0.0.1".to_string(),
        port,
        project: surface.project.clone(),
        home: surface.home.clone(),
        process_id,
        process_executable_path: process_executable.display().to_string(),
        executable_sha256: surface.executable_sha256.clone(),
        daemon_port_file,
        daemon_pid_file,
        daemon_metadata_file,
        started_outside_seatbelt: true,
        process_identity_observed_by_runner: true,
        healthy: true,
        health_schema_version: health.health_schema_version,
        mcp_contract_version: health.mcp_contract_version,
        mcp_tools_sha256: health.mcp_tools_sha256,
        mcp_protocol_version: health.mcp_protocol_version,
        health_build_sha: health.build_sha,
        auth_required: health.auth_required,
        storage_status: health.storage_status,
        storage_ready: health.storage_ready,
        storage_available_bytes: health.storage_available_bytes,
        storage_required_bytes: health.storage_required_bytes,
        health_response_sha256,
        health_contract_sha256,
        health_observed_by_runner: true,
        observed_unix_ms,
        raw_bearer_token_read: false,
        attestation_sha256: String::new(),
    };
    attestation.attestation_sha256 = claude_engram_daemon_attestation_digest(&attestation)?;
    validate_claude_engram_daemon_attestation(surface, transport, &attestation)?;
    Ok(attestation)
}

#[cfg(not(target_os = "macos"))]
pub fn observe_claude_engram_daemon_attestation(
    _surface: &IsolationEngramSurface,
    _transport: &ClaudeNarrowTransportContract,
) -> EvalResult<ClaudeEngramDaemonAttestation> {
    Err(invalid(
        "runner-observed Engram daemon attestation is currently frozen for macOS only",
    ))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct ObservedEngramHealth {
    status: String,
    service: String,
    version: String,
    build_sha: Option<String>,
    pid: u32,
    health_schema_version: u64,
    mcp_contract_version: u64,
    mcp_tools_sha256: String,
    mcp_protocol_version: String,
    auth_required: bool,
    storage_status: String,
    storage_ready: bool,
    storage_available_bytes: u64,
    storage_required_bytes: u64,
}

#[cfg(target_os = "macos")]
fn fetch_exact_engram_health(
    surface: &IsolationEngramSurface,
    port: u16,
    process_id: u32,
) -> EvalResult<(ObservedEngramHealth, String)> {
    let timeout = Duration::from_millis(500);
    let mut stream =
        TcpStream::connect_timeout(&SocketAddr::from(([127, 0, 0, 1], port)), timeout)?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;
    let request =
        format!("GET /health HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes())?;
    let mut response = Vec::new();
    stream
        .take((MAX_ENGRAM_HEALTH_RESPONSE_BYTES as u64).saturating_add(1))
        .read_to_end(&mut response)?;
    if response.len() > MAX_ENGRAM_HEALTH_RESPONSE_BYTES {
        return Err(invalid("Engram health response exceeded its byte ceiling"));
    }
    let separator = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| invalid("Engram health response has no HTTP header boundary"))?;
    let header = std::str::from_utf8(&response[..separator])
        .map_err(|_| invalid("Engram health response header is not UTF-8"))?;
    let mut header_lines = header.split("\r\n");
    if header_lines.next() != Some("HTTP/1.1 200 OK")
        || header_lines.any(|line| {
            line.eq_ignore_ascii_case("transfer-encoding: chunked")
                || line
                    .bytes()
                    .any(|byte| byte == 0 || byte == b'\n' || byte == b'\r')
        })
    {
        return Err(invalid(
            "Engram health response is not an exact bounded HTTP 200 response",
        ));
    }
    let body = &response[separator + 4..];
    let value: serde_json::Value = serde_json::from_slice(body)
        .map_err(|_| invalid("Engram health response body is not JSON"))?;
    let object = value
        .as_object()
        .ok_or_else(|| invalid("Engram health response is not an object"))?;
    let expected_keys = BTreeSet::from([
        "auth_required",
        "build_sha",
        "health_schema_version",
        "mcp_contract_version",
        "mcp_protocol_version",
        "mcp_tools_sha256",
        "pid",
        "service",
        "status",
        "storage_available_bytes",
        "storage_ready",
        "storage_required_bytes",
        "storage_status",
        "version",
    ]);
    if object.keys().map(String::as_str).collect::<BTreeSet<_>>() != expected_keys {
        return Err(invalid("Engram health response key set drifted"));
    }
    let health: ObservedEngramHealth = serde_json::from_value(value)
        .map_err(|_| invalid("Engram health response types drifted"))?;
    if health.status != "ok"
        || health.service != "engram"
        || health.version != surface.executable_version
        || health.pid != process_id
        || health.health_schema_version != ENGRAM_HEALTH_SCHEMA_VERSION
        || health.mcp_contract_version != ENGRAM_MCP_CONTRACT_VERSION
        || health.mcp_tools_sha256 != surface.mcp_tools_sha256
        || health.mcp_protocol_version != ENGRAM_MCP_PROTOCOL_VERSION
        || !health.auth_required
        || health.storage_status != "ready"
        || !health.storage_ready
        || health.storage_available_bytes < health.storage_required_bytes
    {
        return Err(invalid(
            "Engram health identity/storage/auth contract is not ready and exact",
        ));
    }
    Ok((health, sha256_bytes(body)))
}

fn parse_exact_decimal<T>(bytes: &[u8], label: &str) -> EvalResult<T>
where
    T: std::str::FromStr,
{
    if bytes.len() < 2
        || bytes.last() != Some(&b'\n')
        || !bytes[..bytes.len() - 1].iter().all(u8::is_ascii_digit)
    {
        return Err(invalid(format!("{label} is not an exact decimal line")));
    }
    std::str::from_utf8(&bytes[..bytes.len() - 1])
        .map_err(|_| invalid(format!("{label} is not UTF-8")))?
        .parse::<T>()
        .map_err(|_| invalid(format!("{label} is out of range")))
}

pub fn validate_claude_engram_daemon_attestation(
    surface: &IsolationEngramSurface,
    transport: &ClaudeNarrowTransportContract,
    attestation: &ClaudeEngramDaemonAttestation,
) -> EvalResult<()> {
    validate_engram_surface(surface)?;
    validate_claude_transport(transport)?;
    let port = transport
        .engram_daemon_port
        .ok_or_else(|| invalid("Engram daemon attestation has no allowed loopback port"))?;
    let daemon_directory = Path::new(&surface.home)
        .join("projects")
        .join(&surface.project);
    require_private_directory(&daemon_directory, "Engram daemon directory")?;
    let expected_port = bound_file(&daemon_directory.join("daemon.port"), true)?;
    let expected_pid = bound_file(&daemon_directory.join("daemon.pid"), true)?;
    let expected_metadata = bound_file(&daemon_directory.join("daemon.meta.json"), true)?;
    let port_bytes = read_bound_file(Path::new(&expected_port.path), &expected_port.sha256)?;
    let pid_bytes = read_bound_file(Path::new(&expected_pid.path), &expected_pid.sha256)?;
    let observed_port = parse_exact_decimal::<u16>(&port_bytes, "Engram daemon port").ok();
    let observed_pid = parse_exact_decimal::<u32>(&pid_bytes, "Engram daemon PID").ok();
    let metadata_bytes = read_bound_file(
        Path::new(&expected_metadata.path),
        &expected_metadata.sha256,
    )?;
    let metadata: serde_json::Value = serde_json::from_slice(&metadata_bytes)
        .map_err(|_| invalid("Engram daemon metadata is not JSON"))?;
    let metadata = metadata
        .as_object()
        .ok_or_else(|| invalid("Engram daemon metadata is not an object"))?;
    let expected_keys = BTreeSet::from([
        "executable_path",
        "executable_sha256",
        "executable_version",
        "pid",
        "port",
        "schema_version",
    ]);
    let metadata_matches = metadata.keys().map(String::as_str).collect::<BTreeSet<_>>()
        == expected_keys
        && metadata
            .get("schema_version")
            .and_then(serde_json::Value::as_u64)
            == Some(1)
        && metadata
            .get("executable_path")
            .and_then(serde_json::Value::as_str)
            == Some(surface.executable.as_str())
        && metadata
            .get("executable_sha256")
            .and_then(serde_json::Value::as_str)
            == Some(surface.executable_sha256.as_str())
        && metadata
            .get("executable_version")
            .and_then(serde_json::Value::as_str)
            == Some(surface.executable_version.as_str())
        && metadata.get("pid").and_then(serde_json::Value::as_u64)
            == Some(u64::from(attestation.process_id))
        && metadata.get("port").and_then(serde_json::Value::as_u64) == Some(u64::from(port));
    let expected_process_path = Path::new(&surface.executable).canonicalize()?;
    let observed_process_path = Path::new(&attestation.process_executable_path);
    let expected_health = ObservedEngramHealth {
        status: "ok".to_string(),
        service: "engram".to_string(),
        version: surface.executable_version.clone(),
        build_sha: attestation.health_build_sha.clone(),
        pid: attestation.process_id,
        health_schema_version: attestation.health_schema_version,
        mcp_contract_version: attestation.mcp_contract_version,
        mcp_tools_sha256: attestation.mcp_tools_sha256.clone(),
        mcp_protocol_version: attestation.mcp_protocol_version.clone(),
        auth_required: attestation.auth_required,
        storage_status: attestation.storage_status.clone(),
        storage_ready: attestation.storage_ready,
        storage_available_bytes: attestation.storage_available_bytes,
        storage_required_bytes: attestation.storage_required_bytes,
    };
    if !attestation.valid
        || attestation.host != "127.0.0.1"
        || attestation.port != port
        || attestation.project != surface.project
        || attestation.home != surface.home
        || attestation.process_id == 0
        || !observed_process_path.is_absolute()
        || observed_process_path.canonicalize()? != expected_process_path
        || attestation.executable_sha256 != surface.executable_sha256
        || attestation.daemon_port_file != expected_port
        || attestation.daemon_pid_file != expected_pid
        || attestation.daemon_metadata_file != expected_metadata
        || observed_port != Some(port)
        || observed_pid != Some(attestation.process_id)
        || !metadata_matches
        || !attestation.started_outside_seatbelt
        || !attestation.process_identity_observed_by_runner
        || !attestation.healthy
        || attestation.health_schema_version != ENGRAM_HEALTH_SCHEMA_VERSION
        || attestation.mcp_contract_version != ENGRAM_MCP_CONTRACT_VERSION
        || attestation.mcp_tools_sha256 != surface.mcp_tools_sha256
        || attestation.mcp_protocol_version != ENGRAM_MCP_PROTOCOL_VERSION
        || attestation.health_build_sha.as_ref().is_some_and(|build| {
            build.is_empty()
                || build.len() > 128
                || !build
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        })
        || !attestation.auth_required
        || attestation.storage_status != "ready"
        || !attestation.storage_ready
        || attestation.storage_required_bytes == 0
        || attestation.storage_available_bytes < attestation.storage_required_bytes
        || !is_sha256(&attestation.health_response_sha256)
        || attestation.health_contract_sha256 != sha256_serialized(&expected_health)?
        || !attestation.health_observed_by_runner
        || attestation.observed_unix_ms == 0
        || !timestamp_is_recent(attestation.observed_unix_ms, 30_000)?
        || attestation.raw_bearer_token_read
        || attestation.attestation_sha256 != claude_engram_daemon_attestation_digest(attestation)?
    {
        return Err(invalid(
            "Engram daemon is not exactly prestarted, private, healthy, and hash-bound",
        ));
    }
    Ok(())
}

fn claude_engram_daemon_attestation_digest(
    attestation: &ClaudeEngramDaemonAttestation,
) -> EvalResult<String> {
    let mut digest_surface = attestation.clone();
    digest_surface.attestation_sha256.clear();
    sha256_serialized(&digest_surface)
}

/// Exact macOS managed-policy source list for Claude Code 2.1.260.
#[must_use]
pub fn known_claude_macos_managed_policy_paths() -> Vec<String> {
    vec![
        CLAUDE_MANAGED_SETTINGS_PATH.to_string(),
        CLAUDE_MANAGED_MCP_PATH.to_string(),
        CLAUDE_MANAGED_PREFERENCES_PATH.to_string(),
        CLAUDE_MANAGED_SETTINGS_DROP_IN_DIRECTORY.to_string(),
    ]
}

/// Re-observe every known macOS managed-policy source without executing a helper or provider.
#[cfg(target_os = "macos")]
pub fn observe_claude_macos_managed_policy() -> EvalResult<ClaudeManagedPolicyAttestation> {
    let (sources, opaque_preferences, api_key_helper, policy_helper, external_execution) =
        collect_claude_macos_managed_policy()?;
    let mut attestation = ClaudeManagedPolicyAttestation {
        valid: true,
        sources,
        all_known_sources_observed: false,
        server_managed_effective_policy_unobserved: true,
        opaque_managed_preferences_present: opaque_preferences,
        api_key_helper_present: api_key_helper,
        policy_helper_present: policy_helper,
        external_execution_policy_present: external_execution,
        // The local-host foundation intentionally never upgrades this to true: macOS managed
        // preferences and helper execution are not removed by Claude's cleared user settings.
        safe_for_local_provider_execution: false,
        confounded_exploratory_only: true,
        raw_policy_contents_retained: false,
        observed_unix_ms: now_unix_ms()?,
        attestation_sha256: String::new(),
    };
    attestation.attestation_sha256 = claude_managed_policy_attestation_digest(&attestation)?;
    validate_claude_macos_managed_policy_attestation(&attestation)?;
    Ok(attestation)
}

#[cfg(not(target_os = "macos"))]
pub fn observe_claude_macos_managed_policy() -> EvalResult<ClaudeManagedPolicyAttestation> {
    Err(invalid(
        "Claude managed-policy attestation is currently frozen for macOS only",
    ))
}

#[cfg(target_os = "macos")]
pub fn validate_claude_macos_managed_policy_attestation(
    attestation: &ClaudeManagedPolicyAttestation,
) -> EvalResult<()> {
    let (sources, opaque_preferences, api_key_helper, policy_helper, external_execution) =
        collect_claude_macos_managed_policy()?;
    if !attestation.valid
        || attestation.all_known_sources_observed
        || !attestation.server_managed_effective_policy_unobserved
        || attestation.sources != sources
        || attestation.opaque_managed_preferences_present != opaque_preferences
        || attestation.api_key_helper_present != api_key_helper
        || attestation.policy_helper_present != policy_helper
        || attestation.external_execution_policy_present != external_execution
        || attestation.safe_for_local_provider_execution
        || !attestation.confounded_exploratory_only
        || attestation.raw_policy_contents_retained
        || attestation.observed_unix_ms == 0
        || !timestamp_is_recent(attestation.observed_unix_ms, 30_000)?
        || attestation.attestation_sha256 != claude_managed_policy_attestation_digest(attestation)?
    {
        return Err(invalid(
            "Claude macOS managed-policy source attestation is stale or overclaims containment",
        ));
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn validate_claude_macos_managed_policy_attestation(
    _attestation: &ClaudeManagedPolicyAttestation,
) -> EvalResult<()> {
    Err(invalid(
        "Claude managed-policy attestation is currently frozen for macOS only",
    ))
}

#[cfg(target_os = "macos")]
fn collect_claude_macos_managed_policy(
) -> EvalResult<(Vec<ClaudeManagedPolicySourceAudit>, bool, bool, bool, bool)> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    let mut roles = vec![
        (
            "managed_settings_json".to_string(),
            CLAUDE_MANAGED_SETTINGS_PATH.to_string(),
            true,
        ),
        (
            "managed_mcp_json".to_string(),
            CLAUDE_MANAGED_MCP_PATH.to_string(),
            true,
        ),
        (
            "managed_preferences_plist".to_string(),
            CLAUDE_MANAGED_PREFERENCES_PATH.to_string(),
            false,
        ),
    ];
    let drop_in_directory = Path::new(CLAUDE_MANAGED_SETTINGS_DROP_IN_DIRECTORY);
    match fs::symlink_metadata(drop_in_directory) {
        Ok(metadata) => {
            let mode = metadata.permissions().mode() & 0o777;
            if metadata.file_type().is_symlink()
                || !metadata.is_dir()
                || metadata.uid() != 0
                || mode & 0o022 != 0
            {
                return Err(invalid(
                    "Claude managed-settings.d is not a root-owned non-writable directory",
                ));
            }
            let mut entries = fs::read_dir(drop_in_directory)?.collect::<Result<Vec<_>, _>>()?;
            entries.sort_by_key(std::fs::DirEntry::file_name);
            if entries.len() > 128 {
                return Err(invalid(
                    "Claude managed-settings.d exceeds its file-count bound",
                ));
            }
            for entry in entries {
                let path = entry.path();
                let name = entry
                    .file_name()
                    .into_string()
                    .map_err(|_| invalid("Claude managed-settings.d filename is not UTF-8"))?;
                if name.is_empty()
                    || name.starts_with('.')
                    || Path::new(&name)
                        .extension()
                        .and_then(|value| value.to_str())
                        != Some("json")
                {
                    return Err(invalid(
                        "Claude managed-settings.d contains an unrecognized entry",
                    ));
                }
                roles.push((
                    format!("managed_settings_drop_in:{name}"),
                    path.display().to_string(),
                    true,
                ));
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(EvalError::Io(error)),
    }
    let mut sources = Vec::with_capacity(roles.len());
    let mut opaque_preferences = false;
    let mut api_key_helper = false;
    let mut policy_helper = false;
    let mut external_execution = false;
    for (role, path, parse_json) in roles {
        let path = Path::new(&path);
        let metadata = match fs::symlink_metadata(path) {
            Ok(metadata) => Some(metadata),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(EvalError::Io(error)),
        };
        let Some(metadata) = metadata else {
            sources.push(ClaudeManagedPolicySourceAudit {
                role: role.to_string(),
                path: path.display().to_string(),
                present: false,
                sha256: None,
                size_bytes: None,
                mode: None,
                owner_uid: None,
                security_relevant_keys: Vec::new(),
            });
            continue;
        };
        let mode = metadata.permissions().mode() & 0o777;
        if metadata.file_type().is_symlink()
            || !metadata.is_file()
            || metadata.nlink() != 1
            || metadata.uid() != 0
            || mode & 0o022 != 0
            || metadata.len() == 0
            || metadata.len() > 64 * 1024
        {
            return Err(invalid(format!(
                "Claude managed-policy source is not a bounded root-owned regular file: {}",
                path.display()
            )));
        }
        let bytes = fs::read(path)?;
        let mut security_relevant_keys = BTreeSet::new();
        if parse_json {
            let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|_| {
                invalid(format!(
                    "Claude managed-policy JSON is invalid: {}",
                    path.display()
                ))
            })?;
            collect_managed_policy_security_keys(&value, &mut security_relevant_keys);
        } else {
            opaque_preferences = true;
        }
        api_key_helper |= security_relevant_keys.contains("apikeyhelper");
        policy_helper |= security_relevant_keys.contains("policyhelper");
        external_execution |= role == "managed_mcp_json"
            || security_relevant_keys.iter().any(|key| {
                matches!(
                    key.as_str(),
                    "apikeyhelper"
                        | "policyhelper"
                        | "hooks"
                        | "mcpservers"
                        | "plugins"
                        | "enabledplugins"
                )
            });
        sources.push(ClaudeManagedPolicySourceAudit {
            role,
            path: path.display().to_string(),
            present: true,
            sha256: Some(sha256_bytes(&bytes)),
            size_bytes: Some(metadata.len()),
            mode: Some(mode),
            owner_uid: Some(metadata.uid()),
            security_relevant_keys: security_relevant_keys.into_iter().collect(),
        });
    }
    Ok((
        sources,
        opaque_preferences,
        api_key_helper,
        policy_helper,
        external_execution,
    ))
}

fn collect_managed_policy_security_keys(
    value: &serde_json::Value,
    observed: &mut BTreeSet<String>,
) {
    match value {
        serde_json::Value::Object(object) => {
            for (key, value) in object {
                let normalized = key.to_ascii_lowercase();
                if matches!(
                    normalized.as_str(),
                    "apikeyhelper"
                        | "policyhelper"
                        | "hooks"
                        | "mcpservers"
                        | "plugins"
                        | "enabledplugins"
                        | "permissions"
                        | "allowmanagedhooks"
                ) {
                    observed.insert(normalized);
                }
                collect_managed_policy_security_keys(value, observed);
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                collect_managed_policy_security_keys(value, observed);
            }
        }
        _ => {}
    }
}

fn claude_managed_policy_attestation_digest(
    attestation: &ClaudeManagedPolicyAttestation,
) -> EvalResult<String> {
    let mut digest_surface = attestation.clone();
    digest_surface.attestation_sha256.clear();
    sha256_serialized(&digest_surface)
}

/// Build a bounded, content-free receipt for a future runner-owned CONNECT proxy.
fn build_claude_provider_proxy_receipt(
    transport: &ClaudeNarrowTransportContract,
    session: &ClaudeProviderProxySessionBinding,
    requests: &[ClaudeConnectRequestAudit],
    client_to_upstream_bytes: u64,
    upstream_to_client_bytes: u64,
    started_unix_ms: u64,
    finished_unix_ms: u64,
) -> EvalResult<ClaudeProviderProxyReceipt> {
    validate_claude_transport(transport)?;
    validate_proxy_session_binding(session)?;
    if requests.is_empty()
        || requests.len() > transport.max_connect_requests as usize
        || requests.iter().any(|request| {
            request.raw_request_retained
                || request.body_bytes != 0
                || request.header_bytes == 0
                || request.header_bytes > transport.max_connect_header_bytes
        })
        || started_unix_ms < session.provider_started_unix_ms
        || finished_unix_ms < started_unix_ms
        || finished_unix_ms.saturating_sub(started_unix_ms) > transport.total_timeout_ms
        || finished_unix_ms
            > session
                .proxy_started_unix_ms
                .saturating_add(transport.total_timeout_ms)
        || !timestamp_is_recent(started_unix_ms, 15 * 60 * 1_000)?
        || !timestamp_is_recent(finished_unix_ms, 15 * 60 * 1_000)?
        || client_to_upstream_bytes > transport.max_tunnel_bytes_each_direction
        || upstream_to_client_bytes > transport.max_tunnel_bytes_each_direction
    {
        return Err(invalid("Claude provider proxy observation is not bounded"));
    }
    let accepted = requests.iter().filter(|request| request.accepted).count();
    let rejected = requests.len() - accepted;
    if accepted == 0
        || rejected != 0
        || requests
            .iter()
            .filter(|request| request.accepted)
            .any(|request| {
                request.method != "CONNECT"
                    || request.target != transport.provider_connect_authority
            })
    {
        return Err(invalid(
            "Claude provider proxy did not accept the exact CONNECT authority",
        ));
    }
    let accepted_connect_count =
        u32::try_from(accepted).map_err(|_| invalid("Claude proxy accepted-count overflow"))?;
    let rejected_request_count =
        u32::try_from(rejected).map_err(|_| invalid("Claude proxy rejected-count overflow"))?;
    let mut receipt = ClaudeProviderProxyReceipt {
        valid: true,
        target: transport.provider_connect_authority.clone(),
        accepted_connect_count,
        rejected_request_count,
        client_to_upstream_bytes,
        upstream_to_client_bytes,
        started_unix_ms,
        finished_unix_ms,
        raw_headers_or_body_retained: false,
        session: session.clone(),
        receipt_sha256: String::new(),
    };
    receipt.receipt_sha256 = sha256_serialized(&(
        &receipt.target,
        receipt.accepted_connect_count,
        receipt.rejected_request_count,
        receipt.client_to_upstream_bytes,
        receipt.upstream_to_client_bytes,
        receipt.started_unix_ms,
        receipt.finished_unix_ms,
        receipt.raw_headers_or_body_retained,
        &receipt.session,
    ))?;
    Ok(receipt)
}

pub fn validate_claude_provider_proxy_receipt(
    transport: &ClaudeNarrowTransportContract,
    session: &ClaudeProviderProxySessionBinding,
    receipt: &ClaudeProviderProxyReceipt,
) -> EvalResult<()> {
    validate_claude_transport(transport)?;
    validate_proxy_session_binding(session)?;
    let expected = sha256_serialized(&(
        &receipt.target,
        receipt.accepted_connect_count,
        receipt.rejected_request_count,
        receipt.client_to_upstream_bytes,
        receipt.upstream_to_client_bytes,
        receipt.started_unix_ms,
        receipt.finished_unix_ms,
        receipt.raw_headers_or_body_retained,
        &receipt.session,
    ))?;
    if !receipt.valid
        || receipt.target != transport.provider_connect_authority
        || receipt.accepted_connect_count == 0
        || receipt.accepted_connect_count > transport.max_connect_requests
        || receipt.rejected_request_count != 0
        || receipt.client_to_upstream_bytes > transport.max_tunnel_bytes_each_direction
        || receipt.upstream_to_client_bytes > transport.max_tunnel_bytes_each_direction
        || receipt.started_unix_ms == 0
        || receipt.started_unix_ms < session.provider_started_unix_ms
        || receipt.finished_unix_ms < receipt.started_unix_ms
        || receipt
            .finished_unix_ms
            .saturating_sub(receipt.started_unix_ms)
            > transport.total_timeout_ms
        || receipt.finished_unix_ms
            > session
                .proxy_started_unix_ms
                .saturating_add(transport.total_timeout_ms)
        || !timestamp_is_recent(receipt.started_unix_ms, 15 * 60 * 1_000)?
        || !timestamp_is_recent(receipt.finished_unix_ms, 15 * 60 * 1_000)?
        || receipt.raw_headers_or_body_retained
        || &receipt.session != session
        || receipt.receipt_sha256 != expected
    {
        return Err(invalid("Claude provider proxy receipt drifted"));
    }
    Ok(())
}

/// Run a real local CONNECT exchange through the generated Seatbelt profile and a fake upstream.
///
/// This is a provider-free enforcement probe. The trusted runner owns the loopback listener,
/// records the actual sandboxed client PID, classifies the live request, and creates the receipt;
/// callers cannot supply request observations. It deliberately does not authorize a real provider
/// launch because the production resolver/TLS relay and Claude managed-policy boundary remain
/// outside this standalone module.
#[cfg(target_os = "macos")]
pub fn execute_claude_provider_free_transport_probe(
    spec: &ClaudeSeatbeltSpec,
    profile_path: &Path,
    launch_sha256: &str,
    invocation_id: &str,
    environment: &BTreeMap<String, String>,
) -> EvalResult<ClaudeProviderFreeTransportProbeAudit> {
    validate_seatbelt_spec(spec)?;
    validate_claude_probe_environment(environment)?;
    if !is_sha256(launch_sha256) {
        return Err(invalid("fake transport probe lacks its launch digest"));
    }
    let profile_bytes = generate_claude_seatbelt_profile(spec)?;
    require_exact_file_bytes(profile_path, profile_bytes.as_bytes(), true)?;
    let profile_sha256 = sha256_bytes(profile_bytes.as_bytes());
    let secret = generate_claude_proxy_invocation_secret(invocation_id)?;
    let expected_proxy_authorization = secret.proxy_authorization_value();
    let child_proxy_authorization = secret.proxy_authorization_value();
    let proxy = TcpListener::bind((
        spec.transport.provider_proxy_host.as_str(),
        spec.transport.provider_proxy_port,
    ))?;
    let fake_upstream = TcpListener::bind(("127.0.0.1", 0))?;
    proxy.set_nonblocking(true)?;
    fake_upstream.set_nonblocking(true)?;
    let fake_upstream_address = fake_upstream.local_addr()?;
    let transport = spec.transport.clone();
    let proxy_started_unix_ms = now_unix_ms()?;

    let upstream_thread = std::thread::spawn(move || -> EvalResult<(u64, u64)> {
        let (mut stream, _) = accept_with_timeout(&fake_upstream, Duration::from_secs(3))?;
        stream.set_read_timeout(Some(Duration::from_secs(2)))?;
        stream.set_write_timeout(Some(Duration::from_secs(2)))?;
        let mut input = [0_u8; 4];
        stream.read_exact(&mut input)?;
        if &input != b"ping" {
            return Err(invalid(
                "fake upstream received an unexpected bounded payload",
            ));
        }
        stream.write_all(b"pong")?;
        Ok((4, 4))
    });
    let proxy_transport = transport.clone();
    let proxy_thread = std::thread::spawn(move || -> EvalResult<_> {
        for expected_negative in ["missing", "wrong"] {
            let (mut client, peer) = accept_with_timeout(&proxy, Duration::from_secs(3))?;
            if !peer.ip().is_loopback() {
                return Err(invalid("fake proxy client did not originate on loopback"));
            }
            let timeout = Duration::from_millis(proxy_transport.idle_timeout_ms);
            client.set_read_timeout(Some(timeout))?;
            client.set_write_timeout(Some(timeout))?;
            let header =
                read_bounded_connect_header(&mut client, proxy_transport.max_connect_header_bytes)?;
            let request = inspect_claude_connect_request_with_auth(
                &proxy_transport,
                &header,
                Some(expected_proxy_authorization.as_bytes()),
            )?;
            if request.accepted {
                return Err(invalid(format!(
                    "fake proxy accepted the {expected_negative} credential probe"
                )));
            }
            client.write_all(
                b"HTTP/1.1 407 Proxy Authentication Required\r\nConnection: close\r\n\r\n",
            )?;
        }
        let (mut client, peer) = accept_with_timeout(&proxy, Duration::from_secs(3))?;
        if !peer.ip().is_loopback() {
            return Err(invalid("fake proxy client did not originate on loopback"));
        }
        let timeout = Duration::from_millis(proxy_transport.idle_timeout_ms);
        client.set_read_timeout(Some(timeout))?;
        client.set_write_timeout(Some(timeout))?;
        let started = now_unix_ms()?;
        let header =
            read_bounded_connect_header(&mut client, proxy_transport.max_connect_header_bytes)?;
        let request = inspect_claude_connect_request_with_auth(
            &proxy_transport,
            &header,
            Some(expected_proxy_authorization.as_bytes()),
        )?;
        if !request.accepted {
            return Err(invalid("live fake proxy rejected the correct credential"));
        }
        let mut upstream = TcpStream::connect_timeout(&fake_upstream_address, timeout)?;
        upstream.set_read_timeout(Some(timeout))?;
        upstream.set_write_timeout(Some(timeout))?;
        client.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")?;
        let mut client_payload = [0_u8; 4];
        client.read_exact(&mut client_payload)?;
        upstream.write_all(&client_payload)?;
        let mut upstream_payload = [0_u8; 4];
        upstream.read_exact(&mut upstream_payload)?;
        client.write_all(&upstream_payload)?;
        Ok((request, 4_u64, 4_u64, started, now_unix_ms()?, true, true))
    });

    let script = format!(
        r#"import os
import socket
p = {}
auth = os.environ.pop("ENGRAM_PROXY_AUTH")
def exchange(value, expected):
    stream = socket.create_connection(("127.0.0.1", p), 2)
    stream.settimeout(2)
    header = "CONNECT api.anthropic.com:443 HTTP/1.1\r\nHost: api.anthropic.com:443\r\n"
    if value is not None:
        header += "Proxy-Authorization: " + value + "\r\n"
    stream.sendall((header + "\r\n").encode())
    response = b""
    while not response.endswith(b"\r\n\r\n"):
        response += stream.recv(1)
    assert response.startswith(("HTTP/1.1 " + expected).encode()), repr(response)
    return stream
exchange(None, "407").close()
exchange("Basic wrong", "407").close()
stream = exchange(auth, "200")
stream.sendall(b"ping")
assert stream.recv(4) == b"pong"
"#,
        transport.provider_proxy_port
    );
    let argv = vec![
        spec.sandbox_exec.clone(),
        "-f".to_string(),
        profile_path.display().to_string(),
        "/usr/bin/python3".to_string(),
        "-c".to_string(),
        script,
    ];
    let provider_started_unix_ms = now_unix_ms()?;
    let mut command = Command::new(&argv[0]);
    command
        .args(&argv[1..])
        .env_clear()
        .envs(environment)
        .env("ENGRAM_PROXY_AUTH", child_proxy_authorization.as_str())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let child = command.spawn()?;
    drop(command);
    let provider_process_id = child.id();
    let output = child.wait_with_output()?;
    let exit_code = output
        .status
        .code()
        .ok_or_else(|| invalid("sandboxed fake client ended without an exit code"))?;
    let client_audit = IsolationSandboxCommandAudit {
        label: "provider_proxy_fake_upstream".to_string(),
        argv_sha256: sha256_serialized(&argv)?,
        exit_code,
        succeeded: output.status.success(),
        sandbox_denial_evidence: has_sandbox_denial_evidence(&output.stdout)
            || has_sandbox_denial_evidence(&output.stderr),
        stdout_sha256: sha256_bytes(&output.stdout),
        stderr_sha256: sha256_bytes(&output.stderr),
    };
    if output.stdout.len() + output.stderr.len() > 64 * 1024 {
        return Err(invalid(
            "sandboxed fake proxy client output exceeded its bound",
        ));
    }
    let proxy_result = proxy_thread
        .join()
        .map_err(|_| invalid("runner-owned fake proxy thread panicked"))?;
    let upstream_result = upstream_thread
        .join()
        .map_err(|_| invalid("fake upstream thread panicked"))?;
    let (
        request,
        client_bytes,
        upstream_bytes,
        started,
        finished,
        missing_proxy_secret_rejected,
        wrong_proxy_secret_rejected,
    ) = proxy_result?;
    let (fake_received, fake_sent) = upstream_result?;
    if !client_audit.succeeded {
        return Err(invalid("sandboxed fake proxy client did not complete"));
    }
    if (client_bytes, upstream_bytes) != (fake_received, fake_sent) {
        return Err(invalid(
            "fake transport byte counts are not end-to-end consistent",
        ));
    }
    let session = ClaudeProviderProxySessionBinding {
        invocation_id: invocation_id.to_string(),
        launch_sha256: launch_sha256.to_string(),
        seatbelt_profile_sha256: profile_sha256,
        proxy_auth_sha256: secret.salted_sha256().to_string(),
        provider_process_id,
        proxy_process_id: std::process::id(),
        proxy_started_unix_ms,
        provider_started_unix_ms,
    };
    let receipt = build_claude_provider_proxy_receipt(
        &transport,
        &session,
        &[request],
        client_bytes,
        upstream_bytes,
        started,
        finished,
    )?;
    let mut audit = ClaudeProviderFreeTransportProbeAudit {
        valid: true,
        session,
        receipt,
        sandboxed_client: client_audit,
        missing_proxy_secret_rejected,
        wrong_proxy_secret_rejected,
        intentional_negative_attempt_count: 2,
        fake_upstream_received_bytes: fake_received,
        fake_upstream_sent_bytes: fake_sent,
        raw_transport_bytes_retained: false,
        audit_sha256: String::new(),
    };
    audit.audit_sha256 = sha256_serialized(&(
        audit.valid,
        &audit.session,
        &audit.receipt,
        &audit.sandboxed_client,
        audit.missing_proxy_secret_rejected,
        audit.wrong_proxy_secret_rejected,
        audit.intentional_negative_attempt_count,
        audit.fake_upstream_received_bytes,
        audit.fake_upstream_sent_bytes,
        audit.raw_transport_bytes_retained,
    ))?;
    validate_claude_provider_free_transport_probe(spec, launch_sha256, invocation_id, &audit)?;
    Ok(audit)
}

#[cfg(not(target_os = "macos"))]
pub fn execute_claude_provider_free_transport_probe(
    _spec: &ClaudeSeatbeltSpec,
    _profile_path: &Path,
    _launch_sha256: &str,
    _invocation_id: &str,
    _environment: &BTreeMap<String, String>,
) -> EvalResult<ClaudeProviderFreeTransportProbeAudit> {
    Err(invalid(
        "Claude Seatbelt transport probing is only available on macOS",
    ))
}

pub fn validate_claude_provider_free_transport_probe(
    spec: &ClaudeSeatbeltSpec,
    expected_launch_sha256: &str,
    expected_invocation_id: &str,
    audit: &ClaudeProviderFreeTransportProbeAudit,
) -> EvalResult<()> {
    validate_seatbelt_spec(spec)?;
    validate_claude_provider_proxy_receipt(&spec.transport, &audit.session, &audit.receipt)?;
    let expected_profile_sha256 = sha256_bytes(generate_claude_seatbelt_profile(spec)?.as_bytes());
    let expected = sha256_serialized(&(
        audit.valid,
        &audit.session,
        &audit.receipt,
        &audit.sandboxed_client,
        audit.missing_proxy_secret_rejected,
        audit.wrong_proxy_secret_rejected,
        audit.intentional_negative_attempt_count,
        audit.fake_upstream_received_bytes,
        audit.fake_upstream_sent_bytes,
        audit.raw_transport_bytes_retained,
    ))?;
    if !audit.valid
        || !audit.sandboxed_client.succeeded
        || audit.sandboxed_client.label != "provider_proxy_fake_upstream"
        || audit.sandboxed_client.exit_code != 0
        || audit.sandboxed_client.sandbox_denial_evidence
        || !audit.missing_proxy_secret_rejected
        || !audit.wrong_proxy_secret_rejected
        || audit.intentional_negative_attempt_count != 2
        || !is_sha256(&audit.sandboxed_client.argv_sha256)
        || !is_sha256(&audit.sandboxed_client.stdout_sha256)
        || !is_sha256(&audit.sandboxed_client.stderr_sha256)
        || audit.session.launch_sha256 != expected_launch_sha256
        || audit.session.seatbelt_profile_sha256 != expected_profile_sha256
        || audit.session.invocation_id != expected_invocation_id
        || audit.receipt.accepted_connect_count != 1
        || audit.receipt.rejected_request_count != 0
        || audit.fake_upstream_received_bytes != 4
        || audit.fake_upstream_sent_bytes != 4
        || audit.fake_upstream_received_bytes != audit.receipt.client_to_upstream_bytes
        || audit.fake_upstream_sent_bytes != audit.receipt.upstream_to_client_bytes
        || audit.raw_transport_bytes_retained
        || audit.audit_sha256 != expected
    {
        return Err(invalid(
            "provider-free proxy/Seatbelt/fake-upstream execution receipt drifted",
        ));
    }
    Ok(())
}

/// Validate the exact system-resolver candidates which the production proxy may dial.
///
/// The runner resolves only `api.anthropic.com:443`, connects directly to one address from this
/// set, and never re-resolves a caller-provided authority. The frozen classifier rejects local and
/// well-known special-use ranges; end-to-end TLS/certificate verification by Claude, not this
/// classifier, authenticates the provider.
pub fn validate_claude_provider_resolution_candidates(
    transport: &ClaudeNarrowTransportContract,
    candidates: &[SocketAddr],
) -> EvalResult<Vec<String>> {
    validate_claude_transport(transport)?;
    if candidates.is_empty() || candidates.len() > usize::from(transport.max_resolved_addresses) {
        return Err(invalid(
            "Claude provider resolution is empty or exceeds its exact address bound",
        ));
    }
    if candidates.iter().any(|address| {
        address.port() != CLAUDE_PROVIDER_PORT || !provider_ip_is_public(address.ip())
    }) {
        return Err(invalid(
            "Claude provider resolution contains a locally scoped/special or wrong-port address",
        ));
    }
    let mut exact = candidates
        .iter()
        .map(SocketAddr::to_string)
        .collect::<Vec<_>>();
    exact.sort();
    exact.dedup();
    if exact.len() != candidates.len() {
        return Err(invalid(
            "Claude provider resolution contains duplicate addresses",
        ));
    }
    Ok(exact)
}

pub fn validate_claude_provider_resolution_audit(
    transport: &ClaudeNarrowTransportContract,
    audit: &ClaudeProviderResolutionAudit,
) -> EvalResult<()> {
    validate_claude_transport(transport)?;
    let candidates = audit
        .resolved_addresses
        .iter()
        .map(|address| {
            address
                .parse::<SocketAddr>()
                .map_err(|_| invalid("Claude provider resolution contains an invalid address"))
        })
        .collect::<EvalResult<Vec<_>>>()?;
    let exact = validate_claude_provider_resolution_candidates(transport, &candidates)?;
    let expected = sha256_serialized(&(
        audit.valid,
        &audit.host,
        audit.port,
        &audit.resolver_argv_sha256,
        &audit.resolver_stdout_sha256,
        &audit.resolver_stderr_sha256,
        audit.resolver_exit_code,
        audit.resolver_output_bytes,
        audit.resolver_observed_by_runner,
        &audit.resolved_addresses,
        &audit.selected_address,
        audit.resolution_started_unix_ms,
        audit.resolution_finished_unix_ms,
        audit.raw_dns_response_retained,
    ))?;
    if !audit.valid
        || audit.host != CLAUDE_PROVIDER_HOST
        || audit.port != CLAUDE_PROVIDER_PORT
        || !is_sha256(&audit.resolver_argv_sha256)
        || !is_sha256(&audit.resolver_stdout_sha256)
        || !is_sha256(&audit.resolver_stderr_sha256)
        || audit.resolver_exit_code != 0
        || audit.resolver_output_bytes == 0
        || audit.resolver_output_bytes > 64 * 1024
        || !audit.resolver_observed_by_runner
        || audit.resolved_addresses != exact
        || !audit
            .resolved_addresses
            .iter()
            .any(|address| address == &audit.selected_address)
        || audit.resolution_started_unix_ms == 0
        || audit.resolution_finished_unix_ms < audit.resolution_started_unix_ms
        || audit
            .resolution_finished_unix_ms
            .saturating_sub(audit.resolution_started_unix_ms)
            > transport.total_timeout_ms
        || !timestamp_is_recent(audit.resolution_started_unix_ms, 15 * 60 * 1_000)?
        || !timestamp_is_recent(audit.resolution_finished_unix_ms, 15 * 60 * 1_000)?
        || audit.raw_dns_response_retained
        || audit.audit_sha256 != expected
    {
        return Err(invalid(
            "Claude provider resolution audit is not exact and hash-bound",
        ));
    }
    Ok(())
}

#[cfg(target_os = "macos")]
struct BoundedClaudeResolution {
    addresses: Vec<SocketAddr>,
    argv_sha256: String,
    stdout_sha256: String,
    stderr_sha256: String,
    exit_code: i32,
    output_bytes: u64,
}

#[cfg(target_os = "macos")]
fn resolve_claude_provider_bounded(
    transport: &ClaudeNarrowTransportContract,
    deadline: Instant,
) -> EvalResult<BoundedClaudeResolution> {
    let argv = vec![
        "/usr/bin/dscacheutil".to_string(),
        "-q".to_string(),
        "host".to_string(),
        "-a".to_string(),
        "name".to_string(),
        CLAUDE_PROVIDER_HOST.to_string(),
    ];
    let mut child = Command::new(&argv[0])
        .args(&argv[1..])
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("LANG", "C.UTF-8")
        .env("LC_ALL", "C.UTF-8")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| invalid("bounded resolver stdout pipe is absent"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| invalid("bounded resolver stderr pipe is absent"))?;
    let stdout_reader = std::thread::spawn(move || read_bounded_output(stdout, 64 * 1024));
    let stderr_reader = std::thread::spawn(move || read_bounded_output(stderr, 64 * 1024));
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if Instant::now() >= deadline {
            child.kill()?;
            let _ = child.wait();
            return Err(invalid(
                "exact Claude provider DNS process exceeded the wall deadline",
            ));
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    let stdout = stdout_reader
        .join()
        .map_err(|_| invalid("bounded resolver stdout reader panicked"))??;
    let stderr = stderr_reader
        .join()
        .map_err(|_| invalid("bounded resolver stderr reader panicked"))??;
    let output_bytes = u64::try_from(stdout.len() + stderr.len())
        .map_err(|_| invalid("bounded resolver output byte count overflow"))?;
    let exit_code = status
        .code()
        .ok_or_else(|| invalid("bounded resolver ended without an exit code"))?;
    if exit_code != 0 || !stderr.is_empty() || stdout.is_empty() || output_bytes > 64 * 1024 {
        return Err(invalid(
            "exact Claude provider DNS process did not complete cleanly",
        ));
    }
    let output = std::str::from_utf8(&stdout)
        .map_err(|_| invalid("bounded resolver output is not UTF-8"))?;
    let mut addresses = Vec::new();
    for line in output.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        if !matches!(key.trim(), "ip_address" | "ipv6_address") {
            continue;
        }
        if addresses.len() >= usize::from(transport.max_resolved_addresses) {
            return Err(invalid(
                "Claude provider resolver emitted more than the frozen raw address bound",
            ));
        }
        let ip = value
            .trim()
            .parse::<IpAddr>()
            .map_err(|_| invalid("bounded resolver emitted an invalid IP address"))?;
        addresses.push(SocketAddr::new(ip, CLAUDE_PROVIDER_PORT));
    }
    validate_claude_provider_resolution_candidates(transport, &addresses)?;
    Ok(BoundedClaudeResolution {
        addresses,
        argv_sha256: sha256_serialized(&argv)?,
        stdout_sha256: sha256_bytes(&stdout),
        stderr_sha256: sha256_bytes(&stderr),
        exit_code,
        output_bytes,
    })
}

/// Run one production CONNECT relay using the exact provider hostname and system resolver.
///
/// Merely compiling this function does not authorize its use. The caller must have already bound
/// the listener before launching the exact Claude process. This implementation independently
/// verifies the listener, current proxy PID, live provider PID/executable, exact CONNECT request,
/// resolver output, selected address, byte ceilings, and chronology. It never retains headers,
/// bodies, tunnel bytes, or credentials in its receipt.
#[cfg(target_os = "macos")]
pub fn execute_claude_production_connect_relay_once(
    transport: &ClaudeNarrowTransportContract,
    listener: TcpListener,
    session: &ClaudeProviderProxySessionBinding,
    provider_executable: &Path,
    provider_executable_sha256: &str,
    proxy_secret: &ClaudeProxyInvocationSecret,
) -> EvalResult<ClaudeProductionRelayAudit> {
    validate_claude_transport(transport)?;
    validate_proxy_session_binding(session)?;
    let observed_now_unix_ms = now_unix_ms()?;
    let deadline_unix_ms = session
        .proxy_started_unix_ms
        .checked_add(transport.total_timeout_ms)
        .ok_or_else(|| invalid("Claude proxy wall-clock deadline overflow"))?;
    let remaining_ms = deadline_unix_ms
        .checked_sub(observed_now_unix_ms)
        .ok_or_else(|| invalid("Claude proxy wall-clock deadline already passed"))?;
    let absolute_deadline = Instant::now() + Duration::from_millis(remaining_ms);
    require_normalized_absolute_path(provider_executable, "Claude provider process executable")?;
    let expected_executable = provider_executable.canonicalize()?;
    if session.proxy_process_id != std::process::id()
        || proxy_secret.invocation_id != session.invocation_id
        || proxy_secret.salted_sha256() != session.proxy_auth_sha256
        || listener.local_addr()?
            != SocketAddr::from(([127, 0, 0, 1], transport.provider_proxy_port))
        || !is_sha256(provider_executable_sha256)
        || read_regular_file_digest(&expected_executable)? != provider_executable_sha256
        || observed_process_executable(session.provider_process_id)? != expected_executable
    {
        return Err(invalid(
            "production Claude proxy is not bound to its exact listener/process executable",
        ));
    }
    require_process_alive(session.provider_process_id)?;
    listener.set_nonblocking(true)?;
    let timeout = Duration::from_millis(transport.idle_timeout_ms);
    let (mut client, peer) = accept_with_timeout(&listener, remaining_until(absolute_deadline)?)?;
    if !peer.ip().is_loopback() {
        return Err(invalid("production Claude proxy client is not loopback"));
    }
    client.set_read_timeout(Some(timeout))?;
    client.set_write_timeout(Some(timeout))?;
    let started_unix_ms = now_unix_ms()?;
    let header = read_bounded_connect_header_until(
        &mut client,
        transport.max_connect_header_bytes,
        absolute_deadline,
    )?;
    let expected_proxy_authorization = proxy_secret.proxy_authorization_value();
    let request = inspect_claude_connect_request_with_auth(
        transport,
        &header,
        Some(expected_proxy_authorization.as_bytes()),
    )?;
    if !request.accepted {
        let _ = client.write_all(b"HTTP/1.1 403 Forbidden\r\nConnection: close\r\n\r\n");
        return Err(invalid(
            "production Claude proxy rejected a non-exact CONNECT request",
        ));
    }
    let resolution_started_unix_ms = now_unix_ms()?;
    let resolved = resolve_claude_provider_bounded(transport, absolute_deadline)?;
    let exact_addresses =
        validate_claude_provider_resolution_candidates(transport, &resolved.addresses)?;
    let resolution_finished_unix_ms = now_unix_ms()?;
    let connect_timeout = Duration::from_millis(transport.connect_timeout_ms);
    let mut selected = None;
    let mut upstream = None;
    for address in &resolved.addresses {
        let remaining = remaining_until(absolute_deadline)?;
        let address_timeout = connect_timeout.min(remaining);
        if let Ok(stream) = TcpStream::connect_timeout(address, address_timeout) {
            selected = Some(*address);
            upstream = Some(stream);
            break;
        }
    }
    let selected = selected.ok_or_else(|| {
        invalid("exact Claude provider addresses were resolved but none accepted TCP")
    })?;
    let upstream = upstream.ok_or_else(|| invalid("Claude provider stream is absent"))?;
    upstream.set_read_timeout(Some(timeout))?;
    upstream.set_write_timeout(Some(timeout))?;
    client.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")?;
    let (client_to_upstream_bytes, upstream_to_client_bytes) = relay_bounded_bidirectional(
        client,
        upstream,
        transport.max_tunnel_bytes_each_direction,
        absolute_deadline,
    )?;
    require_process_alive(session.provider_process_id)?;
    let finished_unix_ms = now_unix_ms()?;
    let mut resolution = ClaudeProviderResolutionAudit {
        valid: true,
        host: CLAUDE_PROVIDER_HOST.to_string(),
        port: CLAUDE_PROVIDER_PORT,
        resolver_argv_sha256: resolved.argv_sha256,
        resolver_stdout_sha256: resolved.stdout_sha256,
        resolver_stderr_sha256: resolved.stderr_sha256,
        resolver_exit_code: resolved.exit_code,
        resolver_output_bytes: resolved.output_bytes,
        resolver_observed_by_runner: true,
        resolved_addresses: exact_addresses,
        selected_address: selected.to_string(),
        resolution_started_unix_ms,
        resolution_finished_unix_ms,
        raw_dns_response_retained: false,
        audit_sha256: String::new(),
    };
    resolution.audit_sha256 = sha256_serialized(&(
        resolution.valid,
        &resolution.host,
        resolution.port,
        &resolution.resolver_argv_sha256,
        &resolution.resolver_stdout_sha256,
        &resolution.resolver_stderr_sha256,
        resolution.resolver_exit_code,
        resolution.resolver_output_bytes,
        resolution.resolver_observed_by_runner,
        &resolution.resolved_addresses,
        &resolution.selected_address,
        resolution.resolution_started_unix_ms,
        resolution.resolution_finished_unix_ms,
        resolution.raw_dns_response_retained,
    ))?;
    validate_claude_provider_resolution_audit(transport, &resolution)?;
    let receipt = build_claude_provider_proxy_receipt(
        transport,
        session,
        &[request],
        client_to_upstream_bytes,
        upstream_to_client_bytes,
        started_unix_ms,
        finished_unix_ms,
    )?;
    let mut audit = ClaudeProductionRelayAudit {
        valid: true,
        authorizes_provider_execution: false,
        provider_executable_path: expected_executable.display().to_string(),
        provider_executable_sha256: provider_executable_sha256.to_string(),
        session: session.clone(),
        resolution,
        receipt,
        runner_observed_provider_process: true,
        malicious_same_uid_process_excluded: true,
        total_timeout_ms: transport.total_timeout_ms,
        deadline_unix_ms,
        completed_before_deadline: Instant::now() <= absolute_deadline,
        raw_transport_bytes_retained: false,
        audit_sha256: String::new(),
    };
    audit.audit_sha256 = claude_production_relay_audit_digest(&audit)?;
    validate_claude_production_relay_audit(transport, session, &audit)?;
    Ok(audit)
}

#[cfg(not(target_os = "macos"))]
pub fn execute_claude_production_connect_relay_once(
    _transport: &ClaudeNarrowTransportContract,
    _listener: std::net::TcpListener,
    _session: &ClaudeProviderProxySessionBinding,
    _provider_executable: &Path,
    _provider_executable_sha256: &str,
    _proxy_secret: &ClaudeProxyInvocationSecret,
) -> EvalResult<ClaudeProductionRelayAudit> {
    Err(invalid(
        "the exact production Claude relay is currently frozen for macOS only",
    ))
}

pub fn validate_claude_production_relay_audit(
    transport: &ClaudeNarrowTransportContract,
    session: &ClaudeProviderProxySessionBinding,
    audit: &ClaudeProductionRelayAudit,
) -> EvalResult<()> {
    validate_claude_transport(transport)?;
    validate_proxy_session_binding(session)?;
    validate_claude_provider_resolution_audit(transport, &audit.resolution)?;
    validate_claude_provider_proxy_receipt(transport, session, &audit.receipt)?;
    if !audit.valid
        || audit.authorizes_provider_execution
        || audit.session != *session
        || audit.receipt.session != *session
        || !Path::new(&audit.provider_executable_path).is_absolute()
        || !is_sha256(&audit.provider_executable_sha256)
        || !audit.runner_observed_provider_process
        || !audit.malicious_same_uid_process_excluded
        || audit.total_timeout_ms != transport.total_timeout_ms
        || audit.deadline_unix_ms
            != session
                .proxy_started_unix_ms
                .checked_add(transport.total_timeout_ms)
                .unwrap_or(0)
        || !audit.completed_before_deadline
        || audit.raw_transport_bytes_retained
        || audit.resolution.resolution_started_unix_ms < session.provider_started_unix_ms
        || audit.resolution.resolution_started_unix_ms < audit.receipt.started_unix_ms
        || audit.receipt.finished_unix_ms < audit.resolution.resolution_finished_unix_ms
        || audit.audit_sha256 != claude_production_relay_audit_digest(audit)?
    {
        return Err(invalid(
            "production Claude relay audit is incomplete, replayable, or authorizing",
        ));
    }
    Ok(())
}

fn claude_production_relay_audit_digest(audit: &ClaudeProductionRelayAudit) -> EvalResult<String> {
    sha256_serialized(&(
        audit.valid,
        audit.authorizes_provider_execution,
        &audit.provider_executable_path,
        &audit.provider_executable_sha256,
        &audit.session,
        &audit.resolution,
        &audit.receipt,
        audit.runner_observed_provider_process,
        audit.malicious_same_uid_process_excluded,
        audit.total_timeout_ms,
        audit.deadline_unix_ms,
        audit.completed_before_deadline,
        audit.raw_transport_bytes_retained,
    ))
}

fn provider_ip_is_public(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let octets = ip.octets();
            octets[0] != 0
                && !ip.is_unspecified()
                && !ip.is_loopback()
                && !ip.is_private()
                && !ip.is_link_local()
                && !ip.is_multicast()
                && !ip.is_broadcast()
                && !ip.is_documentation()
                && !(octets[0] == 100 && (64..=127).contains(&octets[1]))
                && !(octets[0] == 198 && matches!(octets[1], 18 | 19))
                && !(octets[0] == 192 && matches!(octets[1], 0 | 88))
                && octets[0] < 224
        }
        IpAddr::V6(ip) => {
            if let Some(ipv4) = ip.to_ipv4() {
                return provider_ip_is_public(IpAddr::V4(ipv4));
            }
            let segments = ip.segments();
            !ip.is_unspecified()
                && !ip.is_loopback()
                && !ip.is_multicast()
                && segments[0] & 0xfe00 != 0xfc00
                && segments[0] & 0xffc0 != 0xfe80
                && segments[0] & 0xffc0 != 0xfec0
                && !(segments[0] == 0x0100 && segments[1..].iter().all(|segment| *segment == 0))
                && !(segments[0] == 0x2001 && segments[1] == 0)
                && !(segments[0] == 0x2001 && segments[1] == 2)
                && !(segments[0] == 0x2001 && segments[1] == 0x0db8)
                && !(segments[0] == 0x3fff && segments[1] & 0xf000 == 0)
        }
    }
}

#[cfg(target_os = "macos")]
fn read_bounded_connect_header(stream: &mut TcpStream, max_bytes: u64) -> EvalResult<Vec<u8>> {
    read_bounded_connect_header_until(stream, max_bytes, Instant::now() + Duration::from_secs(30))
}

#[cfg(target_os = "macos")]
fn read_bounded_connect_header_until(
    stream: &mut TcpStream,
    max_bytes: u64,
    deadline: Instant,
) -> EvalResult<Vec<u8>> {
    let max_bytes = usize::try_from(max_bytes)
        .map_err(|_| invalid("Claude CONNECT header byte bound does not fit usize"))?;
    let mut header = Vec::new();
    let mut byte = [0_u8; 1];
    while !header.ends_with(b"\r\n\r\n") {
        stream.set_read_timeout(Some(remaining_until(deadline)?))?;
        stream.read_exact(&mut byte)?;
        header.push(byte[0]);
        if header.len() > max_bytes {
            return Err(invalid("Claude CONNECT header exceeded its byte bound"));
        }
    }
    Ok(header)
}

#[cfg(target_os = "macos")]
fn relay_bounded_bidirectional(
    client: TcpStream,
    upstream: TcpStream,
    max_bytes: u64,
    deadline: Instant,
) -> EvalResult<(u64, u64)> {
    let watchdog_client = client.try_clone()?;
    let watchdog_upstream = upstream.try_clone()?;
    let (cancel_sender, cancel_receiver) = std::sync::mpsc::sync_channel::<()>(1);
    let remaining = remaining_until(deadline)?;
    let watchdog = std::thread::spawn(move || {
        if cancel_receiver.recv_timeout(remaining).is_err() {
            let _ = watchdog_client.shutdown(Shutdown::Both);
            let _ = watchdog_upstream.shutdown(Shutdown::Both);
        }
    });
    let client_reader = client.try_clone()?;
    let upstream_reader = upstream.try_clone()?;
    let client_to_upstream =
        std::thread::spawn(move || copy_bounded_and_shutdown(client_reader, upstream, max_bytes));
    let upstream_to_client =
        std::thread::spawn(move || copy_bounded_and_shutdown(upstream_reader, client, max_bytes));
    let sent = client_to_upstream
        .join()
        .map_err(|_| invalid("Claude relay client-to-provider thread panicked"))??;
    let received = upstream_to_client
        .join()
        .map_err(|_| invalid("Claude relay provider-to-client thread panicked"))??;
    let _ = cancel_sender.send(());
    watchdog
        .join()
        .map_err(|_| invalid("Claude relay deadline watchdog panicked"))?;
    if Instant::now() > deadline {
        return Err(invalid(
            "Claude provider relay exceeded its wall-clock deadline",
        ));
    }
    Ok((sent, received))
}

#[cfg(target_os = "macos")]
fn remaining_until(deadline: Instant) -> EvalResult<Duration> {
    deadline
        .checked_duration_since(Instant::now())
        .filter(|remaining| !remaining.is_zero())
        .ok_or_else(|| invalid("Claude provider relay exceeded its wall-clock deadline"))
}

#[cfg(target_os = "macos")]
fn copy_bounded_and_shutdown(
    reader: TcpStream,
    mut writer: TcpStream,
    max_bytes: u64,
) -> EvalResult<u64> {
    let mut bounded = reader.take(max_bytes.saturating_add(1));
    let copied = std::io::copy(&mut bounded, &mut writer)?;
    let _ = writer.shutdown(Shutdown::Write);
    if copied > max_bytes {
        return Err(invalid("Claude provider tunnel exceeded its byte ceiling"));
    }
    Ok(copied)
}

#[cfg(target_os = "macos")]
fn observed_process_executable(process_id: u32) -> EvalResult<PathBuf> {
    let process_id = i32::try_from(process_id)
        .map_err(|_| invalid("process identifier does not fit macOS pid_t"))?;
    let mut buffer = vec![0_u8; libc::PROC_PIDPATHINFO_MAXSIZE as usize];
    // SAFETY: `buffer` is writable for the supplied length and `proc_pidpath` does not retain it.
    let length = unsafe {
        libc::proc_pidpath(
            process_id,
            buffer.as_mut_ptr().cast::<libc::c_void>(),
            u32::try_from(buffer.len())
                .map_err(|_| invalid("proc_pidpath buffer length overflow"))?,
        )
    };
    if length <= 0 {
        return Err(invalid(
            "macOS could not observe the expected live process path",
        ));
    }
    let length = usize::try_from(length).map_err(|_| invalid("negative process path length"))?;
    buffer.truncate(length);
    let path =
        std::str::from_utf8(&buffer).map_err(|_| invalid("observed process path is not UTF-8"))?;
    Path::new(path).canonicalize().map_err(EvalError::Io)
}

#[cfg(target_os = "macos")]
fn require_process_alive(process_id: u32) -> EvalResult<()> {
    let process_id = i32::try_from(process_id)
        .map_err(|_| invalid("process identifier does not fit macOS pid_t"))?;
    // SAFETY: signal zero performs a liveness/permission check and sends no signal.
    if unsafe { libc::kill(process_id, 0) } != 0 {
        return Err(invalid("expected provider/daemon process is not alive"));
    }
    Ok(())
}

fn now_unix_ms() -> EvalResult<u64> {
    u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| invalid("system clock precedes the Unix epoch"))?
            .as_millis(),
    )
    .map_err(|_| invalid("Unix timestamp does not fit u64"))
}

fn timestamp_is_recent(timestamp_unix_ms: u64, max_age_ms: u64) -> EvalResult<bool> {
    let now = now_unix_ms()?;
    Ok(timestamp_unix_ms <= now.saturating_add(5_000)
        && now.saturating_sub(timestamp_unix_ms) <= max_age_ms)
}

#[cfg(target_os = "macos")]
fn accept_with_timeout(
    listener: &TcpListener,
    timeout: Duration,
) -> EvalResult<(TcpStream, std::net::SocketAddr)> {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        match listener.accept() {
            Ok((stream, address)) => {
                stream.set_nonblocking(false)?;
                return Ok((stream, address));
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                if std::time::Instant::now() >= deadline {
                    return Err(invalid("provider-free fake transport accept timed out"));
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            Err(error) => return Err(EvalError::Io(error)),
        }
    }
}

fn validate_seatbelt_probe_targets(
    protected_paths: &[String],
    targets: &[ClaudeSeatbeltProbeTarget],
    existing_read_canary: bool,
) -> EvalResult<()> {
    let expected = protected_paths.iter().cloned().collect::<BTreeSet<_>>();
    let observed = targets
        .iter()
        .map(|target| target.protected_path.clone())
        .collect::<BTreeSet<_>>();
    if expected.len() != protected_paths.len()
        || observed.len() != targets.len()
        || expected != observed
    {
        return Err(invalid(
            "Seatbelt probe targets do not exactly cover protected paths",
        ));
    }
    for target in targets {
        let protected = Path::new(&target.protected_path);
        let canary = Path::new(&target.canary_path);
        require_normalized_absolute_path(protected, "Seatbelt protected path")?;
        require_normalized_absolute_path(canary, "Seatbelt canary")?;
        if canary == protected || !canary.starts_with(protected) {
            return Err(invalid(
                "Seatbelt canary must be a strict descendant of its protected path",
            ));
        }
        let protected = canonical_directory(protected)?;
        if existing_read_canary {
            let metadata = fs::symlink_metadata(canary)?;
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(invalid("Seatbelt read canary is not a regular file"));
            }
            require_single_link(&metadata, canary)?;
            let canonical = canary.canonicalize()?;
            if canonical.display().to_string() != target.canary_path
                || !canonical.starts_with(&protected)
            {
                return Err(invalid("Seatbelt read canary is not canonical"));
            }
        } else {
            if canary.exists() {
                return Err(invalid("Seatbelt write canary already exists"));
            }
            let parent = canonical_directory(
                canary
                    .parent()
                    .ok_or_else(|| invalid("Seatbelt write canary has no parent"))?,
            )?;
            if !parent.starts_with(&protected) {
                return Err(invalid(
                    "Seatbelt write canary parent escapes its protected path",
                ));
            }
        }
    }
    Ok(())
}

fn validate_probe_read_canary(path: &Path) -> EvalResult<()> {
    require_normalized_absolute_path(path, "sandbox read canary")?;
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(invalid("sandbox read canary is not a regular file"));
    }
    require_single_link(&metadata, path)?;
    if path.canonicalize()?.display().to_string() != path.display().to_string() {
        return Err(invalid("sandbox read canary must be canonical"));
    }
    Ok(())
}

fn validate_claude_probe_environment(environment: &BTreeMap<String, String>) -> EvalResult<()> {
    let expected = BTreeMap::from([
        ("LANG".to_string(), "C.UTF-8".to_string()),
        ("LC_ALL".to_string(), "C.UTF-8".to_string()),
        ("NO_COLOR".to_string(), "1".to_string()),
        ("PATH".to_string(), "/usr/bin:/bin".to_string()),
    ]);
    if environment != &expected {
        return Err(invalid(
            "Claude Seatbelt probe environment is not the exact cleared allowlist",
        ));
    }
    Ok(())
}

fn validate_claude_probe_command_set(
    spec: &ClaudeSeatbeltSpec,
    profile_path: &Path,
    commands: &[ClaudeSeatbeltProbeCommand],
) -> EvalResult<()> {
    let mut expected_labels = vec!["profile_compile", "allowed_read"];
    expected_labels
        .extend(std::iter::repeat("forbidden_read").take(spec.forbidden_read_paths.len()));
    expected_labels
        .extend(std::iter::repeat("forbidden_write").take(spec.forbidden_write_paths.len()));
    expected_labels.push("provider_proxy_allowed");
    if spec.transport.engram_daemon_port.is_some() {
        expected_labels.push("engram_loopback_allowed");
    }
    expected_labels.extend([
        "network_denial",
        "network_errno_denial",
        "external_network_denial",
        "unexpected_bind_denial",
    ]);
    if commands
        .iter()
        .map(|command| command.label.as_str())
        .ne(expected_labels)
    {
        return Err(invalid("Claude Seatbelt probe matrix is not exact"));
    }
    let prefix = [
        spec.sandbox_exec.as_str(),
        "-f",
        profile_path
            .to_str()
            .ok_or_else(|| invalid("Seatbelt profile path is not UTF-8"))?,
    ];
    let mut reads = BTreeSet::new();
    let mut writes = BTreeSet::new();
    for command in commands {
        if command.argv.len() <= prefix.len()
            || command.argv[..prefix.len()]
                .iter()
                .map(String::as_str)
                .ne(prefix)
        {
            return Err(invalid("Claude Seatbelt probe prefix is not frozen"));
        }
        let tail = &command.argv[prefix.len()..];
        let valid = match command.label.as_str() {
            "profile_compile" => {
                command.protected_path.is_none()
                    && tail == ["/usr/bin/true"]
                    && command.expect_success
                    && !command.require_sandbox_denial_evidence
                    && command.must_remain_absent.is_none()
            }
            "allowed_read" => {
                command.protected_path.is_none()
                    && tail.len() == 2
                    && tail[0] == "/bin/cat"
                    && command.expect_success
                    && !command.require_sandbox_denial_evidence
                    && command.must_remain_absent.is_none()
            }
            "forbidden_read" => {
                command
                    .protected_path
                    .as_ref()
                    .is_some_and(|path| reads.insert(path.clone()))
                    && tail.len() == 2
                    && tail[0] == "/bin/cat"
                    && !command.expect_success
                    && command.require_sandbox_denial_evidence
                    && command.must_remain_absent.is_none()
            }
            "forbidden_write" => {
                command
                    .protected_path
                    .as_ref()
                    .is_some_and(|path| writes.insert(path.clone()))
                    && tail.len() == 2
                    && tail[0] == "/usr/bin/touch"
                    && !command.expect_success
                    && command.require_sandbox_denial_evidence
                    && command.must_remain_absent.as_deref()
                        == command.argv.last().map(String::as_str)
            }
            "provider_proxy_allowed" => {
                command.protected_path.is_none()
                    && tail
                        == [
                            "/usr/bin/nc".to_string(),
                            "-z".to_string(),
                            "-w".to_string(),
                            "1".to_string(),
                            spec.transport.provider_proxy_host.clone(),
                            spec.transport.provider_proxy_port.to_string(),
                        ]
                    && command.expect_success
                    && !command.require_sandbox_denial_evidence
                    && command.must_remain_absent.is_none()
            }
            "engram_loopback_allowed" => {
                command.protected_path.is_none()
                    && spec.transport.engram_daemon_port.is_some_and(|port| {
                        tail == [
                            "/usr/bin/nc".to_string(),
                            "-z".to_string(),
                            "-w".to_string(),
                            "1".to_string(),
                            "127.0.0.1".to_string(),
                            port.to_string(),
                        ]
                    })
                    && command.expect_success
                    && !command.require_sandbox_denial_evidence
                    && command.must_remain_absent.is_none()
            }
            "network_denial" => {
                command.protected_path.is_none()
                    && tail.len() == 6
                    && tail[..5] == ["/usr/bin/nc", "-z", "-w", "1", "127.0.0.1"]
                    && tail[5].parse::<u16>().is_ok_and(|port| port != 0)
                    && !command.expect_success
                    && !command.require_sandbox_denial_evidence
                    && command.must_remain_absent.is_none()
            }
            "network_errno_denial" => {
                command.protected_path.is_none()
                    && tail.len() == 7
                    && tail[..6] == ["/usr/bin/nc", "-z", "-v", "-w", "1", "127.0.0.1"]
                    && tail[6].parse::<u16>().is_ok_and(|port| port != 0)
                    && !command.expect_success
                    && command.require_sandbox_denial_evidence
                    && command.must_remain_absent.is_none()
            }
            "external_network_denial" => {
                command.protected_path.is_none()
                    && tail
                        == [
                            "/usr/bin/nc",
                            "-z",
                            "-v",
                            "-w",
                            "1",
                            CLAUDE_EXTERNAL_PROBE_ADDRESS,
                            CLAUDE_EXTERNAL_PROBE_PORT,
                        ]
                    && !command.expect_success
                    && command.require_sandbox_denial_evidence
                    && command.must_remain_absent.is_none()
            }
            "unexpected_bind_denial" => {
                command.protected_path.is_none()
                    && tail.len() == 3
                    && tail[0] == "/usr/bin/python3"
                    && tail[1] == "-c"
                    && parse_exact_python_bind_probe(&tail[2]).is_some_and(|port| {
                        port != spec.transport.provider_proxy_port
                            && spec.transport.engram_daemon_port != Some(port)
                    })
                    && !command.expect_success
                    && command.require_sandbox_denial_evidence
                    && command.must_remain_absent.is_none()
            }
            _ => false,
        };
        if !valid {
            return Err(invalid("Claude Seatbelt probe command is not frozen"));
        }
    }
    let denied_loopback_port = commands
        .iter()
        .find(|command| command.label == "network_denial")
        .and_then(|command| command.argv.last())
        .and_then(|port| port.parse::<u16>().ok());
    let unexpected_bind_port = commands
        .iter()
        .find(|command| command.label == "unexpected_bind_denial")
        .and_then(|command| command.argv.last())
        .and_then(|script| parse_exact_python_bind_probe(script));
    if reads != spec.forbidden_read_paths.iter().cloned().collect()
        || writes != spec.forbidden_write_paths.iter().cloned().collect()
        || commands
            .iter()
            .find(|command| command.label == "network_denial")
            .and_then(|command| command.argv.last())
            != commands
                .iter()
                .find(|command| command.label == "network_errno_denial")
                .and_then(|command| command.argv.last())
        || denied_loopback_port.is_none()
        || unexpected_bind_port.is_none()
        || denied_loopback_port == unexpected_bind_port
        || denied_loopback_port.is_some_and(|port| {
            port == spec.transport.provider_proxy_port
                || spec.transport.engram_daemon_port == Some(port)
        })
        || unexpected_bind_port.is_some_and(|port| {
            port == spec.transport.provider_proxy_port
                || spec.transport.engram_daemon_port == Some(port)
        })
    {
        return Err(invalid(
            "Claude Seatbelt probe commands do not cover every protected path",
        ));
    }
    let allowed = Path::new(
        commands[1]
            .argv
            .last()
            .ok_or_else(|| invalid("Claude allowed-read probe lacks a canary"))?,
    );
    validate_probe_read_canary(allowed)?;
    if spec
        .forbidden_read_paths
        .iter()
        .any(|path| allowed.starts_with(path))
    {
        return Err(invalid("Claude allowed canary is under a denied path"));
    }
    let read_targets = commands
        .iter()
        .filter(|command| command.label == "forbidden_read")
        .map(|command| ClaudeSeatbeltProbeTarget {
            protected_path: command.protected_path.clone().unwrap_or_default(),
            canary_path: command.argv.last().cloned().unwrap_or_default(),
        })
        .collect::<Vec<_>>();
    validate_seatbelt_probe_targets(&spec.forbidden_read_paths, &read_targets, true)?;
    let write_targets = commands
        .iter()
        .filter(|command| command.label == "forbidden_write")
        .map(|command| ClaudeSeatbeltProbeTarget {
            protected_path: command.protected_path.clone().unwrap_or_default(),
            canary_path: command.argv.last().cloned().unwrap_or_default(),
        })
        .collect::<Vec<_>>();
    validate_seatbelt_probe_targets(&spec.forbidden_write_paths, &write_targets, false)?;
    Ok(())
}

fn parse_exact_python_bind_probe(script: &str) -> Option<u16> {
    let prefix = "import socket; s=socket.socket(); s.bind(('127.0.0.1', ";
    let suffix = "))";
    script
        .strip_prefix(prefix)?
        .strip_suffix(suffix)?
        .parse::<u16>()
        .ok()
        .filter(|port| *port != 0)
}

fn execute_exact_network_control(
    wrapped_argv: &[String],
    wrapper_len: usize,
    environment: &BTreeMap<String, String>,
) -> EvalResult<IsolationSandboxCommandAudit> {
    let argv = wrapped_argv
        .get(wrapper_len..)
        .ok_or_else(|| invalid("network sandbox probe wrapper is truncated"))?;
    if argv.len() != 6
        || argv[0] != "/usr/bin/nc"
        || argv[1] != "-z"
        || argv[2] != "-w"
        || argv[3] != "1"
        || argv[4] != "127.0.0.1"
        || !argv[5].parse::<u16>().is_ok_and(|port| port != 0)
    {
        return Err(invalid(
            "network control is not the exact stripped /usr/bin/nc command",
        ));
    }
    let audit = run_sanitized_probe_command("network_control", argv, environment, false)?;
    if !audit.succeeded || audit.exit_code != 0 {
        return Err(invalid(
            "exact unwrapped network control failed under the cleared probe environment",
        ));
    }
    Ok(audit)
}

fn run_sanitized_probe_command(
    label: &str,
    argv: &[String],
    environment: &BTreeMap<String, String>,
    _requires_denial: bool,
) -> EvalResult<IsolationSandboxCommandAudit> {
    let executable = argv
        .first()
        .ok_or_else(|| invalid("sandbox probe command is empty"))?;
    let output = Command::new(executable)
        .args(&argv[1..])
        .env_clear()
        .envs(environment)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    let exit_code = output
        .status
        .code()
        .ok_or_else(|| invalid("sandbox probe terminated without an exit code"))?;
    Ok(IsolationSandboxCommandAudit {
        label: label.to_string(),
        argv_sha256: sha256_serialized(&argv)?,
        exit_code,
        succeeded: output.status.success(),
        sandbox_denial_evidence: has_sandbox_denial_evidence(&output.stdout)
            || has_sandbox_denial_evidence(&output.stderr),
        stdout_sha256: sha256_bytes(&output.stdout),
        stderr_sha256: sha256_bytes(&output.stderr),
    })
}

fn has_sandbox_denial_evidence(bytes: &[u8]) -> bool {
    let output = String::from_utf8_lossy(bytes).to_ascii_lowercase();
    [
        "operation not permitted",
        "permission denied",
        "sandbox violation",
        "sandbox: deny",
        "deny file-",
        "deny network",
        "blocked by sandbox",
        "network access denied",
    ]
    .iter()
    .any(|needle| output.contains(needle))
}

fn require_exact_denials(
    expected: &[String],
    observed: &BTreeMap<String, IsolationProbeOutcome>,
    label: &str,
) -> EvalResult<()> {
    let expected = expected.iter().cloned().collect::<BTreeSet<_>>();
    let actual = observed.keys().cloned().collect::<BTreeSet<_>>();
    if expected != actual
        || observed
            .values()
            .any(|outcome| *outcome != IsolationProbeOutcome::PermissionDenied)
    {
        return Err(invalid(format!(
            "{label} probe set did not fail with EPERM/EACCES"
        )));
    }
    Ok(())
}

/// Required path categories that must be named at the evaluation boundary.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum IsolationBoundaryPathClass {
    RunArtifacts,
    TeachingArtifacts,
    CredentialMaterial,
    NativeState,
    EngramState,
}

/// One canonical path that must not leak to the model-facing launch surfaces.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IsolationBoundaryPath {
    pub class: IsolationBoundaryPathClass,
    pub path: String,
}

/// Closed, lane-specific source/evaluation boundary used by launch and sandbox validation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IsolationBoundaryContract {
    pub schema_version: u32,
    pub host: IsolationHost,
    pub native_memory_enabled: bool,
    pub uses_engram: bool,
    pub source_root: String,
    pub evaluation_root: String,
    pub forbidden_paths: Vec<IsolationBoundaryPath>,
    pub forbidden_markers: Vec<String>,
}

/// Exact, environment-cleared provider launch envelope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IsolationLaunchContract {
    pub schema_version: u32,
    pub host: IsolationHost,
    /// Standalone provider-free foundations never authorize a real provider process.
    pub authorizes_provider_execution: bool,
    pub env_clear: bool,
    pub argv: Vec<String>,
    pub cwd: String,
    pub environment: BTreeMap<String, String>,
    pub prompt: String,
    pub provider_cli_version: String,
    pub provider_executable_sha256: String,
    /// Exact non-secret configuration surfaces read and scanned by the runner.
    pub config_files: Vec<IsolationFileBinding>,
    pub expected_tools: Vec<String>,
    pub native_memory_enabled: bool,
    pub uses_engram: bool,
    pub boundary_sha256: String,
    pub semantics: IsolationLaunchSemantics,
}

/// Hash-bound non-secret file in the exact launch surface.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct IsolationFileBinding {
    pub path: String,
    pub sha256: String,
}

/// Host-specific typed semantics from which argv and configuration are regenerated.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "host", rename_all = "snake_case")]
pub enum IsolationLaunchSemantics {
    Codex {
        output_schema: String,
        permission_profile: CodexPermissionProfile,
        arm: CodexEvaluationArm,
        permission_probe: Box<CodexSandboxProbeAudit>,
    },
    ClaudeCode {
        output_schema: String,
        settings: String,
        mcp_config: String,
        seatbelt_profile: String,
        seatbelt_spec: ClaudeSeatbeltSpec,
        seatbelt_probe: Box<ClaudeSeatbeltProbeAudit>,
        auto_memory_directory: Option<String>,
        engram: Option<IsolationEngramSurface>,
        engram_daemon: Option<Box<ClaudeEngramDaemonAttestation>>,
        limits: ClaudeEvaluationLimits,
        /// Managed settings are loaded by Claude even with `--restricted`; they are not claimed
        /// to be part of the cleared local-host environment.
        managed_settings_outside_exact_environment_claim: bool,
    },
}

/// Lossless protocol-supplied limits for one Claude evaluation phase.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeEvaluationLimits {
    pub model: String,
    pub max_turns: u32,
    /// One USD is exactly 1,000,000 micro-USD; argv formatting always uses six decimals.
    pub max_budget_micro_usd: u64,
}

/// Sanitized runner-parsed init and terminal trace evidence for a later Claude run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeIsolationTraceAttestation {
    pub trace_sha256: String,
    pub init_present: bool,
    pub terminal_present: bool,
    pub init_claude_code_version: String,
    pub init_cwd: String,
    pub init_output_style: String,
    pub init_model: String,
    pub init_mcp_servers: Vec<String>,
    pub init_mcp_server_statuses: BTreeMap<String, String>,
    pub init_tools: Vec<String>,
    pub init_permission_mode: String,
    pub init_plugins: Vec<String>,
    pub init_subagents: Vec<String>,
    pub init_skills: Vec<String>,
    pub init_slash_commands: Vec<String>,
    pub init_terminal_slash_commands: Vec<String>,
    pub init_capabilities: Vec<String>,
    pub init_api_key_source: String,
    pub init_analytics_disabled: bool,
    pub init_product_feedback_disabled: bool,
    pub managed_api_key_helper_attempted: bool,
    pub managed_api_key_helper_succeeded: bool,
    pub local_subscription_pure: bool,
    pub confounded_exploratory_only: bool,
    pub configured_max_turns: u32,
    pub configured_max_budget_micro_usd: u64,
    pub observed_turns: u32,
    pub reported_cost_micro_usd: u64,
    pub terminal_subtype: String,
    pub terminal_budget_exhausted: bool,
}

/// Frozen typed input for a Codex 0.153.3 evaluation launch.
#[derive(Debug, Clone)]
pub struct CodexIsolationLaunchSpec {
    pub executable: String,
    pub executable_sha256: String,
    pub cli_version: String,
    pub cwd: String,
    pub environment: BTreeMap<String, String>,
    pub prompt: String,
    pub output_schema: String,
    pub permission_profile: CodexPermissionProfile,
    pub arm: CodexEvaluationArm,
    pub permission_probe: CodexSandboxProbeAudit,
}

/// Frozen typed input for a Claude Code 2.1.260 evaluation launch.
#[derive(Debug, Clone)]
pub struct ClaudeIsolationLaunchSpec {
    pub executable: String,
    pub executable_sha256: String,
    pub cli_version: String,
    pub cwd: String,
    pub environment: BTreeMap<String, String>,
    pub prompt: String,
    pub output_schema: String,
    pub settings: String,
    pub mcp_config: String,
    pub seatbelt_profile: String,
    pub seatbelt_spec: ClaudeSeatbeltSpec,
    pub seatbelt_probe: ClaudeSeatbeltProbeAudit,
    pub auto_memory_directory: Option<String>,
    pub engram: Option<IsolationEngramSurface>,
    pub engram_daemon: Option<ClaudeEngramDaemonAttestation>,
    pub limits: ClaudeEvaluationLimits,
}

/// Build the only accepted Codex 0.153.3 evaluation argv and configuration contract.
pub fn build_codex_isolation_launch(
    spec: &CodexIsolationLaunchSpec,
    boundary: &IsolationBoundaryContract,
) -> EvalResult<IsolationLaunchContract> {
    validate_positional_prompt(&spec.prompt)?;
    if spec.cli_version != CODEX_CLI_VERSION
        || boundary.host != IsolationHost::Codex
        || boundary.native_memory_enabled != spec.arm.native_memory_enabled
        || boundary.uses_engram != spec.arm.engram.is_some()
        || spec.permission_profile.config_layer_name != CODEX_CONFIG_LAYER
        || spec.permission_profile.permission_profile_name != CODEX_PERMISSION_PROFILE
        || spec.permission_profile.evaluation_checkout != spec.cwd
        || !spec.permission_probe.valid
    {
        return Err(invalid("invalid typed Codex 0.153.3 launch specification"));
    }
    let (evaluation_root, _) = validate_boundary_shape(boundary)?;
    let codex_home = spec
        .environment
        .get("CODEX_HOME")
        .ok_or_else(|| invalid("Codex launch has no isolated CODEX_HOME"))?;
    let profile_path = Path::new(codex_home).join("isolation.config.toml");
    let mut pre_read_paths = vec![
        ("Codex executable", PathBuf::from(&spec.executable)),
        ("Codex evaluation cwd", PathBuf::from(&spec.cwd)),
        ("Codex output schema", PathBuf::from(&spec.output_schema)),
        ("Codex config layer", profile_path.clone()),
    ];
    if let Some(engram) = &spec.arm.engram {
        pre_read_paths.extend([
            ("Codex Engram executable", PathBuf::from(&engram.executable)),
            ("Codex Engram home", PathBuf::from(&engram.home)),
            ("Codex Engram skill", PathBuf::from(&engram.skill_path)),
        ]);
    }
    validate_pre_read_paths_disjoint_from_credential(boundary, &pre_read_paths)?;
    validate_provider_executable(
        Path::new(&spec.executable),
        &spec.executable_sha256,
        &spec.cli_version,
        &spec.environment,
        "Codex",
    )?;
    validate_codex_permission_profile(
        &spec.permission_profile,
        &evaluation_root,
        &spec.permission_probe.profile,
    )?;
    validate_codex_probe_audit(&spec.permission_probe)?;
    validate_engram_arm(
        spec.arm.engram.as_ref(),
        boundary,
        &spec.environment,
        &evaluation_root,
    )?;
    let expected_profile =
        generate_codex_evaluation_config_toml(&spec.permission_profile, &spec.arm)?;
    if spec.permission_probe.profile_sha256 != sha256_bytes(expected_profile.as_bytes()) {
        return Err(invalid(
            "Codex launch config bytes do not match its real probe receipt",
        ));
    }
    require_exact_file_bytes(&profile_path, expected_profile.as_bytes(), true)?;
    require_normalized_absolute_path(Path::new(&spec.output_schema), "Codex output schema")?;

    let mut config_files = vec![
        bound_file(Path::new(&spec.output_schema), true)?,
        bound_file(&profile_path, true)?,
    ];
    if let Some(engram) = &spec.arm.engram {
        config_files.push(bound_file(Path::new(&engram.skill_path), true)?);
    }
    let expected_tools = expected_tool_surface(IsolationHost::Codex, spec.arm.engram.is_some());
    let argv = vec![
        spec.executable.clone(),
        "exec".to_string(),
        "--profile".to_string(),
        CODEX_CONFIG_LAYER.to_string(),
        "--ignore-user-config".to_string(),
        "--ignore-rules".to_string(),
        "--strict-config".to_string(),
        "--ephemeral".to_string(),
        "--cd".to_string(),
        spec.cwd.clone(),
        "--json".to_string(),
        "--output-schema".to_string(),
        spec.output_schema.clone(),
        spec.prompt.clone(),
    ];
    let launch = IsolationLaunchContract {
        schema_version: ISOLATION_SCHEMA_VERSION,
        host: IsolationHost::Codex,
        authorizes_provider_execution: false,
        env_clear: true,
        argv,
        cwd: spec.cwd.clone(),
        environment: spec.environment.clone(),
        prompt: spec.prompt.clone(),
        provider_cli_version: spec.cli_version.clone(),
        provider_executable_sha256: spec.executable_sha256.clone(),
        config_files,
        expected_tools,
        native_memory_enabled: spec.arm.native_memory_enabled,
        uses_engram: spec.arm.engram.is_some(),
        boundary_sha256: sha256_serialized(boundary)?,
        semantics: IsolationLaunchSemantics::Codex {
            output_schema: spec.output_schema.clone(),
            permission_profile: spec.permission_profile.clone(),
            arm: spec.arm.clone(),
            permission_probe: Box::new(spec.permission_probe.clone()),
        },
    };
    validate_codex_native_memory_arm(&launch, boundary)?;
    Ok(launch)
}

/// Build the only accepted Seatbelt-wrapped Claude Code 2.1.260 evaluation contract.
pub fn build_claude_isolation_launch(
    spec: &ClaudeIsolationLaunchSpec,
    boundary: &IsolationBoundaryContract,
) -> EvalResult<IsolationLaunchContract> {
    validate_positional_prompt(&spec.prompt)?;
    if spec.cli_version != CLAUDE_CLI_VERSION
        || boundary.host != IsolationHost::ClaudeCode
        || boundary.native_memory_enabled != spec.auto_memory_directory.is_some()
        || boundary.uses_engram != spec.engram.is_some()
        || boundary.uses_engram != spec.engram_daemon.is_some()
        || !spec.seatbelt_probe.valid
    {
        return Err(invalid(
            "invalid typed Claude Code 2.1.260 launch specification",
        ));
    }
    validate_claude_limits(&spec.limits)?;
    validate_claude_transport_environment(
        &spec.environment,
        &spec.seatbelt_spec.transport,
        boundary.native_memory_enabled,
    )?;
    let (evaluation_root, _) = validate_boundary_shape(boundary)?;
    let mut pre_read_paths = vec![
        ("Claude executable", PathBuf::from(&spec.executable)),
        ("Claude evaluation cwd", PathBuf::from(&spec.cwd)),
        ("Claude output schema", PathBuf::from(&spec.output_schema)),
        ("Claude settings", PathBuf::from(&spec.settings)),
        ("Claude MCP config", PathBuf::from(&spec.mcp_config)),
        (
            "Claude Seatbelt profile",
            PathBuf::from(&spec.seatbelt_profile),
        ),
        (
            "Claude Seatbelt executable",
            PathBuf::from(&spec.seatbelt_spec.sandbox_exec),
        ),
    ];
    if let Some(auto_memory) = &spec.auto_memory_directory {
        pre_read_paths.push(("Claude auto-memory root", PathBuf::from(auto_memory)));
    }
    if let Some(engram) = &spec.engram {
        pre_read_paths.extend([
            (
                "Claude Engram executable",
                PathBuf::from(&engram.executable),
            ),
            ("Claude Engram home", PathBuf::from(&engram.home)),
            ("Claude Engram skill", PathBuf::from(&engram.skill_path)),
        ]);
    }
    if let Some(daemon) = &spec.engram_daemon {
        pre_read_paths.extend([
            (
                "Claude Engram daemon executable",
                PathBuf::from(&daemon.process_executable_path),
            ),
            (
                "Claude Engram daemon port receipt",
                PathBuf::from(&daemon.daemon_port_file.path),
            ),
            (
                "Claude Engram daemon PID receipt",
                PathBuf::from(&daemon.daemon_pid_file.path),
            ),
            (
                "Claude Engram daemon metadata receipt",
                PathBuf::from(&daemon.daemon_metadata_file.path),
            ),
        ]);
    }
    validate_pre_read_paths_disjoint_from_credential(boundary, &pre_read_paths)?;
    validate_provider_executable(
        Path::new(&spec.executable),
        &spec.executable_sha256,
        &spec.cli_version,
        &spec.environment,
        "Claude",
    )?;
    validate_claude_seatbelt_boundary(&spec.seatbelt_spec, boundary)?;
    let seatbelt_bytes = generate_claude_seatbelt_profile(&spec.seatbelt_spec)?;
    require_exact_file_bytes(
        Path::new(&spec.seatbelt_profile),
        seatbelt_bytes.as_bytes(),
        true,
    )?;
    if spec.seatbelt_probe.profile_sha256 != sha256_bytes(seatbelt_bytes.as_bytes()) {
        return Err(invalid(
            "Claude launch Seatbelt bytes do not match its real probe receipt",
        ));
    }
    validate_claude_seatbelt_probe(&spec.seatbelt_spec, &spec.seatbelt_probe.profile)?;
    validate_claude_probe_audit(&spec.seatbelt_spec, &spec.seatbelt_probe)?;
    validate_engram_arm(
        spec.engram.as_ref(),
        boundary,
        &spec.environment,
        &evaluation_root,
    )?;
    if let (Some(engram), Some(daemon)) = (&spec.engram, &spec.engram_daemon) {
        validate_claude_engram_daemon_attestation(engram, &spec.seatbelt_spec.transport, daemon)?;
    }
    validate_claude_auto_memory(
        spec.auto_memory_directory.as_deref(),
        boundary,
        &evaluation_root,
    )?;

    let expected_settings = generate_claude_settings(
        spec.auto_memory_directory.as_deref(),
        boundary.native_memory_enabled,
    )?;
    let expected_mcp = generate_claude_mcp_config(spec.engram.as_ref())?;
    require_exact_file_bytes(
        Path::new(&spec.settings),
        expected_settings.as_bytes(),
        true,
    )?;
    require_exact_file_bytes(Path::new(&spec.mcp_config), expected_mcp.as_bytes(), true)?;
    let output_schema = read_bound_file(
        Path::new(&spec.output_schema),
        &file_sha256(&spec.output_schema)?,
    )?;
    let output_schema_text = String::from_utf8(output_schema)
        .map_err(|_| invalid("Claude output schema must be UTF-8 JSON"))?;
    serde_json::from_str::<serde_json::Value>(&output_schema_text)
        .map_err(|error| invalid(format!("Claude output schema is invalid JSON: {error}")))?;

    let expected_tools = expected_tool_surface(IsolationHost::ClaudeCode, spec.engram.is_some());
    let mut argv = vec![
        spec.seatbelt_spec.sandbox_exec.clone(),
        "-f".to_string(),
        spec.seatbelt_profile.clone(),
        "--".to_string(),
        spec.executable.clone(),
        "--print".to_string(),
        "--output-format".to_string(),
        "stream-json".to_string(),
        "--verbose".to_string(),
        "--restricted".to_string(),
        "--permission-mode".to_string(),
        CLAUDE_PERMISSION_MODE.to_string(),
        "--strict-mcp-config".to_string(),
        "--no-session-persistence".to_string(),
        "--disable-slash-commands".to_string(),
        "--no-chrome".to_string(),
        "--setting-sources".to_string(),
        "".to_string(),
        "--permission-prompts".to_string(),
        "none".to_string(),
        "--settings".to_string(),
        spec.settings.clone(),
        "--mcp-config".to_string(),
        spec.mcp_config.clone(),
        "--add-dir".to_string(),
        spec.cwd.clone(),
        "--tools".to_string(),
        "Read".to_string(),
        "--disallowed-tools".to_string(),
        CLAUDE_DISALLOWED_TOOLS.to_string(),
    ];
    if spec.engram.is_some() {
        argv.extend(["--allowed-tools".to_string(), ENGRAM_AGENT_TOOLS.join(",")]);
    }
    argv.extend([
        "--model".to_string(),
        spec.limits.model.clone(),
        "--max-turns".to_string(),
        spec.limits.max_turns.to_string(),
        "--max-budget-usd".to_string(),
        format_micro_usd(spec.limits.max_budget_micro_usd)?,
        "--json-schema".to_string(),
        output_schema_text,
    ]);
    if let Some(engram) = &spec.engram {
        argv.extend([
            "--append-system-prompt-file".to_string(),
            engram.skill_path.clone(),
        ]);
    }
    argv.push(spec.prompt.clone());
    let mut config_files = vec![
        bound_file(Path::new(&spec.output_schema), true)?,
        bound_file(Path::new(&spec.settings), true)?,
        bound_file(Path::new(&spec.mcp_config), true)?,
        bound_file(Path::new(&spec.seatbelt_profile), true)?,
    ];
    if let Some(engram) = &spec.engram {
        config_files.push(bound_file(Path::new(&engram.skill_path), true)?);
    }
    if let Some(daemon) = &spec.engram_daemon {
        config_files.extend([
            daemon.daemon_port_file.clone(),
            daemon.daemon_pid_file.clone(),
            daemon.daemon_metadata_file.clone(),
        ]);
    }
    Ok(IsolationLaunchContract {
        schema_version: ISOLATION_SCHEMA_VERSION,
        host: IsolationHost::ClaudeCode,
        authorizes_provider_execution: false,
        env_clear: true,
        argv,
        cwd: spec.cwd.clone(),
        environment: spec.environment.clone(),
        prompt: spec.prompt.clone(),
        provider_cli_version: spec.cli_version.clone(),
        provider_executable_sha256: spec.executable_sha256.clone(),
        config_files,
        expected_tools,
        native_memory_enabled: boundary.native_memory_enabled,
        uses_engram: boundary.uses_engram,
        boundary_sha256: sha256_serialized(boundary)?,
        semantics: IsolationLaunchSemantics::ClaudeCode {
            output_schema: spec.output_schema.clone(),
            settings: spec.settings.clone(),
            mcp_config: spec.mcp_config.clone(),
            seatbelt_profile: spec.seatbelt_profile.clone(),
            seatbelt_spec: spec.seatbelt_spec.clone(),
            seatbelt_probe: Box::new(spec.seatbelt_probe.clone()),
            auto_memory_directory: spec.auto_memory_directory.clone(),
            engram: spec.engram.clone(),
            engram_daemon: spec.engram_daemon.clone().map(Box::new),
            limits: spec.limits.clone(),
            managed_settings_outside_exact_environment_claim: true,
        },
    })
}

fn expected_tool_surface(host: IsolationHost, uses_engram: bool) -> Vec<String> {
    let mut tools = match host {
        IsolationHost::Codex => vec!["shell_read_only".to_string()],
        IsolationHost::ClaudeCode => vec!["Read".to_string(), "StructuredOutput".to_string()],
    };
    if uses_engram {
        tools.extend(ENGRAM_AGENT_TOOLS.iter().map(|tool| (*tool).to_string()));
    }
    tools
}

fn validate_claude_limits(limits: &ClaudeEvaluationLimits) -> EvalResult<()> {
    if limits.model.trim() != limits.model
        || limits.model.is_empty()
        || limits.model.starts_with('-')
        || limits.model.chars().any(char::is_whitespace)
        || limits.max_turns == 0
        || limits.max_budget_micro_usd == 0
    {
        return Err(invalid(
            "Claude model, max turns, and micro-USD budget must be positive exact protocol inputs",
        ));
    }
    Ok(())
}

fn validate_positional_prompt(prompt: &str) -> EvalResult<()> {
    if prompt.is_empty() || prompt.starts_with('-') || prompt.contains('\0') {
        return Err(invalid(
            "provider prompt must be a nonempty unambiguous positional argument",
        ));
    }
    Ok(())
}

fn format_micro_usd(micro_usd: u64) -> EvalResult<String> {
    if micro_usd == 0 {
        return Err(invalid("Claude micro-USD budget must be positive"));
    }
    Ok(format!(
        "{}.{:06}",
        micro_usd / 1_000_000,
        micro_usd % 1_000_000
    ))
}

fn generate_claude_settings(
    auto_memory_directory: Option<&str>,
    native_memory_enabled: bool,
) -> EvalResult<String> {
    let value = if native_memory_enabled {
        serde_json::json!({
            "autoMemoryDirectory": auto_memory_directory
                .ok_or_else(|| invalid("Claude native arm lacks auto-memory directory"))?,
            "autoMemoryEnabled": true,
        })
    } else {
        if auto_memory_directory.is_some() {
            return Err(invalid(
                "Claude non-native arm unexpectedly names auto-memory state",
            ));
        }
        serde_json::json!({"autoMemoryEnabled": false})
    };
    Ok(serde_json::to_string(&value)? + "\n")
}

fn generate_claude_mcp_config(engram: Option<&IsolationEngramSurface>) -> EvalResult<String> {
    let value = if let Some(engram) = engram {
        validate_engram_surface(engram)?;
        serde_json::json!({
            "mcpServers": {
                "engram": {
                    "command": engram.executable,
                    "args": ["serve", "--project", engram.project, "--profile", "agent"],
                    "env": {"ENGRAM_HOME": engram.home}
                }
            }
        })
    } else {
        serde_json::json!({"mcpServers": {}})
    };
    Ok(serde_json::to_string(&value)? + "\n")
}

fn validate_provider_executable(
    path: &Path,
    expected_sha256: &str,
    expected_version: &str,
    environment: &BTreeMap<String, String>,
    label: &str,
) -> EvalResult<()> {
    require_normalized_absolute_path(path, &format!("{label} executable"))?;
    if !is_sha256(expected_sha256) || read_regular_file_digest(path)? != expected_sha256 {
        return Err(invalid(format!("{label} executable digest drifted")));
    }
    let mut child = Command::new(path)
        .arg("--version")
        .env_clear()
        .envs(environment)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| invalid(format!("{label} version stdout pipe is absent")))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| invalid(format!("{label} version stderr pipe is absent")))?;
    let stdout_reader = std::thread::spawn(move || read_bounded_output(stdout, 4_096));
    let stderr_reader = std::thread::spawn(move || read_bounded_output(stderr, 4_096));
    let deadline =
        std::time::Instant::now() + std::time::Duration::from_millis(PROVIDER_VERSION_TIMEOUT_MS);
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if std::time::Instant::now() >= deadline {
            child.kill()?;
            let _ = child.wait();
            return Err(invalid(format!(
                "{label} --version exceeded its ten-second bound"
            )));
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    };
    let stdout = stdout_reader
        .join()
        .map_err(|_| invalid(format!("{label} version stdout reader panicked")))??;
    let stderr = stderr_reader
        .join()
        .map_err(|_| invalid(format!("{label} version stderr reader panicked")))??;
    let expected_stdout = format!("{expected_version}\n");
    if !status.success() || stdout != expected_stdout.as_bytes() || !stderr.is_empty() {
        return Err(invalid(format!(
            "{label} executable did not report the exact frozen version"
        )));
    }
    Ok(())
}

fn validate_engram_arm(
    engram: Option<&IsolationEngramSurface>,
    boundary: &IsolationBoundaryContract,
    environment: &BTreeMap<String, String>,
    evaluation_root: &Path,
) -> EvalResult<()> {
    if boundary.uses_engram != engram.is_some()
        || boundary.uses_engram != environment.contains_key("ENGRAM_HOME")
    {
        return Err(invalid(
            "Engram environment/server/tool/skill surface does not match the lane arm",
        ));
    }
    let Some(engram) = engram else {
        return Ok(());
    };
    validate_engram_surface(engram)?;
    let home = canonical_directory(Path::new(&engram.home))?;
    require_private_directory(&home, "Engram home")?;
    if home != evaluation_root.join("engram-home")
        || environment.get("ENGRAM_HOME") != Some(&engram.home)
    {
        return Err(invalid("Engram home is not the exact isolated lane state"));
    }
    let skill = Path::new(&engram.skill_path);
    if !skill.starts_with(evaluation_root) {
        return Err(invalid(
            "Engram skill/instruction surface is outside the enclave",
        ));
    }
    let protected = boundary
        .forbidden_paths
        .iter()
        .find(|path| path.class == IsolationBoundaryPathClass::EngramState)
        .ok_or_else(|| invalid("Engram lane lacks a protected Engram state path"))?;
    let expected_project_state = home.join("projects").join(&engram.project);
    let project_state = canonical_directory(&expected_project_state)?;
    require_private_directory(&project_state, "Engram project state")?;
    if Path::new(&protected.path) != project_state
        || protected.path != project_state.display().to_string()
    {
        return Err(invalid(
            "protected Engram state is not the exact daemon project-state root",
        ));
    }
    Ok(())
}

fn validate_codex_native_memory_arm(
    launch: &IsolationLaunchContract,
    boundary: &IsolationBoundaryContract,
) -> EvalResult<()> {
    let codex_home = launch
        .environment
        .get("CODEX_HOME")
        .ok_or_else(|| invalid("Codex home is missing"))?;
    let expected = Path::new(codex_home).join("memories");
    let observed = boundary
        .forbidden_paths
        .iter()
        .find(|path| path.class == IsolationBoundaryPathClass::NativeState);
    if launch.native_memory_enabled {
        let observed = observed.ok_or_else(|| invalid("Codex native arm lacks memory state"))?;
        if Path::new(&observed.path) != expected {
            return Err(invalid("Codex native state is not CODEX_HOME/memories"));
        }
    } else if observed.is_some() {
        return Err(invalid("Codex non-native arm names native state"));
    }
    Ok(())
}

fn validate_claude_auto_memory(
    auto_memory_directory: Option<&str>,
    boundary: &IsolationBoundaryContract,
    evaluation_root: &Path,
) -> EvalResult<()> {
    let observed = boundary
        .forbidden_paths
        .iter()
        .find(|path| path.class == IsolationBoundaryPathClass::NativeState);
    match (auto_memory_directory, observed) {
        (Some(directory), Some(protected)) => {
            let directory = canonical_directory(Path::new(directory))?;
            require_private_directory(&directory, "Claude auto-memory")?;
            if directory != evaluation_root.join("claude-config/auto-memory")
                || protected.path != directory.display().to_string()
            {
                return Err(invalid(
                    "Claude native state is not the exact isolated auto-memory directory",
                ));
            }
        }
        (None, None) => {}
        _ => return Err(invalid("Claude auto-memory does not match the native arm")),
    }
    Ok(())
}

fn require_successful_network_control(audit: &IsolationSandboxCommandAudit) -> EvalResult<()> {
    if audit.label != "network_control"
        || !audit.succeeded
        || audit.exit_code != 0
        || audit.sandbox_denial_evidence
    {
        return Err(invalid(
            "sandbox probe receipt lacks the exact successful unwrapped network control",
        ));
    }
    Ok(())
}

fn validate_codex_probe_audit(audit: &CodexSandboxProbeAudit) -> EvalResult<()> {
    if !audit.valid || !is_sha256(&audit.profile_sha256) {
        return Err(invalid("Codex sandbox probe receipt is not valid"));
    }
    require_successful_network_control(&audit.network_control)?;
    validate_sanitized_probe_result(&audit.network_control, "network_control", true, false)?;
    let expected = [
        ("allowed_read", true, false),
        ("sibling_traversal", false, true),
        ("direct_forbidden_read", false, true),
        ("child_shell_forbidden_read", false, true),
        ("write_denial", false, true),
        ("network_denial", false, false),
        ("network_policy_denial", false, false),
        ("network_errno_denial", false, true),
    ];
    validate_exact_sanitized_probe_results(&audit.commands, &expected)?;
    if !audit.profile.allowed_read_succeeded
        || !audit.profile.sibling_traversal_denied
        || !audit.profile.direct_forbidden_read_denied
        || !audit.profile.child_shell_forbidden_read_denied
        || !audit.profile.write_denied
        || !audit.profile.network_denied
        || !audit.profile.network_sandbox_denial_observed
    {
        return Err(invalid(
            "Codex permission summary does not match the exact probe receipt",
        ));
    }
    Ok(())
}

fn validate_claude_probe_audit(
    spec: &ClaudeSeatbeltSpec,
    audit: &ClaudeSeatbeltProbeAudit,
) -> EvalResult<()> {
    if !audit.valid || !is_sha256(&audit.profile_sha256) {
        return Err(invalid("Claude Seatbelt probe receipt is not valid"));
    }
    require_successful_network_control(&audit.network_control)?;
    validate_sanitized_probe_result(&audit.network_control, "network_control", true, false)?;
    let mut expected = Vec::with_capacity(
        7 + usize::from(spec.transport.engram_daemon_port.is_some())
            + spec.forbidden_read_paths.len()
            + spec.forbidden_write_paths.len(),
    );
    expected.push(("profile_compile", true, false));
    expected.push(("allowed_read", true, false));
    expected.extend(
        std::iter::repeat(("forbidden_read", false, true)).take(spec.forbidden_read_paths.len()),
    );
    expected.extend(
        std::iter::repeat(("forbidden_write", false, true)).take(spec.forbidden_write_paths.len()),
    );
    expected.push(("provider_proxy_allowed", true, false));
    if spec.transport.engram_daemon_port.is_some() {
        expected.push(("engram_loopback_allowed", true, false));
    }
    expected.push(("network_denial", false, false));
    expected.push(("network_errno_denial", false, true));
    expected.push(("external_network_denial", false, true));
    expected.push(("unexpected_bind_denial", false, true));
    validate_exact_sanitized_probe_results(&audit.commands, &expected)?;
    validate_claude_seatbelt_probe(spec, &audit.profile)
}

fn validate_exact_sanitized_probe_results(
    observed: &[IsolationSandboxCommandAudit],
    expected: &[(&str, bool, bool)],
) -> EvalResult<()> {
    if observed.len() != expected.len() {
        return Err(invalid("sandbox probe receipt command set is not exact"));
    }
    for (audit, (label, succeeded, denial)) in observed.iter().zip(expected) {
        validate_sanitized_probe_result(audit, label, *succeeded, *denial)?;
    }
    let unique_argv = observed
        .iter()
        .map(|audit| audit.argv_sha256.as_str())
        .collect::<BTreeSet<_>>();
    if unique_argv.len() != observed.len() {
        return Err(invalid(
            "sandbox probe receipt reuses an argv digest across distinct commands",
        ));
    }
    Ok(())
}

fn validate_sanitized_probe_result(
    audit: &IsolationSandboxCommandAudit,
    expected_label: &str,
    expected_success: bool,
    require_denial: bool,
) -> EvalResult<()> {
    if audit.label != expected_label
        || audit.succeeded != expected_success
        || (audit.exit_code == 0) != audit.succeeded
        || (audit.succeeded && audit.sandbox_denial_evidence)
        || (require_denial && !audit.sandbox_denial_evidence)
        || !is_sha256(&audit.argv_sha256)
        || !is_sha256(&audit.stdout_sha256)
        || !is_sha256(&audit.stderr_sha256)
    {
        return Err(invalid(format!(
            "sandbox probe receipt drifted for {expected_label}"
        )));
    }
    Ok(())
}

/// Descriptor inventory immediately before spawning the provider process.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IsolationInheritedFd {
    pub fd: i32,
    pub purpose: String,
    pub target: String,
    pub device: u64,
    pub inode: u64,
    pub mode: u32,
    pub link_count: u64,
}

/// Runner-generated fstat evidence for the exact child stdio and ambient descriptor boundary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IsolationPreSpawnFdAudit {
    pub valid: bool,
    pub child_descriptors: Vec<IsolationInheritedFd>,
    pub ambient_inheritable_descriptors: Vec<IsolationInheritedFd>,
    pub audit_sha256: String,
}

/// Open child stdio handles paired with the immediately preceding runner-generated fstat audit.
#[derive(Debug)]
pub struct PreparedIsolationStdio {
    pub audit: IsolationPreSpawnFdAudit,
    evaluation_root: PathBuf,
    stdin: File,
    stdout: File,
    stderr: File,
}

impl PreparedIsolationStdio {
    /// Re-fstat the exact handles and enumerate ambient inheritable descriptors, then spawn.
    ///
    /// The stdio handles are moved into `command` only after the second audit, and `spawn` follows
    /// immediately in this method so a caller cannot interpose another pathname-based check.
    #[cfg(unix)]
    pub fn spawn(
        self,
        command: &mut Command,
    ) -> EvalResult<(std::process::Child, IsolationPreSpawnFdAudit)> {
        use std::os::unix::io::AsRawFd;

        let ambient = inspect_ambient_inheritable_fds()?;
        if ambient.iter().any(|descriptor| descriptor.fd > 2) {
            return Err(invalid(
                "runner has an unexpected inheritable descriptor immediately before spawn",
            ));
        }
        let child_descriptors = vec![
            fstat_descriptor(self.stdin.as_raw_fd(), "stdin_null", "/dev/null")?.with_child_fd(0),
            fstat_descriptor(
                self.stdout.as_raw_fd(),
                "stdout",
                &self.audit.child_descriptors[1].target,
            )?
            .with_child_fd(1),
            fstat_descriptor(
                self.stderr.as_raw_fd(),
                "stderr",
                &self.audit.child_descriptors[2].target,
            )?
            .with_child_fd(2),
        ];
        validate_generated_child_descriptors(&child_descriptors, &self.evaluation_root)?;
        if child_descriptors != self.audit.child_descriptors {
            return Err(invalid(
                "runner-generated child descriptors drifted before spawn",
            ));
        }
        let audit = IsolationPreSpawnFdAudit {
            valid: true,
            audit_sha256: sha256_serialized(&(&child_descriptors, &ambient))?,
            child_descriptors,
            ambient_inheritable_descriptors: ambient,
        };
        command
            .stdin(Stdio::from(self.stdin))
            .stdout(Stdio::from(self.stdout))
            .stderr(Stdio::from(self.stderr));
        let child = command.spawn()?;
        Ok((child, audit))
    }
}

/// Sanitized authentication-status command and result. Credential bytes are not fields here.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IsolationAuthStatusAttestation {
    pub host: IsolationHost,
    pub argv_sha256: String,
    pub environment_sha256: String,
    pub isolation_profile_sha256: String,
    pub exit_code: i32,
    pub account_ready: bool,
    pub subscription_login: bool,
    pub api_key_mode: bool,
    pub raw_output_retained: bool,
    pub stdout_sha256: String,
    pub stderr_sha256: String,
    pub output_bytes: u64,
}

/// Exact auth-status command that must run with the evaluation environment and wrapper.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IsolationAuthStatusContract {
    pub host: IsolationHost,
    pub argv: Vec<String>,
    pub environment: BTreeMap<String, String>,
    pub cwd: String,
    pub isolation_profile_sha256: String,
    pub max_output_bytes: u64,
    pub timeout_ms: u64,
}

/// Derive the only accepted auth-status command from the already-validated evaluation launch.
pub fn build_auth_status_contract(
    launch: &IsolationLaunchContract,
    boundary: &IsolationBoundaryContract,
) -> EvalResult<IsolationAuthStatusContract> {
    validate_boundary_shape(boundary)?;
    let (argv, profile) = derive_auth_status_command_paths(launch)?;
    let auth_paths = match launch.host {
        IsolationHost::Codex => vec![
            ("Codex auth-status executable", PathBuf::from(&argv[0])),
            ("Codex auth-status profile", profile.clone()),
        ],
        IsolationHost::ClaudeCode => vec![
            ("Claude auth-status wrapper", PathBuf::from(&argv[0])),
            ("Claude auth-status profile", PathBuf::from(&argv[2])),
            ("Claude auth-status executable", PathBuf::from(&argv[4])),
        ],
    };
    validate_pre_read_paths_disjoint_from_credential(boundary, &auth_paths)?;
    validate_isolation_launch(launch, boundary)?;
    let profile_sha256 = read_regular_file_digest(&profile)?;
    Ok(IsolationAuthStatusContract {
        host: launch.host,
        argv,
        environment: launch.environment.clone(),
        cwd: launch.cwd.clone(),
        isolation_profile_sha256: profile_sha256,
        max_output_bytes: 64 * 1024,
        timeout_ms: 5_000,
    })
}

fn derive_auth_status_command_paths(
    launch: &IsolationLaunchContract,
) -> EvalResult<(Vec<String>, PathBuf)> {
    match &launch.semantics {
        IsolationLaunchSemantics::Codex { .. } => {
            let executable = launch
                .argv
                .first()
                .ok_or_else(|| invalid("Codex evaluation argv is empty"))?;
            let codex_home = launch
                .environment
                .get("CODEX_HOME")
                .ok_or_else(|| invalid("Codex launch has no isolated CODEX_HOME"))?;
            Ok((
                vec![
                    executable.clone(),
                    "--profile".to_string(),
                    CODEX_CONFIG_LAYER.to_string(),
                    "login".to_string(),
                    "status".to_string(),
                ],
                Path::new(codex_home).join("isolation.config.toml"),
            ))
        }
        IsolationLaunchSemantics::ClaudeCode {
            seatbelt_profile, ..
        } => Ok((
            vec![
                "/usr/bin/sandbox-exec".to_string(),
                "-f".to_string(),
                seatbelt_profile.clone(),
                "--".to_string(),
                launch
                    .argv
                    .get(4)
                    .ok_or_else(|| invalid("Claude evaluation argv lacks executable"))?
                    .clone(),
                "auth".to_string(),
                "status".to_string(),
                "--json".to_string(),
            ],
            PathBuf::from(seatbelt_profile),
        )),
    }
}

/// Validate the launch without invoking a provider.
pub fn validate_isolation_launch(
    launch: &IsolationLaunchContract,
    boundary: &IsolationBoundaryContract,
) -> EvalResult<()> {
    if launch.schema_version != ISOLATION_SCHEMA_VERSION
        || launch.authorizes_provider_execution
        || !launch.env_clear
    {
        return Err(invalid(
            "provider-free schema-v1 launch must be non-authorizing and use env_clear",
        ));
    }
    let (evaluation_root, forbidden_literals) = validate_boundary_contract(boundary, launch)?;
    if launch.boundary_sha256 != sha256_serialized(boundary)? {
        return Err(invalid(
            "launch is not bound to the exact isolation boundary bytes",
        ));
    }
    let pre_read_paths = isolation_launch_pre_read_paths(launch)?;
    validate_pre_read_paths_disjoint_from_credential(boundary, &pre_read_paths)?;
    let cwd = canonical_directory(Path::new(&launch.cwd))?;
    if !cwd.starts_with(&evaluation_root) || cwd.display().to_string() != launch.cwd {
        return Err(invalid("evaluation cwd is outside the enclave"));
    }
    validate_environment(launch, &evaluation_root)?;
    validate_launch_text_surfaces(launch, &forbidden_literals, &boundary.forbidden_markers)?;
    validate_config_files(
        launch,
        &evaluation_root,
        &forbidden_literals,
        &boundary.forbidden_markers,
    )?;
    match launch.host {
        IsolationHost::Codex => validate_codex_argv(launch, boundary, &evaluation_root),
        IsolationHost::ClaudeCode => validate_claude_argv(launch, boundary, &evaluation_root),
    }
}

fn isolation_launch_pre_read_paths(
    launch: &IsolationLaunchContract,
) -> EvalResult<Vec<(&'static str, PathBuf)>> {
    let mut paths = vec![("evaluation cwd", PathBuf::from(&launch.cwd))];
    paths.extend(
        launch
            .config_files
            .iter()
            .map(|binding| ("bound launch file", PathBuf::from(&binding.path))),
    );
    match &launch.semantics {
        IsolationLaunchSemantics::Codex {
            output_schema, arm, ..
        } => {
            let executable = launch
                .argv
                .first()
                .ok_or_else(|| invalid("Codex launch argv has no executable"))?;
            paths.extend([
                ("Codex executable", PathBuf::from(executable)),
                ("Codex output schema", PathBuf::from(output_schema)),
            ]);
            let codex_home = launch
                .environment
                .get("CODEX_HOME")
                .ok_or_else(|| invalid("Codex launch has no isolated CODEX_HOME"))?;
            paths.push((
                "Codex config layer",
                Path::new(codex_home).join("isolation.config.toml"),
            ));
            if let Some(engram) = &arm.engram {
                paths.extend([
                    ("Codex Engram executable", PathBuf::from(&engram.executable)),
                    ("Codex Engram home", PathBuf::from(&engram.home)),
                    ("Codex Engram skill", PathBuf::from(&engram.skill_path)),
                ]);
            }
        }
        IsolationLaunchSemantics::ClaudeCode {
            output_schema,
            settings,
            mcp_config,
            seatbelt_profile,
            seatbelt_spec,
            auto_memory_directory,
            engram,
            engram_daemon,
            ..
        } => {
            let executable = launch
                .argv
                .get(4)
                .ok_or_else(|| invalid("Claude launch argv has no wrapped executable"))?;
            paths.extend([
                ("Claude executable", PathBuf::from(executable)),
                ("Claude output schema", PathBuf::from(output_schema)),
                ("Claude settings", PathBuf::from(settings)),
                ("Claude MCP config", PathBuf::from(mcp_config)),
                ("Claude Seatbelt profile", PathBuf::from(seatbelt_profile)),
                (
                    "Claude Seatbelt executable",
                    PathBuf::from(&seatbelt_spec.sandbox_exec),
                ),
            ]);
            if let Some(auto_memory) = auto_memory_directory {
                paths.push(("Claude auto-memory root", PathBuf::from(auto_memory)));
            }
            if let Some(engram) = engram {
                paths.extend([
                    (
                        "Claude Engram executable",
                        PathBuf::from(&engram.executable),
                    ),
                    ("Claude Engram home", PathBuf::from(&engram.home)),
                    ("Claude Engram skill", PathBuf::from(&engram.skill_path)),
                ]);
            }
            if let Some(daemon) = engram_daemon {
                paths.extend([
                    (
                        "Claude Engram daemon executable",
                        PathBuf::from(&daemon.process_executable_path),
                    ),
                    (
                        "Claude Engram daemon port receipt",
                        PathBuf::from(&daemon.daemon_port_file.path),
                    ),
                    (
                        "Claude Engram daemon PID receipt",
                        PathBuf::from(&daemon.daemon_pid_file.path),
                    ),
                    (
                        "Claude Engram daemon metadata receipt",
                        PathBuf::from(&daemon.daemon_metadata_file.path),
                    ),
                ]);
            }
        }
    }
    Ok(paths)
}

fn validate_boundary_contract(
    boundary: &IsolationBoundaryContract,
    launch: &IsolationLaunchContract,
) -> EvalResult<(PathBuf, Vec<String>)> {
    if boundary.schema_version != ISOLATION_SCHEMA_VERSION
        || boundary.host != launch.host
        || boundary.native_memory_enabled != launch.native_memory_enabled
        || boundary.uses_engram != launch.uses_engram
    {
        return Err(invalid(
            "isolation boundary identity does not match the launch lane",
        ));
    }
    validate_boundary_shape(boundary)
}

fn validate_boundary_shape(
    boundary: &IsolationBoundaryContract,
) -> EvalResult<(PathBuf, Vec<String>)> {
    if boundary.forbidden_markers.is_empty()
        || boundary
            .forbidden_markers
            .iter()
            .any(|marker| marker.len() < 8)
    {
        return Err(invalid(
            "isolation boundary requires nontrivial teaching markers",
        ));
    }
    require_unique_strings(boundary.forbidden_markers.iter(), "boundary marker")?;

    let source_root = canonical_directory(Path::new(&boundary.source_root))?;
    let evaluation_root = canonical_directory(Path::new(&boundary.evaluation_root))?;
    if source_root.display().to_string() != boundary.source_root
        || evaluation_root.display().to_string() != boundary.evaluation_root
    {
        return Err(invalid("isolation boundary roots must be canonical paths"));
    }
    require_disjoint_roots(&source_root, &evaluation_root)?;
    require_private_directory(&evaluation_root, "evaluation enclave")?;

    let mut required_classes = BTreeSet::from([
        IsolationBoundaryPathClass::RunArtifacts,
        IsolationBoundaryPathClass::TeachingArtifacts,
        IsolationBoundaryPathClass::CredentialMaterial,
    ]);
    if boundary.native_memory_enabled {
        required_classes.insert(IsolationBoundaryPathClass::NativeState);
    }
    if boundary.uses_engram {
        required_classes.insert(IsolationBoundaryPathClass::EngramState);
    }
    let actual_classes = boundary
        .forbidden_paths
        .iter()
        .map(|entry| entry.class)
        .collect::<BTreeSet<_>>();
    if actual_classes != required_classes {
        return Err(invalid(
            "boundary path classes do not match the lane's required protections",
        ));
    }

    let mut seen = BTreeSet::new();
    let mut paths = Vec::with_capacity(boundary.forbidden_paths.len());
    for entry in &boundary.forbidden_paths {
        let path = Path::new(&entry.path);
        require_normalized_absolute_path(path, "boundary path")?;
        if !seen.insert(entry.path.as_str()) {
            return Err(invalid("isolation boundary contains duplicate paths"));
        }
        let expected_root = match entry.class {
            IsolationBoundaryPathClass::RunArtifacts
            | IsolationBoundaryPathClass::TeachingArtifacts => &source_root,
            IsolationBoundaryPathClass::CredentialMaterial
            | IsolationBoundaryPathClass::NativeState
            | IsolationBoundaryPathClass::EngramState => &evaluation_root,
        };
        if path == expected_root || !path.starts_with(expected_root) {
            return Err(invalid(
                "boundary path is outside its required source/evaluation root",
            ));
        }
        let metadata = fs::symlink_metadata(path)?;
        if metadata.file_type().is_symlink()
            || (!metadata.is_dir() && !metadata.is_file())
            || path.canonicalize()?.display().to_string() != entry.path
        {
            return Err(invalid(
                "boundary paths must be existing canonical files or directories",
            ));
        }
        if matches!(
            entry.class,
            IsolationBoundaryPathClass::CredentialMaterial
                | IsolationBoundaryPathClass::NativeState
                | IsolationBoundaryPathClass::EngramState
        ) {
            require_owner_private_path(path, "protected evaluation path")?;
        }
        paths.push(path.to_path_buf());
    }
    require_nonoverlapping_absolute_paths(&paths, "boundary paths")?;
    validate_boundary_state_semantics(boundary, &evaluation_root)?;

    let mut forbidden_literals = vec![boundary.source_root.clone()];
    forbidden_literals.extend(
        boundary
            .forbidden_paths
            .iter()
            .map(|entry| entry.path.clone()),
    );
    Ok((evaluation_root, forbidden_literals))
}

fn validate_boundary_state_semantics(
    boundary: &IsolationBoundaryContract,
    evaluation_root: &Path,
) -> EvalResult<()> {
    let credential = boundary
        .forbidden_paths
        .iter()
        .find(|path| path.class == IsolationBoundaryPathClass::CredentialMaterial)
        .ok_or_else(|| invalid("lane boundary lacks its exact credential surface"))?;
    let expected_credential = match boundary.host {
        IsolationHost::Codex => evaluation_root.join("codex-home/auth.json"),
        IsolationHost::ClaudeCode => evaluation_root.join("claude-config/.credentials.json"),
    };
    let credential_metadata = fs::symlink_metadata(&expected_credential)?;
    if Path::new(&credential.path) != expected_credential || !credential_metadata.is_file() {
        return Err(invalid(
            "credential boundary is not the host's exact isolated auth-cache file",
        ));
    }
    require_single_link(&credential_metadata, &expected_credential)?;

    let native = boundary
        .forbidden_paths
        .iter()
        .find(|path| path.class == IsolationBoundaryPathClass::NativeState);
    let expected_native = match boundary.host {
        IsolationHost::Codex => evaluation_root.join("codex-home/memories"),
        IsolationHost::ClaudeCode => evaluation_root.join("claude-config/auto-memory"),
    };
    match (boundary.native_memory_enabled, native) {
        (true, Some(path))
            if Path::new(&path.path) == expected_native
                && fs::symlink_metadata(&expected_native)?.is_dir() => {}
        (false, None) if path_entry_absent(&expected_native)? => {}
        _ => {
            return Err(invalid(
                "native-memory boundary or on-disk state does not match the exact host arm",
            ))
        }
    }

    let engram = boundary
        .forbidden_paths
        .iter()
        .find(|path| path.class == IsolationBoundaryPathClass::EngramState);
    let projects_root = evaluation_root.join("engram-home/projects");
    match (boundary.uses_engram, engram) {
        (true, Some(path)) => {
            let path = Path::new(&path.path);
            if path.parent() != Some(projects_root.as_path())
                || path.file_name().and_then(|name| name.to_str()).is_none()
                || !fs::symlink_metadata(path)?.is_dir()
            {
                return Err(invalid(
                    "Engram boundary is not one exact isolated project-state root",
                ));
            }
        }
        (false, None) if path_entry_absent(&evaluation_root.join("engram-home"))? => {}
        _ => {
            return Err(invalid(
                "Engram boundary or on-disk state does not match the exact lane arm",
            ))
        }
    }

    let unrelated_home = match boundary.host {
        IsolationHost::Codex => evaluation_root.join("claude-config"),
        IsolationHost::ClaudeCode => evaluation_root.join("codex-home"),
    };
    if !path_entry_absent(&unrelated_home)? {
        return Err(invalid(
            "evaluation enclave contains stale state for the other provider host",
        ));
    }
    Ok(())
}

fn path_entry_absent(path: &Path) -> EvalResult<bool> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(true),
        Ok(_) => Ok(false),
        Err(error) => Err(EvalError::Io(error)),
    }
}

/// Reject every later-opened launch path that could expose the exact credential file.
///
/// This guard runs immediately after the metadata-only boundary check and before any provider,
/// schema, profile, settings, skill, or daemon receipt is opened, hashed, or executed. Lexical
/// equality/ancestry is rejected before filesystem inspection; canonical paths and Unix object
/// identities then close symlink, case-alias, bind-alias, and hard-link equivalents without
/// reading file contents.
fn validate_pre_read_paths_disjoint_from_credential(
    boundary: &IsolationBoundaryContract,
    paths: &[(&str, PathBuf)],
) -> EvalResult<()> {
    let credential = boundary
        .forbidden_paths
        .iter()
        .find(|path| path.class == IsolationBoundaryPathClass::CredentialMaterial)
        .ok_or_else(|| invalid("lane boundary lacks its exact credential surface"))?;
    let credential_path = Path::new(&credential.path);
    require_normalized_absolute_path(credential_path, "credential boundary")?;

    for (label, candidate) in paths {
        require_normalized_absolute_path(candidate, label)?;
        if candidate == credential_path
            || candidate.starts_with(credential_path)
            || credential_path.starts_with(candidate)
        {
            return Err(invalid(format!(
                "{label} overlaps exact credential material before content access"
            )));
        }
    }

    let canonical_credential = credential_path.canonicalize()?;
    let mut credential_ancestors = Vec::new();
    for ancestor in canonical_credential.ancestors() {
        credential_ancestors.push(fs::metadata(ancestor)?);
    }
    for (label, candidate) in paths {
        let canonical_candidate = candidate.canonicalize()?;
        if canonical_candidate == canonical_credential
            || canonical_candidate.starts_with(&canonical_credential)
            || canonical_credential.starts_with(&canonical_candidate)
        {
            return Err(invalid(format!(
                "{label} canonically aliases credential material before content access"
            )));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;

            let candidate_metadata = fs::metadata(candidate)?;
            if credential_ancestors.iter().any(|credential_metadata| {
                candidate_metadata.dev() == credential_metadata.dev()
                    && candidate_metadata.ino() == credential_metadata.ino()
            }) {
                return Err(invalid(format!(
                    "{label} aliases credential material before content access"
                )));
            }
        }
    }
    Ok(())
}

/// Require the Claude raw Seatbelt profile to match every applicable boundary path exactly.
pub fn claude_seatbelt_spec_for_boundary(
    boundary: &IsolationBoundaryContract,
    transport: &ClaudeNarrowTransportContract,
) -> EvalResult<ClaudeSeatbeltSpec> {
    if boundary.host != IsolationHost::ClaudeCode {
        return Err(invalid("Claude Seatbelt cannot be bound to a Codex lane"));
    }
    validate_boundary_shape(boundary)?;
    validate_claude_transport(transport)?;
    if boundary.uses_engram != transport.engram_daemon_port.is_some() {
        return Err(invalid(
            "Claude Engram arm does not match its exact loopback daemon transport",
        ));
    }
    let mut forbidden_read_paths = BTreeSet::from([boundary.source_root.clone()]);
    let mut forbidden_write_paths = BTreeSet::from([boundary.source_root.clone()]);
    for entry in &boundary.forbidden_paths {
        if entry.class != IsolationBoundaryPathClass::CredentialMaterial {
            forbidden_write_paths.insert(entry.path.clone());
        }
        match entry.class {
            IsolationBoundaryPathClass::RunArtifacts
            | IsolationBoundaryPathClass::TeachingArtifacts => {
                forbidden_read_paths.insert(entry.path.clone());
            }
            IsolationBoundaryPathClass::CredentialMaterial
            | IsolationBoundaryPathClass::NativeState
            | IsolationBoundaryPathClass::EngramState => {}
        }
    }
    Ok(ClaudeSeatbeltSpec {
        schema_version: ISOLATION_SCHEMA_VERSION,
        sandbox_exec: "/usr/bin/sandbox-exec".to_string(),
        transport: transport.clone(),
        forbidden_read_paths: forbidden_read_paths.into_iter().collect(),
        forbidden_write_paths: forbidden_write_paths.into_iter().collect(),
    })
}

/// Require the Claude raw Seatbelt profile to match every applicable boundary path exactly.
pub fn validate_claude_seatbelt_boundary(
    spec: &ClaudeSeatbeltSpec,
    boundary: &IsolationBoundaryContract,
) -> EvalResult<()> {
    let expected = claude_seatbelt_spec_for_boundary(boundary, &spec.transport)?;
    if spec
        .forbidden_read_paths
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>()
        != expected
            .forbidden_read_paths
            .into_iter()
            .collect::<BTreeSet<_>>()
        || spec.transport != expected.transport
        || spec
            .forbidden_write_paths
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            != expected
                .forbidden_write_paths
                .into_iter()
                .collect::<BTreeSet<_>>()
    {
        return Err(invalid(
            "Claude Seatbelt paths do not exactly match the frozen isolation boundary",
        ));
    }
    validate_seatbelt_spec(spec)
}

pub fn validate_auth_status_attestation(
    contract: &IsolationAuthStatusContract,
    attestation: &IsolationAuthStatusAttestation,
    evaluation_launch: &IsolationLaunchContract,
    boundary: &IsolationBoundaryContract,
) -> EvalResult<()> {
    let expected_contract = build_auth_status_contract(evaluation_launch, boundary)?;
    if contract != &expected_contract {
        return Err(invalid(
            "auth-status contract is not the internally derived exact command; caller paths were not accessed",
        ));
    }
    if expected_contract.host != attestation.host
        || expected_contract.host != evaluation_launch.host
        || sha256_serialized(&contract.argv)? != attestation.argv_sha256
        || sha256_serialized(&contract.environment)? != attestation.environment_sha256
        || contract.isolation_profile_sha256 != attestation.isolation_profile_sha256
        || !is_sha256(&contract.isolation_profile_sha256)
        || contract.max_output_bytes != 64 * 1024
        || contract.timeout_ms != 5_000
        || attestation.exit_code != 0
        || !attestation.account_ready
        || !attestation.subscription_login
        || attestation.api_key_mode
        || attestation.raw_output_retained
        || !is_sha256(&attestation.stdout_sha256)
        || !is_sha256(&attestation.stderr_sha256)
        || attestation.output_bytes > contract.max_output_bytes
    {
        return Err(invalid(
            "authentication status does not match the exact evaluation wrapper/environment",
        ));
    }
    if contract
        .environment
        .keys()
        .any(|key| is_api_key_environment(key))
    {
        return Err(invalid(
            "authentication status environment contains an API-key variable",
        ));
    }
    Ok(())
}

/// Validate the later runner-parsed Claude init/terminal trace against its exact launch contract.
///
/// A single provider call may cross the configured monetary boundary before Claude stops. Such an
/// overshoot is accepted only when the terminal envelope explicitly reports the budget boundary.
pub fn validate_claude_trace_attestation(
    launch: &IsolationLaunchContract,
    boundary: &IsolationBoundaryContract,
    attestation: &ClaudeIsolationTraceAttestation,
) -> EvalResult<()> {
    validate_isolation_launch(launch, boundary)?;
    let IsolationLaunchSemantics::ClaudeCode { limits, engram, .. } = &launch.semantics else {
        return Err(invalid(
            "Claude trace attestation was paired with a Codex launch",
        ));
    };
    let expected_mcp = if engram.is_some() {
        vec!["engram".to_string()]
    } else {
        Vec::new()
    };
    let expected_mcp_statuses = if engram.is_some() {
        BTreeMap::from([("engram".to_string(), "connected".to_string())])
    } else {
        BTreeMap::new()
    };
    let expected_agents = CLAUDE_BUILT_IN_AGENTS
        .iter()
        .map(|agent| (*agent).to_string())
        .collect::<Vec<_>>();
    let expected_capabilities = CLAUDE_CAPABILITIES
        .iter()
        .map(|capability| (*capability).to_string())
        .collect::<Vec<_>>();
    let auth_surface_consistent = match attestation.init_api_key_source.as_str() {
        "apiKeyHelper" => {
            attestation.managed_api_key_helper_attempted
                && !attestation.local_subscription_pure
                && attestation.confounded_exploratory_only
        }
        "claude.ai" => {
            !attestation.managed_api_key_helper_attempted
                && !attestation.managed_api_key_helper_succeeded
                && attestation.local_subscription_pure
                && attestation.confounded_exploratory_only
        }
        _ => false,
    };
    let budget_subtype = attestation.terminal_subtype == "error_max_budget_usd";
    if !is_sha256(&attestation.trace_sha256)
        || !attestation.init_present
        || !attestation.terminal_present
        || attestation.init_claude_code_version != CLAUDE_TRACE_CLI_VERSION
        || attestation.init_cwd != launch.cwd
        || attestation.init_output_style != "default"
        || attestation.init_model != limits.model
        || attestation.init_mcp_servers != expected_mcp
        || attestation.init_mcp_server_statuses != expected_mcp_statuses
        || attestation.init_tools != launch.expected_tools
        || attestation.init_permission_mode != CLAUDE_PERMISSION_MODE
        || !attestation.init_plugins.is_empty()
        || attestation.init_subagents != expected_agents
        || !attestation.init_skills.is_empty()
        || !attestation.init_slash_commands.is_empty()
        || !attestation.init_terminal_slash_commands.is_empty()
        || attestation.init_capabilities != expected_capabilities
        || !attestation.init_analytics_disabled
        || attestation.init_product_feedback_disabled
        || !auth_surface_consistent
        || (attestation.managed_api_key_helper_succeeded
            && !attestation.managed_api_key_helper_attempted)
        || attestation.configured_max_turns != limits.max_turns
        || attestation.configured_max_budget_micro_usd != limits.max_budget_micro_usd
        || attestation.observed_turns == 0
        || attestation.observed_turns > limits.max_turns
        || !matches!(
            attestation.terminal_subtype.as_str(),
            "success" | "error_max_budget_usd"
        )
        || (attestation.terminal_budget_exhausted != budget_subtype)
        || (attestation.reported_cost_micro_usd > limits.max_budget_micro_usd
            && !attestation.terminal_budget_exhausted)
    {
        return Err(invalid(
            "Claude init/terminal trace does not match its model, tool, MCP, permission, turn, or cost contract",
        ));
    }
    Ok(())
}

/// Execute and parse a bounded auth-status command; retain only status and content digests.
pub fn execute_auth_status_contract(
    evaluation_launch: &IsolationLaunchContract,
    boundary: &IsolationBoundaryContract,
) -> EvalResult<IsolationAuthStatusAttestation> {
    let contract = build_auth_status_contract(evaluation_launch, boundary)?;
    let executable = contract
        .argv
        .first()
        .ok_or_else(|| invalid("auth-status argv is empty"))?;
    let mut child = Command::new(executable)
        .args(&contract.argv[1..])
        .current_dir(&contract.cwd)
        .env_clear()
        .envs(&contract.environment)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| invalid("auth-status stdout pipe is absent"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| invalid("auth-status stderr pipe is absent"))?;
    let limit = contract.max_output_bytes;
    let stdout_reader = std::thread::spawn(move || read_bounded_output(stdout, limit));
    let stderr_reader = std::thread::spawn(move || read_bounded_output(stderr, limit));
    let deadline =
        std::time::Instant::now() + std::time::Duration::from_millis(contract.timeout_ms);
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if std::time::Instant::now() >= deadline {
            child.kill()?;
            let _ = child.wait();
            return Err(invalid(
                "auth-status command exceeded its five-second bound",
            ));
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    };
    let stdout = stdout_reader
        .join()
        .map_err(|_| invalid("auth-status stdout reader panicked"))??;
    let stderr = stderr_reader
        .join()
        .map_err(|_| invalid("auth-status stderr reader panicked"))??;
    let output_bytes = u64::try_from(stdout.len() + stderr.len())
        .map_err(|_| invalid("auth-status output byte count overflow"))?;
    if output_bytes > contract.max_output_bytes {
        return Err(invalid("auth-status output exceeded its exact byte bound"));
    }
    let (account_ready, subscription_login, api_key_mode) =
        parse_auth_status(contract.host, status.success(), &stdout, &stderr)?;
    let attestation = IsolationAuthStatusAttestation {
        host: contract.host,
        argv_sha256: sha256_serialized(&contract.argv)?,
        environment_sha256: sha256_serialized(&contract.environment)?,
        isolation_profile_sha256: contract.isolation_profile_sha256.clone(),
        exit_code: status
            .code()
            .ok_or_else(|| invalid("auth-status command ended without an exit code"))?,
        account_ready,
        subscription_login,
        api_key_mode,
        raw_output_retained: false,
        stdout_sha256: sha256_bytes(&stdout),
        stderr_sha256: sha256_bytes(&stderr),
        output_bytes,
    };
    validate_auth_status_attestation(&contract, &attestation, evaluation_launch, boundary)?;
    Ok(attestation)
}

fn read_bounded_output(reader: impl Read, limit: u64) -> EvalResult<Vec<u8>> {
    let mut bytes = Vec::new();
    reader.take(limit + 1).read_to_end(&mut bytes)?;
    if u64::try_from(bytes.len()).map_or(true, |length| length > limit) {
        return Err(invalid("auth-status stream exceeded its byte bound"));
    }
    Ok(bytes)
}

fn parse_auth_status(
    host: IsolationHost,
    success: bool,
    stdout: &[u8],
    stderr: &[u8],
) -> EvalResult<(bool, bool, bool)> {
    match host {
        IsolationHost::Codex => {
            let stdout = std::str::from_utf8(stdout).unwrap_or_default().trim();
            let stderr = std::str::from_utf8(stderr).unwrap_or_default().trim();
            let subscription = success
                && [stdout, stderr]
                    .into_iter()
                    .any(|value| value == "Logged in using ChatGPT");
            let api_key = [stdout, stderr]
                .into_iter()
                .any(|value| value == "Logged in using an API key");
            Ok((subscription, subscription, api_key))
        }
        IsolationHost::ClaudeCode => {
            let value: serde_json::Value = serde_json::from_slice(stdout)
                .map_err(|_| invalid("Claude auth status did not return bounded JSON"))?;
            let object = value
                .as_object()
                .ok_or_else(|| invalid("Claude auth status JSON is not an object"))?;
            let allowed = BTreeSet::from(["loggedIn", "authMethod", "apiProvider"]);
            if object.keys().map(String::as_str).collect::<BTreeSet<_>>() != allowed {
                return Err(invalid("Claude auth status JSON shape drifted"));
            }
            let logged_in =
                object.get("loggedIn").and_then(serde_json::Value::as_bool) == Some(true);
            let method = object
                .get("authMethod")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            let provider = object
                .get("apiProvider")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            let subscription =
                success && logged_in && method == "claude.ai" && provider == "firstParty";
            let api_key = method.to_ascii_lowercase().contains("api")
                || provider.to_ascii_lowercase().contains("api");
            Ok((subscription, subscription, api_key))
        }
    }
}

fn validate_environment(
    launch: &IsolationLaunchContract,
    evaluation_root: &Path,
) -> EvalResult<()> {
    let mut allowed = BTreeSet::from([
        "HOME",
        "TMPDIR",
        "PATH",
        "LANG",
        "LC_ALL",
        "NO_COLOR",
        "DO_NOT_TRACK",
        "OTEL_SDK_DISABLED",
    ]);
    match launch.host {
        IsolationHost::Codex => {
            allowed.insert("CODEX_HOME");
        }
        IsolationHost::ClaudeCode => {
            allowed.insert("CLAUDE_CONFIG_DIR");
            allowed.insert("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC");
            allowed.insert("CLAUDE_CODE_DISABLE_AUTO_MEMORY");
            allowed.insert("CLAUDE_CODE_PROXY_RESOLVES_HOSTS");
            allowed.insert("DISABLE_TELEMETRY");
            allowed.insert("DISABLE_UPDATES");
            allowed.insert("ENABLE_CLAUDEAI_MCP_SERVERS");
            allowed.insert("HTTPS_PROXY");
        }
    }
    if launch.uses_engram {
        allowed.insert("ENGRAM_HOME");
    }
    let actual = launch
        .environment
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if actual != allowed {
        return Err(invalid(
            "evaluation environment does not match the exact cleared allowlist",
        ));
    }
    let host_path_key = match launch.host {
        IsolationHost::Codex => "CODEX_HOME",
        IsolationHost::ClaudeCode => "CLAUDE_CONFIG_DIR",
    };
    let required = [
        "HOME",
        "TMPDIR",
        "PATH",
        "LANG",
        "LC_ALL",
        "NO_COLOR",
        "DO_NOT_TRACK",
        "OTEL_SDK_DISABLED",
        host_path_key,
    ];
    if required
        .iter()
        .any(|key| !launch.environment.contains_key(*key))
    {
        return Err(invalid(
            "evaluation environment is missing a required isolated path",
        ));
    }
    if launch.uses_engram != launch.environment.contains_key("ENGRAM_HOME") {
        return Err(invalid("ENGRAM_HOME does not match the frozen lane arm"));
    }
    for (key, relative) in [
        ("HOME", "home"),
        ("TMPDIR", "tmp"),
        (
            host_path_key,
            match launch.host {
                IsolationHost::Codex => "codex-home",
                IsolationHost::ClaudeCode => "claude-config",
            },
        ),
    ] {
        let path = canonical_directory(Path::new(&launch.environment[key]))?;
        require_private_directory(&path, key)?;
        if path != evaluation_root.join(relative)
            || path.display().to_string() != launch.environment[key]
        {
            return Err(invalid(format!(
                "{key} is not the exact canonical enclave directory"
            )));
        }
    }
    if let Some(path) = launch.environment.get("ENGRAM_HOME") {
        let canonical = canonical_directory(Path::new(path))?;
        require_private_directory(&canonical, "ENGRAM_HOME")?;
        if canonical != evaluation_root.join("engram-home")
            || canonical.display().to_string() != *path
        {
            return Err(invalid(
                "ENGRAM_HOME is not the exact canonical enclave directory",
            ));
        }
    }
    if launch
        .environment
        .keys()
        .any(|key| is_api_key_environment(key))
    {
        return Err(invalid(
            "evaluation environment contains an API-key variable",
        ));
    }
    if launch.environment.get("PATH").map(String::as_str) != Some("/usr/bin:/bin")
        || launch.environment.get("LANG").map(String::as_str) != Some("C.UTF-8")
        || launch.environment.get("LC_ALL").map(String::as_str) != Some("C.UTF-8")
        || launch.environment.get("NO_COLOR").map(String::as_str) != Some("1")
        || launch.environment.get("DO_NOT_TRACK").map(String::as_str) != Some("1")
        || launch
            .environment
            .get("OTEL_SDK_DISABLED")
            .map(String::as_str)
            != Some("true")
    {
        return Err(invalid(
            "locale and telemetry-off environment values are not frozen",
        ));
    }
    if launch.host == IsolationHost::ClaudeCode {
        let IsolationLaunchSemantics::ClaudeCode { seatbelt_spec, .. } = &launch.semantics else {
            return Err(invalid("Claude launch lacks its typed Seatbelt semantics"));
        };
        validate_claude_transport_environment(
            &launch.environment,
            &seatbelt_spec.transport,
            launch.native_memory_enabled,
        )?;
    }
    Ok(())
}

fn validate_claude_transport_environment(
    environment: &BTreeMap<String, String>,
    transport: &ClaudeNarrowTransportContract,
    native_memory_enabled: bool,
) -> EvalResult<()> {
    validate_claude_transport(transport)?;
    let expected_proxy = format!(
        "http://engram-proxy:{}@{}:{}",
        CLAUDE_REDACTED_PROXY_PASSWORD,
        transport.provider_proxy_host,
        transport.provider_proxy_port
    );
    let expected_auto_memory = if native_memory_enabled { "0" } else { "1" };
    if environment
        .get("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC")
        .map(String::as_str)
        != Some("1")
        || environment
            .get("CLAUDE_CODE_DISABLE_AUTO_MEMORY")
            .map(String::as_str)
            != Some(expected_auto_memory)
        || environment
            .get("CLAUDE_CODE_PROXY_RESOLVES_HOSTS")
            .map(String::as_str)
            != Some("1")
        || environment.get("DISABLE_TELEMETRY").map(String::as_str) != Some("1")
        || environment.get("DISABLE_UPDATES").map(String::as_str) != Some("1")
        || environment
            .get("ENABLE_CLAUDEAI_MCP_SERVERS")
            .map(String::as_str)
            != Some("false")
        || environment.get("HTTPS_PROXY") != Some(&expected_proxy)
        || environment.contains_key("HTTP_PROXY")
        || environment.contains_key("ALL_PROXY")
        || environment.contains_key("NO_PROXY")
        || environment.contains_key("https_proxy")
        || environment.contains_key("http_proxy")
        || environment.contains_key("all_proxy")
        || environment.contains_key("no_proxy")
    {
        return Err(invalid(
            "Claude traffic, update, connector, auto-memory, and exact proxy environment drifted",
        ));
    }
    Ok(())
}

fn validate_launch_text_surfaces(
    launch: &IsolationLaunchContract,
    forbidden_literals: &[String],
    forbidden_markers: &[String],
) -> EvalResult<()> {
    let environment = serde_json::to_string(&launch.environment)?;
    let argv = serde_json::to_string(&launch.argv)?;
    for (label, surface) in [
        ("argv", argv.as_str()),
        ("environment", environment.as_str()),
        ("prompt", launch.prompt.as_str()),
    ] {
        for forbidden in forbidden_literals.iter().chain(forbidden_markers.iter()) {
            if !forbidden.is_empty() && surface.contains(forbidden) {
                return Err(invalid(format!(
                    "evaluation {label} contains a source/run/marker literal"
                )));
            }
        }
    }
    Ok(())
}

fn validate_config_files(
    launch: &IsolationLaunchContract,
    evaluation_root: &Path,
    forbidden_literals: &[String],
    forbidden_markers: &[String],
) -> EvalResult<()> {
    let mut seen = BTreeSet::new();
    if launch.config_files.is_empty() {
        return Err(invalid("launch has no hash-bound configuration files"));
    }
    for config in &launch.config_files {
        let path = Path::new(&config.path);
        if !path.is_absolute() || !is_sha256(&config.sha256) || !seen.insert(&config.path) {
            return Err(invalid("config scan paths must be unique and absolute"));
        }
        let canonical = path.canonicalize()?;
        if !canonical.starts_with(evaluation_root) || canonical.display().to_string() != config.path
        {
            return Err(invalid("config scan path is outside the enclave"));
        }
        let relative = canonical
            .strip_prefix(evaluation_root)
            .map_err(|_| invalid("config scan path escaped the enclave"))?;
        reject_credential_semantic_path(relative, IsolationCopyClass::Settings)?;
        let metadata = fs::symlink_metadata(&canonical)?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(invalid(
                "config scan surface must be a regular non-symlink file",
            ));
        }
        require_single_link(&metadata, &canonical)?;
        if metadata.len() > MAX_MANIFEST_FILE_BYTES {
            return Err(invalid(
                "config scan surface exceeds the bounded file limit",
            ));
        }
        #[cfg(unix)]
        use std::os::unix::fs::PermissionsExt;
        #[cfg(unix)]
        let mode = metadata.permissions().mode() & 0o777;
        #[cfg(not(unix))]
        let mode = 0;
        if mode != 0o600 {
            return Err(invalid(
                "config scan surfaces must be owner-only mode-0600 files",
            ));
        }
        let bytes =
            read_exact_regular_file(&canonical, metadata.len(), mode, MAX_MANIFEST_FILE_BYTES)?;
        if sha256_bytes(&bytes) != config.sha256 {
            return Err(invalid("config scan surface digest drifted"));
        }
        let is_boundary_bound_seatbelt = matches!(
            &launch.semantics,
            IsolationLaunchSemantics::ClaudeCode {
                seatbelt_profile, ..
            } if seatbelt_profile == &config.path
        );
        let allowed_native_settings_literal = match &launch.semantics {
            IsolationLaunchSemantics::ClaudeCode {
                settings,
                auto_memory_directory,
                ..
            } if settings == &config.path => auto_memory_directory.as_deref(),
            _ => None,
        };
        for forbidden in forbidden_literals {
            if is_boundary_bound_seatbelt
                || allowed_native_settings_literal == Some(forbidden.as_str())
            {
                continue;
            }
            if !forbidden.is_empty() && contains_bytes(&bytes, forbidden.as_bytes()) {
                return Err(invalid(format!(
                    "config contains a source/run/marker literal: {} contains {}",
                    config.path, forbidden
                )));
            }
        }
        for marker in forbidden_markers {
            if !marker.is_empty() && contains_bytes(&bytes, marker.as_bytes()) {
                return Err(invalid(format!(
                    "config contains a teaching marker: {}",
                    config.path
                )));
            }
        }
    }
    Ok(())
}

fn validate_codex_argv(
    launch: &IsolationLaunchContract,
    boundary: &IsolationBoundaryContract,
    evaluation_root: &Path,
) -> EvalResult<()> {
    validate_positional_prompt(&launch.prompt)?;
    let IsolationLaunchSemantics::Codex {
        output_schema,
        permission_profile,
        arm,
        permission_probe,
    } = &launch.semantics
    else {
        return Err(invalid("Codex launch carries Claude semantics"));
    };
    if launch.provider_cli_version != CODEX_CLI_VERSION
        || arm.native_memory_enabled != launch.native_memory_enabled
        || arm.engram.is_some() != launch.uses_engram
        || permission_profile.config_layer_name != CODEX_CONFIG_LAYER
        || permission_profile.permission_profile_name != CODEX_PERMISSION_PROFILE
        || permission_profile.evaluation_checkout != launch.cwd
        || !permission_probe.valid
    {
        return Err(invalid("Codex typed launch semantics drifted"));
    }
    let executable = launch
        .argv
        .first()
        .ok_or_else(|| invalid("Codex argv is empty"))?;
    validate_provider_executable(
        Path::new(executable),
        &launch.provider_executable_sha256,
        &launch.provider_cli_version,
        &launch.environment,
        "Codex",
    )?;
    let expected_argv = vec![
        executable.clone(),
        "exec".to_string(),
        "--profile".to_string(),
        CODEX_CONFIG_LAYER.to_string(),
        "--ignore-user-config".to_string(),
        "--ignore-rules".to_string(),
        "--strict-config".to_string(),
        "--ephemeral".to_string(),
        "--cd".to_string(),
        launch.cwd.clone(),
        "--json".to_string(),
        "--output-schema".to_string(),
        output_schema.clone(),
        launch.prompt.clone(),
    ];
    if launch.argv != expected_argv {
        return Err(invalid(
            "Codex argv differs byte-for-byte from the typed 0.153.3 contract",
        ));
    }
    validate_codex_permission_profile(
        permission_profile,
        evaluation_root,
        &permission_probe.profile,
    )?;
    validate_codex_probe_audit(permission_probe)?;
    validate_engram_arm(
        arm.engram.as_ref(),
        boundary,
        &launch.environment,
        evaluation_root,
    )?;
    validate_codex_native_memory_arm(launch, boundary)?;
    let profile_path = Path::new(&launch.environment["CODEX_HOME"]).join("isolation.config.toml");
    let expected_profile = generate_codex_evaluation_config_toml(permission_profile, arm)?;
    if permission_probe.profile_sha256 != sha256_bytes(expected_profile.as_bytes()) {
        return Err(invalid("Codex config/probe receipt digest binding drifted"));
    }
    require_exact_file_bytes(&profile_path, expected_profile.as_bytes(), true)?;
    let mut expected_files = vec![
        bound_file(Path::new(output_schema), true)?,
        bound_file(&profile_path, true)?,
    ];
    if let Some(engram) = &arm.engram {
        expected_files.push(bound_file(Path::new(&engram.skill_path), true)?);
    }
    if launch.config_files != expected_files
        || launch.expected_tools != expected_tool_surface(IsolationHost::Codex, launch.uses_engram)
    {
        return Err(invalid(
            "Codex configuration/tool surface is not the exact typed lane contract",
        ));
    }
    Ok(())
}

fn validate_claude_argv(
    launch: &IsolationLaunchContract,
    boundary: &IsolationBoundaryContract,
    evaluation_root: &Path,
) -> EvalResult<()> {
    validate_positional_prompt(&launch.prompt)?;
    let IsolationLaunchSemantics::ClaudeCode {
        output_schema,
        settings,
        mcp_config,
        seatbelt_profile,
        seatbelt_spec,
        seatbelt_probe,
        auto_memory_directory,
        engram,
        engram_daemon,
        limits,
        managed_settings_outside_exact_environment_claim,
    } = &launch.semantics
    else {
        return Err(invalid("Claude launch carries Codex semantics"));
    };
    if launch.provider_cli_version != CLAUDE_CLI_VERSION
        || launch.native_memory_enabled != auto_memory_directory.is_some()
        || launch.uses_engram != engram.is_some()
        || launch.uses_engram != engram_daemon.is_some()
        || !seatbelt_probe.valid
        || !managed_settings_outside_exact_environment_claim
    {
        return Err(invalid("Claude typed launch semantics drifted"));
    }
    validate_claude_limits(limits)?;
    let executable = launch
        .argv
        .get(4)
        .ok_or_else(|| invalid("Claude argv lacks its executable"))?;
    validate_provider_executable(
        Path::new(executable),
        &launch.provider_executable_sha256,
        &launch.provider_cli_version,
        &launch.environment,
        "Claude",
    )?;
    validate_claude_seatbelt_boundary(seatbelt_spec, boundary)?;
    let expected_seatbelt = generate_claude_seatbelt_profile(seatbelt_spec)?;
    require_exact_file_bytes(
        Path::new(seatbelt_profile),
        expected_seatbelt.as_bytes(),
        true,
    )?;
    if seatbelt_probe.profile_sha256 != sha256_bytes(expected_seatbelt.as_bytes()) {
        return Err(invalid(
            "Claude Seatbelt probe/profile digest binding drifted",
        ));
    }
    validate_claude_seatbelt_probe(seatbelt_spec, &seatbelt_probe.profile)?;
    validate_claude_probe_audit(seatbelt_spec, seatbelt_probe)?;
    validate_engram_arm(
        engram.as_ref(),
        boundary,
        &launch.environment,
        evaluation_root,
    )?;
    if let (Some(engram), Some(daemon)) = (engram, engram_daemon) {
        validate_claude_engram_daemon_attestation(engram, &seatbelt_spec.transport, daemon)?;
    }
    validate_claude_auto_memory(auto_memory_directory.as_deref(), boundary, evaluation_root)?;
    let settings_bytes = generate_claude_settings(
        auto_memory_directory.as_deref(),
        launch.native_memory_enabled,
    )?;
    let mcp_bytes = generate_claude_mcp_config(engram.as_ref())?;
    require_exact_file_bytes(Path::new(settings), settings_bytes.as_bytes(), true)?;
    require_exact_file_bytes(Path::new(mcp_config), mcp_bytes.as_bytes(), true)?;
    let schema_bytes = read_bound_file(Path::new(output_schema), &file_sha256(output_schema)?)?;
    let schema = String::from_utf8(schema_bytes)
        .map_err(|_| invalid("Claude output schema is not UTF-8"))?;
    serde_json::from_str::<serde_json::Value>(&schema)
        .map_err(|error| invalid(format!("Claude output schema is invalid JSON: {error}")))?;
    let tools = expected_tool_surface(IsolationHost::ClaudeCode, launch.uses_engram);
    let mut expected_argv = vec![
        seatbelt_spec.sandbox_exec.clone(),
        "-f".to_string(),
        seatbelt_profile.clone(),
        "--".to_string(),
        executable.clone(),
        "--print".to_string(),
        "--output-format".to_string(),
        "stream-json".to_string(),
        "--verbose".to_string(),
        "--restricted".to_string(),
        "--permission-mode".to_string(),
        CLAUDE_PERMISSION_MODE.to_string(),
        "--strict-mcp-config".to_string(),
        "--no-session-persistence".to_string(),
        "--disable-slash-commands".to_string(),
        "--no-chrome".to_string(),
        "--setting-sources".to_string(),
        "".to_string(),
        "--permission-prompts".to_string(),
        "none".to_string(),
        "--settings".to_string(),
        settings.clone(),
        "--mcp-config".to_string(),
        mcp_config.clone(),
        "--add-dir".to_string(),
        launch.cwd.clone(),
        "--tools".to_string(),
        "Read".to_string(),
        "--disallowed-tools".to_string(),
        CLAUDE_DISALLOWED_TOOLS.to_string(),
    ];
    if engram.is_some() {
        expected_argv.extend(["--allowed-tools".to_string(), ENGRAM_AGENT_TOOLS.join(",")]);
    }
    expected_argv.extend([
        "--model".to_string(),
        limits.model.clone(),
        "--max-turns".to_string(),
        limits.max_turns.to_string(),
        "--max-budget-usd".to_string(),
        format_micro_usd(limits.max_budget_micro_usd)?,
        "--json-schema".to_string(),
        schema,
    ]);
    if let Some(engram) = engram {
        expected_argv.extend([
            "--append-system-prompt-file".to_string(),
            engram.skill_path.clone(),
        ]);
    }
    expected_argv.push(launch.prompt.clone());
    if launch.argv != expected_argv {
        return Err(invalid(
            "Claude argv differs byte-for-byte from the typed 2.1.260 contract",
        ));
    }
    let mut expected_files = vec![
        bound_file(Path::new(output_schema), true)?,
        bound_file(Path::new(settings), true)?,
        bound_file(Path::new(mcp_config), true)?,
        bound_file(Path::new(seatbelt_profile), true)?,
    ];
    if let Some(engram) = engram {
        expected_files.push(bound_file(Path::new(&engram.skill_path), true)?);
    }
    if let Some(daemon) = engram_daemon {
        expected_files.extend([
            daemon.daemon_port_file.clone(),
            daemon.daemon_pid_file.clone(),
            daemon.daemon_metadata_file.clone(),
        ]);
    }
    if launch.config_files != expected_files || launch.expected_tools != tools {
        return Err(invalid(
            "Claude configuration/tool surface is not the exact typed lane contract",
        ));
    }
    Ok(())
}

/// Enumerate and fstat every descriptor that lacks `FD_CLOEXEC` in the current runner.
///
/// macOS `_SC_OPEN_MAX` is commonly greater than 65,536. Scanning a capped numeric range would
/// therefore miss a deliberately duplicated high descriptor. `PROC_PIDLISTFDS` returns the
/// kernel's complete current descriptor inventory; every returned descriptor is then checked
/// with `F_GETFD` immediately before the provider spawn.
#[cfg(target_os = "macos")]
pub fn inspect_ambient_inheritable_fds() -> EvalResult<Vec<IsolationInheritedFd>> {
    let limit = unsafe { libc::sysconf(libc::_SC_OPEN_MAX) };
    if limit <= 0 {
        return Err(EvalError::Io(std::io::Error::last_os_error()));
    }
    let entry_capacity = usize::try_from(limit)
        .map_err(|_| invalid("macOS open-descriptor limit does not fit usize"))?;
    let entry_bytes = std::mem::size_of::<libc::proc_fdinfo>();
    let buffer_bytes = entry_capacity
        .checked_mul(entry_bytes)
        .ok_or_else(|| invalid("macOS descriptor inventory buffer size overflow"))?;
    let buffer_bytes_i32 = i32::try_from(buffer_bytes)
        .map_err(|_| invalid("macOS descriptor inventory exceeds proc_pidinfo's byte limit"))?;
    let mut inventory = Vec::<std::mem::MaybeUninit<libc::proc_fdinfo>>::new();
    inventory
        .try_reserve_exact(entry_capacity)
        .map_err(|_| invalid("could not reserve the complete macOS descriptor inventory"))?;
    // SAFETY: `MaybeUninit` permits uninitialized storage, and `proc_pidinfo` receives precisely
    // the reserved byte capacity. Only the returned, complete entries are read below.
    unsafe { inventory.set_len(entry_capacity) };
    // SAFETY: the buffer is writable for `buffer_bytes_i32`; the call does not retain it.
    let observed_bytes = unsafe {
        libc::proc_pidinfo(
            libc::getpid(),
            libc::PROC_PIDLISTFDS,
            0,
            inventory.as_mut_ptr().cast::<libc::c_void>(),
            buffer_bytes_i32,
        )
    };
    if observed_bytes <= 0 {
        return Err(EvalError::Io(std::io::Error::last_os_error()));
    }
    let observed_bytes = usize::try_from(observed_bytes)
        .map_err(|_| invalid("macOS descriptor inventory byte count is negative"))?;
    if observed_bytes > buffer_bytes || observed_bytes % entry_bytes != 0 {
        return Err(invalid(
            "macOS descriptor inventory returned a truncated entry set",
        ));
    }

    let mut descriptors = Vec::new();
    let mut seen = BTreeSet::new();
    for entry in &inventory[..observed_bytes / entry_bytes] {
        // SAFETY: `proc_pidinfo` initialized each complete returned entry.
        let fd = unsafe { entry.assume_init_ref() }.proc_fd;
        if fd < 0 || !seen.insert(fd) {
            return Err(invalid(
                "macOS descriptor inventory contains an invalid or duplicate descriptor",
            ));
        }
        // SAFETY: fcntl with F_GETFD only queries the integer descriptor.
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
        if flags == -1 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::EBADF) {
                continue;
            }
            return Err(EvalError::Io(error));
        }
        if flags & libc::FD_CLOEXEC == 0 {
            descriptors.push(fstat_descriptor(
                fd,
                "ambient_inheritable",
                &format!("fd:{fd}"),
            )?);
        }
    }
    descriptors.sort_by_key(|descriptor| descriptor.fd);
    Ok(descriptors)
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn inspect_ambient_inheritable_fds() -> EvalResult<Vec<IsolationInheritedFd>> {
    let limit = unsafe { libc::sysconf(libc::_SC_OPEN_MAX) };
    if limit <= 0 {
        return Err(EvalError::Io(std::io::Error::last_os_error()));
    }
    let limit = i32::try_from(limit)
        .map_err(|_| invalid("Unix open-descriptor limit does not fit an fd integer"))?;
    let mut descriptors = Vec::new();
    for fd in 0..limit {
        // SAFETY: fcntl with F_GETFD only queries the integer descriptor.
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
        if flags == -1 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::EBADF) {
                continue;
            }
            return Err(EvalError::Io(error));
        }
        if flags & libc::FD_CLOEXEC == 0 {
            descriptors.push(fstat_descriptor(
                fd,
                "ambient_inheritable",
                &format!("fd:{fd}"),
            )?);
        }
    }
    Ok(descriptors)
}

#[cfg(not(unix))]
pub fn inspect_ambient_inheritable_fds() -> EvalResult<Vec<IsolationInheritedFd>> {
    Err(invalid(
        "native isolation descriptor inspection requires Unix",
    ))
}

/// Create fresh owner-only stdout/stderr and fstat the exact handles just before provider spawn.
#[cfg(unix)]
pub fn prepare_isolation_stdio(
    evaluation_root: &Path,
    stdout_path: &Path,
    stderr_path: &Path,
) -> EvalResult<PreparedIsolationStdio> {
    use std::os::unix::fs::OpenOptionsExt;
    use std::os::unix::io::AsRawFd;

    let evaluation_root = canonical_directory(evaluation_root)?;
    require_private_directory(&evaluation_root, "evaluation enclave")?;
    let ambient = inspect_ambient_inheritable_fds()?;
    if ambient.iter().any(|descriptor| descriptor.fd > 2) {
        return Err(invalid(
            "runner has an unexpected inheritable descriptor immediately before spawn",
        ));
    }
    require_distinct_fresh_output_path(&evaluation_root, stdout_path, stderr_path)?;
    require_distinct_fresh_output_path(&evaluation_root, stderr_path, stdout_path)?;
    let stdin = OpenOptions::new().read(true).open("/dev/null")?;
    let stdout = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(stdout_path)?;
    let stderr = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(stderr_path)?;
    let child_descriptors = vec![
        fstat_descriptor(stdin.as_raw_fd(), "stdin_null", "/dev/null")?.with_child_fd(0),
        fstat_descriptor(
            stdout.as_raw_fd(),
            "stdout",
            &stdout_path.display().to_string(),
        )?
        .with_child_fd(1),
        fstat_descriptor(
            stderr.as_raw_fd(),
            "stderr",
            &stderr_path.display().to_string(),
        )?
        .with_child_fd(2),
    ];
    validate_generated_child_descriptors(&child_descriptors, &evaluation_root)?;
    let audit_sha256 = sha256_serialized(&(&child_descriptors, &ambient))?;
    Ok(PreparedIsolationStdio {
        audit: IsolationPreSpawnFdAudit {
            valid: true,
            child_descriptors,
            ambient_inheritable_descriptors: ambient,
            audit_sha256,
        },
        evaluation_root,
        stdin,
        stdout,
        stderr,
    })
}

#[cfg(not(unix))]
pub fn prepare_isolation_stdio(
    _evaluation_root: &Path,
    _stdout_path: &Path,
    _stderr_path: &Path,
) -> EvalResult<PreparedIsolationStdio> {
    Err(invalid("native isolation stdio preparation requires Unix"))
}

impl IsolationInheritedFd {
    fn with_child_fd(mut self, fd: i32) -> Self {
        self.fd = fd;
        self
    }
}

#[cfg(unix)]
fn fstat_descriptor(fd: i32, purpose: &str, target: &str) -> EvalResult<IsolationInheritedFd> {
    let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
    // SAFETY: `stat` is valid writable storage and `fd` was proven live by the caller/open.
    if unsafe { libc::fstat(fd, stat.as_mut_ptr()) } != 0 {
        return Err(EvalError::Io(std::io::Error::last_os_error()));
    }
    // SAFETY: successful fstat initialized the complete output structure.
    let stat = unsafe { stat.assume_init() };
    let device =
        u64::try_from(stat.st_dev).map_err(|_| invalid("descriptor device number is negative"))?;
    Ok(IsolationInheritedFd {
        fd,
        purpose: purpose.to_string(),
        target: target.to_string(),
        device,
        inode: stat.st_ino,
        mode: u32::from(stat.st_mode),
        link_count: u64::from(stat.st_nlink),
    })
}

fn require_distinct_fresh_output_path(
    evaluation_root: &Path,
    path: &Path,
    other: &Path,
) -> EvalResult<()> {
    require_normalized_absolute_path(path, "provider output")?;
    if path == other || !path.starts_with(evaluation_root) {
        return Err(invalid(
            "provider output paths must be distinct enclave paths",
        ));
    }
    if !fs::symlink_metadata(path).is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound)
    {
        return Err(invalid("provider output path is not fresh"));
    }
    let parent = canonical_directory(
        path.parent()
            .ok_or_else(|| invalid("provider output has no parent"))?,
    )?;
    require_private_directory(&parent, "provider output directory")?;
    if !parent.starts_with(evaluation_root) {
        return Err(invalid("provider output parent escapes the enclave"));
    }
    Ok(())
}

fn validate_generated_child_descriptors(
    descriptors: &[IsolationInheritedFd],
    evaluation_root: &Path,
) -> EvalResult<()> {
    if descriptors.len() != 3
        || descriptors
            .iter()
            .map(|descriptor| descriptor.fd)
            .collect::<Vec<_>>()
            != [0, 1, 2]
        || descriptors[0].purpose != "stdin_null"
        || descriptors[0].target != "/dev/null"
    {
        return Err(invalid(
            "runner-generated child descriptor set is not exact",
        ));
    }
    for (descriptor, purpose) in descriptors[1..].iter().zip(["stdout", "stderr"]) {
        let path = Path::new(&descriptor.target);
        let metadata = fs::symlink_metadata(path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::{MetadataExt, PermissionsExt};
            if descriptor.purpose != purpose
                || !path.starts_with(evaluation_root)
                || metadata.permissions().mode() & 0o777 != 0o600
                || metadata.dev() != descriptor.device
                || metadata.ino() != descriptor.inode
                || metadata.nlink() != 1
            {
                return Err(invalid(format!(
                    "runner-generated output descriptor fstat drifted: purpose={} expected={} inside={} mode={:o} dev={}/{} ino={}/{} nlink={}",
                    descriptor.purpose,
                    purpose,
                    path.starts_with(evaluation_root),
                    metadata.permissions().mode() & 0o777,
                    metadata.dev(),
                    descriptor.device,
                    metadata.ino(),
                    descriptor.inode,
                    metadata.nlink(),
                )));
            }
        }
    }
    Ok(())
}

/// Output/disk/cache gates observed immediately before an executable phase.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IsolationLocalGateAudit {
    pub valid: bool,
    pub available_bytes: u64,
    pub required_bytes: u64,
    pub outputs_absent: bool,
    pub phase_root_empty: bool,
    pub repository_target_debug_absent: bool,
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IsolationExecutablePhase {
    Activation,
    Evaluation,
}

/// Closed-world output set and positive disk reserve for one future executable phase.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IsolationPhaseOutputContract {
    pub phase: IsolationExecutablePhase,
    pub phase_root: String,
    pub required_bytes: u64,
    pub output_paths: Vec<String>,
}

pub fn derive_isolation_phase_outputs(
    phase: IsolationExecutablePhase,
    phase_root: &Path,
    required_bytes: u64,
) -> EvalResult<IsolationPhaseOutputContract> {
    if required_bytes == 0 {
        return Err(invalid("isolation phase disk reserve must be positive"));
    }
    let phase_root = canonical_directory(phase_root)?;
    require_private_directory(&phase_root, "phase output directory")?;
    let prefix = match phase {
        IsolationExecutablePhase::Activation => "activation",
        IsolationExecutablePhase::Evaluation => "evaluation",
    };
    let output_paths = [
        "trace.jsonl",
        "stderr.log",
        "agent-output.json",
        "runner-receipt.json",
    ]
    .iter()
    .map(|suffix| {
        phase_root
            .join(format!("{prefix}-{suffix}"))
            .display()
            .to_string()
    })
    .collect();
    Ok(IsolationPhaseOutputContract {
        phase,
        phase_root: phase_root.display().to_string(),
        required_bytes,
        output_paths,
    })
}

pub fn inspect_isolation_local_gates(
    disk_path: &Path,
    phase: &IsolationPhaseOutputContract,
    repository_root: &Path,
) -> EvalResult<IsolationLocalGateAudit> {
    let expected = derive_isolation_phase_outputs(
        phase.phase,
        Path::new(&phase.phase_root),
        phase.required_bytes,
    )?;
    if &expected != phase || phase.output_paths.is_empty() {
        return Err(invalid(
            "phase output contract is not the derived closed-world set",
        ));
    }
    let mut failures = Vec::new();
    let available_bytes = available_space(disk_path)?;
    if available_bytes < phase.required_bytes {
        failures.push("disk reserve is below the frozen requirement".to_string());
    }
    let outputs_absent = phase.output_paths.iter().all(|path| {
        fs::symlink_metadata(path).is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound)
    });
    if !outputs_absent {
        failures.push("one or more future phase outputs already exist".to_string());
    }
    let phase_root_empty = fs::read_dir(&phase.phase_root)?.next().is_none();
    if !phase_root_empty {
        failures.push(
            "phase root contains an output, sidecar, temporary, or other residue".to_string(),
        );
    }
    let repository_target_debug_absent = fs::symlink_metadata(repository_root.join("target/debug"))
        .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound);
    if !repository_target_debug_absent {
        failures.push("repository target/debug exists".to_string());
    }
    Ok(IsolationLocalGateAudit {
        valid: failures.is_empty(),
        available_bytes,
        required_bytes: phase.required_bytes,
        outputs_absent,
        phase_root_empty,
        repository_target_debug_absent,
        failures,
    })
}

/// Frozen design for a later mount-free Colima environment.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ColimaIsolationContract {
    pub schema_version: u32,
    pub profile_name: String,
    pub vm_type: String,
    pub architecture: String,
    pub mount_mode: String,
    pub rosetta: bool,
    pub forward_ssh_agent: bool,
    pub disk_gib: u32,
    pub transfer_mode: String,
    pub teaching_user: String,
    pub evaluation_user: String,
    pub required_runtimes: BTreeMap<String, ColimaRuntimeAttestation>,
    pub colima_profile_sha256: String,
    pub bwrap_profile_sha256: String,
    pub command_contracts: Vec<ColimaCommandContract>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ColimaCommandContract {
    pub label: String,
    pub argv: Vec<String>,
    pub argv_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ColimaCommandReceipt {
    pub label: String,
    pub argv_sha256: String,
    pub exit_code: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ColimaMountObservation {
    pub source: String,
    pub target: String,
    pub filesystem: String,
    pub options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ColimaRuntimeAttestation {
    pub path: String,
    pub version: String,
    pub sha256: String,
}

/// Provider-free observations from a future disposable guest. No provider result belongs here.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ColimaIsolationObservation {
    /// False for an unexecuted/synthetic model; such a model can never authorize a provider run.
    pub observed_in_disposable_guest: bool,
    pub environment_id: String,
    pub operating_system: String,
    pub distribution: String,
    pub architecture: String,
    pub root_filesystem: String,
    pub mounts: Vec<ColimaMountObservation>,
    pub host_home_visible: bool,
    pub rosetta_registered: bool,
    pub ssh_auth_sock_present: bool,
    pub ssh_forward_agent: bool,
    pub docker_socket_present: bool,
    pub teaching_uid: u32,
    pub evaluation_uid: u32,
    pub bwrap: Option<ColimaRuntimeAttestation>,
    pub apparmor_enabled: bool,
    pub bwrap_apparmor_profile_enforced: bool,
    pub bwrap_cross_uid_probe_passed: bool,
    pub runtimes: BTreeMap<String, ColimaRuntimeAttestation>,
    pub transfer_manifest_sha256: String,
    pub guest_manifest_sha256: String,
    pub guest_bundle_on_vm_owned_ext4: bool,
    pub authentication_performed_inside_guest: bool,
    pub api_key_fallback_used: bool,
    pub colima_profile_sha256: String,
    pub bwrap_profile_sha256: String,
    pub command_receipts: Vec<ColimaCommandReceipt>,
}

/// Provider-free attestation for the future Linux replication boundary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ColimaIsolationAudit {
    pub valid: bool,
    /// This standalone foundation is deliberately never an execution authorization.
    pub authorizes_provider_execution: bool,
    pub environment_id: String,
    pub failures: Vec<String>,
    pub claim: String,
}

pub fn validate_colima_isolation(
    contract: &ColimaIsolationContract,
    observation: &ColimaIsolationObservation,
) -> ColimaIsolationAudit {
    let mut failures = Vec::new();
    if contract.schema_version != COLIMA_ISOLATION_SCHEMA_VERSION {
        failures.push("unsupported Colima isolation schema".to_string());
    }
    if !observation.observed_in_disposable_guest {
        failures.push("Colima observations were not collected in a disposable guest".to_string());
    }
    if contract.profile_name.trim().is_empty() || contract.profile_name == "default" {
        failures.push("Colima profile must be a named disposable profile".to_string());
    }
    if contract.vm_type != "vz"
        || contract.architecture != "aarch64"
        || contract.mount_mode != "none"
        || contract.rosetta
        || contract.forward_ssh_agent
        || contract.disk_gib < MIN_VM_DISK_GIB
        || contract.transfer_mode != "ssh_stream_hash_manifest"
    {
        failures.push("Colima launch contract is not the frozen mount-free VZ shape".to_string());
    }
    if contract.teaching_user == contract.evaluation_user
        || contract.teaching_user.is_empty()
        || contract.evaluation_user.is_empty()
    {
        failures.push("teaching and evaluation must use distinct guest users".to_string());
    }
    let exact_runtime_names = BTreeSet::from([
        "cargo".to_string(),
        "claude".to_string(),
        "codex".to_string(),
        "engram".to_string(),
        "engram_eval".to_string(),
        "node".to_string(),
        "rustc".to_string(),
    ]);
    if contract
        .required_runtimes
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>()
        != exact_runtime_names
        || !exact_distinct_runtime_attestations(&contract.required_runtimes)
    {
        failures.push("Colima contract does not freeze the exact required runtime set".to_string());
    }
    if !is_sha256(&contract.colima_profile_sha256)
        || !is_sha256(&contract.bwrap_profile_sha256)
        || observation.colima_profile_sha256 != contract.colima_profile_sha256
        || observation.bwrap_profile_sha256 != contract.bwrap_profile_sha256
        || !validate_colima_command_receipts(contract, observation)
    {
        failures.push("Colima profile or command receipts are not exact".to_string());
    }
    if observation.environment_id.trim().is_empty()
        || observation.operating_system != "linux"
        || !observation.distribution.starts_with("ubuntu-24.04")
        || observation.architecture != contract.architecture
        || observation.root_filesystem != "ext4"
    {
        failures
            .push("guest identity/OS/filesystem does not match the frozen contract".to_string());
    }
    validate_colima_mounts(observation, &mut failures);
    if observation.host_home_visible
        || observation.rosetta_registered
        || observation.ssh_auth_sock_present
        || observation.ssh_forward_agent
        || observation.docker_socket_present
    {
        failures
            .push("guest exposes a forbidden host bridge, socket, agent, or Rosetta".to_string());
    }
    if observation.teaching_uid == 0
        || observation.evaluation_uid == 0
        || observation.teaching_uid == observation.evaluation_uid
    {
        failures
            .push("guest teaching/evaluation UIDs are not distinct unprivileged users".to_string());
    }
    if observation.bwrap.as_ref().map_or(true, |binary| {
        !is_sha256(&binary.sha256)
            || !Path::new(&binary.path).is_absolute()
            || binary.version.is_empty()
    }) || !observation.apparmor_enabled
        || !observation.bwrap_apparmor_profile_enforced
        || !observation.bwrap_cross_uid_probe_passed
    {
        failures.push("official bubblewrap/AppArmor prerequisite is not proven".to_string());
    }
    if observation.runtimes != contract.required_runtimes
        || !exact_distinct_runtime_attestations(&observation.runtimes)
    {
        failures.push("guest runtime set or runtime attestation is incomplete".to_string());
    }
    if !is_sha256(&observation.transfer_manifest_sha256)
        || observation.transfer_manifest_sha256 != observation.guest_manifest_sha256
        || !observation.guest_bundle_on_vm_owned_ext4
    {
        failures.push("streamed bundle was not rehashed on VM-owned ext4".to_string());
    }
    if !observation.authentication_performed_inside_guest || observation.api_key_fallback_used {
        failures
            .push("guest authentication is absent or silently changed to API-key mode".to_string());
    }
    ColimaIsolationAudit {
        valid: failures.is_empty(),
        authorizes_provider_execution: false,
        environment_id: observation.environment_id.clone(),
        failures,
        claim:
            "bounded separate OS/filesystem/runtime on the same physical Mac, account, and network"
                .to_string(),
    }
}

fn validate_colima_mounts(observation: &ColimaIsolationObservation, failures: &mut Vec<String>) {
    let root = observation.mounts.iter().find(|mount| mount.target == "/");
    if root.map_or(true, |mount| mount.filesystem != "ext4") {
        failures.push("findmnt does not show an ext4 guest root".to_string());
    }
    let recognized = BTreeMap::from([
        ("/", "ext4"),
        ("/dev", "devtmpfs"),
        ("/proc", "proc"),
        ("/run", "tmpfs"),
        ("/sys", "sysfs"),
        ("/tmp", "tmpfs"),
    ]);
    let mut targets = BTreeSet::new();
    if observation.mounts.is_empty()
        || observation.mounts.iter().any(|mount| {
            !targets.insert(mount.target.as_str())
                || recognized.get(mount.target.as_str()).copied() != Some(mount.filesystem.as_str())
                || matches!(mount.filesystem.as_str(), "virtiofs" | "9p" | "fuse.sshfs")
                || mount.target.starts_with("/Users")
                || mount.target.starts_with("/Volumes")
                || mount.source.contains("/Users/")
                || mount.source.contains("/Volumes/")
                || mount.target == "/var/run/docker.sock"
        })
    {
        failures.push("findmnt contains a host/shared filesystem or Docker socket".to_string());
    }
}

fn exact_distinct_runtime_attestations(
    runtimes: &BTreeMap<String, ColimaRuntimeAttestation>,
) -> bool {
    let paths = runtimes
        .values()
        .map(|runtime| runtime.path.as_str())
        .collect::<BTreeSet<_>>();
    let hashes = runtimes
        .values()
        .map(|runtime| runtime.sha256.as_str())
        .collect::<BTreeSet<_>>();
    paths.len() == runtimes.len()
        && hashes.len() == runtimes.len()
        && runtimes.values().all(|runtime| {
            runtime.version.trim().len() >= 3
                && Path::new(&runtime.path).is_absolute()
                && is_sha256(&runtime.sha256)
        })
}

fn validate_colima_command_receipts(
    contract: &ColimaIsolationContract,
    observation: &ColimaIsolationObservation,
) -> bool {
    let required = BTreeSet::from([
        "bwrap_probe",
        "colima_start",
        "findmnt",
        "guest_rehash",
        "runtime_attestation",
        "ssh_config",
        "transfer_bundle",
    ]);
    let labels = contract
        .command_contracts
        .iter()
        .map(|command| command.label.as_str())
        .collect::<BTreeSet<_>>();
    if labels != required || labels.len() != contract.command_contracts.len() {
        return false;
    }
    let expected = contract
        .command_contracts
        .iter()
        .map(|command| {
            (
                command.label.as_str(),
                (
                    command.argv_sha256.as_str(),
                    sha256_serialized(&command.argv).ok(),
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let observed = observation
        .command_receipts
        .iter()
        .map(|receipt| (receipt.label.as_str(), receipt))
        .collect::<BTreeMap<_, _>>();
    expected.len() == contract.command_contracts.len()
        && observed.len() == observation.command_receipts.len()
        && expected.len() == observed.len()
        && expected
            .iter()
            .all(|(label, (expected_digest, recomputed))| {
                recomputed.as_deref() == Some(*expected_digest)
                    && observed.get(label).is_some_and(|receipt| {
                        receipt.exit_code == 0 && receipt.argv_sha256 == *expected_digest
                    })
            })
}

fn clean_relative(value: &str, label: &str) -> EvalResult<PathBuf> {
    let path = Path::new(value);
    if value.is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(invalid(format!("{label} must be a clean relative path")));
    }
    Ok(path.to_path_buf())
}

fn lexical_relative_path(base: &Path, target: &Path) -> EvalResult<PathBuf> {
    if !base.is_absolute() || !target.is_absolute() {
        return Err(invalid("relative-path inputs must be absolute"));
    }
    let base = base.components().collect::<Vec<_>>();
    let target = target.components().collect::<Vec<_>>();
    let common = base
        .iter()
        .zip(&target)
        .take_while(|(left, right)| left == right)
        .count();
    if common == 0 {
        return Err(invalid("relative-path inputs have no common root"));
    }
    let mut relative = PathBuf::new();
    for _ in common..base.len() {
        relative.push("..");
    }
    for component in &target[common..] {
        relative.push(component.as_os_str());
    }
    if relative.as_os_str().is_empty() {
        return Err(invalid("relative-path target equals its base"));
    }
    Ok(relative)
}

fn canonical_directory(path: &Path) -> EvalResult<PathBuf> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(invalid(format!(
            "expected a real directory: {}",
            path.display()
        )));
    }
    path.canonicalize().map_err(EvalError::Io)
}

#[cfg(unix)]
fn require_private_directory(path: &Path, label: &str) -> EvalResult<()> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    let metadata = fs::symlink_metadata(path)?;
    // SAFETY: `geteuid` has no preconditions and does not access caller memory.
    let effective_uid = unsafe { libc::geteuid() };
    if metadata.file_type().is_symlink()
        || !metadata.is_dir()
        || metadata.permissions().mode() & 0o777 != 0o700
        || metadata.uid() != effective_uid
    {
        return Err(invalid(format!(
            "{label} must be an owner-controlled mode-0700 directory"
        )));
    }
    Ok(())
}

#[cfg(unix)]
fn require_owner_private_path(path: &Path, label: &str) -> EvalResult<()> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    let metadata = fs::symlink_metadata(path)?;
    // SAFETY: `geteuid` has no preconditions and does not access caller memory.
    let effective_uid = unsafe { libc::geteuid() };
    let mode = metadata.permissions().mode() & 0o777;
    if metadata.uid() != effective_uid
        || metadata.file_type().is_symlink()
        || (metadata.is_dir() && mode != 0o700)
        || (metadata.is_file() && mode != 0o600)
        || (!metadata.is_dir() && !metadata.is_file())
    {
        return Err(invalid(format!(
            "{label} must be an owner-controlled 0700 directory or 0600 file"
        )));
    }
    Ok(())
}

#[cfg(not(unix))]
fn require_owner_private_path(_path: &Path, _label: &str) -> EvalResult<()> {
    Err(invalid(
        "native isolation requires Unix path ownership checks",
    ))
}

#[cfg(not(unix))]
fn require_private_directory(_path: &Path, _label: &str) -> EvalResult<()> {
    Err(invalid(
        "native isolation requires Unix directory ownership checks",
    ))
}

fn require_disjoint_roots(left: &Path, right: &Path) -> EvalResult<()> {
    if left == right || left.starts_with(right) || right.starts_with(left) {
        return Err(invalid("source and evaluation roots overlap"));
    }
    Ok(())
}

fn validate_existing_ancestors(root: &Path, target: &Path) -> EvalResult<()> {
    if !target.starts_with(root) {
        return Err(invalid("path escaped its declared root"));
    }
    let relative = target
        .strip_prefix(root)
        .map_err(|_| invalid("path escaped its declared root"))?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        current.push(component.as_os_str());
        let metadata = fs::symlink_metadata(&current)?;
        if metadata.file_type().is_symlink() {
            return Err(invalid(format!(
                "path contains a symlink: {}",
                current.display()
            )));
        }
    }
    Ok(())
}

fn create_private_directory_tree(path: &Path) -> EvalResult<()> {
    let mut missing = Vec::new();
    let mut current = path;
    while !current.exists() {
        missing.push(current.to_path_buf());
        current = current
            .parent()
            .ok_or_else(|| invalid("directory tree has no existing ancestor"))?;
    }
    let ancestor = fs::symlink_metadata(current)?;
    if ancestor.file_type().is_symlink() || !ancestor.is_dir() {
        return Err(invalid("directory tree ancestor is not a real directory"));
    }
    fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for directory in missing {
            fs::set_permissions(directory, fs::Permissions::from_mode(0o700))?;
        }
    }
    Ok(())
}

fn require_nonoverlapping_relative_roots(paths: &[PathBuf], label: &str) -> EvalResult<()> {
    for (index, left) in paths.iter().enumerate() {
        if paths
            .iter()
            .skip(index + 1)
            .any(|right| left == right || left.starts_with(right) || right.starts_with(left))
        {
            return Err(invalid(format!("{label} overlap")));
        }
    }
    Ok(())
}

fn require_nonoverlapping_absolute_paths(paths: &[PathBuf], label: &str) -> EvalResult<()> {
    for path in paths {
        require_normalized_absolute_path(path, label)?;
    }
    for (index, left) in paths.iter().enumerate() {
        if paths
            .iter()
            .skip(index + 1)
            .any(|right| left == right || left.starts_with(right) || right.starts_with(left))
        {
            return Err(invalid(format!("{label} overlap")));
        }
    }
    Ok(())
}

fn require_normalized_absolute_path(path: &Path, label: &str) -> EvalResult<()> {
    if !path.is_absolute()
        || path.as_os_str().is_empty()
        || path
            .components()
            .any(|component| !matches!(component, Component::RootDir | Component::Normal(_)))
        || path.to_string_lossy().contains('\n')
        || path.to_string_lossy().contains('\0')
    {
        return Err(invalid(format!(
            "{label} must be an absolute normalized single-line path"
        )));
    }
    Ok(())
}

fn require_unique_strings<'a>(
    values: impl Iterator<Item = &'a String>,
    label: &str,
) -> EvalResult<()> {
    let values = values.collect::<Vec<_>>();
    let unique = values
        .iter()
        .map(|value| value.as_str())
        .collect::<BTreeSet<_>>();
    if values.len() != unique.len() {
        return Err(invalid(format!("duplicate {label}")));
    }
    Ok(())
}

fn require_unique_absolute_paths(values: &[String], label: &str) -> EvalResult<()> {
    require_unique_strings(values.iter(), label)?;
    for value in values {
        require_normalized_absolute_path(Path::new(value), label)?;
    }
    Ok(())
}

#[cfg(unix)]
fn require_single_link(metadata: &fs::Metadata, path: &Path) -> EvalResult<()> {
    use std::os::unix::fs::MetadataExt;
    if metadata.nlink() != 1 {
        return Err(invalid(format!(
            "hard-linked files are forbidden: {}",
            path.display()
        )));
    }
    Ok(())
}

#[cfg(not(unix))]
fn require_single_link(_metadata: &fs::Metadata, _path: &Path) -> EvalResult<()> {
    Err(invalid("native isolation requires Unix hard-link checks"))
}

fn seatbelt_escape(path: &str) -> EvalResult<String> {
    if !Path::new(path).is_absolute() || path.contains('\n') || path.contains('\0') {
        return Err(invalid("Seatbelt paths must be absolute and single-line"));
    }
    Ok(path.replace('\\', "\\\\").replace('"', "\\\""))
}

fn toml_basic_string(value: &str) -> EvalResult<String> {
    if value.contains('\n') || value.contains('\r') || value.contains('\0') {
        return Err(invalid("TOML permission-profile path must be single-line"));
    }
    Ok(value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

fn bound_file(path: &Path, owner_only: bool) -> EvalResult<IsolationFileBinding> {
    require_normalized_absolute_path(path, "bound file")?;
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || path.canonicalize()?.display().to_string() != path.display().to_string()
        || (owner_only && metadata.len() > MAX_MANIFEST_FILE_BYTES)
    {
        return Err(invalid("bound file must be a canonical regular file"));
    }
    require_single_link(&metadata, path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        // SAFETY: `geteuid` has no preconditions and does not access caller memory.
        let effective_uid = unsafe { libc::geteuid() };
        if metadata.uid() != effective_uid
            || (owner_only && metadata.permissions().mode() & 0o777 != 0o600)
        {
            return Err(invalid(
                "bound file is not owned by the effective user with mode 0600",
            ));
        }
    }
    Ok(IsolationFileBinding {
        path: path.display().to_string(),
        sha256: read_regular_file_digest(path)?,
    })
}

fn file_sha256(path: &str) -> EvalResult<String> {
    read_regular_file_digest(Path::new(path))
}

fn read_bound_file(path: &Path, expected_sha256: &str) -> EvalResult<Vec<u8>> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_MANIFEST_TOTAL_BYTES
    {
        return Err(invalid("hash-bound file is not a bounded regular file"));
    }
    require_single_link(&metadata, path)?;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    #[cfg(unix)]
    let mode = metadata.permissions().mode() & 0o777;
    #[cfg(not(unix))]
    let mode = 0;
    let bytes = read_exact_regular_file(path, metadata.len(), mode, MAX_MANIFEST_TOTAL_BYTES)?;
    if !is_sha256(expected_sha256) || sha256_bytes(&bytes) != expected_sha256 {
        return Err(invalid("hash-bound file digest drifted"));
    }
    Ok(bytes)
}

fn read_regular_file_digest(path: &Path) -> EvalResult<String> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_MANIFEST_TOTAL_BYTES
    {
        return Err(invalid(
            "executable/config binding is not a bounded regular file",
        ));
    }
    require_single_link(&metadata, path)?;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    #[cfg(unix)]
    let mode = metadata.permissions().mode() & 0o777;
    #[cfg(not(unix))]
    let mode = 0;
    let bytes = read_exact_regular_file(path, metadata.len(), mode, MAX_MANIFEST_TOTAL_BYTES)?;
    Ok(sha256_bytes(&bytes))
}

fn require_exact_file_bytes(path: &Path, expected: &[u8], owner_only: bool) -> EvalResult<()> {
    let binding = bound_file(path, owner_only)?;
    let bytes = read_bound_file(path, &binding.sha256)?;
    if bytes != expected {
        return Err(invalid("generated configuration bytes drifted"));
    }
    Ok(())
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn sha256_serialized(value: &impl Serialize) -> EvalResult<String> {
    Ok(sha256_bytes(&serde_json::to_vec(value)?))
}

fn is_api_key_environment(key: &str) -> bool {
    matches!(
        key,
        "OPENAI_API_KEY"
            | "ANTHROPIC_API_KEY"
            | "CODEX_ACCESS_TOKEN"
            | "AWS_ACCESS_KEY_ID"
            | "AWS_SECRET_ACCESS_KEY"
    )
}

#[cfg(unix)]
fn available_space(path: &Path) -> EvalResult<u64> {
    use std::os::unix::ffi::OsStrExt;
    let path = CString::new(path.as_os_str().as_bytes())
        .map_err(|_| invalid("disk path contains a NUL byte"))?;
    let mut stat = std::mem::MaybeUninit::<libc::statvfs>::uninit();
    // SAFETY: `path` is a valid C string and `stat` points to writable initialized-on-success
    // storage owned by this function.
    let result = unsafe { libc::statvfs(path.as_ptr(), stat.as_mut_ptr()) };
    if result != 0 {
        return Err(EvalError::Io(std::io::Error::last_os_error()));
    }
    // SAFETY: statvfs returned success and initialized the output structure.
    let stat = unsafe { stat.assume_init() };
    u64::from(stat.f_bavail)
        .checked_mul(stat.f_frsize)
        .ok_or_else(|| invalid("available disk byte count overflow"))
}

#[cfg(not(unix))]
fn available_space(_path: &Path) -> EvalResult<u64> {
    Err(invalid("native isolation disk gates require Unix"))
}

fn invalid(message: impl Into<String>) -> EvalError {
    EvalError::Invalid(message.into())
}
