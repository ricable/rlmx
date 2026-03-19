// ---------------------------------------------------------------------------
// @aix/mesh — MeshDevice entity and related types
//
// Maps to: crates/rlmx-mesh/src/device.rs
// ---------------------------------------------------------------------------

/** Unique identifier for a device in the mesh (UUID string). */
export type DeviceId = string;

/**
 * The zone a device is assigned to within the 5-zone topology (ADR-001/022).
 *
 * - ADesktop: Laptop / Mac / Linux desktop -- coordinator, SONA master
 * - AMobile:  Phone (iOS/Android) -- always-on agents, quick inference, UI
 * - BCloud:   Cloud (NUC/VM) -- burst compute, large model inference
 * - CEdge:    Home Hub (RPi5) -- privacy anchor, long-term storage, sentinel
 * - DBrowser: Browser tab -- WASM/WebGPU compute workers
 */
export type DeviceZone = "ADesktop" | "AMobile" | "BCloud" | "CEdge" | "DBrowser";

/** Physical device type. */
export type DeviceType =
  | "Laptop"
  | "Phone"
  | "HomeHub"
  | "CloudNode"
  | "Browser"
  | "Sensor";

/** Connectivity and operational status of a device. */
export type DeviceStatus = "Online" | "Offline" | "Syncing" | "Degraded";

/** Hardware and runtime capabilities of a device. */
export interface DeviceCapabilities {
  cpuCores: number;
  memoryMb: number;
  hasGpu: boolean;
  gpuType?: string;
  hasBattery: boolean;
  batteryPct?: number;
  networkType?: string;
}

/** A physical device participating in the personal mesh. */
export interface MeshDevice {
  deviceId: DeviceId;
  name: string;
  deviceType: DeviceType;
  zone: DeviceZone;
  capabilities: DeviceCapabilities;
  status: DeviceStatus;
  lastSeen: string; // ISO 8601
  token: string;
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/** Returns the swarm zone identifier string for a zone. */
export function zoneToSwarmZone(zone: DeviceZone): string {
  const mapping: Record<DeviceZone, string> = {
    ADesktop: "zone-a-desktop",
    AMobile: "zone-a-mobile",
    BCloud: "zone-b",
    CEdge: "zone-c",
    DBrowser: "zone-d",
  };
  return mapping[zone];
}

/** Whether a zone can act as a coordinator. */
export function isCoordinatorZone(zone: DeviceZone): boolean {
  return zone === "ADesktop";
}

/** Whether a zone can serve as a privacy anchor (Zone C / Edge only). */
export function isPrivacyAnchorZone(zone: DeviceZone): boolean {
  return zone === "CEdge";
}

/** Returns the default zone for a given device type. */
export function defaultZoneForDeviceType(deviceType: DeviceType): DeviceZone {
  const mapping: Record<DeviceType, DeviceZone> = {
    Laptop: "ADesktop",
    Phone: "AMobile",
    HomeHub: "CEdge",
    CloudNode: "BCloud",
    Browser: "DBrowser",
    Sensor: "CEdge",
  };
  return mapping[deviceType];
}

/** Default device capabilities. */
export function defaultCapabilities(): DeviceCapabilities {
  return {
    cpuCores: 4,
    memoryMb: 8192,
    hasGpu: false,
    hasBattery: false,
  };
}

/** Create a new MeshDevice with defaults derived from its type. */
export function createMeshDevice(
  deviceId: DeviceId,
  name: string,
  deviceType: DeviceType,
  capabilities: DeviceCapabilities,
  token: string,
): MeshDevice {
  return {
    deviceId,
    name,
    deviceType,
    zone: defaultZoneForDeviceType(deviceType),
    capabilities,
    status: "Online",
    lastSeen: new Date().toISOString(),
    token,
  };
}

/** Whether a device is currently reachable. */
export function isDeviceOnline(device: MeshDevice): boolean {
  return device.status === "Online" || device.status === "Syncing";
}

/** All five zones in the mesh topology. */
export const ALL_ZONES: readonly DeviceZone[] = [
  "ADesktop",
  "AMobile",
  "BCloud",
  "CEdge",
  "DBrowser",
] as const;
