export interface RagSearchInput {
  query: string;
  scope?: 'project' | 'all';
  backends?: string[];
  k?: number;
  filters?: Record<string, unknown>;
}

export interface RagSearchResult {
  id: string;
  content: string;
  score: number;
  source: string;
  backend: string;
  metadata?: Record<string, unknown>;
}

export interface RagIngestInput {
  file: string;
  collection?: string;
  tags?: string[];
}

export interface RagIngestResult {
  id: string;
  chunks: number;
  collection: string;
  backend: string;
}

export interface RagCompareInput {
  textA: string;
  textB: string;
}

export interface RagCompareResult {
  similarity: number;
  method: string;
}

export interface RagClusterInput {
  query: string;
  k?: number;
}

export interface RagClusterResult {
  clusters: Array<{
    label: string;
    items: RagSearchResult[];
  }>;
}

export interface RagProvenanceInput {
  id: string;
}

export interface RagProvenanceResult {
  chain: Array<{
    stage: string;
    source: string;
    timestamp: string;
    details?: Record<string, unknown>;
  }>;
}

export interface RagStatsResult {
  backends: Array<{
    name: string;
    status: 'available' | 'unavailable';
    documents?: number;
    collections?: number;
  }>;
  totalDocuments: number;
}

export interface RagReindexInput {
  collection?: string;
  force?: boolean;
}

export interface RagReindexResult {
  indexed: number;
  collection: string;
  duration_ms: number;
}

export interface RagExportInput {
  query: string;
  format?: 'json' | 'markdown' | 'csv';
  k?: number;
}

export interface RagExportResult {
  content: string;
  format: string;
  resultCount: number;
}

export interface RagConfig {
  qmdUrl: string;
  qmdCollection: string;
  doclingEnabled: boolean;
  ingestedDir: string;
  rrfK: number;
  sonaEnabled: boolean;
  searchTimeout: number;
}

export const DEFAULT_RAG_CONFIG: RagConfig = {
  qmdUrl: 'http://localhost:8181',
  qmdCollection: 'rlmx',
  doclingEnabled: true,
  ingestedDir: 'docs/ingested',
  rrfK: 60,
  sonaEnabled: true,
  searchTimeout: 5000,
};
