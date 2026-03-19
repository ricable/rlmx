import { describe, it, expect } from 'vitest';
import {
  SubscriptionTier,
  subscribe,
  upgrade,
  downgrade,
  cancel,
  reactivate,
  markPastDue,
  suspend,
  isActive,
  BillingError,
} from '../src/index.js';

describe('Subscription', () => {
  it('subscribe creates an active subscription', () => {
    const [sub, event] = subscribe('user-1', SubscriptionTier.Personal);
    expect(sub.status).toBe('Active');
    expect(sub.plan).toBe(SubscriptionTier.Personal);
    expect(sub.familyGroup).toBeNull();
    expect(sub.developerAccount).toBeNull();
    expect(event.type).toBe('Created');
  });

  it('Family tier creates family group', () => {
    const [sub] = subscribe('user-1', SubscriptionTier.Family);
    expect(sub.familyGroup).not.toBeNull();
    expect(sub.familyGroup!.owner).toBe('user-1');
  });

  it('Developer tier creates developer account', () => {
    const [sub] = subscribe('user-1', SubscriptionTier.Developer);
    expect(sub.developerAccount).not.toBeNull();
  });

  describe('upgrade', () => {
    it('upgrades to a higher tier', () => {
      const [sub] = subscribe('user-1', SubscriptionTier.Personal);
      const [upgraded, event] = upgrade(sub, SubscriptionTier.Pro);
      expect(upgraded.plan).toBe(SubscriptionTier.Pro);
      expect(event.type).toBe('Upgraded');
      if (event.type === 'Upgraded') {
        expect(event.from).toBe(SubscriptionTier.Personal);
        expect(event.to).toBe(SubscriptionTier.Pro);
      }
    });

    it('upgrade to Family creates family group', () => {
      const [sub] = subscribe('user-1', SubscriptionTier.Personal);
      const [upgraded] = upgrade(sub, SubscriptionTier.Family);
      expect(upgraded.familyGroup).not.toBeNull();
    });

    it('upgrade to Developer creates dev account', () => {
      const [sub] = subscribe('user-1', SubscriptionTier.Personal);
      const [upgraded] = upgrade(sub, SubscriptionTier.Developer);
      expect(upgraded.developerAccount).not.toBeNull();
    });

    it('cannot upgrade to a lower-priced tier', () => {
      const [sub] = subscribe('user-1', SubscriptionTier.Pro);
      expect(() => upgrade(sub, SubscriptionTier.Personal)).toThrow(BillingError);
    });

    it('cannot upgrade when not active', () => {
      const [sub] = subscribe('user-1', SubscriptionTier.Personal);
      const [cancelled] = cancel(sub);
      expect(() => upgrade(cancelled, SubscriptionTier.Pro)).toThrow(BillingError);
    });
  });

  describe('downgrade', () => {
    it('downgrades to a lower tier', () => {
      const [sub] = subscribe('user-1', SubscriptionTier.Pro);
      const [downgraded, event] = downgrade(sub, SubscriptionTier.Personal);
      expect(downgraded.plan).toBe(SubscriptionTier.Personal);
      expect(event.type).toBe('Downgraded');
    });

    it('downgrade removes family group', () => {
      const [sub] = subscribe('user-1', SubscriptionTier.Family);
      expect(sub.familyGroup).not.toBeNull();
      const [downgraded] = downgrade(sub, SubscriptionTier.Personal);
      expect(downgraded.familyGroup).toBeNull();
    });

    it('cannot downgrade to a higher-priced tier', () => {
      const [sub] = subscribe('user-1', SubscriptionTier.Personal);
      expect(() => downgrade(sub, SubscriptionTier.Pro)).toThrow(BillingError);
    });
  });

  describe('cancel and reactivate', () => {
    it('cancels the subscription', () => {
      const [sub] = subscribe('user-1', SubscriptionTier.Personal);
      const [cancelled, event] = cancel(sub);
      expect(cancelled.status).toBe('Cancelled');
      expect(event.type).toBe('Cancelled');
    });

    it('double cancel throws', () => {
      const [sub] = subscribe('user-1', SubscriptionTier.Personal);
      const [cancelled] = cancel(sub);
      expect(() => cancel(cancelled)).toThrow(BillingError);
    });

    it('reactivate from cancelled', () => {
      const [sub] = subscribe('user-1', SubscriptionTier.Personal);
      const [cancelled] = cancel(sub);
      const [reactivated, event] = reactivate(cancelled, SubscriptionTier.Pro);
      expect(reactivated.status).toBe('Active');
      expect(reactivated.plan).toBe(SubscriptionTier.Pro);
      expect(event.type).toBe('Reactivated');
    });

    it('reactivate requires cancelled or suspended', () => {
      const [sub] = subscribe('user-1', SubscriptionTier.Personal);
      expect(() => reactivate(sub, SubscriptionTier.Personal)).toThrow(BillingError);
    });
  });

  describe('grace period flow', () => {
    it('mark past due sets grace period', () => {
      const [sub] = subscribe('user-1', SubscriptionTier.Personal);
      const [pastDue, event] = markPastDue(sub);
      expect(pastDue.status).toBe('GracePeriod');
      expect(pastDue.gracePeriodEnd).not.toBeNull();
      expect(isActive(pastDue)).toBe(true);
      expect(event.type).toBe('StatusChanged');
    });

    it('suspend after grace period', () => {
      const [sub] = subscribe('user-1', SubscriptionTier.Personal);
      const [pastDue] = markPastDue(sub);
      const [suspended] = suspend(pastDue);
      expect(suspended.status).toBe('Suspended');
      expect(isActive(suspended)).toBe(false);
    });

    it('reactivate from suspended', () => {
      const [sub] = subscribe('user-1', SubscriptionTier.Personal);
      const [pastDue] = markPastDue(sub);
      const [suspended] = suspend(pastDue);
      expect(isActive(suspended)).toBe(false);

      const [reactivated] = reactivate(suspended, SubscriptionTier.Personal);
      expect(isActive(reactivated)).toBe(true);
    });
  });

  describe('usage limits match tier', () => {
    it('Free tier has 5 agent limit and 0 cloud tokens', () => {
      const [sub] = subscribe('user-1', SubscriptionTier.Free);
      expect(sub.usage.agentsLimit).toBe(5);
      expect(sub.usage.cloudTokensLimit).toBe(0);
    });

    it('upgrade updates usage limits', () => {
      const [sub] = subscribe('user-1', SubscriptionTier.Free);
      expect(sub.usage.cloudTokensLimit).toBe(0);
      const [upgraded] = upgrade(sub, SubscriptionTier.Personal);
      expect(upgraded.usage.cloudTokensLimit).toBe(100_000);
      expect(upgraded.usage.agentsLimit).toBeNull();
    });
  });
});
