// ---------------------------------------------------------------------------
// @aix/mesh — Domain events emitted by mesh operations
//
// Maps to: MeshDomainEvent enum in crates/rlmx-mesh/src/mesh.rs
//
// INVARIANT: These are crate-local events (MeshDomainEvent), NOT kernel
// DomainEvent. Cross-context communication uses these local events.
// ---------------------------------------------------------------------------

import type { DeviceZone } from "./device.js";
import type { MigrationReason } from "./failover.js";

/** A device joined the mesh. */
export interface DeviceJoinedEvent {
  readonly type: "DeviceJoined";
  readonly meshId: string;
  readonly deviceId: string;
  readonly zone: DeviceZone;
}

/** A device left the mesh. */
export interface DeviceLeftEvent {
  readonly type: "DeviceLeft";
  readonly meshId: string;
  readonly deviceId: string;
  readonly graceful: boolean;
  readonly pendingOps: number;
}

/** The mesh was reconfigured (zone reassignment, topology change). */
export interface MeshReconfiguredEvent {
  readonly type: "MeshReconfigured";
  readonly meshId: string;
  readonly manifestVersion: number;
  readonly changes: string[];
}

/** A sync operation completed between two devices. */
export interface SyncCompletedEvent {
  readonly type: "SyncCompleted";
  readonly sourceDevice: string;
  readonly targetDevice: string;
  readonly opsSynced: number;
}

/** A zone failover occurred. */
export interface ZoneFailoverEvent {
  readonly type: "ZoneFailover";
  readonly meshId: string;
  readonly fromZone: DeviceZone;
  readonly toZone: DeviceZone;
  readonly promotedDevice: string;
}

/** The privacy anchor was restored after being offline. */
export interface PrivacyAnchorRestoredEvent {
  readonly type: "PrivacyAnchorRestored";
  readonly meshId: string;
  readonly anchorDevice: string;
  readonly queuedSyncs: number;
}

/** An agent was migrated between devices. */
export interface AgentMigratedEvent {
  readonly type: "AgentMigrated";
  readonly agentId: string;
  readonly fromDevice: string;
  readonly toDevice: string;
  readonly reason: MigrationReason;
}

/** Union of all mesh domain events. */
export type MeshDomainEvent =
  | DeviceJoinedEvent
  | DeviceLeftEvent
  | MeshReconfiguredEvent
  | SyncCompletedEvent
  | ZoneFailoverEvent
  | PrivacyAnchorRestoredEvent
  | AgentMigratedEvent;
