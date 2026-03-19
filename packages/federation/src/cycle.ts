/**
 * FederationCycle: the aggregate root for the Federated Learning bounded context.
 *
 * A FederationCycle represents one weekly federation round. It progresses
 * through four statuses: Collecting -> Aggregating -> Distributing -> Completed.
 *
 * The cycle owns:
 * - All contributions received during the collection window.
 * - Aggregated models per domain (once aggregation is complete).
 * - Distribution packages built from aggregated models.
 *
 * Maps to rlmx-federation cycle.rs (DDD-012 aggregate root).
 */

import type { LifeDomain } from '@aix/shared';
import type { Contribution } from './contribution.js';
import { validateContribution } from './contribution.js';
import type { AggregatedModel, AggregatorConfig } from './aggregator.js';
import { FederatedAggregator, DEFAULT_AGGREGATOR_CONFIG } from './aggregator.js';
import type { FederationPackage } from './distribution.js';
import {
  createPackageFromAggregated,
  PackageDistributor,
} from './distribution.js';
import { invalidCycleStatus } from './errors.js';

// ---------------------------------------------------------------------------
// CycleStatus
// ---------------------------------------------------------------------------

/** Status of a federation cycle. */
export type CycleStatus = 'Collecting' | 'Aggregating' | 'Distributing' | 'Completed';

// ---------------------------------------------------------------------------
// FederationCycle
// ---------------------------------------------------------------------------

/**
 * The aggregate root for the Federated Learning bounded context (DDD-012).
 *
 * Manages a single weekly federation round from contribution collection
 * through aggregation and distribution.
 */
export class FederationCycle {
  /** Unique identifier for this cycle. */
  readonly id: string;
  /** Monotonically increasing cycle number. */
  readonly cycleNumber: number;
  /** Current status of this cycle. */
  status: CycleStatus;
  /** All contributions received during the collection phase. */
  readonly contributions: Contribution[] = [];
  /** Aggregated models keyed by domain (populated during Aggregating phase). */
  readonly aggregatedModels: Map<LifeDomain, AggregatedModel> = new Map();
  /** Distribution packages (populated during Distributing phase). */
  readonly packages: FederationPackage[] = [];
  /** When this cycle started (ISO 8601). */
  readonly startedAt: string;
  /** When this cycle completed (ISO 8601), null if still in progress. */
  completedAt: string | null = null;

  private readonly aggregatorConfig: AggregatorConfig;

  private constructor(
    cycleNumber: number,
    aggregatorConfig: AggregatorConfig,
  ) {
    this.id = crypto.randomUUID();
    this.cycleNumber = cycleNumber;
    this.status = 'Collecting';
    this.startedAt = new Date().toISOString();
    this.aggregatorConfig = aggregatorConfig;
  }

  /** Start a new federation cycle. */
  static startCycle(
    cycleNumber: number,
    aggregatorConfig: Partial<AggregatorConfig> = {},
  ): FederationCycle {
    const config = { ...DEFAULT_AGGREGATOR_CONFIG, ...aggregatorConfig };
    return new FederationCycle(cycleNumber, config);
  }

  /**
   * Accept a contribution during the collection phase.
   * Throws if the cycle is not in Collecting status.
   */
  acceptContribution(contribution: Contribution): void {
    if (this.status !== 'Collecting') {
      throw invalidCycleStatus(this.id);
    }
    validateContribution(contribution);
    this.contributions.push(contribution);
  }

  /**
   * Transition from Collecting to Aggregating, then perform aggregation.
   * Returns the number of domains successfully aggregated.
   */
  aggregate(): number {
    if (this.status !== 'Collecting') {
      throw invalidCycleStatus(this.id);
    }

    this.status = 'Aggregating';

    const aggregator = new FederatedAggregator(this.aggregatorConfig);
    const results = aggregator.aggregateAll(this.contributions);

    for (const [domain, model] of results) {
      this.aggregatedModels.set(domain, model);
    }

    return this.aggregatedModels.size;
  }

  /**
   * Transition from Aggregating to Distributing; build distribution packages.
   * Returns the number of packages built.
   */
  async distribute(): Promise<number> {
    if (this.status !== 'Aggregating') {
      throw invalidCycleStatus(this.id);
    }

    this.status = 'Distributing';
    const version = `${this.cycleNumber}.0.0`;

    let built = 0;
    for (const model of this.aggregatedModels.values()) {
      try {
        const pkg = await createPackageFromAggregated(
          model,
          this.id,
          version,
        );
        this.packages.push(pkg);
        built++;
      } catch {
        // Failed to build package for this domain; skip.
      }
    }

    return built;
  }

  /**
   * Mark the cycle as completed and publish packages to the distributor.
   */
  complete(distributor: PackageDistributor): void {
    if (this.status !== 'Distributing') {
      throw invalidCycleStatus(this.id);
    }

    for (const pkg of this.packages) {
      distributor.publish(pkg);
    }

    this.status = 'Completed';
    this.completedAt = new Date().toISOString();
  }

  /**
   * Check if this cycle's collection window has expired (7 days).
   */
  collectionWindowExpired(): boolean {
    const windowMs = 7 * 24 * 60 * 60 * 1000; // 7 days
    return Date.now() - new Date(this.startedAt).getTime() > windowMs;
  }

  /** Number of unique contributors across all domains. */
  uniqueContributorCount(): number {
    const pseudonyms = new Set(this.contributions.map((c) => c.userPseudonym));
    return pseudonyms.size;
  }

  /** Number of contributions for a specific domain. */
  domainContributionCount(domain: LifeDomain): number {
    return this.contributions.filter((c) => c.domain === domain).length;
  }

  /** Get a summary of contributions per domain. */
  contributionSummary(): Map<LifeDomain, number> {
    const summary = new Map<LifeDomain, number>();
    for (const c of this.contributions) {
      summary.set(c.domain, (summary.get(c.domain) ?? 0) + 1);
    }
    return summary;
  }
}
