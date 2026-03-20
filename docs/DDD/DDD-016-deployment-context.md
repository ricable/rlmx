# DDD-016: Agent Deployment Bounded Context

## Overview

The Agent Deployment bounded context manages the lifecycle of agent deployment manifests across heterogeneous origins (RLMX, Cognitum Seed, external, custom). It provides a unified manifest format, transport abstraction, device discovery, and deployment artifact generation.

Package: `@aix/deploy` (TypeScript). See [ADR-029](../ADR/ADR-029-universal-agent-deployment-layer.md) for architectural decisions.

## Aggregate Root

### ManifestRegistry

The `ManifestRegistry` is the aggregate root for the deployment context. It owns the consistency boundary for all registered agent manifests.

**Invariants**:
- Manifest IDs are unique within the registry (enforced at register time)
- All registered manifests pass `validateManifest()` checks
- Search operations are single-pass with AND semantics across all filter criteria

**Operations**:
- `register(manifest)` -- add a manifest, throws `DuplicateManifest` (-36003) on collision
- `get(id)` / `getOrThrow(id)` -- lookup by id
- `update(id, manifest)` -- replace, throws `ManifestNotFound` (-36002)
- `remove(id)` -- delete, returns boolean
- `search(options)` -- single-pass AND filter by domain, origin, modality, tag, nameContains
- `list()` -- all manifests
- `size` / `has(id)` / `clear()` -- utility

## Entities

### AgentManifest

The primary entity. Each manifest describes a deployable agent with:
- Identity: `id`, `name`, `version` (semver)
- Origin: discriminated union with 4 variants (rlmx, seed, external, custom)
- Classification: `lifeDomain` (closed, 12 variants), `domainTags` (open)
- Runtime: `capabilities`, `modalities`, `resourceEnvelope`, `transports`
- Infrastructure: `discovery`, `security`, `deployment`, `sensorConfig`, `learning`

### DeploymentProfile

Named deployment target with resource constraints. 7 built-in profiles mapping to the 6-zone swarm topology. Each profile specifies: target hardware, zone, enabled features, max power (watts), and max memory (MB).

### DiscoveredNode

A node found via mDNS discovery. Contains: id, name, host, port, serviceType, origin label, TXT records, and discovery timestamp. Managed by DiscoveryBridge, not persisted in ManifestRegistry.

## Value Objects

| Value Object | Defined In | Description |
|-------------|-----------|-------------|
| `AgentOrigin` | manifest.ts | Discriminated union: rlmx, seed, external, custom |
| `TransportSpec` | manifest.ts | Discriminated union: mcp, rest, ws, quic, mqtt, grpc, broadcast-channel |
| `ResourceEnvelope` | manifest.ts | cpuCores, memoryMb, gpuType, diskMb, maxRuntimeMs |
| `SecuritySpec` | manifest.ts | authMethod (bearer/mtls/macaroon/none), attestation, sandboxProfile, rateLimit |
| `SensorSpec` | manifest.ts | Sensor interfaces (gpio/i2c/spi/uart/usb), drift detection, sampling interval |
| `Modality` | manifest.ts | String literal union: text, voice, vision, sensor, haptic, multimodal |
| `Capability` | manifest.ts | Named capability with required flag |
| `LearningSpec` | manifest.ts | SONA, federation, drift detectors, EWC lambda |
| `DeploymentSpec` | manifest.ts | Profile names, systemd unit, container image, RVF path |
| `DiscoverySpec` | manifest.ts | mDNS service type, port, TXT records |
| `ValidationError` | validation.ts | Field path + error message |

## Domain Services

### DomainMapper

Maps open `domainTags` to the closed `LifeDomain` enum. 126 built-in mappings loaded from an immutable array into a shared `Map`. Supports:
- Single tag lookup: `map(tag)`, `mapOrDefault(tag, fallback)`
- Domain inference from multiple tags: `inferDomain(tags)` (majority vote)
- Custom registration: `register(tag, domain)`, `unregister(tag)`
- Reverse lookup: `tagsForDomain(domain)`

### SeedBridge

Anti-corruption layer translating between Cognitum Seed's REST API and the @aix/deploy manifest model.

**Seed-specific concerns handled**:
- EMBED_DIM=64 validation on all vector operations
- HTTP 429 exponential backoff (configurable retries and base delay)
- Peer epoch tracking for delta sync
- Dual transport: REST (:8443) primary, MCP optional

**Operations**: status, sensorRead, sensorHistory, vectorQuery, vectorInsert, vectorDelete, witnessVerify, witnessChain, gpioRead, gpioWrite, clusterStatus, clusterPeers.

### DiscoveryBridge

Unified mDNS discovery across Cognitum Seed, RLMX mesh, and custom service types.

**Default service types**:
- `_cognitum._tcp` -> origin `seed`
- `_rlmx._tcp` -> origin `rlmx`

**Events**: `node-discovered`, `node-lost` (event handler registration via `on`/`off`).

**Auto-manifest**: `manifestForSeedNode(node)` generates a full `AgentManifest` from a discovered node's TXT records.

Note: This is a domain model skeleton. Production mDNS transport (e.g., multicast-dns npm) is injected at integration time.

### TransportAdapter

Interface for sending requests to agents. Four implementations:
- `McpTransportAdapter` -- JSON-RPC 2.0 over HTTP
- `RestTransportAdapter` -- REST with method-to-path mapping
- `WebSocketTransportAdapter` -- persistent connection with request/response correlation
- `StubTransportAdapter` -- returns `unavailable` for unsupported protocols

Factory: `createTransportAdapter(spec)` selects by `spec.type`, falling back to Stub.

## Domain Events

| Event | Source | Payload |
|-------|--------|---------|
| `node-discovered` | DiscoveryBridge | DiscoveredNode |
| `node-lost` | DiscoveryBridge | DiscoveredNode |

Note: ManifestRegistry does not emit domain events. Manifest lifecycle is synchronous CRUD. Discovery events are the primary integration point with the Personal Mesh context (DDD-011).

## Context Map

### Upstream Dependencies

| Context | Package | Types Consumed |
|---------|---------|---------------|
| Shared Types | `@aix/shared` | LifeDomain (12 variants), AgentType (17 variants), SyscallPermission (17 variants), AixError, JSON-RPC utilities |
| Core Bridge | `@aix/core` | Ed25519 attestation (via SecuritySpec) |

### Downstream Consumers

| Context | Package | Integration Point |
|---------|---------|------------------|
| Swarm Coordination | `@aix/swarm` | FleetManifest structural compatibility via `generateFleetManifest()` |
| MCP Server | `@aix/mcp-server` | 8 deploy-related MCP tools (planned) |
| Personal Mesh | `@aix/mesh` | DiscoveryBridge events feed mesh device registration |

### Anti-Corruption Layers

- **SeedBridge**: Translates Cognitum Seed's REST API responses into @aix/deploy types. Handles Seed-specific concerns (EMBED_DIM, 429 backoff, peer epochs) without leaking them into the manifest model.
- **DiscoveryBridge.manifestForSeedNode()**: Transforms raw mDNS TXT records into a typed AgentManifest, applying default values for missing fields.

## Ubiquitous Language

| Term | Definition |
|------|-----------|
| **Manifest** | A complete description of a deployable agent: identity, origin, capabilities, resources, transports, and security |
| **Origin** | The framework or system an agent comes from (rlmx, seed, external, custom) |
| **Template** | A preconfigured frozen manifest that can be cloned and customized |
| **Profile** | A named deployment target with hardware constraints and enabled features |
| **Domain Tag** | An open string label mapped to a closed LifeDomain via DomainMapper |
| **Seed** | A Cognitum Seed IoT sensor node, typically running on RPi Zero 2W |
| **Transport** | The communication protocol used to reach an agent (MCP, REST, WS, etc.) |
| **Resource Envelope** | CPU, memory, GPU, disk, and runtime limits for an agent |
| **Discovery** | The process of finding agents on the network via mDNS service announcements |

## Invariants

1. Manifest IDs are globally unique within a ManifestRegistry instance
2. All vector operations through SeedBridge validate EMBED_DIM=64 before network calls
3. Templates are immutable (Object.freeze) -- consumers must clone before modification
4. LifeDomain is closed (12 variants) -- new agent domains map to existing variants via DomainMapper
5. DeployError codes occupy the -36xxx range exclusively
6. Generator output is shell-escaped to prevent injection

## Related

- [ADR-029: Universal Agent Deployment Layer](../ADR/ADR-029-universal-agent-deployment-layer.md)
- [DDD-011: Personal Mesh Context](DDD-011-personal-mesh-context.md) -- DiscoveryBridge feeds mesh
- [DDD-015: TypeScript Package Context Map](DDD-015-typescript-package-context-map.md) -- overall TS context map
- [DDD-010: Marketplace Context](DDD-010-marketplace-context.md) -- agent templates overlap with marketplace listings
