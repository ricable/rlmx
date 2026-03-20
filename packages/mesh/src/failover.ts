// ---------------------------------------------------------------------------
// @aix/mesh — Graceful degradation and failover policies
//
// Maps to: crates/rlmx-mesh/src/failover.rs
// ---------------------------------------------------------------------------

import type { DeviceZone } from "./device.js";
import { ALL_ZONES, isCoordinatorZone } from "./device.js";

/** Level of service degradation when devices are unavailable. */
export type DegradationLevel =
  | "Full"       // All devices online, full capability
  | "Reduced"    // Some devices offline, reduced but functional
  | "Offline"    // Only local device available, no sync
  | "CacheOnly"; // Operating from cached data only, no new computations

/** Reason for a zone failover event. */
export type FailoverReason =
  | "DeviceOffline"
  | "BatteryLow"
  | "NetworkDegraded"
  | "Overloaded"
  | "UserRequested";

/** Reason an agent was migrated between devices. */
export type MigrationReason =
  | "DeviceOffline"
  | "BatteryLow"
  | "CapabilityMismatch"
  | "LoadBalancing"
  | "UserRequested";

/** Policy dictating behavior when a specific zone goes offline. */
export interface FailoverPolicy {
  zone: DeviceZone;
  fallbackZone?: DeviceZone;
  degradation: DegradationLevel;
  migrateAgents: boolean;
  description: string;
}

/** Result of evaluating mesh degradation across all devices. */
export interface MeshDegradation {
  level: DegradationLevel;
  offlineZones: DeviceZone[];
  availableZones: DeviceZone[];
  description: string;
}

// ---------------------------------------------------------------------------
// Default policies per ADR-022
// ---------------------------------------------------------------------------

/** Default failover policies for all five zones. */
export function defaultFailoverPolicies(): FailoverPolicy[] {
  return [
    {
      zone: "AMobile",
      fallbackZone: "BCloud",
      degradation: "Reduced",
      migrateAgents: false,
      description:
        "Phone offline: 5 always-on WASM agents continue with local SONA cache",
    },
    {
      zone: "ADesktop",
      fallbackZone: "BCloud",
      degradation: "Reduced",
      migrateAgents: true,
      description:
        "Laptop offline: phone routes to cloud burst for complex queries",
    },
    {
      zone: "CEdge",
      degradation: "Offline",
      migrateAgents: false,
      description:
        "Home hub offline: devices use local cache, queue syncs in OfflineOutbox",
    },
    {
      zone: "BCloud",
      fallbackZone: "ADesktop",
      degradation: "Reduced",
      migrateAgents: false,
      description:
        "Cloud offline: all local inference, no federation updates",
    },
    {
      zone: "DBrowser",
      degradation: "CacheOnly",
      migrateAgents: false,
      description:
        "Browser tab closed: stateless workers terminated, no migration needed",
    },
  ];
}

/** Look up the failover policy for a given zone from defaults. */
export function failoverPolicyForZone(zone: DeviceZone): FailoverPolicy {
  const policy = defaultFailoverPolicies().find((p) => p.zone === zone);
  if (!policy) {
    throw new Error(`No failover policy for zone: ${zone}`);
  }
  return policy;
}

/**
 * Evaluate the overall degradation level given the set of online zones.
 *
 * Logic matches the Rust `evaluate_degradation` function:
 * - No online zones => CacheOnly
 * - All zones online => Full
 * - Coordinator online => Reduced
 * - No coordinator => Offline
 */
export function evaluateDegradation(
  onlineZones: readonly DeviceZone[],
): MeshDegradation {
  if (onlineZones.length === 0) {
    return {
      level: "CacheOnly",
      offlineZones: [...ALL_ZONES],
      availableZones: [],
      description: "All devices offline: operating from cached patterns only",
    };
  }

  const offline = ALL_ZONES.filter((z) => !onlineZones.includes(z));

  let level: DegradationLevel;
  if (offline.length === 0) {
    level = "Full";
  } else if (onlineZones.some((z) => isCoordinatorZone(z))) {
    level = "Reduced";
  } else {
    level = "Offline";
  }

  const descriptions: Record<DegradationLevel, string> = {
    Full: "All zones operational",
    Reduced: `Reduced: ${offline.length} zone(s) offline, coordinator available`,
    Offline: `Offline: coordinator unavailable, ${onlineZones.length} zone(s) online`,
    CacheOnly: "Cache only: no zones available",
  };
  const description = descriptions[level];

  return {
    level,
    offlineZones: offline,
    availableZones: [...onlineZones],
    description,
  };
}
