/**
 * FeaturedEngine: ML-ranked agent recommendations and trending.
 * Mirrors rlmx-marketplace/src/featured.rs.
 */

import type { LifeDomain } from '@aix/shared';
import type { AgentListing, ScoredAgent } from './types.js';

// ---------------------------------------------------------------------------
// FeaturedEngine
// ---------------------------------------------------------------------------

/** The featured engine computes rankings and recommendations. */
export class FeaturedEngine {
  /** Manual featured agent IDs (curated by platform). */
  private readonly curated: string[] = [];

  /** Add an agent to the curated featured list. */
  addCurated(agentId: string): void {
    if (!this.curated.includes(agentId)) {
      this.curated.push(agentId);
    }
  }

  /** Remove an agent from the curated list. */
  removeCurated(agentId: string): void {
    const idx = this.curated.indexOf(agentId);
    if (idx !== -1) {
      this.curated.splice(idx, 1);
    }
  }

  /** Get the curated featured list. */
  curatedList(): readonly string[] {
    return this.curated;
  }

  /**
   * Compute a feature score for an agent listing.
   * Stub ML scoring: weighted combination of rating, installs, and recency.
   *
   * Weights: 40% rating, 35% installs (log-scaled), 25% recency.
   */
  static computeScore(listing: AgentListing): number {
    const ratingScore = listing.rating / 5.0;
    const installScore = Math.log1p(listing.installCount) / 10.0;

    let recencyScore = 0;
    if (listing.publishedAt !== null) {
      const daysOld = Math.max(
        1,
        (Date.now() - new Date(listing.publishedAt).getTime()) /
          (1000 * 60 * 60 * 24),
      );
      recencyScore = 1.0 / Math.sqrt(daysOld);
    }

    return ratingScore * 0.4 + installScore * 0.35 + recencyScore * 0.25;
  }

  /**
   * Score, sort descending, and truncate a set of listings.
   */
  private static rankBy(
    listings: readonly AgentListing[],
    scorer: (listing: AgentListing) => number,
    topN: number,
  ): ScoredAgent[] {
    const scored: ScoredAgent[] = listings.map((l) => ({
      agentId: l.id,
      score: scorer(l),
    }));

    scored.sort((a, b) => b.score - a.score);
    return scored.slice(0, topN);
  }

  /** Rank a set of listings by feature score, returning top N. */
  static rank(listings: readonly AgentListing[], topN: number): ScoredAgent[] {
    return FeaturedEngine.rankBy(
      listings,
      FeaturedEngine.computeScore,
      topN,
    );
  }

  /** Get recommended agents for a specific domain. */
  static recommendForDomain(
    listings: readonly AgentListing[],
    domain: LifeDomain,
    topN: number,
  ): ScoredAgent[] {
    const domainListings = listings.filter((l) => l.domain === domain);
    return FeaturedEngine.rank(domainListings, topN);
  }

  /**
   * Compute trending agents based on recent install velocity.
   * Stub: uses install count as proxy for velocity.
   */
  static trending(
    listings: readonly AgentListing[],
    topN: number,
  ): ScoredAgent[] {
    return FeaturedEngine.rankBy(
      listings,
      (l) => l.installCount,
      topN,
    );
  }
}
