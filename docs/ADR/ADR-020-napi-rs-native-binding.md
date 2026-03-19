# ADR-020: NAPI-RS Native Binding Layer

| Field    | Value                     |
|----------|---------------------------|
| Status   | Implemented               |
| Date     | 2026-03-19                |
| Authors  | Cedric                    |
| Replaces | —                         |

## Context

RuVix Mesh needs a native Node.js binding layer so the RLMX cognition kernel can be used from Electron desktop apps, VS Code extensions, CLI tools, and Claude Code integrations — all on the laptop (Zone A). The binding must expose kernel syscalls with zero HTTP overhead (sub-millisecond dispatch). Currently, external consumers must go through the MCP HTTP server (ADR-010) at `:3000`, adding network latency and serialization overhead that is unacceptable for tight integration loops such as VS Code inline completions or Electron real-time dashboards.

## Decision

Create a new `rlmx-napi` crate using napi-rs to compile the kernel into a native Node.js addon, published as `@ruvix/mesh-native` npm package.

### Exposed API Surface

```rust
#[napi]
pub fn dispatch(syscall: String, params: JsObject) -> Result<JsObject>;

#[napi]
pub async fn sona_query(embedding: Vec<f32>, k: u32) -> Result<Vec<PatternMatch>>;

#[napi]
pub fn rvf_seal(data: Buffer) -> Result<ProofResult>;

#[napi]
pub async fn agent_spawn(agent_type: String, config: JsObject) -> Result<AgentHandle>;

#[napi]
pub fn swarm_status() -> Result<SwarmHealth>;

#[napi]
pub fn route(query: String) -> Result<RoutingDecision>;
```

- `dispatch(syscall, params)` — All 15 kernel syscalls (ADR-019)
- `sona_query(embedding, k)` — SONA pattern bank search
- `rvf_seal(data)` — ProofEngine witness sealing
- `agent_spawn(agent_type, config)` — Agent lifecycle management
- `swarm_status()` — Cross-zone swarm health
- `route(query)` — TinyDancerRouter strategy selection (18-dim, ADR-003/019)

### Build Targets

| Target Triple                  | Platform             | GPU Feature |
|-------------------------------|----------------------|-------------|
| `aarch64-apple-darwin`        | Mac Apple Silicon    | Metal       |
| `x86_64-unknown-linux-gnu`    | Linux x86_64         | None        |
| `x86_64-pc-windows-msvc`     | Windows (future)     | None        |

### Architecture

- NAPI-RS generates N-API bindings from Rust structs annotated with `#[napi]`
- Async operations use napi-rs `AsyncTask` trait backed by Tokio runtime
- The kernel instance is wrapped in `Arc<RwLock<Kernel>>` shared across JS calls
- Capability tokens are validated on every dispatch call — no bypass path (ADR-005)
- SONA master pattern bank consolidation happens on the laptop (Zone A)

```
Node.js Process
  └── @ruvix/mesh-native (native addon)
        ├── napi-rs FFI layer
        │     ├── JsObject ↔ Rust serde conversion
        │     └── AsyncTask ↔ Tokio spawn_blocking bridge
        └── Arc<RwLock<Kernel>>
              ├── rlmx-kernel (15 syscalls, TinyDancerRouter)
              ├── rlmx-cognitive (SONA pattern bank)
              ├── rlmx-rvf (container sealing)
              ├── rlmx-agents (spawn, lifecycle)
              └── rlmx-swarm (zone coordination)
```

### Integration Points

| Crate             | Usage                                      |
|-------------------|--------------------------------------------|
| `rlmx-kernel`    | All syscall dispatch and type re-exports   |
| `rlmx-cognitive`  | SONA pattern bank queries                  |
| `rlmx-rvf`       | Container sealing and verification         |
| `rlmx-agents`    | Agent spawn and lifecycle                  |
| `rlmx-swarm`     | Swarm status and zone coordination         |

### Feature Gates

- `napi` — Enables NAPI-RS compilation (opt-in, not in default workspace build)
- `metal` — Enables Metal GPU inference on macOS
- Combined: `--features "napi,metal"` for full Mac build

The `rlmx-napi` crate is excluded from the default workspace members list. It is built separately:

```bash
# Mac Apple Silicon (full)
cd crates/rlmx-napi && npm run build -- --features "napi,metal"

# Linux (no GPU)
cd crates/rlmx-napi && npm run build -- --features "napi"
```

This preserves the strict rule: `cargo build --workspace` compiles clean without napi-rs dependencies.

### Crate Layout

```
crates/rlmx-napi/
  ├── Cargo.toml          # napi-rs deps behind `napi` feature gate
  ├── package.json        # @ruvix/mesh-native npm metadata
  ├── build.rs            # napi-rs build script
  ├── src/
  │   ├── lib.rs          # #[napi] exports, Kernel singleton init
  │   ├── dispatch.rs     # Syscall dispatch bridge
  │   ├── sona.rs         # SONA query bridge
  │   ├── agents.rs       # Agent spawn/lifecycle bridge
  │   └── swarm.rs        # Swarm status bridge
  └── __test__/
      └── index.spec.ts   # Node.js integration tests
```

### Capability Token Validation

Every call through the NAPI boundary requires a valid capability token:

```typescript
import { createKernel } from '@ruvix/mesh-native';

const kernel = createKernel({ token: process.env.RLMX_TOKEN });
const result = await kernel.dispatch('VecSearch', { query: embedding, k: 5 });
```

Tokens are validated against the same RBAC rules as MCP (ADR-005). The NAPI layer does not introduce a second auth path — it reuses `rlmx-kernel` token validation. Clients cannot self-escalate.

## Consequences

### Positive

- Sub-millisecond syscall dispatch from Node.js (no HTTP/IPC overhead)
- Single binary ships as npm package — `npm install @ruvix/mesh-native`
- Type-safe bindings auto-generated from Rust types via napi-rs
- Coordinator agent on laptop gets direct kernel access (Zone A-Desktop)
- Enables VS Code extension and Electron desktop app without running MCP server
- Same capability token model as MCP — no separate auth system to maintain

### Negative

- Adds napi-rs build complexity to CI (native compilation per platform)
- Node.js version compatibility constraints (N-API version 6+ floor, Node 14+)
- Debugging native crashes requires Rust toolchain knowledge
- Separate build step outside `cargo build --workspace` (by design, to avoid pulling napi-rs into default build)

### Risks

- napi-rs upstream breaking changes (mitigated: pin to stable release, e.g. `napi = "2.x"`)
- Memory safety at FFI boundary (mitigated: napi-rs handles prevent raw pointer exposure)
- Tokio runtime lifecycle management — the Node.js event loop and Tokio runtime must coexist without deadlocks (mitigated: napi-rs `AsyncTask` uses `spawn_blocking` to bridge the two)

## References

- PRD: "RuVix Mesh — The Personal Agent Cloud", Step 1
- ADR-005: Capability-Secured Agents (token validation at NAPI boundary)
- ADR-007: RuVix Kernel Migration (kernel type system)
- ADR-010: MCP Tool Expansion (existing MCP HTTP interface this complements)
- ADR-019: Kernel Expansion (15 syscalls, 18-dim router)
