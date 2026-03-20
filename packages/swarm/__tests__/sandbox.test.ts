import { describe, it, expect, beforeEach } from 'vitest';
import {
  SandboxManager,
  SandboxState,
  GpuRequirement,
  NetworkPolicy,
  validTransition,
  type SandboxProfile,
  type FleetManifest,
} from '../src/index.js';

function workerProfile(name = 'test-worker'): SandboxProfile {
  return {
    name,
    agentType: 'Worker',
    resources: {
      cpuCores: 4,
      memoryMb: 8192,
      gpu: GpuRequirement.None,
      diskMb: 10240,
      maxRuntimeMs: 3600_000,
    },
    network: NetworkPolicy.ClusterOnly,
    zonePreference: 'zone-b',
    modelTier: 'Small',
    crateFeatures: [],
  };
}

function researcherProfile(name = 'fleet-researcher'): SandboxProfile {
  return {
    name,
    agentType: 'Researcher',
    resources: {
      cpuCores: 8,
      memoryMb: 32768,
      gpu: GpuRequirement.Metal,
      diskMb: 51200,
      maxRuntimeMs: 86400_000,
    },
    network: NetworkPolicy.EgressOnly,
    zonePreference: 'zone-a',
    modelTier: 'Medium',
    crateFeatures: ['ruvllm', 'metal'],
  };
}

describe('validTransition', () => {
  it('should allow valid forward transitions', () => {
    expect(validTransition(SandboxState.Provisioning, SandboxState.Starting)).toBe(true);
    expect(validTransition(SandboxState.Starting, SandboxState.Running)).toBe(true);
    expect(validTransition(SandboxState.Running, SandboxState.Suspended)).toBe(true);
    expect(validTransition(SandboxState.Suspended, SandboxState.Running)).toBe(true);
    expect(validTransition(SandboxState.Running, SandboxState.Stopping)).toBe(true);
    expect(validTransition(SandboxState.Stopping, SandboxState.Terminated)).toBe(true);
  });

  it('should allow failure transitions', () => {
    expect(validTransition(SandboxState.Provisioning, SandboxState.Failed)).toBe(true);
    expect(validTransition(SandboxState.Starting, SandboxState.Failed)).toBe(true);
    expect(validTransition(SandboxState.Running, SandboxState.Failed)).toBe(true);
    expect(validTransition(SandboxState.Stopping, SandboxState.Failed)).toBe(true);
  });

  it('should reject invalid transitions', () => {
    expect(validTransition(SandboxState.Provisioning, SandboxState.Running)).toBe(false);
    expect(validTransition(SandboxState.Terminated, SandboxState.Running)).toBe(false);
    expect(validTransition(SandboxState.Failed, SandboxState.Running)).toBe(false);
  });
});

describe('SandboxManager', () => {
  let mgr: SandboxManager;

  beforeEach(() => {
    mgr = new SandboxManager();
  });

  it('should register and spawn a sandbox', () => {
    mgr.registerProfile(workerProfile());
    expect(mgr.listProfiles()).toHaveLength(1);

    const id = mgr.spawn('test-worker');
    const instance = mgr.status(id);
    expect(instance.state).toBe(SandboxState.Provisioning);
    expect(instance.profile.name).toBe('test-worker');
  });

  it('should reject duplicate profile registration', () => {
    mgr.registerProfile(workerProfile());
    expect(() => mgr.registerProfile(workerProfile())).toThrow(/duplicate/i);
  });

  it('should reject spawning unknown profile', () => {
    expect(() => mgr.spawn('nonexistent')).toThrow(/profile not found/i);
  });

  it('should perform full lifecycle transitions', () => {
    mgr.registerProfile(workerProfile('lifecycle-test'));
    const id = mgr.spawn('lifecycle-test');

    mgr.transition(id, SandboxState.Starting);
    mgr.transition(id, SandboxState.Running);
    mgr.transition(id, SandboxState.Stopping);
    mgr.transition(id, SandboxState.Terminated);

    expect(mgr.status(id).state).toBe(SandboxState.Terminated);
  });

  it('should reject invalid state transitions', () => {
    mgr.registerProfile(workerProfile('invalid-trans'));
    const id = mgr.spawn('invalid-trans');
    // Provisioning -> Running is invalid (must go through Starting)
    expect(() => mgr.transition(id, SandboxState.Running)).toThrow(/invalid state transition/i);
  });

  it('should terminate a running sandbox', () => {
    mgr.registerProfile(workerProfile('term-test'));
    const id = mgr.spawn('term-test');
    mgr.transition(id, SandboxState.Starting);
    mgr.transition(id, SandboxState.Running);

    mgr.terminate(id);
    expect(mgr.status(id).state).toBe(SandboxState.Terminated);
  });

  it('should no-op when terminating already terminated sandbox', () => {
    mgr.registerProfile(workerProfile('term-noop'));
    const id = mgr.spawn('term-noop');
    mgr.transition(id, SandboxState.Starting);
    mgr.transition(id, SandboxState.Running);
    mgr.terminate(id);
    // Should not throw
    mgr.terminate(id);
    expect(mgr.status(id).state).toBe(SandboxState.Terminated);
  });

  it('should count instances by state', () => {
    mgr.registerProfile(workerProfile('count-test'));
    mgr.spawn('count-test');
    mgr.spawn('count-test');
    expect(mgr.countByState(SandboxState.Provisioning)).toBe(2);
    expect(mgr.countByState(SandboxState.Running)).toBe(0);
  });

  it('should list all instances', () => {
    mgr.registerProfile(workerProfile('list-test'));
    mgr.spawn('list-test');
    mgr.spawn('list-test');
    expect(mgr.listInstances()).toHaveLength(2);
  });

  it('should deploy a fleet manifest', () => {
    mgr.registerProfile(workerProfile('fleet-worker'));
    mgr.registerProfile(researcherProfile());

    const manifest: FleetManifest = {
      name: 'test-fleet',
      version: '1.0',
      sandboxes: [
        { profile: 'fleet-worker', count: 3 },
        { profile: 'fleet-researcher', count: 1 },
      ],
    };

    const ids = mgr.deployFleet(manifest);
    expect(ids).toHaveLength(4);
    expect(mgr.listInstances()).toHaveLength(4);
  });

  it('should reject fleet with missing profile', () => {
    const manifest: FleetManifest = {
      name: 'bad-fleet',
      version: '1.0',
      sandboxes: [{ profile: 'missing', count: 1 }],
    };
    expect(() => mgr.deployFleet(manifest)).toThrow(/profile not found/i);
  });

  it('should reject empty fleet manifest', () => {
    const manifest: FleetManifest = {
      name: 'empty',
      version: '1.0',
      sandboxes: [],
    };
    expect(() => mgr.validateFleet(manifest)).toThrow(/no sandboxes/i);
  });

  it('should reject fleet with zero count', () => {
    mgr.registerProfile(workerProfile('zero-count'));
    const manifest: FleetManifest = {
      name: 'zero',
      version: '1.0',
      sandboxes: [{ profile: 'zero-count', count: 0 }],
    };
    expect(() => mgr.validateFleet(manifest)).toThrow(/count 0/);
  });

  it('should throw on status of unknown sandbox', () => {
    expect(() => mgr.status('nonexistent')).toThrow(/sandbox not found/i);
  });
});
