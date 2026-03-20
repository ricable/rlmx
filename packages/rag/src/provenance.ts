import { generateId } from '@aix/shared';

export interface ProvenanceEntry {
  stage: string;
  source: string;
  timestamp: string;
  details?: Record<string, unknown>;
}

/**
 * In-memory provenance chain tracker.
 * Tracks: source → conversion → chunk → embed for each document.
 * Capped at MAX_CHAINS to prevent unbounded memory growth.
 */
export class ProvenanceTracker {
  private static readonly MAX_CHAINS = 10_000;
  private chains = new Map<string, ProvenanceEntry[]>();

  record(id: string, entry: ProvenanceEntry): void {
    const chain = this.chains.get(id) ?? [];
    chain.push(entry);
    this.chains.set(id, chain);
    // Evict oldest entries when over capacity
    if (this.chains.size > ProvenanceTracker.MAX_CHAINS) {
      const oldest = this.chains.keys().next().value;
      if (oldest !== undefined) this.chains.delete(oldest);
    }
  }

  get(id: string): ProvenanceEntry[] | undefined {
    return this.chains.get(id);
  }

  startChain(source: string): string {
    const id = generateId();
    this.record(id, {
      stage: 'source',
      source,
      timestamp: new Date().toISOString(),
    });
    return id;
  }

  clear(): void {
    this.chains.clear();
  }
}
