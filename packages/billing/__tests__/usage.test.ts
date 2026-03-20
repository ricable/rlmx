import { describe, it, expect } from 'vitest';
import {
  SubscriptionTier,
  tierLimits,
  createUsageMetrics,
  recordTokenUsage,
  checkAgentQuota,
  recordAgentSpawn,
  recordAgentTerminate,
  resetPeriod,
  tokenUtilization,
  BillingError,
} from '../src/index.js';

function makeMetrics(tier: SubscriptionTier) {
  const now = new Date().toISOString();
  const later = new Date(Date.now() + 30 * 86400_000).toISOString();
  return createUsageMetrics(tierLimits(tier), now, later);
}

describe('UsageMetrics', () => {
  describe('recordTokenUsage', () => {
    it('records within limit', () => {
      let m = makeMetrics(SubscriptionTier.Personal);
      m = recordTokenUsage(m, 50_000);
      expect(m.cloudTokensUsed).toBe(50_000);
      m = recordTokenUsage(m, 50_000);
      expect(m.cloudTokensUsed).toBe(100_000);
    });

    it('throws when exceeding limit', () => {
      const m = makeMetrics(SubscriptionTier.Personal);
      expect(() => recordTokenUsage(m, 100_001)).toThrow(BillingError);
    });
  });

  describe('agent quota', () => {
    it('Free tier: allows 5, blocks 6th', () => {
      let m = makeMetrics(SubscriptionTier.Free);
      for (let i = 0; i < 5; i++) {
        expect(() => checkAgentQuota(m)).not.toThrow();
        m = recordAgentSpawn(m);
      }
      // 6th should fail
      expect(() => checkAgentQuota(m)).toThrow(BillingError);
    });

    it('unlimited tier allows any count', () => {
      let m = makeMetrics(SubscriptionTier.Personal);
      for (let i = 0; i < 100; i++) {
        expect(() => checkAgentQuota(m)).not.toThrow();
        m = recordAgentSpawn(m);
      }
    });
  });

  describe('resetPeriod', () => {
    it('clears token usage', () => {
      let m = makeMetrics(SubscriptionTier.Personal);
      m = recordTokenUsage(m, 50_000);
      const now = new Date().toISOString();
      const later = new Date(Date.now() + 30 * 86400_000).toISOString();
      m = resetPeriod(m, now, later);
      expect(m.cloudTokensUsed).toBe(0);
    });
  });

  describe('tokenUtilization', () => {
    it('returns 0 when no usage', () => {
      const m = makeMetrics(SubscriptionTier.Personal);
      expect(tokenUtilization(m)).toBeCloseTo(0.0);
    });

    it('returns 0.5 at half usage', () => {
      let m = makeMetrics(SubscriptionTier.Personal);
      m = recordTokenUsage(m, 50_000);
      expect(tokenUtilization(m)).toBeCloseTo(0.5);
    });

    it('returns 0 for free tier with no usage', () => {
      const m = makeMetrics(SubscriptionTier.Free);
      expect(tokenUtilization(m)).toBeCloseTo(0.0);
    });
  });

  describe('agent terminate', () => {
    it('decrements active count', () => {
      let m = makeMetrics(SubscriptionTier.Free);
      m = recordAgentSpawn(m);
      m = recordAgentSpawn(m);
      expect(m.agentsActive).toBe(2);
      m = recordAgentTerminate(m);
      expect(m.agentsActive).toBe(1);
    });

    it('saturates at zero', () => {
      let m = makeMetrics(SubscriptionTier.Free);
      m = recordAgentTerminate(m);
      expect(m.agentsActive).toBe(0);
    });
  });
});
