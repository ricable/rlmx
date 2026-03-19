/**
 * Tests for the Marketplace aggregate root.
 * Mirrors rlmx-marketplace/src/marketplace.rs tests.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { LifeDomain } from '@aix/shared';
import { Marketplace } from '../src/marketplace.js';
import {
  DeveloperType,
  DeviceType,
  ListingSort,
  ListingStatus,
  ModelTier,
  ReviewDecision,
  ReviewStatus,
  createPermission,
} from '../src/types.js';
import { MarketplaceError, MarketplaceErrorCode } from '../src/errors.js';

describe('Marketplace', () => {
  let mp: Marketplace;

  beforeEach(() => {
    mp = new Marketplace();
  });

  it('should have all 12 life domain categories', () => {
    expect(mp.categories).toHaveLength(12);
  });

  describe('full agent lifecycle', () => {
    it('should handle submit -> review -> publish -> install -> rate', () => {
      // Register publisher
      const pubId = mp.registerPublisher(
        'TestDev',
        'dev@test.com',
        DeveloperType.Individual,
      );

      // Submit agent
      const { agentId, submissionId, event: submitEvent } = mp.submitAgent({
        name: 'BudgetBot',
        description: 'Personal budget assistant',
        domain: LifeDomain.Finance,
        publisherId: pubId,
        rvfHash: 'abc123',
        version: '1.0.0',
        price: { type: 'OneTime', cents: 999 },
        permissions: [createPermission('vec_search')],
        devices: [DeviceType.Desktop, DeviceType.Mobile],
        minModelTier: ModelTier.Small,
        sizeBytes: 2048,
      });

      expect(submitEvent.type).toBe('AgentSubmitted');

      // Run review (should auto-pass, no sensitive permissions)
      const { status, event: reviewEvent } = mp.runReview(submissionId);
      expect(status).toBe(ReviewStatus.AutoPassed);
      expect(reviewEvent.type).toBe('ReviewCompleted');

      // Verify listing is published
      const listing = mp.registry.get(agentId);
      expect(listing).toBeDefined();
      expect(listing!.status).toBe(ListingStatus.Published);

      // Install
      const installEvent = mp.installAgent(
        agentId,
        crypto.randomUUID(),
        DeviceType.Desktop,
      );
      expect(installEvent.type).toBe('AgentInstalled');
      expect(mp.registry.get(agentId)!.installCount).toBe(1);

      // Rate
      const rateEvents = mp.rateAgent(
        agentId,
        crypto.randomUUID(),
        4.5,
        'Great!',
      );
      expect(rateEvents).toHaveLength(1);
      expect(rateEvents[0].type).toBe('AgentRated');

      const ratedListing = mp.registry.get(agentId)!;
      expect(ratedListing.rating).toBeCloseTo(4.5, 1);
    });
  });

  describe('sensitive agent requires human review', () => {
    it('should flag sensitive permissions for human review', () => {
      const pubId = mp.registerPublisher(
        'HealthDev',
        'health@dev.com',
        DeveloperType.Organization,
      );

      const { submissionId } = mp.submitAgent({
        name: 'HealthTracker',
        description: 'Health monitoring agent',
        domain: LifeDomain.Health,
        publisherId: pubId,
        rvfHash: 'def456',
        version: '1.0.0',
        price: { type: 'Monthly', cents: 499 },
        permissions: [createPermission('health_data')],
        devices: [DeviceType.Mobile],
        minModelTier: ModelTier.Medium,
        sizeBytes: 4096,
      });

      const { status } = mp.runReview(submissionId);
      expect(status).toBe(ReviewStatus.FlaggedForHuman);

      // Human approves
      const result = mp.completeHumanReview(
        submissionId,
        'doc_reviewer',
        ReviewDecision.Approve,
        'HIPAA compliant',
      );
      expect(result.status).toBe(ReviewStatus.Approved);
    });
  });

  describe('uninstall agent', () => {
    it('should track analytics on uninstall', () => {
      const pubId = mp.registerPublisher(
        'UninstDev',
        'uninst@dev.com',
        DeveloperType.Individual,
      );

      const { agentId, submissionId } = mp.submitAgent({
        name: 'RemoveMe',
        description: 'Agent to uninstall',
        domain: LifeDomain.Shopping,
        publisherId: pubId,
        rvfHash: 'ghi789',
        version: '1.0.0',
        price: { type: 'Free' },
        permissions: [],
        devices: [DeviceType.Desktop],
        minModelTier: ModelTier.Small,
        sizeBytes: 1024,
      });

      mp.runReview(submissionId);

      const userId = crypto.randomUUID();
      mp.installAgent(agentId, userId, DeviceType.Desktop);

      const event = mp.uninstallAgent(agentId, userId);
      expect(event.type).toBe('AgentUninstalled');
      if (event.type === 'AgentUninstalled') {
        expect(event.agentId).toBe(agentId);
        expect(event.userId).toBe(userId);
      }

      // Verify analytics recorded the uninstall
      const metrics = mp.analytics.agentMetrics(agentId);
      expect(metrics).toBeDefined();
      expect(metrics!.totalUninstalls).toBe(1);
    });
  });

  describe('low rating auto-suspension', () => {
    it('should auto-suspend agents with rating < 2.0 after 50+ ratings', () => {
      const pubId = mp.registerPublisher(
        'BadDev',
        'bad@dev.com',
        DeveloperType.Individual,
      );

      const { agentId, submissionId } = mp.submitAgent({
        name: 'BadAgent',
        description: 'Poorly rated agent',
        domain: LifeDomain.Social,
        publisherId: pubId,
        rvfHash: 'jkl012',
        version: '1.0.0',
        price: { type: 'Free' },
        permissions: [],
        devices: [DeviceType.Desktop],
        minModelTier: ModelTier.Small,
        sizeBytes: 512,
      });

      mp.runReview(submissionId);

      // Directly set the listing stats to avoid 50+ rate_agent calls
      const listing = mp.registry.getMut(agentId)!;
      listing.rating = 1.8;
      listing.ratingCount = 49;

      // The 50th rating should trigger the policy
      const events = mp.rateAgent(agentId, crypto.randomUUID(), 1.0, null);

      // Should have both the rating event and the suspension event
      expect(events).toHaveLength(2);
      expect(events.some((e) => e.type === 'AgentSuspended')).toBe(true);

      // Verify listing is suspended
      expect(mp.registry.get(agentId)!.status).toBe(ListingStatus.Suspended);
    });

    it('should not suspend agents above 2.0 rating', () => {
      const pubId = mp.registerPublisher(
        'OkDev',
        'ok@dev.com',
        DeveloperType.Individual,
      );

      const { agentId, submissionId } = mp.submitAgent({
        name: 'OkAgent',
        description: 'Decent agent',
        domain: LifeDomain.Finance,
        publisherId: pubId,
        rvfHash: 'mno345',
        version: '1.0.0',
        price: { type: 'Free' },
        permissions: [],
        devices: [DeviceType.Desktop],
        minModelTier: ModelTier.Small,
        sizeBytes: 512,
      });

      mp.runReview(submissionId);

      const listing = mp.registry.getMut(agentId)!;
      listing.rating = 3.0;
      listing.ratingCount = 49;

      const events = mp.rateAgent(agentId, crypto.randomUUID(), 2.5, null);
      expect(events).toHaveLength(1); // Only rating, no suspension
      expect(mp.registry.get(agentId)!.status).toBe(ListingStatus.Published);
    });
  });

  describe('search', () => {
    it('should search by domain', () => {
      const pubId = mp.registerPublisher(
        'SearchDev',
        'search@dev.com',
        DeveloperType.Individual,
      );

      const { submissionId: s1 } = mp.submitAgent({
        name: 'FinAgent',
        description: 'Finance agent',
        domain: LifeDomain.Finance,
        publisherId: pubId,
        rvfHash: 'a1',
        version: '1.0.0',
        price: { type: 'Free' },
        permissions: [],
        devices: [DeviceType.Desktop],
        minModelTier: ModelTier.Small,
        sizeBytes: 1024,
      });
      mp.runReview(s1);

      const { submissionId: s2 } = mp.submitAgent({
        name: 'HealthAgent',
        description: 'Health agent',
        domain: LifeDomain.Health,
        publisherId: pubId,
        rvfHash: 'a2',
        version: '1.0.0',
        price: { type: 'Free' },
        permissions: [],
        devices: [DeviceType.Desktop],
        minModelTier: ModelTier.Small,
        sizeBytes: 1024,
      });
      mp.runReview(s2);

      const results = mp.search(
        { domain: LifeDomain.Finance, status: ListingStatus.Published },
        ListingSort.Rating,
      );
      expect(results).toHaveLength(1);
      expect(results[0].domain).toBe(LifeDomain.Finance);
    });
  });

  describe('featured agents', () => {
    it('should return featured agents ranked by score', () => {
      const pubId = mp.registerPublisher(
        'FeatDev',
        'feat@dev.com',
        DeveloperType.Individual,
      );

      // Create two published agents with different stats
      for (let i = 0; i < 2; i++) {
        const { agentId, submissionId } = mp.submitAgent({
          name: `Agent${i}`,
          description: `Agent ${i} description`,
          domain: LifeDomain.Career,
          publisherId: pubId,
          rvfHash: `feat${i}`,
          version: '1.0.0',
          price: { type: 'Free' },
          permissions: [],
          devices: [DeviceType.Desktop],
          minModelTier: ModelTier.Small,
          sizeBytes: 1024,
        });
        mp.runReview(submissionId);

        // Give them different ratings
        const listing = mp.registry.getMut(agentId)!;
        listing.rating = i === 0 ? 4.5 : 3.0;
        listing.installCount = i === 0 ? 1000 : 100;
      }

      const featured = mp.featuredAgents(5);
      expect(featured.length).toBeGreaterThanOrEqual(1);
      // First should have higher score
      if (featured.length >= 2) {
        expect(featured[0].score).toBeGreaterThanOrEqual(featured[1].score);
      }
    });
  });

  describe('payout cycle', () => {
    it('should process payouts for eligible publishers', () => {
      const pubId = mp.registerPublisher(
        'PayDev',
        'pay@dev.com',
        DeveloperType.Individual,
      );

      // Set payout method
      mp.billing.setPayoutMethod(pubId, {
        type: 'StripeConnect',
        accountId: 'acct_123',
      });

      const { agentId, submissionId } = mp.submitAgent({
        name: 'PaidAgent',
        description: 'Agent to test payouts',
        domain: LifeDomain.Finance,
        publisherId: pubId,
        rvfHash: 'pay1',
        version: '1.0.0',
        price: { type: 'OneTime', cents: 10000 },
        permissions: [],
        devices: [DeviceType.Desktop],
        minModelTier: ModelTier.Small,
        sizeBytes: 1024,
      });
      mp.runReview(submissionId);

      // Install (triggers billing)
      mp.installAgent(agentId, crypto.randomUUID(), DeviceType.Desktop);

      // Run payout cycle
      const events = mp.runPayoutCycle();
      expect(events).toHaveLength(1);
      expect(events[0].type).toBe('PayoutProcessed');
      if (events[0].type === 'PayoutProcessed') {
        expect(events[0].publisherId).toBe(pubId);
        expect(events[0].amountCents).toBe(7000); // 70% of 10000
      }
    });
  });

  describe('error handling', () => {
    it('should throw on listing not found for install', () => {
      expect(() =>
        mp.installAgent(crypto.randomUUID(), crypto.randomUUID(), DeviceType.Desktop),
      ).toThrow(MarketplaceError);
    });

    it('should throw on publisher not found for submit', () => {
      expect(() =>
        mp.submitAgent({
          name: 'Orphan',
          description: 'No publisher',
          domain: LifeDomain.Finance,
          publisherId: crypto.randomUUID() as any,
          rvfHash: 'x',
          version: '1.0.0',
          price: { type: 'Free' },
          permissions: [],
          devices: [DeviceType.Desktop],
          minModelTier: ModelTier.Small,
          sizeBytes: 100,
        }),
      ).toThrow(MarketplaceError);
    });

    it('should throw on submission not found for review', () => {
      expect(() => mp.runReview(crypto.randomUUID())).toThrow(MarketplaceError);
    });

    it('should throw on listing not found for uninstall', () => {
      expect(() =>
        mp.uninstallAgent(crypto.randomUUID(), crypto.randomUUID()),
      ).toThrow(MarketplaceError);
    });

    it('should throw on listing not found for rate', () => {
      expect(() =>
        mp.rateAgent(crypto.randomUUID(), crypto.randomUUID(), 5.0, null),
      ).toThrow(MarketplaceError);
    });
  });
});
