//! Embedding configuration.

use std::ffi::OsString;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

/// Engram-specific override for the embedding model cache directory.
pub const ENGRAM_EMBED_CACHE_DIR_ENV: &str = "ENGRAM_EMBED_CACHE_DIR";

/// Upstream fastembed cache override, preserved for compatibility.
pub const FASTEMBED_CACHE_DIR_ENV: &str = "FASTEMBED_CACHE_DIR";

/// Configuration for the embedder.
#[derive(Debug, Clone)]
pub struct EmbedConfig {
    /// Model to use for embeddings.
    pub model: EmbeddingModel,

    /// Batch size for processing.
    pub batch_size: usize,

    /// Whether to normalize embeddings.
    pub normalize: bool,

    /// Directory used to cache downloaded embedding model files.
    pub cache_dir: PathBuf,
}

/// Available embedding models.
#[derive(Debug, Clone, Default)]
pub enum EmbeddingModel {
    /// all-MiniLM-L6-v2 (384 dimensions, fast)
    #[default]
    AllMiniLmL6V2,

    /// bge-small-en-v1.5 (384 dimensions, better quality)
    BgeSmallEnV15,

    /// Custom model path
    Custom(String),
}

impl Default for EmbedConfig {
    fn default() -> Self {
        Self {
            model: EmbeddingModel::AllMiniLmL6V2,
            batch_size: 32,
            normalize: true,
            cache_dir: default_cache_dir(),
        }
    }
}

/// Get the default embedding model cache directory.
#[must_use]
pub fn default_cache_dir() -> PathBuf {
    default_cache_dir_from_env(|key| std::env::var_os(key))
}

/// Return true when the cache already appears to contain a downloaded ONNX model.
#[must_use]
pub fn cache_dir_has_model_files(cache_dir: &Path) -> bool {
    contains_onnx_file(cache_dir)
}

fn default_cache_dir_from_env(get_env: impl Fn(&str) -> Option<OsString>) -> PathBuf {
    if let Some(path) = non_empty_env(&get_env, ENGRAM_EMBED_CACHE_DIR_ENV) {
        return PathBuf::from(path);
    }

    if let Some(path) = non_empty_env(&get_env, FASTEMBED_CACHE_DIR_ENV) {
        return PathBuf::from(path);
    }

    if let Some(home) = non_empty_env(&get_env, "HOME") {
        return PathBuf::from(home)
            .join(".engram")
            .join("cache")
            .join("fastembed");
    }

    PathBuf::from(".engram").join("cache").join("fastembed")
}

fn non_empty_env(get_env: &impl Fn(&str) -> Option<OsString>, key: &str) -> Option<OsString> {
    get_env(key).filter(|value| !value.as_os_str().is_empty())
}

fn contains_onnx_file(path: &Path) -> bool {
    let mut pending = vec![path.to_path_buf()];

    while let Some(dir) = pending.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("onnx"))
            {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cache_dir_with(vars: &[(&str, &str)]) -> PathBuf {
        default_cache_dir_from_env(|key| {
            vars.iter()
                .find(|(candidate, _)| *candidate == key)
                .map(|(_, value)| OsString::from(value))
        })
    }

    #[test]
    fn engram_cache_env_wins() {
        assert_eq!(
            cache_dir_with(&[
                (ENGRAM_EMBED_CACHE_DIR_ENV, "/tmp/engram-cache"),
                (FASTEMBED_CACHE_DIR_ENV, "/tmp/fastembed-cache"),
                ("HOME", "/tmp/home"),
            ]),
            PathBuf::from("/tmp/engram-cache")
        );
    }

    #[test]
    fn fastembed_cache_env_is_preserved() {
        assert_eq!(
            cache_dir_with(&[
                (ENGRAM_EMBED_CACHE_DIR_ENV, ""),
                (FASTEMBED_CACHE_DIR_ENV, "/tmp/fastembed-cache"),
                ("HOME", "/tmp/home"),
            ]),
            PathBuf::from("/tmp/fastembed-cache")
        );
    }

    #[test]
    fn home_fallback_matches_engram_layout() {
        assert_eq!(
            cache_dir_with(&[("HOME", "/tmp/home")]),
            PathBuf::from("/tmp/home/.engram/cache/fastembed")
        );
    }

    #[test]
    fn relative_fallback_when_home_is_unavailable() {
        assert_eq!(
            cache_dir_with(&[]),
            PathBuf::from(".engram/cache/fastembed")
        );
    }

    #[test]
    fn cache_dir_model_detection_finds_nested_onnx_file() {
        let dir = test_cache_dir("nested-onnx");
        let model_dir = dir.join("models").join("snapshot");
        fs::create_dir_all(&model_dir).expect("model dir");
        fs::write(model_dir.join("model.onnx"), b"model").expect("model file");

        assert!(cache_dir_has_model_files(&dir));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn cache_dir_model_detection_treats_missing_cache_as_cold() {
        let dir = test_cache_dir("missing-cache");

        assert!(!cache_dir_has_model_files(&dir.join("missing")));
        let _ = fs::remove_dir_all(dir);
    }

    fn test_cache_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("engram-cache-test-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("temp test dir");
        dir
    }
}
