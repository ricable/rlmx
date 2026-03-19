import { describe, it, expect } from 'vitest';
import { LifeDomain } from '@aix/shared';
import { FederatedAggregator } from '../src/aggregator.js';
import { createContribution } from '../src/contribution.js';
import type { AnonymizedPattern, Contribution, LoraDelta } from '../src/contribution.js';

function makePattern(quality = 0.85): AnonymizedPattern {
  return {
    id: crypto.randomUUID(),
    sanitizedEmbedding: new Array(64).fill(0.1),
    actionsTaken: ['action'],
    resultQuality: quality,
    emotionBucket: null,
    noisyUrgency: 0.5,
    noisySatisfaction: null,
    interactionModality: 'Voice',
    timestamp: new Date().toISOString(),
  };
}

function makeContribution(
  domain: LifeDomain,
  pseudonym: string,
  quality = 0.85,
): Contribution {
  return createContribution(pseudonym, domain, [makePattern(quality)]);
}

function makeManyContributions(
  domain: LifeDomain,
  count: number,
): Contribution[] {
  return Array.from({ length: count }, (_, i) =>
    makeContribution(domain, `user-${i}`, 0.5 + i * 0.0001),
  );
}

describe('FederatedAggregator', () => {
  describe('aggregation threshold enforcement', () => {
    it('should reject aggregation when contributor count is below threshold (default 1000)', () => {
      const agg = new FederatedAggregator();
      const contribs = makeManyContributions(LifeDomain.Finance, 10);
      expect(() => agg.aggregate(LifeDomain.Finance, contribs)).toThrow(
        'Aggregation threshold not met',
      );
    });

    it('should include required and actual counts in the error', () => {
      const agg = new FederatedAggregator();
      const contribs = makeManyContributions(LifeDomain.Finance, 10);
      try {
        agg.aggregate(LifeDomain.Finance, contribs);
        expect.fail('should have thrown');
      } catch (e: unknown) {
        const err = e as { code: string; details: { required: number; actual: number } };
        expect(err.code).toBe('AggregationThresholdNotMet');
        expect(err.details?.required).toBe(1000);
        expect(err.details?.actual).toBe(10);
      }
    });

    it('should reject when duplicate pseudonyms do not meet unique threshold', () => {
      const agg = new FederatedAggregator({ minContributors: 3 });
      // 5 contributions but only 2 unique pseudonyms.
      const contribs = [
        makeContribution(LifeDomain.Finance, 'alice'),
        makeContribution(LifeDomain.Finance, 'alice'),
        makeContribution(LifeDomain.Finance, 'alice'),
        makeContribution(LifeDomain.Finance, 'bob'),
        makeContribution(LifeDomain.Finance, 'bob'),
      ];
      expect(() => agg.aggregate(LifeDomain.Finance, contribs)).toThrow(
        'Aggregation threshold not met',
      );
    });
  });

  describe('no domain contributions', () => {
    it('should throw when no contributions exist for the requested domain', () => {
      const agg = new FederatedAggregator();
      const contribs = makeManyContributions(LifeDomain.Finance, 10);
      expect(() => agg.aggregate(LifeDomain.Health, contribs)).toThrow(
        'No contributions for domain Health',
      );
    });
  });

  describe('successful aggregation', () => {
    it('should aggregate with a low threshold', () => {
      const agg = new FederatedAggregator({
        minContributors: 3,
        topKPatterns: 5,
      });
      const contribs = makeManyContributions(LifeDomain.Finance, 5);
      const model = agg.aggregate(LifeDomain.Finance, contribs);
      expect(model.domain).toBe(LifeDomain.Finance);
      expect(model.contributorCount).toBe(5);
      expect(model.patterns).toHaveLength(5);
    });

    it('should truncate to top-K patterns', () => {
      const agg = new FederatedAggregator({
        minContributors: 2,
        topKPatterns: 3,
      });
      const contribs = makeManyContributions(LifeDomain.Health, 5);
      const model = agg.aggregate(LifeDomain.Health, contribs);
      expect(model.patterns).toHaveLength(3);
      // Top-3 should be sorted by quality descending.
      expect(model.patterns[0].resultQuality).toBeGreaterThanOrEqual(
        model.patterns[1].resultQuality,
      );
    });
  });

  describe('LoRA delta aggregation', () => {
    it('should compute weighted average of compatible LoRA deltas', () => {
      const agg = new FederatedAggregator({
        minContributors: 2,
        topKPatterns: 100,
      });

      const delta1: LoraDelta = {
        layerName: 'layer_0',
        deltaA: [1.0, 2.0],
        deltaB: [0.5],
        rank: 1,
        appliedAt: new Date().toISOString(),
      };
      const delta2: LoraDelta = {
        layerName: 'layer_0',
        deltaA: [3.0, 4.0],
        deltaB: [1.5],
        rank: 1,
        appliedAt: new Date().toISOString(),
      };

      const c1 = createContribution(
        'user-a',
        LifeDomain.Finance,
        [makePattern(0.8)],
        delta1,
      );
      const c2 = createContribution(
        'user-b',
        LifeDomain.Finance,
        [makePattern(0.8)],
        delta2,
      );

      const model = agg.aggregate(LifeDomain.Finance, [c1, c2]);
      expect(model.loraDelta).not.toBeNull();
      // Equal quality weights -> simple average.
      expect(model.loraDelta!.deltaA[0]).toBeCloseTo(2.0, 1);
      expect(model.loraDelta!.deltaA[1]).toBeCloseTo(3.0, 1);
      expect(model.loraDelta!.deltaB[0]).toBeCloseTo(1.0, 1);
    });

    it('should return null when no LoRA deltas present', () => {
      const agg = new FederatedAggregator({
        minContributors: 2,
        topKPatterns: 100,
      });
      const contribs = makeManyContributions(LifeDomain.Finance, 3);
      const model = agg.aggregate(LifeDomain.Finance, contribs);
      expect(model.loraDelta).toBeNull();
    });
  });

  describe('aggregateAll', () => {
    it('should aggregate multiple domains and skip those below threshold', () => {
      const agg = new FederatedAggregator({
        minContributors: 2,
        topKPatterns: 100,
      });

      const contribs = [
        ...makeManyContributions(LifeDomain.Finance, 3),
        ...makeManyContributions(LifeDomain.Health, 2),
        // Only 1 Legal contribution -- won't meet threshold.
        makeContribution(LifeDomain.Legal, 'legal-user', 0.9),
      ];

      const results = agg.aggregateAll(contribs);
      expect(results.has(LifeDomain.Finance)).toBe(true);
      expect(results.has(LifeDomain.Health)).toBe(true);
      expect(results.has(LifeDomain.Legal)).toBe(false);
    });
  });
});
