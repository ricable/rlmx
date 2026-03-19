import { describe, it, expect } from 'vitest';
import { LifeDomain, AgentType, SyscallPermission } from '@aix/shared';
import { createManifest, ALL_MODALITIES, TRANSPORT_TYPES } from '../src/manifest.js';
import type { AgentManifest, AgentOrigin, TransportSpec } from '../src/manifest.js';

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

// ---------------------------------------------------------------------------
// createManifest helper
// ---------------------------------------------------------------------------

describe('createManifest', () => {
  it('should create a manifest with all required fields', () => {
    const m = createManifest({
      id: 'agent-1',
      name: 'Agent One',
      version: '0.1.0',
      origin: { type: 'rlmx', agentType: AgentType.Worker, permissions: [SyscallPermission.VecSearch] },
      lifeDomain: LifeDomain.Finance,
      domainTags: ['finance'],
      capabilities: [],
      modalities: ['text'],
      resourceEnvelope: { cpuCores: 1, memoryMb: 256, diskMb: 512, maxRuntimeMs: 30000 },
      transports: [{ type: 'mcp', endpoint: 'http://localhost:3000' }],
      security: { authMethod: 'bearer' },
      deployment: { profiles: ['local'] },
    });

    expect(m.id).toBe('agent-1');
    expect(m.name).toBe('Agent One');
    expect(m.version).toBe('0.1.0');
    expect(m.metadata).toEqual({});
  });

  it('should default metadata to empty object when omitted', () => {
    const m = createManifest({
      id: 'a', name: 'A', version: '1.0.0',
      origin: { type: 'custom', handler: 'h' },
      lifeDomain: LifeDomain.Health,
      domainTags: [],
      capabilities: [],
      modalities: ['text'],
      resourceEnvelope: { cpuCores: 1, memoryMb: 128, diskMb: 64, maxRuntimeMs: 5000 },
      transports: [{ type: 'rest', baseUrl: 'http://example.com' }],
      security: { authMethod: 'none' },
      deployment: { profiles: ['edge'] },
    });
    expect(m.metadata).toEqual({});
  });

  it('should preserve explicit metadata when provided', () => {
    const meta = { author: 'test', priority: 5 };
    const m = createManifest({
      id: 'b', name: 'B', version: '2.0.0',
      origin: { type: 'custom', handler: 'h' },
      lifeDomain: LifeDomain.Education,
      domainTags: [],
      capabilities: [],
      modalities: ['voice'],
      resourceEnvelope: { cpuCores: 1, memoryMb: 128, diskMb: 64, maxRuntimeMs: 5000 },
      transports: [{ type: 'rest', baseUrl: 'http://example.com' }],
      security: { authMethod: 'none' },
      deployment: { profiles: ['cloud'] },
      metadata: meta,
    });
    expect(m.metadata).toEqual(meta);
  });
});

// ---------------------------------------------------------------------------
// Agent Origin discriminated union
// ---------------------------------------------------------------------------

describe('AgentOrigin discriminated union', () => {
  it('should support rlmx origin with agentType and permissions', () => {
    const origin: AgentOrigin = {
      type: 'rlmx',
      agentType: AgentType.Coordinator,
      permissions: [SyscallPermission.ProcessFork, SyscallPermission.ProcessSend],
    };
    expect(origin.type).toBe('rlmx');
    expect(origin.agentType).toBe(AgentType.Coordinator);
    expect(origin.permissions).toHaveLength(2);
  });

  it('should support seed origin with mcpEndpoint and sensorProfile', () => {
    const origin: AgentOrigin = {
      type: 'seed',
      mcpEndpoint: 'http://seed:3000',
      sensorProfile: 'greenhouse-v1',
    };
    expect(origin.type).toBe('seed');
    expect(origin.mcpEndpoint).toBe('http://seed:3000');
    expect(origin.sensorProfile).toBe('greenhouse-v1');
  });

  it('should support seed origin with optional restEndpoint', () => {
    const origin: AgentOrigin = {
      type: 'seed',
      mcpEndpoint: 'http://seed:3000',
      sensorProfile: 'soil-v2',
      restEndpoint: 'http://seed:8080',
    };
    expect(origin.restEndpoint).toBe('http://seed:8080');
  });

  it('should support external origin with protocol and endpoint', () => {
    const origin: AgentOrigin = {
      type: 'external',
      protocol: 'openai-compat',
      endpoint: 'https://api.openai.com/v1',
    };
    expect(origin.type).toBe('external');
    expect(origin.protocol).toBe('openai-compat');
    expect(origin.endpoint).toBe('https://api.openai.com/v1');
  });

  it('should support custom origin with handler', () => {
    const origin: AgentOrigin = { type: 'custom', handler: 'my-custom-handler' };
    expect(origin.type).toBe('custom');
    expect(origin.handler).toBe('my-custom-handler');
  });
});

// ---------------------------------------------------------------------------
// Modalities
// ---------------------------------------------------------------------------

describe('ALL_MODALITIES', () => {
  it('should contain exactly 6 modality values', () => {
    expect(ALL_MODALITIES).toHaveLength(6);
  });

  it.each(['text', 'voice', 'vision', 'sensor', 'haptic', 'multimodal'] as const)(
    'should include modality "%s"',
    (mod) => {
      expect(ALL_MODALITIES).toContain(mod);
    },
  );
});

// ---------------------------------------------------------------------------
// TransportSpec variants
// ---------------------------------------------------------------------------

describe('TransportSpec variants', () => {
  it('should contain all 7 transport types', () => {
    expect(TRANSPORT_TYPES).toHaveLength(7);
  });

  it('should support mcp transport', () => {
    const t: TransportSpec = { type: 'mcp', endpoint: 'http://localhost:3000' };
    expect(t.type).toBe('mcp');
  });

  it('should support rest transport', () => {
    const t: TransportSpec = { type: 'rest', baseUrl: 'http://api.example.com' };
    expect(t.type).toBe('rest');
    expect(t.baseUrl).toBe('http://api.example.com');
  });

  it('should support ws transport', () => {
    const t: TransportSpec = { type: 'ws', url: 'ws://localhost:3001' };
    expect(t.type).toBe('ws');
  });

  it('should support quic transport', () => {
    const t: TransportSpec = { type: 'quic', addr: '0.0.0.0:4433' };
    expect(t.type).toBe('quic');
  });

  it('should support mqtt transport', () => {
    const t: TransportSpec = { type: 'mqtt', broker: 'mqtt://broker:1883', topic: 'sensors/temp' };
    expect(t.type).toBe('mqtt');
    expect(t.topic).toBe('sensors/temp');
  });

  it('should support grpc transport', () => {
    const t: TransportSpec = { type: 'grpc', endpoint: 'grpc://localhost:50051' };
    expect(t.type).toBe('grpc');
  });

  it('should support broadcast-channel transport', () => {
    const t: TransportSpec = { type: 'broadcast-channel', channel: 'rlmx-sync' };
    expect(t.type).toBe('broadcast-channel');
    expect(t.channel).toBe('rlmx-sync');
  });
});

// ---------------------------------------------------------------------------
// ResourceEnvelope
// ---------------------------------------------------------------------------

describe('ResourceEnvelope', () => {
  it('should store all resource fields', () => {
    const m = makeValidManifest({
      resourceEnvelope: { cpuCores: 4, memoryMb: 2048, diskMb: 8192, maxRuntimeMs: 120000 },
    });
    expect(m.resourceEnvelope.cpuCores).toBe(4);
    expect(m.resourceEnvelope.memoryMb).toBe(2048);
    expect(m.resourceEnvelope.diskMb).toBe(8192);
    expect(m.resourceEnvelope.maxRuntimeMs).toBe(120000);
  });

  it('should support optional gpuType', () => {
    const m = makeValidManifest({
      resourceEnvelope: { cpuCores: 2, memoryMb: 512, diskMb: 1024, maxRuntimeMs: 60000, gpuType: 'metal' },
    });
    expect(m.resourceEnvelope.gpuType).toBe('metal');
  });

  it('should default gpuType to undefined when not set', () => {
    const m = makeValidManifest();
    expect(m.resourceEnvelope.gpuType).toBeUndefined();
  });
});

// ---------------------------------------------------------------------------
// SecuritySpec
// ---------------------------------------------------------------------------

describe('SecuritySpec', () => {
  it.each(['bearer', 'mtls', 'macaroon', 'none'] as const)(
    'should accept authMethod "%s"',
    (method) => {
      const m = makeValidManifest({ security: { authMethod: method } });
      expect(m.security.authMethod).toBe(method);
    },
  );

  it('should support optional attestation', () => {
    const m = makeValidManifest({
      security: { authMethod: 'bearer', attestation: 'ed25519' },
    });
    expect(m.security.attestation).toBe('ed25519');
  });

  it('should support optional sandboxProfile', () => {
    const m = makeValidManifest({
      security: { authMethod: 'macaroon', sandboxProfile: 'restricted-worker' },
    });
    expect(m.security.sandboxProfile).toBe('restricted-worker');
  });

  it('should support optional rateLimit', () => {
    const m = makeValidManifest({
      security: { authMethod: 'bearer', rateLimit: { maxRequestsPerMinute: 100 } },
    });
    expect(m.security.rateLimit?.maxRequestsPerMinute).toBe(100);
  });
});

// ---------------------------------------------------------------------------
// SensorSpec
// ---------------------------------------------------------------------------

describe('SensorSpec', () => {
  it('should store sensor interfaces', () => {
    const m = makeValidManifest({
      sensorConfig: { interfaces: ['gpio', 'i2c', 'spi'] },
    });
    expect(m.sensorConfig?.interfaces).toEqual(['gpio', 'i2c', 'spi']);
  });

  it('should support optional drift detection', () => {
    const m = makeValidManifest({
      sensorConfig: { interfaces: ['uart'], driftDetection: true, samplingIntervalMs: 500 },
    });
    expect(m.sensorConfig?.driftDetection).toBe(true);
    expect(m.sensorConfig?.samplingIntervalMs).toBe(500);
  });

  it('should support optional calibration flag', () => {
    const m = makeValidManifest({
      sensorConfig: { interfaces: ['usb'], calibrationRequired: true },
    });
    expect(m.sensorConfig?.calibrationRequired).toBe(true);
  });
});

// ---------------------------------------------------------------------------
// LearningSpec
// ---------------------------------------------------------------------------

describe('LearningSpec', () => {
  it('should store SONA and federation flags', () => {
    const m = makeValidManifest({
      learning: { sona: true, federation: true, ewcLambda: 0.5 },
    });
    expect(m.learning?.sona).toBe(true);
    expect(m.learning?.federation).toBe(true);
    expect(m.learning?.ewcLambda).toBe(0.5);
  });

  it('should support drift detectors list', () => {
    const m = makeValidManifest({
      learning: { driftDetectors: ['ks-test', 'psi'] },
    });
    expect(m.learning?.driftDetectors).toEqual(['ks-test', 'psi']);
  });
});

// ---------------------------------------------------------------------------
// DeploymentSpec
// ---------------------------------------------------------------------------

describe('DeploymentSpec', () => {
  it('should store deployment profiles', () => {
    const m = makeValidManifest({
      deployment: { profiles: ['rlmx-edge', 'rlmx-cloud'] },
    });
    expect(m.deployment.profiles).toEqual(['rlmx-edge', 'rlmx-cloud']);
  });

  it('should support optional systemdUnit', () => {
    const m = makeValidManifest({
      deployment: { profiles: ['rpi5'], systemdUnit: 'rlmx-agent.service' },
    });
    expect(m.deployment.systemdUnit).toBe('rlmx-agent.service');
  });

  it('should support optional containerImage', () => {
    const m = makeValidManifest({
      deployment: { profiles: ['cloud'], containerImage: 'ghcr.io/rlmx/agent:latest' },
    });
    expect(m.deployment.containerImage).toBe('ghcr.io/rlmx/agent:latest');
  });

  it('should support optional rvfPath', () => {
    const m = makeValidManifest({
      deployment: { profiles: ['edge'], rvfPath: '/agents/worker.rvf' },
    });
    expect(m.deployment.rvfPath).toBe('/agents/worker.rvf');
  });
});

// ---------------------------------------------------------------------------
// Round-trip serialization
// ---------------------------------------------------------------------------

describe('Round-trip serialization', () => {
  it('should preserve all fields through JSON round-trip', () => {
    const original = makeValidManifest({
      discovery: { serviceType: '_rlmx._tcp', dnsSd: true, port: 5353, txtRecords: { v: '1' } },
      sensorConfig: { interfaces: ['gpio', 'i2c'], driftDetection: true, samplingIntervalMs: 100, calibrationRequired: false },
      learning: { sona: true, federation: false, driftDetectors: ['ks'], ewcLambda: 0.3 },
      metadata: { author: 'test', tags: [1, 2, 3] },
    });

    const json = JSON.stringify(original);
    const parsed: AgentManifest = JSON.parse(json);

    expect(parsed).toEqual(original);
  });

  it('should preserve rlmx origin through round-trip', () => {
    const m = makeValidManifest();
    const parsed: AgentManifest = JSON.parse(JSON.stringify(m));
    expect(parsed.origin).toEqual(m.origin);
  });

  it('should preserve seed origin through round-trip', () => {
    const m = makeValidManifest({
      origin: { type: 'seed', mcpEndpoint: 'http://seed:3000', sensorProfile: 'soil' },
    });
    const parsed: AgentManifest = JSON.parse(JSON.stringify(m));
    expect(parsed.origin).toEqual(m.origin);
  });

  it('should preserve external origin through round-trip', () => {
    const m = makeValidManifest({
      origin: { type: 'external', protocol: 'openai', endpoint: 'https://api.openai.com' },
    });
    const parsed: AgentManifest = JSON.parse(JSON.stringify(m));
    expect(parsed.origin).toEqual(m.origin);
  });
});

// ---------------------------------------------------------------------------
// Missing optional fields
// ---------------------------------------------------------------------------

describe('Missing optional fields', () => {
  it('should allow undefined discovery', () => {
    const m = makeValidManifest();
    expect(m.discovery).toBeUndefined();
  });

  it('should allow undefined sensorConfig', () => {
    const m = makeValidManifest();
    expect(m.sensorConfig).toBeUndefined();
  });

  it('should allow undefined learning', () => {
    const m = makeValidManifest();
    expect(m.learning).toBeUndefined();
  });

  it('should allow empty domainTags', () => {
    const m = makeValidManifest({ domainTags: [] });
    expect(m.domainTags).toEqual([]);
  });

  it('should allow empty capabilities', () => {
    const m = makeValidManifest({ capabilities: [] });
    expect(m.capabilities).toEqual([]);
  });
});
