/**
 * Usage tracking: metering cloud tokens and agent counts per billing period.
 * Maps to rlmx-billing/src/usage.rs UsageMetrics.
 */

import type { TierLimits } from '@aix/shared';
import { quotaExceeded } from './errors.js';

export interface UsageMetrics {
  /** Cloud inference tokens consumed this period. */
  cloudTokensUsed: number;
  /** Cloud token limit for the current tier. */
  cloudTokensLimit: number;
  /** Currently active agent count. */
  agentsActive: number;
  /** Agent limit for the current tier (null = unlimited). */
  agentsLimit: number | null;
  /** Start of the current billing period (ISO 8601). */
  periodStart: string;
  /** End of the current billing period (ISO 8601). */
  periodEnd: string;
}

/** Create a new UsageMetrics for the given tier limits and period. */
export function createUsageMetrics(
  limits: TierLimits,
  periodStart: string,
  periodEnd: string,
): UsageMetrics {
  return {
    cloudTokensUsed: 0,
    cloudTokensLimit: limits.cloudTokens,
    agentsActive: 0,
    agentsLimit: limits.maxAgents,
    periodStart,
    periodEnd,
  };
}

/**
 * Record cloud token usage. Throws BillingError if quota would be exceeded.
 * Returns a new UsageMetrics (immutable style).
 */
export function recordTokenUsage(
  metrics: UsageMetrics,
  tokens: number,
): UsageMetrics {
  const newTotal = metrics.cloudTokensUsed + tokens;
  if (newTotal > metrics.cloudTokensLimit) {
    throw quotaExceeded('cloud_tokens', newTotal, metrics.cloudTokensLimit);
  }
  return { ...metrics, cloudTokensUsed: newTotal };
}

/**
 * Check whether a new agent can be spawned within the current quota.
 * Throws BillingError if the agent limit is reached.
 */
export function checkAgentQuota(metrics: UsageMetrics): void {
  if (
    metrics.agentsLimit !== null &&
    metrics.agentsActive >= metrics.agentsLimit
  ) {
    throw quotaExceeded(
      'agents',
      metrics.agentsActive,
      metrics.agentsLimit,
    );
  }
}

/** Increment the active agent count (after spawn-time check). */
export function recordAgentSpawn(metrics: UsageMetrics): UsageMetrics {
  return { ...metrics, agentsActive: metrics.agentsActive + 1 };
}

/** Decrement the active agent count when an agent is terminated. */
export function recordAgentTerminate(metrics: UsageMetrics): UsageMetrics {
  return {
    ...metrics,
    agentsActive: Math.max(0, metrics.agentsActive - 1),
  };
}

/** Reset usage counters for a new billing period. */
export function resetPeriod(
  metrics: UsageMetrics,
  newStart: string,
  newEnd: string,
): UsageMetrics {
  return {
    ...metrics,
    cloudTokensUsed: 0,
    periodStart: newStart,
    periodEnd: newEnd,
  };
}

/** Update limits when the subscription tier changes. */
export function updateLimits(
  metrics: UsageMetrics,
  limits: TierLimits,
): UsageMetrics {
  return {
    ...metrics,
    cloudTokensLimit: limits.cloudTokens,
    agentsLimit: limits.maxAgents,
  };
}

/** Percentage of cloud tokens consumed (0.0 - 1.0). */
export function tokenUtilization(metrics: UsageMetrics): number {
  if (metrics.cloudTokensLimit === 0) {
    return metrics.cloudTokensUsed > 0 ? 1.0 : 0.0;
  }
  return metrics.cloudTokensUsed / metrics.cloudTokensLimit;
}
