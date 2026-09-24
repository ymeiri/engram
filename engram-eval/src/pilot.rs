//! Reproducible, execution-gated preparation for the native-host pilot.

use crate::fixture::{materialize_engineering_context_fixture, FixtureLayout};
use crate::seed::seed_engineering_context_fixture;
use crate::{load_suite, EvalError, EvalResult, EvaluationArm, EvaluationManifest, Scenario};
use engram_core::harness::HarnessKind;
use engram_index::HarnessService;
use engram_store::ENGRAM_HOME_ENV;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const PILOT_SCHEMA_VERSION: u32 = 1;
pub(crate) const CODEX_CODE_MODE_HOST_NAME: &str = "codex-code-mode-host";
pub(crate) const AGENT_OUTPUT_SCHEMA: &str = r#"{
  "type": "object",
  "properties": {
    "answer": {"type": "string"},
    "repository_remote": {"type": ["string", "null"]},
    "project": {"type": ["string", "null"]},
    "component": {"type": ["string", "null"]},
    "first_action": {"type": ["string", "null"]},
    "returned_context_keys": {"type": "array", "items": {"type": "string"}},
    "applied_context_keys": {"type": "array", "items": {"type": "string"}},
    "evidence_targets": {"type": "array", "items": {"type": "string"}},
    "abstained": {"type": "boolean"}
  },
  "required": [
    "answer",
    "repository_remote",
    "project",
    "component",
    "first_action",
    "returned_context_keys",
    "applied_context_keys",
    "evidence_targets",
    "abstained"
  ],
  "additionalProperties": false
}
"#;

/// One run in the preregistered pilot order.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct PilotRunRef {
    /// Scenario from the frozen base suite.
    pub scenario_id: String,
    /// Arm from the frozen base suite.
    pub arm: String,
    /// One-based repetition.
    pub repetition: u32,
}

/// A small preregistered subset of a frozen evaluation suite.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PilotProtocol {
    /// Protocol schema version.
    pub schema_version: u32,
    /// Stable pilot identifier.
    pub pilot_id: String,
    /// Frozen suite manifest, relative to this protocol file.
    pub base_manifest: String,
    /// Selected scenario IDs.
    pub scenario_ids: Vec<String>,
    /// Selected comparison arms.
    pub arm_ids: Vec<String>,
    /// Required repetitions per scenario/arm pair.
    pub repetitions: u32,
    /// Fixed randomized order declared before provider execution.
    pub run_order: Vec<PilotRunRef>,
    /// Hard aggregate Claude Code budget in cents.
    pub claude_budget_cents: u32,
    /// Provider execution must remain disabled until the user approves it.
    pub requires_explicit_execution_approval: bool,
}

/// Immutable local executable identity recorded before paid execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PilotBinaryAttestation {
    /// Canonical executable path.
    pub path: String,
    /// Version identity, normally from `--version`.
    pub version: String,
    /// SHA-256 of the executable bytes.
    pub sha256: String,
}

/// One prepared, but not executed, native-host run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PreparedPilotRun {
    /// One-based execution order.
    pub order: u32,
    /// Frozen scenario ID.
    pub scenario_id: String,
    /// Frozen arm ID.
    pub arm: String,
    /// Host name from the base manifest.
    pub host: String,
    /// One-based repetition.
    pub repetition: u32,
    /// Scenario prompt, unchanged from the frozen suite.
    pub prompt: String,
    /// Materialized absolute cwd.
    pub cwd: Option<String>,
    /// Explicit project supplied by the frozen scenario.
    pub project: Option<String>,
    /// Per-run fixture root.
    pub fixture_root: String,
    /// Path-independent fixture content revision.
    pub fixture_revision: String,
    /// SHA-256 over generated native boundary adapter files.
    pub adapter_sha256: Option<String>,
    /// Generated adapter directory outside the fixture checkout.
    pub adapter_root: Option<String>,
    /// Isolated Engram state root for Engram arms.
    pub engram_home: Option<String>,
    /// Isolated daemon project for Engram arms.
    pub engram_project: Option<String>,
    /// Seeded RocksDB directory for Engram arms.
    pub data_dir: Option<String>,
    /// Environment passed only to the host process.
    pub environment: BTreeMap<String, String>,
    /// Exact host argv, recorded but never executed by preparation.
    pub argv: Vec<String>,
    /// Exact cleanup argv for an isolated Engram daemon, when applicable.
    pub cleanup_argv: Option<Vec<String>>,
    /// Future raw host trace path.
    pub trace_path: String,
    /// Future structured agent result path.
    pub agent_output_path: String,
}

/// Fully materialized pilot plan. Creating this report never calls a model provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PreparedPilot {
    /// Pilot identifier.
    pub pilot_id: String,
    /// Canonical frozen base manifest path.
    pub base_manifest: String,
    /// Derived subset manifest.
    pub derived_manifest: String,
    /// Shared structured-output schema copied into the output directory.
    pub agent_output_schema: String,
    /// Aggregate Claude Code hard budget in cents.
    pub claude_budget_cents: u32,
    /// Always false: preparation cannot authorize provider execution.
    pub execution_approved: bool,
    /// Attested local executables used in command previews.
    pub binaries: BTreeMap<String, PilotBinaryAttestation>,
    /// Fixed run order and isolated paths.
    pub runs: Vec<PreparedPilotRun>,
}

/// Resolve, attest, materialize, and seed a pilot without invoking Codex or Claude Code.
pub async fn prepare_pilot(
    protocol_path: &Path,
    output: &Path,
    engram_binary: &Path,
    codex_binary: &Path,
    claude_binary: &Path,
) -> EvalResult<PreparedPilot> {
    require_empty_target(output)?;
    let output = output.canonicalize()?;
    let protocol: PilotProtocol = serde_json::from_reader(fs::File::open(protocol_path)?)?;
    let base_manifest = resolve_relative(protocol_path, &protocol.base_manifest)?;
    let (base_suite, base_scenarios) = load_suite(&base_manifest)?;
    validate_protocol(&protocol, &base_suite, &base_scenarios)?;

    let engram = attest_binary(engram_binary)?;
    if !engram.version.contains(env!("CARGO_PKG_VERSION")) {
        return Err(EvalError::Invalid(format!(
            "Engram binary version '{}' does not match evaluation crate {}",
            engram.version,
            env!("CARGO_PKG_VERSION")
        )));
    }
    let codex = attest_binary(codex_binary)?;
    let claude = attest_binary(claude_binary)?;
    let binaries = BTreeMap::from([
        ("claude_code".to_string(), claude.clone()),
        ("codex".to_string(), codex.clone()),
        ("engram".to_string(), engram.clone()),
    ]);

    fs::create_dir_all(&output)?;
    let selected_arms = select_arms(&protocol, &base_suite)?;
    let selected_scenarios = select_scenarios(&protocol, &base_scenarios)?;
    let derived_manifest_path = output.join("manifest.json");
    let derived_scenarios_path = output.join("scenarios.jsonl");
    let schema_path = output.join("agent-output.schema.json");
    write_derived_suite(
        &protocol,
        selected_arms,
        &selected_scenarios,
        &derived_manifest_path,
        &derived_scenarios_path,
    )?;
    fs::write(&schema_path, AGENT_OUTPUT_SCHEMA)?;

    let arms_by_id = base_suite
        .arms
        .iter()
        .map(|arm| (arm.id.as_str(), arm))
        .collect::<BTreeMap<_, _>>();
    let scenarios_by_id = base_scenarios
        .iter()
        .map(|scenario| (scenario.id.as_str(), scenario))
        .collect::<BTreeMap<_, _>>();
    let claude_runs = protocol
        .run_order
        .iter()
        .filter(|run| arms_by_id[run.arm.as_str()].host == "claude_code")
        .count();
    let claude_budget_millis = if claude_runs == 0 {
        0
    } else {
        u64::from(protocol.claude_budget_cents) * 10 / claude_runs as u64
    };

    let mut runs = Vec::with_capacity(protocol.run_order.len());
    for (index, run_ref) in protocol.run_order.iter().enumerate() {
        let order = u32::try_from(index + 1)
            .map_err(|_| EvalError::Invalid("pilot contains too many runs".to_string()))?;
        let arm = arms_by_id[run_ref.arm.as_str()];
        let scenario = scenarios_by_id[run_ref.scenario_id.as_str()];
        runs.push(
            prepare_run(
                &output,
                order,
                run_ref,
                arm,
                scenario,
                &engram,
                &codex,
                &claude,
                &schema_path,
                claude_budget_millis,
            )
            .await?,
        );
    }

    let prepared = PreparedPilot {
        pilot_id: protocol.pilot_id,
        base_manifest: base_manifest.canonicalize()?.display().to_string(),
        derived_manifest: derived_manifest_path.canonicalize()?.display().to_string(),
        agent_output_schema: schema_path.canonicalize()?.display().to_string(),
        claude_budget_cents: protocol.claude_budget_cents,
        execution_approved: false,
        binaries,
        runs,
    };
    fs::write(
        output.join("run-plan.json"),
        serde_json::to_string_pretty(&prepared)? + "\n",
    )?;
    Ok(prepared)
}

fn validate_protocol(
    protocol: &PilotProtocol,
    base_suite: &EvaluationManifest,
    scenarios: &[Scenario],
) -> EvalResult<()> {
    if protocol.schema_version != PILOT_SCHEMA_VERSION {
        return Err(EvalError::Invalid(format!(
            "unsupported pilot schema version: {}",
            protocol.schema_version
        )));
    }
    if protocol.pilot_id.trim().is_empty() {
        return Err(EvalError::Invalid("pilot_id must not be empty".to_string()));
    }
    if protocol.repetitions == 0 {
        return Err(EvalError::Invalid(
            "pilot repetitions must be positive".to_string(),
        ));
    }
    if !protocol.requires_explicit_execution_approval {
        return Err(EvalError::Invalid(
            "pilot must require explicit provider execution approval".to_string(),
        ));
    }
    let available_arms = base_suite
        .arms
        .iter()
        .map(|arm| arm.id.as_str())
        .collect::<BTreeSet<_>>();
    let available_scenarios = scenarios
        .iter()
        .map(|scenario| scenario.id.as_str())
        .collect::<BTreeSet<_>>();
    ensure_unique_known("arm", &protocol.arm_ids, &available_arms)?;
    ensure_unique_known("scenario", &protocol.scenario_ids, &available_scenarios)?;

    let expected = protocol
        .scenario_ids
        .iter()
        .flat_map(|scenario_id| {
            protocol.arm_ids.iter().flat_map(move |arm| {
                (1..=protocol.repetitions).map(move |repetition| PilotRunRef {
                    scenario_id: scenario_id.clone(),
                    arm: arm.clone(),
                    repetition,
                })
            })
        })
        .collect::<BTreeSet<_>>();
    let actual = protocol.run_order.iter().cloned().collect::<BTreeSet<_>>();
    if actual.len() != protocol.run_order.len() {
        return Err(EvalError::Invalid(
            "pilot run_order contains duplicate runs".to_string(),
        ));
    }
    if actual != expected {
        return Err(EvalError::Invalid(
            "pilot run_order must contain every selected scenario/arm/repetition exactly once"
                .to_string(),
        ));
    }
    Ok(())
}

fn ensure_unique_known(
    label: &str,
    selected: &[String],
    available: &BTreeSet<&str>,
) -> EvalResult<()> {
    if selected.is_empty() {
        return Err(EvalError::Invalid(format!(
            "pilot must select at least one {label}"
        )));
    }
    let unique = selected.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if unique.len() != selected.len() {
        return Err(EvalError::Invalid(format!(
            "pilot {label}_ids contain duplicates"
        )));
    }
    if let Some(unknown) = unique.iter().find(|value| !available.contains(**value)) {
        return Err(EvalError::Invalid(format!(
            "pilot references unknown {label}: {unknown}"
        )));
    }
    Ok(())
}

fn select_arms(
    protocol: &PilotProtocol,
    suite: &EvaluationManifest,
) -> EvalResult<Vec<EvaluationArm>> {
    protocol
        .arm_ids
        .iter()
        .map(|id| {
            suite
                .arms
                .iter()
                .find(|arm| arm.id == *id)
                .cloned()
                .ok_or_else(|| EvalError::Invalid(format!("unknown arm: {id}")))
        })
        .collect()
}

fn select_scenarios<'a>(
    protocol: &PilotProtocol,
    scenarios: &'a [Scenario],
) -> EvalResult<Vec<&'a Scenario>> {
    protocol
        .scenario_ids
        .iter()
        .map(|id| {
            scenarios
                .iter()
                .find(|scenario| scenario.id == *id)
                .ok_or_else(|| EvalError::Invalid(format!("unknown scenario: {id}")))
        })
        .collect()
}

fn write_derived_suite(
    protocol: &PilotProtocol,
    arms: Vec<EvaluationArm>,
    scenarios: &[&Scenario],
    manifest_path: &Path,
    scenario_path: &Path,
) -> EvalResult<()> {
    let manifest = EvaluationManifest {
        schema_version: 1,
        suite_id: protocol.pilot_id.clone(),
        scenario_file: "scenarios.jsonl".to_string(),
        arms,
        repetitions: protocol.repetitions,
        comparisons: Vec::new(),
    };
    fs::write(
        manifest_path,
        serde_json::to_string_pretty(&manifest)? + "\n",
    )?;
    let mut jsonl = String::new();
    for scenario in scenarios {
        jsonl.push_str(&serde_json::to_string(scenario)?);
        jsonl.push('\n');
    }
    fs::write(scenario_path, jsonl)?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn prepare_run(
    output: &Path,
    order: u32,
    run_ref: &PilotRunRef,
    arm: &EvaluationArm,
    scenario: &Scenario,
    engram: &PilotBinaryAttestation,
    codex: &PilotBinaryAttestation,
    claude: &PilotBinaryAttestation,
    schema_path: &Path,
    claude_budget_millis: u64,
) -> EvalResult<PreparedPilotRun> {
    let run_dir = output.join(format!(
        "runs/{order:02}-{}-{}",
        run_ref.scenario_id, run_ref.arm
    ));
    let fixture_root = run_dir.join("fixture");
    let layout = materialize_engineering_context_fixture(&fixture_root)?;
    let checkout_root = scenario
        .cwd
        .as_deref()
        .map(|uri| resolve_checkout_root(&layout, uri))
        .transpose()?;
    let cwd = scenario
        .cwd
        .as_deref()
        .map(|uri| resolve_fixture_uri(&layout, uri))
        .transpose()?
        .map(|path| path.display().to_string());
    let trace_path = run_dir.join("trace.jsonl");
    let agent_output_path = run_dir.join("agent-output.json");

    let mut environment = BTreeMap::new();
    let mut adapter_sha256 = None;
    let mut adapter_root = None;
    let mut engram_home = None;
    let mut engram_project = None;
    let mut data_dir = None;
    let mut cleanup_argv = None;
    let mut claude_mcp_config = None;
    if arm.uses_engram {
        let generated_adapter_root = run_dir.join("adapter");
        let checkout_root = checkout_root.as_deref().ok_or_else(|| {
            EvalError::Invalid(format!(
                "Engram pilot scenario {} requires a checkout root",
                scenario.id
            ))
        })?;
        adapter_sha256 = Some(install_eval_adapter(
            &generated_adapter_root,
            checkout_root,
            &arm.host,
        )?);
        adapter_root = Some(generated_adapter_root.canonicalize()?.display().to_string());
        let state_root = run_dir.join("engram-home");
        let project = format!("pilot-{order:02}");
        let seeded_data = state_root.join("projects").join(&project).join("data");
        seed_engineering_context_fixture(&fixture_root.join("fixture-map.json"), &seeded_data)
            .await?;
        environment.insert(
            ENGRAM_HOME_ENV.to_string(),
            state_root.canonicalize()?.display().to_string(),
        );
        engram_home = Some(state_root.canonicalize()?.display().to_string());
        engram_project = Some(project.clone());
        data_dir = Some(seeded_data.canonicalize()?.display().to_string());
        cleanup_argv = Some(vec![
            engram.path.clone(),
            "daemon".to_string(),
            "stop".to_string(),
            "--project".to_string(),
            project.clone(),
        ]);
        if arm.host == "claude_code" {
            let config_path = run_dir.join("claude-mcp.json");
            write_claude_mcp_config(&config_path, engram, &project, &environment)?;
            claude_mcp_config = Some(config_path.canonicalize()?);
        }
    }

    let argv = match arm.host.as_str() {
        "codex" => codex_argv(
            codex,
            engram,
            arm,
            scenario,
            cwd.as_deref(),
            schema_path,
            engram_project.as_deref(),
            engram_home.as_deref(),
            adapter_root.as_deref(),
        )?,
        "claude_code" => claude_argv(
            claude,
            arm,
            scenario,
            cwd.as_deref(),
            schema_path,
            claude_mcp_config.as_deref(),
            &run_dir,
            checkout_root.as_deref(),
            adapter_root.as_deref(),
            claude_budget_millis,
        )?,
        other => {
            return Err(EvalError::Invalid(format!(
                "unsupported pilot host: {other}"
            )))
        }
    };

    Ok(PreparedPilotRun {
        order,
        scenario_id: run_ref.scenario_id.clone(),
        arm: run_ref.arm.clone(),
        host: arm.host.clone(),
        repetition: run_ref.repetition,
        prompt: scenario.prompt.clone(),
        cwd,
        project: scenario.project.clone(),
        fixture_root: fixture_root.canonicalize()?.display().to_string(),
        fixture_revision: layout.fixture_revision,
        adapter_sha256,
        adapter_root,
        engram_home,
        engram_project,
        data_dir,
        environment,
        argv,
        cleanup_argv,
        trace_path: trace_path.display().to_string(),
        agent_output_path: agent_output_path.display().to_string(),
    })
}

pub(crate) fn install_eval_adapter(
    root: &Path,
    checkout_root: &Path,
    host: &str,
) -> EvalResult<String> {
    let harness = match host {
        "codex" => HarnessKind::Codex,
        "claude_code" => HarnessKind::ClaudeCode,
        other => {
            return Err(EvalError::Invalid(format!(
                "cannot render pilot adapter for host: {other}"
            )))
        }
    };
    let service = HarnessService::new();
    let adapter_name = match harness {
        HarnessKind::Codex => "codex-memory-session-skill",
        HarnessKind::ClaudeCode => "claude-memory-session-command",
        _ => unreachable!(),
    };
    let mut rendered = service.render_adapters(harness, Some(adapter_name));
    let adapter = rendered
        .pop()
        .ok_or_else(|| EvalError::Invalid(format!("missing generated adapter: {adapter_name}")))?;
    if !rendered.is_empty() {
        return Err(EvalError::Invalid(format!(
            "generated adapter name is not unique: {adapter_name}"
        )));
    }
    write_generated_file(root, &adapter.relative_path, &adapter.contents)?;

    let mut digest = Sha256::new();
    digest.update(adapter.relative_path.as_bytes());
    digest.update([0]);
    digest.update(adapter.contents.as_bytes());
    digest.update([0xff]);
    if harness == HarnessKind::ClaudeCode {
        let base_instructions = fs::read_to_string(checkout_root.join("CLAUDE.md"))?;
        let combined = format!(
            "{}\n{}",
            base_instructions.trim_end(),
            adapter.contents.trim_start()
        );
        write_generated_file(root, "claude-eval-instructions.md", &combined)?;
        digest.update(b"claude-eval-instructions.md");
        digest.update([0]);
        digest.update(combined.as_bytes());
        digest.update([0xff]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn write_generated_file(root: &Path, relative: &str, contents: &str) -> EvalResult<()> {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)?;
    Ok(())
}

pub(crate) fn write_claude_mcp_config(
    path: &Path,
    engram: &PilotBinaryAttestation,
    project: &str,
    environment: &BTreeMap<String, String>,
) -> EvalResult<()> {
    let value = serde_json::json!({
        "mcpServers": {
            "engram": {
                "type": "stdio",
                "command": engram.path,
                "args": ["serve", "--project", project, "--profile", "agent"],
                "env": environment,
            }
        }
    });
    fs::write(path, serde_json::to_string_pretty(&value)? + "\n")?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn codex_argv(
    codex: &PilotBinaryAttestation,
    engram: &PilotBinaryAttestation,
    arm: &EvaluationArm,
    scenario: &Scenario,
    cwd: Option<&str>,
    schema_path: &Path,
    engram_project: Option<&str>,
    engram_home: Option<&str>,
    adapter_root: Option<&str>,
) -> EvalResult<Vec<String>> {
    let cwd = cwd.ok_or_else(|| {
        EvalError::Invalid(format!(
            "Codex pilot scenario {} requires a cwd",
            scenario.id
        ))
    })?;
    let mut argv = vec![
        codex.path.clone(),
        "exec".to_string(),
        "--ephemeral".to_string(),
        "--ignore-user-config".to_string(),
        "--ignore-rules".to_string(),
        "--strict-config".to_string(),
        "--sandbox".to_string(),
        "read-only".to_string(),
        "--cd".to_string(),
        cwd.to_string(),
        "--json".to_string(),
        "--output-schema".to_string(),
        schema_path.canonicalize()?.display().to_string(),
        "--config".to_string(),
        "features.memories=false".to_string(),
        "--config".to_string(),
        "feedback.enabled=false".to_string(),
    ];
    if arm.uses_engram {
        let project = engram_project.ok_or_else(|| {
            EvalError::Invalid("Engram Codex arm is missing daemon project".to_string())
        })?;
        let state_root = engram_home.ok_or_else(|| {
            EvalError::Invalid("Engram Codex arm is missing isolated state root".to_string())
        })?;
        let adapter_root = adapter_root.ok_or_else(|| {
            EvalError::Invalid("Engram Codex arm is missing generated adapter".to_string())
        })?;
        let skill_path = Path::new(adapter_root)
            .join(".codex/skills/engram-memory-session/SKILL.md")
            .canonicalize()?;
        let developer_instructions = fs::read_to_string(&skill_path)?;
        let args = vec!["serve", "--project", project, "--profile", "agent"];
        argv.extend([
            "--config".to_string(),
            format!(
                "mcp_servers.engram.command={}",
                serde_json::to_string(&engram.path)?
            ),
            "--config".to_string(),
            format!("mcp_servers.engram.args={}", serde_json::to_string(&args)?),
            "--config".to_string(),
            format!(
                "mcp_servers.engram.env={{ENGRAM_HOME={}}}",
                serde_json::to_string(state_root)?
            ),
            "--config".to_string(),
            "mcp_servers.engram.required=true".to_string(),
            "--config".to_string(),
            "mcp_servers.engram.startup_timeout_sec=60".to_string(),
            "--config".to_string(),
            format!(
                "skills.config=[{{path={},enabled=true}}]",
                serde_json::to_string(&skill_path.display().to_string())?
            ),
            "--config".to_string(),
            format!(
                "developer_instructions={}",
                serde_json::to_string(&developer_instructions)?
            ),
        ]);
    }
    argv.push(scenario.prompt.clone());
    Ok(argv)
}

#[allow(clippy::too_many_arguments)]
fn claude_argv(
    claude: &PilotBinaryAttestation,
    arm: &EvaluationArm,
    scenario: &Scenario,
    cwd: Option<&str>,
    schema_path: &Path,
    mcp_config: Option<&Path>,
    run_dir: &Path,
    checkout_root: Option<&Path>,
    adapter_root: Option<&str>,
    budget_millis: u64,
) -> EvalResult<Vec<String>> {
    let cwd = cwd.ok_or_else(|| {
        EvalError::Invalid(format!(
            "Claude pilot scenario {} requires a cwd",
            scenario.id
        ))
    })?;
    let schema = fs::read_to_string(schema_path)?;
    let checkout_root = checkout_root.ok_or_else(|| {
        EvalError::Invalid(format!(
            "Claude pilot scenario {} requires a checkout root",
            scenario.id
        ))
    })?;
    let empty_mcp_path = run_dir.join("claude-mcp-empty.json");
    if !arm.uses_engram {
        fs::write(&empty_mcp_path, "{\"mcpServers\":{}}\n")?;
    }
    let selected_mcp = if arm.uses_engram {
        mcp_config.ok_or_else(|| {
            EvalError::Invalid("Engram Claude arm is missing MCP config".to_string())
        })?
    } else {
        empty_mcp_path.as_path()
    };
    let mut argv = vec![
        claude.path.clone(),
        "--print".to_string(),
        "--bare".to_string(),
        "--no-session-persistence".to_string(),
        "--output-format".to_string(),
        "stream-json".to_string(),
        "--verbose".to_string(),
        "--permission-mode".to_string(),
        "dontAsk".to_string(),
        "--tools".to_string(),
        "Read,Bash".to_string(),
        "--disallowed-tools".to_string(),
        "Write,Edit,WebFetch,WebSearch,NotebookEdit,Task".to_string(),
        "--strict-mcp-config".to_string(),
        "--mcp-config".to_string(),
        selected_mcp.canonicalize()?.display().to_string(),
        "--json-schema".to_string(),
        schema,
        "--max-budget-usd".to_string(),
        format_budget_usd(budget_millis),
        "--add-dir".to_string(),
        cwd.to_string(),
    ];
    let instruction_file = if arm.uses_engram {
        Path::new(adapter_root.ok_or_else(|| {
            EvalError::Invalid("Engram Claude arm is missing generated adapter".to_string())
        })?)
        .join("claude-eval-instructions.md")
    } else {
        checkout_root.join("CLAUDE.md")
    };
    argv.extend([
        "--append-system-prompt-file".to_string(),
        instruction_file.canonicalize()?.display().to_string(),
    ]);
    argv.push(scenario.prompt.clone());
    Ok(argv)
}

pub(crate) fn format_budget_usd(millis: u64) -> String {
    format!("{}.{:03}", millis / 1000, millis % 1000)
}

pub(crate) fn resolve_fixture_uri(layout: &FixtureLayout, uri: &str) -> EvalResult<PathBuf> {
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

pub(crate) fn resolve_checkout_root(layout: &FixtureLayout, uri: &str) -> EvalResult<PathBuf> {
    layout
        .checkouts
        .iter()
        .filter(|(prefix, _)| uri == prefix.as_str() || uri.starts_with(&format!("{prefix}/")))
        .max_by_key(|(prefix, _)| prefix.len())
        .map(|(_, root)| PathBuf::from(root))
        .ok_or_else(|| EvalError::Invalid(format!("unmapped fixture checkout URI: {uri}")))
}

pub(crate) fn attest_binary(path: &Path) -> EvalResult<PilotBinaryAttestation> {
    let resolved = resolve_executable(path)?;
    let output = Command::new(&resolved)
        .arg("--version")
        .output()
        .map_err(|error| {
            EvalError::Invalid(format!(
                "failed to run {} --version: {error}",
                resolved.display()
            ))
        })?;
    if !output.status.success() {
        return Err(EvalError::Invalid(format!(
            "{} --version exited with {}",
            resolved.display(),
            output.status
        )));
    }
    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if version.is_empty() {
        return Err(EvalError::Invalid(format!(
            "{} --version returned no stdout",
            resolved.display()
        )));
    }
    let mut hasher = Sha256::new();
    hasher.update(fs::read(&resolved)?);
    Ok(PilotBinaryAttestation {
        path: resolved.display().to_string(),
        version,
        sha256: format!("{:x}", hasher.finalize()),
    })
}

/// Attest the Codex code-mode companion resolved relative to the Codex executable.
///
/// The companion does not implement `--version`, so its compatibility identity is tied to the
/// already-attested Codex version while its own executable bytes remain independently hashed.
pub(crate) fn attest_codex_code_mode_host(
    codex: &PilotBinaryAttestation,
) -> EvalResult<PilotBinaryAttestation> {
    let codex_path = Path::new(&codex.path);
    let parent = codex_path.parent().ok_or_else(|| {
        EvalError::Invalid("attested Codex executable has no parent directory".to_string())
    })?;
    let candidate = parent.join(CODEX_CODE_MODE_HOST_NAME);
    let metadata = fs::symlink_metadata(&candidate).map_err(|error| {
        EvalError::Invalid(format!(
            "missing Codex code-mode companion {}: {error}",
            candidate.display()
        ))
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(EvalError::Invalid(format!(
            "Codex code-mode companion must be a regular non-symlink file: {}",
            candidate.display()
        )));
    }
    let resolved = candidate.canonicalize()?;
    let output = Command::new(&resolved)
        .arg("--help")
        .output()
        .map_err(|error| {
            EvalError::Invalid(format!(
                "failed to run {} --help: {error}",
                resolved.display()
            ))
        })?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() || !stdout.contains("Usage: codex-code-mode-host") {
        return Err(EvalError::Invalid(format!(
            "{} --help did not identify the Codex code-mode host",
            resolved.display()
        )));
    }
    let mut hasher = Sha256::new();
    hasher.update(fs::read(&resolved)?);
    Ok(PilotBinaryAttestation {
        path: resolved.display().to_string(),
        version: format!("companion for {}", codex.version),
        sha256: format!("{:x}", hasher.finalize()),
    })
}

fn resolve_executable(path: &Path) -> EvalResult<PathBuf> {
    if path.is_absolute() || path.components().count() > 1 {
        return path.canonicalize().map_err(EvalError::Io);
    }
    let search_path = std::env::var_os("PATH")
        .ok_or_else(|| EvalError::Invalid("PATH is not set".to_string()))?;
    for directory in std::env::split_paths(&search_path) {
        let candidate = directory.join(path);
        if candidate.is_file() {
            return candidate.canonicalize().map_err(EvalError::Io);
        }
    }
    Err(EvalError::Invalid(format!(
        "executable was not found on PATH: {}",
        path.display()
    )))
}

fn resolve_relative(owner: &Path, referenced: &str) -> EvalResult<PathBuf> {
    let path = Path::new(referenced);
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        owner.parent().unwrap_or_else(|| Path::new(".")).join(path)
    };
    resolved.canonicalize().map_err(EvalError::Io)
}

pub(crate) fn create_private_dir_all(path: &Path) -> EvalResult<()> {
    fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

pub(crate) fn require_empty_target(path: &Path) -> EvalResult<()> {
    if path.exists() && fs::read_dir(path)?.next().transpose()?.is_some() {
        return Err(EvalError::Invalid(format!(
            "pilot target is not empty: {}",
            path.display()
        )));
    }
    create_private_dir_all(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn protocol() -> PilotProtocol {
        PilotProtocol {
            schema_version: 1,
            pilot_id: "pilot".to_string(),
            base_manifest: "unused".to_string(),
            scenario_ids: vec!["scenario".to_string()],
            arm_ids: vec!["arm".to_string()],
            repetitions: 1,
            run_order: vec![PilotRunRef {
                scenario_id: "scenario".to_string(),
                arm: "arm".to_string(),
                repetition: 1,
            }],
            claude_budget_cents: 25,
            requires_explicit_execution_approval: true,
        }
    }

    fn suite() -> (EvaluationManifest, Vec<Scenario>) {
        let manifest = EvaluationManifest {
            schema_version: 1,
            suite_id: "base".to_string(),
            scenario_file: "scenarios.jsonl".to_string(),
            arms: vec![EvaluationArm {
                id: "arm".to_string(),
                description: "arm".to_string(),
                host: "codex".to_string(),
                uses_engram: false,
            }],
            repetitions: 3,
            comparisons: Vec::new(),
        };
        let scenario = Scenario {
            id: "scenario".to_string(),
            journey: crate::Journey::RepositoryIdentity,
            prompt: "Orient".to_string(),
            cwd: Some("fixture://atlas/main".to_string()),
            project: None,
            expected: Default::default(),
            budgets: Default::default(),
            notes: None,
        };
        (manifest, vec![scenario])
    }

    #[test]
    fn protocol_requires_a_complete_unique_matrix_and_execution_gate() {
        let (suite, scenarios) = suite();
        validate_protocol(&protocol(), &suite, &scenarios).unwrap();

        let mut duplicate = protocol();
        duplicate.run_order.push(duplicate.run_order[0].clone());
        assert!(validate_protocol(&duplicate, &suite, &scenarios)
            .unwrap_err()
            .to_string()
            .contains("duplicate"));

        let mut ungated = protocol();
        ungated.requires_explicit_execution_approval = false;
        assert!(validate_protocol(&ungated, &suite, &scenarios)
            .unwrap_err()
            .to_string()
            .contains("explicit provider execution approval"));
    }

    #[test]
    fn budget_formatter_is_exact_to_a_thousandth_of_a_dollar() {
        assert_eq!(format_budget_usd(62), "0.062");
        assert_eq!(format_budget_usd(1_250), "1.250");
    }

    #[test]
    fn output_schema_is_a_portable_json_schema_subset() {
        let value: serde_json::Value = serde_json::from_str(AGENT_OUTPUT_SCHEMA).unwrap();
        assert!(value.get("$schema").is_none());
        assert_eq!(value["additionalProperties"], false);
    }

    #[cfg(unix)]
    #[test]
    fn codex_code_mode_companion_is_sibling_attested_without_version_flag() {
        use std::os::unix::fs::PermissionsExt;

        let root = tempfile::tempdir().unwrap();
        let codex_path = root.path().join("codex");
        fs::write(&codex_path, "codex").unwrap();
        let host_path = root.path().join(CODEX_CODE_MODE_HOST_NAME);
        fs::write(
            &host_path,
            "#!/bin/sh\nif [ \"$1\" = \"--help\" ]; then echo 'Usage: codex-code-mode-host [OPTIONS]'; exit 0; fi\nexit 64\n",
        )
        .unwrap();
        fs::set_permissions(&host_path, fs::Permissions::from_mode(0o700)).unwrap();
        let codex = PilotBinaryAttestation {
            path: codex_path.display().to_string(),
            version: "codex-cli test".to_string(),
            sha256: "a".repeat(64),
        };

        let attestation = attest_codex_code_mode_host(&codex).unwrap();
        assert_eq!(
            attestation.path,
            host_path.canonicalize().unwrap().display().to_string()
        );
        assert_eq!(attestation.version, "companion for codex-cli test");
        assert_eq!(attestation.sha256.len(), 64);

        fs::remove_file(&host_path).unwrap();
        assert!(attest_codex_code_mode_host(&codex)
            .unwrap_err()
            .to_string()
            .contains("missing Codex code-mode companion"));
    }

    #[cfg(unix)]
    #[test]
    fn pilot_target_defaults_to_private_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let parent = tempfile::tempdir().unwrap();
        let target = parent.path().join("pilot");
        require_empty_target(&target).unwrap();

        assert_eq!(
            fs::metadata(target).unwrap().permissions().mode() & 0o777,
            0o700
        );
    }
}
