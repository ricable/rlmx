use chrono::{DateTime, Utc};
use tracing::info;

use crate::types::{AgentId, AgentStatus};

/// Worker agent — executes tasks, auto-terminates after idle timeout.
pub struct WorkerAgent {
    pub id: AgentId,
    pub parent: AgentId,
    pub status: AgentStatus,
    pub tasks_completed: u64,
    pub idle_since: Option<DateTime<Utc>>,
}

impl WorkerAgent {
    pub fn new(parent: AgentId) -> Self {
        Self {
            id: AgentId::new(),
            parent,
            status: AgentStatus::Running,
            tasks_completed: 0,
            idle_since: Some(Utc::now()),
        }
    }

    /// Execute a task and return the result.
    pub async fn execute_task(&mut self, task: &str) -> Result<serde_json::Value, String> {
        self.idle_since = None;
        self.status = AgentStatus::Running;

        info!(worker_id = %self.id.0, task = task, "Executing task");

        // Simulate task execution
        let result = serde_json::json!({
            "worker_id": self.id.0.to_string(),
            "task": task,
            "status": "completed",
            "timestamp": Utc::now().to_rfc3339(),
        });

        self.tasks_completed += 1;
        self.idle_since = Some(Utc::now());

        info!(worker_id = %self.id.0, tasks_completed = self.tasks_completed, "Task completed");
        Ok(result)
    }

    /// Check if the worker has been idle longer than the given timeout.
    pub fn is_idle_timeout(&self, timeout: std::time::Duration) -> bool {
        match self.idle_since {
            Some(since) => {
                let elapsed = Utc::now().signed_duration_since(since);
                elapsed.to_std().map(|d| d >= timeout).unwrap_or(false)
            }
            None => false, // Currently executing
        }
    }

    /// Default idle timeout: 5 minutes.
    pub fn default_idle_timeout() -> std::time::Duration {
        std::time::Duration::from_secs(300)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_task() {
        let parent = AgentId::new();
        let mut worker = WorkerAgent::new(parent);
        let result = worker.execute_task("build project").await.unwrap();
        assert_eq!(result["status"], "completed");
        assert_eq!(result["task"], "build project");
        assert_eq!(worker.tasks_completed, 1);
    }

    #[tokio::test]
    async fn test_multiple_tasks() {
        let parent = AgentId::new();
        let mut worker = WorkerAgent::new(parent);
        worker.execute_task("task 1").await.unwrap();
        worker.execute_task("task 2").await.unwrap();
        worker.execute_task("task 3").await.unwrap();
        assert_eq!(worker.tasks_completed, 3);
    }

    #[test]
    fn test_idle_timeout_not_exceeded() {
        let parent = AgentId::new();
        let worker = WorkerAgent::new(parent);
        // Just created, should not be timed out
        assert!(!worker.is_idle_timeout(std::time::Duration::from_secs(300)));
    }

    #[test]
    fn test_idle_timeout_zero_duration() {
        let parent = AgentId::new();
        let worker = WorkerAgent::new(parent);
        // With zero timeout, any idle time exceeds it
        assert!(worker.is_idle_timeout(std::time::Duration::ZERO));
    }

    #[test]
    fn test_not_idle_during_execution() {
        let parent = AgentId::new();
        let mut worker = WorkerAgent::new(parent);
        worker.idle_since = None; // Simulating active execution
        assert!(!worker.is_idle_timeout(std::time::Duration::ZERO));
    }

    #[test]
    fn test_default_idle_timeout() {
        assert_eq!(
            WorkerAgent::default_idle_timeout(),
            std::time::Duration::from_secs(300)
        );
    }
}
