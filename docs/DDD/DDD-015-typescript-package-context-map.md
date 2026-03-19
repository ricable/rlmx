# DDD-015: TypeScript Package Context Map

## Overview
This document maps the 13 TypeScript `@aix/*` packages to the existing 13 bounded contexts (DDD-001 through DDD-013) and the new NAPI bridge context (DDD-014). The TypeScript migration (ADR-026) replaces 10 Rust crate implementations with TypeScript packages while preserving the bounded context boundaries, aggregate root ownership, and domain event flows defined in the original DDD documents.

## Context Map

### Preserved Contexts (Rust — no TypeScript replacement)
These bounded contexts remain entirely in Rust, accessed only through `@aix/core` NAPI functions:

| Context | DDD | Rust Crate | Access via |
|---------|-----|------------|------------|
| Kernel Syscall | DDD-001 | rlmx-kernel | `@aix/core.napiDispatch()` |
| Inference Routing | DDD-004 | rlmx-kernel, rlmx-ruvllm | `@aix/core.napiRoute()`, `@aix/core.napiInference*()` |
| Observation & Health | DDD-006 | rlmx-cognitive | `@aix/core.napiSona*()`, `@aix/core.napiFatigue*()` |
| Container & Storage | DDD-007 | rlmx-rvf | `@aix/core.napiRvfSeal()`, `@aix/core.napiRvfVerify()` |
| Voice Interaction | DDD-008 | rlmx-voice | `@aix/core.napiVoice*()` |
| Phone Runtime | DDD-009 | rlmx-phone | `@aix/core.napiPhone*()` |

### Migrated Contexts (Rust → TypeScript)
These bounded contexts move their orchestration logic to TypeScript while delegating compute to `@aix/core`:

| Context | DDD | Rust Crate (archived) | TypeScript Package | Aggregate Root |
|---------|-----|-----------------------|--------------------|----------------|
| Agent Lifecycle | DDD-002 | rlmx-agents | `@aix/agents` | `AgentRegistry` |
| Swarm Coordination | DDD-003 | rlmx-swarm | `@aix/swarm` | `SwarmCluster` |
| Research & Evolution | DDD-005 | rlmx-agents | `@aix/agents` | `Researcher` |
| Agent Marketplace | DDD-010 | rlmx-marketplace | `@aix/marketplace` | `Marketplace` |
| Personal Mesh | DDD-011 | rlmx-mesh | `@aix/mesh` | `PersonalMesh` |
| Federated Learning | DDD-012 | rlmx-federation | `@aix/federation` | `FederationCycle` |
| Subscription Billing | DDD-013 | rlmx-billing | `@aix/billing` | `Subscription` |

### New Contexts

| Context | DDD | Package | Purpose |
|---------|-----|---------|---------|
| NAPI Core Bridge | DDD-014 | `@aix/core` | Anti-corruption layer between Rust and TypeScript |

### Support Packages (no dedicated bounded context)

| Package | Role | Serves |
|---------|------|--------|
| `@aix/shared` | Shared kernel (DDD term) — common types | All packages |
| `@aix/rlm` | Infrastructure service — vLLM HTTP client | `@aix/mcp-server`, `aix` CLI |
| `@aix/plugin` | Infrastructure service — plugin registry | `@aix/mcp-server`, `aix` CLI |
| `@aix/mcp-server` | Application service — HTTP/WS/JSON-RPC server | External clients |
| `@aix/wasm` | Infrastructure service — browser kernel subset | Browser clients |
| `aix` | Application service — CLI entry point | End users |

## Dependency Flows

### Upstream/Downstream Relationships

```
@aix/shared (Shared Kernel)
    ↑ used by all packages

@aix/core (NAPI Bridge — DDD-014)
    ↑ used by: agents, swarm, marketplace, mesh, federation, billing, mcp-server

@aix/agents (DDD-002, DDD-005)
    ↑ used by: swarm, mcp-server

@aix/billing (DDD-013)
    ↑ used by: marketplace, mcp-server

@aix/marketplace (DDD-010)
    ↑ used by: mcp-server

@aix/mesh (DDD-011)
    ↑ used by: mcp-server

@aix/federation (DDD-012)
    ↑ used by: mcp-server

@aix/swarm (DDD-003)
    ↑ used by: mcp-server

@aix/mcp-server (Application Service)
    ↑ used by: aix CLI

aix (CLI Entry Point)
    → top-level, depends on all @aix/* packages
```

### Anti-Corruption Layer Patterns

Each migrated TypeScript package interacts with Rust compute through `@aix/core` using one of two patterns:

**Pattern 1: Direct Bridge Call**
Used when the TypeScript package needs a single Rust computation:
```typescript
// @aix/agents — spawning calls kernel
import { napiAgentSpawn } from '@aix/core';

export class AgentRegistry {
  async spawn(type: AgentType, config: AgentConfig): Promise<Agent> {
    const result = await napiAgentSpawn(type, config);
    return this.toAgent(result); // ACL: Rust result → TS domain object
  }
}
```

**Pattern 2: Delegated Compute**
Used when the TypeScript package orchestrates a multi-step flow with Rust compute in the middle:
```typescript
// @aix/federation — orchestration in TS, privacy in Rust
import { napiFederatedAnonymize, napiSonaRecord } from '@aix/core';

export class FederationCycle {
  async contribute(patterns: Pattern[]): Promise<Contribution> {
    // Step 1: Anonymize in Rust (privacy invariant)
    const anonymized = await napiFederatedAnonymize(JSON.stringify(patterns));
    // Step 2: Orchestrate submission in TypeScript
    return this.submitToAggregator(JSON.parse(anonymized));
  }
}
```

## Domain Event Flows (Post-Migration)

### Events that stay in Rust (emitted and consumed within @aix/core)
- `SyscallDispatched` — kernel internal
- `QueryRouted` — router internal
- `VoiceSessionStarted` / `IntentsDecomposed` — voice pipeline internal

### Events that cross the NAPI boundary (Rust → TypeScript)
- `AgentSpawned` — `@aix/core` emits → `@aix/agents` consumes (lifecycle tracking)
- `EngagementUpdated` — `@aix/core` emits → `@aix/mcp-server` broadcasts via WebSocket

### Events that stay in TypeScript (emitted and consumed in TS packages)
- `AgentRegistered` — `@aix/agents` → `@aix/marketplace` (listing update)
- `SubscriptionChanged` — `@aix/billing` → `@aix/agents` (tier enforcement)
- `MeshDeviceJoined` — `@aix/mesh` → `@aix/mcp-server` (WebSocket broadcast)
- `FederationCycleCompleted` — `@aix/federation` → `@aix/mcp-server` (status update)
- `MarketplaceAgentInstalled` — `@aix/marketplace` → `@aix/agents` (agent provisioning)

### Event Transport
- **Within Rust**: `DomainEventBus` (existing, unchanged)
- **NAPI boundary**: Callback-based — Rust calls registered JS callback on event emission
- **Within TypeScript**: EventEmitter-based bus in `@aix/shared`:
```typescript
export class DomainEventBus {
  private emitter = new EventEmitter();
  emit(event: DomainEvent): void { this.emitter.emit(event.type, event); }
  on(type: string, handler: (event: DomainEvent) => void): void { ... }
}
```

## Invariant Preservation

| Invariant | Original Enforcement | Post-Migration Enforcement |
|-----------|---------------------|---------------------------|
| Voice privacy (no audio off device) | rlmx-voice in Rust | Unchanged — stays in Rust via @aix/core |
| Federation privacy (Laplace e=1.0) | rlmx-cognitive in Rust | `napiFederatedAnonymize()` — stays in Rust |
| Tier enforcement (5 agent limit) | rlmx-billing in Rust | `@aix/billing` in TS calls `@aix/core` for token validation |
| Permission matrix (17x17) | rlmx-agents in Rust | `@aix/agents` in TS — matrix is data, not compute |
| Engagement invariants (LifeScore floor 30) | rlmx-phone in Rust | Unchanged — stays in Rust via @aix/core |
| Marketplace security review | rlmx-marketplace in Rust | `@aix/marketplace` in TS — review logic is CRUD, not crypto |
| Mesh privacy anchor | rlmx-mesh in Rust | `@aix/mesh` in TS — policy enforcement, sync via @aix/core |

## Migration Verification Checklist

For each migrated bounded context:
1. TypeScript package exports the same aggregate root interface as the Rust crate
2. All domain events are emitted at the same trigger points
3. All invariants from the DDD document are enforced (in TS or delegated to Rust)
4. Rust crate tests are ported to vitest with equivalent coverage
5. Anti-corruption layer correctly translates between @aix/core results and domain objects
6. Graceful degradation works when @aix/core is unavailable

## Ubiquitous Language Additions
- **Shared Kernel** (DDD term): `@aix/shared` — types shared across all packages without ownership by any single context
- **Bridge Call**: A TypeScript function invoking a Rust function via @aix/core NAPI
- **Event Crossing**: A domain event that originates in Rust and is consumed in TypeScript (or vice versa)
- **Archived Crate**: A Rust crate whose orchestration logic has been replaced by a TypeScript package; kept in `crates/archive/` for reference
- **Graceful Degradation**: TypeScript packages returning `{ status: 'unavailable' }` when @aix/core cannot load
