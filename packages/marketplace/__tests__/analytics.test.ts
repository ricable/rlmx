/**
 * Tests for MarketplaceAnalytics.
 * Mirrors rlmx-marketplace/src/analytics.rs tests.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { LifeDomain } from '@aix/shared';
import { MarketplaceAnalytics, retentionRate } from '../src/analytics.js';

describe('MarketplaceAnalytics', () => {
  let analytics: MarketplaceAnalytics;

  beforeEach(() => {
    analytics = new MarketplaceAnalytics();
  });

  describe('install tracking', () => {
    it('should track installs', () => {
      const agentId = crypto.randomUUID();
      analytics.recordInstall(agentId, crypto.randomUUID(), LifeDomain.Finance);
      analytics.recordInstall(agentId, crypto.randomUUID(), LifeDomain.Finance);

      expect(analytics.totalInstalls()).toBe(2);
      expect(analytics.agentMetrics(agentId)!.totalInstalls).toBe(2);
    });

    it('should track installs across agents', () => {
      const a1 = crypto.randomUUID();
      const a2 = crypto.randomUUID();
      analytics.recordInstall(a1, crypto.randomUUID(), LifeDomain.Finance);
      analytics.recordInstall(a2, crypto.randomUUID(), LifeDomain.Health);

      expect(analytics.totalInstalls()).toBe(2);
      expect(analytics.agentMetrics(a1)!.totalInstalls).toBe(1);
      expect(analytics.agentMetrics(a2)!.totalInstalls).toBe(1);
    });
  });

  describe('uninstall tracking', () => {
    it('should track uninstalls', () => {
      const agentId = crypto.randomUUID();
      analytics.recordInstall(agentId, crypto.randomUUID(), LifeDomain.Finance);
      analytics.recordUninstall(agentId, crypto.randomUUID());

      expect(analytics.agentMetrics(agentId)!.totalUninstalls).toBe(1);
    });
  });

  describe('revenue tracking', () => {
    it('should track revenue per agent', () => {
      const a1 = crypto.randomUUID();
      const a2 = crypto.randomUUID();
      analytics.recordRevenue(a1, 1000);
      analytics.recordRevenue(a2, 500);

      expect(analytics.totalRevenueCents()).toBe(1500);
      expect(analytics.agentMetrics(a1)!.revenueCents).toBe(1000);
      expect(analytics.agentMetrics(a2)!.revenueCents).toBe(500);
    });

    it('should accumulate revenue', () => {
      const agentId = crypto.randomUUID();
      analytics.recordRevenue(agentId, 1000);
      analytics.recordRevenue(agentId, 500);

      expect(analytics.agentMetrics(agentId)!.revenueCents).toBe(1500);
    });
  });

  describe('top by installs', () => {
    it('should return top agents by install count', () => {
      const a1 = crypto.randomUUID();
      const a2 = crypto.randomUUID();

      for (let i = 0; i < 5; i++) {
        analytics.recordInstall(a1, crypto.randomUUID(), LifeDomain.Health);
      }
      for (let i = 0; i < 10; i++) {
        analytics.recordInstall(a2, crypto.randomUUID(), LifeDomain.Health);
      }

      const top = analytics.topByInstalls(1);
      expect(top).toHaveLength(1);
      expect(top[0].agentId).toBe(a2);
      expect(top[0].installs).toBe(10);
    });
  });

  describe('top by revenue', () => {
    it('should return top agents by revenue', () => {
      const a1 = crypto.randomUUID();
      const a2 = crypto.randomUUID();
      analytics.recordRevenue(a1, 500);
      analytics.recordRevenue(a2, 2000);

      const top = analytics.topByRevenue(1);
      expect(top).toHaveLength(1);
      expect(top[0].agentId).toBe(a2);
      expect(top[0].revenue).toBe(2000);
    });
  });

  describe('domain metrics', () => {
    it('should aggregate installs by domain', () => {
      analytics.recordInstall(crypto.randomUUID(), crypto.randomUUID(), LifeDomain.Finance);
      analytics.recordInstall(crypto.randomUUID(), crypto.randomUUID(), LifeDomain.Finance);
      analytics.recordInstall(crypto.randomUUID(), crypto.randomUUID(), LifeDomain.Health);

      const dm = analytics.domainMetrics();
      expect(dm.get(LifeDomain.Finance)!.totalInstalls).toBe(2);
      expect(dm.get(LifeDomain.Health)!.totalInstalls).toBe(1);
    });
  });

  describe('retention rate', () => {
    it('should compute retention rate', () => {
      const rate = retentionRate({
        totalInstalls: 100,
        totalUninstalls: 20,
        revenueCents: 0,
      });
      expect(rate).toBeCloseTo(80.0, 2);
    });

    it('should return 0 for zero installs', () => {
      const rate = retentionRate({
        totalInstalls: 0,
        totalUninstalls: 0,
        revenueCents: 0,
      });
      expect(rate).toBe(0);
    });

    it('should handle all users uninstalling', () => {
      const rate = retentionRate({
        totalInstalls: 50,
        totalUninstalls: 50,
        revenueCents: 0,
      });
      expect(rate).toBeCloseTo(0, 2);
    });
  });

  describe('non-existent agent', () => {
    it('should return undefined for unknown agent metrics', () => {
      expect(analytics.agentMetrics(crypto.randomUUID())).toBeUndefined();
    });
  });
});
