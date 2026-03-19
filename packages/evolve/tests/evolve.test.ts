import { describe, it, expect, beforeEach } from 'vitest';
import {
  canTransition,
  EvolveError,
  LifecycleManager,
  ScoreEngine,
  FeedbackEngine,
} from '../src/index.js';
import type {
  FunctionStatus,
  ScoreResult,
  FeedbackDecision,
} from '../src/index.js';

// ---------------------------------------------------------------------------
// canTransition
// ---------------------------------------------------------------------------

describe('canTransition', () => {
  it('allows draft -> staging', () => {
    expect(canTransition('draft', 'staging')).toBe(true);
  });

  it('allows staging -> production', () => {
    expect(canTransition('staging', 'production')).toBe(true);
  });

  it('allows production -> deprecated', () => {
    expect(canTransition('production', 'deprecated')).toBe(true);
  });

  it('allows any -> killed', () => {
    const statuses: FunctionStatus[] = [
      'draft',
      'staging',
      'production',
      'deprecated',
    ];
    for (const s of statuses) {
      expect(canTransition(s, 'killed')).toBe(true);
    }
  });

  it('rejects draft -> production (skip)', () => {
    expect(canTransition('draft', 'production')).toBe(false);
  });

  it('rejects backward staging -> draft', () => {
    expect(canTransition('staging', 'draft')).toBe(false);
  });

  it('rejects backward production -> staging', () => {
    expect(canTransition('production', 'staging')).toBe(false);
  });

  it('rejects killed -> anything', () => {
    for (const s of [
      'draft',
      'staging',
      'production',
      'deprecated',
    ] as FunctionStatus[]) {
      expect(canTransition('killed', s)).toBe(false);
    }
  });

  it('rejects deprecated -> production', () => {
    expect(canTransition('deprecated', 'production')).toBe(false);
  });
});

// ---------------------------------------------------------------------------
// EvolveError
// ---------------------------------------------------------------------------

describe('EvolveError', () => {
  it('has correct name and code', () => {
    const err = new EvolveError('test', 'TEST_CODE');
    expect(err.name).toBe('EvolveError');
    expect(err.code).toBe('TEST_CODE');
    expect(err.message).toBe('test');
  });

  it('is instanceof Error', () => {
    const err = new EvolveError('test', 'X');
    expect(err instanceof Error).toBe(true);
    expect(err instanceof EvolveError).toBe(true);
  });
});

// ---------------------------------------------------------------------------
// LifecycleManager
// ---------------------------------------------------------------------------

describe('LifecycleManager', () => {
  let mgr: LifecycleManager;

  beforeEach(() => {
    mgr = new LifecycleManager();
  });

  it('creates a function in draft status', () => {
    const id = mgr.create('test_fn', 'return 1;', 'return one');
    const fn = mgr.get(id);
    expect(fn).toBeDefined();
    expect(fn!.status).toBe('draft');
    expect(fn!.name).toBe('test_fn');
  });

  it('assigns version 1 to first function', () => {
    const id = mgr.create('fn', 'code', 'goal');
    expect(mgr.get(id)!.version).toBe(1);
  });

  it('increments version for same name', () => {
    const id1 = mgr.create('fn', 'v1', 'goal');
    const id2 = mgr.create('fn', 'v2', 'goal', id1);
    expect(mgr.get(id1)!.version).toBe(1);
    expect(mgr.get(id2)!.version).toBe(2);
  });

  it('different names have independent versions', () => {
    const a = mgr.create('alpha', 'code', 'goal');
    const b = mgr.create('beta', 'code', 'goal');
    expect(mgr.get(a)!.version).toBe(1);
    expect(mgr.get(b)!.version).toBe(1);
  });

  it('transitions draft -> staging', () => {
    const id = mgr.create('fn', 'code', 'goal');
    mgr.transition(id, 'staging');
    expect(mgr.get(id)!.status).toBe('staging');
  });

  it('transitions staging -> production', () => {
    const id = mgr.create('fn', 'code', 'goal');
    mgr.transition(id, 'staging');
    mgr.transition(id, 'production');
    expect(mgr.get(id)!.status).toBe('production');
  });

  it('transitions production -> deprecated', () => {
    const id = mgr.create('fn', 'code', 'goal');
    mgr.transition(id, 'staging');
    mgr.transition(id, 'production');
    mgr.transition(id, 'deprecated');
    expect(mgr.get(id)!.status).toBe('deprecated');
  });

  it('transitions any -> killed', () => {
    const id = mgr.create('fn', 'code', 'goal');
    mgr.transition(id, 'killed');
    expect(mgr.get(id)!.status).toBe('killed');
  });

  it('rejects invalid skip draft -> production', () => {
    const id = mgr.create('fn', 'code', 'goal');
    expect(() => mgr.transition(id, 'production')).toThrow(EvolveError);
  });

  it('rejects backward transition', () => {
    const id = mgr.create('fn', 'code', 'goal');
    mgr.transition(id, 'staging');
    expect(() => mgr.transition(id, 'draft')).toThrow('invalid transition');
  });

  it('rejects killed -> anything', () => {
    const id = mgr.create('fn', 'code', 'goal');
    mgr.transition(id, 'killed');
    expect(() => mgr.transition(id, 'draft')).toThrow(EvolveError);
  });

  it('throws for nonexistent function', () => {
    expect(() => mgr.transition('fake-id', 'staging')).toThrow(
      'function not found',
    );
  });

  it('get returns undefined for nonexistent', () => {
    expect(mgr.get('nonexistent')).toBeUndefined();
  });

  it('listByStatus filters correctly', () => {
    const a = mgr.create('a', 'code', 'goal');
    mgr.create('b', 'code', 'goal');
    mgr.transition(a, 'staging');

    expect(mgr.listByStatus('draft').length).toBe(1);
    expect(mgr.listByStatus('staging').length).toBe(1);
    expect(mgr.listByStatus('production').length).toBe(0);
  });

  it('listAll returns everything', () => {
    mgr.create('a', 'code', 'goal');
    mgr.create('b', 'code', 'goal');
    expect(mgr.listAll().length).toBe(2);
  });

  it('byName returns all versions', () => {
    const id1 = mgr.create('shared', 'v1', 'goal');
    mgr.create('shared', 'v2', 'goal', id1);
    mgr.create('other', 'v1', 'goal');
    expect(mgr.byName('shared').length).toBe(2);
    expect(mgr.byName('other').length).toBe(1);
    expect(mgr.byName('nonexistent').length).toBe(0);
  });

  it('stores parentId', () => {
    const parent = mgr.create('fn', 'v1', 'goal');
    const child = mgr.create('fn', 'v2', 'goal', parent);
    expect(mgr.get(child)!.parentId).toBe(parent);
    expect(mgr.get(parent)!.parentId).toBeNull();
  });
});

// ---------------------------------------------------------------------------
// ScoreEngine
// ---------------------------------------------------------------------------

describe('ScoreEngine', () => {
  describe('exactMatch', () => {
    it('returns 1.0 for equal numbers', () => {
      expect(ScoreEngine.exactMatch(42, 42)).toBe(1.0);
    });

    it('returns 0.0 for unequal numbers', () => {
      expect(ScoreEngine.exactMatch(42, 43)).toBe(0.0);
    });

    it('returns 1.0 for equal strings', () => {
      expect(ScoreEngine.exactMatch('hello', 'hello')).toBe(1.0);
    });

    it('returns 0.0 for different types', () => {
      expect(ScoreEngine.exactMatch(42, '42')).toBe(0.0);
    });

    it('handles nested objects', () => {
      expect(
        ScoreEngine.exactMatch({ a: [1, 2], b: 'c' }, { a: [1, 2], b: 'c' }),
      ).toBe(1.0);
    });

    it('detects nested mismatch', () => {
      expect(ScoreEngine.exactMatch({ a: [1, 2] }, { a: [1, 3] })).toBe(0.0);
    });

    it('handles null', () => {
      expect(ScoreEngine.exactMatch(null, null)).toBe(1.0);
    });
  });

  describe('semanticSimilarity', () => {
    it('returns 1.0 for identical strings', () => {
      expect(ScoreEngine.semanticSimilarity('hello world', 'hello world')).toBe(
        1.0,
      );
    });

    it('returns 0.0 for disjoint strings', () => {
      expect(ScoreEngine.semanticSimilarity('hello world', 'foo bar')).toBe(
        0.0,
      );
    });

    it('computes partial overlap correctly', () => {
      const sim = ScoreEngine.semanticSimilarity(
        'the quick brown fox',
        'the slow brown dog',
      );
      // intersection: {the, brown} = 2, union = 6
      expect(Math.abs(sim - 2 / 6)).toBeLessThan(1e-10);
    });

    it('returns 1.0 for both empty', () => {
      expect(ScoreEngine.semanticSimilarity('', '')).toBe(1.0);
    });

    it('returns 0.0 for one empty', () => {
      expect(ScoreEngine.semanticSimilarity('hello', '')).toBe(0.0);
    });
  });

  describe('computeOverall', () => {
    it('returns 1.0 for perfect inputs', () => {
      const r = ScoreEngine.computeOverall(1.0, 1.0, 0, 1000, 0, 10000);
      expect(Math.abs(r.overall - 1.0)).toBeLessThan(1e-10);
    });

    it('returns 0.0 for worst inputs', () => {
      const r = ScoreEngine.computeOverall(0.0, 0.0, 2000, 1000, 20000, 10000);
      expect(Math.abs(r.overall)).toBeLessThan(1e-10);
    });

    it('computes mixed score correctly', () => {
      const r = ScoreEngine.computeOverall(0.8, 0.6, 500, 1000, 3000, 10000);
      // 0.8*0.5 + 0.6*0.25 + 0.5*0.15 + 0.7*0.10 = 0.695
      expect(Math.abs(r.overall - 0.695)).toBeLessThan(1e-10);
    });

    it('clamps latency score when exceeding timeout', () => {
      const r = ScoreEngine.computeOverall(1.0, 1.0, 5000, 1000, 0, 10000);
      expect(r.latencyScore).toBe(0);
    });

    it('clamps cost score when exceeding budget', () => {
      const r = ScoreEngine.computeOverall(1.0, 1.0, 0, 1000, 50000, 10000);
      expect(r.costScore).toBe(0);
    });

    it('handles zero timeout', () => {
      const r = ScoreEngine.computeOverall(1.0, 1.0, 100, 0, 0, 10000);
      expect(r.latencyScore).toBe(0);
    });

    it('handles zero budget limit', () => {
      const r = ScoreEngine.computeOverall(1.0, 1.0, 0, 1000, 100, 0);
      expect(r.costScore).toBe(0);
    });

    it('stores all components', () => {
      const r = ScoreEngine.computeOverall(0.9, 0.8, 200, 1000, 1000, 5000);
      expect(r.correctness).toBe(0.9);
      expect(r.safety).toBe(0.8);
    });
  });
});

// ---------------------------------------------------------------------------
// FeedbackEngine
// ---------------------------------------------------------------------------

describe('FeedbackEngine', () => {
  let engine: FeedbackEngine;

  beforeEach(() => {
    engine = new FeedbackEngine();
  });

  function makeScore(correctness: number, overall: number): ScoreResult {
    return {
      overall,
      correctness,
      safety: 1.0,
      latencyScore: 1.0,
      costScore: 1.0,
    };
  }

  it('returns keep for no history', () => {
    expect(engine.review('unknown')).toEqual({ type: 'keep' });
  });

  it('returns keep for all good scores', () => {
    for (let i = 0; i < 5; i++) {
      engine.recordEval('fn1', makeScore(1.0, 0.9));
    }
    expect(engine.review('fn1')).toEqual({ type: 'keep' });
  });

  it('returns kill for 3+ low correctness in last 5', () => {
    engine.recordEval('fn1', makeScore(0.3, 0.3));
    engine.recordEval('fn1', makeScore(0.8, 0.8));
    engine.recordEval('fn1', makeScore(0.2, 0.2));
    engine.recordEval('fn1', makeScore(0.1, 0.1));
    engine.recordEval('fn1', makeScore(0.9, 0.9));
    const decision = engine.review('fn1');
    expect(decision.type).toBe('kill');
  });

  it('returns improve for low average overall', () => {
    for (let i = 0; i < 5; i++) {
      engine.recordEval('fn1', makeScore(0.5, 0.3));
    }
    const decision = engine.review('fn1');
    expect(decision.type).toBe('improve');
    if (decision.type === 'improve') {
      expect(decision.reason).toContain('0.3');
    }
  });

  it('only considers last 5 results', () => {
    // 10 terrible old scores
    for (let i = 0; i < 10; i++) {
      engine.recordEval('fn1', makeScore(0.0, 0.0));
    }
    // 5 great new scores
    for (let i = 0; i < 5; i++) {
      engine.recordEval('fn1', makeScore(1.0, 0.9));
    }
    expect(engine.review('fn1')).toEqual({ type: 'keep' });
  });

  it('kill takes priority over improve', () => {
    for (let i = 0; i < 4; i++) {
      engine.recordEval('fn1', makeScore(0.1, 0.1));
    }
    engine.recordEval('fn1', makeScore(0.8, 0.8));
    expect(engine.review('fn1').type).toBe('kill');
  });

  it('fewer than 5 evals works', () => {
    engine.recordEval('fn1', makeScore(1.0, 0.8));
    expect(engine.review('fn1')).toEqual({ type: 'keep' });
  });

  it('exactly at threshold is keep', () => {
    for (let i = 0; i < 5; i++) {
      engine.recordEval('fn1', makeScore(0.5, 0.5));
    }
    expect(engine.review('fn1')).toEqual({ type: 'keep' });
  });

  it('two low correctness is not kill', () => {
    engine.recordEval('fn1', makeScore(0.1, 0.8));
    engine.recordEval('fn1', makeScore(0.1, 0.8));
    engine.recordEval('fn1', makeScore(0.9, 0.8));
    engine.recordEval('fn1', makeScore(0.9, 0.8));
    engine.recordEval('fn1', makeScore(0.9, 0.8));
    expect(engine.review('fn1')).toEqual({ type: 'keep' });
  });

  it('lastNResults returns empty for unknown', () => {
    expect(engine.lastNResults('unknown', 5)).toEqual([]);
  });

  it('lastNResults returns most recent', () => {
    engine.recordEval('fn1', makeScore(0.1, 0.1));
    engine.recordEval('fn1', makeScore(0.5, 0.5));
    engine.recordEval('fn1', makeScore(0.9, 0.9));
    const last2 = engine.lastNResults('fn1', 2);
    expect(last2.length).toBe(2);
    expect(last2[0].correctness).toBe(0.5);
    expect(last2[1].correctness).toBe(0.9);
  });

  it('lastNResults returns fewer than n if not enough', () => {
    engine.recordEval('fn1', makeScore(0.7, 0.7));
    expect(engine.lastNResults('fn1', 10).length).toBe(1);
  });

  it('leaderboard is empty initially', () => {
    expect(engine.leaderboard()).toEqual([]);
  });

  it('leaderboard sorted descending', () => {
    engine.recordEval('low', makeScore(0.2, 0.2));
    engine.recordEval('high', makeScore(1.0, 0.95));
    engine.recordEval('mid', makeScore(0.5, 0.6));

    const board = engine.leaderboard();
    expect(board.length).toBe(3);
    expect(board[0].functionId).toBe('high');
    expect(board[1].functionId).toBe('mid');
    expect(board[2].functionId).toBe('low');
  });

  it('leaderboard averages multiple evals', () => {
    engine.recordEval('fn1', makeScore(1.0, 0.6));
    engine.recordEval('fn1', makeScore(1.0, 0.8));
    const board = engine.leaderboard();
    expect(board.length).toBe(1);
    expect(Math.abs(board[0].avgScore - 0.7)).toBeLessThan(1e-10);
  });
});
