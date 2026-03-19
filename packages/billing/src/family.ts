/**
 * Family plan: shared groups with per-member privacy boundaries.
 * Maps to rlmx-billing/src/family.rs.
 */

import {
  familyGroupFull,
  memberAlreadyExists,
  memberNotFound,
  cannotRemoveOwner,
} from './errors.js';

/** Maximum members in a family group. */
export const MAX_FAMILY_MEMBERS = 6;

/** Role within a family group. */
export type FamilyRole = 'Owner' | 'Adult' | 'Child';

/** Privacy boundary for a family member, controlling data visibility. */
export interface PrivacyBoundary {
  /** Whether this member's agent usage is visible to the owner. */
  usageVisibleToOwner: boolean;
  /** Whether this member's voice transcripts are shared. */
  transcriptsShared: boolean;
  /** Whether this member's agent data is isolated from other members. */
  dataIsolated: boolean;
}

/** Default privacy boundary for adult members. */
export function defaultPrivacyBoundary(): PrivacyBoundary {
  return {
    usageVisibleToOwner: true,
    transcriptsShared: false,
    dataIsolated: true,
  };
}

/** Privacy boundary for child members: owner-visible, isolated data. */
export function childPrivacyBoundary(): PrivacyBoundary {
  return {
    usageVisibleToOwner: true,
    transcriptsShared: false,
    dataIsolated: true,
  };
}

/** A member within a family group. */
export interface FamilyMember {
  userId: string;
  role: FamilyRole;
  privacyBoundary: PrivacyBoundary;
  joinedAt: string;
}

/** A family group sharing a Family-tier subscription. */
export interface FamilyGroup {
  id: string;
  owner: string;
  members: FamilyMember[];
  /** Agent IDs shared across the family group. */
  sharedAgents: string[];
  createdAt: string;
}

let _nextId = 0;
function generateId(): string {
  return `fg-${Date.now()}-${++_nextId}-${Math.random().toString(36).slice(2, 8)}`;
}

/** Create a new family group with the given owner. */
export function createFamilyGroup(ownerId: string): FamilyGroup {
  const now = new Date().toISOString();
  const ownerMember: FamilyMember = {
    userId: ownerId,
    role: 'Owner',
    privacyBoundary: {
      usageVisibleToOwner: true,
      transcriptsShared: false,
      dataIsolated: false,
    },
    joinedAt: now,
  };
  return {
    id: generateId(),
    owner: ownerId,
    members: [ownerMember],
    sharedAgents: [],
    createdAt: now,
  };
}

/**
 * Add a member to the family group.
 * Returns a new FamilyGroup with the member added.
 * Throws BillingError if the group is full or member already exists.
 */
export function addMember(
  group: FamilyGroup,
  userId: string,
  role: FamilyRole,
  privacy: PrivacyBoundary,
): FamilyGroup {
  if (group.members.length >= MAX_FAMILY_MEMBERS) {
    throw familyGroupFull(MAX_FAMILY_MEMBERS);
  }
  if (group.members.some((m) => m.userId === userId)) {
    throw memberAlreadyExists(userId);
  }
  const member: FamilyMember = {
    userId,
    role,
    privacyBoundary: privacy,
    joinedAt: new Date().toISOString(),
  };
  return {
    ...group,
    members: [...group.members, member],
  };
}

/**
 * Remove a member from the family group. Cannot remove the owner.
 * Returns a new FamilyGroup with the member removed.
 */
export function removeMember(
  group: FamilyGroup,
  userId: string,
): FamilyGroup {
  if (userId === group.owner) {
    throw cannotRemoveOwner();
  }
  const filtered = group.members.filter((m) => m.userId !== userId);
  if (filtered.length === group.members.length) {
    throw memberNotFound(userId);
  }
  return { ...group, members: filtered };
}

/** Share an agent with all family members. Idempotent. */
export function shareAgent(
  group: FamilyGroup,
  agentId: string,
): FamilyGroup {
  if (group.sharedAgents.includes(agentId)) {
    return group;
  }
  return {
    ...group,
    sharedAgents: [...group.sharedAgents, agentId],
  };
}

/** Unshare an agent from the family group. */
export function unshareAgent(
  group: FamilyGroup,
  agentId: string,
): FamilyGroup {
  return {
    ...group,
    sharedAgents: group.sharedAgents.filter((a) => a !== agentId),
  };
}

/** Get a member by userId, or undefined if not found. */
export function getMember(
  group: FamilyGroup,
  userId: string,
): FamilyMember | undefined {
  return group.members.find((m) => m.userId === userId);
}

/** Number of members in the group. */
export function memberCount(group: FamilyGroup): number {
  return group.members.length;
}
