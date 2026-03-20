//! Neural model router — FastGRNN-inspired tiny 2-layer network for strategy
//! selection. Runs in <1ms, suitable for inline routing decisions.

use std::collections::HashMap;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::scheduler::Strategy;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Feature vector fed into the router (14 dimensions).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterInput {
    pub query_length: usize,
    pub has_code: bool,
    pub is_question: bool,
    pub trigram_entropy: f64,
    pub edge_available: bool,
    pub node_load: f64,
    /// Fraction of Zone A nodes that are healthy (0.0--1.0).
    pub zone_a_available: f64,
    /// Fraction of Zone B nodes that are healthy (0.0--1.0).
    pub zone_b_available: f64,
    /// Estimated token count, normalized (raw_tokens / 2048).
    pub token_count_estimate: f64,
    /// Rolling success rate per strategy: [Rlm, Trm, Edge, Hybrid, Swarm].
    pub recent_strategy_success: [f64; 5],
}

/// Router decision including confidence, per-strategy scores, and inference latency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterOutput {
    pub strategy: Strategy,
    pub confidence: f64,
    pub scores: HashMap<String, f64>,
    /// Time taken for the forward pass, in nanoseconds.
    pub latency_ns: u64,
}

// Strategy labels in fixed order — indices must match weight columns.
const STRATEGY_LABELS: [&str; 5] = ["Rlm", "Trm", "Edge", "Hybrid", "Swarm"];

// ---------------------------------------------------------------------------
// TinyDancerRouter
// ---------------------------------------------------------------------------

/// A FastGRNN-inspired 2-layer neural network for strategy routing.
///
/// Layer 1: hidden = ReLU(W1 * input + b1)   (input_dim -> hidden_dim)
/// Layer 2: output = W2 * hidden + b2         (hidden_dim -> output_dim)
/// Decision: softmax(output) -> best strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TinyDancerRouter {
    /// Layer-1 weights: hidden_dim x input_dim
    pub weights1: Vec<Vec<f64>>,
    /// Layer-1 bias: hidden_dim
    pub bias1: Vec<f64>,
    /// Layer-2 weights: output_dim x hidden_dim
    pub weights2: Vec<Vec<f64>>,
    /// Layer-2 bias: output_dim
    pub bias2: Vec<f64>,
    pub input_dim: usize,
    pub hidden_dim: usize,
    pub output_dim: usize,
    /// Online learning buffer: (input, strategy_idx, reward).
    #[serde(default)]
    pub online_buffer: Vec<(RouterInput, usize, f64)>,
    /// Number of examples to accumulate before triggering auto-retrain.
    #[serde(default = "default_retrain_threshold")]
    pub retrain_threshold: usize,
}

fn default_retrain_threshold() -> usize {
    100
}

impl TinyDancerRouter {
    /// Create a router with sensible default weights for 14 inputs, 5 outputs.
    ///
    /// Inputs (14):
    ///   [query_length_norm, has_code, is_question, trigram_entropy,
    ///    edge_available, node_load, zone_a_available, zone_b_available,
    ///    token_count_estimate, success_rlm, success_trm, success_edge,
    ///    success_hybrid, success_swarm]
    /// Outputs: [Rlm, Trm, Edge, Hybrid, Swarm]
    pub fn new() -> Self {
        let input_dim = 14;
        let hidden_dim = 32;
        let output_dim = 5;

        // Layer 1: 32x14 — hand-tuned so defaults produce reasonable routing.
        // First 8 neurons have semantic roles; remaining 24 start near-zero.
        let mut weights1 = Vec::with_capacity(hidden_dim);

        // h0: short-question detector -> boosts Rlm
        weights1.push(vec![
            -0.3, 0.0, 0.8, -0.2, 0.0, -0.1, 0.0, 0.0, -0.1, 0.3, 0.0, 0.0, 0.0, 0.0,
        ]);
        // h1: code/complexity detector -> boosts Trm
        weights1.push(vec![
            0.5, 0.9, 0.0, 0.3, 0.0, 0.0, 0.0, 0.0, 0.2, 0.0, 0.3, 0.0, 0.0, 0.0,
        ]);
        // h2: edge feasibility -> boosts Edge
        weights1.push(vec![
            -0.2, 0.0, 0.0, -0.1, 0.9, -0.3, 0.0, 0.0, -0.2, 0.0, 0.0, 0.3, 0.0, 0.0,
        ]);
        // h3: high-entropy catch-all -> boosts Hybrid
        weights1.push(vec![
            0.2, 0.1, 0.0, 0.7, 0.0, 0.2, 0.0, 0.0, 0.1, 0.0, 0.0, 0.0, 0.3, 0.0,
        ]);
        // h4: high-load + multi-zone -> boosts Swarm
        weights1.push(vec![
            0.1, 0.0, 0.0, 0.1, 0.0, 0.8, 0.3, 0.3, 0.1, 0.0, 0.0, 0.0, 0.0, 0.3,
        ]);
        // h5: zone-a health detector
        weights1.push(vec![
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.6, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ]);
        // h6: zone-b health detector
        weights1.push(vec![
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.6, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ]);
        // h7: token-count detector
        weights1.push(vec![
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.7, 0.0, 0.0, 0.0, 0.0, 0.0,
        ]);
        // h8-h31: near-zero initialization for learned features
        for _ in 8..hidden_dim {
            weights1.push(vec![0.01; input_dim]);
        }

        let mut bias1 = vec![0.0; hidden_dim];
        bias1[0] = 0.1; // slight bias for question detector
        bias1[5] = 0.1; // slight bias for zone-a detector

        // Layer 2: 5x32 — maps hidden activations to strategy logits.
        let mut weights2 = Vec::with_capacity(output_dim);

        // Rlm: relies on h0 (question), h5 (zone-a)
        let mut rlm_w = vec![0.0; hidden_dim];
        rlm_w[0] = 0.8;
        rlm_w[5] = 0.1;
        weights2.push(rlm_w);

        // Trm: relies on h1 (code), h7 (tokens)
        let mut trm_w = vec![0.0; hidden_dim];
        trm_w[1] = 0.8;
        trm_w[7] = 0.2;
        weights2.push(trm_w);

        // Edge: relies on h2 (edge feasibility)
        let mut edge_w = vec![0.0; hidden_dim];
        edge_w[2] = 0.9;
        weights2.push(edge_w);

        // Hybrid: relies on h3 (entropy)
        let mut hybrid_w = vec![0.0; hidden_dim];
        hybrid_w[3] = 0.7;
        hybrid_w[5] = 0.1;
        hybrid_w[6] = 0.1;
        weights2.push(hybrid_w);

        // Swarm: relies on h4 (load + zones), h5 (zone-a), h6 (zone-b)
        let mut swarm_w = vec![0.0; hidden_dim];
        swarm_w[4] = 0.9;
        swarm_w[5] = 0.1;
        swarm_w[6] = 0.1;
        weights2.push(swarm_w);

        let bias2 = vec![0.0; output_dim];

        Self {
            weights1,
            bias1,
            weights2,
            bias2,
            input_dim,
            hidden_dim,
            output_dim,
            online_buffer: Vec::new(),
            retrain_threshold: 100,
        }
    }

    /// Run the forward pass and return the best strategy with confidence.
    ///
    /// Measures wall-clock inference time and stores it in `latency_ns`.
    pub fn route(&self, input: &RouterInput) -> RouterOutput {
        let start = Instant::now();
        let features = Self::input_to_vec(input);

        // Layer 1: hidden = ReLU(W1 * x + b1)
        let mut hidden = vec![0.0; self.hidden_dim];
        for (i, row) in self.weights1.iter().enumerate() {
            let mut sum = self.bias1[i];
            for (j, &w) in row.iter().enumerate() {
                sum += w * features[j];
            }
            hidden[i] = sum.max(0.0); // ReLU
        }

        // Layer 2: logits = W2 * hidden + b2
        let mut logits = vec![0.0; self.output_dim];
        for (i, row) in self.weights2.iter().enumerate() {
            let mut sum = self.bias2[i];
            for (j, &w) in row.iter().enumerate() {
                sum += w * hidden[j];
            }
            logits[i] = sum;
        }

        let probs = softmax(&logits);
        let elapsed = start.elapsed();

        // Build scores map and find best.
        let mut scores = HashMap::new();
        let mut best_idx = 0;
        let mut best_prob = 0.0_f64;
        for (i, &p) in probs.iter().enumerate() {
            scores.insert(STRATEGY_LABELS[i].to_string(), p);
            if p > best_prob {
                best_prob = p;
                best_idx = i;
            }
        }

        let strategy = idx_to_strategy(best_idx);

        debug!(
            strategy = STRATEGY_LABELS[best_idx],
            confidence = best_prob,
            latency_ns = elapsed.as_nanos() as u64,
            "TinyDancerRouter selected strategy"
        );

        RouterOutput {
            strategy,
            confidence: best_prob,
            scores,
            latency_ns: elapsed.as_nanos() as u64,
        }
    }

    /// Simple online learning from (input, chosen_strategy, reward) triples.
    ///
    /// Uses a REINFORCE-style gradient approximation: nudge the weights so that
    /// the chosen strategy's logit increases proportional to the reward, and
    /// competing logits decrease.
    pub fn train_from_history(&mut self, records: &[(RouterInput, Strategy, f64)]) {
        let lr = 0.01;

        for (input, target_strategy, reward) in records {
            let target_idx = strategy_to_idx(target_strategy);
            let features = Self::input_to_vec(input);

            // Forward pass to get hidden activations.
            let mut hidden = vec![0.0; self.hidden_dim];
            for (i, row) in self.weights1.iter().enumerate() {
                let mut sum = self.bias1[i];
                for (j, &w) in row.iter().enumerate() {
                    sum += w * features[j];
                }
                hidden[i] = sum.max(0.0);
            }

            let mut logits = vec![0.0; self.output_dim];
            for (i, row) in self.weights2.iter().enumerate() {
                let mut sum = self.bias2[i];
                for (j, &w) in row.iter().enumerate() {
                    sum += w * hidden[j];
                }
                logits[i] = sum;
            }

            let probs = softmax(&logits);

            // Gradient for layer 2: delta_i = reward * (1(i==target) - prob_i)
            for (i, prob) in probs.iter().enumerate().take(self.output_dim) {
                let indicator = if i == target_idx { 1.0 } else { 0.0 };
                let delta = reward * lr * (indicator - prob);
                self.bias2[i] += delta;
                for (j, h) in hidden.iter().enumerate().take(self.hidden_dim) {
                    self.weights2[i][j] += delta * h;
                }
            }

            // Backprop into layer 1 (simplified — only update if hidden > 0).
            for (j, h_val) in hidden.iter().enumerate().take(self.hidden_dim) {
                if *h_val <= 0.0 {
                    continue; // ReLU gate closed
                }
                let mut grad = 0.0;
                for (i, prob) in probs.iter().enumerate().take(self.output_dim) {
                    let indicator = if i == target_idx { 1.0 } else { 0.0 };
                    grad += (indicator - prob) * self.weights2[i][j];
                }
                grad *= reward * lr;

                self.bias1[j] += grad;
                for (k, feat) in features.iter().enumerate().take(self.input_dim) {
                    self.weights1[j][k] += grad * feat;
                }
            }
        }

        debug!(
            records = records.len(),
            "TinyDancerRouter updated weights from history"
        );
    }

    /// Record an execution outcome into the online buffer.
    ///
    /// When the buffer reaches `retrain_threshold`, automatically triggers
    /// `train_from_history()` over the buffered examples and clears the buffer.
    pub fn record_outcome(&mut self, input: RouterInput, strategy: &Strategy, reward: f64) {
        let strategy_idx = strategy_to_idx(strategy);
        self.online_buffer.push((input, strategy_idx, reward));

        if self.online_buffer.len() >= self.retrain_threshold {
            // Drain buffer into training records.
            let records: Vec<(RouterInput, Strategy, f64)> = self
                .online_buffer
                .drain(..)
                .map(|(inp, idx, r)| (inp, idx_to_strategy(idx), r))
                .collect();
            self.train_from_history(&records);
        }
    }

    /// Build a `RouterInput` from a query string and context flags.
    ///
    /// New fields default to: zone availability = 1.0, token estimate from
    /// whitespace-split word count / 2048, success rates = 0.5.
    pub fn extract_features(query: &str, edge_available: bool, node_load: f64) -> RouterInput {
        let has_code =
            query.contains("```") || query.contains("fn ") || query.contains("def ");
        let is_question = query.trim_end().ends_with('?');

        RouterInput {
            query_length: query.len(),
            has_code,
            is_question,
            trigram_entropy: trigram_entropy(query),
            edge_available,
            node_load,
            zone_a_available: 1.0,
            zone_b_available: 1.0,
            token_count_estimate: query.split_whitespace().count() as f64 / 2048.0,
            recent_strategy_success: [0.5; 5],
        }
    }

    /// Convert `RouterInput` to a fixed-size 14-element feature vector with
    /// normalization. Order must match the weight columns in `new()`.
    fn input_to_vec(input: &RouterInput) -> Vec<f64> {
        let mut v = Vec::with_capacity(14);
        // 0: Normalize query length: sigmoid-ish squash so 500 chars -> ~0.5
        v.push(1.0 - (-((input.query_length as f64) / 500.0)).exp());
        // 1: has_code
        v.push(if input.has_code { 1.0 } else { 0.0 });
        // 2: is_question
        v.push(if input.is_question { 1.0 } else { 0.0 });
        // 3: trigram_entropy normalized to [0,1]
        v.push(input.trigram_entropy.min(5.0) / 5.0);
        // 4: edge_available
        v.push(if input.edge_available { 1.0 } else { 0.0 });
        // 5: node_load
        v.push(input.node_load.clamp(0.0, 1.0));
        // 6: zone_a_available
        v.push(input.zone_a_available.clamp(0.0, 1.0));
        // 7: zone_b_available
        v.push(input.zone_b_available.clamp(0.0, 1.0));
        // 8: token_count_estimate (already normalized)
        v.push(input.token_count_estimate.clamp(0.0, 1.0));
        // 9-13: recent_strategy_success [Rlm, Trm, Edge, Hybrid, Swarm]
        for &s in &input.recent_strategy_success {
            v.push(s.clamp(0.0, 1.0));
        }
        v
    }
}

impl Default for TinyDancerRouter {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Numerically stable softmax over a slice of logits.
pub fn softmax(values: &[f64]) -> Vec<f64> {
    if values.is_empty() {
        return vec![];
    }
    let max_val = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let exps: Vec<f64> = values.iter().map(|&v| (v - max_val).exp()).collect();
    let sum: f64 = exps.iter().sum();
    exps.iter().map(|&e| e / sum).collect()
}

/// Compute trigram entropy of a string (Shannon entropy over trigram
/// frequencies). Higher entropy -> more diverse/complex text.
pub fn trigram_entropy(text: &str) -> f64 {
    if text.len() < 3 {
        return 0.0;
    }

    let chars: Vec<char> = text.chars().collect();
    let mut counts: HashMap<(char, char, char), usize> = HashMap::new();
    let mut total = 0usize;

    for window in chars.windows(3) {
        let trigram = (window[0], window[1], window[2]);
        *counts.entry(trigram).or_default() += 1;
        total += 1;
    }

    if total == 0 {
        return 0.0;
    }

    let mut entropy = 0.0;
    let total_f = total as f64;
    for &count in counts.values() {
        let p = count as f64 / total_f;
        entropy -= p * p.ln();
    }

    entropy
}

/// Map strategy index to `Strategy` variant.
fn idx_to_strategy(idx: usize) -> Strategy {
    match idx {
        0 => Strategy::Rlm,
        1 => Strategy::Trm("default".into()),
        2 => Strategy::Edge,
        3 => Strategy::Hybrid {
            triage: "auto-triage".into(),
            threshold: 0.5,
        },
        4 => Strategy::swarm_default(),
        _ => Strategy::Auto,
    }
}

/// Map a `Strategy` variant to its index in the output vector.
fn strategy_to_idx(strategy: &Strategy) -> usize {
    match strategy {
        Strategy::Rlm => 0,
        Strategy::Trm(_) => 1,
        Strategy::Edge => 2,
        Strategy::Hybrid { .. } => 3,
        Strategy::Swarm { .. } => 4,
        Strategy::Auto => 0, // fallback
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_softmax() {
        let probs = softmax(&[1.0, 2.0, 3.0]);
        assert_eq!(probs.len(), 3);
        let sum: f64 = probs.iter().sum();
        assert!((sum - 1.0).abs() < 1e-9, "softmax should sum to 1.0");
        // Largest logit should have largest probability.
        assert!(probs[2] > probs[1]);
        assert!(probs[1] > probs[0]);
    }

    #[test]
    fn test_softmax_empty() {
        let probs = softmax(&[]);
        assert!(probs.is_empty());
    }

    #[test]
    fn test_softmax_uniform() {
        let probs = softmax(&[0.0, 0.0, 0.0]);
        for &p in &probs {
            assert!((p - 1.0 / 3.0).abs() < 1e-9);
        }
    }

    #[test]
    fn test_trigram_entropy_short() {
        assert_eq!(trigram_entropy("ab"), 0.0);
        assert_eq!(trigram_entropy(""), 0.0);
    }

    #[test]
    fn test_trigram_entropy_nonzero() {
        let e = trigram_entropy("hello world, this is a test of entropy computation");
        assert!(e > 0.0, "entropy of varied text should be positive");
    }

    #[test]
    fn test_extract_features_14_dimensions() {
        let input = TinyDancerRouter::extract_features("fn main() { }", true, 0.5);
        assert!(input.has_code);
        assert!(!input.is_question);
        assert!(input.edge_available);
        assert!((input.node_load - 0.5).abs() < f64::EPSILON);
        assert!(input.query_length > 0);
        // Verify new fields have defaults.
        assert!((input.zone_a_available - 1.0).abs() < f64::EPSILON);
        assert!((input.zone_b_available - 1.0).abs() < f64::EPSILON);
        assert!(input.token_count_estimate >= 0.0);
        for &s in &input.recent_strategy_success {
            assert!((s - 0.5).abs() < f64::EPSILON);
        }
        // Verify input_to_vec produces 14 elements.
        let vec = TinyDancerRouter::input_to_vec(&input);
        assert_eq!(vec.len(), 14, "feature vector must have 14 elements");
    }

    #[test]
    fn test_extract_features_question() {
        let input =
            TinyDancerRouter::extract_features("What is the meaning of life?", false, 0.0);
        assert!(!input.has_code);
        assert!(input.is_question);
        assert!(!input.edge_available);
    }

    #[test]
    fn test_default_routing() {
        let router = TinyDancerRouter::new();
        assert_eq!(router.input_dim, 14, "input_dim must be 14");
        assert_eq!(router.hidden_dim, 32, "hidden_dim must be 32");
        assert_eq!(router.output_dim, 5, "output_dim must be 5");

        // Short question should lean towards Rlm.
        let input = TinyDancerRouter::extract_features("What is Rust?", false, 0.0);
        let output = router.route(&input);
        assert!(output.confidence > 0.0);
        assert!(!output.scores.is_empty());
        let total: f64 = output.scores.values().sum();
        assert!(
            (total - 1.0).abs() < 1e-6,
            "scores should sum to 1.0, got {total}"
        );

        // Code query should lean towards Trm.
        let input = TinyDancerRouter::extract_features(
            "```rust\nfn main() { println!(\"hello\"); }\n```",
            false,
            0.0,
        );
        let output = router.route(&input);
        assert!(output.confidence > 0.0);

        // Edge available, low load should consider Edge.
        let input = TinyDancerRouter::extract_features("summarize this", true, 0.1);
        let output = router.route(&input);
        assert!(output.scores.contains_key("Edge"));

        // High load: Swarm should get a boost.
        let input = TinyDancerRouter::extract_features("process data", false, 0.95);
        let output = router.route(&input);
        assert!(
            output.scores.get("Swarm").copied().unwrap_or(0.0) > 0.05,
            "Swarm should have notable score under high load"
        );
    }

    #[test]
    fn test_train_updates_weights() {
        let mut router = TinyDancerRouter::new();
        let original_bias2 = router.bias2.clone();

        let records: Vec<(RouterInput, Strategy, f64)> = vec![
            (
                TinyDancerRouter::extract_features("short question?", false, 0.0),
                Strategy::Rlm,
                1.0,
            ),
            (
                TinyDancerRouter::extract_features("```code block```", false, 0.0),
                Strategy::Trm("default".into()),
                0.8,
            ),
            (
                TinyDancerRouter::extract_features("distribute work", false, 0.9),
                Strategy::swarm_default(),
                1.0,
            ),
        ];

        router.train_from_history(&records);

        // Weights should have changed.
        assert_ne!(
            router.bias2, original_bias2,
            "training should update bias2"
        );
    }

    #[test]
    fn test_route_deterministic() {
        let router = TinyDancerRouter::new();
        let input = TinyDancerRouter::extract_features("hello world?", false, 0.3);
        let out1 = router.route(&input);
        let out2 = router.route(&input);
        assert!(
            (out1.confidence - out2.confidence).abs() < 1e-12,
            "same input should produce identical confidence"
        );
    }

    #[test]
    fn test_latency_ns_is_populated() {
        let router = TinyDancerRouter::new();
        let input = TinyDancerRouter::extract_features("test query?", false, 0.0);
        let output = router.route(&input);
        // latency_ns should be non-negative (it is u64, so always >= 0).
        // On any modern machine a forward pass takes at least 1ns.
        // We just verify the field exists and the route completes.
        assert!(output.latency_ns < 100_000_000, "routing should be sub-100ms");
    }

    #[test]
    fn test_online_buffer_record_and_auto_retrain() {
        let mut router = TinyDancerRouter::new();
        router.retrain_threshold = 5; // low threshold for testing

        let original_bias2 = router.bias2.clone();

        // Record 4 outcomes (below threshold -- no retrain yet).
        for i in 0..4 {
            let input = TinyDancerRouter::extract_features(
                &format!("query number {i}?"),
                false,
                0.0,
            );
            router.record_outcome(input, &Strategy::Rlm, 1.0);
        }
        assert_eq!(router.online_buffer.len(), 4);
        assert_eq!(
            router.bias2, original_bias2,
            "no retrain should happen below threshold"
        );

        // 5th record triggers retrain and drains the buffer.
        let input = TinyDancerRouter::extract_features("fifth query?", false, 0.0);
        router.record_outcome(input, &Strategy::Rlm, 1.0);
        assert!(
            router.online_buffer.is_empty(),
            "buffer should be drained after retrain"
        );
        assert_ne!(
            router.bias2, original_bias2,
            "weights should have changed after auto-retrain"
        );
    }

    #[test]
    fn test_network_dimensions() {
        let router = TinyDancerRouter::new();
        assert_eq!(router.weights1.len(), 32, "W1 should have 32 rows");
        for (i, row) in router.weights1.iter().enumerate() {
            assert_eq!(
                row.len(),
                14,
                "W1 row {i} should have 14 columns"
            );
        }
        assert_eq!(router.bias1.len(), 32, "b1 should have 32 elements");
        assert_eq!(router.weights2.len(), 5, "W2 should have 5 rows");
        for (i, row) in router.weights2.iter().enumerate() {
            assert_eq!(
                row.len(),
                32,
                "W2 row {i} should have 32 columns"
            );
        }
        assert_eq!(router.bias2.len(), 5, "b2 should have 5 elements");
    }
}
