//! Brain harness telemetry service.

use crate::error::{IndexError, IndexResult};
use engram_core::id::Id;
use engram_core::telemetry::{AgentFeedback, BrainHarnessTrace, IntentTelemetryStats};
use engram_store::{Db, TelemetryRepo};
use std::collections::HashMap;

/// Service for retrieval traces and agent feedback.
#[derive(Clone)]
pub struct TelemetryService {
    repo: TelemetryRepo,
}

impl TelemetryService {
    /// Create a new telemetry service.
    pub fn new(db: Db) -> Self {
        Self {
            repo: TelemetryRepo::new(db),
        }
    }

    /// Initialize telemetry schema.
    pub async fn init_schema(&self) -> IndexResult<()> {
        self.repo.init_schema().await?;
        Ok(())
    }

    /// Record a brain-harness operation trace.
    pub async fn record_trace(&self, trace: BrainHarnessTrace) -> IndexResult<BrainHarnessTrace> {
        validate_trace(&trace)?;
        self.repo.save_trace(&trace).await?;
        Ok(trace)
    }

    /// Get a trace by ID.
    pub async fn get_trace(&self, id: &Id) -> IndexResult<Option<BrainHarnessTrace>> {
        Ok(self.repo.get_trace(id).await?)
    }

    /// List traces.
    pub async fn list_traces(&self, limit: Option<usize>) -> IndexResult<Vec<BrainHarnessTrace>> {
        Ok(self.repo.list_traces(limit).await?)
    }

    /// Submit agent feedback for a trace.
    pub async fn submit_feedback(&self, feedback: AgentFeedback) -> IndexResult<AgentFeedback> {
        validate_feedback(&feedback)?;
        if self.repo.get_trace(&feedback.trace_id).await?.is_none() {
            return Err(IndexError::NotFound(format!(
                "trace not found: {}",
                feedback.trace_id
            )));
        }

        self.repo.save_feedback(&feedback).await?;
        Ok(feedback)
    }

    /// Get feedback by ID.
    pub async fn get_feedback(&self, id: &Id) -> IndexResult<Option<AgentFeedback>> {
        Ok(self.repo.get_feedback(id).await?)
    }

    /// List feedback for a trace.
    pub async fn list_feedback_for_trace(&self, trace_id: &Id) -> IndexResult<Vec<AgentFeedback>> {
        Ok(self.repo.list_feedback_for_trace(trace_id).await?)
    }

    /// List recent feedback.
    pub async fn list_feedback(&self, limit: Option<usize>) -> IndexResult<Vec<AgentFeedback>> {
        Ok(self.repo.list_feedback(limit).await?)
    }

    /// Aggregate traces and feedback by intent.
    pub async fn stats_by_intent(&self) -> IndexResult<Vec<IntentTelemetryStats>> {
        let traces = self.repo.list_traces(Some(10_000)).await?;
        let feedback = self.repo.list_feedback(Some(10_000)).await?;

        let trace_intents = traces
            .iter()
            .map(|trace| (trace.id, intent_key(trace)))
            .collect::<HashMap<_, _>>();

        let mut groups = HashMap::<String, IntentAggregate>::new();
        for trace in &traces {
            let key = intent_key(trace);
            let group = groups
                .entry(key.clone())
                .or_insert_with(|| IntentAggregate {
                    stats: IntentTelemetryStats {
                        intent: key,
                        ..Default::default()
                    },
                    latency_sum: 0,
                    latency_count: 0,
                    usefulness_sum: 0,
                    usefulness_count: 0,
                    correctness_sum: 0,
                    correctness_count: 0,
                    noise_sum: 0,
                    noise_count: 0,
                });
            group.stats.trace_count += 1;
            if let Some(latency) = trace.latency_ms {
                group.latency_sum += latency;
                group.latency_count += 1;
            }
        }

        for item in &feedback {
            let key = trace_intents
                .get(&item.trace_id)
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());
            let group = groups
                .entry(key.clone())
                .or_insert_with(|| IntentAggregate {
                    stats: IntentTelemetryStats {
                        intent: key,
                        ..Default::default()
                    },
                    latency_sum: 0,
                    latency_count: 0,
                    usefulness_sum: 0,
                    usefulness_count: 0,
                    correctness_sum: 0,
                    correctness_count: 0,
                    noise_sum: 0,
                    noise_count: 0,
                });

            group.stats.feedback_count += 1;
            group.stats.used_memory_count += item.used_memory_ids.len();
            group.stats.rejected_memory_count += item.rejected_memory_ids.len();
            if item
                .missing_context
                .as_deref()
                .is_some_and(|value| !value.trim().is_empty())
            {
                group.stats.missing_context_count += 1;
            }
            add_score(
                item.usefulness_score,
                &mut group.usefulness_sum,
                &mut group.usefulness_count,
            );
            add_score(
                item.correctness_score,
                &mut group.correctness_sum,
                &mut group.correctness_count,
            );
            add_score(
                item.noise_score,
                &mut group.noise_sum,
                &mut group.noise_count,
            );
        }

        let mut stats = groups
            .into_values()
            .map(IntentAggregate::into_stats)
            .collect::<Vec<_>>();
        stats.sort_by(|left, right| {
            right
                .trace_count
                .cmp(&left.trace_count)
                .then_with(|| left.intent.cmp(&right.intent))
        });
        Ok(stats)
    }
}

#[derive(Debug)]
struct IntentAggregate {
    stats: IntentTelemetryStats,
    latency_sum: u64,
    latency_count: usize,
    usefulness_sum: u32,
    usefulness_count: usize,
    correctness_sum: u32,
    correctness_count: usize,
    noise_sum: u32,
    noise_count: usize,
}

impl IntentAggregate {
    fn into_stats(mut self) -> IntentTelemetryStats {
        self.stats.avg_latency_ms = average_u64(self.latency_sum, self.latency_count);
        self.stats.avg_usefulness_score = average_u32(self.usefulness_sum, self.usefulness_count);
        self.stats.avg_correctness_score =
            average_u32(self.correctness_sum, self.correctness_count);
        self.stats.avg_noise_score = average_u32(self.noise_sum, self.noise_count);
        self.stats
    }
}

fn validate_trace(trace: &BrainHarnessTrace) -> IndexResult<()> {
    if trace.returned_memory_ids.is_empty()
        && trace.returned_result_ids.is_empty()
        && trace.query.as_deref().unwrap_or_default().trim().is_empty()
    {
        return Err(IndexError::Parse(
            "trace must include query, returned_memory_ids, or returned_result_ids".to_string(),
        ));
    }
    Ok(())
}

fn validate_feedback(feedback: &AgentFeedback) -> IndexResult<()> {
    validate_score("usefulness_score", feedback.usefulness_score)?;
    validate_score("correctness_score", feedback.correctness_score)?;
    validate_score("noise_score", feedback.noise_score)?;

    let has_signal = !feedback.used_memory_ids.is_empty()
        || !feedback.rejected_memory_ids.is_empty()
        || !feedback.used_result_ids.is_empty()
        || !feedback.rejected_result_ids.is_empty()
        || !feedback.stale_memory_ids.is_empty()
        || !feedback.wrong_scope_memory_ids.is_empty()
        || feedback
            .missing_context
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
        || feedback
            .suggested_memory_changes
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
        || feedback
            .note
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty());

    if !has_signal {
        return Err(IndexError::Parse(
            "feedback must include at least one concrete signal".to_string(),
        ));
    }

    Ok(())
}

fn validate_score(name: &str, score: Option<u8>) -> IndexResult<()> {
    if let Some(score) = score {
        if !(1..=5).contains(&score) {
            return Err(IndexError::Parse(format!("{name} must be between 1 and 5")));
        }
    }
    Ok(())
}

fn intent_key(trace: &BrainHarnessTrace) -> String {
    trace
        .intent
        .as_ref()
        .map(std::string::ToString::to_string)
        .unwrap_or_else(|| "unspecified".to_string())
}

fn add_score(score: Option<u8>, sum: &mut u32, count: &mut usize) {
    if let Some(score) = score {
        *sum += u32::from(score);
        *count += 1;
    }
}

fn average_u64(sum: u64, count: usize) -> Option<f32> {
    if count == 0 {
        None
    } else {
        Some(sum as f32 / count as f32)
    }
}

fn average_u32(sum: u32, count: usize) -> Option<f32> {
    if count == 0 {
        None
    } else {
        Some(sum as f32 / count as f32)
    }
}
