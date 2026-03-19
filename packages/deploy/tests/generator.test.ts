import { describe, it, expect } from 'vitest';
import {
  generateSystemdUnit,
  generateFleetManifest,
  generateVerificationScript,
  generateDockerCompose,
} from '../src/generator.js';
import { DEPLOYMENT_PROFILES } from '../src/profiles.js';
import { getTemplate, getTemplatesByDomain } from '../src/templates.js';
import { LifeDomain, AgentType } from '@aix/shared';
import type { AgentManifest } from '../src/manifest.js';

function makeManifest(overrides?: Partial<AgentManifest>): AgentManifest {
  return {
    id: 'test-agent',
    name: 'Test Agent',
    version: '1.0.0',
    origin: { type: 'rlmx', agentType: AgentType.Worker, permissions: [] },
    lifeDomain: LifeDomain.Finance,
    domainTags: ['test'],
    capabilities: [],
    modalities: ['text'],
    resourceEnvelope: { cpuCores: 2, memoryMb: 512, diskMb: 1024, maxRuntimeMs: 60000 },
    transports: [{ type: 'mcp', endpoint: 'http://localhost:3000' }],
    security: { authMethod: 'bearer' },
    deployment: { profiles: ['rlmx-edge'] },
    metadata: {},
    ...overrides,
  };
}

describe('generateSystemdUnit', () => {
  it('includes Description', () => {
    const profile = DEPLOYMENT_PROFILES['rlmx-edge'];
    const unit = generateSystemdUnit(makeManifest(), profile);
    expect(unit).toContain('Description=Test Agent (test-agent)');
  });

  it('for seed origin uses cognitum-seed exec', () => {
    const profile = DEPLOYMENT_PROFILES['seed-compatible'];
    const manifest = makeManifest({
      origin: { type: 'seed', mcpEndpoint: 'http://localhost:5353/mcp', sensorProfile: 'temp' },
    });
    const unit = generateSystemdUnit(manifest, profile);
    expect(unit).toContain('cognitum-seed');
  });

  it('for rlmx origin uses aix deploy spawn', () => {
    const profile = DEPLOYMENT_PROFILES['rlmx-edge'];
    const unit = generateSystemdUnit(makeManifest(), profile);
    expect(unit).toContain('aix deploy spawn');
  });

  it('includes MemoryMax when profile has maxMemoryMb', () => {
    const profile = DEPLOYMENT_PROFILES['seed-compatible'];
    const manifest = makeManifest({
      origin: { type: 'seed', mcpEndpoint: 'http://localhost:5353/mcp', sensorProfile: 'temp' },
    });
    const unit = generateSystemdUnit(manifest, profile);
    expect(unit).toContain('MemoryMax=256M');
  });
});

describe('generateFleetManifest', () => {
  it('creates valid fleet manifest', () => {
    const profile = DEPLOYMENT_PROFILES['rlmx-edge'];
    const manifests = [makeManifest(), makeManifest({ id: 'test-agent-2', name: 'Agent 2' })];
    const fleet = generateFleetManifest(manifests, profile);

    expect(fleet.name).toContain('fleet-');
    expect(fleet.version).toBeDefined();
    expect(fleet.sandboxes).toHaveLength(2);
  });

  it('throws on empty manifests', () => {
    const profile = DEPLOYMENT_PROFILES['rlmx-edge'];
    expect(() => generateFleetManifest([], profile)).toThrow();
  });

  it('sets correct sandbox count', () => {
    const profile = DEPLOYMENT_PROFILES['rlmx-edge'];
    const manifests = [makeManifest(), makeManifest({ id: 'a2' }), makeManifest({ id: 'a3' })];
    const fleet = generateFleetManifest(manifests, profile);
    expect(fleet.sandboxes).toHaveLength(3);
    for (const sb of fleet.sandboxes) {
      expect(sb.count).toBe(1);
    }
  });
});

describe('generateVerificationScript', () => {
  it('includes curl health check for seed devices', () => {
    const manifest = makeManifest({
      id: 'temp-sensor',
      name: 'Temp Sensor',
      origin: { type: 'seed', mcpEndpoint: 'http://localhost:5353/mcp', sensorProfile: 'temp', restEndpoint: 'https://192.168.1.50:8443' },
    });
    const script = generateVerificationScript([manifest]);
    expect(script).toContain('curl');
    expect(script).toContain('health');
  });

  it('handles no seed devices', () => {
    const script = generateVerificationScript([makeManifest()]);
    expect(script).toContain('No seed devices to verify');
  });

  it('includes shebang and set flags', () => {
    const script = generateVerificationScript([]);
    expect(script).toContain('#!/usr/bin/env bash');
    expect(script).toContain('set -euo pipefail');
  });
});

describe('generateDockerCompose', () => {
  it('includes service entries', () => {
    const compose = generateDockerCompose([makeManifest()]);
    expect(compose).toContain('services:');
    expect(compose).toContain('test-agent:');
  });

  it('includes memory limits', () => {
    const compose = generateDockerCompose([makeManifest({ resourceEnvelope: { cpuCores: 2, memoryMb: 1024, diskMb: 2048, maxRuntimeMs: 60000 } })]);
    expect(compose).toContain('memory: "1024M"');
  });

  it('uses manifest version in image tag', () => {
    const compose = generateDockerCompose([makeManifest({ version: '2.3.4' })]);
    expect(compose).toContain('aix/test-agent:2.3.4');
  });
});
