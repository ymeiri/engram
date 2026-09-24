//! Sanitized, read-only structural audit surface for strict native successor documents.
//!
//! This module deliberately exposes no preparation, execution, recovery, repair, or run API.

use crate::native_document::inspect_successor_native_document;
use crate::EvalError;
use serde::Serialize;
use std::path::Path;

pub use crate::native_document::{NativeSuccessorDocumentKind, NativeSuccessorFamily};

/// Stable, non-sensitive reason that a structural document audit failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeSuccessorStructuralFailure {
    Unreadable,
    UnsafePathOrIdentity,
    MalformedJson,
    InvalidEnvelope,
    UnsupportedFamily,
    UnsupportedVersion,
    UnsupportedDocumentKind,
}

/// Sanitized structural evidence. It deliberately omits the path and all payload content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeSuccessorDocumentAudit {
    pub structurally_valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<NativeSuccessorFamily>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_kind: Option<NativeSuccessorDocumentKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure: Option<NativeSuccessorStructuralFailure>,
}

/// Inspect only the exact successor envelope and discriminator boundary.
///
/// This is not whole-protocol validation: closed-world payload schemas are intentionally deferred
/// until each successor protocol is frozen.
pub fn audit_native_successor_document_structure(path: &Path) -> NativeSuccessorDocumentAudit {
    match inspect_successor_native_document(path) {
        Ok(structure) => NativeSuccessorDocumentAudit {
            structurally_valid: true,
            family: Some(structure.family),
            schema_version: Some(structure.schema_version),
            document_kind: Some(structure.document_kind),
            document_sha256: Some(structure.sha256),
            document_bytes: Some(structure.byte_len),
            failure: None,
        },
        Err(error) => NativeSuccessorDocumentAudit {
            structurally_valid: false,
            family: None,
            schema_version: None,
            document_kind: None,
            document_sha256: None,
            document_bytes: None,
            failure: Some(classify_structural_failure(&error)),
        },
    }
}

fn classify_structural_failure(error: &EvalError) -> NativeSuccessorStructuralFailure {
    match error {
        EvalError::Io(_) => NativeSuccessorStructuralFailure::Unreadable,
        EvalError::Json(_) => NativeSuccessorStructuralFailure::MalformedJson,
        EvalError::Invalid(message)
            if message.contains("path")
                || message.contains("identity")
                || message.contains("owner-controlled")
                || message.contains("single-link")
                || message.contains("symbolic-link")
                || message.contains("writable by group or other")
                || message.contains("extended ACL") =>
        {
            NativeSuccessorStructuralFailure::UnsafePathOrIdentity
        }
        EvalError::Invalid(message) if message.contains("family is not accepted") => {
            NativeSuccessorStructuralFailure::UnsupportedFamily
        }
        EvalError::Invalid(message) if message.contains("schema_version must be exactly") => {
            NativeSuccessorStructuralFailure::UnsupportedVersion
        }
        EvalError::Invalid(message) if message.contains("document_kind") => {
            NativeSuccessorStructuralFailure::UnsupportedDocumentKind
        }
        EvalError::Invalid(_) => NativeSuccessorStructuralFailure::InvalidEnvelope,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::collections::BTreeSet;
    use std::fs;

    fn canonical_file(value: Value) -> (tempfile::TempDir, std::path::PathBuf) {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().canonicalize().unwrap().join("successor.json");
        fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        (root, path)
    }

    #[test]
    fn public_audit_contains_only_sanitized_structural_fields() {
        let secret_marker = "PRIVATE-CANARY-MUST-NOT-ESCAPE";
        let (_root, path) = canonical_file(serde_json::json!({
            "family": "native_correction_v1",
            "schema_version": 1,
            "payload": {
                "document_kind": "run_plan",
                "token": secret_marker,
                "argv": ["/private/provider"],
                "environment": {"SECRET": secret_marker}
            }
        }));
        let audit = audit_native_successor_document_structure(&path);
        assert!(audit.structurally_valid);
        let value = serde_json::to_value(&audit).unwrap();
        let object = value.as_object().unwrap();
        assert_eq!(
            object.keys().map(String::as_str).collect::<BTreeSet<_>>(),
            BTreeSet::from([
                "document_bytes",
                "document_kind",
                "document_sha256",
                "family",
                "schema_version",
                "structurally_valid",
            ])
        );
        let serialized = serde_json::to_string(&value).unwrap();
        for forbidden in [
            secret_marker,
            "token",
            "argv",
            "environment",
            "/private/provider",
            "process_group_id",
            "execution_seal",
            "vm_witness",
        ] {
            assert!(!serialized.contains(forbidden), "{forbidden}");
        }
    }

    #[test]
    fn public_failure_is_categorical_and_does_not_echo_untrusted_markers() {
        let marker = "UNTRUSTED-FAMILY-CANARY";
        let (_root, path) = canonical_file(serde_json::json!({
            "family": marker,
            "schema_version": 1,
            "payload": {"document_kind": "protocol"}
        }));
        let audit = audit_native_successor_document_structure(&path);
        assert!(!audit.structurally_valid);
        assert_eq!(
            audit.failure,
            Some(NativeSuccessorStructuralFailure::UnsupportedFamily)
        );
        assert!(!serde_json::to_string(&audit).unwrap().contains(marker));
    }

    #[cfg(unix)]
    #[test]
    fn public_audit_classifies_unsafe_permissions_without_exposing_the_path() {
        use std::os::unix::fs::PermissionsExt;

        let (_root, path) = canonical_file(serde_json::json!({
            "family": "native_correction_v1",
            "schema_version": 1,
            "payload": {"document_kind": "protocol"}
        }));
        fs::set_permissions(&path, fs::Permissions::from_mode(0o666)).unwrap();
        let audit = audit_native_successor_document_structure(&path);
        assert!(!audit.structurally_valid);
        assert_eq!(
            audit.failure,
            Some(NativeSuccessorStructuralFailure::UnsafePathOrIdentity)
        );
        assert!(!serde_json::to_string(&audit)
            .unwrap()
            .contains(path.to_str().unwrap()));
    }
}
