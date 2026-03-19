import {
  type LifeDomain,
  FederationMode,
  type SubscriptionTier,
} from './enums.js';

// ---------------------------------------------------------------------------
// Voice / Intent types
// ---------------------------------------------------------------------------

/**
 * A single parsed intent extracted from a voice transcript.
 * Maps to rlmx-kernel Intent struct.
 */
export interface Intent {
  /** The life domain this intent belongs to. */
  domain: LifeDomain;
  /** Action verb or short description (e.g. "schedule", "pay", "lookup"). */
  action: string;
  /** Named entities extracted from the transcript. */
  entities: string[];
  /** Urgency score in [0.0, 1.0] -- higher means more time-sensitive. */
  urgency: number;
  /** Confidence that this intent was correctly parsed, in [0.0, 1.0]. */
  confidence: number;
}

// ---------------------------------------------------------------------------
// Strategy types
// ---------------------------------------------------------------------------

/** How to aggregate results from multi-zone scatter queries. */
export type GatherStrategy =
  | { type: 'First' }
  | { type: 'All' }
  | { type: 'Quorum' }
  | { type: 'Custom'; name: string };

/** Scheduling strategy selection. Maps to rlmx-kernel Strategy enum. */
export type Strategy =
  | { type: 'Rlm' }
  | { type: 'Trm'; model: string }
  | { type: 'Auto' }
  | { type: 'Hybrid'; triage: string; threshold: number }
  | { type: 'Edge' }
  | {
      type: 'Swarm';
      scatterZones: string[];
      gatherStrategy: GatherStrategy;
      timeoutMs: number;
    };

// ---------------------------------------------------------------------------
// Billing / Tier types
// ---------------------------------------------------------------------------

/** Feature flags available per subscription tier. */
export interface TierFeatures {
  customSdk: boolean;
  apiAccess: boolean;
  marketplacePublish: boolean;
  familySharing: boolean;
  soc2Hipaa: boolean;
  prioritySupport: boolean;
}

/** Resource limits for a subscription tier. */
export interface TierLimits {
  /** Maximum number of active agents. null means unlimited. */
  maxAgents: number | null;
  /** Maximum cloud inference tokens per billing period. */
  cloudTokens: number;
  /** Federation participation mode. */
  federationMode: FederationMode;
  /** Feature flags for this tier. */
  features: TierFeatures;
}

// ---------------------------------------------------------------------------
// Tier data helpers
// ---------------------------------------------------------------------------

/** Monthly price in cents for each tier. Enterprise returns 0 (custom). */
export function monthlyPriceCents(tier: SubscriptionTier): number {
  const prices: Record<SubscriptionTier, number> = {
    Free: 0,
    Personal: 999,
    Family: 1999,
    Pro: 2999,
    Enterprise: 0,
    Developer: 0,
  };
  return prices[tier];
}

/** Returns the default TierLimits for a given SubscriptionTier. */
export function tierLimits(tier: SubscriptionTier): TierLimits {
  const noFeatures: TierFeatures = {
    customSdk: false,
    apiAccess: false,
    marketplacePublish: false,
    familySharing: false,
    soc2Hipaa: false,
    prioritySupport: false,
  };

  switch (tier) {
    case 'Free':
      return {
        maxAgents: 5,
        cloudTokens: 0,
        federationMode: FederationMode.ReceiveOnly,
        features: { ...noFeatures },
      };
    case 'Personal':
      return {
        maxAgents: null,
        cloudTokens: 100_000,
        federationMode: FederationMode.Full,
        features: { ...noFeatures },
      };
    case 'Family':
      return {
        maxAgents: null,
        cloudTokens: 200_000,
        federationMode: FederationMode.Full,
        features: { ...noFeatures, familySharing: true },
      };
    case 'Pro':
      return {
        maxAgents: null,
        cloudTokens: 500_000,
        federationMode: FederationMode.Full,
        features: {
          ...noFeatures,
          customSdk: true,
          apiAccess: true,
          prioritySupport: true,
        },
      };
    case 'Enterprise':
      return {
        maxAgents: null,
        cloudTokens: Number.MAX_SAFE_INTEGER,
        federationMode: FederationMode.Full,
        features: {
          customSdk: true,
          apiAccess: true,
          marketplacePublish: true,
          familySharing: true,
          soc2Hipaa: true,
          prioritySupport: true,
        },
      };
    case 'Developer':
      return {
        maxAgents: null,
        cloudTokens: 100_000,
        federationMode: FederationMode.Full,
        features: {
          ...noFeatures,
          customSdk: true,
          apiAccess: true,
          marketplacePublish: true,
        },
      };
  }

  // Exhaustiveness guard — unreachable if all SubscriptionTier cases are handled above.
  throw new Error(`Unknown subscription tier: ${tier as string}`);
}
