//! Local inference engine.
//!
//! When compiled without the `ruvllm` feature, all methods return
//! `RuvllmError::NotAvailable`. When compiled with the feature,
//! the actual ruvllm CandleBackend is used for GGUF model inference.

#[cfg(feature = "ruvllm")]
use ruvllm::LlmBackend;
#[cfg(feature = "ruvllm")]
use std::sync::Mutex;
#[cfg(feature = "ruvllm")]
use std::time::Instant;

use serde::{Deserialize, Serialize};
use tracing::info;

use crate::config::{EdgeConfig, HardwareBackend, ModelSpec};
use crate::error::RuvllmError;

/// Result of a text generation call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResult {
    /// The generated text.
    pub text: String,
    /// Number of tokens generated.
    pub tokens_generated: u32,
    /// Wall-clock latency in milliseconds.
    pub latency_ms: u64,
    /// Confidence score for the generated output (0.0–1.0).
    pub confidence: f64,
}

/// A chat message (role + content), independent of rlmx-rlm types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// The role: "system", "user", or "assistant".
    pub role: String,
    /// The message content.
    pub content: String,
}

/// The local LLM inference engine.
///
/// Wraps ruvllm's `CandleBackend` behind a feature gate. Without the
/// `ruvllm` feature, this is a stub that reports the engine as unavailable.
pub struct LocalEngine {
    config: EdgeConfig,
    loaded: bool,
    #[cfg(feature = "ruvllm")]
    backend: Mutex<ruvllm::CandleBackend>,
}

impl LocalEngine {
    /// Create a new local inference engine with the given configuration.
    pub fn new(config: EdgeConfig) -> Result<Self, RuvllmError> {
        info!(
            model = %config.model.name,
            backend = %config.backend,
            "Creating local inference engine"
        );

        #[cfg(feature = "ruvllm")]
        {
            let device_type = match &config.backend {
                HardwareBackend::Cpu => ruvllm::DeviceType::Cpu,
                HardwareBackend::Metal => ruvllm::DeviceType::Metal,
                HardwareBackend::Cuda => ruvllm::DeviceType::Cuda(0),
                HardwareBackend::Auto | HardwareBackend::WebGpu => {
                    // Auto-detect: try Metal first (macOS), fall back to CPU
                    if cfg!(target_os = "macos") {
                        ruvllm::DeviceType::Metal
                    } else {
                        ruvllm::DeviceType::Cpu
                    }
                }
            };

            let backend = ruvllm::CandleBackend::with_device(device_type)
                .map_err(|e| RuvllmError::LoadError(format!("Failed to create backend: {}", e)))?;

            Ok(Self {
                config,
                loaded: false,
                backend: Mutex::new(backend),
            })
        }

        #[cfg(not(feature = "ruvllm"))]
        {
            Ok(Self {
                config,
                loaded: false,
            })
        }
    }

    /// Load the configured model into memory.
    #[cfg(not(feature = "ruvllm"))]
    pub fn load(&mut self) -> Result<(), RuvllmError> {
        Err(RuvllmError::NotAvailable)
    }

    /// Load the configured model into memory (with ruvllm feature).
    #[cfg(feature = "ruvllm")]
    pub fn load(&mut self) -> Result<(), RuvllmError> {
        if !self.config.model.path.exists() {
            return Err(RuvllmError::ModelNotFound(
                self.config.model.path.display().to_string(),
            ));
        }

        let model_path = self.config.model.path.display().to_string();

        // Determine architecture from model name heuristics.
        let arch = detect_architecture(&self.config.model.name);

        let model_config = ruvllm::ModelConfig {
            architecture: arch,
            quantization: Some(ruvllm::Quantization::Q4K),
            use_flash_attention: false,
            max_sequence_length: self.config.model.context_length,
            device: if cfg!(target_os = "macos") {
                ruvllm::DeviceType::Metal
            } else {
                ruvllm::DeviceType::Cpu
            },
            dtype: ruvllm::DType::F32,
            ..Default::default()
        };

        let mut backend = self
            .backend
            .lock()
            .map_err(|e| RuvllmError::LoadError(format!("Lock error: {}", e)))?;

        backend
            .load_model(&model_path, model_config)
            .map_err(|e| RuvllmError::LoadError(format!("{}", e)))?;

        // Try to load a tokenizer from a companion file next to the GGUF.
        // Look for: <model-dir>/tokenizer.json or <model-stem>-tokenizer.json
        let model_dir = self
            .config
            .model
            .path
            .parent()
            .unwrap_or(std::path::Path::new("."));
        let stem = self
            .config
            .model
            .path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("model");

        let tokenizer_candidates = [
            model_dir.join(format!("{}-tokenizer.json", stem)),
            model_dir.join("tokenizer.json"),
        ];

        for tok_path in &tokenizer_candidates {
            if tok_path.exists() {
                info!(path = %tok_path.display(), "Loading tokenizer");
                if let Err(e) = backend.load_tokenizer(tok_path) {
                    tracing::warn!(
                        "Failed to load tokenizer from {}: {}",
                        tok_path.display(),
                        e
                    );
                } else {
                    info!("Tokenizer loaded successfully");
                    break;
                }
            }
        }

        self.loaded = true;
        info!(model = %self.config.model.name, "Model loaded successfully");
        Ok(())
    }

    /// Generate text from a prompt.
    pub async fn generate(
        &self,
        prompt: &str,
        max_tokens: u32,
    ) -> Result<GenerateResult, RuvllmError> {
        #[cfg(not(feature = "ruvllm"))]
        {
            let _ = (prompt, max_tokens);
            Err(RuvllmError::NotAvailable)
        }

        #[cfg(feature = "ruvllm")]
        {
            if !self.loaded {
                return Err(RuvllmError::NoModelLoaded);
            }

            let start = Instant::now();

            let params = ruvllm::GenerateParams::default()
                .with_max_tokens(max_tokens as usize)
                .with_temperature(self.config.temperature as f32);

            let backend = self
                .backend
                .lock()
                .map_err(|e| RuvllmError::GenerationError(format!("Lock error: {}", e)))?;

            let text = backend
                .generate(prompt, params)
                .map_err(|e| RuvllmError::GenerationError(format!("{}", e)))?;

            let elapsed = start.elapsed();
            let tokens_generated = text.split_whitespace().count() as u32; // approximate

            // Compute confidence from output length ratio: how much of
            // max_tokens was actually generated. Short outputs relative to
            // the budget suggest low model confidence; near-full outputs
            // suggest the model had enough to say.
            let length_ratio = if max_tokens > 0 {
                (tokens_generated as f64 / max_tokens as f64).min(1.0)
            } else {
                0.0
            };
            // Map ratio to a confidence range [0.3, 0.95] to avoid
            // extremes that would bypass or always trigger escalation.
            let confidence = 0.3 + length_ratio * 0.65;

            Ok(GenerateResult {
                text,
                tokens_generated,
                latency_ms: elapsed.as_millis() as u64,
                confidence,
            })
        }
    }

    /// Chat completion from a list of messages.
    pub async fn chat(
        &self,
        messages: &[ChatMessage],
        max_tokens: u32,
    ) -> Result<String, RuvllmError> {
        #[cfg(not(feature = "ruvllm"))]
        {
            let _ = (messages, max_tokens);
            Err(RuvllmError::NotAvailable)
        }

        #[cfg(feature = "ruvllm")]
        {
            if !self.loaded {
                return Err(RuvllmError::NoModelLoaded);
            }
            // Format messages as a simple prompt.
            let prompt: String = messages
                .iter()
                .map(|m| format!("{}: {}", m.role, m.content))
                .collect::<Vec<_>>()
                .join("\n");
            let result = self.generate(&prompt, max_tokens).await?;
            Ok(result.text)
        }
    }

    /// Get the currently loaded model specification.
    pub fn model_info(&self) -> &ModelSpec {
        &self.config.model
    }

    /// Whether a model is currently loaded and ready for inference.
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// Estimated memory usage in megabytes.
    pub fn memory_usage_mb(&self) -> usize {
        if self.loaded {
            self.config.max_memory_mb
        } else {
            0
        }
    }

    /// Get the configured hardware backend.
    pub fn backend(&self) -> &HardwareBackend {
        &self.config.backend
    }

    /// Get the full engine configuration.
    pub fn config(&self) -> &EdgeConfig {
        &self.config
    }
}

/// Detect model architecture from the model name.
#[cfg(feature = "ruvllm")]
fn detect_architecture(name: &str) -> ruvllm::ModelArchitecture {
    let lower = name.to_lowercase();
    if lower.contains("qwen") {
        ruvllm::ModelArchitecture::Qwen
    } else if lower.contains("llama") {
        ruvllm::ModelArchitecture::Llama
    } else if lower.contains("mistral") {
        ruvllm::ModelArchitecture::Mistral
    } else if lower.contains("phi3") || lower.contains("phi-3") {
        ruvllm::ModelArchitecture::Phi3
    } else if lower.contains("phi") {
        ruvllm::ModelArchitecture::Phi
    } else if lower.contains("gemma2") || lower.contains("gemma-2") {
        ruvllm::ModelArchitecture::Gemma2
    } else if lower.contains("gemma") {
        ruvllm::ModelArchitecture::Gemma
    } else {
        // Default to Llama — most compatible
        ruvllm::ModelArchitecture::Llama
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::EdgeConfig;

    #[test]
    fn test_engine_creation() {
        let config = EdgeConfig::default();
        let engine = LocalEngine::new(config).unwrap();
        assert!(!engine.is_loaded());
        assert_eq!(engine.memory_usage_mb(), 0);
    }

    #[tokio::test]
    async fn test_generate_without_feature() {
        let config = EdgeConfig::default();
        let engine = LocalEngine::new(config).unwrap();
        let result = engine.generate("test", 10).await;
        #[cfg(not(feature = "ruvllm"))]
        assert!(matches!(result, Err(RuvllmError::NotAvailable)));
        #[cfg(feature = "ruvllm")]
        assert!(matches!(result, Err(RuvllmError::NoModelLoaded)));
    }

    #[test]
    fn test_model_info() {
        let config = EdgeConfig::default();
        let engine = LocalEngine::new(config).unwrap();
        assert_eq!(engine.model_info().name, "none");
    }
}
