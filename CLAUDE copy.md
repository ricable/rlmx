# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

RLMX ("RuVix") is a **cognition kernel** — an OS-kernel-inspired runtime for LLM agents. It provides capability-secured syscall primitives that agents call instead of accessing arbitrary APIs. Written in Rust (edition 2021), async on Tokio.

The system supports three inference paths:
- **Cloud/Dev**: `VllmClient` → vLLM on Metal (Mac) or NVIDIA GPU (prod)
- **Edge**: `LocalEngine` → ruvllm CandleBackend on CPU/Metal/CUDA with GGUF models
- **Browser**: `@ruvector/ruvllm-wasm` → WebGPU/WASM SIMD (inference primitives + HTTP fallback for generation)

## Build & Development Commands

```bash
# Standard development
cargo build --workspace              # Build all 9 crates (no ruvllm engine)
cargo test --workspace               # Run all tests (134 tests)
cargo test -p rlmx-kernel            # Run tests for a single crate
cargo clippy --workspace             # Lint
cargo fmt --check                    # Check formatting
cargo fmt                            # Format code

# Run the CLI
cargo run -p rlmx-cli -- <cmd>       # Run CLI (query, ingest, serve, edge, etc.)
cargo run -p rlmx-cli -- serve --port 3000   # Start MCP server

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

## Workspace Structure

9 crates under `crates/`, with this dependency graph:

```
rlmx-cli (binary)
  ├── rlmx-kernel    (core — no rlmx deps)
  ├── rlmx-mcp       → rlmx-kernel, rlmx-rvf, rlmx-ruvllm
  ├── rlmx-rvf       (standalone)
  ├── rlmx-plugin    (standalone)
  └── rlmx-ruvllm    (standalone — optional ruvllm dep behind feature gate)

rlmx-rlm            (standalone — vLLM HTTP client)
rlmx-trm            (standalone — pure numeric NN)
rlmx-cognitive       (standalone — self-learning)
```

Additional project directories:
- `frontend/` — Single-page web UI + `@ruvector/ruvllm-wasm` WASM module
- `deploy/` — Systemd service template for RPi5 edge deployment
- `.cargo/` — Cross-compilation config for `aarch64-unknown-linux-gnu`

## Architecture

### Kernel (`rlmx-kernel`)
The core abstraction: a **12-syscall dispatch interface** (`Syscall` enum in `syscall.rs`). `KernelContext` holds `Arc<Mutex<T>>` handles to subsystems; `dispatch()` routes each variant:

- **Vector memory**: `VecInsert`/`VecSearch`/`VecDelete` — brute-force cosine similarity with 64-dim hash-based pseudo-embeddings, 3-tier Hot/Warm/Cold classification
- **Graph**: `GraphQuery`/`GraphCut`/`GraphDiffuse` — in-memory property graph, minimal Cypher parser, Karger/Stoer-Wagner min-cut, heat-kernel diffusion
- **Processes**: `ProcessFork`/`ProcessSend`/`ProcessRecv` — child processes with narrowed capability tokens, tokio mpsc channels
- **Proof**: `StateMutate` — SHA-256 witness-chained append-only audit log
- **Attention/Halting**: `AttentionSelect`, `HaltCheck`

### Capability Security (`capability.rs`)
HMAC-SHA256-signed `CapabilityToken` with per-syscall permission grants, TTL expiry, and hierarchical derivation (children cannot exceed parent permissions).

### MCP Server (`rlmx-mcp`)
JSON-RPC 2.0 server exposing **15 tools** (12 core + 3 edge). Two transports: stdio and HTTP (`POST /mcp`). Enforces initialization handshake before accepting `tools/list`/`tools/call`. RBAC resolved from server-side `token_roles` map — clients cannot self-escalate to Admin/System.

**Edge tools**: `rlmx_edge_generate` (Operator+), `rlmx_edge_status` (Viewer+), `rlmx_edge_load` (Engineer+).

### Scheduling Policies
- **RLM** (`rlmx-rlm`): Recursive LLM agent hierarchy with constrained 5-action grammar (`RETRIEVE`, `REASON`, `DELEGATE`, `COMMIT`, `FINAL`). Uses OpenAI-compatible `VllmClient`.
- **TRM** (`rlmx-trm`): 2-layer neural network with 3 streams (x/y/z), adaptive halting on confidence threshold or convergence.
- **Edge**: Local inference via ruvllm `CandleBackend`. Direct text generation (prompt → text), no agent grammar. For tiny models (0.5B–1.1B Q4) that can't follow structured output.
- **Scheduler** (`kernel/scheduler.rs`): Routes queries to RLM, TRM, Edge, Auto, or Hybrid strategy. When `edge_available=true`, auto-selects Edge for simple short queries.

### Edge Inference (`rlmx-ruvllm`)
Local GGUF model inference via ruvllm's `CandleBackend`. Feature-gated: compiles as stub without `ruvllm` feature.

- **`LocalEngine`**: wraps `CandleBackend` with `load_model()` + `generate()` + `load_tokenizer()`
- **`ModelManager`**: discovers GGUF files in `~/.rlmx/models/`
- **`EdgeConfig`**: backend selection (CPU/Metal/CUDA/Auto), memory budget, temperature, threads
- **Architecture detection**: auto-detects Llama/Qwen/Mistral/Phi/Gemma from model name
- **Tokenizer**: loads companion `<model>-tokenizer.json` file adjacent to GGUF

### Plugin System (`rlmx-plugin`)
`DomainPlugin` trait (~14 methods) is the primary extension point: ingest adapters, strategy preferences, safety constraints, graph schema, TRM models, embedding config. `SafetyEngine` enforces `ParameterBound` and `RateLimit` constraints. `EricssonRanPlugin` is the reference implementation.

### RVF Container (`rlmx-rvf`)
Sealed container format: manifest + typed segments + witness chain + optional Ed25519 signature. Supports COW branching via `BranchManager`. 6-role RBAC model (`Viewer`/`Operator`/`Engineer`/`Admin`/`Auditor`/`System`) shared with MCP layer.

### Cognitive Layer (`rlmx-cognitive`)
- **SONA**: micro-LoRA adaptation + EWC++ regularization, pattern bank with bounded capacity
- **DagOptimizer**: tracks strategy success rates from execution records
- **NervousSystem**: bio-inspired components (BTSP one-shot memory, hyperdimensional computing, winner-take-all, circadian controller, global workspace)

### Frontend (`frontend/index.html`)
Single-page dark-themed dashboard (vanilla JS, no framework). 8 views: Query, Ingest, Memory Stats, Graph Query, Edge Inference, List Tools, Raw JSON-RPC, Request Log. Communicates via JSON-RPC 2.0 to `POST /mcp`.

The Edge Inference view integrates `@ruvector/ruvllm-wasm` (v2.0.2) for in-browser capabilities:
- Feature detection (WASM, SIMD, WebGPU, SharedArrayBuffer, Web Workers)
- Chat template formatting (ChatML, Llama3, Mistral, Gemma, Phi) — runs fully in WASM
- Parallel matmul benchmark via `ParallelInference` with Web Workers
- Text generation via HTTP fallback to MCP server's `rlmx_edge_generate`

**WASM init**: must call `await mod.default()` before any other API. Do NOT call `mod.init()` after `default()` — it will error. `format()` on `ChatTemplateWasm` consumes message pointers — do NOT call `.free()` on messages after formatting.

## Conventions

- Shared mutable state uses `Arc<Mutex<T>>` or `Arc<RwLock<T>>`
- Per-crate error types via `thiserror` (e.g., `KernelError`, `RvfError`, `PluginError`, `RuvllmError`)
- `tracing` for structured logging — no `println!` in library code
- Tests are inline `#[cfg(test)]` blocks, no separate `tests/` directories
- Some crates pin their own dependency versions instead of using `[workspace.dependencies]`
- Feature gates: `ruvllm` and `metal` are opt-in. Without them, `LocalEngine` is a stub returning `NotAvailable`

## Strict Rules

1. **Never break the stub build**: `cargo build --workspace` (without features) MUST always compile clean. All ruvllm code behind `#[cfg(feature = "ruvllm")]`.
2. **Never add `println!` to library crates**: use `tracing::info!`, `tracing::warn!`, etc.
3. **Never bypass RBAC**: privileged roles (Admin, System) can only be assigned server-side via `token_roles` in `McpConfig`. The `_role` parameter rejects `admin`/`system`.
4. **Keep tests passing**: `cargo test --workspace` must pass all 134+ tests before any PR.
5. **Edge tools return `"status": "unavailable"`** when no engine is configured — never panic or error.
6. **GGUF models require companion tokenizers**: place `<model-name>-tokenizer.json` next to the `.gguf` file in `~/.rlmx/models/`.
7. **Don't modify the ruvllm git dependency source**: work around its API limitations in `rlmx-ruvllm`'s engine.rs.
8. **MCP initialization handshake is mandatory**: tools/list and tools/call are rejected before `initialize` is called.

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RLMX_EDGE_MODEL` | GGUF model name (without .gguf extension) | unset |
| `RLMX_MODEL_DIR` | Directory containing GGUF + tokenizer files | `~/.rlmx/models` |

## Key File Locations

| Path | Purpose |
|------|---------|
| `crates/rlmx-kernel/src/scheduler.rs` | Strategy enum (Rlm/Trm/Edge/Auto/Hybrid) |
| `crates/rlmx-mcp/src/tools.rs` | All 15 MCP tool definitions and handlers |
| `crates/rlmx-mcp/src/server.rs` | RBAC enforcement, tool dispatch, init handshake |
| `crates/rlmx-ruvllm/src/engine.rs` | LocalEngine — CandleBackend wrapper |
| `crates/rlmx-cli/src/main.rs` | CLI entry point, cmd_serve, cmd_edge |
| `frontend/index.html` | Web dashboard with Edge AI view |
| `frontend/ruvllm-wasm/` | @ruvector/ruvllm-wasm v2.0.2 assets |
| `deploy/rlmx-edge.service` | Systemd unit for RPi5 edge deployment |
