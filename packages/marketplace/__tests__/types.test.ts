/**
 * Tests for marketplace types and helper functions.
 * Mirrors rlmx-marketplace/src/domain.rs tests.
 */

import { describe, it, expect } from 'vitest';
import {
  createPermission,
  isSensitivePermission,
  priceAmountCents,
  standardSplit,
  celebritySplit,
  isValidSplit,
  createAgentPack,
  newPublisherId,
  severityGte,
  Severity,
} from '../src/types.js';
import type { AgentPrice, RevenueSplit } from '../src/types.js';

describe('AgentPrice', () => {
  it('should return 0 for Free', () => {
    expect(priceAmountCents({ type: 'Free' })).toBe(0);
  });

  it('should return cents for OneTime', () => {
    expect(priceAmountCents({ type: 'OneTime', cents: 999 })).toBe(999);
  });

  it('should return cents for Monthly', () => {
    expect(priceAmountCents({ type: 'Monthly', cents: 499 })).toBe(499);
  });
});

describe('RevenueSplit', () => {
  it('should validate standard split (70/30)', () => {
    const split = standardSplit();
    expect(split.creatorPct).toBe(70);
    expect(split.platformPct).toBe(30);
    expect(split.baseDevPct).toBe(0);
    expect(isValidSplit(split)).toBe(true);
  });

  it('should validate celebrity split (50/30/20)', () => {
    const split = celebritySplit();
    expect(split.creatorPct).toBe(50);
    expect(split.platformPct).toBe(30);
    expect(split.baseDevPct).toBe(20);
    expect(isValidSplit(split)).toBe(true);
  });

  it('should reject invalid split', () => {
    const bad: RevenueSplit = {
      creatorPct: 50,
      platformPct: 50,
      baseDevPct: 50,
    };
    expect(isValidSplit(bad)).toBe(false);
  });

  it('should reject split summing to less than 100', () => {
    const bad: RevenueSplit = {
      creatorPct: 30,
      platformPct: 30,
      baseDevPct: 10,
    };
    expect(isValidSplit(bad)).toBe(false);
  });
});

describe('Permission', () => {
  it('should identify health_data as sensitive', () => {
    expect(isSensitivePermission(createPermission('health_data'))).toBe(true);
  });

  it('should identify finance_data as sensitive', () => {
    expect(isSensitivePermission(createPermission('finance_data'))).toBe(true);
  });

  it('should identify legal_data as sensitive', () => {
    expect(isSensitivePermission(createPermission('legal_data'))).toBe(true);
  });

  it('should not flag vec_search as sensitive', () => {
    expect(isSensitivePermission(createPermission('vec_search'))).toBe(false);
  });

  it('should not flag graph_query as sensitive', () => {
    expect(isSensitivePermission(createPermission('graph_query'))).toBe(false);
  });
});

describe('AgentPack', () => {
  it('should default to celebrity split', () => {
    const pack = createAgentPack(
      'CoolPack',
      newPublisherId(),
      [crypto.randomUUID(), crypto.randomUUID()],
      { type: 'OneTime', cents: 1999 },
      'A curated pack',
    );
    expect(isValidSplit(pack.revenueSplit)).toBe(true);
    expect(pack.revenueSplit.creatorPct).toBe(50);
    expect(pack.revenueSplit.platformPct).toBe(30);
    expect(pack.revenueSplit.baseDevPct).toBe(20);
    expect(pack.agents).toHaveLength(2);
  });

  it('should accept custom split', () => {
    const split: RevenueSplit = {
      creatorPct: 60,
      platformPct: 30,
      baseDevPct: 10,
    };
    const pack = createAgentPack(
      'CustomPack',
      newPublisherId(),
      [crypto.randomUUID()],
      { type: 'Monthly', cents: 499 },
      'Custom split pack',
      split,
    );
    expect(isValidSplit(pack.revenueSplit)).toBe(true);
    expect(pack.revenueSplit.creatorPct).toBe(60);
  });
});

describe('PublisherId', () => {
  it('should generate unique IDs', () => {
    const a = newPublisherId();
    const b = newPublisherId();
    expect(a).not.toBe(b);
  });
});

describe('Severity', () => {
  it('should compare severity levels', () => {
    expect(severityGte(Severity.Critical, Severity.High)).toBe(true);
    expect(severityGte(Severity.High, Severity.High)).toBe(true);
    expect(severityGte(Severity.Medium, Severity.High)).toBe(false);
    expect(severityGte(Severity.Low, Severity.Critical)).toBe(false);
  });
});
