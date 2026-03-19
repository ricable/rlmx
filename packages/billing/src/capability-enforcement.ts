/**
 * Tier-based capability token enforcement.
 * Maps to rlmx-billing/src/capability_enforcement.rs.
 *
 * Derives capability tokens with tier-specific caveats, enforcing spawn-time
 * limits based on the user's subscription tier.
 */

import { SyscallPermission, type SubscriptionTier, FederationMode } from '@aix/shared';
import { tierLimits } from './tier.js';
import { featureNotAvailable, type BillingError } from './errors.js';
import { checkAgentQuota, recordTokenUsage, type UsageMetrics } from './usage.js';

/** A caveat attached to a capability token restricting its use based on tier. */
export type TierCaveat =
  | { type: 'AgentLimit'; limit: number }
  | { type: 'CloudBurstAllowed'; allowed: boolean }
  | { type: 'FederationAllowed'; allowed: boolean }
  | { type: 'MarketplacePublishAllowed'; allowed: boolean }
  | { type: 'ApiAccessAllowed'; allowed: boolean };

/** A set of caveats derived from a subscription tier. */
export interface TierCapabilityToken {
  /** The token identifier. */
  tokenId: string;
  /** The subscription tier this token was derived from. */
  tier: SubscriptionTier;
  /** Syscall permissions granted by this token. */
  permissions: SyscallPermission[];
  /** Tier-specific caveats restricting token use. */
  caveats: TierCaveat[];
}

let _nextTokenId = 0;
function generateTokenId(): string {
  return `tok-${Date.now()}-${++_nextTokenId}-${Math.random().toString(36).slice(2, 8)}`;
}

/**
 * TierCapabilityEnforcer: derives tokens and enforces tier-based capabilities.
 */
export const TierCapabilityEnforcer = {
  /**
   * Derive a capability token with tier-specific caveats.
   */
  deriveToken(tier: SubscriptionTier): TierCapabilityToken {
    const limits = tierLimits(tier);

    const permissions: SyscallPermission[] = [
      SyscallPermission.VecInsert,
      SyscallPermission.VecSearch,
      SyscallPermission.VecDelete,
      SyscallPermission.GraphQuery,
      SyscallPermission.ProcessFork,
      SyscallPermission.ProcessSend,
      SyscallPermission.ProcessRecv,
      SyscallPermission.VoiceSynthesize,
      SyscallPermission.IntentRoute,
    ];

    // Higher tiers get additional permissions
    if (tier === 'Pro' || tier === 'Enterprise' || tier === 'Developer') {
      permissions.push(
        SyscallPermission.GraphCut,
        SyscallPermission.GraphDiffuse,
        SyscallPermission.StateMutate,
        SyscallPermission.AttentionSelect,
      );
    }

    const caveats: TierCaveat[] = [];

    // Agent limit caveat
    if (limits.maxAgents !== null) {
      caveats.push({ type: 'AgentLimit', limit: limits.maxAgents });
    }

    // Cloud burst caveat: free tier has 0 tokens = no cloud
    caveats.push({
      type: 'CloudBurstAllowed',
      allowed: limits.cloudTokens > 0,
    });

    // Federation caveat
    caveats.push({
      type: 'FederationAllowed',
      allowed: limits.federationMode === FederationMode.Full,
    });

    // Marketplace publish caveat
    caveats.push({
      type: 'MarketplacePublishAllowed',
      allowed: limits.features.marketplacePublish,
    });

    // API access caveat
    caveats.push({
      type: 'ApiAccessAllowed',
      allowed: limits.features.apiAccess,
    });

    return {
      tokenId: generateTokenId(),
      tier,
      permissions,
      caveats,
    };
  },

  /**
   * Enforce spawn-time limits: checks whether a new agent can be spawned
   * given current usage and tier limits.
   * Throws BillingError if quota exceeded.
   */
  enforceSpawn(usage: UsageMetrics): void {
    checkAgentQuota(usage);
  },

  /**
   * Enforce cloud token consumption.
   * Returns updated UsageMetrics. Throws BillingError if quota exceeded.
   */
  enforceCloudUsage(usage: UsageMetrics, tokens: number): UsageMetrics {
    return recordTokenUsage(usage, tokens);
  },

  /**
   * Check whether a feature is allowed by the tier.
   * Throws BillingError if not available.
   */
  enforceFeature(tier: SubscriptionTier, feature: string): void {
    const limits = tierLimits(tier);
    const featureMap: Record<string, boolean> = {
      custom_sdk: limits.features.customSdk,
      api_access: limits.features.apiAccess,
      marketplace_publish: limits.features.marketplacePublish,
      family_sharing: limits.features.familySharing,
      soc2_hipaa: limits.features.soc2Hipaa,
      priority_support: limits.features.prioritySupport,
    };
    const available = featureMap[feature] ?? false;
    if (!available) {
      throw featureNotAvailable(tier, feature);
    }
  },
} as const;
