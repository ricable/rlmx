import { describe, it, expect } from 'vitest';
import {
  MAX_FAMILY_MEMBERS,
  createFamilyGroup,
  addMember,
  removeMember,
  shareAgent,
  unshareAgent,
  getMember,
  memberCount,
  defaultPrivacyBoundary,
  BillingError,
} from '../src/index.js';

describe('FamilyGroup', () => {
  it('creates a family group with owner as first member', () => {
    const group = createFamilyGroup('owner-1');
    expect(group.owner).toBe('owner-1');
    expect(memberCount(group)).toBe(1);
    expect(group.members[0].role).toBe('Owner');
  });

  it('adds and removes members', () => {
    let group = createFamilyGroup('owner-1');
    group = addMember(group, 'member-1', 'Adult', defaultPrivacyBoundary());
    expect(memberCount(group)).toBe(2);

    group = removeMember(group, 'member-1');
    expect(memberCount(group)).toBe(1);
  });

  it('cannot remove the owner', () => {
    const group = createFamilyGroup('owner-1');
    expect(() => removeMember(group, 'owner-1')).toThrow(BillingError);
  });

  it('enforces max 6 members', () => {
    let group = createFamilyGroup('owner-1');
    // Add 5 more members (total 6)
    for (let i = 0; i < 5; i++) {
      group = addMember(
        group,
        `member-${i}`,
        'Adult',
        defaultPrivacyBoundary(),
      );
    }
    expect(memberCount(group)).toBe(MAX_FAMILY_MEMBERS);

    // 7th member should fail
    expect(() =>
      addMember(group, 'member-extra', 'Adult', defaultPrivacyBoundary()),
    ).toThrow(BillingError);
  });

  it('rejects duplicate members', () => {
    let group = createFamilyGroup('owner-1');
    group = addMember(group, 'member-1', 'Adult', defaultPrivacyBoundary());
    expect(() =>
      addMember(group, 'member-1', 'Adult', defaultPrivacyBoundary()),
    ).toThrow(BillingError);
  });

  it('shares and unshares agents (idempotent)', () => {
    let group = createFamilyGroup('owner-1');
    group = shareAgent(group, 'agent-1');
    expect(group.sharedAgents.length).toBe(1);

    // Idempotent
    group = shareAgent(group, 'agent-1');
    expect(group.sharedAgents.length).toBe(1);

    group = unshareAgent(group, 'agent-1');
    expect(group.sharedAgents.length).toBe(0);
  });

  it('gets member by userId', () => {
    let group = createFamilyGroup('owner-1');
    group = addMember(group, 'member-1', 'Child', defaultPrivacyBoundary());
    const member = getMember(group, 'member-1');
    expect(member).toBeDefined();
    expect(member!.role).toBe('Child');
  });

  it('returns undefined for non-existent member', () => {
    const group = createFamilyGroup('owner-1');
    expect(getMember(group, 'nobody')).toBeUndefined();
  });

  it('removing non-existent member throws', () => {
    const group = createFamilyGroup('owner-1');
    expect(() => removeMember(group, 'nobody')).toThrow(BillingError);
  });
});
