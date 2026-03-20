import { describe, it, expect } from 'vitest';
import { reciprocalRankFusion } from '../src/merge/rrf.js';
import type { RagSearchResult } from '../src/types.js';

function makeResult(id: string, score: number, backend: string): RagSearchResult {
  return { id, content: `content-${id}`, score, source: `source-${id}`, backend };
}

describe('reciprocalRankFusion', () => {
  it('merges single backend results', () => {
    const sets = [{ backend: 'qmd', results: [makeResult('a', 0.9, 'qmd'), makeResult('b', 0.8, 'qmd')] }];
    const weights = new Map([['qmd', 1.0]]);
    const merged = reciprocalRankFusion(sets, weights);
    expect(merged).toHaveLength(2);
    expect(merged[0].id).toBe('a');
    expect(merged[1].id).toBe('b');
  });

  it('merges multiple backends with uniform weights', () => {
    const sets = [
      { backend: 'qmd', results: [makeResult('a', 0.9, 'qmd'), makeResult('b', 0.7, 'qmd')] },
      { backend: 'memory', results: [makeResult('c', 0.95, 'memory'), makeResult('a', 0.85, 'memory')] },
    ];
    const weights = new Map([['qmd', 1.0], ['memory', 1.0]]);
    const merged = reciprocalRankFusion(sets, weights);
    // 'a' appears in both backends so should have highest combined score
    expect(merged[0].id).toBe('a');
  });

  it('applies different weights', () => {
    const sets = [
      { backend: 'qmd', results: [makeResult('a', 0.9, 'qmd')] },
      { backend: 'memory', results: [makeResult('b', 0.95, 'memory')] },
    ];
    const weights = new Map([['qmd', 2.0], ['memory', 0.5]]);
    const merged = reciprocalRankFusion(sets, weights);
    // qmd has higher weight, so 'a' should rank above 'b'
    expect(merged[0].id).toBe('a');
  });

  it('handles empty result sets', () => {
    const sets = [{ backend: 'qmd', results: [] }];
    const weights = new Map([['qmd', 1.0]]);
    const merged = reciprocalRankFusion(sets, weights);
    expect(merged).toHaveLength(0);
  });

  it('handles overlapping results from multiple backends', () => {
    const sets = [
      { backend: 'a', results: [makeResult('x', 1.0, 'a')] },
      { backend: 'b', results: [makeResult('x', 1.0, 'b')] },
      { backend: 'c', results: [makeResult('x', 1.0, 'c')] },
    ];
    const weights = new Map([['a', 1.0], ['b', 1.0], ['c', 1.0]]);
    const merged = reciprocalRankFusion(sets, weights);
    expect(merged).toHaveLength(1);
    expect(merged[0].id).toBe('x');
    // Score should be sum of 3 RRF scores
    expect(merged[0].score).toBeGreaterThan(0);
  });

  it('uses custom k parameter', () => {
    const sets = [{ backend: 'qmd', results: [makeResult('a', 1.0, 'qmd')] }];
    const weights = new Map([['qmd', 1.0]]);
    const k10 = reciprocalRankFusion(sets, weights, 10);
    const k100 = reciprocalRankFusion(sets, weights, 100);
    // Higher k dampens scores more
    expect(k10[0].score).toBeGreaterThan(k100[0].score);
  });

  it('preserves result metadata', () => {
    const sets = [{
      backend: 'qmd',
      results: [{ id: 'a', content: 'hello', score: 1.0, source: 'test.rs', backend: 'qmd', metadata: { line: 42 } }],
    }];
    const weights = new Map([['qmd', 1.0]]);
    const merged = reciprocalRankFusion(sets, weights);
    expect(merged[0].metadata).toEqual({ line: 42 });
  });

  it('sorts by descending score', () => {
    const sets = [
      { backend: 'a', results: [makeResult('z', 0.1, 'a'), makeResult('y', 0.2, 'a'), makeResult('x', 0.3, 'a')] },
    ];
    const weights = new Map([['a', 1.0]]);
    const merged = reciprocalRankFusion(sets, weights);
    expect(merged[0].id).toBe('z');
    expect(merged[1].id).toBe('y');
    expect(merged[2].id).toBe('x');
  });

  it('handles missing weight defaulting to 1.0', () => {
    const sets = [{ backend: 'unknown', results: [makeResult('a', 1.0, 'unknown')] }];
    const weights = new Map<string, number>();
    const merged = reciprocalRankFusion(sets, weights);
    expect(merged).toHaveLength(1);
    expect(merged[0].score).toBeGreaterThan(0);
  });

  it('handles many backends', () => {
    const sets = Array.from({ length: 10 }, (_, i) => ({
      backend: `b${i}`,
      results: [makeResult(`r${i}`, 1.0 - i * 0.1, `b${i}`)],
    }));
    const weights = new Map(sets.map(s => [s.backend, 1.0]));
    const merged = reciprocalRankFusion(sets, weights);
    expect(merged).toHaveLength(10);
  });

  it('handles backend with no weight entry', () => {
    const sets = [
      { backend: 'qmd', results: [makeResult('a', 1.0, 'qmd')] },
      { backend: 'new', results: [makeResult('b', 1.0, 'new')] },
    ];
    const weights = new Map([['qmd', 1.0]]);
    const merged = reciprocalRankFusion(sets, weights);
    expect(merged).toHaveLength(2);
  });

  it('returns stable ordering for equal scores', () => {
    const sets = [
      { backend: 'a', results: [makeResult('x', 1.0, 'a')] },
      { backend: 'b', results: [makeResult('y', 1.0, 'b')] },
    ];
    const weights = new Map([['a', 1.0], ['b', 1.0]]);
    const merged1 = reciprocalRankFusion(sets, weights);
    const merged2 = reciprocalRankFusion(sets, weights);
    expect(merged1.map(r => r.id)).toEqual(merged2.map(r => r.id));
  });
});
