// ---------------------------------------------------------------------------
// @aix/swarm — Fleet orchestration
// Provides higher-level fleet lifecycle management on top of SandboxManager.
// ---------------------------------------------------------------------------

import type { SandboxId, FleetManifest, SandboxProfile, SandboxInstance } from './sandbox.js';
import { SandboxManager, SandboxState } from './sandbox.js';

/**
 * Status summary for a deployed fleet.
 */
export interface FleetStatus {
  name: string;
  version: string;
  totalSandboxes: number;
  byState: Record<string, number>;
  sandboxIds: SandboxId[];
}

/**
 * FleetOrchestrator wraps SandboxManager with fleet-level operations
 * such as batch transitions and status summaries.
 */
export class FleetOrchestrator {
  readonly manager: SandboxManager;
  private deployedFleets: Map<string, { manifest: FleetManifest; sandboxIds: SandboxId[] }> =
    new Map();

  constructor(manager?: SandboxManager) {
    this.manager = manager ?? new SandboxManager();
  }

  /** Register a sandbox profile. */
  registerProfile(profile: SandboxProfile): void {
    this.manager.registerProfile(profile);
  }

  /** Deploy a fleet and track it by name. */
  deployFleet(manifest: FleetManifest): SandboxId[] {
    const ids = this.manager.deployFleet(manifest);
    this.deployedFleets.set(manifest.name, { manifest, sandboxIds: ids });
    return ids;
  }

  /** Get status summary for a deployed fleet. */
  fleetStatus(fleetName: string): FleetStatus | undefined {
    const entry = this.deployedFleets.get(fleetName);
    if (!entry) return undefined;

    const byState: Record<string, number> = {};
    for (const id of entry.sandboxIds) {
      const instance = this.manager.status(id);
      byState[instance.state] = (byState[instance.state] ?? 0) + 1;
    }

    return {
      name: entry.manifest.name,
      version: entry.manifest.version,
      totalSandboxes: entry.sandboxIds.length,
      byState,
      sandboxIds: [...entry.sandboxIds],
    };
  }

  /** Terminate all sandboxes in a fleet. */
  terminateFleet(fleetName: string): void {
    const entry = this.deployedFleets.get(fleetName);
    if (!entry) return;

    for (const id of entry.sandboxIds) {
      try {
        this.manager.terminate(id);
      } catch {
        // Best effort: sandbox may already be terminated
      }
    }
  }

  /** List all tracked fleet names. */
  listFleets(): string[] {
    return [...this.deployedFleets.keys()];
  }
}
