use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Training configuration for an experiment run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    pub learning_rate: f64,
    pub batch_size: usize,
    pub max_steps: usize,
    /// Compute backend: "mlx", "cuda", "cpu".
    pub backend: String,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            learning_rate: 1e-4,
            batch_size: 32,
            max_steps: 1000,
            backend: "cpu".into(),
        }
    }
}

/// A serializable genome representing strategy parameters (ADR-006).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationStrategy {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub generation: u32,
    pub feature_weights: HashMap<String, f64>,
    pub routing_thresholds: Vec<f64>,
    pub prompt_templates: Vec<String>,
    pub training_config: TrainingConfig,
    pub fitness: Option<f64>,
}

impl MutationStrategy {
    /// Create a random genome with the given number of feature weights and thresholds.
    pub fn random(weight_count: usize, threshold_count: usize) -> Self {
        let mut rng = rand::thread_rng();
        let feature_weights: HashMap<String, f64> = (0..weight_count)
            .map(|i| (format!("w{i}"), rng.gen_range(-1.0..1.0)))
            .collect();
        let routing_thresholds = (0..threshold_count)
            .map(|_| rng.gen_range(0.0..1.0))
            .collect();
        Self {
            id: Uuid::new_v4(),
            parent_id: None,
            generation: 0,
            feature_weights,
            routing_thresholds,
            prompt_templates: Vec::new(),
            training_config: TrainingConfig::default(),
            fitness: None,
        }
    }

    /// Convenience: collect feature weights as a sorted Vec for deterministic iteration.
    pub fn weights_vec(&self) -> Vec<f64> {
        let mut keys: Vec<_> = self.feature_weights.keys().collect();
        keys.sort();
        keys.iter().map(|k| self.feature_weights[*k]).collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mutation {
    pub id: Uuid,
    pub generation: u32,
    pub parent_id: Option<Uuid>,
    pub strategy: MutationStrategy,
    pub fitness: f64,
    pub timestamp: DateTime<Utc>,
}

/// Cross-pollination: combine two genomes via weighted averaging and tournament selection.
pub struct CrossPollinator {
    /// Adopt a foreign mutation if it improves fitness by at least this amount.
    pub adoption_threshold: f64,
}

impl CrossPollinator {
    pub fn new(threshold: f64) -> Self {
        Self {
            adoption_threshold: threshold,
        }
    }

    /// Combine two parent genomes into a child genome.
    ///
    /// Feature weights are blended using a weighted average based on parent fitness
    /// (higher fitness gets more influence). Prompt templates are taken from the
    /// fitter parent (tournament selection). Generation is incremented from the
    /// maximum of the two parents.
    pub fn crossover(
        &self,
        parent_a: &MutationStrategy,
        parent_b: &MutationStrategy,
    ) -> MutationStrategy {
        let fit_a = parent_a.fitness.unwrap_or(0.0);
        let fit_b = parent_b.fitness.unwrap_or(0.0);
        let total = fit_a + fit_b;
        let (w_a, w_b) = if total > 0.0 {
            (fit_a / total, fit_b / total)
        } else {
            (0.5, 0.5)
        };

        // Weighted average of feature weights (union of keys)
        let mut feature_weights = HashMap::new();
        let all_keys: std::collections::HashSet<&String> = parent_a
            .feature_weights
            .keys()
            .chain(parent_b.feature_weights.keys())
            .collect();
        for key in all_keys {
            let va = parent_a.feature_weights.get(key).copied().unwrap_or(0.0);
            let vb = parent_b.feature_weights.get(key).copied().unwrap_or(0.0);
            feature_weights.insert(key.clone(), w_a * va + w_b * vb);
        }

        // Average routing thresholds (use shorter length to stay safe)
        let min_len = parent_a
            .routing_thresholds
            .len()
            .min(parent_b.routing_thresholds.len());
        let routing_thresholds: Vec<f64> = (0..min_len)
            .map(|i| w_a * parent_a.routing_thresholds[i] + w_b * parent_b.routing_thresholds[i])
            .collect();

        // Tournament selection: pick prompt_templates from the fitter parent
        let prompt_templates = if fit_a >= fit_b {
            parent_a.prompt_templates.clone()
        } else {
            parent_b.prompt_templates.clone()
        };

        // Training config from the fitter parent
        let training_config = if fit_a >= fit_b {
            parent_a.training_config.clone()
        } else {
            parent_b.training_config.clone()
        };

        let gen = parent_a.generation.max(parent_b.generation) + 1;

        MutationStrategy {
            id: Uuid::new_v4(),
            parent_id: Some(if fit_a >= fit_b {
                parent_a.id
            } else {
                parent_b.id
            }),
            generation: gen,
            feature_weights,
            routing_thresholds,
            prompt_templates,
            training_config,
            fitness: None,
        }
    }

    /// Decide whether to adopt a foreign mutation.
    /// Returns true if the candidate fitness is better (lower val_bpb) by at
    /// least `adoption_threshold`.
    pub fn should_adopt(&self, current_best: f64, candidate: f64) -> bool {
        candidate < current_best - self.adoption_threshold
    }
}

/// Cloud escalation tracker: detects stalled evolution and triggers Tier-3 escalation.
pub struct CloudEscalation {
    /// Number of generations without improvement before escalating.
    pub stall_threshold: u32,
    /// Current consecutive stall count.
    pub stall_counter: u32,
    /// Best fitness observed so far (lower is better for val_bpb).
    pub best_fitness: Option<f64>,
}

impl CloudEscalation {
    pub fn new(stall_threshold: u32) -> Self {
        Self {
            stall_threshold,
            stall_counter: 0,
            best_fitness: None,
        }
    }

    /// Record a generation's best fitness. Returns `true` if escalation is needed
    /// (stall counter has reached the threshold).
    pub fn record_generation(&mut self, best_fitness: f64) -> bool {
        if let Some(prev_best) = self.best_fitness {
            if best_fitness >= prev_best {
                self.stall_counter += 1;
            } else {
                self.stall_counter = 0;
                self.best_fitness = Some(best_fitness);
            }
        } else {
            self.best_fitness = Some(best_fitness);
        }
        self.stall_counter >= self.stall_threshold
    }

    /// Reset the escalation tracker.
    pub fn reset(&mut self) {
        self.stall_counter = 0;
        self.best_fitness = None;
    }
}

/// Evolutionary mutation engine — mutates strategies and tracks fitness history.
pub struct MutationEngine {
    history: Vec<Mutation>,
    best_fitness: f64,
    /// Standard deviation for Gaussian mutation.
    mutation_rate: f64,
}

impl MutationEngine {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            best_fitness: f64::NEG_INFINITY,
            mutation_rate: 0.1,
        }
    }

    /// Create a mutated variant of a parent strategy using Gaussian perturbation.
    pub fn mutate(&self, parent: &MutationStrategy) -> MutationStrategy {
        let mut rng = rand::thread_rng();

        let feature_weights: HashMap<String, f64> = parent
            .feature_weights
            .iter()
            .map(|(k, w)| {
                let noise: f64 = rng.gen_range(-self.mutation_rate..self.mutation_rate);
                (k.clone(), (w + noise).clamp(-1.0, 1.0))
            })
            .collect();

        let routing_thresholds = parent
            .routing_thresholds
            .iter()
            .map(|t| {
                let noise: f64 = rng.gen_range(-self.mutation_rate..self.mutation_rate);
                (t + noise).clamp(0.0, 1.0)
            })
            .collect();

        MutationStrategy {
            id: Uuid::new_v4(),
            parent_id: Some(parent.id),
            generation: parent.generation + 1,
            feature_weights,
            routing_thresholds,
            prompt_templates: parent.prompt_templates.clone(),
            training_config: parent.training_config.clone(),
            fitness: None,
        }
    }

    /// Record a mutation with its fitness score.
    pub fn record(&mut self, mutation: Mutation) {
        if mutation.fitness > self.best_fitness {
            self.best_fitness = mutation.fitness;
        }
        self.history.push(mutation);
    }

    /// Get the best mutation by fitness.
    pub fn best(&self) -> Option<&Mutation> {
        self.history.iter().max_by(|a, b| {
            a.fitness
                .partial_cmp(&b.fitness)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Get the mutation history.
    pub fn history(&self) -> &[Mutation] {
        &self.history
    }

    /// Get the best fitness seen so far.
    pub fn best_fitness(&self) -> f64 {
        self.best_fitness
    }

    /// Set the mutation rate (standard deviation).
    pub fn set_mutation_rate(&mut self, rate: f64) {
        self.mutation_rate = rate.abs();
    }
}

impl Default for MutationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_parent(weights: &[(&str, f64)], thresholds: &[f64]) -> MutationStrategy {
        MutationStrategy {
            id: Uuid::new_v4(),
            parent_id: None,
            generation: 0,
            feature_weights: weights.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            routing_thresholds: thresholds.to_vec(),
            prompt_templates: Vec::new(),
            training_config: TrainingConfig::default(),
            fitness: None,
        }
    }

    #[test]
    fn test_mutation_strategy_random() {
        let strategy = MutationStrategy::random(8, 3);
        assert_eq!(strategy.feature_weights.len(), 8);
        assert_eq!(strategy.routing_thresholds.len(), 3);
        assert!(strategy
            .feature_weights
            .values()
            .all(|w| *w >= -1.0 && *w <= 1.0));
        assert!(strategy
            .routing_thresholds
            .iter()
            .all(|t| *t >= 0.0 && *t <= 1.0));
        assert!(strategy.fitness.is_none());
        assert!(strategy.parent_id.is_none());
        assert_eq!(strategy.generation, 0);
    }

    #[test]
    fn test_mutate_preserves_dimensions() {
        let engine = MutationEngine::new();
        let parent = make_parent(&[("a", 0.5), ("b", -0.3), ("c", 0.8)], &[0.2, 0.7]);
        let child = engine.mutate(&parent);
        assert_eq!(child.feature_weights.len(), 3);
        assert_eq!(child.routing_thresholds.len(), 2);
        assert_eq!(child.generation, 1);
        assert_eq!(child.parent_id, Some(parent.id));
    }

    #[test]
    fn test_mutate_stays_in_bounds() {
        let engine = MutationEngine::new();
        let parent = make_parent(&[("x", 0.99), ("y", -0.99)], &[0.01, 0.99]);
        for _ in 0..100 {
            let child = engine.mutate(&parent);
            assert!(child
                .feature_weights
                .values()
                .all(|w| *w >= -1.0 && *w <= 1.0));
            assert!(child
                .routing_thresholds
                .iter()
                .all(|t| *t >= 0.0 && *t <= 1.0));
        }
    }

    #[test]
    fn test_record_mutation() {
        let mut engine = MutationEngine::new();
        let mutation = Mutation {
            id: Uuid::new_v4(),
            generation: 1,
            parent_id: None,
            strategy: MutationStrategy::random(4, 2),
            fitness: 0.85,
            timestamp: Utc::now(),
        };
        engine.record(mutation);
        assert_eq!(engine.history().len(), 1);
        assert_eq!(engine.best_fitness(), 0.85);
    }

    #[test]
    fn test_best_mutation() {
        let mut engine = MutationEngine::new();
        let m1 = Mutation {
            id: Uuid::new_v4(),
            generation: 1,
            parent_id: None,
            strategy: MutationStrategy::random(4, 2),
            fitness: 0.5,
            timestamp: Utc::now(),
        };
        let m1_id = m1.id;
        let m2 = Mutation {
            id: Uuid::new_v4(),
            generation: 2,
            parent_id: Some(m1_id),
            strategy: MutationStrategy::random(4, 2),
            fitness: 0.9,
            timestamp: Utc::now(),
        };
        engine.record(m1);
        engine.record(m2);
        let best = engine.best().unwrap();
        assert_eq!(best.fitness, 0.9);
    }

    #[test]
    fn test_empty_best() {
        let engine = MutationEngine::new();
        assert!(engine.best().is_none());
    }

    #[test]
    fn test_mutation_rate() {
        let mut engine = MutationEngine::new();
        engine.set_mutation_rate(0.5);
        let parent = MutationStrategy::random(10, 5);
        let child = engine.mutate(&parent);
        assert!(child
            .feature_weights
            .values()
            .all(|w| *w >= -1.0 && *w <= 1.0));
    }

    #[test]
    fn test_best_fitness_tracking() {
        let mut engine = MutationEngine::new();
        assert_eq!(engine.best_fitness(), f64::NEG_INFINITY);

        engine.record(Mutation {
            id: Uuid::new_v4(),
            generation: 1,
            parent_id: None,
            strategy: MutationStrategy::random(2, 1),
            fitness: 0.3,
            timestamp: Utc::now(),
        });
        assert_eq!(engine.best_fitness(), 0.3);

        engine.record(Mutation {
            id: Uuid::new_v4(),
            generation: 2,
            parent_id: None,
            strategy: MutationStrategy::random(2, 1),
            fitness: 0.1,
            timestamp: Utc::now(),
        });
        assert_eq!(engine.best_fitness(), 0.3); // Still 0.3
    }

    #[test]
    fn test_training_config_default() {
        let cfg = TrainingConfig::default();
        assert!((cfg.learning_rate - 1e-4).abs() < 1e-10);
        assert_eq!(cfg.batch_size, 32);
        assert_eq!(cfg.max_steps, 1000);
        assert_eq!(cfg.backend, "cpu");
    }

    #[test]
    fn test_weights_vec_sorted() {
        let mut s = MutationStrategy::random(0, 0);
        s.feature_weights.insert("z".into(), 0.1);
        s.feature_weights.insert("a".into(), 0.9);
        s.feature_weights.insert("m".into(), 0.5);
        let v = s.weights_vec();
        assert_eq!(v.len(), 3);
        assert!((v[0] - 0.9).abs() < 1e-10); // a
        assert!((v[1] - 0.5).abs() < 1e-10); // m
        assert!((v[2] - 0.1).abs() < 1e-10); // z
    }

    // --- CrossPollinator tests ---

    #[test]
    fn test_crossover_blends_weights() {
        let cp = CrossPollinator::new(0.01);
        let mut a = make_parent(&[("x", 1.0)], &[0.8]);
        a.fitness = Some(0.8);
        let mut b = make_parent(&[("x", 0.0)], &[0.2]);
        b.fitness = Some(0.2);

        let child = cp.crossover(&a, &b);
        let x = child.feature_weights["x"];
        // w_a = 0.8, w_b = 0.2, blend = 0.8*1.0 + 0.2*0.0 = 0.8
        assert!((x - 0.8).abs() < 1e-10);
        assert_eq!(child.generation, 1);
        assert!(child.fitness.is_none());
    }

    #[test]
    fn test_crossover_equal_fitness() {
        let cp = CrossPollinator::new(0.01);
        let mut a = make_parent(&[("x", 1.0)], &[]);
        a.fitness = Some(0.5);
        let mut b = make_parent(&[("x", 0.0)], &[]);
        b.fitness = Some(0.5);
        let child = cp.crossover(&a, &b);
        let x = child.feature_weights["x"];
        assert!((x - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_crossover_no_fitness_defaults_to_equal() {
        let cp = CrossPollinator::new(0.01);
        let a = make_parent(&[("x", 1.0)], &[]);
        let b = make_parent(&[("x", 0.0)], &[]);
        let child = cp.crossover(&a, &b);
        let x = child.feature_weights["x"];
        assert!((x - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_crossover_tournament_selects_fitter_prompts() {
        let cp = CrossPollinator::new(0.01);
        let mut a = make_parent(&[], &[]);
        a.prompt_templates = vec!["prompt_a".into()];
        a.fitness = Some(0.9);
        let mut b = make_parent(&[], &[]);
        b.prompt_templates = vec!["prompt_b".into()];
        b.fitness = Some(0.1);
        let child = cp.crossover(&a, &b);
        assert_eq!(child.prompt_templates, vec!["prompt_a".to_string()]);
    }

    #[test]
    fn test_crossover_union_keys() {
        let cp = CrossPollinator::new(0.01);
        let mut a = make_parent(&[("only_a", 1.0)], &[]);
        a.fitness = Some(0.5);
        let mut b = make_parent(&[("only_b", 0.6)], &[]);
        b.fitness = Some(0.5);
        let child = cp.crossover(&a, &b);
        assert!(child.feature_weights.contains_key("only_a"));
        assert!(child.feature_weights.contains_key("only_b"));
    }

    #[test]
    fn test_should_adopt_true() {
        let cp = CrossPollinator::new(0.05);
        assert!(cp.should_adopt(0.5, 0.39));
    }

    #[test]
    fn test_should_adopt_false_not_enough_improvement() {
        let cp = CrossPollinator::new(0.05);
        assert!(!cp.should_adopt(0.5, 0.46));
    }

    #[test]
    fn test_should_adopt_false_worse() {
        let cp = CrossPollinator::new(0.05);
        assert!(!cp.should_adopt(0.5, 0.6));
    }

    // --- CloudEscalation tests ---

    #[test]
    fn test_cloud_escalation_no_stall() {
        let mut esc = CloudEscalation::new(3);
        assert!(!esc.record_generation(0.5));
        assert!(!esc.record_generation(0.4));
        assert!(!esc.record_generation(0.3));
        assert!(!esc.record_generation(0.2));
    }

    #[test]
    fn test_cloud_escalation_stall_triggers() {
        let mut esc = CloudEscalation::new(3);
        assert!(!esc.record_generation(0.5)); // sets baseline
        assert!(!esc.record_generation(0.5)); // stall 1
        assert!(!esc.record_generation(0.6)); // stall 2
        assert!(esc.record_generation(0.7)); // stall 3 => escalate
    }

    #[test]
    fn test_cloud_escalation_reset_on_improvement() {
        let mut esc = CloudEscalation::new(3);
        esc.record_generation(0.5);
        esc.record_generation(0.5); // stall 1
        esc.record_generation(0.6); // stall 2
        esc.record_generation(0.4); // improvement resets counter
        assert_eq!(esc.stall_counter, 0);
        assert!(!esc.record_generation(0.4)); // stall 1 again
    }

    #[test]
    fn test_cloud_escalation_reset() {
        let mut esc = CloudEscalation::new(3);
        esc.record_generation(0.5);
        esc.record_generation(0.5);
        esc.reset();
        assert_eq!(esc.stall_counter, 0);
        assert!(esc.best_fitness.is_none());
    }
}
