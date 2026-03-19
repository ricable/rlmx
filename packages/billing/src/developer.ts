/**
 * Developer accounts: revenue sharing, payout tracking, marketplace publishing.
 * Maps to rlmx-billing/src/developer.rs.
 */

import { noPayoutMethod, payoutThresholdNotMet } from './errors.js';

/** Developer revenue share percentage (developer gets 70%). */
export const DEVELOPER_SHARE_PCT = 70;

/** Default payout threshold in cents ($50). */
export const DEFAULT_PAYOUT_THRESHOLD_CENTS = 5000;

/** Payout method for developer earnings. */
export type PayoutMethod =
  | { type: 'StripeConnect'; accountId: string }
  | { type: 'BankTransfer'; routing: string; account: string };

/** A record of a single agent sale. */
export interface SaleRecord {
  id: string;
  agentId: string;
  buyerId: string;
  grossCents: number;
  developerCents: number;
  platformCents: number;
  timestamp: string;
}

/** A payout record. */
export interface PayoutRecord {
  id: string;
  amountCents: number;
  processedAt: string;
}

/** A developer account with earnings tracking and marketplace publishing. */
export interface DeveloperAccount {
  id: string;
  publisherName: string;
  payoutMethod: PayoutMethod | null;
  balanceCents: number;
  lifetimeEarnedCents: number;
  payoutThresholdCents: number;
  publishedAgents: string[];
  sales: SaleRecord[];
  payouts: PayoutRecord[];
  createdAt: string;
}

let _nextDevId = 0;
function generateId(): string {
  return `dev-${Date.now()}-${++_nextDevId}-${Math.random().toString(36).slice(2, 8)}`;
}

/** Create a new developer account. */
export function createDeveloperAccount(
  publisherName: string,
): DeveloperAccount {
  return {
    id: generateId(),
    publisherName,
    payoutMethod: null,
    balanceCents: 0,
    lifetimeEarnedCents: 0,
    payoutThresholdCents: DEFAULT_PAYOUT_THRESHOLD_CENTS,
    publishedAgents: [],
    sales: [],
    payouts: [],
    createdAt: new Date().toISOString(),
  };
}

/** Set the payout method. Returns a new DeveloperAccount. */
export function setPayoutMethod(
  account: DeveloperAccount,
  method: PayoutMethod,
): DeveloperAccount {
  return { ...account, payoutMethod: method };
}

/**
 * Record a sale with the standard 70/30 revenue split.
 * Returns [updatedAccount, saleRecord].
 */
export function recordSale(
  account: DeveloperAccount,
  agentId: string,
  buyerId: string,
  grossCents: number,
): [DeveloperAccount, SaleRecord] {
  const developerCents = Math.floor(
    (grossCents * DEVELOPER_SHARE_PCT) / 100,
  );
  const platformCents = grossCents - developerCents;

  const record: SaleRecord = {
    id: generateId(),
    agentId,
    buyerId,
    grossCents,
    developerCents,
    platformCents,
    timestamp: new Date().toISOString(),
  };

  const updated: DeveloperAccount = {
    ...account,
    balanceCents: account.balanceCents + developerCents,
    lifetimeEarnedCents: account.lifetimeEarnedCents + developerCents,
    sales: [...account.sales, record],
  };

  return [updated, record];
}

/** Calculate the pending payout amount. */
export function calculatePayout(account: DeveloperAccount): number {
  return account.balanceCents;
}

/**
 * Request a payout. Returns [updatedAccount, payoutRecord] if eligible.
 * Throws BillingError if no payout method or threshold not met.
 */
export function requestPayout(
  account: DeveloperAccount,
): [DeveloperAccount, PayoutRecord] {
  if (account.payoutMethod === null) {
    throw noPayoutMethod();
  }
  if (account.balanceCents < account.payoutThresholdCents) {
    throw payoutThresholdNotMet(
      account.balanceCents,
      account.payoutThresholdCents,
    );
  }

  const amount = account.balanceCents;
  const record: PayoutRecord = {
    id: generateId(),
    amountCents: amount,
    processedAt: new Date().toISOString(),
  };

  const updated: DeveloperAccount = {
    ...account,
    balanceCents: 0,
    payouts: [...account.payouts, record],
  };

  return [updated, record];
}

/** Register a published agent. Idempotent. Returns a new DeveloperAccount. */
export function publishAgent(
  account: DeveloperAccount,
  agentId: string,
): DeveloperAccount {
  if (account.publishedAgents.includes(agentId)) {
    return account;
  }
  return {
    ...account,
    publishedAgents: [...account.publishedAgents, agentId],
  };
}

/** Total number of sales. */
export function totalSales(account: DeveloperAccount): number {
  return account.sales.length;
}
