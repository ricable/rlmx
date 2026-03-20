/**
 * Per-agent budget ledger (ADR-032).
 *
 * Tracks per-agent LLM inference costs with soft/hard limits and CAS-based
 * optimistic concurrency control on the ledger.
 */

import { BillingError } from './errors.js';

/** A single cost entry recording one LLM inference call. */
export interface BudgetEntry {
  /** The agent that incurred this cost. */
  agentId: string;
  /** The model used (e.g. "claude-3-opus"). */
  model: string;
  /** The provider (e.g. "anthropic", "openai"). */
  provider: string;
  /** Input tokens consumed. */
  tokensIn: number;
  /** Output tokens produced. */
  tokensOut: number;
  /** Cost in microcents (1 cent = 1_000_000 microcents). */
  costMicrocents: number;
  /** ISO 8601 timestamp. */
  timestamp: string;
}

/** Budget policy for a single agent. */
export interface BudgetPolicy {
  /** Crossing this emits a warning but does not block. */
  softLimitMicrocents: number;
  /** Crossing this blocks the call entirely. */
  hardLimitMicrocents: number;
  /** Optional cap on tokens per single call. */
  perCallMaxTokens?: number;
}

/** Result of a budget check. */
export interface BudgetDecision {
  /** Whether the agent is allowed to proceed. */
  allowed: boolean;
  /** Optional warning (e.g. soft limit crossed). */
  warning?: string;
  /** Remaining budget before hard limit. */
  remainingMicrocents: number;
}

/** Report entry: breakdown by model and provider. */
export interface BudgetReportEntry {
  model: string;
  provider: string;
  totalCost: number;
}

/**
 * In-memory per-agent budget ledger with CAS versioning.
 * Mirrors the Rust `BudgetLedger` from `rlmx-billing/src/budget.rs`.
 */
export class BudgetLedger {
  private perAgent: Map<string, BudgetEntry[]> = new Map();
  private policies: Map<string, BudgetPolicy> = new Map();
  private versions: Map<string, number> = new Map();

  /** Set or replace the budget policy for an agent. */
  setPolicy(agentId: string, policy: BudgetPolicy): void {
    this.policies.set(agentId, policy);
  }

  /** Get the budget policy for an agent, if any. */
  getPolicy(agentId: string): BudgetPolicy | undefined {
    return this.policies.get(agentId);
  }

  /**
   * Record a budget entry with optional CAS check.
   *
   * If `expectedVersion` is provided and the current version for this
   * agent does not equal it, the write is rejected with a BillingError.
   *
   * Returns the new version number on success.
   */
  record(entry: BudgetEntry, expectedVersion?: number): number {
    const agentId = entry.agentId;
    const currentVersion = this.versions.get(agentId) ?? 0;

    if (expectedVersion !== undefined && expectedVersion !== currentVersion) {
      throw new BillingError(
        `CAS conflict: expected version ${expectedVersion}, found ${currentVersion}`,
        'INTERNAL',
        { expectedVersion, currentVersion },
      );
    }

    // ADR-032: enforce per-call token limit
    const policy = this.policies.get(agentId);
    if (policy?.perCallMaxTokens !== undefined) {
      const totalTokens = entry.tokensIn + entry.tokensOut;
      if (totalTokens > policy.perCallMaxTokens) {
        throw new BillingError(
          `usage quota exceeded: per_call_tokens (${totalTokens}/${policy.perCallMaxTokens})`,
          'QUOTA_EXCEEDED',
          { resource: 'per_call_tokens', used: totalTokens, limit: policy.perCallMaxTokens },
        );
      }
    }

    const newVersion = currentVersion + 1;
    this.versions.set(agentId, newVersion);

    const entries = this.perAgent.get(agentId);
    if (entries) {
      entries.push(entry);
    } else {
      this.perAgent.set(agentId, [entry]);
    }

    return newVersion;
  }

  /** Check whether an agent is within budget. */
  check(agentId: string): BudgetDecision {
    const spent = this.totalSpent(agentId);
    const policy = this.policies.get(agentId);

    if (!policy) {
      return {
        allowed: true,
        remainingMicrocents: Number.MAX_SAFE_INTEGER,
      };
    }

    if (spent >= policy.hardLimitMicrocents) {
      return {
        allowed: false,
        warning: `hard limit exceeded: spent ${spent} >= limit ${policy.hardLimitMicrocents}`,
        remainingMicrocents: 0,
      };
    }

    const remaining = policy.hardLimitMicrocents - spent;

    if (spent >= policy.softLimitMicrocents) {
      return {
        allowed: true,
        warning: `soft limit exceeded: spent ${spent} >= limit ${policy.softLimitMicrocents}`,
        remainingMicrocents: remaining,
      };
    }

    return {
      allowed: true,
      remainingMicrocents: remaining,
    };
  }

  /** Total cost in microcents for an agent. */
  totalSpent(agentId: string): number {
    const entries = this.perAgent.get(agentId);
    if (!entries) return 0;
    return entries.reduce((sum, e) => sum + e.costMicrocents, 0);
  }

  /** Breakdown of costs by model and provider, sorted by cost descending. */
  report(agentId: string): BudgetReportEntry[] {
    const entries = this.perAgent.get(agentId);
    if (!entries) return [];

    const breakdown = new Map<string, number>();
    for (const entry of entries) {
      const key = `${entry.model}\0${entry.provider}`;
      breakdown.set(key, (breakdown.get(key) ?? 0) + entry.costMicrocents);
    }

    const result: BudgetReportEntry[] = [];
    for (const [key, totalCost] of breakdown) {
      const [model, provider] = key.split('\0');
      result.push({ model, provider, totalCost });
    }

    result.sort((a, b) => b.totalCost - a.totalCost);
    return result;
  }

  /** Get all entries for an agent. */
  entries(agentId: string): readonly BudgetEntry[] {
    return this.perAgent.get(agentId) ?? [];
  }
}
