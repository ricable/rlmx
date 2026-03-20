import { describe, it, expect } from 'vitest';
import {
  DEVELOPER_SHARE_PCT,
  DEFAULT_PAYOUT_THRESHOLD_CENTS,
  createDeveloperAccount,
  setPayoutMethod,
  recordSale,
  calculatePayout,
  requestPayout,
  publishAgent,
  totalSales,
  BillingError,
} from '../src/index.js';

describe('DeveloperAccount', () => {
  it('creates a developer account with defaults', () => {
    const dev = createDeveloperAccount('TestDev');
    expect(dev.publisherName).toBe('TestDev');
    expect(dev.balanceCents).toBe(0);
    expect(dev.payoutThresholdCents).toBe(DEFAULT_PAYOUT_THRESHOLD_CENTS);
    expect(dev.payoutThresholdCents).toBe(5000);
  });

  it('records a sale with 70/30 split', () => {
    const dev = createDeveloperAccount('TestDev');
    const [updated, sale] = recordSale(dev, 'agent-1', 'buyer-1', 1000);
    expect(sale.developerCents).toBe(700);
    expect(sale.platformCents).toBe(300);
    expect(updated.balanceCents).toBe(700);
    expect(updated.lifetimeEarnedCents).toBe(700);
  });

  it('enforces 70% developer share', () => {
    expect(DEVELOPER_SHARE_PCT).toBe(70);
    const dev = createDeveloperAccount('TestDev');
    const [, sale] = recordSale(dev, 'a', 'b', 10000);
    expect(sale.developerCents).toBe(7000);
    expect(sale.platformCents).toBe(3000);
  });

  it('enforces payout threshold ($50)', () => {
    let dev = createDeveloperAccount('TestDev');
    dev = setPayoutMethod(dev, {
      type: 'StripeConnect',
      accountId: 'acct_123',
    });

    // $49 gross -> $34.30 dev share, below $50 threshold
    [dev] = recordSale(dev, 'a', 'b', 4900);
    expect(() => requestPayout(dev)).toThrow(BillingError);

    // Add more to exceed threshold
    [dev] = recordSale(dev, 'a', 'b', 3000);
    const [paid, payout] = requestPayout(dev);
    expect(payout.amountCents).toBeGreaterThan(0);
    expect(paid.balanceCents).toBe(0);
  });

  it('payout requires a payout method', () => {
    let dev = createDeveloperAccount('TestDev');
    [dev] = recordSale(dev, 'a', 'b', 100_000);
    expect(() => requestPayout(dev)).toThrow(BillingError);
  });

  it('publish agent is idempotent', () => {
    let dev = createDeveloperAccount('TestDev');
    dev = publishAgent(dev, 'agent-1');
    dev = publishAgent(dev, 'agent-1');
    expect(dev.publishedAgents.length).toBe(1);
  });

  it('multiple sales accumulate', () => {
    let dev = createDeveloperAccount('TestDev');
    [dev] = recordSale(dev, 'a', 'b', 1000);
    [dev] = recordSale(dev, 'a', 'c', 2000);
    expect(dev.balanceCents).toBe(2100); // 700 + 1400
    expect(totalSales(dev)).toBe(2);
  });

  it('calculatePayout returns current balance', () => {
    let dev = createDeveloperAccount('TestDev');
    [dev] = recordSale(dev, 'a', 'b', 1000);
    expect(calculatePayout(dev)).toBe(700);
  });
});
