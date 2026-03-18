# Claude Code Configuration — RLMX Cognition Kernel

## Behavioral Rules (Always Enforced)

- Do what has been asked; nothing more, nothing less
- NEVER create files unless they're absolutely necessary for achieving your goal
- ALWAYS prefer editing an existing file to creating a new one
- NEVER proactively create documentation files (*.md) or README files unless explicitly requested
- NEVER save working files, text/mds, or tests to the root folder
- Never continuously check status after spawning a swarm — wait for results
- ALWAYS read a file before editing it
- NEVER commit secrets, credentials, or .env files

## File Organization

- NEVER save to root folder — use the directories below
- Use `/crates` for Rust source code (each crate has its own `src/`)
- Use `/docs` for documentation, ADRs, and DDD documents
- Use `/frontend` for web UI files (single-page `index.html` — do NOT split into multiple files)
- Use `/deploy` for deployment configs
- Use `/scripts` for utility scripts
- Documentation changes go in `/docs` — do NOT create `.md` files in other directories

## Project Overview

RLMX ("RuVix") is a **cognition kernel** — an OS-kernel-inspired runtime for LLM agents. It provides capability-secured syscall primitives that agents call instead of accessing arbitrary APIs. Written in Rust (edition 2021), async on Tokio.

**11 crates, 28 MCP tools, 12 agent types, 5 swarm zones, 504+ tests, 20+ dashboard views, 11 sandbox profiles.**

The system supports five inference paths:
- **Cloud/Dev**: `VllmClient` → vLLM on Metal (Mac) or NVIDIA GPU (prod)
- **Edge**: `LocalEngine` → ruvllm CandleBackend on CPU/Metal/CUDA with GGUF models
- **Tiered**: `TieredEngine` → Small (0.5B) → Medium (MLX 3-8B) → Remote (vLLM) with confidence escalation
- **Browser**: `@ruvector/ruvllm-wasm` → WebGPU/WASM SIMD (inference primitives + HTTP fallback)
- **Swarm**: Cross-zone scatter-gather via `Strategy::Swarm` with configurable gather strategy

## Build & Development Commands

```bash
# Standard development
cargo build --workspace              # Build all 11 crates
cargo test --workspace               # Run all tests (504+)
cargo test -p rlmx-kernel            # Run tests for a single crate
cargo clippy --workspace -- -D warnings  # Lint (must be zero warnings)
cargo fmt --check                    # Check formatting
cargo fmt                            # Format code

# Run the CLI
cargo run -p rlmx-cli -- <cmd>                  # Run CLI command
cargo run -p rlmx-cli -- serve --port 3000      # Start MCP + WS server

# With edge inference (requires ruvllm + model)
cargo build -p rlmx-cli --features ruvllm        # CPU inference
cargo build -p rlmx-cli --features "ruvllm,metal" # Metal GPU (macOS)

# Start server with edge model
RLMX_EDGE_MODEL=tinyllama-1.1b-q4_k_m \
  cargo run -p rlmx-cli --features "ruvllm,metal" -- serve --port 3000

# Frontend (separate terminal)
cd frontend && python3 -m http.server 8080

# Cross-compilation for RPi5 / ARM64 edge
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu -p rlmx-cli --features ruvllm
```

No Makefile, no CI pipeline. Default rustfmt and clippy settings apply.

## Workspace Structure (11 crates)

```
rlmx-cli (binary)
  ├── rlmx-kernel     (core — router, events, no other rlmx deps)
  ├── rlmx-mcp        → rlmx-kernel, rlmx-rvf, rlmx-ruvllm
  ├── rlmx-swarm      → rlmx-kernel
  ├── rlmx-agents     → rlmx-kernel, rlmx-cognitive
  ├── rlmx-rvf        (standalone)
  ├── rlmx-plugin     (standalone)
  └── rlmx-ruvllm     (standalone — feature-gated)

rlmx-rlm             (standalone — vLLM HTTP client)
rlmx-trm             (standalone — pure numeric NN)
rlmx-cognitive        (standalone — self-learning)
```

Additional directories:
- `frontend/` — Single-page web UI (13 views) + WASM module
- `frontend/dashboard/` — Svelte 5 + TailwindCSS v4 dashboard (Vite, :5173)
- `deploy/` — Systemd service for RPi5
- `docs/ADR/` — 11 Architecture Decision Records
- `docs/DDD/` — 7 Domain-Driven Design documents
- `.cargo/` — Cross-compilation config

## Architecture

### Kernel (`rlmx-kernel`)
**12-syscall dispatch** with `DomainEventBus` (6 cross-context events). `TinyDancerRouter` (FastGRNN 14→32→5) with online learning and 0.6 confidence gating. `Strategy` enum includes `Swarm { scatter_zones, gather_strategy, timeout_ms }`.

### MCP Server (`rlmx-mcp`)
**28 JSON-RPC 2.0 tools**, HTTP :3000, WebSocket :3001 with typed `SwarmEvent` enum (9 variants), auth, heartbeat, backpressure. RBAC: 6 roles, clients cannot self-escalate. Sandbox tools: `rlmx_sandbox_spawn`, `_terminate`, `_status`, `_list`, `rlmx_fleet_deploy`.

### Swarm (`rlmx-swarm`)
**4-zone topology** (A=Compute/PBFT, B=Inference/Raft, C=Edge/Gossip, D=Burst). Strategy-to-Zone mapping with fallbacks. `BrowserComputePool` with priority queue and fault tolerance. `SimulatedSwarm` with per-zone latency config. **Sandbox orchestration** (ADR-011): `SandboxManager` with `SandboxProfile`, `ResourceEnvelope`, `NetworkPolicy`, `FleetManifest`. 11 profiles mapping to AgentType + zone + ModelTier.

### Agents (`rlmx-agents`)
**12 typed roles** with const 12x12 permission matrix. `AgentLifecycle` state machine. Auto-research: `ResearchObjective`, `MutationStrategy` genome, `CrossPollinator`, `FitnessEvaluator`, `CloudEscalation`.

### Tiered Inference (`rlmx-ruvllm`)
`TieredEngine` with escalation (threshold 0.4). `MlxSubprocess` for Apple Silicon. `ModelTier`: Small/ClaudeCode/Medium/Custom.

### Cognitive (`rlmx-cognitive`)
SONA (micro-LoRA + EWC++), DagOptimizer, NervousSystem (BTSP, HDC, WTA, circadian, global workspace).

### Frontend (`frontend/index.html`)
**20+ views** (~2900 lines vanilla JS/CSS/HTML, no framework). Works standalone with demo data or connected to MCP server.

**Key modules in the `<script>` block:**
- `DemoData` — IIFE generating 25 nodes, 8 agents, 3 experiments, 12-node graph, 60-point time series
- `SimEngine` — 2-second setInterval tick updating metrics, generating SwarmEvents
- `TabCoord` — BroadcastChannel('rlmx-swarm') for multi-tab coordination
- `sparkline()` — Inline SVG generator for metric card mini-charts
- `renderForceGraph()` — Fruchterman-Reingold physics for knowledge graph canvas
- `drawTopology()` — Canvas renderer for 5-zone swarm topology
- `viewRenderers` — Object mapping section.view keys to render/onShow functions

**Dashboard rules:**
- The frontend is a SINGLE FILE (`index.html`). Do NOT split it into separate JS/CSS files.
- All action handlers (doAgentList, loadTopology, etc.) must try MCP server first, then fall back to DemoData when server returns empty results (not just on error).
- The `forceGraphRaf` must be cancelled via `cancelAnimationFrame` when switching away from graph view.
- Demo data uses `crypto.randomUUID()` for IDs — must work in secure contexts only.

**WASM init**: call `await mod.default()` before any other API. Do NOT call `mod.init()` after.

## Conventions

- `Arc<Mutex<T>>` or `Arc<RwLock<T>>` for shared mutable state
- Per-crate error types via `thiserror`
- `tracing` for structured logging — no `println!` in library code
- Tests are inline `#[cfg(test)]` blocks
- Feature gates: `ruvllm` and `metal` are opt-in
- `SyscallPermission` defined in `rlmx-kernel`, re-exported by `rlmx-agents` — never duplicate

## Strict Rules

1. **Never break the stub build**: `cargo build --workspace` without features MUST compile clean.
2. **Never add `println!` to library crates**: use `tracing::*`.
3. **Never bypass RBAC**: Admin/System only via server-side `token_roles`.
4. **Keep tests passing**: `cargo test --workspace` must pass all 504+ tests.
5. **Zero clippy warnings**: `cargo clippy --workspace -- -D warnings` must be clean.
6. **Edge tools return `"status": "unavailable"`** when no engine is configured.
7. **GGUF models require companion tokenizers**: `<model>-tokenizer.json` next to `.gguf`.
8. **MCP initialization handshake is mandatory**.
9. **Never duplicate kernel types**: import `SyscallPermission`, `Strategy`, `ProcessId` from `rlmx-kernel`.
10. **Permission matrix is the source of truth**: const `MATRIX` in `registry.rs` per ADR-005.
11. **Domain events must flow**: dispatch() emits SyscallDispatched, resolve_strategy() emits QueryRouted.
12. **Frontend fallback pattern**: Action handlers must check if server returned EMPTY data (e.g., `node_count === 0`, `agents.length === 0`), not just catch errors. The MCP server may respond successfully but with no swarm running.
13. **Single-file frontend**: `frontend/index.html` is one file. Do NOT split into separate JS/CSS files or add npm/bundler tooling.

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RLMX_EDGE_MODEL` | GGUF model name (without .gguf) | unset |
| `RLMX_MODEL_DIR` | Directory for GGUF + tokenizer files | `~/.rlmx/models` |

## Key File Locations

| Path | Purpose |
|------|---------|
| `crates/rlmx-kernel/src/scheduler.rs` | Strategy enum, GatherStrategy, Scheduler |
| `crates/rlmx-kernel/src/router.rs` | TinyDancerRouter (FastGRNN 14→32→5) |
| `crates/rlmx-kernel/src/events.rs` | DomainEvent (6 variants), DomainEventBus |
| `crates/rlmx-mcp/src/tools.rs` | All 28 MCP tool definitions |
| `crates/rlmx-mcp/src/server.rs` | RBAC, tool dispatch, McpConfig |
| `crates/rlmx-mcp/src/ws.rs` | WebSocket server, SwarmEvent (9 variants) |
| `crates/rlmx-swarm/src/consensus.rs` | PBFT/Raft/Gossip layers |
| `crates/rlmx-swarm/src/browser_pool.rs` | BrowserComputePool, priority queue |
| `crates/rlmx-swarm/src/orchestrator.rs` | SwarmOrchestrator, route_syscall() |
| `crates/rlmx-agents/src/registry.rs` | PermissionMatrix (12x12), validate() |
| `crates/rlmx-agents/src/mutation.rs` | MutationStrategy, CrossPollinator |
| `crates/rlmx-agents/src/lifecycle.rs` | AgentLifecycle state machine |
| `crates/rlmx-ruvllm/src/tiered.rs` | TieredEngine, escalation |
| `crates/rlmx-ruvllm/src/mlx_bridge.rs` | MlxSubprocess for Apple Silicon |
| `crates/rlmx-swarm/src/sandbox.rs` | SandboxManager, SandboxProfile, FleetManifest (ADR-011) |
| `crates/rlmx-cli/src/main.rs` | CLI: serve, swarm, agent, research, sandbox, edge |
| `frontend/index.html` | Web dashboard (20+ views, demo data, force graph, sparklines) |
| `frontend/ruvllm-wasm/` | @ruvector/ruvllm-wasm v2.0.2 pre-built WASM module |
| `docs/ADR/` | 11 Architecture Decision Records |
| `docs/DDD/` | 7 Domain-Driven Design documents |

## Concurrency: 1 MESSAGE = ALL RELATED OPERATIONS

- All operations MUST be concurrent/parallel in a single message
- ALWAYS batch ALL file reads/writes/edits in ONE message
- ALWAYS batch ALL Bash commands in ONE message
- ALWAYS use `run_in_background: true` for all agent Task calls
- ALWAYS put ALL agent Task calls in ONE message for parallel execution
- After spawning agents, STOP — wait for results

## Swarm Configuration

- Use hierarchical topology for coding swarms
- Keep maxAgents at 6-12 for tight coordination
- Use specialized strategy for clear role boundaries
- Use `raft` consensus for hive-mind

## Security Rules

- NEVER hardcode API keys, secrets, or credentials in source files
- NEVER commit .env files or any file containing secrets
- Always validate user input at system boundaries
- Always sanitize file paths to prevent directory traversal

## Important Commands Reference

```bash
# === Build & Test ===
cargo build --workspace                          # Build all 11 crates (stub, no ruvllm)
cargo test --workspace                           # Run all 504+ tests
cargo clippy --workspace -- -D warnings          # Lint (must be zero warnings)
cargo fmt --check                                # Check formatting
cargo fmt                                        # Auto-format

# === Run ===
cargo run -p rlmx-cli -- serve --port 3000       # Start MCP + WS server
cd frontend && python3 -m http.server 8080       # Serve dashboard at :8080

# === Edge inference ===
cargo build -p rlmx-cli --features "ruvllm,metal"  # Build with Metal GPU
RLMX_EDGE_MODEL=tinyllama-1.1b-q4_k_m \
  cargo run -p rlmx-cli --features "ruvllm,metal" -- serve --port 3000

# === CLI ===
cargo run -p rlmx-cli -- query -i "search term"    # Semantic search
cargo run -p rlmx-cli -- swarm start --zone A       # Start swarm node
cargo run -p rlmx-cli -- agent spawn --type worker   # Spawn agent
cargo run -p rlmx-cli -- research start --topic "X"  # Start research

# === Cross-compile (RPi5/ARM64) ===
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu -p rlmx-cli --features ruvllm
```

## Dashboard Development

When modifying `frontend/index.html`:
- Keep file under 3000 lines total
- Use the existing design token CSS variables (--cyan, --violet, --green, --amber, --red, --pink)
- All new views go into the `viewRenderers` object as `'section.view': { desc, render, onShow }`
- New action handlers must use the fallback pattern: try server → check if empty → use DemoData
- Canvas animations must store their `requestAnimationFrame` ID and cancel on view switch
- The `SimEngine.start()` runs at 2s intervals — do not add additional setIntervals for metrics
- Test both modes: with MCP server running (`:3000`) and without (demo-only)

## Distributed Deployment

| Target | Method | Status |
|--------|--------|--------|
| Local Mac (multi-tab) | BroadcastChannel | Working |
| RPi5 / NUC (bare metal) | Cross-compile + systemd | Deploy config in `deploy/` |
| Cloud GPU burst | SkyPilot | Planned (ADR-006 cloud escalation) |
| HuggingFace Jobs | Container | Planned |
| Browser workers | WASM compute pool | Working (ADR-009) |

## Support

- Documentation: https://github.com/ruvnet/claude-flow
- Issues: https://github.com/ruvnet/claude-flow/issues
