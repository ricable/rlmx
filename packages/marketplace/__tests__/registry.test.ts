/**
 * Tests for the AgentRegistry.
 * Mirrors rlmx-marketplace/src/registry.rs tests.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { LifeDomain } from '@aix/shared';
import {
  AgentRegistry,
  createDraftListing,
  addRating,
} from '../src/registry.js';
import {
  DeviceType,
  ListingSort,
  ListingStatus,
  ModelTier,
  newPublisherId,
} from '../src/types.js';
import type { AgentListing, AgentPrice, PublisherId } from '../src/types.js';

function makeListing(
  name: string,
  domain: LifeDomain,
  price: AgentPrice,
): AgentListing {
  return createDraftListing({
    name,
    description: `${name} description`,
    domain,
    publisher: newPublisherId(),
    rvfHash: '0'.repeat(64),
    version: '1.0.0',
    price,
    permissions: [],
    devices: [DeviceType.Desktop],
    minModelTier: ModelTier.Small,
    sizeBytes: 1024,
  });
}

describe('AgentRegistry', () => {
  let reg: AgentRegistry;

  beforeEach(() => {
    reg = new AgentRegistry();
  });

  it('should insert and get a listing', () => {
    const listing = makeListing('TestAgent', LifeDomain.Finance, { type: 'Free' });
    const id = listing.id;
    reg.insert(listing);
    expect(reg.get(id)).toBeDefined();
    expect(reg.count()).toBe(1);
  });

  it('should remove a listing', () => {
    const listing = makeListing('ToRemove', LifeDomain.Health, { type: 'Free' });
    const id = listing.id;
    reg.insert(listing);
    expect(reg.remove(id)).toBeDefined();
    expect(reg.count()).toBe(0);
  });

  it('should return undefined for non-existent listing', () => {
    expect(reg.get(crypto.randomUUID())).toBeUndefined();
  });

  describe('search', () => {
    it('should search by domain', () => {
      reg.insert(makeListing('FinAgent', LifeDomain.Finance, { type: 'Free' }));
      reg.insert(makeListing('HealthAgent', LifeDomain.Health, { type: 'Free' }));

      const results = reg.search(
        { domain: LifeDomain.Finance },
        ListingSort.Rating,
      );
      expect(results).toHaveLength(1);
      expect(results[0].domain).toBe(LifeDomain.Finance);
    });

    it('should search by keyword', () => {
      reg.insert(
        makeListing('BudgetTracker', LifeDomain.Finance, { type: 'Free' }),
      );
      reg.insert(
        makeListing('FitnessCoach', LifeDomain.Health, { type: 'Free' }),
      );

      const results = reg.search(
        { keyword: 'budget' },
        ListingSort.Rating,
      );
      expect(results).toHaveLength(1);
      expect(results[0].name).toBe('BudgetTracker');
    });

    it('should search by keyword in description', () => {
      const listing = makeListing('Agent', LifeDomain.Finance, { type: 'Free' });
      listing.description = 'Helps with crypto investments';
      reg.insert(listing);

      const results = reg.search(
        { keyword: 'crypto' },
        ListingSort.Rating,
      );
      expect(results).toHaveLength(1);
    });

    it('should search by max price', () => {
      reg.insert(
        makeListing('Cheap', LifeDomain.Finance, { type: 'OneTime', cents: 500 }),
      );
      reg.insert(
        makeListing('Expensive', LifeDomain.Finance, {
          type: 'OneTime',
          cents: 5000,
        }),
      );

      const results = reg.search(
        { maxPriceCents: 1000 },
        ListingSort.PriceLow,
      );
      expect(results).toHaveLength(1);
      expect(results[0].name).toBe('Cheap');
    });

    it('should search by status', () => {
      const published = makeListing('Pub', LifeDomain.Finance, { type: 'Free' });
      published.status = ListingStatus.Published;
      reg.insert(published);

      const draft = makeListing('Draft', LifeDomain.Finance, { type: 'Free' });
      reg.insert(draft);

      const results = reg.search(
        { status: ListingStatus.Published },
        ListingSort.Rating,
      );
      expect(results).toHaveLength(1);
      expect(results[0].name).toBe('Pub');
    });

    it('should search by device', () => {
      const desktop = makeListing('Desktop', LifeDomain.Finance, { type: 'Free' });
      desktop.supportedDevices = [DeviceType.Desktop];
      reg.insert(desktop);

      const mobile = makeListing('Mobile', LifeDomain.Finance, { type: 'Free' });
      mobile.supportedDevices = [DeviceType.Mobile];
      reg.insert(mobile);

      const results = reg.search(
        { device: DeviceType.Mobile },
        ListingSort.Rating,
      );
      expect(results).toHaveLength(1);
      expect(results[0].name).toBe('Mobile');
    });

    it('should search by publisher', () => {
      const pubId = newPublisherId();
      const listing = makeListing('MyAgent', LifeDomain.Finance, { type: 'Free' });
      (listing as any).publisher = pubId;
      reg.insert(listing);

      reg.insert(makeListing('Other', LifeDomain.Finance, { type: 'Free' }));

      const results = reg.search(
        { publisher: pubId },
        ListingSort.Rating,
      );
      expect(results).toHaveLength(1);
      expect(results[0].publisher).toBe(pubId);
    });

    it('should search by min rating', () => {
      const high = makeListing('High', LifeDomain.Finance, { type: 'Free' });
      high.rating = 4.5;
      reg.insert(high);

      const low = makeListing('Low', LifeDomain.Finance, { type: 'Free' });
      low.rating = 2.0;
      reg.insert(low);

      const results = reg.search(
        { minRating: 4.0 },
        ListingSort.Rating,
      );
      expect(results).toHaveLength(1);
      expect(results[0].name).toBe('High');
    });
  });

  describe('sorting', () => {
    it('should sort by rating descending', () => {
      const a = makeListing('A', LifeDomain.Finance, { type: 'Free' });
      a.rating = 3.0;
      reg.insert(a);

      const b = makeListing('B', LifeDomain.Finance, { type: 'Free' });
      b.rating = 5.0;
      reg.insert(b);

      const results = reg.search({}, ListingSort.Rating);
      expect(results[0].rating).toBe(5.0);
      expect(results[1].rating).toBe(3.0);
    });

    it('should sort by installs descending', () => {
      const a = makeListing('A', LifeDomain.Finance, { type: 'Free' });
      a.installCount = 50;
      reg.insert(a);

      const b = makeListing('B', LifeDomain.Finance, { type: 'Free' });
      b.installCount = 200;
      reg.insert(b);

      const results = reg.search({}, ListingSort.Installs);
      expect(results[0].installCount).toBe(200);
    });

    it('should sort by price low to high', () => {
      reg.insert(
        makeListing('Exp', LifeDomain.Finance, { type: 'OneTime', cents: 5000 }),
      );
      reg.insert(
        makeListing('Cheap', LifeDomain.Finance, { type: 'OneTime', cents: 100 }),
      );

      const results = reg.search({}, ListingSort.PriceLow);
      expect(results[0].name).toBe('Cheap');
    });

    it('should sort by price high to low', () => {
      reg.insert(
        makeListing('Exp', LifeDomain.Finance, { type: 'OneTime', cents: 5000 }),
      );
      reg.insert(
        makeListing('Cheap', LifeDomain.Finance, { type: 'OneTime', cents: 100 }),
      );

      const results = reg.search({}, ListingSort.PriceHigh);
      expect(results[0].name).toBe('Exp');
    });
  });

  describe('rating aggregation', () => {
    it('should compute running average', () => {
      const listing = makeListing('Rated', LifeDomain.Pet, { type: 'Free' });
      addRating(listing, 4.0);
      addRating(listing, 5.0);
      expect(listing.rating).toBeCloseTo(4.5, 2);
      expect(listing.ratingCount).toBe(2);
    });

    it('should handle single rating', () => {
      const listing = makeListing('Single', LifeDomain.Pet, { type: 'Free' });
      addRating(listing, 3.0);
      expect(listing.rating).toBeCloseTo(3.0, 2);
      expect(listing.ratingCount).toBe(1);
    });
  });

  describe('enforce rating policy', () => {
    it('should suspend low-rated published agents', () => {
      const listing = makeListing('BadAgent', LifeDomain.Social, { type: 'Free' });
      listing.status = ListingStatus.Published;
      listing.rating = 1.5;
      listing.ratingCount = 60;
      const id = listing.id;
      reg.insert(listing);

      const suspended = reg.enforceRatingPolicy();
      expect(suspended).toHaveLength(1);
      expect(suspended[0]).toBe(id);
      expect(reg.get(id)!.status).toBe(ListingStatus.Suspended);
    });

    it('should not suspend agents with fewer than 50 ratings', () => {
      const listing = makeListing('NewBad', LifeDomain.Social, { type: 'Free' });
      listing.status = ListingStatus.Published;
      listing.rating = 1.0;
      listing.ratingCount = 10;
      reg.insert(listing);

      const suspended = reg.enforceRatingPolicy();
      expect(suspended).toHaveLength(0);
    });

    it('should not suspend draft agents', () => {
      const listing = makeListing('DraftBad', LifeDomain.Social, { type: 'Free' });
      listing.status = ListingStatus.Draft;
      listing.rating = 1.0;
      listing.ratingCount = 100;
      reg.insert(listing);

      const suspended = reg.enforceRatingPolicy();
      expect(suspended).toHaveLength(0);
    });
  });

  describe('update status', () => {
    it('should set published_at when transitioning to Published', () => {
      const listing = makeListing('PubTest', LifeDomain.Career, { type: 'Free' });
      const id = listing.id;
      reg.insert(listing);

      expect(reg.get(id)!.publishedAt).toBeNull();
      reg.updateStatus(id, ListingStatus.Published);
      expect(reg.get(id)!.publishedAt).not.toBeNull();
    });

    it('should not overwrite publishedAt on second call', () => {
      const listing = makeListing('PubTwice', LifeDomain.Career, { type: 'Free' });
      const id = listing.id;
      reg.insert(listing);

      reg.updateStatus(id, ListingStatus.Published);
      const firstPublished = reg.get(id)!.publishedAt;

      reg.updateStatus(id, ListingStatus.Published);
      expect(reg.get(id)!.publishedAt).toBe(firstPublished);
    });

    it('should throw for non-existent listing', () => {
      expect(() =>
        reg.updateStatus(crypto.randomUUID(), ListingStatus.Published),
      ).toThrow();
    });
  });

  describe('byPublisher', () => {
    it('should return all listings for a publisher', () => {
      const pubId = newPublisherId();
      const a = makeListing('A', LifeDomain.Finance, { type: 'Free' });
      (a as any).publisher = pubId;
      reg.insert(a);

      const b = makeListing('B', LifeDomain.Health, { type: 'Free' });
      (b as any).publisher = pubId;
      reg.insert(b);

      reg.insert(makeListing('Other', LifeDomain.Legal, { type: 'Free' }));

      const results = reg.byPublisher(pubId);
      expect(results).toHaveLength(2);
    });
  });
});
