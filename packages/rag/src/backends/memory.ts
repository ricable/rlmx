import type { BackendAdapter } from './backend.js';
import type { RagSearchResult } from '../types.js';

/**
 * Memory backend — stub for future rlmx-kernel MemoryRegion integration.
 * When rlmx-kernel is available via NAPI, this will delegate to the
 * Rust HNSW-backed vector store.
 */
export class MemoryBackend implements BackendAdapter {
  readonly name = 'memory';

  async health(): Promise<{ status: 'available' | 'unavailable'; details?: string }> {
    return { status: 'unavailable', details: 'kernel MemoryRegion not yet wired' };
  }

  async search(_query: string, _k: number): Promise<RagSearchResult[]> {
    return [];
  }
}
