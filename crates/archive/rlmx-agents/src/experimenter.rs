use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::experiment::Experiment;
use crate::types::AgentId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResult {
    pub hypothesis_id: Uuid,
    pub success: bool,
    pub metrics: HashMap<String, f64>,
    pub artifacts: Vec<String>,
}

/// Experimenter agent — runs experiments and evaluates hypotheses.
pub struct ExperimenterAgent {
    pub id: AgentId,
    pub parent: AgentId,
    pub experiment: Experiment,
    pub generation: u32,
}

impl ExperimenterAgent {
    pub fn new(parent: AgentId, hypothesis: impl Into<String>) -> Self {
        let experiment = Experiment::new(hypothesis);
        Self {
            id: AgentId::new(),
            parent,
            experiment,
            generation: 0,
        }
    }

    pub fn with_generation(mut self, generation: u32) -> Self {
        self.generation = generation;
        self.experiment.generation = generation;
        self
    }

    /// Run the experiment and produce a result.
    pub async fn run_experiment(&mut self) -> Result<ExperimentResult, String> {
        self.experiment.start();

        info!(
            experimenter_id = %self.id.0,
            hypothesis = %self.experiment.hypothesis,
            generation = self.generation,
            "Running experiment"
        );

        // Simulate experiment execution
        // In a real implementation, this would invoke kernel syscalls
        let mut metrics = HashMap::new();
        metrics.insert("accuracy".into(), 0.85);
        metrics.insert("latency_ms".into(), 12.5);
        metrics.insert("throughput".into(), 1000.0);

        let fitness = metrics.get("accuracy").copied().unwrap_or(0.0);
        let success = fitness > 0.5;

        if success {
            self.experiment.complete(fitness);
        } else {
            self.experiment.fail("Fitness below threshold");
        }

        let result = ExperimentResult {
            hypothesis_id: self.experiment.id,
            success,
            metrics,
            artifacts: vec![format!("gen{}_result.json", self.generation)],
        };

        info!(
            experimenter_id = %self.id.0,
            success = success,
            fitness = fitness,
            "Experiment completed"
        );

        Ok(result)
    }

    /// Get the current experiment.
    pub fn experiment(&self) -> &Experiment {
        &self.experiment
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_experimenter_new() {
        let parent = AgentId::new();
        let exp = ExperimenterAgent::new(parent, "test hypothesis");
        assert_eq!(exp.experiment.hypothesis, "test hypothesis");
        assert_eq!(exp.generation, 0);
    }

    #[test]
    fn test_with_generation() {
        let parent = AgentId::new();
        let exp = ExperimenterAgent::new(parent, "h1").with_generation(5);
        assert_eq!(exp.generation, 5);
        assert_eq!(exp.experiment.generation, 5);
    }

    #[tokio::test]
    async fn test_run_experiment() {
        let parent = AgentId::new();
        let mut exp = ExperimenterAgent::new(parent, "neural architecture search");
        let result = exp.run_experiment().await.unwrap();
        assert!(result.success);
        assert!(result.metrics.contains_key("accuracy"));
        assert!(!result.artifacts.is_empty());
    }

    #[tokio::test]
    async fn test_experiment_status_after_run() {
        let parent = AgentId::new();
        let mut exp = ExperimenterAgent::new(parent, "test");
        exp.run_experiment().await.unwrap();
        assert_eq!(
            exp.experiment().status,
            crate::experiment::ExperimentStatus::Completed
        );
    }

    #[tokio::test]
    async fn test_experiment_result_has_hypothesis_id() {
        let parent = AgentId::new();
        let mut exp = ExperimenterAgent::new(parent, "h1");
        let expected_id = exp.experiment.id;
        let result = exp.run_experiment().await.unwrap();
        assert_eq!(result.hypothesis_id, expected_id);
    }

    #[test]
    fn test_experimenter_parent() {
        let parent = AgentId::new();
        let parent_clone = parent.clone();
        let exp = ExperimenterAgent::new(parent, "h1");
        assert_eq!(exp.parent, parent_clone);
    }
}
