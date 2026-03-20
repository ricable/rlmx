import { describe, it, expect } from 'vitest';
import { BackendRegistry } from '../src/backends/backend.js';
import { QmdBackend } from '../src/backends/qmd.js';
import { DoclingBackend } from '../src/backends/docling.js';
import { MemoryBackend } from '../src/backends/memory.js';

describe('BackendRegistry', () => {
  it('registers and retrieves backends', () => {
    const registry = new BackendRegistry();
    const qmd = new QmdBackend('http://localhost:8181', 'test');
    registry.register(qmd);
    expect(registry.get('qmd')).toBe(qmd);
    expect(registry.names()).toEqual(['qmd']);
  });

  it('returns undefined for unknown backend', () => {
    const registry = new BackendRegistry();
    expect(registry.get('unknown')).toBeUndefined();
  });

  it('lists all backends', () => {
    const registry = new BackendRegistry();
    registry.register(new QmdBackend('http://localhost:8181', 'test'));
    registry.register(new MemoryBackend());
    expect(registry.all()).toHaveLength(2);
  });
});

describe('QmdBackend', () => {
  it('has correct name', () => {
    const qmd = new QmdBackend('http://localhost:8181', 'test');
    expect(qmd.name).toBe('qmd');
  });

  it('returns unavailable when server is not running', async () => {
    const qmd = new QmdBackend('http://localhost:19999', 'test');
    const health = await qmd.health();
    expect(health.status).toBe('unavailable');
  });

  it('returns empty results on search failure', async () => {
    const qmd = new QmdBackend('http://localhost:19999', 'test');
    const results = await qmd.search('test', 10);
    expect(results).toEqual([]);
  });
});

describe('DoclingBackend', () => {
  it('has correct name', () => {
    const docling = new DoclingBackend('/tmp/test-ingested');
    expect(docling.name).toBe('docling');
  });

  it('returns empty results for search (ingest-only backend)', async () => {
    const docling = new DoclingBackend('/tmp/test-ingested');
    const results = await docling.search('test', 10);
    expect(results).toEqual([]);
  });

  it('supports ingest', async () => {
    const docling = new DoclingBackend('/tmp/test-ingested');
    expect(docling.ingest).toBeDefined();
  });
});

describe('MemoryBackend', () => {
  it('reports unavailable health', async () => {
    const mem = new MemoryBackend();
    const health = await mem.health();
    expect(health.status).toBe('unavailable');
  });

  it('returns empty search results', async () => {
    const mem = new MemoryBackend();
    const results = await mem.search('test', 10);
    expect(results).toEqual([]);
  });
});
