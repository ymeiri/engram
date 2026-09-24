//! Memory OS repository.
//!
//! Persists source-grounded memory items and Git-like knowledge commits.

use crate::error::{StoreError, StoreResult};
use crate::secret::{likely_secret_kind, reject_serialized_secret_material};
use crate::Db;
use engram_core::id::Id;
use engram_core::memory::{
    CorrectionProposal, CorrectionProposalStatus, KnowledgeCommit, MemoryItem, MemoryStatus,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use tracing::{debug, info};

fn snapshot_digest<T: Serialize>(value: &T) -> StoreResult<String> {
    let bytes = serde_json::to_vec(value)?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn reject_generic_proposal_markers(item: &MemoryItem) -> StoreResult<()> {
    if item.correction_proposal_id.is_some() || item.pending_correction_proposal_id.is_some() {
        return Err(StoreError::Policy(
            "correction proposal markers may only be written by dedicated atomic proposal transitions"
                .to_string(),
        ));
    }
    Ok(())
}

/// SurrealDB datetime representation.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum SurrealDateTime {
    /// ISO 8601 string format.
    String(String),
    /// SurrealDB native datetime format.
    Native(serde_json::Value),
}

impl SurrealDateTime {
    fn to_offset_datetime(&self) -> StoreResult<OffsetDateTime> {
        match self {
            SurrealDateTime::String(s) => {
                OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339)
                    .map_err(|e| StoreError::Deserialization(format!("Invalid datetime: {e}")))
            }
            SurrealDateTime::Native(v) => {
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
                        .map_err(|e| {
                            StoreError::Deserialization(format!("Invalid date in datetime: {e}"))
                        })?;

                        let time = time::Time::from_hms(hour, min, sec).map_err(|e| {
                            StoreError::Deserialization(format!("Invalid time in datetime: {e}"))
                        })?;

                        return Ok(OffsetDateTime::new_utc(date, time));
                    }
                }

                Err(StoreError::Deserialization(
                    "Invalid SurrealDB datetime value".to_string(),
                ))
            }
        }
    }
}

/// Record representation for memory items.
#[derive(Debug, Clone, Deserialize)]
struct MemoryItemRecord {
    record_id: String,
    item: serde_json::Value,
}

/// Record representation for typed correction proposals.
#[derive(Debug, Clone, Deserialize)]
struct CorrectionProposalRecord {
    record_id: String,
    proposal: serde_json::Value,
}

impl CorrectionProposalRecord {
    fn into_correction_proposal(self) -> StoreResult<CorrectionProposal> {
        let mut proposal: CorrectionProposal = from_json(self.proposal)?;
        proposal.id = Id::parse(&self.record_id).map_err(|e| {
            StoreError::Deserialization(format!("Invalid correction proposal ID: {e}"))
        })?;
        Ok(proposal)
    }
}

impl MemoryItemRecord {
    fn into_memory_item(self) -> StoreResult<MemoryItem> {
        let mut item: MemoryItem = from_json(self.item)?;
        item.id = Id::parse(&self.record_id)
            .map_err(|e| StoreError::Deserialization(format!("Invalid memory item ID: {e}")))?;
        Ok(item)
    }
}

/// Record representation for knowledge commits.
#[derive(Debug, Clone, Deserialize)]
struct KnowledgeCommitRecord {
    record_id: String,
    commit: serde_json::Value,
}

/// Internal references removed before permanently deleting a memory item.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MemoryReferencePurge {
    /// Other memory items updated to remove links or evidence naming the forgotten item.
    pub memory_items_updated: usize,
    /// Knowledge commits stripped of changes and messages associated with the forgotten item.
    pub commits_redacted: usize,
}

/// Correction projections atomically removed with a forgotten memory item.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CorrectionProjectionPurge {
    /// Whether the requested canonical memory item existed and was deleted.
    pub deleted: bool,
    /// Typed proposal records removed because their pair referenced the forgotten item.
    pub proposal_ids: Vec<Id>,
    /// Pending replacement items removed with a forgotten obsolete item.
    pub pending_replacement_ids: Vec<Id>,
    /// Active obsolete items unlocked when their pending replacement was forgotten.
    pub unlocked_obsolete_ids: Vec<Id>,
}

#[derive(Debug, Deserialize)]
struct CorrectionProjectionPurgeRecord {
    deleted: bool,
    proposal_ids: Vec<String>,
    pending_replacement_ids: Vec<String>,
    unlocked_obsolete_ids: Vec<String>,
}

impl KnowledgeCommitRecord {
    fn into_knowledge_commit(self) -> StoreResult<KnowledgeCommit> {
        let mut commit: KnowledgeCommit = from_json(self.commit)?;
        commit.id = Id::parse(&self.record_id).map_err(|e| {
            StoreError::Deserialization(format!("Invalid knowledge commit ID: {e}"))
        })?;
        Ok(commit)
    }
}

/// Latest memory update timestamp record.
#[derive(Debug, Clone, Deserialize)]
struct LatestMemoryTimestampRecord {
    updated_at: SurrealDateTime,
}

/// Latest knowledge commit timestamp record.
#[derive(Debug, Clone, Deserialize)]
struct LatestCommitTimestampRecord {
    created_at: SurrealDateTime,
}

/// Repository for Memory OS persistence.
#[derive(Clone)]
pub struct MemoryRepo {
    db: Db,
}

impl MemoryRepo {
    /// Create a new memory repository.
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    /// Initialize Memory OS tables and indexes.
    ///
    /// # Errors
    ///
    /// Returns an error if schema creation fails.
    pub async fn init_schema(&self) -> StoreResult<()> {
        info!("Initializing memory schema");

        self.db
            .query(
                r#"
                DEFINE TABLE IF NOT EXISTS memory_item SCHEMALESS;
                DEFINE INDEX IF NOT EXISTS idx_memory_item_kind ON memory_item FIELDS kind_key;
                DEFINE INDEX IF NOT EXISTS idx_memory_item_status ON memory_item FIELDS status_key;
                DEFINE INDEX IF NOT EXISTS idx_memory_item_scope ON memory_item FIELDS scope_key;
                DEFINE INDEX IF NOT EXISTS idx_memory_item_harness ON memory_item FIELDS harness_key;
                DEFINE INDEX IF NOT EXISTS idx_memory_item_session ON memory_item FIELDS session_id;
                DEFINE INDEX IF NOT EXISTS idx_memory_item_created ON memory_item FIELDS created_at;
                DEFINE INDEX IF NOT EXISTS idx_memory_item_updated ON memory_item FIELDS updated_at;

                DEFINE TABLE IF NOT EXISTS correction_proposal SCHEMALESS;
                DEFINE INDEX IF NOT EXISTS idx_correction_proposal_status ON correction_proposal FIELDS status_key;
                DEFINE INDEX IF NOT EXISTS idx_correction_proposal_obsolete ON correction_proposal FIELDS obsolete_id;
                DEFINE INDEX IF NOT EXISTS idx_correction_proposal_replacement ON correction_proposal FIELDS replacement_id;
                DEFINE INDEX IF NOT EXISTS idx_correction_proposal_created ON correction_proposal FIELDS created_at;
                DEFINE INDEX IF NOT EXISTS idx_correction_proposal_pending_obsolete ON correction_proposal FIELDS pending_obsolete_id UNIQUE;

                DEFINE TABLE IF NOT EXISTS memory_forget_receipt SCHEMALESS;

                DEFINE TABLE IF NOT EXISTS knowledge_commit SCHEMALESS;
                DEFINE INDEX IF NOT EXISTS idx_knowledge_commit_parent ON knowledge_commit FIELDS parent_id;
                DEFINE INDEX IF NOT EXISTS idx_knowledge_commit_session ON knowledge_commit FIELDS session_id;
                DEFINE INDEX IF NOT EXISTS idx_knowledge_commit_created ON knowledge_commit FIELDS created_at;
                "#,
            )
            .await?
            .check()?;

        info!("Memory schema initialized");
        Ok(())
    }

    /// Save a memory item.
    pub async fn save_memory_item(&self, item: &MemoryItem) -> StoreResult<()> {
        reject_generic_proposal_markers(item)?;
        reject_likely_secret_material(item)?;
        debug!("Saving memory item: {}", item.id);

        let item_json = to_json(item)?;
        let item_snapshot_digest = snapshot_digest(item)?;
        self.db
            .query(
                r#"
                BEGIN TRANSACTION;
                LET $current_item = (
                    SELECT VALUE item FROM type::thing("memory_item", $id)
                );
                IF array::len($current_item) > 0
                    AND (
                        $current_item[0].correction_proposal_id != NONE
                        OR $current_item[0].pending_correction_proposal_id != NONE
                    )
                {
                    THROW "proposal-bound memory requires its dedicated atomic transition";
                };
                UPSERT type::thing("memory_item", $id) SET
                    item = $item,
                    kind_key = $kind_key,
                    status_key = $status_key,
                    scope_key = $scope_key,
                    harness_key = $harness_key,
                    model_key = $model_key,
                    session_id = $session_id,
                    snapshot_digest = $snapshot_digest,
                    created_at = $created_at,
                    updated_at = $updated_at;
                COMMIT TRANSACTION;
                "#,
            )
            .bind(("id", item.id.to_string()))
            .bind(("item", item_json))
            .bind(("kind_key", item.kind.to_string()))
            .bind(("status_key", item.status.to_string()))
            .bind(("scope_key", scope_key(item)))
            .bind(("harness_key", item.writer.harness.to_string()))
            .bind(("model_key", item.writer.model.model.clone()))
            .bind((
                "session_id",
                item.writer.session_id.map(|id| id.to_string()),
            ))
            .bind(("snapshot_digest", item_snapshot_digest))
            .bind(("created_at", format_rfc3339(item.created_at)?))
            .bind(("updated_at", format_rfc3339(item.updated_at)?))
            .await?
            .check()?;

        Ok(())
    }

    /// Persist one existing memory item only if its previously-read snapshot is still current.
    pub async fn save_memory_item_if_unchanged(
        &self,
        expected: &MemoryItem,
        updated: &MemoryItem,
    ) -> StoreResult<()> {
        reject_generic_proposal_markers(updated)?;
        reject_likely_secret_material(updated)?;
        let expected_snapshot_digest = snapshot_digest(expected)?;
        let updated_snapshot_digest = snapshot_digest(updated)?;
        let updated_json = to_json(updated)?;
        self.db
            .query(
                r#"
                BEGIN TRANSACTION;
                LET $updated = (
                    UPDATE type::thing("memory_item", $id) SET
                        item = $item,
                        kind_key = $kind_key,
                        status_key = $status_key,
                        scope_key = $scope_key,
                        harness_key = $harness_key,
                        model_key = $model_key,
                        session_id = $session_id,
                        snapshot_digest = $updated_snapshot_digest,
                        created_at = $created_at,
                        updated_at = $updated_at
                    WHERE (
                        snapshot_digest = $expected_snapshot_digest
                        OR (
                            snapshot_digest = NONE
                            AND updated_at = $expected_updated_at
                            AND status_key = $expected_status_key
                            AND kind_key = $expected_kind_key
                            AND scope_key = $expected_scope_key
                        )
                    )
                    AND item.correction_proposal_id = NONE
                    AND item.pending_correction_proposal_id = NONE
                    RETURN VALUE id
                );
                IF array::len($updated) != 1 {
                    THROW "memory item changed before guarded update";
                };
                COMMIT TRANSACTION;
                "#,
            )
            .bind(("id", updated.id.to_string()))
            .bind(("item", updated_json))
            .bind(("kind_key", updated.kind.to_string()))
            .bind(("status_key", updated.status.to_string()))
            .bind(("scope_key", scope_key(updated)))
            .bind(("harness_key", updated.writer.harness.to_string()))
            .bind(("model_key", updated.writer.model.model.clone()))
            .bind((
                "session_id",
                updated.writer.session_id.map(|id| id.to_string()),
            ))
            .bind(("updated_snapshot_digest", updated_snapshot_digest))
            .bind(("created_at", format_rfc3339(updated.created_at)?))
            .bind(("updated_at", format_rfc3339(updated.updated_at)?))
            .bind(("expected_snapshot_digest", expected_snapshot_digest))
            .bind(("expected_updated_at", format_rfc3339(expected.updated_at)?))
            .bind(("expected_status_key", expected.status.to_string()))
            .bind(("expected_kind_key", expected.kind.to_string()))
            .bind(("expected_scope_key", scope_key(expected)))
            .await?
            .check()?;
        Ok(())
    }

    /// Atomically create a server-minted correction proposal and its pending replacement.
    pub async fn save_correction_proposal(
        &self,
        proposal: &CorrectionProposal,
        replacement: &MemoryItem,
        expected_obsolete: &MemoryItem,
        locked_obsolete: &MemoryItem,
    ) -> StoreResult<()> {
        reject_likely_secret_material(replacement)?;
        reject_likely_secret_material(expected_obsolete)?;
        reject_likely_secret_material(locked_obsolete)?;
        reject_serialized_secret_material("correction_proposal", proposal)?;

        let proposal_json = to_json(proposal)?;
        let replacement_json = to_json(replacement)?;
        let expected_obsolete_snapshot_digest = snapshot_digest(expected_obsolete)?;
        let replacement_snapshot_digest = snapshot_digest(replacement)?;
        let proposal_snapshot_digest = snapshot_digest(proposal)?;
        let locked_obsolete_json = to_json(locked_obsolete)?;
        let locked_obsolete_snapshot_digest = snapshot_digest(locked_obsolete)?;
        self.db
            .query(
                r#"
                BEGIN TRANSACTION;
                LET $guarded_obsolete = (
                    UPDATE type::thing("memory_item", $obsolete_id)
                    SET item = $locked_obsolete_item,
                        snapshot_digest = $locked_obsolete_snapshot_digest
                    WHERE updated_at = $expected_obsolete_updated_at
                        AND status_key = $expected_obsolete_status_key
                        AND kind_key = $expected_obsolete_kind_key
                        AND scope_key = $expected_obsolete_scope_key
                        AND (snapshot_digest = NONE
                            OR snapshot_digest = $expected_obsolete_snapshot_digest)
                    RETURN VALUE snapshot_digest
                );
                IF array::len($guarded_obsolete) != 1 {
                    THROW "obsolete memory changed before correction proposal creation";
                };
                CREATE type::thing("memory_item", $replacement_id) SET
                    item = $replacement_item,
                    kind_key = $replacement_kind_key,
                    status_key = $replacement_status_key,
                    scope_key = $replacement_scope_key,
                    harness_key = $replacement_harness_key,
                    model_key = $replacement_model_key,
                    session_id = $replacement_session_id,
                    snapshot_digest = $replacement_snapshot_digest,
                    created_at = $replacement_created_at,
                    updated_at = $replacement_updated_at;
                CREATE type::thing("correction_proposal", $proposal_id) SET
                    proposal = $proposal,
                    status_key = $proposal_status_key,
                    obsolete_id = $obsolete_id,
                    pending_obsolete_id = $obsolete_id,
                    replacement_id = $replacement_id,
                    snapshot_digest = $proposal_snapshot_digest,
                    created_at = $proposal_created_at;
                COMMIT TRANSACTION;
                "#,
            )
            .bind((
                "expected_obsolete_snapshot_digest",
                expected_obsolete_snapshot_digest,
            ))
            .bind(("locked_obsolete_item", locked_obsolete_json))
            .bind((
                "locked_obsolete_snapshot_digest",
                locked_obsolete_snapshot_digest,
            ))
            .bind((
                "expected_obsolete_updated_at",
                format_rfc3339(expected_obsolete.updated_at)?,
            ))
            .bind((
                "expected_obsolete_status_key",
                expected_obsolete.status.to_string(),
            ))
            .bind((
                "expected_obsolete_kind_key",
                expected_obsolete.kind.to_string(),
            ))
            .bind(("expected_obsolete_scope_key", scope_key(expected_obsolete)))
            .bind(("replacement_id", replacement.id.to_string()))
            .bind(("replacement_item", replacement_json))
            .bind(("replacement_kind_key", replacement.kind.to_string()))
            .bind(("replacement_status_key", replacement.status.to_string()))
            .bind(("replacement_scope_key", scope_key(replacement)))
            .bind((
                "replacement_harness_key",
                replacement.writer.harness.to_string(),
            ))
            .bind((
                "replacement_model_key",
                replacement.writer.model.model.clone(),
            ))
            .bind((
                "replacement_session_id",
                replacement.writer.session_id.map(|id| id.to_string()),
            ))
            .bind(("replacement_snapshot_digest", replacement_snapshot_digest))
            .bind((
                "replacement_created_at",
                format_rfc3339(replacement.created_at)?,
            ))
            .bind((
                "replacement_updated_at",
                format_rfc3339(replacement.updated_at)?,
            ))
            .bind(("proposal_id", proposal.id.to_string()))
            .bind(("proposal", proposal_json))
            .bind(("proposal_status_key", proposal.status.to_string()))
            .bind(("obsolete_id", proposal.obsolete_id.to_string()))
            .bind(("proposal_created_at", format_rfc3339(proposal.created_at)?))
            .bind(("proposal_snapshot_digest", proposal_snapshot_digest))
            .await?
            .check()?;

        Ok(())
    }

    /// Atomically attach verification proof to an inactive procedure replacement and rotate the
    /// pending proposal digest. All three records are compare-and-swapped; the obsolete procedure
    /// remains active and the replacement remains `needs_review`.
    pub async fn verify_correction_procedure(
        &self,
        expected_proposal: &CorrectionProposal,
        expected_replacement: &MemoryItem,
        expected_obsolete: &MemoryItem,
        verified_proposal: &CorrectionProposal,
        verified_replacement: &MemoryItem,
        unchanged_obsolete: &MemoryItem,
    ) -> StoreResult<()> {
        reject_likely_secret_material(verified_replacement)?;
        reject_likely_secret_material(unchanged_obsolete)?;
        reject_serialized_secret_material("correction_proposal", verified_proposal)?;

        let expected_proposal_snapshot_digest = snapshot_digest(expected_proposal)?;
        let expected_replacement_snapshot_digest = snapshot_digest(expected_replacement)?;
        let expected_obsolete_snapshot_digest = snapshot_digest(expected_obsolete)?;
        let proposal_json = to_json(verified_proposal)?;
        let replacement_json = to_json(verified_replacement)?;
        let obsolete_json = to_json(unchanged_obsolete)?;
        let proposal_snapshot_digest = snapshot_digest(verified_proposal)?;
        let replacement_snapshot_digest = snapshot_digest(verified_replacement)?;
        let obsolete_snapshot_digest = snapshot_digest(unchanged_obsolete)?;

        self.db
            .query(
                r#"
                BEGIN TRANSACTION;
                LET $updated_replacement = (
                    UPDATE type::thing("memory_item", $replacement_id) SET
                        item = $replacement_item,
                        kind_key = $replacement_kind_key,
                        status_key = $replacement_status_key,
                        scope_key = $replacement_scope_key,
                        harness_key = $replacement_harness_key,
                        model_key = $replacement_model_key,
                        session_id = $replacement_session_id,
                        snapshot_digest = $replacement_snapshot_digest,
                        created_at = $replacement_created_at,
                        updated_at = $replacement_updated_at
                    WHERE snapshot_digest = $expected_replacement_snapshot_digest
                    RETURN VALUE id
                );
                LET $updated_obsolete = (
                    UPDATE type::thing("memory_item", $obsolete_id) SET
                        item = $obsolete_item,
                        kind_key = $obsolete_kind_key,
                        status_key = $obsolete_status_key,
                        scope_key = $obsolete_scope_key,
                        harness_key = $obsolete_harness_key,
                        model_key = $obsolete_model_key,
                        session_id = $obsolete_session_id,
                        snapshot_digest = $obsolete_snapshot_digest,
                        created_at = $obsolete_created_at,
                        updated_at = $obsolete_updated_at
                    WHERE snapshot_digest = $expected_obsolete_snapshot_digest
                    RETURN VALUE id
                );
                LET $updated_proposal = (
                    UPDATE type::thing("correction_proposal", $proposal_id) SET
                        proposal = $proposal,
                        status_key = $proposal_status_key,
                        obsolete_id = $obsolete_id,
                        pending_obsolete_id = $obsolete_id,
                        replacement_id = $replacement_id,
                        snapshot_digest = $proposal_snapshot_digest,
                        created_at = $proposal_created_at
                    WHERE snapshot_digest = $expected_proposal_snapshot_digest
                    RETURN VALUE id
                );
                IF array::len($updated_replacement) != 1
                    OR array::len($updated_obsolete) != 1
                    OR array::len($updated_proposal) != 1
                {
                    THROW "procedure correction changed before atomic verification";
                };
                COMMIT TRANSACTION;
                "#,
            )
            .bind((
                "expected_proposal_snapshot_digest",
                expected_proposal_snapshot_digest,
            ))
            .bind((
                "expected_replacement_snapshot_digest",
                expected_replacement_snapshot_digest,
            ))
            .bind((
                "expected_obsolete_snapshot_digest",
                expected_obsolete_snapshot_digest,
            ))
            .bind(("replacement_id", verified_replacement.id.to_string()))
            .bind(("replacement_item", replacement_json))
            .bind((
                "replacement_kind_key",
                verified_replacement.kind.to_string(),
            ))
            .bind((
                "replacement_status_key",
                verified_replacement.status.to_string(),
            ))
            .bind(("replacement_scope_key", scope_key(verified_replacement)))
            .bind((
                "replacement_harness_key",
                verified_replacement.writer.harness.to_string(),
            ))
            .bind((
                "replacement_model_key",
                verified_replacement.writer.model.model.clone(),
            ))
            .bind((
                "replacement_session_id",
                verified_replacement
                    .writer
                    .session_id
                    .map(|id| id.to_string()),
            ))
            .bind((
                "replacement_created_at",
                format_rfc3339(verified_replacement.created_at)?,
            ))
            .bind((
                "replacement_updated_at",
                format_rfc3339(verified_replacement.updated_at)?,
            ))
            .bind(("replacement_snapshot_digest", replacement_snapshot_digest))
            .bind(("obsolete_id", unchanged_obsolete.id.to_string()))
            .bind(("obsolete_item", obsolete_json))
            .bind(("obsolete_kind_key", unchanged_obsolete.kind.to_string()))
            .bind(("obsolete_status_key", unchanged_obsolete.status.to_string()))
            .bind(("obsolete_scope_key", scope_key(unchanged_obsolete)))
            .bind((
                "obsolete_harness_key",
                unchanged_obsolete.writer.harness.to_string(),
            ))
            .bind((
                "obsolete_model_key",
                unchanged_obsolete.writer.model.model.clone(),
            ))
            .bind((
                "obsolete_session_id",
                unchanged_obsolete
                    .writer
                    .session_id
                    .map(|id| id.to_string()),
            ))
            .bind((
                "obsolete_created_at",
                format_rfc3339(unchanged_obsolete.created_at)?,
            ))
            .bind((
                "obsolete_updated_at",
                format_rfc3339(unchanged_obsolete.updated_at)?,
            ))
            .bind(("obsolete_snapshot_digest", obsolete_snapshot_digest))
            .bind(("proposal_id", verified_proposal.id.to_string()))
            .bind(("proposal", proposal_json))
            .bind(("proposal_status_key", verified_proposal.status.to_string()))
            .bind((
                "proposal_created_at",
                format_rfc3339(verified_proposal.created_at)?,
            ))
            .bind(("proposal_snapshot_digest", proposal_snapshot_digest))
            .await?
            .check()?;

        Ok(())
    }

    /// Get a typed correction proposal by ID.
    pub async fn get_correction_proposal(
        &self,
        id: &Id,
    ) -> StoreResult<Option<CorrectionProposal>> {
        let mut result = self
            .db
            .query(
                r#"
                SELECT meta::id(id) AS record_id, proposal
                FROM type::thing("correction_proposal", $id)
                "#,
            )
            .bind(("id", id.to_string()))
            .await?
            .check()?;

        let records: Vec<CorrectionProposalRecord> = result.take(0)?;
        records
            .into_iter()
            .next()
            .map(CorrectionProposalRecord::into_correction_proposal)
            .transpose()
    }

    /// List typed correction proposals, newest first.
    pub async fn list_correction_proposals(
        &self,
        status: Option<CorrectionProposalStatus>,
        limit: Option<usize>,
    ) -> StoreResult<Vec<CorrectionProposal>> {
        let mut query =
            "SELECT meta::id(id) AS record_id, proposal, created_at FROM correction_proposal"
                .to_string();
        if status.is_some() {
            query.push_str(" WHERE status_key = $status");
        }
        query.push_str(" ORDER BY created_at DESC");
        if let Some(limit) = limit {
            query.push_str(&format!(" LIMIT {limit}"));
        }

        let mut result = if let Some(status) = status {
            self.db
                .query(query)
                .bind(("status", status.to_string()))
                .await?
                .check()?
        } else {
            self.db.query(query).await?.check()?
        };
        let records: Vec<CorrectionProposalRecord> = result.take(0)?;
        records
            .into_iter()
            .map(CorrectionProposalRecord::into_correction_proposal)
            .collect()
    }

    /// List correction proposals whose immutable pair references a memory item.
    pub async fn list_correction_proposals_for_memory(
        &self,
        id: &Id,
    ) -> StoreResult<Vec<CorrectionProposal>> {
        let id = id.to_string();
        let mut result = self
            .db
            .query(
                r#"
                SELECT meta::id(id) AS record_id, proposal, created_at
                FROM correction_proposal
                WHERE obsolete_id = $id OR replacement_id = $id
                ORDER BY created_at DESC
                "#,
            )
            .bind(("id", id))
            .await?
            .check()?;
        let records: Vec<CorrectionProposalRecord> = result.take(0)?;
        records
            .into_iter()
            .map(CorrectionProposalRecord::into_correction_proposal)
            .collect()
    }

    /// Atomically activate a proposed replacement, supersede the old item, and consume the
    /// proposal record, but only while all three records still match the snapshots authorized by
    /// the service.
    pub async fn apply_correction_proposal(
        &self,
        expected_proposal: &CorrectionProposal,
        expected_replacement: &MemoryItem,
        expected_obsolete: &MemoryItem,
        applied_proposal: &CorrectionProposal,
        active_replacement: &MemoryItem,
        superseded_obsolete: &MemoryItem,
    ) -> StoreResult<()> {
        reject_likely_secret_material(active_replacement)?;
        reject_likely_secret_material(superseded_obsolete)?;
        reject_serialized_secret_material("correction_proposal", applied_proposal)?;

        let expected_proposal_snapshot_digest = snapshot_digest(expected_proposal)?;
        let expected_replacement_snapshot_digest = snapshot_digest(expected_replacement)?;
        let expected_obsolete_snapshot_digest = snapshot_digest(expected_obsolete)?;
        let proposal_json = to_json(applied_proposal)?;
        let replacement_json = to_json(active_replacement)?;
        let obsolete_json = to_json(superseded_obsolete)?;
        let proposal_snapshot_digest = snapshot_digest(applied_proposal)?;
        let replacement_snapshot_digest = snapshot_digest(active_replacement)?;
        let obsolete_snapshot_digest = snapshot_digest(superseded_obsolete)?;
        self.db
            .query(
                r#"
                BEGIN TRANSACTION;
                LET $updated_replacement = (
                UPDATE type::thing("memory_item", $replacement_id) SET
                    item = $replacement_item,
                    kind_key = $replacement_kind_key,
                    status_key = $replacement_status_key,
                    scope_key = $replacement_scope_key,
                    harness_key = $replacement_harness_key,
                    model_key = $replacement_model_key,
                    session_id = $replacement_session_id,
                    snapshot_digest = $replacement_snapshot_digest,
                    created_at = $replacement_created_at,
                    updated_at = $replacement_updated_at
                WHERE snapshot_digest = $expected_replacement_snapshot_digest
                RETURN VALUE id
                );
                LET $updated_obsolete = (
                UPDATE type::thing("memory_item", $obsolete_id) SET
                    item = $obsolete_item,
                    kind_key = $obsolete_kind_key,
                    status_key = $obsolete_status_key,
                    scope_key = $obsolete_scope_key,
                    harness_key = $obsolete_harness_key,
                    model_key = $obsolete_model_key,
                    session_id = $obsolete_session_id,
                    snapshot_digest = $obsolete_snapshot_digest,
                    created_at = $obsolete_created_at,
                    updated_at = $obsolete_updated_at
                WHERE snapshot_digest = $expected_obsolete_snapshot_digest
                RETURN VALUE id
                );
                LET $updated_proposal = (
                UPDATE type::thing("correction_proposal", $proposal_id) SET
                    proposal = $proposal,
                    status_key = $proposal_status_key,
                    obsolete_id = $obsolete_id,
                    pending_obsolete_id = NONE,
                    replacement_id = $replacement_id,
                    snapshot_digest = $proposal_snapshot_digest,
                    created_at = $proposal_created_at
                WHERE snapshot_digest = $expected_proposal_snapshot_digest
                RETURN VALUE id
                );
                IF array::len($updated_replacement) != 1
                    OR array::len($updated_obsolete) != 1
                    OR array::len($updated_proposal) != 1
                {
                    THROW "correction proposal changed before atomic apply";
                };
                COMMIT TRANSACTION;
                "#,
            )
            .bind((
                "expected_proposal_snapshot_digest",
                expected_proposal_snapshot_digest,
            ))
            .bind((
                "expected_replacement_snapshot_digest",
                expected_replacement_snapshot_digest,
            ))
            .bind((
                "expected_obsolete_snapshot_digest",
                expected_obsolete_snapshot_digest,
            ))
            .bind(("replacement_id", active_replacement.id.to_string()))
            .bind(("replacement_item", replacement_json))
            .bind(("replacement_kind_key", active_replacement.kind.to_string()))
            .bind((
                "replacement_status_key",
                active_replacement.status.to_string(),
            ))
            .bind(("replacement_scope_key", scope_key(active_replacement)))
            .bind((
                "replacement_harness_key",
                active_replacement.writer.harness.to_string(),
            ))
            .bind((
                "replacement_model_key",
                active_replacement.writer.model.model.clone(),
            ))
            .bind((
                "replacement_session_id",
                active_replacement
                    .writer
                    .session_id
                    .map(|id| id.to_string()),
            ))
            .bind((
                "replacement_created_at",
                format_rfc3339(active_replacement.created_at)?,
            ))
            .bind((
                "replacement_updated_at",
                format_rfc3339(active_replacement.updated_at)?,
            ))
            .bind(("replacement_snapshot_digest", replacement_snapshot_digest))
            .bind(("obsolete_id", superseded_obsolete.id.to_string()))
            .bind(("obsolete_item", obsolete_json))
            .bind(("obsolete_kind_key", superseded_obsolete.kind.to_string()))
            .bind((
                "obsolete_status_key",
                superseded_obsolete.status.to_string(),
            ))
            .bind(("obsolete_scope_key", scope_key(superseded_obsolete)))
            .bind((
                "obsolete_harness_key",
                superseded_obsolete.writer.harness.to_string(),
            ))
            .bind((
                "obsolete_model_key",
                superseded_obsolete.writer.model.model.clone(),
            ))
            .bind((
                "obsolete_session_id",
                superseded_obsolete
                    .writer
                    .session_id
                    .map(|id| id.to_string()),
            ))
            .bind((
                "obsolete_created_at",
                format_rfc3339(superseded_obsolete.created_at)?,
            ))
            .bind((
                "obsolete_updated_at",
                format_rfc3339(superseded_obsolete.updated_at)?,
            ))
            .bind(("obsolete_snapshot_digest", obsolete_snapshot_digest))
            .bind(("proposal_id", applied_proposal.id.to_string()))
            .bind(("proposal", proposal_json))
            .bind(("proposal_status_key", applied_proposal.status.to_string()))
            .bind((
                "proposal_created_at",
                format_rfc3339(applied_proposal.created_at)?,
            ))
            .bind(("proposal_snapshot_digest", proposal_snapshot_digest))
            .await?
            .check()?;

        Ok(())
    }

    /// Atomically persist an active correction and the item it supersedes.
    pub async fn save_memory_correction(
        &self,
        expected_replacement: &MemoryItem,
        expected_obsolete: &MemoryItem,
        corrected_replacement: &MemoryItem,
        corrected_obsolete: &MemoryItem,
    ) -> StoreResult<()> {
        reject_generic_proposal_markers(corrected_replacement)?;
        reject_generic_proposal_markers(corrected_obsolete)?;
        reject_likely_secret_material(corrected_replacement)?;
        reject_likely_secret_material(corrected_obsolete)?;

        let expected_replacement_snapshot_digest = snapshot_digest(expected_replacement)?;
        let expected_obsolete_snapshot_digest = snapshot_digest(expected_obsolete)?;
        let replacement_json = to_json(corrected_replacement)?;
        let obsolete_json = to_json(corrected_obsolete)?;
        let replacement_snapshot_digest = snapshot_digest(corrected_replacement)?;
        let obsolete_snapshot_digest = snapshot_digest(corrected_obsolete)?;
        self.db
            .query(
                r#"
                BEGIN TRANSACTION;
                LET $updated_replacement = (
                UPDATE type::thing("memory_item", $replacement_id) SET
                    item = $replacement_item,
                    kind_key = $replacement_kind_key,
                    status_key = $replacement_status_key,
                    scope_key = $replacement_scope_key,
                    harness_key = $replacement_harness_key,
                    model_key = $replacement_model_key,
                    session_id = $replacement_session_id,
                    snapshot_digest = $replacement_snapshot_digest,
                    created_at = $replacement_created_at,
                    updated_at = $replacement_updated_at
                WHERE (
                    snapshot_digest = $expected_replacement_snapshot_digest
                    OR (
                        snapshot_digest = NONE
                        AND updated_at = $expected_replacement_updated_at
                        AND status_key = $expected_replacement_status_key
                        AND kind_key = $expected_replacement_kind_key
                        AND scope_key = $expected_replacement_scope_key
                    )
                )
                AND item.correction_proposal_id = NONE
                AND item.pending_correction_proposal_id = NONE
                RETURN VALUE id
                );
                LET $updated_obsolete = (
                UPDATE type::thing("memory_item", $obsolete_id) SET
                    item = $obsolete_item,
                    kind_key = $obsolete_kind_key,
                    status_key = $obsolete_status_key,
                    scope_key = $obsolete_scope_key,
                    harness_key = $obsolete_harness_key,
                    model_key = $obsolete_model_key,
                    session_id = $obsolete_session_id,
                    snapshot_digest = $obsolete_snapshot_digest,
                    created_at = $obsolete_created_at,
                    updated_at = $obsolete_updated_at
                WHERE (
                    snapshot_digest = $expected_obsolete_snapshot_digest
                    OR (
                        snapshot_digest = NONE
                        AND updated_at = $expected_obsolete_updated_at
                        AND status_key = $expected_obsolete_status_key
                        AND kind_key = $expected_obsolete_kind_key
                        AND scope_key = $expected_obsolete_scope_key
                    )
                )
                AND item.correction_proposal_id = NONE
                AND item.pending_correction_proposal_id = NONE
                RETURN VALUE id
                );
                IF array::len($updated_replacement) != 1
                    OR array::len($updated_obsolete) != 1
                {
                    THROW "direct correction pair changed before atomic apply";
                };
                COMMIT TRANSACTION;
                "#,
            )
            .bind((
                "expected_replacement_snapshot_digest",
                expected_replacement_snapshot_digest,
            ))
            .bind((
                "expected_obsolete_snapshot_digest",
                expected_obsolete_snapshot_digest,
            ))
            .bind((
                "expected_replacement_updated_at",
                format_rfc3339(expected_replacement.updated_at)?,
            ))
            .bind((
                "expected_replacement_status_key",
                expected_replacement.status.to_string(),
            ))
            .bind((
                "expected_replacement_kind_key",
                expected_replacement.kind.to_string(),
            ))
            .bind((
                "expected_replacement_scope_key",
                scope_key(expected_replacement),
            ))
            .bind((
                "expected_obsolete_updated_at",
                format_rfc3339(expected_obsolete.updated_at)?,
            ))
            .bind((
                "expected_obsolete_status_key",
                expected_obsolete.status.to_string(),
            ))
            .bind((
                "expected_obsolete_kind_key",
                expected_obsolete.kind.to_string(),
            ))
            .bind(("expected_obsolete_scope_key", scope_key(expected_obsolete)))
            .bind(("replacement_id", corrected_replacement.id.to_string()))
            .bind(("replacement_item", replacement_json))
            .bind((
                "replacement_kind_key",
                corrected_replacement.kind.to_string(),
            ))
            .bind((
                "replacement_status_key",
                corrected_replacement.status.to_string(),
            ))
            .bind(("replacement_scope_key", scope_key(corrected_replacement)))
            .bind((
                "replacement_harness_key",
                corrected_replacement.writer.harness.to_string(),
            ))
            .bind((
                "replacement_model_key",
                corrected_replacement.writer.model.model.clone(),
            ))
            .bind((
                "replacement_session_id",
                corrected_replacement
                    .writer
                    .session_id
                    .map(|id| id.to_string()),
            ))
            .bind((
                "replacement_created_at",
                format_rfc3339(corrected_replacement.created_at)?,
            ))
            .bind((
                "replacement_updated_at",
                format_rfc3339(corrected_replacement.updated_at)?,
            ))
            .bind(("replacement_snapshot_digest", replacement_snapshot_digest))
            .bind(("obsolete_id", corrected_obsolete.id.to_string()))
            .bind(("obsolete_item", obsolete_json))
            .bind(("obsolete_kind_key", corrected_obsolete.kind.to_string()))
            .bind(("obsolete_status_key", corrected_obsolete.status.to_string()))
            .bind(("obsolete_scope_key", scope_key(corrected_obsolete)))
            .bind((
                "obsolete_harness_key",
                corrected_obsolete.writer.harness.to_string(),
            ))
            .bind((
                "obsolete_model_key",
                corrected_obsolete.writer.model.model.clone(),
            ))
            .bind((
                "obsolete_session_id",
                corrected_obsolete
                    .writer
                    .session_id
                    .map(|id| id.to_string()),
            ))
            .bind((
                "obsolete_created_at",
                format_rfc3339(corrected_obsolete.created_at)?,
            ))
            .bind((
                "obsolete_updated_at",
                format_rfc3339(corrected_obsolete.updated_at)?,
            ))
            .bind(("obsolete_snapshot_digest", obsolete_snapshot_digest))
            .await?
            .check()?;

        Ok(())
    }

    /// Get a memory item by ID.
    pub async fn get_memory_item(&self, id: &Id) -> StoreResult<Option<MemoryItem>> {
        debug!("Getting memory item: {id}");

        let mut result = self
            .db
            .query(
                r#"
                SELECT meta::id(id) AS record_id, item
                FROM type::thing("memory_item", $id)
                "#,
            )
            .bind(("id", id.to_string()))
            .await?
            .check()?;

        let records: Vec<MemoryItemRecord> = result.take(0)?;
        records
            .into_iter()
            .next()
            .map(MemoryItemRecord::into_memory_item)
            .transpose()
    }

    /// Permanently delete a memory item from the canonical store.
    #[cfg(test)]
    async fn delete_memory_item(&self, id: &Id) -> StoreResult<bool> {
        if self.get_memory_item(id).await?.is_none() {
            return Ok(false);
        }
        self.db
            .query(r#"DELETE type::thing("memory_item", $id)"#)
            .bind(("id", id.to_string()))
            .await?
            .check()?;
        Ok(true)
    }

    /// Atomically delete one memory item and correction projections owned by its pair(s).
    pub async fn delete_memory_with_correction_projections(
        &self,
        id: &Id,
    ) -> StoreResult<CorrectionProjectionPurge> {
        let mut result = self
            .db
            .query(
                r#"
                BEGIN TRANSACTION;
                LET $target_ids = (
                    SELECT VALUE meta::id(id)
                    FROM type::thing("memory_item", $id)
                );
                LET $proposal_ids = (
                    SELECT VALUE meta::id(id)
                    FROM correction_proposal
                    WHERE obsolete_id = $id OR replacement_id = $id
                );
                LET $pending_replacement_ids = (
                    SELECT VALUE replacement_id
                    FROM correction_proposal
                    WHERE obsolete_id = $id AND status_key = "pending"
                );
                LET $unlocked_obsolete_ids = (
                    SELECT VALUE obsolete_id
                    FROM correction_proposal
                    WHERE replacement_id = $id AND status_key = "pending"
                );
                UPDATE memory_item
                    SET item.pending_correction_proposal_id = NONE,
                        snapshot_digest = NONE
                    WHERE meta::id(id) IN (
                        SELECT VALUE obsolete_id
                        FROM correction_proposal
                        WHERE replacement_id = $id AND status_key = "pending"
                    );
                DELETE memory_item
                    WHERE meta::id(id) IN $pending_replacement_ids;
                DELETE correction_proposal
                    WHERE obsolete_id = $id OR replacement_id = $id;
                DELETE type::thing("memory_item", $id);
                IF array::len($target_ids) = 1 {
                    UPSERT type::thing("memory_forget_receipt", $id) SET
                        deleted = true,
                        proposal_ids = $proposal_ids,
                        pending_replacement_ids = $pending_replacement_ids,
                        unlocked_obsolete_ids = $unlocked_obsolete_ids,
                        cleanup_complete = false,
                        created_at = time::now(),
                        completed_at = NONE;
                };
                RETURN {
                    deleted: array::len($target_ids) = 1,
                    proposal_ids: $proposal_ids,
                    pending_replacement_ids: $pending_replacement_ids,
                    unlocked_obsolete_ids: $unlocked_obsolete_ids
                };
                COMMIT TRANSACTION;
                "#,
            )
            .bind(("id", id.to_string()))
            .await?
            .check()?;
        let record: Option<CorrectionProjectionPurgeRecord> = result.take(0)?;
        let record = record.ok_or_else(|| {
            StoreError::Deserialization(
                "memory forget transaction did not return its correction projection set"
                    .to_string(),
            )
        })?;
        decode_correction_projection_purge(record)
    }

    /// Load a content-free receipt for canonical deletion whose projection cleanup is incomplete.
    pub async fn get_pending_memory_forget_receipt(
        &self,
        id: &Id,
    ) -> StoreResult<Option<CorrectionProjectionPurge>> {
        let mut result = self
            .db
            .query(
                r#"
                SELECT deleted, proposal_ids, pending_replacement_ids, unlocked_obsolete_ids
                FROM type::thing("memory_forget_receipt", $id)
                WHERE cleanup_complete = false
                "#,
            )
            .bind(("id", id.to_string()))
            .await?
            .check()?;
        let record: Option<CorrectionProjectionPurgeRecord> = result.take(0)?;
        record.map(decode_correction_projection_purge).transpose()
    }

    /// Remove the content-free retry receipt after every idempotent purge has succeeded.
    pub async fn complete_memory_forget_receipt(&self, id: &Id) -> StoreResult<()> {
        self.db
            .query(
                r#"
                DELETE type::thing("memory_forget_receipt", $id)
                WHERE cleanup_complete = false
                "#,
            )
            .bind(("id", id.to_string()))
            .await?
            .check()?;
        Ok(())
    }

    /// Remove internal memory and commit references to an item before permanent deletion.
    pub async fn purge_memory_item_references(&self, id: &Id) -> StoreResult<MemoryReferencePurge> {
        let id_text = id.to_string();
        let mut purge = MemoryReferencePurge::default();

        for mut item in self.list_memory_items(None, None).await? {
            if item.id == *id {
                continue;
            }
            let expected = item.clone();
            let supersedes_before = item.supersedes.len();
            let evidence_before = item.evidence.len();
            item.supersedes.retain(|candidate| candidate != id);
            item.evidence.retain(|evidence| {
                !evidence.target.contains(&id_text)
                    && !evidence
                        .summary
                        .as_deref()
                        .is_some_and(|summary| summary.contains(&id_text))
                    && !evidence
                        .excerpt
                        .as_deref()
                        .is_some_and(|excerpt| excerpt.contains(&id_text))
            });
            if item.supersedes.len() != supersedes_before || item.evidence.len() != evidence_before
            {
                item.updated_at = OffsetDateTime::now_utc();
                self.save_memory_item_if_unchanged(&expected, &item).await?;
                purge.memory_items_updated += 1;
            }
        }

        for mut commit in self.list_knowledge_commits(None).await? {
            let change_count = commit.changes.len();
            commit.changes.retain(|change| change.item_id != Some(*id));
            if commit.changes.len() != change_count {
                commit.message = "[redacted after memory forget]".to_string();
                self.save_knowledge_commit(&commit).await?;
                purge.commits_redacted += 1;
            }
        }

        Ok(purge)
    }

    /// List memory items, newest updates first.
    pub async fn list_memory_items(
        &self,
        status: Option<MemoryStatus>,
        limit: Option<usize>,
    ) -> StoreResult<Vec<MemoryItem>> {
        debug!("Listing memory items (status: {status:?})");

        let mut query =
            "SELECT meta::id(id) AS record_id, item, updated_at FROM memory_item".to_string();
        if status.is_some() {
            query.push_str(" WHERE status_key = $status");
        }
        query.push_str(" ORDER BY updated_at DESC");
        if let Some(limit) = limit {
            query.push_str(&format!(" LIMIT {limit}"));
        }

        let mut result = if let Some(status) = status {
            self.db
                .query(query)
                .bind(("status", status.to_string()))
                .await?
                .check()?
        } else {
            self.db.query(query).await?.check()?
        };

        decode_memory_items(result.take(0)?)
    }

    /// List memory items updated after a timestamp.
    pub async fn list_memory_items_updated_after(
        &self,
        timestamp: OffsetDateTime,
        limit: Option<usize>,
    ) -> StoreResult<Vec<MemoryItem>> {
        debug!("Listing memory items updated after {timestamp}");

        let mut items: Vec<_> = self
            .list_memory_items(None, None)
            .await?
            .into_iter()
            .filter(|item| item.updated_at > timestamp)
            .collect();
        items.sort_by_key(|item| item.updated_at);
        if let Some(limit) = limit {
            items.truncate(limit);
        }
        Ok(items)
    }

    /// List memory items needing review.
    pub async fn list_memory_items_needing_review(
        &self,
        now: OffsetDateTime,
        limit: Option<usize>,
    ) -> StoreResult<Vec<MemoryItem>> {
        let items = self.list_memory_items(None, None).await?;
        let mut filtered: Vec<_> = items
            .into_iter()
            .filter(|item| item.needs_review_at(now))
            .collect();
        filtered.sort_by_key(|item| item.updated_at);
        if let Some(limit) = limit {
            filtered.truncate(limit);
        }
        Ok(filtered)
    }

    /// Save a knowledge commit.
    pub async fn save_knowledge_commit(&self, commit: &KnowledgeCommit) -> StoreResult<()> {
        reject_serialized_secret_material("knowledge commit", commit)?;
        debug!("Saving knowledge commit: {}", commit.id);

        let commit_json = to_json(commit)?;
        self.db
            .query(
                r#"
                UPSERT type::thing("knowledge_commit", $id) SET
                    commit = $commit,
                    parent_id = $parent_id,
                    session_id = $session_id,
                    writer_harness = $writer_harness,
                    message = $message,
                    created_at = $created_at
                "#,
            )
            .bind(("id", commit.id.to_string()))
            .bind(("commit", commit_json))
            .bind(("parent_id", commit.parent_id.map(|id| id.to_string())))
            .bind(("session_id", commit.session_id.map(|id| id.to_string())))
            .bind(("writer_harness", commit.writer.harness.to_string()))
            .bind(("message", commit.message.clone()))
            .bind(("created_at", format_rfc3339(commit.created_at)?))
            .await?
            .check()?;

        Ok(())
    }

    /// Get a knowledge commit by ID.
    pub async fn get_knowledge_commit(&self, id: &Id) -> StoreResult<Option<KnowledgeCommit>> {
        debug!("Getting knowledge commit: {id}");

        let mut result = self
            .db
            .query(
                r#"
                SELECT meta::id(id) AS record_id, commit
                FROM type::thing("knowledge_commit", $id)
                "#,
            )
            .bind(("id", id.to_string()))
            .await?
            .check()?;

        let records: Vec<KnowledgeCommitRecord> = result.take(0)?;
        records
            .into_iter()
            .next()
            .map(KnowledgeCommitRecord::into_knowledge_commit)
            .transpose()
    }

    /// List knowledge commits, newest first.
    pub async fn list_knowledge_commits(
        &self,
        limit: Option<usize>,
    ) -> StoreResult<Vec<KnowledgeCommit>> {
        let mut query = r#"
            SELECT meta::id(id) AS record_id, commit, created_at
            FROM knowledge_commit
            ORDER BY created_at DESC
        "#
        .to_string();
        if let Some(limit) = limit {
            query.push_str(&format!(" LIMIT {limit}"));
        }

        let mut result = self.db.query(query).await?.check()?;
        decode_knowledge_commits(result.take(0)?)
    }

    /// List knowledge commits created after a timestamp.
    pub async fn list_knowledge_commits_after(
        &self,
        timestamp: OffsetDateTime,
        limit: Option<usize>,
    ) -> StoreResult<Vec<KnowledgeCommit>> {
        let mut commits: Vec<_> = self
            .list_knowledge_commits(None)
            .await?
            .into_iter()
            .filter(|commit| commit.created_at > timestamp)
            .collect();
        commits.sort_by_key(|commit| commit.created_at);
        if let Some(limit) = limit {
            commits.truncate(limit);
        }
        Ok(commits)
    }

    /// Get the newest knowledge commit.
    pub async fn latest_knowledge_commit(&self) -> StoreResult<Option<KnowledgeCommit>> {
        Ok(self
            .list_knowledge_commits(Some(1))
            .await?
            .into_iter()
            .next())
    }

    /// Latest memory item update timestamp.
    pub async fn latest_memory_timestamp(&self) -> StoreResult<Option<OffsetDateTime>> {
        let mut result = self
            .db
            .query("SELECT updated_at FROM memory_item ORDER BY updated_at DESC LIMIT 1")
            .await?
            .check()?;

        let records: Vec<LatestMemoryTimestampRecord> = result.take(0)?;
        records
            .into_iter()
            .next()
            .map(|record| record.updated_at.to_offset_datetime())
            .transpose()
    }

    /// Latest knowledge commit timestamp.
    pub async fn latest_commit_timestamp(&self) -> StoreResult<Option<OffsetDateTime>> {
        let mut result = self
            .db
            .query("SELECT created_at FROM knowledge_commit ORDER BY created_at DESC LIMIT 1")
            .await?
            .check()?;

        let records: Vec<LatestCommitTimestampRecord> = result.take(0)?;
        records
            .into_iter()
            .next()
            .map(|record| record.created_at.to_offset_datetime())
            .transpose()
    }
}

fn reject_likely_secret_material(item: &MemoryItem) -> StoreResult<()> {
    for (field, value) in [
        ("title", item.title.as_str()),
        ("content", item.content.as_str()),
    ] {
        if let Some(secret_kind) = likely_secret_kind(value) {
            return Err(StoreError::Policy(format!(
                "memory item {field} contains a likely {secret_kind}; secret material was not persisted"
            )));
        }
    }
    for evidence in &item.evidence {
        for (field, value) in [
            ("evidence target", Some(evidence.target.as_str())),
            ("evidence summary", evidence.summary.as_deref()),
            ("evidence excerpt", evidence.excerpt.as_deref()),
        ] {
            if let Some(secret_kind) = value.and_then(likely_secret_kind) {
                return Err(StoreError::Policy(format!(
                    "memory item {field} contains a likely {secret_kind}; secret material was not persisted"
                )));
            }
        }
    }
    if let Some(procedure) = &item.procedure {
        let mut fields = vec![
            ("procedure task", procedure.task.as_str()),
            (
                "procedure verification command",
                procedure.verification.command.as_str(),
            ),
            (
                "procedure verification marker",
                procedure.verification.expected_output_contains.as_str(),
            ),
        ];
        fields.extend(
            procedure
                .commands
                .iter()
                .map(|value| ("procedure command", value.as_str())),
        );
        fields.extend(
            procedure
                .failure_signatures
                .iter()
                .map(|value| ("procedure failure signature", value.as_str())),
        );
        fields.extend(procedure.prerequisites.iter().flat_map(|prerequisite| {
            [
                ("procedure prerequisite key", prerequisite.key.as_str()),
                (
                    "procedure prerequisite value",
                    prerequisite.expected.as_str(),
                ),
            ]
        }));
        for (field, value) in fields {
            if let Some(secret_kind) = likely_secret_kind(value) {
                return Err(StoreError::Policy(format!(
                    "memory item {field} contains a likely {secret_kind}; secret material was not persisted"
                )));
            }
        }
    }
    reject_serialized_secret_material("memory item", item)
}

fn decode_memory_items(records: Vec<MemoryItemRecord>) -> StoreResult<Vec<MemoryItem>> {
    records
        .into_iter()
        .map(MemoryItemRecord::into_memory_item)
        .collect()
}

fn decode_knowledge_commits(
    records: Vec<KnowledgeCommitRecord>,
) -> StoreResult<Vec<KnowledgeCommit>> {
    records
        .into_iter()
        .map(KnowledgeCommitRecord::into_knowledge_commit)
        .collect()
}

fn format_rfc3339(value: OffsetDateTime) -> StoreResult<String> {
    value
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| StoreError::Deserialization(format!("Invalid timestamp: {e}")))
}

fn to_json<T: Serialize>(value: &T) -> StoreResult<serde_json::Value> {
    serde_json::to_value(value).map_err(StoreError::Serialization)
}

fn from_json<T: DeserializeOwned>(value: serde_json::Value) -> StoreResult<T> {
    serde_json::from_value(value).map_err(StoreError::Serialization)
}

fn decode_correction_projection_purge(
    record: CorrectionProjectionPurgeRecord,
) -> StoreResult<CorrectionProjectionPurge> {
    let parse_ids = |ids: Vec<String>, label: &str| {
        ids.into_iter()
            .map(|id| {
                Id::parse(&id).map_err(|error| {
                    StoreError::Deserialization(format!("invalid {label} ID: {error}"))
                })
            })
            .collect::<StoreResult<Vec<_>>>()
    };
    Ok(CorrectionProjectionPurge {
        deleted: record.deleted,
        proposal_ids: parse_ids(record.proposal_ids, "deleted correction proposal")?,
        pending_replacement_ids: parse_ids(
            record.pending_replacement_ids,
            "deleted proposal replacement",
        )?,
        unlocked_obsolete_ids: parse_ids(record.unlocked_obsolete_ids, "unlocked obsolete memory")?,
    })
}

fn scope_key(item: &MemoryItem) -> String {
    match &item.scope {
        engram_core::memory::MemoryScope::Global => "global".to_string(),
        engram_core::memory::MemoryScope::User => "user".to_string(),
        engram_core::memory::MemoryScope::Project { project_name, .. } => {
            format!("project:{project_name}")
        }
        engram_core::memory::MemoryScope::Task { task_name, .. } => format!("task:{task_name}"),
        engram_core::memory::MemoryScope::Entity { entity_name, .. } => {
            format!("entity:{entity_name}")
        }
        engram_core::memory::MemoryScope::Repository {
            remote_url,
            local_path,
            ..
        } => format!(
            "repository:{}",
            remote_url
                .as_deref()
                .or(local_path.as_deref())
                .unwrap_or("")
        ),
        engram_core::memory::MemoryScope::Session { session_id } => {
            format!("session:{session_id}")
        }
        engram_core::memory::MemoryScope::Custom { name } => format!("custom:{name}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engram_core::memory::{
        ClaimOrigin, CorrectionProposal, CorrectionProposalStatus, EvidenceKind, EvidenceRef,
        Harness, MemoryChange, MemoryChangeType, MemoryKind, MemoryScope, ModelIdentity,
        ProcedureCard, ProcedureVerification, WriterProvenance,
    };

    async fn setup_repo() -> MemoryRepo {
        let config = crate::StoreConfig::memory();
        let db = crate::connect_and_init(&config).await.unwrap();
        let repo = MemoryRepo::new(db);
        repo.init_schema().await.unwrap();
        repo
    }

    fn writer() -> WriterProvenance {
        WriterProvenance::agent(Harness::Codex, ModelIdentity::new("openai", "gpt-5.5"))
    }

    fn item(title: &str) -> MemoryItem {
        MemoryItem::new(
            MemoryKind::Decision,
            title,
            "Persist memory items as structured JSON plus query keys.",
            MemoryScope::project("engram"),
            ClaimOrigin::UserStated,
            writer(),
        )
        .with_evidence(EvidenceRef::new(EvidenceKind::ManualReview, "test"))
    }

    #[tokio::test]
    async fn save_and_get_memory_item_round_trips_rich_fields() {
        let repo = setup_repo().await;
        let item = item("Memory Store MVP").with_tag("memory-os");

        repo.save_memory_item(&item).await.unwrap();
        let retrieved = repo.get_memory_item(&item.id).await.unwrap().unwrap();

        assert_eq!(retrieved.id, item.id);
        assert_eq!(retrieved.title, "Memory Store MVP");
        assert_eq!(retrieved.kind, MemoryKind::Decision);
        assert_eq!(retrieved.status, MemoryStatus::Active);
        assert_eq!(retrieved.tags, vec!["memory-os"]);
        assert_eq!(retrieved.evidence.len(), 1);
    }

    #[tokio::test]
    async fn ordinary_store_writes_cannot_mint_correction_proposal_markers() {
        let repo = setup_repo().await;
        let mut forged_create = item("Forged proposal replacement");
        forged_create.correction_proposal_id = Some(Id::new());
        let error = repo.save_memory_item(&forged_create).await.unwrap_err();
        assert!(error.to_string().contains("dedicated atomic proposal"));
        assert!(repo
            .get_memory_item(&forged_create.id)
            .await
            .unwrap()
            .is_none());

        let existing = item("Ordinary existing memory");
        repo.save_memory_item(&existing).await.unwrap();
        let mut forged_update = existing.clone();
        forged_update.pending_correction_proposal_id = Some(Id::new());
        let error = repo
            .save_memory_item_if_unchanged(&existing, &forged_update)
            .await
            .unwrap_err();
        assert!(error.to_string().contains("dedicated atomic proposal"));
        let stored = repo.get_memory_item(&existing.id).await.unwrap().unwrap();
        assert!(stored.pending_correction_proposal_id.is_none());
    }

    #[tokio::test]
    async fn save_memory_item_surfaces_statement_level_database_errors() {
        let repo = setup_repo().await;
        repo.db
            .query("DEFINE FIELD OVERWRITE item ON TABLE memory_item TYPE object ASSERT false")
            .await
            .unwrap()
            .check()
            .unwrap();

        let error = repo
            .save_memory_item(&item("Statement failure"))
            .await
            .expect_err("a failed UPSERT must not be acknowledged as persisted");

        assert!(matches!(error, StoreError::Database(_)));
        assert!(repo.list_memory_items(None, None).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn save_memory_correction_is_atomic_when_obsolete_write_fails() {
        let repo = setup_repo().await;
        let obsolete = item("Obsolete decision");
        let replacement = MemoryItem::new(
            MemoryKind::Decision,
            "Replacement decision",
            "Use the corrected decision.",
            MemoryScope::project("engram"),
            ClaimOrigin::UserCorrected,
            writer(),
        )
        .with_evidence(EvidenceRef::new(EvidenceKind::File, "docs/correction.md"));
        repo.save_memory_item(&obsolete).await.unwrap();
        repo.save_memory_item(&replacement).await.unwrap();

        repo.db
            .query(
                "DEFINE FIELD OVERWRITE status_key ON TABLE memory_item TYPE string ASSERT $value != 'superseded'",
            )
            .await
            .unwrap()
            .check()
            .unwrap();

        let corrected_replacement = replacement
            .clone()
            .with_superseded_item(obsolete.id)
            .with_evidence(EvidenceRef::new(EvidenceKind::ToolCall, "memory.correct"));
        let corrected_obsolete = obsolete.clone().with_status(MemoryStatus::Superseded);
        repo.save_memory_correction(
            &replacement,
            &obsolete,
            &corrected_replacement,
            &corrected_obsolete,
        )
        .await
        .expect_err("a failed obsolete update must roll back the replacement update");

        let stored_replacement = repo
            .get_memory_item(&replacement.id)
            .await
            .unwrap()
            .unwrap();
        let stored_obsolete = repo.get_memory_item(&obsolete.id).await.unwrap().unwrap();
        assert!(stored_replacement.supersedes.is_empty());
        assert_eq!(stored_obsolete.status, MemoryStatus::Active);
    }

    #[tokio::test]
    async fn correction_proposal_create_and_apply_are_atomic() {
        let repo = setup_repo().await;
        let obsolete = item("Proposal target");
        repo.save_memory_item(&obsolete).await.unwrap();
        let mut replacement = MemoryItem::new(
            MemoryKind::Decision,
            "Proposed replacement",
            "Use the proposed replacement.",
            obsolete.scope.clone(),
            ClaimOrigin::AgentInferred,
            writer(),
        )
        .with_status(MemoryStatus::NeedsReview)
        .with_evidence(EvidenceRef::new(EvidenceKind::File, "docs/proposal.md"));
        let proposal = CorrectionProposal::new(
            obsolete.id,
            replacement.id,
            obsolete.kind.clone(),
            obsolete.scope.clone(),
            "a".repeat(64),
            writer(),
        );
        replacement.correction_proposal_id = Some(proposal.id);
        let mut locked_obsolete = obsolete.clone();
        locked_obsolete.pending_correction_proposal_id = Some(proposal.id);
        repo.save_correction_proposal(&proposal, &replacement, &obsolete, &locked_obsolete)
            .await
            .unwrap();
        assert_eq!(
            repo.get_correction_proposal(&proposal.id)
                .await
                .unwrap()
                .unwrap(),
            proposal
        );

        repo.db
            .query(
                "DEFINE FIELD OVERWRITE status_key ON TABLE correction_proposal TYPE string ASSERT $value != 'applied'",
            )
            .await
            .unwrap()
            .check()
            .unwrap();
        let applied_proposal = proposal.clone().with_applied();
        let mut active_replacement = replacement
            .clone()
            .with_status(MemoryStatus::Active)
            .with_superseded_item(obsolete.id);
        active_replacement.correction_proposal_id = None;
        let mut superseded_obsolete = locked_obsolete
            .clone()
            .with_status(MemoryStatus::Superseded);
        superseded_obsolete.pending_correction_proposal_id = None;

        let changed_replacement = replacement.clone().with_tag("concurrent-change");
        repo.save_memory_item(&changed_replacement)
            .await
            .expect_err("generic saves must reject proposal-bound replacements");
        repo.db
            .query(
                r#"UPDATE type::thing("memory_item", $id)
                    SET snapshot_digest = "concurrent-change""#,
            )
            .bind(("id", replacement.id.to_string()))
            .await
            .unwrap()
            .check()
            .unwrap();
        let error = repo
            .apply_correction_proposal(
                &proposal,
                &replacement,
                &locked_obsolete,
                &applied_proposal,
                &active_replacement,
                &superseded_obsolete,
            )
            .await
            .expect_err("a changed snapshot must reject the atomic apply");
        assert!(error.to_string().contains("failed transaction"), "{error}");
        assert!(repo
            .get_memory_item(&replacement.id)
            .await
            .unwrap()
            .unwrap()
            .tags
            .is_empty());
        assert_eq!(
            repo.get_correction_proposal(&proposal.id)
                .await
                .unwrap()
                .unwrap()
                .status,
            CorrectionProposalStatus::Pending
        );
        assert_eq!(
            repo.get_memory_item(&obsolete.id)
                .await
                .unwrap()
                .unwrap()
                .status,
            MemoryStatus::Active
        );
        repo.db
            .query(
                r#"UPDATE type::thing("memory_item", $id)
                    SET snapshot_digest = $snapshot_digest"#,
            )
            .bind(("id", replacement.id.to_string()))
            .bind(("snapshot_digest", snapshot_digest(&replacement).unwrap()))
            .await
            .unwrap()
            .check()
            .unwrap();

        repo.apply_correction_proposal(
            &proposal,
            &replacement,
            &locked_obsolete,
            &applied_proposal,
            &active_replacement,
            &superseded_obsolete,
        )
        .await
        .expect_err("proposal update failure must roll back both memory item updates");

        assert_eq!(
            repo.get_memory_item(&replacement.id)
                .await
                .unwrap()
                .unwrap()
                .status,
            MemoryStatus::NeedsReview
        );
        assert_eq!(
            repo.get_memory_item(&obsolete.id)
                .await
                .unwrap()
                .unwrap()
                .status,
            MemoryStatus::Active
        );
        assert_eq!(
            repo.get_correction_proposal(&proposal.id)
                .await
                .unwrap()
                .unwrap()
                .status,
            CorrectionProposalStatus::Pending
        );
    }

    #[tokio::test]
    async fn procedure_correction_verification_rolls_back_when_proposal_cas_fails() {
        let repo = setup_repo().await;
        let obsolete = MemoryItem::new(
            MemoryKind::Procedure,
            "Obsolete procedure",
            "Use the obsolete procedure.",
            MemoryScope::project("engram"),
            ClaimOrigin::UserStated,
            writer(),
        )
        .with_procedure(ProcedureCard::new(
            "run obsolete check",
            vec!["cargo check".to_string()],
            ProcedureVerification::new("cargo check", 0, "Finished"),
        ));
        repo.save_memory_item(&obsolete).await.unwrap();
        let mut replacement = MemoryItem::new(
            MemoryKind::Procedure,
            "Replacement procedure",
            "Use the replacement procedure.",
            obsolete.scope.clone(),
            ClaimOrigin::AgentInferred,
            writer(),
        )
        .with_status(MemoryStatus::NeedsReview)
        .with_procedure(ProcedureCard::new(
            "run replacement check",
            vec!["cargo test".to_string()],
            ProcedureVerification::new("cargo test", 0, "test result: ok"),
        ))
        .with_evidence(EvidenceRef::new(EvidenceKind::File, "docs/replacement.md"));
        let proposal = CorrectionProposal::new(
            obsolete.id,
            replacement.id,
            MemoryKind::Procedure,
            obsolete.scope.clone(),
            "a".repeat(64),
            writer(),
        );
        replacement.correction_proposal_id = Some(proposal.id);
        let mut locked_obsolete = obsolete.clone();
        locked_obsolete.pending_correction_proposal_id = Some(proposal.id);
        repo.save_correction_proposal(&proposal, &replacement, &obsolete, &locked_obsolete)
            .await
            .unwrap();

        let mut verified_replacement = replacement.clone();
        let procedure = verified_replacement.procedure.as_mut().unwrap();
        procedure.verification.evidence_path = Some("/tmp/replacement-receipt.json".to_string());
        procedure.verification.evidence_sha256 = Some("b".repeat(64));
        procedure.verification.verified_at = Some(OffsetDateTime::now_utc());
        procedure.expires_at = Some(OffsetDateTime::now_utc() + time::Duration::days(30));
        verified_replacement.updated_at = OffsetDateTime::now_utc();
        verified_replacement.evidence.push(EvidenceRef::new(
            EvidenceKind::File,
            "/tmp/replacement-receipt.json",
        ));
        let verified_proposal = proposal.clone().with_canonical_digest("c".repeat(64));

        repo.db
            .query(
                r#"UPDATE type::thing("correction_proposal", $id)
                    SET snapshot_digest = "concurrent-change""#,
            )
            .bind(("id", proposal.id.to_string()))
            .await
            .unwrap()
            .check()
            .unwrap();
        let error = repo
            .verify_correction_procedure(
                &proposal,
                &replacement,
                &locked_obsolete,
                &verified_proposal,
                &verified_replacement,
                &locked_obsolete,
            )
            .await
            .expect_err("a stale proposal snapshot must roll back both memory updates");
        assert!(error.to_string().contains("failed transaction"), "{error}");

        let stored_replacement = repo
            .get_memory_item(&replacement.id)
            .await
            .unwrap()
            .unwrap();
        assert!(stored_replacement
            .procedure
            .as_ref()
            .unwrap()
            .verification
            .evidence_sha256
            .is_none());
        assert_eq!(stored_replacement.status, MemoryStatus::NeedsReview);
        let stored_obsolete = repo.get_memory_item(&obsolete.id).await.unwrap().unwrap();
        assert_eq!(stored_obsolete.status, MemoryStatus::Active);
        assert_eq!(
            stored_obsolete.pending_correction_proposal_id,
            Some(proposal.id)
        );
        assert_eq!(
            repo.get_correction_proposal(&proposal.id)
                .await
                .unwrap()
                .unwrap()
                .canonical_digest,
            proposal.canonical_digest
        );
    }

    #[tokio::test]
    async fn delete_memory_item_removes_canonical_record() {
        let repo = setup_repo().await;
        let item = item("Forget me");
        repo.save_memory_item(&item).await.unwrap();

        assert!(repo.delete_memory_item(&item.id).await.unwrap());
        assert!(repo.get_memory_item(&item.id).await.unwrap().is_none());
        assert!(!repo.delete_memory_item(&item.id).await.unwrap());
    }

    #[tokio::test]
    async fn save_memory_item_rejects_secret_material_at_storage_boundary() {
        let repo = setup_repo().await;
        let canaries = [
            "PASSWORD=correct-horse-battery-staple",
            "Authorization: Bearer synthetic-secret-token",
            "https://agent:synthetic-password@example.test/path",
            "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.synthetic_signature",
        ];

        for canary in canaries {
            let mut candidate = item("Secret canary");
            candidate.content = canary.to_string();
            let error = repo
                .save_memory_item(&candidate)
                .await
                .expect_err("secret-bearing memory must be rejected");
            assert!(error
                .to_string()
                .contains("secret material was not persisted"));
            assert!(repo.get_memory_item(&candidate.id).await.unwrap().is_none());
        }
    }

    #[tokio::test]
    async fn save_memory_item_rejects_secret_material_in_procedure_fields() {
        let repo = setup_repo().await;
        let candidate = MemoryItem::new(
            MemoryKind::Procedure,
            "Unsafe procedure",
            "Candidate must remain outside durable storage.",
            MemoryScope::project("engram"),
            ClaimOrigin::AgentObserved,
            writer(),
        )
        .with_procedure(ProcedureCard::new(
            "authenticate and run tests",
            vec!["API_TOKEN=synthetic-canary cargo test".to_string()],
            ProcedureVerification::new("cargo test", 0, "test result: ok"),
        ));

        let error = repo
            .save_memory_item(&candidate)
            .await
            .expect_err("secret-bearing procedure must be rejected");
        assert!(error
            .to_string()
            .contains("secret material was not persisted"));
        assert!(repo.get_memory_item(&candidate.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn save_memory_item_rejects_secret_material_in_serialized_metadata() {
        let repo = setup_repo().await;
        let candidate = item("Metadata canary").with_tag("API_TOKEN=synthetic-metadata-secret");

        let error = repo
            .save_memory_item(&candidate)
            .await
            .expect_err("secret-bearing metadata must be rejected");
        assert!(error
            .to_string()
            .contains("secret material was not persisted"));
        assert!(repo.get_memory_item(&candidate.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn save_knowledge_commit_rejects_secret_material_at_storage_boundary() {
        let repo = setup_repo().await;
        let commit = KnowledgeCommit::new(writer(), "Capture safe memory change").with_change(
            MemoryChange::new(
                MemoryChangeType::Added,
                "Safe title",
                "Authorization: Bearer synthetic-commit-secret",
            ),
        );

        let error = repo
            .save_knowledge_commit(&commit)
            .await
            .expect_err("secret-bearing commit must be rejected");
        assert!(error
            .to_string()
            .contains("secret material was not persisted"));
        assert!(repo
            .get_knowledge_commit(&commit.id)
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn list_memory_items_filters_by_status() {
        let repo = setup_repo().await;
        let active = item("Active memory");
        let review = item("Review memory").with_status(MemoryStatus::NeedsReview);

        repo.save_memory_item(&active).await.unwrap();
        repo.save_memory_item(&review).await.unwrap();

        let active_items = repo
            .list_memory_items(Some(MemoryStatus::Active), None)
            .await
            .unwrap();
        assert_eq!(active_items.len(), 1);
        assert_eq!(active_items[0].id, active.id);
    }

    #[tokio::test]
    async fn list_memory_items_updated_after_cursor_timestamp() {
        let repo = setup_repo().await;
        let before = OffsetDateTime::now_utc();
        let item = item("Changed after cursor");

        repo.save_memory_item(&item).await.unwrap();

        let changed = repo
            .list_memory_items_updated_after(before, None)
            .await
            .unwrap();
        assert_eq!(changed.len(), 1);
        assert_eq!(changed[0].id, item.id);
    }

    #[tokio::test]
    async fn save_and_list_knowledge_commits() {
        let repo = setup_repo().await;
        let item = item("Committed memory");
        let commit = KnowledgeCommit::new(writer(), "Capture committed memory").with_change(
            MemoryChange::new(
                MemoryChangeType::Added,
                "Committed memory",
                "Added a memory item.",
            )
            .with_item(item.id),
        );

        repo.save_knowledge_commit(&commit).await.unwrap();

        let retrieved = repo
            .get_knowledge_commit(&commit.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(retrieved.id, commit.id);
        assert_eq!(retrieved.change_count(), 1);

        let latest = repo.latest_knowledge_commit().await.unwrap().unwrap();
        assert_eq!(latest.id, commit.id);
    }
}
