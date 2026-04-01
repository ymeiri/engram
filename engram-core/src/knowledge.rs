//! Document intelligence types (Layer 6).
//!
//! Canonical document resolution for managing documentation clutter.
//! Philosophy: Git is Source of Truth, PKAS is Intelligence Layer.

use crate::id::Id;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Type of knowledge document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocType {
    /// Architecture Decision Record.
    Adr,
    /// Runbook / operational guide.
    Runbook,
    /// How-to guide.
    Howto,
    /// Research document.
    Research,
    /// Design document.
    Design,
    /// README file.
    Readme,
    /// Changelog.
    Changelog,
    /// Custom type.
    Custom(String),
}

impl std::fmt::Display for DocType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Adr => write!(f, "adr"),
            Self::Runbook => write!(f, "runbook"),
            Self::Howto => write!(f, "howto"),
            Self::Research => write!(f, "research"),
            Self::Design => write!(f, "design"),
            Self::Readme => write!(f, "readme"),
            Self::Changelog => write!(f, "changelog"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl DocType {
    /// Parse from string.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "adr" => Self::Adr,
            "runbook" => Self::Runbook,
            "howto" => Self::Howto,
            "research" => Self::Research,
            "design" => Self::Design,
            "readme" => Self::Readme,
            "changelog" => Self::Changelog,
            other => Self::Custom(other.to_string()),
        }
    }
}

/// Status of a knowledge document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DocStatus {
    /// Document is active and current.
    #[default]
    Active,
    /// Document is archived (historical).
    Archived,
    /// Document has been superseded by another.
    Superseded,
    /// Document is a draft.
    Draft,
}

/// A knowledge document in the canonical registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeDoc {
    /// Unique identifier.
    pub id: Id,

    /// Human-friendly name.
    pub name: String,

    /// Git path (if file exists).
    pub canonical_path: Option<String>,

    /// Document type.
    pub doc_type: DocType,

    /// Document status.
    pub status: DocStatus,

    /// Owner (for governance).
    pub owner: Option<String>,

    /// Last review date.
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_reviewed: Option<OffsetDateTime>,

    /// Content hash for sync detection.
    pub content_hash: String,

    /// Tags for categorization.
    #[serde(default)]
    pub tags: Vec<String>,

    /// Document content (indexed for search).
    pub content: String,

    /// LLM-generated summary.
    pub summary: Option<String>,

    /// Creation timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,

    /// Last update timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl KnowledgeDoc {
    /// Create a new knowledge document.
    #[must_use]
    pub fn new(name: impl Into<String>, doc_type: DocType, content: impl Into<String>) -> Self {
        let content = content.into();
        let content_hash = Self::compute_hash(&content);
        let now = OffsetDateTime::now_utc();

        Self {
            id: Id::new(),
            name: name.into(),
            canonical_path: None,
            doc_type,
            status: DocStatus::Active,
            owner: None,
            last_reviewed: None,
            content_hash,
            tags: Vec::new(),
            content,
            summary: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Compute content hash.
    fn compute_hash(content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Set the canonical path.
    #[must_use]
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.canonical_path = Some(path.into());
        self
    }

    /// Set the owner.
    #[must_use]
    pub fn with_owner(mut self, owner: impl Into<String>) -> Self {
        self.owner = Some(owner.into());
        self
    }

    /// Add tags.
    #[must_use]
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Check if content has changed.
    #[must_use]
    pub fn content_changed(&self, new_content: &str) -> bool {
        Self::compute_hash(new_content) != self.content_hash
    }

    /// Update content.
    pub fn update_content(&mut self, content: String) {
        self.content_hash = Self::compute_hash(&content);
        self.content = content;
        self.updated_at = OffsetDateTime::now_utc();
    }

    /// Mark as reviewed.
    pub fn mark_reviewed(&mut self) {
        self.last_reviewed = Some(OffsetDateTime::now_utc());
    }

    /// Check if document needs review (older than given days).
    #[must_use]
    pub fn needs_review(&self, max_days: i64) -> bool {
        match self.last_reviewed {
            None => true,
            Some(reviewed) => {
                let now = OffsetDateTime::now_utc();
                (now - reviewed).whole_days() >= max_days
            }
        }
    }
}

/// File sync status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    /// File is synced with PKAS.
    #[default]
    Synced,
    /// File changed in Git, needs re-sync.
    Stale,
    /// File was deleted.
    Deleted,
    /// Conflict between Git and PKAS.
    Conflict,
    /// New file not yet in PKAS.
    New,
}

/// File synchronization state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSync {
    /// Unique identifier.
    pub id: Id,

    /// File path (absolute or repo-relative).
    pub path: String,

    /// Repository identifier.
    pub repo: String,

    /// Linked knowledge document (if mapped).
    pub doc_id: Option<Id>,

    /// Content hash at last sync.
    pub last_hash: String,

    /// Last modification time.
    #[serde(with = "time::serde::rfc3339")]
    pub last_modified: OffsetDateTime,

    /// Last sync time.
    #[serde(with = "time::serde::rfc3339")]
    pub last_synced: OffsetDateTime,

    /// Sync status.
    pub sync_status: SyncStatus,

    /// When the file was deleted (if deleted).
    #[serde(with = "time::serde::rfc3339::option")]
    pub deleted_at: Option<OffsetDateTime>,
}

impl FileSync {
    /// Create a new file sync record.
    #[must_use]
    pub fn new(path: impl Into<String>, repo: impl Into<String>, hash: impl Into<String>) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: Id::new(),
            path: path.into(),
            repo: repo.into(),
            doc_id: None,
            last_hash: hash.into(),
            last_modified: now,
            last_synced: now,
            sync_status: SyncStatus::Synced,
            deleted_at: None,
        }
    }

    /// Link to a knowledge document.
    #[must_use]
    pub fn with_doc(mut self, doc_id: Id) -> Self {
        self.doc_id = Some(doc_id);
        self
    }

    /// Mark as deleted.
    pub fn mark_deleted(&mut self) {
        self.sync_status = SyncStatus::Deleted;
        self.deleted_at = Some(OffsetDateTime::now_utc());
    }

    /// Mark as stale.
    pub fn mark_stale(&mut self) {
        self.sync_status = SyncStatus::Stale;
    }
}

/// Type of document event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocEventType {
    /// Document was created.
    Created,
    /// Documents were merged.
    Merged,
    /// Document was superseded.
    Superseded,
    /// Document was archived.
    Archived,
    /// Document was restored.
    Restored,
    /// Alias was added.
    AliasAdded,
    /// Owner was changed.
    OwnerChanged,
    /// Document was reviewed.
    Reviewed,
}

/// An event in the document lifecycle (for provenance).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocEvent {
    /// Unique identifier.
    pub id: Id,

    /// Document ID.
    pub doc_id: Id,

    /// Event type.
    pub event_type: DocEventType,

    /// Event details.
    pub details: serde_json::Value,

    /// Actor (who/what triggered this).
    pub actor: String,

    /// Timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub occurred_at: OffsetDateTime,
}

impl DocEvent {
    /// Create a new document event.
    #[must_use]
    pub fn new(doc_id: Id, event_type: DocEventType, actor: impl Into<String>) -> Self {
        Self {
            id: Id::new(),
            doc_id,
            event_type,
            details: serde_json::Value::Null,
            actor: actor.into(),
            occurred_at: OffsetDateTime::now_utc(),
        }
    }

    /// Set event details.
    #[must_use]
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = details;
        self
    }
}

/// Alias for a knowledge document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocAlias {
    /// The alias text.
    pub alias: String,

    /// The canonical document ID.
    pub doc_id: Id,
}

impl DocAlias {
    /// Create a new document alias.
    #[must_use]
    pub fn new(alias: impl Into<String>, doc_id: Id) -> Self {
        Self {
            alias: alias.into(),
            doc_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_doc_creation() {
        let doc = KnowledgeDoc::new(
            "RAPID Production Setup",
            DocType::Howto,
            "# Setup Guide\n...",
        )
        .with_path("docs/rapid/setup.md")
        .with_owner("platform-team");

        assert_eq!(doc.name, "RAPID Production Setup");
        assert_eq!(doc.doc_type, DocType::Howto);
        assert!(doc.canonical_path.is_some());
        assert!(doc.owner.is_some());
    }

    #[test]
    fn test_content_change_detection() {
        let doc = KnowledgeDoc::new("Test", DocType::Readme, "Original content");

        assert!(!doc.content_changed("Original content"));
        assert!(doc.content_changed("Modified content"));
    }

    #[test]
    fn test_file_sync_creation() {
        let sync = FileSync::new("docs/readme.md", "my-repo", "abc123");

        assert_eq!(sync.sync_status, SyncStatus::Synced);
        assert!(sync.deleted_at.is_none());
    }

    #[test]
    fn test_file_sync_deletion() {
        let mut sync = FileSync::new("docs/readme.md", "my-repo", "abc123");
        sync.mark_deleted();

        assert_eq!(sync.sync_status, SyncStatus::Deleted);
        assert!(sync.deleted_at.is_some());
    }
}
