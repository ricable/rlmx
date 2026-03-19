/**
 * Marketplace aggregate root: top-level orchestrator for all marketplace operations.
 * Mirrors rlmx-marketplace/src/marketplace.rs.
 *
 * All mutations flow through this class. It coordinates the registry, billing,
 * review pipeline, publisher portal, featured engine, and analytics subsystems.
 */

import { LifeDomain, LIFE_DOMAINS } from '@aix/shared';
import { MarketplaceAnalytics } from './analytics.js';
import { BillingEngine } from './billing.js';
import type { MarketplaceEvent } from './events.js';
import { FeaturedEngine } from './featured.js';
import { PublisherPortal, addAgentToPublisher } from './publisher.js';
import {
  AgentRegistry,
  addRating,
  createDraftListing,
  recordInstall,
} from './registry.js';
import { ReviewPipeline } from './review.js';
import type {
  AgentPrice,
  DeviceType,
  ListingFilter,
  ModelTier,
  Permission,
  PublisherId,
  ScoredAgent,
} from './types.js';
import type { AgentListing } from './types.js';
import {
  DeveloperType,
  ListingSort,
  ListingStatus,
  ReviewDecision,
  ReviewStatus,
} from './types.js';
import { listingNotFound, publisherNotFound, submissionNotFound } from './errors.js';

// ---------------------------------------------------------------------------
// Marketplace
// ---------------------------------------------------------------------------

/** The marketplace aggregate root. All mutations flow through this class. */
export class Marketplace {
  readonly registry: AgentRegistry;
  readonly billing: BillingEngine;
  readonly reviewPipeline: ReviewPipeline;
  readonly publisherPortal: PublisherPortal;
  readonly featured: FeaturedEngine;
  readonly analytics: MarketplaceAnalytics;
  readonly categories: readonly LifeDomain[];

  constructor() {
    this.registry = new AgentRegistry();
    this.billing = new BillingEngine();
    this.reviewPipeline = new ReviewPipeline();
    this.publisherPortal = new PublisherPortal();
    this.featured = new FeaturedEngine();
    this.analytics = new MarketplaceAnalytics();
    this.categories = [...LIFE_DOMAINS];
  }

  /** Register a new publisher. */
  registerPublisher(
    name: string,
    email: string,
    developerType: DeveloperType,
  ): PublisherId {
    const id = this.publisherPortal.register(name, email, developerType);
    this.billing.ensureAccount(id);
    return id;
  }

  /**
   * Submit a new agent listing for review.
   * Returns { agentId, submissionId, event }.
   */
  submitAgent(params: {
    name: string;
    description: string;
    domain: LifeDomain;
    publisherId: PublisherId;
    rvfHash: string;
    version: string;
    price: AgentPrice;
    permissions: Permission[];
    devices: DeviceType[];
    minModelTier: ModelTier;
    sizeBytes: number;
  }): {
    agentId: string;
    submissionId: string;
    event: MarketplaceEvent;
  } {
    // Verify publisher exists
    const publisher = this.publisherPortal.get(params.publisherId);
    if (!publisher) {
      throw publisherNotFound(params.publisherId);
    }

    const listing = createDraftListing({
      name: params.name,
      description: params.description,
      domain: params.domain,
      publisher: params.publisherId,
      rvfHash: params.rvfHash,
      version: params.version,
      price: params.price,
      permissions: params.permissions,
      devices: params.devices,
      minModelTier: params.minModelTier,
      sizeBytes: params.sizeBytes,
    });

    const agentId = listing.id;
    this.registry.insert(listing);

    // Start review
    const submissionId = this.reviewPipeline.submit(agentId);
    this.registry.updateStatus(agentId, ListingStatus.InReview);

    // Add to publisher's portfolio
    const pubMut = this.publisherPortal.getMut(params.publisherId);
    if (pubMut) {
      addAgentToPublisher(pubMut, agentId);
    }

    const event: MarketplaceEvent = {
      type: 'AgentSubmitted',
      agentId,
      publisherId: params.publisherId,
    };

    return { agentId, submissionId, event };
  }

  /** Run automated review for a submission. */
  runReview(
    submissionId: string,
  ): { status: ReviewStatus; event: MarketplaceEvent } {
    const submission = this.reviewPipeline.get(submissionId);
    if (!submission) {
      throw submissionNotFound(submissionId);
    }

    const agentId = submission.agentId;
    const listing = this.registry.get(agentId);
    const permissions = listing?.permissionsRequired ?? [];

    const result = this.reviewPipeline.runAutomatedReview(
      submissionId,
      permissions,
    );

    const event: MarketplaceEvent = {
      type: 'ReviewCompleted',
      agentId,
      status: result.status,
    };

    // If auto-passed, set listing to Published
    if (result.status === ReviewStatus.AutoPassed) {
      this.registry.updateStatus(agentId, ListingStatus.Published);
    }

    return { status: result.status, event };
  }

  /** Complete human review for a flagged submission. */
  completeHumanReview(
    submissionId: string,
    reviewerId: string,
    decision: ReviewDecision,
    notes: string,
  ): { status: ReviewStatus; event: MarketplaceEvent } {
    const submission = this.reviewPipeline.get(submissionId);
    if (!submission) {
      throw submissionNotFound(submissionId);
    }

    const agentId = submission.agentId;
    const status = this.reviewPipeline.completeHumanReview(
      submissionId,
      reviewerId,
      decision,
      notes,
    );

    if (status === ReviewStatus.Approved) {
      this.registry.updateStatus(agentId, ListingStatus.Published);
    }

    const event: MarketplaceEvent = {
      type: 'ReviewCompleted',
      agentId,
      status,
    };

    return { status, event };
  }

  /** Install an agent for a user. */
  installAgent(
    agentId: string,
    userId: string,
    device: DeviceType,
  ): MarketplaceEvent {
    const listing = this.registry.get(agentId);
    if (!listing) {
      throw listingNotFound(agentId);
    }

    const publisherId = listing.publisher;
    const price = listing.price;
    const domain = listing.domain;

    // Record billing if paid
    const tx = this.billing.recordSale(publisherId, agentId, userId, price);
    if (tx) {
      this.analytics.recordRevenue(agentId, tx.grossCents);
    }

    // Update install count
    const listingMut = this.registry.getMut(agentId);
    if (listingMut) {
      recordInstall(listingMut);
    }

    // Track analytics
    this.analytics.recordInstall(agentId, userId, domain);

    return {
      type: 'AgentInstalled',
      agentId,
      userId,
      device,
    };
  }

  /** Uninstall an agent for a user. */
  uninstallAgent(agentId: string, userId: string): MarketplaceEvent {
    const listing = this.registry.get(agentId);
    if (!listing) {
      throw listingNotFound(agentId);
    }

    this.analytics.recordUninstall(agentId, userId);

    return {
      type: 'AgentUninstalled',
      agentId,
      userId,
    };
  }

  /**
   * Rate an agent.
   * Enforces DDD-010 invariant 6: auto-suspends agents with rating < 2.0
   * after 50+ ratings. Returns rating event and any suspension events.
   */
  rateAgent(
    agentId: string,
    userId: string,
    rating: number,
    review: string | null,
  ): MarketplaceEvent[] {
    const listing = this.registry.getMut(agentId);
    if (!listing) {
      throw listingNotFound(agentId);
    }

    addRating(listing, rating);

    const events: MarketplaceEvent[] = [
      {
        type: 'AgentRated',
        agentId,
        userId,
        rating,
        review,
      },
    ];

    // Enforce rating policy on the just-rated agent only.
    if (
      listing.ratingCount >= 50 &&
      listing.rating < 2.0 &&
      listing.status === ListingStatus.Published
    ) {
      listing.status = ListingStatus.Suspended;
      events.push({
        type: 'AgentSuspended',
        agentId,
        reason: 'Auto-suspended: rating below 2.0 after 50+ ratings',
      });
    }

    return events;
  }

  /** Search the registry. */
  search(filter: ListingFilter, sort: ListingSort): AgentListing[] {
    return this.registry.search(filter, sort);
  }

  /** Get featured agents. */
  featuredAgents(topN: number): ScoredAgent[] {
    const filter: ListingFilter = {
      status: ListingStatus.Published,
    };
    const published = this.registry.search(filter, ListingSort.Rating);
    return FeaturedEngine.rank(published, topN);
  }

  /** Run monthly payout cycle. */
  runPayoutCycle(): MarketplaceEvent[] {
    const payouts = this.billing.runPayoutCycle();
    return payouts.map(
      (p): MarketplaceEvent => ({
        type: 'PayoutProcessed',
        publisherId: p.publisherId,
        amountCents: p.amountCents,
      }),
    );
  }
}
