//! Error types for the ruvllm edge inference engine.

use thiserror::Error;

/// Errors that can occur during local inference.
#[derive(Debug, Error)]
pub enum RuvllmError {
    /// The ruvllm engine is not available (compiled without `ruvllm` feature).
    #[error("ruvllm engine not available: compile with --features ruvllm")]
    NotAvailable,

    /// No model is currently loaded.
    #[error("no model loaded")]
    NoModelLoaded,

    /// The requested model was not found in the cache directory.
    #[error("model not found: {0}")]
    ModelNotFound(String),

    /// Failed to load a GGUF model file.
    #[error("failed to load model: {0}")]
    LoadError(String),

    /// An error occurred during text generation.
    #[error("generation error: {0}")]
    GenerationError(String),

    /// An I/O error occurred (e.g., reading model files).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
