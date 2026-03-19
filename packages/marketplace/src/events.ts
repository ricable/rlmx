/**
 * Marketplace domain events (DDD-010).
 * Mirrors rlmx-marketplace/src/domain.rs MarketplaceDomainEvent enum.
 */

import type {
  DeviceType,
  PublisherId,
  ReviewStatus,
} from './types.js';

import type { LifeDomain } from '@aix/shared';

// ---------------------------------------------------------------------------
// Event interfaces
// ---------------------------------------------------------------------------

export interface AgentSubmittedEvent {
  type: 'AgentSubmitted';
  agentId: string;
  publisherId: PublisherId;
}

export interface ReviewStartedEvent {
  type: 'ReviewStarted';
  agentId: string;
  submissionId: string;
}

export interface ReviewCompletedEvent {
  type: 'ReviewCompleted';
  agentId: string;
  status: ReviewStatus;
}

export interface AgentPublishedEvent {
  type: 'AgentPublished';
  agentId: string;
  domain: LifeDomain;
}

export interface AgentInstalledEvent {
  type: 'AgentInstalled';
  agentId: string;
  userId: string;
  device: DeviceType;
}

export interface AgentUninstalledEvent {
  type: 'AgentUninstalled';
  agentId: string;
  userId: string;
}

export interface AgentRatedEvent {
  type: 'AgentRated';
  agentId: string;
  userId: string;
  rating: number;
  review: string | null;
}

export interface AgentSuspendedEvent {
  type: 'AgentSuspended';
  agentId: string;
  reason: string;
}

export interface PayoutProcessedEvent {
  type: 'PayoutProcessed';
  publisherId: PublisherId;
  amountCents: number;
}

export interface PackCreatedEvent {
  type: 'PackCreated';
  packId: string;
  creator: PublisherId;
}

// ---------------------------------------------------------------------------
// Discriminated union
// ---------------------------------------------------------------------------

/** Discriminated union of all marketplace domain events. */
export type MarketplaceEvent =
  | AgentSubmittedEvent
  | ReviewStartedEvent
  | ReviewCompletedEvent
  | AgentPublishedEvent
  | AgentInstalledEvent
  | AgentUninstalledEvent
  | AgentRatedEvent
  | AgentSuspendedEvent
  | PayoutProcessedEvent
  | PackCreatedEvent;

/** All valid marketplace event type discriminators. */
export type MarketplaceEventType = MarketplaceEvent['type'];

/** Extract the event interface for a specific type discriminator. */
export type MarketplaceEventOf<T extends MarketplaceEventType> = Extract<
  MarketplaceEvent,
  { type: T }
>;

/** All MarketplaceEvent type strings. */
export const MARKETPLACE_EVENT_TYPES: readonly MarketplaceEventType[] = [
  'AgentSubmitted',
  'ReviewStarted',
  'ReviewCompleted',
  'AgentPublished',
  'AgentInstalled',
  'AgentUninstalled',
  'AgentRated',
  'AgentSuspended',
  'PayoutProcessed',
  'PackCreated',
] as const;
