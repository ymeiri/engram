//! Embedding generation using fastembed.

use crate::config::{cache_dir_has_model_files, EmbedConfig, EmbeddingModel};
use crate::error::{EmbedError, EmbedResult};
use fastembed::{
    EmbeddingModel as FastEmbedModel, InitOptions, InitOptionsUserDefined, Pooling, TextEmbedding,
    TokenizerFiles, UserDefinedEmbeddingModel,
};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tracing::{info, warn};

/// The embedder for generating text embeddings.
pub struct Embedder {
    model: Arc<TextEmbedding>,
    config: EmbedConfig,
    dimension: usize,
}

impl Embedder {
    /// Create a new embedder with the given configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the model cannot be loaded.
    pub fn new(config: EmbedConfig) -> EmbedResult<Self> {
        let model_type = match &config.model {
            EmbeddingModel::AllMiniLmL6V2 => FastEmbedModel::AllMiniLML6V2,
            EmbeddingModel::BgeSmallEnV15 => FastEmbedModel::BGESmallENV15,
            EmbeddingModel::Custom(_path) => {
                // For custom models, default to AllMiniLM for now
                // TODO: Support custom ONNX models
                FastEmbedModel::AllMiniLML6V2
            }
        };

        info!(
            "Loading embedding model: {:?} from cache {}",
            config.model,
            config.cache_dir.display()
        );
        if !cache_dir_has_model_files(&config.cache_dir) {
            let message = format!(
                "Engram is preparing the local embedding model cache at {}. \
                 First run may download all-MiniLM-L6-v2 (~90 MB) from Hugging Face. \
                 Run `engram warmup embeddings` before offline use, or set \
                 ENGRAM_EMBED_CACHE_DIR to a pre-warmed cache.",
                config.cache_dir.display()
            );
            warn!("{}", message);
            eprintln!("engram: {}", message);
        }

        let options = InitOptions::new(model_type)
            .with_show_download_progress(true)
            .with_cache_dir(config.cache_dir.clone());
        let model =
            TextEmbedding::try_new(options).map_err(|e| EmbedError::ModelLoad(e.to_string()))?;

        // Get dimension from model info
        let dimension = match &config.model {
            EmbeddingModel::AllMiniLmL6V2 => 384,
            EmbeddingModel::BgeSmallEnV15 => 384,
            EmbeddingModel::Custom(_) => 384, // Assume 384 for custom
        };

        Ok(Self {
            model: Arc::new(model),
            config,
            dimension,
        })
    }

    /// Create an all-MiniLM-L6-v2 embedder from an already-resolved local snapshot.
    ///
    /// This path performs no model discovery or download. Callers that require an attested model
    /// should verify the snapshot bytes before invoking it.
    ///
    /// # Errors
    ///
    /// Returns an error if the configured model is unsupported, a required file is missing, or
    /// FastEmbed cannot load the supplied bytes.
    pub fn from_local_snapshot(config: EmbedConfig, snapshot_dir: &Path) -> EmbedResult<Self> {
        if !matches!(config.model, EmbeddingModel::AllMiniLmL6V2) {
            return Err(EmbedError::InvalidInput(
                "local snapshots currently support only all-MiniLM-L6-v2".to_string(),
            ));
        }

        let read = |name: &str| {
            fs::read(snapshot_dir.join(name)).map_err(|error| {
                EmbedError::ModelLoad(format!(
                    "could not read {} from {}: {error}",
                    name,
                    snapshot_dir.display()
                ))
            })
        };
        let tokenizer_files = TokenizerFiles {
            tokenizer_file: read("tokenizer.json")?,
            config_file: read("config.json")?,
            special_tokens_map_file: read("special_tokens_map.json")?,
            tokenizer_config_file: read("tokenizer_config.json")?,
        };
        let supplied = UserDefinedEmbeddingModel::new(read("model.onnx")?, tokenizer_files)
            .with_pooling(Pooling::Mean);
        let model =
            TextEmbedding::try_new_from_user_defined(supplied, InitOptionsUserDefined::default())
                .map_err(|error| EmbedError::ModelLoad(error.to_string()))?;

        Ok(Self {
            model: Arc::new(model),
            config,
            dimension: 384,
        })
    }

    /// Create an embedder with default configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the model cannot be loaded.
    pub fn default_model() -> EmbedResult<Self> {
        Self::new(EmbedConfig::default())
    }

    /// Generate embeddings for a single text.
    ///
    /// # Errors
    ///
    /// Returns an error if embedding generation fails.
    pub fn embed(&self, text: &str) -> EmbedResult<Vec<f32>> {
        self.embed_batch(&[text]).map(|mut v| v.remove(0))
    }

    /// Generate embeddings for multiple texts.
    ///
    /// # Errors
    ///
    /// Returns an error if embedding generation fails.
    pub fn embed_batch(&self, texts: &[&str]) -> EmbedResult<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        // Convert to owned strings for fastembed
        let texts: Vec<String> = texts.iter().map(|s| (*s).to_string()).collect();

        self.model
            .embed(texts, Some(self.config.batch_size))
            .map_err(|e| EmbedError::Embedding(e.to_string()))
    }

    /// Get the embedding dimension.
    #[must_use]
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Get the configuration.
    #[must_use]
    pub fn config(&self) -> &EmbedConfig {
        &self.config
    }

    /// Clone the underlying model (cheap, Arc-based).
    #[must_use]
    pub fn clone_model(&self) -> Arc<TextEmbedding> {
        Arc::clone(&self.model)
    }
}

// Implement Clone manually since TextEmbedding doesn't implement Clone
impl Clone for Embedder {
    fn clone(&self) -> Self {
        Self {
            model: Arc::clone(&self.model),
            config: self.config.clone(),
            dimension: self.dimension,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require downloading the model, so they're ignored by default
    #[test]
    #[ignore = "requires model download"]
    fn test_embedder_creation() {
        let embedder = Embedder::default_model().unwrap();
        assert_eq!(embedder.dimension(), 384);
    }

    #[test]
    #[ignore = "requires model download"]
    fn test_embed_single() {
        let embedder = Embedder::default_model().unwrap();
        let embedding = embedder.embed("Hello, world!").unwrap();
        assert_eq!(embedding.len(), 384);
    }

    #[test]
    #[ignore = "requires model download"]
    fn test_embed_batch() {
        let embedder = Embedder::default_model().unwrap();
        let embeddings = embedder.embed_batch(&["Hello", "World", "Test"]).unwrap();
        assert_eq!(embeddings.len(), 3);
        assert_eq!(embeddings[0].len(), 384);
    }
}
