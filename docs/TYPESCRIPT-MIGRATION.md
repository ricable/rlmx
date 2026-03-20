# TypeScript Migration Reference — ADR-026/027/028

> Comprehensive reference for the RLMX TypeScript migration. For quick-start, see the
> "TypeScript Packages" section in CLAUDE.md. For architecture decisions, see ADR-026/027/028.

## Migration Summary

**What**: 10 of 19 Rust crates ported to 13 TypeScript npm packages under `@aix` namespace.
**Why**: 57% of the codebase is orchestration/business logic better suited to TypeScript for faster iteration.
**Result**: Hybrid architecture — 9 Rust crates (compute) + 13 npm packages (orchestration) + `npx aix` CLI.

## Crate Classification

### Stay Rust (compiled into `@aix/core` NAPI binary)

| Crate | Why Rust | LOC |
|-------|----------|-----|
| `rlmx-kernel` | Vector math, router, proofs, syscall dispatch | Core |
| `rlmx-cognitive` | SONA micro-LoRA, EWC++, GNN | Neural |
| `rlmx-ruvllm` | Candle inference, Metal/CUDA | Inference |
| `rlmx-trm` | Pure numeric NN, 3-stream | Neural |
| `rlmx-rvf` | Ed25519 signatures, witness chain | Crypto |
| `rlmx-voice` | VAD signal processing, STT, TTS | Audio |
| `rlmx-phone` | Engagement scoring, battery scheduling | Runtime |
| `rlmx-napi` | The NAPI bridge itself | FFI |
| `rlmx-wasm` | Browser kernel subset | WASM |

### Migrated to TypeScript

| Rust Crate | npm Package | Aggregate Root | Tests |
|------------|-------------|----------------|-------|
| `rlmx-cli` | `aix` | — (CLI entry) | 15 |
| `rlmx-mcp` | `@aix/mcp-server` | McpServer | 46 |
| `rlmx-agents` | `@aix/agents` | AgentRegistry | 140 |
| `rlmx-swarm` | `@aix/swarm` | SwarmCluster | 72 |
| `rlmx-marketplace` | `@aix/marketplace` | Marketplace | 120 |
| `rlmx-mesh` | `@aix/mesh` | PersonalMesh | 97 |
| `rlmx-billing` | `@aix/billing` | Subscription | 71 |
| `rlmx-federation` | `@aix/federation` | FederationCycle | 50 |
| `rlmx-plugin` | `@aix/plugin` | PluginRegistry | 74 |
| `rlmx-rlm` | `@aix/rlm` | VllmClient | 35 |
| — (new) | `@aix/shared` | — (types) | 65 |
| — (new) | `@aix/core` | — (NAPI loader) | 17 |

**Total: 802 TypeScript tests across 41 test files.**

## Package Dependency Graph

```
                         aix (CLI)
                        /    |    \
              @aix/mcp-server  @aix/swarm  ...all @aix/*
                 /     \         /    \
         @aix/agents  @aix/core  @aix/agents
            |            |            |
        @aix/shared  @aix/shared  @aix/shared
```

## Directory Structure

```
rlmx/
  packages/
    aix/                    # `npx aix` — CLI entry (commander+chalk+ora)
      bin/cli.js            # Shebang entry point
      src/commands/          # 14 command groups mirroring Rust CLI
    core/                    # @aix/core — NAPI JS loader + type declarations
      src/loader.ts          # Platform detection (5 triples) + graceful degradation
      src/types.ts           # 31 NAPI function declarations
    shared/                  # @aix/shared — pure TS types (zero deps)
      src/enums.ts           # SyscallPermission(17), LifeDomain(12), AgentType(17), SubscriptionTier(6)
      src/events.ts          # DomainEvent discriminated union, DomainEventBus
      src/errors.ts          # AixError, AixErrorCode, factory helpers
      src/json-rpc.ts        # JSON-RPC 2.0 request/response types
      src/result.ts          # Result<T,E> type with combinators
      src/id.ts              # generateId() — canonical UUID generator
    mcp-server/              # @aix/mcp-server — 47 MCP tools via SDK
      src/tools.ts           # All 47 tool handlers
      src/rbac.ts            # 6-role RBAC with anti-escalation
      src/websocket.ts       # SwarmEvent broadcasting (12 variants)
    agents/                  # @aix/agents — permissions, lifecycle
      src/registry.ts        # PermissionRegistry (17x17 matrix as Map)
      src/lifecycle.ts       # AgentLifecycle state machine
      src/spawn.ts           # Agent spawning via @aix/core
      src/researcher.ts      # Auto-research with mutation strategies
    marketplace/             # @aix/marketplace — registry, reviews, billing
      src/marketplace.ts     # Marketplace aggregate root
      src/registry.ts        # AgentRegistry (CRUD, search, filter)
      src/billing.ts         # BillingEngine (70/30 split)
      src/review.ts          # ReviewPipeline (auto + human escalation)
      src/featured.ts        # FeaturedEngine (ML-ranked scoring)
    mesh/                    # @aix/mesh — device discovery, sync
      src/mesh.ts            # PersonalMesh aggregate root
      src/device.ts          # MeshDevice, DeviceType, DeviceZone
      src/discovery.ts       # DiscoveryService (Map-backed, O(1) lookups)
      src/sync.ts            # SyncProtocol, transport selection
      src/failover.ts        # FailoverPolicy, DegradationLevel
    billing/                 # @aix/billing — subscriptions, tiers
      src/subscription.ts    # Subscription aggregate with state machine
      src/tier.ts            # 6 tiers, TierLimits, enforcement
      src/capability-enforcement.ts  # TierCapabilityEnforcer
      src/family.ts          # FamilyGroup (6-member max)
      src/developer.ts       # DeveloperAccount (70/30 split)
    federation/              # @aix/federation — federated learning cycles
      src/cycle.ts           # FederationCycle aggregate root
      src/anonymizer.ts      # Delegates to Rust for Laplace noise
      src/aggregator.ts      # 1000-user threshold, pre-grouped by domain
      src/contribution.ts    # Pseudonymous keys, SHA-256
    swarm/                   # @aix/swarm — orchestration, zones
      src/cluster.ts         # SwarmCluster aggregate root
      src/consensus.ts       # PBFT/Raft/Gossip layers
      src/sandbox.ts         # SandboxManager, ResourceEnvelope
      src/browser-pool.ts    # BrowserComputePool with priority queue
      src/zones.ts           # 6 canonical zones
    plugin/                  # @aix/plugin — plugin registry
      src/registry.ts        # PluginRegistry with safety engine
      src/types.ts           # DomainPlugin interface, ActionDefinition
    rlm/                     # @aix/rlm — vLLM HTTP client
      src/client.ts          # VllmClient (fetch, SSE streaming, retry)
  package.json               # Root workspace: { "workspaces": ["packages/*"] }
  tsconfig.base.json         # Shared: ES2022, NodeNext, strict
  vitest.workspace.ts        # Vitest workspace config
```

## Build & Test Commands

```bash
# === TypeScript ===
npm install                    # Install all workspace dependencies
npm run build:ts               # Build all TS packages (tsup → CJS + ESM + .d.ts)
npm run test:ts                # Run all vitest tests (802 tests)

# Per-package testing
cd packages/shared && npx vitest run    # 65 tests — types, events, errors
cd packages/agents && npx vitest run    # 140 tests — permissions, lifecycle
cd packages/marketplace && npx vitest run  # 120 tests — registry, reviews
cd packages/mesh && npx vitest run      # 97 tests — device, sync, failover
cd packages/swarm && npx vitest run     # 72 tests — consensus, sandbox
cd packages/billing && npx vitest run   # 71 tests — tiers, enforcement
cd packages/federation && npx vitest run  # 50 tests — cycles, anonymizer
cd packages/mcp-server && npx vitest run  # 46 tests — tools, RBAC
cd packages/rlm && npx vitest run       # 35 tests — HTTP client
cd packages/plugin && npx vitest run    # 74 tests — registry, safety
cd packages/core && npx vitest run      # 17 tests — loader, degradation
cd packages/aix && npx vitest run       # 15 tests — CLI commands

# Typecheck all packages
for pkg in shared core rlm plugin billing agents mesh federation marketplace swarm mcp-server aix; do
  npx tsc --noEmit -p packages/$pkg/tsconfig.json
done

# === Combined (Rust + TypeScript) ===
npm run build                  # build:native + build:wasm + build:ts
npm run test                   # test:rust + test:ts
```

## NAPI Bridge (ADR-027)

The `@aix/core` package exposes 31 NAPI functions across 6 groups. TypeScript packages call these for Rust compute:

| Group | Functions | Purpose |
|-------|-----------|---------|
| **Kernel (6)** | `napiDispatch`, `napiSonaQuery`, `napiRvfSeal`, `napiAgentSpawn`, `napiSwarmStatus`, `napiRoute` | Original kernel operations |
| **Voice (6)** | `napiVoicePipelineCreate`, `napiVoiceVadProcess`, `napiVoiceTranscribe`, `napiVoiceDecomposeIntents`, `napiVoiceSynthesize`, `napiVoiceSessionCreate` | Voice pipeline ops |
| **Phone (8)** | `napiPhoneRuntimeCreate`, `napiPhoneEngagementScore`, `napiPhoneSavingsRecord`, `napiPhoneStreakCheckIn/Status`, `napiPhoneBatteryPolicy`, `napiPhoneCoordinatorStatus`, `napiPhoneNotificationSend` | Phone runtime ops |
| **Cognitive (5)** | `napiSonaRecord`, `napiSonaAdapt`, `napiVoicePatternSearch`, `napiFederatedAnonymize`, `napiFatigueCheck` | SONA + privacy |
| **RVF (1)** | `napiRvfVerify` | Witness verification |
| **Inference (5)** | `napiInferenceLoad`, `napiInferenceGenerate`, `napiInferenceStatus`, `napiInferenceTieredRoute`, `napiInferenceUnload` | Model management |

### Graceful Degradation

When `@aix/core` binary is unavailable, all functions return `{ status: 'unavailable', reason: 'native module not loaded' }`. This allows TS packages to work in degraded mode (e.g., demo data).

```typescript
import { napiDispatch, isNativeAvailable } from '@aix/core';

if (isNativeAvailable) {
  const result = await napiDispatch('VecSearch', { query: [1.0, 0.0] });
} else {
  // Fallback to demo/stub behavior
}
```

## DDD Context Map (Post-Migration)

### Preserved in Rust (accessed via `@aix/core`)
| Context | DDD | Access Pattern |
|---------|-----|----------------|
| Kernel Syscall | DDD-001 | `napiDispatch()` |
| Inference Routing | DDD-004 | `napiRoute()`, `napiInference*()` |
| Observation & Health | DDD-006 | `napiSona*()`, `napiFatigue*()` |
| Container & Storage | DDD-007 | `napiRvfSeal()`, `napiRvfVerify()` |
| Voice Interaction | DDD-008 | `napiVoice*()` |
| Phone Runtime | DDD-009 | `napiPhone*()` |

### Migrated to TypeScript
| Context | DDD | Package | Aggregate Root |
|---------|-----|---------|----------------|
| Agent Lifecycle | DDD-002 | `@aix/agents` | `AgentRegistry` |
| Swarm Coordination | DDD-003 | `@aix/swarm` | `SwarmCluster` |
| Research & Evolution | DDD-005 | `@aix/agents` | `Researcher` |
| Agent Marketplace | DDD-010 | `@aix/marketplace` | `Marketplace` |
| Personal Mesh | DDD-011 | `@aix/mesh` | `PersonalMesh` |
| Federated Learning | DDD-012 | `@aix/federation` | `FederationCycle` |
| Subscription Billing | DDD-013 | `@aix/billing` | `Subscription` |
| NAPI Core Bridge | DDD-014 | `@aix/core` | `NapiKernel` |

### Event Flows
- **Rust-internal**: `SyscallDispatched`, `QueryRouted`, `VoiceSessionStarted` — stay in Rust
- **NAPI boundary**: Rust -> JS via callback (e.g., `AgentSpawned`, `EngagementUpdated`)
- **TS-internal**: `AgentRegistered`, `SubscriptionChanged`, `MeshDeviceJoined` — EventEmitter-based

## Invariant Preservation

All critical invariants from the Rust codebase are preserved in TypeScript:

| Invariant | Enforcement Location |
|-----------|---------------------|
| Voice privacy (no audio off device) | Stays in Rust via `@aix/core` |
| Federation privacy (Laplace e=1.0) | `napiFederatedAnonymize()` in Rust |
| Free tier: 5 agents | `@aix/billing` TierCapabilityEnforcer |
| Permission matrix (17x17) | `@aix/agents` PermissionRegistry |
| LifeScore floor 30 | Stays in Rust via `@aix/core` |
| Marketplace 70/30 split | `@aix/marketplace` BillingEngine |
| Mesh privacy anchor | `@aix/mesh` PersonalMesh |
| 1000-user aggregation threshold | `@aix/federation` FederatedAggregator |

## TypeScript Conventions

- **Build tool**: tsup (esbuild-based) — CJS + ESM + .d.ts in one pass
- **Test tool**: vitest — fast, ESM-native
- **Workspace**: npm workspaces (`packages/*`)
- **Types**: Import from `@aix/shared` — never re-define kernel types
- **Events**: `DomainEventBus` (from `@aix/shared`) or crate-local `EventEmitter`
- **Errors**: Per-package error class with `Object.setPrototypeOf` for `instanceof`
- **IDs**: Use `generateId()` from `@aix/shared` (wraps `crypto.randomUUID()`)
- **Graceful degradation**: All packages handle `@aix/core` unavailability
- **Immutable updates**: Functions return new objects, don't mutate inputs

## Per-Package Test Counts (802 total)

```
@aix/agents: 140      @aix/marketplace: 120    @aix/mesh: 97
@aix/plugin: 74        @aix/swarm: 72           @aix/billing: 71
@aix/shared: 65        @aix/federation: 50      @aix/mcp-server: 46
@aix/rlm: 35           @aix/core: 17            aix: 15
```

## Archive Strategy

When a TypeScript package passes its ported test suite:
1. Move `crates/rlmx-<name>/` -> `crates/archive/rlmx-<name>/`
2. Remove from `Cargo.toml` workspace `members`
3. Verify `cargo build --workspace` compiles clean
4. Archived crates remain as reference — not compiled, not tested

**Post-archive Rust workspace (9 crates)**: rlmx-kernel, rlmx-cognitive, rlmx-ruvllm, rlmx-trm, rlmx-rvf, rlmx-voice, rlmx-phone, rlmx-napi, rlmx-wasm

## Next Steps

1. **Expand NAPI bridge** — implement 25 new functions in `crates/rlmx-napi/src/lib.rs`
2. **Build WASM package** — `wasm-pack build` -> `@aix/wasm`
3. **Archive ported crates** — move 10 Rust crates to `crates/archive/`
4. **Integration tests** — end-to-end tests crossing the NAPI boundary
5. **CI pipeline** — GitHub Actions: `cargo test` + `npm test` + typecheck
6. **npm publish** — register `@aix` namespace, per-platform NAPI binaries
7. **Error hierarchy unification** — make domain errors extend `AixError`
8. **TypedEventBus** — extract generic from `DomainEventBus` for all packages

## Related Documents

- [ADR-026: TypeScript Migration Strategy](ADR/ADR-026-typescript-migration-strategy.md)
- [ADR-027: NAPI Bridge Expansion](ADR/ADR-027-napi-bridge-expansion.md)
- [ADR-028: @aix Package Architecture](ADR/ADR-028-aix-package-architecture.md)
- [DDD-014: NAPI Core Bridge Context](DDD/DDD-014-napi-core-bridge-context.md)
- [DDD-015: TypeScript Package Context Map](DDD/DDD-015-typescript-package-context-map.md)
- [move-to-typescript-plan.md](../archives/root/move-to-typescript-plan.md) — original migration plan (archived)
