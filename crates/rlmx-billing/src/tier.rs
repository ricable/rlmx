//! Subscription tiers, limits, and enforcement.

use serde::{Deserialize, Serialize};

/// The six subscription tiers offered by the platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubscriptionTier {
    /// Free tier: limited agents, no cloud, receive-only federation.
    Free,
    /// Personal ($9.99/mo): unlimited agents, 100K cloud tokens.
    Personal,
    /// Family ($19.99/mo): shared family agents, 200K tokens.
    Family,
    /// Pro ($29.99/mo): custom SDK, API access, 500K tokens.
    Pro,
    /// Enterprise (custom pricing): SOC2/HIPAA, unlimited tokens.
    Enterprise,
    /// Developer (free + rev share): marketplace publishing, 100K tokens.
    Developer,
}

impl SubscriptionTier {
    /// Monthly price in cents. Enterprise returns 0 (custom negotiated).
    pub fn monthly_price_cents(&self) -> u64 {
        match self {
            Self::Free => 0,
            Self::Personal => 999,
            Self::Family => 1999,
            Self::Pro => 2999,
            Self::Enterprise => 0, // custom pricing
            Self::Developer => 0,
        }
    }

    /// Returns the tier limits for this subscription tier.
    pub fn limits(&self) -> TierLimits {
        match self {
            Self::Free => TierLimits {
                max_agents: Some(5),
                cloud_tokens: 0,
                federation_mode: FederationMode::ReceiveOnly,
                features: TierFeatures {
                    custom_sdk: false,
                    api_access: false,
                    marketplace_publish: false,
                    family_sharing: false,
                    soc2_hipaa: false,
                    priority_support: false,
                },
            },
            Self::Personal => TierLimits {
                max_agents: None,
                cloud_tokens: 100_000,
                federation_mode: FederationMode::Full,
                features: TierFeatures {
                    custom_sdk: false,
                    api_access: false,
                    marketplace_publish: false,
                    family_sharing: false,
                    soc2_hipaa: false,
                    priority_support: false,
                },
            },
            Self::Family => TierLimits {
                max_agents: None,
                cloud_tokens: 200_000,
                federation_mode: FederationMode::Full,
                features: TierFeatures {
                    custom_sdk: false,
                    api_access: false,
                    marketplace_publish: false,
                    family_sharing: true,
                    soc2_hipaa: false,
                    priority_support: false,
                },
            },
            Self::Pro => TierLimits {
                max_agents: None,
                cloud_tokens: 500_000,
                federation_mode: FederationMode::Full,
                features: TierFeatures {
                    custom_sdk: true,
                    api_access: true,
                    marketplace_publish: false,
                    family_sharing: false,
                    soc2_hipaa: false,
                    priority_support: true,
                },
            },
            Self::Enterprise => TierLimits {
                max_agents: None,
                cloud_tokens: u64::MAX,
                federation_mode: FederationMode::Full,
                features: TierFeatures {
                    custom_sdk: true,
                    api_access: true,
                    marketplace_publish: true,
                    family_sharing: true,
                    soc2_hipaa: true,
                    priority_support: true,
                },
            },
            Self::Developer => TierLimits {
                max_agents: None,
                cloud_tokens: 100_000,
                federation_mode: FederationMode::Full,
                features: TierFeatures {
                    custom_sdk: true,
                    api_access: true,
                    marketplace_publish: true,
                    family_sharing: false,
                    soc2_hipaa: false,
                    priority_support: false,
                },
            },
        }
    }
}

/// Federation mode determines how voice/agent patterns are shared.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FederationMode {
    /// Can receive federated patterns but never contributes.
    ReceiveOnly,
    /// Full bidirectional federation participation.
    Full,
}

/// Feature flags available per tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TierFeatures {
    pub custom_sdk: bool,
    pub api_access: bool,
    pub marketplace_publish: bool,
    pub family_sharing: bool,
    pub soc2_hipaa: bool,
    pub priority_support: bool,
}

/// Resource limits for a subscription tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TierLimits {
    /// Maximum number of active agents. `None` means unlimited.
    pub max_agents: Option<u64>,
    /// Maximum cloud inference tokens per billing period.
    pub cloud_tokens: u64,
    /// Federation participation mode.
    pub federation_mode: FederationMode,
    /// Feature flags for this tier.
    pub features: TierFeatures,
}

impl TierLimits {
    /// Check whether the given agent count is within the tier limit.
    pub fn enforce_agent_limit(&self, active_agents: u64) -> Result<(), EnforcementViolation> {
        if let Some(max) = self.max_agents {
            if active_agents >= max {
                return Err(EnforcementViolation::AgentLimitReached {
                    current: active_agents,
                    limit: max,
                });
            }
        }
        Ok(())
    }

    /// Check whether cloud token usage is within limits.
    pub fn enforce_token_limit(&self, tokens_used: u64) -> Result<(), EnforcementViolation> {
        if tokens_used >= self.cloud_tokens {
            return Err(EnforcementViolation::CloudTokensExhausted {
                used: tokens_used,
                limit: self.cloud_tokens,
            });
        }
        Ok(())
    }

    /// Check whether a specific feature is available.
    pub fn enforce_feature(&self, feature: &str) -> Result<(), EnforcementViolation> {
        let available = match feature {
            "custom_sdk" => self.features.custom_sdk,
            "api_access" => self.features.api_access,
            "marketplace_publish" => self.features.marketplace_publish,
            "family_sharing" => self.features.family_sharing,
            "soc2_hipaa" => self.features.soc2_hipaa,
            "priority_support" => self.features.priority_support,
            _ => false,
        };
        if !available {
            return Err(EnforcementViolation::FeatureUnavailable(
                feature.to_string(),
            ));
        }
        Ok(())
    }
}

/// Violations detected during tier enforcement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnforcementViolation {
    AgentLimitReached { current: u64, limit: u64 },
    CloudTokensExhausted { used: u64, limit: u64 },
    FeatureUnavailable(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_tier_limits() {
        let limits = SubscriptionTier::Free.limits();
        assert_eq!(limits.max_agents, Some(5));
        assert_eq!(limits.cloud_tokens, 0);
        assert_eq!(limits.federation_mode, FederationMode::ReceiveOnly);
        assert!(!limits.features.custom_sdk);
        assert!(!limits.features.marketplace_publish);
    }

    #[test]
    fn personal_tier_unlimited_agents() {
        let limits = SubscriptionTier::Personal.limits();
        assert_eq!(limits.max_agents, None);
        assert_eq!(limits.cloud_tokens, 100_000);
        assert_eq!(limits.federation_mode, FederationMode::Full);
    }

    #[test]
    fn family_tier_sharing() {
        let limits = SubscriptionTier::Family.limits();
        assert!(limits.features.family_sharing);
        assert_eq!(limits.cloud_tokens, 200_000);
    }

    #[test]
    fn pro_tier_sdk_and_api() {
        let limits = SubscriptionTier::Pro.limits();
        assert!(limits.features.custom_sdk);
        assert!(limits.features.api_access);
        assert!(limits.features.priority_support);
        assert_eq!(limits.cloud_tokens, 500_000);
    }

    #[test]
    fn enterprise_tier_everything() {
        let limits = SubscriptionTier::Enterprise.limits();
        assert!(limits.features.soc2_hipaa);
        assert!(limits.features.custom_sdk);
        assert!(limits.features.api_access);
        assert!(limits.features.marketplace_publish);
        assert!(limits.features.family_sharing);
        assert!(limits.features.priority_support);
    }

    #[test]
    fn developer_tier_publishing() {
        let limits = SubscriptionTier::Developer.limits();
        assert!(limits.features.marketplace_publish);
        assert!(limits.features.api_access);
        assert_eq!(limits.cloud_tokens, 100_000);
    }

    #[test]
    fn enforce_agent_limit_free() {
        let limits = SubscriptionTier::Free.limits();
        assert!(limits.enforce_agent_limit(4).is_ok());
        assert!(limits.enforce_agent_limit(5).is_err());
        assert!(limits.enforce_agent_limit(6).is_err());
    }

    #[test]
    fn enforce_agent_limit_unlimited() {
        let limits = SubscriptionTier::Personal.limits();
        assert!(limits.enforce_agent_limit(1_000_000).is_ok());
    }

    #[test]
    fn enforce_token_limit() {
        let limits = SubscriptionTier::Personal.limits();
        assert!(limits.enforce_token_limit(99_999).is_ok());
        assert!(limits.enforce_token_limit(100_000).is_err());
    }

    #[test]
    fn enforce_feature_check() {
        let limits = SubscriptionTier::Free.limits();
        assert!(limits.enforce_feature("custom_sdk").is_err());
        assert!(limits.enforce_feature("api_access").is_err());

        let pro_limits = SubscriptionTier::Pro.limits();
        assert!(pro_limits.enforce_feature("custom_sdk").is_ok());
        assert!(pro_limits.enforce_feature("api_access").is_ok());
    }

    #[test]
    fn monthly_prices() {
        assert_eq!(SubscriptionTier::Free.monthly_price_cents(), 0);
        assert_eq!(SubscriptionTier::Personal.monthly_price_cents(), 999);
        assert_eq!(SubscriptionTier::Family.monthly_price_cents(), 1999);
        assert_eq!(SubscriptionTier::Pro.monthly_price_cents(), 2999);
        assert_eq!(SubscriptionTier::Enterprise.monthly_price_cents(), 0);
        assert_eq!(SubscriptionTier::Developer.monthly_price_cents(), 0);
    }
}
