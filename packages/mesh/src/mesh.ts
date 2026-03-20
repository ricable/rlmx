// ---------------------------------------------------------------------------
// @aix/mesh — PersonalMesh aggregate root
//
// Maps to: crates/rlmx-mesh/src/mesh.rs
//
// CRITICAL INVARIANTS:
// - Home hub (Zone C/E) is sole long-term data store for personal data
// - Phone and laptop sync TO the hub, never to each other for persistence
// - Max 1 MeshCoordinator per mesh
// - Graceful degradation when devices go offline
// - Own domain events via MeshDomainEvent (not kernel DomainEvent)
// ---------------------------------------------------------------------------

import type {
  DeviceId,
  DeviceZone,
  DeviceType,
  DeviceStatus,
  DeviceCapabilities,
  MeshDevice,
} from "./device.js";
import {
  createMeshDevice,
  isCoordinatorZone,
  isDeviceOnline,
  isPrivacyAnchorZone,
} from "./device.js";
import type { MeshDomainEvent } from "./events.js";
import type { MigrationReason } from "./failover.js";
import { MeshError } from "./errors.js";
import type { SyncState } from "./sync.js";
import { createSyncState, selectTransport } from "./sync.js";

/** Unique identifier for a mesh instance (UUID string). */
export type MeshId = string;

/** An agent instance running on a specific device within the mesh. */
export interface MeshAgent {
  agentId: string;
  agentType: string;
  deviceId: DeviceId;
  processId?: string;
  sonaSnapshot: SonaSnapshot;
}

/** Snapshot of an agent's SONA learning state for sync purposes. */
export interface SonaSnapshot {
  lastSync: string; // ISO 8601
  patternCount: number;
}

/** Create a default SonaSnapshot. */
function defaultSonaSnapshot(): SonaSnapshot {
  return {
    lastSync: new Date().toISOString(),
    patternCount: 0,
  };
}

/** Summary of mesh fleet status. */
export interface FleetStatus {
  meshId: MeshId;
  totalDevices: number;
  onlineDevices: number;
  totalAgents: number;
  pendingSyncOps: number;
  hasPrivacyAnchor: boolean;
  hasCoordinator: boolean;
}

/** Maximum number of devices in a single personal mesh (ADR-022). */
const MAX_DEVICES = 10;

/**
 * The PersonalMesh aggregate root.
 *
 * Owns the consistency boundary for a single user's device fleet.
 * All mutations to device membership, zone assignments, agent placement,
 * and sync state flow through this aggregate.
 */
export class PersonalMesh {
  readonly id: MeshId;
  private _devices: MeshDevice[] = [];
  private _agents: MeshAgent[] = [];
  private _syncStates: SyncState[] = [];
  private _privacyAnchor?: DeviceId;
  private _pendingEvents: MeshDomainEvent[] = [];
  readonly createdAt: string;
  private _lastReconfigured: string;

  constructor(id: MeshId) {
    this.id = id;
    const now = new Date().toISOString();
    this.createdAt = now;
    this._lastReconfigured = now;
  }

  // ---- Read-only accessors ------------------------------------------------

  get devices(): readonly MeshDevice[] {
    return this._devices;
  }

  get agents(): readonly MeshAgent[] {
    return this._agents;
  }

  get syncStates(): readonly SyncState[] {
    return this._syncStates;
  }

  get privacyAnchor(): DeviceId | undefined {
    return this._privacyAnchor;
  }

  get lastReconfigured(): string {
    return this._lastReconfigured;
  }

  // ---- Device management --------------------------------------------------

  /**
   * Add a device to the mesh with its default zone.
   *
   * @throws {MeshError} If the mesh has reached MAX_DEVICES or the device already exists.
   */
  addDevice(
    deviceId: DeviceId,
    name: string,
    deviceType: DeviceType,
    capabilities: DeviceCapabilities,
    token: string,
  ): void {
    if (this._devices.length >= MAX_DEVICES) {
      throw new MeshError("MAX_DEVICES_REACHED", `Max devices reached: ${MAX_DEVICES}`, {
        detail: { max: MAX_DEVICES },
      });
    }
    if (this._devices.some((d) => d.deviceId === deviceId)) {
      throw new MeshError("DEVICE_ALREADY_EXISTS", `Device already exists in mesh: ${deviceId}`, {
        entityId: deviceId,
      });
    }

    const device = createMeshDevice(deviceId, name, deviceType, capabilities, token);
    const zone = device.zone;

    // Create sync states between the new device and all existing devices.
    for (const existing of this._devices) {
      const transport = selectTransport(existing.zone, zone);
      this._syncStates.push(createSyncState(existing.deviceId, deviceId, transport));
    }

    this._devices.push(device);
    this._lastReconfigured = new Date().toISOString();

    this._pendingEvents.push({
      type: "DeviceJoined",
      meshId: this.id,
      deviceId,
      zone,
    });
  }

  /**
   * Remove a device from the mesh.
   *
   * @throws {MeshError} If the device is not found.
   */
  removeDevice(deviceId: DeviceId, graceful: boolean): void {
    if (!this._devices.some((d) => d.deviceId === deviceId)) {
      throw new MeshError("DEVICE_NOT_FOUND", `Device not found: ${deviceId}`, {
        entityId: deviceId,
      });
    }

    // Calculate pending ops for the departing device.
    const pendingOps = this._syncStates
      .filter((s) => s.sourceDevice === deviceId || s.targetDevice === deviceId)
      .reduce((sum, s) => sum + s.pendingOps, 0);

    // Remove sync states involving this device.
    this._syncStates = this._syncStates.filter(
      (s) => s.sourceDevice !== deviceId && s.targetDevice !== deviceId,
    );

    // Clear privacy anchor if this was the anchor device.
    if (this._privacyAnchor === deviceId) {
      this._privacyAnchor = undefined;
    }

    this._devices = this._devices.filter((d) => d.deviceId !== deviceId);
    this._lastReconfigured = new Date().toISOString();

    this._pendingEvents.push({
      type: "DeviceLeft",
      meshId: this.id,
      deviceId,
      graceful,
      pendingOps,
    });
  }

  /**
   * Reassign a device to a different zone.
   *
   * @throws {MeshError} If the device is not found.
   */
  assignZone(deviceId: DeviceId, zone: DeviceZone): void {
    const device = this._devices.find((d) => d.deviceId === deviceId);
    if (!device) {
      throw new MeshError("DEVICE_NOT_FOUND", `Device not found: ${deviceId}`, {
        entityId: deviceId,
      });
    }

    const oldZone = device.zone;
    device.zone = zone;
    this._lastReconfigured = new Date().toISOString();

    this._pendingEvents.push({
      type: "MeshReconfigured",
      meshId: this.id,
      manifestVersion: 0,
      changes: [`zone reassignment: ${deviceId} from ${oldZone} to ${zone}`],
    });
  }

  /**
   * Set a device as the privacy anchor (must be Zone CEdge).
   *
   * INVARIANT: Home hub (Zone C/E) is sole long-term data store.
   *
   * @throws {MeshError} If the device is not found or not in the correct zone.
   */
  setPrivacyAnchor(deviceId: DeviceId): void {
    const device = this._devices.find((d) => d.deviceId === deviceId);
    if (!device) {
      throw new MeshError("DEVICE_NOT_FOUND", `Device not found: ${deviceId}`, {
        entityId: deviceId,
      });
    }
    if (!isPrivacyAnchorZone(device.zone)) {
      throw new MeshError(
        "INVALID_PRIVACY_ANCHOR_ZONE",
        `Privacy anchor must be Zone C (Edge), got ${device.zone}`,
        { entityId: deviceId, zone: device.zone },
      );
    }

    this._privacyAnchor = deviceId;
    this._lastReconfigured = new Date().toISOString();
  }

  // ---- Agent management ---------------------------------------------------

  /**
   * Place an agent on a specific device.
   *
   * @throws {MeshError} If the device is not found or agent is already assigned.
   */
  placeAgent(agentId: string, agentType: string, deviceId: DeviceId): void {
    if (!this._devices.some((d) => d.deviceId === deviceId)) {
      throw new MeshError("DEVICE_NOT_FOUND", `Device not found: ${deviceId}`, {
        entityId: deviceId,
      });
    }
    if (this._agents.some((a) => a.agentId === agentId)) {
      throw new MeshError("AGENT_ALREADY_ASSIGNED", `Agent already assigned: ${agentId}`, {
        entityId: agentId,
      });
    }

    this._agents.push({
      agentId,
      agentType,
      deviceId,
      sonaSnapshot: defaultSonaSnapshot(),
    });
  }

  /**
   * Migrate an agent from one device to another.
   *
   * @throws {MeshError} If the target device or agent is not found.
   */
  migrateAgent(
    agentId: string,
    targetDevice: DeviceId,
    reason: MigrationReason,
  ): void {
    if (!this._devices.some((d) => d.deviceId === targetDevice)) {
      throw new MeshError("DEVICE_NOT_FOUND", `Device not found: ${targetDevice}`, {
        entityId: targetDevice,
      });
    }

    const agent = this._agents.find((a) => a.agentId === agentId);
    if (!agent) {
      throw new MeshError("AGENT_NOT_FOUND", `Agent not found: ${agentId}`, {
        entityId: agentId,
      });
    }

    const fromDevice = agent.deviceId;
    agent.deviceId = targetDevice;

    this._pendingEvents.push({
      type: "AgentMigrated",
      agentId,
      fromDevice,
      toDevice: targetDevice,
      reason,
    });
  }

  // ---- Queries ------------------------------------------------------------

  /** Get all agents running on a specific device. */
  agentsOnDevice(deviceId: DeviceId): MeshAgent[] {
    return this._agents.filter((a) => a.deviceId === deviceId);
  }

  /** Get all devices in a specific zone. */
  devicesInZone(zone: DeviceZone): MeshDevice[] {
    return this._devices.filter((d) => d.zone === zone);
  }

  /** Get online devices only. */
  onlineDevices(): MeshDevice[] {
    return this._devices.filter((d) => isDeviceOnline(d));
  }

  /** Get the coordinator device (Zone ADesktop), if present and online. */
  coordinator(): MeshDevice | undefined {
    return this._devices.find(
      (d) => isCoordinatorZone(d.zone) && isDeviceOnline(d),
    );
  }

  /** Get the privacy anchor device, if set. */
  privacyAnchorDevice(): MeshDevice | undefined {
    if (!this._privacyAnchor) return undefined;
    return this._devices.find((d) => d.deviceId === this._privacyAnchor);
  }

  /** Get fleet status summary. */
  getFleetStatus(): FleetStatus {
    const online = this._devices.filter((d) => isDeviceOnline(d)).length;
    const pendingSync = this._syncStates.reduce((sum, s) => sum + s.pendingOps, 0);

    return {
      meshId: this.id,
      totalDevices: this._devices.length,
      onlineDevices: online,
      totalAgents: this._agents.length,
      pendingSyncOps: pendingSync,
      hasPrivacyAnchor: this._privacyAnchor != null,
      hasCoordinator: this.coordinator() != null,
    };
  }

  /** Drain and return pending domain events. */
  takeEvents(): MeshDomainEvent[] {
    const events = this._pendingEvents;
    this._pendingEvents = [];
    return events;
  }

  /**
   * Update a device's status.
   *
   * @throws {MeshError} If the device is not found.
   */
  updateDeviceStatus(deviceId: DeviceId, status: DeviceStatus): void {
    const device = this._devices.find((d) => d.deviceId === deviceId);
    if (!device) {
      throw new MeshError("DEVICE_NOT_FOUND", `Device not found: ${deviceId}`, {
        entityId: deviceId,
      });
    }
    device.status = status;
    device.lastSeen = new Date().toISOString();
  }
}
