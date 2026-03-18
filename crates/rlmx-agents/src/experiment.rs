use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExperimentStatus {
    Proposed,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub description: String,
    pub score: f64,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experiment {
    pub id: Uuid,
    pub hypothesis: String,
    pub status: ExperimentStatus,
    pub evidence: Vec<Evidence>,
    pub fitness: f64,
    pub generation: u32,
    pub created_at: DateTime<Utc>,
    pub failure_reason: Option<String>,
}

impl Experiment {
    pub fn new(hypothesis: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            hypothesis: hypothesis.into(),
            status: ExperimentStatus::Proposed,
            evidence: Vec::new(),
            fitness: 0.0,
            generation: 0,
            created_at: Utc::now(),
            failure_reason: None,
        }
    }

    pub fn with_generation(mut self, generation: u32) -> Self {
        self.generation = generation;
        self
    }

    /// Add evidence to this experiment.
    pub fn add_evidence(&mut self, evidence: Evidence) {
        self.evidence.push(evidence);
        // Update fitness as average of evidence scores
        if !self.evidence.is_empty() {
            self.fitness =
                self.evidence.iter().map(|e| e.score).sum::<f64>() / self.evidence.len() as f64;
        }
    }

    /// Mark the experiment as completed with a final fitness score.
    pub fn complete(&mut self, fitness: f64) {
        self.status = ExperimentStatus::Completed;
        self.fitness = fitness;
    }

    /// Mark the experiment as failed with a reason.
    pub fn fail(&mut self, reason: impl Into<String>) {
        self.status = ExperimentStatus::Failed;
        self.failure_reason = Some(reason.into());
    }

    /// Start running the experiment.
    pub fn start(&mut self) {
        self.status = ExperimentStatus::Running;
    }
}

/// Multi-dimensional fitness score combining accuracy, latency, and cost.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FitnessScore {
    pub accuracy: f64,
    pub latency_ms: f64,
    pub cost: f64,
    /// Weighted combination: accuracy_w * accuracy - latency_w * (latency/1000) - cost_w * cost.
    pub combined: f64,
}

/// Evaluates experiment fitness using a weighted combination of accuracy, latency, and cost.
pub struct FitnessEvaluator {
    pub accuracy_weight: f64,
    pub latency_weight: f64,
    pub cost_weight: f64,
}

impl FitnessEvaluator {
    pub fn new(accuracy_weight: f64, latency_weight: f64, cost_weight: f64) -> Self {
        Self {
            accuracy_weight,
            latency_weight,
            cost_weight,
        }
    }

    /// Compute a combined fitness score from raw metrics.
    pub fn evaluate(&self, accuracy: f64, latency_ms: f64, cost: f64) -> FitnessScore {
        let combined = self.accuracy_weight * accuracy
            - self.latency_weight * (latency_ms / 1000.0)
            - self.cost_weight * cost;
        FitnessScore {
            accuracy,
            latency_ms,
            cost,
            combined,
        }
    }
}

impl Default for FitnessEvaluator {
    fn default() -> Self {
        Self {
            accuracy_weight: 0.5,
            latency_weight: 0.3,
            cost_weight: 0.2,
        }
    }
}

/// Tracks experiments across generations.
pub struct ExperimentTracker {
    experiments: Vec<Experiment>,
}

impl ExperimentTracker {
    pub fn new() -> Self {
        Self {
            experiments: Vec::new(),
        }
    }

    /// Add an experiment to the tracker.
    pub fn add(&mut self, experiment: Experiment) {
        self.experiments.push(experiment);
    }

    /// List all experiments.
    pub fn list(&self) -> &[Experiment] {
        &self.experiments
    }

    /// Get active (Running) experiments.
    pub fn active(&self) -> Vec<&Experiment> {
        self.experiments
            .iter()
            .filter(|e| e.status == ExperimentStatus::Running)
            .collect()
    }

    /// Get completed experiments.
    pub fn completed(&self) -> Vec<&Experiment> {
        self.experiments
            .iter()
            .filter(|e| e.status == ExperimentStatus::Completed)
            .collect()
    }

    /// Find an experiment by id.
    pub fn by_id(&self, id: Uuid) -> Option<&Experiment> {
        self.experiments.iter().find(|e| e.id == id)
    }

    /// Find a mutable experiment by id.
    pub fn by_id_mut(&mut self, id: Uuid) -> Option<&mut Experiment> {
        self.experiments.iter_mut().find(|e| e.id == id)
    }

    /// Get the best completed experiment by fitness.
    pub fn best(&self) -> Option<&Experiment> {
        self.completed().into_iter().max_by(|a, b| {
            a.fitness
                .partial_cmp(&b.fitness)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Count experiments by status.
    pub fn count(&self) -> usize {
        self.experiments.len()
    }
}

impl Default for ExperimentTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_experiment_new() {
        let exp = Experiment::new("test hypothesis");
        assert_eq!(exp.hypothesis, "test hypothesis");
        assert_eq!(exp.status, ExperimentStatus::Proposed);
        assert_eq!(exp.fitness, 0.0);
    }

    #[test]
    fn test_experiment_with_generation() {
        let exp = Experiment::new("h1").with_generation(5);
        assert_eq!(exp.generation, 5);
    }

    #[test]
    fn test_add_evidence() {
        let mut exp = Experiment::new("h1");
        exp.add_evidence(Evidence {
            description: "positive result".into(),
            score: 0.8,
            source: "test".into(),
        });
        exp.add_evidence(Evidence {
            description: "neutral result".into(),
            score: 0.4,
            source: "test".into(),
        });
        assert_eq!(exp.evidence.len(), 2);
        assert!((exp.fitness - 0.6).abs() < 0.001); // avg(0.8, 0.4) = 0.6
    }

    #[test]
    fn test_complete_experiment() {
        let mut exp = Experiment::new("h1");
        exp.complete(0.95);
        assert_eq!(exp.status, ExperimentStatus::Completed);
        assert_eq!(exp.fitness, 0.95);
    }

    #[test]
    fn test_fail_experiment() {
        let mut exp = Experiment::new("h1");
        exp.fail("timeout");
        assert_eq!(exp.status, ExperimentStatus::Failed);
        assert_eq!(exp.failure_reason.as_deref(), Some("timeout"));
    }

    #[test]
    fn test_tracker_add_and_list() {
        let mut tracker = ExperimentTracker::new();
        tracker.add(Experiment::new("h1"));
        tracker.add(Experiment::new("h2"));
        assert_eq!(tracker.list().len(), 2);
    }

    #[test]
    fn test_tracker_active() {
        let mut tracker = ExperimentTracker::new();
        let mut e1 = Experiment::new("h1");
        e1.start();
        let e2 = Experiment::new("h2"); // Proposed, not running
        tracker.add(e1);
        tracker.add(e2);
        assert_eq!(tracker.active().len(), 1);
    }

    #[test]
    fn test_tracker_completed() {
        let mut tracker = ExperimentTracker::new();
        let mut e1 = Experiment::new("h1");
        e1.complete(0.9);
        let mut e2 = Experiment::new("h2");
        e2.complete(0.7);
        tracker.add(e1);
        tracker.add(e2);
        assert_eq!(tracker.completed().len(), 2);
    }

    #[test]
    fn test_tracker_by_id() {
        let mut tracker = ExperimentTracker::new();
        let exp = Experiment::new("h1");
        let id = exp.id;
        tracker.add(exp);
        assert!(tracker.by_id(id).is_some());
        assert!(tracker.by_id(Uuid::new_v4()).is_none());
    }

    #[test]
    fn test_tracker_best() {
        let mut tracker = ExperimentTracker::new();
        let mut e1 = Experiment::new("h1");
        e1.complete(0.5);
        let mut e2 = Experiment::new("h2");
        e2.complete(0.9);
        tracker.add(e1);
        tracker.add(e2);
        let best = tracker.best().unwrap();
        assert_eq!(best.fitness, 0.9);
    }

    #[test]
    fn test_tracker_best_none() {
        let tracker = ExperimentTracker::new();
        assert!(tracker.best().is_none());
    }

    // --- FitnessEvaluator tests ---

    #[test]
    fn test_fitness_evaluator_default() {
        let eval = FitnessEvaluator::default();
        assert!((eval.accuracy_weight - 0.5).abs() < 1e-10);
        assert!((eval.latency_weight - 0.3).abs() < 1e-10);
        assert!((eval.cost_weight - 0.2).abs() < 1e-10);
    }

    #[test]
    fn test_fitness_evaluate_combined() {
        let eval = FitnessEvaluator::default();
        // combined = 0.5*0.9 - 0.3*(100/1000) - 0.2*0.5
        // = 0.45 - 0.03 - 0.1 = 0.32
        let score = eval.evaluate(0.9, 100.0, 0.5);
        assert!((score.combined - 0.32).abs() < 1e-10);
        assert!((score.accuracy - 0.9).abs() < 1e-10);
        assert!((score.latency_ms - 100.0).abs() < 1e-10);
        assert!((score.cost - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_fitness_evaluate_perfect() {
        let eval = FitnessEvaluator::new(1.0, 0.0, 0.0);
        let score = eval.evaluate(1.0, 5000.0, 100.0);
        assert!((score.combined - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_fitness_evaluate_high_latency_penalized() {
        let eval = FitnessEvaluator::default();
        let fast = eval.evaluate(0.8, 10.0, 0.1);
        let slow = eval.evaluate(0.8, 5000.0, 0.1);
        assert!(fast.combined > slow.combined);
    }

    #[test]
    fn test_fitness_evaluate_high_cost_penalized() {
        let eval = FitnessEvaluator::default();
        let cheap = eval.evaluate(0.8, 100.0, 0.01);
        let expensive = eval.evaluate(0.8, 100.0, 10.0);
        assert!(cheap.combined > expensive.combined);
    }
}
