/**
 * Tier enforcement helpers.
 * Re-exports shared tier types and adds enforcement violation handling.
 *
 * The canonical SubscriptionTier enum, TierLimits, TierFeatures, monthlyPriceCents,
 * and tierLimits are defined in @aix/shared. This module provides the enforcement
 * violation types and convenience helpers that wrap them.
 */

// Re-export everything from @aix/shared so consumers can import from @aix/billing
export {
  SubscriptionTier,
  FederationMode,
  monthlyPriceCents,
  tierLimits,
} from '@aix/shared';
export type { TierLimits, TierFeatures } from '@aix/shared';

import type { TierLimits } from '@aix/shared';

/** Violations detected during tier enforcement. */
export type EnforcementViolation =
  | { type: 'AgentLimitReached'; current: number; limit: number }
  | { type: 'CloudTokensExhausted'; used: number; limit: number }
  | { type: 'FeatureUnavailable'; feature: string };

/**
 * Check whether the given agent count is within the tier limit.
 * Returns null on success, or an EnforcementViolation on failure.
 */
export function enforceAgentLimit(
  limits: TierLimits,
  activeAgents: number,
): EnforcementViolation | null {
  if (limits.maxAgents !== null && activeAgents >= limits.maxAgents) {
    return {
      type: 'AgentLimitReached',
      current: activeAgents,
      limit: limits.maxAgents,
    };
  }
  return null;
}

/**
 * Check whether cloud token usage is within limits.
 */
export function enforceTokenLimit(
  limits: TierLimits,
  tokensUsed: number,
): EnforcementViolation | null {
  if (tokensUsed >= limits.cloudTokens) {
    return {
      type: 'CloudTokensExhausted',
      used: tokensUsed,
      limit: limits.cloudTokens,
    };
  }
  return null;
}

/** Known feature flag names for tier enforcement. */
export type TierFeatureName =
  | 'custom_sdk'
  | 'api_access'
  | 'marketplace_publish'
  | 'family_sharing'
  | 'soc2_hipaa'
  | 'priority_support';

const FEATURE_MAP: Record<TierFeatureName, keyof import('@aix/shared').TierFeatures> = {
  custom_sdk: 'customSdk',
  api_access: 'apiAccess',
  marketplace_publish: 'marketplacePublish',
  family_sharing: 'familySharing',
  soc2_hipaa: 'soc2Hipaa',
  priority_support: 'prioritySupport',
};

/**
 * Check whether a specific feature is available on the tier.
 */
export function enforceFeature(
  limits: TierLimits,
  feature: string,
): EnforcementViolation | null {
  const key = FEATURE_MAP[feature as TierFeatureName];
  const available = key ? limits.features[key] : false;
  if (!available) {
    return { type: 'FeatureUnavailable', feature };
  }
  return null;
}
