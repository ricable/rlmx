// ---------------------------------------------------------------------------
// @aix/mesh — Device discovery for the personal mesh
//
// Maps to: crates/rlmx-mesh/src/discovery.rs
//
// Handles mDNS announcement on LAN (home hub <-> laptop) and
// WebSocket registration for phone/cloud devices.
// ---------------------------------------------------------------------------

import type { DeviceId, DeviceType, DeviceCapabilities } from "./device.js";
import { defaultZoneForDeviceType, zoneToSwarmZone } from "./device.js";

/** Method by which a device was discovered. */
export type DiscoveryMethod =
  | "Mdns"             // mDNS on local network
  | "WebSocket"        // WebSocket registration via cloud relay
  | "Manual"           // Manual configuration by user
  | "BroadcastChannel"; // Browser tabs, same origin

/** A device announcement broadcast during discovery. */
export interface DeviceAnnouncement {
  deviceId: DeviceId;
  deviceType: DeviceType;
  name: string;
  capabilities: DeviceCapabilities;
  meshId?: string;
  method: DiscoveryMethod;
  endpoint: string;
  announcedAt: string; // ISO 8601
}

/** Result of a device join request. */
export interface JoinResult {
  deviceId: DeviceId;
  accepted: boolean;
  assignedZone?: string;
  reason?: string;
}

/**
 * Service responsible for device discovery and mesh join operations.
 *
 * In production this would use mDNS for LAN and WebSocket for WAN.
 * The current implementation provides the domain model without
 * transport-specific logic (see ADR-022 for protocol details).
 */
export class DiscoveryService {
  private announcements: Map<DeviceId, DeviceAnnouncement> = new Map();
  private readonly _meshId?: string;

  constructor(meshId?: string) {
    this._meshId = meshId;
  }

  /** Get the mesh ID this service is bound to. */
  get meshId(): string | undefined {
    return this._meshId;
  }

  /** Record a device announcement. Replaces any existing from the same device. */
  announce(announcement: DeviceAnnouncement): void {
    this.announcements.set(announcement.deviceId, announcement);
  }

  /** Return all known device announcements. */
  discover(): readonly DeviceAnnouncement[] {
    return Array.from(this.announcements.values());
  }

  /** Filter announcements to devices not yet in a mesh (available to join). */
  discoverAvailable(): DeviceAnnouncement[] {
    return Array.from(this.announcements.values()).filter((a) => a.meshId == null);
  }

  /**
   * Attempt to join the mesh with a device.
   *
   * Returns a JoinResult indicating acceptance or rejection.
   */
  joinMesh(deviceId: DeviceId, meshId: string): JoinResult {
    const ann = this.announcements.get(deviceId);
    if (!ann) {
      return {
        deviceId,
        accepted: false,
        reason: "device not found in discovery",
      };
    }
    if (ann.meshId != null) {
      return {
        deviceId,
        accepted: false,
        reason: "device already in a mesh",
      };
    }

    const zone = defaultZoneForDeviceType(ann.deviceType);
    ann.meshId = meshId;

    return {
      deviceId,
      accepted: true,
      assignedZone: zoneToSwarmZone(zone),
    };
  }

  /** Remove a device from the known announcements. */
  remove(deviceId: DeviceId): void {
    this.announcements.delete(deviceId);
  }
}
