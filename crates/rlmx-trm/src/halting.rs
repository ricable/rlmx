use serde::{Deserialize, Serialize};

use crate::config::HaltConfig;
use crate::streams::TrmStreams;

/// Describes which halting condition was triggered.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HaltCondition {
    /// The top-class confidence exceeded a threshold.
    ConfidenceThreshold(f64),
    /// The hard cycle limit was reached.
    MaxCycles(usize),
    /// The improvement between successive cycles fell below epsilon.
    ConvergenceThreshold(f64),
    /// A combination of conditions (confidence OR convergence OR max-cycles).
    Combined,
}

/// Absolute hard upper bound on refinement cycles, enforced regardless of
/// any configuration value.  This prevents runaway loops even when callers
/// supply a very large (or default-zero) `max_cycles`.
pub const HARD_CYCLE_LIMIT: usize = 1_000;

/// Tracks confidence history and decides when to stop refining.
#[derive(Debug, Clone)]
pub struct AdaptiveHalter {
    /// Per-cycle confidence values.
    pub history: Vec<f64>,
}

impl AdaptiveHalter {
    /// Create a new halter with an empty history.
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
        }
    }

    /// Record the current confidence and return whether we should halt.
    ///
    /// Halting rules (evaluated in order):
    /// 0. `cycle >= HARD_CYCLE_LIMIT` (absolute safety ceiling)
    /// 1. `confidence >= config.confidence_threshold`
    /// 2. `cycle >= config.max_cycles`
    /// 3. Improvement from the previous cycle < `config.convergence_epsilon`
    pub fn should_halt(
        &mut self,
        streams: &TrmStreams,
        cycle: usize,
        config: &HaltConfig,
    ) -> bool {
        // Rule 0 -- hard safety ceiling (always enforced)
        if cycle + 1 >= HARD_CYCLE_LIMIT {
            return true;
        }

        let confidence = streams.confidence();
        self.history.push(confidence);

        // Rule 1 -- confidence threshold
        if confidence >= config.confidence_threshold {
            return true;
        }

        // Rule 2 -- max cycles (1-indexed cycle count)
        if cycle + 1 >= config.max_cycles {
            return true;
        }

        // Rule 3 -- convergence (need at least two data points)
        if self.history.len() >= 2 {
            let prev = self.history[self.history.len() - 2];
            let improvement = (confidence - prev).abs();
            if improvement < config.convergence_epsilon {
                return true;
            }
        }

        false
    }

    /// Return a snapshot of the convergence history.
    pub fn convergence_history(&self) -> Vec<f64> {
        self.history.clone()
    }
}

impl Default for AdaptiveHalter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streams::TrmStreams;

    fn make_streams_with_confidence(confidence: f64, num_classes: usize) -> TrmStreams {
        // Build a y vector whose max equals `confidence` and the rest share (1 - confidence).
        let mut y = ndarray::Array1::zeros(num_classes);
        if num_classes > 0 {
            y[0] = confidence;
            let rest = (1.0 - confidence) / (num_classes as f64 - 1.0).max(1.0);
            for i in 1..num_classes {
                y[i] = rest;
            }
        }
        TrmStreams {
            x: ndarray::Array1::zeros(4),
            y,
            z: ndarray::Array1::zeros(4),
        }
    }

    #[test]
    fn test_halt_at_confidence_threshold() {
        let mut halter = AdaptiveHalter::new();
        let config = HaltConfig {
            confidence_threshold: 0.90,
            max_cycles: 100,
            convergence_epsilon: 1e-6,
        };
        let streams = make_streams_with_confidence(0.95, 4);
        assert!(halter.should_halt(&streams, 0, &config));
    }

    #[test]
    fn test_halt_at_max_cycles() {
        let mut halter = AdaptiveHalter::new();
        let config = HaltConfig {
            confidence_threshold: 0.99,
            max_cycles: 3,
            convergence_epsilon: 1e-8,
        };
        let streams = make_streams_with_confidence(0.5, 4);
        // Cycles 0 and 1 should not halt (confidence low, cycles remaining)
        assert!(!halter.should_halt(&streams, 0, &config));
        // Manually push different confidence to avoid convergence halt
        halter.history.clear();
        halter.history.push(0.40);
        assert!(!halter.should_halt(&streams, 1, &config));
        // Cycle 2 is the third cycle (0-indexed), max_cycles=3 => halt
        halter.history.clear();
        halter.history.push(0.30);
        halter.history.push(0.40);
        assert!(halter.should_halt(&streams, 2, &config));
    }

    #[test]
    fn test_convergence_history_tracking() {
        let mut halter = AdaptiveHalter::new();
        let config = HaltConfig {
            confidence_threshold: 0.99,
            max_cycles: 100,
            convergence_epsilon: 1e-8,
        };

        let s1 = make_streams_with_confidence(0.30, 4);
        let s2 = make_streams_with_confidence(0.50, 4);
        let s3 = make_streams_with_confidence(0.70, 4);

        halter.should_halt(&s1, 0, &config);
        halter.should_halt(&s2, 1, &config);
        halter.should_halt(&s3, 2, &config);

        let history = halter.convergence_history();
        assert_eq!(history.len(), 3);
        assert!((history[0] - 0.30).abs() < 1e-6);
        assert!((history[1] - 0.50).abs() < 1e-6);
        assert!((history[2] - 0.70).abs() < 1e-6);
    }
}
