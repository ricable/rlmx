use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::types::NodeId;

/// Health status of a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// Severity of a health alert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// A health alert for a node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthAlert {
    pub node_id: NodeId,
    pub severity: AlertSeverity,
    pub message: String,
}

/// Per-node health tracking data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeHealth {
    pub node_id: NodeId,
    pub consecutive_failures: u32,
    pub latency_ms: Vec<u64>,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub last_check: DateTime<Utc>,
    pub status: HealthStatus,
}

impl NodeHealth {
    fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
            consecutive_failures: 0,
            latency_ms: Vec::new(),
            cpu_usage: 0.0,
            memory_usage: 0.0,
            last_check: Utc::now(),
            status: HealthStatus::Unknown,
        }
    }
}

/// Monitors the health of all nodes in the swarm.
#[derive(Debug, Clone)]
pub struct HealthMonitor {
    pub nodes: HashMap<NodeId, NodeHealth>,
    pub check_interval: Duration,
    pub failure_threshold: u32,
}

impl HealthMonitor {
    pub fn new(interval: Duration, threshold: u32) -> Self {
        Self {
            nodes: HashMap::new(),
            check_interval: interval,
            failure_threshold: threshold,
        }
    }

    /// Record a successful heartbeat from a node.
    pub fn record_heartbeat(&mut self, node_id: NodeId, latency_ms: u64) {
        let health = self
            .nodes
            .entry(node_id)
            .or_insert_with(|| NodeHealth::new(node_id));
        health.consecutive_failures = 0;
        health.last_check = Utc::now();

        // Keep last 100 latency samples.
        if health.latency_ms.len() >= 100 {
            health.latency_ms.remove(0);
        }
        health.latency_ms.push(latency_ms);

        health.status = if latency_ms > 500 {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        };
    }

    /// Record a failed health check for a node.
    pub fn record_failure(&mut self, node_id: NodeId) {
        let health = self
            .nodes
            .entry(node_id)
            .or_insert_with(|| NodeHealth::new(node_id));
        health.consecutive_failures += 1;
        health.last_check = Utc::now();

        health.status = if health.consecutive_failures >= self.failure_threshold {
            HealthStatus::Critical
        } else {
            HealthStatus::Warning
        };
    }

    /// Check all nodes and return alerts for those in bad state.
    pub fn check_all(&self) -> Vec<HealthAlert> {
        let mut alerts = Vec::new();
        for health in self.nodes.values() {
            match health.status {
                HealthStatus::Warning => {
                    alerts.push(HealthAlert {
                        node_id: health.node_id,
                        severity: AlertSeverity::Warning,
                        message: format!(
                            "node {} has {} consecutive failures",
                            health.node_id, health.consecutive_failures
                        ),
                    });
                }
                HealthStatus::Critical => {
                    alerts.push(HealthAlert {
                        node_id: health.node_id,
                        severity: AlertSeverity::Critical,
                        message: format!(
                            "node {} critical: {} consecutive failures (threshold: {})",
                            health.node_id, health.consecutive_failures, self.failure_threshold
                        ),
                    });
                }
                _ => {}
            }
        }
        alerts
    }

    /// Get the health status of a specific node.
    pub fn get_status(&self, node_id: &NodeId) -> HealthStatus {
        self.nodes
            .get(node_id)
            .map(|h| h.status)
            .unwrap_or(HealthStatus::Unknown)
    }

    /// Average latency for a node (returns 0.0 if no samples).
    pub fn average_latency(&self, node_id: &NodeId) -> f64 {
        self.nodes
            .get(node_id)
            .and_then(|h| {
                if h.latency_ms.is_empty() {
                    None
                } else {
                    let sum: u64 = h.latency_ms.iter().sum();
                    Some(sum as f64 / h.latency_ms.len() as f64)
                }
            })
            .unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_heartbeat_healthy() {
        let mut monitor = HealthMonitor::new(Duration::from_secs(5), 3);
        let nid = NodeId::new();
        monitor.record_heartbeat(nid, 10);
        assert_eq!(monitor.get_status(&nid), HealthStatus::Healthy);
    }

    #[test]
    fn test_record_heartbeat_high_latency_warning() {
        let mut monitor = HealthMonitor::new(Duration::from_secs(5), 3);
        let nid = NodeId::new();
        monitor.record_heartbeat(nid, 600);
        assert_eq!(monitor.get_status(&nid), HealthStatus::Warning);
    }

    #[test]
    fn test_record_failure_warning_then_critical() {
        let mut monitor = HealthMonitor::new(Duration::from_secs(5), 3);
        let nid = NodeId::new();
        monitor.record_failure(nid);
        assert_eq!(monitor.get_status(&nid), HealthStatus::Warning);
        monitor.record_failure(nid);
        assert_eq!(monitor.get_status(&nid), HealthStatus::Warning);
        monitor.record_failure(nid);
        assert_eq!(monitor.get_status(&nid), HealthStatus::Critical);
    }

    #[test]
    fn test_heartbeat_resets_failures() {
        let mut monitor = HealthMonitor::new(Duration::from_secs(5), 3);
        let nid = NodeId::new();
        monitor.record_failure(nid);
        monitor.record_failure(nid);
        monitor.record_heartbeat(nid, 5);
        assert_eq!(monitor.get_status(&nid), HealthStatus::Healthy);
        assert_eq!(monitor.nodes[&nid].consecutive_failures, 0);
    }

    #[test]
    fn test_check_all_alerts() {
        let mut monitor = HealthMonitor::new(Duration::from_secs(5), 2);
        let n1 = NodeId::new();
        let n2 = NodeId::new();
        monitor.record_heartbeat(n1, 5); // healthy
        monitor.record_failure(n2); // warning
        monitor.record_failure(n2); // critical

        let alerts = monitor.check_all();
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].severity, AlertSeverity::Critical);
    }

    #[test]
    fn test_average_latency() {
        let mut monitor = HealthMonitor::new(Duration::from_secs(5), 3);
        let nid = NodeId::new();
        monitor.record_heartbeat(nid, 10);
        monitor.record_heartbeat(nid, 20);
        monitor.record_heartbeat(nid, 30);
        assert!((monitor.average_latency(&nid) - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_average_latency_unknown_node() {
        let monitor = HealthMonitor::new(Duration::from_secs(5), 3);
        assert!((monitor.average_latency(&NodeId::new()) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_unknown_node_status() {
        let monitor = HealthMonitor::new(Duration::from_secs(5), 3);
        assert_eq!(monitor.get_status(&NodeId::new()), HealthStatus::Unknown);
    }
}
