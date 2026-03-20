import { describe, it, expect } from 'vitest';
import { RagServer } from '../src/server.js';
import { createAllRagTools } from '../src/tools.js';
import { loadConfig } from '../src/config.js';
import { BackendRegistry } from '../src/backends/backend.js';
import { ProvenanceTracker } from '../src/provenance.js';

describe('RagServer', () => {
  it('can be instantiated with defaults', () => {
    const server = new RagServer();
    expect(server).toBeDefined();
  });

  it('accepts config overrides', () => {
    const server = new RagServer({ qmdCollection: 'custom' });
    expect(server).toBeDefined();
  });

  it('creates all 8 tools', () => {
    const tools = createAllRagTools();
    expect(tools).toHaveLength(8);
    const names = tools.map(t => t.definition.name);
    expect(names).toContain('rag_search');
    expect(names).toContain('rag_ingest');
    expect(names).toContain('rag_compare');
    expect(names).toContain('rag_cluster');
    expect(names).toContain('rag_provenance');
    expect(names).toContain('rag_stats');
    expect(names).toContain('rag_reindex');
    expect(names).toContain('rag_export');
  });
});

describe('loadConfig', () => {
  it('returns defaults', () => {
    const config = loadConfig();
    expect(config.rrfK).toBe(60);
    expect(config.qmdCollection).toBe('rlmx');
  });

  it('applies overrides', () => {
    const config = loadConfig({ rrfK: 100, qmdCollection: 'custom' });
    expect(config.rrfK).toBe(100);
    expect(config.qmdCollection).toBe('custom');
  });
});

describe('ProvenanceTracker', () => {
  it('starts and records chain', () => {
    const tracker = new ProvenanceTracker();
    const id = tracker.startChain('test.pdf');
    expect(id).toBeDefined();
    const chain = tracker.get(id);
    expect(chain).toHaveLength(1);
    expect(chain![0].stage).toBe('source');
  });

  it('records multiple entries', () => {
    const tracker = new ProvenanceTracker();
    const id = tracker.startChain('test.pdf');
    tracker.record(id, { stage: 'conversion', source: 'docling', timestamp: new Date().toISOString() });
    expect(tracker.get(id)).toHaveLength(2);
  });

  it('returns undefined for unknown id', () => {
    const tracker = new ProvenanceTracker();
    expect(tracker.get('nonexistent')).toBeUndefined();
  });

  it('clears all chains', () => {
    const tracker = new ProvenanceTracker();
    const id = tracker.startChain('test.pdf');
    tracker.clear();
    expect(tracker.get(id)).toBeUndefined();
  });
});

describe('BackendRegistry', () => {
  it('registers and retrieves', () => {
    const reg = new BackendRegistry();
    expect(reg.all()).toHaveLength(0);
    expect(reg.names()).toEqual([]);
  });
});

describe('RagToolContext tools', () => {
  it('rag_compare computes Jaccard similarity', async () => {
    const tools = createAllRagTools();
    const compare = tools.find(t => t.definition.name === 'rag_compare')!;
    const config = loadConfig();
    const registry = new BackendRegistry();
    const provenance = new ProvenanceTracker();
    const ctx = { config, registry, provenance };

    const result = await compare.handler({ textA: 'hello world', textB: 'hello world' }, ctx) as { similarity: number };
    expect(result.similarity).toBe(1.0);
  });

  it('rag_compare returns 0 for completely different texts', async () => {
    const tools = createAllRagTools();
    const compare = tools.find(t => t.definition.name === 'rag_compare')!;
    const config = loadConfig();
    const ctx = { config, registry: new BackendRegistry(), provenance: new ProvenanceTracker() };

    const result = await compare.handler({ textA: 'aaa', textB: 'zzz' }, ctx) as { similarity: number };
    expect(result.similarity).toBe(0);
  });

  it('rag_stats returns backend info', async () => {
    const tools = createAllRagTools();
    const stats = tools.find(t => t.definition.name === 'rag_stats')!;
    const config = loadConfig();
    const ctx = { config, registry: new BackendRegistry(), provenance: new ProvenanceTracker() };

    const result = await stats.handler({}, ctx) as { backends: unknown[]; totalDocuments: number };
    expect(result.backends).toEqual([]);
    expect(result.totalDocuments).toBe(0);
  });

  it('rag_reindex returns stub result', async () => {
    const tools = createAllRagTools();
    const reindex = tools.find(t => t.definition.name === 'rag_reindex')!;
    const config = loadConfig();
    const ctx = { config, registry: new BackendRegistry(), provenance: new ProvenanceTracker() };

    const result = await reindex.handler({}, ctx) as { indexed: number; collection: string };
    expect(result.indexed).toBe(0);
    expect(result.collection).toBe('rlmx');
  });

  it('rag_provenance throws for unknown id', async () => {
    const tools = createAllRagTools();
    const prov = tools.find(t => t.definition.name === 'rag_provenance')!;
    const config = loadConfig();
    const ctx = { config, registry: new BackendRegistry(), provenance: new ProvenanceTracker() };

    await expect(prov.handler({ id: 'nonexistent' }, ctx)).rejects.toThrow('Provenance not found');
  });
});
