import { describe, it, expect } from 'vitest';
import { LifeDomain, AgentType, SyscallPermission } from '@aix/shared';
import { validateManifest } from '../src/validation.js';
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

/** Helper to strip a field entirely from the manifest object. */
function omit<T extends Record<string, unknown>>(obj: T, key: keyof T): Partial<T> {
  const clone = { ...obj };
  delete clone[key];
  return clone;
}

describe('validateManifest', () => {
  // -------------------------------------------------------------------------
  // Valid manifest
  // -------------------------------------------------------------------------

  it('should pass for a valid manifest', () => {
    const result = validateManifest(makeValidManifest());
    expect(result.valid).toBe(true);
    if (result.valid) {
      expect(result.errors).toHaveLength(0);
    }
  });

  // -------------------------------------------------------------------------
  // Missing / invalid id
  // -------------------------------------------------------------------------

  it('should fail when id is missing', () => {
    const m = omit(makeValidManifest(), 'id') as AgentManifest;
    const result = validateManifest(m);
    expect(result.valid).toBe(false);
  });

  it('should fail when id is empty string', () => {
    const result = validateManifest(makeValidManifest({ id: '' }));
    expect(result.valid).toBe(false);
  });

  // -------------------------------------------------------------------------
  // Bad semver
  // -------------------------------------------------------------------------

  it('should fail when version is not valid semver', () => {
    const result = validateManifest(makeValidManifest({ version: 'not-semver' }));
    expect(result.valid).toBe(false);
  });

  it('should pass for valid semver with prerelease', () => {
    const result = validateManifest(makeValidManifest({ version: '1.0.0-alpha.1' }));
    expect(result.valid).toBe(true);
  });

  // -------------------------------------------------------------------------
  // Missing / invalid origin
  // -------------------------------------------------------------------------

  it('should fail when origin is missing', () => {
    const m = omit(makeValidManifest(), 'origin') as AgentManifest;
    const result = validateManifest(m);
    expect(result.valid).toBe(false);
  });

  it('should fail when origin type is invalid', () => {
    const m = makeValidManifest();
    (m.origin as any).type = 'unknown-origin';
    const result = validateManifest(m);
    expect(result.valid).toBe(false);
  });

  // -------------------------------------------------------------------------
  // Origin-specific validation
  // -------------------------------------------------------------------------

  it('should fail when seed origin is missing mcpEndpoint', () => {
    const m = makeValidManifest({
      origin: { type: 'seed', sensorProfile: 'soil' } as any,
    });
    const result = validateManifest(m);
    expect(result.valid).toBe(false);
  });

  it('should fail when rlmx origin is missing agentType', () => {
    const m = makeValidManifest({
      origin: { type: 'rlmx', permissions: [SyscallPermission.VecSearch] } as any,
    });
    const result = validateManifest(m);
    expect(result.valid).toBe(false);
  });

  it('should fail when external origin is missing endpoint', () => {
    const m = makeValidManifest({
      origin: { type: 'external', protocol: 'openai' } as any,
    });
    const result = validateManifest(m);
    expect(result.valid).toBe(false);
  });

  it('should fail when custom origin is missing handler', () => {
    const m = makeValidManifest({
      origin: { type: 'custom' } as any,
    });
    const result = validateManifest(m);
    expect(result.valid).toBe(false);
  });

  // -------------------------------------------------------------------------
  // Modalities
  // -------------------------------------------------------------------------

  it('should fail when modalities array is empty', () => {
    const result = validateManifest(makeValidManifest({ modalities: [] }));
    expect(result.valid).toBe(false);
  });

  it('should fail when modalities contains invalid value', () => {
    const result = validateManifest(makeValidManifest({ modalities: ['text', 'telekinesis' as any] }));
    expect(result.valid).toBe(false);
  });

  // -------------------------------------------------------------------------
  // Transports
  // -------------------------------------------------------------------------

  it('should fail when transports array is empty', () => {
    const result = validateManifest(makeValidManifest({ transports: [] }));
    expect(result.valid).toBe(false);
  });

  it('should fail when transport has invalid type', () => {
    const result = validateManifest(makeValidManifest({
      transports: [{ type: 'carrier-pigeon' as any, endpoint: 'lol' }],
    }));
    expect(result.valid).toBe(false);
  });

  // -------------------------------------------------------------------------
  // ResourceEnvelope
  // -------------------------------------------------------------------------

  it('should fail when resourceEnvelope is missing', () => {
    const m = omit(makeValidManifest(), 'resourceEnvelope') as AgentManifest;
    const result = validateManifest(m);
    expect(result.valid).toBe(false);
  });

  it('should fail when cpuCores is zero', () => {
    const result = validateManifest(makeValidManifest({
      resourceEnvelope: { cpuCores: 0, memoryMb: 512, diskMb: 1024, maxRuntimeMs: 60000 },
    }));
    expect(result.valid).toBe(false);
  });

  it('should fail when cpuCores is negative', () => {
    const result = validateManifest(makeValidManifest({
      resourceEnvelope: { cpuCores: -1, memoryMb: 512, diskMb: 1024, maxRuntimeMs: 60000 },
    }));
    expect(result.valid).toBe(false);
  });

  // -------------------------------------------------------------------------
  // Security
  // -------------------------------------------------------------------------

  it('should fail when security is missing', () => {
    const m = omit(makeValidManifest(), 'security') as AgentManifest;
    const result = validateManifest(m);
    expect(result.valid).toBe(false);
  });

  it('should fail when authMethod is invalid', () => {
    const result = validateManifest(makeValidManifest({
      security: { authMethod: 'plain-text-password' as any },
    }));
    expect(result.valid).toBe(false);
  });

  // -------------------------------------------------------------------------
  // Deployment
  // -------------------------------------------------------------------------

  it('should fail when deployment is missing', () => {
    const m = omit(makeValidManifest(), 'deployment') as AgentManifest;
    const result = validateManifest(m);
    expect(result.valid).toBe(false);
  });

  it('should fail when deployment profiles is empty', () => {
    const result = validateManifest(makeValidManifest({
      deployment: { profiles: [] },
    }));
    expect(result.valid).toBe(false);
  });
});
