import type { ScoreResult } from './types.js';

/**
 * Stateless scoring engine for evolved function executions.
 */
export class ScoreEngine {
  /**
   * Deep-equality scorer. Returns 1.0 if values match exactly, 0.0 otherwise.
   * Uses JSON serialization for deep comparison.
   */
  static exactMatch(actual: unknown, expected: unknown): number {
    return JSON.stringify(actual) === JSON.stringify(expected) ? 1.0 : 0.0;
  }

  /**
   * Jaccard word-set similarity between two strings.
   * Returns the size of the intersection divided by the size of the union.
   */
  static semanticSimilarity(a: string, b: string): number {
    const wordsA = new Set(a.split(/\s+/).filter(Boolean));
    const wordsB = new Set(b.split(/\s+/).filter(Boolean));

    if (wordsA.size === 0 && wordsB.size === 0) {
      return 1.0;
    }

    let intersection = 0;
    for (const w of wordsA) {
      if (wordsB.has(w)) intersection++;
    }

    const union = new Set([...wordsA, ...wordsB]).size;
    if (union === 0) return 0.0;

    return intersection / union;
  }

  /**
   * Compute the overall score using the weighted formula.
   *
   * overall = correctness * 0.50 + safety * 0.25 + latencyScore * 0.15 + costScore * 0.10
   *
   * Where:
   * - latencyScore = clamp(1.0 - actualMs / timeoutMs, 0, 1)
   * - costScore = clamp(1.0 - cost / budgetLimit, 0, 1)
   */
  static computeOverall(
    correctness: number,
    safety: number,
    latencyMs: number,
    timeoutMs: number,
    costMicrocents: number,
    budgetLimitMicrocents: number,
  ): ScoreResult {
    const latencyScore =
      timeoutMs === 0
        ? 0.0
        : Math.max(0, Math.min(1, 1.0 - latencyMs / timeoutMs));

    const costScore =
      budgetLimitMicrocents === 0
        ? 0.0
        : Math.max(
            0,
            Math.min(1, 1.0 - costMicrocents / budgetLimitMicrocents),
          );

    const overall =
      correctness * 0.5 + safety * 0.25 + latencyScore * 0.15 + costScore * 0.1;

    return {
      overall,
      correctness,
      safety,
      latencyScore,
      costScore,
    };
  }
}
