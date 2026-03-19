/**
 * Tests for the PublisherPortal.
 * Mirrors rlmx-marketplace/src/publisher.rs tests.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import {
  PublisherPortal,
  addAgentToPublisher,
  removeAgentFromPublisher,
  updateReputation,
} from '../src/publisher.js';
import { DeveloperType } from '../src/types.js';
import { MarketplaceError } from '../src/errors.js';

describe('PublisherPortal', () => {
  let portal: PublisherPortal;

  beforeEach(() => {
    portal = new PublisherPortal();
  });

  it('should register and look up a publisher', () => {
    const id = portal.register(
      'Acme',
      'dev@acme.com',
      DeveloperType.Organization,
    );

    const pub = portal.get(id);
    expect(pub).toBeDefined();
    expect(pub!.name).toBe('Acme');
    expect(pub!.verified).toBe(false);
  });

  it('should reject duplicate email', () => {
    portal.register('A', 'a@test.com', DeveloperType.Individual);

    expect(() =>
      portal.register('B', 'a@test.com', DeveloperType.Individual),
    ).toThrow(MarketplaceError);
  });

  it('should verify a publisher', () => {
    const id = portal.register(
      'Dev',
      'dev@test.com',
      DeveloperType.Individual,
    );

    portal.verify(id);
    expect(portal.get(id)!.verified).toBe(true);
  });

  it('should throw on verify non-existent publisher', () => {
    expect(() => portal.verify(crypto.randomUUID() as any)).toThrow(
      MarketplaceError,
    );
  });

  it('should look up by email', () => {
    portal.register('EmailDev', 'find@me.com', DeveloperType.Individual);

    const found = portal.byEmail('find@me.com');
    expect(found).toBeDefined();
    expect(found!.name).toBe('EmailDev');
  });

  it('should return undefined for unknown email', () => {
    expect(portal.byEmail('unknown@test.com')).toBeUndefined();
  });

  it('should count publishers', () => {
    expect(portal.count()).toBe(0);
    portal.register('A', 'a@test.com', DeveloperType.Individual);
    portal.register('B', 'b@test.com', DeveloperType.Individual);
    expect(portal.count()).toBe(2);
  });

  it('should list verified publishers', () => {
    const id1 = portal.register('A', 'a@test.com', DeveloperType.Individual);
    portal.register('B', 'b@test.com', DeveloperType.Individual);
    portal.verify(id1);

    const verified = portal.verifiedPublishers();
    expect(verified).toHaveLength(1);
    expect(verified[0].name).toBe('A');
  });
});

describe('Publisher helpers', () => {
  it('should add agent to publisher portfolio', () => {
    const portal = new PublisherPortal();
    const id = portal.register('Dev', 'dev@test.com', DeveloperType.Individual);
    const pub = portal.getMut(id)!;

    const agentId = crypto.randomUUID();
    addAgentToPublisher(pub, agentId);
    expect(pub.agents).toHaveLength(1);

    // Should not duplicate
    addAgentToPublisher(pub, agentId);
    expect(pub.agents).toHaveLength(1);
  });

  it('should remove agent from publisher portfolio', () => {
    const portal = new PublisherPortal();
    const id = portal.register('Dev', 'dev@test.com', DeveloperType.Individual);
    const pub = portal.getMut(id)!;

    const agentId = crypto.randomUUID();
    addAgentToPublisher(pub, agentId);
    removeAgentFromPublisher(pub, agentId);
    expect(pub.agents).toHaveLength(0);
  });

  it('should update reputation score', () => {
    const portal = new PublisherPortal();
    const id = portal.register('Dev', 'dev@test.com', DeveloperType.Individual);
    const pub = portal.getMut(id)!;

    updateReputation(pub, 4.0, 0.9);
    // 4.0 * 0.6 + (0.9 * 5.0) * 0.4 = 2.4 + 1.8 = 4.2
    expect(pub.reputationScore).toBeCloseTo(4.2, 2);
  });
});
