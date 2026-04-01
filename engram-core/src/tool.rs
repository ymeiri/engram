//! Tool intelligence types (Layer 4).
//!
//! Tracks tool usage patterns, outcomes, and provides recommendations.

use crate::id::Id;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use time::OffsetDateTime;

/// Outcome of a tool usage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolOutcome {
    /// Tool succeeded.
    Success,
    /// Tool partially succeeded.
    Partial,
    /// Tool failed.
    Failed,
    /// User switched to a different tool.
    Switched,
}

impl fmt::Display for ToolOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ToolOutcome::Success => write!(f, "success"),
            ToolOutcome::Partial => write!(f, "partial"),
            ToolOutcome::Failed => write!(f, "failed"),
            ToolOutcome::Switched => write!(f, "switched"),
        }
    }
}

impl FromStr for ToolOutcome {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "success" => Ok(ToolOutcome::Success),
            "partial" => Ok(ToolOutcome::Partial),
            "failed" => Ok(ToolOutcome::Failed),
            "switched" => Ok(ToolOutcome::Switched),
            _ => Err(format!("Unknown tool outcome: {}", s)),
        }
    }
}

/// Aggregate statistics for a tool.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolStats {
    /// Total number of usages.
    pub total_usages: usize,
    /// Number of successful usages.
    pub success_count: usize,
    /// Number of failed usages.
    pub failure_count: usize,
    /// Success rate (0.0 - 1.0).
    pub success_rate: f32,
    /// Number of learned preferences involving this tool.
    pub preferences_count: usize,
}

/// A record of tool usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsage {
    /// Unique identifier.
    pub id: Id,

    /// Tool entity ID.
    pub tool_id: Id,

    /// Session ID (optional).
    pub session_id: Option<Id>,

    /// Context of usage (what was the user trying to do?).
    pub context: String,

    /// Outcome of the usage.
    pub outcome: ToolOutcome,

    /// If switched, which tool was used instead?
    pub switched_to: Option<Id>,

    /// Timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
}

impl ToolUsage {
    /// Create a new tool usage record.
    #[must_use]
    pub fn new(tool_id: Id, context: impl Into<String>, outcome: ToolOutcome) -> Self {
        Self {
            id: Id::new(),
            tool_id,
            session_id: None,
            context: context.into(),
            outcome,
            switched_to: None,
            timestamp: OffsetDateTime::now_utc(),
        }
    }

    /// Set the session.
    #[must_use]
    pub fn with_session(mut self, session_id: Id) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Set the tool that was switched to.
    #[must_use]
    pub fn with_switched_to(mut self, tool_id: Id) -> Self {
        self.switched_to = Some(tool_id);
        self
    }
}

/// A learned tool preference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPreference {
    /// Context pattern (e.g., "building go service").
    pub context_pattern: String,

    /// Preferred tool entity ID.
    pub preferred_tool_id: Id,

    /// Confidence score (0.0 - 1.0).
    pub confidence: f32,

    /// Number of samples this preference is based on.
    pub sample_count: u32,

    /// Last update timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl ToolPreference {
    /// Create a new tool preference.
    #[must_use]
    pub fn new(
        context_pattern: impl Into<String>,
        preferred_tool_id: Id,
        confidence: f32,
        sample_count: u32,
    ) -> Self {
        Self {
            context_pattern: context_pattern.into(),
            preferred_tool_id,
            confidence,
            sample_count,
            updated_at: OffsetDateTime::now_utc(),
        }
    }
}

/// A workflow (sequence of tools/steps).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// Unique identifier.
    pub id: Id,

    /// Workflow name.
    pub name: String,

    /// Description.
    pub description: Option<String>,

    /// Steps in the workflow.
    pub steps: Vec<WorkflowStep>,

    /// Contexts where this workflow applies.
    #[serde(default)]
    pub applicable_contexts: Vec<String>,

    /// Creation timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,

    /// Last update timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl Workflow {
    /// Create a new workflow.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: Id::new(),
            name: name.into(),
            description: None,
            steps: Vec::new(),
            applicable_contexts: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Set the description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a step.
    #[must_use]
    pub fn with_step(mut self, step: WorkflowStep) -> Self {
        self.steps.push(step);
        self
    }
}

/// A step in a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// Step number (1-indexed).
    pub order: u32,

    /// Description of what to do.
    pub description: String,

    /// Tool to use (optional).
    pub tool_id: Option<Id>,

    /// Expected output or result.
    pub expected_output: Option<String>,
}

impl WorkflowStep {
    /// Create a new workflow step.
    #[must_use]
    pub fn new(order: u32, description: impl Into<String>) -> Self {
        Self {
            order,
            description: description.into(),
            tool_id: None,
            expected_output: None,
        }
    }
}

/// A tool recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRecommendation {
    /// Recommended tool ID.
    pub tool_id: Id,

    /// Tool name.
    pub tool_name: String,

    /// Confidence score (0.0 - 1.0).
    pub confidence: f32,

    /// Reason for the recommendation.
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_usage_creation() {
        let tool_id = Id::new();
        let usage = ToolUsage::new(tool_id, "building go service", ToolOutcome::Success);

        assert_eq!(usage.tool_id, tool_id);
        assert_eq!(usage.outcome, ToolOutcome::Success);
    }

    #[test]
    fn test_workflow_creation() {
        let workflow = Workflow::new("jira-creation")
            .with_description("Create a Jira ticket")
            .with_step(WorkflowStep::new(1, "Gather requirements"))
            .with_step(WorkflowStep::new(2, "Create ticket"));

        assert_eq!(workflow.name, "jira-creation");
        assert_eq!(workflow.steps.len(), 2);
    }

    #[test]
    fn test_tool_outcome_display() {
        assert_eq!(ToolOutcome::Success.to_string(), "success");
        assert_eq!(ToolOutcome::Partial.to_string(), "partial");
        assert_eq!(ToolOutcome::Failed.to_string(), "failed");
        assert_eq!(ToolOutcome::Switched.to_string(), "switched");
    }

    #[test]
    fn test_tool_outcome_from_str() {
        assert_eq!(
            ToolOutcome::from_str("success").unwrap(),
            ToolOutcome::Success
        );
        assert_eq!(
            ToolOutcome::from_str("SUCCESS").unwrap(),
            ToolOutcome::Success
        );
        assert_eq!(
            ToolOutcome::from_str("partial").unwrap(),
            ToolOutcome::Partial
        );
        assert_eq!(
            ToolOutcome::from_str("failed").unwrap(),
            ToolOutcome::Failed
        );
        assert_eq!(
            ToolOutcome::from_str("switched").unwrap(),
            ToolOutcome::Switched
        );
        assert!(ToolOutcome::from_str("unknown").is_err());
    }

    #[test]
    fn test_tool_stats_default() {
        let stats = ToolStats::default();
        assert_eq!(stats.total_usages, 0);
        assert_eq!(stats.success_count, 0);
        assert_eq!(stats.failure_count, 0);
        assert_eq!(stats.success_rate, 0.0);
        assert_eq!(stats.preferences_count, 0);
    }
}
