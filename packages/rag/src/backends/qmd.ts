import type { BackendAdapter } from './backend.js';
import type { RagSearchResult } from '../types.js';

export class QmdBackend implements BackendAdapter {
  readonly name = 'qmd';
  private readonly url: string;
  private readonly collection: string;

  constructor(url: string, collection: string) {
    this.url = url;
    this.collection = collection;
  }

  async health(): Promise<{ status: 'available' | 'unavailable'; details?: string }> {
    try {
      const res = await fetch(`${this.url}/api/status`, { signal: AbortSignal.timeout(2000) });
      if (res.ok) return { status: 'available' };
      return { status: 'unavailable', details: `HTTP ${res.status}` };
    } catch {
      return { status: 'unavailable', details: 'connection refused' };
    }
  }

  async search(query: string, k: number): Promise<RagSearchResult[]> {
    try {
      const res = await fetch(`${this.url}/api/search`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ query, collection: this.collection, limit: k }),
        signal: AbortSignal.timeout(5000),
      });
      if (!res.ok) return [];
      const data = await res.json() as { results?: Array<{ id?: string; content?: string; score?: number; source?: string; metadata?: Record<string, unknown> }> };
      return (data.results ?? []).map((r, i) => ({
        id: r.id ?? `qmd-${i}`,
        content: r.content ?? '',
        score: r.score ?? 0,
        source: r.source ?? 'qmd',
        backend: 'qmd',
        metadata: r.metadata,
      }));
    } catch {
      return [];
    }
  }
}
