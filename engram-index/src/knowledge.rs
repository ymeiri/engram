//! Knowledge service for Layer 6: Document Intelligence.
//!
//! Provides canonical document resolution, duplicate detection, version tracking,
//! and integration with Layer 3 for semantic search.

use crate::error::{IndexError, IndexResult};
use crate::service::DocumentService;
use crate::version::{group_by_base_name, VersionDetector, VersionInfo};
use engram_core::id::Id;
use engram_core::knowledge::{
    DocAlias, DocEvent, DocEventType, DocStatus, DocType, FileSync, KnowledgeDoc,
};
use engram_store::{Db, KnowledgeRepo};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use time::OffsetDateTime;
use tracing::{info, warn};

/// Configuration for the knowledge service.
#[derive(Debug, Clone)]
pub struct KnowledgeConfig {
    /// Path to the personal knowledge repository.
    pub knowledge_repo_path: PathBuf,
    /// Whether to auto-initialize git if not present.
    pub auto_init_git: bool,
    /// File extensions to scan.
    pub extensions: Vec<String>,
    /// Whether to scan recursively.
    pub recursive: bool,
}

impl Default for KnowledgeConfig {
    fn default() -> Self {
        Self {
            knowledge_repo_path: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".engram")
                .join("knowledge"),
            auto_init_git: true,
            extensions: vec!["md".to_string(), "markdown".to_string()],
            recursive: true,
        }
    }
}

/// The knowledge service for Layer 6.
pub struct KnowledgeService {
    repo: KnowledgeRepo,
    doc_service: Option<DocumentService>,
    version_detector: VersionDetector,
    config: KnowledgeConfig,
}

impl KnowledgeService {
    /// Create a new knowledge service.
    pub fn new(db: Db, config: KnowledgeConfig) -> Self {
        Self {
            repo: KnowledgeRepo::new(db),
            doc_service: None,
            version_detector: VersionDetector::new(),
            config,
        }
    }

    /// Create with default configuration.
    pub fn with_defaults(db: Db) -> Self {
        Self::new(db, KnowledgeConfig::default())
    }

    /// Set the document service for Layer 3 integration.
    pub fn with_doc_service(mut self, service: DocumentService) -> Self {
        self.doc_service = Some(service);
        self
    }

    /// Initialize the knowledge system.
    ///
    /// Creates the personal knowledge repo directory and initializes git if configured.
    pub async fn init(&self) -> IndexResult<()> {
        info!("Initializing knowledge system");

        // Initialize database schema
        self.repo.init_schema().await?;

        // Create knowledge repo directory
        let repo_path = &self.config.knowledge_repo_path;
        if !repo_path.exists() {
            info!("Creating knowledge repo at: {}", repo_path.display());
            std::fs::create_dir_all(repo_path)?;

            // Create subdirectories for each doc type
            for subdir in [
                "adrs",
                "runbooks",
                "howtos",
                "research",
                "designs",
                "readmes",
                "changelogs",
                "custom",
            ] {
                std::fs::create_dir_all(repo_path.join(subdir))?;
            }
        }

        // Initialize git if configured
        if self.config.auto_init_git && !repo_path.join(".git").exists() {
            info!("Initializing git repository");
            std::process::Command::new("git")
                .args(["init"])
                .current_dir(repo_path)
                .output()
                .map_err(|e| IndexError::Io(e))?;

            // Create initial .gitignore
            let gitignore_path = repo_path.join(".gitignore");
            if !gitignore_path.exists() {
                std::fs::write(&gitignore_path, "# Engram knowledge repo\n.DS_Store\n")?;
            }
        }

        info!("Knowledge system initialized");
        Ok(())
    }

    /// Get the knowledge repo path.
    pub fn knowledge_repo_path(&self) -> &Path {
        &self.config.knowledge_repo_path
    }

    // ==================== Scanning ====================

    /// Scan a directory for markdown files.
    ///
    /// Creates FileSync entries for each file found.
    /// Does NOT create KnowledgeDoc entries - those are created via register/import.
    pub async fn scan_directory(&self, path: &Path, repo_name: &str) -> IndexResult<ScanResult> {
        info!(
            "Scanning directory: {} (repo: {})",
            path.display(),
            repo_name
        );

        let mut files_found = 0;
        let mut files_new = 0;
        let mut files_updated = 0;

        let files = self.find_files(path)?;

        for file_path in files {
            files_found += 1;

            let content = std::fs::read_to_string(&file_path)?;
            let hash = compute_hash(&content);
            let path_str = file_path.display().to_string();

            // Check if we already have a FileSync for this
            if let Some(mut existing) = self.repo.find_file_sync(&path_str, repo_name).await? {
                // Check if content changed
                if existing.last_hash != hash {
                    existing.last_hash = hash;
                    existing.last_modified = OffsetDateTime::now_utc();
                    existing.mark_stale();
                    self.repo.save_file_sync(&existing).await?;
                    files_updated += 1;
                }
            } else {
                // Create new FileSync entry
                let sync = FileSync::new(&path_str, repo_name, hash);
                self.repo.save_file_sync(&sync).await?;
                files_new += 1;
            }
        }

        info!(
            "Scan complete: {} files found, {} new, {} updated",
            files_found, files_new, files_updated
        );

        Ok(ScanResult {
            files_found,
            files_new,
            files_updated,
        })
    }

    /// Find all indexable files in a directory.
    fn find_files(&self, path: &Path) -> IndexResult<Vec<PathBuf>> {
        let mut files = Vec::new();
        self.find_files_recursive(path, &mut files)?;
        Ok(files)
    }

    fn find_files_recursive(&self, path: &Path, files: &mut Vec<PathBuf>) -> IndexResult<()> {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if self.config.recursive {
                    self.find_files_recursive(&path, files)?;
                }
            } else if self.should_scan(&path) {
                files.push(path);
            }
        }
        Ok(())
    }

    fn should_scan(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|ext| self.config.extensions.iter().any(|e| e == ext))
            .unwrap_or(false)
    }

    // ==================== Duplicate Detection ====================

    /// Find duplicate documents by content hash.
    pub async fn find_duplicates(&self) -> IndexResult<Vec<DuplicateGroup>> {
        info!("Finding duplicates by content hash");

        let all_syncs = self.repo.list_file_syncs().await?;

        // Group by hash
        let mut hash_groups: HashMap<String, Vec<FileSync>> = HashMap::new();
        for sync in all_syncs {
            hash_groups
                .entry(sync.last_hash.clone())
                .or_default()
                .push(sync);
        }

        // Filter to groups with more than one file
        let duplicates: Vec<DuplicateGroup> = hash_groups
            .into_iter()
            .filter(|(_, files)| files.len() > 1)
            .map(|(hash, files)| DuplicateGroup {
                content_hash: hash,
                files,
            })
            .collect();

        info!("Found {} duplicate groups", duplicates.len());
        Ok(duplicates)
    }

    // ==================== Version Detection ====================

    /// Detect version chains in scanned files.
    pub async fn detect_versions(&self) -> IndexResult<Vec<VersionChain>> {
        info!("Detecting version chains");

        let all_syncs = self.repo.list_file_syncs().await?;

        // Detect versions for each file
        let mut version_infos: Vec<(String, VersionInfo)> = Vec::new();
        for sync in &all_syncs {
            let path = Path::new(&sync.path);

            // Try to read content for content-based version detection
            let content = std::fs::read_to_string(path).ok();
            let info = self.version_detector.detect(path, content.as_deref());

            version_infos.push((sync.path.clone(), info));
        }

        // Group by base name
        let groups = group_by_base_name(&version_infos);

        // Convert to VersionChains
        let chains: Vec<VersionChain> = groups
            .into_iter()
            .map(|group| VersionChain {
                base_name: group.base_name,
                versions: group
                    .versions
                    .into_iter()
                    .map(|(path, info)| VersionedFile {
                        path,
                        version: info.version,
                    })
                    .collect(),
            })
            .collect();

        info!("Found {} version chains", chains.len());
        Ok(chains)
    }

    // ==================== Canonical Resolution ====================

    /// Auto-select canonical document for a version chain.
    ///
    /// Selection criteria:
    /// 1. Highest version number
    /// 2. Newest modification time (tiebreaker)
    pub async fn resolve_canonical(&self, chain: &VersionChain) -> IndexResult<Option<String>> {
        if chain.versions.is_empty() {
            return Ok(None);
        }

        // Sort by version (highest first), then by modification time
        let mut candidates: Vec<(String, Option<u32>, OffsetDateTime)> = Vec::new();

        for v in &chain.versions {
            let mtime = std::fs::metadata(&v.path)
                .map(|m| {
                    m.modified()
                        .ok()
                        .map(|t| OffsetDateTime::from(t))
                        .unwrap_or_else(OffsetDateTime::now_utc)
                })
                .unwrap_or_else(|_| OffsetDateTime::now_utc());

            candidates.push((v.path.clone(), v.version, mtime));
        }

        // Sort: highest version first, then newest modification time
        candidates.sort_by(|a, b| {
            match (a.1, b.1) {
                (Some(va), Some(vb)) => vb.cmp(&va), // Descending by version
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => b.2.cmp(&a.2), // Descending by mtime
            }
        });

        Ok(candidates.first().map(|(path, _, _)| path.clone()))
    }

    // ==================== Document Registration ====================

    /// Register a document as a KnowledgeDoc (creates a reference, doesn't copy).
    pub async fn register_doc(
        &self,
        path: &Path,
        name: &str,
        doc_type: DocType,
    ) -> IndexResult<KnowledgeDoc> {
        info!("Registering document: {} as '{}'", path.display(), name);

        let content = std::fs::read_to_string(path)?;
        let path_str = path.display().to_string();

        // Create KnowledgeDoc
        let doc = KnowledgeDoc::new(name, doc_type, content).with_path(&path_str);

        // Save to database
        self.repo.save_doc(&doc).await?;

        // Record event
        let event = DocEvent::new(doc.id, DocEventType::Created, "engram");
        self.repo.save_event(&event).await?;

        // Index in Layer 3 if available
        if let Some(ref doc_service) = self.doc_service {
            if let Err(e) = doc_service.index_file(path).await {
                warn!("Failed to index in Layer 3: {}", e);
            }
        }

        Ok(doc)
    }

    /// Import a document to the personal knowledge repo.
    ///
    /// Copies the file to the appropriate subdirectory.
    pub async fn import_doc(
        &self,
        source_path: &Path,
        name: &str,
        doc_type: DocType,
    ) -> IndexResult<KnowledgeDoc> {
        info!(
            "Importing document: {} as '{}'",
            source_path.display(),
            name
        );

        let content = std::fs::read_to_string(source_path)?;

        // Determine destination path
        let subdir = doc_type_to_subdir(&doc_type);
        let dest_dir = self.config.knowledge_repo_path.join(subdir);
        std::fs::create_dir_all(&dest_dir)?;

        // Sanitize filename
        let filename = sanitize_filename(name);
        let dest_path = dest_dir.join(format!("{}.md", filename));

        // Check for existing file
        if dest_path.exists() {
            return Err(IndexError::FileExists(dest_path.display().to_string()));
        }

        // Copy content
        std::fs::write(&dest_path, &content)?;

        // Create KnowledgeDoc
        let doc =
            KnowledgeDoc::new(name, doc_type, content).with_path(dest_path.display().to_string());

        // Save to database
        self.repo.save_doc(&doc).await?;

        // Record event
        let event = DocEvent::new(doc.id, DocEventType::Created, "engram").with_details(
            serde_json::json!({
                "source": source_path.display().to_string(),
                "imported": true
            }),
        );
        self.repo.save_event(&event).await?;

        // Index in Layer 3 if available
        if let Some(ref doc_service) = self.doc_service {
            if let Err(e) = doc_service.index_file(&dest_path).await {
                warn!("Failed to index in Layer 3: {}", e);
            }
        }

        info!("Document imported to: {}", dest_path.display());
        Ok(doc)
    }

    // ==================== Document Management ====================

    /// List all knowledge documents.
    pub async fn list_docs(&self) -> IndexResult<Vec<KnowledgeDoc>> {
        Ok(self.repo.list_docs().await?)
    }

    /// Get a document by ID.
    pub async fn get_doc(&self, id: &Id) -> IndexResult<KnowledgeDoc> {
        Ok(self.repo.get_doc(id).await?)
    }

    /// Find a document by name or alias.
    pub async fn find_doc(&self, name_or_alias: &str) -> IndexResult<Option<KnowledgeDoc>> {
        // Try by name first
        if let Some(doc) = self.repo.find_doc_by_name(name_or_alias).await? {
            return Ok(Some(doc));
        }

        // Try by alias
        if let Some(doc) = self.repo.find_doc_by_alias(name_or_alias).await? {
            return Ok(Some(doc));
        }

        Ok(None)
    }

    /// Add an alias for a document.
    pub async fn add_alias(&self, doc_id: &Id, alias: &str) -> IndexResult<()> {
        let alias_entry = DocAlias::new(alias, *doc_id);
        self.repo.save_alias(&alias_entry).await?;

        // Record event
        let event = DocEvent::new(*doc_id, DocEventType::AliasAdded, "engram")
            .with_details(serde_json::json!({ "alias": alias }));
        self.repo.save_event(&event).await?;

        Ok(())
    }

    /// Set a document as the canonical version.
    pub async fn set_canonical(&self, doc_id: &Id) -> IndexResult<()> {
        let mut doc = self.repo.get_doc(doc_id).await?;
        doc.status = DocStatus::Active;
        self.repo.save_doc(&doc).await?;
        Ok(())
    }

    /// Get statistics.
    pub async fn stats(&self) -> IndexResult<KnowledgeStats> {
        let db_stats = self.repo.stats().await?;
        Ok(KnowledgeStats {
            doc_count: db_stats.doc_count,
            file_sync_count: db_stats.file_sync_count,
            alias_count: db_stats.alias_count,
        })
    }
}

// ==================== Helper Types ====================

/// Result of scanning a directory.
#[derive(Debug, Clone)]
pub struct ScanResult {
    /// Total files found.
    pub files_found: usize,
    /// New files (not previously scanned).
    pub files_new: usize,
    /// Updated files (content changed).
    pub files_updated: usize,
}

/// A group of duplicate files.
#[derive(Debug, Clone)]
pub struct DuplicateGroup {
    /// The shared content hash.
    pub content_hash: String,
    /// The files with this hash.
    pub files: Vec<FileSync>,
}

/// A chain of document versions.
#[derive(Debug, Clone)]
pub struct VersionChain {
    /// The base name shared by all versions.
    pub base_name: String,
    /// The versions, sorted by version number.
    pub versions: Vec<VersionedFile>,
}

/// A versioned file.
#[derive(Debug, Clone)]
pub struct VersionedFile {
    /// Path to the file.
    pub path: String,
    /// Detected version number (if any).
    pub version: Option<u32>,
}

/// Knowledge statistics.
#[derive(Debug, Clone)]
pub struct KnowledgeStats {
    /// Number of knowledge documents.
    pub doc_count: u64,
    /// Number of file sync records.
    pub file_sync_count: u64,
    /// Number of aliases.
    pub alias_count: u64,
}

// ==================== Helper Functions ====================

/// Compute content hash.
fn compute_hash(content: &str) -> String {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Convert DocType to subdirectory name.
fn doc_type_to_subdir(doc_type: &DocType) -> &'static str {
    match doc_type {
        DocType::Adr => "adrs",
        DocType::Runbook => "runbooks",
        DocType::Howto => "howtos",
        DocType::Research => "research",
        DocType::Design => "designs",
        DocType::Readme => "readmes",
        DocType::Changelog => "changelogs",
        DocType::Custom(_) => "custom",
    }
}

/// Sanitize a filename.
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim()
        .to_lowercase()
        .replace(' ', "-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("My Document!"), "my-document-");
        assert_eq!(sanitize_filename("Setup Guide v2"), "setup-guide-v2");
        assert_eq!(sanitize_filename("test_doc"), "test_doc");
    }

    #[test]
    fn test_doc_type_to_subdir() {
        assert_eq!(doc_type_to_subdir(&DocType::Adr), "adrs");
        assert_eq!(doc_type_to_subdir(&DocType::Howto), "howtos");
        assert_eq!(doc_type_to_subdir(&DocType::Custom("foo".into())), "custom");
    }
}
