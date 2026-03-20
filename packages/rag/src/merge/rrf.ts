import type { RagSearchResult } from '../types.js';

/**
 * Reciprocal Rank Fusion: merges results from multiple backends.
 * score = sum(weight_i * 1/(k + rank_i))
 */
export function reciprocalRankFusion(
  resultSets: Array<{ backend: string; results: RagSearchResult[] }>,
  weights: Map<string, number>,
  k: number = 60,
): RagSearchResult[] {
  const scoreMap = new Map<string, { score: number; result: RagSearchResult }>();

  for (const { backend, results } of resultSets) {
    const w = weights.get(backend) ?? 1.0;
    for (let rank = 0; rank < results.length; rank++) {
      const r = results[rank];
      const rrfScore = w * (1.0 / (k + rank + 1));
      const existing = scoreMap.get(r.id);
      if (existing) {
        existing.score += rrfScore;
      } else {
        scoreMap.set(r.id, { score: rrfScore, result: r });
      }
    }
  }

  return [...scoreMap.values()]
    .sort((a, b) => b.score - a.score)
    .map(({ score, result }) => ({ ...result, score }));
}
