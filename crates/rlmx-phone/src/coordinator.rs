use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::battery::BatteryPolicy;

/// The 5 always-on free agent types that run on-device with zero network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FreeAgentType {
    EmailTriage,
    Calendar,
    WeatherCommute,
    NewsDigest,
    ShoppingComparison,
}

impl FreeAgentType {
    /// All free agent types as a slice.
    pub fn all() -> &'static [FreeAgentType] {
        &[
            FreeAgentType::EmailTriage,
            FreeAgentType::Calendar,
            FreeAgentType::WeatherCommute,
            FreeAgentType::NewsDigest,
            FreeAgentType::ShoppingComparison,
        ]
    }

    /// Human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            FreeAgentType::EmailTriage => "Email Triage",
            FreeAgentType::Calendar => "Calendar",
            FreeAgentType::WeatherCommute => "Weather + Commute",
            FreeAgentType::NewsDigest => "News Digest",
            FreeAgentType::ShoppingComparison => "Shopping Comparison",
        }
    }
}

/// Status of a local agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Running,
    Suspended,
    Starting,
}

/// An agent running on-device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAgent {
    pub id: Uuid,
    pub agent_type: String,
    pub status: AgentStatus,
    pub is_free_tier: bool,
}

/// On-device swarm coordinator managing local agents within mobile OS limits.
///
/// The coordinator caps concurrency based on battery level (3-8 agents) and
/// ensures the 5 free-tier agents are always available regardless of
/// subscription state (invariant 7 from DDD-009).
#[derive(Debug)]
pub struct LightweightCoordinator {
    pub active_agents: Vec<LocalAgent>,
    pub max_concurrent: u8,
}

impl LightweightCoordinator {
    /// Create a new coordinator with the given concurrency limit.
    pub fn new(max_concurrent: u8) -> Self {
        Self {
            active_agents: Vec::new(),
            max_concurrent,
        }
    }

    /// Update concurrency limit based on battery policy.
    pub fn apply_battery_policy(&mut self, policy: BatteryPolicy) {
        self.max_concurrent = match policy {
            BatteryPolicy::Full => 8,
            BatteryPolicy::Balanced => 5,
            BatteryPolicy::LowPower => 3,
            BatteryPolicy::Critical => 0,
        };
    }

    /// Attempt to start a new agent. Returns the agent id on success.
    ///
    /// Free-tier agents are always allowed (invariant 7).
    /// Non-free agents respect the concurrency limit.
    pub fn start_agent(
        &mut self,
        agent_type: &str,
        is_free_tier: bool,
    ) -> Result<Uuid, CoordinatorError> {
        let active_count = self
            .active_agents
            .iter()
            .filter(|a| a.status == AgentStatus::Running)
            .count() as u8;

        if !is_free_tier && active_count >= self.max_concurrent {
            return Err(CoordinatorError::ConcurrencyLimitReached {
                max: self.max_concurrent,
                current: active_count,
            });
        }

        let agent = LocalAgent {
            id: Uuid::new_v4(),
            agent_type: agent_type.to_string(),
            status: AgentStatus::Running,
            is_free_tier,
        };
        let id = agent.id;
        self.active_agents.push(agent);
        tracing::info!(agent_id = %id, agent_type, "agent started");
        Ok(id)
    }

    /// Suspend an agent by id.
    pub fn suspend_agent(&mut self, agent_id: Uuid, reason: &str) -> Result<(), CoordinatorError> {
        let agent = self
            .active_agents
            .iter_mut()
            .find(|a| a.id == agent_id)
            .ok_or(CoordinatorError::AgentNotFound(agent_id))?;

        agent.status = AgentStatus::Suspended;
        tracing::info!(agent_id = %agent_id, reason, "agent suspended");
        Ok(())
    }

    /// Suspend all non-free-tier agents (used under Critical battery policy).
    pub fn suspend_non_essential(&mut self) -> Vec<Uuid> {
        let mut suspended = Vec::new();
        for agent in &mut self.active_agents {
            if !agent.is_free_tier && agent.status == AgentStatus::Running {
                agent.status = AgentStatus::Suspended;
                suspended.push(agent.id);
            }
        }
        if !suspended.is_empty() {
            tracing::info!(count = suspended.len(), "suspended non-essential agents");
        }
        suspended
    }

    /// Count of actively running agents.
    pub fn running_count(&self) -> usize {
        self.active_agents
            .iter()
            .filter(|a| a.status == AgentStatus::Running)
            .count()
    }

    /// Spawn all 5 free-tier agents.
    pub fn spawn_free_agents(&mut self) -> Vec<Uuid> {
        FreeAgentType::all()
            .iter()
            .filter_map(|ft| {
                // Skip if already active.
                let already_running = self.active_agents.iter().any(|a| {
                    a.agent_type == ft.name() && a.is_free_tier && a.status == AgentStatus::Running
                });
                if already_running {
                    return None;
                }
                self.start_agent(ft.name(), true).ok()
            })
            .collect()
    }
}

/// Errors from the lightweight coordinator.
#[derive(Debug, thiserror::Error)]
pub enum CoordinatorError {
    #[error("concurrency limit reached: {current}/{max} agents running")]
    ConcurrencyLimitReached { max: u8, current: u8 },

    #[error("agent not found: {0}")]
    AgentNotFound(Uuid),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_free_agents() {
        let mut coord = LightweightCoordinator::new(8);
        let ids = coord.spawn_free_agents();
        assert_eq!(ids.len(), 5);
        assert_eq!(coord.running_count(), 5);
    }

    #[test]
    fn test_free_agents_bypass_limit() {
        let mut coord = LightweightCoordinator::new(0); // zero limit
                                                        // Free agents should still start.
        let result = coord.start_agent("Email Triage", true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_non_free_respects_limit() {
        let mut coord = LightweightCoordinator::new(1);
        coord.start_agent("custom-agent", false).unwrap();
        let result = coord.start_agent("another-agent", false);
        assert!(result.is_err());
    }

    #[test]
    fn test_suspend_agent() {
        let mut coord = LightweightCoordinator::new(8);
        let id = coord.start_agent("test", false).unwrap();
        coord.suspend_agent(id, "battery low").unwrap();
        assert_eq!(coord.running_count(), 0);
    }

    #[test]
    fn test_suspend_non_essential() {
        let mut coord = LightweightCoordinator::new(8);
        coord.spawn_free_agents();
        coord.start_agent("premium-1", false).unwrap();
        coord.start_agent("premium-2", false).unwrap();
        let suspended = coord.suspend_non_essential();
        assert_eq!(suspended.len(), 2);
        // Free agents still running.
        assert_eq!(coord.running_count(), 5);
    }

    #[test]
    fn test_apply_battery_policy() {
        let mut coord = LightweightCoordinator::new(8);
        coord.apply_battery_policy(BatteryPolicy::LowPower);
        assert_eq!(coord.max_concurrent, 3);
    }
}
