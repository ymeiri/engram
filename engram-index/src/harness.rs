//! Agent harness policy rendering and installation.
//!
//! This module is deliberately filesystem-only. It does not persist state in the
//! Engram database; instead it renders a stable policy and verifies local adapter
//! files that make Engram's lifecycle visible to agent surfaces.

use crate::error::{IndexError, IndexResult};
use engram_core::harness::{
    HarnessAdapterCheck, HarnessAdapterKind, HarnessAdapterSpec, HarnessAdapterStatus,
    HarnessEnforcementProfile, HarnessHostCheck, HarnessInstallFile, HarnessInstallReport,
    HarnessKind, HarnessLifecycleReport, HarnessLifecycleTrigger, HarnessMcpServerCheck,
    HarnessMcpToolReport, HarnessPolicy, HarnessRenderedAdapter, HarnessSettingsCheck,
    HarnessStatusReport,
};
use engram_core::memory::{
    ClaimOrigin, EvidenceKind, EvidenceRef, Harness, MemoryItem, MemoryKind, MemoryScope,
    ModelIdentity, WriterProvenance,
};
use engram_core::obligation::{
    AgentObligation, AgentObligationKind, AgentObligationResolution, AgentObligationResolutionKind,
    AgentObligationTrigger,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

const MARKER_MD: &str = "<!-- engram:harness-adapter:v1 -->";
const MARKER_SH: &str = "# engram:harness-adapter:v1";
const CLAUDE_HOOK_COMMAND: &str = concat!(
    "start_dir=\"${CLAUDE_PROJECT_DIR:-$PWD}\"; ",
    "project_root=\"$(git -C \"$start_dir\" rev-parse --show-toplevel 2>/dev/null || true)\"; ",
    "if [ -z \"$project_root\" ]; then project_root=\"$start_dir\"; fi; ",
    "project_hook=\"${project_root:+$project_root/.claude/hooks/engram-session-start.sh}\"; ",
    "home_hook=\"${HOME:-}/.claude/hooks/engram-session-start.sh\"; ",
    "if [ -n \"$project_hook\" ] && [ -f \"$project_hook\" ]; then exec /usr/bin/env bash \"$project_hook\"; fi; ",
    "if [ -n \"${HOME:-}\" ] && [ -f \"$home_hook\" ]; then exec /usr/bin/env bash \"$home_hook\"; fi; ",
    "printf '%s\\n' 'Engram SessionStart hook skipped: generated hook script was not found at the repository root or under HOME.' >&2; ",
    "exit 0"
);
const CLAUDE_SESSION_END_HOOK_COMMAND: &str = concat!(
    "start_dir=\"${CLAUDE_PROJECT_DIR:-$PWD}\"; ",
    "project_root=\"$(git -C \"$start_dir\" rev-parse --show-toplevel 2>/dev/null || true)\"; ",
    "if [ -z \"$project_root\" ]; then project_root=\"$start_dir\"; fi; ",
    "project_hook=\"${project_root:+$project_root/.claude/hooks/engram-session-end.sh}\"; ",
    "home_hook=\"${HOME:-}/.claude/hooks/engram-session-end.sh\"; ",
    "if [ -n \"$project_hook\" ] && [ -f \"$project_hook\" ]; then exec /usr/bin/env bash \"$project_hook\"; fi; ",
    "if [ -n \"${HOME:-}\" ] && [ -f \"$home_hook\" ]; then exec /usr/bin/env bash \"$home_hook\"; fi; ",
    "printf '%s\\n' 'Engram SessionEnd hook skipped: generated hook script was not found at the repository root or under HOME.' >&2; ",
    "exit 0"
);
const CLAUDE_LEGACY_HOOK_COMMAND: &str =
    "\"${CLAUDE_PROJECT_DIR:-.}/.claude/hooks/engram-session-start.sh\"";
const CLAUDE_LEGACY_SESSION_END_HOOK_COMMAND: &str =
    "\"${CLAUDE_PROJECT_DIR:-.}/.claude/hooks/engram-session-end.sh\"";
const CLAUDE_EFFECTIVE_HOOK_VERIFICATION_WARNING: &str = concat!(
    "Claude Code static readiness confirms generated adapter files and settings entries; ",
    "it does not prove live effective hook visibility. Verify effective hook configuration with ",
    "Claude Code /hooks before claiming native Claude hook behavior."
);
const CODEX_EFFECTIVE_ADAPTER_VERIFICATION_WARNING: &str = concat!(
    "Codex static readiness confirms generated skill and hook files on disk; it does not prove ",
    "the project is trusted, the hook hash is accepted, or that an already-running Codex task ",
    "loaded those files. ",
    "Start a fresh Codex task and inspect /hooks before claiming effective adapter behavior."
);
const CODEX_HOOK_COMMAND: &str = concat!(
    "project_root=\"$(git rev-parse --show-toplevel 2>/dev/null || true)\"; ",
    "project_hook=\"${project_root:+$project_root/.codex/hooks/engram-session-start.sh}\"; ",
    "home_hook=\"${CODEX_HOME:-${HOME:-}/.codex}/hooks/engram-session-start.sh\"; ",
    "if [ -n \"$project_hook\" ] && [ -f \"$project_hook\" ]; then exec /usr/bin/env bash \"$project_hook\"; fi; ",
    "if [ -f \"$home_hook\" ]; then exec /usr/bin/env bash \"$home_hook\"; fi; ",
    "printf '%s\\n' 'Engram Codex SessionStart hook skipped: generated hook script was not found at the repository root or under CODEX_HOME.' >&2; ",
    "exit 0"
);

/// Options for harness adapter installation.
#[derive(Debug, Clone, Copy, Default)]
pub struct HarnessInstallOptions {
    /// Actually write generated adapters and settings changes.
    pub write: bool,
    /// Back up and replace user-owned adapter files.
    pub adopt_user_owned: bool,
    /// Claude Code settings target for generated permissions and hooks.
    pub settings_target: HarnessSettingsTarget,
    /// Lifecycle enforcement profile to encode in generated policy and adapters.
    pub enforcement_profile: HarnessEnforcementProfile,
}

/// Claude Code settings target for harness installation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HarnessSettingsTarget {
    /// Merge into project-shared `.claude/settings.json`.
    #[default]
    Project,
    /// Merge into local, gitignored `.claude/settings.local.json`.
    Local,
    /// Do not merge settings; generate the snippet only.
    SnippetOnly,
}

impl HarnessSettingsTarget {
    /// Parse a settings target.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.to_lowercase().replace('-', "_").as_str() {
            "settings.json" | "settings_json" | "project" | "project_settings" => {
                Ok(Self::Project)
            }
            "settings.local.json" | "settings_local_json" | "local" | "local_settings" => {
                Ok(Self::Local)
            }
            "snippet" | "snippet_only" | "none" => Ok(Self::SnippetOnly),
            _ => Err(format!(
                "invalid settings target '{value}'; expected settings.json, settings.local.json, or snippet-only"
            )),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Project => "settings.json",
            Self::Local => "settings.local.json",
            Self::SnippetOnly => "snippet-only",
        }
    }

    fn path(self, root: &Path) -> Option<PathBuf> {
        match self {
            Self::Project => Some(claude_project_settings_path(root)),
            Self::Local => Some(claude_local_settings_path(root)),
            Self::SnippetOnly => None,
        }
    }
}

impl std::fmt::Display for HarnessSettingsTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// A Claude Code hook event routed through the Engram harness.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HarnessHookEvent {
    /// Harness handling the event.
    pub harness: HarnessKind,
    /// Lifecycle enforcement profile.
    pub enforcement_profile: HarnessEnforcementProfile,
    /// Claude Code hook event name.
    pub hook_event_name: String,
    /// Claude Code session identifier.
    pub session_id: Option<String>,
    /// Current working directory.
    pub cwd: Option<String>,
    /// Transcript path.
    pub transcript_path: Option<String>,
    /// Submitted user prompt.
    pub prompt: Option<String>,
    /// Tool name for tool hooks.
    pub tool_name: Option<String>,
    /// Tool failure error.
    pub tool_error: Option<String>,
    /// Tool input command, when available.
    pub tool_input_command: Option<String>,
    /// Engram tool action, when available.
    pub tool_input_action: Option<String>,
    /// File path touched by a tool, when available.
    pub file_path: Option<String>,
    /// Last assistant message for stop hooks.
    pub last_assistant_message: Option<String>,
    /// Compaction summary.
    pub compact_summary: Option<String>,
    /// Compaction trigger or hook matcher.
    pub trigger: Option<String>,
    /// Session end reason or permission reason.
    pub reason: Option<String>,
    /// Whether Claude is already continuing because of a Stop hook.
    pub stop_hook_active: bool,
    /// Write policy, normally "durable" or "nudge".
    pub write_policy: Option<String>,
    /// Project scope override.
    pub project: Option<String>,
    /// Model provider for writer provenance.
    pub model_provider: Option<String>,
    /// Model name for writer provenance.
    pub model: Option<String>,
    /// Surface label for writer provenance.
    pub surface: Option<String>,
    /// Actor label for writer provenance.
    pub actor: Option<String>,
}

/// Services used by hook-event handling. All are optional so hook handling stays soft.
pub struct HarnessHookServices<'a> {
    /// Memory service.
    pub memory: Option<&'a crate::memory::MemoryService>,
    /// Obligation service.
    pub obligations: Option<&'a crate::obligation::ObligationService>,
    /// Handoff service.
    pub handoff: Option<&'a crate::handoff::HandoffService>,
}

/// Result of handling a Claude Code hook event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessHookEventOutcome {
    /// Claude hook JSON response.
    pub response: Value,
    /// Human-readable additional context.
    pub additional_context: String,
    /// Memory items written.
    pub memory_written: usize,
    /// Obligations written.
    pub obligations_written: usize,
    /// Whether a handoff was written.
    pub handoff_written: bool,
    /// Whether the hook blocked the current stop; Claude adapters currently keep this false.
    pub blocked: bool,
    /// Non-fatal warnings.
    pub warnings: Vec<String>,
}

/// Stateless service for rendering and verifying harness integration adapters.
#[derive(Debug, Default, Clone, Copy)]
pub struct HarnessService;

impl HarnessService {
    /// Create a harness service.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Return the canonical policy for a harness.
    #[must_use]
    pub fn policy(&self, harness: HarnessKind) -> HarnessPolicy {
        self.policy_with_enforcement(harness, HarnessEnforcementProfile::default())
    }

    /// Return the canonical policy for a harness and enforcement profile.
    #[must_use]
    pub fn policy_with_enforcement(
        &self,
        harness: HarnessKind,
        enforcement_profile: HarnessEnforcementProfile,
    ) -> HarnessPolicy {
        HarnessPolicy {
            harness,
            enforcement_profile,
            soft_contract: enforcement_profile.is_soft(),
            lifecycle_triggers: lifecycle_triggers(),
            required_mcp_tools: required_mcp_tools(),
            adapters: adapters_for(harness, enforcement_profile),
        }
    }

    /// Return status for a harness at an install root.
    pub fn status(
        &self,
        harness: HarnessKind,
        root: Option<&Path>,
        observed_mcp_tools: &[String],
    ) -> IndexResult<HarnessStatusReport> {
        self.status_with_enforcement(
            harness,
            root,
            observed_mcp_tools,
            HarnessEnforcementProfile::default(),
        )
    }

    /// Return status for a harness at an install root with an enforcement profile.
    pub fn status_with_enforcement(
        &self,
        harness: HarnessKind,
        root: Option<&Path>,
        observed_mcp_tools: &[String],
        enforcement_profile: HarnessEnforcementProfile,
    ) -> IndexResult<HarnessStatusReport> {
        let root = resolve_root(root)?;
        let policy = self.policy_with_enforcement(harness, enforcement_profile);
        let adapters: Vec<_> = policy
            .adapters
            .iter()
            .map(|adapter| check_adapter(&root, adapter))
            .collect::<IndexResult<_>>()?;
        let mcp_tools_checked = !observed_mcp_tools.is_empty();
        let missing_mcp_tools: Vec<String> = policy
            .required_mcp_tools
            .iter()
            .filter(|tool| {
                mcp_tools_checked && !observed_mcp_tools.iter().any(|name| name == *tool)
            })
            .cloned()
            .collect();
        let mcp_tools = HarnessMcpToolReport {
            checked: mcp_tools_checked,
            required_tools: policy.required_mcp_tools.clone(),
            observed_tools: observed_mcp_tools.to_vec(),
            missing_tools: missing_mcp_tools.clone(),
            message: if !mcp_tools_checked {
                "MCP tool availability was not checked; provide observed_mcp_tools to verify the required tool set.".to_string()
            } else if missing_mcp_tools.is_empty() {
                "All required MCP tools were observed.".to_string()
            } else {
                format!(
                    "Missing required MCP tools: {}.",
                    missing_mcp_tools.join(", ")
                )
            },
        };

        let mut warnings = Vec::new();
        for check in &adapters {
            match check.status {
                HarnessAdapterStatus::Missing if check.required => {
                    warnings.push(format!(
                        "Required adapter '{}' is missing at {}.",
                        check.name, check.path
                    ));
                }
                HarnessAdapterStatus::Drifted => {
                    warnings.push(format!(
                        "Adapter '{}' has an Engram marker but differs from current generated content.",
                        check.name
                    ));
                }
                HarnessAdapterStatus::UserOwned => {
                    warnings.push(format!(
                        "Adapter path for '{}' exists but is user-owned; Engram will not overwrite it.",
                        check.name
                    ));
                }
                HarnessAdapterStatus::Installed | HarnessAdapterStatus::Missing => {}
            }
        }
        for tool in &missing_mcp_tools {
            warnings.push(format!(
                "Required MCP tool '{tool}' was not reported by the client."
            ));
        }

        let mut ready = adapters
            .iter()
            .filter(|check| check.required)
            .all(|check| check.status == HarnessAdapterStatus::Installed)
            && missing_mcp_tools.is_empty();

        let mut settings = Vec::new();
        if harness == HarnessKind::ClaudeCode {
            let settings_status = claude_settings_status(&root, policy.enforcement_profile)?;
            if settings_status.has_missing_required() {
                ready = false;
            }
            warnings.extend(settings_status.warnings);
            settings = settings_status.checks;
            warn_for_installed_claude_hook_files_without_settings(
                &adapters,
                &settings,
                &mut warnings,
            );
            if ready {
                warnings.push(CLAUDE_EFFECTIVE_HOOK_VERIFICATION_WARNING.to_string());
            }
        } else if harness == HarnessKind::Codex && ready {
            warnings.push(CODEX_EFFECTIVE_ADAPTER_VERIFICATION_WARNING.to_string());
        }

        let host = check_harness_host(harness);
        if host.checked && host.executable_path.is_none() {
            warnings.push(host.message.clone());
        }

        Ok(HarnessStatusReport {
            harness,
            root: root.display().to_string(),
            lifecycle: lifecycle_report(&policy),
            policy,
            adapters,
            missing_mcp_tools,
            mcp_tools,
            mcp_server: unchecked_mcp_server(),
            settings,
            host,
            warnings,
            ready,
        })
    }

    /// Add an explicit, read-only host MCP configuration attestation to a status report.
    ///
    /// Codex is queried through its native JSON configuration resolver. Claude Code does not
    /// provide a non-connecting query, so its known config sources are inspected statically and
    /// the weaker evidence boundary is preserved in the result.
    pub fn attest_host_configuration(
        &self,
        report: &mut HarnessStatusReport,
        cwd: Option<&Path>,
    ) -> IndexResult<()> {
        let cwd = match cwd {
            Some(cwd) => cwd.to_path_buf(),
            None => std::env::current_dir()?,
        };
        let check = match report.harness {
            HarnessKind::Codex => codex_mcp_server_check(&report.host),
            HarnessKind::ClaudeCode => claude_mcp_server_check(&cwd)?,
            _ => unsupported_mcp_server_check(report.harness),
        };

        if check.checked && !check.agent_profile_launch_configured {
            report.ready = false;
            report.warnings.push(check.message.clone());
        }
        report.mcp_server = check;
        Ok(())
    }

    /// Doctor currently extends status with soft lifecycle warnings.
    pub fn doctor(
        &self,
        harness: HarnessKind,
        root: Option<&Path>,
        observed_mcp_tools: &[String],
    ) -> IndexResult<HarnessStatusReport> {
        self.doctor_with_enforcement(
            harness,
            root,
            observed_mcp_tools,
            HarnessEnforcementProfile::default(),
        )
    }

    /// Run harness diagnostics with an enforcement profile.
    pub fn doctor_with_enforcement(
        &self,
        harness: HarnessKind,
        root: Option<&Path>,
        observed_mcp_tools: &[String],
        enforcement_profile: HarnessEnforcementProfile,
    ) -> IndexResult<HarnessStatusReport> {
        let mut report =
            self.status_with_enforcement(harness, root, observed_mcp_tools, enforcement_profile)?;
        if report.ready {
            let triggers = report
                .lifecycle
                .advisory_triggers
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");
            report
                .warnings
                .push(format!(
                    "Harness adapter files are present; enforcement_profile={} and lifecycle triggers are: {triggers}.",
                    report.lifecycle.enforcement_profile
                ));
        } else {
            report.warnings.push(
                "Harness is not fully installed; agents may still use Engram manually through MCP."
                    .to_string(),
            );
        }
        Ok(report)
    }

    /// Render the policy as pretty JSON.
    pub fn render_policy(&self, harness: HarnessKind) -> IndexResult<String> {
        self.render_policy_with_enforcement(harness, HarnessEnforcementProfile::default())
    }

    /// Render the policy as pretty JSON with an enforcement profile.
    pub fn render_policy_with_enforcement(
        &self,
        harness: HarnessKind,
        enforcement_profile: HarnessEnforcementProfile,
    ) -> IndexResult<String> {
        serde_json::to_string_pretty(&self.policy_with_enforcement(harness, enforcement_profile))
            .map_err(|e| IndexError::Parse(format!("failed to render harness policy: {e}")))
    }

    /// Render one adapter or all adapters.
    #[must_use]
    pub fn render_adapters(
        &self,
        harness: HarnessKind,
        adapter_name: Option<&str>,
    ) -> Vec<HarnessRenderedAdapter> {
        self.render_adapters_with_enforcement(
            harness,
            adapter_name,
            HarnessEnforcementProfile::default(),
        )
    }

    /// Render one adapter or all adapters with an enforcement profile.
    #[must_use]
    pub fn render_adapters_with_enforcement(
        &self,
        harness: HarnessKind,
        adapter_name: Option<&str>,
        enforcement_profile: HarnessEnforcementProfile,
    ) -> Vec<HarnessRenderedAdapter> {
        self.policy_with_enforcement(harness, enforcement_profile)
            .adapters
            .into_iter()
            .filter(|adapter| {
                adapter_name
                    .map(|name| adapter.name.eq_ignore_ascii_case(name))
                    .unwrap_or(true)
            })
            .map(|adapter| HarnessRenderedAdapter {
                name: adapter.name,
                kind: adapter.kind,
                relative_path: adapter.relative_path,
                contents: adapter.contents,
            })
            .collect()
    }

    /// Install harness adapters. Dry-run mode is the default and writes nothing.
    pub fn install(
        &self,
        harness: HarnessKind,
        root: Option<&Path>,
        write: bool,
    ) -> IndexResult<HarnessInstallReport> {
        self.install_with_options(
            harness,
            root,
            HarnessInstallOptions {
                write,
                adopt_user_owned: false,
                settings_target: HarnessSettingsTarget::default(),
                enforcement_profile: HarnessEnforcementProfile::default(),
            },
        )
    }

    /// Install harness adapters with explicit safety options.
    pub fn install_with_options(
        &self,
        harness: HarnessKind,
        root: Option<&Path>,
        options: HarnessInstallOptions,
    ) -> IndexResult<HarnessInstallReport> {
        let root = resolve_root(root)?;
        let mut planned = Vec::new();
        let mut written = Vec::new();
        let mut skipped = Vec::new();
        let mut warnings = Vec::new();

        for adapter in self
            .policy_with_enforcement(harness, options.enforcement_profile)
            .adapters
        {
            let path = root.join(&adapter.relative_path);
            let check = check_adapter(&root, &adapter)?;
            match check.status {
                HarnessAdapterStatus::Missing | HarnessAdapterStatus::Drifted => {
                    planned.push(HarnessInstallFile {
                        name: adapter.name.clone(),
                        path: path.display().to_string(),
                        written: false,
                        message: install_plan_message(check.status),
                    });
                    if !options.write {
                        continue;
                    }

                    if let Some(parent) = path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::write(&path, adapter.contents.as_bytes())?;
                    set_executable_if_hook(&path, adapter.kind)?;
                    written.push(HarnessInstallFile {
                        name: adapter.name,
                        path: path.display().to_string(),
                        written: true,
                        message: "written".to_string(),
                    });
                }
                HarnessAdapterStatus::Installed => {
                    skipped.push(HarnessInstallFile {
                        name: adapter.name,
                        path: path.display().to_string(),
                        written: false,
                        message: "already installed".to_string(),
                    });
                }
                HarnessAdapterStatus::UserOwned => {
                    if options.adopt_user_owned {
                        let message =
                            "will back up and adopt user-owned file without Engram marker"
                                .to_string();
                        planned.push(HarnessInstallFile {
                            name: adapter.name.clone(),
                            path: path.display().to_string(),
                            written: false,
                            message,
                        });
                        if !options.write {
                            continue;
                        }

                        let backup = backup_path(&path);
                        fs::copy(&path, &backup)?;
                        fs::write(&path, adapter.contents.as_bytes())?;
                        set_executable_if_hook(&path, adapter.kind)?;
                        written.push(HarnessInstallFile {
                            name: adapter.name,
                            path: path.display().to_string(),
                            written: true,
                            message: format!(
                                "adopted user-owned file; backup={}",
                                backup.display()
                            ),
                        });
                    } else {
                        let message = "skipped user-owned file without Engram marker".to_string();
                        warnings.push(format!("{}: {}", path.display(), message));
                        skipped.push(HarnessInstallFile {
                            name: adapter.name,
                            path: path.display().to_string(),
                            written: false,
                            message,
                        });
                    }
                }
            }
        }

        if harness == HarnessKind::ClaudeCode {
            merge_claude_settings(
                &root,
                options.settings_target,
                options.write,
                options.enforcement_profile,
                &mut planned,
                &mut written,
                &mut skipped,
                &mut warnings,
            )?;
        }

        Ok(HarnessInstallReport {
            harness,
            enforcement_profile: options.enforcement_profile,
            root: root.display().to_string(),
            dry_run: !options.write,
            planned,
            written,
            skipped,
            warnings,
        })
    }

    /// Handle a Claude Code hook event and return valid Claude hook JSON.
    pub async fn handle_hook_event(
        &self,
        event: HarnessHookEvent,
        services: HarnessHookServices<'_>,
    ) -> IndexResult<HarnessHookEventOutcome> {
        let mut warnings = Vec::new();
        let mut memory_written = 0;
        let mut obligations_written = 0;
        let mut handoff_written = false;
        let write_durable = event
            .write_policy
            .as_deref()
            .map(|policy| policy.eq_ignore_ascii_case("durable"))
            .unwrap_or(false);
        let project = event.project.clone();
        let writer = hook_writer(&event);
        let enforce_runtime = event.harness == HarnessKind::ClaudeCode
            && event.enforcement_profile != HarnessEnforcementProfile::Soft;
        if enforce_runtime
            && services.obligations.is_none()
            && matches!(
                normalized_event_name(&event.hook_event_name).as_str(),
                "userpromptsubmit" | "pretooluse" | "posttooluse" | "stop"
            )
        {
            warnings.push(
                "Engram runtime enforcement degraded: obligation service is unavailable."
                    .to_string(),
            );
        }

        match normalized_event_name(&event.hook_event_name).as_str() {
            "userpromptsubmit" => {
                if let Some(service) = services.obligations {
                    if enforce_runtime && is_substantive_prompt(event.prompt.as_deref()) {
                        match ensure_orientation_obligation(
                            service,
                            &event,
                            project.as_deref(),
                            &writer,
                        )
                        .await
                        {
                            Ok(written) => {
                                if written {
                                    obligations_written += 1;
                                }
                            }
                            Err(error) => warnings
                                .push(format!("orientation obligation write failed: {error}")),
                        }
                    }
                    let detection = service
                        .detect(crate::obligation::ObligationDetectOptions {
                            cwd: event.cwd.clone(),
                            prompt: event.prompt.clone(),
                            project: project.clone(),
                            writer: writer.clone(),
                            write: write_durable,
                            limit: Some(16),
                        })
                        .await;
                    match detection {
                        Ok(detection) => {
                            obligations_written += detection.written.len();
                            warnings.extend(detection.warnings);
                        }
                        Err(error) => {
                            warnings.push(format!("obligation detection failed: {error}"))
                        }
                    }
                }
                if write_durable {
                    if let (Some(service), Some(item)) = (
                        services.memory,
                        explicit_user_memory_from_prompt(&event, &writer),
                    ) {
                        match service.capture_memory(item).await {
                            Ok(_) => memory_written += 1,
                            Err(error) => warnings.push(format!("memory capture failed: {error}")),
                        }
                    }
                }
            }
            "pretooluse" => {}
            "posttooluse" => {
                if let Some(service) = services.obligations {
                    if enforce_runtime && is_orientation_boundary_tool(&event) {
                        match resolve_orientation_obligations(
                            service,
                            project.as_deref(),
                            event.cwd.as_deref(),
                            event
                                .tool_name
                                .as_deref()
                                .unwrap_or("engram_orientation_boundary"),
                        )
                        .await
                        {
                            Ok(0) => {}
                            Ok(count) => warnings.push(format!(
                                "resolved {count} Engram orientation obligation(s)."
                            )),
                            Err(error) => warnings
                                .push(format!("orientation obligation resolve failed: {error}")),
                        }
                    }
                    let detection = service
                        .detect(crate::obligation::ObligationDetectOptions {
                            cwd: event.cwd.clone(),
                            prompt: tool_prompt(&event),
                            project: project.clone(),
                            writer: writer.clone(),
                            write: write_durable,
                            limit: Some(16),
                        })
                        .await;
                    match detection {
                        Ok(detection) => {
                            obligations_written += detection.written.len();
                            warnings.extend(detection.warnings);
                        }
                        Err(error) => {
                            warnings.push(format!("obligation detection failed: {error}"))
                        }
                    }
                }
                if write_durable {
                    if let (Some(service), Some(item)) =
                        (services.memory, document_memory_from_tool(&event, &writer))
                    {
                        match service.capture_memory(item).await {
                            Ok(_) => memory_written += 1,
                            Err(error) => warnings.push(format!("memory capture failed: {error}")),
                        }
                    }
                }
            }
            "posttoolusefailure" => {
                if let Some(service) = services.obligations {
                    let detection = service
                        .detect(crate::obligation::ObligationDetectOptions {
                            cwd: event.cwd.clone(),
                            prompt: tool_failure_prompt(&event),
                            project: project.clone(),
                            writer: writer.clone(),
                            write: write_durable,
                            limit: Some(16),
                        })
                        .await;
                    match detection {
                        Ok(detection) => {
                            obligations_written += detection.written.len();
                            warnings.extend(detection.warnings);
                        }
                        Err(error) => {
                            warnings.push(format!("obligation detection failed: {error}"))
                        }
                    }
                }
                if write_durable {
                    if let Some(service) = services.memory {
                        let item = tool_failure_memory(&event, &writer);
                        match service.capture_memory(item).await {
                            Ok(_) => memory_written += 1,
                            Err(error) => warnings.push(format!("memory capture failed: {error}")),
                        }
                    }
                }
            }
            "precompact" => {
                if write_durable {
                    if let (Some(service), Some(project)) = (services.handoff, project.clone()) {
                        let content = format!(
                            "# Claude Code Pre-Compact Handoff\n\nSession: {}\nCWD: {}\nTrigger: {}\nTranscript: {}\n\n## Next Actions\n- Resume by calling orient and using scoped search only if its compact context is insufficient.\n",
                            event.session_id.as_deref().unwrap_or("unknown"),
                            event.cwd.as_deref().unwrap_or("unknown"),
                            event.trigger.as_deref().unwrap_or("unknown"),
                            event.transcript_path.as_deref().unwrap_or("unknown"),
                        );
                        match service
                            .update(
                                Some(project),
                                None,
                                content,
                                vec!["Resume by calling orient and inspecting scoped context."
                                    .to_string()],
                                writer.clone(),
                                false,
                            )
                            .await
                        {
                            Ok(update) => handoff_written = update.written,
                            Err(error) => warnings.push(format!("handoff update failed: {error}")),
                        }
                    } else if services.handoff.is_some() {
                        warnings.push(
                            "handoff update skipped: no canonical project was supplied or resolved"
                                .to_string(),
                        );
                    }
                }
            }
            "postcompact" => {
                if write_durable {
                    if let (Some(service), Some(summary), Some(project)) = (
                        services.handoff,
                        event.compact_summary.clone(),
                        project.clone(),
                    ) {
                        let content = format!(
                            "# Claude Code Post-Compact Summary\n\n{}\n\n## Next Actions\n- Continue with orient and use scoped search only if its compact context is insufficient.\n",
                            summary.trim()
                        );
                        match service
                            .update(
                                Some(project),
                                None,
                                content,
                                vec!["Continue with orient and scoped context after compaction."
                                    .to_string()],
                                writer.clone(),
                                false,
                            )
                            .await
                        {
                            Ok(update) => handoff_written = update.written,
                            Err(error) => warnings.push(format!("handoff update failed: {error}")),
                        }
                    } else if services.handoff.is_some()
                        && event.compact_summary.is_some()
                        && project.is_none()
                    {
                        warnings.push(
                            "handoff update skipped: no canonical project was supplied or resolved"
                                .to_string(),
                        );
                    }
                }
            }
            "stop" => {
                if let Some(service) = services.obligations {
                    let detection = service
                        .detect(crate::obligation::ObligationDetectOptions {
                            cwd: event.cwd.clone(),
                            prompt: None,
                            project: project.clone(),
                            writer: writer.clone(),
                            write: write_durable && !event.stop_hook_active,
                            limit: Some(16),
                        })
                        .await;
                    match detection {
                        Ok(detection) => {
                            obligations_written += detection.written.len();
                            warnings.extend(detection.warnings);
                        }
                        Err(error) => {
                            warnings.push(format!("obligation detection failed: {error}"))
                        }
                    }
                }
            }
            "sessionend" if write_durable => {
                if let (Some(service), Some(project)) = (services.handoff, project.clone()) {
                    let content = format!(
                        "# Claude Code Session-End Handoff\n\nSession: {}\nCWD: {}\nReason: {}\nTranscript: {}\n\n## Next Actions\n- On resume, call orient and inspect this handoff before acting.\n",
                        event.session_id.as_deref().unwrap_or("unknown"),
                        event.cwd.as_deref().unwrap_or("unknown"),
                        event.reason.as_deref().unwrap_or("unknown"),
                        event.transcript_path.as_deref().unwrap_or("unknown"),
                    );
                    match service
                        .update(
                            Some(project),
                            None,
                            content,
                            vec!["On resume, call orient and inspect this handoff.".to_string()],
                            writer.clone(),
                            false,
                        )
                        .await
                    {
                        Ok(update) => handoff_written = update.written,
                        Err(error) => warnings.push(format!("handoff update failed: {error}")),
                    }
                } else if services.handoff.is_some() {
                    warnings.push(
                        "handoff update skipped: no canonical project was supplied or resolved"
                            .to_string(),
                    );
                }
            }
            _ => {}
        }

        let open_obligations = if let Some(service) = services.obligations {
            match service
                .list_open_for_context(project.as_deref(), event.cwd.as_deref())
                .await
            {
                Ok(obligations) => obligations,
                Err(error) => {
                    warnings.push(format!("obligation doctor failed: {error}"));
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };
        if enforce_runtime
            && event.enforcement_profile == HarnessEnforcementProfile::Graduated
            && normalized_event_name(&event.hook_event_name) == "stop"
            && event.stop_hook_active
            && !open_obligations.is_empty()
        {
            warnings.push(format!(
                "Engram graduated Stop allowed exit because stop_hook_active=true; {} open obligation(s) remain.",
                open_obligations.len()
            ));
        }

        let additional_context = hook_additional_context(
            &event,
            memory_written,
            obligations_written,
            handoff_written,
            open_obligations.len(),
            &warnings,
        );
        let enforcement =
            claude_runtime_enforcement_response(&event, &open_obligations, &additional_context);
        let blocked = enforcement.is_some();
        let response =
            enforcement.unwrap_or_else(|| claude_hook_response(&event, &additional_context));

        Ok(HarnessHookEventOutcome {
            response,
            additional_context,
            memory_written,
            obligations_written,
            handoff_written,
            blocked,
            warnings,
        })
    }
}

fn lifecycle_report(policy: &HarnessPolicy) -> HarnessLifecycleReport {
    let runtime_enforced = policy.harness == HarnessKind::ClaudeCode
        && policy.enforcement_profile != HarnessEnforcementProfile::Soft;
    let message = match (policy.harness, policy.enforcement_profile) {
        (HarnessKind::ClaudeCode, HarnessEnforcementProfile::Soft) => {
            "Lifecycle compliance is advisory and low-overhead: Claude Code installs session, compaction, and session-end hooks; in-session Engram use is driven by generated commands and agent instructions."
        }
        (HarnessKind::Codex, HarnessEnforcementProfile::Soft) => {
            "Lifecycle compliance is advisory and low-overhead: Codex installs a deterministic SessionStart context hook plus generated skills; the hook does not block host actions."
        }
        (_, HarnessEnforcementProfile::Soft) => {
            "Lifecycle compliance is advisory and low-overhead; Engram use is driven by generated commands, skills, and agent instructions."
        }
        (HarnessKind::ClaudeCode, HarnessEnforcementProfile::Graduated) => {
            "Lifecycle compliance uses graduated enforcement: task-start orientation and final obligations are enforced at high-value Claude Code hook boundaries."
        }
        (_, HarnessEnforcementProfile::Graduated) => {
            "The graduated lifecycle is mandatory agent guidance, but this host adapter does not provide runtime blocking."
        }
        (HarnessKind::ClaudeCode, HarnessEnforcementProfile::Strict) => {
            "Lifecycle compliance is strict: Claude Code hooks keep blocking until required obligations are resolved or explicitly skipped."
        }
        (_, HarnessEnforcementProfile::Strict) => {
            "The strict lifecycle is mandatory agent guidance, but this host adapter does not provide runtime blocking."
        }
    };

    HarnessLifecycleReport {
        enforcement_profile: policy.enforcement_profile,
        soft_contract: policy.enforcement_profile.is_soft(),
        enforced: runtime_enforced,
        advisory_triggers: policy.lifecycle_triggers.clone(),
        message: message.to_string(),
    }
}

fn lifecycle_triggers() -> Vec<HarnessLifecycleTrigger> {
    vec![
        HarnessLifecycleTrigger::TaskStartOrient,
        HarnessLifecycleTrigger::BeforeMajorDecisionChangesSince,
        HarnessLifecycleTrigger::AfterDiscoveryRecord,
        HarnessLifecycleTrigger::BeforeFinalChangesSince,
        HarnessLifecycleTrigger::BeforeFinalObligations,
        HarnessLifecycleTrigger::BeforeContextCompactionSave,
        HarnessLifecycleTrigger::SessionEndHandoff,
        HarnessLifecycleTrigger::CommitWorkflowConsultMemory,
    ]
}

fn normalized_event_name(event: &str) -> String {
    event
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>()
        .to_lowercase()
}

fn hook_writer(event: &HarnessHookEvent) -> WriterProvenance {
    let harness = match event.harness {
        HarnessKind::ClaudeCode => Harness::ClaudeCode,
        HarnessKind::Codex => Harness::Codex,
        HarnessKind::Cursor => Harness::Cursor,
        HarnessKind::GeminiCli => Harness::Other("gemini_cli".to_string()),
        HarnessKind::Generic => Harness::Other("generic".to_string()),
    };
    let model = ModelIdentity::new(
        event.model_provider.as_deref().unwrap_or("anthropic"),
        event.model.as_deref().unwrap_or("claude-code"),
    );
    let mut writer = WriterProvenance::agent(harness, model);
    writer.surface = Some(
        event
            .surface
            .clone()
            .unwrap_or_else(|| "claude-code".to_string()),
    );
    writer.actor = event.actor.clone().unwrap_or_else(|| "agent".to_string());
    writer
}

fn hook_scope(event: &HarnessHookEvent) -> MemoryScope {
    event
        .project
        .clone()
        .map(MemoryScope::project)
        .unwrap_or_else(|| {
            event
                .cwd
                .clone()
                .map(|cwd| MemoryScope::Custom {
                    name: format!("cwd:{cwd}"),
                })
                .unwrap_or(MemoryScope::Global)
        })
}

fn is_substantive_prompt(prompt: Option<&str>) -> bool {
    let Some(prompt) = prompt.map(str::trim).filter(|prompt| !prompt.is_empty()) else {
        return false;
    };
    let lower = prompt.to_lowercase();
    !matches!(
        lower.as_str(),
        "ok" | "okay" | "yes" | "no" | "thanks" | "thank you" | "continue" | "go on"
    )
}

async fn ensure_orientation_obligation(
    service: &crate::obligation::ObligationService,
    event: &HarnessHookEvent,
    project: Option<&str>,
    writer: &WriterProvenance,
) -> IndexResult<bool> {
    let open = service
        .list_open_for_context(project, event.cwd.as_deref())
        .await?;
    if open.iter().any(is_orientation_obligation) {
        return Ok(false);
    }

    let mut obligation = AgentObligation::new(
        AgentObligationKind::EngramOrientation,
        "Run Engram orientation before non-Engram tool use",
        "Claude Code must complete either lean orient or local memory(action=procedure_match) before using non-Engram tools for this task.",
        obligation_scope(project, event.cwd.as_deref()),
        AgentObligationTrigger::new("user_prompt", "substantive task prompt submitted")
            .with_target("mcp__engram__orient"),
        writer.clone(),
    )
    .with_required_resolution(AgentObligationResolutionKind::EngramOriented)
    .with_required_resolution(AgentObligationResolutionKind::SkippedWithReason)
    .with_tag("engram-orientation")
    .with_tag("claude-code")
    .with_tag("runtime-enforcement");

    if let Some(prompt) = event.prompt.as_deref().map(str::trim) {
        obligation = obligation.with_evidence(
            EvidenceRef::new(EvidenceKind::ManualReview, "claude_user_prompt")
                .with_summary("Claude Code UserPromptSubmit hook observed a substantive prompt")
                .with_excerpt(prompt.chars().take(1200).collect::<String>()),
        );
    }

    service.add(obligation).await?;
    Ok(true)
}

async fn resolve_orientation_obligations(
    service: &crate::obligation::ObligationService,
    project: Option<&str>,
    cwd: Option<&str>,
    boundary_tool: &str,
) -> IndexResult<usize> {
    let open = service.list_open_for_context(project, cwd).await?;
    let mut count = 0;
    for obligation in open.into_iter().filter(is_orientation_obligation) {
        let resolution = AgentObligationResolution::new(
            AgentObligationResolutionKind::EngramOriented,
            format!("{boundary_tool} completed the Engram identity boundary for this task."),
            "engram-harness",
        )
        .with_evidence(
            EvidenceRef::new(EvidenceKind::ToolCall, boundary_tool)
                .with_summary("Claude Code PostToolUse hook observed an Engram identity boundary"),
        );
        service.resolve(obligation.id, resolution).await?;
        count += 1;
    }
    Ok(count)
}

fn obligation_scope(project: Option<&str>, cwd: Option<&str>) -> MemoryScope {
    if let Some(project) = project {
        MemoryScope::project(project)
    } else if let Some(cwd) = cwd {
        MemoryScope::Custom {
            name: format!("cwd:{cwd}"),
        }
    } else {
        MemoryScope::Global
    }
}

fn is_orientation_obligation(obligation: &AgentObligation) -> bool {
    obligation.kind == AgentObligationKind::EngramOrientation
}

fn is_orientation_boundary_tool(event: &HarnessHookEvent) -> bool {
    match event.tool_name.as_deref() {
        Some(tool) if tool.eq_ignore_ascii_case("mcp__engram__orient") => true,
        Some(tool) if tool.eq_ignore_ascii_case("mcp__engram__memory") => event
            .tool_input_action
            .as_deref()
            .is_some_and(|action| action.eq_ignore_ascii_case("procedure_match")),
        _ => false,
    }
}

fn is_engram_tool(event: &HarnessHookEvent) -> bool {
    event
        .tool_name
        .as_deref()
        .is_some_and(|tool| tool.starts_with("mcp__engram__"))
}

fn claude_runtime_enforcement_response(
    event: &HarnessHookEvent,
    open_obligations: &[AgentObligation],
    additional_context: &str,
) -> Option<Value> {
    if event.harness != HarnessKind::ClaudeCode
        || event.enforcement_profile == HarnessEnforcementProfile::Soft
    {
        return None;
    }

    match normalized_event_name(&event.hook_event_name).as_str() {
        "pretooluse" if !is_engram_tool(event) => {
            let orientation = open_obligations.iter().find(|obligation| {
                is_orientation_obligation(obligation)
                    && obligation
                        .trigger
                        .target
                        .as_deref()
                        .is_some_and(|target| target == "mcp__engram__orient")
            })?;
            let tool = event.tool_name.as_deref().unwrap_or("unknown tool");
            let reason = format!(
                "Engram {} enforcement denied `{tool}` because task-start orientation is still open (obligation {}). For actionable repository work, call `mcp__engram__memory` with `action=\"procedure_match\"`, a bounded task-focused query copied from the user's operation request, and current cwd; otherwise call `mcp__engram__orient` with `agent=\"claude_code\"`, current project/cwd/prompt, and `response_shape=\"lean\"`. Then retry the tool.",
                event.enforcement_profile,
                orientation.id
            );
            Some(json!({
                "continue": true,
                "hookSpecificOutput": {
                    "hookEventName": event.hook_event_name.trim(),
                    "permissionDecision": "deny",
                    "permissionDecisionReason": reason
                },
                "systemMessage": additional_context
            }))
        }
        "stop" if !open_obligations.is_empty() => {
            if event.enforcement_profile == HarnessEnforcementProfile::Graduated
                && event.stop_hook_active
            {
                return None;
            }
            let reason = stop_block_reason(event, open_obligations);
            Some(json!({
                "decision": "block",
                "reason": reason,
                "systemMessage": additional_context
            }))
        }
        _ => None,
    }
}

fn stop_block_reason(event: &HarnessHookEvent, open_obligations: &[AgentObligation]) -> String {
    let doctor_args = if let Some(project) = event.project.as_deref() {
        let mut scope = vec![
            "relevance_mode:\"related\"".to_string(),
            format!(
                "project:{}",
                serde_json::to_string(project).expect("project string should serialize")
            ),
        ];
        if let Some(cwd) = event.cwd.as_deref() {
            scope.push(format!(
                "cwd:{}",
                serde_json::to_string(cwd).expect("cwd string should serialize")
            ));
        }
        format!("scope={{{}}}", scope.join(", "))
    } else {
        let cwd_filter = event
            .cwd
            .as_deref()
            .map(|cwd| {
                format!(
                    ", cwd={}",
                    serde_json::to_string(cwd).expect("cwd string should serialize")
                )
            })
            .unwrap_or_default();
        format!("scope={{relevance_mode:\"global\"}}{cwd_filter}")
    };
    let first = &open_obligations[0];
    format!(
        "Engram {} enforcement blocked the final response because {} open obligation(s) remain. First open obligation: {} ({}, id={}). Run `obligations(action=doctor, {})`, then either `obligations(action=resolve, id=\"{}\", resolution=\"{}\", summary=\"...\", actor=\"agent\")` or `obligations(action=skip, id=\"{}\", reason=\"...\", actor=\"agent\")`. Then answer again.",
        event.enforcement_profile,
        open_obligations.len(),
        first.title,
        first.kind,
        first.id,
        doctor_args,
        first.id,
        first
            .required_resolution
            .first()
            .map(ToString::to_string)
            .unwrap_or_else(|| "memory_recorded".to_string()),
        first.id
    )
}

fn tool_prompt(event: &HarnessHookEvent) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(tool) = &event.tool_name {
        parts.push(format!("Tool used: {tool}."));
    }
    if let Some(command) = &event.tool_input_command {
        if !command.trim().is_empty() {
            parts.push(format!("Command: {command}"));
        }
    }
    if let Some(path) = &event.file_path {
        if is_durable_doc_path(path) {
            parts.push(format!("Durable document changed: {path}"));
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n"))
    }
}

fn tool_failure_prompt(event: &HarnessHookEvent) -> Option<String> {
    let mut prompt = String::from("A failed tool call occurred and needs recovery.");
    if let Some(tool) = &event.tool_name {
        prompt.push_str(&format!(" Tool: {tool}."));
    }
    if let Some(error) = &event.tool_error {
        prompt.push_str(&format!(" Error: {error}."));
    }
    Some(prompt)
}

fn explicit_user_memory_from_prompt(
    event: &HarnessHookEvent,
    writer: &WriterProvenance,
) -> Option<MemoryItem> {
    let prompt = event.prompt.as_deref()?.trim();
    if prompt.is_empty() {
        return None;
    }
    let lower = prompt.to_lowercase();
    let kind = if contains_any(
        &lower,
        &[
            "remember",
            "my preference",
            "i prefer",
            "i don't like",
            "i do not like",
        ],
    ) {
        MemoryKind::Preference
    } else if contains_any(&lower, &["from now on", "always ", "never "]) {
        MemoryKind::Rule
    } else {
        return None;
    };

    Some(
        MemoryItem::new(
            kind,
            "Claude Code user-stated instruction",
            prompt.chars().take(1200).collect::<String>(),
            hook_scope(event),
            ClaimOrigin::UserStated,
            writer.clone(),
        )
        .with_evidence(
            EvidenceRef::new(EvidenceKind::ManualReview, "claude_user_prompt")
                .with_summary("Explicit user prompt captured by Claude Code UserPromptSubmit hook"),
        )
        .with_tag("claude-code")
        .with_tag("hook-event")
        .with_tag("user-stated"),
    )
}

fn document_memory_from_tool(
    event: &HarnessHookEvent,
    writer: &WriterProvenance,
) -> Option<MemoryItem> {
    let path = event.file_path.as_deref()?.trim();
    if !is_durable_doc_path(path) {
        return None;
    }
    let absolute = event
        .cwd
        .as_deref()
        .map(|cwd| absolutize(Path::new(cwd), path))
        .unwrap_or_else(|| PathBuf::from(path));
    Some(
        MemoryItem::new(
            MemoryKind::SessionInsight,
            format!("Claude Code durable document changed: {path}"),
            format!(
                "Claude Code edited or wrote durable document `{path}`. The agent must index, register, record, handoff-link, or explicitly skip this document before claiming the task is complete."
            ),
            hook_scope(event),
            ClaimOrigin::ToolResult,
            writer.clone(),
        )
        .with_evidence(
            EvidenceRef::new(EvidenceKind::File, absolute.to_string_lossy())
                .with_summary("Claude Code PostToolUse hook observed durable document change"),
        )
        .with_tag("claude-code")
        .with_tag("document-disposition")
        .with_tag("hook-event"),
    )
}

fn tool_failure_memory(event: &HarnessHookEvent, writer: &WriterProvenance) -> MemoryItem {
    let tool_name = event.tool_name.as_deref().unwrap_or("unknown-tool");
    let error = event.tool_error.as_deref().unwrap_or("unknown error");
    MemoryItem::new(
        MemoryKind::SessionInsight,
        format!("Claude Code tool failure: {tool_name}"),
        format!(
            "Claude Code observed a failed tool call for `{tool_name}`. Error: {error}. The agent should inspect the schema/help, retry correctly if the action still matters, abandon explicitly if it does not, and record reusable gotchas when non-obvious."
        ),
        hook_scope(event),
        ClaimOrigin::ToolResult,
        writer.clone(),
    )
    .with_evidence(
        EvidenceRef::new(EvidenceKind::ToolCall, tool_name)
            .with_summary("Claude Code PostToolUseFailure hook")
            .with_excerpt(error.chars().take(1200).collect::<String>()),
    )
    .with_tag("claude-code")
    .with_tag("tool-failure")
    .with_tag("hook-event")
}

fn hook_additional_context(
    event: &HarnessHookEvent,
    memory_written: usize,
    obligations_written: usize,
    handoff_written: bool,
    open_obligations: usize,
    warnings: &[String],
) -> String {
    let mut lines = vec![format!(
        "<engram_hook event=\"{}\" write_policy=\"{}\">",
        event.hook_event_name,
        event.write_policy.as_deref().unwrap_or("nudge")
    )];
    lines.push(format!(
        "Engram captured lifecycle state: memory_written={memory_written}, obligations_written={obligations_written}, handoff_written={handoff_written}, open_obligations={open_obligations}."
    ));
    match normalized_event_name(&event.hook_event_name).as_str() {
        "userpromptsubmit" => lines.push(
            "Before acting, call orient if this is a new task, keep returned trace_id values from orient/search, and use obligations for source/design, document, failed-tool, verification, handoff, and commit-preference checks."
                .to_string(),
        ),
        "posttoolusefailure" => lines.push(
            "A tool failed. Inspect the tool schema/help before retrying; record reusable gotchas when non-obvious."
                .to_string(),
        ),
        "posttooluse" => lines.push(
            "If a durable document changed, resolve its document disposition before final response."
                .to_string(),
        ),
        "stop" => lines.push(
            match event.enforcement_profile {
                HarnessEnforcementProfile::Soft => {
                    "Engram already ran final document-obligation detection for changed durable docs. Resolve or explicitly skip open obligations without blocking the user, and rerun obligations(action=detect, project=..., cwd=...) if more files change."
                        .to_string()
                }
                HarnessEnforcementProfile::Graduated => {
                    "Engram ran final obligation detection. Graduated enforcement blocks this Stop once when open obligations remain; resolve or explicitly skip open obligations with obligations(action=resolve/skip), rerun obligations(action=detect, project=..., cwd=...) if more files change, then answer. If Claude is already in stop_hook_active recovery, report the remaining obligations and exit."
                        .to_string()
                }
                HarnessEnforcementProfile::Strict => {
                    "Engram ran final obligation detection. Strict enforcement blocks Stop while open obligations remain; resolve or explicitly skip them with obligations(action=resolve/skip), rerun obligations(action=detect, project=..., cwd=...) if more files change, then answer."
                        .to_string()
                }
            },
        ),
        "precompact" | "postcompact" => lines.push(
            "Before relying on compacted context, call orient again and use scoped search only if its compact context is insufficient."
                .to_string(),
        ),
        "sessionend" => lines.push(
            "The session ended; the next session should resume by calling orient."
                .to_string(),
        ),
        _ => {}
    }
    if !warnings.is_empty() {
        lines.push(format!("Warnings: {}", warnings.join("; ")));
    }
    lines.push("</engram_hook>".to_string());
    lines.join("\n")
}

fn claude_hook_response(event: &HarnessHookEvent, additional_context: &str) -> Value {
    let event_name = event.hook_event_name.trim();
    if !event_supports_hook_specific_output(event_name) {
        return json!({
            "continue": true,
            "systemMessage": additional_context
        });
    }

    json!({
        "continue": true,
        "hookSpecificOutput": {
            "hookEventName": event_name,
            "additionalContext": additional_context
        }
    })
}

fn event_supports_hook_specific_output(event_name: &str) -> bool {
    matches!(
        normalized_event_name(event_name).as_str(),
        "pretooluse" | "userpromptsubmit" | "posttooluse" | "posttoolbatch"
    )
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn is_durable_doc_path(path: &str) -> bool {
    let lower = path.to_lowercase();
    let extension = Path::new(&lower)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    if !matches!(extension, "md" | "markdown" | "mdx" | "rst" | "txt") {
        return false;
    }

    extension == "md"
        || lower.contains("/docs/")
        || lower.starts_with("docs/")
        || lower.contains("guide")
        || lower.contains("plan")
        || lower.contains("design")
        || lower.contains("runbook")
        || lower.contains("adr")
        || lower.contains("eval")
        || lower.ends_with("readme.md")
}

fn absolutize(cwd: &Path, path: &str) -> PathBuf {
    let path = Path::new(path);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}

fn required_mcp_tools() -> Vec<String> {
    [
        "orient",
        "memory",
        "repo",
        "search",
        "harness",
        "obligations",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn adapters_for(
    harness: HarnessKind,
    enforcement_profile: HarnessEnforcementProfile,
) -> Vec<HarnessAdapterSpec> {
    match harness {
        HarnessKind::ClaudeCode => claude_adapters(enforcement_profile),
        HarnessKind::Codex => codex_adapters(enforcement_profile),
        HarnessKind::GeminiCli => gemini_adapters(enforcement_profile),
        HarnessKind::Cursor => cursor_adapters(enforcement_profile),
        HarnessKind::Generic => generic_adapters(enforcement_profile),
    }
}

fn claude_adapters(enforcement_profile: HarnessEnforcementProfile) -> Vec<HarnessAdapterSpec> {
    vec![
        adapter(
            "claude-memory-session-command",
            HarnessAdapterKind::ClaudeCommand,
            ".claude/commands/engram-memory-session.md",
            "Claude command that states the Memory OS lifecycle contract.",
            true,
            claude_memory_session_command(enforcement_profile),
        ),
        adapter(
            "claude-resume-session-command",
            HarnessAdapterKind::ClaudeCommand,
            ".claude/commands/engram-resume-session.md",
            "Claude command for resuming with orient and handoff context.",
            true,
            claude_resume_session_command(),
        ),
        adapter(
            "claude-end-session-command",
            HarnessAdapterKind::ClaudeCommand,
            ".claude/commands/engram-end-session.md",
            "Claude command for handoff compilation and knowledge commit candidates.",
            true,
            claude_end_session_command(),
        ),
        adapter(
            "claude-session-start-hook",
            HarnessAdapterKind::ClaudeHook,
            ".claude/hooks/engram-session-start.sh",
            "Claude hook nudge for session/task start orientation.",
            true,
            claude_session_start_hook(enforcement_profile),
        ),
        adapter(
            "claude-stop-nudge-hook",
            HarnessAdapterKind::ClaudeHook,
            ".claude/hooks/engram-stop-nudge.sh",
            "Claude hook nudge before stopping/final response.",
            true,
            claude_stop_nudge_hook(enforcement_profile),
        ),
        adapter(
            "claude-session-end-hook",
            HarnessAdapterKind::ClaudeHook,
            ".claude/hooks/engram-session-end.sh",
            "Claude command hook for session-end handoff when MCP tool hooks are unavailable.",
            true,
            claude_session_end_hook(enforcement_profile),
        ),
        adapter(
            "claude-settings-snippet",
            HarnessAdapterKind::PolicyDocument,
            ".claude/engram-settings-snippet.json",
            "Claude Code settings snippet for Engram MCP permissions and lifecycle hooks.",
            false,
            claude_settings_snippet(enforcement_profile),
        ),
        adapter(
            "project-agents-snippet",
            HarnessAdapterKind::ProjectInstructions,
            "AGENTS.engram.md",
            "Project instruction snippet that can be merged into AGENTS.md.",
            false,
            agents_snippet(enforcement_profile),
        ),
    ]
}

fn codex_adapters(enforcement_profile: HarnessEnforcementProfile) -> Vec<HarnessAdapterSpec> {
    vec![
        adapter(
            "codex-session-start-hook",
            HarnessAdapterKind::CodexHook,
            ".codex/hooks/engram-session-start.sh",
            "Codex SessionStart hook that injects deterministic, advisory Engram startup context.",
            true,
            codex_session_start_hook(enforcement_profile),
        ),
        adapter(
            "codex-hooks-config",
            HarnessAdapterKind::PolicyDocument,
            ".codex/hooks.json",
            "Codex lifecycle hook configuration for deterministic Engram startup context.",
            true,
            codex_hooks_config(),
        ),
        adapter(
            "codex-memory-session-skill",
            HarnessAdapterKind::CodexSkill,
            ".codex/skills/engram-memory-session/SKILL.md",
            "Codex skill for the Memory OS lifecycle contract.",
            true,
            codex_memory_session_skill(enforcement_profile),
        ),
        adapter(
            "codex-resume-session-skill",
            HarnessAdapterKind::CodexSkill,
            ".codex/skills/engram-resume-session/SKILL.md",
            "Codex skill for project/session resumption from Engram.",
            true,
            codex_resume_session_skill(),
        ),
        adapter(
            "project-agents-snippet",
            HarnessAdapterKind::ProjectInstructions,
            "AGENTS.engram.md",
            "Project instruction snippet that can be merged into AGENTS.md.",
            false,
            agents_snippet(enforcement_profile),
        ),
    ]
}

fn gemini_adapters(enforcement_profile: HarnessEnforcementProfile) -> Vec<HarnessAdapterSpec> {
    vec![
        adapter(
            "gemini-memory-session-command",
            HarnessAdapterKind::GeminiCommand,
            ".gemini/commands/engram/memory-session.toml",
            "Gemini CLI custom command for the Memory OS lifecycle contract.",
            true,
            gemini_memory_session_command(enforcement_profile),
        ),
        adapter(
            "gemini-resume-session-command",
            HarnessAdapterKind::GeminiCommand,
            ".gemini/commands/engram/resume-session.toml",
            "Gemini CLI custom command for project/session resumption from Engram.",
            true,
            gemini_resume_session_command(),
        ),
        adapter(
            "gemini-end-session-command",
            HarnessAdapterKind::GeminiCommand,
            ".gemini/commands/engram/end-session.toml",
            "Gemini CLI custom command for handoff compilation and memory commits.",
            true,
            gemini_end_session_command(),
        ),
        adapter(
            "gemini-global-context",
            HarnessAdapterKind::GeminiContext,
            ".gemini/GEMINI.md",
            "Gemini CLI global context file for Memory OS lifecycle nudges.",
            true,
            gemini_global_context(enforcement_profile),
        ),
    ]
}

fn cursor_adapters(enforcement_profile: HarnessEnforcementProfile) -> Vec<HarnessAdapterSpec> {
    vec![
        adapter(
            "cursor-memory-session-skill",
            HarnessAdapterKind::CursorSkill,
            ".cursor/skills/engram-memory-session/SKILL.md",
            "Cursor Agent skill for the Memory OS lifecycle contract.",
            true,
            cursor_memory_session_skill(enforcement_profile),
        ),
        adapter(
            "cursor-resume-session-skill",
            HarnessAdapterKind::CursorSkill,
            ".cursor/skills/engram-resume-session/SKILL.md",
            "Cursor Agent skill for project/session resumption from Engram.",
            true,
            cursor_resume_session_skill(),
        ),
        adapter(
            "cursor-end-session-skill",
            HarnessAdapterKind::CursorSkill,
            ".cursor/skills/engram-end-session/SKILL.md",
            "Cursor Agent skill for handoff compilation and memory commits.",
            true,
            cursor_end_session_skill(),
        ),
    ]
}

fn generic_adapters(enforcement_profile: HarnessEnforcementProfile) -> Vec<HarnessAdapterSpec> {
    vec![adapter(
        "generic-harness-policy",
        HarnessAdapterKind::PolicyDocument,
        ".engram/harness-policy.md",
        "Generic Memory OS harness lifecycle policy.",
        true,
        generic_policy_document(enforcement_profile),
    )]
}

fn adapter(
    name: &str,
    kind: HarnessAdapterKind,
    relative_path: &str,
    description: &str,
    required: bool,
    contents: String,
) -> HarnessAdapterSpec {
    HarnessAdapterSpec {
        name: name.to_string(),
        kind,
        relative_path: relative_path.to_string(),
        description: description.to_string(),
        required,
        contents,
    }
}

fn check_adapter(root: &Path, adapter: &HarnessAdapterSpec) -> IndexResult<HarnessAdapterCheck> {
    let path = root.join(&adapter.relative_path);
    let expected_sha256 = sha256_bytes(adapter.contents.as_bytes());
    let (status, actual_sha256, message) = match fs::read_to_string(&path) {
        Ok(existing) => {
            let actual_sha256 = Some(sha256_bytes(existing.as_bytes()));
            if existing == adapter.contents {
                (
                    HarnessAdapterStatus::Installed,
                    actual_sha256,
                    "generated adapter is installed".to_string(),
                )
            } else if has_marker(&existing) {
                (
                    HarnessAdapterStatus::Drifted,
                    actual_sha256,
                    "generated adapter has drifted from current policy".to_string(),
                )
            } else {
                (
                    HarnessAdapterStatus::UserOwned,
                    actual_sha256,
                    "file exists without Engram generated marker".to_string(),
                )
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (
            HarnessAdapterStatus::Missing,
            None,
            "adapter is missing".to_string(),
        ),
        Err(error) => return Err(error.into()),
    };

    Ok(HarnessAdapterCheck {
        name: adapter.name.clone(),
        kind: adapter.kind,
        path: path.display().to_string(),
        status,
        required: adapter.required,
        expected_sha256,
        actual_sha256,
        message,
    })
}

fn unchecked_mcp_server() -> HarnessMcpServerCheck {
    HarnessMcpServerCheck {
        checked: false,
        evidence_kind: None,
        source: None,
        server_found: false,
        enabled: None,
        transport: None,
        command: None,
        args: Vec::new(),
        env_keys: Vec::new(),
        executable_path: None,
        executable_sha256: None,
        agent_profile_launch_configured: false,
        resolved_configuration_verified: false,
        running_host_loaded_verified: false,
        live_runtime_verified: false,
        message: "Host MCP configuration was not attested; request explicit host configuration attestation to inspect it read-only.".to_string(),
    }
}

fn unsupported_mcp_server_check(harness: HarnessKind) -> HarnessMcpServerCheck {
    HarnessMcpServerCheck {
        checked: true,
        message: format!(
            "No first-class read-only MCP configuration attestation is defined for {harness}."
        ),
        ..unchecked_mcp_server()
    }
}

fn codex_mcp_server_check(host: &HarnessHostCheck) -> HarnessMcpServerCheck {
    let Some(codex) = host.executable_path.as_deref() else {
        return HarnessMcpServerCheck {
            checked: true,
            evidence_kind: Some("host_cli_resolved".to_string()),
            message: "Codex MCP configuration could not be resolved because the Codex executable was not found.".to_string(),
            ..unchecked_mcp_server()
        };
    };

    let output = match Command::new(codex)
        .args(["mcp", "get", "engram", "--json"])
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            return HarnessMcpServerCheck {
                checked: true,
                evidence_kind: Some("host_cli_resolved".to_string()),
                message: format!("Codex MCP configuration query failed: {error}"),
                ..unchecked_mcp_server()
            };
        }
    };
    if !output.status.success() {
        return HarnessMcpServerCheck {
            checked: true,
            evidence_kind: Some("host_cli_resolved".to_string()),
            message:
                "Codex's native configuration resolver did not return an Engram MCP server entry."
                    .to_string(),
            ..unchecked_mcp_server()
        };
    }
    let value: Value = match serde_json::from_slice(&output.stdout) {
        Ok(value) => value,
        Err(error) => {
            return HarnessMcpServerCheck {
                checked: true,
                evidence_kind: Some("host_cli_resolved".to_string()),
                message: format!("Codex returned invalid MCP configuration JSON: {error}"),
                ..unchecked_mcp_server()
            };
        }
    };
    let transport = value.get("transport").unwrap_or(&Value::Null);
    let enabled = value.get("enabled").and_then(Value::as_bool);
    build_mcp_server_check(
        "host_cli_resolved",
        None,
        transport,
        enabled,
        true,
        "Codex's native configuration resolver",
    )
}

fn claude_mcp_server_check(cwd: &Path) -> IndexResult<HarnessMcpServerCheck> {
    let Some(home) = dirs::home_dir() else {
        return Ok(HarnessMcpServerCheck {
            checked: true,
            evidence_kind: Some("static_config".to_string()),
            message: "Claude MCP configuration could not be inspected because the home directory could not be resolved.".to_string(),
            ..unchecked_mcp_server()
        });
    };
    let user_path = home.join(".claude.json");
    let user_config = read_optional_json(&user_path)?;
    let canonical_cwd = fs::canonicalize(cwd).unwrap_or_else(|_| cwd.to_path_buf());
    let cwd_key = canonical_cwd.display().to_string();
    let project_path = canonical_cwd.join(".mcp.json");
    let project_config = read_optional_json(&project_path)?;

    let candidates = claude_mcp_candidates(
        user_config.as_ref(),
        project_config.as_ref(),
        &cwd_key,
        &user_path,
        &project_path,
    );

    let Some((source, server)) = candidates.first() else {
        return Ok(HarnessMcpServerCheck {
            checked: true,
            evidence_kind: Some("static_config".to_string()),
            message: format!(
                "No Engram MCP server was found in Claude's user or project configuration for {}.",
                canonical_cwd.display()
            ),
            ..unchecked_mcp_server()
        });
    };
    let source = source.clone();
    let mut check = build_mcp_server_check(
        "static_config",
        Some(source),
        server,
        None,
        false,
        "Claude's statically selected highest-precedence known config entry",
    );
    if candidates.len() > 1 {
        check.message.push_str(&format!(
            " {} lower-precedence Engram entry or entries were also found; Claude was not launched to resolve approval or runtime precedence.",
            candidates.len() - 1
        ));
    }
    Ok(check)
}

fn claude_mcp_candidates(
    user_config: Option<&Value>,
    project_config: Option<&Value>,
    cwd_key: &str,
    user_path: &Path,
    project_path: &Path,
) -> Vec<(String, Value)> {
    let mut candidates = Vec::new();
    if let Some(server) = user_config
        .and_then(|config| config.get("projects"))
        .and_then(|projects| projects.get(cwd_key))
        .and_then(|project| project.pointer("/mcpServers/engram"))
    {
        candidates.push((
            format!(
                "{}#/projects/{}/mcpServers/engram",
                user_path.display(),
                json_pointer_escape(cwd_key)
            ),
            server.clone(),
        ));
    }
    if let Some(server) = project_config.and_then(|config| config.pointer("/mcpServers/engram")) {
        candidates.push((
            format!("{}#/mcpServers/engram", project_path.display()),
            server.clone(),
        ));
    }
    if let Some(server) = user_config.and_then(|config| config.pointer("/mcpServers/engram")) {
        candidates.push((
            format!("{}#/mcpServers/engram", user_path.display()),
            server.clone(),
        ));
    }
    candidates
}

fn read_optional_json(path: &Path) -> IndexResult<Option<Value>> {
    match fs::read(path) {
        Ok(contents) => serde_json::from_slice(&contents)
            .map(Some)
            .map_err(|error| {
                IndexError::Parse(format!(
                    "failed to parse host configuration at {}: {error}",
                    path.display()
                ))
            }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn json_pointer_escape(value: &str) -> String {
    value.replace('~', "~0").replace('/', "~1")
}

fn build_mcp_server_check(
    evidence_kind: &str,
    source: Option<String>,
    server: &Value,
    enabled: Option<bool>,
    resolved_configuration_verified: bool,
    evidence_label: &str,
) -> HarnessMcpServerCheck {
    let transport = server
        .get("type")
        .and_then(Value::as_str)
        .map(str::to_string);
    let command = server
        .get("command")
        .and_then(Value::as_str)
        .map(str::to_string);
    let raw_args = server
        .get("args")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect::<Vec<_>>();
    let args = redact_sensitive_args(&raw_args);
    let mut env_keys = BTreeSet::new();
    if let Some(env) = server.get("env").and_then(Value::as_object) {
        env_keys.extend(env.keys().cloned());
    }
    if let Some(keys) = server.get("env_vars").and_then(Value::as_array) {
        env_keys.extend(keys.iter().filter_map(Value::as_str).map(str::to_string));
    }
    let (executable_path, executable_sha256) = command
        .as_deref()
        .and_then(resolve_configured_executable)
        .map(|path| {
            let hash = sha256_file(&path).ok();
            (Some(path.display().to_string()), hash)
        })
        .unwrap_or((None, None));
    let agent_profile_launch_configured = enabled.unwrap_or(true)
        && transport.as_deref().unwrap_or("stdio") == "stdio"
        && command.is_some()
        && executable_path.is_some()
        && has_agent_profile_serve_args(&raw_args);
    let message = if agent_profile_launch_configured {
        format!(
            "{evidence_label} reports a resolvable stdio command with `serve --profile agent`; this does not prove a running host loaded it or that the MCP runtime is live."
        )
    } else {
        format!(
            "{evidence_label} does not report a resolvable enabled stdio command with `serve --profile agent`."
        )
    };

    HarnessMcpServerCheck {
        checked: true,
        evidence_kind: Some(evidence_kind.to_string()),
        source,
        server_found: true,
        enabled,
        transport,
        command,
        args,
        env_keys: env_keys.into_iter().collect(),
        executable_path,
        executable_sha256,
        agent_profile_launch_configured,
        resolved_configuration_verified,
        running_host_loaded_verified: false,
        live_runtime_verified: false,
        message,
    }
}

fn redact_sensitive_args(args: &[String]) -> Vec<String> {
    let mut redact_next = false;
    args.iter()
        .map(|arg| {
            if redact_next {
                redact_next = false;
                return "<redacted>".to_string();
            }
            if let Some((name, _)) = arg.split_once('=') {
                if is_sensitive_arg_name(name) {
                    return format!("{name}=<redacted>");
                }
            }
            if is_sensitive_arg_name(arg) {
                redact_next = true;
            }
            arg.clone()
        })
        .collect()
}

fn is_sensitive_arg_name(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    [
        "secret",
        "token",
        "password",
        "credential",
        "api-key",
        "api_key",
        "authorization",
    ]
    .iter()
    .any(|needle| value.contains(needle))
}

fn resolve_configured_executable(command: &str) -> Option<PathBuf> {
    let path = Path::new(command);
    let resolved = if path.components().count() > 1 {
        path.to_path_buf()
    } else {
        resolve_executable_on_path(command)?
    };
    resolved
        .is_file()
        .then(|| fs::canonicalize(&resolved).unwrap_or(resolved))
}

fn has_agent_profile_serve_args(args: &[String]) -> bool {
    if args.first().map(String::as_str) != Some("serve")
        || args
            .iter()
            .any(|arg| matches!(arg.as_str(), "--http" | "--memory"))
    {
        return false;
    }
    args.windows(2)
        .any(|pair| pair[0] == "--profile" && pair[1] == "agent")
        || args.iter().any(|arg| arg == "--profile=agent")
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn check_harness_host(harness: HarnessKind) -> HarnessHostCheck {
    let executable_name = match harness {
        HarnessKind::ClaudeCode => Some("claude"),
        HarnessKind::Codex => Some("codex"),
        _ => None,
    };
    let Some(executable_name) = executable_name else {
        return HarnessHostCheck {
            checked: false,
            executable_path: None,
            executable_sha256: None,
            version: None,
            effective_configuration_verified: false,
            message: "No first-class host executable probe is defined for this harness."
                .to_string(),
        };
    };
    let Some(path) = resolve_executable_on_path(executable_name) else {
        return HarnessHostCheck {
            checked: true,
            executable_path: None,
            executable_sha256: None,
            version: None,
            effective_configuration_verified: false,
            message: format!("Host executable '{executable_name}' was not found on PATH."),
        };
    };

    let canonical_path = fs::canonicalize(&path).unwrap_or(path);
    let executable_sha256 = sha256_file(&canonical_path).ok();
    let version = Command::new(&canonical_path)
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| {
            let output = if output.stdout.is_empty() {
                output.stderr
            } else {
                output.stdout
            };
            String::from_utf8(output)
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        });

    HarnessHostCheck {
        checked: true,
        executable_path: Some(canonical_path.display().to_string()),
        executable_sha256,
        version,
        effective_configuration_verified: false,
        message: "Host executable identity is attested, but effective configuration in an already-running host is not verified by static checks.".to_string(),
    }
}

fn resolve_executable_on_path(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for directory in std::env::split_paths(&path) {
        let candidate = directory.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
        #[cfg(windows)]
        {
            let candidate = directory.join(format!("{name}.exe"));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn sha256_file(path: &Path) -> IndexResult<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn has_marker(contents: &str) -> bool {
    contents.contains(MARKER_MD) || contents.contains(MARKER_SH)
}

fn install_plan_message(status: HarnessAdapterStatus) -> String {
    match status {
        HarnessAdapterStatus::Missing => "will create generated adapter".to_string(),
        HarnessAdapterStatus::Drifted => "will update generated adapter".to_string(),
        HarnessAdapterStatus::Installed | HarnessAdapterStatus::UserOwned => {
            "no generated adapter write planned".to_string()
        }
    }
}

fn backup_path(path: &Path) -> PathBuf {
    for index in 0.. {
        let suffix = if index == 0 {
            "engram-backup".to_string()
        } else {
            format!("engram-backup.{index}")
        };
        let candidate = PathBuf::from(format!("{}.{}", path.display(), suffix));
        if !candidate.exists() {
            return candidate;
        }
    }
    unreachable!("infinite backup path search should always return")
}

struct ClaudeSettingsSource {
    label: &'static str,
    path: PathBuf,
    settings: Option<Value>,
}

struct ClaudeSettingsStatus {
    checks: Vec<HarnessSettingsCheck>,
    warnings: Vec<String>,
}

impl ClaudeSettingsStatus {
    fn has_missing_required(&self) -> bool {
        self.checks
            .iter()
            .any(|check| check.required && check.locations.is_empty())
    }
}

fn claude_project_settings_path(root: &Path) -> PathBuf {
    root.join(".claude/settings.json")
}

fn claude_local_settings_path(root: &Path) -> PathBuf {
    root.join(".claude/settings.local.json")
}

fn claude_settings_snippet_path(root: &Path) -> PathBuf {
    root.join(".claude/engram-settings-snippet.json")
}

fn read_claude_settings_source(
    label: &'static str,
    path: PathBuf,
) -> IndexResult<ClaudeSettingsSource> {
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(ClaudeSettingsSource {
                label,
                path,
                settings: None,
            });
        }
        Err(error) => return Err(error.into()),
    };
    let settings: Value = serde_json::from_str(&contents).map_err(|e| {
        IndexError::Parse(format!(
            "failed to parse Claude settings at {}: {e}",
            path.display()
        ))
    })?;
    Ok(ClaudeSettingsSource {
        label,
        path,
        settings: Some(settings),
    })
}

fn read_claude_settings_sources(root: &Path) -> IndexResult<Vec<ClaudeSettingsSource>> {
    Ok(vec![
        read_claude_settings_source("settings.json", claude_project_settings_path(root))?,
        read_claude_settings_source("settings.local.json", claude_local_settings_path(root))?,
    ])
}

fn claude_settings_status(
    root: &Path,
    enforcement_profile: HarnessEnforcementProfile,
) -> IndexResult<ClaudeSettingsStatus> {
    let sources = read_claude_settings_sources(root)?;
    let present_sources: Vec<_> = sources
        .iter()
        .filter(|source| source.settings.is_some())
        .collect();

    let mut warnings = Vec::new();
    if present_sources.is_empty() {
        warnings.push(format!(
            "Claude settings are missing at {} and {}; run harness install --write --settings-target settings.json for shared setup or --settings-target settings.local.json for local setup.",
            claude_project_settings_path(root).display(),
            claude_local_settings_path(root).display()
        ));
    }

    let mut checks = Vec::new();
    for permission in claude_required_permissions() {
        let locations = sources
            .iter()
            .filter(|source| {
                source
                    .settings
                    .as_ref()
                    .map(|settings| claude_settings_has_permission(settings, permission))
                    .unwrap_or(false)
            })
            .map(|source| source.label.to_string())
            .collect::<Vec<_>>();
        if locations.is_empty() {
            warnings.push(format!(
                "Claude settings are missing permission allow entry '{permission}' in both settings.json and settings.local.json."
            ));
        }
        checks.push(HarnessSettingsCheck {
            name: permission.to_string(),
            kind: "permission".to_string(),
            required: true,
            message: settings_check_message(&locations),
            locations,
        });
    }

    for (event, matcher) in claude_required_hook_events(enforcement_profile) {
        let name = match matcher {
            Some(matcher) => format!("{event}:{matcher}"),
            None => event.to_string(),
        };
        let locations = sources
            .iter()
            .filter(|source| {
                source
                    .settings
                    .as_ref()
                    .map(|settings| {
                        claude_settings_has_hook(settings, event, matcher, enforcement_profile)
                    })
                    .unwrap_or(false)
            })
            .map(|source| source.label.to_string())
            .collect::<Vec<_>>();
        if locations.is_empty() {
            warnings.push(format!(
                "Claude settings are missing Engram hook registration for {event} in both settings.json and settings.local.json."
            ));
        }
        checks.push(HarnessSettingsCheck {
            name,
            kind: "hook".to_string(),
            required: true,
            message: settings_check_message(&locations),
            locations,
        });
    }

    warn_for_stale_engram_permissions(&sources, &mut warnings);
    warn_for_split_settings(&checks, &mut warnings);

    Ok(ClaudeSettingsStatus { checks, warnings })
}

fn settings_check_message(locations: &[String]) -> String {
    if locations.is_empty() {
        "missing from Claude settings".to_string()
    } else {
        format!("found in {}", locations.join(", "))
    }
}

fn warn_for_stale_engram_permissions(sources: &[ClaudeSettingsSource], warnings: &mut Vec<String>) {
    let required: BTreeSet<_> = claude_required_permissions().iter().copied().collect();
    for source in sources {
        let Some(settings) = &source.settings else {
            continue;
        };
        let stale = claude_engram_permissions(settings)
            .into_iter()
            .filter(|permission| !required.contains(permission.as_str()))
            .collect::<Vec<_>>();
        if !stale.is_empty() {
            warnings.push(format!(
                "{} contains Engram permission entries that are not part of the current Claude harness contract: {}.",
                source.label,
                stale.join(", ")
            ));
        }
    }
}

fn warn_for_split_settings(checks: &[HarnessSettingsCheck], warnings: &mut Vec<String>) {
    let mut locations = BTreeSet::new();
    for check in checks {
        for location in &check.locations {
            locations.insert(location.as_str());
        }
    }
    if locations.len() > 1 {
        warnings.push(format!(
            "Engram Claude settings are split across {}; verify effective hook configuration with Claude Code /hooks.",
            locations.into_iter().collect::<Vec<_>>().join(" and ")
        ));
    }
}

fn warn_for_installed_claude_hook_files_without_settings(
    adapters: &[HarnessAdapterCheck],
    settings: &[HarnessSettingsCheck],
    warnings: &mut Vec<String>,
) {
    let hook_mappings = [
        (
            "SessionStart:startup|resume|compact",
            "claude-session-start-hook",
            "SessionStart startup|resume|compact",
        ),
        ("SessionEnd", "claude-session-end-hook", "SessionEnd"),
    ];

    for (setting_name, adapter_name, event_label) in hook_mappings {
        let missing_required_setting = settings.iter().any(|check| {
            check.kind == "hook"
                && check.required
                && check.name == setting_name
                && check.locations.is_empty()
        });
        if !missing_required_setting {
            continue;
        }

        let Some(adapter) = adapters.iter().find(|check| {
            check.name == adapter_name && check.status == HarnessAdapterStatus::Installed
        }) else {
            continue;
        };

        warnings.push(format!(
            "Generated Claude hook file for {event_label} is installed at {}, but Claude settings do not register the required {setting_name} hook; Claude will not run that file until settings.json or settings.local.json references it.",
            adapter.path
        ));
    }
}

fn claude_engram_permissions(settings: &Value) -> Vec<String> {
    settings
        .pointer("/permissions/allow")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter(|permission| permission.starts_with("mcp__engram__"))
        .map(str::to_string)
        .collect()
}

fn merge_claude_settings(
    root: &Path,
    target: HarnessSettingsTarget,
    write: bool,
    enforcement_profile: HarnessEnforcementProfile,
    planned: &mut Vec<HarnessInstallFile>,
    written: &mut Vec<HarnessInstallFile>,
    skipped: &mut Vec<HarnessInstallFile>,
    warnings: &mut Vec<String>,
) -> IndexResult<()> {
    let Some(path) = target.path(root) else {
        skipped.push(HarnessInstallFile {
            name: "claude-settings-merge".to_string(),
            path: claude_settings_snippet_path(root).display().to_string(),
            written: false,
            message: "settings target is snippet-only; no Claude settings file will be modified"
                .to_string(),
        });
        warnings.push(
            "Claude settings were not modified because settings target is snippet-only; merge .claude/engram-settings-snippet.json manually or rerun with --settings-target settings.json."
                .to_string(),
        );
        return Ok(());
    };
    warn_for_settings_target(target, root, warnings, enforcement_profile)?;
    let mut settings = match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str::<Value>(&contents)
            .map_err(|e| IndexError::Parse(format!("failed to parse Claude settings: {e}")))?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => json!({}),
        Err(error) => return Err(error.into()),
    };

    let changed_permissions = merge_claude_permissions(&mut settings);
    let changed_hooks = merge_claude_hooks(&mut settings, enforcement_profile);
    let changed = changed_permissions || changed_hooks;
    let path_string = path.display().to_string();
    if changed {
        planned.push(HarnessInstallFile {
            name: "claude-settings-merge".to_string(),
            path: path_string.clone(),
            written: false,
            message: format!(
                "will merge Engram MCP permissions and lifecycle hooks into {}",
                target
            ),
        });
        if write {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let contents = serde_json::to_string_pretty(&settings)
                .map_err(|e| IndexError::Parse(format!("failed to render Claude settings: {e}")))?;
            fs::write(&path, contents + "\n")?;
            written.push(HarnessInstallFile {
                name: "claude-settings-merge".to_string(),
                path: path_string,
                written: true,
                message: format!(
                    "merged Engram MCP permissions and lifecycle hooks into {}",
                    target
                ),
            });
        }
    } else {
        skipped.push(HarnessInstallFile {
            name: "claude-settings-merge".to_string(),
            path: path_string,
            written: false,
            message: format!("{} already includes Engram permissions and hooks", target),
        });
    }
    Ok(())
}

fn warn_for_settings_target(
    target: HarnessSettingsTarget,
    root: &Path,
    warnings: &mut Vec<String>,
    enforcement_profile: HarnessEnforcementProfile,
) -> IndexResult<()> {
    match target {
        HarnessSettingsTarget::Project => {
            let local = read_claude_settings_source(
                "settings.local.json",
                claude_local_settings_path(root),
            )?;
            if let Some(settings) = local.settings {
                let permissions = claude_engram_permissions(&settings);
                let has_hooks = claude_required_hook_events(enforcement_profile)
                    .into_iter()
                    .any(|(event, matcher)| {
                        claude_settings_has_hook(&settings, event, matcher, enforcement_profile)
                    });
                if !permissions.is_empty() || has_hooks {
                    warnings.push(format!(
                        "{} already contains Engram entries; project settings will be written to settings.json, while local settings remain personal and have higher precedence.",
                        local.path.display()
                    ));
                }
            }
        }
        HarnessSettingsTarget::Local => warnings.push(
            "Writing Claude settings to settings.local.json; this is personal, gitignored configuration and will not make the repo agent-ready for collaborators."
                .to_string(),
        ),
        HarnessSettingsTarget::SnippetOnly => {}
    }
    Ok(())
}

fn merge_claude_permissions(settings: &mut Value) -> bool {
    ensure_object(settings);
    if settings
        .get("permissions")
        .and_then(Value::as_object)
        .is_none()
    {
        settings["permissions"] = json!({});
    }
    if settings["permissions"]
        .get("allow")
        .and_then(Value::as_array)
        .is_none()
    {
        settings["permissions"]["allow"] = json!([]);
    }
    let allow = settings["permissions"]["allow"]
        .as_array_mut()
        .expect("allow must be an array");
    let mut changed = false;
    for permission in claude_required_permissions() {
        if !allow.iter().any(|value| value.as_str() == Some(permission)) {
            allow.push(Value::String(permission.to_string()));
            changed = true;
        }
    }
    changed
}

fn merge_claude_hooks(
    settings: &mut Value,
    enforcement_profile: HarnessEnforcementProfile,
) -> bool {
    ensure_object(settings);
    if settings.get("hooks").and_then(Value::as_object).is_none() {
        settings["hooks"] = json!({});
    }
    let mut changed = false;
    changed |= remove_claude_generated_mcp_hooks(settings, enforcement_profile);
    changed |= remove_claude_hook_handler(
        settings,
        "SessionEnd",
        None,
        &claude_mcp_hook_handler("SessionEnd", enforcement_profile),
    );
    changed |= remove_stale_claude_dispatch_hook_handlers(
        settings,
        "SessionStart",
        Some("startup|resume|compact"),
        "engram-session-start.sh",
        CLAUDE_HOOK_COMMAND,
    );
    changed |= remove_stale_claude_dispatch_hook_handlers(
        settings,
        "SessionEnd",
        None,
        "engram-session-end.sh",
        CLAUDE_SESSION_END_HOOK_COMMAND,
    );
    changed |= remove_claude_hook_handler(
        settings,
        "SessionStart",
        Some("startup|resume|compact"),
        &json!({
            "type": "command",
            "command": CLAUDE_LEGACY_HOOK_COMMAND,
            "timeout": 10
        }),
    );
    changed |= remove_claude_hook_handler(
        settings,
        "SessionEnd",
        None,
        &json!({
            "type": "command",
            "command": CLAUDE_LEGACY_SESSION_END_HOOK_COMMAND,
            "timeout": 15
        }),
    );
    changed |= ensure_claude_hook(
        settings,
        "SessionStart",
        Some("startup|resume|compact"),
        json!({
            "type": "command",
            "command": CLAUDE_HOOK_COMMAND,
            "timeout": 10
        }),
    );
    for (event, matcher) in claude_mcp_hook_events(enforcement_profile) {
        changed |= ensure_claude_hook(
            settings,
            event,
            matcher,
            claude_mcp_hook_handler(event, enforcement_profile),
        );
    }
    changed |= ensure_claude_hook(
        settings,
        "SessionEnd",
        None,
        json!({
            "type": "command",
            "command": CLAUDE_SESSION_END_HOOK_COMMAND,
            "timeout": 15
        }),
    );
    changed
}

fn remove_claude_generated_mcp_hooks(
    settings: &mut Value,
    enforcement_profile: HarnessEnforcementProfile,
) -> bool {
    let mut changed = false;
    let desired_events = claude_mcp_hook_events(enforcement_profile);
    let mut known_events = Vec::new();
    for profile in [
        HarnessEnforcementProfile::Soft,
        HarnessEnforcementProfile::Graduated,
        HarnessEnforcementProfile::Strict,
    ] {
        for (event, matcher) in claude_mcp_hook_events(profile) {
            if !known_events.contains(&(event, matcher)) {
                known_events.push((event, matcher));
            }
        }
    }
    changed |= remove_claude_generated_mcp_hook_handlers(settings, "SessionEnd", None, None);
    for (event, matcher) in known_events {
        let desired_handler = desired_events
            .contains(&(event, matcher))
            .then(|| claude_mcp_hook_handler(event, enforcement_profile));
        changed |= remove_claude_generated_mcp_hook_handlers(
            settings,
            event,
            matcher,
            desired_handler.as_ref(),
        );
    }
    changed
}

fn ensure_object(value: &mut Value) {
    if !value.is_object() {
        *value = json!({});
    }
}

fn ensure_claude_hook(
    settings: &mut Value,
    event: &str,
    matcher: Option<&str>,
    handler: Value,
) -> bool {
    let hooks = settings["hooks"]
        .as_object_mut()
        .expect("hooks must be object");
    let event_entry = hooks.entry(event.to_string()).or_insert_with(|| json!([]));
    if !event_entry.is_array() {
        *event_entry = json!([]);
    }
    if claude_event_has_handler(event_entry, matcher, &handler) {
        return false;
    }

    let mut group = json!({ "hooks": [handler] });
    if let Some(matcher) = matcher {
        group["matcher"] = Value::String(matcher.to_string());
    }
    event_entry
        .as_array_mut()
        .expect("event entry must be array")
        .push(group);
    true
}

fn claude_event_has_handler(event_entry: &Value, matcher: Option<&str>, handler: &Value) -> bool {
    event_entry.as_array().into_iter().flatten().any(|group| {
        let matcher_matches = match matcher {
            Some(expected) => group.get("matcher").and_then(Value::as_str) == Some(expected),
            None => group.get("matcher").is_none(),
        };
        matcher_matches
            && group
                .get("hooks")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .any(|existing| existing == handler)
    })
}

fn remove_claude_hook_handler(
    settings: &mut Value,
    event: &str,
    matcher: Option<&str>,
    handler: &Value,
) -> bool {
    let Some(hooks) = settings.get_mut("hooks").and_then(Value::as_object_mut) else {
        return false;
    };
    let Some(groups) = hooks.get_mut(event).and_then(Value::as_array_mut) else {
        return false;
    };

    let mut changed = false;
    for group in groups.iter_mut() {
        let matcher_matches = match matcher {
            Some(expected) => group.get("matcher").and_then(Value::as_str) == Some(expected),
            None => group.get("matcher").is_none(),
        };
        if !matcher_matches {
            continue;
        }
        if let Some(hooks) = group.get_mut("hooks").and_then(Value::as_array_mut) {
            let before = hooks.len();
            hooks.retain(|existing| existing != handler);
            changed |= hooks.len() != before;
        }
    }
    let before = groups.len();
    groups.retain(|group| {
        group
            .get("hooks")
            .and_then(Value::as_array)
            .map(|hooks| !hooks.is_empty())
            .unwrap_or(true)
    });
    changed |= groups.len() != before;
    if groups.is_empty() {
        hooks.remove(event);
        changed = true;
    }
    changed
}

fn remove_stale_claude_dispatch_hook_handlers(
    settings: &mut Value,
    event: &str,
    matcher: Option<&str>,
    hook_file: &str,
    keep_command: &str,
) -> bool {
    let Some(hooks) = settings.get_mut("hooks").and_then(Value::as_object_mut) else {
        return false;
    };
    let Some(groups) = hooks.get_mut(event).and_then(Value::as_array_mut) else {
        return false;
    };

    let mut changed = false;
    for group in groups.iter_mut() {
        let matcher_matches = match matcher {
            Some(expected) => group.get("matcher").and_then(Value::as_str) == Some(expected),
            None => group.get("matcher").is_none(),
        };
        if !matcher_matches {
            continue;
        }
        if let Some(group_hooks) = group.get_mut("hooks").and_then(Value::as_array_mut) {
            let before = group_hooks.len();
            group_hooks.retain(|handler| {
                let Some(command) = handler.get("command").and_then(Value::as_str) else {
                    return true;
                };
                command == keep_command
                    || handler.get("type").and_then(Value::as_str) != Some("command")
                    || !command.contains(hook_file)
                    || !command.contains("project_hook=")
                    || !command.contains("home_hook=")
                    || !command.contains("Engram ")
            });
            changed |= group_hooks.len() != before;
        }
    }
    let before = groups.len();
    groups.retain(|group| {
        group
            .get("hooks")
            .and_then(Value::as_array)
            .map(|hooks| !hooks.is_empty())
            .unwrap_or(true)
    });
    changed |= groups.len() != before;
    if groups.is_empty() {
        hooks.remove(event);
        changed = true;
    }
    changed
}

fn remove_claude_generated_mcp_hook_handlers(
    settings: &mut Value,
    event: &str,
    matcher: Option<&str>,
    keep_handler: Option<&Value>,
) -> bool {
    let Some(hooks) = settings.get_mut("hooks").and_then(Value::as_object_mut) else {
        return false;
    };
    let Some(groups) = hooks.get_mut(event).and_then(Value::as_array_mut) else {
        return false;
    };

    let mut changed = false;
    for group in groups.iter_mut() {
        let matcher_matches = match matcher {
            Some(expected) => group.get("matcher").and_then(Value::as_str) == Some(expected),
            None => group.get("matcher").is_none(),
        };
        if !matcher_matches {
            continue;
        }
        if let Some(group_hooks) = group.get_mut("hooks").and_then(Value::as_array_mut) {
            let before = group_hooks.len();
            group_hooks.retain(|existing| {
                !is_claude_generated_mcp_hook_handler(existing, event)
                    || keep_handler == Some(existing)
            });
            changed |= group_hooks.len() != before;
        }
    }
    let before = groups.len();
    groups.retain(|group| {
        group
            .get("hooks")
            .and_then(Value::as_array)
            .map(|hooks| !hooks.is_empty())
            .unwrap_or(true)
    });
    changed |= groups.len() != before;
    if groups.is_empty() {
        hooks.remove(event);
        changed = true;
    }
    changed
}

fn is_claude_generated_mcp_hook_handler(handler: &Value, event: &str) -> bool {
    handler.get("type").and_then(Value::as_str) == Some("mcp_tool")
        && handler.get("server").and_then(Value::as_str) == Some("engram")
        && handler.get("tool").and_then(Value::as_str) == Some("harness")
        && handler.pointer("/input/action").and_then(Value::as_str) == Some("hook_event")
        && handler.pointer("/input/harness").and_then(Value::as_str) == Some("claude_code")
        && handler
            .pointer("/input/hook_event_name")
            .and_then(Value::as_str)
            == Some(event)
}

fn claude_settings_has_hook(
    settings: &Value,
    event: &str,
    matcher: Option<&str>,
    enforcement_profile: HarnessEnforcementProfile,
) -> bool {
    settings
        .pointer(&format!("/hooks/{event}"))
        .map(|entry| {
            claude_event_has_handler(
                entry,
                matcher,
                &claude_expected_handler(event, enforcement_profile),
            )
        })
        .unwrap_or(false)
}

fn claude_settings_has_permission(settings: &Value, permission: &str) -> bool {
    settings
        .pointer("/permissions/allow")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|value| value.as_str() == Some(permission))
}

fn claude_expected_handler(event: &str, enforcement_profile: HarnessEnforcementProfile) -> Value {
    if event == "SessionStart" {
        json!({
            "type": "command",
            "command": CLAUDE_HOOK_COMMAND,
            "timeout": 10
        })
    } else if event == "SessionEnd" {
        json!({
            "type": "command",
            "command": CLAUDE_SESSION_END_HOOK_COMMAND,
            "timeout": 15
        })
    } else {
        claude_mcp_hook_handler(event, enforcement_profile)
    }
}

fn claude_required_permissions() -> &'static [&'static str] {
    &[
        "mcp__engram__orient",
        "mcp__engram__memory",
        "mcp__engram__harness",
        "mcp__engram__obligations",
        "mcp__engram__repo",
        "mcp__engram__search",
    ]
}

fn claude_required_hook_events(
    enforcement_profile: HarnessEnforcementProfile,
) -> Vec<(&'static str, Option<&'static str>)> {
    let mut events = vec![("SessionStart", Some("startup|resume|compact"))];
    events.extend(claude_mcp_hook_events(enforcement_profile));
    events.push(("SessionEnd", None));
    events
}

fn claude_mcp_hook_events(
    enforcement_profile: HarnessEnforcementProfile,
) -> Vec<(&'static str, Option<&'static str>)> {
    let mut events = vec![
        ("PreCompact", Some("manual|auto")),
        ("PostCompact", Some("manual|auto")),
    ];
    if enforcement_profile == HarnessEnforcementProfile::Soft {
        return events;
    }
    events.extend([
        ("UserPromptSubmit", None),
        ("PreToolUse", Some("*")),
        (
            "PostToolUse",
            Some("mcp__engram__orient|mcp__engram__memory"),
        ),
        ("PostToolUse", Some("Write|Edit|MultiEdit")),
        ("PostToolUseFailure", Some("*")),
        ("Stop", None),
    ]);
    events
}

fn claude_mcp_hook_handler(event: &str, enforcement_profile: HarnessEnforcementProfile) -> Value {
    json!({
        "type": "mcp_tool",
        "server": "engram",
        "tool": "harness",
        "timeout": 10,
        "input": {
            "action": "hook_event",
            "harness": "claude_code",
            "enforcement": enforcement_profile.to_string(),
            "hook_event_name": event,
            "session_id": "${session_id}",
            "cwd": "${cwd}",
            "transcript_path": "${transcript_path}",
            "prompt": "${prompt}",
            "tool_name": "${tool_name}",
            "tool_error": "${error}",
            "tool_input_command": "${tool_input.command}",
            "tool_input_action": "${tool_input.action}",
            "file_path": "${tool_input.file_path}",
            "last_assistant_message": "${last_assistant_message}",
            "compact_summary": "${compact_summary}",
            "trigger": "${trigger}",
            "reason": "${reason}",
            "stop_hook_active": "${stop_hook_active}",
            "write_policy": "durable",
            "model_provider": "anthropic",
            "model": "claude-code",
            "surface": "claude-code",
            "actor": "agent"
        }
    })
}

fn resolve_root(root: Option<&Path>) -> IndexResult<PathBuf> {
    match root {
        Some(path) => Ok(path.to_path_buf()),
        None => dirs::home_dir()
            .ok_or_else(|| IndexError::NotConfigured("could not determine home directory".into())),
    }
}

fn set_executable_if_hook(path: &Path, kind: HarnessAdapterKind) -> IndexResult<()> {
    if !matches!(
        kind,
        HarnessAdapterKind::ClaudeHook | HarnessAdapterKind::CodexHook
    ) {
        return Ok(());
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(path)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions)?;
    }

    Ok(())
}

fn harness_contract_sentence(
    enforcement_profile: HarnessEnforcementProfile,
    runtime_enforced: bool,
) -> &'static str {
    match (enforcement_profile, runtime_enforced) {
        (HarnessEnforcementProfile::Soft, _) => {
            "This uses the soft profile. Missing lifecycle steps should be reported as warnings, not blockers."
        }
        (HarnessEnforcementProfile::Graduated, true) => {
            "This uses the graduated profile. Claude Code hooks enforce task-start lean orientation before non-Engram tools and block final response once when open obligations remain."
        }
        (HarnessEnforcementProfile::Graduated, false) => {
            "This uses the graduated profile. Follow the lifecycle as mandatory agent behavior; runtime blocking is only available where the host exposes hooks."
        }
        (HarnessEnforcementProfile::Strict, true) => {
            "This uses the strict profile. Claude Code hooks keep blocking gated actions until required obligations are resolved, explicitly skipped, or Engram is degraded."
        }
        (HarnessEnforcementProfile::Strict, false) => {
            "This uses the strict profile. Follow the lifecycle as mandatory agent behavior; runtime blocking is only available where the host exposes hooks."
        }
    }
}

const CONCRETE_MEMORY_TRIGGERS: &str = "Treat JIRA keys, PRs/issues, known entities/services, project-state claims, final responses, and durable discoveries as concrete Engram triggers.";
const RELATED_RETRIEVAL_SCOPE_EXAMPLE: &str =
    r#"scope={relevance_mode:"related", project:..., cwd:...}"#;
const LOCAL_PROCEDURE_SCOPE_EXAMPLE: &str = r#"scope={relevance_mode:"local", cwd:...}"#;
const TASK_SCOPE_GUIDANCE: &str = concat!(
    "Pass `task=<exact task name or tracker key>` to `orient` when task identity is known; ",
    "task names require an explicit project. If task identity or its project relationship cannot ",
    "be resolved, ask instead of applying task-scoped memory."
);
const VERIFIED_PROCEDURE_GUIDANCE: &str = concat!(
    "Every actionable repository task has a mandatory procedure route, even when the user does ",
    "not say earlier, previous, remembered, or learned. Requests to handle, run, debug, test, ",
    "build, deploy, modify, or determine how to perform a repository-local operation, and requests ",
    "to use durable procedure memory, are actionable. ",
    "Use local procedure matching itself as the task-start identity boundary for actionable ",
    "repository work. Before checking native memory or exploring commands, call ",
    "`memory(action=procedure_match, query=<a bounded task-focused excerpt from the current user request>, ",
    "scope={relevance_mode:\"local\", cwd:...}, conditions={...})`. The `query` is required and ",
    "must be at most 512 characters. Preserve concrete operation nouns, identifiers, and failure ",
    "text verbatim; omit unrelated meta/output instructions and secret values. The query is ",
    "retrieval text, not an authorization channel. ",
    "Pass the host's exact current `cwd`; do not call `orient` first solely to obtain it, and do not ",
    "replace it with the repository checkout root. The match response returns the structured ",
    "repository/project/component identity and authorization boundary. If lean `orient` already ran ",
    "for another reason, pass that same original cwd. ",
    "`memory(action=list)` is never a substitute for procedure matching. ",
    "Project ambiguity blocks project/task-scoped memory only; it does not block this ",
    "repository-local procedure match. Pass only exact conditions already observed for prerequisites ",
    "that have no declarative source. Procedure matching is force-local at the API boundary; never ",
    "request related or global procedure matching. Local procedure matching authorizes ",
    "global/user and directly ",
    "applicable repository memory without inventing a work project; supply a canonical project ",
    "only for project/task guidance. ",
    "For source-backed prerequisites, Engram reads the configured Git-tracked file from the ",
    "resolved current checkout and returns value-redacted `condition_observations`; caller text ",
    "cannot override those observations. Execute only returned procedures. If Engram abstains ",
    "because evidence, scope, freshness, or unsourced prerequisites do not match, use ",
    "`required_condition_keys` and `next_actions` to search ",
    "from the returned `current_checkout_root` through authoritative local repository ",
    "configuration or tool output for the exact keys. After locating a file-backed value, read ",
    "that exact source path in a separate direct tool call from `current_checkout_root`, then ",
    "retry with exact observed values. Do not guess; do not apply the rejected procedure. ",
    "An empty `procedures` list proves only that Engram found no applicable verified procedure. ",
    "Use the returned structured `identity` as the checkout and authorization boundary: ",
    "rely only on non-null identity fields returned by the same call, while ",
    "`identity.project.status` states whether project-scoped retrieval is authorized. If ",
    "`suggested_operation_evidence` is present, treat it as a required host-action protocol, not ",
    "an optional suggestion. When `required_before_final_abstention=true`, the next tool call must ",
    "read its absolute `resolved_path`, which Engram canonicalized under ",
    "`identity.repository.checkout_root`, unchanged exactly once. Do this before interpreting ",
    "`abstained`, before asking for project confirmation, and before final output, including when ",
    "the user requested durable-memory-only handling. ",
    "`allowed_when_project_requires_confirmation=true` means this repository-local evidence read ",
    "remains authorized despite unresolved project scope. ",
    "`authorizes_procedure_execution=false` means the read never permits executing a remembered ",
    "procedure. Treat its checkout-relative `path` as provenance only and do not substitute a ",
    "relative path. Treat it only as source evidence after that direct read. Otherwise, ",
    "before final abstention on an actionable repository task, make at most one ",
    "bounded read-only lookup for the directly relevant tracked runbook, configuration, or source. ",
    "Do not re-read identity files. Do not re-derive a project from the repository name or directory ",
    "basename, broaden outside the checkout, or execute candidate commands. If project status is ",
    "`requires_confirmation`, report the material ambiguity and ask the user. ",
    "Abstain from executing a remembered procedure, but report the exact local evidence and next ",
    "action. ",
    "Execute ",
    "repository-scoped commands from the returned `current_checkout_root`; stored procedure ",
    "`scope.local_path` and evidence paths are provenance only and must never redirect execution ",
    "to an older checkout. A successful attempt may be stored as ",
    "a procedure candidate, ",
    "but do not claim it is verified until its machine-readable execution receipt is verified."
);
const SCOPED_SEARCH_GUIDANCE: &str = concat!(
    "Use `search(query=..., project=..., cwd=..., relevance_mode=\"related\")` only when ",
    "orientation is insufficient. Do not broaden retrieval unless the user explicitly requests it."
);
const DURABLE_CAPTURE_GUIDANCE: &str = concat!(
    "After a user-confirmed decision or a non-obvious source-grounded discovery, use ",
    "`memory(action=add)` with the narrowest project/repository/task scope, writer provenance, ",
    "and file, tool, commit, or URL evidence. Every add requires `kind`, `title`, `content`, ",
    "`origin`, `scope_type`, `writer_harness`, `model_provider`, and `model`; also pass the ",
    "scope selector required by `scope_type` (repository scope needs `remote_url` or ",
    "`local_path`). A retrieval `scope` object does not replace these write-scope fields. For ",
    "`kind=procedure`, also pass the structured `procedure` card with task, commands, ",
    "prerequisites, failure signatures, verification command, expected exit code, and output ",
    "marker. When a prerequisite has a stable scalar in a Git-tracked TOML file, map its exact ",
    "key through `prerequisite_sources` so Engram can re-read it from future checkouts. Store a ",
    "compact handoff memory only when future sessions need a concrete next ",
    "action. Never store credentials, tokens, private keys, or other secrets."
);

fn claude_memory_session_command(enforcement_profile: HarnessEnforcementProfile) -> String {
    let contract = harness_contract_sentence(enforcement_profile, true);
    format!(
        r#"{MARKER_MD}
# Engram Memory Session

Use this command when a Claude Code session needs persistent project memory.

Lifecycle contract ({enforcement_profile}):
- At task/session start, use the procedure route below as the identity boundary for actionable
  repository work. For other tasks, call `orient` with current cwd, prompt, `agent=claude_code`,
  and `response_shape="lean"`; supply project only when its canonical identity is known.
- {TASK_SCOPE_GUIDANCE}
- {VERIFIED_PROCEDURE_GUIDANCE}
- For requests other than repository-local procedure matching, if project/repository resolution is
  ambiguous, stop and ask the user instead of applying project/task-scoped memory.
- {CONCRETE_MEMORY_TRIGGERS}
- {SCOPED_SEARCH_GUIDANCE}
- {DURABLE_CAPTURE_GUIDANCE}
- Use `memory(action=archive)` for obsolete history. Use `memory(action=forget)` only for an exact
  item ID after explicit confirmation because deletion is irreversible.
- In commit workflows, consult memory for relevant preferences, rules, and limitations first.

{contract}
"#
    )
}

fn claude_resume_session_command() -> String {
    format!(
        r#"{MARKER_MD}
# Resume Engram Session

1. Call `orient` with current cwd, `agent=claude_code`, and `response_shape="lean"`; supply
   project only when its canonical identity is known.
   {TASK_SCOPE_GUIDANCE}
2. Read the selected scope, compact context, ambiguities, and candidate IDs. Ask the user before
   project/task-scoped memory if that scope cannot be resolved safely; project ambiguity alone does
   not block the repository-local procedure route in step 4.
3. Use scoped `search` only when the compact orientation lacks required evidence.
4. For every actionable repository task, even when the project is unresolved and even when the
   user does not say remembered/learned, call
   `memory(action=procedure_match, query=<a bounded task-focused excerpt from the current user request>,
   {LOCAL_PROCEDURE_SCOPE_EXAMPLE}, conditions=...)`. The query is required and must be at most
   512 characters. Preserve concrete operation nouns, identifiers, and failure text verbatim;
   omit unrelated meta/output instructions and secret values. The query is retrieval text, not an
   authorization channel. Project ambiguity
   blocks project/task memory only, not repository-local procedure matching. `memory(action=list)`
   is not a substitute. Copy the exact cwd returned by `orient`; do not replace it with the
   checkout root. Execute only a returned verified match. On no-result, read exactly one
   absolute `suggested_operation_evidence.resolved_path` unchanged when present. When
   `required_before_final_abstention=true`, perform that read before interpreting `abstained`,
   asking for project confirmation, or returning final output. The read is repository-local
   evidence collection, not procedure execution. Otherwise keep the bounded local fallback. Do
   not substitute the checkout-relative `path` for the host read.
5. Store only compact, evidenced durable memory that a future session genuinely needs. Never
   store secrets.
"#
    )
}

fn claude_end_session_command() -> String {
    format!(
        r#"{MARKER_MD}
# End Engram Session

Before ending:
- Store a project- or repository-scoped `kind=handoff` memory only if another session needs a
  concrete next action, unresolved decision, or material risk.
- Store source-grounded decisions and discoveries separately with writer provenance and evidence.
- Do not copy the transcript, routine progress, command output, or secrets into memory.
- Archive obsolete memory; permanently forget only an exact item after explicit confirmation.
"#
    )
}

fn claude_session_start_hook(enforcement_profile: HarnessEnforcementProfile) -> String {
    let contract = harness_contract_sentence(enforcement_profile, true);
    format!(
        r#"#!/usr/bin/env bash
{MARKER_SH}
set -euo pipefail

INPUT=$(cat)
CWD=$(printf '%s' "$INPUT" | jq -r '.cwd // empty')
SOURCE=$(printf '%s' "$INPUT" | jq -r '.source // empty')
SESSION_ID=$(printf '%s' "$INPUT" | jq -r '.session_id // empty')

if [ -z "$CWD" ] || [ "$CWD" = "null" ]; then
  CWD="${{CLAUDE_PROJECT_DIR:-}}"
fi

CONTEXT_BODY=$(cat <<'ENGRAM_CONTEXT'
Engram is the durable Memory OS for this Claude Code session.
For actionable repository work, use the local procedure_match route below as the task-start identity boundary before any shell, filesystem, native-memory, repository-inspection, or non-Engram MCP action. For other tasks, call the Engram MCP orient tool with the activation cwd, prompt, agent=claude_code, and response_shape=lean. The activation cwd is already supplied; do not run `pwd` first. Supply project only when its canonical identity is known; never infer it from the directory basename.
{TASK_SCOPE_GUIDANCE}
{CONCRETE_MEMORY_TRIGGERS}
{VERIFIED_PROCEDURE_GUIDANCE}
For requests other than repository-local procedure matching, ask when project/task scope is
ambiguous. Use scoped search only when orientation is insufficient.
{DURABLE_CAPTURE_GUIDANCE}
{contract} Do not fabricate missing memory or turn routine session history into durable context.
ENGRAM_CONTEXT
)

CONTEXT="<engram_session_activation source=\"$SOURCE\" cwd=\"$CWD\" session_id=\"$SESSION_ID\">
$CONTEXT_BODY
</engram_session_activation>"

CONTEXT_JSON=$(printf '%s' "$CONTEXT" | jq -Rs .)

cat <<EOF
{{
  "continue": true,
  "systemMessage": $CONTEXT_JSON
}}
EOF
"#
    )
}

fn claude_stop_nudge_hook(enforcement_profile: HarnessEnforcementProfile) -> String {
    let final_message = match enforcement_profile {
        HarnessEnforcementProfile::Soft => {
            "Engram final-response check is advisory: if this hook surfaced open obligations, resolve them or record an explicit skip reason. Store only compact, evidenced durable memory that a future session genuinely needs, then answer."
        }
        HarnessEnforcementProfile::Graduated => {
            "Engram graduated final-response check: call obligations(action=doctor, scope={relevance_mode:\"related\", project:..., cwd:...}), then resolve or explicitly skip open obligations. Claude MCP Stop enforcement blocks once when open obligations remain."
        }
        HarnessEnforcementProfile::Strict => {
            "Engram strict final-response check: call obligations(action=doctor, scope={relevance_mode:\"related\", project:..., cwd:...}), then resolve or explicitly skip open obligations. Claude MCP Stop enforcement keeps blocking while obligations remain."
        }
    };
    format!(
        r#"#!/usr/bin/env bash
{MARKER_SH}
set -euo pipefail

INPUT=$(cat)
STOP_HOOK_ACTIVE=$(printf '%s' "$INPUT" | jq -r '.stop_hook_active // false')

if [ "$STOP_HOOK_ACTIVE" = "true" ]; then
  cat <<'EOF'
{{
  "continue": true,
  "systemMessage": "Engram final-response check already ran for this Stop turn."
}}
EOF
  exit 0
fi

cat <<'EOF'
{{
  "continue": true,
  "systemMessage": "{final_message}"
}}
EOF
"#
    )
}

fn claude_session_end_hook(enforcement_profile: HarnessEnforcementProfile) -> String {
    let body = r#"set -euo pipefail

if ! command -v jq >/dev/null 2>&1; then
  printf '%s\n' '{"continue":true,"systemMessage":"Engram SessionEnd hook skipped: jq is unavailable."}'
  exit 0
fi

fallback() {
  local message="$1"
  jq -n --arg message "$message" '{continue: true, systemMessage: $message}'
}

if ! command -v curl >/dev/null 2>&1; then
  fallback "Engram SessionEnd hook skipped: curl is unavailable."
  exit 0
fi

INPUT=$(cat)
CWD=$(printf '%s' "$INPUT" | jq -r '.cwd // empty')
SESSION_ID=$(printf '%s' "$INPUT" | jq -r '.session_id // empty')
TRANSCRIPT_PATH=$(printf '%s' "$INPUT" | jq -r '.transcript_path // empty')
REASON=$(printf '%s' "$INPUT" | jq -r '.reason // empty')
WRITE_POLICY=$(printf '%s' "$INPUT" | jq -r '.write_policy // "nudge"')
ENFORCEMENT_PROFILE="{enforcement_profile}"

if [ -z "$CWD" ] || [ "$CWD" = "null" ]; then
  CWD="${CLAUDE_PROJECT_DIR:-}"
fi

PORT_FILE="${ENGRAM_DAEMON_PORT_FILE:-$HOME/.engram/daemon.port}"
if [ ! -r "$PORT_FILE" ]; then
  fallback "Engram SessionEnd handoff skipped: daemon port file was not found."
  exit 0
fi

PORT=$(tr -d '[:space:]' < "$PORT_FILE")
if [ -z "$PORT" ]; then
  fallback "Engram SessionEnd handoff skipped: daemon port file was empty."
  exit 0
fi

MCP_URL="http://127.0.0.1:${PORT}/mcp"
HEADERS=$(mktemp)
trap 'rm -f "$HEADERS"' EXIT

INIT_PAYLOAD=$(jq -nc '{jsonrpc:"2.0",id:1,method:"initialize",params:{protocolVersion:"2024-11-05",capabilities:{},clientInfo:{name:"engram-claude-session-end-hook",version:"1.0"}}}')
if ! curl -sS --max-time 5 -D "$HEADERS" -H 'Content-Type: application/json' -H 'Accept: application/json, text/event-stream' -X POST "$MCP_URL" -d "$INIT_PAYLOAD" >/dev/null; then
  fallback "Engram SessionEnd handoff skipped: could not initialize MCP session with daemon."
  exit 0
fi

MCP_SESSION_ID=$(awk 'tolower($1)=="mcp-session-id:" {print $2}' "$HEADERS" | tr -d '\r')
if [ -z "$MCP_SESSION_ID" ]; then
  fallback "Engram SessionEnd handoff skipped: daemon did not return an MCP session id."
  exit 0
fi

curl -sS --max-time 5 -H 'Content-Type: application/json' -H 'Accept: application/json, text/event-stream' -H "mcp-session-id: $MCP_SESSION_ID" -X POST "$MCP_URL" -d '{"jsonrpc":"2.0","method":"notifications/initialized"}' >/dev/null || true

CALL_PAYLOAD=$(jq -nc \
  --arg session_id "$SESSION_ID" \
  --arg cwd "$CWD" \
  --arg transcript_path "$TRANSCRIPT_PATH" \
  --arg reason "$REASON" \
  --arg write_policy "$WRITE_POLICY" \
  --arg enforcement "$ENFORCEMENT_PROFILE" \
  '{jsonrpc:"2.0",id:2,method:"tools/call",params:{name:"harness",arguments:{action:"hook_event",harness:"claude_code",enforcement:$enforcement,hook_event_name:"SessionEnd",session_id:$session_id,cwd:$cwd,transcript_path:$transcript_path,reason:$reason,write_policy:$write_policy,model_provider:"anthropic",model:"claude-code",surface:"claude-code",actor:"agent"}}}')

if ! CALL_RESPONSE=$(curl -sS --max-time 10 -H 'Content-Type: application/json' -H 'Accept: application/json, text/event-stream' -H "mcp-session-id: $MCP_SESSION_ID" -X POST "$MCP_URL" -d "$CALL_PAYLOAD"); then
  fallback "Engram SessionEnd handoff skipped: harness hook_event call failed."
  exit 0
fi

HOOK_JSON=$(printf '%s' "$CALL_RESPONSE" | sed -n 's/^data: //p' | jq -rs -r 'map(select(type=="object" and (.result? != null)))[0].result.content[0].text // ""' 2>/dev/null || true)
if [ -z "$HOOK_JSON" ] || ! printf '%s' "$HOOK_JSON" | jq -e . >/dev/null 2>&1; then
  fallback "Engram SessionEnd handoff attempted, but daemon returned an unreadable hook response."
  exit 0
fi

printf '%s\n' "$HOOK_JSON"
"#;

    let body = body.replace("{enforcement_profile}", &enforcement_profile.to_string());
    format!("#!/usr/bin/env bash\n{MARKER_SH}\n{body}")
}

fn claude_settings_snippet(enforcement_profile: HarnessEnforcementProfile) -> String {
    let mut hooks = serde_json::Map::new();
    hooks.insert(
        "SessionStart".to_string(),
        json!([{
            "matcher": "startup|resume|compact",
            "hooks": [{
                "type": "command",
                "command": CLAUDE_HOOK_COMMAND,
                "timeout": 10
            }]
        }]),
    );
    for (event, matcher) in claude_mcp_hook_events(enforcement_profile) {
        let entry = hooks.entry(event.to_string()).or_insert_with(|| json!([]));
        let mut group = json!({
            "hooks": [claude_mcp_hook_handler(event, enforcement_profile)]
        });
        if let Some(matcher) = matcher {
            group["matcher"] = Value::String(matcher.to_string());
        }
        entry
            .as_array_mut()
            .expect("generated hook group must be an array")
            .push(group);
    }
    hooks.insert(
        "SessionEnd".to_string(),
        json!([{
            "hooks": [{
                "type": "command",
                "command": CLAUDE_SESSION_END_HOOK_COMMAND,
                "timeout": 15
            }]
        }]),
    );

    serde_json::to_string_pretty(&json!({
        "permissions": {
            "allow": claude_required_permissions()
        },
        "hooks": hooks
    }))
    .expect("Claude settings snippet should serialize")
}

fn codex_memory_session_skill(enforcement_profile: HarnessEnforcementProfile) -> String {
    let contract = harness_contract_sentence(enforcement_profile, false);
    format!(
        r#"---
name: engram-memory-session
description: Use whenever Codex is asked to recall or run an earlier, previous, remembered, or learned procedure, or needs scoped persistent engineering context, durable decisions, or session handoffs from Engram.
---
{MARKER_MD}
# Engram Memory Session

Use when Codex is working in a repo or project with persistent Engram memory.

Workflow ({enforcement_profile}):
- For actionable repository work, use the procedure route below as the task-start identity
  boundary. For other tasks, start by calling `orient` with current cwd, prompt, `agent=codex`, and
  `response_shape="lean"`; supply project only when its canonical identity is known.
- {TASK_SCOPE_GUIDANCE}
- If project/repository resolution is ambiguous, stop and ask the user instead of applying memory.
- {CONCRETE_MEMORY_TRIGGERS}
- {VERIFIED_PROCEDURE_GUIDANCE}
- {SCOPED_SEARCH_GUIDANCE}
- {DURABLE_CAPTURE_GUIDANCE}
- Use `memory(action=archive)` for obsolete history. Use `memory(action=forget)` only for an exact
  item ID after explicit confirmation because deletion is irreversible.
- For commit messages, check memory for user/project commit preferences first.

{contract}
"#
    )
}

fn codex_session_start_hook(enforcement_profile: HarnessEnforcementProfile) -> String {
    let contract = harness_contract_sentence(enforcement_profile, false);
    format!(
        r#"#!/usr/bin/env bash
{MARKER_SH}
set -u

while IFS= read -r _engram_hook_input; do :; done

printf '%s\n' \
  'Engram startup context (advisory):' \
  '- For non-actionable work, call Engram MCP orient before repository shell exploration with the current cwd and prompt, agent=codex, response_shape=lean; supply project only when its canonical identity is known.' \
  '- Every actionable repository task uses one bounded local procedure match as its task-start identity boundary, even without remembered/learned wording. Call memory(action=procedure_match, query=<a task-focused excerpt of at most 512 characters copied from the current user request>, {LOCAL_PROCEDURE_SCOPE_EXAMPLE}, conditions=<only exact already-observed unsourced prerequisites>). Preserve concrete operation nouns, identifiers, and failure text verbatim; omit unrelated meta/output instructions and secret values. The query is retrieval text, not an authorization channel. memory(action=list) is not a substitute, and do not call orient first solely to obtain cwd.' \
  '- Pass the host exact current cwd to procedure_match; do not replace it with the repository checkout root. The match response supplies structured repository, project, and component identity.' \
  '- Source-backed prerequisites are read deterministically by Engram from Git-tracked files in current_checkout_root and reported as value-redacted condition_observations; caller text cannot override them.' \
  '- Execute only a returned verified procedure. If rejected for unsourced prerequisites, do not apply it: use required_condition_keys and next_actions, search authoritative repository files from the returned current_checkout_root, read each located value, then retry with exact observed values.' \
  '- An empty procedures list proves only that no applicable verified procedure matched. Use only non-null fields in the returned structured identity. Treat suggested_operation_evidence as a required host-action protocol, not an optional suggestion. When required_before_final_abstention=true, the next tool call must read its absolute resolved_path unchanged exactly once, before interpreting abstained, asking for project confirmation, or returning final output, including for durable-memory-only requests. allowed_when_project_requires_confirmation=true authorizes only that repository-local evidence read; authorizes_procedure_execution=false forbids executing a remembered procedure. Treat the checkout-relative path as provenance only. Otherwise make at most one bounded read-only operation lookup. Do not re-read identity files. Do not re-derive a project or broaden outside the checkout.' \
  '- Execute repository-scoped commands from current_checkout_root. Stored procedure scope.local_path and evidence paths are provenance only and must never redirect execution to an older checkout.' \
  '- Never silently apply another project or repository guidance. Stop and ask when repository/project resolution remains materially ambiguous.' \
  '- {contract}'
"#
    )
}

fn codex_hooks_config() -> String {
    serde_json::to_string_pretty(&json!({
        "description": format!(
            "{MARKER_SH}\nDeterministic, advisory Engram orientation and verified-procedure startup context."
        ),
        "hooks": {
            "SessionStart": [{
                "matcher": "startup|resume|clear",
                "hooks": [{
                    "type": "command",
                    "command": CODEX_HOOK_COMMAND,
                    "timeout": 10,
                    "statusMessage": "Loading Engram startup context",
                    "additionalContextLimit": 3000
                }]
            }]
        }
    }))
    .expect("Codex hooks config should serialize")
}

fn codex_resume_session_skill() -> String {
    format!(
        r#"---
name: engram-resume-session
description: Use when Codex resumes or continues repository work using Engram orientation, current plans, and handoffs.
---
{MARKER_MD}
# Engram Resume Session

Use when the user asks to continue, resume, or load prior Engram context.

Steps:
- Call `orient` with `response_shape="lean"` before reading broad files.
- {TASK_SCOPE_GUIDANCE}
- Inspect project/repository resolution and ask only if ambiguity cannot be resolved.
- Use scoped `search` only when the compact orientation lacks required evidence.
- For every actionable repository task, call `memory(action=procedure_match, query=<a task-focused
  excerpt of at most 512 characters copied from the current user request>,
  {LOCAL_PROCEDURE_SCOPE_EXAMPLE}, conditions=...)`; preserve concrete operation nouns,
  identifiers, and failure text verbatim, while omitting unrelated meta/output instructions and
  secret values. The query is retrieval text, not an authorization channel.
  `memory(action=list)` is not a substitute. Copy the exact cwd returned by `orient`; use exact observed conditions and execute
  only a returned verified match. On no-result, read exactly one
  absolute `suggested_operation_evidence.resolved_path` unchanged when present. If
  `required_before_final_abstention=true`, do that before interpreting `abstained`, asking for
  project confirmation, or returning final output; this evidence read does not authorize procedure
  execution. Treat the checkout-relative `path` as provenance only and otherwise keep the bounded
  local fallback.
- Store only compact, evidenced durable memory that a future session genuinely needs. A handoff is
  a project- or repository-scoped `kind=handoff` memory with concrete next actions, not a transcript.
- Never store secrets. Archive obsolete memory; permanently forget only after exact confirmation.
"#
    )
}

fn gemini_memory_session_command(enforcement_profile: HarnessEnforcementProfile) -> String {
    let contract = harness_contract_sentence(enforcement_profile, false);
    format!(
        r#"{MARKER_SH}
description = "Follow the Engram Memory OS lifecycle contract."
prompt = """
# Engram Memory Session

You are Gemini CLI working in a repository or project with persistent Engram memory.
This command is invoked as `/engram:memory-session`.

Follow this {enforcement_profile} lifecycle contract:
- Start by calling the Engram MCP `orient` tool with current cwd, prompt, `agent=gemini_cli`, and
  `response_shape="lean"`; supply project only when its canonical identity is known.
- {TASK_SCOPE_GUIDANCE}
- {CONCRETE_MEMORY_TRIGGERS}
- Treat the returned memory cursor as the baseline for this turn.
- Keep the returned `trace_id` from `orient` or `search`; before final response, call
  `telemetry(action=submit_feedback)` with `task_success`, `preference_adhered`,
  `repeated_context_questions`, `bad_memory_used`, `missing_context`, `used_memory_ids`, and
  `rejected_memory_ids`, plus `stale_memory_ids` and `wrong_scope_memory_ids` when those
  outcomes or attribution judgments can be made. Use `used_memory_ids` for returned memory that
  shaped the answer, implementation, safety decision, or plan; leave it empty only when no returned
  memory influenced behavior.
- Before a major decision or final response, call `memory(action=changes_since,
  commit_id=<memory_cursor.commit_id>, timestamp=<memory_cursor.timestamp>,
  {RELATED_RETRIEVAL_SCOPE_EXAMPLE})`.
- Record source-grounded discoveries, decisions, rules, preferences, limitations, and handoffs.
- When the current method, plan, or next action should survive resume, use
  `memory(action=capture_current_plan)` with compact content and file/tool/manual-review evidence.
- Use `obligations(action=detect, project=..., cwd=...)` when documents change, tools fail,
  or source/design reading is needed; before final response, run
  `obligations(action=doctor, {RELATED_RETRIEVAL_SCOPE_EXAMPLE})` and resolve or explicitly skip open
  obligations.
- Before context compaction or any expected context loss, update `handoff` and record or commit
  compact durable memory for the next session.
- For commit messages, check memory for user/project commit preferences first.
- If handoff or durable memory changes are needed, use `handoff` and `memory(action=commit)`.

{contract}
"""
"#
    )
}

fn gemini_resume_session_command() -> String {
    format!(
        r#"{MARKER_SH}
description = "Resume project work from Engram memory and handoffs."
prompt = """
# Resume Engram Session

You are Gemini CLI resuming work with persistent Engram memory.
This command is invoked as `/engram:resume-session`.

Steps:
- Call the Engram MCP `orient` tool with `response_shape="lean"` before reading broad files.
- Inspect project/repository resolution and ask only if ambiguity cannot be resolved.
- Keep returned `trace_id` values from `orient` or `search`; submit telemetry feedback with
  outcome, gap, and attribution fields before final response when memory quality can be judged.
  Include `used_memory_ids` for returned memory that shaped behavior and `rejected_memory_ids` for
  returned memory considered but not used. Include `stale_memory_ids` and
  `wrong_scope_memory_ids` for rejected memory specifically judged stale or out of scope.
- Use `handoff(action=get, {RELATED_RETRIEVAL_SCOPE_EXAMPLE})` when available.
- Poll `memory(action=changes_since, commit_id=<memory_cursor.commit_id>,
  timestamp=<memory_cursor.timestamp>, {RELATED_RETRIEVAL_SCOPE_EXAMPLE})` before major decisions
  and final response.
- Poll `obligations(action=detect)` and close or explicitly skip open obligations before final
  response.
- Store compact, evidenced memory if the session discovered something future agents need.
- Use `memory(action=capture_current_plan)` for compact current method, plan, or next-action
  guidance that should surface on the next resume.
- If resuming after compaction, read `handoff(action=get, {RELATED_RETRIEVAL_SCOPE_EXAMPLE})` and
  recent `memory(action=changes_since, commit_id=<memory_cursor.commit_id>,
  timestamp=<memory_cursor.timestamp>, {RELATED_RETRIEVAL_SCOPE_EXAMPLE})` before continuing.
"""
"#
    )
}

fn gemini_end_session_command() -> String {
    format!(
        r#"{MARKER_SH}
description = "Compile a Memory OS handoff and knowledge commit candidate."
prompt = """
# End Engram Session

You are Gemini CLI closing out work with persistent Engram memory.
This command is invoked as `/engram:end-session`.

Before ending:
- Call `memory(action=changes_since, commit_id=<memory_cursor.commit_id>,
  timestamp=<memory_cursor.timestamp>, {RELATED_RETRIEVAL_SCOPE_EXAMPLE})` from the latest cursor.
- Call `obligations(action=detect, project=..., cwd=...)` and
  `obligations(action=doctor, {RELATED_RETRIEVAL_SCOPE_EXAMPLE})`.
- Resolve open obligations or state explicit skip reasons in the handoff.
- Update or compile `handoff` with completed work, open decisions, next actions, and risks.
- If durable memory changed, prepare a `memory(action=commit)` candidate.
- Use this same flow before context compaction or any context transition.
- Leave migration and digest promotions review-gated; do not auto-promote orphan data.
"""
"#
    )
}

fn gemini_global_context(enforcement_profile: HarnessEnforcementProfile) -> String {
    let contract = harness_contract_sentence(enforcement_profile, false);
    format!(
        r#"{MARKER_MD}
# Engram Memory OS Harness

Gemini CLI should treat Engram as persistent project memory when Engram MCP tools are available.

- Start work by calling `orient` with current cwd, prompt, `agent=gemini_cli`, and
  `response_shape="lean"`; supply project only when its canonical identity is known.
- {CONCRETE_MEMORY_TRIGGERS}
- Keep the returned memory cursor and call `memory(action=changes_since,
  commit_id=<memory_cursor.commit_id>, timestamp=<memory_cursor.timestamp>,
  {RELATED_RETRIEVAL_SCOPE_EXAMPLE})` before major decisions, before final response, and during
  long sessions.
- Keep returned `trace_id` values from `orient` or `search` and call
  `telemetry(action=submit_feedback)` with `task_success`, `preference_adhered`,
  `repeated_context_questions`, `bad_memory_used`, `missing_context`, `used_memory_ids`, and
  `rejected_memory_ids`, plus `stale_memory_ids` and `wrong_scope_memory_ids` before final
  response when those outcomes or attribution judgments can be made. Use `used_memory_ids` for
  returned memory that shaped the answer, implementation, safety decision, or plan; leave it empty
  only when no returned memory influenced behavior.
- Record source-grounded decisions, preferences, rules, limitations, and non-obvious
  discoveries. Use writer provenance so Gemini CLI, Claude Code, Codex, and other harnesses
  can be distinguished.
- Use `memory(action=capture_current_plan)` when the current method, plan, or next action should
  survive resume.
- Detect and close agent obligations before final response: document dispositions, failed tool
  recovery, source/design reading, verification, handoff, and commit-preference checks.
- Before context compaction or expected context loss, update `handoff` and record or commit
  compact durable memory.
- Maintain rolling handoffs for multi-turn work. Handoffs must include next actions.
- Keep migration review-gated. Do not auto-promote orphan, digest, or legacy data.
- {contract}

Useful commands when installed:
- `/engram:memory-session`
- `/engram:resume-session`
- `/engram:end-session`
"#
    )
}

fn cursor_memory_session_skill(enforcement_profile: HarnessEnforcementProfile) -> String {
    let contract = harness_contract_sentence(enforcement_profile, false);
    format!(
        r#"{MARKER_MD}
---
name: engram-memory-session
description: Use when Cursor Agent is working in a repo or project with persistent Engram memory, especially at task start, before major decisions, before final responses, or when commit preferences may matter.
---
# Engram Memory Session

Use this skill when Cursor Agent is working in a repository or project with persistent Engram memory.

Workflow ({enforcement_profile}):
- Start by calling the Engram MCP `orient` tool with current cwd, prompt, `agent=cursor`, and
  `response_shape="lean"`; supply project only when its canonical identity is known.
- {TASK_SCOPE_GUIDANCE}
- {CONCRETE_MEMORY_TRIGGERS}
- Treat the returned memory cursor as the baseline for this turn.
- Keep the returned `trace_id` from `orient` or `search`; before final response, call
  `telemetry(action=submit_feedback)` with `task_success`, `preference_adhered`,
  `repeated_context_questions`, `bad_memory_used`, `missing_context`, `used_memory_ids`, and
  `rejected_memory_ids`, plus `stale_memory_ids` and `wrong_scope_memory_ids` when those
  outcomes or attribution judgments can be made. Use `used_memory_ids` for returned memory that
  shaped the answer, implementation, safety decision, or plan; leave it empty only when no returned
  memory influenced behavior.
- Before a major decision or final response, call `memory(action=changes_since,
  commit_id=<memory_cursor.commit_id>, timestamp=<memory_cursor.timestamp>,
  {RELATED_RETRIEVAL_SCOPE_EXAMPLE})`.
- Record source-grounded discoveries, decisions, rules, preferences, limitations, and handoffs.
- When the current method, plan, or next action should survive resume, use
  `memory(action=capture_current_plan)` with compact content and file/tool/manual-review evidence.
- Use `obligations(action=detect, project=..., cwd=...)` when documents change, tools fail,
  or source/design reading is needed; before final response, run
  `obligations(action=doctor, {RELATED_RETRIEVAL_SCOPE_EXAMPLE})` and resolve or explicitly skip open
  obligations.
- Before context compaction or any expected context loss, update `handoff` and record or commit
  compact durable memory for the next session.
- Use writer provenance with `writer_harness=cursor` when writing durable memory.
- For commit messages, check memory for user/project commit preferences first.
- If handoff or durable memory changes are needed, use `handoff` and `memory(action=commit)`.

{contract}
"#
    )
}

fn cursor_resume_session_skill() -> String {
    format!(
        r#"{MARKER_MD}
---
name: engram-resume-session
description: Use when Cursor Agent resumes, continues, or loads prior project context from Engram memory and rolling handoffs.
---
# Engram Resume Session

Use this skill when the user asks Cursor Agent to continue, resume, or load prior Engram context.

Steps:
- Call the Engram MCP `orient` tool with `response_shape="lean"` before reading broad files.
- {TASK_SCOPE_GUIDANCE}
- Inspect project/repository resolution and ask only if ambiguity cannot be resolved.
- Keep returned `trace_id` values from `orient` or `search`; submit telemetry feedback with
  outcome, gap, and attribution fields before final response when memory quality can be judged.
  Include `used_memory_ids` for returned memory that shaped behavior and `rejected_memory_ids` for
  returned memory considered but not used. Include `stale_memory_ids` and
  `wrong_scope_memory_ids` for rejected memory specifically judged stale or out of scope.
- Use `handoff(action=get, {RELATED_RETRIEVAL_SCOPE_EXAMPLE})` when available.
- Poll `memory(action=changes_since, commit_id=<memory_cursor.commit_id>,
  timestamp=<memory_cursor.timestamp>, {RELATED_RETRIEVAL_SCOPE_EXAMPLE})` before major decisions
  and final response.
- Poll `obligations(action=detect)` and close or explicitly skip open obligations before final
  response.
- Store compact, evidenced memory if the session discovered something future agents need.
- Use `memory(action=capture_current_plan)` for compact current method, plan, or next-action
  guidance that should surface on the next resume.
- If resuming after compaction, read `handoff(action=get, {RELATED_RETRIEVAL_SCOPE_EXAMPLE})` and
  recent `memory(action=changes_since, commit_id=<memory_cursor.commit_id>,
  timestamp=<memory_cursor.timestamp>, {RELATED_RETRIEVAL_SCOPE_EXAMPLE})` before continuing.
- Use writer provenance with `writer_harness=cursor` for durable memory writes.
"#
    )
}

fn cursor_end_session_skill() -> String {
    format!(
        r#"{MARKER_MD}
---
name: engram-end-session
description: Use when Cursor Agent is closing out work, preparing a handoff, or recording durable Memory OS changes.
---
# Engram End Session

Use this skill when Cursor Agent is closing out a task or preparing a handoff.

Before ending:
- Call `memory(action=changes_since, commit_id=<memory_cursor.commit_id>,
  timestamp=<memory_cursor.timestamp>, {RELATED_RETRIEVAL_SCOPE_EXAMPLE})` from the latest cursor.
- Call `obligations(action=detect, project=..., cwd=...)` and
  `obligations(action=doctor, {RELATED_RETRIEVAL_SCOPE_EXAMPLE})`.
- Resolve open obligations or state explicit skip reasons in the handoff.
- Update or compile `handoff` with completed work, open decisions, next actions, and risks.
- If durable memory changed, prepare a `memory(action=commit)` candidate.
- Use this same flow before context compaction or any context transition.
- Use writer provenance with `writer_harness=cursor`.
- Leave migration and digest promotions review-gated; do not auto-promote orphan data.
"#
    )
}

fn agents_snippet(enforcement_profile: HarnessEnforcementProfile) -> String {
    let contract = harness_contract_sentence(enforcement_profile, false);
    format!(
        r#"{MARKER_MD}
# Engram Memory OS Harness

- Start work by calling `orient` with current cwd, prompt, harness name, and
  `response_shape="lean"`; supply project only when its canonical identity is known.
- {TASK_SCOPE_GUIDANCE}
- {CONCRETE_MEMORY_TRIGGERS}
- {VERIFIED_PROCEDURE_GUIDANCE}
- Keep the returned memory cursor and call `memory(action=changes_since,
  commit_id=<memory_cursor.commit_id>, timestamp=<memory_cursor.timestamp>,
  {RELATED_RETRIEVAL_SCOPE_EXAMPLE})` before major decisions, before final response, and during
  long sessions.
- Keep returned `trace_id` values from `orient` or `search` and call
  `telemetry(action=submit_feedback)` with `task_success`, `preference_adhered`,
  `repeated_context_questions`, `bad_memory_used`, `missing_context`, `used_memory_ids`, and
  `rejected_memory_ids`, plus `stale_memory_ids` and `wrong_scope_memory_ids` before final
  response when those outcomes or attribution judgments can be made. Use `used_memory_ids` for
  returned memory that shaped the answer, implementation, safety decision, or plan; leave it empty
  only when no returned memory influenced behavior.
- Record source-grounded decisions, preferences, rules, limitations, and non-obvious
  discoveries. Use writer provenance so Claude Code, Codex, and other harnesses can be
  distinguished.
- Use `memory(action=capture_current_plan)` when the current method, plan, or next action should
  survive resume.
- Use `obligations(action=detect)` at task start and before final response. Resolve or explicitly
  skip document, failed-tool, source/design reading, verification, handoff, and commit-preference
  obligations before claiming the task is done.
- Before context compaction or expected context loss, update `handoff` and record or commit
  compact durable memory for the next session.
- Maintain rolling handoffs for multi-turn work. Handoffs must include next actions.
- Keep migration review-gated. Do not auto-promote orphan, digest, or legacy data.
- {contract}
"#
    )
}

fn generic_policy_document(enforcement_profile: HarnessEnforcementProfile) -> String {
    let contract = harness_contract_sentence(enforcement_profile, false);
    format!(
        r#"{MARKER_MD}
# Engram Generic Harness Policy

Required MCP tools: orient, memory, harness, lint, graph, handoff, obligations, telemetry, vault.

Lifecycle ({enforcement_profile}):
- task/session start: call `orient` with `response_shape="lean"`
- {TASK_SCOPE_GUIDANCE}
- {CONCRETE_MEMORY_TRIGGERS}
- before major decisions: call `memory(action=changes_since,
  commit_id=<memory_cursor.commit_id>, timestamp=<memory_cursor.timestamp>,
  {RELATED_RETRIEVAL_SCOPE_EXAMPLE})`
- after non-obvious discoveries: record memory/session event
- after current method/plan/next-action changes: use `memory(action=capture_current_plan)` with
  compact content and evidence
- before final response: call `changes_since` and distill if needed
- before final response: detect obligations with current project/cwd, run obligations doctor with
  the same project/cwd scope, and close or explicitly skip open obligations
- before final response: submit `telemetry(action=submit_feedback)` for relevant `trace_id`
  values with outcome, gap, and attribution fields when memory quality can be judged; include
  `used_memory_ids` for returned memory that shaped behavior and `rejected_memory_ids` for returned
  memory considered but not used; include `stale_memory_ids` and `wrong_scope_memory_ids` for
  rejected memory specifically judged stale or out of scope
- before context compaction/context loss: update handoff and persist compact durable memory
- session end/handoff: compile handoff and knowledge commit candidate
- commit workflows: consult memory for relevant preferences/rules

{contract}
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_server_check_redacts_environment_values_and_attests_agent_profile_shape() {
        let root = tempfile::tempdir().unwrap();
        let executable = root.path().join("engram");
        fs::write(&executable, b"fixture executable").unwrap();
        let server = json!({
            "type": "stdio",
            "command": executable,
            "args": [
                "serve", "--project", "demo", "--profile", "agent",
                "--api-token", "super-secret-value"
            ],
            "env": {
                "ENGRAM_TOKEN": "must-never-appear",
                "ENGRAM_HOME": "/private/path"
            }
        });

        let check = build_mcp_server_check(
            "static_config",
            Some("fixture.json#/mcpServers/engram".to_string()),
            &server,
            None,
            false,
            "fixture",
        );

        assert!(check.agent_profile_launch_configured);
        assert_eq!(check.env_keys, vec!["ENGRAM_HOME", "ENGRAM_TOKEN"]);
        assert_eq!(check.args.last().map(String::as_str), Some("<redacted>"));
        assert!(check.executable_sha256.is_some());
        assert!(!check.resolved_configuration_verified);
        assert!(!check.running_host_loaded_verified);
        assert!(!check.live_runtime_verified);
        let serialized = serde_json::to_string(&check).unwrap();
        assert!(!serialized.contains("must-never-appear"));
        assert!(!serialized.contains("/private/path"));
        assert!(!serialized.contains("super-secret-value"));
    }

    #[test]
    fn mcp_server_check_rejects_full_or_non_stdio_launch_shape() {
        let root = tempfile::tempdir().unwrap();
        let executable = root.path().join("engram");
        fs::write(&executable, b"fixture executable").unwrap();

        for server in [
            json!({
                "type": "stdio",
                "command": executable,
                "args": ["serve"]
            }),
            json!({
                "type": "stdio",
                "command": executable,
                "args": ["serve", "--profile", "full"]
            }),
            json!({
                "type": "stdio",
                "command": executable,
                "args": ["serve", "--profile", "agent", "--http"]
            }),
            json!({
                "type": "http",
                "command": executable,
                "args": ["serve", "--profile", "agent"]
            }),
        ] {
            let check = build_mcp_server_check(
                "host_cli_resolved",
                None,
                &server,
                Some(true),
                true,
                "fixture",
            );
            assert!(!check.agent_profile_launch_configured, "{server}");
        }
    }

    #[test]
    fn claude_static_config_prefers_local_then_project_then_user() {
        let cwd = "/workspace/demo";
        let user_path = Path::new("/home/test/.claude.json");
        let project_path = Path::new("/workspace/demo/.mcp.json");
        let user = json!({
            "mcpServers": {"engram": {"command": "user-engram"}},
            "projects": {
                (cwd): {"mcpServers": {"engram": {"command": "local-engram"}}}
            }
        });
        let project = json!({
            "mcpServers": {"engram": {"command": "project-engram"}}
        });

        let candidates =
            claude_mcp_candidates(Some(&user), Some(&project), cwd, user_path, project_path);

        assert_eq!(candidates.len(), 3);
        assert_eq!(candidates[0].1["command"], "local-engram");
        assert_eq!(candidates[1].1["command"], "project-engram");
        assert_eq!(candidates[2].1["command"], "user-engram");
        assert!(candidates[0].0.contains("#/projects/~1workspace~1demo/"));
    }

    fn assert_retrieval_calls_declare_scope(contents: &str, marker: &str, scope: &str) {
        let mut remaining = contents;
        while let Some(start) = remaining.find(marker) {
            let call = &remaining[start..];
            let end = call
                .find(')')
                .unwrap_or_else(|| panic!("unterminated generated call starting with {marker}"));
            assert!(
                call[..=end].contains(scope),
                "generated call lacks required scope: {}",
                &call[..=end]
            );
            remaining = &call[end + 1..];
        }
    }

    #[test]
    fn status_reports_missing_required_codex_adapters() {
        let root = tempfile::tempdir().unwrap();
        let report = HarnessService::new()
            .status(HarnessKind::Codex, Some(root.path()), &[])
            .unwrap();

        assert!(!report.ready);
        assert!(report
            .adapters
            .iter()
            .any(|check| check.status == HarnessAdapterStatus::Missing));
    }

    #[test]
    fn status_reports_missing_required_gemini_adapters() {
        let root = tempfile::tempdir().unwrap();
        let report = HarnessService::new()
            .status(HarnessKind::GeminiCli, Some(root.path()), &[])
            .unwrap();

        assert!(!report.ready);
        assert_eq!(report.harness, HarnessKind::GeminiCli);
        assert!(report
            .adapters
            .iter()
            .filter(|check| check.required)
            .all(|check| check.status == HarnessAdapterStatus::Missing));
    }

    #[test]
    fn status_reports_missing_required_cursor_adapters() {
        let root = tempfile::tempdir().unwrap();
        let report = HarnessService::new()
            .status(HarnessKind::Cursor, Some(root.path()), &[])
            .unwrap();

        assert!(!report.ready);
        assert_eq!(report.harness, HarnessKind::Cursor);
        assert!(report
            .adapters
            .iter()
            .filter(|check| check.required)
            .all(|check| check.status == HarnessAdapterStatus::Missing));
    }

    #[test]
    fn dry_run_install_writes_nothing() {
        let root = tempfile::tempdir().unwrap();
        let report = HarnessService::new()
            .install(HarnessKind::Codex, Some(root.path()), false)
            .unwrap();

        assert!(report.dry_run);
        assert!(report.written.is_empty());
        assert!(!root.path().join(".codex").exists());
    }

    #[test]
    fn doctor_names_soft_lifecycle_triggers_when_ready() {
        let root = tempfile::tempdir().unwrap();
        let service = HarnessService::new();
        service
            .install(HarnessKind::Codex, Some(root.path()), true)
            .unwrap();

        let report = service
            .doctor(HarnessKind::Codex, Some(root.path()), &[])
            .unwrap();

        assert!(report.ready);
        assert_eq!(
            report.lifecycle.enforcement_profile,
            HarnessEnforcementProfile::Soft
        );
        assert!(report.lifecycle.soft_contract);
        assert!(!report.lifecycle.enforced);
        assert_eq!(
            report.lifecycle.advisory_triggers,
            vec![
                HarnessLifecycleTrigger::TaskStartOrient,
                HarnessLifecycleTrigger::BeforeMajorDecisionChangesSince,
                HarnessLifecycleTrigger::AfterDiscoveryRecord,
                HarnessLifecycleTrigger::BeforeFinalChangesSince,
                HarnessLifecycleTrigger::BeforeFinalObligations,
                HarnessLifecycleTrigger::BeforeContextCompactionSave,
                HarnessLifecycleTrigger::SessionEndHandoff,
                HarnessLifecycleTrigger::CommitWorkflowConsultMemory,
            ]
        );
        let lifecycle_warning = report
            .warnings
            .iter()
            .find(|warning| warning.contains("enforcement_profile=soft"))
            .expect("ready doctor should name soft lifecycle profile");
        assert!(lifecycle_warning.contains("task_start_orient"));
        assert!(lifecycle_warning.contains("before_final_obligations"));
        assert!(lifecycle_warning.contains("session_end_handoff"));
        assert!(lifecycle_warning.contains("commit_workflow_consult_memory"));
    }

    #[test]
    fn soft_profile_preserves_advisory_lifecycle_report() {
        let root = tempfile::tempdir().unwrap();
        let service = HarnessService::new();
        service
            .install_with_options(
                HarnessKind::Codex,
                Some(root.path()),
                HarnessInstallOptions {
                    write: true,
                    adopt_user_owned: false,
                    settings_target: HarnessSettingsTarget::default(),
                    enforcement_profile: HarnessEnforcementProfile::Soft,
                },
            )
            .unwrap();

        let report = service
            .doctor_with_enforcement(
                HarnessKind::Codex,
                Some(root.path()),
                &[],
                HarnessEnforcementProfile::Soft,
            )
            .unwrap();

        assert!(report.ready);
        assert_eq!(
            report.lifecycle.enforcement_profile,
            HarnessEnforcementProfile::Soft
        );
        assert!(report.lifecycle.soft_contract);
        assert!(!report.lifecycle.enforced);
        assert!(report.lifecycle.message.contains("advisory"));
    }

    #[test]
    fn status_distinguishes_unchecked_from_missing_mcp_tools() {
        let root = tempfile::tempdir().unwrap();
        let service = HarnessService::new();
        service
            .install(HarnessKind::Codex, Some(root.path()), true)
            .unwrap();

        let unchecked = service
            .status(HarnessKind::Codex, Some(root.path()), &[])
            .unwrap();
        assert!(unchecked.ready);
        assert!(!unchecked.mcp_tools.checked);
        assert!(unchecked.mcp_tools.missing_tools.is_empty());
        assert!(unchecked.missing_mcp_tools.is_empty());

        let observed_without_search = vec![
            "orient".to_string(),
            "memory".to_string(),
            "harness".to_string(),
            "repo".to_string(),
            "obligations".to_string(),
        ];
        let checked = service
            .status(
                HarnessKind::Codex,
                Some(root.path()),
                &observed_without_search,
            )
            .unwrap();
        assert!(!checked.ready);
        assert!(checked.mcp_tools.checked);
        assert_eq!(checked.mcp_tools.missing_tools, vec!["search"]);
        assert_eq!(checked.missing_mcp_tools, vec!["search"]);
    }

    #[test]
    fn write_install_creates_generated_adapters() {
        let root = tempfile::tempdir().unwrap();
        let report = HarnessService::new()
            .install(HarnessKind::Codex, Some(root.path()), true)
            .unwrap();

        assert!(!report.dry_run);
        assert!(!report.written.is_empty());
        let skill = root
            .path()
            .join(".codex/skills/engram-memory-session/SKILL.md");
        assert!(skill.exists());
        let contents = fs::read_to_string(skill).unwrap();
        assert!(contents.contains(MARKER_MD));
        assert!(contents.contains("orient"));
        assert!(contents.contains("memory(action=procedure_match"));

        let hook = root.path().join(".codex/hooks/engram-session-start.sh");
        assert!(hook.exists());
        let hook_contents = fs::read_to_string(&hook).unwrap();
        assert!(hook_contents.contains(MARKER_SH));
        assert!(hook_contents.contains("current_checkout_root"));

        let config: Value = serde_json::from_str(
            &fs::read_to_string(root.path().join(".codex/hooks.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            config.pointer("/hooks/SessionStart/0/matcher"),
            Some(&Value::String("startup|resume|clear".to_string()))
        );

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(hook).unwrap().permissions().mode() & 0o111,
                0o111
            );
        }
    }

    #[test]
    fn write_install_creates_gemini_adapters() {
        let root = tempfile::tempdir().unwrap();
        let report = HarnessService::new()
            .install(HarnessKind::GeminiCli, Some(root.path()), true)
            .unwrap();

        assert!(!report.dry_run);
        assert_eq!(report.written.len(), 4);

        let command = root
            .path()
            .join(".gemini/commands/engram/memory-session.toml");
        let context = root.path().join(".gemini/GEMINI.md");
        assert!(command.exists());
        assert!(context.exists());

        let command_contents = fs::read_to_string(command).unwrap();
        assert!(command_contents.contains(MARKER_SH));
        assert!(command_contents.contains("description = "));
        assert!(command_contents.contains("prompt = "));
        assert!(command_contents.contains("/engram:memory-session"));

        let context_contents = fs::read_to_string(context).unwrap();
        assert!(context_contents.contains(MARKER_MD));
        assert!(context_contents.contains("agent=gemini_cli"));
    }

    #[test]
    fn write_install_creates_cursor_adapters() {
        let root = tempfile::tempdir().unwrap();
        let report = HarnessService::new()
            .install(HarnessKind::Cursor, Some(root.path()), true)
            .unwrap();

        assert!(!report.dry_run);
        assert_eq!(report.written.len(), 3);

        let skill = root
            .path()
            .join(".cursor/skills/engram-memory-session/SKILL.md");
        assert!(skill.exists());

        let contents = fs::read_to_string(skill).unwrap();
        assert!(contents.contains(MARKER_MD));
        assert!(contents.contains("name: engram-memory-session"));
        assert!(contents.contains("agent=cursor"));
        assert!(contents.contains("writer_harness=cursor"));
    }

    #[test]
    fn status_detects_installed_generated_adapter() {
        let root = tempfile::tempdir().unwrap();
        let service = HarnessService::new();
        service
            .install(HarnessKind::Codex, Some(root.path()), true)
            .unwrap();

        let report = service
            .status(HarnessKind::Codex, Some(root.path()), &[])
            .unwrap();

        assert!(report
            .adapters
            .iter()
            .filter(|check| check.required)
            .all(|check| check.status == HarnessAdapterStatus::Installed));
    }

    #[test]
    fn write_install_skips_user_owned_file() {
        let root = tempfile::tempdir().unwrap();
        let path = root
            .path()
            .join(".codex/skills/engram-memory-session/SKILL.md");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "user-owned").unwrap();

        let report = HarnessService::new()
            .install(HarnessKind::Codex, Some(root.path()), true)
            .unwrap();

        assert!(report
            .skipped
            .iter()
            .any(|file| file.path == path.display().to_string()));
        assert_eq!(fs::read_to_string(path).unwrap(), "user-owned");
    }

    #[test]
    fn dry_run_install_reports_user_owned_file_as_skipped() {
        let root = tempfile::tempdir().unwrap();
        let path = root
            .path()
            .join(".codex/skills/engram-memory-session/SKILL.md");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "user-owned").unwrap();

        let report = HarnessService::new()
            .install(HarnessKind::Codex, Some(root.path()), false)
            .unwrap();

        assert!(report.dry_run);
        assert!(report.written.is_empty());
        assert!(report
            .skipped
            .iter()
            .any(|file| file.path == path.display().to_string()));
        assert!(report
            .warnings
            .iter()
            .any(|warning| warning.contains("user-owned")));
        assert!(!report
            .planned
            .iter()
            .any(|file| file.path == path.display().to_string()));
        assert_eq!(fs::read_to_string(path).unwrap(), "user-owned");
    }

    #[test]
    fn status_is_not_ready_when_required_adapter_is_user_owned() {
        let root = tempfile::tempdir().unwrap();
        let path = root
            .path()
            .join(".codex/skills/engram-memory-session/SKILL.md");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "user-owned").unwrap();

        let report = HarnessService::new()
            .status(HarnessKind::Codex, Some(root.path()), &[])
            .unwrap();

        assert!(!report.ready);
        assert!(report
            .adapters
            .iter()
            .any(|check| check.status == HarnessAdapterStatus::UserOwned));
    }

    #[test]
    fn render_claude_session_end_hook_defaults_missing_write_policy_to_nudge() {
        let adapters = HarnessService::new()
            .render_adapters(HarnessKind::ClaudeCode, Some("claude-session-end-hook"));
        assert_eq!(adapters.len(), 1);
        let contents = &adapters[0].contents;

        assert!(contents.contains(r#".write_policy // "nudge""#));
        assert!(!contents.contains(r#".write_policy // "durable""#));
    }

    #[test]
    fn render_adapter_mentions_commit_preferences() {
        let adapters = HarnessService::new()
            .render_adapters(HarnessKind::Codex, Some("codex-memory-session-skill"));
        assert_eq!(adapters.len(), 1);
        assert!(adapters[0].contents.contains("commit preferences"));
    }

    #[test]
    fn render_codex_session_start_hook_is_root_aware_and_provenance_safe() {
        let service = HarnessService::new();
        let hooks = service.render_adapters(HarnessKind::Codex, Some("codex-session-start-hook"));
        assert_eq!(hooks.len(), 1);
        for expected in [
            "call Engram MCP orient",
            "memory(action=procedure_match",
            "required_condition_keys",
            "current_checkout_root",
            "provenance only",
            "Never silently apply another project or repository guidance",
        ] {
            assert!(hooks[0].contents.contains(expected), "missing `{expected}`");
        }

        let configs = service.render_adapters(HarnessKind::Codex, Some("codex-hooks-config"));
        assert_eq!(configs.len(), 1);
        let config: Value = serde_json::from_str(&configs[0].contents).unwrap();
        assert_eq!(
            config
                .as_object()
                .unwrap()
                .keys()
                .map(String::as_str)
                .collect::<BTreeSet<_>>(),
            BTreeSet::from(["description", "hooks"])
        );
        assert!(config["description"].as_str().unwrap().contains(MARKER_SH));
        assert_eq!(
            config.pointer("/hooks/SessionStart/0/hooks/0/type"),
            Some(&Value::String("command".to_string()))
        );
        assert_eq!(
            config.pointer("/hooks/SessionStart/0/hooks/0/additionalContextLimit"),
            Some(&json!(3000))
        );
    }

    #[test]
    fn codex_and_claude_adapters_enforce_verified_procedure_abstention() {
        let service = HarnessService::new();
        for (harness, adapter) in [
            (HarnessKind::Codex, "codex-memory-session-skill"),
            (HarnessKind::ClaudeCode, "claude-memory-session-command"),
        ] {
            let adapters = service.render_adapters(harness, Some(adapter));
            assert_eq!(adapters.len(), 1);
            let contents = &adapters[0].contents;
            assert!(contents.contains("memory(action=procedure_match"));
            assert!(contents.contains("Execute only returned procedures"));
            assert!(contents.contains("separate direct tool call"));
            assert!(contents.contains("do not apply the rejected procedure"));
            assert!(contents.contains("execution receipt is verified"));
            assert!(contents.contains("An empty `procedures` list proves only"));
            assert!(contents.contains("structured `identity`"));
            assert!(contents.contains("`identity.project.status`"));
            assert!(contents.contains("`suggested_operation_evidence`"));
            assert!(contents.contains("required host-action protocol"));
            assert!(contents.contains("`required_before_final_abstention=true`"));
            assert!(contents.contains("`allowed_when_project_requires_confirmation=true`"));
            assert!(contents.contains("`authorizes_procedure_execution=false`"));
            assert!(contents.contains("durable-memory-only"));
            assert!(contents.contains("absolute `resolved_path`"));
            assert!(contents.contains("checkout-relative `path` as provenance only"));
            assert!(contents.contains("do not substitute a relative path"));
            assert!(contents.contains("Pass the host's exact current `cwd`"));
            assert!(contents.contains("do not call `orient` first solely to obtain it"));
            assert!(contents.contains("at most one bounded read-only lookup"));
            assert!(contents.contains("`identity.repository.checkout_root`"));
            assert!(contents.contains("Do not re-read identity files"));
            assert!(contents.contains("Do not re-derive a project"));
            assert!(contents.contains("broaden outside the checkout"));
            assert!(contents.contains("execute candidate commands"));
            assert!(contents.contains("`requires_confirmation`"));
        }
        assert!(VERIFIED_PROCEDURE_GUIDANCE.len() < 5_000);
    }

    #[test]
    fn claude_adapters_prioritize_repository_local_procedure_matching() {
        let service = HarnessService::new();
        for adapter in ["claude-memory-session-command", "claude-session-start-hook"] {
            let adapters = service.render_adapters(HarnessKind::ClaudeCode, Some(adapter));
            assert_eq!(adapters.len(), 1);
            let contents = &adapters[0].contents;
            let procedure_route = contents
                .find("Every actionable repository task has a mandatory procedure route")
                .expect("Claude adapter should contain the mandatory procedure route");
            let project_stop = contents
                .find("For requests other than repository-local procedure matching")
                .expect("Claude adapter should qualify the project ambiguity stop");

            assert!(procedure_route < project_stop);
            assert!(contents.contains(
                "Use local procedure matching itself as the task-start identity boundary"
            ));
            assert!(contents.contains("do not call `orient` first solely to obtain it"));
            assert!(contents.contains("The `query` is required"));
            assert!(contents.contains("bounded task-focused excerpt"));
            assert!(contents.contains("must be at most 512 characters"));
            assert!(contents.contains("not an authorization channel"));
            assert!(contents.contains("`memory(action=list)` is never a substitute"));
            assert!(contents.contains("requests to use durable procedure memory, are actionable"));
            assert!(contents.contains("Project ambiguity blocks project/task-scoped memory only"));
        }

        let resume = service.render_adapters(
            HarnessKind::ClaudeCode,
            Some("claude-resume-session-command"),
        );
        assert_eq!(resume.len(), 1);
        assert!(resume[0].contents.contains("project ambiguity alone does"));
        assert!(resume[0]
            .contents
            .contains("not block the repository-local procedure route"));
        assert!(resume[0]
            .contents
            .contains("query=<a bounded task-focused excerpt from the current user request>"));
        assert!(resume[0].contents.contains("The query is required"));
        assert!(resume[0].contents.contains("`memory(action=list)`"));
        assert!(resume[0].contents.contains("is not a substitute"));
    }

    #[test]
    fn primary_harness_adapters_require_exact_task_orientation_when_known() {
        let service = HarnessService::new();
        for (harness, adapter) in [
            (HarnessKind::Codex, "codex-memory-session-skill"),
            (HarnessKind::ClaudeCode, "claude-memory-session-command"),
            (HarnessKind::Cursor, "cursor-memory-session-skill"),
        ] {
            let adapters = service.render_adapters(harness, Some(adapter));
            assert_eq!(adapters.len(), 1);
            let contents = &adapters[0].contents;
            assert!(contents.contains("task=<exact task name or tracker key>"));
            assert!(contents.contains("ask instead of applying task-scoped memory"));
        }
    }

    #[test]
    fn generated_retrieval_calls_declare_authorized_scope() {
        let service = HarnessService::new();
        for harness in [
            HarnessKind::ClaudeCode,
            HarnessKind::Codex,
            HarnessKind::GeminiCli,
            HarnessKind::Cursor,
            HarnessKind::Generic,
        ] {
            for adapter in service.render_adapters_with_enforcement(
                harness,
                None,
                HarnessEnforcementProfile::Graduated,
            ) {
                assert_retrieval_calls_declare_scope(
                    &adapter.contents,
                    "memory(action=procedure_match",
                    LOCAL_PROCEDURE_SCOPE_EXAMPLE,
                );
                for marker in [
                    "memory(action=changes_since",
                    "handoff(action=get",
                    "obligations(action=doctor",
                ] {
                    assert_retrieval_calls_declare_scope(
                        &adapter.contents,
                        marker,
                        RELATED_RETRIEVAL_SCOPE_EXAMPLE,
                    );
                }
            }
        }
    }

    #[test]
    fn claude_session_start_uses_cwd_without_fabricating_project_identity() {
        let hook = claude_session_start_hook(HarnessEnforcementProfile::Soft);

        assert!(hook.contains(r#"cwd=\"$CWD\""#));
        assert!(hook.contains("do not run `pwd` first"));
        assert!(hook.contains("Supply project only when its canonical identity is known"));
        assert!(hook.contains("<<'ENGRAM_CONTEXT'"));
        assert!(!hook.contains("PROJECT_NAME"));
        assert!(!hook.contains("basename \"$CWD\""));
    }

    #[test]
    fn claude_session_start_executes_with_literal_markdown_guidance() {
        use std::io::Write as _;
        use std::process::Stdio;

        let root = tempfile::tempdir().unwrap();
        let hook_path = root.path().join("engram-session-start.sh");
        fs::write(
            &hook_path,
            claude_session_start_hook(HarnessEnforcementProfile::Soft),
        )
        .unwrap();

        let mut child = Command::new("bash")
            .arg(&hook_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(br#"{"cwd":"/tmp/atlas","source":"startup","session_id":"s1"}"#)
            .unwrap();
        let output = child.wait_with_output().unwrap();

        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
        let response: Value = serde_json::from_slice(&output.stdout).unwrap();
        let context = response["systemMessage"].as_str().unwrap();
        assert!(context.contains("`task=<exact task name or tracker key>`"));
        assert!(context.contains("`memory(action=procedure_match"));
        assert!(context.contains("source=\"startup\" cwd=\"/tmp/atlas\""));
    }

    #[test]
    fn codex_skills_have_required_yaml_frontmatter() {
        let service = HarnessService::new();
        for (adapter, expected_name) in [
            ("codex-memory-session-skill", "engram-memory-session"),
            ("codex-resume-session-skill", "engram-resume-session"),
        ] {
            let rendered = service.render_adapters(HarnessKind::Codex, Some(adapter));
            assert_eq!(rendered.len(), 1);
            let contents = &rendered[0].contents;
            assert!(contents.starts_with("---\n"));
            assert!(contents.contains(&format!("\nname: {expected_name}\n")));
            assert!(contents.contains("\ndescription: "));
            assert!(contents.contains(&format!("\n---\n{MARKER_MD}\n# Engram")));
        }
    }

    #[test]
    fn policy_requires_compact_agent_profile_tools() {
        let policy = HarnessService::new().policy(HarnessKind::Codex);
        assert_eq!(
            policy.required_mcp_tools,
            [
                "orient",
                "memory",
                "repo",
                "search",
                "harness",
                "obligations"
            ]
        );
    }

    #[test]
    fn render_codex_adapter_uses_only_agent_profile_workflow() {
        let adapters = HarnessService::new()
            .render_adapters(HarnessKind::Codex, Some("codex-memory-session-skill"));
        assert_eq!(adapters.len(), 1);
        let contents = &adapters[0].contents;
        assert!(contents.contains("`orient`"));
        assert!(contents.contains("search(query=..."));
        assert!(contents.contains("memory(action=add)"));
        assert!(!contents.contains("telemetry(action="));
        assert!(!contents.contains("obligations(action="));
        assert!(!contents.contains("memory(action=changes_since)"));
    }

    #[test]
    fn render_codex_adapter_spells_out_safe_durable_capture() {
        let adapters = HarnessService::new()
            .render_adapters(HarnessKind::Codex, Some("codex-memory-session-skill"));
        assert_eq!(adapters.len(), 1);
        let contents = &adapters[0].contents;

        assert!(contents.contains("memory(action=add)"));
        assert!(contents.contains("narrowest project/repository/task scope"));
        assert!(contents.contains("writer provenance"));
        assert!(contents.contains("Every add requires"));
        assert!(contents.contains("`scope_type`"));
        assert!(contents.contains("`remote_url` or `local_path`"));
        assert!(contents.contains("A retrieval `scope` object does not replace"));
        assert!(contents.contains("`kind=procedure`"));
        assert!(contents.contains("evidence"));
        assert!(contents.contains("Never store credentials"));
    }

    #[test]
    fn codex_policy_mentions_context_compaction_save() {
        let policy = HarnessService::new()
            .render_policy(HarnessKind::Codex)
            .unwrap();
        assert!(policy.contains("before_context_compaction_save"));
        assert!(policy.contains("\"search\""));

        let adapters = HarnessService::new()
            .render_adapters(HarnessKind::Codex, Some("codex-memory-session-skill"));
        assert_eq!(adapters.len(), 1);
        assert!(adapters[0].contents.contains("handoff"));
    }

    #[test]
    fn render_gemini_adapter_mentions_namespaced_command() {
        let adapters = HarnessService::new().render_adapters(
            HarnessKind::GeminiCli,
            Some("gemini-memory-session-command"),
        );
        assert_eq!(adapters.len(), 1);
        assert_eq!(
            adapters[0].relative_path,
            ".gemini/commands/engram/memory-session.toml"
        );
        assert!(adapters[0].contents.contains("/engram:memory-session"));
    }

    #[test]
    fn render_cursor_adapter_uses_cursor_skill_path() {
        let adapters = HarnessService::new()
            .render_adapters(HarnessKind::Cursor, Some("cursor-memory-session-skill"));
        assert_eq!(adapters.len(), 1);
        assert_eq!(
            adapters[0].relative_path,
            ".cursor/skills/engram-memory-session/SKILL.md"
        );
        assert!(adapters[0].contents.contains("writer_harness=cursor"));
    }

    #[test]
    fn claude_install_merges_settings_and_is_ready() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join(".claude")).unwrap();
        let stale_session_end_handler =
            claude_mcp_hook_handler("SessionEnd", HarnessEnforcementProfile::Graduated);
        let mut stale_user_prompt_handler =
            claude_mcp_hook_handler("UserPromptSubmit", HarnessEnforcementProfile::Graduated);
        let stale_pre_tool_handler =
            claude_mcp_hook_handler("PreToolUse", HarnessEnforcementProfile::Graduated);
        let stale_tool_failure_handler =
            claude_mcp_hook_handler("PostToolUseFailure", HarnessEnforcementProfile::Graduated);
        let mut stale_pre_compact_handler =
            claude_mcp_hook_handler("PreCompact", HarnessEnforcementProfile::Graduated);
        stale_user_prompt_handler
            .pointer_mut("/input")
            .and_then(Value::as_object_mut)
            .unwrap()
            .remove("enforcement");
        stale_pre_compact_handler
            .pointer_mut("/input")
            .and_then(Value::as_object_mut)
            .unwrap()
            .remove("enforcement");
        fs::write(
            root.path().join(".claude/settings.json"),
            serde_json::to_string(&serde_json::json!({
                "hooks": {
                    "UserPromptSubmit": [{
                        "hooks": [
                            {
                                "type": "command",
                                "command": "existing"
                            },
                            stale_user_prompt_handler
                        ]
                    }],
                    "SessionStart": [{
                        "matcher": "startup|resume|compact",
                        "hooks": [{
                            "type": "command",
                            "command": CLAUDE_LEGACY_HOOK_COMMAND,
                            "timeout": 10
                        }]
                    }],
                    "SessionEnd": [{
                        "hooks": [
                            stale_session_end_handler,
                            {
                                "type": "command",
                                "command": CLAUDE_LEGACY_SESSION_END_HOOK_COMMAND,
                                "timeout": 15
                            }
                        ]
                    }],
                    "PreToolUse": [{
                        "matcher": "*",
                        "hooks": [
                            stale_pre_tool_handler
                        ]
                    }],
                    "PostToolUseFailure": [{
                        "matcher": "*",
                        "hooks": [
                            stale_tool_failure_handler
                        ]
                    }],
                    "PreCompact": [{
                        "matcher": "manual|auto",
                        "hooks": [
                            stale_pre_compact_handler
                        ]
                    }]
                },
                "permissions": {
                    "allow": ["mcp__engram__search"]
                }
            }))
            .unwrap(),
        )
        .unwrap();

        let service = HarnessService::new();
        let report = service
            .install_with_options(
                HarnessKind::ClaudeCode,
                Some(root.path()),
                HarnessInstallOptions {
                    write: true,
                    adopt_user_owned: false,
                    settings_target: HarnessSettingsTarget::default(),
                    enforcement_profile: HarnessEnforcementProfile::default(),
                },
            )
            .unwrap();

        assert!(report
            .written
            .iter()
            .any(|file| file.name == "claude-settings-merge"));
        let settings = fs::read_to_string(root.path().join(".claude/settings.json")).unwrap();
        assert!(settings.contains("mcp__engram__orient"));
        assert!(settings.contains("mcp__engram__search"));
        assert!(settings.contains("mcp__engram__harness"));
        assert!(!settings.contains("mcp__engram__telemetry"));
        assert!(!settings.contains("\"PreToolUse\""));
        assert!(!settings.contains("\"PostToolUseFailure\""));
        assert!(settings.contains("\"PreCompact\""));
        assert!(settings.contains("\"PostCompact\""));
        assert!(settings.contains("existing"));
        let settings_json: Value = serde_json::from_str(&settings).unwrap();
        let user_prompt_hooks = settings_json
            .pointer("/hooks/UserPromptSubmit/0/hooks")
            .and_then(Value::as_array)
            .unwrap();
        assert!(!user_prompt_hooks
            .iter()
            .any(|hook| hook.get("type").and_then(Value::as_str) == Some("mcp_tool")));
        let pre_compact_hooks = settings_json
            .pointer("/hooks/PreCompact/0/hooks")
            .and_then(Value::as_array)
            .unwrap();
        assert_eq!(pre_compact_hooks.len(), 1);
        assert_eq!(
            pre_compact_hooks[0]
                .pointer("/input/enforcement")
                .and_then(Value::as_str),
            Some("soft")
        );
        let session_start_hooks = settings_json
            .pointer("/hooks/SessionStart")
            .and_then(Value::as_array)
            .unwrap();
        assert!(!session_start_hooks.iter().any(|group| {
            group
                .get("hooks")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .any(|hook| {
                    hook.get("type").and_then(Value::as_str) == Some("command")
                        && hook.get("command").and_then(Value::as_str)
                            == Some(CLAUDE_LEGACY_HOOK_COMMAND)
                })
        }));
        let session_end_hooks = settings_json
            .pointer("/hooks/SessionEnd")
            .and_then(Value::as_array)
            .unwrap();
        assert!(session_end_hooks.iter().any(|group| {
            group
                .get("hooks")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .any(|hook| {
                    hook.get("type").and_then(Value::as_str) == Some("command")
                        && hook.get("command").and_then(Value::as_str)
                            == Some(CLAUDE_SESSION_END_HOOK_COMMAND)
                })
        }));
        assert!(!session_end_hooks.iter().any(|group| {
            group
                .get("hooks")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .any(|hook| hook.get("type").and_then(Value::as_str) == Some("mcp_tool"))
        }));
        assert!(!session_end_hooks.iter().any(|group| {
            group
                .get("hooks")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .any(|hook| {
                    hook.get("type").and_then(Value::as_str) == Some("command")
                        && hook.get("command").and_then(Value::as_str)
                            == Some(CLAUDE_LEGACY_SESSION_END_HOOK_COMMAND)
                })
        }));
        let session_start_hook =
            fs::read_to_string(root.path().join(".claude/hooks/engram-session-start.sh")).unwrap();
        assert!(session_start_hook.contains("\"systemMessage\""));
        assert!(!session_start_hook.contains("hookSpecificOutput"));
        let session_end_hook =
            fs::read_to_string(root.path().join(".claude/hooks/engram-session-end.sh")).unwrap();
        assert!(session_end_hook.contains("daemon.port"));
        assert!(session_end_hook.contains("SessionEnd"));
        assert!(session_end_hook.contains(r#".write_policy // "nudge""#));
        assert!(!session_end_hook.contains(r#".write_policy // "durable""#));

        let status = service
            .status(HarnessKind::ClaudeCode, Some(root.path()), &[])
            .unwrap();
        assert!(status.ready, "{:?}", status.warnings);
    }

    #[test]
    fn claude_default_install_is_idempotent_after_migration() {
        let root = tempfile::tempdir().unwrap();
        let service = HarnessService::new();
        let options = HarnessInstallOptions {
            write: true,
            adopt_user_owned: false,
            settings_target: HarnessSettingsTarget::default(),
            enforcement_profile: HarnessEnforcementProfile::default(),
        };

        service
            .install_with_options(HarnessKind::ClaudeCode, Some(root.path()), options)
            .unwrap();
        let second = service
            .install_with_options(HarnessKind::ClaudeCode, Some(root.path()), options)
            .unwrap();

        assert!(!second
            .written
            .iter()
            .any(|file| file.name == "claude-settings-merge"));
        assert!(second
            .skipped
            .iter()
            .any(|file| file.name == "claude-settings-merge"
                && file.message.contains("already includes")));
    }

    #[test]
    fn claude_ready_status_warns_effective_hooks_need_live_hooks_proof() {
        let root = tempfile::tempdir().unwrap();
        let service = HarnessService::new();
        service
            .install_with_options(
                HarnessKind::ClaudeCode,
                Some(root.path()),
                HarnessInstallOptions {
                    write: true,
                    adopt_user_owned: false,
                    settings_target: HarnessSettingsTarget::Project,
                    enforcement_profile: HarnessEnforcementProfile::default(),
                },
            )
            .unwrap();

        let status = service
            .status(HarnessKind::ClaudeCode, Some(root.path()), &[])
            .unwrap();

        assert!(status.ready, "{:?}", status.warnings);
        assert!(status.warnings.iter().any(|warning| {
            warning.contains("does not prove live effective hook visibility")
                && warning.contains("Claude Code /hooks")
        }));
    }

    #[test]
    fn claude_settings_commands_support_project_and_home_hook_locations() {
        let settings: Value = serde_json::from_str(&claude_settings_snippet(
            HarnessEnforcementProfile::default(),
        ))
        .unwrap();

        let start_command = settings["hooks"]["SessionStart"][0]["hooks"][0]["command"]
            .as_str()
            .unwrap();
        assert!(start_command.contains("git -C \"$start_dir\" rev-parse --show-toplevel"));
        assert!(start_command
            .contains("${project_root:+$project_root/.claude/hooks/engram-session-start.sh}"));
        assert!(start_command.contains("${HOME:-}/.claude/hooks/engram-session-start.sh"));
        assert!(start_command.contains("/usr/bin/env bash"));

        let end_command = settings["hooks"]["SessionEnd"][0]["hooks"][0]["command"]
            .as_str()
            .unwrap();
        assert!(end_command.contains("git -C \"$start_dir\" rev-parse --show-toplevel"));
        assert!(end_command
            .contains("${project_root:+$project_root/.claude/hooks/engram-session-end.sh}"));
        assert!(end_command.contains("${HOME:-}/.claude/hooks/engram-session-end.sh"));
        assert!(end_command.contains("/usr/bin/env bash"));
    }

    #[test]
    fn claude_settings_merge_replaces_stale_generated_dispatch_commands() {
        let stale_start = concat!(
            "project_hook=\"old/.claude/hooks/engram-session-start.sh\"; ",
            "home_hook=\"old/.claude/hooks/engram-session-start.sh\"; ",
            "printf 'Engram SessionStart hook skipped'"
        );
        let stale_end = concat!(
            "project_hook=\"old/.claude/hooks/engram-session-end.sh\"; ",
            "home_hook=\"old/.claude/hooks/engram-session-end.sh\"; ",
            "printf 'Engram SessionEnd hook skipped'"
        );
        let mut settings = json!({
            "hooks": {
                "SessionStart": [{
                    "matcher": "startup|resume|compact",
                    "hooks": [{"type": "command", "command": stale_start, "timeout": 10}]
                }],
                "SessionEnd": [{
                    "hooks": [{"type": "command", "command": stale_end, "timeout": 15}]
                }]
            }
        });

        assert!(merge_claude_hooks(
            &mut settings,
            HarnessEnforcementProfile::Soft
        ));
        let rendered = serde_json::to_string(&settings).unwrap();
        assert!(!rendered.contains(stale_start));
        assert!(!rendered.contains(stale_end));
        let start_handlers = settings["hooks"]["SessionStart"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|group| group["hooks"].as_array().unwrap());
        assert_eq!(
            start_handlers
                .filter(|handler| handler["command"].as_str() == Some(CLAUDE_HOOK_COMMAND))
                .count(),
            1
        );
        let end_handlers = settings["hooks"]["SessionEnd"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|group| group["hooks"].as_array().unwrap());
        assert_eq!(
            end_handlers
                .filter(|handler| {
                    handler["command"].as_str() == Some(CLAUDE_SESSION_END_HOOK_COMMAND)
                })
                .count(),
            1
        );
    }

    #[test]
    fn claude_default_settings_omit_high_frequency_runtime_hooks() {
        let settings: Value = serde_json::from_str(&claude_settings_snippet(
            HarnessEnforcementProfile::default(),
        ))
        .unwrap();

        assert!(settings.pointer("/hooks/SessionStart").is_some());
        assert!(settings.pointer("/hooks/PreCompact").is_some());
        assert!(settings.pointer("/hooks/PostCompact").is_some());
        assert!(settings.pointer("/hooks/SessionEnd").is_some());
        assert!(settings.pointer("/hooks/UserPromptSubmit").is_none());
        assert!(settings.pointer("/hooks/PreToolUse").is_none());
        assert!(settings.pointer("/hooks/PostToolUse").is_none());
        assert!(settings.pointer("/hooks/PostToolUseFailure").is_none());
        assert!(settings.pointer("/hooks/Stop").is_none());
    }

    #[test]
    fn claude_graduated_settings_include_runtime_enforcement_hooks() {
        let settings: Value = serde_json::from_str(&claude_settings_snippet(
            HarnessEnforcementProfile::Graduated,
        ))
        .unwrap();

        assert!(settings.pointer("/hooks/UserPromptSubmit").is_some());
        assert!(settings.pointer("/hooks/PreToolUse").is_some());
        assert!(settings.pointer("/hooks/PostToolUse").is_some());
        assert!(settings.pointer("/hooks/PostToolUseFailure").is_some());
        assert!(settings.pointer("/hooks/Stop").is_some());
        assert_eq!(
            settings["hooks"]["PreToolUse"][0]["hooks"][0]["input"]["enforcement"],
            "graduated"
        );
        let rendered = serde_json::to_string(&settings).unwrap();
        assert!(rendered.contains("mcp__engram__orient|mcp__engram__memory"));
        assert!(rendered.contains("${tool_input.action}"));
    }

    #[test]
    fn adopt_user_owned_hook_backs_up_and_replaces_file() {
        let root = tempfile::tempdir().unwrap();
        let hook = root.path().join(".claude/hooks/engram-session-start.sh");
        fs::create_dir_all(hook.parent().unwrap()).unwrap();
        fs::write(&hook, "user-owned hook").unwrap();

        let report = HarnessService::new()
            .install_with_options(
                HarnessKind::ClaudeCode,
                Some(root.path()),
                HarnessInstallOptions {
                    write: true,
                    adopt_user_owned: true,
                    settings_target: HarnessSettingsTarget::default(),
                    enforcement_profile: HarnessEnforcementProfile::default(),
                },
            )
            .unwrap();

        assert!(report.written.iter().any(|file| {
            file.path == hook.display().to_string() && file.message.contains("backup=")
        }));
        assert!(fs::read_to_string(&hook).unwrap().contains(MARKER_SH));
        assert!(root
            .path()
            .join(".claude/hooks/engram-session-start.sh.engram-backup")
            .exists());
    }

    #[test]
    fn claude_install_can_target_local_settings_explicitly() {
        let root = tempfile::tempdir().unwrap();
        let service = HarnessService::new();

        service
            .install_with_options(
                HarnessKind::ClaudeCode,
                Some(root.path()),
                HarnessInstallOptions {
                    write: true,
                    adopt_user_owned: false,
                    settings_target: HarnessSettingsTarget::Local,
                    enforcement_profile: HarnessEnforcementProfile::default(),
                },
            )
            .unwrap();

        assert!(root.path().join(".claude/settings.local.json").exists());
        assert!(!root.path().join(".claude/settings.json").exists());
        let status = service
            .status(HarnessKind::ClaudeCode, Some(root.path()), &[])
            .unwrap();
        assert!(status.ready, "{:?}", status.warnings);
        assert!(status
            .settings
            .iter()
            .filter(|check| check.required)
            .all(|check| check.locations == vec!["settings.local.json".to_string()]));
    }

    #[test]
    fn claude_install_snippet_only_does_not_modify_settings_files() {
        let root = tempfile::tempdir().unwrap();
        let report = HarnessService::new()
            .install_with_options(
                HarnessKind::ClaudeCode,
                Some(root.path()),
                HarnessInstallOptions {
                    write: true,
                    adopt_user_owned: false,
                    settings_target: HarnessSettingsTarget::SnippetOnly,
                    enforcement_profile: HarnessEnforcementProfile::default(),
                },
            )
            .unwrap();

        assert!(!root.path().join(".claude/settings.json").exists());
        assert!(!root.path().join(".claude/settings.local.json").exists());
        assert!(root
            .path()
            .join(".claude/engram-settings-snippet.json")
            .exists());
        assert!(report
            .skipped
            .iter()
            .any(|file| file.name == "claude-settings-merge"
                && file.message.contains("snippet-only")));
        assert!(report
            .warnings
            .iter()
            .any(|warning| warning.contains("settings target is snippet-only")));
    }

    #[test]
    fn claude_install_snippet_only_repairs_adapters_without_rewriting_existing_settings() {
        let root = tempfile::tempdir().unwrap();
        let claude_dir = root.path().join(".claude");
        let commands_dir = claude_dir.join("commands");
        fs::create_dir_all(&commands_dir).unwrap();

        let settings_path = claude_dir.join("settings.json");
        let local_settings_path = claude_dir.join("settings.local.json");
        let snippet_path = claude_dir.join("engram-settings-snippet.json");
        let stale_command_path = commands_dir.join("engram-memory-session.md");
        let settings_contents = r#"{"permissions":{"allow":["mcp__engram__search"]}}"#;
        let local_settings_contents = r#"{"hooks":{"Stop":[{"hooks":[]}]}}"#;
        let user_snippet_contents = r#"{"user":"owned"}"#;
        fs::write(&settings_path, settings_contents).unwrap();
        fs::write(&local_settings_path, local_settings_contents).unwrap();
        fs::write(&snippet_path, user_snippet_contents).unwrap();
        fs::write(&stale_command_path, format!("{MARKER_MD}\nstale adapter\n")).unwrap();

        let report = HarnessService::new()
            .install_with_options(
                HarnessKind::ClaudeCode,
                Some(root.path()),
                HarnessInstallOptions {
                    write: true,
                    adopt_user_owned: false,
                    settings_target: HarnessSettingsTarget::SnippetOnly,
                    enforcement_profile: HarnessEnforcementProfile::default(),
                },
            )
            .unwrap();

        assert!(report
            .written
            .iter()
            .any(|file| file.path == stale_command_path.display().to_string()));
        assert!(report
            .written
            .iter()
            .all(|file| !file.path.ends_with("settings.json")
                && !file.path.ends_with("settings.local.json")
                && !file.path.ends_with("engram-settings-snippet.json")));
        assert!(report
            .skipped
            .iter()
            .any(|file| file.name == "claude-settings-merge"
                && file.message.contains("snippet-only")));
        assert_eq!(
            fs::read_to_string(&settings_path).unwrap(),
            settings_contents
        );
        assert_eq!(
            fs::read_to_string(&local_settings_path).unwrap(),
            local_settings_contents
        );
        assert_eq!(
            fs::read_to_string(&snippet_path).unwrap(),
            user_snippet_contents
        );
        assert!(fs::read_to_string(&stale_command_path)
            .unwrap()
            .contains("memory(action=add)"));
    }

    #[test]
    fn status_warns_when_claude_hook_files_are_installed_but_settings_missing() {
        let root = tempfile::tempdir().unwrap();
        let service = HarnessService::new();
        service
            .install_with_options(
                HarnessKind::ClaudeCode,
                Some(root.path()),
                HarnessInstallOptions {
                    write: true,
                    adopt_user_owned: false,
                    settings_target: HarnessSettingsTarget::SnippetOnly,
                    enforcement_profile: HarnessEnforcementProfile::default(),
                },
            )
            .unwrap();

        let status = service
            .status(HarnessKind::ClaudeCode, Some(root.path()), &[])
            .unwrap();

        assert!(!status.ready);
        for adapter_name in ["claude-session-start-hook", "claude-session-end-hook"] {
            assert!(status.adapters.iter().any(|check| {
                check.name == adapter_name && check.status == HarnessAdapterStatus::Installed
            }));
        }
        for setting_name in ["SessionStart:startup|resume|compact", "SessionEnd"] {
            assert!(status.settings.iter().any(|check| {
                check.name == setting_name && check.kind == "hook" && check.locations.is_empty()
            }));
        }
        assert!(status.warnings.iter().any(|warning| {
            warning.contains("SessionStart startup|resume|compact")
                && warning.contains("Claude settings do not register")
        }));
        assert!(status.warnings.iter().any(|warning| {
            warning.contains("SessionEnd") && warning.contains("Claude settings do not register")
        }));
    }

    #[test]
    fn claude_project_settings_target_warns_about_existing_local_engram_entries() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join(".claude")).unwrap();
        fs::write(
            root.path().join(".claude/settings.local.json"),
            serde_json::to_string(&serde_json::json!({
                "permissions": {
                    "allow": ["mcp__engram__entity_get"]
                }
            }))
            .unwrap(),
        )
        .unwrap();

        let service = HarnessService::new();
        let report = service
            .install_with_options(
                HarnessKind::ClaudeCode,
                Some(root.path()),
                HarnessInstallOptions {
                    write: false,
                    adopt_user_owned: false,
                    settings_target: HarnessSettingsTarget::Project,
                    enforcement_profile: HarnessEnforcementProfile::default(),
                },
            )
            .unwrap();
        assert!(report
            .warnings
            .iter()
            .any(|warning| warning.contains("settings.local.json")
                && warning.contains("already contains Engram entries")));

        let status = service
            .status(HarnessKind::ClaudeCode, Some(root.path()), &[])
            .unwrap();
        assert!(status
            .warnings
            .iter()
            .any(|warning| warning.contains("not part of the current Claude harness contract")));
    }

    #[tokio::test]
    async fn hook_event_session_end_missing_write_policy_does_not_write_handoff() {
        let config = engram_store::StoreConfig::memory();
        let db = engram_store::connect_and_init(&config).await.unwrap();
        let handoff = crate::handoff::HandoffService::new(db);
        handoff.init_schema().await.unwrap();

        let outcome = HarnessService::new()
            .handle_hook_event(
                HarnessHookEvent {
                    harness: HarnessKind::ClaudeCode,
                    hook_event_name: "SessionEnd".to_string(),
                    session_id: Some("claude-session-1".to_string()),
                    cwd: Some("/tmp/engram".to_string()),
                    transcript_path: Some("/tmp/transcript.jsonl".to_string()),
                    reason: Some("shutdown".to_string()),
                    ..HarnessHookEvent::default()
                },
                HarnessHookServices {
                    memory: None,
                    obligations: None,
                    handoff: Some(&handoff),
                },
            )
            .await
            .unwrap();

        assert!(!outcome.handoff_written);
        assert!(outcome
            .additional_context
            .contains(r#"<engram_hook event="SessionEnd" write_policy="nudge">"#));
        assert!(handoff
            .get(Some("engram"), None)
            .await
            .unwrap()
            .item
            .is_none());
    }

    #[tokio::test]
    async fn hook_event_session_end_explicit_durable_writes_handoff() {
        let config = engram_store::StoreConfig::memory();
        let db = engram_store::connect_and_init(&config).await.unwrap();
        let handoff = crate::handoff::HandoffService::new(db);
        handoff.init_schema().await.unwrap();

        let outcome = HarnessService::new()
            .handle_hook_event(
                HarnessHookEvent {
                    harness: HarnessKind::ClaudeCode,
                    hook_event_name: "SessionEnd".to_string(),
                    session_id: Some("claude-session-1".to_string()),
                    cwd: Some("/tmp/engram".to_string()),
                    transcript_path: Some("/tmp/transcript.jsonl".to_string()),
                    reason: Some("shutdown".to_string()),
                    write_policy: Some("durable".to_string()),
                    project: Some("engram".to_string()),
                    ..HarnessHookEvent::default()
                },
                HarnessHookServices {
                    memory: None,
                    obligations: None,
                    handoff: Some(&handoff),
                },
            )
            .await
            .unwrap();

        assert!(outcome.handoff_written);
        let handoff = handoff.get(Some("engram"), None).await.unwrap();
        let item = handoff
            .item
            .expect("explicit durable SessionEnd should write");
        assert!(item.content.contains("Claude Code Session-End Handoff"));
        assert!(item.content.contains("claude-session-1"));
    }

    #[tokio::test]
    async fn hook_event_session_end_abstains_without_canonical_project() {
        let config = engram_store::StoreConfig::memory();
        let db = engram_store::connect_and_init(&config).await.unwrap();
        let handoff = crate::handoff::HandoffService::new(db);
        handoff.init_schema().await.unwrap();

        let outcome = HarnessService::new()
            .handle_hook_event(
                HarnessHookEvent {
                    harness: HarnessKind::ClaudeCode,
                    hook_event_name: "SessionEnd".to_string(),
                    cwd: Some("/tmp/unregistered-worktree".to_string()),
                    write_policy: Some("durable".to_string()),
                    ..HarnessHookEvent::default()
                },
                HarnessHookServices {
                    memory: None,
                    obligations: None,
                    handoff: Some(&handoff),
                },
            )
            .await
            .unwrap();

        assert!(!outcome.handoff_written);
        assert!(outcome
            .warnings
            .iter()
            .any(|warning| warning.contains("no canonical project")));
        assert!(handoff.get(None, None).await.unwrap().item.is_none());
    }

    #[tokio::test]
    async fn hook_event_does_not_persist_generic_task_instruction_as_memory() {
        let config = engram_store::StoreConfig::memory();
        let db = engram_store::connect_and_init(&config).await.unwrap();
        let memory = crate::memory::MemoryService::new(db);
        memory.init_schema().await.unwrap();

        let outcome = HarnessService::new()
            .handle_hook_event(
                HarnessHookEvent {
                    harness: HarnessKind::ClaudeCode,
                    hook_event_name: "UserPromptSubmit".to_string(),
                    prompt: Some("You should implement the design and commit it.".to_string()),
                    cwd: Some("/tmp/engram".to_string()),
                    write_policy: Some("durable".to_string()),
                    ..HarnessHookEvent::default()
                },
                HarnessHookServices {
                    memory: Some(&memory),
                    obligations: None,
                    handoff: None,
                },
            )
            .await
            .unwrap();

        assert_eq!(outcome.memory_written, 0);
    }

    #[tokio::test]
    async fn hook_event_stop_blocks_once_when_obligations_are_open() {
        let config = engram_store::StoreConfig::memory();
        let db = engram_store::connect_and_init(&config).await.unwrap();
        let memory = crate::memory::MemoryService::new(db.clone());
        memory.init_schema().await.unwrap();
        let obligations = crate::obligation::ObligationService::new(db.clone());
        obligations.init_schema().await.unwrap();
        let handoff = crate::handoff::HandoffService::new(db);
        handoff.init_schema().await.unwrap();
        let service = HarnessService::new();

        let prompt_outcome = service
            .handle_hook_event(
                HarnessHookEvent {
                    harness: HarnessKind::ClaudeCode,
                    enforcement_profile: HarnessEnforcementProfile::Graduated,
                    hook_event_name: "UserPromptSubmit".to_string(),
                    prompt: Some("Implement the design and commit it".to_string()),
                    cwd: Some("/tmp/engram".to_string()),
                    write_policy: Some("durable".to_string()),
                    ..HarnessHookEvent::default()
                },
                HarnessHookServices {
                    memory: Some(&memory),
                    obligations: Some(&obligations),
                    handoff: Some(&handoff),
                },
            )
            .await
            .unwrap();
        assert!(prompt_outcome.obligations_written >= 2);
        assert_eq!(
            prompt_outcome.response["hookSpecificOutput"]["hookEventName"],
            "UserPromptSubmit"
        );

        let stop_outcome = service
            .handle_hook_event(
                HarnessHookEvent {
                    harness: HarnessKind::ClaudeCode,
                    enforcement_profile: HarnessEnforcementProfile::Graduated,
                    hook_event_name: "Stop".to_string(),
                    cwd: Some("/tmp/engram".to_string()),
                    write_policy: Some("durable".to_string()),
                    stop_hook_active: false,
                    ..HarnessHookEvent::default()
                },
                HarnessHookServices {
                    memory: Some(&memory),
                    obligations: Some(&obligations),
                    handoff: Some(&handoff),
                },
            )
            .await
            .unwrap();
        assert!(stop_outcome.blocked);
        assert_eq!(stop_outcome.response["decision"], "block");
        assert!(stop_outcome.response.get("hookSpecificOutput").is_none());
        let stop_reason = stop_outcome.response["reason"].as_str().unwrap();
        assert!(stop_reason.contains("obligations(action=doctor"));
        assert!(stop_reason.contains("scope={relevance_mode:\"global\"}"));
        assert!(stop_reason.contains("cwd=\"/tmp/engram\""));
        assert!(stop_reason.contains("resolution=\""));
        assert!(!stop_reason.contains("resolution_kind="));
        assert!(!stop_reason.contains("project=\"engram\""));

        let active_stop = service
            .handle_hook_event(
                HarnessHookEvent {
                    harness: HarnessKind::ClaudeCode,
                    enforcement_profile: HarnessEnforcementProfile::Graduated,
                    hook_event_name: "Stop".to_string(),
                    cwd: Some("/tmp/engram".to_string()),
                    write_policy: Some("durable".to_string()),
                    stop_hook_active: true,
                    ..HarnessHookEvent::default()
                },
                HarnessHookServices {
                    memory: Some(&memory),
                    obligations: Some(&obligations),
                    handoff: Some(&handoff),
                },
            )
            .await
            .unwrap();
        assert!(!active_stop.blocked);
        assert_eq!(active_stop.response["continue"], true);
        assert!(active_stop.response.get("hookSpecificOutput").is_none());
        assert!(active_stop.response["systemMessage"]
            .as_str()
            .unwrap()
            .contains("open_obligations="));
        assert!(active_stop
            .warnings
            .iter()
            .any(|warning| warning.contains("stop_hook_active=true")));

        let strict_active_stop = service
            .handle_hook_event(
                HarnessHookEvent {
                    harness: HarnessKind::ClaudeCode,
                    enforcement_profile: HarnessEnforcementProfile::Strict,
                    hook_event_name: "Stop".to_string(),
                    cwd: Some("/tmp/engram".to_string()),
                    write_policy: Some("durable".to_string()),
                    stop_hook_active: true,
                    ..HarnessHookEvent::default()
                },
                HarnessHookServices {
                    memory: Some(&memory),
                    obligations: Some(&obligations),
                    handoff: Some(&handoff),
                },
            )
            .await
            .unwrap();
        assert!(strict_active_stop.blocked);
        assert_eq!(strict_active_stop.response["decision"], "block");
    }

    #[tokio::test]
    async fn soft_hook_event_stop_preserves_non_blocking_behavior() {
        let config = engram_store::StoreConfig::memory();
        let db = engram_store::connect_and_init(&config).await.unwrap();
        let memory = crate::memory::MemoryService::new(db.clone());
        memory.init_schema().await.unwrap();
        let obligations = crate::obligation::ObligationService::new(db.clone());
        obligations.init_schema().await.unwrap();
        let handoff = crate::handoff::HandoffService::new(db);
        handoff.init_schema().await.unwrap();
        let service = HarnessService::new();

        service
            .handle_hook_event(
                HarnessHookEvent {
                    harness: HarnessKind::ClaudeCode,
                    enforcement_profile: HarnessEnforcementProfile::Soft,
                    hook_event_name: "UserPromptSubmit".to_string(),
                    prompt: Some("Implement the design and commit it".to_string()),
                    cwd: Some("/tmp/engram".to_string()),
                    write_policy: Some("durable".to_string()),
                    ..HarnessHookEvent::default()
                },
                HarnessHookServices {
                    memory: Some(&memory),
                    obligations: Some(&obligations),
                    handoff: Some(&handoff),
                },
            )
            .await
            .unwrap();

        let stop_outcome = service
            .handle_hook_event(
                HarnessHookEvent {
                    harness: HarnessKind::ClaudeCode,
                    enforcement_profile: HarnessEnforcementProfile::Soft,
                    hook_event_name: "Stop".to_string(),
                    cwd: Some("/tmp/engram".to_string()),
                    write_policy: Some("durable".to_string()),
                    stop_hook_active: false,
                    ..HarnessHookEvent::default()
                },
                HarnessHookServices {
                    memory: Some(&memory),
                    obligations: Some(&obligations),
                    handoff: Some(&handoff),
                },
            )
            .await
            .unwrap();

        assert!(!stop_outcome.blocked);
        assert_eq!(stop_outcome.response["continue"], true);
        assert!(stop_outcome.response["systemMessage"]
            .as_str()
            .unwrap()
            .contains("without blocking the user"));
    }

    #[test]
    fn procedure_match_is_an_orientation_boundary_but_other_memory_actions_are_not() {
        assert!(is_orientation_boundary_tool(&HarnessHookEvent {
            tool_name: Some("mcp__engram__orient".to_string()),
            ..HarnessHookEvent::default()
        }));
        assert!(is_orientation_boundary_tool(&HarnessHookEvent {
            tool_name: Some("mcp__engram__memory".to_string()),
            tool_input_action: Some("procedure_match".to_string()),
            ..HarnessHookEvent::default()
        }));
        assert!(!is_orientation_boundary_tool(&HarnessHookEvent {
            tool_name: Some("mcp__engram__memory".to_string()),
            tool_input_action: Some("list".to_string()),
            ..HarnessHookEvent::default()
        }));
    }

    #[tokio::test]
    async fn pre_tool_use_blocks_until_an_identity_boundary_resolves_orientation_obligation() {
        let config = engram_store::StoreConfig::memory();
        let db = engram_store::connect_and_init(&config).await.unwrap();
        let obligations = crate::obligation::ObligationService::new(db);
        obligations.init_schema().await.unwrap();
        let service = HarnessService::new();
        let services = || HarnessHookServices {
            memory: None,
            obligations: Some(&obligations),
            handoff: None,
        };

        let prompt_outcome = service
            .handle_hook_event(
                HarnessHookEvent {
                    harness: HarnessKind::ClaudeCode,
                    enforcement_profile: HarnessEnforcementProfile::Graduated,
                    hook_event_name: "UserPromptSubmit".to_string(),
                    prompt: Some("Inspect the repository and explain the GA status".to_string()),
                    cwd: Some("/tmp/engram".to_string()),
                    write_policy: Some("durable".to_string()),
                    ..HarnessHookEvent::default()
                },
                services(),
            )
            .await
            .unwrap();
        assert!(prompt_outcome.obligations_written >= 1);

        let denied = service
            .handle_hook_event(
                HarnessHookEvent {
                    harness: HarnessKind::ClaudeCode,
                    enforcement_profile: HarnessEnforcementProfile::Graduated,
                    hook_event_name: "PreToolUse".to_string(),
                    cwd: Some("/tmp/engram".to_string()),
                    tool_name: Some("Bash".to_string()),
                    ..HarnessHookEvent::default()
                },
                services(),
            )
            .await
            .unwrap();
        assert!(denied.blocked);
        assert_eq!(
            denied.response["hookSpecificOutput"]["permissionDecision"],
            "deny"
        );
        assert!(
            denied.response["hookSpecificOutput"]["permissionDecisionReason"]
                .as_str()
                .unwrap()
                .contains("response_shape=\"lean\"")
        );

        let orient_pretool = service
            .handle_hook_event(
                HarnessHookEvent {
                    harness: HarnessKind::ClaudeCode,
                    enforcement_profile: HarnessEnforcementProfile::Graduated,
                    hook_event_name: "PreToolUse".to_string(),
                    cwd: Some("/tmp/engram".to_string()),
                    tool_name: Some("mcp__engram__orient".to_string()),
                    ..HarnessHookEvent::default()
                },
                services(),
            )
            .await
            .unwrap();
        assert!(!orient_pretool.blocked);
        assert_eq!(orient_pretool.response["continue"], true);

        service
            .handle_hook_event(
                HarnessHookEvent {
                    harness: HarnessKind::ClaudeCode,
                    enforcement_profile: HarnessEnforcementProfile::Graduated,
                    hook_event_name: "PostToolUse".to_string(),
                    cwd: Some("/tmp/engram".to_string()),
                    tool_name: Some("mcp__engram__orient".to_string()),
                    ..HarnessHookEvent::default()
                },
                services(),
            )
            .await
            .unwrap();

        let allowed = service
            .handle_hook_event(
                HarnessHookEvent {
                    harness: HarnessKind::ClaudeCode,
                    enforcement_profile: HarnessEnforcementProfile::Graduated,
                    hook_event_name: "PreToolUse".to_string(),
                    cwd: Some("/tmp/engram".to_string()),
                    tool_name: Some("Bash".to_string()),
                    ..HarnessHookEvent::default()
                },
                services(),
            )
            .await
            .unwrap();
        assert!(!allowed.blocked);
        assert_eq!(allowed.response["continue"], true);

        service
            .handle_hook_event(
                HarnessHookEvent {
                    harness: HarnessKind::ClaudeCode,
                    enforcement_profile: HarnessEnforcementProfile::Graduated,
                    hook_event_name: "UserPromptSubmit".to_string(),
                    prompt: Some("Handle the worker deployment procedure".to_string()),
                    cwd: Some("/tmp/engram".to_string()),
                    write_policy: Some("durable".to_string()),
                    ..HarnessHookEvent::default()
                },
                services(),
            )
            .await
            .unwrap();

        service
            .handle_hook_event(
                HarnessHookEvent {
                    harness: HarnessKind::ClaudeCode,
                    enforcement_profile: HarnessEnforcementProfile::Graduated,
                    hook_event_name: "PostToolUse".to_string(),
                    cwd: Some("/tmp/engram".to_string()),
                    tool_name: Some("mcp__engram__memory".to_string()),
                    tool_input_action: Some("list".to_string()),
                    ..HarnessHookEvent::default()
                },
                services(),
            )
            .await
            .unwrap();
        let still_denied = service
            .handle_hook_event(
                HarnessHookEvent {
                    harness: HarnessKind::ClaudeCode,
                    enforcement_profile: HarnessEnforcementProfile::Graduated,
                    hook_event_name: "PreToolUse".to_string(),
                    cwd: Some("/tmp/engram".to_string()),
                    tool_name: Some("Bash".to_string()),
                    ..HarnessHookEvent::default()
                },
                services(),
            )
            .await
            .unwrap();
        assert!(still_denied.blocked);

        service
            .handle_hook_event(
                HarnessHookEvent {
                    harness: HarnessKind::ClaudeCode,
                    enforcement_profile: HarnessEnforcementProfile::Graduated,
                    hook_event_name: "PostToolUse".to_string(),
                    cwd: Some("/tmp/engram".to_string()),
                    tool_name: Some("mcp__engram__memory".to_string()),
                    tool_input_action: Some("procedure_match".to_string()),
                    ..HarnessHookEvent::default()
                },
                services(),
            )
            .await
            .unwrap();
        let procedure_allowed = service
            .handle_hook_event(
                HarnessHookEvent {
                    harness: HarnessKind::ClaudeCode,
                    enforcement_profile: HarnessEnforcementProfile::Graduated,
                    hook_event_name: "PreToolUse".to_string(),
                    cwd: Some("/tmp/engram".to_string()),
                    tool_name: Some("Bash".to_string()),
                    ..HarnessHookEvent::default()
                },
                services(),
            )
            .await
            .unwrap();
        assert!(!procedure_allowed.blocked);
    }

    #[tokio::test]
    async fn hook_event_stop_detects_changed_document_obligations() {
        let root = tempfile::tempdir().unwrap();
        std::process::Command::new("git")
            .arg("init")
            .current_dir(root.path())
            .output()
            .unwrap();
        fs::create_dir_all(root.path().join("docs")).unwrap();
        fs::write(root.path().join("docs/SESSION_FINDINGS.md"), "# Findings\n").unwrap();

        let config = engram_store::StoreConfig::memory();
        let db = engram_store::connect_and_init(&config).await.unwrap();
        let memory = crate::memory::MemoryService::new(db.clone());
        memory.init_schema().await.unwrap();
        let obligations = crate::obligation::ObligationService::new(db.clone());
        obligations.init_schema().await.unwrap();
        let handoff = crate::handoff::HandoffService::new(db);
        handoff.init_schema().await.unwrap();
        let service = HarnessService::new();

        let unrelated_outcome = service
            .handle_hook_event(
                HarnessHookEvent {
                    harness: HarnessKind::ClaudeCode,
                    hook_event_name: "UserPromptSubmit".to_string(),
                    prompt: Some("Implement the design and commit it.".to_string()),
                    cwd: Some("/tmp/unrelated-project".to_string()),
                    project: Some("unrelated-project".to_string()),
                    write_policy: Some("durable".to_string()),
                    ..HarnessHookEvent::default()
                },
                HarnessHookServices {
                    memory: Some(&memory),
                    obligations: Some(&obligations),
                    handoff: Some(&handoff),
                },
            )
            .await
            .unwrap();
        assert!(unrelated_outcome.obligations_written >= 1);

        let outcome = service
            .handle_hook_event(
                HarnessHookEvent {
                    harness: HarnessKind::ClaudeCode,
                    hook_event_name: "Stop".to_string(),
                    cwd: Some(root.path().display().to_string()),
                    project: Some("engram-stop-hook-smoke".to_string()),
                    write_policy: Some("durable".to_string()),
                    stop_hook_active: false,
                    ..HarnessHookEvent::default()
                },
                HarnessHookServices {
                    memory: Some(&memory),
                    obligations: Some(&obligations),
                    handoff: Some(&handoff),
                },
            )
            .await
            .unwrap();

        assert_eq!(outcome.obligations_written, 1);
        assert!(outcome.response["systemMessage"]
            .as_str()
            .unwrap()
            .contains("obligations_written=1"));
        assert!(outcome.response["systemMessage"]
            .as_str()
            .unwrap()
            .contains("open_obligations=1"));

        let doctor = obligations.doctor(None, None, Some(8)).await.unwrap();
        assert!(doctor.open.len() > 1);
        assert!(doctor
            .open
            .iter()
            .any(|obligation| obligation.title.contains("docs/SESSION_FINDINGS.md")));
    }

    #[tokio::test]
    async fn hook_event_failed_tool_writes_memory_and_obligation() {
        let config = engram_store::StoreConfig::memory();
        let db = engram_store::connect_and_init(&config).await.unwrap();
        let memory = crate::memory::MemoryService::new(db.clone());
        memory.init_schema().await.unwrap();
        let obligations = crate::obligation::ObligationService::new(db.clone());
        obligations.init_schema().await.unwrap();
        let handoff = crate::handoff::HandoffService::new(db);
        handoff.init_schema().await.unwrap();

        let outcome = HarnessService::new()
            .handle_hook_event(
                HarnessHookEvent {
                    harness: HarnessKind::ClaudeCode,
                    hook_event_name: "PostToolUseFailure".to_string(),
                    cwd: Some("/tmp/engram".to_string()),
                    tool_name: Some("mcp__engram__memory".to_string()),
                    tool_error: Some("invalid type: string, expected struct".to_string()),
                    write_policy: Some("durable".to_string()),
                    ..HarnessHookEvent::default()
                },
                HarnessHookServices {
                    memory: Some(&memory),
                    obligations: Some(&obligations),
                    handoff: Some(&handoff),
                },
            )
            .await
            .unwrap();

        assert_eq!(outcome.memory_written, 1);
        assert!(outcome.obligations_written >= 1);
        assert_eq!(outcome.response["continue"], true);
        assert!(outcome.response.get("hookSpecificOutput").is_none());
        assert!(outcome.response["systemMessage"]
            .as_str()
            .unwrap()
            .contains("memory_written=1"));
    }
}
