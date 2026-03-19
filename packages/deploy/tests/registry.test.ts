import { describe, it, expect, beforeEach } from 'vitest';
import { LifeDomain, AgentType, SyscallPermission } from '@aix/shared';
import { ManifestRegistry } from '../src/registry.js';
import type { AgentManifest } from '../src/manifest.js';

function makeValidManifest(overrides?: Partial<AgentManifest>): AgentManifest {
  return {
    id: 'test-agent-001',
    name: 'Test Agent',
    version: '1.0.0',
    origin: { type: 'rlmx', agentType: AgentType.Worker, permissions: [SyscallPermission.VecSearch] },
    lifeDomain: LifeDomain.Finance,
    domainTags: ['investment', 'trading'],
    capabilities: [{ name: 'search', required: true }],
    modalities: ['text'],
    resourceEnvelope: { cpuCores: 2, memoryMb: 512, diskMb: 1024, maxRuntimeMs: 60000 },
    transports: [{ type: 'mcp', endpoint: 'http://localhost:3000' }],
    security: { authMethod: 'bearer' },
    deployment: { profiles: ['rlmx-edge'] },
    metadata: {},
    ...overrides,
  };
}

describe('ManifestRegistry', () => {
  let registry: ManifestRegistry;

  beforeEach(() => {
    registry = new ManifestRegistry();
  });

  // -------------------------------------------------------------------------
  // Register & get
  // -------------------------------------------------------------------------

  it('should register a manifest and retrieve it by id', () => {
    const m = makeValidManifest();
    registry.register(m);
    expect(registry.get(m.id)).toEqual(m);
  });

  it('should return undefined for an unregistered id via get', () => {
    expect(registry.get('nonexistent')).toBeUndefined();
  });

  // -------------------------------------------------------------------------
  // Duplicate registration
  // -------------------------------------------------------------------------

  it('should throw on duplicate registration', () => {
    const m = makeValidManifest();
    registry.register(m);
    expect(() => registry.register(m)).toThrow();
  });

  // -------------------------------------------------------------------------
  // getOrThrow
  // -------------------------------------------------------------------------

  it('should throw on getOrThrow for missing manifest', () => {
    expect(() => registry.getOrThrow('missing-id')).toThrow();
  });

  it('should return manifest from getOrThrow when it exists', () => {
    const m = makeValidManifest();
    registry.register(m);
    expect(registry.getOrThrow(m.id)).toEqual(m);
  });

  // -------------------------------------------------------------------------
  // Update
  // -------------------------------------------------------------------------

  it('should update an existing manifest', () => {
    const m = makeValidManifest();
    registry.register(m);
    const updated = { ...m, name: 'Updated Agent', version: '2.0.0' };
    registry.update(m.id, updated);
    expect(registry.get(m.id)?.name).toBe('Updated Agent');
    expect(registry.get(m.id)?.version).toBe('2.0.0');
  });

  // -------------------------------------------------------------------------
  // Remove
  // -------------------------------------------------------------------------

  it('should remove a manifest', () => {
    const m = makeValidManifest();
    registry.register(m);
    registry.remove(m.id);
    expect(registry.get(m.id)).toBeUndefined();
  });

  // -------------------------------------------------------------------------
  // List
  // -------------------------------------------------------------------------

  it('should list all registered manifests', () => {
    const m1 = makeValidManifest({ id: 'a1' });
    const m2 = makeValidManifest({ id: 'a2', lifeDomain: LifeDomain.Health });
    registry.register(m1);
    registry.register(m2);
    const all = registry.list();
    expect(all).toHaveLength(2);
    expect(all.map((m) => m.id).sort()).toEqual(['a1', 'a2']);
  });

  // -------------------------------------------------------------------------
  // Search by domain
  // -------------------------------------------------------------------------

  it('should search by domain', () => {
    registry.register(makeValidManifest({ id: 'fin-1', lifeDomain: LifeDomain.Finance }));
    registry.register(makeValidManifest({ id: 'health-1', lifeDomain: LifeDomain.Health }));
    const results = registry.search({ domain: LifeDomain.Finance });
    expect(results).toHaveLength(1);
    expect(results[0].id).toBe('fin-1');
  });

  // -------------------------------------------------------------------------
  // Search by origin type
  // -------------------------------------------------------------------------

  it('should search by origin type', () => {
    registry.register(makeValidManifest({ id: 'rlmx-1' }));
    registry.register(makeValidManifest({
      id: 'ext-1',
      origin: { type: 'external', protocol: 'openai', endpoint: 'https://api.openai.com' },
    }));
    const results = registry.search({ origin: 'external' });
    expect(results).toHaveLength(1);
    expect(results[0].id).toBe('ext-1');
  });

  // -------------------------------------------------------------------------
  // Search by modality
  // -------------------------------------------------------------------------

  it('should search by modality', () => {
    registry.register(makeValidManifest({ id: 'txt-1', modalities: ['text'] }));
    registry.register(makeValidManifest({ id: 'voice-1', modalities: ['voice', 'text'] }));
    const results = registry.search({ modality: 'voice' });
    expect(results).toHaveLength(1);
    expect(results[0].id).toBe('voice-1');
  });

  // -------------------------------------------------------------------------
  // Search by tag
  // -------------------------------------------------------------------------

  it('should search by tag', () => {
    registry.register(makeValidManifest({ id: 't1', domainTags: ['investment', 'trading'] }));
    registry.register(makeValidManifest({ id: 't2', domainTags: ['medical', 'diagnostics'] }));
    const results = registry.search({ tag: 'investment' });
    expect(results).toHaveLength(1);
    expect(results[0].id).toBe('t1');
  });

  // -------------------------------------------------------------------------
  // Search by nameContains
  // -------------------------------------------------------------------------

  it('should search by nameContains (case-insensitive)', () => {
    registry.register(makeValidManifest({ id: 'n1', name: 'Finance Helper' }));
    registry.register(makeValidManifest({ id: 'n2', name: 'Health Monitor' }));
    const results = registry.search({ nameContains: 'finance' });
    expect(results).toHaveLength(1);
    expect(results[0].id).toBe('n1');
  });

  // -------------------------------------------------------------------------
  // Combined search filters (AND logic)
  // -------------------------------------------------------------------------

  it('should apply combined search filters with AND logic', () => {
    registry.register(makeValidManifest({
      id: 'combo-1',
      lifeDomain: LifeDomain.Finance,
      modalities: ['text', 'voice'],
      domainTags: ['trading'],
    }));
    registry.register(makeValidManifest({
      id: 'combo-2',
      lifeDomain: LifeDomain.Finance,
      modalities: ['text'],
      domainTags: ['banking'],
    }));
    registry.register(makeValidManifest({
      id: 'combo-3',
      lifeDomain: LifeDomain.Health,
      modalities: ['voice'],
      domainTags: ['trading'],
    }));

    const results = registry.search({
      domain: LifeDomain.Finance,
      modality: 'voice',
      tag: 'trading',
    });
    expect(results).toHaveLength(1);
    expect(results[0].id).toBe('combo-1');
  });

  // -------------------------------------------------------------------------
  // Clear
  // -------------------------------------------------------------------------

  it('should clear all manifests', () => {
    registry.register(makeValidManifest({ id: 'c1' }));
    registry.register(makeValidManifest({ id: 'c2' }));
    registry.clear();
    expect(registry.list()).toHaveLength(0);
    expect(registry.size).toBe(0);
  });

  // -------------------------------------------------------------------------
  // size and has
  // -------------------------------------------------------------------------

  it('should report correct size', () => {
    expect(registry.size).toBe(0);
    registry.register(makeValidManifest({ id: 's1' }));
    expect(registry.size).toBe(1);
    registry.register(makeValidManifest({ id: 's2' }));
    expect(registry.size).toBe(2);
    registry.remove('s1');
    expect(registry.size).toBe(1);
  });

  it('should report has correctly', () => {
    expect(registry.has('test-agent-001')).toBe(false);
    registry.register(makeValidManifest());
    expect(registry.has('test-agent-001')).toBe(true);
    registry.remove('test-agent-001');
    expect(registry.has('test-agent-001')).toBe(false);
  });
});
