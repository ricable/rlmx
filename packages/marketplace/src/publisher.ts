/**
 * PublisherPortal: developer registration, management, and earnings tracking.
 * Mirrors rlmx-marketplace/src/publisher.rs.
 */

import type { DeveloperType, PublisherId, PublisherProfile } from './types.js';
import { newPublisherId } from './types.js';
import { duplicateEmail, publisherNotFound } from './errors.js';

// ---------------------------------------------------------------------------
// Publisher creation helper
// ---------------------------------------------------------------------------

/** Create a new unverified publisher. */
function createPublisher(
  name: string,
  email: string,
  developerType: DeveloperType,
): PublisherProfile {
  return {
    id: newPublisherId(),
    name,
    email,
    developerType,
    verified: false,
    agents: [],
    reputationScore: 0,
    joinedAt: new Date().toISOString(),
  };
}

/** Add an agent to this publisher's portfolio. */
export function addAgentToPublisher(
  publisher: PublisherProfile,
  agentId: string,
): void {
  if (!publisher.agents.includes(agentId)) {
    publisher.agents.push(agentId);
  }
}

/** Remove an agent from this publisher's portfolio. */
export function removeAgentFromPublisher(
  publisher: PublisherProfile,
  agentId: string,
): void {
  publisher.agents = publisher.agents.filter((a) => a !== agentId);
}

/**
 * Update reputation score based on marketplace-internal signals.
 * Weighted average: 60% ratings, 40% review pass rate (scaled to 5).
 */
export function updateReputation(
  publisher: PublisherProfile,
  avgRating: number,
  reviewPassRate: number,
): void {
  publisher.reputationScore = avgRating * 0.6 + reviewPassRate * 5.0 * 0.4;
}

// ---------------------------------------------------------------------------
// PublisherPortal
// ---------------------------------------------------------------------------

/** The publisher portal manages all publisher accounts. */
export class PublisherPortal {
  private readonly publishers = new Map<PublisherId, PublisherProfile>();
  private readonly emailIndex = new Map<string, PublisherId>();

  /** Register a new publisher. Returns the publisher ID. */
  register(
    name: string,
    email: string,
    developerType: DeveloperType,
  ): PublisherId {
    if (this.emailIndex.has(email)) {
      throw duplicateEmail(email);
    }

    const publisher = createPublisher(name, email, developerType);
    this.emailIndex.set(email, publisher.id);
    this.publishers.set(publisher.id, publisher);

    return publisher.id;
  }

  /** Get a publisher by ID. */
  get(id: PublisherId): PublisherProfile | undefined {
    return this.publishers.get(id);
  }

  /** Get a mutable publisher by ID. */
  getMut(id: PublisherId): PublisherProfile | undefined {
    return this.publishers.get(id);
  }

  /** Verify a publisher. */
  verify(id: PublisherId): void {
    const publisher = this.publishers.get(id);
    if (!publisher) {
      throw publisherNotFound(id);
    }
    publisher.verified = true;
  }

  /** Look up a publisher by email. */
  byEmail(email: string): PublisherProfile | undefined {
    const id = this.emailIndex.get(email);
    if (!id) return undefined;
    return this.publishers.get(id);
  }

  /** Total number of publishers. */
  count(): number {
    return this.publishers.size;
  }

  /** List all verified publishers. */
  verifiedPublishers(): PublisherProfile[] {
    return Array.from(this.publishers.values()).filter((p) => p.verified);
  }
}
