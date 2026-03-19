import type { FeedbackDecision, ScoreResult } from './types.js';

/**
 * Tracks evaluation history and produces feedback decisions.
 */
export class FeedbackEngine {
  private evalHistory = new Map<string, ScoreResult[]>();

  /** Record a score result for a function. */
  recordEval(functionId: string, scoreResult: ScoreResult): void {
    const history = this.evalHistory.get(functionId) ?? [];
    history.push(scoreResult);
    this.evalHistory.set(functionId, history);
  }

  /**
   * Review the last 5 results for a function and produce a decision.
   *
   * - KILL: 3+ of last 5 have correctness < 0.5
   * - IMPROVE: average overall of last 5 < 0.5
   * - KEEP: otherwise
   */
  review(functionId: string): FeedbackDecision {
    const history = this.evalHistory.get(functionId);
    if (!history || history.length === 0) {
      return { type: 'keep' };
    }

    const last5 = history.slice(-5);

    const lowCorrectnessCount = last5.filter(
      (s) => s.correctness < 0.5,
    ).length;

    if (lowCorrectnessCount >= 3) {
      return {
        type: 'kill',
        reason: `${lowCorrectnessCount} of last ${last5.length} evaluations had correctness < 0.5`,
      };
    }

    const avgOverall =
      last5.reduce((sum, s) => sum + s.overall, 0) / last5.length;

    if (avgOverall < 0.5) {
      return {
        type: 'improve',
        reason: `average overall score ${avgOverall.toFixed(3)} is below 0.5 threshold`,
      };
    }

    return { type: 'keep' };
  }

  /** Get the last N score results for a function. */
  lastNResults(functionId: string, n: number): ScoreResult[] {
    const history = this.evalHistory.get(functionId);
    if (!history) return [];
    return history.slice(-n);
  }

  /** Produce a leaderboard sorted by average overall score (descending). */
  leaderboard(): Array<{ functionId: string; avgScore: number }> {
    const entries: Array<{ functionId: string; avgScore: number }> = [];

    for (const [id, results] of this.evalHistory) {
      if (results.length === 0) continue;
      const avg =
        results.reduce((sum, s) => sum + s.overall, 0) / results.length;
      entries.push({ functionId: id, avgScore: avg });
    }

    entries.sort((a, b) => b.avgScore - a.avgScore);
    return entries;
  }
}
