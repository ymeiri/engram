//! Embedding configuration.

/// Configuration for the embedder.
#[derive(Debug, Clone)]
pub struct EmbedConfig {
    /// Model to use for embeddings.
    pub model: EmbeddingModel,

    /// Batch size for processing.
    pub batch_size: usize,

    /// Whether to normalize embeddings.
    pub normalize: bool,
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
        }
    }
}
