//! # rlmx-billing
//!
//! Subscription billing bounded context (DDD-013, ADR-025).
//! Manages 6-tier subscriptions, family plans, developer revenue share,
//! usage tracking, and tier-based capability enforcement.

pub mod budget;
pub mod capability_enforcement;
pub mod developer;
pub mod error;
pub mod family;
pub mod subscription;
pub mod tier;
pub mod usage;

// Re-export primary types for convenient access.
pub use budget::{BudgetDecision, BudgetEntry, BudgetEvent, BudgetLedger, BudgetPolicy};
pub use capability_enforcement::{TierCapabilityEnforcer, TierCapabilityToken, TierCaveat};
pub use developer::{DeveloperAccount, PayoutMethod, PayoutRecord, SaleRecord};
pub use error::{BillingError, BillingResult};
pub use family::{FamilyGroup, FamilyMember, FamilyRole, PrivacyBoundary, MAX_FAMILY_MEMBERS};
pub use subscription::{Subscription, SubscriptionEvent, SubscriptionStatus};
pub use tier::{EnforcementViolation, FederationMode, SubscriptionTier, TierFeatures, TierLimits};
pub use usage::UsageMetrics;
