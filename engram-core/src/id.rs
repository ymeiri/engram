//! ID types for engram entities.
//!
//! We use UUIDv7 for all IDs, which provides:
//! - Temporal ordering (timestamp-based)
//! - Uniqueness without coordination
//! - Sortable by creation time

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// A unique identifier for engram entities.
///
/// Uses UUIDv7 internally for temporal ordering and uniqueness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Id(Uuid);

impl Id {
    /// Generate a new unique ID.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Create an ID from a UUID.
    #[must_use]
    pub const fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID.
    #[must_use]
    pub const fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Parse an ID from a string.
    pub fn parse(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl Default for Id {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for Id {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<Id> for Uuid {
    fn from(id: Id) -> Self {
        id.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_generation() {
        let id1 = Id::new();
        let id2 = Id::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_id_display() {
        let id = Id::new();
        let s = id.to_string();
        assert_eq!(s.len(), 36); // UUID format: 8-4-4-4-12
    }

    #[test]
    fn test_id_parse() {
        let id = Id::new();
        let s = id.to_string();
        let parsed = Id::parse(&s).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_id_serialization() {
        let id = Id::new();
        let json = serde_json::to_string(&id).unwrap();
        let parsed: Id = serde_json::from_str(&json).unwrap();
        assert_eq!(id, parsed);
    }
}
