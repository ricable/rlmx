import { generateId } from '@aix/shared';
import type { BackendAdapter } from './backend.js';
import type { RagSearchResult } from '../types.js';

/**
 * Docling backend — ingestion only (PDF/DOCX → markdown conversion).
 * Search is not supported; docling converts documents which then get indexed by qmd.
 */
export class DoclingBackend implements BackendAdapter {
  readonly name = 'docling';
  private readonly ingestedDir: string;

  constructor(ingestedDir: string) {
    this.ingestedDir = ingestedDir;
  }

  async health(): Promise<{ status: 'available' | 'unavailable'; details?: string }> {
    // Docling is available if the ingested directory exists
    try {
      const { access } = await import('node:fs/promises');
      await access(this.ingestedDir);
      return { status: 'available' };
    } catch {
      return { status: 'unavailable', details: 'ingested directory not accessible' };
    }
  }

  async search(_query: string, _k: number): Promise<RagSearchResult[]> {
    // Docling does not support search — documents are indexed via qmd after conversion
    return [];
  }

  async ingest(file: string, _collection: string): Promise<{ id: string; chunks: number }> {
    // Stub: in production, this would invoke docling-mcp to convert PDF/DOCX → markdown
    // then save to ingestedDir and trigger qmd reindex
    const id = generateId();
    return { id, chunks: 0 };
  }
}
