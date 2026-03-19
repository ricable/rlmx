# ADR-029: Universal Agent Deployment Layer (@aix/deploy)

Status: Implemented

## Context

RLMX is migrating to TypeScript (ADR-026, ADR-028). The system must deploy agents from multiple origins: native RLMX agents (17 typed roles with capability-secured permissions), Cognitum Seed sensor agents (edge IoT devices running on RPi Zero 2W), and arbitrary external agents accessible via REST/WebSocket. Each origin has different transport protocols, discovery mechanisms, resource constraints, and security models.

Without a unified deployment layer, each origin requires bespoke deployment logic, duplicated validation, and separate artifact generation. The personal mesh (ADR-022) and swarm (ADR-001) subsystems need a common manifest format to orchestrate agents across device zones.

Phase 6 of the TypeScript migration requires a deployment package that unifies all agent origins under a single manifest format while preserving origin-specific semantics.

## Decision

Create `@aix/deploy` as the universal agent deployment layer. It provides:

1. A discriminated-union `AgentManifest` format that handles all four agent origins
2. In-process transport adapters (not sidecars) for MCP, REST, WebSocket, and stub protocols
3. A discovery bridge that unifies mDNS across `_cognitum._tcp`, `_rlmx._tcp`, and custom service types
4. 50 preconfigured frozen templates with O(1) index lookups
5. 7 deployment profiles mapping to the existing 6-zone swarm topology
6. Artifact generators for systemd, fleet manifests, docker-compose, and verification scripts

### Package Structure

```
@aix/deploy
├── manifest.ts      — AgentManifest, AgentOrigin (4 variants), TransportSpec (7 variants)
├── domain-mapper.ts — DomainMapper (126 tag-to-LifeDomain mappings)
├── registry.ts      — ManifestRegistry (CRUD + single-pass AND search)
├── validation.ts    — validateManifest (origin, transport, resource, security)
├── transport.ts     — TransportAdapter interface + 4 implementations (MCP, REST, WS, Stub)
├── bridge.ts        — DiscoveryBridge (unified mDNS: _cognitum._tcp + _rlmx._tcp + custom)
├── seed-bridge.ts   — SeedBridge (Cognitum Seed REST/MCP client, EMBED_DIM=64)
├── templates.ts     — 50 frozen agent templates with pre-built index Maps
├── profiles.ts      — 7 deployment profiles (seed-compatible through mobile)
├── generator.ts     — Artifact generators (systemd, fleet, docker-compose, verify)
├── error.ts         — DeployError extends AixError (-36xxx code range)
└── index.ts         — Public API barrel export
```

### Dependencies

- `@aix/shared` — LifeDomain (12 variants), AgentType (17 variants), SyscallPermission (17 variants), AixError base class, JSON-RPC utilities
- `@aix/core` — NAPI Ed25519 attestation (used by security spec)

### AgentManifest Format

The manifest is a flat JSON object with a discriminated `origin` field:

```typescript
interface AgentManifest {
  id: string;
  name: string;
  version: string;                  // semver
  origin: AgentOrigin;              // discriminated union (4 variants)
  lifeDomain: LifeDomain;           // closed enum (12 variants from @aix/shared)
  domainTags: string[];             // open taxonomy (126 built-in mappings)
  capabilities: Capability[];
  modalities: Modality[];           // text | voice | vision | sensor | haptic | multimodal
  resourceEnvelope: ResourceEnvelope;
  transports: TransportSpec[];      // 7 transport types
  discovery?: DiscoverySpec;        // mDNS service type + TXT records
  security: SecuritySpec;           // bearer | mtls | macaroon | none
  deployment: DeploymentSpec;       // profile names, systemd unit, container image
  sensorConfig?: SensorSpec;        // gpio/i2c/spi/uart/usb interfaces
  learning?: LearningSpec;          // SONA, federation, drift detectors
  metadata: Record<string, unknown>;
}
```

### Agent Origins (4 variants)

| Origin | Fields | Use Case |
|--------|--------|----------|
| `rlmx` | `agentType`, `permissions[]` | Native RLMX agents with kernel syscall permissions |
| `seed` | `mcpEndpoint`, `sensorProfile`, `restEndpoint?` | Cognitum Seed IoT sensor agents |
| `external` | `protocol`, `endpoint` | Third-party agents via REST/WS/gRPC |
| `custom` | `handler` | User-defined handler function name |

### Domain Taxonomy

The package uses a two-tier taxonomy:

- **Closed**: `LifeDomain` enum (12 variants) from `@aix/shared` -- invariant per strict rule 31
- **Open**: `domainTags` string array with 126 built-in mappings via `DomainMapper`

The `DomainMapper` provides `inferDomain(tags)` which votes across tags to find the most common LifeDomain. Custom tags can be registered at runtime without modifying the closed LifeDomain enum.

Tag distribution across domains:
- Home: 29 (IoT, automation, agriculture, environmental)
- Finance: 12
- Career: 12
- Health: 9
- Travel: 7, Social: 7, Legal: 7, Education: 7
- Shopping: 6, Government: 6, Automotive: 6, Pet: 6
- Domain name aliases: 12

### Transport Adapters

In-process adapters (not sidecars) implementing a common `TransportAdapter` interface:

| Adapter | Protocol | Notes |
|---------|----------|-------|
| `McpTransportAdapter` | MCP JSON-RPC 2.0 | Uses `@aix/shared` `createRequest`/`isSuccessResponse` |
| `RestTransportAdapter` | REST over HTTP | Method path mapping (`sensors.read` -> `/sensors/read`) |
| `WebSocketTransportAdapter` | WebSocket | Connection pooling, timeout, pending request map |
| `StubTransportAdapter` | Any | Returns `{ status: 'unavailable' }` for unsupported protocols |

The `createTransportAdapter(spec)` factory selects the adapter by `spec.type`. Unknown types fall back to `StubTransportAdapter`.

### Seed Integration

`SeedBridge` wraps Cognitum Seed's REST API (:8443) and optional MCP tools:

- **EMBED_DIM=64**: All vector operations validate embedding dimension before sending
- **Exponential backoff**: HTTP 429 triggers retry with `baseMs * 2^attempt` delay (default 3 retries, 1000ms base)
- **Peer epoch tracking**: `clusterPeers()` updates a local `peerEpochs` map for delta sync
- **Operations**: status, sensorRead, sensorHistory, vectorQuery, vectorInsert, vectorDelete, witnessVerify, witnessChain, gpioRead, gpioWrite, clusterStatus, clusterPeers

### Discovery Bridge

`DiscoveryBridge` provides unified mDNS discovery:

- Default service types: `_cognitum._tcp` (seed), `_rlmx._tcp` (rlmx)
- Custom service types registered via `registerServiceType()`
- Events: `node-discovered`, `node-lost`
- Auto-manifest generation: `manifestForSeedNode(node)` creates an `AgentManifest` from a discovered Seed node's TXT records
- Domain model skeleton -- mDNS transport injected at integration time

### Templates (50)

50 preconfigured agent templates across 12 categories:

| Category | Count | Origin | Notes |
|----------|-------|--------|-------|
| Finance | 6 | rlmx | bill-negotiator, tax-planner, investment-tracker, expense-analyzer, budget-optimizer, crypto-monitor |
| Health | 6 | rlmx | medication-reminder, fitness-tracker, nutrition-advisor, sleep-optimizer, symptom-checker, mental-wellness |
| Legal | 4 | rlmx | contract-reviewer, rights-advisor, compliance-checker, dispute-resolver |
| Home IoT | 6 | seed | temperature-monitor, humidity-tracker, motion-detector, light-controller, energy-meter, water-leak-detector |
| Home Automation | 6 | rlmx | thermostat-agent, lighting-agent, security-cam-agent, appliance-scheduler, doorbell-agent, garage-agent |
| Career/Industrial | 4 | rlmx (hybrid) | predictive-maintenance, quality-inspector, energy-optimizer, vibration-analyzer |
| Agriculture | 4 | seed | soil-monitor, irrigation-controller, crop-health-analyzer, greenhouse-manager |
| Environmental | 4 | seed | air-quality-monitor, water-quality-monitor, weather-forecaster, uv-monitor |
| Education | 4 | rlmx | study-planner, flashcard-tutor, language-coach, skill-tracker |
| Travel | 3 | rlmx | flight-tracker, itinerary-builder, currency-converter |
| Social | 3 | rlmx | event-planner, birthday-reminder, gift-suggester |
| Pet | 2 | rlmx | feeding-scheduler, vet-reminder |
| Shopping | 2 | rlmx | price-comparator, deal-finder |

All templates are `Object.freeze()`-d at module load time. Pre-built `Map` indexes provide O(1) lookup by id, domain, and origin type. Consumers must clone before mutation.

### Deployment Profiles (7)

| Profile | Target | Zone | Features | Max Memory |
|---------|--------|------|----------|------------|
| `seed-compatible` | rpi-zero-2w | CEdge | sensor, drift-detection, witness | 256 MB |
| `rlmx-edge` | rpi5 | CEdge | ruvllm, gguf-inference | 4096 MB |
| `rlmx-cloud` | cloud-vm | BCloud | full-features, large-models | 16384 MB |
| `hybrid` | rpi5 | CEdge | rlmx, seed-bridge | 4096 MB |
| `minimal` | arm-generic | CEdge | routing-only | 512 MB |
| `browser` | browser | DBrowser | wasm, webgpu | 256 MB |
| `mobile` | phone | AMobile | react-native, free-agents | 1024 MB |

Profiles map to the 6-zone swarm topology from ADR-001 (A-Mobile, A-Desktop, B-Cloud, C-Edge, D-Browser, E-HomeHub).

### Artifact Generators

| Generator | Output | Notes |
|-----------|--------|-------|
| `generateSystemdUnit()` | `.service` file | Memory limits from profile, seed vs aix ExecStart |
| `generateFleetManifest()` | FleetManifest JSON | Compatible with `@aix/swarm` FleetManifest shape |
| `generateVerificationScript()` | Bash script | Shell-escaped, tests health/sensor/witness for Seed devices |
| `generateDockerCompose()` | docker-compose.yml | Resource limits from ResourceEnvelope |

### Error Handling

`DeployError` extends `AixError` with code range -36001 through -36010:

| Code | Name | Trigger |
|------|------|---------|
| -36001 | ManifestValidation | Invalid manifest fields |
| -36002 | ManifestNotFound | Registry lookup miss |
| -36003 | DuplicateManifest | Duplicate id on register |
| -36004 | TransportUnavailable | Unsupported protocol |
| -36005 | DiscoveryFailed | mDNS failure |
| -36006 | SeedBridgeError | Seed REST/MCP failure |
| -36007 | ProfileNotFound | Unknown profile name |
| -36008 | TemplateNotFound | Unknown template id |
| -36009 | DimensionMismatch | Vector dim != EMBED_DIM |
| -36010 | GeneratorError | Artifact generation failure |

### Validation

`validateManifest()` performs comprehensive validation:
- Required string fields (id, name, version as semver)
- Origin type discriminant + origin-specific required fields
- Modalities against the 6-value closed set
- Transport types against the 7-value closed set
- Resource envelope bounds (cpuCores > 0, memoryMb > 0, maxRuntimeMs >= 0)
- Security auth method against 4-value closed set
- Deployment profiles non-empty

## Consequences

### Good

- Unified manifest format eliminates per-origin deployment code
- Discriminated union `AgentOrigin` provides type-safe origin handling with exhaustive switch
- 50 frozen templates give instant agent deployment with zero configuration
- Single-pass registry search with AND semantics avoids multi-index complexity
- `DomainMapper` preserves the closed LifeDomain invariant while allowing open tag extension
- `SeedBridge` handles Cognitum Seed's idiosyncrasies (EMBED_DIM=64, 429 backoff, peer epochs) behind a clean API
- Shell-escaped generator output prevents injection in systemd units and verification scripts
- DeployError code range (-36xxx) is disjoint from all other @aix packages

### Bad

- `ResourceEnvelope.gpuType` uses a string union (`'none' | 'metal' | 'cuda' | 'webgpu'`) while `@aix/swarm` may use a numeric enum -- intentional divergence for JSON manifest ergonomics
- `DeployFleetManifest` is defined locally in `generator.ts` to avoid a hard runtime dependency on `@aix/swarm`, creating a structural (not nominal) compatibility contract
- DiscoveryBridge is a domain model skeleton -- production mDNS transport must be injected at integration time

### Neutral

- Templates are frozen at module load time -- adding new templates requires a package release, not runtime configuration
- 126 domain tag mappings cover the current agent catalog but will grow as new templates are added

## Related

- [ADR-026: TypeScript Migration Strategy](ADR-026-typescript-migration-strategy.md) -- migration framework
- [ADR-028: @aix Package Architecture](ADR-028-aix-package-architecture.md) -- workspace layout
- [ADR-022: Personal Mesh Topology](ADR-022-personal-mesh-topology.md) -- device zones
- [ADR-001: Distributed Swarm Architecture](ADR-001-distributed-swarm-architecture.md) -- 6-zone topology
- [ADR-025: Subscription Billing Tiers](ADR-025-subscription-billing-tiers.md) -- tier-enforced agent limits
- [DDD-016: Deployment Context](../DDD/DDD-016-deployment-context.md) -- bounded context model

## Tests

202 tests passing (vitest). Coverage includes manifest validation, registry CRUD and search, all 50 templates, all 7 profiles, domain mapper (126 mappings), transport adapters, SeedBridge retry logic, DiscoveryBridge events, and generator output.
