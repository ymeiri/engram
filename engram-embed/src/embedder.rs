//! Embedding generation using fastembed.

use crate::config::{EmbedConfig, EmbeddingModel};
use crate::error::{EmbedError, EmbedResult};
use fastembed::{EmbeddingModel as FastEmbedModel, InitOptions, TextEmbedding};
use std::sync::Arc;
use tracing::info;

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

        info!("Loading embedding model: {:?}", config.model);

        let model =
            TextEmbedding::try_new(InitOptions::new(model_type).with_show_download_progress(true))
                .map_err(|e| EmbedError::ModelLoad(e.to_string()))?;

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
