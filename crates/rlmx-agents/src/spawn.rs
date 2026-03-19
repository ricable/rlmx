use std::collections::HashMap;

use chrono::Utc;
use tokio::sync::mpsc;
use tracing::info;

use crate::registry::PermissionRegistry;
use crate::types::{
    AgentConfig, AgentError, AgentId, AgentInfo, AgentMessage, AgentMetrics, AgentStatus, AgentType,
};

/// Manages agent lifecycle: spawn, terminate, messaging.
pub struct AgentSpawner {
    agents: HashMap<AgentId, AgentInfo>,
    message_senders: HashMap<AgentId, mpsc::Sender<AgentMessage>>,
    message_receivers: HashMap<AgentId, mpsc::Receiver<AgentMessage>>,
}

impl AgentSpawner {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            message_senders: HashMap::new(),
            message_receivers: HashMap::new(),
        }
    }

    /// Spawn a new agent, validating max instances and spawn hierarchy.
    pub fn spawn(
        &mut self,
        config: AgentConfig,
        parent: Option<&AgentId>,
    ) -> Result<AgentId, AgentError> {
        // Validate max instances
        let current_count = self.count_by_type(config.agent_type);
        let max = config.agent_type.max_instances();
        if current_count >= max {
            return Err(AgentError::MaxInstancesExceeded {
                agent_type: config.agent_type,
                max,
            });
        }

        // Validate spawn hierarchy if parent is specified
        let parent_info = if let Some(parent_id) = parent {
            let parent_agent = self
                .agents
                .get(parent_id)
                .ok_or_else(|| AgentError::NotFound(format!("Parent agent {}", parent_id.0)))?;
            if !PermissionRegistry::can_spawn(parent_agent.agent_type, config.agent_type) {
                return Err(AgentError::SpawnNotAllowed {
                    parent: parent_agent.agent_type,
                    child: config.agent_type,
                });
            }
            Some(parent_agent.id.clone())
        } else {
            None
        };

        let id = AgentId::new();
        let zone = config
            .zone
            .unwrap_or_else(|| config.agent_type.preferred_zone().to_string());
        let name = config
            .name
            .unwrap_or_else(|| format!("{}-{}", config.agent_type, &id.0.to_string()[..8]));

        let info = AgentInfo {
            id: id.clone(),
            agent_type: config.agent_type,
            name,
            status: AgentStatus::Running,
            task: config.task,
            zone,
            spawned_at: Utc::now(),
            parent_id: parent_info.clone(),
            children: Vec::new(),
            model_tier: config.agent_type.model_tier(),
            metrics: AgentMetrics::default(),
        };

        // Create message channel
        let (tx, rx) = mpsc::channel(256);
        self.message_senders.insert(id.clone(), tx);
        self.message_receivers.insert(id.clone(), rx);

        info!(agent_id = %id.0, agent_type = %config.agent_type, "Agent spawned");

        self.agents.insert(id.clone(), info);

        // Register as child of parent
        if let Some(parent_id) = parent {
            if let Some(parent_agent) = self.agents.get_mut(parent_id) {
                parent_agent.children.push(id.clone());
            }
        }

        Ok(id)
    }

    /// Terminate an agent by id.
    pub fn terminate(&mut self, id: &AgentId) -> Result<(), AgentError> {
        let agent = self
            .agents
            .get_mut(id)
            .ok_or_else(|| AgentError::NotFound(id.0.to_string()))?;

        if agent.status == AgentStatus::Terminated {
            return Err(AgentError::AlreadyTerminated);
        }

        agent.status = AgentStatus::Terminated;
        self.message_senders.remove(id);
        self.message_receivers.remove(id);

        info!(agent_id = %id.0, "Agent terminated");
        Ok(())
    }

    /// Get agent info by id.
    pub fn get(&self, id: &AgentId) -> Option<&AgentInfo> {
        self.agents.get(id)
    }

    /// List all agents.
    pub fn list(&self) -> Vec<&AgentInfo> {
        self.agents.values().collect()
    }

    /// List agents by type.
    pub fn list_by_type(&self, agent_type: AgentType) -> Vec<&AgentInfo> {
        self.agents
            .values()
            .filter(|a| a.agent_type == agent_type)
            .collect()
    }

    /// Count running agents of a given type.
    pub fn count_by_type(&self, agent_type: AgentType) -> usize {
        self.agents
            .values()
            .filter(|a| a.agent_type == agent_type && a.status != AgentStatus::Terminated)
            .count()
    }

    /// Send a message to an agent.
    pub async fn send_message(&self, msg: AgentMessage) -> Result<(), AgentError> {
        let sender = self
            .message_senders
            .get(&msg.to)
            .ok_or_else(|| AgentError::NotFound(msg.to.0.to_string()))?;

        sender
            .send(msg)
            .await
            .map_err(|e| AgentError::Internal(format!("Failed to send message: {e}")))?;

        Ok(())
    }
}

impl Default for AgentSpawner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn worker_config() -> AgentConfig {
        AgentConfig {
            agent_type: AgentType::Worker,
            name: Some("test-worker".into()),
            task: Some("test task".into()),
            zone: None,
            max_runtime_secs: None,
        }
    }

    fn coordinator_config() -> AgentConfig {
        AgentConfig {
            agent_type: AgentType::Coordinator,
            name: Some("coordinator".into()),
            task: None,
            zone: None,
            max_runtime_secs: None,
        }
    }

    #[test]
    fn test_spawn_agent() {
        let mut spawner = AgentSpawner::new();
        let id = spawner.spawn(worker_config(), None).unwrap();
        let info = spawner.get(&id).unwrap();
        assert_eq!(info.agent_type, AgentType::Worker);
        assert_eq!(info.status, AgentStatus::Running);
        assert_eq!(info.zone, "Multi"); // Worker preferred zone
    }

    #[test]
    fn test_spawn_with_custom_zone() {
        let mut spawner = AgentSpawner::new();
        let config = AgentConfig {
            zone: Some("X".into()),
            ..worker_config()
        };
        let id = spawner.spawn(config, None).unwrap();
        assert_eq!(spawner.get(&id).unwrap().zone, "X");
    }

    #[test]
    fn test_terminate_agent() {
        let mut spawner = AgentSpawner::new();
        let id = spawner.spawn(worker_config(), None).unwrap();
        spawner.terminate(&id).unwrap();
        assert_eq!(spawner.get(&id).unwrap().status, AgentStatus::Terminated);
    }

    #[test]
    fn test_terminate_already_terminated() {
        let mut spawner = AgentSpawner::new();
        let id = spawner.spawn(worker_config(), None).unwrap();
        spawner.terminate(&id).unwrap();
        assert!(matches!(
            spawner.terminate(&id),
            Err(AgentError::AlreadyTerminated)
        ));
    }

    #[test]
    fn test_max_instances_enforced() {
        let mut spawner = AgentSpawner::new();
        // Coordinator max is 1
        spawner.spawn(coordinator_config(), None).unwrap();
        let result = spawner.spawn(coordinator_config(), None);
        assert!(matches!(
            result,
            Err(AgentError::MaxInstancesExceeded { .. })
        ));
    }

    #[test]
    fn test_terminated_agents_dont_count() {
        let mut spawner = AgentSpawner::new();
        let id = spawner.spawn(coordinator_config(), None).unwrap();
        spawner.terminate(&id).unwrap();
        // Should be able to spawn another since previous is terminated
        spawner.spawn(coordinator_config(), None).unwrap();
    }

    #[test]
    fn test_spawn_hierarchy_valid() {
        let mut spawner = AgentSpawner::new();
        let coord_id = spawner.spawn(coordinator_config(), None).unwrap();
        let worker_id = spawner.spawn(worker_config(), Some(&coord_id)).unwrap();

        // Verify parent-child relationship
        let coord = spawner.get(&coord_id).unwrap();
        assert!(coord.children.contains(&worker_id));

        let worker = spawner.get(&worker_id).unwrap();
        assert_eq!(worker.parent_id.as_ref().unwrap(), &coord_id);
    }

    #[test]
    fn test_spawn_hierarchy_invalid() {
        let mut spawner = AgentSpawner::new();
        let worker_id = spawner.spawn(worker_config(), None).unwrap();

        // Worker cannot spawn anything
        let result = spawner.spawn(worker_config(), Some(&worker_id));
        assert!(matches!(result, Err(AgentError::SpawnNotAllowed { .. })));
    }

    #[test]
    fn test_list_by_type() {
        let mut spawner = AgentSpawner::new();
        spawner.spawn(worker_config(), None).unwrap();
        spawner.spawn(worker_config(), None).unwrap();
        spawner.spawn(coordinator_config(), None).unwrap();

        assert_eq!(spawner.list_by_type(AgentType::Worker).len(), 2);
        assert_eq!(spawner.list_by_type(AgentType::Coordinator).len(), 1);
        assert_eq!(spawner.list_by_type(AgentType::Monitor).len(), 0);
    }

    #[test]
    fn test_list_all() {
        let mut spawner = AgentSpawner::new();
        spawner.spawn(worker_config(), None).unwrap();
        spawner.spawn(coordinator_config(), None).unwrap();
        assert_eq!(spawner.list().len(), 2);
    }

    #[test]
    fn test_spawn_not_found_parent() {
        let mut spawner = AgentSpawner::new();
        let fake_id = AgentId::new();
        let result = spawner.spawn(worker_config(), Some(&fake_id));
        assert!(matches!(result, Err(AgentError::NotFound(_))));
    }
}
