import { describe, it, expect } from 'vitest';
import { createAllRagTools } from '../src/tools.js';

describe('tool definitions', () => {
  const tools = createAllRagTools();

  it('creates exactly 8 tools', () => {
    expect(tools).toHaveLength(8);
  });

  it('all tools have unique names', () => {
    const names = tools.map(t => t.definition.name);
    expect(new Set(names).size).toBe(names.length);
  });

  it('all tools have descriptions', () => {
    for (const tool of tools) {
      expect(tool.definition.description).toBeTruthy();
      expect(tool.definition.description.length).toBeGreaterThan(10);
    }
  });

  it('all tools have input schemas', () => {
    for (const tool of tools) {
      expect(tool.definition.inputSchema).toBeDefined();
      expect(tool.definition.inputSchema.type).toBe('object');
    }
  });

  it('all tools have handler functions', () => {
    for (const tool of tools) {
      expect(typeof tool.handler).toBe('function');
    }
  });

  it('rag_search requires query', () => {
    const search = tools.find(t => t.definition.name === 'rag_search')!;
    const schema = search.definition.inputSchema as { required?: string[] };
    expect(schema.required).toContain('query');
  });

  it('rag_ingest requires file', () => {
    const ingest = tools.find(t => t.definition.name === 'rag_ingest')!;
    const schema = ingest.definition.inputSchema as { required?: string[] };
    expect(schema.required).toContain('file');
  });

  it('rag_compare requires textA and textB', () => {
    const compare = tools.find(t => t.definition.name === 'rag_compare')!;
    const schema = compare.definition.inputSchema as { required?: string[] };
    expect(schema.required).toContain('textA');
    expect(schema.required).toContain('textB');
  });

  it('rag_export supports json, markdown, csv formats', () => {
    const exp = tools.find(t => t.definition.name === 'rag_export')!;
    const schema = exp.definition.inputSchema as { properties?: { format?: { enum?: string[] } } };
    expect(schema.properties?.format?.enum).toEqual(['json', 'markdown', 'csv']);
  });

  it('tool names follow rag_ prefix convention', () => {
    for (const tool of tools) {
      expect(tool.definition.name).toMatch(/^rag_/);
    }
  });

  it('rag_search supports scope parameter', () => {
    const search = tools.find(t => t.definition.name === 'rag_search')!;
    const schema = search.definition.inputSchema as { properties?: { scope?: { enum?: string[] } } };
    expect(schema.properties?.scope?.enum).toEqual(['project', 'all']);
  });

  it('rag_provenance requires id', () => {
    const prov = tools.find(t => t.definition.name === 'rag_provenance')!;
    const schema = prov.definition.inputSchema as { required?: string[] };
    expect(schema.required).toContain('id');
  });
});
