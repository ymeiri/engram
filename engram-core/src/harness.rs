//! Agent harness policy types.
//!
//! The harness layer describes the lifecycle steps an agent should follow and
//! the local adapters that make those steps discoverable or enforceable.

use serde::{Deserialize, Serialize};

/// Agent harness supported by first-class adapter rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum HarnessKind {
    /// Claude Code.
    ClaudeCode,
    /// Codex.
    Codex,
    /// Gemini CLI.
    GeminiCli,
    /// Cursor Agent.
    Cursor,
    /// Generic policy with no surface-specific files.
    #[default]
    Generic,
}

impl std::fmt::Display for HarnessKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ClaudeCode => write!(f, "claude_code"),
            Self::Codex => write!(f, "codex"),
            Self::GeminiCli => write!(f, "gemini_cli"),
            Self::Cursor => write!(f, "cursor"),
            Self::Generic => write!(f, "generic"),
        }
    }
}

impl HarnessKind {
    /// Parse a harness name.
    #[must_use]
    pub fn parse(value: &str) -> Self {
        match value.to_lowercase().replace('-', "_").as_str() {
            "claude" | "claude_code" => Self::ClaudeCode,
            "codex" => Self::Codex,
            "gemini" | "gemini_cli" => Self::GeminiCli,
            "cursor" | "cursor_agent" => Self::Cursor,
            _ => Self::Generic,
        }
    }
}

/// Harness lifecycle enforcement profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum HarnessEnforcementProfile {
    /// Advisory-only lifecycle. Hooks never block agent work.
    Soft,
    /// GA default: block high-value boundaries once, with an escape for host loops.
    #[default]
    Graduated,
    /// Strict lifecycle. Hooks keep blocking until obligations are closed or Engram is degraded.
    Strict,
}

impl std::fmt::Display for HarnessEnforcementProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Soft => write!(f, "soft"),
            Self::Graduated => write!(f, "graduated"),
            Self::Strict => write!(f, "strict"),
        }
    }
}

impl HarnessEnforcementProfile {
    /// Parse an enforcement profile name.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.to_lowercase().replace('-', "_").as_str() {
            "soft" | "advisory" => Ok(Self::Soft),
            "graduated" | "default" | "ga" => Ok(Self::Graduated),
            "strict" | "hard" => Ok(Self::Strict),
            _ => Err(format!(
                "invalid harness enforcement '{value}'; expected soft, graduated, or strict"
            )),
        }
    }

    /// True when the profile is advisory only.
    #[must_use]
    pub const fn is_soft(self) -> bool {
        matches!(self, Self::Soft)
    }
}

/// Lifecycle trigger expected from an agent harness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarnessLifecycleTrigger {
    /// At task/session start, orient before acting.
    TaskStartOrient,
    /// Before major decisions, check whether other writers added relevant memory.
    BeforeMajorDecisionChangesSince,
    /// After non-obvious discoveries, record source-grounded memory or session events.
    AfterDiscoveryRecord,
    /// Before the final response, check for memory changes and distill if needed.
    BeforeFinalChangesSince,
    /// Before the final response, detect and close open agent obligations.
    BeforeFinalObligations,
    /// Before context compaction or context loss, persist useful state to Engram.
    BeforeContextCompactionSave,
    /// At session end, compile a handoff and knowledge commit candidate.
    SessionEndHandoff,
    /// Before commit messages, consult relevant preferences and rules.
    CommitWorkflowConsultMemory,
}

impl std::fmt::Display for HarnessLifecycleTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TaskStartOrient => write!(f, "task_start_orient"),
            Self::BeforeMajorDecisionChangesSince => {
                write!(f, "before_major_decision_changes_since")
            }
            Self::AfterDiscoveryRecord => write!(f, "after_discovery_record"),
            Self::BeforeFinalChangesSince => write!(f, "before_final_changes_since"),
            Self::BeforeFinalObligations => write!(f, "before_final_obligations"),
            Self::BeforeContextCompactionSave => write!(f, "before_context_compaction_save"),
            Self::SessionEndHandoff => write!(f, "session_end_handoff"),
            Self::CommitWorkflowConsultMemory => write!(f, "commit_workflow_consult_memory"),
        }
    }
}

/// Local adapter file type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarnessAdapterKind {
    /// Claude Code slash command.
    ClaudeCommand,
    /// Claude Code hook script.
    ClaudeHook,
    /// Codex skill.
    CodexSkill,
    /// Gemini CLI custom command.
    GeminiCommand,
    /// Gemini CLI context file.
    GeminiContext,
    /// Cursor Agent skill.
    CursorSkill,
    /// Project instruction snippet.
    ProjectInstructions,
    /// Generic policy document.
    PolicyDocument,
}

/// Status of a local adapter file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarnessAdapterStatus {
    /// File is missing.
    Missing,
    /// File exists and matches generated content.
    Installed,
    /// File has an Engram marker but does not match current generated content.
    Drifted,
    /// File exists but lacks the generated marker and is treated as user-owned.
    UserOwned,
}

/// Adapter file rendered by a harness policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessAdapterSpec {
    /// Stable adapter name.
    pub name: String,
    /// Adapter kind.
    pub kind: HarnessAdapterKind,
    /// Path relative to the selected install root.
    pub relative_path: String,
    /// Human-readable purpose.
    pub description: String,
    /// Whether this adapter is part of the first-class contract for the harness.
    pub required: bool,
    /// Generated file contents.
    pub contents: String,
}

/// Verification result for one adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessAdapterCheck {
    /// Stable adapter name.
    pub name: String,
    /// Adapter kind.
    pub kind: HarnessAdapterKind,
    /// Absolute path checked.
    pub path: String,
    /// Status found on disk.
    pub status: HarnessAdapterStatus,
    /// Whether this adapter is required.
    pub required: bool,
    /// Explanation for humans and agents.
    pub message: String,
}

/// Harness policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessPolicy {
    /// Harness the policy targets.
    pub harness: HarnessKind,
    /// Canonical lifecycle enforcement profile.
    pub enforcement_profile: HarnessEnforcementProfile,
    /// Whether lifecycle enforcement is advisory rather than blocking.
    pub soft_contract: bool,
    /// Lifecycle triggers agents should follow.
    pub lifecycle_triggers: Vec<HarnessLifecycleTrigger>,
    /// MCP tools expected from Engram.
    pub required_mcp_tools: Vec<String>,
    /// Local adapters for this harness.
    pub adapters: Vec<HarnessAdapterSpec>,
}

/// Structured lifecycle compliance state for a harness report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessLifecycleReport {
    /// Canonical lifecycle enforcement profile.
    pub enforcement_profile: HarnessEnforcementProfile,
    /// Whether lifecycle enforcement is advisory rather than blocking.
    pub soft_contract: bool,
    /// Whether Engram enforces lifecycle compliance as a hard runtime gate.
    pub enforced: bool,
    /// Lifecycle triggers that agents should follow when enforcement is advisory.
    pub advisory_triggers: Vec<HarnessLifecycleTrigger>,
    /// Human-readable summary.
    pub message: String,
}

/// Structured MCP tool availability state for a harness report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessMcpToolReport {
    /// Whether observed MCP tool names were supplied by the caller.
    pub checked: bool,
    /// Required MCP tools from the harness policy.
    pub required_tools: Vec<String>,
    /// Observed MCP tool names supplied by the caller.
    pub observed_tools: Vec<String>,
    /// Required MCP tools that were not observed when `checked` is true.
    pub missing_tools: Vec<String>,
    /// Human-readable summary.
    pub message: String,
}

/// Harness status/doctor report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessStatusReport {
    /// Harness checked.
    pub harness: HarnessKind,
    /// Install root.
    pub root: String,
    /// Policy used for verification.
    pub policy: HarnessPolicy,
    /// Adapter checks.
    pub adapters: Vec<HarnessAdapterCheck>,
    /// Missing required MCP tools, if the caller provided observed tool names.
    pub missing_mcp_tools: Vec<String>,
    /// Structured MCP tool availability state.
    pub mcp_tools: HarnessMcpToolReport,
    /// Settings checks for harness-specific configuration files.
    pub settings: Vec<HarnessSettingsCheck>,
    /// Structured lifecycle compliance state.
    pub lifecycle: HarnessLifecycleReport,
    /// Soft warnings for incomplete lifecycle integration.
    pub warnings: Vec<String>,
    /// True when every required adapter, MCP tool, and settings entry is present.
    pub ready: bool,
}

/// Verification result for one settings entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessSettingsCheck {
    /// Stable settings entry name.
    pub name: String,
    /// Entry kind, for example permission or hook.
    pub kind: String,
    /// Whether this entry is required for first-class harness integration.
    pub required: bool,
    /// Settings files where the entry was found.
    pub locations: Vec<String>,
    /// Explanation for humans and agents.
    pub message: String,
}

/// Rendered adapter payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessRenderedAdapter {
    /// Stable adapter name.
    pub name: String,
    /// Adapter kind.
    pub kind: HarnessAdapterKind,
    /// Relative install path.
    pub relative_path: String,
    /// Generated content.
    pub contents: String,
}

/// File action planned or performed by harness install.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessInstallFile {
    /// Stable adapter name.
    pub name: String,
    /// Absolute path.
    pub path: String,
    /// Whether a write happened.
    pub written: bool,
    /// Explanation.
    pub message: String,
}

/// Harness install report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessInstallReport {
    /// Harness installed.
    pub harness: HarnessKind,
    /// Lifecycle enforcement profile used for generated files.
    pub enforcement_profile: HarnessEnforcementProfile,
    /// Root used for installation.
    pub root: String,
    /// True when no writes were performed.
    pub dry_run: bool,
    /// Files planned.
    pub planned: Vec<HarnessInstallFile>,
    /// Files written.
    pub written: Vec<HarnessInstallFile>,
    /// Files skipped.
    pub skipped: Vec<HarnessInstallFile>,
    /// Warnings.
    pub warnings: Vec<String>,
}
