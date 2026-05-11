//! Memory health lint types.

use crate::id::Id;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Lint rule identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LintRule {
    /// Memory item has no evidence.
    MissingEvidence,
    /// Preference is due for recalibration.
    StalePreference,
    /// Multiple entity-scoped items look like duplicates.
    DuplicateEntityCandidate,
    /// Project/subproject scope appears disconnected.
    OrphanProjectSubproject,
    /// Active session is old enough to be suspicious.
    StaleActiveSession,
    /// An active item has been superseded by another item.
    SupersededItemStillActive,
    /// Generated vault page is missing marker or frontmatter.
    VaultPageMissingMarkerFrontmatter,
    /// Handoff content does not include next actions.
    HandoffMissingNextActions,
    /// Agent-native obligation is still open.
    UnresolvedAgentObligation,
}

impl std::fmt::Display for LintRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingEvidence => write!(f, "missing_evidence"),
            Self::StalePreference => write!(f, "stale_preference"),
            Self::DuplicateEntityCandidate => write!(f, "duplicate_entity_candidate"),
            Self::OrphanProjectSubproject => write!(f, "orphan_project_subproject"),
            Self::StaleActiveSession => write!(f, "stale_active_session"),
            Self::SupersededItemStillActive => write!(f, "superseded_item_still_active"),
            Self::VaultPageMissingMarkerFrontmatter => {
                write!(f, "vault_page_missing_marker_frontmatter")
            }
            Self::HandoffMissingNextActions => write!(f, "handoff_missing_next_actions"),
            Self::UnresolvedAgentObligation => write!(f, "unresolved_agent_obligation"),
        }
    }
}

/// Lint severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LintSeverity {
    /// Informational finding.
    Info,
    /// Warning finding.
    Warning,
    /// Error finding.
    Error,
}

/// Safe remediation available for a lint finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LintSafeAction {
    /// No automatic action is safe.
    None,
    /// Archive a stale/superseded item.
    ArchiveMemoryItem,
}

/// One lint finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LintFinding {
    /// Deterministic finding ID.
    pub id: String,
    /// Rule that produced this finding.
    pub rule: LintRule,
    /// Severity.
    pub severity: LintSeverity,
    /// Human-readable title.
    pub title: String,
    /// Explanation.
    pub message: String,
    /// Related memory item, if applicable.
    pub item_id: Option<Id>,
    /// Related session, if applicable.
    pub session_id: Option<Id>,
    /// Related agent obligation, if applicable.
    pub obligation_id: Option<Id>,
    /// Related file path, if applicable.
    pub path: Option<String>,
    /// Safe action available.
    pub safe_action: LintSafeAction,
    /// Creation timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

impl LintFinding {
    /// Create a lint finding.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        rule: LintRule,
        severity: LintSeverity,
        title: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            rule,
            severity,
            title: title.into(),
            message: message.into(),
            item_id: None,
            session_id: None,
            obligation_id: None,
            path: None,
            safe_action: LintSafeAction::None,
            created_at: OffsetDateTime::now_utc(),
        }
    }

    /// Attach a memory item ID.
    #[must_use]
    pub fn with_item(mut self, item_id: Id) -> Self {
        self.item_id = Some(item_id);
        self
    }

    /// Attach a session ID.
    #[must_use]
    pub fn with_session(mut self, session_id: Id) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Attach an agent obligation ID.
    #[must_use]
    pub fn with_obligation(mut self, obligation_id: Id) -> Self {
        self.obligation_id = Some(obligation_id);
        self
    }

    /// Attach a file path.
    #[must_use]
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Attach a safe remediation action.
    #[must_use]
    pub fn with_safe_action(mut self, action: LintSafeAction) -> Self {
        self.safe_action = action;
        self
    }
}

/// Lint report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LintReport {
    /// Findings.
    pub findings: Vec<LintFinding>,
    /// Number of safe actions applied.
    pub applied_safe_actions: usize,
}

impl LintReport {
    /// Create a report.
    #[must_use]
    pub fn new(findings: Vec<LintFinding>) -> Self {
        Self {
            findings,
            applied_safe_actions: 0,
        }
    }
}
