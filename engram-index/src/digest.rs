//! Digest source inventory for Memory OS.
//!
//! Daily email, Slack, AI, SWE, and notes digests can be valuable memory
//! evidence, but they are sensitive and noisy. This module only inventories
//! digest-like source files and classifies operational artifacts. It does not
//! read digest contents or promote facts into active memory.

use crate::error::{IndexError, IndexResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use time::OffsetDateTime;

/// Stateless service for digest source discovery.
#[derive(Debug, Default, Clone)]
pub struct DigestService;

impl DigestService {
    /// Create a digest service.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Inventory digest-like files under a root path.
    ///
    /// This is intentionally read-light: only filesystem metadata and path names
    /// are inspected. File contents are not read.
    pub fn inventory(&self, options: DigestInventoryOptions) -> IndexResult<DigestInventory> {
        inventory_digest_sources(options)
    }
}

/// Options for digest source inventory.
#[derive(Debug, Clone)]
pub struct DigestInventoryOptions {
    /// Root directory to scan.
    pub root_path: PathBuf,
    /// Maximum included candidates to return.
    pub limit: Option<usize>,
    /// Include files that are normally treated as operational artifacts.
    pub include_operational: bool,
}

impl DigestInventoryOptions {
    /// Create inventory options.
    #[must_use]
    pub fn new(root_path: impl Into<PathBuf>) -> Self {
        Self {
            root_path: root_path.into(),
            limit: None,
            include_operational: false,
        }
    }
}

/// Digest source inventory result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestInventory {
    /// Inventory timestamp.
    #[serde(with = "time::serde::rfc3339")]
    pub generated_at: OffsetDateTime,
    /// Root path scanned.
    pub root_path: String,
    /// Files encountered under digest-like paths.
    pub files_scanned: usize,
    /// Total candidate digest files found before truncation.
    pub total_candidates: usize,
    /// Returned candidate count after limit.
    pub returned_candidates: usize,
    /// Whether candidate output was truncated by limit.
    pub truncated: bool,
    /// Excluded file count.
    pub excluded_count: usize,
    /// Candidate counts by digest source kind.
    pub by_source_kind: BTreeMap<String, usize>,
    /// Candidate counts by file format.
    pub by_format: BTreeMap<String, usize>,
    /// Candidate counts by collection directory.
    pub by_collection: BTreeMap<String, usize>,
    /// Candidate digest files.
    pub candidates: Vec<DigestSourceCandidate>,
    /// Excluded digest-adjacent files.
    pub exclusions: Vec<DigestExcludedPath>,
    /// Operator warnings.
    pub warnings: Vec<String>,
}

/// Candidate digest file that can later become review-gated source evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestSourceCandidate {
    /// Source kind inferred from path.
    pub source_kind: DigestSourceKind,
    /// Top-level digest collection directory, such as slack-digest.
    pub collection: String,
    /// Bucket within the collection, such as morning or ai-radar.
    pub bucket: Option<String>,
    /// File format.
    pub format: DigestFileFormat,
    /// Relative path from inventory root.
    pub relative_path: String,
    /// Absolute path.
    pub absolute_path: String,
    /// File name.
    pub file_name: String,
    /// Date or date range inferred from the path, when present.
    pub date_hint: Option<String>,
    /// File size from metadata.
    pub size_bytes: u64,
    /// Last modified timestamp from metadata, when available.
    #[serde(with = "time::serde::rfc3339::option")]
    pub modified_at: Option<OffsetDateTime>,
    /// Sensitivity classification.
    pub sensitivity: DigestSensitivity,
    /// Recommended next action.
    pub proposed_action: DigestProposedAction,
    /// Classification reasons.
    pub reasons: Vec<String>,
}

/// Digest-adjacent file excluded from source candidates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestExcludedPath {
    /// Relative path from inventory root.
    pub relative_path: String,
    /// Absolute path.
    pub absolute_path: String,
    /// Exclusion reason.
    pub reason: String,
}

/// Inferred digest source kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DigestSourceKind {
    /// Slack digest.
    Slack,
    /// Email digest.
    Email,
    /// AI/news digest.
    Ai,
    /// Software engineering digest.
    Swe,
    /// Notes digest.
    Notes,
    /// Unknown digest source.
    Unknown,
}

impl std::fmt::Display for DigestSourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Slack => write!(f, "slack"),
            Self::Email => write!(f, "email"),
            Self::Ai => write!(f, "ai"),
            Self::Swe => write!(f, "swe"),
            Self::Notes => write!(f, "notes"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// Supported digest file format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DigestFileFormat {
    /// Markdown digest.
    Markdown,
    /// HTML digest.
    Html,
}

impl std::fmt::Display for DigestFileFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Markdown => write!(f, "markdown"),
            Self::Html => write!(f, "html"),
        }
    }
}

/// Sensitivity classification for source material.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DigestSensitivity {
    /// Personal or workplace communication digest.
    SensitiveCommunication,
    /// General notes/news digest.
    PersonalNotes,
}

impl std::fmt::Display for DigestSensitivity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SensitiveCommunication => write!(f, "sensitive_communication"),
            Self::PersonalNotes => write!(f, "personal_notes"),
        }
    }
}

/// Recommended action for a candidate source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DigestProposedAction {
    /// Keep as evidence/index source; do not promote directly into memory.
    IndexAsSource,
    /// Generate candidate memories that need review before activation.
    ReviewGatedExtraction,
}

impl std::fmt::Display for DigestProposedAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IndexAsSource => write!(f, "index_as_source"),
            Self::ReviewGatedExtraction => write!(f, "review_gated_extraction"),
        }
    }
}

fn inventory_digest_sources(options: DigestInventoryOptions) -> IndexResult<DigestInventory> {
    if !options.root_path.exists() {
        return Err(IndexError::FileNotFound(
            options.root_path.display().to_string(),
        ));
    }
    if !options.root_path.is_dir() {
        return Err(IndexError::InvalidState(format!(
            "digest inventory root is not a directory: {}",
            options.root_path.display()
        )));
    }

    let root = fs::canonicalize(&options.root_path)?;
    let mut all_candidates = Vec::new();
    let mut exclusions = Vec::new();
    let mut files_scanned = 0;
    scan_directory(
        &root,
        &root,
        options.include_operational,
        &mut files_scanned,
        &mut all_candidates,
        &mut exclusions,
    )?;

    all_candidates.sort_by(|left, right| {
        left.relative_path
            .cmp(&right.relative_path)
            .then_with(|| left.absolute_path.cmp(&right.absolute_path))
    });
    exclusions.sort_by(|left, right| {
        left.relative_path
            .cmp(&right.relative_path)
            .then_with(|| left.absolute_path.cmp(&right.absolute_path))
    });

    let total_candidates = all_candidates.len();
    let limit = options.limit.unwrap_or(total_candidates);
    let truncated = total_candidates > limit;
    let candidates = all_candidates.into_iter().take(limit).collect::<Vec<_>>();
    let by_source_kind = count_by(
        candidates
            .iter()
            .map(|candidate| candidate.source_kind.to_string()),
    );
    let by_format = count_by(
        candidates
            .iter()
            .map(|candidate| candidate.format.to_string()),
    );
    let by_collection = count_by(
        candidates
            .iter()
            .map(|candidate| candidate.collection.clone()),
    );

    Ok(DigestInventory {
        generated_at: OffsetDateTime::now_utc(),
        root_path: root.display().to_string(),
        files_scanned,
        total_candidates,
        returned_candidates: candidates.len(),
        truncated,
        excluded_count: exclusions.len(),
        by_source_kind,
        by_format,
        by_collection,
        candidates,
        exclusions,
        warnings: vec![
            "Inventory only: no digest contents were read and no Memory OS records were written."
                .to_string(),
            "Digest-derived memories should be generated as review-gated candidates with source evidence."
                .to_string(),
        ],
    })
}

fn scan_directory(
    root: &Path,
    current: &Path,
    include_operational: bool,
    files_scanned: &mut usize,
    candidates: &mut Vec<DigestSourceCandidate>,
    exclusions: &mut Vec<DigestExcludedPath>,
) -> IndexResult<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if file_type.is_dir() {
            if !include_operational && is_operational_directory(&name) {
                if path_has_digest_component(root, &path) {
                    exclusions.push(exclusion(root, &path, "operational directory"));
                }
                continue;
            }
            scan_directory(
                root,
                &path,
                include_operational,
                files_scanned,
                candidates,
                exclusions,
            )?;
        } else if file_type.is_file() && path_has_digest_component(root, &path) {
            *files_scanned += 1;
            match classify_file(root, &path, include_operational)? {
                FileClassification::Candidate(candidate) => candidates.push(candidate),
                FileClassification::Excluded(excluded) => exclusions.push(excluded),
            }
        }
    }
    Ok(())
}

enum FileClassification {
    Candidate(DigestSourceCandidate),
    Excluded(DigestExcludedPath),
}

fn classify_file(
    root: &Path,
    path: &Path,
    include_operational: bool,
) -> IndexResult<FileClassification> {
    let relative_path = relative_path(root, path);
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string();

    if !include_operational && is_operational_file(&file_name) {
        return Ok(FileClassification::Excluded(exclusion(
            root,
            path,
            "operational artifact",
        )));
    }

    let Some(format) = digest_file_format(path) else {
        return Ok(FileClassification::Excluded(exclusion(
            root,
            path,
            "unsupported digest-adjacent file format",
        )));
    };

    let metadata = fs::metadata(path)?;
    let modified_at = metadata.modified().ok().map(OffsetDateTime::from);
    let collection = digest_collection(root, path).unwrap_or_else(|| "digest".to_string());
    let source_kind = source_kind_from_collection(&collection);
    let bucket = digest_bucket(root, path, &collection);
    let sensitivity = sensitivity_for(source_kind);
    let proposed_action = match sensitivity {
        DigestSensitivity::SensitiveCommunication => DigestProposedAction::ReviewGatedExtraction,
        DigestSensitivity::PersonalNotes => DigestProposedAction::IndexAsSource,
    };
    let mut reasons = vec![
        "Path is under a digest-named collection.".to_string(),
        "Supported source format for future safe parsing.".to_string(),
        "Inventory does not read file contents.".to_string(),
    ];
    if sensitivity == DigestSensitivity::SensitiveCommunication {
        reasons.push(
            "Communication digests must remain review-gated before becoming active memory."
                .to_string(),
        );
    }

    Ok(FileClassification::Candidate(DigestSourceCandidate {
        source_kind,
        collection,
        bucket,
        format,
        relative_path: relative_path.clone(),
        absolute_path: path.display().to_string(),
        file_name,
        date_hint: extract_date_hint(&relative_path),
        size_bytes: metadata.len(),
        modified_at,
        sensitivity,
        proposed_action,
        reasons,
    }))
}

fn digest_file_format(path: &Path) -> Option<DigestFileFormat> {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_lowercase())
        .as_deref()
    {
        Some("md") | Some("markdown") => Some(DigestFileFormat::Markdown),
        Some("html") | Some("htm") => Some(DigestFileFormat::Html),
        _ => None,
    }
}

fn path_has_digest_component(root: &Path, path: &Path) -> bool {
    if root_digest_collection(root).is_some() {
        return true;
    }

    path.strip_prefix(root)
        .ok()
        .into_iter()
        .flat_map(Path::components)
        .any(|component| {
            component
                .as_os_str()
                .to_string_lossy()
                .to_lowercase()
                .contains("digest")
        })
}

fn digest_collection(root: &Path, path: &Path) -> Option<String> {
    let relative_collection = path
        .strip_prefix(root)
        .ok()?
        .components()
        .find_map(|component| {
            let value = component.as_os_str().to_string_lossy();
            if value.to_lowercase().contains("digest") {
                Some(value.to_string())
            } else {
                None
            }
        });

    relative_collection.or_else(|| root_digest_collection(root))
}

fn root_digest_collection(root: &Path) -> Option<String> {
    let name = root.file_name()?.to_string_lossy();
    if name.to_lowercase().contains("digest") {
        Some(name.to_string())
    } else {
        None
    }
}

fn digest_bucket(root: &Path, path: &Path, collection: &str) -> Option<String> {
    let relative = path.strip_prefix(root).ok()?;
    if root_digest_collection(root).as_deref() == Some(collection) {
        let parent = relative.parent()?;
        if parent.as_os_str().is_empty() {
            return None;
        }
        return Some(
            parent
                .components()
                .map(|component| component.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/"),
        );
    }

    let mut after_collection = false;
    let mut components = Vec::new();
    for component in relative.components() {
        let value = component.as_os_str().to_string_lossy();
        if !after_collection {
            after_collection = value == collection;
            continue;
        }
        components.push(value.to_string());
    }

    components.pop();
    if components.is_empty() {
        None
    } else {
        Some(components.join("/"))
    }
}

fn source_kind_from_collection(collection: &str) -> DigestSourceKind {
    let lower = collection.to_lowercase();
    if lower.contains("slack") {
        DigestSourceKind::Slack
    } else if lower.contains("mail") || lower.contains("email") {
        DigestSourceKind::Email
    } else if lower.contains("swe") {
        DigestSourceKind::Swe
    } else if lower.contains("ai") {
        DigestSourceKind::Ai
    } else if lower.contains("note") {
        DigestSourceKind::Notes
    } else {
        DigestSourceKind::Unknown
    }
}

fn sensitivity_for(source_kind: DigestSourceKind) -> DigestSensitivity {
    match source_kind {
        DigestSourceKind::Slack | DigestSourceKind::Email => {
            DigestSensitivity::SensitiveCommunication
        }
        DigestSourceKind::Ai
        | DigestSourceKind::Swe
        | DigestSourceKind::Notes
        | DigestSourceKind::Unknown => DigestSensitivity::PersonalNotes,
    }
}

fn is_operational_directory(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.starts_with('.') || lower == "node_modules" || lower == "target"
}

fn is_operational_file(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.starts_with('.')
        || lower == "_seen.json"
        || lower == "_queue.json"
        || lower == "skill.md"
        || lower == "settings.local.json"
        || lower.ends_with(".log")
        || lower.ends_with(".yml")
        || lower.ends_with(".yaml")
        || lower.ends_with(".json")
}

fn exclusion(root: &Path, path: &Path, reason: impl Into<String>) -> DigestExcludedPath {
    DigestExcludedPath {
        relative_path: relative_path(root, path),
        absolute_path: path.display().to_string(),
        reason: reason.into(),
    }
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn extract_date_hint(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut dates = Vec::new();
    for idx in 0..bytes.len().saturating_sub(9) {
        let candidate = &value[idx..idx + 10];
        if is_iso_date(candidate) {
            dates.push(candidate.to_string());
        }
    }

    match dates.as_slice() {
        [] => None,
        [one] => Some(one.clone()),
        [start, end, ..] => Some(format!("{start}..{end}")),
    }
}

fn is_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[..4].iter().all(u8::is_ascii_digit)
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[8..].iter().all(u8::is_ascii_digit)
}

fn count_by(items: impl Iterator<Item = String>) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for key in items {
        *counts.entry(key).or_insert(0) += 1;
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn inventory_classifies_digest_sources_without_reading_contents() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("slack-digest/morning")).unwrap();
        fs::create_dir_all(dir.path().join("mail-digest")).unwrap();
        fs::create_dir_all(dir.path().join("ai-digest/digests")).unwrap();
        fs::write(
            dir.path().join("slack-digest/morning/2026-04-26.md"),
            "private slack content",
        )
        .unwrap();
        fs::write(
            dir.path().join("mail-digest/digest-2026-04-26.html"),
            "<html>private mail content</html>",
        )
        .unwrap();
        fs::write(
            dir.path().join("ai-digest/digests/2026-04-26.md"),
            "AI digest",
        )
        .unwrap();
        fs::write(dir.path().join("ai-digest/_seen.json"), "{}").unwrap();
        fs::write(dir.path().join("ai-digest/SKILL.md"), "# skill").unwrap();

        let inventory = DigestService::new()
            .inventory(DigestInventoryOptions::new(dir.path()))
            .unwrap();

        assert_eq!(inventory.files_scanned, 5);
        assert_eq!(inventory.total_candidates, 3);
        assert_eq!(inventory.excluded_count, 2);
        assert_eq!(inventory.by_source_kind["slack"], 1);
        assert_eq!(inventory.by_source_kind["email"], 1);
        assert_eq!(inventory.by_source_kind["ai"], 1);
        assert!(inventory
            .candidates
            .iter()
            .any(|candidate| candidate.bucket.as_deref() == Some("morning")));
        assert!(inventory
            .exclusions
            .iter()
            .any(|excluded| excluded.relative_path.ends_with("_seen.json")));
        assert!(inventory
            .warnings
            .iter()
            .any(|warning| warning.contains("no digest contents were read")));
    }

    #[test]
    fn inventory_limit_truncates_candidates_after_counting() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("notes-digest")).unwrap();
        fs::write(dir.path().join("notes-digest/digest-2026-04-25.md"), "one").unwrap();
        fs::write(dir.path().join("notes-digest/digest-2026-04-26.md"), "two").unwrap();

        let mut options = DigestInventoryOptions::new(dir.path());
        options.limit = Some(1);
        let inventory = DigestService::new().inventory(options).unwrap();

        assert_eq!(inventory.total_candidates, 2);
        assert_eq!(inventory.returned_candidates, 1);
        assert!(inventory.truncated);
    }

    #[test]
    fn inventory_accepts_digest_directory_as_root() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("slack-digest");
        fs::create_dir_all(root.join("evening")).unwrap();
        fs::write(root.join("evening/2026-04-26.md"), "digest").unwrap();

        let inventory = DigestService::new()
            .inventory(DigestInventoryOptions::new(&root))
            .unwrap();

        assert_eq!(inventory.total_candidates, 1);
        assert_eq!(inventory.candidates[0].collection, "slack-digest");
        assert_eq!(inventory.candidates[0].bucket.as_deref(), Some("evening"));
    }

    #[test]
    fn inventory_prefers_child_collection_over_digest_named_root() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("digest-smoke-root");
        fs::create_dir_all(root.join("slack-digest/morning")).unwrap();
        fs::write(root.join("slack-digest/morning/2026-04-26.md"), "digest").unwrap();

        let inventory = DigestService::new()
            .inventory(DigestInventoryOptions::new(&root))
            .unwrap();

        assert_eq!(inventory.total_candidates, 1);
        assert_eq!(inventory.candidates[0].collection, "slack-digest");
        assert_eq!(inventory.candidates[0].source_kind, DigestSourceKind::Slack);
        assert_eq!(inventory.candidates[0].bucket.as_deref(), Some("morning"));
    }

    #[test]
    fn inventory_rejects_missing_root() {
        let dir = tempdir().unwrap();
        let err = DigestService::new()
            .inventory(DigestInventoryOptions::new(dir.path().join("missing")))
            .unwrap_err();

        assert!(err.to_string().contains("file not found"));
    }

    #[test]
    fn date_hint_extracts_single_dates_and_ranges() {
        assert_eq!(
            extract_date_hint("slack-digest/morning/2026-04-26.md"),
            Some("2026-04-26".to_string())
        );
        assert_eq!(
            extract_date_hint("catchup/2026-02-26-to-2026-03-29.md"),
            Some("2026-02-26..2026-03-29".to_string())
        );
        assert_eq!(extract_date_hint("handoff.md"), None);
    }
}
