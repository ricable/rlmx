// ---------------------------------------------------------------------------
// @aix/mesh — State synchronization between mesh devices
//
// Maps to: crates/rlmx-mesh/src/sync.rs
// ---------------------------------------------------------------------------

import type { DeviceId, DeviceZone } from "./device.js";

/** Transport protocol used for device-to-device synchronization. */
export type SyncTransport =
  | "Quic"          // Sub-millisecond LAN sync (hub <-> laptop)
  | "WebSocket"     // Phone <-> cloud relay via MCP WS server (:3001)
  | "Http"          // Fallback for cloud burst nodes behind firewalls
  | "BroadcastChannel"; // Browser tab sync (same origin)

/** Protocol used for state consistency. */
export type SyncProtocol =
  | "RaftConsensus"  // Critical state (agent placement, capability tokens)
  | "Crdt"           // Eventual consistency on metrics, counters, engagement scores
  | "LastWriterWins"; // Simple timestamp-based conflict resolution

/** What category of data is being synced. */
export type SyncScope =
  | "AgentPlacement"
  | "CapabilityTokens"
  | "SonaPatterns"
  | "EngagementState"
  | "EphemeralMetrics";

/** Priority level for sync operations. */
export type SyncPriority =
  | "Critical"   // Must sync before operations proceed
  | "Normal"     // Sync on schedule
  | "BestEffort"; // Sync when bandwidth available

/** Health status of sync between a device pair. */
export type SyncHealth =
  | "Healthy"       // Last sync within threshold
  | "Degraded"      // Pending ops above warning threshold
  | "Disconnected"; // No sync within timeout

/** Policy governing how a particular scope of data synchronizes. */
export interface SyncPolicy {
  scope: SyncScope;
  protocol: SyncProtocol;
  intervalMs: number;
  priority: SyncPriority;
  maxPending: number;
}

/** Synchronization state between a pair of devices. */
export interface SyncState {
  sourceDevice: DeviceId;
  targetDevice: DeviceId;
  transport: SyncTransport;
  lastSync: string; // ISO 8601
  pendingOps: number;
  health: SyncHealth;
  /** Last known epoch per remote peer for delta sync compatibility. */
  peerEpochs: Map<DeviceId, number>;
}

// ---------------------------------------------------------------------------
// Factory / helpers
// ---------------------------------------------------------------------------

/** Create a new SyncState for a device pair. */
export function createSyncState(
  sourceDevice: DeviceId,
  targetDevice: DeviceId,
  transport: SyncTransport,
): SyncState {
  return {
    sourceDevice,
    targetDevice,
    transport,
    lastSync: new Date().toISOString(),
    pendingOps: 0,
    health: "Healthy",
    peerEpochs: new Map(),
  };
}

/** Record a successful sync, reducing pending ops. */
export function markSynced(state: SyncState, opsSynced: number): void {
  state.pendingOps = Math.max(0, state.pendingOps - opsSynced);
  state.lastSync = new Date().toISOString();
  state.health = state.pendingOps === 0 ? "Healthy" : "Degraded";
}

/** Enqueue pending operations. */
export function enqueueSyncOps(state: SyncState, count: number): void {
  state.pendingOps += count;
}

/** Evaluate health based on pending ops threshold. */
export function evaluateSyncHealth(
  state: SyncState,
  degradedThreshold: number,
): void {
  if (state.pendingOps === 0) {
    state.health = "Healthy";
  } else if (state.pendingOps > degradedThreshold) {
    state.health = "Disconnected";
  } else {
    state.health = "Degraded";
  }
}

/**
 * Default sync policies per ADR-022.
 *
 * Covers all five SyncScope categories with appropriate protocols
 * and timing for each.
 */
export function defaultSyncPolicies(): SyncPolicy[] {
  return [
    {
      scope: "AgentPlacement",
      protocol: "RaftConsensus",
      intervalMs: 1_000,
      priority: "Critical",
      maxPending: 10,
    },
    {
      scope: "CapabilityTokens",
      protocol: "RaftConsensus",
      intervalMs: 5_000,
      priority: "Critical",
      maxPending: 5,
    },
    {
      scope: "SonaPatterns",
      protocol: "Crdt",
      intervalMs: 30_000,
      priority: "Normal",
      maxPending: 100,
    },
    {
      scope: "EngagementState",
      protocol: "Crdt",
      intervalMs: 60_000,
      priority: "Normal",
      maxPending: 50,
    },
    {
      scope: "EphemeralMetrics",
      protocol: "LastWriterWins",
      intervalMs: 10_000,
      priority: "BestEffort",
      maxPending: 200,
    },
  ];
}

/**
 * Select the appropriate transport for a device pair based on their zones.
 *
 * Matches the Rust `select_transport` function logic.
 */
export function selectTransport(
  zoneA: DeviceZone,
  zoneB: DeviceZone,
): SyncTransport {
  // LAN devices use QUIC for sub-millisecond sync.
  if (
    (zoneA === "ADesktop" && zoneB === "CEdge") ||
    (zoneA === "CEdge" && zoneB === "ADesktop")
  ) {
    return "Quic";
  }
  // Browser uses BroadcastChannel.
  if (zoneA === "DBrowser" || zoneB === "DBrowser") {
    return "BroadcastChannel";
  }
  // Phone uses WebSocket via cloud relay.
  if (zoneA === "AMobile" || zoneB === "AMobile") {
    return "WebSocket";
  }
  // Cloud uses HTTP.
  if (zoneA === "BCloud" || zoneB === "BCloud") {
    return "Http";
  }
  // Default to WebSocket.
  return "WebSocket";
}
