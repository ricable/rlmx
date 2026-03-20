import { describe, it, expect } from 'vitest';
import { LifeDomain } from '@aix/shared';
import {
  FederationAnonymizer,
  DEFAULT_ANONYMIZATION_CONFIG,
} from '../src/anonymizer.js';
import type { VoiceEnrichedPattern } from '../src/anonymizer.js';

function makeVoicePattern(
  emotion: number | null = 0.5,
  urgency = 0.3,
): VoiceEnrichedPattern {
  return {
    id: crypto.randomUUID(),
    queryEmbedding: new Array(64).fill(0.1),
    actionsTaken: ['test_action'],
    resultQuality: 0.85,
    voiceTriggerEmotion: emotion,
    urgencyLevel: urgency,
    interactionModality: 'Voice',
    responseSatisfaction: 0.7,
    timestamp: new Date().toISOString(),
  };
}

describe('FederationAnonymizer', () => {
  it('should use default config with epsilon=1.0 and threshold=1000', () => {
    const anon = new FederationAnonymizer();
    expect(anon.epsilon).toBe(1.0);
    expect(anon.aggregationThreshold).toBe(1000);
  });

  it('should anonymize patterns into a valid contribution', async () => {
    const anon = new FederationAnonymizer();
    const patterns = [makeVoicePattern(0.5, 0.3)];
    const contrib = await anon.anonymizePatterns(
      patterns,
      'pseudo',
      LifeDomain.Finance,
    );
    expect(contrib.patterns).toHaveLength(1);
    expect(contrib.domain).toBe(LifeDomain.Finance);
    expect(contrib.userPseudonym).toBe('pseudo');
  });

  it('should reject empty patterns', async () => {
    const anon = new FederationAnonymizer();
    await expect(
      anon.anonymizePatterns([], 'pseudo', LifeDomain.Health),
    ).rejects.toThrow('no patterns to anonymize');
  });

  it('should respect maxPatternsPerContribution limit', async () => {
    const anon = new FederationAnonymizer({
      maxPatternsPerContribution: 2,
    });
    const patterns = Array.from({ length: 5 }, () => makeVoicePattern());
    const contrib = await anon.anonymizePatterns(
      patterns,
      'pseudo',
      LifeDomain.Shopping,
    );
    expect(contrib.patterns).toHaveLength(2);
  });

  describe('privacy invariants', () => {
    it('should strip PII from embeddings (zero first 4 dimensions)', () => {
      const embedding = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
      const stripped = FederationAnonymizer.stripPii(embedding);
      expect(stripped[0]).toBe(0);
      expect(stripped[1]).toBe(0);
      expect(stripped[2]).toBe(0);
      expect(stripped[3]).toBe(0);
      expect(stripped[4]).toBe(5.0);
      expect(stripped[5]).toBe(6.0);
    });

    it('should bucket emotion valence into 5 discrete levels', () => {
      expect(FederationAnonymizer.bucketEmotion(-0.8)).toBe('VeryNegative');
      expect(FederationAnonymizer.bucketEmotion(-0.4)).toBe('Negative');
      expect(FederationAnonymizer.bucketEmotion(0.0)).toBe('Neutral');
      expect(FederationAnonymizer.bucketEmotion(0.4)).toBe('Positive');
      expect(FederationAnonymizer.bucketEmotion(0.8)).toBe('VeryPositive');
    });

    it('should add Laplace noise with epsilon=1.0', () => {
      const anon = new FederationAnonymizer({ epsilon: 1.0 });
      // Run many samples to verify noise is applied (value changes).
      const values = Array.from({ length: 100 }, () =>
        anon.addLaplaceNoise(0.5),
      );
      // Not all values should be exactly 0.5.
      const different = values.filter((v) => v !== 0.5);
      expect(different.length).toBeGreaterThan(0);
    });

    it('should apply Laplace noise to urgency and satisfaction during anonymization', async () => {
      const anon = new FederationAnonymizer();
      const patterns = [makeVoicePattern(null, 0.5)];
      // Run multiple times to check noise is applied.
      const results = await Promise.all(
        Array.from({ length: 20 }, () =>
          anon.anonymizePatterns(
            patterns,
            'pseudo',
            LifeDomain.Finance,
          ),
        ),
      );
      const urgencies = results.map((c) => c.patterns[0].noisyUrgency);
      const uniqueUrgencies = new Set(urgencies);
      // With Laplace noise, we should get different values.
      expect(uniqueUrgencies.size).toBeGreaterThan(1);
    });

    it('should bucket emotion in anonymized patterns', async () => {
      const anon = new FederationAnonymizer();
      const patterns = [makeVoicePattern(0.8, 0.5)];
      const contrib = await anon.anonymizePatterns(
        patterns,
        'pseudo',
        LifeDomain.Finance,
      );
      // 0.8 should map to VeryPositive.
      expect(contrib.patterns[0].emotionBucket).toBe('VeryPositive');
    });

    it('should leave emotion null when input has no emotion', async () => {
      const anon = new FederationAnonymizer();
      const patterns = [makeVoicePattern(null, 0.5)];
      const contrib = await anon.anonymizePatterns(
        patterns,
        'pseudo',
        LifeDomain.Finance,
      );
      expect(contrib.patterns[0].emotionBucket).toBeNull();
    });

    it('should zero first 4 embedding dimensions in anonymized output', async () => {
      const anon = new FederationAnonymizer();
      const pattern = makeVoicePattern();
      pattern.queryEmbedding = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
      const contrib = await anon.anonymizePatterns(
        [pattern],
        'pseudo',
        LifeDomain.Finance,
      );
      const emb = contrib.patterns[0].sanitizedEmbedding;
      expect(emb[0]).toBe(0);
      expect(emb[1]).toBe(0);
      expect(emb[2]).toBe(0);
      expect(emb[3]).toBe(0);
      expect(emb[4]).toBe(5.0);
    });
  });
});

describe('PII detection', () => {
  it('should detect email addresses', () => {
    expect(FederationAnonymizer.likelyContainsPii('user@example.com')).toBe(
      true,
    );
  });

  it('should detect phone numbers', () => {
    expect(FederationAnonymizer.likelyContainsPii('call 555-123-4567')).toBe(
      true,
    );
  });

  it('should detect SSN', () => {
    expect(FederationAnonymizer.likelyContainsPii('123-45-6789')).toBe(true);
  });

  it('should not flag clean text', () => {
    expect(FederationAnonymizer.likelyContainsPii('pay electric bill')).toBe(
      false,
    );
  });
});

describe('validateAnonymized', () => {
  it('should accept clean patterns', () => {
    expect(() =>
      FederationAnonymizer.validateAnonymized({
        id: crypto.randomUUID(),
        sanitizedEmbedding: [0.1],
        actionsTaken: ['bill_pay'],
        resultQuality: 0.8,
        emotionBucket: null,
        noisyUrgency: 0.5,
        noisySatisfaction: null,
        interactionModality: 'Text',
        timestamp: new Date().toISOString(),
      }),
    ).not.toThrow();
  });

  it('should reject patterns with PII in actions', () => {
    expect(() =>
      FederationAnonymizer.validateAnonymized({
        id: crypto.randomUUID(),
        sanitizedEmbedding: [0.1],
        actionsTaken: ['email john@example.com'],
        resultQuality: 0.8,
        emotionBucket: null,
        noisyUrgency: 0.5,
        noisySatisfaction: null,
        interactionModality: 'Text',
        timestamp: new Date().toISOString(),
      }),
    ).toThrow('PII detected');
  });
});
