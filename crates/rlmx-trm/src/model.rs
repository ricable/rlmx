use std::path::Path;

use ndarray::{Array1, Array2};
use rand::Rng;
use thiserror::Error;

use crate::config::{HaltConfig, TrmClassification, TrmModelConfig};
use crate::halting::AdaptiveHalter;
use crate::streams::TrmStreams;

#[derive(Debug, Error)]
pub enum TrmModelError {
    #[error("failed to load model from {path}: {reason}")]
    LoadError { path: String, reason: String },

    #[error("dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch { expected: usize, got: usize },
}

/// A simple 2-layer neural network that performs recursive refinement.
///
/// Layer 1 maps the concatenated `[x, y, z]` to a hidden representation.
/// Layer 2 maps the hidden representation to new `y` logits.
/// A separate set of weights (`wz`, `bz`) refines the `z` stream.
pub struct TrmModel {
    pub name: String,
    pub params: usize,
    pub layers: usize,
    pub input_dim: usize,
    pub output_classes: usize,
    pub max_cycles: usize,
    pub halt_threshold: f64,

    // Layer 1: maps concat(x, y, z) -> hidden
    w1: Array2<f64>,
    b1: Array1<f64>,

    // Layer 2: maps hidden -> y logits
    w2: Array2<f64>,
    b2: Array1<f64>,

    // Refinement weights for z stream: maps hidden -> new z
    wz: Array2<f64>,
    bz: Array1<f64>,
}

impl std::fmt::Debug for TrmModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrmModel")
            .field("name", &self.name)
            .field("params", &self.params)
            .field("layers", &self.layers)
            .field("input_dim", &self.input_dim)
            .field("output_classes", &self.output_classes)
            .finish()
    }
}

impl TrmModel {
    /// Initialise the model with random (Xavier-ish) weights.
    pub fn new(config: TrmModelConfig) -> Self {
        let latent_dim = config.effective_latent_dim();
        let concat_dim = config.input_dim + config.output_classes + latent_dim;
        let hidden_dim = concat_dim; // keep it simple: hidden = concat width

        let mut rng = rand::thread_rng();
        let scale1 = (2.0 / concat_dim as f64).sqrt();
        let scale2 = (2.0 / hidden_dim as f64).sqrt();

        let w1 = Array2::from_shape_fn((concat_dim, hidden_dim), |_| {
            rng.gen_range(-scale1..scale1)
        });
        let b1 = Array1::zeros(hidden_dim);

        let w2 = Array2::from_shape_fn((hidden_dim, config.output_classes), |_| {
            rng.gen_range(-scale2..scale2)
        });
        let b2 = Array1::zeros(config.output_classes);

        let wz =
            Array2::from_shape_fn((hidden_dim, latent_dim), |_| rng.gen_range(-scale2..scale2));
        let bz = Array1::zeros(latent_dim);

        let params = w1.len() + b1.len() + w2.len() + b2.len() + wz.len() + bz.len();

        Self {
            name: config.name,
            params,
            layers: config.layers,
            input_dim: config.input_dim,
            output_classes: config.output_classes,
            max_cycles: config.max_cycles,
            halt_threshold: config.halt_threshold,
            w1,
            b1,
            w2,
            b2,
            wz,
            bz,
        }
    }

    /// Stub for loading weights from a file (ONNX support placeholder).
    pub fn load(path: &Path) -> Result<Self, TrmModelError> {
        Err(TrmModelError::LoadError {
            path: path.display().to_string(),
            reason: "ONNX loading not yet implemented; use TrmModel::new() for random init"
                .to_string(),
        })
    }

    /// Execute one refinement cycle: (x, y, z) -> (new_y, new_z).
    ///
    /// 1. Concatenate x, y, z into a single vector.
    /// 2. Hidden = ReLU(concat . W1 + b1).
    /// 3. new_y = softmax(hidden . W2 + b2).
    /// 4. new_z = ReLU(hidden . Wz + bz).
    pub fn forward(&self, x: &Array1<f64>, y: &Array1<f64>, z: &Array1<f64>) -> (Array1<f64>, Array1<f64>) {
        // Concatenate
        let concat = concatenate(x, y, z);

        // Layer 1
        let hidden_raw = concat.dot(&self.w1) + &self.b1;
        let hidden = relu(&hidden_raw);

        // Layer 2 -> y logits -> softmax
        let y_logits = hidden.dot(&self.w2) + &self.b2;
        let new_y = softmax(&y_logits);

        // z refinement
        let z_raw = hidden.dot(&self.wz) + &self.bz;
        let new_z = relu(&z_raw);

        (new_y, new_z)
    }

    /// Run the full recursive classification pipeline with adaptive halting.
    pub fn classify(&self, input: &[f64]) -> TrmClassification {
        let latent_dim = self.wz.ncols();
        let mut streams = TrmStreams::new(input, self.output_classes, latent_dim);
        let mut halter = AdaptiveHalter::new();

        let halt_config = HaltConfig {
            confidence_threshold: self.halt_threshold,
            max_cycles: self.max_cycles,
            convergence_epsilon: 1e-4,
        };

        let mut cycles = 0;
        for cycle in 0..self.max_cycles {
            streams.refine(self);
            cycles = cycle + 1;
            if halter.should_halt(&streams, cycle, &halt_config) {
                break;
            }
        }

        TrmClassification {
            class_index: streams.prediction(),
            confidence: streams.confidence(),
            probabilities: streams.probabilities(),
            cycles,
        }
    }
}

// ── Activation helpers ──────────────────────────────────────────────────────

fn relu(a: &Array1<f64>) -> Array1<f64> {
    a.mapv(|v| v.max(0.0))
}

fn softmax(logits: &Array1<f64>) -> Array1<f64> {
    let max_val = logits.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let exps = logits.mapv(|v| (v - max_val).exp());
    let sum: f64 = exps.sum();
    if sum > 0.0 {
        exps / sum
    } else {
        Array1::from_elem(logits.len(), 1.0 / logits.len() as f64)
    }
}

fn concatenate(x: &Array1<f64>, y: &Array1<f64>, z: &Array1<f64>) -> Array1<f64> {
    let mut out = Vec::with_capacity(x.len() + y.len() + z.len());
    out.extend(x.iter());
    out.extend(y.iter());
    out.extend(z.iter());
    Array1::from_vec(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TrmModelConfig;

    fn default_model() -> TrmModel {
        let cfg = TrmModelConfig {
            input_dim: 8,
            output_classes: 4,
            latent_dim: 8,
            max_cycles: 10,
            halt_threshold: 0.95,
            ..TrmModelConfig::default()
        };
        TrmModel::new(cfg)
    }

    #[test]
    fn test_forward_produces_valid_output() {
        let model = default_model();
        let x = Array1::from_vec(vec![0.1; 8]);
        let y = Array1::from_elem(4, 0.25);
        let z = Array1::zeros(8);

        let (new_y, new_z) = model.forward(&x, &y, &z);

        // new_y should have correct length
        assert_eq!(new_y.len(), 4);
        // new_z should have correct length
        assert_eq!(new_z.len(), 8);
        // new_y should be a valid probability distribution (softmax)
        assert!(new_y.iter().all(|&v| v >= 0.0));
        // new_z values should be >= 0 (ReLU)
        assert!(new_z.iter().all(|&v| v >= 0.0));
    }

    #[test]
    fn test_softmax_sums_to_one() {
        let logits = Array1::from_vec(vec![2.0, 1.0, 0.1, -1.0, 3.5]);
        let probs = softmax(&logits);
        let sum: f64 = probs.sum();
        assert!(
            (sum - 1.0).abs() < 1e-9,
            "softmax should sum to 1.0, got {sum}"
        );
    }

    #[test]
    fn test_full_classification_pipeline() {
        let model = default_model();
        let input = vec![0.5, -0.3, 1.2, 0.0, 0.8, -0.1, 0.4, 0.6];
        let result = model.classify(&input);

        assert!(result.class_index < 4);
        assert!(result.confidence > 0.0);
        assert!(result.confidence <= 1.0);
        assert_eq!(result.probabilities.len(), 4);
        assert!(result.cycles >= 1);
        assert!(result.cycles <= 10);

        let prob_sum: f64 = result.probabilities.iter().sum();
        assert!(
            (prob_sum - 1.0).abs() < 1e-6,
            "probabilities should sum to 1.0, got {prob_sum}"
        );
    }
}
