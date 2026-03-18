use std::sync::Arc;

use tokio::sync::Mutex;
use tracing::info;

use crate::spawn::AgentSpawner;
use crate::types::{AgentConfig, AgentError, AgentId, AgentInfo, AgentStatus, AgentType};

/// The Coordinator agent — PID 0, Raft leader, orchestrates all other agents.
pub struct CoordinatorAgent {
    pub id: AgentId,
    spawner: Arc<Mutex<AgentSpawner>>,
    status: AgentStatus,
}

impl CoordinatorAgent {
    /// Create a new Coordinator and register it in the spawner.
    pub async fn new(spawner: Arc<Mutex<AgentSpawner>>) -> Result<Self, AgentError> {
        let config = AgentConfig {
            agent_type: AgentType::Coordinator,
            name: Some("coordinator-0".into()),
            task: Some("Orchestrate swarm".into()),
            zone: Some("A".into()),
            max_runtime_secs: None,
        };

        let id = spawner.lock().await.spawn(config, None)?;

        Ok(Self {
            id,
            spawner,
            status: AgentStatus::Running,
        })
    }

    /// Start the coordinator event loop (placeholder for future async work).
    pub async fn start(&mut self) -> Result<(), AgentError> {
        self.status = AgentStatus::Running;
        info!(coordinator_id = %self.id.0, "Coordinator started");
        Ok(())
    }

    /// Spawn a Researcher to handle a research request.
    pub async fn handle_research_request(&self, topic: &str) -> Result<AgentId, AgentError> {
        let config = AgentConfig {
            agent_type: AgentType::Researcher,
            name: Some(format!("researcher-{}", &topic[..topic.len().min(16)])),
            task: Some(topic.to_string()),
            zone: None,
            max_runtime_secs: Some(300),
        };

        let id = self.spawner.lock().await.spawn(config, Some(&self.id))?;

        info!(researcher_id = %id.0, topic = topic, "Spawned researcher");
        Ok(id)
    }

    /// Spawn a Worker to handle a generic task.
    pub async fn handle_task(&self, task: &str) -> Result<serde_json::Value, AgentError> {
        let config = AgentConfig {
            agent_type: AgentType::Worker,
            name: None,
            task: Some(task.to_string()),
            zone: None,
            max_runtime_secs: Some(60),
        };

        let worker_id = self.spawner.lock().await.spawn(config, Some(&self.id))?;

        info!(worker_id = %worker_id.0, task = task, "Delegated task to worker");

        Ok(serde_json::json!({
            "status": "delegated",
            "worker_id": worker_id.0.to_string(),
            "task": task,
        }))
    }

    /// Return info about active (non-terminated) children.
    pub async fn active_children(&self) -> Vec<AgentInfo> {
        let spawner = self.spawner.lock().await;
        let coord = match spawner.get(&self.id) {
            Some(c) => c,
            None => return Vec::new(),
        };

        coord
            .children
            .iter()
            .filter_map(|child_id| spawner.get(child_id))
            .filter(|info| info.status != AgentStatus::Terminated)
            .cloned()
            .collect()
    }

    pub fn status(&self) -> AgentStatus {
        self.status
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_coordinator_creation() {
        let spawner = Arc::new(Mutex::new(AgentSpawner::new()));
        let coord = CoordinatorAgent::new(spawner.clone()).await.unwrap();
        assert_eq!(coord.status(), AgentStatus::Running);

        let s = spawner.lock().await;
        let info = s.get(&coord.id).unwrap();
        assert_eq!(info.agent_type, AgentType::Coordinator);
    }

    #[tokio::test]
    async fn test_handle_research_request() {
        let spawner = Arc::new(Mutex::new(AgentSpawner::new()));
        let coord = CoordinatorAgent::new(spawner.clone()).await.unwrap();
        let researcher_id = coord
            .handle_research_request("quantum computing")
            .await
            .unwrap();

        let s = spawner.lock().await;
        let info = s.get(&researcher_id).unwrap();
        assert_eq!(info.agent_type, AgentType::Researcher);
        assert_eq!(info.task.as_deref(), Some("quantum computing"));
    }

    #[tokio::test]
    async fn test_handle_task() {
        let spawner = Arc::new(Mutex::new(AgentSpawner::new()));
        let coord = CoordinatorAgent::new(spawner.clone()).await.unwrap();
        let result = coord.handle_task("compile project").await.unwrap();
        assert_eq!(result["status"], "delegated");
        assert_eq!(result["task"], "compile project");
    }

    #[tokio::test]
    async fn test_active_children() {
        let spawner = Arc::new(Mutex::new(AgentSpawner::new()));
        let coord = CoordinatorAgent::new(spawner.clone()).await.unwrap();
        coord.handle_task("task1").await.unwrap();
        coord.handle_task("task2").await.unwrap();

        let children = coord.active_children().await;
        assert_eq!(children.len(), 2);
    }

    #[tokio::test]
    async fn test_active_children_excludes_terminated() {
        let spawner = Arc::new(Mutex::new(AgentSpawner::new()));
        let coord = CoordinatorAgent::new(spawner.clone()).await.unwrap();
        let worker_id = {
            let result = coord.handle_task("task1").await.unwrap();
            let id_str = result["worker_id"].as_str().unwrap();
            let uuid = uuid::Uuid::parse_str(id_str).unwrap();
            AgentId(uuid)
        };
        coord.handle_task("task2").await.unwrap();

        spawner.lock().await.terminate(&worker_id).unwrap();

        let children = coord.active_children().await;
        assert_eq!(children.len(), 1);
    }
}
