import { describe, it, expect } from 'vitest';
import { LifeDomain } from '@aix/shared';
import { FederationCycle } from '../src/cycle.js';
import { PackageDistributor } from '../src/distribution.js';
import { createContribution } from '../src/contribution.js';
import type { AnonymizedPattern } from '../src/contribution.js';

function makePattern(quality = 0.85): AnonymizedPattern {
  return {
    id: crypto.randomUUID(),
    sanitizedEmbedding: new Array(8).fill(0.1),
    actionsTaken: ['action'],
    resultQuality: quality,
    emotionBucket: null,
    noisyUrgency: 0.5,
    noisySatisfaction: null,
    interactionModality: 'Voice',
    timestamp: new Date().toISOString(),
  };
}

function makeContribution(domain: LifeDomain, pseudonym: string) {
  return createContribution(pseudonym, domain, [makePattern()]);
}

describe('FederationCycle', () => {
  it('should start in Collecting status', () => {
    const cycle = FederationCycle.startCycle(1);
    expect(cycle.cycleNumber).toBe(1);
    expect(cycle.status).toBe('Collecting');
    expect(cycle.contributions).toHaveLength(0);
    expect(cycle.completedAt).toBeNull();
  });

  it('should accept contributions during Collecting', () => {
    const cycle = FederationCycle.startCycle(1, { minContributors: 2 });
    const contrib = makeContribution(LifeDomain.Finance, 'user-1');
    cycle.acceptContribution(contrib);
    expect(cycle.contributions).toHaveLength(1);
  });

  it('should reject contributions when not Collecting', () => {
    const cycle = FederationCycle.startCycle(1, { minContributors: 2 });
    // Force status change.
    cycle.status = 'Aggregating';
    const contrib = makeContribution(LifeDomain.Finance, 'user-1');
    expect(() => cycle.acceptContribution(contrib)).toThrow(
      'not in the expected status',
    );
  });

  it('should reject invalid contributions (empty pseudonym)', () => {
    const cycle = FederationCycle.startCycle(1, { minContributors: 2 });
    const contrib = createContribution('', LifeDomain.Finance, [makePattern()]);
    expect(() => cycle.acceptContribution(contrib)).toThrow('empty pseudonym');
  });

  describe('full cycle lifecycle', () => {
    it('should complete a full Collecting -> Aggregating -> Distributing -> Completed cycle', async () => {
      const cycle = FederationCycle.startCycle(1, { minContributors: 2 });

      // Collect contributions.
      for (let i = 0; i < 3; i++) {
        cycle.acceptContribution(
          makeContribution(LifeDomain.Finance, `user-${i}`),
        );
      }

      // Aggregate.
      const domains = cycle.aggregate();
      expect(domains).toBe(1);
      expect(cycle.status).toBe('Aggregating');
      expect(cycle.aggregatedModels.has(LifeDomain.Finance)).toBe(true);

      // Distribute.
      const pkgCount = await cycle.distribute();
      expect(pkgCount).toBe(1);
      expect(cycle.status).toBe('Distributing');

      // Complete.
      const dist = new PackageDistributor();
      cycle.complete(dist);
      expect(cycle.status).toBe('Completed');
      expect(cycle.completedAt).not.toBeNull();
      expect(dist.count).toBe(1);
    });
  });

  describe('status transition guards', () => {
    it('should reject aggregate when not Collecting', () => {
      const cycle = FederationCycle.startCycle(1);
      cycle.status = 'Completed';
      expect(() => cycle.aggregate()).toThrow('not in the expected status');
    });

    it('should reject distribute when not Aggregating', () => {
      const cycle = FederationCycle.startCycle(1);
      // Still in Collecting.
      expect(cycle.distribute()).rejects.toThrow('not in the expected status');
    });

    it('should reject complete when not Distributing', () => {
      const cycle = FederationCycle.startCycle(1);
      const dist = new PackageDistributor();
      expect(() => cycle.complete(dist)).toThrow('not in the expected status');
    });
  });

  describe('contributor counting', () => {
    it('should count unique contributors', () => {
      const cycle = FederationCycle.startCycle(1, { minContributors: 2 });
      cycle.acceptContribution(makeContribution(LifeDomain.Finance, 'alice'));
      cycle.acceptContribution(makeContribution(LifeDomain.Health, 'alice'));
      cycle.acceptContribution(makeContribution(LifeDomain.Finance, 'bob'));
      expect(cycle.uniqueContributorCount()).toBe(2);
    });

    it('should count contributions per domain', () => {
      const cycle = FederationCycle.startCycle(1, { minContributors: 2 });
      cycle.acceptContribution(makeContribution(LifeDomain.Finance, 'a'));
      cycle.acceptContribution(makeContribution(LifeDomain.Finance, 'b'));
      cycle.acceptContribution(makeContribution(LifeDomain.Health, 'c'));
      expect(cycle.domainContributionCount(LifeDomain.Finance)).toBe(2);
      expect(cycle.domainContributionCount(LifeDomain.Health)).toBe(1);
      expect(cycle.domainContributionCount(LifeDomain.Legal)).toBe(0);
    });

    it('should produce a contribution summary', () => {
      const cycle = FederationCycle.startCycle(1, { minContributors: 2 });
      cycle.acceptContribution(makeContribution(LifeDomain.Finance, 'a'));
      cycle.acceptContribution(makeContribution(LifeDomain.Finance, 'b'));
      cycle.acceptContribution(makeContribution(LifeDomain.Health, 'c'));
      const summary = cycle.contributionSummary();
      expect(summary.get(LifeDomain.Finance)).toBe(2);
      expect(summary.get(LifeDomain.Health)).toBe(1);
    });
  });

  describe('aggregation threshold enforcement in full cycle', () => {
    it('should skip domains that do not meet the 1000-user threshold', () => {
      const cycle = FederationCycle.startCycle(1); // Default: 1000 min.
      // Add only 10 contributions -- well below threshold.
      for (let i = 0; i < 10; i++) {
        cycle.acceptContribution(
          makeContribution(LifeDomain.Finance, `user-${i}`),
        );
      }
      const domains = cycle.aggregate();
      // No domains should meet the 1000-user threshold.
      expect(domains).toBe(0);
      expect(cycle.aggregatedModels.size).toBe(0);
    });
  });
});
