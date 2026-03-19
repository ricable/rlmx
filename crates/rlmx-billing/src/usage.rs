//! Usage tracking: metering cloud tokens and agent counts per billing period.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{BillingError, BillingResult};
use crate::tier::TierLimits;

/// Tracked usage metrics for a billing period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageMetrics {
    /// Cloud inference tokens consumed this period.
    pub cloud_tokens_used: u64,
    /// Cloud token limit for the current tier.
    pub cloud_tokens_limit: u64,
    /// Currently active agent count.
    pub agents_active: u64,
    /// Agent limit for the current tier (`None` = unlimited).
    pub agents_limit: Option<u64>,
    /// Start of the current billing period.
    pub period_start: DateTime<Utc>,
    /// End of the current billing period.
    pub period_end: DateTime<Utc>,
}

impl UsageMetrics {
    /// Create a new usage tracker for the given tier limits and period.
    pub fn new(
        limits: &TierLimits,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Self {
        Self {
            cloud_tokens_used: 0,
            cloud_tokens_limit: limits.cloud_tokens,
            agents_active: 0,
            agents_limit: limits.max_agents,
            period_start,
            period_end,
        }
    }

    /// Record cloud token usage. Returns error if quota would be exceeded.
    pub fn record_token_usage(&mut self, tokens: u64) -> BillingResult<()> {
        let new_total = self.cloud_tokens_used + tokens;
        if new_total > self.cloud_tokens_limit {
            return Err(BillingError::QuotaExceeded {
                resource: "cloud_tokens".to_string(),
                used: new_total,
                limit: self.cloud_tokens_limit,
            });
        }
        self.cloud_tokens_used = new_total;
        tracing::debug!(
            tokens_used = self.cloud_tokens_used,
            limit = self.cloud_tokens_limit,
            "token usage recorded"
        );
        Ok(())
    }

    /// Check whether a new agent can be spawned within the current quota.
    pub fn check_agent_quota(&self) -> BillingResult<()> {
        if let Some(limit) = self.agents_limit {
            if self.agents_active >= limit {
                return Err(BillingError::QuotaExceeded {
                    resource: "agents".to_string(),
                    used: self.agents_active,
                    limit,
                });
            }
        }
        Ok(())
    }

    /// Increment the active agent count (after spawn-time check).
    pub fn record_agent_spawn(&mut self) {
        self.agents_active += 1;
    }

    /// Decrement the active agent count when an agent is terminated.
    pub fn record_agent_terminate(&mut self) {
        self.agents_active = self.agents_active.saturating_sub(1);
    }

    /// Reset usage counters for a new billing period.
    pub fn reset_period(&mut self, new_start: DateTime<Utc>, new_end: DateTime<Utc>) {
        self.cloud_tokens_used = 0;
        self.period_start = new_start;
        self.period_end = new_end;
        tracing::info!(period_start = %new_start, period_end = %new_end, "billing period reset");
    }

    /// Update limits when the subscription tier changes.
    pub fn update_limits(&mut self, limits: &TierLimits) {
        self.cloud_tokens_limit = limits.cloud_tokens;
        self.agents_limit = limits.max_agents;
    }

    /// Percentage of cloud tokens consumed (0.0 - 1.0).
    pub fn token_utilization(&self) -> f64 {
        if self.cloud_tokens_limit == 0 {
            return if self.cloud_tokens_used > 0 { 1.0 } else { 0.0 };
        }
        self.cloud_tokens_used as f64 / self.cloud_tokens_limit as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tier::SubscriptionTier;
    use chrono::Duration;

    fn make_metrics(tier: SubscriptionTier) -> UsageMetrics {
        let now = Utc::now();
        UsageMetrics::new(&tier.limits(), now, now + Duration::days(30))
    }

    #[test]
    fn record_token_usage_within_limit() {
        let mut m = make_metrics(SubscriptionTier::Personal);
        assert!(m.record_token_usage(50_000).is_ok());
        assert_eq!(m.cloud_tokens_used, 50_000);
        assert!(m.record_token_usage(50_000).is_ok());
        assert_eq!(m.cloud_tokens_used, 100_000);
    }

    #[test]
    fn record_token_usage_exceeds_limit() {
        let mut m = make_metrics(SubscriptionTier::Personal);
        assert!(m.record_token_usage(100_001).is_err());
    }

    #[test]
    fn agent_quota_free_tier() {
        let mut m = make_metrics(SubscriptionTier::Free);
        for _ in 0..5 {
            assert!(m.check_agent_quota().is_ok());
            m.record_agent_spawn();
        }
        // 6th should fail
        assert!(m.check_agent_quota().is_err());
    }

    #[test]
    fn agent_quota_unlimited() {
        let mut m = make_metrics(SubscriptionTier::Personal);
        for _ in 0..100 {
            assert!(m.check_agent_quota().is_ok());
            m.record_agent_spawn();
        }
    }

    #[test]
    fn reset_period_clears_tokens() {
        let mut m = make_metrics(SubscriptionTier::Personal);
        m.record_token_usage(50_000).unwrap();
        let now = Utc::now();
        m.reset_period(now, now + Duration::days(30));
        assert_eq!(m.cloud_tokens_used, 0);
    }

    #[test]
    fn token_utilization_calculation() {
        let mut m = make_metrics(SubscriptionTier::Personal);
        assert!((m.token_utilization() - 0.0).abs() < f64::EPSILON);
        m.record_token_usage(50_000).unwrap();
        assert!((m.token_utilization() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn agent_terminate_decrements() {
        let mut m = make_metrics(SubscriptionTier::Free);
        m.record_agent_spawn();
        m.record_agent_spawn();
        assert_eq!(m.agents_active, 2);
        m.record_agent_terminate();
        assert_eq!(m.agents_active, 1);
    }

    #[test]
    fn agent_terminate_saturates_at_zero() {
        let mut m = make_metrics(SubscriptionTier::Free);
        m.record_agent_terminate();
        assert_eq!(m.agents_active, 0);
    }
}
