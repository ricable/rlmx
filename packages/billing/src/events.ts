/**
 * Subscription domain events — discriminated union.
 * Maps to rlmx-billing SubscriptionEvent + SubscriptionStatus.
 */

import type { SubscriptionTier } from '@aix/shared';

/** Status of a subscription in the state machine. */
export type SubscriptionStatus =
  | 'Active'
  | 'PastDue'
  | 'GracePeriod'
  | 'Suspended'
  | 'Cancelled';

/** Domain events emitted by the Subscription aggregate. */
export type SubscriptionEvent =
  | {
      type: 'Created';
      subscriptionId: string;
      userId: string;
      tier: SubscriptionTier;
    }
  | {
      type: 'Upgraded';
      subscriptionId: string;
      from: SubscriptionTier;
      to: SubscriptionTier;
    }
  | {
      type: 'Downgraded';
      subscriptionId: string;
      from: SubscriptionTier;
      to: SubscriptionTier;
    }
  | {
      type: 'Cancelled';
      subscriptionId: string;
    }
  | {
      type: 'Reactivated';
      subscriptionId: string;
      tier: SubscriptionTier;
    }
  | {
      type: 'StatusChanged';
      subscriptionId: string;
      from: SubscriptionStatus;
      to: SubscriptionStatus;
    };
