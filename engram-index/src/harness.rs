//! Agent harness policy rendering and installation.
//!
//! This module is deliberately filesystem-only. It does not persist state in the
//! Engram database; instead it renders a stable policy and verifies local adapter
//! files that make Engram's lifecycle visible to agent surfaces.

use crate::error::{IndexError, IndexResult};
use engram_core::harness::{
    HarnessAdapterCheck, HarnessAdapterKind, HarnessAdapterSpec, HarnessAdapterStatus,
    HarnessInstallFile, HarnessInstallReport, HarnessKind, HarnessLifecycleTrigger, HarnessPolicy,
    HarnessRenderedAdapter, HarnessSettingsCheck, HarnessStatusReport,
};
use engram_core::memory::{
    ClaimOrigin, EvidenceKind, EvidenceRef, Harness, MemoryItem, MemoryKind, MemoryScope,
    ModelIdentity, WriterProvenance,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const MARKER_MD: &str = "<!-- engram:harness-adapter:v1 -->";
const MARKER_SH: &str = "# engram:harness-adapter:v1";
const CLAUDE_HOOK_COMMAND: &str =
    "\"${CLAUDE_PROJECT_DIR:-.}/.claude/hooks/engram-session-start.sh\"";
const CLAUDE_SESSION_END_HOOK_COMMAND: &str =
    "\"${CLAUDE_PROJECT_DIR:-.}/.claude/hooks/engram-session-end.sh\"";

/// Options for harness adapter installation.
#[derive(Debug, Clone, Copy, Default)]
pub struct HarnessInstallOptions {
    /// Actually write generated adapters and settings changes.
    pub write: bool,
    /// Back up and replace user-owned adapter files.
    pub adopt_user_owned: bool,
    /// Claude Code settings target for generated permissions and hooks.
    pub settings_target: HarnessSettingsTarget,
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
        HarnessPolicy {
            harness,
            soft_contract: true,
            lifecycle_triggers: lifecycle_triggers(),
            required_mcp_tools: required_mcp_tools(),
            adapters: adapters_for(harness),
        }
    }

    /// Return status for a harness at an install root.
    pub fn status(
        &self,
        harness: HarnessKind,
        root: Option<&Path>,
        observed_mcp_tools: &[String],
    ) -> IndexResult<HarnessStatusReport> {
        let root = resolve_root(root)?;
        let policy = self.policy(harness);
        let adapters: Vec<_> = policy
            .adapters
            .iter()
            .map(|adapter| check_adapter(&root, adapter))
            .collect::<IndexResult<_>>()?;
        let missing_mcp_tools: Vec<String> = policy
            .required_mcp_tools
            .iter()
            .filter(|tool| {
                !observed_mcp_tools.is_empty()
                    && !observed_mcp_tools.iter().any(|name| name == *tool)
            })
            .cloned()
            .collect();

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
            let settings_status = claude_settings_status(&root)?;
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
        }

        Ok(HarnessStatusReport {
            harness,
            root: root.display().to_string(),
            policy,
            adapters,
            missing_mcp_tools,
            settings,
            warnings,
            ready,
        })
    }

    /// Doctor currently extends status with soft lifecycle warnings.
    pub fn doctor(
        &self,
        harness: HarnessKind,
        root: Option<&Path>,
        observed_mcp_tools: &[String],
    ) -> IndexResult<HarnessStatusReport> {
        let mut report = self.status(harness, root, observed_mcp_tools)?;
        if report.ready {
            report
                .warnings
                .push("Harness adapter files are present; lifecycle compliance is still soft and depends on the agent following the policy.".to_string());
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
        serde_json::to_string_pretty(&self.policy(harness))
            .map_err(|e| IndexError::Parse(format!("failed to render harness policy: {e}")))
    }

    /// Render one adapter or all adapters.
    #[must_use]
    pub fn render_adapters(
        &self,
        harness: HarnessKind,
        adapter_name: Option<&str>,
    ) -> Vec<HarnessRenderedAdapter> {
        self.policy(harness)
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

        for adapter in self.policy(harness).adapters {
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
                &mut planned,
                &mut written,
                &mut skipped,
                &mut warnings,
            )?;
        }

        Ok(HarnessInstallReport {
            harness,
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
        let project = event
            .project
            .clone()
            .or_else(|| project_from_cwd(event.cwd.as_deref()));
        let writer = hook_writer(&event);

        match normalized_event_name(&event.hook_event_name).as_str() {
            "userpromptsubmit" => {
                if let Some(service) = services.obligations {
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
            "posttooluse" => {
                if let Some(service) = services.obligations {
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
                    if let Some(service) = services.handoff {
                        let content = format!(
                            "# Claude Code Pre-Compact Handoff\n\nSession: {}\nCWD: {}\nTrigger: {}\nTranscript: {}\n\n## Next Actions\n- Resume by calling orient, handoff(action=get), and memory(action=changes_since).\n",
                            event.session_id.as_deref().unwrap_or("unknown"),
                            event.cwd.as_deref().unwrap_or("unknown"),
                            event.trigger.as_deref().unwrap_or("unknown"),
                            event.transcript_path.as_deref().unwrap_or("unknown"),
                        );
                        match service
                            .update(
                                project.clone(),
                                None,
                                content,
                                vec![
                                    "Resume by calling orient and inspecting the rolling handoff."
                                        .to_string(),
                                ],
                                writer.clone(),
                                false,
                            )
                            .await
                        {
                            Ok(update) => handoff_written = update.written,
                            Err(error) => warnings.push(format!("handoff update failed: {error}")),
                        }
                    }
                }
            }
            "postcompact" => {
                if write_durable {
                    if let (Some(service), Some(summary)) =
                        (services.handoff, event.compact_summary.clone())
                    {
                        let content = format!(
                            "# Claude Code Post-Compact Summary\n\n{}\n\n## Next Actions\n- Continue with orient, handoff(action=get), and memory(action=changes_since).\n",
                            summary.trim()
                        );
                        match service
                            .update(
                                project.clone(),
                                None,
                                content,
                                vec![
                                    "Continue with orient and recent memory changes after compaction."
                                        .to_string(),
                                ],
                                writer.clone(),
                                false,
                            )
                            .await
                        {
                            Ok(update) => handoff_written = update.written,
                            Err(error) => warnings.push(format!("handoff update failed: {error}")),
                        }
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
                if let Some(service) = services.handoff {
                    let content = format!(
                        "# Claude Code Session-End Handoff\n\nSession: {}\nCWD: {}\nReason: {}\nTranscript: {}\n\n## Next Actions\n- On resume, call orient and inspect this handoff before acting.\n",
                        event.session_id.as_deref().unwrap_or("unknown"),
                        event.cwd.as_deref().unwrap_or("unknown"),
                        event.reason.as_deref().unwrap_or("unknown"),
                        event.transcript_path.as_deref().unwrap_or("unknown"),
                    );
                    match service
                        .update(
                            project.clone(),
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
                }
            }
            _ => {}
        }

        let open_obligations = if let Some(service) = services.obligations {
            match service
                .list_open_for_context(project.as_deref(), event.cwd.as_deref())
                .await
            {
                Ok(obligations) => obligations.len(),
                Err(error) => {
                    warnings.push(format!("obligation doctor failed: {error}"));
                    0
                }
            }
        } else {
            0
        };

        let additional_context = hook_additional_context(
            &event,
            memory_written,
            obligations_written,
            handoff_written,
            open_obligations,
            &warnings,
        );
        let response = claude_hook_response(&event, &additional_context);

        Ok(HarnessHookEventOutcome {
            response,
            additional_context,
            memory_written,
            obligations_written,
            handoff_written,
            blocked: false,
            warnings,
        })
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

fn project_from_cwd(cwd: Option<&str>) -> Option<String> {
    cwd.and_then(|cwd| {
        Path::new(cwd)
            .file_name()
            .and_then(|name| name.to_str())
            .map(str::to_string)
    })
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
        .or_else(|| project_from_cwd(event.cwd.as_deref()))
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
            "Engram already ran final document-obligation detection for changed durable docs. Before final response, check memory(action=changes_since) and obligations(action=doctor, project=..., cwd=...); resolve or explicitly skip open obligations without blocking the user, rerun obligations(action=detect, project=..., cwd=...) if more files change, and when outcome is assessable call telemetry(action=submit_feedback) with task_success, preference_adhered, repeated_context_questions, bad_memory_used, missing_context, used_memory_ids, rejected_memory_ids, stale_memory_ids, and wrong_scope_memory_ids for the relevant trace_id."
                .to_string(),
        ),
        "precompact" | "postcompact" => lines.push(
            "Before relying on compacted context, use handoff(action=get) and memory(action=changes_since)."
                .to_string(),
        ),
        "sessionend" => lines.push(
            "The session ended; the next session should resume from orient and the rolling handoff."
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
        "harness",
        "lint",
        "graph",
        "handoff",
        "obligations",
        "telemetry",
        "vault",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn adapters_for(harness: HarnessKind) -> Vec<HarnessAdapterSpec> {
    match harness {
        HarnessKind::ClaudeCode => claude_adapters(),
        HarnessKind::Codex => codex_adapters(),
        HarnessKind::GeminiCli => gemini_adapters(),
        HarnessKind::Cursor => cursor_adapters(),
        HarnessKind::Generic => generic_adapters(),
    }
}

fn claude_adapters() -> Vec<HarnessAdapterSpec> {
    vec![
        adapter(
            "claude-memory-session-command",
            HarnessAdapterKind::ClaudeCommand,
            ".claude/commands/engram-memory-session.md",
            "Claude command that states the Memory OS lifecycle contract.",
            true,
            claude_memory_session_command(),
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
            claude_session_start_hook(),
        ),
        adapter(
            "claude-stop-nudge-hook",
            HarnessAdapterKind::ClaudeHook,
            ".claude/hooks/engram-stop-nudge.sh",
            "Claude hook nudge before stopping/final response.",
            true,
            claude_stop_nudge_hook(),
        ),
        adapter(
            "claude-session-end-hook",
            HarnessAdapterKind::ClaudeHook,
            ".claude/hooks/engram-session-end.sh",
            "Claude command hook for session-end handoff when MCP tool hooks are unavailable.",
            true,
            claude_session_end_hook(),
        ),
        adapter(
            "claude-settings-snippet",
            HarnessAdapterKind::PolicyDocument,
            ".claude/engram-settings-snippet.json",
            "Claude Code settings snippet for Engram MCP permissions and lifecycle hooks.",
            false,
            claude_settings_snippet(),
        ),
        adapter(
            "project-agents-snippet",
            HarnessAdapterKind::ProjectInstructions,
            "AGENTS.engram.md",
            "Project instruction snippet that can be merged into AGENTS.md.",
            false,
            agents_snippet(),
        ),
    ]
}

fn codex_adapters() -> Vec<HarnessAdapterSpec> {
    vec![
        adapter(
            "codex-memory-session-skill",
            HarnessAdapterKind::CodexSkill,
            ".codex/skills/engram-memory-session/SKILL.md",
            "Codex skill for the Memory OS lifecycle contract.",
            true,
            codex_memory_session_skill(),
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
            agents_snippet(),
        ),
    ]
}

fn gemini_adapters() -> Vec<HarnessAdapterSpec> {
    vec![
        adapter(
            "gemini-memory-session-command",
            HarnessAdapterKind::GeminiCommand,
            ".gemini/commands/engram/memory-session.toml",
            "Gemini CLI custom command for the Memory OS lifecycle contract.",
            true,
            gemini_memory_session_command(),
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
            gemini_global_context(),
        ),
    ]
}

fn cursor_adapters() -> Vec<HarnessAdapterSpec> {
    vec![
        adapter(
            "cursor-memory-session-skill",
            HarnessAdapterKind::CursorSkill,
            ".cursor/skills/engram-memory-session/SKILL.md",
            "Cursor Agent skill for the Memory OS lifecycle contract.",
            true,
            cursor_memory_session_skill(),
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

fn generic_adapters() -> Vec<HarnessAdapterSpec> {
    vec![adapter(
        "generic-harness-policy",
        HarnessAdapterKind::PolicyDocument,
        ".engram/harness-policy.md",
        "Generic Memory OS harness lifecycle policy.",
        true,
        generic_policy_document(),
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
    let (status, message) = match fs::read_to_string(&path) {
        Ok(existing) => {
            if existing == adapter.contents {
                (
                    HarnessAdapterStatus::Installed,
                    "generated adapter is installed".to_string(),
                )
            } else if has_marker(&existing) {
                (
                    HarnessAdapterStatus::Drifted,
                    "generated adapter has drifted from current policy".to_string(),
                )
            } else {
                (
                    HarnessAdapterStatus::UserOwned,
                    "file exists without Engram generated marker".to_string(),
                )
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (
            HarnessAdapterStatus::Missing,
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
        message,
    })
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

fn claude_settings_status(root: &Path) -> IndexResult<ClaudeSettingsStatus> {
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

    for (event, matcher) in claude_required_hook_events() {
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
                    .map(|settings| claude_settings_has_hook(settings, event, matcher))
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
    warn_for_settings_target(target, root, warnings)?;
    let mut settings = match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str::<Value>(&contents)
            .map_err(|e| IndexError::Parse(format!("failed to parse Claude settings: {e}")))?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => json!({}),
        Err(error) => return Err(error.into()),
    };

    let changed_permissions = merge_claude_permissions(&mut settings);
    let changed_hooks = merge_claude_hooks(&mut settings);
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
) -> IndexResult<()> {
    match target {
        HarnessSettingsTarget::Project => {
            let local = read_claude_settings_source(
                "settings.local.json",
                claude_local_settings_path(root),
            )?;
            if let Some(settings) = local.settings {
                let permissions = claude_engram_permissions(&settings);
                let has_hooks = claude_required_hook_events().into_iter().any(|(event, matcher)| {
                    claude_settings_has_hook(&settings, event, matcher)
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

fn merge_claude_hooks(settings: &mut Value) -> bool {
    ensure_object(settings);
    if settings.get("hooks").and_then(Value::as_object).is_none() {
        settings["hooks"] = json!({});
    }
    let mut changed = false;
    changed |= remove_claude_hook_handler(
        settings,
        "SessionEnd",
        None,
        &claude_mcp_hook_handler("SessionEnd"),
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
    for (event, matcher) in claude_mcp_hook_events() {
        changed |= ensure_claude_hook(settings, event, matcher, claude_mcp_hook_handler(event));
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
    let Some(groups) = settings
        .pointer_mut(&format!("/hooks/{event}"))
        .and_then(Value::as_array_mut)
    else {
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
    changed || groups.len() != before
}

fn claude_settings_has_hook(settings: &Value, event: &str, matcher: Option<&str>) -> bool {
    settings
        .pointer(&format!("/hooks/{event}"))
        .map(|entry| claude_event_has_handler(entry, matcher, &claude_expected_handler(event)))
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

fn claude_expected_handler(event: &str) -> Value {
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
        claude_mcp_hook_handler(event)
    }
}

fn claude_required_permissions() -> &'static [&'static str] {
    &[
        "mcp__engram__orient",
        "mcp__engram__memory",
        "mcp__engram__harness",
        "mcp__engram__lint",
        "mcp__engram__graph",
        "mcp__engram__handoff",
        "mcp__engram__obligations",
        "mcp__engram__telemetry",
        "mcp__engram__vault",
        "mcp__engram__digest",
        "mcp__engram__repo",
    ]
}

fn claude_required_hook_events() -> Vec<(&'static str, Option<&'static str>)> {
    let mut events = vec![("SessionStart", Some("startup|resume|compact"))];
    events.extend(claude_mcp_hook_events());
    events.push(("SessionEnd", None));
    events
}

fn claude_mcp_hook_events() -> Vec<(&'static str, Option<&'static str>)> {
    vec![
        ("UserPromptSubmit", None),
        ("PostToolUse", Some("Write|Edit|MultiEdit")),
        ("PostToolUseFailure", Some("*")),
        ("Stop", None),
        ("PreCompact", Some("manual|auto")),
        ("PostCompact", Some("manual|auto")),
    ]
}

fn claude_mcp_hook_handler(event: &str) -> Value {
    json!({
        "type": "mcp_tool",
        "server": "engram",
        "tool": "harness",
        "timeout": 10,
        "input": {
            "action": "hook_event",
            "harness": "claude_code",
            "hook_event_name": event,
            "session_id": "${session_id}",
            "cwd": "${cwd}",
            "transcript_path": "${transcript_path}",
            "prompt": "${prompt}",
            "tool_name": "${tool_name}",
            "tool_error": "${error}",
            "tool_input_command": "${tool_input.command}",
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
    if kind != HarnessAdapterKind::ClaudeHook {
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

fn claude_memory_session_command() -> String {
    format!(
        r#"{MARKER_MD}
# Engram Memory Session

Use this command when a Claude Code session needs persistent project memory.

Lifecycle contract:
- At task/session start, call `orient` with the project, cwd, prompt, and harness.
- Keep the returned `trace_id` from `orient` or `search`; before final response, call
  `telemetry(action=submit_feedback)` with `task_success`, `preference_adhered`,
  `repeated_context_questions`, `bad_memory_used`, `missing_context`, `used_memory_ids`, and
  `rejected_memory_ids`, plus `stale_memory_ids` and `wrong_scope_memory_ids` when those
  outcomes or attribution judgments can be made. Use `used_memory_ids` for returned memory that
  shaped the answer, implementation, safety decision, or plan; leave it empty only when no returned
  memory influenced behavior.
- Before major decisions, call `memory(action=changes_since)` with the orientation cursor.
- After non-obvious discoveries, record source-grounded memory or a session event.
- When the current method, plan, or next action should survive resume, use
  `memory(action=capture_current_plan)` with compact content and file/tool/manual-review evidence.
- Before final response, call `changes_since`; if relevant updates appeared, account for them.
- Before final response, call `obligations(action=detect, project=..., cwd=...)` and
  `obligations(action=doctor, project=..., cwd=...)`; resolve open obligations or report
  explicit skip reasons.
- Before context compaction, context transition, or any expected loss of conversation state,
  update `handoff` and record/commit compact durable memory for future sessions.
- At session end, compile a handoff and create a knowledge commit candidate.
- In commit workflows, consult memory for relevant preferences, rules, and limitations first.

This is a soft contract. Missing lifecycle steps should be reported as warnings, not blockers.
"#
    )
}

fn claude_resume_session_command() -> String {
    format!(
        r#"{MARKER_MD}
# Resume Engram Session

1. Call `orient` with the explicit project and current cwd.
2. Read the returned context pack, ambiguities, and memory cursor.
3. Keep returned `trace_id` values from `orient` or `search`; submit telemetry feedback with
   outcome, gap, and attribution fields before final response when memory quality can be judged.
   Include `used_memory_ids` for returned memory that shaped behavior and `rejected_memory_ids` for
   returned memory considered but not used. Include `stale_memory_ids` and
   `wrong_scope_memory_ids` for rejected memory specifically judged stale or out of scope.
4. If a rolling handoff exists, inspect `handoff(action=get)`.
5. Check `memory(action=changes_since)` during the session before major decisions.
6. Check `obligations(action=detect)` for document, tool-failure, source-reading, and design
   obligations; close or explicitly skip open items before final response.
7. Store only source-grounded decisions, rules, limitations, and non-obvious discoveries.
   Use `memory(action=capture_current_plan)` for compact current method, plan, or next-action
   guidance that should surface on the next resume.
8. If this is a resume after compaction, first inspect `handoff(action=get)` and recent
   `memory(action=changes_since)` before continuing.
"#
    )
}

fn claude_end_session_command() -> String {
    format!(
        r#"{MARKER_MD}
# End Engram Session

Before ending:
- Call `memory(action=changes_since)` from the latest cursor.
- Call `obligations(action=detect, project=..., cwd=...)` and
  `obligations(action=doctor, project=..., cwd=...)`.
- Resolve open obligations or state explicit skip reasons in the handoff.
- Update or compile `handoff` with completed work, open decisions, next actions, and risks.
- If durable memory changed, prepare a `memory(action=commit)` candidate.
- Use this same flow before context compaction or any context transition.
- Leave migration and digest promotions review-gated; do not auto-promote orphan data.
"#
    )
}

fn claude_session_start_hook() -> String {
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

PROJECT_NAME=""
if [ -n "$CWD" ] && [ "$CWD" != "null" ]; then
  PROJECT_NAME=$(basename "$CWD")
fi

CONTEXT="<engram_session_activation source=\"$SOURCE\" project=\"$PROJECT_NAME\" session_id=\"$SESSION_ID\">
Engram is the durable Memory OS for this Claude Code session.
Before making claims or edits, call the Engram MCP orient tool with project, cwd, prompt, and agent=claude_code.
Keep the returned memory cursor and use memory(action=changes_since) before major decisions and before final response.
Keep returned trace_id values from orient/search and submit telemetry(action=submit_feedback) with task_success, preference_adhered, repeated_context_questions, bad_memory_used, missing_context, used_memory_ids, rejected_memory_ids, stale_memory_ids, and wrong_scope_memory_ids before final response when those outcomes or attribution judgments can be made.
Use used_memory_ids for returned memory that shaped the answer, implementation, safety decision, or plan; leave it empty only when no returned memory influenced behavior.
Use obligations(action=detect) for source/design reading, durable document disposition, failed tool recovery, verification, handoff, and commit preference checks.
When the current method, plan, or next action should survive resume, use memory(action=capture_current_plan) with compact content and file/tool/manual-review evidence.
Before context compaction or session end, update handoff and commit compact durable memory when useful.
This is a soft contract: resolve obligations or state explicit skip reasons; do not fabricate missing memory.
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

fn claude_stop_nudge_hook() -> String {
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
  "systemMessage": "Engram final-response check: call memory(action=changes_since), obligations(action=detect, project=..., cwd=...), and obligations(action=doctor, project=..., cwd=...); submit telemetry(action=submit_feedback) with task_success, preference_adhered, repeated_context_questions, bad_memory_used, missing_context, used_memory_ids, rejected_memory_ids, stale_memory_ids, and wrong_scope_memory_ids for relevant trace_id values when those outcomes or attribution judgments can be made; resolve or explicitly skip open obligations, update handoff if context would be lost, then answer."
}}
EOF
"#
    )
}

fn claude_session_end_hook() -> String {
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
  '{jsonrpc:"2.0",id:2,method:"tools/call",params:{name:"harness",arguments:{action:"hook_event",harness:"claude_code",hook_event_name:"SessionEnd",session_id:$session_id,cwd:$cwd,transcript_path:$transcript_path,reason:$reason,write_policy:$write_policy,model_provider:"anthropic",model:"claude-code",surface:"claude-code",actor:"agent"}}}')

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

    format!("#!/usr/bin/env bash\n{MARKER_SH}\n{body}")
}

fn claude_settings_snippet() -> String {
    serde_json::to_string_pretty(&json!({
        "permissions": {
            "allow": claude_required_permissions()
        },
        "hooks": {
            "SessionStart": [{
                "matcher": "startup|resume|compact",
                "hooks": [{
                    "type": "command",
                    "command": CLAUDE_HOOK_COMMAND,
                    "timeout": 10
                }]
            }],
            "UserPromptSubmit": [{
                "hooks": [claude_mcp_hook_handler("UserPromptSubmit")]
            }],
            "PostToolUse": [{
                "matcher": "Write|Edit|MultiEdit",
                "hooks": [claude_mcp_hook_handler("PostToolUse")]
            }],
            "PostToolUseFailure": [{
                "matcher": "*",
                "hooks": [claude_mcp_hook_handler("PostToolUseFailure")]
            }],
            "Stop": [{
                "hooks": [claude_mcp_hook_handler("Stop")]
            }],
            "PreCompact": [{
                "matcher": "manual|auto",
                "hooks": [claude_mcp_hook_handler("PreCompact")]
            }],
            "PostCompact": [{
                "matcher": "manual|auto",
                "hooks": [claude_mcp_hook_handler("PostCompact")]
            }],
            "SessionEnd": [{
                "hooks": [{
                    "type": "command",
                    "command": CLAUDE_SESSION_END_HOOK_COMMAND,
                    "timeout": 15
                }]
            }]
        }
    }))
    .expect("Claude settings snippet should serialize")
}

fn codex_memory_session_skill() -> String {
    format!(
        r#"{MARKER_MD}
# Engram Memory Session

Use when Codex is working in a repo or project with persistent Engram memory.

Workflow:
- Start by calling `orient` with project, cwd, prompt, and `agent=codex`.
- Treat the returned memory cursor as the baseline for this turn.
- Keep the returned `trace_id` from `orient` or `search`; before final response, call
  `telemetry(action=submit_feedback)` with `task_success`, `preference_adhered`,
  `repeated_context_questions`, `bad_memory_used`, `missing_context`, `used_memory_ids`, and
  `rejected_memory_ids`, plus `stale_memory_ids` and `wrong_scope_memory_ids` when those
  outcomes or attribution judgments can be made. Use `used_memory_ids` for returned memory that
  shaped the answer, implementation, safety decision, or plan; leave it empty only when no returned
  memory influenced behavior.
- Before a major decision or final response, call `memory(action=changes_since)`.
- Record source-grounded discoveries, decisions, rules, preferences, limitations, and handoffs.
- When the current method, plan, or next action should survive resume, use
  `memory(action=capture_current_plan)` with compact content and file/tool/manual-review evidence.
- When you create or update a durable project document, call
  `obligations(action=detect, write=true, project=..., cwd=...)` so the document disposition is
  persisted instead of only observed. Resolve each document obligation by indexing it with
  `docs(action=index)`, registering it with `knowledge(action=register)`, recording compact memory
  with `memory(action=capture_current_plan)`, linking it in `handoff`, or explicitly skipping it
  with a reason.
- Before final response, run `obligations(action=detect, write=true, project=..., cwd=...)` and
  `obligations(action=doctor, project=..., cwd=...)`; resolve or explicitly skip open
  obligations. If a document changes again after resolution, rerun detection so a fresh content
  state gets its own disposition.
- Before Codex context compaction or any expected context loss, update `handoff` and record or
  commit compact durable memory so the next Codex session can resume without the transcript.
- For commit messages, check memory for user/project commit preferences first.
- If handoff or durable memory changes are needed, use `handoff` and `memory(action=commit)`.

This skill is advisory. Report skipped lifecycle steps as warnings; do not block coding work.
"#
    )
}

fn codex_resume_session_skill() -> String {
    format!(
        r#"{MARKER_MD}
# Engram Resume Session

Use when the user asks to continue, resume, or load prior Engram context.

Steps:
- Call `orient` before reading broad files.
- Inspect project/repository resolution and ask only if ambiguity cannot be resolved.
- Keep returned `trace_id` values from `orient` or `search`; submit telemetry feedback with
  outcome, gap, and attribution fields before final response when memory quality can be judged.
  Include `used_memory_ids` for returned memory that shaped behavior and `rejected_memory_ids` for
  returned memory considered but not used. Include `stale_memory_ids` and
  `wrong_scope_memory_ids` for rejected memory specifically judged stale or out of scope.
- Use `handoff(action=get)` when available.
- Poll `memory(action=changes_since)` before major decisions and final response.
- Poll `obligations(action=detect, write=true, project=..., cwd=...)` before final response so
  changed durable documents become persisted obligations, then close them by indexing,
  registering, recording compact memory, handoff-linking, or explicitly skipping with a reason.
- Store compact, evidenced memory if the session discovered something future agents need.
- Use `memory(action=capture_current_plan)` for compact current method, plan, or next-action
  guidance that should surface on the next resume.
- If resuming after compaction, read `handoff(action=get)` and recent
  `memory(action=changes_since)` before continuing.
"#
    )
}

fn gemini_memory_session_command() -> String {
    format!(
        r#"{MARKER_SH}
description = "Follow the Engram Memory OS lifecycle contract."
prompt = """
# Engram Memory Session

You are Gemini CLI working in a repository or project with persistent Engram memory.
This command is invoked as `/engram:memory-session`.

Follow this soft lifecycle contract:
- Start by calling the Engram MCP `orient` tool with project, cwd, prompt, and `agent=gemini_cli`.
- Treat the returned memory cursor as the baseline for this turn.
- Keep the returned `trace_id` from `orient` or `search`; before final response, call
  `telemetry(action=submit_feedback)` with `task_success`, `preference_adhered`,
  `repeated_context_questions`, `bad_memory_used`, `missing_context`, `used_memory_ids`, and
  `rejected_memory_ids`, plus `stale_memory_ids` and `wrong_scope_memory_ids` when those
  outcomes or attribution judgments can be made. Use `used_memory_ids` for returned memory that
  shaped the answer, implementation, safety decision, or plan; leave it empty only when no returned
  memory influenced behavior.
- Before a major decision or final response, call `memory(action=changes_since)`.
- Record source-grounded discoveries, decisions, rules, preferences, limitations, and handoffs.
- When the current method, plan, or next action should survive resume, use
  `memory(action=capture_current_plan)` with compact content and file/tool/manual-review evidence.
- Use `obligations(action=detect, project=..., cwd=...)` when documents change, tools fail,
  or source/design reading is needed; before final response, run
  `obligations(action=doctor, project=..., cwd=...)` and resolve or explicitly skip open
  obligations.
- Before context compaction or any expected context loss, update `handoff` and record or commit
  compact durable memory for the next session.
- For commit messages, check memory for user/project commit preferences first.
- If handoff or durable memory changes are needed, use `handoff` and `memory(action=commit)`.

This contract is advisory. Report skipped lifecycle steps as warnings; do not block coding work.
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
- Call the Engram MCP `orient` tool before reading broad files.
- Inspect project/repository resolution and ask only if ambiguity cannot be resolved.
- Keep returned `trace_id` values from `orient` or `search`; submit telemetry feedback with
  outcome, gap, and attribution fields before final response when memory quality can be judged.
  Include `used_memory_ids` for returned memory that shaped behavior and `rejected_memory_ids` for
  returned memory considered but not used. Include `stale_memory_ids` and
  `wrong_scope_memory_ids` for rejected memory specifically judged stale or out of scope.
- Use `handoff(action=get)` when available.
- Poll `memory(action=changes_since)` before major decisions and final response.
- Poll `obligations(action=detect)` and close or explicitly skip open obligations before final
  response.
- Store compact, evidenced memory if the session discovered something future agents need.
- Use `memory(action=capture_current_plan)` for compact current method, plan, or next-action
  guidance that should surface on the next resume.
- If resuming after compaction, read `handoff(action=get)` and recent
  `memory(action=changes_since)` before continuing.
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
- Call `memory(action=changes_since)` from the latest cursor.
- Call `obligations(action=detect, project=..., cwd=...)` and
  `obligations(action=doctor, project=..., cwd=...)`.
- Resolve open obligations or state explicit skip reasons in the handoff.
- Update or compile `handoff` with completed work, open decisions, next actions, and risks.
- If durable memory changed, prepare a `memory(action=commit)` candidate.
- Use this same flow before context compaction or any context transition.
- Leave migration and digest promotions review-gated; do not auto-promote orphan data.
"""
"#
    )
}

fn gemini_global_context() -> String {
    format!(
        r#"{MARKER_MD}
# Engram Memory OS Harness

Gemini CLI should treat Engram as persistent project memory when Engram MCP tools are available.

- Start work by calling `orient` with the current project, cwd, prompt, and `agent=gemini_cli`.
- Keep the returned memory cursor and call `memory(action=changes_since)` before major
  decisions, before final response, and during long sessions.
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
- Treat lifecycle enforcement as soft: warn about skipped steps, but do not block coding.

Useful commands when installed:
- `/engram:memory-session`
- `/engram:resume-session`
- `/engram:end-session`
"#
    )
}

fn cursor_memory_session_skill() -> String {
    format!(
        r#"{MARKER_MD}
---
name: engram-memory-session
description: Use when Cursor Agent is working in a repo or project with persistent Engram memory, especially at task start, before major decisions, before final responses, or when commit preferences may matter.
---
# Engram Memory Session

Use this skill when Cursor Agent is working in a repository or project with persistent Engram memory.

Workflow:
- Start by calling the Engram MCP `orient` tool with project, cwd, prompt, and `agent=cursor`.
- Treat the returned memory cursor as the baseline for this turn.
- Keep the returned `trace_id` from `orient` or `search`; before final response, call
  `telemetry(action=submit_feedback)` with `task_success`, `preference_adhered`,
  `repeated_context_questions`, `bad_memory_used`, `missing_context`, `used_memory_ids`, and
  `rejected_memory_ids`, plus `stale_memory_ids` and `wrong_scope_memory_ids` when those
  outcomes or attribution judgments can be made. Use `used_memory_ids` for returned memory that
  shaped the answer, implementation, safety decision, or plan; leave it empty only when no returned
  memory influenced behavior.
- Before a major decision or final response, call `memory(action=changes_since)`.
- Record source-grounded discoveries, decisions, rules, preferences, limitations, and handoffs.
- When the current method, plan, or next action should survive resume, use
  `memory(action=capture_current_plan)` with compact content and file/tool/manual-review evidence.
- Use `obligations(action=detect, project=..., cwd=...)` when documents change, tools fail,
  or source/design reading is needed; before final response, run
  `obligations(action=doctor, project=..., cwd=...)` and resolve or explicitly skip open
  obligations.
- Before context compaction or any expected context loss, update `handoff` and record or commit
  compact durable memory for the next session.
- Use writer provenance with `writer_harness=cursor` when writing durable memory.
- For commit messages, check memory for user/project commit preferences first.
- If handoff or durable memory changes are needed, use `handoff` and `memory(action=commit)`.

This skill is advisory. Report skipped lifecycle steps as warnings; do not block coding work.
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
- Call the Engram MCP `orient` tool before reading broad files.
- Inspect project/repository resolution and ask only if ambiguity cannot be resolved.
- Keep returned `trace_id` values from `orient` or `search`; submit telemetry feedback with
  outcome, gap, and attribution fields before final response when memory quality can be judged.
  Include `used_memory_ids` for returned memory that shaped behavior and `rejected_memory_ids` for
  returned memory considered but not used. Include `stale_memory_ids` and
  `wrong_scope_memory_ids` for rejected memory specifically judged stale or out of scope.
- Use `handoff(action=get)` when available.
- Poll `memory(action=changes_since)` before major decisions and final response.
- Poll `obligations(action=detect)` and close or explicitly skip open obligations before final
  response.
- Store compact, evidenced memory if the session discovered something future agents need.
- Use `memory(action=capture_current_plan)` for compact current method, plan, or next-action
  guidance that should surface on the next resume.
- If resuming after compaction, read `handoff(action=get)` and recent
  `memory(action=changes_since)` before continuing.
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
- Call `memory(action=changes_since)` from the latest cursor.
- Call `obligations(action=detect, project=..., cwd=...)` and
  `obligations(action=doctor, project=..., cwd=...)`.
- Resolve open obligations or state explicit skip reasons in the handoff.
- Update or compile `handoff` with completed work, open decisions, next actions, and risks.
- If durable memory changed, prepare a `memory(action=commit)` candidate.
- Use this same flow before context compaction or any context transition.
- Use writer provenance with `writer_harness=cursor`.
- Leave migration and digest promotions review-gated; do not auto-promote orphan data.
"#
    )
}

fn agents_snippet() -> String {
    format!(
        r#"{MARKER_MD}
# Engram Memory OS Harness

- Start work by calling `orient` with the current project, cwd, prompt, and harness name.
- Keep the returned memory cursor and call `memory(action=changes_since)` before major
  decisions, before final response, and during long sessions.
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
- Treat lifecycle enforcement as soft: warn about skipped steps, but do not block coding.
"#
    )
}

fn generic_policy_document() -> String {
    format!(
        r#"{MARKER_MD}
# Engram Generic Harness Policy

Required MCP tools: orient, memory, harness, lint, graph, handoff, obligations, telemetry, vault.

Lifecycle:
- task/session start: call `orient`
- before major decisions: call `memory(action=changes_since)`
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

Enforcement is soft. Missing lifecycle steps produce warnings, not hard blocks.
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(contents.contains("changes_since"));
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
    fn policy_requires_telemetry_tool_for_feedback() {
        let policy = HarnessService::new().policy(HarnessKind::Codex);
        assert!(policy.required_mcp_tools.contains(&"telemetry".to_string()));
    }

    #[test]
    fn render_adapter_mentions_feedback_trace_id() {
        let adapters = HarnessService::new()
            .render_adapters(HarnessKind::Codex, Some("codex-memory-session-skill"));
        assert_eq!(adapters.len(), 1);
        assert!(adapters[0].contents.contains("trace_id"));
        assert!(adapters[0]
            .contents
            .contains("telemetry(action=submit_feedback)"));
        assert!(adapters[0].contents.contains("task_success"));
        assert!(adapters[0].contents.contains("missing_context"));
        assert!(adapters[0].contents.contains("used_memory_ids"));
        assert!(adapters[0].contents.contains("rejected_memory_ids"));
        assert!(adapters[0].contents.contains("stale_memory_ids"));
        assert!(adapters[0].contents.contains("wrong_scope_memory_ids"));
    }

    #[test]
    fn render_codex_adapter_spells_out_document_lifecycle_disposition() {
        let adapters = HarnessService::new()
            .render_adapters(HarnessKind::Codex, Some("codex-memory-session-skill"));
        assert_eq!(adapters.len(), 1);
        let contents = &adapters[0].contents;

        assert!(contents.contains("obligations(action=detect, write=true"));
        assert!(contents.contains("obligations(action=doctor, project=..., cwd=...)"));
        assert!(contents.contains("docs(action=index)"));
        assert!(contents.contains("knowledge(action=register)"));
        assert!(contents.contains("memory(action=capture_current_plan)"));
        assert!(contents.contains("explicitly skipping it"));
        assert!(contents.contains("fresh content"));
        assert!(contents.contains("state gets its own disposition"));
    }

    #[test]
    fn codex_policy_mentions_context_compaction_save() {
        let policy = HarnessService::new()
            .render_policy(HarnessKind::Codex)
            .unwrap();
        assert!(policy.contains("before_context_compaction_save"));
        assert!(policy.contains("\"telemetry\""));

        let adapters = HarnessService::new()
            .render_adapters(HarnessKind::Codex, Some("codex-memory-session-skill"));
        assert_eq!(adapters.len(), 1);
        assert!(adapters[0].contents.contains("context compaction"));
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
        let stale_session_end_handler = claude_mcp_hook_handler("SessionEnd");
        fs::write(
            root.path().join(".claude/settings.json"),
            serde_json::to_string(&serde_json::json!({
                "hooks": {
                    "UserPromptSubmit": [{
                        "hooks": [{
                            "type": "command",
                            "command": "existing"
                        }]
                    }],
                    "SessionEnd": [{
                        "hooks": [stale_session_end_handler]
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
                },
            )
            .unwrap();

        assert!(report
            .written
            .iter()
            .any(|file| file.name == "claude-settings-merge"));
        let settings = fs::read_to_string(root.path().join(".claude/settings.json")).unwrap();
        assert!(settings.contains("mcp__engram__orient"));
        assert!(settings.contains("mcp__engram__telemetry"));
        assert!(settings.contains("\"PostToolUseFailure\""));
        assert!(settings.contains("existing"));
        let settings_json: Value = serde_json::from_str(&settings).unwrap();
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
            .contains("obligations(action=detect, project=..., cwd=...)"));
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
    async fn hook_event_stop_nudges_without_blocking_when_obligations_are_open() {
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
        assert!(stop_outcome.response.get("hookSpecificOutput").is_none());
        assert!(stop_outcome.response["systemMessage"]
            .as_str()
            .unwrap()
            .contains("without blocking the user"));

        let active_stop = service
            .handle_hook_event(
                HarnessHookEvent {
                    harness: HarnessKind::ClaudeCode,
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
