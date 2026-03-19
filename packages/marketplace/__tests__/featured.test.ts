/**
 * Tests for the FeaturedEngine.
 * Mirrors rlmx-marketplace/src/featured.rs tests.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { LifeDomain } from '@aix/shared';
import { FeaturedEngine } from '../src/featured.js';
import { createDraftListing } from '../src/registry.js';
import {
  DeviceType,
  ListingStatus,
  ModelTier,
  newPublisherId,
} from '../src/types.js';
import type { AgentListing } from '../src/types.js';

function makeListingWithStats(
  name: string,
  domain: LifeDomain,
  rating: number,
  installs: number,
): AgentListing {
  const listing = createDraftListing({
    name,
    description: `${name} desc`,
    domain,
    publisher: newPublisherId(),
    rvfHash: '0'.repeat(64),
    version: '1.0.0',
    price: { type: 'Free' },
    permissions: [],
    devices: [DeviceType.Desktop],
    minModelTier: ModelTier.Small,
    sizeBytes: 1024,
  });
  listing.rating = rating;
  listing.installCount = installs;
  listing.publishedAt = new Date().toISOString();
  listing.status = ListingStatus.Published;
  return listing;
}

describe('FeaturedEngine', () => {
  describe('compute score', () => {
    it('should score higher for higher ratings', () => {
      const low = makeListingWithStats('Low', LifeDomain.Finance, 1.0, 100);
      const high = makeListingWithStats('High', LifeDomain.Finance, 5.0, 100);

      expect(FeaturedEngine.computeScore(high)).toBeGreaterThan(
        FeaturedEngine.computeScore(low),
      );
    });

    it('should score higher for more installs', () => {
      const few = makeListingWithStats('Few', LifeDomain.Finance, 4.0, 10);
      const many = makeListingWithStats('Many', LifeDomain.Finance, 4.0, 10000);

      expect(FeaturedEngine.computeScore(many)).toBeGreaterThan(
        FeaturedEngine.computeScore(few),
      );
    });

    it('should return 0 recency for unpublished listings', () => {
      const listing = makeListingWithStats('New', LifeDomain.Finance, 4.0, 100);
      listing.publishedAt = null;

      // Should still compute without error, recency component is 0
      const score = FeaturedEngine.computeScore(listing);
      expect(score).toBeGreaterThanOrEqual(0);
    });
  });

  describe('rank', () => {
    it('should return top N listings by score', () => {
      const a = makeListingWithStats('A', LifeDomain.Finance, 4.5, 1000);
      const b = makeListingWithStats('B', LifeDomain.Finance, 3.0, 500);
      const c = makeListingWithStats('C', LifeDomain.Finance, 5.0, 2000);

      const ranked = FeaturedEngine.rank([a, b, c], 2);
      expect(ranked).toHaveLength(2);
      // Highest scorer first
      expect(ranked[0].score).toBeGreaterThanOrEqual(ranked[1].score);
    });

    it('should return empty for empty input', () => {
      const ranked = FeaturedEngine.rank([], 5);
      expect(ranked).toHaveLength(0);
    });

    it('should return all if topN exceeds list length', () => {
      const a = makeListingWithStats('A', LifeDomain.Finance, 4.0, 100);
      const ranked = FeaturedEngine.rank([a], 10);
      expect(ranked).toHaveLength(1);
    });
  });

  describe('recommend for domain', () => {
    it('should filter by domain before ranking', () => {
      const fin = makeListingWithStats('Fin', LifeDomain.Finance, 5.0, 1000);
      const health = makeListingWithStats('Health', LifeDomain.Health, 5.0, 1000);

      const recs = FeaturedEngine.recommendForDomain(
        [fin, health],
        LifeDomain.Finance,
        5,
      );
      expect(recs).toHaveLength(1);
      expect(recs[0].agentId).toBe(fin.id);
    });

    it('should return empty for domain with no agents', () => {
      const fin = makeListingWithStats('Fin', LifeDomain.Finance, 5.0, 1000);
      const recs = FeaturedEngine.recommendForDomain(
        [fin],
        LifeDomain.Pet,
        5,
      );
      expect(recs).toHaveLength(0);
    });
  });

  describe('trending', () => {
    it('should rank by install count', () => {
      const few = makeListingWithStats('Few', LifeDomain.Finance, 5.0, 10);
      const many = makeListingWithStats('Many', LifeDomain.Finance, 3.0, 10000);

      const trending = FeaturedEngine.trending([few, many], 2);
      expect(trending[0].agentId).toBe(many.id);
    });
  });

  describe('curated management', () => {
    let engine: FeaturedEngine;

    beforeEach(() => {
      engine = new FeaturedEngine();
    });

    it('should add and list curated agents', () => {
      const id = crypto.randomUUID();
      engine.addCurated(id);
      expect(engine.curatedList()).toHaveLength(1);
      expect(engine.curatedList()[0]).toBe(id);
    });

    it('should not add duplicates', () => {
      const id = crypto.randomUUID();
      engine.addCurated(id);
      engine.addCurated(id);
      expect(engine.curatedList()).toHaveLength(1);
    });

    it('should remove curated agents', () => {
      const id = crypto.randomUUID();
      engine.addCurated(id);
      engine.removeCurated(id);
      expect(engine.curatedList()).toHaveLength(0);
    });

    it('should handle removing non-existent agent', () => {
      engine.removeCurated(crypto.randomUUID());
      expect(engine.curatedList()).toHaveLength(0);
    });
  });
});
