use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::AgentId;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TrainingStatus {
    Queued,
    Running,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingMetrics {
    pub epoch: u32,
    pub loss: f64,
    pub val_loss: f64,
    pub learning_rate: f64,
}

impl Default for TrainingMetrics {
    fn default() -> Self {
        Self {
            epoch: 0,
            loss: f64::MAX,
            val_loss: f64::MAX,
            learning_rate: 0.001,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingJob {
    pub id: Uuid,
    pub model_name: String,
    pub status: TrainingStatus,
    pub metrics: TrainingMetrics,
    pub started_at: DateTime<Utc>,
    pub config: serde_json::Value,
}

/// Trainer agent — manages model training jobs.
pub struct TrainerAgent {
    pub id: AgentId,
    pub training_jobs: Vec<TrainingJob>,
}

impl TrainerAgent {
    pub fn new() -> Self {
        Self {
            id: AgentId::new(),
            training_jobs: Vec::new(),
        }
    }

    /// Submit a new training job.
    pub fn submit_job(&mut self, model_name: impl Into<String>, config: serde_json::Value) -> Uuid {
        let id = Uuid::new_v4();
        let job = TrainingJob {
            id,
            model_name: model_name.into(),
            status: TrainingStatus::Queued,
            metrics: TrainingMetrics::default(),
            started_at: Utc::now(),
            config,
        };
        self.training_jobs.push(job);
        id
    }

    /// Get the status of a training job.
    pub fn job_status(&self, id: Uuid) -> Option<&TrainingJob> {
        self.training_jobs.iter().find(|j| j.id == id)
    }

    /// List all training jobs.
    pub fn list_jobs(&self) -> &[TrainingJob] {
        &self.training_jobs
    }

    /// Update a job's status and metrics.
    pub fn update_job(
        &mut self,
        id: Uuid,
        status: TrainingStatus,
        metrics: TrainingMetrics,
    ) -> bool {
        if let Some(job) = self.training_jobs.iter_mut().find(|j| j.id == id) {
            job.status = status;
            job.metrics = metrics;
            true
        } else {
            false
        }
    }

    /// Count jobs by status.
    pub fn count_by_status(&self, status: &TrainingStatus) -> usize {
        self.training_jobs
            .iter()
            .filter(|j| std::mem::discriminant(&j.status) == std::mem::discriminant(status))
            .count()
    }
}

impl Default for TrainerAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_submit_job() {
        let mut trainer = TrainerAgent::new();
        let config = serde_json::json!({"epochs": 10, "batch_size": 32});
        let id = trainer.submit_job("test-model", config);
        let job = trainer.job_status(id).unwrap();
        assert_eq!(job.model_name, "test-model");
        assert_eq!(job.status, TrainingStatus::Queued);
    }

    #[test]
    fn test_list_jobs() {
        let mut trainer = TrainerAgent::new();
        trainer.submit_job("model-a", serde_json::json!({}));
        trainer.submit_job("model-b", serde_json::json!({}));
        assert_eq!(trainer.list_jobs().len(), 2);
    }

    #[test]
    fn test_update_job() {
        let mut trainer = TrainerAgent::new();
        let id = trainer.submit_job("model", serde_json::json!({}));

        let metrics = TrainingMetrics {
            epoch: 5,
            loss: 0.1,
            val_loss: 0.15,
            learning_rate: 0.0001,
        };

        assert!(trainer.update_job(id, TrainingStatus::Running, metrics));
        let job = trainer.job_status(id).unwrap();
        assert_eq!(job.status, TrainingStatus::Running);
        assert_eq!(job.metrics.epoch, 5);
    }

    #[test]
    fn test_update_nonexistent_job() {
        let mut trainer = TrainerAgent::new();
        assert!(!trainer.update_job(
            Uuid::new_v4(),
            TrainingStatus::Running,
            TrainingMetrics::default()
        ));
    }

    #[test]
    fn test_count_by_status() {
        let mut trainer = TrainerAgent::new();
        trainer.submit_job("a", serde_json::json!({}));
        trainer.submit_job("b", serde_json::json!({}));
        assert_eq!(trainer.count_by_status(&TrainingStatus::Queued), 2);
        assert_eq!(trainer.count_by_status(&TrainingStatus::Running), 0);
    }

    #[test]
    fn test_job_not_found() {
        let trainer = TrainerAgent::new();
        assert!(trainer.job_status(Uuid::new_v4()).is_none());
    }

    #[test]
    fn test_complete_job() {
        let mut trainer = TrainerAgent::new();
        let id = trainer.submit_job("model", serde_json::json!({}));
        let metrics = TrainingMetrics {
            epoch: 10,
            loss: 0.01,
            val_loss: 0.02,
            learning_rate: 0.00001,
        };
        trainer.update_job(id, TrainingStatus::Completed, metrics);
        let job = trainer.job_status(id).unwrap();
        assert_eq!(job.status, TrainingStatus::Completed);
        assert!(job.metrics.loss < 0.1);
    }
}
