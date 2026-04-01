//! Tool repository for Layer 4: Tool Intelligence.
//!
//! Handles persistence of ToolUsage and ToolPreference.
//! Provides queries for recommendations and statistics.

use crate::error::{StoreError, StoreResult};
use crate::Db;
use engram_core::id::Id;
use engram_core::tool::{ToolOutcome, ToolPreference, ToolStats, ToolUsage};
use serde::Deserialize;
use std::str::FromStr;
use tracing::{debug, info};

/// SurrealDB datetime representation (handles both string and native formats).
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum SurrealDateTime {
    /// ISO 8601 string format
    String(String),
    /// Native SurrealDB datetime format (array of integers)
    Native(serde_json::Value),
}

impl SurrealDateTime {
    fn to_offset_datetime(&self) -> StoreResult<time::OffsetDateTime> {
        match self {
            SurrealDateTime::String(s) => {
                time::OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339)
                    .map_err(|e| StoreError::Deserialization(format!("Invalid datetime: {}", e)))
            }
            SurrealDateTime::Native(v) => {
                // SurrealDB native format: [year, month, day, hour, min, sec, nano, offset_h, offset_m]
                if let Some(arr) = v.as_array() {
                    if arr.len() >= 6 {
                        let year = arr[0].as_i64().unwrap_or(2000) as i32;
                        let month = arr[1].as_i64().unwrap_or(1) as u8;
                        let day = arr[2].as_i64().unwrap_or(1) as u8;
                        let hour = arr[3].as_i64().unwrap_or(0) as u8;
                        let min = arr[4].as_i64().unwrap_or(0) as u8;
                        let sec = arr[5].as_i64().unwrap_or(0) as u8;

                        let date = time::Date::from_calendar_date(
                            year,
                            time::Month::try_from(month).unwrap_or(time::Month::January),
                            day,
                        )
                        .unwrap_or(
                            time::Date::from_calendar_date(2000, time::Month::January, 1).unwrap(),
                        );

                        let time_val =
                            time::Time::from_hms(hour, min, sec).unwrap_or(time::Time::MIDNIGHT);

                        return Ok(time::OffsetDateTime::new_utc(date, time_val));
                    }
                }
                Ok(time::OffsetDateTime::now_utc())
            }
        }
    }
}

/// Tool usage record from SurrealDB.
#[derive(Debug, Deserialize)]
struct ToolUsageRecord {
    tool_id: String,
    session_id: Option<String>,
    context: String,
    outcome: String,
    switched_to: Option<String>,
    timestamp: SurrealDateTime,
}

/// Tool usage record with ID for list queries.
#[derive(Debug, Deserialize)]
struct ToolUsageRecordWithId {
    id: String,
    tool_id: String,
    session_id: Option<String>,
    context: String,
    outcome: String,
    switched_to: Option<String>,
    timestamp: SurrealDateTime,
}

/// Tool preference record from SurrealDB.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ToolPreferenceRecord {
    context_pattern: String,
    preferred_tool_id: String,
    confidence: f32,
    sample_count: u32,
    updated_at: SurrealDateTime,
}

/// Tool preference record with ID.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ToolPreferenceRecordWithId {
    id: String,
    context_pattern: String,
    preferred_tool_id: String,
    confidence: f32,
    sample_count: u32,
    updated_at: SurrealDateTime,
}

/// Count result for stats queries.
#[derive(Debug, Deserialize)]
struct CountResult {
    count: u64,
}

/// Table names for tool storage.
const TABLE_USAGE: &str = "tool_usage";
const TABLE_PREFERENCE: &str = "tool_preference";

/// Repository for tool intelligence operations.
#[derive(Clone)]
pub struct ToolRepo {
    db: Db,
}

impl ToolRepo {
    /// Create a new tool repository.
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Initialize the tool schema.
    pub async fn init_schema(&self) -> StoreResult<()> {
        info!("Initializing tool schema (Layer 4)");

        // Tool usage table
        self.db
            .query(format!(
                r#"
                DEFINE TABLE IF NOT EXISTS {TABLE_USAGE} SCHEMALESS;
                DEFINE INDEX IF NOT EXISTS idx_usage_tool ON {TABLE_USAGE} FIELDS tool_id;
                DEFINE INDEX IF NOT EXISTS idx_usage_session ON {TABLE_USAGE} FIELDS session_id;
                DEFINE INDEX IF NOT EXISTS idx_usage_outcome ON {TABLE_USAGE} FIELDS outcome;
                DEFINE INDEX IF NOT EXISTS idx_usage_timestamp ON {TABLE_USAGE} FIELDS timestamp;
                "#
            ))
            .await?;

        // Tool preference table
        self.db
            .query(format!(
                r#"
                DEFINE TABLE IF NOT EXISTS {TABLE_PREFERENCE} SCHEMALESS;
                DEFINE INDEX IF NOT EXISTS idx_pref_pattern ON {TABLE_PREFERENCE} FIELDS context_pattern;
                DEFINE INDEX IF NOT EXISTS idx_pref_tool ON {TABLE_PREFERENCE} FIELDS preferred_tool_id;
                "#
            ))
            .await?;

        info!("Tool schema initialized");
        Ok(())
    }

    // =========================================================================
    // Tool Usage Operations
    // =========================================================================

    /// Save a tool usage record.
    pub async fn save_usage(&self, usage: &ToolUsage) -> StoreResult<()> {
        debug!("Saving tool usage: {} ({})", usage.tool_id, usage.outcome);

        self.db
            .query(
                r#"
                UPSERT type::thing("tool_usage", $id) SET
                    tool_id = $tool_id,
                    session_id = $session_id,
                    context = $context,
                    outcome = $outcome,
                    switched_to = $switched_to,
                    timestamp = $timestamp
            "#,
            )
            .bind(("id", usage.id.to_string()))
            .bind(("tool_id", usage.tool_id.to_string()))
            .bind((
                "session_id",
                usage.session_id.as_ref().map(|id| id.to_string()),
            ))
            .bind(("context", usage.context.clone()))
            .bind(("outcome", usage.outcome.to_string()))
            .bind((
                "switched_to",
                usage.switched_to.as_ref().map(|id| id.to_string()),
            ))
            .bind((
                "timestamp",
                usage
                    .timestamp
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            ))
            .await?;

        Ok(())
    }

    /// Get a tool usage by ID.
    pub async fn get_usage(&self, id: &Id) -> StoreResult<Option<ToolUsage>> {
        debug!("Getting tool usage: {}", id);

        let mut result = self
            .db
            .query(r#"SELECT * FROM type::thing("tool_usage", $id)"#)
            .bind(("id", id.to_string()))
            .await?;

        let records: Vec<ToolUsageRecord> = result.take(0)?;

        if let Some(record) = records.into_iter().next() {
            Ok(Some(self.record_to_usage(id.clone(), record)?))
        } else {
            Ok(None)
        }
    }

    /// List tool usages with optional filters.
    pub async fn list_usages(
        &self,
        outcome: Option<&ToolOutcome>,
        limit: Option<usize>,
    ) -> StoreResult<Vec<ToolUsage>> {
        debug!("Listing tool usages (outcome filter: {:?})", outcome);

        let query = match outcome {
            Some(o) => format!(
                "SELECT meta::id(id) as id, tool_id, session_id, context, outcome, switched_to, timestamp FROM {} WHERE outcome = '{}' ORDER BY timestamp DESC LIMIT {}",
                TABLE_USAGE,
                o,
                limit.unwrap_or(100)
            ),
            None => format!(
                "SELECT meta::id(id) as id, tool_id, session_id, context, outcome, switched_to, timestamp FROM {} ORDER BY timestamp DESC LIMIT {}",
                TABLE_USAGE,
                limit.unwrap_or(100)
            ),
        };

        let mut result = self.db.query(query).await?;
        let records: Vec<ToolUsageRecordWithId> = result.take(0)?;

        let mut usages = Vec::new();
        for record in records {
            let id = Id::parse(&record.id)
                .map_err(|e| StoreError::Deserialization(format!("Invalid ID: {}", e)))?;
            usages.push(self.record_to_usage(id, record.into())?);
        }

        Ok(usages)
    }

    /// Get usages for a specific tool.
    pub async fn get_usages_for_tool(&self, tool_id: &Id) -> StoreResult<Vec<ToolUsage>> {
        debug!("Getting usages for tool: {}", tool_id);

        let mut result = self.db
            .query("SELECT meta::id(id) as id, tool_id, session_id, context, outcome, switched_to, timestamp FROM tool_usage WHERE tool_id = $tool_id ORDER BY timestamp DESC")
            .bind(("tool_id", tool_id.to_string()))
            .await?;

        let records: Vec<ToolUsageRecordWithId> = result.take(0)?;

        let mut usages = Vec::new();
        for record in records {
            let id = Id::parse(&record.id)
                .map_err(|e| StoreError::Deserialization(format!("Invalid ID: {}", e)))?;
            usages.push(self.record_to_usage(id, record.into())?);
        }

        Ok(usages)
    }

    /// Get usages for a specific session.
    pub async fn get_usages_for_session(&self, session_id: &Id) -> StoreResult<Vec<ToolUsage>> {
        debug!("Getting usages for session: {}", session_id);

        let mut result = self.db
            .query("SELECT meta::id(id) as id, tool_id, session_id, context, outcome, switched_to, timestamp FROM tool_usage WHERE session_id = $session_id ORDER BY timestamp DESC")
            .bind(("session_id", session_id.to_string()))
            .await?;

        let records: Vec<ToolUsageRecordWithId> = result.take(0)?;

        let mut usages = Vec::new();
        for record in records {
            let id = Id::parse(&record.id)
                .map_err(|e| StoreError::Deserialization(format!("Invalid ID: {}", e)))?;
            usages.push(self.record_to_usage(id, record.into())?);
        }

        Ok(usages)
    }

    /// Search usages by context.
    pub async fn search_usages(
        &self,
        query: &str,
        limit: Option<usize>,
    ) -> StoreResult<Vec<ToolUsage>> {
        debug!("Searching tool usages: {}", query);

        let mut result = self.db
            .query(format!(
                "SELECT meta::id(id) as id, tool_id, session_id, context, outcome, switched_to, timestamp FROM tool_usage WHERE string::lowercase(context) CONTAINS $query ORDER BY timestamp DESC LIMIT {}",
                limit.unwrap_or(50)
            ))
            .bind(("query", query.to_lowercase()))
            .await?;

        let records: Vec<ToolUsageRecordWithId> = result.take(0)?;

        let mut usages = Vec::new();
        for record in records {
            let id = Id::parse(&record.id)
                .map_err(|e| StoreError::Deserialization(format!("Invalid ID: {}", e)))?;
            usages.push(self.record_to_usage(id, record.into())?);
        }

        Ok(usages)
    }

    // =========================================================================
    // Tool Preference Operations
    // =========================================================================

    /// Save a tool preference.
    pub async fn save_preference(&self, pref: &ToolPreference) -> StoreResult<()> {
        debug!(
            "Saving tool preference: {} -> {}",
            pref.context_pattern, pref.preferred_tool_id
        );

        // Use context_pattern as the key to allow upsert
        let pref_id = format!("pref_{}", pref.context_pattern.replace(' ', "_"));

        self.db
            .query(
                r#"
                UPSERT type::thing("tool_preference", $id) SET
                    context_pattern = $context_pattern,
                    preferred_tool_id = $preferred_tool_id,
                    confidence = $confidence,
                    sample_count = $sample_count,
                    updated_at = $updated_at
            "#,
            )
            .bind(("id", pref_id))
            .bind(("context_pattern", pref.context_pattern.clone()))
            .bind(("preferred_tool_id", pref.preferred_tool_id.to_string()))
            .bind(("confidence", pref.confidence))
            .bind(("sample_count", pref.sample_count))
            .bind((
                "updated_at",
                pref.updated_at
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap(),
            ))
            .await?;

        Ok(())
    }

    /// Get preferences matching a context.
    pub async fn get_preferences_for_context(
        &self,
        context: &str,
    ) -> StoreResult<Vec<ToolPreference>> {
        debug!("Getting preferences for context: {}", context);

        // Find preferences where the context contains the pattern or pattern contains context
        let mut result = self.db
            .query(r#"
                SELECT meta::id(id) as id, context_pattern, preferred_tool_id, confidence, sample_count, updated_at FROM tool_preference
                WHERE string::lowercase($context) CONTAINS string::lowercase(context_pattern)
                   OR string::lowercase(context_pattern) CONTAINS string::lowercase($context)
                ORDER BY confidence DESC, sample_count DESC
            "#)
            .bind(("context", context.to_string()))
            .await?;

        let records: Vec<ToolPreferenceRecordWithId> = result.take(0)?;

        let mut preferences = Vec::new();
        for record in records {
            preferences.push(self.record_to_preference(record)?);
        }

        Ok(preferences)
    }

    /// Get all preferences.
    pub async fn list_preferences(&self) -> StoreResult<Vec<ToolPreference>> {
        debug!("Listing all tool preferences");

        let mut result = self.db
            .query("SELECT meta::id(id) as id, context_pattern, preferred_tool_id, confidence, sample_count, updated_at FROM tool_preference ORDER BY confidence DESC")
            .await?;

        let records: Vec<ToolPreferenceRecordWithId> = result.take(0)?;

        let mut preferences = Vec::new();
        for record in records {
            preferences.push(self.record_to_preference(record)?);
        }

        Ok(preferences)
    }

    // =========================================================================
    // Statistics
    // =========================================================================

    /// Calculate success rate for a tool.
    pub async fn calculate_success_rate(&self, tool_id: &Id) -> StoreResult<f32> {
        debug!("Calculating success rate for tool: {}", tool_id);

        let mut result = self
            .db
            .query(
                r#"
                SELECT
                    count() as total,
                    count(outcome = 'success') as success
                FROM tool_usage
                WHERE tool_id = $tool_id
                GROUP ALL
            "#,
            )
            .bind(("tool_id", tool_id.to_string()))
            .await?;

        #[derive(Debug, Deserialize)]
        struct RateResult {
            total: u64,
            success: u64,
        }

        let rate: Option<RateResult> = result.take(0)?;

        Ok(rate
            .map(|r| {
                if r.total > 0 {
                    r.success as f32 / r.total as f32
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0))
    }

    /// Get tool statistics for a specific tool.
    pub async fn tool_stats(&self, tool_id: &Id) -> StoreResult<ToolStats> {
        debug!("Getting stats for tool: {}", tool_id);

        let mut result = self.db
            .query(r#"
                SELECT
                    count() as total,
                    count(outcome = 'success') as success,
                    count(outcome = 'failed') as failed
                FROM tool_usage
                WHERE tool_id = $tool_id
                GROUP ALL;

                SELECT count() as count FROM tool_preference WHERE preferred_tool_id = $tool_id GROUP ALL;
            "#)
            .bind(("tool_id", tool_id.to_string()))
            .await?;

        #[derive(Debug, Deserialize)]
        struct UsageStats {
            total: u64,
            success: u64,
            failed: u64,
        }

        let usage_stats: Option<UsageStats> = result.take(0)?;
        let pref_count: Option<CountResult> = result.take(1)?;

        let (total, success, failed) = usage_stats
            .map(|s| (s.total as usize, s.success as usize, s.failed as usize))
            .unwrap_or((0, 0, 0));

        let success_rate = if total > 0 {
            success as f32 / total as f32
        } else {
            0.0
        };

        Ok(ToolStats {
            total_usages: total,
            success_count: success,
            failure_count: failed,
            success_rate,
            preferences_count: pref_count.map(|c| c.count as usize).unwrap_or(0),
        })
    }

    /// Get overall statistics.
    pub async fn stats(&self) -> StoreResult<ToolIntelStats> {
        let mut result = self
            .db
            .query(format!(
                r#"
                SELECT count() as count FROM {TABLE_USAGE} GROUP ALL;
                SELECT count() as count FROM {TABLE_PREFERENCE} GROUP ALL;
                "#
            ))
            .await?;

        let usage_count: Option<CountResult> = result.take(0)?;
        let preference_count: Option<CountResult> = result.take(1)?;

        Ok(ToolIntelStats {
            usage_count: usage_count.map(|c| c.count).unwrap_or(0),
            preference_count: preference_count.map(|c| c.count).unwrap_or(0),
        })
    }

    // =========================================================================
    // Helper Methods
    // =========================================================================

    #[allow(dead_code)]
    fn parse_record_id(&self, record_id: &str) -> StoreResult<Id> {
        // Record ID format is "table:id"
        if let Some(id_str) = record_id.split(':').nth(1) {
            Id::parse(id_str).map_err(|e| StoreError::Deserialization(format!("Invalid ID: {}", e)))
        } else {
            Err(StoreError::Deserialization(format!(
                "Invalid record ID format: {}",
                record_id
            )))
        }
    }

    fn record_to_usage(&self, id: Id, record: ToolUsageRecord) -> StoreResult<ToolUsage> {
        let tool_id = Id::parse(&record.tool_id)
            .map_err(|e| StoreError::Deserialization(format!("Invalid tool ID: {}", e)))?;

        let session_id = record
            .session_id
            .map(|s| Id::parse(&s))
            .transpose()
            .map_err(|e| StoreError::Deserialization(format!("Invalid session ID: {}", e)))?;

        let outcome =
            ToolOutcome::from_str(&record.outcome).map_err(|e| StoreError::Deserialization(e))?;

        let switched_to = record
            .switched_to
            .map(|s| Id::parse(&s))
            .transpose()
            .map_err(|e| StoreError::Deserialization(format!("Invalid switched_to ID: {}", e)))?;

        Ok(ToolUsage {
            id,
            tool_id,
            session_id,
            context: record.context,
            outcome,
            switched_to,
            timestamp: record.timestamp.to_offset_datetime()?,
        })
    }

    fn record_to_preference(
        &self,
        record: ToolPreferenceRecordWithId,
    ) -> StoreResult<ToolPreference> {
        let preferred_tool_id = Id::parse(&record.preferred_tool_id)
            .map_err(|e| StoreError::Deserialization(format!("Invalid tool ID: {}", e)))?;

        Ok(ToolPreference {
            context_pattern: record.context_pattern,
            preferred_tool_id,
            confidence: record.confidence,
            sample_count: record.sample_count,
            updated_at: record.updated_at.to_offset_datetime()?,
        })
    }
}

impl From<ToolUsageRecordWithId> for ToolUsageRecord {
    fn from(r: ToolUsageRecordWithId) -> Self {
        ToolUsageRecord {
            tool_id: r.tool_id,
            session_id: r.session_id,
            context: r.context,
            outcome: r.outcome,
            switched_to: r.switched_to,
            timestamp: r.timestamp,
        }
    }
}

/// Overall tool intelligence statistics.
#[derive(Debug, Clone, Default)]
pub struct ToolIntelStats {
    /// Number of usage records.
    pub usage_count: u64,
    /// Number of learned preferences.
    pub preference_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_intel_stats_default() {
        let stats = ToolIntelStats::default();
        assert_eq!(stats.usage_count, 0);
        assert_eq!(stats.preference_count, 0);
    }
}
