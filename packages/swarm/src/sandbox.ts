// ---------------------------------------------------------------------------
// @aix/swarm — SandboxManager, SandboxProfile, ResourceEnvelope
// Port of rlmx-swarm/src/sandbox.rs.
// ---------------------------------------------------------------------------

import { randomUUID } from 'node:crypto';

import type { NodeId } from './types.js';
import {
  sandboxNotFound,
  profileNotFound,
  invalidTransition,
  duplicateProfile,
  fleetValidation,
} from './errors.js';

// -- GPU & Network types ----------------------------------------------------

/** Hardware GPU requirement for a sandbox. */
export enum GpuRequirement {
  None = 'None',
  Metal = 'Metal',
  Cuda = 'Cuda',
  WebGpu = 'WebGpu',
}

/** Network isolation policy for sandbox egress/ingress. */
export enum NetworkPolicy {
  /** No external network access. */
  Isolated = 'Isolated',
  /** Can reach out, no inbound connections. */
  EgressOnly = 'EgressOnly',
  /** Only swarm cluster peers reachable. */
  ClusterOnly = 'ClusterOnly',
  /** Full network access (cloud burst workers). */
  Open = 'Open',
}

// -- ResourceEnvelope -------------------------------------------------------

/** Resource envelope constraining a sandbox's compute budget. */
export interface ResourceEnvelope {
  cpuCores: number;
  memoryMb: number;
  gpu: GpuRequirement;
  diskMb: number;
  /** Maximum runtime in milliseconds. */
  maxRuntimeMs: number;
}

// -- SandboxProfile ---------------------------------------------------------

/**
 * A sandbox profile: deployment descriptor composing an agent type
 * with resource limits, isolation policy, and model tier.
 * NOT a new agent type -- wraps an existing AgentType with deployment
 * concerns (ADR-011).
 */
export interface SandboxProfile {
  name: string;
  /** AgentType variant name. */
  agentType: string;
  resources: ResourceEnvelope;
  network: NetworkPolicy;
  /** Preferred ZoneId. */
  zonePreference: string;
  /** ModelTier variant name. */
  modelTier: string;
  /** e.g. ['ruvllm', 'metal'] */
  crateFeatures: string[];
  /** Optional .rvf container path. */
  rvfPayload?: string;
}

// -- SandboxState -----------------------------------------------------------

/** Unique identifier for a running sandbox instance. */
export type SandboxId = string;

/** Lifecycle state of a sandbox instance. */
export enum SandboxState {
  Provisioning = 'Provisioning',
  Starting = 'Starting',
  Running = 'Running',
  Suspended = 'Suspended',
  Stopping = 'Stopping',
  Terminated = 'Terminated',
  Failed = 'Failed',
}

/** Runtime metrics for a sandbox instance. */
export interface SandboxMetrics {
  cpuUsagePct: number;
  memoryUsageMb: number;
  uptimeSecs: number;
  tasksCompleted: number;
}

/** Create default zero-value metrics. */
export function defaultMetrics(): SandboxMetrics {
  return { cpuUsagePct: 0, memoryUsageMb: 0, uptimeSecs: 0, tasksCompleted: 0 };
}

/** A running sandbox instance binding a profile to a swarm node. */
export interface SandboxInstance {
  id: SandboxId;
  profile: SandboxProfile;
  state: SandboxState;
  nodeId?: NodeId;
  vmId?: string;
  createdAt: string;
  metrics: SandboxMetrics;
}

// -- State transitions ------------------------------------------------------

/**
 * Validates that a SandboxState transition is legal.
 * Returns true if the transition from `from` to `to` is allowed.
 */
export function validTransition(from: SandboxState, to: SandboxState): boolean {
  const allowed: [SandboxState, SandboxState][] = [
    [SandboxState.Provisioning, SandboxState.Starting],
    [SandboxState.Provisioning, SandboxState.Failed],
    [SandboxState.Starting, SandboxState.Running],
    [SandboxState.Starting, SandboxState.Failed],
    [SandboxState.Running, SandboxState.Suspended],
    [SandboxState.Running, SandboxState.Stopping],
    [SandboxState.Running, SandboxState.Failed],
    [SandboxState.Suspended, SandboxState.Running],
    [SandboxState.Suspended, SandboxState.Stopping],
    [SandboxState.Stopping, SandboxState.Terminated],
    [SandboxState.Stopping, SandboxState.Failed],
  ];
  return allowed.some(([f, t]) => f === from && t === to);
}

// -- SandboxManager ---------------------------------------------------------

/**
 * Manages sandbox profiles and running instances.
 * Port of Rust SandboxManager.
 */
export class SandboxManager {
  private profiles: Map<string, SandboxProfile> = new Map();
  private instances: Map<string, SandboxInstance> = new Map();

  /** Register a sandbox profile by name. Throws on duplicate. */
  registerProfile(profile: SandboxProfile): void {
    if (this.profiles.has(profile.name)) {
      throw duplicateProfile(profile.name);
    }
    this.profiles.set(profile.name, profile);
  }

  /** Get a profile by name. */
  getProfile(name: string): SandboxProfile | undefined {
    return this.profiles.get(name);
  }

  /** List all registered profile names. */
  listProfiles(): string[] {
    return [...this.profiles.keys()];
  }

  /**
   * Spawn a new sandbox instance from a named profile.
   * Starts in Provisioning state.
   */
  spawn(profileName: string): SandboxId {
    const profile = this.profiles.get(profileName);
    if (!profile) throw profileNotFound(profileName);

    const id: SandboxId = randomUUID();
    const instance: SandboxInstance = {
      id,
      profile: { ...profile },
      state: SandboxState.Provisioning,
      createdAt: new Date().toISOString(),
      metrics: defaultMetrics(),
    };
    this.instances.set(id, instance);
    return id;
  }

  /** Transition a sandbox to a new state. Throws on invalid transition. */
  transition(sandboxId: string, newState: SandboxState): void {
    const instance = this.instances.get(sandboxId);
    if (!instance) throw sandboxNotFound(sandboxId);

    if (!validTransition(instance.state, newState)) {
      throw invalidTransition(instance.state, newState);
    }
    instance.state = newState;
  }

  /** Terminate a sandbox (transition through Stopping to Terminated). */
  terminate(sandboxId: string): void {
    const instance = this.instances.get(sandboxId);
    if (!instance) throw sandboxNotFound(sandboxId);

    // Already done
    if (instance.state === SandboxState.Terminated || instance.state === SandboxState.Failed) {
      return;
    }

    // Running or Suspended go through Stopping
    if (instance.state === SandboxState.Running || instance.state === SandboxState.Suspended) {
      this.transition(sandboxId, SandboxState.Stopping);
    }

    this.transition(sandboxId, SandboxState.Terminated);
  }

  /** Get the status of a sandbox instance. */
  status(sandboxId: string): SandboxInstance {
    const instance = this.instances.get(sandboxId);
    if (!instance) throw sandboxNotFound(sandboxId);
    return instance;
  }

  /** List all sandbox instances. */
  listInstances(): SandboxInstance[] {
    return [...this.instances.values()];
  }

  /** Count instances by state. */
  countByState(state: SandboxState): number {
    let count = 0;
    for (const instance of this.instances.values()) {
      if (instance.state === state) count++;
    }
    return count;
  }

  /** Validate a fleet manifest: check all referenced profiles exist and counts > 0. */
  validateFleet(manifest: FleetManifest): void {
    if (manifest.sandboxes.length === 0) {
      throw fleetValidation('fleet manifest has no sandboxes');
    }
    for (const spec of manifest.sandboxes) {
      if (!this.profiles.has(spec.profile)) {
        throw profileNotFound(spec.profile);
      }
      if (spec.count === 0) {
        throw fleetValidation(`sandbox '${spec.profile}' has count 0`);
      }
    }
  }

  /** Deploy a fleet manifest, spawning all specified sandbox instances. */
  deployFleet(manifest: FleetManifest): SandboxId[] {
    this.validateFleet(manifest);
    const ids: SandboxId[] = [];
    for (const spec of manifest.sandboxes) {
      for (let i = 0; i < spec.count; i++) {
        ids.push(this.spawn(spec.profile));
      }
    }
    return ids;
  }
}

// -- Fleet types ------------------------------------------------------------

/** A single sandbox entry in a fleet manifest. */
export interface SandboxSpec {
  profile: string;
  count: number;
  overrides?: Record<string, unknown>;
}

/** Declarative fleet manifest describing multiple sandboxes to deploy. */
export interface FleetManifest {
  name: string;
  version: string;
  sandboxes: SandboxSpec[];
  cloudPolicy?: Record<string, unknown>;
}
