# DDD-011: Personal Mesh Bounded Context

## Overview

The Personal Mesh context manages multi-device coordination, zone assignment,
device discovery, state synchronization, and fleet management for a single
user's device mesh. It is the orchestration layer that turns independent
devices into a coordinated agent runtime.

A personal mesh represents one user's collection of devices -- phone, laptop,
home hub, browser tabs, cloud burst nodes -- unified under a single capability
token hierarchy. The mesh continuously discovers devices, assigns them to
swarm zones, synchronizes agent state across them, and handles failover when
devices go offline.

This context extends the Swarm Coordination context (DDD-003) with user-scoped
topology management. Where DDD-003 defines zone semantics and consensus
protocols, DDD-011 owns the lifecycle of a specific user's device fleet and
the policies that govern how agents and state flow between those devices.

**Crate**: `crates/rlmx-mesh/` (or extension of `crates/rlmx-swarm/`)

## Aggregate Root: PersonalMesh

```rust
pub struct PersonalMesh {
    pub id: MeshId,
    pub root_token: CapabilityToken,        // User's root capability token
    pub devices: Vec<MeshDevice>,
    pub agents: Vec<MeshAgent>,
    pub sync_states: Vec<SyncState>,
    pub fleet_manifest: FleetManifest,
    pub privacy_anchor: Option<DeviceId>,   // Zone C home hub
    pub created_at: DateTime<Utc>,
    pub last_reconfigured: DateTime<Utc>,
}
```

`PersonalMesh` is the aggregate root because it owns the consistency boundary
for a single user's device fleet. All mutations to device membership, zone
assignments, agent placement, and sync state flow through the mesh. The mesh
is the unit of capability delegation -- every device token is derived from
the mesh's root token, and revoking the root token invalidates the entire
fleet.

### Identity

- `MeshId` -- Unique identifier for a user's mesh instance, derived from
  the user's root capability token via deterministic hash.

## Entities

### MeshDevice

A physical device participating in the mesh:

```rust
pub struct MeshDevice {
    pub device_id: DeviceId,
    pub zone: Zone,                         // A-Desktop, C-Edge, D-Browser, B-Cloud
    pub capabilities: DeviceCapabilities,   // CPU, GPU type, memory, battery, network
    pub status: DeviceStatus,
    pub last_seen: DateTime<Utc>,
    pub transport: TransportType,
    pub device_token: CapabilityToken,      // Derived from mesh root token
}

pub enum DeviceStatus {
    Online,
    Offline,
    Sleeping,
    Syncing,
}

pub enum TransportType {
    Quic,
    WebSocket,
    Http,
}
```

A `MeshDevice` is an entity because it has identity and a lifecycle independent
of the mesh aggregate. Devices join, leave, sleep, and wake. Their zone
assignment can change over time as capabilities shift (e.g., laptop unplugged
from power moves from A-Desktop to C-Edge equivalent).

### MeshAgent

An agent instance running on a specific device:

```rust
pub struct MeshAgent {
    pub agent_id: AgentId,
    pub agent_type: AgentType,
    pub device_id: DeviceId,
    pub sandbox_profile: SandboxProfile,
    pub model_tier: ModelTier,
    pub sona_snapshot: SonaSnapshot,
}

pub struct SonaSnapshot {
    pub last_sync: DateTime<Utc>,
    pub pattern_count: u64,
}
```

`MeshAgent` is an entity because agents have identity and state that persists
across device migrations. When a device goes offline, its agents may be
migrated to another device -- the agent identity is preserved even though the
hosting device changes.

### SyncState

Synchronization state between a pair of devices:

```rust
pub struct SyncState {
    pub source_device: DeviceId,
    pub target_device: DeviceId,
    pub last_sync: DateTime<Utc>,
    pub pending_ops: u64,
    pub protocol: SyncProtocol,
    pub health: SyncHealth,
}

pub enum SyncProtocol {
    Raft,       // Critical state (agent placement, capability tokens)
    Crdt,       // Metrics, counters, engagement scores
    Gossip,     // Discovery, heartbeat, ephemeral state
}

pub enum SyncHealth {
    Healthy,        // last_sync within threshold
    Degraded,       // pending_ops above warning threshold
    Disconnected,   // no sync within timeout
}
```

`SyncState` is an entity because it tracks the evolving relationship between
two specific devices. Each device pair has independent sync progress, backlog,
and health status.

## Value Objects

### FleetManifest

```rust
pub struct FleetManifest {
    pub mesh_id: MeshId,
    pub version: u64,
    pub zone_assignments: Vec<ZoneAssignment>,
    pub transport_configs: Vec<TransportConfig>,
    pub sync_policies: Vec<SyncPolicy>,
    pub updated_at: DateTime<Utc>,
}
```

The `FleetManifest` is the declarative source of truth for mesh topology.
Devices self-configure from it. It extends the existing `FleetManifest` type
in `rlmx-swarm` with user-scoped policies and transport configuration.

### ZoneAssignment

```rust
pub struct ZoneAssignment {
    pub device_id: DeviceId,
    pub zone: Zone,
    pub rationale: ZoneRationale,
}

pub enum ZoneRationale {
    ManualOverride,             // User explicitly assigned zone
    CapabilityDerived,          // Auto-assigned based on device capabilities
    FailoverPromotion,          // Promoted due to primary zone device offline
    BatteryDemotion,            // Demoted due to low battery
}
```

### TransportConfig

```rust
pub struct TransportConfig {
    pub source_device: DeviceId,
    pub target_device: DeviceId,
    pub transport: TransportType,
    pub endpoint: String,
    pub max_payload_bytes: u64,
    pub keepalive_interval_ms: u64,
}
```

Per-link transport configuration. The mesh selects transport based on device
capabilities and network conditions: QUIC for low-latency device-to-device,
WebSocket for browser tabs, HTTP for cloud burst nodes behind firewalls.

### SyncPolicy

```rust
pub struct SyncPolicy {
    pub scope: SyncScope,
    pub protocol: SyncProtocol,
    pub interval_ms: u64,
    pub priority: SyncPriority,
}

pub enum SyncScope {
    AgentPlacement,     // Which agents run where
    CapabilityTokens,   // Device token derivations
    SonaPatterns,       // Learned patterns from SONA
    EngagementState,    // Life Score, streaks, savings
    EphemeralMetrics,   // CPU/GPU utilization, latency
}

pub enum SyncPriority {
    Critical,       // Must sync before operations proceed
    Normal,         // Sync on schedule
    BestEffort,     // Sync when bandwidth available
}
```

Rules for what syncs where and how. Critical state (agent placement, tokens)
uses Raft consensus. Metrics and counters use CRDTs for eventual consistency.
Discovery and heartbeat use gossip protocol.

## Value Object Summary

| Value Object | Definition |
|-------------|------------|
| `FleetManifest` | Declarative mesh topology: zone assignments, transport configs, sync policies |
| `ZoneAssignment` | Mapping of device to zone with rationale for the assignment |
| `ZoneRationale` | Enum: `ManualOverride`, `CapabilityDerived`, `FailoverPromotion`, `BatteryDemotion` |
| `TransportConfig` | Per-link transport configuration (type, endpoint, keepalive) |
| `SyncPolicy` | Rules for what data syncs where, via which protocol, at what priority |
| `SyncScope` | Enum: `AgentPlacement`, `CapabilityTokens`, `SonaPatterns`, `EngagementState`, `EphemeralMetrics` |
| `SyncProtocol` | Enum: `Raft`, `Crdt`, `Gossip` |
| `SyncPriority` | Enum: `Critical`, `Normal`, `BestEffort` |
| `SonaSnapshot` | Last sync timestamp + pattern count for an agent's SONA state |

## Domain Events

| Event | Trigger | Data |
|-------|---------|------|
| `DeviceJoined` | New device discovered and authenticated | `{ mesh_id, device_id, zone, capabilities }` |
| `DeviceLeft` | Device goes offline or is removed | `{ mesh_id, device_id, graceful, pending_ops }` |
| `MeshReconfigured` | FleetManifest updated | `{ mesh_id, manifest_version, changes }` |
| `SyncCompleted` | Device-to-device sync finished | `{ source_device, target_device, ops_synced, protocol }` |
| `ZoneFailover` | Primary zone device offline, secondary takes over | `{ mesh_id, from_zone, to_zone, promoted_device }` |
| `PrivacyAnchorRestored` | Home hub comes back online after outage | `{ mesh_id, anchor_device, queued_syncs }` |
| `AgentMigrated` | Agent moved from one device to another | `{ agent_id, from_device, to_device, reason }` |
| `TransportDegraded` | Link quality dropped below threshold | `{ source_device, target_device, latency_ms, packet_loss }` |

```rust
pub enum MeshDomainEvent {
    DeviceJoined {
        mesh_id: MeshId,
        device_id: DeviceId,
        zone: Zone,
        capabilities: DeviceCapabilities,
    },
    DeviceLeft {
        mesh_id: MeshId,
        device_id: DeviceId,
        graceful: bool,
        pending_ops: u64,
    },
    MeshReconfigured {
        mesh_id: MeshId,
        manifest_version: u64,
        changes: Vec<String>,
    },
    SyncCompleted {
        source_device: DeviceId,
        target_device: DeviceId,
        ops_synced: u64,
        protocol: SyncProtocol,
    },
    ZoneFailover {
        mesh_id: MeshId,
        from_zone: Zone,
        to_zone: Zone,
        promoted_device: DeviceId,
    },
    PrivacyAnchorRestored {
        mesh_id: MeshId,
        anchor_device: DeviceId,
        queued_syncs: u64,
    },
    AgentMigrated {
        agent_id: AgentId,
        from_device: DeviceId,
        to_device: DeviceId,
        reason: MigrationReason,
    },
    TransportDegraded {
        source_device: DeviceId,
        target_device: DeviceId,
        latency_ms: u64,
        packet_loss: f32,
    },
}

pub enum MigrationReason {
    DeviceOffline,
    BatteryLow,
    CapabilityMismatch,
    LoadBalancing,
    UserRequested,
}
```

## Domain Services

### `discover_and_join(device_capabilities, user_token) -> JoinResult`

Device discovery and mesh join flow:

1. Device announces itself via gossip protocol on the local network.
2. Mesh validates the user token against the root capability token.
3. Derive a device-scoped capability token (always narrower than parent).
4. Auto-assign zone based on device capabilities (GPU presence -> A-Desktop,
   battery-powered + cellular -> A-Mobile, always-on low-power -> C-Edge,
   browser tab -> D-Browser).
5. Update `FleetManifest` with new zone assignment.
6. Emit `DeviceJoined` event.
7. Trigger initial sync of critical state (agent placement, capability tokens).

### `leave_mesh(device_id, graceful) -> LeaveResult`

Graceful or ungraceful device departure:

1. If graceful: flush pending sync operations, migrate agents to other
   devices, revoke device capability token.
2. If ungraceful (device disappeared): detect via missed heartbeats,
   queue pending ops for retry when device returns or reassign to
   another device.
3. Update `FleetManifest` to remove device.
4. Emit `DeviceLeft` event.
5. If the departed device was the coordinator (Zone A), trigger failover.

### `reconfigure_mesh(new_fleet_manifest) -> ReconfigureResult`

Mesh topology update:

1. Validate new manifest against invariants (exactly one privacy anchor
   or zero, no device in multiple meshes, capability tokens consistent).
2. Diff against current manifest to identify changes.
3. For zone reassignments: notify affected devices, migrate agents if needed.
4. For new sync policies: update sync state protocol selections.
5. Increment manifest version and distribute to all devices.
6. Emit `MeshReconfigured` event.

### `sync_devices(source, target, scope) -> SyncResult`

State synchronization between two devices:

1. Select protocol based on `SyncPolicy` for the requested scope.
2. For Raft: propose state change, wait for quorum (critical state).
3. For CRDT: merge state vectors, resolve conflicts automatically.
4. For Gossip: exchange digests, pull missing state.
5. Update `SyncState` for the device pair.
6. Emit `SyncCompleted` event.
7. Update `SonaSnapshot` on `MeshAgent` if SONA patterns were synced.

### `failover_zone(from_zone, to_zone) -> FailoverResult`

Zone failover when primary device goes offline:

1. Identify the best candidate device in `to_zone` (or promote a device
   from another zone based on capabilities).
2. Transfer coordinator responsibilities: agent registry, pending
   syscall queue, active voice sessions.
3. Update `FleetManifest` with temporary zone reassignment.
4. Set `ZoneRationale::FailoverPromotion` on the promoted device.
5. Emit `ZoneFailover` event.
6. When original device returns, evaluate whether to restore original
   assignment or keep the new topology.

## Commands

| Command | Parameters | Description |
|---------|-----------|-------------|
| `JoinMesh` | `device_capabilities, user_token` | Authenticate device, assign zone, derive device token |
| `LeaveMesh` | `device_id` | Graceful departure, transfer state to remaining devices |
| `ReconfigureMesh` | `new_fleet_manifest` | Update topology, reassign agents |
| `SyncDevice` | `source, target, scope` | Trigger state sync between two devices |
| `FailoverZone` | `from_zone, to_zone` | Transfer coordinator responsibilities |
| `MigrateAgent` | `agent_id, target_device` | Move agent to a different device |
| `SetPrivacyAnchor` | `device_id` | Designate a device as the Zone C home hub |

## Queries

| Query | Returns | Description |
|-------|---------|-------------|
| `GetMeshStatus` | `MeshStatus` | All devices, their status, last sync times |
| `GetDeviceAgents` | `Vec<MeshAgent>` | Agents running on a specific device |
| `GetSyncBacklog` | `Vec<SyncState>` | Pending sync operations across all device pairs |
| `GetFleetManifest` | `FleetManifest` | Current mesh topology declaration |
| `GetZoneDevices` | `Vec<MeshDevice>` | All devices assigned to a specific zone |
| `GetTransportHealth` | `Vec<TransportHealth>` | Link quality metrics for all device pairs |

## Invariants

1. **Single privacy anchor**: Every mesh has exactly one privacy anchor
   (Zone C home hub) -- or zero if not yet configured. The privacy anchor
   is the only device that stores long-term SONA patterns and engagement
   history. If the anchor is offline, other devices queue state changes
   in their offline outbox (bounded at 100 ops per ADR-013).

2. **Single mesh membership**: A device can belong to exactly one mesh at
   a time. Attempting to join a second mesh requires explicitly leaving the
   first. This is enforced at the capability token level -- a device token
   is scoped to exactly one mesh root.

3. **Coordinator failover**: The coordinator agent (Zone A) is the swarm
   orchestrator. If Zone A is offline, Zone C takes over temporarily with
   `ZoneRationale::FailoverPromotion`. When Zone A returns, the mesh
   evaluates whether to restore the original assignment.

4. **Capability token narrowing**: Device capability tokens are derived
   from the mesh root token and are always strictly narrower than the
   parent. A device token can never grant permissions beyond what the
   mesh root allows. Token derivation is deterministic and verifiable.

5. **FleetManifest is source of truth**: The `FleetManifest` is the
   authoritative description of mesh topology. Devices self-configure
   from the manifest. Any ad-hoc state (e.g., failover promotions) is
   eventually reconciled back into the manifest.

6. **Privacy-respecting sync**: State sync respects privacy boundaries.
   Agent data syncs only to devices authorized by the capability token
   hierarchy. Speaker embeddings never sync (per DDD-008 invariant 2).
   Emotion data is bucketed before sync (per ADR-017).

7. **Bounded sync backlog**: Each device pair's pending sync operations
   are bounded. If the backlog exceeds the threshold, the mesh triggers
   a full state reconciliation rather than unbounded queue growth.

8. **Heartbeat timeout**: A device that misses 3 consecutive heartbeats
   (configurable interval, default 10s) transitions to `Offline` status.
   This triggers `DeviceLeft` event processing and potential failover.

## Context Relationships

| Related Context | Relationship | Description |
|----------------|-------------|-------------|
| Swarm Coordination (DDD-003) | Extends | Personal mesh is a user-scoped swarm; maps mesh zones to swarm zones, FleetManifest to swarm topology |
| Kernel Syscall (DDD-001) | Upstream | Translates mesh device operations into kernel syscalls (ProcessFork for agent spawn on device, ProcessSend for cross-device messages) |
| Agent Lifecycle (DDD-002) | Downstream | Agent spawn/terminate on device join/leave; agent migration across devices |
| Phone Runtime (DDD-009) | Partner | Device capabilities from PhoneRuntime inform zone assignment; battery state triggers zone demotion |
| Voice Interaction (DDD-008) | Partner | Voice sessions may span devices (start on phone, continue on laptop); session state syncs via mesh |
| Observation & Health (DDD-006) | Downstream | SONA patterns sync across devices via mesh sync policies |
| Container & Storage (DDD-007) | Upstream | RVF containers are deployed to devices via mesh; container images cached per-device |

### Relationship Details

**Extends Swarm Coordination**: The personal mesh reuses swarm primitives
(zones, consensus protocols, scatter-gather) but adds user-scoped policies.
Where the swarm context is concerned with abstract zone topology, the mesh
context maps that topology to concrete devices with capabilities, battery
levels, and network conditions.

**Partner with Phone Runtime**: The phone runtime (DDD-009) provides device
capabilities (`DeviceCapabilities`, `BatteryState`) that the mesh uses for
zone assignment decisions. When battery drops below threshold, the mesh may
demote a device's zone assignment (e.g., A-Mobile to passive mode) and
migrate agents to a plugged-in device. Neither context owns the other; they
coordinate through domain events on the `DomainEventBus`.

**Partner with Voice Interaction**: Voice sessions may span devices -- a user
starts a conversation on their phone and continues on their laptop. The mesh
handles session state transfer between devices, ensuring conversation turns,
active intents, and emotion trajectory are synced. The voice context (DDD-008)
is unaware of device boundaries; the mesh ACL translates cross-device session
continuity into the voice context's session model.

## Anti-Corruption Layer

The personal mesh context maintains strict boundaries with adjacent contexts:

- **To Kernel Syscall Context**: Translates mesh device operations into kernel
  syscalls. `JoinMesh` maps to `ProcessFork` for spawning agents on the new
  device. Cross-device messages use `ProcessSend` with device routing metadata.
  The kernel is unaware of device topology -- it processes syscalls uniformly.

- **To Swarm Coordination Context**: Maps mesh zones to swarm zones and
  `FleetManifest` to swarm topology. The swarm context operates on abstract
  zones (A, B, C, D); the mesh ACL resolves zone identifiers to concrete
  device endpoints and transport configurations.

- **From Phone Runtime Context**: Consumes `DeviceCapabilities` and
  `BatteryState` from the phone runtime. These are translated into
  `ZoneAssignment` decisions and `ZoneRationale` values. The mesh never
  directly queries phone hardware -- it receives capability snapshots through
  the event bus.

- **To Agent Lifecycle Context**: Agent spawn and terminate commands flow
  through the mesh ACL, which adds device placement information. The agent
  lifecycle context (DDD-002) manages agent state machines; the mesh context
  manages where those agents physically run.

- **Privacy boundary enforcement**: The ACL enforces that speaker embeddings
  (DDD-008) never cross device boundaries during sync. Emotion data is
  verified to be bucketed (5 discrete levels) before inclusion in any sync
  payload. SONA patterns are stripped of PII by the `FederatedAnonymizer`
  (from `rlmx-cognitive`) before cross-device sync.

## Crate Mapping

| Crate | Role |
|-------|------|
| `rlmx-mesh` (new) or `rlmx-swarm` (extension) | Primary: aggregate root, entities, domain events, services |
| `rlmx-kernel` | Types (`DeviceId`, `AgentId`, `Zone`), syscalls (`ProcessFork`, `ProcessSend`) |
| `rlmx-swarm` | `FleetManifest`, zone definitions, consensus protocols |
| `rlmx-cognitive` | `FederatedAnonymizer` for privacy-safe sync, SONA `PatternBank` |

External dependencies (if implemented as separate crate):
- `ruvix-cap` -- Capability token derivation and validation
- `midstreamer-quic` -- QUIC transport for low-latency device-to-device links
- `ruv-swarm-core` -- Gossip-based device discovery

## File Map

| File | Types | Status |
|------|-------|--------|
| `crates/rlmx-mesh/src/lib.rs` | `PersonalMesh`, `MeshId`, `MeshDomainEvent` | Planned |
| `crates/rlmx-mesh/src/device.rs` | `MeshDevice`, `DeviceStatus`, `TransportType` | Planned |
| `crates/rlmx-mesh/src/agent.rs` | `MeshAgent`, `SonaSnapshot`, agent migration | Planned |
| `crates/rlmx-mesh/src/sync.rs` | `SyncState`, `SyncProtocol`, `SyncHealth`, device sync | Planned |
| `crates/rlmx-mesh/src/manifest.rs` | `FleetManifest`, `ZoneAssignment`, `TransportConfig`, `SyncPolicy` | Planned |
| `crates/rlmx-mesh/src/discovery.rs` | Device discovery via gossip, heartbeat monitoring | Planned |
| `crates/rlmx-mesh/src/failover.rs` | Zone failover logic, coordinator transfer | Planned |
| `crates/rlmx-mesh/src/acl.rs` | Anti-corruption layer, privacy enforcement, context translation | Planned |

## Implementation Status

**Status**: Planned

This bounded context is designed but not yet implemented. The existing
`rlmx-swarm` crate provides foundational primitives (zones, `FleetManifest`,
consensus protocols) that this context will extend with user-scoped device
management, capability token derivation, and privacy-respecting state
synchronization.
