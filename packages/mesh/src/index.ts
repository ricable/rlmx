// ---------------------------------------------------------------------------
// @aix/mesh — Personal mesh topology and device management
//
// Re-exports all public types and functions from the mesh bounded context.
// Maps to: crates/rlmx-mesh/src/lib.rs
// ---------------------------------------------------------------------------

// Device types and helpers
export type {
  DeviceId,
  DeviceZone,
  DeviceType,
  DeviceStatus,
  DeviceCapabilities,
  MeshDevice,
} from "./device.js";
export {
  zoneToSwarmZone,
  isCoordinatorZone,
  isPrivacyAnchorZone,
  defaultZoneForDeviceType,
  defaultCapabilities,
  createMeshDevice,
  isDeviceOnline,
  ALL_ZONES,
} from "./device.js";

// Discovery
export type {
  DiscoveryMethod,
  DeviceAnnouncement,
  JoinResult,
} from "./discovery.js";
export { DiscoveryService } from "./discovery.js";

// Sync
export type {
  SyncTransport,
  SyncProtocol,
  SyncScope,
  SyncPriority,
  SyncHealth,
  SyncPolicy,
  SyncState,
} from "./sync.js";
export {
  createSyncState,
  markSynced,
  enqueueSyncOps,
  evaluateSyncHealth,
  defaultSyncPolicies,
  selectTransport,
} from "./sync.js";

// Failover
export type {
  DegradationLevel,
  FailoverReason,
  MigrationReason,
  FailoverPolicy,
  MeshDegradation,
} from "./failover.js";
export {
  defaultFailoverPolicies,
  failoverPolicyForZone,
  evaluateDegradation,
} from "./failover.js";

// Events
export type {
  DeviceJoinedEvent,
  DeviceLeftEvent,
  MeshReconfiguredEvent,
  SyncCompletedEvent,
  ZoneFailoverEvent,
  PrivacyAnchorRestoredEvent,
  AgentMigratedEvent,
  MeshDomainEvent,
} from "./events.js";

// Errors
export type { MeshErrorCode, MeshResult } from "./errors.js";
export { MeshError } from "./errors.js";

// Mesh aggregate root
export type {
  MeshId,
  MeshAgent,
  SonaSnapshot,
  FleetStatus,
} from "./mesh.js";
export { PersonalMesh } from "./mesh.js";
