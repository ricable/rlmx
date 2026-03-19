//! Marketplace analytics: install tracking, revenue metrics, retention.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::LifeDomain;

/// A single install event for analytics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallEvent {
    pub agent_id: Uuid,
    pub user_id: Uuid,
    pub domain: LifeDomain,
    pub timestamp: DateTime<Utc>,
}

/// An uninstall event for retention tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UninstallEvent {
    pub agent_id: Uuid,
    pub user_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

/// Per-agent analytics summary.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub total_installs: u64,
    pub total_uninstalls: u64,
    pub revenue_cents: u64,
}

impl AgentMetrics {
    /// Retention rate as a percentage.
    pub fn retention_rate(&self) -> f64 {
        if self.total_installs == 0 {
            return 0.0;
        }
        let active = self.total_installs.saturating_sub(self.total_uninstalls);
        (active as f64 / self.total_installs as f64) * 100.0
    }
}

/// Per-domain analytics summary.
#[derive(Debug, Clone, Default)]
pub struct DomainMetrics {
    pub total_installs: u64,
    pub total_revenue_cents: u64,
    pub agent_count: u64,
}

/// Marketplace-wide analytics engine.
#[derive(Debug, Default)]
pub struct MarketplaceAnalytics {
    agent_metrics: HashMap<Uuid, AgentMetrics>,
    installs: Vec<InstallEvent>,
    uninstalls: Vec<UninstallEvent>,
}

impl MarketplaceAnalytics {
    pub fn new() -> Self {
        Self {
            agent_metrics: HashMap::new(),
            installs: Vec::new(),
            uninstalls: Vec::new(),
        }
    }

    /// Record an agent installation.
    pub fn record_install(&mut self, agent_id: Uuid, user_id: Uuid, domain: LifeDomain) {
        let event = InstallEvent {
            agent_id,
            user_id,
            domain,
            timestamp: Utc::now(),
        };
        self.installs.push(event);

        let metrics = self.agent_metrics.entry(agent_id).or_default();
        metrics.total_installs += 1;

        tracing::debug!(agent_id = %agent_id, "install recorded");
    }

    /// Record an agent uninstall.
    pub fn record_uninstall(&mut self, agent_id: Uuid, user_id: Uuid) {
        let event = UninstallEvent {
            agent_id,
            user_id,
            timestamp: Utc::now(),
        };
        self.uninstalls.push(event);

        let metrics = self.agent_metrics.entry(agent_id).or_default();
        metrics.total_uninstalls += 1;

        tracing::debug!(agent_id = %agent_id, "uninstall recorded");
    }

    /// Record revenue for an agent.
    pub fn record_revenue(&mut self, agent_id: Uuid, amount_cents: u64) {
        let metrics = self.agent_metrics.entry(agent_id).or_default();
        metrics.revenue_cents += amount_cents;
    }

    /// Get metrics for a specific agent.
    pub fn agent_metrics(&self, agent_id: &Uuid) -> Option<&AgentMetrics> {
        self.agent_metrics.get(agent_id)
    }

    /// Compute per-domain metrics from install history.
    pub fn domain_metrics(&self) -> HashMap<LifeDomain, DomainMetrics> {
        let mut metrics: HashMap<LifeDomain, DomainMetrics> = HashMap::new();
        for event in &self.installs {
            let dm = metrics.entry(event.domain).or_default();
            dm.total_installs += 1;
        }
        metrics
    }

    /// Total installs across all agents.
    pub fn total_installs(&self) -> u64 {
        self.installs.len() as u64
    }

    /// Total revenue across all agents.
    pub fn total_revenue_cents(&self) -> u64 {
        self.agent_metrics.values().map(|m| m.revenue_cents).sum()
    }

    /// Top agents by install count.
    pub fn top_by_installs(&self, n: usize) -> Vec<(Uuid, u64)> {
        let mut agents: Vec<(Uuid, u64)> = self
            .agent_metrics
            .iter()
            .map(|(id, m)| (*id, m.total_installs))
            .collect();
        agents.sort_by(|a, b| b.1.cmp(&a.1));
        agents.truncate(n);
        agents
    }

    /// Top agents by revenue.
    pub fn top_by_revenue(&self, n: usize) -> Vec<(Uuid, u64)> {
        let mut agents: Vec<(Uuid, u64)> = self
            .agent_metrics
            .iter()
            .map(|(id, m)| (*id, m.revenue_cents))
            .collect();
        agents.sort_by(|a, b| b.1.cmp(&a.1));
        agents.truncate(n);
        agents
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_tracking() {
        let mut analytics = MarketplaceAnalytics::new();
        let agent_id = Uuid::new_v4();
        analytics.record_install(agent_id, Uuid::new_v4(), LifeDomain::Finance);
        analytics.record_install(agent_id, Uuid::new_v4(), LifeDomain::Finance);

        assert_eq!(analytics.total_installs(), 2);
        assert_eq!(analytics.agent_metrics(&agent_id).unwrap().total_installs, 2);
    }

    #[test]
    fn retention_rate() {
        let mut metrics = AgentMetrics::default();
        metrics.total_installs = 100;
        metrics.total_uninstalls = 20;
        assert!((metrics.retention_rate() - 80.0).abs() < 0.01);
    }

    #[test]
    fn retention_rate_zero_installs() {
        let metrics = AgentMetrics::default();
        assert_eq!(metrics.retention_rate(), 0.0);
    }

    #[test]
    fn revenue_tracking() {
        let mut analytics = MarketplaceAnalytics::new();
        let a1 = Uuid::new_v4();
        let a2 = Uuid::new_v4();
        analytics.record_revenue(a1, 1000);
        analytics.record_revenue(a2, 500);

        assert_eq!(analytics.total_revenue_cents(), 1500);
    }

    #[test]
    fn top_by_installs() {
        let mut analytics = MarketplaceAnalytics::new();
        let a1 = Uuid::new_v4();
        let a2 = Uuid::new_v4();

        for _ in 0..5 {
            analytics.record_install(a1, Uuid::new_v4(), LifeDomain::Health);
        }
        for _ in 0..10 {
            analytics.record_install(a2, Uuid::new_v4(), LifeDomain::Health);
        }

        let top = analytics.top_by_installs(1);
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].0, a2);
        assert_eq!(top[0].1, 10);
    }

    #[test]
    fn domain_metrics_aggregation() {
        let mut analytics = MarketplaceAnalytics::new();
        analytics.record_install(Uuid::new_v4(), Uuid::new_v4(), LifeDomain::Finance);
        analytics.record_install(Uuid::new_v4(), Uuid::new_v4(), LifeDomain::Finance);
        analytics.record_install(Uuid::new_v4(), Uuid::new_v4(), LifeDomain::Health);

        let dm = analytics.domain_metrics();
        assert_eq!(dm.get(&LifeDomain::Finance).unwrap().total_installs, 2);
        assert_eq!(dm.get(&LifeDomain::Health).unwrap().total_installs, 1);
    }
}
