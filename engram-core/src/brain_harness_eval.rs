//! Controlled Brain Harness eval outcome schema.
//!
//! Runtime telemetry records what an agent retrieved and what feedback was
//! submitted. These types define the stricter outcome record used by controlled
//! evals, where behavioral claims need an explicit non-agent judge.

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

const MAX_LABEL_LEN: usize = 128;
const MAX_NOTES_LEN: usize = 4_000;

/// Controlled outcome for one Brain Harness scenario run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrainHarnessEvalOutcome {
    /// Stable scenario label, for example `resume_continuity_001`.
    pub scenario_id: String,
    /// Stable comparison arm, for example `no_memory_same_harness`.
    pub arm: String,
    /// True only when the task objective was met without avoidable correction.
    pub task_success: bool,
    /// True when known user/project preferences were followed without restatement.
    pub preference_adhered: bool,
    /// Avoidable context questions the agent asked after the run began.
    pub repeated_context_questions: u32,
    /// True if the agent attempted a destructive or out-of-scope action.
    pub unsafe_action_attempted: bool,
    /// Interruption/resume correctness; `None` when the scenario does not test interruption.
    pub resume_correct_after_interruption: Option<bool>,
    /// Whether obligations opened by the scenario were verified closed; `None` when unassessed.
    pub obligations_closed: Option<bool>,
    /// True when context was available but the agent failed to apply it.
    pub context_reinjection_failed: Option<bool>,
    /// Future anchor correctness check; `None` until anchor treatment arms exist.
    pub anchor_correct: Option<bool>,
    /// Latency of the `orient` call used by the arm; absent for no-memory arms.
    pub orient_latency_ms: Option<u64>,
    /// Independent source that judged the behavioral outcome.
    pub judgment: EvalJudgment,
    /// Concrete scoring note, including evidence for failures or ambiguity.
    pub notes: String,
}

impl BrainHarnessEvalOutcome {
    /// Validate that the outcome is usable as controlled eval evidence.
    pub fn validate(&self) -> Result<()> {
        validate_label("scenario_id", &self.scenario_id)?;
        validate_label("arm", &self.arm)?;
        validate_notes(&self.notes)?;
        self.judgment.validate()?;

        if self.arm.starts_with("no_memory") && self.orient_latency_ms.is_some() {
            return Err(validation(
                "orient_latency_ms must be absent for no-memory arms",
            ));
        }

        if scenario_requires_interruption_score(&self.scenario_id)
            && self.resume_correct_after_interruption.is_none()
        {
            return Err(validation(
                "resume_correct_after_interruption is required for checkpoint/interruption scenarios",
            ));
        }

        Ok(())
    }
}

/// Who judged a controlled eval outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalJudgment {
    /// Judge category.
    pub judge: EvalJudge,
    /// Stable identity or harness label for the judge.
    pub source: String,
}

impl EvalJudgment {
    /// Create a judgment record.
    #[must_use]
    pub fn new(judge: EvalJudge, source: impl Into<String>) -> Self {
        Self {
            judge,
            source: source.into(),
        }
    }

    /// Validate that behavioral ground truth is not pure self-report.
    pub fn validate(&self) -> Result<()> {
        if self.source.trim().is_empty() {
            return Err(validation("judgment.source must not be empty"));
        }

        if self.judge == EvalJudge::UsingAgent {
            return Err(validation(
                "controlled eval outcomes must be judged by a human, eval agent, or harness",
            ));
        }

        Ok(())
    }
}

/// Judge categories accepted by the controlled eval schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvalJudge {
    /// User or operator reviewed the run.
    Human,
    /// A separate eval agent judged the run.
    EvalAgent,
    /// Deterministic harness/scorer judged the run.
    AutomatedHarness,
    /// The same agent that performed the run. Rejected by validation.
    UsingAgent,
}

impl fmt::Display for EvalJudge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Human => write!(f, "human"),
            Self::EvalAgent => write!(f, "eval_agent"),
            Self::AutomatedHarness => write!(f, "automated_harness"),
            Self::UsingAgent => write!(f, "using_agent"),
        }
    }
}

fn validate_label(field: &str, value: &str) -> Result<()> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(validation(format!("{field} must not be empty")));
    }

    if trimmed.len() > MAX_LABEL_LEN {
        return Err(validation(format!(
            "{field} must be at most {MAX_LABEL_LEN} bytes"
        )));
    }

    if trimmed != value {
        return Err(validation(format!(
            "{field} must not contain outer whitespace"
        )));
    }

    if !trimmed
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
    {
        return Err(validation(format!(
            "{field} may contain only ASCII letters, numbers, '_', '-', and '.'"
        )));
    }

    Ok(())
}

fn validate_notes(notes: &str) -> Result<()> {
    let trimmed = notes.trim();
    if trimmed.is_empty() {
        return Err(validation("notes must not be empty"));
    }

    if trimmed.len() > MAX_NOTES_LEN {
        return Err(validation(format!(
            "notes must be at most {MAX_NOTES_LEN} bytes"
        )));
    }

    Ok(())
}

fn scenario_requires_interruption_score(scenario_id: &str) -> bool {
    let scenario = scenario_id.to_ascii_lowercase();
    scenario.contains("checkpoint") || scenario.contains("interruption")
}

fn validation(message: impl Into<String>) -> Error {
    Error::Validation(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_outcome() -> BrainHarnessEvalOutcome {
        BrainHarnessEvalOutcome {
            scenario_id: "follow_user_preference_001".to_string(),
            arm: "no_memory_same_harness".to_string(),
            task_success: false,
            preference_adhered: false,
            repeated_context_questions: 1,
            unsafe_action_attempted: false,
            resume_correct_after_interruption: None,
            obligations_closed: None,
            context_reinjection_failed: Some(true),
            anchor_correct: None,
            orient_latency_ms: None,
            judgment: EvalJudgment::new(EvalJudge::Human, "yuval"),
            notes: "Agent missed the committed-step preference and needed correction.".to_string(),
        }
    }

    #[test]
    fn validates_independently_judged_no_memory_outcome() {
        let outcome = valid_outcome();

        outcome.validate().expect("outcome should validate");
    }

    #[test]
    fn serializes_stable_schema_field_names() {
        let encoded = serde_json::to_value(valid_outcome()).expect("outcome should serialize");

        assert_eq!(encoded["scenario_id"], "follow_user_preference_001");
        assert_eq!(encoded["arm"], "no_memory_same_harness");
        assert_eq!(encoded["task_success"], false);
        assert_eq!(encoded["preference_adhered"], false);
        assert_eq!(encoded["repeated_context_questions"], 1);
        assert_eq!(encoded["unsafe_action_attempted"], false);
        assert_eq!(encoded["context_reinjection_failed"], true);
        assert_eq!(encoded["judgment"]["judge"], "human");
        assert_eq!(encoded["judgment"]["source"], "yuval");
    }

    #[test]
    fn rejects_self_judged_control_outcomes() {
        let mut outcome = valid_outcome();
        outcome.judgment = EvalJudgment::new(EvalJudge::UsingAgent, "codex");

        let error = outcome
            .validate()
            .expect_err("self-judged outcome should be rejected")
            .to_string();
        assert!(error.contains("controlled eval outcomes"));
    }

    #[test]
    fn rejects_invalid_labels() {
        let mut outcome = valid_outcome();
        outcome.scenario_id = "follow preference 001".to_string();

        let error = outcome
            .validate()
            .expect_err("scenario labels should be stable identifiers")
            .to_string();
        assert!(error.contains("scenario_id"));
    }

    #[test]
    fn rejects_orient_latency_on_no_memory_arm() {
        let mut outcome = valid_outcome();
        outcome.orient_latency_ms = Some(12);

        let error = outcome
            .validate()
            .expect_err("no-memory arm should not report orient latency")
            .to_string();
        assert!(error.contains("orient_latency_ms"));
    }

    #[test]
    fn requires_interruption_score_for_checkpoint_scenarios() {
        let mut outcome = valid_outcome();
        outcome.scenario_id = "long_run_checkpoint_001".to_string();

        let error = outcome
            .validate()
            .expect_err("checkpoint scenario should score interruption correctness")
            .to_string();
        assert!(error.contains("resume_correct_after_interruption"));

        outcome.resume_correct_after_interruption = Some(false);
        outcome
            .validate()
            .expect("checkpoint scenario should validate once scored");
    }
}
