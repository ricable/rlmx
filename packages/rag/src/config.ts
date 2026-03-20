import type { RagConfig } from './types.js';
import { DEFAULT_RAG_CONFIG } from './types.js';

export function loadConfig(overrides?: Partial<RagConfig>): RagConfig {
  return {
    ...DEFAULT_RAG_CONFIG,
    qmdUrl: process.env.RAG_QMD_URL ?? DEFAULT_RAG_CONFIG.qmdUrl,
    qmdCollection: process.env.RAG_QMD_COLLECTION ?? DEFAULT_RAG_CONFIG.qmdCollection,
    doclingEnabled: process.env.RAG_DOCLING_ENABLED !== 'false',
    ingestedDir: process.env.RAG_INGESTED_DIR ?? DEFAULT_RAG_CONFIG.ingestedDir,
    sonaEnabled: process.env.RAG_SONA_ENABLED !== 'false',
    ...overrides,
  };
}
