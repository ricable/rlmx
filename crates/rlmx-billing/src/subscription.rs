//! Subscription aggregate root: lifecycle management for user subscriptions.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::developer::DeveloperAccount;
use crate::error::{BillingError, BillingResult};
use crate::family::FamilyGroup;
use crate::tier::SubscriptionTier;
use crate::usage::UsageMetrics;

/// Grace period duration before a past-due subscription is suspended.
const GRACE_PERIOD_DAYS: i64 = 7;

/// Status of a subscription.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubscriptionStatus {
    /// Subscription is active and in good standing.
    Active,
    /// Payment failed; grace period active.
    PastDue,
    /// Grace period: subscription still works but payment overdue.
    GracePeriod,
    /// Suspended due to non-payment after grace period.
    Suspended,
    /// User-initiated cancellation.
    Cancelled,
}

/// Domain events emitted by the subscription aggregate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubscriptionEvent {
    Created {
        subscription_id: Uuid,
        user_id: Uuid,
        tier: SubscriptionTier,
    },
    Upgraded {
        subscription_id: Uuid,
        from: SubscriptionTier,
        to: SubscriptionTier,
    },
    Downgraded {
        subscription_id: Uuid,
        from: SubscriptionTier,
        to: SubscriptionTier,
    },
    Cancelled {
        subscription_id: Uuid,
    },
    Reactivated {
        subscription_id: Uuid,
        tier: SubscriptionTier,
    },
    StatusChanged {
        subscription_id: Uuid,
        from: SubscriptionStatus,
        to: SubscriptionStatus,
    },
}

/// The Subscription aggregate root — owns billing lifecycle for a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub plan: SubscriptionTier,
    pub status: SubscriptionStatus,
    pub usage: UsageMetrics,
    pub family_group: Option<FamilyGroup>,
    pub developer_account: Option<DeveloperAccount>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// When the grace period expires (only set during PastDue/GracePeriod).
    pub grace_period_end: Option<DateTime<Utc>>,
}

impl Subscription {
    /// Create a new subscription for a user on the given tier.
    pub fn subscribe(user_id: Uuid, tier: SubscriptionTier) -> (Self, SubscriptionEvent) {
        let now = Utc::now();
        let period_end = now + Duration::days(30);
        let limits = tier.limits();

        let family_group = if tier == SubscriptionTier::Family {
            Some(FamilyGroup::new(user_id))
        } else {
            None
        };

        let developer_account = if tier == SubscriptionTier::Developer {
            Some(DeveloperAccount::new(format!("dev-{}", user_id)))
        } else {
            None
        };

        let sub = Self {
            id: Uuid::new_v4(),
            user_id,
            plan: tier,
            status: SubscriptionStatus::Active,
            usage: UsageMetrics::new(&limits, now, period_end),
            family_group,
            developer_account,
            created_at: now,
            updated_at: now,
            grace_period_end: None,
        };

        let event = SubscriptionEvent::Created {
            subscription_id: sub.id,
            user_id,
            tier,
        };

        tracing::info!(
            subscription_id = %sub.id,
            user_id = %user_id,
            tier = ?tier,
            "subscription created"
        );

        (sub, event)
    }

    /// Upgrade to a higher tier.
    pub fn upgrade(&mut self, new_tier: SubscriptionTier) -> BillingResult<SubscriptionEvent> {
        self.require_active()?;

        // Developer and Enterprise tiers have custom/zero pricing and are always
        // valid upgrade targets regardless of price comparison.
        let is_special_target =
            new_tier == SubscriptionTier::Enterprise || new_tier == SubscriptionTier::Developer;

        if !is_special_target && new_tier.monthly_price_cents() <= self.plan.monthly_price_cents() {
            return Err(BillingError::InvalidTierTransition {
                from: self.plan,
                to: new_tier,
            });
        }

        let old_tier = self.plan;
        self.plan = new_tier;
        self.usage.update_limits(&new_tier.limits());
        self.updated_at = Utc::now();

        // Create family group if upgrading to Family tier.
        if new_tier == SubscriptionTier::Family && self.family_group.is_none() {
            self.family_group = Some(FamilyGroup::new(self.user_id));
        }

        // Create developer account if upgrading to Developer tier.
        if new_tier == SubscriptionTier::Developer && self.developer_account.is_none() {
            self.developer_account = Some(DeveloperAccount::new(format!("dev-{}", self.user_id)));
        }

        let event = SubscriptionEvent::Upgraded {
            subscription_id: self.id,
            from: old_tier,
            to: new_tier,
        };

        tracing::info!(
            subscription_id = %self.id,
            from = ?old_tier,
            to = ?new_tier,
            "subscription upgraded"
        );

        Ok(event)
    }

    /// Downgrade to a lower tier.
    pub fn downgrade(&mut self, new_tier: SubscriptionTier) -> BillingResult<SubscriptionEvent> {
        self.require_active()?;

        if new_tier.monthly_price_cents() >= self.plan.monthly_price_cents()
            && self.plan != SubscriptionTier::Enterprise
        {
            return Err(BillingError::InvalidTierTransition {
                from: self.plan,
                to: new_tier,
            });
        }

        let old_tier = self.plan;
        self.plan = new_tier;
        self.usage.update_limits(&new_tier.limits());
        self.updated_at = Utc::now();

        // Remove family group if downgrading away from Family tier.
        if new_tier != SubscriptionTier::Family {
            self.family_group = None;
        }

        let event = SubscriptionEvent::Downgraded {
            subscription_id: self.id,
            from: old_tier,
            to: new_tier,
        };

        tracing::info!(
            subscription_id = %self.id,
            from = ?old_tier,
            to = ?new_tier,
            "subscription downgraded"
        );

        Ok(event)
    }

    /// Cancel the subscription.
    pub fn cancel(&mut self) -> BillingResult<SubscriptionEvent> {
        if self.status == SubscriptionStatus::Cancelled {
            return Err(BillingError::AlreadyCancelled);
        }

        self.status = SubscriptionStatus::Cancelled;
        self.updated_at = Utc::now();

        let event = SubscriptionEvent::Cancelled {
            subscription_id: self.id,
        };

        tracing::info!(subscription_id = %self.id, "subscription cancelled");

        Ok(event)
    }

    /// Reactivate a cancelled or suspended subscription.
    pub fn reactivate(&mut self, tier: SubscriptionTier) -> BillingResult<SubscriptionEvent> {
        if self.status != SubscriptionStatus::Cancelled
            && self.status != SubscriptionStatus::Suspended
        {
            return Err(BillingError::NotActive(self.status));
        }

        self.plan = tier;
        self.status = SubscriptionStatus::Active;
        self.grace_period_end = None;
        self.usage.update_limits(&tier.limits());

        let now = Utc::now();
        self.usage.reset_period(now, now + Duration::days(30));
        self.updated_at = now;

        let event = SubscriptionEvent::Reactivated {
            subscription_id: self.id,
            tier,
        };

        tracing::info!(subscription_id = %self.id, tier = ?tier, "subscription reactivated");

        Ok(event)
    }

    /// Mark the subscription as past-due with a grace period.
    pub fn mark_past_due(&mut self) -> SubscriptionEvent {
        let old_status = self.status;
        self.status = SubscriptionStatus::GracePeriod;
        self.grace_period_end = Some(Utc::now() + Duration::days(GRACE_PERIOD_DAYS));
        self.updated_at = Utc::now();

        tracing::warn!(
            subscription_id = %self.id,
            grace_end = ?self.grace_period_end,
            "subscription marked past due"
        );

        SubscriptionEvent::StatusChanged {
            subscription_id: self.id,
            from: old_status,
            to: SubscriptionStatus::GracePeriod,
        }
    }

    /// Suspend the subscription (after grace period expires).
    pub fn suspend(&mut self) -> SubscriptionEvent {
        let old_status = self.status;
        self.status = SubscriptionStatus::Suspended;
        self.grace_period_end = None;
        self.updated_at = Utc::now();

        tracing::warn!(subscription_id = %self.id, "subscription suspended");

        SubscriptionEvent::StatusChanged {
            subscription_id: self.id,
            from: old_status,
            to: SubscriptionStatus::Suspended,
        }
    }

    /// Check whether the subscription is in an active (usable) state.
    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            SubscriptionStatus::Active | SubscriptionStatus::GracePeriod
        )
    }

    /// Internal helper to require the subscription be active for mutations.
    fn require_active(&self) -> BillingResult<()> {
        if !self.is_active() {
            return Err(BillingError::NotActive(self.status));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscribe_creates_active_subscription() {
        let (sub, event) = Subscription::subscribe(Uuid::new_v4(), SubscriptionTier::Personal);
        assert_eq!(sub.status, SubscriptionStatus::Active);
        assert_eq!(sub.plan, SubscriptionTier::Personal);
        assert!(sub.family_group.is_none());
        assert!(sub.developer_account.is_none());
        assert!(matches!(event, SubscriptionEvent::Created { .. }));
    }

    #[test]
    fn family_tier_creates_family_group() {
        let user = Uuid::new_v4();
        let (sub, _) = Subscription::subscribe(user, SubscriptionTier::Family);
        assert!(sub.family_group.is_some());
        assert_eq!(sub.family_group.unwrap().owner, user);
    }

    #[test]
    fn developer_tier_creates_dev_account() {
        let (sub, _) = Subscription::subscribe(Uuid::new_v4(), SubscriptionTier::Developer);
        assert!(sub.developer_account.is_some());
    }

    #[test]
    fn upgrade_tier() {
        let (mut sub, _) = Subscription::subscribe(Uuid::new_v4(), SubscriptionTier::Personal);
        let event = sub.upgrade(SubscriptionTier::Pro).unwrap();
        assert_eq!(sub.plan, SubscriptionTier::Pro);
        assert!(matches!(event, SubscriptionEvent::Upgraded { .. }));
    }

    #[test]
    fn upgrade_to_family_creates_group() {
        let (mut sub, _) = Subscription::subscribe(Uuid::new_v4(), SubscriptionTier::Personal);
        sub.upgrade(SubscriptionTier::Family).unwrap();
        assert!(sub.family_group.is_some());
    }

    #[test]
    fn upgrade_to_developer_creates_account() {
        let (mut sub, _) = Subscription::subscribe(Uuid::new_v4(), SubscriptionTier::Personal);
        sub.upgrade(SubscriptionTier::Developer).unwrap();
        assert!(sub.developer_account.is_some());
    }

    #[test]
    fn cannot_downgrade_as_upgrade() {
        let (mut sub, _) = Subscription::subscribe(Uuid::new_v4(), SubscriptionTier::Pro);
        let result = sub.upgrade(SubscriptionTier::Personal);
        assert!(result.is_err());
    }

    #[test]
    fn downgrade_tier() {
        let (mut sub, _) = Subscription::subscribe(Uuid::new_v4(), SubscriptionTier::Pro);
        let event = sub.downgrade(SubscriptionTier::Personal).unwrap();
        assert_eq!(sub.plan, SubscriptionTier::Personal);
        assert!(matches!(event, SubscriptionEvent::Downgraded { .. }));
    }

    #[test]
    fn downgrade_removes_family_group() {
        let (mut sub, _) = Subscription::subscribe(Uuid::new_v4(), SubscriptionTier::Family);
        assert!(sub.family_group.is_some());
        sub.downgrade(SubscriptionTier::Personal).unwrap();
        assert!(sub.family_group.is_none());
    }

    #[test]
    fn cancel_and_reactivate() {
        let (mut sub, _) = Subscription::subscribe(Uuid::new_v4(), SubscriptionTier::Personal);
        sub.cancel().unwrap();
        assert_eq!(sub.status, SubscriptionStatus::Cancelled);

        sub.reactivate(SubscriptionTier::Pro).unwrap();
        assert_eq!(sub.status, SubscriptionStatus::Active);
        assert_eq!(sub.plan, SubscriptionTier::Pro);
    }

    #[test]
    fn double_cancel_fails() {
        let (mut sub, _) = Subscription::subscribe(Uuid::new_v4(), SubscriptionTier::Personal);
        sub.cancel().unwrap();
        assert!(sub.cancel().is_err());
    }

    #[test]
    fn reactivate_requires_cancelled_or_suspended() {
        let (mut sub, _) = Subscription::subscribe(Uuid::new_v4(), SubscriptionTier::Personal);
        assert!(sub.reactivate(SubscriptionTier::Personal).is_err());
    }

    #[test]
    fn grace_period_flow() {
        let (mut sub, _) = Subscription::subscribe(Uuid::new_v4(), SubscriptionTier::Personal);
        sub.mark_past_due();
        assert_eq!(sub.status, SubscriptionStatus::GracePeriod);
        assert!(sub.grace_period_end.is_some());
        assert!(sub.is_active());

        sub.suspend();
        assert_eq!(sub.status, SubscriptionStatus::Suspended);
        assert!(!sub.is_active());
    }

    #[test]
    fn reactivate_from_suspended() {
        let (mut sub, _) = Subscription::subscribe(Uuid::new_v4(), SubscriptionTier::Personal);
        sub.mark_past_due();
        sub.suspend();
        assert!(!sub.is_active());

        sub.reactivate(SubscriptionTier::Personal).unwrap();
        assert!(sub.is_active());
    }

    #[test]
    fn usage_limits_match_tier() {
        let (sub, _) = Subscription::subscribe(Uuid::new_v4(), SubscriptionTier::Free);
        assert_eq!(sub.usage.agents_limit, Some(5));
        assert_eq!(sub.usage.cloud_tokens_limit, 0);
    }

    #[test]
    fn upgrade_updates_usage_limits() {
        let (mut sub, _) = Subscription::subscribe(Uuid::new_v4(), SubscriptionTier::Free);
        assert_eq!(sub.usage.cloud_tokens_limit, 0);
        sub.upgrade(SubscriptionTier::Personal).unwrap();
        assert_eq!(sub.usage.cloud_tokens_limit, 100_000);
        assert_eq!(sub.usage.agents_limit, None);
    }
}
