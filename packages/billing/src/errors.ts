/**
 * Billing error types.
 * Maps to rlmx-billing/src/error.rs BillingError enum.
 */

import type { SubscriptionTier } from '@aix/shared';
import type { SubscriptionStatus } from './events.js';

/** Base billing error with a discriminated `code` field. */
export class BillingError extends Error {
  constructor(
    message: string,
    public readonly code: BillingErrorCode,
    public readonly details?: Record<string, unknown>,
  ) {
    super(message);
    this.name = 'BillingError';
    Object.setPrototypeOf(this, new.target.prototype);
  }
}

export type BillingErrorCode =
  | 'SUBSCRIPTION_NOT_FOUND'
  | 'INVALID_TIER_TRANSITION'
  | 'NOT_ACTIVE'
  | 'QUOTA_EXCEEDED'
  | 'FAMILY_GROUP_NOT_FOUND'
  | 'FAMILY_GROUP_FULL'
  | 'MEMBER_NOT_FOUND'
  | 'MEMBER_ALREADY_EXISTS'
  | 'CANNOT_REMOVE_OWNER'
  | 'DEVELOPER_ACCOUNT_NOT_FOUND'
  | 'PAYOUT_THRESHOLD_NOT_MET'
  | 'NO_PAYOUT_METHOD'
  | 'FEATURE_NOT_AVAILABLE'
  | 'ALREADY_CANCELLED'
  | 'INTERNAL';

export function subscriptionNotFound(id: string): BillingError {
  return new BillingError(
    `subscription not found: ${id}`,
    'SUBSCRIPTION_NOT_FOUND',
    { id },
  );
}

export function invalidTierTransition(
  from: SubscriptionTier,
  to: SubscriptionTier,
): BillingError {
  return new BillingError(
    `invalid tier transition from ${from} to ${to}`,
    'INVALID_TIER_TRANSITION',
    { from, to },
  );
}

export function notActive(status: SubscriptionStatus): BillingError {
  return new BillingError(
    `subscription not active: current status is ${status}`,
    'NOT_ACTIVE',
    { status },
  );
}

export function quotaExceeded(
  resource: string,
  used: number,
  limit: number,
): BillingError {
  return new BillingError(
    `usage quota exceeded: ${resource} (${used}/${limit})`,
    'QUOTA_EXCEEDED',
    { resource, used, limit },
  );
}

export function familyGroupFull(max: number): BillingError {
  return new BillingError(
    `family group full: max ${max} members`,
    'FAMILY_GROUP_FULL',
    { max },
  );
}

export function memberNotFound(userId: string): BillingError {
  return new BillingError(
    `member not found: ${userId}`,
    'MEMBER_NOT_FOUND',
    { userId },
  );
}

export function memberAlreadyExists(userId: string): BillingError {
  return new BillingError(
    `member already in family group: ${userId}`,
    'MEMBER_ALREADY_EXISTS',
    { userId },
  );
}

export function cannotRemoveOwner(): BillingError {
  return new BillingError(
    'cannot remove family owner',
    'CANNOT_REMOVE_OWNER',
  );
}

export function payoutThresholdNotMet(
  balanceCents: number,
  thresholdCents: number,
): BillingError {
  return new BillingError(
    `payout threshold not met: balance ${balanceCents} < threshold ${thresholdCents}`,
    'PAYOUT_THRESHOLD_NOT_MET',
    { balanceCents, thresholdCents },
  );
}

export function noPayoutMethod(): BillingError {
  return new BillingError(
    'no payout method configured',
    'NO_PAYOUT_METHOD',
  );
}

export function featureNotAvailable(
  tier: SubscriptionTier,
  feature: string,
): BillingError {
  return new BillingError(
    `feature not available on ${tier} tier: ${feature}`,
    'FEATURE_NOT_AVAILABLE',
    { tier, feature },
  );
}

export function alreadyCancelled(): BillingError {
  return new BillingError(
    'already cancelled',
    'ALREADY_CANCELLED',
  );
}
