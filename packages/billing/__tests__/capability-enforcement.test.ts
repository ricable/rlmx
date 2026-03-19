import { describe, it, expect } from 'vitest';
import {
  SubscriptionTier,
  TierCapabilityEnforcer,
  BillingError,
  createUsageMetrics,
  tierLimits,
  recordAgentSpawn,
} from '../src/index.js';
import { SyscallPermission } from '@aix/shared';

describe('TierCapabilityEnforcer', () => {
  describe('deriveToken', () => {
    it('Free tier token has AgentLimit(5) caveat', () => {
      const token = TierCapabilityEnforcer.deriveToken(SubscriptionTier.Free);
      expect(token.caveats).toContainEqual({ type: 'AgentLimit', limit: 5 });
      expect(token.caveats).toContainEqual({
        type: 'CloudBurstAllowed',
        allowed: false,
      });
      expect(token.caveats).toContainEqual({
        type: 'FederationAllowed',
        allowed: false,
      });
    });

    it('Personal tier token has no AgentLimit caveat', () => {
      const token = TierCapabilityEnforcer.deriveToken(
        SubscriptionTier.Personal,
      );
      const agentLimitCaveat = token.caveats.find(
        (c) => c.type === 'AgentLimit',
      );
      expect(agentLimitCaveat).toBeUndefined();
      expect(token.caveats).toContainEqual({
        type: 'CloudBurstAllowed',
        allowed: true,
      });
      expect(token.caveats).toContainEqual({
        type: 'FederationAllowed',
        allowed: true,
      });
    });

    it('Developer tier has marketplace publish caveat', () => {
      const token = TierCapabilityEnforcer.deriveToken(
        SubscriptionTier.Developer,
      );
      expect(token.caveats).toContainEqual({
        type: 'MarketplacePublishAllowed',
        allowed: true,
      });
      expect(token.caveats).toContainEqual({
        type: 'ApiAccessAllowed',
        allowed: true,
      });
    });

    it('Pro tier has advanced permissions', () => {
      const token = TierCapabilityEnforcer.deriveToken(SubscriptionTier.Pro);
      expect(token.permissions).toContain(SyscallPermission.GraphCut);
      expect(token.permissions).toContain(SyscallPermission.StateMutate);
    });

    it('Free tier lacks advanced permissions', () => {
      const token = TierCapabilityEnforcer.deriveToken(SubscriptionTier.Free);
      expect(token.permissions).not.toContain(SyscallPermission.GraphCut);
      expect(token.permissions).not.toContain(SyscallPermission.StateMutate);
    });
  });

  describe('enforceFeature', () => {
    it('Free tier denies custom_sdk and api_access', () => {
      expect(() =>
        TierCapabilityEnforcer.enforceFeature(
          SubscriptionTier.Free,
          'custom_sdk',
        ),
      ).toThrow(BillingError);
      expect(() =>
        TierCapabilityEnforcer.enforceFeature(
          SubscriptionTier.Free,
          'api_access',
        ),
      ).toThrow(BillingError);
    });

    it('Pro tier allows custom_sdk and api_access', () => {
      expect(() =>
        TierCapabilityEnforcer.enforceFeature(
          SubscriptionTier.Pro,
          'custom_sdk',
        ),
      ).not.toThrow();
      expect(() =>
        TierCapabilityEnforcer.enforceFeature(
          SubscriptionTier.Pro,
          'api_access',
        ),
      ).not.toThrow();
    });
  });

  describe('enforceSpawn', () => {
    it('Free tier blocks spawn at 5 agents', () => {
      const now = new Date().toISOString();
      const limits = tierLimits(SubscriptionTier.Free);
      let usage = createUsageMetrics(limits, now, now);

      // Spawn 5 agents
      for (let i = 0; i < 5; i++) {
        TierCapabilityEnforcer.enforceSpawn(usage);
        usage = recordAgentSpawn(usage);
      }

      // 6th should be denied
      expect(() => TierCapabilityEnforcer.enforceSpawn(usage)).toThrow(
        BillingError,
      );
    });

    it('Personal tier allows unlimited spawns', () => {
      const now = new Date().toISOString();
      const limits = tierLimits(SubscriptionTier.Personal);
      let usage = createUsageMetrics(limits, now, now);

      for (let i = 0; i < 100; i++) {
        TierCapabilityEnforcer.enforceSpawn(usage);
        usage = recordAgentSpawn(usage);
      }
      // Should not throw
    });
  });

  describe('enforceCloudUsage', () => {
    it('blocks when tokens exceed limit', () => {
      const now = new Date().toISOString();
      const limits = tierLimits(SubscriptionTier.Personal);
      const usage = createUsageMetrics(limits, now, now);

      expect(() =>
        TierCapabilityEnforcer.enforceCloudUsage(usage, 100_001),
      ).toThrow(BillingError);
    });

    it('allows usage within limit', () => {
      const now = new Date().toISOString();
      const limits = tierLimits(SubscriptionTier.Personal);
      const usage = createUsageMetrics(limits, now, now);

      const updated = TierCapabilityEnforcer.enforceCloudUsage(usage, 50_000);
      expect(updated.cloudTokensUsed).toBe(50_000);
    });
  });
});
