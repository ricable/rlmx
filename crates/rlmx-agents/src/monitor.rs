use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::AgentId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthCheckStatus {
    Healthy,
    Degraded,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHealth {
    pub agent_id: AgentId,
    pub last_heartbeat: DateTime<Utc>,
    pub consecutive_failures: u32,
    pub status: HealthCheckStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub agent_id: AgentId,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

/// Monitor agent — watches other agents for health and generates alerts.
pub struct MonitorAgent {
    pub id: AgentId,
    watched_agents: HashMap<AgentId, AgentHealth>,
    alerts: Vec<Alert>,
    /// Heartbeat timeout in seconds before degraded.
    heartbeat_timeout_secs: i64,
    /// Consecutive failures before critical.
    failure_threshold: u32,
}

impl MonitorAgent {
    pub fn new() -> Self {
        Self {
            id: AgentId::new(),
            watched_agents: HashMap::new(),
            alerts: Vec::new(),
            heartbeat_timeout_secs: 30,
            failure_threshold: 3,
        }
    }

    /// Start watching an agent.
    pub fn watch(&mut self, agent_id: AgentId) {
        let health = AgentHealth {
            agent_id: agent_id.clone(),
            last_heartbeat: Utc::now(),
            consecutive_failures: 0,
            status: HealthCheckStatus::Healthy,
        };
        self.watched_agents.insert(agent_id, health);
    }

    /// Record a heartbeat from an agent.
    pub fn record_heartbeat(&mut self, agent_id: &AgentId) {
        if let Some(health) = self.watched_agents.get_mut(agent_id) {
            health.last_heartbeat = Utc::now();
            health.consecutive_failures = 0;
            health.status = HealthCheckStatus::Healthy;
        }
    }

    /// Check health of all watched agents and generate alerts.
    pub fn check_health(&mut self) -> Vec<Alert> {
        let now = Utc::now();
        let mut new_alerts = Vec::new();

        for health in self.watched_agents.values_mut() {
            let elapsed = now
                .signed_duration_since(health.last_heartbeat)
                .num_seconds();

            if elapsed > self.heartbeat_timeout_secs * 2 {
                health.consecutive_failures += 1;
                if health.consecutive_failures >= self.failure_threshold {
                    health.status = HealthCheckStatus::Failed;
                    new_alerts.push(Alert {
                        agent_id: health.agent_id.clone(),
                        severity: AlertSeverity::Critical,
                        message: format!(
                            "Agent {} failed: {} consecutive failures, last heartbeat {}s ago",
                            health.agent_id.0, health.consecutive_failures, elapsed
                        ),
                        timestamp: now,
                    });
                } else {
                    health.status = HealthCheckStatus::Degraded;
                    new_alerts.push(Alert {
                        agent_id: health.agent_id.clone(),
                        severity: AlertSeverity::Warning,
                        message: format!(
                            "Agent {} degraded: no heartbeat for {}s",
                            health.agent_id.0, elapsed
                        ),
                        timestamp: now,
                    });
                }
            } else if elapsed > self.heartbeat_timeout_secs {
                health.status = HealthCheckStatus::Degraded;
                new_alerts.push(Alert {
                    agent_id: health.agent_id.clone(),
                    severity: AlertSeverity::Info,
                    message: format!(
                        "Agent {} heartbeat delayed: {}s",
                        health.agent_id.0, elapsed
                    ),
                    timestamp: now,
                });
            }
        }

        self.alerts.extend(new_alerts.clone());
        new_alerts
    }

    /// Get all accumulated alerts.
    pub fn alerts(&self) -> &[Alert] {
        &self.alerts
    }

    /// Get health status for a specific agent.
    pub fn agent_health(&self, agent_id: &AgentId) -> Option<&AgentHealth> {
        self.watched_agents.get(agent_id)
    }

    /// Number of agents being watched.
    pub fn watched_count(&self) -> usize {
        self.watched_agents.len()
    }
}

impl Default for MonitorAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watch_agent() {
        let mut monitor = MonitorAgent::new();
        let agent_id = AgentId::new();
        monitor.watch(agent_id.clone());
        assert_eq!(monitor.watched_count(), 1);
        let health = monitor.agent_health(&agent_id).unwrap();
        assert_eq!(health.status, HealthCheckStatus::Healthy);
    }

    #[test]
    fn test_record_heartbeat() {
        let mut monitor = MonitorAgent::new();
        let agent_id = AgentId::new();
        monitor.watch(agent_id.clone());
        monitor.record_heartbeat(&agent_id);
        let health = monitor.agent_health(&agent_id).unwrap();
        assert_eq!(health.consecutive_failures, 0);
        assert_eq!(health.status, HealthCheckStatus::Healthy);
    }

    #[test]
    fn test_healthy_agents_no_alerts() {
        let mut monitor = MonitorAgent::new();
        let agent_id = AgentId::new();
        monitor.watch(agent_id);
        let alerts = monitor.check_health();
        assert!(alerts.is_empty());
    }

    #[test]
    fn test_check_health_detects_stale() {
        let mut monitor = MonitorAgent::new();
        monitor.heartbeat_timeout_secs = 0; // Immediate timeout for testing
        let agent_id = AgentId::new();
        monitor.watch(agent_id.clone());

        // Manually set old heartbeat
        if let Some(health) = monitor.watched_agents.get_mut(&agent_id) {
            health.last_heartbeat = Utc::now() - chrono::Duration::seconds(120);
        }

        let alerts = monitor.check_health();
        assert!(!alerts.is_empty());
    }

    #[test]
    fn test_alerts_accumulate() {
        let mut monitor = MonitorAgent::new();
        monitor.heartbeat_timeout_secs = 0;
        let agent_id = AgentId::new();
        monitor.watch(agent_id.clone());

        if let Some(health) = monitor.watched_agents.get_mut(&agent_id) {
            health.last_heartbeat = Utc::now() - chrono::Duration::seconds(120);
        }

        monitor.check_health();
        monitor.check_health();
        assert!(monitor.alerts().len() >= 2);
    }

    #[test]
    fn test_heartbeat_resets_failures() {
        let mut monitor = MonitorAgent::new();
        let agent_id = AgentId::new();
        monitor.watch(agent_id.clone());

        // Simulate failures
        if let Some(health) = monitor.watched_agents.get_mut(&agent_id) {
            health.consecutive_failures = 5;
            health.status = HealthCheckStatus::Failed;
        }

        monitor.record_heartbeat(&agent_id);
        let health = monitor.agent_health(&agent_id).unwrap();
        assert_eq!(health.consecutive_failures, 0);
        assert_eq!(health.status, HealthCheckStatus::Healthy);
    }

    #[test]
    fn test_multiple_watched_agents() {
        let mut monitor = MonitorAgent::new();
        monitor.watch(AgentId::new());
        monitor.watch(AgentId::new());
        monitor.watch(AgentId::new());
        assert_eq!(monitor.watched_count(), 3);
    }
}
