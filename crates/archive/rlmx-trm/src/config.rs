use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Configuration for initializing a TRM model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrmModelConfig {
    /// Human-readable model name.
    pub name: String,
    /// Optional path to serialized weights (ONNX stub).
    pub path: Option<PathBuf>,
    /// Total parameter count (informational).
    pub params: usize,
    /// Number of layers (currently fixed at 2).
    pub layers: usize,
    /// Dimension of the input feature vector (x stream).
    pub input_dim: usize,
    /// Number of output classes.
    pub output_classes: usize,
    /// Maximum refinement cycles before forced halt.
    pub max_cycles: usize,
    /// Confidence threshold for early halting.
    pub halt_threshold: f64,
    /// Dimension of the latent z stream. Defaults to `input_dim` when zero.
    pub latent_dim: usize,
}

impl Default for TrmModelConfig {
    fn default() -> Self {
        Self {
            name: "trm-default".to_string(),
            path: None,
            params: 0,
            layers: 2,
            input_dim: 16,
            output_classes: 4,
            max_cycles: 10,
            halt_threshold: 0.95,
            latent_dim: 0, // will be resolved to input_dim
        }
    }
}

impl TrmModelConfig {
    /// Returns the effective latent dimension (falls back to `input_dim`).
    pub fn effective_latent_dim(&self) -> usize {
        if self.latent_dim == 0 {
            self.input_dim
        } else {
            self.latent_dim
        }
    }
}

/// Runtime configuration for a single TRM execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrmConfig {
    /// Which model to use by name.
    pub model_name: String,
    /// Halting behaviour configuration.
    pub halt_config: HaltConfig,
    /// Whether to seed the z-stream from a pattern bank.
    pub init_z_from_patterns: bool,
}

impl Default for TrmConfig {
    fn default() -> Self {
        Self {
            model_name: "trm-default".to_string(),
            halt_config: HaltConfig::default(),
            init_z_from_patterns: false,
        }
    }
}

/// Configuration for the adaptive halting mechanism.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaltConfig {
    /// Confidence above which we halt.
    pub confidence_threshold: f64,
    /// Hard ceiling on cycles.
    pub max_cycles: usize,
    /// Minimum improvement between cycles to continue.
    pub convergence_epsilon: f64,
}

impl Default for HaltConfig {
    fn default() -> Self {
        Self {
            confidence_threshold: 0.95,
            max_cycles: 10,
            convergence_epsilon: 1e-4,
        }
    }
}

/// Input to a TRM classification run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrmInput {
    /// Raw feature values.
    pub values: Vec<f64>,
    /// Optional ground-truth label (for evaluation).
    pub label: Option<String>,
}

/// Result of a TRM classification run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrmResult {
    /// Predicted class index (argmax of probabilities).
    pub prediction: usize,
    /// Optional human-readable class name.
    pub class_name: Option<String>,
    /// Confidence of the prediction (max probability).
    pub confidence: f64,
    /// Full probability distribution over classes.
    pub probabilities: Vec<f64>,
    /// Number of refinement cycles actually executed.
    pub cycles_used: usize,
    /// Wall-clock latency in microseconds.
    pub latency_us: u64,
    /// Per-cycle confidence values for convergence analysis.
    pub convergence_history: Vec<f64>,
}

/// Output of a single model classification call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrmClassification {
    /// Predicted class index.
    pub class_index: usize,
    /// Confidence of the top class.
    pub confidence: f64,
    /// Full probability vector.
    pub probabilities: Vec<f64>,
    /// Number of cycles used.
    pub cycles: usize,
}
