use ndarray::Array1;

use crate::model::TrmModel;

/// The three-stream state used by TRM recursive refinement.
///
/// Following the TRM paper:
/// - **x**: Question / input stream (fixed throughout refinement).
/// - **y**: Answer stream (refined each cycle; probability distribution over classes).
/// - **z**: Latent reasoning stream (refined each cycle; encodes implicit reasoning).
#[derive(Debug, Clone)]
pub struct TrmStreams {
    /// x: input features (immutable during refinement).
    pub x: Array1<f64>,
    /// y: answer distribution (refined each cycle).
    pub y: Array1<f64>,
    /// z: latent reasoning vector (refined each cycle).
    pub z: Array1<f64>,
}

/// Outcome of a single refinement step.
#[derive(Debug, Clone)]
pub struct RefinementResult {
    /// Updated y stream after this cycle.
    pub new_y: Array1<f64>,
    /// Updated z stream after this cycle.
    pub new_z: Array1<f64>,
    /// Confidence (max of y) after this cycle.
    pub confidence: f64,
}

impl TrmStreams {
    /// Initialise the three streams from a raw input vector.
    ///
    /// - `x` is set to the input (L2-normalised).
    /// - `y` is a uniform distribution of length `num_classes`.
    /// - `z` is zero-initialised with dimension `latent_dim`.
    pub fn new(input: &[f64], num_classes: usize, latent_dim: usize) -> Self {
        // Normalise x
        let x_raw = Array1::from_vec(input.to_vec());
        let norm = x_raw.dot(&x_raw).sqrt().max(1e-12);
        let x = &x_raw / norm;

        // Uniform y
        let y = Array1::from_elem(num_classes, 1.0 / num_classes as f64);

        // Zero z
        let z = Array1::zeros(latent_dim);

        Self { x, y, z }
    }

    /// Run one refinement cycle through the model, updating y and z in place.
    pub fn refine(&mut self, model: &TrmModel) -> RefinementResult {
        let (new_y, new_z) = model.forward(&self.x, &self.y, &self.z);
        self.y = new_y.clone();
        self.z = new_z.clone();

        let confidence = self.confidence();
        RefinementResult {
            new_y,
            new_z,
            confidence,
        }
    }

    /// The confidence is the maximum value in the answer stream y.
    pub fn confidence(&self) -> f64 {
        self.y.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
    }

    /// The predicted class is the argmax of y.
    pub fn prediction(&self) -> usize {
        self.y
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    /// Return a clone of y as a Vec.
    pub fn probabilities(&self) -> Vec<f64> {
        self.y.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TrmModelConfig;
    use crate::model::TrmModel;

    #[test]
    fn test_streams_initialization_shapes() {
        let input = vec![1.0, 2.0, 3.0, 4.0];
        let streams = TrmStreams::new(&input, 5, 4);

        assert_eq!(streams.x.len(), 4);
        assert_eq!(streams.y.len(), 5);
        assert_eq!(streams.z.len(), 4);

        // y should be uniform
        for &v in streams.y.iter() {
            assert!((v - 0.2).abs() < 1e-9);
        }

        // z should be zero
        for &v in streams.z.iter() {
            assert!((v - 0.0).abs() < 1e-9);
        }

        // x should be normalised
        let norm: f64 = streams.x.iter().map(|v| v * v).sum::<f64>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_refinement_changes_y_and_z() {
        let cfg = TrmModelConfig {
            input_dim: 4,
            output_classes: 3,
            latent_dim: 4,
            ..TrmModelConfig::default()
        };
        let model = TrmModel::new(cfg);
        let input = vec![1.0, 0.5, -0.3, 0.8];
        let mut streams = TrmStreams::new(&input, 3, 4);

        let y_before = streams.y.clone();
        let z_before = streams.z.clone();

        streams.refine(&model);

        // After one refinement, y and z should have changed
        let y_changed = streams
            .y
            .iter()
            .zip(y_before.iter())
            .any(|(a, b)| (a - b).abs() > 1e-12);
        assert!(y_changed, "y should change after refinement");

        let z_changed = streams
            .z
            .iter()
            .zip(z_before.iter())
            .any(|(a, b)| (a - b).abs() > 1e-12);
        assert!(z_changed, "z should change after refinement");
    }
}
