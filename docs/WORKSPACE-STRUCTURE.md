# Workspace Structure

## Rust Crates (22)

```
rlmx-cli (binary)
  ├── rlmx-kernel       (core — 18 syscall permissions, router, events, approval, triggers, a2a)
  ├── rlmx-mcp          -> rlmx-kernel, rlmx-rvf, rlmx-ruvllm
  ├── rlmx-swarm        -> rlmx-kernel
  ├── rlmx-agents       -> rlmx-kernel, rlmx-cognitive
  ├── rlmx-voice        -> rlmx-kernel (voice pipeline, intent decomposition)
  ├── rlmx-phone        -> rlmx-kernel (mobile runtime, engagement)
  ├── rlmx-marketplace  -> rlmx-kernel, rlmx-rvf (agent marketplace)
  ├── rlmx-napi         -> rlmx-kernel (Node.js native bindings via napi-rs, ADR-020)
  ├── rlmx-wasm         -> rlmx-kernel (WASM kernel subset for browser, ADR-021)
  ├── rlmx-mesh         -> rlmx-kernel (personal mesh topology, ADR-022)
  ├── rlmx-federation   -> rlmx-kernel, rlmx-cognitive (federated learning, ADR-023)
  ├── rlmx-billing      -> rlmx-kernel (subscription billing + budget ledger, ADR-025/032)
  ├── rlmx-artifact     (standalone — content-addressed DAG, ADR-030)
  ├── rlmx-evolve       (standalone — dynamic function evolution, ADR-036)
  ├── rlmx-channels     (standalone — Telegram/WhatsApp/Teams/Discord, ADR-039)
  ├── rlmx-rvf          (standalone — container format)
  ├── rlmx-plugin       (standalone — domain plugins)
  └── rlmx-ruvllm       (standalone — feature-gated edge inference)

rlmx-rlm               (standalone — vLLM HTTP client)
rlmx-trm               (standalone — pure numeric NN)
rlmx-cognitive          (standalone — SONA self-learning, voice patterns)
```

## TypeScript Packages (packages/)

Per ADR-026/027/028/030-039:

| Package | Description |
|---------|-------------|
| `shared` | `@aix/shared` — pure TS types, events, errors (zero deps) |
| `core` | `@aix/core` — NAPI loader, 31 function declarations, graceful degradation |
| `rlm` | `@aix/rlm` — vLLM HTTP client (fetch, SSE, retry) |
| `plugin` | `@aix/plugin` — plugin registry, safety engine |
| `billing` | `@aix/billing` — subscription tiers, enforcement, family, developer |
| `agents` | `@aix/agents` — 17x17 permissions, lifecycle, spawning, researcher |
| `mesh` | `@aix/mesh` — PersonalMesh, discovery, sync, failover |
| `federation` | `@aix/federation` — cycles, anonymizer, aggregator, bootstrap |
| `marketplace` | `@aix/marketplace` — registry, reviews, featured, publisher, analytics |
| `swarm` | `@aix/swarm` — cluster, consensus, sandbox, fleet, browser-pool |
| `mcp-server` | `@aix/mcp-server` — 47 MCP tools, RBAC, WebSocket events |
| `aix` | aix CLI — commander+chalk+ora, 14 command groups |
| `deploy` | `@aix/deploy` — universal agent deployment + bridge adapters (ADR-029/033) |
| `artifact` | `@aix/artifact` — content-addressed artifact DAG client (ADR-030) |
| `a2a` | `@aix/a2a` — A2A protocol: agent cards, task store, JSON-RPC server/client (ADR-034) |
| `skills` | `@aix/skills` — skill registry, SKILL.md parser, 12 bundled domain skills (ADR-035) |
| `evolve` | `@aix/evolve` — function evolution lifecycle, scoring, feedback (ADR-036) |
| `triggers` | `@aix/triggers` — declarative trigger registry, event/HTTP/schedule/channel bindings (ADR-038) |
| `channels` | `@aix/channels` — messaging channel adapters: Telegram, WhatsApp, Teams, Discord (ADR-039) |

## Additional Directories

| Path | Purpose |
|------|---------|
| `mobile/` | React Native mobile app (8 screens, 11 components, TypeScript) |
| `frontend/index.html` | Single-page web UI (20+ views) + WASM module |
| `frontend/dashboard/` | Svelte 5 + TailwindCSS v4 dashboard (Vite, :5173) |
| `deploy/` | Systemd service for RPi5 |
| `scripts/hooks/` | 5 preconfigured event hooks |
| `docs/ADR/` | 39 Architecture Decision Records |
| `docs/DDD/` | 16 Domain-Driven Design documents |
| `.cargo/` | Cross-compilation config |

## DDD Bounded Contexts (16)

| # | Context | Crate/Package | Aggregate Root |
|---|---------|-------|----------------|
| 1 | Kernel Syscall | `rlmx-kernel` | KernelContext |
| 2 | Agent Lifecycle | `rlmx-agents` | AgentRegistry |
| 3 | Swarm Coordination | `rlmx-swarm` | SwarmCluster |
| 4 | Inference Routing | `rlmx-kernel`, `rlmx-ruvllm` | Scheduler |
| 5 | Research & Evolution | `rlmx-agents` | Researcher |
| 6 | Observation & Health | `rlmx-cognitive` | Sona |
| 7 | Container & Storage | `rlmx-rvf` | RvfContainer |
| 8 | Voice Interaction | `rlmx-voice` | VoiceSession |
| 9 | Phone Runtime | `rlmx-phone` | PhoneRuntime |
| 10 | Agent Marketplace | `rlmx-marketplace` | Marketplace |
| 11 | Personal Mesh | `rlmx-mesh` | PersonalMesh |
| 12 | Federated Learning | `rlmx-federation` | FederationCycle |
| 13 | Subscription Billing | `rlmx-billing` | Subscription |
| 14 | NAPI Core Bridge | `rlmx-napi` | NapiKernel |
| 15 | TypeScript Packages | `packages/*` | (context map) |
| 16 | Agent Deployment | `packages/deploy` | ManifestRegistry |
