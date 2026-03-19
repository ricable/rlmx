/**
 * On-device anonymization for federated learning.
 *
 * Orchestrates the anonymization pipeline but delegates actual privacy
 * operations (Laplace noise injection, PII stripping, emotion bucketing)
 * to the Rust layer via napiFederatedAnonymize from @aix/core.
 *
 * Privacy invariants (ADR-023, non-negotiable):
 * - No raw user data ever leaves the device.
 * - Patterns anonymized BEFORE leaving device.
 * - Emotion valence bucketed into 5 levels.
 * - Laplace noise epsilon=1.0 on urgency/satisfaction.
 * - No speaker embeddings federated.
 * - Minimum 1000-user aggregation threshold.
 *
 * Maps to rlmx-federation anonymizer.rs.
 */

import type { LifeDomain } from '@aix/shared';
import type { NativeBindings, UnavailableResult } from '@aix/core';
import type {
  AnonymizedPattern,
  Contribution,
  EmotionBucket,
  LoraDelta,
} from './contribution.js';
import { createContribution } from './contribution.js';
import { anonymizationFailed, privacyViolation } from './errors.js';

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/** Configuration for the anonymization pipeline. */
export interface AnonymizationConfig {
  /** Differential privacy epsilon parameter. Must be 1.0 per ADR-023. */
  epsilon: number;
  /** Minimum number of contributors before patterns can be aggregated. */
  aggregationThreshold: number;
  /** Maximum patterns per contribution to limit information leakage. */
  maxPatternsPerContribution: number;
}

/** Default anonymization config enforcing privacy invariants. */
export const DEFAULT_ANONYMIZATION_CONFIG: Readonly<AnonymizationConfig> = {
  epsilon: 1.0,
  aggregationThreshold: 1000,
  maxPatternsPerContribution: 500,
};

// ---------------------------------------------------------------------------
// Voice-enriched pattern (pre-anonymization input)
// ---------------------------------------------------------------------------

/** A voice-enriched pattern before anonymization. Contains raw data. */
export interface VoiceEnrichedPattern {
  /** Unique identifier. */
  id: string;
  /** Raw query embedding (will be PII-stripped). */
  queryEmbedding: number[];
  /** Actions taken during the interaction. */
  actionsTaken: string[];
  /** Outcome quality [0.0, 1.0]. */
  resultQuality: number;
  /** Raw emotion valence (will be bucketed). */
  voiceTriggerEmotion: number | null;
  /** Raw urgency level (will have Laplace noise applied). */
  urgencyLevel: number;
  /** Interaction modality. */
  interactionModality: 'Voice' | 'Text' | 'Multimodal';
  /** Raw response satisfaction (will have Laplace noise applied). */
  responseSatisfaction: number | null;
  /** Timestamp (ISO 8601). */
  timestamp: string;
}

// ---------------------------------------------------------------------------
// FederationAnonymizer
// ---------------------------------------------------------------------------

/**
 * Federation-level anonymizer that orchestrates the anonymization pipeline.
 *
 * Delegates actual Laplace noise injection to the Rust native layer
 * via napiFederatedAnonymize. When native is unavailable, falls back
 * to a pure-TS implementation that applies the same privacy guarantees.
 */
export class FederationAnonymizer {
  private readonly config: AnonymizationConfig;
  private readonly nativeBindings: NativeBindings | null;

  constructor(
    config: Partial<AnonymizationConfig> = {},
    nativeBindings: NativeBindings | null = null,
  ) {
    this.config = { ...DEFAULT_ANONYMIZATION_CONFIG, ...config };
    this.nativeBindings = nativeBindings;
  }

  /** Return the aggregation threshold. */
  get aggregationThreshold(): number {
    return this.config.aggregationThreshold;
  }

  /** Return the epsilon value. */
  get epsilon(): number {
    return this.config.epsilon;
  }

  /**
   * Anonymize a batch of voice-enriched patterns into a contribution.
   *
   * This is the main entry point for on-device anonymization before federation.
   * Applies the full pipeline: strip PII, bucket emotion, add Laplace noise.
   */
  async anonymizePatterns(
    patterns: VoiceEnrichedPattern[],
    userPseudonym: string,
    domain: LifeDomain,
    loraDelta: LoraDelta | null = null,
  ): Promise<Contribution> {
    if (patterns.length === 0) {
      throw anonymizationFailed('no patterns to anonymize');
    }

    // Limit the number of patterns to prevent information leakage.
    const limit = Math.min(
      this.config.maxPatternsPerContribution,
      patterns.length,
    );
    const selected = patterns.slice(0, limit);

    // Attempt native anonymization first, fall back to TS implementation.
    const anonymized = await this.anonymizeViaBackend(selected);

    return createContribution(userPseudonym, domain, anonymized, loraDelta);
  }

  /**
   * Anonymize patterns using the native backend if available,
   * otherwise fall back to the pure-TS implementation.
   */
  private async anonymizeViaBackend(
    patterns: VoiceEnrichedPattern[],
  ): Promise<AnonymizedPattern[]> {
    if (this.nativeBindings) {
      try {
        const input = JSON.stringify(patterns);
        const result = await this.nativeBindings.napiFederatedAnonymize(input);

        // Check for degraded mode.
        if (
          typeof result === 'object' &&
          result !== null &&
          'status' in result &&
          (result as UnavailableResult).status === 'unavailable'
        ) {
          return this.anonymizeFallback(patterns);
        }

        return JSON.parse(result as string) as AnonymizedPattern[];
      } catch {
        // Native call failed, fall back to TS implementation.
        return this.anonymizeFallback(patterns);
      }
    }

    return this.anonymizeFallback(patterns);
  }

  /**
   * Pure-TS fallback anonymization that applies the same privacy guarantees.
   * Used when the native Rust layer is unavailable.
   */
  private anonymizeFallback(
    patterns: VoiceEnrichedPattern[],
  ): AnonymizedPattern[] {
    return patterns.map((p) => ({
      id: p.id,
      sanitizedEmbedding: FederationAnonymizer.stripPii(p.queryEmbedding),
      actionsTaken: p.actionsTaken,
      resultQuality: p.resultQuality,
      emotionBucket:
        p.voiceTriggerEmotion !== null
          ? FederationAnonymizer.bucketEmotion(p.voiceTriggerEmotion)
          : null,
      noisyUrgency: this.addLaplaceNoise(p.urgencyLevel),
      noisySatisfaction:
        p.responseSatisfaction !== null
          ? this.addLaplaceNoise(p.responseSatisfaction)
          : null,
      interactionModality: p.interactionModality,
      timestamp: p.timestamp,
    }));
  }

  /**
   * Strip PII from an embedding by zeroing the first 4 components
   * (which typically encode speaker-specific information).
   */
  static stripPii(embedding: number[]): number[] {
    const result = [...embedding];
    // Zero out first 4 dimensions (speaker identity components).
    for (let i = 0; i < Math.min(4, result.length); i++) {
      result[i] = 0;
    }
    return result;
  }

  /**
   * Bucket an emotion valence into 5 discrete levels.
   * Privacy-safe: collapses continuous values into coarse categories.
   */
  static bucketEmotion(valence: number): EmotionBucket {
    if (valence < -0.6) return 'VeryNegative';
    if (valence < -0.2) return 'Negative';
    if (valence <= 0.2) return 'Neutral';
    if (valence <= 0.6) return 'Positive';
    return 'VeryPositive';
  }

  /**
   * Add Laplace noise for differential privacy (epsilon from config).
   *
   * Laplace distribution with scale = 1/epsilon. Uses the inverse CDF
   * method: sample U ~ Uniform(-0.5, 0.5), then X = -b * sign(U) * ln(1 - 2|U|).
   */
  addLaplaceNoise(value: number): number {
    const b = 1.0 / this.config.epsilon;
    const u = Math.random() - 0.5;
    const noise = -b * Math.sign(u) * Math.log(1 - 2 * Math.abs(u));
    return value + noise;
  }

  /**
   * Check whether a text string likely contains PII.
   *
   * Simple heuristic: looks for patterns resembling email addresses,
   * phone numbers, or social security numbers.
   */
  static likelyContainsPii(text: string): boolean {
    // Email pattern.
    if (text.includes('@') && text.includes('.')) {
      return true;
    }
    // Phone-like sequences (9+ consecutive digits possibly with separators).
    const digitCount = text.split('').filter((c) => /\d/.test(c)).length;
    if (digitCount >= 9) {
      return true;
    }
    // SSN pattern (NNN-NN-NNNN).
    if (
      text.length === 11 &&
      /^\d{3}-\d{2}-\d{4}$/.test(text)
    ) {
      return true;
    }
    return false;
  }

  /**
   * Validate that an anonymized pattern does not violate privacy invariants.
   * Throws FederationError if PII is detected in action strings.
   */
  static validateAnonymized(pattern: AnonymizedPattern): void {
    for (const action of pattern.actionsTaken) {
      if (FederationAnonymizer.likelyContainsPii(action)) {
        throw privacyViolation(
          'PII detected in anonymized pattern actions',
        );
      }
    }
  }
}
