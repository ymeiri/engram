//! Agent harness policy rendering and installation.
//!
//! This module is deliberately filesystem-only. It does not persist state in the
//! Engram database; instead it renders a stable policy and verifies local adapter
//! files that make Engram's lifecycle visible to agent surfaces.

use crate::error::{IndexError, IndexResult};
use engram_core::harness::{
    HarnessAdapterCheck, HarnessAdapterKind, HarnessAdapterSpec, HarnessAdapterStatus,
    HarnessInstallFile, HarnessInstallReport, HarnessKind, HarnessLifecycleTrigger, HarnessPolicy,
    HarnessRenderedAdapter, HarnessStatusReport,
};
use std::fs;
use std::path::{Path, PathBuf};

const MARKER_MD: &str = "<!-- engram:harness-adapter:v1 -->";
const MARKER_SH: &str = "# engram:harness-adapter:v1";

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

        let ready = adapters
            .iter()
            .filter(|check| check.required)
            .all(|check| check.status == HarnessAdapterStatus::Installed)
            && missing_mcp_tools.is_empty();

        Ok(HarnessStatusReport {
            harness,
            root: root.display().to_string(),
            policy,
            adapters,
            missing_mcp_tools,
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
        let root = resolve_root(root)?;
        let mut planned = Vec::new();
        let mut written = Vec::new();
        let mut skipped = Vec::new();
        let mut warnings = Vec::new();

        for adapter in self.policy(harness).adapters {
            let path = root.join(&adapter.relative_path);
            let check = check_adapter(&root, &adapter)?;
            let plan_message = install_plan_message(check.status, write);
            let plan = HarnessInstallFile {
                name: adapter.name.clone(),
                path: path.display().to_string(),
                written: false,
                message: plan_message,
            };
            planned.push(plan.clone());

            if !write {
                continue;
            }

            match check.status {
                HarnessAdapterStatus::Missing | HarnessAdapterStatus::Drifted => {
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

        Ok(HarnessInstallReport {
            harness,
            root: root.display().to_string(),
            dry_run: !write,
            planned,
            written,
            skipped,
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
        HarnessLifecycleTrigger::SessionEndHandoff,
        HarnessLifecycleTrigger::CommitWorkflowConsultMemory,
    ]
}

fn required_mcp_tools() -> Vec<String> {
    [
        "orient", "memory", "harness", "lint", "graph", "handoff", "vault",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn adapters_for(harness: HarnessKind) -> Vec<HarnessAdapterSpec> {
    match harness {
        HarnessKind::ClaudeCode => claude_adapters(),
        HarnessKind::Codex => codex_adapters(),
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

fn install_plan_message(status: HarnessAdapterStatus, write: bool) -> String {
    match (status, write) {
        (_, false) => "dry-run; no file will be written".to_string(),
        (HarnessAdapterStatus::Missing, true) => "will create generated adapter".to_string(),
        (HarnessAdapterStatus::Drifted, true) => "will update generated adapter".to_string(),
        (HarnessAdapterStatus::Installed, true) => "already installed".to_string(),
        (HarnessAdapterStatus::UserOwned, true) => {
            "will skip user-owned file without Engram marker".to_string()
        }
    }
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
- Before major decisions, call `memory(action=changes_since)` with the orientation cursor.
- After non-obvious discoveries, record source-grounded memory or a session event.
- Before final response, call `changes_since`; if relevant updates appeared, account for them.
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
3. If a rolling handoff exists, inspect `handoff(action=get)`.
4. Check `memory(action=changes_since)` during the session before major decisions.
5. Store only source-grounded decisions, rules, limitations, and non-obvious discoveries.
"#
    )
}

fn claude_end_session_command() -> String {
    format!(
        r#"{MARKER_MD}
# End Engram Session

Before ending:
- Call `memory(action=changes_since)` from the latest cursor.
- Update or compile `handoff` with completed work, open decisions, next actions, and risks.
- If durable memory changed, prepare a `memory(action=commit)` candidate.
- Leave migration and digest promotions review-gated; do not auto-promote orphan data.
"#
    )
}

fn claude_session_start_hook() -> String {
    format!(
        r#"{MARKER_SH}
#!/usr/bin/env bash
set -euo pipefail

printf '%s\n' \
  'Engram: start by calling orient with project/cwd/prompt, then keep the memory cursor for changes_since.'
"#
    )
}

fn claude_stop_nudge_hook() -> String {
    format!(
        r#"{MARKER_SH}
#!/usr/bin/env bash
set -euo pipefail

printf '%s\n' \
  'Engram: before final response, check changes_since, record non-obvious discoveries, and update handoff if needed.'
"#
    )
}

fn codex_memory_session_skill() -> String {
    format!(
        r#"{MARKER_MD}
# Engram Memory Session

Use when Codex is working in a repo or project with persistent Engram memory.

Workflow:
- Start by calling `orient` with project, cwd, prompt, and `agent=codex`.
- Treat the returned memory cursor as the baseline for this turn.
- Before a major decision or final response, call `memory(action=changes_since)`.
- Record source-grounded discoveries, decisions, rules, preferences, limitations, and handoffs.
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
- Use `handoff(action=get)` when available.
- Poll `memory(action=changes_since)` before major decisions and final response.
- Store compact, evidenced memory if the session discovered something future agents need.
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
- Record source-grounded decisions, preferences, rules, limitations, and non-obvious
  discoveries. Use writer provenance so Claude Code, Codex, and other harnesses can be
  distinguished.
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

Required MCP tools: orient, memory, harness, lint, graph, handoff, vault.

Lifecycle:
- task/session start: call `orient`
- before major decisions: call `memory(action=changes_since)`
- after non-obvious discoveries: record memory/session event
- before final response: call `changes_since` and distill if needed
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
    fn render_adapter_mentions_commit_preferences() {
        let adapters = HarnessService::new()
            .render_adapters(HarnessKind::Codex, Some("codex-memory-session-skill"));
        assert_eq!(adapters.len(), 1);
        assert!(adapters[0].contents.contains("commit preferences"));
    }
}
