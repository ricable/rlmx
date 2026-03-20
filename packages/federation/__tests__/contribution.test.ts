import { describe, it, expect } from 'vitest';
import { LifeDomain } from '@aix/shared';
import {
  createContribution,
  validateContribution,
  withRegion,
  withSignature,
  generatePseudonym,
} from '../src/contribution.js';
import type { AnonymizedPattern } from '../src/contribution.js';

function makePattern(quality = 0.85): AnonymizedPattern {
  return {
    id: crypto.randomUUID(),
    sanitizedEmbedding: new Array(64).fill(0.1),
    actionsTaken: ['test_action'],
    resultQuality: quality,
    emotionBucket: null,
    noisyUrgency: 0.5,
    noisySatisfaction: 0.7,
    interactionModality: 'Voice',
    timestamp: new Date().toISOString(),
  };
}

describe('Contribution', () => {
  it('should create a contribution with correct fields', () => {
    const patterns = [makePattern(), makePattern()];
    const contrib = createContribution(
      'pseudo123',
      LifeDomain.Finance,
      patterns,
    );
    expect(contrib.patterns).toHaveLength(2);
    expect(contrib.aggregateQuality).toBeCloseTo(0.85, 2);
    expect(contrib.domain).toBe(LifeDomain.Finance);
    expect(contrib.userPseudonym).toBe('pseudo123');
    expect(contrib.loraDelta).toBeNull();
    expect(contrib.regionBucket).toBeNull();
    expect(contrib.signature).toBeNull();
  });

  it('should compute aggregate quality as mean of pattern qualities', () => {
    const patterns = [makePattern(0.6), makePattern(0.8)];
    const contrib = createContribution('p', LifeDomain.Health, patterns);
    expect(contrib.aggregateQuality).toBeCloseTo(0.7, 2);
  });

  it('should handle zero patterns with zero aggregate quality', () => {
    const contrib = createContribution('p', LifeDomain.Health, []);
    expect(contrib.aggregateQuality).toBe(0);
  });

  it('should set region via withRegion', () => {
    const contrib = createContribution('p', LifeDomain.Shopping, [makePattern()]);
    const updated = withRegion(contrib, 'US-West');
    expect(updated.regionBucket).toBe('US-West');
    // Original should be unchanged (immutable).
    expect(contrib.regionBucket).toBeNull();
  });

  it('should set signature via withSignature', () => {
    const contrib = createContribution('p', LifeDomain.Shopping, [makePattern()]);
    const updated = withSignature(contrib, 'deadbeef');
    expect(updated.signature).toBe('deadbeef');
    expect(contrib.signature).toBeNull();
  });
});

describe('validateContribution', () => {
  it('should reject empty pseudonym', () => {
    const contrib = createContribution('', LifeDomain.Health, [makePattern()]);
    expect(() => validateContribution(contrib)).toThrow('empty pseudonym');
  });

  it('should reject no patterns', () => {
    const contrib = createContribution('pseudo', LifeDomain.Health, []);
    expect(() => validateContribution(contrib)).toThrow('no patterns');
  });

  it('should accept valid contribution', () => {
    const contrib = createContribution('pseudo', LifeDomain.Health, [makePattern()]);
    expect(() => validateContribution(contrib)).not.toThrow();
  });
});

describe('generatePseudonym', () => {
  it('should be deterministic for same seed and cycle', async () => {
    const seed = new TextEncoder().encode('user-seed-42');
    const p1 = await generatePseudonym(seed, 1);
    const p2 = await generatePseudonym(seed, 1);
    expect(p1).toBe(p2);
  });

  it('should vary by cycle number', async () => {
    const seed = new TextEncoder().encode('user-seed-42');
    const p1 = await generatePseudonym(seed, 1);
    const p2 = await generatePseudonym(seed, 2);
    expect(p1).not.toBe(p2);
  });

  it('should vary by user seed', async () => {
    const seedA = new TextEncoder().encode('user-a');
    const seedB = new TextEncoder().encode('user-b');
    const p1 = await generatePseudonym(seedA, 1);
    const p2 = await generatePseudonym(seedB, 1);
    expect(p1).not.toBe(p2);
  });

  it('should produce a 32-character hex string', async () => {
    const seed = new TextEncoder().encode('test');
    const pseudonym = await generatePseudonym(seed, 1);
    expect(pseudonym).toHaveLength(32);
    expect(pseudonym).toMatch(/^[0-9a-f]{32}$/);
  });
});
