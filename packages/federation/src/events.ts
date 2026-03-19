/**
 * Federation domain events — crate-local event discriminated union.
 *
 * These are distinct from the kernel-level DomainEvent. They model
 * transitions within the Federated Learning bounded context (DDD-012).
 */

import type { LifeDomain } from '@aix/shared';

// ---------------------------------------------------------------------------
// Event interfaces
// ---------------------------------------------------------------------------

export interface CycleStartedEvent {
  type: 'CycleStarted';
  cycleId: string;
  cycleNumber: number;
  startedAt: string;
}

export interface ContributionReceivedEvent {
  type: 'ContributionReceived';
  cycleId: string;
  contributionId: string;
  domain: LifeDomain;
  pseudonym: string;
  patternCount: number;
  timestamp: string;
}

export interface AggregationCompletedEvent {
  type: 'AggregationCompleted';
  cycleId: string;
  domainsAggregated: number;
  totalContributions: number;
  timestamp: string;
}

export interface PackageDistributedEvent {
  type: 'PackageDistributed';
  cycleId: string;
  packageId: string;
  domain: LifeDomain;
  version: string;
  contributorCount: number;
  timestamp: string;
}

export interface CycleCompletedEvent {
  type: 'CycleCompleted';
  cycleId: string;
  cycleNumber: number;
  packagesPublished: number;
  completedAt: string;
}

/**
 * Discriminated union of all federation domain events.
 */
export type FederationEvent =
  | CycleStartedEvent
  | ContributionReceivedEvent
  | AggregationCompletedEvent
  | PackageDistributedEvent
  | CycleCompletedEvent;

/** All valid FederationEvent type discriminators. */
export type FederationEventType = FederationEvent['type'];

/** Extract the event interface for a specific type discriminator. */
export type FederationEventOf<T extends FederationEventType> = Extract<
  FederationEvent,
  { type: T }
>;

/** All event type strings for iteration/validation. */
export const FEDERATION_EVENT_TYPES: readonly FederationEventType[] = [
  'CycleStarted',
  'ContributionReceived',
  'AggregationCompleted',
  'PackageDistributed',
  'CycleCompleted',
] as const;
