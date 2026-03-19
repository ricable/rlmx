/**
 * Federated pattern aggregation: merges contributions from 1000+ users.
 *
 * Implements the cloud-side aggregation step (ADR-023 Step 3):
 * - Top-K pattern selection per domain, ranked by outcome quality.
 * - Aggregated LoRA delta computation (weighted average).
 * - Minimum 1000-user aggregation threshold enforced.
 *
 * Maps to rlmx-federation aggregator.rs.
 */

import type { LifeDomain } from '@aix/shared';
import type { AnonymizedPattern, Contribution, LoraDelta } from './contribution.js';
import {
  aggregationThresholdNotMet,
  noDomainContributions,
} from './errors.js';

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/** Configuration for the aggregation engine. */
export interface AggregatorConfig {
  /** Minimum number of contributors required before aggregation. */
  minContributors: number;
  /** Maximum number of patterns to keep per domain after aggregation. */
  topKPatterns: number;
}

/** Default aggregator config with 1000-user threshold. */
export const DEFAULT_AGGREGATOR_CONFIG: Readonly<AggregatorConfig> = {
  minContributors: 1000,
  topKPatterns: 10_000,
};

// ---------------------------------------------------------------------------
// AggregatedModel
// ---------------------------------------------------------------------------

/** Result of aggregating contributions for a single domain. */
export interface AggregatedModel {
  /** Unique identifier for this aggregated model. */
  id: string;
  /** The domain these patterns belong to. */
  domain: LifeDomain;
  /** Number of distinct contributors. */
  contributorCount: number;
  /** Top-K patterns ranked by quality. */
  patterns: AnonymizedPattern[];
  /** Aggregated LoRA delta (weighted average across contributors). */
  loraDelta: LoraDelta | null;
  /** Updated router weights (stub: domain-level confidence). */
  routerConfidence: number;
  /** When this model was created (ISO 8601). */
  createdAt: string;
}

// ---------------------------------------------------------------------------
// FederatedAggregator
// ---------------------------------------------------------------------------

/**
 * Aggregates anonymized pattern contributions across users.
 * Enforces the minimum contributor threshold (default: 1000).
 */
export class FederatedAggregator {
  private readonly config: AggregatorConfig;

  constructor(config: Partial<AggregatorConfig> = {}) {
    this.config = { ...DEFAULT_AGGREGATOR_CONFIG, ...config };
  }

  /** Return the minimum contributor threshold. */
  get minContributors(): number {
    return this.config.minContributors;
  }

  /**
   * Aggregate contributions for a given domain.
   *
   * Enforces the minimum contributor threshold before proceeding.
   * Returns the aggregated model with top-K patterns and merged LoRA delta.
   */
  aggregate(
    domain: LifeDomain,
    contributions: readonly Contribution[],
  ): AggregatedModel {
    // Filter contributions for this domain.
    const domainContributions = contributions.filter(
      (c) => c.domain === domain,
    );

    if (domainContributions.length === 0) {
      throw noDomainContributions(domain);
    }

    // Count distinct contributors.
    const uniqueContributors = new Set(
      domainContributions.map((c) => c.userPseudonym),
    );
    const contributorCount = uniqueContributors.size;

    if (contributorCount < this.config.minContributors) {
      throw aggregationThresholdNotMet(
        this.config.minContributors,
        contributorCount,
      );
    }

    // Select top-K patterns by quality.
    const topPatterns = this.selectTopPatterns(domainContributions);

    // Compute aggregated LoRA delta.
    const aggregatedLora = this.aggregateLoraDeltas(domainContributions);

    // Compute average router confidence from contribution qualities.
    const avgQuality =
      domainContributions.reduce((sum, c) => sum + c.aggregateQuality, 0) /
      domainContributions.length;

    return {
      id: crypto.randomUUID(),
      domain,
      contributorCount,
      patterns: topPatterns,
      loraDelta: aggregatedLora,
      routerConfidence: avgQuality,
      createdAt: new Date().toISOString(),
    };
  }

  /**
   * Aggregate all domains at once from a mixed set of contributions.
   *
   * Returns a map of domain to aggregated model. Domains that do not meet
   * the threshold are silently omitted.
   */
  aggregateAll(
    contributions: readonly Contribution[],
  ): Map<LifeDomain, AggregatedModel> {
    // Pre-group contributions by domain in a single pass to avoid N+1 filtering.
    const byDomain = new Map<LifeDomain, Contribution[]>();
    for (const c of contributions) {
      let arr = byDomain.get(c.domain);
      if (!arr) {
        arr = [];
        byDomain.set(c.domain, arr);
      }
      arr.push(c);
    }

    const results = new Map<LifeDomain, AggregatedModel>();
    for (const [domain, domainContribs] of byDomain) {
      try {
        const model = this.aggregateDirect(domain, domainContribs);
        results.set(domain, model);
      } catch {
        // Domain did not meet threshold; skip.
      }
    }
    return results;
  }

  /**
   * Internal: aggregate pre-filtered domain contributions (avoids re-filtering).
   */
  private aggregateDirect(
    domain: LifeDomain,
    domainContributions: readonly Contribution[],
  ): AggregatedModel {
    if (domainContributions.length === 0) {
      throw noDomainContributions(domain);
    }

    const uniqueContributors = new Set(
      domainContributions.map((c) => c.userPseudonym),
    );
    const contributorCount = uniqueContributors.size;

    if (contributorCount < this.config.minContributors) {
      throw aggregationThresholdNotMet(
        this.config.minContributors,
        contributorCount,
      );
    }

    const topPatterns = this.selectTopPatterns(domainContributions);
    const aggregatedLora = this.aggregateLoraDeltas(domainContributions);
    const avgQuality =
      domainContributions.reduce((sum, c) => sum + c.aggregateQuality, 0) /
      domainContributions.length;

    return {
      id: crypto.randomUUID(),
      domain,
      contributorCount,
      patterns: topPatterns,
      loraDelta: aggregatedLora,
      routerConfidence: avgQuality,
      createdAt: new Date().toISOString(),
    };
  }

  /**
   * Select the top-K patterns by result quality across all contributions.
   */
  private selectTopPatterns(
    contributions: readonly Contribution[],
  ): AnonymizedPattern[] {
    // Collect all patterns with their quality scores.
    const allPatterns: AnonymizedPattern[] = contributions.flatMap(
      (c) => c.patterns,
    );

    if (allPatterns.length <= this.config.topKPatterns) {
      return allPatterns;
    }

    // Sort by quality descending and take top-K.
    return allPatterns
      .sort((a, b) => b.resultQuality - a.resultQuality)
      .slice(0, this.config.topKPatterns);
  }

  /**
   * Compute a weighted average of LoRA deltas across contributions.
   * Weights are the contribution's aggregate quality score.
   */
  private aggregateLoraDeltas(
    contributions: readonly Contribution[],
  ): LoraDelta | null {
    const deltasWithWeights: Array<{ delta: LoraDelta; weight: number }> =
      contributions
        .filter((c): c is Contribution & { loraDelta: LoraDelta } =>
          c.loraDelta !== null,
        )
        .map((c) => ({ delta: c.loraDelta, weight: c.aggregateQuality }));

    if (deltasWithWeights.length === 0) {
      return null;
    }

    // Use the first delta as a template for shape.
    const template = deltasWithWeights[0].delta;
    const { rank, layerName } = template;
    const aLen = template.deltaA.length;
    const bLen = template.deltaB.length;

    // Only aggregate deltas with matching shapes.
    const compatible = deltasWithWeights.filter(
      ({ delta }) =>
        delta.rank === rank &&
        delta.deltaA.length === aLen &&
        delta.deltaB.length === bLen,
    );

    if (compatible.length === 0) {
      return null;
    }

    const totalWeight = compatible.reduce((sum, { weight }) => sum + weight, 0);
    if (totalWeight <= 0) {
      return null;
    }

    const avgA = new Array<number>(aLen).fill(0);
    const avgB = new Array<number>(bLen).fill(0);

    for (const { delta, weight } of compatible) {
      const w = weight / totalWeight;
      for (let i = 0; i < aLen; i++) {
        avgA[i] += delta.deltaA[i] * w;
      }
      for (let i = 0; i < bLen; i++) {
        avgB[i] += delta.deltaB[i] * w;
      }
    }

    return {
      layerName,
      deltaA: avgA,
      deltaB: avgB,
      rank,
      appliedAt: new Date().toISOString(),
    };
  }
}
