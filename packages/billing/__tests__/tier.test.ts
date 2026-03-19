import { describe, it, expect } from 'vitest';
import {
  SubscriptionTier,
  FederationMode,
  monthlyPriceCents,
  tierLimits,
  enforceAgentLimit,
  enforceTokenLimit,
  enforceFeature,
} from '../src/index.js';

describe('SubscriptionTier', () => {
  describe('monthlyPriceCents', () => {
    it('should return correct prices for all tiers', () => {
      expect(monthlyPriceCents(SubscriptionTier.Free)).toBe(0);
      expect(monthlyPriceCents(SubscriptionTier.Personal)).toBe(999);
      expect(monthlyPriceCents(SubscriptionTier.Family)).toBe(1999);
      expect(monthlyPriceCents(SubscriptionTier.Pro)).toBe(2999);
      expect(monthlyPriceCents(SubscriptionTier.Enterprise)).toBe(0);
      expect(monthlyPriceCents(SubscriptionTier.Developer)).toBe(0);
    });
  });

  describe('tierLimits', () => {
    it('Free tier: exactly 5 agents, no cloud, receive-only federation', () => {
      const limits = tierLimits(SubscriptionTier.Free);
      expect(limits.maxAgents).toBe(5);
      expect(limits.cloudTokens).toBe(0);
      expect(limits.federationMode).toBe(FederationMode.ReceiveOnly);
      expect(limits.features.customSdk).toBe(false);
      expect(limits.features.marketplacePublish).toBe(false);
    });

    it('Personal tier: unlimited agents, 100K tokens', () => {
      const limits = tierLimits(SubscriptionTier.Personal);
      expect(limits.maxAgents).toBeNull();
      expect(limits.cloudTokens).toBe(100_000);
      expect(limits.federationMode).toBe(FederationMode.Full);
    });

    it('Family tier: family sharing enabled, 200K tokens', () => {
      const limits = tierLimits(SubscriptionTier.Family);
      expect(limits.features.familySharing).toBe(true);
      expect(limits.cloudTokens).toBe(200_000);
    });

    it('Pro tier: SDK, API, priority support, 500K tokens', () => {
      const limits = tierLimits(SubscriptionTier.Pro);
      expect(limits.features.customSdk).toBe(true);
      expect(limits.features.apiAccess).toBe(true);
      expect(limits.features.prioritySupport).toBe(true);
      expect(limits.cloudTokens).toBe(500_000);
    });

    it('Enterprise tier: everything enabled', () => {
      const limits = tierLimits(SubscriptionTier.Enterprise);
      expect(limits.features.soc2Hipaa).toBe(true);
      expect(limits.features.customSdk).toBe(true);
      expect(limits.features.apiAccess).toBe(true);
      expect(limits.features.marketplacePublish).toBe(true);
      expect(limits.features.familySharing).toBe(true);
      expect(limits.features.prioritySupport).toBe(true);
    });

    it('Developer tier: publishing and API access, 100K tokens', () => {
      const limits = tierLimits(SubscriptionTier.Developer);
      expect(limits.features.marketplacePublish).toBe(true);
      expect(limits.features.apiAccess).toBe(true);
      expect(limits.cloudTokens).toBe(100_000);
    });
  });

  describe('enforceAgentLimit', () => {
    it('Free tier: allows 4 agents, blocks at 5', () => {
      const limits = tierLimits(SubscriptionTier.Free);
      expect(enforceAgentLimit(limits, 4)).toBeNull();
      expect(enforceAgentLimit(limits, 5)).toEqual({
        type: 'AgentLimitReached',
        current: 5,
        limit: 5,
      });
      expect(enforceAgentLimit(limits, 6)).toEqual({
        type: 'AgentLimitReached',
        current: 6,
        limit: 5,
      });
    });

    it('unlimited tiers allow any count', () => {
      const limits = tierLimits(SubscriptionTier.Personal);
      expect(enforceAgentLimit(limits, 1_000_000)).toBeNull();
    });
  });

  describe('enforceTokenLimit', () => {
    it('blocks when tokens exhausted', () => {
      const limits = tierLimits(SubscriptionTier.Personal);
      expect(enforceTokenLimit(limits, 99_999)).toBeNull();
      expect(enforceTokenLimit(limits, 100_000)).toEqual({
        type: 'CloudTokensExhausted',
        used: 100_000,
        limit: 100_000,
      });
    });
  });

  describe('enforceFeature', () => {
    it('Free tier blocks custom_sdk and api_access', () => {
      const limits = tierLimits(SubscriptionTier.Free);
      expect(enforceFeature(limits, 'custom_sdk')).toEqual({
        type: 'FeatureUnavailable',
        feature: 'custom_sdk',
      });
      expect(enforceFeature(limits, 'api_access')).toEqual({
        type: 'FeatureUnavailable',
        feature: 'api_access',
      });
    });

    it('Pro tier allows custom_sdk and api_access', () => {
      const limits = tierLimits(SubscriptionTier.Pro);
      expect(enforceFeature(limits, 'custom_sdk')).toBeNull();
      expect(enforceFeature(limits, 'api_access')).toBeNull();
    });

    it('unknown feature is unavailable', () => {
      const limits = tierLimits(SubscriptionTier.Enterprise);
      expect(enforceFeature(limits, 'unknown_feature')).toEqual({
        type: 'FeatureUnavailable',
        feature: 'unknown_feature',
      });
    });
  });
});
