import { describe, it, expect, beforeEach } from 'vitest';
import {
  FleetOrchestrator,
  SandboxState,
  GpuRequirement,
  NetworkPolicy,
  type SandboxProfile,
  type FleetManifest,
} from '../src/index.js';

function workerProfile(name = 'worker'): SandboxProfile {
  return {
    name,
    agentType: 'Worker',
    resources: {
      cpuCores: 2,
      memoryMb: 4096,
      gpu: GpuRequirement.None,
      diskMb: 2048,
      maxRuntimeMs: 300_000,
    },
    network: NetworkPolicy.ClusterOnly,
    zonePreference: 'zone-b',
    modelTier: 'Small',
    crateFeatures: [],
  };
}

describe('FleetOrchestrator', () => {
  let orchestrator: FleetOrchestrator;

  beforeEach(() => {
    orchestrator = new FleetOrchestrator();
  });

  it('should deploy and track a fleet', () => {
    orchestrator.registerProfile(workerProfile());
    const manifest: FleetManifest = {
      name: 'my-fleet',
      version: '1.0',
      sandboxes: [{ profile: 'worker', count: 3 }],
    };

    const ids = orchestrator.deployFleet(manifest);
    expect(ids).toHaveLength(3);
    expect(orchestrator.listFleets()).toEqual(['my-fleet']);
  });

  it('should report fleet status', () => {
    orchestrator.registerProfile(workerProfile());
    orchestrator.deployFleet({
      name: 'status-fleet',
      version: '2.0',
      sandboxes: [{ profile: 'worker', count: 2 }],
    });

    const status = orchestrator.fleetStatus('status-fleet');
    expect(status).toBeDefined();
    expect(status!.name).toBe('status-fleet');
    expect(status!.totalSandboxes).toBe(2);
    expect(status!.byState[SandboxState.Provisioning]).toBe(2);
  });

  it('should return undefined for unknown fleet', () => {
    expect(orchestrator.fleetStatus('nope')).toBeUndefined();
  });

  it('should terminate all sandboxes in a fleet', () => {
    orchestrator.registerProfile(workerProfile());
    const ids = orchestrator.deployFleet({
      name: 'term-fleet',
      version: '1.0',
      sandboxes: [{ profile: 'worker', count: 2 }],
    });

    // Move to running so terminate goes through Stopping -> Terminated
    for (const id of ids) {
      orchestrator.manager.transition(id, SandboxState.Starting);
      orchestrator.manager.transition(id, SandboxState.Running);
    }

    orchestrator.terminateFleet('term-fleet');

    const status = orchestrator.fleetStatus('term-fleet');
    expect(status!.byState[SandboxState.Terminated]).toBe(2);
  });

  it('should handle terminating unknown fleet gracefully', () => {
    // Should not throw
    orchestrator.terminateFleet('nonexistent');
  });
});
