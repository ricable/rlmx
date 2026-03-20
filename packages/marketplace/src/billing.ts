/**
 * BillingEngine: revenue splitting, payout tracking, transaction records.
 * Mirrors rlmx-marketplace/src/billing.rs.
 *
 * Enforces the 70/30 developer/platform revenue split and $50 payout threshold.
 */

import type {
  AgentPrice,
  EarningTransaction,
  EarningsAccount,
  PayoutMethod,
  PayoutRecord,
  PublisherId,
  RevenueSplit,
} from './types.js';
import { priceAmountCents } from './types.js';
import { publisherNotFound } from './errors.js';

/** Default payout threshold in cents ($50). */
const DEFAULT_PAYOUT_THRESHOLD_CENTS = 5000;

/** Developer revenue share percentage. */
const DEVELOPER_SHARE_PCT = 70;

// ---------------------------------------------------------------------------
// EarningsAccount helpers
// ---------------------------------------------------------------------------

/** Create a new earnings account for a publisher. */
function createEarningsAccount(publisherId: PublisherId): EarningsAccount {
  return {
    publisherId,
    balanceCents: 0,
    lifetimeEarnedCents: 0,
    payoutMethod: null,
    payoutThresholdCents: DEFAULT_PAYOUT_THRESHOLD_CENTS,
    transactions: [],
  };
}

/** Credit earnings from an agent sale. */
function creditAccount(
  account: EarningsAccount,
  developerCents: number,
  transaction: EarningTransaction,
): void {
  account.balanceCents += developerCents;
  account.lifetimeEarnedCents += developerCents;
  account.transactions.push(transaction);
}

/** Returns true if balance meets payout threshold and method is set. */
function isEligibleForPayout(account: EarningsAccount): boolean {
  return (
    account.balanceCents >= account.payoutThresholdCents &&
    account.payoutMethod !== null
  );
}

/** Process a payout, zeroing the balance. Returns the payout amount. */
function processAccountPayout(account: EarningsAccount): number {
  const amount = account.balanceCents;
  account.balanceCents = 0;
  return amount;
}

// ---------------------------------------------------------------------------
// BillingEngine
// ---------------------------------------------------------------------------

/** The billing engine manages revenue splitting and payouts. */
export class BillingEngine {
  private readonly accounts = new Map<PublisherId, EarningsAccount>();
  private platformRevenueCents = 0;
  private readonly payouts: PayoutRecord[] = [];

  /** Ensure an earnings account exists for a publisher. */
  ensureAccount(publisherId: PublisherId): void {
    if (!this.accounts.has(publisherId)) {
      this.accounts.set(publisherId, createEarningsAccount(publisherId));
    }
  }

  /** Set payout method for a publisher. */
  setPayoutMethod(publisherId: PublisherId, method: PayoutMethod): void {
    const account = this.accounts.get(publisherId);
    if (!account) {
      throw publisherNotFound(publisherId);
    }
    account.payoutMethod = method;
  }

  /**
   * Process a sale with standard 70/30 revenue split.
   * Returns the transaction if the price is non-zero, null otherwise.
   */
  recordSale(
    publisherId: PublisherId,
    agentId: string,
    userId: string,
    price: AgentPrice,
  ): EarningTransaction | null {
    const gross = priceAmountCents(price);
    if (gross === 0) {
      return null;
    }

    const developerCents = Math.floor((gross * DEVELOPER_SHARE_PCT) / 100);
    const platformCents = gross - developerCents;

    const tx: EarningTransaction = {
      id: crypto.randomUUID(),
      agentId,
      userId,
      grossCents: gross,
      developerCents,
      platformCents,
      timestamp: new Date().toISOString(),
    };

    const account = this.accounts.get(publisherId);
    if (!account) {
      throw publisherNotFound(publisherId);
    }

    creditAccount(account, developerCents, tx);
    this.platformRevenueCents += platformCents;

    return tx;
  }

  /**
   * Process a sale with a custom revenue split (for agent packs).
   */
  recordSplitSale(
    creatorId: PublisherId,
    baseDevId: PublisherId | null,
    agentId: string,
    userId: string,
    grossCents: number,
    split: RevenueSplit,
  ): void {
    if (grossCents === 0) {
      return;
    }

    const creatorCents = Math.floor(
      (grossCents * split.creatorPct) / 100,
    );
    const platformCents = Math.floor(
      (grossCents * split.platformPct) / 100,
    );
    const baseDevCents = grossCents - creatorCents - platformCents;

    // Credit creator
    const creatorTx: EarningTransaction = {
      id: crypto.randomUUID(),
      agentId,
      userId,
      grossCents,
      developerCents: creatorCents,
      platformCents,
      timestamp: new Date().toISOString(),
    };

    const creatorAccount = this.accounts.get(creatorId);
    if (!creatorAccount) {
      throw publisherNotFound(creatorId);
    }
    creditAccount(creatorAccount, creatorCents, creatorTx);

    // Credit base developer if applicable
    if (baseDevId !== null && baseDevCents > 0) {
      const devTx: EarningTransaction = {
        id: crypto.randomUUID(),
        agentId,
        userId,
        grossCents,
        developerCents: baseDevCents,
        platformCents: 0,
        timestamp: new Date().toISOString(),
      };
      const devAccount = this.accounts.get(baseDevId);
      if (!devAccount) {
        throw publisherNotFound(baseDevId);
      }
      creditAccount(devAccount, baseDevCents, devTx);
    }

    this.platformRevenueCents += platformCents;
  }

  /** Run monthly payout cycle. Returns list of processed payouts. */
  runPayoutCycle(): PayoutRecord[] {
    const results: PayoutRecord[] = [];

    const eligible: PublisherId[] = [];
    for (const account of this.accounts.values()) {
      if (isEligibleForPayout(account)) {
        eligible.push(account.publisherId);
      }
    }

    for (const pid of eligible) {
      const account = this.accounts.get(pid);
      if (!account) continue;

      const amount = processAccountPayout(account);
      const record: PayoutRecord = {
        id: crypto.randomUUID(),
        publisherId: pid,
        amountCents: amount,
        processedAt: new Date().toISOString(),
      };
      results.push(record);
      this.payouts.push(record);
    }

    return results;
  }

  /** Get earnings account for a publisher. */
  getAccount(publisherId: PublisherId): EarningsAccount | undefined {
    return this.accounts.get(publisherId);
  }

  /** Total platform revenue collected. */
  platformRevenue(): number {
    return this.platformRevenueCents;
  }

  /** All payout records. */
  payoutHistory(): readonly PayoutRecord[] {
    return this.payouts;
  }
}
