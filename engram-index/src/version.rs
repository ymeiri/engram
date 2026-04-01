//! Version detection for documents.
//!
//! Detects version information from filenames and content.
//! Supports patterns like: `-v1`, `-v2`, `_v1`, `_v2`, ` v1`, `(1)`, `(2)`, etc.

use std::path::Path;

/// Extracted version information from a document.
#[derive(Debug, Clone, PartialEq)]
pub struct VersionInfo {
    /// The detected version number (if any).
    pub version: Option<u32>,
    /// The base name without version suffix.
    pub base_name: String,
    /// Source of version detection.
    pub source: VersionSource,
}

/// Source of version detection.
#[derive(Debug, Clone, PartialEq)]
pub enum VersionSource {
    /// Version from filename pattern.
    Filename,
    /// Version from content header.
    Content,
    /// No version detected.
    None,
}

/// Version detector configuration.
#[derive(Debug, Clone)]
pub struct VersionDetectorConfig {
    /// Whether to check content for version headers.
    pub check_content: bool,
}

impl Default for VersionDetectorConfig {
    fn default() -> Self {
        Self {
            check_content: true,
        }
    }
}

/// Detects version information from files.
pub struct VersionDetector {
    config: VersionDetectorConfig,
}

impl VersionDetector {
    /// Create a new version detector with default config.
    pub fn new() -> Self {
        Self {
            config: VersionDetectorConfig::default(),
        }
    }

    /// Create with custom config.
    pub fn with_config(config: VersionDetectorConfig) -> Self {
        Self { config }
    }

    /// Detect version from a file path and optional content.
    pub fn detect(&self, path: &Path, content: Option<&str>) -> VersionInfo {
        // Try filename first
        if let Some(info) = self.detect_from_filename(path) {
            return info;
        }

        // Try content if enabled and available
        if self.config.check_content {
            if let Some(content) = content {
                if let Some(info) = self.detect_from_content(path, content) {
                    return info;
                }
            }
        }

        // No version detected
        let base_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        VersionInfo {
            version: None,
            base_name,
            source: VersionSource::None,
        }
    }

    /// Detect version from filename patterns.
    fn detect_from_filename(&self, path: &Path) -> Option<VersionInfo> {
        let stem = path.file_stem()?.to_str()?;

        // Patterns to check (in order of preference):
        // - file-v1.md, file-v2.md
        // - file_v1.md, file_v2.md
        // - file v1.md, file v2.md
        // - file(1).md, file(2).md
        // - file-1.md, file-2.md (less specific, check last)

        // Pattern: -v{N} or _v{N} or  v{N}
        for sep in ["-v", "_v", " v", "-V", "_V", " V"] {
            if let Some((base, version)) = self.extract_version_with_separator(stem, sep) {
                return Some(VersionInfo {
                    version: Some(version),
                    base_name: base,
                    source: VersionSource::Filename,
                });
            }
        }

        // Pattern: (N) at end
        if let Some((base, version)) = self.extract_version_parentheses(stem) {
            return Some(VersionInfo {
                version: Some(version),
                base_name: base,
                source: VersionSource::Filename,
            });
        }

        // Pattern: -N or _N at end (less specific, only for single digits to avoid false positives)
        for sep in ["-", "_"] {
            if let Some((base, version)) = self.extract_simple_version(stem, sep) {
                return Some(VersionInfo {
                    version: Some(version),
                    base_name: base,
                    source: VersionSource::Filename,
                });
            }
        }

        None
    }

    /// Extract version with separator like "-v" or "_v".
    fn extract_version_with_separator(&self, stem: &str, separator: &str) -> Option<(String, u32)> {
        let lower = stem.to_lowercase();
        let sep_lower = separator.to_lowercase();

        if let Some(pos) = lower.rfind(&sep_lower) {
            let version_part = &stem[pos + separator.len()..];
            if let Ok(version) = version_part.parse::<u32>() {
                let base = stem[..pos].to_string();
                if !base.is_empty() {
                    return Some((base, version));
                }
            }
        }
        None
    }

    /// Extract version from parentheses like "(1)" or "(2)".
    fn extract_version_parentheses(&self, stem: &str) -> Option<(String, u32)> {
        if stem.ends_with(')') {
            if let Some(start) = stem.rfind('(') {
                let version_part = &stem[start + 1..stem.len() - 1];
                if let Ok(version) = version_part.parse::<u32>() {
                    let base = stem[..start].trim_end().to_string();
                    if !base.is_empty() {
                        return Some((base, version));
                    }
                }
            }
        }
        None
    }

    /// Extract simple version like "-1" or "_1" (only single digit).
    fn extract_simple_version(&self, stem: &str, separator: &str) -> Option<(String, u32)> {
        if let Some(pos) = stem.rfind(separator) {
            let version_part = &stem[pos + separator.len()..];
            // Only match single digits to avoid false positives
            if version_part.len() == 1 {
                if let Ok(version) = version_part.parse::<u32>() {
                    let base = stem[..pos].to_string();
                    if !base.is_empty() {
                        return Some((base, version));
                    }
                }
            }
        }
        None
    }

    /// Detect version from content headers.
    fn detect_from_content(&self, path: &Path, content: &str) -> Option<VersionInfo> {
        let base_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        // Check first 20 lines for version patterns
        for line in content.lines().take(20) {
            let line = line.trim();

            // Pattern: "Version: N" or "Version N" or "version: N"
            if let Some(version) = self.extract_version_from_line(line, "version") {
                return Some(VersionInfo {
                    version: Some(version),
                    base_name,
                    source: VersionSource::Content,
                });
            }

            // Pattern: "# Version N" or "## Version N"
            let without_hashes = line.trim_start_matches('#').trim();
            if let Some(version) = self.extract_version_from_line(without_hashes, "version") {
                return Some(VersionInfo {
                    version: Some(version),
                    base_name,
                    source: VersionSource::Content,
                });
            }

            // Pattern: "v1.0" or "V1.0" at start of line
            if let Some(version) = self.extract_semver_major(line) {
                return Some(VersionInfo {
                    version: Some(version),
                    base_name,
                    source: VersionSource::Content,
                });
            }
        }

        None
    }

    /// Extract version from a line containing "version".
    fn extract_version_from_line(&self, line: &str, keyword: &str) -> Option<u32> {
        let lower = line.to_lowercase();
        if let Some(pos) = lower.find(keyword) {
            let after = &line[pos + keyword.len()..];
            let after = after.trim_start_matches(':').trim_start();

            // Extract first number
            let version_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            if !version_str.is_empty() {
                return version_str.parse().ok();
            }
        }
        None
    }

    /// Extract major version from semver-like "v1.0.0" or "V2.1".
    fn extract_semver_major(&self, line: &str) -> Option<u32> {
        let line = line.trim();
        if (line.starts_with('v') || line.starts_with('V')) && line.len() > 1 {
            let rest = &line[1..];
            // Must start with a digit
            if rest.chars().next()?.is_ascii_digit() {
                let version_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
                return version_str.parse().ok();
            }
        }
        None
    }
}

impl Default for VersionDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Group files by their base name (for version chain detection).
pub fn group_by_base_name(files: &[(String, VersionInfo)]) -> Vec<VersionGroup> {
    use std::collections::HashMap;

    let mut groups: HashMap<String, Vec<(String, VersionInfo)>> = HashMap::new();

    for (path, info) in files {
        groups
            .entry(info.base_name.to_lowercase())
            .or_default()
            .push((path.clone(), info.clone()));
    }

    groups
        .into_iter()
        .filter(|(_, versions)| {
            versions.len() > 1 || versions.iter().any(|(_, v)| v.version.is_some())
        })
        .map(|(base_name, mut versions)| {
            // Sort by version number (None last, then ascending)
            versions.sort_by(|a, b| match (a.1.version, b.1.version) {
                (Some(va), Some(vb)) => va.cmp(&vb),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.0.cmp(&b.0),
            });

            VersionGroup {
                base_name,
                versions,
            }
        })
        .collect()
}

/// A group of file versions.
#[derive(Debug, Clone)]
pub struct VersionGroup {
    /// The base name shared by all versions.
    pub base_name: String,
    /// The versions, sorted by version number ascending.
    pub versions: Vec<(String, VersionInfo)>,
}

impl VersionGroup {
    /// Get the canonical (highest version) file path.
    pub fn canonical_path(&self) -> Option<&str> {
        self.versions.last().map(|(path, _)| path.as_str())
    }

    /// Get the highest version number.
    pub fn highest_version(&self) -> Option<u32> {
        self.versions
            .iter()
            .filter_map(|(_, info)| info.version)
            .max()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_version_dash_v() {
        let detector = VersionDetector::new();
        let path = Path::new("docs/setup-guide-v2.md");
        let info = detector.detect(path, None);

        assert_eq!(info.version, Some(2));
        assert_eq!(info.base_name, "setup-guide");
        assert_eq!(info.source, VersionSource::Filename);
    }

    #[test]
    fn test_detect_version_underscore_v() {
        let detector = VersionDetector::new();
        let path = Path::new("docs/setup_guide_v3.md");
        let info = detector.detect(path, None);

        assert_eq!(info.version, Some(3));
        assert_eq!(info.base_name, "setup_guide");
        assert_eq!(info.source, VersionSource::Filename);
    }

    #[test]
    fn test_detect_version_parentheses() {
        let detector = VersionDetector::new();
        let path = Path::new("docs/setup guide (2).md");
        let info = detector.detect(path, None);

        assert_eq!(info.version, Some(2));
        assert_eq!(info.base_name, "setup guide");
        assert_eq!(info.source, VersionSource::Filename);
    }

    #[test]
    fn test_detect_version_from_content() {
        let detector = VersionDetector::new();
        let path = Path::new("docs/setup-guide.md");
        let content = "# Setup Guide\n\nVersion: 3\n\nThis is the setup guide.";
        let info = detector.detect(path, Some(content));

        assert_eq!(info.version, Some(3));
        assert_eq!(info.base_name, "setup-guide");
        assert_eq!(info.source, VersionSource::Content);
    }

    #[test]
    fn test_no_version() {
        let detector = VersionDetector::new();
        let path = Path::new("docs/readme.md");
        let info = detector.detect(path, None);

        assert_eq!(info.version, None);
        assert_eq!(info.base_name, "readme");
        assert_eq!(info.source, VersionSource::None);
    }

    #[test]
    fn test_group_by_base_name() {
        let files = vec![
            (
                "docs/guide-v1.md".to_string(),
                VersionInfo {
                    version: Some(1),
                    base_name: "guide".to_string(),
                    source: VersionSource::Filename,
                },
            ),
            (
                "docs/guide-v2.md".to_string(),
                VersionInfo {
                    version: Some(2),
                    base_name: "guide".to_string(),
                    source: VersionSource::Filename,
                },
            ),
            (
                "other/readme.md".to_string(),
                VersionInfo {
                    version: None,
                    base_name: "readme".to_string(),
                    source: VersionSource::None,
                },
            ),
        ];

        let groups = group_by_base_name(&files);

        // Only "guide" group should be returned (has multiple versions)
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].base_name, "guide");
        assert_eq!(groups[0].versions.len(), 2);
        assert_eq!(groups[0].canonical_path(), Some("docs/guide-v2.md"));
        assert_eq!(groups[0].highest_version(), Some(2));
    }
}
