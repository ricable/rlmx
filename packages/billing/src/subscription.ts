/**
 * Subscription aggregate root: lifecycle management for user subscriptions.
 * Maps to rlmx-billing/src/subscription.rs.
 */

import { type SubscriptionTier, monthlyPriceCents, tierLimits } from '@aix/shared';
import type { SubscriptionStatus, SubscriptionEvent } from './events.js';
import {
  invalidTierTransition,
  notActive,
  alreadyCancelled,
} from './errors.js';
import {
  type UsageMetrics,
  createUsageMetrics,
  updateLimits,
  resetPeriod,
} from './usage.js';
import { createFamilyGroup, type FamilyGroup } from './family.js';
import { createDeveloperAccount, type DeveloperAccount } from './developer.js';

/** Grace period duration in days before a past-due subscription is suspended. */
export const GRACE_PERIOD_DAYS = 7;

/** The Subscription aggregate root -- owns billing lifecycle for a user. */
export interface Subscription {
  id: string;
  userId: string;
  plan: SubscriptionTier;
  status: SubscriptionStatus;
  usage: UsageMetrics;
  familyGroup: FamilyGroup | null;
  developerAccount: DeveloperAccount | null;
  createdAt: string;
  updatedAt: string;
  /** When the grace period expires (only set during GracePeriod). */
  gracePeriodEnd: string | null;
}

let _nextSubId = 0;
function generateId(): string {
  return `sub-${Date.now()}-${++_nextSubId}-${Math.random().toString(36).slice(2, 8)}`;
}

function addDays(date: Date, days: number): Date {
  const result = new Date(date);
  result.setDate(result.getDate() + days);
  return result;
}

/**
 * Create a new subscription for a user on the given tier.
 * Returns [subscription, event].
 */
export function subscribe(
  userId: string,
  tier: SubscriptionTier,
): [Subscription, SubscriptionEvent] {
  const now = new Date();
  const periodEnd = addDays(now, 30);
  const limits = tierLimits(tier);

  const familyGroup =
    tier === 'Family' ? createFamilyGroup(userId) : null;
  const developerAccount =
    tier === 'Developer'
      ? createDeveloperAccount(`dev-${userId}`)
      : null;

  const sub: Subscription = {
    id: generateId(),
    userId,
    plan: tier,
    status: 'Active',
    usage: createUsageMetrics(
      limits,
      now.toISOString(),
      periodEnd.toISOString(),
    ),
    familyGroup,
    developerAccount,
    createdAt: now.toISOString(),
    updatedAt: now.toISOString(),
    gracePeriodEnd: null,
  };

  const event: SubscriptionEvent = {
    type: 'Created',
    subscriptionId: sub.id,
    userId,
    tier,
  };

  return [sub, event];
}

/** Check whether the subscription is in an active (usable) state. */
export function isActive(sub: Subscription): boolean {
  return sub.status === 'Active' || sub.status === 'GracePeriod';
}

function requireActive(sub: Subscription): void {
  if (!isActive(sub)) {
    throw notActive(sub.status);
  }
}

/**
 * Upgrade to a higher tier.
 * Returns [updatedSubscription, event].
 */
export function upgrade(
  sub: Subscription,
  newTier: SubscriptionTier,
): [Subscription, SubscriptionEvent] {
  requireActive(sub);

  // Developer and Enterprise tiers have custom/zero pricing and are always
  // valid upgrade targets regardless of price comparison.
  const isSpecialTarget =
    newTier === 'Enterprise' || newTier === 'Developer';

  if (
    !isSpecialTarget &&
    monthlyPriceCents(newTier) <= monthlyPriceCents(sub.plan)
  ) {
    throw invalidTierTransition(sub.plan, newTier);
  }

  const oldTier = sub.plan;
  const newLimits = tierLimits(newTier);

  let familyGroup = sub.familyGroup;
  if (newTier === 'Family' && familyGroup === null) {
    familyGroup = createFamilyGroup(sub.userId);
  }

  let developerAccount = sub.developerAccount;
  if (newTier === 'Developer' && developerAccount === null) {
    developerAccount = createDeveloperAccount(`dev-${sub.userId}`);
  }

  const updated: Subscription = {
    ...sub,
    plan: newTier,
    usage: updateLimits(sub.usage, newLimits),
    familyGroup,
    developerAccount,
    updatedAt: new Date().toISOString(),
  };

  const event: SubscriptionEvent = {
    type: 'Upgraded',
    subscriptionId: sub.id,
    from: oldTier,
    to: newTier,
  };

  return [updated, event];
}

/**
 * Downgrade to a lower tier.
 * Returns [updatedSubscription, event].
 */
export function downgrade(
  sub: Subscription,
  newTier: SubscriptionTier,
): [Subscription, SubscriptionEvent] {
  requireActive(sub);

  if (
    monthlyPriceCents(newTier) >= monthlyPriceCents(sub.plan) &&
    sub.plan !== 'Enterprise'
  ) {
    throw invalidTierTransition(sub.plan, newTier);
  }

  const oldTier = sub.plan;
  const newLimits = tierLimits(newTier);

  // Remove family group if downgrading away from Family tier
  const familyGroup = newTier === 'Family' ? sub.familyGroup : null;

  const updated: Subscription = {
    ...sub,
    plan: newTier,
    usage: updateLimits(sub.usage, newLimits),
    familyGroup,
    updatedAt: new Date().toISOString(),
  };

  const event: SubscriptionEvent = {
    type: 'Downgraded',
    subscriptionId: sub.id,
    from: oldTier,
    to: newTier,
  };

  return [updated, event];
}

/**
 * Cancel the subscription.
 * Returns [updatedSubscription, event].
 */
export function cancel(
  sub: Subscription,
): [Subscription, SubscriptionEvent] {
  if (sub.status === 'Cancelled') {
    throw alreadyCancelled();
  }

  const updated: Subscription = {
    ...sub,
    status: 'Cancelled',
    updatedAt: new Date().toISOString(),
  };

  const event: SubscriptionEvent = {
    type: 'Cancelled',
    subscriptionId: sub.id,
  };

  return [updated, event];
}

/**
 * Reactivate a cancelled or suspended subscription.
 * Returns [updatedSubscription, event].
 */
export function reactivate(
  sub: Subscription,
  tier: SubscriptionTier,
): [Subscription, SubscriptionEvent] {
  if (sub.status !== 'Cancelled' && sub.status !== 'Suspended') {
    throw notActive(sub.status);
  }

  const now = new Date();
  const periodEnd = addDays(now, 30);
  const newLimits = tierLimits(tier);

  const updated: Subscription = {
    ...sub,
    plan: tier,
    status: 'Active',
    gracePeriodEnd: null,
    usage: resetPeriod(
      updateLimits(sub.usage, newLimits),
      now.toISOString(),
      periodEnd.toISOString(),
    ),
    updatedAt: now.toISOString(),
  };

  const event: SubscriptionEvent = {
    type: 'Reactivated',
    subscriptionId: sub.id,
    tier,
  };

  return [updated, event];
}

/**
 * Mark the subscription as past-due with a grace period.
 * Returns [updatedSubscription, event].
 */
export function markPastDue(
  sub: Subscription,
): [Subscription, SubscriptionEvent] {
  const oldStatus = sub.status;
  const now = new Date();
  const graceEnd = addDays(now, GRACE_PERIOD_DAYS);

  const updated: Subscription = {
    ...sub,
    status: 'GracePeriod',
    gracePeriodEnd: graceEnd.toISOString(),
    updatedAt: now.toISOString(),
  };

  const event: SubscriptionEvent = {
    type: 'StatusChanged',
    subscriptionId: sub.id,
    from: oldStatus,
    to: 'GracePeriod',
  };

  return [updated, event];
}

/**
 * Suspend the subscription (after grace period expires).
 * Returns [updatedSubscription, event].
 */
export function suspend(
  sub: Subscription,
): [Subscription, SubscriptionEvent] {
  const oldStatus = sub.status;

  const updated: Subscription = {
    ...sub,
    status: 'Suspended',
    gracePeriodEnd: null,
    updatedAt: new Date().toISOString(),
  };

  const event: SubscriptionEvent = {
    type: 'StatusChanged',
    subscriptionId: sub.id,
    from: oldStatus,
    to: 'Suspended',
  };

  return [updated, event];
}
