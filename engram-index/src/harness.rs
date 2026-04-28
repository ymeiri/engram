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
            match check.status {
                HarnessAdapterStatus::Missing | HarnessAdapterStatus::Drifted => {
                    planned.push(HarnessInstallFile {
                        name: adapter.name.clone(),
                        path: path.display().to_string(),
                        written: false,
                        message: install_plan_message(check.status),
                    });
                    if !write {
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
        HarnessLifecycleTrigger::BeforeFinalObligations,
        HarnessLifecycleTrigger::SessionEndHandoff,
        HarnessLifecycleTrigger::CommitWorkflowConsultMemory,
    ]
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
- Before final response, call `obligations(action=detect)` and `obligations(action=doctor)`;
  resolve open obligations or report explicit skip reasons.
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
5. Check `obligations(action=detect)` for document, tool-failure, source-reading, and design
   obligations; close or explicitly skip open items before final response.
6. Store only source-grounded decisions, rules, limitations, and non-obvious discoveries.
"#
    )
}

fn claude_end_session_command() -> String {
    format!(
        r#"{MARKER_MD}
# End Engram Session

Before ending:
- Call `memory(action=changes_since)` from the latest cursor.
- Call `obligations(action=detect)` and `obligations(action=doctor)`.
- Resolve open obligations or state explicit skip reasons in the handoff.
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
  'Engram: before final response, check changes_since and obligations doctor; resolve or explicitly skip open obligations.'
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
- Use `obligations(action=detect)` when documents change, tools fail, or source/design reading
  is needed; before final response, run `obligations(action=doctor)` and resolve or explicitly
  skip open obligations.
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
- Poll `obligations(action=detect)` and close or explicitly skip open obligations before final
  response.
- Store compact, evidenced memory if the session discovered something future agents need.
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
- Before a major decision or final response, call `memory(action=changes_since)`.
- Record source-grounded discoveries, decisions, rules, preferences, limitations, and handoffs.
- Use `obligations(action=detect)` when documents change, tools fail, or source/design reading
  is needed; before final response, run `obligations(action=doctor)` and resolve or explicitly
  skip open obligations.
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
- Use `handoff(action=get)` when available.
- Poll `memory(action=changes_since)` before major decisions and final response.
- Poll `obligations(action=detect)` and close or explicitly skip open obligations before final
  response.
- Store compact, evidenced memory if the session discovered something future agents need.
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
- Call `obligations(action=detect)` and `obligations(action=doctor)`.
- Resolve open obligations or state explicit skip reasons in the handoff.
- Update or compile `handoff` with completed work, open decisions, next actions, and risks.
- If durable memory changed, prepare a `memory(action=commit)` candidate.
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
- Record source-grounded decisions, preferences, rules, limitations, and non-obvious
  discoveries. Use writer provenance so Gemini CLI, Claude Code, Codex, and other harnesses
  can be distinguished.
- Detect and close agent obligations before final response: document dispositions, failed tool
  recovery, source/design reading, verification, handoff, and commit-preference checks.
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
- Before a major decision or final response, call `memory(action=changes_since)`.
- Record source-grounded discoveries, decisions, rules, preferences, limitations, and handoffs.
- Use `obligations(action=detect)` when documents change, tools fail, or source/design reading
  is needed; before final response, run `obligations(action=doctor)` and resolve or explicitly
  skip open obligations.
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
- Use `handoff(action=get)` when available.
- Poll `memory(action=changes_since)` before major decisions and final response.
- Poll `obligations(action=detect)` and close or explicitly skip open obligations before final
  response.
- Store compact, evidenced memory if the session discovered something future agents need.
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
- Call `obligations(action=detect)` and `obligations(action=doctor)`.
- Resolve open obligations or state explicit skip reasons in the handoff.
- Update or compile `handoff` with completed work, open decisions, next actions, and risks.
- If durable memory changed, prepare a `memory(action=commit)` candidate.
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
- Record source-grounded decisions, preferences, rules, limitations, and non-obvious
  discoveries. Use writer provenance so Claude Code, Codex, and other harnesses can be
  distinguished.
- Use `obligations(action=detect)` at task start and before final response. Resolve or explicitly
  skip document, failed-tool, source/design reading, verification, handoff, and commit-preference
  obligations before claiming the task is done.
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

Required MCP tools: orient, memory, harness, lint, graph, handoff, obligations, vault.

Lifecycle:
- task/session start: call `orient`
- before major decisions: call `memory(action=changes_since)`
- after non-obvious discoveries: record memory/session event
- before final response: call `changes_since` and distill if needed
- before final response: detect obligations, run obligations doctor, and close or explicitly skip
  open obligations
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
    fn render_adapter_mentions_commit_preferences() {
        let adapters = HarnessService::new()
            .render_adapters(HarnessKind::Codex, Some("codex-memory-session-skill"));
        assert_eq!(adapters.len(), 1);
        assert!(adapters[0].contents.contains("commit preferences"));
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
}
