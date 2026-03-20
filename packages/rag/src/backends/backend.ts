import type { RagSearchResult } from '../types.js';

export interface BackendAdapter {
  readonly name: string;
  health(): Promise<{ status: 'available' | 'unavailable'; details?: string }>;
  search(query: string, k: number): Promise<RagSearchResult[]>;
  ingest?(file: string, collection: string): Promise<{ id: string; chunks: number }>;
}

export class BackendRegistry {
  private backends = new Map<string, BackendAdapter>();

  register(adapter: BackendAdapter): void {
    this.backends.set(adapter.name, adapter);
  }

  get(name: string): BackendAdapter | undefined {
    return this.backends.get(name);
  }

  all(): BackendAdapter[] {
    return [...this.backends.values()];
  }

  names(): string[] {
    return [...this.backends.keys()];
  }
}
