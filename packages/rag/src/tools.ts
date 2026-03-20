import path from 'path';
import type { RagConfig, RagSearchInput, RagSearchResult, RagStatsResult } from './types.js';
import type { BackendRegistry } from './backends/backend.js';
import { reciprocalRankFusion } from './merge/rrf.js';
import { getAdaptiveWeights } from './merge/sona.js';
import { ProvenanceTracker } from './provenance.js';
import { searchFailed, ingestFailed, provenanceNotFound, exportFailed } from './errors.js';

export interface RagToolContext {
  config: RagConfig;
  registry: BackendRegistry;
  provenance: ProvenanceTracker;
}

export interface RagToolDefinition {
  name: string;
  description: string;
  inputSchema: Record<string, unknown>;
}

export interface RegisteredRagTool {
  definition: RagToolDefinition;
  handler: (args: Record<string, unknown>, ctx: RagToolContext) => Promise<unknown>;
}

// ---------------------------------------------------------------------------
// Shared search logic (used by rag_search, rag_cluster, rag_export)
// ---------------------------------------------------------------------------
async function executeSearch(input: RagSearchInput, ctx: RagToolContext): Promise<RagSearchResult[]> {
  const adapters = input.backends
    ? ctx.registry.all().filter(a => input.backends!.includes(a.name))
    : ctx.registry.all();

  const results = await Promise.allSettled(
    adapters.map(async a => ({
      backend: a.name,
      results: await a.search(input.query, (input.k ?? 10) * 2),
    })),
  );

  const fulfilled = results
    .filter((r): r is PromiseFulfilledResult<{ backend: string; results: RagSearchResult[] }> => r.status === 'fulfilled')
    .map(r => r.value);

  if (fulfilled.length === 0) {
    throw searchFailed('All backends failed');
  }

  const weights = ctx.config.sonaEnabled
    ? await getAdaptiveWeights(fulfilled.map(f => f.backend))
    : new Map(fulfilled.map(f => [f.backend, 1.0] as const));

  const merged = reciprocalRankFusion(fulfilled, weights, ctx.config.rrfK);
  return merged.slice(0, input.k ?? 10);
}

// ---------------------------------------------------------------------------
// Tool: rag_search
// ---------------------------------------------------------------------------
function createRagSearch(): RegisteredRagTool {
  return {
    definition: {
      name: 'rag_search',
      description: 'Unified search across all RAG backends (qmd, docling, memory). Uses Reciprocal Rank Fusion to merge results.',
      inputSchema: {
        type: 'object',
        properties: {
          query: { type: 'string', description: 'Search query' },
          scope: { type: 'string', enum: ['project', 'all'], description: 'Search scope (default: project)' },
          backends: { type: 'array', items: { type: 'string' }, description: 'Specific backends to query' },
          k: { type: 'number', description: 'Number of results (default: 10)' },
        },
        required: ['query'],
      },
    },
    handler: async (args, ctx) => {
      const input: RagSearchInput = {
        query: args.query as string,
        scope: (args.scope as 'project' | 'all') ?? 'project',
        backends: args.backends as string[] | undefined,
        k: (args.k as number) ?? 10,
      };
      return { results: await executeSearch(input, ctx) };
    },
  };
}

// ---------------------------------------------------------------------------
// Tool: rag_ingest
// ---------------------------------------------------------------------------
function createRagIngest(): RegisteredRagTool {
  return {
    definition: {
      name: 'rag_ingest',
      description: 'Ingest a document (PDF, DOCX, markdown) into the RAG system. Converts via docling, indexes via qmd.',
      inputSchema: {
        type: 'object',
        properties: {
          file: { type: 'string', description: 'Path to the file to ingest' },
          collection: { type: 'string', description: 'Target collection (default: project collection)' },
          tags: { type: 'array', items: { type: 'string' }, description: 'Tags for the ingested document' },
        },
        required: ['file'],
      },
    },
    handler: async (args, ctx) => {
      const file = args.file as string;
      if (file.includes('..') || path.isAbsolute(file)) {
        return { error: 'Invalid file path: directory traversal not allowed' };
      }
      const collection = (args.collection as string) ?? ctx.config.qmdCollection;
      const docling = ctx.registry.get('docling');
      if (!docling?.ingest) {
        throw ingestFailed(`No ingest-capable backend available for ${file}`);
      }
      const id = ctx.provenance.startChain(file);
      const result = await docling.ingest(file, collection);
      ctx.provenance.record(id, {
        stage: 'conversion',
        source: 'docling',
        timestamp: new Date().toISOString(),
        details: { chunks: result.chunks },
      });
      return { id: result.id, chunks: result.chunks, collection, backend: 'docling' };
    },
  };
}

// ---------------------------------------------------------------------------
// Tool: rag_compare
// ---------------------------------------------------------------------------
function createRagCompare(): RegisteredRagTool {
  return {
    definition: {
      name: 'rag_compare',
      description: 'Compare semantic similarity between two text snippets.',
      inputSchema: {
        type: 'object',
        properties: {
          textA: { type: 'string' },
          textB: { type: 'string' },
        },
        required: ['textA', 'textB'],
      },
    },
    handler: async (args, _ctx) => {
      // Stub: would use kernel embeddings for real similarity
      const a = (args.textA as string).toLowerCase();
      const b = (args.textB as string).toLowerCase();
      const wordsA = new Set(a.split(/\s+/));
      const wordsB = new Set(b.split(/\s+/));
      const intersection = [...wordsA].filter(w => wordsB.has(w)).length;
      const union = new Set([...wordsA, ...wordsB]).size;
      const similarity = union > 0 ? intersection / union : 0;
      return { similarity, method: 'jaccard' };
    },
  };
}

// ---------------------------------------------------------------------------
// Tool: rag_cluster
// ---------------------------------------------------------------------------
function createRagCluster(): RegisteredRagTool {
  return {
    definition: {
      name: 'rag_cluster',
      description: 'Cluster search results by semantic similarity.',
      inputSchema: {
        type: 'object',
        properties: {
          query: { type: 'string' },
          k: { type: 'number', description: 'Number of clusters (default: 3)' },
        },
        required: ['query'],
      },
    },
    handler: async (args, ctx) => {
      const query = args.query as string;
      const k = (args.k as number) ?? 3;
      // Search first, then group by source as a simple clustering heuristic
      const searchResults = await executeSearch({ query, k: k * 5 }, ctx);
      const groups = new Map<string, RagSearchResult[]>();
      for (const r of searchResults) {
        const key = r.backend;
        const group = groups.get(key) ?? [];
        group.push(r);
        groups.set(key, group);
      }
      return {
        clusters: [...groups.entries()].map(([label, items]) => ({ label, items })),
      };
    },
  };
}

// ---------------------------------------------------------------------------
// Tool: rag_provenance
// ---------------------------------------------------------------------------
function createRagProvenance(): RegisteredRagTool {
  return {
    definition: {
      name: 'rag_provenance',
      description: 'Retrieve the provenance chain for an ingested document.',
      inputSchema: {
        type: 'object',
        properties: {
          id: { type: 'string', description: 'Document or chunk ID' },
        },
        required: ['id'],
      },
    },
    handler: async (args, ctx) => {
      const id = args.id as string;
      const chain = ctx.provenance.get(id);
      if (!chain) throw provenanceNotFound(id);
      return { chain };
    },
  };
}

// ---------------------------------------------------------------------------
// Tool: rag_stats
// ---------------------------------------------------------------------------
function createRagStats(): RegisteredRagTool {
  return {
    definition: {
      name: 'rag_stats',
      description: 'Get statistics about all RAG backends.',
      inputSchema: { type: 'object', properties: {} },
    },
    handler: async (_args, ctx) => {
      const backends = await Promise.all(
        ctx.registry.all().map(async a => {
          const h = await a.health();
          return { name: a.name, status: h.status, details: h.details };
        }),
      );
      const result: RagStatsResult = {
        backends,
        totalDocuments: 0, // Would query each backend for counts
      };
      return result;
    },
  };
}

// ---------------------------------------------------------------------------
// Tool: rag_reindex
// ---------------------------------------------------------------------------
function createRagReindex(): RegisteredRagTool {
  return {
    definition: {
      name: 'rag_reindex',
      description: 'Trigger reindexing of a collection.',
      inputSchema: {
        type: 'object',
        properties: {
          collection: { type: 'string' },
          force: { type: 'boolean' },
        },
      },
    },
    handler: async (args, ctx) => {
      const collection = (args.collection as string) ?? ctx.config.qmdCollection;
      const start = Date.now();
      // Stub: would trigger qmd collection update
      return { indexed: 0, collection, duration_ms: Date.now() - start };
    },
  };
}

// ---------------------------------------------------------------------------
// Tool: rag_export
// ---------------------------------------------------------------------------
function createRagExport(): RegisteredRagTool {
  return {
    definition: {
      name: 'rag_export',
      description: 'Export search results in a specified format.',
      inputSchema: {
        type: 'object',
        properties: {
          query: { type: 'string' },
          format: { type: 'string', enum: ['json', 'markdown', 'csv'] },
          k: { type: 'number' },
        },
        required: ['query'],
      },
    },
    handler: async (args, ctx) => {
      const query = args.query as string;
      const format = (args.format as string) ?? 'json';
      const k = (args.k as number) ?? 10;

      const results = await executeSearch({ query, k }, ctx);

      let content: string;
      switch (format) {
        case 'markdown':
          content = results.map((r, i) => `## ${i + 1}. ${r.source}\n\n${r.content}\n\n*Score: ${r.score.toFixed(4)}*\n`).join('\n---\n\n');
          break;
        case 'csv':
          content = 'id,score,source,backend,content\n' + results.map(r => `${escapeCsvField(r.id)},${r.score},${escapeCsvField(r.source)},${escapeCsvField(r.backend)},${escapeCsvField(r.content)}`).join('\n');
          break;
        default:
          content = JSON.stringify(results, null, 2);
      }

      return { content, format, resultCount: results.length };
    },
  };
}

// ---------------------------------------------------------------------------
// CSV escaping helper
// ---------------------------------------------------------------------------
function escapeCsvField(value: string): string {
  const needsPrefix = /^[=+\-@\t\r]/.test(value);
  const escaped = value.replace(/"/g, '""');
  if (needsPrefix) return `"'${escaped}"`;
  if (escaped.includes(',') || escaped.includes('"') || escaped.includes('\n')) return `"${escaped}"`;
  return escaped;
}

// ---------------------------------------------------------------------------
// Factory
// ---------------------------------------------------------------------------
export function createAllRagTools(): RegisteredRagTool[] {
  return [
    createRagSearch(),
    createRagIngest(),
    createRagCompare(),
    createRagCluster(),
    createRagProvenance(),
    createRagStats(),
    createRagReindex(),
    createRagExport(),
  ];
}
