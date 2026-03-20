//! Billing error types.

use uuid::Uuid;

use crate::subscription::SubscriptionStatus;
use crate::tier::SubscriptionTier;

/// Errors produced by the billing bounded context.
#[derive(Debug, thiserror::Error)]
pub enum BillingError {
    #[error("subscription not found: {0}")]
    SubscriptionNotFound(Uuid),

    #[error("invalid tier transition from {from:?} to {to:?}")]
    InvalidTierTransition {
        from: SubscriptionTier,
        to: SubscriptionTier,
    },

    #[error("subscription not active: current status is {0:?}")]
    NotActive(SubscriptionStatus),

    #[error("usage quota exceeded: {resource} ({used}/{limit})")]
    QuotaExceeded {
        resource: String,
        used: u64,
        limit: u64,
    },

    #[error("family group not found: {0}")]
    FamilyGroupNotFound(Uuid),

    #[error("family group full: max {max} members")]
    FamilyGroupFull { max: usize },

    #[error("member not found: {0}")]
    MemberNotFound(Uuid),

    #[error("member already in family group: {0}")]
    MemberAlreadyExists(Uuid),

    #[error("cannot remove family owner")]
    CannotRemoveOwner,

    #[error("developer account not found: {0}")]
    DeveloperAccountNotFound(Uuid),

    #[error("payout threshold not met: balance {balance_cents} < threshold {threshold_cents}")]
    PayoutThresholdNotMet {
        balance_cents: u64,
        threshold_cents: u64,
    },

    #[error("no payout method configured")]
    NoPayoutMethod,

    #[error("feature not available on {tier:?} tier: {feature}")]
    FeatureNotAvailable {
        tier: SubscriptionTier,
        feature: String,
    },

    #[error("already cancelled")]
    AlreadyCancelled,

    #[error("internal error: {0}")]
    Internal(String),
}

pub type BillingResult<T> = Result<T, BillingError>;
