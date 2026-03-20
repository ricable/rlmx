# ADR-028: @aix Package Architecture

Status: Proposed

## Context
The TypeScript migration (ADR-026) produces 13 npm packages that need a coherent workspace layout, build tooling, and distribution strategy. The architecture follows the ruvector packaging pattern: per-platform NAPI binaries as optional dependencies, a JavaScript loader, TypeScript declarations, and a top-level CLI binary.

## Decision
Structure the TypeScript codebase as an npm workspace under `packages/` with the `@aix` namespace, using tsup for builds, vitest for testing, and napi-rs for native bindings.

### Directory Layout

```
rlmx/
  packages/
    aix/                          # `npx aix` CLI entry point
    core/                         # @aix/core — NAPI JS loader + .d.ts
    core-darwin-arm64/            # Per-platform .node binaries
    core-darwin-x64/
    core-linux-x64-gnu/
    core-linux-arm64-gnu/
    core-win32-x64-msvc/
    wasm/                         # @aix/wasm — wasm-pack output
    shared/                       # @aix/shared — pure TS types (zero deps)
    mcp-server/                   # @aix/mcp-server — MCP SDK server
    agents/                       # @aix/agents — permissions, lifecycle
    marketplace/                  # @aix/marketplace — registry, reviews
    mesh/                         # @aix/mesh — device discovery, sync
    billing/                      # @aix/billing — Stripe, tiers
    swarm/                        # @aix/swarm — orchestration
    federation/                   # @aix/federation — cycle management
    plugin/                       # @aix/plugin — plugin registry
    rlm/                          # @aix/rlm — vLLM HTTP client
  crates/                         # 9 Rust crates (unchanged)
  package.json                    # Root workspace config
  tsconfig.base.json              # Shared TS compiler options
```

### Package Dependency Graph

```
                        aix (CLI)
                       /    |    \
              @aix/mcp-server  @aix/swarm  ...all @aix/*
                 /     \         /    \
         @aix/agents  @aix/core  @aix/agents
            |            |            |
        @aix/shared  @aix/shared  @aix/shared
```

### Package Specifications

| Package | npm Name | Type | Key Dependencies | Exports |
|---------|----------|------|------------------|---------|
| aix | `aix` | CLI binary | all @aix/*, commander, chalk, ora | `bin/cli.js` |
| core | `@aix/core` | NAPI loader | per-platform optionalDeps | ~31 native functions |
| wasm | `@aix/wasm` | WASM module | none | `aix_wasm.js`, `_bg.wasm` |
| shared | `@aix/shared` | Pure types | none | TS enums, interfaces, utilities |
| mcp-server | `@aix/mcp-server` | Server | @modelcontextprotocol/sdk, ws | McpServer, tool handlers |
| agents | `@aix/agents` | Library | @aix/core, @aix/shared | PermissionRegistry, AgentLifecycle |
| marketplace | `@aix/marketplace` | Library | @aix/core, @aix/shared | Marketplace, AgentRegistry |
| mesh | `@aix/mesh` | Library | @aix/core, @aix/shared, mdns-js, ws | PersonalMesh, DiscoveryService |
| billing | `@aix/billing` | Library | @aix/core, @aix/shared, stripe | Subscription, TierEnforcer |
| swarm | `@aix/swarm` | Library | @aix/agents, @aix/core, @aix/shared | SwarmCluster, SandboxManager |
| federation | `@aix/federation` | Library | @aix/core, @aix/shared | FederationCycle, Anonymizer |
| plugin | `@aix/plugin` | Library | @aix/shared | PluginRegistry, DomainPlugin |
| rlm | `@aix/rlm` | Library | @aix/shared | VllmClient, ChatMessage |

### Root Workspace Configuration

```json
{
  "private": true,
  "workspaces": ["packages/*"],
  "scripts": {
    "build:native": "cd crates/rlmx-napi && npx napi build --platform --release --features napi",
    "build:wasm": "wasm-pack build crates/rlmx-wasm --target web --out-dir ../../packages/wasm --out-name aix_wasm -- --features wasm",
    "build:ts": "npm run build --workspaces",
    "build": "npm run build:native && npm run build:wasm && npm run build:ts",
    "test:rust": "cargo test --workspace",
    "test:ts": "npm test --workspaces",
    "test": "npm run test:rust && npm run test:ts"
  }
}
```

### Shared TypeScript Configuration

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "NodeNext",
    "moduleResolution": "NodeNext",
    "strict": true,
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true,
    "outDir": "./dist",
    "rootDir": "./src",
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true
  }
}
```

### Per-Package Build Configuration

Each TypeScript package uses tsup with a standard configuration:
```typescript
// packages/<name>/tsup.config.ts
import { defineConfig } from 'tsup';

export default defineConfig({
  entry: ['src/index.ts'],
  format: ['cjs', 'esm'],
  dts: true,
  clean: true,
  sourcemap: true,
});
```

### @aix/core Distribution Pattern

Following the ruvector model, the `@aix/core` package uses optional dependencies for per-platform binaries:

```json
{
  "name": "@aix/core",
  "main": "index.js",
  "types": "index.d.ts",
  "optionalDependencies": {
    "@aix/core-darwin-arm64": "workspace:*",
    "@aix/core-darwin-x64": "workspace:*",
    "@aix/core-linux-x64-gnu": "workspace:*",
    "@aix/core-linux-arm64-gnu": "workspace:*",
    "@aix/core-win32-x64-msvc": "workspace:*"
  }
}
```

The loader (`index.js`, ~11KB) detects the platform and dynamically requires the correct `.node` binary.

### @aix/shared Type Definitions

Ports kernel types as TypeScript equivalents:

```typescript
// SyscallPermission — 17 variants matching rlmx-kernel
export enum SyscallPermission {
  VecSearch, VecInsert, GraphQuery, HaltCheck, StateMutate,
  LogRead, LogWrite, NetSend, NetRecv, ProcSpawn,
  ProcKill, StorageRead, VoiceTranscribe, VoiceSynthesize,
  IntentRoute, MeshSync, FederationContribute
}

// LifeDomain — 12 variants (closed set per CLAUDE.md rule 31)
export enum LifeDomain {
  Finance, Health, Legal, Career, Education, Home,
  Shopping, Travel, Social, Government, Automotive, Pet
}

// DomainEvent — discriminated union (10 variants)
export type DomainEvent =
  | { type: 'SyscallDispatched'; syscall: SyscallPermission; process_id: string }
  | { type: 'QueryRouted'; strategy: Strategy; confidence: number }
  | { type: 'VoiceSessionStarted'; session_id: string }
  // ... 7 more variants
```

### MCP Server Migration

`@aix/mcp-server` uses the official `@modelcontextprotocol/sdk` instead of custom JSON-RPC:

```typescript
import { Server } from '@modelcontextprotocol/sdk/server/index.js';

const server = new Server({ name: 'aix', version: '1.0.0' }, {
  capabilities: { tools: {} }
});

// 47 tool handlers registered via SDK patterns
server.setRequestHandler(ListToolsRequestSchema, async () => ({
  tools: toolDefinitions // ported from crates/rlmx-mcp/src/tools.rs
}));
```

### CLI Entry Point

```
npx aix serve --port 3000          # → @aix/mcp-server
npx aix query -i "term"            # → @aix/core.napiDispatch
npx aix voice start                # → @aix/core.napiVoicePipelineCreate
npx aix agent spawn --type worker  # → @aix/agents
npx aix marketplace search         # → @aix/marketplace
npx aix mesh status                # → @aix/mesh
npx aix billing status             # → @aix/billing
npx aix federation status          # → @aix/federation
```

## Consequences

### Positive
- `npx aix` — zero-config install, familiar Node.js tooling
- Per-platform binaries — no Rust toolchain required for users
- npm workspace — standard dependency management, no custom build orchestration
- @aix namespace — clean, professional package naming
- tsup + vitest — fast builds and tests for TypeScript packages

### Negative
- 5 per-platform binary packages to publish on every release
- npm workspace hoisting can cause subtle dependency resolution issues
- Two build systems (cargo + npm) must be coordinated
- WASM package requires wasm-pack in the build pipeline

### Risks
- npm registry namespace `@aix` availability (mitigated: register early)
- Per-platform binary size (~15-20MB) may concern users (mitigated: only target platform downloaded)
- TypeScript type drift from Rust source of truth (mitigated: CI job comparing @aix/shared with kernel types)

## References
- ADR-020: NAPI-RS Native Bindings (ruvector distribution pattern)
- ADR-021: WASM Kernel Subset (wasm-pack build)
- ADR-026: TypeScript Migration Strategy (migration phases)
- ADR-027: NAPI Bridge Expansion (function inventory)
- DDD-015: TypeScript Package Context Map
