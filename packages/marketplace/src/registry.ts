/**
 * AgentRegistry: catalog of all marketplace agents with CRUD and search.
 * Mirrors rlmx-marketplace/src/registry.rs.
 */

import type { LifeDomain } from '@aix/shared';
import type {
  AgentListing,
  AgentPrice,
  DeviceType,
  ListingFilter,
  ModelTier,
  Permission,
  PublisherId,
} from './types.js';
import {
  ListingSort,
  ListingStatus,
  priceAmountCents,
} from './types.js';
import { listingNotFound } from './errors.js';
import type { MarketplaceError } from './errors.js';

// ---------------------------------------------------------------------------
// Listing creation helper
// ---------------------------------------------------------------------------

/** Create a new draft AgentListing. */
export function createDraftListing(params: {
  name: string;
  description: string;
  domain: LifeDomain;
  publisher: PublisherId;
  rvfHash: string;
  version: string;
  price: AgentPrice;
  permissions: Permission[];
  devices: DeviceType[];
  minModelTier: ModelTier;
  sizeBytes: number;
}): AgentListing {
  return {
    id: crypto.randomUUID(),
    name: params.name,
    description: params.description,
    domain: params.domain,
    publisher: params.publisher,
    rvfHash: params.rvfHash,
    version: params.version,
    price: params.price,
    rating: 0,
    ratingCount: 0,
    installCount: 0,
    permissionsRequired: params.permissions,
    supportedDevices: params.devices,
    minModelTier: params.minModelTier,
    sizeBytes: params.sizeBytes,
    status: ListingStatus.Draft,
    publishedAt: null,
    createdAt: new Date().toISOString(),
  };
}

/** Record a user rating and update the running average. */
export function addRating(listing: AgentListing, score: number): void {
  const total = listing.rating * listing.ratingCount + score;
  listing.ratingCount += 1;
  listing.rating = total / listing.ratingCount;
}

/** Increment install count. */
export function recordInstall(listing: AgentListing): void {
  listing.installCount += 1;
}

// ---------------------------------------------------------------------------
// AgentRegistry
// ---------------------------------------------------------------------------

/** The agent registry: in-memory catalog of all agent listings. */
export class AgentRegistry {
  private readonly listings = new Map<string, AgentListing>();

  /** Insert a new listing. Returns the listing ID. */
  insert(listing: AgentListing): string {
    this.listings.set(listing.id, listing);
    return listing.id;
  }

  /** Get a listing by ID (readonly). */
  get(id: string): AgentListing | undefined {
    return this.listings.get(id);
  }

  /** Get a mutable listing by ID. */
  getMut(id: string): AgentListing | undefined {
    return this.listings.get(id);
  }

  /** Remove a listing by ID. */
  remove(id: string): AgentListing | undefined {
    const listing = this.listings.get(id);
    if (listing) {
      this.listings.delete(id);
    }
    return listing;
  }

  /** Update the status of a listing. */
  updateStatus(id: string, status: ListingStatus): void {
    const listing = this.listings.get(id);
    if (!listing) {
      throw listingNotFound(id);
    }
    listing.status = status;
    if (status === ListingStatus.Published && listing.publishedAt === null) {
      listing.publishedAt = new Date().toISOString();
    }
  }

  /** Search listings with filters and sorting. */
  search(filter: ListingFilter, sort: ListingSort): AgentListing[] {
    let results = Array.from(this.listings.values()).filter((l) => {
      if (filter.domain !== undefined && l.domain !== filter.domain) {
        return false;
      }
      if (filter.keyword !== undefined) {
        const kw = filter.keyword.toLowerCase();
        if (
          !l.name.toLowerCase().includes(kw) &&
          !l.description.toLowerCase().includes(kw)
        ) {
          return false;
        }
      }
      if (filter.minRating !== undefined && l.rating < filter.minRating) {
        return false;
      }
      if (
        filter.maxPriceCents !== undefined &&
        priceAmountCents(l.price) > filter.maxPriceCents
      ) {
        return false;
      }
      if (filter.status !== undefined && l.status !== filter.status) {
        return false;
      }
      if (filter.publisher !== undefined && l.publisher !== filter.publisher) {
        return false;
      }
      if (
        filter.device !== undefined &&
        !l.supportedDevices.includes(filter.device)
      ) {
        return false;
      }
      return true;
    });

    switch (sort) {
      case ListingSort.Rating:
        results.sort((a, b) => b.rating - a.rating);
        break;
      case ListingSort.Installs:
        results.sort((a, b) => b.installCount - a.installCount);
        break;
      case ListingSort.Newest:
        results.sort(
          (a, b) =>
            new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime(),
        );
        break;
      case ListingSort.PriceLow:
        results.sort(
          (a, b) => priceAmountCents(a.price) - priceAmountCents(b.price),
        );
        break;
      case ListingSort.PriceHigh:
        results.sort(
          (a, b) => priceAmountCents(b.price) - priceAmountCents(a.price),
        );
        break;
    }

    return results;
  }

  /** Total number of listings. */
  count(): number {
    return this.listings.size;
  }

  /** All listings for a given publisher. */
  byPublisher(publisherId: PublisherId): AgentListing[] {
    return Array.from(this.listings.values()).filter(
      (l) => l.publisher === publisherId,
    );
  }

  /**
   * Auto-suspend agents with rating below 2.0 after 50+ ratings (invariant 6).
   * Returns IDs of suspended agents.
   */
  enforceRatingPolicy(): string[] {
    const suspended: string[] = [];
    for (const listing of this.listings.values()) {
      if (
        listing.status === ListingStatus.Published &&
        listing.ratingCount >= 50 &&
        listing.rating < 2.0
      ) {
        listing.status = ListingStatus.Suspended;
        suspended.push(listing.id);
      }
    }
    return suspended;
  }
}
