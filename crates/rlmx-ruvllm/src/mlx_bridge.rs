//! MLX subprocess bridge for Apple Silicon inference.
//!
//! Wraps Python MLX via `tokio::process` for Medium tier models.
//! When the MLX Python runtime is not available, all operations
//! return `RuvllmError::NotAvailable`.

use serde::{Deserialize, Serialize};

use crate::error::RuvllmError;

/// MLX subprocess bridge for Apple Silicon inference.
///
/// Wraps Python MLX via `tokio::process` for Medium tier models.
pub struct MlxSubprocess {
    model_name: String,
    available: bool,
}

impl MlxSubprocess {
    /// Create a new MLX bridge for the given model.
    ///
    /// The bridge starts in an unavailable state; call [`spawn`](Self::spawn)
    /// to probe for the Python MLX runtime.
    pub fn new(model_name: &str) -> Self {
        Self {
            model_name: model_name.to_string(),
            available: false,
        }
    }

    /// Check if the MLX Python runtime is available.
    pub fn is_available(&self) -> bool {
        self.available
    }

    /// Get the model name this bridge was configured for.
    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    /// Attempt to spawn the MLX Python subprocess.
    ///
    /// Checks for `python3` and `mlx` availability. If found, marks
    /// this bridge as available for generation requests.
    pub async fn spawn(&mut self) -> Result<(), RuvllmError> {
        match tokio::process::Command::new("python3")
            .arg("-c")
            .arg("import mlx")
            .output()
            .await
        {
            Ok(output) if output.status.success() => {
                self.available = true;
                Ok(())
            }
            _ => {
                self.available = false;
                Err(RuvllmError::NotAvailable)
            }
        }
    }

    /// Generate text using MLX.
    ///
    /// Returns `RuvllmError::NotAvailable` when the MLX runtime is not present.
    /// When available, this is currently a stub that returns an empty response.
    pub async fn generate(
        &self,
        _prompt: &str,
        _max_tokens: usize,
    ) -> Result<MlxResponse, RuvllmError> {
        if !self.available {
            return Err(RuvllmError::NotAvailable);
        }
        Ok(MlxResponse {
            text: String::new(),
            confidence: 0.0,
            tokens_generated: 0,
        })
    }
}

/// Response from an MLX generation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlxResponse {
    /// The generated text.
    pub text: String,
    /// Confidence score for the generated output (0.0-1.0).
    pub confidence: f64,
    /// Number of tokens generated.
    pub tokens_generated: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mlx_creation() {
        let mlx = MlxSubprocess::new("test-model");
        assert_eq!(mlx.model_name(), "test-model");
        assert!(!mlx.is_available());
    }

    #[tokio::test]
    async fn test_mlx_unavailable_generate() {
        let mlx = MlxSubprocess::new("test-model");
        let result = mlx.generate("hello", 100).await;
        assert!(result.is_err());
        match result {
            Err(RuvllmError::NotAvailable) => {} // expected
            other => panic!("expected NotAvailable, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_mlx_spawn_likely_unavailable() {
        // In most CI/test environments, MLX Python is not installed.
        // This test verifies that spawn handles the missing runtime gracefully.
        let mut mlx = MlxSubprocess::new("test-model");
        let result = mlx.spawn().await;
        // Either succeeds (MLX installed) or returns NotAvailable.
        match result {
            Ok(()) => assert!(mlx.is_available()),
            Err(RuvllmError::NotAvailable) => assert!(!mlx.is_available()),
            Err(e) => panic!("unexpected error: {:?}", e),
        }
    }
}
