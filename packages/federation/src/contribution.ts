/**
 * User contributions: pseudonymous, domain-scoped pattern packages.
 *
 * A Contribution represents an anonymized set of patterns from a single user
 * for a single domain within a federation cycle. Contributions use pseudonymous
 * keys that are NOT linkable to the user's identity.
 *
 * Maps to rlmx-federation contribution.rs.
 */

import type { LifeDomain } from '@aix/shared';
import { contributionRejected } from './errors.js';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/** Interaction modality for a pattern. */
export type Modality = 'Voice' | 'Text' | 'Multimodal';

/** Emotion bucket — 5 discrete levels for privacy-safe emotion bucketing. */
export type EmotionBucket =
  | 'VeryNegative'
  | 'Negative'
  | 'Neutral'
  | 'Positive'
  | 'VeryPositive';

/** An anonymized pattern safe for federation. No raw user data. */
export interface AnonymizedPattern {
  /** Unique identifier for this pattern. */
  id: string;
  /** PII-stripped embedding vector. */
  sanitizedEmbedding: number[];
  /** Actions taken (must not contain PII). */
  actionsTaken: string[];
  /** Outcome quality score [0.0, 1.0]. */
  resultQuality: number;
  /** Bucketed emotion (5 discrete levels). */
  emotionBucket: EmotionBucket | null;
  /** Urgency with Laplace noise applied (epsilon=1.0). */
  noisyUrgency: number;
  /** Satisfaction with Laplace noise applied (epsilon=1.0). */
  noisySatisfaction: number | null;
  /** How the user interacted. */
  interactionModality: Modality;
  /** When this pattern was recorded. */
  timestamp: string;
}

/** LoRA delta for micro-adaptation. */
export interface LoraDelta {
  /** Layer name this delta applies to. */
  layerName: string;
  /** Low-rank matrix A. */
  deltaA: number[];
  /** Low-rank matrix B. */
  deltaB: number[];
  /** Rank of the decomposition. */
  rank: number;
  /** When this delta was computed. */
  appliedAt: string;
}

/**
 * A pseudonymous contribution from a single user for a single domain.
 *
 * The userPseudonym is a contribution-specific key that cannot be linked
 * back to the user's real identity. Each cycle generates a fresh pseudonym.
 */
export interface Contribution {
  /** Unique identifier for this contribution. */
  id: string;
  /** Pseudonymous key (not linkable to user identity). */
  userPseudonym: string;
  /** The life domain this contribution covers. */
  domain: LifeDomain;
  /** Anonymized patterns included in this contribution. */
  patterns: AnonymizedPattern[];
  /** Optional LoRA delta from the user's local adaptation. */
  loraDelta: LoraDelta | null;
  /** When the anonymization was performed (ISO 8601). */
  anonymizedAt: string;
  /** Aggregate quality score across all contributed patterns. */
  aggregateQuality: number;
  /** Coarse geographic bucket (e.g. "US-West", "EU-Central"). */
  regionBucket: string | null;
  /** Ed25519 signature hex over the serialized contribution body. */
  signature: string | null;
}

// ---------------------------------------------------------------------------
// Factory and helpers
// ---------------------------------------------------------------------------

/**
 * Create a new contribution from anonymized patterns.
 */
export function createContribution(
  userPseudonym: string,
  domain: LifeDomain,
  patterns: AnonymizedPattern[],
  loraDelta: LoraDelta | null = null,
): Contribution {
  const aggregateQuality =
    patterns.length === 0
      ? 0
      : patterns.reduce((sum, p) => sum + p.resultQuality, 0) /
        patterns.length;

  return {
    id: crypto.randomUUID(),
    userPseudonym,
    domain,
    patterns,
    loraDelta,
    anonymizedAt: new Date().toISOString(),
    aggregateQuality,
    regionBucket: null,
    signature: null,
  };
}

/**
 * Set the coarse geographic region bucket on a contribution.
 * Returns a new contribution (immutable pattern).
 */
export function withRegion(
  contribution: Contribution,
  region: string,
): Contribution {
  return { ...contribution, regionBucket: region };
}

/**
 * Set the cryptographic signature on a contribution.
 * Returns a new contribution (immutable pattern).
 */
export function withSignature(
  contribution: Contribution,
  signature: string,
): Contribution {
  return { ...contribution, signature };
}

/**
 * Validate that a contribution meets basic integrity requirements.
 * Throws FederationError if validation fails.
 */
export function validateContribution(contribution: Contribution): void {
  if (!contribution.userPseudonym || contribution.userPseudonym.length === 0) {
    throw contributionRejected('empty pseudonym');
  }
  if (contribution.patterns.length === 0) {
    throw contributionRejected('no patterns provided');
  }
}

/**
 * Generate a pseudonymous contribution key from a user seed and cycle number.
 *
 * Uses SHA-256 to derive a one-way pseudonym that cannot be reversed to
 * recover the original user identity. Returns a 32-character hex string
 * (128-bit pseudonym).
 */
export async function generatePseudonym(
  userSeed: Uint8Array,
  cycleNumber: number,
): Promise<string> {
  // Combine user seed with cycle number (little-endian 8 bytes).
  const cycleBytes = new Uint8Array(8);
  const view = new DataView(cycleBytes.buffer);
  // Write cycle number as uint64 little-endian (split into two uint32).
  view.setUint32(0, cycleNumber & 0xffffffff, true);
  view.setUint32(4, Math.floor(cycleNumber / 0x100000000) & 0xffffffff, true);

  const combined = new Uint8Array(userSeed.length + 8);
  combined.set(userSeed);
  combined.set(cycleBytes, userSeed.length);

  const hashBuffer = await crypto.subtle.digest('SHA-256', combined);
  const hashArray = new Uint8Array(hashBuffer);

  // Take first 16 bytes (128-bit pseudonym) and hex-encode.
  return Array.from(hashArray.slice(0, 16))
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}
