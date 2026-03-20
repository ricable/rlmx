import { describe, it, expect } from 'vitest';
import { BudgetLedger, BillingError } from '../src/index.js';
import type { BudgetEntry, BudgetPolicy } from '../src/index.js';

function makeEntry(agentId: string, model: string, provider: string, cost: number): BudgetEntry {
  return {
    agentId,
    model,
    provider,
    tokensIn: 100,
    tokensOut: 50,
    costMicrocents: cost,
    timestamp: new Date().toISOString(),
  };
}

describe('BudgetLedger', () => {
  it('new ledger has zero spent for unknown agent', () => {
    const ledger = new BudgetLedger();
    expect(ledger.totalSpent('agent-1')).toBe(0);
  });

  it('record without CAS increments version', () => {
    const ledger = new BudgetLedger();
    const v = ledger.record(makeEntry('a1', 'gpt-4', 'openai', 5000));
    expect(v).toBe(1);
    expect(ledger.totalSpent('a1')).toBe(5000);
  });

  it('record with valid CAS succeeds', () => {
    const ledger = new BudgetLedger();
    const v1 = ledger.record(makeEntry('a1', 'gpt-4', 'openai', 1000), 0);
    expect(v1).toBe(1);
    const v2 = ledger.record(makeEntry('a1', 'gpt-4', 'openai', 2000), 1);
    expect(v2).toBe(2);
  });

  it('CAS rejects stale version', () => {
    const ledger = new BudgetLedger();
    ledger.record(makeEntry('a1', 'gpt-4', 'openai', 1000));
    expect(() => ledger.record(makeEntry('a1', 'gpt-4', 'openai', 2000), 0))
      .toThrow(BillingError);
  });

  it('CAS rejects future version', () => {
    const ledger = new BudgetLedger();
    expect(() => ledger.record(makeEntry('a1', 'gpt-4', 'openai', 1000), 99))
      .toThrow(BillingError);
  });

  it('no policy = unlimited (always allowed)', () => {
    const ledger = new BudgetLedger();
    const decision = ledger.check('a1');
    expect(decision.allowed).toBe(true);
    expect(decision.warning).toBeUndefined();
    expect(decision.remainingMicrocents).toBe(Number.MAX_SAFE_INTEGER);
  });

  it('hard limit blocks', () => {
    const ledger = new BudgetLedger();
    ledger.setPolicy('a1', { softLimitMicrocents: 5000, hardLimitMicrocents: 10000 });
    ledger.record(makeEntry('a1', 'gpt-4', 'openai', 10000));
    const decision = ledger.check('a1');
    expect(decision.allowed).toBe(false);
    expect(decision.warning).toBeDefined();
    expect(decision.remainingMicrocents).toBe(0);
  });

  it('soft limit warns but allows', () => {
    const ledger = new BudgetLedger();
    ledger.setPolicy('a1', { softLimitMicrocents: 5000, hardLimitMicrocents: 10000 });
    ledger.record(makeEntry('a1', 'gpt-4', 'openai', 7000));
    const decision = ledger.check('a1');
    expect(decision.allowed).toBe(true);
    expect(decision.warning).toBeDefined();
    expect(decision.remainingMicrocents).toBe(3000);
  });

  it('under soft limit has no warning', () => {
    const ledger = new BudgetLedger();
    ledger.setPolicy('a1', { softLimitMicrocents: 5000, hardLimitMicrocents: 10000 });
    ledger.record(makeEntry('a1', 'gpt-4', 'openai', 3000));
    const decision = ledger.check('a1');
    expect(decision.allowed).toBe(true);
    expect(decision.warning).toBeUndefined();
    expect(decision.remainingMicrocents).toBe(7000);
  });

  it('totalSpent sums multiple entries', () => {
    const ledger = new BudgetLedger();
    ledger.record(makeEntry('a1', 'gpt-4', 'openai', 1000));
    ledger.record(makeEntry('a1', 'claude-3', 'anthropic', 2000));
    ledger.record(makeEntry('a1', 'gpt-4', 'openai', 3000));
    expect(ledger.totalSpent('a1')).toBe(6000);
  });

  it('report breakdown by model/provider sorted by cost', () => {
    const ledger = new BudgetLedger();
    ledger.record(makeEntry('a1', 'gpt-4', 'openai', 1000));
    ledger.record(makeEntry('a1', 'gpt-4', 'openai', 2000));
    ledger.record(makeEntry('a1', 'claude-3', 'anthropic', 5000));
    const report = ledger.report('a1');
    expect(report.length).toBe(2);
    expect(report[0]).toEqual({ model: 'claude-3', provider: 'anthropic', totalCost: 5000 });
    expect(report[1]).toEqual({ model: 'gpt-4', provider: 'openai', totalCost: 3000 });
  });

  it('report empty for unknown agent', () => {
    const ledger = new BudgetLedger();
    expect(ledger.report('unknown')).toEqual([]);
  });

  it('entries returns all for agent', () => {
    const ledger = new BudgetLedger();
    ledger.record(makeEntry('a1', 'gpt-4', 'openai', 1000));
    ledger.record(makeEntry('a1', 'claude-3', 'anthropic', 2000));
    expect(ledger.entries('a1').length).toBe(2);
  });

  it('entries empty for unknown agent', () => {
    const ledger = new BudgetLedger();
    expect(ledger.entries('unknown').length).toBe(0);
  });

  it('setPolicy replaces existing policy', () => {
    const ledger = new BudgetLedger();
    ledger.setPolicy('a1', { softLimitMicrocents: 1000, hardLimitMicrocents: 2000 });
    ledger.setPolicy('a1', { softLimitMicrocents: 5000, hardLimitMicrocents: 10000, perCallMaxTokens: 4096 });
    const policy = ledger.getPolicy('a1');
    expect(policy?.softLimitMicrocents).toBe(5000);
    expect(policy?.hardLimitMicrocents).toBe(10000);
    expect(policy?.perCallMaxTokens).toBe(4096);
  });

  it('getPolicy returns undefined if unset', () => {
    const ledger = new BudgetLedger();
    expect(ledger.getPolicy('a1')).toBeUndefined();
  });

  it('multiple agents are independent', () => {
    const ledger = new BudgetLedger();
    ledger.record(makeEntry('a1', 'gpt-4', 'openai', 5000));
    ledger.record(makeEntry('a2', 'claude-3', 'anthropic', 3000));
    expect(ledger.totalSpent('a1')).toBe(5000);
    expect(ledger.totalSpent('a2')).toBe(3000);
  });

  it('exact soft limit triggers warning', () => {
    const ledger = new BudgetLedger();
    ledger.setPolicy('a1', { softLimitMicrocents: 5000, hardLimitMicrocents: 10000 });
    ledger.record(makeEntry('a1', 'gpt-4', 'openai', 5000));
    const decision = ledger.check('a1');
    expect(decision.allowed).toBe(true);
    expect(decision.warning).toBeDefined();
  });

  it('per-call token limit allows within budget', () => {
    const ledger = new BudgetLedger();
    ledger.setPolicy('a1', { softLimitMicrocents: 100000, hardLimitMicrocents: 200000, perCallMaxTokens: 1000 });
    const entry = makeEntry('a1', 'gpt-4', 'openai', 500);
    entry.tokensIn = 400;
    entry.tokensOut = 500; // total 900 <= 1000
    expect(() => ledger.record(entry)).not.toThrow();
  });

  it('per-call token limit rejects exceeding', () => {
    const ledger = new BudgetLedger();
    ledger.setPolicy('a1', { softLimitMicrocents: 100000, hardLimitMicrocents: 200000, perCallMaxTokens: 1000 });
    const entry = makeEntry('a1', 'gpt-4', 'openai', 500);
    entry.tokensIn = 600;
    entry.tokensOut = 500; // total 1100 > 1000
    expect(() => ledger.record(entry)).toThrow(BillingError);
  });

  it('no per-call token policy allows any tokens', () => {
    const ledger = new BudgetLedger();
    // No policy at all
    const entry = makeEntry('a1', 'gpt-4', 'openai', 500);
    entry.tokensIn = 999999;
    entry.tokensOut = 999999;
    expect(() => ledger.record(entry)).not.toThrow();
  });
});
