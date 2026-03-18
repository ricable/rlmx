use std::collections::HashMap;
use std::time::Instant;

use thiserror::Error;
use tracing::{debug, info};

use crate::config::{TrmConfig, TrmInput, TrmModelConfig, TrmResult};
use crate::model::TrmModel;

#[derive(Debug, Error)]
pub enum TrmStrategyError {
    #[error("model '{0}' not found in strategy registry")]
    ModelNotFound(String),

    #[error("input dimension mismatch: model expects {expected}, got {got}")]
    DimensionMismatch { expected: usize, got: usize },
}

/// The TRM Scheduling Policy.
///
/// Holds one or more TRM models and orchestrates recursive refinement
/// over the three streams (x, y, z) until a halting condition is met.
pub struct TrmStrategy {
    /// Registered models keyed by name.
    models: HashMap<String, TrmModel>,
    /// Default maximum refinement cycles.
    pub default_max_cycles: usize,
    /// Default confidence halt threshold.
    pub default_halt_threshold: f64,
}

impl TrmStrategy {
    /// Create a new strategy with no registered models.
    pub fn new(default_max_cycles: usize, default_halt_threshold: f64) -> Self {
        Self {
            models: HashMap::new(),
            default_max_cycles,
            default_halt_threshold,
        }
    }

    /// Register a model in the strategy.
    pub fn register_model(&mut self, config: TrmModelConfig) {
        let name = config.name.clone();
        let model = TrmModel::new(config);
        info!(model = %name, params = model.params, "registered TRM model");
        self.models.insert(name, model);
    }

    /// Execute a TRM classification run.
    ///
    /// Selects the model specified in `config`, builds the three streams from
    /// `input`, runs up to K refinement cycles, and returns the result.
    pub fn execute(
        &self,
        input: TrmInput,
        config: TrmConfig,
    ) -> Result<TrmResult, TrmStrategyError> {
        let model = self
            .models
            .get(&config.model_name)
            .ok_or_else(|| TrmStrategyError::ModelNotFound(config.model_name.clone()))?;

        if input.values.len() != model.input_dim {
            return Err(TrmStrategyError::DimensionMismatch {
                expected: model.input_dim,
                got: input.values.len(),
            });
        }

        let start = Instant::now();

        debug!(
            model = %config.model_name,
            input_len = input.values.len(),
            "starting TRM classification"
        );

        let classification = model.classify(&input.values);

        let latency_us = start.elapsed().as_micros() as u64;

        info!(
            model = %config.model_name,
            class = classification.class_index,
            confidence = %format!("{:.4}", classification.confidence),
            cycles = classification.cycles,
            latency_us = latency_us,
            "TRM classification complete"
        );

        Ok(TrmResult {
            prediction: classification.class_index,
            class_name: input.label.clone(),
            confidence: classification.confidence,
            probabilities: classification.probabilities,
            cycles_used: classification.cycles,
            latency_us,
            convergence_history: Vec::new(), // filled by detailed runs
        })
    }

    /// Convenience: execute with detailed convergence tracking.
    pub fn execute_with_history(
        &self,
        input: TrmInput,
        config: TrmConfig,
    ) -> Result<TrmResult, TrmStrategyError> {
        use crate::halting::AdaptiveHalter;
        use crate::streams::TrmStreams;

        let model = self
            .models
            .get(&config.model_name)
            .ok_or_else(|| TrmStrategyError::ModelNotFound(config.model_name.clone()))?;

        if input.values.len() != model.input_dim {
            return Err(TrmStrategyError::DimensionMismatch {
                expected: model.input_dim,
                got: input.values.len(),
            });
        }

        let start = Instant::now();
        let latent_dim = model.input_dim; // default
        let mut streams = TrmStreams::new(&input.values, model.output_classes, latent_dim);
        let mut halter = AdaptiveHalter::new();

        let mut cycles = 0;
        for cycle in 0..config.halt_config.max_cycles {
            streams.refine(model);
            cycles = cycle + 1;
            if halter.should_halt(&streams, cycle, &config.halt_config) {
                break;
            }
        }

        let latency_us = start.elapsed().as_micros() as u64;

        Ok(TrmResult {
            prediction: streams.prediction(),
            class_name: input.label.clone(),
            confidence: streams.confidence(),
            probabilities: streams.probabilities(),
            cycles_used: cycles,
            latency_us,
            convergence_history: halter.convergence_history(),
        })
    }
}

impl Default for TrmStrategy {
    fn default() -> Self {
        Self::new(10, 0.95)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{HaltConfig, TrmConfig, TrmInput, TrmModelConfig};

    fn setup_strategy() -> TrmStrategy {
        let mut strategy = TrmStrategy::default();
        strategy.register_model(TrmModelConfig {
            name: "test-model".to_string(),
            input_dim: 4,
            output_classes: 3,
            latent_dim: 4,
            max_cycles: 10,
            halt_threshold: 0.95,
            ..TrmModelConfig::default()
        });
        strategy
    }

    #[test]
    fn test_execute_end_to_end() {
        let strategy = setup_strategy();
        let input = TrmInput {
            values: vec![1.0, 0.5, -0.2, 0.8],
            label: Some("test".to_string()),
        };
        let config = TrmConfig {
            model_name: "test-model".to_string(),
            ..TrmConfig::default()
        };

        let result = strategy.execute(input, config).expect("execute should succeed");
        assert!(result.prediction < 3);
        assert!(result.confidence > 0.0 && result.confidence <= 1.0);
        assert_eq!(result.probabilities.len(), 3);
        assert!(result.cycles_used >= 1);
    }

    #[test]
    fn test_execute_with_convergence_history() {
        let strategy = setup_strategy();
        let input = TrmInput {
            values: vec![0.3, -0.1, 0.7, 0.4],
            label: None,
        };
        let config = TrmConfig {
            model_name: "test-model".to_string(),
            halt_config: HaltConfig {
                confidence_threshold: 0.99,
                max_cycles: 5,
                convergence_epsilon: 1e-8,
            },
            init_z_from_patterns: false,
        };

        let result = strategy
            .execute_with_history(input, config)
            .expect("execute_with_history should succeed");

        // convergence_history should have one entry per cycle
        assert!(!result.convergence_history.is_empty());
        assert!(result.convergence_history.len() <= 5);
        // Each entry should be a valid confidence
        for &c in &result.convergence_history {
            assert!(c >= 0.0 && c <= 1.0);
        }
    }
}
