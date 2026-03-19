//! Tier-based capability token enforcement.
//!
//! Derives capability tokens with tier-specific caveats, enforcing spawn-time
//! limits based on the user's subscription tier.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use rlmx_kernel::SyscallPermission;

use crate::error::{BillingError, BillingResult};
use crate::tier::SubscriptionTier;
use crate::usage::UsageMetrics;

/// A caveat attached to a capability token restricting its use based on tier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TierCaveat {
    /// Maximum number of agents this token can spawn.
    AgentLimit(u64),
    /// Whether cloud burst inference is allowed.
    CloudBurstAllowed(bool),
    /// Whether federation participation is allowed.
    FederationAllowed(bool),
    /// Whether marketplace publishing is allowed.
    MarketplacePublishAllowed(bool),
    /// Whether API access is allowed.
    ApiAccessAllowed(bool),
    /// Per-agent budget limit in microcents (ADR-032).
    BudgetLimit(u64),
}

/// A set of caveats derived from a subscription tier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierCapabilityToken {
    /// The token identifier.
    pub token_id: Uuid,
    /// The subscription tier this token was derived from.
    pub tier: SubscriptionTier,
    /// Syscall permissions granted by this token.
    pub permissions: Vec<SyscallPermission>,
    /// Tier-specific caveats restricting token use.
    pub caveats: Vec<TierCaveat>,
}

/// Enforces tier-based capabilities at spawn time and syscall invocation.
pub struct TierCapabilityEnforcer;

impl TierCapabilityEnforcer {
    /// Derive a capability token with tier-specific caveats.
    pub fn derive_token(tier: SubscriptionTier) -> TierCapabilityToken {
        let limits = tier.limits();

        let mut permissions = vec![
            SyscallPermission::VecInsert,
            SyscallPermission::VecSearch,
            SyscallPermission::VecDelete,
            SyscallPermission::GraphQuery,
            SyscallPermission::ProcessFork,
            SyscallPermission::ProcessSend,
            SyscallPermission::ProcessRecv,
            SyscallPermission::VoiceSynthesize,
            SyscallPermission::IntentRoute,
        ];

        // Higher tiers get additional permissions.
        match tier {
            SubscriptionTier::Pro | SubscriptionTier::Enterprise | SubscriptionTier::Developer => {
                permissions.push(SyscallPermission::GraphCut);
                permissions.push(SyscallPermission::GraphDiffuse);
                permissions.push(SyscallPermission::StateMutate);
                permissions.push(SyscallPermission::AttentionSelect);
            }
            _ => {}
        }

        let mut caveats = Vec::new();

        // Agent limit caveat
        if let Some(max) = limits.max_agents {
            caveats.push(TierCaveat::AgentLimit(max));
        }

        // Cloud burst caveat: free tier has 0 tokens = no cloud
        caveats.push(TierCaveat::CloudBurstAllowed(limits.cloud_tokens > 0));

        // Federation caveat
        caveats.push(TierCaveat::FederationAllowed(
            limits.federation_mode == crate::tier::FederationMode::Full,
        ));

        // Marketplace publish caveat
        caveats.push(TierCaveat::MarketplacePublishAllowed(
            limits.features.marketplace_publish,
        ));

        // API access caveat
        caveats.push(TierCaveat::ApiAccessAllowed(limits.features.api_access));

        TierCapabilityToken {
            token_id: Uuid::new_v4(),
            tier,
            permissions,
            caveats,
        }
    }

    /// Enforce spawn-time limits: checks whether a new agent can be spawned
    /// given current usage and tier limits.
    pub fn enforce_spawn(usage: &UsageMetrics) -> BillingResult<()> {
        usage.check_agent_quota().map_err(|e| {
            tracing::warn!(error = %e, "agent spawn denied by tier enforcement");
            e
        })
    }

    /// Enforce cloud token consumption.
    pub fn enforce_cloud_usage(usage: &mut UsageMetrics, tokens: u64) -> BillingResult<()> {
        usage.record_token_usage(tokens).map_err(|e| {
            tracing::warn!(error = %e, "cloud token usage denied by tier enforcement");
            e
        })
    }

    /// Check whether a feature is allowed by the tier.
    pub fn enforce_feature(tier: SubscriptionTier, feature: &str) -> BillingResult<()> {
        tier.limits()
            .enforce_feature(feature)
            .map_err(|_| BillingError::FeatureNotAvailable {
                tier,
                feature: feature.to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_tier_token_has_agent_limit() {
        let token = TierCapabilityEnforcer::derive_token(SubscriptionTier::Free);
        assert!(token.caveats.contains(&TierCaveat::AgentLimit(5)));
        assert!(token
            .caveats
            .contains(&TierCaveat::CloudBurstAllowed(false)));
        assert!(token
            .caveats
            .contains(&TierCaveat::FederationAllowed(false)));
    }

    #[test]
    fn personal_tier_token_no_agent_limit() {
        let token = TierCapabilityEnforcer::derive_token(SubscriptionTier::Personal);
        assert!(!token
            .caveats
            .iter()
            .any(|c| matches!(c, TierCaveat::AgentLimit(_))));
        assert!(token.caveats.contains(&TierCaveat::CloudBurstAllowed(true)));
        assert!(token.caveats.contains(&TierCaveat::FederationAllowed(true)));
    }

    #[test]
    fn developer_tier_has_marketplace_publish() {
        let token = TierCapabilityEnforcer::derive_token(SubscriptionTier::Developer);
        assert!(token
            .caveats
            .contains(&TierCaveat::MarketplacePublishAllowed(true)));
        assert!(token.caveats.contains(&TierCaveat::ApiAccessAllowed(true)));
    }

    #[test]
    fn pro_tier_has_advanced_permissions() {
        let token = TierCapabilityEnforcer::derive_token(SubscriptionTier::Pro);
        assert!(token.permissions.contains(&SyscallPermission::GraphCut));
        assert!(token.permissions.contains(&SyscallPermission::StateMutate));
    }

    #[test]
    fn free_tier_lacks_advanced_permissions() {
        let token = TierCapabilityEnforcer::derive_token(SubscriptionTier::Free);
        assert!(!token.permissions.contains(&SyscallPermission::GraphCut));
        assert!(!token.permissions.contains(&SyscallPermission::StateMutate));
    }

    #[test]
    fn enforce_feature_free_tier() {
        assert!(
            TierCapabilityEnforcer::enforce_feature(SubscriptionTier::Free, "custom_sdk").is_err()
        );
        assert!(
            TierCapabilityEnforcer::enforce_feature(SubscriptionTier::Free, "api_access").is_err()
        );
    }

    #[test]
    fn enforce_feature_pro_tier() {
        assert!(
            TierCapabilityEnforcer::enforce_feature(SubscriptionTier::Pro, "custom_sdk").is_ok()
        );
        assert!(
            TierCapabilityEnforcer::enforce_feature(SubscriptionTier::Pro, "api_access").is_ok()
        );
    }
}
