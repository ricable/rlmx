# ADR-022: Personal Mesh Topology

Status: Implemented

## Context
RuVix Mesh extends the existing 5-zone swarm topology (ADR-001) into a personal multi-device mesh where each user's devices form a coordinated agent runtime. The mesh must handle device discovery, state synchronization, zone assignment, and graceful degradation when devices go offline.

## Decision
Implement a personal mesh topology where each user's devices self-organize into a coordinated runtime with defined zone roles, gossip-based discovery, and a privacy anchor pattern.

### Zone Assignment
| Device | Zone | Role | Runtime |
|--------|------|------|---------|
| Laptop (Mac/Linux) | A-Desktop | Coordinator, SONA master, primary inference | NAPI-RS native |
| Phone (iOS/Android) | D-Browser | Always-on agents, quick inference, UI | WASM/WebGPU in PWA |
| Home Hub (RPi5) | C-Edge | Privacy anchor, long-term storage, sentinel | Native binary + systemd |
| Cloud (NUC/VM) | B-Cloud | Burst compute, large model inference, federation registry | Docker/K8s |

### Discovery & Join Protocol
1. Device starts -> announces via mDNS on LAN (home hub <-> laptop) or WebSocket registration (phone -> cloud relay)
2. `ruv-swarm-core`: New device discovers existing mesh members via gossip protocol
3. `ruvix-cap`: Device presents user's root capability token, derives device-scoped token with zone-specific caveats
4. `ruvector-raft`: Device joins Raft cluster for consistent state (family plan: multi-user Raft)
5. `midstreamer-quic`: QUIC transport established on LAN for sub-millisecond sync

### Privacy Anchor Pattern
- Home Hub (Zone C) is the **sole long-term data store** for personal data
- Phone and laptop sync TO the home hub, not to each other directly for persistence
- If phone is lost: agent state survives on home hub, restores to new phone
- If home hub is offline: phone and laptop operate independently with local cache, sync when hub returns
- Cloud (Zone B) NEVER stores user data -- only anonymized aggregated patterns

### State Synchronization
- **LAN (hub <-> laptop)**: `midstreamer-quic` QUIC transport, sub-millisecond, encrypted
- **Phone <-> cloud relay**: WebSocket via `rlmx-mcp` WS server (existing :3001)
- **Phone <-> LAN**: When on home WiFi, direct QUIC; otherwise cloud relay
- **Conflict resolution**: `ruvector-raft` for strong consistency on critical state (agent configs, capability tokens); CRDT for eventual consistency on metrics/patterns

### Graceful Degradation
| Scenario | Behavior |
|----------|----------|
| Phone offline | 5 always-on WASM agents continue with local SONA cache |
| Laptop offline | Phone routes to cloud burst for complex queries |
| Home hub offline | Devices use local cache, queue syncs in OfflineOutbox |
| Cloud offline | All local inference, no federation updates |
| All offline | Phone agents work with cached patterns (offline-first) |

### FleetManifest
Each user's mesh is described by a `FleetManifest` (existing type in `rlmx-swarm`) extended with:
- `device` field per sandbox (laptop, phone-wasm, home-hub-pi)
- `zone` mapping per device
- `model_tier` assignment per device capability
- Auto-generated on first mesh setup, evolves as devices join/leave

## Consequences

### Positive
- True multi-device coordination with clear zone responsibilities
- Privacy by architecture -- home hub as sole data anchor
- Offline-first -- every device works independently when disconnected
- Sub-millisecond sync on LAN via QUIC
- Existing swarm infrastructure (zones, FleetManifest, consensus) extends naturally

### Negative
- Home hub becomes single point of failure for data persistence (mitigated: encrypted backup to cloud opt-in)
- mDNS discovery may not work on all networks (mitigated: cloud relay fallback)
- QUIC on LAN adds networking complexity vs simple HTTP

### Risks
- NAT traversal for phone <-> home hub when not on same network
- Battery drain from persistent connections on phone (mitigated: BatteryAwareScheduler)

## References
- PRD: "RuVix Mesh -- The Personal Agent Cloud", Device-to-Zone Mapping
- ADR-001: Distributed Swarm Architecture
- ADR-008: WebSocket Realtime Events
- ADR-013: Phone Command Center
