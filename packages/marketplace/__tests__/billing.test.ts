/**
 * Tests for the BillingEngine.
 * Mirrors rlmx-marketplace/src/billing.rs tests.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { BillingEngine } from '../src/billing.js';
import { newPublisherId, celebritySplit } from '../src/types.js';
import type { AgentPrice, PublisherId } from '../src/types.js';
import { MarketplaceError } from '../src/errors.js';

describe('BillingEngine', () => {
  let engine: BillingEngine;

  beforeEach(() => {
    engine = new BillingEngine();
  });

  describe('standard revenue split (70/30)', () => {
    it('should split revenue 70/30 developer/platform', () => {
      const pid = newPublisherId();
      engine.ensureAccount(pid);

      const tx = engine.recordSale(
        pid,
        crypto.randomUUID(),
        crypto.randomUUID(),
        { type: 'OneTime', cents: 1000 },
      );

      expect(tx).not.toBeNull();
      expect(tx!.developerCents).toBe(700);
      expect(tx!.platformCents).toBe(300);
      expect(engine.getAccount(pid)!.balanceCents).toBe(700);
      expect(engine.platformRevenue()).toBe(300);
    });

    it('should handle odd amounts correctly', () => {
      const pid = newPublisherId();
      engine.ensureAccount(pid);

      const tx = engine.recordSale(
        pid,
        crypto.randomUUID(),
        crypto.randomUUID(),
        { type: 'OneTime', cents: 999 },
      );

      expect(tx).not.toBeNull();
      // 999 * 70 / 100 = 699.3 -> floor to 699
      expect(tx!.developerCents).toBe(699);
      expect(tx!.platformCents).toBe(300); // 999 - 699
      expect(tx!.developerCents + tx!.platformCents).toBe(999);
    });
  });

  describe('free agent', () => {
    it('should return null for free agent sale', () => {
      const pid = newPublisherId();
      engine.ensureAccount(pid);

      const result = engine.recordSale(
        pid,
        crypto.randomUUID(),
        crypto.randomUUID(),
        { type: 'Free' },
      );

      expect(result).toBeNull();
    });
  });

  describe('payout threshold', () => {
    it('should not pay out below $50 threshold', () => {
      const pid = newPublisherId();
      engine.ensureAccount(pid);
      engine.setPayoutMethod(pid, {
        type: 'StripeConnect',
        accountId: 'acct_123',
      });

      // $49 gross -> $34.30 dev share (below $50)
      engine.recordSale(
        pid,
        crypto.randomUUID(),
        crypto.randomUUID(),
        { type: 'OneTime', cents: 4900 },
      );

      const payouts = engine.runPayoutCycle();
      expect(payouts).toHaveLength(0);
    });

    it('should pay out above $50 threshold', () => {
      const pid = newPublisherId();
      engine.ensureAccount(pid);
      engine.setPayoutMethod(pid, {
        type: 'StripeConnect',
        accountId: 'acct_123',
      });

      // $49 gross -> $34.30 dev share
      engine.recordSale(
        pid,
        crypto.randomUUID(),
        crypto.randomUUID(),
        { type: 'OneTime', cents: 4900 },
      );

      // Push above threshold: +$30 gross -> +$21 dev share, total $55.30
      engine.recordSale(
        pid,
        crypto.randomUUID(),
        crypto.randomUUID(),
        { type: 'OneTime', cents: 3000 },
      );

      const payouts = engine.runPayoutCycle();
      expect(payouts).toHaveLength(1);
      expect(engine.getAccount(pid)!.balanceCents).toBe(0);
    });
  });

  describe('payout without method', () => {
    it('should not pay out without a payout method set', () => {
      const pid = newPublisherId();
      engine.ensureAccount(pid);

      // Add enough balance but no payout method
      engine.recordSale(
        pid,
        crypto.randomUUID(),
        crypto.randomUUID(),
        { type: 'OneTime', cents: 10000 },
      );

      const payouts = engine.runPayoutCycle();
      expect(payouts).toHaveLength(0);
    });
  });

  describe('multiple sales accumulate', () => {
    it('should accumulate multiple sales into account balance', () => {
      const pid = newPublisherId();
      engine.ensureAccount(pid);

      engine.recordSale(
        pid,
        crypto.randomUUID(),
        crypto.randomUUID(),
        { type: 'OneTime', cents: 1000 },
      );

      engine.recordSale(
        pid,
        crypto.randomUUID(),
        crypto.randomUUID(),
        { type: 'OneTime', cents: 2000 },
      );

      const account = engine.getAccount(pid)!;
      expect(account.balanceCents).toBe(700 + 1400);
      expect(account.lifetimeEarnedCents).toBe(700 + 1400);
      expect(account.transactions).toHaveLength(2);
    });
  });

  describe('split sale (agent packs)', () => {
    it('should distribute revenue with custom split', () => {
      const creatorId = newPublisherId();
      const baseDevId = newPublisherId();
      engine.ensureAccount(creatorId);
      engine.ensureAccount(baseDevId);

      const split = celebritySplit(); // 50/30/20

      engine.recordSplitSale(
        creatorId,
        baseDevId,
        crypto.randomUUID(),
        crypto.randomUUID(),
        10000, // $100 gross
        split,
      );

      const creatorAccount = engine.getAccount(creatorId)!;
      const baseDevAccount = engine.getAccount(baseDevId)!;

      expect(creatorAccount.balanceCents).toBe(5000); // 50%
      expect(baseDevAccount.balanceCents).toBe(2000); // 20%
      expect(engine.platformRevenue()).toBe(3000); // 30%
    });

    it('should handle split sale with no base developer', () => {
      const creatorId = newPublisherId();
      engine.ensureAccount(creatorId);

      engine.recordSplitSale(
        creatorId,
        null,
        crypto.randomUUID(),
        crypto.randomUUID(),
        10000,
        celebritySplit(),
      );

      expect(engine.getAccount(creatorId)!.balanceCents).toBe(5000);
      expect(engine.platformRevenue()).toBe(3000);
    });

    it('should skip split sale for zero amount', () => {
      const creatorId = newPublisherId();
      engine.ensureAccount(creatorId);

      engine.recordSplitSale(
        creatorId,
        null,
        crypto.randomUUID(),
        crypto.randomUUID(),
        0,
        celebritySplit(),
      );

      expect(engine.getAccount(creatorId)!.balanceCents).toBe(0);
    });
  });

  describe('error handling', () => {
    it('should throw on sale for unknown publisher', () => {
      expect(() =>
        engine.recordSale(
          newPublisherId(),
          crypto.randomUUID(),
          crypto.randomUUID(),
          { type: 'OneTime', cents: 1000 },
        ),
      ).toThrow(MarketplaceError);
    });

    it('should throw on set payout method for unknown publisher', () => {
      expect(() =>
        engine.setPayoutMethod(newPublisherId(), {
          type: 'StripeConnect',
          accountId: 'acct_x',
        }),
      ).toThrow(MarketplaceError);
    });
  });

  describe('payout history', () => {
    it('should record payout history', () => {
      const pid = newPublisherId();
      engine.ensureAccount(pid);
      engine.setPayoutMethod(pid, {
        type: 'BankTransfer',
        routing: '111000025',
        account: '123456789',
      });

      engine.recordSale(
        pid,
        crypto.randomUUID(),
        crypto.randomUUID(),
        { type: 'OneTime', cents: 10000 },
      );

      engine.runPayoutCycle();

      expect(engine.payoutHistory()).toHaveLength(1);
      expect(engine.payoutHistory()[0].publisherId).toBe(pid);
      expect(engine.payoutHistory()[0].amountCents).toBe(7000);
    });
  });

  describe('monthly subscription revenue', () => {
    it('should handle monthly subscription pricing', () => {
      const pid = newPublisherId();
      engine.ensureAccount(pid);

      const tx = engine.recordSale(
        pid,
        crypto.randomUUID(),
        crypto.randomUUID(),
        { type: 'Monthly', cents: 499 },
      );

      expect(tx).not.toBeNull();
      expect(tx!.grossCents).toBe(499);
      expect(tx!.developerCents).toBe(349); // floor(499 * 70 / 100)
      expect(tx!.platformCents).toBe(150); // 499 - 349
    });
  });
});
